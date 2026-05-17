use std::collections::BTreeMap;

use sea_orm::{
    ActiveValue::Set, DbBackend, DeriveEntityModel, DeriveRelation, EnumIter, MockDatabase,
    MockExecResult, entity::prelude::*, sea_query::Value,
};
use seaorm::{OrmError, Repository, store::repository::repo, types::Page};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "widgets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

fn widget(id: i32, name: &str) -> Model {
    Model {
        id,
        name: name.to_owned(),
    }
}

fn active_widget(id: i32, name: &str) -> ActiveModel {
    ActiveModel {
        id: Set(id),
        name: Set(name.to_owned()),
    }
}

fn count_row(total: i64) -> BTreeMap<String, Value> {
    let mut row = BTreeMap::new();
    row.insert("num_items".to_owned(), total.into());
    row
}

#[tokio::test]
async fn repository_insert_update_get_and_delete_use_the_database_connection() {
    let db = MockDatabase::new(DbBackend::Postgres)
        .append_query_results([
            [widget(1, "created")],
            [widget(1, "updated")],
            [widget(1, "updated")],
        ])
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let repository = Repository::<Entity>::new(db);

    let created = repository
        .insert(active_widget(1, "created"))
        .await
        .unwrap();
    assert_eq!(created, widget(1, "created"));

    let updated = repository
        .update(active_widget(1, "updated"))
        .await
        .unwrap();
    assert_eq!(updated, widget(1, "updated"));

    let found = repository.get_by_id(1).await.unwrap();
    assert_eq!(found, widget(1, "updated"));

    let rows_deleted = repository.delete_by_id(1).await.unwrap();
    assert_eq!(rows_deleted, 1);
}

#[tokio::test]
async fn repository_get_by_id_maps_missing_rows_to_not_found() {
    let db = MockDatabase::new(DbBackend::Postgres)
        .append_query_results([Vec::<Model>::new()])
        .into_connection();
    let repository = Repository::<Entity>::new(db);

    let err = repository.get_by_id(404).await.unwrap_err();

    assert!(matches!(err, OrmError::NotFound));
}

#[tokio::test]
async fn repository_list_returns_paginated_results() {
    let db = MockDatabase::new(DbBackend::Postgres)
        .append_query_results([[count_row(2)]])
        .append_query_results([[widget(1, "first"), widget(2, "second")]])
        .into_connection();
    let repository = repo::<Entity>(db);
    let page = Page {
        page: 1,
        per_page: 50,
    };

    let results = repository.list(&page, |query| query).await.unwrap();

    assert_eq!(results.items, vec![widget(1, "first"), widget(2, "second")]);
    assert_eq!(results.total, 2);
    assert_eq!(results.page, 1);
    assert_eq!(results.per_page, 50);
    assert_eq!(results.total_pages, 1);
}
