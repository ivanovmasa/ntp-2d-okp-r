use macroquad::prelude::*;
use ::rand::rng;
use std::time::Instant;
use serde_json::Value;
use std::fs;

mod genetic;  
use genetic::genetic_algorithm;

mod util; 
mod max_rects;
mod skyline;
mod guillotine;
use util::Rect;

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
    fn name(&self) -> &str {
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
struct Button {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    text: String,
    color: Color,
    hover_color: Color,
}

impl Button {
    fn new(x: f32, y: f32, width: f32, height: f32, text: &str, color: Color) -> Self {
        Self {
            x,
            y,
            width,
            height,
            text: text.to_string(),
            color,
            hover_color: Color::new(
                color.r * 1.2,
                color.g * 1.2,
                color.b * 1.2,
                color.a
            ),
        }
    }
    
    fn is_hovered(&self) -> bool {
        let (mouse_x, mouse_y) = mouse_position();
        mouse_x >= self.x && mouse_x <= self.x + self.width &&
        mouse_y >= self.y && mouse_y <= self.y + self.height
    }
    
    fn is_clicked(&self) -> bool {
        self.is_hovered() && is_mouse_button_pressed(MouseButton::Left)
    }
    
    fn draw(&self) {
        let color = if self.is_hovered() {
            self.hover_color
        } else {
            self.color
        };
        
        draw_rectangle(self.x, self.y, self.width, self.height, color);
        draw_rectangle_lines(self.x, self.y, self.width, self.height, 2.0, BLACK);
        
        let text_size = 20.0;
        let text_dims = measure_text(&self.text, None, text_size as u16, 1.0);
        let text_x = self.x + (self.width - text_dims.width) / 2.0;
        let text_y = self.y + (self.height + text_size) / 2.0 - 5.0;
        
        draw_text(&self.text, text_x, text_y, text_size, WHITE);
    }
}

// Text input field
struct TextField {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    text: String,
    is_active: bool,
    placeholder: String,
}

impl TextField {
    fn new(x: f32, y: f32, width: f32, height: f32, placeholder: &str) -> Self {
        Self {
            x,
            y,
            width,
            height,
            text: String::new(),
            is_active: false,
            placeholder: placeholder.to_string(),
        }
    }
    
    fn is_clicked(&self) -> bool {
        let (mouse_x, mouse_y) = mouse_position();
        is_mouse_button_pressed(MouseButton::Left) &&
        mouse_x >= self.x && mouse_x <= self.x + self.width &&
        mouse_y >= self.y && mouse_y <= self.y + self.height
    }
    
    fn update(&mut self) {
        if self.is_clicked() {
            self.is_active = true;
        } else if is_mouse_button_pressed(MouseButton::Left) {
            self.is_active = false;
        }
        
        if self.is_active {
            // Handle text input
            if let Some(key) = get_last_key_pressed() {
                match key {
                    KeyCode::Backspace => {
                        self.text.pop();
                    }
                    KeyCode::Enter | KeyCode::Tab => {
                        self.is_active = false;
                    }
                    _ => {}
                }
            }
            
            // Get character input
            if let Some(character) = get_char_pressed() {
                if character.is_ascii_digit() || character == ' ' {
                    self.text.push(character);
                }
            }
        }
    }
    
    fn draw(&self) {
        let bg_color = if self.is_active { WHITE } else { LIGHTGRAY };
        let border_color = if self.is_active { BLUE } else { BLACK };
        
        draw_rectangle(self.x, self.y, self.width, self.height, bg_color);
        draw_rectangle_lines(self.x, self.y, self.width, self.height, 2.0, border_color);
        
        let display_text = if self.text.is_empty() && !self.is_active {
            &self.placeholder
        } else {
            &self.text
        };
        
        let text_color = if self.text.is_empty() && !self.is_active {
            GRAY
        } else {
            BLACK
        };
        
        draw_text(display_text, self.x + 10.0, self.y + 25.0, 20.0, text_color);
    }
    
    fn parse_i32(&self) -> Option<i32> {
        self.text.trim().parse().ok()
    }
}

// Menu state
#[derive(PartialEq)]
enum MenuState {
    MainMenu,
    HeuristicSelection,
    JsonSelection,
    ManualInput,
    Solution,
    Comparison,
}

struct HeuristicResult {
    heuristic: Heuristic,
    fitness: f32,
    time_seconds: f64,
    placed_rects: Vec<Rect>,
}


fn load_problem_from_json(file_num: usize) -> Result<Problem, Box<dyn std::error::Error>> {
    let file_path = format!("json/{}.json", file_num);
    let json_content = fs::read_to_string(&file_path)?;
    let data: Value = serde_json::from_str(&json_content)?;
    let bin_width = data["Objects"][0]["Length"].as_i64().unwrap() as i32;
    let bin_height = data["Objects"][0]["Height"].as_i64().unwrap() as i32;
    
    let mut rectangles = Vec::new();
    if let Some(items) = data["Items"].as_array() {
        for item in items {
            let width = item["Length"].as_i64().unwrap() as i32;
            let height = item["Height"].as_i64().unwrap() as i32;
            let demand = item["Demand"].as_i64().unwrap_or(1) as i32;

            for _ in 0..demand {
                rectangles.push(Rect::new_unplaced(width, height));
            }
        }
    }
    
    Ok(Problem {
        bin_width,
        bin_height,
        rectangles,
    })
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
    let mut menu_state = MenuState::MainMenu;
    let mut selected_heuristic = Heuristic::MaxRects;
    let mut selected_json: usize = 1;
    let mut problem: Option<Problem> = None;
    let mut placed_rects: Vec<Rect> = Vec::new();
    let mut best_fitness = 0.0;
    let mut comparison_results: Vec<HeuristicResult> = Vec::new();
    
    // Manual input fields
    let mut bin_width_field = TextField::new(300.0, 200.0, 200.0, 40.0, "Bin width");
    let mut bin_height_field = TextField::new(300.0, 250.0, 200.0, 40.0, "Bin height");
    let mut num_rects_field = TextField::new(100.0, 220.0, 200.0, 40.0, "Number of rectangles");
    let mut manual_rects: Vec<(TextField, TextField)> = Vec::new();
    let mut scroll_offset = 0.0;
    
    loop {
        clear_background(Color::new(0.9, 0.9, 0.95, 1.0));
        
        match menu_state {
            MenuState::MainMenu => {
                // Title
                draw_text("2D-OKP-R Solver", 375.0, 100.0, 40.0, BLACK);
                draw_text("Choose input method:", 380.0, 180.0, 25.0, DARKGRAY);
                
                // Buttons
                let json_button = Button::new(350.0, 250.0, 300.0, 60.0, "Load from JSON", DARKBLUE);
                let manual_button = Button::new(350.0, 330.0, 300.0, 60.0, "Manual Input", DARKGREEN);
                
                json_button.draw();
                manual_button.draw();
                
                if json_button.is_clicked() {
                    menu_state = MenuState::HeuristicSelection;
                }
                
                if manual_button.is_clicked() {
                    menu_state = MenuState::ManualInput;
                    manual_rects.clear();
                    bin_width_field.text.clear();
                    bin_height_field.text.clear();
                    num_rects_field.text.clear();
                    scroll_offset = 0.0;
                }
            }
            
            MenuState::HeuristicSelection => {
                draw_text("Select Heuristic Algorithm", 320.0, 80.0, 30.0, BLACK);
                
                let start_y = 200.0;
                let button_height = 70.0;
                let spacing = 20.0;
                
                let heuristics = [
                    (Heuristic::MaxRects, DARKBLUE),
                    (Heuristic::Skyline, DARKGREEN),
                    (Heuristic::Guillotine, PURPLE),
                ];
                
                for (i, (heuristic, color)) in heuristics.iter().enumerate() {
                    let y = start_y + i as f32 * (button_height + spacing);
                    let is_selected = selected_heuristic == *heuristic;
                    let button_color = if is_selected { *color } else { DARKGRAY };
                    let button = Button::new(250.0, y, 500.0, button_height, heuristic.name(), button_color);
                    button.draw();
                    
                    if button.is_clicked() {
                        selected_heuristic = *heuristic;
                    }
                }
                
                // Next and Back buttons
                let next_button = Button::new(250.0, 720.0, 180.0, 50.0, "Next", GREEN);
                let back_button = Button::new(570.0, 720.0, 180.0, 50.0, "Back", RED);
                
                next_button.draw();
                back_button.draw();
                
                draw_text(&format!("Selected: {}", selected_heuristic.name()), 280.0, 120.0, 20.0, BLUE);
                
                if next_button.is_clicked() {
                    menu_state = MenuState::JsonSelection;
                }
                
                if back_button.is_clicked() {
                    menu_state = MenuState::MainMenu;
                }
            }
            
            MenuState::JsonSelection => {
                draw_text("Select JSON File (1-13)", 350.0, 80.0, 30.0, BLACK);
                
                // Two-column grid layout
                let start_x = 250.0;
                let start_y = 150.0;
                let button_width = 200.0;
                let button_height = 60.0;
                let spacing_x = 20.0;
                let spacing_y = 15.0;
                let columns = 2;
                
                for i in 1..=13 {
                    let col = (i - 1) % columns;
                    let row = (i - 1) / columns;
                    
                    let x = start_x + col as f32 * (button_width + spacing_x);
                    let y = start_y + row as f32 * (button_height + spacing_y);
                    
                    let color = if selected_json == i { BLUE } else { DARKGRAY };
                    let button = Button::new(x, y, button_width, button_height, &format!("Problem {}", i), color);
                    button.draw();
                    
                    if button.is_clicked() {
                        selected_json = i;
                    }
                }
                
                // Solve and Back buttons at bottom
                let solve_button = Button::new(250.0, 720.0, 180.0, 50.0, "Solve", GREEN);
                let back_button = Button::new(570.0, 720.0, 180.0, 50.0, "Back", RED);
                
                solve_button.draw();
                back_button.draw();
                
                // Highlight selected problem
                draw_text(&format!("Selected: Problem {}", selected_json), 350.0, 120.0, 20.0, BLUE);
                
                if solve_button.is_clicked() {
                    match load_problem_from_json(selected_json) {
                        Ok(p) => {
                            println!("Loaded problem from json/{}.json", selected_json);
                            println!("Bin: {}x{}", p.bin_width, p.bin_height);
                            println!("Rectangles: {}", p.rectangles.len());
                            println!("Heuristic: {}", selected_heuristic.name());
                            
                            let mut rng = rng();
                            let start = Instant::now();
                            let (best_chromosome, fitness) = genetic_algorithm(
                                &p,
                                100,
                                0.1,
                                0.1,
                                200,
                                &mut rng,
                                selected_heuristic,
                            );
                            
                            println!("GA took: {:.2}s", start.elapsed().as_secs_f64());
                            println!("Fitness: {:.2}%", fitness * 100.0);
                            
                            let (rects, _) = selected_heuristic.decode_chromosome(&best_chromosome, &p);
                            placed_rects = rects;
                            best_fitness = fitness;
                            problem = Some(p);
                            menu_state = MenuState::Solution;
                        }
                        Err(e) => {
                            eprintln!("Error loading json/{}.json: {}", selected_json, e);
                        }
                    }
                }
                
                if back_button.is_clicked() {
                    menu_state = MenuState::MainMenu;
                }
            }
            
            MenuState::ManualInput => {
                draw_text("Manual Input", 400.0, 50.0, 30.0, BLACK);
                
                // Check for Tab key press
                let tab_pressed = is_key_pressed(KeyCode::Tab);
                
                // Bin size inputs (always visible)
                draw_text("Bin dimensions:", 100.0, 110.0, 20.0, BLACK);
                
                bin_width_field.x = 100.0;
                bin_width_field.y = 120.0;
                let bin_width_was_active = bin_width_field.is_active && tab_pressed;
                bin_width_field.update();
                bin_width_field.draw();
                
                bin_height_field.x = 320.0;
                bin_height_field.y = 120.0;
                let bin_height_was_active = bin_height_field.is_active && tab_pressed;
                bin_height_field.update();
                bin_height_field.draw();
                
                draw_text("W", 260.0, 145.0, 18.0, GRAY);
                draw_text("H", 480.0, 145.0, 18.0, GRAY);
                
                // Number of rectangles
                draw_text("Number of rectangles:", 100.0, 210.0, 20.0, BLACK);
                let num_rects_was_active = num_rects_field.is_active && tab_pressed;
                num_rects_field.update();
                num_rects_field.draw();
                
                // Update rectangle count
                if let Some(count) = num_rects_field.parse_i32() {
                    let count = count.max(0).min(50) as usize; // Limit to 50 rectangles
                    while manual_rects.len() < count {
                        manual_rects.push((
                            TextField::new(0.0, 0.0, 140.0, 35.0, "Width"),
                            TextField::new(0.0, 0.0, 140.0, 35.0, "Height"),
                        ));
                    }
                    while manual_rects.len() > count {
                        manual_rects.pop();
                    }
                }
                
                // Apply tab navigation for top fields after rectangles are created
                if tab_pressed {
                    // Check if any rectangle field is active
                    let any_rect_active = manual_rects.iter().any(|(w, h)| w.is_active || h.is_active);
                    
                    if !any_rect_active {
                        if bin_width_was_active {
                            bin_width_field.is_active = false;
                            bin_height_field.is_active = true;
                        } else if bin_height_was_active {
                            bin_height_field.is_active = false;
                            num_rects_field.is_active = true;
                        } else if num_rects_was_active {
                            num_rects_field.is_active = false;
                            // Will activate first rect width after drawing loop
                        }
                    }
                }
                
                // Scrollable rectangle input area
                let scroll_area_y = 300.0;
                let scroll_area_height = 380.0;
                let max_visible = 6;
                let item_height = 55.0;
                
                // Draw scroll area background
                draw_rectangle(50.0, scroll_area_y, 900.0, scroll_area_height, Color::new(0.95, 0.95, 0.95, 1.0));
                draw_rectangle_lines(50.0, scroll_area_y, 900.0, scroll_area_height, 2.0, DARKGRAY);
                
                if !manual_rects.is_empty() {
                    draw_text("Rectangle dimensions:", 50.0, scroll_area_y - 10.0, 20.0, BLACK);
                    
                    let total_rects = manual_rects.len();
                    let scroll_start = scroll_offset as usize;
                    let scroll_end = (scroll_start + max_visible).min(total_rects);
                    
                    // Find which field is currently active (before drawing loop)
                    let mut active_width_idx: Option<usize> = None;
                    let mut active_height_idx: Option<usize> = None;
                    for idx in 0..total_rects {
                        if manual_rects[idx].0.is_active {
                            active_width_idx = Some(idx);
                        }
                        if manual_rects[idx].1.is_active {
                            active_height_idx = Some(idx);
                        }
                    }
                    
                    // Draw visible rectangles
                    for idx in scroll_start..scroll_end {
                        let (width_field, height_field) = &mut manual_rects[idx];
                        let display_y = scroll_area_y + 15.0 + (idx - scroll_start) as f32 * item_height;
                        
                        // Rectangle label
                        draw_text(&format!("Rect {}:", idx + 1), 70.0, display_y + 25.0, 18.0, BLACK);
                        
                        // Width field
                        width_field.x = 150.0;
                        width_field.y = display_y;
                        width_field.update();
                        width_field.draw();
                        
                        draw_text("×", 300.0, display_y + 25.0, 20.0, GRAY);
                        
                        // Height field
                        height_field.x = 320.0;
                        height_field.y = display_y;
                        height_field.update();
                        height_field.draw();
                    }
                    
                    // Handle Tab navigation for rectangle fields
                    if tab_pressed {
                        if let Some(idx) = active_width_idx {
                            manual_rects[idx].0.is_active = false;
                            manual_rects[idx].1.is_active = true;
                        } else if let Some(idx) = active_height_idx {
                            manual_rects[idx].1.is_active = false;
                            if idx + 1 < total_rects {
                                // Auto-scroll if next rect is not visible
                                if idx + 1 >= scroll_end {
                                    scroll_offset += 1.0;
                                }
                                manual_rects[idx + 1].0.is_active = true;
                            }
                        } else if num_rects_was_active && !manual_rects.is_empty() {
                            // Coming from num_rects field - activate first rectangle width
                            manual_rects[0].0.is_active = true;
                        }
                    }
                    
                    // Scroll indicator
                    if manual_rects.len() > max_visible {
                        let showing_end = (scroll_offset as usize + max_visible).min(manual_rects.len());
                        let scroll_info = format!("Showing {}-{} of {} (Use ↑↓ or Mouse Wheel)", 
                            scroll_offset as usize + 1,
                            showing_end,
                            manual_rects.len()
                        );
                        //draw_text(&scroll_info, 60.0, scroll_area_y + scroll_area_height + 20.0, 16.0, DARKGRAY);
                        
                        // Scroll bar
                        let scrollbar_x = 960.0;
                        let scrollbar_height = scroll_area_height - 20.0;
                        let thumb_height = (max_visible as f32 / manual_rects.len() as f32) * scrollbar_height;
                        let thumb_y = scroll_area_y + 10.0 + (scroll_offset / (manual_rects.len() - max_visible) as f32) * (scrollbar_height - thumb_height);
                        
                        // Scrollbar track
                        draw_rectangle(scrollbar_x, scroll_area_y + 10.0, 15.0, scrollbar_height, LIGHTGRAY);
                        // Scrollbar thumb
                        draw_rectangle(scrollbar_x, thumb_y, 15.0, thumb_height, DARKGRAY);
                        
                        // Scroll with arrow keys
                        if is_key_pressed(KeyCode::Down) && scroll_offset < (manual_rects.len() - max_visible) as f32 {
                            scroll_offset += 1.0;
                        }
                        if is_key_pressed(KeyCode::Up) && scroll_offset > 0.0 {
                            scroll_offset -= 1.0;
                        }
                        
                        // Scroll with mouse wheel
                        let wheel = mouse_wheel().1;
                        if wheel != 0.0 {
                            scroll_offset = (scroll_offset - wheel).max(0.0).min((manual_rects.len() - max_visible) as f32);
                        }
                    }
                } else {
                    draw_text("Enter number of rectangles above", 350.0, 480.0, 18.0, GRAY);
                }
                
                // Heuristic selection in manual input
                draw_text("Select Heuristic:", 100.0, 700.0, 18.0, BLACK);
                
                let heuristic_button_y = 720.0;
                let heuristics = [
                    (Heuristic::MaxRects, "MaxRects", DARKBLUE),
                    (Heuristic::Skyline, "Skyline", DARKGREEN),
                    (Heuristic::Guillotine, "Guillotine", PURPLE),
                ];
                
                for (i, (heuristic, name, color)) in heuristics.iter().enumerate() {
                    let button_x = 100.0 + i as f32 * 160.0;
                    let is_selected = selected_heuristic == *heuristic;
                    let button_color = if is_selected { *color } else { DARKGRAY };
                    let button = Button::new(button_x, heuristic_button_y, 150.0, 40.0, name, button_color);
                    button.draw();
                    
                    if button.is_clicked() {
                        selected_heuristic = *heuristic;
                    }
                }
                
                // Buttons
                let solve_button = Button::new(600.0, 720.0, 150.0, 50.0, "Solve", GREEN);
                let back_button = Button::new(770.0, 720.0, 150.0, 50.0, "Back", RED);
                
                solve_button.draw();
                back_button.draw();
                
                if solve_button.is_clicked() {
                    // Validate and create problem
                    if let (Some(bin_w), Some(bin_h)) = (bin_width_field.parse_i32(), bin_height_field.parse_i32()) {
                        let mut rectangles = Vec::new();
                        let mut valid = true;
                        
                        for (w_field, h_field) in &manual_rects {
                            if let (Some(w), Some(h)) = (w_field.parse_i32(), h_field.parse_i32()) {
                                if w > 0 && h > 0 {
                                    rectangles.push(Rect::new_unplaced(w, h));
                                } else {
                                    valid = false;
                                    break;
                                }
                            } else {
                                valid = false;
                                break;
                            }
                        }
                        
                        if valid && !rectangles.is_empty() && bin_w > 0 && bin_h > 0 {
                            let p = Problem {
                                bin_width: bin_w,
                                bin_height: bin_h,
                                rectangles,
                            };
                            
                            println!("Manual problem created:");
                            println!("Bin: {}x{}", p.bin_width, p.bin_height);
                            println!("Rectangles: {}", p.rectangles.len());
                            
                            let mut rng = rng();
                            let start = Instant::now();
                            let (best_chromosome, fitness) = genetic_algorithm(
                                &p,
                                100,
                                0.1,
                                0.1,
                                200,
                                &mut rng,
                                selected_heuristic,
                            );
                            
                            println!("GA took: {:.2}s", start.elapsed().as_secs_f64());
                            println!("Fitness: {:.2}%", fitness * 100.0);
                            
                            let (rects, _) = selected_heuristic.decode_chromosome(&best_chromosome, &p);
                            placed_rects = rects;
                            best_fitness = fitness;
                            problem = Some(p);
                            menu_state = MenuState::Solution;
                        } else {
                            println!("Invalid input - please fill all fields with positive numbers");
                        }
                    }
                }
                
                if back_button.is_clicked() {
                    menu_state = MenuState::MainMenu;
                    scroll_offset = 0.0;
                    manual_rects.clear();
                }
            }
            
            MenuState::Comparison => {
                if let Some(ref prob) = problem {
                    draw_text("Heuristic Comparison", 350.0, 40.0, 35.0, BLACK);
                    
                    // Graph area dimensions
                    let graph_padding = 80.0;
                    let graph_width = (screen_width() - 3.0 * graph_padding) / 2.0;
                    let graph_height = 300.0;
                    let graph_y = 100.0;
                    
                    // Fitness comparison graph (left)
                    let fitness_graph_x = graph_padding;
                    draw_rectangle(fitness_graph_x, graph_y, graph_width, graph_height, WHITE);
                    draw_rectangle_lines(fitness_graph_x, graph_y, graph_width, graph_height, 2.0, BLACK);
                    
                    draw_text("Fitness Comparison (%)", fitness_graph_x + 10.0, graph_y - 10.0, 22.0, BLACK);
                    
                    // Draw fitness bars
                    if !comparison_results.is_empty() {
                        let max_fitness = comparison_results.iter().map(|r| r.fitness).fold(0.0f32, f32::max);
                        let bar_width = (graph_width * 0.7) / comparison_results.len() as f32;
                        let bar_spacing = bar_width * 0.4;
                        let total_bars_width = comparison_results.len() as f32 * bar_width + (comparison_results.len() as f32 - 1.0) * bar_spacing;
                        let left_padding = (graph_width - total_bars_width) / 2.0;
                        
                        for (i, result) in comparison_results.iter().enumerate() {
                            let bar_x = fitness_graph_x + left_padding + i as f32 * (bar_width + bar_spacing);
                            let bar_height = (result.fitness / max_fitness.max(1.0)) * (graph_height - 40.0);
                            let bar_y = graph_y + graph_height - bar_height - 20.0;
                            
                            let color = match result.heuristic {
                                Heuristic::MaxRects => DARKBLUE,
                                Heuristic::Skyline => DARKGREEN,
                                Heuristic::Guillotine => PURPLE,
                            };
                            
                            draw_rectangle(bar_x, bar_y, bar_width, bar_height, color);
                            draw_rectangle_lines(bar_x, bar_y, bar_width, bar_height, 2.0, BLACK);
                            
                            // Draw percentage on top of bar
                            let pct_text = format!("{:.1}%", result.fitness * 100.0);
                            let text_dims = measure_text(&pct_text, None, 16, 1.0);
                            draw_text(&pct_text, bar_x + (bar_width - text_dims.width) / 2.0, bar_y - 5.0, 16.0, BLACK);
                        }
                    }
                    
                    // Time comparison graph (right)
                    let time_graph_x = fitness_graph_x + graph_width + graph_padding;
                    draw_rectangle(time_graph_x, graph_y, graph_width, graph_height, WHITE);
                    draw_rectangle_lines(time_graph_x, graph_y, graph_width, graph_height, 2.0, BLACK);
                    
                    draw_text("Execution Time (seconds)", time_graph_x + 10.0, graph_y - 10.0, 22.0, BLACK);
                    
                    // Draw time bars
                    if !comparison_results.is_empty() {
                        let max_time = comparison_results.iter().map(|r| r.time_seconds).fold(0.0f64, f64::max);
                        let bar_width = (graph_width * 0.7) / comparison_results.len() as f32;
                        let bar_spacing = bar_width * 0.4;
                        let total_bars_width = comparison_results.len() as f32 * bar_width + (comparison_results.len() as f32 - 1.0) * bar_spacing;
                        let left_padding = (graph_width - total_bars_width) / 2.0;
                        
                        for (i, result) in comparison_results.iter().enumerate() {
                            let bar_x = time_graph_x + left_padding + i as f32 * (bar_width + bar_spacing);
                            let bar_height = (result.time_seconds / max_time.max(0.001)) as f32 * (graph_height - 40.0);
                            let bar_y = graph_y + graph_height - bar_height - 20.0;
                            
                            let color = match result.heuristic {
                                Heuristic::MaxRects => DARKBLUE,
                                Heuristic::Skyline => DARKGREEN,
                                Heuristic::Guillotine => PURPLE,
                            };
                            
                            draw_rectangle(bar_x, bar_y, bar_width, bar_height, color);
                            draw_rectangle_lines(bar_x, bar_y, bar_width, bar_height, 2.0, BLACK);
                            
                            // Draw time on top of bar
                            let time_text = format!("{:.2}s", result.time_seconds);
                            let text_dims = measure_text(&time_text, None, 16, 1.0);
                            draw_text(&time_text, bar_x + (bar_width - text_dims.width) / 2.0, bar_y - 5.0, 16.0, BLACK);
                        }
                    }
                    
                    // Legend
                    let legend_y = graph_y + graph_height + 40.0;
                    draw_text("Legend:", 80.0, legend_y, 20.0, BLACK);
                    
                    let legend_colors = [
                        (Heuristic::MaxRects, DARKBLUE),
                        (Heuristic::Skyline, DARKGREEN),
                        (Heuristic::Guillotine, PURPLE),
                    ];
                    
                    for (i, (heuristic, color)) in legend_colors.iter().enumerate() {
                        let legend_x = 80.0 + i as f32 * 250.0;
                        draw_rectangle(legend_x, legend_y + 10.0, 30.0, 20.0, *color);
                        draw_rectangle_lines(legend_x, legend_y + 10.0, 30.0, 20.0, 2.0, BLACK);
                        draw_text(heuristic.name(), legend_x + 40.0, legend_y + 25.0, 18.0, BLACK);
                    }
                    
                    // Detailed results table
                    let table_y = legend_y + 70.0;
                    draw_text("Detailed Results:", 80.0, table_y, 22.0, BLACK);
                    
                    let table_start_y = table_y + 30.0;
                    let row_height = 30.0;
                    
                    // Table headers
                    draw_text("Heuristic", 80.0, table_start_y, 18.0, DARKGRAY);
                    draw_text("Fitness", 350.0, table_start_y, 18.0, DARKGRAY);
                    draw_text("Time (s)", 480.0, table_start_y, 18.0, DARKGRAY);
                    draw_text("Placed", 610.0, table_start_y, 18.0, DARKGRAY);
                    draw_text("Waste", 730.0, table_start_y, 18.0, DARKGRAY);
                    
                    // Table rows
                    for (i, result) in comparison_results.iter().enumerate() {
                        let row_y = table_start_y + (i as f32 + 1.0) * row_height;
                        
                        let color = match result.heuristic {
                            Heuristic::MaxRects => DARKBLUE,
                            Heuristic::Skyline => DARKGREEN,
                            Heuristic::Guillotine => PURPLE,
                        };
                        
                        let name = result.heuristic.name().split(" ").next().unwrap_or("");
                        draw_text(name, 80.0, row_y, 18.0, color);
                        draw_text(&format!("{:.2}%", result.fitness * 100.0), 350.0, row_y, 18.0, BLACK);
                        draw_text(&format!("{:.3}", result.time_seconds), 480.0, row_y, 18.0, BLACK);
                        draw_text(&format!("{}/{}", result.placed_rects.len(), prob.rectangles.len()), 610.0, row_y, 18.0, BLACK);
                        draw_text(&format!("{:.2}%", (1.0 - result.fitness) * 100.0), 730.0, row_y, 18.0, BLACK);
                    }
                    
                    // Best result highlight
                    if let Some(best) = comparison_results.iter().max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap()) {
                        let best_y = table_y + 180.0;
                        draw_text(&format!("Best: {} with {:.2}% fitness", 
                            best.heuristic.name().split(" ").next().unwrap_or(""),
                            best.fitness * 100.0
                        ), 80.0, best_y, 20.0, GREEN);
                    }
                    
                    // Back button
                    let back_button = Button::new(400.0, 720.0, 200.0, 50.0, "Back to Solution", RED);
                    back_button.draw();
                    
                    if back_button.is_clicked() {
                        menu_state = MenuState::Solution;
                    }
                }
            }
            
            MenuState::Solution => {
                if let Some(ref prob) = problem {
                    // Draw solution
                    let padding = 50.0;
                    let available_width = screen_width() - (2.0 * padding) - 200.0;
                    let available_height = screen_height() - (2.0 * padding);
                    
                    let scale_x = available_width / prob.bin_width as f32;
                    let scale_y = available_height / prob.bin_height as f32;
                    let scale = scale_x.min(scale_y);
                    
                    let bin_pixel_width = prob.bin_width as f32 * scale;
                    let bin_pixel_height = prob.bin_height as f32 * scale;
                    let offset_x = padding;
                    let offset_y = (screen_height() - bin_pixel_height) / 2.0;
                    
                    draw_rectangle(offset_x, offset_y, bin_pixel_width, bin_pixel_height, LIGHTGRAY);
                    
                    for rect in &placed_rects {
                        draw_rectangle(
                            offset_x + rect.x as f32 * scale,
                            offset_y + rect.y as f32 * scale,
                            rect.width as f32 * scale,
                            rect.height as f32 * scale,
                            PURPLE,
                        );
                        
                        draw_rectangle_lines(
                            offset_x + rect.x as f32 * scale,
                            offset_y + rect.y as f32 * scale,
                            rect.width as f32 * scale,
                            rect.height as f32 * scale,
                            2.0,
                            DARKBLUE,
                        );
                    }

                    draw_rectangle_lines(offset_x, offset_y, bin_pixel_width, bin_pixel_height, 3.0, BLACK);
                    
                    // Info panel
                    let info_x = screen_width() - 200.0;
                    draw_rectangle(info_x, 0.0, 200.0, screen_height(), Color::new(0.2, 0.2, 0.2, 0.9));
                    
                    draw_text("SOLUTION", info_x + 40.0, 40.0, 25.0, WHITE);
                    draw_text(&format!("Fitness: {:.1}%", best_fitness * 100.0), info_x + 20.0, 80.0, 18.0, YELLOW);
                    draw_text(&format!("Waste: {:.1}%", (1.0 - best_fitness) * 100.0), info_x + 20.0, 110.0, 18.0, ORANGE);
                    draw_text(&format!("Placed: {}/{}", placed_rects.len(), prob.rectangles.len()), info_x + 20.0, 140.0, 18.0, GREEN);
                    draw_text(&format!("Bin: {}x{}", prob.bin_width, prob.bin_height), info_x + 20.0, 170.0, 18.0, WHITE);
                    
                    // Display heuristic used
                    draw_text("Heuristic:", info_x + 20.0, 210.0, 16.0, LIGHTGRAY);
                    let heuristic_name = selected_heuristic.name().split(" ").next().unwrap_or("");
                    draw_text(heuristic_name, info_x + 20.0, 230.0, 18.0, SKYBLUE);
                    
                    // Compare and Back buttons
                    let compare_button = Button::new(info_x + 25.0, screen_height() - 140.0, 150.0, 50.0, "Compare", ORANGE);
                    let back_button = Button::new(info_x + 25.0, screen_height() - 70.0, 150.0, 50.0, "Back", RED);
                    
                    compare_button.draw();
                    back_button.draw();
                    
                    if compare_button.is_clicked() {
                        // Run all heuristics and compare
                        comparison_results.clear();
                        
                        let heuristics = [Heuristic::MaxRects, Heuristic::Skyline, Heuristic::Guillotine];
                        
                        for heuristic in &heuristics {
                            let mut rng = rng();
                            let start = Instant::now();
                            
                            let (best_chromosome, fitness) = genetic_algorithm(
                                prob,
                                100,
                                0.1,
                                0.1,
                                200,
                                &mut rng,
                                *heuristic,
                            );
                            
                            let time_seconds = start.elapsed().as_secs_f64();
                            let (rects, _) = heuristic.decode_chromosome(&best_chromosome, prob);
                            
                            comparison_results.push(HeuristicResult {
                                heuristic: *heuristic,
                                fitness,
                                time_seconds,
                                placed_rects: rects,
                            });
                            
                            println!("{}: {:.2}% in {:.2}s", heuristic.name(), fitness * 100.0, time_seconds);
                        }
                        
                        menu_state = MenuState::Comparison;
                    }
                    
                    if back_button.is_clicked() {
                        menu_state = MenuState::MainMenu;
                    }
                }
            }
        }
        
        next_frame().await;
    }
}