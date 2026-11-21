use super::*;

/// 持久集合按游程存、读回来逐条相同（空集合、以落了的写起头、以没落的写起头、全落）。
#[test]
fn a_persisted_set_round_trips_through_its_runs() {
    for persisted in [
        Vec::new(),
        vec![true],
        vec![false, false, true, true, true, false],
        vec![true, true, true],
        vec![false; 5],
    ] {
        let mut bytes = Vec::new();
        put_persisted_as_runs(&mut bytes, &persisted);
        let mut reader = RecordedBytesReader {
            bytes: &bytes,
            at: 0,
        };
        assert_eq!(
            read_persisted_from_runs(&mut reader),
            Some(persisted.clone()),
            "{persisted:?}"
        );
        assert_eq!(reader.at, bytes.len(), "读完没有剩");
    }
}
