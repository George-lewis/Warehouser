Hello Shopify Reviewer

My name is George Lewis, and thank you for taking a look at my project.

*I have implemented both the CSV and warehouse optional requirement, but please evaluate the warehouse option

I'm very fond of Rust and decided to try something new and build a backend in it.
I have experience building backend in other languages, like Python, but I must say that I quite enjoyed this
the learning curve was pretty steep, but once I got going the static typing (among other things) was a blessing.

Alrght let's get it set up.

## Setting up

You'll need The Rust Compiler, and Cargo for this project.
I highly recommend using the Rust toolchain manager, Rustup: https://rustup.rs/
Instructions follow on that link.

I built this project on top of Postgresql 14. It may work with earlier versions, but I can't guarantee that.
So, please install Postgresql if you don't have it: https://www.postgresql.org/download/

Please make sure to set up a PG (Postgresql) user account, we will need it shortly.

I used a Rust ORM for this project called Diesel. It was a minor pain to get working, but after that
it provided a very nice interface for database interactions.
Anyway, you're going to need the Diesel CLI.
You can do so with this command: `cargo install diesel_cli`
Additional reading: https://diesel.rs/guides/getting-started

Only a few more steps!
The Diesel CLI will actually create our database for us, but first you need to modify the .env file.
In there you'll see this:

`DATABASE_URL=postgres://glewis:glewis@localhost/warehouser`

I need you to replace the first `glewis` with your (PG) account's username, and the second with the password.
The last bit is the name of the database, feel free to change it, or not.

Now you can run `diesel setup` from the project root. This should create the database for us and
run some SQL commands (the ones defined in ./migrations)
and get us set up.

If Diesel decides to regenerate the `schema.rs` file, please revert it to as it is in the repository, because Diesel will generate it incorrectly and cause the program to not compile.

Okay great, we should be ready to go.
Go ahead and run `cargo run` in the project root, or `cargo run --release` if you wan't it to be extra speedy

Warning: Compiling Rust is very CPU-intensive, this may take a minute, and make your fans spin

If all went well, you should see that the program is running.
When it starts up it outputs the line "Warehouser Startup!", so you can check to see if it's there.

If it didn't compile, or it's complaining about the env/database, my apologies.
It's difficult for me to be prepared for your environment.
If it's a Diesel/database issue, please check your PG install, and the Diesel getting started guide (linked above).

## Using it

I have provided a Postman collection for your convenience.

The server is hard-coded to bind to `127.0.0.1:8087`, if that doesn't work for you may modify it in `main.rs`.

I made up a bit of a schema myself, items have a weight, a value, dimensions, id, and so on.
Using the collection you can create items, please feel free to modify the body JSON.
You can delete items, and so on. If you create and item with the warehouse field filled-in, 
Warehouser will do some work behind-the-scenes to add the item to the appropriate warehouse (if it exists).

Similarly for warehouses, you can create, delete, and add or remove items from them.
When you delete a warehouse the items it contains are reset, in that they will reside in no warehouse afterwards.
If you create a warehouse with the items array filled-in, Warehouser will try to add the items to the warehouse automatically.

## Architecture and Guide

**main.rs** is the 'main' file of the program, it connects all of the modules together and contains the entrypoint `fn main()` of the program. Inside main I load the env, establish a connection to the database, configure the web server, and begin accepting requests.

**models.rs** contains all the models used by the program. `Error` is my error type, when a function (in the service or api layer) is failable, this is what it'll produce. This file also contains the types stored in the database, (Pg)Transport, (Pg)Dimensions, InventoryItem, and Warehouse. These types are annotated with a lot of `#[derive(..)]`, this is Rust codegen, and it pulls a lot of the weight for us in serialization/deserialization and database interactions.

**schema.rs** describes the layout of the tables, this is generated automatically by Diesel and isn't touched by us, with the exception of correcting the `Transportation` and `Dimension` types to their Pg* variants.

**db.rs** is the database layer. While Diesel can be used directly, this thin wrapper provides a more intentful interface, and makes unit testing easier (it's hard to mock diesel functions, but these can be mocked relatively easily). The code could probably be simplified with a macro, but I didn't do that here.

**service.rs** is the service layer, it sits between the db layer and api layer as a bit of indirection. It offers services to the api layer by using the db layer. This file re-exports all of the functions in the db layer, but also shadows a few with its own definitions. This allows us to avoid writing superfluous code to forward db functions like `db::get_item`, but also allows us to seamlessly override the behaviour, and do so in way that doesn't break the api layer. A lot of business logic is implemented in this file, notably most of the work behind the warehouse feature.

**api.rs** is where our endpoints are defined and the entrypoint of a request in to the system. It makes use of a heavily-genericized function `request` and manipulates the db through the indirection provided by the service layer. Here I'm making use of Actix-Web's (web framework) and Serde's (serialization framework) amazing powers in a few ways. Actix-Web allows much of the code to be asynchronous, although unfortunately Diesel doesn't have async database queries yet, so the operations have to be done within Actix' CPU thread pool. Actix lets me use handy decorators on the functions to define the routes (along with the scopes in main.rs), and extractors using Serde to automatically extract and deserialize values from the request body, path, and query.

And that's it!