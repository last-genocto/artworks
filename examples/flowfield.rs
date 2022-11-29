use artworks::{make_recorder_app, Artwork, BaseModel, Options};
use colourado::{Color, ColorPalette, PaletteType};
use nannou::{noise, noise::NoiseFn, prelude::*};

fn main() {
    make_recorder_app::<Model>().run();
}

struct Model {
    pub base: BaseModel,
    flow: GridFlow,
}

#[derive(Clone)]
struct LinePath {
    points: Vec<Point2>,
    color: Color,
    max_len: usize,
    adding: bool,
    weight: f32,
    alpha: f32,
}

fn draw_path(draw: &Draw, l_path: &LinePath) {
    let color = l_path.color;
    draw.scale(0.7)
        .path()
        .stroke()
        .caps_round()
        .weight(l_path.weight)
        .color(srgba(color.red, color.green, color.blue, l_path.alpha))
        .points(l_path.points.clone());
}

struct GridFlow {
    init_points: Vec<LinePath>,
    points: Vec<LinePath>,
    noise: noise::OpenSimplex,
    width: f32,
    palette: ColorPalette,
}

const N_LINES: u32 = 50;
const L_MIN: usize = 50;
const L_MAX: usize = 300;

impl GridFlow {
    fn update(&mut self, fact: f32, tau: f64, time: f64) {
        let w = self.width;
        for (n, p) in self.points.iter_mut().enumerate() {
            let mut use_init = false;
            if time > 0.6 {
                p.adding = false;
                if p.points.len() > 1 {
                    let vec = self.init_points[n].points[0];
                    for v in p.points.iter_mut() {
                        *v -= 0.0001 * (*v - vec);
                    }
                    p.points.remove(p.points.len() - 1);
                } else {
                    *p = self.init_points[n].clone();
                }
                use_init = true;
            } else {
                if (p.adding == true) && (p.points.len() < p.max_len) {
                    let last_p = p.points.last().unwrap().clone();
                    p.points.push(Point2::new(
                        last_p.x
                            - fact
                                * self.noise.get([
                                    last_p.x as f64 / (5. * N_LINES as f64),
                                    last_p.y as f64 / (5. * N_LINES as f64),
                                    tau.cos() / 5.,
                                    tau.sin() / 5.,
                                ]) as f32,
                        last_p.y
                            - fact
                                * self.noise.get([
                                    last_p.x as f64 / (5. * N_LINES as f64),
                                    last_p.y as f64 / (5. * N_LINES as f64),
                                    tau.cos() / 5. + 2.,
                                    tau.sin() / 5. - 1.,
                                ]) as f32,
                    ));
                } else if p.points.len() > 1 {
                    p.adding = false;
                    p.points.remove(0);
                } else {
                    p.adding = true;
                    let mut x = random_pos(w as u32);
                    let mut y = random_pos(w as u32);
                    while (x.powf(2.) + y.powf(2.)).sqrt() >= w / 2. {
                        x = random_pos(w as u32);
                        y = random_pos(w as u32);
                    }
                    p.points = vec![Point2::new(x, y)];

                    p.max_len = random_range(L_MIN, L_MAX);
                    p.color = self.palette.colors[random_range(0, self.palette.colors.len())];
                    p.weight = random_range(10., 100.);
                }
            }
            p.alpha = if p.adding || use_init {
                map_range(p.points.len(), 0, p.max_len, 0.2, 0.7)
            } else {
                map_range(p.points.len(), 0, p.max_len, 0.0, 0.7)
            };
        }
    }
}

impl Artwork for Model {
    fn draw_at_time(&mut self, time: f64) {
        // First, reset the `draw` state.
        let draw = &self.base.draw;
        draw.reset();
        let [_w, _h] = self.base.texture.size();
        let _seed = (self.base.seed % 1000) as f64 / 1000.;
        draw.background()
            .color(srgba(0.08627, 0.08627, 0.08627, 1.));

        for p in self.flow.points.iter() {
            draw_path(&draw, p);
            // draw.ellipse().xy(*p).radius(2.);
        }
        self.flow.update(5., TAU as f64 * time, time);
    }

    fn get_model(&self) -> &BaseModel {
        &self.base
    }
    fn get_mut_model(&mut self) -> &mut BaseModel {
        &mut self.base
    }
    fn new(base: BaseModel) -> Model {
        let [w, _h] = base.texture.size();
        let palette = ColorPalette::new(4, PaletteType::Random, false);
        let mut vec = vec![];
        for _ in 0..N_LINES.pow(2) {
            let x = random_pos(w);
            let y = random_pos(w);
            if (x.powf(2.) + y.powf(2.)).sqrt() < w as f32 / 2. {
                vec.push(LinePath {
                    points: vec![Point2::new(x, y)],
                    max_len: random_range(L_MIN, L_MAX),
                    adding: true,
                    color: palette.colors[random_range(0, palette.colors.len())],
                    weight: random_range(10., 100.),
                    alpha: 0.1,
                });
            }
        }
        let flow = GridFlow {
            init_points: vec.clone(),
            points: vec,
            noise: noise::OpenSimplex::new(),
            width: w as f32,
            palette,
        };
        Model { base, flow }
    }

    fn n_sec(&self) -> Option<u32> {
        Some(15)
    }

    fn get_options() -> Option<Options> {
        Some(Options {
            chroma: 0.4,
            sample_per_frame: 1,
            shutter_angle: 0.3,
            extra_tex: None,
            noise_amount: 0.0,
        })
    }

    fn key_pressed(&mut self, _app: &App, key: Key) {
        match key {
            Key::P => {
                self.flow.palette = ColorPalette::new(4, PaletteType::Random, false);
                self.flow.points.iter_mut().for_each(|x| {
                    x.color =
                        self.flow.palette.colors[random_range(0, self.flow.palette.colors.len())]
                })
            }
            Key::R => {
                let mut vec = vec![];
                let [w, _h] = self.base.texture.size();
                for _ in 0..N_LINES.pow(2) {
                    let x = random_pos(w);
                    let y = random_pos(w);
                    if (x.powf(2.) + y.powf(2.)).sqrt() < w as f32 / 2. {
                        vec.push(LinePath {
                            points: vec![Point2::new(x, y)],
                            max_len: random_range(L_MIN, L_MAX),
                            adding: true,
                            color: self.flow.palette.colors
                                [random_range(0, self.flow.palette.colors.len())],
                            weight: random_range(10., 100.),
                            alpha: 0.1,
                        });
                    }
                }
                self.flow.init_points = vec.clone();
                self.flow.points = vec;
            }
            __ => {}
        }
    }
}

fn random_pos(w: u32) -> f32 {
    map_range(
        random::<f32>() * N_LINES as f32,
        0.,
        N_LINES as f32,
        -(w as f32) / 2.,
        w as f32 / 2.,
    )
}
