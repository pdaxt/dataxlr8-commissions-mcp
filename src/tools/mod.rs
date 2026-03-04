use dataxlr8_mcp_core::Database;
use rmcp::model::*;
use rmcp::service::{RequestContext, RoleServer};
use rmcp::ServerHandler;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

// ============================================================================
// Data types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Manager {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub commission_rate: f64,
    pub total_earned: f64,
    pub total_pending: f64,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CommissionRecord {
    pub id: String,
    pub manager_id: String,
    pub client_id: String,
    pub project_id: String,
    pub amount: f64,
    pub status: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Referral {
    pub id: String,
    pub manager_id: String,
    pub referred_email: String,
    pub status: String,
    pub commission_share: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub converted_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct CommissionStats {
    pub total_earned: f64,
    pub total_pending: f64,
    pub total_paid: f64,
    pub count: i64,
    pub recent: Vec<CommissionRecord>,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardEntry {
    pub name: String,
    pub email: String,
    pub total_earned: f64,
    pub deal_count: i64,
}

// ============================================================================
// Tool schema helpers
// ============================================================================

fn make_schema(
    properties: serde_json::Value,
    required: Vec<&str>,
) -> Arc<serde_json::Map<String, serde_json::Value>> {
    let mut m = serde_json::Map::new();
    m.insert("type".into(), serde_json::Value::String("object".into()));
    m.insert("properties".into(), properties);
    if !required.is_empty() {
        m.insert(
            "required".into(),
            serde_json::Value::Array(required.into_iter().map(|s| serde_json::Value::String(s.into())).collect()),
        );
    }
    Arc::new(m)
}

fn empty_schema() -> Arc<serde_json::Map<String, serde_json::Value>> {
    let mut m = serde_json::Map::new();
    m.insert("type".into(), serde_json::Value::String("object".into()));
    Arc::new(m)
}

fn build_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_managers".into(),
            title: None,
            description: Some("List all managers with their commission rates and earnings".into()),
            input_schema: empty_schema(),
            output_schema: None, annotations: None, execution: None, icons: None, meta: None,
        },
        Tool {
            name: "get_manager".into(),
            title: None,
            description: Some("Get a manager profile by ID or email".into()),
            input_schema: make_schema(serde_json::json!({
                "id": { "type": "string", "description": "Manager ID" },
                "email": { "type": "string", "description": "Manager email (alternative lookup)" }
            }), vec![]),
            output_schema: None, annotations: None, execution: None, icons: None, meta: None,
        },
        Tool {
            name: "create_manager".into(),
            title: None,
            description: Some("Create a new manager profile".into()),
            input_schema: make_schema(serde_json::json!({
                "name": { "type": "string", "description": "Full name" },
                "email": { "type": "string", "description": "Email address" },
                "role": { "type": "string", "description": "Role (default: manager)" },
                "commission_rate": { "type": "number", "description": "Commission rate as decimal, e.g. 0.10 for 10%" }
            }), vec!["name", "email"]),
            output_schema: None, annotations: None, execution: None, icons: None, meta: None,
        },
        Tool {
            name: "record_commission".into(),
            title: None,
            description: Some("Record a new commission for a manager".into()),
            input_schema: make_schema(serde_json::json!({
                "manager_id": { "type": "string", "description": "Manager ID" },
                "client_id": { "type": "string", "description": "Client/deal ID" },
                "project_id": { "type": "string", "description": "Project ID" },
                "amount": { "type": "number", "description": "Commission amount" },
                "description": { "type": "string", "description": "Description of the commission" }
            }), vec!["manager_id", "client_id", "amount"]),
            output_schema: None, annotations: None, execution: None, icons: None, meta: None,
        },
        Tool {
            name: "update_commission_status".into(),
            title: None,
            description: Some("Update a commission record status (pending → approved → paid)".into()),
            input_schema: make_schema(serde_json::json!({
                "id": { "type": "string", "description": "Commission record ID" },
                "status": { "type": "string", "enum": ["pending", "approved", "paid", "cancelled"], "description": "New status" }
            }), vec!["id", "status"]),
            output_schema: None, annotations: None, execution: None, icons: None, meta: None,
        },
        Tool {
            name: "get_commissions".into(),
            title: None,
            description: Some("Get commission records, optionally filtered by manager or status".into()),
            input_schema: make_schema(serde_json::json!({
                "manager_id": { "type": "string", "description": "Filter by manager ID" },
                "status": { "type": "string", "enum": ["pending", "approved", "paid", "cancelled"], "description": "Filter by status" },
                "limit": { "type": "integer", "description": "Max results (default 50)" }
            }), vec![]),
            output_schema: None, annotations: None, execution: None, icons: None, meta: None,
        },
        Tool {
            name: "commission_stats".into(),
            title: None,
            description: Some("Get commission statistics — totals, pending, paid, recent activity".into()),
            input_schema: make_schema(serde_json::json!({
                "manager_id": { "type": "string", "description": "Filter stats by manager ID (omit for global)" }
            }), vec![]),
            output_schema: None, annotations: None, execution: None, icons: None, meta: None,
        },
        Tool {
            name: "leaderboard".into(),
            title: None,
            description: Some("Get manager leaderboard ranked by total earnings".into()),
            input_schema: empty_schema(),
            output_schema: None, annotations: None, execution: None, icons: None, meta: None,
        },
    ]
}

// ============================================================================
// MCP Server
// ============================================================================

#[derive(Clone)]
pub struct CommissionsMcpServer {
    db: Database,
}

impl CommissionsMcpServer {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn json_result<T: Serialize>(data: &T) -> CallToolResult {
        match serde_json::to_string_pretty(data) {
            Ok(json) => CallToolResult::success(vec![Content::text(json)]),
            Err(e) => CallToolResult::error(vec![Content::text(format!("Serialization error: {e}"))]),
        }
    }

    fn error_result(msg: &str) -> CallToolResult {
        CallToolResult::error(vec![Content::text(msg.to_string())])
    }

    fn get_str(args: &serde_json::Value, key: &str) -> Option<String> {
        args.get(key).and_then(|v| v.as_str()).map(String::from)
    }

    fn get_f64(args: &serde_json::Value, key: &str) -> Option<f64> {
        args.get(key).and_then(|v| v.as_f64())
    }

    fn get_i64(args: &serde_json::Value, key: &str) -> Option<i64> {
        args.get(key).and_then(|v| v.as_i64())
    }

    // ---- Tool handlers ----

    async fn handle_list_managers(&self) -> CallToolResult {
        match sqlx::query_as::<_, Manager>("SELECT * FROM commissions.managers ORDER BY total_earned DESC")
            .fetch_all(self.db.pool())
            .await
        {
            Ok(managers) => Self::json_result(&managers),
            Err(e) => Self::error_result(&format!("Database error: {e}")),
        }
    }

    async fn handle_get_manager(&self, args: &serde_json::Value) -> CallToolResult {
        let manager: Option<Manager> = if let Some(id) = Self::get_str(args, "id") {
            sqlx::query_as("SELECT * FROM commissions.managers WHERE id = $1")
                .bind(&id)
                .fetch_optional(self.db.pool())
                .await
                .unwrap_or(None)
        } else if let Some(email) = Self::get_str(args, "email") {
            sqlx::query_as("SELECT * FROM commissions.managers WHERE email = $1")
                .bind(&email)
                .fetch_optional(self.db.pool())
                .await
                .unwrap_or(None)
        } else {
            return Self::error_result("Provide either id or email");
        };

        match manager {
            Some(m) => Self::json_result(&m),
            None => Self::error_result("Manager not found"),
        }
    }

    async fn handle_create_manager(&self, args: &serde_json::Value) -> CallToolResult {
        let name = match Self::get_str(args, "name") {
            Some(n) => n,
            None => return Self::error_result("Missing required: name"),
        };
        let email = match Self::get_str(args, "email") {
            Some(e) => e,
            None => return Self::error_result("Missing required: email"),
        };
        let role = Self::get_str(args, "role").unwrap_or_else(|| "manager".into());
        let rate = Self::get_f64(args, "commission_rate").unwrap_or(0.10);
        let id = uuid::Uuid::new_v4().to_string();

        match sqlx::query_as::<_, Manager>(
            "INSERT INTO commissions.managers (id, name, email, role, commission_rate) \
             VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(&id).bind(&name).bind(&email).bind(&role).bind(rate)
        .fetch_one(self.db.pool())
        .await
        {
            Ok(m) => {
                info!(id = id, name = name, "Created manager");
                Self::json_result(&m)
            }
            Err(e) => Self::error_result(&format!("Failed to create manager: {e}")),
        }
    }

    async fn handle_record_commission(&self, args: &serde_json::Value) -> CallToolResult {
        let manager_id = match Self::get_str(args, "manager_id") {
            Some(i) => i,
            None => return Self::error_result("Missing required: manager_id"),
        };
        let client_id = match Self::get_str(args, "client_id") {
            Some(i) => i,
            None => return Self::error_result("Missing required: client_id"),
        };
        let amount = match Self::get_f64(args, "amount") {
            Some(a) => a,
            None => return Self::error_result("Missing required: amount"),
        };
        let project_id = Self::get_str(args, "project_id").unwrap_or_default();
        let description = Self::get_str(args, "description").unwrap_or_default();
        let id = uuid::Uuid::new_v4().to_string();

        // Verify manager exists
        let exists: Option<(String,)> = sqlx::query_as("SELECT id FROM commissions.managers WHERE id = $1")
            .bind(&manager_id)
            .fetch_optional(self.db.pool())
            .await
            .unwrap_or(None);
        if exists.is_none() {
            return Self::error_result(&format!("Manager '{manager_id}' not found"));
        }

        match sqlx::query_as::<_, CommissionRecord>(
            "INSERT INTO commissions.commission_records (id, manager_id, client_id, project_id, amount, description) \
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        )
        .bind(&id).bind(&manager_id).bind(&client_id).bind(&project_id).bind(amount).bind(&description)
        .fetch_one(self.db.pool())
        .await
        {
            Ok(rec) => {
                // Update manager's pending total
                let _ = sqlx::query(
                    "UPDATE commissions.managers SET total_pending = total_pending + $1, updated_at = now() WHERE id = $2",
                )
                .bind(amount).bind(&manager_id)
                .execute(self.db.pool())
                .await;

                info!(id = id, manager_id = manager_id, amount = amount, "Recorded commission");
                Self::json_result(&rec)
            }
            Err(e) => Self::error_result(&format!("Failed to record commission: {e}")),
        }
    }

    async fn handle_update_commission_status(&self, args: &serde_json::Value) -> CallToolResult {
        let id = match Self::get_str(args, "id") {
            Some(i) => i,
            None => return Self::error_result("Missing required: id"),
        };
        let new_status = match Self::get_str(args, "status") {
            Some(s) => s,
            None => return Self::error_result("Missing required: status"),
        };

        // Get existing record
        let existing: Option<CommissionRecord> = match sqlx::query_as(
            "SELECT * FROM commissions.commission_records WHERE id = $1",
        )
        .bind(&id)
        .fetch_optional(self.db.pool())
        .await
        {
            Ok(r) => r,
            Err(e) => return Self::error_result(&format!("Database error: {e}")),
        };

        let existing = match existing {
            Some(r) => r,
            None => return Self::error_result(&format!("Commission '{id}' not found")),
        };

        let old_status = &existing.status;
        let paid_at = if new_status == "paid" {
            Some(chrono::Utc::now())
        } else {
            None
        };

        match sqlx::query_as::<_, CommissionRecord>(
            "UPDATE commissions.commission_records SET status = $1, paid_at = COALESCE($2, paid_at) WHERE id = $3 RETURNING *",
        )
        .bind(&new_status).bind(&paid_at).bind(&id)
        .fetch_one(self.db.pool())
        .await
        {
            Ok(rec) => {
                // Update manager totals based on status transition
                let amount = existing.amount;
                let manager_id = &existing.manager_id;

                if old_status == "pending" && new_status == "paid" {
                    let _ = sqlx::query(
                        "UPDATE commissions.managers SET total_pending = total_pending - $1, total_earned = total_earned + $1, updated_at = now() WHERE id = $2",
                    ).bind(amount).bind(manager_id).execute(self.db.pool()).await;
                } else if old_status == "pending" && new_status == "approved" {
                    // No change to totals, just status
                } else if old_status == "approved" && new_status == "paid" {
                    let _ = sqlx::query(
                        "UPDATE commissions.managers SET total_pending = total_pending - $1, total_earned = total_earned + $1, updated_at = now() WHERE id = $2",
                    ).bind(amount).bind(manager_id).execute(self.db.pool()).await;
                } else if new_status == "cancelled" && old_status != "paid" {
                    let _ = sqlx::query(
                        "UPDATE commissions.managers SET total_pending = total_pending - $1, updated_at = now() WHERE id = $2",
                    ).bind(amount).bind(manager_id).execute(self.db.pool()).await;
                }

                info!(id = id, old_status = old_status.as_str(), new_status = new_status, "Updated commission status");
                Self::json_result(&rec)
            }
            Err(e) => Self::error_result(&format!("Failed to update: {e}")),
        }
    }

    async fn handle_get_commissions(&self, args: &serde_json::Value) -> CallToolResult {
        let manager_id = Self::get_str(args, "manager_id");
        let status = Self::get_str(args, "status");
        let limit = Self::get_i64(args, "limit").unwrap_or(50);

        let mut sql = String::from("SELECT * FROM commissions.commission_records WHERE 1=1");
        let mut param_idx = 1u32;
        let mut params: Vec<String> = Vec::new();

        if let Some(ref mid) = manager_id {
            sql.push_str(&format!(" AND manager_id = ${param_idx}"));
            param_idx += 1;
            params.push(mid.clone());
        }
        if let Some(ref s) = status {
            sql.push_str(&format!(" AND status = ${param_idx}"));
            param_idx += 1;
            params.push(s.clone());
        }
        sql.push_str(&format!(" ORDER BY created_at DESC LIMIT ${param_idx}"));

        let mut query = sqlx::query_as::<_, CommissionRecord>(&sql);
        for p in &params {
            query = query.bind(p);
        }
        query = query.bind(limit);

        match query.fetch_all(self.db.pool()).await {
            Ok(records) => Self::json_result(&records),
            Err(e) => Self::error_result(&format!("Database error: {e}")),
        }
    }

    async fn handle_commission_stats(&self, args: &serde_json::Value) -> CallToolResult {
        let manager_id = Self::get_str(args, "manager_id");

        let (where_clause, bind_val) = match &manager_id {
            Some(mid) => (" WHERE manager_id = $1", Some(mid.clone())),
            None => ("", None),
        };

        let total_q = format!("SELECT COALESCE(SUM(amount), 0) FROM commissions.commission_records{where_clause}");
        let pending_q = format!("SELECT COALESCE(SUM(amount), 0) FROM commissions.commission_records{} AND status = 'pending'",
            if bind_val.is_some() { " WHERE manager_id = $1" } else { " WHERE 1=1" });
        let paid_q = format!("SELECT COALESCE(SUM(amount), 0) FROM commissions.commission_records{} AND status = 'paid'",
            if bind_val.is_some() { " WHERE manager_id = $1" } else { " WHERE 1=1" });
        let count_q = format!("SELECT COUNT(*) FROM commissions.commission_records{where_clause}");
        let recent_q = format!("SELECT * FROM commissions.commission_records{where_clause} ORDER BY created_at DESC LIMIT 10");

        let total: (f64,) = if let Some(ref v) = bind_val {
            sqlx::query_as(&total_q).bind(v).fetch_one(self.db.pool()).await.unwrap_or((0.0,))
        } else {
            sqlx::query_as(&total_q).fetch_one(self.db.pool()).await.unwrap_or((0.0,))
        };

        let pending: (f64,) = if let Some(ref v) = bind_val {
            sqlx::query_as(&pending_q).bind(v).fetch_one(self.db.pool()).await.unwrap_or((0.0,))
        } else {
            sqlx::query_as(&pending_q).fetch_one(self.db.pool()).await.unwrap_or((0.0,))
        };

        let paid: (f64,) = if let Some(ref v) = bind_val {
            sqlx::query_as(&paid_q).bind(v).fetch_one(self.db.pool()).await.unwrap_or((0.0,))
        } else {
            sqlx::query_as(&paid_q).fetch_one(self.db.pool()).await.unwrap_or((0.0,))
        };

        let count: (i64,) = if let Some(ref v) = bind_val {
            sqlx::query_as(&count_q).bind(v).fetch_one(self.db.pool()).await.unwrap_or((0,))
        } else {
            sqlx::query_as(&count_q).fetch_one(self.db.pool()).await.unwrap_or((0,))
        };

        let recent: Vec<CommissionRecord> = if let Some(ref v) = bind_val {
            sqlx::query_as(&recent_q).bind(v).fetch_all(self.db.pool()).await.unwrap_or_default()
        } else {
            sqlx::query_as(&recent_q).fetch_all(self.db.pool()).await.unwrap_or_default()
        };

        Self::json_result(&CommissionStats {
            total_earned: total.0,
            total_pending: pending.0,
            total_paid: paid.0,
            count: count.0,
            recent,
        })
    }

    async fn handle_leaderboard(&self) -> CallToolResult {
        match sqlx::query_as::<_, (String, String, f64,)>(
            "SELECT m.name, m.email, m.total_earned FROM commissions.managers m ORDER BY m.total_earned DESC",
        )
        .fetch_all(self.db.pool())
        .await
        {
            Ok(rows) => {
                let mut entries = Vec::new();
                for (name, email, total_earned) in rows {
                    let deal_count: (i64,) = sqlx::query_as(
                        "SELECT COUNT(DISTINCT client_id) FROM commissions.commission_records WHERE manager_id = (SELECT id FROM commissions.managers WHERE email = $1)",
                    )
                    .bind(&email)
                    .fetch_one(self.db.pool())
                    .await
                    .unwrap_or((0,));

                    entries.push(LeaderboardEntry {
                        name,
                        email,
                        total_earned,
                        deal_count: deal_count.0,
                    });
                }
                Self::json_result(&entries)
            }
            Err(e) => Self::error_result(&format!("Database error: {e}")),
        }
    }
}

// ============================================================================
// ServerHandler trait implementation
// ============================================================================

impl ServerHandler for CommissionsMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "DataXLR8 Commissions MCP — track manager commissions, referrals, and leaderboard".into(),
            ),
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, rmcp::ErrorData>> + Send + '_ {
        async {
            Ok(ListToolsResult {
                tools: build_tools(),
                next_cursor: None,
                meta: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, rmcp::ErrorData>> + Send + '_ {
        async move {
            let args = serde_json::to_value(&request.arguments).unwrap_or(serde_json::Value::Null);
            let name_str: &str = request.name.as_ref();

            let result = match name_str {
                "list_managers" => self.handle_list_managers().await,
                "get_manager" => self.handle_get_manager(&args).await,
                "create_manager" => self.handle_create_manager(&args).await,
                "record_commission" => self.handle_record_commission(&args).await,
                "update_commission_status" => self.handle_update_commission_status(&args).await,
                "get_commissions" => self.handle_get_commissions(&args).await,
                "commission_stats" => self.handle_commission_stats(&args).await,
                "leaderboard" => self.handle_leaderboard().await,
                _ => Self::error_result(&format!("Unknown tool: {}", request.name)),
            };

            Ok(result)
        }
    }
}
