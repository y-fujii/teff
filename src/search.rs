// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::tile::*;
use rayon::prelude::*;
use std::*;

const WEIGHT_PAIR: usize = 2;
const WEIGHT_TRIAD: usize = 3;

pub fn count_pair_and_triad(hand: &mut TileSet) -> usize {
    let n_simples = [
        count_triad_simple(hand, 0, 0),
        count_triad_simple(hand, 1, 0),
        count_triad_simple(hand, 2, 0),
    ];
    let n_honor = count_triad_honor(hand);

    let mut n_total = n_simples[0] + n_simples[1] + n_simples[2] + n_honor;
    for t in 0..3 {
        let n_others = n_simples[(t + 1) % 3] + n_simples[(t + 2) % 3] + n_honor + WEIGHT_PAIR;
        for i in 0..9 {
            if hand.simple(t, i) >= 2 {
                *hand.simple_mut(t, i) -= 2;
                n_total = cmp::max(n_total, count_triad_simple(hand, t, 0) + n_others);
                *hand.simple_mut(t, i) += 2;
            }
        }
    }

    let n_others = n_simples[0] + n_simples[1] + n_simples[2] + WEIGHT_PAIR;
    for i in 0..7 {
        if hand.honor(i) >= 2 {
            *hand.honor_mut(i) -= 2;
            n_total = cmp::max(n_total, count_triad_honor(hand) + n_others);
            *hand.honor_mut(i) += 2;
        }
    }

    n_total
}

pub fn count_triad_simple(hand: &mut TileSet, t: usize, i0: usize) -> usize {
    let mut n_total = 0;
    for i in i0..7 {
        if hand.simple(t, i + 0) > 0 && hand.simple(t, i + 1) > 0 && hand.simple(t, i + 2) > 0 {
            *hand.simple_mut(t, i + 0) -= 1;
            *hand.simple_mut(t, i + 1) -= 1;
            *hand.simple_mut(t, i + 2) -= 1;
            n_total = cmp::max(n_total, count_triad_simple(hand, t, i) + WEIGHT_TRIAD);
            *hand.simple_mut(t, i + 0) += 1;
            *hand.simple_mut(t, i + 1) += 1;
            *hand.simple_mut(t, i + 2) += 1;
        }
    }
    for i in i0..9 {
        if hand.simple(t, i) >= 3 {
            *hand.simple_mut(t, i) -= 3;
            n_total = cmp::max(n_total, count_triad_simple(hand, t, i) + WEIGHT_TRIAD);
            *hand.simple_mut(t, i) += 3;
        }
    }
    n_total
}

pub fn count_triad_honor(hand: &TileSet) -> usize {
    let mut n_total = 0;
    for i in 0..7 {
        n_total += WEIGHT_TRIAD * (hand.honor(i) as usize / 3);
    }
    n_total
}

pub fn discard_tile(hand: &mut TileSet, wall: &mut TileSet, depth: usize) -> (f64, Vec<(usize, f64)>) {
    let n_tiles = hand.count();
    let count = count_pair_and_triad(hand);
    if count == n_tiles || depth == 0 {
        return ((n_tiles - count) as f64, Vec::new());
    }

    let mut best_score = f64::MAX;
    let mut discards = Vec::with_capacity(n_tiles);
    for i in 0..hand.len() {
        if hand.tile(i) > 0 {
            *hand.tile_mut(i) -= 1;
            let score = draw_tile(hand, wall, depth - 1);
            *hand.tile_mut(i) += 1;
            discards.push((i, score));
            best_score = f64::min(best_score, score);
        }
    }
    (best_score, discards)
}

pub fn discard_tile_parallel(hand: &mut TileSet, wall: &mut TileSet, depth: usize) -> (f64, Vec<(usize, f64)>) {
    let n_tiles = hand.count();
    let count = count_pair_and_triad(hand);
    if count == n_tiles || depth == 0 {
        return ((n_tiles - count) as f64, Vec::new());
    }

    let discards: Vec<_> = (0..hand.len())
        .into_par_iter()
        .filter_map(|i| {
            if hand.tile(i) == 0 {
                return None;
            }
            let mut hand = hand.clone();
            *hand.tile_mut(i) -= 1;
            let score = draw_tile(&mut hand, &mut wall.clone(), depth - 1);
            *hand.tile_mut(i) += 1;
            Some((i, score))
        })
        .collect();

    let best_score = discards.iter().fold(f64::MAX, |r, e| r.min(e.1));
    (best_score, discards)
}

pub fn draw_tile(hand: &mut TileSet, wall: &mut TileSet, depth: usize) -> f64 {
    let mut sum = 0.0;
    for i in 0..wall.len() {
        if wall.tile(i) > 0 {
            *wall.tile_mut(i) -= 1;
            *hand.tile_mut(i) += 1;
            let (score, _) = discard_tile(hand, wall, depth);
            *hand.tile_mut(i) -= 1;
            *wall.tile_mut(i) += 1;
            sum += score * wall.tile(i) as f64;
        }
    }
    sum / wall.count() as f64
}

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
            let count = count_pair_and_triad(&mut hand);
            if count >= n_tiles && (n_tiles % 3 == 0 || count % 3 != 0) {
                sum += i;
                break;
            }
        }
    }
    sum as f64 / n_samples as f64
}

pub fn discard_tile_by_playout<R: rand::Rng>(
    hand: &mut TileSet,
    wall: &mut TileSet,
    depth: usize,
    n_samples: usize,
    rng: &mut R,
) -> (f64, Vec<(usize, f64)>) {
    let count = count_pair_and_triad(hand);
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
            let score = draw_tile_by_playout(hand, wall, depth, n_samples, rng) + 1.0;
            *hand.tile_mut(i) += 1;
            discards.push((i, score));
            best_score = f64::min(best_score, score);
        }
    }
    (best_score, discards)
}

pub fn discard_tile_by_playout_parallel(
    hand: &mut TileSet,
    wall: &mut TileSet,
    depth: usize,
    n_samples: usize,
) -> (f64, Vec<(usize, f64)>) {
    let count = count_pair_and_triad(hand);
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
            let score =
                draw_tile_by_playout(&mut hand, &mut wall.clone(), depth, n_samples, &mut rand::thread_rng()) + 1.0;
            *hand.tile_mut(i) += 1;
            Some((i, score))
        })
        .collect();

    let best_score = discards.iter().fold(f64::MAX, |r, e| r.min(e.1));
    (best_score, discards)
}

pub fn draw_tile_by_playout<R: rand::Rng>(
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
            let (score, _) = discard_tile_by_playout(hand, wall, depth - 1, wall.tile(i) as usize * n_samples, rng);
            *hand.tile_mut(i) -= 1;
            *wall.tile_mut(i) += 1;
            sum += score * wall.tile(i) as f64;
        }
    }
    sum / wall.count() as f64
}
