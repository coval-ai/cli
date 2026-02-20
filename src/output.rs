use clap::ValueEnum;
use serde::Serialize;
use tabled::settings::Style;

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
}

pub trait Tabular {
    fn headers() -> Vec<&'static str>;
    fn row(&self) -> Vec<String>;
}

pub fn print_list<T: Serialize + Tabular>(items: &[T], format: OutputFormat) {
    match format {
        OutputFormat::Table => {
            if items.is_empty() {
                println!("No results found.");
                return;
            }
            let rows: Vec<Vec<String>> = items.iter().map(Tabular::row).collect();
            let headers = T::headers();
            let mut builder = tabled::builder::Builder::new();
            builder.push_record(headers);
            for row in rows {
                builder.push_record(row);
            }
            let table = builder.build().with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(items).expect("Failed to serialize");
            println!("{json}");
        }
    }
}

pub fn print_one<T: Serialize>(item: &T, _format: OutputFormat) {
    let json = serde_json::to_string_pretty(item).expect("Failed to serialize");
    println!("{json}");
}

#[allow(dead_code)]
pub fn print_id(id: &str) {
    println!("{id}");
}

pub fn print_success(message: &str) {
    println!("{message}");
}
