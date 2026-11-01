use crate::counts::Sample;
use clap::ValueEnum;
use rsomics_common::{Context, Result, RsomicsError};
use rsomics_phylo_tree::{NodeId, NodeRef, Tree};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RootMode {
    #[default]
    Auto,
    Rooted,
    Unrooted,
}

#[derive(Serialize)]
pub(crate) struct PdOptions {
    pub root_mode: RootMode,
    pub effective_rooted: bool,
    pub weight: f64,
}

pub(crate) struct PreparedPhylogeny {
    tree: Tree,
    feature_tips: Vec<NodeId>,
    lengths: Vec<f64>,
    postorder_rank: Vec<usize>,
    tip_root_lengths: Vec<f64>,
    infinite_tip_count: usize,
    undefined_internal_path: bool,
    pub root_degree: usize,
}

impl PreparedPhylogeny {
    pub fn read(path: &Path, features: &[String], source_root_profile: bool) -> Result<Self> {
        let source = fs::read_to_string(path)
            .rs_with_context(|| format!("reading tree {}", path.display()))?;
        let tree = Tree::from_newick(&source)
            .map_err(|error| invalid(format!("tree {}: {error}", path.display())))?;
        let root = tree.root();
        let root_degree = tree
            .node(root)
            .expect("root belongs to this tree")
            .children()
            .len();
        if source_root_profile && root_degree > 2 {
            return Err(invalid(
                "Faith PD and UniFrac require source root-degree<=2",
            ));
        }
        let tips = tree
            .tip_index()
            .map_err(|error| invalid(error.to_string()))?;
        let feature_tips: Vec<NodeId> = features
            .iter()
            .map(|feature| {
                tips.get(feature.as_str())
                    .copied()
                    .filter(|&tip| tip != root)
                    .ok_or_else(|| {
                        invalid(format!(
                            "feature {feature:?} is not a non-root tip in the tree"
                        ))
                    })
            })
            .collect::<Result<_>>()?;
        let mut lengths = vec![0.0; tree.node_count()];
        let mut postorder_rank = vec![0; tree.node_count()];
        for (rank, id) in tree.postorder().enumerate() {
            let node = tree.node(id).expect("traversal node belongs to this tree");
            let length = match node.branch_length() {
                Some(length) if length.is_finite() && length >= 0.0 => length,
                None if id == root => 0.0,
                _ => {
                    return Err(invalid(format!(
                        "node {} requires a finite nonnegative branch length",
                        id.index()
                    )));
                }
            };
            lengths[rank] = length;
            postorder_rank[id.index()] = rank;
        }
        let mut root_lengths = vec![0.0; tree.node_count()];
        for id in tree.preorder() {
            let parent_length = tree
                .node(id)
                .expect("traversal node belongs to this tree")
                .parent()
                .map_or(0.0, |parent| root_lengths[parent.index()]);
            root_lengths[id.index()] = lengths[postorder_rank[id.index()]] + parent_length;
        }
        let mut tip_root_lengths = vec![0.0; tree.node_count()];
        let mut infinite_tip_count = 0;
        let mut undefined_internal_path = false;
        for id in tree.postorder() {
            let length = root_lengths[id.index()];
            if tree
                .node(id)
                .expect("traversal node belongs to this tree")
                .children()
                .is_empty()
            {
                tip_root_lengths[postorder_rank[id.index()]] = length;
                infinite_tip_count += usize::from(length.is_infinite());
            } else {
                undefined_internal_path |= !length.is_finite();
            }
        }
        Ok(Self {
            tree,
            feature_tips,
            lengths,
            postorder_rank,
            tip_root_lengths,
            infinite_tip_count,
            undefined_internal_path,
            root_degree,
        })
    }

    pub fn options(&self, mode: RootMode, weight: f64) -> PdOptions {
        PdOptions {
            root_mode: mode,
            effective_rooted: match mode {
                RootMode::Auto => self.root_degree == 2,
                RootMode::Rooted => true,
                RootMode::Unrooted => false,
            },
            weight,
        }
    }

    pub fn workspace(&self) -> Workspace {
        Workspace {
            counts: vec![0; self.tree.node_count()],
            active: vec![false; self.tree.node_count()],
            touched: Vec::new(),
        }
    }

    pub fn diversity(
        &self,
        sample: &Sample,
        options: Option<&PdOptions>,
        work: &mut Workspace,
    ) -> (f64, f64) {
        self.accumulate(sample, work);
        let mut faith = 0.0;
        let mut generalized = 0.0;
        let total = sample.total as f64;
        for &id in &work.touched {
            let count = work.counts[id.index()];
            let length = self.branch_length(self.postorder_rank[id.index()]);
            faith += length;
            if let Some(options) = options {
                let factor = if options.weight == 0.0 {
                    f64::from(options.effective_rooted || count < sample.total)
                } else {
                    let fraction = count as f64 / total;
                    let balance = if options.effective_rooted {
                        fraction
                    } else {
                        2.0 * fraction.min(1.0 - fraction)
                    };
                    if options.weight == 1.0 {
                        balance
                    } else {
                        balance.powf(options.weight)
                    }
                };
                generalized += length * factor;
            }
        }
        (faith, generalized)
    }

    fn accumulate(&self, sample: &Sample, work: &mut Workspace) {
        for id in work.touched.drain(..) {
            work.counts[id.index()] = 0;
            work.active[id.index()] = false;
        }
        for entry in &sample.entries {
            let tip = self.feature_tips[entry.feature];
            work.counts[tip.index()] = entry.count;
            let mut current = Some(tip);
            while let Some(id) = current {
                if work.active[id.index()] {
                    break;
                }
                work.active[id.index()] = true;
                work.touched.push(id);
                current = self.node(id).parent();
            }
        }
        work.touched
            .sort_unstable_by_key(|id| self.postorder_rank[id.index()]);
        for &id in &work.touched {
            if let Some(parent) = self.node(id).parent() {
                work.counts[parent.index()] += work.counts[id.index()];
            }
        }
    }

    pub fn branch_length(&self, rank: usize) -> f64 {
        self.lengths[rank]
    }

    pub fn tip_root_length(&self, rank: usize) -> f64 {
        self.tip_root_lengths[rank]
    }

    pub fn infinite_tip_count(&self) -> usize {
        self.infinite_tip_count
    }

    pub fn has_undefined_internal_path(&self) -> bool {
        self.undefined_internal_path
    }

    pub fn prepare_sample(
        &self,
        sample: &Sample,
        weighted: bool,
        work: &mut Workspace,
    ) -> Result<PreparedSample> {
        self.accumulate(sample, work);
        let mut branches = Vec::new();
        branches
            .try_reserve_exact(work.touched.len())
            .map_err(|error| invalid(format!("allocating sample branches: {error}")))?;
        for &id in &work.touched {
            branches.push(BranchValue {
                rank: self.postorder_rank[id.index()],
                value: if weighted {
                    work.counts[id.index()] as f64 / sample.total as f64
                } else {
                    1.0
                },
            });
        }
        Ok(PreparedSample {
            branches,
            total: sample.total,
        })
    }

    fn node(&self, id: NodeId) -> NodeRef<'_> {
        self.tree
            .node(id)
            .expect("prepared node belongs to this tree")
    }
}

pub(crate) struct Workspace {
    counts: Vec<u64>,
    active: Vec<bool>,
    touched: Vec<NodeId>,
}

#[derive(Default)]
pub(crate) struct PreparedSample {
    pub branches: Vec<BranchValue>,
    pub total: u64,
}

pub(crate) struct BranchValue {
    pub rank: usize,
    pub value: f64,
}

fn invalid(message: impl Into<String>) -> RsomicsError {
    RsomicsError::InvalidInput(message.into())
}
