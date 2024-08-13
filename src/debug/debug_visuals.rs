use std::process::Command;

use bevy::{color::palettes::css::{DEEP_PINK, LIME, RED, WHITE}, pbr::wireframe::{Wireframe, WireframeColor, WireframeConfig}, prelude::*};

use bevy_dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};

pub struct DebugVisualsPlugin;

impl Plugin for DebugVisualsPlugin {
    fn build(&self, app: &mut App) {
        app
        // .insert_resource(WireframeConfig {
        //     // The global wireframe config enables drawing of wireframes on every mesh,
        //     // except those with `NoWireframe`. Meshes with `Wireframe` will always have a wireframe,
        //     // regardless of the global configuration.
        //     global: true,
        //     // Controls the default color of all wireframes. Used as the default color for global wireframes.
        //     // Can be changed per mesh using the `WireframeColor` component.
        //     default_color: WHITE.into(),
        // })
        .add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextStyle {
                    // Here we define size of our overlay
                    font_size: 50.0,
                    // We can also change color of the overlay
                    color: Color::srgb(0.0, 1.0, 0.0),
                    // If we want, we can use a custom font
                    font: default(),
                },
            },
        },)
        .add_systems(PostStartup, spawn_help_text)
        .add_systems(Update, update_help_text)
        ;
    }
}

// fn setup(mut commands: Commands) {
//     // Text used to show controls
//     commands.spawn(
//         TextBundle::from_section("", TextStyle::default()).with_style(Style {
//             position_type: PositionType::Absolute,
//             top: Val::Px(12.0),
//             left: Val::Px(12.0),
//             ..default()
//         }),
//     );
// }

#[derive(Resource)]
struct HelpTextBool(bool);

#[derive(Component)]
struct HelpText;

fn spawn_help_text(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle::default(),
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(36.0),
            left: Val::Px(12.0),
            ..default()
        }),
        HelpText
    )
    );
    commands.insert_resource::<HelpTextBool>( HelpTextBool { 0: false });
}

fn update_help_text(
    mut helptext: Query<&mut Text, With<HelpText>>,
    mut helptextbool: ResMut<HelpTextBool>,
    keys: Res<ButtonInput<KeyCode>>
) {
    let mut text = helptext.single_mut();
    let text = &mut text.sections[0].value;

    text.clear();

    text.push_str("Press H for help");

    if keys.just_pressed(KeyCode::KeyH) {
        helptextbool.0 = !helptextbool.0;
    }

    if helptextbool.0 {
        text.push_str(
            "\n (WASD) to walk
            \n (space) to jump
            \n (C) to crouch
            \n (shift) to run
            \n (U) to open debug panels
            \n (F) to toggle fly
            \n (Q) to descend while flying
            \n (E) to ascend while flying
        ");
    }
}