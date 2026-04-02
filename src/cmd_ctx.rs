use std::collections::BTreeMap;

use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};

use crate::{
    app::Cli,
    cloud_writer::{CloudWriter, DryRunCloudWriter, LiveCloudWriter, LoggingCloudWriter},
    ids::ThingsId,
    wire::wire_object::WireObject,
};

pub trait CmdCtx {
    fn now_timestamp(&self) -> f64;
    fn today_timestamp(&self) -> i64;
    fn today(&self) -> DateTime<Utc> {
        let ts = self.today_timestamp();
        Utc.timestamp_opt(ts, 0)
            .single()
            .unwrap_or_else(Utc::now)
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .map(|d| Utc.from_utc_datetime(&d))
            .unwrap_or_else(Utc::now)
    }
    fn next_id(&mut self) -> String;
    fn commit_changes(
        &mut self,
        changes: BTreeMap<String, WireObject>,
        ancestor_index: Option<i64>,
    ) -> Result<i64>;
    fn current_head_index(&self) -> i64;
}

#[derive(Default)]
pub struct DefaultCmdCtx {
    no_cloud: bool,
    today_ts_override: Option<i64>,
    now_ts_override: Option<f64>,
    writer: Option<Box<dyn CloudWriter>>,
}

impl DefaultCmdCtx {
    pub fn from_cli(cli: &Cli) -> Self {
        Self {
            no_cloud: cli.no_cloud,
            today_ts_override: cli.today_ts,
            now_ts_override: cli.now_ts,
            writer: None,
        }
    }

    fn writer_mut(&mut self) -> Result<&mut dyn CloudWriter> {
        if self.writer.is_none() {
            let inner: Box<dyn CloudWriter> = if self.no_cloud {
                Box::new(DryRunCloudWriter::new())
            } else {
                Box::new(LiveCloudWriter::new()?)
            };
            self.writer = Some(Box::new(LoggingCloudWriter::new(inner)));
        }
        Ok(self.writer.as_deref_mut().expect("writer initialized"))
    }
}

impl CmdCtx for DefaultCmdCtx {
    fn now_timestamp(&self) -> f64 {
        self.now_ts_override
            .unwrap_or_else(crate::common::now_ts_f64)
    }

    fn today_timestamp(&self) -> i64 {
        self.today_ts_override
            .unwrap_or_else(|| crate::common::today_utc().timestamp())
    }

    fn next_id(&mut self) -> String {
        ThingsId::random().to_string()
    }

    fn commit_changes(
        &mut self,
        changes: BTreeMap<String, WireObject>,
        ancestor_index: Option<i64>,
    ) -> Result<i64> {
        self.writer_mut()?.commit(changes, ancestor_index)
    }

    fn current_head_index(&self) -> i64 {
        self.writer.as_deref().map_or(0, CloudWriter::head_index)
    }
}
