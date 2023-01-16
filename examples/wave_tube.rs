use artworks::{make_recorder_app, Artwork, BaseModel, Options};
use nannou::{noise::NoiseFn, prelude::*};

fn main() {
    make_recorder_app::<Model>().run();
}

struct Model {
    pub base: BaseModel,
    // pt_offsets: Vec<u32>,
}

const C_DET: u32 = 30;
const N_CIRC: i32 = 50;

impl Artwork for Model {
    fn draw_at_time(&mut self, time: f64) {
        // First, reset the `draw` state.
        let [w, h] = self.base.texture.size();
        let draw = &mut self.base.draw;
        draw.reset();
        let bg = srgba(0.08627, 0.08627, 0.08627, 1.);
        draw.background().color(bg);
        let draw = draw.x_degrees(30.);
        let seed = (self.base.seed % 1000) as f64 / 1000.;
        let amp = 5;
        let div = 20.;
        let t_mut = 30;
        let noise = nannou::noise::OpenSimplex::new();
        for k in -amp - 1..=amp + 1 {
            let ratk = (-3. * time as f32 + k as f32) / amp as f32;
            for i in -t_mut - 50..N_CIRC + t_mut + 50 {
                let arrow = if k % 2 == 0 { 1. } else { -1. };
                let base = 1.3;
                let rat = (arrow * t_mut as f32 * time as f32 + i as f32) / N_CIRC as f32;
                let rat_faster = (arrow * t_mut as f32 * time as f32 + i as f32) / N_CIRC as f32;
                let add = 0.01
                    + map_range(
                        noise.get([
                            rat_faster as f64,
                            ratk as f64 / 2.,
                            0.4 * (TAU as f64 * time).cos() + seed / 2.3,
                            0.4 * (TAU as f64 * time).sin() + seed,
                        ]) as f32,
                        -1.,
                        1.,
                        0.,
                        1.6,
                    );

                let x = (-2. * time as f32 + k as f32) * (2. * base) * w as f32 / div;
                let y = (-0.5 + rat) * h as f32;

                let v = 6. * TAU * rat;

                let var = if k % 2 == 0 { v.sin() } else { (v - PI).sin() };
                let r = (base + add * var) * (w as f32 / div);
                let grey = map_range(1., 0., 1., 0.1, 1.);

                let points = (0..=C_DET).map(|d| {
                    let msk = 200.;
                    let mult = 8.
                        * map_range(
                            noise.get([
                                rat_faster as f64,
                                ratk as f64,
                                0.4 * (TAU as f64 * time).cos() + seed / 3.3,
                                0.4 * (TAU as f64 * time).sin() + 2. * seed,
                            ]) as f32,
                            -1.,
                            1.,
                            0.2,
                            1.,
                        );
                    let dis = 0.5 / N_CIRC as f32;
                    let index = fmod(
                        d as f32 + mult * arrow * C_DET as f32 * time as f32,
                        C_DET as f32,
                    );
                    let theta = map_range(index, 0., C_DET as f32, PI / msk, PI - PI / msk);
                    let displacement = map_range(
                        theta,
                        0.,
                        PI,
                        -(h as f32) * arrow * dis,
                        h as f32 * arrow * dis,
                    );
                    let alpha = map_range((theta - PI / 2.).abs(), 0., PI / 2., 1., 0.);
                    (
                        Point3::new(x + r * (theta.cos()), y + displacement, -r * theta.sin()),
                        srgba(grey, grey, grey, alpha),
                    )
                });
                for (p, c) in points {
                    draw.ellipse().color(c).radius(5.).xyz(p);
                }
            }
        }
    }

    fn get_model(&self) -> &BaseModel {
        &self.base
    }
    fn get_mut_model(&mut self) -> &mut BaseModel {
        &mut self.base
    }

    fn get_options() -> Option<Options> {
        Some(Options {
            chroma: 0.4,
            sample_per_frame: 5,
            shutter_angle: 0.8,
            extra_tex: None,
            noise_amount: 0.0,
        })
    }
    fn new(base: BaseModel) -> Model {
        // let pt_offsets = (0..=C_DET).map(|_| random_range::<u32>(2, 8)).collect();
        Model {
            base,
            // pt_offsets
        }
    }

    fn n_sec(&self) -> Option<u32> {
        Some(5)
    }
}
