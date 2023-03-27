use pgx::{prelude::*, JsonString};
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
  match env::var("OPENAI_KEY") {
    Ok(val) => set_key(val),
    Err(e) => println!("Couldn't interpret {}: {}", "OPENAI_KEY", e),
  }

  let schema_sql = "SELECT json_object_agg(table_name, columns)::text
  FROM (
    SELECT table_name, json_agg(column_name) AS columns
    FROM information_schema.columns
    WHERE table_schema = 'public'
    GROUP BY table_name
  ) subquery;";


  let rt = Runtime::new().unwrap();
  // let mut schema = String::new();
  let schema: Result<Option<String>, pgx::spi::Error> = Spi::get_one(schema_sql);
  if schema.is_err() {
    return format!("Error: {}", schema.err().unwrap());
  }
  if schema.as_ref().unwrap().is_none() {
    return format!("Error: {}", "No result");
  }

  let mut messages = vec![ChatCompletionMessage {
    role: ChatCompletionMessageRole::System,
    content: "You are an SQL assistent that helps translate questions into SQL".to_string(),
    name: None,
  }];
  messages.push(ChatCompletionMessage {
    role: ChatCompletionMessageRole::User,
    content: format!("Here is a schema for the database in json format: {}", schema.unwrap().unwrap()),
    name: None,
  });
  messages.push(ChatCompletionMessage {
    role: ChatCompletionMessageRole::User,
    content: format!("Please return an SQL statement as a single string for the following question: {}", input),
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
    fn test_gpt() {
      extension_sql!(
    r#"
CREATE TABLE examples (
    id serial8 not null primary key,
    title text
);
"#,
    name = "create_example_table",
);
      assert_eq!("SELECT title FROM examples;", crate::gpt("list all example titles"));
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
