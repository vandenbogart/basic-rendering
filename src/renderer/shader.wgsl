// Vertex shader
struct Globals {
    view_proj: mat4x4<f32>,
    ambient_color: vec3<f32>,
    ambient_strength: f32,
}

struct Locals {
    position: vec4<f32>,
    diffuse_light_position: vec4<f32>,
    diffuse_light_color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

@group(1) @binding(0)
var<uniform> locals: Locals;


struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) color: vec3<f32>,
}

struct InstanceInput {
    @location(4) v1: vec4<f32>,
    @location(5) v2: vec4<f32>,
    @location(6) v3: vec4<f32>,
    @location(7) v4: vec4<f32>,
    @location(8) n1: vec3<f32>,
    @location(9) n2: vec3<f32>,
    @location(10) n3: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
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
        instance.n1,
        instance.n2,
        instance.n3,
    );
    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.world_normal = normal_matrix *  model.normal;
    out.tex_coords = model.tex_coords;
    out.clip_position = globals.view_proj * world_position;
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color = vec4<f32>(0.7, 0.7, 0.7, 1.0);
    let ambient_color = globals.ambient_color * globals.ambient_strength;


    let light_dir = normalize(locals.diffuse_light_position.xyz - in.world_position);
    let diffuse_intensity = max(dot(light_dir, in.world_normal), 0.0);
    let diffuse_color = locals.diffuse_light_color.xyz * diffuse_intensity;

    let result = (diffuse_intensity + ambient_color) * object_color.xyz;
    return vec4<f32>(result, object_color.a);
}