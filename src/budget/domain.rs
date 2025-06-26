use std::collections::HashMap;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, prelude::*};
use tracing::instrument;

#[derive(Debug)]
pub enum BudgetError {
    NotFound,
    VersionConflict,
    CategoryNotFound,
    InvalidPeriod,
    DatabaseError(sqlx::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Budget {
    pub id: i64,
    pub name: String,
    pub version: i64,
    pub created_at: NaiveDateTime,
    pub categories: Vec<BudgetCategory>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetCategory {
    pub id: i64,
    pub budget_id: i64,
    pub name: String,
    pub entries: Vec<BudgetEntry>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BudgetEntry {
    pub id: i64,
    pub category_id: i64,
    pub amount_cents: i64,
    pub month: i64,
    pub year: i64,
    pub created_at: NaiveDateTime,
}

impl Budget {
    #[instrument]
    pub async fn get_by_id_with_categories(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<Budget>, sqlx::Error> {
        let budget_row = sqlx::query!(
            "SELECT id, name, version, created_at FROM budgets WHERE id = ?",
            id
        )
        .fetch_optional(pool)
        .await?;

        let budget_row = match budget_row {
            Some(row) => row,
            None => return Ok(None),
        };

        let category_rows = sqlx::query!(
            "SELECT
                c.id as category_id,
                c.name as category_name,
                e.id as entry_id,
                e.amount_cents,
                e.month,
                e.year,
                e.created_at as entry_created_at
            FROM budget_categories c
            LEFT JOIN budget_entries e ON c.id = e.category_id
            WHERE c.budget_id = ?
            ORDER BY c.name, e.year, e.month",
            id
        )
        .fetch_all(pool)
        .await?;

        let mut categories_map: HashMap<i64, BudgetCategory> = HashMap::new();

        for row in category_rows {
            let category_id: i64 = row.category_id.unwrap();
            let category_name: String = row.category_name;

            let category = categories_map
                .entry(category_id)
                .or_insert_with(|| BudgetCategory {
                    id: category_id,
                    budget_id: id,
                    name: category_name.clone(),
                    entries: Vec::new(),
                });

            if let (
                Some(entry_id),
                Some(category_id),
                amount_cents,
                month,
                year,
                Some(created_at),
            ) = (
                row.entry_id,
                row.category_id,
                row.amount_cents,
                row.month,
                row.year,
                row.entry_created_at,
            ) {
                category.entries.push(BudgetEntry {
                    id: entry_id,
                    category_id,
                    amount_cents,
                    month,
                    year,
                    created_at,
                });
            }
        }

        // Convert database row to domain object
        let budget = Budget {
            id: budget_row.id,
            name: budget_row.name,
            version: budget_row.version.unwrap(),
            created_at: budget_row.created_at.unwrap(),
            categories: categories_map.into_values().collect(),
        };

        Ok(Some(budget))
    }
}
