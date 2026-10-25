use std::path::PathBuf;

use ratatui::{
    layout::{Constraint, Layout, Spacing},
    prelude::*,
    symbols::merge::MergeStrategy,
    text::*,
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};

use crate::main_loop::*;
use crate::{inode::*, layout::*};

// https://en.wikipedia.org/wiki/Kilobyte
const SI_STEP_SIZE: u128 = 1000;
const IEC_STEP_SIZE: u128 = 1024;

impl State {
    pub fn main_render(&mut self, area: Rect, buf: &mut Buffer, cells: &Vec<Cell>) {
        let [top, _, bottom] = Layout::vertical(vec![
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .areas(area);
        let [directory_block, file_name_block, file_info_block] =
            Layout::horizontal(vec![Constraint::Fill(1); 3])
                .spacing(Spacing::Overlap(1))
                .areas(top);

        self.render_info_bar(
            directory_block,
            file_name_block,
            file_info_block,
            &cells[self.now.selected],
            buf,
        );

        self.render_cells(cells, buf);

        self.render_help_bar(bottom, buf);
    }

    fn render_info_bar(
        &self,
        directory_block: Rect,
        file_name_block: Rect,
        file_info_block: Rect,
        selected_cell: &Cell,
        buf: &mut Buffer,
    ) {
        let directory = self.now.directory.borrow();
        let directory_path = directory.path_directory_string();

        let info_name = selected_cell.content.info_bar_name();
        let info_usage = human_readable_usage(match selected_cell.content.usage() {
            None => 0,
            Some(usage) => usage,
        });

        Paragraph::new(directory_path)
            .block(Block::bordered().merge_borders(MergeStrategy::Exact))
            .render(directory_block, buf);

        Paragraph::new(info_name)
            .block(Block::bordered().merge_borders(MergeStrategy::Exact))
            .render(file_name_block, buf);

        Paragraph::new(info_usage)
            .block(Block::bordered().merge_borders(MergeStrategy::Exact))
            .render(file_info_block, buf);
    }

    fn render_cells(&self, cells: &Vec<Cell>, buf: &mut Buffer) {
        let selected = self.now.selected;
        for i in 0..cells.len() {
            let Cell { rect, content } = &(cells[i]);

            let mut lines = Vec::new();
            let cell_name = content.cell_base_name();
            let maybe_cell_usage = content.usage();

	    if cell_name.len() + 2 <= (rect.width as usize) {
		lines.push(Line::from(cell_name));
	    }
            if let Some(usage) = maybe_cell_usage {
		let readable_usage = human_readable_usage(usage);
		if readable_usage.len() + 2 <= rect.width as usize {
                    lines.push(Line::from(readable_usage));
		}
            }
            let text = Text::from(lines);

            let on_style = Style::new().fg(Color::White).bg(Color::DarkGray);
            let off_style = Style::new().fg(Color::White).bg(Color::Black);
            let outer_block = Block::bordered().merge_borders(MergeStrategy::Exact);
            let inner_block =
                Paragraph::new(text).style(if i == selected { on_style } else { off_style });
            let inner_area = outer_block.inner(*rect);

            outer_block.render(*rect, buf);
            inner_block.render(inner_area, buf);
        }
    }

    fn render_help_bar(&mut self, helper_bar: Rect, buf: &mut Buffer) {
        Paragraph::new("Arrow Keys: Movement    Enter: Open selected    Backspace: Open Parent    r: Refresh current directory")
	    .block(Block::bordered())
	    .render(helper_bar, buf);
    }
}

fn human_readable_usage(usage: DiskUsage) -> String {
    let mut unit_size = 1;
    while usage / (unit_size * IEC_STEP_SIZE) != 0 {
        unit_size *= IEC_STEP_SIZE;
    }

    let value = usage / unit_size;
    let unit = if unit_size == IEC_STEP_SIZE.pow(0) {
        String::from("B")
    } else if unit_size == IEC_STEP_SIZE.pow(1) {
        String::from("KiB")
    } else if unit_size == IEC_STEP_SIZE.pow(2) {
        String::from("MiB")
    } else if unit_size == IEC_STEP_SIZE.pow(3) {
        String::from("GiB")
    } else if unit_size == IEC_STEP_SIZE.pow(4) {
        String::from("TiB")
    } else if unit_size == IEC_STEP_SIZE.pow(5) {
        String::from("PiB")
    } else if unit_size == IEC_STEP_SIZE.pow(6) {
        String::from("EiB")
    } else if unit_size == IEC_STEP_SIZE.pow(7) {
        String::from("ZiB")
    } else if unit_size == IEC_STEP_SIZE.pow(8) {
        String::from("YiB")
    } else if unit_size == IEC_STEP_SIZE.pow(9) {
        String::from("RiB")
    } else if unit_size == IEC_STEP_SIZE.pow(10) {
        String::from("QiB")
    } else {
        String::from("???iB")
    };

    format!("{value} {unit}")
}
