
// ───────────── 攻方原型（m3-batch60-61-code-r1 opus）：rerun.sh 把这一段接在拷进原型 crate 的流水线用例文件末尾 ─────────────
// P4：按计划导入之后本机库对这条流的读回，与整库导入（旧的 import_from）逐项比：结论（两种判器、观察者，按内容）、核对行（CPU 核对代码）
// 与它点名的事实行、计划里每条路径；两趟各自的对账、第 ③ 段比对。历史里由用户定的几步放开扫：份数（2、3）、本机那一份先跑与否、
// 第二台从本机库的快照起步与否（双机驱动的实际做法，crash-amplification-two-hosts.sh 的 library-for-peer）、本机先判着第二台那一份与否、
// 导入那一趟再跑一次、带不带 ignore 的点；核对开（SINGLEFS_CRASH_AMPLIFICATION_VERIFY=cpu，配了 GPU 的趟一律核）与不开。

fn opus_copy_library(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).expect("建得了副本目录");
    for entry in std::fs::read_dir(from).expect("读得了库目录") {
        let entry = entry.expect("库目录的一项");
        assert!(entry.file_type().expect("文件类型").is_file(), "判定库目录里只有文件");
        std::fs::copy(entry.path(), to.join(entry.file_name())).expect("拷得了库文件");
    }
}

/// 一趟的环境：每趟现建一个守卫（清掉一趟读的全部变量再设这几个），用完就丢。
fn opus_environment(
    directory: &EnvironmentRunDirectory,
    library: &std::path::Path,
    share: &str,
    weights: Option<&str>,
    import: Option<&str>,
    stop_file: Option<&std::path::Path>,
    verify: bool,
) -> common_crash_points::CrashAmplificationEnvironment {
    let mut environment = directory.environment();
    environment.set("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY", library.as_os_str());
    environment.set("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new(share));
    if let Some(weights) = weights {
        environment.set("SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS", OsStr::new(weights));
    }
    if let Some(import) = import {
        environment.set("SINGLEFS_CRASH_AMPLIFICATION_IMPORT", OsStr::new(import));
    }
    if let Some(stop_file) = stop_file {
        environment.set("SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE", stop_file.as_os_str());
    }
    if verify {
        environment.set("SINGLEFS_CRASH_AMPLIFICATION_VERIFY", OsStr::new("cpu"));
    }
    environment
}

/// 库里这条流的读回：一台判完那份库里列出的每个（块键，种类，版本）按内容读回；每个块键在 CPU 核对代码下的核对行与它点名的事实行；
/// 计划里每条路径的身份。
fn opus_readback(
    store: &VerdictStore,
    conclusion_keys: &[(VerdictBlockKey, ConclusionKind, ConclusionVersion)],
    plan_paths: &[String],
) -> Vec<String> {
    let cpu_digest = FactsVerifierOn::Cpu.verifier_digest();
    let mut lines = Vec::new();
    let mut block_keys: Vec<VerdictBlockKey> = Vec::new();
    for (key, kind, version) in conclusion_keys {
        if !block_keys.contains(key) {
            block_keys.push(key.clone());
        }
        lines.push(format!(
            "conclusion node={} start={} kind={kind:?} version={:02x}{:02x} {:?}",
            key.stream_or_node_name.as_str(),
            key.block_start.0,
            version.0[0],
            version.0[1],
            conclusion_read_back_by_content(store, key, kind, version)
        ));
    }
    for key in &block_keys {
        let row = store.read_verifier_block(key, &cpu_digest).expect("读得了核对行");
        let facts = row.as_ref().map(|row| {
            store
                .read_flow_facts(&row.facts_digest)
                .expect("读得了事实行")
                .map(|bytes| bytes.len())
        });
        lines.push(format!(
            "verifier node={} start={} row={row:?} facts_bytes={facts:?}",
            key.stream_or_node_name.as_str(),
            key.block_start.0
        ));
    }
    for path in plan_paths {
        lines.push(format!(
            "path {path} identity={:?}",
            store.identity_of_path(path).expect("读得了路径")
        ));
    }
    lines
}

fn opus_comparison_text(run: &EnvironmentRun) -> String {
    match &run.comparison {
        Some(comparison) => format!(
            "compared={} without_verifier={} without_judge={} disagreements={}",
            comparison.states_compared,
            comparison.blocks_without_verifier,
            comparison.blocks_without_judge,
            comparison.disagreements
        ),
        None => "comparison=none".to_string(),
    }
}

fn opus_run_text(run: &EnvironmentRun) -> String {
    format!(
        "share={}/{} checked={} imported={} unjudged={} violations={} stopped={} {}",
        run.share.index,
        run.share.count,
        run.checking.blocks_checked,
        run.imported_blocks,
        run.unjudged_blocks,
        run.violations.len(),
        run.stopped_early,
        opus_comparison_text(run)
    )
}

/// 一段历史：几份、本机那一份在导入之前先跑没有、第二台起步时拿不拿本机库的快照、本机先判着第二台那一份（双机驱动「本机先领整份判着」那一段判到的、归第二台的块）没有、
/// 导入那一趟跑几次、发布那个点 ignore 没有、核不核。
#[derive(Clone, Copy, Debug)]
struct OpusHistory {
    shares: u64,
    this_share_runs_first: bool,
    peers_start_from_a_snapshot: bool,
    warm_takes_the_peer_share: bool,
    import_runs: usize,
    ignore_the_publish: bool,
    verify: bool,
}

fn opus_run_history(name: &str, history: OpusHistory) {
    let flow_name = format!("opus-{name}");
    let recorded = record_new_pool_file_creation(&flow_name, history.ignore_the_publish, file_content());
    let observe_by_counting =
        |_image: &CrashImage<'_>, _report: &RecoveryReport, counts: &mut Layer0ObserverCounts| {
            counts.add(STATES_SEEN_BY_THE_IMPORT_PROBE_COUNTER, 1);
        };
    let observers = [ObserverOnTheNodes {
        observer_name: ObserverName::new("import_probe").expect("观察者的名字非空、不长"),
        observer_code_file: THIS_FILE.to_string(),
        extra_inputs_digest: [0; DIGEST_BYTES],
        nodes: vec![NodeOfAnObserver {
            setting_file: THIS_FILE.to_string(),
            entry_name: "publish_first_file".to_string(),
        }],
        observe: &observe_by_counting,
    }];
    let plan = plan_crash_points(&recorded.flow());
    let plan_paths: Vec<String> = plan.points.iter().map(|point| point.path_number.clone()).collect();
    let planned_states: u64 = plan.points.iter().map(|point| point.ordinals.end - point.ordinals.start).sum();
    println!(
        "OPUS_P4 history={name} {history:?} blocks={} planned_states={planned_states} closed_form_full={}",
        total_blocks(&plan),
        singlefs_harness::memory_pool::closed_form_state_count(&recorded.segments)
    );
    let directory = EnvironmentRunDirectory::new(&flow_name);
    let root = directory.temporary.directory.clone();
    let n = history.shares;
    let one_host = root.join("one-host");
    let this_library = root.join("this-share");
    let peers: Vec<PathBuf> = (1..n).map(|index| root.join(format!("peer-{index}"))).collect();
    let amplify_once = |library: &std::path::Path, share: &str, import: Option<&str>, stop: Option<&std::path::Path>| {
        let _environment = opus_environment(&directory, library, share, None, import, stop, history.verify);
        amplify_with_observers(&recorded, &flow_name, &observers)
    };
    let run = amplify_once(&one_host, "0/1", None, None);
    println!("OPUS_P4 history={name} step=one-host {}", opus_run_text(&run));
    if history.warm_takes_the_peer_share {
        let run = amplify_once(&this_library, &format!("1/{n}"), None, None);
        println!("OPUS_P4 history={name} step=warm {}", opus_run_text(&run));
    }
    for (index, peer) in peers.iter().enumerate() {
        if history.peers_start_from_a_snapshot && this_library.join("CURRENT").exists() {
            opus_copy_library(&this_library, peer);
        }
        let run = amplify_once(peer, &format!("{}/{n}", index + 1), None, None);
        println!("OPUS_P4 history={name} step=peer-{} {}", index + 1, opus_run_text(&run));
    }
    let this_share = format!("0/{n}");
    if history.this_share_runs_first {
        let run = amplify_once(&this_library, &this_share, None, None);
        println!("OPUS_P4 history={name} step=this-share {}", opus_run_text(&run));
    }
    let planned_library = root.join("planned");
    let whole_library = root.join("whole");
    if this_library.join("CURRENT").exists() {
        opus_copy_library(&this_library, &planned_library);
        opus_copy_library(&this_library, &whole_library);
    }
    let import_list = peers
        .iter()
        .map(|peer| peer.display().to_string())
        .collect::<Vec<_>>()
        .join(":");
    for attempt in 0..history.import_runs {
        let run = amplify_once(&planned_library, &this_share, Some(&import_list), None);
        println!("OPUS_P4 history={name} step=planned-import-{attempt} {}", opus_run_text(&run));
    }
    {
        let mut whole = if whole_library.join("CURRENT").exists() {
            VerdictStore::open_existing(&whole_library).expect("开得了整库导入那一份").0
        } else {
            VerdictStore::create_empty(&whole_library).expect("建得了整库导入那一份")
        };
        for peer in &peers {
            let (other, _) = VerdictStore::open_existing(peer).expect("开得了第二台的库");
            whole.import_from(&other).expect("整库导入");
        }
    }
    let run = amplify_once(&whole_library, &this_share, None, None);
    println!("OPUS_P4 history={name} step=after-whole-import {}", opus_run_text(&run));
    let (one_host_store, _) = VerdictStore::open_existing(&one_host).expect("开得了一台判完的库");
    let keys = one_host_store.stored_conclusion_keys().expect("列得出结论");
    let one_host_lines = opus_readback(&one_host_store, &keys, &plan_paths);
    drop(one_host_store);
    let read = |library: &std::path::Path| {
        let (store, _) = VerdictStore::open_existing(library).expect("开得了库");
        opus_readback(&store, &keys, &plan_paths)
    };
    let planned_lines = read(&planned_library);
    let whole_lines = read(&whole_library);
    let differing = |left: &[String], right: &[String]| -> Vec<(String, String)> {
        left.iter()
            .zip(right)
            .filter(|(left, right)| left != right)
            .map(|(left, right)| (left.clone(), right.clone()))
            .collect()
    };
    let planned_vs_whole = differing(&planned_lines, &whole_lines);
    println!(
        "OPUS_P4 history={name} readback_lines={} planned_vs_whole={} planned_vs_one_host={} whole_vs_one_host={}",
        one_host_lines.len(),
        planned_vs_whole.len(),
        differing(&planned_lines, &one_host_lines).len(),
        differing(&whole_lines, &one_host_lines).len()
    );
    for (planned, whole) in planned_vs_whole.iter().take(4) {
        println!("OPUS_P4 history={name} differs planned: {planned}");
        println!("OPUS_P4 history={name} differs whole:   {whole}");
    }
    drop(directory);
}

const OPUS_BASE_HISTORY: OpusHistory = OpusHistory {
    shares: 2,
    this_share_runs_first: true,
    peers_start_from_a_snapshot: false,
    warm_takes_the_peer_share: false,
    import_runs: 1,
    ignore_the_publish: false,
    verify: false,
};

#[test]
fn opus_h01_batch_shape_without_verifying() {
    opus_run_history("h01", OPUS_BASE_HISTORY);
}

#[test]
fn opus_h02_batch_shape_verifying() {
    opus_run_history("h02", OpusHistory { verify: true, ..OPUS_BASE_HISTORY });
}

#[test]
fn opus_h03_driver_shape_warm_then_snapshot_verifying() {
    opus_run_history(
        "h03",
        OpusHistory {
            peers_start_from_a_snapshot: true,
            warm_takes_the_peer_share: true,
            verify: true,
            ..OPUS_BASE_HISTORY
        },
    );
}

#[test]
fn opus_h04_import_before_this_share_ever_ran_verifying() {
    opus_run_history("h04", OpusHistory { this_share_runs_first: false, verify: true, ..OPUS_BASE_HISTORY });
}

#[test]
fn opus_h05_three_hosts_driver_shape_verifying() {
    opus_run_history(
        "h05",
        OpusHistory {
            shares: 3,
            peers_start_from_a_snapshot: true,
            warm_takes_the_peer_share: true,
            verify: true,
            ..OPUS_BASE_HISTORY
        },
    );
}

#[test]
fn opus_h06_ignored_publish_snapshot_verifying() {
    opus_run_history(
        "h06",
        OpusHistory { ignore_the_publish: true, peers_start_from_a_snapshot: true, verify: true, ..OPUS_BASE_HISTORY },
    );
}

#[test]
fn opus_h07_import_twice_snapshot_verifying() {
    opus_run_history(
        "h07",
        OpusHistory { import_runs: 2, peers_start_from_a_snapshot: true, verify: true, ..OPUS_BASE_HISTORY },
    );
}

#[test]
fn opus_h08_three_hosts_import_first_twice_verifying() {
    opus_run_history(
        "h08",
        OpusHistory { shares: 3, this_share_runs_first: false, import_runs: 2, verify: true, ..OPUS_BASE_HISTORY },
    );
}
