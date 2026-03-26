use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use tempfile::NamedTempFile;
use things_cli::app::Cli;
use things_cli::cmd_ctx::CmdCtx;
use things_cli::commands::{Command, Commands};
use things_cli::wire::WireObject;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitExpectation {
    pub changes: BTreeMap<String, WireObject>,
    pub ancestor_index: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MutationFixture {
    pub test_name: String,
    pub cli_args: String,
    pub today_ts: Option<i64>,
    pub now_ts: Option<f64>,
    pub id_sequence: Option<Vec<String>>,
    pub journal: Vec<Value>,
    pub expected_output: Option<String>,
    pub expected_commits: Vec<CommitExpectation>,
}

#[derive(Debug, Clone)]
struct RecordedCommit {
    changes: BTreeMap<String, WireObject>,
    ancestor_index: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct MutationRun {
    pub stdout: String,
    pub commits: Vec<CommitExpectation>,
}

struct FakeCmdCtx {
    now_ts: f64,
    today_ts: i64,
    ids: VecDeque<String>,
    commits: Vec<RecordedCommit>,
    head_index: i64,
}

impl CmdCtx for FakeCmdCtx {
    fn now_timestamp(&self) -> f64 {
        self.now_ts
    }

    fn today_timestamp(&self) -> i64 {
        self.today_ts
    }

    fn next_id(&mut self) -> String {
        self.ids
            .pop_front()
            .unwrap_or_else(|| panic!("test fixture exhausted id_sequence"))
    }

    fn commit_changes(
        &mut self,
        changes: BTreeMap<String, WireObject>,
        ancestor_index: Option<i64>,
    ) -> anyhow::Result<i64> {
        self.commits.push(RecordedCommit {
            changes,
            ancestor_index,
        });
        self.head_index += 1;
        Ok(self.head_index)
    }

    fn current_head_index(&self) -> i64 {
        self.head_index
    }
}

fn parse_cli(cli_args: &str, journal_path: &str) -> Cli {
    let mut argv: Vec<String> = vec!["things3".to_owned()];
    argv.push("--no-color".to_owned());
    argv.push("--load-journal".to_owned());
    argv.push(journal_path.to_owned());
    for token in split_shell_words(cli_args) {
        argv.push(token);
    }
    Cli::try_parse_from(argv).unwrap_or_else(|e| panic!("Failed to parse args '{cli_args}': {e}"))
}

fn split_shell_words(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' if !in_single => {
                if let Some(next) = chars.next() {
                    cur.push(next);
                }
            }
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            c if c.is_whitespace() && !in_single && !in_double => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(ch),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn fixture_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("mutations")
}

pub fn load_fixture(name: &str) -> MutationFixture {
    let path = fixture_dir().join(format!("{name}.json"));
    let raw =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("Cannot read {path:?}: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("Cannot parse {path:?}: {e}"))
}

pub fn run_fixture(fixture: &MutationFixture) -> MutationRun {
    let mut tmp = NamedTempFile::new().expect("create temp file");
    serde_json::to_writer(&mut tmp, &fixture.journal).expect("write journal");
    let path = tmp.path().to_str().expect("valid path").to_owned();

    let cli = parse_cli(&fixture.cli_args, &path);

    let mut ctx = FakeCmdCtx {
        now_ts: fixture.now_ts.unwrap_or(0.0),
        today_ts: fixture.today_ts.unwrap_or(0),
        ids: fixture.id_sequence.clone().unwrap_or_default().into(),
        commits: Vec::new(),
        head_index: 0,
    };

    let mut stdout = Vec::new();
    let default_cmd = Commands::Today(Default::default());
    let cmd = cli.command.as_ref().unwrap_or(&default_cmd);
    cmd.run_with_ctx(&cli, &mut stdout, &mut ctx)
        .unwrap_or_else(|e| panic!("Command failed for {}: {e}", fixture.test_name));

    let commits = ctx
        .commits
        .iter()
        .map(|c| CommitExpectation {
            changes: c.changes.clone(),
            ancestor_index: c.ancestor_index,
        })
        .collect::<Vec<_>>();

    MutationRun {
        stdout: String::from_utf8(stdout).expect("stdout utf8"),
        commits,
    }
}

pub fn assert_fixture(name: &str) {
    let fixture = load_fixture(name);
    let run = run_fixture(&fixture);
    if let Some(expected_output) = fixture.expected_output {
        assert_eq!(run.stdout, expected_output, "stdout mismatch for {name}");
    }
    assert_eq!(
        serde_json::to_value(run.commits).expect("run commits to value"),
        serde_json::to_value(fixture.expected_commits).expect("expected commits to value"),
        "commit payload mismatch for {name}"
    );
}
