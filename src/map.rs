use super::Rect;
use rltk::{RandomNumberGenerator, Rltk, RGB};
use std::cmp::{max, min};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub fn xy_index(x: i32, y: i32) -> usize {
    const WINDOW_WIDTH: i32 = 80;
    return (y as usize * WINDOW_WIDTH as usize) + x as usize;
}

/// Make an 80x50 map with rooms, connected by hallways
pub fn new_map_rooms_and_corridors() -> (Vec<Rect>, Vec<TileType>) {
    let mut map = vec![TileType::Wall; 80 * 50];

    let mut rooms: Vec<Rect> = Vec::new();
    const MAX_ROOMS: i32 = 30;
    const MIN_SIZE: i32 = 6;
    const MAX_SIZE: i32 = 10;
    let mut rng = RandomNumberGenerator::new();

    for _ in 0..MAX_ROOMS {
        let w = rng.range(MIN_SIZE, MAX_SIZE);
        let h = rng.range(MIN_SIZE, MAX_SIZE);
        let x = rng.roll_dice(1, 80 - w - 1) - 1;
        let y = rng.roll_dice(1, 50 - h - 1) - 1;
        let new_room = Rect::new(x, y, w, h);
        let mut ok = true;

        for other_room in rooms.iter() {
            if new_room.intersect(other_room) {
                ok = false
            }
        }
        if ok {
            apply_room_to_map(&new_room, &mut map);

            if !rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                if rng.range(0, 2) == 1 {
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y);
                    apply_vertical_tunnel(&mut map, prev_y, new_y, new_x);
                } else {
                    apply_vertical_tunnel(&mut map, prev_y, new_y, new_x);
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y);
                }
            }

            rooms.push(new_room);
        }
    }

    return (rooms, map);
}

/// Make an 80x50 map with 400 Walls, randomly placed.
pub fn new_map_test() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; 80 * 50];

    // make boundaries
    for x in 0..80 {
        map[xy_index(x, 0)] = TileType::Wall;
        map[xy_index(x, 49)] = TileType::Wall;
    }
    for y in 0..50 {
        map[xy_index(0, y)] = TileType::Wall;
        map[xy_index(79, y)] = TileType::Wall;
    }

    // add random walls
    let mut rng = rltk::RandomNumberGenerator::new();

    for _i in 0..400 {
        let x = rng.roll_dice(1, 79);
        let y = rng.roll_dice(1, 49);
        let index = xy_index(x, y);

        if index != xy_index(40, 25) {
            map[index] = TileType::Wall;
        }
    }

    return map;
}

pub fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;

    for tile in map.iter() {
        match tile {
            TileType::Floor => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.05, 0.1, 0.05),
                    RGB::from_f32(0.0, 0.0, 0.0),
                    rltk::to_cp437('.'),
                );
            }
            TileType::Wall => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.0, 1.0, 0.0),
                    RGB::from_f32(0.0, 0.0, 0.0),
                    rltk::to_cp437('#'),
                );
            }
        }

        x += 1;

        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}

fn apply_room_to_map(room: &Rect, map: &mut [TileType]) {
    for y in (room.y1 + 1)..=room.y2 {
        for x in (room.x1 + 1)..=room.x2 {
            map[xy_index(x, y)] = TileType::Floor;
        }
    }
}

fn apply_horizontal_tunnel(map: &mut [TileType], x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        let index = xy_index(x, y);
        if index > 0 && index < 80 * 50 {
            map[index as usize] = TileType::Floor;
        }
    }
}

fn apply_vertical_tunnel(map: &mut [TileType], y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let index = xy_index(x, y);
        if index > 0 && index < 80 * 50 {
            map[index as usize] = TileType::Floor;
        }
    }
}
