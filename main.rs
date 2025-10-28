use rand::{rng, rngs::ThreadRng, Rng};

fn cost_function(chromosome: &Vec<u8>) -> f64 {
    chromosome.iter().map(|&gene| gene as f64).sum()
}   

fn generate_chromosome(len: usize, rng: &mut ThreadRng) -> Vec<u8> {
    let mut chromosome = vec![1u8; len];
    let zeros = len / 10;
    
    (0..zeros).for_each(|_| {
        chromosome[rng.random_range(0..len)] = 0;
    });
    
    chromosome
}

fn generate_initial_chromosomes(size: usize, count: usize) -> Vec<Vec<u8>> {
    let mut rng = rng();
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

fn elitism(parents: &[Vec<u8>], children: &[Vec<u8>], rate: f32, population_size: usize)  -> Vec<Vec<u8>>{
    let old_ind_size = (population_size as f32 * rate).round() as usize;
    let remaining = population_size - old_ind_size;
    
    [
        &parents[..old_ind_size],
        &children[..remaining]
    ].concat()
}

fn restrictions(r1: &[i32; 4], r2: &[i32; 4], w: i32, h: i32) -> bool {
    r1[0] + r1[2] <= w 
        && r1[1] + r1[3] <= h
        && (r1[0] - (r2[0] + r2[2]))
            .max(r2[0] - (r1[0] + r1[2]))
            .max(r1[1] - (r2[1] + r2[3]))
            .max(r2[1] - (r1[1] + r1[3])) >= 0
}


fn rank_chromosomes() {}
fn genetic_algorithm() {}

fn decode_chromosome() {}

fn main() {
    let mut rng = rng();
    let initial_population = generate_initial_chromosomes(100, 10);

    let pairs = roulette_selection(&initial_population, &mut rng);
    let children = two_point_crossover(&pairs, &mut rng);

    println!("Generated {} children", children.len());
}