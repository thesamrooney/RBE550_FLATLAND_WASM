use rand::seq::SliceRandom;
use rand::{Rng, rng};

use crate::active_entity::enemy::get_enemy_action;
use crate::active_entity::hero::get_hero_action;
use crate::map;
use crate::map::MapGenerationError;
use crate::map::MapItem;

pub struct Game<const DIMX: usize, const DIMY: usize> {
    pub game_map: map::Map<DIMX, DIMY>,
    pub game_state: GameState,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GameRunningState {
    NotStarted,
    InProgress,
    HeroVictory,
    HeroFailure,
}

#[derive(Copy, Clone)]
pub struct GameState {
    pub num_steps_run: u32,
    pub running_state: GameRunningState,
    pub hero_teleports_remaining: u32,
}

#[derive(Debug, PartialEq)]
pub enum EntityAction {
    None,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Teleport,
    HonorableSuicide,
}

#[derive(Debug)]
pub struct DisambiguatedEntityAction {
    entity_position: [usize; 2],
    entity_action: EntityAction,
}

impl<const DIMX: usize, const DIMY: usize> Game<DIMX, DIMY> {
    pub fn new(
        wall_coverage: f32,
        num_enemies: usize,
        hero_quantity_teleports: u32,
    ) -> Result<Game<DIMX, DIMY>, MapGenerationError> {
        let mut new_game = Game {
            game_map: map::Map::new(),
            game_state: GameState {
                running_state: GameRunningState::NotStarted,
                num_steps_run: 0,
                hero_teleports_remaining: hero_quantity_teleports,
            },
        };
        let map_gen_attempt = new_game.game_map.generate_map(wall_coverage, num_enemies);
        if map_gen_attempt.is_err() {
            return Err(map_gen_attempt.err().unwrap());
        }
        new_game.game_map.map = map_gen_attempt.ok();
        return Ok(new_game);
    }

    pub fn run_game_iteration(&mut self) {
        let (new_map, new_state) = self.inner_run_game_iteration();
        self.game_map = new_map;
        self.game_state = new_state;
    }

    fn inner_run_game_iteration(&self) -> (map::Map<DIMX, DIMY>, GameState) {
        let requested_actions = self.get_action_requests_from_entities();
        let (new_map, new_state) = self.apply_entity_actions(requested_actions);
        return (new_map, new_state);
    }

    fn get_action_requests_from_entities(&self) -> Vec<DisambiguatedEntityAction> {
        let mut requested_actions: Vec<DisambiguatedEntityAction> = Vec::new();
        if let Some(map) = self.game_map.map {
            for rowidx in 0..DIMX {
                for colidx in 0..DIMY {
                    match map[rowidx][colidx] {
                        MapItem::EnemyEntity => {
                            requested_actions.push(DisambiguatedEntityAction {
                                entity_position: [rowidx, colidx],
                                entity_action: get_enemy_action([rowidx, colidx], &self.game_map),
                            });
                        }
                        MapItem::HeroEntity => {
                            requested_actions.push(DisambiguatedEntityAction {
                                entity_position: [rowidx, colidx],
                                entity_action: get_hero_action(
                                    [rowidx, colidx],
                                    &self.game_map,
                                    &self.game_state,
                                ),
                            });
                        }
                        _ => {
                            continue;
                        }
                    }
                }
            }
        } else {
            // maybe define error types? (map is None if we get here)
            // If we do nothing, this function will just return an empty Vec
            // Honestly, that's fine for this project.
        }
        requested_actions.shuffle(&mut rng()); // ouch spicy
        return requested_actions;
    }

    fn apply_entity_actions(
        &self,
        requested_actions: Vec<DisambiguatedEntityAction>,
    ) -> (map::Map<DIMX, DIMY>, GameState) {
        let mut next_map = self.game_map.clone();
        let mut working_map = next_map
            .map
            .expect("Game map must exist when applying entity actions!");
        let mut next_state = self.game_state;
        next_state.running_state = GameRunningState::InProgress;
        next_state.num_steps_run += 1;
        for disambiguated_action in requested_actions {
            let action = disambiguated_action.entity_action;
            let [pos_x, pos_y] = disambiguated_action.entity_position;
            let entity_type = working_map[pos_x][pos_y];
            if !(entity_type == MapItem::EnemyEntity || entity_type == MapItem::HeroEntity) {
                // If we get here, this entity has been killed or otherwise moved already.
                continue;
            }

            let mut target_position: Option<[i32; 2]> = None;
            match action {
                EntityAction::None => {}
                EntityAction::MoveUp => target_position = Some([pos_x as i32, pos_y as i32 + 1]),
                EntityAction::MoveDown => target_position = Some([pos_x as i32, pos_y as i32 - 1]),
                EntityAction::MoveLeft => target_position = Some([pos_x as i32 - 1, pos_y as i32]),
                EntityAction::MoveRight => target_position = Some([pos_x as i32 + 1, pos_y as i32]),
                EntityAction::Teleport => {
                    if entity_type == MapItem::HeroEntity
                        && self.game_state.hero_teleports_remaining > 0
                    {
                        let unoccupied_positions = self
                            .game_map
                            .list_unoccupied_positions(self.game_map.map.unwrap());
                        let position_idx = rand::rng().random_range(0..unoccupied_positions.len());
                        let [targ_x, targ_y] = unoccupied_positions.get(position_idx).unwrap();
                        target_position = Some([*targ_x as i32, *targ_y as i32]);
                        next_state.hero_teleports_remaining -= 1;
                    }
                }
                EntityAction::HonorableSuicide => {
                    target_position = Some([pos_x as i32, pos_y as i32])
                }
            }
            if let Some([target_x, target_y]) = target_position {
                if target_x > DIMX as i32 || target_y > DIMY as i32 || target_x < 0 || target_y < 0
                {
                    working_map[pos_x][pos_y] = MapItem::Junk;
                    if entity_type == MapItem::HeroEntity {
                        // hero has died: game over
                        next_state.running_state = GameRunningState::HeroFailure;
                    }
                    continue;
                }
                let [tx, ty] = [target_x as usize, target_y as usize];
                match working_map[tx][ty] {
                    MapItem::Empty => {
                        working_map[pos_x][pos_y] = MapItem::Empty;
                        working_map[tx][ty] = entity_type;
                    }
                    MapItem::Obstacle => {
                        working_map[pos_x][pos_y] = MapItem::Junk;
                        if entity_type == MapItem::HeroEntity {
                            next_state.running_state = GameRunningState::HeroFailure;
                        }
                    }
                    MapItem::Junk => {
                        working_map[pos_x][pos_y] = MapItem::Junk;
                        if entity_type == MapItem::HeroEntity {
                            next_state.running_state = GameRunningState::HeroFailure;
                        }
                    }
                    MapItem::EnemyEntity => {
                        working_map[pos_x][pos_y] = MapItem::Junk;
                        working_map[tx][ty] = MapItem::Junk;
                        if entity_type == MapItem::HeroEntity {
                            next_state.running_state = GameRunningState::HeroFailure;
                        }
                    }
                    MapItem::Goal => {
                        if entity_type == MapItem::HeroEntity {
                            next_state.running_state = GameRunningState::HeroVictory;
                            working_map[pos_x][pos_y] = MapItem::Empty;
                        }
                    }
                    MapItem::HeroEntity => {
                        working_map[pos_x][pos_y] = MapItem::Junk;
                        working_map[tx][ty] = MapItem::Junk;
                        next_state.running_state = GameRunningState::HeroFailure;
                    }
                }
            }
        }
        next_map.map = Some(working_map);
        return (next_map, next_state);
    }

    /// Print the game state to the terminal
    pub fn print_game_state(&self) {
        println!("{}", self.generate_game_string());
    }

    /// Add a fancy box around the game map and return it as a string
    fn generate_game_string(&self) -> String {
        let mut game_display_string: String = String::new();
        game_display_string.push('\u{250C}'); // light top-left corner
        game_display_string.push_str(String::from('\u{2500}').repeat(DIMY * 2).as_str()); // light horizontal line
        game_display_string.push_str("\u{2510}\r\n"); // light top-right corner
        if let Some(map_string) = self.game_map.generate_display_string() {
            for line in map_string.lines() {
                game_display_string.push('\u{2502}'); // light vertical line
                game_display_string.push_str(line);
                game_display_string.push('\u{2502}'); // light vertical line
                game_display_string.push_str("\r\n");
            }
        }
        game_display_string.push('\u{2514}'); // light bottom-left corner
        game_display_string.push_str(String::from('\u{2500}').repeat(DIMY * 2).as_str()); // light horizontal lines
        game_display_string.push_str("\u{2518}\r\n"); // light bottom-right corner
        return game_display_string;
    }
}
