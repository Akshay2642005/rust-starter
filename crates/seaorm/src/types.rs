//! Shared create/update/pagination types for the store layer.
//!
//! `CreateFields` and `UpdateFields` are optional service-layer carriers.
//! Generated SeaORM entities remain the database source of truth.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PER_PAGE: u64 = 20;
const MAX_PER_PAGE: u64 = 100;

/// Optional shared fields for service-level create DTOs.
#[derive(Debug, Default, Clone)]
pub struct CreateFields {
    pub metadata: Option<serde_json::Value>,
    pub slug: Option<String>,
}

/// Optional shared fields for service-level update DTOs.
#[derive(Debug, Default, Clone)]
pub struct UpdateFields {
    pub deleted_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub slug: Option<String>,
}

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
            page: DEFAULT_PAGE,
            per_page: DEFAULT_PER_PAGE,
        }
    }
}

impl Page {
    /// Return a 1-indexed page number clamped to the supported range.
    #[inline]
    pub fn page(&self) -> u64 {
        self.page.max(1)
    }

    /// Return a page size clamped to the supported range.
    #[inline]
    pub fn per_page(&self) -> u64 {
        self.per_page.clamp(1, MAX_PER_PAGE)
    }

    /// Convert to a 0-indexed offset for SeaORM's paginator.
    #[inline]
    pub fn zero_indexed(&self) -> u64 {
        self.page().saturating_sub(1)
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
        let per_page = p.per_page();
        let total_pages = total.div_ceil(per_page).max(1);
        Self {
            items,
            total,
            page: p.page(),
            per_page,
            total_pages,
        }
    }
}

fn default_page() -> u64 {
    DEFAULT_PAGE
}
fn default_per_page() -> u64 {
    DEFAULT_PER_PAGE
}
