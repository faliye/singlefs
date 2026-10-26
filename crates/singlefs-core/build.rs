//! 把这一次编译里 cargo 给本包打开的特性交给直接依赖本包的构建脚本：`links` 的元数据，依赖方读 `DEP_SINGLEFS_CORE_ENABLED_FEATURES`
//! （`singlefs_unit_graph::build_features::features_enabled_on_the_dependencies`，键名是那里的 `ENABLED_FEATURES_METADATA_KEY`）。
//! cargo 解出来的特性是并过之后的：别的包在同一次构建里替本包打开的特性（harness 档的 dev 依赖打开的 `lookup-work-counters`）
//! 也在里面，依赖方自己的特性看不到它们。checker 档的构建脚本拿这一份算判法单元键。同一个值另经 `cargo::rustc-env` 编进本包
//! （`transaction::lookup_work::ENABLED_FEATURES_REPORTED_BY_THE_BUILD_SCRIPT`），测试拿它判「cargo 真开着时报开着」。
//!
//! 取法：cargo 编本包的构建脚本时带着本包这一次开了的特性（每个开着的特性一个 `--cfg feature="<名>"`，与编本包的库同一份），
//! 所以 `cfg!(feature = "<名>")` 就是 cargo 这一次真开没开；特性一变，构建脚本按另一份 cfg 重编、重跑。不读进程环境里的
//! `CARGO_FEATURE_*`：那是 cargo 设的，外面带进来的同名变量（`CARGO_FEATURE_LOOKUP_WORK_COUNTERS=1` 而不开特性）它也照收，
//! 分不出来（七·6 后续批代码轮第一轮判决第二节 T4）。同仓 `Cargo.lock` 里的 `librocksdb-sys` 构建脚本按同一个取法读它的特性
//! （`if cfg!(feature = "snappy")`）。
//!
//! 这个取法差一种编法（七·6 后续批代码轮第二轮判决第二节 T4 ①，用户 2026-10-23 定并进）：带 `--target` 时 `RUSTFLAGS` 里的
//! `--cfg feature="<名>"` 只给编库的那一次、不给编构建脚本的那一次，库按开着编、这里的 cfg 却说没开。cargo 交给构建脚本的
//! `CARGO_ENCODED_RUSTFLAGS` 是编库那一份参数（本机 cargo 1.98.0 带 `--target` 量过：构建脚本读到 `--cfg` 与 `feature="…"` 两项，
//! 自己的 cfg 是假），所以那里点名打开、这里的 cfg 却说没开的特性，构建当场停下、说出是哪一个，不交一份与编出来的库不符的元数据。
//!
//! 只做交出与这一判：不生成代码，值是开着的特性名写进 cargo 环境变量名的样子（大写、`-` 换成 `_`），按清单里的名字排好、逗号隔开。
//! 清单 `[features]` 里的每个特性都要在 [`FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG`] 里有一行：`singlefs-unit-graph` 的
//! `build_features` 单测把两边逐个比（`the_core_build_script_lists_every_feature_of_its_manifest`）。

/// 本包清单 `[features]` 里的每个特性（清单里的写法，按名字排好）与 cargo 这一次开没开它。
/// `pub(crate)`：harness 档的 `publish_lookup_by_identity` 与 `singlefs-unit-graph` 的单测经 `#[path]` 把这份构建脚本编成一个模块，
/// 要读它与下面两个函数。
pub(crate) const FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG: [(&str, bool); 2] = [
    (
        "lookup-work-counters",
        cfg!(feature = "lookup-work-counters"),
    ),
    (
        "test-only-publish-thresholds",
        cfg!(feature = "test-only-publish-thresholds"),
    ),
];

/// cargo 交给构建脚本的、编本包的 rustc 额外参数：参数之间用 0x1f 隔开。
const ENCODED_RUSTFLAGS_VARIABLE: &str = "CARGO_ENCODED_RUSTFLAGS";

/// [`ENCODED_RUSTFLAGS_VARIABLE`] 里参数之间的分隔符（ASCII 单元分隔符）。
const ENCODED_RUSTFLAGS_SEPARATOR: char = '\u{1f}';

/// 交去编本包的那一行环境变量名（`cargo::rustc-env`）。
const ENABLED_FEATURES_RUSTC_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_CORE_ENABLED_FEATURES_REPORTED_BY_THE_BUILD_SCRIPT";

/// 交出去的那一行的值：`features_with_their_cfg` 里开着的那几个，写成 cargo 环境变量名的样子（大写、`-` 换成 `_`），逗号隔开。
/// 只看 `features_with_their_cfg`，不读进程环境：外面带进来的 `CARGO_FEATURE_*` 不算开着。
pub(crate) fn enabled_features_metadata_value(features_with_their_cfg: &[(&str, bool)]) -> String {
    features_with_their_cfg
        .iter()
        .filter(|(_, is_enabled_by_cargo)| *is_enabled_by_cargo)
        .map(|(feature, _)| feature.to_ascii_uppercase().replace('-', "_"))
        .collect::<Vec<String>>()
        .join(",")
}

/// `encoded_rustflags`（[`ENCODED_RUSTFLAGS_VARIABLE`] 的值）里经 `--cfg feature="<名>"`（`--cfg` 与它的值分两项，或写成
/// `--cfg=feature="<名>"` 一项；等号两边与引号里外的空白不算）点名打开、而 `features_with_their_cfg` 说这一次没开的那几个特性，
/// 按表的次序。表里没有的名字不管（`RUSTFLAGS` 给每个包都带，别的包的特性也在里面）。
pub(crate) fn features_opened_by_the_rustflags_but_off_in_this_build_script<'table>(
    encoded_rustflags: &str,
    features_with_their_cfg: &[(&'table str, bool)],
) -> Vec<&'table str> {
    let arguments: Vec<&str> = encoded_rustflags
        .split(ENCODED_RUSTFLAGS_SEPARATOR)
        .filter(|argument| !argument.is_empty())
        .collect();
    let mut feature_names_opened_by_the_rustflags: Vec<String> = Vec::new();
    // 迭代上界是参数个数；跨轮携带的是下一项是不是 `--cfg` 的值（只看紧跟在 `--cfg` 后面的那一项）。
    let mut is_the_value_of_a_cfg_flag = false;
    for argument in arguments {
        let cfg_value = if is_the_value_of_a_cfg_flag {
            is_the_value_of_a_cfg_flag = false;
            Some(argument)
        } else if argument == "--cfg" {
            is_the_value_of_a_cfg_flag = true;
            None
        } else {
            argument.strip_prefix("--cfg=")
        };
        let Some(cfg_value) = cfg_value else {
            continue;
        };
        let Some((name, value)) = cfg_value.split_once('=') else {
            continue;
        };
        if name.trim() == "feature" {
            feature_names_opened_by_the_rustflags.push(value.trim().trim_matches('"').to_string());
        }
    }
    features_with_their_cfg
        .iter()
        .filter(|(feature, is_enabled_by_cargo)| {
            !*is_enabled_by_cargo
                && feature_names_opened_by_the_rustflags
                    .iter()
                    .any(|opened| opened.as_str() == *feature)
        })
        .map(|(feature, _)| *feature)
        .collect()
}

fn main() {
    // cargo 跑构建脚本时恒设它（没有额外参数时是空串）；不是 cargo 起的、没设的当作没有额外参数。
    let encoded_rustflags = match std::env::var(ENCODED_RUSTFLAGS_VARIABLE) {
        Ok(value) => value,
        Err(std::env::VarError::NotPresent) => String::new(),
        Err(std::env::VarError::NotUnicode(value)) => {
            panic!("{ENCODED_RUSTFLAGS_VARIABLE} 不是 UTF-8：{value:?}")
        }
    };
    let disagreeing = features_opened_by_the_rustflags_but_off_in_this_build_script(
        &encoded_rustflags,
        &FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG,
    );
    assert!(
        disagreeing.is_empty(),
        "singlefs-core：特性 {disagreeing:?} 由 {ENCODED_RUSTFLAGS_VARIABLE} 里的 --cfg feature=… 给编库的那一次打开了，编构建脚本的这一次却没开\
         （带 --target 时 RUSTFLAGS 不给构建脚本）：照这里的 cfg 交出去的特性与编出来的库不符，依赖方的判法单元键会按没开算。\
         出路：用 cargo 的 --features 打开它，不经 RUSTFLAGS 的 --cfg"
    );
    println!(
        "cargo::metadata=enabled_features={}",
        enabled_features_metadata_value(&FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG)
    );
    println!(
        "cargo::rustc-env={ENABLED_FEATURES_RUSTC_ENVIRONMENT_VARIABLE}={}",
        enabled_features_metadata_value(&FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG)
    );
    println!("cargo::rerun-if-changed=build.rs");
}
