use artworks::{make_recorder_app, Artwork, BaseModel};
use nannou::{noise::NoiseFn, prelude::*};

fn main() {
    make_recorder_app::<Model>().run();
}

struct Model {
    pub base: BaseModel,
}

const C_DET: u32 = 100;
const N_CIRC: i32 = 80;

impl Artwork for Model {
    fn draw_at_time(&mut self, time: f64) {
        // First, reset the `draw` state.
        let [w, h] = self.base.texture.size();
        let draw = &self.base.draw;
        draw.reset();
        let bg = srgba(0.08627, 0.08627, 0.08627, 1.);
        draw.background().color(bg);
        let _seed = (self.base.seed % 1000) as f64 / 1000.;
        let amp = 5;
        let div = 20.;
        let t_mut = 50;
        let noise = nannou::noise::OpenSimplex::new();
        for k in -amp..=amp {
            for i in -t_mut - 20..N_CIRC + t_mut {
                let arrow = if k % 2 == 0 { 1. } else { -1. };
                let base = 1.3;
                let add = 0.3;

                let rat = (arrow * t_mut as f32 * time as f32 + i as f32) / N_CIRC as f32;
                let nval = noise.get([
                    k as f64,
                    5. * rat as f64,
                    (TAU as f64 * time).cos(),
                    (TAU as f64 * time).sin(),
                ]);

                let x = k as f32 * (2. * base) * w as f32 / div;
                let y = (-0.5 + rat) * h as f32;
                let sfreq = 5.;
                let nint = noise.get([
                    sfreq * x as f64 / w as f64
                        - 9. * ((TAU as f64 * time).cos() + (TAU as f64 * time).sin()),
                    sfreq * y as f64 / w as f64,
                    (TAU as f64 * time).cos(),
                    (TAU as f64 * time).sin(),
                ]);

                let v = 6. * TAU * rat;

                let var = if k % 2 == 0 { v.sin() } else { (v - PI).sin() };
                let r = (base + add * var) * (w as f32 / div);
                let grey = map_range(1., 0., 1., 0.1, 1.);
                // draw.path().fill().color(bg)
                // .points(points);

                let points = (0..=C_DET).map(|d| {
                    let msk = 100.;
                    let dis = 0.5 / N_CIRC as f32;
                    let displacement =
                        map_range(d as f32, 0., C_DET as f32, - (h as f32) * arrow * dis,  h as f32 * arrow * dis);
                    let theta = map_range(d as f32, 0., C_DET as f32, PI / msk, PI - PI / msk);
                    let alpha = map_range(
                        (d as f32 - C_DET as f32 / 2.).abs(),
                        0.,
                        C_DET as f32 / 2.,
                        1.,
                        0.,
                    );
                    (
                        Point2::new(x + r * (theta.cos()), y + displacement + r * theta.sin()),
                        srgba(grey, grey, grey, alpha),
                    )
                });
                draw.path().stroke().weight(6.0).points_colored(points);
            }
        }
    }

    fn get_model(&self) -> &BaseModel {
        &self.base
    }
    fn get_mut_model(&mut self) -> &mut BaseModel {
        &mut self.base
    }
    fn new(base: BaseModel) -> Model {
        Model { base }
    }
}
