
#![cfg_attr(feature = "cargo-clippy", allow(cyclomatic_complexity))]

extern crate tokio_core;

extern crate rincon_core;
extern crate rincon_connector;
extern crate rincon_client;
extern crate rincon_test_helper;

use rincon_core::api::ErrorCode;
use rincon_core::api::connector::{Error, Execute};
use rincon_core::api::query::Query;
use rincon_core::api::types::{EMPTY, Empty, JsonValue};
use rincon_client::aql::types::OptimizerRule;
use rincon_client::cursor::methods::*;
use rincon_client::cursor::types::*;

use rincon_test_helper::*;


#[test]
fn query_returns_cursor_with_no_results() {
    arango_test_with_document_collection("cursor_customers01", |conn, ref mut core| {

        let query = Query::new("FOR c IN cursor_customers01 RETURN c");

        let method = CreateCursor::<JsonValue>::from_query(query);
        let work = conn.execute(method);
        let cursor = core.run(work).unwrap();

        assert!(cursor.result().is_empty());
        assert_eq!(false, cursor.has_more());
        assert_eq!(None, cursor.id());
    });
}

#[test]
fn insert_documents_and_return_their_names() {
    arango_test_with_document_collection("cursor_customers02", |conn, ref mut core| {

        let query = Query::new(
            "FOR i IN 1..10 \
              INSERT { \
                name: CONCAT('No.', i), \
                age: i + 21 \
              } IN cursor_customers02 \
              RETURN NEW.name"
        );

        let method = CreateCursor::<String>::from_query(query);
        let work = conn.execute(method);
        let cursor = core.run(work).unwrap();

        assert!(cursor.result().contains(&"No.1".to_owned()));
        assert!(cursor.result().contains(&"No.2".to_owned()));
        assert!(cursor.result().contains(&"No.3".to_owned()));
        assert!(cursor.result().contains(&"No.4".to_owned()));
        assert!(cursor.result().contains(&"No.5".to_owned()));
        assert!(cursor.result().contains(&"No.6".to_owned()));
        assert!(cursor.result().contains(&"No.7".to_owned()));
        assert!(cursor.result().contains(&"No.8".to_owned()));
        assert!(cursor.result().contains(&"No.9".to_owned()));
        assert!(cursor.result().contains(&"No.10".to_owned()));
        assert_eq!(10, cursor.result().len());
        assert_eq!(10, cursor.extra().unwrap().stats().writes_executed());
        assert_eq!(false, cursor.has_more());
        assert_eq!(None, cursor.id());
    });
}

#[test]
fn query_reads_from_cursor_in_batches_of_5_results() {
    arango_test_with_document_collection("cursor_customers03", |conn, ref mut core| {

        let inserts = Query::new(
            "FOR i IN 1..21 \
              INSERT { \
                name: CONCAT('No.', i), \
                age: i + 21 \
              } IN cursor_customers03"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();

        let query = Query::new(
            "FOR c IN cursor_customers03 \
              FILTER c.age <= 37 \
              SORT c.name \
              RETURN c.name"
        );
        let mut new_cursor = NewCursor::from(query);
        new_cursor.set_batch_size(5);
        let method = CreateCursor::<String>::new(new_cursor);
        let cursor = core.run(conn.execute(method)).unwrap();
        assert_eq!(21, cursor.extra().unwrap().stats().scanned_full());
        assert_eq!(5, cursor.extra().unwrap().stats().filtered());

        assert!(cursor.result().contains(&"No.1".to_owned()));
        assert!(cursor.result().contains(&"No.10".to_owned()));
        assert!(cursor.result().contains(&"No.11".to_owned()));
        assert!(cursor.result().contains(&"No.12".to_owned()));
        assert!(cursor.result().contains(&"No.13".to_owned()));
        assert_eq!(5, cursor.result().len());
        assert_eq!(true, cursor.has_more());
        assert_eq!(21, cursor.extra().unwrap().stats().scanned_full());
        assert_eq!(5, cursor.extra().unwrap().stats().filtered());
        let cursor_id = cursor.id().unwrap();

        let method = ReadNextBatchFromCursor::with_id_ref(cursor.id().unwrap());
        let cursor = core.run(conn.execute(method)).unwrap();

        assert!(cursor.result().contains(&"No.14".to_owned()));
        assert!(cursor.result().contains(&"No.15".to_owned()));
        assert!(cursor.result().contains(&"No.16".to_owned()));
        assert!(cursor.result().contains(&"No.2".to_owned()));
        assert!(cursor.result().contains(&"No.3".to_owned()));
        assert_eq!(5, cursor.result().len());
        assert_eq!(true, cursor.has_more());
        assert_eq!(cursor_id, cursor.id().unwrap());
        assert_eq!(21, cursor.extra().unwrap().stats().scanned_full());
        assert_eq!(5, cursor.extra().unwrap().stats().filtered());

        let method = ReadNextBatchFromCursor::with_id_ref(cursor.id().unwrap());
        let cursor = core.run(conn.execute(method)).unwrap();

        assert!(cursor.result().contains(&"No.4".to_owned()));
        assert!(cursor.result().contains(&"No.5".to_owned()));
        assert!(cursor.result().contains(&"No.6".to_owned()));
        assert!(cursor.result().contains(&"No.7".to_owned()));
        assert!(cursor.result().contains(&"No.8".to_owned()));
        assert_eq!(5, cursor.result().len());
        assert_eq!(true, cursor.has_more());
        assert_eq!(cursor_id, cursor.id().unwrap());
        assert_eq!(21, cursor.extra().unwrap().stats().scanned_full());
        assert_eq!(5, cursor.extra().unwrap().stats().filtered());

        let method = ReadNextBatchFromCursor::with_id_ref(cursor.id().unwrap());
        let cursor = core.run(conn.execute(method)).unwrap();

        assert!(cursor.result().contains(&"No.9".to_owned()));
        assert_eq!(1, cursor.result().len());
        assert_eq!(false, cursor.has_more());
        assert_eq!(None, cursor.id());
        assert_eq!(21, cursor.extra().unwrap().stats().scanned_full());
        assert_eq!(5, cursor.extra().unwrap().stats().filtered());
    });
}

#[test]
fn delete_cursor_before_fetching_all_results() {
    arango_test_with_document_collection("cursor_customers04", |conn, ref mut core| {

        let inserts = Query::new(
            "FOR i IN 1..21 \
              INSERT { \
                name: CONCAT('No.', i), \
                age: i + 21 \
              } IN cursor_customers04"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();

        let query = Query::new(
            "FOR c IN cursor_customers04 \
              FILTER c.age <= 37 \
              SORT c.name \
              RETURN c.name"
        );
        let mut new_cursor = NewCursor::from(query);
        new_cursor.set_batch_size(5);
        let method = CreateCursor::<String>::new(new_cursor);
        let cursor = core.run(conn.execute(method)).unwrap();
        assert_eq!(21, cursor.extra().unwrap().stats().scanned_full());
        assert_eq!(5, cursor.extra().unwrap().stats().filtered());

        assert!(cursor.result().contains(&"No.1".to_owned()));
        assert!(cursor.result().contains(&"No.10".to_owned()));
        assert!(cursor.result().contains(&"No.11".to_owned()));
        assert!(cursor.result().contains(&"No.12".to_owned()));
        assert!(cursor.result().contains(&"No.13".to_owned()));
        assert_eq!(5, cursor.result().len());
        assert_eq!(true, cursor.has_more());
        assert_eq!(21, cursor.extra().unwrap().stats().scanned_full());
        assert_eq!(5, cursor.extra().unwrap().stats().filtered());
        let cursor_id = cursor.id().unwrap();

        let method = ReadNextBatchFromCursor::with_id_ref(cursor.id().unwrap());
        let cursor = core.run(conn.execute(method)).unwrap();

        assert!(cursor.result().contains(&"No.14".to_owned()));
        assert!(cursor.result().contains(&"No.15".to_owned()));
        assert!(cursor.result().contains(&"No.16".to_owned()));
        assert!(cursor.result().contains(&"No.2".to_owned()));
        assert!(cursor.result().contains(&"No.3".to_owned()));
        assert_eq!(5, cursor.result().len());
        assert_eq!(true, cursor.has_more());
        assert_eq!(cursor_id, cursor.id().unwrap());
        assert_eq!(21, cursor.extra().unwrap().stats().scanned_full());
        assert_eq!(5, cursor.extra().unwrap().stats().filtered());

        let method = DeleteCursor::with_id_ref(cursor.id().unwrap());
        let deleted = core.run(conn.execute(method)).unwrap();

        assert_eq!(EMPTY, deleted);

        let method = ReadNextBatchFromCursor::<String>::with_id_ref(cursor.id().unwrap());
        let result = core.run(conn.execute(method));

        match result {
            Err(Error::Method(error)) => {
                assert_eq!(404, error.status_code());
                assert_eq!(ErrorCode::CursorNotFound, error.error_code());
                assert_eq!("cursor not found", error.message());
            },
            _ => panic!("Error::ApiError expected but got {:?}", result),
        };
    });
}

#[test]
fn query_with_optimizer_rules() {
    arango_test_with_document_collection("cursor_customers05", |conn, ref mut core| {

        let query = Query::new("FOR c IN cursor_customers05 RETURN c");

        let mut new_cursor = NewCursor::from(query);

            new_cursor.options_mut().optimizer_mut().rules_mut()
                .exclude(OptimizerRule::All)
                .include(OptimizerRule::InterchangeAdjacentEnumerations)
                .include(OptimizerRule::InlineSubQueries)
                .include(OptimizerRule::MoveFiltersUp)
                .exclude(OptimizerRule::PropagateConstantAttributes)
            ;

        let method = CreateCursor::<JsonValue>::new(new_cursor);
        let cursor = core.run(conn.execute(method)).unwrap();

        assert!(cursor.result().is_empty());
        assert_eq!(false, cursor.has_more());
        assert_eq!(None, cursor.id());
    });
}
