extern crate ggez;

use ggez::*;

struct MainState {

}

impl ggez::event::EventHandler for MainState {
  fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
      Ok(())
  }
  fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
      Ok(())
  }
}

fn main() {
    let state = &mut MainState { };
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("generative_art", "awesome_person", c).unwrap();
    event::run(ctx, state).unwrap();
}
