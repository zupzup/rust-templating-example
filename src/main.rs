use chrono::prelude::*;
use std::convert::Infallible;
use std::sync::{Arc, RwLock};
use warp::{Filter, Rejection};

type WebResult<T> = std::result::Result<T, Rejection>;

mod error;
mod handler;

#[derive(Clone, Debug)]
pub struct Book {
    pub id: String,
    pub name: String,
    pub author: String,
    pub language: String,
    pub pages: i32,
    pub added_at: DateTime<Utc>,
}

type DB = Arc<RwLock<Vec<Book>>>;

#[tokio::main]
async fn main() {
    let db = DB::default();

    let books = warp::path("books");
    let new = warp::path("new");
    let list = warp::path("list");
    let edit = warp::path("edit");
    let delete = warp::path("delete");

    let welcome_route = warp::path::end().and_then(handler::welcome_handler);

    let books_routes = books
        .and(new)
        .and(warp::get())
        .and_then(handler::new_book_handler)
        .or(books
            .and(new)
            .and(warp::post())
            .and(warp::body::form())
            .and(with_db(db.clone()))
            .and_then(handler::create_book_handler))
        .or(books
            .and(edit)
            .and(warp::get())
            .and(warp::path::param())
            .and(with_db(db.clone()))
            .and_then(handler::edit_book_handler))
        .or(books
            .and(edit)
            .and(warp::post())
            .and(warp::path::param())
            .and(warp::body::form())
            .and(with_db(db.clone()))
            .and_then(handler::do_edit_book_handler))
        .or(books
            .and(delete)
            .and(warp::get())
            .and(warp::path::param())
            .and(with_db(db.clone()))
            .and_then(handler::delete_book_handler))
        .or(books
            .and(list)
            .and(warp::get())
            .and(with_db(db.clone()))
            .and_then(handler::books_list_handler));

    let routes = welcome_route
        .or(books_routes)
        .recover(error::handle_rejection);

    println!("Started on port 8080");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

fn with_db(db: DB) -> impl Filter<Extract = (DB,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}
