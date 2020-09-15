use rltk::{GameState, Point, Rltk};
use specs::prelude::*;
mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod monster_ai_system;
use monster_ai_system::MonsterAI;
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod damage_system;
use damage_system::DamageSystem;
mod gamelog;
use gamelog::GameLog;
mod gui;
mod inventory_system;
mod spawner;
use inventory_system::EquipmentRemoveSystem;
use inventory_system::ItemBagSystem;
use inventory_system::ItemDropSystem;
use inventory_system::ItemUseSystem;
extern crate serde;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
mod random_table;
mod saveload_system;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    SaveGame,
    NextLevel,
    ShowRemoveEquipment,
    GameOver,
}

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);
        let mut inventory = ItemBagSystem {};
        inventory.run_now(&self.ecs);
        let mut use_item = ItemUseSystem {};
        use_item.run_now(&self.ecs);
        let mut drop_item = ItemDropSystem {};
        drop_item.run_now(&self.ecs);
        let mut equipment_remove = EquipmentRemoveSystem {};
        equipment_remove.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn register_components(&mut self) {
        self.ecs.register::<Position>();
        self.ecs.register::<Renderable>();
        self.ecs.register::<Player>();
        self.ecs.register::<Viewshed>();
        self.ecs.register::<Monster>();
        self.ecs.register::<Name>();
        self.ecs.register::<BlocksTile>();
        self.ecs.register::<CombatStats>();
        self.ecs.register::<WantsToMelee>();
        self.ecs.register::<SufferDamage>();
        self.ecs.register::<Item>();
        self.ecs.register::<ProvidesHealing>();
        self.ecs.register::<InBackpack>();
        self.ecs.register::<WantsToPickUpItem>();
        self.ecs.register::<WantsToUseItem>();
        self.ecs.register::<WantsToDropItem>();
        self.ecs.register::<Consumable>();
        self.ecs.register::<Ranged>();
        self.ecs.register::<InflictsDamage>();
        self.ecs.register::<AreaOfEffect>();
        self.ecs.register::<Confusion>();
        self.ecs.register::<SimpleMarker<SerializeMe>>();
        self.ecs.register::<SerializationHelper>();
        self.ecs.register::<Equippable>();
        self.ecs.register::<Equipped>();
        self.ecs.register::<MeleePowerBonus>();
        self.ecs.register::<DefenseBonus>();
        self.ecs.register::<WantsToRemoveEquipment>();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        ctx.cls();

        match newrunstate {
            RunState::MainMenu { .. } => {}
            RunState::GameOver { .. } => {}
            _ => {
                draw_map(&self.ecs, ctx);

                {
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let map = self.ecs.fetch::<Map>();

                    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));

                    for (pos, render) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.visible_tiles[idx] {
                            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
                        }
                    }

                    gui::draw_ui(&self.ecs, ctx);
                }
            }
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);

                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let ranged_entities = self.ecs.read_storage::<Ranged>();

                        if let Some(is_ranged) = ranged_entities.get(item_entity) {
                            newrunstate = RunState::ShowTargeting {
                                range: is_ranged.range,
                                item: item_entity,
                            }
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();

                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToUseItem {
                                        item: item_entity,
                                        target: None,
                                    },
                                )
                                .expect("Unable to insert intent");
                            newrunstate = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);

                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let result = gui::ranged_target(self, ctx, range);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToUseItem {
                                    item,
                                    target: result.1,
                                },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::MainMenu { .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection { selected } => {
                        newrunstate = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    gui::MainMenuResult::Selected { selected } => match selected {
                        gui::MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                        gui::MainMenuSelection::LoadGame => {
                            saveload_system::load_game(&mut self.ecs);
                            newrunstate = RunState::AwaitingInput;
                            saveload_system::delete_save();
                        }
                        gui::MainMenuSelection::Quit => ::std::process::exit(0),
                    },
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu {
                    menu_selection: gui::MainMenuSelection::LoadGame,
                };
            }
            RunState::NextLevel => {
                self.goto_next_level();
                newrunstate = RunState::PreRun;
            }
            RunState::ShowRemoveEquipment => {
                let result = gui::remove_equipment_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveEquipment>();

                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToRemoveEquipment { item: entity },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::NoSelection => {}
                    gui::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        newrunstate = RunState::MainMenu {
                            menu_selection: gui::MainMenuSelection::NewGame,
                        };
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        damage_system::delete_the_dead(&mut self.ecs);
    }
}

impl State {
    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let player_entity = self.ecs.fetch::<Entity>();
        let equipped = self.ecs.read_storage::<Equipped>();

        let mut to_delete: Vec<Entity> = Vec::new();
        for entity in entities.join() {
            let mut should_delete = true;

            if let Some(_p) = player.get(entity) {
                should_delete = false;
            }
            if let Some(backpack) = backpack.get(entity) {
                if backpack.owner == *player_entity {
                    should_delete = false;
                }
            }
            if let Some(equipment) = equipped.get(entity) {
                if equipment.owner == *player_entity {
                    should_delete = false;
                }
            }

            if should_delete {
                to_delete.push(entity);
            }
        }

        return to_delete;
    }

    fn goto_next_level(&mut self) {
        let to_delete = self.entities_to_remove_on_level_change();
        for entity in to_delete {
            self.ecs
                .delete_entity(entity)
                .expect("Unable to delete entity");
        }

        let world_map;
        let current_depth;
        {
            let mut world_map_resource = self.ecs.write_resource::<Map>();
            current_depth = world_map_resource.depth;
            *world_map_resource = Map::new_map_rooms_and_corridors(current_depth + 1);
            world_map = world_map_resource.clone();
        }

        // spawn enemies
        for room in world_map.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.ecs, room, current_depth + 1);
        }

        // place the player
        let (p_x, p_y) = world_map.rooms[0].center();
        let mut player_pos = self.ecs.write_resource::<Point>();
        *player_pos = Point::new(p_x, p_y);
        let mut position_components = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        if let Some(p_pos_comp) = position_components.get_mut(*player_entity) {
            p_pos_comp.x = p_x;
            p_pos_comp.y = p_y;
        }

        // mark viewsheds for update
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
        if let Some(viewshed) = viewshed_components.get_mut(*player_entity) {
            viewshed.dirty = true;
        }

        // heal the Player
        let mut player_health_store = self.ecs.write_storage::<CombatStats>();
        let mut amount_healed: i32 = 0;
        if let Some(player_health) = player_health_store.get_mut(*player_entity) {
            amount_healed = player_health.max_hp / 2;
            player_health.hp += amount_healed;
        }

        // write to the log
        let mut gamelog = self.ecs.fetch_mut::<GameLog>();
        gamelog.entries.push(format!(
            "You descend to floor {}, and heal {} hp.",
            world_map.depth, amount_healed,
        ));
    }

    fn game_over_cleanup(&mut self) {
        let mut to_delete = Vec::new();
        for entity in self.ecs.entities().join() {
            to_delete.push(entity);
        }
        for entity in to_delete.iter() {
            self.ecs.delete_entity(*entity).expect("Deletion failed");
        }

        let world_map;
        {
            let mut world_map_resource = self.ecs.write_resource::<Map>();
            *world_map_resource = Map::new_map_rooms_and_corridors(1);
            world_map = world_map_resource.clone()
        }

        // spawn enemies
        for room in world_map.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.ecs, room, 1);
        }

        // make new player
        let (p_x, p_y) = world_map.rooms[0].center();
        let player_entity = spawner::player(&mut self.ecs, p_x, p_y);
        let mut player_pos = self.ecs.write_resource::<Point>();
        *player_pos = Point::new(p_x, p_y);
        let mut position_components = self.ecs.write_storage::<Position>();
        let mut player_entity_writer = self.ecs.write_resource::<Entity>();
        *player_entity_writer = player_entity;
        if let Some(p_pos_comp) = position_components.get_mut(player_entity) {
            p_pos_comp.x = p_x;
            p_pos_comp.y = p_y;
        }

        // mark viewshed for update
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
        if let Some(viewshed) = viewshed_components.get_mut(player_entity) {
            viewshed.dirty = true;
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_title("Generic Roguelike")
        .build()?;
    context.with_post_scanlines(true);
    let mut gs = State { ecs: World::new() };
    gs.register_components();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    let map: Map = Map::new_map_rooms_and_corridors(1);
    let (player_x, player_y) = map.rooms[0].center();
    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

    gs.ecs.insert(rltk::RandomNumberGenerator::new());

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room, 1);
    }

    gs.ecs.insert(GameLog {
        entries: vec!["Welcome to Generic Roguelike".to_string()],
    });
    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::MainMenu {
        menu_selection: gui::MainMenuSelection::NewGame,
    });

    return rltk::main_loop(context, gs);
}
