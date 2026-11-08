use ratatui::{
    layout::{Constraint, Flex, Layout, Spacing},
    prelude::*,
    symbols::merge::MergeStrategy,
    widgets::{Block, Paragraph},
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{logic::*, squarify::HEIGHT_TO_WIDTH_RATIO};
use crate::{inode::*, layout::*};

// https://en.wikipedia.org/wiki/Kilobyte
const SI_STEP_SIZE: u128 = 1000;
const IEC_STEP_SIZE: u128 = 1024;

impl State {
    pub fn render_display(
        &mut self,
        info_bar: Rect,
        cells: &Vec<Cell>,
        help_bar: Rect,
        buf: &mut Buffer,
    ) {
        let [directory_block, file_name_block, file_info_block] =
            Layout::horizontal(vec![Constraint::Fill(1); 3])
                .spacing(Spacing::Overlap(1))
                .areas(info_bar);

        self.render_info_bar(
            directory_block,
            file_name_block,
            file_info_block,
            &cells[self.now.selected],
            buf,
        );

        self.render_cells(cells, buf);

        self.render_help_bar(help_bar, buf);
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

            lines.push(Line::from(cell.cell_display_name()));
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

    pub fn render_scanning_modal(&self, buf: &mut Buffer) {
        let modal_rect = if let Some(DrawAreas { modal_rect, .. }) = self.draw_areas {
            modal_rect
        } else {
            panic!("Render: No modal box in state to render scanning modal")
        };
        let dots = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            / 250
            % 4;
        let scanning_string = format!("Scanning{}", ".".repeat(dots as usize));
        let modal_text_area = modal_rect.centered(Constraint::Max(12), Constraint::Max(1));

        Block::bordered().render(modal_rect, buf);
        Paragraph::new(scanning_string).render(modal_text_area, buf);
    }
}

pub fn area_to_draw_areas(area: Rect) -> DrawAreas {
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

    DrawAreas {
        directory_path_rect,
        entry_name_rect,
        entry_info_rect,
        canvas,
        help_rect,
        modal_rect,
    }
}

fn human_readable_usage(usage: SizeUnit) -> String {
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
        String::from("???")
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
    pub fn cell_display_name(&self) -> String {
        let internal_width: usize = (self.rect.width - 2) as usize;
        match &self.content {
            CellContent::Inode(inode) => {
                let inode = inode.borrow();
                let base_name_pathbuf = inode.base_name_pathbuf();
                let base_name_string: String = base_name_pathbuf.to_string_lossy().into();

                /* File or Link File Name

                Attempts to include the starting character and the full extension.
                If not possible, shrink the entire file name.
                 */
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
                }
                /* Directory Name

                Only attempt to include at the first and last character.
                 */
                else if inode.is_dir() {
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
            CellContent::Other(_) => String::from("[...]"),
            CellContent::Empty => String::from("There's nothing more..."),
        }
    }
}
