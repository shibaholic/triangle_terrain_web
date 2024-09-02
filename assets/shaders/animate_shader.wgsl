#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::forward_io::Vertex
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
#import bevy_shader_utils::simplex_noise_3d::simplex_noise_3d

fn linear_conversion(
    value: f32, 
    old_min: f32,
    old_max: f32, 
    new_min: f32,
    new_max: f32     
) -> f32 {
    return (((value - old_min) * (new_max - new_min)) / (old_max - old_min)) + new_min;
}

struct Material {
    scale: f32
};

@group(2) @binding(0)
var<uniform> material: Material;

@vertex
fn vertex(input: Vertex) -> VertexOutput {
    // Initialize the output structure
    var output: VertexOutput;

    output.position = mesh_position_local_to_clip(
        get_world_from_local(input.instance_index),
        vec4<f32>(input.position, 1.0)
    );
    output.world_normal = input.normal;

    let grass = vec4<f32>(0.08, 0.2, 0.05, 1.0);
    let rock = vec4<f32>(0.1, 0.1, 0.1, 1.0);

    let normalized_factor = 1.0 - linear_conversion(input.normal.y, 0.7, 0.8, 0.0, 1.0);

    if input.normal.y > 0.8 {
        output.color = grass;
    } else  if input.normal.y < 0.7 {
        output.color = rock;
    } else {
        output.color = mix(grass, rock, normalized_factor);
    }

    // output.color = input.color;

    // if angle <= 45.0 {
    //     output.color = vec4<f32>(0.5, 1.0, 0.5, 1.0);
    // } else {
    //     output.color = vec4<f32>(1.0, 0.5, 0.5, 1.0);
    // }

    return output;
}

@fragment
fn fragment(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    // let f: f32 = simplex_noise_3d(material.scale * in.world_position.xyz);

    // let color_a = vec3(1.0, 0.0, 0.0);
    // let color_b = vec3(0.0, 1.0, 0.0);
    // let mixed = mix(color_a, color_b, f);
    // return vec4(mixed, 1.0);

    return vec4(0.5, 0.5, 0.5, 1.0);
}