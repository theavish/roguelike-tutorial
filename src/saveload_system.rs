use super::components::*;
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{
    DeserializeComponents, MarkedBuilder, SerializeComponents, SimpleMarker, SimpleMarkerAllocator,
};
use std::fs;
use std::fs::File;
use std::path::Path;

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}
macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        DeserializeComponents::<NoError, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &mut $data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocater
            &mut $de,
        )
        .unwrap();
        )*
    };
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(ecs: &mut World) {
    let map_copy = ecs.get_mut::<super::map::Map>().unwrap().clone();
    let save_helper = ecs
        .create_entity()
        .with(SerializationHelper { map: map_copy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    {
        let data = (
            ecs.entities(),
            ecs.read_storage::<SimpleMarker<SerializeMe>>(),
        );

        let writer = File::create("./savegame.json").unwrap();
        let mut serializer = serde_json::Serializer::new(writer);

        serialize_individually!(
            ecs,
            serializer,
            data,
            Position,
            Renderable,
            Player,
            Viewshed,
            Monster,
            Name,
            BlocksTile,
            CombatStats,
            SufferDamage,
            WantsToMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            WantsToPickUpItem,
            WantsToUseItem,
            WantsToDropItem,
            SerializationHelper
        );
    }

    ecs.delete_entity(save_helper).expect("Crash on cleanup");
}
#[cfg(target_arch = "wasm32")]
pub fn save_game(_ecs: &mut World) {}

pub fn load_game(ecs: &mut World) {
    {
        // clear current entities/state
        let mut to_delete = Vec::new();
        for entity in ecs.entities().join() {
            to_delete.push(entity);
        }
        for entity in to_delete.iter() {
            ecs.delete_entity(*entity).expect("Deletion failed");
        }
    }

    let reader = fs::read_to_string("./savegame.json").unwrap();
    let mut deserializer = serde_json::Deserializer::from_str(&reader);
    {
        let mut data = (
            &mut ecs.entities(),
            &mut ecs.write_storage::<SimpleMarker<SerializeMe>>(),
            &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>(),
        );

        deserialize_individually!(
            ecs,
            deserializer,
            data,
            Position,
            Renderable,
            Player,
            Viewshed,
            Monster,
            Name,
            BlocksTile,
            CombatStats,
            SufferDamage,
            WantsToMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            WantsToPickUpItem,
            WantsToUseItem,
            WantsToDropItem,
            SerializationHelper
        );
    }

    let mut entity_to_delete: Option<Entity> = None;
    {
        let entities = ecs.entities();
        let serialization_helper = ecs.read_storage::<SerializationHelper>();
        let player = ecs.read_storage::<Player>();
        let position = ecs.read_storage::<Position>();

        for (entity, helper) in (&entities, &serialization_helper).join() {
            let mut world_map = ecs.write_resource::<super::map::Map>();
            *world_map = helper.map.clone();
            world_map.tile_content = vec![Vec::new(); super::map::MAPCOUNT];
            entity_to_delete = Some(entity);
        }

        for (entity, _player, pos) in (&entities, &player, &position).join() {
            let mut player_pos = ecs.write_resource::<rltk::Point>();
            *player_pos = rltk::Point::new(pos.x, pos.y);

            let mut player_entity = ecs.write_resource::<Entity>();
            *player_entity = entity;
        }
    }
    ecs.delete_entity(entity_to_delete.unwrap())
        .expect("Unable to delete helper");
}

pub fn does_save_exist() -> bool {
    return Path::new("./savegame.json").exists();
}

pub fn delete_save() {
    if does_save_exist() {
        std::fs::remove_file("./savegame.json").expect("Unable to delete file");
    }
}
