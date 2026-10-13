use ratatui::{
    prelude::*,
    layout::{Constraint, Layout, Spacing},
    symbols::merge::MergeStrategy,
    widgets::{Paragraph, Block, Borders},
    DefaultTerminal, Frame,
};

use crate::{layout::get_cell_content_name, logic::AppState};

impl AppState {

    pub fn render_helper(&mut self, area: Rect, buf: &mut Buffer) {
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
