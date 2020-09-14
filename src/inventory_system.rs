use super::{
    AreaOfEffect, CombatStats, Confusion, Consumable, Equippable, Equipped, GameLog, InBackpack,
    InflictsDamage, Map, Name, Position, ProvidesHealing, SufferDamage, WantsToDropItem,
    WantsToPickUpItem, WantsToRemoveEquipment, WantsToUseItem,
};
use specs::prelude::*;

pub struct ItemBagSystem {}

impl<'a> System<'a> for ItemBagSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickUpItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_to_pickup, mut positions, names, mut backpack) =
            data;

        for pickup in wants_to_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert backpack entry");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!(
                    "You pick up the {}.",
                    names.get(pickup.item).unwrap().value
                ));
            }
        }

        wants_to_pickup.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            map,
            entities,
            mut item_to_use,
            names,
            healing_items,
            damaging_items,
            aoe_items,
            mut confusions,
            mut combat_stats,
            mut suffer_damage,
            consumables,
            equippable_items,
            mut equipped_items,
            mut backpack,
        ) = data;

        for (entity, item_used) in (&entities, &mut item_to_use).join() {
            let mut item_is_used = false;

            // targeting
            let mut targets: Vec<Entity> = Vec::new();
            match item_used.target {
                None => targets.push(*player_entity),
                Some(target) => {
                    let aoe = aoe_items.get(item_used.item);
                    match aoe {
                        None => {
                            let idx = map.xy_idx(target.x, target.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                        Some(aoe) => {
                            let mut blast_tiles = rltk::field_of_view(target, aoe.radius, &*map);
                            blast_tiles.retain(|p| {
                                p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1
                            });
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                }
                            }
                        }
                    }
                }
            }

            let equippable_item = equippable_items.get(item_used.item);
            match equippable_item {
                None => {}
                Some(equip) => {
                    let target_slot = equip.slot;
                    let target = targets[0];

                    let mut to_unequip: Vec<Entity> = Vec::new();
                    for (item_entity, already_equipped, name) in
                        (&entities, &equipped_items, &names).join()
                    {
                        if already_equipped.owner == target && already_equipped.slot == target_slot
                        {
                            to_unequip.push(item_entity);
                            if target == *player_entity {
                                gamelog
                                    .entries
                                    .push(format!("You enquip the {}.", name.value));
                            }
                        }
                    }
                    for item in to_unequip.iter() {
                        equipped_items.remove(*item);
                        backpack
                            .insert(*item, InBackpack { owner: target })
                            .expect("Unable to insert backpack entry");
                    }

                    // wield it
                    equipped_items
                        .insert(
                            item_used.item,
                            Equipped {
                                owner: target,
                                slot: target_slot,
                            },
                        )
                        .expect("Unable to insert equipped component");
                    backpack.remove(item_used.item);
                    if target == *player_entity {
                        gamelog.entries.push(format!(
                            "You equip the {}.",
                            names.get(item_used.item).unwrap().value
                        ));
                    }
                }
            }

            let healing_item = healing_items.get(item_used.item);
            match healing_item {
                None => {}
                Some(heal) => {
                    for target in targets.iter() {
                        if let Some(stats) = combat_stats.get_mut(*target) {
                            stats.hp = i32::min(stats.max_hp, stats.hp + heal.heal_amount);
                            if entity == *player_entity {
                                gamelog.entries.push(format!(
                                    "You use the {}, healing for {}.",
                                    names.get(item_used.item).unwrap().value,
                                    heal.heal_amount
                                ));
                            }
                            item_is_used = true;
                        }
                    }
                }
            }

            let damaging_item = damaging_items.get(item_used.item);
            match damaging_item {
                None => {}
                Some(damage) => {
                    for target in targets.iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *target, damage.damage);
                        if entity == *player_entity {
                            gamelog.entries.push(format!(
                                "You use the {} on {}, dealing {} damage.",
                                names.get(item_used.item).unwrap().value,
                                names.get(*target).unwrap().value,
                                damage.damage
                            ));
                        }
                        item_is_used = true;
                    }
                }
            }

            let mut add_confusion = Vec::new();
            let causes_confusion = confusions.get(item_used.item);
            match causes_confusion {
                None => {}
                Some(confusion) => {
                    for target in targets.iter() {
                        add_confusion.push((*target, confusion.turns));
                        if entity == *player_entity {
                            gamelog.entries.push(format!(
                                "You use the {} on {}, confusing them.",
                                names.get(item_used.item).unwrap().value,
                                names.get(*target).unwrap().value
                            ));
                        }
                    }
                    item_is_used = true;
                }
            }
            for (target, turns) in add_confusion.iter() {
                confusions
                    .insert(*target, Confusion { turns: *turns })
                    .expect("Unable to insert status");
            }

            if item_is_used {
                let consumable = consumables.get(item_used.item);
                match consumable {
                    None => {}
                    Some(_) => entities.delete(item_used.item).expect("Delete failed"),
                }
            }
        }

        item_to_use.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_to_drop_item,
            names,
            mut positions,
            mut backpack,
        ) = data;

        for (entity, to_drop) in (&entities, &wants_to_drop_item).join() {
            let mut dropper_pos: Position = Position { x: 0, y: 0 };
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions
                .insert(
                    to_drop.item,
                    Position {
                        x: dropper_pos.x,
                        y: dropper_pos.y,
                    },
                )
                .expect("Unable to insert position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!(
                    "You drop the {}.",
                    names.get(to_drop.item).unwrap().value
                ));
            }
        }

        wants_to_drop_item.clear();
    }
}

pub struct EquipmentRemoveSystem {}

impl<'a> System<'a> for EquipmentRemoveSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToRemoveEquipment>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_remove, mut equipped, mut backpack) = data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            equipped.remove(to_remove.item);
            backpack
                .insert(to_remove.item, InBackpack { owner: entity })
                .expect("Unable to insert backpack");
        }

        wants_remove.clear();
    }
}
