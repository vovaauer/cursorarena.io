use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use game_logic::{Game as GameLogic, MapData, PlayerInput};

#[wasm_bindgen]
pub struct Game(GameLogic);

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(map_data_js: &JsValue) -> Self {
        let map_data: Option<MapData> = if map_data_js.is_null() || map_data_js.is_undefined() {
            None
        } else {
            serde_wasm_bindgen::from_value(map_data_js.clone()).ok()
        };
        let mut game = GameLogic::new(map_data);
        game.add_player(0); // Add a default player for local game
        Self(game)
    }

    pub fn tick(&mut self, mouse_dx: f32, mouse_dy: f32, is_mouse_down: bool) {
        let input = PlayerInput {
            mouse_dx,
            mouse_dy,
            is_mouse_down,
        };
        self.0.apply_input(0, input);
        self.0.tick();
    }

    pub fn get_game_state(&self) -> String {
        let game_state = self.0.get_game_state();
        serde_json::to_string(&game_state).unwrap()
    }

    #[wasm_bindgen]
    pub fn pause(&mut self) {
        self.0.pause();
    }

    #[wasm_bindgen]
    pub fn restart(&mut self) {
        self.0.restart();
    }
}