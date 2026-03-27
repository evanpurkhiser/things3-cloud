fn main() {
    if let Err(err) = things_cli::app::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    fn things3_bin_path() -> std::path::PathBuf {
        if let Some(path) = std::env::var_os("CARGO_BIN_EXE_things-cli") {
            return std::path::PathBuf::from(path);
        }

        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("debug")
            .join(format!("things-cli{}", std::env::consts::EXE_SUFFIX))
    }

    #[test]
    fn cli_trycmd() {
        let things3_bin = things3_bin_path();

        trycmd::TestCases::new()
            .env("TRYCMD_BIN_THINGS3", things3_bin.display().to_string())
            .register_bin("things3", &things3_bin)
            .register_bin(
                "run.sh",
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("trycmd")
                    .join("run.sh"),
            )
            .case("trycmd/**/*.trycmd");
    }
}
