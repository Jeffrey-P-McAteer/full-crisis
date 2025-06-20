use iced::highlighter;
use iced::keyboard;
use iced::widget::{
    self, button, center, column, container, horizontal_space, pick_list,
    row, text, text_editor, toggler, tooltip, center_x,
};
use iced::{Center, Element, Fill, Font, Task, Theme};

use std::ffi;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct GameWindow {

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
    //FileSaved(Result<PathBuf, Error>),
    Nop,
}

impl GameWindow {
    pub fn new() -> (Self, Task<GameMessage>) {
        (
            Self {
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
                    |_| { GameMessage::Nop }, // _wierd_, why?
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
            GameMessage::Nop => {
              Task::none()
            },
            _ => unimplemented!()
        }
    }

    pub fn view(&self) -> Element<GameMessage> {
        let controls = row![
            /*action(new_icon(), "New file", Some(Message::NewFile)),
            action(
                open_icon(),
                "Open file",
                (!self.is_loading).then_some(Message::OpenFile)
            ),
            action(
                save_icon(),
                "Save file",
                self.is_dirty.then_some(Message::SaveFile)
            ),
            horizontal_space(),
            toggler(self.word_wrap)
                .label("Word Wrap")
                .on_toggle(Message::WordWrapToggled),
            pick_list(
                highlighter::Theme::ALL,
                Some(self.theme),
                Message::ThemeSelected
            )
            .text_size(14)
            .padding([5, 10])*/
        ]
        .spacing(10)
        .align_y(Center);

        let status = row![
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
        .spacing(10);

        column![
            controls,
            /*text_editor(&self.content)
                .height(Fill)
                .on_action(Message::ActionPerformed)
                .wrapping(if self.word_wrap {
                    text::Wrapping::Word
                } else {
                    text::Wrapping::None
                })
                .highlight(
                    self.file
                        .as_deref()
                        .and_then(Path::extension)
                        .and_then(ffi::OsStr::to_str)
                        .unwrap_or("rs"),
                    self.theme,
                )
                .key_binding(|key_press| {
                    match key_press.key.as_ref() {
                        keyboard::Key::Character("s")
                            if key_press.modifiers.command() =>
                        {
                            Some(text_editor::Binding::Custom(
                                Message::SaveFile,
                            ))
                        }
                        _ => text_editor::Binding::from_key_press(key_press),
                    }
                }),*/
            status,
        ]
        .spacing(10)
        .padding(10)
        .into()
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

