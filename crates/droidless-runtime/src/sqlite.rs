use crate::{
    heap::{Data, SqlValue, Word, bits64, fault, wide},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::dex::Method;
use rusqlite::{
    Connection, params_from_iter,
    types::{Value, ValueRef},
};
use std::sync::{Arc, Mutex};

const OPEN_HELPER: &str = "Landroid/database/sqlite/SQLiteOpenHelper;";
const DATABASE: &str = "Landroid/database/sqlite/SQLiteDatabase;";
const STATEMENT: &str = "Landroid/database/sqlite/SQLiteStatement;";

impl Runtime {
    pub(crate) fn sqlite_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        match method.class.as_str() {
            OPEN_HELPER => self.sqlite_helper_native(method, args),
            DATABASE => self.sqlite_database_native(method, args),
            STATEMENT => self.sqlite_statement_native(method, args),
            "Landroid/database/Cursor;" => self.cursor_native(method, args),
            "Landroid/content/ContentValues;" => self.content_values_native(method, args),
            _ => Ok(None),
        }
    }

    fn sqlite_helper_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let signature = method.signature();
        let receiver = *args.first().context("SQLiteOpenHelper receiver missing")?;
        match signature.as_str() {
            "<init>(Landroid/content/Context;Ljava/lang/String;Landroid/database/sqlite/SQLiteDatabase$CursorFactory;I)V" =>
            {
                let name = *args
                    .get(2)
                    .context("SQLiteOpenHelper database name missing")?;
                let version = args
                    .get(4)
                    .copied()
                    .context("SQLiteOpenHelper version missing")?
                    .int()?;
                if version < 1 {
                    return Err(fault(
                        "Ljava/lang/IllegalArgumentException;",
                        "database version must be at least 1",
                    ));
                }
                let helper = self.heap.get_mut(receiver)?;
                helper
                    .fields
                    .insert("droidless:sqlite:name".into(), vec![name]);
                helper
                    .fields
                    .insert("droidless:sqlite:version".into(), vec![Word::from(version)]);
                Ok(Some(vec![]))
            }
            "getWritableDatabase()Landroid/database/sqlite/SQLiteDatabase;"
            | "getReadableDatabase()Landroid/database/sqlite/SQLiteDatabase;" => {
                if let Some(database) = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:sqlite:database")
                    .and_then(|values| values.first())
                    .copied()
                    .filter(|value| *value != Word::ZERO)
                {
                    return Ok(Some(vec![database]));
                }
                let name = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:sqlite:name")
                    .and_then(|values| values.first())
                    .copied()
                    .context("uninitialized SQLiteOpenHelper")?;
                let version = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:sqlite:version")
                    .and_then(|values| values.first())
                    .copied()
                    .context("uninitialized SQLiteOpenHelper")?
                    .int()?;
                let key = if name == Word::ZERO {
                    format!(
                        "memory:{}:{}",
                        self.apk.manifest.package,
                        receiver.reference()?
                    )
                } else {
                    let name = self.heap.text(name)?;
                    format!("/data/data/{}/databases/{name}", self.apk.manifest.package)
                };
                let relative = if key.starts_with("/") {
                    Some(self.guest_file_relative(&key).ok_or_else(|| {
                        fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "database path escapes app-private storage",
                        )
                    })?)
                } else {
                    None
                };
                if let Some(path) = relative.as_ref()
                    && let Some(storage) = self.storage.as_ref()
                {
                    storage.ensure_app_dir(&path[..path.len() - 1])?;
                }
                let existed = self.databases.contains_key(&key)
                    || if let (Some(storage), Some(path)) = (self.storage.as_ref(), &relative) {
                        storage
                            .read_app_file(path)?
                            .is_some_and(|bytes| !bytes.is_empty())
                    } else {
                        false
                    };
                let connection = self.database_connection(&key, relative.as_deref())?;
                let database = self.heap.instance(DATABASE)?;
                self.heap.get_mut(database)?.data = Data::SqlDatabase(key.clone());
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:sqlite:database".into(), vec![database]);
                self.invoke_sqlite_callback(receiver, database, "onConfigure", vec![])?;
                let current = {
                    let connection = connection
                        .lock()
                        .map_err(|_| anyhow::anyhow!("SQLite connection poisoned"))?;
                    connection
                        .query_row("PRAGMA user_version", [], |row| row.get::<_, i32>(0))
                        .map_err(sql_fault)?
                };
                if !existed || current == 0 || current != version {
                    connection
                        .lock()
                        .map_err(|_| anyhow::anyhow!("SQLite connection poisoned"))?
                        .execute_batch("BEGIN IMMEDIATE")
                        .map_err(sql_fault)?;
                    self.database_transactions.insert(key.clone(), vec![false]);
                    let migration = (|| -> Result<()> {
                        let (callback, extras) = if !existed || current == 0 {
                            ("onCreate", vec![])
                        } else if current < version {
                            ("onUpgrade", vec![Word::from(current), Word::from(version)])
                        } else {
                            (
                                "onDowngrade",
                                vec![Word::from(current), Word::from(version)],
                            )
                        };
                        self.invoke_sqlite_callback(receiver, database, callback, extras)?;
                        let connection = connection
                            .lock()
                            .map_err(|_| anyhow::anyhow!("SQLite connection poisoned"))?;
                        connection
                            .execute_batch(&format!("PRAGMA user_version = {version}"))
                            .map_err(sql_fault)?;
                        connection.execute_batch("COMMIT").map_err(sql_fault)?;
                        Ok(())
                    })();
                    self.database_transactions.remove(&key);
                    if let Err(error) = migration {
                        let _ = connection
                            .lock()
                            .map(|connection| connection.execute_batch("ROLLBACK"));
                        return Err(error);
                    }
                    self.persist_database(&key)?;
                }
                self.invoke_sqlite_callback(receiver, database, "onOpen", vec![])?;
                Ok(Some(vec![database]))
            }
            "close()V" => {
                self.heap.get(receiver)?;
                Ok(Some(vec![]))
            }
            "onConfigure(Landroid/database/sqlite/SQLiteDatabase;)V"
            | "onOpen(Landroid/database/sqlite/SQLiteDatabase;)V" => {
                self.heap.get(receiver)?;
                Ok(Some(vec![]))
            }
            _ => Ok(None),
        }
    }

    fn invoke_sqlite_callback(
        &mut self,
        helper: Word,
        database: Word,
        name: &str,
        extras: Vec<Word>,
    ) -> Result<()> {
        let parameters = match name {
            "onUpgrade" | "onDowngrade" => vec![
                "Landroid/database/sqlite/SQLiteDatabase;".into(),
                "I".into(),
                "I".into(),
            ],
            _ => vec!["Landroid/database/sqlite/SQLiteDatabase;".into()],
        };
        self.invoke(
            Method {
                class: OPEN_HELPER.into(),
                name: name.into(),
                parameters,
                returns: "V".into(),
            },
            std::iter::once(helper)
                .chain(std::iter::once(database))
                .chain(extras)
                .collect(),
            true,
        )?;
        Ok(())
    }

    fn database_connection(
        &mut self,
        key: &str,
        path: Option<&[String]>,
    ) -> Result<Arc<Mutex<Connection>>> {
        if let Some(connection) = self.databases.get(key) {
            return Ok(connection.clone());
        }
        let mut connection = Connection::open_in_memory().map_err(sql_fault)?;
        if let (Some(storage), Some(path)) = (&self.storage, path)
            && let Some(bytes) = storage.read_app_file(path)?
            && !bytes.is_empty()
        {
            connection
                .deserialize_read_exact("main", bytes.as_slice(), bytes.len(), false)
                .map_err(sql_fault)?;
        }
        let connection = Arc::new(Mutex::new(connection));
        self.databases.insert(key.into(), connection.clone());
        Ok(connection)
    }

    fn persist_database(&mut self, key: &str) -> Result<()> {
        if self.database_transactions.contains_key(key) || !key.starts_with('/') {
            return Ok(());
        }
        let Some(path) = self.guest_file_relative(key) else {
            return Err(fault(
                "Ljava/lang/IllegalArgumentException;",
                "database path escapes app-private storage",
            ));
        };
        let connection = self
            .databases
            .get(key)
            .cloned()
            .context("SQLite connection missing")?;
        let bytes = connection
            .lock()
            .map_err(|_| anyhow::anyhow!("SQLite connection poisoned"))?
            .serialize("main")
            .map_err(sql_fault)?
            .to_vec();
        if let Some(storage) = self.storage.as_mut() {
            storage.write_app_file(&path, &bytes)?;
        }
        Ok(())
    }

    fn sqlite_database_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let receiver = *args.first().context("SQLiteDatabase receiver missing")?;
        let key = match &self.heap.get(receiver)?.data {
            Data::SqlDatabase(key) => key.clone(),
            _ => bail!("uninitialized SQLiteDatabase"),
        };
        let connection = self
            .databases
            .get(&key)
            .cloned()
            .context("SQLite connection missing")?;
        let arg = |index: usize| {
            args.get(index)
                .copied()
                .context("SQLiteDatabase argument missing")
        };
        match method.signature().as_str() {
            "getVersion()I" => {
                let version = connection
                    .lock()
                    .map_err(|_| anyhow::anyhow!("SQLite connection poisoned"))?
                    .query_row("PRAGMA user_version", [], |row| row.get::<_, i32>(0))
                    .map_err(sql_fault)?;
                Ok(Some(vec![Word::from(version)]))
            }
            "execSQL(Ljava/lang/String;)V" => {
                let sql = self.heap.text(arg(1)?)?.to_owned();
                connection
                    .lock()
                    .map_err(|_| anyhow::anyhow!("SQLite connection poisoned"))?
                    .execute_batch(&sql)
                    .map_err(sql_fault)?;
                self.persist_database(&key)?;
                Ok(Some(vec![]))
            }
            "execSQL(Ljava/lang/String;[Ljava/lang/Object;)V" => {
                let sql = self.heap.text(arg(1)?)?.to_owned();
                let values = self.sql_values_array(arg(2)?)?;
                connection
                    .lock()
                    .map_err(|_| anyhow::anyhow!("SQLite connection poisoned"))?
                    .execute(&sql, params_from_iter(values.iter().map(sql_value)))
                    .map_err(sql_fault)?;
                self.persist_database(&key)?;
                Ok(Some(vec![]))
            }
            "compileStatement(Ljava/lang/String;)Landroid/database/sqlite/SQLiteStatement;" => {
                let sql = self.heap.text(arg(1)?)?.to_owned();
                let parameter_count = {
                    let connection = connection
                        .lock()
                        .map_err(|_| anyhow::anyhow!("SQLite connection poisoned"))?;
                    connection
                        .prepare(&sql)
                        .map_err(sql_fault)?
                        .parameter_count()
                };
                let statement = self.heap.instance(STATEMENT)?;
                self.heap.get_mut(statement)?.data = Data::SqlStatement {
                    database: key,
                    sql,
                    bindings: vec![SqlValue::Null; parameter_count],
                };
                Ok(Some(vec![statement]))
            }
            "rawQuery(Ljava/lang/String;[Ljava/lang/String;)Landroid/database/Cursor;" => {
                let sql = self.heap.text(arg(1)?)?.to_owned();
                let arguments = arg(2)?;
                let values = if arguments == Word::ZERO {
                    vec![]
                } else {
                    self.sql_values_array(arguments)?
                };
                let (columns, rows) = {
                    let connection = connection
                        .lock()
                        .map_err(|_| anyhow::anyhow!("SQLite connection poisoned"))?;
                    let mut statement = connection.prepare(&sql).map_err(sql_fault)?;
                    let columns = statement
                        .column_names()
                        .iter()
                        .map(|name| (*name).to_owned())
                        .collect::<Vec<_>>();
                    let column_count = statement.column_count();
                    let mut query = statement
                        .query(params_from_iter(values.iter().map(sql_value)))
                        .map_err(sql_fault)?;
                    let mut rows = Vec::new();
                    while let Some(row) = query.next().map_err(sql_fault)? {
                        if rows.len() >= 100_000 {
                            return Err(fault(
                                "Landroid/database/sqlite/SQLiteException;",
                                "query result exceeds 100000 rows",
                            ));
                        }
                        let mut values = Vec::with_capacity(column_count);
                        for index in 0..column_count {
                            values.push(read_value(row.get_ref(index).map_err(sql_fault)?));
                        }
                        rows.push(values);
                    }
                    (columns, rows)
                };
                let cursor = self.heap.instance("Landroid/database/Cursor;")?;
                self.heap.get_mut(cursor)?.data = Data::Cursor {
                    columns,
                    rows,
                    position: -1,
                    closed: false,
                };
                Ok(Some(vec![cursor]))
            }
            "beginTransaction()V" => {
                let depth = self.database_transactions.get(&key).map_or(0, Vec::len);
                let statement = if depth == 0 {
                    "BEGIN".into()
                } else {
                    format!("SAVEPOINT droidless_{depth}")
                };
                connection
                    .lock()
                    .map_err(|_| anyhow::anyhow!("SQLite connection poisoned"))?
                    .execute_batch(&statement)
                    .map_err(sql_fault)?;
                self.database_transactions
                    .entry(key)
                    .or_default()
                    .push(false);
                Ok(Some(vec![]))
            }
            "setTransactionSuccessful()V" => {
                let Some(successful) = self
                    .database_transactions
                    .get_mut(&key)
                    .and_then(|transactions| transactions.last_mut())
                else {
                    return Err(fault(
                        "Ljava/lang/IllegalStateException;",
                        "no SQLite transaction is active",
                    ));
                };
                *successful = true;
                Ok(Some(vec![]))
            }
            "endTransaction()V" => {
                let transactions = self
                    .database_transactions
                    .get(&key)
                    .context("no SQLite transaction is active")?;
                let depth = transactions.len();
                ensure!(depth > 0, "empty SQLite transaction stack");
                let successful = transactions[depth - 1];
                let statement = if depth == 1 {
                    if successful { "COMMIT" } else { "ROLLBACK" }.into()
                } else if successful {
                    format!("RELEASE SAVEPOINT droidless_{}", depth - 1)
                } else {
                    format!(
                        "ROLLBACK TO SAVEPOINT droidless_{}; RELEASE SAVEPOINT droidless_{}",
                        depth - 1,
                        depth - 1
                    )
                };
                connection
                    .lock()
                    .map_err(|_| anyhow::anyhow!("SQLite connection poisoned"))?
                    .execute_batch(&statement)
                    .map_err(sql_fault)?;
                let transactions = self
                    .database_transactions
                    .get_mut(&key)
                    .context("SQLite transaction disappeared")?;
                transactions.pop();
                if !successful && let Some(parent) = transactions.last_mut() {
                    *parent = false;
                }
                if transactions.is_empty() {
                    self.database_transactions.remove(&key);
                }
                if successful && depth == 1 {
                    self.persist_database(&key)?;
                }
                Ok(Some(vec![]))
            }
            "close()V" => {
                self.persist_database(&key)?;
                Ok(Some(vec![]))
            }
            _ => Ok(None),
        }
    }

    fn sql_values_array(&self, array: Word) -> Result<Vec<SqlValue>> {
        let object = self.heap.get(array)?;
        let Data::Array { element, values } = &object.data else {
            return Err(fault(
                "Ljava/lang/IllegalArgumentException;",
                "SQL arguments must be an object array",
            ));
        };
        if !element.starts_with('L') {
            return Err(fault(
                "Ljava/lang/IllegalArgumentException;",
                "SQL arguments must be an object array",
            ));
        }
        values
            .iter()
            .map(|value| {
                self.sql_value(
                    value
                        .first()
                        .copied()
                        .context("SQL argument has no value")?,
                )
            })
            .collect()
    }

    fn sql_value(&self, value: Word) -> Result<SqlValue> {
        if value.reference()? == 0 {
            return Ok(SqlValue::Null);
        }
        let object = self.heap.get(value)?;
        match object.class.as_str() {
            "Ljava/lang/String;" => Ok(SqlValue::Text(self.heap.text(value)?.to_owned())),
            "[B" => {
                let Data::Array { values, .. } = &object.data else {
                    bail!("invalid SQL blob");
                };
                Ok(SqlValue::Blob(
                    values
                        .iter()
                        .map(|value| value[0].int().map(|byte| byte as i8 as u8))
                        .collect::<Result<Vec<_>>>()?,
                ))
            }
            "Ljava/lang/Boolean;" => Ok(SqlValue::Integer(i64::from(
                object
                    .fields
                    .get("value")
                    .and_then(|value| value.first())
                    .copied()
                    .context("uninitialized Boolean")?
                    .int()?
                    != 0,
            ))),
            "Ljava/lang/Byte;"
            | "Ljava/lang/Character;"
            | "Ljava/lang/Short;"
            | "Ljava/lang/Integer;" => Ok(SqlValue::Integer(i64::from(
                object
                    .fields
                    .get("value")
                    .and_then(|value| value.first())
                    .copied()
                    .context("uninitialized integer wrapper")?
                    .int()?,
            ))),
            "Ljava/lang/Long;" => Ok(SqlValue::Integer(bits64(
                object.fields.get("value").context("uninitialized Long")?,
            )? as i64)),
            "Ljava/lang/Float;" => Ok(SqlValue::Real(f64::from(f32::from_bits(
                object
                    .fields
                    .get("value")
                    .and_then(|value| value.first())
                    .copied()
                    .context("uninitialized Float")?
                    .int()? as u32,
            )))),
            "Ljava/lang/Double;" => Ok(SqlValue::Real(f64::from_bits(bits64(
                object.fields.get("value").context("uninitialized Double")?,
            )?))),
            _ => Err(fault(
                "Ljava/lang/IllegalArgumentException;",
                format!("unsupported SQLite value class {}", object.class),
            )),
        }
    }

    fn sqlite_statement_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let receiver = *args.first().context("SQLiteStatement receiver missing")?;
        let signature = method.signature();
        let arg = |index: usize| {
            args.get(index)
                .copied()
                .context("SQLiteStatement argument missing")
        };
        if signature.starts_with("bind") {
            let index = arg(1)?.int()?;
            if !(1..=32_766).contains(&index) {
                return Err(fault(
                    "Landroid/database/sqlite/SQLiteException;",
                    "invalid SQLite binding index",
                ));
            }
            let value = match signature.as_str() {
                "bindNull(I)V" => SqlValue::Null,
                "bindLong(IJ)V" => SqlValue::Integer(bits64(&args[2..])? as i64),
                "bindString(ILjava/lang/String;)V" => {
                    SqlValue::Text(self.heap.text(arg(2)?)?.to_owned())
                }
                "bindBlob(I[B)V" => self.sql_value(arg(2)?)?,
                _ => return Ok(None),
            };
            let Data::SqlStatement { bindings, .. } = &mut self.heap.get_mut(receiver)?.data else {
                bail!("uninitialized SQLiteStatement");
            };
            if index as usize > bindings.len() {
                return Err(fault(
                    "Landroid/database/sqlite/SQLiteException;",
                    "SQLite binding index exceeds parameter count",
                ));
            }
            bindings[index as usize - 1] = value;
            return Ok(Some(vec![]));
        }
        if signature == "clearBindings()V" {
            let Data::SqlStatement { bindings, .. } = &mut self.heap.get_mut(receiver)?.data else {
                bail!("uninitialized SQLiteStatement");
            };
            bindings.clear();
            return Ok(Some(vec![]));
        }
        if signature == "close()V" {
            self.heap.get(receiver)?;
            return Ok(Some(vec![]));
        }
        let (database, sql, bindings) = match &self.heap.get(receiver)?.data {
            Data::SqlStatement {
                database,
                sql,
                bindings,
            } => (database.clone(), sql.clone(), bindings.clone()),
            _ => bail!("uninitialized SQLiteStatement"),
        };
        let connection = self
            .databases
            .get(&database)
            .cloned()
            .context("SQLite connection missing")?;
        match signature.as_str() {
            "execute()V" | "executeInsert()J" | "executeUpdateDelete()I" => {
                let (changes, row_id) = {
                    let connection = connection
                        .lock()
                        .map_err(|_| anyhow::anyhow!("SQLite connection poisoned"))?;
                    let changes = connection
                        .execute(&sql, params_from_iter(bindings.iter().map(sql_value)))
                        .map_err(sql_fault)?;
                    (changes, connection.last_insert_rowid())
                };
                self.persist_database(&database)?;
                Ok(Some(match signature.as_str() {
                    "execute()V" => vec![],
                    "executeInsert()J" => wide(row_id as u64),
                    _ => vec![Word::from(changes as i32)],
                }))
            }
            "simpleQueryForLong()J" => {
                let value = connection
                    .lock()
                    .map_err(|_| anyhow::anyhow!("SQLite connection poisoned"))?
                    .query_row(
                        &sql,
                        params_from_iter(bindings.iter().map(sql_value)),
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(|error| match error {
                        rusqlite::Error::QueryReturnedNoRows => fault(
                            "Landroid/database/sqlite/SQLiteDoneException;",
                            error.to_string(),
                        ),
                        error => sql_fault(error),
                    })?;
                Ok(Some(wide(value as u64)))
            }
            _ => Ok(None),
        }
    }

    fn content_values_native(
        &mut self,
        _method: &Method,
        _args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        Ok(None)
    }

    fn cursor_native(&mut self, method: &Method, args: &[Word]) -> Result<Option<Vec<Word>>> {
        let receiver = *args.first().context("Cursor receiver missing")?;
        let signature = method.signature();
        match signature.as_str() {
            "close()V" => {
                let Data::Cursor { closed, .. } = &mut self.heap.get_mut(receiver)?.data else {
                    bail!("uninitialized Cursor");
                };
                *closed = true;
                Ok(Some(vec![]))
            }
            "isClosed()Z" => {
                let Data::Cursor { closed, .. } = &self.heap.get(receiver)?.data else {
                    bail!("uninitialized Cursor");
                };
                Ok(Some(vec![Word::from(i32::from(*closed))]))
            }
            "getCount()I" | "getPosition()I" => {
                let Data::Cursor {
                    rows,
                    position,
                    closed,
                    ..
                } = &self.heap.get(receiver)?.data
                else {
                    bail!("uninitialized Cursor");
                };
                ensure_cursor_open(*closed)?;
                let value = if signature == "getCount()I" {
                    rows.len() as i32
                } else {
                    *position
                };
                Ok(Some(vec![Word::from(value)]))
            }
            "moveToFirst()Z" | "moveToNext()Z" | "moveToPosition(I)Z" => {
                let requested = if signature == "moveToFirst()Z" {
                    0
                } else if signature == "moveToNext()Z" {
                    match &self.heap.get(receiver)?.data {
                        Data::Cursor { position, .. } => position.saturating_add(1),
                        _ => bail!("uninitialized Cursor"),
                    }
                } else {
                    args.get(1).context("Cursor position missing")?.int()?
                };
                let Data::Cursor {
                    rows,
                    position,
                    closed,
                    ..
                } = &mut self.heap.get_mut(receiver)?.data
                else {
                    bail!("uninitialized Cursor");
                };
                ensure_cursor_open(*closed)?;
                let count = rows.len() as i32;
                *position = requested.clamp(-1, count);
                Ok(Some(vec![Word::from(i32::from(
                    requested >= 0 && requested < count,
                ))]))
            }
            "getColumnIndex(Ljava/lang/String;)I"
            | "getColumnIndexOrThrow(Ljava/lang/String;)I" => {
                let name = self
                    .heap
                    .text(*args.get(1).context("column name missing")?)?;
                let Data::Cursor {
                    columns, closed, ..
                } = &self.heap.get(receiver)?.data
                else {
                    bail!("uninitialized Cursor");
                };
                ensure_cursor_open(*closed)?;
                let index = columns
                    .iter()
                    .position(|column| column.eq_ignore_ascii_case(name));
                if signature.starts_with("getColumnIndexOrThrow") && index.is_none() {
                    return Err(fault(
                        "Ljava/lang/IllegalArgumentException;",
                        format!("column '{name}' does not exist"),
                    ));
                }
                Ok(Some(vec![Word::from(
                    index.map_or(-1, |index| index as i32),
                )]))
            }
            "getString(I)Ljava/lang/String;"
            | "getInt(I)I"
            | "getLong(I)J"
            | "getBlob(I)[B"
            | "isNull(I)Z" => {
                let value = self.cursor_value(receiver, args.get(1).copied())?;
                match signature.as_str() {
                    "getString(I)Ljava/lang/String;" => match value {
                        SqlValue::Null => Ok(Some(vec![Word::ZERO])),
                        SqlValue::Text(value) => Ok(Some(vec![self.heap.string(value)?])),
                        SqlValue::Integer(value) => {
                            Ok(Some(vec![self.heap.string(value.to_string())?]))
                        }
                        SqlValue::Real(value) => {
                            Ok(Some(vec![self.heap.string(value.to_string())?]))
                        }
                        SqlValue::Blob(value) => Ok(Some(vec![
                            self.heap
                                .string(String::from_utf8_lossy(&value).into_owned())?,
                        ])),
                    },
                    "getInt(I)I" => Ok(Some(vec![Word::from(sql_integer(&value)? as i32)])),
                    "getLong(I)J" => Ok(Some(wide(sql_integer(&value)? as u64))),
                    "getBlob(I)[B" => match value {
                        SqlValue::Null => Ok(Some(vec![Word::ZERO])),
                        SqlValue::Blob(value) => Ok(Some(vec![self.byte_array(&value)?])),
                        SqlValue::Text(value) => Ok(Some(vec![self.byte_array(value.as_bytes())?])),
                        SqlValue::Integer(value) => {
                            Ok(Some(vec![self.byte_array(value.to_string().as_bytes())?]))
                        }
                        SqlValue::Real(value) => {
                            Ok(Some(vec![self.byte_array(value.to_string().as_bytes())?]))
                        }
                    },
                    _ => Ok(Some(vec![Word::from(i32::from(matches!(
                        value,
                        SqlValue::Null
                    )))])),
                }
            }
            "getExtras()Landroid/os/Bundle;" => {
                let bundle = self.heap.instance("Landroid/os/Bundle;")?;
                self.heap.get_mut(bundle)?.data = Data::Bundle(Default::default());
                Ok(Some(vec![bundle]))
            }
            "requery()Z" => {
                let Data::Cursor {
                    position, closed, ..
                } = &mut self.heap.get_mut(receiver)?.data
                else {
                    bail!("uninitialized Cursor");
                };
                ensure_cursor_open(*closed)?;
                *position = -1;
                Ok(Some(vec![Word::from(1)]))
            }
            "registerContentObserver(Landroid/database/ContentObserver;)V"
            | "registerDataSetObserver(Landroid/database/DataSetObserver;)V"
            | "unregisterContentObserver(Landroid/database/ContentObserver;)V"
            | "unregisterDataSetObserver(Landroid/database/DataSetObserver;)V" => {
                let Data::Cursor { closed, .. } = &self.heap.get(receiver)?.data else {
                    bail!("uninitialized Cursor");
                };
                ensure_cursor_open(*closed)?;
                Ok(Some(vec![]))
            }
            _ => Ok(None),
        }
    }

    fn cursor_value(&self, cursor: Word, index: Option<Word>) -> Result<SqlValue> {
        let Data::Cursor {
            columns,
            rows,
            position,
            closed,
        } = &self.heap.get(cursor)?.data
        else {
            bail!("uninitialized Cursor");
        };
        ensure_cursor_open(*closed)?;
        let column = index.context("Cursor column index missing")?.int()?;
        let row = usize::try_from(*position)
            .ok()
            .and_then(|position| rows.get(position))
            .context("Cursor is not positioned on a row")?;
        row.get(usize::try_from(column).unwrap_or(usize::MAX))
            .cloned()
            .with_context(|| {
                format!(
                    "Cursor column {column} is outside {} columns",
                    columns.len()
                )
            })
    }

    fn byte_array(&mut self, bytes: &[u8]) -> Result<Word> {
        ensure!(bytes.len() <= 64 * 1_048_576, "SQLite blob exceeds 64 MiB");
        let array = self.array("B".into(), bytes.len())?;
        let Data::Array { values, .. } = &mut self.heap.get_mut(array)?.data else {
            unreachable!();
        };
        for (target, byte) in values.iter_mut().zip(bytes) {
            *target = vec![Word::from(i32::from(*byte as i8))];
        }
        Ok(array)
    }
}

fn sql_value(value: &SqlValue) -> Value {
    match value {
        SqlValue::Null => Value::Null,
        SqlValue::Integer(value) => Value::Integer(*value),
        SqlValue::Real(value) => Value::Real(*value),
        SqlValue::Text(value) => Value::Text(value.clone()),
        SqlValue::Blob(value) => Value::Blob(value.clone()),
    }
}

fn read_value(value: ValueRef<'_>) -> SqlValue {
    match value {
        ValueRef::Null => SqlValue::Null,
        ValueRef::Integer(value) => SqlValue::Integer(value),
        ValueRef::Real(value) => SqlValue::Real(value),
        ValueRef::Text(value) => SqlValue::Text(String::from_utf8_lossy(value).into_owned()),
        ValueRef::Blob(value) => SqlValue::Blob(value.to_vec()),
    }
}

fn sql_integer(value: &SqlValue) -> Result<i64> {
    match value {
        SqlValue::Null => Ok(0),
        SqlValue::Integer(value) => Ok(*value),
        SqlValue::Real(value) => Ok(*value as i64),
        SqlValue::Text(value) => Ok(value.parse()?),
        SqlValue::Blob(_) => Err(fault(
            "Ljava/lang/IllegalArgumentException;",
            "cannot convert SQLite blob to integer",
        )),
    }
}

fn ensure_cursor_open(closed: bool) -> Result<()> {
    if closed {
        Err(fault(
            "Ljava/lang/IllegalStateException;",
            "Cursor is closed",
        ))
    } else {
        Ok(())
    }
}

fn sql_fault(error: rusqlite::Error) -> anyhow::Error {
    fault(
        "Landroid/database/sqlite/SQLiteException;",
        error.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use droidless_formats::{apk::Apk, dex::Method};

    fn call(
        vm: &mut Runtime,
        class: &str,
        name: &str,
        parameters: &[&str],
        returns: &str,
        args: &[Word],
    ) -> Result<Vec<Word>> {
        vm.invoke(
            Method {
                class: class.into(),
                name: name.into(),
                parameters: parameters
                    .iter()
                    .map(|parameter| (*parameter).into())
                    .collect(),
                returns: returns.into(),
            },
            args.to_vec(),
            false,
        )
    }

    #[test]
    fn raw_query_cursor_reads_typed_rows_and_closes() {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap(),
        )
        .unwrap();
        vm.databases.insert(
            "memory".into(),
            Arc::new(Mutex::new(Connection::open_in_memory().unwrap())),
        );
        let database = vm.heap.instance(DATABASE).unwrap();
        vm.heap.get_mut(database).unwrap().data = Data::SqlDatabase("memory".into());
        let sql = vm
            .heap
            .string("SELECT 7 AS id, 'hello' AS label, NULL AS absent".into())
            .unwrap();
        let cursor = call(
            &mut vm,
            DATABASE,
            "rawQuery",
            &["Ljava/lang/String;", "[Ljava/lang/String;"],
            "Landroid/database/Cursor;",
            &[database, sql, Word::ZERO],
        )
        .unwrap()[0];

        assert_eq!(
            call(
                &mut vm,
                "Landroid/database/Cursor;",
                "getCount",
                &[],
                "I",
                &[cursor]
            )
            .unwrap()[0]
                .int()
                .unwrap(),
            1
        );
        let column = vm.heap.string("label".into()).unwrap();
        assert_eq!(
            call(
                &mut vm,
                "Landroid/database/Cursor;",
                "getColumnIndex",
                &["Ljava/lang/String;"],
                "I",
                &[cursor, column]
            )
            .unwrap()[0]
                .int()
                .unwrap(),
            1
        );
        assert_eq!(
            call(
                &mut vm,
                "Landroid/database/Cursor;",
                "moveToFirst",
                &[],
                "Z",
                &[cursor]
            )
            .unwrap()[0]
                .int()
                .unwrap(),
            1
        );
        assert_eq!(
            call(
                &mut vm,
                "Landroid/database/Cursor;",
                "getInt",
                &["I"],
                "I",
                &[cursor, Word::from(0)]
            )
            .unwrap()[0]
                .int()
                .unwrap(),
            7
        );
        let value = call(
            &mut vm,
            "Landroid/database/Cursor;",
            "getString",
            &["I"],
            "Ljava/lang/String;",
            &[cursor, Word::from(1)],
        )
        .unwrap()[0];
        assert_eq!(vm.heap.text(value).unwrap(), "hello");
        assert_eq!(
            call(
                &mut vm,
                "Landroid/database/Cursor;",
                "isNull",
                &["I"],
                "Z",
                &[cursor, Word::from(2)]
            )
            .unwrap()[0]
                .int()
                .unwrap(),
            1
        );
        assert_eq!(
            call(
                &mut vm,
                "Landroid/database/Cursor;",
                "moveToNext",
                &[],
                "Z",
                &[cursor]
            )
            .unwrap()[0]
                .int()
                .unwrap(),
            0
        );
        call(
            &mut vm,
            "Landroid/database/Cursor;",
            "close",
            &[],
            "V",
            &[cursor],
        )
        .unwrap();
        assert!(
            call(
                &mut vm,
                "Landroid/database/Cursor;",
                "getCount",
                &[],
                "I",
                &[cursor]
            )
            .is_err()
        );
    }

    #[test]
    fn nested_transactions_commit_through_savepoints() {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap(),
        )
        .unwrap();
        let connection = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
        connection
            .lock()
            .unwrap()
            .execute_batch("CREATE TABLE values_table (value INTEGER)")
            .unwrap();
        vm.databases.insert("memory".into(), connection);
        let database = vm.heap.instance(DATABASE).unwrap();
        vm.heap.get_mut(database).unwrap().data = Data::SqlDatabase("memory".into());
        let begin = || Method {
            class: DATABASE.into(),
            name: "beginTransaction".into(),
            parameters: vec![],
            returns: "V".into(),
        };
        let exec = |vm: &mut Runtime, sql: &str| {
            let sql = vm.heap.string(sql.into()).unwrap();
            call(
                vm,
                DATABASE,
                "execSQL",
                &["Ljava/lang/String;"],
                "V",
                &[database, sql],
            )
            .unwrap();
        };
        vm.invoke(begin(), vec![database], false).unwrap();
        exec(&mut vm, "INSERT INTO values_table VALUES (1)");
        vm.invoke(begin(), vec![database], false).unwrap();
        exec(&mut vm, "INSERT INTO values_table VALUES (2)");
        call(
            &mut vm,
            DATABASE,
            "setTransactionSuccessful",
            &[],
            "V",
            &[database],
        )
        .unwrap();
        call(&mut vm, DATABASE, "endTransaction", &[], "V", &[database]).unwrap();
        call(
            &mut vm,
            DATABASE,
            "setTransactionSuccessful",
            &[],
            "V",
            &[database],
        )
        .unwrap();
        call(&mut vm, DATABASE, "endTransaction", &[], "V", &[database]).unwrap();

        let sql = vm
            .heap
            .string("SELECT count(*) FROM values_table".into())
            .unwrap();
        let cursor = call(
            &mut vm,
            DATABASE,
            "rawQuery",
            &["Ljava/lang/String;", "[Ljava/lang/String;"],
            "Landroid/database/Cursor;",
            &[database, sql, Word::ZERO],
        )
        .unwrap()[0];
        call(
            &mut vm,
            "Landroid/database/Cursor;",
            "moveToFirst",
            &[],
            "Z",
            &[cursor],
        )
        .unwrap();
        assert_eq!(
            call(
                &mut vm,
                "Landroid/database/Cursor;",
                "getInt",
                &["I"],
                "I",
                &[cursor, Word::ZERO],
            )
            .unwrap()[0]
                .int()
                .unwrap(),
            2
        );
        assert!(!vm.database_transactions.contains_key("memory"));
    }
}
