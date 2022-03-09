use nannou::{noise::NoiseFn, prelude::*};

const FPS: u32 = 60;
const N_SEC: u32 = 10;

fn main() {
    nannou::app(model).update(update).exit(exit).run();
}

struct Model {
    // The texture that we will draw to.
    texture: wgpu::Texture,
    // Create a `Draw` instance for drawing to our texture.
    draw: nannou::Draw,
    // The type used to render the `Draw` vertices to our texture.
    renderer: nannou::draw::Renderer,
    // The type used to capture the texture.
    texture_capturer: wgpu::TextureCapturer,
    // The type used to resize our texture to the window texture.
    texture_reshaper: wgpu::TextureReshaper,
    current_frame: u32,
    recording: bool,
    seed: i32,
}

fn model(app: &App) -> Model {
    // Lets write to a 4K UHD texture.
    let texture_size = [2160, 2160];

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

    // Create our `Draw` instance and a renderer for it.
    let draw = nannou::Draw::new();
    let descriptor = texture.descriptor();
    let renderer =
        nannou::draw::RendererBuilder::new().build_from_texture_descriptor(device, descriptor);

    // Create the texture capturer.
    let texture_capturer = wgpu::TextureCapturer::default();

    // Create the texture reshaper.
    let texture_view = texture.view().build();
    let texture_sample_type = texture.sample_type();
    let dst_format = Frame::TEXTURE_FORMAT;
    let texture_reshaper = wgpu::TextureReshaper::new(
        device,
        &texture_view,
        sample_count,
        texture_sample_type,
        sample_count,
        dst_format,
    );

    // Make sure the directory where we will save images to exists.
    std::fs::create_dir_all(&capture_directory(app)).unwrap();

    Model {
        texture,
        draw,
        renderer,
        texture_capturer,
        texture_reshaper,
        current_frame: 0,
        recording: false,
        seed: random(),
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // First, reset the `draw` state.
    let draw = &model.draw;
    draw.reset();

    // Create a `Rect` for our texture to help with drawing.
    let [w, _h] = model.texture.size();

    // Use the frame number to animate, ensuring we get a constant update time.
    let elapsed_frames = if model.recording {
        model.current_frame
    } else {
        let pos = 2. * (4. * app.mouse.x + (w as f32)) / w as f32;
        (pos * (FPS * N_SEC) as f32) as u32 % (FPS * N_SEC)
    };
    let t = elapsed_frames as f64 / (FPS * N_SEC) as f64;
    let tau = TAU * t as f32;
    let seed = (model.seed % 1000) as f64 / 1000.;

    // Draw like we normally would in the `view`.
    draw.background().color(BLACK);
    let n_points = 30;
    let n_circles = 40;
    let queue = 20;
    let spacen = 2.;
    let amort = 1.;


    for j in -queue..=n_circles + queue {
        let rat = (- queue as f32 * t as f32 + j as f32) / n_circles as f32;
        let ns = nannou::noise::SuperSimplex::new();
        let nsa = 1.
            + ns.get([
                (2. * (spacen * rat + tau)).sin() as f64 / amort,
                (spacen * rat + tau).cos() as f64 / amort,
                seed,
            ]);
        let xpos = rat * w as f32 - w as f32 / 2.;
        let ws = 5. * nsa as f32 * w as f32 / n_points as f32;
        for i in 0..n_points {
            let rato = i as f32 / n_points as f32;
            let nsb = 1.
                + ns.get([
                    (1. * (spacen * rat + tau)).sin() as f64 / amort,
                    (spacen * rat + tau).cos() as f64 / amort,
                    ((1. * (spacen * rato + tau)).sin() as f64 + seed + (spacen * rato + tau).cos() as f64) / amort,
                ]);
            let theta = tau + TAU * rato + 0.3 * TAU * (rat + nsb as f32 + 1.);
            let ypos = (theta).sin();
            let zpos = (theta).cos();
            let alpha = map_range(zpos, -1., 1., 0.2, 0.6);
            let weight = map_range(nsb, 0., 1., 1.5, 3.);
            draw.ellipse()
                .no_fill()
                .stroke_color(srgba(1., 1., 1., alpha))
                .stroke_weight(weight)
                .radius(10. + nsb as f32 + i as f32 * (tau + i as f32).sin())
                .x_y_z(xpos, ws * ypos, ws * zpos);
        }
    }

    // Render our drawing to the texture.
    let window = app.main_window();
    let device = window.device();
    let ce_desc = wgpu::CommandEncoderDescriptor {
        label: Some("texture renderer"),
    };
    let mut encoder = device.create_command_encoder(&ce_desc);
    model
        .renderer
        .render_to_texture(device, &mut encoder, draw, &model.texture);
    let snapshot = model
        .texture_capturer
        .capture(device, &mut encoder, &model.texture);
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
        .texture_reshaper
        .encode_render_pass(frame.texture_view(), &mut *encoder);
}

// Wait for capture to finish.
fn exit(app: &App, model: Model) {
    println!("Waiting for PNG writing to complete...");
    let window = app.main_window();
    let device = window.device();
    model
        .texture_capturer
        .await_active_snapshots(&device)
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
