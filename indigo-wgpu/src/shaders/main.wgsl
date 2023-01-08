// struct CameraUniform {
    // view_proj: mat4x4<f32>;
// };

@group(0)
@binding(0)
var<uniform> camera: mat4x4<f32>;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) col: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tint_color: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = camera * vec4<f32>(model.pos, 1.0);
    out.tint_color = model.col;

    return out;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    out.color = in.tint_color;

    return out;
}