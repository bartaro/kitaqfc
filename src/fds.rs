//! FDS metadata, runtime load tables and fixed-size disk-side packaging.
pub mod disk;
pub mod metadata;
pub use metadata::{File, Metadata};

pub fn stable_name_hash(name: &str) -> u16 {
    name.encode_utf16().fold(0x811cu16, |hash, unit| {
        (hash ^ (unit as u8 as u16)).wrapping_mul(0x0101)
    })
}

pub fn overlay_name(prefix: &str, bank: i32) -> String {
    let mut name: String = prefix
        .chars()
        .filter(|c| ('!'..='~').contains(c))
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if name.is_empty() {
        name.push_str("KQFB");
    }
    name.push_str(&format!("{bank:03}"));
    name.truncate(name.len().min(8));
    while name.len() < 8 {
        name.push(' ');
    }
    name
}
