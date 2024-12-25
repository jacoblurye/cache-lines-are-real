use std::hint::black_box;

use plotly::{layout::Axis, Layout, Plot, Scatter};

const CAPACITY: usize = 32 * 1024 * 1024 as usize;
const N: usize = 10;

/// Performs a strided copy from `src` to `dst` with the given `stride`.
/// The number of copies will be `src.len() / stride`.
pub fn strided_copy(stride: usize, src: Vec<u8>, mut dst: Vec<u8>) {
    for i in (0..src.len()).step_by(stride) {
        dst[i] = src[i];
    }
}

fn main() {
    let strides: Vec<usize> = (16..=512).into_iter().collect();
    let mut mins = Vec::with_capacity(N);
    let mut avgs = Vec::with_capacity(N);
    let mut maxs = Vec::with_capacity(N);

    let pb = indicatif::ProgressBar::new(strides.len() as u64);
    for &stride in &strides {
        let timings = (0..N).map(|_| {
            // Black box the vectors to prevent the compiler from optimizing the copy away.
            let src = black_box(vec![0; CAPACITY]);
            let dst = black_box(vec![0; CAPACITY]);

            let start = std::time::Instant::now();
            strided_copy(stride, src, dst);
            1e9 / start.elapsed().as_nanos() as f32
        });

        mins.push(timings.clone().fold(f32::INFINITY, f32::min));
        maxs.push(timings.clone().fold(f32::NEG_INFINITY, f32::max));
        avgs.push(timings.clone().sum::<f32>() / N as f32);

        pb.inc(1);
    }

    let mut plot = Plot::new();
    let min_trace = Scatter::new(strides.clone(), mins).name("Min");
    let max_trace = Scatter::new(strides.clone(), maxs).name("Max");
    let avg_trace = Scatter::new(strides.clone(), avgs).name("Average");
    plot.add_trace(min_trace);
    plot.add_trace(avg_trace);
    plot.add_trace(max_trace);
    plot.set_layout(
        Layout::new()
            .title(format!(
                "strided copies on a {}mb buffer",
                CAPACITY / 1024 / 1024
            ))
            .x_axis(Axis::new().title("stride length (bytes)"))
            .y_axis(Axis::new().title("loops per second")),
    );

    plot.show();
}
