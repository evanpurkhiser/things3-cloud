use std::{io::Write, sync::Arc};

use anyhow::Result;
use clap::Args;
use iocraft::prelude::*;

use crate::{
    app::Cli,
    commands::{Command, DetailedArgs},
    ui::{render_element_to_string, views::anytime::AnytimeView},
};

#[derive(Debug, Default, Args)]
pub struct AnytimeArgs {
    #[command(flatten)]
    pub detailed: DetailedArgs,
}

impl Command for AnytimeArgs {
    fn run_with_ctx(
        &self,
        cli: &Cli,
        out: &mut dyn Write,
        ctx: &mut dyn crate::cmd_ctx::CmdCtx,
    ) -> Result<()> {
        let store = Arc::new(cli.load_store()?);
        let today = ctx.today();
        let tasks = store.anytime(&today);

        let mut ui = element! {
            ContextProvider(value: Context::owned(store.clone())) {
                ContextProvider(value: Context::owned(today)) {
                    AnytimeView(
                        items: &tasks,
                        detailed: self.detailed.detailed,
                    )
                }
            }
        };

        let rendered = render_element_to_string(&mut ui, cli.no_color);
        writeln!(out, "{}", rendered)?;
        Ok(())
    }
}
