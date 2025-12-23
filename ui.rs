use macroquad::prelude::*;

pub struct Button {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub text: String,
    pub color: Color,
    pub hover_color: Color,
}

impl Button {
    pub fn new(x: f32, y: f32, width: f32, height: f32, text: &str, color: Color) -> Self {
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
    
    pub fn is_hovered(&self) -> bool {
        let (mouse_x, mouse_y) = mouse_position();
        mouse_x >= self.x && mouse_x <= self.x + self.width &&
        mouse_y >= self.y && mouse_y <= self.y + self.height
    }
    
    pub fn is_clicked(&self) -> bool {
        self.is_hovered() && is_mouse_button_pressed(MouseButton::Left)
    }
    
    pub fn draw(&self) {
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
pub struct TextField {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub text: String,
    pub is_active: bool,
    pub placeholder: String,
}

impl TextField {
    pub fn new(x: f32, y: f32, width: f32, height: f32, placeholder: &str) -> Self {
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
    
    pub fn is_clicked(&self) -> bool {
        let (mouse_x, mouse_y) = mouse_position();
        is_mouse_button_pressed(MouseButton::Left) &&
        mouse_x >= self.x && mouse_x <= self.x + self.width &&
        mouse_y >= self.y && mouse_y <= self.y + self.height
    }
    
    pub fn update(&mut self) {
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
    
    pub fn draw(&self) {
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
    
    pub fn parse_i32(&self) -> Option<i32> {
        self.text.trim().parse().ok()
    }
}
