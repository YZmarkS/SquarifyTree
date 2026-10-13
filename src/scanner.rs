use std::collections::{HashMap, VecDeque};
use std::fs::*;
use std::io::ErrorKind::*;
use std::io::{Error, ErrorKind::NotADirectory};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};
use std::thread;

use crate::inode::*;

type InodeIndex = usize;
type Pending = usize;

pub const ROOT_ID: InodeIndex = 0;

pub struct WorkState {
    pendings: HashMap<InodeIndex, Pending>,
    job_queue: VecDeque<InodeIndex>,
}

#[derive(Clone)]
struct ScannerInode {
    path: PathBuf,
    usage: DiskUsage,
    detail: ScannerInodeDetail,
    parent: InodeIndex,
}

#[derive(Clone)]
enum ScannerInodeDetail {
    Reg,
    Sym,
    Dir(Vec<AM<ScannerInode>>),
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
            None => continue,
        };

        /* Scan directory */
        let dir_path = {
            let inodes_arc_ = (&inodes_arc).clone();
            let inodes = inodes_arc_.lock().unwrap();
            let dir_inode = inodes.get(dir_index).unwrap().lock().unwrap();
            dir_inode.path.clone()
        };

        let mut usage_subtotal = 0;
        let mut subdir_count = 0;

        let mut children = Vec::new();

        for entry_result in read_dir(dir_path).unwrap() {
            let entry_inode = entry_to_inode(&entry_result.unwrap(), dir_index);

            usage_subtotal += entry_inode.usage;
            subdir_count += if entry_inode.is_dir() { 1 } else { 0 };
            children.push(entry_inode);
        }

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
        {
            let inodes_arc_ = (&inodes_arc).clone();
            let mut inodes = inodes_arc_.lock().unwrap();
            let mut dir_inode = inodes
		.get_mut(dir_index).unwrap()
		.lock().unwrap();
            dir_inode.usage = usage_subtotal;
        }
        /* Update pending */
        {
            (&state_arc)
                .lock()
                .unwrap()
                .pendings
                .insert(dir_index, subdir_count);
        }

        /* Add inode to total collection and as child */
        let inodes_arc_ = (&inodes_arc).clone();
        let mut inodes = inodes_arc_.lock().unwrap();
        let dir_inode_arc = inodes.get_mut(dir_index).unwrap().clone();
        let mut dir_inode = dir_inode_arc.lock().unwrap();
        let dir_children = if let ScannerInodeDetail::Dir(dir_children) = &mut dir_inode.detail {
            dir_children
        } else {
            panic!()
        };

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

        /* Send message to collector */
        sender.send((dir_index, subdir_count)).unwrap();

        /* Add new jobs */
        let job_queue = &mut state_arc.lock().unwrap().job_queue;
        for job in jobs {
            job_queue.push_back(job);
        }
    }
    return ();
}

fn collector(inodes_arc: Arc<Mutex<Vec<Arc<Mutex<ScannerInode>>>>>,
	     root_collected: Arc<Mutex<bool>>,
	     state_arc: Arc<Mutex<WorkState>>,
	     receiver: Receiver<(InodeIndex, Pending)>,
) -> () {
    loop {
        let (dir_index, pending) = receiver.recv().unwrap();

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
        This means a "leaf directory", aka a directory with no more subdirectory
        has finished scanning. The information collected by the leaf directory
        can now be propagated back to its parent. The parent directory is pending
        on one less subtask.
        Completion may propagate to grandparent, a loop is used to account for
        that. This goes all the way until the parent is the root of the scan,
        which needs some manual processing.
         */
        if pending > 0 {
            continue;
        } else if dir_index == ROOT_ID {
            *(root_collected.lock().unwrap()) = true;
            break;
        } else {
            let mut state = state_arc.lock().unwrap();

            let mut usage_to_add = (*state).inodes.get(&dir_index).unwrap().lock().unwrap().usage;
            let mut parent_id = (*state).parent_id_by_id.get(&dir_index).unwrap().to_owned();
            let mut parent_inode_arc = (*state).inodes.get(&parent_id).unwrap().clone();
            let mut parent_pending = (*state).pending_count_by_id.get_mut(&parent_id).unwrap();
            let mut propagate = true;
            while propagate && parent_id != ROOT_ID {
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
            if parent_id == ROOT_ID {
                (*parent_inode_arc).lock().unwrap().usage += usage_to_add;
                (*parent_pending) -= 1;

                if *parent_pending == 0 {
                    *(root_collected.lock().unwrap()) = true;
                    break;
                }
            }
        }
    }
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
                    children.push(Rc::new(inode));
                }
                children.sort_by_key(|inode| inode.usage);
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

pub fn scan_directory_with_pool(
    path: &Path,
    worker_pool_size: usize,
) -> std::io::Result<Rc<Inode>> {
    let path_metadata = symlink_metadata(path)?;
    if !path_metadata.is_dir() {
        return Err(Error::from(NotADirectory));
    }

    if worker_pool_size == 1 || worker_pool_size == 2 {
        scan_single_threaded(path)
    } else {
        let end_state_arc = scan_multi_threaded(path, worker_pool_size - 1)?;
        let end_state = match end_state_arc.lock() {
            Ok(end_state) => Ok(end_state),
            Err(_) => Err(Error::from(Deadlock)),
        }?;
        let root_inode_arc = match end_state.inodes.get(&ROOT_ID) {
            Some(arc) => Ok(arc),
            None => Err(Error::from(NotFound)),
        }?
        .to_owned();
        match Inode::try_from(root_inode_arc) {
            Ok(inode) => Ok(Rc::new(inode)),
            Err(_) => Err(Error::from(Deadlock)),
        }
    }
}

pub fn scan_multi_threaded(
    path: &Path,
    worker_pool_size: usize,
) -> std::io::Result<Arc<Mutex<ScannerState>>> {
    let mut state = ScannerState {
        next_inode_id: ROOT_ID,
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
        collector(collector_state_arc, collector_root_collected, receiver);
    });

    for scanner_handle in scanner_handles {
        scanner_handle.join().unwrap();
    }
    collector_handle.join().unwrap();

    Ok(state_arc)
}

pub fn scan_single_threaded(path: &Path) -> std::io::Result<Rc<Inode>> {
    use crate::inode::InodeDetail::*;

    let inode_path = path.to_path_buf();
    let metadata = symlink_metadata(path)?;

    if metadata.is_file() {
        Ok(Rc::new(Inode {
            path: inode_path,
            usage: DiskUsage::from(metadata.len()),
            detail: Reg,
        }))
    } else if metadata.is_symlink() {
        Ok(Rc::new(Inode {
            path: inode_path,
            usage: DiskUsage::from(metadata.len()),
            detail: Sym,
        }))
    } else {
        // metadata.is_dir()
        let mut inode_usage: DiskUsage = 0;
        let mut children = Vec::new();
        for dir_entry in read_dir(path)? {
            let dir_entry = match dir_entry {
                Ok(dir_entry) => dir_entry,
                Err(_) => continue,
            };
            let child = scan_single_threaded(&dir_entry.path())?;
            inode_usage += child.usage;
            children.push(child);
        }
        children.sort_by_key(|inode_rc| (*inode_rc).usage);
        children.reverse();
        Ok(Rc::new(Inode {
            path: inode_path,
            usage: inode_usage,
            detail: Dir(children),
        }))
    }
}
