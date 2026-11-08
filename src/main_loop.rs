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

	self.draw_areas = Some(area_to_draw_areas(area));

        let old_status = replace(&mut self.status, Swapping);

        let new_status: Status = match old_status {
	    /* Get Draw Areas

	    Used to calculate the draw areas.

	    Interleaved with [delta] to get the most up-to-date draw areas.
	    Common reasons of trigger are:
	    - Window resize.
	    - Enter different directory.
	     */
            GetDrawAreas => {
                let [info_rect, canvas, help_rect] = Layout::vertical(vec![
                    Constraint::Length(3),
                    Constraint::Fill(1),
                    Constraint::Length(3),
                ])
                .areas(area);

                let [directory_path_rect, entry_name_rect, entry_info_rect] =
                    Layout::horizontal(vec![Constraint::Fill(1); 3])
                        .spacing(Spacing::Overlap(1))
                        .areas(info_rect);

                let [_, mid_col, _] = Layout::new(
                    Direction::Horizontal,
                    vec![
                        Constraint::Length(2 * HEIGHT_TO_WIDTH_RATIO),
                        Constraint::Fill(1),
                        Constraint::Length(2 * HEIGHT_TO_WIDTH_RATIO),
                    ],
                )
                .flex(Flex::Center)
                .areas(area);
                let [_, modal_rect, _] = Layout::new(
                    Direction::Vertical,
                    vec![
                        Constraint::Length(2),
                        Constraint::Max(9),
                        Constraint::Length(2),
                    ],
                )
                .flex(Flex::Center)
                .areas(mid_col);

                self.draw_areas = Some(DrawAreas {
                    directory_path_rect,
                    entry_name_rect,
                    entry_info_rect,
                    canvas,
                    help_rect,
                    modal_rect,
                });

                CalcNewLayout
            }
            scanning_state @ Scanning { .. } => {
		self.render_scanning_modal(buf);

                scanning_state
            }
            Displaying(DisplayingContext {
                info_bar,
                cells,
                neighbours,
                help_bar,
            }) => {
                self.render_display(info_bar, &cells, help_bar, buf);

                Displaying(DisplayingContext {
                    info_bar,
                    cells,
                    neighbours,
                    help_bar,
                })
            }
            other => other,
        };

        self.status = new_status;
    }
}
