//! Shared create/update/pagination types for the store layer.
//!
//! `CreateFields` and `UpdateFields` carry only the plugin-level fields
//! that `#[derive(OrmEntity)]` knows about.  Every entity repository defines
//! its own `CreateXxx` / `UpdateXxx` DTOs on top of these.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Plugin-level create/update carriers ───────────────────────────────────────

/// Plugin fields used by `SeaOrmModel::new_active`.
/// Entity repositories wrap this inside their own `CreateXxx` DTO.
#[derive(Debug, Default, Clone)]
pub struct CreateFields {
    // Plugin: metadata
    pub metadata: Option<serde_json::Value>,
    // Plugin: slug
    pub slug: Option<String>,
}

/// Plugin fields used by `SeaOrmModel::stamp_updated_at`.
/// Entity repositories wrap this inside their own `UpdateXxx` DTO.
#[derive(Debug, Default, Clone)]
pub struct UpdateFields {
    // Plugin: soft-delete
    pub deleted_at: Option<DateTime<Utc>>,
    // Plugin: metadata
    pub metadata: Option<serde_json::Value>,
    // Plugin: slug
    pub slug: Option<String>,
}

// ── Pagination ────────────────────────────────────────────────────────────────

/// Page cursor for list queries.
#[derive(Debug, Clone, Deserialize)]
pub struct Page {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
        }
    }
}

impl Page {
    /// Convert to a 0-indexed offset for SeaORM's paginator.
    #[inline]
    pub fn zero_indexed(&self) -> u64 {
        self.page.saturating_sub(1)
    }
}

/// Paginated response envelope — generic over any item type.
#[derive(Debug, Serialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

impl<T> Paginated<T> {
    pub fn new(items: Vec<T>, total: u64, p: &Page) -> Self {
        let total_pages = total.div_ceil(p.per_page).max(1);
        Self {
            items,
            total,
            page: p.page,
            per_page: p.per_page,
            total_pages,
        }
    }
}

fn default_page() -> u64 {
    1
}
fn default_per_page() -> u64 {
    20
}
