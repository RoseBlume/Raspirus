use std::time;

use log::{debug, info, warn};
use rusqlite::{params, Connection};

use crate::backend::{
    downloader::{download_all, index},
    utils::{generic::send, update_utils::insert_all},
};

#[allow(unused)]
pub struct DBOps {
    db_conn: Connection,
    db_file: String,
    file_nr: i32,
    total_files: i32,
}

impl DBOps {
    /// Returns a new `DBOps` struct with a connection to the specified database file
    /// and initializes the table if it does not exist.
    ///
    /// # Arguments
    ///
    /// * `db_file` - The path to the database file.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusqlite::Connection;
    /// use virus_scanner::backend::db_ops::DBOps;
    /// let db_ops = DBOps::new("signatures.db").unwrap();
    /// assert_eq!(db_ops.db_conn, Connection::open("signatures.db").unwrap());
    /// ```
    pub fn new(db_file: &str) -> Result<Self, rusqlite::Error> {
        let conn = match Connection::open(db_file) {
            Ok(conn) => conn,
            Err(err) => return Err(err),
        };
        info!("New database connection at: {}", db_file);

        let ret = DBOps {
            db_conn: conn,
            db_file: db_file.to_owned(),
            file_nr: 0,
            total_files: 0,
        };
        ret.init_table()?;
        Ok(ret)
    }

    /// Initializes the `signatures` table if it does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusqlite::Connection;
    /// use virus_scanner::backend::db_ops::DBOps;
    /// let db_ops = DBOps::new("signatures.db").unwrap();
    /// assert!(db_ops.init_table().is_ok());
    /// ```
    pub fn init_table(&self) -> Result<(), rusqlite::Error> {
        debug!("Creating table if not present...");
        let _ = self.db_conn.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (hash varchar(32))",
                crate::DB_TABLE
            ),
            [],
        )?;
        Ok(())
    }

    /// Creates index
    pub fn create_index(&self) -> Result<(), rusqlite::Error> {
        let _ = self.db_conn.execute(
            &format!("CREATE INDEX idx ON {} (hash)", crate::DB_TABLE),
            [],
        )?;
        Ok(())
    }

    /// Drops index
    pub fn drop_index(&self) -> Result<(), rusqlite::Error> {
        let _ = self.db_conn.execute("DROP INDEX IF EXISTS idx", [])?;
        Ok(())
    }

    /// Updates the database by downloading any missing files and inserting their hashes into the `signatures` table.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusqlite::Connection;
    /// use virus_scanner::backend::db_ops::DBOps;
    /// let mut db_ops = DBOps::new("signatures.db").unwrap();
    /// db_ops.update_db();
    /// ```
    pub fn update_db(&mut self, window: &Option<tauri::Window>) -> Result<u32, std::io::Error> {
        info!("Updating database...");
        let max_file = index()?;
        send(window, "dwld", String::from("0"));
        download_all(max_file, window)?;
        send(window, "ins", String::from("0"));
        debug!("Moving old dataset to backup before writing...");
        // move table to backup
        self.rename("signatures", "old")
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
        insert_all(self, window).map_err(|err_1| {
            warn!("Failed to write, undoing backup...");
            // undo table move
            match self.drop("signatures").map_err(|err| err.to_string()) {
                Ok(_) => {}
                Err(err) => return std::io::Error::new(std::io::ErrorKind::Other, err),
            }
            match self
                .rename("old", "signatures")
                .map_err(|err| err.to_string())
            {
                Ok(_) => {}
                Err(err) => return std::io::Error::new(std::io::ErrorKind::Other, err),
            }
            std::io::Error::new(std::io::ErrorKind::Other, err_1)
        })?;

        send(window, "idx", String::new());
        debug!("Dropping old dataset...");
        self.drop("old")
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
        debug!("Droping old index...");
        self.drop_index()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
        debug!("Vacuuming...");
        self.vacuum()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
        debug!("Creating index for {}...", crate::DB_TABLE);
        self.create_index()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
        let hashes = self
            .count_hashes()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
        info!("Total hashes in DB: {}", hashes);
        Ok(hashes)
    }

    /// Drops provided table, removes index and cleans database
    pub fn drop(&mut self, tablename: &str) -> Result<(), rusqlite::Error> {
        let _ = self
            .db_conn
            .execute(&format!("DROP TABLE {tablename}"), [])?;
        Ok(())
    }

    pub fn vacuum(&mut self) -> Result<(), rusqlite::Error> {
        let _ = self.db_conn.execute("VACUUM", [])?;
        Ok(())
    }

    /// Renames old table to new
    pub fn rename(
        &mut self,
        old_tablename: &str,
        new_tablename: &str,
    ) -> Result<(), rusqlite::Error> {
        // remove possible conflict
        let _ = self
            .db_conn
            .execute(&format!("DROP TABLE IF EXISTS {new_tablename} "), [])?;
        let _ = self.db_conn.execute(
            &format!("ALTER TABLE {old_tablename} RENAME TO {new_tablename} "),
            [],
        )?;
        Ok(())
    }

    /// Inserts the given hashes into the signatures table.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusqlite::Connection;
    /// use virus_scanner::backend::db_ops::DBOps;
    /// let mut db_ops = DBOps::new("signatures.db").unwrap();
    /// db_ops.insert_hashes(vec!["abcdef".to_owned()]).unwrap();
    /// ```
    pub fn insert_hashes(&mut self, hashes: &Vec<String>) -> Result<usize, rusqlite::Error> {
        self.init_table()?;
        info!("Inserting {} hashes...", hashes.len());
        let transact = self.db_conn.transaction()?;

        let big_tic = time::Instant::now();
        let mut inserted = 0;
        let mut skipped = 0;
        for hash in hashes {
            match transact.execute(
                &format!("INSERT INTO {}(hash) VALUES (?)", crate::DB_TABLE),
                [hash.clone()],
            ) {
                Ok(_) => inserted += 1,
                Err(err) => {
                    warn!("Got {err} when trying to insert {hash}. Skipping...");
                    skipped += 1;
                }
            }
        }
        transact.commit()?;
        let big_toc = time::Instant::now();
        info!(
            "=> Inserted: {}, Skipped: {}, Time: {} seconds",
            inserted,
            skipped,
            big_toc.duration_since(big_tic).as_secs_f64()
        );
        Ok(inserted)
    }

    /// Returns true or false depending on if the given hash gets found in the database
    ///
    /// # Examples
    ///
    /// ```
    /// use rusqlite::Connection;
    /// use virus_scanner::backend::db_ops::DBOps;
    /// let db_ops = DBOps::new("signatures.db").unwrap();
    /// assert_eq!(db_ops.hash_exists("abcd1234").unwrap(), false);
    /// ```
    pub fn hash_exists(&self, hash_str: &str) -> Result<bool, rusqlite::Error> {
        info!("Searching hash: {}", hash_str);

        let mut stmt = self.db_conn.prepare(&format!(
            "SELECT COUNT(*) FROM {} WHERE hash = ?",
            crate::DB_TABLE
        ))?;
        let count: i64 = stmt.query_row(params![hash_str], |row| row.get(0))?;

        Ok(count > 0)
    }

    /// Returns the number of hashes in the `signatures` table.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusqlite::Connection;
    /// use virus_scanner::backend::db_ops::DBOps;
    /// let db_ops = DBOps::new("signatures.db").unwrap();
    /// assert_eq!(db_ops.count_hashes().unwrap(), 0);
    /// ```
    pub fn count_hashes(&self) -> Result<u32, rusqlite::Error> {
        let mut stmt = self
            .db_conn
            .prepare(&format!("SELECT COUNT(*) FROM {}", crate::DB_TABLE))?;
        let count: u32 = stmt.query_row([], |row| row.get(0))?;
        Ok(count as u32)
    }

    /// Removes a vec of specified hashes from the `signatures` table.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusqlite::Connection;
    /// use virus_scanner::backend::db_ops::DBOps;
    /// let db_ops = DBOps::new("signatures.db").unwrap();
    /// assert!(db_ops._remove_hash(vec!["abcd1234".to_owned()]).is_ok());
    /// ```
    pub fn remove_hashes(&mut self, hashes: &Vec<String>) -> Result<usize, rusqlite::Error> {
        self.init_table()?;
        info!("Removing {} hashes...", hashes.len());
        let transact = self.db_conn.transaction()?;

        let big_tic = time::Instant::now();
        let mut removed = 0;
        let mut skipped = 0;
        for hash in hashes {
            match transact.execute(
                &format!("DELETE FROM {} WHERE hash = ?", crate::DB_TABLE),
                [hash.clone()],
            ) {
                Ok(_) => removed += 1,
                Err(err) => {
                    warn!("Got {err} when trying to delete {hash}. Skipping...");
                    skipped += 1;
                }
            }
        }
        transact.commit()?;
        let big_toc = time::Instant::now();
        info!(
            "=> Removed: {}, Skipped: {}, Time: {} seconds",
            removed,
            skipped,
            big_toc.duration_since(big_tic).as_secs_f64()
        );
        Ok(removed)
    }
}
