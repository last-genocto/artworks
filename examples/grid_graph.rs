use artworks::{make_recorder_app, Artwork, BaseModel, Options};
use nannou::prelude::*;
use rand::Rng;
use rust_ca::{
    automaton::{Automaton, AutomatonImpl},
    rule::Rule,
};

fn main() {
    make_recorder_app::<Model>().run();
}

struct Model {
    pub base: BaseModel,
    graph: force_graph::ForceGraph,
    node_indices: Vec<force_graph::DefaultNodeIdx>,
    ca: Automaton,
    counter: u32,
}

const N_POINTS: usize = 50;

impl Artwork for Model {
    fn draw_at_time(&mut self, time: f64) {
        // First, reset the `draw` state.
        let draw = &self.base.draw.scale(0.7);
        draw.reset();
        draw.background()
            .color(srgba(0.08627, 0.08627, 0.08627, 1.));
        let [w, h] = self.base.texture.size();

        let _seed = (self.base.seed % 1000) as f64 / 1000.;
        let mut pos = vec![(0., 0.); N_POINTS.pow(2)];
        let mut draw_edges: Vec<(usize, usize, f32)> = vec![];
        let step = 4;
        let st_jmp = 8.;
        let max_mass = 2.;
        let min_mass = 0.3;

        if self.counter == step {
            self.ca.update();
            self.counter = 0;
        } else {
            self.counter += 1;
        }
        self.graph.visit_nodes_mut(|node| {
            pos[node.index().index()] = (node.x(), node.y());
            let idx = node.index().index();
            if self.ca[idx] == 1 {
                if node.data.mass < max_mass {
                    node.data.mass += (max_mass - min_mass) / st_jmp;
                }
            } else {
                if node.data.mass > min_mass {
                    node.data.mass -= (max_mass - min_mass) / st_jmp;
                }
            };

            let wd = map_range(node.data.mass, min_mass, max_mass, 8., 65.);
            let alpha = map_range(node.data.mass, min_mass, max_mass, 0.2, 0.8);

            draw.rect()
                .color(srgba(1., 1., 1., alpha))
                .w_h(wd, wd)
                .x_y(node.x(), node.y());
            if self.ca[idx] == 1 {
                let alpha = map_range(node.data.mass, min_mass, max_mass, 0.0, 0.3);
                let idx = node.index().index();
                let j = idx % N_POINTS;
                let i = idx / N_POINTS;
                if i > 0 {
                    draw_edges.push((idx, (i - 1) * N_POINTS + j, alpha));
                    if j > 0 {
                        draw_edges.push((idx, (i - 1) * N_POINTS + j - 1, alpha));
                    }
                    if j < N_POINTS - 1 {
                        draw_edges.push((idx, (i - 1) * N_POINTS + j + 1, alpha));
                    }
                }
                if i < N_POINTS - 1 {
                    draw_edges.push((idx, (i + 1) * N_POINTS + j, alpha));
                    if j > 0 {
                        draw_edges.push((idx, (i + 1) * N_POINTS + j - 1, alpha));
                    }
                    if j < N_POINTS - 1 {
                        draw_edges.push((idx, (i + 1) * N_POINTS + j + 1, alpha));
                    }
                }
                if j > 0 {
                    draw_edges.push((idx, i * N_POINTS + j - 1, alpha));
                }
                if j < N_POINTS - 1 {
                    draw_edges.push((idx, i * N_POINTS + j + 1, alpha));
                }
            }
        });
        for (s, e, alpha) in draw_edges.iter() {
            draw.line()
                .color(srgba(1., 1., 1., *alpha))
                .caps_round()
                .weight(10.)
                .points(
                    Point2::new(pos[*s].0, pos[*s].1),
                    Point2::new(pos[*e].0, pos[*e].1),
                );
        }
        // if self.base.recording {
        self.graph.update(0.022);
        // }
        self.graph.visit_nodes_mut(|node| {
            let idx = node.index().index();
            let j = idx % N_POINTS;
            let i = idx / N_POINTS;
            node.data.x += 0.05 * (-node.x() + (w as f32) * (i as f32 / N_POINTS as f32 - 0.5));
            node.data.y += 0.05 * (-node.y() + (h as f32) * (j as f32 / N_POINTS as f32 - 0.5));
        })
    }
    fn n_sec(&self) -> Option<u32> {
        Some(20)
    }

    fn get_model(&self) -> &BaseModel {
        &self.base
    }
    fn get_mut_model(&mut self) -> &mut BaseModel {
        &mut self.base
    }
    fn new(base: BaseModel) -> Model {
        let [w, h] = base.texture.size();
        let mut graph = <force_graph::ForceGraph>::new(Default::default());
        let mut indices: Vec<force_graph::DefaultNodeIdx> = vec![];
        for i in 0..N_POINTS {
            for j in 0..N_POINTS {
                indices.push(graph.add_node(force_graph::NodeData {
                    x: (w as f32) * (i as f32 / N_POINTS as f32 - 0.5),
                    y: (h as f32) * (j as f32 / N_POINTS as f32 - 0.5),
                    mass: 0.5,
                    ..Default::default()
                }));
            }
        }
        for i in 0..N_POINTS {
            for j in 0..N_POINTS {
                let idx = i * N_POINTS + j;
                graph.add_edge(indices[idx], indices[i * N_POINTS + j], Default::default());
                if i > 0 {
                    graph.add_edge(
                        indices[idx],
                        indices[(i - 1) * N_POINTS + j],
                        Default::default(),
                    );
                    if j > 0 {
                        graph.add_edge(
                            indices[idx],
                            indices[(i - 1) * N_POINTS + (j - 1)],
                            Default::default(),
                        );
                    }
                    if j < N_POINTS - 1 {
                        graph.add_edge(
                            indices[idx],
                            indices[(i - 1) * N_POINTS + (j + 1)],
                            Default::default(),
                        );
                    }
                }
                if i < N_POINTS - 1 {
                    graph.add_edge(
                        indices[idx],
                        indices[(i + 1) * N_POINTS + j],
                        Default::default(),
                    );
                    if j > 0 {
                        graph.add_edge(
                            indices[idx],
                            indices[(i + 1) * N_POINTS + (j - 1)],
                            Default::default(),
                        );
                    }
                    if j < N_POINTS - 1 {
                        graph.add_edge(
                            indices[idx],
                            indices[(i + 1) * N_POINTS + (j + 1)],
                            Default::default(),
                        );
                    }
                }
                if j > 0 {
                    graph.add_edge(
                        indices[idx],
                        indices[i * N_POINTS + (j - 1)],
                        Default::default(),
                    );
                }
                if j < N_POINTS - 1 {
                    graph.add_edge(
                        indices[idx],
                        indices[i * N_POINTS + (j + 1)],
                        Default::default(),
                    );
                }
            }
        }
        let mut ca = Automaton::new(2, N_POINTS, Rule::gol());
        let o = N_POINTS.pow(2) / 2 + N_POINTS / 2;
        ca[o] = 1;
        ca[o - 1] = 1;
        ca[o - 2] = 1;
        ca[o - N_POINTS] = 1;
        ca[o - 2 * N_POINTS - 1] = 1;

        ca[o - 8] = 1;
        ca[o - 9] = 1;
        ca[o - 10] = 1;
        ca[o - N_POINTS - 8] = 1;
        ca[o - 2 * N_POINTS - 1 - 8] = 1;
        ca.random_init();

        Model {
            base,
            graph,
            node_indices: indices,
            ca,
            counter: 0,
        }
    }
}
