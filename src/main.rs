// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use teff::bench;
use teff::tile::*;

fn main() {
    if env::args().len() <= 1 {
        bench::benchmark(&mut rand::thread_rng());
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
            bench::analyze_hand(&mut hand);
        }
    }
}
