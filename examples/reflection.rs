use artworks::{make_recorder_app, Artwork, BaseModel, Options};

use nannou::{
    ease::cubic::{ease_in, ease_out},
    noise::{NoiseFn, OpenSimplex},
    prelude::*,
};

fn main() {
    make_recorder_app::<Model>().run();
}

struct Model {
    pub base: BaseModel,
    balls: Vec<Ball>,
}

struct Ball {
    size: f32,
    offset: f64,
    offset_ax: f32,
}

fn draw_fuzzy_line(draw: &Draw, p1: Vec2, p2: Vec2) {
    for c in 0..20 {
        let wg = map_range(c, 0, 20, 70., 5.);
        let col = ease_in(map_range(c, 0, 20, 0., 1.), 0.01, 0.3, 1.);
        draw.line()
            .points(p1, p2)
            .stroke_weight(wg)
            .color(srgba(1., 1., 1., col))
            .caps_round();
    }
}

fn draw_lines(draw: &Draw, total: f64, w: f32, fact: f32, time: f64) {
    let line_offset = map_range(total, 1., 1.2, 0., w / 30.);
    let t = ease_out(
        map_range(((TAU as f64 * time).cos()) as f32, -1., 1., 0., 1.),
        0.,
        1.,
        1.,
    );
    let line_size = t * w / 10. + w / 6.;
    let d_to_center = fact * w / 2. + 15.;
    draw_fuzzy_line(
        &draw,
        Vec2::new(-d_to_center, -d_to_center) + Vec2::new(line_size, -line_size),
        Vec2::new(-d_to_center, -d_to_center) + Vec2::new(-line_size, line_size),
    );
    draw_fuzzy_line(
        &draw,
        Vec2::new(d_to_center, -d_to_center) + Vec2::new(line_size, line_size),
        Vec2::new(d_to_center, -d_to_center) + Vec2::new(-line_size, -line_size),
    );
    draw_fuzzy_line(
        &draw,
        Vec2::new(-d_to_center, d_to_center) + Vec2::new(line_size, line_size),
        Vec2::new(-d_to_center, d_to_center) + Vec2::new(-line_size, -line_size),
    );
    draw_fuzzy_line(
        &draw,
        Vec2::new(d_to_center, d_to_center) + Vec2::new(line_size, -line_size),
        Vec2::new(d_to_center, d_to_center) + Vec2::new(-line_size, line_size),
    );
}

impl Artwork for Model {
    fn draw_at_time(&mut self, time: f64) {
        // First, reset the `draw` state.
        let draw = &self.base.draw;
        draw.reset();

        // Get the width and height of the animation.
        let [w, _h] = self.base.texture.size();
        // Set the seed
        let seed = (self.base.seed % 1000) as f64 / 1000.;
        draw.background()
            .color(srgba(0.08627, 0.08627, 0.08627, 1.));
        let draw = draw.scale(0.6);
        let ns = OpenSimplex::new();
        let total = 1. + 0.2 * (TAU as f64 * time).sin();
        let fact = (0.7 + 0.3 * (TAU as f64 * time).cos()) as f32;
        for ball in self.balls.iter() {
            let ltime = (2. * (time - ball.offset + 1.)).fract();
            let rs = ns.get([
                seed,
                ball.offset_ax as f64,
                (TAU as f64 * ltime).sin(),
                (TAU as f64 * ltime).cos(),
            ]);
            let step_low = map_range(fact, 0.4, 1., 0.13, 0.18);
            let step_high = map_range(fact, 0.4, 1., 0.37, 0.32);
            let steps = map_range(rs, -1., 1., step_low, step_high);

            let dist_x = map_range(steps, 0., total / 4., 0., fact * (w as f32) / 2.);
            let dist_y = map_range(
                total / 2. - steps,
                0.,
                total / 4.,
                0.,
                fact * (w as f32) / 2.,
            );

            let z = 10.
                * ns.get([
                    seed - 100.,
                    100. * ball.offset_ax as f64,
                    (TAU as f64 * ltime).sin(),
                    (TAU as f64 * ltime).cos(),
                ]) as f32;
            let col_val = map_range(z, -10., 7., 0.2, 1.);
            let col = srgba(col_val, col_val, col_val, 0.9);

            let (x, y) = if ltime < 0.25 {
                (map_range(ltime, 0., 0.25, -dist_x, dist_x), -dist_y)
            } else if ltime < 0.5 {
                (dist_x, map_range(ltime, 0.25, 0.5, -dist_y, dist_y))
            } else if ltime < 0.75 {
                (map_range(ltime, 0.5, 0.75, dist_x, -dist_x), dist_y)
            } else {
                (-dist_x, map_range(ltime, 0.75, 1., dist_y, -dist_y))
            };
            draw.ellipse().x_y_z(x, y, z).radius(ball.size).color(col);
        }

        draw_lines(&draw, total, w as f32, fact, time);
    }

    fn get_options() -> Option<Options> {
        Some(Options {
            chroma: 0.4,
            sample_per_frame: 5,
            shutter_angle: 0.5,
            extra_tex: None,
            noise_amount: 0.0,
        })
    }
    fn n_sec(&self) -> Option<u32> {
        Some(15)
    }

    fn get_model(&self) -> &BaseModel {
        &self.base
    }
    fn get_mut_model(&mut self) -> &mut BaseModel {
        &mut self.base
    }
    fn new(base: BaseModel) -> Model {
        let [w, _h] = base.texture.size();
        Model {
            base,
            balls: (0..400)
                .map(|_| Ball {
                    size: 5. + 10. * random_f32(),
                    offset: random_f64(),
                    offset_ax: (w as f32 / 50.) * (2. * random_f32() - 1.),
                })
                .collect(),
        }
    }
}
