//! 整理每一趟的按身份查：512 与 2048 个数据单元的两种池上，每批各解一次进映射的叶、看过的索引项数次线性增长。

use super::*;

/// 增长那一条量的两档：每个文件八个数据单元，512 个与 2048 个数据单元。
const DATA_UNITS_PER_FILE_OF_THE_GROWTH_POOLS: usize = 8;

const FILES_OF_THE_SMALLER_GROWTH_POOL: u64 = 64;

const FILES_OF_THE_LARGER_GROWTH_POOL: u64 = 256;

/// 一批整理的批大小（E185 的 K = 8）。
const COMPACTION_BATCH_UNITS_OF_THE_GROWTH_POOLS: usize = 8;

/// 每档量几次（一次调用至多发一版）。
const MEASURED_CALLS_PER_POOL: usize = 8;

/// 每档至少要有几次真发了一版才算数。
const PUBLISHING_CALLS_REQUIRED_PER_POOL: usize = 5;

/// 每次整理之前那一次用户改动（建一个空 inode）写进记录的时间秒：比造池那几次都晚，各次相同。
const WRITE_TIME_SECONDS_OF_THE_USER_CHANGE_BEFORE_EACH_CALL: u64 =
    crate::common::FIXED_WRITE_TIME_SECONDS + 7200;

/// 一次调用推一次整理：先发一次用户改动（建一个空 inode），再把这次挂载数过的发布数压到上限，一次调用至多发一版
/// （照 E185 装置 `measure_one_intent`）。那一次改动发成就清掉会话上跨趟留着的上一条完成的意图，
/// 每次调用都照常签新意图或接着推活意图，不因为前面哪一条意图没有净产出就停在「上一条无产出」：这一条量的是查找量随池子长的增长，与停不停无关。
/// 调用之前判整理读这一批要读的那一版恰好解进映射的 key 走到的那几片叶；交回这一次整理在产品里做的查找（每一处合起来，那一次用户改动不在其内）；
/// 一版都没发的交回 `None`。
fn lookup_work_of_one_compaction_call(
    session: &mut MountedSession,
    devices: &mut SparseDevices,
) -> Option<LookupWork> {
    session
        .publish_user_change(
            devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_USER_CHANGE_BEFORE_EACH_CALL,
            },
        )
        .unwrap_or_else(|refusal| panic!("整理之前建一个空 inode：{refusal:?}"));
    session.compaction.publishes_since_the_last_user_change = 1000;
    assert_the_compaction_read_decodes_exactly_the_routed_leaves(version_of_the_session(session));
    let (run, lookup_work) = run_compaction_and_count_its_lookups(session, devices);
    match run.outcome {
        CompactionRunOutcome::Stopped(_) => {}
        CompactionRunOutcome::Failed(failure) => panic!("整理的一次发布没发成：{failure:?}"),
    }
    (!run.versions_published.is_empty()).then(|| lookup_work.of_every_site())
}

/// 造一档池（`files` 个八单元的文件），批大小强制 K 重挂，推 `MEASURED_CALLS_PER_POOL` 次整理（每次之前一次用户改动）：交回真发了一版的那几次
/// 各自做的查找，按调用的次序。
fn lookup_work_of_the_compaction_calls_on_a_pool_of(files: u64) -> Vec<LookupWork> {
    let mut devices = build_a_pool_of_files(
        PoolOfFiles {
            files,
            data_units_per_file: DATA_UNITS_PER_FILE_OF_THE_GROWTH_POOLS,
        },
        &mut |_| {},
    )
    .devices;
    let parameters = crate::common::parameters();
    let switches = product_switches_with_the_batch_size(COMPACTION_BATCH_UNITS_OF_THE_GROWTH_POOLS);
    let (_, mut session) = MountedSession::of_the_mount(
        mount_writable_with_compaction_switches(&parameters, &mut devices, switches)
            .unwrap_or_else(|refusal| panic!("重挂 {files} 个文件的池：{refusal:?}")),
    );
    // 迭代上界是 MEASURED_CALLS_PER_POOL；跨轮携带的是会话与盘。
    (0..MEASURED_CALLS_PER_POOL)
        .filter_map(|_| lookup_work_of_one_compaction_call(&mut session, &mut devices))
        .collect()
}

/// 几次调用做的查找合起来。
fn sum_of_the_lookup_work(readings: &[LookupWork]) -> LookupWork {
    readings
        .iter()
        .fold(LookupWork::default(), |sum, reading| sum.plus(*reading))
}

/// 512 个数据单元的池与 2048 个的池（每个文件八个单元），各推 8 次整理（K = 8、一次至多一版，每次之前发一次用户改动），读产品里记下的计数，不读挂钟、
/// 不读 CPU 时间：每一批读的那一版上，整理读它恰好解进映射的 key 走到的那几片叶；每一趟整理每次查至多一次折半查找、计划里每一项都经
/// 按 key 的索引查过；查一次平均看过的索引项数与每一趟建索引的次数从小池到大池涨不到门槛的倍数。两档互不依赖：各造各的池、各挂各的会话，
/// 计数记在各自的线程上，各在一个线程里量，合并按档的次序。
/// 判别力：`crates/mutations.tsv` 里以「七·6 增长：」起头的几行（按身份查的索引退回逐项找、整理读版本退回逐 key 查映射）。
// harness-test-granularity:one-scenario 两档池同一个场景：比的是同一串操作在两档上的比，单造一档量不出比。
pub fn run_the_scenario_where_compaction_batches_decode_each_routed_mapping_leaf_once_and_examine_index_entries_sublinearly_from_512_to_2048_data_units(
) {
    let (smaller_readings, larger_readings) = std::thread::scope(|scope| {
        let smaller = scope.spawn(|| {
            lookup_work_of_the_compaction_calls_on_a_pool_of(FILES_OF_THE_SMALLER_GROWTH_POOL)
        });
        let larger = scope.spawn(|| {
            lookup_work_of_the_compaction_calls_on_a_pool_of(FILES_OF_THE_LARGER_GROWTH_POOL)
        });
        (
            smaller.join().expect("量小池的线程没 panic"),
            larger.join().expect("量大池的线程没 panic"),
        )
    });
    println!(
        "publish_lookup_by_identity growth smaller_readings={smaller_readings:?} larger_readings={larger_readings:?}"
    );
    assert!(
        smaller_readings.len() >= PUBLISHING_CALLS_REQUIRED_PER_POOL
            && larger_readings.len() >= PUBLISHING_CALLS_REQUIRED_PER_POOL,
        "两档各至少 {PUBLISHING_CALLS_REQUIRED_PER_POOL} 次真发了一版：小池 {smaller_readings:?}，大池 {larger_readings:?}"
    );
    let smaller = sum_of_the_lookup_work(&smaller_readings);
    let larger = sum_of_the_lookup_work(&larger_readings);
    let smaller_calls = u64::try_from(smaller_readings.len()).expect("次数");
    let larger_calls = u64::try_from(larger_readings.len()).expect("次数");
    let growth_in_percent_of_the_entries_examined =
        growth_in_percent_of_the_entries_examined_per_lookup(smaller, larger);
    let growth_in_percent_of_the_index_builds =
        larger.index_builds * smaller_calls * 100 / (smaller.index_builds * larger_calls);
    println!(
        "publish_lookup_by_identity growth smaller={smaller:?} larger={larger:?} growth_in_percent={growth_in_percent_of_the_entries_examined} index_builds_growth_in_percent={growth_in_percent_of_the_index_builds}"
    );
    assert!(
        growth_in_percent_of_the_entries_examined
            < HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE,
        "数据单元四倍，整理每次按身份查平均看过的索引项数涨了 {growth_in_percent_of_the_entries_examined}%，门槛 {HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE}%（小池 {smaller:?}，大池 {larger:?}）"
    );
    assert!(
        growth_in_percent_of_the_index_builds
            < HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE,
        "数据单元四倍，整理每一趟建索引的次数涨了 {growth_in_percent_of_the_index_builds}%，门槛 {HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE}%（每查一把 key 现建一个索引就按 key 数涨；小池 {smaller:?} 共 {smaller_calls} 次，大池 {larger:?} 共 {larger_calls} 次）"
    );
}
