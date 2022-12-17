use serde::Deserialize;
use surrealdb_rs::param::binding::AppendBinding;
use surrealdb_rs::param::Root;
use surrealdb_rs::protocol::Ws;
use surrealdb_rs::Surreal;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct User {
	id: String,
	name: String,
	company: String,
}

impl AppendBinding for User {
	fn append_binding(self, bindings: &mut surrealdb_rs::param::binding::BindingMap) {
		bindings.insert("id".to_owned(), self.id.into());
		bindings.insert("name".to_owned(), self.name.into());
		bindings.insert("company".to_owned(), self.company.into());
	}
}

#[tokio::main]
async fn main() -> surrealdb_rs::Result<()> {
	tracing_subscriber::fmt::init();

	let client = Surreal::connect::<Ws>("localhost:8000").await?;

	client
		.signin(Root {
			username: "root",
			password: "root",
		})
		.await?;

	client.use_ns("namespace").use_db("database").await?;

	let results = client
		.query("CREATE user SET name = $name, company = $company")
		.bind(User {
			company: "ACME".to_owned(),
			name: "John".to_owned(),
			id: String::new(),
		})
		.await?;

	// print the created user:
	let user: Option<User> = results.get(0, 0)?;
	tracing::info!("{user:?}");

	let response = client.query("SELECT * FROM user").await?;

	// print all users:
	let users: Vec<User> = response.get(0, ..)?;
	tracing::info!("{users:?}");

	Ok(())
}
