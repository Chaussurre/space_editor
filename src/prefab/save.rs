use bevy::{prelude::*, utils::HashSet, tasks::IoTaskPool};
use std::{any::TypeId, default, fs::File, io::Write};

use crate::{inspector::registration::{EditorRegistry, EditorRegistryExt}, PrefabMarker};

#[derive(Reflect, Default, Component)]
#[reflect(Component)]
pub struct ChildrenPrefab(Vec<Entity>);

#[derive(Component, Default)]
struct PrefabLoader {
    pub path : String
}

impl ChildrenPrefab {
    pub fn from_children(children : &Children) -> Self {
        Self(children.to_vec())
    }
}

pub struct SavePrefabPlugin;

impl Plugin for SavePrefabPlugin {
    fn build(&self, app: &mut App) {
        app.editor_registry::<ChildrenPrefab>();


        app.init_resource::<SaveConfig>();
        app.add_state::<SaveState>();
        app.add_systems(OnEnter(SaveState::Save), 
        (prepare_children, apply_deferred, serialize_prefab, delete_prepared_children).chain());
    }
}

#[derive(Resource, Clone, Default)]
pub struct SaveConfig {
    pub path : String
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum SaveState {
    Save,
    #[default]
    Idle
}

fn prepare_children(
    mut commands : Commands,
    query : Query<(Entity, &Children), With<PrefabMarker>>
) {
    for (entity, children) in query.iter() {
        commands.entity(entity).insert(ChildrenPrefab::from_children(children));
    }
}

fn delete_prepared_children(
    mut commands : Commands,
    query : Query<Entity, With<ChildrenPrefab>>
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<ChildrenPrefab>();
    }
}

pub fn serialize_prefab(
    world : &mut World
){

    let config = world.resource::<SaveConfig>().clone();

    let mut prefab_query = world.query_filtered::<Entity, With<PrefabMarker>>();
    let entities = prefab_query.iter(world).collect::<Vec<_>>();

    let registry = world.resource::<EditorRegistry>().clone();
    let allow_types : Vec<TypeId> = registry.registry.read().iter().map(|a| {
        a.type_id()
    }).collect();
    let mut builder = DynamicSceneBuilder::from_world(world);
    builder
        .allow_all()
        .with_filter(
            SceneFilter::Allowlist(
                HashSet::from_iter(allow_types.iter().cloned())))
        .extract_entities(entities.iter().map(|e| *e));
    let scene = builder.build();

    let res = scene.serialize_ron(&world.resource::<AppTypeRegistry>());

    if let Ok(str) = res {
        IoTaskPool::get()
            .spawn(async move {
                // Write the scene RON data to file
                let path = config.path;
                File::create(format!("assets/{path}.scn.ron"))
                    .and_then(|mut file| file.write(str.as_bytes()))
                    .expect("Error while writing scene to file");

                info!("Saved prefab to file");
        })
        .detach();
    } else if let Err(e) = res {
        error!("failed to serialize prefab: {:?}", e);
    }

    world.resource_mut::<NextState<SaveState>>().set(SaveState::Idle);

}



fn load_prefab(
    mut commands : Commands,
    query : Query<(Entity, &PrefabLoader)>,
    asset_server : Res<AssetServer>
) {

}