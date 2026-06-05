#![no_main]
//! Fuzz the **vendor-oracle receipt JSON ingester** (`from_json`) — the no-dep, receipt-scoped,
//! fail-closed reader for externally-produced `vendor-oracle-receipt-v1` JSON. This is a hostile
//! third-party-input surface (the receipts come from an external lab). Contract: any byte string must
//! be a typed `Ok(receipt)` or `Err(ReceiptParseError)` — **never a panic**; and a parse failure is
//! distinct from an inadmissible receipt. The producer's self-assessed `admission` is ignored by design.
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The reader takes &str; non-UTF-8 is simply not a receipt (skip — that path is the lexer's, not
    // the JSON reader's). Valid-UTF-8 garbage must still never panic the recursive-descent reader.
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = tzcompile::vendor_oracle::VendorOracleReceipt::from_json(s);
    }
});
