
#![allow(unreachable_patterns, non_camel_case_types)]

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
// Immutable global data
const SPLASH_PNG_BYTES: &[u8] = include_bytes!("../../../icon/full-crisis-splash.transparent.png");

pub struct GameWindow {
    // Settings grade data
    pub os_theme: crate::game::OSColorTheme,

    // Current UI location data, high-level
    pub game_state: crate::game::GameState,

    // Current UI location data, low-level
    pub new_game_player_name: String,
    pub new_game_game_template: Option<String>,

    pub continue_game_game_choice: Option<String>,
}

#[derive(Debug, Clone)]
pub enum GameMessage {
    Nop,

    // view_menu_screen states
    Menu_NewGameRequested,
        Menu_NewGamePlayerNameAltered(String),
        Menu_NewGameTemplateChoiceAltered(String),
        Menu_NewGameStartClicked,

    Menu_ContinueGameRequested,
        Menu_ContinueGameChoiceAltered(String),

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
                os_theme: crate::OS_COLOR_THEME.get().unwrap_or(&crate::game::OSColorTheme::Light).clone(),
                game_state: crate::game::GameState::new(),
                new_game_player_name: String::new(),
                new_game_game_template: None,
                continue_game_game_choice: None,
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
        if let Ok(evt_loop_rguard) = self.game_state.active_event_loop.read() {
            match evt_loop_rguard.clone() {
                crate::game::ActiveEventLoop::WelcomeScreen(_welcome_screen_state) => {
                    self.view_menu_screen()
                }
                crate::game::ActiveEventLoop::ActiveGame(game_view_state) => {
                    text(format!("TODO write UI for ActiveGame({:?})", game_view_state)).into()
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

    pub fn build_menu_screen_right_ui(&self) -> Container<GameMessage> {
        if let Ok(evt_loop_val) = self.game_state.active_event_loop.try_read() {
            if let crate::game::ActiveEventLoop::WelcomeScreen(ref ws_area) = *evt_loop_val {
                match ws_area {
                    crate::game::WelcomeScreenView::NewGame => {
                        self.build_new_game_ui()
                    }
                    crate::game::WelcomeScreenView::ContinueGame => {
                        self.build_continue_game_ui()
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

    pub fn build_continue_game_ui<'a>(&self) -> Container<'a, GameMessage> {
        // TODO replace w/ dynamic list of game names
        let saved_games = crate::crisis::get_saved_crisis_names();
        let game_type_picker = pick_list(
            saved_games,
            self.continue_game_game_choice.clone(),
            GameMessage::Menu_ContinueGameChoiceAltered,
        )
        .placeholder("Select game")
        .padding(10)
        .width(Length::Fill);

        let game_type_row = row![
            Text::new("Saved Game:"), game_type_picker,
        ]
            .spacing(10)
            .align_y(Center);

        // Go Button (aligned bottom-right)
        let go_button = button(Text::new("Play"))
            .on_press(GameMessage::Menu_NewGameStartClicked)
            .padding(10)
            ;//.style(theme::Button::Primary);

        let layout = Column::new()
            .spacing(20)
            .padding(20)
            .push(game_type_row)
            .push(
                Container::new(go_button)
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill),
            )
            .height(Length::Fill)
            .align_x(Left);

        let self_theme = self.theme();
        Container::<GameMessage, Theme, iced::Renderer>::new(layout)
            .width(Length::Fixed(600.0))
            .height(Length::Fixed(400.0))
            .style(move |_theme| menu_right_box_style(&self_theme))
            .padding(10)
    }

    pub fn build_new_game_ui<'a>(&self) -> Container<'a, GameMessage> {
        // Player Name Row
        let name_input = text_input("Enter name...", &self.new_game_player_name)
            .on_input(GameMessage::Menu_NewGamePlayerNameAltered)
            .padding(10)
            .width(Length::Fill);

        let name_row = row![
                Text::new("Player Name:"), name_input
            ]
            .spacing(10)
            .align_y(Center);

        // Game Type Row
        let crisis_names = crate::crisis::get_crisis_names();
        let game_type_picker = pick_list(
            crisis_names,
            self.new_game_game_template.clone(),
            GameMessage::Menu_NewGameTemplateChoiceAltered,
        )
        .placeholder("Select game type")
        .padding(10)
        .width(Length::Fill);

        let game_type_row = row![
            Text::new("Game Type:"), game_type_picker,
        ]
            .spacing(10)
            .align_y(Center);

        // Go Button (aligned bottom-right)
        let go_button = button(Text::new("Go"))
            .on_press(GameMessage::Menu_NewGameStartClicked)
            .padding(10)
            ;//.style(theme::Button::Primary);

        let layout = Column::new()
            .spacing(20)
            .padding(20)
            .push(name_row)
            .push(game_type_row)
            .push(
                Container::new(go_button)
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill),
            )
            .height(Length::Fill)
            .align_x(Left);

        let self_theme = self.theme();
        Container::<GameMessage, Theme, iced::Renderer>::new(layout)
            .width(Length::Fixed(600.0))
            .height(Length::Fixed(400.0))
            .style(move |_theme| menu_right_box_style(&self_theme))
            .padding(10)
    }

    // pub fn build_settings_ui<'a>(&self) -> Container<'a, GameMessage> {
    //     Container::<GameMessage, Theme, iced::Renderer>::new(text("TODO settings UI"))
    // }


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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewGame_Type {
    Type_A, Type_B, Type_C,
}
impl std::fmt::Display for NewGame_Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NewGame_Type::Type_A => write!(f, "Type A"),
            NewGame_Type::Type_B => write!(f, "Type B"),
            NewGame_Type::Type_C => write!(f, "type C"),
        }
    }
}
impl NewGame_Type {
    const ALL: [NewGame_Type; 3] = [
        NewGame_Type::Type_A,
        NewGame_Type::Type_B,
        NewGame_Type::Type_C,
    ];
}

