use crate::{components::tile_transform::TileTransform, level::SpriteRequest, HEIGHT, WIDTH};
use noise::{Fbm, NoiseFn, Seedable};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use rayon::{iter::ParallelIterator, prelude::IntoParallelIterator};
use std::collections::HashMap;

pub const PERLIN_SCALE: f64 = 5.0;

///for walls which need more info than 8 bits
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum WallType {
    Back,
    Front,
    Left,
    Right,
}

lazy_static! {
    static ref BITS_TO_SPRS: HashMap<[Option<WallType>; 8], SpriteRequest> = {
        let mut map = HashMap::new();
        let mut a = |list: [Option<WallType>; 8], s: SpriteRequest| {
            map.insert(list, s);
        };

        use SpriteRequest::*;
        let n = None;
        let b = Some(WallType::Back);
        let f = Some(WallType::Front);
        let l = Some(WallType::Left);
        let r = Some(WallType::Right);

        a([n, n, n, b, b, n, n, n], BackWall);
        a([n, n, r, b, r, n, n, n], BackWall);
        a([l, n, n, l, b, n, n, n], BackWall);
        a([n, l, n, n, b, n, n, n], BackWallRightCorner);
        a([n, r, n, b, n, n, n, n], BackWallLeftCorner);

        a([n, n, n, n, f, n, l, n], FrontWallLeftCorner);
        a([n, n, n, f, n, n, r, n], FrontWallRightCorner);
        a([n, n, n, f, f, n, n, n], FrontWall);
        a([n, n, n, l, f, l, n, n], FrontWall);
        a([n, n, n, f, r, n, n, r], FrontWall);

        a([n, l, n, n, n, n, l, n], LeftWall);
        a([n, l, n, n, n, n, l, b], LeftWall);
        a([n, l, n, n, n, n, l, f], LeftWall);
        a([n, l, b, n, n, n, l, n], LeftWall);
        a([n, l, f, n, n, n, l, n], LeftWall);
        a([n, r, n, n, n, n, r, n], RightWall);
        a([n, r, n, n, n, b, r, n], RightWall);
        a([n, r, n, n, n, f, r, n], RightWall);
        a([b, r, n, n, n, n, r, n], RightWall);
        a([f, r, n, n, n, n, r, n], RightWall);

        map
    };
}

pub struct ProceduralGenerator {
    seed: u32,
}

type Map = Vec<(usize, usize, SpriteRequest)>;
type MapSlice = [(usize, usize, SpriteRequest)];

pub const TREE_THRESHOLD: f64 = 0.5;
pub const OVERRIDE_WALL_THRESHOLD: f64 = 0.4;
pub const SHRUBBERY_THRESHOLD: f64 = 0.0;
// pub const DOOR_REPLACER_MODIFIER: f64 = 5.0;

impl ProceduralGenerator {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    pub fn get(&self) -> Map {
        let mut map = Self::generate_walls_sprs(self.seed as u64);
        Self::add_plants(self.seed, &mut map);
        Self::add_players(self.seed, &mut map);

        map
    }

    fn generate_plants(seed: u32, map: &mut Map) {
        let blocked_bits: Vec<(usize, usize)> = map
            .clone()
            .into_par_iter()
            .filter(|(_, _, spr)| spr != &SpriteRequest::Blank && spr != &SpriteRequest::Door)
            .map(|(x, y, _)| (x, y))
            .collect();
        let plant_places: Vec<(usize, usize)> = map
            .clone()
            .into_par_iter()
            .filter(|(_, _, spr)| spr == &SpriteRequest::Door)
            .map(|(x, y, _)| (x, y))
            .collect();

    fn add_players (seed: u32, map: &mut Map) {
        let mut rng = Pcg64::seed_from_u64(seed as u64);
        let blocked_bits = Self::get_blocked_bits(map);

        #[allow(clippy::needless_collect)]
            let no_players: Vec<i32> = (0..rng.gen_range(1..=4)).into_iter().map(|id| {
            let offset = 4 - id;
            rng.gen_range(2*offset..5*offset)
        }).collect();

        let mut players = Vec::new();
        for (id, no) in no_players.into_iter().enumerate() {
            for _ in 0..no {
                loop {
                    let x = rng.gen_range(0..WIDTH as usize);
                    let y = rng.gen_range(0..HEIGHT as usize);
                    if !blocked_bits.contains(&(x, y)) && !players.contains(&(x, y)) {
                        players.push((x, y));
                        map.push((x, y, SpriteRequest::Player(id)));
                        break;
                    }
                }
            }
        }
    }

    fn add_plants(seed: u32, map: &mut Map) {
        let blocked_bits = Self::get_blocked_bits(map);
        let plant_places: Vec<(usize, usize)> = map
            .clone()
            .into_par_iter()
            .filter(|(_, _, spr)| spr == &SpriteRequest::Door)
            .map(|(x, y, _)| (x, y))
            .collect();

        let p1 = Fbm::new().set_seed(seed);
        let p2 = Fbm::new().set_seed(seed + 100);
        let p3 = Fbm::new().set_seed(seed / 3); //for overrides

        let (sender, receiver) = channel();

        (0..WIDTH as usize)
            .into_par_iter()
            .for_each_with(sender, |s, x| {
                let (sender, receiver) = channel();

                (0..HEIGHT as usize)
                    .into_par_iter()
                    .for_each_with(sender, |s, y| {
                        let p_val = [x as f64 / PERLIN_SCALE, y as f64 / PERLIN_SCALE];

                        let no_1 = p1.get(p_val);
                        let no_2 = p2.get(p_val);
                        let no_3 = if plant_places.contains(&(x, y)) {
                            if no_2 > SHRUBBERY_THRESHOLD {
                                Some(1)
                            } else {
                                Some(0)
                            }
                        } else {
                            None
                        };

                    let mut changer = |shrubbery: SpriteRequest,
                                       tree_spr: SpriteRequest,
                                       v: f64| {
                        if (blocked_bits.contains(&(x, y)) && can_override) || v > TREE_THRESHOLD {
                            map.push((x, y, tree_spr));
                        } else if v > SHRUBBERY_THRESHOLD {
                            map.push((x, y, shrubbery));
                        }
                    });

                    if let Some((t, val)) = no_3 {
                        if t == 0 {
                            changer(
                                SpriteRequest::Shrubbery,
                                SpriteRequest::Tree,
                                val.abs() * DOOR_REPLACER_MODIFIER,
                            );
                        } else {
                            changer(
                                SpriteRequest::DarkShrubbery,
                                SpriteRequest::WarpedTree,
                                val.abs() * DOOR_REPLACER_MODIFIER,
                            );
                        }
                    }
                    if no_1 > 0.0 {
                        changer(SpriteRequest::Shrubbery, SpriteRequest::Tree, no_1);
                    }
                    if no_2 > 0.0 {
                        changer(
                            SpriteRequest::DarkShrubbery,
                            SpriteRequest::WarpedTree,
                            no_2,
                        );
                    }
                }
            }
        }
    }

    fn generate_walls_sprs(seed: u64) -> Map {
        let mut rng = Pcg64::seed_from_u64(seed as u64);
        let walls: [[Option<WallType>; HEIGHT as usize]; WIDTH as usize] =
            Self::generate_walls(&mut rng);
        println!("{:?}", walls);

        let get_bits = |x: usize, y: usize| {
            let thing_works = |xo: i32, yo: i32| {
                let xtot = x as i32 + xo;
                let ytot = y as i32 + yo;
                if !(0..WIDTH).contains(&xtot) || !(0..HEIGHT).contains(&ytot) {
                    None
                } else {
                    walls[xtot as usize][ytot as usize]
                }
            };

            [
                thing_works(-1, 1),
                thing_works(0, 1),
                thing_works(1, 1),
                thing_works(-1, 0),
                thing_works(1, 0),
                thing_works(-1, -1),
                thing_works(0, -1),
                thing_works(1, -1),
            ]
        };

        let mut map = Vec::new();

        for (x, col) in walls.iter().enumerate().take(WIDTH as usize) {
            for (y, cell) in col
                .iter()
                .enumerate()
                .take(HEIGHT as usize)
                .map(|(y, cell)| (y, cell.is_some()))
            {
                if cell {
                    let bits = get_bits(x, y);

                    let spr = *BITS_TO_SPRS.get(&bits).unwrap_or(&SpriteRequest::Door);
                    let res = (x, y, spr);

                    map.push(res);
                }
            }
        }

        map
    }

    fn generate_walls(rng: &mut Pcg64) -> [[Option<WallType>; HEIGHT as usize]; WIDTH as usize] {
        let no_rooms = rng.gen_range(3..=8);

        //we generate the x and y for no_roo rooms
        let room_max_width = 20;
        let room_max_height = 20;

        let mut gen_room = || -> (TileTransform, TileTransform) {
            let x_pos = rng.gen_range(0..(WIDTH as usize - room_max_width));
            let y_pos = rng.gen_range(0..(HEIGHT as usize - room_max_height));

            let width = rng.gen_range(4..room_max_width);
            let height = rng.gen_range(4..room_max_height);

            let tup: (TileTransform, TileTransform) = (
                (x_pos, y_pos).into(),
                (x_pos + width, y_pos + height).into(),
            );
            log::info!("Making {:?}", tup);
            tup
        };

        let mut rooms = Vec::new();
        (0..no_rooms)
            .into_iter()
            .for_each(|_| rooms.push(gen_room()));

        let mut map = [[None; HEIGHT as usize]; WIDTH as usize];

        for (top_left, btm_right) in rooms {
            for col in map
                .iter_mut()
                .take((btm_right.x as usize) + 1)
                .skip(top_left.x as usize)
            {
                col[top_left.y as usize] = Some(WallType::Back);
                col[btm_right.y as usize] = Some(WallType::Front);
            }

            for y in (top_left.y as usize)..=(btm_right.y as usize) {
                map[top_left.x as usize][y] = Some(WallType::Left);
                map[btm_right.x as usize][y] = Some(WallType::Right);
            }
        }

        map
    }
}
