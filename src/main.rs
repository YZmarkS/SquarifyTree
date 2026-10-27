use std::cell::RefCell;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::{io, path::Path};

mod inode;
mod layout;
mod logic;
mod main_loop;
mod render;
mod scanner;
mod squarify;
use inode::*;
use main_loop::*;
use scanner::*;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let env_path = std::env::var("SCAN_ROOT").unwrap();

    let path = PathBuf::from(env_path);
    // let path = Path::new(args.get(1).unwrap()).canonicalize().unwrap();

    let worker_pool_size = 10;

    let root_inode: Rc<RefCell<Inode>> = Rc::new(RefCell::new(Inode {
        path: path.clone(),
        usage: 0,
        detail: InodeDetail::Dir(Vec::new()),
    }));

    let base_collected = Arc::new(RwLock::new(None));
    let base_collected_clone = base_collected.clone();
    scan_base_path(10, path, base_collected_clone);

    loop {
        if base_collected.read().unwrap().is_none() {
            continue;
        }
        break;
    }

    let maybe_base_collected = base_collected.read().unwrap().to_owned();
    let new_state = match maybe_base_collected {
        None => panic!("impossible"),
        Some(scanned_collector_inodes) => {
	    println!("{:?}", scanned_collector_inodes.len());
            let scanned_collector_inodes = scanned_collector_inodes.clone();
            let mut inodes = collector_inodes_to_inodes(&scanned_collector_inodes);
            populate_children_list(&scanned_collector_inodes, &mut inodes);
        }
    };

    // let mut state = State {
    //     status: Status::StartScanning,
    //     root_path: path.clone(),
    //     worker_pool_size,

    //     root_inode: root_inode.clone(),
    //     history: Vec::new(),
    //     now: HistoryEntry {
    //         directory: root_inode.clone(),
    //         starting_index: 100000,
    //         selected: 0,
    //     },
    // };

    // ratatui::run(|terminal| state.main_loop(terminal)).unwrap();

    Ok(())
}
