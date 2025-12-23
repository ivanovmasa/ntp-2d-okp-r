use macroquad::prelude::*;

mod genetic;  
pub use genetic::genetic_algorithm;

mod util; 
mod max_rects;
mod skyline;
mod guillotine;
mod ui;
mod menu_state;

pub use util::Rect;
use menu_state::AppState;

pub struct Problem {
    pub bin_width: i32,
    pub bin_height: i32,
    pub rectangles: Vec<Rect>, 
}

#[derive(Clone, Copy, PartialEq)]
pub enum Heuristic {
    MaxRects,
    Skyline,
    Guillotine,
}

impl Heuristic {
    pub fn name(&self) -> &str {
        match self {
            Heuristic::MaxRects => "MaxRects (Best Area Fit)",
            Heuristic::Skyline => "Skyline (Bottom-Left)",
            Heuristic::Guillotine => "Guillotine (Best Short Side)",
        }
    }
    
    pub fn decode_chromosome(&self, chromosome: &[u8], problem: &Problem) -> (Vec<Rect>, f32) {
        match self {
            Heuristic::MaxRects => max_rects::decode_chromosome(chromosome, problem),
            Heuristic::Skyline => skyline::decode_chromosome(chromosome, problem),
            Heuristic::Guillotine => guillotine::decode_chromosome(chromosome, problem),
        }
    }
}

fn window_config() -> Conf {
    Conf {
        window_title: "2D-OKP-R Genetic Algorithm".to_owned(),
        window_width: 1000, 
        window_height: 800,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_config)]
async fn main() {
    let mut app_state = AppState::new();
    
    loop {
        clear_background(WHITE);
        
        app_state.render();
        
        next_frame().await;
    }
}
