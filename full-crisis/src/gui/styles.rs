use iced::Theme;

pub fn menu_right_box_style(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();

    iced::widget::container::Style {
        background: Some(palette.background.weak.color.into()),
        border: iced::border::rounded(12)
            .color(palette.primary.weak.color)
            .width(2),
        ..iced::widget::container::Style::default()
    }
}

pub fn focused_button_style(is_focused: bool) -> impl Fn(&Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |theme: &Theme, status: iced::widget::button::Status| {
        let palette = theme.extended_palette();
        let mut style = match status {
            iced::widget::button::Status::Active => iced::widget::button::Style {
                background: Some(palette.primary.base.color.into()),
                text_color: palette.primary.base.text,
                border: iced::border::rounded(4),
                ..iced::widget::button::Style::default()
            },
            iced::widget::button::Status::Hovered => iced::widget::button::Style {
                background: Some(palette.primary.strong.color.into()),
                text_color: palette.primary.strong.text,
                border: iced::border::rounded(4),
                ..iced::widget::button::Style::default()
            },
            iced::widget::button::Status::Pressed => iced::widget::button::Style {
                background: Some(palette.primary.base.color.into()),
                text_color: palette.primary.base.text,
                border: iced::border::rounded(4),
                ..iced::widget::button::Style::default()
            },
            iced::widget::button::Status::Disabled => iced::widget::button::Style {
                background: Some(palette.background.weak.color.into()),
                text_color: palette.background.strong.text,
                border: iced::border::rounded(4)
                    .color(palette.background.strong.color)
                    .width(1),
                ..iced::widget::button::Style::default()
            },
        };

        // Add focus outline without affecting layout
        if is_focused {
            style.border = iced::border::rounded(4)
                .color(iced::Color::from_rgb(1.0, 1.0, 1.0)) // Bright white focus outline
                .width(3);
        }

        style
    }
}

pub fn focused_text_input_style(is_focused: bool, is_actively_focused: bool) -> impl Fn(&Theme, iced::widget::text_input::Status) -> iced::widget::text_input::Style {
    move |theme: &Theme, status: iced::widget::text_input::Status| {
        let palette = theme.extended_palette();
        let mut style = match status {
            iced::widget::text_input::Status::Active => iced::widget::text_input::Style {
                background: palette.background.base.color.into(),
                border: iced::border::rounded(4)
                    .color(palette.primary.weak.color)
                    .width(1),
                icon: palette.background.weak.text,
                placeholder: palette.background.strong.text,
                value: palette.background.base.text,
                selection: palette.primary.weak.color,
            },
            iced::widget::text_input::Status::Hovered => iced::widget::text_input::Style {
                background: palette.background.base.color.into(),
                border: iced::border::rounded(4)
                    .color(palette.primary.strong.color)
                    .width(1),
                icon: palette.background.weak.text,
                placeholder: palette.background.strong.text,
                value: palette.background.base.text,
                selection: palette.primary.weak.color,
            },
            iced::widget::text_input::Status::Focused { is_hovered: _ } => iced::widget::text_input::Style {
                background: palette.background.base.color.into(),
                border: iced::border::rounded(4)
                    .color(palette.primary.base.color)
                    .width(2),
                icon: palette.background.weak.text,
                placeholder: palette.background.strong.text,
                value: palette.background.base.text,
                selection: palette.primary.weak.color,
            },
            iced::widget::text_input::Status::Disabled => iced::widget::text_input::Style {
                background: palette.background.weak.color.into(),
                border: iced::border::rounded(4)
                    .color(palette.background.strong.color)
                    .width(1),
                icon: palette.background.strong.text,
                placeholder: palette.background.weak.text,
                value: palette.background.strong.text,
                selection: palette.primary.weak.color,
            },
        };
        
        // Add focus outline
        if is_focused {
            if is_actively_focused {
                // Active input focus - bright blue with thicker border
                style.border = iced::border::rounded(4)
                    .color(iced::Color::from_rgb(0.2, 0.6, 1.0))
                    .width(4);
                style.background = palette.primary.weak.color.into();
            } else {
                // Navigation focus - white outline
                style.border = iced::border::rounded(4)
                    .color(iced::Color::from_rgb(1.0, 1.0, 1.0))
                    .width(3);
            }
        }
        
        style
    }
}

pub fn focused_pick_list_style(is_focused: bool) -> impl Fn(&Theme, iced::widget::pick_list::Status) -> iced::widget::pick_list::Style {
    move |theme: &Theme, status: iced::widget::pick_list::Status| {
        let palette = theme.extended_palette();
        let mut style = match status {
            iced::widget::pick_list::Status::Active => iced::widget::pick_list::Style {
                background: palette.background.weak.color.into(),
                text_color: palette.background.weak.text,
                placeholder_color: palette.background.strong.text,
                border: iced::border::rounded(4)
                    .color(palette.background.strong.color)
                    .width(1),
                handle_color: palette.background.weak.text,
            },
            iced::widget::pick_list::Status::Hovered => iced::widget::pick_list::Style {
                background: palette.background.base.color.into(),
                text_color: palette.background.base.text,
                placeholder_color: palette.background.strong.text,
                border: iced::border::rounded(4)
                    .color(palette.primary.weak.color)
                    .width(1),
                handle_color: palette.background.weak.text,
            },
            iced::widget::pick_list::Status::Opened { is_hovered: _ } => iced::widget::pick_list::Style {
                background: palette.background.base.color.into(),
                text_color: palette.background.base.text,
                placeholder_color: palette.background.strong.text,
                border: iced::border::rounded(4)
                    .color(palette.primary.base.color)
                    .width(2),
                handle_color: palette.background.weak.text,
            },
        };
        
        // Add focus outline
        if is_focused {
            style.border = iced::border::rounded(4)
                .color(iced::Color::from_rgb(1.0, 1.0, 1.0))
                .width(3);
        }
        
        style
    }
}

pub fn focused_toggler_style(is_focused: bool) -> impl Fn(&Theme, iced::widget::toggler::Status) -> iced::widget::toggler::Style {
    move |theme: &Theme, status: iced::widget::toggler::Status| {
        let palette = theme.extended_palette();
        let mut style = match status {
            iced::widget::toggler::Status::Active { is_toggled } => iced::widget::toggler::Style {
                background: if is_toggled {
                    palette.primary.strong.color
                } else {
                    palette.background.strong.color
                },
                background_border_width: 1.0,
                background_border_color: palette.background.strong.color,
                foreground: palette.background.base.color,
                foreground_border_width: 0.0,
                foreground_border_color: iced::Color::TRANSPARENT,
            },
            iced::widget::toggler::Status::Hovered { is_toggled } => iced::widget::toggler::Style {
                background: if is_toggled {
                    palette.primary.base.color
                } else {
                    palette.background.weak.color
                },
                background_border_width: 1.0,
                background_border_color: palette.primary.weak.color,
                foreground: palette.background.base.color,
                foreground_border_width: 0.0,
                foreground_border_color: iced::Color::TRANSPARENT,
            },
            iced::widget::toggler::Status::Disabled => iced::widget::toggler::Style {
                background: palette.background.weak.color,
                background_border_width: 1.0,
                background_border_color: palette.background.strong.color,
                foreground: palette.background.strong.color,
                foreground_border_width: 0.0,
                foreground_border_color: iced::Color::TRANSPARENT,
            },
        };
        
        // Add focus outline
        if is_focused {
            style.background_border_width = 3.0;
            style.background_border_color = iced::Color::from_rgb(1.0, 1.0, 1.0);
        }
        
        style
    }
}

pub fn focused_slider_style(is_focused: bool) -> impl Fn(&Theme, iced::widget::slider::Status) -> iced::widget::slider::Style {
    move |theme: &Theme, status: iced::widget::slider::Status| {
        let palette = theme.extended_palette();
        let mut style = match status {
            iced::widget::slider::Status::Active => iced::widget::slider::Style {
                rail: iced::widget::slider::Rail {
                    backgrounds: (palette.background.strong.color.into(), palette.primary.base.color.into()),
                    width: 4.0,
                    border: iced::border::rounded(2.0).width(0.0),
                },
                handle: iced::widget::slider::Handle {
                    shape: iced::widget::slider::HandleShape::Circle { radius: 8.0 },
                    background: palette.primary.base.color.into(),
                    border_color: palette.background.base.color,
                    border_width: 2.0,
                },
            },
            iced::widget::slider::Status::Hovered => iced::widget::slider::Style {
                rail: iced::widget::slider::Rail {
                    backgrounds: (palette.background.strong.color.into(), palette.primary.strong.color.into()),
                    width: 4.0,
                    border: iced::border::rounded(2.0).width(0.0),
                },
                handle: iced::widget::slider::Handle {
                    shape: iced::widget::slider::HandleShape::Circle { radius: 9.0 },
                    background: palette.primary.strong.color.into(),
                    border_color: palette.background.base.color,
                    border_width: 2.0,
                },
            },
            iced::widget::slider::Status::Dragged => iced::widget::slider::Style {
                rail: iced::widget::slider::Rail {
                    backgrounds: (palette.background.strong.color.into(), palette.primary.base.color.into()),
                    width: 4.0,
                    border: iced::border::rounded(2.0).width(0.0),
                },
                handle: iced::widget::slider::Handle {
                    shape: iced::widget::slider::HandleShape::Circle { radius: 8.0 },
                    background: palette.primary.base.color.into(),
                    border_color: palette.background.base.color,
                    border_width: 2.0,
                },
            },
        };
        
        // Add focus indication by making handle more prominent
        if is_focused {
            style.handle.shape = iced::widget::slider::HandleShape::Circle { radius: 10.0 };
            style.handle.border_color = iced::Color::from_rgb(1.0, 1.0, 1.0);
            style.handle.border_width = 3.0;
        }
        
        style
    }
}
