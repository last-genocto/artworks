use artworks::{make_recorder_app, Artwork, BaseModel, Options};
use nannou::{
    color::Gradient,
    color::Srgb,
    ease::cubic::{self, ease_in},
    noise,
    noise::{NoiseFn, OpenSimplex},
    prelude::*,
    text::Font,
};

fn main() {
    make_recorder_app::<Model>().run();
}

struct Model {
    pub base: BaseModel,
    os: OpenSimplex,
    star_pos: Vec<(Vec2, f32, Srgb)>,
    shoot_star_pos: Vec<(Vec2, Srgb, Vec2, f32)>,
    rand: Vec<f32>,
    gal_coords: Vec<(f32, f32, f32, i32)>,
}

const N_CIRCLES: usize = 8;
const N_STARS: usize = 300;
const N_C_LAYERS: usize = 10;
const N_CP: usize = 60;
const N_LINES: usize = 7;
const N_BG: usize = 20;
const N_GALS: usize = 3;
const N_GAL: usize = 40;
const N_BR: usize = 5;
const N_GRS: usize = 7;

impl Artwork for Model {
    fn draw_at_time(&mut self, time: f64) {
        // First, reset the `draw` state.
        let drw = &self.base.draw;
        let draw = drw;
        draw.reset();
        let [w, h] = self.base.texture.size();
        let seed = (self.base.seed % 1000) as f64 / 1000.;
        let bg: Srgba = srgba(8. / 255., 37. / 255., 163. / 255., 1.);
        draw.background().color(bg);

        let fov_y = PI / 1.9;
        let near = 0.01;
        let far = 1.0;
        let aspect_ratio = w as f32 / h as f32;
        let trans = Mat4::perspective_rh(fov_y, aspect_ratio, near, far);
        let ffar = far * 0.8;

        let points = [
            (Point2::new(-1., -1.), (0., 0.)),
            (Point2::new(-1., 1.), (0., 1.)),
            (Point2::new(1., 1.), (1., 1.)),
            (Point2::new(1., -1.), (1., 0.)),
        ];
        draw.translate(Vec3::new(0., 0., -800.))
            .scale(w as f32 / 2.)
            .polygon()
            .points_textured(&self.base.extra_tex.as_ref().unwrap()[0], points);
        // Moon
        // let x_m = w as f32 / 3.; // + (TAU * time as f32).cos() * w as f32 / 8.;
        // let y_m = w as f32 / 10.; // + (TAU * time as f32).sin() * w as f32 / 3.;
        // let r_m = w as f32 / 13.;
        // draw.ellipse()
        //     .color(srgba(0.9, 0.9, 0.9, 0.9))
        //     .radius(r_m)
        //     .x_y(x_m, y_m);
        // for i in 0..20 {
        //     let alph = 1. / 10.;
        //     let r = map_range(i as f32, 0., 20., r_m, r_m + r_m / 10.);
        //     draw.ellipse()
        //         .color(srgba(0.9, 0.9, 0.9, alph))
        //         .radius(r)
        //         .x_y(x_m, y_m);
        // }

        // Stars
        for (s, off, c) in self.star_pos.iter() {
            let c = c.into_linear();
            let p = ((*s) - 0.5) * w as f32;
            let r = 1. + 4. * (3. * TAU * (time as f32 - off)).sin();
            draw.ellipse().color(c).radius(r).xy(p);
        }

        // Shooting stars
        let n_p = 10;
        for (s, c, rd, time_shine) in self.shoot_star_pos.iter() {
            let c = c.into_linear();
            let p_base = ((*s) - 0.5) * w as f32;
            for t in 0..=n_p {
                let q = clamp(
                    map_range(t as f32, 0., n_p as f32, time, time - 0.005),
                    0.,
                    1.,
                );
                let r_mul = map_range(t as f32, 0., n_p as f32, 1., 0.);
                let add = map_range(q, 0., 1., 0., w as f32);
                let p = p_base + *rd * add;
                let p = Point2::new(
                    fmod(p.x, w as f32) - w as f32 / 2.,
                    fmod(p.y, w as f32) - w as f32 / 2.,
                );
                // Pulse of 0.01 time units
                let pulse_time = 0.03;
                let norm = (-(pulse_time / 2.) * (-pulse_time / 2.)).powf(1. / 15.);
                let fac = clamp(
                    (-(time as f32 - time_shine - pulse_time / 2.)
                        * (time as f32 - time_shine + pulse_time / 2.))
                        .powf(1. / 15.)
                        / norm,
                    0.,
                    1.,
                );
                let r = (1. + 5. * fac) * r_mul;
                draw.ellipse().color(c).radius(r).xy(p);
            }
        }

        // Galaxy
        for (x, y, s, sp) in self.gal_coords.iter() {
            let base_x = 0. + x * w as f32 / 2.;
            let base_y = 1.3 * w as f32 / 4. + y * w as f32 / 4.4;
            let scale = w as f32 / 15. + s * w as f32 / 25.;
            self.make_galaxy(
                draw, w as f32, time, base_x, base_y, seed, scale, *sp as f32,
            );
        }

        // Background mountains
        self.draw_mountains(draw, seed, w as f32, h as f32, time);

        // Gold circles
        self.draw_circles(draw, time, near, ffar, &trans);
        // Road side lines
        // let n_tiles = N_LINES * 10;
        // for p in 0..=n_tiles {
        //     let depth = map_range(p as f32 + time as f32, 0., n_tiles as f32, near + 0.01, 70. * far);
        //     let depth2 = map_range(p as f32 + 1., 0., n_tiles as f32, near + 0.01, 70. * far);
        //     let x = 1.56 * w as f32;
        //     let y = h as f32;
        //     let quad: Vec<Vec2> = vec![
        //         trans.project_point3(Vec3::new(-x, y, depth)),
        //         trans.project_point3(Vec3::new(-x, y, depth2)),
        //         trans.project_point3(Vec3::new(x, y, depth2)),
        //         trans.project_point3(Vec3::new(x, y, depth)),
        //     ].iter().map(|x| x.truncate()).collect();
        //     let alpha = map_range(p as f32, 0., n_tiles as f32, 1., 0.);
        //     let col: Srgba = srgba(49. / 255., 12. / 255., 50. / 255., alpha);
        //     draw.line().color(WHITE).points(quad[0], quad[1]);
        //     draw.line().color(WHITE).points(quad[3], quad[2]);
        //     // draw.polygon().color(col).points(quad);
        // }

        // Road lines
        let x = 1.52 * w as f32;
        let y1 = h as f32;
        let y2 = -8. * (h as f32);
        let z1 = 70. * far;
        let z2 = 0.00001;
        let quad = [
            trans.project_point3(Vec3::new(-x, y2, z1)),
            trans.project_point3(Vec3::new(x, y2, z1)),
            trans.project_point3(Vec3::new(x, y1, z2)),
            trans.project_point3(Vec3::new(-x, y1, z2)),
        ]
        .iter()
        .map(|x| x.truncate())
        .collect::<Vec<Vec2>>();
        let col: Srgba = srgba(0., 0., 0., 0.9);
        draw.polygon().color(col).points(quad);
        for i in 0..=N_LINES + 3 {
            let inc = fmod((i as f32) / N_CIRCLES as f32 + time as f32, 1.);
            let x = 1.2;
            let y1 = map_range(inc, 1., 0., 20., -8. * 20.);
            let y2 = map_range(inc - 0.05, 1., 0., 20., -8. * 20.);
            let z1 = map_range(inc, 1., 0., 0.0001, ffar);
            let z2 = clamp_min(z1 - 0.05, 0.00001);
            let alpha: f32 = map_range(inc, 1., 0., 0.4, 0.);

            let quad = [
                trans.project_point3(Vec3::new(-x, y2, z1)),
                trans.project_point3(Vec3::new(x, y2, z1)),
                trans.project_point3(Vec3::new(x, y1, z2)),
                trans.project_point3(Vec3::new(-x, y1, z2)),
            ]
            .iter()
            .map(|x| x.truncate())
            .collect::<Vec<Vec2>>();
            let points = [(0., 0.), (0., 1.), (1., 1.), (1., 0.)];

            let col: Srgba = srgba(255. / 255., 255. / 255., 255. / 255., alpha);
            draw.polygon().color(col).points_textured(
                &self.base.extra_tex.as_ref().unwrap()[2],
                quad.clone().into_iter().zip(points.into_iter()),
            );
            draw.polygon().color(col).points(quad);
        }
        let n_tiles = N_LINES * 5;
        for i in 0..=n_tiles {
            let inc = fmod((i as f32) / n_tiles as f32 + time as f32, 1.);
            let x = 1.48 * w as f32;
            let y1 = map_range(inc, 1., 0., h as f32, -8. * (h as f32));
            let y2 = map_range(inc - 0.008, 1., 0., h as f32, -8. * (h as f32));
            let z1 = map_range(inc, 1., 0., 0.0001, 70. * far);
            let z2 = clamp_min(z1 - 0.5, 0.00001);
            let alpha: f32 = 0.8;

            let quad = [
                trans.project_point3(Vec3::new(-x, y2, z1)),
                trans.project_point3(Vec3::new(x, y2, z1)),
                trans.project_point3(Vec3::new(x, y1, z2)),
                trans.project_point3(Vec3::new(-x, y1, z2)),
            ]
            .iter()
            .map(|x| x.truncate())
            .collect::<Vec<Vec2>>();

            let col: Srgba = srgba(255. / 255., 255. / 255., 255. / 255., alpha);
            // let col: Srgba = match i % 3 {
            //    0 => srgba(60./255., 127./255., 205./255., alpha),
            //    1 => srgba(56./255., 54./255., 160./255., alpha),
            //    _ => srgba(160./255., 52./255., 95./255., alpha),
            // };
            let wg = map_range(inc, 1., 0., 20., 0.);
            draw.line().caps_round().weight(wg).color(col).points(quad[0], quad[3]);
            draw.line().caps_round().weight(wg).color(col).points(quad[1], quad[2]);
        }
        make_text(draw, w as f32, h as f32);
    }

    fn key_pressed(&mut self, _app: &App, key: Key) {
        match key {
            Key::S => {
                let os = noise::OpenSimplex::new();
                let cols = [
                    srgb(1., 1., 1.),
                    srgb(1., 1., 0.),
                    srgb(1., 1., 96. / 255.),
                    srgb(1., 1., 146. / 255.),
                    srgb(1., 147. / 255., 0.),
                ];
                let stars = (0..N_STARS)
                    .map(|_| {
                        (
                            Vec2::new(random_range(0., 1.), random_range(0., 1.)),
                            random_range::<f32>(0., 1.),
                            cols[random_range(0, cols.len())],
                        )
                    })
                    .collect();
                let shoot_stars = (0..N_STARS / 8)
                    .map(|_| {
                        let mut speed_vec =
                            Vec2::new(random_range(-5., 5.), random_range(-1., 1.)).normalize();
                        speed_vec *= random_range(5., 10.);
                        (
                            Vec2::new(random_range(0., 1.), random_range(0., 1.)),
                            cols[random_range(0, cols.len())],
                            speed_vec,
                            random_range::<f32>(0., 1.),
                        )
                    })
                    .collect();
                let rand = (0..N_GAL * N_BR).map(|_| random_range(-1., 1.)).collect();
                let gs = (0..N_GALS)
                    .map(|_| {
                        let base_x = random_range(-1., 1.);
                        let base_y = random_range(-1., 1.);
                        let scale = random_range(0.1, 1.);
                        let speed = random_range(-2, 3);

                        (base_x, base_y, scale, speed)
                    })
                    .collect();
                self.os = os;
                self.star_pos = stars;
                self.rand = rand;
                self.gal_coords = gs;
                self.shoot_star_pos = shoot_stars;
            }
            _ => {}
        }
    }

    fn get_model(&self) -> &BaseModel {
        &self.base
    }

    fn get_mut_model(&mut self) -> &mut BaseModel {
        &mut self.base
    }

    fn n_sec(&self) -> Option<u32> {
        Some(11)
    }

    fn new(base: BaseModel) -> Model {
        let os = noise::OpenSimplex::new();
        let cols = [
            srgb(1., 1., 1.),
            srgb(1., 1., 0.),
            srgb(1., 1., 96. / 255.),
            srgb(1., 1., 146. / 255.),
            srgb(1., 147. / 255., 0.),
        ];
        let stars = (0..N_STARS)
            .map(|_| {
                (
                    Vec2::new(random_range(0., 1.), random_range(0., 1.)),
                    random_range::<f32>(0., 1.),
                    cols[random_range(0, cols.len())],
                )
            })
            .collect();
        let shoot_stars = (0..N_STARS / 8)
            .map(|_| {
                let mut speed_vec = Vec2::new(random_range(-5., 5.), random_range(-0.2, 0.8));
                while speed_vec.x.abs() < 2. {
                    speed_vec.x = random_range(-5., 5.);
                }
                speed_vec = speed_vec.normalize();
                speed_vec *= random_range(5., 15.);
                (
                    Vec2::new(random_range(0., 1.), random_range(0., 1.)),
                    cols[random_range(0, cols.len())],
                    speed_vec,
                    random_range::<f32>(0., 1.),
                )
            })
            .collect();
        let rand = (0..N_GAL * N_BR).map(|_| random_range(-1., 1.)).collect();
        let gs = (0..N_GALS)
            .map(|_| {
                let base_x = random_range(-1., 1.);
                let base_y = random_range(-1., 1.);
                let scale = random_range(0.1, 1.);
                let speed = random_range(-2, 3);

                (base_x, base_y, scale, speed)
            })
            .collect();
        Model {
            base,
            os,
            star_pos: stars,
            rand,
            gal_coords: gs,
            shoot_star_pos: shoot_stars,
        }
    }

    fn get_options() -> Option<Options> {
        Some(Options {
            chroma: 0.5,
            sample_per_frame: 10,
            shutter_angle: 0.5,
            extra_tex: Some(vec![
                "tst.jpg".to_string(),
                "halo.png".to_string(),
                "road.jpg".to_string(),
            ]),
            noise_amout: 0.01
        })
    }
}

impl Model {
    fn draw_mountains(&self, draw: &Draw, seed: f64, w: f32, h: f32, time: f64) {
        let g = Gradient::new([
            lin_srgba(1., 0., 0.9, 0.),
            lin_srgba(1., 0., 0.9, 0.3),
            lin_srgba(0., 0., 0., 1.),
        ]);
        self.draw_single_line_mountains(draw, seed, w, h, time, &g, 0.);
        self.draw_single_line_mountains(draw, seed * 8.4 + 200., w, h, time, &g, 50.);
        self.draw_single_line_mountains(draw, seed * 2.8 - 100., w, h, time, &g, 100.);
    }

    fn draw_single_line_mountains(
        &self,
        draw: &Draw,
        seed: f64,
        w: f32,
        h: f32,
        time: f64,
        g: &Gradient<LinSrgba>,
        baseline_y: f32,
    ) {
        let scale = 300.;
        let os = self.os;
        for a in 0..=N_GRS {
            let val = map_range(a, 0, N_GRS, 0., 1.);
            let add_y = map_range(val, 0., 1., 40., 0.);
            let col = g.get(ease_in(val, 0., 1., 1.));
            let mut pts: Vec<Vec2> = (0..=N_BG)
                .map(|i| {
                    let x = map_range(i as f32, 0., N_BG as f32, 0., w as f32 / 2. - w as f32 / 6.)
                        - w as f32 / 2.;
                    let mut y = 0.1
                        * os.get([
                            seed + (4. * TAU as f64 * time + (10. * x / scale) as f64).cos(),
                            (4. * TAU as f64 * time + (10. * x / scale) as f64).sin(),
                            // (TAU as f64 * time).sin(),
                            // (TAU as f64 * time).cos(),
                        ]) as f32
                        + 0.5 * os.get([seed, (2. * x / scale) as f64]) as f32
                        + 1. * os.get([seed, (x / scale) as f64]) as f32
                        + 0.01 * os.get([seed, (100. * x / scale) as f64]) as f32;
                    y += 0.5;
                    y *= w as f32 / 5.;
                    y *= ((i as i32 - N_BG as i32) as f32).abs().powf(0.6) / 10.;
                    y *= map_range(i as f32, 0., N_BG as f32, 1.5, 1.);
                    Vec2::new(x, y + add_y - baseline_y)
                })
                .collect();
            pts.push(Vec2::new(0., -(h / 2.)));
            pts.push(Vec2::new(-w / 2., -(h / 2.)));
            draw.path().fill().color(col).points_closed(pts);
            let mut pts: Vec<Vec2> = (0..=N_BG)
                .map(|i| {
                    let x = map_range(
                        i as f32,
                        0.,
                        N_BG as f32,
                        w as f32 / 2. + w as f32 / 6.,
                        w as f32,
                    ) - w as f32 / 2.;
                    let mut y = 0.1
                        * os.get([
                            seed + (4. * TAU as f64 * time + (10. * x / scale) as f64).cos(),
                            (4. * TAU as f64 * time + (10. * x / scale) as f64).sin(),
                            // (TAU as f64 * time).sin(),
                            // (TAU as f64 * time).cos(),
                        ]) as f32
                        + 0.5 * os.get([seed, (2. * x / scale) as f64]) as f32
                        + 1. * os.get([seed, (x / scale) as f64]) as f32
                        + 0.01 * os.get([seed, (100. * x / scale) as f64]) as f32;
                    y += 0.5;
                    y *= w as f32 / 5.;
                    y *= (i as f32).abs().powf(0.6) / 10.;
                    y *= map_range(i as f32, 0., N_BG as f32, 1., 1.5);
                    Vec2::new(x, y + add_y - baseline_y)
                })
                .collect();
            pts.push(Vec2::new(w / 2., -(h / 2.)));
            pts.push(Vec2::new(0., -(h / 2.)));

            draw.path().fill().color(col).points_closed(pts);
        }
        draw.path().fill().color(BLACK).points_closed([
            Vec2::new(-w / 2., -(h / 2.)),
            Vec2::new(w / 2., -(h / 2.)),
            Vec2::new(w / 2., -baseline_y),
            Vec2::new(-w / 2.,-baseline_y),
        ]);
    }

    fn make_galaxy(
        &self,
        draw: &Draw,
        w: f32,
        time: f64,
        base_x: f32,
        base_y: f32,
        seed: f64,
        scale: f32,
        speed: f32,
    ) {
        let t_m = N_BR;
        for t in 0..t_m {
            let off = map_range(t as f32, 0., t_m as f32, 0., TAU);
            for i in 0..N_GAL {
                let thet = map_range(i as f32, 0., N_GAL as f32, 0., PI);
                let tt = self.os.get([
                    seed + (TAU as f64 * time).cos(),
                    i as f64,
                    t as f64,
                    (TAU as f64 * time).sin(),
                ]);
                let thet = thet + PI * tt as f32 / 2.;
                let r = scale * thet / TAU;
                let tr = Vec3::new(base_x, base_y, 0.);
                let disx = (w as f32 / (r + 600.)) * self.rand[t * N_GAL + i];
                let disy = (w as f32 / (r + 600.)) * self.rand[t * N_GAL + i];
                let disz = (w as f32 / (r + 600.)) * self.rand[t * N_GAL + i];
                let x = base_x + r * thet.cos() + disx;
                let y = base_y + r * thet.sin() + disy;
                draw.z(-100.)
                    .translate(tr)
                    .x_radians(PI / 3.)
                    .z_radians(off + speed * time as f32 * TAU)
                    .translate(-tr)
                    .ellipse()
                    .color(srgba(1., 1., 1., 0.5))
                    .radius(3.)
                    .x_y_z(x, y, disz);
            }
        }
    }

    fn draw_circles(&self, draw: &Draw, time: f64, near: f32, ffar: f32, trans: &Mat4) {
        for i in 0..=N_CIRCLES {
            let inc = fmod((i as f32 - 0.2) / N_CIRCLES as f32 + time as f32, 1.);

            let x = 0.;
            let y = -20.; //w as f32 / 3.;
            let y = map_range(inc, 1., 0., -25., -8. * (147.));
            let z = map_range(inc, 1., 0., near, 5. * ffar);
            let thet_min = PI / 3.7;
            let thet_max = TAU - thet_min;
            let alpha: f32 = cubic::ease_in(map_range(inc, 1., 0., 0., 1.), 0., 1., 1.);

            for u in 0..N_C_LAYERS {
                let r = map_range(u, 0, N_C_LAYERS - 1, 100., 110.);
                let add = map_range(u, 0, N_C_LAYERS - 1, 0., 0.006);
                let r_tex = map_range(u, 0, N_C_LAYERS - 1, 0.55, 0.5);
                let points = (0..=N_CP)
                    .map(|p| {
                        let theta = map_range(p, 0, N_CP, thet_min, thet_max);
                        (
                            Vec3::new(x + r * theta.sin(), y + r * theta.cos(), z + add),
                            Vec2::new(0.5 + r_tex * theta.sin(), 0.5 + r_tex * theta.cos()),
                        )
                    })
                    .map(|(p1, p2)| (trans.project_point3(p1), p2));
                let col: Srgba = srgba(0., 0., 0., alpha);

                draw.path()
                    .stroke()
                    .caps_round()
                    .color(col)
                    .stroke_weight(20.)
                    .points_textured(&self.base.extra_tex.as_ref().unwrap()[1], points.clone());
                draw.path()
                    .stroke()
                    .caps_round()
                    .color(col)
                    .stroke_weight(20.)
                    .points(points.into_iter().map(|(x, _)| x));
            }
        }
    }
}

fn make_text(draw: &Draw, w: f32, h: f32) {
    let bbox_w = w / 2.3;
    let x1 = -700.;
    let y_off = -160.;
    let x2 = 600.;
    let f_size = 90;
    let y = -0.8 * w / 3.;

    let font_data: &[u8] = include_bytes!("/Users/hugo/Library/Fonts/space age.ttf");
    let font: Font = Font::from_bytes(font_data).unwrap();

    let rect = Rect::from_x_y_w_h(x1, y, bbox_w, h / 2.).pad(20.);
    draw.text("Bruni")
        .right_justify()
        .font(font.clone())
        .align_text_middle_y()
        .font_size(f_size)
        .xy(rect.xy())
        .wh(rect.wh());
    let rect = Rect::from_x_y_w_h(x1, y + y_off, bbox_w, h / 2.).pad(20.);
    draw.text("04.26.22")
        .right_justify()
        .font(font.clone())
        .align_text_middle_y()
        .font_size(f_size)
        .xy(rect.xy())
        .wh(rect.wh());
    let rect = Rect::from_x_y_w_h(x2, y, bbox_w, h / 2.).pad(20.);
    draw.text("Ottobar")
        .left_justify()
        .font(font.clone())
        .align_text_middle_y()
        .font_size(f_size)
        .xy(rect.xy())
        .wh(rect.wh());
    let rect = Rect::from_x_y_w_h(x2, y + y_off, bbox_w, h / 2.).pad(20.);
    draw.text("Baltimore")
        .left_justify()
        .font(font.clone())
        .align_text_middle_y()
        .font_size(f_size)
        .xy(rect.xy())
        .wh(rect.wh());
}
