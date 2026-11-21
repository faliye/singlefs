use super::*;

#[test]
fn the_spread_takes_the_middle_value_or_the_mean_of_the_two_middle_ones() {
    assert_eq!(Spread::of(&[]), None);
    assert_eq!(
        Spread::of(&[5.0]),
        Some(Spread {
            minimum: 5.0,
            median: 5.0,
            maximum: 5.0
        })
    );
    assert_eq!(
        Spread::of(&[3.0, 1.0, 2.0]),
        Some(Spread {
            minimum: 1.0,
            median: 2.0,
            maximum: 3.0
        })
    );
    assert_eq!(
        Spread::of(&[4.0, 1.0, 3.0, 2.0]),
        Some(Spread {
            minimum: 1.0,
            median: 2.5,
            maximum: 4.0
        })
    );
    assert_eq!(Spread::text(None), "minimum=none median=none maximum=none");
}

#[test]
fn the_options_accept_repeated_values_and_flags_and_refuse_unknown_ones() {
    let words =
        |list: &[&str]| -> Vec<String> { list.iter().map(|word| (*word).to_string()).collect() };
    let parsed = ParsedOptions::parse(
        &words(&["--dump", "/a", "--dump", "/b", "--out", "/o", "--controls"]),
        &["--dump", "--out"],
        &["--controls"],
    )
    .expect("合法");
    assert_eq!(
        parsed.paths("--dump"),
        Ok(vec![PathBuf::from("/a"), PathBuf::from("/b")])
    );
    assert_eq!(parsed.single("--out"), Ok("/o"));
    assert!(parsed.flag("--controls"));
    assert!(
        parsed.single("--dump").is_err(),
        "--dump 给了两次，single 要拒"
    );
    assert!(ParsedOptions::parse(&words(&["--out"]), &["--out"], &[])
        .expect_err("缺值")
        .contains("缺值"));
    assert!(
        ParsedOptions::parse(&words(&["--nonsense"]), &["--out"], &[])
            .expect_err("认不出")
            .contains("--nonsense")
    );
    assert!(parsed.paths("--missing").is_err());
}
