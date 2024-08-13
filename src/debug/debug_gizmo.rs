use bevy::{color::palettes::css::*, prelude::*};
use bevy_fps_controller::controller::LogicalPlayer;
use std::f32::consts::PI;

use crate::ingame::{environment::terrain::{Chunks, TerrainConfig}, tricoord::{trichunk_to_coord, tricoord_vec_gen_distance, Coord}};

use super::TriBool;

pub struct DebugGizmoPlugin;

impl Plugin for DebugGizmoPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<GizmoConfig>()
        .add_systems(Update, origin_gizmo.run_if(resource_equals(GizmoConfig {origin_gizmo: TriBool::True, chunks_gizmo: TriBool::Wildcard})))
        .add_systems(Update, chunks_gizmo.run_if(resource_equals(GizmoConfig {origin_gizmo: TriBool::Wildcard, chunks_gizmo: TriBool::True})))
        ;
    }
}

#[derive(Resource, PartialEq)]
pub struct GizmoConfig {
    pub origin_gizmo: TriBool,
    pub chunks_gizmo: TriBool,
}

impl Default for GizmoConfig {
    fn default() -> Self {
        GizmoConfig {
            origin_gizmo: TriBool::True,
            chunks_gizmo: TriBool::False
        }
    }
}

fn origin_gizmo(
    mut gizmos: Gizmos
) {
    gizmos.sphere(Vec3::splat(0.), Quat::IDENTITY, 1.0, BLACK);

    let ray_length = 4.0;
    let ray_start = Vec3::new(0., 2., 0.);
    // +z
    gizmos.arrow(
        ray_start,
        ray_start + Vec3::new(0., 0., ray_length),
        BLUE,
    )
    .with_tip_length(0.5);
    // +x
    gizmos.arrow(
        ray_start,
        ray_start + Vec3::new(ray_length, 0., 0.),
        RED,
    )
    .with_tip_length(0.5);
}

fn chunks_gizmo(
    mut gizmos: Gizmos, 
    query: Query<&Transform, With<LogicalPlayer>>,
    chunks: Res<Chunks>,
    terrain_config: Res<TerrainConfig>
) /* -> Vec<TriCoord<i16>> */ {
    let player_transform = query.get_single().unwrap();

    let gen_origin = player_transform.translation; 

    gizmos.circle(gen_origin.with_y(0.0), Dir3::Y, terrain_config.chunk_gen_radius, BLACK);

    let in_radius_tricoords = tricoord_vec_gen_distance(Coord {x:gen_origin.x, z:gen_origin.z}, terrain_config.chunk_gen_radius);

    for tricoord in chunks.generating.clone() {
        let chunk_coord = trichunk_to_coord(tricoord, 0);
        gizmos.cuboid(Transform::from_xyz(chunk_coord.x as f32, 0.0, chunk_coord.z as f32), ORANGE);
    }

    for tricoord in chunks.generated.clone() {
        let chunk_coord = trichunk_to_coord(tricoord, 0);
        if in_radius_tricoords.contains(&tricoord) {
            gizmos.cuboid(Transform::from_xyz(chunk_coord.x as f32, 0.0, chunk_coord.z as f32), RED);
        } else {
            gizmos.cuboid(Transform::from_xyz(chunk_coord.x as f32, 0.0, chunk_coord.z as f32), BLUE);
        }
    }

}