struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

@group(1)
@binding(0)
var s: sampler;
@group(1)
@binding(1)
var t: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    let tex = textureSampleLevel(t, s, in.uv, 0.0);

    out.color = tex;

    return out;
}