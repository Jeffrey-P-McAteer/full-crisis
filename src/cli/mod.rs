
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span,Line},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{error::Error, io, sync::{Arc}};
use tokio::{sync::mpsc, time::{self, Duration}};
use rand::Rng;


pub async fn run() -> Result<(), crate::err::BoxError> {
  let game = crate::GAME.get().unwrap();
  /*loop {
    {
      if *game.active_event_loop.read().await == crate::game::ActiveEventLoop::Exit {
          break;
      }

    }

    // idk
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
  }*/


    enable_raw_mode().map_err(crate::err::eloc!())?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(crate::err::eloc!())?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(crate::err::eloc!())?;

    // Shared state
    let number = Arc::new(tokio::sync::Mutex::new(0));
    let input = Arc::new(tokio::sync::Mutex::new(String::new()));

    // Channel to wake up UI on updates
    let (tx, mut rx) = mpsc::channel::<()>(1);

    // Clone for background task
    let number_clone = Arc::clone(&number);
    let tx_clone = tx.clone();

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let mut num = number_clone.lock().await;
            *num = rand::thread_rng().gen_range(1..=100);
            let _ = tx_clone.send(()).await;
        }
    });

    loop {
        {
          if *game.active_event_loop.read().await == crate::game::ActiveEventLoop::Exit {
              break;
          }
        }

        // Draw UI
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(f.size());

            // Main display (updated number)
            if let Some(num) = maybe_block_on(number.lock(), 2) {
                let number_display = {
                    Paragraph::new(Line::from(vec![
                        Span::styled("Random Number: ", Style::default().fg(Color::Blue)),
                        Span::raw(format!("{}", *num)),
                    ]))
                    .block(Block::default().borders(Borders::ALL).title("Output"))
                };
                f.render_widget(number_display, chunks[0]);
            }

            // Input field
            if let Some(input_str) = maybe_block_on(input.lock(), 2) {
                let input_str = input_str.to_string();
                let input_widget = {
                    Paragraph::new(&*input_str)
                        .block(Block::default().borders(Borders::ALL).title("Command"))
                };
                f.render_widget(input_widget, chunks[1]);
            }
        }).map_err(crate::err::eloc!())?;

        // Poll for key events or UI update
        if event::poll(Duration::from_millis(100)).map_err(crate::err::eloc!())? {
            match event::read().map_err(crate::err::eloc!())? {
                Event::Key(key) => {
                    let mut input_str = input.lock().await;
                    match key.code {
                        KeyCode::Char(c) => input_str.push(c),
                        KeyCode::Backspace => { input_str.pop(); },
                        KeyCode::Enter => {
                            // Process command
                            //println!("Entered command: {}", *input_str);
                            let cmd = input_str.to_string();
                            // TODO better input processing
                            if cmd == "exit" || cmd == "quit" || cmd == "e" || cmd == "q" {
                                *game.active_event_loop.write().await = crate::game::ActiveEventLoop::Exit;
                            }

                            input_str.clear();
                        },
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Wake UI if background task sent update
        if let Ok(_) = rx.try_recv() {
            // nothing needed, UI will re-render on next loop
        }
    }

    // Restore terminal
    disable_raw_mode().map_err(crate::err::eloc!())?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).map_err(crate::err::eloc!())?;
    terminal.show_cursor().map_err(crate::err::eloc!())?;

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

    fn block_on(self) -> Self::Output where Self: Sized { maybe_block_on(self, usize::MAX).expect("block_on ran forever") }

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
