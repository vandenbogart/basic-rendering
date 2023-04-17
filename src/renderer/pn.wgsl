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

struct InstanceInput {
    @location(0) v1: vec4<f32>,
    @location(1) v2: vec4<f32>,
    @location(2) v3: vec4<f32>,
    @location(3) v4: vec4<f32>,
}

struct VertexInput {
    @location(4) position: vec3<f32>,
    @location(5) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    let model_matrix = mat4x4<f32>(
        instance.v1,
        instance.v2,
        instance.v3,
        instance.v4,
    );
    let normal_matrix = mat3x3<f32>(
        instance.v1.xyz,
        instance.v2.xyz,
        instance.v3.xyz,
    );
    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    var world_normal: vec3<f32> = normal_matrix * model.normal;
    out.world_position = world_position.xyz;
    out.world_normal = world_normal;
    out.clip_position = globals.view_proj * world_position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ambient_color = globals.ambient_color * globals.ambient_strength;

    let light_dir = normalize(locals.diffuse_light_position.xyz - in.world_position);
    let diffuse_intensity = max(dot(light_dir, in.world_normal), 0.0) * locals.diffuse_light_color.a;
    let diffuse_color = locals.diffuse_light_color.xyz * diffuse_intensity;

    let result = (diffuse_intensity + ambient_color.rgb) * vec3<f32>(0.5, 0.5, 0.5);
    return vec4<f32>(result, 1.0);
}
