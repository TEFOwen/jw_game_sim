use crate::{
    card::{Card, Rank, Suit},
    evaluator::evaluate,
    game::play,
    strategies::{FlushNaive, HaydenSimple},
};

mod card;
mod evaluator;
mod game;
mod strategies;

fn main() {
    println!("Initialising hand ranks...");
    evaluate(&[
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::King, Suit::Spades),
        Card::new(Rank::Queen, Suit::Spades),
        Card::new(Rank::Jack, Suit::Spades),
        Card::new(Rank::Ten, Suit::Spades),
    ]);

    println!("Playing games...");

    let start = std::time::Instant::now();

    // Create multiple threads to play games in parallel
    // As many threads as available or 8
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8);
    let num_games_per_thread = 1000000;
    let mut handles = vec![];
    for _ in 0..num_threads {
        handles.push(std::thread::spawn(move || {
            let mut wins = 0;
            let mut total = 0;
            for _ in 0..num_games_per_thread {
                if play::<HaydenSimple>() {
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
