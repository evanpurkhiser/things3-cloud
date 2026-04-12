fn main() {
    // Install an explicit rustls crypto provider at startup. We use `ring`
    // because it tends to be the most reliable/easiest provider to build
    // across common Linux packaging environments.
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");

    if let Err(err) = things3_cloud::app::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
