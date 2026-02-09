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
        
        if let Some((_best_idx, placed_rect)) = find_best_area_fit(
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
                split_all_free_rects(&mut free_rects, &placed_rect);  
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
    let mut best_short_side_fit = i32::MAX;
    let mut best_long_side_fit = i32::MAX;
    let mut best_area_diff = i32::MAX;
    let mut best_rect = None;
    
    for (idx, free_rect) in free_rects.iter().enumerate() {
        for &(w, h) in &[(rect_width, rect_height), (rect_height, rect_width)] {
            if free_rect.contains(w, h) {
                let leftover_horiz = free_rect.width - w;
                let leftover_vert = free_rect.height - h;
                let short_side_fit = leftover_horiz.min(leftover_vert);
                let long_side_fit = leftover_horiz.max(leftover_vert);
                let area_diff = free_rect.area() - (rect_width * rect_height);
                
                if short_side_fit < best_short_side_fit || 
                   (short_side_fit == best_short_side_fit && long_side_fit < best_long_side_fit) ||
                   (short_side_fit == best_short_side_fit && long_side_fit == best_long_side_fit && area_diff < best_area_diff) {
                    best_short_side_fit = short_side_fit;
                    best_long_side_fit = long_side_fit;
                    best_area_diff = area_diff;
                    best_idx = Some(idx);
                    best_rect = Some(Rect {
                        x: free_rect.x,
                        y: free_rect.y,
                        width: w,
                        height: h,
                    });
                }
            }
        }
    }
    
    best_idx.and_then(|idx| best_rect.map(|rect| (idx, rect)))
}

fn split_all_free_rects(free_rects: &mut Vec<Rect>, placed: &Rect) {
    let mut new_rects = Vec::new();
    let mut i = 0;
    
    while i < free_rects.len() {
        let free_rect = &free_rects[i];
        
        // Check if the placed rectangle intersects with this free rectangle
        if rectangles_overlap(placed, free_rect) {
            let old_free = free_rects.remove(i);
            
            // Create up to 4 new free rectangles from the intersection
            // Left side
            if placed.x > old_free.x {
                new_rects.push(Rect {
                    x: old_free.x,
                    y: old_free.y,
                    width: placed.x - old_free.x,
                    height: old_free.height,
                });
            }
            
            // Right side
            if placed.x + placed.width < old_free.x + old_free.width {
                new_rects.push(Rect {
                    x: placed.x + placed.width,
                    y: old_free.y,
                    width: (old_free.x + old_free.width) - (placed.x + placed.width),
                    height: old_free.height,
                });
            }
            
            // Top side
            if placed.y > old_free.y {
                new_rects.push(Rect {
                    x: old_free.x,
                    y: old_free.y,
                    width: old_free.width,
                    height: placed.y - old_free.y,
                });
            }
            
            // Bottom side
            if placed.y + placed.height < old_free.y + old_free.height {
                new_rects.push(Rect {
                    x: old_free.x,
                    y: placed.y + placed.height,
                    width: old_free.width,
                    height: (old_free.y + old_free.height) - (placed.y + placed.height),
                });
            }
        } else {
            i += 1;
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