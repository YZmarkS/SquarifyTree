use crate::logic::*;
use crate::squarify::HEIGHT_TO_WIDTH_RATIO;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout};
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

        let old_status = replace(&mut self.status, Swapping);

        let new_status: Status = match old_status {
            GetCanvasSize => {
                let [info_bar, canvas, help_bar] = Layout::vertical(vec![
                    Constraint::Length(3),
                    Constraint::Fill(1),
                    Constraint::Length(3),
                ])
                .areas(area);

                CalcNewLayout(NewLayoutContext {
                    info_bar,
                    canvas,
                    help_bar,
                })
            }
            scanning_state @ Scanning { .. } => {
                Block::bordered().render(area, buf);

                let [_, mid_col, _] = Layout::new(
                    Direction::Horizontal,
                    vec![
                        Constraint::Length(2 * HEIGHT_TO_WIDTH_RATIO),
                        Constraint::Max(50),
                        Constraint::Length(2 * HEIGHT_TO_WIDTH_RATIO),
                    ],
                )
                .flex(Flex::Center)
                .areas(area);
                let [_, mid_box, _] = Layout::new(
                    Direction::Vertical,
                    vec![
                        Constraint::Length(2),
                        Constraint::Max(9),
                        Constraint::Length(2),
                    ],
                )
                .flex(Flex::Center)
                .areas(mid_col);
                let mid_text_area = mid_box.centered(Constraint::Max(12), Constraint::Max(1));

                let dots = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    / 250
                    % 4;
                let scanning_string = format!("Scanning{}", ".".repeat(dots as usize));

                Block::bordered().render(mid_box, buf);
                Paragraph::new(scanning_string).render(mid_text_area, buf);

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
