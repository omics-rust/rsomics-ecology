use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const METRICS: &str = "observed_features,shannon,simpson,dominance,inv_simpson,simpson_e,pielou_e,chao1,goods_coverage,faith_pd,phydiv";

fn alpha(counts: &Path, tree: Option<&Path>, output: &Path, extra: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rsomics-ecology"));
    command.args(["alpha", "--counts"]).arg(counts);
    command.arg("--output").arg(output);
    if let Some(tree) = tree {
        command.arg("--tree").arg(tree);
    }
    command.args(extra).output().unwrap()
}

fn success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn multi_alpha_preserves_sparse_identities_root_weights_and_unavailable_values() {
    let directory = tempfile::tempdir().unwrap();
    let counts = directory.path().join("counts.tsv");
    let tree = directory.path().join("tree.nwk");
    let output = directory.path().join("alpha.json");
    fs::write(
        &counts,
        "#OTU ID\tmixed\tsingle\tempty\nA\t 1 \t0\t0\nB\t1\t4\t0\nabsent\t0\t0\t0\n",
    )
    .unwrap();
    fs::write(&tree, "((A:1,B:2):3,absent:4)root:0.5;\n").unwrap();
    let mut reports = Vec::new();
    for workers in ["1", "4"] {
        let envelope = success(alpha(
            &counts,
            Some(&tree),
            &output,
            &[
                "--metrics",
                METRICS,
                "--format",
                "json",
                "--threads",
                workers,
                "--json",
            ],
        ));
        assert_eq!(envelope["status"], "ok");
        assert_eq!(
            envelope["result"]["workers"],
            workers.parse::<u64>().unwrap()
        );
        let report: Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
        assert_eq!(
            report["feature_ids"],
            serde_json::json!(["A", "B", "absent"])
        );
        assert_eq!(
            report["sample_ids"],
            serde_json::json!(["mixed", "single", "empty"])
        );
        assert_eq!(
            report["metrics"],
            serde_json::json!(METRICS.split(',').collect::<Vec<_>>())
        );
        let values = &report["values"];
        let mixed = [
            2.0,
            std::f64::consts::LN_2,
            0.5,
            0.5,
            2.0,
            1.0,
            1.0,
            3.0,
            0.0,
            6.5,
            6.5,
        ];
        for (index, expected) in mixed.into_iter().enumerate() {
            assert!((values[0][index]["value"].as_f64().unwrap() - expected).abs() < 1e-12);
        }
        assert_eq!(values[1][6]["value"], 1.0);
        assert_eq!(values[1][9]["value"], 5.5);
        for index in [1, 2, 3, 4, 5, 6, 8] {
            assert!(values[2][index]["value"].is_null());
            assert_eq!(values[2][index]["reason"], "empty_community");
            assert_eq!(values[2][index]["nonfinite"], "nan");
        }
        for index in [0, 7, 9, 10] {
            assert_eq!(values[2][index]["value"], 0.0);
        }
        reports.push(report);
    }
    assert_eq!(reports[0], reports[1]);
    success(alpha(
        &counts,
        Some(&tree),
        &output,
        &[
            "--metrics",
            "phydiv,observed_features",
            "--pd-rooted",
            "unrooted",
            "--pd-weight",
            "0.5",
            "--format",
            "json",
            "--json",
        ],
    ));
    let report: Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
    assert_eq!(
        report["metrics"],
        serde_json::json!(["phydiv", "observed_features"])
    );
    assert_eq!(report["values"][0][0]["value"], 3.0);
    assert_eq!(report["values"][1][0]["value"], 0.0);
    success(alpha(&counts, None, &output, &["--json"]));
    let text = fs::read_to_string(&output).unwrap();
    assert!(text.starts_with("sample\tobserved_features\tshannon\t"));
    assert!(text.lines().last().unwrap().contains("\tnan\t"));
    fs::write(&counts, "feature\tempty\n").unwrap();
    success(alpha(
        &counts,
        Some(&tree),
        &output,
        &["--metrics", METRICS, "--format", "json", "--json"],
    ));
    let report: Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
    assert_eq!(report["feature_ids"], serde_json::json!([]));
    assert_eq!(report["values"][0][9]["value"], 0.0);
}

#[test]
fn invalid_inputs_and_aliases_leave_named_destinations_unchanged() {
    let directory = tempfile::tempdir().unwrap();
    let counts = directory.path().join("counts.tsv");
    let tree = directory.path().join("tree.nwk");
    let output = directory.path().join("alpha.tsv");
    fs::write(&tree, "(A:1,B:2)root;").unwrap();
    for bad in [
        "",
        "wrong\ts\nA\t1\n",
        "feature\ts\ts\nA\t1\t0\n",
        "feature\t\nA\t1\n",
        "feature\ts\n\t1\n",
        "feature\ts\n A\t1\n",
        "feature\ts\nA\t1\nA\t2\n",
        "feature\ts\nA\n",
        "feature\ts\nA\t1\t2\n",
        "feature\ts\nA\t\n",
        "feature\ts\nA\t-1\n",
        "feature\ts\nA\t+1\n",
        "feature\ts\nA\t1.0\n",
        "feature\ts\nA\tNaN\n",
        "feature\ts\nA\tinf\n",
        "feature\ts\nA\t9007199254740992\n",
        "feature\ts\nA\t9007199254740991\nB\t1\n",
    ] {
        fs::write(&counts, bad).unwrap();
        fs::write(&output, "keep\n").unwrap();
        let result = alpha(&counts, None, &output, &["--json"]);
        assert!(!result.status.success(), "accepted {bad:?}");
        assert!(result.stdout.is_empty());
        assert_eq!(
            serde_json::from_slice::<Value>(&result.stderr).unwrap()["status"],
            "error"
        );
        assert_eq!(fs::read_to_string(&output).unwrap(), "keep\n");
    }
    fs::write(&counts, "feature\ts\nA\t1\nB\t2\n").unwrap();
    for bad_tree in [
        "(A:1,B:2,C:3)r;",
        "(A:1,C:2)r;",
        "(A:1,A:2)r;",
        "(A,B:2)r;",
        "(A:-1,B:2)r;",
        "(A:NaN,B:2)r;",
    ] {
        fs::write(&tree, bad_tree).unwrap();
        assert!(
            !alpha(&counts, Some(&tree), &output, &["--metrics", "faith_pd"])
                .status
                .success(),
            "accepted {bad_tree}"
        );
        assert_eq!(fs::read_to_string(&output).unwrap(), "keep\n");
    }
    for extra in [
        vec!["--threads", "0"],
        vec!["--metrics", "shannon,shannon"],
        vec!["--metrics", "faith_pd"],
        vec!["--metrics", "phydiv", "--pd-weight", "NaN"],
        vec!["--pd-rooted", "rooted"],
    ] {
        assert!(!alpha(&counts, None, &output, &extra).status.success());
        assert_eq!(fs::read_to_string(&output).unwrap(), "keep\n");
    }
    let original = fs::read(&counts).unwrap();
    let hard_link = directory.path().join("alias.tsv");
    fs::hard_link(&counts, &hard_link).unwrap();
    for alias in [&counts, &hard_link] {
        assert!(!alpha(&counts, None, alias, &[]).status.success());
        assert_eq!(fs::read(&counts).unwrap(), original);
    }
    let help = Command::new(env!("CARGO_BIN_EXE_rsomics-ecology"))
        .args(["alpha", "--help"])
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("--pd-weight"));
}
