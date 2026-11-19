//! Durable Inquiry's dependency direction is part of its design, so it is tested.
//! (The diagrams leave out the `durable-inquiry-` that starts every crate's name.)
//!
//!   web (Topcoat)   odori (Odori, Temporal SDK)
//!        \              /
//!         ▼            ▼
//!              app
//!               │
//!     diagnostics ─ evidence ─ knowledge ─ diagrams
//!               │
//!            domain
//!
//! The domain knows nothing of frameworks; Topcoat appears only in the web
//! crate; Odori and the Temporal SDK only in the orchestration crate. The
//! knowledge extractor (`durable-inquiry-extract`) stands apart: it writes knowledge files
//! and nothing depends on it. The demo's program (`durable-inquiry-demo`) stands apart too:
//! it is the Temporal application the demo investigates, built on the Temporal SDK,
//! and it and Durable Inquiry's crates never use each other.
//!
//! The companion collector stands apart too. `durable-inquiry-samples` is the sample
//! block format alone, which the evidence interpreter reads; `durable-inquiry-collect`
//! is the companion's HTTP client and binary, built on the format and nothing
//! else of Durable Inquiry's, which the web crate also uses for Collect now:
//!
//!   web ──▶ collect ──▶ samples ◀── evidence
//!
//! `durable-inquiry-dashboards` reads Grafana dashboards. Like the diagrams, it knows
//! nothing of Temporal or of the rest of Durable Inquiry; the application layer pairs
//! its panels with the knowledge.
//!
//! `durable-inquiry-sim` runs Durable Envision, the cluster simulator, and answers PromQL over
//! its scrapes. It is the only crate that knows Durable Envision, and of Durable Inquiry it knows
//! only the sample block format, so simulated evidence can travel as collected
//! evidence does. The application layer uses it for the "big enough?" check,
//! and nothing else does. The extractor reads Temporal's dynamic configuration
//! with Durable Envision's parser, which the two share rather than copy:
//!
//!   app ──▶ sim ──▶ durable-envision ◀── extract

use std::{collections::BTreeMap, process::Command};

fn dependencies() -> BTreeMap<String, Vec<String>> {
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo metadata");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("metadata json");
    json["packages"]
        .as_array()
        .expect("packages")
        .iter()
        .map(|p| {
            let name = p["name"].as_str().unwrap_or_default().to_owned();
            let deps = p["dependencies"]
                .as_array()
                .expect("dependencies")
                .iter()
                .filter(|d| d["kind"].is_null())
                .map(|d| d["name"].as_str().unwrap_or_default().to_owned())
                .collect();
            (name, deps)
        })
        .collect()
}

fn forbid(deps: &BTreeMap<String, Vec<String>>, krate: &str, patterns: &[&str]) {
    let own = deps
        .get(krate)
        .unwrap_or_else(|| panic!("{krate} is a workspace member"));
    for d in own {
        for p in patterns {
            assert!(
                !(d == p || (p.ends_with('*') && d.starts_with(p.trim_end_matches('*')))),
                "{krate} must not depend on {d}"
            );
        }
    }
}

#[test]
fn dependency_direction_is_preserved() {
    let deps = dependencies();
    let frameworks = ["topcoat", "odori", "temporalio-*", "tokeira-*", "tokio"];
    forbid(&deps, "durable-inquiry-domain", &["durable-inquiry-*"]);
    forbid(&deps, "durable-inquiry-domain", &frameworks);
    for core in [
        "durable-inquiry-knowledge",
        "durable-inquiry-evidence",
        "durable-inquiry-diagrams",
        "durable-inquiry-diagnostics",
    ] {
        forbid(&deps, core, &frameworks);
        forbid(
            &deps,
            core,
            &[
                "durable-inquiry-app",
                "durable-inquiry-web",
                "durable-inquiry-odori",
            ],
        );
    }
    forbid(
        &deps,
        "durable-inquiry-diagrams",
        &[
            "durable-inquiry-knowledge",
            "durable-inquiry-diagnostics",
            "durable-inquiry-evidence",
        ],
    );
    forbid(
        &deps,
        "durable-inquiry-app",
        &[
            "topcoat",
            "odori",
            "temporalio-*",
            "tokeira-*",
            "durable-inquiry-web",
            "durable-inquiry-odori",
        ],
    );
    forbid(
        &deps,
        "durable-inquiry-web",
        &[
            "odori",
            "temporalio-*",
            "tokeira-*",
            "durable-inquiry-odori",
        ],
    );
    forbid(
        &deps,
        "durable-inquiry-odori",
        &["topcoat", "durable-inquiry-web"],
    );
    // The sample block format is pure: no frameworks and none of Durable Inquiry, so
    // the companion carries no model, agents or knowledge.
    forbid(&deps, "durable-inquiry-samples", &frameworks);
    forbid(&deps, "durable-inquiry-samples", &["durable-inquiry-*"]);
    forbid(
        &deps,
        "durable-inquiry-collect",
        &["topcoat", "odori", "temporalio-*", "tokeira-*"],
    );
    let collect = &deps["durable-inquiry-collect"];
    assert!(
        collect
            .iter()
            .all(|d| !d.starts_with("durable-inquiry-") || d == "durable-inquiry-samples"),
        "durable-inquiry-collect depends on no Durable Inquiry crate but durable-inquiry-samples: {collect:?}"
    );
    forbid(&deps, "durable-inquiry-dashboards", &frameworks);
    forbid(&deps, "durable-inquiry-dashboards", &["durable-inquiry-*"]);
    for core in [
        "durable-inquiry-domain",
        "durable-inquiry-knowledge",
        "durable-inquiry-evidence",
        "durable-inquiry-diagnostics",
        "durable-inquiry-diagrams",
        "durable-inquiry-samples",
    ] {
        forbid(&deps, core, &["durable-inquiry-dashboards"]);
    }
    // Only the interfaces read metrics over the network.
    for (krate, own) in &deps {
        if !["durable-inquiry", "durable-inquiry-web"].contains(&krate.as_str()) {
            assert!(
                !own.iter().any(|d| d == "durable-inquiry-collect"),
                "{krate} must not depend on durable-inquiry-collect"
            );
        }
    }
    // The extractor is a maintainer tool: it reads Temporal source and writes knowledge
    // files, and the rest of Durable Inquiry never depends on it.
    forbid(&deps, "durable-inquiry-extract", &frameworks);
    forbid(&deps, "durable-inquiry-extract", &["durable-inquiry-*"]);
    for (krate, own) in &deps {
        if krate != "durable-inquiry-extract" {
            assert!(
                !own.iter().any(|d| d == "durable-inquiry-extract"),
                "{krate} must not depend on durable-inquiry-extract"
            );
        }
    }
    // The demo's program is the application Durable Inquiry investigates in the demo, not
    // part of Durable Inquiry.
    forbid(&deps, "durable-inquiry-demo", &["durable-inquiry-*"]);
    for (krate, own) in &deps {
        assert!(
            !own.iter().any(|d| d == "durable-inquiry-demo"),
            "{krate} must not depend on durable-inquiry-demo"
        );
    }
    forbid(&deps, "durable-inquiry-sim", &frameworks);
    let sim = &deps["durable-inquiry-sim"];
    assert!(
        sim.iter()
            .all(|d| !d.starts_with("durable-inquiry-") || d == "durable-inquiry-samples"),
        "durable-inquiry-sim depends on no Durable Inquiry crate but durable-inquiry-samples: {sim:?}"
    );
    let sim_users: Vec<&String> = deps
        .iter()
        .filter(|(_, d)| d.iter().any(|x| x == "durable-inquiry-sim"))
        .map(|(k, _)| k)
        .collect();
    assert_eq!(
        sim_users,
        vec!["durable-inquiry-app"],
        "only the application layer runs simulations"
    );
    // `cargo metadata` names a dependency by its package, and Durable Envision's
    // is `tempdes` at the pinned commit.
    let durable_envision_users: Vec<&String> = deps
        .iter()
        .filter(|(_, d)| d.iter().any(|x| x == "tempdes"))
        .map(|(k, _)| k)
        .collect();
    assert_eq!(
        durable_envision_users,
        vec!["durable-inquiry-extract", "durable-inquiry-sim"],
        "Durable Envision is confined to durable-inquiry-sim, and the extractor's configuration parser"
    );
    let topcoat_users: Vec<&String> = deps
        .iter()
        .filter(|(_, d)| d.iter().any(|x| x == "topcoat"))
        .map(|(k, _)| k)
        .collect();
    assert_eq!(
        topcoat_users,
        vec!["durable-inquiry-web"],
        "Topcoat is confined to the web crate"
    );
}
