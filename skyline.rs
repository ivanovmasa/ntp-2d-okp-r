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
    let mut x = segment.x;
    let mut max_y = segment.y;
    let mut waste = 0;
    
    let target_x = segment.x + width;
    
    for seg in &skyline[start_idx..] {
        if x >= target_x {
            break;
        }
        
        let overlap_width = (seg.x + seg.width).min(target_x) - x.max(seg.x);
        if overlap_width > 0 {
            waste += overlap_width * (max_y - seg.y).abs();
            max_y = max_y.max(seg.y);
            x = seg.x + seg.width;
        }
    }
    
    (max_y, waste)
}

fn update_skyline(skyline: &mut Vec<SkylineSegment>, placed: &Rect) {
    let new_segment = SkylineSegment {
        x: placed.x,
        y: placed.y + placed.height,
        width: placed.width,
    };
    
    // Remove segments that are covered by the new rectangle
    skyline.retain(|seg| {
        let seg_end = seg.x + seg.width;
        let placed_end = placed.x + placed.width;
        
        // Keep if no overlap or if extends beyond placed rect
        seg_end <= placed.x || seg.x >= placed_end || seg.y > placed.y + placed.height
    });
    
    // Split segments that partially overlap
    let mut new_segments = Vec::new();
    let placed_end = placed.x + placed.width;
    
    for seg in skyline.iter() {
        let seg_end = seg.x + seg.width;
        
        // Left part before placed rect
        if seg.x < placed.x && seg_end > placed.x && seg.y < placed.y + placed.height {
            new_segments.push(SkylineSegment {
                x: seg.x,
                y: seg.y,
                width: placed.x - seg.x,
            });
        }
        
        // Right part after placed rect
        if seg.x < placed_end && seg_end > placed_end && seg.y < placed.y + placed.height {
            new_segments.push(SkylineSegment {
                x: placed_end,
                y: seg.y,
                width: seg_end - placed_end,
            });
        }
    }
    
    skyline.extend(new_segments);
    skyline.push(new_segment);
    skyline.sort_by_key(|seg| seg.x);
    
    // Merge adjacent segments at same height
    let mut merged: Vec<SkylineSegment> = Vec::new();
    for seg in skyline.iter() {
        if let Some(last) = merged.last_mut() {
            if last.y == seg.y && last.x + last.width == seg.x {
                last.width += seg.width;
                continue;
            }
        }
        merged.push(seg.clone());
    }
    *skyline = merged;
}
