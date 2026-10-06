use std::collections::{HashMap, VecDeque};
use std::io::{Error, ErrorKind, Result};
use std::fs::*;
use std::io::ErrorKind::NotADirectory;
use std::path::{Path, PathBuf};
use std::thread;
use std::sync::{Arc, Mutex, mpsc, mpsc::{Sender, Receiver}};

type InodeId = usize;
type ScanTask = InodeId;
type CompletedTask = InodeId;

struct ScannerState {
    next_inode_id: InodeId,
    inodes: HashMap<InodeId, Arc<Mutex<ScannerInode>>>,
    parent_by_id: HashMap<InodeId, InodeId>,
    pending_count_by_id: HashMap<InodeId, usize>,
    work_queue: VecDeque<ScanTask>,
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
    size: u64,
    detail: ScannerInodeDetail,
}

enum ScannerInodeDetail {
    Reg,
    Sym,
    Dir(Vec<Arc<Mutex<ScannerInode>>>),
}

fn scan(path: &Path, worker_pool_size: usize) -> Result<ScannerInode> {
    todo!()
}

fn entry_to_init_inode(dir_entry: &DirEntry) -> ScannerInode {
    let entry_type = dir_entry.file_type().unwrap();
    let entry_path = dir_entry.path();
    let mut entry_size = 0;
    let mut entry_detail = ScannerInodeDetail::Reg;

    if entry_type.is_file() {
	entry_size = dir_entry.metadata().unwrap().len();
	entry_detail = ScannerInodeDetail::Reg;
    } else if entry_type.is_symlink() {
	entry_size = symlink_metadata(entry_path.clone()).unwrap().len();
	entry_detail = ScannerInodeDetail::Sym;
    } else if entry_type.is_dir() {
	entry_size = 0;
	entry_detail = ScannerInodeDetail::Dir(Vec::new());
    }

    ScannerInode {
	path: entry_path.clone(),
	size: entry_size,
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
	   sender: Sender<InodeId>) -> () {
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
	    - Update the size of parent
	    - Insert new inode into the record
	    - Insert new parent-child relation
	    - Insert new inode to children list of parent
	     */
	    dir_inode.size += dir_entry_inode.size;
	    let dir_entry_inode_arc = Arc::new(Mutex::new(dir_entry_inode));
	    {
		let mut state = (&state_arc).lock().unwrap();
		(*state).inodes.insert(dir_entry_id, dir_entry_inode_arc.clone());
		(*state).parent_by_id.insert(dir_entry_id, dir_id);
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
	    for subdir_id in subdir_ids {
		(*state).work_queue.push_back(subdir_id);
	    }
	}
    }
}

fn collector(state: ScannerState) -> () {
}
