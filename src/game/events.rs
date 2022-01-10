use ggez::{
    event::{EventHandler, KeyCode, KeyMods, MouseButton},
    Context, GameError, GameResult,
};
use libosu::timing::{TimingPoint, TimingPointKind};

use crate::utils::rect_contains;

use super::{Game, Tool};

impl EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _: &mut Context,
        x: f32,
        y: f32,
        _: f32,
        _: f32,
    ) -> GameResult {
        self.mouse_pos = (x, y);
        self.imgui.update_mouse_pos(x, y);
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _: &mut Context,
        btn: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        self.imgui.update_mouse_down(btn);
        // TODO: figure out if the UI handled anything, and then whether or not to keep going into
        // letting the rest of the code handle the mouse press or not
        match btn {
            MouseButton::Left => {
                use super::seeker::BOUNDS;
                if rect_contains(&BOUNDS, x, y) {
                    let jump_percent = (x - BOUNDS.x) / BOUNDS.w;
                    if let Some(song) = &self.song {
                        let pos = jump_percent as f64 * song.length().unwrap();
                        song.set_position(pos);
                    }
                }
                self.left_drag_start = Some((x, y));
            }
            MouseButton::Right => self.right_drag_start = Some((x, y)),
            _ => {}
        }
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _: &mut Context,
        btn: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        self.imgui.update_mouse_up(btn);
        match btn {
            MouseButton::Left => {
                if let Some((px, py)) = self.left_drag_start {
                    if px == x && py == y {
                        self.handle_click(MouseButton::Left, x, y).unwrap();
                    }
                }
                self.left_drag_start = None;
            }
            MouseButton::Right => {
                if let Some((px, py)) = self.right_drag_start {
                    if px == x && py == y {
                        self.handle_click(MouseButton::Right, x, y).unwrap();
                    }
                }
                self.right_drag_start = None;
            }
            _ => {}
        }
        Ok(())
    }

    fn mouse_wheel_event(&mut self, _: &mut Context, x: f32, y: f32) -> GameResult {
        self.seek_by_steps(-y as i32);
        Ok(())
    }

    fn key_up_event(&mut self, _: &mut Context, keycode: KeyCode, keymods: KeyMods) -> GameResult {
        use KeyCode::*;
        self.imgui.update_key_up(keycode, keymods);

        match keycode {
            Space => self.toggle_playing(),
            Colon => {}
            G => {
                self.toggle_grid();
            }
            _ => {}
        };

        Ok(())
    }

    fn key_down_event(
        &mut self,
        _: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        _: bool,
    ) -> GameResult {
        use KeyCode::*;
        self.imgui.update_key_down(keycode, keymods);

        self.keymap.insert(keycode);
        match keycode {
            Key1 => self.switch_tool_to(Tool::Select),
            Key2 => self.switch_tool_to(Tool::Circle),
            Key3 => self.switch_tool_to(Tool::Slider),

            Left => {
                if let Some(TimingPoint {
                    kind: TimingPointKind::Uninherited(info),
                    ..
                }) = &self.current_uninherited_timing_point
                {
                    let steps = -if keymods.contains(KeyMods::SHIFT) {
                        info.meter as i32
                    } else {
                        1
                    };
                    self.seek_by_steps(steps).unwrap();
                }
            }
            Right => {
                if let Some(TimingPoint {
                    kind: TimingPointKind::Uninherited(info),
                    ..
                }) = &self.current_uninherited_timing_point
                {
                    let steps = if keymods.contains(KeyMods::SHIFT) {
                        info.meter as i32
                    } else {
                        1
                    };
                    self.seek_by_steps(steps).unwrap();
                }
            }
            _ => {}
        };

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        if let Err(err) = self.draw_helper(ctx) {
            return Err(GameError::RenderError(err.to_string()));
        };
        Ok(())
    }
}
