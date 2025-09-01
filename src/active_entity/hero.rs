use crate::game::{EntityAction, GameState};
use crate::map::{Map, MapItem};
use rand::prelude::SliceRandom;
use std::cmp::min;
use std::collections::VecDeque;

#[derive(PartialEq, PartialOrd)]
struct ComparableMapNode {
    node: [usize; 2],
    est_cost: f32,
}

pub fn get_hero_action<const DIMX: usize, const DIMY: usize>(
    position: [usize; 2],
    map: &Map<DIMX, DIMY>,
    game_state: &GameState,
) -> EntityAction {
    let action = get_pathfinding_action(position, game_state, map);

    return action;
}

fn find_goal<const DIMX: usize, const DIMY: usize>(
    map: [[MapItem; DIMY]; DIMX],
) -> Option<[usize; 2]> {
    for pos_x in 0..DIMX {
        for pos_y in 0..DIMY {
            if map[pos_x][pos_y] == MapItem::Goal {
                return Some([pos_x, pos_y]);
            }
        }
    }
    return None;
}

fn find_enemies<const DIMX: usize, const DIMY: usize>(
    map: [[MapItem; DIMY]; DIMX],
) -> Vec<[usize; 2]> {
    let mut enemies = Vec::new();
    for pos_x in 0..DIMX {
        for pos_y in 0..DIMY {
            if map[pos_x][pos_y] == MapItem::EnemyEntity {
                enemies.push([pos_x, pos_y]);
            }
        }
    }
    return enemies;
}

fn precompute_distance_to_goal<const DIMX: usize, const DIMY: usize>(
    map: &Map<DIMX, DIMY>,
) -> [[Option<u32>; DIMY]; DIMX] {
    let mut distance_map: [[Option<u32>; DIMY]; DIMX] = [[None; DIMY]; DIMX];
    let goal_pos = find_goal(map.map.expect("The map must be initialized!"))
        .expect("The map must contain a goal!");
    let mut frontier: VecDeque<[usize; 2]> = VecDeque::new();
    for neighbor in map.get_empty_neighbors(goal_pos) {
        frontier.push_back(neighbor);
    }
    distance_map[goal_pos[0]][goal_pos[1]] = Some(0);
    loop {
        if let Some([pos_x, pos_y]) = frontier.pop_front() {
            let neighbors = map.get_empty_neighbors([pos_x, pos_y]);
            let mut lowest_neighbor_value: u32 = u32::MAX;
            let mut lowest_neighbor: Option<[usize; 2]> = None;
            for [n_x, n_y] in &neighbors {
                if let Some(neighbor_val) = distance_map[*n_x][*n_y] {
                    if neighbor_val < lowest_neighbor_value {
                        lowest_neighbor_value = neighbor_val;
                        lowest_neighbor = Some([*n_x, *n_y]);
                    }
                } else if !frontier.contains(&[*n_x, *n_y]) {
                    frontier.push_back([*n_x, *n_y]);
                }
            }
            if lowest_neighbor.is_some() {
                distance_map[pos_x][pos_y] = Some(lowest_neighbor_value + 1);
            } else {
                dbg!(
                    goal_pos,
                    [pos_x, pos_y],
                    &neighbors,
                    // distance_map,
                    lowest_neighbor_value,
                    lowest_neighbor
                );
                for [n_x, n_y] in &neighbors {
                    dbg!(distance_map[*n_x][*n_y]);
                }
                panic!("how did we get here");
            }
        } else {
            break;
        }
    }

    return distance_map;
}

fn precompute_distance_to_enemy<const DIMX: usize, const DIMY: usize>(
    map: &Map<DIMX, DIMY>,
) -> [[Option<u32>; DIMY]; DIMX] {
    let mut distance_map: [[Option<u32>; DIMY]; DIMX] = [[None; DIMY]; DIMX];
    let enemy_positions = find_enemies(map.map.expect("The map must be initialized!"));
    let mut frontier: VecDeque<[usize; 2]> = VecDeque::new();
    for enemy in &enemy_positions {
        distance_map[enemy[0]][enemy[1]] = Some(0);
        for neighbor in map.get_empty_neighbors(*enemy) {
            if distance_map[neighbor[0]][neighbor[1]].is_none() && !frontier.contains(&neighbor) {
                frontier.push_back(neighbor);
            }
        }
    }
    loop {
        if let Some([pos_x, pos_y]) = frontier.pop_front() {
            let neighbors = map.get_empty_neighbors([pos_x, pos_y]);
            let mut lowest_neighbor_value: u32 = u32::MAX;
            let mut lowest_neighbor: Option<[usize; 2]> = None;
            for [n_x, n_y] in &neighbors {
                if let Some(neighbor_val) = distance_map[*n_x][*n_y] {
                    if neighbor_val < lowest_neighbor_value {
                        lowest_neighbor_value = neighbor_val;
                        lowest_neighbor = Some([*n_x, *n_y]);
                    }
                } else if !frontier.contains(&[*n_x, *n_y]) {
                    frontier.push_back([*n_x, *n_y]);
                }
            }
            if lowest_neighbor.is_some() {
                distance_map[pos_x][pos_y] = Some(lowest_neighbor_value + 1);
            } else {
                // Similar to the original, but adapted
                dbg!(
                    neighbors,
                    enemy_positions,
                    lowest_neighbor_value,
                    lowest_neighbor
                );
                panic!("No valid neighbor found for enemy distance computation");
            }
        } else {
            break;
        }
    }

    return distance_map;
}

fn print_distance_map<const DIMX: usize, const DIMY: usize>(
    distance_map: [[Option<u32>; DIMY]; DIMX],
) {
    let mut display_string = String::new();
    for colidx in 0..DIMY {
        for rowidx in 0..DIMX {
            let map_item = distance_map[rowidx][colidx];
            if map_item.is_some() {
                let val = map_item.unwrap();
                let val_str = format!("{val:x}");
                if val_str.len() == 1 {
                    display_string.push_str(&val_str);
                    display_string.push(' ');
                } else {
                    display_string.push_str(&val_str);
                }
            } else {
                display_string.push_str(&"  ");
            }
        }
        display_string.push('\n');
    }
    println!("{}", display_string);
}

fn get_pathfinding_action<const DIMX: usize, const DIMY: usize>(
    starting_position: [usize; 2],
    game_state: &GameState,
    map: &Map<DIMX, DIMY>,
) -> EntityAction {
    let mut frontier: VecDeque<ComparableMapNode> = VecDeque::with_capacity(DIMX * DIMY);
    frontier.push_back(ComparableMapNode {
        node: starting_position,
        est_cost: 0.,
    });
    let goal_distance_map = precompute_distance_to_goal(map);
    // println!("PRINTING GOAL DISTANCE MAP");
    // print_distance_map(goal_distance_map);

    let enemy_distance_map = precompute_distance_to_enemy(map);
    // println!("PRINTING ENEMY DISTANCE MAP");
    // print_distance_map(enemy_distance_map);

    let fear_of_enemy_falloff: f64 = 6. / (game_state.num_steps_run as f64 / 50.);

    let [pos_x, pos_y] = starting_position;
    if enemy_distance_map[pos_x][pos_y].unwrap_or(u32::MAX) <= 2 {
        return EntityAction::Teleport;
    }
    if goal_distance_map[pos_x][pos_y] == None
        && (enemy_distance_map[pos_x][pos_y] == None || fear_of_enemy_falloff <= 3.)
    {
        if game_state.hero_teleports_remaining != 0 {
            return EntityAction::Teleport;
        } else {
            return EntityAction::HonorableSuicide;
        }
    }

    let mut least_neighbor = None;
    let mut least_neighbor_value: f64 = u32::MAX as f64;
    let mut neighbors = map.get_empty_neighbors(starting_position);
    neighbors.shuffle(&mut rand::rng());
    for [n_x, n_y] in neighbors {
        let neighbor_value: f64 = f64::sqrt(goal_distance_map[n_x][n_y].unwrap_or(u32::MAX) as f64)
            - f64::min(
                fear_of_enemy_falloff,
                enemy_distance_map[n_x][n_y].unwrap_or(1) as f64,
            );
        if neighbor_value < least_neighbor_value {
            least_neighbor_value = neighbor_value;
            least_neighbor = Some([n_x, n_y]);
        }
    }

    if let Some([n_x, n_y]) = least_neighbor {
        let [p_x, p_y] = starting_position;
        let (d_x, d_y) = (n_x as i32 - p_x as i32, n_y as i32 - p_y as i32);
        if d_x < 0 {
            return EntityAction::MoveLeft;
        };
        if d_x > 0 {
            return EntityAction::MoveRight;
        };
        if d_y < 0 {
            return EntityAction::MoveDown;
        };
        if d_y > 0 {
            return EntityAction::MoveUp;
        };
    }
    return EntityAction::None;
}
