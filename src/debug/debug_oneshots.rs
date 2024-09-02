use std::any::TypeId;

use bevy::{ecs::system::SystemId, pbr::ExtendedMaterial, prelude::*, utils::HashMap};

use crate::ingame::environment::terrain::{MyMaterial, SelectedTerrainMat, TerrainHandles, TerrainMesh};

pub struct DebugOneShotsPlugin;

impl Plugin for DebugOneShotsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OneShotSystems>();
    }
}

// one-shot systems register
#[derive(Resource)]
pub struct OneShotSystems(pub HashMap<String, SystemId>);

impl FromWorld for OneShotSystems {
    fn from_world(world: &mut World) -> Self {
        let mut one_shot_systems = OneShotSystems(HashMap::new());

        one_shot_systems.0.insert(
            "change_terrain_material".into(),
            world.register_system(change_material)
        );

        one_shot_systems
    }
}

fn change_material(
    mut commands: Commands,
    terrain_hdls: Res<TerrainHandles>,
    selected_mat: Res<SelectedTerrainMat>,
    mut query: Query<Entity, With<TerrainMesh>>,
) {
    for entity in query.iter_mut() {
        commands.entity(entity).remove::<Handle<StandardMaterial>>();

        let mat_type = terrain_hdls.mat_hdls.get(&selected_mat.selected_mat).unwrap().type_id();
        if mat_type == TypeId::of::<StandardMaterial>() {
            // standard material
            commands.entity(entity).insert(
                terrain_hdls.mat_hdls.get(&selected_mat.selected_mat).unwrap().clone()
                .typed::<StandardMaterial>()
            );
        } else if mat_type == TypeId::of::<ExtendedMaterial<StandardMaterial, MyMaterial>>() {
            // MyMaterial
            commands.entity(entity).insert(
                terrain_hdls.mat_hdls.get(&selected_mat.selected_mat).unwrap().clone()
                .typed::<ExtendedMaterial<StandardMaterial, MyMaterial>>()
            );
        }
    }
}