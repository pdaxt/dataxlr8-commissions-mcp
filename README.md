# dataxlr8-commissions-mcp

Commission tracking MCP server for the DataXLR8 platform.

## What It Does

Tracks sales manager commissions from deal closings. Register managers, record commissions with amounts and percentages, update payment status, and view leaderboards and aggregate stats. Supports referral tracking between managers.

## Tools

| Tool | Description |
|------|-------------|
| `list_managers` | List all registered managers |
| `get_manager` | Get a manager's details by ID |
| `create_manager` | Register a new sales manager |
| `record_commission` | Record a commission for a deal |
| `update_commission_status` | Update payment status (pending/paid/cancelled) |
| `get_commissions` | List commissions with optional filters |
| `commission_stats` | Aggregate stats (total earned, paid, pending) |
| `leaderboard` | Ranked manager leaderboard by earnings |

## Quick Start

```bash
export DATABASE_URL=postgres://user:pass@localhost:5432/dataxlr8

cargo build
cargo run
```

## Schema

Creates a `commissions` schema with:

| Table | Purpose |
|-------|---------|
| `commissions.managers` | Manager profiles (name, email, tier, status) |
| `commissions.commission_records` | Individual commission entries (amount, rate, status) |
| `commissions.referrals` | Manager-to-manager referral tracking |

## Part of the [DataXLR8](https://github.com/pdaxt) Platform
