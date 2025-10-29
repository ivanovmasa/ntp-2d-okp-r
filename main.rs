use macroquad::prelude::*;
use ::rand::prelude::*;
use ::rand::rng;
use ::rand::rngs::ThreadRng;
use rayon::prelude::*;
//use ::rand::SeedableRng; 
//use ::rand::rngs::SmallRng;
use std::time::Instant;
use serde_json::Value;
use std::fs;
use std::env; 

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

fn generate_chromosome(len: usize, rng: &mut ThreadRng) -> Vec<u8> {
    let mut chromosome = vec![1u8; len];
    let zeros = len / 10;
    
    (0..zeros).for_each(|_| {
        chromosome[rng.random_range(0..len)] = 0;
    });
    
    chromosome
}

fn generate_initial_chromosomes(size: usize, count: usize, mut rng: &mut ThreadRng) -> Vec<Vec<u8>> {
    (0..count)
        .map(|_| generate_chromosome(size, &mut rng))
        .collect()
}

fn roulette_selection(parents: &[Vec<u8>], rng: &mut ThreadRng) -> Vec<(Vec<u8>, Vec<u8>)> {
    (0..(parents.len() / 2))
        .map(|_| {
            let weights: Vec<f32> = (0..parents.len())
                .map(|i| (i as f32 + 1.0) * rng.random::<f32>())
                .collect();
            
            let mut indices: Vec<usize> = (0..parents.len()).collect();
            indices.sort_by(|a, b| weights[*b].partial_cmp(&weights[*a]).unwrap());
            
            (parents[indices[0]].clone(), parents[indices[1]].clone())
        })
        .collect()
}

//SERIAL VERSION
fn two_point_crossover(pairs: &[(Vec<u8>, Vec<u8>)], rng: &mut ThreadRng) -> Vec<Vec<u8>> {
    pairs.iter()
        .flat_map(|(parent1, parent2)| {
            let len = parent1.len();
            let r1 = rng.random_range(0..len);
            let r2 = rng.random_range(0..len);
            let (start, end) = if r1 < r2 { (r1, r2) } else { (r2, r1) };

            let child1 = [&parent1[..start], &parent2[start..end], &parent1[end..]].concat();
            let child2 = [&parent2[..start], &parent1[start..end], &parent2[end..]].concat();

            vec![child1, child2]
        })
        .collect()
}

//PARALEL VERSION
// fn two_point_crossover(pairs: &[(Vec<u8>, Vec<u8>)], rng: &mut ThreadRng) -> Vec<Vec<u8>> {
//     let seed = rng.random::<u64>();
//     pairs.par_iter()
//         .enumerate()
//         .flat_map(|(i, (parent1, parent2))| {
//             let mut local_rng = SmallRng::seed_from_u64(seed + i as u64);
//             let len = parent1.len();
//             let r1 = local_rng.random_range(0..len);
//             let r2 = local_rng.random_range(0..len);
//             let (start, end) = if r1 < r2 { (r1, r2) } else { (r2, r1) };

//             let child1 = [&parent1[..start], &parent2[start..end], &parent1[end..]].concat();
//             let child2 = [&parent2[..start], &parent1[start..end], &parent2[end..]].concat();

//             vec![child1, child2]
//         })
//         .collect()
// }

//SERIAL VERSION
fn mutation(chromosomes: &Vec<Vec<u8>>, rate: f32, rng: &mut ThreadRng) -> Vec<Vec<u8>> {
    chromosomes.iter().map(|chromosome| {
        let mut mutated = chromosome.clone();
        for gene in mutated.iter_mut() {
            if rng.random::<f32>() < rate {
                *gene = if *gene == 0 { 1 } else { 0 };
            }
        }
        mutated
    }).collect()
}

//PARALEL VERSION
// fn mutation(chromosomes: &Vec<Vec<u8>>, rate: f32, rng: &mut ThreadRng) -> Vec<Vec<u8>> {
//     let seed = rng.random::<u64>();
    
//     chromosomes.par_iter()
//     .enumerate()
//     .map(|(i, chromosome)| {
//         let mut mutated = chromosome.clone();
//         let mut local_rng = SmallRng::seed_from_u64(seed + i as u64);
//         for gene in mutated.iter_mut() {
//             if local_rng.random::<f32>() < rate {
//                 *gene = if *gene == 0 { 1 } else { 0 };
//             }
//         }
//         mutated
//     }).collect()
// }

fn elitism(parents: &[Vec<u8>], children: &[Vec<u8>], rate: f32, population_size: usize)  -> Vec<Vec<u8>>{
    let old_ind_size = (population_size as f32 * rate).round() as usize;
    let remaining = population_size - old_ind_size;
    
    [
        &parents[..old_ind_size],
        &children[..remaining]
    ].concat()
}

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

//SERIAL VERSION
// fn rank_chromosomes(
//     population: &[Vec<u8>],
//     problem: &Problem,
// ) -> Vec<(Vec<u8>, f32)> {
//     let mut ranked: Vec<(Vec<u8>, f32)> = population
//         .iter()
//         .map(|chromosome| {
//             let (_, fitness) = decode_chromosome(chromosome, problem);
//             (chromosome.clone(), fitness)
//         })
//         .collect();
    
//     ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
//     ranked
// }

//PARALEL VERSION
fn rank_chromosomes(
    population: &[Vec<u8>],
    problem: &Problem,
) -> Vec<(Vec<u8>, f32)> {
    let mut ranked: Vec<(Vec<u8>, f32)> = population
        .par_iter()
        .map(|chromosome| {
            let (_, fitness) = decode_chromosome(chromosome, problem);
            (chromosome.clone(), fitness)
        })
        .collect();
    
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    ranked
}

fn genetic_algorithm(
    problem: &Problem,
    population_size: usize,
    mutation_rate: f32,
    elitism_rate: f32,
    max_iterations: usize,
    rng: &mut ThreadRng,
) -> (Vec<u8>, f32) {
    let mut population = generate_initial_chromosomes(
        problem.rectangles.len(),
        population_size,
        rng,
    );
    
    let mut best_chromosome = Vec::new();
    let mut best_fitness = 0.0f32;
    
    for _iteration in 0..max_iterations {
        let ranked = rank_chromosomes(&population, problem);

        if ranked[0].1 > best_fitness {
            best_fitness = ranked[0].1;
            best_chromosome = ranked[0].0.clone();
        }
        let parents: Vec<Vec<u8>> = ranked.iter()
            .map(|(chr, _)| chr.clone())
            .collect();
        let pairs = roulette_selection(&parents, rng);
        let children = two_point_crossover(&pairs, rng);
        let mutated_children = mutation(&children, mutation_rate, rng);
        population = elitism(&parents, &mutated_children, elitism_rate, population_size);
    }
    
    (best_chromosome, best_fitness)
}

async fn draw_solution(placed_rects: &[Rect], problem: &Problem) {
    loop {
        clear_background(WHITE);
        
        let padding = 50.0;
        let available_width = screen_width() - (2.0 * padding);
        let available_height = screen_height() - (2.0 * padding);
        
        let scale_x = available_width / problem.bin_width as f32;
        let scale_y = available_height / problem.bin_height as f32;
        let scale = scale_x.min(scale_y);
        
        let bin_pixel_width = problem.bin_width as f32 * scale;
        let bin_pixel_height = problem.bin_height as f32 * scale;
        let offset_x = (screen_width() - bin_pixel_width) / 2.0;
        let offset_y = (screen_height() - bin_pixel_height) / 2.0;
        
        draw_rectangle(
            offset_x, 
            offset_y, 
            bin_pixel_width, 
            bin_pixel_height, 
            LIGHTGRAY
        );
        
        for rect in placed_rects {
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

        draw_rectangle_lines(
            offset_x, 
            offset_y, 
            bin_pixel_width, 
            bin_pixel_height, 
            3.0, 
            BLACK
        );
        
        draw_text(
            &format!("Rectangles: {}/{}", placed_rects.len(), problem.rectangles.len()),
            10.0, 20.0, 20.0, BLACK
        );
        
        draw_text(
            &format!("Bin: {}x{}", problem.bin_width, problem.bin_height),
            10.0, 45.0, 20.0, BLACK
        );
        
        next_frame().await;
    }
}

fn load_problem_from_json(file_path: &str) -> Result<Problem, Box<dyn std::error::Error>> {
    let json_content = fs::read_to_string(file_path)?;
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
    
    println!("Loaded problem: {}", data["Name"].as_str().unwrap_or("Unknown"));
    
    Ok(Problem {
        bin_width,
        bin_height,
        rectangles,
    })
}

fn get_problem() -> Problem {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && args[1] == "--file" {
        if args.len() < 3 {
            eprintln!("Error: --file flag requires a filename");
            eprintln!("Usage: cargo run -- --file <filename.json>");
            std::process::exit(1);
        }
        
        let file_path = &args[2];
        println!("Loading problem from file: {}", file_path);
        
        match load_problem_from_json(file_path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error loading JSON file '{}': {}", file_path, e);
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Error: No input file specified.");
        eprintln!("Usage: cargo run -- --file <filename.json>");
        std::process::exit(1);
    }
}

fn window_config() -> Conf {
    Conf {
        window_title: "2D-OKP-R Genetic Algorithm".to_owned(),
        window_width: 1000, 
        window_height: 800,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_config)]
async fn main() {
    let mut rng = rng();
    let problem = get_problem();
    
    println!("Bin: {}x{}", problem.bin_width, problem.bin_height);
    println!("Rectangles to pack: {}", problem.rectangles.len());
    
    let total_start = Instant::now();

    let (best_chromosome, best_fitness) = genetic_algorithm(
        &problem,
        100,   // population_size
        0.1,  // mutation_rate
        0.1,   // elitism_rate
        200,   // max_iterations
        &mut rng,
    );

    let ga_duration = total_start.elapsed();

    println!("\n=== PERFORMANCE METRICS ===");
    println!("GA execution time: {:.3}s ({} ms)", 
        ga_duration.as_secs_f64(), 
        ga_duration.as_millis()
    );
    
    println!("\n=== FINAL RESULT ===");
    println!("Best fitness: {:.2}%", best_fitness * 100.0);
    println!("Waste percentage: {:.2}%", (1.0 - best_fitness) * 100.0);
    
    
    let (placed_rects, _) = decode_chromosome(&best_chromosome, &problem);
    println!("Placed {} rectangles:", placed_rects.len());
    
    for (i, rect) in placed_rects.iter().enumerate() {
        println!("  Rect {}: pos=({}, {}), size={}x{}", 
            i, rect.x, rect.y, rect.width, rect.height);
    }

    draw_solution(&placed_rects, &problem).await;
}