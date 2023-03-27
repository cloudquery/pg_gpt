# Postgres GPT

Postgres GPT is an Experimental PostgreSQL extension that enables use of OpenAI GPT API inside PostgreSQL. This allows you to generate SQL queries from natural language.

**Note**: This plugins sends schema (without the data) to OpenAI GPT API, so it is not recommended to use it on production databases.
**Note**: This is an experimental plugin and not officially supported by CloudQuery.

## Installation

This extension requires [pgx](https://github.com/tcdi/pgx), which needs to be installed first:

```bash
cargo install --locked cargo-pgx
cargo pgx init
```

Now you can install run the `pg_gpt` extension:

```bash
git clone https://github.com/cloudquery/pg_gpt
cd pg_gpt
export OPENAI_KEY=<YOUR_KEY>
cargo pgx run
# will drop into psql shell
```

```sql
create extension pg_gpt;
select gpt('show me all open aws s3 buckets');
-- will output the following query, so you can execute it
-- select * from aws_s3_bucket;
```

## Limitations

[TODO]
* Schema Size - X
* Introduce an API that specifies specific tables instead of uploading the whole schema
