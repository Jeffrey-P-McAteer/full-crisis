
#![allow(unreachable_patterns, non_camel_case_types)]

pub mod types;
pub mod settings;
pub mod ui_builders;
use types::*;

// Re-export key types for public use
pub use types::{GameWindow, GameMessage, DifficultyLevel, GameSettings};

use iced::widget::Button;
use iced::widget::Space;
use iced::widget::Text;
use iced::widget::text::Alignment;
use iced::widget::Column;
use iced::widget::Row;
use iced::advanced::Widget;

//use iced::widget::row::Row as _;

//use iced::highlighter;

use iced::keyboard;
use iced::widget::{
    self, Container, Image, button, center, center_x, column, container, horizontal_space,
    pick_list, row, text, text_editor, toggler, tooltip, text_input,
};
use iced::{Center, Element, Fill, Font, Left, Length, Right, Task, Theme};

use std::ffi;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
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
                game_text_input: String::new(),
                player_cash: 1000,
                player_health: 100,
                player_popularity: 50,
            },
            Task::batch([
                /*Task::perform(
                    load_file(format!(
                        "{}/src/main.rs",
                        env!("CARGO_MANIFEST_DIR")
                    )),
                    Message::FileOpened,
                ),*/
                Task::perform(
                    run_background_async_tasks(),
                    |_| GameMessage::Nop, // _wierd_, why?
                ),
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
                // TODO read UI input and record for start of game

                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::ActiveGame(crate::game::GameView::FirstScene);
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
            GameMessage::QuitGameRequested => {
                // TODO and game-save requirements should go here

                iced::window::get_latest().and_then(iced::window::close).chain(iced::exit())
                // ^ this exit assumes a single window exists, if we have 2+ we will need to iterate, close them all, and then call iced::exit()
            }
            GameMessage::Game_TextInputChanged(input) => {
                self.game_text_input = input;
                Task::none()
            }
            GameMessage::Game_TextInputSubmitted => {
                eprintln!("Player submitted: {}", self.game_text_input);
                self.game_text_input.clear();
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

        let buttons = column![
            button(text("Continue Game"))
                .on_press(GameMessage::Menu_ContinueGameRequested)
                .width(Length::Fill),
            button(text("New Game"))
                .on_press(GameMessage::Menu_NewGameRequested)
                .width(Length::Fill),
            button(text("Settings"))
                .on_press(GameMessage::Menu_SettingsRequested)
                .width(Length::Fill),
            button(text("Quit Game"))
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

        // TODO swap out w/ button state
        /*let right_panel = container(text("Select an option"))
            .width(Length::Fill)
            .align_x(Center)
            .align_y(Center);*/
        let right_panel = container(self.build_menu_screen_right_ui())
            .width(Length::Fixed(680.0f32))
            .align_x(Center)
            //.align_y(Center)
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
        // Stats bar at the top
        let cash_stat: Row<GameMessage> = row![
            text("💰"), 
            text(format!("{}", self.player_cash))
        ].spacing(5).align_y(Center);

        let health_stat: Row<GameMessage> = row![
            text("❤️"), 
            text(format!("{}", self.player_health))
        ].spacing(5).align_y(Center);

        let popularity_stat: Row<GameMessage> = row![
            text("⭐"), 
            text(format!("{}", self.player_popularity))
        ].spacing(5).align_y(Center);

        let stats_bar: Row<GameMessage> = row![
            cash_stat,
            horizontal_space().width(30),
            health_stat,
            horizontal_space().width(30),
            popularity_stat,
        ]
        .padding(20)
        .align_y(Center)
        .width(Length::Fill);

        // Text input area on the left
        let text_input_area = text_input("Type your command...", &self.game_text_input)
            .on_input(GameMessage::Game_TextInputChanged)
            .on_submit(GameMessage::Game_TextInputSubmitted)
            .padding(10)
            .width(Length::Fill);

        let text_input_container = container(
            column![
                text("Command Input:").size(16),
                text_input_area
            ]
            .spacing(10)
        )
        .padding(20)
        .width(Length::FillPortion(2))
        .height(Length::Fill);

        // Character display on the right
        let character_display = container(
            column![
                text("Character").size(20).align_x(Center),
                container(text("🧙‍♂️").size(80))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),
                text("Ready for adventure!").align_x(Center)
            ]
            .spacing(20)
            .align_x(Center)
        )
        .padding(20)
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .center_x(Length::Fill);

        // Main game layout
        let game_content = column![
            container(stats_bar)
                .style(|theme| {
                    let palette = theme.extended_palette();
                    iced::widget::container::Style {
                        background: Some(palette.background.weak.color.into()),
                        border: iced::border::rounded(8)
                            .color(palette.primary.weak.color)
                            .width(1),
                        ..iced::widget::container::Style::default()
                    }
                }),
            row![
                text_input_container,
                character_display
            ]
            .height(Length::Fill)
        ]
        .height(Length::Fill)
        .width(Length::Fill);

        container(game_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }


}

async fn run_background_async_tasks() -> Result<(), crate::err::BoxError> {
    eprintln!("TODO run_background_async_tasks");

    Ok(())
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


/*
fn action<'a, GameMessage: Clone + 'a>(
    content: impl Into<Element<'a, GameMessage>>,
    label: &'a str,
    on_press: Option<GameMessage>,
) -> Element<'a, GameMessage> {
    let action = button(center_x(content).width(30));

    if let Some(on_press) = on_press {
        tooltip(
            action.on_press(on_press),
            label,
            tooltip::Position::FollowCursor,
        )
        .style(container::rounded_box)
        .into()
    } else {
        action.style(button::secondary).into()
    }
}
*/


