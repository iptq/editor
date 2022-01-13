use anyhow::Result;
use ggez::{
    graphics::{Color, DrawParam},
    Context,
};
use libosu::prelude::*;

use crate::{beatmap::STACK_DISTANCE, hitobject::HitObjectExt};

use super::{Game, PLAYFIELD_BOUNDS};

pub struct DrawInfo<'a> {
    hit_object: &'a HitObjectExt,
    fade_opacity: f64,
    end_time: f64,
    color: Color,
    /// Whether or not the circle (slider head for sliders) should appear to have already
    /// been hit in the editor after the object's time has already come.
    circle_is_hit: bool,
}

impl Game {
    pub(super) fn draw_hitobjects(&mut self, ctx: &mut Context, current_time: f64) -> Result<()> {
        // figure out what objects will be visible on the screen at the current instant
        // 1.5 cus editor so people can see objects for longer durations
        let mut playfield_hitobjects = Vec::new();
        let preempt = 1.5
            * self
                .beatmap
                .inner
                .difficulty
                .approach_preempt()
                .as_seconds();
        let fade_in = 1.5
            * self
                .beatmap
                .inner
                .difficulty
                .approach_fade_time()
                .as_seconds();
        let fade_out_time = 0.75; // TODO: figure out what this number actually is
        let fade_out_offset = 0.0; // TODO: figure out what this number actually is

        // TODO: tighten this loop even more by binary searching for the start of the timeline and
        // playfield hitobjects rather than looping through the entire beatmap, better yet, just
        // keeping track of the old index will probably be much faster
        for ho in self.beatmap.hit_objects.iter().rev() {
            let ho_time = ho.inner.start_time.as_seconds();
            let color = self.combo_colors[ho.color_idx];

            // draw in timeline
            self.draw_hitobject_to_timeline(ctx, current_time, ho)?;

            // draw hitobject in playfield
            let end_time;
            match ho.inner.kind {
                HitObjectKind::Circle => end_time = ho_time,
                HitObjectKind::Slider(_) => {
                    let duration = self.beatmap.inner.get_slider_duration(&ho.inner).unwrap();
                    end_time = ho_time + duration;
                }
                HitObjectKind::Spinner(SpinnerInfo {
                    end_time: spinner_end,
                }) => end_time = spinner_end.as_seconds(),
            };

            let fade_opacity = if current_time <= ho_time - fade_in {
                // before the hitobject's time arrives, it fades in
                // TODO: calculate ease
                (current_time - (ho_time - preempt)) / fade_in
            } else if current_time < ho_time + fade_out_time {
                // while the object is on screen the opacity should be 1
                1.0
            } else if current_time < end_time + fade_out_offset {
                // after the hitobject's time, it fades out
                // TODO: calculate ease
                ((end_time + fade_out_offset) - current_time) / fade_out_time
            } else {
                // completely faded out
                0.0
            };
            let circle_is_hit = current_time > ho_time;

            if ho_time - preempt <= current_time && current_time <= end_time + fade_out_offset {
                playfield_hitobjects.push(DrawInfo {
                    hit_object: ho,
                    fade_opacity,
                    end_time,
                    color,
                    circle_is_hit,
                });
            }
        }

        let cs_scale = PLAYFIELD_BOUNDS.w / 640.0;
        let osupx_scale_x = PLAYFIELD_BOUNDS.w / 512.0;
        let osupx_scale_y = PLAYFIELD_BOUNDS.h / 384.0;
        let cs_osupx = self.beatmap.inner.difficulty.circle_size_osupx();
        let cs_real = cs_osupx * cs_scale;

        for draw_info in playfield_hitobjects.iter() {
            let ho = draw_info.hit_object;
            let ho_time = ho.inner.start_time.as_seconds();
            let stacking = ho.stacking as f32 * STACK_DISTANCE as f32;
            let pos = [
                PLAYFIELD_BOUNDS.x + osupx_scale_x * (ho.inner.pos.x as f32 - stacking),
                PLAYFIELD_BOUNDS.y + osupx_scale_y * (ho.inner.pos.y as f32 - stacking),
            ];
            let mut color = draw_info.color;
            color.a = 0.6 * draw_info.fade_opacity as f32;

            let mut slider_info = None;
            if let HitObjectKind::Slider(info) = &ho.inner.kind {
                let mut control_points = vec![ho.inner.pos];
                control_points.extend(&info.control_points);

                Game::render_slider_body(
                    &mut self.slider_cache,
                    info,
                    control_points.as_ref(),
                    ctx,
                    PLAYFIELD_BOUNDS,
                    &self.beatmap.inner,
                    color,
                )?;
                slider_info = Some((info, control_points));

                let end_pos = ho.inner.end_pos();
                let end_pos = [
                    PLAYFIELD_BOUNDS.x + osupx_scale_x * end_pos.x as f32,
                    PLAYFIELD_BOUNDS.y + osupx_scale_y * end_pos.y as f32,
                ];
                self.skin.hitcircle.draw(
                    ctx,
                    (cs_real * 2.0, cs_real * 2.0),
                    DrawParam::default().dest(end_pos).color(color),
                )?;
                self.skin.hitcircleoverlay.draw(
                    ctx,
                    (cs_real * 2.0, cs_real * 2.0),
                    DrawParam::default().dest(end_pos),
                )?;
            }

            // draw main hitcircle
            let faded_color = Color::new(1.0, 1.0, 1.0, 0.6 * draw_info.fade_opacity as f32);
            self.skin.hitcircle.draw(
                ctx,
                (cs_real * 2.0, cs_real * 2.0),
                DrawParam::default().dest(pos).color(color),
            )?;
            self.skin.hitcircleoverlay.draw(
                ctx,
                (cs_real * 2.0, cs_real * 2.0),
                DrawParam::default().dest(pos).color(faded_color),
            )?;

            // draw numbers
            self.draw_numbers_on_circle(ctx, ho.number, pos, cs_real, faded_color)?;

            if let Some((info, control_points)) = slider_info {
                let spline = self.slider_cache.get(&control_points).unwrap();
                Game::render_slider_wireframe(ctx, &control_points, PLAYFIELD_BOUNDS, faded_color)?;

                if current_time > ho_time && current_time < draw_info.end_time {
                    let elapsed_time = current_time - ho_time;
                    let total_duration = draw_info.end_time - ho_time;
                    let single_duration = total_duration / info.num_repeats as f64;
                    let finished_repeats = (elapsed_time / single_duration).floor();
                    let this_repeat_time = elapsed_time - finished_repeats * single_duration;
                    let mut travel_percent = this_repeat_time / single_duration;

                    // reverse direction on odd trips
                    if finished_repeats as u32 % 2 == 1 {
                        travel_percent = 1.0 - travel_percent;
                    }
                    let travel_length = travel_percent * info.pixel_length;
                    let pos = spline.point_at_length(travel_length);
                    let ball_pos = [
                        PLAYFIELD_BOUNDS.x + osupx_scale_x * pos.x as f32,
                        PLAYFIELD_BOUNDS.y + osupx_scale_y * pos.y as f32,
                    ];
                    self.skin.sliderb.draw_frame(
                        ctx,
                        (cs_real * 1.8, cs_real * 1.8),
                        DrawParam::default().dest(ball_pos).color(color),
                        (travel_percent / 0.25) as usize,
                    )?;
                }
            }

            if current_time < ho_time {
                let time_diff = ho_time - current_time;
                let approach_r = cs_real * (1.0 + 2.0 * time_diff as f32 / 0.75);
                self.skin.approachcircle.draw(
                    ctx,
                    (approach_r * 2.0, approach_r * 2.0),
                    DrawParam::default().dest(pos).color(color),
                )?;
            }
        }

        Ok(())
    }
}
