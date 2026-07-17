//! Experimental Rust SDK surface for embedding Things Cloud workflows.
//!
//! The CLI remains the stable user interface. This module exposes the same
//! read and mutation behavior through typed Rust APIs for app integrations.

use std::{collections::BTreeMap, fs, io::Read, path::PathBuf};

use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    app::Cli,
    arg_types::IdentifierToken,
    auth::{load_auth, write_auth},
    client::ThingsCloudClient,
    cmd_ctx::{CmdCtx, DefaultCmdCtx},
    commands::{
        DetailedArgs, TagDeltaArgs,
        areas::{AreasEditArgs, AreasNewArgs, build_area_new_plan, build_areas_edit_plan},
        delete::{DeleteArgs, build_delete_plan},
        edit::{EditArgs, build_edit_plan},
        find::{FindArgs, find_tasks},
        mark::{MarkArgs, build_mark_checklist_plan, build_mark_status_plan},
        new::{NewArgs, build_new_plan},
        projects::{
            ProjectsEditArgs, ProjectsNewArgs, build_project_new_plan, build_projects_edit_plan,
        },
        reorder::{ReorderArgs, build_reorder_plan},
        schedule::{ScheduleArgs, build_schedule_plan},
        tags::{
            TagsDeleteArgs, TagsEditArgs, TagsNewArgs, build_tag_new_plan, build_tags_edit_plan,
        },
    },
    common::{parse_day, resolve_single_tag},
    dirs::append_log_dir,
    log_cache::{fold_state_from_append_log, get_state_with_append_log},
    logging,
    store::{RawState, Task, ThingsStore, fold_items},
    ui::views::json::common::{
        ResolvedAreaJson, ResolvedTagJson, ResolvedTaskJson, build_area_json, build_tags_json,
        build_tasks_json,
    },
    wire::{
        task::TaskStatus,
        wire_object::{EntityType, WireItem, WireObject},
    },
};

pub type SdkResult<T> = std::result::Result<T, ThingsSdkError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub enum ThingsSdkError {
    Auth(String),
    Sync(String),
    Validation(String),
    NotFound(String),
    CloudCommit(String),
    Io(String),
}

impl std::fmt::Display for ThingsSdkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auth(message)
            | Self::Sync(message)
            | Self::Validation(message)
            | Self::NotFound(message)
            | Self::CloudCommit(message)
            | Self::Io(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for ThingsSdkError {}

#[derive(Debug, Clone, Default)]
pub struct ThingsServiceConfig {
    pub cache_only: bool,
    pub dry_run: bool,
    pub journal_path: Option<PathBuf>,
    pub today_ts: Option<i64>,
    pub now_ts: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub configured: bool,
    pub email: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationResult {
    pub ids: Vec<String>,
    pub titles: Vec<String>,
    pub labels: Vec<String>,
    pub head_index: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogbookQuery {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AreaQuery {
    pub all: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FindQuery {
    pub query: Option<String>,
    pub notes: bool,
    pub checklists: bool,
    pub incomplete: bool,
    pub completed: bool,
    pub canceled: bool,
    pub any_status: bool,
    pub tags: Vec<String>,
    pub projects: Vec<String>,
    pub areas: Vec<String>,
    pub inbox: bool,
    pub today: bool,
    pub someday: bool,
    pub evening: bool,
    pub has_deadline: bool,
    pub no_deadline: bool,
    pub recurring: bool,
    pub deadline: Vec<String>,
    pub scheduled: Vec<String>,
    pub created: Vec<String>,
    pub completed_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub in_target: Option<String>,
    pub when: Option<String>,
    pub before_id: Option<String>,
    pub after_id: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<String>,
    pub deadline: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditTasksRequest {
    pub task_ids: Vec<String>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub move_target: Option<String>,
    pub add_tags: Option<String>,
    pub remove_tags: Option<String>,
    pub add_checklist: Vec<String>,
    pub remove_checklist: Option<String>,
    pub rename_checklist: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkStatus {
    Done,
    Incomplete,
    Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkTasksRequest {
    pub task_ids: Vec<String>,
    pub status: MarkStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChecklistStatus {
    Checked,
    Unchecked,
    Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutateChecklistRequest {
    pub task_id: String,
    pub checklist_ids: String,
    pub status: ChecklistStatus,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScheduleTaskRequest {
    pub task_id: String,
    pub when: Option<String>,
    pub deadline: Option<String>,
    pub clear_deadline: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReorderItemRequest {
    pub item_id: String,
    pub before_id: Option<String>,
    pub after_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteItemsRequest {
    pub item_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub title: String,
    pub area: Option<String>,
    pub when: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<String>,
    pub deadline: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditProjectRequest {
    pub project_id: String,
    pub title: Option<String>,
    pub move_target: Option<String>,
    pub notes: Option<String>,
    pub add_tags: Option<String>,
    pub remove_tags: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAreaRequest {
    pub title: String,
    pub tags: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditAreaRequest {
    pub area_id: String,
    pub title: Option<String>,
    pub add_tags: Option<String>,
    pub remove_tags: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditTagRequest {
    pub tag_id: String,
    pub name: Option<String>,
    pub move_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTagRequest {
    pub tag_id: String,
}

#[derive(Debug, Clone)]
pub struct ThingsService {
    config: ThingsServiceConfig,
}

impl ThingsService {
    pub fn new(config: ThingsServiceConfig) -> Self {
        Self { config }
    }

    pub fn auth_status(&self) -> AuthStatus {
        match load_auth() {
            Ok((email, _)) => AuthStatus {
                configured: true,
                email: Some(email),
                message: None,
            },
            Err(err) => AuthStatus {
                configured: false,
                email: None,
                message: Some(err.to_string()),
            },
        }
    }

    pub fn save_auth(&self, email: &str, password: &str) -> SdkResult<PathBuf> {
        write_auth(email, password).map_err(|err| ThingsSdkError::Auth(err.to_string()))
    }

    pub fn load_store(&self) -> SdkResult<ThingsStore> {
        let state = self.load_state()?;
        Ok(ThingsStore::from_raw_state(&state))
    }

    pub fn inbox(&self) -> SdkResult<Vec<ResolvedTaskJson>> {
        let store = self.load_store()?;
        Ok(self.tasks_json(&store.inbox(), &store))
    }

    pub fn today(&self) -> SdkResult<Vec<ResolvedTaskJson>> {
        let store = self.load_store()?;
        let today = self.current_day();
        let mut items: Vec<_> = store
            .tasks(Some(TaskStatus::Incomplete), Some(false), None)
            .into_iter()
            .filter(|t| {
                !t.is_heading()
                    && !t.title.trim().is_empty()
                    && t.entity == "Task6"
                    && (t.is_today(&today) || t.evening)
            })
            .collect();
        items.sort_by_key(|task| {
            let tir = task.today_index_reference.unwrap_or(0);
            (
                std::cmp::Reverse(tir),
                task.today_index,
                std::cmp::Reverse(task.index),
            )
        });
        Ok(self.tasks_json(&items, &store))
    }

    pub fn upcoming(&self) -> SdkResult<Vec<ResolvedTaskJson>> {
        let store = self.load_store()?;
        let today = self.current_day();
        let now_ts = today.timestamp();
        let mut tasks = Vec::new();
        for task in store.tasks(Some(TaskStatus::Incomplete), Some(false), None) {
            if task.in_someday() {
                continue;
            }
            let Some(start_date) = task.start_date else {
                continue;
            };
            if start_date.timestamp() > now_ts {
                tasks.push(task);
            }
        }
        tasks.sort_by_key(|task| task.start_date);
        Ok(self.tasks_json(&tasks, &store))
    }

    pub fn anytime(&self) -> SdkResult<Vec<ResolvedTaskJson>> {
        let store = self.load_store()?;
        Ok(self.tasks_json(&store.anytime(&self.current_day()), &store))
    }

    pub fn someday(&self) -> SdkResult<Vec<ResolvedTaskJson>> {
        let store = self.load_store()?;
        Ok(self.tasks_json(&store.someday(), &store))
    }

    pub fn logbook(&self, query: LogbookQuery) -> SdkResult<Vec<ResolvedTaskJson>> {
        let store = self.load_store()?;
        let from =
            parse_day(query.from.as_deref(), "--from").map_err(ThingsSdkError::Validation)?;
        let to = parse_day(query.to.as_deref(), "--to").map_err(ThingsSdkError::Validation)?;
        if let (Some(from), Some(to)) = (from, to)
            && from > to
        {
            return Err(ThingsSdkError::Validation(
                "--from date must be before or equal to --to date".to_string(),
            ));
        }
        Ok(self.tasks_json(&store.logbook(from, to), &store))
    }

    pub fn projects(&self) -> SdkResult<Vec<ResolvedTaskJson>> {
        let store = self.load_store()?;
        Ok(self.tasks_json(&store.projects(Some(TaskStatus::Incomplete)), &store))
    }

    pub fn project(&self, project_id: &str) -> SdkResult<Vec<ResolvedTaskJson>> {
        let store = self.load_store()?;
        let (project, err, _) = store.resolve_mark_identifier(project_id);
        let Some(project) = project else {
            return Err(not_found(err));
        };
        if !project.is_project() {
            return Err(ThingsSdkError::Validation(format!(
                "Not a project: {}",
                project.title
            )));
        }
        let mut children = store
            .tasks(None, Some(false), None)
            .into_iter()
            .filter(|task| store.effective_project_uuid(task).as_ref() == Some(&project.uuid))
            .collect::<Vec<_>>();
        children.sort_by_key(|task| task.index);
        Ok(self.tasks_json(&children, &store))
    }

    pub fn areas(&self) -> SdkResult<Vec<ResolvedAreaJson>> {
        let store = self.load_store()?;
        Ok(store
            .areas()
            .iter()
            .map(|area| build_area_json(area, &store))
            .collect())
    }

    pub fn area(&self, area_id: &str, query: AreaQuery) -> SdkResult<Vec<ResolvedTaskJson>> {
        let store = self.load_store()?;
        let (area, err, _) = store.resolve_area_identifier(area_id);
        let Some(area) = area else {
            return Err(not_found(err));
        };

        let status_filter = if query.all {
            None
        } else {
            Some(TaskStatus::Incomplete)
        };
        let mut items = store
            .projects(status_filter)
            .into_iter()
            .filter(|project| project.area.as_ref() == Some(&area.uuid))
            .collect::<Vec<_>>();
        items.extend(
            store
                .tasks(status_filter, Some(false), None)
                .into_iter()
                .filter(|task| {
                    task.area.as_ref() == Some(&area.uuid)
                        && !task.is_project()
                        && store.effective_project_uuid(task).is_none()
                }),
        );
        items.sort_by(|a, b| {
            let a_proj = if a.is_project() { 0 } else { 1 };
            let b_proj = if b.is_project() { 0 } else { 1 };
            (a_proj, a.index, &a.uuid).cmp(&(b_proj, b.index, &b.uuid))
        });
        Ok(self.tasks_json(&items, &store))
    }

    pub fn tags(&self) -> SdkResult<Vec<ResolvedTagJson>> {
        let store = self.load_store()?;
        Ok(build_tags_json(&store.tags(), &store))
    }

    pub fn find(&self, query: FindQuery) -> SdkResult<Vec<ResolvedTaskJson>> {
        let store = self.load_store()?;
        let args = query.into_find_args();
        let tasks =
            find_tasks(&store, &args, &self.current_day()).map_err(ThingsSdkError::Validation)?;
        Ok(self.tasks_json(&tasks, &store))
    }

    pub fn create_task(&self, request: CreateTaskRequest) -> SdkResult<MutationResult> {
        let store = self.load_store()?;
        let mut ctx = self.ctx();
        let args = NewArgs {
            title: request.title,
            in_target: request.in_target.unwrap_or_else(|| "inbox".to_string()),
            when: request.when,
            before_id: request.before_id,
            after_id: request.after_id,
            notes: request.notes.unwrap_or_default(),
            tags: request.tags,
            deadline_date: request.deadline,
        };
        let now = ctx.now_timestamp();
        let today = ctx.today_timestamp();
        let mut next_id = || ctx.next_id();
        let plan = build_new_plan(&args, &store, now, today, &mut next_id)
            .map_err(ThingsSdkError::Validation)?;
        let head_index = self.commit(&mut ctx, plan.changes, None)?;
        Ok(MutationResult {
            ids: vec![plan.new_uuid],
            titles: vec![plan.title],
            labels: vec!["created".to_string()],
            head_index: Some(head_index),
        })
    }

    pub fn edit_tasks(&self, request: EditTasksRequest) -> SdkResult<MutationResult> {
        let store = self.load_store()?;
        let mut ctx = self.ctx();
        let args = EditArgs {
            task_ids: ids(request.task_ids),
            title: request.title,
            notes: request.notes,
            move_target: request.move_target,
            tag_delta: TagDeltaArgs {
                add_tags: request.add_tags,
                remove_tags: request.remove_tags,
            },
            add_checklist: request.add_checklist,
            remove_checklist: request.remove_checklist,
            rename_checklist: request.rename_checklist,
        };
        let now = ctx.now_timestamp();
        let mut next_id = || ctx.next_id();
        let plan = build_edit_plan(&args, &store, now, &mut next_id)
            .map_err(ThingsSdkError::Validation)?;
        let titles = plan.tasks.iter().map(|task| task.title.clone()).collect();
        let ids = plan
            .tasks
            .iter()
            .map(|task| task.uuid.to_string())
            .collect();
        let head_index = self.commit(&mut ctx, plan.changes, None)?;
        Ok(MutationResult {
            ids,
            titles,
            labels: plan.labels,
            head_index: Some(head_index),
        })
    }

    pub fn mark_tasks(&self, request: MarkTasksRequest) -> SdkResult<MutationResult> {
        let store = self.load_store()?;
        let mut ctx = self.ctx();
        let args = MarkArgs {
            task_ids: ids(request.task_ids),
            done: matches!(request.status, MarkStatus::Done),
            incomplete: matches!(request.status, MarkStatus::Incomplete),
            canceled: matches!(request.status, MarkStatus::Canceled),
            check_ids: None,
            uncheck_ids: None,
            check_cancel_ids: None,
        };
        let (plan, successes, errors) = build_mark_status_plan(&args, &store, ctx.now_timestamp());
        if !errors.is_empty() {
            return Err(ThingsSdkError::Validation(errors.join("; ")));
        }
        let titles = successes.iter().map(|task| task.title.clone()).collect();
        let ids = successes.iter().map(|task| task.uuid.to_string()).collect();
        let head_index = self.commit(&mut ctx, plan.changes, None)?;
        Ok(MutationResult {
            ids,
            titles,
            labels: vec![format!("{:?}", request.status).to_lowercase()],
            head_index: Some(head_index),
        })
    }

    pub fn mutate_checklist(&self, request: MutateChecklistRequest) -> SdkResult<MutationResult> {
        let store = self.load_store()?;
        let mut ctx = self.ctx();
        let args = MarkArgs {
            task_ids: ids(vec![request.task_id.clone()]),
            done: false,
            incomplete: false,
            canceled: false,
            check_ids: matches!(request.status, ChecklistStatus::Checked)
                .then(|| request.checklist_ids.clone()),
            uncheck_ids: matches!(request.status, ChecklistStatus::Unchecked)
                .then(|| request.checklist_ids.clone()),
            check_cancel_ids: matches!(request.status, ChecklistStatus::Canceled)
                .then(|| request.checklist_ids.clone()),
        };
        let (task, err, _) = store.resolve_mark_identifier(&request.task_id);
        let Some(task) = task else {
            return Err(not_found(err));
        };
        if task.checklist_items.is_empty() {
            return Err(ThingsSdkError::Validation(format!(
                "Task has no checklist items: {}",
                task.title
            )));
        }
        let (plan, items, label) =
            build_mark_checklist_plan(&args, &task, &request.checklist_ids, ctx.now_timestamp())
                .map_err(ThingsSdkError::Validation)?;
        let head_index = self.commit(&mut ctx, plan.changes, None)?;
        Ok(MutationResult {
            ids: items.iter().map(|item| item.uuid.to_string()).collect(),
            titles: items.iter().map(|item| item.title.clone()).collect(),
            labels: vec![label],
            head_index: Some(head_index),
        })
    }

    pub fn schedule_task(&self, request: ScheduleTaskRequest) -> SdkResult<MutationResult> {
        let store = self.load_store()?;
        let mut ctx = self.ctx();
        let args = ScheduleArgs {
            task_id: request.task_id,
            when: request.when,
            deadline_date: request.deadline,
            clear_deadline: request.clear_deadline,
        };
        let plan = build_schedule_plan(&args, &store, ctx.now_timestamp(), ctx.today_timestamp())
            .map_err(ThingsSdkError::Validation)?;
        let mut changes = BTreeMap::new();
        changes.insert(
            plan.task.uuid.to_string(),
            WireObject::update(EntityType::from(plan.task.entity.clone()), plan.update),
        );
        let head_index = self.commit(&mut ctx, changes, None)?;
        Ok(MutationResult {
            ids: vec![plan.task.uuid.to_string()],
            titles: vec![plan.task.title],
            labels: plan.labels,
            head_index: Some(head_index),
        })
    }

    pub fn reorder_item(&self, request: ReorderItemRequest) -> SdkResult<MutationResult> {
        let store = self.load_store()?;
        let mut ctx = self.ctx();
        let args = ReorderArgs {
            item_id: request.item_id,
            before_id: request.before_id,
            after_id: request.after_id,
        };
        let plan = build_reorder_plan(
            &args,
            &store,
            ctx.now_timestamp(),
            ctx.today_timestamp(),
            None,
        )
        .map_err(ThingsSdkError::Validation)?;
        let mut last_head = None;
        for commit in plan.commits {
            last_head = Some(self.commit(&mut ctx, commit.changes, commit.ancestor_index)?);
        }
        Ok(MutationResult {
            ids: vec![plan.item.uuid.to_string()],
            titles: vec![plan.item.title],
            labels: vec![plan.reorder_label],
            head_index: last_head,
        })
    }

    pub fn delete_items(&self, request: DeleteItemsRequest) -> SdkResult<MutationResult> {
        let store = self.load_store()?;
        let mut ctx = self.ctx();
        let plan = build_delete_plan(
            &DeleteArgs {
                item_ids: ids(request.item_ids),
            },
            &store,
        );
        if plan.targets.is_empty() {
            return Err(ThingsSdkError::NotFound(
                "No matching items to delete.".to_string(),
            ));
        }
        let ids = plan.targets.iter().map(|(id, _, _)| id.clone()).collect();
        let titles = plan
            .targets
            .iter()
            .map(|(_, _, title)| title.clone())
            .collect();
        let head_index = self.commit(&mut ctx, plan.changes, None)?;
        Ok(MutationResult {
            ids,
            titles,
            labels: vec!["deleted".to_string()],
            head_index: Some(head_index),
        })
    }

    pub fn create_project(&self, request: CreateProjectRequest) -> SdkResult<MutationResult> {
        let store = self.load_store()?;
        let mut ctx = self.ctx();
        let args = ProjectsNewArgs {
            title: request.title,
            area: request.area,
            when: request.when,
            notes: request.notes.unwrap_or_default(),
            tags: request.tags,
            deadline_date: request.deadline,
        };
        let now = ctx.now_timestamp();
        let today = ctx.today_timestamp();
        let mut next_id = || ctx.next_id();
        let plan = build_project_new_plan(&args, &store, now, today, &mut next_id)
            .map_err(|err| project_create_error(&store, &args, err))?;
        let head_index = self.commit(&mut ctx, plan.changes, None)?;
        Ok(MutationResult {
            ids: vec![plan.uuid],
            titles: vec![plan.title],
            labels: vec!["created".to_string()],
            head_index: Some(head_index),
        })
    }

    pub fn edit_project(&self, request: EditProjectRequest) -> SdkResult<MutationResult> {
        let store = self.load_store()?;
        let mut ctx = self.ctx();
        let args = ProjectsEditArgs {
            project_id: request.project_id,
            title: request.title,
            move_target: request.move_target,
            notes: request.notes,
            tag_delta: TagDeltaArgs {
                add_tags: request.add_tags,
                remove_tags: request.remove_tags,
            },
        };
        let plan = build_projects_edit_plan(&args, &store, ctx.now_timestamp())
            .map_err(ThingsSdkError::Validation)?;
        let mut changes = BTreeMap::new();
        changes.insert(
            plan.project.uuid.to_string(),
            WireObject::update(EntityType::from(plan.project.entity.clone()), plan.update),
        );
        let head_index = self.commit(&mut ctx, changes, None)?;
        Ok(MutationResult {
            ids: vec![plan.project.uuid.to_string()],
            titles: vec![plan.project.title],
            labels: plan.labels,
            head_index: Some(head_index),
        })
    }

    pub fn create_area(&self, request: CreateAreaRequest) -> SdkResult<MutationResult> {
        let store = self.load_store()?;
        let mut ctx = self.ctx();
        let args = AreasNewArgs {
            title: request.title,
            tags: request.tags,
        };
        let mut next_id = || ctx.next_id();
        let plan =
            build_area_new_plan(&args, &store, &mut next_id).map_err(ThingsSdkError::Validation)?;
        let head_index = self.commit(&mut ctx, plan.changes, None)?;
        Ok(MutationResult {
            ids: vec![plan.uuid],
            titles: vec![plan.title],
            labels: vec!["created".to_string()],
            head_index: Some(head_index),
        })
    }

    pub fn edit_area(&self, request: EditAreaRequest) -> SdkResult<MutationResult> {
        let store = self.load_store()?;
        let mut ctx = self.ctx();
        let args = AreasEditArgs {
            area_id: request.area_id,
            title: request.title,
            tag_delta: TagDeltaArgs {
                add_tags: request.add_tags,
                remove_tags: request.remove_tags,
            },
        };
        let plan = build_areas_edit_plan(&args, &store, ctx.now_timestamp())
            .map_err(ThingsSdkError::Validation)?;
        let mut changes = BTreeMap::new();
        changes.insert(
            plan.area.uuid.to_string(),
            WireObject::update(EntityType::Area3, plan.update),
        );
        let head_index = self.commit(&mut ctx, changes, None)?;
        Ok(MutationResult {
            ids: vec![plan.area.uuid.to_string()],
            titles: vec![plan.area.title],
            labels: plan.labels,
            head_index: Some(head_index),
        })
    }

    pub fn create_tag(&self, request: CreateTagRequest) -> SdkResult<MutationResult> {
        let store = self.load_store()?;
        let mut ctx = self.ctx();
        let args = TagsNewArgs {
            name: request.name,
            parent: request.parent,
        };
        let mut next_id = || ctx.next_id();
        let plan =
            build_tag_new_plan(&args, &store, &mut next_id).map_err(ThingsSdkError::Validation)?;
        let head_index = self.commit(&mut ctx, plan.changes, None)?;
        Ok(MutationResult {
            ids: vec![plan.uuid],
            titles: vec![plan.name],
            labels: vec!["created".to_string()],
            head_index: Some(head_index),
        })
    }

    pub fn edit_tag(&self, request: EditTagRequest) -> SdkResult<MutationResult> {
        let store = self.load_store()?;
        let mut ctx = self.ctx();
        let args = TagsEditArgs {
            tag_id: request.tag_id,
            name: request.name,
            move_target: request.move_target,
        };
        let plan = build_tags_edit_plan(&args, &store, ctx.now_timestamp())
            .map_err(ThingsSdkError::Validation)?;
        let mut changes = BTreeMap::new();
        changes.insert(
            plan.tag.uuid.to_string(),
            WireObject::update(EntityType::Tag4, plan.update),
        );
        let head_index = self.commit(&mut ctx, changes, None)?;
        Ok(MutationResult {
            ids: vec![plan.tag.uuid.to_string()],
            titles: vec![plan.tag.title],
            labels: plan.labels,
            head_index: Some(head_index),
        })
    }

    pub fn delete_tag(&self, request: DeleteTagRequest) -> SdkResult<MutationResult> {
        let store = self.load_store()?;
        let mut ctx = self.ctx();
        let args = TagsDeleteArgs {
            tag_id: request.tag_id,
        };
        let (tag, err) = resolve_single_tag(&store, &args.tag_id);
        let Some(tag) = tag else {
            return Err(not_found(err));
        };
        let mut changes = BTreeMap::new();
        changes.insert(tag.uuid.to_string(), WireObject::delete(EntityType::Tag4));
        let head_index = self.commit(&mut ctx, changes, None)?;
        Ok(MutationResult {
            ids: vec![tag.uuid.to_string()],
            titles: vec![tag.title],
            labels: vec!["deleted".to_string()],
            head_index: Some(head_index),
        })
    }

    fn load_state(&self) -> SdkResult<RawState> {
        if let Some(journal_path) = &self.config.journal_path {
            let mut raw = String::new();
            if journal_path == std::path::Path::new("-") {
                std::io::stdin()
                    .read_to_string(&mut raw)
                    .map_err(|err| ThingsSdkError::Io(err.to_string()))?;
            } else {
                raw = fs::read_to_string(journal_path)
                    .map_err(|err| ThingsSdkError::Io(err.to_string()))?;
            };
            let items: Vec<WireItem> =
                serde_json::from_str(&raw).map_err(|err| ThingsSdkError::Sync(err.to_string()))?;
            return Ok(fold_items(items));
        }

        if self.config.cache_only || self.config.dry_run {
            return fold_state_from_append_log(&append_log_dir())
                .map_err(|err| ThingsSdkError::Sync(err.to_string()));
        }

        let (email, password) = load_auth().map_err(|err| ThingsSdkError::Auth(err.to_string()))?;
        let mut client = ThingsCloudClient::new(email, password)
            .map_err(|err| ThingsSdkError::Auth(err.to_string()))?;
        get_state_with_append_log(&mut client, append_log_dir())
            .map_err(|err| ThingsSdkError::Sync(err.to_string()))
    }

    fn tasks_json(&self, tasks: &[Task], store: &ThingsStore) -> Vec<ResolvedTaskJson> {
        build_tasks_json(tasks, store, &self.current_day())
    }

    fn current_day(&self) -> chrono::DateTime<Utc> {
        let ts = self
            .config
            .today_ts
            .unwrap_or_else(|| crate::common::today_utc().timestamp());
        Utc.timestamp_opt(ts, 0)
            .single()
            .unwrap_or_else(crate::common::today_utc)
    }

    fn ctx(&self) -> DefaultCmdCtx {
        DefaultCmdCtx::from_cli(&Cli {
            no_color: true,
            json: true,
            no_sync: self.config.cache_only || self.config.dry_run,
            no_cloud: self.config.dry_run,
            log_level: logging::Level::Info,
            log_format: logging::LogFormat::Auto,
            log_filter: None,
            today_ts: self.config.today_ts,
            now_ts: self.config.now_ts,
            load_journal: self.config.journal_path.clone(),
            command: None,
        })
    }

    fn commit(
        &self,
        ctx: &mut DefaultCmdCtx,
        changes: BTreeMap<String, WireObject>,
        ancestor_index: Option<i64>,
    ) -> SdkResult<i64> {
        ctx.commit_changes(changes, ancestor_index)
            .map_err(|err| ThingsSdkError::CloudCommit(err.to_string()))
    }
}

impl FindQuery {
    fn into_find_args(self) -> FindArgs {
        FindArgs {
            detailed: DetailedArgs { detailed: false },
            query: self.query,
            incomplete: self.incomplete,
            notes: self.notes,
            checklists: self.checklists,
            completed: self.completed,
            canceled: self.canceled,
            any_status: self.any_status,
            tag_filters: ids(self.tags),
            project_filters: ids(self.projects),
            area_filters: ids(self.areas),
            inbox: self.inbox,
            today: self.today,
            someday: self.someday,
            evening: self.evening,
            has_deadline: self.has_deadline,
            no_deadline: self.no_deadline,
            recurring: self.recurring,
            deadline: self.deadline,
            scheduled: self.scheduled,
            created: self.created,
            completed_on: self.completed_on,
        }
    }
}

fn ids(values: Vec<String>) -> Vec<IdentifierToken> {
    values.into_iter().map(IdentifierToken::from).collect()
}

fn not_found(message: String) -> ThingsSdkError {
    ThingsSdkError::NotFound(message)
}

fn project_create_error(
    store: &ThingsStore,
    args: &ProjectsNewArgs,
    err: String,
) -> ThingsSdkError {
    if let Some(area_id) = &args.area {
        let (area, area_err, _) = store.resolve_area_identifier(area_id);
        if area.is_none() && area_err == err {
            return not_found(err);
        }
    }
    ThingsSdkError::Validation(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
    }

    #[test]
    fn sdk_reads_today_view_from_journal_fixture() {
        let service = ThingsService::new(ThingsServiceConfig {
            journal_path: Some(fixture("trycmd/today/basic_list.in/journal.json")),
            today_ts: Some(1_774_396_800),
            ..Default::default()
        });

        let tasks = service.today().expect("today view");
        let value = serde_json::to_value(tasks).expect("serialize tasks");
        let titles = value
            .as_array()
            .expect("array")
            .iter()
            .map(|task| task["title"].as_str().expect("title").to_string())
            .collect::<Vec<_>>();

        assert_eq!(titles, vec!["Morning workout", "Read email"]);
    }

    #[test]
    fn sdk_can_plan_create_task_in_dry_run_mode() {
        let service = ThingsService::new(ThingsServiceConfig {
            dry_run: true,
            journal_path: Some(fixture("trycmd/today/basic_list.in/journal.json")),
            today_ts: Some(1_700_000_000),
            now_ts: Some(1_700_000_000.0),
            ..Default::default()
        });

        let result = service
            .create_task(CreateTaskRequest {
                title: "Ship release".to_string(),
                in_target: None,
                when: None,
                before_id: None,
                after_id: None,
                notes: None,
                tags: None,
                deadline: None,
            })
            .expect("create task");

        assert_eq!(result.titles, vec!["Ship release"]);
        assert_eq!(result.labels, vec!["created"]);
        assert_eq!(result.head_index, Some(1));
        assert_eq!(result.ids.len(), 1);
    }
}
