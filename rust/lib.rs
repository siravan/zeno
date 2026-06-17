#![allow(uncommon_codepoints)]

use std::collections::HashSet;
use std::ffi::{c_char, CStr, CString};
use std::fmt::Debug;
use std::str::FromStr;

mod analyzer;
mod compiler;
mod composer;
mod config;
mod instruction;
// mod model;
mod parser;
// mod runnable;
mod application;
mod types;

// use model::{CellModel, Program};

use application::Application;
use compiler::Translator;
use config::{CompilerType, Config};

#[derive(Debug, Clone, Copy)]
pub enum CompilerStatus {
    Ok,
    Incomplete,
    InvalidUtf8,
    ParseError,
    InvalidCompiler,
    CompilationError,
}

pub struct CompilerResult {
    app: Option<Application>,
    status: CompilerStatus,
    msg: CString,
}

fn error_message<E: Debug>(msg: &str, err: E) -> CString {
    let s = format!("{:?}: {:?}", msg, err);
    CString::from_str(&s).unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn translate(
    json: *const c_char,
    ty: *const c_char,
    opt: u32,
    num_params: usize,
) -> *const CompilerResult {
    let mut res = CompilerResult {
        app: None,
        status: CompilerStatus::Incomplete,
        msg: CString::from_str("Success").unwrap(),
    };

    let json = unsafe {
        match CStr::from_ptr(json).to_str() {
            Ok(json) => json,
            Err(msg) => {
                res.status = CompilerStatus::InvalidUtf8;
                res.msg = error_message("Invalid encoding", msg);
                return Box::into_raw(Box::new(res)) as *const _;
            }
        }
    };

    let ty = unsafe {
        match CStr::from_ptr(ty).to_str() {
            Ok(ty) => ty,
            Err(msg) => {
                res.status = CompilerStatus::InvalidUtf8;
                res.msg = error_message("Invalid compiler type", msg);
                return Box::into_raw(Box::new(res)) as *const _;
            }
        }
    };

    if let Ok(mut config) = Config::from_name(ty, opt) {
        let mut comp = Translator::new(config);
        let app = comp.translate(json.to_string(), num_params);

        match app {
            Ok(app) => {
                res.app = Some(app);
                res.status = CompilerStatus::Ok;
            }
            Err(msg) => {
                res.status = CompilerStatus::InvalidCompiler;
                res.msg = error_message("Compilation error", msg);
            }
        }
    } else {
        res.status = CompilerStatus::InvalidCompiler;
        res.msg = error_message("Config error", opt);
    }

    Box::into_raw(Box::new(res)) as *const _
}

/// Checks the status of a `CompilerResult`.
///
/// Returns a null-terminated string representing the status message.
///
/// # Safety
///     it is the responsibility of the calling function to ensure
///     that q points to a valid CompilerResult.
///
#[no_mangle]
pub unsafe extern "C" fn check_status(q: *const CompilerResult) -> *const c_char {
    let q: &CompilerResult = unsafe { &*q };
    q.msg.as_ptr() as *const _
}

/// Deallocates the CompilerResult pointed by `q`.
///
/// # Safety
///     it is the responsibility of the calling function to ensure
///     that q points to a valid CompilerResult and that after
///     calling this function, q is invalid and should not
///     be used anymore.
///
#[no_mangle]
pub unsafe extern "C" fn finalize(q: *mut CompilerResult) {
    if !q.is_null() {
        let _ = unsafe { Box::from_raw(q) };
    }
}

/// Returns a null-terminated string representing the version.
///
/// Used for debugging.
///
/// # Safety
///     the return value is a null-terminated string that should not
///     be freed.
///
#[no_mangle]
pub unsafe extern "C" fn info() -> *const c_char {
    // let msg = c"symjit 1.3.3";
    let msg = CString::new(env!("CARGO_PKG_VERSION")).unwrap();
    msg.into_raw() as *const _
}
