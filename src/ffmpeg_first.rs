use ffmpeg_next::encoder::video::Video;
use ffmpeg_next::encoder::Encoder;
use ffmpeg_next::{
    codec, decoder, encoder, format, frame, log, media, picture, Dictionary, Packet, Rational,
};
use nannou::prelude::*;
use std::fs;
use std::io::ErrorKind;

const ROWS: u32 = 22;
const COLS: u32 = 12;
const SIZE: u32 = 30;
const LINE_WIDTH: f32 = 0.06;
const MARGIN: u32 = 35;
const WIDTH: u32 = COLS * SIZE + 2 * MARGIN;
const HEIGHT: u32 = ROWS * SIZE + 2 * MARGIN;

const DEFAULT_X264_OPTS: &str = "preset=medium";

fn main() {
    ffmpeg_next::init().unwrap();
    nannou::app(model)
        .update(update)
        .loop_mode(LoopMode::refresh_sync())
        .run()
}

struct Stone {
    x: f32,
    y: f32,
    x_offset: f32,
    y_offset: f32,
    rotation: f32,
    x_velocity: f32,
    y_velocity: f32,
    rot_velocity: f32,
    cycles: u32,
}

impl Stone {
    fn new(x: f32, y: f32) -> Self {
        let x_offset = 0.0;
        let y_offset = 0.0;
        let rotation = 0.0;
        let x_velocity = 0.0;
        let y_velocity = 0.0;
        let rot_velocity = 0.0;
        let cycles = 0;
        Stone {
            x,
            y,
            x_offset,
            y_offset,
            rotation,
            x_velocity,
            y_velocity,
            rot_velocity,
            cycles,
        }
    }
}

struct Model {
    main_window: WindowId,
    frames_dir: String,
    cur_frame: u32,
    recording: bool,
    disp_adj: f32,
    rot_adj: f32,
    motion: f32,
    gravel: Vec<Stone>,
    encoder: Video,
    texture: wgpu::Texture,
    draw: nannou::Draw,
    renderer: nannou::draw::Renderer,
    // The type used to capture the texture.
    texture_capturer: wgpu::TextureCapturer,
    // The type used to resize our texture to the window texture.
    texture_reshaper: wgpu::TextureReshaper,
}

fn model(app: &App) -> Model {
    let main_window = app
        .new_window()
        .title(app.exe_name().unwrap())
        .size(WIDTH, HEIGHT)
        .view(view)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    let frames_dir = app.exe_name().unwrap() + "_frames";
    let recording = false;
    let cur_frame = 0;

    let disp_adj = 1.0;
    let rot_adj = 1.0;
    let motion = 0.5;

    let mut gravel = Vec::new();
    for y in 0..ROWS {
        for x in 0..COLS {
            let stone = Stone::new(x as f32, y as f32);
            gravel.push(stone);
        }
    }
    let mut octx = format::output(&"test.mp4").unwrap();
    let mut ost = octx.add_stream(encoder::find(codec::Id::H264)).unwrap();
    let mut encoder = codec::context::Context::from_parameters(ost.parameters())
        .unwrap()
        .encoder()
        .video()
        .unwrap();
    encoder.set_height(HEIGHT);
    encoder.set_width(WIDTH);
    encoder.set_frame_rate(Some(60.));
    encoder.set_time_base(1. / 60.);
    encoder
        .open_with(parse_opts(DEFAULT_X264_OPTS.to_string()).unwrap())
        .expect("error opening libx264 encoder with supplied settings");
    encoder = codec::context::Context::from_parameters(ost.parameters())
        .unwrap()
        .encoder()
        .video()
        .unwrap();
    ost.set_parameters(&encoder);

    let window = app.window(main_window).unwrap();
    let device = window.device();
    let sample_count = window.msaa_samples();
    let texture = wgpu::TextureBuilder::new()
        .size([WIDTH, HEIGHT])
        // Our texture will be used as the RENDER_ATTACHMENT for our `Draw` render pass.
        // It will also be SAMPLED by the `TextureCapturer` and `TextureResizer`.
        .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
        // Use nannou's default multisampling sample count.
        .sample_count(sample_count)
        // Use a spacious 16-bit linear sRGBA format suitable for high quality drawing.
        .format(wgpu::TextureFormat::Rgba16Float)
        // Build it!
        .build(device);
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

    Model {
        main_window,
        frames_dir,
        recording,
        cur_frame,
        disp_adj,
        rot_adj,
        motion,
        gravel,
        encoder,
        texture,
        draw,
        renderer,
        texture_capturer,
        texture_reshaper,
    }
}

fn parse_opts<'a>(s: String) -> Option<Dictionary<'a>> {
    let mut dict = Dictionary::new();
    for keyval in s.split_terminator(',') {
        let tokens: Vec<&str> = keyval.split('=').collect();
        match tokens[..] {
            [key, val] => dict.set(key, val),
            _ => return None,
        }
    }
    Some(dict)
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let draw = app.draw();
    let gdraw = draw
        .scale(SIZE as f32)
        .scale_y(-1.0)
        .x_y(COLS as f32 / -2.0 + 0.5, ROWS as f32 / -2.0 + 0.5);

    draw.background().color(SNOW);

    for stone in &model.gravel {
        let cdraw = gdraw.x_y(stone.x, stone.y);
        cdraw
            .rect()
            .no_fill()
            .stroke(BLACK)
            .stroke_weight(LINE_WIDTH)
            .w_h(1.0, 1.0)
            .x_y(stone.x_offset, stone.y_offset)
            .rotate(stone.rotation);
    }

    draw.to_frame(app, &frame).unwrap();

    for stone in &mut model.gravel {
        if stone.cycles == 0 {
            if random_f32() > model.motion {
                stone.x_velocity = 0.0;
                stone.y_velocity = 0.0;
                stone.rot_velocity = 0.0;
                stone.cycles = random_range(50, 300);
            } else {
                let factor = stone.y / ROWS as f32;
                let disp_factor = factor * model.disp_adj;
                let rot_factor = factor * model.rot_adj;
                let new_x = disp_factor * random_range(-0.5, 0.5);
                let new_y = disp_factor * random_range(-0.5, 0.5);
                let new_rot = rot_factor * random_range(-PI / 4.0, PI / 4.0);
                let new_cycles = random_range(50, 300);
                stone.x_velocity = (new_x - stone.x_offset) / new_cycles as f32;
                stone.y_velocity = (new_y - stone.y_offset) / new_cycles as f32;
                stone.rot_velocity = (new_rot - stone.rotation) / new_cycles as f32;
                stone.cycles = new_cycles;
            }
        } else {
            stone.x_offset += stone.x_velocity;
            stone.y_offset += stone.y_velocity;
            stone.rotation += stone.rot_velocity;
            stone.cycles -= 1;
        }
    }

    if model.recording && app.elapsed_frames() % 2 == 0 {
        model.cur_frame += 1;
        if model.cur_frame > 9999 {
            model.recording = false;
        } else {
            let window = app.main_window();
            let device = window.device();
            let ce_desc = wgpu::CommandEncoderDescriptor {
                label: Some("texture renderer"),
            };
            let mut encoder = device.create_command_encoder(&ce_desc);
            model.renderer
                .render_to_texture(device, &mut encoder, draw, &model.texture);
            let filename = format!("{}/schotter{:>04}.png", model.frames_dir, model.cur_frame);
            match app.window(model.main_window) {
                Some(window) => {
                    window.capture_frame(filename);
                }
                None => {}
            }
        }
    }
}

fn view(_app: &App, model: &Model, frame: Frame) {
    // Sample the texture and write it to the frame.
    let mut encoder = frame.command_encoder();
    model
        .texture_reshaper
        .encode_render_pass(frame.texture_view(), &mut *encoder);
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::S => match app.window(model.main_window) {
            Some(window) => {
                window.capture_frame(app.exe_name().unwrap() + ".png");
            }
            None => {}
        },
        Key::R => {
            if model.recording {
                model.recording = false;
            } else {
                fs::create_dir(&model.frames_dir).unwrap_or_else(|error| {
                    if error.kind() != ErrorKind::AlreadyExists {
                        panic! {"Problem creating directory {:?}", model.frames_dir};
                    }
                });
                model.recording = true;
                model.cur_frame = 0;
            }
        }
        Key::Up => {
            model.disp_adj += 0.1;
        }
        Key::Down => {
            if model.disp_adj > 0.0 {
                model.disp_adj -= 0.1;
            }
        }
        Key::Right => {
            model.rot_adj += 0.1;
        }
        Key::Left => {
            if model.rot_adj > 0.0 {
                model.rot_adj -= 0.1;
            }
        }
        _other_key => {}
    }
}
use nannou::prelude::*;

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
}

fn model(app: &App) -> Model {
    // Lets write to a 4K UHD texture.
    let texture_size = [3_840, 2_160];

    // Create the window.
    let [win_w, win_h] = [texture_size[0] / 4, texture_size[1] / 4];
    let w_id = app
        .new_window()
        .size(win_w, win_h)
        .title("nannou")
        .view(view)
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
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // First, reset the `draw` state.
    let draw = &model.draw;
    draw.reset();

    // Create a `Rect` for our texture to help with drawing.
    let [w, h] = model.texture.size();
    let r = geom::Rect::from_w_h(w as f32, h as f32);

    // Use the frame number to animate, ensuring we get a constant update time.
    let elapsed_frames = app.main_window().elapsed_frames();
    let t = elapsed_frames as f32 / 60.0;

    // Draw like we normally would in the `view`.
    draw.background().color(BLACK);
    let n_points = 10;
    let weight = 8.0;
    let hz = 6.0;
    let vertices = (0..n_points)
        .map(|i| {
            let x = map_range(i, 0, n_points - 1, r.left(), r.right());
            let fract = i as f32 / n_points as f32;
            let amp = (t + fract * hz * TAU).sin();
            let y = map_range(amp, -1.0, 1.0, r.bottom() * 0.75, r.top() * 0.75);
            pt2(x, y)
        })
        .enumerate()
        .map(|(i, p)| {
            let fract = i as f32 / n_points as f32;
            let r = (t + fract) % 1.0;
            let g = (t + 1.0 - fract) % 1.0;
            let b = (t + 0.5 + fract) % 1.0;
            let rgba = srgba(r, g, b, 1.0);
            (p, rgba)
        });
    draw.polyline()
        .weight(weight)
        .join_round()
        .points_colored(vertices);

    // Draw frame number and size in bottom left.
    let string = format!("Frame {} - {:?}", elapsed_frames, [w, h]);
    let text = text(&string)
        .font_size(48)
        .left_justify()
        .align_bottom()
        .build(r.pad(r.h() * 0.05));
    draw.path().fill().color(WHITE).events(text.path_events());

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

    // Take a snapshot of the texture. The capturer will do the following:
    //
    // 1. Resolve the texture to a non-multisampled texture if necessary.
    // 2. Convert the format to non-linear 8-bit sRGBA ready for image storage.
    // 3. Copy the result to a buffer ready to be mapped for reading.
    let snapshot = model
        .texture_capturer
        .capture(device, &mut encoder, &model.texture);

    // Submit the commands for our drawing and texture capture to the GPU.
    window.queue().submit(Some(encoder.finish()));

    // Submit a function for writing our snapshot to a PNG.
    //
    // NOTE: It is essential that the commands for capturing the snapshot are `submit`ted before we
    // attempt to read the snapshot - otherwise we will read a blank texture!
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
