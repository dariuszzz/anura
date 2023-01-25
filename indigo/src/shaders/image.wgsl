@group(0)
@binding(0)
var<uniform> camera: mat4x4<f32>;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = camera * vec4<f32>(input.pos, 1.0);
    out.color = input.color;
    out.uv = input.uv;

    return out;
}

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