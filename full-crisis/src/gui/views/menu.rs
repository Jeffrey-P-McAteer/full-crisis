use crate::gui::types::*;
use iced::widget::{
    Container, Image, button, column, container, row, text, Space, Column, Row
};
use iced::{Center, Element, Length, Theme};

const SPLASH_PNG_BYTES: &[u8] = include_bytes!("../../../../icon/full-crisis-splash.transparent.png");

impl GameWindow {
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

        let app_version = env!("CARGO_PKG_VERSION");
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
            button(text(crate::translations::t(crate::translations::TranslationKey::Licenses, user_language)))
                .on_press(GameMessage::Menu_LicensesRequested)
                .width(Length::Fill),
            button(text(crate::translations::t(crate::translations::TranslationKey::QuitGame, user_language)))
                .on_press(GameMessage::QuitGameRequested)
                .width(Length::Fill),
            text(format!("Version {}", app_version))
                .width(Length::Fill),
        ]
        .spacing(10)
        .width(240)
        .padding(10)
        .align_x(Center)
        .width(Length::Fixed(186.0f32));

        let right_panel = container(self.build_menu_screen_right_ui())
            .width(Length::Fixed(760.0f32))
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
            .push(foreground_content)
            .push(Space::with_height(Length::Fill));

        let foreground_content = Row::new()
            .height(Length::Fill)
            .width(Length::Shrink)
            .align_y(iced::alignment::Vertical::Center)
            .push(Space::with_height(Length::Fill))
            .push(foreground_content)
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
}