// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use teff::search::*;
use teff::tile::*;

fn analyze_hand(hand: &mut TileSet) {
    println!("Hand: {}", format_tile_set(hand));

    let n_tiles = hand.count();
    if n_tiles % 3 != 0 && n_tiles % 3 != 2 {
        println!("  # of tiles must be 3n or 3n + 2.");
        return;
    }

    let mut wall = TileSet::new();
    for i in 0..wall.len() {
        *wall.tile_mut(i) = 4 - hand.tile(i);
    }

    let (score, _) = discard_tile(hand, &mut wall, 0);
    println!("  min-mean # of non-meld tiles, depth = 0:");
    println!("       {:>11.8}", score);
    for i in 1..4 {
        let (_, mut discards) = discard_tile_parallel(hand, &mut wall, i);
        discards.sort_by(|(_, s0), (_, s1)| s0.partial_cmp(s1).unwrap());
        println!("  min-mean # of non-meld tiles, depth = {}:", i);
        for (tile, score) in discards {
            println!("    {} {:>11.8}", format_tile(tile), score);
        }
    }

    let n_samples = 1 << 21;
    for i in 0..2 {
        let (_, mut discards) = discard_tile_by_playout_parallel(hand, &mut wall, i, n_samples);
        discards.sort_by(|(_, s0), (_, s1)| s0.partial_cmp(s1).unwrap());
        println!(
            "  min-mean # of turns to win by playout, depth = {}, n_samples = {}:",
            i, n_samples
        );
        for (tile, score) in discards {
            println!("    {} {:>5.2}", format_tile(tile), score);
        }
    }

    println!();
}

fn main() {
    if env::args().len() <= 1 {
        loop {
            let mut hand = generate_random_hand(&mut rand::thread_rng());
            analyze_hand(&mut hand);
        }
    } else {
        for arg in env::args().skip(1) {
            let mut hand = match parse_tile_set(&arg) {
                Some(t) => t,
                None => {
                    println!("Syntax error: {}", arg);
                    println!();
                    continue;
                }
            };
            analyze_hand(&mut hand);
        }
    }
}
