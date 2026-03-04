use anyhow::Result;
use sqlx::PgPool;

pub async fn setup_schema(pool: &PgPool) -> Result<()> {
    sqlx::raw_sql(
        r#"
        CREATE SCHEMA IF NOT EXISTS commissions;

        CREATE TABLE IF NOT EXISTS commissions.managers (
            id              TEXT PRIMARY KEY,
            name            TEXT NOT NULL,
            email           TEXT NOT NULL UNIQUE,
            role            TEXT NOT NULL DEFAULT 'manager',
            commission_rate DOUBLE PRECISION NOT NULL DEFAULT 0.10,
            total_earned    DOUBLE PRECISION NOT NULL DEFAULT 0,
            total_pending   DOUBLE PRECISION NOT NULL DEFAULT 0,
            status          TEXT NOT NULL DEFAULT 'active'
                            CHECK (status IN ('active', 'inactive')),
            metadata        JSONB NOT NULL DEFAULT '{}',
            created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
            updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
        );

        CREATE TABLE IF NOT EXISTS commissions.commission_records (
            id          TEXT PRIMARY KEY,
            manager_id  TEXT NOT NULL REFERENCES commissions.managers(id) ON DELETE CASCADE,
            client_id   TEXT NOT NULL,
            project_id  TEXT NOT NULL DEFAULT '',
            amount      DOUBLE PRECISION NOT NULL,
            status      TEXT NOT NULL DEFAULT 'pending'
                        CHECK (status IN ('pending', 'approved', 'paid', 'cancelled')),
            description TEXT NOT NULL DEFAULT '',
            created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
            paid_at     TIMESTAMPTZ
        );

        CREATE TABLE IF NOT EXISTS commissions.referrals (
            id                TEXT PRIMARY KEY,
            manager_id        TEXT NOT NULL REFERENCES commissions.managers(id) ON DELETE CASCADE,
            referred_email    TEXT NOT NULL,
            status            TEXT NOT NULL DEFAULT 'pending'
                              CHECK (status IN ('pending', 'converted', 'expired')),
            commission_share  DOUBLE PRECISION NOT NULL DEFAULT 0.05,
            created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
            converted_at      TIMESTAMPTZ
        );

        CREATE INDEX IF NOT EXISTS idx_commission_records_manager ON commissions.commission_records(manager_id);
        CREATE INDEX IF NOT EXISTS idx_commission_records_status ON commissions.commission_records(status);
        CREATE INDEX IF NOT EXISTS idx_referrals_manager ON commissions.referrals(manager_id);
        CREATE INDEX IF NOT EXISTS idx_managers_email ON commissions.managers(email);
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
