use std::cell::RefCell;
use std::path::Path;
use std::{cmp, path::PathBuf, rc::Rc};
use std::sync::{Arc, Mutex};
use Status::{CalcNewLayout, Displaying, NeedRefresh};
use crossterm::event::Event;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::symbols::merge::MergeStrategy;
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::{DefaultTerminal, prelude::Rect};
use crate::{inode::*, squarify, scanner::*};
use crate::layout::*;

type CellIndex = usize;

#[derive(Clone)]
pub struct HistoryEntry {
    pub directory: Rc<RefCell<Inode>>,
    pub starting_index: usize,
    pub selected: CellIndex,
}

pub enum Status {
    Scanning(Arc<Mutex<Vec<Arc<Mutex<ScannerInode>>>>>, Arc<Mutex<bool>>), // this needs working
    NeedRefresh,
    CalcNewLayout(Rect),
    Displaying(Vec<Cell>),
}

#[allow(unused)]
pub struct State {
    pub status: Status,
    pub root_path: PathBuf,
    pub worker_pool_size: usize,

    pub root_inode: Rc<RefCell<Inode>>,

    pub history: Vec<HistoryEntry>,
    pub now: HistoryEntry,
}



impl Widget for &mut State {
    fn render(self, area: Rect, buf: &mut Buffer) {
	use Status::*;
	match self.status {
	    Scanning(inodes, root_collected) => {
	    }
	    NeedRefresh => {
		let [ _, middle, _ ] = Layout::vertical(vec![
		    Constraint::Length(3),
		    Constraint::Fill(1),
		    Constraint::Length(3)
		]).areas(area);
		self.status = Status::CalcNewLayout(middle);
	    },
	    Displaying(cells) => {
		self.render_helper(area, buf);
	    },
	    _ => {
	    },
	}
    }
}


impl AppState {
    pub fn ratatui_loop(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
	loop {
	    terminal.draw(|frame| self.draw(frame))?;

	    match self.status {
		CalcNewLayout(canvas) => {
		    let directory = self.now.directory.clone();
		    let starting_index = self.now.starting_index;
		    let children = if let InodeDetail::Dir(children) = &directory.detail {
			children
		    } else {
			panic!("Current is not a directory");
		    };

		    let squarify = squarify(&children[starting_index..],
					    canvas.width - 1, canvas.height - 1);
		    let rects = squarify_to_rects(&squarify, canvas);
		    let cells = make_cell_list(&rects, children, starting_index);
		    let neighbours = relate_cells(&cells);
		    self.cells = cells;
		    self.neighbours = neighbours;
		    self.on = cmp::min(self.on, self.cells.len() - 1);

		    self.status = Displaying;
		},
		_ => {
		    let event = crossterm::event::read()?;
		    match event {
			Event::Resize(_, _) => {
			    self.status = NeedRefresh;
			},
			Event::Key(event) => {
			    use crossterm::event::KeyCode;
			    match &event.code {
				KeyCode::Up => { self.up() },
				KeyCode::Down => { self.down(); },
				KeyCode::Left => { self.left(); },
				KeyCode::Right => { self.right(); },
				KeyCode::Backspace => { self.go_out(); },
				KeyCode::Enter => { self.go_in(); },
				KeyCode::Char('q') => { break; },
				_ => {},
			    }
			},
			_ => {},
		    }
		},
	    }
	}
	Ok(())
    }

    pub fn draw(&mut self, frame: &mut Frame) {
	frame.render_widget(self, frame.area());
    }

    pub fn up(&mut self) {
	match self.neighbours[self.on].top {
	    None => (),
	    Some(new_index) => self.on = new_index
	}
    }

    pub fn down(&mut self) {
	match self.neighbours[self.on].bottom {
	    None => (),
	    Some(new_index) => self.on = new_index
	}
    }

    pub fn left(&mut self) {
	match self.neighbours[self.on].left {
	    None => (),
	    Some(new_index) => self.on = new_index
	}
    }

    pub fn right(&mut self) {
	match self.neighbours[self.on].right {
	    None => (),
	    Some(new_index) => self.on = new_index
	}
    }

    pub fn go_in(&mut self) {
	let cell = &self.cells[self.on];

	match &cell.content {
	    CellContent::Inode(inode) => {
		let inode_rc = inode.clone();

		if let InodeDetail::Dir(_) = (*inode_rc).detail {
		    let now = HistoryEntry {
			directory: self.now.directory.clone(),
			starting_index: self.now.starting_index,
			was_on: self.on,
		    };
		    let next = HistoryEntry {
			directory: inode.clone(),
			starting_index: 0,
			was_on: 0, // Placeholde
		    };
		    self.history.push(now);
		    self.now = next;
		    self.on = 0;
		    self.status = NeedRefresh;
		}
	    },
	    CellContent::Other(starting_index) => {
		let now = HistoryEntry {
		    directory: self.now.directory.clone(),
		    starting_index: self.now.starting_index,
		    was_on: self.on,
		};
		let next = HistoryEntry {
		    directory: self.now.directory.clone(),
		    starting_index: *starting_index,
		    was_on: self.on, // Placeholde
		};
		self.history.push(now);
		self.now = next;
		self.status = NeedRefresh;
	    },
	    CellContent::Nothing => {
	    }
	}
    }

    pub fn go_out(&mut self) {
	if 0 < self.history.len() {
	    self.now = self.history.pop().unwrap();
	    self.on = self.now.was_on;
	    self.status = NeedRefresh;
	}
    }


}
