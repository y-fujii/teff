// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::tile::*;
use std::*;

struct DiscardNode {
    count: usize,
    value: usize,
    children: [Option<Box<DrawNode>>; 34],
}

struct DrawNode {
    count: usize,
    value: usize,
    children: [Option<Box<DiscardNode>>; 34],
}

impl DiscardNode {
    fn new() -> Self {
        DiscardNode {
            count: 0,
            value: 0,
            children: arr_macro::arr![None; 34],
        }
    }

    fn sample(&mut self, hand: &mut TileSet, wall: &mut Vec<usize>) -> usize {
        if self.count == usize::MAX {
            return 0;
        }
        let n_tiles = hand.count();
        if count_head_and_triad(hand, false) >= n_tiles {
            self.count = usize::MAX;
            self.value = 0;
            return 0;
        }

        let v = 1 + if self.count < n_tiles {
            playout(hand, wall, n_tiles)
        } else {
            let t = f64::sqrt(2.0) * n_tiles as f64 * f64::sqrt(f64::ln((self.count - n_tiles) as f64));
            let mut min_tile = usize::MAX;
            let mut min_score = f64::INFINITY;
            for i in 0..34 {
                if hand.tile(i) == 0 {
                    continue;
                }
                let score = match self.children[i] {
                    Some(ref n) => n.value as f64 / n.count as f64 - t / f64::sqrt(n.count as f64),
                    None => -f64::INFINITY,
                };
                if score < min_score {
                    min_tile = i;
                    min_score = score;
                }
            }

            *hand.tile_mut(min_tile) -= 1;
            self.children[min_tile]
                .get_or_insert_with(|| Box::new(DrawNode::new()))
                .sample(hand, wall)
        };
        self.count += 1;
        self.value += v;
        v
    }
}

impl DrawNode {
    fn new() -> Self {
        DrawNode {
            count: 0,
            value: 0,
            children: arr_macro::arr![None; 34],
        }
    }

    fn sample(&mut self, hand: &mut TileSet, wall: &mut Vec<usize>) -> usize {
        let tile = wall.pop().unwrap();
        *hand.tile_mut(tile) += 1;
        let v = self.children[tile]
            .get_or_insert_with(|| Box::new(DiscardNode::new()))
            .sample(hand, wall);
        self.count += 1;
        self.value += v;
        v
    }
}

fn playout(hand: &mut TileSet, wall: &mut Vec<usize>, n_tiles: usize) -> usize {
    for (i, tile) in wall.iter().rev().enumerate() {
        *hand.tile_mut(*tile) += 1;
        if count_head_and_triad(hand, false) >= n_tiles {
            return i;
        }
    }
    unreachable!();
}

pub fn discard_tile<R: rand::Rng>(
    hand: &TileSet,
    wall: &TileSet,
    n_samples: usize,
    rng: &mut R,
) -> Vec<(usize, usize, f64)> {
    let mut root = DiscardNode::new();

    let mut acc = Vec::new();
    for _ in 0..n_samples {
        acc.clear();
        for i in 0..wall.len() {
            for _ in 0..wall.tile(i) {
                acc.push(i);
            }
        }
        rand::seq::SliceRandom::shuffle(&mut acc[..], rng);
        root.sample(&mut hand.clone(), &mut acc);
    }

    let mut discards = Vec::new();
    for (i, node) in root.children.iter().enumerate() {
        let node = match node {
            Some(n) => n,
            None => continue,
        };
        let score = node.value as f64 / node.count as f64;
        discards.push((i, node.count, score));
    }
    discards
}
