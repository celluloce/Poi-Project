#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate ggez;

use ggez::graphics;
use ggez::event::{self, Keycode, Mod};
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf;

use serde_json::Value;

use std::fs::File;
use std::{path, env};
use std::io::Read;

pub mod shot_type;

const GAME_COUNT: u32 = 3400;

pub const SCREEN_WIDTH: u32 = 1280;
pub const SCREEN_HEIGHT: u32 = 960;

pub const STAGE_UP: u32 = 30;
pub const STAGE_DOWN: u32 = 930;
pub const STAGE_LEFT: u32 = 60;
pub const STAGE_RIGHT: u32 = 830;
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
	Boss,
	PlShot,
	EnShot,
}

#[derive(Debug, PartialEq)]
enum WindowState {
	Title,
	Gaming,
	GamingBoss,
	GameOver,
}

#[derive(Debug)]
pub struct Actor {
	actor_type: ActorType,
	point: [f32; 2],
	// 位置 [x, y]
	velocity: [f32; 2],
	// 1秒の移動距離 [x, y]
	// Shot: [Angle(0.0 <= x < 2.0, 真下が0, 右回り), scalar]
	accel: [f32; 2],
	// 加速度
	// Shot: Anglの値とスカラの値がそれぞれ加速（ベクトル？知らん）
	bbox_size: f32,
	// 当たり判定の半径
	life: f32,
	// Shot: 1.0と0.0でboolのように使う
	moving: Vec<MovingElement>,
	// 動作の記録
	// Playerは多分使わない
	memo: String,
	// メモ用
	// Enemy: 放つ弾幕の種類を書く
}

impl Actor {
	fn player_new() -> Actor {
		Actor {
			actor_type: ActorType::Player,
			point: [300.0, 500.0],
			velocity: [0.0; 2],
			accel: [0.0, 0.0],
			bbox_size: 5.0,
			life: 3.0,
			moving: Vec::new(),
			memo: String::new(),
		}
	}
	fn player_shot_new(p_point: [f32; 2]) -> Actor {
		Actor {
			actor_type: ActorType::PlShot,
			point: p_point,
			velocity: [1.0, 3000.0],
			accel: [0.0, 0.0],
			bbox_size: 10.0,
			life: 1.0,
			moving: Vec::new(),
			memo: String::new(),
		}
	}
	fn enemy_s_new(point: [f32; 2],vel: [f32; 2], life: f32, moving: Vec<MovingElement>) -> Actor {
		Actor {
			actor_type: ActorType::Enemy,
			point: point,
			velocity: vel,
			accel: [0.0; 2],
			bbox_size: 20.0,
			life: life,
			moving: moving,
			memo: String::new(),
		}
	}
	fn boss_new(point: [f32; 2],vel: [f32; 2], life: f32, memo: &str) -> Actor {
		Actor {
			actor_type: ActorType::Boss,
			point: point,
			velocity: vel,
			accel: [0.0; 2],
			bbox_size: 20.0,
			life: life,
			moving: Vec::new(),
			memo: memo.to_string(),
		}
	}
	fn enemy_shot_new(point: [f32; 2], velocity: [f32; 2]) -> Actor {
		Actor {
			actor_type: ActorType::EnShot,
			point: point,
			accel: [0.0; 2],
			velocity: velocity,
			bbox_size: 10.0,
			life: 1.0,
			moving: Vec::new(),
			memo: String::new(),
		}
	}
	fn update_point(actor: &mut Actor, dt: f32) {
		let mut x_vel = actor.velocity[0];
		let mut y_vel = actor.velocity[1];
		let x_acc = actor.accel[0];
		let y_acc = actor.accel[1];

		let mut window_end = 0.0;
		let mut enemy_dead = false;

		if actor.actor_type == ActorType::Enemy {
			window_end = 30.0;
		}

		let s_up = STAGE_UP as f32 - window_end;
		let s_down = STAGE_DOWN as f32 + window_end;
		let s_left = STAGE_LEFT as f32 - window_end;
		let s_right = STAGE_RIGHT as f32 + window_end;


		if (actor.point[0] < s_left && x_vel < 0.0) || (actor.point[0] > s_right && x_vel > 0.0) {
			x_vel = 0.0;
			enemy_dead = true;
		}
		if (actor.point[1] < s_up && y_vel < 0.0) || (actor.point[1] > s_down && y_vel > 0.0) {
			y_vel = 0.0;
			enemy_dead = true;
		}

		if enemy_dead && actor.actor_type == ActorType::Enemy {
			actor.life = 0.0;
		}

		actor.velocity[0] += x_acc * dt;
		actor.velocity[1] += y_acc * dt;
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

		if (actor.point[0] < s_left && x_vel < 0.0) || (actor.point[0] > s_right && x_vel > 0.0) {
			x_vel = 0.0;
			actor.life = 0.0;
		}
		if (actor.point[1] < s_up && y_vel < 0.0) || (actor.point[1] > s_down && y_vel > 0.0) {
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
struct Assets {
	frame_img: graphics::Image,
}

impl Assets {
	fn new(ctx: &mut Context) -> GameResult<Assets> {
		let frame_img = graphics::Image::new(ctx, "/Frame.png").unwrap();
		Ok(Assets {
			frame_img,
		})
	}
}

// Jsonから取り込むためだけの構造体
#[derive(Deserialize, Debug, Clone, PartialEq)]
struct StageFromJson {
	count: Vec<u32>,
	// イベントを起こすカウント数
	char_type: String,
	// キャラクタの種類
	number_class: [u32; 2],
	// [繰り返す回数, 間隔カウント]
	point: [f32; 2],
	// 初期位置 [x, y]
	shift_point: [f32; 2],
	// 出現位置のずれ [x, y]
	velocity: [f32; 2],
	// 初期速度
	// Shot: [角度, スカラ値]
	life: f32,
	// 初期life
	moving: Vec<MovingElement>,
	// 移動データ

}

// 敵の出現, 行動のデータの構造体
#[derive(Deserialize, Debug, Clone, PartialEq)]
struct Stage {
	count: u32,
	// イベントを起こすカウント数
	char_type: String,
	// キャラクタの種類
	number_class: [u32; 2],
	// [繰り返す回数, 間隔カウント]
	point: [f32; 2],
	// 初期位置 [x, y]
	shift_point: [f32; 2],
	// 出現位置のずれ [x, y]
	velocity: [f32; 2],
	// 初期速度
	// Shot: [角度, スカラ値]
	life: f32,
	// 初期life
	moving: Vec<MovingElement>,
	// 移動データ

}

// Jsonから取り込んだ移動データ
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MovingElement {
	count: u32,
	// イベントを起こすカウント数
	// Stage.countからの相対値ではなく絶対値
	accel: [f32; 2],
	// 加速度 [x, y]
	// Shot: [角度の値, スカラ値] （ベクトル演算ではない）
	shot_type: String,
	// 放つShotの種類
}

#[derive(Debug)]
pub struct MainState {
	window_state: WindowState,
	player: Actor,
	plshots: Vec<Actor>,
	enemys: Vec<Actor>,
	enshots: Vec<Actor>,
	stage: Vec<Stage>,
	input: InputState,
	game_count: [u32; 2],
	// [道中, Boss]
	assets: Assets,
	score: u32,
}

impl MainState {
	pub fn new(ctx: &mut Context) -> GameResult<MainState> {

		// JsonFileからDateを取得, 構造体型に変換
		let mut f = File::open("resources/story.json").expect("open json file");
		let mut s = String::new();
		f.read_to_string(&mut s).expect("read json to string");

		let v: Value = serde_json::from_str(&s).expect("serde json from str");
		let sv: &Value = &v["stage1"];
		let mut stage_from_json: Vec<StageFromJson>
			= serde_json::from_value(sv.to_owned()).expect("serde json from value");
		// ---------------------

		// StageFromJsonをStageに変換
		//let mut buf_count: (u32, usize) = (stage_from_json[0].count, 0);
		let mut stage1: Vec<Stage> = Vec::new();
		for sfj in stage_from_json {
			for sfj_c in sfj.count {
				stage1.push(Stage {
					count: sfj_c,
					char_type: sfj.char_type.clone(),
					number_class: sfj.number_class,
					point: sfj.point,
					shift_point: sfj.shift_point,
					velocity: sfj.velocity,
					life: sfj.life,
					moving: sfj.moving.clone(),
				})
			}
		}
		println!("Inputed stage: {:?}", stage1);
		// ---------------------

		// moving countにstage countを加算
		// Jsonが書きやすくなる
			for stage in &mut stage1 {
				for moving in &mut stage.moving {
					moving.count += stage.count;
				}
			}
		// ---------------------

		let s = MainState{
			window_state: WindowState::Title,
			player: Actor::player_new(),
			plshots: Vec::with_capacity(50),
			enemys: Vec::with_capacity(30),
			enshots: Vec::with_capacity(100),
			stage: stage1,
			input: InputState::new(),
			game_count: [GAME_COUNT, 0],
			assets: Assets::new(ctx).unwrap(),
			score: 0,
		};

		Ok(s)
	}
}

impl ggez::event::EventHandler for MainState {
	fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
		const FPS: u32 = 60;
		let seconds = 1.0 / FPS as f32;

		//println!("");
		//println!("{:?}", timer::duration_to_f64(timer::get_delta(ctx)));
		//println!("{:?}", timer::duration_to_f64(timer::get_average_delta(ctx)));
		//println!("{:?}", timer::duration_to_f64(timer::get_remaining_update_time(ctx)));
		//println!("{:?}", timer::get_fps(ctx));
		// 開始からの経過時間を計測----------
		//let since_start = timer::get_time_since_start(ctx);
		// println!("{:?}", since_start);
		// -------------------------


		while timer::check_update_time(ctx, FPS) {
			let mut game_count_use;

			// WindowStateの分岐----------
			match self.window_state {
				WindowState::Title => {
					if self.input.shot {
						self.window_state = WindowState::Gaming;
					}
					continue
				},
				WindowState::Gaming => {
					self.game_count[0] += 1;
					game_count_use = self.game_count[0];
				}
				WindowState::GamingBoss => {
					// Update boss counter----------
					self.game_count[1] += 1;
					game_count_use = self.game_count[1];
					// -------------------------
				}
				WindowState::GameOver => {
					continue
				}
			}
			// --------------------

			// Update player point----------
			// キーインプット基底ベクトルをInputState値として定める
			// -> InputState値*スカラ値=ActorVelocity
			// -> ActorVelocity*1Frameあたりかかる秒=1Frameあたり進む距離
			// PlayerLifeがゼロの時、WindowStateがGameoverになる
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

			if self.player.life <= 0.0 {
				self.window_state = WindowState::GameOver;
			}

			// -------------------------

			// Update shot state----------
			if self.input.shot && game_count_use % 3 == 0 {
				let mut pp = self.player.point;
				pp[0] += 20.0;
				self.plshots.push(Actor::player_shot_new(pp));
				pp[0] -= 40.0;
				self.plshots.push(Actor::player_shot_new(pp));
			}
			for s in &mut self.plshots {
				Actor::update_point_shot(s, seconds);
			}
			// -------------------------

			// Jsonから取得したデータから、Enemyを生成
			// char_type = "boss"が見つかった場合、its_boss_timeがtrueになる
			let mut its_boss_time = false;
			for i in 0..self.stage.len() {
				let en_date = self.stage[i].clone();
				if en_date.count == self.game_count[0] {
					match en_date.char_type.as_str() {
						"boss" => {
							its_boss_time = true;
							self.game_count[0] += 1;
							continue;
						}
						_ => (),
					}
					let p = [en_date.point[0], en_date.point[1]];
					let v = [en_date.velocity[0], en_date.velocity[1]];
					let l = en_date.life;
					let m = en_date.moving.clone();
					self.enemys.push(Actor::enemy_s_new(p, v, l, m));

					if en_date.number_class[0] > 0 {
						let add_count = en_date.number_class[1];
						self.stage[i].count += add_count;
						self.stage[i].number_class[0] -= 1;
						self.stage[i].point[0] += en_date.shift_point[0];
						self.stage[i].point[1] += en_date.shift_point[1];
						for j in 0..en_date.moving.len() {
							self.stage[i].moving[j].count += add_count;
						}
					}
				}
			}
			self.stage.retain(|c| c.number_class[0] >= 0);
			// --------------------

			// Enemyの更新
			// - Jsonから取得したデータから、Enemyの動作を書き換え
			// - 弾幕を張る
			// - 位置の更新
			for e in &mut self.enemys {
				for m in 0..e.moving.len() {
					if e.moving[m].count == self.game_count[0] {
						e.accel = e.moving[m].accel;
						e.memo = e.moving[m].shot_type.clone();
					}
				}

				match e.memo.as_str() {
					"six" => shot_type::six(e, self.player.point, &mut self.enshots, game_count_use),
					"b_six_rotate" => shot_type::b_six_rotate(e, self.player.point, &mut self.enshots, game_count_use),
					_ => (),
				}

				Actor::update_point(e, seconds);
			}
			//-------------------------

			let in_bbox = |ac1: &mut Actor, ac2: &mut Actor| {
				// rr > xx + yy
				let x = ac1.point[0] - ac2.point[0];
				let y = ac1.point[1] - ac2.point[1];
				let r = ac1.bbox_size + ac2.bbox_size;

				let xx = x * x;
				let yy = y * y;
				let rr = r * r;
				rr > xx + yy
			};

			// Hit EnemyShots & Player----------
				for es in &mut self.enshots {
					Actor::update_point_shot(es, seconds);

					let pl = &mut self.player;
					if in_bbox(pl, es) {
						es.life = 0.0;
						pl.life -= 1.0;
					}
				}
			// -------------------------

			// Hit PlayerShots & Enemy----------
				for en in &mut self.enemys {
					for ps in &mut self.plshots {
						if in_bbox(en, ps) {
							en.life -= ps.life;
							ps.life = 0.0;
							if en.life <= 0.0 {
								self.score += 30;
							}
						}
					}
				}
			// -------------------------

			// Clear zero_life Enemy, Shot----------
			self.plshots.retain(|s| s.life > 0.0);
			self.enemys.retain(|s| s.life > 0.0);
			self.enshots.retain(|s| s.life > 0.0);
			// -------------------------

			if its_boss_time {
				self.enemys = Vec::new();
				self.enshots = Vec::new();
				self.window_state = WindowState::GamingBoss;
				self.enemys.push(Actor::boss_new(
							[350.0, 200.0],
							[0.0, 0.0],
							500.0,
							"b_six_rotate"
						));
			}
		}
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
		graphics::clear(ctx);

		let point = self.player.point;
		let font = graphics::Font::new(ctx, "/SoberbaSerif-Regular.ttf", 18).unwrap();

		// match Window State
		match self.window_state {
			WindowState::Title => {
				// Print "poi-Project"
				let font = graphics::Font::new(ctx, "/SoberbaSerif-Regular.ttf", 100).unwrap();
				let display = graphics::Text::new(ctx, "poi-Project", &font).unwrap();
				let display_point = graphics::Point2::new(300.0, 300.0);
				graphics::draw(ctx, &display, display_point, 0.0).unwrap();

				// Write "-"
				graphics::rectangle(
					ctx,
					graphics::DrawMode::Fill,
					graphics::Rect::new(495.0, 380.0, 40.0, 10.0),
				);

				// Print "press Z key"
				let font = graphics::Font::new(ctx, "/SoberbaSerif-Regular.ttf", 30).unwrap();
				let display = graphics::Text::new(ctx, "Please press Z key", &font).unwrap();
				let display_point = graphics::Point2::new(400.0, 600.0);
				graphics::draw(ctx, &display, display_point, 0.0).unwrap();

				// Skip other code
				graphics::present(ctx);
				return Ok(());
			},
			WindowState::Gaming => (),
			WindowState::GamingBoss => {
				// Print Boss life
				if self.enemys.len() == 1 {
					let display_str = format!("Boss: {}", self.enemys[0].life);
					let display = graphics::Text::new(ctx, &display_str, &font).unwrap();
					let display_point = graphics::Point2::new(500.0, 200.0);
					graphics::draw(ctx, &display, display_point, 0.0).unwrap();
				}
			}
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
		for act in &mut self.plshots {
			let point = act.point;
			graphics::rectangle(
				ctx,
				graphics::DrawMode::Fill,
				graphics::Rect::new(point[0] - 8.0, point[1] - 15.0, 16.0, 30.0),
			);
		}

		// drow enemy circle
		for act in &mut self.enemys {
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
		let drawable = &self.assets.frame_img;
		let params = graphics::DrawParam {
			..Default::default()
		};
		graphics::draw_ex(ctx, drawable, params);

		// Print score
		let display_str = format!("Score: {}", self.score);
		let display = graphics::Text::new(ctx, &display_str, &font).unwrap();
		let display_point = graphics::Point2::new(900.0, 100.0);
		graphics::draw(ctx, &display, display_point, 0.0).unwrap();

		// Print Player life
		let display_str = format!("Life: {}", self.player.life as usize);
		let display = graphics::Text::new(ctx, &display_str, &font).unwrap();
		let display_point = graphics::Point2::new(900.0, 150.0);
		graphics::draw(ctx, &display, display_point, 0.0).unwrap();

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

