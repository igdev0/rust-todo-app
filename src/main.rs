use std::{
    any::Any,
    borrow::{Borrow, BorrowMut},
    fmt::Debug,
    io::Error,
    os::macos::raw::stat,
    rc::Rc,
    vec,
};

use clap::{builder::Str, Parser};
use sqlite::{Connection, Row};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The action title
    #[arg(short, long)]
    title: String,
    /// The action description
    #[arg(short, long, default_value = "")]
    description: String,
    #[arg(short, long)]
    list: bool,
}
fn main() {
    struct Action {
        title: String,
        description: String,
        complete: bool,
    }

    struct ActionRow {
        id: i64,
        title: String,
        description: String,
        complete: bool,
    }

    // struct Action

    impl Action {
        fn new(title: String, description: String) -> Self {
            Action {
                title,
                description,
                complete: false,
            }
        }

        fn complete(&mut self) {
            self.complete = true;
        }

        fn incomplete(&mut self) {
            self.complete = false;
        }
    }
    let connection = sqlite::open("actions.db").unwrap();
    let query = "
        CREATE TABLE IF NOT EXISTS actions ( id INTEGER PRIMARY KEY, title TEXT, description TEXT, complete BOOLEAN);
    ";

    connection
        .execute(query)
        .expect("There was some error while trying to create the actions table ");

    struct Store {
        actions: Vec<ActionRow>,
        connection: Connection,
    }

    impl Store {
        fn new(connection: Connection) -> Self {
            let statement = "SELECT * FROM actions;";
            let mut prepared_statement = connection.prepare(statement).unwrap().into_iter();
            // Execute the query and parse the results into ActionRow structs

            let mut actions = vec![];
            while let Some(res) = prepared_statement.next() {
                if let Ok(_) = res {
                    let id: i64 = prepared_statement.read::<i64, _>(0).unwrap();
                    let title: String = prepared_statement.read::<String, _>(1).unwrap();
                    let description: String = prepared_statement.read::<String, _>(2).unwrap();
                    let complete_int: i64 = prepared_statement.read::<i64, _>(3).unwrap();
                    let complete = complete_int != 0;

                    actions.push(ActionRow {
                        id,
                        complete,
                        description,
                        title,
                    })
                }
            }
            drop(prepared_statement);
            Store {
                actions,
                connection,
            }
        }
        fn add_action(&mut self, action: Action) -> () {
            // let row:ActionRow = &action;
            let stmt =
                "INSERT INTO actions (id, title, description, complete) VALUES (?, ?, ?, ?);";
            // &self.actions.push(action)
            let res = self
                .connection
                .prepare(stmt)
                .unwrap()
                .into_iter()
                .bind((1, self.actions.len() as i64 + 1))
                .unwrap()
                .bind((2, action.title.as_str()))
                .unwrap()
                .bind((3, action.description.as_str()))
                .unwrap()
                .bind((4, action.complete as i64))
                .unwrap()
                .next();
        }

        fn remove_action(&mut self, id: usize) {
            &self.actions.iter().filter(|item| item.complete == false);
        }
    }

    let find_all_stmt = "SELECT * FROM actions;";

    for row in connection.prepare(find_all_stmt).unwrap().into_iter() {
        if let Ok(r) = row {
            println!("{:#?}", r);
        }
    }

    let mut store = Store::new(connection);

    let args = Args::parse();

    let action = Action {
        complete: false,
        description: args.description,
        title: args.title,
    };

    store.add_action(action);
}
