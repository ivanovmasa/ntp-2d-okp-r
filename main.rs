use macroquad::prelude::*;
use ::rand::prelude::*;
use ::rand::rng;
use ::rand::rngs::ThreadRng;
use rayon::prelude::*;
use ::rand::SeedableRng; 
use ::rand::rngs::SmallRng;
use std::time::Instant;

#[derive(Clone, Copy, Debug)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Clone, Debug)]
struct FreeRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl FreeRect {
    fn area(&self) -> i32 {
        self.width * self.height
    }
    
    fn contains(&self, rect_width: i32, rect_height: i32) -> bool {
        self.width >= rect_width && self.height >= rect_height
    }
}

struct Problem {
    bin_width: i32,
    bin_height: i32,
    rectangles: Vec<(i32, i32)>, 
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

fn decode_chromosome(
    chromosome: &[u8],
    problem: &Problem,
) -> (Vec<Rect>, f32) {
    let mut placed_rects: Vec<Rect> = Vec::new();
    let mut free_rects: Vec<FreeRect> = vec![FreeRect {
        x: 0,
        y: 0,
        width: problem.bin_width,
        height: problem.bin_height,
    }];
    
    for (i, &gene) in chromosome.iter().enumerate() {
        if gene == 0 || i >= problem.rectangles.len() {
            continue;
        }
        
        let (rect_width, rect_height) = problem.rectangles[i];
        
        if let Some((best_idx, placed_rect)) = find_best_area_fit(
            &free_rects,
            rect_width,
            rect_height,
        ) {
            placed_rects.push(placed_rect);
            split_free_rect(&mut free_rects, best_idx, &placed_rect);
            prune_free_rects(&mut free_rects);
        }
    }
    
    let total_area = (problem.bin_width * problem.bin_height) as f32;
    let used_area: i32 = placed_rects.iter()
        .map(|r| r.width * r.height)
        .sum();
    let fitness = (used_area as f32) / total_area;
    
    (placed_rects, fitness)
}

fn find_best_area_fit(
    free_rects: &[FreeRect],
    rect_width: i32,
    rect_height: i32,
) -> Option<(usize, Rect)> {
    let mut best_idx = None;
    let mut best_area_diff = i32::MAX;
    let mut best_rect = None;
    
    for (idx, free_rect) in free_rects.iter().enumerate() {
        if free_rect.contains(rect_width, rect_height) {
            let area_diff = free_rect.area() - (rect_width * rect_height);
            
            if area_diff < best_area_diff {
                best_area_diff = area_diff;
                best_idx = Some(idx);
                best_rect = Some(Rect {
                    x: free_rect.x,
                    y: free_rect.y,
                    width: rect_width,
                    height: rect_height,
                });
            }
        }
        
        if free_rect.contains(rect_height, rect_width) {
            let area_diff = free_rect.area() - (rect_width * rect_height);
            
            if area_diff < best_area_diff {
                best_area_diff = area_diff;
                best_idx = Some(idx);
                best_rect = Some(Rect {
                    x: free_rect.x,
                    y: free_rect.y,
                    width: rect_height, 
                    height: rect_width,  
                });
            }
        }
    }
    
    best_idx.and_then(|idx| best_rect.map(|rect| (idx, rect)))
}

fn split_free_rect(free_rects: &mut Vec<FreeRect>, used_idx: usize, placed: &Rect) {
    let used_rect = free_rects.remove(used_idx);
    
    let mut new_rects = Vec::new();
    
    if used_rect.x + used_rect.width > placed.x + placed.width {
        new_rects.push(FreeRect {
            x: placed.x + placed.width,
            y: used_rect.y,
            width: used_rect.width - placed.width,
            height: used_rect.height,
        });
    }
    
    if used_rect.y + used_rect.height > placed.y + placed.height {
        new_rects.push(FreeRect {
            x: used_rect.x,
            y: placed.y + placed.height,
            width: used_rect.width,
            height: used_rect.height - placed.height,
        });
    }
    
    free_rects.extend(new_rects);
}

fn prune_free_rects(free_rects: &mut Vec<FreeRect>) {
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

fn is_contained_in(rect1: &FreeRect, rect2: &FreeRect) -> bool {
    rect1.x >= rect2.x
        && rect1.y >= rect2.y
        && rect1.x + rect1.width <= rect2.x + rect2.width
        && rect1.y + rect1.height <= rect2.y + rect2.height
}

//SERIAL VERSION
fn rank_chromosomes(
    population: &[Vec<u8>],
    problem: &Problem,
) -> Vec<(Vec<u8>, f32)> {
    let mut ranked: Vec<(Vec<u8>, f32)> = population
        .iter()
        .map(|chromosome| {
            let (_, fitness) = decode_chromosome(chromosome, problem);
            (chromosome.clone(), fitness)
        })
        .collect();
    
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    ranked
}

//PARALEL VERSION
// fn rank_chromosomes(
//     population: &[Vec<u8>],
//     problem: &Problem,
// ) -> Vec<(Vec<u8>, f32)> {
//     let mut ranked: Vec<(Vec<u8>, f32)> = population
//         .par_iter()
//         .map(|chromosome| {
//             let (_, fitness) = decode_chromosome(chromosome, problem);
//             (chromosome.clone(), fitness)
//         })
//         .collect();
    
//     ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
//     ranked
// }

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
    
    for iteration in 0..max_iterations {
        let ranked = rank_chromosomes(&population, problem);

        if ranked[0].1 > best_fitness {
            best_fitness = ranked[0].1;
            best_chromosome = ranked[0].0.clone();
            println!("Iteration {}: Best fitness = {:.2}%", iteration, best_fitness * 100.0);
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
    let scale = 4.0;
    
    loop {
        clear_background(WHITE);
        
        draw_rectangle(
            0.0, 
            0.0, 
            problem.bin_width as f32 * scale, 
            problem.bin_height as f32 * scale, 
            LIGHTGRAY
        );
        
        for rect in placed_rects {
            draw_rectangle(
                rect.x as f32 * scale,
                rect.y as f32 * scale,
                rect.width as f32 * scale,
                rect.height as f32 * scale,
                PURPLE,
            );
            
            draw_rectangle_lines(
                rect.x as f32 * scale,
                rect.y as f32 * scale,
                rect.width as f32 * scale,
                rect.height as f32 * scale,
                2.0,
                DARKBLUE,
            );
        }

        draw_rectangle_lines(
            0.0, 0.0, 
            problem.bin_width as f32 * scale, 
            problem.bin_height as f32 * scale, 
            3.0, 
            BLACK
        );
        
        draw_text(
            &format!("Rectangles: {}/{}", placed_rects.len(), problem.rectangles.len()),
            10.0, 20.0, 20.0, BLACK
        );
        
        next_frame().await;
    }
}

fn window_config() -> Conf {
    let scale = 4.0;
    let bin_width = 150;
    let bin_height = 80;
    let padding = 100; 
    
    Conf {
        window_title: "2D-OKP-R Genetic Algorithm".to_owned(),
        window_width: (bin_width as f32 * scale + padding as f32) as i32,
        window_height: (bin_height as f32 * scale + padding as f32) as i32,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_config)]
async fn main() {
    let mut rng = rng();

    let problem = Problem {
        bin_width: 150,
        bin_height: 80,
        rectangles: vec![
            (50, 30),
            (40, 40),
            (30, 20),
            (60, 25),
            (35, 35),
            (45, 50),
            (25, 25),
            (55, 30),
        ],
    };
    
    println!("Starting genetic algorithm...");
    println!("Bin: {}x{}", problem.bin_width, problem.bin_height);
    println!("Rectangles to pack: {}", problem.rectangles.len());
    
    let total_start = Instant::now();

    let (best_chromosome, best_fitness) = genetic_algorithm(
        &problem,
        100,   // population_size
        0.05,  // mutation_rate
        0.1,   // elitism_rate
        100,   // max_iterations
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