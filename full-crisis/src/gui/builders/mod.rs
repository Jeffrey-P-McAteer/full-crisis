pub mod menu_components;

use crate::gui::types::*;
use iced::widget::{Container, Text, Button};
use iced::{Element, Length, Padding};

/// Common UI patterns and utilities
pub struct UIHelpers;

impl UIHelpers {
    /// Create a standardized section container
    pub fn section_container<'a>(content: Element<'a, GameMessage>) -> Container<'a, GameMessage> {
        Container::new(content)
            .padding(Padding::from(20))
            .width(Length::Fill)
    }
    
    /// Create a button with consistent font scaling
    pub fn scaled_button<'a>(
        text: &'a str,
        message: GameMessage,
        font_scale: f32
    ) -> Button<'a, GameMessage> {
        let font_size = 22.0 * font_scale;
        Button::new(Text::new(text).size(font_size))
            .on_press(message)
            .padding(Padding::from(10))
    }
    
    /// Create text with consistent font scaling  
    pub fn scaled_text<'a>(text: &'a str, font_scale: f32) -> Text<'a> {
        let font_size = 18.0 * font_scale;
        Text::new(text).size(font_size)
    }
    
    /// Create a title with consistent font scaling
    pub fn scaled_title<'a>(text: &'a str, font_scale: f32) -> Text<'a> {
        let font_size = 28.0 * font_scale;
        Text::new(text).size(font_size)
    }
}