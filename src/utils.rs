pub mod colors {
    use nannou::{
        color::{hsva, srgb, Hsva, IntoLinSrgba, LinSrgba, Rgb, Srgba},
        prelude::TAU,
        rand::random_f32,
    };

    const GOLDEN_RATIO: f32 = 0.618033988749895;

    pub fn get_random_gr_palette(length: usize, saturation: f32, value: f32) -> Vec<Hsva> {
        let mut palette = vec![];
        let mut hue = random_f32();
        for _ in 0..length {
            hue += GOLDEN_RATIO;
            hue = hue.fract();
            palette.push(hsva(hue, saturation, value, 1.0))
        }
        palette
    }

    // Adapted from https://iquilezles.org/articles/palettes/
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
        // let col = (c.into_lin_srgba() * t + d.into_lin_srgba()) * TAU;
        // a.into_lin_srgba()
        //     + b.into_lin_srgba()
        //         * Srgba::from_components((
        //             col.red.cos(),
        //             col.green.cos(),
        //             col.blue.cos(),
        //             col.alpha.cos(),
        //         ))
        //         .into_lin_srgba()
        // a + b * (TAU * (c * t + d)).cos()
    }
}
