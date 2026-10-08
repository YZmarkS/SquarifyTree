use std::{cmp, collections::HashMap, path::{PathBuf}, rc::{Rc}};
use Status::NeedRefresh;
use ratatui::{prelude::{Rect}};
use crate::inode::*;
use crate::squarify::*;
use crate::layout::*;


pub enum Status {
    NeedRefresh,
    Setup,
    Displaying,
}

pub struct AppState {
    status: Status,
    path: PathBuf,
    worker_count: usize,

    root_inode: Rc<Inode>,

    cells: Vec<Cell>,
    neighbours: Vec<Neighbours>,
    curr_cell_index: CellId,

    history: Vec<HistoryEntry>,
}

pub struct HistoryEntry {
    parent: Rc<Inode>,
    starting_index: usize,
}


impl AppState {
    pub fn up(&mut self) {
	match self.neighbours[self.curr_cell_index].top {
	    None => (),
	    Some(new_index) => self.curr_cell_index = new_index
	}
    }

    pub fn down(&mut self) {
	match self.neighbours[self.curr_cell_index].bottom {
	    None => (),
	    Some(new_index) => self.curr_cell_index = new_index
	}
    }

    pub fn left(&mut self) {
	match self.neighbours[self.curr_cell_index].left {
	    None => (),
	    Some(new_index) => self.curr_cell_index = new_index
	}
    }

    pub fn right(&mut self) {
	match self.neighbours[self.curr_cell_index].right {
	    None => (),
	    Some(new_index) => self.curr_cell_index = new_index
	}
    }

    pub fn go_in(&mut self) {
	let history_top = self.history.last().unwrap();
	let curr_cell = &self.cells[self.curr_cell_index];

	match &(curr_cell.content) {
	    CellContent::Inode(inode_rc) => {
		match &inode_rc.detail {
		    InodeDetail::Reg => (),
		    InodeDetail::Sym => (),
		    InodeDetail::Dir(_) => {
			self.history.push(HistoryEntry {
			    parent: inode_rc.clone(),
			    starting_index: 0,
			});
			self.status = NeedRefresh;
		    },
		}
	    },
	    CellContent::Other(starting_index) => {
		self.history.push(HistoryEntry {
		    parent: history_top.parent.clone(),
		    starting_index: *starting_index,
		});
		self.status = NeedRefresh;
	    },
	}
    }

    pub fn go_out(&mut self) {
	if 1 < self.history.len() {
	    self.history.pop();
	    self.status = NeedRefresh;
	}
    }
}
