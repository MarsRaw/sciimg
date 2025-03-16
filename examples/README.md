# GPU Gaussian Blur Example

This crate contains multiple Gaussian blur options, a single threaded, a rayon multithreaded and a GPU accelerated version.

![before]("/Users/smak/Documents/sciimg/tests/testdata/ZL0_0053_0671642352_402ECM_N0032046ZCAM05025_110085J01.png")
![after]("../assets/gaussian_gpu.png)

This example demonstrates performing a Gaussian blur on an image using GPU acceleration with the `sciimg` crate. It provides a benchmark of the GPU processing time.

## Running the Example

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
