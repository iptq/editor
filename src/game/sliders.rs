use anyhow::Result;
use ggez::{
    graphics::{
        self, Canvas, Color, DrawMode, DrawParam, FillOptions, LineCap, LineJoin, Mesh, Rect,
        StrokeOptions,
    },
    mint::Point2,
    Context,
};
use libosu::{beatmap::Beatmap, hitobject::SliderInfo, math::Point, spline::Spline};

use super::{Game, SliderCache};

impl Game {
    pub fn render_spline(
        ctx: &mut Context,
        beatmap: &Beatmap,
        spline: &Spline,
        rect: Rect,
        color: Color,
    ) -> Result<()> {
        let cs_scale = rect.w / 640.0;
        let osupx_scale_x = rect.w as f64 / 512.0;
        let osupx_scale_y = rect.h as f64 / 384.0;
        let cs_osupx = beatmap.difficulty.circle_size_osupx() as f64;
        let cs_real = cs_osupx * cs_scale as f64;

        let (mut boundx, mut boundy, mut boundw, mut boundh) = (f64::MAX, f64::MAX, 0.0f64, 0.0f64);
        let spline_mapped = spline
            .spline_points
            .iter()
            .map(|point| {
                let x2 = rect.x as f64 + osupx_scale_x * point.x;
                let y2 = rect.y as f64 + osupx_scale_y * point.y;
                boundx = boundx.min(x2 - cs_osupx);
                boundy = boundy.min(y2 - cs_osupx);
                boundw = boundw.max(x2 + cs_osupx - boundx);
                boundh = boundh.max(y2 + cs_osupx - boundy);
                [x2 as f32, y2 as f32].into()
            })
            .collect::<Vec<Point2<f32>>>();

        // draw slider border
        let canvas = Canvas::with_window_size(ctx)?;
        let opts = StrokeOptions::default()
            .with_line_cap(LineCap::Round)
            .with_line_join(LineJoin::Round)
            .with_line_width(cs_real as f32 * 2.0);
        let body = Mesh::new_polyline(
            ctx,
            DrawMode::Stroke(opts),
            spline_mapped.as_ref(),
            Color::WHITE,
        )?;
        graphics::set_canvas(ctx, Some(&canvas));
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 0.0));
        graphics::draw(ctx, &body, DrawParam::default())?;
        graphics::set_canvas(ctx, None);
        let mut border_color = Color::WHITE;
        border_color.a = color.a;
        graphics::draw(ctx, &canvas, DrawParam::default().color(border_color))?;

        // draw slider body
        let canvas = Canvas::with_window_size(ctx)?;
        let opts = StrokeOptions::default()
            .with_line_cap(LineCap::Round)
            .with_line_join(LineJoin::Round)
            .with_line_width(cs_real as f32 * 1.8);
        let body = Mesh::new_polyline(
            ctx,
            DrawMode::Stroke(opts),
            spline_mapped.as_ref(),
            Color::WHITE,
        )?;
        graphics::set_canvas(ctx, Some(&canvas));
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 0.0));
        graphics::draw(ctx, &body, DrawParam::default())?;
        graphics::set_canvas(ctx, None);
        graphics::draw(ctx, &canvas, DrawParam::default().color(color))?;

        Ok(())
    }

    pub fn render_slider_body<'a>(
        slider_cache: &'a mut SliderCache,
        slider_info: &SliderInfo,
        control_points: &[Point<i32>],
        ctx: &mut Context,
        rect: Rect,
        beatmap: &Beatmap,
        color: Color,
    ) -> Result<()> {
        debug!(
            "Rendering slider body with control points {:?}",
            control_points
        );

        if control_points.len() < 2
            || (control_points.len() == 2 && control_points[0] == control_points[1])
        {
            debug!("Slider too short, not rendering!");
            return Ok(());
        }

        let spline = if slider_cache.contains_key(control_points) {
            slider_cache.get(control_points).expect("just checked")
        } else {
            let new_spline = Spline::from_control(
                slider_info.kind,
                control_points,
                Some(slider_info.pixel_length),
            );
            slider_cache.insert(control_points.to_vec(), new_spline);
            slider_cache.get(control_points).expect("just inserted it")
        };
        debug!("spline length: {}", spline.spline_points.len());

        if spline.spline_points.len() < 2
            || (spline.spline_points.len() == 2
                && spline.spline_points[0] == spline.spline_points[1])
        {
            debug!("Slider too short, not rendering!");
            return Ok(());
        }

        Game::render_spline(ctx, beatmap, spline, rect, color)
    }

    pub fn render_slider_wireframe(
        ctx: &mut Context,
        control_points: &[Point<i32>],
        rect: Rect,
    ) -> Result<()> {
        let osupx_scale_x = rect.w as f32 / 512.0;
        let osupx_scale_y = rect.h as f32 / 384.0;

        let points_mapped = control_points
            .iter()
            .map(|point| {
                let x2 = rect.x as f32 + osupx_scale_x * point.x as f32;
                let y2 = rect.y as f32 + osupx_scale_y * point.y as f32;
                [x2, y2].into()
            })
            .collect::<Vec<Point2<_>>>();

        // draw control points wireframe
        if control_points.len() > 1
            && !(control_points.len() == 2 && control_points[0] == control_points[1])
        {
            let frame = Mesh::new_polyline(
                ctx,
                DrawMode::Stroke(StrokeOptions::default()),
                &points_mapped,
                Color::WHITE,
            )?;
            graphics::draw(ctx, &frame, DrawParam::default())?;
        }

        // draw points on wireframe
        let mut i = 0;
        while i < points_mapped.len() {
            let fst = points_mapped[i];
            let mut color = Color::WHITE;
            if i < points_mapped.len() - 1 {
                let snd = points_mapped[i + 1];
                if fst.eq(&snd) {
                    i += 1;
                    color = Color::new(1.0, 0.0, 0.0, 1.0);
                }
            }
            let size = 5.0;
            let rect = Rect::new(fst.x - size, fst.y - size, size * 2.0, size * 2.0);
            let rect =
                Mesh::new_rectangle(ctx, DrawMode::Fill(FillOptions::default()), rect, color)?;
            graphics::draw(ctx, &rect, DrawParam::default())?;
            i += 1;
        }

        Ok(())
    }
}
