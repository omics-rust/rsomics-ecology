use crate::counts::{Count, Counts, Sample};
use crate::rng;
use rayon::prelude::*;
use rsomics_common::{Context, Result, RsomicsError};

pub(crate) const PROFILE: &str = "skbio-0.7.3/biom-2.1.17/numpy-2.5.4-pcg64-choice-shuffle-false";

pub(crate) fn compute(counts: &mut Counts, depth: u64, seed: u128) -> Result<()> {
    for (id, sample) in counts.sample_ids.iter().zip(&counts.samples) {
        if depth > sample.total {
            return Err(RsomicsError::InvalidInput(format!(
                "sample {id:?} total {} is smaller than rarefaction depth {depth}",
                sample.total
            )));
        }
    }
    counts
        .samples
        .par_iter_mut()
        .zip(&counts.sample_ids)
        .try_for_each(|(sample, id)| {
            subsample(sample, depth, seed).rs_with_context(|| format!("rarefying sample {id:?}"))
        })
}

fn subsample(sample: &mut Sample, depth: u64, seed: u128) -> Result<()> {
    if depth == sample.total {
        return Ok(());
    }
    if depth == 0 {
        sample.entries = Vec::new();
        sample.total = 0;
        return Ok(());
    }
    let mut selected = rng::choose(sample.total, depth, seed)?;
    selected.sort_unstable();
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(sample.entries.len().min(selected.len()))
        .map_err(|error| {
            RsomicsError::ConfigError(format!("allocating rarefied sparse counts: {error}"))
        })?;
    let mut read = 0;
    let mut offset = 0;
    for entry in &sample.entries {
        let end = offset + entry.count;
        let start = read;
        while selected.get(read).is_some_and(|&index| index < end) {
            read += 1;
        }
        if read != start {
            entries.push(Count {
                feature: entry.feature,
                count: (read - start) as u64,
            });
        }
        offset = end;
    }
    sample.entries = entries;
    sample.total = depth;
    Ok(())
}
