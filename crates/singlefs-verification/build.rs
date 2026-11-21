//! 编译时按判法单元的入口登记表（`src/judging_units_entries.rs`，经 `#[path]` 编同一份）逐组算键与组成清单，写进
//! `OUT_DIR/judging_units_keys.tsv`；库经 `include_str!(concat!(env!("OUT_DIR"), …))` 读回编进去的那一份，跑时不读盘上的树
//! （剪枝复查 design 路报告 3.8「build.rs 做什么」）。组有两种：每个观察者那一组（只留产品包的单元：观察者引到的被检测产品代码，观察者自己的代码不进）
//! 与判器引到的被检测产品代码那一组（从 CPU 参照判器、池级 checker 与记录核对器那几个入口收，只留产品包的单元）。录入键不在这里算：
//! 它只由录下的写流摘要与场景标识组成，代码一个单元都不进（`crate::crash_library::recording_key_of_the_scenario_and_the_inputs`）。
//!
//! 只做生成、不藏逻辑：切分、闭包、键、表的写法、特性的取法都在 `singlefs-unit-graph` 里（`entry_groups`、`build_features`）。
//! 这里只取 cargo 给的：本包开了哪些特性（cargo 编本构建脚本时带着本包这一次开了的特性的 cfg，[`FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CONDITIONAL_COMPILATION_FLAGS`]；
//! 不读进程环境里的 `CARGO_FEATURE_*`，外面带进来的同名变量分不出）、依赖包在这一次编译里开了哪些特性（带 `links` 的依赖包构建脚本交出的
//! `DEP_<links>_ENABLED_FEATURES`：编测试时本包的 dev 依赖替 `singlefs-core` 并上的特性，本包的 `CARGO_FEATURE_*` 看不到）与 `TARGET`
//! （收闭包时照它判平台条件）。工具链（`$RUSTC -vV`、`$CARGO -V`、RUSTFLAGS）与编译档不读、不进键，`Cargo.lock` 不进键（照旧盯着、改了重跑构建脚本；D13（验证路线） 已定项 17）；
//! 表里 `toolchain` 那一行只记收闭包时照的目标。
//! 进键的两种组都只按一组固定的 cfg 收：本包特性照登记表那一份的 `OBSERVER_FEATURES`、`JUDGED_PRODUCT_CODE_FEATURES`（都只开 `gpu`），
//! 依赖包一个特性都不当开着（交空的「依赖包被并上来的特性」）：带不带 `gpu`、带不带 `lookup-work-counters` 的几种构建（只录的那一趟与判段那一趟
//! 可以各在一份二进制里跑）算出同一把观察者的版本、同一把判器那一组的键。依赖包这一次实际开了哪些照旧读、照旧写进表末的 `build-feature` 行，
//! 只当记录（测试拿它对照），不进任何一组的键。

#[allow(
    dead_code,
    reason = "登记表那一份是库与构建脚本共用的：库里加一项只给库用的，构建脚本不为它报未用"
)]
#[path = "src/judging_units_entries.rs"]
mod judging_units_entries;

use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use singlefs_unit_graph::build_features::{
    features_enabled_on_the_dependencies, features_of_the_crate_being_built,
    FeaturesEnabledByTheBuild,
};
use singlefs_unit_graph::closure::PackagesKeptInTheKey;
use singlefs_unit_graph::entry_groups::{
    entry_group_key_keeping_only_the_listed_packages,
    entry_group_table_text_with_the_features_of_the_build, target_line_of_the_table, EntryGroupKey,
    EntryGroupRequest,
};
use singlefs_unit_graph::sha256::sha256_of;
use singlefs_unit_graph::{load_workspace, RuleSwitches, TargetSelector};

/// 写进 `OUT_DIR` 的那份表的文件名（库按同一个名字读回）。
const JUDGING_UNITS_KEYS_FILE_NAME: &str = "judging_units_keys.tsv";

/// 写进 `OUT_DIR` 的被监视文件摘要表的文件名（库的判法单元键测试按同一个名字读回，先比摘要再比键：`crash_identity` 测试模块的
/// `COMPILED_WATCHED_FILE_DIGESTS_TEXT`）。每行「SHA-256 小写十六进制<TAB>相对仓根的路径」，路径照 `rerun-if-changed` 那一份、按名字排好。
const WATCHED_FILE_DIGESTS_FILE_NAME: &str = "watched_file_digests.tsv";

/// 被监视的每份文件此刻的摘要（写法见 [`WATCHED_FILE_DIGESTS_FILE_NAME`]）。算完键之后才读：算键与读摘要之间（构建脚本自己跑的那几秒）
/// 树被改了的，摘要取的是改后的那一份，测试那一侧比摘要认不出，照旧报键不等。
///
/// # Panics
/// 有一份被监视的文件读不了：它在闭包或清单里，刚才算键时读过。
fn watched_file_digests_text(repository_root: &Path, watched: &BTreeSet<String>) -> String {
    watched
        .iter()
        .map(|path| {
            let bytes = std::fs::read(repository_root.join(path)).unwrap_or_else(|error| {
                panic!("被监视的文件 {path} 读不了（算键时读过）：{error}")
            });
            format!("{}\t{path}\n", sha256_of(&bytes).hexadecimal())
        })
        .collect()
}

/// 本包清单 `[features]` 里的每个特性（清单里的写法，按名字排好）与 cargo 这一次开没开它。与清单对不上（漏写、写错名字）的，
/// `features_of_the_crate_being_built` 交错误、构建脚本停下。
const FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CONDITIONAL_COMPILATION_FLAGS: [(&str, bool); 3] = [
    ("gpu", cfg!(feature = "gpu")),
    ("apparatus-binaries", cfg!(feature = "apparatus-binaries")),
    ("default", cfg!(feature = "default")),
];

fn environment_variable(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("cargo 起构建脚本时设 {name}"))
}

/// 一组一组互不依赖的活按 `threads` 条线程算，结果按 `requests` 的次序收（与一条线程逐个算时逐项相同）。
///
/// # Panics
/// 算一组的那一下 panic（登记表写错了），或收结果时有一格没算过：每一格都由某一条线程算过。
fn computed_in_parallel<Request: Sync, Computed: Send>(
    requests: &[Request],
    threads: NonZeroUsize,
    compute: impl Fn(&Request) -> Computed + Sync,
) -> Vec<Computed> {
    let next_request = std::sync::atomic::AtomicUsize::new(0);
    let computed: Vec<std::sync::Mutex<Option<Computed>>> = requests
        .iter()
        .map(|_| std::sync::Mutex::new(None))
        .collect();
    std::thread::scope(|scope| {
        for _ in 0..threads.get().min(requests.len()) {
            scope.spawn(|| loop {
                let index = next_request.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(request) = requests.get(index) else {
                    break;
                };
                let computed_for_the_request = compute(request);
                *computed[index]
                    .lock()
                    .expect("没有线程在持着这把锁时 panic") = Some(computed_for_the_request);
            });
        }
    });
    computed
        .into_iter()
        .map(|slot| {
            slot.into_inner()
                .expect("算的线程都已收回，没有一条在持着锁时 panic")
                .expect("每一格都由某一条线程算过")
        })
        .collect()
}

fn main() {
    let manifest_directory = PathBuf::from(environment_variable("CARGO_MANIFEST_DIR"));
    // 包目录往上两层就是仓根（crates/<包>）：用 parent() 取，不写往上的相对路径字面量
    let repository_root = manifest_directory
        .parent()
        .and_then(Path::parent)
        .expect("包目录在 crates/ 下，往上两层是仓根")
        .to_path_buf();
    let threads = std::env::var("NUM_JOBS")
        .or_else(|_| std::env::var("SINGLEFS_THREAD_CAP"))
        .ok()
        .and_then(|text| text.parse::<usize>().ok())
        .and_then(NonZeroUsize::new)
        .unwrap_or(NonZeroUsize::MIN);
    let switches = RuleSwitches::every_rule_intact();
    let workspace = load_workspace(&repository_root, threads, &switches)
        .unwrap_or_else(|error| panic!("装不起工作区：{error}"));
    let own_crate = workspace
        .crate_named(&environment_variable("CARGO_PKG_NAME"))
        .expect("工作区里有正在编的这个包");
    let read_environment = |name: &str| std::env::var(name).ok();
    // 只核本包特性表与清单 `[features]` 逐个对得上（表末交给依赖本包的构建脚本那一行照这张表写）；进键的几组不照这一次开了哪些
    features_of_the_crate_being_built(
        &workspace,
        own_crate,
        &FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CONDITIONAL_COMPILATION_FLAGS,
    )
    .unwrap_or_else(|error| panic!("本包这一次开了哪些特性取不了：{error}"));
    let features_enabled_by_the_build =
        features_enabled_on_the_dependencies(&workspace, own_crate, &read_environment)
            .unwrap_or_else(|error| panic!("依赖包在这一次编译里开了哪些特性取不全：{error}"));
    let target_triple = environment_variable("TARGET");
    let observer_requests: Vec<EntryGroupRequest> = judging_units_entries::OBSERVER_ENTRIES
        .iter()
        .map(|observer| {
            let group_name = judging_units_entries::observer_group_name(
                observer.observer_code_file,
                observer.observer_name,
            );
            EntryGroupRequest {
                target: TargetSelector::parse(observer.target).unwrap_or_else(|text| {
                    panic!("登记表里组 {group_name} 的目标写法认不得：{text}")
                }),
                entry_paths: observer
                    .entry_functions
                    .iter()
                    .map(|entry| (*entry).to_string())
                    .collect(),
                features: judging_units_entries::OBSERVER_FEATURES
                    .iter()
                    .map(|feature| (*feature).to_string())
                    .collect(),
                group_name,
            }
        })
        .collect();
    let packages_kept_in_the_keys = PackagesKeptInTheKey {
        package_names: judging_units_entries::PRODUCT_PACKAGES_KEPT_IN_THE_KEYS
            .iter()
            .map(|package_name| (*package_name).to_string())
            .collect(),
    };
    // 观察者那几组只键产品包（观察者自己的代码不进），只按一组固定的 cfg 收：本包特性固定、依赖包一个特性都不当开着，
    // 不照这一次编译开了哪些
    let observer_keys = computed_in_parallel(&observer_requests, threads, |request| {
        entry_group_key_keeping_only_the_listed_packages(
            &workspace,
            request,
            &packages_kept_in_the_keys,
            &FeaturesEnabledByTheBuild::default(),
            &target_triple,
            &switches,
        )
        .unwrap_or_else(|error| panic!("登记表里组 {} 的入口写错了：{error}", request.group_name))
    });
    // 判器引到的被检测产品代码那一组只键产品包（判器、池级 checker、记录核对器的代码不进），只按一组固定的 cfg 收：
    // 本包特性固定、依赖包一个特性都不当开着，不照这一次编译开了哪些
    let judged_product_code_key = entry_group_key_keeping_only_the_listed_packages(
        &workspace,
        &EntryGroupRequest {
            group_name: judging_units_entries::JUDGED_PRODUCT_CODE_GROUP_NAME.to_string(),
            target: TargetSelector::parse(judging_units_entries::JUDGED_PRODUCT_CODE_TARGET)
                .unwrap_or_else(|text| panic!("登记表里判器那一组的目标写法认不得：{text}")),
            entry_paths: judging_units_entries::JUDGED_PRODUCT_CODE_ENTRIES
                .iter()
                .map(|entry| (*entry).to_string())
                .collect(),
            features: judging_units_entries::JUDGED_PRODUCT_CODE_FEATURES
                .iter()
                .map(|feature| (*feature).to_string())
                .collect(),
        },
        &packages_kept_in_the_keys,
        &FeaturesEnabledByTheBuild::default(),
        &target_triple,
        &switches,
    )
    .unwrap_or_else(|error| panic!("登记表里判器引到的被检测产品代码那一组的入口写错了：{error}"));
    let keys: Vec<EntryGroupKey> = observer_keys
        .into_iter()
        .chain(std::iter::once(judged_product_code_key))
        .collect();
    let output_path =
        PathBuf::from(environment_variable("OUT_DIR")).join(JUDGING_UNITS_KEYS_FILE_NAME);
    std::fs::write(
        &output_path,
        entry_group_table_text_with_the_features_of_the_build(
            &[target_line_of_the_table(&target_triple)],
            &features_enabled_by_the_build,
            &keys,
        ),
    )
    .unwrap_or_else(|error| panic!("写不了 {}：{error}", output_path.display()));
    let mut watched: BTreeSet<String> = [
        "Cargo.lock",
        "Cargo.toml",
        "crates/singlefs-verification/build.rs",
        "crates/singlefs-verification/src/judging_units_entries.rs",
    ]
    .iter()
    .map(|path| (*path).to_string())
    .collect();
    watched.extend(
        workspace
            .crates
            .iter()
            .map(|workspace_crate| workspace_crate.directory.join("Cargo.toml"))
            .filter_map(|path| {
                path.strip_prefix(&repository_root)
                    .ok()
                    .map(|relative| relative.display().to_string())
            }),
    );
    for key in &keys {
        watched.extend(key.watched_files.iter().cloned());
    }
    for path in &watched {
        println!(
            "cargo:rerun-if-changed={}",
            repository_root.join(path).display()
        );
    }
    let digests_path =
        PathBuf::from(environment_variable("OUT_DIR")).join(WATCHED_FILE_DIGESTS_FILE_NAME);
    std::fs::write(
        &digests_path,
        watched_file_digests_text(&repository_root, &watched),
    )
    .unwrap_or_else(|error| panic!("写不了 {}：{error}", digests_path.display()));
    // 把本包这一次开了的特性交给直接依赖本包的构建脚本（`links` 的元数据，键名是 `singlefs_unit_graph::build_features::ENABLED_FEATURES_METADATA_KEY`，
    // 值的写法同 `crates/singlefs-core/build.rs`：开着的特性名写成 cargo 环境变量名的样子、逗号隔开）。
    println!(
        "cargo:{}={}",
        singlefs_unit_graph::build_features::ENABLED_FEATURES_METADATA_KEY,
        FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CONDITIONAL_COMPILATION_FLAGS
            .iter()
            .filter(|(_, is_enabled_by_cargo)| *is_enabled_by_cargo)
            .map(|(feature, _)| feature.to_ascii_uppercase().replace('-', "_"))
            .collect::<Vec<String>>()
            .join(",")
    );
}
