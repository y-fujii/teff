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
        let mut n = 0;
        for i in 0..self.len() {
            n += self.tile(i) as usize;
        }
        n
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

pub fn format_tile_set(hand: &mut TileSet) -> String {
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

pub fn generate_random_hand<R: rand::Rng>(rng: &mut R) -> TileSet {
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
