// Vertex shader
struct Globals {
    view_proj: mat4x4<f32>,
    ambient_color: vec4<f32>,
    ambient_strength: f32,
}

struct Locals {
    diffuse_light_position: vec4<f32>,
    diffuse_light_color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

@group(1) @binding(0)
var<uniform> locals: Locals;


struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) color: vec3<f32>
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    var world_position: vec4<f32> = vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.world_normal = model.normal;
    out.color = model.color;
    out.clip_position = globals.view_proj * world_position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ambient_color = globals.ambient_color * globals.ambient_strength;

    let light_dir = normalize(locals.diffuse_light_position.xyz - in.world_position);
    let diffuse_intensity = max(dot(light_dir, in.world_normal), 0.0) * locals.diffuse_light_color.a;
    let diffuse_color = locals.diffuse_light_color.xyz * diffuse_intensity;

    let result = (diffuse_intensity + ambient_color.rgb) * in.color;
    return vec4<f32>(result, 1.0);
}
