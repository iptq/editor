use anyhow::Result;
use ggez::Context;
use imgui::{Condition, MenuItem, TabBar, TabItem, Window};

use super::Game;

#[derive(Debug, Default)]
pub struct UiState {
    song_setup_selected: bool,
    song_setup_artist: String,
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
                    MenuItem::new("Song Setup").build_with_ref(ui, &mut state.song_setup_selected);
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

            if state.song_setup_selected {
                Window::new("Song Setup")
                    .size([80.0, 120.0], Condition::Appearing)
                    .build(&ui, || {
                        TabBar::new("song_setup").build(&ui, || {
                            TabItem::new("General").build(&ui, || {
                                ui.group(|| {
                                    ui.text("Artist");
                                    ui.same_line();
                                    ui.input_text("", &mut state.song_setup_artist).build();
                                });
                            });
                            TabItem::new("Difficulty").build(&ui, || {});
                            TabItem::new("Audio").build(&ui, || {});
                            TabItem::new("Colors").build(&ui, || {});
                            TabItem::new("Design").build(&ui, || {});
                            TabItem::new("Advanced").build(&ui, || {});
                        });
                    });
            }
        });

        Ok(())
    }
}
