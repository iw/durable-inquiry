//! What the collector says to someone who still has a name from before the
//! rename: a variable it no longer reads, or the state directory it kept its
//! redaction key in. It says so in one line and uses neither.

use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

const QUERY_OUTPUT: &str = r#"{"status":"success","data":{"resultType":"matrix","result":[{"metric":{"namespace":"orders"},"values":[[1790775300,"1"],[1790775315,"2"]]}]}}"#;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "durable-inquiry-collect-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create");
    dir
}

/// The collector, with nothing of the environment it is tested in.
fn collector(home: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_durable-inquiry-collect"));
    command
        .env_clear()
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"));
    command
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn a_variable_under_its_earlier_name_is_named_and_not_read() {
    let home = scratch("variables");
    let output = collector(&home)
        .args(["--dry-run", "--query", "up"])
        .env("KYUMEI_COLLECT_URL", "http://earlier.invalid:9090")
        .env("KYUMEI_COLLECT_TOKEN", "s3cret-value")
        .env("DURABLE_INQUIRY_COLLECT_TENANT", "tenant")
        .env("KYUMEI_COLLECT_TENANT", "earlier-tenant")
        .output()
        .expect("run");
    let said = stderr(&output);
    assert!(output.status.success(), "{said}");
    let hints: Vec<&str> = said.lines().filter(|l| l.contains("KYUMEI_")).collect();
    assert_eq!(
        hints,
        [
            "durable-inquiry-collect: $KYUMEI_COLLECT_URL is set but is not read: the variable is now $DURABLE_INQUIRY_COLLECT_URL.",
            "durable-inquiry-collect: $KYUMEI_COLLECT_TOKEN is set but is not read: the variable is now $DURABLE_INQUIRY_COLLECT_TOKEN.",
        ],
        "{said}"
    );
    // A value is never repeated: one of them is a credential.
    assert!(!said.contains("s3cret-value"), "{said}");
    assert!(!said.contains("earlier.invalid"), "{said}");

    // The earlier variable is not read: without --url there is still no endpoint.
    let output = collector(&home)
        .args(["--query", "up"])
        .env("KYUMEI_COLLECT_URL", "http://earlier.invalid:9090")
        .output()
        .expect("run");
    let said = stderr(&output);
    assert!(!output.status.success(), "{said}");
    assert!(said.contains("say where to read"), "{said}");
    assert!(said.contains("$DURABLE_INQUIRY_COLLECT_URL."), "{said}");
    let _ = std::fs::remove_dir_all(home);
}

#[test]
fn a_state_directory_left_behind_is_named_and_not_used() {
    let home = scratch("state");
    let config = home.join(".config");
    let earlier = config.join("kyumei-collect");
    std::fs::create_dir_all(&earlier).expect("create");
    let key = "ab".repeat(32);
    std::fs::write(earlier.join("redaction.key"), &key).expect("write");
    let file = home.join("output.json");
    std::fs::write(&file, QUERY_OUTPUT).expect("write");
    let encode = |extra: &[&str]| {
        collector(&home)
            .arg("encode")
            .arg(&file)
            .args(["--query", "up", "--redact", "namespace"])
            .args(extra)
            .output()
            .expect("run")
    };

    let output = encode(&[]);
    let said = stderr(&output);
    assert!(output.status.success(), "{said}");
    let now = config.join("durable-inquiry-collect");
    let hints: Vec<&str> = said.lines().filter(|l| l.contains("is not read")).collect();
    assert_eq!(hints.len(), 1, "{said}");
    assert!(
        hints[0].starts_with(&format!(
            "durable-inquiry-collect: The state directory {} is not read: its place is now {}.",
            earlier.display(),
            now.display()
        )),
        "{said}"
    );
    assert!(
        hints[0].contains(&format!("--state-dir {}", earlier.display())),
        "{said}"
    );
    // The earlier directory is as it was, and the new one has a key of its own.
    assert_eq!(
        std::fs::read_to_string(earlier.join("redaction.key")).expect("the earlier key"),
        key
    );
    assert!(!earlier.join("pseudonyms.tsv").exists());
    assert_ne!(
        std::fs::read_to_string(now.join("redaction.key"))
            .expect("the new key")
            .trim(),
        key
    );

    // Once the new directory exists, the earlier one is not mentioned again.
    let said = stderr(&encode(&[]));
    assert!(!said.contains("is not read"), "{said}");
    // Named with --state-dir, the earlier directory works as it did, unremarked.
    let output = encode(&["--state-dir", earlier.to_str().expect("a UTF-8 path")]);
    let said = stderr(&output);
    assert!(output.status.success(), "{said}");
    assert!(!said.contains("is not read"), "{said}");
    assert!(earlier.join("pseudonyms.tsv").exists());
    let _ = std::fs::remove_dir_all(home);
}
