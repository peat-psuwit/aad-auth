use rusqlite::{params, Connection, OpenFlags, Rows, Statement};

use log::debug;
use std::path::Path;

const DB_PATH: &str = "/var/lib/aad/cache";

#[derive(Debug)]
pub struct Passwd {
    pub name: String,
    pub passwd: String,
    pub uid: u32,
    pub gid: u32,
    pub gecos: String,
    pub home: String,
    pub shell: String,
}

#[derive(Debug)]
pub enum CacheError {
    DatabaseError(String),
    QueryError(String),
    NoRecord,
}

pub struct CacheDB {
    conn: Connection,
}

pub struct CacheDBBuilder {
    db_path: Option<String>,
}

impl CacheDBBuilder {
    pub fn with_db_path(&mut self, db_path: &str) -> &mut Self {
        debug!("using custom db path: {}", db_path);
        self.db_path = Some(db_path.to_string());
        self
    }

    pub fn build(&self) -> Result<CacheDB, CacheError> {
        let db_path = self.db_path.clone().unwrap_or(DB_PATH.to_string());
        let passwd_db = Path::new(&db_path).join("passwd.db");
        let passwd_db = passwd_db.to_str().unwrap();

        debug!("opening database connection from {}", passwd_db);

        let conn = match Connection::open_with_flags(passwd_db, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            Ok(conn) => conn,
            Err(e) => return Err(CacheError::DatabaseError(e.to_string())),
        };

        // TODO: attach shadow if root. Handle file permissions…

        Ok(CacheDB { conn: conn })
    }
}

impl CacheDB {
    pub fn new() -> CacheDBBuilder {
        CacheDBBuilder { db_path: None }
    }

    /* Passwd */

    pub fn get_passwd_from_uid(self: &CacheDB, uid: u32) -> Result<Passwd, CacheError> {
        let mut stmt = self.prepare_statement(
            "SELECT login, password, uid, gid, gecos, home, shell FROM passwd WHERE uid = ?", // Last empty field is the shadow password
        )?;

        let rows = match stmt.query(params![uid]) {
            Ok(rows) => rows,
            Err(error) => return Err(CacheError::QueryError(error.to_string())),
        };

        let mut entries = CacheDB::rows_to_passwd_entries(rows);

        let passwd = CacheDB::expect_one_row(&mut entries)?;
        Ok(passwd)
    }

    pub fn get_passwd_from_name(self: &CacheDB, login: &str) -> Result<Passwd, CacheError> {
        let mut stmt = self.prepare_statement(
            "SELECT login, password, uid, gid, gecos, home, shell FROM passwd WHERE login = ?", // Last empty field is the shadow password
        )?;

        let rows = match stmt.query(params![login]) {
            Ok(rows) => rows,
            Err(error) => return Err(CacheError::QueryError(error.to_string())),
        };

        let mut entries = CacheDB::rows_to_passwd_entries(rows);

        let passwd = CacheDB::expect_one_row(&mut entries)?;
        Ok(passwd)
    }

    pub fn get_all_passwd(self: &CacheDB) -> Result<Vec<Passwd>, CacheError> {
        let mut stmt = self.prepare_statement(
            "SELECT login, password, uid, gid, gecos, home, shell FROM passwd", // Last empty field is the shadow password
        )?;

        let rows = match stmt.query(params![]) {
            Ok(rows) => rows,
            Err(error) => return Err(CacheError::QueryError(error.to_string())),
        };

        let entries = CacheDB::rows_to_passwd_entries(rows);
        Ok(entries)
    }

    /* Common */

    fn prepare_statement(self: &CacheDB, request: &str) -> Result<Statement, CacheError> {
        let stmt = match self.conn.prepare(request) {
            Ok(stmt) => stmt,
            Err(e) => return Err(CacheError::QueryError(e.to_string())),
        };

        Ok(stmt)
    }

    fn rows_to_passwd_entries(mut rows: Rows) -> Vec<Passwd> {
        let mut entries = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            entries.push(Passwd {
                name: row.get(0).expect("invalid name"),
                passwd: row.get(1).expect("invalid passwd"),
                uid: row.get(2).expect("invalid uid"),
                gid: row.get(3).expect("invalid gid"),
                gecos: row.get(4).expect("invalid gecos"),
                home: row.get(5).expect("invalid home"),
                shell: row.get(6).expect("invalid shell"),
            });
        }
        entries
    }

    fn expect_one_row<T>(entries: &mut Vec<T>) -> Result<T, CacheError> {
        if entries.len() == 0 {
            return Err(CacheError::NoRecord);
        }

        if entries.len() > 1 {
            return Err(CacheError::DatabaseError(
                "More than one entry found".to_string(),
            ));
        }

        let entry = match entries.pop() {
            Some(passwd) => passwd,
            None => return Err(CacheError::NoRecord),
        };

        Ok(entry)
    }
}
