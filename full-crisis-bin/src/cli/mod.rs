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
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io;
use tui_menu::{Menu, MenuState, MenuItem, MenuEvent};
use full_crisis::gui::types::{DifficultyLevel, GameSettings};

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppState {
    MainMenu,
    Settings,
    SettingsPopup,
    TextInput,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SelectedSetting {
    Difficulty,
    Language,
    Autosave,
    CrisesFolder,
}


struct AppData {
    state: AppState,
    main_menu_state: ListState,
    main_menu_items: Vec<MainMenuChoice>,
    
    // Settings grid navigation
    selected_setting: SelectedSetting,
    settings_grid_selection: (usize, usize), // (row, col)
    
    // Popup menus for each setting type
    difficulty_menu_state: MenuState<DifficultyLevel>,
    language_menu_state: MenuState<String>,
    autosave_menu_state: MenuState<bool>,
    
    settings: GameSettings,
    crises_folder_input: String,
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
    
    // Create popup menu items for each setting type
    let difficulty_items = vec![
        MenuItem::item("Easy", DifficultyLevel::Easy),
        MenuItem::item("Medium", DifficultyLevel::Medium),
        MenuItem::item("Hard", DifficultyLevel::Hard),
    ];
    
    let language_items: Vec<MenuItem<String>> = full_crisis::language::get_available_languages()
        .into_iter()
        .map(|(code, display_name)| MenuItem::item(display_name.clone(), code))
        .collect();
    
    let autosave_items = vec![
        MenuItem::item("Enabled", true),
        MenuItem::item("Disabled", false),
    ];
    
    let crises_folder = settings.game_crises_folder.clone();
    let mut app_data = AppData {
        state: AppState::MainMenu,
        main_menu_state: ListState::default(),
        main_menu_items,
        
        selected_setting: SelectedSetting::Difficulty,
        settings_grid_selection: (0, 0),
        
        difficulty_menu_state: MenuState::new(difficulty_items),
        language_menu_state: MenuState::new(language_items),
        autosave_menu_state: MenuState::new(autosave_items),
        
        settings,
        crises_folder_input: crises_folder,
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
                AppState::Settings | AppState::SettingsPopup | AppState::TextInput => draw_settings(f, &mut app_data),
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
                        AppState::Settings | AppState::SettingsPopup | AppState::TextInput => {
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
    match app_data.state {
        AppState::Settings => draw_settings_grid(f, app_data),
        AppState::SettingsPopup => draw_settings_with_popup(f, app_data),
        AppState::TextInput => draw_crises_folder_editor(f, app_data),
        _ => {} // Should not happen
    }
}

fn draw_settings_grid(f: &mut ratatui::Frame, app_data: &AppData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Settings grid
            Constraint::Length(3),  // Instructions
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Settings")
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Settings grid (2x2)
    let grid_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),  // Top row (Difficulty, Language)
            Constraint::Length(6),  // Bottom row (Autosave, Crises Folder)
        ])
        .split(chunks[1]);

    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(grid_area[0]);

    let bottom_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(grid_area[1]);

    // Draw each setting cell
    draw_setting_cell(f, top_row[0], "Difficulty", &format!("{}", app_data.settings.difficulty_level), 
                      app_data.settings_grid_selection == (0, 0));
    
    let language_display = full_crisis::language::get_language_display_name(&app_data.settings.language);
    draw_setting_cell(f, top_row[1], "Language", &language_display, 
                      app_data.settings_grid_selection == (0, 1));
    
    let autosave_display = if app_data.settings.autosave { "Enabled" } else { "Disabled" };
    draw_setting_cell(f, bottom_row[0], "Autosave", autosave_display, 
                      app_data.settings_grid_selection == (1, 0));
    
    draw_setting_cell(f, bottom_row[1], "Crises Folder", &app_data.settings.game_crises_folder, 
                      app_data.settings_grid_selection == (1, 1));

    // Instructions
    let instructions = Paragraph::new("Arrow Keys: Navigate, Enter: Edit Setting, Esc: Back to Main Menu")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[2]);
}

fn draw_setting_cell(f: &mut ratatui::Frame, area: ratatui::layout::Rect, title: &str, value: &str, selected: bool) {
    let style = if selected {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let content = format!("{}\n\n{}", title, value);
    let cell = Paragraph::new(content)
        .style(style)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    
    f.render_widget(cell, area);
}

fn draw_settings_with_popup(f: &mut ratatui::Frame, app_data: &mut AppData) {
    // First draw the background settings grid (dimmed)
    draw_settings_grid(f, app_data);
    
    // Calculate popup area (centered, smaller than screen)
    let popup_area = centered_rect(60, 50, f.area());
    
    // Clear the popup area
    f.render_widget(Clear, popup_area);
    
    // Draw the popup menu based on selected setting
    match app_data.selected_setting {
        SelectedSetting::Difficulty => {
            let menu = Menu::new();
            let popup_block = Block::default()
                .borders(Borders::ALL)
                .title("Select Difficulty")
                .style(Style::default().bg(Color::Black).fg(Color::White));
            
            let inner_area = popup_block.inner(popup_area);
            f.render_widget(popup_block, popup_area);
            f.render_stateful_widget(menu, inner_area, &mut app_data.difficulty_menu_state);
        }
        SelectedSetting::Language => {
            let menu = Menu::new();
            let popup_block = Block::default()
                .borders(Borders::ALL)
                .title("Select Language")
                .style(Style::default().bg(Color::Black).fg(Color::White));
            
            let inner_area = popup_block.inner(popup_area);
            f.render_widget(popup_block, popup_area);
            f.render_stateful_widget(menu, inner_area, &mut app_data.language_menu_state);
        }
        SelectedSetting::Autosave => {
            let menu = Menu::new();
            let popup_block = Block::default()
                .borders(Borders::ALL)
                .title("Select Autosave")
                .style(Style::default().bg(Color::Black).fg(Color::White));
            
            let inner_area = popup_block.inner(popup_area);
            f.render_widget(popup_block, popup_area);
            f.render_stateful_widget(menu, inner_area, &mut app_data.autosave_menu_state);
        }
        SelectedSetting::CrisesFolder => {
            // This should transition to TextInput state instead
        }
    }
    
    // Popup instructions
    let instructions_area = ratatui::layout::Rect {
        x: popup_area.x,
        y: popup_area.y + popup_area.height + 1,
        width: popup_area.width,
        height: 1,
    };
    
    if instructions_area.y < f.area().height {
        let instructions = Paragraph::new("↑/↓: Navigate, Enter: Select, Esc: Cancel")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        f.render_widget(instructions, instructions_area);
    }
}

// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
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
    match app_data.state {
        AppState::Settings => handle_settings_grid_input(app_data, key),
        AppState::SettingsPopup => handle_settings_popup_input(app_data, key),
        AppState::TextInput => handle_crises_folder_input(app_data, key),
        _ => {} // Should not happen
    }
}

fn handle_settings_grid_input(app_data: &mut AppData, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app_data.state = AppState::MainMenu;
        }
        KeyCode::Up => {
            let (row, col) = app_data.settings_grid_selection;
            let new_row = if row == 0 { 1 } else { 0 };
            app_data.settings_grid_selection = (new_row, col);
        }
        KeyCode::Down => {
            let (row, col) = app_data.settings_grid_selection;
            let new_row = if row == 1 { 0 } else { 1 };
            app_data.settings_grid_selection = (new_row, col);
        }
        KeyCode::Left => {
            let (row, col) = app_data.settings_grid_selection;
            let new_col = if col == 0 { 1 } else { 0 };
            app_data.settings_grid_selection = (row, new_col);
        }
        KeyCode::Right => {
            let (row, col) = app_data.settings_grid_selection;
            let new_col = if col == 1 { 0 } else { 1 };
            app_data.settings_grid_selection = (row, new_col);
        }
        KeyCode::Enter => {
            // Determine which setting is selected and show appropriate popup
            match app_data.settings_grid_selection {
                (0, 0) => {
                    app_data.selected_setting = SelectedSetting::Difficulty;
                    app_data.state = AppState::SettingsPopup;
                }
                (0, 1) => {
                    app_data.selected_setting = SelectedSetting::Language;
                    app_data.state = AppState::SettingsPopup;
                }
                (1, 0) => {
                    app_data.selected_setting = SelectedSetting::Autosave;
                    app_data.state = AppState::SettingsPopup;
                }
                (1, 1) => {
                    app_data.selected_setting = SelectedSetting::CrisesFolder;
                    app_data.state = AppState::TextInput;
                }
                _ => {} // Invalid selection
            }
        }
        _ => {}
    }
}

fn handle_settings_popup_input(app_data: &mut AppData, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app_data.state = AppState::Settings;
        }
        KeyCode::Up => {
            match app_data.selected_setting {
                SelectedSetting::Difficulty => app_data.difficulty_menu_state.up(),
                SelectedSetting::Language => app_data.language_menu_state.up(),
                SelectedSetting::Autosave => app_data.autosave_menu_state.up(),
                _ => {}
            }
        }
        KeyCode::Down => {
            match app_data.selected_setting {
                SelectedSetting::Difficulty => app_data.difficulty_menu_state.down(),
                SelectedSetting::Language => app_data.language_menu_state.down(),
                SelectedSetting::Autosave => app_data.autosave_menu_state.down(),
                _ => {}
            }
        }
        KeyCode::Enter => {
            // Process menu selection based on selected setting
            match app_data.selected_setting {
                SelectedSetting::Difficulty => {
                    for event in app_data.difficulty_menu_state.drain_events() {
                        let MenuEvent::Selected(difficulty) = event;
                        app_data.settings.difficulty_level = difficulty;
                        save_settings(&app_data.settings);
                        app_data.state = AppState::Settings;
                    }
                }
                SelectedSetting::Language => {
                    for event in app_data.language_menu_state.drain_events() {
                        let MenuEvent::Selected(language) = event;
                        app_data.settings.language = language;
                        save_settings(&app_data.settings);
                        app_data.state = AppState::Settings;
                    }
                }
                SelectedSetting::Autosave => {
                    for event in app_data.autosave_menu_state.drain_events() {
                        let MenuEvent::Selected(autosave) = event;
                        app_data.settings.autosave = autosave;
                        save_settings(&app_data.settings);
                        app_data.state = AppState::Settings;
                    }
                }
                _ => {}
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
            app_data.state = AppState::Settings;
        }
        KeyCode::Esc => {
            // Cancel changes - revert to original value
            app_data.crises_folder_input = app_data.settings.game_crises_folder.clone();
            app_data.state = AppState::Settings;
        }
        _ => {}
    }
}


fn save_settings(settings: &GameSettings) {
    if let Ok(serialized) = serde_json::to_string(&settings) {
        full_crisis::storage::set_attr("game_settings", &serialized);
    }
}