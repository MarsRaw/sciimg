# How to contribute GPU accelerated work

### Conventions:

- use `encase`'s `ShaderType` derive on your shared `struct`s.
- use `pollster` when you're at the mercy of `wgpu`'s `async` first architecture.
- use our `GpuImage` and `GpuContext` types, extend them if needed.
- shaders go in `gpu/shaders/*.wgsl`
- image processing workloads should be `gpu.<your image processing algo>($args) -> GpuImage`
- 'gpu-accelerated' workloads REQUIRE tests and benchmarks -- if a benchmark reveals that the gains are negligible please hold off on making a PR.
- for any given workload you're looking to make a gpu workflow for try to use the same arguments that the CPU implementations in your `Uniforms` see the [guide](#guide).

### guide:

TODO

```rust
pub trait RgbImageBlur {
    fn gaussian_blur(&mut self, sigma: f32);
}
```

So the `Image` actual, and a `sigma` value:

```rust
#[derive(ShaderType)]
struct GaussianBlurUniform {
    pub radius: u32,
    pub sigma: f32,
    pub width: u32,
    pub height: u32,
}
```

which with some boilerplate (unfortunately a lot) you're able to use in your shadercode like this:

```rust
struct GaussianBlurUniform {
    radius : u32,
    sigma  : f32,
    width  : u32,
    height : u32,
};

@group(0) @binding(0) var<uniform> blur_params: GaussianBlurUniform;
```

This is because the `main` of a shader isn't really like a regular `Rust` function.
