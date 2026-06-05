//! LOOM model of zic-rs's only lock-free shared state: the per-process AtomicU64 sequence counters
//! that generate UNIQUE temp-file names during atomic writes (`src/fs/atomic_write.rs::TEMP_SEQ`,
//! `src/fs/output_tree.rs::SEQ`). Both use the identical pattern `let seq = SEQ.fetch_add(1, Relaxed);`
//! and embed `seq` into the temp name `.{base}.tmp.{pid}.{seq}`.
//!
//! The load-bearing concurrency invariant: `fetch_add(1, Relaxed)` hands a DISTINCT value to every
//! concurrent caller, so two threads in one process writing files in parallel never generate the same
//! temp name -> never collide -> the `create_new`/`hard_link` atomic-write exclusivity (the TOCTOU-safe
//! path) holds under intra-process concurrency. loom exhaustively explores thread interleavings.
//!
//! Honest scope: loom cannot instrument a `static` directly (loom atomics aren't const-constructible as
//! statics), so this models the exact `fetch_add(1, Relaxed)` ALGORITHM the real counters use, over a
//! shared loom Arc<AtomicU64> — the standard loom methodology. Inter-process collisions are handled
//! separately by PID + `create_new` exclusivity (not a loom concern).
#![cfg(loom)]

#[cfg(test)]
mod tests {
    use loom::sync::atomic::{AtomicU64, Ordering};
    use loom::sync::Arc;
    use loom::thread;
    use std::collections::HashSet;
    use std::sync::Mutex;

    // Two concurrent callers of the temp-name counter must receive distinct sequence values
    // under EVERY interleaving loom explores.
    #[test]
    fn fetch_add_yields_unique_sequence_under_all_interleavings() {
        loom::model(|| {
            let seq = Arc::new(AtomicU64::new(0));
            let seen = Arc::new(Mutex::new(HashSet::new()));

            let handles: Vec<_> = (0..2)
                .map(|_| {
                    let seq = seq.clone();
                    let seen = seen.clone();
                    thread::spawn(move || {
                        let v = seq.fetch_add(1, Ordering::Relaxed);
                        // record the value this caller would embed into its temp name
                        let mut s = seen.lock().unwrap();
                        // the invariant: no other concurrent caller saw the same value
                        assert!(s.insert(v), "duplicate sequence {v} -> temp-name collision");
                    })
                })
                .collect();
            for h in handles {
                h.join().unwrap();
            }
            // both callers ran; two distinct values were produced
            assert_eq!(seen.lock().unwrap().len(), 2);
        });
    }
}
