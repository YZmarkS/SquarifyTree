use crate::logic::*;
use crate::render::area_to_draw_areas;
use crate::squarify::HEIGHT_TO_WIDTH_RATIO;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Spacing};
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::{prelude::Rect, DefaultTerminal};
use std::mem::replace;
use std::time::{SystemTime, UNIX_EPOCH};

impl State {
    pub fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| frame.render_widget(&mut *self, frame.area()))?;

            let old_status = replace(&mut self.status, Status::Swapping);

            let new_status: Status = self.delta(old_status);

            match new_status {
                Status::Exit => {
                    break;
                }
                other => {
                    self.status = other;
                }
            }
        }
        Ok(())
    }
}

impl Widget for &mut State {
    /* Render Function (Weird Code Ahead)

    Due to the choice of single source of truth, the [render] method also serves
    the purpose of passing the canvas size to the computation stage.
     */
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Status::*;

        self.draw_areas = area_to_draw_areas(area);

        let old_status = replace(&mut self.status, Swapping);

        let new_status: Status = match old_status {
            scanning_state @ Scanning { .. } => {
                self.render_scanning_modal(buf);

                scanning_state
            }
            Displaying { cells, neighbours } => {
                self.render_display(&cells, buf);

                Displaying { cells, neighbours }
            }
            other => other,
        };

        self.status = new_status;
    }
}
