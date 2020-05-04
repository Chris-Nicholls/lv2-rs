mod effect;


#[cfg(feature = "plot")]
use crate::effect::*;
#[cfg(feature = "plot")]
use plotters::prelude::*;

#[cfg(feature = "plot")]
use std::error::Error;

#[cfg(feature = "plot")]
const PRE_GAIN: f32 = 1000.0;
#[cfg(feature = "plot")]
const POST_GAIN: f32 = 1.0;

fn main() {
    #[cfg(feature = "plot")]
    plot().unwrap();
    // loop {
    #[cfg(feature = "plot")]
    audio().unwrap();
    // }
}

#[cfg(feature = "plot")]
fn audio() -> Result<(), Box<dyn Error>> {
    let mut reader = hound::WavReader::open("samples/5-notes.wav").unwrap();
    let mut spec = reader.spec();
    let r_cs = spec.channels;
    println!("CHANNELS: {}", spec.channels);
    spec.channels = 2;
    let mut writer = hound::WavWriter::create("samples/out.wav", spec).unwrap();

    let mut effect = Effect::new();

    let mut samples = reader.samples::<i16>();
    loop {
        let l = samples.next();
        if l.is_none() {
            break;
        }
        let l = l.unwrap().unwrap();
        let r = if r_cs == 1 {
            0
        } else {
            samples.next().unwrap().unwrap()
        };
        let s = sample_to_f32(l + r);
        let (l, r) = effect.add_sample(s * PRE_GAIN);

        writer
            .write_sample(f32_to_sample(l * MIX * POST_GAIN + s * (1.0 - MIX)))
            .expect("Could not write sample");
        writer
            .write_sample(f32_to_sample(r * MIX * POST_GAIN + s * (1.0 - MIX)))
            .expect("Could not write sample");
    }
    for _ in 0..SAMPLE_RATE * 10 {
        let (l, r) = effect.add_sample(0.0);
        writer
            .write_sample(f32_to_sample(l * MIX * POST_GAIN))
            .expect("Could not write sample");
        writer
            .write_sample(f32_to_sample(r * MIX * POST_GAIN))
            .expect("Could not write sample");
    }

    Ok(())
}

#[cfg(feature = "plot")]
fn plot() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let num_samples = SAMPLE_RATE * 10;
    let mut effect = Effect::new();
    let mut max: f32 = 0.0;
    effect.add_sample(1.0 * PRE_GAIN);
    let mut samples = Vec::new();
    for i in 0..num_samples {
        let s = effect.add_sample(0.0).0.abs() * POST_GAIN;
        let db = if s > 0.0 { s.log(10.0) * 20.0 } else { -90.0 };
        max = max.max(db);
        samples.push((i, db));
    }

    let root =
        BitMapBackend::new("samples/1.png", ((15.0 * 400.0) as u32, 1880)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_ranged(0..num_samples, -80.0f32..max)?;

    chart.configure_mesh().draw()?;

    // .filter(|f| f.vols.iter().find(|(_, x)| x > &ON_THRESHOLD).is_some())
    {
        let c = HSLColor(0.6, 0.8, 0.3);
        // chart.draw_series(LineSeries::new(f.vols_noise.to_vec(), &c))?;
        chart.draw_series(LineSeries::new(samples, &c)).unwrap();
        // .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &c));
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}
