use crate::util::{rectangles_overlap, Rect};
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
        
        if let Some((best_idx, placed_rect)) = find_best_guillotine_fit(
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
                split_free_rectangle(&mut free_rects, best_idx, &placed_rect);
            }
        }
    }
    
    let total_area = (problem.bin_width * problem.bin_height) as f32;
    let used_area: f32 = placed_rects.iter()
        .map(|r| r.area() as f32)
        .sum();
    let fitness = used_area / total_area;
    
    (placed_rects, fitness)
}

/// Returns (short_side_fit, long_side_fit) if the rectangle fits, None otherwise.
fn try_fit(free: &Rect, width: i32, height: i32) -> Option<(i32, i32)> {
    if free.width >= width && free.height >= height {
        let leftover_horiz = free.width - width;
        let leftover_vert = free.height - height;
        Some((leftover_horiz.min(leftover_vert), leftover_horiz.max(leftover_vert)))
    } else {
        None
    }
}

fn find_best_guillotine_fit(
    free_rects: &[Rect],
    rect_width: i32,
    rect_height: i32,
) -> Option<(usize, Rect)> {
    let mut best_idx: Option<usize> = None;
    let mut best_short_side_fit = i32::MAX;
    let mut best_long_side_fit = i32::MAX;
    let mut best_rect: Option<Rect> = None;
    
    for (idx, free_rect) in free_rects.iter().enumerate() {
        // Try both normal and rotated orientations
        for &(w, h) in &[(rect_width, rect_height), (rect_height, rect_width)] {
            if let Some((short_fit, long_fit)) = try_fit(free_rect, w, h) {
                if short_fit < best_short_side_fit
                    || (short_fit == best_short_side_fit && long_fit < best_long_side_fit)
                {
                    best_idx = Some(idx);
                    best_short_side_fit = short_fit;
                    best_long_side_fit = long_fit;
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

fn split_free_rectangle(
    free_rects: &mut Vec<Rect>,
    used_idx: usize,
    placed: &Rect,
) {
    let used_rect = free_rects.remove(used_idx);
    
    // Decide split direction based on remaining space
    let width_left = used_rect.width - placed.width;
    let height_left = used_rect.height - placed.height;
    
    if width_left >= height_left {
        // Vertical split (prefer horizontal cuts)
        
        // Right rectangle
        if width_left > 0 {
            free_rects.push(Rect {
                x: placed.x + placed.width,
                y: used_rect.y,
                width: width_left,
                height: used_rect.height,
            });
        }
        
        // Bottom rectangle (only the placed width)
        if height_left > 0 {
            free_rects.push(Rect {
                x: used_rect.x,
                y: placed.y + placed.height,
                width: placed.width,
                height: height_left,
            });
        }
    } else {
        // Horizontal split (prefer vertical cuts)
        
        // Bottom rectangle
        if height_left > 0 {
            free_rects.push(Rect {
                x: used_rect.x,
                y: placed.y + placed.height,
                width: used_rect.width,
                height: height_left,
            });
        }
        
        // Right rectangle (only the placed height)
        if width_left > 0 {
            free_rects.push(Rect {
                x: placed.x + placed.width,
                y: used_rect.y,
                width: width_left,
                height: placed.height,
            });
        }
    }
}
