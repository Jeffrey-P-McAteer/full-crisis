use crate::gui::types::*;
use crate::gui::helpers::TranslationUtils;
use iced::widget::{Column, Row, Text, Button, PickList, TextInput, Slider, Checkbox};
use iced::{Element, Length, Padding};

/// Helper functions for common menu UI components
pub struct MenuComponents;

impl MenuComponents {
    /// Create a localized button with consistent styling
    pub fn localized_button<'a>(
        translation_key: crate::translations::TranslationKey,
        language: &str,
        message: GameMessage,
        font_scale: f32
    ) -> Button<'a, GameMessage> {
        let text = TranslationUtils::translate(translation_key, language);
        let font_size = 22.0 * font_scale;
        
        Button::new(Text::new(text).size(font_size))
            .on_press(message)
            .padding(Padding::from(10))
    }
    
    /// Create a labeled input field
    pub fn labeled_input<'a>(
        label: String,
        placeholder: String,
        value: &'a str,
        on_change: impl Fn(String) -> GameMessage + 'static,
        font_scale: f32
    ) -> Column<'a, GameMessage> {
        let label_size = 16.0 * font_scale;
        let input_size = 18.0 * font_scale;
        
        Column::new()
            .spacing(5)
            .push(Text::new(label).size(label_size))
            .push(
                TextInput::new(&placeholder, value)
                    .on_input(on_change)
                    .padding(Padding::from(8))
                    .size(input_size)
            )
    }
    
    /// Create a form section with title and content
    pub fn form_section<'a>(
        title: String,
        content: Element<'a, GameMessage>,
        font_scale: f32
    ) -> Column<'a, GameMessage> {
        let title_size = 24.0 * font_scale;
        
        Column::new()
            .spacing(15)
            .push(Text::new(title).size(title_size))
            .push(content)
    }
    
    /// Create a button row with consistent spacing
    pub fn button_row<'a>(buttons: Vec<Element<'a, GameMessage>>) -> Row<'a, GameMessage> {
        buttons.into_iter().fold(
            Row::new().spacing(10),
            |row, button| row.push(button)
        )
    }
}