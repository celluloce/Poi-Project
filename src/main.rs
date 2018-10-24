extern crate ggez;
extern crate poi_project;

use ggez::graphics;
use ggez::event::{self, Keycode, Mod};
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf;

use std::{path, env};

use poi_project::MainState;

const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 960;

pub fn main() {
	let mut cb = ContextBuilder::new("poi-project", "ggez")
		.window_setup(conf::WindowSetup::default().title("poi-project"))
		.window_mode(conf::WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT));

	if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
		let mut path = path::PathBuf::from(manifest_dir);
		path.push("resources");
		cb = cb.add_resource_path(path);
	} else {
		println!("Not building from cargo?  Ok.");
	}

	let ctx = &mut cb.build().unwrap();

	ctx.print_resource_stats();

	match MainState::new(ctx) {
		Err(e) => {
			println!("Could not load game!");
			println!("Error: {}", e);
		}
		Ok(ref mut game) => {
			let result = event::run(ctx, game);
			if let Err(e) = result {
				println!("Error encountered running game: {}", e);
			} else {
				println!("Game exited cleanly.");
			}
		}
	}
}
