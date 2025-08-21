use crate::gui::types::*;
use iced::widget::{
    Button, Container, Text, button, center_x, column, pick_list, row, text, text_input, toggler,
};
use iced::{Center, Element, Length, Left, Theme};

pub const MAIN_MENU_RIGHT_WIDTH_PX: f32 = 720.0;
pub const MAIN_MENU_HEIGHT_PX: f32 = 460.0;

impl GameWindow {
    pub fn build_menu_screen_right_ui(&self) -> Container<'_, GameMessage> {
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
                        self.build_settings_ui()
                    }
                    crate::game::WelcomeScreenView::Licenses => {
                        self.build_licenses_ui()
                    }
                    _ => {
                        Container::<GameMessage, Theme, iced::Renderer>::new(text(
                            crate::translations::t(crate::translations::TranslationKey::SelectFromLeftMenu, "eng")
                        ))
                    }
                }
            }
            else {
                Container::<GameMessage, Theme, iced::Renderer>::new(text(
                    crate::translations::t(crate::translations::TranslationKey::SelectFromLeftMenu, "eng")
                ))
            }
        }
        else {
            Container::<GameMessage, Theme, iced::Renderer>::new(text(
                crate::translations::t(crate::translations::TranslationKey::SelectFromLeftMenu, "eng")
            ))
        }
    }

    pub fn build_continue_game_ui(&self) -> Container<'_, GameMessage> {
        let user_language = &self.settings_language;
        
        let saved_games = crate::crisis::get_saved_crisis_names();
        let game_type_picker = pick_list(
            saved_games,
            self.continue_game_game_choice.clone(),
            GameMessage::Menu_ContinueGameChoiceAltered,
        )
        .placeholder(crate::translations::t(crate::translations::TranslationKey::SelectGame, user_language))
        .padding(10)
        .width(Length::Fill);

        let game_type_row = row![
            Text::new(crate::translations::t(crate::translations::TranslationKey::SavedGame, user_language)), 
            game_type_picker,
        ]
            .spacing(10)
            .align_y(Center);

        let go_button = button(Text::new(crate::translations::t(crate::translations::TranslationKey::Play, user_language)))
            .on_press(GameMessage::Menu_ContinueGameStartClicked)
            .padding(10);

        let delete_button = if self.continue_game_game_choice.is_some() {
            button(Text::new(crate::translations::t(crate::translations::TranslationKey::Delete, user_language)))
                .on_press(GameMessage::Menu_ContinueGameDeleteRequested(
                    self.continue_game_game_choice.clone().unwrap_or_default()
                ))
                .padding(10)
                .style(move |theme: &Theme, status| {
                    let palette = theme.extended_palette();
                    iced::widget::button::Style {
                        background: Some(match status {
                            iced::widget::button::Status::Active => palette.danger.base.color.into(),
                            iced::widget::button::Status::Hovered => palette.danger.strong.color.into(),
                            _ => palette.danger.weak.color.into(),
                        }),
                        text_color: palette.danger.base.text,
                        border: iced::border::rounded(4),
                        ..iced::widget::button::Style::default()
                    }
                })
        } else {
            button(Text::new(crate::translations::t(crate::translations::TranslationKey::Delete, user_language)))
                .padding(10)
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

        let button_row = row![delete_button, iced::widget::horizontal_space(), go_button]
            .spacing(10)
            .align_y(Center);

        let mut layout = iced::widget::Column::new()
            .spacing(20)
            .padding(20)
            .push(game_type_row);

        // Add confirmation dialog if delete is requested
        if let Some(ref game_name) = self.continue_game_delete_confirmation {
            let confirmation_text = Text::new(crate::translations::t(crate::translations::TranslationKey::DeleteGame, user_language));
            let game_info_text = Text::new(format!("\"{}\"", game_name))
                .size(16)
                .color(iced::Color::from_rgb(0.6, 0.6, 0.6));
            
            let confirm_button = button(Text::new(crate::translations::t(crate::translations::TranslationKey::ConfirmDelete, user_language)))
                .on_press(GameMessage::Menu_ContinueGameDeleteConfirmed(game_name.clone()))
                .padding(10)
                .style(move |theme: &Theme, status| {
                    let palette = theme.extended_palette();
                    iced::widget::button::Style {
                        background: Some(match status {
                            iced::widget::button::Status::Active => palette.danger.base.color.into(),
                            iced::widget::button::Status::Hovered => palette.danger.strong.color.into(),
                            _ => palette.danger.weak.color.into(),
                        }),
                        text_color: palette.danger.base.text,
                        border: iced::border::rounded(4),
                        ..iced::widget::button::Style::default()
                    }
                });
            
            let cancel_button = button(Text::new(crate::translations::t(crate::translations::TranslationKey::Cancel, user_language)))
                .on_press(GameMessage::Menu_ContinueGameDeleteRequested("".to_string())) // Cancel by clearing
                .padding(10);
            
            let confirmation_buttons = row![confirm_button, cancel_button]
                .spacing(10)
                .align_y(Center);
                
            let confirmation_dialog = column![
                confirmation_text,
                game_info_text,
                Container::new(confirmation_buttons)
                    .align_x(iced::alignment::Horizontal::Center)
                    .width(Length::Fill)
            ]
            .spacing(10)
            .padding(15)
            .align_x(Center);
            
            let confirmation_container = Container::new(confirmation_dialog)
                .width(Length::Fill)
                .style(move |theme: &Theme| {
                    let palette = theme.extended_palette();
                    iced::widget::container::Style {
                        background: Some(palette.danger.weak.color.into()),
                        border: iced::border::rounded(8)
                            .color(palette.danger.base.color)
                            .width(2),
                        ..iced::widget::container::Style::default()
                    }
                });
                
            layout = layout.push(confirmation_container);
        }

        layout = layout.push(button_row)
            .height(Length::Fill)
            .align_x(Left);

        let self_theme = self.theme();
        Container::<GameMessage, Theme, iced::Renderer>::new(layout)
            .width(Length::Fixed(MAIN_MENU_RIGHT_WIDTH_PX))
            .height(Length::Fixed(MAIN_MENU_HEIGHT_PX))
            .style(move |_theme| super::menu_right_box_style(&self_theme))
            .padding(10)
    }

    pub fn build_new_game_ui(&self) -> Container<'_, GameMessage> {
        let user_language = &self.settings_language;
        
        let name_input = text_input(
            &crate::translations::t(crate::translations::TranslationKey::EnterName, user_language), 
            &self.new_game_player_name
        )
            .on_input(GameMessage::Menu_NewGamePlayerNameAltered)
            .padding(10)
            .width(Length::Fill);

        let name_row = row![
                Text::new(crate::translations::t(crate::translations::TranslationKey::PlayerName, user_language)), 
                name_input
            ]
            .spacing(10)
            .align_y(Center);

        let crisis_names = crate::crisis::get_crisis_names_localized(user_language);
        let selected_display_name = self.new_game_game_template.as_ref()
            .and_then(|template_name| {
                // Find the display name that matches this template
                for display_name in &crisis_names {
                    let found_template = crate::crisis::get_template_name_from_display_name(display_name);
                    if found_template == *template_name {
                        return Some(display_name.clone());
                    }
                }
                None
            });
        let game_type_picker = pick_list(
            crisis_names,
            selected_display_name,
            GameMessage::Menu_NewGameTemplateChoiceAltered,
        )
        .placeholder(crate::translations::t(crate::translations::TranslationKey::SelectGameType, user_language))
        .padding(10)
        .width(Length::Fill);

        let game_type_row = row![
            Text::new(crate::translations::t(crate::translations::TranslationKey::GameType, user_language)), 
            game_type_picker,
        ]
            .spacing(10)
            .align_y(Center);

        let go_button = button(Text::new(crate::translations::t(crate::translations::TranslationKey::Go, user_language)))
            .on_press(GameMessage::Menu_NewGameStartClicked)
            .padding(10);

        let mut layout = iced::widget::Column::new()
            .spacing(20)
            .padding(20)
            .push(name_row)
            .push(game_type_row);

        // Add description area if a crisis is selected
        if let Some(ref description) = self.new_game_selected_description {
            let description_text = Text::new(description)
                .size(14)
                .wrapping(iced::widget::text::Wrapping::Word)
                .color(iced::Color::from_rgb(0.6, 0.6, 0.6));
            
            let description_container = Container::new(description_text)
                .width(Length::Fill)
                .padding(15)
                .style(move |theme: &Theme| {
                    let palette = theme.extended_palette();
                    iced::widget::container::Style {
                        background: Some(palette.background.weak.color.into()),
                        border: iced::border::rounded(8)
                            .color(palette.primary.weak.color)
                            .width(1),
                        ..iced::widget::container::Style::default()
                    }
                });
            
            layout = layout.push(description_container);
        }

        layout = layout.push(
            Container::new(go_button)
                .align_x(iced::alignment::Horizontal::Right)
                .width(Length::Fill),
        )
        .height(Length::Fill)
        .align_x(Left);

        let self_theme = self.theme();
        Container::<GameMessage, Theme, iced::Renderer>::new(layout)
            .width(Length::Fixed(MAIN_MENU_RIGHT_WIDTH_PX))
            .height(Length::Fixed(MAIN_MENU_HEIGHT_PX))
            .style(move |_theme| super::menu_right_box_style(&self_theme))
            .padding(10)
    }

    pub fn build_settings_ui(&self) -> Container<'_, GameMessage> {
        let user_language = &self.settings_language;
        
        // Pre-translate all strings to avoid lifetime issues
        let save_folder_placeholder = crate::translations::t(crate::translations::TranslationKey::EnterCrisesFolderPath, user_language);
        let save_folder_label = crate::translations::t(crate::translations::TranslationKey::GameCrisesFolder, user_language);
        let difficulty_label = crate::translations::t(crate::translations::TranslationKey::DifficultyLevel, user_language);
        let difficulty_placeholder = crate::translations::t(crate::translations::TranslationKey::SelectDifficulty, user_language);
        let autosave_label = crate::translations::t(crate::translations::TranslationKey::Autosave, user_language);
        let language_label = crate::translations::t(crate::translations::TranslationKey::Language, user_language);
        let language_placeholder = crate::translations::t(crate::translations::TranslationKey::SelectLanguage, user_language);
        
        let save_folder_input = text_input(
            &save_folder_placeholder, 
            &self.settings_game_crises_folder
        )
            .on_input(GameMessage::Menu_SettingsGameCrisesFolderChanged)
            .padding(10)
            .width(Length::Fill);

        // Create the save folder row with optional Open button
        #[cfg(not(target_arch = "wasm32"))]
        let save_folder_row = {
            let open_button = button(Text::new(crate::translations::t(crate::translations::TranslationKey::OpenFolder, user_language)))
                .on_press(GameMessage::Menu_SettingsOpenCrisesFolder)
                .padding([10, 15]);

            row![
                Text::new(save_folder_label), 
                save_folder_input,
                open_button
            ]
            .spacing(10)
            .align_y(Center)
        };

        #[cfg(target_arch = "wasm32")]
        let save_folder_row = row![
            Text::new(save_folder_label), 
            save_folder_input
        ]
        .spacing(10)
        .align_y(Center);

        let crises_folder_explanation = Text::new(crate::translations::t(crate::translations::TranslationKey::CrisesFolderExplanation, user_language))
            .size(12)
            .color(iced::Color::from_rgb(0.6, 0.6, 0.6));

        // Create difficulty options with translations
        let difficulty_options: Vec<String> = DifficultyLevel::ALL.iter()
            .map(|d| d.to_translated_string(user_language))
            .collect();
        let current_difficulty_display = self.settings_difficulty_level.to_translated_string(user_language);
        
        let difficulty_picker = pick_list(
            difficulty_options,
            Some(current_difficulty_display),
            |selected| {
                // Map back from translated string to enum
                for difficulty in &DifficultyLevel::ALL {
                    if difficulty.to_translated_string(user_language) == selected {
                        return GameMessage::Menu_SettingsDifficultyLevelChanged(*difficulty);
                    }
                }
                GameMessage::Menu_SettingsDifficultyLevelChanged(DifficultyLevel::Medium)
            },
        )
        .placeholder(&difficulty_placeholder)
        .padding(10)
        .width(Length::Fill);

        let difficulty_row = row![
            Text::new(difficulty_label), 
            difficulty_picker,
        ]
        .spacing(10)
        .align_y(Center);

        let autosave_toggle = toggler(self.settings_autosave)
            .on_toggle(GameMessage::Menu_SettingsAutosaveToggled)
            .width(Length::Shrink);

        let autosave_row = row![
            Text::new(autosave_label), 
            autosave_toggle,
        ]
        .spacing(10)
        .align_y(Center);

        let available_languages = crate::language::get_available_languages();
        let language_options: Vec<String> = available_languages.iter().map(|(code, name)| format!("{} ({})", name, code)).collect();
        let current_language_display = available_languages.iter()
            .find(|(code, _)| code == &self.settings_language)
            .map(|(code, name)| format!("{} ({})", name, code))
            .unwrap_or_else(|| format!("Unknown ({})", self.settings_language));

        let language_picker = pick_list(
            language_options,
            Some(current_language_display),
            |selected| {
                let lang_code = selected.split(" (").last().unwrap_or("eng").trim_end_matches(')').to_string();
                GameMessage::Menu_SettingsLanguageChanged(lang_code)
            },
        )
        .placeholder(&language_placeholder)
        .padding(10)
        .width(Length::Fill);

        let language_row = row![
            Text::new(language_label), 
            language_picker,
        ]
        .spacing(10)
        .align_y(Center);

        let layout = iced::widget::Column::new()
            .spacing(20)
            .padding(20)
            .push(save_folder_row)
            .push(crises_folder_explanation)
            .push(difficulty_row)
            .push(autosave_row)
            .push(language_row)
            .height(Length::Fill)
            .align_x(Left);

        let self_theme = self.theme();
        Container::<GameMessage, Theme, iced::Renderer>::new(layout)
            .width(Length::Fixed(MAIN_MENU_RIGHT_WIDTH_PX))
            .height(Length::Fixed(MAIN_MENU_HEIGHT_PX))
            .style(move |_theme| super::menu_right_box_style(&self_theme))
            .padding(10)
    }

    pub fn build_licenses_ui(&self) -> Container<'_, GameMessage> {
        
        let license_content = text(Self::get_license_content())
            .size(14)
            .wrapping(iced::widget::text::Wrapping::Word);
        
        let scrollable_content = iced::widget::Scrollable::new(
            column![license_content]
                .spacing(10)
                .padding(20)
        )
        .width(Length::Fill)
        .height(Length::Fill);
        
        let layout = column![scrollable_content]
            .spacing(10)
            .padding(10)
            .height(Length::Fill)
            .width(Length::Fill);

        let self_theme = self.theme();
        Container::<GameMessage, Theme, iced::Renderer>::new(layout)
            .width(Length::Fixed(MAIN_MENU_RIGHT_WIDTH_PX))
            .height(Length::Fixed(MAIN_MENU_HEIGHT_PX))
            .style(move |_theme| super::menu_right_box_style(&self_theme))
            .padding(10)
    }

    fn get_license_content() -> String {
        r#"THIRD-PARTY SOFTWARE LICENSES AND ATTRIBUTIONS

This software includes the following third-party libraries:

=== RUST LIBRARIES ===

Iced GUI Framework
License: MIT
Copyright (c) 2019 Héctor Ramón
Used for: User interface rendering and interaction

Serde
License: MIT OR Apache-2.0
Copyright (c) 2014 Erick Tryzelaar and David Tolnay
Used for: Serialization and deserialization

TOML
License: MIT OR Apache-2.0
Copyright (c) 2014 Alex Crichton
Used for: Configuration file parsing

once_cell
License: MIT OR Apache-2.0
Copyright (c) 2018 Aleksey Kladov
Used for: Lazy static initialization

rust-embed
License: MIT
Copyright (c) 2018 pyros2097
Used for: Asset embedding in binary

=== IMAGE ATTRIBUTIONS ===

Background Images and Character Assets:
- Crisis background images are original compositions or sourced from public domain collections
- Character portraits are generated or sourced from royalty-free collections
- UI elements and icons are custom-designed for this application

If you believe any content violates copyright, please contact the development team.

=== FULL LICENSE TEXTS ===

MIT License

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

Apache License 2.0

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License."#.to_string()
    }
}
