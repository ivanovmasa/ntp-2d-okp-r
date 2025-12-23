use macroquad::prelude::*;
use crate::ui::{Button, TextField};
use crate::{Problem, Heuristic};
use crate::util::Rect;
use crate::genetic::genetic_algorithm;
use ::rand::rng;
use std::time::Instant;
use std::fs;
use serde_json::Value;

#[derive(PartialEq)]
pub enum MenuState {
    MainMenu,
    HeuristicSelection,
    JsonSelection,
    ManualInput,
    Solution,
    Comparison,
}

pub struct HeuristicResult {
    pub heuristic: Heuristic,
    pub fitness: f32,
    pub time_seconds: f64,
    pub placed_rects: Vec<Rect>,
}

pub struct AppState {
    pub menu_state: MenuState,
    pub selected_heuristic: Heuristic,
    pub selected_json: usize,
    pub placed_rects: Vec<Rect>,
    pub best_fitness: f32,
    pub problem: Option<Problem>,
    pub comparison_results: Vec<HeuristicResult>,
    pub manual_rects: Vec<(TextField, TextField)>,
    pub bin_width_field: TextField,
    pub bin_height_field: TextField,
    pub num_rects_field: TextField,
    pub scroll_offset: f32,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            menu_state: MenuState::MainMenu,
            selected_heuristic: Heuristic::MaxRects,
            selected_json: 1,
            placed_rects: Vec::new(),
            best_fitness: 0.0,
            problem: None,
            comparison_results: Vec::new(),
            manual_rects: Vec::new(),
            bin_width_field: TextField::new(100.0, 120.0, 140.0, 35.0, "Width"),
            bin_height_field: TextField::new(320.0, 120.0, 140.0, 35.0, "Height"),
            num_rects_field: TextField::new(100.0, 220.0, 140.0, 35.0, "Count"),
            scroll_offset: 0.0,
        }
    }
    
    pub fn render(&mut self) {
        match self.menu_state {
            MenuState::MainMenu => self.render_main_menu(),
            MenuState::HeuristicSelection => self.render_heuristic_selection(),
            MenuState::JsonSelection => self.render_json_selection(),
            MenuState::ManualInput => self.render_manual_input(),
            MenuState::Solution => self.render_solution(),
            MenuState::Comparison => self.render_comparison(),
        }
    }
    
    fn render_main_menu(&mut self) {
        draw_text("2D-OKP-R Solver", 375.0, 100.0, 40.0, BLACK);
        draw_text("Choose input method:", 380.0, 180.0, 25.0, DARKGRAY);
        
        let json_button = Button::new(350.0, 250.0, 300.0, 60.0, "Load from JSON", DARKBLUE);
        let manual_button = Button::new(350.0, 330.0, 300.0, 60.0, "Manual Input", DARKGREEN);
        
        json_button.draw();
        manual_button.draw();
        
        if json_button.is_clicked() {
            self.menu_state = MenuState::HeuristicSelection;
        }
        
        if manual_button.is_clicked() {
            self.menu_state = MenuState::ManualInput;
            self.manual_rects.clear();
            self.bin_width_field.text.clear();
            self.bin_height_field.text.clear();
            self.num_rects_field.text.clear();
            self.scroll_offset = 0.0;
        }
    }
    
    fn render_heuristic_selection(&mut self) {
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
            let is_selected = self.selected_heuristic == *heuristic;
            let button_color = if is_selected { *color } else { DARKGRAY };
            let button = Button::new(250.0, y, 500.0, button_height, heuristic.name(), button_color);
            button.draw();
            
            if button.is_clicked() {
                self.selected_heuristic = *heuristic;
            }
        }
        
        let next_button = Button::new(250.0, 720.0, 180.0, 50.0, "Next", GREEN);
        let back_button = Button::new(570.0, 720.0, 180.0, 50.0, "Back", RED);
        
        next_button.draw();
        back_button.draw();
        
        draw_text(&format!("Selected: {}", self.selected_heuristic.name()), 280.0, 120.0, 20.0, BLUE);
        
        if next_button.is_clicked() {
            self.menu_state = MenuState::JsonSelection;
        }
        
        if back_button.is_clicked() {
            self.menu_state = MenuState::MainMenu;
        }
    }
    
    fn render_json_selection(&mut self) {
        draw_text("Select JSON File (1-13)", 350.0, 80.0, 30.0, BLACK);
        
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
            
            let color = if self.selected_json == i { BLUE } else { DARKGRAY };
            let button = Button::new(x, y, button_width, button_height, &format!("Problem {}", i), color);
            button.draw();
            
            if button.is_clicked() {
                self.selected_json = i;
            }
        }
        
        let solve_button = Button::new(250.0, 720.0, 180.0, 50.0, "Solve", GREEN);
        let back_button = Button::new(570.0, 720.0, 180.0, 50.0, "Back", RED);
        
        solve_button.draw();
        back_button.draw();
        
        draw_text(&format!("Selected: Problem {}", self.selected_json), 350.0, 120.0, 20.0, BLUE);
        
        if solve_button.is_clicked() {
            match load_problem_from_json(self.selected_json) {
                Ok(p) => {
                    println!("Loaded problem from json/{}.json", self.selected_json);
                    println!("Bin: {}x{}", p.bin_width, p.bin_height);
                    println!("Rectangles: {}", p.rectangles.len());
                    println!("Heuristic: {}", self.selected_heuristic.name());
                    
                    let mut rng = rng();
                    let start = Instant::now();
                    let (best_chromosome, fitness) = genetic_algorithm(
                        &p,
                        100,
                        0.1,
                        0.1,
                        200,
                        &mut rng,
                        self.selected_heuristic,
                    );
                    
                    println!("GA took: {:.2}s", start.elapsed().as_secs_f64());
                    println!("Fitness: {:.2}%", fitness * 100.0);
                    
                    let (rects, _) = self.selected_heuristic.decode_chromosome(&best_chromosome, &p);
                    self.placed_rects = rects;
                    self.best_fitness = fitness;
                    self.problem = Some(p);
                    self.menu_state = MenuState::Solution;
                }
                Err(e) => {
                    eprintln!("Error loading json/{}.json: {}", self.selected_json, e);
                }
            }
        }
        
        if back_button.is_clicked() {
            self.menu_state = MenuState::MainMenu;
        }
    }
    
    fn render_manual_input(&mut self) {
        draw_text("Manual Input", 400.0, 50.0, 30.0, BLACK);
        
        let tab_pressed = is_key_pressed(KeyCode::Tab);
        
        draw_text("Bin dimensions:", 100.0, 110.0, 20.0, BLACK);
        
        self.bin_width_field.x = 100.0;
        self.bin_width_field.y = 120.0;
        let bin_width_was_active = self.bin_width_field.is_active && tab_pressed;
        self.bin_width_field.update();
        self.bin_width_field.draw();
        
        self.bin_height_field.x = 320.0;
        self.bin_height_field.y = 120.0;
        let bin_height_was_active = self.bin_height_field.is_active && tab_pressed;
        self.bin_height_field.update();
        self.bin_height_field.draw();
        
        draw_text("W", 260.0, 145.0, 18.0, GRAY);
        draw_text("H", 480.0, 145.0, 18.0, GRAY);
        
        draw_text("Number of rectangles:", 100.0, 210.0, 20.0, BLACK);
        let num_rects_was_active = self.num_rects_field.is_active && tab_pressed;
        self.num_rects_field.update();
        self.num_rects_field.draw();
        
        if let Some(count) = self.num_rects_field.parse_i32() {
            let count = count.max(0).min(50) as usize;
            while self.manual_rects.len() < count {
                self.manual_rects.push((
                    TextField::new(0.0, 0.0, 140.0, 35.0, "Width"),
                    TextField::new(0.0, 0.0, 140.0, 35.0, "Height"),
                ));
            }
            while self.manual_rects.len() > count {
                self.manual_rects.pop();
            }
        }
        
        if tab_pressed {
            let any_rect_active = self.manual_rects.iter().any(|(w, h)| w.is_active || h.is_active);
            
            if !any_rect_active {
                if bin_width_was_active {
                    self.bin_width_field.is_active = false;
                    self.bin_height_field.is_active = true;
                } else if bin_height_was_active {
                    self.bin_height_field.is_active = false;
                    self.num_rects_field.is_active = true;
                } else if num_rects_was_active {
                    self.num_rects_field.is_active = false;
                }
            }
        }
        
        let scroll_area_y = 300.0;
        let scroll_area_height = 380.0;
        let max_visible = 6;
        let item_height = 55.0;
        
        draw_rectangle(50.0, scroll_area_y, 900.0, scroll_area_height, Color::new(0.95, 0.95, 0.95, 1.0));
        draw_rectangle_lines(50.0, scroll_area_y, 900.0, scroll_area_height, 2.0, DARKGRAY);
        
        if !self.manual_rects.is_empty() {
            draw_text("Rectangle dimensions:", 50.0, scroll_area_y - 10.0, 20.0, BLACK);
            
            let total_rects = self.manual_rects.len();
            let scroll_start = self.scroll_offset as usize;
            let scroll_end = (scroll_start + max_visible).min(total_rects);
            
            let mut active_width_idx: Option<usize> = None;
            let mut active_height_idx: Option<usize> = None;
            for idx in 0..total_rects {
                if self.manual_rects[idx].0.is_active {
                    active_width_idx = Some(idx);
                }
                if self.manual_rects[idx].1.is_active {
                    active_height_idx = Some(idx);
                }
            }
            
            for idx in scroll_start..scroll_end {
                let (width_field, height_field) = &mut self.manual_rects[idx];
                let display_y = scroll_area_y + 15.0 + (idx - scroll_start) as f32 * item_height;
                
                draw_text(&format!("Rect {}:", idx + 1), 70.0, display_y + 25.0, 18.0, BLACK);
                
                width_field.x = 150.0;
                width_field.y = display_y;
                width_field.update();
                width_field.draw();
                
                draw_text("×", 300.0, display_y + 25.0, 20.0, GRAY);
                
                height_field.x = 320.0;
                height_field.y = display_y;
                height_field.update();
                height_field.draw();
            }
            
            if tab_pressed {
                if let Some(idx) = active_width_idx {
                    self.manual_rects[idx].0.is_active = false;
                    self.manual_rects[idx].1.is_active = true;
                } else if let Some(idx) = active_height_idx {
                    self.manual_rects[idx].1.is_active = false;
                    if idx + 1 < total_rects {
                        if idx + 1 >= scroll_end {
                            self.scroll_offset += 1.0;
                        }
                        self.manual_rects[idx + 1].0.is_active = true;
                    }
                } else if num_rects_was_active && !self.manual_rects.is_empty() {
                    self.manual_rects[0].0.is_active = true;
                }
            }
            
            if self.manual_rects.len() > max_visible {
                let scrollbar_x = 960.0;
                let scrollbar_height = scroll_area_height - 20.0;
                let thumb_height = (max_visible as f32 / self.manual_rects.len() as f32) * scrollbar_height;
                let thumb_y = scroll_area_y + 10.0 + (self.scroll_offset / (self.manual_rects.len() - max_visible) as f32) * (scrollbar_height - thumb_height);
                
                draw_rectangle(scrollbar_x, scroll_area_y + 10.0, 15.0, scrollbar_height, LIGHTGRAY);
                draw_rectangle(scrollbar_x, thumb_y, 15.0, thumb_height, DARKGRAY);
                
                if is_key_pressed(KeyCode::Down) && self.scroll_offset < (self.manual_rects.len() - max_visible) as f32 {
                    self.scroll_offset += 1.0;
                }
                if is_key_pressed(KeyCode::Up) && self.scroll_offset > 0.0 {
                    self.scroll_offset -= 1.0;
                }
                
                let wheel = mouse_wheel().1;
                if wheel != 0.0 {
                    self.scroll_offset = (self.scroll_offset - wheel).max(0.0).min((self.manual_rects.len() - max_visible) as f32);
                }
            }
        } else {
            draw_text("Enter number of rectangles above", 350.0, 480.0, 18.0, GRAY);
        }
        
        draw_text("Select Heuristic:", 100.0, 700.0, 18.0, BLACK);
        
        let heuristic_button_y = 720.0;
        let heuristics = [
            (Heuristic::MaxRects, "MaxRects", DARKBLUE),
            (Heuristic::Skyline, "Skyline", DARKGREEN),
            (Heuristic::Guillotine, "Guillotine", PURPLE),
        ];
        
        for (i, (heuristic, name, color)) in heuristics.iter().enumerate() {
            let button_x = 100.0 + i as f32 * 160.0;
            let is_selected = self.selected_heuristic == *heuristic;
            let button_color = if is_selected { *color } else { DARKGRAY };
            let button = Button::new(button_x, heuristic_button_y, 150.0, 40.0, name, button_color);
            button.draw();
            
            if button.is_clicked() {
                self.selected_heuristic = *heuristic;
            }
        }
        
        let solve_button = Button::new(600.0, 720.0, 150.0, 50.0, "Solve", GREEN);
        let back_button = Button::new(770.0, 720.0, 150.0, 50.0, "Back", RED);
        
        solve_button.draw();
        back_button.draw();
        
        if solve_button.is_clicked() {
            if let (Some(bin_w), Some(bin_h)) = (self.bin_width_field.parse_i32(), self.bin_height_field.parse_i32()) {
                let mut rectangles = Vec::new();
                let mut valid = true;
                
                for (w_field, h_field) in &self.manual_rects {
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
                        self.selected_heuristic,
                    );
                    
                    println!("GA took: {:.2}s", start.elapsed().as_secs_f64());
                    println!("Fitness: {:.2}%", fitness * 100.0);
                    
                    let (rects, _) = self.selected_heuristic.decode_chromosome(&best_chromosome, &p);
                    self.placed_rects = rects;
                    self.best_fitness = fitness;
                    self.problem = Some(p);
                    self.menu_state = MenuState::Solution;
                } else {
                    println!("Invalid input - please fill all fields with positive numbers");
                }
            }
        }
        
        if back_button.is_clicked() {
            self.menu_state = MenuState::MainMenu;
            self.scroll_offset = 0.0;
            self.manual_rects.clear();
        }
    }
    
    fn render_solution(&mut self) {
        if let Some(ref prob) = self.problem {
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
            
            for rect in &self.placed_rects {
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
            
            let info_x = screen_width() - 200.0;
            draw_rectangle(info_x, 0.0, 200.0, screen_height(), Color::new(0.2, 0.2, 0.2, 0.9));
            
            draw_text("SOLUTION", info_x + 40.0, 40.0, 25.0, WHITE);
            draw_text(&format!("Fitness: {:.1}%", self.best_fitness * 100.0), info_x + 20.0, 80.0, 18.0, YELLOW);
            draw_text(&format!("Waste: {:.1}%", (1.0 - self.best_fitness) * 100.0), info_x + 20.0, 110.0, 18.0, ORANGE);
            draw_text(&format!("Placed: {}/{}", self.placed_rects.len(), prob.rectangles.len()), info_x + 20.0, 140.0, 18.0, GREEN);
            draw_text(&format!("Bin: {}x{}", prob.bin_width, prob.bin_height), info_x + 20.0, 170.0, 18.0, WHITE);
            
            draw_text("Heuristic:", info_x + 20.0, 210.0, 16.0, LIGHTGRAY);
            let heuristic_name = self.selected_heuristic.name().split(" ").next().unwrap_or("");
            draw_text(heuristic_name, info_x + 20.0, 230.0, 18.0, SKYBLUE);
            
            let compare_button = Button::new(info_x + 25.0, screen_height() - 140.0, 150.0, 50.0, "Compare", ORANGE);
            let back_button = Button::new(info_x + 25.0, screen_height() - 70.0, 150.0, 50.0, "Back", RED);
            
            compare_button.draw();
            back_button.draw();
            
            if compare_button.is_clicked() {
                self.comparison_results.clear();
                
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
                    
                    self.comparison_results.push(HeuristicResult {
                        heuristic: *heuristic,
                        fitness,
                        time_seconds,
                        placed_rects: rects,
                    });
                    
                    println!("{}: {:.2}% in {:.2}s", heuristic.name(), fitness * 100.0, time_seconds);
                }
                
                self.menu_state = MenuState::Comparison;
            }
            
            if back_button.is_clicked() {
                self.menu_state = MenuState::MainMenu;
            }
        }
    }
    
    fn render_comparison(&mut self) {
        if let Some(ref prob) = self.problem {
            draw_text("Heuristic Comparison", 350.0, 40.0, 35.0, BLACK);
            
            let graph_padding = 80.0;
            let graph_width = (screen_width() - 3.0 * graph_padding) / 2.0;
            let graph_height = 300.0;
            let graph_y = 100.0;
            
            let fitness_graph_x = graph_padding;
            draw_rectangle(fitness_graph_x, graph_y, graph_width, graph_height, WHITE);
            draw_rectangle_lines(fitness_graph_x, graph_y, graph_width, graph_height, 2.0, BLACK);
            
            draw_text("Fitness Comparison (%)", fitness_graph_x + 10.0, graph_y - 10.0, 22.0, BLACK);
            
            if !self.comparison_results.is_empty() {
                let max_fitness = self.comparison_results.iter().map(|r| r.fitness).fold(0.0f32, f32::max);
                let bar_width = (graph_width * 0.7) / self.comparison_results.len() as f32;
                let bar_spacing = bar_width * 0.4;
                let total_width = self.comparison_results.len() as f32 * bar_width + (self.comparison_results.len() as f32 - 1.0) * bar_spacing;
                let left_padding = (graph_width - total_width) / 2.0;
                
                for (i, result) in self.comparison_results.iter().enumerate() {
                    let x = fitness_graph_x + left_padding + i as f32 * (bar_width + bar_spacing);
                    let bar_height = (result.fitness / max_fitness) * (graph_height - 60.0);
                    let y = graph_y + graph_height - bar_height - 30.0;
                    
                    let color = match result.heuristic {
                        Heuristic::MaxRects => DARKBLUE,
                        Heuristic::Skyline => DARKGREEN,
                        Heuristic::Guillotine => PURPLE,
                    };
                    
                    draw_rectangle(x, y, bar_width, bar_height, color);
                    draw_text(&format!("{:.1}%", result.fitness * 100.0), x, y - 5.0, 16.0, BLACK);
                    
                    let label = result.heuristic.name().split(" ").next().unwrap_or("");
                    let label_dims = measure_text(label, None, 14, 1.0);
                    let label_x = x + (bar_width - label_dims.width) / 2.0;
                    draw_text(label, label_x, graph_y + graph_height - 10.0, 14.0, BLACK);
                }
            }
            
            // Time comparison graph
            let time_graph_x = fitness_graph_x + graph_width + graph_padding;
            draw_rectangle(time_graph_x, graph_y, graph_width, graph_height, WHITE);
            draw_rectangle_lines(time_graph_x, graph_y, graph_width, graph_height, 2.0, BLACK);
            
            draw_text("Time Comparison (s)", time_graph_x + 10.0, graph_y - 10.0, 22.0, BLACK);
            
            if !self.comparison_results.is_empty() {
                let max_time = self.comparison_results.iter().map(|r| r.time_seconds).fold(0.0f64, f64::max);
                let bar_width = (graph_width * 0.7) / self.comparison_results.len() as f32;
                let bar_spacing = bar_width * 0.4;
                let total_width = self.comparison_results.len() as f32 * bar_width + (self.comparison_results.len() as f32 - 1.0) * bar_spacing;
                let left_padding = (graph_width - total_width) / 2.0;
                
                for (i, result) in self.comparison_results.iter().enumerate() {
                    let x = time_graph_x + left_padding + i as f32 * (bar_width + bar_spacing);
                    let bar_height = (result.time_seconds / max_time) as f32 * (graph_height - 60.0);
                    let y = graph_y + graph_height - bar_height - 30.0;
                    
                    let color = match result.heuristic {
                        Heuristic::MaxRects => DARKBLUE,
                        Heuristic::Skyline => DARKGREEN,
                        Heuristic::Guillotine => PURPLE,
                    };
                    
                    draw_rectangle(x, y, bar_width, bar_height, color);
                    draw_text(&format!("{:.2}s", result.time_seconds), x, y - 5.0, 16.0, BLACK);
                    
                    let label = result.heuristic.name().split(" ").next().unwrap_or("");
                    let label_dims = measure_text(label, None, 14, 1.0);
                    let label_x = x + (bar_width - label_dims.width) / 2.0;
                    draw_text(label, label_x, graph_y + graph_height - 10.0, 14.0, BLACK);
                }
            }
            
            // Visual comparison of solutions
            draw_text("Solution Visualizations:", 80.0, 450.0, 25.0, BLACK);
            
            if !self.comparison_results.is_empty() {
                let viz_width = (screen_width() - 4.0 * 40.0) / 3.0;
                let viz_height = 200.0;
                let viz_y = 490.0;
                
                for (i, result) in self.comparison_results.iter().enumerate() {
                    let viz_x = 40.0 + i as f32 * (viz_width + 40.0);
                    
                    let scale_x = viz_width / prob.bin_width as f32;
                    let scale_y = viz_height / prob.bin_height as f32;
                    let scale = scale_x.min(scale_y) * 0.9;
                    
                    let actual_width = prob.bin_width as f32 * scale;
                    let actual_height = prob.bin_height as f32 * scale;
                    let centered_x = viz_x + (viz_width - actual_width) / 2.0;
                    
                    draw_rectangle(centered_x, viz_y, actual_width, actual_height, LIGHTGRAY);
                    
                    for rect in &result.placed_rects {
                        draw_rectangle(
                            centered_x + rect.x as f32 * scale,
                            viz_y + rect.y as f32 * scale,
                            rect.width as f32 * scale,
                            rect.height as f32 * scale,
                            PURPLE,
                        );
                        
                        draw_rectangle_lines(
                            centered_x + rect.x as f32 * scale,
                            viz_y + rect.y as f32 * scale,
                            rect.width as f32 * scale,
                            rect.height as f32 * scale,
                            1.0,
                            DARKBLUE,
                        );
                    }
                    
                    draw_rectangle_lines(centered_x, viz_y, actual_width, actual_height, 2.0, BLACK);
                    
                    let label = result.heuristic.name().split(" ").next().unwrap_or("");
                    draw_text(label, centered_x, viz_y - 10.0, 20.0, BLACK);
                    draw_text(&format!("Placed: {}/{}", result.placed_rects.len(), prob.rectangles.len()), 
                        centered_x, viz_y + actual_height + 20.0, 16.0, DARKGRAY);
                }
            }
            
            let back_button = Button::new(screen_width() / 2.0 - 75.0, 750.0, 150.0, 40.0, "Back", RED);
            back_button.draw();
            
            if back_button.is_clicked() {
                self.menu_state = MenuState::Solution;
            }
        }
    }
}

pub fn load_problem_from_json(file_num: usize) -> Result<Problem, Box<dyn std::error::Error>> {
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
