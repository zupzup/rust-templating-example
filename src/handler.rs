use crate::{error::Error::*, Book, WebResult, DB};
use askama::Template;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{reject, reply::html, Reply};

#[derive(Template)]
#[template(path = "book/list.html")]
struct BooklistTemplate<'a> {
    books: &'a Vec<Book>,
}

#[derive(Template)]
#[template(path = "book/new.html")]
struct NewBookTemplate {}

#[derive(Template)]
#[template(path = "book/edit.html")]
struct EditBookTemplate<'a> {
    book: &'a Book,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BookRequest {
    pub name: String,
    pub author: String,
    pub language: String,
    pub pages: i32,
}

#[derive(Template)]
#[template(path = "welcome.html")]
struct WelcomeTemplate<'a> {
    title: &'a str,
    body: &'a str,
}

pub async fn welcome_handler() -> WebResult<impl Reply> {
    let template = WelcomeTemplate {
        title: "Welcome",
        body: "To The Bookstore!",
    };
    let res = template
        .render()
        .map_err(|e| reject::custom(TemplateError(e)))?;
    Ok(html(res))
}

pub async fn books_list_handler(db: DB) -> WebResult<impl Reply> {
    match db.read() {
        Ok(books) => {
            let template = BooklistTemplate { books: &books };
            let res = template
                .render()
                .map_err(|e| reject::custom(TemplateError(e)))?;
            Ok(html(res))
        }
        Err(_) => Err(reject::custom(DBAccessError)),
    }
}

pub async fn new_book_handler() -> WebResult<impl Reply> {
    let template = NewBookTemplate {};
    let res = template
        .render()
        .map_err(|e| reject::custom(TemplateError(e)))?;
    Ok(html(res))
}

pub async fn create_book_handler(body: BookRequest, db: DB) -> WebResult<impl Reply> {
    let new_book = Book {
        id: Uuid::new_v4().to_string(),
        name: body.name,
        author: body.author,
        language: body.language,
        pages: body.pages,
        added_at: Utc::now(),
    };
    match db.write() {
        Ok(mut books) => {
            books.push(new_book);
        }
        Err(_) => return Err(reject::custom(DBAccessError)),
    };
    books_list_handler(db).await
}

pub async fn edit_book_handler(id: String, db: DB) -> WebResult<impl Reply> {
    let book = match db.read() {
        Ok(books) => match books.iter().find(|b| b.id == id) {
            Some(book) => book.clone(),
            None => return Err(reject::custom(BookNotFoundError)),
        },
        Err(_) => return Err(reject::custom(DBAccessError)),
    };

    let template = EditBookTemplate { book: &book };
    let res = template
        .render()
        .map_err(|e| reject::custom(TemplateError(e)))?;
    Ok(html(res))
}

pub async fn do_edit_book_handler(id: String, body: BookRequest, db: DB) -> WebResult<impl Reply> {
    match db.write() {
        Ok(mut books) => match books.iter_mut().find(|b| b.id == id) {
            Some(ref mut book) => {
                book.name = body.name;
                book.language = body.language;
                book.author = body.author;
                book.pages = body.pages;
            }
            None => return Err(reject::custom(BookNotFoundError)),
        },
        Err(_) => return Err(reject::custom(DBAccessError)),
    };
    books_list_handler(db).await
}

pub async fn delete_book_handler(id: String, db: DB) -> WebResult<impl Reply> {
    match db.write() {
        Ok(mut books) => {
            let mut delete_idx = None;
            for (i, b) in books.iter().enumerate() {
                if b.id == id {
                    delete_idx = Some(i);
                }
            }
            match delete_idx {
                Some(i) => {
                    books.remove(i);
                }
                None => return Err(reject::custom(BookNotFoundError)),
            }
        }
        Err(_) => return Err(reject::custom(DBAccessError)),
    };
    books_list_handler(db).await
}
