use super::*;

#[test]
fn the_verdict_library_directory_is_only_the_one_the_caller_gives() {
    let refusal = verdict_library_directory_given_by(None).expect_err("没设变量时不落任何缺省目录");
    for named in [
        LIBRARY_ENVIRONMENT_VARIABLE,
        "scripts/gate/admission.py library-files",
        "scripts/experiment-apparatus/e182-single-device-long-run.sh",
        "scripts/experiment-apparatus/e182-single-device-units-kept-long-run.sh",
    ] {
        assert!(refusal.contains(named), "拒绝里点名出路 {named}：{refusal}");
    }
    assert!(
        verdict_library_directory_given_by(Some("")).is_err(),
        "设成空串与没设一样拒"
    );
    assert_eq!(
        verdict_library_directory_given_by(Some("/transit/e182-single-device")),
        Ok(std::path::PathBuf::from("/transit/e182-single-device")),
        "给了就用给的那一个"
    );
}

#[test]
fn stored_shard_is_read_back_only_under_the_same_key() {
    // 守卫认领临时目录：用例做完、断言失败、panic 都删掉它
    let directory = singlefs_verification::temporary_files::TemporaryPathRemovedOnDrop::claiming(
        std::env::temp_dir().join(format!("e182-library-test-{}", std::process::id())),
    );
    std::fs::create_dir_all(&*directory).expect("建得起临时目录");
    let fingerprint = ClosureFingerprint {
        key_hex: "ab".repeat(32),
        toolchain_lines: vec!["rustc 1".to_string()],
    };
    let key = shard_key_text(&fingerprint, "history-a", 3, Expansion::FullSubsets);
    let mut counters = Counters::default();
    counters.add("states", 9);
    store_shard(&directory, &key, &counters).expect("存得进");
    assert_eq!(read_stored_shard(&directory, &key), Some(counters));
    for other_key in [
        shard_key_text(&fingerprint, "history-b", 3, Expansion::FullSubsets),
        shard_key_text(&fingerprint, "history-a", 4, Expansion::FullSubsets),
        shard_key_text(&fingerprint, "history-a", 3, Expansion::FinalState),
    ] {
        assert_eq!(
            read_stored_shard(&directory, &other_key),
            None,
            "键里任何一段不同都取不到"
        );
    }
}

/// 合成树的根 `Cargo.toml`：装工作区时照它认两个成员。
const SYNTHETIC_ROOT_MANIFEST: &str =
    "[workspace]\nresolver = \"2\"\nmembers = [\"crates/judging\", \"crates/singlefs-core\"]\n";
/// 合成树的 `Cargo.lock`：两个都是路径包，没有外部依赖要按闭包收窄。
const SYNTHETIC_LOCK_FILE: &str = "version = 3\n\n[[package]]\nname = \"judging\"\nversion = \"0.1.0\"\n\
        dependencies = [\n \"singlefs-core\",\n]\n\n[[package]]\nname = \"singlefs-core\"\nversion = \"0.1.0\"\n";
/// 合成树里的依赖包：包名与两个特性名同核心层（编进来的判器键表与 `features_the_core_layer_was_compiled_with` 点名的就是这个包、这两个特性），
/// 计数那个函数挂在 `lookup-work-counters` 上。
const SYNTHETIC_CORE_MANIFEST: &str = "[package]\nname = \"singlefs-core\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\
        links = \"singlefs-core\"\n\n[features]\nlookup-work-counters = []\ntest-only-publish-thresholds = []\n";
const SYNTHETIC_CORE_LIBRARY: &str = "#[cfg(feature = \"lookup-work-counters\")]\n\
        pub fn count_one_lookup(lookups_counted: &mut u64) {\n    *lookups_counted += 1;\n}\n\n\
        #[cfg(not(feature = \"lookup-work-counters\"))]\npub fn count_one_lookup(_lookups_counted: &mut u64) {}\n";
/// 合成树里的入口包：普通依赖核心层、不要它的特性，入口经调用引到计数那个函数。
const SYNTHETIC_JUDGING_MANIFEST: &str =
    "[package]\nname = \"judging\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
        [dependencies]\nsinglefs-core = { path = \"../singlefs-core\" }\n";
const SYNTHETIC_JUDGING_LIBRARY: &str =
    "pub fn judge_one_image() -> u64 {\n    let mut lookups_counted = 0;\n    \
        singlefs_core::count_one_lookup(&mut lookups_counted);\n    lookups_counted\n}\n";
/// 合成树里闭包的根：入口包库目标里的那一个入口。
const SYNTHETIC_JUDGING_ENTRIES: ClosureEntries = ClosureEntries {
    group_name: "synthetic_judgement",
    target: "lib",
    entry_functions: &["judging::judge_one_image"],
};

/// 同一进程里并行跑的几条用例各用一棵合成树：根的名字里带进程号与这个序号。
static SYNTHETIC_TREE_SERIAL: AtomicU64 = AtomicU64::new(0);

/// 临时目录里现造的一棵最小工作区（上面几份常量），用完删掉：用例做完、断言失败、panic 都删，写到一半 panic 留下的半棵也删。
struct SyntheticTree {
    root: std::path::PathBuf,
}

impl SyntheticTree {
    /// 先认领根、再写：写到一半 panic 时 Drop 删掉写了的那一半。
    fn new() -> Self {
        let tree = SyntheticTree {
            root: std::env::temp_dir().join(format!(
                "singlefs-e182-closure-fingerprint-tree-{}-{}",
                std::process::id(),
                SYNTHETIC_TREE_SERIAL.fetch_add(1, Ordering::SeqCst)
            )),
        };
        for (relative_path, text) in [
            ("Cargo.toml", SYNTHETIC_ROOT_MANIFEST),
            ("Cargo.lock", SYNTHETIC_LOCK_FILE),
            ("crates/singlefs-core/Cargo.toml", SYNTHETIC_CORE_MANIFEST),
            ("crates/singlefs-core/src/lib.rs", SYNTHETIC_CORE_LIBRARY),
            ("crates/judging/Cargo.toml", SYNTHETIC_JUDGING_MANIFEST),
            ("crates/judging/src/lib.rs", SYNTHETIC_JUDGING_LIBRARY),
        ] {
            let path = tree.root.join(relative_path);
            std::fs::create_dir_all(path.parent().expect("合成树的文件都在根之下"))
                .expect("临时目录写得了");
            std::fs::write(&path, text).expect("临时目录写得了");
        }
        tree
    }
}

impl Drop for SyntheticTree {
    /// 删不掉只打一行，不在 Drop 里 panic（panic 途中再 panic 会直接中止进程）。
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.root) {
            if error.kind() != std::io::ErrorKind::NotFound {
                eprintln!("删不掉自己建的合成树 {}：{error}", self.root.display());
            }
        }
    }
}

/// 场景：现造的合成树里，入口包经调用引到依赖包 `singlefs-core` 里挂在 `lookup-work-counters` 上的函数；照「依赖包没并上特性」与
/// 「并上 `lookup-work-counters`」两份依赖包特性各算一把闭包指纹。预期：两把不同，工具链行相同（闭包指纹照给定的依赖包特性算，不只看本包特性）。
/// 两把各在一个线程里算（互不依赖）。
/// 判别力（`crates/mutations.tsv`）：算指纹改回只照本包特性（给定的依赖包特性不交给单元图），两把相同，这条红。
#[test]
fn the_closure_key_follows_the_features_unified_onto_the_core_layer() {
    let tree = SyntheticTree::new();
    let features_with_counters = singlefs_unit_graph::FeaturesEnabledByTheBuild {
        features_of_each_package: BTreeMap::from([(
            singlefs_verification::crash_identity::CORE_LAYER_PACKAGE_NAME.to_string(),
            BTreeSet::from([
                singlefs_verification::crash_identity::LOOKUP_WORK_COUNTERS_FEATURE.to_string(),
            ]),
        )]),
    };
    let features_without_counters = singlefs_unit_graph::FeaturesEnabledByTheBuild::default();
    let (without_counters, with_counters) = std::thread::scope(|scope| {
        let without_counters = scope.spawn(|| {
            closure_fingerprint_with_the_features_of_the_build(
                &tree.root,
                &SYNTHETIC_JUDGING_ENTRIES,
                &features_without_counters,
            )
        });
        let with_counters = scope.spawn(|| {
            closure_fingerprint_with_the_features_of_the_build(
                &tree.root,
                &SYNTHETIC_JUDGING_ENTRIES,
                &features_with_counters,
            )
        });
        (
            without_counters
                .join()
                .expect("算没并上特性那一把的线程不 panic"),
            with_counters
                .join()
                .expect("算并上计数那一把的线程不 panic"),
        )
    });
    let without_counters = without_counters.expect("依赖包没并上特性时合成树入口的闭包算得出指纹");
    let with_counters = with_counters.expect("依赖包并上计数时合成树入口的闭包算得出指纹");
    assert_ne!(
        without_counters.key_hex, with_counters.key_hex,
        "依赖包并上计数与没并上，闭包指纹两把"
    );
    assert_eq!(
        without_counters.toolchain_lines, with_counters.toolchain_lines,
        "两把只差在依赖包特性上，工具链行相同"
    );
}

/// 场景：同一棵合成树上，比装置跑时取的闭包指纹（[`closure_fingerprint_of_the_tree`]，依赖包特性取编进库的那份判器键表）与照这个二进制
/// 链进来的核心层实际开着的特性（`features_the_core_layer_was_compiled_with`，从核心层编出来的代码里认）算的那一把，再算一把依赖包没并上特性的。
/// 预期：前两把相同；编实验包的测试时 dev 依赖替核心层并上 `lookup-work-counters`，合成树的入口经调用引到挂在它上面的函数，
/// 所以没并上特性的那一把与前两把不同（这条用例在这次编译里有判别力的前提）。三把各在一个线程里算（互不依赖）。
/// 判别力（`crates/mutations.tsv`，编测试的那一行）：跑时取指纹改回不带依赖包特性，跑时那一把等于没并上特性的那一把，这条红。
#[test]
fn the_closure_key_is_computed_with_the_features_this_build_unified_onto_the_core_layer() {
    let tree = SyntheticTree::new();
    let features_the_core_layer_was_built_with =
        singlefs_verification::crash_identity::features_the_core_layer_was_compiled_with();
    let features_without_any = singlefs_unit_graph::FeaturesEnabledByTheBuild::default();
    let (taken_at_run_time, with_the_features_the_core_layer_was_built_with, without_any) =
        std::thread::scope(|scope| {
            let taken_at_run_time = scope
                .spawn(|| closure_fingerprint_of_the_tree(&tree.root, &SYNTHETIC_JUDGING_ENTRIES));
            let with_the_features_the_core_layer_was_built_with = scope.spawn(|| {
                closure_fingerprint_with_the_features_of_the_build(
                    &tree.root,
                    &SYNTHETIC_JUDGING_ENTRIES,
                    &features_the_core_layer_was_built_with,
                )
            });
            let without_any = scope.spawn(|| {
                closure_fingerprint_with_the_features_of_the_build(
                    &tree.root,
                    &SYNTHETIC_JUDGING_ENTRIES,
                    &features_without_any,
                )
            });
            (
                taken_at_run_time
                    .join()
                    .expect("算跑时那一把的线程不 panic"),
                with_the_features_the_core_layer_was_built_with
                    .join()
                    .expect("算照核心层开着的特性那一把的线程不 panic"),
                without_any
                    .join()
                    .expect("算没并上特性那一把的线程不 panic"),
            )
        });
    let taken_at_run_time = taken_at_run_time.expect("跑时取的闭包指纹算得出");
    let with_the_features_the_core_layer_was_built_with =
        with_the_features_the_core_layer_was_built_with
            .expect("照链进来的核心层开着的特性算得出闭包指纹");
    let without_any = without_any.expect("依赖包没并上特性时算得出闭包指纹");
    assert_ne!(
        with_the_features_the_core_layer_was_built_with.key_hex, without_any.key_hex,
        "编测试时核心层并上了计数，合成树的入口引到计数那个函数，照开着的特性算的与没并上的两把不同"
    );
    assert_eq!(
        taken_at_run_time, with_the_features_the_core_layer_was_built_with,
        "跑时取的闭包指纹照的是这一次编译链进来的核心层实际开着的特性"
    );
}
