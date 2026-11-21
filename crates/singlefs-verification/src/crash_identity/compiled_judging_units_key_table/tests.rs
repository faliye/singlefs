//! 判法单元键表的测试，与几组测试共用的取键、拷树、收闭包的辅助（装工作区的线程从单测进程里层那一份额度拿）。

use super::*;
use crate::judging_units_entries::{
    observer_group_name, JUDGED_PRODUCT_CODE_ENTRIES, JUDGED_PRODUCT_CODE_FEATURES,
    JUDGED_PRODUCT_CODE_GROUP_NAME, JUDGED_PRODUCT_CODE_TARGET, OBSERVER_ENTRIES,
    OBSERVER_FEATURES, PRODUCT_PACKAGES_KEPT_IN_THE_KEYS,
};
use singlefs_unit_graph::closure::PackagesKeptInTheKey;
use singlefs_unit_graph::conditional_compilation::CARGO_TARGET_TOOLCHAIN_LINE_PREFIX;
use singlefs_unit_graph::entry_groups::{
    entry_group_key_keeping_only_the_listed_packages,
    entry_group_key_not_following_the_listed_units, EntryGroupKey, EntryGroupRequest,
    EntryGroupVerdict,
};
use singlefs_unit_graph::heavy_threads::default_heavy_threads;
use singlefs_unit_graph::sha256::sha256_of;
use singlefs_unit_graph::{load_workspace, RuleSwitches, TargetSelector, Workspace};
use std::path::{Path, PathBuf};

#[cfg(feature = "gpu")]
mod dispatch_widths_judge_the_same_verdict_words;

mod build_inputs_and_tree_copies;

mod keys_after_edits;

mod observer_keys;

mod compiled_compositions;

mod judged_product_code;

/// 单测进程里层那一份额度同时容得下几次装工作区（`load_workspace`）：一次装工作区只要那一份的这么多分之一（至少 1 条），
/// 这几条同时装工作区的用例各拿各的、一起跑，不互相排成一列（推的：这个模块里装工作区的用例几十条，同时到的有几条没量过）。
const WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS: usize = 4;

/// 里层那一份额度是 `inner_threads_of_the_share` 条时，一次装工作区要几条：那一份的
/// [`WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS`] 分之一，向下取整，至少 1 条。
fn workspace_loading_threads_wanted(inner_threads_of_the_share: usize) -> usize {
    (inner_threads_of_the_share / WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS).max(1)
}

/// 装工作区那一下开的线程：从单测进程里层那一份额度拿 [`workspace_loading_threads_wanted`] 条
/// （[`crate::crash_library::InnerThreadsTaken`]，放不下就等别的用例还），不照每台默认重活线程数开：libtest 已照这个数并跑那么多条用例。
/// 交回守卫与交给 `load_workspace` 的条数；守卫在 `load_workspace` 返回之后丢掉才还（它的线程在返回之前都收齐了）。
fn workspace_loading_threads_from_the_test_process_share() -> (
    crate::crash_library::InnerThreadsTaken,
    std::num::NonZeroUsize,
) {
    let inner_threads_of_the_share =
        crate::crash_library::shares_of_the_test_process(default_heavy_threads().get())
            .inner_threads;
    let threads_taken = crate::crash_library::InnerThreadsTaken::take(
        workspace_loading_threads_wanted(inner_threads_of_the_share),
    );
    let threads = std::num::NonZeroUsize::new(threads_taken.threads())
        .expect("里层额度至少给一条（inner_threads_granted 取了 max(1)）");
    (threads_taken, threads)
}

/// 场景：里层那一份额度从 1 条到 64 条逐个取，每一种按 [`workspace_loading_threads_wanted`] 与
/// `crate::crash_library::inner_threads_granted` 算一次装工作区拿到几条，再照 `crate::crash_library::inner_threads_fit`
/// 把装工作区逐次放进这一份，放到 [`WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS`] 次或放不下为止。只算数，不碰单测进程此刻占着的额度，不等别的用例。
/// 预期分两种：里层不少于 [`WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS`] 条时，那几次逐次都放得下（同时装工作区的用例不互相排成一列），
/// 几次合起来拿到的与那一份差不到 [`WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS`] 条（分份只取整掉零头）；里层少于那个数时，
/// 每次只拿 1 条，放得下的恰好是里层的条数那么多次（每条线程各装一次工作区，不多占也不空着）。
/// 判别力（`crates/mutations.tsv`）：一次装工作区要整份里层额度，里层 2 条起第二次就放不下，放进的次数对不上，这条红。
#[test]
fn workspace_loadings_sharing_the_inner_threads_fit_in_the_share_together() {
    for inner_threads_of_the_share in 1..=64 {
        let granted = crate::crash_library::inner_threads_granted(
            workspace_loading_threads_wanted(inner_threads_of_the_share),
            inner_threads_of_the_share,
        );
        let mut in_use = 0;
        let mut loadings_placed = 0;
        // 迭代上界是 WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS 次；放不下就停
        while loadings_placed < WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS
            && crate::crash_library::inner_threads_fit(in_use, granted, inner_threads_of_the_share)
        {
            in_use += granted;
            loadings_placed += 1;
        }
        if inner_threads_of_the_share >= WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS {
            assert_eq!(
                    loadings_placed, WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS,
                    "里层 {inner_threads_of_the_share} 条、每次装工作区拿 {granted} 条：只放进 {loadings_placed} 次（占 {in_use} 条），同时装工作区的用例排成了一列"
                );
            assert!(
                    in_use + WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS > inner_threads_of_the_share,
                    "里层 {inner_threads_of_the_share} 条、每次装工作区拿 {granted} 条：几次合起来只 {in_use} 条，余下的够再分一轮"
                );
        } else {
            assert_eq!(
                    (granted, loadings_placed),
                    (1, inner_threads_of_the_share),
                    "里层 {inner_threads_of_the_share} 条少于 {WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS}：每次装工作区该拿 1 条、放得下里层条数那么多次"
                );
        }
    }
}

// ============================== 判法单元：编进二进制的那份判器键表 ==============================

/// 组名 → 那一组的键（拒算或不编进来的是 `None`）。
type KeysOfEachGroup = BTreeMap<String, Option<[u8; DIGEST_BYTES]>>;

/// 编进二进制的那份表里构建脚本收闭包时照的目标三元组（`toolchain` 那一行 `target: <三元组>`）：重算照同一个目标判平台条件。
/// 工具链行与 `Cargo.lock` 不进键，重算不拼它们。
fn target_triple_compiled_in() -> String {
    let compiled = &compiled_judging_units_keys().toolchain_lines;
    compiled
        .iter()
        .find_map(|line| line.strip_prefix(CARGO_TARGET_TOOLCHAIN_LINE_PREFIX))
        .unwrap_or_else(|| {
            panic!("编进去的表里有「{CARGO_TARGET_TOOLCHAIN_LINE_PREFIX}」那一行：{compiled:?}")
        })
        .to_string()
}

/// 一组怎么收（与构建脚本同一套）：观察者那几组与判器引到的被检测产品代码那一组都只键产品包、只按一组固定的 cfg 收
/// （本包特性照请求，依赖包一个特性都不当开着）；收全闭包的那一种只给阳性对照用（改的那一处真在闭包里），依赖包的特性同样不当开着。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnitsOfTheGroup {
    ProductPackagesOnly,
    WholeClosure,
}

/// 观察者那几组的请求（与构建脚本交的相同：本包特性固定，按登记的次序）。
fn observer_requests() -> Vec<EntryGroupRequest> {
    OBSERVER_ENTRIES
        .iter()
        .map(|observer| EntryGroupRequest {
            group_name: observer_group_name(observer.observer_code_file, observer.observer_name),
            target: TargetSelector::parse(observer.target).expect("登记表里的目标写法认得"),
            entry_paths: observer
                .entry_functions
                .iter()
                .map(|entry| (*entry).to_string())
                .collect(),
            features: OBSERVER_FEATURES
                .iter()
                .map(|feature| (*feature).to_string())
                .collect(),
        })
        .collect()
}

/// 判器引到的被检测产品代码那一组的请求（与构建脚本交的相同：本包特性固定）。
fn judged_product_code_request() -> EntryGroupRequest {
    EntryGroupRequest {
        group_name: JUDGED_PRODUCT_CODE_GROUP_NAME.to_string(),
        target: TargetSelector::parse(JUDGED_PRODUCT_CODE_TARGET).expect("登记表里的目标写法认得"),
        entry_paths: JUDGED_PRODUCT_CODE_ENTRIES
            .iter()
            .map(|entry| (*entry).to_string())
            .collect(),
        features: JUDGED_PRODUCT_CODE_FEATURES
            .iter()
            .map(|feature| (*feature).to_string())
            .collect(),
    }
}

/// 登记表里的每一组（观察者那几组、判器引到的被检测产品代码那一组，按构建脚本写表的次序），与构建脚本交的请求相同。
fn requests_of_the_entries_table() -> Vec<(EntryGroupRequest, UnitsOfTheGroup)> {
    observer_requests()
        .into_iter()
        .map(|request| (request, UnitsOfTheGroup::ProductPackagesOnly))
        .chain(std::iter::once((
            judged_product_code_request(),
            UnitsOfTheGroup::ProductPackagesOnly,
        )))
        .collect()
}

/// 进键的几组留的包（与构建脚本交的相同）。
fn packages_kept_in_the_keys() -> PackagesKeptInTheKey {
    PackagesKeptInTheKey {
        package_names: PRODUCT_PACKAGES_KEPT_IN_THE_KEYS
            .iter()
            .map(|package_name| (*package_name).to_string())
            .collect(),
    }
}

/// `root` 那棵树装成的工作区与编进去的那份表里收闭包时照的目标（[`target_triple_compiled_in`]）：算键的几样输入。
struct TreeLoadedForTheKeys {
    workspace: Workspace,
    target_triple: String,
}

impl TreeLoadedForTheKeys {
    fn load(root: &Path) -> Self {
        let switches = RuleSwitches::every_rule_intact();
        let (threads_taken, threads) = workspace_loading_threads_from_the_test_process_share();
        let workspace = load_workspace(root, threads, &switches).expect("工作区装得起来");
        drop(threads_taken);
        Self {
            workspace,
            target_triple: target_triple_compiled_in(),
        }
    }

    /// 照构建脚本那一套算一组：观察者那几组与判器引到的被检测产品代码那一组只键产品包、依赖包一个特性都不当开着
    /// （[`Self::product_code_key_under`] 交空表）；收全闭包的那一种给阳性对照用，依赖包的特性同样交空表。
    fn group_key(&self, request: &EntryGroupRequest, units: UnitsOfTheGroup) -> EntryGroupKey {
        match units {
            UnitsOfTheGroup::ProductPackagesOnly => {
                self.product_code_key_under(request, &FeaturesEnabledByTheBuild::default())
            }
            UnitsOfTheGroup::WholeClosure => entry_group_key_not_following_the_listed_units(
                &self.workspace,
                request,
                &BTreeSet::new(),
                &FeaturesEnabledByTheBuild::default(),
                &self.target_triple,
                &RuleSwitches::every_rule_intact(),
            )
            .map(|keyed| keyed.key)
            .unwrap_or_else(|error| panic!("组 {} 的入口写错了：{error}", request.group_name)),
        }
    }

    /// 只键产品包的一组（观察者那几组、判器引到的被检测产品代码那一组）照 `dependency_features` 当依赖包开着的特性算
    /// （构建脚本交空表；单测另交一次编译里实际并上来的那一份对照）。
    fn product_code_key_under(
        &self,
        request: &EntryGroupRequest,
        dependency_features: &FeaturesEnabledByTheBuild,
    ) -> EntryGroupKey {
        entry_group_key_keeping_only_the_listed_packages(
            &self.workspace,
            request,
            &packages_kept_in_the_keys(),
            dependency_features,
            &self.target_triple,
            &RuleSwitches::every_rule_intact(),
        )
        .unwrap_or_else(|error| panic!("组 {} 的入口写错了：{error}", request.group_name))
    }
}

/// 一组的键（拒算或不编进来的是 `None`）。
fn key_of_the_verdict(verdict: &EntryGroupVerdict) -> Option<[u8; DIGEST_BYTES]> {
    match verdict {
        EntryGroupVerdict::Keyed { key } => Some(key.0),
        EntryGroupVerdict::Refused { .. } | EntryGroupVerdict::NotCompiled { .. } => None,
    }
}

/// 照入口登记表在 `root` 那棵树上逐组算键（与构建脚本同一套），只算 `is_wanted` 点了名的组：组名 → 键（拒算或不编进来的是 `None`）。
fn registry_keys_of_the_groups_under(
    root: &Path,
    is_wanted: &dyn Fn(&str) -> bool,
) -> KeysOfEachGroup {
    let tree = TreeLoadedForTheKeys::load(root);
    requests_of_the_entries_table()
        .iter()
        .filter(|(request, _units)| is_wanted(&request.group_name))
        .map(|(request, units)| {
            (
                request.group_name.clone(),
                key_of_the_verdict(&tree.group_key(request, *units).verdict),
            )
        })
        .collect()
}

/// 照入口登记表在 `root` 那棵树上逐组算键（与构建脚本同一套）：组名 → 键（拒算或不编进来的是 `None`）。
fn registry_keys_under(root: &Path) -> KeysOfEachGroup {
    registry_keys_of_the_groups_under(root, &|_group_name| true)
}

/// 编进二进制的那份表里每一组的键（拒算或不编进来的是 `None`）。
fn compiled_keys() -> KeysOfEachGroup {
    compiled_judging_units_keys()
        .groups
        .iter()
        .map(|group| {
            let key = match &group.verdict {
                ReadBackEntryGroupVerdict::Keyed { key } => Some(*key),
                ReadBackEntryGroupVerdict::Refused { .. }
                | ReadBackEntryGroupVerdict::NotCompiled { .. } => None,
            };
            (group.group_name.clone(), key)
        })
        .collect()
}

/// 被监视的文件（`watched` 行的写法：相对仓根的路径）里不在 `crates/` 底下、又不是根上 `Cargo.toml`、`Cargo.lock` 的那几份，按给的次序：
/// 拷树重算时 `crates/` 与根上那两份另拷，这几份要单独拷。
fn watched_files_outside_the_crates_and_the_root_manifests(watched_files: &[String]) -> Vec<&str> {
    watched_files
        .iter()
        .map(String::as_str)
        .filter(|path| {
            !path.starts_with("crates/") && *path != "Cargo.toml" && *path != "Cargo.lock"
        })
        .collect()
}

/// 构建脚本算键时闭包读到、又不在 `crates/` 底下的文件（编进去的那份表的 `watched` 行里
/// [`watched_files_outside_the_crates_and_the_root_manifests`] 挑出的那几份：`include_str!` 按清单目录往上读到的数据文件，
/// 闭包够到 `crates/` 之外经 path 依赖收进来的 crate 时那个 crate 库目标的文件）：拷树重算时要连它们一起拷，两边才是同一套输入——
/// 拷漏了，读它的那一组在拷贝上判 `data-file-unreadable` 拒算，键对不上编进去的那一把。
fn data_files_outside_the_crates_the_build_read() -> Vec<&'static str> {
    watched_files_outside_the_crates_and_the_root_manifests(
        &compiled_judging_units_keys().watched_files,
    )
}

/// 构建脚本写进 `OUT_DIR` 的被监视文件摘要表（文件名与 `build.rs` 的 `WATCHED_FILE_DIGESTS_FILE_NAME` 相同）：
/// 每行「SHA-256 小写十六进制<TAB>相对仓根的路径」，罩着构建脚本 `rerun-if-changed` 的每份文件。
const COMPILED_WATCHED_FILE_DIGESTS_TEXT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/watched_file_digests.tsv"));

/// 摘要表（写法见 [`COMPILED_WATCHED_FILE_DIGESTS_TEXT`]）里与 `read_file` 此刻读到的内容摘要不同的路径，读不了（`None`）的也算，
/// 按表里的次序。
///
/// # Panics
/// 表里有一行不是「摘要<TAB>路径」两段。
fn watched_files_changed_since_the_build(
    digests_text: &str,
    read_file: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Vec<String> {
    digests_text
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let Some((digest_at_the_build, path)) = line.split_once('\t') else {
                panic!("被监视文件摘要表一行不是「摘要<TAB>路径」两段：{line:?}");
            };
            let is_unchanged = read_file(path)
                .is_some_and(|bytes| sha256_of(&bytes).hexadecimal() == digest_at_the_build);
            (!is_unchanged).then(|| path.to_string())
        })
        .collect()
}

/// `repository` 那棵树装成工作区（`load_workspace`）时收进来的、不在 `crates/` 底下的 crate 目录（相对仓根）：根清单 `members` 点名的
/// 成员（例：`gpu-verification-server`）与顺 path 依赖收进来的工作区外 crate（例：`prune-server`）。构建脚本监视 `load_workspace`
/// 收进来的每个 crate 的 `Cargo.toml`（`crates/singlefs-verification/build.rs` 里从 `workspace.crates` 取清单路径的那一段），这里调同一个
/// `load_workspace`：收成员的规则是 `crates/singlefs-unit-graph/src/workspace.rs` 的 `directories_of_the_workspace_members`（文档注释
/// 「与 `scripts/gate/lib_function_units.py` 的 `discover_rust_crates` 收成员的那一段同一条规则」），那个函数不公开，`load_workspace`
/// 是单元图照它收成员的公开入口，不另抄一份。拷树漏了哪个 crate 的清单，先比摘要那一步点名它读不了。
///
/// # Panics
/// 工作区装不起来；收进来的 crate 落在仓根之外，或就是仓根自己（根清单带 `[package]`：拷树只拷根上的两份清单，不拷仓根这个包的源码）。
fn crate_directories_outside_the_crates_directory(repository: &Path) -> BTreeSet<PathBuf> {
    let switches = RuleSwitches::every_rule_intact();
    let (threads_taken, threads) = workspace_loading_threads_from_the_test_process_share();
    let workspace = load_workspace(repository, threads, &switches).expect("工作区装得起来");
    drop(threads_taken);
    let crates_directory = workspace.repository_root.join("crates");
    workspace
        .crates
        .iter()
        .filter(|workspace_crate| !workspace_crate.directory.starts_with(&crates_directory))
        .map(|workspace_crate| {
            let relative = workspace_crate
                .directory
                .strip_prefix(&workspace.repository_root)
                .unwrap_or_else(|_| {
                    panic!(
                        "收进来的 crate {} 落在仓根之外，拷树拷不到它",
                        workspace_crate.directory.display()
                    )
                })
                .to_path_buf();
            assert!(
                !relative.as_os_str().is_empty(),
                "仓根自己是一个包：拷树只拷根上的两份清单，不拷这个包的源码"
            );
            relative
        })
        .collect()
}

/// 一棵仓（[`RepositoryCopy::new`] 是编这个测试二进制的那一份活树）里的 `crates/`、`Cargo.toml`、`Cargo.lock`、工作区收进来的 `crates/` 之外的 crate 目录
/// （[`crate_directories_outside_the_crates_directory`]）与另拷的 `crates/` 之外的文件（活树那一份是构建时闭包读到的数据文件，
/// [`data_files_outside_the_crates_the_build_read`]）拷进一个新的临时目录（离开作用域时删掉）。
/// 用活树那一份的测试断言的就是真仓上的判器登记表（登记的入口都编得进、真仓上改一处哪几组键变），照旧拒算（断言本身关于真仓的留着）；
/// 活树的仓根只在 [`RepositoryCopy::new`] 里现取，别的测试够不到它。
struct RepositoryCopy {
    root: crate::crash_library::ExclusiveTemporaryDirectory,
}

impl RepositoryCopy {
    /// 拷编这个测试二进制的那一份活树，另拷的 `crates/` 之外的文件是构建时闭包读到的那几份（[`data_files_outside_the_crates_the_build_read`]）。
    fn new(label: &str) -> Self {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("本 crate 在仓根的 crates/ 下")
            .to_path_buf();
        Self::of_the_repository(
            label,
            &repository,
            &data_files_outside_the_crates_the_build_read(),
        )
    }

    /// 拷 `repository` 那棵树：`crates/`（各包的 `target` 不拷）、根上的 `Cargo.toml` 与 `Cargo.lock`、工作区收进来的 `crates/` 之外的 crate 目录
    /// （根清单 `members` 点名的成员与顺 path 依赖收进来的，同样不拷 `target`），
    /// 再把 `data_files_outside_the_crates`（相对仓根的路径）逐份拷到临时根底下同样的相对路径（缺的上级目录现建）。
    /// 另拷的清单是参数：活树那一份由 [`RepositoryCopy::new`] 交构建时读到的那几份，不要活树的用例交现造的小仓与清单。
    ///
    /// # Panics
    /// 读不了、拷不了、建不了目录，或清单里有一份在 `repository` 里读不了。
    fn of_the_repository(
        label: &str,
        repository: &Path,
        data_files_outside_the_crates: &[&str],
    ) -> Self {
        fn copy_directory(from: &Path, to: &Path) {
            std::fs::create_dir_all(to).expect("建得了目录");
            for entry in std::fs::read_dir(from).expect("读得了目录").flatten() {
                let destination = to.join(entry.file_name());
                if entry.file_type().expect("读得了类型").is_dir() {
                    if entry.file_name() != "target" {
                        copy_directory(&entry.path(), &destination);
                    }
                } else {
                    std::fs::copy(entry.path(), &destination).expect("拷得了文件");
                }
            }
        }
        let root = crate::crash_library::ExclusiveTemporaryDirectory::create(&format!(
            "judging-units-{label}"
        ))
        .expect("建得了一次性临时目录");
        copy_directory(&repository.join("crates"), &root.join("crates"));
        for file in ["Cargo.toml", "Cargo.lock"] {
            std::fs::copy(repository.join(file), root.join(file)).expect("拷得了根文件");
        }
        for crate_directory in crate_directories_outside_the_crates_directory(repository) {
            copy_directory(
                &repository.join(&crate_directory),
                &root.join(&crate_directory),
            );
        }
        for data_file in data_files_outside_the_crates {
            let destination = root.join(data_file);
            std::fs::create_dir_all(
                destination
                    .parent()
                    .expect("仓内相对路径拼到临时根底下，总有上一级目录"),
            )
            .expect("建得了数据文件的目录");
            std::fs::copy(repository.join(data_file), &destination)
                .unwrap_or_else(|error| panic!("拷得了构建时读到的数据文件 {data_file}：{error}"));
        }
        Self { root }
    }

    /// 先比摘要：拷贝里被监视的每份文件与编进二进制的摘要表逐份相同，才拿这份拷贝重算的键去比编进去的键；
    /// 对不上的，键算的已经不是编进去的那一份树，报「树在编译后变了」并点名那几份，不往下比键。
    ///
    /// # Panics
    /// 有被监视的文件此刻与编译时的摘要不同或读不了。
    fn assert_the_watched_files_are_the_ones_the_build_read(&self) {
        let changed =
            watched_files_changed_since_the_build(COMPILED_WATCHED_FILE_DIGESTS_TEXT, &|path| {
                std::fs::read(self.root.join(path)).ok()
            });
        assert!(
                changed.is_empty(),
                "树在编译后变了：这几份被监视的文件与编进二进制的摘要对不上（改过或读不了），键不比，重编再跑：{changed:?}"
            );
    }

    /// 一份文件里恰好出现一次的原文换成替换文；不是恰好一次就 panic（锚点腐化不许悄悄放过）。
    fn replace_exactly_once(&self, relative: &str, original: &str, replacement: &str) {
        let path = self.root.join(relative);
        let text = std::fs::read_to_string(&path).expect("读得了");
        assert_eq!(
            text.matches(original).count(),
            1,
            "{relative} 里「{original}」要恰好一处"
        );
        std::fs::write(&path, text.replacen(original, replacement, 1)).expect("写得了");
    }
}

/// 一组在一棵树上：照构建脚本那一套算的那一把（拒算的是 `None`）与它的组成清单，连同同一组入口收全闭包
/// （不排观察者、不只留几个包）时的组成清单——阳性对照用：改的那一处真在这一组的闭包里。组成清单一行一个单元
/// （`<种类>\t<限定名>\t<文件>:<起>-<止>\t<指纹>`）。
struct GroupUnderTheTree {
    key: Option<[u8; DIGEST_BYTES]>,
    kept_composition: Vec<String>,
    whole_composition: Vec<String>,
}

impl GroupUnderTheTree {
    /// 照构建脚本那一套算的那一把；拒算或没编进来就 panic，点名是哪一组（两边都拒算时相等，判不出变没变）。
    fn key_of(&self, group: &str) -> [u8; DIGEST_BYTES] {
        self.key
            .unwrap_or_else(|| panic!("{group} 那一组算得出键（拒算或没编进来就判不出变没变）"))
    }
}

/// 组成清单里有没有一个单元的限定名以 `::<name>` 收尾（测试目标里的单元，限定名中间带着目标的标记，`<test 测试目标>`）。
fn composition_holds_a_unit_named(composition: &[String], name: &str) -> bool {
    let suffix = format!("::{name}");
    composition.iter().any(|line| {
        line.split('\t')
            .nth(1)
            .is_some_and(|qualified_name| qualified_name.ends_with(&suffix))
    })
}

/// 组成清单里有没有一个单元落在 `file`（从仓根起的路径）。
fn composition_holds_a_unit_in_the_file(composition: &[String], file: &str) -> bool {
    composition.iter().any(|line| {
        line.split('\t')
            .nth(2)
            .is_some_and(|location| location.starts_with(&format!("{file}:")))
    })
}

/// 一组在 `tree` 上照 `units` 那一种算的那一把、它的组成清单与收全闭包的组成清单（[`GroupUnderTheTree`]）。
fn group_under(
    tree: &TreeLoadedForTheKeys,
    request: &EntryGroupRequest,
    units: UnitsOfTheGroup,
) -> GroupUnderTheTree {
    let kept = tree.group_key(request, units);
    let whole = tree.group_key(request, UnitsOfTheGroup::WholeClosure);
    GroupUnderTheTree {
        key: key_of_the_verdict(&kept.verdict),
        kept_composition: kept.composition_lines,
        whole_composition: whole.composition_lines,
    }
}

/// 判器引到的被检测产品代码那一组的请求与它怎么收（只键产品包、固定的 cfg）。
fn judged_product_code_group() -> (EntryGroupRequest, UnitsOfTheGroup) {
    (
        judged_product_code_request(),
        UnitsOfTheGroup::ProductPackagesOnly,
    )
}

/// 登记表里一个观察者那一组的请求与它怎么收（只键产品包、固定的 cfg）；没登记就 panic。
fn observer_group_of(
    observer_code_file: &str,
    observer_name: &str,
) -> (EntryGroupRequest, UnitsOfTheGroup) {
    let group_name = observer_group_name(observer_code_file, observer_name);
    let request = observer_requests()
        .into_iter()
        .find(|request| request.group_name == group_name)
        .unwrap_or_else(|| panic!("OBSERVER_ENTRIES 里登记了 {group_name}"));
    (request, UnitsOfTheGroup::ProductPackagesOnly)
}

/// 在临时拷贝上施加一处改动（文件从仓根起、恰好出现一次的原文、替换文），交施加前与施加后这几组（按给的次序）。
fn groups_before_and_after(
    label: &str,
    groups: &[(EntryGroupRequest, UnitsOfTheGroup)],
    (relative, original, replacement): (&str, &str, &str),
) -> (Vec<GroupUnderTheTree>, Vec<GroupUnderTheTree>) {
    let copy = RepositoryCopy::new(label);
    let groups_under = |root: &Path| {
        let tree = TreeLoadedForTheKeys::load(root);
        groups
            .iter()
            .map(|(request, units)| group_under(&tree, request, *units))
            .collect::<Vec<GroupUnderTheTree>>()
    };
    let before = groups_under(&copy.root);
    copy.replace_exactly_once(relative, original, replacement);
    let after = groups_under(&copy.root);
    (before, after)
}
