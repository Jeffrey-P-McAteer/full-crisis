
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
  //let game = crate::GAME.get().unwrap();
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

    let h = tokio::runtime::Handle::current();

    loop {
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
            let number_display = {
                let num = pollster::block_on(number.lock());
                Paragraph::new(Line::from(vec![
                    Span::styled("Random Number: ", Style::default().fg(Color::Blue)),
                    Span::raw(format!("{}", *num)),
                ]))
                .block(Block::default().borders(Borders::ALL).title("Output"))
            };
            f.render_widget(number_display, chunks[0]);

            // Input field
            let input_widget = {
                let input_str = pollster::block_on(input.lock()).clone();
                Paragraph::new(input_str)
                    .block(Block::default().borders(Borders::ALL).title("Command"))
            };
            f.render_widget(input_widget, chunks[1]);
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
                            println!("Entered command: {}", *input_str);
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
