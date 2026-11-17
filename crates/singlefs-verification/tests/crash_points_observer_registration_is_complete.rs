//! 验证包模块：crash、crash_state_enumeration、crash_identity、crash_library、crash_amplification
//! 观察者按流的来历统一挂的登记完整性（G4 丙-置）。
//! 来历表在 `singlefs_verification::crash_point_replay_provenance::PROVENANCE_OF_THE_FLOWS`，跑流的两处（崩溃枚举用例的枚举入口、崩溃放量）按它挂。
//! 这里判四件：
//! - 每一个崩溃点重放项与崩溃放量分项在来历表里有行、表里点名的项都还登着：项名现读两处登记位——崩溃点重放合表
//!   `.claude/gate.d/crash-point-replay-items.tsv` 的第一列（合表第二列的崩溃枚举用例经 `.claude/gate.d/stage-inputs.tsv` 的 `test=` 对到测试目标，
//!   测试目标里调了 `enumerate_crash_states` 一族的那几项，崩溃枚举那一处跑它），与崩溃放量入口 `scripts/crash/crash-amplification.sh` 的 ITEMS
//!   第一列；（跑的那一处，流名）恰好一行。
//! - 来历是 mkfs 返回之后的那几条流都挂了两槽全废盘面观察者（判段从判器交回的逐盘自证槽数看的那一种，`ObserverCode::FromTheSelfVerifiedSystemConfigurationSlotsTheJudgeHandsBack`），
//!   mkfs 自己的流没挂（合表说明列写了「mkfs 自己的流」的那几项现读）；两处都不登的一个都不挂；并集对走读观察者撤了，
//!   哪条流都不挂。挂法经跑流那两处调的同一个函数（`observers_on_the_nodes_with_the_ones_attached_to_the_flow`）判。
//! - 观察者登记表 `singlefs_verification::judging_units_entries::OBSERVER_ENTRIES` 的每一行都还有人挂：观察者代码文件在仓里，观察者名写在那份文件
//!   或那个目标的源码里（现读源码）。
//! - 反过来，崩溃枚举用例自己挂的每个观察者（观察者代码文件，观察者名）在观察者登记表里都有一行：`tests/` 下每份测试文件现读，
//!   认法照 `crash_state_enumeration` 的 `observers_on_the_nodes_of_the_item`（用例文件或挂法点名的共用文件）。
//!
//! 测试里现拼的项名（`<流>::in_place_subsets::<标签>`、`<流>::segments_shorter_than_ten::<标签>`、`stream.flow_name()`）登记位上没有，它们的完整由跑流的那一处兜：来历表里查不到就在录任何东西
//! 之前拒，这一条用例不扫测试源码里的项名。
//! 判别力在 `crates/mutations.tsv`（名字以「来历表」起头的几行）：来历表漏一行、mkfs 返回之后的流不挂两槽全废观察者、mkfs 自己的流也挂，各让这里一条红。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use singlefs_verification::crash_amplification::{
    NodeOfAnObserver, ObserverCode, ObserverOnTheNodes,
};
use singlefs_verification::crash_identity::{judging_units_key, JudgingUnits, ObserverName};
use singlefs_verification::crash_library::toolchain_of_this_process;
use singlefs_verification::crash_point_replay_provenance::{
    flow_name_of_the_name_at_the_run, observers_attached_by_the_provenance,
    observers_attached_to_the_flow, observers_on_the_nodes_with_the_ones_attached_to_the_flow,
    provenance_row_of_the_flow, FlowProvenance, ObserverAttachedByTheProvenance, ProvenanceRefusal,
    RegistrationOfTheFlow, RunOfTheFlow, PROVENANCE_OF_THE_FLOWS, THIS_FILE,
};
use singlefs_verification::judging_units_entries::{ObserverEntry, OBSERVER_ENTRIES};
use singlefs_verification::sha256::DIGEST_BYTES;

/// 仓根：这个包在 `crates/singlefs-verification`。
fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("验证包在仓根下的 crates/ 里")
        .to_path_buf()
}

/// 现读一处登记位。路径在调用处写成清单目录接一段字面路径（判法单元的键把读到的文件记成数据文件），这里只读。
fn read_registry_file(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("登记位 {} 读不出：{error}", path.display()))
}

/// 这个包的测试目录 `crates/singlefs-verification/tests/`：清单目录接一段字面路径，判法单元的键把它记成数据目录（目录下每份文件的字节都进键）。
fn test_directory_of_this_package() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
}

/// 崩溃点重放合表的一项：项名、第二列点名的崩溃枚举用例键、说明列。
struct ItemOfTheCrashPointReplayTable {
    item_name: String,
    crash_case_keys: Vec<String>,
    explanation: String,
}

/// 现读崩溃点重放合表（`.claude/gate.d/crash-point-replay-items.tsv`：`#` 起头的是注释，一行一项、制表符分隔三列，再接一段 `# 说明`）。
fn items_of_the_crash_point_replay_table() -> Vec<ItemOfTheCrashPointReplayTable> {
    let table = read_registry_file(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.claude/gate.d/crash-point-replay-items.tsv"),
    );
    table
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert!(
                fields.len() >= 3,
                "合表一行至少三列（项名、崩溃枚举用例、放量分项）：{line}"
            );
            ItemOfTheCrashPointReplayTable {
                item_name: fields[0].to_string(),
                crash_case_keys: fields[1]
                    .split(' ')
                    .filter(|key| !key.is_empty() && *key != "-")
                    .map(str::to_string)
                    .collect(),
                explanation: fields[3..].join("\t"),
            }
        })
        .collect()
}

/// 现读 `.claude/gate.d/stage-inputs.tsv` 里一条崩溃枚举用例（`crash-case:<名>`）的测试目标：第三列 `test=singlefs-verification:<测试目标>:<用例函数>`。
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
                .find_map(|field| field.strip_prefix("test=singlefs-verification:"))
        })
        .unwrap_or_else(|| {
            panic!("{crash_case_key} 那一行第三列没有 test=singlefs-verification:…：{line}")
        });
    test_field
        .split(':')
        .next()
        .expect("test= 的测试目标那一截")
        .to_string()
}

/// 现读崩溃放量入口 `scripts/crash/crash-amplification.sh` 的 ITEMS（`ITEMS=(` 到 `)` 之间一行一项 `$'<分项名>\t<测试目标>\t<用例函数>\t<档位>'`）：分项名。
fn items_of_the_crash_amplification_entry() -> Vec<String> {
    let entry = read_registry_file(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/crash/crash-amplification.sh"),
    );
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
        read_registry_file(&test_directory_of_this_package().join(format!("{target}.rs")))
            .contains("enumerate_crash_states")
    })
}

/// 这个包的源码目录 `crates/singlefs-verification/src/`：清单目录接一段字面路径，判法单元的键把它记成数据目录。
fn source_directory_of_this_package() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// 观察者登记表里一份观察者代码文件（仓内路径）在这个包的测试目录或源码目录下的那一份；不在这个包的这两个目录下的交 `None`。
fn file_of_this_package(path_in_the_repository: &str) -> Option<PathBuf> {
    let path_in_the_package =
        path_in_the_repository.strip_prefix("crates/singlefs-verification/")?;
    if let Some(path_in_the_tests) = path_in_the_package.strip_prefix("tests/") {
        return Some(test_directory_of_this_package().join(path_in_the_tests));
    }
    path_in_the_package
        .strip_prefix("src/")
        .map(|path_in_the_sources| source_directory_of_this_package().join(path_in_the_sources))
}

/// 一段源码去掉每行 `//` 起的注释（文档注释里提到观察者名不算挂了它）。
fn without_line_comments(source: &str) -> String {
    source
        .lines()
        .map(|line| {
            line.find("//")
                .map_or(line, |comment_start| &line[..comment_start])
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

/// 观察者登记表里一个编译目标（`test:<测试目标>`、`bin:<名>`）的主文件与它直接声明的模块文件（`mod <名>;`：带 `#[path = "…"]` 的照那条路径，
/// 相对主文件所在目录；不带的取 `<名>/mod.rs`，没有再取 `<名>.rs`），各自去掉注释。`lib` 交空表：编在库里的观察者，名字写在它的观察者代码文件里。
///
/// # Panics
/// 目标写法不是这三种之一；目标的主文件不在。
fn source_texts_of_the_target(target: &str) -> Vec<String> {
    let main_file = if let Some(test_target) = target.strip_prefix("test:") {
        test_directory_of_this_package().join(format!("{test_target}.rs"))
    } else if let Some(binary_name) = target.strip_prefix("bin:") {
        source_directory_of_this_package()
            .join("bin")
            .join(format!("{binary_name}.rs"))
    } else {
        assert_eq!(
            target, "lib",
            "观察者登记表的目标只有 lib、test:<测试目标>、bin:<名> 三种写法"
        );
        return Vec::new();
    };
    assert!(
        main_file.is_file(),
        "目标 {target} 的主文件 {} 不在",
        main_file.display()
    );
    let main_text = read_registry_file(&main_file);
    let directory_of_the_main_file = main_file.parent().expect("主文件在测试目录或源码目录里");
    let mut module_files = Vec::new();
    let mut path_attribute_of_the_next_module: Option<&str> = None;
    for line in main_text.lines().map(str::trim) {
        if let Some(path) = line
            .strip_prefix("#[path = \"")
            .and_then(|rest| rest.strip_suffix("\"]"))
        {
            path_attribute_of_the_next_module = Some(path);
        } else if let Some(module_name) = line
            .strip_prefix("mod ")
            .or_else(|| line.strip_prefix("pub mod "))
            .and_then(|rest| rest.strip_suffix(';'))
        {
            let module_file = match path_attribute_of_the_next_module.take() {
                Some(path_of_the_module) => directory_of_the_main_file.join(path_of_the_module),
                None => {
                    let module_directory_file =
                        directory_of_the_main_file.join(module_name).join("mod.rs");
                    if module_directory_file.is_file() {
                        module_directory_file
                    } else {
                        directory_of_the_main_file.join(format!("{module_name}.rs"))
                    }
                }
            };
            module_files.push(module_file);
        }
    }
    std::iter::once(main_file)
        .chain(
            module_files
                .into_iter()
                .filter(|module_file| module_file.is_file()),
        )
        .map(|source_file| without_line_comments(&read_registry_file(&source_file)))
        .collect()
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
// crash-case-check:not-a-crash-case 只核崩溃点重放合表、放量入口 ITEMS 与来历表三处登记位对得上，不碰崩溃状态
#[test]
fn every_crash_point_replay_item_and_every_crash_amplification_item_has_a_row_and_every_row_names_a_registered_one(
) {
    let table = items_of_the_crash_point_replay_table();
    let stage_inputs = read_registry_file(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.claude/gate.d/stage-inputs.tsv"),
    );
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
            "crates/singlefs-verification/tests/crash_points_observer_registration_is_complete.rs"
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

/// 场景：来历表每一行，经跑流那两处调的同一步挂观察者（崩溃枚举那一处另用带展开标签的项名 `<流>::in_place_subsets::<标签>` 查一遍）。
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
            names.push(format!("{}::in_place_subsets::probe", row.flow_name));
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

/// 场景：观察者登记表 `OBSERVER_ENTRIES` 的每一行（观察者代码文件、观察者名、编译目标）对着此刻的源码。
/// 预期：观察者代码文件在这个包里、真有这份文件；观察者名作为串字面量写在观察者代码文件或那个目标的源码里（主文件与它直接声明的模块，
/// 去掉注释），也就是还有用例或跑流的那一处拿这个名字挂它。目标与入口 fn 找不找得到由构建脚本管（`build.rs` 逐组收闭包，找不到就停在编译）；
/// 观察者代码文件与观察者名只进组名，构建脚本不核：挂它的用例删了而登记行留着（按计划导入的用例删掉之后留下的那一行），要靠这一条。
/// 判别力（`crates/mutations.tsv`「观察者登记表」起头的两行）：一行的观察者名换成没有用例挂的名字，这条红；一行的观察者代码文件换成仓里没有的路径，这条红。
#[test]
fn every_registered_observer_has_its_observer_code_file_and_is_hung_by_its_name_in_the_code_of_its_target(
) {
    let rows_no_code_hangs: Vec<String> = OBSERVER_ENTRIES
        .iter()
        .filter_map(why_no_code_hangs_the_observer_of_the_row)
        .collect();
    assert!(
        rows_no_code_hangs.is_empty(),
        "观察者登记表里这几行没有人挂（共 {} 行，逐行该删或改）：{rows_no_code_hangs:#?}",
        OBSERVER_ENTRIES.len()
    );
}

/// 观察者登记表的一行为什么没有人挂：观察者代码文件不在这个包的 `tests/`、`src/` 下，或仓里没有这份文件，或观察者名作为串字面量
/// 在观察者代码文件与目标的源码（[`source_texts_of_the_target`]，去掉注释）里都没有；有人挂交 `None`。
fn why_no_code_hangs_the_observer_of_the_row(entry: &ObserverEntry) -> Option<String> {
    let Some(observer_code_file) = file_of_this_package(entry.observer_code_file) else {
        return Some(format!(
            "观察者 {} 的观察者代码文件 {} 不在这个包的 tests/ 或 src/ 下",
            entry.observer_name, entry.observer_code_file
        ));
    };
    if !observer_code_file.is_file() {
        return Some(format!(
            "观察者 {} 的观察者代码文件 {} 不在仓里",
            entry.observer_name, entry.observer_code_file
        ));
    }
    let quoted_observer_name = format!("\"{}\"", entry.observer_name);
    let is_hung_by_its_name = std::iter::once(without_line_comments(&read_registry_file(
        &observer_code_file,
    )))
    .chain(source_texts_of_the_target(entry.target))
    .any(|source| source.contains(&quoted_observer_name));
    if is_hung_by_its_name {
        return None;
    }
    Some(format!(
        "观察者 {}（观察者代码文件 {}，目标 {}）的名字在观察者代码文件与目标的源码里都没有：没有用例挂它",
        entry.observer_name, entry.observer_code_file, entry.target
    ))
}

/// 崩溃枚举用例挂自己的观察者走的三个入口（`singlefs_verification::crash_state_enumeration`）：经它们挂的观察者，观察者代码文件按挂法取，
/// 缺省的挂法是用例文件（`CrashPointReplayObserverCodeFile::TheCaseFile`）。只写名字、不连括号：这份文件自己不算调了它们。
const ENTRIES_HANGING_THE_OBSERVERS_OF_A_CRASH_ENUMERATION_CASE: [&str; 3] = [
    "enumerate_crash_states_with_expansion",
    "enumerate_crash_states_with_every_kind_of_observer",
    "enumerate_crash_states_with_attached_observers",
];

/// 用例自己造的两种观察者（只记计数的、交回镜像的）的类型名：字面量写 `<类型名> {`，造它的 fn 的签名写 `-> <类型名><`。
const TYPES_OF_THE_OBSERVERS_OF_A_CRASH_ENUMERATION_CASE: [&str; 2] = [
    "CrashPointReplayObserver",
    "CrashPointReplayObserverHandingBackAnImage",
];

fn is_an_identifier_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

/// `text` 里 `needle` 每一处的起点，只算前一个字符不是标识符字符的（`AttachedCrashPointReplayObserver {` 里的 `CrashPointReplayObserver {` 不算）。
fn positions_of_the_word(text: &str, needle: &str) -> Vec<usize> {
    text.match_indices(needle)
        .map(|(position, _)| position)
        .filter(|position| {
            !text[..*position]
                .chars()
                .next_back()
                .is_some_and(is_an_identifier_character)
        })
        .collect()
}

/// 几份源码里 `const <名>: &str = "<值>";` 一形的串常量（值可以折到下一行）：名 → 值。
fn string_constants_of_the_sources(sources: &[String]) -> BTreeMap<String, String> {
    let mut constants = BTreeMap::new();
    for source in sources {
        for position in positions_of_the_word(source, "const ") {
            let Some((name, after_the_name)) =
                source[position + "const ".len()..].split_once(": &str =")
            else {
                continue;
            };
            if name.is_empty() || !name.chars().all(is_an_identifier_character) {
                continue;
            }
            let Some((value, _)) = after_the_name
                .trim_start()
                .strip_prefix('"')
                .and_then(|literal| literal.split_once('"'))
            else {
                continue;
            };
            constants.insert(name.to_string(), value.to_string());
        }
    }
    constants
}

/// 一格写法折成串：串字面量照取，常量（带不带模块路径）按 `constants` 取。
///
/// # Panics
/// 认不出的写法：这条用例要跟着认它，不能当没挂。
fn string_of_the_expression(
    expression: &str,
    constants: &BTreeMap<String, String>,
    where_it_is: &str,
) -> String {
    let expression = expression.trim().trim_end_matches(',').trim();
    if let Some(literal) = expression
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        return literal.to_string();
    }
    let constant_name = expression.rsplit("::").next().unwrap_or(expression);
    constants.get(constant_name).cloned().unwrap_or_else(|| {
        panic!("{where_it_is} 里观察者名或观察者代码文件写成 {expression}：认不出，这条用例要跟着认这种写法")
    })
}

/// `text` 从 `start` 起第一处 `name:` 那一格的写法（冒号之后到行末）；观察者的两种结构体第一格都是它。
fn name_field_after(text: &str, start: usize) -> Option<&str> {
    let after = &text[start..];
    let position = positions_of_the_word(after, "name:").into_iter().next()?;
    let field = &after[position + "name:".len()..];
    field.split('\n').next()
}

/// 一格的写法：从 `text` 起到同一层的 `,` 或收住这一格所在字面量的那个括号为止。
fn expression_up_to_the_end_of_the_field(text: &str) -> &str {
    let mut depth = 0_usize;
    for (position, character) in text.char_indices() {
        match character {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' if depth == 0 => return &text[..position],
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => return &text[..position],
            _ => {}
        }
    }
    text
}

/// 用例主文件 `main` 里 `position` 那一处挂的观察者的观察者代码文件：那一处直接写在 `observer:` 之后（挂法字面量
/// `AttachedCrashPointReplayObserver…` 的那一格）、同一个字面量的 `attachment:` 点名了 `CrashPointReplayObserverCodeFile::SharedFile(<共用文件>)` 的
/// 取共用文件，别的取用例文件。
fn observer_code_file_at(
    main: &str,
    position: usize,
    case_file: &str,
    constants: &BTreeMap<String, String>,
) -> String {
    let before = main[..position].trim_end();
    let is_the_observer_field_of_an_attachment =
        before.strip_suffix("observer:").is_some_and(|rest| {
            !rest
                .chars()
                .next_back()
                .is_some_and(is_an_identifier_character)
        });
    if !is_the_observer_field_of_an_attachment {
        return case_file.to_string();
    }
    let Some(attachment_start) = main[position..]
        .find("attachment:")
        .map(|offset| position + offset + "attachment:".len())
    else {
        return case_file.to_string();
    };
    let attachment = expression_up_to_the_end_of_the_field(&main[attachment_start..]);
    let Some((_, shared_file_onwards)) = attachment.split_once("SharedFile(") else {
        return case_file.to_string();
    };
    string_of_the_expression(
        expression_up_to_the_end_of_the_field(shared_file_onwards),
        constants,
        case_file,
    )
}

/// 一份源码里每个 fn 的名字。
fn names_of_the_functions_defined_in(source: &str) -> BTreeSet<String> {
    positions_of_the_word(source, "fn ")
        .into_iter()
        .map(|position| {
            source[position + "fn ".len()..]
                .chars()
                .take_while(|character| is_an_identifier_character(*character))
                .collect()
        })
        .collect()
}

/// 一份源码里造观察者的 fn（签名的返回类型是两种观察者之一）：fn 名 → 它造的观察者名（函数体里第一格 `name:`）。
fn observer_making_functions_of_the_source(
    source: &str,
    constants: &BTreeMap<String, String>,
    where_it_is: &str,
) -> BTreeMap<String, String> {
    let mut functions = BTreeMap::new();
    for position in positions_of_the_word(source, "fn ") {
        let rest = &source[position + "fn ".len()..];
        let function_name: String = rest
            .chars()
            .take_while(|character| is_an_identifier_character(*character))
            .collect();
        let Some(body_offset) = rest.find('{') else {
            continue;
        };
        let returns_an_observer = TYPES_OF_THE_OBSERVERS_OF_A_CRASH_ENUMERATION_CASE
            .iter()
            .any(|observer_type| rest[..body_offset].contains(&format!("-> {observer_type}<")));
        if !returns_an_observer {
            continue;
        }
        let name_field = name_field_after(rest, body_offset).unwrap_or_else(|| {
            panic!("{where_it_is} 里造观察者的 fn {function_name} 的函数体里没有 name: 那一格")
        });
        functions.insert(
            function_name,
            string_of_the_expression(name_field, constants, where_it_is),
        );
    }
    functions
}

/// 一个测试目标（`tests/<名>.rs`）是崩溃枚举用例、经 [`ENTRIES_HANGING_THE_OBSERVERS_OF_A_CRASH_ENUMERATION_CASE`] 挂观察者时，
/// 它挂的每个（观察者代码文件，观察者名）：主文件里两种观察者的字面量，与主文件调的、它声明的模块里造观察者的 fn（主文件自己定义了同名 fn 的
/// 按主文件的算）；观察者代码文件照 [`observer_code_file_at`]。不经那三个入口挂观察者的交空集。
fn observers_hung_by_the_crash_enumeration_case(test_target: &str) -> BTreeSet<(String, String)> {
    let sources = source_texts_of_the_target(&format!("test:{test_target}"));
    let main = &sources[0];
    let mut hung = BTreeSet::new();
    if !ENTRIES_HANGING_THE_OBSERVERS_OF_A_CRASH_ENUMERATION_CASE
        .iter()
        .any(|entry| main.contains(&format!("{entry}(")))
    {
        return hung;
    }
    let case_file = format!("crates/singlefs-verification/tests/{test_target}.rs");
    let constants = string_constants_of_the_sources(&sources);
    for observer_type in TYPES_OF_THE_OBSERVERS_OF_A_CRASH_ENUMERATION_CASE {
        for position in positions_of_the_word(main, &format!("{observer_type} {{")) {
            let name_field = name_field_after(main, position).unwrap_or_else(|| {
                panic!("{case_file} 里 {observer_type} 的字面量没有 name: 那一格")
            });
            hung.insert((
                observer_code_file_at(main, position, &case_file, &constants),
                string_of_the_expression(name_field, &constants, &case_file),
            ));
        }
    }
    let functions_of_the_main = names_of_the_functions_defined_in(main);
    for module in &sources[1..] {
        for (function_name, observer_name) in
            observer_making_functions_of_the_source(module, &constants, &case_file)
        {
            if functions_of_the_main.contains(&function_name) {
                continue;
            }
            for position in positions_of_the_word(main, &format!("{function_name}(")) {
                hung.insert((
                    observer_code_file_at(main, position, &case_file, &constants),
                    observer_name.clone(),
                ));
            }
        }
    }
    hung
}

/// 场景：这个包 `tests/` 下每份测试文件（每一个测试目标）；是崩溃枚举用例、经 `enumerate_crash_states_with_expansion`、
/// `…_with_every_kind_of_observer`、`…_with_attached_observers` 挂自己的观察者的，按 `crash_state_enumeration` 的
/// `observers_on_the_nodes_of_the_item` 的取法认它挂的每个（观察者代码文件，观察者名）：用例主文件里两种观察者的字面量，与它调的公共模块里
/// 造观察者的 fn；观察者代码文件是用例文件，挂法点名共用文件的取共用文件。
/// 预期：每一个在观察者登记表 `OBSERVER_ENTRIES` 里都有一行（没登记的观察者取键拒算：门禁先只录、再只判，两个进程各取各的占位键，
/// 判段读不到录入段写的块）；认出的观察者至少一个（认法坏了这里先红）。按来历挂的观察者（`crash_point_replay_provenance`）另由上面的用例管。
/// 判别力（`crates/mutations.tsv`「观察者登记表：读坏盘挂载流」起头的那一行）：登记表删掉读坏盘挂载流的一个观察者那一行，这条红。
#[test]
fn every_observer_a_crash_enumeration_case_hangs_by_itself_has_its_row_in_the_observer_registry() {
    let registered: BTreeSet<(&str, &str)> = OBSERVER_ENTRIES
        .iter()
        .map(|entry| (entry.observer_code_file, entry.observer_name))
        .collect();
    let mut test_targets: Vec<String> = std::fs::read_dir(test_directory_of_this_package())
        .unwrap_or_else(|error| panic!("这个包的测试目录列不出：{error}"))
        .map(|entry| entry.expect("测试目录的目录项读得出").path())
        .filter(|path| {
            path.is_file() && path.extension().is_some_and(|extension| extension == "rs")
        })
        .map(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .expect("测试文件名是 UTF-8")
                .to_string()
        })
        .collect();
    test_targets.sort();
    let mut cases_hanging_observers = 0_usize;
    let mut observers_hung = 0_usize;
    let mut unregistered = Vec::new();
    for test_target in &test_targets {
        let hung = observers_hung_by_the_crash_enumeration_case(test_target);
        if hung.is_empty() {
            continue;
        }
        cases_hanging_observers += 1;
        observers_hung += hung.len();
        for (observer_code_file, observer_name) in hung {
            if !registered.contains(&(observer_code_file.as_str(), observer_name.as_str())) {
                unregistered.push(format!(
                    "测试目标 {test_target}：观察者代码文件 {observer_code_file}，观察者名 {observer_name}"
                ));
            }
        }
    }
    println!(
        "OBSERVERS_HUNG_BY_THE_CRASH_ENUMERATION_CASES cases={cases_hanging_observers} observers={observers_hung} unregistered={}",
        unregistered.len()
    );
    assert!(observers_hung > 0, "一个挂着的观察者都没认出：认法坏了");
    assert!(
        unregistered.is_empty(),
        "这几个崩溃枚举用例挂的观察者在 OBSERVER_ENTRIES 里没有行（取键拒算，录入段与判段两个进程对不上块），逐个登记：{unregistered:#?}"
    );
}

/// 场景：跑流的那一处拿一个来历表里没有的名字去查、拿一个带展开标签的项名去查、调用方挂了一个与按来历挂的同名的观察者。
/// 预期：没有的名字拒（点名跑的那一处与名字）；带标签的项名查到它那条流的行；同名的拒——都在录任何东西之前。
#[test]
fn a_flow_without_a_row_is_refused_a_labelled_item_finds_its_flow_and_a_clashing_observer_is_refused(
) {
    assert_eq!(
        observers_attached_to_the_flow(
            RunOfTheFlow::CrashPointReplayEnumeration,
            "a_flow_nobody_registered::in_place_subsets::probe"
        ),
        Err(ProvenanceRefusal::NoRowForTheFlow {
            run: RunOfTheFlow::CrashPointReplayEnumeration,
            name_at_the_run: "a_flow_nobody_registered::in_place_subsets::probe".to_string(),
        }),
        "来历表里没有的流拒"
    );
    assert_eq!(
        flow_name_of_the_name_at_the_run("truncate_to_zero::segments_shorter_than_ten::rerun"),
        "truncate_to_zero",
        "项名第一个 :: 之前那一截是流名"
    );
    let observe_nothing =
        |_ordinal: u64,
         _image: &singlefs_verification::memory_pool::CrashImage<'_>,
         _report: &singlefs_core::recovery::RecoveryReport,
         _counts: &mut singlefs_verification::crash::CrashPointReplayObserverCounts| {};
    let clashing = [ObserverOnTheNodes {
        observer_name: ObserverName::new(
            ObserverAttachedByTheProvenance::DevicesWithoutASelfVerifiedSystemConfigurationSlot
                .observer_name(),
        )
        .expect("名字非空、不长"),
        observer_code_file:
            "crates/singlefs-verification/tests/crash_points_observer_registration_is_complete.rs"
                .to_string(),
        extra_inputs_digest: [0; DIGEST_BYTES],
        nodes: Vec::new(),
        observe: ObserverCode::CountsOnly(&observe_nothing),
    }];
    let refusal = observers_on_the_nodes_with_the_ones_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        "truncate_to_zero::segments_shorter_than_ten::rerun",
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
        "truncate_to_zero::segments_shorter_than_ten::probe",
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
        let mut counts = singlefs_verification::crash::CrashPointReplayObserverCounts::default();
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
