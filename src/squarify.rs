use std::cell::RefCell;
use std::cmp;
use std::rc::Rc;

use crate::inode::*;

pub type Measure = u16;

pub enum StackDirection {
    Vertical,
    Horizontal,
}

const HEIGHT_TO_WIDTH_RATIO: Measure = 2;

pub const CELL_MIN_WIDTH: Measure = 8;
pub const CELL_MIN_HEIGHT: Measure = 3;

const SPLIT_LEFT_RIGHT_THRESHOLD: Measure = 16;
const SPLIT_TOP_BOTTOM_THRESHOLD: Measure = 6;

pub fn is_landscape(width: Measure, height: Measure) -> bool {
    height * HEIGHT_TO_WIDTH_RATIO <= width
}

fn calc_stack_spans(inodes: &[Rc<RefCell<Inode>>], total_span: Measure) -> Vec<Measure> {
    let total_usage: DiskUsage = inodes.iter().map(|inode| inode.borrow().usage).sum();

    // What happens when all the inodes have 0 usage.
    // TODO: You gotta fix this at some point man, please, this is so ugly
    if total_usage == 0 {
        let mut spans = Vec::new();
        let len = inodes.len();
        let base_span = total_span / len as Measure;
        let remainder = total_span % len as Measure;
        for _ in 0..remainder {
            spans.push(base_span + 1);
        }
        for _ in (remainder as usize)..len {
            spans.push(base_span);
        }
        return spans;
    }

    let mut spans = Vec::new();

    let mut remaining_span = total_span;

    for inode in inodes {
        let inode_usage = inode.borrow().usage;
        let precise_span = (total_span as f64) * (inode_usage as f64) / (total_usage as f64);

        let truncated_span = precise_span.trunc() as Measure;

        spans.push(truncated_span);

        remaining_span -= truncated_span;
    }

    for span in &mut spans {
        if 0 < remaining_span {
            (*span) += 1;
            remaining_span -= 1;
        }
    }

    assert_eq!(remaining_span, 0);
    spans
}

/* Wost Aspect Ratio

Precondition: [spans] and [breadth] must be non-zero
 */
fn worst_aspect_ratio(spans: &[Measure], breadth: Measure, direction: &StackDirection) -> f64 {
    let mut worst: f64 = 0.0;
    let span_adjustment = match direction {
        StackDirection::Vertical => HEIGHT_TO_WIDTH_RATIO,
        StackDirection::Horizontal => 1,
    };
    let breadth_adjustment = match direction {
        StackDirection::Vertical => 1,
        StackDirection::Horizontal => HEIGHT_TO_WIDTH_RATIO,
    };

    for span in spans {
        let normalized_span = (span * span_adjustment) as f64;
        let normalized_breadth = (breadth * breadth_adjustment) as f64;
        let aspect_ratio =
            (normalized_breadth / normalized_span).max(normalized_span / normalized_breadth);
        worst = worst.max(aspect_ratio);
    }

    worst
}

pub enum LayoutDetail {
    Split(SplitDetail),
    Whole,
}

pub struct SplitDetail {
    pub breadth: Measure,
    pub spans: Vec<Measure>,
    pub other: Box<Squarify>,
}

pub struct Squarify {
    pub width: Measure,
    pub height: Measure,
    pub detail: LayoutDetail,
}

pub struct StackInfo {
    stack_breadth: Measure,
    stack_spans: Vec<Measure>,
    worst_ratio: f64,
}

fn calc_stack_info(
    inodes: &[Rc<RefCell<Inode>>],
    total_usage: DiskUsage,
    total_breadth: Measure,
    total_span: Measure,
    direction: &StackDirection,
) -> StackInfo {
    assert_ne!(inodes.len(), 0);

    let subtotal_usage: DiskUsage = inodes.iter().map(|inode| inode.borrow().usage).sum();

    let precise_stack_breadth =
        (total_breadth as f64) * (subtotal_usage as f64) / (total_usage as f64);
    let rounded_stack_breadth = precise_stack_breadth.round() as Measure;
    let stack_spans = calc_stack_spans(&inodes, total_span);
    let worst_ratio = worst_aspect_ratio(&stack_spans, rounded_stack_breadth, &direction);

    StackInfo {
        stack_breadth: rounded_stack_breadth,
        stack_spans,
        worst_ratio,
    }
}

pub fn squarify(inodes: &[Rc<RefCell<Inode>>], width: Measure, height: Measure) -> Squarify {
    let is_landscape = is_landscape(width, height);
    let total_breadth = if is_landscape { width } else { height };
    let split_threshold = if is_landscape {
        SPLIT_LEFT_RIGHT_THRESHOLD
    } else {
        SPLIT_TOP_BOTTOM_THRESHOLD
    };

    if inodes.len() == 0 || total_breadth < split_threshold {
        return Squarify {
            width,
            height,
            detail: LayoutDetail::Whole,
        };
    }

    let (cell_min_breadth, cell_min_span) = if is_landscape {
        (CELL_MIN_WIDTH, CELL_MIN_HEIGHT)
    } else {
        (CELL_MIN_HEIGHT, CELL_MIN_WIDTH)
    };

    use StackDirection::*;

    let total_span = if is_landscape { height } else { width };
    let direction = if is_landscape { Vertical } else { Horizontal };

    let total_usage: DiskUsage = inodes.iter().map(|inode| inode.borrow().usage).sum();

    let mut take = 1;
    let mut stack_info = calc_stack_info(
        &inodes[..take],
        total_usage,
        total_breadth,
        total_span,
        &direction,
    );

    if cell_min_breadth <= total_breadth - stack_info.stack_breadth {
        while take + 1 <= inodes.len() {
            let candidate_stack_info = calc_stack_info(
                &inodes[..(take + 1)],
                total_usage,
                total_breadth,
                total_span,
                &direction,
            );

            let no_space_for_other = take + 1 < inodes.len()
                && (total_breadth - candidate_stack_info.stack_breadth) < cell_min_breadth;
            let some_layer_too_short =
                *candidate_stack_info.stack_spans.iter().min().unwrap() < cell_min_span;
            let worst_is_worse = stack_info.worst_ratio < candidate_stack_info.worst_ratio;

            if no_space_for_other || some_layer_too_short || worst_is_worse {
                break;
            }

            take += 1;
            stack_info = candidate_stack_info;
        }
    }

    let corrected_stack_breadth = if take == inodes.len() {
	total_breadth

    } else {
	cmp::min(cmp::max(cell_min_breadth, stack_info.stack_breadth),
		 total_breadth - cell_min_breadth)
    };

    let (sub_width, sub_height) = if is_landscape {
        (width - corrected_stack_breadth, height)
    } else {
        (width, height - corrected_stack_breadth)
    };

    let sub_layout_box = Box::new(squarify(&(inodes[take..]), sub_width, sub_height));

    Squarify {
        width,
        height,
        detail: LayoutDetail::Split(SplitDetail {
            breadth: corrected_stack_breadth,
            spans: stack_info.stack_spans,
            other: sub_layout_box,
        }),
    }
}

#[allow(unused)]
pub fn print_layout(layout: &Squarify, indent: u8) {
    for _ in 0..indent {
        eprint!("    ")
    }
    if let LayoutDetail::Split(detail) = &layout.detail {
        eprintln!(
            "Orientation: {}, Width: {}, Height: {}, Breadth: {} ",
            if is_landscape(layout.width, layout.height) {
                "Landscape"
            } else {
                "Portrait"
            },
            layout.width,
            layout.height,
            detail.breadth
        );
        for span in &detail.spans {
            for _ in 0..indent {
                eprint!("    ")
            }
            eprintln!("{}", span);
        }
        print_layout(&detail.other, indent + 1);
    }
}
