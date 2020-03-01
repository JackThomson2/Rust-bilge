use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;

fn gen_rand_board(x: i32, y: i32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((x * y * 4) as usize);
    let mut rng = rand::thread_rng();

    for _ in 0..(x * y * 4) {
        pixels.push(rng.gen::<u8>())
    }

    pixels
}

fn place_img_inside(
    haystack: &mut Vec<u8>,
    inplacing: &Vec<u8>,
    x: usize,
    y: usize,
    outer_width: usize,
    inner_width: usize,
) {
    for i in 0..inplacing.len() {
        let x_pos = i % (inner_width * 4);
        let y_pos = (i / (inner_width * 4)) as usize;

        let hay_y = (y_pos + y) * 4;
        let hay_x = x_pos + x;

        let replacing = (hay_y * outer_width) + hay_x;
        haystack[replacing] = inplacing[i]
    }
}

fn gray_scale_me(converting: &Vec<u8>) -> Vec<u8> {
    let new_size = (converting.len() / 4) as usize;

    let mut new_vec = Vec::with_capacity(new_size);

    for i in 0..new_size {
        let pos = i * 4;
        let new_val = ((converting[pos] + converting[pos + 1] + converting[pos + 2]) / 3) as u8;
        new_vec.push(new_val);
    }

    new_vec
}

#[inline]
fn find_needle(
    haystack: &Vec<u8>,
    needle: &Vec<u8>,
    s_w: usize,
    s_h: usize,
    n_w: usize,
    n_h: usize,
) -> bool {
    for oy in 0..s_h {
        'outer: for ox in 0..s_w {
            for iy in 0..n_h {
                for ix in 0..n_w {
                    let n_p = ((iy * n_w) + ix) * 4;
                    let h_p = (((oy + iy) * s_w) + ix + ox) * 4;

                    for i in 0..3 {
                        if haystack[h_p + i] != needle[n_p + i] {
                            continue 'outer;
                        }
                    }
                }
            }
            return true;
        }
    }
    false
}

fn get_patterns(needle: &Vec<u8>, width: usize, height: usize) -> Vec<Vec<u8>> {
    let mut searching = vec![];
    for y in 0..height {
        let mut row = vec![];
        for x in 0..width {
            let pos = x + (y * width);
            row.push(needle[pos])
        }
        searching.push(row);
    }

    searching
}

#[inline]
fn jack_attempt(searcher: &AhoCorasick, searching: &Vec<u8>) -> usize {
    searcher.find_iter(searching).count()
}

#[inline]
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut haystack = gen_rand_board(1080, 1920);
    let needle = gen_rand_board(20, 20);

    place_img_inside(&mut haystack, &needle, 1000, 1000, 1080, 20);

    let gray_haystack = gray_scale_me(&haystack);
    let gray_needle = gray_scale_me(&needle);
    println!(
        "Gray size {}, needle size {}",
        gray_haystack.len(),
        gray_needle.len()
    );

    let patterns = get_patterns(&gray_needle, 20, 20);

    println!("{:?}", patterns);
    let ac: AhoCorasick = AhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostFirst)
        .byte_classes(false)
        .dfa(true)
        .build(patterns.clone());

    find_needle(&haystack, &needle, 1080, 1920, 20, 20);

    c.bench_function("Dani approach", |b| {
        b.iter(|| find_needle(&haystack, &needle, 1080, 1920, 20, black_box(20)))
    });

    c.bench_function("Jack approach", |b| b.iter(|| jack_attempt(&ac, &haystack)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
