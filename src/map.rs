use rltk::{Rltk, RGB};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub fn xy_index(x: i32, y: i32) -> usize {
    const WINDOW_WIDTH: i32 = 80;
    return (y as usize * WINDOW_WIDTH as usize) + x as usize;
}

pub fn new_map() -> Vec<TileType> {
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
