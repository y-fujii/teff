// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::fmt::Write;
use std::*;

#[derive(Clone)]
struct TileSet {
    tiles: [u8; 34],
}

impl TileSet {
    fn new() -> Self {
        TileSet { tiles: [0; 34] }
    }

    fn tile(&self, i: usize) -> u8 {
        self.tiles[i]
    }

    fn tile_mut(&mut self, i: usize) -> &mut u8 {
        &mut self.tiles[i]
    }

    fn simple(&self, t: usize, i: usize) -> u8 {
        self.tiles[9 * t + i]
    }

    fn simple_mut(&mut self, t: usize, i: usize) -> &mut u8 {
        &mut self.tiles[9 * t + i]
    }

    fn honor(&self, i: usize) -> u8 {
        self.tiles[9 * 3 + i]
    }

    fn honor_mut(&mut self, i: usize) -> &mut u8 {
        &mut self.tiles[9 * 3 + i]
    }

    fn count(&self) -> usize {
        let mut n = 0;
        for i in 0..self.tiles.len() {
            n += self.tile(i) as usize;
        }
        n
    }
}

const WEIGHT_PAIR: usize = 2;
const WEIGHT_TRIAD: usize = 3;

fn count_pair_and_triad(hand: &mut TileSet) -> usize {
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

fn count_triad_simple(hand: &mut TileSet, t: usize, i0: usize) -> usize {
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

fn count_triad_honor(hand: &TileSet) -> usize {
    let mut n_total = 0;
    for i in 0..7 {
        n_total += WEIGHT_TRIAD * (hand.honor(i) as usize / 3);
    }
    n_total
}

fn discard_tile(hand: &mut TileSet, wall: &mut TileSet, depth: usize) -> (f64, Vec<(usize, f64)>) {
    let n_tiles = hand.count();
    let count = count_pair_and_triad(hand);
    if count == n_tiles || depth == 0 {
        return ((n_tiles - count) as f64, Vec::new());
    }

    let mut best_score = f64::MAX;
    let mut discards = Vec::with_capacity(n_tiles);
    for i in 0..hand.tiles.len() {
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

fn draw_tile(hand: &mut TileSet, wall: &mut TileSet, depth: usize) -> f64 {
    let mut sum = 0.0;
    for i in 0..wall.tiles.len() {
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

fn playout<R: rand::Rng>(hand: &TileSet, wall: &TileSet, n_samples: usize, rng: &mut R) -> f64 {
    let n_tiles = hand.count() + 1;
    let mut acc = Vec::new();
    for i in 0..wall.tiles.len() {
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
            if count >= n_tiles && count % 3 == n_tiles % 3 {
                sum += i;
                break;
            }
        }
    }
    sum as f64 / n_samples as f64
}

fn discard_tile_by_playout<R: rand::Rng>(
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
    let n = hand.tiles.iter().filter(|i| **i > 0).count();
    let n_samples = cmp::max(n_samples / n, 1);
    for i in 0..hand.tiles.len() {
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

fn draw_tile_by_playout<R: rand::Rng>(
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
    for i in 0..wall.tiles.len() {
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

fn format_tile(tile: usize) -> String {
    match tile {
        0..=8 => format!("{}m", tile + 1),
        9..=17 => format!("{}p", tile - 8),
        18..=26 => format!("{}s", tile - 17),
        27..=33 => format!("{}z", tile - 26),
        _ => panic!(),
    }
}

fn format_tile_set(hand: &mut TileSet) -> String {
    let mut buf = String::new();
    let mut is_empty = true;

    for t in 0..3 {
        for i in 0..9 {
            for _ in 0..hand.simple(t, i) {
                write!(buf, "{}", i + 1).unwrap();
                is_empty = false;
            }
        }
        if !mem::replace(&mut is_empty, true) {
            write!(buf, "{} ", ['m', 'p', 's'][t]).unwrap();
        }
    }

    for i in 0..7 {
        for _ in 0..hand.honor(i) {
            write!(buf, "{}", i + 1).unwrap();
            is_empty = false;
        }
    }
    if !mem::replace(&mut is_empty, true) {
        write!(buf, "z ").unwrap();
    }

    buf
}

fn parse_tile_set(text: &str) -> Option<TileSet> {
    let mut hand = TileSet::new();
    let mut nums = Vec::new();
    for c in text.chars() {
        match c {
            '1'..='9' => nums.push(c.to_digit(10).unwrap() as usize),
            'm' => {
                for i in nums.drain(..) {
                    *hand.simple_mut(0, i - 1) += 1;
                }
            }
            'p' => {
                for i in nums.drain(..) {
                    *hand.simple_mut(1, i - 1) += 1;
                }
            }
            's' => {
                for i in nums.drain(..) {
                    *hand.simple_mut(2, i - 1) += 1;
                }
            }
            'z' => {
                for i in nums.drain(..) {
                    if i > 7 {
                        return None;
                    }
                    *hand.honor_mut(i - 1) += 1;
                }
            }
            ' ' | '\t' | '\n' => (),
            _ => return None,
        }
    }
    if !nums.is_empty() {
        return None;
    }
    Some(hand)
}

fn generate_random_hand<R: rand::Rng>(rng: &mut R) -> TileSet {
    let mut acc = Vec::new();
    for i in 0..34 {
        for _ in 0..4 {
            acc.push(i);
        }
    }
    rand::seq::SliceRandom::shuffle(&mut acc[..], rng);

    let mut hand = TileSet::new();
    for i in acc[0..14].iter() {
        *hand.tile_mut(*i) += 1;
    }
    hand
}

fn analyze_hand<R: rand::Rng>(hand: &mut TileSet, rng: &mut R) {
    println!("Hand: {}", format_tile_set(hand));

    let n_tiles = hand.count();
    if n_tiles % 3 != 0 && n_tiles % 3 != 2 {
        println!("  # of tiles must be 3n or 3n + 2.");
        return;
    }

    let mut wall = TileSet::new();
    for i in 0..wall.tiles.len() {
        *wall.tile_mut(i) = 4 - hand.tile(i);
    }

    let (score, _) = discard_tile(hand, &mut wall, 0);
    println!("  min-mean # of non-meld tiles, depth = 0:");
    println!("       {:>11.8}", score);
    for i in 1..4 {
        let (_, mut discards) = discard_tile(hand, &mut wall, i);
        discards.sort_by(|(_, s0), (_, s1)| s0.partial_cmp(s1).unwrap());
        println!("  min-mean # of non-meld tiles, depth = {}:", i);
        for (tile, score) in discards {
            println!("    {} {:>11.8}", format_tile(tile), score);
        }
    }

    let n_samples = 1 << 21;
    for i in 0..2 {
        let (_, mut discards) = discard_tile_by_playout(hand, &mut wall, i, n_samples, rng);
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
    let mut rng = rand::thread_rng();
    if env::args().len() <= 1 {
        loop {
            let mut hand = generate_random_hand(&mut rng);
            analyze_hand(&mut hand, &mut rng);
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
            analyze_hand(&mut hand, &mut rng);
        }
    }
}
