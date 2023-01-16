struct FragmentOutput {
    [[location(0)]] out_color: vec4<f32>;
};

[[block]]
struct Data {
    chroma: f32;
    sample_per_frame: i32;
    noise_amount: f32;
};


[[group(0), binding(0)]]
var tex: texture_multisampled_2d<f32>;
[[group(0), binding(1)]]
var tex_sampler: sampler;
[[group(0), binding(2)]]
var<uniform> uniforms: Data;

[[stage(fragment)]]
fn main(
    [[location(0)]] tex_coords: vec2<f32>,
) -> FragmentOutput {

    let tex_size: vec2<i32> = textureDimensions(tex);
    let tex_x: f32 = f32(tex_size.x) * tex_coords.x;
    let tex_y: f32 = f32(tex_size.y) * tex_coords.y;;
    let itex_coords: vec2<f32> = vec2<f32>(tex_x, tex_y);


    let pr: vec2<i32> = vec2<i32>(i32(itex_coords[0]), i32(itex_coords[1]));

    let itexg: vec2<f32> = (itex_coords - 0.5) * (1. + 0.008 * uniforms.chroma) + 0.5;
    let pg: vec2<i32> = vec2<i32>(i32(itexg[0]), i32(itexg[1]));

    let itexb: vec2<f32> = (itex_coords - 0.5) * (1. + 0.016 * uniforms.chroma) + 0.5;
    let pb: vec2<i32> = vec2<i32>(i32(itexb[0]), i32(itexb[1]));

    // Manually unroll the resolve. The less conditions the better!
    var color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    color[0] = color[0] + textureLoad(tex, pr, 0)[0];
    color[0] = color[0] + textureLoad(tex, pr, 1)[0];
    color[0] = color[0] + textureLoad(tex, pr, 2)[0];
    color[0] = color[0] + textureLoad(tex, pr, 3)[0];

    color[1] = color[1] + textureLoad(tex, pg, 0)[1];
    color[1] = color[1] + textureLoad(tex, pg, 1)[1];
    color[1] = color[1] + textureLoad(tex, pg, 2)[1];
    color[1] = color[1] + textureLoad(tex, pg, 3)[1];

    color[2] = color[2] + textureLoad(tex, pb, 0)[2];
    color[2] = color[2] + textureLoad(tex, pb, 1)[2];
    color[2] = color[2] + textureLoad(tex, pb, 2)[2];
    color[2] = color[2] + textureLoad(tex, pb, 3)[2];

    color[3] = color[3] + textureLoad(tex, pr, 0)[3];
    color[3] = color[3] + textureLoad(tex, pr, 1)[3];
    color[3] = color[3] + textureLoad(tex, pr, 2)[3];
    color[3] = color[3] + textureLoad(tex, pr, 3)[3];

    color = color * 0.25 / f32(uniforms.sample_per_frame);

    // Grain
    // from https://www.shadertoy.com/view/3sGGRz
    let mdf: f32 = uniforms.noise_amount / f32(uniforms.sample_per_frame); // increase for noise amount
    let noise: f32 = fract(sin(dot(tex_coords, vec2<f32>(12.9898,78.233) * 2.0)) * 43758.5453);

    color = color - noise * mdf;

    return FragmentOutput(color);
}
