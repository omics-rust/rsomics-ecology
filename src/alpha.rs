use crate::counts::{Counts, Sample};
use crate::phylogeny::{PdOptions, PreparedPhylogeny};
use clap::ValueEnum;
use rayon::prelude::*;
use serde::Serialize;
use std::io::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
#[value(rename_all = "snake_case")]
pub(crate) enum Metric {
    ObservedFeatures,
    Shannon,
    Simpson,
    Dominance,
    InvSimpson,
    SimpsonE,
    PielouE,
    Chao1,
    GoodsCoverage,
    FaithPd,
    Phydiv,
}

impl Metric {
    pub fn name(self) -> &'static str {
        match self {
            Self::ObservedFeatures => "observed_features",
            Self::Shannon => "shannon",
            Self::Simpson => "simpson",
            Self::Dominance => "dominance",
            Self::InvSimpson => "inv_simpson",
            Self::SimpsonE => "simpson_e",
            Self::PielouE => "pielou_e",
            Self::Chao1 => "chao1",
            Self::GoodsCoverage => "goods_coverage",
            Self::FaithPd => "faith_pd",
            Self::Phydiv => "phydiv",
        }
    }

    pub fn phylogenetic(self) -> bool {
        matches!(self, Self::FaithPd | Self::Phydiv)
    }
}

#[derive(Serialize)]
pub(crate) struct AlphaValue {
    value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<UnavailableReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nonfinite: Option<Nonfinite>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum UnavailableReason {
    EmptyCommunity,
    NonfiniteResult,
}

#[derive(Serialize)]
enum Nonfinite {
    #[serde(rename = "nan")]
    Nan,
    #[serde(rename = "+inf")]
    PositiveInfinity,
    #[serde(rename = "-inf")]
    NegativeInfinity,
}

impl AlphaValue {
    fn finite(value: f64) -> Self {
        if value.is_finite() {
            Self {
                value: Some(value),
                reason: None,
                nonfinite: None,
            }
        } else {
            Self {
                value: None,
                reason: Some(UnavailableReason::NonfiniteResult),
                nonfinite: Some(if value.is_nan() {
                    Nonfinite::Nan
                } else if value.is_sign_positive() {
                    Nonfinite::PositiveInfinity
                } else {
                    Nonfinite::NegativeInfinity
                }),
            }
        }
    }

    fn empty() -> Self {
        Self {
            value: None,
            reason: Some(UnavailableReason::EmptyCommunity),
            nonfinite: Some(Nonfinite::Nan),
        }
    }
}

struct Summary {
    observed: f64,
    total: f64,
    singles: f64,
    doubles: f64,
    shannon: f64,
    dominance: f64,
}

impl Summary {
    fn new(sample: &Sample) -> Self {
        let total = sample.total as f64;
        let mut singles = 0;
        let mut doubles = 0;
        let mut shannon = 0.0;
        let mut dominance = 0.0;
        for entry in &sample.entries {
            singles += usize::from(entry.count == 1);
            doubles += usize::from(entry.count == 2);
            let proportion = entry.count as f64 / total;
            shannon -= proportion * proportion.ln();
            dominance += proportion * proportion;
        }
        Self {
            observed: sample.entries.len() as f64,
            total,
            singles: singles as f64,
            doubles: doubles as f64,
            shannon,
            dominance,
        }
    }

    fn value(&self, metric: Metric, faith: f64, generalized: f64) -> AlphaValue {
        if self.total == 0.0
            && !matches!(
                metric,
                Metric::ObservedFeatures | Metric::Chao1 | Metric::FaithPd | Metric::Phydiv
            )
        {
            return AlphaValue::empty();
        }
        AlphaValue::finite(match metric {
            Metric::ObservedFeatures => self.observed,
            Metric::Shannon => self.shannon,
            Metric::Dominance => self.dominance,
            Metric::Simpson => 1.0 - self.dominance,
            Metric::InvSimpson => 1.0 / self.dominance,
            Metric::SimpsonE => 1.0 / (self.observed * self.dominance),
            Metric::PielouE if self.observed == 1.0 => 1.0,
            Metric::PielouE => self.shannon / self.observed.ln(),
            Metric::Chao1 => {
                self.observed + self.singles * (self.singles - 1.0) / (2.0 * (self.doubles + 1.0))
            }
            Metric::GoodsCoverage => 1.0 - self.singles / self.total,
            Metric::FaithPd => faith,
            Metric::Phydiv => generalized,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct Report<'a> {
    schema_version: u8,
    feature_ids: &'a [String],
    sample_ids: &'a [String],
    metrics: &'a [Metric],
    pd: Option<PdOptions>,
    values: Vec<Vec<AlphaValue>>,
}

pub(crate) fn compute<'a>(
    counts: &'a Counts,
    metrics: &'a [Metric],
    tree: Option<&PreparedPhylogeny>,
    pd: Option<PdOptions>,
    workers: usize,
) -> Report<'a> {
    let chunk_size = counts.samples.len().div_ceil(workers);
    let values = counts
        .samples
        .par_chunks(chunk_size)
        .flat_map_iter(|samples| {
            let mut workspace = tree.map(PreparedPhylogeny::workspace);
            samples
                .iter()
                .map(|sample| {
                    let summary = Summary::new(sample);
                    let (faith, generalized) = match (tree, workspace.as_mut()) {
                        (Some(tree), Some(workspace)) => {
                            tree.diversity(sample, pd.as_ref(), workspace)
                        }
                        _ => (0.0, 0.0),
                    };
                    metrics
                        .iter()
                        .map(|&metric| summary.value(metric, faith, generalized))
                        .collect()
                })
                .collect::<Vec<_>>()
        })
        .collect();
    Report {
        schema_version: 1,
        feature_ids: &counts.feature_ids,
        sample_ids: &counts.sample_ids,
        metrics,
        pd,
        values,
    }
}

impl Report<'_> {
    pub fn write_tsv(&self, output: &mut dyn Write) -> std::io::Result<()> {
        write!(output, "sample")?;
        for metric in self.metrics {
            write!(output, "\t{}", metric.name())?;
        }
        writeln!(output)?;
        for (sample, values) in self.sample_ids.iter().zip(&self.values) {
            write!(output, "{sample}")?;
            for value in values {
                if let Some(number) = value.value {
                    write!(output, "\t{number}")?;
                } else {
                    let text = match value.nonfinite.as_ref() {
                        Some(Nonfinite::PositiveInfinity) => "inf",
                        Some(Nonfinite::NegativeInfinity) => "-inf",
                        _ => "nan",
                    };
                    write!(output, "\t{text}")?;
                }
            }
            writeln!(output)?;
        }
        Ok(())
    }
}
