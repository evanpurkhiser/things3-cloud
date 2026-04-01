use crate::app::Cli;
use crate::auth::write_auth;
use crate::commands::Command;
use anyhow::Result;
use clap::Args;
use std::io::{self, Write};

#[derive(Debug, Default, Args)]
#[command(about = "Configure Things Cloud credentials")]
pub struct SetAuthArgs {}

impl Command for SetAuthArgs {
    fn run_with_ctx(
        &self,
        _cli: &Cli,
        out: &mut dyn std::io::Write,
        _ctx: &mut dyn crate::cmd_ctx::CmdCtx,
    ) -> Result<()> {
        print!("Things Cloud email: ");
        io::stdout().flush()?;
        let mut email = String::new();
        io::stdin().read_line(&mut email)?;

        let password = rpassword::prompt_password("Things Cloud password: ")?;

        let path = write_auth(email.trim(), password.trim_end())?;
        writeln!(out, "Saved auth to {}", path.display())?;
        Ok(())
    }
}
