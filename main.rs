use macroquad::prelude::*;
use ::rand::rng;
use std::time::Instant;
use serde_json::Value;
use std::fs;
use std::env; 

mod genetic;  
use genetic::genetic_algorithm;

#[derive(Clone, Copy, Debug)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Rect {
    fn new_unplaced(width: i32, height: i32) -> Self {
        Self { x: 0, y: 0, width, height }
    }
    
    fn area(&self) -> i32 {
        self.width * self.height
    }

    fn contains(&self, rect_width: i32, rect_height: i32) -> bool {
        self.width >= rect_width && self.height >= rect_height
    }

    fn contains_rect(&self, other: &Rect) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.x + other.width <= self.x + self.width
            && other.y + other.height <= self.y + self.height
    }
}

struct Problem {
    bin_width: i32,
    bin_height: i32,
    rectangles: Vec<Rect>, 
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
                    KeyCode::Enter => {
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
    JsonSelection,
    ManualInput,
    Solution,
}

// ...keep all your existing functions (rectangles_overlap, decode_chromosome, etc.)...

fn rectangles_overlap(r1: &Rect, r2: &Rect) -> bool {
    !(r1.x + r1.width <= r2.x ||  
      r2.x + r2.width <= r1.x ||  
      r1.y + r1.height <= r2.y || 
      r2.y + r2.height <= r1.y)   
}

fn decode_chromosome(
    chromosome: &[u8],
    problem: &Problem,
) -> (Vec<Rect>, f32) {
    let mut placed_rects: Vec<Rect> = Vec::new();
    let mut free_rects: Vec<Rect> = vec![Rect {
        x: 0,
        y: 0,
        width: problem.bin_width,
        height: problem.bin_height,
    }];
    
    for (i, &gene) in chromosome.iter().enumerate() {
        if gene == 0 || i >= problem.rectangles.len() {
            continue;
        }
        
        let rect = &problem.rectangles[i];
        
        if let Some((best_idx, placed_rect)) = find_best_area_fit(
            &free_rects,
            rect.width,
            rect.height,
        ) {
            let within_bounds = placed_rect.x + placed_rect.width <= problem.bin_width
                && placed_rect.y + placed_rect.height <= problem.bin_height;
            
            let no_overlap = placed_rects.iter()
                .all(|existing| !rectangles_overlap(&placed_rect, existing));
            
            if within_bounds && no_overlap {
                placed_rects.push(placed_rect);
                split_free_rect(&mut free_rects, best_idx, &placed_rect, problem.bin_width, problem.bin_height);  
                prune_free_rects(&mut free_rects);
            }
        }
    }
    
    let total_area = (problem.bin_width * problem.bin_height) as f32;
    let used_area: i32 = placed_rects.iter()
        .map(|r| r.area())  
        .sum();
    let fitness = (used_area as f32) / total_area;
    
    (placed_rects, fitness)
}

fn find_best_area_fit(
    free_rects: &[Rect],
    rect_width: i32,
    rect_height: i32,
) -> Option<(usize, Rect)> {
    let mut best_idx = None;
    let mut best_area_diff = i32::MAX;
    let mut best_rect = None;
    
    for (idx, free_rect) in free_rects.iter().enumerate() {
        if free_rect.contains(rect_width, rect_height) {
            let placed = Rect {
                x: free_rect.x,
                y: free_rect.y,
                width: rect_width,
                height: rect_height,
            };
            
            let area_diff = free_rect.area() - (rect_width * rect_height);
            
            if area_diff < best_area_diff {
                best_area_diff = area_diff;
                best_idx = Some(idx);
                best_rect = Some(placed);
            }
        }
        
        if free_rect.contains(rect_height, rect_width) {
            let placed = Rect {
                x: free_rect.x,
                y: free_rect.y,
                width: rect_height, 
                height: rect_width,  
            };
        
            let area_diff = free_rect.area() - (rect_width * rect_height);
            
            if area_diff < best_area_diff {
                best_area_diff = area_diff;
                best_idx = Some(idx);
                best_rect = Some(placed);
            }
        }
    }
    
    best_idx.and_then(|idx| best_rect.map(|rect| (idx, rect)))
}

fn split_free_rect(free_rects: &mut Vec<Rect>, used_idx: usize, placed: &Rect, bin_width: i32, bin_height: i32) {  
    let used_rect = free_rects.remove(used_idx);
    
    let mut new_rects = Vec::new();
    
    if placed.x + placed.width < used_rect.x + used_rect.width {
        let new_rect = Rect {
            x: placed.x + placed.width,
            y: used_rect.y,
            width: (used_rect.x + used_rect.width) - (placed.x + placed.width),
            height: used_rect.height,
        };
        
        if new_rect.x + new_rect.width <= bin_width && new_rect.y + new_rect.height <= bin_height {
            new_rects.push(new_rect);
        }
    }
    
    if placed.y + placed.height < used_rect.y + used_rect.height {
        let new_rect = Rect {
            x: used_rect.x,
            y: placed.y + placed.height,
            width: used_rect.width,
            height: (used_rect.y + used_rect.height) - (placed.y + placed.height),
        };
        
        if new_rect.x + new_rect.width <= bin_width && new_rect.y + new_rect.height <= bin_height {
            new_rects.push(new_rect);
        }
    }
    
    free_rects.extend(new_rects);
}

fn prune_free_rects(free_rects: &mut Vec<Rect>) {
    let mut i = 0;
    while i < free_rects.len() {
        let mut j = i + 1;
        let mut remove_i = false;
        
        while j < free_rects.len() {
            if is_contained_in(&free_rects[i], &free_rects[j]) {
                remove_i = true;
                break;
            } else if is_contained_in(&free_rects[j], &free_rects[i]) {
                free_rects.remove(j);
            } else {
                j += 1;
            }
        }
        
        if remove_i {
            free_rects.remove(i);
        } else {
            i += 1;
        }
    }
}

fn is_contained_in(rect1: &Rect, rect2: &Rect) -> bool {
    rect2.contains_rect(rect1)
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
    let mut selected_json: usize = 1;
    let mut problem: Option<Problem> = None;
    let mut placed_rects: Vec<Rect> = Vec::new();
    let mut best_fitness = 0.0;
    
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
                draw_text("2D-OKP-R Solver", 350.0, 100.0, 40.0, BLACK);
                draw_text("Choose input method:", 380.0, 180.0, 25.0, DARKGRAY);
                
                // Buttons
                let json_button = Button::new(350.0, 250.0, 300.0, 60.0, "Load from JSON", DARKBLUE);
                let manual_button = Button::new(350.0, 330.0, 300.0, 60.0, "Manual Input", DARKGREEN);
                
                json_button.draw();
                manual_button.draw();
                
                if json_button.is_clicked() {
                    menu_state = MenuState::JsonSelection;
                }
                
                if manual_button.is_clicked() {
                    menu_state = MenuState::ManualInput;
                    manual_rects.clear();
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
                            
                            let mut rng = rng();
                            let start = Instant::now();
                            let (best_chromosome, fitness) = genetic_algorithm(
                                &p,
                                100,
                                0.1,
                                0.1,
                                200,
                                &mut rng,
                            );
                            
                            println!("GA took: {:.2}s", start.elapsed().as_secs_f64());
                            println!("Fitness: {:.2}%", fitness * 100.0);
                            
                            let (rects, _) = decode_chromosome(&best_chromosome, &p);
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
                
                // Bin size inputs (always visible)
                draw_text("Bin dimensions:", 100.0, 110.0, 20.0, BLACK);
                
                bin_width_field.x = 100.0;
                bin_width_field.y = 120.0;
                bin_width_field.update();
                bin_width_field.draw();
                
                bin_height_field.x = 320.0;
                bin_height_field.y = 120.0;
                bin_height_field.update();
                bin_height_field.draw();
                
                draw_text("W", 260.0, 145.0, 18.0, GRAY);
                draw_text("H", 480.0, 145.0, 18.0, GRAY);
                
                // Number of rectangles
                draw_text("Number of rectangles:", 100.0, 210.0, 20.0, BLACK);
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
                    
                    // Draw visible rectangles
                    for (idx, (width_field, height_field)) in manual_rects.iter_mut()
                        .enumerate()
                        .skip(scroll_offset as usize)
                        .take(max_visible) 
                    {
                        let display_y = scroll_area_y + 15.0 + (idx as f32 - scroll_offset) * item_height;
                        
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
                    
                    // Scroll indicator
                    if manual_rects.len() > max_visible {
                        let showing_end = (scroll_offset as usize + max_visible).min(manual_rects.len());
                        let scroll_info = format!("Showing {}-{} of {} (Use ↑↓ or Mouse Wheel)", 
                            scroll_offset as usize + 1,
                            showing_end,
                            manual_rects.len()
                        );
                        draw_text(&scroll_info, 60.0, scroll_area_y + scroll_area_height + 20.0, 16.0, DARKGRAY);
                        
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
                
                // Buttons
                let solve_button = Button::new(250.0, 720.0, 180.0, 50.0, "Solve", GREEN);
                let back_button = Button::new(570.0, 720.0, 180.0, 50.0, "Back", RED);
                
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
                            );
                            
                            println!("GA took: {:.2}s", start.elapsed().as_secs_f64());
                            println!("Fitness: {:.2}%", fitness * 100.0);
                            
                            let (rects, _) = decode_chromosome(&best_chromosome, &p);
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
                    
                    // Back button
                    let back_button = Button::new(info_x + 25.0, screen_height() - 70.0, 150.0, 50.0, "Back", RED);
                    back_button.draw();
                    
                    if back_button.is_clicked() {
                        menu_state = MenuState::MainMenu;
                    }
                }
            }
        }
        
        next_frame().await;
    }
}