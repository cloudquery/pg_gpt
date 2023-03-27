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
  match env::var("OPENAI_KEY") {
    Ok(val) => set_key(val),
    Err(e) => println!("Couldn't interpret {}: {}", "OPENAI_KEY", e),
  }
  let rt = Runtime::new().unwrap();
  println!("sending tables query");

  let query = "SELECT table_name, column_name FROM information_schema.columns WHERE table_schema = 'public' ORDER BY table_name, ordinal_position;";
  println!("Before query execution");
  let schema: Result<
    TableIterator<
      'static,
      (
        name!(table_name, Result<Option<String>, pgx::spi::Error>),
        name!(column_name, Result<Option<String>, pgx::spi::Error>),
      ),
    >,
    spi::Error,
  > = Spi::connect(|client| {
    Ok(client.select(query, None, None)?.map(|row| (row["table_name"].value(), row["column_name"].value())))
  })
    .map(|results| {
      TableIterator::new(results)
    });
  println!("After query execution");

  let mut schema_str = String::new();
  let tables: HashMap<String, Vec<String>> = HashMap::new();
  println!("got schema");

  if schema.is_err() {
    println!("schema is error");
    return "Error".to_string();
  }
  println!("getting tables");

  match schema {
    Ok(iter_result) => {
      println!("OK got iter_result");
      for (table_name_result, column_name_result) in iter_result {
        println!("OK inside for loop");
        if table_name_result.is_err() || column_name_result.is_err() {
          println!("got error");
          return "Error".to_string();
        }
        let table_name = table_name_result.unwrap().unwrap_or_else(|| "".to_owned());
        let column_name = column_name_result.unwrap().unwrap_or_else(|| "".to_owned());
        println!("table_name: {}", table_name.as_str());
        println!("column_name: {}", column_name.as_str());
        if !tables.contains_key(table_name.as_str()) {
          schema_str.push_str(&format!("{}(", table_name.as_str()));
        }
        schema_str.push_str(&format!("{},", column_name.as_str()));
      }
      println!("done with loop");
      schema_str.push_str(")");
      println!("done with loop");
    }
    Err(e) => {
      println!("Error: {}", e);
      return "Error".to_string();
    }
  }

  println!("schema: {}", schema_str.as_str());

  let mut messages = vec![ChatCompletionMessage {
    role: ChatCompletionMessageRole::System,
    content: "You are a SQL assistant and you will return raw PostgreSQL queries without any additional words ready to execute in one line".to_string(),
    name: None,
  }];
  messages.push(ChatCompletionMessage {
    role: ChatCompletionMessageRole::User,
    content: format!("Here is the schema: {}", schema_str.as_str()),
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
