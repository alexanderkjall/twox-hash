use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use rand::{Rng, RngCore, SeedableRng};
use std::{hash::Hasher as _, iter};

use xx_hash_sys as c;
use xx_renu as rust;

const TINY_DATA_SIZE: usize = 32;
const BIG_DATA_SIZE: usize = 4 * 1024 * 1024;
const MIN_BIG_DATA_SIZE: usize = 256 * 1024;
const MAX_CHUNKS: usize = 64;
const SEED: u64 = 0xc651_4843_1995_363f;

fn tiny_data(c: &mut Criterion) {
    let (seed, data) = gen_data(TINY_DATA_SIZE);
    let mut g = c.benchmark_group("tiny_data");

    for size in 0..=data.len() {
        let data = &data[..size];
        g.throughput(Throughput::Bytes(data.len() as _));

        let id = format!("impl-c/fn-oneshot/size-{size:02}");
        g.bench_function(id, |b| b.iter(|| c::XxHash64::oneshot(seed, data)));

        let id = format!("impl-c/fn-streaming/size-{size:02}");
        g.bench_function(id, |b| {
            b.iter(|| {
                let mut hasher = c::XxHash64::with_seed(seed);
                hasher.write(data);
                hasher.finish()
            })
        });

        let id = format!("impl-rust/fn-oneshot/size-{size:02}");
        g.bench_function(id, |b| b.iter(|| rust::XxHash64::oneshot(seed, data)));

        let id = format!("impl-rust/fn-streaming/size-{size:02}");
        g.bench_function(id, |b| {
            b.iter(|| {
                let mut hasher = rust::XxHash64::with_seed(seed);
                hasher.write(data);
                hasher.finish()
            })
        });
    }

    g.finish();
}

fn oneshot(c: &mut Criterion) {
    let (seed, data) = gen_data(BIG_DATA_SIZE);
    let mut g = c.benchmark_group("oneshot");

    for size in half_sizes(data.len()).take_while(|&s| s >= MIN_BIG_DATA_SIZE) {
        let data = &data[..size];
        g.throughput(Throughput::Bytes(data.len() as _));

        let id = format!("impl-c/size-{size:07}");
        g.bench_function(id, |b| b.iter(|| c::XxHash64::oneshot(seed, data)));

        let id = format!("impl-rust/size-{size:07}");
        g.bench_function(id, |b| b.iter(|| rust::XxHash64::oneshot(seed, data)));
    }

    g.finish();
}

fn streaming(c: &mut Criterion) {
    let mut g = c.benchmark_group("streaming_many_chunks");

    for size in half_sizes(BIG_DATA_SIZE).take_while(|&s| s >= MIN_BIG_DATA_SIZE) {
        for n_chunks in half_sizes(MAX_CHUNKS) {
            let (seed, chunks) = gen_chunked_data(size, n_chunks);
            g.throughput(Throughput::Bytes(size as _));

            let id = format!("impl-c/size-{size:07}/chunks-{n_chunks:02}");
            g.bench_function(id, |b| {
                b.iter(|| {
                    let mut hasher = c::XxHash64::with_seed(seed);
                    for chunk in &chunks {
                        hasher.write(chunk);
                    }
                    hasher.finish()
                })
            });

            let id = format!("impl-rust/size-{size:07}/chunks-{n_chunks:02}");
            g.bench_function(id, |b| {
                b.iter(|| {
                    let mut hasher = rust::XxHash64::with_seed(seed);
                    for chunk in &chunks {
                        hasher.write(chunk);
                    }
                    hasher.finish()
                })
            });
        }
    }

    g.finish();
}

fn gen_data(length: usize) -> (u64, Vec<u8>) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(SEED);

    let seed = rng.gen();

    let mut data = vec![0; length];
    rng.fill_bytes(&mut data);

    (seed, data)
}

fn gen_chunked_data(length: usize, n_chunks: usize) -> (u64, Vec<Vec<u8>>) {
    assert!(length >= n_chunks);

    let mut rng = rand::rngs::StdRng::seed_from_u64(SEED);

    let seed = rng.gen();

    let chunk_size = length / n_chunks;

    let mut total = 0;
    let mut chunks = Vec::with_capacity(2 * n_chunks);

    while total < length {
        let mut data = vec![0; chunk_size];
        rng.fill_bytes(&mut data);

        total += data.len();
        chunks.push(data)
    }

    (seed, chunks)
}

fn half_sizes(max: usize) -> impl Iterator<Item = usize> {
    iter::successors(Some(max), |&v| if v == 1 { None } else { Some(v / 2) })
}

mod xxhash3_64 {
    use super::*;

    fn tiny_data(c: &mut Criterion) {
        let (seed, data) = gen_data(240);
        let mut g = c.benchmark_group("xxhash3_64/tiny_data");

        // let categories = 0..=data.len();

        // Visual inspection of all the data points showed these as
        // examples of thier nearby neighbors.
        let categories = [
            0, 2, 6, 13, 25, 50, 80, 113, 135, 150, 165, 185, 200, 215, 230,
        ];

        for size in categories {
            let data = &data[..size];
            g.throughput(Throughput::Bytes(data.len() as _));

            let id = format!("impl-c/fn-oneshot/size-{size:03}");
            g.bench_function(id, |b| {
                b.iter(|| c::XxHash3_64::oneshot_with_seed(seed, data))
            });

            let id = format!("impl-rust/fn-oneshot/size-{size:03}");
            g.bench_function(id, |b| {
                b.iter(|| rust::XxHash3_64::oneshot_with_seed(seed, data))
            });
        }

        g.finish();
    }

    fn oneshot(c: &mut Criterion) {
        let (seed, data) = gen_data(BIG_DATA_SIZE);
        let mut g = c.benchmark_group("xxhash3_64/oneshot");

        for size in half_sizes(data.len()).take_while(|&s| s >= MIN_BIG_DATA_SIZE) {
            let data = &data[..size];
            g.throughput(Throughput::Bytes(data.len() as _));

            let id = format!("impl-c/size-{size:07}");
            g.bench_function(id, |b| {
                b.iter(|| c::XxHash3_64::oneshot_with_seed(seed, data))
            });

            let id = format!("impl-c-scalar/size-{size:07}");
            g.bench_function(id, |b| {
                b.iter(|| c::scalar::XxHash3_64::oneshot_with_seed(seed, data))
            });

            #[cfg(target_arch = "aarch64")]
            {
                let id = format!("impl-c-neon/size-{size:07}");
                g.bench_function(id, |b| {
                    b.iter(|| c::neon::XxHash3_64::oneshot_with_seed(seed, data))
                });
            }

            #[cfg(target_arch = "x86_64")]
            {
                let id = format!("impl-c-avx2/size-{size:07}");
                g.bench_function(id, |b| {
                    b.iter(|| c::avx2::XxHash3_64::oneshot_with_seed(seed, data))
                });

                let id = format!("impl-c-sse2/size-{size:07}");
                g.bench_function(id, |b| {
                    b.iter(|| c::sse2::XxHash3_64::oneshot_with_seed(seed, data))
                });
            }

            let id = format!("impl-rust/size-{size:07}");
            g.bench_function(id, |b| {
                b.iter(|| rust::XxHash3_64::oneshot_with_seed(seed, data))
            });
        }

        g.finish();
    }

    fn streaming(c: &mut Criterion) {
        let mut g = c.benchmark_group("xxhash3_64/streaming_many_chunks");

        for size in [1024 * 1024] {
            for n_chunks in half_sizes(size) {
                let (seed, chunks) = gen_chunked_data(size, n_chunks);
                g.throughput(Throughput::Bytes(size as _));

                let id = format!("impl-c/size-{size:07}/chunks-{n_chunks:02}");
                g.bench_function(id, |b| {
                    b.iter(|| {
                        let mut hasher = c::XxHash3_64::with_seed(seed);
                        for chunk in &chunks {
                            hasher.write(chunk);
                        }
                        hasher.finish()
                    })
                });

                let id = format!("impl-c-scalar/size-{size:07}/chunks-{n_chunks:02}");
                g.bench_function(id, |b| {
                    b.iter(|| {
                        let mut hasher = c::scalar::XxHash3_64::with_seed(seed);
                        for chunk in &chunks {
                            hasher.write(chunk);
                        }
                        hasher.finish()
                    })
                });

                #[cfg(target_arch = "aarch64")]
                {
                    let id = format!("impl-c-neon/size-{size:07}/chunks-{n_chunks:02}");
                    g.bench_function(id, |b| {
                        b.iter(|| {
                            let mut hasher = c::neon::XxHash3_64::with_seed(seed);
                            for chunk in &chunks {
                                hasher.write(chunk);
                            }
                            hasher.finish()
                        })
                    });
                }

                #[cfg(target_arch = "x86_64")]
                {
                    let id = format!("impl-c-avx2/size-{size:07}/chunks-{n_chunks:02}");
                    g.bench_function(id, |b| {
                        b.iter(|| {
                            let mut hasher = c::avx2::XxHash3_64::with_seed(seed);
                            for chunk in &chunks {
                                hasher.write(chunk);
                            }
                            hasher.finish()
                        })
                    });

                    let id = format!("impl-c-sse2/size-{size:07}/chunks-{n_chunks:02}");
                    g.bench_function(id, |b| {
                        b.iter(|| {
                            let mut hasher = c::sse2::XxHash3_64::with_seed(seed);
                            for chunk in &chunks {
                                hasher.write(chunk);
                            }
                            hasher.finish()
                        })
                    });
                }

                let id = format!("impl-rust/size-{size:07}/chunks-{n_chunks:02}");
                g.bench_function(id, |b| {
                    b.iter(|| {
                        let mut hasher = rust::XxHash3_64::with_seed(seed);
                        for chunk in &chunks {
                            hasher.write(chunk);
                        }
                        hasher.finish()
                    })
                });
            }
        }

        g.finish();
    }

    criterion_group!(benches, tiny_data, oneshot, streaming);
}

criterion_group!(benches, tiny_data, oneshot, streaming);

criterion_main!(benches, xxhash3_64::benches);
