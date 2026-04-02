use crate::app::Cli;
use crate::commands::Command;
use anyhow::Result;
use clap::{Args, CommandFactory};
use clap_complete::{Shell, generate};

#[derive(Debug, Clone, Args)]
pub struct CompletionsArgs {
    #[arg(value_enum)]
    pub shell: Shell,
}

impl Command for CompletionsArgs {
    fn run_with_ctx(
        &self,
        _cli: &Cli,
        out: &mut dyn std::io::Write,
        _ctx: &mut dyn crate::cmd_ctx::CmdCtx,
    ) -> Result<()> {
        let mut cmd = Cli::command();
        let bin_name = cmd.get_name().to_string();
        generate(self.shell, &mut cmd, bin_name, out);
        Ok(())
    }
}
