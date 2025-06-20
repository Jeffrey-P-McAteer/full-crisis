

pub fn run() -> Result<(), crate::err::BoxError> {

  println!("TODO cli mode!");

  println!("GAME.get().unwrap() = {:?}", crate::GAME.get().unwrap());

  Ok(())
}
