//! A small, safe(-ish) wrapper around the raw `sqlite-wasm-rs` C API.
//!
//! This intentionally only implements the handful of operations the app
//! needs (batch exec, parameterized query, parameterized execute) rather
//! than a full driver.

use sqlite_wasm_rs as ffi;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

/// SQLite's `SQLITE_TRANSIENT` destructor sentinel: passing this instead of a
/// real function pointer tells SQLite to make its own private copy of a
/// bound string/blob immediately, rather than assuming the pointer stays
/// valid. It is conventionally the value `-1` cast to the destructor
/// function-pointer type; `sqlite-wasm-rs` doesn't re-export it as a
/// constant, so it's computed here once (as a function, since transmuting a
/// non-null-but-non-function bit pattern can't be const-evaluated) and
/// reused at every bind call site.
fn sqlite_transient() -> ffi::sqlite3_destructor_type {
    Some(unsafe { std::mem::transmute::<isize, unsafe extern "C" fn(*mut c_void)>(-1) })
}

/// A single bound parameter value.
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Int(i64),
    Real(f64),
    Text(String),
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Text(s.to_string())
    }
}
impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Text(s)
    }
}
impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int(v)
    }
}
impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Real(v)
    }
}
impl From<Option<i64>> for Value {
    fn from(v: Option<i64>) -> Self {
        v.map(Value::Int).unwrap_or(Value::Null)
    }
}

/// One returned row: a vector of column values, in column order.
pub type Row = Vec<Value>;

impl Value {
    pub fn as_i64(&self) -> i64 {
        match self {
            Value::Int(v) => *v,
            Value::Real(v) => *v as i64,
            _ => 0,
        }
    }
    pub fn as_f64(&self) -> f64 {
        match self {
            Value::Real(v) => *v,
            Value::Int(v) => *v as f64,
            _ => 0.0,
        }
    }
    pub fn as_str(&self) -> String {
        match self {
            Value::Text(v) => v.clone(),
            _ => String::new(),
        }
    }
    pub fn as_opt_i64(&self) -> Option<i64> {
        match self {
            Value::Null => None,
            v => Some(v.as_i64()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbError(pub String);

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for DbError {}

/// A single, non-thread-safe SQLite connection (matches how sqlite-wasm-rs is
/// compiled: `SQLITE_THREADSAFE=0`). Safe to use from a CSR (single-threaded)
/// WASM app.
pub struct Conn {
    handle: *mut ffi::sqlite3,
}

// SAFETY: the WASM target this app compiles for is single-threaded; nothing
// here is ever sent across a real OS thread.
#[allow(unsafe_code)]
unsafe impl Send for Conn {}

impl Conn {
    /// Open an in-memory database.
    pub fn open_memory() -> Result<Self, DbError> {
        let mut handle: *mut ffi::sqlite3 = std::ptr::null_mut();
        let filename = CString::new(":memory:").unwrap();
        let rc = unsafe {
            ffi::sqlite3_open_v2(
                filename.as_ptr(),
                &mut handle as *mut _,
                ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE,
                std::ptr::null(),
            )
        };
        if rc != ffi::SQLITE_OK {
            return Err(DbError(format!("sqlite3_open_v2 failed: rc={rc}")));
        }
        Ok(Conn { handle })
    }

    fn last_error(&self) -> String {
        unsafe {
            let msg = ffi::sqlite3_errmsg(self.handle);
            if msg.is_null() {
                "unknown sqlite error".to_string()
            } else {
                CStr::from_ptr(msg).to_string_lossy().into_owned()
            }
        }
    }

    /// Execute one or more semicolon-separated statements with no parameters
    /// and no result rows (used for migrations and seed data).
    pub fn execute_batch(&self, sql: &str) -> Result<(), DbError> {
        let csql = CString::new(sql).map_err(|e| DbError(e.to_string()))?;
        let mut errmsg: *mut c_char = std::ptr::null_mut();
        let rc = unsafe {
            ffi::sqlite3_exec(
                self.handle,
                csql.as_ptr(),
                None,
                std::ptr::null_mut(),
                &mut errmsg as *mut _,
            )
        };
        if rc != ffi::SQLITE_OK {
            let msg = if errmsg.is_null() {
                self.last_error()
            } else {
                unsafe {
                    let m = CStr::from_ptr(errmsg).to_string_lossy().into_owned();
                    ffi::sqlite3_free(errmsg as *mut c_void);
                    m
                }
            };
            return Err(DbError(format!("execute_batch failed: {msg}")));
        }
        Ok(())
    }

    /// Run a parameterized statement that doesn't return rows (INSERT/UPDATE/DELETE).
    /// Returns the number of rows affected by the change.
    pub fn execute(&self, sql: &str, params: &[Value]) -> Result<i64, DbError> {
        let stmt = self.prepare(sql, params)?;
        let rc = unsafe { ffi::sqlite3_step(stmt) };
        let affected = unsafe { ffi::sqlite3_changes(self.handle) as i64 };
        unsafe { ffi::sqlite3_finalize(stmt) };
        if rc != ffi::SQLITE_DONE && rc != ffi::SQLITE_ROW {
            return Err(DbError(format!(
                "execute failed: rc={rc} ({})",
                self.last_error()
            )));
        }
        Ok(affected)
    }

    /// Run a parameterized INSERT and return `last_insert_rowid`.
    pub fn insert(&self, sql: &str, params: &[Value]) -> Result<i64, DbError> {
        self.execute(sql, params)?;
        Ok(unsafe { ffi::sqlite3_last_insert_rowid(self.handle) })
    }

    /// Run a parameterized query and collect all result rows.
    pub fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>, DbError> {
        let stmt = self.prepare(sql, params)?;
        let ncols = unsafe { ffi::sqlite3_column_count(stmt) };
        let mut rows = Vec::new();
        loop {
            let rc = unsafe { ffi::sqlite3_step(stmt) };
            if rc == ffi::SQLITE_ROW {
                let mut row = Vec::with_capacity(ncols as usize);
                for i in 0..ncols {
                    row.push(column_value(stmt, i));
                }
                rows.push(row);
            } else if rc == ffi::SQLITE_DONE {
                break;
            } else {
                unsafe { ffi::sqlite3_finalize(stmt) };
                return Err(DbError(format!(
                    "query step failed: rc={rc} ({})",
                    self.last_error()
                )));
            }
        }
        unsafe { ffi::sqlite3_finalize(stmt) };
        Ok(rows)
    }

    fn prepare(&self, sql: &str, params: &[Value]) -> Result<*mut ffi::sqlite3_stmt, DbError> {
        let csql = CString::new(sql).map_err(|e| DbError(e.to_string()))?;
        let mut stmt: *mut ffi::sqlite3_stmt = std::ptr::null_mut();
        let rc = unsafe {
            ffi::sqlite3_prepare_v2(
                self.handle,
                csql.as_ptr(),
                -1,
                &mut stmt as *mut _,
                std::ptr::null_mut(),
            )
        };
        if rc != ffi::SQLITE_OK {
            return Err(DbError(format!(
                "prepare failed: rc={rc} ({})",
                self.last_error()
            )));
        }
        for (i, param) in params.iter().enumerate() {
            let idx = (i + 1) as i32;
            let rc = match param {
                Value::Null => unsafe { ffi::sqlite3_bind_null(stmt, idx) },
                Value::Int(v) => unsafe { ffi::sqlite3_bind_int64(stmt, idx, *v) },
                Value::Real(v) => unsafe { ffi::sqlite3_bind_double(stmt, idx, *v) },
                Value::Text(s) => {
                    let cs = CString::new(s.as_str()).map_err(|e| DbError(e.to_string()))?;
                    // SQLITE_TRANSIENT: tell sqlite to copy the string, since
                    // `cs` is freed at the end of this loop iteration.
                    unsafe {
                        ffi::sqlite3_bind_text(stmt, idx, cs.as_ptr(), -1, sqlite_transient())
                    }
                }
            };
            if rc != ffi::SQLITE_OK {
                unsafe { ffi::sqlite3_finalize(stmt) };
                return Err(DbError(format!("bind failed: rc={rc}")));
            }
        }
        Ok(stmt)
    }
}

fn column_value(stmt: *mut ffi::sqlite3_stmt, i: i32) -> Value {
    let col_type = unsafe { ffi::sqlite3_column_type(stmt, i) };
    match col_type {
        ffi::SQLITE_NULL => Value::Null,
        ffi::SQLITE_INTEGER => Value::Int(unsafe { ffi::sqlite3_column_int64(stmt, i) }),
        ffi::SQLITE_FLOAT => Value::Real(unsafe { ffi::sqlite3_column_double(stmt, i) }),
        _ => {
            let ptr = unsafe { ffi::sqlite3_column_text(stmt, i) };
            if ptr.is_null() {
                Value::Text(String::new())
            } else {
                let s = unsafe { CStr::from_ptr(ptr as *const c_char) };
                Value::Text(s.to_string_lossy().into_owned())
            }
        }
    }
}

impl Drop for Conn {
    fn drop(&mut self) {
        unsafe {
            ffi::sqlite3_close(self.handle);
        }
    }
}
