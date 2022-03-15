use artworks::{make_recorder_app, Artwork, BaseModel, Options};
use nannou::{noise::NoiseFn, prelude::*};

fn main() {
    make_recorder_app::<Tube>().run();
}

struct Tube {
    pub base: BaseModel,
}

impl Artwork for Tube {
    fn draw_at_time(&mut self, time: f64) {
        // First, reset the `draw` state.
        let draw = &self.base.draw;
        draw.reset();

        let [w, _h] = self.base.texture.size();

        let tau = TAU * time as f32;
        let seed = (self.base.seed % 1000) as f64 / 1000.;

        // Draw like we normally would in the `view`.
        draw.background()
            .color(srgba(0.08627, 0.08627, 0.08627, 1.));
        let n_points = 30;
        let n_circles = 40;
        let queue = 20;
        let spacen = 2.;
        let amort = 1.;

        for j in -queue..=n_circles + queue {
            for i in 0..n_points {
                let rat = (-queue as f32 * time as f32 + j as f32) / n_circles as f32;
                let ns = nannou::noise::SuperSimplex::new();
                let nsa = 1.
                    + ns.get([
                        (2. * (spacen * rat + tau)).sin() as f64 / amort,
                        (spacen * rat + tau).cos() as f64 / amort,
                        seed,
                    ]);
                let ws = 5. * nsa as f32 * w as f32 / n_points as f32;
                let rato = i as f32 / n_points as f32;
                let nsc = 1.
                    + ns.get([
                        (2. * (spacen * rato + tau)).sin() as f64 / amort,
                        (spacen * rato + tau).cos() as f64 / amort,
                        2. * seed,
                    ]);
                let xpos = nsc as f32 * w as f32 / 20. + rat * w as f32 - w as f32 / 2.;
                let nsb = 1.
                    + ns.get([
                        (1. * (spacen * rat + tau)).sin() as f64 / amort,
                        (spacen * rat + tau).cos() as f64 / amort,
                        ((1. * (spacen * rato + tau)).sin() as f64
                            + seed
                            + (spacen * rato + tau).cos() as f64)
                            / amort,
                    ]);
                let theta = tau + TAU * rato + 0.3 * TAU * (rat + nsb as f32 + 1.);
                let ypos = (theta).sin();
                let zpos = (theta).cos();
                let alpha = map_range(zpos, -1., 1., 0.2, 0.6);
                // let weight = map_range(nsb, 0., 1., 1.5, 3.);
                draw.ellipse()
                    .color(srgba(1., 1., 1., alpha))
                    .radius(5. + 10. * nsb as f32)
                    .x_y_z(xpos, ws * ypos, ws * zpos);
            }
        }
    }

    fn get_model(&self) -> &BaseModel {
        &self.base
    }
    fn get_mut_model(&mut self) -> &mut BaseModel {
        &mut self.base
    }
    fn new(base: BaseModel) -> Tube {
        Tube { base }
    }

    fn get_options() -> Option<Options> {
        Some(Options {
            chroma: 0.3,
            sample_per_frame: 5,
            shutter_angle: 0.3,
        })
    }
}
