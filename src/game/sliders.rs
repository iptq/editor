use anyhow::Result;
use ggez::{
    graphics::{
        self, Color, DrawMode, DrawParam, FillOptions, LineCap, LineJoin, Mesh, Rect, StrokeOptions,
    },
    nalgebra::Point2,
    Context,
};
use libosu::{
    beatmap::Beatmap,
    hitobject::{HitObject, HitObjectKind},
    spline::Spline,
};

use crate::game::SliderCache;

pub fn render_slider<'a>(
    slider_cache: &'a mut SliderCache,
    ctx: &mut Context,
    rect: Rect,
    beatmap: &Beatmap,
    slider: &HitObject,
    color: Color,
) -> Result<&'a Spline> {
    let mut control_points = vec![slider.pos];
    let slider_info = match &slider.kind {
        HitObjectKind::Slider(info) => info,
        _ => unreachable!("retard"),
    };
    control_points.extend(&slider_info.control_points);

    let spline = if slider_cache.contains_key(&control_points) {
        slider_cache.get(&control_points).unwrap()
    } else {
        let new_spline =
            Spline::from_control(slider_info.kind, &control_points, slider_info.pixel_length);
        slider_cache.insert(control_points.clone(), new_spline);
        slider_cache.get(&control_points).unwrap()
    };

    let cs_scale = rect.w / 640.0;
    let osupx_scale_x = rect.w as f64 / 512.0;
    let osupx_scale_y = rect.h as f64 / 384.0;
    let cs_osupx = beatmap.difficulty.circle_size_osupx() as f64;
    let cs_real = cs_osupx * cs_scale as f64;

    let points_mapped = control_points
        .iter()
        .map(|point| {
            let (x, y) = (point.0 as f64, point.1 as f64);
            let x2 = rect.x as f64 + osupx_scale_x * x;
            let y2 = rect.y as f64 + osupx_scale_y * y;
            [x2 as f32, y2 as f32].into()
        })
        .collect::<Vec<Point2<_>>>();

    let (mut boundx, mut boundy, mut boundw, mut boundh) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let spline_mapped = spline
        .spline_points
        .iter()
        .map(|point| {
            let (x, y) = (point.0, point.1);
            boundx = boundx.min(x - cs_osupx);
            boundy = boundy.min(y - cs_osupx);
            boundw = boundw.max(x + cs_osupx - boundx);
            boundh = boundh.max(y + cs_osupx - boundy);

            let x2 = rect.x as f64 + osupx_scale_x * x;
            let y2 = rect.y as f64 + osupx_scale_y * y;
            [x2 as f32, y2 as f32].into()
        })
        .collect::<Vec<Point2<f32>>>();

    let opts = StrokeOptions::default()
        .with_line_cap(LineCap::Round)
        .with_line_join(LineJoin::Round)
        .with_line_width(cs_real as f32 * 2.0);
    let body = Mesh::new_polyline(ctx, DrawMode::Stroke(opts), &spline_mapped, color)?;
    graphics::draw(ctx, &body, DrawParam::default())?;

    let frame = Mesh::new_polyline(
        ctx,
        DrawMode::Stroke(StrokeOptions::default()),
        &points_mapped,
        graphics::WHITE,
    )?;
    graphics::draw(ctx, &frame, DrawParam::default())?;
    for point in points_mapped {
        let size = 5.0;
        let rect = Rect::new(point.x - size, point.y - size, size * 2.0, size * 2.0);
        let rect = Mesh::new_rectangle(
            ctx,
            DrawMode::Fill(FillOptions::default()),
            rect,
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &rect, DrawParam::default())?;
    }

    Ok(spline)
}
