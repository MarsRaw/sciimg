//! So this is a carbon copy of the two implementations in src/gaussian.rs
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use sciimg::image::Image;

use std::time::Duration;

const INPAINT_TEST_IMAGE: &str = "tests/testdata/MSL_MAHLI_INPAINT_Sol2904_V1.png";

// st
fn st(img: &mut Image, sigma: f32) {
    use sciimg::gaussianblur::st_gaussian_blur;

    let mut buffers = vec![];
    (0..img.num_bands()).for_each(|b| {
        buffers.push(img.get_band(b).to_owned());
    });

    if let Ok(buffers) = st_gaussian_blur(&mut buffers, sigma) {
        buffers.iter().enumerate().for_each(|(b, _)| {
            img.set_band(&buffers[b], b);
        });
    }
}

#[cfg(feature = "rayon")]
// rayon
fn rayon(img: &mut Image, sigma: f32) {
    use sciimg::gaussianblur::par_gaussian_blur;

    let mut buffers = vec![];
    (0..img.num_bands()).for_each(|b| {
        buffers.push(img.get_band(b).to_owned());
    });

    if let Ok(buffers) = par_gaussian_blur(&mut buffers, sigma) {
        buffers.iter().enumerate().for_each(|(b, _)| {
            img.set_band(&buffers[b], b);
        });
    }
}

fn setup_benchmark_data() -> Image {
    Image::open(&String::from(INPAINT_TEST_IMAGE)).unwrap()
}

fn benchmark_gaussian_blur(c: &mut Criterion) {
    let mut group = c.benchmark_group("GaussianBlur");
    group.measurement_time(Duration::from_secs(10));

    // Setup benchmark parameters
    let sigma_values = [2.0];

    for sigma in sigma_values.iter() {
        // Benchmark original implementation
        group.bench_with_input(
            BenchmarkId::new("single-threaded", sigma),
            sigma,
            |b, &sigma| {
                b.iter(|| {
                    let mut img = setup_benchmark_data();
                    black_box(st(&mut img, sigma))
                })
            },
        );
    }

    #[cfg(feature = "rayon")]
    {
        for sigma in sigma_values.iter() {
            // Benchmark original implementation
            group.bench_with_input(BenchmarkId::new("rayon", sigma), sigma, |b, &sigma| {
                b.iter(|| {
                    let mut img = setup_benchmark_data();
                    black_box(rayon(&mut img, sigma))
                })
            });
        }
    }
    group.finish();
}

criterion_group!(benches, benchmark_gaussian_blur);
criterion_main!(benches);
