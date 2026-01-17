//! Linear GraphQL API client with connection pooling.

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const GRAPHQL_ENDPOINT: &str = "https://api.linear.app/graphql";

/// Linear API client with persistent connection pooling.
pub struct LinearClient {
    client: Client,
    api_key: String,
}

impl LinearClient {
    /// Create a new Linear client.
    pub fn new(api_key: String) -> Result<Self> {
        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("fgp-linear/0.1.0")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, api_key })
    }

    /// Execute a GraphQL query.
    async fn graphql<T: for<'de> Deserialize<'de>>(
        &self,
        query: &str,
        variables: Option<Value>,
    ) -> Result<T> {
        let body = GraphQLRequest {
            query: query.to_string(),
            variables,
        };

        let response = self
            .client
            .post(GRAPHQL_ENDPOINT)
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send GraphQL request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("GraphQL request failed: {} - {}", status, text);
        }

        let text = response.text().await.context("Failed to read response")?;

        let result: GraphQLResponse<T> = serde_json::from_str(&text).map_err(|e| {
            anyhow::anyhow!(
                "JSON parse error: {} | Raw: {}",
                e,
                &text[..text.len().min(500)]
            )
        })?;

        if let Some(errors) = result.errors {
            if !errors.is_empty() {
                let messages: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
                bail!("GraphQL errors: {}", messages.join(", "));
            }
        }

        result.data.context("GraphQL response missing data field")
    }

    /// Check API connectivity.
    pub async fn ping(&self) -> Result<bool> {
        let query = r#"
            query {
                viewer {
                    id
                }
            }
        "#;

        #[derive(Deserialize)]
        struct ViewerResponse {
            viewer: Viewer,
        }

        #[derive(Deserialize)]
        struct Viewer {
            id: String,
        }

        let result: ViewerResponse = self.graphql(query, None).await?;
        Ok(!result.viewer.id.is_empty())
    }

    /// Get current user info.
    pub async fn me(&self) -> Result<Value> {
        let query = r#"
            query {
                viewer {
                    id
                    name
                    email
                    displayName
                    avatarUrl
                    admin
                    createdAt
                    organization {
                        id
                        name
                    }
                }
            }
        "#;

        #[derive(Deserialize)]
        struct ViewerResponse {
            viewer: Value,
        }

        let result: ViewerResponse = self.graphql(query, None).await?;
        Ok(result.viewer)
    }

    /// List teams.
    pub async fn teams(&self) -> Result<Vec<Value>> {
        let query = r#"
            query {
                teams {
                    nodes {
                        id
                        name
                        key
                        description
                        icon
                        color
                        issueCount
                    }
                }
            }
        "#;

        #[derive(Deserialize)]
        struct TeamsResponse {
            teams: Nodes,
        }

        #[derive(Deserialize)]
        struct Nodes {
            nodes: Vec<Value>,
        }

        let result: TeamsResponse = self.graphql(query, None).await?;
        Ok(result.teams.nodes)
    }

    /// List issues with optional filters.
    pub async fn issues(
        &self,
        team_key: Option<&str>,
        state: Option<&str>,
        assignee: Option<&str>,
        limit: i32,
    ) -> Result<Vec<Value>> {
        let mut filters = Vec::new();

        if let Some(team) = team_key {
            filters.push(format!(r#"team: {{ key: {{ eq: "{}" }} }}"#, team));
        }

        if let Some(state) = state {
            filters.push(format!(r#"state: {{ name: {{ eq: "{}" }} }}"#, state));
        }

        if let Some(assignee) = assignee {
            filters.push(format!(r#"assignee: {{ email: {{ eq: "{}" }} }}"#, assignee));
        }

        let filter_str = if filters.is_empty() {
            String::new()
        } else {
            format!(", filter: {{ {} }}", filters.join(", "))
        };

        let query = format!(
            r#"
            query($first: Int!) {{
                issues(first: $first, orderBy: updatedAt{}) {{
                    nodes {{
                        id
                        identifier
                        title
                        description
                        priority
                        priorityLabel
                        url
                        createdAt
                        updatedAt
                        state {{
                            id
                            name
                            type
                            color
                        }}
                        team {{
                            id
                            name
                            key
                        }}
                        assignee {{
                            id
                            name
                            email
                        }}
                        labels {{
                            nodes {{
                                id
                                name
                                color
                            }}
                        }}
                        comments {{
                            nodes {{
                                id
                            }}
                        }}
                    }}
                }}
            }}
        "#,
            filter_str
        );

        #[derive(Deserialize)]
        struct IssuesResponse {
            issues: Nodes,
        }

        #[derive(Deserialize)]
        struct Nodes {
            nodes: Vec<Value>,
        }

        let variables = serde_json::json!({ "first": limit });
        let result: IssuesResponse = self.graphql(&query, Some(variables)).await?;
        Ok(result.issues.nodes)
    }

    /// Get a single issue by ID or identifier.
    pub async fn issue(&self, id: &str) -> Result<Value> {
        let query = r#"
            query($id: String!) {
                issue(id: $id) {
                    id
                    identifier
                    title
                    description
                    priority
                    priorityLabel
                    url
                    createdAt
                    updatedAt
                    state {
                        id
                        name
                        type
                        color
                    }
                    team {
                        id
                        name
                        key
                    }
                    assignee {
                        id
                        name
                        email
                    }
                    labels {
                        nodes {
                            id
                            name
                            color
                        }
                    }
                    comments {
                        nodes {
                            id
                            body
                            createdAt
                            user {
                                name
                                email
                            }
                        }
                    }
                }
            }
        "#;

        #[derive(Deserialize)]
        struct IssueResponse {
            issue: Value,
        }

        let variables = serde_json::json!({ "id": id });
        let result: IssueResponse = self.graphql(query, Some(variables)).await?;
        Ok(result.issue)
    }

    /// Create a new issue.
    pub async fn create_issue(
        &self,
        team_id: &str,
        title: &str,
        description: Option<&str>,
        priority: Option<i32>,
        assignee_id: Option<&str>,
    ) -> Result<Value> {
        let query = r#"
            mutation($teamId: String!, $title: String!, $description: String, $priority: Int, $assigneeId: String) {
                issueCreate(input: {
                    teamId: $teamId,
                    title: $title,
                    description: $description,
                    priority: $priority,
                    assigneeId: $assigneeId
                }) {
                    success
                    issue {
                        id
                        identifier
                        title
                        url
                        createdAt
                        state {
                            name
                        }
                    }
                }
            }
        "#;

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct CreateResponse {
            issue_create: CreateData,
        }

        #[derive(Deserialize)]
        struct CreateData {
            success: bool,
            issue: Value,
        }

        let variables = serde_json::json!({
            "teamId": team_id,
            "title": title,
            "description": description,
            "priority": priority,
            "assigneeId": assignee_id
        });

        let result: CreateResponse = self.graphql(query, Some(variables)).await?;
        if !result.issue_create.success {
            bail!("Failed to create issue");
        }
        Ok(result.issue_create.issue)
    }

    /// Update an issue.
    pub async fn update_issue(
        &self,
        id: &str,
        title: Option<&str>,
        description: Option<&str>,
        state_id: Option<&str>,
        priority: Option<i32>,
        assignee_id: Option<&str>,
    ) -> Result<Value> {
        let query = r#"
            mutation($id: String!, $title: String, $description: String, $stateId: String, $priority: Int, $assigneeId: String) {
                issueUpdate(id: $id, input: {
                    title: $title,
                    description: $description,
                    stateId: $stateId,
                    priority: $priority,
                    assigneeId: $assigneeId
                }) {
                    success
                    issue {
                        id
                        identifier
                        title
                        url
                        updatedAt
                        state {
                            name
                        }
                    }
                }
            }
        "#;

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct UpdateResponse {
            issue_update: UpdateData,
        }

        #[derive(Deserialize)]
        struct UpdateData {
            success: bool,
            issue: Value,
        }

        let variables = serde_json::json!({
            "id": id,
            "title": title,
            "description": description,
            "stateId": state_id,
            "priority": priority,
            "assigneeId": assignee_id
        });

        let result: UpdateResponse = self.graphql(query, Some(variables)).await?;
        if !result.issue_update.success {
            bail!("Failed to update issue");
        }
        Ok(result.issue_update.issue)
    }

    /// Get comments for an issue.
    pub async fn comments(&self, issue_id: &str) -> Result<Vec<Value>> {
        let query = r#"
            query($issueId: String!) {
                issue(id: $issueId) {
                    comments {
                        nodes {
                            id
                            body
                            createdAt
                            updatedAt
                            user {
                                id
                                name
                                email
                            }
                        }
                    }
                }
            }
        "#;

        #[derive(Deserialize)]
        struct IssueResponse {
            issue: IssueData,
        }

        #[derive(Deserialize)]
        struct IssueData {
            comments: Nodes,
        }

        #[derive(Deserialize)]
        struct Nodes {
            nodes: Vec<Value>,
        }

        let variables = serde_json::json!({ "issueId": issue_id });
        let result: IssueResponse = self.graphql(query, Some(variables)).await?;
        Ok(result.issue.comments.nodes)
    }

    /// Add a comment to an issue.
    pub async fn add_comment(&self, issue_id: &str, body: &str) -> Result<Value> {
        let query = r#"
            mutation($issueId: String!, $body: String!) {
                commentCreate(input: {
                    issueId: $issueId,
                    body: $body
                }) {
                    success
                    comment {
                        id
                        body
                        createdAt
                        user {
                            name
                        }
                    }
                }
            }
        "#;

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct CommentResponse {
            comment_create: CommentData,
        }

        #[derive(Deserialize)]
        struct CommentData {
            success: bool,
            comment: Value,
        }

        let variables = serde_json::json!({
            "issueId": issue_id,
            "body": body
        });

        let result: CommentResponse = self.graphql(query, Some(variables)).await?;
        if !result.comment_create.success {
            bail!("Failed to create comment");
        }
        Ok(result.comment_create.comment)
    }

    /// List projects.
    pub async fn projects(&self, team_key: Option<&str>, limit: i32) -> Result<Vec<Value>> {
        let filter = team_key
            .map(|k| format!(r#", filter: {{ accessibleTeams: {{ key: {{ eq: "{}" }} }} }}"#, k))
            .unwrap_or_default();

        let query = format!(
            r#"
            query($first: Int!) {{
                projects(first: $first{}) {{
                    nodes {{
                        id
                        name
                        description
                        url
                        state
                        progress
                        startedAt
                        targetDate
                        lead {{
                            id
                            name
                        }}
                        teams {{
                            nodes {{
                                id
                                name
                                key
                            }}
                        }}
                    }}
                }}
            }}
        "#,
            filter
        );

        #[derive(Deserialize)]
        struct ProjectsResponse {
            projects: Nodes,
        }

        #[derive(Deserialize)]
        struct Nodes {
            nodes: Vec<Value>,
        }

        let variables = serde_json::json!({ "first": limit });
        let result: ProjectsResponse = self.graphql(&query, Some(variables)).await?;
        Ok(result.projects.nodes)
    }

    /// List cycles (sprints).
    pub async fn cycles(&self, team_key: Option<&str>, limit: i32) -> Result<Vec<Value>> {
        let filter = team_key
            .map(|k| format!(r#", filter: {{ team: {{ key: {{ eq: "{}" }} }} }}"#, k))
            .unwrap_or_default();

        let query = format!(
            r#"
            query($first: Int!) {{
                cycles(first: $first{}) {{
                    nodes {{
                        id
                        name
                        number
                        startsAt
                        endsAt
                        completedAt
                        progress
                        issueCountHistory
                        team {{
                            id
                            name
                            key
                        }}
                    }}
                }}
            }}
        "#,
            filter
        );

        #[derive(Deserialize)]
        struct CyclesResponse {
            cycles: Nodes,
        }

        #[derive(Deserialize)]
        struct Nodes {
            nodes: Vec<Value>,
        }

        let variables = serde_json::json!({ "first": limit });
        let result: CyclesResponse = self.graphql(&query, Some(variables)).await?;
        Ok(result.cycles.nodes)
    }

    /// Search issues.
    pub async fn search(&self, query_str: &str, limit: i32) -> Result<Vec<Value>> {
        let query = r#"
            query($query: String!, $first: Int!) {
                issueSearch(query: $query, first: $first) {
                    nodes {
                        id
                        identifier
                        title
                        url
                        state {
                            name
                        }
                        team {
                            key
                        }
                        assignee {
                            name
                        }
                    }
                }
            }
        "#;

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SearchResponse {
            issue_search: Nodes,
        }

        #[derive(Deserialize)]
        struct Nodes {
            nodes: Vec<Value>,
        }

        let variables = serde_json::json!({
            "query": query_str,
            "first": limit
        });

        let result: SearchResponse = self.graphql(query, Some(variables)).await?;
        Ok(result.issue_search.nodes)
    }

    /// Get workflow states for a team.
    pub async fn states(&self, team_key: &str) -> Result<Vec<Value>> {
        let query = r#"
            query($teamKey: String!) {
                team(key: $teamKey) {
                    states {
                        nodes {
                            id
                            name
                            type
                            color
                            position
                        }
                    }
                }
            }
        "#;

        #[derive(Deserialize)]
        struct TeamResponse {
            team: TeamData,
        }

        #[derive(Deserialize)]
        struct TeamData {
            states: Nodes,
        }

        #[derive(Deserialize)]
        struct Nodes {
            nodes: Vec<Value>,
        }

        let variables = serde_json::json!({ "teamKey": team_key });
        let result: TeamResponse = self.graphql(query, Some(variables)).await?;
        Ok(result.team.states.nodes)
    }
}

/// GraphQL request body.
#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<Value>,
}

/// GraphQL response wrapper.
#[derive(Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

/// GraphQL error.
#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

#[cfg(test)]
mod tests {
    use super::GraphQLResponse;
    use serde_json::json;

    #[test]
    fn graphql_response_with_data() {
        let value = json!({
            "data": {
                "viewer": {
                    "id": "user_1"
                }
            }
        });
        let response: GraphQLResponse<serde_json::Value> =
            serde_json::from_value(value).expect("deserialize response");

        let data = response.data.expect("data");
        assert_eq!(data["viewer"]["id"].as_str(), Some("user_1"));
        assert!(response.errors.is_none());
    }

    #[test]
    fn graphql_response_with_errors() {
        let value = json!({
            "errors": [
                { "message": "Unauthorized" }
            ]
        });
        let response: GraphQLResponse<serde_json::Value> =
            serde_json::from_value(value).expect("deserialize response");

        assert!(response.data.is_none());
        let errors = response.errors.expect("errors");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Unauthorized");
    }
}
