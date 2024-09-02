#import bevy_pbr::forward_io::VertexOutput

#import bevy_shader_utils::simplex_noise_3d::simplex_noise_3d

struct Material {
    scale: f32
};

@group(2) @binding(0)
var<uniform> material: Material;

@fragment
fn fragment(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let f: f32 = simplex_noise_3d(material.scale * in.world_position.xyz);

    let color_a = vec3(1.0, 0.0, 0.0);
    let color_b = vec3(0.0, 1.0, 0.0);
    let mixed = mix(color_a, color_b, f);
    return vec4(mixed, 1.0);
}