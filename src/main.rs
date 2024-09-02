// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::asset::AssetMetaCheck;

mod ingame;
mod debug;

use std::f32::consts::TAU;

use bevy::{
     prelude::*, render::{camera::Exposure, }, window::CursorGrabMode
};

use bevy_rapier3d::prelude::*;

use bevy_fps_controller::controller::*;

use ingame::{environment::EnvironmentPlugin};
use debug::DebugPlugin;

use bevy_shader_utils::ShaderUtilsPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy game".to_string(), // ToDo
                        // Bind to canvas included in `index.html`
                        canvas: Some("#bevy".to_owned()),
                        fit_canvas_to_parent: true,
                        // Tells wasm not to override default event handling, like F5 and Ctrl+R
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins(ShaderUtilsPlugin)
        .add_plugins(DebugPlugin)

        .add_plugins(EnvironmentPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())

        .add_plugins(FpsControllerPlugin)

        .add_systems(Startup, setup)

        .add_systems(Update, manage_cursor)

        .run();
}

fn setup(
    mut commands: Commands, 
    mut window: Query<&mut Window>, 
    asset_server: Res<AssetServer>
) {
    let mut window = window.single_mut();
    window.title = String::from("FPS_2");

    let height = 2.0;
    let logical_entity = commands.spawn((
        Collider::cylinder(height / 2.0, 0.3),
        Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min
        },
        ActiveEvents::COLLISION_EVENTS,
        Velocity::zero(),
        RigidBody::Dynamic,
        Sleeping::disabled(),
        LockedAxes::ROTATION_LOCKED,
        AdditionalMassProperties::Mass(1.0),
        GravityScale(0.0),
        Ccd { enabled: true },
        TransformBundle::from_transform(Transform::from_translation(Vec3::new(0.0, 4.0, 0.0))),
        LogicalPlayer,
        FpsControllerInput {
            // the up and down ness. 0. is level. -pi/2 is looking down. pi/2 is looking up.
            pitch: 0., //-TAU / 12.0,
            // the side to sideness. -z is 0.
            yaw: 0., // TAU * 5.0 / 8.0,
            ..default()
        },
        FpsController {
            air_acceleration: 80.0,
            key_crouch: KeyCode::KeyC,
            ..default()
        },
    ))
    .insert(CameraConfig {
        height_offset: -0.5
    })
    .insert(Name::new("LogicalPlayer"))
    .id();

    commands.spawn((
        Camera3dBundle {
            projection: Projection::Perspective(PerspectiveProjection {
                fov: TAU / 5.0,
                ..default()
            }),
            camera: Camera {
                hdr: true,
                ..default()
            },

            ..default()
        },
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 2000.0,
        },
        RenderPlayer { logical_entity }
    ))
    .insert(Name::new("RenderPlayer"));
}

struct MouseLocked {
    locked: bool
}

impl Default for MouseLocked {
    fn default() -> Self {
        return Self { locked: false };
    }
}

fn manage_cursor(
    btn: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<&mut Window>,
    mut controller_query: Query<&mut FpsController>,
    mut mouse_locked_local: Local<MouseLocked>
) {
    for mut window in &mut window_query {
        if key.just_pressed(KeyCode::Escape) {
            mouse_locked_local.locked = false;
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
            for mut controller in &mut controller_query {
                controller.enable_input = false;
            }
        }
        if key.just_pressed(KeyCode::Backquote) {
            if !mouse_locked_local.locked {
                mouse_locked_local.locked = true;
                window.cursor.grab_mode = CursorGrabMode::Locked;
                window.cursor.visible = false;
                for mut controller in &mut controller_query {
                    controller.enable_input = true;
                }
            } else {
                mouse_locked_local.locked = false;
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
                for mut controller in &mut controller_query {
                    controller.enable_input = false;
                }
            }
        }
    }
}