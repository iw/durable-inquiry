//! What Durable Inquiry tells someone who still has a name it had as Kyumei.
//!
//! Nothing set under an earlier name is read: an environment variable that is
//! quietly ignored, or a directory that is quietly left behind, would look
//! like lost credentials or lost investigations. So each of them gets one line
//! saying what the name is now. Nothing is ever moved or rewritten here.

use std::path::Path;

/// The prefix of every environment variable Durable Inquiry reads.
pub const ENV_PREFIX: &str = "DURABLE_INQUIRY_";
/// The prefix those variables had before.
const EARLIER_ENV_PREFIX: &str = "KYUMEI_";
/// The directory, under the platform's data and configuration directories,
/// that the application used before.
pub const EARLIER_APP_DIR: &str = "kyumei";
/// The collector's state directory before, under the configuration directory.
pub const EARLIER_COLLECT_DIR: &str = "kyumei-collect";

/// The name `variable` had before, when it has the prefix.
fn earlier_name(variable: &str) -> Option<String> {
    variable
        .strip_prefix(ENV_PREFIX)
        .map(|rest| format!("{EARLIER_ENV_PREFIX}{rest}"))
}

/// One line for each of `variables` that is unset while the name it had
/// before is set. A line names the two variables and never a value, since
/// some of them hold credentials.
pub fn env_hints(variables: &[&str], is_set: impl Fn(&str) -> bool) -> Vec<String> {
    variables
        .iter()
        .filter(|variable| !is_set(variable))
        .filter_map(|variable| {
            let earlier = earlier_name(variable).filter(|e| is_set(e))?;
            Some(format!(
                "${earlier} is set but is not read: the variable is now ${variable}."
            ))
        })
        .collect()
}

/// One line when nothing is at `now` and `earlier` exists: `what` (a directory
/// or a file) is where this version does not look. `after` says what to do
/// about it, given the two paths.
pub fn left_behind(
    what: &str,
    earlier: &Path,
    now: &Path,
    after: impl Fn(&str, &str) -> String,
) -> Option<String> {
    if now.exists() || !earlier.exists() {
        return None;
    }
    let (earlier, now) = (quoted(earlier), quoted(now));
    Some(format!(
        "{what} {earlier} is not read: its place is now {now}. {}",
        after(&earlier, &now)
    ))
}

/// A path as a shell takes it: in quotes when it holds a space or anything
/// else a shell would not read literally, since the line is there to be
/// copied and the platform's own directories can have spaces in their names.
fn quoted(path: &Path) -> String {
    let text = path.display().to_string();
    let plain = |c: char| {
        c.is_ascii_alphanumeric() || "/._-+:@%~".contains(c) || (cfg!(windows) && c == '\\')
    };
    if !text.is_empty() && text.chars().all(plain) {
        text
    } else {
        format!("'{}'", text.replace('\'', r"'\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "DURABLE_INQUIRY_COLLECT_TOKEN";
    const URL: &str = "DURABLE_INQUIRY_COLLECT_URL";

    #[test]
    fn a_variable_set_under_its_earlier_name_is_named_and_its_value_is_not() {
        let set = [("KYUMEI_COLLECT_TOKEN", "s3cret-value")];
        let is_set = |name: &str| set.iter().any(|(n, _)| *n == name);
        let hints = env_hints(&[TOKEN, URL], is_set);
        assert_eq!(
            hints,
            [
                "$KYUMEI_COLLECT_TOKEN is set but is not read: the variable is now $DURABLE_INQUIRY_COLLECT_TOKEN."
            ]
        );
        assert!(hints.iter().all(|h| !h.contains("s3cret-value")));
    }

    #[test]
    fn a_variable_set_under_its_new_name_needs_no_hint() {
        let both = |name: &str| name == TOKEN || name == "KYUMEI_COLLECT_TOKEN";
        assert!(env_hints(&[TOKEN, URL], both).is_empty());
        assert!(env_hints(&[TOKEN, URL], |_| false).is_empty());
        // A variable without the prefix never had an earlier name.
        assert!(env_hints(&["GITHUB_TOKEN"], |name| name != "GITHUB_TOKEN").is_empty());
    }

    #[test]
    fn a_directory_left_behind_is_named_once_and_only_while_the_new_one_is_missing() {
        let root = std::env::temp_dir().join(format!(
            "durable-inquiry-collect-earlier-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let (earlier, now) = (root.join(EARLIER_COLLECT_DIR), root.join("now"));
        let after = |earlier: &str, now: &str| format!("Move {earlier} to {now}.");
        // Neither exists: nothing was left behind.
        assert_eq!(left_behind("The directory", &earlier, &now, after), None);
        std::fs::create_dir_all(&earlier).expect("create");
        std::fs::write(earlier.join("redaction.key"), "k").expect("write");
        let hint = left_behind("The directory", &earlier, &now, after).expect("a hint");
        assert_eq!(hint.lines().count(), 1, "{hint}");
        assert!(hint.contains(&quoted(&earlier)), "{hint}");
        assert!(hint.contains(&quoted(&now)), "{hint}");
        // Saying so changes nothing on disk.
        assert!(earlier.join("redaction.key").exists());
        assert!(!now.exists());
        // Once the new place exists, the earlier one is someone's to keep or delete.
        std::fs::create_dir_all(&now).expect("create");
        assert_eq!(left_behind("The directory", &earlier, &now, after), None);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_path_with_a_space_is_quoted_for_the_shell() {
        assert_eq!(
            quoted(Path::new("/home/op/.config/kyumei")),
            "/home/op/.config/kyumei"
        );
        assert_eq!(
            quoted(Path::new("/Users/op/Library/Application Support/kyumei")),
            "'/Users/op/Library/Application Support/kyumei'"
        );
        assert_eq!(quoted(Path::new("/tmp/it's")), r"'/tmp/it'\''s'");
    }
}
