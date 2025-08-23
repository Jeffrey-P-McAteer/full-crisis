use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::{io, sync::Arc};
use tokio::time::Duration;

pub async fn run() -> Result<(), full_crisis::err::BoxError> {
    let game = full_crisis::GAME.get().unwrap();

    enable_raw_mode().map_err(full_crisis::err::eloc!())?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(full_crisis::err::eloc!())?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(full_crisis::err::eloc!())?;

    // Shared state
    let input = Arc::new(tokio::sync::Mutex::new(String::new()));

    loop {
        {
            if let Ok(evt_loop_val) = game.active_event_loop.try_read() {
                if *evt_loop_val == full_crisis::game::ActiveEventLoop::Exit {
                    break;
                }
            }
        }

        // Draw UI
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Min(1), Constraint::Length(3)])
                    .split(f.size());

                // Main display (menu options)
                let menu_text = vec![
                    Line::from(vec![Span::styled("Full Crisis - Main Menu", Style::default().fg(Color::Green))]),
                    Line::from(""),
                    Line::from("Available Commands:"),
                    Line::from(vec![
                        Span::styled("  continue", Style::default().fg(Color::Cyan)),
                        Span::raw(" (c) - Continue existing game")
                    ]),
                    Line::from(vec![
                        Span::styled("  new", Style::default().fg(Color::Cyan)),
                        Span::raw(" (n) - Start new game")
                    ]),
                    Line::from(vec![
                        Span::styled("  settings", Style::default().fg(Color::Cyan)),
                        Span::raw(" (s) - Game settings")
                    ]),
                    Line::from(vec![
                        Span::styled("  licenses", Style::default().fg(Color::Cyan)),
                        Span::raw(" (l) - View licenses")
                    ]),
                    Line::from(vec![
                        Span::styled("  quit", Style::default().fg(Color::Cyan)),
                        Span::raw(" (q) - Exit game")
                    ]),
                ];
                
                let menu_display = Paragraph::new(menu_text)
                    .block(Block::default().borders(Borders::ALL).title("Menu"));
                f.render_widget(menu_display, chunks[0]);

                // Input field
                if let Some(input_str) = maybe_block_on(input.lock(), 2) {
                    let input_str = input_str.to_string();
                    let input_widget = {
                        Paragraph::new(&*input_str)
                            .block(Block::default().borders(Borders::ALL).title("Command"))
                    };
                    f.render_widget(input_widget, chunks[1]);
                }
            })
            .map_err(full_crisis::err::eloc!())?;

        // Poll for key events or UI update
        if event::poll(Duration::from_millis(100)).map_err(full_crisis::err::eloc!())? {
            match event::read().map_err(full_crisis::err::eloc!())? {
                Event::Key(key) => {
                    let mut input_str = input.lock().await;
                    match key.code {
                        KeyCode::Char(c) => input_str.push(c),
                        KeyCode::Backspace => {
                            input_str.pop();
                        }
                        KeyCode::Enter => {
                            // Process command
                            let cmd = input_str.trim().to_lowercase();
                            
                            match cmd.as_str() {
                                "continue" | "c" => {
                                    if let Ok(mut evt_loop_wguard) = game.active_event_loop.write() {
                                        *evt_loop_wguard = full_crisis::game::ActiveEventLoop::WelcomeScreen(full_crisis::game::WelcomeScreenView::ContinueGame);
                                    }
                                }
                                "new" | "new game" | "n" => {
                                    if let Ok(mut evt_loop_wguard) = game.active_event_loop.write() {
                                        *evt_loop_wguard = full_crisis::game::ActiveEventLoop::WelcomeScreen(full_crisis::game::WelcomeScreenView::NewGame);
                                    }
                                }
                                "settings" | "s" => {
                                    if let Ok(mut evt_loop_wguard) = game.active_event_loop.write() {
                                        *evt_loop_wguard = full_crisis::game::ActiveEventLoop::WelcomeScreen(full_crisis::game::WelcomeScreenView::Settings);
                                    }
                                }
                                "licenses" | "l" => {
                                    if let Ok(mut evt_loop_wguard) = game.active_event_loop.write() {
                                        *evt_loop_wguard = full_crisis::game::ActiveEventLoop::WelcomeScreen(full_crisis::game::WelcomeScreenView::Licenses);
                                    }
                                }
                                "quit" | "exit" | "q" | "e" => {
                                    if let Ok(mut evt_loop_wguard) = game.active_event_loop.write() {
                                        *evt_loop_wguard = full_crisis::game::ActiveEventLoop::Exit;
                                    }
                                }
                                "" => {
                                    // Empty command, do nothing
                                }
                                _ => {
                                    // Unknown command, could show help or ignore
                                }
                            }

                            input_str.clear();
                        }
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
                _ => {}
            }
        }

    }

    // Restore terminal
    disable_raw_mode().map_err(full_crisis::err::eloc!())?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .map_err(full_crisis::err::eloc!())?;
    terminal.show_cursor().map_err(full_crisis::err::eloc!())?;

    Ok(())
}

use std::{
    future::{Future, IntoFuture},
    sync::{Condvar, Mutex},
    task::{Context, Poll, Wake, Waker},
};

pub fn maybe_block_on<F: IntoFuture>(fut: F, max_polls: usize) -> Option<F::Output> {
    // TODO See https://docs.rs/pollster/latest/src/pollster/lib.rs.html#111-131
    let mut fut = core::pin::pin!(fut.into_future());

    // Signal used to wake up the thread for polling as the future moves to completion. We need to use an `Arc`

    // because, although the lifetime of `fut` is limited to this function, the underlying IO abstraction might keep

    // the signal alive for far longer. `Arc` is a thread-safe way to allow this to happen.

    // TODO: Investigate ways to reuse this `Arc<Signal>`... perhaps via a `static`?

    let signal = Arc::new(Signal::new());

    // Create a context that will be passed to the future.

    let waker = Waker::from(Arc::clone(&signal));

    let mut context = Context::from_waker(&waker);

    // Poll the future to completion
    let mut remaining_polls = max_polls;
    loop {
        if remaining_polls < 1 {
            break;
        }
        remaining_polls -= 1;

        match fut.as_mut().poll(&mut context) {
            Poll::Pending => signal.wait(),

            Poll::Ready(item) => return Some(item),
        }
    }
    None
}

/// An extension trait that allows blocking on a future in suffix position.

pub trait FutureExt: Future {
    /// Block the thread until the future is ready.
    ///
    /// # Example
    ///
    /// ```
    /// use pollster::FutureExt as _;
    ///
    /// let my_fut = async {};
    ///
    /// let result = my_fut.block_on();
    /// ```
    fn block_on(self) -> Self::Output
    where
        Self: Sized,
    {
        maybe_block_on(self, usize::MAX).expect("block_on ran forever")
    }
}

impl<F: Future> FutureExt for F {}

enum SignalState {
    Empty,
    Waiting,
    Notified,
}

struct Signal {
    state: Mutex<SignalState>,
    cond: Condvar,
}

impl Signal {
    fn new() -> Self {
        Self {
            state: Mutex::new(SignalState::Empty),

            cond: Condvar::new(),
        }
    }

    fn wait(&self) {
        let mut state = self.state.lock().unwrap();

        match *state {
            // Notify() was called before we got here, consume it here without waiting and return immediately.
            SignalState::Notified => *state = SignalState::Empty,

            // This should not be possible because our signal is created within a function and never handed out to any

            // other threads. If this is the case, we have a serious problem so we panic immediately to avoid anything

            // more problematic happening.
            SignalState::Waiting => {
                unreachable!("Multiple threads waiting on the same signal: Open a bug report!");
            }

            SignalState::Empty => {
                // Nothing has happened yet, and we're the only thread waiting (as should be the case!). Set the state

                // accordingly and begin polling the condvar in a loop until it's no longer telling us to wait. The

                // loop prevents incorrect spurious wakeups.

                *state = SignalState::Waiting;

                while let SignalState::Waiting = *state {
                    state = self.cond.wait(state).unwrap();
                }
            }
        }
    }

    fn notify(&self) {
        let mut state = self.state.lock().unwrap();

        match *state {
            // The signal was already notified, no need to do anything because the thread will be waking up anyway
            SignalState::Notified => {}

            // The signal wasn't notified but a thread isn't waiting on it, so we can avoid doing unnecessary work by

            // skipping the condvar and leaving behind a message telling the thread that a notification has already

            // occurred should it come along in the future.
            SignalState::Empty => *state = SignalState::Notified,

            // The signal wasn't notified and there's a waiting thread. Reset the signal so it can be wait()'ed on again

            // and wake up the thread. Because there should only be a single thread waiting, `notify_all` would also be

            // valid.
            SignalState::Waiting => {
                *state = SignalState::Empty;

                self.cond.notify_one();
            }
        }
    }
}

impl Wake for Signal {
    fn wake(self: Arc<Self>) {
        self.notify();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.notify();
    }
}
