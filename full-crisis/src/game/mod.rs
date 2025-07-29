#[derive(Debug)]
pub struct GameState {
    pub active_event_loop: std::sync::Arc<std::sync::RwLock<ActiveEventLoop>>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            active_event_loop: std::sync::Arc::new(std::sync::RwLock::new(ActiveEventLoop::WelcomeScreen(
                WelcomeScreenView::Empty,
            ))),
        }
    }
}

/// This tracks what event loop should be running
#[derive(Debug, PartialEq, Eq)]
pub enum ActiveEventLoop {
    WelcomeScreen(WelcomeScreenView),
    ActiveGame(GameView),
    Exit,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WelcomeScreenView {
    Empty,
    NewGame,
    ContinueGame,
    Settings,
}

#[derive(Debug, PartialEq, Eq)]
pub enum GameView {}
