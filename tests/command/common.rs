use chrono::Utc;
use clap::Parser;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use tempfile::NamedTempFile;
use things_cli::app::Cli;
use things_cli::cmd_ctx::CmdCtx;
use things_cli::commands::{Command, Commands};
use things_cli::wire::wire_object::WireObject;

#[derive(Deserialize)]
struct Fixture {
    test_name: String,
    cli_args: String,
    today_ts: Option<i64>,
    journal: Vec<Value>,
    expected_output: String,
}

struct FakeCmdCtx {
    today_ts: i64,
}

impl CmdCtx for FakeCmdCtx {
    fn now_timestamp(&self) -> f64 {
        0.0
    }

    fn today_timestamp(&self) -> i64 {
        self.today_ts
    }

    fn next_id(&mut self) -> String {
        panic!("read command should not call next_id")
    }

    fn commit_changes(
        &mut self,
        _changes: BTreeMap<String, WireObject>,
        _ancestor_index: Option<i64>,
    ) -> anyhow::Result<i64> {
        panic!("read command should not call commit_changes")
    }

    fn current_head_index(&self) -> i64 {
        0
    }
}

fn parse_cli(cli_args: &str, journal_path: &str) -> Cli {
    let mut argv: Vec<String> = vec!["things3".to_owned()];
    argv.push("--no-color".to_owned());
    argv.push("--load-journal".to_owned());
    argv.push(journal_path.to_owned());
    for token in cli_args.split_whitespace() {
        let t = token.trim_matches('\'').trim_matches('"').to_owned();
        argv.push(t);
    }

    Cli::try_parse_from(argv).unwrap_or_else(|e| panic!("Failed to parse args '{cli_args}': {e}"))
}

fn run_fixture(fixture: &Fixture) -> String {
    let mut tmp = NamedTempFile::new().expect("create temp file");
    serde_json::to_writer(&mut tmp, &fixture.journal).expect("write journal");
    let path = tmp.path().to_str().expect("valid path").to_owned();

    let cli = parse_cli(&fixture.cli_args, &path);

    let mut ctx = FakeCmdCtx {
        today_ts: fixture.today_ts.unwrap_or_else(|| Utc::now().timestamp()),
    };

    let mut buf: Vec<u8> = Vec::new();
    let default_cmd = Commands::Today(Default::default());
    let command = cli.command.as_ref().unwrap_or(&default_cmd);
    let result = command.run_with_ctx(&cli, &mut buf, &mut ctx);

    if let Err(e) = result {
        panic!("Command failed for {}: {e}", fixture.test_name);
    }

    String::from_utf8(buf).expect("output is valid UTF-8")
}

fn fixture_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("rust")
}

fn load_fixture(name: &str) -> Fixture {
    let path = fixture_dir().join(format!("{name}.json"));
    let raw =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("Cannot read {path:?}: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("Cannot parse {path:?}: {e}"))
}

pub fn run_named_fixture(name: &str) {
    let fixture = load_fixture(name);
    let got = run_fixture(&fixture);
    assert_eq!(
        got, fixture.expected_output,
        "fixture failed: {}\ncli_args: {}\n",
        fixture.test_name, fixture.cli_args
    );
}

macro_rules! fixture_test {
    ($fixture_name:ident) => {
        #[test]
        #[allow(non_snake_case)]
        fn $fixture_name() {
            crate::command::common::run_named_fixture(stringify!($fixture_name));
        }
    };
}

pub(crate) use fixture_test;
