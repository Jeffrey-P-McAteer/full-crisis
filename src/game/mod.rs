
#[derive(Debug)]
pub struct GameState {
  pub active_event_loop: ActiveEventLoop,
}

impl GameState {
  pub fn new() -> Self {
    Self {
      active_event_loop: ActiveEventLoop::WelcomeScreen(WelcomeScreen_View::Empty)
    }
  }
}

/// This tracks what event loop should be running
#[derive(Debug)]
pub enum ActiveEventLoop {
  WelcomeScreen(WelcomeScreen_View),
  ActiveGame(Game_View),
}

#[derive(Debug)]
pub enum WelcomeScreen_View {
  Empty,
  NewGame,
  LoadGame,
  Settings
}

#[derive(Debug)]
pub enum Game_View {


}


