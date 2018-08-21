extern crate ggez;

use ggez::*;
use ggez::event::{Keycode, Mod};

#[derive(Debug)]
enum ActorType {
	Player,
}

#[derive(Debug)]
struct Actor {
	actor_type: ActorType,
	point: [f32; 2],
	// 位置 [y, x]
	velocity: [f32; 2],
	// 速度 [y, x]
}

impl Actor {
	fn player_new() -> Actor {
		Actor {
			actor_type: ActorType::Player,
			point: [300.0, 500.0],
			velocity: [0.0; 2],
		}
	}
}

#[derive(Debug)]
struct InputState {
	xaxis: f32,
	yaxis: f32,
	shift: bool,
	shot: bool,
}

impl Default for InputState {
	fn default() -> InputState {
		InputState {
			xaxis: 0.0,
			yaxis: 0.0,
			shift: false,
			shot: false,
		}
	}
}

#[derive(Debug)]
struct MainState {
	player: Actor,
	input: InputState,
}

impl MainState {
	fn new(ctx: &mut Context) -> GameResult<MainState> {
		let s = MainState{
			player: Actor::player_new(),
			input: InputState::default(),
		};

		Ok(s)
	}
}

impl ggez::event::EventHandler for MainState {
  fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
      Ok(())
  }

  fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
		graphics::clear(ctx);

		// **********
		// drow player circle
		let point = self.player.point;
		graphics::circle(
			ctx,
			graphics::DrawMode::Fill,
			graphics::Point2::new(point[0],point[1]),
			10.0,
			0.1,
		);
		// **********

		graphics::present(ctx);
	Ok(())
	}

	fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
		match keycode {
			Keycode::Up => {
				self.input.yaxis = -1.0;
			}
			Keycode::Down => {
				self.input.yaxis = 1.0;
			}
			Keycode::Right => {
				self.input.xaxis = 1.0;
			}
			Keycode::Left => {
				self.input.xaxis = -1.0;
			}
			Keycode::LShift => {
				self.input.shift = true;
			}
			Keycode::Z => {
				self.input.shot = true;
			}
			_ => ()
		}
	}

	fn key_up_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
		match keycode {
			Keycode::Up => {
				self.input.yaxis = 0.0;
			}
			Keycode::Down => {
				self.input.yaxis = 0.0;
			}
			Keycode::Right => {
				self.input.xaxis = 0.0;
			}
			Keycode::Left => {
				self.input.xaxis = 0.0;
			}
			Keycode::LShift => {
				self.input.shift = false;
			}
			Keycode::Z => {
				self.input.shot = false;
			}
			_ => ()
		}
	}
}
pub fn main() {
    let mut cb = ContextBuilder::new("poi-project", "ggez")
        .window_setup(conf::WindowSetup::default().title("poi-project"))
        .window_mode(conf::WindowMode::default().dimensions(1280, 960));

    let ctx = &mut cb.build().unwrap();

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
