use nannou::{
    color::{hsva, Hsva, IntoLinSrgba, Srgba},
    prelude::TAU,
    rand::random_f32,
};
pub mod colorbrewer;

const GOLDEN_RATIO: f64 = 0.618_033_988_749_895;

/// This function generates a random evenly distributed color palette using a
/// golden ratio spacing of hue values.
///
/// Adapted from https://martin.ankerl.com/2009/12/09/how-to-create-random-colors-programmatically/
pub fn get_random_gr_palette(length: usize, saturation: f32, value: f32) -> Vec<Hsva> {
    let mut palette = vec![];
    let mut hue = random_f32();
    for _ in 0..length {
        hue += GOLDEN_RATIO as f32;
        hue = hue.fract();
        palette.push(hsva(hue, saturation, value, 1.0))
    }
    palette
}

/// This function generates a continuous interpolation of colors.
///
/// Adapted from https://iquilezles.org/articles/palettes/
pub fn interpolate<C>(t: f32, a: C, b: C, c: C, d: C) -> Srgba
where
    C: IntoLinSrgba<f32>,
{
    let (a, b, c, d) = (
        a.into_lin_srgba(),
        b.into_lin_srgba(),
        c.into_lin_srgba(),
        d.into_lin_srgba(),
    );
    Srgba::from_components((
        a.red + b.red * (TAU * (c.red * t + d.red)).cos(),
        a.green + b.green * (TAU * (c.green * t + d.green)).cos(),
        a.blue + b.blue * (TAU * (c.blue * t + d.blue)).cos(),
        1.0,
    ))
}
