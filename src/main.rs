use std::{io, path::Path};

use inode::InodeDetail;
use layout::layout_to_rects;
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


    // let mut state = AppState {
    // 	path: Path::new("/home/lemon/Projects/SquarifyTree/C/src"),
    // 	worker_count: 10
    // };

    just_calc();

    // ratatui::run(|terminal| state.run(terminal)).unwrap();
    Ok(())
    // layout_to_rects(&rects, &squarify,

}

fn just_calc() {
    let path = Path::new("/home/lemon/Projects/SquarifyTree/C/src");
    let worker_count = 10;

    let root_inode = path_to_inode(path, worker_count);
    print_inodes(&root_inode, 0);

    if let InodeDetail::Dir(children) = &(root_inode.detail) {
	let layout = squarify(&children[..], 209 - 1, 51 - 1);
	print_layout(&layout, 0);
    }
}

// struct AppState<'app> {




//     path: &'app Path,
//     worker_count: usize,
//     all_inodes: Option<Vec<Inode>>,
//     curr_inodes: Option<[Inode]>,


// }

impl AppState {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
	loop {
	    terminal.draw(|frame| self.draw(frame))?;
	    if crossterm::event::read()?.is_key_press() {
		break;
            }
	}
	Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {

	let root_inode = path_to_inode(self.path, self.worker_count);

	let area = frame.area();

	print!("{}, {}", area.width, area.height);

	let squarify = if let InodeDetail::Dir(children) = &(root_inode.detail) {
	    squarify(&children[..], area.width - 1, area.height - 1)
	} else {
	    panic!();
	};

	let mut rects: Vec<Rect> = Vec::new();
	layout_to_rects(&mut rects, &squarify, frame.area());

	for rect in rects {
	    frame.render_widget(
		Block::new()
		    .borders(Borders::ALL)
		    .merge_borders(MergeStrategy::Exact),
		rect
	    );
	}

    }
}
