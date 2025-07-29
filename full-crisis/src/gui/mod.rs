
#![allow(unreachable_patterns, non_camel_case_types)]

use iced::widget::Column;
use iced::advanced::Widget;

//use iced::highlighter;

use iced::keyboard;
use iced::widget::{
    self, Container, Image, button, center, center_x, column, container, horizontal_space,
    pick_list, row, text, text_editor, toggler, tooltip,
};
use iced::{Center, Element, Fill, Font, Left, Length, Right, Task, Theme};

use std::ffi;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
// Immutable global data
const SPLASH_PNG_BYTES: &[u8] = include_bytes!("../../../icon/full-crisis-splash.transparent.png");

pub struct GameWindow {
    pub game_state: crate::game::GameState,
}

#[derive(Debug, Clone)]
pub enum GameMessage {
    Nop,
    // view_menu_screen states
    Menu_NewGameRequested,
    Menu_ContinueGameRequested,
    Menu_SettingsRequested,
    QuitGameRequested,
    // view_game_screen states
}

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
        (
            Self {
                game_state: crate::game::GameState::new(),
                /*file: None,
                content: text_editor::Content::new(),
                theme: highlighter::Theme::SolarizedDark,
                word_wrap: true,
                is_loading: true,
                is_dirty: false,*/
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
            GameMessage::Menu_ContinueGameRequested => {
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::ContinueGame);
                }
                Task::none()
            }
            GameMessage::Menu_SettingsRequested => {
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::Settings);
                }
                Task::none()
            }
            GameMessage::QuitGameRequested => {
                // TODO and game-save requirements should go here

                iced::window::get_latest().and_then(iced::window::close).chain(iced::exit())
                // ^ this exit assumes a single window exists, if we have 2+ we will need to iterate, close them all, and then call iced::exit()
            }
            GameMessage::Nop => {
                eprintln!("Recieved a GameMessage::Nop");
                Task::none()
            },
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<GameMessage> {
        // TODO if/else for menu_screen and game_screen
        self.view_menu_screen()
    }

    pub fn theme(&self) -> Theme {
        /*if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }*/
        Theme::Light
    }

    pub fn view_menu_screen(&self) -> Element<GameMessage> {
        let splash_handle = iced::widget::image::Handle::from_bytes(SPLASH_PNG_BYTES);
        let splash_img = Image::<iced::widget::image::Handle>::new(splash_handle)
            .width(Length::Fill)
            .height(Length::Fill);
        let background = Container::<GameMessage, Theme, iced::Renderer>::new(splash_img)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        //let buttons: iced::widget::Column<'_, GameMessage, Theme, iced::Renderer> = column![
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
        ]
        .spacing(10)
        .width(240)
        .padding(10)
        .align_x(Center);

        // TODO swap out w/ button state
        /*let right_panel = container(text("Select an option"))
            .width(Length::Fill)
            .align_x(Center)
            .align_y(Center);*/
        let right_panel = container(self.build_menu_screen_right_ui())
            .width(Length::Fill)
            .align_x(Center)
            .align_y(Center);


        let foreground_content = row![buttons, right_panel]
            .height(Length::Fill)
            .width(Length::Fill);

        // UI consists of background image + foreground content
        Container::<GameMessage, Theme, iced::Renderer>::new(
            Column::new()
                .push(background)  // bottom layer
                .push(Container::new(foreground_content)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Center)
                        .align_y(Center)
                ), // top layer
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn build_menu_screen_right_ui(&self) -> Container<GameMessage> {
        if let Ok(evt_loop_val) = self.game_state.active_event_loop.try_read() {
            if let crate::game::ActiveEventLoop::WelcomeScreen(ref ws_area) = *evt_loop_val {
                match ws_area {
                    crate::game::WelcomeScreenView::NewGame => {
                        Container::<GameMessage, Theme, iced::Renderer>::new(text("TODO new game UI"))
                    }
                    crate::game::WelcomeScreenView::ContinueGame => {
                        Container::<GameMessage, Theme, iced::Renderer>::new(text("TODO continue UI"))
                    }
                    crate::game::WelcomeScreenView::Settings => {
                        Container::<GameMessage, Theme, iced::Renderer>::new(text("TODO settings UI"))
                    }
                    _ => {
                        Container::<GameMessage, Theme, iced::Renderer>::new(text("Select from left menu"))
                    }
                }
            }
            else {
                Container::<GameMessage, Theme, iced::Renderer>::new(text("Select from left menu"))
            }
        }
        else {
            Container::<GameMessage, Theme, iced::Renderer>::new(text("Select from left menu"))
        }
    }

    // pub fn build_continue_ui<'a>(&self) -> Container<'a, GameMessage> {
    //     Container::<GameMessage, Theme, iced::Renderer>::new(text("TODO continue UI"))
    // }

    // pub fn build_new_game_ui<'a>(&self) -> Container<'a, GameMessage> {
    //     Container::<GameMessage, Theme, iced::Renderer>::new(text("TODO new game UI"))
    // }

    // pub fn build_settings_ui<'a>(&self) -> Container<'a, GameMessage> {
    //     Container::<GameMessage, Theme, iced::Renderer>::new(text("TODO settings UI"))
    // }


}

async fn run_background_async_tasks() -> Result<(), crate::err::BoxError> {
    eprintln!("TODO run_background_async_tasks");

    Ok(())
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
