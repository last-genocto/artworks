use nannou::{
    color::{hsla, Hsla},
    prelude::{Vec3, PI},
    rand::random_f32,
};

fn point_to_hsl(point: Vec3) -> Vec3 {
    // cx and cy are the center (x and y) values
    let cx = 0.5;
    let cy = 0.5;

    // Calculate the angle in radians
    let rad = (point.y - cy).atan2(point.x - cx);
    let deg = rad.to_degrees();

    // Saturation
    let s = point.z;

    // Lightness
    let l = ((point.y - cy).powi(2) + (point.x - cx).powi(2)).sqrt() / cx;

    Vec3::new(deg, s, l)
}

fn hsl_to_point(hsl: Vec3) -> Vec3 {
    // cx and cy are the center (x and y) values
    let cx = 0.5;
    let cy = 0.5;

    // Calculate the angle in radians based on the hue value
    let radians = hsl.x.to_radians();

    // Calculate the distance from the center based on the lightness value
    let dist = hsl.z * cx;

    // Calculate the x and y coordinates based on the distance and angle
    let x = cx + dist * radians.cos();
    let y = cy + dist * radians.sin();

    // The z coordinate is equal to the saturation value
    let z = hsl.y;

    // Return the (x, y, z) coordinate as an array [x, y, z]
    Vec3::new(x, y, z)
}

fn random_hsl_pair(
    start_hue: f32,
    saturations: (f32, f32),
    lightnesses: (f32, f32),
) -> (Vec3, Vec3) {
    (
        Vec3::new(start_hue, saturations.0, lightnesses.0),
        Vec3::new(
            (start_hue + 60. + random_f32() * 180.) % 360.,
            saturations.1,
            lightnesses.1,
        ),
    )
}

fn random_hsl_triple(
    start_hue: f32,
    saturations: (f32, f32, f32),
    lightnesses: (f32, f32, f32),
) -> (Vec3, Vec3, Vec3) {
    (
        Vec3::new(start_hue, saturations.0, lightnesses.0),
        Vec3::new(
            (start_hue + 60. + random_f32() * 180.) % 360.,
            saturations.1,
            lightnesses.1,
        ),
        Vec3::new(
            (start_hue + 60. + random_f32() * 180.) % 360.,
            saturations.2,
            lightnesses.2,
        ),
    )
}

type PositionFunction = dyn Fn(f32, bool) -> f32;

fn linear_position(t: f32, _: bool) -> f32 {
    t
}

fn exponential_position(t: f32, reverse: bool) -> f32 {
    if reverse {
        1. - (1. - t).powi(2)
    } else {
        t.powi(2)
    }
}

fn quadratic_position(t: f32, reverse: bool) -> f32 {
    if reverse {
        1. - (1. - t).powi(3)
    } else {
        t.powi(3)
    }
}

fn cubic_position(t: f32, reverse: bool) -> f32 {
    if reverse {
        1. - (1. - t).powi(4)
    } else {
        t.powi(4)
    }
}

fn quartic_position(t: f32, reverse: bool) -> f32 {
    if reverse {
        1. - (1. - t).powi(5)
    } else {
        t.powi(5)
    }
}

fn sinusoidal_position(t: f32, reverse: bool) -> f32 {
    if reverse {
        1. - (((1. - t) * PI) / 2.).sin()
    } else {
        ((t * PI) / 2.).sin()
    }
}

fn asinusoidal_position(t: f32, reverse: bool) -> f32 {
    if reverse {
        return 1. - (1. - t).asin() / (PI / 2.);
    } else {
        t.asin() / (PI / 2.)
    }
}

fn arc_position(t: f32, reverse: bool) -> f32 {
    if reverse {
        (1. - (1. - t).powi(2)).sqrt()
    } else {
        1. - (1. - t).sqrt()
    }
}

fn smooth_step_position(t: f32, _: bool) -> f32 {
    return t.powi(2) * (3. - 2. * t);
}

pub enum PosFunctions<'a> {
    LinearPosition,
    ExponentialPosition,
    QuadraticPosition,
    CubicPosition,
    QuarticPosition,
    SinusoidalPosition,
    AsinusoidalPosition,
    ArcPosition,
    SmoothStepPosition,
    Custom {
        fx: &'a PositionFunction,
        fy: &'a PositionFunction,
        fz: &'a PositionFunction,
    },
}

fn vectors_on_line(
    p1: Vec3,
    p2: Vec3,
    num_points: u32,
    invert: bool,
    fx: &PositionFunction,
    fy: &PositionFunction,
    fz: &PositionFunction,
) -> Vec<Vec3> {
    let mut points = vec![];

    for i in 0..num_points {
        let t = i as f32 / (num_points - 1) as f32;
        let t_mod_x = fx(t, invert);
        let t_mod_y = fy(t, invert);
        let t_mod_z = fz(t, invert);
        let x = (1. - t_mod_x) * p1.x + t_mod_x * p2.x;
        let y = (1. - t_mod_y) * p1.y + t_mod_y * p2.y;
        let z = (1. - t_mod_z) * p1.z + t_mod_z * p2.z;

        points.push(Vec3::new(x, y, z));
    }

    points
}

pub fn get_random_color_palette(length: usize, pos_functions: PosFunctions) -> Vec<Hsla> {
    let (c1, c2) = random_hsl_pair(
        360. * random_f32(),
        (random_f32(), random_f32()),
        (0.75 + random_f32() * 0.2, 0.3 + random_f32() * 0.2),
    );
    let (fx, fy, fz): (&PositionFunction, &PositionFunction, &PositionFunction) =
        match pos_functions {
            PosFunctions::LinearPosition => (&linear_position, &linear_position, &linear_position),
            PosFunctions::ExponentialPosition => (
                &exponential_position,
                &exponential_position,
                &exponential_position,
            ),
            PosFunctions::Custom { fx, fy, fz } => (fx, fy, fz),
            PosFunctions::QuadraticPosition => (
                &quadratic_position,
                &quadratic_position,
                &quadratic_position,
            ),
            PosFunctions::CubicPosition => (&cubic_position, &cubic_position, &cubic_position),
            PosFunctions::QuarticPosition => {
                (&quartic_position, &quartic_position, &quartic_position)
            }
            PosFunctions::SinusoidalPosition => (
                &sinusoidal_position,
                &sinusoidal_position,
                &sinusoidal_position,
            ),
            PosFunctions::AsinusoidalPosition => (
                &asinusoidal_position,
                &asinusoidal_position,
                &asinusoidal_position,
            ),
            PosFunctions::ArcPosition => (&arc_position, &arc_position, &arc_position),
            PosFunctions::SmoothStepPosition => (
                &smooth_step_position,
                &smooth_step_position,
                &smooth_step_position,
            ),
        };
    vectors_on_line(c1, c2, length as u32, false, fx, fy, fz)
        .into_iter()
        .map(point_to_hsl)
        .map(|hsl| hsla(hsl.x / 360., hsl.y, hsl.z, 1.0))
        .collect()
}

mod tests {}
