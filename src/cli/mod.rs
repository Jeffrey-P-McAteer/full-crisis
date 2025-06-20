

pub async fn run() -> Result<(), crate::err::BoxError> {
  let game = crate::GAME.get().unwrap();
  loop {
    {
      if *game.active_event_loop.read().await == crate::game::ActiveEventLoop::Exit {
          break;
      }

    }

    // idk
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
  }
  Ok(())
}
