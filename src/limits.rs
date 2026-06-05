//! Resource limits (T17.1b) — generous hard caps on input-driven dimensions.
//!
//! Reference `zic` caps none of these, so every cap here is a **bucket-3 intentional safer
//! divergence** (see `docs/differences-from-reference-zic.md`): a reliability boundary that stops a
//! malformed or adversarial source set from exhausting memory before [`compile`](crate::compile) ever
//! runs. The defaults sit **far above any real tzdb** (2026b: ~350 zones, ~600 links, tens of rules
//! per set, 27 leap seconds, link chains 1–3 deep, ~15 eras per zone), so no legitimate input is ever
//! rejected — they exist only to bound the pathological tail.
//!
//! A breach is a plain [`Error::config`](crate::error::Error::config) (exit 1), **not** a `ZIC###`
//! diagnostic: a cap is an operational safety limit, not a `zic`-grammar violation, so the diagnostic
//! contract's code space stays reserved for source-grammar conditions. This pairs with the
//! `tzif::validate::parse` bounds-guard (T17.1a) and the panic policy (`docs/panic-policy.md`).

use crate::error::{Error, Result};
use crate::model::Database;

/// Default per-file source-byte ceiling (512 MiB; a real `tzdata.zi` is a few hundred KB).
pub const DEFAULT_SOURCE_BYTES_MAX: usize = 512 * 1024 * 1024;
/// Default cap on the number of `Zone` records (real tzdb: ~350).
pub const DEFAULT_ZONE_COUNT_MAX: usize = 1_000_000;
/// Default cap on the number of `Rule` rows in any one named rule set (real: tens–hundreds).
pub const DEFAULT_RULE_COUNT_MAX: usize = 1_000_000;
/// Default cap on the number of `Link` records (real: ~600).
pub const DEFAULT_LINK_COUNT_MAX: usize = 1_000_000;
/// Default cap on leap-second-table entries (real: 27).
pub const DEFAULT_LEAP_COUNT_MAX: usize = 100_000;
/// Default cap on link-chain resolution depth (real chains are 1–3 hops; also bounds the
/// `visited.contains` cost in [`resolve_link_target`](crate::resolve_link_target)).
pub const DEFAULT_LINK_CHAIN_DEPTH_MAX: usize = 256;
/// Default cap on continuation eras within a single `Zone` (real: ~15).
pub const DEFAULT_ZONE_ERA_COUNT_MAX: usize = 100_000;

/// Generous reliability caps on input-driven resource dimensions. [`Default`] is the production set;
/// tests construct tiny instances to exercise enforcement without giant fixtures, and a future CLI
/// (T17.2) can expose overrides.
#[derive(Debug, Clone, Copy)]
pub struct ResourceLimits {
    /// Maximum bytes in any single source file (checked before parsing each file).
    pub source_bytes_max: usize,
    /// Maximum number of `Zone` records in the assembled database.
    pub zone_count_max: usize,
    /// Maximum number of `Rule` rows in any one named rule set.
    pub rule_count_max: usize,
    /// Maximum number of `Link` records in the assembled database.
    pub link_count_max: usize,
    /// Maximum number of leap-second-table entries.
    pub leap_count_max: usize,
    /// Maximum link-chain resolution depth.
    pub link_chain_depth_max: usize,
    /// Maximum continuation eras within a single `Zone`.
    pub zone_era_count_max: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        ResourceLimits {
            source_bytes_max: DEFAULT_SOURCE_BYTES_MAX,
            zone_count_max: DEFAULT_ZONE_COUNT_MAX,
            rule_count_max: DEFAULT_RULE_COUNT_MAX,
            link_count_max: DEFAULT_LINK_COUNT_MAX,
            leap_count_max: DEFAULT_LEAP_COUNT_MAX,
            link_chain_depth_max: DEFAULT_LINK_CHAIN_DEPTH_MAX,
            zone_era_count_max: DEFAULT_ZONE_ERA_COUNT_MAX,
        }
    }
}

impl ResourceLimits {
    /// Reject a source file whose byte length exceeds [`Self::source_bytes_max`]. Checked once per
    /// file, before it is handed to the parser (so an oversize input never gets fully tokenised).
    pub fn check_source_bytes(&self, len: usize, path: &std::path::Path) -> Result<()> {
        if len > self.source_bytes_max {
            return Err(Error::config(format!(
                "source file {} is {len} bytes, exceeding the zic-rs resource limit of {} bytes",
                path.display(),
                self.source_bytes_max
            )));
        }
        Ok(())
    }

    /// Reject a leap-second table with more than [`Self::leap_count_max`] entries.
    pub fn check_leap_count(&self, n: usize) -> Result<()> {
        if n > self.leap_count_max {
            return Err(Error::config(format!(
                "leap-second table has {n} entries, exceeding the zic-rs resource limit of {}",
                self.leap_count_max
            )));
        }
        Ok(())
    }

    /// Validate an assembled [`Database`] against the count caps: zones, links, rows-per-rule-set, and
    /// continuation-eras-per-zone. Called once after parsing, before any compile.
    pub fn enforce(&self, db: &Database) -> Result<()> {
        if db.zones.len() > self.zone_count_max {
            return Err(Error::config(format!(
                "zone count {} exceeds the zic-rs resource limit of {}",
                db.zones.len(),
                self.zone_count_max
            )));
        }
        if db.links.len() > self.link_count_max {
            return Err(Error::config(format!(
                "link count {} exceeds the zic-rs resource limit of {}",
                db.links.len(),
                self.link_count_max
            )));
        }
        for (name, set) in &db.rules {
            if set.len() > self.rule_count_max {
                return Err(Error::config(format!(
                    "rule set {name:?} has {} rows, exceeding the zic-rs resource limit of {}",
                    set.len(),
                    self.rule_count_max
                )));
            }
        }
        for z in &db.zones {
            if z.eras.len() > self.zone_era_count_max {
                return Err(Error::config(format!(
                    "zone {:?} has {} continuation eras, exceeding the zic-rs resource limit of {}",
                    z.name,
                    z.eras.len(),
                    self.zone_era_count_max
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::time::Offset;
    use crate::model::{Database, LinkRecord, Origin, ZoneEra, ZoneRecord, ZoneRules};

    fn origin() -> Origin {
        Origin::new(std::path::Path::new("test.zi"), 1)
    }

    fn tiny() -> ResourceLimits {
        ResourceLimits {
            source_bytes_max: 10,
            zone_count_max: 1,
            rule_count_max: 1,
            link_count_max: 1,
            leap_count_max: 1,
            link_chain_depth_max: 4,
            zone_era_count_max: 1,
        }
    }

    fn zone(name: &str) -> ZoneRecord {
        ZoneRecord {
            name: name.to_string(),
            eras: Vec::new(),
            origin: origin(),
        }
    }

    fn link(from: &str, to: &str) -> LinkRecord {
        LinkRecord {
            target: to.to_string(),
            link_name: from.to_string(),
            origin: origin(),
        }
    }

    #[test]
    fn source_bytes_cap_rejects_oversize() {
        let l = tiny();
        assert!(l
            .check_source_bytes(11, std::path::Path::new("big.zi"))
            .is_err());
        assert!(l
            .check_source_bytes(10, std::path::Path::new("ok.zi"))
            .is_ok());
    }

    #[test]
    fn leap_count_cap_rejects_overflow() {
        let l = tiny();
        assert!(l.check_leap_count(2).is_err());
        assert!(l.check_leap_count(1).is_ok());
    }

    #[test]
    fn zone_and_link_count_caps() {
        let l = tiny();
        let mut db = Database::default();
        db.zones.push(zone("A"));
        db.links.push(link("L", "A"));
        assert!(l.enforce(&db).is_ok(), "one each is within the cap");
        db.zones.push(zone("B"));
        let err = l.enforce(&db).unwrap_err();
        assert!(err.to_string().contains("zone count 2 exceeds"));
    }

    fn era() -> ZoneEra {
        ZoneEra {
            stdoff: Offset(0),
            rules: ZoneRules::None,
            format: String::new(),
            until: None,
            origin: origin(),
        }
    }

    #[test]
    fn era_count_cap() {
        // tiny cap is 1 era per zone; a single zone with 2 continuation eras breaches it (and the
        // rule-set cap shares this exact `len() > max` loop shape).
        let l = tiny();
        let mut db = Database::default();
        let mut z = zone("A");
        z.eras.push(era());
        db.zones.push(z.clone());
        assert!(l.enforce(&db).is_ok(), "one era is within the cap");
        db.zones[0].eras.push(era());
        let err = l.enforce(&db).unwrap_err();
        assert!(
            err.to_string().contains("continuation eras"),
            "expected an era-count breach, got: {err}"
        );
    }

    #[test]
    fn link_chain_depth_cap_bounds_long_acyclic_chains() {
        // A straight acyclic chain L0 -> L1 -> ... -> L400 that never reaches a zone. The cycle check
        // can't catch it (no repeat), so the depth cap must stop it with a typed error, not a hang.
        let mut db = Database::default();
        for i in 0..400 {
            db.links
                .push(link(&format!("L{i}"), &format!("L{}", i + 1)));
        }
        let err = crate::resolve_link_target(&db, "L0").unwrap_err();
        assert!(
            err.to_string().contains("depth limit"),
            "expected a link-chain depth-limit error, got: {err}"
        );
    }

    #[test]
    fn defaults_pass_a_realistic_database() {
        // The production defaults must never reject a normal-sized database.
        let l = ResourceLimits::default();
        let mut db = Database::default();
        for i in 0..500 {
            db.zones.push(zone(&format!("Zone/{i}")));
            db.links
                .push(link(&format!("Alias/{i}"), &format!("Zone/{i}")));
        }
        assert!(l.enforce(&db).is_ok());
    }
}
