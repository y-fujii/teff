// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::fmt::Write;
use std::*;

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
        count_triad_simple(hand, 0),
        count_triad_simple(hand, 1),
        count_triad_simple(hand, 2),
    ];
    let n_honor = count_triad_honor(hand);

    let mut n_total = n_simples[0] + n_simples[1] + n_simples[2] + n_honor;
    for t in 0..3 {
        let n_others = n_simples[(t + 1) % 3] + n_simples[(t + 2) % 3] + WEIGHT_PAIR;
        for i in 0..9 {
            if hand.simple(t, i) >= 2 {
                *hand.simple_mut(t, i) -= 2;
                n_total = cmp::max(n_total, count_triad_simple(hand, t) + n_others);
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

fn count_triad_simple(hand: &mut TileSet, t: usize) -> usize {
    let mut n_total = 0;
    for i in 0..7 {
        if hand.simple(t, i + 0) > 0 && hand.simple(t, i + 1) > 0 && hand.simple(t, i + 2) > 0 {
            *hand.simple_mut(t, i + 0) -= 1;
            *hand.simple_mut(t, i + 1) -= 1;
            *hand.simple_mut(t, i + 2) -= 1;
            n_total = cmp::max(n_total, count_triad_simple(hand, t) + WEIGHT_TRIAD);
            *hand.simple_mut(t, i + 0) += 1;
            *hand.simple_mut(t, i + 1) += 1;
            *hand.simple_mut(t, i + 2) += 1;
        }
    }
    for i in 0..9 {
        if hand.simple(t, i) >= 3 {
            *hand.simple_mut(t, i) -= 3;
            n_total = cmp::max(n_total, count_triad_simple(hand, t) + WEIGHT_TRIAD);
            *hand.simple_mut(t, i) += 3;
        }
    }
    n_total
}

fn count_triad_honor(hand: &mut TileSet) -> usize {
    let mut n_total = 0;
    for i in 0..7 {
        if hand.honor(i) >= 3 {
            *hand.honor_mut(i) -= 3;
            n_total = cmp::max(n_total, count_triad_honor(hand) + WEIGHT_TRIAD);
            *hand.honor_mut(i) += 3;
        }
    }
    n_total
}

fn discard_tile(hand: &mut TileSet, wall: &mut TileSet, depth: usize) -> (f64, Vec<usize>) {
    let count = count_pair_and_triad(hand);
    if count == hand.count() || depth == 0 {
        return (count as f64, Vec::new());
    }

    let mut best_score = 0.0;
    let mut best_tile = Vec::new();
    for i in 0..34 {
        if hand.tile(i) > 0 {
            *hand.tile_mut(i) -= 1;
            let score = draw_tile(hand, wall, depth - 1);
            *hand.tile_mut(i) += 1;
            if score >= best_score {
                if score > best_score {
                    best_score = score;
                    best_tile.clear();
                }
                best_tile.push(i);
            }
        }
    }
    (best_score, best_tile)
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
    Some(hand)
}

fn main() {
    for arg in env::args().skip(1) {
        let hand = parse_tile_set(&arg);
        let mut hand = match hand {
            Some(t) => t,
            None => {
                println!("Syntax error: {}", arg);
                println!();
                continue;
            }
        };

        println!("Hand: {}", format_tile_set(&mut hand));
        let mut wall = TileSet::new();
        for i in 0..34 {
            *wall.tile_mut(i) = 4 - hand.tile(i);
        }
        for i in 0..4 {
            let (score, tiles) = discard_tile(&mut hand, &mut wall, i);
            print!("depth = {} | score = {:.2} |", i, score);
            for tile in tiles {
                print!(" {}", format_tile(tile));
            }
            println!();
        }
        println!();
    }
}

/*
fn main() -> Result<(), Box<dyn error::Error>> {
    let mut rng = rand::thread_rng();

    loop {
        let mut hand = TileSet::new();
        for _ in 0..14 {
            *hand.tile_mut(rand::Rng::gen_range(&mut rng, 0, 27)) += 1;
        }

        println!("Hand: {}", format_tile_set(&mut hand));
        let mut wall = TileSet::new();
        for i in 0..34 {
            *wall.tile_mut(i) = 4 - hand.tile(i);
        }
        for i in 0..3 {
            print!("Depth {} result:", i);
            for tile in put(&mut hand, &mut wall, i).1 {
                print!(" {}", format_tile(tile));
            }
            println!();
        }
    }
}
*/
