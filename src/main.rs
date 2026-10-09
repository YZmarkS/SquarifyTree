use std::{io, path::Path};

use inode::InodeDetail;
use layout::squarify_to_rects;
use ratatui::{
    prelude::*,
    layout::{Constraint, Layout, Spacing},
    symbols::merge::MergeStrategy,
    widgets::{Paragraph, Block, Borders},
    DefaultTerminal, Frame,
};

mod scanner;
mod squarify;
mod inode;
mod layout;
mod logic;
use scanner::*;
use squarify::*;
use inode::*;
use logic::*;

fn main() -> io::Result<()> {

    let path = Path::new("/home/lemon/Projects/SquarifyTree/C");
    let worker_pool_size = 10;

    let mut state = path_to_app_state(path, worker_pool_size)?;

    // just_calc();

    ratatui::run(|terminal| state.ratatui_loop(terminal)).unwrap();
    Ok(())

}

fn just_calc() {

    // let path = Path::new("/home/lemon/Projects/SquarifyTree/C/src");
    // let worker_pool_size = 10;

    // let root_inode = path_to_inode(path, worker_pool_size);
    // print_inodes(&root_inode, 0);

    // if let InodeDetail::Dir(children) = &(root_inode.detail) {
    // 	let layout = squarify(&children[..], 209 - 1, 51 - 1);
    // 	print_layout(&layout, 0);
    // }
}
