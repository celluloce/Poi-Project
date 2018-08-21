extern crate ggez;

use ggez::*;

struct MainState {

}

impl MainState {
	fn new(ctx: &mut Context) -> GameResult<MainState> {
		let s = MainState{};

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
        .window_mode(conf::WindowMode::default().dimensions(640, 480));

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
