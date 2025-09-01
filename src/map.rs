// Map definitions

use rand::Rng;
use rand::distr::Distribution;
use rand::distr::slice::Choose;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum MapItem {
    Empty,
    Obstacle,
    HeroEntity,
    EnemyEntity,
    Goal,
    Junk,
}

#[derive(Debug)]
pub enum MapGenerationError {
    RanOutOfSpace,
    PrevOpFailed,
    InvalidWallCoverage,
}

#[derive(Clone)]
pub struct Map<const DIMX: usize, const DIMY: usize> {
    pub map: Option<[[MapItem; DIMY]; DIMX]>,
    unicode_mappings: HashMap<MapItem, char>,
}

impl MapItem {
    /// Generate default Unicode mappings that can be printed to the console.
    fn generate_default_unicode_mappings() -> HashMap<MapItem, char> {
        let mut mappings: HashMap<MapItem, char> = HashMap::new();

        mappings.insert(MapItem::Empty, ' ');
        mappings.insert(MapItem::Obstacle, '\u{2588}'); // unicode solid block
        mappings.insert(MapItem::HeroEntity, '\u{2022}'); // unicode bullet
        mappings.insert(MapItem::EnemyEntity, '\u{25B2}'); // unicode solid triangle
        mappings.insert(MapItem::Goal, '\u{25CE}'); // unicode bullseye
        mappings.insert(MapItem::Junk, '\u{2592}'); // medium shaded block

        return mappings;
    }
}

impl<const DIMX: usize, const DIMY: usize> Map<DIMX, DIMY> {
    /// Create a new Map object.
    pub fn new() -> Map<DIMX, DIMY> {
        assert!(DIMX > 0 && DIMY > 0);
        Map {
            map: Option::None,
            unicode_mappings: MapItem::generate_default_unicode_mappings(),
        }
    }

    /// Generate a map with approximate % coverage using a tetromino-based algorithm.
    /// Then, fill the map with n enemies, place the hero, and finally place the goal.
    pub fn generate_map(
        &self,
        wall_coverage: f32,
        num_enemies: usize,
    ) -> Result<[[MapItem; DIMY]; DIMX], MapGenerationError> {
        let mut new_map = self.generate_map_with_obstacles(wall_coverage);
        if new_map.is_err() {
            return new_map;
        }
        new_map = self.add_enemies_to_map(new_map.ok(), num_enemies);
        if new_map.is_err() {
            return new_map;
        }
        return self.add_hero_and_goal_to_map(new_map.ok());
    }

    /// Create a map and fill it with obstacles using a tetromino-based coverage algorithm.
    fn generate_map_with_obstacles(
        &self,
        wall_coverage: f32,
    ) -> Result<[[MapItem; DIMY]; DIMX], MapGenerationError> {
        let mut working_map: [[MapItem; DIMY]; DIMX] = [[MapItem::Empty; DIMY]; DIMX];

        if wall_coverage.clamp(0., 1.) != wall_coverage {
            return Err(MapGenerationError::InvalidWallCoverage);
        }

        struct Tetromino {
            shape: Vec<Vec<bool>>,
        }
        let mut tet_options: Vec<Tetromino> = Vec::new();
        tet_options.push(Tetromino {
            shape: vec![vec![false, false, false, false]],
        });
        tet_options.push(Tetromino {
            shape: vec![vec![false, true, true], vec![false, false, false]],
        });
        tet_options.push(Tetromino {
            shape: vec![vec![false, false, false], vec![false, true, true]],
        });
        tet_options.push(Tetromino {
            shape: vec![vec![false, false, true], vec![true, false, false]],
        });
        tet_options.push(Tetromino {
            shape: vec![vec![true, false, false], vec![false, false, true]],
        });
        tet_options.push(Tetromino {
            shape: vec![vec![true, false, true], vec![false, false, false]],
        });
        tet_options.push(Tetromino {
            shape: vec![vec![false, false], vec![false, false]],
        });
        let tets_dist = Choose::new(&tet_options).unwrap();

        let num_tetrominos: usize = ((DIMX * DIMY) as f32 * wall_coverage / 4.) as usize;

        let tets: Vec<&Tetromino> = tets_dist
            .sample_iter(&mut rand::rng())
            .take(num_tetrominos)
            .collect();

        for idx in 0..num_tetrominos {
            let tet = tets[idx as usize];
            let rotation = rand::random_range(0..=3);

            match rotation {
                0 => {
                    // no rotation
                    let pos = [
                        rand::random_range(0..=DIMX - tet.shape.len()),
                        rand::random_range(0..=DIMY - tet.shape[0].len()),
                    ];
                    for i in 0..tet.shape.len() {
                        for j in 0..tet.shape[0].len() {
                            if tet.shape[i][j] != true {
                                working_map[pos[0] + i][pos[1] + j] = MapItem::Obstacle;
                            }
                        }
                    }
                }
                1 => {
                    // 180 degree rotation
                    let pos = [
                        rand::random_range(0..=DIMX - tet.shape.len()),
                        rand::random_range(0..=DIMY - tet.shape[0].len()),
                    ];
                    for i in 0..tet.shape.len() {
                        for j in 0..tet.shape[0].len() {
                            if tet.shape[i][j] != true {
                                working_map[pos[0] + tet.shape.len() - i - 1]
                                    [pos[1] + tet.shape[0].len() - j - 1] = MapItem::Obstacle;
                            }
                        }
                    }
                }
                2 => {
                    // 90 degree rotation
                    let pos = [
                        rand::random_range(0..=DIMX - tet.shape[0].len()),
                        rand::random_range(0..=DIMY - tet.shape.len()),
                    ];
                    for i in 0..tet.shape.len() {
                        for j in 0..tet.shape[0].len() {
                            if tet.shape[i][j] != true {
                                working_map[pos[0] + tet.shape[0].len() - j - 1][pos[1] + i] =
                                    MapItem::Obstacle;
                            }
                        }
                    }
                }
                3 => {
                    // 270 degree rotation
                    let pos = [
                        rand::random_range(0..=DIMX - tet.shape[0].len()),
                        rand::random_range(0..=DIMY - tet.shape.len()),
                    ];
                    for i in 0..tet.shape.len() {
                        for j in 0..tet.shape[0].len() {
                            if tet.shape[i][j] != true {
                                working_map[pos[0] + j][pos[1] + tet.shape.len() - i - 1] =
                                    MapItem::Obstacle;
                            }
                        }
                    }
                }
                _ => {
                    unreachable!()
                }
            }
        }

        return Ok(working_map);
    }

    /// Add n enemies to the map and return a copy.
    fn add_enemies_to_map(
        &self,
        map_without_enemies: Option<[[MapItem; DIMY]; DIMX]>,
        num_enemies: usize,
    ) -> Result<[[MapItem; DIMY]; DIMX], MapGenerationError> {
        if map_without_enemies.is_none() {
            return Err(MapGenerationError::PrevOpFailed);
        }
        let mut working_map = map_without_enemies.unwrap();
        let mut unoccupied_positions = self.list_unoccupied_positions(working_map);

        for n_enemies_added in 0..num_enemies {
            if !unoccupied_positions.is_empty() {
                let position_idx = rand::rng().random_range(0..unoccupied_positions.len());
                let [pos_x, pos_y] = unoccupied_positions.remove(position_idx);
                working_map[pos_x][pos_y] = MapItem::EnemyEntity;
            } else {
                dbg!(
                    "Map ran out of space in add_enemies_to_map after {} enemies were added.",
                    n_enemies_added
                );
                return Err(MapGenerationError::RanOutOfSpace);
            }
        }
        return Ok(working_map);
    }

    /// Add the hero and the goal to the map at unoccupied positions.
    fn add_hero_and_goal_to_map(
        &self,
        map_without_hero_or_goal: Option<[[MapItem; DIMY]; DIMX]>,
    ) -> Result<[[MapItem; DIMY]; DIMX], MapGenerationError> {
        if map_without_hero_or_goal.is_none() {
            return Err(MapGenerationError::PrevOpFailed);
        }
        let mut working_map = map_without_hero_or_goal.unwrap();

        // yes I know I could just cache this value instead
        let mut unoccupied_positions = self.list_unoccupied_positions(working_map);

        if !unoccupied_positions.is_empty() {
            let position_idx = rand::rng().random_range(0..unoccupied_positions.len());
            let [pos_x, pos_y] = unoccupied_positions.remove(position_idx);
            working_map[pos_x][pos_y] = MapItem::HeroEntity;
        } else {
            dbg!(
                "Map ran out of space in add_hero_and_goal_to_map when adding the hero to the map."
            );
            return Err(MapGenerationError::RanOutOfSpace);
        }

        if !unoccupied_positions.is_empty() {
            let position_idx = rand::rng().random_range(0..unoccupied_positions.len());
            let [pos_x, pos_y] = unoccupied_positions.remove(position_idx);
            working_map[pos_x][pos_y] = MapItem::Goal;
        } else {
            dbg!(
                "Map ran out of space in add_hero_and_goal_to_map when adding the goal to the map."
            );
            return Err(MapGenerationError::RanOutOfSpace);
        }

        return Ok(working_map);
    }

    pub fn list_unoccupied_positions(
        &self,
        map_to_check: [[MapItem; DIMY]; DIMX],
    ) -> Vec<[usize; 2]> {
        let mut unoccupied_positions: Vec<[usize; 2]> = Vec::new();

        for rowidx in 0..DIMX {
            for colidx in 0..DIMY {
                if map_to_check[rowidx][colidx] == MapItem::Empty {
                    unoccupied_positions.push([rowidx, colidx]);
                }
            }
        }

        return unoccupied_positions;
    }

    /// Generate a string that contains the map converted to Unicode.
    pub fn generate_display_string(&self) -> Option<String> {
        let mut display_string = String::new();
        if let Some(map) = self.map {
            for colidx in 0..DIMY {
                for rowidx in 0..DIMX {
                    let map_item = map[rowidx][colidx];
                    let map_char = self.unicode_mappings.get(&map_item);

                    // mappings.insert(MapItem::Empty, ' ');
                    // mappings.insert(MapItem::Obstacle, '\u{2588}'); // unicode solid block
                    // mappings.insert(MapItem::HeroEntity, '\u{2022}'); // unicode bullet
                    // mappings.insert(MapItem::EnemyEntity, '\u{25B2}'); // unicode solid triangle
                    // mappings.insert(MapItem::Goal, '\u{25CE}'); // unicode bullseye
                    // mappings.insert(MapItem::Junk, '\u{2592}'); // medium shaded block

                    if map_char.is_none() {
                        return None;
                    }
                    display_string.push(*map_char.unwrap());
                    if map_char.unwrap() == &'\u{2588}' {
                        display_string.push('\u{2588}')
                    } else if map_char.unwrap() == &'\u{2592}' {
                        display_string.push('\u{2592}')
                    } else {
                        display_string.push(' ');
                    }
                }
                display_string.push('\n');
            }
            return Some(display_string);
        }
        return None;
    }

    pub fn get_empty_neighbors(&self, position: [usize; 2]) -> Vec<[usize; 2]> {
        let mut neighbors: Vec<[usize; 2]> = Vec::new();

        let pos_x = position[0] as i32;
        let pos_y = position[1] as i32;

        for [delta_x, delta_y] in [[-1, 0], [0, -1], [0, 1], [1, 0]] {
            let tx = pos_x + delta_x;
            let ty = pos_y + delta_y;
            if tx < DIMX as i32 && tx >= 0 && ty < DIMY as i32 && ty >= 0 {
                match self
                    .map
                    .expect("Map should exist when doing pathfinding calculations!")
                    [tx as usize][ty as usize]
                {
                    MapItem::Empty => {
                        neighbors.push([tx as usize, ty as usize]);
                    }
                    MapItem::Goal => {
                        neighbors.push([tx as usize, ty as usize]);
                    }
                    MapItem::HeroEntity => {
                        neighbors.push([tx as usize, ty as usize]);
                    }
                    MapItem::EnemyEntity => {
                        neighbors.push([tx as usize, ty as usize]);
                    }
                    _ => {}
                }
            }
        }

        return neighbors;
    }
}
