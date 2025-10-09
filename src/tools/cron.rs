use crate::tool::{Output, Tool};
use anyhow::Context;
use chrono::{DateTime, FixedOffset, Utc};
use clap::{Command, CommandFactory, Parser};
use cron::Schedule;
use serde_json::json;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(
    name = "cron",
    about = "Parse crontab expression and show upcoming firing times"
)]
/// TODO:
/// 1. Support --before
/// 2. Output in a different timezone
pub struct CronTool {
    /// Crontab expression (e.g., "0 9 * * 1-5" for weekdays at 9 AM, or "0 0 9 * * 1-5" for extended format)
    pub expression: String,

    /// Number of upcoming firing times to show (default: 5)
    #[arg(short = 'n', long = "count", default_value = "5")]
    pub count: usize,

    /// Calculate firing times after this time (ISO 8601 format, defaults to now)
    #[arg(short = 'a', long = "after")]
    pub after: Option<String>,
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

        let (after_utc, offset) = match &self.after {
            Some(time_str) => {
                let parsed = DateTime::parse_from_rfc3339(time_str).context(
                    "Invalid after time format. Use ISO 8601 format (e.g., 2024-01-01T00:00:00Z)",
                )?;
                let offset = parsed.timezone();
                (parsed.with_timezone(&Utc), offset)
            }
            None => {
                let now = Utc::now();
                let offset = FixedOffset::east_opt(0).unwrap(); // UTC has offset 0
                (now, offset)
            }
        };

        Ok(Some(Output::JsonValue(json!(get_upcoming_times(
            &schedule, after_utc, offset, self.count
        )?))))
    }
}

fn get_upcoming_times(
    schedule: &Schedule,
    after: DateTime<Utc>,
    offset: FixedOffset,
    count: usize,
) -> anyhow::Result<Vec<String>> {
    let upcoming_times: Vec<String> = schedule
        .after(&after)
        .take(count)
        .map(|dt| {
            let dt_with_offset = dt.with_timezone(&offset);
            dt_with_offset.to_rfc3339()
        })
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
            after: Some("2024-01-01T00:00:00Z".to_string()),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        let arr = val.as_array().unwrap();
        assert_eq!(arr.len(), 3);

        // 2024-01-01 was Monday, so next 3 weekdays at 9 AM are Jan 1, 2, 3
        assert_eq!(arr[0].as_str().unwrap(), "2024-01-01T09:00:00+00:00");
        assert_eq!(arr[1].as_str().unwrap(), "2024-01-02T09:00:00+00:00");
        assert_eq!(arr[2].as_str().unwrap(), "2024-01-03T09:00:00+00:00");
    }

    #[test]
    fn test_parse_daily_cron() {
        let tool = CronTool {
            expression: "0 0 * * *".to_string(),
            count: 2,
            after: Some("2024-01-01T00:00:00Z".to_string()),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        let arr = val.as_array().unwrap();
        assert_eq!(arr.len(), 2);

        // Daily at midnight, starting from 2024-01-01
        assert_eq!(arr[0].as_str().unwrap(), "2024-01-02T00:00:00+00:00");
        assert_eq!(arr[1].as_str().unwrap(), "2024-01-03T00:00:00+00:00");
    }

    #[test]
    fn test_parse_hourly_cron() {
        let tool = CronTool {
            expression: "0 * * * *".to_string(),
            count: 5,
            after: Some("2024-01-01T00:00:00Z".to_string()),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        let arr = val.as_array().unwrap();
        assert_eq!(arr.len(), 5);

        // Hourly starting from 2024-01-01 00:00:00
        assert_eq!(arr[0].as_str().unwrap(), "2024-01-01T01:00:00+00:00");
        assert_eq!(arr[1].as_str().unwrap(), "2024-01-01T02:00:00+00:00");
        assert_eq!(arr[2].as_str().unwrap(), "2024-01-01T03:00:00+00:00");
        assert_eq!(arr[3].as_str().unwrap(), "2024-01-01T04:00:00+00:00");
        assert_eq!(arr[4].as_str().unwrap(), "2024-01-01T05:00:00+00:00");
    }

    #[test]
    fn test_parse_with_after_time() {
        let tool = CronTool {
            expression: "0 9 * * 1-5".to_string(),
            count: 2,
            after: Some("2024-03-15T10:00:00Z".to_string()),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        let arr = val.as_array().unwrap();
        assert_eq!(arr.len(), 2);

        // 2024-03-15 is Friday at 10:00, next weekdays at 9 AM are Mar 17 (Sun), Mar 18 (Mon)
        assert_eq!(arr[0].as_str().unwrap(), "2024-03-17T09:00:00+00:00");
        assert_eq!(arr[1].as_str().unwrap(), "2024-03-18T09:00:00+00:00");
    }

    #[test]
    fn test_parse_invalid_expression() {
        let tool = CronTool {
            expression: "invalid".to_string(),
            count: 5,
            after: None,
        };
        let result = tool.execute();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_after_time() {
        let tool = CronTool {
            expression: "0 9 * * 1-5".to_string(),
            count: 5,
            after: Some("invalid-time".to_string()),
        };
        let result = tool.execute();

        assert!(result.is_err());
    }

    #[test]
    fn test_timezone_preserved() {
        let tool = CronTool {
            expression: "0 9 * * 1-5".to_string(),
            count: 2,
            after: Some("2024-01-01T00:00:00+05:30".to_string()),
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        let arr = val.as_array().unwrap();
        assert_eq!(arr.len(), 2);

        // Times should be in +05:30 timezone (IST)
        assert_eq!(arr[0].as_str().unwrap(), "2024-01-01T14:30:00+05:30");
        assert_eq!(arr[1].as_str().unwrap(), "2024-01-02T14:30:00+05:30");
    }
}
