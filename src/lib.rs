use pgx::prelude::*;
use tokio::runtime::Runtime;
use pgx::{spi};
use std::collections::HashMap;

use openai::{
  chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
  set_key,
};

use std::{
  env,
};

pgx::pg_module_magic!();

#[pg_extern]
fn gpt(input: &str) -> String {
  let rt = Runtime::new().unwrap();
  set_key(env::var("OPENAI_KEY").unwrap());
  let query = "SELECT table_name, column_name FROM information_schema.columns WHERE table_schema = 'public' ORDER BY table_name, ordinal_position;";

  let schema: Result<
  TableIterator<
      'static,
      (
          name!(oid, Result<Option<String>, pgx::spi::Error>),
          name!(name, Result<Option<String>, pgx::spi::Error>),
      ),
  >,
  spi::Error,
> = Spi::connect(|client| {
    Ok(client.select(query, None, None)?.map(|row| (row["table_name"].value(), row["column_name"].value())))
  })
  .map(|results| TableIterator::new(results));

  let mut schema_str = String::new();
  let tables: HashMap<String, Vec<String>> = HashMap::new();
  for value in schema.unwrap() {
    let table_name = value.0.unwrap().unwrap_or_else(|| "".to_owned());
    let column_name = value.1.unwrap().unwrap_or_else(|| "".to_owned());
    if !tables.contains_key(table_name.as_str()) {
      schema_str.push_str(&format!("){}(", table_name));
    } else {
      schema_str.push_str(&format!("{},", column_name));
    }
  }

  let mut messages = vec![ChatCompletionMessage {
    role: ChatCompletionMessageRole::System,
    content: "You are an SQL assistent and you will return raw PostgreSQL queries without any additional words ready to execute".to_string(),
    name: None,
  }];
  messages.push(ChatCompletionMessage {
    role: ChatCompletionMessageRole::User,
    content: format!("Here is the schema: {}", schema_str),
    name: None,
  });
  messages.push(ChatCompletionMessage {
    role: ChatCompletionMessageRole::User,
    content: format!("Here is the question: {}", input),
    name: None,
  });

  let chat_completion = rt.block_on(async move {
    ChatCompletion::builder("gpt-3.5-turbo", messages.clone())
    .create()
    .await
    .unwrap()
    .unwrap()
  });
  let returned_message = chat_completion.choices.first().unwrap().message.clone();

  returned_message.content.to_string()

}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgx::prelude::*;

    #[pg_test]
    fn test_hello_my_extension() {
        assert_eq!("Hello, my_extension", crate::hello_my_extension("show me all aws s3 buckets"));
    }

}

/// This module is required by `cargo pgx test` invocations. 
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
