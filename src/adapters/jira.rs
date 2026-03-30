use crate::domain::task::{Task, TaskSource};
use crate::adapters::tui::config::JiraConfig;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;
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
        if !self.config.enabled || self.config.projects.is_empty() {
            return Ok(vec![]);
        }

        let mut all_tasks = Vec::new();

        for project in &self.config.projects {
            let jql = format!("project = '{}' AND status != 'Done' ORDER BY updated DESC", project);
            let url = format!("https://{}/rest/api/3/search?jql={}", self.config.domain, urlencoding::encode(&jql));

            let mut headers = HeaderMap::new();
            let auth = format!("{}:{}", self.config.email, self.config.api_token);
            let auth_base64 = general_purpose::STANDARD.encode(auth);
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Basic {}", auth_base64))?);
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            let response = self.client.get(&url).headers(headers).send()?;
            if !response.status().is_success() {
                return Err(format!("Jira API error: {}", response.status()).into());
            }

            let data: Value = response.json()?;
            if let Some(issues) = data["issues"].as_array() {
                for issue in issues {
                    let key = issue["key"].as_str().unwrap_or_default().to_string();
                    let summary = issue["fields"]["summary"].as_str().unwrap_or_default().to_string();
                    let status = issue["fields"]["status"]["name"].as_str().unwrap_or_default();
                    
                    let mut task = Task::new(Uuid::new_v4().to_string(), format!("[{}] {}", key, summary));
                    task.external_id = Some(key);
                    task.source = TaskSource::Jira;
                    task.completed = status == "Done" || status == "Closed";
                    
                    // Jira dates are often in "fields"]["updated"] or custom fields
                    // For simplicity, we'll just set source and content for now.
                    // We could parse "fields"]["duedate"] if needed.
                    
                    all_tasks.push(task);
                }
            }
        }

        Ok(all_tasks)
    }
}
