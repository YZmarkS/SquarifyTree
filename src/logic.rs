use std::path::Path;
use std::{cmp, path::PathBuf, rc::Rc};
use Status::{CalcNewLayout, Displaying, NeedRefresh};
use crossterm::event::Event;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::symbols::merge::MergeStrategy;
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::{DefaultTerminal, prelude::Rect};
use crate::{inode::*, scanner::scan_directory_with_pool, squarify};
use crate::layout::*;

pub enum Status {
    NeedRefresh,
    CalcNewLayout(Rect),
    Displaying,
}

#[allow(unused)]
pub struct AppState {
    status: Status,
    path: PathBuf,
    worker_pool_size: usize,

    root_inode: Rc<Inode>,

    cells: Vec<Cell>,
    neighbours: Vec<Neighbours>,
    on: CellId,

    history: Vec<HistoryEntry>,
    now: HistoryEntry,
}

pub struct HistoryEntry {
    directory: Rc<Inode>,
    starting_index: usize,
    was_on: CellId,
}

pub fn path_to_app_state(path: &Path, worker_pool_size: usize) -> std::io::Result<AppState> {
    let root_inode = scan_directory_with_pool(path, worker_pool_size)?;
    let now = HistoryEntry {
	directory: root_inode.clone(),
	starting_index: 0,
	was_on: 0 // just a place holder
    };
    Ok(AppState {
	status: NeedRefresh,
	path: path.to_path_buf(),
	worker_pool_size,

	root_inode,
	cells: Vec::new(),
	neighbours: Vec::new(),
	on: 0,

	history: Vec::new(),
	now,
    })
}


impl Widget for &mut AppState {
    fn render(self, area: Rect, buf: &mut Buffer) {
	match self.status {
	    NeedRefresh => {
		let [ _, middle, _ ] = Layout::vertical(vec![
		    Constraint::Length(3),
		    Constraint::Fill(1),
		    Constraint::Length(3)
		]).areas(area);
		self.status = Status::CalcNewLayout(middle);
	    },
	    Displaying => {
		self.render_aux(area, buf);
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


    fn render_aux(&mut self, area: Rect, buf: &mut Buffer) {
	let directory = &self.now.directory;
	let directory_path = directory.path.to_string_lossy().into_owned();

	let current_cell = &self.cells[self.on];
	let current_name = get_cell_content_name(current_cell);
	let [ top, _, bottom ] = Layout::vertical(vec![
	    Constraint::Length(3),
	    Constraint::Fill(1),
	    Constraint::Length(3)
	]).areas(area);
	let [ top_left, top_right ] = Layout::horizontal(vec![
	    Constraint::Ratio(1, 2) ; 2
	]).areas(top);

	Paragraph::new(directory_path)
	    .block(Block::bordered())
	    .render(top_left, buf);
	Paragraph::new(current_name)
	    .block(Block::bordered())
	    .render(top_right, buf);
	Paragraph::new("Help")
	    .block(Block::bordered())
	    .render(bottom, buf);

	for i in 0..self.cells.len() {
	    let cell_name = get_cell_content_name(&self.cells[i]);
	    let outer_block = Block::bordered()
		.merge_borders(MergeStrategy::Exact);
	    let on_style = Style::new().fg(Color::White).bg(Color::DarkGray);
	    let off_style = Style::new().fg(Color::White).bg(Color::Black);
	    let inner_block = Paragraph::new(cell_name)
		.style(if i == self.on { on_style } else { off_style });
	    let inner_area = outer_block.inner(self.cells[i].rect);

	    outer_block.render(self.cells[i].rect, buf);
	    inner_block.render(inner_area, buf);
	}
    }
}
