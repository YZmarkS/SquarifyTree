use ratatui::{layout::*, prelude::*};
use std::{cell::RefCell, cmp, collections::HashMap, rc::Rc};

use crate::inode::*;
use crate::squarify::*;

pub fn squarify_to_rects(squarify: &Squarify, canvas: Rect) -> Vec<Rect> {
    let mut list = squarify_to_rects_aux(squarify, canvas);
    list.reverse();
    list
}

fn squarify_to_rects_aux(squarify: &Squarify, canvas: Rect) -> Vec<Rect> {
    match &(squarify.detail) {
        LayoutDetail::Whole => {
            let mut rects = Vec::new();
            if 0 < squarify.width && 0 < squarify.height {
                rects.push(canvas);
            }
            rects
        }
        LayoutDetail::Split(detail) => {
            let is_landscape = is_landscape(squarify.width, squarify.height);
            let breadth = detail.breadth;
            let other_breadth = if is_landscape {
                squarify.width - breadth
            } else {
                squarify.height - breadth
            };

            /* [+ 1]s

            There are many [+ 1] in the calculation. It is a result of how we
            interpret the value of width and height.
            */
            let split_constraints = vec![
                Constraint::Length(detail.breadth + 1),
                Constraint::Length(other_breadth + 1),
            ];
            let stack_constrains_iter = detail
                .spans
                .iter()
                .map(|span| Constraint::Length(span.to_owned() + 1));

            use ratatui::prelude::Direction::*;
            let split_rects = Layout::default()
                .direction(if is_landscape { Horizontal } else { Vertical })
                .spacing(Spacing::Overlap(1))
                .constraints(split_constraints)
                .split(canvas);

            let stack = split_rects[0];
            let other = split_rects[1];

            let stack_rects = Layout::default()
                .direction(if is_landscape { Vertical } else { Horizontal })
                .spacing(Spacing::Overlap(1))
                .constraints(stack_constrains_iter)
                .split(stack);

            let mut sub_rects = squarify_to_rects_aux(&detail.other, other);
            for stack_rect in stack_rects.iter().rev() {
                sub_rects.push(*stack_rect);
            }

            sub_rects
        }
    }
}

fn x_interval(rect: Rect) -> (Measure, Measure) {
    (rect.x, rect.x + rect.width)
}

fn y_interval(rect: Rect) -> (Measure, Measure) {
    (rect.y, rect.y + rect.height)
}

fn calc_overlap((low1, high1): (Measure, Measure), (low2, high2): (Measure, Measure)) -> i32 {
    let max_low = cmp::max(low1, low2);
    let min_high = cmp::min(high1, high2);
    i32::from(min_high) - i32::from(max_low)
}

fn calc_offset((low1, high1): (Measure, Measure), (low2, high2): (Measure, Measure)) -> i32 {
    let mid1 = i32::from((high1 - low1) / 2 + low1);
    let mid2 = i32::from((high2 - low2) / 2 + low2);
    (mid2 - mid1).abs()
}

pub struct Cell {
    pub rect: Rect,
    pub content: CellContent,
}

pub enum CellContent {
    Inode(Rc<RefCell<Inode>>),
    Other(OtherCellContent),
    Empty,
}

pub struct OtherCellContent {
    pub starting_index: usize,
    pub usage: DiskUsage,
}

impl CellContent {
    pub fn info_bar_name(&self) -> String {
	match self {
	    CellContent::Inode(inode) => inode.borrow().base_name_string(),
	    CellContent::Other(_) => String::from("More Entries in Directory..."),
	    CellContent::Empty => String::from("No more entries"),
	}
    }

    pub fn usage(&self) -> Option<DiskUsage> {
	match self {
            CellContent::Inode(inode) => Some(inode.clone().borrow().usage),
            CellContent::Other(OtherCellContent { usage, .. }) => Some(*usage),
            CellContent::Empty => None
	}
    }

    pub fn cell_base_name(&self) -> String {
	match self {
	    CellContent::Inode(inode) => inode.borrow().base_name_string(),
	    CellContent::Other(_) => String::from("Other..."),
	    CellContent::Empty => String::from("Empty"),
	}
    }

}

pub type CellId = usize;

pub struct Neighbours {
    pub top: Option<CellId>,
    pub bottom: Option<CellId>,
    pub left: Option<CellId>,
    pub right: Option<CellId>,
}

pub fn make_cell_list(
    rects: &Vec<Rect>,
    inodes: &Vec<Rc<RefCell<Inode>>>,
    starting_index: usize,
) -> Vec<Cell> {
    assert!(0 < rects.len());
    use CellContent::*;
    let mut cells: Vec<Cell> = Vec::new();
    if inodes.len() <= starting_index {
        cells.push(Cell {
            rect: rects[0],
            content: Empty,
        });
        return cells;
    }

    for i in 0..(rects.len() - 1) {
        cells.push(Cell {
            rect: rects[i].to_owned(),
            content: Inode(inodes[starting_index + i].clone()),
        });
    }
    cells.push(Cell {
        rect: rects[rects.len() - 1].to_owned(),
        content: if starting_index + rects.len() == inodes.len() {
            Inode(inodes[starting_index + rects.len() - 1].clone())
        } else {
            Other(OtherCellContent {
                starting_index: starting_index + rects.len() - 1,
                usage: inodes[starting_index + rects.len() - 1..]
                    .iter()
                    .map(|inode| inode.borrow().usage)
                    .sum(),
            })
        },
    });
    cells
}

type CellIdByCoord = HashMap<Measure, Vec<CellId>>;

fn push_by_key<K: Eq + core::hash::Hash, V>(map: &mut HashMap<K, Vec<V>>, key: K, value: V) {
    if map.contains_key(&key) {
        map.get_mut(&key).unwrap().push(value);
    } else {
        map.insert(key, vec![value]);
    }
}

enum Ends {
    TopBottom,
    LeftRight,
}

fn group_by_endpoint(cells: &Vec<Cell>, ends_to_group_by: Ends) -> (CellIdByCoord, CellIdByCoord) {
    let (mut by_low, mut by_high) = (HashMap::new(), HashMap::new());

    for i in 0..(cells.len()) {
        let (low, high) = match ends_to_group_by {
            Ends::TopBottom => y_interval(cells[i].rect),
            Ends::LeftRight => x_interval(cells[i].rect),
        };
        push_by_key(&mut by_low, low, i);
        push_by_key(&mut by_high, high, i);
    }

    (by_low, by_high)
}

fn find_most_overlapping(
    cells: &Vec<Cell>,
    candidates: Option<&Vec<CellId>>,
    cell: &Cell,
    ends_to_compare_by: Ends,
) -> Option<CellId> {
    let candidates = candidates?;

    let mut best_candidate = None;
    let mut best_overlap = -1;
    let mut best_offset = i32::MAX;

    let cell_interval = match ends_to_compare_by {
        Ends::TopBottom => y_interval(cell.rect),
        Ends::LeftRight => x_interval(cell.rect),
    };

    for candidate in candidates {
        let candidate_interval = match ends_to_compare_by {
            Ends::TopBottom => y_interval(cells[*candidate].rect),
            Ends::LeftRight => x_interval(cells[*candidate].rect),
        };

        let overlap = calc_overlap(candidate_interval, cell_interval);
        let offset = calc_offset(candidate_interval, cell_interval);

        if best_overlap < overlap {
            best_overlap = overlap;
            best_candidate = Some(*candidate);
        } else if best_overlap == overlap && offset < best_offset {
            best_offset = offset;
            best_candidate = Some(*candidate);
        }
    }
    best_candidate
}

pub fn make_neighbours(cells: &Vec<Cell>) -> Vec<Neighbours> {
    use Ends::*;
    let mut neighbours = Vec::new();

    let (by_top, by_bottom) = group_by_endpoint(cells, TopBottom);
    let (by_left, by_right) = group_by_endpoint(cells, LeftRight);

    for cell in cells.iter() {
        let (top, bottom) = y_interval(cell.rect);
        let (left, right) = x_interval(cell.rect);

        let top_neighbour =
            find_most_overlapping(cells, by_bottom.get(&(top + 1)), cell, LeftRight);
        let bottom_neighbour =
            find_most_overlapping(cells, by_top.get(&(bottom - 1)), cell, LeftRight);
        let left_neighbour =
            find_most_overlapping(cells, by_right.get(&(left + 1)), cell, TopBottom);
        let right_neighbour =
            find_most_overlapping(cells, by_left.get(&(right - 1)), cell, TopBottom);

        neighbours.push(Neighbours {
            top: top_neighbour,
            bottom: bottom_neighbour,
            left: left_neighbour,
            right: right_neighbour,
        });
    }
    neighbours
}
