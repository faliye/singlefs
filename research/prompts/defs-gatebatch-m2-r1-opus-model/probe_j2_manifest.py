"""J2 探针：崩溃枚举用例按排除法算输入，减多了（改了用例会读的文件，指纹不变）与减少了（改了用例读不到的文件，指纹照变）。
复跑：python3 probe_j2_manifest.py [--admission <admission.py>]
第一部分在 admission.py 自证那个小仓上造改动；第二部分把真仓 crates/ 与 Cargo 清单、锁、登记表按 git 列得出的文件拷进临时仓，算四条登记用例的指纹。"""
import os
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import probe_common  # noqa: E402

ADMISSION = probe_common.option("--admission", probe_common.DEFAULT_ADMISSION)
module = probe_common.load_admission(ADMISSION)
cells = probe_common.Cells("J2 排除法")
OWN_ROW = "crash-case:own\tcrates/ Cargo.toml\ttest=pkg:own_case:the_case count-line=LAYER0 exhaustive=LAYER0\t# 样本"


def fingerprint_in(work, key, environment):
    manifest = os.path.join(work, ".probe-manifest.out")
    exit_code, output, messages = module.run_in_environment(
        [sys.executable, ADMISSION, "crash-case-manifest", work, key, manifest, "--toolchain", "--build-environment"], environment)
    parts = output.split()
    return (parts[0], int(parts[1]), int(parts[2])) if exit_code == 0 and len(parts) == 3 else (f"退 {exit_code}：{output.strip()} {messages.strip()}", -1, -1)


def toolchain_environment(work):
    tools = os.path.join(work, ".tools")
    for name, script in module.FAKE_TOOLCHAIN_SCRIPTS.items():
        module.write_executable(os.path.join(tools, name), script)
    return module.environment_without_build_settings({"PATH": tools + os.pathsep + os.environ.get("PATH", ""),
                                                      "CARGO_HOME": os.path.join(tools, "cargo-home")})


def changed_by(work, key, environment, relative, new_text):
    """改一个文件前后这条用例的指纹：返回 (改前, 改后)，改完照原样写回。"""
    path = os.path.join(work, relative)
    original = open(path, encoding="utf-8").read() if os.path.exists(path) else None
    before = fingerprint_in(work, key, environment)[0]
    module.write_text(path, new_text(original or ""))
    after = fingerprint_in(work, key, environment)[0]
    if original is None:
        os.remove(path)
    else:
        module.write_text(path, original)
    return before, after


small = tempfile.mkdtemp(prefix="opus-j2-small-")
try:
    module.build_crash_case_repository(small, [OWN_ROW])
    module.write_text(os.path.join(small, ".gitignore"), ".tools/\n*.out\n")
    environment = toolchain_environment(small)
    own_case = "crates/pkg/tests/own_case.rs"
    base_text = open(os.path.join(small, own_case), encoding="utf-8").read()
    edit_other = lambda text: text + "#[test]\nfn other_changed() { assert_eq!(1 + 1, 3); }\n"  # noqa: E731

    module.write_text(os.path.join(small, own_case), base_text + 'include!(concat!("other_", "target.rs"));\n')
    before, after = changed_by(small, "crash-case:own", environment, "crates/pkg/tests/other_target.rs", edit_other)
    cells.expect("ATTACK", "U1 用例在编译期 include!(concat!(\"other_\", \"target.rs\")) 把别的测试目标编进自己；改那个目标 ⇒ 指纹应当变",
                 before != after, f"改前 {before[:16]}…，改后 {after[:16]}…")
    module.write_text(os.path.join(small, own_case), base_text + 'const OTHER: &str = include_str!("other_target.rs");\n')
    before, after = changed_by(small, "crash-case:own", environment, "crates/pkg/tests/other_target.rs", edit_other)
    cells.expect("CONTROL", "U0 同一处写成字面路径 include_str!(\"other_target.rs\") ⇒ 指纹变（整词命中）", before != after,
                 f"改前 {before[:16]}…，改后 {after[:16]}…")
    module.write_text(os.path.join(small, own_case), base_text)

    module.write_text(os.path.join(small, "crates/gen/Cargo.toml"), '[package]\nname = "gen"\nversion = "0.0.0"\nbuild = "build.rs"\n')
    module.write_text(os.path.join(small, "crates/gen/build.rs"),
                      'fn main() {\n    for entry in std::fs::read_dir("../pkg/tests").unwrap() {\n'
                      '        println!("cargo:rerun-if-changed={}", entry.unwrap().path().display());\n    }\n}\n')
    module.write_text(os.path.join(small, "crates/gen/src/lib.rs"), "include!(concat!(env!(\"OUT_DIR\"), \"/generated.rs\"));\n")
    before, after = changed_by(small, "crash-case:own", environment, "crates/pkg/tests/other_target.rs", edit_other)
    cells.expect("ATTACK", "U2 另一个包的 build.rs 按目录读 crates/pkg/tests/（pkg 自己没有 build.rs）；改 pkg 里别的测试目标 ⇒ 指纹应当变",
                 before != after, f"改前 {before[:16]}…，改后 {after[:16]}…")
finally:
    shutil.rmtree(small, ignore_errors=True)

REPOSITORY = probe_common.REPOSITORY
CASE_KEYS = ["crash-case:layer0-first-stream", "crash-case:layer0-second-stream", "crash-case:floor-raise-pushed-by-the-session",
             "crash-case:c561-sigma-full"]
copy = tempfile.mkdtemp(prefix="opus-j2-copy-")
try:
    listed = subprocess.run(["git", "-C", REPOSITORY, "ls-files", "-z", "-co", "--exclude-standard", "--", "crates/", "Cargo.toml", "Cargo.lock"],
                            capture_output=True, check=True).stdout.split(b"\0")
    for name in [os.fsdecode(item) for item in listed if item] + [".claude/gate.d/stage-inputs.tsv", ".gitignore"]:
        source = os.path.join(REPOSITORY, name)
        if os.path.isfile(source):
            os.makedirs(os.path.dirname(os.path.join(copy, name)), exist_ok=True)
            shutil.copy(source, os.path.join(copy, name))
    module.run_quietly(["git", "init", "-q", "-b", "master", copy], module.GIT_IDENTITY)
    environment = toolchain_environment(copy)
    with open(os.path.join(copy, ".gitignore"), "a", encoding="utf-8") as handle:
        handle.write(".tools/\n.probe-manifest.out\n")
    for key in CASE_KEYS:
        fingerprint, file_count, excluded = fingerprint_in(copy, key, environment)
        cells.expect("INFO", f"真仓拷贝 {key}", True, f"指纹 {fingerprint[:16]}…，留 {file_count} 个文件，减去 {excluded} 个")
    edits = [
        ("O1 只给 crates/mutations.tsv 加一行（没有哪条用例在编译期或运行期读它）", "crates/mutations.tsv",
         lambda text: text + "probe\tcrates/singlefs-core/src/lib.rs\tx\ty\tprobe\n", CASE_KEYS),
        ("O2 只给 tests/common_tree_split/mod.rs 加一个没人调的函数（四条用例的测试目标都不 mod 它）",
         "crates/singlefs-harness/tests/common_tree_split/mod.rs", lambda text: text + "pub fn probe_unused_helper() {}\n", CASE_KEYS),
        ("O3 只给 harness 的 src/bin/e142_first_transaction_write_dump.rs 加一个函数（集成测试不链接 bin，四条都不读 CARGO_BIN_EXE_*）",
         "crates/singlefs-harness/src/bin/e142_first_transaction_write_dump.rs",
         lambda text: text + "#[allow(dead_code)]\nfn probe_unused() {}\n", CASE_KEYS),
        ("O4 只给 tests/common_admission/mod.rs 加一个函数（只有 floor-raise 那条 mod 它）",
         "crates/singlefs-harness/tests/common_admission/mod.rs", lambda text: text + "pub fn probe_unused_helper() {}\n", CASE_KEYS[:2] + CASE_KEYS[3:]),
    ]
    for label, relative, edit, unaffected in edits:
        if not os.path.isfile(os.path.join(copy, relative)):
            cells.expect("CONTROL", f"{label}：文件在", False, f"真仓拷贝里没有 {relative}")
            continue
        moved = []
        for key in unaffected:
            before, after = changed_by(copy, key, environment, relative, edit)
            if before != after:
                moved.append(key.split(":", 1)[1])
        cells.expect("ATTACK", f"{label} ⇒ 读不到它的用例指纹应当不变", not moved,
                     f"指纹变了、下一趟 --full 要重跑的：{'、'.join(moved) or '无'}（共 {len(unaffected)} 条读不到它）")
finally:
    shutil.rmtree(copy, ignore_errors=True)
sys.exit(cells.finish())
