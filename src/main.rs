#![recursion_limit = "128"]

use std::{thread, time::Duration};

use console_error_panic_hook::set_once as set_panic_hook;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, Element, HtmlCanvasElement, js_sys::Function, window};

use crate::{
    game::{Game, GameRunningState},
    map::MapItem,
};

mod active_entity;
mod game;
mod map;

#[wasm_bindgen]
struct WasmGame {
    game: Game<64, 64>,
    canvas: Option<HtmlCanvasElement>,
}

#[wasm_bindgen]
impl WasmGame {
    pub fn new(fill_ratio: f32, num_enemies: usize, num_teleports: u32) -> WasmGame {
        let window = window().expect("Could not access window.");
        let document = window.document().expect("Could not access document.");
        let body = document.body().expect("Could not access document.body.");
        let canvas = document
            .get_element_by_id("flatland_canvas")
            .expect("Canvas should exist!");
        return WasmGame {
            game: Game::new(fill_ratio, num_enemies, num_teleports)
                .expect("Game should generate properly"),
            canvas: Some(canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap()),
        };
    }

    pub fn is_game_over(&self) -> bool {
        match self.game.game_state.running_state {
            GameRunningState::NotStarted => {
                return false;
            }
            GameRunningState::InProgress => {
                return false;
            }
            GameRunningState::HeroVictory => {
                return true;
            }
            GameRunningState::HeroFailure => {
                return true;
            }
        }
    }

    pub fn update_flatland(&mut self) {
        self.game.run_game_iteration();
    }

    pub fn render(&self) {
        let canvas: &HtmlCanvasElement = self.canvas.as_ref().expect("Canvas should exist");
        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .expect("Browser should support 2D canvas!")
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        ctx.set_fill_style_str("white");
        ctx.fill_rect(0., 0., 640., 640.);
        let cell_square_dim_px = 10.;
        for row_idx in 0..64 {
            for col_idx in 0..64 {
                let mapitem: MapItem = self.game.game_map.map.unwrap()[row_idx][col_idx];
                match mapitem {
                    MapItem::Empty => {
                        ctx.set_fill_style_str("#FFFFFF");
                        ctx.fill_rect(
                            col_idx as f64 * cell_square_dim_px,
                            row_idx as f64 * cell_square_dim_px,
                            cell_square_dim_px,
                            cell_square_dim_px,
                        );
                    }
                    MapItem::Obstacle => {
                        ctx.set_fill_style_str("#000000");
                        ctx.fill_rect(
                            col_idx as f64 * cell_square_dim_px,
                            row_idx as f64 * cell_square_dim_px,
                            cell_square_dim_px,
                            cell_square_dim_px,
                        );
                    }
                    MapItem::Junk => {
                        ctx.set_fill_style_str("#888888");
                        ctx.fill_rect(
                            col_idx as f64 * cell_square_dim_px,
                            row_idx as f64 * cell_square_dim_px,
                            cell_square_dim_px,
                            cell_square_dim_px,
                        );
                    }
                    MapItem::EnemyEntity => {
                        ctx.set_font("10px sans-serif");
                        let x_pos = col_idx as f64 * cell_square_dim_px;
                        let y_pos = row_idx as f64 * cell_square_dim_px;
                        ctx.set_fill_style_str("#FFFFFF");
                        ctx.fill_rect(x_pos, y_pos, cell_square_dim_px, cell_square_dim_px);
                        ctx.set_fill_style_str("#FF0000");
                        let _ = ctx.fill_text(&"\u{25B2}", x_pos, y_pos + 8.);
                    }
                    MapItem::HeroEntity => {
                        ctx.set_font("8px sans-serif");
                        let x_pos = col_idx as f64 * cell_square_dim_px;
                        let y_pos = row_idx as f64 * cell_square_dim_px;
                        ctx.set_fill_style_str("#FFFFFF");
                        ctx.fill_rect(x_pos, y_pos, cell_square_dim_px, cell_square_dim_px);
                        ctx.set_fill_style_str("#0000FF");
                        let _ = ctx.fill_text("\u{2B24}", x_pos, y_pos + 8.);
                    }
                    MapItem::Goal => {
                        ctx.set_fill_style_str("#00FF00");
                        ctx.fill_rect(
                            col_idx as f64 * cell_square_dim_px,
                            row_idx as f64 * cell_square_dim_px,
                            cell_square_dim_px,
                            cell_square_dim_px,
                        );
                    }
                }
            }
        }
    }

    // pub fn render(&self) {
    //     let window = window().expect("Could not access window.");
    //     let document = window.document().expect("Could not access document.");
    //     for row_idx in 0..64 {
    //         for col_idx in 0..64 {
    //             let cell = document
    //                 .get_element_by_id(format!("cell{}x{}", row_idx, col_idx).as_str())
    //                 .expect("Cell should exist!");
    //             match self.game.game_map.map.expect("Map should exist!")[row_idx][col_idx] {
    //                 map::MapItem::Empty => {
    //                     cell.set_inner_html("");
    //                     let _ = cell.set_attribute(
    //                         "style",
    //                         "background-color: white; width: 10px; height: 10px;",
    //                     );
    //                 }
    //                 map::MapItem::Junk => {
    //                     cell.set_inner_html("");
    //                     let _ = cell.set_attribute(
    //                         "style",
    //                         "background-color: grey; width: 10px; height: 10px;",
    //                     );
    //                 }
    //                 map::MapItem::Obstacle => {
    //                     cell.set_inner_html("");
    //                     let _ = cell.set_attribute(
    //                         "style",
    //                         "background-color: black; width: 10px; height: 10px;",
    //                     );
    //                 }
    //                 map::MapItem::Goal => {
    //                     cell.set_inner_html("");
    //                     let _ = cell.set_attribute(
    //                         "style",
    //                         "background-color: lightgreen; width: 10px; height: 10px;",
    //                     );
    //                 }
    //                 map::MapItem::HeroEntity => {
    //                     cell.set_inner_html("\u{2B24}");
    //                     let _ = cell.set_attribute(
    //                         "style",
    //                         "background-color: white; color: blue; overflow: hidden; width: 10px; height: 10px;",
    //                     );
    //                 }
    //                 map::MapItem::EnemyEntity => {
    //                     cell.set_inner_html("\u{25B2}");
    //                     let _ = cell.set_attribute(
    //                         "style",
    //                         "background-color: white; font-size: 10px; color: red; overflow: hidden; width: 10px; height: 10px;",
    //                     );
    //                 }
    //             }
    //         }
    //     }
    // }
}

fn main() {
    set_panic_hook();
}
