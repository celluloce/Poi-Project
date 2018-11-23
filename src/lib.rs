#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate ggez;
extern crate rand;

use ggez::graphics;
use ggez::event::{Keycode, Mod};
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::error::GameError;

use serde_json::Value;

use std::fs::File;
use std::{path, env};
use std::io::Read;
use rand::ThreadRng;

pub mod shot_type;

pub const SCREEN_WIDTH: f32 = 880.0;
pub const SCREEN_HEIGHT: f32 = 460.0;

//const GAME_COUNT: u32 = 3500;

const RELATIVE_Y: f32 = SCREEN_HEIGHT / 960.0;
const RELATIVE_X: f32 = SCREEN_WIDTH / 1280.0;
pub const STAGE_UP: f32 = 30.0 * RELATIVE_Y;
pub const STAGE_DOWN: f32 = 930.0 * RELATIVE_Y;
pub const STAGE_LEFT: f32 = 60.0 * RELATIVE_X;
pub const STAGE_RIGHT: f32 = 830.0 * RELATIVE_X;
// +---+-------+-----+ 0px
// |   |       |     |
// +---+-------+-----+ 30px
// |   |       |     |
// |   |       |     |
// +---+-------+-----+ 930px
// +---+-------+-----+ 960px
// 0px 60px   830px 1280px


#[derive(Debug, PartialEq, Clone)]
enum ActorType {
	Player,
	Enemy,
	Boss,
	PlShot,
	EnShot,
	Effect,
}

#[derive(Debug, PartialEq)]
enum WindowState {
	Title,
	Gaming,
	GamingBoss,
	GameOver,
	GameClear,
	ThankYouForPlaying,
}

#[derive(Debug, Clone)]
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
	count: u32,
	// 経過countメモ
	// TODO: 実装タイミングが遅かったので書き換え必須
	memo: String,
	// メモ用
	// Enemy: 放つ弾幕の種類を書く
}

impl Actor {
	fn player_new() -> Actor {
		Actor {
			actor_type: ActorType::Player,
			point: [440.0, 800.0],
			velocity: [0.0; 2],
			accel: [0.0, 0.0],
			bbox_size: 5.0,
			life: 3.0,
			moving: Vec::new(),
			count: 0,
			memo: String::new(),
		}
	}
	fn trans_pleyer_new(life: f32) -> Actor {
		Actor {
			actor_type: ActorType::Player,
			point: [440.0, 800.0],
			velocity: [0.0; 2],
			accel: [0.0, 0.0],
			bbox_size: 0.0,
			life: life,
			moving: Vec::new(),
			count: 0,
			memo: String::from("trans"),
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
			count: 0,
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
			count: 0,
			memo: String::new(),
		}
	}
	fn boss_new(point: [f32; 2],vel: [f32; 2], life: f32, moving: Vec<MovingElement>, memo: &str) -> Actor {
		Actor {
			actor_type: ActorType::Boss,
			point: point,
			velocity: vel,
			accel: [0.0; 2],
			bbox_size: 30.0,
			life: life,
			moving: moving,
			count: 0,
			memo: memo.to_owned()
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
			count: 0,
			memo: String::new(),
		}
	}
	fn effect_new(point: [f32; 2], velocity: [f32; 2], moving: Vec<MovingElement>, memo: &str) -> Actor {
		Actor {
			actor_type: ActorType::Effect,
			point: point,
			accel: [0.0; 2],
			velocity: velocity,
			bbox_size: 10.0,
			life: 1.0,
			moving: moving,
			count: 0,
			memo: memo.to_owned(),
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

		let mergin_x = window_end * RELATIVE_X;
		let mergin_y = window_end * RELATIVE_Y;


		let s_up = STAGE_UP  - mergin_y;
		let s_down = STAGE_DOWN  + mergin_y;
		let s_left = STAGE_LEFT  - mergin_x;
		let s_right = STAGE_RIGHT  + mergin_x;


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

		let mergin_x = 30.0 * RELATIVE_X ;
		let mergin_y = 30.0 * RELATIVE_Y ;
		let s_up = STAGE_UP  - mergin_y;
		let s_down = STAGE_DOWN  + mergin_y;
		let s_left = STAGE_LEFT  - mergin_x;
		let s_right = STAGE_RIGHT  + mergin_x;

		let scalar = actor.accel[1];
		let ragian = actor.accel[0] * PI;
		let mut x_acc = scalar * ragian.sin();
		let mut y_acc = scalar * ragian.cos();

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

		actor.velocity[0] += actor.accel[0] * dt;
		actor.velocity[1] += actor.accel[1] * dt;
		actor.point[0] += x_vel * RELATIVE_X * dt;
		actor.point[1] += y_vel * RELATIVE_Y * dt;

	}

	fn to_relative_window(mut self) -> Actor {
		self.point[0] *= RELATIVE_X;
		self.point[1] *= RELATIVE_Y;
		match self.actor_type {
			ActorType::PlShot | ActorType::EnShot => (),
			_ => {
				self.velocity[0] *= RELATIVE_X ;
				self.velocity[1] *= RELATIVE_Y ;
				self.accel[0] *= RELATIVE_X ;
				self.accel[1] *= RELATIVE_Y ;
			}
		}
		Actor {
			actor_type: self.actor_type,
			point: self.point,
			accel: self.accel,
			velocity: self.velocity,
			bbox_size: self.bbox_size,
			life: self.life,
			moving: self.moving,
			count: self.count,
			memo: self.memo,
		}
	}
}

impl Default for Actor {
	fn default() -> Actor {
		Actor {
			actor_type: ActorType::EnShot,
			point: [0.0; 2],
			velocity: [0.0; 2],
			accel: [0.0; 2],
			bbox_size: 10.0,
			life: 1.0,
			moving: Vec::new(),
			count: 0,
			memo: String::new(),
		}
	}
}

#[derive(Debug, Clone, Copy)]
struct InputState {
	up: bool,
	down: bool,
	right: bool,
	left: bool,
	shift: bool,
	shot: bool,
	bomb: bool,
}

impl InputState {
	fn new() -> InputState {
		InputState {
			up: false,
			down: false,
			right: false,
			left: false,
			shift: false,
			shot: false,
			bomb: false,
		}
	}
}

#[derive(Debug)]
struct Assets {
	frame_img: graphics::Image,
	brack_out_img: graphics::Image,
	player_front_img: graphics::Image,
	player_right_img: graphics::Image,
	player_left_img: graphics::Image,
	effect_img: graphics::Image,
	ending_img: graphics::Image,
}

impl Assets {
	fn new(ctx: &mut Context) -> GameResult<Assets> {
		let frame_img = graphics::Image::new(ctx, "/Frame.png").unwrap();
		let brack_out_img = graphics::Image::new(ctx, "/brack_out.png").unwrap();
		let player_front_img = graphics::Image::new(ctx,"/player_front.png").unwrap();
		let player_left_img = graphics::Image::new(ctx,"/player_left.png").unwrap();
		let player_right_img = graphics::Image::new(ctx,"/player_right.png").unwrap();
		let effect_img = graphics::Image::new(ctx, "/effect.png").unwrap();
		let ending_img = graphics::Image::new(ctx, "/thanks_sign.png").unwrap();
		Ok(Assets {
			frame_img,
			brack_out_img,
			player_front_img,
			player_right_img,
			player_left_img,
			effect_img,
			ending_img,
		})
	}
	fn draw_player (
		ctx: &mut Context,
		assets: &mut Assets,
		player: &Actor,
		input: InputState,
		) -> GameResult<()> {
		let mut img: &mut graphics::Image;
		let mut front = false;
		if input.right && input.left || front {
			img = &mut assets.player_front_img;
		} else {
			if input.right {
				img = &mut assets.player_right_img;
			} else if input.left {
				img = &mut assets.player_left_img;
			} else {
				img = &mut assets.player_front_img;
			}
		}
		let point = {
			let x = player.point[0];
			let y = player.point[1];
			graphics::Point2::new(x, y)
		};
		let drawparams = graphics::DrawParam {
			dest: point,
			offset: graphics::Point2::new(0.5, 0.5),
			..Default::default()
		};
		graphics::draw_ex(ctx, img, drawparams)
	}
}

// Jsonから取り込むためだけの構造体
#[derive(Deserialize, Debug, Clone, PartialEq)]
struct StageFromJson {
	_comment: Vec<String>,
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
impl MovingElement {
	fn new (count: u32, accel: [f32; 2], shot_type: &str) -> MovingElement{
		MovingElement {
			count: count,
			accel: accel,
			shot_type: shot_type.to_string(),
		}
	}
}

#[derive(Debug)]
pub struct MainState {
	window_state: WindowState,
	player: Actor,
	plshots: Vec<Actor>,
	enemys: Vec<Actor>,
	boss: Vec<Actor>,
	enshots: Vec<Actor>,
	effects: Vec<Actor>,
	stage: Vec<Stage>,
	input: InputState,
	input_break: InputState,
	game_count: [u32; 2],
	// [道中, Boss]
	//rand_v: Vec<f32>,
	rand: ThreadRng,
	assets: Assets,
	bomb: u32,
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

		let initial_count = v["initial_count"].as_u64().unwrap() as u32;

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
		//println!("Inputed stage: {:?}", stage1);
		// ---------------------

		// moving countにstage countを加算
		// Jsonが書きやすくなる
		for stage in &mut stage1 {
			for moving in &mut stage.moving {
				moving.count += stage.count;
			}
		}
		// ---------------------

		// 乱数の配列を作成
		let rng = rand::thread_rng();
		//let mut rand_v = vec![0.0; 100];
		//for i in rand_v.iter_mut() {
		//	*i = rng.gen();
		//}
		//println!("rand_v: {:?}", rand_v);
		// --------------------

		let s = MainState{
			window_state: WindowState::Title,
			player: Actor::player_new().to_relative_window(),
			plshots: Vec::with_capacity(50),
			enemys: Vec::with_capacity(30),
			boss: Vec::with_capacity(1),
			enshots: Vec::with_capacity(100),
			effects: Vec::with_capacity(30),
			stage: stage1,
			input: InputState::new(),
			input_break: InputState::new(),
			//rand_v: rand_v,
			rand: rng,
			game_count: [initial_count, 0],
			assets: Assets::new(ctx).unwrap(),
			bomb: 4,
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
			//println!("{:?}", self.enemys);
			let mut game_count_use = 0;

			// WindowStateの分岐----------
			match self.window_state {
				WindowState::Title => {
					// InputStateのbreak----------
					if self.input_break.shot {
						self.input.shot = false;
					}
					if self.input.shot || self.input_break.shot {
						self.input_break.shot = true;
					}
					// ---------------------

					if self.input.shot {
						self.window_state = WindowState::Gaming;
					}
					continue
				},
				WindowState::Gaming => {
					self.game_count[0] += 1;
					game_count_use = self.game_count[0];

					// Jsonから取得したデータから、Enemyを生成
					// stage element number
					'stage: for st in &mut self.stage {
						if st.count == self.game_count[0] {
							match st.char_type.as_str() {
								"clear" => {
									self.enemys = Vec::new();
									self.window_state = WindowState::GameClear;
									self.game_count[0] = 0;
									break 'stage;
								}
								b_type @  "boss" | b_type @ "m_boss" => {
									// char_type = "boss"が見つかった場合、WindowStateがGamingBossになる
									// Window切り替え
									self.window_state = WindowState::GamingBoss;
									self.game_count[0] += 1;
									// -------------------------

									// GamingBoss 初期化
									self.enemys = Vec::new();
									self.enshots = Vec::new();
									let mut boss_moving: Vec<MovingElement>;
									let ac = [0.0; 2];
									match b_type {
										"m_boss" => {
											boss_moving = vec![
												MovingElement::new(30,[0.0; 2], "waiting"),
												MovingElement::new(1500, [0.0; 2], "m_six_rotate"),
												MovingElement::new(1500, [0.0; 2], "m_six_fireflower"),
											];
										},
										"boss" => {
											boss_moving = vec![
												MovingElement::new(30, [0.0; 2],"waiting"),
												MovingElement::new(1500, [0.0; 2], "b_normal"),
												MovingElement::new(1500, [0.0; 2], "b_6rotate_4rand"),
												MovingElement::new(1500, [0.0; 2], "b_normal"),
												MovingElement::new(1500, [0.0; 2], "b_2fireflower_4pdis"),
												MovingElement::new(1500, [0.0; 2], "b_normal"),
												MovingElement::new(1500, [0.0; 2], "b_6carpet_fireflower"),
											];
										},
										_ => boss_moving = Vec::new(),
									}
									self.boss.push(Actor::boss_new(
												[450.0, 200.0],
												[0.0, 0.0],
												0.1,
												boss_moving,
												"",
											).to_relative_window());
									// -------------------------
									break 'stage;
								}
								_ => (),
							}
							// --------------------
							let p = [st.point[0], st.point[1]];
							let v = [st.velocity[0], st.velocity[1]];
							let l = st.life;
							let m = st.moving.clone();
							self.enemys.push(Actor::enemy_s_new(p, v, l, m).to_relative_window());

							if st.number_class[0] > 0 {
								let add_count = st.number_class[1];
								st.count += add_count;
								st.number_class[0] -= 1;
								st.point[0] += st.shift_point[0];
								st.point[1] += st.shift_point[1];
								// en date moving number
								for edmn in 0..st.moving.len() {
									st.moving[edmn].count += add_count;
								}
							}
						}
					}
					self.stage.retain(|c| c.number_class[0] >= 0);
					// --------------------

				}
				WindowState::GamingBoss => {
					game_count_use = self.game_count[1];


					// Update boss counter----------
					self.game_count[1] += 1;
					// -------------------------

					// Bossの更新
					// BossShotをmatch分岐で生成
					//println!("{:?}", self.boss);
					let mut bs = &mut self.boss;
					let pp = self.player.point;
					let mut es = &mut self.enshots;
					let gc = game_count_use;
					let rn = &self.rand;
					match bs[0].memo.as_str() {
						"m_six_rotate" => shot_type::m_six_rotate(&mut bs[0], pp, es, gc),
						"m_six_fireflower" => shot_type::m_six_fireflower(&mut bs[0], pp, es, gc),
						"b_normal" => shot_type::b_normal(&mut bs[0], pp, es, gc, rn),
						"b_6rotate_4rand" => shot_type::b_6rotate_4rand(&mut bs[0], pp, es, gc, rn),
						"b_2fireflower_4pdis" => shot_type::b_2fireflower_4pdis(&mut bs[0], pp, es, gc, rn),
						"b_6carpet_fireflower" => shot_type::b_6carpet_fireflower(&mut bs[0], pp, es, gc, rn),
						_ => (),
						//_ => println!("no shot"),
					}

					if bs[0].life < 0.0 || game_count_use == bs[0].moving[0].count {
						if bs[0].moving.len() > 1 {
							self.game_count[1] = 1;
							bs[0].life = 500.0;
							*es = Vec::new();
							bs[0].moving.remove(0);
							bs[0].memo = bs[0].moving[0].shot_type.clone();
						} else {
							//println!("boss end");
							*bs = Vec::new();
							*es = Vec::new();
							self.window_state = WindowState::Gaming;
						}
					}
					// -------------------------
				}
				WindowState::GameOver => {
					// InputStateのbreak----------
					if self.input_break.shot {
						self.input.shot = false;
					}
					if self.input.shot || self.input_break.shot {
						self.input_break.shot = true;
					}
					// ---------------------

					self.game_count[0] += 1;
					if self.game_count[0] >= 180 && self.input.shot {
						return Err(GameError::UnknownError(String::from("You Lose")));
					}
					continue;
				},
				WindowState::GameClear => {
					self.game_count[0] += 1;
					if self.game_count[0] >= 180 {
						self.game_count[0] = 0;
						self.window_state = WindowState::ThankYouForPlaying;
					}
				},
				WindowState::ThankYouForPlaying => {
					// InputStateのbreak----------
					if self.input_break.shot {
						self.input.shot = false;
					}
					if self.input.shot || self.input_break.shot {
						self.input_break.shot = true;
					}
					// ---------------------
					self.game_count[0] += 1;
					if self.game_count[0] >= 240 && self.input.shot {
						return Err(GameError::UnknownError(String::from("You Win")));
					}
				},
			}
			// --------------------

			// Update player point----------
			// キーインプットに応じて、Playerのvelocityを書き換える
			// Playerの位置を更新
			// PlayerLifeがゼロの時、WindowStateがGameoverになる
			if self.input.up {
				self.player.velocity[1] = -1.0;
			} else if self.input.down {
				self.player.velocity[1] = 1.0;
			} else {
				self.player.velocity[1] = 0.0;
			}
			if self.input.right {
				self.player.velocity[0] = 1.0;
			} else if self.input.left {
				self.player.velocity[0] = -1.0;
			} else {
				self.player.velocity[0] = 0.0;
			}

			if self.input.up && self.input.down {
				self.player.velocity[1] = 0.0;
			}
			if self.input.right && self.input.left {
				self.player.velocity[0] = 0.0;
			}

			if !self.input.shift {
				// 高速移動
				self.player.velocity[0] *= 350.0;
				self.player.velocity[1] *= 350.0;
			} else {
				// 低速Shift移動
				self.player.velocity[0] *= 150.0;
				self.player.velocity[1] *= 150.0;
			}

			Actor::update_point(&mut self.player, seconds);

			if self.player.life <= 0.0 {
				self.window_state = WindowState::GameOver;
				self.game_count[0] = 0;
			}

			// -------------------------

			// Update Plshot state----------
			if self.input.shot && game_count_use % 3 == 0 {
				let mut pp = self.player.point;
				pp[0] += 20.0;
				self.plshots.push(Actor::player_shot_new(pp).to_relative_window());
				pp[0] -= 40.0;
				self.plshots.push(Actor::player_shot_new(pp).to_relative_window());
			}
			for s in &mut self.plshots {
				Actor::update_point_shot(s, seconds);
			}
			// -------------------------

			// Bomb ----------
			if self.input.bomb && !self.input_break.bomb && self.bomb > 0 {
				self.player.memo = "trans".to_owned();
				self.input_break.bomb = true;
				self.bomb -= 1;
				self.effects.push(Actor::effect_new(self.player.point, [0.0; 2], Vec::new(), "bomb_wave").to_relative_window())
			}
			// -------------------------

			// Enemyの更新
			// - Jsonから取得したデータから、Enemyの動作を書き換え
			// - 弾幕を張る
			// - 位置の更新
			for e in &mut self.enemys {
				if self.input_break.bomb {
					e.life = 0.0;
					self.score += 30;
					continue;
				}
				for m in 0..e.moving.len() {
					if e.moving[m].count == self.game_count[0] {
						e.accel = e.moving[m].accel;
						e.memo = e.moving[m].shot_type.clone();
					}
				}

				let pp = self.player.point;
				let mut es = &mut self.enshots;
				let gc = game_count_use;
				let rn = &self.rand;
				match e.memo.as_str() {
					"six" => shot_type::n_six(e, pp, es, gc),
					"four-two_disp"=> shot_type::n_four_two_disp(e, pp, es, rn),
					_ => (),
				}

				Actor::update_point(e, seconds);
			}
			//-------------------------

			let in_bbox = |ac1: &Actor, ac2: &Actor| {
				// rr > xx + yy
				let x = ac1.point[0] - ac2.point[0];
				let y = ac1.point[1] - ac2.point[1];
				let r = ac1.bbox_size + ac2.bbox_size;

				let xx = x * x;
				let yy = y * y;
				let rr = r * r;
				rr > xx + yy
			};

			// Update EnShot----------
			// Hit EnemyShots & Player
			for es in &mut self.enshots {
				es.count += 1;
				if self.input_break.bomb {
					es.life = 0.0;
					continue;
				}
				for esm in es.moving.iter() {
					if esm.count == es.count {
						es.accel = esm.accel;
						es.memo = esm.shot_type.clone();
					}
				}
				Actor::update_point_shot(es, seconds);

				let pl = &mut self.player;
				if pl.memo != "trans".to_owned() && in_bbox(pl, es) {
					es.life = 0.0;
					pl.life -= 1.0;
					*pl = Actor::trans_pleyer_new(pl.life).to_relative_window();
				}
			}
			let mut p_count = self.player.count;
			if self.player.memo == "trans".to_owned() {
				self.player.count += 1;
				if self.player.count > 180 {
					self.player.memo = "".to_owned();
					self.input_break.bomb = false;
					self.player.count= 0;
				}
			}
			// -------------------------

			// Hit PlayerShots & Boss----------
			for bs in &mut self.boss {
				for ps in &mut self.plshots {
					if in_bbox(bs, ps) {
						bs.life -= ps.life;
						ps.life = 0.0;
						if bs.life < 0.0 {
							self.score += 30;
						}
					}
				}
			}
			// -------------------------

			// Hit PlayerShots & Enemy, Player & Enemy----------
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

				if self.player.memo != "trans".to_owned() && in_bbox(&self.player, en) {
					self.player.life -= 1.0;
					self.player = Actor::trans_pleyer_new(self.player.life).to_relative_window();
				}
			}
			// -------------------------

			// Update Bomb----------
			for ef in &mut self.effects {
				match ef.memo.as_str() {
					"bomb_wave" => {
						ef.count += 1;
						if ef.count >= 180 {
							ef.life = 0.0;
						}
					}
					_ => (),
				}
			}
			// -------------------------

			// Clear zero_life Enemy, Shot----------
			self.plshots.retain(|s| s.life > 0.0);
			self.enemys.retain(|s| s.life > 0.0);
			self.enshots.retain(|s| s.life > 0.0);
			self.effects.retain(|s| s.life > 0.0);
			// -------------------------
		}
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
		graphics::clear(ctx);

		let pl_point = self.player.point;
		let mut game_count_use = 0;
		let graphics_draw = |ctx: &mut Context, fs: u32, ds: &str, dp: [f32; 2]| {
			let font = graphics::Font::new(ctx, "/SoberbaSerif-Regular.ttf", fs).unwrap();
			let display = graphics::Text::new(ctx, ds, &font).unwrap();
			let display_point = graphics::Point2::new(dp[0], dp[1]);
			graphics::draw(ctx, &display, display_point, 0.0).unwrap();

		};

		// match Window State
		match self.window_state {
			WindowState::Title => {
				// Print "poi-Project"
				graphics_draw(ctx, 100, "poi-Project", [300.0; 2]);

				// Write "-"
				graphics::rectangle(
					ctx,
					graphics::DrawMode::Fill,
					graphics::Rect::new(495.0, 380.0, 40.0, 10.0),
				);

				// Print "press Z key"
				graphics_draw(ctx, 30, "Please press Z key", [400.0, 600.0]);

				// Skip other code
				graphics::present(ctx);
				return Ok(());
			},
			WindowState::Gaming => {
				game_count_use = self.game_count[0];

			},
			WindowState::GamingBoss => {
				game_count_use = self.game_count[1];
				// Print Boss life
				let bs = &self.boss;
				if self.boss.len() >= 1 {
					let dis_str = format!("Boss: {}", bs[0].life);
					graphics_draw(ctx, 18, &dis_str, [bs[0].point[0] + 50.0, bs[0].point[1]]);
					let count_down = (bs[0].moving[0].count - self.game_count[1]) / 60;
					let dis_str = format!("Time: {}", count_down);
					graphics_draw(ctx, 18, &dis_str, [700.0 ,40.0]);
				} else {
					//eprintln!("there is no boss");
				}
			}
			_ => (),
		}

		// drow player circle
		if !(self.player.memo == "trans".to_owned() && game_count_use % 3 == 0) {
			Assets::draw_player(ctx, &mut self.assets, &self.player, self.input)?;
			if self.input.shift {
				graphics::circle(
					ctx,
					graphics::DrawMode::Fill,
					graphics::Point2::new(pl_point[0],pl_point[1]),
					10.0,
					0.1,
				);
			}
		}

		// drow shot rectangle
		for act in &mut self.plshots {
			let point = act.point;
			graphics::rectangle(
				ctx,
				graphics::DrawMode::Fill,
				graphics::Rect::new(point[0] - 8.0, point[1] - 15.0, 16.0, 30.0),
			);
		}

		// drow boss circle
		for act in &mut self.boss {
			let point = act.point;
			graphics::circle(
				ctx,
				graphics::DrawMode::Fill,
				graphics::Point2::new(point[0],point[1]),
				30.0,
				0.1,
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

		// draw effect
		for ef in &mut self.effects {
			match ef.memo.as_str() {
				"bomb_wave" => {
					let img = &self.assets.effect_img;
					let img_scale = ef.count as f32 * 0.1;
					let ce = 255 - ef.count as u8;
					let img_color = graphics::Color::from((ce, ce, ce, 255));

					let drawparams = graphics::DrawParam {
						dest: graphics::Point2::new(ef.point[0], ef.point[1]),
						offset: graphics::Point2::new(0.482, 0.5),
						scale: graphics::Point2::new(img_scale, img_scale),
						color: Some(img_color),
						..Default::default()
					};
					graphics::draw_ex(ctx, img, drawparams);
				},
				_ => (),
			}
		}

		// draw frame
		let drawable = &self.assets.frame_img;
		let params = graphics::DrawParam {
			scale: graphics::Point2::new(RELATIVE_X, RELATIVE_Y),
			..Default::default()
		};
		graphics::draw_ex(ctx, drawable, params);

		// Print score
		let dis_str = format!("Score: {}", self.score);
		graphics_draw(ctx, 18, &dis_str, [900.0, 100.0]);

		// Print Player life
		let dis_str = format!("Life: {}", self.player.life as usize);
		graphics_draw(ctx, 18, &dis_str, [900.0, 150.0]);

		// Print bomb
		let dis_str = format!("Bomb: {}", self.bomb as usize);
		graphics_draw(ctx, 18, &dis_str, [900.0, 200.0]);

		match self.window_state {
			WindowState::GameOver => {
				let drawable = &self.assets.brack_out_img;
				let params = graphics::DrawParam {
					..Default::default()
				};
				graphics::draw_ex(ctx, drawable, params);

				// ゆっくり明るくなっていく
				let mut c = 0;
				if self.game_count[0] * 5 < 255 {
					c = self.game_count[0] as u8 * 5;;
				} else {
					c = 255;
				}
				graphics::set_color(ctx, graphics::Color::from((c, c, c, 255)))?;
				// --------------------

				graphics_draw(ctx, 30, "GameOver", [400.0, 300.0]);
				if self.game_count[0] >= 240 {
					graphics_draw(ctx, 20, "Press Z to close winodw", [350.0, 400.0]);
				}

				graphics::set_color(ctx, graphics::Color::from((255, 255, 255, 255)))?;
			}
			WindowState::GameClear => {
				let drawable = &self.assets.brack_out_img;
				let params = graphics::DrawParam {
					..Default::default()
				};
				graphics::draw_ex(ctx, drawable, params);
				graphics_draw(ctx, 30, "GameClear", [400.0, 300.0]);
			},
			WindowState::ThankYouForPlaying => {
				// ゆっくり明るくなっていく
				let mut c = 0;
				if self.game_count[0] * 5 < 255 {
					c = self.game_count[0] as u8 * 5;;
				} else {
					c = 255;
				}
				graphics::set_color(ctx, graphics::Color::from((c, c, c, 255)))?;
				// --------------------

				// draw picture
				let drawable = &self.assets.ending_img;
				let params = graphics::DrawParam {
					..Default::default()
				};
				graphics::draw_ex(ctx, drawable, params);

				// draw text
				graphics_draw(ctx, 60, "Thank you", [700.0, 200.0]);
				graphics_draw(ctx, 60, "for playing", [750.0, 350.0]);

				if self.game_count[0] >= 240 {
					graphics_draw(ctx, 30, "Press Z to close winodw", [700.0, 550.0]);
				}
			}
			_ => (),
		}

		graphics::present(ctx);
		Ok(())

	}

	fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
		match keycode {
			Keycode::Up => self.input.up = true,
			Keycode::Down => self.input.down = true,
			Keycode::Right => self.input.right = true,
			Keycode::Left => self.input.left = true,
			Keycode::LShift => self.input.shift = true,
			Keycode::Z => self.input.shot = true,
			Keycode::X => self.input.bomb = true,
			_ => ()
		}
	}

	fn key_up_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
		match keycode {
			Keycode::Up => self.input.up = false,
			Keycode::Down => self.input.down = false,
			Keycode::Right => self.input.right = false,
			Keycode::Left => self.input.left = false,
			Keycode::LShift => self.input.shift = false,
			Keycode::Z => {
				self.input.shot = false;
				self.input_break.shot = false;
			}
			Keycode::X => self.input.bomb = false,
			_ => ()
		}
	}
}

