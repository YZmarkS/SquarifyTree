use std::cmp;
use std::rc::Rc;
use StackDirection::Horizontal;
use ratatui::symbols::line::VERTICAL;

use crate::scanner::*;
use crate::inode::*;

type Measure = u64;

pub enum StackDirection {
    Vertical,
    Horizontal,
}

const HEIGHT_TO_WIDTH_RATIO: Measure = 2;
const CELL_MIN_MEASURE: Measure = 2;
const SPLIT_THRESHOLD: Measure = 4;

fn is_landscape(width: Measure, height: Measure) -> bool {
    height * HEIGHT_TO_WIDTH_RATIO <= width
}

fn calc_stack_breadth(usage: DiskUsage,
		      total_usage: DiskUsage,
		      total_breadth: Measure) -> Measure {
    let precise_breadth = (total_breadth as f64) * (usage as f64) / (total_usage as f64);
    let rounded_breadth = precise_breadth.round() as Measure;

    cmp::max(rounded_breadth, CELL_MIN_MEASURE)
}

fn calc_stack_spans(inodes: &[Inode],
		    total_span: Measure) -> Vec<Measure> {
    let total_usage: DiskUsage = inodes.iter().map(|inode| inode.usage).sum();
    let mut spans = Vec::new();

    let mut remaining_span = total_span;

    println!("total usage: {}, total_span: {}",
	     total_usage, total_span);

    for inode in inodes {
	let inode_usage = inode.usage;
	let precise_span = (total_span as f64) * (inode_usage as f64) / (total_usage as f64);
	let truncated_span = precise_span.trunc() as Measure;

	println!("Inode: {}, Inode usage: {}, precise: {}, truncated: {}",
		 inode.path.file_name().unwrap().display(),
		 inode_usage,
		 precise_span,
		 truncated_span);
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

fn worst_aspect_ratio(spans: &[Measure],
		      breadth: Measure,
		      direction: &StackDirection) -> f64 {
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
	    (normalized_breadth / normalized_span)
	    .max(normalized_span / normalized_breadth);
	worst = worst.max(aspect_ratio);
    }

    worst
}

pub enum LayoutDetail {
    Split(SplitDetail),
    Whole,
}

pub struct SplitDetail {
    breadth: Measure,
    spans: Vec<Measure>,
    other: Box<SquarifyLayout>,
}

pub struct SquarifyLayout {
    width: Measure,
    height: Measure,
    detail: LayoutDetail,
}

struct StackInfo {
    breadth: Measure,
    other_breadth: Measure,
    spans: Vec<Measure>,
    min_span: Measure,
    worst_ratio: f64,
}

fn calc_stack_info(inodes: &[Inode], take: usize,
		   total_usage: DiskUsage,
		   total_breadth: Measure, total_span: Measure,
		   direction: &StackDirection)
		   -> StackInfo {
    assert_ne!(inodes.len(), 0);

    let subtotal_usage = inodes[..take].iter().map(|inode| inode.usage).sum();
    let stack_breadth = calc_stack_breadth(subtotal_usage, total_usage, total_breadth);
    let other_breadth = total_breadth - stack_breadth;
    let stack_spans = calc_stack_spans(&(inodes[..take]), total_span);
    let min_span = (&stack_spans).iter().min().cloned().unwrap();
    let worst_ratio = worst_aspect_ratio(&stack_spans, stack_breadth, &direction);

    StackInfo {
	breadth: stack_breadth,
	other_breadth,
	spans: stack_spans,
	min_span,
	worst_ratio,
    }
}


pub fn squarify(inodes: &[Inode],
		width: Measure, height: Measure) -> SquarifyLayout {

    let is_landscape = is_landscape(width, height);
    let total_breadth = if is_landscape { width } else { height };

    if inodes.len() == 0 || total_breadth < SPLIT_THRESHOLD {
	return SquarifyLayout {
	    width,
	    height,
	    detail: LayoutDetail::Whole,
	}
    }

    use StackDirection::*;

    let total_span = if is_landscape { height } else { width };
    let direction = if is_landscape { Vertical } else { Horizontal };

    let total_usage: DiskUsage = inodes.iter().map(|inode| inode.usage).sum();

    let mut take = 1;
    let mut stack_info = calc_stack_info(inodes, take,
					 total_usage, total_breadth, total_span,
					 &direction);

    if CELL_MIN_MEASURE <= stack_info.other_breadth {
	while take + 1 <= inodes.len() {
	    let candidate_stack_info = calc_stack_info(inodes, take + 1,
						       total_usage, total_breadth, total_span,
						       &direction);

	    let no_space_for_other = take + 1 < inodes.len()
		&& candidate_stack_info.other_breadth < CELL_MIN_MEASURE;
	    let some_layer_too_short = candidate_stack_info.min_span < CELL_MIN_MEASURE;
	    let worst_is_worse = stack_info.worst_ratio < candidate_stack_info.worst_ratio;

	    if no_space_for_other || some_layer_too_short || worst_is_worse {
		break;
	    }

	    take += 1;
	    stack_info = candidate_stack_info;
	}
    }

    let corrected_other_breadth = cmp::max(stack_info.other_breadth, CELL_MIN_MEASURE);
    let corrected_breadth = total_breadth - corrected_other_breadth;

    let (sub_width, sub_height) = if is_landscape {
	(width - corrected_breadth, height)
    } else {
	(width, height - corrected_breadth)
    };
    let sub_layout_box = Box::new(squarify(&(inodes[take..]),
					   sub_width, sub_height));

    SquarifyLayout {
	width,
	height,
	detail: LayoutDetail::Split( SplitDetail {
	    breadth: stack_info.breadth,
	    spans: stack_info.spans,
	    other: sub_layout_box,
	}),
    }
}

pub fn print_layout(layout: &SquarifyLayout, indent: u8) {
    for _ in 0..indent {
	print!("    ")
    }
    println!("Width: {}, Height: {}", layout.width, layout.height);
    if let LayoutDetail::Split(detail) = &layout.detail {
	for span in &detail.spans {
	    for _ in 0..indent  {
		print!("    ")
	    }
	    println!("{}", span);
	}
	print_layout(&detail.other, indent + 1);
    }
}
