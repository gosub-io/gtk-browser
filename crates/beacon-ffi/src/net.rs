//! The network panel, over the C ABI.
//!
//! Same shape as the log and timing snapshots next door: take a copy, then read it by
//! index. A panel refreshing several times a second must not be walking a live table, and
//! a request that vanished between two accessor calls would be worse than a stale row.
//!
//! The rows themselves are `beacon_core::devtools`'s, folded together from the engine's
//! resource events by [`beacon_core::beacon::Beacon::on_engine_event`] — so every shell
//! gets the same panel over the same data rather than each learning to read the engine.

use std::ffi::c_char;

use beacon_core::devtools;

use crate::{to_c_string, BeaconBrowser};

/// A field the engine never reported, as distinct from one that really is zero: a request
/// served from a pooled connection resolved nothing, which is not the same as resolving in
/// no time at all.
pub const BEACON_ABSENT: u64 = u64::MAX;

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum BeaconRequestState {
    Queued = 0,
    Running = 1,
    Finished = 2,
    Failed = 3,
    Cancelled = 4,
}

/// How far an unfinished request got. "Loading" for thirty seconds says nothing; *where*
/// it has been loading for thirty seconds says everything.
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum BeaconRequestPhase {
    Queued = 0,
    Opening = 1,
    Connecting = 2,
    Waiting = 3,
    Receiving = 4,
    Done = 5,
}

/// What the HTTP cache did with a hop. 0 is "nothing reported", so a zeroed value and an
/// out-of-range read both say that.
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum BeaconCacheOutcome {
    None = 0,
    /// A stored response was used without contacting the server.
    Hit = 1,
    /// A stored response was revalidated: the server answered `304`.
    Validated = 2,
    /// The network's response was written to the cache.
    Stored = 3,
    /// Stored responses for the URL were dropped.
    Invalidated = 4,
    /// An outcome this build does not know.
    Other = 5,
}

fn cache_outcome(outcome: Option<gosub_engine::events::CacheOutcome>) -> BeaconCacheOutcome {
    use gosub_engine::events::CacheOutcome;
    match outcome {
        None => BeaconCacheOutcome::None,
        Some(CacheOutcome::Hit) => BeaconCacheOutcome::Hit,
        Some(CacheOutcome::Validated) => BeaconCacheOutcome::Validated,
        Some(CacheOutcome::Stored) => BeaconCacheOutcome::Stored,
        Some(CacheOutcome::Invalidated) => BeaconCacheOutcome::Invalidated,
        Some(_) => BeaconCacheOutcome::Other,
    }
}

/// Everything about a request that is a number. The strings are read with the accessors
/// below, because a struct of owned pointers would need a matching free function and this
/// way the caller frees each string exactly as it frees every other string here.
#[repr(C)]
pub struct BeaconRequestRow {
    /// When it was first seen, milliseconds since the epoch. Rows are in this order, which
    /// is the order the page actually fetched things.
    pub started_ms: u64,
    pub received_bytes: u64,
    /// What the server said it would send, which is not always what arrived.
    pub content_length: u64,
    /// Total time, once it finished. `BEACON_ABSENT` while it is still in flight.
    pub elapsed_us: u64,
    /// Name resolution, connection setup, and the moment the response headers landed.
    /// `connect_us` encloses `dns_us` rather than following it: resolution happens inside
    /// the connector being timed.
    pub dns_us: u64,
    pub connect_us: u64,
    pub headers_ms: u64,
    /// 0 until a response line arrives.
    pub status: u32,
    pub state: BeaconRequestState,
    pub phase: BeaconRequestPhase,
    pub request_header_count: usize,
    pub response_header_count: usize,
    pub redirect_count: usize,
    pub has_body: bool,
    /// The response continued past the captured preview.
    pub body_truncated: bool,
    /// A body was captured and then dropped to stay inside the total budget — which is a
    /// different thing to say than "none was captured", and a panel should say it.
    pub body_evicted: bool,
    /// How the cache answered the hop the row ends on: `Hit`, `Validated`, or `None` when
    /// the bytes came from the network. What a status column marks as "(cache)".
    pub cache_answer: BeaconCacheOutcome,
    /// How many entries each detail list holds, for the indexed getters below.
    pub cache_count: usize,
    pub auth_count: usize,
    pub upgrade_count: usize,
    pub change_count: usize,
    pub retry_count: usize,
    pub preflight_count: usize,
}

fn absent(value: Option<u64>) -> u64 {
    value.unwrap_or(BEACON_ABSENT)
}

/// Copy the requests for `tab` — or every tab when it is 0 — and return how many are
/// readable. Newest last, in the order the page fetched them.
///
/// # Safety
/// `browser` must be a live handle from [`crate::beacon_new`].
#[no_mangle]
pub unsafe extern "C" fn beacon_net_snapshot(browser: *mut BeaconBrowser, tab: u64) -> usize {
    let b = browser!(browser, 0);
    // 0 means "every tab" rather than "no tab": a window that has not settled on an active
    // tab yet should show the page's requests, not an empty panel.
    //
    // A tab handle that is not a tab is a different thing again, and must not fall back to
    // "everything": a panel asking about a tab that has gone would then quietly show every
    // other tab's requests as though they were its own.
    let filter = if tab == 0 {
        None
    } else {
        match b.tab(tab) {
            Some(tab_id) => Some(tab_id),
            None => {
                b.requests.clear();
                return 0;
            }
        }
    };
    b.requests = devtools::requests(filter);
    b.requests.len()
}

/// Forget every recorded request. Process-wide, like the log: there is one recorder.
///
/// # Safety
/// `browser` must be a live handle from [`crate::beacon_new`].
#[no_mangle]
pub unsafe extern "C" fn beacon_net_clear(browser: *mut BeaconBrowser) {
    let b = browser!(browser);
    devtools::clear_requests();
    b.requests.clear();
}

/// Copy response bodies as they arrive, so the panel has something to show.
///
/// Off by default and meant to follow the panel's visibility: bodies are only worth
/// keeping while someone can look at them, and this is what keeps every page that is *not*
/// being inspected free of the cost.
///
/// # Safety
/// `browser` must be a live handle from [`crate::beacon_new`].
#[no_mangle]
pub unsafe extern "C" fn beacon_net_set_capture_bodies(browser: *mut BeaconBrowser, enabled: bool) {
    let _ = browser!(browser);
    devtools::set_capture_bodies(enabled);
}

/// Show `cookie`, `authorization` and `proxy-authorization` in full rather than redacted.
///
/// Same window as body capture, and for the same reason: a request view that will not show
/// you your own `Cookie` cannot answer the question it was usually opened to answer, but a
/// panel nobody has opened has no business copying credentials into memory.
///
/// # Safety
/// `browser` must be a live handle from [`crate::beacon_new`].
#[no_mangle]
pub unsafe extern "C" fn beacon_net_set_show_sensitive_headers(browser: *mut BeaconBrowser, enabled: bool) {
    let _ = browser!(browser);
    devtools::set_show_sensitive_headers(enabled);
}

/// How many bytes of captured bodies are being held, across every request.
///
/// # Safety
/// `browser` must be a live handle from [`crate::beacon_new`].
#[no_mangle]
pub unsafe extern "C" fn beacon_net_captured_body_bytes(browser: *mut BeaconBrowser) -> usize {
    let _ = browser!(browser, 0);
    devtools::captured_body_bytes()
}

/// Read row `index` of the last snapshot. False when out of range, leaving `out` untouched.
///
/// # Safety
/// `browser` must be a live handle; `out` must point at a `BeaconRequestRow`.
#[no_mangle]
pub unsafe extern "C" fn beacon_net_at(browser: *mut BeaconBrowser, index: usize, out: *mut BeaconRequestRow) -> bool {
    let b = browser!(browser, false);
    if out.is_null() {
        return false;
    }
    let Some(request) = b.requests.get(index) else {
        return false;
    };

    let row = BeaconRequestRow {
        started_ms: request.started_ms,
        received_bytes: request.received_bytes,
        content_length: absent(request.content_length),
        elapsed_us: absent(request.elapsed_us),
        dns_us: absent(request.dns_us),
        connect_us: absent(request.connect_us),
        headers_ms: absent(request.headers_ms),
        status: request.status.unwrap_or(0) as u32,
        state: match request.state {
            devtools::RequestState::Queued => BeaconRequestState::Queued,
            devtools::RequestState::Running => BeaconRequestState::Running,
            devtools::RequestState::Finished => BeaconRequestState::Finished,
            devtools::RequestState::Failed => BeaconRequestState::Failed,
            devtools::RequestState::Cancelled => BeaconRequestState::Cancelled,
        },
        phase: match request.phase() {
            devtools::Phase::Queued => BeaconRequestPhase::Queued,
            devtools::Phase::Opening => BeaconRequestPhase::Opening,
            devtools::Phase::Connecting => BeaconRequestPhase::Connecting,
            devtools::Phase::Waiting => BeaconRequestPhase::Waiting,
            devtools::Phase::Receiving => BeaconRequestPhase::Receiving,
            devtools::Phase::Done => BeaconRequestPhase::Done,
        },
        request_header_count: request.request_headers.len(),
        response_header_count: request.headers.len(),
        redirect_count: request.redirects.len(),
        has_body: request.body.is_some(),
        body_truncated: request.body_truncated,
        body_evicted: request.body_evicted,
        cache_answer: cache_outcome(request.cache_answer()),
        cache_count: request.cache.len(),
        auth_count: request.auth.len(),
        upgrade_count: request.upgrades.len(),
        change_count: request.changes.len(),
        retry_count: request.retries.len(),
        preflight_count: request.preflights.len(),
    };
    unsafe { std::ptr::write(out, row) };
    true
}

/// One string off a row. Every one of these frees with [`crate::beacon_string_free`], and
/// returns NULL rather than an empty string for "nothing to say" — a request that never
/// reached the wire has no method, which is not the same as an empty one.
macro_rules! row_string {
    ($name:ident, $doc:literal, |$request:ident| $value:expr) => {
        #[doc = $doc]
        ///
        /// Free with [`crate::beacon_string_free`]; NULL when there is none.
        ///
        /// # Safety
        /// `browser` must be a live handle from [`crate::beacon_new`].
        #[no_mangle]
        pub unsafe extern "C" fn $name(browser: *mut BeaconBrowser, index: usize) -> *mut c_char {
            let b = browser!(browser, std::ptr::null_mut());
            let Some($request) = b.requests.get(index) else {
                return std::ptr::null_mut();
            };
            match $value {
                Some(value) => to_c_string(&value),
                None => std::ptr::null_mut(),
            }
        }
    };
}

row_string!(beacon_net_url, "The URL requested.", |r| Some(r.url.clone()));
row_string!(beacon_net_kind, "`document`, `stylesheet`, `script`, `image`, `font`, …", |r| {
    Some(r.kind.clone())
});
row_string!(
    beacon_net_initiator,
    "What caused it: `navigation`, `parser`, `script`, `css`.",
    |r| Some(r.initiator.clone())
);
row_string!(
    beacon_net_method,
    "The method actually sent. NULL for a request that never reached the network -- a cache hit (`cache_answer` says so), a file:// load, or one that failed before it was sent.",
    |r| r.method.clone()
);
row_string!(beacon_net_content_type, "The response's declared content type.", |r| r
    .content_type
    .clone());
row_string!(beacon_net_error, "What went wrong, in the network stack's words.", |r| r
    .error
    .clone());
row_string!(
    beacon_net_state_label,
    "A word for the row's state: `queued`, `loading`, `done`, `failed`, `cancelled`.",
    |r| Some(r.state.label().to_string())
);
row_string!(
    beacon_net_phase_label,
    "Where an unfinished request has got to: `opening`, `connecting`, `waiting`, `receiving`.",
    |r| Some(r.phase().label().to_string())
);
row_string!(
    beacon_net_phase_hint,
    "What a developer looking at a request stuck in this phase should suspect.",
    |r| Some(r.phase().hint().to_string())
);
row_string!(
    beacon_net_failure_label,
    "A short name for why it failed — `TLS`, `timeout`, `no connection` — when the stack classified it. NULL otherwise.",
    |r| r.failure_label().map(str::to_string)
);
row_string!(
    beacon_net_failure_hint,
    "What that failure means, in the words someone debugging a dead page needs.",
    |r| r.failure_hint().map(str::to_string)
);
row_string!(
    beacon_net_failure_fix,
    "What would make the request succeed, when the failure names a rule precise enough to say -- a CORS refusal or a classified TLS failure. NULL otherwise.",
    |r| r.failure_fix().map(str::to_string)
);
row_string!(
    beacon_net_failure_spec,
    "A link to the rule that refused the request, alongside `failure_fix`. NULL otherwise.",
    |r| r.failure_spec().map(str::to_string)
);
row_string!(
    beacon_net_tls_host,
    "The host a failed TLS handshake was with. NULL for any other failure.",
    |r| r.tls_host().map(str::to_string)
);
row_string!(
    beacon_net_tls_fingerprint,
    "The SHA-256 fingerprint of the certificate a failed TLS handshake was shown, as `AB:CD:...`. NULL when there was none, or for any other failure.",
    |r| r.tls_fingerprint()
);
row_string!(
    beacon_net_body_text,
    "The captured response preview, decoded for display. NULL when no body was captured; check `body_evicted` to tell 'dropped' from 'never taken'.",
    |r| r
        .body
        .as_ref()
        .map(|bytes| devtools::format_body(bytes, r.body_truncated))
);

/// One header off a row, by index into `request_header_count` / `response_header_count`.
macro_rules! header_string {
    ($name:ident, $doc:literal, |$request:ident| $list:expr, $part:tt) => {
        #[doc = $doc]
        ///
        /// Free with [`crate::beacon_string_free`]; NULL when out of range.
        ///
        /// # Safety
        /// `browser` must be a live handle from [`crate::beacon_new`].
        #[no_mangle]
        pub unsafe extern "C" fn $name(browser: *mut BeaconBrowser, index: usize, header: usize) -> *mut c_char {
            let b = browser!(browser, std::ptr::null_mut());
            let Some($request) = b.requests.get(index) else {
                return std::ptr::null_mut();
            };
            match $list.get(header) {
                Some(pair) => to_c_string(&pair.$part),
                None => std::ptr::null_mut(),
            }
        }
    };
}

header_string!(
    beacon_net_request_header_name,
    "The name of request header `header`.",
    |r| r.request_headers,
    0
);
header_string!(
    beacon_net_request_header_value,
    "The value of request header `header`, redacted unless sensitive headers were asked for.",
    |r| r.request_headers,
    1
);
header_string!(
    beacon_net_response_header_name,
    "The name of response header `header`.",
    |r| r.headers,
    0
);
header_string!(
    beacon_net_response_header_value,
    "The value of response header `header`.",
    |r| r.headers,
    1
);
header_string!(beacon_net_redirect_url, "Where redirect hop `hop` pointed.", |r| r.redirects, 1);

/// The status code of redirect hop `hop`, or 0 when out of range.
///
/// # Safety
/// `browser` must be a live handle from [`crate::beacon_new`].
#[no_mangle]
pub unsafe extern "C" fn beacon_net_redirect_status(browser: *mut BeaconBrowser, index: usize, hop: usize) -> u32 {
    let b = browser!(browser, 0);
    b.requests
        .get(index)
        .and_then(|r| r.redirects.get(hop))
        .map(|(status, _)| *status as u32)
        .unwrap_or(0)
}

/// One string off entry `entry` of one of a row's detail lists, by index into the matching
/// `*_count`.
macro_rules! detail_string {
    ($name:ident, $doc:literal, |$request:ident| $list:expr, |$item:ident| $value:expr) => {
        #[doc = $doc]
        ///
        /// Free with [`crate::beacon_string_free`]; NULL when out of range.
        ///
        /// # Safety
        /// `browser` must be a live handle from [`crate::beacon_new`].
        #[no_mangle]
        pub unsafe extern "C" fn $name(browser: *mut BeaconBrowser, index: usize, entry: usize) -> *mut c_char {
            let b = browser!(browser, std::ptr::null_mut());
            let Some($request) = b.requests.get(index) else {
                return std::ptr::null_mut();
            };
            match $list.get(entry) {
                Some($item) => to_c_string(&$value),
                None => std::ptr::null_mut(),
            }
        }
    };
}

/// One number off entry `entry` of a detail list, `$absent` when out of range.
macro_rules! detail_value {
    ($name:ident, $doc:literal, $ty:ty, $absent:expr, |$request:ident| $list:expr, |$item:ident| $value:expr) => {
        #[doc = $doc]
        ///
        /// # Safety
        /// `browser` must be a live handle from [`crate::beacon_new`].
        #[no_mangle]
        pub unsafe extern "C" fn $name(browser: *mut BeaconBrowser, index: usize, entry: usize) -> $ty {
            let b = browser!(browser, $absent);
            let Some($request) = b.requests.get(index) else {
                return $absent;
            };
            match $list.get(entry) {
                Some($item) => $value,
                None => $absent,
            }
        }
    };
}

detail_value!(
    beacon_net_cache_outcome,
    "What the cache did, for cache entry `entry`. `None` when out of range.",
    BeaconCacheOutcome,
    BeaconCacheOutcome::None,
    |r| r.cache,
    |item| cache_outcome(Some(item.1))
);
detail_string!(
    beacon_net_cache_label,
    "What the cache did, for cache entry `entry`, in words: `served from cache`, `revalidated (304)`, ...",
    |r| r.cache,
    |item| item.1.to_string()
);
detail_string!(
    beacon_net_cache_url,
    "The hop cache entry `entry` was about.",
    |r| r.cache,
    |item| item.0
);

detail_string!(
    beacon_net_auth_url,
    "The hop that was answered with auth entry `entry`'s `401` or `407`.",
    |r| r.auth,
    |item| item.url
);
detail_string!(
    beacon_net_auth_challenges,
    "The challenges auth entry `entry` offered, as one line: `Basic realm=\"staff\", Bearer`.",
    |r| r.auth,
    |item| item.challenges_label()
);
detail_value!(
    beacon_net_auth_is_proxy,
    "Whether auth entry `entry` was a proxy's `407` rather than the server's `401`. False when out of range.",
    bool,
    false,
    |r| r.auth,
    |item| item.target == gosub_engine::events::AuthTarget::Proxy
);
detail_value!(
    beacon_net_auth_retried,
    "Whether the hop of auth entry `entry` was sent again with credentials. False means the `401`/`407` is the response; also false when out of range.",
    bool,
    false,
    |r| r.auth,
    |item| item.retried
);

detail_string!(
    beacon_net_upgrade_from,
    "The `http` URL upgrade `entry` rewrote.",
    |r| r.upgrades,
    |item| item.from
);
detail_string!(
    beacon_net_upgrade_to,
    "The `https` URL upgrade `entry` sent instead.",
    |r| r.upgrades,
    |item| item.to
);
detail_string!(
    beacon_net_upgrade_reason,
    "Why upgrade `entry` happened: `HSTS`, `mixed content`, ...",
    |r| r.upgrades,
    |item| item.reason.to_string()
);

detail_string!(
    beacon_net_change_url,
    "The hop a policy changed, for change `entry`.",
    |r| r.changes,
    |item| item.0
);
detail_string!(
    beacon_net_change_label,
    "What a policy changed instead of refusing the request, for change `entry`, in words.",
    |r| r.changes,
    |item| item.1.to_string()
);

detail_value!(
    beacon_net_retry_attempt,
    "Which retry entry `entry` was, from 1. 0 when out of range.",
    u32,
    0,
    |r| r.retries,
    |item| item.attempt
);
detail_value!(
    beacon_net_retry_delay_us,
    "How long the stack waited before retry `entry`. `BEACON_ABSENT` when out of range.",
    u64,
    BEACON_ABSENT,
    |r| r.retries,
    |item| item.delay_us
);
detail_string!(
    beacon_net_retry_reason,
    "Why the attempt before retry `entry` failed, in the stack's words.",
    |r| r.retries,
    |item| item.reason
);

detail_string!(
    beacon_net_preflight_url,
    "The hop CORS preflight `entry` asked permission for.",
    |r| r.preflights,
    |item| item.url
);
detail_value!(
    beacon_net_preflight_us,
    "How long CORS preflight `entry` took. `BEACON_ABSENT` while it is out, for one that failed outright, and when out of range.",
    u64,
    BEACON_ABSENT,
    |r| r.preflights,
    |item| absent(item.elapsed_us)
);
