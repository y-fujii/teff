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
        return (count as f64, Vec::new());
    }

    let mut best_score = 0.0;
    let mut discards = Vec::with_capacity(n_tiles);
    for i in 0..34 {
        if hand.tile(i) > 0 {
            *hand.tile_mut(i) -= 1;
            let score = draw_tile(hand, wall, depth - 1);
            *hand.tile_mut(i) += 1;
            discards.push((i, score));
            best_score = f64::max(best_score, score);
        }
    }
    (best_score, discards)
}

fn playout(hand: &TileSet, wall: &TileSet, n_samples: usize) -> f64 {
    let n_tiles = hand.count() + 1;
    let mut rng = rand::thread_rng();
    let mut acc = Vec::new();
    for i in 0..34 {
        for _ in 0..wall.tile(i) {
            acc.push(i);
        }
    }
    let mut sum = 0;
    for _ in 0..n_samples {
        let mut hand = hand.clone();
        let mut acc = acc.clone();
        for turn in 0.. {
            let count = count_pair_and_triad(&mut hand);
            if count >= n_tiles && count % 3 == n_tiles % 3 {
                sum += turn;
                break;
            }
            let tile = acc.remove(rand::Rng::gen_range(&mut rng, 0, acc.len()));
            *hand.tile_mut(tile) += 1;
        }
    }
    sum as f64 / n_samples as f64
}

fn discard_tile_by_mc(hand: &mut TileSet, wall: &TileSet, n_samples: usize) -> Vec<(usize, f64)> {
    let mut discards = Vec::new();
    for i in 0..34 {
        if hand.tile(i) > 0 {
            *hand.tile_mut(i) -= 1;
            let score = playout(&hand, &wall, n_samples);
            *hand.tile_mut(i) += 1;
            discards.push((i, score));
        }
    }
    discards
}

fn draw_tile(hand: &mut TileSet, wall: &mut TileSet, depth: usize) -> f64 {
    let mut sum = 0.0;
    for i in 0..34 {
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

fn generate_random_hand() -> TileSet {
    let mut rng = rand::thread_rng();
    let mut hand = TileSet::new();
    for _ in 0..14 {
        *hand.tile_mut(rand::Rng::gen_range(&mut rng, 0, 34)) += 1;
    }
    hand
}

fn analyze_hand(hand: &mut TileSet) {
    println!("Hand: {}", format_tile_set(hand));

    let n_tiles = hand.count();
    if n_tiles % 3 != 0 && n_tiles % 3 != 2 {
        println!("  # of tiles must be 3n or 3n + 2.");
        return;
    }

    let mut wall = TileSet::new();
    for i in 0..34 {
        *wall.tile_mut(i) = 4 - hand.tile(i);
    }

    let (score, _) = discard_tile(hand, &mut wall, 0);
    println!("  max-mean search, depth 0:");
    println!("       {:>11.8}", score);
    for i in 1..4 {
        let (_, mut discards) = discard_tile(hand, &mut wall, i);
        discards.sort_by(|(_, s0), (_, s1)| s1.partial_cmp(s0).unwrap());
        println!("  max-mean search, depth {}:", i);
        for (tile, score) in discards {
            println!("    {} {:>11.8}", format_tile(tile), score);
        }
    }

    let n_samples = 65536;
    let mut discards = discard_tile_by_mc(hand, &wall, n_samples);
    discards.sort_by(|(_, s0), (_, s1)| s0.partial_cmp(s1).unwrap());
    println!("  mc playout, {} samples:", n_samples);
    for (tile, score) in discards.iter() {
        println!("    {} {:>7.4}", format_tile(*tile), score);
    }

    println!();
}

fn main() {
    if env::args().len() <= 1 {
        loop {
            let mut hand = generate_random_hand();
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
