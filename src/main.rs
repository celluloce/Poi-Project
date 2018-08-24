extern crate ggez;

use ggez::graphics;
use ggez::event::{self, Keycode, Mod};
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf;

const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 960;

#[derive(Debug)]
enum ActorType {
	Player,
	Shot,
}

#[derive(Debug)]
struct Actor {
	actor_type: ActorType,
	point: [f32; 2],
	// 位置 [x, y]
	velocity: [f32; 2],
	// 1秒の移動距離 [x, y]
}

impl Actor {
	fn player_new() -> Actor {
		Actor {
			actor_type: ActorType::Player,
			point: [300.0, 500.0],
			velocity: [0.0; 2],
		}
	}
	fn shot_new(p_point: [f32; 2]) -> Actor {
		Actor {
			actor_type: ActorType::Shot,
			point: p_point,
			velocity: [0.0, -3000.0]
		}
	}
	fn update_point(actor: &mut Actor, dt: f32) {
		let mut x_vel = actor.velocity[0];
		let mut y_vel = actor.velocity[1];
		let s_width = SCREEN_WIDTH as f32;
		let s_height = SCREEN_HEIGHT as f32;

		if actor.point[0] < 0.0 && x_vel < 0.0 {
			x_vel = 0.0;
		}
		if actor.point[0] > s_width && x_vel > 0.0 {
			x_vel = 0.0;
		}
		if actor.point[1] < 0.0 && y_vel < 0.0 {
			y_vel = 0.0;
		}
		if actor.point[1] > s_height && y_vel > 0.0 {
			y_vel = 0.0;
		}

		actor.point[0] += x_vel * dt;
		actor.point[1] += y_vel * dt;
	}
}

#[derive(Debug, Clone, Copy)]
struct InputState {
	up: f32,
	down: f32,
	right: f32,
	left: f32,
	shift: bool,
	shot: bool,
}

impl Default for InputState {
	fn default() -> InputState {
		InputState {
			up: 0.0,
			down: 0.0,
			right: 0.0,
			left: 0.0,
			shift: false,
			shot: false,
		}
	}
}

#[derive(Debug)]
struct MainState {
	player: Actor,
	shots: Vec<Actor>,
	input: InputState,
}

impl MainState {
	fn new(ctx: &mut Context) -> GameResult<MainState> {
		let s = MainState{
			player: Actor::player_new(),
			shots: Vec::with_capacity(50),
			input: InputState::default(),
		};

		Ok(s)
	}

}

impl ggez::event::EventHandler for MainState {
	fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
		const FPS: u32 = 60;
		let seconds = 1.0 / FPS as f32;

		while timer::check_update_time(ctx, FPS) {
			// Update player point
			// キーインプット基底ベクトルをInputState値として定める
			// -> InputState値*スカラ値=ActorVelocity
			// -> ActorVelocity*1Frameあたりかかる秒=1Frameあたり進む距離
			let s_input = self.input;
			if !s_input.shift {
				// 高速移動
				self.player.velocity[0] = (s_input.right + s_input.left) * 200.0;
				self.player.velocity[1] = (s_input.up + s_input.down) * 200.0;
				// 低速Shift移動
			} else {
				self.player.velocity[0] = (s_input.right + s_input.left) * 100.0;
				self.player.velocity[1] = (s_input.up + s_input.down) * 100.0;
			}
			Actor::update_point(&mut self.player, seconds);

			// Update shot state
			if self.input.shot {
				// InputStateのshotがtrueの時、shotをVectorに入れる
				self.shots.push(Actor::shot_new(self.player.point))
			}
			for act in &mut self.shots {
				Actor::update_point(act, seconds);
			}
		}
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
		graphics::clear(ctx);

		let point = self.player.point;

		// drow player circle
		graphics::circle(
			ctx,
			graphics::DrawMode::Fill,
			graphics::Point2::new(point[0],point[1]),
			10.0,
			0.1,
		);

		// drow shot rectangle
		for act in &mut self.shots {
			let point = act.point;
			graphics::rectangle(
				ctx,
				graphics::DrawMode::Fill,
				graphics::Rect::new(point[0], point[1], 10.0, 20.0),
			);
		}

		graphics::present(ctx);
	Ok(())
	}

	fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
		match keycode {
			Keycode::Up => {
				self.input.up = -1.0;
			}
			Keycode::Down => {
				self.input.down = 1.0;
			}
			Keycode::Right => {
				self.input.right = 1.0;
			}
			Keycode::Left => {
				self.input.left = -1.0;
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
				self.input.up = 0.0;
			}
			Keycode::Down => {
				self.input.down = 0.0;
			}
			Keycode::Right => {
				self.input.right = 0.0;
			}
			Keycode::Left => {
				self.input.left = 0.0;
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
		.window_mode(conf::WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT));

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
