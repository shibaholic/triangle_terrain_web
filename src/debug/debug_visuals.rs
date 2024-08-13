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
