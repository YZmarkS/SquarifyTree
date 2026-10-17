use std::cell::RefCell;
use std::collections::{self, HashMap, VecDeque};
use std::fs::*;
use std::io::ErrorKind::*;
use std::io::{Error, ErrorKind::NotADirectory};
use std::path::{Path, PathBuf};
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

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub struct ScanJob {
    path: PathBuf,
    parent_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct CollectorInode {
    path: PathBuf,
    parent_path: Option<PathBuf>,
    usage: DiskUsage,
    detail: CollectorInodeDetail,
}

#[derive(Debug, Clone)]
pub enum CollectorInodeDetail {
    Reg,
    Sym,
    Dir(DirectoryDetail),
}

#[derive(Debug, Clone)]
pub struct DirectoryDetail {
    pending: usize,
    child_paths: Vec<PathBuf>,
}

impl CollectorInode {
    pub fn is_dir(&self) -> bool {
        match self.detail {
            CollectorInodeDetail::Dir(_) => true,
            _ => false,
        }
    }
}

fn scanner(
    job_queue: Arc<Mutex<VecDeque<ScanJob>>>,
    base_collected: Arc<RwLock<Option<HashMap<PathBuf, CollectorInode>>>>,
    sender: Sender<CollectorInode>,
) -> () {
    loop {
        /* Check there is job to be done */
        if base_collected.read().unwrap().is_some() {
            break;
        }

        let ScanJob { path, parent_path } = match (job_queue.lock().unwrap()).pop_front() {
            Some(job) => job,
            None => continue,
        };

        let mut usage = 0;
        let mut pending = 0;
        let mut child_paths: Vec<PathBuf> = Vec::new();
        let mut jobs: Vec<ScanJob> = Vec::new();

        for entry_result in read_dir(&path).unwrap() {
            let entry = entry_result.unwrap();

            let entry_metadata = entry.metadata().unwrap();
            let entry_type = entry.file_type().unwrap();
            let entry_path = entry.path();

            if entry_type.is_file() {
                let entry_usage = u128::from(entry_metadata.len());
                usage += entry_usage;
                sender
                    .send(CollectorInode {
                        path: entry_path.clone(),
                        parent_path: Some(path.clone()),
                        usage: entry_usage,
                        detail: CollectorInodeDetail::Reg,
                    })
                    .unwrap();
            } else if entry_type.is_symlink() {
                let entry_usage = u128::from(entry_metadata.len());
                usage += entry_usage;
                sender
                    .send(CollectorInode {
                        path: entry_path.clone(),
                        parent_path: Some(path.clone()),
                        usage: entry_usage,
                        detail: CollectorInodeDetail::Sym,
                    })
                    .unwrap();
            } else if entry_type.is_dir() {
                pending += 1;
                jobs.push(ScanJob {
                    path: entry_path.clone(),
                    parent_path: Some(path.clone()),
                });
            } else {
                println!(
                    "Unaccounted entry type {:?} produced by {:?}",
                    entry_type, entry_path
                );
		continue;
            };

            child_paths.push((&entry_path).clone());
        }

        let inode = CollectorInode {
            path,
            parent_path,
            usage,
            detail: CollectorInodeDetail::Dir(DirectoryDetail {
                pending,
                child_paths,
            }),
        };

        sender.send(inode).unwrap();

	let mut job_queue_locked = job_queue.lock().unwrap();
        for job in jobs {
	    (&mut job_queue_locked).push_back(job);
        }
    }
    return ();
}

fn collector(
    base_collected: Arc<RwLock<Option<HashMap<PathBuf, CollectorInode>>>>,
    receiver: Receiver<CollectorInode>,
) -> () {
    use CollectorInodeDetail as CID;
    let mut collector_inodes: HashMap<PathBuf, CollectorInode> = HashMap::new();
    loop {
        let new_inode = receiver.recv().unwrap();
        let curr_path = new_inode.path.clone();

        collector_inodes.insert(curr_path.clone(), new_inode);

        let curr_inode = collector_inodes.get(&curr_path).unwrap();

        if !curr_inode.is_dir() {
            continue;
        }

        let pending = if let CID::Dir(DirectoryDetail { pending, .. }) = curr_inode.detail {
            pending
        } else {
            panic!(
                "Scanning: {:?} should be a directory inode",
                curr_inode.path
            );
        };

        /* Inode is a leaf directory

        There are two cases here:
        1. The leaf directory is the base directory. Scan is complete
        2. The leaf directory has a parent. Propagate usage to parent, reduce
        parent's pending by 1. If the parent has no other pending subdirectory,
        propagate up more.
         */

        let mut propagate = pending == 0;

        let mut usage_to_add = curr_inode.usage;
        let mut parent_path_option = curr_inode.parent_path.clone();

        while propagate {
            if let Some(parent_path) = parent_path_option {
                let parent_inode = collector_inodes.get_mut(&parent_path).unwrap();
                let parent_pending =
                    if let CID::Dir(DirectoryDetail { pending, .. }) = &mut (parent_inode.detail) {
                        pending
                    } else {
                        panic!(
                            "Scanning: Parent directory {:?} should be a directory inode",
                            &parent_path
                        );
                    };

                *parent_pending -= 1;
                parent_inode.usage += usage_to_add;

                propagate = *parent_pending == 0;
                usage_to_add = parent_inode.usage;
                parent_path_option = parent_inode.parent_path.clone();
            } else {
                break;
            }
        }

        if propagate {
	    println!("Collector going to writing HashMap");
            *(base_collected.write().unwrap()) = Some(collector_inodes);
	    println!("Collector done writing HashMap");
            break;
        }
    }
    return ();
}

fn collector_inodes_to_inodes(
    collector_inodes: &HashMap<PathBuf, CollectorInode>,
) -> HashMap<PathBuf, Rc<RefCell<Inode>>> {
    let mut result = HashMap::new();

    for (path, collector_inode) in collector_inodes.iter() {
        use CollectorInodeDetail::*;
        result.insert(
            path.clone(),
            Rc::new(RefCell::new(Inode {
                path: collector_inode.path.clone(),
                usage: collector_inode.usage,
                detail: match collector_inode.detail {
                    Reg => InodeDetail::Reg,
                    Sym => InodeDetail::Sym,
                    Dir(_) => InodeDetail::Dir(Vec::new()),
                },
            })),
        );
    }
    result
}

fn populate_children_list(
    collector_inodes: &HashMap<PathBuf, CollectorInode>,
    inodes: &mut HashMap<PathBuf, Rc<RefCell<Inode>>>,
) {
    let mut collector_keys: Vec<PathBuf> = collector_inodes.keys().cloned().collect();
    collector_keys.sort();
    let mut inodes_keys: Vec<PathBuf> = inodes.keys().cloned().collect();
    inodes_keys.sort();

    for (inode_path, collector_inode) in collector_inodes.iter() {
        if let CollectorInodeDetail::Dir(DirectoryDetail { child_paths, .. }) =
            &(collector_inode.detail)
        {
            let mut inodes_to_add = Vec::new();
            for child_path in child_paths {
                inodes_to_add.push(inodes.get(child_path).unwrap().clone());
            }
            let inode = inodes.get_mut(inode_path).unwrap();
            if let InodeDetail::Dir(child_inodes) = &mut (inode.borrow_mut().detail) {
                for inode_to_add in inodes_to_add.into_iter() {
                    child_inodes.push(inode_to_add);
                }
		child_inodes.sort_by_key(|inode| inode.borrow().usage);
		child_inodes.reverse();
            } else {
                panic!("Scanning: Directory collector inode was not mapped to directory inode");
            }
        }
    }
}

pub fn scan_base_path(base_path: &Path) -> Rc<RefCell<Inode>> {
    let job_queue = Arc::new(Mutex::new(VecDeque::from(vec![ScanJob {
        path: base_path.to_path_buf(),
        parent_path: None,
    }])));
    let base_collected = Arc::new(RwLock::new(None));

    let (sender, receiver) = mpsc::channel();

    let mut scanner_handles = vec![];
    for _ in 0..20 {
        let job_queue = (&job_queue).clone();
        let base_collected = (&base_collected).clone();
        let sender = sender.clone();
        let handle = thread::spawn(move || {
            scanner(job_queue, base_collected, sender);
        });
        scanner_handles.push(handle);
    }

    let base_collected_clone = (&base_collected).clone();
    let collector_handle = thread::spawn(|| {
        collector(base_collected_clone, receiver);
    });

    collector_handle.join().unwrap();
    println!("Collector Joined");
    for handle in scanner_handles {
        handle.join().unwrap();
    }
    println!("Scanners Joined");

    println!("Start post processing");
    println!("Extract final collector inodes");
    println!();
    let final_collector_inodes = base_collected.read().unwrap();

    println!("Turn collector inodes into inodes");
    println!("Slow");
    let mut final_inodes = collector_inodes_to_inodes(final_collector_inodes.as_ref().unwrap());

    println!("Populate inodes");
    println!("Slow");
    populate_children_list(final_collector_inodes.as_ref().unwrap(), &mut final_inodes);
    println!("Done post processing");


    final_inodes
        .get(&base_path.to_path_buf())
        .unwrap()
        .to_owned()
}
