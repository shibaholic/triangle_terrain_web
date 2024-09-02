mod debug_visuals;
mod debug_gui;
mod debug_gizmo;
mod debug_oneshots;

use bevy::prelude::*;

use debug_gui::DebugGUIPlugin;
use debug_visuals::DebugVisualsPlugin;
use debug_gizmo::DebugGizmoPlugin;
use debug_oneshots::DebugOneShotsPlugin;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(DebugVisualsPlugin)
        .add_plugins(DebugGUIPlugin)
        .add_plugins(DebugGizmoPlugin)
        .add_plugins(DebugOneShotsPlugin)
        ;

    }
}

pub enum TriBool {
    True,
    False,
    Wildcard,
}

impl PartialEq for TriBool {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TriBool::Wildcard, _) | (_, TriBool::Wildcard) => true,
            (TriBool::True, TriBool::True) => true,
            (TriBool::False, TriBool::False) => true,
            _ => false,
        }
    }
}

impl From<TriBool> for bool {
    fn from(tri_bool: TriBool) -> Self {
        match tri_bool {
            TriBool::True => true,
            TriBool::False => false,
            TriBool::Wildcard => true, // Decide what Wildcard should convert to
        }
    }
}

impl From<bool> for TriBool {
    fn from(value: bool) -> Self {
        if value {
            TriBool::True
        } else {
            TriBool::False
        }
    }
}

impl TriBool {
    pub fn to_bool(&self) -> bool {
        match self {
            TriBool::True => true,
            TriBool::False => false,
            TriBool::Wildcard => true, // Decide what Wildcard should convert to
        }
    }
}