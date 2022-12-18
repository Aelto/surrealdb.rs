use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::json;
use surrealdb::sql::{self, thing};

use super::from_json;

///
pub type BindingMap = BTreeMap<String, sql::Value>;

///
pub trait AppendBinding {
	///
	fn append_binding(self, bindings: &mut BindingMap);
}

/// Implementation for tuples ("key", IntoValue)
// impl<IntoValue> AppendBinding for (&str, IntoValue)
// where
// 	IntoValue: Into<sql::Value>,
// {
// 	default fn append_binding(self, bindings: &mut BindingMap) {
// 		bindings.insert(self.0.to_owned(), self.1.into());
// 	}
// }

impl<T> AppendBinding for T
where
	T: Serialize,
{
	default fn append_binding(self, bindings: &mut BindingMap) {
		let json_value = json!(self);

		if let serde_json::Value::Object(map) = json_value {
			for (key, value) in map {
				if let serde_json::Value::String(s) = value {
					(key, s.as_str()).append_binding(bindings);
				} else {
					(key, from_json(value)).append_binding(bindings);
				}
			}
		}
	}
}

impl AppendBinding for (String, sql::Value) {
	fn append_binding(self, bindings: &mut BindingMap) {
		bindings.insert(self.0, self.1);
	}
}

/// Implementation for tuples ("key", "thing_or_string")
impl AppendBinding for (&str, &str) {
	fn append_binding(self, bindings: &mut BindingMap) {
		(self.0.to_owned(), self.1).append_binding(bindings);
	}
}

impl AppendBinding for (&str, String) {
	fn append_binding(self, bindings: &mut BindingMap) {
		(self.0, self.1.as_ref()).append_binding(bindings);
	}
}

impl AppendBinding for (&str, Option<&str>) {
	fn append_binding(self, bindings: &mut BindingMap) {
		if let Some(s) = self.1 {
			(self.0, s).append_binding(bindings);
		}
	}
}

impl AppendBinding for (String, String) {
	fn append_binding(self, bindings: &mut BindingMap) {
		(self.0, self.1.as_str()).append_binding(bindings);
	}
}

impl AppendBinding for (String, &str) {
	fn append_binding(self, bindings: &mut BindingMap) {
		let value = match thing(&self.1) {
			Ok(thing) => thing.into(),
			Err(_) => self.1.into(),
		};

		bindings.insert(self.0, value);
	}
}
