use bevy::prelude::*;
use space_editor::{editor::core::EditorEvent, prelude::*};

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpaceEditorPlugin::default())
        .add_systems(Startup, simple_editor_setup)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut editor_events: EventWriter<EditorEvent>) {
    editor_events.send(EditorEvent::LoadGltfAsPrefab(
        "low_poly_fighter_2.gltf".to_string(),
    ));
}