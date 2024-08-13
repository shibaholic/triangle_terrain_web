use std::f32::consts::PI;

use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use terrain::TerrainPlugin;

pub mod terrain;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, setup_ambience)
        .add_plugins(TerrainPlugin)
        ;
    }
}

fn setup_ambience(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    // direction light
    commands.spawn(
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 15_000.,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_rotation(Quat::from_euler(
                EulerRot::ZYX,
                0.0,
                PI * -0.15,
                PI * -0.15,
            )),
            cascade_shadow_config: CascadeShadowConfigBuilder {
                maximum_distance: 100.0,
                first_cascade_far_bound: 50.0,
                ..default()
            }
            .into(),
            ..default()
        }
    );

    // background color
    commands.insert_resource(ClearColor(Color::srgb(1.0, 1.0, 1.0)));
}




