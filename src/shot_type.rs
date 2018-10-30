use Actor;
use MainState;
pub fn six(enemy: &mut Actor, en_shots: &mut Vec<Actor>, count: u32) {
	if count % 50 == 0 {
		for c in 0..5 {
			let ep = enemy.point;
			let sv = {
				let shot_scal = 100.0;
				let angle = (c as f32) / 3.0;
				[angle, shot_scal]
			};
			en_shots.push(Actor::enemy_shot_new(ep, sv));
		}
	}
}
