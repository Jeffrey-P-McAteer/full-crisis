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
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ActiveEventLoop {
    WelcomeScreen(WelcomeScreenView),
    ActiveGame(GameView),
    Exit,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum WelcomeScreenView {
    Empty,
    NewGame,
    ContinueGame,
    Settings,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GameView {
    FirstScene
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OSColorTheme {
    Light,
    Dark,
}

unsafe impl Sync for GameState {}
unsafe impl Sync for OSColorTheme {}
unsafe impl Sync for GameView {}
unsafe impl Sync for WelcomeScreenView {}
unsafe impl Sync for ActiveEventLoop {}
