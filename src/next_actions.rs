use crate::output::NextAction;

pub fn context(resource: &str) -> NextAction {
    NextAction::new(
        format!("{resource}.context"),
        format!("{resource} context"),
        argv([resource, "context"]),
        true,
    )
}

pub fn list(resource: &str) -> NextAction {
    NextAction::new(
        format!("{resource}.list"),
        format!("List {resource}"),
        argv([resource, "list"]),
        true,
    )
}

pub fn get(resource: &str, id: &str) -> NextAction {
    NextAction::new(
        format!("{resource}.get"),
        format!("Get {resource}"),
        argv([resource, "get", id]),
        true,
    )
}

pub fn list_result(resource: &str, id: Option<&str>) -> Vec<NextAction> {
    let mut actions = vec![context(resource)];
    if let Some(id) = id {
        actions.insert(0, get(resource, id).primary());
    }
    actions
}

pub fn item_result(resource: &str, id: &str) -> Vec<NextAction> {
    vec![get(resource, id).primary(), context(resource)]
}

pub fn delete_result(resource: &str) -> Vec<NextAction> {
    vec![list(resource).primary(), context(resource)]
}

pub fn runs_watch(run_id: &str) -> NextAction {
    NextAction::new(
        "runs.watch",
        "Watch run",
        argv(["runs", "watch", run_id]),
        true,
    )
    .primary()
    .with_description("Wait until the run reaches a terminal state.")
}

pub fn simulations_for_run(run_id: &str) -> NextAction {
    NextAction::new(
        "simulated-conversations.list_for_run",
        "List run simulated conversations",
        argv(["simulated-conversations", "list", "--run-id", run_id]),
        true,
    )
}

pub fn runs_for_agent(agent_id: &str) -> NextAction {
    NextAction::new(
        "runs.list_for_agent",
        "List agent runs",
        argv([
            "runs",
            "list",
            "--filter",
            &format!("agent_id=\"{agent_id}\""),
        ]),
        true,
    )
}

pub fn mutations_for_agent(agent_id: &str) -> NextAction {
    NextAction::new(
        "mutations.list_for_agent",
        "List agent mutations",
        argv(["mutations", "list", "--agent-id", agent_id]),
        true,
    )
}

pub fn mutation_get(agent_id: &str, mutation_id: &str) -> NextAction {
    NextAction::new(
        "mutations.get",
        "Get mutation",
        argv(["mutations", "get", "--agent-id", agent_id, mutation_id]),
        true,
    )
}

pub fn test_cases_for_set(test_set_id: &str) -> NextAction {
    NextAction::new(
        "test-cases.list_for_test_set",
        "List test cases",
        argv(["test-cases", "list", "--test-set-id", test_set_id]),
        true,
    )
}

pub fn metrics(resource: &str, id: &str) -> NextAction {
    NextAction::new(
        format!("{resource}.metrics"),
        format!("List {resource} metrics"),
        argv([resource, "metrics", id]),
        true,
    )
}

pub fn audio(resource: &str, id: &str) -> NextAction {
    NextAction::new(
        format!("{resource}.audio"),
        format!("Get {resource} audio URL"),
        argv([resource, "audio", id]),
        true,
    )
}

pub fn dashboard_widgets(dashboard_id: &str) -> NextAction {
    NextAction::new(
        "dashboards.widgets.list",
        "List dashboard widgets",
        argv(["dashboards", "widgets", "list", dashboard_id]),
        true,
    )
}

pub fn argv<I, S>(args: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    std::iter::once("coval")
        .chain(std::iter::once("--agent"))
        .map(str::to_string)
        .chain(args.into_iter().map(|arg| arg.as_ref().to_string()))
        .collect()
}
