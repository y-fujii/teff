// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::tile::*;
use rayon::prelude::*;
use std::*;

const TURN_BIAS: f64 = 0.5;

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

pub fn discard_tile(hand: &mut TileSet, wall: &mut TileSet, depth: usize) -> (f64, Vec<(usize, f64)>) {
    let n_tiles = hand.count();
    let count = count_head_and_triad(hand, true);
    if count == n_tiles || depth == 0 {
        return ((n_tiles - count) as f64, Vec::new());
    }

    let mut best_score = f64::MAX;
    let mut discards = Vec::with_capacity(n_tiles);
    for i in 0..hand.len() {
        if hand.tile(i) > 0 {
            *hand.tile_mut(i) -= 1;
            let score = draw_tile(hand, wall, depth - 1) + TURN_BIAS;
            *hand.tile_mut(i) += 1;
            discards.push((i, score));
            best_score = f64::min(best_score, score);
        }
    }
    (best_score, discards)
}

pub fn discard_tile_parallel(hand: &mut TileSet, wall: &mut TileSet, depth: usize) -> (f64, Vec<(usize, f64)>) {
    let n_tiles = hand.count();
    let count = count_head_and_triad(hand, true);
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
            let score = draw_tile(&mut hand, &mut wall.clone(), depth - 1) + TURN_BIAS;
            *hand.tile_mut(i) += 1;
            Some((i, score))
        })
        .collect();

    let best_score = discards.iter().fold(f64::MAX, |r, e| r.min(e.1));
    (best_score, discards)
}
