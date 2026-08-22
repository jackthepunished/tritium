//! C ABI.
//!
//! Opaque handles only; no Rust type crosses the boundary. Every entry point is
//! wrapped in `catch_unwind` -- a panic unwinding into C is undefined behavior,
//! and the kernels use `assert!` liberally on their preconditions.
//!
//! Errors return a negative code and stash a message in a thread-local that
//! `trit_last_error` returns, so a caller gets something actionable without the
//! ABI having to carry a string type.

use std::cell::RefCell;
use std::ffi::{c_char, c_int, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::Arc;

use trit_core::model::Model;
use trit_core::sampler::SamplerParams;
use trit_core::session::Session;
use trit_core::tokenizer::{Detokenizer, Tokenizer};

use crate::{HfTokenizer, LoadOptions, Runtime};

pub const TRIT_OK: c_int = 0;
pub const TRIT_ERR: c_int = -1;
pub const TRIT_ERR_NULL: c_int = -2;
pub const TRIT_ERR_UTF8: c_int = -3;
pub const TRIT_ERR_PANIC: c_int = -4;
pub const TRIT_ERR_BUFFER: c_int = -5;

thread_local! {
    static LAST_ERROR: RefCell<CString> = RefCell::new(CString::new("").unwrap());
}

fn set_error(msg: impl std::fmt::Display) {
    let text = msg.to_string().replace('\0', " ");
    LAST_ERROR.with(|e| *e.borrow_mut() = CString::new(text).unwrap_or_default());
}

/// Run `f`, converting a panic or an error into a code and a stored message.
fn guard<F: FnOnce() -> anyhow::Result<c_int>>(f: F) -> c_int {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => {
            set_error(format!("{e:#}"));
            TRIT_ERR
        }
        Err(p) => {
            let what = p
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| p.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "panic".into());
            set_error(format!("panic: {what}"));
            TRIT_ERR_PANIC
        }
    }
}

/// Same, for functions returning a pointer.
fn guard_ptr<T, F: FnOnce() -> anyhow::Result<*mut T>>(f: F) -> *mut T {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(p)) => p,
        Ok(Err(e)) => {
            set_error(format!("{e:#}"));
            std::ptr::null_mut()
        }
        Err(_) => {
            set_error("panic");
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `p` must be null or a valid NUL-terminated C string.
unsafe fn cstr(p: *const c_char) -> anyhow::Result<Option<String>> {
    if p.is_null() {
        return Ok(None);
    }
    Ok(Some(CStr::from_ptr(p).to_str()?.to_string()))
}

pub struct TritModel {
    runtime: Runtime,
}

pub struct TritSession {
    model: Arc<Model>,
    session: Session,
    detok: Detokenizer,
    tokenizer: *const HfTokenizer,
    finished: bool,
}

/// Sampling knobs, laid out to match `include/tritium.h`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TritSamplerParams {
    pub temperature: f32,
    pub top_p: f32,
    pub repetition_penalty: f32,
    pub top_k: u32,
    pub seed: u64,
}

impl From<TritSamplerParams> for SamplerParams {
    fn from(p: TritSamplerParams) -> Self {
        SamplerParams {
            temperature: p.temperature,
            top_p: if p.top_p <= 0.0 { 1.0 } else { p.top_p },
            top_k: p.top_k as usize,
            repetition_penalty: if p.repetition_penalty <= 0.0 {
                1.0
            } else {
                p.repetition_penalty
            },
            seed: p.seed,
        }
    }
}

/// Runtime version string. Valid for the process lifetime.
#[no_mangle]
pub extern "C" fn trit_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

/// Bit 0: a CPU backend is compiled in. Bit 1: the RTL backend is.
#[no_mangle]
pub extern "C" fn trit_backend_mask() -> u32 {
    let mut m = 0u32;
    for b in crate::backend::compiled_backends() {
        match *b {
            "cpu" => m |= 1,
            "rtl" => m |= 2,
            _ => {}
        }
    }
    m
}

/// The active CPU kernel, e.g. `"avx512vnni"`. Valid for the process lifetime.
#[no_mangle]
pub extern "C" fn trit_kernel_name() -> *const c_char {
    // The dispatcher's names are compile-time literals, but not NUL-terminated;
    // intern one CString per process so the pointer stays valid.
    use std::sync::OnceLock;
    static NAME: OnceLock<CString> = OnceLock::new();
    NAME.get_or_init(|| CString::new(trit_cpu::kernel_name()).unwrap())
        .as_ptr()
}

/// The last error on this thread. Valid until the next call on this thread.
#[no_mangle]
pub extern "C" fn trit_last_error() -> *const c_char {
    LAST_ERROR.with(|e| e.borrow().as_ptr())
}

/// Load a model. Returns NULL on failure; see `trit_last_error`.
///
/// `tokenizer_path` may be NULL to use `tokenizer.json` beside the model.
/// `backend` may be NULL for `"cpu"`.
///
/// # Safety
/// All pointers must be NULL or valid NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn trit_model_load(
    model_path: *const c_char,
    tokenizer_path: *const c_char,
    backend: *const c_char,
    threads: u32,
) -> *mut TritModel {
    guard_ptr(|| {
        let model = cstr(model_path)?.ok_or_else(|| anyhow::anyhow!("model_path is NULL"))?;
        let opts = LoadOptions {
            model: PathBuf::from(model),
            tokenizer: cstr(tokenizer_path)?.map(PathBuf::from),
            backend: cstr(backend)?.unwrap_or_else(|| "cpu".into()),
            threads: threads as usize,
            kernel: None,
            verify: false,
        };
        Ok(Box::into_raw(Box::new(TritModel {
            runtime: Runtime::load(&opts)?,
        })))
    })
}

/// # Safety
/// `m` must come from `trit_model_load` and must not be used afterwards. All
/// sessions created from it must be freed first.
#[no_mangle]
pub unsafe extern "C" fn trit_model_free(m: *mut TritModel) {
    if !m.is_null() {
        drop(Box::from_raw(m));
    }
}

/// # Safety
/// `m` must be a live handle from `trit_model_load`.
#[no_mangle]
pub unsafe extern "C" fn trit_model_vocab_size(m: *const TritModel) -> u32 {
    if m.is_null() {
        return 0;
    }
    (*m).runtime.model.config().vocab_size as u32
}

/// Weight bytes touched per decoded token -- the numerator of the bandwidth
/// roofline.
///
/// # Safety
/// `m` must be a live handle from `trit_model_load`.
#[no_mangle]
pub unsafe extern "C" fn trit_model_weight_bytes(m: *const TritModel) -> u64 {
    if m.is_null() {
        return 0;
    }
    (*m).runtime.model.weight_bytes()
}

/// # Safety
/// `m` must be a live handle; `params` must be NULL or a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn trit_session_new(
    m: *mut TritModel,
    params: *const TritSamplerParams,
) -> *mut TritSession {
    guard_ptr(|| {
        anyhow::ensure!(!m.is_null(), "model handle is NULL");
        let rt = &(*m).runtime;
        let p: SamplerParams = if params.is_null() {
            SamplerParams::default()
        } else {
            (*params).into()
        };
        Ok(Box::into_raw(Box::new(TritSession {
            session: Session::new(rt.model.clone(), &p),
            model: rt.model.clone(),
            detok: Detokenizer::new(),
            // Borrowed from the model handle, which the caller must outlive the
            // session; documented in tritium.h.
            tokenizer: &rt.tokenizer as *const HfTokenizer,
            finished: false,
        })))
    })
}

/// # Safety
/// `s` must come from `trit_session_new` and must not be used afterwards.
#[no_mangle]
pub unsafe extern "C" fn trit_session_free(s: *mut TritSession) {
    if !s.is_null() {
        drop(Box::from_raw(s));
    }
}

/// # Safety
/// `s` must be a live session handle.
#[no_mangle]
pub unsafe extern "C" fn trit_session_reset(s: *mut TritSession) -> c_int {
    guard(|| {
        anyhow::ensure!(!s.is_null(), "session handle is NULL");
        (*s).session.reset();
        (*s).detok = Detokenizer::new();
        (*s).finished = false;
        Ok(TRIT_OK)
    })
}

/// Tokenize and run the prompt. `len` is the byte length; pass 0 to treat
/// `utf8` as NUL-terminated.
///
/// # Safety
/// `s` must be a live session handle; `utf8` must point to `len` readable bytes
/// (or be NUL-terminated when `len` is 0).
#[no_mangle]
pub unsafe extern "C" fn trit_session_prefill(
    s: *mut TritSession,
    utf8: *const c_char,
    len: usize,
) -> c_int {
    guard(|| {
        anyhow::ensure!(!s.is_null(), "session handle is NULL");
        anyhow::ensure!(!utf8.is_null(), "prompt is NULL");
        let bytes = if len == 0 {
            CStr::from_ptr(utf8).to_bytes()
        } else {
            std::slice::from_raw_parts(utf8 as *const u8, len)
        };
        let text = std::str::from_utf8(bytes)?;
        let sess = &mut *s;
        let tk = &*sess.tokenizer;
        let ids = tk.encode(text, true)?;
        sess.session.prefill(&ids)?;
        Ok(TRIT_OK)
    })
}

/// Produce the next token.
///
/// Returns 1 when a token was produced, 0 at end of stream (EOS or context
/// full), and a negative code on error. `out_id` receives the token id;
/// `out_buf`/`buf_len` receive its UTF-8 text, NUL-terminated, with the byte
/// length (excluding the NUL) written to `out_len`.
///
/// The text may legitimately be empty while the return value is 1: byte-level
/// BPE splits multi-byte codepoints across tokens, and the runtime holds an
/// incomplete tail until it completes.
///
/// # Safety
/// `s` must be a live session handle. `out_id` may be NULL. `out_buf` must point
/// to `buf_len` writable bytes; `out_len` may be NULL.
#[no_mangle]
pub unsafe extern "C" fn trit_session_next(
    s: *mut TritSession,
    out_id: *mut u32,
    out_buf: *mut c_char,
    buf_len: usize,
    out_len: *mut usize,
) -> c_int {
    guard(|| {
        anyhow::ensure!(!s.is_null(), "session handle is NULL");
        anyhow::ensure!(
            !out_buf.is_null() && buf_len > 0,
            "output buffer is NULL or empty"
        );
        let sess = &mut *s;
        if sess.finished {
            return Ok(0);
        }
        let tk = &*sess.tokenizer;

        let id = sess.session.sample();
        if tk.is_eos(id) || sess.session.remaining() == 0 {
            sess.finished = true;
            return Ok(0);
        }
        sess.session.advance(id)?;
        let text = sess.detok.push(tk, id)?;

        let bytes = text.as_bytes();
        if bytes.len() + 1 > buf_len {
            set_error(format!(
                "buffer of {buf_len} bytes is too small for {} + NUL",
                bytes.len()
            ));
            return Ok(TRIT_ERR_BUFFER);
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_buf as *mut u8, bytes.len());
        *out_buf.add(bytes.len()) = 0;
        if !out_id.is_null() {
            *out_id = id;
        }
        if !out_len.is_null() {
            *out_len = bytes.len();
        }
        let _ = &sess.model; // keeps the model alive for the session's lifetime
        Ok(1)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_and_backend_mask_are_sane() {
        let v = unsafe { CStr::from_ptr(trit_version()) }.to_str().unwrap();
        assert_eq!(v, env!("CARGO_PKG_VERSION"));
        assert_eq!(
            trit_backend_mask() & 1,
            1,
            "a CPU backend is always compiled in"
        );
    }

    #[test]
    fn null_handles_do_not_crash() {
        unsafe {
            trit_model_free(std::ptr::null_mut());
            trit_session_free(std::ptr::null_mut());
            assert_eq!(trit_model_vocab_size(std::ptr::null()), 0);
            assert_eq!(trit_model_weight_bytes(std::ptr::null()), 0);
            assert_eq!(trit_session_reset(std::ptr::null_mut()), TRIT_ERR);
            let m = trit_model_load(std::ptr::null(), std::ptr::null(), std::ptr::null(), 1);
            assert!(m.is_null());
            let msg = CStr::from_ptr(trit_last_error()).to_str().unwrap();
            assert!(msg.contains("NULL"), "{msg}");
        }
    }

    #[test]
    fn a_panic_is_converted_not_unwound_into_c() {
        // catch_unwind must turn this into a code; letting it cross the FFI
        // boundary would be undefined behavior.
        let rc = guard(|| panic!("boom"));
        assert_eq!(rc, TRIT_ERR_PANIC);
        let msg = LAST_ERROR.with(|e| e.borrow().to_str().unwrap().to_string());
        assert!(msg.contains("boom"), "{msg}");
    }

    #[test]
    fn error_messages_survive_an_embedded_nul() {
        set_error("bad\0value");
        let msg = LAST_ERROR.with(|e| e.borrow().to_str().unwrap().to_string());
        assert_eq!(msg, "bad value");
    }
}
