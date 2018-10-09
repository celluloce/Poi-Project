extern crate ggez;

use ggez::graphics;
use ggez::event::{self, Keycode, Mod};
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf;

use std::{path, env};

const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 960;

const STAGE_UP: u32 = 30;
const STAGE_DOWN: u32 = 930;
const STAGE_LEFT: u32 = 60;
const STAGE_RIGHT: u32 = 830;
// +---+-------+-----+ 0px
// |   |       |     |
// +---+-------+-----+ 30px
// |   |       |     |
// |   |       |     |
// +---+-------+-----+ 930px
// +---+-------+-----+ 960px
// 0px 60px   830px 1280px


#[derive(Debug, PartialEq)]
enum ActorType {
	Player,
	Enemy,
	PlShot,
	EnShot,
}

#[derive(Debug, PartialEq)]
enum WindowState {
	Title,
	Gaming,
	GameOver,
}

#[derive(Debug)]
struct Actor {
	actor_type: ActorType,
	point: [f32; 2],
	// 位置 [x, y]
	velocity: [f32; 2],
	// 1秒の移動距離 [x, y]
	// Shot: [Angle(0.0 <= x < 2.0, 真下が0, 右回り), scalar]
	bbox_size: f32,
	// 当たり判定の半径
	life: f32,
	// Shot: 1.0と0.0でboolのように使う
}

impl Actor {
	fn player_new() -> Actor {
		Actor {
			actor_type: ActorType::Player,
			point: [300.0, 500.0],
			velocity: [0.0; 2],
			bbox_size: 5.0,
			life: 3.0,
		}
	}
	fn player_shot_new(p_point: [f32; 2]) -> Actor {
		Actor {
			actor_type: ActorType::PlShot,
			point: p_point,
			velocity: [1.0, 3000.0],
			bbox_size: 8.0,
			life: 1.0,
		}
	}
	fn enemy_new(point: [f32; 2], velocity: [f32; 2], life: f32) -> Actor {
		Actor {
			actor_type: ActorType::Enemy,
			point: point,
			velocity: velocity,
			bbox_size: 20.0,
			life: life,
		}
	}
	fn enemy_shot_new(point: [f32; 2], velocity: [f32; 2]) -> Actor {
		Actor {
			actor_type: ActorType::EnShot,
			point: point,
			velocity: velocity,
			bbox_size: 10.0,
			life: 1.0,
		}
	}
	fn update_point(actor: &mut Actor, dt: f32) {
		let mut x_vel = actor.velocity[0];
		let mut y_vel = actor.velocity[1];

		let mut window_end = 0.0;
		let mut enemy_dead = false;

		if actor.actor_type == ActorType::Enemy {
			window_end = 30.0;
		}

		let s_up = STAGE_UP as f32 - window_end;
		let s_down = STAGE_DOWN as f32 + window_end;
		let s_left = STAGE_LEFT as f32 - window_end;
		let s_right = STAGE_RIGHT as f32 + window_end;

		if actor.point[0] < s_left && x_vel < 0.0 {
			x_vel = 0.0;
			enemy_dead = true;
		}
		if actor.point[0] > s_right && x_vel > 0.0 {
			x_vel = 0.0;
			enemy_dead = true;
		}
		if actor.point[1] < s_up && y_vel < 0.0 {
			y_vel = 0.0;
			enemy_dead = true;
		}
		if actor.point[1] > s_down && y_vel > 0.0 {
			y_vel = 0.0;
			enemy_dead = true;
		}

		if enemy_dead && actor.actor_type == ActorType::Enemy {
			actor.life = 0.0;
		}

		actor.point[0] += x_vel * dt;
		actor.point[1] += y_vel * dt;
	}

	fn update_point_shot(actor: &mut Actor, dt: f32) {
		use std::f32::consts::PI;

		let s_up = STAGE_UP as f32 - 30.0;
		let s_down = STAGE_DOWN as f32 + 30.0;
		let s_left = STAGE_LEFT as f32 - 30.0;
		let s_right = STAGE_RIGHT as f32 + 30.0;

		let scalar = actor.velocity[1];
		let ragian = actor.velocity[0] * PI;
		let mut x_vel = scalar * ragian.sin();
		let mut y_vel = scalar * ragian.cos();

		if actor.point[0] < s_left && x_vel < 0.0 {
			x_vel = 0.0;
			actor.life = 0.0;
		}
		if actor.point[0] > s_right && x_vel > 0.0 {
			x_vel = 0.0;
			actor.life = 0.0;
		}
		if actor.point[1] < s_up && y_vel < 0.0 {
			y_vel = 0.0;
			actor.life = 0.0;
		}
		if actor.point[1] > s_down && y_vel > 0.0 {
			y_vel = 0.0;
			actor.life = 0.0;
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

impl InputState {
	fn new() -> InputState {
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
	window_state: WindowState,
	player: Actor,
	shots: Vec<Actor>,
	enemy: Vec<Actor>,
	enshots: Vec<Actor>,
	input: InputState,
	game_count: u32,
	score: u32,
}

impl MainState {
	fn new(ctx: &mut Context) -> GameResult<MainState> {
		let s = MainState{
			window_state: WindowState::Title,
			player: Actor::player_new(),
			shots: Vec::with_capacity(50),
			enemy: Vec::with_capacity(30),
			enshots: Vec::with_capacity(100),
			input: InputState::new(),
			game_count: 0,
			score: 0,
		};

		Ok(s)
	}
	fn game_count_new(&mut self) {
		self.game_count = 0;
	}

}

impl ggez::event::EventHandler for MainState {
	fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
		const FPS: u32 = 60;
		let seconds = 1.0 / FPS as f32;

		while timer::check_update_time(ctx, FPS) {

			// PlayerLifeがゼロの時、WindowStateがGameoverになる
			if self.player.life <= 0.0 {
				self.window_state = WindowState::GameOver;
			}

			// WindowStateの分岐----------
			match self.window_state {
				WindowState::Title => {
					if self.input.shot {
						self.window_state = WindowState::Gaming;
					}
					continue
				},
				WindowState::Gaming => (),
				WindowState::GameOver => {
					continue
				}
			}
			// --------------------

			// 開始からの経過時間を計測----------
			let since_start = timer::get_time_since_start(ctx);
			// println!("{:?}", since_start);
			// -------------------------

			// Update player point----------
			// キーインプット基底ベクトルをInputState値として定める
			// -> InputState値*スカラ値=ActorVelocity
			// -> ActorVelocity*1Frameあたりかかる秒=1Frameあたり進む距離
			let s_input = self.input;
			if !s_input.shift {
				// 高速移動
				self.player.velocity[0] = (s_input.right + s_input.left) * 200.0;
				self.player.velocity[1] = (s_input.up + s_input.down) * 200.0;
			} else {
				// 低速Shift移動
				self.player.velocity[0] = (s_input.right + s_input.left) * 100.0;
				self.player.velocity[1] = (s_input.up + s_input.down) * 100.0;
			}
			Actor::update_point(&mut self.player, seconds);
			// -------------------------

			// Update shot state----------
			if self.input.shot && self.game_count % 3 == 0 {
				// println!("shot: {}", self.game_count);
				// InputStateのshotがtrueの時、shotをVectorに入れる
				let mut pp = self.player.point;
				pp[0] += 20.0;
				self.shots.push(Actor::player_shot_new(pp));
				pp[0] -= 40.0;
				self.shots.push(Actor::player_shot_new(pp));
			}
			for act in &mut self.shots {
				Actor::update_point_shot(act, seconds);
			}
			// -------------------------

			// debug shot----------
			// for act in &self.shots {
			// 	print!("{}", act.life);
			// }
			// println!("");
			// println!("shot len: {}", self.shots.len());
			// println!("");
			// -------------------------

			// Update Enemy State----------
			if self.game_count % 100 == 0 && self.game_count < 300{
				self.enemy.push(Actor::enemy_new([1000.0, 100.0], [-100.0, 30.0], 30.0))
			}
			for act in &mut self.enemy {
				Actor::update_point(act, seconds)
			}
			// -------------------------

			// Update Enemy Shot ----------
			if self.game_count % 50 == 0 {
				for enemy in &self.enemy {
					for c in 0..5 {
						let en_point = enemy.point;
						let shot_vel = {
							let shot_scal = 100.0;
							let angle = (c as f32) / 3.0;
							[angle, shot_scal]
						};
						self.enshots.push(Actor::enemy_shot_new(en_point, shot_vel));
					}
				}
			}
			for act in &mut self.enshots {
				Actor::update_point_shot(act, seconds)
			}
			// -------------------------

			// Hit EnemyShots & Player----------
				for enshot in &mut self.enshots {
						// rr > xx + yy
						let player = &mut self.player;
						let x = player.point[0] - enshot.point[0];
						let y = player.point[1] - enshot.point[1];
						let r = player.bbox_size + enshot.bbox_size;

						let xx = x * x;
						let yy = y * y;
						let rr = r * r;
						if rr > xx + yy {
							enshot.life = 0.0;
							player.life -= 1.0;
							println!("{}", player.life);
						}
				}
			// -------------------------

			// Hit PlayerShots & Enemy----------
				for enemy in &mut self.enemy {
					for shot in &mut self.shots {
						// rr > xx + yy
						let x = shot.point[0] - enemy.point[0];
						let y = shot.point[1] - enemy.point[1];
						let r = shot.bbox_size + enemy.bbox_size;

						let xx = x * x;
						let yy = y * y;
						let rr = r * r;
						if rr > xx + yy {
							enemy.life -= shot.life;
							shot.life = 0.0;
							if enemy.life <= 0.0 {
								self.score += 30;
							}
						}
					}
				}
			// -------------------------

			// Clear zero_life Enemy, Shot----------
			self.shots.retain(|s| s.life > 0.0);
			self.enemy.retain(|s| s.life > 0.0);
			self.enshots.retain(|s| s.life > 0.0);
			// -------------------------

			// Update game counter----------
			self.game_count += 1;
			// println!("{}", self.game_count);
			// -------------------------

		}
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
		graphics::clear(ctx);

		let point = self.player.point;

		// match Window State
		match self.window_state {
			WindowState::Title => println!("Title, Press Z key"),
			WindowState::Gaming => (),
			WindowState::GameOver => {
				let font = graphics::Font::new(ctx, "/SoberbaSerif-Regular.ttf", 30).unwrap();
				let gameover_display = graphics::Text::new(ctx, "GameOver", &font).unwrap();
				let display_point = graphics::Point2::new(400.0, 300.0);
				graphics::draw(ctx, &gameover_display, display_point, 0.0).unwrap();
			}
		}

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
				graphics::Rect::new(point[0] - 8.0, point[1] - 15.0, 16.0, 30.0),
			);
		}

		// drow enemy circle
		for act in &mut self.enemy {
			let point = act.point;
			graphics::circle(
				ctx,
				graphics::DrawMode::Fill,
				graphics::Point2::new(point[0],point[1]),
				20.0,
				0.1,
			);
		}

		// drow enshots circle
		for act in &mut self.enshots {
			let point = act.point;
			graphics::circle(
				ctx,
				graphics::DrawMode::Fill,
				graphics::Point2::new(point[0],point[1]),
				10.0,
				0.1,
			);
		}
		// draw frame
		let drawable = graphics::Image::new(ctx, "/Frame.png").unwrap();
		let params = graphics::DrawParam {
			..Default::default()
		};
		graphics::draw_ex(ctx, &drawable, params);

		// Print score
		let score_str = format!("Score: {}", self.score);
		let font = graphics::Font::new(ctx, "/SoberbaSerif-Regular.ttf", 18).unwrap();
		let score_display = graphics::Text::new(ctx, &score_str, &font).unwrap();
		let score_point = graphics::Point2::new(900.0, 100.0);
		graphics::draw(ctx, &score_display, score_point, 0.0).unwrap();


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
