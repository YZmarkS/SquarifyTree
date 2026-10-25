use std::cell::RefCell;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;
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

fn main() -> io::Result<()> {

    let args: Vec<String> = env::args().collect();

    let path = Path::new(args.get(1).unwrap()).canonicalize().unwrap();

    let worker_pool_size = 10;

    let root_inode: Rc<RefCell<Inode>> = Rc::new(RefCell::new(Inode {
        path: path.clone(),
        usage: 0,
        detail: InodeDetail::Dir(Vec::new()),
    }));

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
