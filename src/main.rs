#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![allow(dead_code)]

mod bits;
use bits::Bits;

fn main() {
    println!("Hello, world!");
}

const ROUNDS: usize = 16;

fn encrypt(plaintext: Bits<64>, key: Bits<64>) -> Bits<64> {
    let keys = generate_keys(key);
    dbg!(plaintext);

    // Apply IP
    let data = plaintext.permute(&IP);
    dbg!(data);

    // split
    let (mut u, mut v) = data.split::<32>();

    println!("L {:#034b}", u.as_u64());
    println!("R {:#034b}\n", v.as_u64());

    // Apply rounds
    for round in 1..=ROUNDS {
        println!("Round {}", round);
        // Apply Feistel function
        let key = keys[round-1];

        // apply E
        let e = v.permute(&E);
        println!("E {:#050b}", e.as_u64());
        println!("fake E {:#050b}", e.as_u64());

        let keyed = e ^ key;
        println!("E^K {:#050b}", e.as_u64());

        // apply S-box
        let mut sbox_output: u64 = 0;

        for (i, sbox) in SBOX.iter().enumerate() {
            let slice: Bits<6> = keyed.range(i * 6 + 1, i * 6 + 6);
            let p = 2 * slice.get(1) as u64 + slice.get(6) as u64;

            let n = slice.const_range::<2, 5>().as_u64();

            // Sbox i, row p and element n
            sbox_output <<= 4;
            sbox_output |= sbox[p as usize][n as usize] as u64;
        }
        let sbox_output: Bits<32> = Bits::new(sbox_output);
        println!("Sbox {:#034b}", sbox_output.as_u64());

        // Apply P
        let p = sbox_output.permute(&P);
        println!("P {:#034b}", p.as_u64());

        // xor p onto u
        u = u ^ p;

        println!("L {:#034b}", u.as_u64());
        println!("R {:#034b}\n", v.as_u64());

        // Swap sides
        (u, v) = (v, u);
    }

    // un-swap the last swap amd concatenate them
    let data = v.concat(u);

    // Apply IP_INVERSE
    let ciphertext = data.permute(&IP_INVERSE);
    ciphertext
}

fn trim_key(key: Bits<64>) -> Bits<56> {
    let a: Bits<7> = key.const_range::<1, 7>();
    let b: Bits<7> = key.const_range::<9, 15>();
    let c: Bits<7> = key.const_range::<17, 23>();
    let d: Bits<7> = key.const_range::<25, 31>();
    let e: Bits<7> = key.const_range::<33, 39>();
    let f: Bits<7> = key.const_range::<41, 47>();
    let g: Bits<7> = key.const_range::<49, 55>();
    let h: Bits<7> = key.const_range::<57, 63>();

    a.concat(b)
        .concat(c)
        .concat(d)
        .concat(e)
        .concat(f)
        .concat(g)
        .concat(h)
}

const PC1: [u8; 56] = [
    57,  49,  41,  33,  25,  17,   9,
     1,  58,  50,  42,  34,  26,  18,
    10,   2,  59,  51,  43,  35,  27,
    19,  11,   3,  60,  52,  44,  36,
    63,  55,  47,  39,  31,  23,  15,
     7,  62,  54,  46,  38,  30,  22,
    14,   6,  61,  53,  45,  37,  29,
    21,  13,   5,  28,  20,  12,   4,
];

const PC2: [u8; 48] = [
    14,  17,  11,  24,   1,   5,
     3,   28,  15,  6,  21,  10,
    23,  19,  12,   4,  26,   8,
    16,   7,  27,  20,  13,   2,
    41,  52,  31,  37,  47,  55,
    30,  40,  51,  45,  33,  48,
    44,  49,  39,  56,  34,  53,
    46,  42,  50,  36,  29,  32,
];

const LSHIFT_MAP: [u8; 16] = [1, 1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1];

fn generate_keys(key: Bits<64>) -> Vec<Bits<48>> {
    let mut cd = key.permute(&PC1);
    println!("cd[ 0] = {:#058b}", cd.as_u64());

    let mut keys: Vec<Bits<48>> = Vec::with_capacity(ROUNDS);

    for i in 1..=ROUNDS {
        let shift = LSHIFT_MAP[i-1] as usize;

        let (mut c, mut d) = cd.split::<28>();
        c = c.rotate_left(shift);
        d = d.rotate_left(shift);
        cd = c.concat(d);

        let key = cd.permute(&PC2);
        keys.push(key);

        println!("cd[{:2}] = {:#058b}", i, cd.as_u64());
        println!("k[{:2}]  = {:#050b}", i, key.as_u64());
    }

    keys
}

#[rustfmt::skip]
// 64 bits -> 64 bits
const IP: [u8; 64] = [
    58, 50, 42, 34, 26, 18, 10, 2,
    60, 52, 44, 36, 28, 20, 12, 4,
    62, 54, 46, 38, 30, 22, 14, 6,
    64, 56, 48, 40, 32, 24, 16, 8,
    57, 49, 41, 33, 25, 17,  9, 1,
    59, 51, 43, 35, 27, 19, 11, 3,
    61, 53, 45, 37, 29, 21, 13, 5,
    63, 55, 47, 39, 31, 23, 15, 7,
];

#[rustfmt::skip]
// 64 bits -> 64 bits
const IP_INVERSE: [u8; 64] = [
    40, 8, 48, 16, 56, 24, 64, 32,
    39, 7, 47, 15, 55, 23, 63, 31,
    38, 6, 46, 14, 54, 22, 62, 30,
    37, 5, 45, 13, 53, 21, 61, 29,
    36, 4, 44, 12, 52, 20, 60, 28,
    35, 3, 43, 11, 51, 19, 59, 27,
    34, 2, 42, 10, 50, 18, 58, 26,
    33, 1, 41,  9, 49, 17, 57, 25,
];

#[rustfmt::skip]
// 32 bits -> 32 bits
const P: [u8; 32] = [
    16,  7, 20, 21, 29, 12, 28, 17,
     1, 15, 23, 26,  5, 18, 31, 10,
     2,  8, 24, 14, 32, 27,  3,  9,
    19, 13, 30,  6, 22, 11,  4, 25,
];

#[rustfmt::skip]
// 32 bits -> 48 bits
const E: [u8; 48] = [
    32,  1,  2,  3,  4,  5,
     4,  5,  6,  7,  8,  9,
     8,  9, 10, 11, 12, 13,
    12, 13, 14, 15, 16, 17,
    16, 17, 18, 19, 20, 21,
    20, 21, 22, 23, 24, 25,
    24, 25, 26, 27, 28, 29,
    28, 29, 30, 31, 32,  1,
];

#[rustfmt::skip]
// This one is complex, but when using it properly it's 48 bits -> 32 bits
const SBOX: [[[u8; 16]; 4]; 8] = [
    // S1
    [
        [14, 4, 13, 1, 2, 15, 11, 8, 3, 10, 6, 12, 5, 9, 0, 7],
        [0, 15, 7, 4, 14, 2, 13, 1, 10, 6, 12, 11, 9, 5, 3, 8],
        [4, 1, 14, 8, 13, 6, 2, 11, 15, 12, 9, 7, 3, 10, 5, 0],
        [15, 12, 8, 2, 4, 9, 1, 7, 5, 11, 3, 14, 10, 0, 6, 13],
    ], 
    // S2
    [
        [15, 1, 8, 14, 6, 11, 3, 4, 9, 7, 2, 13, 12, 0, 5, 10],
        [3, 13, 4, 7, 15, 2, 8, 14, 12, 0, 1, 10, 6, 9, 11, 5],
        [0, 14, 7, 11, 10, 4, 13, 1, 5, 8, 12, 6, 9, 3, 2, 15],
        [13, 8, 10, 1, 3, 15, 4, 2, 11, 6, 7, 12, 0, 5, 14, 9],
    ], 
    // S3
    [
        [10, 0, 9, 14, 6, 3, 15, 5, 1, 13, 12, 7, 11, 4, 2, 8],
        [13, 7, 0, 9, 3, 4, 6, 10, 2, 8, 5, 14, 12, 11, 15, 1],
        [13, 6, 4, 9, 8, 15, 3, 0, 11, 1, 2, 12, 5, 10, 14, 7],
        [1, 10, 13, 0, 6, 9, 8, 7, 4, 15, 14, 3, 11, 5, 2, 12],
    ], 
    // S4
    [
        [7, 13, 14, 3, 0, 6, 9, 10, 1, 2, 8, 5, 11, 12, 4, 15],
        [13, 8, 11, 5, 6, 15, 0, 3, 4, 7, 2, 12, 1, 10, 14, 9],
        [10, 6, 9, 0, 12, 11, 7, 13, 15, 1, 3, 14, 5, 2, 8, 4],
        [3, 15, 0, 6, 10, 1, 13, 8, 9, 4, 5, 11, 12, 7, 2, 14],
    ], 
    // S5
    [
        [2, 12, 4, 1, 7, 10, 11, 6, 8, 5, 3, 15, 13, 0, 14, 9],
        [14, 11, 2, 12, 4, 7, 13, 1, 5, 0, 15, 10, 3, 9, 8, 6],
        [4, 2, 1, 11, 10, 13, 7, 8, 15, 9, 12, 5, 6, 3, 0, 14],
        [11, 8, 12, 7, 1, 14, 2, 13, 6, 15, 0, 9, 10, 4, 5, 3],
    ], 
    // S6
    [
        [12, 1, 10, 15, 9, 2, 6, 8, 0, 13, 3, 4, 14, 7, 5, 11],
        [10, 15, 4, 2, 7, 12, 9, 5, 6, 1, 13, 14, 0, 11, 3, 8],
        [9, 14, 15, 5, 2, 8, 12, 3, 7, 0, 4, 10, 1, 13, 11, 6],
        [4, 3, 2, 12, 9, 5, 15, 10, 11, 14, 1, 7, 6, 0, 8, 13],
    ], 
    // S7
    [
        [4, 11, 2, 14, 15, 0, 8, 13, 3, 12, 9, 7, 5, 10, 6, 1],
        [13, 0, 11, 7, 4, 9, 1, 10, 14, 3, 5, 12, 2, 15, 8, 6],
        [1, 4, 11, 13, 12, 3, 7, 14, 10, 15, 6, 8, 0, 5, 9, 2],
        [6, 11, 13, 8, 1, 4, 10, 7, 9, 5, 0, 15, 14, 2, 3, 12],
    ], 
    // S8
    [
        [13, 2, 8, 4, 6, 15, 11, 1, 10, 9, 3, 14, 5, 0, 12, 7],
        [1, 15, 13, 8, 10, 3, 7, 4, 12, 5, 6, 11, 0, 14, 9, 2],
        [7, 11, 4, 1, 9, 12, 14, 2, 0, 6, 10, 13, 15, 3, 5, 8],
        [2, 1, 14, 7, 4, 10, 8, 13, 15, 12, 9, 0, 3, 5, 6, 11],
    ],
];

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encrypt_doesnt_panic() {
        let plaintext = 0x4141414141414141;
        encrypt(Bits::<64>::new(plaintext), Bits::new(0));
    }

    #[test]
    // #[ignore = "gah"]
    fn test_vector() {
        let plaintext: Bits<64> = Bits::new(0x4e6f772069732074);

        let key: Bits<64> = Bits::new(0x0123456789abcdef);
        println!("key  = {:#066b}", key.as_u64());

        assert_eq!(encrypt(plaintext, key).as_u64(), 0x3fa40e8a984d4815);
    }

    #[test]
    fn zero_key() {
        let key: Bits<64> = Bits::new(0);
        let plaintext: Bits<64> = Bits::<64>::new(0);

        let ciphertext = encrypt(plaintext, key);

        assert_eq!(ciphertext.as_u64(), 0x8ca64de9c1b123a7);
    }
    
    #[test]
    fn keys() {
        let key: Bits<64> = Bits::new(0xFF);
        generate_keys(key);
    }
}
