# Examples

To see them all:
`cargo run --example`

## Table of Contents

1. [Examples](#examples)
   - [GPU Gaussian Blur Example](#gpu-gaussian-blur-example)
   - [Hotpixel detection & Correction] TODO

## GPU Gaussian Blur Example

| Before                                                                                                                                    | After                                                                                      |
| ----------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| ![before](https://github.com/MarsRaw/sciimg/blob/main/tests/testdata/ZL0_0053_0671642352_402ECM_N0032046ZCAM05025_110085J01.png?raw=true) | ![after](https://github.com/alphastrata/sciimg/blob/play/assets/gaussian_gpu.png?raw=true) |

This crate contains multiple Gaussian blur options, a single threaded, a rayon multithreaded and a GPU accelerated version.

This example demonstrates performing a Gaussian blur on an image using our `GpuImage` type, which can be constructed from our `sciimg::Image` type.

### Running the Example

This example requires a working Rust installation and the `sciimg` crate. You can run it with the following command:

```bash
RUST_LOG=info cargo run --example gpu_gaussian --release

 INFO  gpu_gaussian > GPU context created in: 381.329962ms
 INFO  gpu_gaussian > Original sciimg loaded in: 57.954144ms
 INFO  gpu_gaussian > Conversion to gpuimg complete in: 21.903735ms
 INFO  gpu_gaussian > Gaussian blur completed in: 1.440983303s
 INFO  gpu_gaussian > Saving the processed image...
 INFO  gpu_gaussian > Processed image saved as 'gaussian_gpu.png' in: 105.944385ms
 INFO  gpu_gaussian > Total runtime 2.008285s
```

> Note this was run on a very old machine, your times will likely be better.

```txt
OS: macOS 11.7.10 20G1427 x86_64
Host: MacBookPro 2013
CPU: Intel i7-4960HQ (8) @ 2.60GHz
GPU: Intel Iris Pro, NVIDIA GeForce GT 750M
Memory: 10927MiB / 16384MiB
```

> Note the reason we provide an example like this with such verbose timing information is that it's not always worthwhile to use GPU acceleration, we provide benchmarks in the `./benches/gaussian.rs` so you can reference them and be armed with the best tools to get your work done as efficiently as possible.
