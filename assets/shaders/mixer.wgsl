@group(0) @binding(0) var pattern_textures: texture_2d_array<f32>;
@group(0) @binding(1) var pattern_sampler: sampler;

@group(0) @binding(2) var output: texture_storage_2d<rgba8unorm, write>;


@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let coord = vec2<i32>(gid.xy);
    let uv = vec2<f32>(coord) / vec2<f32>(512.0 - 1.0);
    var pattern = vec4<f32>(1.0);
    for (var i = 0u; i < 1; i++) {
        pattern *= textureSample(pattern_textures, pattern_sampler, uv, i);
    }
    textureStore(output, coord, pattern);
}