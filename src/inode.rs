use std::{rc::Rc};
use std::cell::RefCell;
use std::path::{PathBuf};

pub type DiskUsage = u128;

pub struct Inode {
    pub path: PathBuf,
    pub usage: DiskUsage,
    pub detail: InodeDetail,
}

pub enum InodeDetail {
    Reg,
    Sym,
    Dir(Vec<Rc<RefCell<Inode>>>),
}

#[allow(unused)]
pub fn print_inodes(inode: &Inode, indent: u8) -> () {
    for _ in 0..indent {
	print!("    ");
    }
    println!("{} {}", inode.path.display(), inode.usage);
    if let InodeDetail::Dir(children) = &(inode.detail) {
	for child in children {
	    print_inodes(&child.borrow(), indent + 1);
	}
    }
}
