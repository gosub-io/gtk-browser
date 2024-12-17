use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct Bookmark {
    _id: usize,
    pub title: String,
    pub url: String,
    pub tags: Vec<String>,
    pub favicon: Option<Vec<u8>>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_accessed: Option<DateTime<Utc>>,
    pub last_checked: Option<DateTime<Utc>>,
}

impl Bookmark {
    pub fn new(id: usize, title: &str, url: &str, favicon: Option<Vec<u8>>, tags: Vec<String>, created_at: Option<DateTime<Utc>>, last_accessed: Option<DateTime<Utc>>, last_checked: Option<DateTime<Utc>>) -> Self {
        Self {
            _id: id,
            title: title.into(),
            url: url.into(),
            favicon,
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

pub struct BookmarkDb {
    connection: sqlite::Connection,
}

impl BookmarkDb {
    pub fn new() -> Self {
        let connection = sqlite::open("bookmarks.db").unwrap();
        Self { connection }
    }

    pub fn query(&self, q: &str) -> Vec<Bookmark> {
        let mut stmt = if q.is_empty() {
            let query = "SELECT * FROM bookmarks";
            self.connection.prepare(query).unwrap()
        } else {
            let query = "SELECT * FROM bookmarks WHERE \
                title LIKE :q OR \
                url LIKE :q OR \
                tags LIKE :q";
            let mut stmt = self.connection.prepare(query).unwrap();
            stmt.bind((":q", q)).unwrap();

            stmt
        };

        let mut rows = vec![];

        while let sqlite::State::Row = stmt.next().unwrap() {
            let id: usize = stmt.read::<i64, _>("id").unwrap_or(0) as usize;
            let title: String = stmt.read::<String, _>("title").unwrap_or_default();
            let url: String = stmt.read::<String, _>("url").unwrap_or_default();
            let favicon: Option<Vec<u8>> = stmt.read::<Option<Vec<u8>>, _>("favicon").unwrap_or_default();
            let tags_str: String = stmt.read::<String, _>("tags").unwrap_or_default();
            let created_at: String = stmt.read::<String, _>("created_at").unwrap_or_default();
            let last_accessed: String = stmt.read::<String, _>("last_accessed").unwrap_or_default();
            let last_checked: String = stmt.read::<String, _>("last_checked").unwrap_or_default();

            let tags: Vec<String> = tags_str.split(',').map(|s| s.trim().to_string()).collect();

            rows.push(Bookmark::new(
                id,
                &title,
                &url,
                favicon,
                tags,
                to_datetime(&created_at),
                to_datetime(&last_accessed),
                to_datetime(&last_checked),
            ));
        }

        rows
    }
}


fn to_datetime(dt: &str) -> Option<DateTime<Utc>> {
    let r = chrono::NaiveDateTime::parse_from_str(dt, "%Y-%m-%d %H:%M:%S")
        .map(|d| DateTime::from_naive_utc_and_offset(d, Utc));

    match r {
        Ok(dt) => Some(dt),
        Err(_) => None,
    }
}