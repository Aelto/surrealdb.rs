use std::collections::BTreeMap;

use surrealdb::sql;

///
pub type BindingMap = BTreeMap<String, sql::Value>;

///
pub trait AppendBinding {
	///
	fn append_binding(self, bindings: &mut BindingMap);
}

/// Implementation for tuples ("key", IntoValue)
impl<IntoValue> AppendBinding for (&str, IntoValue)
where
	IntoValue: Into<sql::Value>,
{
	fn append_binding(self, bindings: &mut BindingMap) {
		bindings.insert(self.0.to_owned(), self.1.into());
	}
}
