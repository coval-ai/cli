use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorInfo,
}

#[derive(Debug, Deserialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub details: Vec<ErrorDetail>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorDetail {
    pub field: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct ListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<String>,
}

impl ListParams {
    pub fn apply_to(&self, url: &mut Url) {
        let mut pairs = url.query_pairs_mut();
        if let Some(ref filter) = self.filter {
            pairs.append_pair("filter", filter);
        }
        if let Some(size) = self.page_size {
            pairs.append_pair("page_size", &size.to_string());
        }
        if let Some(ref token) = self.page_token {
            pairs.append_pair("page_token", token);
        }
        if let Some(ref order) = self.order_by {
            pairs.append_pair("order_by", order);
        }
    }
}
