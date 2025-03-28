#![allow(unused_imports)]
/**
 *  full-crisis - An emergency-response simulator videogame
 *  Copyright (C) 2025  Jeffrey McAteer <jeffrey@jmcateer.com>
 *
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; version 2 of the License ONLY.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License along
 *  with this program; if not, write to the Free Software Foundation, Inc.,
 *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */

use macroquad::{
    prelude::*,
    ui::{hash, root_ui, widgets::InputText},
    window::request_new_screen_size,
};

// TODO move beyond hello world
#[macroquad::main(window_conf)]
async fn main() {
    let mut user_text_input = String::new();
    loop {
        clear_background(RED);

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);

        draw_text("Hello, Macroquad!", 20.0, 20.0, 30.0, DARKGRAY);

        let window_id = hash!();
        root_ui().window(
            window_id,
            vec2(12.0, 40.0),
            vec2(screen_width() * 0.75, 48.0),
            |ui| {
                let input_text_id = hash!();
                InputText::new(input_text_id)
                    .label("")
                    .size(vec2(screen_width() - 4.0, 48.0 - 4.0))
                    .ui(ui, &mut user_text_input);
            },
        );


        next_frame().await
    }
}


fn window_conf() -> Conf {
    Conf {
        window_title: "Full-Crisis".to_owned(),
        fullscreen: false,
        ..Default::default()
    }
}
