use std::{io, path::Path};
use std::cell::RefCell;
use std::rc::Rc;
use std::env;

mod inode;
mod layout;
mod main_loop;
mod render;
mod scanner;
mod squarify;
mod logic;
use main_loop::*;
use inode::*;


fn main() -> io::Result<()> {

    let args: Vec<String> = env::args().collect();

    let path = Path::new(args.get(1).unwrap());
    let worker_pool_size = 10;

    let root_inode: Rc<RefCell<Inode>> = Rc::new(RefCell::new(Inode {
        path: path.to_path_buf(),
        usage: 0,
        detail: InodeDetail::Dir(Vec::new()),
    }));

    let mut state = State {
        status: Status::StartScanning,
        root_path: path.to_path_buf(),
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
