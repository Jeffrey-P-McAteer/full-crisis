
#![allow(unreachable_patterns, non_camel_case_types)]

pub mod types;
pub mod settings;
pub mod ui_builders;
use types::*;

// Re-export key types for public use
pub use types::{GameWindow, GameMessage, DifficultyLevel, GameSettings};

use iced::widget::Space;
use iced::widget::Column;
use iced::widget::Row;
use iced::widget::{
    self, Container, Image, button, center_x, column, container, horizontal_space,
    pick_list, row, text, toggler, text_input,
};
use iced::{Center, Element, Length, Task, Theme};
// Immutable global data
const SPLASH_PNG_BYTES: &[u8] = include_bytes!("../../../icon/full-crisis-splash.transparent.png");





impl GameWindow {

    pub fn make_app_settings() -> iced::Settings {
        iced::Settings {
            ..Default::default()
        }
    }
    pub fn make_window_settings() -> iced::window::Settings {
        iced::window::Settings {
            resizable: true,
            decorations: true, // TODO other way
            fullscreen: true,
            ..Default::default()
        }
    }
    pub fn new() -> (Self, Task<GameMessage>) {
        let loaded_settings = Self::load_settings();
        (

            Self {
                os_theme: crate::OS_COLOR_THEME.get().unwrap_or(&crate::game::OSColorTheme::Light).clone(),
                game_state: crate::game::GameState::new(),
                new_game_player_name: String::new(),
                new_game_game_template: None,
                continue_game_game_choice: None,
                settings_game_save_folder: loaded_settings.game_save_folder,
                settings_difficulty_level: loaded_settings.difficulty_level,
                settings_autosave: loaded_settings.autosave,
                settings_language: loaded_settings.language,
                current_crisis: None,
                story_state: None,
            },
            Task::batch([
                widget::focus_next(),
            ]),
        )
    }

    pub fn update(&mut self, message: GameMessage) -> Task<GameMessage> {
        match message {
            GameMessage::Menu_NewGameRequested => {
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::NewGame);
                }
                Task::none()
            }
            GameMessage::Menu_NewGamePlayerNameAltered(name) => {
                self.new_game_player_name = name;
                Task::none()
            }
            GameMessage::Menu_NewGameTemplateChoiceAltered(game_template) => {
                self.new_game_game_template = Some(game_template);
                Task::none()
            }
            GameMessage::Menu_NewGameStartClicked => {
                let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
                if *verbosity > 0 {
                    eprintln!("[VERBOSE] Menu_NewGameStartClicked: template_name={:?}", self.new_game_game_template);
                }
                
                if let Some(ref template_name) = self.new_game_game_template {
                    match crate::crisis::load_crisis(template_name) {
                        Ok(crisis) => {
                            if *verbosity > 0 {
                                eprintln!("[VERBOSE] Menu_NewGameStartClicked: Crisis loaded successfully");
                            }
                            
                            let character_name = if self.new_game_player_name.is_empty() {
                                crate::crisis::get_random_character_name(&crisis, None, &self.settings_language)
                            } else {
                                self.new_game_player_name.clone()
                            };
                            
                            if *verbosity > 0 {
                                eprintln!("[VERBOSE] Menu_NewGameStartClicked: Character name: {}", character_name);
                            }
                            
                            let user_language = self.settings_language.clone();
                            let mut story_state = crate::crisis::GameState::new(
                                crisis.metadata.id.clone(),
                                user_language.clone(),
                            );
                            story_state.current_scene = crisis.story.starting_scene.clone();
                            story_state.character_name = character_name;
                            
                            if *verbosity > 0 {
                                eprintln!("[VERBOSE] Menu_NewGameStartClicked: Starting scene: {}", story_state.current_scene);
                            }
                            
                            self.current_crisis = Some(crisis);
                            self.story_state = Some(story_state);
                            
                            if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                                *evt_loop_wguard = crate::game::ActiveEventLoop::ActiveGame(crate::game::GameView::StoryScene);
                                if *verbosity > 0 {
                                    eprintln!("[VERBOSE] Menu_NewGameStartClicked: Successfully switched to ActiveGame state");
                                }
                            } else {
                                if *verbosity > 0 {
                                    eprintln!("[VERBOSE] Menu_NewGameStartClicked: Failed to acquire event loop write lock");
                                }
                            }
                        }
                        Err(e) => {
                            if *verbosity > 0 {
                                eprintln!("[VERBOSE] Menu_NewGameStartClicked: Failed to load crisis: {}", e);
                            }
                        }
                    }
                } else {
                    if *verbosity > 0 {
                        eprintln!("[VERBOSE] Menu_NewGameStartClicked: No template selected");
                    }
                }
                Task::none()
            }

            GameMessage::Menu_ContinueGameRequested => {
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::ContinueGame);
                }
                Task::none()
            }
            GameMessage::Menu_ContinueGameChoiceAltered(game_name) => {
                self.continue_game_game_choice = Some(game_name);
                Task::none()
            }

            GameMessage::Menu_SettingsRequested => {
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::Settings);
                }
                Task::none()
            }
            GameMessage::Menu_SettingsGameSaveFolderChanged(folder_path) => {
                eprintln!("Settings: Game Save Folder changed to: {}", folder_path);
                self.settings_game_save_folder = folder_path;
                self.save_settings();
                Task::none()
            }
            GameMessage::Menu_SettingsDifficultyLevelChanged(difficulty) => {
                eprintln!("Settings: Difficulty Level changed to: {:?}", difficulty);
                self.settings_difficulty_level = difficulty;
                self.save_settings();
                Task::none()
            }
            GameMessage::Menu_SettingsAutosaveToggled(enabled) => {
                eprintln!("Settings: Autosave toggled to: {}", enabled);
                self.settings_autosave = enabled;
                self.save_settings();
                Task::none()
            }
            GameMessage::Menu_SettingsLanguageChanged(language) => {
                eprintln!("Settings: Language changed to: {}", language);
                self.settings_language = language;
                self.save_settings();
                Task::none()
            }
            GameMessage::QuitGameRequested => {
                // TODO and game-save requirements should go here

                iced::window::get_latest().and_then(iced::window::close).chain(iced::exit())
                // ^ this exit assumes a single window exists, if we have 2+ we will need to iterate, close them all, and then call iced::exit()
            }
            GameMessage::Game_ChoiceSelected(choice_index) => {
                if let (Some(crisis), Some(story_state)) = (&self.current_crisis, &mut self.story_state) {
                    if let Some(current_scene) = crisis.scenes.get(&story_state.current_scene) {
                        if let Some(choice) = current_scene.choices.get(choice_index) {
                            // Apply choice effects
                            if let Some(ref choice_effects) = crisis.conditions.choice_effects {
                                if let Some(effects) = choice_effects.get(&choice.leads_to) {
                                    for (var, value) in effects {
                                        *story_state.variables.entry(var.clone()).or_insert(0) += value;
                                    }
                                }
                            }
                            
                            // Move to next scene
                            story_state.current_scene = choice.leads_to.clone();
                            
                            // Handle character type selection
                            if let Some(ref char_type) = choice.character_type {
                                story_state.character_type = Some(char_type.clone());
                                story_state.character_name = crate::crisis::get_random_character_name(
                                    crisis, 
                                    Some(char_type), 
                                    &story_state.language
                                );
                            }
                        }
                    }
                }
                Task::none()
            }
            GameMessage::Game_RestartRequested => {
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::Empty);
                }
                self.current_crisis = None;
                self.story_state = None;
                Task::none()
            }
            GameMessage::Nop => {
                eprintln!("Recieved a GameMessage::Nop");
                Task::none()
            },
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, GameMessage> {
        if let Ok(evt_loop_rguard) = self.game_state.active_event_loop.read() {
            match evt_loop_rguard.clone() {
                crate::game::ActiveEventLoop::WelcomeScreen(_welcome_screen_state) => {
                    self.view_menu_screen()
                }
                crate::game::ActiveEventLoop::ActiveGame(_game_view_state) => {
                    self.view_game_screen()
                }
                crate::game::ActiveEventLoop::Exit => {
                    text("Exiting...").into()
                }
            }
        }
        else {
            text("Error, cannot read game_state.active_event_loop").into()
        }
    }

    pub fn theme(&self) -> Theme {
        if self.os_theme == crate::game::OSColorTheme::Dark {
            Theme::Dark
        }
        else {
            Theme::Light
        }
    }

    pub fn view_menu_screen(&self) -> Element<'_, GameMessage> {
        let splash_handle = iced::widget::image::Handle::from_bytes(SPLASH_PNG_BYTES);
        let splash_img = Image::<iced::widget::image::Handle>::new(splash_handle)
            .width(Length::Fill)
            .height(Length::Fill);
        let background = Container::<GameMessage, Theme, iced::Renderer>::new(splash_img)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        let num_times_run = crate::storage::get_attr("run-times").unwrap_or_else(|| "First Run!".to_string());

        let user_language = &self.settings_language;
        let buttons = column![
            button(text(crate::translations::t(crate::translations::TranslationKey::ContinueGame, user_language)))
                .on_press(GameMessage::Menu_ContinueGameRequested)
                .width(Length::Fill),
            button(text(crate::translations::t(crate::translations::TranslationKey::NewGame, user_language)))
                .on_press(GameMessage::Menu_NewGameRequested)
                .width(Length::Fill),
            button(text(crate::translations::t(crate::translations::TranslationKey::Settings, user_language)))
                .on_press(GameMessage::Menu_SettingsRequested)
                .width(Length::Fill),
            button(text(crate::translations::t(crate::translations::TranslationKey::QuitGame, user_language)))
                .on_press(GameMessage::QuitGameRequested)
                .width(Length::Fill),
            text(format!("Run: {}", num_times_run))
                .width(Length::Fill),
        ]
        .spacing(10)
        .width(240)
        .padding(10)
        .align_x(Center)
        .width(Length::Fixed(186.0f32));

        let right_panel = container(self.build_menu_screen_right_ui())
            .width(Length::Fixed(680.0f32))
            .align_x(Center)
            .center_y(iced::Length::Shrink);


        let foreground_content = row![buttons, right_panel]
            .height(Length::Fill)
            .width(Length::Shrink);

        let foreground_content = Column::new()
            .height(Length::Fill)
            .width(Length::Shrink)
            .align_x(iced::alignment::Horizontal::Center)
            .push(Space::with_height(Length::Fill))
            .push(
                foreground_content
            )
            .push(Space::with_height(Length::Fill));

        let foreground_content = Row::new()
            .height(Length::Fill)
            .width(Length::Shrink)
            .align_y(iced::alignment::Vertical::Center)
            .push(Space::with_height(Length::Fill))
            .push(
                foreground_content
            )
            .push(Space::with_height(Length::Fill));


        iced::widget::stack![
            background,
            Container::new(foreground_content)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Center)
                .align_y(Center)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }





    pub fn view_game_screen(&self) -> Element<'_, GameMessage> {
        if let (Some(crisis), Some(story_state)) = (&self.current_crisis, &self.story_state) {
            self.render_story_scene(crisis, story_state)
        } else {
            container(
                column![
                    text(crate::translations::t(crate::translations::TranslationKey::LoadingCrisis, &self.settings_language)).size(20),
                    button(text(crate::translations::t(crate::translations::TranslationKey::ReturnToMenu, &self.settings_language)))
                        .on_press(GameMessage::Game_RestartRequested)
                        .padding(10)
                ]
                .spacing(20)
                .align_x(Center)
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }
    }
    
    fn render_story_scene(&self, crisis: &crate::crisis::CrisisDefinition, story_state: &crate::crisis::GameState) -> Element<'_, GameMessage> {
        if let Some(current_scene) = crisis.scenes.get(&story_state.current_scene) {
            // Create background image if specified
            let background_layer = if let Some(ref bg_path) = current_scene.background_image {
                if let Some(bg_file) = crate::crisis::PlayableCrises::get(bg_path) {
                    let bg_handle = iced::widget::image::Handle::from_bytes(bg_file.data.as_ref().to_vec());
                    let bg_img = Image::<iced::widget::image::Handle>::new(bg_handle)
                        .width(Length::Fill)
                        .height(Length::Fill);
                    Some(Container::<GameMessage, Theme, iced::Renderer>::new(bg_img)
                        .width(Length::Fill)
                        .height(Length::Fill))
                } else {
                    None
                }
            } else {
                None
            };

            // Game data at center-top
            let title = crate::crisis::get_localized_text(&crisis.name, &story_state.language);
            let mut vars = std::collections::HashMap::new();
            vars.insert("character_name".to_string(), story_state.character_name.clone());
            let character_info = text(
                crate::translations::t_vars(crate::translations::TranslationKey::PlayingAs, &story_state.language, &vars)
            )
                .size(16)
                .color(iced::Color::from_rgb(0.6, 0.6, 0.6));
            
            let mut variables_text = String::new();
            if !story_state.variables.is_empty() {
                variables_text = format!("Variables: {:?}", story_state.variables);
            }
            
            let top_data = container(
                column![
                    text(title.clone()).size(24).align_x(Center),
                    character_info.align_x(Center),
                    if !variables_text.is_empty() {
                        text(variables_text.clone()).size(12).color(iced::Color::from_rgb(0.5, 0.5, 0.5))
                    } else {
                        text("")
                    }
                ]
                .spacing(5)
                .align_x(Center)
            )
            .width(Length::Fill)
            .align_x(Center)
            .padding(20);

            // Story text in center
            let scene_text = crate::crisis::get_scene_text(current_scene, &story_state.language, &story_state.character_name);
            let story_text_display = container(
                container(
                    text(scene_text.clone())
                        .size(18)
                        .wrapping(iced::widget::text::Wrapping::Word)
                )
                .padding(20)
                .style(move |theme: &Theme| {
                    let palette = theme.extended_palette();
                    iced::widget::container::Style {
                        background: Some(palette.background.weak.color.into()),
                        border: iced::border::rounded(8)
                            .color(palette.primary.weak.color)
                            .width(1),
                        ..iced::widget::container::Style::default()
                    }
                })
            )
            .width(Length::Fill)
            .max_width(600)
            .align_x(Center)
            .padding(20);

            // User choices at lower-left
            let mut choices_column = column![].spacing(10);
            
            if current_scene.choices.is_empty() {
                choices_column = choices_column.push(
                    column![
                        text(crate::translations::t(crate::translations::TranslationKey::End, &story_state.language)).size(20),
                        button(text(crate::translations::t(crate::translations::TranslationKey::ReturnToMenu, &story_state.language)))
                            .on_press(GameMessage::Game_RestartRequested)
                            .padding(10)
                            .width(Length::Fixed(200.0))
                    ]
                    .spacing(10)
                );
            } else {
                choices_column = choices_column.push(
                    text(crate::translations::t(crate::translations::TranslationKey::WhatDoYouChoose, &story_state.language))
                        .size(16)
                );
                
                for (index, choice) in current_scene.choices.iter().enumerate() {
                    let choice_text = crate::crisis::get_localized_text(&choice.text, &story_state.language);
                    
                    let mut available = true;
                    if let Some(ref requirements) = choice.requires {
                        for (var, required_value) in requirements {
                            if let Some(current_value) = story_state.variables.get(var) {
                                if current_value < required_value {
                                    available = false;
                                    break;
                                }
                            } else {
                                available = false;
                                break;
                            }
                        }
                    }
                    
                    let choice_button = if available {
                        button(text(choice_text.clone()))
                            .on_press(GameMessage::Game_ChoiceSelected(index))
                            .padding(10)
                            .width(Length::Fixed(300.0))
                    } else {
                        button(text(format!("{} {}", choice_text, 
                            crate::translations::t(crate::translations::TranslationKey::RequirementsNotMet, &story_state.language))))
                            .padding(10)
                            .width(Length::Fixed(300.0))
                            .style(move |theme: &Theme, _status| {
                                let palette = theme.extended_palette();
                                iced::widget::button::Style {
                                    background: Some(palette.background.weak.color.into()),
                                    text_color: palette.background.strong.text,
                                    border: iced::border::rounded(4)
                                        .color(palette.background.strong.color)
                                        .width(1),
                                    ..iced::widget::button::Style::default()
                                }
                            })
                    };
                    
                    choices_column = choices_column.push(choice_button);
                }
            }
            
            let choices_container = container(choices_column)
                .width(Length::Fixed(320.0))
                .padding(20);

            // Speaking character at lower-right
            let character_display = if let Some(ref char_path) = current_scene.speaking_character_image {
                if let Some(char_file) = crate::crisis::PlayableCrises::get(char_path) {
                    let char_handle = iced::widget::image::Handle::from_bytes(char_file.data.as_ref().to_vec());
                    let char_img = Image::<iced::widget::image::Handle>::new(char_handle)
                        .width(Length::Fixed(200.0))
                        .height(Length::Fixed(300.0));
                    container(char_img)
                        .width(Length::Fixed(220.0))
                        .padding(20)
                } else {
                    container(Space::with_width(Length::Fixed(220.0)))
                }
            } else {
                container(Space::with_width(Length::Fixed(220.0)))
            };

            // Bottom row with choices on left and character on right
            let bottom_row = row![
                choices_container,
                horizontal_space(),
                character_display
            ]
            .align_y(iced::alignment::Vertical::Bottom)
            .width(Length::Fill);

            // Main layout: top data, center story, bottom choices+character
            let main_layout = column![
                top_data,
                container(story_text_display)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Center)
                    .align_y(iced::alignment::Vertical::Center),
                bottom_row
            ]
            .width(Length::Fill)
            .height(Length::Fill);

            // Stack background and foreground if background exists
            if let Some(background) = background_layer {
                iced::widget::stack![
                    background,
                    Container::new(main_layout)
                        .width(Length::Fill)
                        .height(Length::Fill)
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else {
                Container::new(main_layout)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        } else {
            container(
                column![
                    text(format!("{} '{}' {}", 
                        crate::translations::t(crate::translations::TranslationKey::SceneNotFound, &story_state.language).replace("!", ""),
                        story_state.current_scene,
                        "!"
                    )).size(20),
                    button(text(crate::translations::t(crate::translations::TranslationKey::ReturnToMenu, &story_state.language)))
                        .on_press(GameMessage::Game_RestartRequested)
                        .padding(10)
                ]
                .spacing(20)
                .align_x(Center)
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }
    }


}


pub fn menu_right_box_style(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();

    iced::widget::container::Style {
        background: Some(palette.background.weak.color.into()),
        border: iced::border::rounded(12)
            .color(palette.primary.weak.color)
            .width(2),
        ..iced::widget::container::Style::default()
    }
}




