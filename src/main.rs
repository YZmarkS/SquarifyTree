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
    let path = Path::new(args.get(1).unwrap()).canonicalize().unwrap();

    // let env_path = std::env::var("SCAN_ROOT").unwrap();
    // let path = PathBuf::from(env_path);


    let worker_pool_size = 10;

    let root_inode: Rc<RefCell<Inode>> = Rc::new(RefCell::new(Inode {
        path: path.clone(),
        usage: 0,
        detail: InodeDetail::Dir(Vec::new()),
    }));

    // let base_inode_collected = Arc::new(RwLock::new(None));
    // let base_collected_clone = base_inode_collected.clone();
    // scan_base_path(10, path, base_collected_clone);

    // loop {
    //     if base_inode_collected.read().unwrap().is_none() {
    //         continue;
    //     }
    //     break;
    // }

    // let maybe_base_inode_collected = base_inode_collected.read().unwrap().to_owned();

    // let inode_rc = match maybe_base_inode_collected {
    //     None => panic!("impossible"),
    //     Some(base_inode) => {
    //         Rc::new(RefCell::new(Inode::try_from(base_inode).unwrap()))
    //     }
    // };

    // print_inodes(&inode_rc, 0);

    let mut state = State {
        status: Status::StartScanning,
        root_path: path.clone(),
        worker_pool_size,

        root_inode: root_inode.clone(),
        history: Vec::new(),
        now: HistoryEntry {
            directory: root_inode.clone(),
            starting_index: 100000,
            selected: 0,
        },
    };

    ratatui::run(|terminal| state.main_loop(terminal)).unwrap();

    Ok(())
}
