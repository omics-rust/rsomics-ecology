use rsomics_common::{Context, Result, RsomicsError};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

const MAX_EXACT_INTEGER: u64 = (1 << 53) - 1;

#[derive(Clone, Copy)]
pub(crate) struct Count {
    pub feature: usize,
    pub count: u64,
}

#[derive(Default)]
pub(crate) struct Sample {
    pub entries: Vec<Count>,
    pub total: u64,
}

pub(crate) struct Counts {
    pub feature_ids: Vec<String>,
    pub sample_ids: Vec<String>,
    pub samples: Vec<Sample>,
}

impl Counts {
    pub fn read(path: &Path) -> Result<Self> {
        let input =
            File::open(path).rs_with_context(|| format!("opening counts {}", path.display()))?;
        Self::parse(BufReader::new(input))
            .rs_with_context(|| format!("reading counts {}", path.display()))
    }

    fn parse(mut input: impl BufRead) -> Result<Self> {
        let mut line = String::new();
        input.read_line(&mut line)?;
        let mut fields = record(&line).split('\t');
        if !matches!(fields.next(), Some("feature" | "#OTU ID")) {
            return Err(invalid("first TSV header must be feature or #OTU ID"));
        }
        let sample_ids: Vec<_> = fields.map(str::to_owned).collect();
        if sample_ids.is_empty() {
            return Err(invalid("count table must name at least one sample"));
        }
        let mut identities = HashSet::new();
        for sample in &sample_ids {
            validate_id(sample, "sample")?;
            if !identities.insert(sample) {
                return Err(invalid(format!("duplicate sample ID {sample:?}")));
            }
        }
        let mut samples: Vec<_> = sample_ids.iter().map(|_| Sample::default()).collect();
        let mut feature_ids = Vec::new();
        let mut features = HashSet::new();
        let mut row = 1;
        loop {
            line.clear();
            if input.read_line(&mut line)? == 0 {
                break;
            }
            row += 1;
            let mut fields = record(&line).split('\t');
            let feature = fields.next().unwrap_or("");
            validate_id(feature, "feature").rs_with_context(|| format!("row {row}"))?;
            if !features.insert(feature.to_owned()) {
                return Err(invalid(format!(
                    "row {row}: duplicate feature ID {feature:?}"
                )));
            }
            for (column, sample) in samples.iter_mut().enumerate() {
                let text = fields.next().ok_or_else(|| {
                    invalid(format!(
                        "row {row}: missing count for sample {:?}",
                        sample_ids[column]
                    ))
                })?;
                let text = text.trim();
                let count = if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
                    None
                } else {
                    text.parse::<u64>().ok().filter(|&value| value <= MAX_EXACT_INTEGER)
                }
                .ok_or_else(|| invalid(format!("row {row}, sample {:?}: expected an integer count in 0..={MAX_EXACT_INTEGER}, got {text:?}", sample_ids[column])))?;
                sample.total = sample
                    .total
                    .checked_add(count)
                    .filter(|&value| value <= MAX_EXACT_INTEGER)
                    .ok_or_else(|| {
                        invalid(format!(
                            "row {row}: sample {:?} total exceeds {MAX_EXACT_INTEGER}",
                            sample_ids[column]
                        ))
                    })?;
                if count != 0 {
                    sample.entries.push(Count {
                        feature: feature_ids.len(),
                        count,
                    });
                }
            }
            if fields.next().is_some() {
                return Err(invalid(format!("row {row}: more columns than the header")));
            }
            feature_ids.push(feature.to_owned());
        }
        Ok(Self {
            feature_ids,
            sample_ids,
            samples,
        })
    }

    pub fn nonzeros(&self) -> usize {
        self.samples.iter().map(|sample| sample.entries.len()).sum()
    }

    pub fn write_tsv(&self, output: &mut dyn Write) -> Result<()> {
        let mut cursors = Vec::new();
        cursors
            .try_reserve_exact(self.samples.len())
            .map_err(|error| {
                RsomicsError::ConfigError(format!("allocating count-table cursors: {error}"))
            })?;
        cursors.resize(self.samples.len(), 0);
        write!(output, "feature")?;
        for sample in &self.sample_ids {
            write!(output, "\t{sample}")?;
        }
        writeln!(output)?;
        for (feature, id) in self.feature_ids.iter().enumerate() {
            write!(output, "{id}")?;
            for (sample, cursor) in self.samples.iter().zip(&mut cursors) {
                let count = match sample.entries.get(*cursor) {
                    Some(entry) if entry.feature == feature => {
                        *cursor += 1;
                        entry.count
                    }
                    _ => 0,
                };
                write!(output, "\t{count}")?;
            }
            writeln!(output)?;
        }
        Ok(())
    }
}

fn record(line: &str) -> &str {
    let line = line.strip_suffix('\n').unwrap_or(line);
    line.strip_suffix('\r').unwrap_or(line)
}

fn validate_id(id: &str, role: &str) -> Result<()> {
    if id.is_empty() || id.trim() != id || id.chars().any(char::is_control) {
        Err(invalid(format!(
            "{role} ID must be nonempty and have no control characters or surrounding whitespace"
        )))
    } else {
        Ok(())
    }
}

fn invalid(message: impl Into<String>) -> RsomicsError {
    RsomicsError::InvalidInput(message.into())
}
