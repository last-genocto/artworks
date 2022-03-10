use nannou::{noise::NoiseFn, prelude::*, wgpu};

const FPS: u32 = 60;
const N_SEC: u32 = 10;

fn main() {
    nannou::app(model).update(update).exit(exit).run();
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Uniforms {
    chroma: f32,
    sample_per_frame: i32,
}

// The vertices that make up the rectangle to which the image will be drawn.
const VERTICES: [Vertex; 4] = [
    Vertex {
        position: [-1.0, 1.0],
    },
    Vertex {
        position: [-1.0, -1.0],
    },
    Vertex {
        position: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0],
    },
];

fn vertices_as_bytes(data: &[Vertex]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}

struct Model {
    sample_per_frame: i32,
    shutter_angle: f64,
    uniforms: wgpu::Buffer,
    texture_view: wgpu::TextureView,
    texture_accumulate_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,

    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    // The texture that we will draw to.
    texture: wgpu::Texture,
    texture_accumulate: wgpu::Texture,

    // Create a `Draw` instance for drawing to our texture.
    draw: nannou::Draw,
    // The type used to render the `Draw` vertices to our texture.
    renderer: nannou::draw::Renderer,
    // The type used to capture the texture.
    texture_capturer: wgpu::TextureCapturer,
    texture_reshaper1: wgpu::TextureReshaper,
    current_frame: u32,
    recording: bool,
    seed: i32,
}

fn model(app: &App) -> Model {
    // Lets write to a 4K UHD texture.
    let texture_size = [1080, 1080];

    // Create the window.
    let [win_w, win_h] = [texture_size[0] / 4, texture_size[1] / 4];
    let w_id = app
        .new_window()
        .size(win_w, win_h)
        .title("nannou")
        .view(view)
        .key_pressed(key_pressed)
        .build()
        .unwrap();
    let window = app.window(w_id).unwrap();

    // Retrieve the wgpu device.
    let device = window.device();
    // Create our custom texture.
    let sample_count = window.msaa_samples();
    let texture = wgpu::TextureBuilder::new()
        .size(texture_size)
        // Our texture will be used as the RENDER_ATTACHMENT for our `Draw` render pass.
        // It will also be SAMPLED by the `TextureCapturer` and `TextureResizer`.
        .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
        // Use nannou's default multisampling sample count.
        .sample_count(sample_count)
        // Use a spacious 16-bit linear sRGBA format suitable for high quality drawing.
        .format(wgpu::TextureFormat::Rgba16Float)
        // Build it!
        .build(device);

    let texture_accumulate = wgpu::TextureBuilder::new()
        .size(texture_size)
        // Our texture will be used as the RENDER_ATTACHMENT for our `Draw` render pass.
        // It will also be SAMPLED by the `TextureCapturer` and `TextureResizer`.
        .usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT)
        // Use nannou's default multisampling sample count.
        .sample_count(sample_count)
        // Use a spacious 16-bit linear sRGBA format suitable for high quality drawing.
        .format(wgpu::TextureFormat::Rgba16Float)
        // Build it!
        .build(device);
    let texture_view = texture.view().build();
    let texture_accumulate_view = texture_accumulate.view().build();

    // Create our `Draw` instance and a renderer for it.
    let draw = nannou::Draw::new();
    let descriptor = texture.descriptor();
    let renderer =
        nannou::draw::RendererBuilder::new().build_from_texture_descriptor(device, descriptor);

    let vs_desc = wgpu::include_wgsl!("shaders/vs.wgsl");
    let fs_desc = wgpu::include_wgsl!("shaders/fs.wgsl");
    let vs_mod = device.create_shader_module(&vs_desc);
    let fs_mod = device.create_shader_module(&fs_desc);

    let sampler_desc = wgpu::SamplerBuilder::new()
        .label(Some("The sampler"))
        .into_descriptor();
    let sampler_filtering = wgpu::sampler_filtering(&sampler_desc);
    let sampler = device.create_sampler(&sampler_desc);

    let bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
        .texture(
            wgpu::ShaderStages::FRAGMENT,
            true,
            wgpu::TextureViewDimension::D2,
            texture_view.sample_type(),
        )
        .sampler(wgpu::ShaderStages::FRAGMENT, sampler_filtering)
        .uniform_buffer(wgpu::ShaderStages::FRAGMENT, false)
        .build(device);

    let sample_per_frame = 5;
    let shutter_angle = 0.3;
    let uniforms = Uniforms {
        chroma: 0.3,
        sample_per_frame,
    };

    let uniforms_bytes = uniforms_as_bytes(&uniforms);
    let usage = wgpu::BufferUsages::UNIFORM;
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: uniforms_bytes,
        usage,
    });

    let desc = wgpu::PipelineLayoutDescriptor {
        label: Some("The pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    };
    let pipeline_layout = device.create_pipeline_layout(&desc);
    let render_pipeline =
        nannou::wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
            .fragment_shader(&fs_mod)
            .color_format(texture_accumulate.format())
            .color_blend(wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            })
            .alpha_blend(wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            })
            .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float32x2])
            .sample_count(sample_count)
            .primitive_topology(wgpu::PrimitiveTopology::TriangleStrip)
            .build(device);

    let vertices_bytes = vertices_as_bytes(&VERTICES[..]);
    let usage = wgpu::BufferUsages::VERTEX;
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: vertices_bytes,
        usage,
    });

    // Create the texture capturer.
    let texture_capturer = wgpu::TextureCapturer::default();

    let texture_sample_type = texture_accumulate.sample_type();
    let dst_format = texture_accumulate.format();
    let texture_reshaper1 = wgpu::TextureReshaper::new(
        device,
        &texture_accumulate_view,
        sample_count,
        texture_sample_type,
        sample_count,
        dst_format,
    );

    // Make sure the directory where we will save images to exists.
    std::fs::create_dir_all(&capture_directory(app)).unwrap();

    Model {
        sample_per_frame,
        shutter_angle,
        uniforms: buffer,
        texture_view,
        texture_accumulate_view,
        sampler,
        bind_group_layout,
        vertex_buffer,
        render_pipeline,
        texture,
        texture_accumulate,
        draw,
        renderer,
        texture_capturer,
        texture_reshaper1,
        current_frame: 0,
        recording: false,
        seed: random(),
    }
}

fn draw_ste(time: f64, model: &mut Model) {
    // First, reset the `draw` state.
    let draw = &model.draw;
    draw.reset();

    let [w, _h] = model.texture.size();

    let tau = TAU * time as f32;
    let seed = (model.seed % 1000) as f64 / 1000.;

    // Draw like we normally would in the `view`.
    draw.background()
        .color(srgba(0.08627, 0.08627, 0.08627, 1.));
    let n_points = 30;
    let n_circles = 40;
    let queue = 20;
    let spacen = 2.;
    let amort = 1.;

    for j in -queue..=n_circles + queue {
        let rat = (-queue as f32 * time as f32 + j as f32) / n_circles as f32;
        let ns = nannou::noise::SuperSimplex::new();
        let nsa = 1.
            + ns.get([
                (2. * (spacen * rat + tau)).sin() as f64 / amort,
                (spacen * rat + tau).cos() as f64 / amort,
                seed,
            ]);
        let ws = 5. * nsa as f32 * w as f32 / n_points as f32;
        for i in 0..n_points {
            let rato = i as f32 / n_points as f32;
            let nsc = 1.
                + ns.get([
                    (2. * (spacen * rato + tau)).sin() as f64 / amort,
                    (spacen * rato + tau).cos() as f64 / amort,
                    2. * seed,
                ]);
            let xpos = nsc as f32 * w as f32 / 20. + rat * w as f32 - w as f32 / 2.;
            let nsb = 1.
                + ns.get([
                    (1. * (spacen * rat + tau)).sin() as f64 / amort,
                    (spacen * rat + tau).cos() as f64 / amort,
                    ((1. * (spacen * rato + tau)).sin() as f64
                        + seed
                        + (spacen * rato + tau).cos() as f64)
                        / amort,
                ]);
            let theta = tau + TAU * rato + 0.3 * TAU * (rat + nsb as f32 + 1.);
            let ypos = (theta).sin();
            let zpos = (theta).cos();
            let alpha = map_range(zpos, -1., 1., 0.2, 0.6);
            // let weight = map_range(nsb, 0., 1., 1.5, 3.);
            draw.ellipse()
                .color(srgba(1., 1., 1., alpha))
                .radius(5. + 10. * nsb as f32)
                .x_y_z(xpos, ws * ypos, ws * zpos);
        }
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // Create a `Rect` for our texture to help with drawing.
    let [w, _h] = model.texture.size();

    // Use the frame number to animate, ensuring we get a constant update time.
    // Render our drawing to the texture.
    let window = app.main_window();
    let device = window.device();

    let elapsed_frames = if model.recording {
        model.current_frame
    } else {
        let pos = 2. * (4. * app.mouse.x + (w as f32)) / w as f32;
        (pos * (FPS * N_SEC) as f32) as u32 % (FPS * N_SEC)
    };
    for i in 0..model.sample_per_frame {
        let t = map_range(
            elapsed_frames as f64 + i as f64 * model.shutter_angle / model.sample_per_frame as f64,
            0.,
            (FPS * N_SEC) as f64,
            0.,
            1.,
        );
        draw_ste(t, model);

        let ce_desc = wgpu::CommandEncoderDescriptor {
            label: Some("single pass texture renderer"),
        };
        let mut encoder = device.create_command_encoder(&ce_desc);
        model
            .renderer
            .render_to_texture(device, &mut encoder, &model.draw, &model.texture);
        window.queue().submit(Some(encoder.finish()));

        let bind_group = wgpu::BindGroupBuilder::new()
            .texture_view(&model.texture_view)
            .sampler(&model.sampler)
            .buffer::<Uniforms>(&model.uniforms, 0..1)
            .build(device, &model.bind_group_layout);

        let ce_desc = wgpu::CommandEncoderDescriptor {
            label: Some("accumulate texture renderer"),
        };
        let mut encoder = device.create_command_encoder(&ce_desc);
        {
            let tex_view = &model.texture_accumulate_view;
            let mut render_pass = if i == 0 {
                wgpu::RenderPassBuilder::new()
                    .color_attachment(tex_view, |color| color)
                    .begin(&mut encoder)
            } else {
                wgpu::RenderPassBuilder::new()
                    .color_attachment(tex_view, |color| color.load_op(wgpu::LoadOp::Load))
                    .begin(&mut encoder)
            };
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_pipeline(&model.render_pipeline);
            render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
            let vertex_range = 0..VERTICES.len() as u32;
            let instance_range = 0..1;
            render_pass.draw(vertex_range, instance_range)
        };

        window.queue().submit(Some(encoder.finish()));
    }
    // =================================================================================

    let ce_desc = wgpu::CommandEncoderDescriptor {
        label: Some("save texture renderer"),
    };
    let mut encoder = device.create_command_encoder(&ce_desc);

    let snapshot = model
        .texture_capturer
        .capture(device, &mut encoder, &model.texture_accumulate);

    window.queue().submit(Some(encoder.finish()));
    if model.recording {
        let path = capture_directory(app)
            .join(elapsed_frames.to_string())
            .with_extension("png");
        snapshot
            .read(move |result| {
                let image = result.expect("failed to map texture memory").to_owned();
                image
                    .save(&path)
                    .expect("failed to save texture to png image");
            })
            .unwrap();
        model.current_frame += 1;
        if model.current_frame > FPS * N_SEC {
            model.recording = false;
        }
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(_app: &App, model: &Model, frame: Frame) {
    // Sample the texture and write it to the frame.

    let mut encoder = frame.command_encoder();
    model
        .texture_reshaper1
        .encode_render_pass(frame.texture_view(), &mut *encoder);
    // let bind_group = wgpu::BindGroupBuilder::new()
    //     .texture_view(&model.texture_accumulate_view)
    //     .sampler(&model.sampler)
    //     .build(device, &model.bind_group_layout);

    // let mut render_pass = wgpu::RenderPassBuilder::new()
    //     .color_attachment(frame.texture_view(), |color| color)
    //     .begin(&mut encoder);
    // render_pass.set_bind_group(0, &bind_group, &[]);
    // render_pass.set_pipeline(&model.render_pipeline);
    // render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
    // let vertex_range = 0..VERTICES.len() as u32;
    // let instance_range = 0..1;
    // render_pass.draw(vertex_range, instance_range);
}

// Wait for capture to finish.
fn exit(app: &App, model: Model) {
    println!("Waiting for PNG writing to complete...");
    let window = app.main_window();
    let device = window.device();
    model
        .texture_capturer
        .await_active_snapshots(device)
        .unwrap();
    println!("Done!");
}

// The directory where we'll save the frames.
fn capture_directory(app: &App) -> std::path::PathBuf {
    app.project_path()
        .expect("could not locate project_path")
        .join(app.exe_name().unwrap())
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::S => {
            model.seed = random();
        }
        Key::R => {
            if model.recording {
                model.recording = false;
            } else {
                model.recording = true;
                model.current_frame = 0;
            }
        }
        _ => {}
    }
}
fn uniforms_as_bytes(uniforms: &Uniforms) -> &[u8] {
    unsafe { wgpu::bytes::from(uniforms) }
}
