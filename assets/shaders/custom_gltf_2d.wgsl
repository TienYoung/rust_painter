#import bevy_sprite::{
    mesh2d_view_bindings::globals,
    mesh2d_functions::{get_world_from_local, mesh2d_position_local_to_clip},
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    // @location(1) color: vec4<f32>,
    @location(1) uv_0: vec2<f32>,
    // @location(2) barycentric: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // @location(0) color: vec4<f32>,
    @location(0) uv: vec2<f32>,
    // @location(2) barycentric: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let world_from_local = get_world_from_local(vertex.instance_index);
    out.clip_position = mesh2d_position_local_to_clip(world_from_local, vec4<f32>(vertex.position, 1.0));
    // out.color = vertex.color;
    out.uv = vertex.uv_0;
    // out.barycentric = vertex.barycentric;
    return out;
}

@group(2) @binding(0) var texture_0: texture_2d<f32>;
@group(2) @binding(1) var sampler_0: sampler;

struct FragmentInput {
    // @location(0) color: vec4<f32>,
    @location(0) uv: vec2<f32>,
    // @location(2) barycentric: vec3<f32>,
};

@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
    // let d = min(input.barycentric.x, min(input.barycentric.y, input.barycentric.z));
    // let t = 0.05 * (0.85 + sin(5.0 * globals.time));
    // return mix(vec4(1.0,1.0,1.0,1.0), input.color, smoothstep(t, t+0.01, d));

    return textureSample(texture_0, sampler_0, input.uv.xy);
}