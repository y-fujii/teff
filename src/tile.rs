// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::fmt::Write;
use std::*;

#[derive(Clone)]
pub struct TileSet {
    tiles: [u8; 34],
}

impl TileSet {
    pub fn new() -> Self {
        TileSet { tiles: [0; 34] }
    }

    pub fn tile(&self, i: usize) -> u8 {
        self.tiles[i]
    }

    pub fn tile_mut(&mut self, i: usize) -> &mut u8 {
        &mut self.tiles[i]
    }

    pub fn simple(&self, t: usize, i: usize) -> u8 {
        self.tiles[9 * t + i]
    }

    pub fn simple_mut(&mut self, t: usize, i: usize) -> &mut u8 {
        &mut self.tiles[9 * t + i]
    }

    pub fn honor(&self, i: usize) -> u8 {
        self.tiles[9 * 3 + i]
    }

    pub fn honor_mut(&mut self, i: usize) -> &mut u8 {
        &mut self.tiles[9 * 3 + i]
    }

    pub fn count(&self) -> usize {
        self.iter().map(|e| *e as usize).sum()
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn iter(&self) -> slice::Iter<u8> {
        self.tiles.iter()
    }
}

pub fn format_tile(tile: usize) -> String {
    match tile {
        0..=8 => format!("{}m", tile + 1),
        9..=17 => format!("{}p", tile - 8),
        18..=26 => format!("{}s", tile - 17),
        27..=33 => format!("{}z", tile - 26),
        _ => panic!(),
    }
}

pub fn format_tile_set(hand: &TileSet) -> String {
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

pub fn parse_tile_set(text: &str) -> Option<TileSet> {
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

const WEIGHT_PAIR: usize = 2;
const WEIGHT_TRIAD: usize = 3;

pub fn count_head_and_triad(hand: &mut TileSet, allow_headless: bool) -> usize {
    let n_simples = [
        count_triad_simple(hand, 0, 0),
        count_triad_simple(hand, 1, 0),
        count_triad_simple(hand, 2, 0),
    ];
    let n_honor = count_triad_honor(hand);

    let mut n_total = if allow_headless {
        n_simples[0] + n_simples[1] + n_simples[2] + n_honor
    } else {
        0
    };
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
