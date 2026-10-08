use std::rc::{Rc};
use std::collections::{HashMap, VecDeque};
use std::io::{Result, Error, ErrorKind::{NotADirectory}};
use std::fs::*;
use std::path::{Path, PathBuf};
use std::thread;
use std::sync::{Arc, Mutex, mpsc, mpsc::{Sender, Receiver}};

use crate::inode::*;

type InodeId = usize;
type Pending = usize;

pub const FIRST_ID: InodeId = 0;

pub struct ScannerState {
    next_inode_id: InodeId,
    inodes: HashMap<InodeId, Arc<Mutex<ScannerInode>>>,
    parent_id_by_id: HashMap<InodeId, InodeId>,
    pending_count_by_id: HashMap<InodeId, usize>,
    work_queue: VecDeque<InodeId>,
}

impl ScannerState {
    fn get_next_inode_id(&mut self) -> InodeId {
	let result = self.next_inode_id;
	self.next_inode_id += 1;
	result
    }
}

struct ScannerInode {
    path: PathBuf,
    usage: DiskUsage,
    detail: ScannerInodeDetail,
}

enum ScannerInodeDetail {
    Reg,
    Sym,
    Dir(Vec<Arc<Mutex<ScannerInode>>>),
}

pub fn scan(path: &Path, worker_pool_size: usize) ->
    Result<Arc<Mutex<ScannerState>>> {
    let path_metadata = metadata(path)?;
    if !path_metadata.is_dir() {
	return Err(Error::from(NotADirectory));
    }

    let mut state = ScannerState {
	next_inode_id: FIRST_ID,
	inodes: HashMap::new(),
	parent_id_by_id: HashMap::new(),
	pending_count_by_id: HashMap::new(),
	work_queue: VecDeque::new(),
    };

    let root_id = state.get_next_inode_id();
    let root_inode = ScannerInode {
	path: path.to_path_buf(),
	usage: 0,
	detail: ScannerInodeDetail::Dir(Vec::new()),
    };
    let root_inode_arc = Arc::new(Mutex::new(root_inode));

    state.inodes.insert(root_id, root_inode_arc);
    state.work_queue.push_back(root_id);

    let state_arc = Arc::new(Mutex::new(state));
    let root_collected = Arc::new(Mutex::new(false));
    let (sender, receiver) = mpsc::channel();

    let mut scanner_handles = vec![];
    for _ in 0..worker_pool_size {
	let state_arc = Arc::clone(&state_arc);
	let root_collected = Arc::clone(&root_collected);
	let sender = sender.clone();
	let scanner_handle = thread::spawn(move || {
	    scanner(state_arc, root_collected, sender);
	});
	scanner_handles.push(scanner_handle);
    }

    let collector_state_arc = Arc::clone(&state_arc);
    let collector_root_collected = Arc::clone(&root_collected);
    let collector_handle = thread::spawn(move || {
	collector(collector_state_arc,
		  collector_root_collected,
		  receiver);
    });

    for scanner_handle in scanner_handles {
	scanner_handle.join().unwrap();
    }
    collector_handle.join().unwrap();

    Ok(state_arc)
}

fn entry_to_init_inode(dir_entry: &DirEntry) -> ScannerInode {
    let entry_type = dir_entry.file_type().unwrap();
    let entry_path = dir_entry.path();
    let mut entry_usage: u128 = 0;
    let mut entry_detail = ScannerInodeDetail::Reg;

    if entry_type.is_file() {
	entry_usage = u128::from(dir_entry.metadata().unwrap().len());
	entry_detail = ScannerInodeDetail::Reg;
    } else if entry_type.is_symlink() {
	entry_usage = u128::from(symlink_metadata(entry_path.clone()).unwrap().len());
	entry_detail = ScannerInodeDetail::Sym;
    } else if entry_type.is_dir() {
	entry_usage = 0;
	entry_detail = ScannerInodeDetail::Dir(Vec::new());
    }

    ScannerInode {
	path: entry_path.clone(),
	usage: entry_usage,
	detail: entry_detail,
    }
}

fn get_next_task_inode(state_arc: &Arc<Mutex<ScannerState>>) ->
    Option<(InodeId, Arc<Mutex<ScannerInode>>)> {
    let mut state = state_arc.lock().unwrap();

    let task_id = (*state).work_queue.pop_front()?;

    let task_inode = (*state).inodes.get_mut(&task_id).unwrap().clone();

    Some((task_id, task_inode))
}

fn get_next_inode_id(state_arc: &Arc<Mutex<ScannerState>>) -> InodeId {
    let mut state = state_arc.lock().unwrap();

    state.get_next_inode_id()
}

fn scanner(state_arc: Arc<Mutex<ScannerState>>,
	   root_collected: Arc<Mutex<bool>>,
	   sender: Sender<(InodeId, Pending)>) -> () {
    loop {
	if *root_collected.lock().unwrap() {
	    break
	}

	/*
	It is possible there are no task because some other thread is adding
	it into the queue.
	 */
	let (dir_id, dir_inode_arc) = match get_next_task_inode(&state_arc) {
	    Some((dir_id, dir_inode_arc)) => (dir_id, dir_inode_arc),
	    None => continue,
	};

	let mut dir_inode = dir_inode_arc.lock().unwrap();

	let mut subdir_ids: Vec<InodeId> = Vec::new();

	for dir_entry_result in read_dir(&dir_inode.path).unwrap() {
	    let dir_entry = dir_entry_result.unwrap();
	    let dir_entry_inode = entry_to_init_inode(&dir_entry);
	    let dir_entry_id = get_next_inode_id(&state_arc);

	    /*
	    Some information can be added now
	    - Update the usage of parent
	    - Insert new inode into the record
	    - Insert new parent-child relation
	    - Insert new inode to children list of parent
	     */
	    dir_inode.usage += dir_entry_inode.usage;
	    let dir_entry_inode_arc = Arc::new(Mutex::new(dir_entry_inode));
	    {
		let mut state = (&state_arc).lock().unwrap();
		(*state).inodes.insert(dir_entry_id, dir_entry_inode_arc.clone());
		(*state).parent_id_by_id.insert(dir_entry_id, dir_id);
	    }
	    if let ScannerInodeDetail::Dir(children) = &mut (*dir_inode).detail {
		children.push(dir_entry_inode_arc.clone());
	    }

	    /*
	    Track subdirectories
	     */
	    if dir_entry.file_type().unwrap().is_dir() {
		subdir_ids.push(dir_entry_id);
	    }

	}

	/*
	Note we did not add the subdirectorys as tasks when they are discovered
	If we added then, if a thread somehow finished scanning the subdirectory
	before this thread, then the collector will be confused because there is
	the parent directory does not have a pending number tracked.
	 */
	{
	    let mut state = (&state_arc).lock().unwrap();
	    (*state).pending_count_by_id.insert(dir_id, subdir_ids.len());
	    for subdir_id in &subdir_ids {
		(*state).work_queue.push_back(subdir_id.to_owned());
	    }
	}
	sender.send((dir_id, subdir_ids.len())).unwrap();
    }
}

fn collector(state_arc: Arc<Mutex<ScannerState>>,
	     root_collected: Arc<Mutex<bool>>,
	     receiver: Receiver<(InodeId, Pending)>) -> () {
    loop {
	let (dir_id, pending_count) = receiver.recv().unwrap();

	/* When the collector receives a message, it is because a worker have
	initialized an directory inode, and have reported the directory id, and
	the number of subdirectories in the directory.
	We remind the reader during the scanning process, the collector will
	receive exactly one message for each directory in the scan space.
	Furthermore, the order of the messages will behave in "pseudo BFS way",
	that we will always receive a message for the parent directory before
	the message for the subdirectory. This is a consequence of the internal
	logic of the worker.

	Branch 1: If the directory has non-zero subdirectory
	The directory data is pending on the information of subdirectory.
	We track the number of pending directory in scanner state.

	Branch 2: The directory has no subdirectory, and is the root directory
	This means the message received is the first message (the BFS invariant),
	Because the root does not have subdirectory, the scan is complete.
	Notify all worker threads through mutex.

	Branch 3: The directory has no subdirectory, and is not the root
	This means a "leaf directory", aka a directory with no more subdirectory
	has finished scanning. The information collected by the leaf directory
	can now be propagated back to its parent. The parent directory is pending
	on one less subtask.
	Completion may propagate to grandparent, a loop is used to account for
	that. This goes all the way until the parent is the root of the scan,
	which needs some manual processing.
	 */
	if pending_count > 0 {
	    let mut state = state_arc.lock().unwrap();
	    (*state).pending_count_by_id.insert(dir_id, pending_count).unwrap();
	} else if dir_id == FIRST_ID {
	    *(root_collected.lock().unwrap()) = true;
	    break;
	} else {
	    let mut state = state_arc.lock().unwrap();

	    let mut usage_to_add = (*state).inodes.get(&dir_id).unwrap().lock().unwrap().usage;
	    let mut parent_id = (*state).parent_id_by_id.get(&dir_id).unwrap().to_owned();
	    let mut parent_inode_arc = (*state).inodes.get(&parent_id).unwrap().clone();
	    let mut parent_pending = (*state).pending_count_by_id.get_mut(&parent_id).unwrap();
	    let mut propagate = true;
	    while propagate && parent_id != FIRST_ID {

		(*parent_inode_arc).lock().unwrap().usage += usage_to_add;
		(*parent_pending) -= 1;

		propagate = false;

		if (*parent_pending) == 0 {
		    propagate = true;
		    usage_to_add = (*parent_inode_arc).lock().unwrap().usage;
		    parent_id = (*state).parent_id_by_id.get(&parent_id).unwrap().to_owned();
		    parent_inode_arc = (*state).inodes.get(&parent_id).unwrap().clone();
		    parent_pending = (*state).pending_count_by_id.get_mut(&parent_id).unwrap();
		}
	    }
	    if parent_id == FIRST_ID {
		(*parent_inode_arc).lock().unwrap().usage += usage_to_add;
		(*parent_pending) -= 1;

		if *parent_pending == 0 {
		    *(root_collected.lock().unwrap()) = true;
		    break
		}
	    }
	}
    }
}


impl From<Arc<Mutex<ScannerInode>>> for Inode {
    fn from(scanner_inode_arc: Arc<Mutex<ScannerInode>>) -> Self {
	let scanner_inode = scanner_inode_arc.lock().unwrap();
	let path = scanner_inode.path.clone();
	let usage = scanner_inode.usage;

	let detail = match &(*scanner_inode).detail {
	    ScannerInodeDetail::Reg => InodeDetail::Reg,
	    ScannerInodeDetail::Sym => InodeDetail::Sym,
	    ScannerInodeDetail::Dir(children) => {
		let mut new_children = Vec::new();
		for child in children {
		    new_children.push(Rc::new(Inode::from(child.to_owned())));
		}
		new_children.sort_by_key(|inode| inode.usage);
		new_children.reverse();
		InodeDetail::Dir(new_children)
	    },
	};

	return Inode {
	    path,
	    usage,
	    detail,
	}
    }
}

pub fn path_to_inode(path: &Path, worker_pool_size: usize) -> Inode {
    let scan_final_state_arc = scan(path, worker_pool_size).unwrap();
    let scan_final_state = scan_final_state_arc.lock().unwrap();
    let root_scanner_inode_arc = scan_final_state.inodes.get(&FIRST_ID)
	.unwrap()
	.to_owned();
    Inode::from(root_scanner_inode_arc)
}
