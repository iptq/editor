use anyhow::Result;
use ggez::Context;
use imgui::{Condition, MenuItem, Slider, TabBar, TabItem, Window};

use super::Game;

#[derive(Debug, Default)]
pub struct UiState {
    song_setup_opened: bool,
    song_setup_artist: String,
    song_setup_romanized_artist: String,
    song_setup_title: String,
    song_setup_romanized_title: String,
    song_setup_mapper: String,
    song_setup_source: String,
    song_setup_tags: String,
    song_setup_hp: f64,
}

impl Game {
    pub(super) fn draw_ui(&mut self, ctx: &mut Context, state: &mut UiState) -> Result<()> {
        self.imgui.render(ctx, 1.0, |ui| {
            // menu bar
            if let Some(menu_bar) = ui.begin_main_menu_bar() {
                if let Some(menu) = ui.begin_menu("File") {
                    MenuItem::new("Save <C-s>").build(ui);
                    MenuItem::new("Create Difficulty").build(ui);
                    ui.separator();
                    MenuItem::new("Song Setup").build_with_ref(ui, &mut state.song_setup_opened);
                    MenuItem::new("Revert to Saved <C-l>").build(ui);
                    ui.separator();
                    MenuItem::new("Open Song Folder").build(ui);
                    MenuItem::new("Exit <Esc>").build(ui);
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("Edit") {
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("View") {
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("Compose") {
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("Design") {
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("Timing") {
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("Web") {
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("Help") {
                    menu.end();
                }
                menu_bar.end();
            }

            if state.song_setup_opened {
                Window::new("Song Setup")
                    .opened(&mut false)
                    .collapsible(false)
                    .always_auto_resize(true)
                    .size([180.0, 240.0], Condition::Appearing)
                    .build(&ui, || {
                        TabBar::new("song_setup").build(&ui, || {
                            TabItem::new("General").build(&ui, || {
                                ui.group(|| {
                                    ui.input_text("Artist", &mut state.song_setup_artist)
                                        .build();
                                    ui.input_text(
                                        "Romanized Artist",
                                        &mut state.song_setup_romanized_artist,
                                    )
                                    .build();
                                    ui.input_text("Title", &mut state.song_setup_title).build();
                                    ui.input_text(
                                        "Romanized Title",
                                        &mut state.song_setup_romanized_title,
                                    )
                                    .build();
                                    ui.input_text("Mapper", &mut state.song_setup_mapper)
                                        .build();
                                    ui.input_text("Source", &mut state.song_setup_source)
                                        .build();
                                    ui.input_text("Tags", &mut state.song_setup_tags).build();
                                });
                            });
                            TabItem::new("Difficulty").build(&ui, || {
                                Slider::new("HP Drain Rate", 0.0, 10.0)
                                    .display_format("%.1f")
                                    .build(ui, &mut state.song_setup_hp);
                                Slider::new("Circle Size", 0.0, 10.0)
                                    .display_format("%.1f")
                                    .build(ui, &mut state.song_setup_hp);
                            });
                            TabItem::new("Audio").build(&ui, || {});
                            TabItem::new("Colors").build(&ui, || {});
                            TabItem::new("Design").build(&ui, || {});
                            TabItem::new("Advanced").build(&ui, || {});
                        });

                        ui.button("OK");
                        ui.same_line();
                        ui.button("Cancel");
                    });
            }
        });

        Ok(())
    }
}
