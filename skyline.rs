use crate::util::{rectangles_overlap, Rect};
use crate::Problem;

#[derive(Clone, Debug)]
struct SkylineSegment {
    x: i32,
    y: i32,
    width: i32,
}

pub fn decode_chromosome(
    chromosome: &[u8],
    problem: &Problem,
) -> (Vec<Rect>, f32) {
    let mut placed_rects: Vec<Rect> = Vec::new();
    let mut skyline: Vec<SkylineSegment> = vec![SkylineSegment {
        x: 0,
        y: 0,
        width: problem.bin_width,
    }];
    
    for (i, &gene) in chromosome.iter().enumerate() {
        if gene == 0 || i >= problem.rectangles.len() {
            continue;
        }
        
        let rect = &problem.rectangles[i];
        
        if let Some(placed_rect) = find_best_skyline_position(
            &skyline,
            rect.width,
            rect.height,
            problem.bin_width,
            problem.bin_height,
        ) {
            let no_overlap = placed_rects.iter()
                .all(|existing| !rectangles_overlap(&placed_rect, existing));
            
            if no_overlap {
                placed_rects.push(placed_rect);
                update_skyline(&mut skyline, &placed_rect);
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

fn find_best_skyline_position(
    skyline: &[SkylineSegment],
    rect_width: i32,
    rect_height: i32,
    bin_width: i32,
    bin_height: i32,
) -> Option<Rect> {
    let mut best_rect: Option<Rect> = None;
    let mut best_y = i32::MAX;
    let mut best_waste = i32::MAX;
    
    // Try normal orientation
    for (i, segment) in skyline.iter().enumerate() {
        if segment.x + rect_width > bin_width {
            continue;
        }
        
        let (fit_y, waste) = calculate_fit(skyline, i, rect_width);
        
        if fit_y + rect_height <= bin_height && (fit_y < best_y || (fit_y == best_y && waste < best_waste)) {
            best_y = fit_y;
            best_waste = waste;
            best_rect = Some(Rect {
                x: segment.x,
                y: fit_y,
                width: rect_width,
                height: rect_height,
            });
        }
    }
    
    // Try rotated orientation (90 degrees)
    for (i, segment) in skyline.iter().enumerate() {
        if segment.x + rect_height > bin_width {
            continue;
        }
        
        let (fit_y, waste) = calculate_fit(skyline, i, rect_height);
        
        if fit_y + rect_width <= bin_height && (fit_y < best_y || (fit_y == best_y && waste < best_waste)) {
            best_y = fit_y;
            best_waste = waste;
            best_rect = Some(Rect {
                x: segment.x,
                y: fit_y,
                width: rect_height,
                height: rect_width,
            });
        }
    }
    
    best_rect
}

fn calculate_fit(skyline: &[SkylineSegment], start_idx: usize, width: i32) -> (i32, i32) {
    let segment = &skyline[start_idx];
    let start_x = segment.x;
    let end_x = start_x + width;
    let mut max_y = 0;
    let mut waste = 0;
    
    // Find all segments that the rectangle would overlap
    for seg in skyline.iter() {
        let seg_end = seg.x + seg.width;
        
        // Check if this segment overlaps with our rectangle's horizontal span
        if seg.x < end_x && seg_end > start_x {
            let overlap_start = seg.x.max(start_x);
            let overlap_end = seg_end.min(end_x);
            let overlap_width = overlap_end - overlap_start;
            
            if overlap_width > 0 {
                // Calculate waste (the gap between current max_y and this segment)
                if max_y > seg.y {
                    waste += overlap_width * (max_y - seg.y);
                }
                max_y = max_y.max(seg.y);
            }
        }
    }
    
    (max_y, waste)
}

fn update_skyline(skyline: &mut Vec<SkylineSegment>, placed: &Rect) {
    let placed_left = placed.x;
    let placed_right = placed.x + placed.width;
    let placed_top = placed.y + placed.height;
    
    let mut new_skyline = Vec::new();
    
    for seg in skyline.iter() {
        let seg_right = seg.x + seg.width;
        
        // Segment is completely to the left of placed rectangle
        if seg_right <= placed_left {
            new_skyline.push(seg.clone());
        }
        // Segment is completely to the right of placed rectangle
        else if seg.x >= placed_right {
            new_skyline.push(seg.clone());
        }
        // Segment overlaps with placed rectangle horizontally
        else {
            // Left portion of segment (before placed rectangle)
            if seg.x < placed_left {
                new_skyline.push(SkylineSegment {
                    x: seg.x,
                    y: seg.y,
                    width: placed_left - seg.x,
                });
            }
            
            // Right portion of segment (after placed rectangle)
            if seg_right > placed_right {
                new_skyline.push(SkylineSegment {
                    x: placed_right,
                    y: seg.y,
                    width: seg_right - placed_right,
                });
            }
        }
    }
    
    // Add the new segment on top of the placed rectangle
    new_skyline.push(SkylineSegment {
        x: placed_left,
        y: placed_top,
        width: placed.width,
    });
    
    // Sort by x position
    new_skyline.sort_by_key(|seg| seg.x);
    
    // Merge adjacent segments at the same height
    let mut merged: Vec<SkylineSegment> = Vec::new();
    for seg in new_skyline {
        if let Some(last) = merged.last_mut() {
            if last.y == seg.y && last.x + last.width == seg.x {
                last.width += seg.width;
                continue;
            }
        }
        merged.push(seg);
    }
    
    *skyline = merged;
}
