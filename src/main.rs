use std::str;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    card::{Card, CardSet, Rank, Suit},
    evaluator::evaluate,
    game::play,
    strategies::{FlushNaive, GeneticStrategy, HaydenSimple, HypergeometricStrategy},
};

mod card;
mod evaluator;
mod game;
mod strategies;

const N_STRATEGIES: usize = 50;
const N_GAMES_PER_STRATEGY: usize = 10000;
const N_STRATEGIES_TO_KEEP: usize = N_STRATEGIES / 5;
const START_STD: f32 = 0.3;
const STARTING_STRATEGY: Option<GeneticStrategy> = None;
const N_GENERATIONS: usize = 300;

fn strategy_fitness(strategy: &GeneticStrategy, n_games: usize) -> f32 {
    let mut wins = 0;
    for _ in 0..n_games {
        if play(strategy.clone()) {
            wins += 1;
        }
    }
    wins as f32 / n_games as f32
}

fn std_dev(generation: usize) -> f32 {
    START_STD * (-(generation as f32) / 50.0).exp()
}

// fn main() {
//     let mut strategies = (0..N_STRATEGIES)
//         .map(|_| {
//             STARTING_STRATEGY
//                 .clone()
//                 .map(|s| s.mutate(0.05))
//                 .unwrap_or_else(GeneticStrategy::random)
//         })
//         .collect::<Vec<_>>();

//     for generation in 0..N_GENERATIONS {
//         let mut fitnesses = strategies
//             .par_iter()
//             .map(|s| {
//                 (
//                     s,
//                     strategy_fitness(s, N_GAMES_PER_STRATEGY + generation * 300),
//                 )
//             })
//             .collect::<Vec<_>>();

//         fitnesses.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
//         println!(
//             "Generation {}/{}: Best fitness: {:.2}%",
//             generation + 1,
//             N_GENERATIONS,
//             fitnesses[0].1 * 100.0
//         );
//         println!("Best strategy: {:?}", fitnesses[0].0);

//         let mut new_strategies = vec![];
//         for i in 0..N_STRATEGIES_TO_KEEP {
//             new_strategies.push(fitnesses[i].0.clone());
//             for _ in 0..(N_STRATEGIES / N_STRATEGIES_TO_KEEP - 1) {
//                 new_strategies.push(fitnesses[i].0.clone().mutate(std_dev(generation)));
//             }
//         }
//         strategies = new_strategies;
//     }
// }

fn main() {
    println!("Initialising hand ranks...");
    evaluate(CardSet(0b11111));

    println!("Playing games...");

    let start = std::time::Instant::now();

    // Create multiple threads to play games in parallel
    // As many threads as available or 8
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8);
    let num_games_per_thread = 1000;
    let mut handles = vec![];
    for _ in 0..num_threads {
        handles.push(std::thread::spawn(move || {
            let mut wins = 0;
            let mut total = 0;
            for _ in 0..num_games_per_thread {
                // if play(GeneticStrategy::new(
                //     2.978385,
                //     0.002643389,
                //     0.07573282,
                //     -0.07551803,
                //     0.13757019,
                //     0.68877584,
                //     CardSet(4503595788627312),
                // )) {
                if play(HypergeometricStrategy) {
                    wins += 1;
                }
                total += 1;
            }
            (wins, total)
        }));
    }

    // Collect results from threads
    let mut wins = 0;
    let mut total = 0;
    for handle in handles {
        let (thread_wins, thread_total) = match handle.join() {
            Err(e) => {
                eprintln!("Thread panicked: {:?}", e);
                continue;
            }
            Ok(result) => result,
        };
        wins += thread_wins;
        total += thread_total;
    }

    println!(
        "Time taken: {:.2} seconds ({} games/second across {} threads)",
        start.elapsed().as_secs_f64(),
        (total as f64 / start.elapsed().as_secs_f64()) as u32,
        num_threads
    );

    println!(
        "Win rate: {:.2}% over {} games",
        wins as f64 / total as f64 * 100.0,
        total
    );
}
