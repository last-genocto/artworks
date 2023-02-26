use artworks::{
    make_recorder_app,
    utils::colors::poline::{get_random_color_palette, get_random_color_palette3, PosFunctions},
    Artwork, BaseModel,
};
use nannou::prelude::*;

fn main() {
    make_recorder_app::<Model>().run();
}

struct Model {
    pub base: BaseModel,
    palettes: Vec<Vec<Hsla>>,
}

const N: usize = 14;
const P: usize = 14;

impl Artwork for Model {
    fn draw_at_time(&mut self, _time: f64) {
        // First, reset the `draw` state.
        let draw = &self.base.draw;
        draw.reset();

        // Get the width and height of the animation.
        let [w, _h] = self.base.texture.size();
        // Set the seed
        let _seed = (self.base.seed % 1000) as f64 / 1000.;
        draw.background()
            .color(srgba(0.08627, 0.08627, 0.08627, 1.));

        for c in 0..=N {
            for i in 0..=P {
                let x = map_range(i, 0, P, -(w as f32) / 3., w as f32 / 3.);
                draw.rect()
                    .color(self.palettes[c as usize][i])
                    .x_y(x, map_range(c, 0, N, -(w as f32) / 3., w as f32 / 3.));
            }
        }
    }

    fn get_model(&self) -> &BaseModel {
        &self.base
    }
    fn get_mut_model(&mut self) -> &mut BaseModel {
        &mut self.base
    }
    fn key_pressed(&mut self, _app: &App, key: Key) {
        match key {
            Key::S => {
                self.palettes = (0..=N)
                    .map(|_| get_random_color_palette3(P + 1, PosFunctions::LinearPosition))
                    .collect();
            }
            _ => {}
        }
    }
    fn new(base: BaseModel) -> Model {
        Model {
            base,
            palettes: (0..=N)
                .map(|_| get_random_color_palette(P + 1, PosFunctions::LinearPosition))
                .collect(),
        }
    }
}
