use artworks::{make_recorder_app, Artwork, BaseModel, Options};
use audrey::dasp_frame::Frame;
use nannou::{
    ease::{
        cubic::{ease_in, ease_in_out, ease_out},
        map_clamp,
    },
    prelude::*,
};

fn main() {
    make_recorder_app::<Model>().run();
}

struct Model {
    pub base: BaseModel,
    translatesx: Vec<f32>,
    translatesy: Vec<f32>,
    rotates: Vec<f32>,
    heights: Vec<f32>,
    widths: Vec<f32>,
}

const N_RECT: u8 = 6;

impl Artwork for Model {
    fn draw_at_time(&mut self, time: f64) {
        // First, reset the `draw` state.
        let draw = &self.base.draw;
        draw.reset();

        // Get the width and height of the animation.
        let [w, h] = self.base.texture.size();
        // Set the seed
        let _seed = (self.base.seed % 1000) as f64 / 1000.;
        draw.background()
            .color(srgba(0.08627, 0.08627, 0.08627, 1.));
        let chapters = [0.2, 0.4, 0.5, 0.8];
        if time < chapters[0] {
            let local_time = (time / chapters[0]) as f32;
            let local_time = ease_out(local_time, 0., 1., 1.);
            draw.ellipse()
                .color(srgba(219. / 255., 245. / 255., 137. / 255., 1.))
                .radius(w as f32 / 22.)
                .x_y(
                    (1. - local_time) * (w as f32),
                    (1. - local_time) * (w as f32),
                );
            draw.ellipse()
                .color(srgba(239. / 255., 225. / 255., 197. / 255., 1.))
                .radius(w as f32 / 12.)
                .x_y(
                    -(1. - local_time) * (w as f32),
                    -(1. - local_time) * (w as f32),
                );
        }
        if time >= chapters[0] && time < chapters[1] {
            let local_time = ((time - chapters[0]) / (chapters[1] - chapters[0])) as f32;
            let alpha = ease_in(local_time, 0., 1., 1.);
            draw.ellipse()
                .color(srgba(239. / 255., 225. / 255., 197. / 255., 1. - alpha))
                .radius((1. + 11. * local_time) * w as f32 / 12.)
                .x_y(0., 0.);
        }
        if time >= chapters[1] && time < chapters[2] {
            let local_time = ((time - chapters[1]) / (chapters[2] - chapters[1])) as f32;
            for c in 0..N_RECT {
                draw.rect()
                    .w_h(100., 100.)
                    .color(srgba(117. / 255., 17. / 255., 16. / 255., 1.))
                    .x_y(0., 0.);
            }
        }
        if time >= chapters[2] && time < chapters[3] {
            let local_time = clamp(
                ((time - chapters[2]) / (chapters[3] - chapters[2])) as f32,
                0.,
                1.0,
            );

            for c in 0..=N_RECT {
                draw.translate(Vec3::new(
                    map_clamp(
                        local_time,
                        0.,
                        1.,
                        0.,
                        self.translatesx[c as usize],
                        ease_in_out,
                    ),
                    map_clamp(
                        local_time,
                        0.,
                        1.,
                        0.,
                        self.translatesy[c as usize],
                        ease_in_out,
                    ),
                    0.,
                ))
                .rotate(map_clamp(
                    local_time,
                    0.,
                    1.,
                    0.,
                    self.rotates[c as usize],
                    ease_in_out,
                ))
                .rect()
                .height(map_clamp(
                    local_time,
                    0.,
                    1.,
                    100.,
                    self.heights[c as usize],
                    ease_in_out,
                ))
                .width(map_clamp(
                    local_time,
                    0.,
                    1.,
                    100.,
                    self.widths[c as usize],
                    ease_in_out,
                ))
                .color(srgba(
                    117. / 255.,
                    17. / 255.,
                    16. / 255.,
                    1.,
                    // map_clamp(time, 0.95, 1., 1., 0., ease_out),
                ))
                .x_y(0., 0.);
            }
        }
        if time >= chapters[3] {
            let local_time = clamp(((time - chapters[3]) / (1. - chapters[3])) as f32, 0., 1.0);

            for c in 0..=N_RECT {
                let c_time = clamp(
                    map_range(
                        local_time,
                        (c as f32 + 1.) / (N_RECT as f32 + 3.),
                        (c as f32 + 2.) / (N_RECT as f32 + 3.),
                        0.,
                        1.,
                    ),
                    0.,
                    1.1,
                );
                draw.translate(Vec3::new(
                    map_range(
                        c_time,
                        0.,
                        1.,
                        self.translatesx[c as usize],
                        self.translatesx[c as usize],
                    ),
                    map_range(
                        c_time,
                        0.,
                        1.,
                        self.translatesy[c as usize],
                        self.translatesy[c as usize] + 1.1 * w as f32,
                    ),
                    0.,
                ))
                .rotate(map_clamp(
                    c_time,
                    0.,
                    1.,
                    self.rotates[c as usize],
                    0.,
                    ease_in,
                ))
                .rect()
                .height(map_clamp(
                    c_time,
                    0.,
                    1.,
                    self.heights[c as usize],
                    100.,
                    ease_in,
                ))
                .width(map_clamp(
                    c_time,
                    0.,
                    1.,
                    self.widths[c as usize],
                    100.,
                    ease_in,
                ))
                .color(srgba(
                    117. / 255.,
                    17. / 255.,
                    16. / 255.,
                    1.,
                    // map_clamp(time, 0.95, 1., 1., 0., ease_out),
                ))
                .x_y(0., 0.);
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
        let [w, _h] = base.texture.size();
        Model {
            base,
            translatesx: (0..=N_RECT)
                .map(|c| {
                    let tr = map_range(c, 0, N_RECT, -(w as f32) / 3., w as f32 / 3.);
                    tr
                })
                .collect(),
            translatesy: (0..=N_RECT)
                .map(|c| {
                    let tr = map_range(c, 0, N_RECT, -(w as f32) / 3., w as f32 / 3.)
                        + (w as f32 / 12.) * (random_f32() * 2. - 1.);
                    tr
                })
                .collect(),
            rotates: (0..=N_RECT)
                .map(|c| -TAU * 45. / 360. + (2. * random_f32() - 1.) / 5.)
                .collect(),
            heights: (0..=N_RECT)
                .map(|c| (1. + 0.2 * (2. * random_f32() - 1.)) * w as f32 / 10.)
                .collect(),
            widths: (0..=N_RECT)
                .map(|c| {
                    let abs = map_range(
                        (c as f32 - N_RECT as f32 / 2.).abs(),
                        0.,
                        N_RECT as f32 / 2.,
                        2.,
                        6.,
                    );
                    (1. + 0.8 * (2. * random_f32() - 1.)) * w as f32 / abs
                })
                .collect(),
        }
    }
    fn key_pressed(&mut self, _app: &App, key: Key) {
        match key {
            Key::S => {
                let [w, _h] = self.base.texture.size();
                self.translatesx = (0..=N_RECT)
                    .map(|c| {
                        let tr = map_range(c, 0, N_RECT, -(w as f32) / 3., w as f32 / 3.)
                            + (w as f32 / 12.) * (random_f32() * 2. - 1.);
                        tr
                    })
                    .collect();
                self.translatesy = (0..=N_RECT)
                    .map(|c| {
                        let tr = map_range(c, 0, N_RECT, -(w as f32) / 3., w as f32 / 3.)
                            + (w as f32 / 12.) * (random_f32() * 2. - 1.);
                        tr
                    })
                    .collect();
                self.rotates = (0..=N_RECT)
                    .map(|c| -TAU * 45. / 360. + (2. * random_f32() - 1.) / 5.)
                    .collect();
                self.heights = (0..=N_RECT)
                    .map(|c| (1. + 0.2 * (2. * random_f32() - 1.)) * w as f32 / 10.)
                    .collect();
                self.widths = (0..=N_RECT)
                    .map(|c| {
                        let abs = map_range(
                            (c as f32 - N_RECT as f32 / 2.).abs(),
                            0.,
                            N_RECT as f32 / 2.,
                            2.,
                            6.,
                        );
                        (1. + 0.8 * (2. * random_f32() - 1.)) * w as f32 / abs
                    })
                    .collect();
            }

            _ => {}
        }
    }
    fn get_options() -> Option<Options> {
        Some(Options {
            chroma: 0.3,
            sample_per_frame: 6,
            shutter_angle: 1.,
            extra_tex: None,
            noise_amount: 0.1,
        })
    }
    fn n_sec(&self) -> Option<u32> {
        Some(15)
    }
}
