use serde::Serialize;

use crate::next_actions;

#[derive(Debug, Serialize)]
pub struct Manifest {
    pub name: &'static str,
    pub description: &'static str,
    pub version: &'static str,
    pub profiles: Profiles,
    pub agent_mode: AgentMode,
    pub help_argv: Vec<String>,
    pub global_flags: Vec<FlagInfo>,
    pub resources: Vec<ResourceManifest>,
    pub doctor_argv: Vec<String>,
    pub skills: SkillsProfile,
}

#[derive(Debug, Serialize)]
pub struct Profiles {
    pub discovery: bool,
    pub structured_input: bool,
    pub skills: bool,
}

#[derive(Debug, Serialize)]
pub struct AgentMode {
    pub argv_prefix: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FlagInfo {
    pub name: &'static str,
    pub flag: &'static str,
    pub scope: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ResourceManifest {
    pub name: &'static str,
    pub commands: Vec<&'static str>,
    pub context_argv: Vec<String>,
    pub help_argv: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SkillsProfile {
    pub implemented: bool,
    pub source: Option<&'static str>,
    pub list_argv: Vec<String>,
    pub install_argv: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ResourceContext {
    pub name: &'static str,
    pub description: &'static str,
    pub help_argv: Vec<String>,
    pub id: IdContext,
    pub commands: Vec<CommandInfo>,
    pub relationships: Relationships,
    pub workflows: Vec<Workflow>,
    pub pitfalls: Vec<&'static str>,
    pub skills: Vec<SkillHint>,
    pub safe_start_argv: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct IdContext {
    pub name: &'static str,
    pub format: &'static str,
    pub find_argv: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CommandInfo {
    pub name: &'static str,
    pub mutates: bool,
    pub requires_auth: bool,
    pub help_argv: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Relationships {
    pub requires: Vec<&'static str>,
    pub optional: Vec<&'static str>,
    pub produces: Vec<&'static str>,
    pub related: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct Workflow {
    pub name: &'static str,
    pub argv: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SkillHint {
    pub id: &'static str,
    pub status: &'static str,
}

pub fn manifest() -> Manifest {
    Manifest {
        name: "coval",
        description:
            "Coval is the testing and observability infrastructure for conversational AI agents: simulate interactions between your agent and personas using test cases from test sets, then evaluate simulations and uploaded production conversations with metrics.",
        version: env!("CARGO_PKG_VERSION"),
        profiles: Profiles {
            discovery: true,
            structured_input: true,
            skills: true,
        },
        agent_mode: AgentMode {
            argv_prefix: next_actions::argv(std::iter::empty::<&str>()),
        },
        help_argv: help_argv(std::iter::empty::<&str>()),
        global_flags: vec![
            FlagInfo {
                name: "agent",
                flag: "--agent",
                scope: "global",
            },
            FlagInfo {
                name: "api-key",
                flag: "--api-key",
                scope: "global",
            },
            FlagInfo {
                name: "api-url",
                flag: "--api-url",
                scope: "global",
            },
            FlagInfo {
                name: "format",
                flag: "--format",
                scope: "global",
            },
            FlagInfo {
                name: "input-json",
                flag: "--input-json",
                scope: "body commands",
            },
        ],
        resources: RESOURCE_SPECS
            .iter()
            .map(|spec| ResourceManifest {
                name: spec.name,
                commands: spec.commands.to_vec(),
                context_argv: next_actions::argv([spec.name, "context"]),
                help_argv: help_argv([spec.name]),
            })
            .collect(),
        doctor_argv: next_actions::argv(["agent", "doctor"]),
        skills: SkillsProfile {
            implemented: true,
            source: None,
            list_argv: next_actions::argv(["agent", "skills", "list", "--source", "<path>"]),
            install_argv: next_actions::argv([
                "agent",
                "skills",
                "install",
                "<skill-id>",
                "--source",
                "<path>",
                "--dest",
                "<path>",
            ]),
        },
    }
}

pub fn resource_context(name: &str) -> Option<ResourceContext> {
    let spec = RESOURCE_SPECS.iter().find(|spec| spec.name == name)?;
    Some(ResourceContext {
        name: spec.name,
        description: spec.description,
        help_argv: help_argv([spec.name]),
        id: IdContext {
            name: spec.id_name,
            format: spec.id_format,
            find_argv: next_actions::argv([spec.name, "list"]),
        },
        commands: spec
            .commands
            .iter()
            .map(|command| CommandInfo {
                name: command,
                mutates: mutates(command),
                requires_auth: *command != "context",
                help_argv: command_help_argv(spec.name, command),
            })
            .collect(),
        relationships: Relationships {
            requires: spec.requires.to_vec(),
            optional: spec.optional.to_vec(),
            produces: spec.produces.to_vec(),
            related: spec.related.to_vec(),
        },
        workflows: spec
            .workflows
            .iter()
            .map(|workflow| Workflow {
                name: workflow.name,
                argv: next_actions::argv(workflow.argv.iter().copied()),
            })
            .collect(),
        pitfalls: spec.pitfalls.to_vec(),
        skills: Vec::new(),
        safe_start_argv: next_actions::argv([spec.name, "list"]),
    })
}

fn mutates(command: &str) -> bool {
    matches!(
        command,
        "create"
            | "update"
            | "delete"
            | "launch"
            | "submit"
            | "widgets.create"
            | "widgets.update"
            | "widgets.delete"
    )
}

fn help_argv<I, S>(args: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    std::iter::once("coval".to_string())
        .chain(args.into_iter().map(|arg| arg.as_ref().to_string()))
        .chain(std::iter::once("--help".to_string()))
        .collect()
}

fn command_help_argv(resource: &str, command: &str) -> Vec<String> {
    help_argv(std::iter::once(resource).chain(command.split('.')))
}

struct ResourceSpec {
    name: &'static str,
    commands: &'static [&'static str],
    description: &'static str,
    id_name: &'static str,
    id_format: &'static str,
    requires: &'static [&'static str],
    optional: &'static [&'static str],
    produces: &'static [&'static str],
    related: &'static [&'static str],
    workflows: &'static [WorkflowSpec],
    pitfalls: &'static [&'static str],
}

struct WorkflowSpec {
    name: &'static str,
    argv: &'static [&'static str],
}

const RESOURCE_SPECS: &[ResourceSpec] = &[
    ResourceSpec {
        name: "agents",
        commands: &["context", "list", "get", "create", "update", "delete"],
        description: "Agents are Coval evaluation targets such as voice, chat, SMS, websocket, and outbound voice systems.",
        id_name: "agent_id",
        id_format: "22-character Coval ID",
        requires: &[],
        optional: &["metrics", "test-sets", "mutations"],
        produces: &["runs"],
        related: &["runs", "mutations", "metrics", "test-sets"],
        workflows: &[WorkflowSpec {
            name: "Inspect agents",
            argv: &["agents", "list"],
        }],
        pitfalls: &["Agent type controls required creation fields."],
    },
    ResourceSpec {
        name: "conversations",
        commands: &[
            "context",
            "list",
            "get",
            "delete",
            "audio",
            "submit",
            "metrics",
            "metric-detail",
        ],
        description: "Conversations are submitted or monitored customer interactions that can be evaluated with metrics.",
        id_name: "conversation_id",
        id_format: "Conversation ID returned by Coval",
        requires: &[],
        optional: &["agents", "metrics"],
        produces: &["metric outputs"],
        related: &["simulations", "metrics", "agents"],
        workflows: &[WorkflowSpec {
            name: "Inspect recent conversations",
            argv: &["conversations", "list"],
        }],
        pitfalls: &["Conversation IDs and simulation IDs are distinct resources."],
    },
    ResourceSpec {
        name: "runs",
        commands: &["context", "list", "get", "launch", "update", "watch", "delete"],
        description: "Runs execute test cases against an agent with a persona, metrics, and optional mutations.",
        id_name: "run_id",
        id_format: "Run ID returned by Coval",
        requires: &["agents", "personas", "test-sets"],
        optional: &["metrics", "mutations"],
        produces: &["simulations", "metric outputs"],
        related: &["simulations", "agents", "personas", "test-sets"],
        workflows: &[WorkflowSpec {
            name: "Launch a run",
            argv: &[
                "runs",
                "launch",
                "--agent-id",
                "<agent_id>",
                "--persona-id",
                "<persona_id>",
                "--test-set-id",
                "<test_set_id>",
            ],
        }],
        pitfalls: &["Launch returns a run; use watch or get before assuming outputs are ready."],
    },
    ResourceSpec {
        name: "simulations",
        commands: &[
            "context",
            "list",
            "get",
            "delete",
            "audio",
            "metrics",
            "metric-detail",
        ],
        description: "Simulations are individual run outputs for one test case and persona interaction.",
        id_name: "simulation_id",
        id_format: "Simulation ID returned by Coval",
        requires: &["runs"],
        optional: &["metrics"],
        produces: &["audio URLs", "metric outputs"],
        related: &["runs", "conversations", "metrics"],
        workflows: &[WorkflowSpec {
            name: "List simulations for a run",
            argv: &["simulations", "list", "--run-id", "<run_id>"],
        }],
        pitfalls: &["Not every simulation has audio."],
    },
    ResourceSpec {
        name: "test-sets",
        commands: &["context", "list", "get", "create", "update", "delete"],
        description: "Test sets group test cases used as run inputs.",
        id_name: "test_set_id",
        id_format: "8-character Coval test set ID",
        requires: &[],
        optional: &["test-cases"],
        produces: &["runs"],
        related: &["test-cases", "runs"],
        workflows: &[WorkflowSpec {
            name: "Inspect test sets",
            argv: &["test-sets", "list"],
        }],
        pitfalls: &["Runs require a test set; empty test sets will not exercise an agent."],
    },
    ResourceSpec {
        name: "test-cases",
        commands: &["context", "list", "get", "create", "update", "delete"],
        description: "Test cases are individual inputs and expected outputs inside a test set.",
        id_name: "test_case_id",
        id_format: "Coval test case ID",
        requires: &["test-sets"],
        optional: &[],
        produces: &["simulation inputs"],
        related: &["test-sets", "runs"],
        workflows: &[WorkflowSpec {
            name: "List cases in a set",
            argv: &["test-cases", "list", "--test-set-id", "<test_set_id>"],
        }],
        pitfalls: &["Bulk stdin create accepts JSON lines, not one JSON array."],
    },
    ResourceSpec {
        name: "personas",
        commands: &[
            "context",
            "list",
            "get",
            "create",
            "update",
            "delete",
            "background-sounds",
            "phone-numbers",
        ],
        description: "Personas define simulated users, voices, languages, behavior, and background sounds for evaluations.",
        id_name: "persona_id",
        id_format: "22-character Coval ID",
        requires: &[],
        optional: &["phone numbers", "custom background sounds"],
        produces: &["runs"],
        related: &["runs", "test-sets"],
        workflows: &[WorkflowSpec {
            name: "Inspect personas",
            argv: &["personas", "list"],
        }],
        pitfalls: &[
            "Voice and language choices affect run behavior and audio output.",
            "Custom background sounds are referenced as custom:<background_sound_id> after upload.",
        ],
    },
    ResourceSpec {
        name: "metrics",
        commands: &["context", "list", "get", "create", "update", "delete"],
        description: "Metrics score conversations and simulations with built-in or configured evaluation logic.",
        id_name: "metric_id",
        id_format: "22-character Coval ID",
        requires: &[],
        optional: &["agents", "runs"],
        produces: &["metric outputs"],
        related: &["agents", "runs", "simulations", "conversations"],
        workflows: &[WorkflowSpec {
            name: "Inspect metrics",
            argv: &["metrics", "list"],
        }],
        pitfalls: &["Metric type controls required creation fields."],
    },
    ResourceSpec {
        name: "mutations",
        commands: &["context", "list", "get", "create", "update", "delete"],
        description: "Mutations are agent configuration overrides used to compare variants in evaluations.",
        id_name: "mutation_id",
        id_format: "26-character ULID",
        requires: &["agents"],
        optional: &[],
        produces: &["runs"],
        related: &["agents", "runs"],
        workflows: &[WorkflowSpec {
            name: "List mutations for an agent",
            argv: &["mutations", "list", "--agent-id", "<agent_id>"],
        }],
        pitfalls: &["Config overrides are deep-merged with the parent agent."],
    },
    ResourceSpec {
        name: "api-keys",
        commands: &["context", "list", "create", "update", "delete"],
        description: "API keys authenticate CLI and integration access to Coval.",
        id_name: "api_key_id",
        id_format: "Coval API key ID",
        requires: &[],
        optional: &[],
        produces: &["credentials"],
        related: &["config", "auth"],
        workflows: &[WorkflowSpec {
            name: "List API keys",
            argv: &["api-keys", "list"],
        }],
        pitfalls: &["New secret values are only shown once."],
    },
    ResourceSpec {
        name: "run-templates",
        commands: &["context", "list", "get", "create", "update", "delete"],
        description: "Run templates save reusable run launch configuration.",
        id_name: "run_template_id",
        id_format: "Coval run template ID",
        requires: &[],
        optional: &["agents", "personas", "test-sets", "metrics", "mutations"],
        produces: &["scheduled-runs"],
        related: &["scheduled-runs", "runs"],
        workflows: &[WorkflowSpec {
            name: "Inspect run templates",
            argv: &["run-templates", "list"],
        }],
        pitfalls: &["Templates describe future runs; they do not launch a run by themselves."],
    },
    ResourceSpec {
        name: "scheduled-runs",
        commands: &["context", "list", "get", "create", "update", "delete"],
        description: "Scheduled runs execute run templates on a recurring schedule.",
        id_name: "scheduled_run_id",
        id_format: "Coval scheduled run ID",
        requires: &["run-templates"],
        optional: &[],
        produces: &["runs"],
        related: &["run-templates", "runs"],
        workflows: &[WorkflowSpec {
            name: "Inspect schedules",
            argv: &["scheduled-runs", "list"],
        }],
        pitfalls: &["Schedule expressions and timezones must match the intended runtime cadence."],
    },
    ResourceSpec {
        name: "dashboards",
        commands: &[
            "context",
            "list",
            "get",
            "create",
            "update",
            "delete",
            "widgets.list",
            "widgets.get",
            "widgets.create",
            "widgets.update",
            "widgets.delete",
        ],
        description: "Dashboards organize Coval reporting widgets.",
        id_name: "dashboard_id",
        id_format: "Dashboard ID extracted from the dashboard resource name",
        requires: &[],
        optional: &["widgets"],
        produces: &["widgets"],
        related: &["runs", "metrics"],
        workflows: &[WorkflowSpec {
            name: "Inspect dashboards",
            argv: &["dashboards", "list"],
        }],
        pitfalls: &["Widget commands require the parent dashboard ID."],
    },
    ResourceSpec {
        name: "review-annotations",
        commands: &["context", "list", "get", "create", "update", "delete"],
        description: "Review annotations assign metric output review work and ground truth corrections.",
        id_name: "annotation_id",
        id_format: "Coval review annotation ID",
        requires: &["simulation outputs", "metrics"],
        optional: &["review-projects"],
        produces: &["review decisions"],
        related: &["review-projects", "metrics", "simulations"],
        workflows: &[WorkflowSpec {
            name: "Inspect review annotations",
            argv: &["review-annotations", "list"],
        }],
        pitfalls: &["Ground truth subvalues must be valid JSON arrays."],
    },
    ResourceSpec {
        name: "review-projects",
        commands: &["context", "list", "get", "create", "update", "delete"],
        description: "Review projects group review annotations and reviewer assignments.",
        id_name: "project_id",
        id_format: "Coval review project ID",
        requires: &[],
        optional: &["review-annotations", "metrics", "simulations"],
        produces: &["review workflows"],
        related: &["review-annotations"],
        workflows: &[WorkflowSpec {
            name: "Inspect review projects",
            argv: &["review-projects", "list"],
        }],
        pitfalls: &["Assignee lists are comma-delimited in CLI flags."],
    },
    ResourceSpec {
        name: "reports",
        commands: &["context", "list", "get", "create", "update", "delete"],
        description: "Reports save a comparison view across multiple runs, grouped by a chosen dimension.",
        id_name: "report_id",
        id_format: "26-character ULID",
        requires: &["runs"],
        optional: &[],
        produces: &["shareable report views"],
        related: &["runs", "agents", "personas", "mutations"],
        workflows: &[WorkflowSpec {
            name: "Compare runs by test case",
            argv: &[
                "reports",
                "create",
                "--name",
                "<name>",
                "--run-ids",
                "<run_id>",
                "--compare-by",
                "test_case",
            ],
        }],
        pitfalls: &[
            "metadata_key is required when compare-by is metadata and rejected otherwise.",
            "PUBLIC reports also mark their runs public.",
        ],
    },
];
