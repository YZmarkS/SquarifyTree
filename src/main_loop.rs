use crate::{inode::*, layout::*, scanner::*};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout};
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::Frame;
use ratatui::{prelude::Rect, DefaultTerminal};
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::replace;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{path::PathBuf, rc::Rc};

type CellIndex = usize;

#[derive(Clone)]
pub struct HistoryEntry {
    pub directory: Rc<RefCell<Inode>>,
    pub starting_index: usize,
    pub selected: CellIndex,
}

pub struct ScanningContext {
    pub base_inode_collected: Arc<RwLock<Option<Arc<Mutex<ScannerInode>>>>>,
    pub scanning_thread_handle: JoinHandle<()>,
}

pub struct DisplayingContext {
    pub cells: Vec<Cell>,
    pub neighbours: Vec<Neighbours>,
}

pub enum Status {
    Exit,
    Swapping,
    StartScanning,
    Scanning(ScanningContext),
    ResetDirectoryHistory,
    GetCanvasSize,
    CalcNewLayout(Rect),
    Displaying(DisplayingContext),
}

pub struct State {
    pub status: Status,
    pub root_path: PathBuf,
    pub worker_pool_size: usize,

    pub root_inode: Rc<RefCell<Inode>>,

    pub history: Vec<HistoryEntry>,
    pub now: HistoryEntry,
}

impl State {
    pub fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        use Status::*;
        loop {
            terminal.draw(|frame| self.widget_render(frame))?;

            let old_status = replace(&mut self.status, Swapping);

            let new_status: Status = self.delta(old_status);

            match new_status {
                Exit => {
                    break;
                }
                other => {
                    self.status = other;
                }
            }
        }
        Ok(())
    }

    pub fn widget_render(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &mut State {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Status::*;

        let old_status = replace(&mut self.status, Swapping);

        let new_status: Status = match old_status {
            GetCanvasSize => {
                let [_, middle, _] = Layout::vertical(vec![
                    Constraint::Length(3),
                    Constraint::Fill(1),
                    Constraint::Length(3),
                ])
                .areas(area);

                CalcNewLayout(middle)
            }
            Scanning(scanning_context) => {
                // Order box
                Block::bordered().render(area, buf);

                let [mid_col] = Layout::new(Direction::Horizontal, vec![Constraint::Max(50)])
                    .flex(Flex::Center)
                    .areas(area);
                let [mid_box] = Layout::new(Direction::Vertical, vec![Constraint::Max(9)])
                    .flex(Flex::Center)
                    .areas(mid_col);
                let mid_text_area = mid_box.centered(Constraint::Max(12), Constraint::Max(1));

                let dots = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    / 250 % 4;
                let scanning_string = format!("Scanning{}", ".".repeat(dots as usize));

                Block::bordered().render(mid_box, buf);
                Paragraph::new(scanning_string).render(mid_text_area, buf);

                Scanning(scanning_context)
            }
            Displaying(DisplayingContext { cells, neighbours }) => {
                self.main_render(area, buf, &cells);

                Displaying(DisplayingContext { cells, neighbours })
            }
            other => other,
        };

        self.status = new_status;
    }
}
