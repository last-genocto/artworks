use artworks::{make_recorder_app, Artwork, BaseModel};
use nannou::prelude::*;

fn main() {
    make_recorder_app::<Model>().run();
}

struct Model {
    pub base: BaseModel,
}

const N_CIRCLES: usize = 3;
const N_LINES: usize = 5;

impl Artwork for Model {
    fn draw_at_time(&mut self, time: f64) {
        // First, reset the `draw` state.
        let drw = &self.base.draw;
        let draw = drw;
        draw.reset();
        let [w, h] = self.base.texture.size();
        let _seed = (self.base.seed % 1000) as f64 / 1000.;

        let bg = srgba(0.08627, 0.08627, 0.08627, 1.);
        draw.background().color(bg);

        let fov_y = std::f32::consts::FRAC_PI_2;
        let near = 0.01;
        let far = 20.0;
        let aspect_ratio = w as f32 / h as f32;
        let trans = Mat4::perspective_rh_gl(fov_y, aspect_ratio, near, far);
        for i in 0..N_CIRCLES {
            let inc = fmod((i as f32) / N_CIRCLES as f32 + time as f32, 1.);

            let x = 0.;
            let y = 50.; //w as f32 / 3.;
            let z = (inc) * (4. * w as f32 / 6.);
            let r = map_range(inc, 0., 1., 100., 100.);
            let transformed = trans.mul_vec4(Vec4::new(x, y, z, r));

            draw.ellipse()
                .radius(transformed[3])
                .no_fill()
                .stroke_weight(10.)
                .stroke_color(YELLOW)
                .xyz(transformed.truncate());
        }
        // Road side lines
        let quad = [
            trans.project_point3(Vec3::new(-1. * w as f32, h as f32, near + 0.01)),
            trans.project_point3(Vec3::new(-1. * w as f32, h as f32, 200.)),
            trans.project_point3(Vec3::new(w as f32, h as f32, 200.)),
            trans.project_point3(Vec3::new(w as f32, h as f32, near + 0.01)),
        ];
        draw.quad()
            .color(bg)
            .stroke_color(WHITE)
            .stroke_weight(10.)
                .points(quad[0], quad[1], quad[2], quad[3]);

        // Road lines
        for i in 0..N_LINES {
            let inc = fmod((i as f32) / N_CIRCLES as f32 + time as f32, 1.);
            let x = 5.;
            let y = 50.;
            let z1 = map_range(inc, 1., 0., 0.0001, far / 30.);
            let z2 = clamp_min(z1 - 0.1, 0.00001);

            let quad = [
                trans.project_point3(Vec3::new(-x, y, z1)),
                trans.project_point3(Vec3::new(x, y, z1)),
                trans.project_point3(Vec3::new(x, y, z2)),
                trans.project_point3(Vec3::new(-x, y, z2)),
            ].iter().map(|x| x.truncate()).collect::<Vec<Vec2>>();

            draw.quad()
                .color(YELLOW)
                .points(quad[0], quad[1], quad[2], quad[3]);
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
