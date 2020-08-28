// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::playout;
use crate::search;
use crate::tile::*;
use crate::uct;
use std::*;

pub fn analyze_hand(hand: &mut TileSet) {
    println!("Hand: {}", format_tile_set(hand));

    let n_tiles = hand.count();
    if n_tiles % 3 != 2 {
        println!("  # of tiles must be 3n + 2.");
        return;
    }

    let mut wall = TileSet::new();
    for i in 0..wall.len() {
        *wall.tile_mut(i) = 4 - hand.tile(i);
    }

    let (score, _) = search::discard_tile(hand, &mut wall, 0);
    println!("  min-mean # of non-meld tiles, depth = 0:");
    println!("       {:>11.8}", score);
    for i in 1..4 {
        let (_, mut discards) = search::discard_tile_parallel(hand, &mut wall, i);
        discards.sort_by(|(_, s0), (_, s1)| s0.partial_cmp(s1).unwrap());
        println!("  min-mean # of non-meld tiles, depth = {}:", i);
        for (tile, score) in discards {
            println!("    {} {:>11.8}", format_tile(tile), score);
        }
    }

    let n_samples = 1 << 21;
    for i in 0..2 {
        let (_, mut discards) = playout::discard_tile_parallel(hand, &mut wall, i, n_samples);
        discards.sort_by(|(_, s0), (_, s1)| s0.partial_cmp(s1).unwrap());
        println!(
            "  min-mean # of turns to win by playout, depth = {}, n_samples = {}:",
            i, n_samples
        );
        for (tile, score) in discards {
            println!("    {} {:>5.2}", format_tile(tile), score);
        }
    }

    let mut discards = uct::discard_tile(hand, &wall, n_samples, &mut rand::thread_rng());
    discards.sort_by(|(_, s0, _), (_, s1, _)| s1.cmp(s0));
    println!("  min-mean # of turns to win by UCT, n_samples = {}:", n_samples);
    for (tile, _, score) in discards {
        println!("    {} {:>5.2}", format_tile(tile), score);
    }

    println!();
}

pub fn benchmark<R: rand::Rng>(rng: &mut R) {
    let discard_names = ["search", "playout", "uct"];
    let discard_funcs: [&dyn Fn(&mut TileSet, &mut TileSet) -> usize; 3] = [
        &|hand: &mut TileSet, wall: &mut TileSet| {
            let (_, discards) = search::discard_tile_parallel(hand, wall, 3);
            let (tile, _) = discards
                .iter()
                .min_by(|(_, s0), (_, s1)| s0.partial_cmp(s1).unwrap())
                .unwrap();
            *tile
        },
        &|hand: &mut TileSet, wall: &mut TileSet| {
            let (_, discards) = playout::discard_tile_parallel(hand, wall, 1, 1 << 21);
            let (tile, _) = discards
                .iter()
                .min_by(|(_, s0), (_, s1)| s0.partial_cmp(s1).unwrap())
                .unwrap();
            *tile
        },
        &|hand: &mut TileSet, wall: &mut TileSet| {
            let discards = uct::discard_tile(&hand, &wall, 1 << 19, &mut rand::thread_rng());
            let (tile, _, _) = discards.iter().max_by_key(|(_, s, _)| s).unwrap();
            *tile
        },
    ];

    let mut sum_l1 = vec![0; discard_funcs.len()];
    let mut sum_l2 = vec![0; discard_funcs.len()];
    for n_samples in 1.. {
        let mut acc = Vec::new();
        for i in 0..34 {
            for _ in 0..4 {
                acc.push(i);
            }
        }
        rand::seq::SliceRandom::shuffle(&mut acc[..], rng);

        let mut hand = TileSet::new();
        for i in acc.drain(acc.len() - 13..) {
            *hand.tile_mut(i) += 1;
        }

        let mut wall = TileSet::new();
        for i in acc.iter() {
            *wall.tile_mut(*i) += 1;
        }

        for (i, discard) in discard_funcs.iter().enumerate() {
            let mut acc = acc.clone();
            let mut hand = hand.clone();
            let mut wall = wall.clone();
            let mut n_turns = 0;
            while let Some(drawed) = acc.pop() {
                *wall.tile_mut(drawed) -= 1;
                *hand.tile_mut(drawed) += 1;

                if count_head_and_triad(&mut hand, false) >= hand.count() {
                    println!("{} -> {:<22}\n", format_tile(drawed), format_tile_set(&hand));
                    break;
                }

                let discarded = discard(&mut hand, &mut wall);
                println!(
                    "{} -> {:<22}-> {}",
                    format_tile(drawed),
                    format_tile_set(&hand),
                    format_tile(discarded)
                );
                *hand.tile_mut(discarded) -= 1;

                n_turns += 1;
            }
            sum_l1[i] += n_turns;
            sum_l2[i] += n_turns * n_turns;
        }

        println!("N = {}", n_samples);
        for (i, name) in discard_names.iter().enumerate() {
            let mu = sum_l1[i] as f64 / n_samples as f64;
            let s2 = (n_samples * sum_l2[i] - sum_l1[i] * sum_l1[i]) as f64 / (n_samples * (n_samples - 1)) as f64;
            println!("{:>7}: μ = {:>6.3}, σ = {:>6.3}", name, mu, f64::sqrt(s2));
        }
        println!();
    }
}
