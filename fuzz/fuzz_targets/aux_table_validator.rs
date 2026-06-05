#![no_main]
//! Fuzz the **auxiliary-table validator** (`aux_tables::validate_zone_table`) over `zone.tab`-shaped
//! untrusted text (column counts, country codes, coordinates, UTF-8, duplicate rows). Contract: every
//! input yields an `AuxTableValidation` verdict — never a panic. Also exercises `iso3166_codes`
//! (the code-set extractor). These are policy/reference tables, not compile input (the T16.4 boundary).
use libfuzzer_sys::fuzz_target;
use tzcompile::aux_tables::{iso3166_codes, validate_zone_table, ZoneTableKind};

fuzz_target!(|data: &[u8]| {
    // Each table kind has slightly different rules; exercise the two cross-validating kinds + the
    // now/future kind. `None` = no iso3166 cross-validation set (the validator must still not panic).
    let _ = validate_zone_table(ZoneTableKind::ZoneTab, data, None);
    let _ = validate_zone_table(ZoneTableKind::Zone1970Tab, data, None);
    let _ = validate_zone_table(ZoneTableKind::ZonenowTab, data, None);
    let _ = iso3166_codes(data);
});
