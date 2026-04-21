pub mod error;
pub mod models;

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::Client;
use url::Url;

use self::error::ApiError;

const DEFAULT_BASE_URL: &str = "https://api.coval.dev";
const USER_AGENT: &str = concat!("coval-cli/", env!("CARGO_PKG_VERSION"));

pub struct CovalClient {
    http: Client,
    base_url: Url,
    api_key: String,
}

impl CovalClient {
    pub fn new(api_key: String, base_url: Option<&str>) -> Self {
        let base_url = base_url
            .and_then(|u| Url::parse(u).ok())
            .unwrap_or_else(|| Url::parse(DEFAULT_BASE_URL).unwrap());

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let http = Client::builder()
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http,
            base_url,
            api_key,
        }
    }

    pub fn url(&self, path: &str) -> Url {
        self.base_url.join(path).expect("Invalid URL path")
    }

    pub async fn get<T: serde::de::DeserializeOwned>(&self, url: Url) -> Result<T, ApiError> {
        let resp = self
            .http
            .get(url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        url: Url,
        body: &B,
    ) -> Result<T, ApiError> {
        let resp = self
            .http
            .post(url)
            .header("X-API-Key", &self.api_key)
            .json(body)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn patch<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        url: Url,
        body: &B,
    ) -> Result<T, ApiError> {
        let resp = self
            .http
            .patch(url)
            .header("X-API-Key", &self.api_key)
            .json(body)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn delete(&self, url: Url) -> Result<(), ApiError> {
        let resp = self
            .http
            .delete(url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(self.parse_error(resp).await)
        }
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T, ApiError> {
        if resp.status().is_success() {
            let body = resp.json().await?;
            Ok(body)
        } else {
            Err(self.parse_error(resp).await)
        }
    }

    async fn parse_error(&self, resp: reqwest::Response) -> ApiError {
        let status = resp.status();

        match resp.json::<models::ErrorResponse>().await {
            Ok(err) => ApiError::from_response(status, err),
            Err(_) => ApiError::Internal {
                message: format!("HTTP {status}"),
            },
        }
    }

    pub fn agents(&self) -> AgentsClient<'_> {
        AgentsClient(self)
    }

    pub fn runs(&self) -> RunsClient<'_> {
        RunsClient(self)
    }

    pub fn simulations(&self) -> SimulationsClient<'_> {
        SimulationsClient(self)
    }

    pub fn conversations(&self) -> ConversationsClient<'_> {
        ConversationsClient(self)
    }

    pub fn test_sets(&self) -> TestSetsClient<'_> {
        TestSetsClient(self)
    }

    pub fn test_cases(&self) -> TestCasesClient<'_> {
        TestCasesClient(self)
    }

    pub fn personas(&self) -> PersonasClient<'_> {
        PersonasClient(self)
    }

    pub fn metrics(&self) -> MetricsClient<'_> {
        MetricsClient(self)
    }

    pub fn mutations(&self, agent_id: &str) -> MutationsClient<'_> {
        MutationsClient {
            client: self,
            agent_id: agent_id.to_string(),
        }
    }

    pub fn api_keys(&self) -> ApiKeysClient<'_> {
        ApiKeysClient(self)
    }

    pub fn run_templates(&self) -> RunTemplatesClient<'_> {
        RunTemplatesClient(self)
    }

    pub fn scheduled_runs(&self) -> ScheduledRunsClient<'_> {
        ScheduledRunsClient(self)
    }

    pub fn dashboards(&self) -> DashboardsClient<'_> {
        DashboardsClient(self)
    }

    pub fn review_annotations(&self) -> ReviewAnnotationsClient<'_> {
        ReviewAnnotationsClient(self)
    }

    pub fn review_projects(&self) -> ReviewProjectsClient<'_> {
        ReviewProjectsClient(self)
    }

    pub fn widgets(&self, dashboard_id: &str) -> WidgetsClient<'_> {
        WidgetsClient {
            client: self,
            dashboard_id: dashboard_id.to_string(),
        }
    }
}

pub struct AgentsClient<'a>(&'a CovalClient);
pub struct ConversationsClient<'a>(&'a CovalClient);
pub struct RunsClient<'a>(&'a CovalClient);
pub struct SimulationsClient<'a>(&'a CovalClient);
pub struct TestSetsClient<'a>(&'a CovalClient);
pub struct TestCasesClient<'a>(&'a CovalClient);
pub struct PersonasClient<'a>(&'a CovalClient);
pub struct MetricsClient<'a>(&'a CovalClient);
pub struct MutationsClient<'a> {
    client: &'a CovalClient,
    agent_id: String,
}
pub struct ApiKeysClient<'a>(&'a CovalClient);
pub struct RunTemplatesClient<'a>(&'a CovalClient);
pub struct ScheduledRunsClient<'a>(&'a CovalClient);
pub struct DashboardsClient<'a>(&'a CovalClient);
pub struct ReviewAnnotationsClient<'a>(&'a CovalClient);
pub struct ReviewProjectsClient<'a>(&'a CovalClient);
pub struct WidgetsClient<'a> {
    client: &'a CovalClient,
    dashboard_id: String,
}

impl AgentsClient<'_> {
    pub async fn list(
        &self,
        params: models::ListParams,
    ) -> Result<models::ListAgentsResponse, ApiError> {
        let mut url = self.0.url("/v1/agents");
        params.apply_to(&mut url);
        self.0.get(url).await
    }

    pub async fn get(&self, id: &str) -> Result<models::Agent, ApiError> {
        let url = self.0.url(&format!("/v1/agents/{id}"));
        let resp: models::GetAgentResponse = self.0.get(url).await?;
        Ok(resp.agent)
    }

    pub async fn create(&self, req: models::CreateAgentRequest) -> Result<models::Agent, ApiError> {
        let url = self.0.url("/v1/agents");
        let resp: models::CreateAgentResponse = self.0.post(url, &req).await?;
        Ok(resp.agent)
    }

    pub async fn update(
        &self,
        id: &str,
        req: models::UpdateAgentRequest,
    ) -> Result<models::Agent, ApiError> {
        let url = self.0.url(&format!("/v1/agents/{id}"));
        let resp: models::UpdateAgentResponse = self.0.patch(url, &req).await?;
        Ok(resp.agent)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = self.0.url(&format!("/v1/agents/{id}"));
        self.0.delete(url).await
    }
}

impl RunsClient<'_> {
    pub async fn list(
        &self,
        params: models::ListParams,
    ) -> Result<models::ListRunsResponse, ApiError> {
        let mut url = self.0.url("/v1/runs");
        params.apply_to(&mut url);
        self.0.get(url).await
    }

    pub async fn get(&self, id: &str) -> Result<models::Run, ApiError> {
        let url = self.0.url(&format!("/v1/runs/{id}"));
        let resp: models::GetRunResponse = self.0.get(url).await?;
        Ok(resp.run)
    }

    pub async fn launch(&self, req: models::LaunchRunRequest) -> Result<models::Run, ApiError> {
        let url = self.0.url("/v1/runs");
        let resp: models::LaunchRunResponse = self.0.post(url, &req).await?;
        Ok(resp.run)
    }

    pub async fn update(
        &self,
        id: &str,
        req: models::UpdateRunRequest,
    ) -> Result<models::Run, ApiError> {
        let url = self.0.url(&format!("/v1/runs/{id}"));
        let resp: models::UpdateRunResponse = self.0.patch(url, &req).await?;
        Ok(resp.run)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = self.0.url(&format!("/v1/runs/{id}"));
        self.0.delete(url).await
    }
}

impl ConversationsClient<'_> {
    pub async fn list(
        &self,
        params: models::ListParams,
    ) -> Result<models::ListConversationsResponse, ApiError> {
        let mut url = self.0.url("/v1/conversations");
        params.apply_to(&mut url);
        self.0.get(url).await
    }

    pub async fn get(&self, id: &str) -> Result<models::Conversation, ApiError> {
        let url = self.0.url(&format!("/v1/conversations/{id}"));
        let resp: models::GetConversationResponse = self.0.get(url).await?;
        Ok(resp.conversation)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = self.0.url(&format!("/v1/conversations/{id}"));
        self.0.delete(url).await
    }

    pub async fn audio(&self, id: &str) -> Result<models::ConversationAudioUrlResponse, ApiError> {
        let url = self.0.url(&format!("/v1/conversations/{id}/audio"));
        self.0.get(url).await
    }

    pub async fn list_metrics(
        &self,
        id: &str,
    ) -> Result<models::ListConversationMetricsResponse, ApiError> {
        let url = self.0.url(&format!("/v1/conversations/{id}/metrics"));
        self.0.get(url).await
    }

    pub async fn get_metric(
        &self,
        id: &str,
        metric_output_id: &str,
    ) -> Result<models::SimpleMetricOutput, ApiError> {
        let url = self.0.url(&format!(
            "/v1/conversations/{id}/metrics/{metric_output_id}"
        ));
        let resp: models::GetConversationMetricResponse = self.0.get(url).await?;
        Ok(resp.metric)
    }
}

impl SimulationsClient<'_> {
    pub async fn list(
        &self,
        params: models::ListParams,
    ) -> Result<models::ListSimulationsResponse, ApiError> {
        let mut url = self.0.url("/v1/simulations");
        params.apply_to(&mut url);
        self.0.get(url).await
    }

    pub async fn get(&self, id: &str) -> Result<models::Simulation, ApiError> {
        let url = self.0.url(&format!("/v1/simulations/{id}"));
        let resp: models::GetSimulationResponse = self.0.get(url).await?;
        Ok(resp.simulation)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = self.0.url(&format!("/v1/simulations/{id}"));
        self.0.delete(url).await
    }

    pub async fn audio(&self, id: &str) -> Result<models::AudioUrlResponse, ApiError> {
        let url = self.0.url(&format!("/v1/simulations/{id}/audio"));
        self.0.get(url).await
    }

    pub async fn list_metrics(
        &self,
        id: &str,
    ) -> Result<models::ListSimulationMetricsResponse, ApiError> {
        let url = self.0.url(&format!("/v1/simulations/{id}/metrics"));
        self.0.get(url).await
    }

    pub async fn get_metric(
        &self,
        id: &str,
        metric_output_id: &str,
    ) -> Result<models::SimpleMetricOutput, ApiError> {
        let url = self
            .0
            .url(&format!("/v1/simulations/{id}/metrics/{metric_output_id}"));
        let resp: models::GetSimulationMetricResponse = self.0.get(url).await?;
        Ok(resp.metric)
    }
}

impl TestSetsClient<'_> {
    pub async fn list(
        &self,
        params: models::ListParams,
    ) -> Result<models::ListTestSetsResponse, ApiError> {
        let mut url = self.0.url("/v1/test-sets");
        params.apply_to(&mut url);
        self.0.get(url).await
    }

    pub async fn get(&self, id: &str) -> Result<models::TestSet, ApiError> {
        let url = self.0.url(&format!("/v1/test-sets/{id}"));
        let resp: models::GetTestSetResponse = self.0.get(url).await?;
        Ok(resp.test_set)
    }

    pub async fn create(
        &self,
        req: models::CreateTestSetRequest,
    ) -> Result<models::TestSet, ApiError> {
        let url = self.0.url("/v1/test-sets");
        let resp: models::CreateTestSetResponse = self.0.post(url, &req).await?;
        Ok(resp.test_set)
    }

    pub async fn update(
        &self,
        id: &str,
        req: models::UpdateTestSetRequest,
    ) -> Result<models::TestSet, ApiError> {
        let url = self.0.url(&format!("/v1/test-sets/{id}"));
        let resp: models::UpdateTestSetResponse = self.0.patch(url, &req).await?;
        Ok(resp.test_set)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = self.0.url(&format!("/v1/test-sets/{id}"));
        self.0.delete(url).await
    }
}

impl TestCasesClient<'_> {
    pub async fn list(
        &self,
        params: models::ListParams,
    ) -> Result<models::ListTestCasesResponse, ApiError> {
        let mut url = self.0.url("/v1/test-cases");
        params.apply_to(&mut url);
        self.0.get(url).await
    }

    pub async fn get(&self, id: &str) -> Result<models::TestCase, ApiError> {
        let url = self.0.url(&format!("/v1/test-cases/{id}"));
        let resp: models::GetTestCaseResponse = self.0.get(url).await?;
        Ok(resp.test_case)
    }

    pub async fn create(
        &self,
        req: models::CreateTestCaseRequest,
    ) -> Result<models::TestCase, ApiError> {
        let url = self.0.url("/v1/test-cases");
        let resp: models::CreateTestCaseResponse = self.0.post(url, &req).await?;
        Ok(resp.test_case)
    }

    pub async fn update(
        &self,
        id: &str,
        req: models::UpdateTestCaseRequest,
    ) -> Result<models::TestCase, ApiError> {
        let url = self.0.url(&format!("/v1/test-cases/{id}"));
        let resp: models::UpdateTestCaseResponse = self.0.patch(url, &req).await?;
        Ok(resp.test_case)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = self.0.url(&format!("/v1/test-cases/{id}"));
        self.0.delete(url).await
    }
}

impl PersonasClient<'_> {
    pub async fn list(
        &self,
        params: models::ListParams,
    ) -> Result<models::ListPersonasResponse, ApiError> {
        let mut url = self.0.url("/v1/personas");
        params.apply_to(&mut url);
        self.0.get(url).await
    }

    pub async fn get(&self, id: &str) -> Result<models::Persona, ApiError> {
        let url = self.0.url(&format!("/v1/personas/{id}"));
        let resp: models::GetPersonaResponse = self.0.get(url).await?;
        Ok(resp.persona)
    }

    pub async fn create(
        &self,
        req: models::CreatePersonaRequest,
    ) -> Result<models::Persona, ApiError> {
        let url = self.0.url("/v1/personas");
        let resp: models::CreatePersonaResponse = self.0.post(url, &req).await?;
        Ok(resp.persona)
    }

    pub async fn update(
        &self,
        id: &str,
        req: models::UpdatePersonaRequest,
    ) -> Result<models::Persona, ApiError> {
        let url = self.0.url(&format!("/v1/personas/{id}"));
        let resp: models::UpdatePersonaResponse = self.0.patch(url, &req).await?;
        Ok(resp.persona)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = self.0.url(&format!("/v1/personas/{id}"));
        self.0.delete(url).await
    }

    pub async fn list_phone_numbers(&self) -> Result<models::ListPhoneNumbersResponse, ApiError> {
        let url = self.0.url("/v1/personas/phone-numbers");
        self.0.get(url).await
    }
}

impl MetricsClient<'_> {
    pub async fn list(
        &self,
        params: models::ListParams,
        include_builtin: bool,
    ) -> Result<models::ListMetricsResponse, ApiError> {
        let mut url = self.0.url("/v1/metrics");
        params.apply_to(&mut url);
        if include_builtin {
            url.query_pairs_mut().append_pair("include_builtin", "true");
        }
        self.0.get(url).await
    }

    pub async fn get(&self, id: &str) -> Result<models::Metric, ApiError> {
        let url = self.0.url(&format!("/v1/metrics/{id}"));
        let resp: models::GetMetricResponse = self.0.get(url).await?;
        Ok(resp.metric)
    }

    pub async fn create(
        &self,
        req: models::CreateMetricRequest,
    ) -> Result<models::Metric, ApiError> {
        let url = self.0.url("/v1/metrics");
        let resp: models::CreateMetricResponse = self.0.post(url, &req).await?;
        Ok(resp.metric)
    }

    pub async fn update(
        &self,
        id: &str,
        req: models::UpdateMetricRequest,
    ) -> Result<models::Metric, ApiError> {
        let url = self.0.url(&format!("/v1/metrics/{id}"));
        let resp: models::UpdateMetricResponse = self.0.patch(url, &req).await?;
        Ok(resp.metric)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = self.0.url(&format!("/v1/metrics/{id}"));
        self.0.delete(url).await
    }
}

impl MutationsClient<'_> {
    pub async fn list(
        &self,
        params: models::ListParams,
    ) -> Result<models::ListMutationsResponse, ApiError> {
        let mut url = self
            .client
            .url(&format!("/v1/agents/{}/mutations", self.agent_id));
        params.apply_to(&mut url);
        self.client.get(url).await
    }

    pub async fn get(&self, id: &str) -> Result<models::Mutation, ApiError> {
        let url = self
            .client
            .url(&format!("/v1/agents/{}/mutations/{id}", self.agent_id));
        let resp: models::GetMutationResponse = self.client.get(url).await?;
        Ok(resp.mutation)
    }

    pub async fn create(
        &self,
        req: models::CreateMutationRequest,
    ) -> Result<models::Mutation, ApiError> {
        let url = self
            .client
            .url(&format!("/v1/agents/{}/mutations", self.agent_id));
        let resp: models::CreateMutationResponse = self.client.post(url, &req).await?;
        Ok(resp.mutation)
    }

    pub async fn update(
        &self,
        id: &str,
        req: models::UpdateMutationRequest,
    ) -> Result<models::Mutation, ApiError> {
        let url = self
            .client
            .url(&format!("/v1/agents/{}/mutations/{id}", self.agent_id));
        let resp: models::UpdateMutationResponse = self.client.patch(url, &req).await?;
        Ok(resp.mutation)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = self
            .client
            .url(&format!("/v1/agents/{}/mutations/{id}", self.agent_id));
        self.client.delete(url).await
    }
}

impl ApiKeysClient<'_> {
    pub async fn list(
        &self,
        params: models::ListParams,
        status: Option<models::ApiKeyStatus>,
        environment: Option<models::ApiKeyEnvironment>,
    ) -> Result<models::ListApiKeysResponse, ApiError> {
        let mut url = self.0.url("/v1/api-keys");
        params.apply_to(&mut url);
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(s) = status {
                pairs.append_pair("status", &s.to_string());
            }
            if let Some(e) = environment {
                pairs.append_pair("environment", &e.to_string());
            }
        }
        self.0.get(url).await
    }

    pub async fn create(
        &self,
        req: models::CreateApiKeyRequest,
    ) -> Result<models::ApiKey, ApiError> {
        let url = self.0.url("/v1/api-keys");
        let resp: models::CreateApiKeyResponse = self.0.post(url, &req).await?;
        Ok(resp.api_key)
    }

    pub async fn update(
        &self,
        id: &str,
        req: models::UpdateApiKeyRequest,
    ) -> Result<models::ApiKey, ApiError> {
        let url = self.0.url(&format!("/v1/api-keys/{id}"));
        let resp: models::UpdateApiKeyResponse = self.0.patch(url, &req).await?;
        Ok(resp.api_key)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = self.0.url(&format!("/v1/api-keys/{id}"));
        self.0.delete(url).await
    }
}

impl RunTemplatesClient<'_> {
    pub async fn list(
        &self,
        params: models::ListParams,
    ) -> Result<models::ListRunTemplatesResponse, ApiError> {
        let mut url = self.0.url("/v1/run-templates");
        params.apply_to(&mut url);
        self.0.get(url).await
    }

    pub async fn get(&self, id: &str) -> Result<models::RunTemplate, ApiError> {
        let url = self.0.url(&format!("/v1/run-templates/{id}"));
        let resp: models::GetRunTemplateResponse = self.0.get(url).await?;
        Ok(resp.run_template)
    }

    pub async fn create(
        &self,
        req: models::CreateRunTemplateRequest,
    ) -> Result<models::RunTemplate, ApiError> {
        let url = self.0.url("/v1/run-templates");
        let resp: models::CreateRunTemplateResponse = self.0.post(url, &req).await?;
        Ok(resp.run_template)
    }

    pub async fn update(
        &self,
        id: &str,
        req: models::UpdateRunTemplateRequest,
    ) -> Result<models::RunTemplate, ApiError> {
        let url = self.0.url(&format!("/v1/run-templates/{id}"));
        let resp: models::UpdateRunTemplateResponse = self.0.patch(url, &req).await?;
        Ok(resp.run_template)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = self.0.url(&format!("/v1/run-templates/{id}"));
        self.0.delete(url).await
    }
}

impl ScheduledRunsClient<'_> {
    pub async fn list(
        &self,
        params: models::ListParams,
        enabled: Option<bool>,
        template_id: Option<&str>,
    ) -> Result<models::ListScheduledRunsResponse, ApiError> {
        let mut url = self.0.url("/v1/scheduled-runs");
        params.apply_to(&mut url);
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(e) = enabled {
                pairs.append_pair("enabled", &e.to_string());
            }
            if let Some(tid) = template_id {
                pairs.append_pair("template_id", tid);
            }
        }
        self.0.get(url).await
    }

    pub async fn get(&self, id: &str) -> Result<models::ScheduledRun, ApiError> {
        let url = self.0.url(&format!("/v1/scheduled-runs/{id}"));
        let resp: models::GetScheduledRunResponse = self.0.get(url).await?;
        Ok(resp.scheduled_run)
    }

    pub async fn create(
        &self,
        req: models::CreateScheduledRunRequest,
    ) -> Result<models::ScheduledRun, ApiError> {
        let url = self.0.url("/v1/scheduled-runs");
        let resp: models::CreateScheduledRunResponse = self.0.post(url, &req).await?;
        Ok(resp.scheduled_run)
    }

    pub async fn update(
        &self,
        id: &str,
        req: models::UpdateScheduledRunRequest,
    ) -> Result<models::ScheduledRun, ApiError> {
        let url = self.0.url(&format!("/v1/scheduled-runs/{id}"));
        let resp: models::UpdateScheduledRunResponse = self.0.patch(url, &req).await?;
        Ok(resp.scheduled_run)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = self.0.url(&format!("/v1/scheduled-runs/{id}"));
        self.0.delete(url).await
    }
}

impl DashboardsClient<'_> {
    pub async fn list(
        &self,
        params: models::ListParams,
    ) -> Result<models::ListDashboardsResponse, ApiError> {
        let mut url = self.0.url("/v1/dashboards");
        params.apply_to(&mut url);
        self.0.get(url).await
    }

    pub async fn get(&self, id: &str) -> Result<models::Dashboard, ApiError> {
        let url = self.0.url(&format!("/v1/dashboards/{id}"));
        let resp: models::GetDashboardResponse = self.0.get(url).await?;
        Ok(resp.dashboard)
    }

    pub async fn create(
        &self,
        req: models::CreateDashboardRequest,
    ) -> Result<models::Dashboard, ApiError> {
        let url = self.0.url("/v1/dashboards");
        let resp: models::CreateDashboardResponse = self.0.post(url, &req).await?;
        Ok(resp.dashboard)
    }

    pub async fn update(
        &self,
        id: &str,
        req: models::UpdateDashboardRequest,
    ) -> Result<models::Dashboard, ApiError> {
        let url = self.0.url(&format!("/v1/dashboards/{id}"));
        let resp: models::UpdateDashboardResponse = self.0.patch(url, &req).await?;
        Ok(resp.dashboard)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = self.0.url(&format!("/v1/dashboards/{id}"));
        self.0.delete(url).await
    }
}

impl ReviewAnnotationsClient<'_> {
    pub async fn list(
        &self,
        params: models::ListParams,
    ) -> Result<models::ListReviewAnnotationsResponse, ApiError> {
        let mut url = self.0.url("/v1/review-annotations");
        params.apply_to(&mut url);
        self.0.get(url).await
    }

    pub async fn get(&self, id: &str) -> Result<models::ReviewAnnotation, ApiError> {
        let url = self.0.url(&format!("/v1/review-annotations/{id}"));
        let resp: models::GetReviewAnnotationResponse = self.0.get(url).await?;
        Ok(resp.review_annotation)
    }

    pub async fn create(
        &self,
        req: models::CreateReviewAnnotationRequest,
    ) -> Result<models::ReviewAnnotation, ApiError> {
        let url = self.0.url("/v1/review-annotations");
        let resp: models::CreateReviewAnnotationResponse = self.0.post(url, &req).await?;
        Ok(resp.review_annotation)
    }

    pub async fn update(
        &self,
        id: &str,
        req: models::UpdateReviewAnnotationRequest,
    ) -> Result<models::ReviewAnnotation, ApiError> {
        let url = self.0.url(&format!("/v1/review-annotations/{id}"));
        let resp: models::UpdateReviewAnnotationResponse = self.0.patch(url, &req).await?;
        Ok(resp.review_annotation)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = self.0.url(&format!("/v1/review-annotations/{id}"));
        self.0.delete(url).await
    }
}

impl ReviewProjectsClient<'_> {
    pub async fn list(
        &self,
        params: models::ListParams,
    ) -> Result<models::ListReviewProjectsResponse, ApiError> {
        let mut url = self.0.url("/v1/review-projects");
        params.apply_to(&mut url);
        self.0.get(url).await
    }

    pub async fn get(&self, id: &str) -> Result<models::ReviewProject, ApiError> {
        let url = self.0.url(&format!("/v1/review-projects/{id}"));
        let resp: models::GetReviewProjectResponse = self.0.get(url).await?;
        Ok(resp.review_project)
    }

    pub async fn create(
        &self,
        req: models::CreateReviewProjectRequest,
    ) -> Result<models::ReviewProject, ApiError> {
        let url = self.0.url("/v1/review-projects");
        let resp: models::CreateReviewProjectResponse = self.0.post(url, &req).await?;
        Ok(resp.review_project)
    }

    pub async fn update(
        &self,
        id: &str,
        req: models::UpdateReviewProjectRequest,
    ) -> Result<models::ReviewProject, ApiError> {
        let url = self.0.url(&format!("/v1/review-projects/{id}"));
        let resp: models::UpdateReviewProjectResponse = self.0.patch(url, &req).await?;
        Ok(resp.review_project)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = self.0.url(&format!("/v1/review-projects/{id}"));
        self.0.delete(url).await
    }
}

impl WidgetsClient<'_> {
    pub async fn list(
        &self,
        params: models::ListParams,
    ) -> Result<models::ListWidgetsResponse, ApiError> {
        let mut url = self
            .client
            .url(&format!("/v1/dashboards/{}/widgets", self.dashboard_id));
        params.apply_to(&mut url);
        self.client.get(url).await
    }

    pub async fn get(&self, id: &str) -> Result<models::Widget, ApiError> {
        let url = self.client.url(&format!(
            "/v1/dashboards/{}/widgets/{id}",
            self.dashboard_id
        ));
        let resp: models::GetWidgetResponse = self.client.get(url).await?;
        Ok(resp.widget)
    }

    pub async fn create(
        &self,
        req: models::CreateWidgetRequest,
    ) -> Result<models::Widget, ApiError> {
        let url = self
            .client
            .url(&format!("/v1/dashboards/{}/widgets", self.dashboard_id));
        let resp: models::CreateWidgetResponse = self.client.post(url, &req).await?;
        Ok(resp.widget)
    }

    pub async fn update(
        &self,
        id: &str,
        req: models::UpdateWidgetRequest,
    ) -> Result<models::Widget, ApiError> {
        let url = self.client.url(&format!(
            "/v1/dashboards/{}/widgets/{id}",
            self.dashboard_id
        ));
        let resp: models::UpdateWidgetResponse = self.client.patch(url, &req).await?;
        Ok(resp.widget)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = self.client.url(&format!(
            "/v1/dashboards/{}/widgets/{id}",
            self.dashboard_id
        ));
        self.client.delete(url).await
    }
}
