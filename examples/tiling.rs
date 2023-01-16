use artworks::{make_recorder_app, Artwork, BaseModel, Options};
use nannou::{
    ease::{elastic::ease_out, map_clamp},
    prelude::*,
};

fn main() {
    make_recorder_app::<Model>().run();
}

struct Model {
    pub base: BaseModel,
    random_ts: Vec<Vec<Option<f64>>>,
}

impl Artwork for Model {
    fn draw_at_time(&mut self, time: f64) {
        // First, reset the `draw` state.
        let draw = &self.base.draw;
        draw.reset();

        // Get the width and height of the animation.
        let [w, _h] = self.base.texture.size();
        // Set the seed
        let _seed = (self.base.seed % 1000) as f64 / 1000.;
        draw.background()
            .color(srgba(0.08627, 0.08627, 0.08627, 1.));
        let size = w as f32 / 20.;
        let side = (3.).sqrt() * size;
        let height = (3. / 2.) * size;
        let f = time as f32 * side;

        for t in -5..5 {
            for u in -5..5 {
                let palette = if (t + u).abs() % 2 == 0 {
                    (
                        srgba(46. / 255., 139. / 255., 192. / 255., 1.),
                        srgba(177. / 255., 212. / 255., 224. / 255., 1.),
                    )
                } else {
                    (
                        srgba(46. / 255., 139. / 255., 192. / 255., 1.),
                        srgba(177. / 255., 212. / 255., 224. / 255., 1.),
                    )
                };

                let rot = 0.;

                let x = t as f32 * side * (3. / 2.);
                let y = u as f32 * side * (3.).sqrt() + height * if t % 2 == 0 { 1. } else { 0. };
                draw_block(
                    &draw.translate(Vec3::new(x, y, 0.)).rotate(rot as f32),
                    size,
                    0.,
                    palette,
                );
            }
        }

        for t in -5..5 {
            for u in -5..5 {
                let palette = (
                    srgba(20. / 255., 93. / 255., 160. / 255., 1.),
                    srgba(12. / 255., 45. / 255., 72. / 255., 1.),
                );

                let rot = match self.random_ts[(t + 5) as usize][(u + 5) as usize] {
                    Some(t) => {
                        let loc_time = clamp(map_range(time, t, t + 0.3, 0., 1.), 0., 1.);
                        map_range(ease_out(loc_time, 0., 1., 1.), 0., 1., 0., 4. * PI / 3.)
                    }
                    None => 0.,
                };

                let x = t as f32 * side * (3. / 2.);
                let y = u as f32 * side * (3.).sqrt() + height * if t % 2 == 0 { 1. } else { 0. };
                draw_block(
                    &draw.translate(Vec3::new(x, y, 0.)).rotate(rot as f32),
                    size,
                    f,
                    palette,
                );
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
        Model {
            base,
            random_ts: (-5..5)
                .map(|_| {
                    (-5..5)
                        .map(|_| {
                            let t = random_f64();
                            if t < 0.7 {
                                Some(t)
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .collect(),
        }
    }

    fn get_options() -> Option<Options> {
        Some(Options {
            chroma: 0.3,
            sample_per_frame: 5,
            shutter_angle: 1.,
            extra_tex: None,
            noise_amount: 0.0,
        })
    }
}

fn draw_pent(draw: &Draw, size: f32, f: f32, color: Srgba) {
    let p0 = Vec2::new(0., size);
    let thet = PI / 2. + TAU / 3.;
    let p1 = Vec2::new(size * thet.cos(), size * thet.sin());
    let p1a = p1 + f * Vec2::new((PI / 3.).cos(), (PI / 3.).sin());
    let p1b = p1 + Vec2::new(f, 0.);
    let thet = PI / 2. + 2. * TAU / 3.;
    let p2 = Vec2::new(size * thet.cos(), size * thet.sin());
    let p2a = p2 - Vec2::new(f, 0.);
    let p2b = p2 + f * Vec2::new((2. * PI / 3.).cos(), (2. * PI / 3.).sin());

    draw.polygon()
        .stroke_color(srgba(0.08627, 0.08627, 0.08627, 1.))
        .stroke_weight(6.)
        .join_round()
        .color(color)
        .points([p0, p1a, p1b, p2a, p2b]);
}

fn draw_block(draw: &Draw, size: f32, f: f32, palette: (Srgba, Srgba)) {
    draw_pent(
        &draw.translate(Vec3::new(0., -size, 0.)),
        size,
        f,
        palette.0,
    );
    draw_pent(
        &draw.rotate(PI).translate(Vec3::new(0., -size, 0.)),
        size,
        f,
        palette.1,
    );
    draw_pent(
        &draw.rotate(PI / 3.).translate(Vec3::new(0., -size, 0.)),
        size,
        f,
        palette.1,
    );
    draw_pent(
        &draw
            .rotate(2. * PI / 3.)
            .translate(Vec3::new(0., -size, 0.)),
        size,
        f,
        palette.0,
    );
    draw_pent(
        &draw
            .rotate(4. * PI / 3.)
            .translate(Vec3::new(0., -size, 0.)),
        size,
        f,
        palette.0,
    );
    draw_pent(
        &draw
            .rotate(5. * PI / 3.)
            .translate(Vec3::new(0., -size, 0.)),
        size,
        f,
        palette.1,
    );
}
