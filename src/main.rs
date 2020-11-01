extern crate chrono;
extern crate plotters;
extern crate rand;

use std::error::Error;
use std::f64::consts::PI;
use std::{process};

use chrono::prelude::*;
use chrono::Duration;
use itertools::Itertools;
use rand::Rng;

mod fingerprint;

use fingerprint::*;


fn example() -> Result<(), Box<dyn Error>> {
    let mut fp: Fingerprinter<(DateTime<Utc>, f64), DateTime<Utc>, (), i64> = Fingerprinter::new(
        // 5 decimal precision to find matches
        |&(_, v)| (v * 10_000.0).round() as i64,
        |&(i, _)| i,
        |_| (),
        100,
        |_| true,
    );

    let start_time = Utc.ymd(2020, 11, 1).and_hms(0, 0, 0);

    // sin(x)
    let sin_data = (0..10_000).into_iter().map(
        |x| (start_time + Duration::seconds(x), (x as f64 / 600.0) * 2.0 * PI)
    ).map(
        |(i, v)| (i, f64::sin(v))
    ).collect_vec();
    fp.process_series(&sin_data);
    let duplicates = fp.duplicates().collect_vec();
    plot_timeseries(
        "./sin.svg",
        "sin(x)",
        |&(_, v)| v,
        |&(i, _)| i,
        &sin_data,
        &duplicates,
    )?;

    // sinc(x) - shouldn't have repeating sequences
    let sinc_data = (-5_000..5_000).into_iter().map(
        |x| (start_time + Duration::seconds(x), (x as f64 / 600.0) * 2.0 * PI)
    ).map(
        |(i, v)| (i, f64::sin(v) / v)
    ).collect_vec();
    fp.process_series(&sinc_data);
    let duplicates = fp.duplicates().collect_vec();
    plot_timeseries(
        "./sinc.svg",
        "sinc(x)",
        |&(_, v)| v,
        |&(i, _)| i,
        &sinc_data,
        &duplicates,
    )?;

    // random data repeated
    let mut rng = rand::thread_rng();
    let mut random_data: Vec<f64> = (0..5_000).into_iter().map(|_| rng.gen()).collect_vec();
    random_data.extend(random_data.clone());
    let datetime = (0..10_000).into_iter().map(
        |x| start_time + Duration::seconds(x)
    );
    let combined = datetime.zip(random_data.iter().cloned()).collect_vec();
    fp.process_series(&combined);
    let duplicates = fp.duplicates().collect_vec();
    plot_timeseries(
        "./random.svg",
        "random()",
        |&(_, v)| v,
        |&(i, _)| i,
        &combined,
        &duplicates,
    )?;

    Ok(())
}

fn main() {
    if let Err(err) = example() {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
