
use std::cell::RefCell;
use std::rc::Rc;

use futures::Future;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use tokio_core::reactor::Core;

pub use rincon_core::api::connector::{Connector, Error};
pub use rincon_core::api::query::Query;
pub use rincon_core::api::types::Empty;
pub use rincon_client::cursor::types::{Cursor, NewCursor};
pub use rincon_client::database::types::{Database, NewDatabase};
pub use rincon_client::graph::types::{EdgeDefinition, Graph, NewGraph};
pub use rincon_client::user::types::{NewUser, UserExtra};

use rincon_core::api::connector::Execute;
use rincon_core::arango::protocol::SYSTEM_DATABASE;
use rincon_client::cursor::methods::CreateCursor;
use rincon_client::database::methods::{CreateDatabase, DropDatabase};
use rincon_client::graph::methods::CreateGraph;

/// An `ArangoSession` defines the entry point to the session api. It basically
/// determines which `Connector` shall be used in an application.
#[derive(Debug)]
pub struct ArangoSession<C> {
    connector: Rc<C>,
    core: Rc<RefCell<Core>>,
}

impl<C> ArangoSession<C>
    where C: 'static + Connector
{
    /// Instantiate a new `ArangoSession` using the given `Connector`.
    pub fn new(connector: C, core: Core) -> Self {
        ArangoSession {
            connector: Rc::new(connector),
            core: Rc::new(RefCell::new(core)),
        }
    }

    pub fn close(self) {
        //TODO see if a close() method has any purpose
    }

    /// Returns a new `DatabaseSession` for the system database.
    ///
    /// In *ArangoDB* the system database usually has the name `_system`.
    pub fn use_system_database(&self) -> DatabaseSession<C> {
        DatabaseSession::new(SYSTEM_DATABASE.to_owned(), self.connector.clone(), self.core.clone())
    }

    /// Returns a new `DatabaseSession` for the given database name.
    pub fn use_database<DbName>(&self, database_name: DbName) -> DatabaseSession<C>
        where DbName: Into<String>
    {
        DatabaseSession::new(database_name.into(), self.connector.clone(), self.core.clone())
    }

    /// Creates a new database with the given attributes.
    ///
    /// If the database could be created successfully a `DatabaseSession` using
    /// the just created database is returned.
    pub fn create_database<UserInfo>(&self, new_database: NewDatabase<UserInfo>) -> Result<DatabaseSession<C>, Error>
        where UserInfo: UserExtra + Serialize + 'static
    {
        let core = self.core.clone();
        let connector = self.connector.clone();
        let database_name = new_database.name().to_owned();
        self.core.borrow_mut().run(self.connector.system_connection()
            .execute(CreateDatabase::new(new_database))
                .map(move |_| DatabaseSession::new(database_name, connector, core))
        )
    }
}

/// A session for operating with a specific database.
#[derive(Debug)]
pub struct DatabaseSession<C> {
    database_name: String,
    connector: Rc<C>,
    core: Rc<RefCell<Core>>,
}

impl<C> DatabaseSession<C>
    where C: 'static + Connector
{
    /// Instantiate a new `DatabaseSession` for the database with the given
    /// name.
    fn new(database_name: String, connector: Rc<C>, core: Rc<RefCell<Core>>) -> Self {
        DatabaseSession {
            database_name,
            connector,
            core,
        }
    }

    /// Returns the name of the database this `DatabaseSession` operates with.
    pub fn name(&self) -> &str {
        &self.database_name
    }

    /// Drops the database that is used in this session.
    ///
    /// After calling this function the associated `DatabaseSession` is no
    /// longer valid.
    pub fn drop(self) -> Result<bool, Error> {
        let database_name = self.database_name.to_owned();
        self.core.borrow_mut().run(self.connector.system_connection()
            .execute(DropDatabase::new(database_name))
        )
    }

    /// Executes a query and returns a cursor with the first result set.
    ///
    /// All cursor options and query execution options are left to their default
    /// settings.
    ///
    /// To specify cursor options and/or query execution options use the
    /// `query_opt(&self, NewCursor)` function.
    pub fn query<T>(&self, query: Query) -> Result<Cursor<T>, Error>
        where T: 'static + DeserializeOwned
    {
        self.core.borrow_mut().run(self.connector.connection(&self.database_name)
            .execute(CreateCursor::from_query(query))
        )
    }

    /// Executes a query and returns a cursor with the first result set.
    ///
    /// It requires a `NewCursor` struct as a parameter which allows full
    /// control over all supported cursor options and query execution options.
    ///
    /// To execute a query with all options left at their defaults the
    /// `query(&self, Query)` function might be more suitable.
    pub fn query_opt<T>(&self, new_cursor: NewCursor) -> Result<Cursor<T>, Error>
        where T: 'static + DeserializeOwned
    {
        self.core.borrow_mut().run(self.connector.connection(&self.database_name)
            .execute(CreateCursor::new(new_cursor))
        )
    }

    /// Creates a new graph in the database represented by this
    /// `DatabaseSession`.
    pub fn create_graph(&self, new_graph: NewGraph) -> Result<GraphSession<C>, Error> {
        let core = self.core.clone();
        let connector = self.connector.clone();
        let database_name = self.database_name.clone();
        self.core.borrow_mut().run(self.connector.connection(&self.database_name)
            .execute(CreateGraph::new(new_graph))
                .map(|graph| GraphSession::new(graph, database_name, connector, core))
        )
    }

    /// Returns a new `GraphSession` using the given graph.
    pub fn use_graph(&self, graph: Graph) -> GraphSession<C> {
        GraphSession::new(
            graph,
            self.database_name.clone(),
            self.connector.clone(),
            self.core.clone()
        )
    }
}

#[derive(Debug)]
pub struct GraphSession<C> {
    graph: Graph,
    database_name: String,
    connector: Rc<C>,
    core: Rc<RefCell<Core>>,
}

impl<C> GraphSession<C>
    where C: 'static + Connector
{
    fn new(graph: Graph, database_name: String, connector: Rc<C>, core: Rc<RefCell<Core>>) -> Self {
        GraphSession {
            graph,
            database_name,
            connector,
            core,
        }
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }
}
