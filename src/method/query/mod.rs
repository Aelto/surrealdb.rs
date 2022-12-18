/// A module for the various wrapping types for the responses and results
/// returned by the database.
pub mod response;

use crate::method::Method;
use crate::param;
use crate::param::from_json;
use crate::param::Param;
use crate::Connection;
use crate::Result;
use crate::Router;
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::future::Future;
use std::future::IntoFuture;
use std::pin::Pin;
use surrealdb::sql;
use surrealdb::sql::Statement;
use surrealdb::sql::Statements;
use surrealdb::sql::Value;

use response::QueryResponse;

/// A query future
#[derive(Debug)]
pub struct Query<'r, C: Connection> {
	pub(super) router: Result<&'r Router<C>>,
	pub(super) query: Vec<Result<Vec<Statement>>>,
	pub(super) bindings: BTreeMap<String, Value>,
}

impl<'r, Client> IntoFuture for Query<'r, Client>
where
	Client: Connection,
{
	type Output = Result<QueryResponse>;
	type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + Sync + 'r>>;

	fn into_future(self) -> Self::IntoFuture {
		Box::pin(async move {
			let mut statements = Vec::with_capacity(self.query.len());
			for query in self.query {
				statements.extend(query?);
			}
			let mut param = vec![sql::Query(Statements(statements)).to_string().into()];
			if !self.bindings.is_empty() {
				param.push(self.bindings.into());
			}
			let mut conn = Client::new(Method::Query);
			conn.execute_query(self.router?, Param::new(param)).await
		})
	}
}

impl<'r, C> Query<'r, C>
where
	C: Connection,
{
	/// Chains a query onto an existing query
	pub fn query(mut self, query: impl param::Query) -> Self {
		self.query.push(query.try_into_query());
		self
	}

	/// Binds a parameter to a query
	pub fn bind(mut self, v: impl crate::param::binding::AppendBinding) -> Self {
		v.append_binding(&mut self.bindings);

		self
	}

	///
	pub fn bindtwo(mut self, o: impl Into<QueryBindings>) -> Self {
		let mut bindings: QueryBindings = o.into();
		self.bindings.append(&mut bindings.bindings);
		self
	}

	///
	pub fn bind_object(self, o: impl Serialize) -> Self {
		self.bindtwo(json!(o))
	}
}

///
pub mod extensions {
	use super::*;

	///
	pub trait QueryBindingExt {
		///
		fn into_binding_value(self) -> sql::Value;
	}

	impl<T> QueryBindingExt for T
	where
		T: Into<sql::Value>,
	{
		fn into_binding_value(self) -> sql::Value {
			self.into()
		}
	}

	///
	pub trait SqlValueExt {
		///
		fn into_value(self) -> sql::Value;
		///
		fn to_value(&self) -> sql::Value;
	}

	impl<T> SqlValueExt for T
	where
		T: Into<sql::Thing>,
		T: Clone,
	{
		fn into_value(self) -> sql::Value {
			self.into().into()
		}

		fn to_value(&self) -> sql::Value {
			self.clone().into().into()
		}
	}

	///
	pub trait SqlValueStringExt {
		///
		fn into_thing(self) -> std::result::Result<sql::Thing, surrealdb::Error>;
	}
	impl<T> SqlValueStringExt for T
	where
		T: AsRef<str>,
	{
		fn into_thing(self) -> std::result::Result<sql::Thing, surrealdb::Error> {
			sql::thing(self.as_ref())
		}
	}
}

///
#[derive(Debug)]
pub struct QueryBindings {
	bindings: BTreeMap<String, Value>,
}
impl QueryBindings {
	fn new() -> Self {
		Self {
			bindings: BTreeMap::new(),
		}
	}
}
impl<K: Into<String>, V: Serialize> From<(K, V)> for QueryBindings {
	fn from(value: (K, V)) -> Self {
		let mut output = Self::new();
		output.bindings.insert(value.0.into(), from_json(json!(value.1)));
		output
	}
}
impl From<serde_json::Value> for QueryBindings {
	fn from(value: serde_json::Value) -> Self {
		let mut output = Self::new();
		match value {
			serde_json::Value::Null => todo!(),
			serde_json::Value::Bool(_) => todo!(),
			serde_json::Value::Number(_) => todo!(),
			serde_json::Value::String(_) => todo!(),
			serde_json::Value::Array(_) => todo!(),
			serde_json::Value::Object(object) => {
				for (key, value) in object {
					output.bindings.insert(key, from_json(value));
				}
			}
		};
		output
	}
}
