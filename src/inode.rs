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

    pub fn path_directory_string(&self) -> String {
	self.path.clone().join("").to_string_lossy().into_owned()
    }

    fn base_name_pathbuf(&self) -> PathBuf {
	match self.path.file_name() {
	    None => PathBuf::from(""),
	    Some(file_name) => file_name.into(),
	}
    }
    fn base_name_directory_string(&self) -> String {
	self.base_name_pathbuf().join("").to_string_lossy().into_owned()
    }
    pub fn base_name_string(&self) -> String {
	if self.is_reg() || self.is_sym() {
	    self.base_name_pathbuf().to_string_lossy().into_owned()
	} else if self.is_dir() {
	    self.base_name_directory_string()
	} else {
	    panic!("Inode base_name_string: Unknown inode type")
	}
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
