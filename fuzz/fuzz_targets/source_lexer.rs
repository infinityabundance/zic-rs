#![no_main]
//! Fuzz the **tzdata source lexer** (`source::lexer::tokenize`) — the first stage over untrusted `.zi`
//! text (line splitting, NUL/overlong/unterminated-line/quote handling: the `ZIC016`/`ZIC017`/`ZIC021`/
//! `ZIC022` admissibility layer, T14). Contract: any byte string is a typed `Ok(lines)` or `Err`
//! diagnostic — never a panic. The line cap (`MAX_LINE_LEN`) bounds work per line.
use libfuzzer_sys::fuzz_target;
use std::path::Path;

fuzz_target!(|data: &[u8]| {
    let _ = tzcompile::source::lexer::tokenize(data, Path::new("<fuzz>"));
});
