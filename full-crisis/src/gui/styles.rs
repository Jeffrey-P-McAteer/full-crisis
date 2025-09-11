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
