use chrono::NaiveDateTime;
use sqlx::prelude::*;

#[derive(Debug, FromRow)]
pub struct BudgetRow {
    pub id: i64,
    pub name: String,
    pub version: i64,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, FromRow)]
pub struct BudgetCategoryRow {
    pub id: i64,
    pub budget_id: i64,
    pub name: String,
}

// Separate struct for the joined query results
#[derive(Debug, FromRow)]
pub struct CategoryWithEntry {
    // Category fields
    pub category_id: i64,
    pub category_name: String,

    // Entry fields (nullable due to LEFT JOIN)
    pub entry_id: Option<i64>,
    pub amount_cents: Option<i64>,
    pub month: Option<i64>,
    pub year: Option<i64>,
    pub entry_created_at: Option<NaiveDateTime>,
}
