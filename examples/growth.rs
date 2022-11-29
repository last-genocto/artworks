use artworks::{make_recorder_app, Artwork, BaseModel};
// use kdtree::distance::squared_euclidean;
use kdtree::KdTree;
use nannou::prelude::*;

fn main() {
    make_recorder_app::<Model>().run();
}

struct Model {
    pub base: BaseModel,
    // kdtree: KdTree<f32, i32, [f32; 2]>,
}

impl Artwork for Model {
    fn draw_at_time(&mut self, _time: f64) {
        // First, reset the `draw` state.
        let draw = &self.base.draw;
        draw.reset();
        let [_w, _h] = self.base.texture.size();
        let _seed = (self.base.seed % 1000) as f64 / 1000.;
    }

    fn get_model(&self) -> &BaseModel {
        &self.base
    }
    fn get_mut_model(&mut self) -> &mut BaseModel {
        &mut self.base
    }
    fn new(base: BaseModel) -> Model {
        let mut kdtree = KdTree::new(2);
        for i in 0..10 {
            let theta = map_range(i, 0, 10, 0.0, TAU);
            let coords = [theta.cos(), theta.sin()];
            kdtree.add(coords, 0).unwrap();
        }
        Model {
            base,
            // kdtree
        }
    }
}
