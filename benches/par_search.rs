use bilge::board;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
  let game = board::board_from_str(
    "130154251324134140214034254524521234125231405023131010130530420204134014",
    3,
  );
  game.draw();

  c.bench_function("Brute", |b| {
    b.iter(|| board::searcher::find_best_move(&game));
  });

  c.bench_function("Rayon", |b| {
    b.iter(|| board::alt_search::find_best_move(&game, 3));
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
