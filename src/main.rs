#![recursion_limit = "128"]

use std::{thread, time::Duration};

use console_error_panic_hook::set_once as set_panic_hook;
use wasm_bindgen::prelude::*;
use web_sys::{Element, js_sys::Function, window};

use crate::game::Game;

mod active_entity;
mod game;
mod map;

struct WasmGame<const DIMX: usize, const DIMY: usize> {
    game: Game<DIMX, DIMY>,
    container: Element,
}

impl<const DIMX: usize, const DIMY: usize> WasmGame<DIMX, DIMY> {
    pub fn new(fill_ratio: f32, num_enemies: usize, num_teleports: u32) -> WasmGame<DIMX, DIMY> {
        let window = window().expect("Could not access window.");
        let document = window.document().expect("Could not access document.");
        let body = document.body().expect("Could not access document.body.");

        let container = document
            .create_element("div")
            .expect("Could not create container.");
        container.set_id("flatland_container");
        body.append_child(&container)
            .expect("Could not append container to body!");

        for row_idx in 0..DIMX {
            for col_idx in 0..DIMY {
                let grid_item = document
                    .create_element("span")
                    .expect("Could not create grid item!");
                let _ = grid_item.set_id(format!("cell{}x{}", row_idx, col_idx).as_str());
                let _ = grid_item.set_class_name("flatland_cell");
                let _ = grid_item.set_inner_html("@");
                container
                    .append_child(&grid_item)
                    .expect("Could not append grid item to container!");
            }
        }

        return WasmGame {
            game: Game::new(fill_ratio, num_enemies, num_teleports)
                .expect("Game should generate properly"),
            container: container,
        };
    }

    pub fn update_flatland(&mut self) {
        let window = window().expect("Could not access window.");
        let document = window.document().expect("Could not access document.");
        self.game.run_game_iteration();
        // self.game.print_game_state();
        for row_idx in 0..DIMX {
            for col_idx in 0..DIMY {
                let cell = document
                    .get_element_by_id(format!("cell{}x{}", row_idx, col_idx).as_str())
                    .expect("Cell should exist!");
                match self.game.game_map.map.expect("Map should exist!")[row_idx][col_idx] {
                    map::MapItem::Empty => {
                        cell.set_inner_html("");
                        let _ = cell.set_attribute(
                            "style",
                            "background-color: white; width: 10px; height: 10px;",
                        );
                    }
                    map::MapItem::Junk => {
                        cell.set_inner_html("");
                        let _ = cell.set_attribute(
                            "style",
                            "background-color: grey; width: 10px; height: 10px;",
                        );
                    }
                    map::MapItem::Obstacle => {
                        cell.set_inner_html("");
                        let _ = cell.set_attribute(
                            "style",
                            "background-color: black; width: 10px; height: 10px;",
                        );
                    }
                    map::MapItem::Goal => {
                        cell.set_inner_html("");
                        let _ = cell.set_attribute(
                            "style",
                            "background-color: lightgreen; width: 10px; height: 10px;",
                        );
                    }
                    map::MapItem::HeroEntity => {
                        cell.set_inner_html("\u{2B24}");
                        let _ = cell.set_attribute(
                            "style",
                            "background-color: white; color: blue; overflow: hidden; width: 10px; height: 10px;",
                        );
                    }
                    map::MapItem::EnemyEntity => {
                        cell.set_inner_html("\u{25B2}");
                        let _ = cell.set_attribute(
                            "style",
                            "background-color: white; font-size: 10px; color: red; overflow: hidden; width: 10px; height: 10px;",
                        );
                    }
                }
            }
        }
    }
}

fn main() {
    set_panic_hook();
    let window = window().expect("Could not access window.");
    let document = window.document().expect("Could not access document.");
    let mut wasm_game: WasmGame<64, 64> = WasmGame::new(0.25, 10, 5);
    loop {
        wasm_game.update_flatland();
        match wasm_game.game.game_state.running_state {
            game::GameRunningState::HeroVictory => {
                let end_message =
                    document.create_text_node("The Hero emerged victorious! Refresh to run again.");
                let _ = document.append_child(&end_message);

                break;
            }
            game::GameRunningState::HeroFailure => {
                let end_message =
                    document.create_text_node("The Hero fell in battle! Refresh to run again.");
                let _ = document.append_child(&end_message);
                break;
            }
            _ => {}
        }
    }
}
