use artworks::{make_recorder_app, Artwork, BaseModel, Options};
use nannou::noise::{NoiseFn, OpenSimplex};
use nannou::{draw::mesh::vertex::Point, prelude::*};

fn main() {
    make_recorder_app::<Model>().run();
}

struct Model {
    pub base: BaseModel,
    noise: OpenSimplex,
}

const OFFSET: f32 = PI / 15.;
const SEP: u32 = 80;
const STEP: f32 = PI / SEP as f32;
const K: f32 = 120.;
const FAC: f32 = 60.;
const SEG: isize = 7;
const SPC: f32 = 1. / 3.;
const N_RINGS: usize = 6;

fn get_points(
    radial_number: f32,
    ring_number: isize,
    time: f32,
    noise: &OpenSimplex,
    seed: f64,
) -> (Point2, Point2) {
    if ring_number == -1 {
        return (Point2::new(0., 0.), Point2::new(0., 0.));
    }
    let nc = radial_number;
    let inc =
        nc + (if (ring_number) % 2 == 0 { -1. } else { 1. }) * OFFSET / (ring_number as f32 + 1.);

    let noisy_time_angle_offset = TAU
        * noise
            .get([
                1. * ring_number as f64 / N_RINGS as f64,
                2. * (time.cos() as f64 + time.sin() as f64),
                seed / 1.,
                radial_number as f64 / SEP as f64,
            ])
            .sin() as f32;
    let s = 1.
        + (if (ring_number) % 2 == 0 { -1. } else { 1. }) * (time + noisy_time_angle_offset).sin()
            / 2.;
    let noisy_radius = 0.4
        * noise.get([
            1. * ring_number as f64 / N_RINGS as f64,
            1. * (time.cos() as f64 + time.sin() as f64) / 10.,
            seed + 4.,
            // radial_number as f64 / SEP as f64,
        ]) as f32;
    let rd = ((ring_number + 1) as f32 + noisy_radius) * K
        + FAC * (nc + (ring_number + 1) as f32 * PI / 2. + time).cos();
    let theta_left = inc + (if (ring_number) % 2 == 0 { -1. } else { 1. }) * 0.5 * STEP * s;
    let theta_right = inc + (if (ring_number) % 2 == 1 { -1. } else { 1. }) * 0.5 * STEP * s;
    let p1 = Point2::new(rd * theta_left.sin(), rd * theta_left.cos());
    let p2 = Point2::new(rd * theta_right.sin(), rd * theta_right.cos());
    (p1, p2)
}

fn draw_pieces(draw: &Draw, p0: Point2, p1: Point2, p2: Point2, p3: Point2, t: f32) {
    (0..SEG).for_each(|sg| {
        let sep = sg as f32 / (SEG as f32 + 1.);
        let v = sep + t;
        let step = (1. - SPC) / (SEG as f32 + 1.) * v.pow(1. / 3.) * (1. - v).pow(1. / 3.);
        draw.quad().color(rgba(1.0, 1.0, 1.0, 0.8)).points(
            p0.lerp(p3, v.max(0.)),
            p1.lerp(p2, v.max(0.)),
            p1.lerp(p2, (v + step).min(1. - 1. / (SEG as f32 + 1.))),
            p0.lerp(p3, (v + step).min(1. - 1. / (SEG as f32 + 1.))),
        );
    })
}

impl Artwork for Model {
    fn draw_at_time(&mut self, time: f64) {
        // First, reset the `draw` state.
        let draw = &self.base.draw;
        draw.reset();

        // Get the width and height of the animation.
        let [_w, _h] = self.base.texture.size();
        // Set the seed
        let seed = (self.base.seed % 1000) as f64 / 1000.;

        draw.background()
            .color(srgba(0.08627, 0.08627, 0.08627, 1.));

        let btime = time;
        let time = time as f32 * TAU;
        (0..SEP).for_each(|val| {
            let nc = TAU * (val as f32 / SEP as f32) + time / 2.;

            (0..N_RINGS).for_each(|rg| {
                let (p0, p1) = get_points(nc, rg as isize - 1, time, &self.noise, seed);
                let (p2, p3) = get_points(nc, rg as isize, time, &self.noise, seed);
                draw_pieces(
                    draw,
                    p0,
                    p1,
                    p2,
                    p3,
                    btime as f32 * (1. / (SEG as f32 + 1.)),
                );
            });
        });
    }

    fn get_model(&self) -> &BaseModel {
        &self.base
    }
    fn get_mut_model(&mut self) -> &mut BaseModel {
        &mut self.base
    }
    fn new(base: BaseModel) -> Model {
        let noise = OpenSimplex::new();

        Model { base, noise }
    }

    fn n_sec(&self) -> Option<u32> {
        Some(7)
    }

    fn get_options() -> Option<Options> {
        Some(Options {
            chroma: 0.2,
            sample_per_frame: 10,
            shutter_angle: 0.5,
            extra_tex: None,
            noise_amount: 0.1,
        })
    }
}
