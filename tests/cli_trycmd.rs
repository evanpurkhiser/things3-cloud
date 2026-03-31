fn run_trycmd_cases(case_glob: &str) {
    let things3_bin = std::env::var_os("CARGO_BIN_EXE_things3")
        .map(std::path::PathBuf::from)
        .expect("CARGO_BIN_EXE_things3 is not set; run this as an integration test via cargo test");

    trycmd::TestCases::new()
        .env("TRYCMD_BIN_THINGS3", things3_bin.display().to_string())
        .register_bin("things3", &things3_bin)
        .register_bin(
            "run.sh",
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("trycmd")
                .join("run.sh"),
        )
        .case(case_glob);
}

#[test]
fn cli_trycmd_anytime() {
    run_trycmd_cases("trycmd/anytime/**/*.trycmd");
}

#[test]
fn cli_trycmd_area() {
    run_trycmd_cases("trycmd/area/**/*.trycmd");
}

#[test]
fn cli_trycmd_areas() {
    run_trycmd_cases("trycmd/areas/**/*.trycmd");
}

#[test]
fn cli_trycmd_delete() {
    run_trycmd_cases("trycmd/delete/**/*.trycmd");
}

#[test]
fn cli_trycmd_edit() {
    run_trycmd_cases("trycmd/edit/**/*.trycmd");
}

#[test]
fn cli_trycmd_find() {
    run_trycmd_cases("trycmd/find/**/*.trycmd");
}

#[test]
fn cli_trycmd_inbox() {
    run_trycmd_cases("trycmd/inbox/**/*.trycmd");
}

#[test]
fn cli_trycmd_logbook() {
    run_trycmd_cases("trycmd/logbook/**/*.trycmd");
}

#[test]
fn cli_trycmd_mark() {
    run_trycmd_cases("trycmd/mark/**/*.trycmd");
}

#[test]
fn cli_trycmd_new() {
    run_trycmd_cases("trycmd/new/**/*.trycmd");
}

#[test]
fn cli_trycmd_project() {
    run_trycmd_cases("trycmd/project/**/*.trycmd");
}

#[test]
fn cli_trycmd_projects() {
    run_trycmd_cases("trycmd/projects/**/*.trycmd");
}

#[test]
fn cli_trycmd_reorder() {
    run_trycmd_cases("trycmd/reorder/**/*.trycmd");
}

#[test]
fn cli_trycmd_schedule() {
    run_trycmd_cases("trycmd/schedule/**/*.trycmd");
}

#[test]
fn cli_trycmd_someday() {
    run_trycmd_cases("trycmd/someday/**/*.trycmd");
}

#[test]
fn cli_trycmd_tags() {
    run_trycmd_cases("trycmd/tags/**/*.trycmd");
}

#[test]
fn cli_trycmd_today() {
    run_trycmd_cases("trycmd/today/**/*.trycmd");
}

#[test]
fn cli_trycmd_upcoming() {
    run_trycmd_cases("trycmd/upcoming/**/*.trycmd");
}
