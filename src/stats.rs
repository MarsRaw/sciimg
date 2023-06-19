pub fn radians(d: f64) -> f64 {
    d * std::f64::consts::PI / 180.0
}

pub fn degrees(r: f64) -> f64 {
    r * 180.0 / std::f64::consts::PI
}

//https://rust-lang-nursery.github.io/rust-cookbook/science/mathematics/statistics.html
pub fn mean(data: &[f32]) -> f32 {
    let filtered: Vec<f32> = data
        .iter()
        .filter(|p| !p.is_nan() && !p.is_infinite())
        .copied()
        .collect();
    filtered.iter().sum::<f32>() / filtered.len() as f32
}

pub fn std_deviation(data: &[f32]) -> f32 {
    let data_mean = mean(data);
    let variance = data
        .iter()
        .filter(|p| !p.is_nan() && !p.is_infinite())
        .map(|value| {
            let diff = data_mean - (*value);

            diff * diff
        })
        .sum::<f32>()
        / data.len() as f32;
    variance.sqrt()
}

pub fn z_score(pixel_value: f32, data: &[f32]) -> f32 {
    let data_mean = mean(data);
    let data_std_deviation = std_deviation(data);
    let data_value = pixel_value;

    let diff = data_value - data_mean;
    diff / data_std_deviation
}
