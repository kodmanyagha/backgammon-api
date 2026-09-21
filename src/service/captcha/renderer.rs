use std::f32::consts::TAU;

use ab_glyph::{Font, FontRef, Outline, OutlineCurve, Point as FontPoint};
use anyhow::anyhow;
use once_cell::sync::Lazy;
use rand::{Rng, RngExt};
use tiny_skia::{FillRule, LineCap, Paint, Path, PathBuilder, Pixmap, Stroke, Transform};

const TEXT_HEIGHT_RATIO: f32 = 0.6;
const TEXT_MAX_WIDTH_RATIO: f32 = 0.92;
const GLYPH_SCALE_JITTER: f32 = 0.1;
const GLYPH_ROTATION_DEGREES: f32 = 20.0;
const GLYPH_SKEW: f32 = 0.2;
const GLYPH_VERTICAL_JITTER_RATIO: f32 = 0.07;
const GLYPH_SHADOW_OFFSET: f32 = 2.0;
const CONTOUR_JOIN_TOLERANCE: f32 = 0.5;
const BACKGROUND_BLOB_COUNT: usize = 3;
const NOISE_CURVES_BEHIND_TEXT: usize = 3;
const NOISE_CURVES_IN_FRONT_OF_TEXT: usize = 2;
const NOISE_ALPHA_BEHIND_TEXT: u8 = 190;
const NOISE_ALPHA_IN_FRONT_OF_TEXT: u8 = 120;
const SPECKLE_AREA_PER_DOT: u32 = 380;

static FONT: Lazy<FontRef<'static>> = Lazy::new(|| {
    FontRef::try_from_slice(include_bytes!("../../../assets/fonts/DejaVuSans-Bold.ttf"))
        .expect("embedded captcha font must be a valid TrueType font")
});

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImageSize {
    pub width: u32,
    pub height: u32,
}

pub fn render_text_png(text: &str, size: ImageSize, rng: &mut impl Rng) -> anyhow::Result<Vec<u8>> {
    let mut pixmap = new_pixmap(size)?;

    paint_background(&mut pixmap, rng);
    paint_noise_curves(&mut pixmap, NOISE_CURVES_BEHIND_TEXT, NOISE_ALPHA_BEHIND_TEXT, rng);
    paint_text(&mut pixmap, text, rng);
    paint_noise_curves(&mut pixmap, NOISE_CURVES_IN_FRONT_OF_TEXT, NOISE_ALPHA_IN_FRONT_OF_TEXT, rng);
    paint_speckles(&mut pixmap, rng);

    encode_opaque_png(&wave(&pixmap, rng)?)
}

fn encode_opaque_png(pixmap: &Pixmap) -> anyhow::Result<Vec<u8>> {
    let rgb_pixels: Vec<u8> = pixmap
        .data()
        .chunks_exact(4)
        .flat_map(|pixel| [pixel[0], pixel[1], pixel[2]])
        .collect();

    let mut png_bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut png_bytes, pixmap.width(), pixmap.height());
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::High);
    encoder.set_filter(png::Filter::Adaptive);
    encoder
        .write_header()
        .and_then(|mut writer| writer.write_image_data(&rgb_pixels))
        .map_err(|err| anyhow!("captcha png encoding failed: {err}"))?;

    Ok(png_bytes)
}

fn new_pixmap(size: ImageSize) -> anyhow::Result<Pixmap> {
    Pixmap::new(size.width, size.height)
        .ok_or_else(|| anyhow!("invalid captcha image size {}x{}", size.width, size.height))
}

fn paint_background(pixmap: &mut Pixmap, rng: &mut impl Rng) {
    let base_hue = rng.random_range(0.0..360.0);
    let [red, green, blue] = hsl_to_rgb(base_hue, 0.35, 0.9);
    pixmap.fill(tiny_skia::Color::from_rgba8(red, green, blue, 255));

    let width = pixmap.width() as f32;
    let height = pixmap.height() as f32;
    for _ in 0..BACKGROUND_BLOB_COUNT {
        let [red, green, blue] = hsl_to_rgb(rng.random_range(0.0..360.0), 0.4, 0.8);
        let Some(blob) = PathBuilder::from_circle(
            rng.random_range(0.0..width),
            rng.random_range(0.0..height),
            rng.random_range(height * 0.3..height * 0.7),
        ) else {
            continue;
        };
        pixmap.fill_path(
            &blob,
            &solid_paint(red, green, blue, 110),
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

fn paint_noise_curves(pixmap: &mut Pixmap, count: usize, alpha: u8, rng: &mut impl Rng) {
    let width = pixmap.width() as f32;
    let height = pixmap.height() as f32;

    for _ in 0..count {
        let mut builder = PathBuilder::new();
        builder.move_to(rng.random_range(0.0..width * 0.15), rng.random_range(0.0..height));
        builder.cubic_to(
            rng.random_range(width * 0.2..width * 0.5),
            rng.random_range(-0.2 * height..1.2 * height),
            rng.random_range(width * 0.5..width * 0.8),
            rng.random_range(-0.2 * height..1.2 * height),
            rng.random_range(width * 0.85..width),
            rng.random_range(0.0..height),
        );
        let Some(curve) = builder.finish() else { continue };

        let [red, green, blue] = hsl_to_rgb(rng.random_range(0.0..360.0), 0.6, 0.45);
        let stroke = Stroke {
            width: rng.random_range(1.4..3.0),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        pixmap.stroke_path(
            &curve,
            &solid_paint(red, green, blue, alpha),
            &stroke,
            Transform::identity(),
            None,
        );
    }
}

fn paint_speckles(pixmap: &mut Pixmap, rng: &mut impl Rng) {
    let width = pixmap.width() as f32;
    let height = pixmap.height() as f32;
    let speckle_count = pixmap.width() * pixmap.height() / SPECKLE_AREA_PER_DOT;

    for _ in 0..speckle_count {
        let Some(dot) = PathBuilder::from_circle(
            rng.random_range(0.0..width),
            rng.random_range(0.0..height),
            rng.random_range(0.5..1.6),
        ) else {
            continue;
        };
        let [red, green, blue] = hsl_to_rgb(rng.random_range(0.0..360.0), 0.5, 0.4);
        pixmap.fill_path(
            &dot,
            &solid_paint(red, green, blue, 200),
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

fn paint_text(pixmap: &mut Pixmap, text: &str, rng: &mut impl Rng) {
    let font = &*FONT;
    let width = pixmap.width() as f32;
    let height = pixmap.height() as f32;
    let units_per_em = font.units_per_em().unwrap_or(2048.0);
    let base_scale = height * TEXT_HEIGHT_RATIO / units_per_em;

    let glyphs: Vec<(ab_glyph::GlyphId, f32)> = text
        .chars()
        .map(|character| {
            let jitter = rng.random_range(1.0 - GLYPH_SCALE_JITTER..1.0 + GLYPH_SCALE_JITTER);
            (font.glyph_id(character), base_scale * jitter)
        })
        .collect();

    let total_advance: f32 = glyphs
        .iter()
        .map(|(glyph_id, scale)| font.h_advance_unscaled(*glyph_id) * scale)
        .sum();
    let fit = (width * TEXT_MAX_WIDTH_RATIO / total_advance).min(1.0);
    let mut pen_x = (width - total_advance * fit) / 2.0;

    for (glyph_id, scale) in glyphs {
        let scale = scale * fit;
        let advance = font.h_advance_unscaled(glyph_id) * scale;

        if let Some(outline) = font.outline(glyph_id) {
            let center_y = height * (0.5 + rng.random_range(-GLYPH_VERTICAL_JITTER_RATIO..GLYPH_VERTICAL_JITTER_RATIO));
            let placement = GlyphPlacement {
                center_x: pen_x + advance / 2.0,
                center_y,
                scale,
                rotation_degrees: rng.random_range(-GLYPH_ROTATION_DEGREES..GLYPH_ROTATION_DEGREES),
                skew: rng.random_range(-GLYPH_SKEW..GLYPH_SKEW),
            };
            let [red, green, blue] = hsl_to_rgb(rng.random_range(0.0..360.0), 0.75, 0.28);
            paint_glyph(pixmap, &outline, &placement, [red, green, blue]);
        }

        pen_x += advance;
    }
}

struct GlyphPlacement {
    center_x: f32,
    center_y: f32,
    scale: f32,
    rotation_degrees: f32,
    skew: f32,
}

fn paint_glyph(pixmap: &mut Pixmap, outline: &Outline, placement: &GlyphPlacement, color: [u8; 3]) {
    let Some(path) = outline_to_path(outline) else { return };

    let outline_center_x = (outline.bounds.min.x + outline.bounds.max.x) / 2.0;
    let outline_center_y = (outline.bounds.min.y + outline.bounds.max.y) / 2.0;
    let transform = Transform::from_translate(placement.center_x, placement.center_y)
        .pre_rotate(placement.rotation_degrees)
        .pre_concat(Transform::from_skew(placement.skew, 0.0))
        .pre_scale(placement.scale, -placement.scale)
        .pre_translate(-outline_center_x, -outline_center_y);

    let [red, green, blue] = color;
    pixmap.fill_path(
        &path,
        &solid_paint(red / 2, green / 2, blue / 2, 120),
        FillRule::Winding,
        transform.post_translate(GLYPH_SHADOW_OFFSET, GLYPH_SHADOW_OFFSET),
        None,
    );
    pixmap.fill_path(
        &path,
        &solid_paint(red, green, blue, 255),
        FillRule::Winding,
        transform,
        None,
    );
}

fn outline_to_path(outline: &Outline) -> Option<Path> {
    let mut builder = PathBuilder::new();
    let mut previous_end: Option<FontPoint> = None;

    for curve in &outline.curves {
        let (start, end) = curve_endpoints(curve);
        let continues_contour = previous_end.is_some_and(|previous| {
            (previous.x - start.x).abs() <= CONTOUR_JOIN_TOLERANCE
                && (previous.y - start.y).abs() <= CONTOUR_JOIN_TOLERANCE
        });

        if !continues_contour {
            if previous_end.is_some() {
                builder.close();
            }
            builder.move_to(start.x, start.y);
        }

        match curve {
            OutlineCurve::Line(_, to) => builder.line_to(to.x, to.y),
            OutlineCurve::Quad(_, control, to) => builder.quad_to(control.x, control.y, to.x, to.y),
            OutlineCurve::Cubic(_, first, second, to) => {
                builder.cubic_to(first.x, first.y, second.x, second.y, to.x, to.y)
            }
        }
        previous_end = Some(end);
    }

    if previous_end.is_some() {
        builder.close();
    }
    builder.finish()
}

fn curve_endpoints(curve: &OutlineCurve) -> (FontPoint, FontPoint) {
    match *curve {
        OutlineCurve::Line(from, to) => (from, to),
        OutlineCurve::Quad(from, _, to) => (from, to),
        OutlineCurve::Cubic(from, _, _, to) => (from, to),
    }
}

fn wave(source: &Pixmap, rng: &mut impl Rng) -> anyhow::Result<Pixmap> {
    let width = source.width();
    let height = source.height();
    let mut waved = Pixmap::new(width, height)
        .ok_or_else(|| anyhow!("invalid captcha image size {width}x{height}"))?;

    let vertical_amplitude = rng.random_range(1.5..3.2);
    let vertical_period = rng.random_range(40.0..75.0);
    let vertical_phase = rng.random_range(0.0..TAU);
    let horizontal_amplitude = rng.random_range(1.0..2.5);
    let horizontal_period = rng.random_range(30.0..55.0);
    let horizontal_phase = rng.random_range(0.0..TAU);

    let source_pixels = source.data();
    let target_pixels = waved.data_mut();
    for y in 0..height {
        for x in 0..width {
            let source_x = x as f32 + horizontal_amplitude * (y as f32 / horizontal_period * TAU + horizontal_phase).sin();
            let source_y = y as f32 + vertical_amplitude * (x as f32 / vertical_period * TAU + vertical_phase).sin();
            let pixel = sample_bilinear(source_pixels, width, height, source_x, source_y);
            let offset = ((y * width + x) * 4) as usize;
            target_pixels[offset..offset + 4].copy_from_slice(&pixel);
        }
    }

    Ok(waved)
}

fn sample_bilinear(pixels: &[u8], width: u32, height: u32, x: f32, y: f32) -> [u8; 4] {
    let x = x.clamp(0.0, (width - 1) as f32);
    let y = y.clamp(0.0, (height - 1) as f32);
    let left = x.floor() as u32;
    let top = y.floor() as u32;
    let right = (left + 1).min(width - 1);
    let bottom = (top + 1).min(height - 1);
    let x_weight = x - left as f32;
    let y_weight = y - top as f32;

    let pixel_at = |px: u32, py: u32, channel: usize| pixels[((py * width + px) * 4) as usize + channel] as f32;

    let mut result = [0u8; 4];
    for (channel, slot) in result.iter_mut().enumerate() {
        let upper = pixel_at(left, top, channel) * (1.0 - x_weight) + pixel_at(right, top, channel) * x_weight;
        let lower = pixel_at(left, bottom, channel) * (1.0 - x_weight) + pixel_at(right, bottom, channel) * x_weight;
        *slot = (upper * (1.0 - y_weight) + lower * y_weight).round() as u8;
    }
    result
}

fn solid_paint(red: u8, green: u8, blue: u8, alpha: u8) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(red, green, blue, alpha);
    paint.anti_alias = true;
    paint
}

fn hsl_to_rgb(hue_degrees: f32, saturation: f32, lightness: f32) -> [u8; 3] {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let hue_sector = hue_degrees.rem_euclid(360.0) / 60.0;
    let secondary = chroma * (1.0 - (hue_sector % 2.0 - 1.0).abs());
    let (red, green, blue) = match hue_sector as u32 {
        0 => (chroma, secondary, 0.0),
        1 => (secondary, chroma, 0.0),
        2 => (0.0, chroma, secondary),
        3 => (0.0, secondary, chroma),
        4 => (secondary, 0.0, chroma),
        _ => (chroma, 0.0, secondary),
    };
    let lightness_offset = lightness - chroma / 2.0;
    [red, green, blue].map(|channel| ((channel + lightness_offset) * 255.0).round().clamp(0.0, 255.0) as u8)
}

#[cfg(test)]
mod tests {
    use rand::{rngs::StdRng, SeedableRng};

    use super::*;

    const SIZE: ImageSize = ImageSize { width: 360, height: 110 };
    const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

    fn render(text: &str, seed: u64) -> Vec<u8> {
        render_text_png(text, SIZE, &mut StdRng::seed_from_u64(seed)).expect("render succeeds")
    }

    #[test]
    fn output_is_a_png_with_the_requested_dimensions() {
        let png = render("12 + 7 = ?", 1);

        assert_eq!(png[..8], PNG_SIGNATURE);
        let decoded = Pixmap::decode_png(&png).expect("valid png");
        assert_eq!((decoded.width(), decoded.height()), (SIZE.width, SIZE.height));
    }

    #[test]
    fn the_same_seed_gives_the_same_image_and_a_new_seed_a_different_one() {
        assert_eq!(render("42", 7), render("42", 7));
        assert_ne!(render("42", 7), render("42", 8));
    }

    #[test]
    fn different_texts_give_different_images() {
        assert_ne!(render("41", 5), render("42", 5));
    }

    #[test]
    fn the_picture_is_not_blank() {
        let decoded = Pixmap::decode_png(&render("8 × 7 = ?", 3)).expect("valid png");
        let mut distinct_pixels: Vec<[u8; 4]> = decoded
            .data()
            .chunks_exact(4)
            .map(|pixel| [pixel[0], pixel[1], pixel[2], pixel[3]])
            .collect();
        distinct_pixels.sort_unstable();
        distinct_pixels.dedup();

        assert!(distinct_pixels.len() > 50);
    }

    #[test]
    fn every_glyph_used_by_questions_and_options_exists_in_the_font() {
        let font = &*FONT;

        for character in "0123456789+−×= ?".chars() {
            assert_ne!(font.glyph_id(character).0, 0, "missing glyph for {character:?}");
        }
    }

    #[test]
    fn a_zero_sized_image_is_rejected() {
        let result = render_text_png("1", ImageSize { width: 0, height: 10 }, &mut StdRng::seed_from_u64(1));

        assert!(result.is_err());
    }
}
