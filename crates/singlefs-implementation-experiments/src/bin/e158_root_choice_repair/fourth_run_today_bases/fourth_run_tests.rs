use super::*;

/// 今天那一臂造的起始镜像经文件原样读回（镜像指纹、补上的根、每一片的读数）。
#[test]
fn today_base_round_trips_through_its_file_encoding() {
    let history = third_run_history_start(&GEOMETRY_PRIMARY, 1).expect("P(1, ·)");
    let base = TodayBase {
        image: history.pool.clone(),
        appended_roots: vec![((2, 7), 2), ((2, 8), 2)],
        page_reads: vec![3, 1],
    };
    let decoded = decode_today_base(&encode_today_base(&base)).expect("读回");
    assert_eq!(
        image_fingerprint(&decoded.image),
        image_fingerprint(&base.image)
    );
    assert_eq!(decoded.image, base.image);
    assert_eq!(decoded.appended_roots, base.appended_roots);
    assert_eq!(decoded.page_reads, base.page_reads);
    assert!(decode_today_base(b"E158R4B0").is_err());
}
