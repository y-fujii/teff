// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::tile::*;
use rayon::prelude::*;
use std::*;

pub fn playout<R: rand::Rng>(hand: &TileSet, wall: &TileSet, n_samples: usize, rng: &mut R) -> f64 {
    let n_tiles = hand.count() + 1;
    let mut acc = Vec::new();
    for i in 0..wall.len() {
        for _ in 0..wall.tile(i) {
            acc.push(i);
        }
    }
    let mut sum = 0;
    for _ in 0..n_samples {
        // XXX
        rand::seq::SliceRandom::shuffle(&mut acc[..], rng);
        let mut hand = hand.clone();
        for i in 0.. {
            *hand.tile_mut(acc[i]) += 1;
            if count_head_and_triad(&mut hand, false) >= n_tiles {
                sum += i;
                break;
            }
        }
    }
    sum as f64 / n_samples as f64
}

pub fn draw_tile<R: rand::Rng>(
    hand: &mut TileSet,
    wall: &mut TileSet,
    depth: usize,
    n_samples: usize,
    rng: &mut R,
) -> f64 {
    if depth == 0 {
        return playout(hand, wall, n_samples, rng);
    }

    let n_samples = cmp::max(n_samples / wall.count(), 1);
    let mut sum = 0.0;
    for i in 0..wall.len() {
        if wall.tile(i) > 0 {
            *wall.tile_mut(i) -= 1;
            *hand.tile_mut(i) += 1;
            let (score, _) = discard_tile(hand, wall, depth - 1, wall.tile(i) as usize * n_samples, rng);
            *hand.tile_mut(i) -= 1;
            *wall.tile_mut(i) += 1;
            sum += score * wall.tile(i) as f64;
        }
    }
    sum / wall.count() as f64
}

pub fn discard_tile<R: rand::Rng>(
    hand: &mut TileSet,
    wall: &mut TileSet,
    depth: usize,
    n_samples: usize,
    rng: &mut R,
) -> (f64, Vec<(usize, f64)>) {
    let count = count_head_and_triad(hand, false);
    if count == hand.count() {
        return (0.0, Vec::new());
    }

    // XXX: use UCB?
    let mut best_score = f64::MAX;
    let mut discards = Vec::new();
    let n = hand.iter().filter(|i| **i > 0).count();
    let n_samples = cmp::max(n_samples / n, 1);
    for i in 0..hand.len() {
        if hand.tile(i) > 0 {
            *hand.tile_mut(i) -= 1;
            let score = draw_tile(hand, wall, depth, n_samples, rng) + 1.0;
            *hand.tile_mut(i) += 1;
            discards.push((i, score));
            best_score = f64::min(best_score, score);
        }
    }
    (best_score, discards)
}

pub fn discard_tile_parallel(
    hand: &mut TileSet,
    wall: &mut TileSet,
    depth: usize,
    n_samples: usize,
) -> (f64, Vec<(usize, f64)>) {
    let count = count_head_and_triad(hand, false);
    if count == hand.count() {
        return (0.0, Vec::new());
    }

    // XXX: use UCB?
    let n = hand.iter().filter(|i| **i > 0).count();
    let n_samples = cmp::max(n_samples / n, 1);
    let discards: Vec<_> = (0..hand.len())
        .into_par_iter()
        .filter_map(|i| {
            if hand.tile(i) == 0 {
                return None;
            }
            let mut hand = hand.clone();
            *hand.tile_mut(i) -= 1;
            let score = draw_tile(&mut hand, &mut wall.clone(), depth, n_samples, &mut rand::thread_rng()) + 1.0;
            *hand.tile_mut(i) += 1;
            Some((i, score))
        })
        .collect();

    let best_score = discards.iter().fold(f64::MAX, |r, e| r.min(e.1));
    (best_score, discards)
}
