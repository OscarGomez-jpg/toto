use crate::adapters::tui::config::JiraConfig;
use crate::domain::task::{Task, TaskBuilder, TaskSource};
use base64::{engine::general_purpose, Engine as _};
use log::{debug, error, info};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use std::error::Error;
use uuid::Uuid;

pub struct JiraAdapter {
    config: JiraConfig,
    client: Client,
}

impl JiraAdapter {
    pub fn new(config: JiraConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub fn fetch_tasks(&self) -> Result<Vec<Task>, Box<dyn Error>> {
        if !self.config.enabled {
            debug!("Jira integration is disabled.");
            return Ok(vec![]);
        }
        if self.config.projects.is_empty() {
            debug!("No Jira projects configured.");
            return Ok(vec![]);
        }

        info!(
            "Fetching tasks from Jira for projects: {:?}",
            self.config.projects
        );
        let mut all_tasks = Vec::new();

        for project in &self.config.projects {
            debug!("Fetching Jira issues for project: {}", project);

            let mut jql = format!("project = '{}' AND status != 'Done'", project);
            if !self.config.labels.is_empty() {
                let labels_str = self
                    .config
                    .labels
                    .iter()
                    .map(|l| format!("'{}'", l))
                    .collect::<Vec<_>>()
                    .join(",");
                jql.push_str(&format!(" AND labels IN ({})", labels_str));
            }
            jql.push_str(" ORDER BY updated DESC");

            let url = format!("https://{}/rest/api/3/search/jql", self.config.domain);

            let mut headers = HeaderMap::new();
            let auth = format!("{}:{}", self.config.email, self.config.api_token);
            let auth_base64 = general_purpose::STANDARD.encode(auth);
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Basic {}", auth_base64))?,
            );
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            let body = json!({
                "jql": jql,
                "fields": ["summary", "status", "duedate", "updated"],
                "maxResults": 100
            });

            let response = self.client.post(&url).headers(headers).json(&body).send()?;
            if !response.status().is_success() {
                let status = response.status();
                let error_body = response.text().unwrap_or_default();
                error!(
                    "Jira API error for project {}: {} - {}",
                    project, status, error_body
                );
                return Err(format!("Jira API error: {}", status).into());
            }

            let data: Value = response.json()?;
            if let Some(issues) = data["issues"].as_array() {
                debug!("Found {} issues in project {}", issues.len(), project);
                for issue in issues {
                    let key = issue["key"].as_str().unwrap_or_default().to_string();
                    let fields = &issue["fields"];
                    let summary = fields["summary"].as_str().unwrap_or_default().to_string();
                    let status_name = fields["status"]["name"].as_str().unwrap_or_default();

                    let is_completed = status_name == "Done" || status_name == "Closed";
                    let task_builder = TaskBuilder::new(Uuid::new_v4().to_string())
                        .with_metadata(key.clone(), summary)
                        .with_status(is_completed, false)
                        .with_external(Some(key), TaskSource::Jira);

                    // Parse due date if available
                    let mut end_date = None;
                    if let Some(due_str) = fields["duedate"].as_str() {
                        // duedate is usually "YYYY-MM-DD"
                        if let Ok(naive_date) =
                            chrono::NaiveDate::parse_from_str(due_str, "%Y-%m-%d")
                        {
                            end_date = Some(naive_date.and_hms_opt(0, 0, 0).unwrap().and_utc());
                        }
                    }

                    let task = task_builder.with_schedule(None, end_date).build();
                    all_tasks.push(task);
                }
            }
        }

        info!(
            "Successfully fetched {} total tasks from Jira.",
            all_tasks.len()
        );
        Ok(all_tasks)
    }
}
