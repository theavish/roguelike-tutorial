use super::{xy_index, Player, Postion, State, TileType};
use rltk::{Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Postion>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, position) in (&mut players, &mut positions).join() {
        let destination_index = xy_index(position.x + delta_x, position.y + delta_y);

        if map[destination_index] != TileType::Wall {
            position.x = min(79, max(0, position.x + delta_x));
            position.y = min(49, max(0, position.y + delta_y));
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) {
    return match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            _ => {}
        },
    };
}
