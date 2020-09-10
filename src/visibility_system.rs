use super::{Map, Player, Position, Viewshed};
use rltk::{field_of_view, Point};
use specs::prelude::*;

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, position, player) = data;
        for (entity, viewshed, position) in (&entities, &mut viewshed, &position).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles =
                    field_of_view(Point::new(position.x, position.y), viewshed.range, &*map);
                viewshed
                    .visible_tiles
                    .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

                // if this is the player, reveal what they should see
                let p: Option<&Player> = player.get(entity);
                if let Some(_p) = p {
                    for tile in map.visible_tiles.iter_mut() {
                        *tile = false
                    }
                    for vis in viewshed.visible_tiles.iter() {
                        let index = map.xy_index(vis.x, vis.y);
                        map.revealed_tiles[index] = true;
                        map.visible_tiles[index] = true;
                    }
                }
            }
        }
    }
}
