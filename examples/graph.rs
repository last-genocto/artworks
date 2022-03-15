use artworks::{make_recorder_app, Artwork, BaseModel, Options};
use nannou::prelude::*;
use rand::Rng;

fn main() {
    make_recorder_app::<Model>().run();
}

struct Model {
    pub base: BaseModel,
    graph: force_graph::ForceGraph,
    node_indices: Vec<force_graph::DefaultNodeIdx>
}

const N_POINTS: usize = 128;
const N_EDGES: usize = 512;

impl Artwork for Model {
    fn draw_at_time(&mut self, time: f64) {
        // First, reset the `draw` state.
        let draw = &self.base.draw;
        draw.reset();
        draw.background()
            .color(srgba(0.08627, 0.08627, 0.08627, 1.));
        let [w, h] = self.base.texture.size();
        let _seed = (self.base.seed % 1000) as f64 / 1000.;
        // draw.ellipse()
        //     .color(srgba(1., 1., 1., 1.))
        //     .radius(10.)
        //     .x_y(w as f32 * (time as f32 - 0.5), 0.);

        self.graph.visit_nodes_mut(|node| {
            if node.index() == *self.node_indices.last().unwrap() {
                node.data.x = w as f32 * (time as f32 - 0.5);
                node.data.y = 0.;
            }
            draw.ellipse()
                .color(srgba(1., 1., 1., 1.))
                .radius(10.)
                .x_y(node.x(), node.y());
        });
        // if self.base.recording {
            self.graph.update(0.005);
        // }
    }

    fn get_model(&self) -> &BaseModel {
        &self.base
    }
    fn get_mut_model(&mut self) -> &mut BaseModel {
        &mut self.base
    }
    fn new(base: BaseModel) -> Model {
        let [w, h] = base.texture.size();
        let mut rng = rand::thread_rng();
        let mut graph = <force_graph::ForceGraph>::new(Default::default());
        let mut indices: Vec<force_graph::DefaultNodeIdx> = vec![];
        for _ in 0..N_POINTS {
            let is_anchor = rand::random::<f32>() > 0.9;
            indices.push(graph.add_node(force_graph::NodeData {
                x: (w as f32 / 5.) * (fmod(rng.gen::<f32>(), 1.) - 0.5),
                y: (h as f32 / 5.) * (fmod(rng.gen::<f32>(), 1.) - 0.5),
                is_anchor,
                ..Default::default()
            }));
        }
        indices.push(graph.add_node(force_graph::NodeData {
            x: 0.,
            y: 0.,
            mass: 1000.,
            is_anchor: true,
            ..Default::default()
        }));
        for _ in 0..N_EDGES {
            graph.add_edge(
                indices[rng.gen_range(0..indices.len())],
                indices[rng.gen_range(0..indices.len())],
                Default::default(),
            );
        }
        Model { base, graph, node_indices: indices }
    }
}
