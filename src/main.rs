use rltk::{FontCharType, GameState, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};

#[derive(Component)]
struct Postion {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(Component)]
struct LeftMover {}

struct State {
    ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        self.run_systems();
        player_input(self, ctx);

        let positions = self.ecs.read_storage::<Postion>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (position, renderable) in (&positions, &renderables).join() {
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

impl State {
    fn run_systems(&mut self) {
        let mut left_walker = LeftWalker {};
        left_walker.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

struct LeftWalker {}

impl<'a> System<'a> for LeftWalker {
    type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Postion>);

    fn run(&mut self, (lefty, mut pos): Self::SystemData) {
        for (_lefty, pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;

            if pos.x < 0 {
                pos.x = 79;
            }
        }
    }
}

#[derive(Component, Debug)]
struct Player {}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Postion>();
    let mut players = ecs.write_storage::<Player>();

    for (_player, position) in (&mut players, &mut positions).join() {
        position.x = min(79, max(0, position.x + delta_x));
        position.y = min(49, max(0, position.y + delta_y));
    }
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            _ => {}
        },
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("RLTK Tutorial")
        .build()?;
    let mut gs = State { ecs: World::new() };

    // Register Components
    gs.ecs.register::<Postion>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();

    // Build entities
    gs.ecs
        .create_entity()
        .with(Postion { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();

    for i in 0..10 {
        gs.ecs
            .create_entity()
            .with(Postion { x: i * 7, y: 20 })
            .with(Renderable {
                glyph: rltk::to_cp437('☺'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(LeftMover {})
            .build();
    }

    rltk::main_loop(context, gs)
}
