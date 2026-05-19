use std::io::Read;

use anyhow::{Context, Result};
use clap::Args;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Args, Debug, Clone, Default)]
pub struct InputJsonArg {
    #[arg(long, value_name = "JSON|@FILE|-")]
    pub input_json: Option<String>,
}

impl InputJsonArg {
    pub fn object(&self) -> Result<Map<String, Value>> {
        let Some(raw) = self.input_json.as_deref() else {
            return Ok(Map::new());
        };

        let contents = if raw == "-" {
            let mut contents = String::new();
            std::io::stdin().read_to_string(&mut contents)?;
            contents
        } else if let Some(path) = raw.strip_prefix('@') {
            std::fs::read_to_string(path)
                .with_context(|| format!("failed to read --input-json file: {path}"))?
        } else {
            raw.to_string()
        };

        match serde_json::from_str::<Value>(&contents)? {
            Value::Object(map) => Ok(map),
            _ => anyhow::bail!("--input-json must contain a JSON object"),
        }
    }
}

pub fn insert<T: Serialize>(
    object: &mut Map<String, Value>,
    key: &'static str,
    value: Option<T>,
) -> Result<()> {
    if let Some(value) = value {
        object.insert(key.to_string(), serde_json::to_value(value)?);
    }
    Ok(())
}

pub fn finish<T: DeserializeOwned>(object: Map<String, Value>) -> Result<T> {
    Ok(serde_json::from_value(Value::Object(object))?)
}
