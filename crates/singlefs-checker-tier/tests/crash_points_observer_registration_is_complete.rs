//! checker 档模块：crash、crash_identity、crash_library、crash_amplification
//! 观察者按流的来历统一挂的登记完整性（G4 第三轮判决丙-置，用户 2026-10-22 定「按流登记+相对基镜像」；主 agent 对 G4 实现员五问的定案）。
//! 来历表在 `singlefs_checker_tier::crash_point_replay_provenance::PROVENANCE_OF_THE_FLOWS`，跑流的两处（崩溃枚举用例的枚举入口、崩溃放量）按它挂。
//! 这里判两件：
//! - 每一个崩溃点重放项与崩溃放量分项在来历表里有行、表里点名的项都还登着：项名现读两处登记位——崩溃点重放合表
//!   `.claude/gate.d/crash-point-replay-items.tsv` 的第一列（合表第二列的崩溃枚举用例经 `.claude/gate.d/stage-inputs.tsv` 的 `test=` 对到测试目标，
//!   测试目标里调了 `enumerate_crash_states` 一族的那几项，崩溃枚举那一处跑它），与崩溃放量入口 `scripts/crash/crash-amplification.sh` 的 ITEMS
//!   第一列；（跑的那一处，流名）恰好一行。
//! - 来历是 mkfs 返回之后的那几条流都挂了两槽全废盘面观察者（判段从判器交回的逐盘自证槽数看的那一种，`ObserverCode::FromTheSelfVerifiedSystemConfigurationSlotsTheJudgeHandsBack`），
//!   mkfs 自己的流没挂（合表说明列写了「mkfs 自己的流」的那几项现读）；两处都不登的一个都不挂；并集对走读观察者撤了（主 agent 2026-10-22 定），
//!   哪条流都不挂。挂法经跑流那两处调的同一个函数（`observers_on_the_nodes_with_the_ones_attached_to_the_flow`）判。
//!
//! 测试里现拼的项名（`<流>::quick_tier::<标签>`、`stream.flow_name()`）登记位上没有，它们的完整由跑流的那一处兜：来历表里查不到就在录任何东西
//! 之前拒，这一条用例不扫测试源码里的项名。
//! 判别力在 `crates/mutations.tsv`（名字以「来历表」起头的几行）：来历表漏一行、mkfs 返回之后的流不挂两槽全废观察者、mkfs 自己的流也挂，各让这里一条红。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use singlefs_checker_tier::crash_amplification::{
    NodeOfAnObserver, ObserverCode, ObserverOnTheNodes,
};
use singlefs_checker_tier::crash_identity::{judging_units_key, JudgingUnits, ObserverName};
use singlefs_checker_tier::crash_library::toolchain_of_this_process;
use singlefs_checker_tier::crash_point_replay_provenance::{
    flow_name_of_the_name_at_the_run, observers_attached_by_the_provenance,
    observers_attached_to_the_flow, observers_on_the_nodes_with_the_ones_attached_to_the_flow,
    provenance_row_of_the_flow, FlowProvenance, ObserverAttachedByTheProvenance, ProvenanceRefusal,
    RegistrationOfTheFlow, RunOfTheFlow, PROVENANCE_OF_THE_FLOWS, THIS_FILE,
};
use singlefs_checker_tier::judging_units_entries::OBSERVER_ENTRIES;
use singlefs_harness::sha256::DIGEST_BYTES;

/// 仓根：这个包在 `crates/singlefs-checker-tier`。
fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("checker 档包在仓根下的 crates/ 里")
        .to_path_buf()
}

fn read_repository_file(relative: &str) -> String {
    let path = repository_root().join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("登记位 {} 读不出：{error}", path.display()))
}

/// 崩溃点重放合表的一项：项名、第二列点名的崩溃枚举用例键、说明列。
struct ItemOfTheCrashPointReplayTable {
    item_name: String,
    crash_case_keys: Vec<String>,
    explanation: String,
}

/// 现读崩溃点重放合表（`.claude/gate.d/crash-point-replay-items.tsv`：`#` 起头的是注释，一行一项、制表符分隔，第五段是 `# 说明`）。
fn items_of_the_crash_point_replay_table() -> Vec<ItemOfTheCrashPointReplayTable> {
    read_repository_file(".claude/gate.d/crash-point-replay-items.tsv")
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert!(
                fields.len() >= 4,
                "合表一行至少四列（项名、崩溃枚举用例、放量分项、轻型取自哪一项）：{line}"
            );
            ItemOfTheCrashPointReplayTable {
                item_name: fields[0].to_string(),
                crash_case_keys: fields[1]
                    .split(' ')
                    .filter(|key| !key.is_empty() && *key != "-")
                    .map(str::to_string)
                    .collect(),
                explanation: fields[4..].join("\t"),
            }
        })
        .collect()
}

/// 现读 `.claude/gate.d/stage-inputs.tsv` 里一条崩溃枚举用例（`crash-case:<名>`）的测试目标：第三列 `test=singlefs-checker-tier:<测试目标>:<用例函数>`。
fn test_target_of_the_crash_case(stage_inputs: &str, crash_case_key: &str) -> String {
    let line = stage_inputs
        .lines()
        .find(|line| line.split('\t').next() == Some(crash_case_key))
        .unwrap_or_else(|| panic!("合表点名的 {crash_case_key} 在 stage-inputs.tsv 里没有那一行"));
    let test_field = line
        .split('\t')
        .nth(2)
        .and_then(|fields| {
            fields
                .split(' ')
                .find_map(|field| field.strip_prefix("test=singlefs-checker-tier:"))
        })
        .unwrap_or_else(|| {
            panic!("{crash_case_key} 那一行第三列没有 test=singlefs-checker-tier:…：{line}")
        });
    test_field
        .split(':')
        .next()
        .expect("test= 的测试目标那一截")
        .to_string()
}

/// 现读崩溃放量入口 `scripts/crash/crash-amplification.sh` 的 ITEMS（`ITEMS=(` 到 `)` 之间一行一项 `$'<分项名>\t<测试目标>\t<用例函数>\t<档位>'`）：分项名。
fn items_of_the_crash_amplification_entry() -> Vec<String> {
    let entry = read_repository_file("scripts/crash/crash-amplification.sh");
    let mut lines = entry.lines();
    lines
        .by_ref()
        .find(|line| line.trim_start() == "ITEMS=(")
        .expect("crash-amplification.sh 里有 ITEMS=( 那一行");
    let items: Vec<String> = lines
        .take_while(|line| line.trim() != ")")
        .map(|line| {
            let row = line
                .trim()
                .strip_prefix("$'")
                .unwrap_or_else(|| panic!("ITEMS 里一行写成 $'…'：{line}"));
            row.split("\\t")
                .next()
                .expect("ITEMS 一行的分项名那一截")
                .to_string()
        })
        .collect();
    assert!(!items.is_empty(), "ITEMS 里至少一项");
    items
}

/// 合表一项的崩溃枚举用例的测试目标里有没有调崩溃枚举的入口（`enumerate_crash_states` 一族）：有才是崩溃枚举那一处跑的流，要在来历表里登。
fn crash_cases_of_the_item_run_the_enumeration(
    item: &ItemOfTheCrashPointReplayTable,
    stage_inputs: &str,
) -> bool {
    item.crash_case_keys.iter().any(|crash_case_key| {
        let target = test_target_of_the_crash_case(stage_inputs, crash_case_key);
        read_repository_file(&format!("crates/singlefs-checker-tier/tests/{target}.rs"))
            .contains("enumerate_crash_states")
    })
}

/// 场景：来历表本身。
/// 预期：（跑的那一处，流名）恰好一行；流名里没有 `::`（跑的那一处只取项名第一个 `::` 之前那一截去查，带 `::` 的行永远查不到）。
#[test]
fn every_flow_has_exactly_one_row_in_the_provenance_table() {
    let mut seen = BTreeSet::new();
    for row in PROVENANCE_OF_THE_FLOWS {
        assert!(
            !row.flow_name.contains("::") && !row.flow_name.is_empty(),
            "来历表里的流名不空、不带 `::`：{row:?}"
        );
        assert!(
            seen.insert((format!("{:?}", row.run), row.flow_name)),
            "（跑的那一处，流名）只许一行：{row:?}"
        );
        assert_eq!(
            provenance_row_of_the_flow(row.run, row.flow_name),
            Ok(row),
            "按流名查得到它自己那一行"
        );
    }
    println!(
        "CRASH_POINT_REPLAY_PROVENANCE_ROWS rows={}",
        PROVENANCE_OF_THE_FLOWS.len()
    );
}

/// 场景：现读两处登记位——崩溃点重放合表（第一列项名；第二列的崩溃枚举用例经 stage-inputs.tsv 对到测试目标）与崩溃放量入口的 ITEMS（第一列分项名）。
/// 预期：合表里崩溃枚举那一处跑的每一项（它的崩溃枚举用例的测试目标调了 `enumerate_crash_states` 一族）在来历表里至少一行（崩溃枚举那一处、
/// 登在这一项上）；ITEMS 的每一分项至少一行（崩溃放量那一处、登在这一分项上）；来历表里每一行点名的项、分项都还在登记位上，跑的那一处与登记位对得上。
#[test]
fn every_crash_point_replay_item_and_every_crash_amplification_item_has_a_row_and_every_row_names_a_registered_one(
) {
    let table = items_of_the_crash_point_replay_table();
    let stage_inputs = read_repository_file(".claude/gate.d/stage-inputs.tsv");
    let amplification_items = items_of_the_crash_amplification_entry();
    let rows_registered_as = |wanted: RegistrationOfTheFlow, run: RunOfTheFlow| {
        PROVENANCE_OF_THE_FLOWS
            .iter()
            .filter(|row| row.registration == wanted && row.run == run)
            .count()
    };
    let mut enumerated_items = 0_usize;
    let mut items_without_a_row = Vec::new();
    for item in &table {
        if !crash_cases_of_the_item_run_the_enumeration(item, &stage_inputs) {
            continue;
        }
        enumerated_items += 1;
        let item_name: &str = &item.item_name;
        let rows = PROVENANCE_OF_THE_FLOWS
            .iter()
            .filter(|row| {
                row.run == RunOfTheFlow::CrashPointReplayEnumeration
                    && matches!(
                        row.registration,
                        RegistrationOfTheFlow::CrashPointReplayItemsTable(name) if name == item_name
                    )
            })
            .count();
        if rows == 0 {
            items_without_a_row.push(format!("合表 {item_name}"));
        }
    }
    for amplification_item in &amplification_items {
        let rows = PROVENANCE_OF_THE_FLOWS
            .iter()
            .filter(|row| {
                row.run == RunOfTheFlow::CrashAmplification
                    && matches!(
                        row.registration,
                        RegistrationOfTheFlow::CrashAmplificationItems(name) if name == amplification_item
                    )
            })
            .count();
        if rows == 0 {
            items_without_a_row.push(format!("ITEMS {amplification_item}"));
        }
    }
    assert!(
        items_without_a_row.is_empty(),
        "登记位上的这几项在来历表里没有行（跑它的那一处会在录任何东西之前拒）：{items_without_a_row:?}"
    );
    let mut rows_naming_what_is_not_registered = Vec::new();
    for row in PROVENANCE_OF_THE_FLOWS {
        match (row.run, row.registration) {
            (
                RunOfTheFlow::CrashPointReplayEnumeration,
                RegistrationOfTheFlow::CrashPointReplayItemsTable(item_name),
            ) => {
                if !table.iter().any(|item| item.item_name == item_name) {
                    rows_naming_what_is_not_registered.push(format!("{row:?}"));
                }
            }
            (
                RunOfTheFlow::CrashAmplification,
                RegistrationOfTheFlow::CrashAmplificationItems(item_name),
            ) => {
                if !amplification_items.iter().any(|item| item == item_name) {
                    rows_naming_what_is_not_registered.push(format!("{row:?}"));
                }
            }
            (RunOfTheFlow::CrashPointReplayEnumeration, RegistrationOfTheFlow::NeitherRegistry)
            | (RunOfTheFlow::CrashAmplification, RegistrationOfTheFlow::NeitherRegistry) => {}
            (
                RunOfTheFlow::CrashPointReplayEnumeration,
                RegistrationOfTheFlow::CrashAmplificationItems(_),
            )
            | (
                RunOfTheFlow::CrashAmplification,
                RegistrationOfTheFlow::CrashPointReplayItemsTable(_),
            ) => {
                rows_naming_what_is_not_registered
                    .push(format!("跑的那一处与登记位对不上：{row:?}"));
            }
        }
    }
    assert!(
        rows_naming_what_is_not_registered.is_empty(),
        "来历表里这几行点名的项不在登记位上（多出来的行）：{rows_naming_what_is_not_registered:?}"
    );
    assert!(
        enumerated_items > 0 && !amplification_items.is_empty(),
        "两处登记位都读到了项：合表里崩溃枚举那一处跑的 {enumerated_items} 项、ITEMS {} 项",
        amplification_items.len()
    );
    println!(
        "CRASH_POINT_REPLAY_PROVENANCE_REGISTRIES table_items={} enumerated_table_items={enumerated_items} amplification_items={} rows_on_the_table={} rows_on_the_amplification_items={} rows_in_neither={}",
        table.len(),
        amplification_items.len(),
        PROVENANCE_OF_THE_FLOWS
            .iter()
            .filter(|row| matches!(row.registration, RegistrationOfTheFlow::CrashPointReplayItemsTable(_)))
            .count(),
        PROVENANCE_OF_THE_FLOWS
            .iter()
            .filter(|row| matches!(row.registration, RegistrationOfTheFlow::CrashAmplificationItems(_)))
            .count(),
        rows_registered_as(
            RegistrationOfTheFlow::NeitherRegistry,
            RunOfTheFlow::CrashPointReplayEnumeration
        ) + rows_registered_as(
            RegistrationOfTheFlow::NeitherRegistry,
            RunOfTheFlow::CrashAmplification
        )
    );
}

/// 跑流那两处调的那一步（[`observers_on_the_nodes_with_the_ones_attached_to_the_flow`]）给这条流挂上的按来历挂的观察者名（调用方不另挂）。
fn observer_names_attached_at_the_run(run: RunOfTheFlow, name_at_the_run: &str) -> Vec<String> {
    let nodes = [NodeOfAnObserver {
        setting_file:
            "crates/singlefs-checker-tier/tests/crash_points_observer_registration_is_complete.rs"
                .to_string(),
        entry_name: name_at_the_run.to_string(),
    }];
    let observers = observers_on_the_nodes_with_the_ones_attached_to_the_flow(
        run,
        name_at_the_run,
        &[],
        &nodes,
    )
    .unwrap_or_else(|refusal| panic!("{}", refusal.sentence()));
    for observer in &observers {
        assert_eq!(
            (
                observer.observer_code_file.as_str(),
                observer.nodes.as_slice()
            ),
            (THIS_FILE, nodes.as_slice()),
            "按来历挂的观察者的代码文件是来历表那一份、挂在跑的那一处交的节点上"
        );
        assert!(
            !observer.observe.is_recorded_in_the_recording_stage(),
            "按来历挂的观察者在判段从判器交回的逐盘自证槽数看，录入段对它什么都不跑：{}",
            observer.observer_name.as_str()
        );
    }
    observers
        .iter()
        .map(|observer| observer.observer_name.as_str().to_string())
        .collect()
}

/// 场景：来历表每一行，经跑流那两处调的同一步挂观察者（崩溃枚举那一处另用带档位标签的项名 `<流>::quick_tier::<标签>` 查一遍）。
/// 预期：登在登记位上、来历是 mkfs 返回之后的流挂两槽全废盘面观察者（录入段不录、判段看的那一种）；登在登记位上、录 mkfs 自己的（与 mkfs 中途崩溃之后的挂载）
/// 一个都不挂；两处都不登的一个都不挂。合表说明列写了「mkfs 自己的流」的项（现读）在来历表里是录 mkfs 自己的；两种来历都有行（两条分支都判到）。
#[test]
fn flows_recorded_after_the_make_filesystem_returned_carry_the_two_slot_observer_and_the_make_filesystem_flows_do_not(
) {
    let two_slot =
        ObserverAttachedByTheProvenance::DevicesWithoutASelfVerifiedSystemConfigurationSlot
            .observer_name();
    let mut registered_after_the_make_filesystem = 0_usize;
    let mut registered_make_filesystem_itself = 0_usize;
    for row in PROVENANCE_OF_THE_FLOWS {
        let mut names = vec![row.flow_name.to_string()];
        if row.run == RunOfTheFlow::CrashPointReplayEnumeration {
            names.push(format!("{}::quick_tier::probe", row.flow_name));
        }
        for name_at_the_run in names {
            let attached = observer_names_attached_at_the_run(row.run, &name_at_the_run);
            let expected: Vec<&str> = match (row.registration, row.provenance) {
                (RegistrationOfTheFlow::NeitherRegistry, _) => Vec::new(),
                (
                    RegistrationOfTheFlow::CrashPointReplayItemsTable(_)
                    | RegistrationOfTheFlow::CrashAmplificationItems(_),
                    FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
                ) => vec![two_slot],
                (
                    RegistrationOfTheFlow::CrashPointReplayItemsTable(_)
                    | RegistrationOfTheFlow::CrashAmplificationItems(_),
                    FlowProvenance::RecordingTheMakeFilesystemItself
                    | FlowProvenance::MountAfterACrashInTheMiddleOfTheMakeFilesystem,
                ) => Vec::new(),
            };
            assert_eq!(
                attached, expected,
                "{:?} 的流 {name_at_the_run}（{:?}，{:?}）按来历挂的观察者",
                row.run, row.registration, row.provenance
            );
        }
        match (row.registration, row.provenance) {
            (RegistrationOfTheFlow::NeitherRegistry, _) => {}
            (_, FlowProvenance::RecordedAfterTheMakeFilesystemReturned) => {
                registered_after_the_make_filesystem += 1;
            }
            (_, FlowProvenance::RecordingTheMakeFilesystemItself) => {
                registered_make_filesystem_itself += 1;
            }
            (_, FlowProvenance::MountAfterACrashInTheMiddleOfTheMakeFilesystem) => {}
        }
    }
    let make_filesystem_items: Vec<String> = items_of_the_crash_point_replay_table()
        .into_iter()
        .filter(|item| item.explanation.contains("mkfs 自己的流"))
        .map(|item| item.item_name)
        .collect();
    assert!(
        !make_filesystem_items.is_empty(),
        "合表说明列里至少一项写着「mkfs 自己的流」"
    );
    for item_name in &make_filesystem_items {
        for row in PROVENANCE_OF_THE_FLOWS.iter().filter(|row| {
            matches!(
                row.registration,
                RegistrationOfTheFlow::CrashPointReplayItemsTable(name) if name == item_name
            )
        }) {
            assert_eq!(
                row.provenance,
                FlowProvenance::RecordingTheMakeFilesystemItself,
                "合表说明 {item_name} 是 mkfs 自己的流：来历表里它那一行 {row:?}"
            );
        }
    }
    assert!(
        registered_after_the_make_filesystem > 0 && registered_make_filesystem_itself > 0,
        "两种来历都有登在登记位上的行：mkfs 返回之后 {registered_after_the_make_filesystem}、mkfs 自己 {registered_make_filesystem_itself}"
    );
    println!(
        "CRASH_POINT_REPLAY_PROVENANCE_ATTACHED rows={} registered_after_the_make_filesystem={registered_after_the_make_filesystem} registered_make_filesystem_itself={registered_make_filesystem_itself} make_filesystem_items_in_the_table={}",
        PROVENANCE_OF_THE_FLOWS.len(),
        make_filesystem_items.len()
    );
}

/// 场景：按来历挂的观察者在观察者登记表 `OBSERVER_ENTRIES` 里的那一行，与它那一组判法单元的键；来历表那一份文件在登记表里的行数。
/// 预期：按来历挂的只剩两槽全废盘面观察者一个（并集对走读观察者撤了），它恰好一行（观察者代码文件是来历表那一份、目标 `lib`、入口是它那个 fn），
/// 来历表那一份在登记表里也只有这一行（撤掉的那个观察者的行跟着删了），这一份文件真在仓里，键取得出（没登、写错入口都会让跑流那两处取键拒算、
/// 照判不入库）。
#[test]
fn the_observers_attached_by_the_provenance_have_their_entries_registered_and_their_keys_compute() {
    assert!(
        repository_root().join(THIS_FILE).is_file(),
        "来历表那一份的仓内路径写对了：{THIS_FILE}"
    );
    let every_attached: BTreeSet<&str> = [
        RegistrationOfTheFlow::CrashPointReplayItemsTable("probe"),
        RegistrationOfTheFlow::CrashAmplificationItems("probe"),
        RegistrationOfTheFlow::NeitherRegistry,
    ]
    .into_iter()
    .flat_map(|registration| {
        [
            FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
            FlowProvenance::RecordingTheMakeFilesystemItself,
            FlowProvenance::MountAfterACrashInTheMiddleOfTheMakeFilesystem,
        ]
        .into_iter()
        .flat_map(move |provenance| observers_attached_by_the_provenance(registration, provenance))
    })
    .map(ObserverAttachedByTheProvenance::observer_name)
    .collect();
    assert_eq!(
        every_attached.len(),
        1,
        "按来历挂的是一个观察者（两槽全废盘面观察者）：{every_attached:?}"
    );
    assert_eq!(
        OBSERVER_ENTRIES
            .iter()
            .filter(|entry| entry.observer_code_file == THIS_FILE)
            .count(),
        every_attached.len(),
        "来历表那一份在观察者登记表里只登按来历挂的那几个"
    );
    for observer_name in &every_attached {
        let entries: Vec<_> = OBSERVER_ENTRIES
            .iter()
            .filter(|entry| {
                entry.observer_code_file == THIS_FILE && entry.observer_name == *observer_name
            })
            .collect();
        assert_eq!(
            entries.len(),
            1,
            "{observer_name} 在 OBSERVER_ENTRIES 里恰好一行"
        );
        assert_eq!(entries[0].target, "lib", "{observer_name} 编在库里");
        assert_eq!(
            entries[0].entry_functions.len(),
            1,
            "{observer_name} 的入口就是它那一个 fn"
        );
        judging_units_key(
            &JudgingUnits::ObserverCode {
                observer_code_file: THIS_FILE.to_string(),
                observer_name: (*observer_name).to_string(),
            },
            toolchain_of_this_process(),
        )
        .unwrap_or_else(|refusal| {
            panic!(
                "{observer_name} 那一组判法单元的键取不出：{}",
                refusal.sentence()
            )
        });
    }
}

/// 场景：跑流的那一处拿一个来历表里没有的名字去查、拿一个带档位标签的项名去查、调用方挂了一个与按来历挂的同名的观察者。
/// 预期：没有的名字拒（点名跑的那一处与名字）；带标签的项名查到它那条流的行；同名的拒——都在录任何东西之前。
#[test]
fn a_flow_without_a_row_is_refused_a_labelled_item_finds_its_flow_and_a_clashing_observer_is_refused(
) {
    assert_eq!(
        observers_attached_to_the_flow(
            RunOfTheFlow::CrashPointReplayEnumeration,
            "a_flow_nobody_registered::quick_tier::probe"
        ),
        Err(ProvenanceRefusal::NoRowForTheFlow {
            run: RunOfTheFlow::CrashPointReplayEnumeration,
            name_at_the_run: "a_flow_nobody_registered::quick_tier::probe".to_string(),
        }),
        "来历表里没有的流拒"
    );
    assert_eq!(
        flow_name_of_the_name_at_the_run("truncate_to_zero::quick_tier::fast"),
        "truncate_to_zero",
        "项名第一个 :: 之前那一截是流名"
    );
    let observe_nothing =
        |_ordinal: u64,
         _image: &singlefs_harness::memory_pool::CrashImage<'_>,
         _report: &singlefs_core::recovery::RecoveryReport,
         _counts: &mut singlefs_checker_tier::crash::CrashPointReplayObserverCounts| {};
    let clashing = [ObserverOnTheNodes {
        observer_name: ObserverName::new(
            ObserverAttachedByTheProvenance::DevicesWithoutASelfVerifiedSystemConfigurationSlot
                .observer_name(),
        )
        .expect("名字非空、不长"),
        observer_code_file:
            "crates/singlefs-checker-tier/tests/crash_points_observer_registration_is_complete.rs"
                .to_string(),
        extra_inputs_digest: [0; DIGEST_BYTES],
        nodes: Vec::new(),
        observe: ObserverCode::CountsOnly(&observe_nothing),
    }];
    let refusal = observers_on_the_nodes_with_the_ones_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        "truncate_to_zero::quick_tier::fast",
        &clashing,
        &[],
    )
    .err()
    .expect("调用方手挂了与按来历挂的同名的观察者：拒");
    assert_eq!(
        refusal,
        ProvenanceRefusal::ObserverNamedLikeOneAttachedByTheProvenance {
            observer_name: "devices_without_a_self_verified_system_configuration_slot".to_string(),
        }
    );
}

/// 场景：按来历挂的两槽全废盘面观察者挂上去的那一份代码（跑流那两处交给判段的那一个），喂判段 CPU 那一路交回的逐盘自证槽数形状的输入：
/// 两块盘，基镜像上都两槽在；一个状态盘 0 一个槽都不在，一个状态两块盘都还有一槽。
/// 预期：盘 0 新出现死盘的那一份判红（panic），另一份不红；两份都记一个看过的状态。挂上去的是判段那一种的代码，不是录入段的。
/// 判别力：`crates/mutations.tsv` 里「来历表：mkfs 返回之后的流不挂两槽全废盘面观察者」那一行（一个都不挂）与「两槽全废盘面观察者不判」那一行
/// （新出现的死盘只打一行）各让这一条红。
#[test]
fn the_observer_attached_by_the_provenance_judges_the_slot_counts_the_judge_hands_back() {
    let observers = observers_on_the_nodes_with_the_ones_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        "truncate_to_zero::quick_tier::probe",
        &[],
        &[],
    )
    .unwrap_or_else(|refusal| panic!("{}", refusal.sentence()));
    let [attached] = observers.as_slice() else {
        panic!(
            "mkfs 返回之后、登在合表上的流按来历挂一个观察者：挂了 {}",
            observers.len()
        );
    };
    let ObserverCode::FromTheSelfVerifiedSystemConfigurationSlotsTheJudgeHandsBack(observe) =
        attached.observe
    else {
        panic!("按来历挂的观察者是判段从判器交回的逐盘自证槽数看的那一种");
    };
    let slots_on_the_base_image = [(0, 2), (1, 2)];
    let observed = |slots_on_the_crash_image: &[(u32, u32)]| {
        let mut counts = singlefs_checker_tier::crash::CrashPointReplayObserverCounts::default();
        let reddened = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            observe(
                slots_on_the_crash_image,
                &slots_on_the_base_image,
                &mut counts,
            );
        }))
        .is_err();
        (
            reddened,
            counts.count_named(
                ObserverAttachedByTheProvenance::DevicesWithoutASelfVerifiedSystemConfigurationSlot
                    .states_seen_counter(),
            ),
        )
    };
    assert_eq!(
        observed(&[(0, 0), (1, 2)]),
        (true, 1),
        "盘 0 在这个状态上一个槽都不在、基镜像上两槽都在：判红，判红之前记一个看过的状态"
    );
    assert_eq!(
        observed(&[(0, 1), (1, 1)]),
        (false, 1),
        "两块盘都还有一槽：不红，记一个看过的状态"
    );
}
