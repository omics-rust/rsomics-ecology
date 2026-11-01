use rayon::prelude::*;
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;
use serde::ser::{SerializeSeq, SerializeStruct, Serializer};
use std::io::Write;

pub(crate) const SOURCE_PROFILE: &str = "skbio-0.7.3/scipy-1.18.1";
pub(crate) const TREE_PROFILE: &str = "source-root-degree<=2/stored-root-length/strict-nonnegative";

pub(crate) struct DistanceMatrix {
    ids: Vec<String>,
    values: Vec<f64>,
    metric: &'static str,
    tree_profile: Option<&'static str>,
}

impl DistanceMatrix {
    pub fn compute(
        ids: &[String],
        metric: &'static str,
        phylogenetic: bool,
        pair: impl Fn(usize, usize) -> f64 + Sync,
    ) -> Result<Self> {
        let n = ids.len();
        let size = n
            .checked_mul(n)
            .ok_or_else(|| allocation("matrix dimensions overflow"))?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(size)
            .map_err(|error| allocation(error.to_string()))?;
        values.resize(size, 0.0);
        let failure = values
            .par_chunks_mut(n)
            .enumerate()
            .filter_map(|(i, row)| {
                for (j, cell) in row.iter_mut().enumerate().skip(i + 1) {
                    let value = pair(i, j);
                    if !value.is_finite() || value < 0.0 {
                        return Some((i, j));
                    }
                    *cell = value;
                }
                None
            })
            .min();
        if let Some((i, j)) = failure {
            return Err(RsomicsError::InvalidInput(format!(
                "{metric}: undefined or nonfinite distance between samples {:?} and {:?}",
                ids[i], ids[j]
            )));
        }
        for i in 0..n {
            for j in i + 1..n {
                values[j * n + i] = values[i * n + j];
            }
        }
        Ok(Self {
            ids: ids.to_vec(),
            values,
            metric,
            tree_profile: phylogenetic.then_some(TREE_PROFILE),
        })
    }

    fn rows(&self) -> impl ExactSizeIterator<Item = &[f64]> {
        self.values.chunks(self.ids.len())
    }

    pub fn write_tsv(&self, output: &mut dyn Write) -> Result<()> {
        for id in &self.ids {
            write!(output, "\t{id}")?;
        }
        writeln!(output)?;
        for (id, row) in self.ids.iter().zip(self.rows()) {
            write!(output, "{id}")?;
            for value in row {
                write!(output, "\t{value}")?;
            }
            writeln!(output)?;
        }
        Ok(())
    }
}

struct Rows<'a>(&'a DistanceMatrix);

impl Serialize for Rows<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut rows = serializer.serialize_seq(Some(self.0.ids.len()))?;
        for row in self.0.rows() {
            rows.serialize_element(row)?;
        }
        rows.end()
    }
}

impl Serialize for DistanceMatrix {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut report = serializer.serialize_struct("DistanceMatrix", 6)?;
        report.serialize_field("schema_version", &1u8)?;
        report.serialize_field("metric", self.metric)?;
        report.serialize_field("source_profile", SOURCE_PROFILE)?;
        report.serialize_field("tree_profile", &self.tree_profile)?;
        report.serialize_field("sample_ids", &self.ids)?;
        report.serialize_field("values", &Rows(self))?;
        report.end()
    }
}

fn allocation(message: impl std::fmt::Display) -> RsomicsError {
    RsomicsError::ConfigError(format!("allocating distance matrix: {message}"))
}
