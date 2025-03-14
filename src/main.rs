use clap::Parser;
use plotters::prelude::*;
use rustfft::{num_complex::Complex, FftPlanner};
use std::f64;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    filename: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Args::parse();

    let filename = cli.filename;
    // Open the WAV file
    let mut reader = hound::WavReader::open(filename)?;

    // Read samples (assuming i32 samples; change if needed)
    let samples: Vec<f32> = reader.samples::<i32>().map(|s| s.unwrap() as f32).collect();

    let sample_rate = reader.spec().sample_rate;
    let fft_size = 2048;
    let hop_size = fft_size / 2; // 50% overlap

    // Create FFT plan
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);

    let mut spectrogram: Vec<Vec<f32>> = Vec::new();

    // Process each window in the input signal
    for start in (0..samples.len().saturating_sub(fft_size)).step_by(hop_size) {
        let mut fft_input: Vec<Complex<f32>> = samples[start..start + fft_size]
            .iter()
            .map(|&x| Complex { re: x, im: 0.0 })
            .collect();

        // Perform FFT
        fft.process(&mut fft_input);

        // Compute magnitudes and normalize
        let fft_magnitude: Vec<f32> = fft_input
            .iter()
            .map(|c| c.norm() / (fft_size as f32))
            .collect();

        spectrogram.push(fft_magnitude);
    }

    // Average the magnitudes across the different FFT windows
    let num_windows = spectrogram.len();
    if num_windows == 0 {
        return Err("No FFT windows processed. Check your input file length.".into());
    }
    let mut avg_magnitudes = vec![0.0f32; fft_size];
    for row in &spectrogram {
        for j in 0..fft_size {
            avg_magnitudes[j] += row[j];
        }
    }
    for mag in &mut avg_magnitudes {
        *mag /= num_windows as f32;
    }

    // Create the spectrum in dB
    let mut spectrum: Vec<(f64, f64)> = Vec::new();
    let epsilon = 1e-10; // Prevents taking log of zero
    for j in 0..(fft_size / 2) {
        let frequency = (j as f64 * sample_rate as f64) / (fft_size as f64);
        let magnitude = avg_magnitudes[j] as f64;

        // Convert to dB: 20 * log10(magnitude + epsilon)
        let mut dB = 20.0 * (magnitude + epsilon).log10();

        // Optionally, clip the lower bound to -100 dB.
        if dB < -100.0 {
            dB = -100.0;
        }

        spectrum.push((frequency, dB));
    }

    // Determine y-axis range based on dB values
    let min_dB = spectrum.iter().map(|&(_, db)| db).fold(100.0, f64::min);
    let max_dB = spectrum.iter().map(|&(_, db)| db).fold(-100.0, f64::max);

    // --- Plotting ---
    let root_area = BitMapBackend::new("fft_spectrum_db.png", (1024, 768)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root_area)
        .caption("FFT Magnitude Spectrum (dB)", ("sans-serif", 30))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0..(sample_rate as f64 / 2.0), min_dB..max_dB)?;

    chart
        .configure_mesh()
        .x_desc("Frequency (Hz)")
        .y_desc("Magnitude (dB)")
        .draw()?;

    chart.draw_series(LineSeries::new(spectrum, &BLUE))?;

    root_area.present()?;
    println!("Saved FFT Magnitude Spectrum with dynamic range as 'fft_spectrum_db.png'");
    Ok(())
}
