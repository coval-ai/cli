// @generated from the external OpenAPI spec runs-v1.yaml — DO NOT EDIT BY HAND.
// Regenerate with `cargo xtask` (see API-400). Presentation/behavior impls
// (Tabular, etc.) live in src/client/models/, not here.
#![allow(
    clippy::all,
    clippy::pedantic,
    dead_code,
    unused_imports,
    non_snake_case
)]

#[doc = r" Error types."]
pub mod error {
    #[doc = r" Error from a `TryFrom` or `FromStr` implementation."]
    pub struct ConversionError(::std::borrow::Cow<'static, str>);
    impl ::std::error::Error for ConversionError {}
    impl ::std::fmt::Display for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Display::fmt(&self.0, f)
        }
    }
    impl ::std::fmt::Debug for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Debug::fmt(&self.0, f)
        }
    }
    impl From<&'static str> for ConversionError {
        fn from(value: &'static str) -> Self {
            Self(value.into())
        }
    }
    impl From<String> for ConversionError {
        fn from(value: String) -> Self {
            Self(value.into())
        }
    }
}
#[doc = "`DeleteRunResponse`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"error\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"error\": {"]
#[doc = "      \"$ref\": \"#/components/schemas/Error\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct DeleteRunResponse {
    pub error: Error,
}
impl DeleteRunResponse {
    pub fn builder() -> builder::DeleteRunResponse {
        Default::default()
    }
}
#[doc = "`Error`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"code\","]
#[doc = "    \"message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"code\": {"]
#[doc = "      \"description\": \"Machine-readable error code\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"INVALID_ARGUMENT\","]
#[doc = "        \"UNAUTHENTICATED\","]
#[doc = "        \"PERMISSION_DENIED\","]
#[doc = "        \"NOT_FOUND\","]
#[doc = "        \"INTERNAL\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"details\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/components/schemas/ErrorDetail\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"message\": {"]
#[doc = "      \"description\": \"Human-readable error message\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct Error {
    #[doc = "Machine-readable error code"]
    pub code: ErrorCode,
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub details: ::std::vec::Vec<ErrorDetail>,
    #[doc = "Human-readable error message"]
    pub message: ::std::string::String,
}
impl Error {
    pub fn builder() -> builder::Error {
        Default::default()
    }
}
#[doc = "Machine-readable error code"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Machine-readable error code\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"INVALID_ARGUMENT\","]
#[doc = "    \"UNAUTHENTICATED\","]
#[doc = "    \"PERMISSION_DENIED\","]
#[doc = "    \"NOT_FOUND\","]
#[doc = "    \"INTERNAL\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum ErrorCode {
    #[serde(rename = "INVALID_ARGUMENT")]
    InvalidArgument,
    #[serde(rename = "UNAUTHENTICATED")]
    Unauthenticated,
    #[serde(rename = "PERMISSION_DENIED")]
    PermissionDenied,
    #[serde(rename = "NOT_FOUND")]
    NotFound,
    #[serde(rename = "INTERNAL")]
    Internal,
}
impl ::std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::InvalidArgument => f.write_str("INVALID_ARGUMENT"),
            Self::Unauthenticated => f.write_str("UNAUTHENTICATED"),
            Self::PermissionDenied => f.write_str("PERMISSION_DENIED"),
            Self::NotFound => f.write_str("NOT_FOUND"),
            Self::Internal => f.write_str("INTERNAL"),
        }
    }
}
impl ::std::str::FromStr for ErrorCode {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "INVALID_ARGUMENT" => Ok(Self::InvalidArgument),
            "UNAUTHENTICATED" => Ok(Self::Unauthenticated),
            "PERMISSION_DENIED" => Ok(Self::PermissionDenied),
            "NOT_FOUND" => Ok(Self::NotFound),
            "INTERNAL" => Ok(Self::Internal),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ErrorCode {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ErrorCode {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ErrorCode {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`ErrorDetail`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"description\": {"]
#[doc = "      \"description\": \"Human-readable description of the error\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"field\": {"]
#[doc = "      \"description\": \"The field that caused the error (if applicable)\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct ErrorDetail {
    #[doc = "Human-readable description of the error"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub description: ::std::option::Option<::std::string::String>,
    #[doc = "The field that caused the error (if applicable)"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub field: ::std::option::Option<::std::string::String>,
}
impl ::std::default::Default for ErrorDetail {
    fn default() -> Self {
        Self {
            description: Default::default(),
            field: Default::default(),
        }
    }
}
impl ErrorDetail {
    pub fn builder() -> builder::ErrorDetail {
        Default::default()
    }
}
#[doc = "`GetRunResponse`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"run\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"run\": {"]
#[doc = "      \"$ref\": \"#/components/schemas/Run\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct GetRunResponse {
    pub run: Run,
}
impl GetRunResponse {
    pub fn builder() -> builder::GetRunResponse {
        Default::default()
    }
}
#[doc = "Metadata for tracking and organization purposes."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Metadata for tracking and organization purposes.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"created_by\": {"]
#[doc = "      \"description\": \"Identifier for who/what created this simulation\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"customer\": {"]
#[doc = "      \"description\": \"Custom customer metadata for tracking and organization\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"additionalProperties\": true"]
#[doc = "    },"]
#[doc = "    \"display_name\": {"]
#[doc = "      \"description\": \"Human-readable name for the run. If not provided, a timestamp-based\\nname is auto-generated (e.g., \\\"Simulation Run 2025-10-14 12:00\\\").\\n\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"tags\": {"]
#[doc = "      \"description\": \"Tags for categorizing and filtering runs. Each tag max 200 characters,\\nmax 20 tags. Duplicate tags are automatically removed. Leading/trailing\\nwhitespace is stripped.\\n\","]
#[doc = "      \"type\": ["]
#[doc = "        \"array\","]
#[doc = "        \"null\""]
#[doc = "      ],"]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct LaunchMetadata {
    #[doc = "Identifier for who/what created this simulation"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub created_by: ::std::option::Option<::std::string::String>,
    #[doc = "Custom customer metadata for tracking and organization"]
    #[serde(default, skip_serializing_if = "::serde_json::Map::is_empty")]
    pub customer: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    #[doc = "Human-readable name for the run. If not provided, a timestamp-based\nname is auto-generated (e.g., \"Simulation Run 2025-10-14 12:00\").\n"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub display_name: ::std::option::Option<::std::string::String>,
    #[doc = "Tags for categorizing and filtering runs. Each tag max 200 characters,\nmax 20 tags. Duplicate tags are automatically removed. Leading/trailing\nwhitespace is stripped.\n"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub tags: ::std::option::Option<::std::vec::Vec<::std::string::String>>,
}
impl ::std::default::Default for LaunchMetadata {
    fn default() -> Self {
        Self {
            created_by: Default::default(),
            customer: Default::default(),
            display_name: Default::default(),
            tags: Default::default(),
        }
    }
}
impl LaunchMetadata {
    pub fn builder() -> builder::LaunchMetadata {
        Default::default()
    }
}
#[doc = "Execution options for launching simulations."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Execution options for launching simulations.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"concurrency\": {"]
#[doc = "      \"description\": \"Number of simulations to run concurrently.\","]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"iteration_count\": {"]
#[doc = "      \"description\": \"Number of times to run each test case.\","]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"sub_sample_seed\": {"]
#[doc = "      \"description\": \"Random seed for reproducible sub-sampling. Providing the same seed\\nwith the same sub_sample_size will select the same test cases.\\nOnly used when sub_sample_size > 0. If null, a random seed is used.\\nValid range: 0 to 2,147,483,647 (2^31 - 1).\\n\","]
#[doc = "      \"type\": ["]
#[doc = "        \"integer\","]
#[doc = "        \"null\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"sub_sample_size\": {"]
#[doc = "      \"description\": \"Number of test cases to randomly sample from the test set.\\nSet to 0 (default) to use all test cases. When set to a positive value,\\na random subset of that size will be selected for the run.\\n\","]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"test_case_ids\": {"]
#[doc = "      \"description\": \"Specific test case IDs to run. When provided, only these test cases\\nwill be executed. All IDs must belong to the specified test set and\\nbe active. Mutually exclusive with `sub_sample_size`.\\n\","]
#[doc = "      \"type\": ["]
#[doc = "        \"array\","]
#[doc = "        \"null\""]
#[doc = "      ],"]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct LaunchOptions {
    #[doc = "Number of simulations to run concurrently."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub concurrency: ::std::option::Option<i64>,
    #[doc = "Number of times to run each test case."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub iteration_count: ::std::option::Option<i64>,
    #[doc = "Random seed for reproducible sub-sampling. Providing the same seed\nwith the same sub_sample_size will select the same test cases.\nOnly used when sub_sample_size > 0. If null, a random seed is used.\nValid range: 0 to 2,147,483,647 (2^31 - 1).\n"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub sub_sample_seed: ::std::option::Option<i64>,
    #[doc = "Number of test cases to randomly sample from the test set.\nSet to 0 (default) to use all test cases. When set to a positive value,\na random subset of that size will be selected for the run.\n"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub sub_sample_size: ::std::option::Option<i64>,
    #[doc = "Specific test case IDs to run. When provided, only these test cases\nwill be executed. All IDs must belong to the specified test set and\nbe active. Mutually exclusive with `sub_sample_size`.\n"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub test_case_ids: ::std::option::Option<::std::vec::Vec<::std::string::String>>,
}
impl ::std::default::Default for LaunchOptions {
    fn default() -> Self {
        Self {
            concurrency: Default::default(),
            iteration_count: Default::default(),
            sub_sample_seed: Default::default(),
            sub_sample_size: Default::default(),
            test_case_ids: Default::default(),
        }
    }
}
impl LaunchOptions {
    pub fn builder() -> builder::LaunchOptions {
        Default::default()
    }
}
#[doc = "`LaunchRunRequest`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"agent_id\","]
#[doc = "    \"persona_id\","]
#[doc = "    \"test_set_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"agent_id\": {"]
#[doc = "      \"description\": \"The agent to test. Must be owned by the authenticated organization.\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"$ref\": \"#/components/schemas/LaunchMetadata\""]
#[doc = "    },"]
#[doc = "    \"metric_ids\": {"]
#[doc = "      \"description\": \"Optional list of metric IDs to evaluate. If not provided, uses agent's default metrics.\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"mutation_id\": {"]
#[doc = "      \"description\": \"Single mutation ID to run in addition to the base agent.\\nMutually exclusive with `mutation_ids`. The base agent always runs.\\n\","]
#[doc = "      \"type\": ["]
#[doc = "        \"string\","]
#[doc = "        \"null\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"mutation_ids\": {"]
#[doc = "      \"description\": \"List of mutation IDs to run in addition to the base agent.\\nMutually exclusive with `mutation_id`. Max 100 mutations.\\nThe base agent always runs alongside all mutations.\\n\\n**Total simulations** = test_cases × iterations × (1 + len(mutation_ids))\\n\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"options\": {"]
#[doc = "      \"$ref\": \"#/components/schemas/LaunchOptions\""]
#[doc = "    },"]
#[doc = "    \"persona_id\": {"]
#[doc = "      \"description\": \"The simulated persona to use for testing.\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"persona_metrics\": {"]
#[doc = "      \"description\": \"List of metric names that should evaluate the persona instead of the agent.\\nEach entry should be the base metric name (e.g., 'latency', not 'persona:latency').\\n\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"test_set_id\": {"]
#[doc = "      \"description\": \"The test set containing test cases to run.\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct LaunchRunRequest {
    #[doc = "The agent to test. Must be owned by the authenticated organization."]
    pub agent_id: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub metadata: ::std::option::Option<LaunchMetadata>,
    #[doc = "Optional list of metric IDs to evaluate. If not provided, uses agent's default metrics."]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub metric_ids: ::std::vec::Vec<::std::string::String>,
    #[doc = "Single mutation ID to run in addition to the base agent.\nMutually exclusive with `mutation_ids`. The base agent always runs.\n"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub mutation_id: ::std::option::Option<::std::string::String>,
    #[doc = "List of mutation IDs to run in addition to the base agent.\nMutually exclusive with `mutation_id`. Max 100 mutations.\nThe base agent always runs alongside all mutations.\n\n**Total simulations** = test_cases × iterations × (1 + len(mutation_ids))\n"]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub mutation_ids: ::std::vec::Vec<::std::string::String>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub options: ::std::option::Option<LaunchOptions>,
    #[doc = "The simulated persona to use for testing."]
    pub persona_id: ::std::string::String,
    #[doc = "List of metric names that should evaluate the persona instead of the agent.\nEach entry should be the base metric name (e.g., 'latency', not 'persona:latency').\n"]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub persona_metrics: ::std::vec::Vec<::std::string::String>,
    #[doc = "The test set containing test cases to run."]
    pub test_set_id: ::std::string::String,
}
impl LaunchRunRequest {
    pub fn builder() -> builder::LaunchRunRequest {
        Default::default()
    }
}
#[doc = "`LaunchRunResponse`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"run\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"run\": {"]
#[doc = "      \"$ref\": \"#/components/schemas/Run\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct LaunchRunResponse {
    pub run: Run,
}
impl LaunchRunResponse {
    pub fn builder() -> builder::LaunchRunResponse {
        Default::default()
    }
}
#[doc = "`ListRunsResponse`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"runs\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"next_page_token\": {"]
#[doc = "      \"description\": \"Token for fetching the next page of results\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"runs\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/components/schemas/Run\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct ListRunsResponse {
    #[doc = "Token for fetching the next page of results"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub next_page_token: ::std::option::Option<::std::string::String>,
    pub runs: ::std::vec::Vec<Run>,
}
impl ListRunsResponse {
    pub fn builder() -> builder::ListRunsResponse {
        Default::default()
    }
}
#[doc = "Aggregated metric statistics"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Aggregated metric statistics\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"mean\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"max\": {"]
#[doc = "      \"description\": \"Maximum value\","]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"mean\": {"]
#[doc = "      \"description\": \"Mean value across all test cases\","]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    \"min\": {"]
#[doc = "      \"description\": \"Minimum value\","]
#[doc = "      \"type\": \"number\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct MetricSummary {
    #[doc = "Maximum value"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub max: ::std::option::Option<f64>,
    #[doc = "Mean value across all test cases"]
    pub mean: f64,
    #[doc = "Minimum value"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub min: ::std::option::Option<f64>,
}
impl MetricSummary {
    pub fn builder() -> builder::MetricSummary {
        Default::default()
    }
}
#[doc = "Progress information for the run"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Progress information for the run\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"completed_test_cases\","]
#[doc = "    \"failed_test_cases\","]
#[doc = "    \"in_progress_test_cases\","]
#[doc = "    \"total_test_cases\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"completed_test_cases\": {"]
#[doc = "      \"description\": \"Number of completed test cases\","]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"failed_test_cases\": {"]
#[doc = "      \"description\": \"Number of failed test cases\","]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"in_progress_test_cases\": {"]
#[doc = "      \"description\": \"Number of test cases currently running\","]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"total_test_cases\": {"]
#[doc = "      \"description\": \"Total number of test cases\","]
#[doc = "      \"type\": \"integer\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct Progress {
    #[doc = "Number of completed test cases"]
    pub completed_test_cases: i64,
    #[doc = "Number of failed test cases"]
    pub failed_test_cases: i64,
    #[doc = "Number of test cases currently running"]
    pub in_progress_test_cases: i64,
    #[doc = "Total number of test cases"]
    pub total_test_cases: i64,
}
impl Progress {
    pub fn builder() -> builder::Progress {
        Default::default()
    }
}
#[doc = "Results summary (only for COMPLETED runs)"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Results summary (only for COMPLETED runs)\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"metrics\","]
#[doc = "    \"output_ids\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"metrics\": {"]
#[doc = "      \"description\": \"Aggregated metric results\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"additionalProperties\": {"]
#[doc = "        \"$ref\": \"#/components/schemas/MetricSummary\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"output_ids\": {"]
#[doc = "      \"description\": \"IDs of simulation outputs (test case results)\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct Results {
    #[doc = "Aggregated metric results"]
    pub metrics: ::std::collections::HashMap<::std::string::String, MetricSummary>,
    #[doc = "IDs of simulation outputs (test case results)"]
    pub output_ids: ::std::vec::Vec<::std::string::String>,
}
impl Results {
    pub fn builder() -> builder::Results {
        Default::default()
    }
}
#[doc = "`Run`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"create_time\","]
#[doc = "    \"name\","]
#[doc = "    \"run_id\","]
#[doc = "    \"status\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"agent_id\": {"]
#[doc = "      \"description\": \"ID of the agent being tested\","]
#[doc = "      \"type\": ["]
#[doc = "        \"string\","]
#[doc = "        \"null\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"create_time\": {"]
#[doc = "      \"description\": \"Timestamp when the run was created (ISO 8601)\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"date-time\""]
#[doc = "    },"]
#[doc = "    \"display_name\": {"]
#[doc = "      \"description\": \"Human-readable name for the run, set via `metadata.display_name` at launch.\","]
#[doc = "      \"type\": ["]
#[doc = "        \"string\","]
#[doc = "        \"null\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"error\": {"]
#[doc = "      \"description\": \"Error message if status is FAILED\","]
#[doc = "      \"type\": ["]
#[doc = "        \"string\","]
#[doc = "        \"null\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"description\": \"Custom metadata provided during launch\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"additionalProperties\": true"]
#[doc = "    },"]
#[doc = "    \"name\": {"]
#[doc = "      \"description\": \"Resource name in format \\\"runs/{run_id}\\\"\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"persona_id\": {"]
#[doc = "      \"description\": \"ID of the simulated persona used\","]
#[doc = "      \"type\": ["]
#[doc = "        \"string\","]
#[doc = "        \"null\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"progress\": {"]
#[doc = "      \"$ref\": \"#/components/schemas/Progress\""]
#[doc = "    },"]
#[doc = "    \"results\": {"]
#[doc = "      \"$ref\": \"#/components/schemas/Results\""]
#[doc = "    },"]
#[doc = "    \"run_id\": {"]
#[doc = "      \"description\": \"Unique identifier for this simulation run\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"status\": {"]
#[doc = "      \"description\": \"Current status of the simulation run\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"PENDING\","]
#[doc = "        \"IN QUEUE\","]
#[doc = "        \"IN PROGRESS\","]
#[doc = "        \"COMPLETED\","]
#[doc = "        \"FAILED\","]
#[doc = "        \"CANCELLED\","]
#[doc = "        \"DELETED\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"tags\": {"]
#[doc = "      \"description\": \"Tags for categorizing and filtering runs\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"test_set_id\": {"]
#[doc = "      \"description\": \"ID of the test set containing test cases\","]
#[doc = "      \"type\": ["]
#[doc = "        \"string\","]
#[doc = "        \"null\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"update_time\": {"]
#[doc = "      \"description\": \"Timestamp when the run was last updated (ISO 8601)\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"date-time\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct Run {
    #[doc = "ID of the agent being tested"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub agent_id: ::std::option::Option<::std::string::String>,
    #[doc = "Timestamp when the run was created (ISO 8601)"]
    pub create_time: ::chrono::DateTime<::chrono::offset::Utc>,
    #[doc = "Human-readable name for the run, set via `metadata.display_name` at launch."]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub display_name: ::std::option::Option<::std::string::String>,
    #[doc = "Error message if status is FAILED"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub error: ::std::option::Option<::std::string::String>,
    #[doc = "Custom metadata provided during launch"]
    #[serde(default, skip_serializing_if = "::serde_json::Map::is_empty")]
    pub metadata: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    #[doc = "Resource name in format \"runs/{run_id}\""]
    pub name: ::std::string::String,
    #[doc = "ID of the simulated persona used"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub persona_id: ::std::option::Option<::std::string::String>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub progress: ::std::option::Option<Progress>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub results: ::std::option::Option<Results>,
    #[doc = "Unique identifier for this simulation run"]
    pub run_id: ::std::string::String,
    #[doc = "Current status of the simulation run"]
    pub status: RunStatus,
    #[doc = "Tags for categorizing and filtering runs"]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub tags: ::std::vec::Vec<::std::string::String>,
    #[doc = "ID of the test set containing test cases"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub test_set_id: ::std::option::Option<::std::string::String>,
    #[doc = "Timestamp when the run was last updated (ISO 8601)"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub update_time: ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
}
impl Run {
    pub fn builder() -> builder::Run {
        Default::default()
    }
}
#[doc = "Current status of the simulation run"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Current status of the simulation run\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"PENDING\","]
#[doc = "    \"IN QUEUE\","]
#[doc = "    \"IN PROGRESS\","]
#[doc = "    \"COMPLETED\","]
#[doc = "    \"FAILED\","]
#[doc = "    \"CANCELLED\","]
#[doc = "    \"DELETED\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum RunStatus {
    #[serde(rename = "PENDING")]
    Pending,
    #[serde(rename = "IN QUEUE")]
    InQueue,
    #[serde(rename = "IN PROGRESS")]
    InProgress,
    #[serde(rename = "COMPLETED")]
    Completed,
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "CANCELLED")]
    Cancelled,
    #[serde(rename = "DELETED")]
    Deleted,
}
impl ::std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Pending => f.write_str("PENDING"),
            Self::InQueue => f.write_str("IN QUEUE"),
            Self::InProgress => f.write_str("IN PROGRESS"),
            Self::Completed => f.write_str("COMPLETED"),
            Self::Failed => f.write_str("FAILED"),
            Self::Cancelled => f.write_str("CANCELLED"),
            Self::Deleted => f.write_str("DELETED"),
        }
    }
}
impl ::std::str::FromStr for RunStatus {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "PENDING" => Ok(Self::Pending),
            "IN QUEUE" => Ok(Self::InQueue),
            "IN PROGRESS" => Ok(Self::InProgress),
            "COMPLETED" => Ok(Self::Completed),
            "FAILED" => Ok(Self::Failed),
            "CANCELLED" => Ok(Self::Cancelled),
            "DELETED" => Ok(Self::Deleted),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for RunStatus {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RunStatus {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RunStatus {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`UpdateRunRequest`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"tags\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"tags\": {"]
#[doc = "      \"description\": \"Full replacement list of run tags. Duplicate tags are automatically\\nremoved. Leading/trailing whitespace is stripped. Provide an empty\\nlist to clear all tags.\\n\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct UpdateRunRequest {
    #[doc = "Full replacement list of run tags. Duplicate tags are automatically\nremoved. Leading/trailing whitespace is stripped. Provide an empty\nlist to clear all tags.\n"]
    pub tags: ::std::vec::Vec<::std::string::String>,
}
impl UpdateRunRequest {
    pub fn builder() -> builder::UpdateRunRequest {
        Default::default()
    }
}
#[doc = "`UpdateRunResponse`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"run\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"run\": {"]
#[doc = "      \"$ref\": \"#/components/schemas/Run\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct UpdateRunResponse {
    pub run: Run,
}
impl UpdateRunResponse {
    pub fn builder() -> builder::UpdateRunResponse {
        Default::default()
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct DeleteRunResponse {
        error: ::std::result::Result<super::Error, ::std::string::String>,
    }
    impl ::std::default::Default for DeleteRunResponse {
        fn default() -> Self {
            Self {
                error: Err("no value supplied for error".to_string()),
            }
        }
    }
    impl DeleteRunResponse {
        pub fn error<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Error>,
            T::Error: ::std::fmt::Display,
        {
            self.error = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for error: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<DeleteRunResponse> for super::DeleteRunResponse {
        type Error = super::error::ConversionError;
        fn try_from(
            value: DeleteRunResponse,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                error: value.error?,
            })
        }
    }
    impl ::std::convert::From<super::DeleteRunResponse> for DeleteRunResponse {
        fn from(value: super::DeleteRunResponse) -> Self {
            Self {
                error: Ok(value.error),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Error {
        code: ::std::result::Result<super::ErrorCode, ::std::string::String>,
        details: ::std::result::Result<::std::vec::Vec<super::ErrorDetail>, ::std::string::String>,
        message: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for Error {
        fn default() -> Self {
            Self {
                code: Err("no value supplied for code".to_string()),
                details: Ok(Default::default()),
                message: Err("no value supplied for message".to_string()),
            }
        }
    }
    impl Error {
        pub fn code<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ErrorCode>,
            T::Error: ::std::fmt::Display,
        {
            self.code = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for code: {e}"));
            self
        }
        pub fn details<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::ErrorDetail>>,
            T::Error: ::std::fmt::Display,
        {
            self.details = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for details: {e}"));
            self
        }
        pub fn message<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.message = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for message: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Error> for super::Error {
        type Error = super::error::ConversionError;
        fn try_from(value: Error) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                code: value.code?,
                details: value.details?,
                message: value.message?,
            })
        }
    }
    impl ::std::convert::From<super::Error> for Error {
        fn from(value: super::Error) -> Self {
            Self {
                code: Ok(value.code),
                details: Ok(value.details),
                message: Ok(value.message),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ErrorDetail {
        description: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        field: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for ErrorDetail {
        fn default() -> Self {
            Self {
                description: Ok(Default::default()),
                field: Ok(Default::default()),
            }
        }
    }
    impl ErrorDetail {
        pub fn description<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.description = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for description: {e}"));
            self
        }
        pub fn field<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.field = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for field: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ErrorDetail> for super::ErrorDetail {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ErrorDetail,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                description: value.description?,
                field: value.field?,
            })
        }
    }
    impl ::std::convert::From<super::ErrorDetail> for ErrorDetail {
        fn from(value: super::ErrorDetail) -> Self {
            Self {
                description: Ok(value.description),
                field: Ok(value.field),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct GetRunResponse {
        run: ::std::result::Result<super::Run, ::std::string::String>,
    }
    impl ::std::default::Default for GetRunResponse {
        fn default() -> Self {
            Self {
                run: Err("no value supplied for run".to_string()),
            }
        }
    }
    impl GetRunResponse {
        pub fn run<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Run>,
            T::Error: ::std::fmt::Display,
        {
            self.run = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for run: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<GetRunResponse> for super::GetRunResponse {
        type Error = super::error::ConversionError;
        fn try_from(
            value: GetRunResponse,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self { run: value.run? })
        }
    }
    impl ::std::convert::From<super::GetRunResponse> for GetRunResponse {
        fn from(value: super::GetRunResponse) -> Self {
            Self { run: Ok(value.run) }
        }
    }
    #[derive(Clone, Debug)]
    pub struct LaunchMetadata {
        created_by: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        customer: ::std::result::Result<
            ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            ::std::string::String,
        >,
        display_name: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        tags: ::std::result::Result<
            ::std::option::Option<::std::vec::Vec<::std::string::String>>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for LaunchMetadata {
        fn default() -> Self {
            Self {
                created_by: Ok(Default::default()),
                customer: Ok(Default::default()),
                display_name: Ok(Default::default()),
                tags: Ok(Default::default()),
            }
        }
    }
    impl LaunchMetadata {
        pub fn created_by<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.created_by = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for created_by: {e}"));
            self
        }
        pub fn customer<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.customer = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for customer: {e}"));
            self
        }
        pub fn display_name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.display_name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for display_name: {e}"));
            self
        }
        pub fn tags<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::std::option::Option<::std::vec::Vec<::std::string::String>>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.tags = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for tags: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<LaunchMetadata> for super::LaunchMetadata {
        type Error = super::error::ConversionError;
        fn try_from(
            value: LaunchMetadata,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                created_by: value.created_by?,
                customer: value.customer?,
                display_name: value.display_name?,
                tags: value.tags?,
            })
        }
    }
    impl ::std::convert::From<super::LaunchMetadata> for LaunchMetadata {
        fn from(value: super::LaunchMetadata) -> Self {
            Self {
                created_by: Ok(value.created_by),
                customer: Ok(value.customer),
                display_name: Ok(value.display_name),
                tags: Ok(value.tags),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct LaunchOptions {
        concurrency: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
        iteration_count: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
        sub_sample_seed: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
        sub_sample_size: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
        test_case_ids: ::std::result::Result<
            ::std::option::Option<::std::vec::Vec<::std::string::String>>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for LaunchOptions {
        fn default() -> Self {
            Self {
                concurrency: Ok(Default::default()),
                iteration_count: Ok(Default::default()),
                sub_sample_seed: Ok(Default::default()),
                sub_sample_size: Ok(Default::default()),
                test_case_ids: Ok(Default::default()),
            }
        }
    }
    impl LaunchOptions {
        pub fn concurrency<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.concurrency = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for concurrency: {e}"));
            self
        }
        pub fn iteration_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.iteration_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for iteration_count: {e}"));
            self
        }
        pub fn sub_sample_seed<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.sub_sample_seed = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sub_sample_seed: {e}"));
            self
        }
        pub fn sub_sample_size<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.sub_sample_size = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sub_sample_size: {e}"));
            self
        }
        pub fn test_case_ids<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::std::option::Option<::std::vec::Vec<::std::string::String>>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.test_case_ids = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for test_case_ids: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<LaunchOptions> for super::LaunchOptions {
        type Error = super::error::ConversionError;
        fn try_from(
            value: LaunchOptions,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                concurrency: value.concurrency?,
                iteration_count: value.iteration_count?,
                sub_sample_seed: value.sub_sample_seed?,
                sub_sample_size: value.sub_sample_size?,
                test_case_ids: value.test_case_ids?,
            })
        }
    }
    impl ::std::convert::From<super::LaunchOptions> for LaunchOptions {
        fn from(value: super::LaunchOptions) -> Self {
            Self {
                concurrency: Ok(value.concurrency),
                iteration_count: Ok(value.iteration_count),
                sub_sample_seed: Ok(value.sub_sample_seed),
                sub_sample_size: Ok(value.sub_sample_size),
                test_case_ids: Ok(value.test_case_ids),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct LaunchRunRequest {
        agent_id: ::std::result::Result<::std::string::String, ::std::string::String>,
        metadata: ::std::result::Result<
            ::std::option::Option<super::LaunchMetadata>,
            ::std::string::String,
        >,
        metric_ids:
            ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
        mutation_id: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        mutation_ids:
            ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
        options: ::std::result::Result<
            ::std::option::Option<super::LaunchOptions>,
            ::std::string::String,
        >,
        persona_id: ::std::result::Result<::std::string::String, ::std::string::String>,
        persona_metrics:
            ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
        test_set_id: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for LaunchRunRequest {
        fn default() -> Self {
            Self {
                agent_id: Err("no value supplied for agent_id".to_string()),
                metadata: Ok(Default::default()),
                metric_ids: Ok(Default::default()),
                mutation_id: Ok(Default::default()),
                mutation_ids: Ok(Default::default()),
                options: Ok(Default::default()),
                persona_id: Err("no value supplied for persona_id".to_string()),
                persona_metrics: Ok(Default::default()),
                test_set_id: Err("no value supplied for test_set_id".to_string()),
            }
        }
    }
    impl LaunchRunRequest {
        pub fn agent_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.agent_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for agent_id: {e}"));
            self
        }
        pub fn metadata<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::LaunchMetadata>>,
            T::Error: ::std::fmt::Display,
        {
            self.metadata = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for metadata: {e}"));
            self
        }
        pub fn metric_ids<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.metric_ids = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for metric_ids: {e}"));
            self
        }
        pub fn mutation_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.mutation_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for mutation_id: {e}"));
            self
        }
        pub fn mutation_ids<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.mutation_ids = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for mutation_ids: {e}"));
            self
        }
        pub fn options<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::LaunchOptions>>,
            T::Error: ::std::fmt::Display,
        {
            self.options = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for options: {e}"));
            self
        }
        pub fn persona_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.persona_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for persona_id: {e}"));
            self
        }
        pub fn persona_metrics<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.persona_metrics = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for persona_metrics: {e}"));
            self
        }
        pub fn test_set_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.test_set_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for test_set_id: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<LaunchRunRequest> for super::LaunchRunRequest {
        type Error = super::error::ConversionError;
        fn try_from(
            value: LaunchRunRequest,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                agent_id: value.agent_id?,
                metadata: value.metadata?,
                metric_ids: value.metric_ids?,
                mutation_id: value.mutation_id?,
                mutation_ids: value.mutation_ids?,
                options: value.options?,
                persona_id: value.persona_id?,
                persona_metrics: value.persona_metrics?,
                test_set_id: value.test_set_id?,
            })
        }
    }
    impl ::std::convert::From<super::LaunchRunRequest> for LaunchRunRequest {
        fn from(value: super::LaunchRunRequest) -> Self {
            Self {
                agent_id: Ok(value.agent_id),
                metadata: Ok(value.metadata),
                metric_ids: Ok(value.metric_ids),
                mutation_id: Ok(value.mutation_id),
                mutation_ids: Ok(value.mutation_ids),
                options: Ok(value.options),
                persona_id: Ok(value.persona_id),
                persona_metrics: Ok(value.persona_metrics),
                test_set_id: Ok(value.test_set_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct LaunchRunResponse {
        run: ::std::result::Result<super::Run, ::std::string::String>,
    }
    impl ::std::default::Default for LaunchRunResponse {
        fn default() -> Self {
            Self {
                run: Err("no value supplied for run".to_string()),
            }
        }
    }
    impl LaunchRunResponse {
        pub fn run<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Run>,
            T::Error: ::std::fmt::Display,
        {
            self.run = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for run: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<LaunchRunResponse> for super::LaunchRunResponse {
        type Error = super::error::ConversionError;
        fn try_from(
            value: LaunchRunResponse,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self { run: value.run? })
        }
    }
    impl ::std::convert::From<super::LaunchRunResponse> for LaunchRunResponse {
        fn from(value: super::LaunchRunResponse) -> Self {
            Self { run: Ok(value.run) }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ListRunsResponse {
        next_page_token: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        runs: ::std::result::Result<::std::vec::Vec<super::Run>, ::std::string::String>,
    }
    impl ::std::default::Default for ListRunsResponse {
        fn default() -> Self {
            Self {
                next_page_token: Ok(Default::default()),
                runs: Err("no value supplied for runs".to_string()),
            }
        }
    }
    impl ListRunsResponse {
        pub fn next_page_token<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.next_page_token = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for next_page_token: {e}"));
            self
        }
        pub fn runs<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Run>>,
            T::Error: ::std::fmt::Display,
        {
            self.runs = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for runs: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<ListRunsResponse> for super::ListRunsResponse {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ListRunsResponse,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                next_page_token: value.next_page_token?,
                runs: value.runs?,
            })
        }
    }
    impl ::std::convert::From<super::ListRunsResponse> for ListRunsResponse {
        fn from(value: super::ListRunsResponse) -> Self {
            Self {
                next_page_token: Ok(value.next_page_token),
                runs: Ok(value.runs),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MetricSummary {
        max: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
        mean: ::std::result::Result<f64, ::std::string::String>,
        min: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
    }
    impl ::std::default::Default for MetricSummary {
        fn default() -> Self {
            Self {
                max: Ok(Default::default()),
                mean: Err("no value supplied for mean".to_string()),
                min: Ok(Default::default()),
            }
        }
    }
    impl MetricSummary {
        pub fn max<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.max = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for max: {e}"));
            self
        }
        pub fn mean<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.mean = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for mean: {e}"));
            self
        }
        pub fn min<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.min = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for min: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<MetricSummary> for super::MetricSummary {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MetricSummary,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                max: value.max?,
                mean: value.mean?,
                min: value.min?,
            })
        }
    }
    impl ::std::convert::From<super::MetricSummary> for MetricSummary {
        fn from(value: super::MetricSummary) -> Self {
            Self {
                max: Ok(value.max),
                mean: Ok(value.mean),
                min: Ok(value.min),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Progress {
        completed_test_cases: ::std::result::Result<i64, ::std::string::String>,
        failed_test_cases: ::std::result::Result<i64, ::std::string::String>,
        in_progress_test_cases: ::std::result::Result<i64, ::std::string::String>,
        total_test_cases: ::std::result::Result<i64, ::std::string::String>,
    }
    impl ::std::default::Default for Progress {
        fn default() -> Self {
            Self {
                completed_test_cases: Err("no value supplied for completed_test_cases".to_string()),
                failed_test_cases: Err("no value supplied for failed_test_cases".to_string()),
                in_progress_test_cases: Err(
                    "no value supplied for in_progress_test_cases".to_string()
                ),
                total_test_cases: Err("no value supplied for total_test_cases".to_string()),
            }
        }
    }
    impl Progress {
        pub fn completed_test_cases<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.completed_test_cases = value.try_into().map_err(|e| {
                format!("error converting supplied value for completed_test_cases: {e}")
            });
            self
        }
        pub fn failed_test_cases<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.failed_test_cases = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for failed_test_cases: {e}"));
            self
        }
        pub fn in_progress_test_cases<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.in_progress_test_cases = value.try_into().map_err(|e| {
                format!("error converting supplied value for in_progress_test_cases: {e}")
            });
            self
        }
        pub fn total_test_cases<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.total_test_cases = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for total_test_cases: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Progress> for super::Progress {
        type Error = super::error::ConversionError;
        fn try_from(value: Progress) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                completed_test_cases: value.completed_test_cases?,
                failed_test_cases: value.failed_test_cases?,
                in_progress_test_cases: value.in_progress_test_cases?,
                total_test_cases: value.total_test_cases?,
            })
        }
    }
    impl ::std::convert::From<super::Progress> for Progress {
        fn from(value: super::Progress) -> Self {
            Self {
                completed_test_cases: Ok(value.completed_test_cases),
                failed_test_cases: Ok(value.failed_test_cases),
                in_progress_test_cases: Ok(value.in_progress_test_cases),
                total_test_cases: Ok(value.total_test_cases),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Results {
        metrics: ::std::result::Result<
            ::std::collections::HashMap<::std::string::String, super::MetricSummary>,
            ::std::string::String,
        >,
        output_ids:
            ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
    }
    impl ::std::default::Default for Results {
        fn default() -> Self {
            Self {
                metrics: Err("no value supplied for metrics".to_string()),
                output_ids: Err("no value supplied for output_ids".to_string()),
            }
        }
    }
    impl Results {
        pub fn metrics<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::std::collections::HashMap<::std::string::String, super::MetricSummary>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.metrics = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for metrics: {e}"));
            self
        }
        pub fn output_ids<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.output_ids = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for output_ids: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Results> for super::Results {
        type Error = super::error::ConversionError;
        fn try_from(value: Results) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                metrics: value.metrics?,
                output_ids: value.output_ids?,
            })
        }
    }
    impl ::std::convert::From<super::Results> for Results {
        fn from(value: super::Results) -> Self {
            Self {
                metrics: Ok(value.metrics),
                output_ids: Ok(value.output_ids),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Run {
        agent_id: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        create_time:
            ::std::result::Result<::chrono::DateTime<::chrono::offset::Utc>, ::std::string::String>,
        display_name: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        error: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        metadata: ::std::result::Result<
            ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            ::std::string::String,
        >,
        name: ::std::result::Result<::std::string::String, ::std::string::String>,
        persona_id: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        progress:
            ::std::result::Result<::std::option::Option<super::Progress>, ::std::string::String>,
        results:
            ::std::result::Result<::std::option::Option<super::Results>, ::std::string::String>,
        run_id: ::std::result::Result<::std::string::String, ::std::string::String>,
        status: ::std::result::Result<super::RunStatus, ::std::string::String>,
        tags: ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
        test_set_id: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        update_time: ::std::result::Result<
            ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for Run {
        fn default() -> Self {
            Self {
                agent_id: Ok(Default::default()),
                create_time: Err("no value supplied for create_time".to_string()),
                display_name: Ok(Default::default()),
                error: Ok(Default::default()),
                metadata: Ok(Default::default()),
                name: Err("no value supplied for name".to_string()),
                persona_id: Ok(Default::default()),
                progress: Ok(Default::default()),
                results: Ok(Default::default()),
                run_id: Err("no value supplied for run_id".to_string()),
                status: Err("no value supplied for status".to_string()),
                tags: Ok(Default::default()),
                test_set_id: Ok(Default::default()),
                update_time: Ok(Default::default()),
            }
        }
    }
    impl Run {
        pub fn agent_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.agent_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for agent_id: {e}"));
            self
        }
        pub fn create_time<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
            T::Error: ::std::fmt::Display,
        {
            self.create_time = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for create_time: {e}"));
            self
        }
        pub fn display_name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.display_name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for display_name: {e}"));
            self
        }
        pub fn error<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.error = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for error: {e}"));
            self
        }
        pub fn metadata<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.metadata = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for metadata: {e}"));
            self
        }
        pub fn name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for name: {e}"));
            self
        }
        pub fn persona_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.persona_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for persona_id: {e}"));
            self
        }
        pub fn progress<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Progress>>,
            T::Error: ::std::fmt::Display,
        {
            self.progress = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for progress: {e}"));
            self
        }
        pub fn results<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Results>>,
            T::Error: ::std::fmt::Display,
        {
            self.results = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for results: {e}"));
            self
        }
        pub fn run_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.run_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for run_id: {e}"));
            self
        }
        pub fn status<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::RunStatus>,
            T::Error: ::std::fmt::Display,
        {
            self.status = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for status: {e}"));
            self
        }
        pub fn tags<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.tags = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for tags: {e}"));
            self
        }
        pub fn test_set_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.test_set_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for test_set_id: {e}"));
            self
        }
        pub fn update_time<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.update_time = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for update_time: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<Run> for super::Run {
        type Error = super::error::ConversionError;
        fn try_from(value: Run) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                agent_id: value.agent_id?,
                create_time: value.create_time?,
                display_name: value.display_name?,
                error: value.error?,
                metadata: value.metadata?,
                name: value.name?,
                persona_id: value.persona_id?,
                progress: value.progress?,
                results: value.results?,
                run_id: value.run_id?,
                status: value.status?,
                tags: value.tags?,
                test_set_id: value.test_set_id?,
                update_time: value.update_time?,
            })
        }
    }
    impl ::std::convert::From<super::Run> for Run {
        fn from(value: super::Run) -> Self {
            Self {
                agent_id: Ok(value.agent_id),
                create_time: Ok(value.create_time),
                display_name: Ok(value.display_name),
                error: Ok(value.error),
                metadata: Ok(value.metadata),
                name: Ok(value.name),
                persona_id: Ok(value.persona_id),
                progress: Ok(value.progress),
                results: Ok(value.results),
                run_id: Ok(value.run_id),
                status: Ok(value.status),
                tags: Ok(value.tags),
                test_set_id: Ok(value.test_set_id),
                update_time: Ok(value.update_time),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct UpdateRunRequest {
        tags: ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
    }
    impl ::std::default::Default for UpdateRunRequest {
        fn default() -> Self {
            Self {
                tags: Err("no value supplied for tags".to_string()),
            }
        }
    }
    impl UpdateRunRequest {
        pub fn tags<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.tags = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for tags: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<UpdateRunRequest> for super::UpdateRunRequest {
        type Error = super::error::ConversionError;
        fn try_from(
            value: UpdateRunRequest,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self { tags: value.tags? })
        }
    }
    impl ::std::convert::From<super::UpdateRunRequest> for UpdateRunRequest {
        fn from(value: super::UpdateRunRequest) -> Self {
            Self {
                tags: Ok(value.tags),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct UpdateRunResponse {
        run: ::std::result::Result<super::Run, ::std::string::String>,
    }
    impl ::std::default::Default for UpdateRunResponse {
        fn default() -> Self {
            Self {
                run: Err("no value supplied for run".to_string()),
            }
        }
    }
    impl UpdateRunResponse {
        pub fn run<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Run>,
            T::Error: ::std::fmt::Display,
        {
            self.run = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for run: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<UpdateRunResponse> for super::UpdateRunResponse {
        type Error = super::error::ConversionError;
        fn try_from(
            value: UpdateRunResponse,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self { run: value.run? })
        }
    }
    impl ::std::convert::From<super::UpdateRunResponse> for UpdateRunResponse {
        fn from(value: super::UpdateRunResponse) -> Self {
            Self { run: Ok(value.run) }
        }
    }
}
