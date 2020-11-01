use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::error::Error;
use std::hash::{Hash, Hasher};

use chrono::{DateTime, Utc};
use itertools::Itertools;
use plotters::prelude::*;

fn hash_sequence<V: Hash>(values: &[V]) -> u64 {
    let mut hasher = DefaultHasher::new();
    Hash::hash_slice(&values, &mut hasher);
    let digest = hasher.finish();

    return digest;
}

#[derive(Debug)]
pub struct Occurrence<Index: Eq + Ord, Metadata> {
    start_index: Index,
    start_meta: Metadata,
    end_index: Index,
    end_meta: Metadata,
}

pub struct Fingerprinter<Data, Index: Eq + Ord, Metadata, Value: Eq + Hash> {
    get_value: fn(&Data) -> Value,
    get_index: fn(&Data) -> Index,
    get_meta: fn(&Data) -> Metadata,
    window_size: usize,
    sequence_filter: fn(&[Value]) -> bool,
    occurrences: HashMap<u64, Vec<Occurrence<Index, Metadata>>>,
}

impl<Data, Index: Eq + Ord, Metadata, Value: Eq + Hash> Fingerprinter<Data, Index, Metadata, Value> {
    pub fn new(
        get_value: fn(&Data) -> Value,
        get_index: fn(&Data) -> Index,
        get_meta: fn(&Data) -> Metadata,
        window_size: usize,
        sequence_filter: fn(&[Value]) -> bool,
    ) -> Fingerprinter<Data, Index, Metadata, Value> {
        Fingerprinter {
            get_value,
            get_index,
            get_meta,
            window_size,
            sequence_filter,
            occurrences: HashMap::new(),
        }
    }

    pub fn process_series(
        &mut self,
        data: &[Data],
    ) {
        if data.len() < self.window_size {
            return;
        }

        self.occurrences = HashMap::new();
        let values = data.iter().map(self.get_value).collect_vec();

        for w_start in 0usize..(data.len() - self.window_size + 1) {
            let w_end = w_start + self.window_size;
            let w_data = &data[w_start..w_end];
            let w_values = &values[w_start..w_end];

            if !(self.sequence_filter)(&w_values) {
                continue;
            }

            let digest = hash_sequence(&w_values);
            let other_occurrences = self.occurrences.entry(digest).or_insert(vec![]);

            let first_record = &w_data[0];
            let last_record = &w_data[w_data.len() - 1];

            other_occurrences.push(Occurrence {
                start_index: (self.get_index)(first_record),
                start_meta: (self.get_meta)(first_record),
                end_index: (self.get_index)(last_record),
                end_meta: (self.get_meta)(last_record),
            });
        }
    }

    pub fn matches(&self, sequence: &[Value]) -> Option<impl Iterator<Item=&Occurrence<Index, Metadata>>> {
        let digest = hash_sequence(sequence);

        if let Some(matches) = self.occurrences.get(&digest) {
            return Some(matches.iter());
        } else {
            return None;
        }
    }

    pub fn duplicates(&self) -> impl Iterator<Item=&Vec<Occurrence<Index, Metadata>>> {
        return self.occurrences.values().filter(|v| v.len() > 1)
    }
}

pub fn plot_timeseries<T, M>(
    output_file: &str,
    series_name: &str,
    get_value: fn(&T) -> f64,
    get_index: fn(&T) -> DateTime<Utc>,
    data: &[T],
    duplicates: &[&Vec<Occurrence<DateTime<Utc>, M>>]
) -> Result<(), Box<dyn Error>> {
    let first_index = get_index(&data[0]);
    let last_index = get_index(&data[data.len() - 1]);

    let values = data.iter().map(get_value).collect_vec();
    let min_value = values.iter().cloned().fold(f64::MAX, f64::min);
    let max_value = values.iter().cloned().fold(f64::MIN, f64::max);

    let backend = SVGBackend::new(&output_file, (1280, 720)).into_drawing_area();
    backend.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&backend)
        .caption(series_name, ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(first_index..last_index, min_value..max_value)?;

    chart.configure_mesh().draw()?;

    let points = data.iter().map(|x| (get_index(x), get_value(x))).collect_vec();
    chart
        .draw_series(LineSeries::new(
            points,
            &BLUE,
        ))?
        .label(series_name)
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    // Draw fingerprint markings
    if duplicates.len() > 0 {
        // to illustrate duplicates it is enough to plot just one type of sequnce
        for occurrence in duplicates[0] {
            chart.draw_series(std::iter::once(
                Rectangle::new(
                    [
                        (occurrence.start_index, min_value),
                        (occurrence.end_index, max_value)
                    ],
                    RED.mix(0.5).filled()
                )
            ))?;
        }
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    return Ok(());
}
