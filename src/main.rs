use std::path::Path;

use inode::InodeDetail;
use ratatui::{
    prelude::*,
    layout::{Constraint, Layout, Spacing},
    symbols::merge::MergeStrategy,
    widgets::{Paragraph, Block, Borders},
    DefaultTerminal, Frame,
};

mod scanner;
mod squarify;
mod inode;
use scanner::*;
use squarify::*;
use inode::*;


// fn main() -> color_eyre::Result<()> {
//     color_eyre::install()?;
//     ratatui::run(app)?;
//     Ok(())
// }

// fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
//     loop {
//         terminal.draw(render)?;
//         if crossterm::event::read()?.is_key_press() {
//             break Ok(());
//         }
//     }
// }

// fn render(frame: &mut Frame) {
//     let layout = Layout::default()
// 	.direction(Direction::Vertical)
// 	.constraints(vec![
//             Constraint::Length(5),
//             Constraint::Length(5),
// 	])
// 	.spacing(Spacing::Overlap(1))
// 	.split(frame.area());
//     frame.render_widget(
// 	Paragraph::new("Top long long long long long long long long long text")
//             .block(Block::new()
// 		   .borders(Borders::ALL)
// 		   .merge_borders(MergeStrategy::Exact)),
// 	layout[0]);
//     frame.render_widget(
// 	Paragraph::new("Bottom long text")
//             .block(Block::new()
// 		   .borders(Borders::ALL)
// 		   .merge_borders(MergeStrategy::Exact)),
// 	layout[1]);


// }

fn main() {

    println!("Hello, world!");

    let path = Path::new("/home/lemon/Projects/SquarifyTree/C/src");
    let worker_count = 10;

    let root_inode = path_to_inode(path, worker_count);
    print_inodes(&root_inode, 0);

    if let InodeDetail::Dir(children) = &(root_inode.detail) {
	let layout = squarify(&children[..], 200, 50);
	print_layout(&layout, 0);
    }

    println!("DONE!");
}
