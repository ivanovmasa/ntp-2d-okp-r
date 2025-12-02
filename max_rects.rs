use crate::util::{is_contained_in, rectangles_overlap, Rect};  
use crate::Problem;  

pub fn decode_chromosome(  
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