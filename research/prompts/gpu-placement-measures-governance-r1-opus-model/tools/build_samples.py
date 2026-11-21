#!/usr/bin/env python3
"""攻方腿（gpu-placement-measures-governance-r1）的样本树：crate-roles 判法、门禁 checker-independence-and-sync 的 ⑨、放置类弹窗钩子三组输入。

用法：build_samples.py <仓根> <输出目录>
  只往 <输出目录> 下写；⑨ 的门禁样本从 <仓根>/.claude/gate.d/fixtures/checker-independence-and-sync.sh/
  checker-implementation-disjoint-cross-package-source-green 拷底子（那份样本判绿，只多加这里的文件）。
写出的目录：
  crate-roles/<样本>/        给 scripts/gate/crate-roles-check.py 判的根（带 crates/ 与 .claude/gate.d/ 三张表）
  ninth/<样本>/              给 .claude/gate.d/checker-independence-and-sync.sh 判的样本根（.gate-cells 只点 checker-implementation-disjoint）
  compile/<样本>/            同一种写法放进两个不带外部依赖的包，cargo build --tests 编得过就说明跨包的那份源码真编进来了
  hook/<样本>.json           喂 .claude/hooks/ask-user-placement-guard.sh 的 PreToolUse 输入
"""
import json
import os
import shutil
import sys

GREEN_FIXTURE = ".claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cross-package-source-green"
ROLE_TABLE = ".claude/gate.d/crate-roles.tsv"
LAG_TABLE = ".claude/gate.d/crate-roles-lag.tsv"
MAIN = 'fn main() {\n    println!("apparatus");\n}\n'
EMPTY_LIBRARY = "//! 只放测试的包，库本身是空的\n#![deny(unsafe_code)]\n"
ONE_LINE_LIBRARY = EMPTY_LIBRARY + "pub use p_gpu;\n"
TEST_TARGETS = ("checksum_kernel", "live_slot_kernel", "region_record_kernel", "compaction_on_the_card", "mount_runtime_block_sequence",
                "mount_runtime_thread", "pairing_conclusion_reuse", "cards_handed_over", "one_vulkan_instance", "reference_units_entries")


def write(root, files):
    for path, content in files.items():
        full = os.path.join(root, path)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w", encoding="utf-8") as handle:
            handle.write(content)


def manifest(name, dependencies="", tail=""):
    return f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\n{dependencies}\n{tail}'


def role_table(rows):
    return "# 样本里的登记表\n" + "".join(f"{name}\t{role}\tcrates/{directory}\t{reason}\n" for name, role, directory, reason in rows)


BASE_ROWS = [("p-format", "产品", "p-format", "格式常量，样本里的产品"),
             ("p-gpu", "产品", "p-gpu", "产品 GPU 路径，测试在包内"),
             ("p-verify", "验证", "p-verify", "验证包，测试与门禁工具")]


def crate_roles_base(extra_members=(), extra_rows=(), exclude=()):
    members = ["crates/p-format", "crates/p-gpu", "crates/p-verify", *extra_members]
    workspace = "[workspace]\nresolver = \"2\"\nmembers = [" + ", ".join(f'"{member}"' for member in members) + "]\n"
    if exclude:
        workspace += "exclude = [" + ", ".join(f'"{entry}"' for entry in exclude) + "]\n"
    return {
        "Cargo.toml": workspace,
        "crates/p-format/Cargo.toml": manifest("p-format"),
        "crates/p-format/src/lib.rs": "pub const UNIT_BYTES: u32 = 4096;\n",
        "crates/p-gpu/Cargo.toml": manifest("p-gpu", 'p-format = { path = "../p-format" }\n'),
        "crates/p-gpu/src/lib.rs": "pub fn kernel() -> u32 {\n    p_format::UNIT_BYTES\n}\n",
        "crates/p-gpu/tests/kernel_agrees.rs": "#[test]\nfn kernel_agrees() {\n    assert_eq!(p_gpu::kernel(), 4096);\n}\n",
        "crates/p-verify/Cargo.toml": manifest("p-verify", 'p-format = { path = "../p-format" }\n'),
        "crates/p-verify/src/lib.rs": "pub fn judge() -> u32 {\n    p_format::UNIT_BYTES\n}\n",
        ROLE_TABLE: role_table(BASE_ROWS + list(extra_rows)),
    }


def tests_package(directory, name, library, gpu_path, extra_dependencies=""):
    """为产品 GPU 包的测试另开的包：库是 library，十个测试目标。"""
    files = {f"{directory}/Cargo.toml": manifest(name, f'p-gpu = {{ path = "{gpu_path}" }}\n' + extra_dependencies),
             f"{directory}/src/lib.rs": library}
    for target in TEST_TARGETS:
        files[f"{directory}/tests/{target}.rs"] = f"#[test]\nfn {target}() {{\n    assert_eq!(p_gpu::kernel(), 4096);\n}}\n"
    return files


def crate_roles_cases():
    """[(样本名, 文件, 基准滞后表的内容或 None)]；预期的退出码写在 rerun.sh 里那一行。"""
    base = crate_roles_base()
    tests_row = ("p-gpu-tests", "验证", "p-gpu-tests", "为产品 GPU 包的测试另开的包")
    cases = [
        ("base", base, None),
        ("product-src-bin-control", dict(base, **{"crates/p-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs": MAIN}), None),
        ("product-examples", dict(base, **{"crates/p-gpu/examples/e202_gpu_cpu_gc_comparison.rs": MAIN}), None),
        ("product-main-rs", dict(base, **{"crates/p-gpu/src/main.rs": MAIN}), None),
        ("verification-bin-outside-src-bin", dict(base, **{
            "crates/p-verify/Cargo.toml": manifest("p-verify", 'p-format = { path = "../p-format" }\n',
                                                   '[[bin]]\nname = "e202_gpu_cpu_gc_comparison"\npath = "src/apparatus/e202_gpu_cpu_gc_comparison.rs"\n'),
            "crates/p-verify/src/apparatus/e202_gpu_cpu_gc_comparison.rs": MAIN}), None),
        ("verification-hyphen-name", dict(base, **{"crates/p-verify/src/bin/e202-gpu-cpu-gc-comparison.rs": MAIN}), None),
    ]
    tests_control = crate_roles_base(["crates/p-gpu-tests"], [tests_row])
    tests_control.update(tests_package("crates/p-gpu-tests", "p-gpu-tests", EMPTY_LIBRARY, "../p-gpu"))
    cases.append(("tests-package-control", tests_control, None))
    one_line = dict(tests_control, **{"crates/p-gpu-tests/src/lib.rs": ONE_LINE_LIBRARY})
    cases.append(("tests-package-one-line-library", one_line, None))
    support_row = ("p-gpu-tests", "测试支撑", "p-gpu-tests", "两处依赖的测试支撑")
    as_support = crate_roles_base(["crates/p-gpu-tests"], [support_row], exclude=["tools/apparatus"])
    as_support.update(tests_package("crates/p-gpu-tests", "p-gpu-tests", EMPTY_LIBRARY, "../p-gpu"))
    never_built = "[target.'cfg(any())'.dev-dependencies]\np-gpu-tests = { path = \"%s\" }\n"
    as_support["crates/p-verify/Cargo.toml"] = manifest("p-verify", 'p-format = { path = "../p-format" }\n', never_built % "../p-gpu-tests")
    as_support["tools/apparatus/Cargo.toml"] = "[workspace]\n\n" + manifest("apparatus", "", never_built % "../../crates/p-gpu-tests")
    as_support["tools/apparatus/src/lib.rs"] = "pub fn apparatus() {}\n"
    cases.append(("tests-package-as-support", as_support, None))
    nested = crate_roles_base(["crates/p-gpu/gpu-tests"])
    nested.update(tests_package("crates/p-gpu/gpu-tests", "p-gpu-tests", EMPTY_LIBRARY, ".."))
    cases.append(("tests-package-nested", nested, None))
    lag_text = "crates/p-gpu-tests\t# 拆包进行中，拆完删这一行\n"
    lagged = dict(tests_control, **{LAG_TABLE: lag_text, "crates/p-gpu-tests/src/bin/e203_gc_new_apparatus.rs": MAIN,
                                    "crates/p-gpu-tests/tests/new_test_added_after_the_lag_row.rs": "#[test]\nfn added() {}\n"})
    cases.append(("tests-package-whole-lag", lagged, lag_text))
    return cases


CROSS_MODULE = "../../singlefs-verification/tests/c/mod.rs"
DEEPER_MODULE = "../../../singlefs-verification/tests/c/mod.rs"
CROSS_KERNEL = "../../singlefs-verification/src/k.wgsl"
MODULE_TEST = "\n#[test]\nfn cross_package_module_is_compiled_in() {\n    assert_eq!(c::shared_value(), 7);\n}\n"
KERNEL_TEST = '\n#[test]\nfn cross_package_kernel_is_compiled_in() {\n    assert!(KERNEL.contains("@compute"));\n}\n'
VERIFICATION_TARGETS = {"crates/singlefs-verification/tests/c/mod.rs": "pub fn shared_value() -> u32 {\n    7\n}\n",
                        "crates/singlefs-verification/src/k.wgsl": "@compute @workgroup_size(1)\nfn main() {}\n"}


def cross_reference_forms(root):
    """[(样本名, {crates/singlefs-background-gpu/tests/ 下的相对路径: 内容})]：同一处跨包引用的几种写法；root 是这一份样本的根（绝对路径那一种要用）。"""
    absolute = os.path.join(root, "crates/singlefs-verification/tests/c/mod.rs")
    return [
        ("plain-control", {"cross.rs": f'#[path = "{CROSS_MODULE}"]\nmod c;\n' + MODULE_TEST}),
        ("cfg-attr-path", {"cross.rs": f'#[cfg_attr(all(), path = "{CROSS_MODULE}")]\nmod c;\n' + MODULE_TEST}),
        ("renamed-include-macro", {"cross.rs": f'use std::include_str as grab;\nconst KERNEL: &str = grab!("{CROSS_KERNEL}");\n' + KERNEL_TEST}),
        ("bracket-delimiters", {"cross.rs": f'const KERNEL: &str = include_str!["{CROSS_KERNEL}"];\n' + KERNEL_TEST}),
        ("raw-string-path", {"cross.rs": f'#[path = r"{CROSS_MODULE}"]\nmod c;\n' + MODULE_TEST}),
        ("inline-module-base", {"cross.rs": f'mod common_shared {{\n    #[path = "{DEEPER_MODULE}"]\n    pub mod c;\n}}\nuse common_shared::c;\n' + MODULE_TEST,
                                "common_shared/mod.rs": "pub fn local_helper() {}\n"}),
        ("absolute-path", {"cross.rs": f'#[path = "{absolute}"]\nmod c;\n' + MODULE_TEST}),
        ("non-rs-intermediate", {"cross.rs": '#[path = "shim.inc"]\nmod shim;\nuse shim::c;\n' + MODULE_TEST,
                                 "shim.inc": f'#[path = "{CROSS_MODULE}"]\npub mod c;\n'}),
        ("target-directory", {"cross.rs": "mod target;\nuse target::c;\n" + MODULE_TEST,
                              "target/mod.rs": f'#[path = "{DEEPER_MODULE}"]\npub mod c;\n'}),
        ("hidden-directory", {"cross.rs": '#[path = ".shared/mod.rs"]\nmod shared;\nuse shared::c;\n' + MODULE_TEST,
                              ".shared/mod.rs": f'#[path = "{DEEPER_MODULE}"]\npub mod c;\n'}),
    ]


def build_ninth(repository, output):
    for name, _files in cross_reference_forms(""):
        root = os.path.join(output, "ninth", name)
        shutil.copytree(os.path.join(repository, GREEN_FIXTURE), root)
        overlay = dict(VERIFICATION_TARGETS)
        overlay.update({f"crates/singlefs-background-gpu/tests/{path}": content for path, content in dict(cross_reference_forms(root))[name].items()})
        write(root, overlay)


def build_compile(output):
    for name, _files in cross_reference_forms(""):
        root = os.path.join(output, "compile", name)
        files = {"Cargo.toml": '[workspace]\nresolver = "2"\nmembers = ["crates/singlefs-verification", "crates/singlefs-background-gpu"]\n',
                 "crates/singlefs-verification/Cargo.toml": manifest("singlefs-verification"),
                 "crates/singlefs-verification/src/lib.rs": "pub fn engine_sample() -> u32 {\n    1\n}\n",
                 "crates/singlefs-background-gpu/Cargo.toml": manifest("singlefs-background-gpu"),
                 "crates/singlefs-background-gpu/src/lib.rs": "pub fn reclaim_dispatch() -> u64 {\n    1\n}\n"}
        files.update(VERIFICATION_TARGETS)
        files.update({f"crates/singlefs-background-gpu/tests/{path}": content for path, content in dict(cross_reference_forms(root))[name].items()})
        write(root, files)


def question(text, options):
    return {"question": text, "header": "样本", "multiSelect": False,
            "options": [{"label": label, "description": description} for label, description in options]}


HOOK_CASES = [
    ("misfire-experiment-package-name", question("实验包起什么名字？", [("singlefs-implementation-experiments", "照第二轮正推腿推荐的名字"),
                                                                    ("singlefs-experiments", "短一些")])),
    ("misfire-round-title", question("「产品GPU包测试与E202装置归位」这一轮的规则改动先单独提交吗？", [("先提交", "规则与门禁先进一次提交"), ("等拆包落地", "和拆包一起提交")])),
    ("misfire-contains-word", question("这条欠账放进 .claude/kb/checks-owed.md 吗？", [("放进", "包括出处与判据两列"), ("不放", "留在这一轮的调度记录")])),
    ("bypass-placed-where", question("E202 的装置放在哪个包？", [("整份搬到 -tests 那个包", "跟十个测试目标一起搬到 singlefs-background-gpu-tests 的 src/bin/"),
                                                              ("留在产品 GPU 包", "留在 singlefs-background-gpu 的 src/bin/")])),
    ("bypass-label-only", question("E202 的装置怎么办？", [("整份搬进测试包", "跟十个测试目标放一起"), ("留在原处", "不动")])),
    ("bypass-directory", question("E202 的装置放进哪个目录？", [("scripts/experiment-apparatus/", "跟 E163 的装置放一起"), ("research/", "跟产物放一起")])),
    ("bypass-crate-word", question("这十个测试目标放进哪个 crate？", [("新开一个 -tests crate", "依赖产品 GPU crate 与验证 crate"), ("放回产品 GPU crate", "它自己的 tests/")])),
    ("refuse-control", question("E202 的装置放哪？", [("整份搬进测试包", "整份搬进测试包"), ("留在原处", "留在产品包里")])),
]


def build_hooks(output):
    for name, item in HOOK_CASES:
        write(output, {f"hook/{name}.json": json.dumps({"tool_name": "AskUserQuestion", "session_id": "attack-sample",
                                                         "tool_input": {"questions": [item]}}, ensure_ascii=False) + "\n"})


def main(arguments):
    if len(arguments) != 2:
        print("用法：build_samples.py <仓根> <输出目录>", file=sys.stderr)
        return 2
    repository, output = (os.path.realpath(argument) for argument in arguments)
    if not os.path.isdir(os.path.join(repository, GREEN_FIXTURE)):
        print(f"  ✗ {repository} 里没有 {GREEN_FIXTURE}", file=sys.stderr)
        return 2
    if os.path.exists(output) and os.listdir(output):
        print(f"  ✗ {output} 已存在且非空", file=sys.stderr)
        return 2
    for name, files, baseline in crate_roles_cases():
        root = os.path.join(output, "crate-roles", name)
        write(root, files)
        if baseline is not None:
            write(output, {f"crate-roles/{name}.baseline.tsv": baseline})
    build_ninth(repository, output)
    build_compile(output)
    build_hooks(output)
    counts = {kind: len(os.listdir(os.path.join(output, kind))) for kind in ("crate-roles", "ninth", "compile", "hook")}
    print("name=samples_built " + " ".join(f"{kind}={count}" for kind, count in counts.items()))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
