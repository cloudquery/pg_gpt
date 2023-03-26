# pg_gpt

Experimental PostgreSQL extensions that enables the use of OpenAI GPT API inside PostgreSQL and query it using natural language by sharing the schema.

**Note**: This plugins sends schema (without the data) to OpenAI GPT API, so it is not recommended to use it on production databases.
**Note**: This is an experimental plugin and not officially supported by CloudQuery.

## Installation

Requires:

* [pgx](https://github.com/tcdi/pgx)

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
