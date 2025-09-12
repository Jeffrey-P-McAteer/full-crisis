use crate::gui::types::*;
use crate::gui::helpers::TranslationUtils;
use iced::widget::{
    Container, Image, button, column, container, row, text, Space, text_input
};
use iced::{Center, Element, Length, Theme};

impl GameWindow {
    pub fn view_game_screen(&self) -> Element<'_, GameMessage> {
        if let (Some(crisis), Some(story_state)) = (&self.current_crisis, &self.story_state) {
            self.render_story_scene(crisis, story_state)
        } else {
            container(
                column![
                    text(TranslationUtils::translate(crate::translations::TranslationKey::LoadingCrisis, &self.settings_language)).size(self.font_size_large()),
                    button(text(TranslationUtils::translate(crate::translations::TranslationKey::ReturnToMenu, &self.settings_language)).size(self.font_size_base()))
                        .on_press(GameMessage::Game_RestartRequested)
                        .padding(10)
                ]
                .spacing(20)
                .align_x(Center)
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }
    }
    
    // WARNING: Long Function
    fn render_story_scene(&self, crisis: &crate::crisis::CrisisDefinition, story_state: &crate::crisis::GameState) -> Element<'_, GameMessage> {
        if let Some(current_scene) = crisis.scenes.get(&story_state.current_scene) {
            let mut error_messages = Vec::new();
            
            self.validate_background_audio(current_scene, story_state, &mut error_messages);
            let background_layer = self.create_background_layer(current_scene, story_state, &mut error_messages);
            let main_layout = self.create_main_game_layout(crisis, current_scene, story_state, &mut error_messages);
            
            if let Some(background) = background_layer {
                iced::widget::stack![
                    background,
                    Container::new(main_layout)
                        .width(Length::Fill)
                        .height(Length::Fill)
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else {
                Container::new(main_layout)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        } else {
            self.render_scene_not_found(story_state)
        }
    }

    fn create_background_layer(
        &self, 
        current_scene: &crate::crisis::CrisisScene, 
        story_state: &crate::crisis::GameState,
        error_messages: &mut Vec<String>
    ) -> Option<iced::widget::Stack<'_, GameMessage, Theme, iced::Renderer>> {
        if let Some(ref bg_path) = current_scene.background_image {
            if let Some(bg_file) = crate::crisis::PlayableCrises::get(bg_path) {
                let bg_handle = iced::widget::image::Handle::from_bytes(bg_file.data.as_ref().to_vec());
                let bg_img = Image::<iced::widget::image::Handle>::new(bg_handle)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(iced::ContentFit::Cover);
                
                let overlay = Container::<GameMessage, Theme, iced::Renderer>::new(
                    iced::widget::Space::with_width(Length::Fill)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme: &Theme| {
                    iced::widget::container::Style {
                        background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3).into()),
                        ..iced::widget::container::Style::default()
                    }
                });
                
                Some(iced::widget::stack![
                    Container::<GameMessage, Theme, iced::Renderer>::new(bg_img)
                        .width(Length::Fill)
                        .height(Length::Fill),
                    overlay
                ]
                .width(Length::Fill)
                .height(Length::Fill))
            } else {
                self.add_scene_error(error_messages, &story_state.current_scene, &format!("Background image file not found: {}", bg_path));
                None
            }
        } else {
            self.add_scene_error(error_messages, &story_state.current_scene, "No background_image defined");
            None
        }
    }

    fn create_main_game_layout(
        &self,
        crisis: &crate::crisis::CrisisDefinition,
        current_scene: &crate::crisis::CrisisScene,
        story_state: &crate::crisis::GameState,
        error_messages: &mut Vec<String>
    ) -> iced::widget::Column<'_, GameMessage, Theme, iced::Renderer> {
        let top_data = self.create_top_data_section(crisis, story_state);
        let bottom_row = self.create_bottom_game_section(crisis, current_scene, story_state, error_messages);
        
        if !error_messages.is_empty() {
            let error_display = self.create_error_display(error_messages);
            column![top_data, bottom_row, error_display]
                .width(Length::Fill)
                .height(Length::Fill)
        } else {
            column![top_data, bottom_row]
                .width(Length::Fill)
                .height(Length::Fill)
        }
    }

    fn create_top_data_section(&self, crisis: &crate::crisis::CrisisDefinition, story_state: &crate::crisis::GameState) -> Container<'_, GameMessage, Theme, iced::Renderer> {
        let title = crate::crisis::get_localized_text(&crisis.name, &story_state.language);
        let mut vars = std::collections::HashMap::new();
        vars.insert("character_name".to_string(), story_state.character_name.clone());
        let character_info = text(
            crate::translations::t_vars(crate::translations::TranslationKey::PlayingAs, &story_state.language, &vars)
        )
            .size(self.font_size_base())
            .color(iced::Color::from_rgb(0.6, 0.6, 0.6));
        
        let variables_text = if !story_state.variables.is_empty() {
            format!("Variables: {:?}", story_state.variables)
        } else {
            String::new()
        };
        
        let control_buttons = self.create_control_buttons(story_state);
        
        let top_row = row![
            container(control_buttons).align_x(iced::alignment::Horizontal::Left),
            container(
                column![
                    text(title.clone()).size(self.font_size_large()).align_x(Center),
                    character_info.align_x(Center),
                    if !variables_text.is_empty() {
                        text(variables_text.clone()).size(self.font_size_small()).color(iced::Color::from_rgb(0.5, 0.5, 0.5))
                    } else {
                        text("").size(self.font_size_base())
                    }
                ]
                .spacing(5)
                .align_x(Center)
            )
            .width(Length::Fill)
            .align_x(Center),
            container(Space::with_width(Length::Fixed(190.0)))
        ]
        .width(Length::Fill)
        .align_y(iced::alignment::Vertical::Top);
        
        container(top_row).width(Length::Fill).padding(20)
    }

    fn create_control_buttons(&self, story_state: &crate::crisis::GameState) -> iced::widget::Row<'_, GameMessage, Theme, iced::Renderer> {
        let save_button = button(
                text(TranslationUtils::translate(crate::translations::TranslationKey::SaveAndQuit, &story_state.language)).size(self.font_size_base())
                    .align_x(Center)
            )
            .on_press(GameMessage::Game_SaveAndQuitRequested)
            .padding(8)
            .width(Length::Fixed(140.0))
            .style(crate::gui::focused_button_style(self.focus_state.is_focused(FocusId("control", 0))));
            
        let quit_button = button(
                text(TranslationUtils::translate(crate::translations::TranslationKey::Quit, &story_state.language)).size(self.font_size_base())
                    .align_x(Center)
            )
            .on_press(GameMessage::Game_QuitWithoutSaveRequested)
            .padding(8)
            .width(Length::Fixed(60.0))
            .style(crate::gui::focused_button_style(self.focus_state.is_focused(FocusId("control", 1))));
            
        row![save_button, quit_button].spacing(10)
    }

    fn create_bottom_game_section(
        &self,
        _crisis: &crate::crisis::CrisisDefinition,
        current_scene: &crate::crisis::CrisisScene,
        story_state: &crate::crisis::GameState,
        error_messages: &mut Vec<String>
    ) -> iced::widget::Row<'_, GameMessage, Theme, iced::Renderer> {
        let left_column = self.create_story_choices_column(current_scene, story_state);
        let right_column = self.create_character_column(current_scene, story_state, error_messages);
        
        row![left_column, right_column]
            .width(Length::Fill)
            .height(Length::Fill)
    }

    fn create_story_choices_column(&self, current_scene: &crate::crisis::CrisisScene, story_state: &crate::crisis::GameState) -> iced::widget::Column<'_, GameMessage, Theme, iced::Renderer> {
        let scene_text = crate::crisis::get_scene_text_with_substitutions(current_scene, &story_state.language, &story_state.character_name, &story_state.text_inputs);
        let story_text_display = container(
            container(
                text(scene_text.clone())
                    .size(self.font_size_base())
                    .wrapping(iced::widget::text::Wrapping::Word)
            )
            .padding(20)
            .style(move |theme: &Theme| {
                let palette = theme.extended_palette();
                iced::widget::container::Style {
                    background: Some(palette.background.weak.color.into()),
                    border: iced::border::rounded(8)
                        .color(palette.primary.weak.color)
                        .width(1),
                    ..iced::widget::container::Style::default()
                }
            })
        )
        .width(Length::Fill)
        .padding(10);

        let choices_column = self.create_choices_section(current_scene, story_state);

        column![
            Space::with_height(Length::Fill),
            story_text_display,
            container(choices_column).padding(20)
        ]
        .spacing(10)
        .width(Length::FillPortion(60))
        .height(Length::Fill)
    }

    fn create_choices_section(&self, current_scene: &crate::crisis::CrisisScene, story_state: &crate::crisis::GameState) -> iced::widget::Column<'_, GameMessage, Theme, iced::Renderer> {
        let mut choices_column = column![].spacing(10);
        
        if current_scene.choices.is_empty() {
            choices_column = choices_column.push(
                column![
                    text(TranslationUtils::translate(crate::translations::TranslationKey::End, &story_state.language)).size(self.font_size_large()),
                    button(text(TranslationUtils::translate(crate::translations::TranslationKey::ReturnToMenu, &story_state.language)).size(self.font_size_base()))
                        .on_press(GameMessage::Game_RestartRequested)
                        .padding(10)
                        .width(Length::Fill)
                ]
                .spacing(10)
            );
        } else {
            choices_column = choices_column.push(
                text(TranslationUtils::translate(crate::translations::TranslationKey::WhatDoYouChoose, &story_state.language))
                    .size(self.font_size_base())
            );
            
            for (index, choice) in current_scene.choices.iter().enumerate() {
                let choice_element = self.create_choice_element(choice, index, story_state);
                choices_column = choices_column.push(choice_element);
            }
        }

        choices_column
    }

    fn create_choice_element(&self, choice: &crate::crisis::CrisisChoice, index: usize, story_state: &crate::crisis::GameState) -> iced::Element<'_, GameMessage> {
        let choice_text = crate::crisis::get_localized_text(&choice.text, &story_state.language);
        let available = self.is_choice_available(choice, story_state);
        
        if let Some(ref text_input) = choice.text_input {
            self.create_text_input_choice(text_input, choice_text, index, available, story_state)
        } else {
            self.create_regular_choice_button(choice_text, index, available, story_state)
        }
    }

    fn is_choice_available(&self, choice: &crate::crisis::CrisisChoice, story_state: &crate::crisis::GameState) -> bool {
        if let Some(ref requirements) = choice.requires {
            for (var, required_value) in requirements {
                if let Some(current_value) = story_state.variables.get(var) {
                    if current_value < required_value {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }
        true
    }

    fn create_text_input_choice(&self, text_input: &crate::crisis::CrisisTextInput, choice_text: String, index: usize, available: bool, story_state: &crate::crisis::GameState) -> iced::Element<'_, GameMessage> {
        let placeholder_text = if let Some(ref placeholder) = text_input.placeholder {
            crate::crisis::get_localized_text(placeholder, &story_state.language)
        } else {
            match text_input.input_type {
                crate::crisis::TextInputType::Text => "Enter text...".to_string(),
                crate::crisis::TextInputType::Number => "Enter number...".to_string(),
            }
        };
        
        let input_value = self.choice_text_inputs.get(&index).cloned().unwrap_or_default();
        
        let text_input_widget = iced::widget::text_input(&placeholder_text, &input_value)
            .on_input(move |value| GameMessage::Game_TextInputChanged(index, value))
            .on_submit(GameMessage::Game_TextInputSubmitted(index, input_value.clone()))
            .padding(8)
            .width(Length::Fill);
        
        let submit_button = if available && !input_value.is_empty() {
            button(text(choice_text.clone()).size(self.font_size_base()))
                .on_press(GameMessage::Game_TextInputSubmitted(index, input_value.clone()))
                .padding(10)
                .width(Length::Fixed(120.0))
        } else {
            button(text(choice_text.clone()).size(self.font_size_base()))
                .padding(10)
                .width(Length::Fixed(120.0))
                .style(self.create_disabled_button_style())
        };
        
        row![text_input_widget, submit_button]
            .spacing(10)
            .align_y(iced::alignment::Vertical::Center)
            .into()
    }

    fn create_regular_choice_button(&self, choice_text: String, index: usize, available: bool, story_state: &crate::crisis::GameState) -> iced::Element<'_, GameMessage> {
        if available {
            button(text(choice_text.clone()).size(self.font_size_base()))
                .on_press(GameMessage::Game_ChoiceSelected(index))
                .padding(10)
                .width(Length::Fill)
                .style(crate::gui::focused_button_style(self.focus_state.is_focused(FocusId("choice", index))))
                .into()
        } else {
            button(text(format!("{} {}", choice_text, 
                TranslationUtils::translate(crate::translations::TranslationKey::RequirementsNotMet, &story_state.language))))
                .padding(10)
                .width(Length::Fill)
                .style(self.create_disabled_button_style())
                .into()
        }
    }

    fn create_disabled_button_style(&self) -> fn(&Theme, iced::widget::button::Status) -> iced::widget::button::Style {
        move |theme: &Theme, _status| {
            let palette = theme.extended_palette();
            iced::widget::button::Style {
                background: Some(palette.background.weak.color.into()),
                text_color: palette.background.strong.text,
                border: iced::border::rounded(4)
                    .color(palette.background.strong.color)
                    .width(1),
                ..iced::widget::button::Style::default()
            }
        }
    }

    fn create_character_column(&self, current_scene: &crate::crisis::CrisisScene, story_state: &crate::crisis::GameState, error_messages: &mut Vec<String>) -> iced::widget::Column<'_, GameMessage, Theme, iced::Renderer> {
        let character_display = if let Some(ref char_image) = current_scene.speaking_character_image {
            match char_image {
                crate::crisis::SpeakingCharacterImage::Single(char_path) => {
                    if let Some(char_file) = crate::crisis::PlayableCrises::get(char_path) {
                        let char_handle = iced::widget::image::Handle::from_bytes(char_file.data.as_ref().to_vec());
                        let char_img = Image::<iced::widget::image::Handle>::new(char_handle)
                            .width(Length::Fill);
                        container(char_img)
                            .width(Length::Fill)
                            .padding(20)
                            .align_x(Center)
                    } else {
                        self.add_scene_error(error_messages, &story_state.current_scene, &format!("Character image file not found: {}", char_path));
                        container(Space::with_width(Length::Fill))
                    }
                }
                crate::crisis::SpeakingCharacterImage::Animation(image_paths) => {
                    if !image_paths.is_empty() {
                        let current_frame_index = self.animation_frame_index % image_paths.len();
                        let char_path = &image_paths[current_frame_index];
                        
                        if let Some(char_file) = crate::crisis::PlayableCrises::get(char_path) {
                            let char_handle = iced::widget::image::Handle::from_bytes(char_file.data.as_ref().to_vec());
                            let char_img = Image::<iced::widget::image::Handle>::new(char_handle)
                                .width(Length::Fill);
                            container(char_img)
                                .width(Length::Fill)
                                .padding(20)
                                .align_x(Center)
                        } else {
                            self.add_scene_error(error_messages, &story_state.current_scene, &format!("Character animation image file not found: {}", char_path));
                            container(Space::with_width(Length::Fill))
                        }
                    } else {
                        self.add_scene_error(error_messages, &story_state.current_scene, "Empty animation array for speaking_character_image");
                        container(Space::with_width(Length::Fill))
                    }
                }
            }
        } else {
            self.add_scene_error(error_messages, &story_state.current_scene, "No speaking_character_image defined");
            container(Space::with_width(Length::Fill))
        };

        column![
            Space::with_height(Length::Fill),
            character_display
        ]
        .width(Length::FillPortion(40))
        .height(Length::Fill)
    }

    fn create_error_display(&self, error_messages: &[String]) -> Container<'_, GameMessage, Theme, iced::Renderer> {
        let error_text = error_messages.join("; ");
        container(
            text(format!("Media Loading Errors: {}", error_text))
                .size(self.font_size_small())
                .color(iced::Color::from_rgb(0.8, 0.2, 0.2))
                .wrapping(iced::widget::text::Wrapping::Word)
        )
        .width(Length::Fill)
        .padding(10)
        .style(move |_theme: &Theme| {
            iced::widget::container::Style {
                background: Some(iced::Color::from_rgba(0.8, 0.2, 0.2, 0.1).into()),
                border: iced::border::rounded(4)
                    .color(iced::Color::from_rgb(0.8, 0.2, 0.2))
                    .width(1),
                ..iced::widget::container::Style::default()
            }
        })
    }

    fn add_scene_error(&self, error_messages: &mut Vec<String>, scene_name: &str, message: &str) {
        error_messages.push(format!("Scene '{}': {}", scene_name, message));
    }

    fn validate_background_audio(&self, current_scene: &crate::crisis::CrisisScene, story_state: &crate::crisis::GameState, error_messages: &mut Vec<String>) {
        if let Some(ref audio_path) = current_scene.background_audio {
            if crate::crisis::PlayableCrises::get(audio_path).is_none() {
                self.add_scene_error(error_messages, &story_state.current_scene, &format!("Background audio file not found: {}", audio_path));
            }
        } else {
            self.add_scene_error(error_messages, &story_state.current_scene, "No background_audio defined");
        }
    }

    fn render_scene_not_found(&self, story_state: &crate::crisis::GameState) -> iced::Element<'_, GameMessage> {
        container(
            column![
                text(format!("{} '{}' {}", 
                    TranslationUtils::translate(crate::translations::TranslationKey::SceneNotFound, &story_state.language).replace("!", ""),
                    story_state.current_scene,
                    "!"
                )).size(self.font_size_large()),
                button(text(TranslationUtils::translate(crate::translations::TranslationKey::ReturnToMenu, &story_state.language)))
                    .on_press(GameMessage::Game_RestartRequested)
                    .padding(10)
            ]
            .spacing(20)
            .align_x(Center)
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}