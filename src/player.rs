use super::{Map, Player, Position, RunState, State, TileType, Viewshed};
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut player_position = ecs.write_resource::<Point>();
    let map = ecs.fetch::<Map>();

    for (_player, position, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
        let destination_index = map.xy_index(position.x + delta_x, position.y + delta_y);

        player_position.x = position.x;
        player_position.y = position.y;

        if map.tiles[destination_index] != TileType::Wall {
            position.x = min(79, max(0, position.x + delta_x));
            position.y = min(49, max(0, position.y + delta_y));

            viewshed.dirty = true;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => return RunState::Paused,
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::A => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right | VirtualKeyCode::D => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up | VirtualKeyCode::W => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down | VirtualKeyCode::S => try_move_player(0, 1, &mut gs.ecs),
            _ => return RunState::Paused,
        },
    };

    return RunState::Running;
}
