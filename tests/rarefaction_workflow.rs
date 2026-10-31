use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const PROFILE: &str = "skbio-0.7.3/biom-2.1.17/numpy-2.5.4-pcg64-choice-shuffle-false";

fn rarefy(input: &Path, output: &Path, depth: u64, seed: &str, workers: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rsomics-ecology"))
        .args(["--json", "--threads", workers, "rarefy", "--counts"])
        .arg(input)
        .arg("--output")
        .arg(output)
        .args(["--depth", &depth.to_string(), "--seed", seed])
        .output()
        .unwrap()
}

fn success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn table(path: &Path) -> Vec<Vec<String>> {
    let text = fs::read_to_string(path).unwrap();
    let mut rows: Vec<Vec<_>> = text
        .lines()
        .map(|line| line.split('\t').map(str::to_owned).collect())
        .collect();
    rows[0][0] = "feature".into();
    rows
}

#[test]
fn captured_sampling_profiles_replay_with_workers_and_feed_alpha() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/rarefaction");
    let directory = tempfile::tempdir().unwrap();
    for (name, depth, seed) in [
        ("primary", 8, "42"),
        ("small", 200, "42"),
        ("big", 29648, "123"),
    ] {
        let input = fixture.join(format!("{name}-in.tsv"));
        let expected = table(&fixture.join(format!("{name}-expected.tsv")));
        let original = table(&input);
        let output = directory.path().join(format!("{name}.tsv"));
        let mut first = None;
        for workers in ["1", "4"] {
            let mut result =
                success(rarefy(&input, &output, depth, seed, workers))["result"].clone();
            assert_eq!(result["workers"], workers.parse::<u64>().unwrap());
            assert_eq!(result["rarefaction"]["depth"], depth);
            assert_eq!(result["rarefaction"]["seed"], seed);
            assert_eq!(result["rarefaction"]["profile"], PROFILE);
            result["workers"] = serde_json::json!(1);
            assert_eq!(table(&output), expected);
            if let Some(previous) = &first {
                assert_eq!(&result, previous);
            } else {
                first = Some(result);
            }
            for sample in 1..expected[0].len() {
                let mut sum = 0;
                for (actual, input) in expected.iter().skip(1).zip(original.iter().skip(1)) {
                    let actual = actual[sample].parse::<u64>().unwrap();
                    assert!(actual <= input[sample].trim().parse::<u64>().unwrap());
                    sum += actual;
                }
                assert_eq!(sum, depth);
            }
        }
    }
    let input = directory.path().join("primary.tsv");
    let output = directory.path().join("alpha.json");
    let result = Command::new(env!("CARGO_BIN_EXE_rsomics-ecology"))
        .args(["--json", "alpha", "--counts"])
        .arg(&input)
        .arg("--tree")
        .arg(fixture.join("primary-tree.nwk"))
        .arg("--output")
        .arg(&output)
        .args(["--metrics", "observed_features,shannon,simpson,dominance,inv_simpson,simpson_e,pielou_e,chao1,goods_coverage,faith_pd,phydiv", "--format", "json"])
        .output().unwrap();
    success(result);
    let alpha: Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
    let primary = table(&input);
    assert_eq!(alpha["sample_ids"], serde_json::json!(primary[0][1..]));
    assert_eq!(
        alpha["feature_ids"],
        serde_json::json!(
            primary
                .iter()
                .skip(1)
                .map(|row| row[0].clone())
                .collect::<Vec<_>>()
        )
    );
    let max = directory.path().join("max.tsv");
    fs::write(
        &max,
        "feature\tlarge\npresent\t9007199254740991\nabsent\t0\n",
    )
    .unwrap();
    for workers in ["1", "4"] {
        success(rarefy(&max, &output, 1, "42", workers));
        assert_eq!(
            fs::read_to_string(&output).unwrap(),
            "feature\tlarge\npresent\t1\nabsent\t0\n"
        );
    }
}

#[test]
fn zero_depth_and_failures_preserve_dimensions_and_atomic_outputs() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("counts.tsv");
    let output = directory.path().join("output.tsv");
    for (text, depth, expected) in [
        ("feature\ts1\ts2\n", 0, "feature\ts1\ts2\n"),
        (
            "feature\ts1\ts2\nA\t0\t0\nB\t0\t0\n",
            0,
            "feature\ts1\ts2\nA\t0\t0\nB\t0\t0\n",
        ),
        (
            "feature\ts\nA\t0\nB\t3\nC\t0\nD\t1\n",
            0,
            "feature\ts\nA\t0\nB\t0\nC\t0\nD\t0\n",
        ),
        (
            "feature\ts\nA\t0\nB\t3\nC\t0\nD\t1\n",
            4,
            "feature\ts\nA\t0\nB\t3\nC\t0\nD\t1\n",
        ),
    ] {
        fs::write(&input, text).unwrap();
        for workers in ["1", "4"] {
            success(rarefy(&input, &output, depth, "42", workers));
            assert_eq!(fs::read_to_string(&output).unwrap(), expected);
        }
    }
    for (text, depth) in [
        ("feature\ts1\ts2\nA\t10\t2\n", 3),
        ("feature\ts\nA\t0\n", 1),
        ("feature\ts\nA\t1.5\n", 0),
    ] {
        fs::write(&input, text).unwrap();
        fs::write(&output, "keep\n").unwrap();
        let result = rarefy(&input, &output, depth, "42", "4");
        assert!(!result.status.success());
        assert_eq!(
            serde_json::from_slice::<Value>(&result.stderr).unwrap()["status"],
            "error"
        );
        assert_eq!(fs::read_to_string(&output).unwrap(), "keep\n");
    }
    fs::write(&input, "feature\ts\nA\t2\n").unwrap();
    let before = fs::read(&input).unwrap();
    assert!(!rarefy(&input, &input, 1, "42", "1").status.success());
    assert_eq!(fs::read(&input).unwrap(), before);
    let result = Command::new(env!("CARGO_BIN_EXE_rsomics-ecology"))
        .args(["rarefy", "--counts"])
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .args(["--depth", "1"])
        .output()
        .unwrap();
    assert!(!result.status.success());
}
