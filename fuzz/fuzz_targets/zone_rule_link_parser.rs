#![no_main]
//! Fuzz the **Rule/Zone/Link parser** (`source::parser::parse_into`) — the structural stage that turns
//! lexed lines into the `Database` (record arity, continuation handling, `UnknownLineType`/
//! `ContinuationWithoutZone`/`DuplicateZone`, T13). Contract: any byte string is a typed `Ok(())` or
//! `Err` diagnostic — never a panic. (The T17.1b resource caps bound the assembled `Database` via
//! `load_database`; this target is the parser itself, so it relies on the per-line/per-record guards.)
use libfuzzer_sys::fuzz_target;
use std::path::Path;
use tzcompile::model::Database;

fuzz_target!(|data: &[u8]| {
    let mut db = Database::default();
    let _ = tzcompile::source::parser::parse_into(data, Path::new("<fuzz>"), &mut db);
});
