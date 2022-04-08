use artworks::{make_recorder_app, Artwork, BaseModel};
use nannou::prelude::*;

fn main() {
    make_recorder_app::<Model>().run();
}

struct Model {
    pub base: BaseModel,
}

impl Artwork for Model {
    fn draw_at_time(&mut self, time: f64) {
        // First, reset the `draw` state.
        let draw = &self.base.draw;
        draw.reset();
        let [_w, _h] = self.base.texture.size();
        draw.background()
            .color(srgba(0.08627, 0.08627, 0.08627, 1.));

        let _seed = (self.base.seed % 1000) as f64 / 1000.;
        draw.x_radians(time as f32 * PI / 3.)
        .rect()
            .color(WHITE)
            .x_y_z(0., 0., 400. * (8. * PI * time as f32).sin())
            .w_h(100. + 10. * time as f32, 100. + 10. * time as f32);
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
