use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io;
use full_crisis::gui::types::{DifficultyLevel, GameSettings};

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppState {
    MainMenu,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SettingsSection {
    Difficulty,
    Language,
    Autosave,
    CrisesFolder,
}

#[derive(Debug)]
struct AppData {
    state: AppState,
    main_menu_state: ListState,
    main_menu_items: Vec<MainMenuChoice>,
    settings_section: SettingsSection,
    difficulty_state: ListState,
    difficulty_items: Vec<DifficultyLevel>,
    language_state: ListState,
    language_items: Vec<String>,
    autosave_state: ListState,
    autosave_items: Vec<bool>,
    settings: GameSettings,
    crises_folder_input: String,
    editing_crises_folder: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MainMenuChoice {
    Continue,
    New,
    Settings,
    Licenses,
    Quit,
}

impl std::fmt::Display for MainMenuChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MainMenuChoice::Continue => write!(f, "Continue Game"),
            MainMenuChoice::New => write!(f, "New Game"),
            MainMenuChoice::Settings => write!(f, "Settings"),
            MainMenuChoice::Licenses => write!(f, "Licenses"),
            MainMenuChoice::Quit => write!(f, "Quit"),
        }
    }
}

pub async fn run() -> Result<(), full_crisis::err::BoxError> {
    let game = full_crisis::GAME.get().unwrap();

    enable_raw_mode().map_err(full_crisis::err::eloc!())?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(full_crisis::err::eloc!())?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(full_crisis::err::eloc!())?;

    let main_menu_items = vec![
        MainMenuChoice::Continue,
        MainMenuChoice::New,
        MainMenuChoice::Settings,
        MainMenuChoice::Licenses,
        MainMenuChoice::Quit,
    ];

    let difficulty_items = DifficultyLevel::ALL.to_vec();
    
    let language_items: Vec<String> = full_crisis::language::get_available_languages()
        .into_iter()
        .map(|(code, _)| code)
        .collect();

    let autosave_items = vec![true, false];

    let settings = full_crisis::gui::GameWindow::load_settings();
    
    let mut app_data = AppData {
        state: AppState::MainMenu,
        main_menu_state: ListState::default(),
        main_menu_items,
        settings_section: SettingsSection::Difficulty,
        difficulty_state: ListState::default(),
        difficulty_items,
        language_state: ListState::default(),
        language_items,
        autosave_state: ListState::default(),
        autosave_items,
        crises_folder_input: settings.game_crises_folder.clone(),
        settings,
        editing_crises_folder: false,
    };

    // Set initial selections
    app_data.main_menu_state.select(Some(0));
    
    if let Some(pos) = app_data.difficulty_items.iter().position(|&d| d == app_data.settings.difficulty_level) {
        app_data.difficulty_state.select(Some(pos));
    }
    
    if let Some(pos) = app_data.language_items.iter().position(|lang| lang == &app_data.settings.language) {
        app_data.language_state.select(Some(pos));
    }
    
    if let Some(pos) = app_data.autosave_items.iter().position(|&autosave| autosave == app_data.settings.autosave) {
        app_data.autosave_state.select(Some(pos));
    }

    loop {
        {
            if let Ok(evt_loop_val) = game.active_event_loop.try_read() {
                if *evt_loop_val == full_crisis::game::ActiveEventLoop::Exit {
                    break;
                }
            }
        }

        terminal.draw(|f| {
            match app_data.state {
                AppState::MainMenu => draw_main_menu(f, &mut app_data),
                AppState::Settings => draw_settings(f, &mut app_data),
            }
        }).map_err(full_crisis::err::eloc!())?;

        if event::poll(std::time::Duration::from_millis(50)).map_err(full_crisis::err::eloc!())? {
            if let Event::Key(key) = event::read().map_err(full_crisis::err::eloc!())? {
                if key.kind == KeyEventKind::Press {
                    match app_data.state {
                        AppState::MainMenu => {
                            if handle_main_menu_input(&mut app_data, key, game).await? {
                                break;
                            }
                        },
                        AppState::Settings => {
                            handle_settings_input(&mut app_data, key);
                        },
                    }
                }
            }
        }
    }

    disable_raw_mode().map_err(full_crisis::err::eloc!())?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).map_err(full_crisis::err::eloc!())?;
    terminal.show_cursor().map_err(full_crisis::err::eloc!())?;

    Ok(())
}

fn draw_main_menu(f: &mut ratatui::Frame, app_data: &mut AppData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),     // Menu
            Constraint::Length(3),  // Instructions
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("Full Crisis - Main Menu")
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Menu
    let menu_items: Vec<ListItem> = app_data
        .main_menu_items
        .iter()
        .map(|choice| {
            ListItem::new(format!("{}", choice))
        })
        .collect();

    let menu = List::new(menu_items)
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
        .highlight_symbol("► ");
    
    f.render_stateful_widget(menu, chunks[1], &mut app_data.main_menu_state);

    // Instructions
    let instructions = Paragraph::new("↑/↓: Navigate, Enter: Select, Esc: Quit")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[2]);
}

fn draw_settings(f: &mut ratatui::Frame, app_data: &mut AppData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Settings grid
            Constraint::Length(3), // Instructions
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("Settings")
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Settings grid - divide into 2x2 grid
    let settings_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Top row (Difficulty, Language)
            Constraint::Length(8),  // Bottom row (Autosave, Crises Folder)
        ])
        .split(chunks[1]);

    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(settings_area[0]);

    let bottom_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(settings_area[1]);

    // Determine which section is currently selected
    let difficulty_selected = app_data.settings_section == SettingsSection::Difficulty;
    let language_selected = app_data.settings_section == SettingsSection::Language;
    let autosave_selected = app_data.settings_section == SettingsSection::Autosave;
    let folder_selected = app_data.settings_section == SettingsSection::CrisesFolder;

    // Difficulty setting
    let difficulty_items: Vec<ListItem> = app_data
        .difficulty_items
        .iter()
        .map(|&difficulty| {
            ListItem::new(format!("{}", difficulty))
        })
        .collect();

    let difficulty_style = if difficulty_selected {
        Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let difficulty_menu = List::new(difficulty_items)
        .block(Block::default().borders(Borders::ALL).title("Difficulty"))
        .style(Style::default().fg(Color::White))
        .highlight_style(difficulty_style)
        .highlight_symbol("► ");
    f.render_stateful_widget(difficulty_menu, top_row[0], &mut app_data.difficulty_state);

    // Language setting  
    let language_items: Vec<ListItem> = app_data.language_items
        .iter()
        .map(|code| {
            let display_name = full_crisis::language::get_language_display_name(code);
            ListItem::new(format!("{} ({})", display_name, code))
        })
        .collect();

    let language_style = if language_selected {
        Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let language_menu = List::new(language_items)
        .block(Block::default().borders(Borders::ALL).title("Language"))
        .style(Style::default().fg(Color::White))
        .highlight_style(language_style)
        .highlight_symbol("► ");
    f.render_stateful_widget(language_menu, top_row[1], &mut app_data.language_state);

    // Autosave setting
    let autosave_items: Vec<ListItem> = app_data.autosave_items
        .iter()
        .map(|&enabled| {
            let text = if enabled { "Enabled" } else { "Disabled" };
            ListItem::new(text)
        })
        .collect();

    let autosave_style = if autosave_selected {
        Style::default().fg(Color::Black).bg(Color::Magenta).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let autosave_menu = List::new(autosave_items)
        .block(Block::default().borders(Borders::ALL).title("Autosave"))
        .style(Style::default().fg(Color::White))
        .highlight_style(autosave_style)
        .highlight_symbol("► ");
    f.render_stateful_widget(autosave_menu, bottom_row[0], &mut app_data.autosave_state);

    // Crises folder text input
    let folder_style = if folder_selected {
        if app_data.editing_crises_folder {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        }
    } else {
        Style::default().fg(Color::White)
    };
    
    let crises_folder = Paragraph::new(app_data.crises_folder_input.as_str())
        .style(folder_style)
        .block(Block::default().borders(Borders::ALL).title("Crises Folder"))
        .wrap(Wrap { trim: true });
    f.render_widget(crises_folder, bottom_row[1]);

    // Instructions
    let instructions = Paragraph::new("Tab/Arrow: Navigate, Enter: Select/Edit, Esc: Back to Main Menu")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[2]);
}

async fn handle_main_menu_input(
    app_data: &mut AppData,
    key: KeyEvent,
    game: &full_crisis::game::GameState,
) -> Result<bool, full_crisis::err::BoxError> {
    match key.code {
        KeyCode::Up => {
            let i = match app_data.main_menu_state.selected() {
                Some(i) => {
                    if i == 0 {
                        app_data.main_menu_items.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            app_data.main_menu_state.select(Some(i));
        }
        KeyCode::Down => {
            let i = match app_data.main_menu_state.selected() {
                Some(i) => {
                    if i >= app_data.main_menu_items.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            app_data.main_menu_state.select(Some(i));
        }
        KeyCode::Enter => {
            if let Some(selected) = app_data.main_menu_state.selected() {
                match app_data.main_menu_items[selected] {
                    MainMenuChoice::Continue => {
                        if let Ok(mut evt_loop_wguard) = game.active_event_loop.write() {
                            *evt_loop_wguard = full_crisis::game::ActiveEventLoop::WelcomeScreen(
                                full_crisis::game::WelcomeScreenView::ContinueGame
                            );
                        }
                    }
                    MainMenuChoice::New => {
                        if let Ok(mut evt_loop_wguard) = game.active_event_loop.write() {
                            *evt_loop_wguard = full_crisis::game::ActiveEventLoop::WelcomeScreen(
                                full_crisis::game::WelcomeScreenView::NewGame
                            );
                        }
                    }
                    MainMenuChoice::Settings => {
                        app_data.state = AppState::Settings;
                    }
                    MainMenuChoice::Licenses => {
                        if let Ok(mut evt_loop_wguard) = game.active_event_loop.write() {
                            *evt_loop_wguard = full_crisis::game::ActiveEventLoop::WelcomeScreen(
                                full_crisis::game::WelcomeScreenView::Licenses
                            );
                        }
                    }
                    MainMenuChoice::Quit => {
                        return Ok(true);
                    }
                }
            }
        }
        KeyCode::Esc => {
            return Ok(true);
        }
        _ => {}
    }
    Ok(false)
}

fn handle_settings_input(app_data: &mut AppData, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app_data.editing_crises_folder = false;
            app_data.state = AppState::MainMenu;
        }
        KeyCode::Tab => {
            app_data.editing_crises_folder = false;
            app_data.settings_section = match app_data.settings_section {
                SettingsSection::Difficulty => SettingsSection::Language,
                SettingsSection::Language => SettingsSection::Autosave,
                SettingsSection::Autosave => SettingsSection::CrisesFolder,
                SettingsSection::CrisesFolder => SettingsSection::Difficulty,
            };
        }
        KeyCode::Left => {
            app_data.editing_crises_folder = false;
            app_data.settings_section = match app_data.settings_section {
                SettingsSection::Language | SettingsSection::CrisesFolder => SettingsSection::Difficulty,
                SettingsSection::Difficulty => SettingsSection::Language,
                SettingsSection::Autosave => SettingsSection::CrisesFolder,
            };
        }
        KeyCode::Right => {
            app_data.editing_crises_folder = false;
            app_data.settings_section = match app_data.settings_section {
                SettingsSection::Difficulty | SettingsSection::Autosave => SettingsSection::Language,
                SettingsSection::Language => SettingsSection::Difficulty,
                SettingsSection::CrisesFolder => SettingsSection::Autosave,
            };
        }
        KeyCode::Up => {
            app_data.editing_crises_folder = false;
            app_data.settings_section = match app_data.settings_section {
                SettingsSection::Autosave | SettingsSection::CrisesFolder => SettingsSection::Difficulty,
                SettingsSection::Difficulty => SettingsSection::Autosave,
                SettingsSection::Language => SettingsSection::CrisesFolder,
            };
        }
        KeyCode::Down => {
            app_data.editing_crises_folder = false;
            app_data.settings_section = match app_data.settings_section {
                SettingsSection::Difficulty | SettingsSection::Language => SettingsSection::Autosave,
                SettingsSection::Autosave => SettingsSection::Difficulty,
                SettingsSection::CrisesFolder => SettingsSection::Language,
            };
        }
        _ => {
            if app_data.editing_crises_folder {
                handle_crises_folder_input(app_data, key);
            } else {
                handle_menu_selection(app_data, key);
            }
        }
    }
}

fn handle_crises_folder_input(app_data: &mut AppData, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => {
            app_data.crises_folder_input.push(c);
            app_data.settings.game_crises_folder = app_data.crises_folder_input.clone();
            save_settings(&app_data.settings);
        }
        KeyCode::Backspace => {
            app_data.crises_folder_input.pop();
            app_data.settings.game_crises_folder = app_data.crises_folder_input.clone();
            save_settings(&app_data.settings);
        }
        KeyCode::Enter => {
            app_data.editing_crises_folder = false;
        }
        _ => {}
    }
}

fn handle_menu_selection(app_data: &mut AppData, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            match app_data.settings_section {
                SettingsSection::Difficulty => {
                    if let Some(idx) = app_data.difficulty_state.selected() {
                        app_data.settings.difficulty_level = app_data.difficulty_items[idx];
                        save_settings(&app_data.settings);
                    }
                }
                SettingsSection::Language => {
                    if let Some(idx) = app_data.language_state.selected() {
                        app_data.settings.language = app_data.language_items[idx].clone();
                        save_settings(&app_data.settings);
                    }
                }
                SettingsSection::Autosave => {
                    if let Some(idx) = app_data.autosave_state.selected() {
                        app_data.settings.autosave = app_data.autosave_items[idx];
                        save_settings(&app_data.settings);
                    }
                }
                SettingsSection::CrisesFolder => {
                    app_data.editing_crises_folder = true;
                }
            }
        }
        KeyCode::Up => {
            handle_list_navigation_up(app_data);
        }
        KeyCode::Down => {
            handle_list_navigation_down(app_data);
        }
        _ => {}
    }
}

fn handle_list_navigation_up(app_data: &mut AppData) {
    match app_data.settings_section {
        SettingsSection::Difficulty => {
            let i = match app_data.difficulty_state.selected() {
                Some(i) => {
                    if i == 0 {
                        app_data.difficulty_items.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            app_data.difficulty_state.select(Some(i));
        }
        SettingsSection::Language => {
            let i = match app_data.language_state.selected() {
                Some(i) => {
                    if i == 0 {
                        app_data.language_items.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            app_data.language_state.select(Some(i));
        }
        SettingsSection::Autosave => {
            let i = match app_data.autosave_state.selected() {
                Some(i) => {
                    if i == 0 {
                        app_data.autosave_items.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            app_data.autosave_state.select(Some(i));
        }
        SettingsSection::CrisesFolder => {
            // No navigation for text input
        }
    }
}

fn handle_list_navigation_down(app_data: &mut AppData) {
    match app_data.settings_section {
        SettingsSection::Difficulty => {
            let i = match app_data.difficulty_state.selected() {
                Some(i) => {
                    if i >= app_data.difficulty_items.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            app_data.difficulty_state.select(Some(i));
        }
        SettingsSection::Language => {
            let i = match app_data.language_state.selected() {
                Some(i) => {
                    if i >= app_data.language_items.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            app_data.language_state.select(Some(i));
        }
        SettingsSection::Autosave => {
            let i = match app_data.autosave_state.selected() {
                Some(i) => {
                    if i >= app_data.autosave_items.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            app_data.autosave_state.select(Some(i));
        }
        SettingsSection::CrisesFolder => {
            // No navigation for text input
        }
    }
}

fn save_settings(settings: &GameSettings) {
    if let Ok(serialized) = serde_json::to_string(&settings) {
        full_crisis::storage::set_attr("game_settings", &serialized);
    }
}