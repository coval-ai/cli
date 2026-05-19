use anyhow::{bail, Result};
use clap::{Args, Subcommand};

use crate::client::models::{
    CreateDashboardRequest, CreateWidgetRequest, ListParams, UpdateDashboardRequest,
    UpdateWidgetRequest, WidgetType,
};
use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, NextAction,
    OutputContext,
};

#[derive(Subcommand)]
pub enum DashboardCommands {
    Context,
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    Widgets {
        #[command(subcommand)]
        command: WidgetCommands,
    },
}

impl DashboardCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::List(_) => "list",
            Self::Get(_) => "get",
            Self::Create(_) => "create",
            Self::Update(_) => "update",
            Self::Delete(_) => "delete",
            Self::Widgets { command } => command.operation(),
        }
    }
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(long)]
    filter: Option<String>,
    #[arg(long, default_value = "50")]
    page_size: u32,
    #[arg(long)]
    order_by: Option<String>,
}

#[derive(Args)]
pub struct GetArgs {
    dashboard_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    #[command(flatten)]
    input_json: InputJsonArg,
    #[arg(long)]
    name: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    dashboard_id: String,
    #[command(flatten)]
    input_json: InputJsonArg,
    #[arg(long)]
    name: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    dashboard_id: String,
}

#[derive(Subcommand)]
pub enum WidgetCommands {
    List(WidgetListArgs),
    Get(WidgetGetArgs),
    Create(WidgetCreateArgs),
    Update(WidgetUpdateArgs),
    Delete(WidgetDeleteArgs),
}

impl WidgetCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::List(_) => "widgets.list",
            Self::Get(_) => "widgets.get",
            Self::Create(_) => "widgets.create",
            Self::Update(_) => "widgets.update",
            Self::Delete(_) => "widgets.delete",
        }
    }
}

#[derive(Args)]
pub struct WidgetListArgs {
    dashboard_id: String,
    #[arg(long, default_value = "50")]
    page_size: u32,
}

#[derive(Args)]
pub struct WidgetGetArgs {
    dashboard_id: String,
    widget_id: String,
}

#[derive(Args)]
pub struct WidgetCreateArgs {
    dashboard_id: String,
    #[command(flatten)]
    input_json: InputJsonArg,
    #[arg(long)]
    name: Option<String>,
    #[arg(long, value_enum)]
    r#type: Option<WidgetType>,
    #[arg(long)]
    config: Option<String>,
    #[arg(long)]
    grid_x: Option<i32>,
    #[arg(long)]
    grid_y: Option<i32>,
    #[arg(long)]
    grid_w: Option<i32>,
    #[arg(long)]
    grid_h: Option<i32>,
}

#[derive(Args)]
pub struct WidgetUpdateArgs {
    dashboard_id: String,
    widget_id: String,
    #[command(flatten)]
    input_json: InputJsonArg,
    #[arg(long)]
    name: Option<String>,
    #[arg(long, value_enum)]
    r#type: Option<WidgetType>,
    #[arg(long)]
    config: Option<String>,
    #[arg(long)]
    grid_x: Option<i32>,
    #[arg(long)]
    grid_y: Option<i32>,
    #[arg(long)]
    grid_w: Option<i32>,
    #[arg(long)]
    grid_h: Option<i32>,
}

#[derive(Args)]
pub struct WidgetDeleteArgs {
    dashboard_id: String,
    widget_id: String,
}

fn validate_widget_grid(grid_w: Option<i32>, grid_h: Option<i32>) -> Result<()> {
    if let Some(w) = grid_w {
        if w <= 0 {
            bail!("--grid-w must be greater than 0");
        }
    }
    if let Some(h) = grid_h {
        if h <= 0 {
            bail!("--grid-h must be greater than 0");
        }
    }
    Ok(())
}

fn parse_config(raw: &str) -> Result<serde_json::Value> {
    if let Some(path) = raw.strip_prefix('@') {
        let contents = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&contents)?)
    } else {
        Ok(serde_json::from_str(raw)?)
    }
}

pub async fn execute(
    cmd: DashboardCommands,
    client: &CovalClient,
    ctx: &OutputContext,
) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        DashboardCommands::Context => {
            return crate::commands::agent::resource_context("dashboards", ctx);
        }
        DashboardCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.dashboards().list(params).await?;
            emit_list_with_actions(
                ctx,
                "dashboards",
                operation,
                &response.dashboards,
                list_actions(
                    response
                        .dashboards
                        .first()
                        .map(|dashboard| resource_id(&dashboard.name)),
                ),
            );
        }
        DashboardCommands::Get(args) => {
            let dashboard = client.dashboards().get(&args.dashboard_id).await?;
            emit_one_with_actions(
                ctx,
                "dashboards",
                operation,
                &dashboard,
                dashboard_actions(&args.dashboard_id),
            );
        }
        DashboardCommands::Create(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "display_name", args.name)?;
            let req: CreateDashboardRequest = input_json::finish(input)?;
            let dashboard = client.dashboards().create(req).await?;
            let dashboard_id = resource_id(&dashboard.name);
            emit_one_with_actions(
                ctx,
                "dashboards",
                operation,
                &dashboard,
                dashboard_actions(&dashboard_id),
            );
        }
        DashboardCommands::Update(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "display_name", args.name)?;
            let req: UpdateDashboardRequest = input_json::finish(input)?;
            let dashboard = client.dashboards().update(&args.dashboard_id, req).await?;
            emit_one_with_actions(
                ctx,
                "dashboards",
                operation,
                &dashboard,
                dashboard_actions(&args.dashboard_id),
            );
        }
        DashboardCommands::Delete(args) => {
            client.dashboards().delete(&args.dashboard_id).await?;
            emit_success_with_actions(
                ctx,
                "dashboards",
                operation,
                "Dashboard deleted.",
                next_actions::delete_result("dashboards"),
            );
        }
        DashboardCommands::Widgets { command } => execute_widget(command, client, ctx).await?,
    }
    Ok(())
}

async fn execute_widget(
    cmd: WidgetCommands,
    client: &CovalClient,
    ctx: &OutputContext,
) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        WidgetCommands::List(args) => {
            let params = ListParams {
                page_size: Some(args.page_size),
                ..Default::default()
            };
            let response = client.widgets(&args.dashboard_id).list(params).await?;
            emit_list_with_actions(
                ctx,
                "widgets",
                operation,
                &response.widgets,
                vec![
                    next_actions::get("dashboards", &args.dashboard_id).primary(),
                    next_actions::context("dashboards"),
                ],
            );
        }
        WidgetCommands::Get(args) => {
            let widget = client
                .widgets(&args.dashboard_id)
                .get(&args.widget_id)
                .await?;
            emit_one_with_actions(
                ctx,
                "widgets",
                operation,
                &widget,
                vec![
                    next_actions::get("dashboards", &args.dashboard_id).primary(),
                    next_actions::context("dashboards"),
                ],
            );
        }
        WidgetCommands::Create(args) => {
            validate_widget_grid(args.grid_w, args.grid_h)?;
            let mut input = args.input_json.object()?;
            let config = args.config.as_ref().map(|c| parse_config(c)).transpose()?;
            input_json::insert(&mut input, "display_name", args.name)?;
            input_json::insert(&mut input, "type", args.r#type)?;
            input_json::insert(&mut input, "config", config)?;
            input_json::insert(&mut input, "grid_x", args.grid_x)?;
            input_json::insert(&mut input, "grid_y", args.grid_y)?;
            input_json::insert(&mut input, "grid_w", args.grid_w)?;
            input_json::insert(&mut input, "grid_h", args.grid_h)?;
            let req: CreateWidgetRequest = input_json::finish(input)?;
            let widget = client.widgets(&args.dashboard_id).create(req).await?;
            emit_one_with_actions(
                ctx,
                "widgets",
                operation,
                &widget,
                vec![
                    next_actions::dashboard_widgets(&args.dashboard_id).primary(),
                    next_actions::context("dashboards"),
                ],
            );
        }
        WidgetCommands::Update(args) => {
            validate_widget_grid(args.grid_w, args.grid_h)?;
            let mut input = args.input_json.object()?;
            let config = args.config.as_ref().map(|c| parse_config(c)).transpose()?;
            input_json::insert(&mut input, "display_name", args.name)?;
            input_json::insert(&mut input, "type", args.r#type)?;
            input_json::insert(&mut input, "config", config)?;
            input_json::insert(&mut input, "grid_x", args.grid_x)?;
            input_json::insert(&mut input, "grid_y", args.grid_y)?;
            input_json::insert(&mut input, "grid_w", args.grid_w)?;
            input_json::insert(&mut input, "grid_h", args.grid_h)?;
            let req: UpdateWidgetRequest = input_json::finish(input)?;
            let widget = client
                .widgets(&args.dashboard_id)
                .update(&args.widget_id, req)
                .await?;
            emit_one_with_actions(
                ctx,
                "widgets",
                operation,
                &widget,
                vec![
                    next_actions::dashboard_widgets(&args.dashboard_id).primary(),
                    next_actions::context("dashboards"),
                ],
            );
        }
        WidgetCommands::Delete(args) => {
            client
                .widgets(&args.dashboard_id)
                .delete(&args.widget_id)
                .await?;
            emit_success_with_actions(
                ctx,
                "widgets",
                operation,
                "Widget deleted.",
                vec![
                    next_actions::dashboard_widgets(&args.dashboard_id).primary(),
                    next_actions::context("dashboards"),
                ],
            );
        }
    }
    Ok(())
}

fn resource_id(name: &str) -> String {
    name.rsplit('/').next().unwrap_or(name).to_string()
}

fn list_actions(id: Option<String>) -> Vec<NextAction> {
    let mut actions = vec![next_actions::context("dashboards")];
    if let Some(id) = id {
        actions.insert(0, next_actions::get("dashboards", &id).primary());
    }
    actions
}

fn dashboard_actions(dashboard_id: &str) -> Vec<NextAction> {
    vec![
        next_actions::dashboard_widgets(dashboard_id).primary(),
        next_actions::context("dashboards"),
    ]
}
