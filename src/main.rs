use rltk::{GameState, Point, Rltk, RGB};
use specs::prelude::*;

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
pub use player::*;
mod rect;
pub use rect::*;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod monster_ai_system;
use monster_ai_system::MonsterAI;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
}

pub struct State {
    pub ecs: World,
    pub run_state: RunState,
}

impl State {
    fn run_systems(&mut self) {
        let mut visibility = VisibilitySystem {};
        visibility.run_now(&self.ecs);
        let mut monster_ai = MonsterAI {};
        monster_ai.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        if self.run_state == RunState::Running {
            self.run_systems();
            self.run_state = RunState::Paused;
        } else {
            self.run_state = player_input(self, ctx);
        }

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (position, renderable) in (&positions, &renderables).join() {
            let index = map.xy_index(position.x, position.y);
            if map.visible_tiles[index] {
                ctx.set(
                    position.x,
                    position.y,
                    renderable.fg,
                    renderable.bg,
                    renderable.glyph,
                );
            }
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("RLTK Tutorial")
        .build()?;
    let mut gs = State {
        ecs: World::new(),
        run_state: RunState::Running,
    };
    let mut rng = rltk::RandomNumberGenerator::new();

    // Register Components
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();

    let map: Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    gs.ecs.insert(Point::new(player_x, player_y));

    // Make Player
    gs.ecs
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed {
            dirty: true,
            range: 8,
            visible_tiles: Vec::new(),
        })
        .with(Player {})
        .with(Name {
            value: "Player".to_string(),
        })
        .build();

    // Make Monsters
    for (index, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();
        let glyph: rltk::FontCharType;
        let name: String;
        let roll = rng.roll_dice(1, 3);

        match roll {
            1 => {
                glyph = rltk::to_cp437('o');
                name = "Orc".to_string();
            }
            _ => {
                glyph = rltk::to_cp437('g');
                name = "Goblin".to_string();
            }
        }

        gs.ecs
            .create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph: glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Monster {})
            .with(Name {
                value: format!("{} #{}", &name, index),
            })
            .build();
    }

    gs.ecs.insert(map);

    return rltk::main_loop(context, gs);
}
