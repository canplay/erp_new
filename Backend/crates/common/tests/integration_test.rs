//! Integration tests for common crate types

#[cfg(test)]
mod page_query_tests {
    use common::repository::PageQuery;

    #[test]
    fn test_page_query_defaults() {
        let q = PageQuery {
            page: 1,
            page_size: 10,
        };
        assert_eq!(q.page, 1);
        assert_eq!(q.page_size, 10);
    }

    #[test]
    fn test_page_query_new() {
        let q = PageQuery::new(Some(2), Some(20));
        assert_eq!(q.page, 2);
        assert_eq!(q.page_size, 20);
    }

    #[test]
    fn test_page_query_offset() {
        let q = PageQuery {
            page: 3,
            page_size: 10,
        };
        assert_eq!(q.offset(), 20); // (3-1)*10
    }

    #[test]
    fn test_page_query_clamp() {
        let q = PageQuery::new(Some(0), Some(200));
        assert_eq!(q.page, 1); // min 1
        assert_eq!(q.page_size, 100); // max 100
    }
}

#[cfg(test)]
mod page_result_tests {
    use common::repository::PageResult;

    #[test]
    fn test_page_result_new_empty() {
        let r: PageResult<i32> = PageResult::<i32>::new(vec![], 0, 1, 10);
        assert_eq!(r.records.len(), 0);
        assert_eq!(r.total, 0);
        assert_eq!(r.pages, 0);
    }

    #[test]
    fn test_page_result_new_with_data() {
        let r = PageResult::new(vec![1, 2, 3], 3, 1, 10);
        assert_eq!(r.records.len(), 3);
        assert_eq!(r.total, 3);
        assert_eq!(r.pages, 1);
    }

    #[test]
    fn test_page_result_has_next() {
        let r = PageResult::new(vec![1, 2, 3], 30, 1, 10);
        assert!(r.has_next());
    }

    #[test]
    fn test_page_result_no_next() {
        let r = PageResult::new(vec![1, 2, 3], 10, 1, 10);
        assert!(!r.has_next());
    }
}

#[cfg(test)]
mod error_tests {
    use common::AppError;

    #[test]
    fn test_app_error_unauthorized() {
        let e = AppError::Unauthorized("test".into());
        assert!(e.to_string().contains("认证失败" ));
    }

    #[test]
    fn test_app_error_not_found() {
        let e = AppError::NotFound("resource".into());
        assert!(e.to_string().contains("resource" ));
    }
}
