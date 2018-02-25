
#[macro_use] extern crate serde_json;
extern crate tokio_core;

extern crate rincon_core;
extern crate rincon_connector;
extern crate rincon_client;
extern crate rincon_test_helper;

use std::env;

use rincon_core::api::connector::{Error, Execute};
use rincon_core::api::ErrorCode;
use rincon_core::api::query::Query;
use rincon_core::api::types::{Empty, JsonValue};
use rincon_client::index::methods::CreateIndex;
use rincon_client::index::types::{HashIndex, IndexDetails, IndexIdOption, IndexKey, NewHashIndex};
use rincon_client::aql::methods::*;
use rincon_client::aql::types::*;
use rincon_client::cursor::methods::CreateCursor;

use rincon_test_helper::*;


#[test]
fn parse_valid_query() {
    arango_test_with_document_collection("aql_customers01", |conn, ref mut core| {
        let inserts = Query::new(
            "FOR i IN 1..21 \
              INSERT { \
                name: CONCAT('No.', i), \
                age: i + 21 \
              } IN aql_customers01"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();

        let query = String::from(
            "FOR c IN aql_customers01 \
              FILTER c.age <= @age \
              LIMIT 10 \
              SORT c.name \
              RETURN c.name"
        );

        let method = ParseQuery::from_query(query);
        let parsed_query = core.run(conn.execute(method)).unwrap();

        let query_ast = ParsedQuery::new(
            vec!["aql_customers01"],
            vec!["age"],
            vec![
                ParsedAstNode::new(
                    "root",
                    None,
                    None,
                    None,
                    vec![
                        ParsedAstNode::new(
                            "for",
                            None,
                            None,
                            None,
                            vec![
                                ParsedAstNode::new(
                                    "variable",
                                    "c".to_owned(),
                                    AstNodeId(0),
                                    None,
                                    vec![]
                                ),
                                ParsedAstNode::new(
                                    "collection",
                                    "aql_customers01".to_owned(),
                                    None,
                                    None,
                                    vec![]
                                ),
                            ]
                        ),
                        ParsedAstNode::new(
                            "filter",
                            None,
                            None,
                            None,
                            vec![
                                ParsedAstNode::new(
                                    "compare <=",
                                    None,
                                    None,
                                    None,
                                    vec![
                                        ParsedAstNode::new(
                                            "attribute access",
                                            "age".to_owned(),
                                            None,
                                            None,
                                            vec![
                                                ParsedAstNode::new(
                                                    "reference",
                                                    "c".to_owned(),
                                                    AstNodeId(0),
                                                    None,
                                                    vec![]
                                                )
                                            ]
                                        ),
                                        ParsedAstNode::new(
                                            "parameter",
                                            "age".to_owned(),
                                            None,
                                            None,
                                            vec![]
                                        ),
                                    ]
                                )
                            ]
                        ),
                        ParsedAstNode::new(
                            "limit",
                            None,
                            None,
                            None,
                            vec![
                                ParsedAstNode::new(
                                    "value",
                                    None,
                                    None,
                                    JsonValue::from(0),
                                    vec![]
                                ),
                                ParsedAstNode::new(
                                    "value",
                                    None,
                                    None,
                                    JsonValue::from(10),
                                    vec![]
                                )
                            ]
                        ),
                        ParsedAstNode::new(
                            "sort",
                            None,
                            None,
                            None,
                            vec![
                                ParsedAstNode::new(
                                    "array",
                                    None,
                                    None,
                                    None,
                                    vec![
                                        ParsedAstNode::new(
                                            "sort element",
                                            None,
                                            None,
                                            None,
                                            vec![
                                                ParsedAstNode::new(
                                                    "attribute access",
                                                    "name".to_owned(),
                                                    None,
                                                    None,
                                                    vec![
                                                        ParsedAstNode::new(
                                                            "reference",
                                                            "c".to_owned(),
                                                            AstNodeId(0),
                                                            None,
                                                            vec![]
                                                        )
                                                    ]
                                                ),
                                                ParsedAstNode::new(
                                                    "value",
                                                    None,
                                                    None,
                                                    JsonValue::from(true),
                                                    vec![]
                                                ),
                                            ]
                                        )
                                    ]
                                )
                            ]
                        ),
                        ParsedAstNode::new(
                            "return",
                            None,
                            None,
                            None,
                            vec![
                                ParsedAstNode::new(
                                    "attribute access",
                                    "name".to_owned(),
                                    None,
                                    None,
                                    vec![
                                        ParsedAstNode::new(
                                            "reference",
                                            "c".to_owned(),
                                            AstNodeId(0),
                                            None,
                                            vec![]
                                        )
                                    ]
                                )
                            ]
                        )
                    ]
                )
            ]
        );

        assert_eq!(query_ast, parsed_query);

    });
}

#[test]
fn parse_invalid_query() {
    arango_test_with_document_collection("aql_customers02", |conn, ref mut core| {
        let inserts = Query::new(
            "FOR i IN 1..21 \
              INSERT { \
                name: CONCAT('No.', i), \
                age: i + 21 \
              } IN aql_customers02"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();

        let query = String::from(
            "FOR c IN aql_customers02 \
              FILTER c.age = @age \
              LIMIT 2 \
              SORT c.name \
              RETURN c.name"
        );

        let method = ParseQuery::from_query(query);
        let result = core.run(conn.execute(method));

        match result {
            Err(Error::Method(error)) => {
                assert_eq!(400, error.status_code());
                assert_eq!(ErrorCode::QueryParse, error.error_code());
                assert_eq!("syntax error, unexpected assignment near '= @age LIMIT 2 SORT c.name RETUR...' at position 1:39", error.message());
            },
            _ => panic!("Error::ApiError expected but got {:?}", result),
        };
    });
}

#[test]
fn explain_valid_query() {
    arango_test_with_document_collection("aql_customers03", |conn, ref mut core| {
        let database = env::var(ENV_ARANGO_TEST_DATABASE).unwrap();
        let inserts = Query::new(
            "FOR i IN 1..21 \
              INSERT { \
                name: CONCAT('No.', i), \
                age: i + 21 \
              } IN aql_customers03"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();

        let query = Query::new(
            "FOR c IN aql_customers03 \
              RETURN c"
        );

        let method = ExplainQuery::with_defaults(query);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_best_plan(
            ExecutionPlan::new(
                vec![
                    ExecutionNode::Singleton(SingletonNode::new(
                        ExecutionNodeId(1),
                        vec![],
                        1.,
                        1,
                    )),
                    ExecutionNode::EnumerateCollection(EnumerateCollectionNode::new(
                        ExecutionNodeId(2),
                        vec![ ExecutionNodeId(1) ],
                        23.,
                        21,
                        database.clone(),
                        "aql_customers03",
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                        false,
                    )),
                    ExecutionNode::Return(ReturnNode::new(
                        ExecutionNodeId(3),
                        vec![ ExecutionNodeId(2) ],
                        44.,
                        21,
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                    )),
                ],
                Vec::<String>::new(),
                vec![
                    ExecutionCollection::new("read", "aql_customers03"),
                ],
                vec![
                    ExecutionVariable::new(ExecutionVariableId(0), "c"),
                ],
                44.,
                21,
            ),
            Vec::<String>::new(),
            ExecutionStats::new(29, 0, 1),
            true,
        );

        assert_eq!(explanation, explained_query);

    });
}

#[test]
fn explain_a_plan_with_some_optimizer_rules_applied() {
    arango_test_with_document_collection("aql_customers04", |conn, ref mut core| {
        let database = env::var(ENV_ARANGO_TEST_DATABASE).unwrap();
        let inserts = Query::new(
            "FOR i IN 1..21 \
              INSERT { \
                id: i, \
                name: CONCAT('No.', i), \
                age: i + 21 \
              } IN aql_customers04"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();
        let id_index = NewHashIndex::new(vec!["id".to_owned()], true, false, false);
        let index = core.run(conn.execute(CreateIndex::new("aql_customers04", id_index))).unwrap();
        let index_id = match *index.id() {
            IndexIdOption::Qualified(ref index_id) => index_id,
            _ => panic!("Qualified index id expected!"),
        };

        let query = Query::new(
            "FOR c IN aql_customers04 \
              LET id = c.id \
              FILTER id == 21 \
              LET name = c.name \
              SORT c.id \
              LIMIT 1 \
              RETURN name"
        );

        let method = ExplainQuery::with_defaults(query);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_best_plan(
            ExecutionPlan::new(
                vec![
                    ExecutionNode::Singleton(SingletonNode::new(
                        ExecutionNodeId(1),
                        vec![],
                        1.,
                        1,
                    )),
                    ExecutionNode::Index(IndexNode::new(
                        ExecutionNodeId(11),
                        vec![ ExecutionNodeId(1) ],
                        1.95,
                        1,
                        database.clone(),
                        "aql_customers04",
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                        vec![
                            HashIndex::new(
                                IndexKey::new(index_id.index_key()).into(),
                                vec![ "id" ],
                                true,
                                false,
                                false,
                                1,
                            ).into(),
                        ],
                        ExecutionExpression::new(
                            "n-ary or",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "n-ary and",
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "compare ==",
                                            None,
                                            None,
                                            None,
                                            None,
                                            None,
                                            vec![],
                                            vec![
                                                ExecutionExpression::new(
                                                    "attribute access",
                                                    "id".to_owned(),
                                                    None,
                                                    None,
                                                    None,
                                                    None,
                                                    vec![],
                                                    vec![
                                                        ExecutionExpression::new(
                                                            "reference",
                                                            "c".to_owned(),
                                                            ExecutionExpressionId(0),
                                                            None,
                                                            None,
                                                            None,
                                                            vec![],
                                                            vec![],
                                                        )
                                                    ],
                                                ),
                                                ExecutionExpression::new(
                                                    "value",
                                                    None,
                                                    None,
                                                    json!(21),
                                                    None,
                                                    None,
                                                    vec![],
                                                    vec![],
                                                ),
                                            ],
                                        ),
                                    ],
                                )
                            ],
                        ),
                        false,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(4),
                        vec![ ExecutionNodeId(11) ],
                        2.95,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(4), "3"),
                        "simple",
                        ExecutionExpression::new(
                            "compare ==",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "attribute access",
                                    "id".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "reference",
                                            "c".to_owned(),
                                            ExecutionExpressionId(0),
                                            None,
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        ),
                                    ],
                                ),
                                ExecutionExpression::new(
                                    "value",
                                    None,
                                    None,
                                    json!(21),
                                    None,
                                    None,
                                    vec![],
                                    vec![],
                                ),
                            ],
                        ),
                        false,
                    )),
                    ExecutionNode::Filter(FilterNode::new(
                        ExecutionNodeId(5),
                        vec![ ExecutionNodeId(4) ],
                        3.95,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(4), "3"),
                    )),
                    ExecutionNode::Limit(LimitNode::new(
                        ExecutionNodeId(9),
                        vec![ ExecutionNodeId(5) ],
                        4.95,
                        1,
                        0,
                        1,
                        false,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(6),
                        vec![ ExecutionNodeId(9) ],
                        5.95,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(2), "name"),
                        "attribute",
                        ExecutionExpression::new(
                            "attribute access",
                            "name".to_owned(),
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "reference",
                                    "c".to_owned(),
                                    ExecutionExpressionId(0),
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![],
                                ),
                            ],
                        ),
                        false,
                    )),
                    ExecutionNode::Return(ReturnNode::new(
                        ExecutionNodeId(10),
                        vec![ ExecutionNodeId(6) ],
                        6.95,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(2), "name"),
                    )),
                ],
                vec![
                    "move-calculations-up",
                    "remove-redundant-calculations",
                    "remove-unnecessary-calculations",
                    "move-calculations-up-2",
                    "use-indexes",
                    "use-index-for-sort",
                    "remove-unnecessary-calculations-2",
                    "move-calculations-down",
                ],
                vec![
                    ExecutionCollection::new("read", "aql_customers04"),
                ],
                vec![
                    ExecutionVariable::new(ExecutionVariableId(6), "5"),
                    ExecutionVariable::new(ExecutionVariableId(4), "3"),
                    ExecutionVariable::new(ExecutionVariableId(2), "name"),
                    ExecutionVariable::new(ExecutionVariableId(1), "id"),
                    ExecutionVariable::new(ExecutionVariableId(0), "c"),
                ],
                6.95,
                1,
            ),
            Vec::<String>::new(),
            ExecutionStats::new(29, 0, 1),
            true,
        );

        assert_eq!(explanation, explained_query);

    });
}

#[test]
fn explain_2_plans_with_some_optimizer_rules_specified() {
    arango_test_with_document_collection("aql_customers05", |conn, ref mut core| {
        let database = env::var(ENV_ARANGO_TEST_DATABASE).unwrap();
        let inserts = Query::new(
            "FOR i IN 1..21 \
              INSERT { \
                id: i, \
                name: CONCAT('No.', i), \
                age: i + 21 \
              } IN aql_customers05"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();
        let id_index = NewHashIndex::new(vec!["id".to_owned()], true, false, false);
        core.run(conn.execute(CreateIndex::new("aql_customers05", id_index))).unwrap();

        let query = Query::new(
            "FOR c IN aql_customers05 \
              LET id = c.id \
              FILTER id == 21 \
              LET name = c.name \
              SORT c.id \
              LIMIT 1 \
              RETURN name"
        );

        let mut new_explain_query = NewExplainQuery::from(query);
        new_explain_query.options_mut().set_all_plans(true);
        new_explain_query.options_mut().set_max_number_of_plans(2);
        new_explain_query.options_mut().optimizer_mut().rules_mut()
            .exclude(OptimizerRule::All)
            .include(OptimizerRule::UseIndexForSort)
//            .include(OptimizerRule::Custom("use-index-range".to_owned()))
            ;
        let method = ExplainQuery::new(new_explain_query);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_multiple_plans(
            vec![
                ExecutionPlan::new(
                    vec![
                        ExecutionNode::Singleton(SingletonNode::new(
                            ExecutionNodeId(1),
                            vec![],
                            1.,
                            1,
                        )),
                        ExecutionNode::EnumerateCollection(EnumerateCollectionNode::new(
                            ExecutionNodeId(2),
                            vec![ ExecutionNodeId(1) ],
                            23.,
                            21,
                            database.clone(),
                            "aql_customers05",
                            ExecutionVariable::new(ExecutionVariableId(0), "c"),
                            false,
                        )),
                        ExecutionNode::Calculation(CalculationNode::new(
                            ExecutionNodeId(3),
                            vec![ ExecutionNodeId(2) ],
                            44.,
                            21,
                            ExecutionVariable::new(ExecutionVariableId(1), "id"),
                            "attribute",
                            ExecutionExpression::new(
                                "attribute access",
                                "id".to_owned(),
                                None,
                                None,
                                None,
                                None,
                                vec![],
                                vec![
                                    ExecutionExpression::new(
                                        "reference",
                                        "c".to_owned(),
                                        ExecutionExpressionId(0),
                                        None,
                                        None,
                                        None,
                                        vec![],
                                        vec![],
                                    ),
                                ],
                            ),
                            false,
                        )),
                        ExecutionNode::Calculation(CalculationNode::new(
                            ExecutionNodeId(4),
                            vec![ ExecutionNodeId(3) ],
                            65.,
                            21,
                            ExecutionVariable::new(ExecutionVariableId(4), "3"),
                            "simple",
                            ExecutionExpression::new(
                                "compare ==",
                                None,
                                None,
                                None,
                                None,
                                None,
                                vec![],
                                vec![
                                    ExecutionExpression::new(
                                        "reference",
                                        "id".to_owned(),
                                        ExecutionExpressionId(1),
                                        None,
                                        None,
                                        None,
                                        vec![],
                                        vec![],
                                    ),
                                    ExecutionExpression::new(
                                        "value",
                                        None,
                                        None,
                                        json!(21),
                                        None,
                                        None,
                                        vec![],
                                        vec![],
                                    ),
                                ],
                            ),
                            false,
                        )),
                        ExecutionNode::Filter(FilterNode::new(
                            ExecutionNodeId(5),
                            vec![ ExecutionNodeId(4) ],
                            86.,
                            21,
                            ExecutionVariable::new(ExecutionVariableId(4), "3"),
                        )),
                        ExecutionNode::Calculation(CalculationNode::new(
                            ExecutionNodeId(6),
                            vec![ ExecutionNodeId(5) ],
                            107.,
                            21,
                            ExecutionVariable::new(ExecutionVariableId(2), "name"),
                            "attribute",
                            ExecutionExpression::new(
                                "attribute access",
                                "name".to_owned(),
                                None,
                                None,
                                None,
                                None,
                                vec![],
                                vec![
                                    ExecutionExpression::new(
                                        "reference",
                                        "c".to_owned(),
                                        ExecutionExpressionId(0),
                                        None,
                                        None,
                                        None,
                                        vec![],
                                        vec![],
                                    ),
                                ],
                            ),
                            false,
                        )),
                        ExecutionNode::Calculation(CalculationNode::new(
                            ExecutionNodeId(7),
                            vec![ ExecutionNodeId(6) ],
                            128.,
                            21,
                            ExecutionVariable::new(ExecutionVariableId(6), "5"),
                            "attribute",
                            ExecutionExpression::new(
                                "attribute access",
                                "id".to_owned(),
                                None,
                                None,
                                None,
                                None,
                                vec![],
                                vec![
                                    ExecutionExpression::new(
                                        "reference",
                                        "c".to_owned(),
                                        ExecutionExpressionId(0),
                                        None,
                                        None,
                                        None,
                                        vec![],
                                        vec![],
                                    ),
                                ],
                            ),
                            false,
                        )),
                        ExecutionNode::Sort(SortNode::new(
                            ExecutionNodeId(8),
                            vec![ ExecutionNodeId(7) ],
                            220.23866587835397,
                            21,
                        )),
                        ExecutionNode::Limit(LimitNode::new(
                            ExecutionNodeId(9),
                            vec![ ExecutionNodeId(8) ],
                            221.23866587835397,
                            1,
                            0,
                            1,
                            false,
                        )),
                        ExecutionNode::Return(ReturnNode::new(
                            ExecutionNodeId(10),
                            vec![ ExecutionNodeId(9) ],
                            222.23866587835397,
                            1,
                            ExecutionVariable::new(ExecutionVariableId(2), "name"),
                        )),
                    ],
                    Vec::<String>::new(),
//                    vec![
//                        "use-index-for-sort",
//                    ],
                    vec![
                        ExecutionCollection::new("read", "aql_customers05"),
                    ],
                    vec![
                        ExecutionVariable::new(ExecutionVariableId(6), "5"),
                        ExecutionVariable::new(ExecutionVariableId(4), "3"),
                        ExecutionVariable::new(ExecutionVariableId(2), "name"),
                        ExecutionVariable::new(ExecutionVariableId(1), "id"),
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                    ],
                    222.23866587835397,
                    1,
                ),
            ],
            Vec::<String>::new(),
            ExecutionStats::new(1, 28, 1),
        );

        assert_eq!(explanation, explained_query);

    });
}

#[test]
fn explain_query_with_limit_and_offset() {
    arango_test_with_document_collection("aql_customers06", |conn, ref mut core| {
        let database = env::var(ENV_ARANGO_TEST_DATABASE).unwrap();
        let inserts = Query::new(
            "FOR i IN 1..21 \
              INSERT { \
                name: CONCAT('No.', i), \
                age: i + 21 \
              } IN aql_customers06"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();

        let query = Query::new(
            "FOR c IN aql_customers06 \
              LIMIT 2, 5 \
              RETURN c"
        );

        let method = ExplainQuery::with_defaults(query);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_best_plan(
            ExecutionPlan::new(
                vec![
                    ExecutionNode::Singleton(SingletonNode::new(
                        ExecutionNodeId(1),
                        vec![],
                        1.,
                        1,
                    )),
                    ExecutionNode::EnumerateCollection(EnumerateCollectionNode::new(
                        ExecutionNodeId(2),
                        vec![ ExecutionNodeId(1) ],
                        23.,
                        21,
                        database.clone(),
                        "aql_customers06",
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                        false,
                    )),
                    ExecutionNode::Limit(LimitNode::new(
                        ExecutionNodeId(3),
                        vec![ ExecutionNodeId(2) ],
                        28.,
                        5,
                        2,
                        5,
                        false,
                    )),
                    ExecutionNode::Return(ReturnNode::new(
                        ExecutionNodeId(4),
                        vec![ ExecutionNodeId(3) ],
                        33.,
                        5,
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                    )),
                ],
                Vec::<String>::new(),
                vec![
                    ExecutionCollection::new("read", "aql_customers06"),
                ],
                vec![
                    ExecutionVariable::new(ExecutionVariableId(0), "c"),
                ],
                33.,
                5,
            ),
            Vec::<String>::new(),
            ExecutionStats::new(29, 0, 1),
            true,
        );

        assert_eq!(explanation, explained_query);

    });
}

#[test]
fn explain_query_with_basic_collect() {
    arango_test_with_document_collection("aql_customers07", |conn, ref mut core| {
        let database = env::var(ENV_ARANGO_TEST_DATABASE).unwrap();
        let inserts = Query::new(
            "FOR i IN 1..21 \
              INSERT { \
                name: CONCAT('No.', i), \
                age: i + 21 \
              } IN aql_customers07"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();

        let query = Query::new(
            "FOR c IN aql_customers07 \
              COLLECT age = c.age \
              RETURN { \
                \"age\": age \
              }"
        );

        let method = ExplainQuery::with_defaults(query);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_best_plan(
            ExecutionPlan::new(
                vec![
                    ExecutionNode::Singleton(SingletonNode::new(
                        ExecutionNodeId(1),
                        vec![],
                        1.,
                        1,
                    )),
                    ExecutionNode::EnumerateCollection(EnumerateCollectionNode::new(
                        ExecutionNodeId(2),
                        vec![ ExecutionNodeId(1) ],
                        23.,
                        21,
                        database.clone(),
                        "aql_customers07",
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                        false,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(3),
                        vec![ ExecutionNodeId(2) ],
                        44.,
                        21,
                        ExecutionVariable::new(ExecutionVariableId(3), "2"),
                        "attribute",
                        ExecutionExpression::new(
                            "attribute access",
                            "age".to_owned(),
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "reference",
                                    "c".to_owned(),
                                    ExecutionExpressionId(0),
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![],
                                )
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::Aggregate(AggregateNode::new(
                        ExecutionNodeId(4),
                        vec![ ExecutionNodeId(3) ],
                        60.,
                        16,
                        None,
                        vec![
                            ExecutionGroup::new(
                                ExecutionVariable::new(ExecutionVariableId(3), "2"),
                                ExecutionVariable::new(ExecutionVariableId(1), "age"),
                            ),
                        ],
                        vec![],
                        CollectOptions::new(
                            CollectMethod::Hash,
                        )
                    )),
                    ExecutionNode::Sort(SortNode::new(
                        ExecutionNodeId(7),
                        vec![ ExecutionNodeId(4) ],
                        124.,
                        16,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(5),
                        vec![ ExecutionNodeId(7) ],
                        140.,
                        16,
                        ExecutionVariable::new(ExecutionVariableId(5), "4"),
                        "simple",
                        ExecutionExpression::new(
                            "object",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "object element",
                                    "age".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "reference",
                                            "age".to_owned(),
                                            ExecutionExpressionId(1),
                                            None,
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        )
                                    ],
                                ),
                            ],
                        ),
                        false
                    )),
                    ExecutionNode::Return(ReturnNode::new(
                        ExecutionNodeId(6),
                        vec![ ExecutionNodeId(5) ],
                        156.,
                        16,
                        ExecutionVariable::new(ExecutionVariableId(5), "4"),
                    )),
                ],
                vec![
                    "move-calculations-up",
                    "move-calculations-down",
                ],
                vec![
                    ExecutionCollection::new("read", "aql_customers07"),
                ],
                vec![
                    ExecutionVariable::new(ExecutionVariableId(5), "4"),
                    ExecutionVariable::new(ExecutionVariableId(3), "2"),
                    ExecutionVariable::new(ExecutionVariableId(1), "age"),
                    ExecutionVariable::new(ExecutionVariableId(0), "c"),
                ],
                156.,
                16,
            ),
            Vec::<String>::new(),
            ExecutionStats::new(58, 0, 2),
            true,
        );

        assert_eq!(explanation, explained_query);

    });
}

#[test]
fn explain_query_with_collect_into_group_variable() {
    arango_test_with_document_collection("aql_customers08", |conn, ref mut core| {
        let database = env::var(ENV_ARANGO_TEST_DATABASE).unwrap();
        let inserts = Query::new(
            "FOR i IN 1..21 \
              INSERT { \
                name: CONCAT('No.', i), \
                age: i + 21 \
              } IN aql_customers08"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();

        let query = Query::new(
            "FOR c IN aql_customers08 \
              COLLECT age = c.age INTO groups \
              RETURN { \
                \"age\": age, \
                \"customersWithAge\": groups \
              }"
        );

        let method = ExplainQuery::with_defaults(query);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_best_plan(
            ExecutionPlan::new(
                vec![
                    ExecutionNode::Singleton(SingletonNode::new(
                        ExecutionNodeId(1),
                        vec![],
                        1.,
                        1,
                    )),
                    ExecutionNode::EnumerateCollection(EnumerateCollectionNode::new(
                        ExecutionNodeId(2),
                        vec![ ExecutionNodeId(1) ],
                        23.,
                        21,
                        database.clone(),
                        "aql_customers08",
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                        false,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(3),
                        vec![ ExecutionNodeId(2) ],
                        44.,
                        21,
                        ExecutionVariable::new(ExecutionVariableId(4), "3"),
                        "attribute",
                        ExecutionExpression::new(
                            "attribute access",
                            "age".to_owned(),
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "reference",
                                    "c".to_owned(),
                                    ExecutionExpressionId(0),
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![],
                                )
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::Sort(SortNode::new(
                        ExecutionNodeId(7),
                        vec![ ExecutionNodeId(3) ],
                        136.23866587835397,
                        21,
                    )),
                    ExecutionNode::Aggregate(AggregateNode::new(
                        ExecutionNodeId(4),
                        vec![ ExecutionNodeId(7) ],
                        152.23866587835397,
                        16,
                        ExecutionVariable::new(ExecutionVariableId(2), "groups"),
                        vec![
                            ExecutionGroup::new(
                                ExecutionVariable::new(ExecutionVariableId(4), "3"),
                                ExecutionVariable::new(ExecutionVariableId(1), "age"),
                            ),
                        ],
                        vec![],
                        CollectOptions::new(
                            CollectMethod::Sorted,
                        )
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(5),
                        vec![ ExecutionNodeId(4) ],
                        168.23866587835397,
                        16,
                        ExecutionVariable::new(ExecutionVariableId(6), "5"),
                        "simple",
                        ExecutionExpression::new(
                            "object",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "object element",
                                    "age".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "reference",
                                            "age".to_owned(),
                                            ExecutionExpressionId(1),
                                            None,
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        )
                                    ],
                                ),
                                ExecutionExpression::new(
                                    "object element",
                                    "customersWithAge".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "reference",
                                            "groups".to_owned(),
                                            ExecutionExpressionId(2),
                                            None,
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        )
                                    ],
                                )
                            ],
                        ),
                        false
                    )),
                    ExecutionNode::Return(ReturnNode::new(
                        ExecutionNodeId(6),
                        vec![ ExecutionNodeId(5) ],
                        184.23866587835397,
                        16,
                        ExecutionVariable::new(ExecutionVariableId(6), "5"),
                    )),
                ],
                Vec::<String>::new(),
                vec![
                    ExecutionCollection::new("read", "aql_customers08"),
                ],
                vec![
                    ExecutionVariable::new(ExecutionVariableId(6), "5"),
                    ExecutionVariable::new(ExecutionVariableId(4), "3"),
                    ExecutionVariable::new(ExecutionVariableId(2), "groups"),
                    ExecutionVariable::new(ExecutionVariableId(1), "age"),
                    ExecutionVariable::new(ExecutionVariableId(0), "c"),
                ],
                184.23866587835397,
                16,
            ),
            Vec::<String>::new(),
            ExecutionStats::new(29, 0, 1),
            true,
        );

        assert_eq!(explanation, explained_query);

    });
}

#[test]
fn explain_query_with_collect_multiple_criteria() {
    arango_test_with_document_collection("aql_customers09", |conn, ref mut core| {
        let database = env::var(ENV_ARANGO_TEST_DATABASE).unwrap();
        let inserts = Query::new(
            "FOR i IN 1..21 \
              INSERT { \
                name: CONCAT('No.', i), \
                age: i + 21 \
              } IN aql_customers09"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();

        let query = Query::new(
            "FOR c IN aql_customers09 \
              COLLECT age = c.age, city = c.city INTO groups \
              RETURN { \
                \"age\": age, \
                \"city\": city, \
                \"customersWithAge\": groups \
              }"
        );

        let method = ExplainQuery::with_defaults(query);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_best_plan(
            ExecutionPlan::new(
                vec![
                    ExecutionNode::Singleton(SingletonNode::new(
                        ExecutionNodeId(1),
                        vec![],
                        1.,
                        1,
                    )),
                    ExecutionNode::EnumerateCollection(EnumerateCollectionNode::new(
                        ExecutionNodeId(2),
                        vec![ ExecutionNodeId(1) ],
                        23.,
                        21,
                        database.clone(),
                        "aql_customers09",
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                        false,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(3),
                        vec![ ExecutionNodeId(2) ],
                        44.,
                        21,
                        ExecutionVariable::new(ExecutionVariableId(5), "4"),
                        "attribute",
                        ExecutionExpression::new(
                            "attribute access",
                            "age".to_owned(),
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "reference",
                                    "c".to_owned(),
                                    ExecutionExpressionId(0),
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![],
                                )
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(4),
                        vec![ ExecutionNodeId(3) ],
                        65.,
                        21,
                        ExecutionVariable::new(ExecutionVariableId(7), "6"),
                        "attribute",
                        ExecutionExpression::new(
                            "attribute access",
                            "city".to_owned(),
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "reference",
                                    "c".to_owned(),
                                    ExecutionExpressionId(0),
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![],
                                )
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::Sort(SortNode::new(
                        ExecutionNodeId(8),
                        vec![ ExecutionNodeId(4) ],
                        157.23866587835397,
                        21,
                    )),
                    ExecutionNode::Aggregate(AggregateNode::new(
                        ExecutionNodeId(5),
                        vec![ ExecutionNodeId(8) ],
                        173.23866587835397,
                        16,
                        ExecutionVariable::new(ExecutionVariableId(3), "groups"),
                        vec![
                            ExecutionGroup::new(
                                ExecutionVariable::new(ExecutionVariableId(5), "4"),
                                ExecutionVariable::new(ExecutionVariableId(1), "age"),
                            ),
                            ExecutionGroup::new(
                                ExecutionVariable::new(ExecutionVariableId(7), "6"),
                                ExecutionVariable::new(ExecutionVariableId(2), "city"),
                            ),
                        ],
                        vec![],
                        CollectOptions::new(
                            CollectMethod::Sorted,
                        )
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(6),
                        vec![ ExecutionNodeId(5) ],
                        189.23866587835397,
                        16,
                        ExecutionVariable::new(ExecutionVariableId(9), "8"),
                        "simple",
                        ExecutionExpression::new(
                            "object",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "object element",
                                    "age".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "reference",
                                            "age".to_owned(),
                                            ExecutionExpressionId(1),
                                            None,
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        )
                                    ],
                                ),
                                ExecutionExpression::new(
                                    "object element",
                                    "city".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "reference",
                                            "city".to_owned(),
                                            ExecutionExpressionId(2),
                                            None,
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        )
                                    ],
                                ),
                                ExecutionExpression::new(
                                    "object element",
                                    "customersWithAge".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "reference",
                                            "groups".to_owned(),
                                            ExecutionExpressionId(3),
                                            None,
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        )
                                    ],
                                ),
                            ],
                        ),
                        false
                    )),
                    ExecutionNode::Return(ReturnNode::new(
                        ExecutionNodeId(7),
                        vec![ ExecutionNodeId(6) ],
                        205.23866587835397,
                        16,
                        ExecutionVariable::new(ExecutionVariableId(9), "8"),
                    )),
                ],
                vec![
                    "move-calculations-up",
                    "move-calculations-up-2",
                ],
                vec![
                    ExecutionCollection::new("read", "aql_customers09"),
                ],
                vec![
                    ExecutionVariable::new(ExecutionVariableId(9), "8"),
                    ExecutionVariable::new(ExecutionVariableId(7), "6"),
                    ExecutionVariable::new(ExecutionVariableId(5), "4"),
                    ExecutionVariable::new(ExecutionVariableId(3), "groups"),
                    ExecutionVariable::new(ExecutionVariableId(2), "city"),
                    ExecutionVariable::new(ExecutionVariableId(1), "age"),
                    ExecutionVariable::new(ExecutionVariableId(0), "c"),
                ],
                205.23866587835397,
                16,
            ),
            Vec::<String>::new(),
            ExecutionStats::new(29, 0, 1),
            true,
        );

        assert_eq!(explanation, explained_query);

    });
}

#[test]
fn explain_query_with_collect_count_aggregation() {
    arango_test_with_document_collection("aql_customers10", |conn, ref mut core| {
        let database = env::var(ENV_ARANGO_TEST_DATABASE).unwrap();
        let inserts = Query::new(
            "FOR i IN 1..21 \
              INSERT { \
                name: CONCAT('No.', i), \
                age: i + 21 \
              } IN aql_customers10"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();

        let query = Query::new(
            "FOR c IN aql_customers10 \
              COLLECT age = c.age WITH COUNT INTO num \
              RETURN { \
                \"age\": age, \
                \"count\": num
              }"
        );

        let method = ExplainQuery::with_defaults(query);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_best_plan(
            ExecutionPlan::new(
                vec![
                    ExecutionNode::Singleton(SingletonNode::new(
                        ExecutionNodeId(1),
                        vec![],
                        1.,
                        1,
                    )),
                    ExecutionNode::EnumerateCollection(EnumerateCollectionNode::new(
                        ExecutionNodeId(2),
                        vec![ ExecutionNodeId(1) ],
                        23.,
                        21,
                        database.clone(),
                        "aql_customers10",
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                        false,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(3),
                        vec![ ExecutionNodeId(2) ],
                        44.,
                        21,
                        ExecutionVariable::new(ExecutionVariableId(4), "3"),
                        "attribute",
                        ExecutionExpression::new(
                            "attribute access",
                            "age".to_owned(),
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "reference",
                                    "c".to_owned(),
                                    ExecutionExpressionId(0),
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![],
                                )
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::Aggregate(AggregateNode::new(
                        ExecutionNodeId(4),
                        vec![ ExecutionNodeId(3) ],
                        60.,
                        16,
                        ExecutionVariable::new(ExecutionVariableId(2), "num"),
                        vec![
                            ExecutionGroup::new(
                                ExecutionVariable::new(ExecutionVariableId(4), "3"),
                                ExecutionVariable::new(ExecutionVariableId(1), "age"),
                            ),
                        ],
                        vec![],
                        CollectOptions::new(
                            CollectMethod::Hash,
                        )
                    )),
                    ExecutionNode::Sort(SortNode::new(
                        ExecutionNodeId(7),
                        vec![ ExecutionNodeId(4) ],
                        124.,
                        16,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(5),
                        vec![ ExecutionNodeId(7) ],
                        140.,
                        16,
                        ExecutionVariable::new(ExecutionVariableId(6), "5"),
                        "simple",
                        ExecutionExpression::new(
                            "object",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "object element",
                                    "age".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "reference",
                                            "age".to_owned(),
                                            ExecutionExpressionId(1),
                                            None,
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        )
                                    ],
                                ),
                                ExecutionExpression::new(
                                    "object element",
                                    "count".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "reference",
                                            "num".to_owned(),
                                            ExecutionExpressionId(2),
                                            None,
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        )
                                    ],
                                )
                            ],
                        ),
                        false
                    )),
                    ExecutionNode::Return(ReturnNode::new(
                        ExecutionNodeId(6),
                        vec![ ExecutionNodeId(5) ],
                        156.,
                        16,
                        ExecutionVariable::new(ExecutionVariableId(6), "5"),
                    )),
                ],
                vec![
                    "move-calculations-up",
                    "move-calculations-down",
                ],
                vec![
                    ExecutionCollection::new("read", "aql_customers10"),
                ],
                vec![
                    ExecutionVariable::new(ExecutionVariableId(6), "5"),
                    ExecutionVariable::new(ExecutionVariableId(4), "3"),
                    ExecutionVariable::new(ExecutionVariableId(2), "num"),
                    ExecutionVariable::new(ExecutionVariableId(1), "age"),
                    ExecutionVariable::new(ExecutionVariableId(0), "c"),
                ],
                156.,
                16,
            ),
            Vec::<String>::new(),
            ExecutionStats::new(58, 0, 2),
            true,
        );

        assert_eq!(explanation, explained_query);

    });
}

#[test]
fn explain_query_with_collect_and_aggregation() {
    arango_test_with_document_collection("aql_customers11", |conn, ref mut core| {
        let database = env::var(ENV_ARANGO_TEST_DATABASE).unwrap();
        let inserts = Query::new(
            "FOR i IN 1..21 \
              INSERT { \
                name: CONCAT('No.', i), \
                age: i + 21 \
              } IN aql_customers11"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();

        let query = Query::new(
            "FOR c IN aql_customers11 \
              COLLECT AGGREGATE minAge = MIN(c.age), maxAge = MAX(c.age) \
              RETURN { \
                minAge, \
                maxAge \
              }"
        );

        let method = ExplainQuery::with_defaults(query);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_best_plan(
            ExecutionPlan::new(
                vec![
                    ExecutionNode::Singleton(SingletonNode::new(
                        ExecutionNodeId(1),
                        vec![],
                        1.,
                        1,
                    )),
                    ExecutionNode::EnumerateCollection(EnumerateCollectionNode::new(
                        ExecutionNodeId(2),
                        vec![ ExecutionNodeId(1) ],
                        23.,
                        21,
                        database.clone(),
                        "aql_customers11",
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                        false,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(3),
                        vec![ ExecutionNodeId(2) ],
                        44.,
                        21,
                        ExecutionVariable::new(ExecutionVariableId(4), "3"),
                        "attribute",
                        ExecutionExpression::new(
                            "attribute access",
                            "age".to_owned(),
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "reference",
                                    "c".to_owned(),
                                    ExecutionExpressionId(0),
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![],
                                )
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::Aggregate(AggregateNode::new(
                        ExecutionNodeId(5),
                        vec![ ExecutionNodeId(3) ],
                        60.,
                        16,
                        None,
                        vec![],
                        vec![
                            ExecutionAggregate::new(
                                "MIN",
                                ExecutionVariable::new(ExecutionVariableId(4), "3"),
                                ExecutionVariable::new(ExecutionVariableId(1), "minAge"),
                            ),
                            ExecutionAggregate::new(
                                "MAX",
                                ExecutionVariable::new(ExecutionVariableId(4), "3"),
                                ExecutionVariable::new(ExecutionVariableId(2), "maxAge"),
                            ),
                        ],
                        CollectOptions::new(
                            CollectMethod::Sorted,
                        )
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(6),
                        vec![ ExecutionNodeId(5) ],
                        76.,
                        16,
                        ExecutionVariable::new(ExecutionVariableId(8), "7"),
                        "simple",
                        ExecutionExpression::new(
                            "object",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "object element",
                                    "minAge".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "reference",
                                            "minAge".to_owned(),
                                            ExecutionExpressionId(1),
                                            None,
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        )
                                    ],
                                ),
                                ExecutionExpression::new(
                                    "object element",
                                    "maxAge".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "reference",
                                            "maxAge".to_owned(),
                                            ExecutionExpressionId(2),
                                            None,
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        )
                                    ],
                                )
                            ],
                        ),
                        false
                    )),
                    ExecutionNode::Return(ReturnNode::new(
                        ExecutionNodeId(7),
                        vec![ ExecutionNodeId(6) ],
                        92.,
                        16,
                        ExecutionVariable::new(ExecutionVariableId(8), "7"),
                    )),
                ],
                vec![
                    "move-calculations-up",
                    "remove-redundant-calculations",
                    "remove-unnecessary-calculations"
                ],
                vec![
                    ExecutionCollection::new("read", "aql_customers11"),
                ],
                vec![
                    ExecutionVariable::new(ExecutionVariableId(8), "7"),
                    ExecutionVariable::new(ExecutionVariableId(6), "5"),
                    ExecutionVariable::new(ExecutionVariableId(4), "3"),
                    ExecutionVariable::new(ExecutionVariableId(2), "maxAge"),
                    ExecutionVariable::new(ExecutionVariableId(1), "minAge"),
                    ExecutionVariable::new(ExecutionVariableId(0), "c"),
                ],
                92.,
                16,
            ),
            Vec::<String>::new(),
            ExecutionStats::new(29, 0, 1),
            true,
        );

        assert_eq!(explanation, explained_query);

    });
}

#[test]
fn explain_insert_into_collection() {
    arango_test_with_document_collection("aql_customers12", |conn, ref mut core| {
        let database = env::var(ENV_ARANGO_TEST_DATABASE).unwrap();
        let inserts = Query::new(
            "INSERT { \
              name: \"Jane Doe\", \
              city: \"Vienna\",
              age: 42 \
            } IN aql_customers12"
        );

        let method = ExplainQuery::with_defaults(inserts);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_best_plan(
            ExecutionPlan::new(
                vec![
                    ExecutionNode::Singleton(SingletonNode::new(
                        ExecutionNodeId(1),
                        vec![],
                        1.,
                        1,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(2),
                        vec![ ExecutionNodeId(1) ],
                        2.,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(2), "1"),
                        "json",
                        ExecutionExpression::new(
                            "object",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "object element",
                                    "name".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "value",
                                            None,
                                            None,
                                            json!("Jane Doe"),
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        ),
                                    ],
                                ),
                                ExecutionExpression::new(
                                    "object element",
                                    "city".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "value",
                                            None,
                                            None,
                                            json!("Vienna"),
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        ),
                                    ],
                                ),
                                ExecutionExpression::new(
                                    "object element",
                                    "age".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "value",
                                            None,
                                            None,
                                            json!(42),
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        ),
                                    ],
                                ),
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::Insert(InsertNode::new(
                        ExecutionNodeId(3),
                        vec![ ExecutionNodeId(2) ],
                        3.,
                        0,
                        database.clone(),
                        "aql_customers12",
                        ExecutionVariable::new(ExecutionVariableId(2), "1"),
                        None,
                        ModificationOptions::new(
                            false,
                            false,
                            false,
                            true,
                            false,
                            false,
                            false,
                            false,
                        ),
                    )),
                ],
                vec![
                    "remove-data-modification-out-variables",
                ],
                vec![
                    ExecutionCollection::new("write", "aql_customers12"),
                ],
                vec![
                    ExecutionVariable::new(ExecutionVariableId(2), "1"),
                    ExecutionVariable::new(ExecutionVariableId(0), "$NEW"),
                ],
                3.,
                0,
            ),
            Vec::<String>::new(),
            ExecutionStats::new(29, 0, 1),
            true,
        );

        assert_eq!(explanation, explained_query);

    });
}

#[test]
fn explain_remove_document_from_collection() {
    arango_test_with_document_collection("aql_customers13", |conn, ref mut core| {
        let database = env::var(ENV_ARANGO_TEST_DATABASE).unwrap();
        let inserts = Query::new(
            "INSERT { \
              name: \"Jane Doe\", \
              city: \"Vienna\",
              age: 42 \
            } IN aql_customers13"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();

        let remove = Query::new(
            "FOR c IN aql_customers13 \
              REMOVE c IN aql_customers13 \
              RETURN OLD._key"
        );

        let method = ExplainQuery::with_defaults(remove);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_best_plan(
            ExecutionPlan::new(
                vec![
                    ExecutionNode::Singleton(SingletonNode::new(
                        ExecutionNodeId(1),
                        vec![],
                        1.,
                        1,
                    )),
                    ExecutionNode::EnumerateCollection(EnumerateCollectionNode::new(
                        ExecutionNodeId(2),
                        vec![ ExecutionNodeId(1) ],
                        3.,
                        1,
                        database.clone(),
                        "aql_customers13",
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                        false,
                    )),
                    ExecutionNode::Remove(RemoveNode::new(
                        ExecutionNodeId(3),
                        vec![ ExecutionNodeId(2) ],
                        4.,
                        1,
                        database.clone(),
                        "aql_customers13",
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                        ExecutionVariable::new(ExecutionVariableId(1), "$OLD"),
                        ModificationOptions::new(
                            false,
                            false,
                            false,
                            true,
                            false,
                            true,
                            false,
                            false,
                        ),
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(4),
                        vec![ ExecutionNodeId(3) ],
                        5.,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(3), "2"),
                        "attribute",
                        ExecutionExpression::new(
                            "attribute access",
                            "_key".to_owned(),
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "reference",
                                    "$OLD".to_owned(),
                                    ExecutionExpressionId(1),
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![]
                                ),
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::Return(ReturnNode::new(
                        ExecutionNodeId(5),
                        vec![ ExecutionNodeId(4) ],
                        6.,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(3), "2"),
                    )),
                ],
                vec![
                    "remove-data-modification-out-variables",
                ],
                vec![
                    ExecutionCollection::new("write", "aql_customers13"),
                ],
                vec![
                    ExecutionVariable::new(ExecutionVariableId(3), "2"),
                    ExecutionVariable::new(ExecutionVariableId(1), "$OLD"),
                    ExecutionVariable::new(ExecutionVariableId(0), "c"),
                ],
                6.,
                1,
            ),
            Vec::<String>::new(),
            ExecutionStats::new(29, 0, 1),
            true,
        );

        assert_eq!(explanation, explained_query);

    });
}

#[test]
fn explain_update_document() {
    arango_test_with_document_collection("aql_customers14", |conn, ref mut core| {
        let database = env::var(ENV_ARANGO_TEST_DATABASE).unwrap();
        let inserts = Query::new(
            "INSERT { \
              name: \"Jane Doe\", \
              city: \"Vienna\", \
              age: 42 \
            } IN aql_customers14"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();

        let mut update = Query::new(
            "FOR c IN aql_customers14 \
               FILTER c.name == @name \
               UPDATE c WITH { \
                 city: \"Berlin\" \
               } IN aql_customers14"
        );
        update.set_parameter("name", "Jane Doe");
        let update = update;

        let method = ExplainQuery::with_defaults(update);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_best_plan(
            ExecutionPlan::new(
                vec![
                    ExecutionNode::Singleton(SingletonNode::new(
                        ExecutionNodeId(1),
                        vec![],
                        1.,
                        1,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(5),
                        vec![ ExecutionNodeId(1) ],
                        2.,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(6), "5"),
                        "json",
                        ExecutionExpression::new(
                            "object",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "object element",
                                    "city".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "value",
                                            None,
                                            None,
                                            json!("Berlin"),
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        ),
                                    ],
                                ),
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::EnumerateCollection(EnumerateCollectionNode::new(
                        ExecutionNodeId(2),
                        vec![ ExecutionNodeId(5) ],
                        4.,
                        1,
                        database.clone(),
                        "aql_customers14",
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                        false,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(3),
                        vec![ ExecutionNodeId(2) ],
                        5.,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(4), "3"),
                        "simple",
                        ExecutionExpression::new(
                            "compare ==",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "attribute access",
                                    "name".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "reference",
                                            "c".to_owned(),
                                            ExecutionExpressionId(0),
                                            None,
                                            None,
                                            None,
                                            vec![],
                                            vec![]
                                        ),
                                    ]
                                ),
                                ExecutionExpression::new(
                                    "value",
                                    None,
                                    None,
                                    json!("Jane Doe"),
                                    None,
                                    None,
                                    vec![],
                                    vec![]
                                ),
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::Filter(FilterNode::new(
                        ExecutionNodeId(4),
                        vec![ ExecutionNodeId(3) ],
                        6.,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(4), "3"),
                    )),
                    ExecutionNode::Update(UpdateNode::new(
                        ExecutionNodeId(6),
                        vec![ ExecutionNodeId(4) ],
                        7.,
                        0,
                        database.clone(),
                        "aql_customers14",
                        ExecutionVariable::new(ExecutionVariableId(6), "5"),
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                        None,
                        None,
                        ModificationOptions::new(
                            false,
                            false,
                            false,
                            true,
                            false,
                            false,
                            false,
                            false,
                        ),
                    )),
                ],
                vec![
                    "move-calculations-up",
                    "remove-data-modification-out-variables",
                    "patch-update-statements",
                ],
                vec![
                    ExecutionCollection::new("write", "aql_customers14"),
                ],
                vec![
                    ExecutionVariable::new(ExecutionVariableId(6), "5"),
                    ExecutionVariable::new(ExecutionVariableId(4), "3"),
                    ExecutionVariable::new(ExecutionVariableId(2), "$NEW"),
                    ExecutionVariable::new(ExecutionVariableId(1), "$OLD"),
                    ExecutionVariable::new(ExecutionVariableId(0), "c"),
                ],
                7.,
                0,
            ),
            Vec::<String>::new(),
            ExecutionStats::new(29, 0, 1),
            true,
        );

        assert_eq!(explanation, explained_query);

    });
}

#[test]
fn explain_replace_document() {
    arango_test_with_document_collection("aql_customers15", |conn, ref mut core| {
        let database = env::var(ENV_ARANGO_TEST_DATABASE).unwrap();
        let inserts = Query::new(
            "INSERT { \
              name: \"Jane Doe\", \
              city: \"Vienna\", \
              age: 42 \
            } IN aql_customers15"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();

        let mut replace = Query::new(
            "FOR c IN aql_customers15 \
               FILTER c.name == @name \
               REPLACE c WITH { \
                 name: \"John Doe\", \
                 city: \"Berlin\", \
                 age: 43 \
               } IN aql_customers15"
        );
        replace.set_parameter("name", "Jane Doe");
        let replace = replace;

        let method = ExplainQuery::with_defaults(replace);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_best_plan(
            ExecutionPlan::new(
                vec![
                    ExecutionNode::Singleton(SingletonNode::new(
                        ExecutionNodeId(1),
                        vec![],
                        1.,
                        1,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(5),
                        vec![ ExecutionNodeId(1) ],
                        2.,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(6), "5"),
                        "json",
                        ExecutionExpression::new(
                            "object",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "object element",
                                    "name".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "value",
                                            None,
                                            None,
                                            json!("John Doe"),
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        ),
                                    ],
                                ),
                                ExecutionExpression::new(
                                    "object element",
                                    "city".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "value",
                                            None,
                                            None,
                                            json!("Berlin"),
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        ),
                                    ],
                                ),
                                ExecutionExpression::new(
                                    "object element",
                                    "age".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "value",
                                            None,
                                            None,
                                            json!(43),
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        ),
                                    ],
                                ),
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::EnumerateCollection(EnumerateCollectionNode::new(
                        ExecutionNodeId(2),
                        vec![ ExecutionNodeId(5) ],
                        4.,
                        1,
                        database.clone(),
                        "aql_customers15",
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                        false,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(3),
                        vec![ ExecutionNodeId(2) ],
                        5.,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(4), "3"),
                        "simple",
                        ExecutionExpression::new(
                            "compare ==",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "attribute access",
                                    "name".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "reference",
                                            "c".to_owned(),
                                            ExecutionExpressionId(0),
                                            None,
                                            None,
                                            None,
                                            vec![],
                                            vec![]
                                        ),
                                    ]
                                ),
                                ExecutionExpression::new(
                                    "value",
                                    None,
                                    None,
                                    json!("Jane Doe"),
                                    None,
                                    None,
                                    vec![],
                                    vec![]
                                ),
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::Filter(FilterNode::new(
                        ExecutionNodeId(4),
                        vec![ ExecutionNodeId(3) ],
                        6.,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(4), "3"),
                    )),
                    ExecutionNode::Replace(ReplaceNode::new(
                        ExecutionNodeId(6),
                        vec![ ExecutionNodeId(4) ],
                        7.,
                        0,
                        database.clone(),
                        "aql_customers15",
                        ExecutionVariable::new(ExecutionVariableId(6), "5"),
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                        None,
                        None,
                        ModificationOptions::new(
                            false,
                            false,
                            false,
                            true,
                            false,
                            true,
                            false,
                            false,
                        ),
                    )),
                ],
                vec![
                    "move-calculations-up",
                    "remove-data-modification-out-variables",
                ],
                vec![
                    ExecutionCollection::new("write", "aql_customers15"),
                ],
                vec![
                    ExecutionVariable::new(ExecutionVariableId(6), "5"),
                    ExecutionVariable::new(ExecutionVariableId(4), "3"),
                    ExecutionVariable::new(ExecutionVariableId(2), "$NEW"),
                    ExecutionVariable::new(ExecutionVariableId(1), "$OLD"),
                    ExecutionVariable::new(ExecutionVariableId(0), "c"),
                ],
                7.,
                0,
            ),
            Vec::<String>::new(),
            ExecutionStats::new(29, 0, 1),
            true,
        );

        assert_eq!(explanation, explained_query);

    });
}

#[test]
fn explain_upsert_when_document_not_existing() {
    arango_test_with_document_collection("aql_customers16", |conn, ref mut core| {
        let database = env::var(ENV_ARANGO_TEST_DATABASE).unwrap();
        let upsert = Query::new(
            "UPSERT { name: 'Jane Doe' } \
               INSERT { \
                 name: \"Jane Doe\", \
                 city: \"Vienna\",
                 age: 42 \
               } \
               UPDATE { \
                 age: 41 \
               } \
               IN aql_customers16"
        );

        let method = ExplainQuery::with_defaults(upsert);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_best_plan(
            ExecutionPlan::new(
                vec![
                    ExecutionNode::Singleton(SingletonNode::new(
                        ExecutionNodeId(1),
                        vec![],
                        1.,
                        1,
                    )),
                    ExecutionNode::SubQuery(SubQueryNode::new(
                        ExecutionNodeId(8),
                        vec![ ExecutionNodeId(1) ],
                        3.,
                        1,
                        ExplainedSubQuery::new(
                            vec![
                                ExecutionNode::Singleton(SingletonNode::new(
                                    ExecutionNodeId(2),
                                    vec![],
                                    1.,
                                    1,
                                )),
                                ExecutionNode::EnumerateCollection(EnumerateCollectionNode::new(
                                    ExecutionNodeId(3),
                                    vec![ ExecutionNodeId(2) ],
                                    2.,
                                    0,
                                    database.clone(),
                                    "aql_customers16",
                                    ExecutionVariable::new(ExecutionVariableId(2), "1"),
                                    false,
                                )),
                                ExecutionNode::Calculation(CalculationNode::new(
                                    ExecutionNodeId(4),
                                    vec![ ExecutionNodeId(3) ],
                                    2.,
                                    0,
                                    ExecutionVariable::new(ExecutionVariableId(7), "6"),
                                    "simple",
                                    ExecutionExpression::new(
                                        "compare ==",
                                        None,
                                        None,
                                        None,
                                        None,
                                        None,
                                        vec![],
                                        vec![
                                            ExecutionExpression::new(
                                                "attribute access",
                                                "name".to_owned(),
                                                None,
                                                None,
                                                None,
                                                None,
                                                vec![],
                                                vec![
                                                    ExecutionExpression::new(
                                                        "reference",
                                                        "1".to_owned(),
                                                        ExecutionExpressionId(2),
                                                        None,
                                                        None,
                                                        None,
                                                        vec![],
                                                        vec![]
                                                    ),
                                                ]
                                            ),
                                            ExecutionExpression::new(
                                                "value",
                                                None,
                                                None,
                                                json!("Jane Doe"),
                                                None,
                                                None,
                                                vec![],
                                                vec![]
                                            ),
                                        ]
                                    ),
                                    false,
                                )),
                                ExecutionNode::Filter(FilterNode::new(
                                    ExecutionNodeId(5),
                                    vec![ ExecutionNodeId(4) ],
                                    2.,
                                    0,
                                    ExecutionVariable::new(ExecutionVariableId(7), "6"),
                                )),
                                ExecutionNode::Limit(LimitNode::new(
                                    ExecutionNodeId(6),
                                    vec![ ExecutionNodeId(5) ],
                                    2.,
                                    0,
                                    0,
                                    1,
                                    false,
                                )),
                                ExecutionNode::Return(ReturnNode::new(
                                    ExecutionNodeId(7),
                                    vec![ ExecutionNodeId(6) ],
                                    2.,
                                    0,
                                    ExecutionVariable::new(ExecutionVariableId(2), "1"),
                                ))
                            ]
                        ),
                        ExecutionVariable::new(ExecutionVariableId(4), "3"),
                        true,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(9),
                        vec![ ExecutionNodeId(8) ],
                        4.,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(0), "$OLD"),
                        "simple",
                        ExecutionExpression::new(
                            "indexed access",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "reference",
                                    "3".to_owned(),
                                    ExecutionExpressionId(4),
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![]
                                ),
                                ExecutionExpression::new(
                                    "value",
                                    None,
                                    None,
                                    json!(0),
                                    None,
                                    None,
                                    vec![],
                                    vec![]
                                ),
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(11),
                        vec![ ExecutionNodeId(9) ],
                        5.,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(11), "10"),
                        "json",
                        ExecutionExpression::new(
                            "object",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "object element",
                                    "age".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "value",
                                            None,
                                            None,
                                            json!(41),
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        ),
                                    ],
                                ),
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(10),
                        vec![ ExecutionNodeId(11) ],
                        6.,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(9), "8"),
                        "json",
                        ExecutionExpression::new(
                            "object",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "object element",
                                    "name".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "value",
                                            None,
                                            None,
                                            json!("Jane Doe"),
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        ),
                                    ],
                                ),
                                ExecutionExpression::new(
                                    "object element",
                                    "city".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "value",
                                            None,
                                            None,
                                            json!("Vienna"),
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        ),
                                    ],
                                ),
                                ExecutionExpression::new(
                                    "object element",
                                    "age".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "value",
                                            None,
                                            None,
                                            json!(42),
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        ),
                                    ],
                                ),
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::Upsert(UpsertNode::new(
                        ExecutionNodeId(12),
                        vec![ ExecutionNodeId(10) ],
                        7.,
                        0,
                        database.clone(),
                        "aql_customers16",
                        ExecutionVariable::new(ExecutionVariableId(0), "$OLD"),
                        None,
                        ExecutionVariable::new(ExecutionVariableId(9), "8"),
                        ExecutionVariable::new(ExecutionVariableId(11), "10"),
                        false,
                        ModificationOptions::new(
                            false,
                            false,
                            false,
                            true,
                            false,
                            false,
                            false,
                            false,
                        ),
                    )),
                ],
                vec![
                    "move-calculations-up",
                    "move-calculations-up-2",
                    "remove-data-modification-out-variables",
                    "move-calculations-down",
                ],
                vec![
                    ExecutionCollection::new("write", "aql_customers16"),
                ],
                vec![
                    ExecutionVariable::new(ExecutionVariableId(9), "8"),
                    ExecutionVariable::new(ExecutionVariableId(7), "6"),
                    ExecutionVariable::new(ExecutionVariableId(5), "$NEW"),
                    ExecutionVariable::new(ExecutionVariableId(4), "3"),
                    ExecutionVariable::new(ExecutionVariableId(2), "1"),
                    ExecutionVariable::new(ExecutionVariableId(11), "10"),
                    ExecutionVariable::new(ExecutionVariableId(0), "$OLD"),
                ],
                7.,
                0,
            ),
            Vec::<String>::new(),
            ExecutionStats::new(29, 0, 1),
            true,
        );

        assert_eq!(explanation, explained_query);

    });
}

#[test]
fn explain_upsert_when_document_is_existing() {
    arango_test_with_document_collection("aql_customers17", |conn, ref mut core| {
        let database = env::var(ENV_ARANGO_TEST_DATABASE).unwrap();
        let inserts = Query::new(
            "INSERT { \
              name: \"Jane Doe\", \
              city: \"Vienna\",
              age: 42 \
            } IN aql_customers17"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();

        let upsert = Query::new(
            "UPSERT { name: 'Jane Doe' } \
               INSERT { \
                 name: \"Jane Doe\", \
                 city: \"Vienna\",
                 age: 42 \
               } \
               UPDATE { \
                 age: 41 \
               } \
               IN aql_customers17"
        );

        let method = ExplainQuery::with_defaults(upsert);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_best_plan(
            ExecutionPlan::new(
                vec![
                    ExecutionNode::Singleton(SingletonNode::new(
                        ExecutionNodeId(1),
                        vec![],
                        1.,
                        1,
                    )),
                    ExecutionNode::SubQuery(SubQueryNode::new(
                        ExecutionNodeId(8),
                        vec![ ExecutionNodeId(1) ],
                        8.,
                        1,
                        ExplainedSubQuery::new(
                            vec![
                                ExecutionNode::Singleton(SingletonNode::new(
                                    ExecutionNodeId(2),
                                    vec![],
                                    1.,
                                    1,
                                )),
                                ExecutionNode::EnumerateCollection(EnumerateCollectionNode::new(
                                    ExecutionNodeId(3),
                                    vec![ ExecutionNodeId(2) ],
                                    3.,
                                    1,
                                    database.clone(),
                                    "aql_customers17",
                                    ExecutionVariable::new(ExecutionVariableId(2), "1"),
                                    false,
                                )),
                                ExecutionNode::Calculation(CalculationNode::new(
                                    ExecutionNodeId(4),
                                    vec![ ExecutionNodeId(3) ],
                                    4.,
                                    1,
                                    ExecutionVariable::new(ExecutionVariableId(7), "6"),
                                    "simple",
                                    ExecutionExpression::new(
                                        "compare ==",
                                        None,
                                        None,
                                        None,
                                        None,
                                        None,
                                        vec![],
                                        vec![
                                            ExecutionExpression::new(
                                                "attribute access",
                                                "name".to_owned(),
                                                None,
                                                None,
                                                None,
                                                None,
                                                vec![],
                                                vec![
                                                    ExecutionExpression::new(
                                                        "reference",
                                                        "1".to_owned(),
                                                        ExecutionExpressionId(2),
                                                        None,
                                                        None,
                                                        None,
                                                        vec![],
                                                        vec![]
                                                    ),
                                                ]
                                            ),
                                            ExecutionExpression::new(
                                                "value",
                                                None,
                                                None,
                                                json!("Jane Doe"),
                                                None,
                                                None,
                                                vec![],
                                                vec![]
                                            ),
                                        ]
                                    ),
                                    false,
                                )),
                                ExecutionNode::Filter(FilterNode::new(
                                    ExecutionNodeId(5),
                                    vec![ ExecutionNodeId(4) ],
                                    5.,
                                    1,
                                    ExecutionVariable::new(ExecutionVariableId(7), "6"),
                                )),
                                ExecutionNode::Limit(LimitNode::new(
                                    ExecutionNodeId(6),
                                    vec![ ExecutionNodeId(5) ],
                                    6.,
                                    1,
                                    0,
                                    1,
                                    false,
                                )),
                                ExecutionNode::Return(ReturnNode::new(
                                    ExecutionNodeId(7),
                                    vec![ ExecutionNodeId(6) ],
                                    7.,
                                    1,
                                    ExecutionVariable::new(ExecutionVariableId(2), "1"),
                                ))
                            ]
                        ),
                        ExecutionVariable::new(ExecutionVariableId(4), "3"),
                        true,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(9),
                        vec![ ExecutionNodeId(8) ],
                        9.,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(0), "$OLD"),
                        "simple",
                        ExecutionExpression::new(
                            "indexed access",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "reference",
                                    "3".to_owned(),
                                    ExecutionExpressionId(4),
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![]
                                ),
                                ExecutionExpression::new(
                                    "value",
                                    None,
                                    None,
                                    json!(0),
                                    None,
                                    None,
                                    vec![],
                                    vec![]
                                ),
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(11),
                        vec![ ExecutionNodeId(9) ],
                        10.,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(11), "10"),
                        "json",
                        ExecutionExpression::new(
                            "object",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "object element",
                                    "age".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "value",
                                            None,
                                            None,
                                            json!(41),
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        ),
                                    ],
                                ),
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(10),
                        vec![ ExecutionNodeId(11) ],
                        11.,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(9), "8"),
                        "json",
                        ExecutionExpression::new(
                            "object",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "object element",
                                    "name".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "value",
                                            None,
                                            None,
                                            json!("Jane Doe"),
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        ),
                                    ],
                                ),
                                ExecutionExpression::new(
                                    "object element",
                                    "city".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "value",
                                            None,
                                            None,
                                            json!("Vienna"),
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        ),
                                    ],
                                ),
                                ExecutionExpression::new(
                                    "object element",
                                    "age".to_owned(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    vec![
                                        ExecutionExpression::new(
                                            "value",
                                            None,
                                            None,
                                            json!(42),
                                            None,
                                            None,
                                            vec![],
                                            vec![],
                                        ),
                                    ],
                                ),
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::Upsert(UpsertNode::new(
                        ExecutionNodeId(12),
                        vec![ ExecutionNodeId(10) ],
                        12.,
                        0,
                        database.clone(),
                        "aql_customers17",
                        ExecutionVariable::new(ExecutionVariableId(0), "$OLD"),
                        None,
                        ExecutionVariable::new(ExecutionVariableId(9), "8"),
                        ExecutionVariable::new(ExecutionVariableId(11), "10"),
                        false,
                        ModificationOptions::new(
                            false,
                            false,
                            false,
                            true,
                            false,
                            false,
                            false,
                            false,
                        ),
                    )),
                ],
                vec![
                    "move-calculations-up",
                    "move-calculations-up-2",
                    "remove-data-modification-out-variables",
                    "move-calculations-down",
                ],
                vec![
                    ExecutionCollection::new("write", "aql_customers17"),
                ],
                vec![
                    ExecutionVariable::new(ExecutionVariableId(9), "8"),
                    ExecutionVariable::new(ExecutionVariableId(7), "6"),
                    ExecutionVariable::new(ExecutionVariableId(5), "$NEW"),
                    ExecutionVariable::new(ExecutionVariableId(4), "3"),
                    ExecutionVariable::new(ExecutionVariableId(2), "1"),
                    ExecutionVariable::new(ExecutionVariableId(11), "10"),
                    ExecutionVariable::new(ExecutionVariableId(0), "$OLD"),
                ],
                12.,
                0,
            ),
            Vec::<String>::new(),
            ExecutionStats::new(29, 0, 1),
            true,
        );

        assert_eq!(explanation, explained_query);

    });
}

#[test]
fn explain_query_with_no_result() {
    arango_test_with_document_collection("aql_customers18", |conn, ref mut core| {
        let database = env::var(ENV_ARANGO_TEST_DATABASE).unwrap();
        let inserts = Query::new(
            "FOR i IN 1..21 \
              INSERT { \
                name: CONCAT('No.', i), \
                age: i + 21 \
              } IN aql_customers18"
        );
        core.run(conn.execute(CreateCursor::<Empty>::from_query(inserts))).unwrap();

        let query = Query::new(
            "FOR c IN aql_customers18 \
              FILTER true != true \
              RETURN c"
        );

        let method = ExplainQuery::with_defaults(query);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_best_plan(
            ExecutionPlan::new(
                vec![
                    ExecutionNode::Singleton(SingletonNode::new(
                        ExecutionNodeId(1),
                        vec![],
                        1.,
                        1,
                    )),
                    ExecutionNode::NoResults(NoResultsNode::new(
                        ExecutionNodeId(6),
                        vec![ ExecutionNodeId(1) ],
                        0.5,
                        0,
                    )),
                    ExecutionNode::EnumerateCollection(EnumerateCollectionNode::new(
                        ExecutionNodeId(2),
                        vec![ ExecutionNodeId(6) ],
                        1.5,
                        0,
                        database.clone(),
                        "aql_customers18",
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                        false,
                    )),
                    ExecutionNode::Return(ReturnNode::new(
                        ExecutionNodeId(5),
                        vec![ ExecutionNodeId(2) ],
                        1.5,
                        0,
                        ExecutionVariable::new(ExecutionVariableId(0), "c"),
                    )),
                ],
                vec![
                    "move-calculations-up",
                    "move-filters-up",
                    "remove-unnecessary-filters",
                    "remove-unnecessary-calculations",
                ],
                vec![
                    ExecutionCollection::new("read", "aql_customers18"),
                ],
                vec![
                    ExecutionVariable::new(ExecutionVariableId(2), "1"),
                    ExecutionVariable::new(ExecutionVariableId(0), "c"),
                ],
                1.5,
                0,
            ),
            Vec::<String>::new(),
            ExecutionStats::new(29, 0, 1),
            true,
        );

        assert_eq!(explanation, explained_query);

    });
}

#[test]
fn explain_query_with_simple_enumeration() {
    arango_test_with_document_collection("aql_customers19", |conn, ref mut core| {

        let query = Query::new(
            "FOR n IN 1..42 \
              RETURN n"
        );

        let method = ExplainQuery::with_defaults(query);
        let explained_query = core.run(conn.execute(method)).unwrap();

        let explanation = ExplainedQuery::from_best_plan(
            ExecutionPlan::new(
                vec![
                    ExecutionNode::Singleton(SingletonNode::new(
                        ExecutionNodeId(1),
                        vec![],
                        1.,
                        1,
                    )),
                    ExecutionNode::Calculation(CalculationNode::new(
                        ExecutionNodeId(2),
                        vec![ ExecutionNodeId(1) ],
                        2.,
                        1,
                        ExecutionVariable::new(ExecutionVariableId(2), "1"),
                        "simple",
                        ExecutionExpression::new(
                            "range",
                            None,
                            None,
                            None,
                            None,
                            None,
                            vec![],
                            vec![
                                ExecutionExpression::new(
                                    "value",
                                    None,
                                    None,
                                    json!(1),
                                    None,
                                    None,
                                    vec![],
                                    vec![]
                                ),
                                ExecutionExpression::new(
                                    "value",
                                    None,
                                    None,
                                    json!(42),
                                    None,
                                    None,
                                    vec![],
                                    vec![]
                                ),
                            ]
                        ),
                        false,
                    )),
                    ExecutionNode::EnumerateList(EnumerateListNode::new(
                        ExecutionNodeId(3),
                        vec![ ExecutionNodeId(2) ],
                        44.,
                        42,
                        ExecutionVariable::new(ExecutionVariableId(2), "1"),
                        ExecutionVariable::new(ExecutionVariableId(0), "n"),
                    )),
                    ExecutionNode::Return(ReturnNode::new(
                        ExecutionNodeId(4),
                        vec![ ExecutionNodeId(3) ],
                        86.,
                        42,
                        ExecutionVariable::new(ExecutionVariableId(0), "n"),
                    )),
                ],
                Vec::<String>::new(),
                vec![],
                vec![
                    ExecutionVariable::new(ExecutionVariableId(2), "1"),
                    ExecutionVariable::new(ExecutionVariableId(0), "n"),
                ],
                86.,
                42,
            ),
            Vec::<String>::new(),
            ExecutionStats::new(29, 0, 1),
            true,
        );

        assert_eq!(explanation, explained_query);

    });
}
