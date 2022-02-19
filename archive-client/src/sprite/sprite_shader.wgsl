struct Global {
    mvp: mat4x4<f32>;
};

struct GpuSprite {
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] size: vec2<f32>;
    [[location(2)]] rotation: f32;
    [[location(3)]] color: u32;
};


[[group(0), binding(0)]]
var<uniform> global: Global;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vi: u32, instance: GpuSprite) -> VertexOutput {
    let tc = vec2<f32>(f32(vi & 1u), 0.5 * f32(vi & 2u));
    let offset = instance.size * (tc - vec2<f32>(0.5, 0.5));
    let trig = vec2<f32>(cos(instance.rotation), sin(instance.rotation));
    let rotate = mat2x2<f32>(trig.x, -trig.y, trig.y, trig.x);
    let model_pos = instance.position + rotate * offset;
    let pos = global.mvp * vec4<f32>(model_pos, 0.0, 1.0);
    let color = vec4<f32>((vec4<u32>(instance.color) >> vec4<u32>(0u, 8u, 16u, 24u)) & vec4<u32>(255u)) / 255.0;
    return VertexOutput(pos, tc, color);
}


[[group(1), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(1), binding(1)]]
var s_diffuse: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color * textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
