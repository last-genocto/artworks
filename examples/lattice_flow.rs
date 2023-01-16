use artworks::{make_recorder_app, Artwork, BaseModel, Options};
use nannou::color::RgbHue;
use nannou::ease::cubic::ease_out;
use nannou::noise::{NoiseFn, OpenSimplex};
use nannou::{noise, prelude::*};

fn main() {
    make_recorder_app::<Model>().run();
}

struct Item {
    x: f32,
    y: f32,
    r: f32,
    sd: u32,
    speed: f32,
}

struct Model {
    pub base: BaseModel,
    grid: Vec<Item>,
    noise: OpenSimplex,
}

impl Artwork for Model {
    fn draw_at_time(&mut self, time: f64) {
        // First, reset the `draw` state.
        let draw = &self.base.draw;
        draw.reset();
        // Get the width and height of the animation.
        let [w, _h] = self.base.texture.size();
        let draw = draw.scale(0.85);
        // Set the seed

        let seed = (self.base.seed % 1000) as f64 / 1000.;
        draw.background()
            .color(srgba(0.08627, 0.08627, 0.08627, 1.));
        for item in &self.grid {
            let x = item.x * (2. * item.speed as f64 * time).exp() as f32;
            let y = item.y * (-2. * item.speed as f64 * time).exp() as f32;
            let r_noise = self.noise.get([
                (TAU_F64 * time).sin(),
                (TAU_F64 * time).cos(),
                seed,
                item.sd as f64,
            ]);
            let x_noise = self.noise.get([
                (TAU_F64 * time).sin() / 5.,
                (TAU_F64 * time).cos() / 5.,
                12. * seed,
                item.sd as f64,
            ]);
            let y_noise = self.noise.get([
                (TAU_F64 * time).sin() / 5.,
                (TAU_F64 * time).cos() / 5.,
                7. * seed,
                item.sd as f64,
            ]);
            let rad = item.r * (1. + (r_noise as f32));
            let x = x + (w as f32 / 20.) * x_noise as f32;
            let y = y + (w as f32 / 20.) * y_noise as f32;
            let c = clamp(
                ease_out(
                map_range(Vec2::new(x, y).length(), 0., w as f32 / 2., 1., 0.),
                0.,
                1.,
                1.,
            ), 0.08627, 1.);

            let color = srgba(c, c, c, 1.);
            if Vec2::new(x, y).length() < w as f32 / 1.3 {
                draw.ellipse()
                    .radius(1.5 * rad)
                    .x_y(x, y)
                    .color(srgba(c, c, c, 0.05));
                draw.ellipse()
                    .radius(1.4 * rad)
                    .x_y(x, y)
                    .color(srgba(c, c, c, 0.05));
                draw.ellipse()
                    .radius(1.3 * rad)
                    .x_y(x, y)
                    .color(srgba(c, c, c, 0.05));
                draw.ellipse().radius(rad).x_y(x, y).color(color);
            }
        }
        // draw.blend(BLEND_ADD)
        //     .ellipse()
        //     .radius(w as f32 / 3.)
        //     .color(BLACK);
    }

    // fn get_options() -> Option<Options> {
    //     Some(Options {
    //         chroma: 0.3,
    //         sample_per_frame: 5,
    //         shutter_angle: 1.,
    //         extra_tex: None,
    //         noise_amount: 0.0,
    //     })
    // }

    fn get_model(&self) -> &BaseModel {
        &self.base
    }
    fn get_mut_model(&mut self) -> &mut BaseModel {
        &mut self.base
    }
    fn n_sec(&self) -> Option<u32> {
        Some(5)
    }
    fn new(base: BaseModel) -> Model {
        let [w, _h] = base.texture.size();
        let mut grid_vec = vec![];
        let grid = 20;
        for i in -grid..2 * grid {
            for j in -grid..2 * grid {
                let item = Item {
                    x: (2. * random_f32() - 1.) * w as f32,
                    y: (2. * random_f32() - 1.) * w as f32,
                    r: random_f32() * 6. + 2.,
                    sd: random(),
                    speed: 0.8 + 4.5 * random_f32(),
                };
                for i in -15..=15 {
                    grid_vec.push(Item {
                        x: item.x * (-(i as f32) * item.speed as f32).exp(),
                        y: item.y * (i as f32 * item.speed as f32).exp(),
                        r: item.r,
                        sd: item.sd,
                        speed: item.speed,
                    });
                }
                grid_vec.push(item);
            }
        }
        let noise = OpenSimplex::new();
        Model {
            base,
            grid: grid_vec,
            noise,
        }
    }
}
