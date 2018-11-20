use Actor;
use MainState;
use rand::{Rng, ThreadRng};
use std::f32;
use std::f32::consts::PI;
// Shot: [Angle(0.0 <= x < 2.0, 真下が0, 右回り), scalar]

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
pub fn six(enemy: &mut Actor, p_point: [f32; 2],  en_shots: &mut Vec<Actor>, count: u32) {
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

pub fn four_two_disp(enemy: &mut Actor, p_point: [f32; 2],  en_shots: &mut Vec<Actor>, count: u32, rand: &ThreadRng) {
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
	enemy.memo = String::new();
}

pub fn b_six_rotate(enemy: &mut Actor, p_point: [f32; 2],  en_shots: &mut Vec<Actor>, count: u32) {
	if count < 50 || count % 3 != 0 {
		return ();
	}
	for i in 0..6 {
		let angle_plus = {
			let round_per_sec = 1.0 / 30.0;
			let count_max60 = (count % 60) as f32 ;
			let round_1sec = count_max60 * round_per_sec;
			round_1sec + (count as f32 * 0.00783)
		};
		let ep = enemy.point;
		let shot_scal = 300.0;
		let angle = i as f32 / 3.0 + angle_plus;
		let sv = [angle, shot_scal];

		en_shots.push(Actor::enemy_shot_new(ep, sv));
	}
}

pub fn b_six_fireflower(enemy: &mut Actor, p_point: [f32; 2],  en_shots: &mut Vec<Actor>, count: u32) {
	// 5秒周期で射出
	let shot_time = count % 300;
	if count >= 50 && shot_time == 50 {
		// shot_timeの間隔でoriginshotを生成
		for i in 0..6 {
			let ep = enemy.point;
			let shot_scal = 300.0;
			let angle = i as f32 / 3.0;
			let sv = [angle, shot_scal];
			let sa = [0.0, -shot_scal];
			let em = Vec::new();
			let estr = "origin";

			en_shots.push(Actor::enemy_shot_from(ep, sv, sa, em, estr));
		}
	} else {
		if shot_time == 120 || shot_time == 180 {
			// ""と"origin", "split"を分離
			// 前者は残し、後者は弾幕生成のために使われ消える
			let mut while_i = 0;
			let mut enshots_org_spl: Vec<Actor> = Vec::new();
			loop {
				let length = en_shots.len();
				if while_i == length {
					break;
				}
				if en_shots[while_i].memo == "".to_owned() {
					while_i += 1;
				} else {
					enshots_org_spl.push(en_shots[while_i].clone());
					en_shots.swap_remove(while_i);
				}
			}
			for es in enshots_org_spl {
				let mut push_shot_memo = "";
				match es.memo.as_str() {
					"origin" => push_shot_memo = "split",
					"split" => push_shot_memo = "",
					_ => (),
				}

				let esp = es.point;
				let esv = es.velocity;
				let esa = [0.0; 2];
				let esm = Vec::new();
				let esstr = push_shot_memo;
				for i in 1..7 {
					let esv = [esv[0] + 0.18 * i as f32, 250.0];
					en_shots.push(Actor::enemy_shot_from(esp, esv, esa, esm.clone(), esstr));
					let esv = [esv[0] - 0.18 * i as f32, 250.0];
					en_shots.push(Actor::enemy_shot_from(esp, esv, esa, esm.clone(), esstr));
				}
			}
		}
	}

}
