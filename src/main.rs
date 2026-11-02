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

    let worker_pool_size = 10;
    let base_inode_collected = Arc::new(RwLock::new(None));

    scan_base_path(worker_pool_size, path, Arc::clone(&base_inode_collected));

    loop {
	if base_inode_collected.read().unwrap().is_none() {
	    continue;
	}
	break;
    }

    let _ = match base_inode_collected.read().unwrap().to_owned() {
	None => panic!("impossible"),
	Some(base_inode) => {
	    Inode::try_from(base_inode)
	},
    };

    Ok(())
}
