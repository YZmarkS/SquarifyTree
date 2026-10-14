use std::collections::{HashMap, VecDeque};
use std::fs::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::io::ErrorKind::*;
use std::io::{Error, ErrorKind::NotADirectory};
use std::path::{Path, PathBuf};

use std::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};
use std::thread;

use crate::inode::*;

type InodeIndex = usize;
type Pending = usize;

pub const ROOT_INDEX: InodeIndex = 0;

pub struct WorkState {
    pendings: HashMap<InodeIndex, Pending>,
    job_queue: VecDeque<InodeIndex>,
}

#[derive(Clone)]
pub struct ScannerInode {
    path: PathBuf,
    usage: DiskUsage,
    detail: ScannerInodeDetail,
    parent: InodeIndex,
}

#[derive(Clone)]
enum ScannerInodeDetail {
    Reg,
    Sym,
    Dir(Vec<Arc<Mutex<ScannerInode>>>),
}

impl ScannerInode {
    fn is_reg(&self) -> bool {
        self.detail.is_reg()
    }
    fn is_sym(&self) -> bool {
        self.detail.is_sym()
    }
    fn is_dir(&self) -> bool {
        self.detail.is_dir()
    }
}

impl ScannerInodeDetail {
    fn is_reg(&self) -> bool {
        match self {
            ScannerInodeDetail::Reg => true,
            _ => false,
        }
    }
    fn is_sym(&self) -> bool {
        match self {
            ScannerInodeDetail::Sym => true,
            _ => false,
        }
    }
    fn is_dir(&self) -> bool {
        match self {
            ScannerInodeDetail::Dir(_) => true,
            _ => false,
        }
    }
}

fn entry_to_inode(dir_entry: &DirEntry, parent: InodeIndex) -> ScannerInode {
    /* According to documentation, this does not traverse symlink */
    let entry_type = dir_entry.file_type().unwrap();
    let entry_path = dir_entry.path();
    let mut entry_usage: DiskUsage = 0;
    let mut entry_detail = ScannerInodeDetail::Reg;

    if entry_type.is_file() {
        entry_usage = DiskUsage::from(dir_entry.metadata().unwrap().len());
        entry_detail = ScannerInodeDetail::Reg;
    } else if entry_type.is_symlink() {
        entry_usage = DiskUsage::from(symlink_metadata(entry_path.clone()).unwrap().len());
        entry_detail = ScannerInodeDetail::Sym;
    } else if entry_type.is_dir() {
        entry_usage = 0;
        entry_detail = ScannerInodeDetail::Dir(Vec::new());
    }

    ScannerInode {
        path: entry_path.clone(),
        usage: entry_usage,
        detail: entry_detail,
        parent: parent,
    }
}

fn get_inode_arc_by_index(inodes_arc: &Arc<Mutex<Vec<Arc<Mutex<ScannerInode>>>>>,
			  index: InodeIndex) -> Option<Arc<Mutex<ScannerInode>>> {
    let mut inodes = inodes_arc.lock().unwrap();
    let inode = inodes.get(index)?.clone();
    Some(inode)
}

fn scanner(
    inodes_arc: Arc<Mutex<Vec<Arc<Mutex<ScannerInode>>>>>,
    root_collected: Arc<Mutex<bool>>,
    state_arc: Arc<Mutex<WorkState>>,
    sender: Sender<(InodeIndex, Pending)>,
) -> () {
    loop {
        /* Check there is job to be done */
        if *(root_collected.lock().unwrap()) {
            break;
        }

        let dir_index = match (state_arc.lock().unwrap().job_queue).pop_front() {
            Some(dir_index) => dir_index,
            None => {
		continue;
	    }
        };



	let dir_inode_arc = get_inode_arc_by_index(&inodes_arc, dir_index).unwrap();
	let mut dir_inode = dir_inode_arc.lock().unwrap();

	let dir_path = (*dir_inode).path.to_path_buf();

	println!("Work {} start a loop: {:?}", dir_index, dir_path);

        let mut usage_subtotal = 0;
        let mut subdir_count = 0;

        let mut children = Vec::new();

        for entry_result in read_dir(dir_path).unwrap() {
            let entry_inode = entry_to_inode(&entry_result.unwrap(), dir_index);

            usage_subtotal += entry_inode.usage;
            subdir_count += if entry_inode.is_dir() { 1 } else { 0 };
            children.push(entry_inode);
        }
	println!("Work {} scanned the directory", dir_index);

        /* Processing the Result of Scanning

        The order of operation is very important. This is a result of using a
        mutlithreading approach.

        - Disk usage :: Can we updated whenever, as long as the update is atomic
                        We choose to update it at earliest possible time.
        - Pending :: Must be inserted before the "job of any subdirectory is
                     completed and sent to collector". This is because the
                     collector needs the number of pending subdirectories to
                     decide whether to propagate information up to ancestors.
	             We choose to insert pending information before any new job
	             is inserted into the queue, thus also before any subjob is
	             completed.
        - Inodes list ::
        - Current directory children list ::
	- Send message to collector ::
        - Job queue :: Must be updated AFTER 1. pending is inserted, and
                       2. inodes list is updated. This is because each job
                       identifies an index in the inodes list, containing the
                       inode need to be worked on.
        */

        /* Update usage subtotal */
	dir_inode.usage += usage_subtotal;

        /* Update pending */
	let mut state = state_arc.lock().unwrap();

	state.pendings.insert(dir_index, subdir_count);

	println!("Work {} updated usage and pending", dir_index);

        /* Add inode to total collection and as child */
        let dir_children = if let ScannerInodeDetail::Dir(dir_children) = &mut dir_inode.detail {
            dir_children
        } else {
            panic!()
        };
	let mut inodes = inodes_arc.lock().unwrap();

        let mut jobs = Vec::new();

        for child in children {
            let index = inodes.len();

            if child.is_dir() {
                jobs.push(index);
            }

            let child_arc = Arc::new(Mutex::new(child));

            inodes.push(child_arc.clone());
            dir_children.push(child_arc.clone());
        }

	println!("Work {} pushed new inodes into collection and parent", dir_index);

        /* Send message to collector */
        sender.send((dir_index, subdir_count)).unwrap();


	println!("Work {} sent message to collector", dir_index);

        /* Add new jobs */
        for job in jobs {
	    state.job_queue.push_back(job);
        }

	println!("Work {} added the jobs", dir_index);


	println!("Work {} done", dir_index);
    }
    println!("Work exiting");
    return ();
}

fn collector(inodes_arc: Arc<Mutex<Vec<Arc<Mutex<ScannerInode>>>>>,
	     root_collected: Arc<Mutex<bool>>,
	     state_arc: Arc<Mutex<WorkState>>,
	     receiver: Receiver<(InodeIndex, Pending)>,
) -> () {
    loop {
        let (leaf_index, pending) = receiver.recv().unwrap();


	println!("{:?}, {:?}, {:?}",
		 leaf_index,
		 inodes_arc.lock().unwrap().get(leaf_index).unwrap().lock().unwrap().path,
		 pending);
        /* When the collector receives a message, it is because a worker have
        initialized an directory inode. The worker is reporting the directory
	index and the number of subdirectories in the directory.
        Reminder: during the scanning process, the collector will receive
	exactly one message for each directory in the scan space. Furthermore,
	the order of the messages will behave in "pseudo BFS way", that we will
	always receive a message for the parent directory before the message for
	the subdirectory. This is a consequence of the internal logic of the
	worker.

        Branch 1: If the directory has non-zero subdirectory
	There is nothing to be done, because the worker inserted the number of
	pending subdirectory already.

        Branch 2: The directory has no subdirectory, and is the root directory
        This means the message received is the first message (the BFS invariant),
        Because the root does not have subdirectory, the scan is complete.
        Notify all worker threads through mutex.

        Branch 3: The directory has no subdirectory, and is not the root
        This means a "leaf directory", a directory with no subdirectory,
        has finished scanning. The information collected by the leaf directory
        can now be propagated back to its parent. The parent directory is pending
        on one less subtask.
        Completion may propagate to grandparent, a loop is used to account for
        that. This goes all the way until the parent is the root of the scan,
        which needs some manual processing.
         */
        if pending > 0 {
            continue;
        } else if leaf_index == ROOT_INDEX {
            *(root_collected.lock().unwrap()) = true;
            break;
        } else {
	    let mut state = state_arc.lock().unwrap();





	    println!("Collector: === Before acquire inodes lock ===");
	    // TODO I cannot acquire this lock! Why?
	    let inodes = inodes_arc.lock().unwrap();
	    println!("Collector: === After acquire inodes lock ===");




	    let leaf_dir = inodes.get(leaf_index).unwrap().lock().unwrap();

	    let mut usage_to_add = leaf_dir.usage;
	    let mut parent_index = leaf_dir.parent;
	    let mut parent_inode_arc = inodes.get(parent_index).unwrap();
	    let mut parent_pending = (*state).pendings.get_mut(&parent_index).unwrap();
	    let mut propagate = true;

	    while propagate && parent_index != ROOT_INDEX {
		(*parent_inode_arc).lock().unwrap().usage += usage_to_add;
		(*parent_pending) -= 1;

		propagate = false;

		if (*parent_pending) == 0 {
		    propagate = true;
		    usage_to_add = (*parent_inode_arc).lock().unwrap().usage;
		    parent_index = (*parent_inode_arc).lock().unwrap().parent;
		    parent_inode_arc = inodes.get(parent_index).unwrap();
		    parent_pending = (*state).pendings.get_mut(&parent_index).unwrap();
		}
	    }

	    if propagate && parent_index == ROOT_INDEX {
		(*parent_inode_arc).lock().unwrap().usage += usage_to_add;
		(*parent_pending) -= 1;

		if (*parent_pending) == 0 {
		    *(root_collected.lock().unwrap()) = true;
		    break;
		}
	    }

        }
    }
    return ();
}

impl TryFrom<Arc<Mutex<ScannerInode>>> for Inode {
    type Error = std::sync::PoisonError<ScannerInode>;
    fn try_from(
        scanner_inode_arc: Arc<Mutex<ScannerInode>>,
    ) -> Result<Self, std::sync::PoisonError<ScannerInode>> {
        let scanner_inode = scanner_inode_arc.lock().unwrap();
        let path = scanner_inode.path.clone();
        let usage = scanner_inode.usage;

        let detail = match &(*scanner_inode).detail {
            ScannerInodeDetail::Reg => InodeDetail::Reg,
            ScannerInodeDetail::Sym => InodeDetail::Sym,
            ScannerInodeDetail::Dir(rc_children) => {
                let mut children = Vec::new();
                for rc_child in rc_children {
                    let inode = Inode::try_from(rc_child.to_owned())?;
                    children.push(Rc::new(RefCell::new(inode)));
                }
                children.sort_by_key(|inode| inode.borrow().usage);
                children.reverse();
                InodeDetail::Dir(children)
            }
        };

        return Ok(Inode {
            path,
            usage,
            detail,
        });
    }
}

pub fn path_to_inode(path: &Path) {
    let mut inodes = Vec::new();
    inodes.push(Arc::new(Mutex::new( ScannerInode {
	path: path.to_path_buf(),
	usage: 0,
	detail: ScannerInodeDetail::Dir(Vec::new()),
	parent: ROOT_INDEX,
    })));
    let inodes_arc = Arc::new(Mutex::new(inodes));
    let root_collected = Arc::new(Mutex::new(false));
    let state_arc = Arc::new(Mutex::new(WorkState { pendings: HashMap::new(),
						    job_queue: VecDeque::from([0]) }));
    let (sender, receiver) = mpsc::channel();

    let mut scanner_handles = vec![];
    for _ in 0..2 {
	let inodes_arc = Arc::clone(&inodes_arc);
	let root_collected = Arc::clone(&root_collected);
	let state_arc = Arc::clone(&state_arc);
	let sender = sender.clone();
	let scanner_handle = thread::spawn(move || {
	    scanner(inodes_arc, root_collected, state_arc, sender)
	});
	scanner_handles.push(scanner_handle);
    }

    let collector_handle =
    {
	let inodes_arc = Arc::clone(&inodes_arc);
	let root_collected = Arc::clone(&root_collected);
	let state_arc = Arc::clone(&state_arc);
	thread::spawn(move || {
	    collector(inodes_arc, root_collected, state_arc, receiver)
	})
    };

    for scanner_handle in scanner_handles {
	scanner_handle.join().unwrap();
    }
    collector_handle.join().unwrap();

    println!("Collected all");

    let root_inode = Inode::try_from(inodes_arc
				     .lock()
				     .unwrap()
				     .get(ROOT_INDEX)
				     .unwrap()
				     .to_owned())
	.unwrap();

    print_inodes(&root_inode, 0);
}
