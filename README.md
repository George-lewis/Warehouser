# Warehouser

This project created for the Shopify Summer 2022 Backend Developer Challenge

---

Hello Shopify Reviewer

My name is George Lewis, and thank you for taking a look at my project.

*I have implemented both the CSV and warehouse optional requirement, but please evaluate the warehouse option

I'm very fond of Rust and decided to challenge myself to something new by building a backend in it. Rust comes with several notable benefits like strong static typing, great serialization support, and super fast web frameworks.

I have experience building backend in other languages, like Python, but I must say that I quite enjoyed this.
The learning curve was a little steep, but once I got going, it was pretty amazing.

Alright let's get it set up.

## Setting up

### Requirements

1. You'll need The Rust Compiler and Cargo
    - I highly recommend the Rust toolchain manager, Rustup for this: https://rustup.rs/
2. Postgresql 14
    - https://www.postgresql.org/download/
    - Please make sure to set up a user account. This can be done with the PgAdmin interface or via SQL
3. Diesel CLI
    - Diesel is an ORM for Rust that I'm using, the CLI will set up the tables for us
    - You can install it from the command line with `cargo install diesel_cli`

We need to set our database url, in the root of the project there's a file `.env` with the following contents:

`DATABASE_URL=postgres://glewis:glewis@localhost/warehouser`

Kindly replace the first `glewis` with your account's username, and the second with the password.

Now you can run `diesel setup` from the project root. This will create the database for us and run some SQL commands (as defined in ./migrations).

Okay great, we should be ready to go.
Go ahead and run `cargo run` in the project root, or `cargo run --release` if you want it to be extra speedy.

Note: Compiling Rust is very CPU-intensive, this may take a minute, and make your fans spin

If all went well, you should see that the program is running.
When it starts up it outputs the line "Warehouser Startup!", so you can check to see if it's there.

If it didn't compile, or it's complaining about the env/database, my apologies.

Troubleshooting:
- If it's a Diesel/databse issue
  - Check your PG install, check that the username/password are correct in `.env`
  - Check if the table exists in Postgres, if it doesn't you can try `diesel setup` again
  - Check that the repositry is the same as when you cloned it, there's a chance that Diesel decided to regenerate the `schema.rs` file, if this happens kindly reset it
If it's a Diesel/database issue, please check your PG install, and the Diesel getting started guide (linked above).

## Using it

**I have provided a Postman collection and environment for your convenience. Please feel free to import it and test out a few endpoints.**

The server is hard-coded to bind to `127.0.0.1:8087`, if that doesn't work for you, then you may modify it in `main.rs`.

I made up a bit of a schema myself, items have a weight, value, dimensions, id, and so on.
Using the collection you can create items, delete items, and so on. Please feel free to modify the body json of the create endpoints, and note that you must set the `id` field to be unique for each item/warehouse. If you create and item with the warehouse field filled-in, Warehouser will do some work behind-the-scenes to add the item to the appropriate warehouse (if it exists).

Similarly for warehouses, you can create, delete, and add or remove items from them.
When you delete a warehouse the items it contains are reset, in that they will reside in no warehouse after the operation is complete.
If you create a warehouse with the items array filled-in, Warehouser will try to add the items to the warehouse while creating it, and fail if it can't.

## Architecture and Guide

**main.rs** is the 'main' file of the program, it connects all of the modules together and contains the entrypoint `fn main()` of the program. Inside main I load the env, establish a connection to the database, configure the web server, and begin accepting requests.

**models.rs** contains all the models used by the program. `Error` is my error type. when a function (in the service or api layer) is failable, this is what it'll return when there's an issue. This file also contains the types stored in the database, (Pg)Transport, (Pg)Dimensions, InventoryItem, and Warehouse. These types are annotated with a lot of `#[derive(..)]`, this is Rust codegen, and it pulls a lot of the weight for us in serialization/deserialization and database interactions.

**schema.rs** describes the layout of the tables, this is generated automatically by Diesel. It isn't touched by us, with the exception of correcting the `Transportation` and `Dimension` types to their Pg* variants.

**db.rs** is the database layer. While Diesel can be used directly, this thin wrapper provides a more intentful interface, and makes unit testing easier (it's hard to mock diesel functions, but these could be mocked relatively easily).

**service.rs** is the service layer, it sits between the db layer and api layer. This is where most of the business logic is implemented, like adding items to warehouses. This module re-exports all of the functions in the db layer, but also shadows a few of them with its own definitions. This allows us to avoid writing superfluous code to forward db functions like `db::get_item`, but also allows us to seamlessly override the behaviour, and do so in way that doesn't break the api layer. For example, if we later decide that we want `db::get_item` to do some extra works and checks, because all callers access it via the re-export `service::get_item` we can create the function `service::get_item`, and if we do so with a compatible definition, require few-to-none changes to any calling code.

**api.rs** is where our endpoints are defined and is the entrypoint of a request in to the system. It makes use of a heavily-genericized function `request` and manipulates the db through the indirection provided by the service layer. Here I'm making use of Actix-Web's (web framework) and Serde's (serialization framework). Actix-Web allows for a very declaritive style and allows for easily writing a fast multithreaded server with asynchronous functions. Unfortunately Diesel doesn't yet support async db operations, so these must be done in a blocking fashion for now on Actix' CPU thread pool. Serde allows me to seamlessly accept input from the path, query params, or from the request body, together with Actix a lot of that boilerplate is handled automatically.

And that's it!
