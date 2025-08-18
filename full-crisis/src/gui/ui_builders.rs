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

        let go_button = button(Text::new("Play"))
            .on_press(GameMessage::Menu_NewGameStartClicked)
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

    pub fn build_new_game_ui<'a>(&self) -> Container<'a, GameMessage> {
        let name_input = text_input("Enter name...", &self.new_game_player_name)
            .on_input(GameMessage::Menu_NewGamePlayerNameAltered)
            .padding(10)
            .width(Length::Fill);

        let name_row = row![
                Text::new("Player Name:"), name_input
            ]
            .spacing(10)
            .align_y(Center);

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

        let go_button = button(Text::new("Go"))
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

    pub fn build_settings_ui<'a>(&self) -> Container<'a, GameMessage> {
        let save_folder_input = text_input("Enter save folder path...", &self.settings_game_save_folder)
            .on_input(GameMessage::Menu_SettingsGameSaveFolderChanged)
            .padding(10)
            .width(Length::Fill);

        let save_folder_row = row![
            Text::new("Game Save Folder:"), save_folder_input
        ]
        .spacing(10)
        .align_y(Center);

        let difficulty_picker = pick_list(
            &DifficultyLevel::ALL[..],
            Some(self.settings_difficulty_level),
            GameMessage::Menu_SettingsDifficultyLevelChanged,
        )
        .placeholder("Select difficulty")
        .padding(10)
        .width(Length::Fill);

        let difficulty_row = row![
            Text::new("Difficulty Level:"), difficulty_picker,
        ]
        .spacing(10)
        .align_y(Center);

        let autosave_toggle = toggler(self.settings_autosave)
            .on_toggle(GameMessage::Menu_SettingsAutosaveToggled)
            .width(Length::Shrink);

        let autosave_row = row![
            Text::new("Autosave:"), autosave_toggle,
        ]
        .spacing(10)
        .align_y(Center);

        let layout = iced::widget::Column::new()
            .spacing(20)
            .padding(20)
            .push(save_folder_row)
            .push(difficulty_row)
            .push(autosave_row)
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