use crate::gui::types::*;
use iced::widget::{
    Button, Container, Text, button, center_x, column, pick_list, row, text, text_input, toggler,
};
use iced::{Center, Element, Length, Left, Theme};

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

        let layout = iced::widget::Column::new()
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

        let crisis_names = crate::crisis::get_crisis_names();
        let game_type_picker = pick_list(
            crisis_names,
            self.new_game_game_template.clone(),
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

        let layout = iced::widget::Column::new()
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
            .style(move |_theme| super::menu_right_box_style(&self_theme))
            .padding(10)
    }

    pub fn build_settings_ui(&self) -> Container<'_, GameMessage> {
        let user_language = &self.settings_language;
        
        // Pre-translate all strings to avoid lifetime issues
        let save_folder_placeholder = crate::translations::t(crate::translations::TranslationKey::EnterSaveFolderPath, user_language);
        let save_folder_label = crate::translations::t(crate::translations::TranslationKey::GameSaveFolder, user_language);
        let difficulty_label = crate::translations::t(crate::translations::TranslationKey::DifficultyLevel, user_language);
        let difficulty_placeholder = crate::translations::t(crate::translations::TranslationKey::SelectDifficulty, user_language);
        let autosave_label = crate::translations::t(crate::translations::TranslationKey::Autosave, user_language);
        let language_label = crate::translations::t(crate::translations::TranslationKey::Language, user_language);
        let language_placeholder = crate::translations::t(crate::translations::TranslationKey::SelectLanguage, user_language);
        
        let save_folder_input = text_input(
            &save_folder_placeholder, 
            &self.settings_game_save_folder
        )
            .on_input(GameMessage::Menu_SettingsGameSaveFolderChanged)
            .padding(10)
            .width(Length::Fill);

        let save_folder_row = row![
            Text::new(save_folder_label), 
            save_folder_input
        ]
        .spacing(10)
        .align_y(Center);

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
            .push(difficulty_row)
            .push(autosave_row)
            .push(language_row)
            .height(Length::Fill)
            .align_x(Left);

        let self_theme = self.theme();
        Container::<GameMessage, Theme, iced::Renderer>::new(layout)
            .width(Length::Fixed(600.0))
            .height(Length::Fixed(400.0))
            .style(move |_theme| super::menu_right_box_style(&self_theme))
            .padding(10)
    }
}