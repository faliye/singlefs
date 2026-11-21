#!/usr/bin/env python3
"""攻方腿探针：门禁 checker-independence-and-sync 的 crate-cross-references 格的判法（scripts/gate/crate-cross-references-check.py）
在真 cargo 写的 dep-info 上，两件事各判成什么：
  复用键：同一棵小工作区，只换不进键的生效构建环境（RUSTFLAGS、工作区上一层的 .cargo/config.toml、家目录里的 .cargo/config.toml、
    CARGO_PROFILE_<名>_DEBUG_ASSERTIONS），判法照库里那一条（无环境那一趟的事实）算出的键与库里那把键同不同（同就复用）、
    这一趟真编出来的事实与库里那一条同不同、判红的引用有几处；
  分类：几种把别的包的源码编进来的写法（.md 后缀当模块、仓里不属任何包的 .inc、构建脚本拷进 OUT_DIR 再 include!）与读别的包的 .json 数据各落哪一类。
只调判法里的纯函数（metadata_packages、toolchain_lines、recorded_package_facts、package_key、Layout），不连判定库、不写真服务。
用法：python3 key_and_classify_probe.py <冻结树根> <运行目录>
编译目录：环境里有 CARGO_TARGET_DIR（任务服务给的）就在它底下按趟分子目录，没有就放 <运行目录>/targets。"""
import importlib.util
import json
import os
import shutil
import subprocess
import sys

root_repo, work = sys.argv[1], os.path.realpath(sys.argv[2])
specification = importlib.util.spec_from_file_location("crate_cross_references_probe", os.path.join(root_repo, "scripts", "gate", "crate-cross-references-check.py"))
judge = importlib.util.module_from_spec(specification)
specification.loader.exec_module(judge)

FLAG = "attack_probe_flag"
FILES = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/alpha", "crates/beta", "crates/gamma", "crates/delta"]\nresolver = "2"\n',
    "shared/snippet.inc": "pub fn from_unpackaged_inc() {}\n",
    "crates/beta/Cargo.toml": '[package]\nname = "beta"\nversion = "0.1.0"\nedition = "2021"\n',
    "crates/beta/src/lib.rs": "pub fn b() {}\n",
    "crates/beta/src/x.rs": "pub fn stolen() {}\n",
    "crates/beta/src/code.md": "pub fn from_markdown() {}\n",
    "crates/beta/data/sample.json": '{"sample": 1}\n',
    "crates/alpha/Cargo.toml": '[package]\nname = "alpha"\nversion = "0.1.0"\nedition = "2021"\n',
    "crates/alpha/src/lib.rs": ("#![allow(unexpected_cfgs)]\n"
                                f"#[cfg({FLAG})]\n#[path = \"../../beta/src/x.rs\"]\nmod stolen_under_flag;\n"
                                "#[cfg(not(debug_assertions))]\n#[path = \"../../beta/src/x.rs\"]\nmod stolen_without_debug_assertions;\n"
                                "pub fn a() {}\n"),
    "crates/gamma/Cargo.toml": '[package]\nname = "gamma"\nversion = "0.1.0"\nedition = "2021"\n',
    "crates/gamma/build.rs": 'fn main() {\n    println!("cargo:rerun-if-changed=build.rs");\n    println!("cargo:rerun-if-env-changed=HOME");\n}\n',
    "crates/gamma/src/lib.rs": ("#![allow(unexpected_cfgs)]\n"
                                f"#[cfg({FLAG})]\n#[path = \"../../beta/src/x.rs\"]\nmod stolen_under_flag;\n"
                                "pub fn g() {}\n"),
    "crates/delta/Cargo.toml": '[package]\nname = "delta"\nversion = "0.1.0"\nedition = "2021"\n',
    "crates/delta/build.rs": ('fn main() {\n    let out = std::env::var("OUT_DIR").unwrap();\n'
                              '    println!("cargo:rerun-if-changed=../beta/src/x.rs");\n'
                              '    std::fs::copy("../beta/src/x.rs", format!("{out}/copied.rs")).unwrap();\n}\n'),
    "crates/delta/src/lib.rs": ('#[path = "../../beta/src/code.md"]\nmod from_markdown;\n'
                                'include!("../../../shared/snippet.inc");\n'
                                'pub const SAMPLE: &str = include_str!("../../beta/data/sample.json");\n'
                                'include!(concat!(env!("OUT_DIR"), "/copied.rs"));\n'),
}
CONFIG_WITH_FLAG = f'[build]\nrustflags = ["--cfg", "{FLAG}"]\n'


def write_tree(base, extra=None):
    for relative, text in list(FILES.items()) + list((extra or {}).items()):
        path = os.path.join(base, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
    subprocess.run(["git", "init", "-q", base], check=True)


original_home = os.environ.get("HOME", "")
rustup_home = os.environ.get("RUSTUP_HOME") or os.path.join(original_home, ".rustup")
target_base = os.environ.get("CARGO_TARGET_DIR") or os.path.join(work, "targets")
plain_root = os.path.join(work, "plain", "workspace")
ancestor_root = os.path.join(work, "ancestor", "workspace")
write_tree(plain_root)
write_tree(ancestor_root)
os.makedirs(os.path.join(work, "ancestor", ".cargo"), exist_ok=True)
with open(os.path.join(work, "ancestor", ".cargo", "config.toml"), "w", encoding="utf-8") as handle:
    handle.write(CONFIG_WITH_FLAG)
empty_home = os.path.join(work, "home-empty")
configured_home = os.path.join(work, "home-with-config")
os.makedirs(empty_home, exist_ok=True)
os.makedirs(os.path.join(configured_home, ".cargo"), exist_ok=True)
with open(os.path.join(configured_home, ".cargo", "config.toml"), "w", encoding="utf-8") as handle:
    handle.write(CONFIG_WITH_FLAG)

base_environment = {name: value for name, value in os.environ.items()
                    if name not in ("CARGO_TARGET_DIR", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_HOME")
                    and not name.startswith("CARGO_PROFILE_")}
base_environment["RUSTUP_HOME"] = rustup_home
base_environment["HOME"] = empty_home
# (趟名, 根, 这一趟比无环境那一趟多出的环境)
RUNS = (
    ("plain", plain_root, {}),
    ("rustflags-environment", plain_root, {"RUSTFLAGS": f"--cfg {FLAG}"}),
    ("ancestor-cargo-config", ancestor_root, {}),
    ("home-cargo-config", plain_root, {"HOME": configured_home}),
    ("debug-assertions-off", plain_root, {"CARGO_PROFILE_DEV_DEBUG_ASSERTIONS": "false", "CARGO_PROFILE_TEST_DEBUG_ASSERTIONS": "false"}),
)
results = {}
for run_name, root, extra in RUNS:
    environment = dict(base_environment, **extra)
    environment["CARGO_TARGET_DIR"] = os.path.join(target_base, f"attack-probe-{run_name}")
    run = judge.Run(root, environment)
    cargo = shutil.which("cargo", path=environment.get("PATH", ""))
    toolchain = judge.toolchain_lines(run, cargo)
    host = judge.host_of(toolchain)
    packages, judged = judge.metadata_packages(run, cargo, host)
    completed = subprocess.run([cargo, *judge.CHECK_ARGUMENTS], cwd=root, env=environment, capture_output=True, text=True)
    facts, missing = judge.recorded_package_facts(completed.stdout, packages, root, set(judged))
    print(f"name=compile_{run_name} cargo_exit={completed.returncode} packages={len(judged)} missing_dep_info={sum(len(value) for value in missing.values())} problems={len(run.problems)}")
    if completed.returncode != 0:
        print("  " + "\n  ".join(completed.stderr.strip().split("\n")[-8:]))
    results[run_name] = (root, environment, packages, judged, facts, toolchain)

plain_root_used, plain_environment, plain_packages, plain_judged, plain_facts, plain_toolchain = results["plain"]
for run_name, (root, environment, packages, judged, facts, toolchain) in results.items():
    layout = judge.Layout(root, packages, judged)
    plain_layout = judge.Layout(plain_root_used, plain_packages, plain_judged)
    for package in judged:
        stored_key = judge.package_key(plain_root_used, package, plain_facts[package], "probe-judge", plain_toolchain, plain_environment)[0]
        this_key, components = judge.package_key(root, package, plain_facts[package], "probe-judge", toolchain, environment)
        reused = this_key == stored_key
        same_facts = judge.fact_digest(facts[package]) == judge.fact_digest(plain_facts[package])
        red_now = len(judge.reference_keys(package, facts[package], layout))
        red_reused = len(judge.reference_keys(package, plain_facts[package], plain_layout))
        verdict = ("false-reuse" if reused and not same_facts else "sound-reuse" if reused else "rejudge")
        print(f"name=key_{run_name}_{package.replace('/', '_')} reuse={'yes' if reused else 'no'} facts_same={'yes' if same_facts else 'no'} "
              f"red_if_judged_now={red_now} red_from_the_reused_fact={red_reused} verdict={verdict}")

layout = judge.Layout(plain_root_used, plain_packages, plain_judged)
fact = plain_facts["crates/delta"]
for path in sorted({path for unit in fact["units"] for path in unit["sources"]}):
    category = layout.classify(path, "crates/delta")
    print(f"name=classify_delta path={path} category={category} red={'yes' if category in judge.RED_CATEGORIES else 'no'}")
print(f"name=classify_delta_build_inputs inputs={','.join(fact['build']['inputs'])} classified=no")
