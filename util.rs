#[derive(Clone, Copy, Debug)]
pub struct Rect {  
    pub x: i32,     
    pub y: i32,     
    pub width: i32, 
    pub height: i32, 
}

impl Rect {
    pub fn new_unplaced(width: i32, height: i32) -> Self {  
        Self { x: 0, y: 0, width, height }
    }
    
    pub fn area(&self) -> i32 {  
        self.width * self.height
    }

    pub fn contains(&self, rect_width: i32, rect_height: i32) -> bool {  
        self.width >= rect_width && self.height >= rect_height
    }

    pub fn contains_rect(&self, other: &Rect) -> bool {  
        other.x >= self.x
            && other.y >= self.y
            && other.x + other.width <= self.x + self.width
            && other.y + other.height <= self.y + self.height
    }
}

pub fn is_contained_in(rect1: &Rect, rect2: &Rect) -> bool {  
    rect2.contains_rect(rect1)
}

pub fn rectangles_overlap(r1: &Rect, r2: &Rect) -> bool {  
    !(r1.x + r1.width <= r2.x ||  
      r2.x + r2.width <= r1.x ||  
      r1.y + r1.height <= r2.y || 
      r2.y + r2.height <= r1.y)   
}