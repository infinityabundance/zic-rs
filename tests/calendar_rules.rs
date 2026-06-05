//! Calendar `ON`-day resolution, exercised both directly (the public calendar API) and
//! end-to-end (through the compiler, so the resolution is tied to real transition instants).

use std::path::PathBuf;

use tzcompile::model::calendar::{resolve_on_day, OnDay, Weekday};
use tzcompile::{compile_zone, load_database};

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

#[test]
fn on_day_forms_resolve() {
    // lastSun of October 2021 is the 31st.
    let r = resolve_on_day(OnDay::Last(Weekday::Sun), 2021, 10).unwrap();
    assert_eq!((r.month, r.day), (10, 31));

    // Sun<=25 in October 2015 is the 25th (a Sunday).
    let r = resolve_on_day(OnDay::OnBefore(Weekday::Sun, 25), 2015, 10).unwrap();
    assert_eq!((r.month, r.day), (10, 25));

    // Sun>=8 in March 2020 is the 8th.
    let r = resolve_on_day(OnDay::OnAfter(Weekday::Sun, 8), 2020, 3).unwrap();
    assert_eq!((r.month, r.day), (3, 8));
}

#[test]
fn sun_le_25_drives_the_right_transition_instant() {
    // The Sle fixture uses `Sun<=25` with a standard-time AT (`2:00s`). April Sun<=25 2015
    // is the 19th; STDOFF is -3:00, so the spring transition is 2015-04-19 05:00 UTC
    // (02:00 standard + 3h). Verifies calendar resolution feeds the standard-time path.
    let db = load_database(&[fixture("fixtures/minimal/sle.zi")]).unwrap();
    let d = compile_zone(&db, "Test/Sle").unwrap();
    assert_eq!(d.transitions.len(), 2);

    // 2015-04-19 05:00:00 UTC = 1429419600.
    assert_eq!(d.transitions[0].at, 1_429_419_600);
    let summer = &d.types[d.transitions[0].type_index as usize];
    assert_eq!((summer.utoff, summer.is_dst), (-7200, true)); // -3:00 + 1:00 DST
}
