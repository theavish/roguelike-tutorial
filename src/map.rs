use super::{Player, Rect, Viewshed};
use rltk::{Algorithm2D, BaseMap, Point, RandomNumberGenerator, Rltk, RGB};
use specs::prelude::*;
use std::cmp::{max, min};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
}

impl Map {
    pub fn xy_index(&self, x: i32, y: i32) -> usize {
        return (y as usize * self.width as usize) + x as usize;
    }
    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in (room.y1 + 1)..=room.y2 {
            for x in (room.x1 + 1)..=room.x2 {
                let index = self.xy_index(x, y);
                self.tiles[index] = TileType::Floor;
            }
        }
    }
    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let index = self.xy_index(x, y);
            if index > 0 && index < (self.width as usize * self.height as usize) {
                self.tiles[index as usize] = TileType::Floor;
            }
        }
    }
    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let index = self.xy_index(x, y);
            if index > 0 && index < (self.width as usize * self.height as usize) {
                self.tiles[index as usize] = TileType::Floor;
            }
        }
    }
    /// Make an 80x50 map with rooms, connected by hallways.
    /// Uses [this algorithm](http://rogueliketutorials.com/tutorials/tcod/part-3).
    pub fn new_map_rooms_and_corridors() -> Map {
        let mut rng = RandomNumberGenerator::new();
        let mut map = Map {
            tiles: vec![TileType::Wall; 80 * 50],
            rooms: Vec::new(),
            width: 80,
            height: 50,
            revealed_tiles: vec![false; 80 * 50],
            visible_tiles: vec![false; 80 * 50],
        };
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;

            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();

                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        return map;
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, index: usize) -> bool {
        return self.tiles[index] == TileType::Wall;
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        return Point::new(self.width, self.height);
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut y = 0;
    let mut x = 0;
    for (index, tile) in map.tiles.iter().enumerate() {
        // Render a tile depending upon the tile type
        if map.revealed_tiles[index] {
            let glyph;
            let mut fg;

            match tile {
                TileType::Floor => {
                    fg = RGB::from_f32(0.0, 0.25, 0.1);
                    glyph = rltk::to_cp437('.');
                }
                TileType::Wall => {
                    fg = RGB::from_f32(0.0, 0.8, 0.0);
                    glyph = rltk::to_cp437('#');
                }
            }
            if !map.visible_tiles[index] {
                fg = fg.to_greyscale();
            }

            ctx.set(x, y, fg, RGB::from_f32(0.0, 0.0, 0.0), glyph);
        }

        // Move the coordinates
        x += 1;
        if x > (map.width - 1) {
            x = 0;
            y += 1;
        }
    }
}
