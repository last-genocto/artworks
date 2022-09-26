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
        let [w, _h] = self.base.texture.size();
        draw.background().color(BLACK);

        let _seed = (self.base.seed % 1000) as f64 / 1000.;

        let centre = pt3(0.0, 0.0, 0.0);
        let size = vec3(1.0, 1.0, 1.0);
        let cuboid = geom::Cuboid::from_xyz_whd(centre, size);
        let points = cuboid
            .triangles_iter()
            .flat_map(geom::Tri::vertices)
            .map(|point| {
                // Tex coords should be in range (0.0, 0.0) to (1.0, 1.0);
                // This will have the logo show on the front and back faces.
                point
            });
        let cube_side = w as f32 * 0.5;
        draw.scale(cube_side)
            .mesh()
            .points(points)
            .stroke_color(WHITE)
            .z_radians(time as f32 * 0.33)
            .x_radians(time as f32 * 0.166)
            .y_radians(time as f32 * 0.25);
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
