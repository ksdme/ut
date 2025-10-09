use crate::tool::{Output, Tool};
use anyhow::Context;
use chrono::{DateTime, Utc};
use clap::{Command, CommandFactory, Parser};
use cron::Schedule;
use serde_json::json;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(
    name = "cron",
    about = "Parse crontab expression and show upcoming firing times"
)]
pub struct CronTool {
    /// Crontab expression (e.g., "0 9 * * 1-5" for weekdays at 9 AM, or "0 0 9 * * 1-5" for extended format)
    pub expression: String,

    /// Number of upcoming firing times to show (default: 5)
    #[arg(short = 'n', long = "count", default_value = "5")]
    pub count: usize,

    /// Start time for calculating next firing times (ISO 8601 format, defaults to now)
    #[arg(short = 's', long = "start")]
    pub start_time: Option<String>,
}

impl Tool for CronTool {
    fn cli() -> Command {
        CronTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        // Try to parse as-is first, then try adding seconds if it fails
        let schedule = Schedule::from_str(&self.expression)
            .or_else(|_| {
                // If parsing fails, try adding "0 " at the beginning for traditional 5-field format
                let extended_expr = format!("0 {}", self.expression);
                Schedule::from_str(&extended_expr)
            })
            .context(
                "Invalid crontab expression. Use format like '0 9 * * 1-5' or '0 0 9 * * 1-5'",
            )?;

        let start = parse_start_time(&self.start_time)?;
        let upcoming_times = get_upcoming_times(&schedule, start, self.count)?;

        let mut result_obj = serde_json::Map::new();
        result_obj.insert("expression".to_string(), json!(self.expression));

        // Add each upcoming time as a separate numbered field
        for (i, time) in upcoming_times.iter().enumerate() {
            result_obj.insert(format!("next_{}", i + 1), json!(time));
        }

        Ok(Some(Output::JsonValue(json!(result_obj))))
    }
}

fn parse_start_time(start_time: &Option<String>) -> anyhow::Result<DateTime<Utc>> {
    match start_time {
        Some(time_str) => Ok(DateTime::parse_from_rfc3339(time_str)
            .context("Invalid start time format. Use ISO 8601 format (e.g., 2024-01-01T00:00:00Z)")?
            .with_timezone(&Utc)),
        None => Ok(Utc::now()),
    }
}

fn get_upcoming_times(
    schedule: &Schedule,
    start: DateTime<Utc>,
    count: usize,
) -> anyhow::Result<Vec<String>> {
    let upcoming_times: Vec<String> = schedule
        .upcoming(Utc)
        .skip_while(|dt| *dt <= start)
        .take(count)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .collect();

    Ok(upcoming_times)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_cron() {
        let tool = CronTool {
            expression: "0 9 * * 1-5".to_string(),
            count: 3,
            start_time: None,
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        assert!(val["expression"].as_str().unwrap() == "0 9 * * 1-5");
        assert!(val["next_1"].as_str().is_some());
        assert!(val["next_2"].as_str().is_some());
        assert!(val["next_3"].as_str().is_some());
    }

    #[test]
    fn test_parse_daily_cron() {
        let tool = CronTool {
            expression: "0 0 * * *".to_string(),
            count: 2,
            start_time: None,
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        assert!(val["expression"].as_str().unwrap() == "0 0 * * *");
        assert!(val["next_1"].as_str().is_some());
        assert!(val["next_2"].as_str().is_some());
    }

    #[test]
    fn test_parse_hourly_cron() {
        let tool = CronTool {
            expression: "0 * * * *".to_string(),
            count: 5,
            start_time: None,
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        assert!(val["expression"].as_str().unwrap() == "0 * * * *");
        assert!(val["next_1"].as_str().is_some());
        assert!(val["next_2"].as_str().is_some());
        assert!(val["next_3"].as_str().is_some());
        assert!(val["next_4"].as_str().is_some());
        assert!(val["next_5"].as_str().is_some());
    }

    #[test]
    fn test_parse_with_start_time() {
        let tool = CronTool {
            expression: "0 9 * * 1-5".to_string(),
            count: 2,
            start_time: Some("2024-01-01T00:00:00Z".to_string()),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        assert!(val["expression"].as_str().unwrap() == "0 9 * * 1-5");
        assert!(val["next_1"].as_str().is_some());
        assert!(val["next_2"].as_str().is_some());
    }

    #[test]
    fn test_parse_invalid_expression() {
        let tool = CronTool {
            expression: "invalid".to_string(),
            count: 5,
            start_time: None,
        };
        let result = tool.execute();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_start_time() {
        let tool = CronTool {
            expression: "0 9 * * 1-5".to_string(),
            count: 5,
            start_time: Some("invalid-time".to_string()),
        };
        let result = tool.execute();

        assert!(result.is_err());
    }
}
