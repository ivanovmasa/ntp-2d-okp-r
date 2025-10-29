use ::rand::prelude::*;
use ::rand::rngs::ThreadRng;
use rayon::prelude::*;
//use ::rand::SeedableRng; 
//use ::rand::rngs::SmallRng;

use crate::{Problem, decode_chromosome};

pub fn generate_chromosome(len: usize, rng: &mut ThreadRng) -> Vec<u8> {
    let mut chromosome = vec![1u8; len];
    let zeros = len / 10;
    
    (0..zeros).for_each(|_| {
        chromosome[rng.random_range(0..len)] = 0;
    });
    
    chromosome
}

pub fn generate_initial_chromosomes(size: usize, count: usize, mut rng: &mut ThreadRng) -> Vec<Vec<u8>> {
    (0..count)
        .map(|_| generate_chromosome(size, &mut rng))
        .collect()
}


pub fn roulette_selection(parents: &[Vec<u8>], rng: &mut ThreadRng) -> Vec<(Vec<u8>, Vec<u8>)> {
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
pub fn two_point_crossover(pairs: &[(Vec<u8>, Vec<u8>)], rng: &mut ThreadRng) -> Vec<Vec<u8>> {
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
//pub fn two_point_crossover(pairs: &[(Vec<u8>, Vec<u8>)], rng: &mut ThreadRng) -> Vec<Vec<u8>> {
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
pub fn mutation(chromosomes: &Vec<Vec<u8>>, rate: f32, rng: &mut ThreadRng) -> Vec<Vec<u8>> {
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
// pub fn mutation(chromosomes: &Vec<Vec<u8>>, rate: f32, rng: &mut ThreadRng) -> Vec<Vec<u8>> {
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

pub fn elitism(parents: &[Vec<u8>], children: &[Vec<u8>], rate: f32, population_size: usize)  -> Vec<Vec<u8>>{
    let old_ind_size = (population_size as f32 * rate).round() as usize;
    let remaining = population_size - old_ind_size;
    
    [
        &parents[..old_ind_size],
        &children[..remaining]
    ].concat()
}

//SERIAL VERSION
// pub fn rank_chromosomes(
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
pub fn rank_chromosomes(
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

pub fn genetic_algorithm(
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