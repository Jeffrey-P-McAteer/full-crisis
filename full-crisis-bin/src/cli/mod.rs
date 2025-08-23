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
use tui_menu::{Menu, MenuState, MenuItem, MenuEvent};
use full_crisis::gui::types::{DifficultyLevel, GameSettings};

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppState {
    MainMenu,
    Settings,
}

#[derive(Debug, Clone)]
enum SettingsAction {
    DifficultyEasy,
    DifficultyMedium,
    DifficultyHard,
    LanguageEnglish,
    LanguageSpanish,
    LanguageFrench,
    LanguageGerman,
    LanguageItalian,
    LanguagePortuguese,
    LanguageRussian,
    LanguageJapanese,
    LanguageKorean,
    LanguageChinese,
    AutosaveEnable,
    AutosaveDisable,
    EditCrisesFolder,
}

struct AppData {
    state: AppState,
    main_menu_state: ListState,
    main_menu_items: Vec<MainMenuChoice>,
    settings_menu_state: MenuState<SettingsAction>,
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

    let settings = full_crisis::gui::GameWindow::load_settings();
    
    // Create settings menu structure
    let settings_menu_items = vec![
        MenuItem::group(
            "Difficulty",
            vec![
                MenuItem::item("Easy", SettingsAction::DifficultyEasy),
                MenuItem::item("Medium", SettingsAction::DifficultyMedium),
                MenuItem::item("Hard", SettingsAction::DifficultyHard),
            ],
        ),
        MenuItem::group(
            "Language",
            vec![
                MenuItem::item("English", SettingsAction::LanguageEnglish),
                MenuItem::item("Español", SettingsAction::LanguageSpanish),
                MenuItem::item("Français", SettingsAction::LanguageFrench),
                MenuItem::item("Deutsch", SettingsAction::LanguageGerman),
                MenuItem::item("Italiano", SettingsAction::LanguageItalian),
                MenuItem::item("Português", SettingsAction::LanguagePortuguese),
                MenuItem::item("Русский", SettingsAction::LanguageRussian),
                MenuItem::item("日本語", SettingsAction::LanguageJapanese),
                MenuItem::item("한국어", SettingsAction::LanguageKorean),
                MenuItem::item("中文", SettingsAction::LanguageChinese),
            ],
        ),
        MenuItem::group(
            "Autosave",
            vec![
                MenuItem::item("Enable", SettingsAction::AutosaveEnable),
                MenuItem::item("Disable", SettingsAction::AutosaveDisable),
            ],
        ),
        MenuItem::item("Edit Crises Folder", SettingsAction::EditCrisesFolder),
    ];
    
    let crises_folder = settings.game_crises_folder.clone();
    let mut app_data = AppData {
        state: AppState::MainMenu,
        main_menu_state: ListState::default(),
        main_menu_items,
        settings_menu_state: MenuState::new(settings_menu_items),
        settings,
        crises_folder_input: crises_folder,
        editing_crises_folder: false,
    };

    // Set initial selection
    app_data.main_menu_state.select(Some(0));

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
        .split(f.area());

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
    if app_data.editing_crises_folder {
        draw_crises_folder_editor(f, app_data);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Settings menu
            Constraint::Length(3), // Instructions
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Settings")
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Settings menu
    let menu = Menu::new();
    f.render_stateful_widget(menu, chunks[1], &mut app_data.settings_menu_state);

    // Instructions
    let instructions = Paragraph::new("↑/↓: Navigate, Enter: Select, Esc: Back to Main Menu")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[2]);
}

fn draw_crises_folder_editor(f: &mut ratatui::Frame, app_data: &mut AppData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(5), // Input field
            Constraint::Min(5),    // Spacer
            Constraint::Length(3), // Instructions
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Edit Crises Folder Path")
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Input field
    let input = Paragraph::new(app_data.crises_folder_input.as_str())
        .style(Style::default().fg(Color::Black).bg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Path"))
        .wrap(Wrap { trim: true });
    f.render_widget(input, chunks[1]);

    // Instructions
    let instructions = Paragraph::new("Type to edit, Enter: Save, Esc: Cancel")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[3]);
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
    if app_data.editing_crises_folder {
        handle_crises_folder_input(app_data, key);
        return;
    }

    match key.code {
        KeyCode::Esc => {
            app_data.state = AppState::MainMenu;
        }
        KeyCode::Up => {
            app_data.settings_menu_state.up();
        }
        KeyCode::Down => {
            app_data.settings_menu_state.down();
        }
        KeyCode::Left => {
            app_data.settings_menu_state.left();
        }
        KeyCode::Right => {
            app_data.settings_menu_state.right();
        }
        KeyCode::Enter => {
            // Process menu events
            for event in app_data.settings_menu_state.drain_events() {
                let MenuEvent::Selected(action) = event;
                handle_settings_action(&mut app_data.settings, action, &mut app_data.editing_crises_folder);
            }
        }
        _ => {}
    }
}

fn handle_crises_folder_input(app_data: &mut AppData, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => {
            app_data.crises_folder_input.push(c);
        }
        KeyCode::Backspace => {
            app_data.crises_folder_input.pop();
        }
        KeyCode::Enter => {
            // Save the changes
            app_data.settings.game_crises_folder = app_data.crises_folder_input.clone();
            save_settings(&app_data.settings);
            app_data.editing_crises_folder = false;
        }
        KeyCode::Esc => {
            // Cancel changes - revert to original value
            app_data.crises_folder_input = app_data.settings.game_crises_folder.clone();
            app_data.editing_crises_folder = false;
        }
        _ => {}
    }
}

fn handle_settings_action(settings: &mut GameSettings, action: SettingsAction, editing_crises_folder: &mut bool) {
    match action {
        SettingsAction::DifficultyEasy => {
            settings.difficulty_level = DifficultyLevel::Easy;
            save_settings(settings);
        }
        SettingsAction::DifficultyMedium => {
            settings.difficulty_level = DifficultyLevel::Medium;
            save_settings(settings);
        }
        SettingsAction::DifficultyHard => {
            settings.difficulty_level = DifficultyLevel::Hard;
            save_settings(settings);
        }
        SettingsAction::LanguageEnglish => {
            settings.language = "eng".to_string();
            save_settings(settings);
        }
        SettingsAction::LanguageSpanish => {
            settings.language = "spa".to_string();
            save_settings(settings);
        }
        SettingsAction::LanguageFrench => {
            settings.language = "fra".to_string();
            save_settings(settings);
        }
        SettingsAction::LanguageGerman => {
            settings.language = "deu".to_string();
            save_settings(settings);
        }
        SettingsAction::LanguageItalian => {
            settings.language = "ita".to_string();
            save_settings(settings);
        }
        SettingsAction::LanguagePortuguese => {
            settings.language = "por".to_string();
            save_settings(settings);
        }
        SettingsAction::LanguageRussian => {
            settings.language = "rus".to_string();
            save_settings(settings);
        }
        SettingsAction::LanguageJapanese => {
            settings.language = "jpn".to_string();
            save_settings(settings);
        }
        SettingsAction::LanguageKorean => {
            settings.language = "kor".to_string();
            save_settings(settings);
        }
        SettingsAction::LanguageChinese => {
            settings.language = "zho".to_string();
            save_settings(settings);
        }
        SettingsAction::AutosaveEnable => {
            settings.autosave = true;
            save_settings(settings);
        }
        SettingsAction::AutosaveDisable => {
            settings.autosave = false;
            save_settings(settings);
        }
        SettingsAction::EditCrisesFolder => {
            *editing_crises_folder = true;
        }
    }
}

fn save_settings(settings: &GameSettings) {
    if let Ok(serialized) = serde_json::to_string(&settings) {
        full_crisis::storage::set_attr("game_settings", &serialized);
    }
}