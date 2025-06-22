use iced::highlighter;
use iced::keyboard;
use iced::widget::{
    self, button, center, column, container, horizontal_space, pick_list,
    row, text, text_editor, toggler, tooltip, center_x,
    Container, Image,
};
use iced::{Length, Left, Right, Center, Element, Fill, Font, Task, Theme};

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
    //FileSaved(Result<PathBuf, Error>),
    Nop,
}

impl GameWindow {
    pub fn new() -> (Self, Task<GameMessage>) {
        #[cfg(target_os = "macos")]
        {
            std::thread::spawn(move || {
                macos_menu::install_run_menu();// Let's see if MacOS is as picky as Windorks about main-thread affinity
            });
        }

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



#[cfg(target_os = "macos")]
mod macos_menu {
    use cocoa::appkit::{
        NSApp, NSApplication, NSApplicationActivationPolicyRegular, NSMenu, NSMenuItem,
        NSRunningApplication, NSWindow, NSWindowStyleMask, NSBackingStoreType,
    };
    use cocoa::base::{id, nil, YES, NO};
    use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
    use objc::runtime::{Class, Object, Sel};
    use objc::{msg_send, sel, sel_impl};

    pub fn install_run_menu() /*-> *mut Object*/ {
        unsafe {
            let _pool = NSAutoreleasePool::new(nil);

            // Create app
            let app = NSApp();
            app.setActivationPolicy_(NSApplicationActivationPolicyRegular);

            // Create menu bar
            let menubar = NSMenu::new(nil).autorelease();
            let app_menu_item = NSMenuItem::new(nil).autorelease();
            menubar.addItem_(app_menu_item);
            app.setMainMenu_(menubar);

            // Create app menu
            let app_menu = NSMenu::new(nil).autorelease();

            // "Say Hello" menu item
            let hello_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
                NSString::alloc(nil).init_str("Say Hello"),
                sel!(sayHello:),
                NSString::alloc(nil).init_str("h"),
            ).autorelease();
            app_menu.addItem_(hello_item);

            // Separator
            app_menu.addItem_(NSMenuItem::separatorItem(nil));

            // "Quit" menu item
            let quit_title = NSString::alloc(nil).init_str("Quit");
            let quit_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
                quit_title,
                sel!(terminate:),
                NSString::alloc(nil).init_str("q"),
            ).autorelease();
            app_menu.addItem_(quit_item);

            app_menu_item.setSubmenu_(app_menu);

            // Create window
            /*let window = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
                NSRect::new(NSPoint::new(0., 0.), NSSize::new(400., 300.)),
                NSWindowStyleMask::NSTitledWindowMask,
                NSBackingStoreType::NSBackingStoreBuffered,
                NO,
            ).autorelease();
            window.cascadeTopLeftFromPoint_(NSPoint::new(20., 20.));
            window.setTitle_(NSString::alloc(nil).init_str("Hello macOS"));
            window.makeKeyAndOrderFront_(nil);
            */

            // Set up responder for "sayHello:"
            let delegate: id = msg_send![create_hello_delegate(), new];
            //window.setDelegate_(delegate);
            app.setDelegate_(delegate);

            // Activate app and run
            NSRunningApplication::currentApplication(nil).activateWithOptions_(cocoa::appkit::NSApplicationActivateIgnoringOtherApps);
            app.run();
        }
    }

    /// Create a custom NSObject subclass with a sayHello: method
    unsafe fn create_hello_delegate() -> *const Class {
        let superclass = Class::get("NSObject").unwrap();
        let mut decl = objc::declare::ClassDecl::new("HelloDelegate", superclass).unwrap();

        // Add method
        extern "C" fn say_hello(_: &Object, _: Sel, _: id) {
            println!("Hello from menu!");
        }

        unsafe { decl.add_method(sel!(sayHello:), say_hello as extern "C" fn(&Object, Sel, id)); }

        decl.register()
    }

}


