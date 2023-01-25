//! This is a library for easily generating animations in nannou, with an API
//! somewhat similar to Processing.

/// To create an animation, you need to create a struct that implements the
/// trait [`Artwork`]. This trait defines the
/// [`draw_at_time`](Artwork::draw_at_time()) method for drawing the content of
/// your animation at a particular `time`, where time is a `f64` value between 0
/// and 1.
///
/// The easiest way to start an animation is to copy the file `template.rs`
/// located in the `examples/` folder for this crate.
///
/// By default, you animation is not being recorded, but you can start a
/// recording by pressing R.
pub mod artwork;
pub mod projection_mapping;
pub mod utils;

pub use crate::artwork::{Artwork, Options};
use nannou::{
    prelude::*,
    wgpu::{self, TextureViewDimension},
};

/// Frame per second for the animations.
pub const FPS: u32 = 60;
/// Default length of an animation.
pub const N_SEC: u32 = 10;
/// The wgpu default texture format.
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// This structure represents a vertex for the vertex shader.
#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
}

/// The uniform values used in the fragment shader.
#[repr(C)]
#[derive(Clone, Copy)]
struct Uniforms {
    chroma: f32,
    sample_per_frame: i32,
    noise_amout: f32,
}

/// The vertices that make up the rectangle to which the image will be drawn.
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

/// The model base that all animations should use.
pub struct BaseModel {
    sample_per_frame: i32,
    shutter_angle: f64,
    uniforms: wgpu::Buffer,
    texture_view: wgpu::TextureView,
    texture_accumulate_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,

    depth_texture_view: wgpu::TextureView,

    /// The texture that we will draw to.
    pub texture: wgpu::Texture,
    /// Create a `Draw` instance for drawing to our texture.
    pub draw: nannou::Draw,

    /// Holds the number of the frame being run. This allows resetting the
    /// animation when starting a recording.
    current_frame: u32,
    recording: bool,
    pub seed: i32,

    /// Holds extra textures that can be used in the animation.
    pub extra_tex: Option<Vec<wgpu::Texture>>,

    /// The texture that will accumulate frames for the motion blur
    texture_accumulate: wgpu::Texture,

    /// The type used to render the `Draw` vertices to our texture.
    renderer: nannou::draw::Renderer,
    /// The type used to capture the texture.
    texture_capturer: wgpu::TextureCapturer,
    /// The type used to reshape the texture. We draw the animation in 4K but
    /// only display a window of 540 time 540 pixels.
    texture_reshaper: wgpu::TextureReshaper,
}

pub fn make_recorder_app<T: 'static + Artwork>() -> nannou::app::Builder<T> {
    nannou::app(model).update(update).exit(exit)
}

fn model<T: 'static + Artwork>(app: &App) -> T {
    T::new(make_base_model::<T>(app, T::get_options()))
}

pub fn make_base_model<T: 'static + Artwork>(app: &App, options: Option<Options>) -> BaseModel {
    // Lets write to a 4K UHD texture.
    let texture_size = [2160, 2160];

    // Create the window.
    let [win_w, win_h] = [texture_size[0] / 4, texture_size[1] / 4];
    let w_id = app
        .new_window()
        .size(win_w, win_h)
        .title("nannou")
        .view::<T>(view)
        .key_pressed::<T>(key_pressed)
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
        .usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT)
        .sample_count(sample_count)
        .format(wgpu::TextureFormat::Rgba16Float)
        .build(device);
    let texture_view = texture.view().build();
    let texture_accumulate_view = texture_accumulate.view().build();

    let depth_texture = create_depth_texture(device, texture_size, DEPTH_FORMAT, sample_count);
    let depth_texture_view = depth_texture.view().build();

    // Create our `Draw` instance and a renderer for it.
    let draw = nannou::Draw::new();
    let descriptor = texture.descriptor();
    let renderer =
        nannou::draw::RendererBuilder::new().build_from_texture_descriptor(device, descriptor);

    // Build shader modules. The vertex shader only displays a square the size
    // of the window.
    let vs_desc = wgpu::include_wgsl!("shaders/vs.wgsl");
    let fs_desc = wgpu::include_wgsl!("shaders/fs.wgsl");
    let vs_mod = device.create_shader_module(&vs_desc);
    let fs_mod = device.create_shader_module(&fs_desc);

    // Build the sampler
    let sampler_desc = wgpu::SamplerBuilder::new()
        .label(Some("The sampler"))
        .into_descriptor();
    let sampler_filtering = wgpu::sampler_filtering(&sampler_desc);
    let sampler = device.create_sampler(&sampler_desc);

    let bind_group_layout =
        create_bind_group_layout(device, texture_view.sample_type(), sampler_filtering);

    let options = options.unwrap_or_default();
    let uniforms = Uniforms {
        chroma: options.chroma,
        sample_per_frame: options.sample_per_frame,
        noise_amout: options.noise_amount,
    };

    // Uniforms to be passed to the shaders
    let uniforms_bytes = uniforms_as_bytes(&uniforms);
    let usage = wgpu::BufferUsages::UNIFORM;
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: uniforms_bytes,
        usage,
    });

    // Render pipeline layout construction
    let desc = wgpu::PipelineLayoutDescriptor {
        label: Some("The pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    };
    let pipeline_layout = device.create_pipeline_layout(&desc);
    // Render pipeline construction
    let render_pipeline = create_render_pipeline(
        device,
        &pipeline_layout,
        &vs_mod,
        &fs_mod,
        texture_accumulate.format(),
        DEPTH_FORMAT,
        sample_count,
    );

    // Vertex buffer
    let vertices_bytes = vertices_as_bytes(&VERTICES[..]);
    let usage = wgpu::BufferUsages::VERTEX;
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: vertices_bytes,
        usage,
    });

    // Create the texture capturer.
    let texture_capturer = wgpu::TextureCapturer::default();

    // Create the texture reshaper for GUI display
    let texture_sample_type = texture_accumulate.sample_type();
    let dst_format = texture_accumulate.format();
    let texture_reshaper = wgpu::TextureReshaper::new(
        device,
        &texture_accumulate_view,
        sample_count,
        texture_sample_type,
        sample_count,
        dst_format,
    );
    let extra_texture = if let Some(name) = options.extra_tex {
        let assets = app.assets_path().unwrap();
        Some(
            name.iter()
                .map(|n| wgpu::Texture::from_path(app, assets.join(n)).unwrap())
                .collect(),
        )
    } else {
        None
    };

    // Make sure the directory where we will save images to exists.
    std::fs::create_dir_all(&capture_directory(app)).unwrap();
    BaseModel {
        sample_per_frame: options.sample_per_frame,
        shutter_angle: options.shutter_angle,
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
        texture_reshaper,
        current_frame: 0,
        recording: false,
        seed: random(),
        depth_texture_view,
        extra_tex: extra_texture,
    }
}

fn update<T: Artwork>(app: &App, model: &mut T, _update: Update) {
    // Create a `Rect` for our texture to help with drawing.
    let [w, _h] = model.get_model().texture.size();
    let n_sec = model.n_sec().unwrap_or(N_SEC);
    // Use the frame number to animate, ensuring we get a constant update time.
    // Render our drawing to the texture.
    let window = app.main_window();
    let device = window.device();

    let elapsed_frames = if model.get_model().recording {
        model.get_model().current_frame
    } else {
        let pos = 2. * (4. * app.mouse.x + (w as f32)) / w as f32;
        (pos * (FPS * n_sec) as f32) as u32 % (FPS * n_sec)
    };
    let n_sample_per_frame = model.get_model().sample_per_frame;
    for i in 0..n_sample_per_frame {
        let t: f64 = map_range(
            elapsed_frames as f64
                + i as f64 * model.get_model().shutter_angle
                    / model.get_model().sample_per_frame as f64,
            0.,
            (FPS * n_sec) as f64,
            0.,
            1.,
        );

        render_pass(device, &window, t, model, i == 0);
    }

    let ce_desc = wgpu::CommandEncoderDescriptor {
        label: Some("save texture renderer"),
    };
    let mut encoder = device.create_command_encoder(&ce_desc);

    let snapshot = model.get_model().texture_capturer.capture(
        device,
        &mut encoder,
        &model.get_model().texture_accumulate,
    );
    window.queue().submit(Some(encoder.finish()));

    if model.get_model().recording {
        record_frame(app, elapsed_frames, model, snapshot)
    }
}

fn record_frame<T: Artwork>(
    app: &App,
    elapsed_frames: u32,
    model: &mut T,
    snapshot: wgpu::TextueSnapshot,
) {
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
    let n_sec = model.n_sec().unwrap_or(N_SEC);
    let mut base_model = model.get_mut_model();
    base_model.current_frame += 1;
    if base_model.current_frame > FPS * n_sec {
        base_model.recording = false;
    }
}

fn render_pass<T: Artwork>(
    device: &wgpu::Device,
    window: &Window,
    t: f64,
    model: &mut T,
    first: bool,
) {
    model.draw_at_time(t);
    let base_model = model.get_mut_model();
    let ce_desc = wgpu::CommandEncoderDescriptor {
        label: Some("single pass texture renderer"),
    };
    let mut encoder = device.create_command_encoder(&ce_desc);
    base_model.renderer.render_to_texture(
        device,
        &mut encoder,
        &base_model.draw,
        &base_model.texture,
    );
    window.queue().submit(Some(encoder.finish()));

    let bind_group = wgpu::BindGroupBuilder::new()
        .texture_view(&base_model.texture_view)
        .sampler(&base_model.sampler)
        .buffer::<Uniforms>(&base_model.uniforms, 0..1)
        .build(device, &base_model.bind_group_layout);

    let ce_desc = wgpu::CommandEncoderDescriptor {
        label: Some("accumulate texture renderer"),
    };
    let mut encoder = device.create_command_encoder(&ce_desc);
    {
        let tex_view = &base_model.texture_accumulate_view;
        let mut render_pass = if first {
            wgpu::RenderPassBuilder::new()
                .color_attachment(tex_view, |color| color)
                .depth_stencil_attachment(&base_model.depth_texture_view, |depth| depth)
                .begin(&mut encoder)
        } else {
            wgpu::RenderPassBuilder::new()
                .color_attachment(tex_view, |color| color.load_op(wgpu::LoadOp::Load))
                .depth_stencil_attachment(&base_model.depth_texture_view, |depth| depth)
                .begin(&mut encoder)
        };
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_pipeline(&base_model.render_pipeline);
        render_pass.set_vertex_buffer(0, base_model.vertex_buffer.slice(..));
        let vertex_range = 0..VERTICES.len() as u32;
        let instance_range = 0..1;
        render_pass.draw(vertex_range, instance_range)
    };

    window.queue().submit(Some(encoder.finish()));
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    vs_mod: &wgpu::ShaderModule,
    fs_mod: &wgpu::ShaderModule,
    dst_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::RenderPipeline {
    nannou::wgpu::RenderPipelineBuilder::from_layout(layout, vs_mod)
        .fragment_shader(fs_mod)
        .color_format(dst_format)
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
        .depth_format(depth_format)
        .sample_count(sample_count)
        .primitive_topology(wgpu::PrimitiveTopology::TriangleStrip)
        .build(device)
}

fn create_bind_group_layout(
    device: &wgpu::Device,
    sample_type: wgpu::TextureSampleType,
    sampler_filtering: bool,
) -> wgpu::BindGroupLayout {
    wgpu::BindGroupLayoutBuilder::new()
        .texture(
            wgpu::ShaderStages::FRAGMENT,
            true,
            TextureViewDimension::D2,
            sample_type,
        )
        .sampler(wgpu::ShaderStages::FRAGMENT, sampler_filtering)
        .uniform_buffer(wgpu::ShaderStages::FRAGMENT, false)
        .build(device)
}

// Draw the state of your `Model` into the given `Frame` here.
fn view<T: Artwork>(_app: &App, model: &T, frame: Frame) {
    // Sample the texture and write it to the frame.
    let mut encoder = frame.command_encoder();
    model
        .get_model()
        .texture_reshaper
        .encode_render_pass(frame.texture_view(), &mut *encoder);
}

// Wait for capture to finish.
fn exit<T: Artwork>(app: &App, model: T) {
    println!("Waiting for PNG writing to complete...");
    let window = app.main_window();
    let device = window.device();
    model
        .get_model()
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

fn create_depth_texture(
    device: &wgpu::Device,
    size: [u32; 2],
    depth_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::Texture {
    wgpu::TextureBuilder::new()
        .size(size)
        .format(depth_format)
        .usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
        .sample_count(sample_count)
        .build(device)
}

fn key_pressed<T: Artwork>(app: &App, model: &mut T, key: Key) {
    let base_model = model.get_mut_model();
    match key {
        Key::S => {
            base_model.seed = random();
        }
        Key::R => {
            if base_model.recording {
                base_model.recording = false;
            } else {
                base_model.recording = true;
                base_model.current_frame = 0;
            }
        }
        _ => {}
    }
    // This is not inside the match to allow the model T to override or extend
    // what happens when one of the pre-configured key is pressed.
    model.key_pressed(app, key)
}

fn uniforms_as_bytes<T>(uniforms: &T) -> &[u8]
where
    T: std::marker::Copy,
{
    unsafe { wgpu::bytes::from(uniforms) }
}

fn vertices_as_bytes(data: &[Vertex]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}
