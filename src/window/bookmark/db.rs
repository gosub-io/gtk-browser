use std::collections::HashMap;
use chrono::{DateTime, NaiveDate, Utc};

struct Bookmark {
    id: usize,
    title: String,
    url: String,
    tags: Vec<String>,
    created_at: Option<DateTime<Utc>>,
    last_accessed: Option<DateTime<Utc>>,
    last_checked: Option<DateTime<Utc>>,
}

impl Bookmark {
    pub fn new(id: usize, title: &str, url: &str, tags: Vec<String>, created_at: Option<DateTime<Utc>>, last_accessed: Option<DateTime<Utc>>, last_checked: Option<DateTime<Utc>>) -> Self {
        Self {
            id,
            title: title.into(),
            url: url.into(),
            tags,
            created_at,
            last_accessed,
            last_checked,
        }
    }

    pub fn date_to_timeago(date: Option<DateTime<Utc>>) -> String {
        match date {
            Some(date) => {
                let formatter = timeago::Formatter::new();
                formatter.convert_chrono(date, Utc::now())
            }
            None => "Never".into(),
        }
    }
}

struct BookmarkDb {
    connection: sqlite::Connection,
}

impl BookmarkDb {
    pub fn new() -> Self {
        let connection = sqlite::open("bookmarks.db").unwrap();
        Self { connection }
    }

    pub fn find_all_bookmarks(&self) -> Vec<Bookmark> {
        let query = "SELECT * FROM bookmarks";

        let mut rows = vec![];

        _ = self.connection.iterate(query, |pairs| {
            let row: HashMap<String, String> = pairs
                .iter()
                .filter_map(|&(column, value)| Some((column, value?))) // Ignore `None` values
                .collect();


            rows.push(Bookmark::new(
                row.get("id").unwrap_or("0".into()).parse().unwrap(),
                row.get("title").unwrap_or("".into()).into(),
                row.get("url").unwrap_or("".into()).into(),
                row.get("tags").unwrap_or("".into()).split(",").collect(),
                to_datetime(row.get("created_at")),
                to_datetime(row.get("last_accessed")),
                to_datetime(row.get("last_checked")),
            ));
        });

        rows
    }

    pub fn query(&self, q: &str) -> Vec<Bookmark> {
        let mut rows = vec![];

        let query = "SELECT * FROM bookmarks WHERE\
            title LIKE :q OR\
            url LIKE :q OR\
            tags LIKE :q";
        let mut statement = self.connection.prepare(query).unwrap();
        statement.bind((":q", format!("%{}%", q)) ).unwrap();


        let res = self.connection.execute(statement).unwrap();

        _ = self.connection.iterate(query, |pairs| {
            let row: HashMap<_, _> = pairs
                .iter()
                .filter_map(|&(column, value)| Some((column, value?)))
                .collect();

            rows.push(Bookmark {
                id: row.get("id").unwrap_or(&0),
                title: row.get("title").unwrap_or(&""),
                url: row.get("url").unwrap_or(&""),
                tags: row.get("tags").unwrap_or(&"").split(",").collect(),
                created_at: row.get("created_at").unwrap_or(None),
                last_accessed: row.get("last_accessed").unwrap_or(None),
                last_checked: row.get("last_checked").unwrap_or(None),
            });
        });

        rows
    }
}


fn to_datetime(dt: &str) -> Option<DateTime<Utc>> {
    let r = chrono::NaiveDateTime::parse_from_str(dt, "%Y-%m-%d %H:%M:%S")
        .map(|d| DateTime::from_naive_utc_and_offset(d, Utc))

    match r {
        Ok(dt) => Some(dt),
        Err(_) => None,
    }
}