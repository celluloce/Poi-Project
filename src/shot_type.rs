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
	for i in 1..=3 {
		for c in 0..6 {
			let ep = enemy.point;
			let sv = {
				let shot_scal = 100.0  + 40.0* i as f32;
				let angle = (c as f32) / 3.0 + get_angle_from_points(enemy.point, p_point);
				[angle, shot_scal]
			};
			en_shots.push(Actor::enemy_shot_new(ep, sv));
		}
	}
	enemy.memo = String::new();
}
