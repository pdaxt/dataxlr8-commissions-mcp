# :moneybag: dataxlr8-commissions-mcp

Commission tracking for AI agents — record earnings, manage sales managers, track payments, and rank by performance.

[![Rust](https://img.shields.io/badge/Rust-2024_edition-orange?logo=rust)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-rmcp_0.17-blue)](https://modelcontextprotocol.io/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

## What It Does

Tracks sales commissions from deal closings through MCP tool calls. Register sales managers with tiers, record commissions with amounts and percentages tied to deals, update payment status from pending through paid, view aggregate earnings stats, and rank managers on a leaderboard. Supports referral tracking between managers — all backed by PostgreSQL.

## Architecture

```
                    ┌───────────────────────────────┐
AI Agent ──stdio──▶ │  dataxlr8-commissions-mcp     │
                    │  (rmcp 0.17 server)            │
                    └──────────┬────────────────────┘
                               │ sqlx 0.8
                               ▼
                    ┌─────────────────────────┐
                    │  PostgreSQL              │
                    │  schema: commissions     │
                    │  ├── managers            │
                    │  ├── commission_records  │
                    │  └── referrals           │
                    └─────────────────────────┘
```

## Tools

| Tool | Description |
|------|-------------|
| `list_managers` | List all registered sales managers |
| `get_manager` | Get a manager's details by ID |
| `create_manager` | Register a new sales manager with tier |
| `record_commission` | Record a commission for a closed deal |
| `update_commission_status` | Update payment status (pending/paid/cancelled) |
| `get_commissions` | List commissions with optional filters |
| `commission_stats` | Aggregate stats: total earned, paid, pending |
| `leaderboard` | Ranked manager leaderboard by earnings |

## Quick Start

```bash
git clone https://github.com/pdaxt/dataxlr8-commissions-mcp
cd dataxlr8-commissions-mcp
cargo build --release

export DATABASE_URL=postgres://user:pass@localhost:5432/dataxlr8
./target/release/dataxlr8-commissions-mcp
```

The server auto-creates the `commissions` schema and all tables on first run.

## Configuration

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection string |
| `LOG_LEVEL` | No | Tracing level (default: `info`) |

## Claude Desktop Integration

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "dataxlr8-commissions": {
      "command": "./target/release/dataxlr8-commissions-mcp",
      "env": {
        "DATABASE_URL": "postgres://user:pass@localhost:5432/dataxlr8"
      }
    }
  }
}
```

## Part of DataXLR8

One of 14 Rust MCP servers that form the [DataXLR8](https://github.com/pdaxt) platform — a modular, AI-native business operations suite. Each server owns a single domain, shares a PostgreSQL instance, and communicates over the Model Context Protocol.

## License

MIT
