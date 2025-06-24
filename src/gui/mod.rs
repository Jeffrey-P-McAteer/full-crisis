use iced::highlighter;
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
const SPLASH_PNG_BYTES: &[u8] = include_bytes!("../../icon/full-crisis-splash.transparent.png");

pub struct GameWindow {
    pub game_state: crate::game::GameState,
}

#[derive(Debug, Clone)]
pub enum GameMessage {
    ActionPerformed(text_editor::Action),
    ThemeSelected(highlighter::Theme),
    WordWrapToggled(bool),
    NewFile,
    OpenFile,
    //FileOpened(Result<(PathBuf, Arc<String>), Error>),
    SaveFile,
    Nop,
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
            /*Message::ActionPerformed(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();

                self.content.perform(action);

                Task::none()
            }
            Message::ThemeSelected(theme) => {
                self.theme = theme;

                Task::none()
            }
            Message::WordWrapToggled(word_wrap) => {
                self.word_wrap = word_wrap;

                Task::none()
            }
            Message::NewFile => {
                if !self.is_loading {
                    self.file = None;
                    self.content = text_editor::Content::new();
                }

                Task::none()
            }
            Message::OpenFile => {
                if self.is_loading {
                    Task::none()
                } else {
                    self.is_loading = true;

                    Task::perform(open_file(), Message::FileOpened)
                }
            }
            Message::FileOpened(result) => {
                self.is_loading = false;
                self.is_dirty = false;

                if let Ok((path, contents)) = result {
                    self.file = Some(path);
                    self.content = text_editor::Content::with_text(&contents);
                }

                Task::none()
            }
            Message::SaveFile => {
                if self.is_loading {
                    Task::none()
                } else {
                    self.is_loading = true;

                    let mut text = self.content.text();

                    if let Some(ending) = self.content.line_ending() {
                        if !text.ends_with(ending.as_str()) {
                            text.push_str(ending.as_str());
                        }
                    }

                    Task::perform(
                        save_file(self.file.clone(), text),
                        Message::FileSaved,
                    )
                }
            }
            Message::FileSaved(result) => {
                self.is_loading = false;

                if let Ok(path) = result {
                    self.file = Some(path);
                    self.is_dirty = false;
                }

                Task::none()
            },*/
            GameMessage::Nop => Task::none(),
            _ => unimplemented!(),
        }
    }

    pub fn view(&self) -> Element<GameMessage> {
        let splash_handle = iced::widget::image::Handle::from_bytes(SPLASH_PNG_BYTES);
        let splash_img = Image::new(splash_handle)
            .width(Length::Fill)
            .height(Length::Fill);
        let background = Container::new(splash_img)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        /*let buttons = column![
            /*text(if let Some(path) = &self.file {
                let path = path.display().to_string();

                if path.len() > 60 {
                    format!("...{}", &path[path.len() - 40..])
                } else {
                    path
                }
            } else {
                String::from("New file")
            }),
            horizontal_space(),
            text({
                let (line, column) = self.content.cursor_position();

                format!("{}:{}", line + 1, column + 1)
            })*/
        ]
        .spacing(10)
        .align_x(Left);*/

        //Modal::new(background, buttons).into()
        background.into()
    }

    pub fn theme(&self) -> Theme {
        /*if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }*/
        Theme::Light
    }
}

async fn run_background_async_tasks() -> Result<(), crate::err::BoxError> {
    eprintln!("TODO run_background_async_tasks");

    Ok(())
}

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
