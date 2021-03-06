# braid [![Build Status](https://travis-ci.org/braidery/braid.svg?branch=master)](https://travis-ci.org/braidery/braid)

A graph database. This software is in the alpha state. Do not use this as your single source of truth, and do not expect peak performance.

## Features at a glance

* Support for directed, weighted, and typed graphs.
* An advanced query DSL.
* Multiple ways to work with the database:
    * Via HTTP API, and the clients that build off of it.
    * Via lua-based scripting.
    * By embedding braid directly as a library.
* Multitenancy / support for multiple accounts.
* Support for metadata.
* Pluggable underlying datastores, with built-in support for [postgres](https://www.postgresql.org/) and [rocksdb](https://github.com/facebook/rocksdb).
* Written in rust!

For more details, see:

* [An overview of the concepts in braid.](https://braidery.github.io/concepts.html)
* [The HTTP API.](https://braidery.github.io/http-api.html)
* [The scripting API.](https://braidery.github.io/scripting.html)
* [The library.](https://github.com/braidery/braid-lib)
* [The python client.](https://github.com/braidery/python-client)

## Getting started

* Make sure you have lua 5.1 installed.
* [Download the latest release for your platform.](https://github.com/braidery/braid/releases)
* Add the binaries to your `PATH`.

Now it's time to choose your own adventure...

### Postgres

If you want to use the postgres-backed datastore, following these steps:

* Create a database: `createdb braid`
* Initialize the database schema: `DATABASE_URL=postgres://localhost:5432/braid braid-db init`
* Create a new account: `DATABASE_URL=postgres://localhost:5432/braid braid-account add`.
* Start the server: `DATABASE_URL=postgres://localhost:5432/braid PORT=8000 braid-server`.
* Make a sample HTTP request to `http://localhost:8000`, with the credentials supplied when you created the account.

### RocksDB

If you want to use the rocksdb-backed datastore, follow these steps:

* Create a new account: `DATABASE_URL=rocksdb://database.rdb braid-account add`.
* Start the server: `DATABASE_URL=rocksdb://database.rdb PORT=8000 braid-server`.
* Make a sample HTTP request to `http://localhost:8000`, with the credentials supplied when you created the account.

## Applications

This exposes three applications:

* `braid-server`: For running the HTTP server.
* `braid-account`: Manages the creation and deletion of accounts.
* `braid-db`: For managing the databases underlying braid datastores. At the moment, this only has one function: to create the database schema for postgres-backed datastores, via `braid-db init`.

## Environment variables

Applications are configured via environment variables:

* `DATABASE_URL` - The connection string to the underlying database. Examples:
    * For a postgres datastore: `postgres://user:pass@localhost:5432/database-name`.
    * For a rocksdb datastore: `rocksdb://braid.rdb`. This will store data in the directory `./braid.rdb`.
* `PORT` - The port to run the server on. Defaults to `8000`.
* `SECRET` - The postgres implementation uses this as a [pepper](https://en.wikipedia.org/wiki/Pepper_%28cryptography%29) for increased security. Defaults to an empty string.
* `BRAID_SCRIPT_ROOT` - The directory housing the lua scripts. Defaults to `./scripts`.

## Install from source

If you don't want to use the pre-built releases, you can build/install from source:

* Install [rust](https://www.rust-lang.org/en-US/install.html) 1.16+ stable or nightly.
* Make sure you have liblua5.1, gcc 5+, and postgres 9.5+ installed.
* Clone the repo: `git clone git@github.com:braidery/braid.git`.
* Build/install it: `cargo install`.

## Running tests

Use `./test.sh` to run the test suite.

