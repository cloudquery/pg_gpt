use pgx::{prelude::*};
use tokio::runtime::Runtime;

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
  gpt_tables("%", input)
}

#[pg_extern]
fn gpt_tables(table_pattern: &str, input: &str) -> String {
  match env::var("OPENAI_KEY") {
    Ok(val) => set_key(val),
    Err(_) => {
      let open_ai_key_sql = "SELECT current_setting('openai.key');";
      let open_ai_key: Result<Option<String>, pgx::spi::Error> = Spi::get_one(open_ai_key_sql);
      if open_ai_key.is_err() {
        return format!("Error: {}", open_ai_key.err().unwrap());
      }
      if open_ai_key.as_ref().unwrap().is_none() {
        return format!("Error: {}", "openai.key not set");
      }
      set_key(open_ai_key.unwrap().unwrap());
    },
  }

  let schema_sql = format!("SELECT json_object_agg(table_name, columns)::text
  FROM (
    SELECT table_name, json_object_agg(column_name, data_type) AS columns
    FROM information_schema.columns
    WHERE table_schema = 'public' AND table_name SIMILAR TO '{}' AND table_name NOT LIKE '_pg%'
    GROUP BY table_name
  ) subquery;", table_pattern);

  let rt = Runtime::new().unwrap();
  // let mut schema = String::new();
  let mut schema: Result<Option<String>, pgx::spi::Error> = Spi::get_one(schema_sql.as_str());
  if schema.is_err() {
    return format!("Error: {}", schema.err().unwrap());
  }
  if schema.as_ref().unwrap().is_none() {
    return format!("Error: {}", "No result");
  }

  let mut schema_str = schema.unwrap().unwrap();
  if schema_str.len() > 10000 {
    // if results are too long, take out the data types
    let schema_sql = format!("SELECT json_object_agg(table_name, columns)::text
  FROM (
    SELECT table_name, json_agg(column_name) AS columns
    FROM information_schema.columns
    WHERE table_schema = 'public' AND table_name SIMILAR TO '{}' AND table_name NOT LIKE '_pg%'
    GROUP BY table_name
  ) subquery;", table_pattern);

    schema = Spi::get_one(schema_sql.as_str());
    if schema.is_err() {
      return format!("Error: {}", schema.err().unwrap());
    }
    if schema.as_ref().unwrap().is_none() {
      return format!("Error: {}", "No result");
    }
    schema_str = schema.unwrap().unwrap();
  }

  let mut messages = vec![ChatCompletionMessage {
    role: ChatCompletionMessageRole::System,
    content: "You are a SQL assistant that helps translate questions into SQL.".to_string(),
    name: None,
  }];

  println!("Schema: {}", schema_str.as_str());

  messages.push(ChatCompletionMessage {
    role: ChatCompletionMessageRole::User,
    content: format!("Here is a schema for the database in json format: {}", schema_str),
    name: None,
  });
  messages.push(ChatCompletionMessage {
    role: ChatCompletionMessageRole::User,
    content: format!("Please return an SQL statement as a single string for the following question. You must respond ONLY with SQL, nothing else. Do NOT use any tables other than the ones provided. The question is: {}", input),
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

      // now test with a table pattern
      extension_sql!(
    r#"
CREATE TABLE book_authors (
    id serial8 not null primary key,
    name text
);
CREATE TABLE books (
    id serial8 not null primary key,
    author_id serial8 not null references book_authors(id),
    title text
);
"#,
    name = "create_book_tables",
);
      assert_eq!("SELECT title \nFROM books \nJOIN book_authors ON books.author_id = book_authors.id \nWHERE book_authors.name = 'Shakespeare';", crate::gpt_tables("book%" ,"list all books written by Shakespeare"));
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
