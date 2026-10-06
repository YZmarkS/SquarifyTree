use std::path::Path;
use std::sync::{Arc, Mutex, mpsc, mpsc::{Sender, Receiver}};

mod scanner;
use scanner::*;

fn print_inodes(inode_arc: &Arc<Mutex<ScannerInode>>, indent: u8) -> () {
    let inode = inode_arc.lock().unwrap();
    for _ in 0..indent {
	print!("    ");
    }
    println!("{} {}", (*inode).path.display(), inode.size);
    if let ScannerInodeDetail::Dir(children) = &(*inode).detail {
	for child in children {
	    print_inodes(child, indent + 1);
	}
    }
}

fn main() {
    println!("Hello, world!");

    let final_state_result = scanner::scan(Path::new("/home/lemon/Algorithms_and_Data_Structures"),
					   10).unwrap();
    let final_state = final_state_result.lock().unwrap();

    print_inodes((*final_state).inodes.get(&FIRST_ID).unwrap(), 0);

    println!("DONE!");
}
