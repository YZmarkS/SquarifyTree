use std::path::PathBuf;

use ratatui::{
    layout::{Constraint, Layout, Spacing},
    prelude::*,
    symbols::merge::MergeStrategy,
    text::*,
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};

use crate::{inode::*, layout::*};
use crate::{main_loop::*, squarify::CELL_MIN_WIDTH};

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
        let directory_path = directory.full_path_with_trailing_separator();

        let info_name = selected_cell.content.info_bar_name();
        let info_usage = match selected_cell.content.cell_usage_string() {
            None => String::from("Empty"),
            Some(usage_string) => usage_string,
        };

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
            let cell = &(cells[i]);

            let mut lines = Vec::new();

            lines.push(Line::from(cell.cell_base_name()));
            if let Some(usage_string) = cell.content.cell_usage_string() {
                lines.push(Line::from(usage_string));
            }
            let text = Text::from(lines);

            let on_style = Style::new().fg(Color::White).bg(Color::DarkGray);
            let off_style = Style::new().fg(Color::White).bg(Color::Black);
            let outer_block = Block::bordered().merge_borders(MergeStrategy::Exact);
            let inner_block =
                Paragraph::new(text).style(if i == selected { on_style } else { off_style });
            let inner_area = outer_block.inner(cell.rect);

            outer_block.render(cell.rect, buf);
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

impl CellContent {
    pub fn info_bar_name(&self) -> String {
        match self {
            CellContent::Inode(inode) => inode.borrow().base_name_formatted_string(),
            CellContent::Other(_) => String::from("More Entries in Directory..."),
            CellContent::Empty => String::from("Empty Directory"),
        }
    }

    pub fn cell_usage_string(&self) -> Option<String> {
        match self {
            CellContent::Inode(inode) => Some(human_readable_usage(inode.clone().borrow().usage)),
            CellContent::Other(OtherCellContent { usage, .. }) => {
                Some(human_readable_usage(*usage))
            }
            CellContent::Empty => None,
        }
    }
}

impl Cell {
    pub fn cell_base_name(&self) -> String {
        let internal_width: usize = (self.rect.width - 2) as usize;
        match &self.content {
            CellContent::Inode(inode) => {
                let inode = inode.borrow();
                let base_name_pathbuf = inode.base_name_pathbuf();
                let base_name_string: String = base_name_pathbuf.to_string_lossy().into();
                if inode.is_reg() || inode.is_sym() {
                    if base_name_string.len() <= internal_width as usize {
                        base_name_string
                    } else {
                        let extension = base_name_pathbuf.extension().and_then(|ext| ext.to_str());
                        // If there is room for first char, a dot, and the extension
                        if extension
                            .clone()
                            .is_some_and(|ext| 1 + "[..]".len() + 1 + ext.len() <= internal_width)
                        {
                            if let Some(extension) = extension {
                                let stem = base_name_pathbuf.file_stem().unwrap().to_str().unwrap();
				let room = internal_width - ("[..]".len() + 1 + extension.len());
                                let back_len = room / 2;
                                let front_len = room - back_len;
                                format!(
                                    "{}[..]{}.{}",
                                    stem[..front_len].to_string(),
                                    stem[stem.len() - back_len..].to_string(),
                                    extension
                                )
                            } else {
                                panic!(
                                    "Render - cell_base_name: Cannot find extension for {:?}",
                                    base_name_pathbuf
                                )
                            }
                        } else {
			    let room = internal_width - "[..]".len();
                            let back_len = room / 2;
                            let front_len = room - back_len;
                            format!(
                                "{}[..]{}",
                                base_name_string[..front_len].to_string(),
                                base_name_string[base_name_string.len() - back_len..].to_string()
                            )
                        }
                    }
                } else if inode.is_dir() {
                    if base_name_string.len() <= (internal_width - 1) as usize {
                        base_name_pathbuf.join("").to_string_lossy().into_owned()
                    } else {
                        let room = internal_width - ("[..]".len() + 1);
                        let back_len = room / 2;
                        let front_len = room - back_len;
                        format!(
                            "{}[..]{}{}",
                            base_name_string[..front_len].to_string(),
                            base_name_string[base_name_string.len() - back_len..].to_string(),
                            std::path::MAIN_SEPARATOR
                        )
                    }
                } else {
                    panic!("Render - cell_base_name: Unknown inode type")
                }
            }
            CellContent::Other(_) => String::from("..."),
            CellContent::Empty => String::from("Empty Directory"),
        }
    }
}
