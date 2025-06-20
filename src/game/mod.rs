
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
pub enum ActiveEventLoop {
  WelcomeScreen(WelcomeScreen_View),
  ActiveGame(Game_View),
}

pub enum WelcomeScreen_View {
  Empty,
  NewGame,
  LoadGame,
  Settings
}

pub enum Game_View {


}


