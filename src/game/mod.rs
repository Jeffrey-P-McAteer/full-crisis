
#[derive(Debug)]
pub struct GameState {
  pub active_event_loop: tokio::sync::RwLock<ActiveEventLoop>,
}

impl GameState {
  pub fn new() -> Self {
    Self {
      active_event_loop: tokio::sync::RwLock::new(
        ActiveEventLoop::WelcomeScreen(WelcomeScreen_View::Empty)
      ),

    }
  }
}

/// This tracks what event loop should be running
#[derive(Debug, PartialEq, Eq)]
pub enum ActiveEventLoop {
  WelcomeScreen(WelcomeScreen_View),
  ActiveGame(Game_View),
  Exit
}

#[derive(Debug, PartialEq, Eq)]
pub enum WelcomeScreen_View {
  Empty,
  NewGame,
  LoadGame,
  Settings
}

#[derive(Debug, PartialEq, Eq)]
pub enum Game_View {


}


