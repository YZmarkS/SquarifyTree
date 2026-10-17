use std::{rc::Rc};
use std::cell::RefCell;
use std::path::{PathBuf};

pub type DiskUsage = u128;

#[derive(Debug)]
pub struct Inode {
    pub path: PathBuf,
    pub usage: DiskUsage,
    pub detail: InodeDetail,
}

#[derive(Debug)]
pub enum InodeDetail {
    Reg,
    Sym,
    Dir(Vec<Rc<RefCell<Inode>>>),
}

impl Inode {
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

impl InodeDetail {
    fn is_reg(&self) -> bool {
        match self {
            InodeDetail::Reg => true,
            _ => false,
        }
    }
    fn is_sym(&self) -> bool {
        match self {
            InodeDetail::Sym => true,
            _ => false,
        }
    }
    fn is_dir(&self) -> bool {
        match self {
            InodeDetail::Dir(_) => true,
            _ => false,
        }
    }
}

#[allow(unused)]
pub fn print_inodes(inode: &Rc<RefCell<Inode>>, indent: u8) -> () {
    for _ in 0..indent {
	print!("    ");
    }
    println!("{:?} {:?}",
	     inode.borrow().path,
	     inode.borrow().usage);
    if let InodeDetail::Dir(children) = &(inode.borrow().detail) {
	for child in children {
	    print_inodes(child, indent + 1);
	}
    }
}
