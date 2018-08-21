extern crate ggez;

use ggez::*;

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
struct MainState {
	player: Actor,
}

impl MainState {
	fn new(ctx: &mut Context) -> GameResult<MainState> {
		let s = MainState{
			player: Actor::player_new(),
		};

		Ok(s)
	}
}

impl ggez::event::EventHandler for MainState {
  fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
      Ok(())
  }
  fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
      Ok(())
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
