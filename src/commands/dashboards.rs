use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{
    CreateDashboardRequest, CreateWidgetRequest, ListParams, UpdateDashboardRequest,
    UpdateWidgetRequest, WidgetType,
};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum DashboardCommands {
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
    #[arg(long)]
    name: String,
}

#[derive(Args)]
pub struct UpdateArgs {
    dashboard_id: String,
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
    #[arg(long)]
    name: String,
    #[arg(long, value_enum)]
    r#type: WidgetType,
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
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        DashboardCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.dashboards().list(params).await?;
            print_list(&response.dashboards, format);
        }
        DashboardCommands::Get(args) => {
            let dashboard = client.dashboards().get(&args.dashboard_id).await?;
            print_one(&dashboard, format);
        }
        DashboardCommands::Create(args) => {
            let req = CreateDashboardRequest {
                display_name: args.name,
            };
            let dashboard = client.dashboards().create(req).await?;
            print_one(&dashboard, format);
        }
        DashboardCommands::Update(args) => {
            let req = UpdateDashboardRequest {
                display_name: args.name,
            };
            let dashboard = client.dashboards().update(&args.dashboard_id, req).await?;
            print_one(&dashboard, format);
        }
        DashboardCommands::Delete(args) => {
            client.dashboards().delete(&args.dashboard_id).await?;
            print_success("Dashboard deleted.");
        }
        DashboardCommands::Widgets { command } => execute_widget(command, client, format).await?,
    }
    Ok(())
}

async fn execute_widget(
    cmd: WidgetCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        WidgetCommands::List(args) => {
            let params = ListParams {
                page_size: Some(args.page_size),
                ..Default::default()
            };
            let response = client.widgets(&args.dashboard_id).list(params).await?;
            print_list(&response.widgets, format);
        }
        WidgetCommands::Get(args) => {
            let widget = client
                .widgets(&args.dashboard_id)
                .get(&args.widget_id)
                .await?;
            print_one(&widget, format);
        }
        WidgetCommands::Create(args) => {
            let config = args.config.as_ref().map(|c| parse_config(c)).transpose()?;
            let req = CreateWidgetRequest {
                display_name: args.name,
                widget_type: args.r#type,
                config,
                grid_x: args.grid_x,
                grid_y: args.grid_y,
                grid_w: args.grid_w,
                grid_h: args.grid_h,
            };
            let widget = client.widgets(&args.dashboard_id).create(req).await?;
            print_one(&widget, format);
        }
        WidgetCommands::Update(args) => {
            let config = args.config.as_ref().map(|c| parse_config(c)).transpose()?;
            let req = UpdateWidgetRequest {
                display_name: args.name,
                widget_type: args.r#type,
                config,
                grid_x: args.grid_x,
                grid_y: args.grid_y,
                grid_w: args.grid_w,
                grid_h: args.grid_h,
            };
            let widget = client
                .widgets(&args.dashboard_id)
                .update(&args.widget_id, req)
                .await?;
            print_one(&widget, format);
        }
        WidgetCommands::Delete(args) => {
            client
                .widgets(&args.dashboard_id)
                .delete(&args.widget_id)
                .await?;
            print_success("Widget deleted.");
        }
    }
    Ok(())
}
