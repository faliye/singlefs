use super::*;

/// 本机 fio 3.36 在一个 8 MiB 文件上跑 4 KiB 随机读写的原样 terse 行（字段号拿同一次的 JSON 输出逐个对过）。
const FIO_3_36_TERSE_LINE: &str = "3;fio-3.36;probe;0;0;3976;62125;15531;64;0;0;0.000000;0.000000;48;281;62.555513;20.863914;1.000000%=50;5.000000%=54;10.000000%=56;20.000000%=58;30.000000%=59;40.000000%=60;50.000000%=60;60.000000%=61;70.000000%=62;80.000000%=62;90.000000%=63;95.000000%=65;99.000000%=248;99.500000%=268;99.900000%=280;99.950000%=280;99.990000%=280;0%=0;0%=0;0%=0;48;281;62.575865;20.868117;0;0;0.000000%;0.000000;0.000000;4216;65875;16468;64;0;0;0.000000;0.000000;0;5;0.816231;0.361605;1.000000%=0;5.000000%=0;10.000000%=0;20.000000%=0;30.000000%=0;40.000000%=0;50.000000%=0;60.000000%=0;70.000000%=0;80.000000%=0;90.000000%=1;95.000000%=1;99.000000%=2;99.500000%=2;99.900000%=4;99.950000%=5;99.990000%=5;0%=0;0%=0;0%=0;0;5;0.840112;0.367051;0;0;0.000000%;0.000000;0.000000;0.000000%;4.761905%;995;0;0;100.0%;0.0%;0.0%;0.0%;0.0%;0.0%;0.0%;50.93%;0.39%;0.15%;0.00%;0.15%;47.90%;0.05%;0.44%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;nvme0n1;0;0;0;0;0;0;0;0.00%";

#[test]
fn block_layer_counters_parse_a_seventeen_field_statistics_line() {
    let text = "    1234        5    67890      100     2222       33   444444      555        0      600      700        0        0        0        0       12       34\n";
    let counters = parse_block_layer_counters(text).expect("17 个字段的 stat 行要解得出来");
    assert_eq!(counters.read_requests, 1234);
    assert_eq!(counters.read_sectors, 67890);
    assert_eq!(counters.write_requests, 2222);
    assert_eq!(counters.write_sectors, 444444);
    assert_eq!(counters.flush_requests, 12, "FLUSH 在下标 15，下标 14 是 discard 耗时");
}

#[test]
fn block_layer_counters_refuse_a_line_without_the_flush_field() {
    let eleven_fields = "1 2 3 4 5 6 7 8 9 10 11";
    assert_eq!(parse_block_layer_counters(eleven_fields), None, "读不到 FLUSH 不许拿 0 顶上");
    assert_eq!(parse_block_layer_counters("1 2 x 4 5 6 7 8 9 10 11 12 13 14 15 16"), None);
}

#[test]
fn block_layer_delta_turns_sectors_into_bytes() {
    let earlier = BlockLayerCounters { read_requests: 10, read_sectors: 100, write_requests: 20, write_sectors: 200, flush_requests: 3 };
    let later = BlockLayerCounters { read_requests: 15, read_sectors: 108, write_requests: 26, write_sectors: 216, flush_requests: 5 };
    let delta = later.delta_since(earlier);
    assert_eq!(delta.read_requests, 5);
    assert_eq!(delta.read_bytes, 4096, "8 个扇区 × 512 字节");
    assert_eq!(delta.write_requests, 6);
    assert_eq!(delta.write_bytes, 8192, "16 个扇区 × 512 字节");
    assert_eq!(delta.flush_requests, 2);
}

#[test]
fn fio_terse_version_3_line_from_fio_3_36_parses_both_directions() {
    let report = parse_fio_terse_version_3(FIO_3_36_TERSE_LINE).expect("真实的 terse 行要解得出来");
    assert_eq!(report.error_code, 0);
    assert_eq!(report.read_kibibytes, 3976);
    assert_eq!(report.read_bandwidth_kibibytes_per_second, 62125);
    assert_eq!(report.read_operations_per_second, 15531);
    assert_eq!(report.read_completion_latency_percentile_99_microseconds, Some(248));
    assert_eq!(report.write_kibibytes, Some(4216));
    assert_eq!(report.write_bandwidth_kibibytes_per_second, Some(65875));
    assert_eq!(report.write_operations_per_second, Some(16468));
    assert_eq!(report.write_completion_latency_percentile_99_microseconds, Some(2));
}

#[test]
fn fio_terse_line_of_a_diskless_engine_with_121_fields_still_parses() {
    let fields: Vec<&str> = FIO_3_36_TERSE_LINE.split(';').take(121).collect();
    assert_eq!(fields.len(), 121);
    let report = parse_fio_terse_version_3(&fields.join(";")).expect("121 个字段的行要解得出来");
    assert_eq!(report.read_operations_per_second, 15531);
    assert_eq!(report.write_operations_per_second, Some(16468));
}

#[test]
fn fio_terse_parser_refuses_a_line_that_is_not_version_3() {
    let version_2 = FIO_3_36_TERSE_LINE.replacen("3;", "2;", 1);
    assert_eq!(parse_fio_terse_version_3(&version_2), None);
    assert_eq!(parse_fio_terse_version_3("fio: pid=0, err=2/file:filesetup.c"), None);
}

#[test]
fn direct_read_below_ninety_percent_of_fio_bytes_is_judged_cache_substituted() {
    let fio_bytes = 1_024_000;
    assert_eq!(judge_direct_input_output(DirectInputOutputCheck::DirectRead, fio_bytes, 921_600), CellVerdict::Counted, "恰好 90% 算数");
    assert_eq!(judge_direct_input_output(DirectInputOutputCheck::DirectRead, fio_bytes, 921_599), CellVerdict::CacheSubstituted);
    assert_eq!(judge_direct_input_output(DirectInputOutputCheck::NotChecked, fio_bytes, 0), CellVerdict::NotChecked);
}

#[test]
fn direct_write_below_ninety_percent_of_fio_bytes_is_judged_writes_absorbed() {
    let fio_bytes = 4_294_967_296;
    assert_eq!(judge_direct_input_output(DirectInputOutputCheck::DirectWrite, fio_bytes, 3_865_470_567), CellVerdict::Counted);
    assert_eq!(judge_direct_input_output(DirectInputOutputCheck::DirectWrite, fio_bytes, 3_865_470_566), CellVerdict::WritesAbsorbed);
}

#[test]
fn control_second_pass_with_no_block_reads_is_judged_cache_substituted() {
    assert_eq!(FioJob::ControlBufferedReadFirstPass.check(), DirectInputOutputCheck::NotChecked);
    assert_eq!(
        judge_direct_input_output(FioJob::ControlBufferedReadSecondPass.check(), 268_435_456, 0),
        CellVerdict::CacheSubstituted,
        "对照的第二遍块层一个字节都没读，判定函数必须把它判成被缓存顶替"
    );
}

#[test]
fn block_layer_counters_of_two_mirror_members_add_up() {
    let first = BlockLayerCounters { read_requests: 10, read_sectors: 100, write_requests: 20, write_sectors: 200, flush_requests: 3 };
    let second = BlockLayerCounters { read_requests: 1, read_sectors: 8, write_requests: 2, write_sectors: 16, flush_requests: 4 };
    let summed = first.plus(second);
    assert_eq!(summed.read_requests, 11);
    assert_eq!(summed.read_sectors, 108);
    assert_eq!(summed.write_requests, 22);
    assert_eq!(summed.write_sectors, 216);
    assert_eq!(summed.flush_requests, 7);
}

#[test]
fn software_raid_state_with_resync_or_recovery_is_synchronizing() {
    let clean = "Personalities : [raid1]\nmd0 : active raid1 vdb[1] vda[0]\n      16760832 blocks super 1.2 [2/2] [UU]\n\nunused devices: <none>\n";
    let resyncing = "Personalities : [raid1]\nmd0 : active raid1 vdb[1] vda[0]\n      16760832 blocks super 1.2 [2/2] [UU]\n      [>....................]  resync =  0.3% (52224/16760832) finish=5.3min speed=52224K/sec\n\nunused devices: <none>\n";
    let recovering = "Personalities : [raid1]\nmd0 : active raid1 vdb[2] vda[0]\n      16760832 blocks super 1.2 [2/1] [U_]\n      [=>...................]  recovery =  7.1% (1190272/16760832) finish=2.1min speed=119027K/sec\n\nunused devices: <none>\n";
    assert!(!software_raid_is_synchronizing(clean), "--assume-clean 建出来的阵列不同步");
    assert!(software_raid_is_synchronizing(resyncing));
    assert!(software_raid_is_synchronizing(recovering), "recovery 也是后台同步");
}
