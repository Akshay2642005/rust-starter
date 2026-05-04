use seaorm::types::{Page, Paginated};

#[test]
fn page_defaults_are_safe_for_pagination() {
    let page = Page::default();

    assert_eq!(page.page(), 1);
    assert_eq!(page.per_page(), 20);
    assert_eq!(page.zero_indexed(), 0);
}

#[test]
fn page_clamps_zero_values_to_safe_minimums() {
    let page = Page {
        page: 0,
        per_page: 0,
    };

    assert_eq!(page.page(), 1);
    assert_eq!(page.per_page(), 1);
    assert_eq!(page.zero_indexed(), 0);
}

#[test]
fn page_caps_large_page_sizes() {
    let page = Page {
        page: 3,
        per_page: 1_000,
    };

    assert_eq!(page.page(), 3);
    assert_eq!(page.per_page(), 100);
    assert_eq!(page.zero_indexed(), 2);
}

#[test]
fn paginated_uses_normalized_page_values() {
    let page = Page {
        page: 0,
        per_page: 0,
    };

    let response = Paginated::new(vec!["a", "b"], 2, &page);

    assert_eq!(response.items, vec!["a", "b"]);
    assert_eq!(response.total, 2);
    assert_eq!(response.page, 1);
    assert_eq!(response.per_page, 1);
    assert_eq!(response.total_pages, 2);
}

#[test]
fn paginated_reports_at_least_one_total_page_for_empty_results() {
    let page = Page {
        page: 1,
        per_page: 25,
    };

    let response = Paginated::<i32>::new(Vec::new(), 0, &page);

    assert_eq!(response.total_pages, 1);
}
