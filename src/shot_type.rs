use Actor;
use MainState;
use MovingElement;
use rand::{Rng, ThreadRng};
use std::f32;
use std::f32::consts::PI;

// Shot: [Angle(0.0 <= x < 2.0, 真下が0, 右回り), scalar]
// 関数名規則：
//   n_..: 道中敵が使用
//   m_..: 中ボスが使用
//   b_..: ステージボスが使用
//   ..: private関数
//
// six: ６方向に単発 自機依存 3060countからアプグレード（要修正）
// four_two_disp: ４方向＊２方向＊７発 角度; 自機, 乱数依存
// m_six_rotate: ６方向＊連射 右回り
// m_six_fireflower: ３００count周期＊６方向＊２度分裂
// b_caopet_bomb: １方向 角度; 乱数依存 カーブした後直線＊１２０C毎に子弾幕生成
// b_normal: ８方向；乱数依存＊２連＊連射 アップグレード機能必要
//
// b_6rotate_4rand:
//   six_roteate＊5連 右回り
//   six_roteate＊5連 左回り
//   4方向＊２方向＊３連 乱角度
//
// b_2fireflower_4pdis:
//   fireflower＊２方向＊３連
//   ４方向＊７連 単発 自機依存角
//
// b_6carpet_fireflower:
//   carpet＊６方向 -> fireflower＊速度,角度乱数

fn get_angle_from_points (p1: [f32; 2], p2: [f32; 2]) -> f32 {
	//    p1
	//   /|<- return angle
	//  / |
	// p2-+
	let dis_x = p2[0] - p1[0];
	let dis_y = p2[1] - p1[1];
	let angle = dis_x.atan2(dis_y);
	angle / PI
}

fn six_rotate(enemy: &mut Actor, p_point: [f32; 2],  en_shots: &mut Vec<Actor>, count: u32, right: bool) {
	for i in 0..6 {
		let angle_plus = {
			let round_per_sec = 1.0 / 30.0;
			let count_max60 = (count % 60) as f32 ;
			let round_1sec = count_max60 * round_per_sec;
			round_1sec + (count as f32 * 0.00783)
		};
		let ep = enemy.point;
		let shot_scal = 300.0;
		let mut angle: f32;
		if right {
			angle = i as f32 / 3.0 + angle_plus;
		} else {
			angle = i as f32 / 3.0 - angle_plus;
		}
		let sv = [angle, shot_scal];

		en_shots.push(Actor::enemy_shot_new(ep, sv));
	}
}

fn four_two_disp(enemy: &mut Actor, p_point: [f32; 2],  en_shots: &mut Vec<Actor>, rand: &ThreadRng) {
	let r = rand.to_owned().gen::<f32>();
	for i in 0..4 {
		let ep = enemy.point;
		let mut shot_scal = 120.0;
		let angle = (i as f32) / 2.0 + get_angle_from_points(enemy.point, p_point);
		let rand_angle = angle + 0.1 * (r - 0.5);
		for j in 0..7 {
			let sv = [rand_angle + 0.05, shot_scal + 30.0 * j as f32];
			en_shots.push(Actor::enemy_shot_new(ep, sv));
			let sv = [rand_angle - 0.05, shot_scal + 30.0 * j as f32];
			en_shots.push(Actor::enemy_shot_new(ep, sv));
		}
	}
}

fn fireflower(
	enemy: &mut Actor,
	p_point: [f32; 2],
	en_shots: &mut Vec<Actor>,
	count: u32,
	shot_num: u32,
	shot_angle: f32,
	origin: bool) {
	if origin {
		// shot_timeの間隔でoriginshotを生成
		for i in 0..shot_num {
			let angle = (i as f32 + 2.0 * shot_angle) * 2.0 / shot_num as f32;

			let push_shot = Actor {
				point: enemy.point,
				velocity: [angle, 300.0],
				accel: [0.0, -300.0],
				memo: "origin".to_owned(),
				..Default::default()
			};
			en_shots.push(push_shot);
		}
	} else {
		let mut push_enshot_buf: Vec<Actor> = Vec::new();
		for es in &mut *en_shots {
			match es.memo.as_str() {
				"origin" => {
					if es.count == 60{
						// 分裂させる
						es.memo = "origin_d".to_owned();
						let mut push_shot = Actor {
							point: es.point,
							velocity: es.velocity,
							memo: "split".to_owned(),
							..Default::default()
						};
						for i in 0..4 {
							push_shot.velocity[1] = 250.0;
							push_shot.velocity[0] += 0.18 * i as f32;
							push_enshot_buf.push(push_shot.clone());
							push_shot.velocity[0] -= 0.18 * i as f32;
							push_enshot_buf.push(push_shot.clone());
						}
					}
				},
				"split" => {
					if es.count == 60 {
						es.memo = "split_d".to_owned();
						let mut push_shot = Actor {
							point: es.point,
							velocity: es.velocity,
							memo: "".to_owned(),
							..Default::default()
						};
						for i in 1..7 {
							push_shot.velocity[1] = 250.0;
							push_shot.velocity[0] += 0.18 * i as f32;
							push_enshot_buf.push(push_shot.clone());
							push_shot.velocity[0] -= 0.18 * i as f32;
							push_enshot_buf.push(push_shot.clone());
						}
					}
				},
				_ => (),
			}
			if es.memo == "origin_d".to_owned() || es.memo == "split_d".to_owned() {
				es.life = 0.0;
			}
		}
		for peb in push_enshot_buf {
			en_shots.push(peb);
		}
	}
}

fn carpet_bomb(
	enemy: &mut Actor,
	p_point: [f32; 2],
	en_shots: &mut Vec<Actor>,
	count: u32,
	rand: &ThreadRng,
	shot_origin: bool) {
	if shot_origin {
		let shot_scal = 200.0;
		let angle = 2.0 * rand.to_owned().gen::<f32>();

		let mut push_shot = Actor {
			point: enemy.point,
			velocity: [angle, shot_scal],
			accel: [0.0, 0.0],
			moving: vec![
				MovingElement::new(30, [0.7, 0.0], "origin"),
				MovingElement::new(120, [0.0, 0.0], ""),
			],
			memo: "".to_owned(),
			..Default::default()
		};

		en_shots.push(push_shot.clone());
		push_shot.velocity[0] += 2.0 / 3.0;
		en_shots.push(push_shot.clone());
		push_shot.velocity[0] += 2.0 / 3.0;
		en_shots.push(push_shot.clone());
	} else {
		let split_rate = count % 120;
		for es in en_shots.clone() {
			// origin弾のみ弾幕を生成
			match es.memo.as_str() {
				"origin" => (),
				_ => continue,
			}
			if count % 5 == 0 {
				let mut push_shot = Actor {
					point: es.point,
					velocity: es.velocity,
					memo: "".to_owned(),
					..Default::default()
				};
				push_shot.velocity[1] = 250.0;
				push_shot.velocity[0] += 0.7;
				en_shots.push(push_shot.clone());
				push_shot.velocity[0] -= 1.4;
				en_shots.push(push_shot.clone());
			}
		}
	}
}


pub fn n_six(enemy: &mut Actor, p_point: [f32; 2],  en_shots: &mut Vec<Actor>, count: u32) {
	for i in 0..6 {
		let ep = enemy.point;
		let mut shot_scal = 100.0;
		let angle = (i as f32) / 3.0 + get_angle_from_points(enemy.point, p_point);

		for j in 1..=3 {
			let sv = [angle, shot_scal + 40.0 * j as f32];
			en_shots.push(Actor::enemy_shot_new(ep, sv));
		}

		if count >= 3060 {
			shot_scal += 60.0;
			let sv = [angle + 0.12, shot_scal];
			en_shots.push(Actor::enemy_shot_new(ep, sv));
			let sv = [angle - 0.12, shot_scal];
			en_shots.push(Actor::enemy_shot_new(ep, sv));

			shot_scal += 30.0;
			let sv = [angle + 1.32, shot_scal];
			en_shots.push(Actor::enemy_shot_new(ep, sv));
			let sv = [angle - 1.32, shot_scal];
			en_shots.push(Actor::enemy_shot_new(ep, sv));

		}
	}
	enemy.memo = String::new();
}

pub fn n_four_two_disp(enemy: &mut Actor, p_point: [f32; 2],  en_shots: &mut Vec<Actor>, rand: &ThreadRng) {
	four_two_disp(enemy, p_point, en_shots, rand);
	enemy.memo = String::new();
}

pub fn m_six_rotate(enemy: &mut Actor, p_point: [f32; 2],  en_shots: &mut Vec<Actor>, count: u32) {
	if count > 50 && count % 3 == 0 {
		six_rotate(enemy, p_point, en_shots, count, true);
	}
}

pub fn m_six_fireflower(enemy: &mut Actor, p_point: [f32; 2],  en_shots: &mut Vec<Actor>, count: u32) {
	// 5秒周期で射出
	let shot_time = count % 300;
	if count >= 50 && shot_time == 50 {
		fireflower(enemy, p_point, en_shots, shot_time, 6, 0.0, true);
	} else {
		fireflower(enemy, p_point, en_shots, shot_time, 6, 0.0, false);
	}
}

pub fn b_normal (
	enemy: &mut Actor,
	p_point: [f32; 2],
	en_shots: &mut Vec<Actor>,
	count: u32,
	rand: &ThreadRng) {
	let rate = count % 60;
	if !(rate == 0 || rate == 20) {
		return ();
	}
	let shot_n: u32 = 16;
	let angle_rand = rand.to_owned().gen::<f32>();
	for i in 0..shot_n {
		let ep = enemy.point;
		let mut shot_scal = 100.0;
		let angle = ((i as f32) + 2.0 * angle_rand) * 2.0 / shot_n as f32;

		for j in 1..=3 {
			let sv = [angle, shot_scal + 40.0 * j as f32];
			en_shots.push(Actor::enemy_shot_new(ep, sv));
		}
	}
}
pub fn b_6rotate_4rand(
	enemy: &mut Actor,
	p_point: [f32; 2],
	en_shots: &mut Vec<Actor>,
	count: u32,
	rand: &ThreadRng) {
		let rate = count % 350;
		if rate % 5 == 0 && rate <= 90 {
			six_rotate(enemy, p_point, en_shots, count, true);
		} else if rate % 5 == 0 && rate <= 200  {
			if rate >= 110 {
			six_rotate(enemy, p_point, en_shots, count, false);
			}
		} else if rate % 30 == 0 && rate <= 320 {
			four_two_disp(enemy, p_point, en_shots, rand);
		} else {}
}

pub fn b_2fireflower_4pdis(
	enemy: &mut Actor,
	p_point: [f32; 2],
	en_shots: &mut Vec<Actor>,
	count: u32,
	rand: &ThreadRng) {
		let rate = count % 300;
		if rate % 30 == 0 && rate <= 90 {
			let r = rand.to_owned().gen::<f32>();
			fireflower(enemy, p_point, en_shots, rate, 2, r, true);
		} else if rate % 30 == 0 && rate <= 210 {
			four_two_disp(enemy, p_point, en_shots, rand);
		}
		fireflower(enemy, p_point, en_shots, rate, 2, 0.0, false);
}

pub fn b_6carpet_fireflower(
	enemy: &mut Actor,
	p_point: [f32; 2],
	en_shots: &mut Vec<Actor>,
	count: u32,
	rand: &ThreadRng) {
	let rate = count % 500;
	if  rate % 3 == 0 && rate % 90 <= 60 && rate <= 240 {
		six_rotate(enemy, p_point, en_shots, count, true);
	} else if rate >= 250 && rate % 30 == 0 {
		carpet_bomb(enemy, p_point, en_shots, count, rand, true);
	}
	if rate % 40 < 20 {
		carpet_bomb(enemy, p_point, en_shots, count, rand, false);
	}
}
