use bitvec::prelude::*;
use std::error::Error;

static INPUT: &str = include_str!("../input.txt");

fn main() -> Result<(), Box<dyn Error>> {
    let ct = find_sync(INPUT, 4).ok_or("no sync sequence found")?;
    println!("Sync marker required reading {ct} bytes");
    let ct = find_sync(INPUT, 14).ok_or("no message sequence found")?;
    println!("Message marker required reading {ct} bytes");
    Ok(())
}

fn find_sync(s: &str, size: usize) -> Option<usize> {
    s.as_bytes()
        .windows(size)
        .enumerate()
        .find(|(_, window)| {
            let mut ba = bitarr!(0; 128);
            for &b in *window {
                ba.set(b as usize, true);
            }
            ba.iter_ones().count() == size
        })
        .map(|(start, _)| start + size)
}
