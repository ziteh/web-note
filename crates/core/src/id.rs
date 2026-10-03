
// 36 characters with 12 length: 100 IDs per hour, ~352 years or 308M IDs needed, in order to have a 1% probability of at least one collision.

const ALPHABET: [char; 36] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];
const LEN: usize = 12;

pub fn generate() -> String {
    nanoid::format(nanoid::rngs::default, &ALPHABET, LEN)
}

pub fn is_valid(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| ALPHABET.contains(&c))
}
