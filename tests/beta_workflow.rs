use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const METRICS: [&str; 8] = [
    "braycurtis",
    "jaccard",
    "euclidean",
    "canberra",
    "cityblock",
    "unweighted_unifrac",
    "weighted_unifrac",
    "weighted_normalized_unifrac",
];
const ALPHA: [&str; 11] = [
    "observed_features",
    "shannon",
    "simpson",
    "dominance",
    "inv_simpson",
    "simpson_e",
    "pielou_e",
    "chao1",
    "goods_coverage",
    "faith_pd",
    "phydiv",
];

fn fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/beta")
        .join(relative)
}

fn load(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn input(request: &Value, counts: &Path, tree: &Path) {
    let mut text = String::from("feature");
    for id in request["sample_ids"].as_array().unwrap() {
        text.push('\t');
        text.push_str(id.as_str().unwrap());
    }
    text.push('\n');
    for (feature, id) in request["feature_ids"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        text.push_str(id.as_str().unwrap());
        for sample in request["counts"].as_array().unwrap() {
            text.push('\t');
            text.push_str(&sample[feature].to_string());
        }
        text.push('\n');
    }
    fs::write(counts, text).unwrap();
    fs::write(tree, request["tree"].as_str().unwrap()).unwrap();
}

fn beta(
    counts: &Path,
    tree: &Path,
    output: &Path,
    metric: &str,
    workers: &str,
    format: &str,
) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rsomics-ecology"));
    command
        .args(["--json", "--threads", workers, "beta", "--counts"])
        .arg(counts)
        .arg("--output")
        .arg(output)
        .args(["--metric", metric, "--format", format]);
    if metric.contains("unifrac") {
        command.arg("--tree").arg(tree);
    }
    command.output().unwrap()
}

fn success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["status"], "ok");
    envelope
}

fn near(actual: f64, expected: f64) {
    assert!(actual.is_finite() && actual >= 0.0);
    assert!(
        (actual - expected).abs() <= 1e-10 + 1e-10 * expected.abs(),
        "{actual} != {expected}"
    );
}

fn matrix(actual: &Value, request: &Value, expected: &Value, metric: &str) {
    assert_eq!(actual["schema_version"], 1);
    assert_eq!(actual["metric"], metric);
    assert_eq!(actual["source_profile"], "skbio-0.7.3/scipy-1.18.1");
    assert_eq!(
        actual["tree_profile"],
        if metric.contains("unifrac") {
            json!("source-root-degree<=2/stored-root-length/strict-nonnegative")
        } else {
            Value::Null
        }
    );
    assert_eq!(actual["sample_ids"], request["sample_ids"]);
    assert_eq!(expected["ids"], request["sample_ids"]);
    let n = request["sample_ids"].as_array().unwrap().len();
    assert_eq!(expected["shape"], json!([n, n]));
    let values = actual["values"].as_array().unwrap();
    assert_eq!(values.len(), n);
    for (i, row) in values.iter().enumerate() {
        assert_eq!(row.as_array().unwrap().len(), n);
        for j in 0..n {
            near(
                row[j].as_f64().unwrap(),
                expected["values"][i][j].as_f64().unwrap(),
            );
            assert_eq!(row[j], values[j][i]);
            if i == j {
                assert_eq!(row[j], 0.0);
            }
        }
    }
}

fn tsv(path: &Path, actual: &Value) {
    let text = fs::read_to_string(path).unwrap();
    let rows: Vec<Vec<_>> = text.lines().map(|row| row.split('\t').collect()).collect();
    let ids = actual["sample_ids"].as_array().unwrap();
    assert_eq!(rows.len(), ids.len() + 1);
    assert_eq!(rows[0][0], "");
    assert_eq!(rows[0].len(), ids.len() + 1);
    for (i, id) in ids.iter().enumerate() {
        assert_eq!(rows[0][i + 1], id.as_str().unwrap());
        assert_eq!(rows[i + 1][0], id.as_str().unwrap());
        assert_eq!(rows[i + 1].len(), ids.len() + 1);
        for j in 0..ids.len() {
            near(
                rows[i + 1][j + 1].parse::<f64>().unwrap(),
                actual["values"][i][j].as_f64().unwrap(),
            );
        }
    }
}

#[test]
fn original_rarefied_reload_all_eight_matrices_and_alpha_match_frozen_source() {
    let directory = tempfile::tempdir().unwrap();
    let counts = directory.path().join("counts.tsv");
    let tree = directory.path().join("tree.nwk");
    let output = directory.path().join("matrix.json");
    let text = directory.path().join("matrix.tsv");
    let original = load(&fixture("cases/original/request.json"));
    input(&original, &counts, &tree);
    for case in ["cases/original", "cases/rarefied/diversity"] {
        let request = load(&fixture(&format!("{case}/request.json")));
        if case.contains("rarefied") {
            let rarefied = directory.path().join("rarefied.tsv");
            let mut prior = None;
            for workers in ["1", "4"] {
                success(
                    Command::new(env!("CARGO_BIN_EXE_rsomics-ecology"))
                        .args(["--json", "--threads", workers, "rarefy", "--counts"])
                        .arg(&counts)
                        .arg("--output")
                        .arg(&rarefied)
                        .args(["--depth", "8", "--seed", "42"])
                        .output()
                        .unwrap(),
                );
                let bytes = fs::read(&rarefied).unwrap();
                let expected = fs::read_to_string(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("tests/data/rarefaction/primary-expected.tsv"),
                )
                .unwrap()
                .replacen("#OTU ID\t", "feature\t", 1);
                assert_eq!(bytes, expected.as_bytes());
                if let Some(prior) = &prior {
                    assert_eq!(&bytes, prior);
                }
                prior = Some(bytes);
            }
            fs::copy(rarefied, &counts).unwrap();
        }
        for metric in METRICS {
            let observation = load(&fixture(&format!("{case}/beta/{metric}/observation.json")));
            assert_eq!(observation["status"], "returned");
            let mut previous = None;
            let mut previous_tsv = None;
            for workers in ["1", "4"] {
                let envelope = success(beta(&counts, &tree, &output, metric, workers, "json"));
                assert_eq!(
                    envelope["result"]["workers"],
                    workers.parse::<u64>().unwrap()
                );
                assert_eq!(envelope["result"]["beta"]["metric"], metric);
                let actual = load(&output);
                matrix(&actual, &request, &observation["result"], metric);
                success(beta(&counts, &tree, &text, metric, workers, "tsv"));
                tsv(&text, &actual);
                let bytes = fs::read(&output).unwrap();
                let tsv_bytes = fs::read(&text).unwrap();
                if let Some(previous) = &previous {
                    assert_eq!(&bytes, previous);
                }
                if let Some(previous) = &previous_tsv {
                    assert_eq!(&tsv_bytes, previous);
                }
                previous = Some(bytes);
                previous_tsv = Some(tsv_bytes);
            }
        }
        for workers in ["1", "4"] {
            success(
                Command::new(env!("CARGO_BIN_EXE_rsomics-ecology"))
                    .args(["--json", "--threads", workers, "alpha", "--counts"])
                    .arg(&counts)
                    .arg("--tree")
                    .arg(&tree)
                    .arg("--output")
                    .arg(&output)
                    .args(["--metrics", &ALPHA.join(","), "--format", "json"])
                    .output()
                    .unwrap(),
            );
            let actual = load(&output);
            assert_eq!(actual["sample_ids"], request["sample_ids"]);
            assert_eq!(actual["feature_ids"], request["feature_ids"]);
            for (column, metric) in ALPHA.iter().enumerate() {
                let source = if *metric == "phydiv" {
                    "phydiv-auto"
                } else {
                    metric
                };
                let expected = load(&fixture(&format!("{case}/alpha/{source}/observation.json")));
                for (row, value) in expected["result"]["values"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                {
                    near(
                        actual["values"][row][column]["value"].as_f64().unwrap(),
                        value.as_f64().unwrap(),
                    );
                }
            }
        }
    }
}

fn failure(result: Output, output: &Path, metric: &str, pair: Option<(&str, &str)>) -> String {
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    let error: Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(error["status"], "error");
    let message = String::from_utf8(result.stderr).unwrap();
    if let Some((a, b)) = pair {
        for required in [metric, a, b] {
            assert!(message.contains(required), "{message}");
        }
    }
    assert_eq!(fs::read_to_string(output).unwrap(), "keep\n");
    message
}

#[test]
fn source_boundaries_strict_validation_and_undefined_pairs_are_atomic() {
    let directory = tempfile::tempdir().unwrap();
    let counts = directory.path().join("counts.tsv");
    let tree = directory.path().join("tree.nwk");
    let output = directory.path().join("matrix.json");
    for case in [
        "zero_samples",
        "no_features",
        "root_length",
        "zero_lengths",
        "extra_tips",
        "unary",
        "polytomy",
    ] {
        let base = format!("source-boundaries/{case}");
        let request = load(&fixture(&format!("{base}/request.json")));
        input(&request, &counts, &tree);
        for metric in METRICS {
            let path = fixture(&format!("{base}/beta/{metric}/observation.json"));
            if !path.exists() {
                assert!(
                    matches!(
                        case,
                        "root_length" | "zero_lengths" | "extra_tips" | "unary" | "polytomy"
                    ) && !metric.contains("unifrac")
                );
                continue;
            }
            let observation = load(&path);
            let values = observation["result"]["values"].as_array();
            let undefined = values.and_then(|rows| {
                rows.iter().enumerate().find_map(|(i, row)| {
                    row.as_array()
                        .unwrap()
                        .iter()
                        .enumerate()
                        .skip(i + 1)
                        .find(|(_, value)| value.is_object())
                        .map(|(j, _)| (i, j))
                })
            });
            let mut previous = None;
            for workers in ["1", "4"] {
                fs::write(&output, "keep\n").unwrap();
                let result = beta(&counts, &tree, &output, metric, workers, "json");
                if observation["status"] != "returned" || undefined.is_some() {
                    let pair = undefined.map(|(i, j)| {
                        (
                            request["sample_ids"][i].as_str().unwrap(),
                            request["sample_ids"][j].as_str().unwrap(),
                        )
                    });
                    let message = failure(result, &output, metric, pair);
                    if let Some(previous) = &previous {
                        assert_eq!(&message, previous);
                    }
                    previous = Some(message);
                    let missing = directory.path().join("not-created.json");
                    assert!(
                        !beta(&counts, &tree, &missing, metric, workers, "json")
                            .status
                            .success()
                    );
                    assert!(!missing.exists());
                } else {
                    success(result);
                    matrix(&load(&output), &request, &observation["result"], metric);
                }
            }
        }
    }
    fs::write(&tree, "(A:0,B:0)r;").unwrap();
    for content in [
        "feature\tfirst\tsecond\nA\t1\t1\n",
        "feature\tempty-a\tempty-b\tpositive\nA\t0\t0\t1\n",
    ] {
        fs::write(&counts, content).unwrap();
        fs::write(&output, "keep\n").unwrap();
        failure(
            beta(
                &counts,
                &tree,
                &output,
                "weighted_normalized_unifrac",
                "4",
                "json",
            ),
            &output,
            "weighted_normalized_unifrac",
            Some(if content.contains("first") {
                ("first", "second")
            } else {
                ("empty-a", "positive")
            }),
        );
    }
    for count in [0, 1] {
        fs::write(&counts, format!("feature\tonly\nA\t{count}\n")).unwrap();
        for metric in METRICS {
            success(beta(&counts, &tree, &output, metric, "4", "json"));
        }
    }
    fs::write(&counts, "feature\tempty\tpositive\nA\t0\t1\n").unwrap();
    fs::write(&tree, "(A:1,B:2)r:0.5;").unwrap();
    for (metric, expected) in METRICS
        .into_iter()
        .zip([1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.5, 1.0])
    {
        success(beta(&counts, &tree, &output, metric, "4", "json"));
        near(load(&output)["values"][0][1].as_f64().unwrap(), expected);
    }
    for content in [
        "feature\n",
        "feature\ta\ta\nA\t1\t2\n",
        "feature\ta\tb\nA\t1\t0\nA\t0\t1\n",
        "feature\ta\tb\nA\t-1\t0\n",
        "feature\ta\tb\nA\t1.2\t0\n",
        "feature\ta\tb\nA\t9007199254740992\t0\n",
        "feature\ta\tb\nA\t9007199254740991\t0\nB\t1\t0\n",
        "feature\ta\tb\nA\t1\n",
    ] {
        fs::write(&counts, content).unwrap();
        fs::write(&output, "keep\n").unwrap();
        failure(
            beta(&counts, &tree, &output, "euclidean", "4", "json"),
            &output,
            "euclidean",
            None,
        );
    }
    fs::write(&counts, "feature\ta\tb\nA\t1\t0\nB\t0\t1\n").unwrap();
    for invalid in [
        "(A:1,B:1,C:1)r;",
        "(A:1,C:1)r;",
        "(A:1,A:1)r;",
        "(A,B:1)r;",
        "(A:-1,B:1)r;",
        "(A:NaN,B:1)r;",
        "(A:1,:1)r;",
    ] {
        fs::write(&tree, invalid).unwrap();
        fs::write(&output, "keep\n").unwrap();
        failure(
            beta(&counts, &tree, &output, "unweighted_unifrac", "4", "json"),
            &output,
            "unweighted_unifrac",
            None,
        );
    }
    fs::write(&tree, "(A:1,B:1)r;").unwrap();
    let alias = directory.path().join("alias.tsv");
    fs::hard_link(&counts, &alias).unwrap();
    let prior = fs::read(&counts).unwrap();
    for path in [&counts, &alias, &tree] {
        assert!(
            !beta(&counts, &tree, path, "unweighted_unifrac", "4", "json")
                .status
                .success()
        );
    }
    assert_eq!(fs::read(&counts).unwrap(), prior);
    // Strict validation still precedes the source's zero-feature fast return.
    fs::write(&counts, "feature\ta\tb\n").unwrap();
    fs::write(&tree, "(A:1,B:1,C:1)r;").unwrap();
    fs::write(&output, "keep\n").unwrap();
    failure(
        beta(&counts, &tree, &output, "unweighted_unifrac", "4", "json"),
        &output,
        "unweighted_unifrac",
        None,
    );
    // These two outcomes derive from retained source arithmetic/IEEE, not a new upstream capture.
    let mut review_failures = Vec::new();
    for (name, table, newick, undefined) in [
        (
            "subnormal",
            "feature\tfirst\tsecond\nA\t1\t1\nB\t1\t1\n",
            "(A:5e-324,B:0)r;",
            false,
        ),
        (
            "nonfinite-extra-tip-path",
            "feature\tfirst\tsecond\nC\t1\t1\n",
            "((A:1e308,B:1e308)x:1e308,C:1)r;",
            true,
        ),
    ] {
        fs::write(&counts, table).unwrap();
        fs::write(&tree, newick).unwrap();
        let mut previous = None;
        for workers in ["1", "4"] {
            fs::write(&output, "keep\n").unwrap();
            let result = beta(
                &counts,
                &tree,
                &output,
                "weighted_normalized_unifrac",
                workers,
                "json",
            );
            if result.status.success() == undefined {
                review_failures.push(format!(
                    "{name}, workers={workers}: expected {}, got success={}: {}",
                    if undefined {
                        "pair-specific failure"
                    } else {
                        "defined zero matrix"
                    },
                    result.status.success(),
                    String::from_utf8_lossy(&result.stderr)
                ));
                continue;
            }
            let bytes = if undefined {
                failure(
                    result,
                    &output,
                    "weighted_normalized_unifrac",
                    Some(("first", "second")),
                )
                .into_bytes()
            } else {
                success(result);
                let actual = load(&output);
                assert_eq!(actual["sample_ids"], json!(["first", "second"]));
                assert_eq!(actual["values"], json!([[0.0, 0.0], [0.0, 0.0]]));
                fs::read(&output).unwrap()
            };
            if let Some(previous) = &previous {
                assert_eq!(&bytes, previous);
            }
            previous = Some(bytes);
        }
    }
    assert!(review_failures.is_empty(), "{}", review_failures.join("\n"));
}
