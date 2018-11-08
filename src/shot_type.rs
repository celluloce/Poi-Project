use Actor;
use MainState;
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

		for i in 1..=3 {
			let sv = [angle, shot_scal + 40.0 * i as f32];
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

pub fn b_six_rotate(enemy: &mut Actor, p_point: [f32; 2],  en_shots: &mut Vec<Actor>, count: u32) {
	if count % 3 == 0 {
		for i in 0..6 {
			let angle_plus = {
				let round_per_sec = 1.0 / 30.0;
				let count_max60 = (count % 60) as f32 ;
				let round_1sec = count_max60 * round_per_sec;
				round_1sec + (count as f32 / 4312.9)
				// ちょっとずらすために変な値を入れる
			};
			let ep = enemy.point;
			let shot_scal = 300.0;
			let angle = i as f32 / 3.0 + angle_plus;
			let sv = [angle, shot_scal];

			en_shots.push(Actor::enemy_shot_new(ep, sv));
		}

	}
}
