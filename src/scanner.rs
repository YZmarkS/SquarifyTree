use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fs::*;
use std::path::PathBuf;
use std::rc::Rc;

use std::sync::RwLock;
use std::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};
use std::thread;

use crate::inode::*;

type InodeIndex = usize;
type Pending = usize;

pub struct ScanJob {
    parent_id: Option<InodeIndex>,
    directory_id: InodeIndex,
}

#[derive(Debug, Clone)]
pub struct ScannerInode {
    path: PathBuf,
    usage: DiskUsage,
    detail: ScannerInodeDetail,
}

pub struct DiscoverSignal {
    parent_id: Option<InodeIndex>,
    directory_id: InodeIndex,
    pending: Pending,
}

#[derive(Debug, Clone)]
pub enum ScannerInodeDetail {
    Reg,
    Sym,
    Dir(Vec<Arc<Mutex<ScannerInode>>>),
}

fn scanner(
    directory_inodes: Arc<Mutex<Vec<Arc<Mutex<ScannerInode>>>>>,
    job_queue: Arc<Mutex<VecDeque<ScanJob>>>,
    base_collected: Arc<RwLock<Option<Arc<Mutex<ScannerInode>>>>>,
    sender: Sender<DiscoverSignal>,
) -> () {
    loop {
        /* Check there is job to be done */
        if base_collected.read().unwrap().is_some() {
            break;
        }

        let ScanJob {
            parent_id,
            directory_id,
        } = match (job_queue.lock().unwrap()).pop_front() {
            Some(job) => job,
            None => continue,
        };

        let directory_path = {
	    let directory_inodes = directory_inodes.lock().unwrap();
            let directory_inode = directory_inodes.get(directory_id).unwrap();
            directory_inode.lock().unwrap().path.clone()
        };

        let mut usage = 0;
        let mut pending = 0;
        let mut entry_inodes: Vec<Arc<Mutex<ScannerInode>>> = Vec::new();
        let mut sub_directory_inodes: Vec<Arc<Mutex<ScannerInode>>> = Vec::new();

        for entry_result in read_dir(&directory_path).unwrap() {
            let entry = entry_result.unwrap();

            let entry_metadata = entry.metadata().unwrap();
            let entry_type = entry.file_type().unwrap();
            let entry_path = entry.path();

            let entry_inode = if entry_type.is_file() {
                let entry_usage = u128::from(entry_metadata.len());
                usage += entry_usage;
                Arc::new(Mutex::new(ScannerInode {
                    path: entry_path,
                    usage: entry_usage,
                    detail: ScannerInodeDetail::Reg,
                }))
            } else if entry_type.is_symlink() {
                let entry_usage = u128::from(entry_metadata.len());
                usage += entry_usage;
                Arc::new(Mutex::new(ScannerInode {
                    path: entry_path,
                    usage: entry_usage,
                    detail: ScannerInodeDetail::Sym,
                }))
            } else if entry_type.is_dir() {
                pending += 1;
                let sub_directory_inode = Arc::new(Mutex::new(ScannerInode {
                    path: entry_path,
                    usage: 0,
                    detail: ScannerInodeDetail::Dir(Vec::new()),
                }));
                sub_directory_inodes.push(Arc::clone(&sub_directory_inode));
                sub_directory_inode
            } else {
                continue;
            };

            entry_inodes.push(entry_inode);
        }

        {
	    let directory_inodes = directory_inodes.lock().unwrap();
            let directory_inode = directory_inodes.get(directory_id).unwrap();
	    let mut directory_inode = directory_inode.lock().unwrap();
            directory_inode.usage = usage;
            if let ScannerInodeDetail::Dir(children) = &mut directory_inode.detail {
                for entry_inode in entry_inodes {
                    children.push(entry_inode);
                }
            } else {
                panic!("Scanner: Non-directory inode in job queue")
            }
        }

        let _ = sender.send(DiscoverSignal {
            parent_id,
            directory_id,
            pending,
        });

        let mut directory_inodes = directory_inodes.lock().unwrap();
        let mut job_queue = job_queue.lock().unwrap();

        for sub_directory_inode in sub_directory_inodes {
            let new_directory_id = directory_inodes.len();
            directory_inodes.push(Arc::clone(&sub_directory_inode));

            (&mut job_queue).push_back(ScanJob {
                parent_id: Some(directory_id),
                directory_id: new_directory_id,
            });
        }
    }
    return ();
}

fn collector(
    directory_inodes: Arc<Mutex<Vec<Arc<Mutex<ScannerInode>>>>>,
    base_collected: Arc<RwLock<Option<Arc<Mutex<ScannerInode>>>>>,
    receiver: Receiver<DiscoverSignal>,
) -> () {
    let mut pendings: HashMap<InodeIndex, Pending> = HashMap::new();
    let mut parent_by_id: HashMap<InodeIndex, Option<InodeIndex>> = HashMap::new();
    loop {
        let DiscoverSignal {
            parent_id: mut maybe_parent_id,
            directory_id,
            pending,
        } = receiver.recv().unwrap();
        pendings.insert(directory_id, pending);
        parent_by_id.insert(directory_id, maybe_parent_id);

        /* Inode is a leaf directory

        There are two cases here:
        1. The leaf directory is the base directory. Scan is complete
        2. The leaf directory has a parent. Propagate usage to parent, reduce
        parent's pending by 1. If the parent has no other pending subdirectory,
        propagate up more.
         */

        let mut propagate = pending == 0;

        let directory_inodes = directory_inodes.lock().unwrap();

        let directory_inode = directory_inodes.get(directory_id).unwrap().clone();
        let mut usage_to_add = directory_inode.lock().unwrap().usage;

        while propagate {
            if let Some(parent_id) = maybe_parent_id {
                let parent_inode = directory_inodes.get(parent_id).unwrap().clone();
                let mut parent_inode = parent_inode.lock().unwrap();
                parent_inode.usage += usage_to_add;
                usage_to_add = parent_inode.usage;

                let parent_pending = pendings.get_mut(&parent_id).unwrap();
                *parent_pending -= 1;
                propagate = *parent_pending == 0;

                maybe_parent_id = parent_by_id.get(&parent_id).unwrap().to_owned();
            } else {
                break;
            }
        }

        if propagate {
            break;
        }
    }

    *(base_collected.write().unwrap()) =
        Some(directory_inodes.lock().unwrap().get(0).unwrap().clone());

    return ();
}

pub fn scan_base_path(
    worker_pool_size: usize,
    base_path: PathBuf,
    base_inode_collected: Arc<RwLock<Option<Arc<Mutex<ScannerInode>>>>>,
) {
    let directory_inodes = Arc::new(Mutex::new(Vec::new()));
    let job_queue = Arc::new(Mutex::new(VecDeque::new()));

    let base_inode = Arc::new(Mutex::new(ScannerInode {
        path: base_path,
        usage: 0,
        detail: ScannerInodeDetail::Dir(Vec::new()),
    }));
    directory_inodes
        .lock()
        .unwrap()
        .push(Arc::clone(&base_inode));
    job_queue.lock().unwrap().push_back(ScanJob {
        parent_id: None,
        directory_id: 0,
    });

    let (sender, receiver) = mpsc::channel();

    let mut scanner_handles = vec![];
    for _ in 0..worker_pool_size {
        let job_queue = Arc::clone(&job_queue);
        let directory_inodes = Arc::clone(&directory_inodes);
        let base_inode_collected = Arc::clone(&base_inode_collected);
        let sender = sender.clone();
        let handle = thread::spawn(move || {
            scanner(directory_inodes, job_queue, base_inode_collected, sender);
        });
        scanner_handles.push(handle);
    }

    let directory_inodes_clone = Arc::clone(&directory_inodes);
    let base_inode_collected_clone = Arc::clone(&base_inode_collected);
    let collector_handle = thread::spawn(move || {
        collector(directory_inodes_clone, base_inode_collected_clone, receiver);
    });

    for handle in scanner_handles {
        handle.join().unwrap();
    }
    collector_handle.join().unwrap();
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
            ScannerInodeDetail::Dir(arc_children) => {
                let mut children = Vec::new();
                for arc_child in arc_children {
                    let inode = Inode::try_from(arc_child.to_owned())?;
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
