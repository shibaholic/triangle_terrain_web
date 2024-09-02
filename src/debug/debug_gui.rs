use bevy::{ecs::{query, system::SystemState}, pbr::wireframe::{Wireframe, WireframeConfig}, prelude::*, scene::ron::value::Float, window::PrimaryWindow};
use bevy_egui::{egui::{self, panel, FontId, RichText}, EguiContext, EguiContexts, EguiPlugin};
use bevy_fps_controller::controller::LogicalPlayer;
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};

use crate::ingame::{environment::terrain::{SelectedTerrainMat, TerrainConfig, TerrainHandles}, tricoord::{halfsides_altitude_to_tricoord, Coord, TriCoord, CHUNK_ALTITUDE, CHUNK_HALFSIDE, CHUNK_SIDE}};
use crate::debug::debug_gizmo::GizmoConfig;

use super::{debug_oneshots::OneShotSystems, TriBool};


pub struct DebugGUIPlugin;

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct DebugPanelConfig {
    left_width: f32,
    left_open: bool,
    right_width: f32,
    hidden: bool,
}

impl Default for DebugPanelConfig {
    fn default() -> Self {
        DebugPanelConfig { 
            left_width: 100.0,
            left_open: false,
            right_width: 100.0, 
            hidden: true, 
        } 
    }
}

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct DebugToolsData {
    player_coord: Coord<f32>,
    player_halfside_altitude: (i16, i16),
    player_chunk_tricoord: TriCoord<i16>,
}

impl Default for DebugToolsData {
    fn default() -> Self {
        DebugToolsData { 
            player_halfside_altitude: (0,0),
            player_coord: Coord { z: 0.0, x: 0.0 },
            player_chunk_tricoord: TriCoord { a:0, b:0, c:0 },
        }
    }
}

impl Plugin for DebugGUIPlugin {
    fn build(&self, app: &mut App) {
        app
        // .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(EguiPlugin)
        .add_plugins(bevy_inspector_egui::DefaultInspectorConfigPlugin)

        .init_resource::<DebugPanelConfig>()
        .register_type::<DebugPanelConfig>()

        .init_resource::<DebugToolsData>()
        .register_type::<DebugToolsData>()

        .add_systems(Update, left_panel.run_if(left_panel_open))  // bevy_inspector
        .add_systems(Update, right_panel) // debug tools
        .add_systems(Update, debug_inputs)
        .add_systems(Update, update_tools_data)
        ;
    }
}

fn left_panel_open(
    panel_config: Res<DebugPanelConfig>
) -> bool {
    panel_config.left_open
}

fn left_panel(
    world: &mut World,
) {    
    if world.get_resource::<DebugPanelConfig>().unwrap().hidden == true {
        return;
    }

    let Ok(egui_context) = world
        .query_filtered::<&EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };

    let mut egui_context = egui_context.clone();

    let width = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(egui_context.get_mut(), |ui| {
            
            egui::ScrollArea::vertical().show(ui, |ui| {     
                ui.heading("Entities");            
                bevy_inspector_egui::bevy_inspector::ui_for_world_entities(world, ui);
                ui.heading("Resources");
                bevy_inspector_egui::bevy_inspector::ui_for_resources(world, ui);
            });

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();

    // fought the borrow checker. this way world can be used inside the ui closure for bevy_inspector
    world.get_resource_mut::<DebugPanelConfig>().unwrap().left_width = width;
}

fn right_panel(
    mut contexts: EguiContexts,
    mut panel_config: ResMut<DebugPanelConfig>,
    tools_data: Res<DebugToolsData>,
    // mut wireframe_config: ResMut<WireframeConfig>,
    mut terrain_config: ResMut<TerrainConfig>,
    mut selected_mat: ResMut<SelectedTerrainMat>,
    mut terrain_hdls: ResMut<TerrainHandles>,
    debug_oneshots: Res<OneShotSystems>,
    mut commands: Commands,
    mut gizmo_config: ResMut<GizmoConfig>
) {
    if panel_config.hidden {
        return;
    }

    let ctx = contexts.ctx_mut();
    panel_config.right_width = egui::SidePanel::right("right_panel")
        .resizable(true)
        .show(ctx, |ui| {

            ui.separator();
            ui.heading("Player coordinates");

            ui.horizontal(|ui| {
                ui.label("World (z,x)");
                ui.code(format!("({:.02}, {:.02})",tools_data.player_coord.z, tools_data.player_coord.x));
            });

            ui.horizontal(|ui| {
                ui.label("Chunk coordinates, (halfsides & altitude)");
                ui.code(format!("({}, {})",tools_data.player_halfside_altitude.0, tools_data.player_halfside_altitude.1));
            });

            ui.horizontal(|ui| {
                ui.label("Chunk coordinates, tricoord (a,b,c)");
                ui.code(format!("({}, {}, {})",tools_data.player_chunk_tricoord.a, tools_data.player_chunk_tricoord.b, tools_data.player_chunk_tricoord.c));
            });
            ui.label(RichText::new("*tricoords are not chunk-border accurate for chunks in horizontal direction").font(FontId::proportional(10.0)));
            

            ui.separator();
            ui.heading("Procedural generation");

            ui.horizontal(|ui| {
                ui.label("Toggle chunk generation");
                ui.add(toggle(&mut terrain_config.active));
            });

            ui.horizontal(|ui| {
                ui.label("Chunk generation radius");
                ui.add(egui::Slider::new(&mut terrain_config.chunk_gen_radius, 0.0..=400.0));
            });

            ui.separator();
            ui.heading("Gizmos");

            ui.horizontal(|ui| {
                ui.label("Toggle origin gizmo");
                let mut origin_gizmo_bool = gizmo_config.origin_gizmo.to_bool();
                ui.add(toggle(&mut origin_gizmo_bool));
                gizmo_config.origin_gizmo = TriBool::from(origin_gizmo_bool);
            });

            ui.horizontal(|ui| {
                ui.label("Toggle chunk gizmos");
                let mut chunk_gizmo_bool = gizmo_config.chunks_gizmo.to_bool();
                ui.add(toggle(&mut chunk_gizmo_bool));
                gizmo_config.chunks_gizmo = TriBool::from(chunk_gizmo_bool);
            });

            ui.separator();
            ui.heading("Terrain material");

            egui::ComboBox::from_label("Pick a material")
            .selected_text(selected_mat.selected_mat.clone())
            .show_ui(ui, |ui| {
                for (key, value) in terrain_hdls.mat_hdls.clone().iter() {
                    if ui.selectable_value(&mut selected_mat.selected_mat, key.clone(), key).changed() {
                        let id = debug_oneshots.0["change_terrain_material"];
                        commands.run_system(id);
                    }
                }
            });

            ui.separator();
            ui.heading("Inspector panel");

            ui.horizontal(|ui| {
                ui.label("Toggle inspector panel");
                ui.add(toggle(&mut panel_config.left_open));
            });

            // ui.horizontal(|ui| {
            //     ui.label("Toggle wireframe");
            //     ui.add(toggle(&mut wireframe_config.global));
            // });

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
}

fn toggle(on: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| toggle_ui(ui, on)
}
fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
    });

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }

    response
}

fn debug_inputs(key: Res<ButtonInput<KeyCode>>, mut debug_panels: ResMut<DebugPanelConfig>) {
    if key.just_pressed(KeyCode::KeyU) {
        debug_panels.hidden = !debug_panels.hidden;
    }
}

fn update_tools_data(mut tools_data: ResMut<DebugToolsData>, query: Query<& Transform, With<LogicalPlayer>>) {
    let transform = query.single();
    let (x,z) = (transform.translation.x, transform.translation.z);
    tools_data.player_coord = Coord { z: z, x: x };
    // tools_data.player_chunk_tricoord = Coord { 
    //     z: flooring_division(transform.translation.z as i16, CHUNK_SIDE as i16), 
    //     x: flooring_division(transform.translation.x as i16, CHUNK_SIDE as i16) 
    // };
    let floor_or_ceil = |value:f64| {
        if value > 0.0 { // positive so floor it
            value.floor()
        } else { // negative so ceil it
            value.ceil()
        }
    };

    let pad = |value:f64, padding:f64| {
        if value > 0.0 {
            value as f64 + padding / 2.0
        } else {
            value as f64 - padding / 2.0
        }
    };

    let halfsides = (
        floor_or_ceil(
            pad(x as f64, CHUNK_HALFSIDE)
        ) / CHUNK_HALFSIDE 
    ) as i32;
    
    let altitudes = (
        floor_or_ceil(
            pad(z as f64, CHUNK_ALTITUDE)
        ) / CHUNK_ALTITUDE 
    ) as i32;
    
    tools_data.player_halfside_altitude = (halfsides as i16, altitudes as i16);

    tools_data.player_chunk_tricoord = halfsides_altitude_to_tricoord(halfsides, altitudes);
}

fn flooring_division(dividend: i16, divisor: i16) -> i16 {
    // Perform the division
    let quotient = dividend / divisor;
    let remainder = dividend % divisor;

    // If the remainder is not zero and the signs of the dividend and divisor are different
    if remainder != 0 && (dividend < 0) != (divisor < 0) {
        // Subtract 1 from the quotient to floor the division result
        quotient - 1
    } else {
        quotient
    }
}