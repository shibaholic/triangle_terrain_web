use bevy::prelude::*;
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

fn setup_ambience(mut commands: Commands) {
    // direction light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 7.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    })
    .insert(Name::new("DirectionalLight"));
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 10000.0,
    });
    // background color
    commands.insert_resource(ClearColor(Color::linear_rgb(0.83, 0.96, 0.96)));
}




