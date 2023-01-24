use morningstar::*;
use rand::random;
use rayon::prelude::*;

fn main() {
    // Differential
    // let d_in = Bits::new(0x00_80_82_00_60_00_00_00);
    // let d_out = Bits::new(0x60_00_00_00_00_00_00_00);
    // let rounds = 2;

    // let d_in = Bits::new(0x00_80_82_00_60_00_00_00);
    // let d_out = Bits::new(0x00_80_82_00_60_00_00_00);
    // let rounds = 3;

    // let d_in = Bits::new(0x40_5C_00_00_04_00_00_00);
    // let d_out = Bits::new(0x40_5C_00_00_04_00_00_00);
    // let rounds = 5;

    // estimate_diff_probability(d_in, d_out, rounds);

    // Linear
    // let alpha = Bits::new(0x21_04_00_80__00_00_80_00);
    // let beta = alpha;
    // estimate_linear_probability(alpha, beta, 3);

    let alpha = Bits::new(0x01_04_00_80__00_01_10_00);
    let beta = Bits::new(0x21_04_00_80__00_00_80_00);
    estimate_linear_probability::<7>(alpha, beta);
}

fn estimate_linear_probability<const R: usize>(alpha: Bits<64>, beta: Bits<64>) {
    let total = 10_000_000;

    // let key = Bits::new(0xdeadbeefcafebabe);
    let key = Bits::new(random::<u64>());
    let keys = generate_keys(key);

    let ones: u64 = (0..total)
        .into_par_iter()
        .map(|_| {
            let plaintext = Bits::new(random::<u64>());

            let ciphertext = weak_encrypt::<R>(plaintext, &keys);

            // println!("{:x}", cipher_diff.as_u64());
            (alpha.dot_product(plaintext) ^ beta.dot_product(ciphertext)) as u64
        })
        .sum();

    let probability = ones as f64 / total as f64;

    println!("ones:  {}", 100.0 * probability);
    println!("zeros: {}", 100.0 * (1.0 - probability));

    println!("bias: {}", (probability - 0.5).abs());
    println!("correlation: {}", (2.0 * probability - 1.0).abs());
}

fn estimate_diff_probability<const R: usize>(d_in: Bits<64>, d_out: Bits<64>) {
    let total = 10_000_000;

    let key = Bits::new(0xdeadbeefcafebabe);
    let keys = generate_keys(key);

    let matches: u64 = (0..total)
        .into_par_iter()
        .map(|_| {
            let plaintext_a = Bits::new(random::<u64>());
            let plaintext_b = plaintext_a ^ d_in;

            let ciphertext_a = weak_encrypt::<R>(plaintext_a, &keys);
            let ciphertext_b = weak_encrypt::<R>(plaintext_b, &keys);

            let cipher_diff = ciphertext_a ^ ciphertext_b;

            // println!("{:x}", cipher_diff.as_u64());
            (cipher_diff.as_u64() == d_out.as_u64()) as u64
        })
        .sum();

    println!("{matches:?}");
    println!("{:?}", matches as f64 / total as f64);
    // println!("{:?} / 64", 64.0 * matches as f64 / total as f64);
}
