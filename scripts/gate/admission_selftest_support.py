"""准入自证（入口 admission.py 的 --selftest）的一块：样本仓的搭法、Selftest 记数类、起子进程与写文件的小工具、弄坏开关 BreakSwitch，以及单元键、判定库、复用哨兵、复用三问、test-plan 并行等格。只在自证里 import。
"""

import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

from admission_base import (
     REGISTRATION_TABLE, REPLAY_SCRIPT, RESULTS_DIRECTORY, GATE_GREEN_TREE_RECORD, BREAK_VARIABLE, FUNCTION_UNITS_LIBRARY,
     VERDICT_LIBRARY_DIRECTORY_NAME, RED_VERDICT_REASON, EXIT_REGISTRATION_ERROR, EXIT_GATE_MUST_RUN, EXIT_GATE_MAY_SKIP, EXIT_UNIT_KEY_REFUSED,
     MULTI_HOST_CONFIGURATION_VARIABLE, THREAD_CAP_VARIABLE, BUILD_ENVIRONMENT_EXACT_VARIABLES, BUILD_ENVIRONMENT_VARIABLE_FORMS, function_units,
     break_is_set, ENTRY_SCRIPT_NAME, UnitKeyError, InputManifestError,
)
from admission_unit_key import (
     JUDGE_ONLY_ENTRY, Request, KeyContext, closure_of_entry, closure_unit_lines, domain_variable_tokens, compute_unit_key, LATEST_COMPOSITIONS_KEPT,
     VerdictLibrary, composition_file_names, composition_packed, composition_unpacked, REUSE_CHECKS_LIBRARY, reuse_checks_module,
     build_environment_lines, command_build_environment_lines,
)
from lib_same_directory_imports import RuntimeDataError, runtime_data_files, same_directory_import_closure
from admission_commands import (
     shell_index_filled, TEST_PLAN_WORKERS_FILE_VARIABLE, RED_RECORD_FAILURE_HEAD, RED_RECORD_FAILURE_SAMPLE_MARKER, OWED_WITHDRAWALS_DIRECTORY,
     RED_RECORD_UNAVAILABLE_SAMPLE_MARKER, OWED_WITHDRAWAL_UNAVAILABLE_SAMPLE_MARKER, PLAN_SERVICE_UNAVAILABLE_EXIT,
)

# ── 自证：单元键、判定库、门禁的复用答法、实验键与复跑表的行键（临时小仓，PATH 前面放假 cargo、假 rustc）──────────

SELFTEST_HERE = os.path.dirname(os.path.abspath(__file__))
SELFTEST_REPOSITORY = os.path.dirname(os.path.dirname(SELFTEST_HERE))   # 真仓的根：拷算键库运行期读的数据文件从这里取
PASSED_ONE_LINE = "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.01s"
# 临时仓里跑真仓脚本的副本时要从真仓链进来的两样：规范副本与项目的 preflight 垫片（.claude/scripts/preflight.sh）所在的目录
LINKED_INTO_SELFTEST_REPOSITORIES = (".claude/singlefs-ai-sop", ".claude/scripts")


def link_sop_copy(work):
    """临时仓里跑真仓脚本的副本（replay.sh 这一类）时，副本开头按相对路径 source .claude/scripts/preflight.sh，垫片再找规范副本里的 preflight.sh：
    LINKED_INTO_SELFTEST_REPOSITORIES 那两样在临时仓里建成指向真仓那一份的符号链接，并写进临时仓的 .gitignore（不算进改动范围）。"""
    os.makedirs(os.path.join(work, ".claude"), exist_ok=True)
    real_root = os.path.dirname(os.path.dirname(SELFTEST_HERE))
    with open(os.path.join(work, ".gitignore"), "a", encoding="utf-8") as handle:
        for relative_path in LINKED_INTO_SELFTEST_REPOSITORIES:
            os.symlink(os.path.join(real_root, relative_path), os.path.join(work, relative_path))
            handle.write(relative_path + "\n")


# 临时仓里跑真仓 replay.sh 的副本时，开跑前的构建经 scripts/hosts/two-logs.sh 起（它再 exec 同目录的 two_logs.py）：这两份从真仓链进来
REPLAY_BUILD_LOG_TOOLS = ("scripts/hosts/two-logs.sh", "scripts/hosts/two_logs.py")
# replay.sh 这一趟自己的编译目录经 scripts/hosts/run-build-directory.sh 建、删（删法在同目录的 run_build_directory.py）：载入时 source，判准入之前就建，
# 不链它连一行都不用跑的几格（准入没放行、复跑表按行的准入键写错、--admission-only）也退 2，所以不归弄坏开关 replay-build-tools-unlinked 管（开关下也链）
REPLAY_BUILD_DIRECTORY_TOOLS = ("scripts/hosts/run-build-directory.sh", "scripts/hosts/run_build_directory.py")
# replay.sh 按行复用读写判定库经 admission.py library-files、replay-row-inputs，它们载入的判定库客户端（临时树里没有就退 2，replay.sh 整趟退出）
REPLAY_ROW_LIBRARY_TOOLS = ("scripts/prune-server/prune_server_client.py",)
# replay.sh 载入时 source 的函数库（只定义函数）：样本仓没有它，载入那一句报「No such file」、照样往下跑；自证给 REPLAY_JOBS=1，里面的 heavy_thread_default 不被调。
# 照样链进来，不留那一句找不到；不归弄坏开关 replay-build-tools-unlinked 管（开关下也链）
REPLAY_LOAD_LIBRARIES = ("scripts/hosts/heavy-threads.sh",)


def link_replay_build_log_tools(work):
    """REPLAY_LOAD_LIBRARIES、REPLAY_BUILD_DIRECTORY_TOOLS、REPLAY_BUILD_LOG_TOOLS 与 REPLAY_ROW_LIBRARY_TOOLS 在临时仓里建成指向真仓那一份的符号链接，写进临时仓的 .gitignore（同 link_sop_copy）。
    弄坏开关 ADMISSION_BREAK=replay-build-tools-unlinked：构建那两样（REPLAY_BUILD_LOG_TOOLS）不链，replay.sh 那几格的构建退 127、一条实验都没跑；
    判定库客户端照链（不链它 replay.sh 在按行复用那一步就退 1、走不到这条开关弄坏的构建）。"""
    linked = REPLAY_LOAD_LIBRARIES + REPLAY_BUILD_DIRECTORY_TOOLS + REPLAY_ROW_LIBRARY_TOOLS
    if not break_is_set("replay-build-tools-unlinked"):
        linked = linked + REPLAY_BUILD_LOG_TOOLS
    real_root = os.path.dirname(os.path.dirname(SELFTEST_HERE))
    with open(os.path.join(work, ".gitignore"), "a", encoding="utf-8") as handle:
        for relative_path in linked:
            os.makedirs(os.path.dirname(os.path.join(work, relative_path)), exist_ok=True)
            os.symlink(os.path.join(real_root, relative_path), os.path.join(work, relative_path))
            handle.write(relative_path + "\n")

FAKE_RUSTC_VERSION_FILE = ".fake-rustc-version"   # 临时仓里假 rustc 读的版本串（E01：换一版工具链）


def fake_toolchain_scripts(version_file):
    """假 cargo、假 rustc：-V / -vV 照 version_file 里的一行打（文件不在打 0.0.0-selftest），别的参数一律退 0、什么都不做。"""
    rustc = ('#!/usr/bin/env bash\nversion="$(cat ' + f"'{version_file}'" + ' 2>/dev/null || echo 0.0.0-selftest)"\n'
             'case "${1:-}" in\n  -V) echo "rustc $version";;\n'
             '  -vV) printf "rustc %s\\nbinary: rustc\\nhost: x86_64-unknown-linux-gnu\\nrelease: %s\\n" "$version" "$version";;\nesac\nexit 0\n')
    cargo = '#!/usr/bin/env bash\n[[ "${1:-}" == -V || "${1:-}" == --version ]] && echo "cargo 0.0.0-selftest"\nexit 0\n'
    return {"rustc": rustc, "cargo": cargo}


UNIT_SAMPLE_FILES = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/demo", "crates/other"]\nresolver = "2"\n\n[profile.release]\noverflow-checks = true\n',
    "Cargo.lock": ('# 样本的锁\nversion = 4\n\n[[package]]\nname = "demo"\nversion = "0.1.0"\ndependencies = [\n "itoa",\n]\n\n'
                   '[[package]]\nname = "itoa"\nversion = "1.0.0"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "aaaa"\n\n'
                   '[[package]]\nname = "other"\nversion = "0.1.0"\ndependencies = [\n "ryu",\n]\n\n'
                   '[[package]]\nname = "ryu"\nversion = "1.0.0"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "bbbb"\n'),
    "crates/demo/Cargo.toml": '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\nitoa = "1"\n',
    "crates/demo/src/lib.rs": ("pub const WIDTH: u32 = 88;\n\npub fn helper() -> u32 {\n    WIDTH + 1\n}\n\npub fn unrelated() -> u32 {\n    7\n}\n\n"
                               "pub fn formats(value: u32) -> String {\n    itoa::Buffer::new().format(value).to_string()\n}\n"),
    "crates/demo/tests/setting.rs": ("use demo::helper;\n\nfn observer() -> u32 {\n    helper() * 2\n}\n\n#[test]\nfn test_t() {\n    assert_eq!(observer(), 178);\n}\n\n"
                                     "#[test]\nfn test_u() {\n    assert_eq!(2, 2);\n}\n\n#[test]\nfn test_v() {\n    assert_eq!(demo::formats(3), \"3\");\n}\n\n"
                                     "#[test]\n#[ignore]\nfn full_case() {}\n"),
    "crates/other/Cargo.toml": '[package]\nname = "other"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\nryu = "1"\n',
    "crates/other/src/lib.rs": "pub fn printed(value: f64) -> String {\n    ryu::Buffer::new().format(value).to_string()\n}\n",
    "scripts/judge.py": "def judge(line):\n    return line.endswith(' ok')\n\n\nif __name__ == '__main__':\n    print(judge('test a ... ok'))\n",
    "scripts/tool.py": ("import sys\n\n\ndef anchor_hints(text):\n    return text.split()\n\n\ndef selftest():\n    return 0 if anchor_hints('a b') == ['a', 'b'] else 1\n\n\n"
                        "if __name__ == '__main__':\n    sys.exit(selftest())\n"),
    "scripts/other.py": "def unrelated():\n    return 2\n\n\nif __name__ == '__main__':\n    print(unrelated())\n",
    ".claude/kb/decisions/08-sample.md": "# 样本决策\n\n一句。\n",
    REGISTRATION_TABLE: "# 样本登记表\ndemo-gate.sh\ttests-of:demo\t# 样本门禁\nother-gate.sh\tscripts/other.py\t# 别的一道\n",
}
UNIT_SAMPLE_TEST_TOKENS = ["profile:release", "judge:scripts/judge.py::judge"]


def unit_sample_requests():
    return [Request(f"crates/demo/tests/setting.rs::{name}", f"crates/demo/tests/setting.rs::{name}", UNIT_SAMPLE_TEST_TOKENS) for name in ("test_t", "test_u", "test_v")]


def edit_once(path, old, new):
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    if text.count(old) != 1:
        raise AssertionError(f"自证样本 {path} 里「{old}」命中 {text.count(old)} 次（要恰好一次）")
    write_text(path, text.replace(old, new))


def build_unit_sample_repository(work):
    """单元键那几格的临时小仓：两个工作区包（demo 用 itoa、other 用 ryu）、一份三条测试的设点文件、一个判法、两份 python 脚本、kb 决策、登记表；提交成 base。"""
    run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
    for path, text in UNIT_SAMPLE_FILES.items():
        write_text(os.path.join(work, path), text)
    tools = os.path.join(work, ".tools")
    for name, script in fake_toolchain_scripts(os.path.join(work, FAKE_RUSTC_VERSION_FILE)).items():
        write_executable(os.path.join(tools, name), script)
    write_text(os.path.join(work, ".gitignore"), ".tools/\n.fake-rustc-version\n*.plan\n*.requests\n*.names\n")
    run_quietly(["git", "-C", work, "add", "-A"], GIT_IDENTITY)
    run_quietly(["git", "-C", work, "commit", "-qm", "base"], GIT_IDENTITY)
    return environment_without_build_settings({"PATH": tools + os.pathsep + os.environ.get("PATH", ""), "CARGO_HOME": os.path.join(work, ".tools", "cargo-home")})


class EnvironmentPatch:
    """在本进程里临时换几样环境变量（给进程内算键那几格），退出时照原样还回去。"""
    def __init__(self, changes):
        self.changes = changes
        self.saved = {}

    def __enter__(self):
        for name, value in self.changes.items():
            self.saved[name] = os.environ.get(name)
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
        return self

    def __exit__(self, *_exception):
        for name, value in self.saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
        return False


SAMPLE_PATCHED_VARIABLES = ("PATH", "CARGO_HOME", "RUSTFLAGS", BREAK_VARIABLE)   # 进程内算键与算版本行那几格临时换的环境变量（EnvironmentPatch）
TOOLCHAIN_LINES_NAME = "工具链行"   # toolchain_lines_of 交的那一项的名字：工具链行按闭包根一份，不按测试分


def sample_keys(work, environment, requests=None):
    """进程内算一遍样本的键：{名字: 键或 None}。每次都新建 KeyContext（不吃上一格的缓存）。"""
    with EnvironmentPatch({name: environment.get(name) for name in SAMPLE_PATCHED_VARIABLES}):
        context = KeyContext(work, environment)
        return {request.name: compute_unit_key(context, request).key for request in (requests or unit_sample_requests())}


def toolchain_lines_of(work, environment):
    """进程内算一遍样本仓的工具链行（KeyContext.rust_toolchain_lines：不进单元键，host_triple 与 scripts/experiment-apparatus/device-code-key.py 读它）：
    {TOOLCHAIN_LINES_NAME: 行或 None}，算不出交 None。"""
    with EnvironmentPatch({name: environment.get(name) for name in SAMPLE_PATCHED_VARIABLES}):
        try:
            return {TOOLCHAIN_LINES_NAME: KeyContext(work, environment).rust_toolchain_lines(work)}
        except UnitKeyError:
            return {TOOLCHAIN_LINES_NAME: None}


def dependency_lines_of(work, environment):
    """进程内算一遍样本三条测试的依赖版本行（KeyContext.dependency_lines：不进单元键，scripts/experiment-apparatus/device-code-key.py 读它）：
    {短名: 行或 None}，算不出交 None。"""
    with EnvironmentPatch({name: environment.get(name) for name in SAMPLE_PATCHED_VARIABLES}):
        context = KeyContext(work, environment)
        lines = {}
        for request in unit_sample_requests():
            try:
                closure, target_id, workspace = closure_of_entry(context, work, request, request.entry)
                lines[short_name(request.name)] = context.dependency_lines(work, workspace, closure, target_id)
            except UnitKeyError:
                lines[short_name(request.name)] = None
        return lines


def build_environment_lines_of(work, environment):
    """进程内算一遍样本仓的构建环境行（build_environment_lines：子命令 build-environment-lines 打它，不进单元键）：[(名字, 内容)]，算不出交 None。"""
    with EnvironmentPatch({name: environment.get(name) for name in SAMPLE_PATCHED_VARIABLES}):
        try:
            return build_environment_lines(work, environment)
        except InputManifestError:
            return None


def short_name(name):
    return name.rsplit("::", 1)[-1]


def run_unit_key_cells(selftest, module):
    """单元键：只改注释、加一个没人调的函数、改别的测试、登记表加一行注释、无关包抬版本、构建配置只改注释 ⇒ 键不变；
    改被调的函数、被引用的常量、判法 ⇒ 用到它的那几条键变、别的不变。
    版本、构建环境与编译档不进单元键（admission.py 文件头「单元键」）：换工具链、RUSTFLAGS、.cargo 配置的值、用到的外部包抬版本、release 档的设置 ⇒ 三条的键都不变；
    前几样另核它们进的那一份跟着变：工具链行（KeyContext.rust_toolchain_lines）、构建环境行（build_environment_lines）、依赖版本行（KeyContext.dependency_lines）。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-units-")
    try:
        environment = build_unit_sample_repository(work)
        baseline = sample_keys(work, environment)
        selftest.expect("单元键：样本三条测试都算得出键（不拒算）", all(baseline.values()), f"算出 {baseline}")
        lib = os.path.join(work, "crates/demo/src/lib.rs")
        setting = os.path.join(work, "crates/demo/tests/setting.rs")
        lock = os.path.join(work, "Cargo.lock")
        settings_file = os.path.join(work, ".cargo/config.toml")
        version_file = os.path.join(work, FAKE_RUSTC_VERSION_FILE)

        def cell(label, path, old, new, changed_names, environment_changes=None, revert=True):
            if path is not None:
                edit_once(path, old, new)
            try:
                after = sample_keys(work, dict(environment, **(environment_changes or {})))
            finally:
                if path is not None and revert:
                    edit_once(path, new, old)
            changed = sorted(short_name(name) for name in baseline if after.get(name) != baseline[name])
            selftest.expect(f"单元键：{label} ⇒ 变的是 {changed_names or '无'}", changed == sorted(changed_names), f"实际变的是 {changed or '无'}")
            return changed

        def edit_of(path, old, new):
            """unkeyed_cell 的一处改动：(改上, 改回)。"""
            return (lambda: edit_once(path, old, new)), (lambda: edit_once(path, new, old))

        def unkeyed_cell(label, outside_name, outside_lines, outside_changed, change=None, environment_changes=None, base_changes=None):
            """变更的输入不进单元键：变更前后三条测试的单元键都不变；它进的那一份（outside_lines 算的 {名字: 行}）里变的恰好是 outside_changed。
            change 是（改上, 改回）两个函数；environment_changes 只给变更后那一趟，base_changes 变更前后都给（弄坏开关那两格把开关设在这里）。"""
            before_environment = dict(environment, **(base_changes or {}))
            after_environment = dict(before_environment, **(environment_changes or {}))
            keys_before, outside_before = sample_keys(work, before_environment), outside_lines(work, before_environment)
            if change is not None:
                change[0]()
            try:
                keys_after, outside_after = sample_keys(work, after_environment), outside_lines(work, after_environment)
            finally:
                if change is not None:
                    change[1]()
            keys_changed = sorted(short_name(name) for name in keys_before if keys_after.get(name) != keys_before[name])
            changed = sorted(name for name in outside_before if outside_after.get(name) != outside_before[name])
            computed = all(keys_before.values()) and None not in outside_before.values() and None not in outside_after.values()
            wanted = "变" if outside_changed == [TOOLCHAIN_LINES_NAME] else f"变的是 {outside_changed or '无'}"
            selftest.expect(f"单元键：{label} ⇒ 单元键都不变，{outside_name}{wanted}", computed and not keys_changed and changed == sorted(outside_changed),
                            f"单元键变的是 {keys_changed or '无'}，{outside_name}变的是 {changed or '无'}；改前 {str(outside_before)[:400]}，改后 {str(outside_after)[:400]}")

        toolchain, dependencies = "工具链行（rust_toolchain_lines）", "依赖版本行（dependency_lines）"
        cell("被调函数 helper 体里只加一行注释（N01）", lib, "    WIDTH + 1\n", "    // 只是注释\n    WIDTH + 1\n", [])
        cell("同文件加一个没人调的函数（N04）", lib, "pub fn unrelated", "pub fn nobody_calls() -> u32 {\n    3\n}\n\npub fn unrelated", [])
        cell("被调函数 helper 改一个字（S02）", lib, "WIDTH + 1", "WIDTH + 2", ["test_t"])
        cell("被引用的常量 WIDTH 88 → 89（V11 形态）", lib, "pub const WIDTH: u32 = 88;", "pub const WIDTH: u32 = 89;", ["test_t"])
        cell("同设点文件里别的测试 test_u 改一个字（N19）", setting, "assert_eq!(2, 2);", "assert_eq!(2, 2 + 0);", ["test_u"])
        cell("观察者函数 observer 改一个字（S17）", setting, "helper() * 2", "helper() * 3", ["test_t"])
        cell("测试模块加一条空单测（N06）", setting, "#[test]\n#[ignore]", "#[test]\nfn added() {}\n\n#[test]\n#[ignore]", [])
        cell("仓根 Cargo.toml 加一行注释（N07）", os.path.join(work, "Cargo.toml"), "resolver = \"2\"\n", "resolver = \"2\"\n# 只是注释\n", [])
        cell("Cargo.lock 里没人用的 ryu 抬版本（N09）", lock, 'name = "ryu"\nversion = "1.0.0"', 'name = "ryu"\nversion = "1.0.1"', [])
        # 依赖版本不进单元键，只进依赖版本行：闭包里有单元点了 itoa 的那一条（test_v）才收它（弄坏开关 dependencies-by-crate 下照包收，三条都收、这一格转红）
        itoa_bumped = edit_of(lock, 'name = "itoa"\nversion = "1.0.0"', 'name = "itoa"\nversion = "1.0.1"')
        unkeyed_cell("Cargo.lock 里 formats 用到的 itoa 抬版本（S14 形态）", dependencies, dependency_lines_of, ["test_v"], change=itoa_bumped)
        unkeyed_cell("弄坏开关 dependencies-by-crate 下 S14 那一格转红（外部包按包收：itoa 抬版本三条测试的依赖版本行全变）", dependencies, dependency_lines_of,
                     ["test_t", "test_u", "test_v"], change=itoa_bumped, base_changes={BREAK_VARIABLE: "dependencies-by-crate"})
        cell("release 档的 overflow-checks 改成 false（S16：编译档不进键）", os.path.join(work, "Cargo.toml"), "overflow-checks = true", "overflow-checks = false", [])
        cell("登记表加一行注释、别的一道多登记一个入口（N22、N23）", os.path.join(work, REGISTRATION_TABLE), "other-gate.sh\tscripts/other.py",
             "# 加一行注释\nother-gate.sh\tscripts/other.py scripts/tool.py", [])
        cell("判法 scripts/judge.py::judge 只加一行注释", os.path.join(work, "scripts/judge.py"), "    return line.endswith(' ok')",
             "    # 只是注释\n    return line.endswith(' ok')", [])
        cell("判法 scripts/judge.py::judge 改一个字", os.path.join(work, "scripts/judge.py"), "line.endswith(' ok')", "line.endswith(' ok ')",
             ["test_t", "test_u", "test_v"])
        # 工具链与构建环境不进单元键，只进工具链行（弄坏开关 skip-build-environment 下构建环境那一截不进工具链行，RUSTFLAGS、rustflags 与 runner 那几格转红）
        unkeyed_cell("工具链换一版（E01：假 rustc 的 -vV 换了）", toolchain, toolchain_lines_of, [TOOLCHAIN_LINES_NAME],
                     change=((lambda: write_text(version_file, "1.99.0-selftest\n")), (lambda: os.remove(version_file))))
        unkeyed_cell("RUSTFLAGS 加 -C target-cpu=native（E02）", toolchain, toolchain_lines_of, [TOOLCHAIN_LINES_NAME],
                     environment_changes={"RUSTFLAGS": "-C target-cpu=native"})
        unkeyed_cell("弄坏开关 skip-build-environment 下 RUSTFLAGS 那一格转红（工具链行不变）", toolchain, toolchain_lines_of, [],
                     environment_changes={"RUSTFLAGS": "-C target-cpu=native"}, base_changes={BREAK_VARIABLE: "skip-build-environment"})
        write_text(settings_file, "# 构建配置\n[build]\nrustflags = [\"-Cdebuginfo=1\"]\n")
        try:
            unkeyed_cell(".cargo/config.toml 只改注释（N42）", toolchain, toolchain_lines_of, [], change=edit_of(settings_file, "# 构建配置\n", "# 构建配置，只改注释\n"))
            unkeyed_cell(".cargo/config.toml 改 [build] rustflags 的值", toolchain, toolchain_lines_of, [TOOLCHAIN_LINES_NAME],
                         change=edit_of(settings_file, "-Cdebuginfo=1", "-Cdebuginfo=2"))
        finally:
            os.remove(settings_file)
        # 构建环境行只收改结局的设置（admission_base.py 的 BUILD_ENVIRONMENT_VARIABLES_LEFT_OUT、BUILD_CONFIGURATION_SECTIONS、BUILD_KEYS_LEFT_OUT）：
        # 增量编译开关（CARGO_BUILD_INCREMENTAL、[build] incremental）、[alias]、[profile.<名>] 都不收，[build] rustflags 收；单元键一样都不读。
        # 弄坏开关 build-environment-all-settings 下整份收，这一格转红
        write_text(settings_file, "[build]\nincremental = false\nrustflags = [\"-Cdebuginfo=1\"]\n\n[alias]\nxt = \"test\"\n")
        try:
            incremental_off = dict(environment, CARGO_BUILD_INCREMENTAL="false")

            def reading(reading_environment):
                return sample_keys(work, reading_environment), build_environment_lines_of(work, reading_environment)
            start = reading(incremental_off)
            readings = {"CARGO_BUILD_INCREMENTAL 换成 true": reading(dict(environment, CARGO_BUILD_INCREMENTAL="true"))}
            edit_once(settings_file, "incremental = false", "incremental = true")
            readings["[build] incremental 换成 true"] = reading(incremental_off)
            edit_once(settings_file, "xt = \"test\"\n", "xt = \"test\"\nxb = \"build\"\n")
            readings["[alias] 加一条"] = reading(incremental_off)
            edit_once(settings_file, "[alias]\n", "[profile.release]\nopt-level = 2\n\n[alias]\n")
            readings["[profile.release] 改一项"] = reading(incremental_off)
            edit_once(settings_file, "-Cdebuginfo=1", "-Cdebuginfo=2")
            rustflags_changed = reading(incremental_off)
        finally:
            os.remove(settings_file)
        differing = [name for name, found in readings.items() if found != start]
        selftest.expect("单元键：构建环境行（build_environment_lines）只收改结局的设置：CARGO_BUILD_INCREMENTAL 与 [build] incremental 换取值、[alias] 加一条、"
                        "[profile.release] 改一项都不变，[build] rustflags 改值变；单元键都不变",
                        all(start[0].values()) and start[1] is not None and not differing
                        and rustflags_changed[0] == start[0] and rustflags_changed[1] not in (None, start[1]),
                        f"跟起点不同的 {differing or '无'}；rustflags 改值之后构建环境行{'变了' if rustflags_changed[1] != start[1] else '没变'}、"
                        f"单元键{'没变' if rustflags_changed[0] == start[0] else '变了'}；起点构建环境行 {start[1]}")
        # cargo 配置文件的找法：被判那棵树的根往上每一层的 .cargo 都读（祖先目录的 config.toml 写 [build] rustflags，构建环境行变、单元键不变）。
        # 弄坏开关 cargo-configuration-ancestors-unread 下祖先目录不读，这一格转红
        outer = tempfile.mkdtemp(prefix="admission-selftest-ancestor-")
        try:
            nested = os.path.join(outer, "nested")
            nested_environment = build_unit_sample_repository(nested)
            nested_keys_before, nested_lines_before = sample_keys(nested, nested_environment), build_environment_lines_of(nested, nested_environment)
            write_text(os.path.join(outer, ".cargo/config.toml"), "[build]\nrustflags = [\"-Cdebuginfo=1\"]\n")
            nested_keys_after, nested_lines_after = sample_keys(nested, nested_environment), build_environment_lines_of(nested, nested_environment)
        finally:
            shutil.rmtree(outer, ignore_errors=True)
        selftest.expect("单元键：祖先目录的 .cargo/config.toml 写 [build] rustflags ⇒ 构建环境行变（cargo 配置从被判那棵树的根往上每一层都读）、单元键都不变",
                        all(nested_keys_before.values()) and nested_keys_after == nested_keys_before
                        and nested_lines_before is not None and nested_lines_after not in (None, nested_lines_before),
                        f"改前键 {nested_keys_before}、改后键 {nested_keys_after}；构建环境行改前 {nested_lines_before}、改后 {nested_lines_after}")
        # 子命令 build-environment-lines（E171 复用库起程序取构建环境）：每行「名字<制表符>内容的 sha256」，与 build_environment_lines 逐项相同；
        # 弄坏开关 build-environment-lines-names-only 下只打名字，这一格转红
        import contextlib
        import io
        write_text(settings_file, "[build]\nrustflags = [\"-Cdebuginfo=1\"]\n")
        printed = io.TextIOWrapper(io.BytesIO(), encoding="utf-8", newline="\n")
        with contextlib.redirect_stdout(printed):
            command_exit = command_build_environment_lines([work])
        printed.flush()
        printed_lines = printed.buffer.getvalue().decode("utf-8").splitlines()
        wanted_lines = [f"{name}\t{hashlib.sha256(content).hexdigest()}" for name, content in build_environment_lines(work)]
        if break_is_set("build-environment-lines-names-only"):
            printed_lines = [line.split("\t")[0] for line in printed_lines]
        os.remove(settings_file)
        selftest.expect("子命令 build-environment-lines：每行「名字<制表符>内容的 sha256」，与 build_environment_lines 逐项相同、不空",
                        command_exit == 0 and bool(wanted_lines) and printed_lines == wanted_lines,
                        f"退 {command_exit}；打了 {len(printed_lines)} 行，要 {len(wanted_lines)} 行；头一行 {printed_lines[:1]} 要 {wanted_lines[:1]}")
        python_requests = [Request("tool", "scripts/tool.py", []), Request("other", "scripts/other.py", [])]
        python_before = sample_keys(work, environment, python_requests)
        edit_once(os.path.join(work, "scripts/tool.py"), "return text.split()", "return text.split(None)")
        python_after = sample_keys(work, environment, python_requests)
        edit_once(os.path.join(work, "scripts/tool.py"), "return text.split(None)", "return text.split()")
        selftest.expect("单元键：研究脚本自证走到的 anchor_hints 改一个字（T21）⇒ 只 scripts/tool.py 那一条变",
                        python_before["tool"] and python_before["tool"] != python_after["tool"] and python_before["other"] == python_after["other"],
                        f"改前 {python_before}，改后 {python_after}")
        # 入口写 -（文件头「判定请求」）：键只由判法与内容输入组成，没有判法的拒算
        judge_file = os.path.join(work, "scripts/judge.py")
        judge_only_tokens = ["judge:scripts/judge.py::judge", "line:data=alpha"]
        judge_only_requests = [Request("judged-by-judge", JUDGE_ONLY_ENTRY, judge_only_tokens), Request("no-judge", JUDGE_ONLY_ENTRY, ["line:data=alpha"])]
        judge_only_before = sample_keys(work, environment, judge_only_requests)
        judge_only_line = sample_keys(work, environment, [Request("judged-by-judge", JUDGE_ONLY_ENTRY, ["judge:scripts/judge.py::judge", "line:data=beta"])])
        edit_once(judge_file, "    return line.endswith(' ok')", "    # 只是注释\n    return line.endswith(' ok')")
        judge_only_comment = sample_keys(work, environment, judge_only_requests[:1])
        edit_once(judge_file, "    # 只是注释\n    return line.endswith(' ok')", "    return line.endswith(' ok')")
        edit_once(judge_file, "line.endswith(' ok')", "line.endswith(' ok ')")
        judge_only_changed = sample_keys(work, environment, judge_only_requests[:1])
        edit_once(judge_file, "line.endswith(' ok ')", "line.endswith(' ok')")
        base_key = judge_only_before["judged-by-judge"]
        selftest.expect("单元键：入口写 -（键只由判法与内容输入组成）：判法改一个字键变、判法只改注释键不变、line: 变键变、一个 judge: 都没有拒算",
                        base_key is not None and judge_only_before["no-judge"] is None and judge_only_line["judged-by-judge"] != base_key
                        and judge_only_comment["judged-by-judge"] == base_key and judge_only_changed["judged-by-judge"] not in (None, base_key),
                        f"改前 {base_key}，没有 judge: 的 {judge_only_before['no-judge']}，line 变了 {judge_only_line['judged-by-judge']}，"
                        f"判法只改注释 {judge_only_comment['judged-by-judge']}，判法改一个字 {judge_only_changed['judged-by-judge']}")
        run_caller_input_key_cells(selftest, work, environment)
        run_not_judging_files_cells(selftest, work, environment)
        exit_code, output, _messages = run_in_environment([sys.executable, module, "unit-key", work, "crates/demo/tests/setting.rs::no_such_test"], environment)
        selftest.expect("unit-key：入口找不到 ⇒ 拒算退 4、打拒算行", exit_code == EXIT_UNIT_KEY_REFUSED and output.startswith("refused\tkey-uncomputable"),
                        f"退 {exit_code}，stdout「{output.strip()[:300]}」")
        exit_code, output, _messages = run_in_environment([sys.executable, module, "test-functions", work, "demo"], environment)
        listed = [line.split("\t") for line in output.strip().split("\n") if line]
        states = {row[3]: row[4] for row in listed if len(row) == 5}
        selftest.expect("test-functions：列出 demo 的四条测试（三条 compiled、标了 ignore 的 full_case 是 ignored）",
                        exit_code == 0 and states == {"test_t": "compiled", "test_u": "compiled", "test_v": "compiled", "full_case": "ignored"},
                        f"退 {exit_code}，列出 {states}")
    except AssertionError as error:
        selftest.expect("单元键的样本改得动", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)


# 运行期读的文件分两类（lib_function_units.py 文件头「运行期读的数据文件」）那一格的样本：判法经别的模块的函数读请求文件（实参是判法入口的参数）
# 与它自己的规则表（实参是模块常量）；工具链询问那一格的判法起 [os.environ["RUSTC"], "-V"]；shell 入口带参数那两格：dispatch.sh 按 $1 分两支各起一份脚本，
# plain.sh 不看参数
CALLER_INPUT_SAMPLE_FILES = {
    "scripts/request_rows.py": 'def read_rows(path):\n    with open(path, encoding="utf-8") as handle:\n        return handle.read().splitlines()\n',
    "scripts/request_judge.py": ('import os\n\nimport request_rows\n\nHERE = os.path.dirname(os.path.abspath(__file__))\nRULES = os.path.join(HERE, "request_rules.tsv")\n\n\n'
                                 'def judge_request(request_file):\n    return len(request_rows.read_rows(request_file)) == len(request_rows.read_rows(RULES))\n'),
    "scripts/request_rules.tsv": "rule\n",
    "requests/one.tsv": "alpha\n",
    "scripts/compiler_judge.py": ('import os\nimport subprocess\n\n\ndef judge_version(line):\n    program = os.environ["RUSTC"]\n'
                                  '    return subprocess.run([program, "-V"], capture_output=True, text=True, check=False).stdout.strip() in line\n'),
    ".claude/hooks/selftest-registry.tsv": "scripts/one.py\tselftest\t# 甲\nscripts/two.py\tselftest\t# 乙\n",
    "scripts/one.py": "print(1)\n",
    "scripts/dispatch.sh": ('#!/usr/bin/env bash\nif [[ "${1:-}" == --selftest ]]; then\n  bash scripts/selftest_child.sh\n  exit $?\nfi\n'
                            "bash scripts/run_child.sh\n"),
    "scripts/selftest_child.sh": "#!/usr/bin/env bash\necho selftest\n",
    "scripts/run_child.sh": "#!/usr/bin/env bash\necho run\n",
    "scripts/plain.sh": "#!/usr/bin/env bash\necho plain\n",
    "scripts/registry_judge.py": ('import os\n\nREGISTRY = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), ".claude", "hooks", '
                                  '"selftest-registry.tsv")\n\n\ndef selftest_registry_at(path):\n    with open(path, encoding="utf-8") as handle:\n'
                                  '        return dict(line.split("\\t", 1) for line in handle.read().splitlines() if "\\t" in line)\n\n\n'
                                  'def read_selftest_registry(path):\n    return selftest_registry_at(path)\n\n\n'
                                  'def registered_selftest(key):\n    return read_selftest_registry(REGISTRY).get(key)\n\n\n'
                                  'def judge(key):\n    return registered_selftest(key) is not None\n'),
}
REQUEST_FILE_CELL = "单元键：请求文件内容变而判法没变：这一份判法键不变、调用方那一条键变（file: 折进键）；判法自己读的规则表改了判法键变"
TOOLCHAIN_QUERY_KEY_CELL = "单元键：判法闭包里有工具链询问（[os.environ[\"RUSTC\"], \"-V\"]）：不拒算、算得出判法键，假 rustc 换一版判法键不变（询问打的版本不进键）"
PROGRAM_ARGUMENTS_KEY_CELL = "单元键：program-argument: 进键：同一程序入口两组参数各算得出、键不同"
SHELL_PROGRAM_ARGUMENTS_NARROWED_CELL = "单元键：shell 入口带 program-argument:run，--selftest 那一支起的脚本改了：带 run 的键不变、带 --selftest 的键变"
SHELL_PROGRAM_ARGUMENTS_KEY_CELL = "单元键：shell 入口不按参数分支、两组 program-argument: 闭包相同：各算得出、键不同"
ROW_KEYED_JUDGE_CELL = ("单元键：判法经登记的取用函数按入口查 .claude/hooks/selftest-registry.tsv 的一行（lib_function_units.py 文件头「按行进键的登记表」）："
                        "改表里别的脚本那一行，这一条的键不变；改它自己那一行，键变")


# 按角色排除（文件头「单元键」）那几格的样本：判法 judge.py 载入 lib_function_units.py，经它调函数、起程序
NOT_JUDGING_SAMPLE_FILES = {
    "scripts/gate/lib_function_units.py": ('import subprocess\n\n\ndef library_helper():\n    return 1\n\n\n'
                                           'def library_entry():\n    return library_helper()\n\n\n'
                                           'def library_launch():\n    return subprocess.run(["bash", "tool/only_via_library.sh"], check=False)\n'),
    "scripts/gate/judge.py": 'import lib_function_units\n\n\ndef judge():\n    lib_function_units.library_launch()\n    return lib_function_units.library_helper() + 1\n',
    "tool/only_via_library.sh": "#!/usr/bin/env bash\necho sample\n",
}
NOT_JUDGING_OTHER_KEY_CELL = "单元键：改 lib_function_units.py 里的代码、改只经它起的程序：别的判定的键不变（入口与 judge: 都算）"
NOT_JUDGING_OWN_KEY_CELL = "单元键：判法入口就在 lib_function_units.py 里的：改它调到的函数键变（入口与 judge: 都算）"
NOT_JUDGING_JUDGE_CHANGED_CELL = "单元键：改别的判定自己的判法：键变（入口与 judge: 都算）"


def run_not_judging_files_cells(selftest, work, environment):
    """按角色排除三格：改样本里的 lib_function_units.py 一处代码、再改只经它起的程序，别的判定（入口与 judge: 两种写法）的键不变（弄坏开关 not-judging-files-keyed 下红）；
    入口本身在它里面的两条（入口 library_entry 只调同一份里的 library_helper，改 library_helper 入口自己的指纹不动）键变（not-judging-entries-excluded 下红：
    入口也排除时 library_helper 不进键）；改 judge.py 自己，别的判定的键变（这一格防把判法也一并排了）。"""
    sample = os.path.join(work, "not-judging-sample")
    for path, text in NOT_JUDGING_SAMPLE_FILES.items():
        write_text(os.path.join(sample, path), text)
    requests = [Request("other-entry", "scripts/gate/judge.py::judge", []), Request("other-judge", JUDGE_ONLY_ENTRY, ["judge:scripts/gate/judge.py::judge"]),
                Request("own-entry", "scripts/gate/lib_function_units.py::library_entry", []),
                Request("own-judge", JUDGE_ONLY_ENTRY, ["judge:scripts/gate/lib_function_units.py::library_entry"])]
    try:
        before = sample_keys(sample, environment, requests)
        edit_once(os.path.join(sample, "scripts/gate/lib_function_units.py"), "return 1", "return 2")
        library_edited = sample_keys(sample, environment, requests)
        edit_once(os.path.join(sample, "tool/only_via_library.sh"), "echo sample", "echo changed")
        program_edited = sample_keys(sample, environment, requests)
        edit_once(os.path.join(sample, "scripts/gate/judge.py"), "+ 1", "+ 2")
        judge_edited = sample_keys(sample, environment, requests)
        others, own = ("other-entry", "other-judge"), ("own-entry", "own-judge")
        selftest.expect(NOT_JUDGING_OTHER_KEY_CELL,
                        all(before[name] is not None and before[name] == library_edited[name] == program_edited[name] for name in others),
                        f"改前 {before}；改库之后 {library_edited}；改只经库起的程序之后 {program_edited}")
        selftest.expect(NOT_JUDGING_OWN_KEY_CELL, all(None not in (before[name], library_edited[name]) and before[name] != library_edited[name] for name in own),
                        f"改前 {before}；改库之后 {library_edited}")
        selftest.expect(NOT_JUDGING_JUDGE_CHANGED_CELL, all(judge_edited[name] not in (None, program_edited[name]) for name in others),
                        f"改判法之前 {program_edited}；改判法之后 {judge_edited}")
    finally:
        shutil.rmtree(sample, ignore_errors=True)


def run_caller_input_key_cells(selftest, work, environment):
    """单元键样本仓里另写几份文件（CALLER_INPUT_SAMPLE_FILES，格完删掉）判六格：REQUEST_FILE_CELL（函数单元库的 FUNCTION_UNITS_BREAK=python-parameter-callers-unresolved、
    python-data-reads-ignored 下红）、TOOLCHAIN_QUERY_KEY_CELL（函数单元库的 FUNCTION_UNITS_BREAK=python-toolchain-queries-keyed 下红：询问收工具链单元，换一版判法键变）、PROGRAM_ARGUMENTS_KEY_CELL（program-arguments-unkeyed 下红）、
    SHELL_PROGRAM_ARGUMENTS_NARROWED_CELL（shell-program-arguments-dropped 与函数单元库的 FUNCTION_UNITS_BREAK=shell-entry-arguments-unnarrowed 下红）、SHELL_PROGRAM_ARGUMENTS_KEY_CELL（program-arguments-unkeyed 下红）、
    ROW_KEYED_JUDGE_CELL（函数单元库的 FUNCTION_UNITS_BREAK=row-keyed-tables-whole 下红）。"""
    for path, text in CALLER_INPUT_SAMPLE_FILES.items():
        write_text(os.path.join(work, path), text)
    request = Request("request-judged", JUDGE_ONLY_ENTRY, ["judge:scripts/request_judge.py::judge_request", "file:requests/one.tsv"])

    def judged(judge):
        with EnvironmentPatch({name: environment.get(name) for name in SAMPLE_PATCHED_VARIABLES}):
            context = KeyContext(work, environment)
            return context.judge_key(judge)[0], compute_unit_key(context, request).key
    try:
        judge_before, request_before = judged("scripts/request_judge.py::judge_request")
        edit_once(os.path.join(work, "requests/one.tsv"), "alpha", "beta")
        judge_input, request_input = judged("scripts/request_judge.py::judge_request")
        edit_once(os.path.join(work, "scripts/request_rules.tsv"), "rule", "rules")
        judge_rules, _request_rules = judged("scripts/request_judge.py::judge_request")
        selftest.expect(REQUEST_FILE_CELL,
                        None not in (judge_before, request_before) and judge_input == judge_before and request_input not in (None, request_before)
                        and judge_rules not in (None, judge_input),
                        f"判法键 改前 {judge_before}、改请求文件后 {judge_input}、改规则表后 {judge_rules}；调用方那一条 改前 {request_before}、改请求文件后 {request_input}")
        compiler_before = judged("scripts/compiler_judge.py::judge_version")[0]
        write_text(os.path.join(work, FAKE_RUSTC_VERSION_FILE), "1.98.0-selftest\n")
        compiler_after = judged("scripts/compiler_judge.py::judge_version")[0]
        os.remove(os.path.join(work, FAKE_RUSTC_VERSION_FILE))
        selftest.expect(TOOLCHAIN_QUERY_KEY_CELL, compiler_before is not None and compiler_after == compiler_before,
                        f"判法键 换工具链前 {compiler_before}、换后 {compiler_after}")
        argument_keys = sample_keys(work, environment, [Request("selftest", "scripts/tool.py", ["program-argument:--selftest"]),
                                                        Request("other", "scripts/tool.py", ["program-argument:--other"])])
        selftest.expect(PROGRAM_ARGUMENTS_KEY_CELL, None not in argument_keys.values() and argument_keys["selftest"] != argument_keys["other"], f"{argument_keys}")
        shell_requests = [Request("run", "scripts/dispatch.sh", ["program-argument:run"]), Request("selftest", "scripts/dispatch.sh", ["program-argument:--selftest"])]
        shell_before = sample_keys(work, environment, shell_requests)
        edit_once(os.path.join(work, "scripts/selftest_child.sh"), "echo selftest", "echo selftest changed")
        shell_after = sample_keys(work, environment, shell_requests)
        selftest.expect(SHELL_PROGRAM_ARGUMENTS_NARROWED_CELL, None not in shell_before.values() and shell_after["run"] == shell_before["run"]
                        and shell_after["selftest"] not in (None, shell_before["selftest"]), f"改前 {shell_before}；改 --selftest 那一支起的脚本之后 {shell_after}")
        plain_keys = sample_keys(work, environment, [Request("selftest", "scripts/plain.sh", ["program-argument:--selftest"]),
                                                     Request("other", "scripts/plain.sh", ["program-argument:--other"])])
        selftest.expect(SHELL_PROGRAM_ARGUMENTS_KEY_CELL, None not in plain_keys.values() and plain_keys["selftest"] != plain_keys["other"], f"{plain_keys}")
        registry_request = [Request("one", "scripts/one.py", ["judge:scripts/registry_judge.py::judge"])]
        registry_before = sample_keys(work, environment, registry_request)["one"]
        edit_once(os.path.join(work, ".claude/hooks/selftest-registry.tsv"), "# 乙", "# 乙改")
        registry_other = sample_keys(work, environment, registry_request)["one"]
        edit_once(os.path.join(work, ".claude/hooks/selftest-registry.tsv"), "# 甲", "# 甲改")
        registry_own = sample_keys(work, environment, registry_request)["one"]
        selftest.expect(ROW_KEYED_JUDGE_CELL, registry_before is not None and registry_other == registry_before and registry_own not in (None, registry_before),
                        f"改前 {registry_before}；改别的脚本那一行之后 {registry_other}；改它自己那一行之后 {registry_own}")
    finally:
        for path in CALLER_INPUT_SAMPLE_FILES:
            if os.path.exists(os.path.join(work, path)):
                os.remove(os.path.join(work, path))


def run_runner_script_cells(selftest):
    """cargo 配置里 target.<…>.runner 指的程序与它参数里的脚本进工具链行（KeyContext.rust_toolchain_lines 的构建环境那一截），不进单元键（文件头「单元键」）：
    runner 写 `bash tools/runner.sh`，脚本在 .cargo 所在那一层（仓根）下，改脚本里一个字 ⇒ 工具链行变、三条测试的单元键都不变；
    runner 写 `bash package-runner.sh`，脚本只在登记的崩溃枚举用例的包目录 crates/demo 下（cargo 在包目录起 runner），改它 ⇒ 同上。
    弄坏开关 runner-program-only（只读程序本身）下两格都红，runner-arguments-from-configuration-directory-only（参数只从 .cargo 那一层解）下第二格红，
    skip-build-environment（构建环境那一截不进工具链行）下两格都红。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-runner-script-")
    try:
        environment = build_unit_sample_repository(work)
        configuration = os.path.join(work, ".cargo/config.toml")

        def toolchain_lines_change(label, script, old, new):
            keys_before, lines_before = sample_keys(work, environment), toolchain_lines_of(work, environment)[TOOLCHAIN_LINES_NAME]
            edit_once(script, old, new)
            try:
                keys_after, lines_after = sample_keys(work, environment), toolchain_lines_of(work, environment)[TOOLCHAIN_LINES_NAME]
            finally:
                edit_once(script, new, old)
            keys_changed = sorted(short_name(name) for name in keys_before if keys_before[name] is None or keys_after.get(name) != keys_before[name])
            selftest.expect(f"单元键：{label} ⇒ 工具链行变、三条测试的单元键都不变", not keys_changed and lines_before is not None and lines_after not in (None, lines_before),
                            f"单元键变了（或算不出）的 {keys_changed or '无'}；工具链行{'变了' if lines_after != lines_before else '没变'}，改后 {str(lines_after)[:400]}")

        root_script = os.path.join(work, "tools/runner.sh")
        write_text(root_script, '#!/usr/bin/env bash\nexec "$@"\n')
        write_text(configuration, '[target.x86_64-unknown-linux-gnu]\nrunner = "bash tools/runner.sh"\n')
        toolchain_lines_change("cargo 配置 runner 写 `bash tools/runner.sh`，参数里的脚本（.cargo 那一层下）改一个字（runner-script-argument）",
                        root_script, "exec \"$@\"", "exec \"$@\" --quiet")
        package_script = os.path.join(work, "crates/demo/package-runner.sh")
        write_text(package_script, '#!/usr/bin/env bash\nexec "$@"\n')
        write_text(configuration, '[target.x86_64-unknown-linux-gnu]\nrunner = "bash package-runner.sh"\n')
        registration = os.path.join(work, REGISTRATION_TABLE)
        edit_once(registration, "other-gate.sh\tscripts/other.py\t# 别的一道\n",
                  "other-gate.sh\tscripts/other.py\t# 别的一道\ncrash-case:sample-runner\t-\ttest=demo:setting:test_t\t# 样本：登记的用例在包 demo\n")
        toolchain_lines_change("cargo 配置 runner 写 `bash package-runner.sh`，脚本只在登记用例的包目录 crates/demo 下，改一个字（runner-script-in-package-directory）",
                        package_script, "exec \"$@\"", "exec \"$@\" --quiet")
    except AssertionError as error:
        selftest.expect("runner 脚本那两格的样本改得动", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)


# 读缓存数据文件那一半的样本（test_plan_data_file_memo_hits）：单元键样本仓的 demo 包加一份 include_str! 读的 table.txt，test_t（经 helper）与
# test_v（经 formats）两条的 Rust 闭包都读它；不带 --profile，清单那一半走不到，记到的命中只来自数据文件单元
DATA_FILE_MEMO_EDITS = (
    ("crates/demo/src/lib.rs", "pub const WIDTH: u32 = 88;\n", "pub const WIDTH: u32 = 88;\npub const TABLE: &str = include_str!(\"table.txt\");\n"),
    ("crates/demo/src/lib.rs", "    WIDTH + 1\n", "    WIDTH + TABLE.len() as u32\n"),
    ("crates/demo/src/lib.rs", "itoa::Buffer::new().format(value).to_string()", "itoa::Buffer::new().format(value + TABLE.len() as u32).to_string()"),
)
DATA_FILE_MEMO_TABLE = ("crates/demo/src/table.txt", "row\n")


def test_plan_data_file_memo_hits(module):
    """单线程、不带 --profile 在 DATA_FILE_MEMO_EDITS 改过、补了 DATA_FILE_MEMO_TABLE 再提交的样本仓上跑一趟 test-plan → (退出码, 记的 reader-memo-hits（没记交 -1）, 说明)。
    两条测试函数的闭包各读一次 table.txt：数据文件单元按路径只读一次时后读的那一条命中，至少 1 次；读缓存只换在总文件上、拆出的几份各调自己那一份时是 0。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-test-plan-data-file-memo-")
    try:
        environment = build_unit_sample_repository(work)
        for path, old, new in DATA_FILE_MEMO_EDITS:
            edit_once(os.path.join(work, path), old, new)
        write_text(os.path.join(work, DATA_FILE_MEMO_TABLE[0]), DATA_FILE_MEMO_TABLE[1])
        run_quietly(["git", "-C", work, "add", "-A"], GIT_IDENTITY)
        run_quietly(["git", "-C", work, "commit", "-qm", "data file read by two tests"], GIT_IDENTITY)
        plan_file, workers_file = os.path.join(work, "plan.data-file"), os.path.join(work, "workers.data-file")
        changes = {THREAD_CAP_VARIABLE: "1", TEST_PLAN_WORKERS_FILE_VARIABLE: workers_file, "SINGLEFS_REUSE_SENTINEL_SEED": "1"}
        exit_code, _output, messages = run_in_environment([sys.executable, module, "test-plan", work, "demo-gate.sh:test-plan-parallel", "demo", plan_file],
                                                          dict(environment, **changes))
        workers = open(workers_file, encoding="utf-8").read().strip() if os.path.exists(workers_file) else ""
        found = re.search(r"\breader-memo-hits=(\d+)\b", workers)
        return exit_code, int(found.group(1)) if found else -1, f"退 {exit_code}、记的「{workers or '没写'}」、stderr「{messages.strip()[-200:]}」"
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_test_plan_parallel_cells(selftest, module):
    """test-plan 分进子进程算（文件头 test-plan「并行」）：同一份输入线程数 1（本进程里算）与 4（分进子进程）各跑一趟，stdout 与计划文件逐字相同；
    线程数 4 那一趟真分到了子进程（ADMISSION_TEST_PLAN_WORKERS_FILE 写的 processes= 大于 1）。
    同一个文件另记内存那两项：单线程那一趟算完 rust_closures 是空的（每条放掉闭包）；并行那一趟 fork 之前父进程算了一条、对象冻进了永久代。
    两趟都带 --profile release：三条测试函数各经 rust_cfg_for_profile 读同一份仓根 Cargo.toml，单线程那一趟的读缓存记得到命中（reader-memo-hits）；
    数据文件那一半另在两条测试读同一份 include_str! 文件的样本仓上单线程、不带 --profile 跑一趟（test_plan_data_file_memo_hits），命中至少 1 次；
    两样合成一格「test-plan：一趟里清单与数据文件按路径只读一次」，弄坏开关 test-plan-file-readers-unmemoized 下两样都是 0、那一格红。
    弄坏开关 test-plan-single-process 下头一格红（改回一个进程串行）；test-plan-closures-kept、test-plan-parent-not-prewarmed 下各自那一格红。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-test-plan-parallel-")
    try:
        environment = build_unit_sample_repository(work)
        runs = {}
        for threads in ("1", "4"):
            plan_file, workers_file = (os.path.join(work, f"{name}.{threads}") for name in ("plan", "workers"))
            # 复用哨兵的种子钉死：不钉每趟随机，打的那一行两趟不同
            changes = {THREAD_CAP_VARIABLE: threads, TEST_PLAN_WORKERS_FILE_VARIABLE: workers_file,
                       "SINGLEFS_REUSE_SENTINEL_SEED": "1"}
            # --profile release：每条测试函数算键时 rust_cfg_for_profile 各读一遍仓根 Cargo.toml（[profile.release] 定 debug_assertions 开不开），
            # 三条读同一份清单，读缓存那一格（reader-memo-hits）才走得到重复读
            exit_code, output, messages = run_in_environment([sys.executable, module, "test-plan", work, "demo-gate.sh:test-plan-parallel", "demo", plan_file,
                                                              "--profile", "release"],
                                                             dict(environment, **changes))
            plan_text = open(plan_file, encoding="utf-8").read() if os.path.exists(plan_file) else None
            workers = open(workers_file, encoding="utf-8").read().strip() if os.path.exists(workers_file) else ""
            runs[threads] = (exit_code, output, plan_text, workers, messages)
        single, parallel = runs["1"], runs["4"]
        processes = re.search(r"processes=(\d+)", parallel[3])
        selftest.expect("test-plan：同一份输入单线程与并行算出的 stdout 与计划文件逐字相同，并行那一趟真分到了子进程",
                        single[0] == 0 and parallel[0] == 0 and single[1] == parallel[1] and single[2] is not None and single[2] == parallel[2]
                        and "PLAN judge=3" in single[1] and bool(processes and int(processes.group(1)) > 1),
                        f"单线程退 {single[0]}、并行退 {parallel[0]}；stdout 相同 {single[1] == parallel[1]}、计划文件相同 {single[2] == parallel[2]}；"
                        f"并行那一趟记的「{parallel[3] or '没写'}」；单线程 stdout 尾部「{single[1].strip()[-200:]}」；stderr「{parallel[4].strip()[-200:]}」")
        selftest.expect("test-plan：每条算完键放掉它的 Rust 闭包，单线程那一趟算完 rust_closures 是空的",
                        re.search(r"\brust-closures-left=0\b", single[3]) is not None, f"单线程那一趟记的「{single[3] or '没写'}」")
        frozen = re.search(r"\bcomputed-before-fork=(\d+) frozen-before-fork=(\d+)\b", parallel[3])
        selftest.expect("test-plan：并行之前父进程先算一条，再把对象冻进永久代（子进程共用判法索引与工具链行）",
                        bool(frozen and int(frozen.group(1)) == 1 and int(frozen.group(2)) > 0), f"并行那一趟记的「{parallel[3] or '没写'}」")

        def recorded(run, name):
            found = re.search(rf"\b{name}=(\d+)\b", run[3])
            return int(found.group(1)) if found else -1

        selftest.expect("test-plan：并行之前父进程铺好全部单元的行表、源文件行与字面串，子进程不各读一份",
                        recorded(parallel, "prefilled-units") > 0, f"并行那一趟记的「{parallel[3] or '没写'}」")
        selftest.expect("test-plan：并行之前词串进驻留表、行号共用一个对象", recorded(parallel, "tokens-shared") > 0, f"并行那一趟记的「{parallel[3] or '没写'}」")
        selftest.expect("test-plan：并行之前把父进程的对象标成永生，子进程读它们不写引用计数",
                        recorded(parallel, "immortal-objects") > 0 or sys.version_info[:2] != (3, 12), f"并行那一趟记的「{parallel[3] or '没写'}」")
        selftest.expect("test-plan：子进程交回的组成清单是压过的下标串", recorded(parallel, "compacted") > 0, f"并行那一趟记的「{parallel[3] or '没写'}」")
        data_exit, data_hits, data_detail = test_plan_data_file_memo_hits(module)
        selftest.expect("test-plan：一趟里清单与数据文件按路径只读一次", recorded(single, "reader-memo-hits") > 0 and data_exit == 0 and data_hits >= 1,
                        f"清单：单线程那一趟记的「{single[3] or '没写'}」；数据文件：两条测试读同一份 table.txt 的那一趟命中 {data_hits} 次（要至少 1 次），{data_detail}")
        planned_files = {relative for line in (parallel[2] or "").split("\n") if line for relative in json.loads(line)["files"]}
        selftest.expect("test-plan：计划文件里的文件摘要同一份文件一趟只算一次",
                        bool(planned_files) and recorded(parallel, "file-digest-reads") == len(planned_files),
                        f"计划文件点到 {len(planned_files)} 份，并行那一趟记的「{parallel[3] or '没写'}」")
        selftest.expect("test-plan：查库与存组成清单分进子进程", recorded(parallel, "lookups-in-children") > 0, f"并行那一趟记的「{parallel[3] or '没写'}」")
        # 线程数取不了（SINGLEFS_THREAD_CAP 写坏）：整次列不出、退 2、不写计划文件，不退回 1 个子进程照算（弄坏开关 test-plan-jobs-fallback-one 下这一格红）
        refused_plan = os.path.join(work, "plan.unreadable-threads")
        exit_code, output, messages = run_in_environment([sys.executable, module, "test-plan", work, "demo-gate.sh:test-plan-parallel", "demo", refused_plan],
                                                         dict(environment, **{THREAD_CAP_VARIABLE: "abc", "SINGLEFS_REUSE_SENTINEL_SEED": "1"}))
        selftest.expect("test-plan：取不了每台默认重活线程数（SINGLEFS_THREAD_CAP=abc）⇒ 整次列不出、退 2，不退回 1 个子进程照算",
                        exit_code == EXIT_REGISTRATION_ERROR and "取不了每台默认重活线程数" in output + messages and not os.path.exists(refused_plan),
                        f"退 {exit_code}、计划文件{'写了' if os.path.exists(refused_plan) else '没写'}；stdout 尾部「{output.strip()[-200:]}」；stderr「{messages.strip()[-200:]}」")
    except AssertionError as error:
        selftest.expect("test-plan 并行那一格的样本搭得起来", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_composition_packing_cell(selftest):
    """组成清单压完再解开（composition_packed、composition_unpacked）：单元行、行表外的行（带不是合法 UTF-8 的字节）、次序交错，逐行相同，文件名单与现算的相同。
    弄坏开关 test-plan-packing-literals-moved 下这一格红（行表外的几行挪到最前面）。"""
    table = ["fn\tb\tsrc/b.rs:1-2\t" + "1" * 64, "fn\ta\tsrc/a.rs:1-2\t" + "0" * 64]
    composition = [table[1], "cfg\ttest", table[0], "file\tresults/kept.out\t" + "2" * 64, table[1], "line\targuments=\udc80"]
    packed = composition_packed(composition, {line: position for position, line in enumerate(table)})
    lines, files = composition_unpacked(packed, table)
    selftest.expect("test-plan：组成清单压完再解开逐行相同，文件名单也相同",
                    lines == composition and sorted(files) == sorted(composition_file_names(composition)) and isinstance(packed, bytes),
                    f"解开 {lines}，文件名单 {files}")


def run_shell_index_filled_cell(selftest):
    """shell_index_filled 与 ShellIndex 逐项相同：entry.sh source 的路径取的变量赋在 settings.sh 里（要再算一趟才折得出），另带起别的脚本、内联 python 与 $(cat <<词 …)。
    弄坏开关 shell-index-rounds-one 下这一格红（只算一趟，entry.sh 到 worker.sh 的边没了）。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-shell-index-")
    try:
        tree = {"scripts/entry.sh": '#!/usr/bin/env bash\nsource scripts/settings.sh\nsource "$WORKER_PATH"\nbash scripts/worker.sh one\n'
                                    'python3 -c "print(1)" one\ncaptured="$(cat <<\'WORD\'\nscripts/worker.sh\nWORD\n)"\nbash "$captured"\n',
                "scripts/settings.sh": "#!/usr/bin/env bash\nWORKER_PATH=scripts/worker.sh\n",
                "scripts/worker.sh": "#!/usr/bin/env bash\nhelper() {\n  echo worker\n}\nhelper\n"}
        for path, text in tree.items():
            write_text(os.path.join(work, path), text)
        library = function_units()
        paths = library.shell_files_under(work, ["scripts"])

        def snapshot(index):
            units = {}
            for path, shell_file in index.files.items():
                for position, unit in enumerate(shell_file.units):
                    units[(path, position)] = None if unit.data else (unit.name, index.links(unit), unit.cache.get("launch_arguments"), unit.cache.get("python_launch_arguments"), unit.facts.programs)
            return (dict(index.sources_of), {path: set(sourcers) for path, sourcers in index.sourcers_of.items() if sourcers}, list(index.files), units,
                    {inner: set(values) for inner, values in index.substitution_outputs.items()})

        expected, (filled, rounds) = snapshot(library.ShellIndex(work, paths)), shell_index_filled(work, paths, 2, 1 if break_is_set("shell-index-rounds-one") else 3)
        selftest.expect("脚本索引：进程池建的与一个进程建的逐项相同，source 的边算到不再变",
                        snapshot(filled) == expected and "scripts/worker.sh" in expected[0]["scripts/entry.sh"],
                        f"算了 {rounds} 趟；一个进程建的边 {expected[0]}，进程池建的边 {filled.sources_of}")
    except Exception as error:  # 进程池或样本出错也是这一格红，别的格照跑
        selftest.expect("脚本索引：进程池建的与一个进程建的逐项相同，source 的边算到不再变", False, f"{type(error).__name__}：{error}")
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_replay_row_parallel_cell(selftest):
    """replay-row-inputs 的 shell 索引分进子进程（文件头 replay-row-inputs「并行」）：同一份样本线程数 1 与 4 各跑一趟，stdout 逐字相同，线程数 4 那一趟真分到了子进程。
    弄坏开关 replay-row-shell-index-serial 下这一格红。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-rows-parallel-")
    try:
        environment = build_research_sample_repository(work)
        copied = os.path.join(work, "scripts/gate/admission.py")
        runs = {}
        for threads in ("1", "4"):
            workers_file = os.path.join(work, f"row-workers.{threads}")
            exit_code, output, messages = run_in_environment([sys.executable, copied, "replay-row-inputs", work, "--gpu", "0"],
                                                             dict(environment, **{THREAD_CAP_VARIABLE: threads, "ADMISSION_REPLAY_ROW_WORKERS_FILE": workers_file}))
            note = open(workers_file, encoding="utf-8").read().strip() if os.path.exists(workers_file) else ""
            runs[threads] = (exit_code, output, note, messages)
        single, parallel = runs["1"], runs["4"]
        selftest.expect("replay-row-inputs：单线程与并行建脚本索引行键逐字相同，并行那一趟真分到了子进程",
                        single[0] == 0 and parallel[0] == 0 and single[1] == parallel[1] and bool(single[1].strip()) and parallel[2].startswith("processes=4 "),
                        f"单线程退 {single[0]}、并行退 {parallel[0]}；stdout 相同 {single[1] == parallel[1]}；并行那一趟记的「{parallel[2] or '没写'}」；stderr「{parallel[3].strip()[-300:]}」")
    except AssertionError as error:
        selftest.expect("replay-row-inputs 并行那一格的样本搭得起来", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_script_closure_sharing_cell(selftest):
    """KeyContext.script_closure 收过的闭包在一个 KeyContext（一趟）里共用、不跨文件内容：同一个入口再问交同一个对象；改了入口所在文件的内容、新建 KeyContext 再问，键跟着变。
    弄坏开关 script-closures-recollected 下「同一个 KeyContext 里…」那一句红（每问一遍收一遍），script-closures-process-wide 下「改了入口所在文件的内容…」那一句红
    （新的 KeyContext 拿到进程里记着的旧闭包）。"""
    shared_name = "脚本闭包：同一个 KeyContext 里同一个入口只收一次，再问交同一个对象"
    changed_name = "脚本闭包：改了入口所在文件的内容、新建 KeyContext 再问，闭包里的单元与键跟着变"
    work = tempfile.mkdtemp(prefix="admission-selftest-script-closures-")
    try:
        library = function_units()
        entry = "scripts/entry.py::run"
        text = "def helper():\n    return {value}\n\n\ndef run():\n    return helper()\n"
        write_text(os.path.join(work, "scripts/entry.py"), text.format(value=1))
        context = KeyContext(work)
        collected, asked_again = context.script_closure(work, entry), context.script_closure(work, entry)
        before = library.closure_key(collected, [], (), {})[0]
        write_text(os.path.join(work, "scripts/entry.py"), text.format(value=2))
        collected_after_edit = KeyContext(work).script_closure(work, entry)
        after = library.closure_key(collected_after_edit, [], (), {})[0]
        selftest.expect(shared_name, collected is asked_again and before is not None, f"再问交的是同一个对象 {collected is asked_again}，键 {before}")
        selftest.expect(changed_name, before is not None and after is not None and before != after and collected is not collected_after_edit,
                        f"改前键 {before}，改后键 {after}，新的 KeyContext 拿到的是旧对象 {collected is collected_after_edit}")
    except Exception as error:  # 样本或库出错也是这两格红
        for name in (shared_name, changed_name):
            selftest.expect(name, False, f"{type(error).__name__}：{error}")
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_composition_line_sharing_cell(selftest):
    """组成清单的单元行按（前缀, 单元）共用一个串对象（UNIT_LINE_TEXTS）：两份闭包都有同一个单元，两份清单里那一行是同一个对象，前缀变了的另算一个。
    弄坏开关 composition-lines-unshared 下这一格红（每份清单现拼一个新串）。"""
    library = function_units()
    unit = library.Unit("python", "def", "shared", "scripts/shared.py", 1, 2, "0" * 64)
    closures = [library.Closure(), library.Closure()]
    for closure in closures:
        closure.add(unit, None, "entry")
    listing, other_listing = closure_unit_lines(closures[0]), closure_unit_lines(closures[1])
    judged = closure_unit_lines(closures[0], prefix="judge ")
    selftest.expect("组成清单：两份闭包里同一个单元的那一行共用一个串对象，内容与现拼的相同，带前缀的另算",
                    listing == [unit.text()] and listing[0] is other_listing[0] and judged == ["judge " + unit.text()],
                    f"一份 {listing}，另一份 {other_listing}，带前缀 {judged}，同一个对象 {listing[0] is other_listing[0]}")


def run_test_plan_exclusion_cells(selftest, module):
    """test-plan --exclude-registered（文件头 test-plan）：登记成 crash-case: 的用例（test_u）与放量分项（test_v，ITEMS 的测试目标与用例函数）各记一行 registered，
    不算键、不进判定，PLAN 末行报 registered=2，plan-skip-names 把它们列进 --skip；不带这个参数三条全判。弄坏开关 exclude-registered-ignored 下这一格红。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-test-plan-exclusion-")
    try:
        environment = build_unit_sample_repository(work)
        edit_once(os.path.join(work, REGISTRATION_TABLE), "other-gate.sh\tscripts/other.py\t# 别的一道\n",
                  "other-gate.sh\tscripts/other.py\t# 别的一道\ncrash-case:sample-excluded\t-\ttest=demo:setting:test_u count-line=X\t# 样本：登记的用例\n")
        write_text(os.path.join(work, "scripts/crash/crash-amplification.sh"), "ITEMS=(\n  $'sample-piped\\tsetting\\ttest_v'\n)\n")
        plan_file, output_file = os.path.join(work, "exclusion.plan"), os.path.join(work, "exclusion.out")

        def plan(*extra, changes=None):
            exit_code, output, _messages = run_in_environment(
                [sys.executable, module, "test-plan", work, "demo-gate.sh:exclusion", "demo", plan_file, *extra], dict(environment, **(changes or {})))
            rows = {short_name(line.split("\t")[0]): line.split("\t")[1] for line in output.split("\n") if line.count("\t") >= 6}
            return exit_code, rows, output

        exit_code, rows, output = plan("--exclude-registered")
        write_text(output_file, output)
        _exit_code, skip_output, _messages = run_in_environment([sys.executable, module, "plan-skip-names", output_file], environment)
        selftest.expect("test-plan --exclude-registered：登记成 crash-case: 的用例与放量分项记 registered、不算键、进 --skip 名单，别的照判",
                        exit_code == 0 and rows == {"test_t": "judge", "test_u": "registered", "test_v": "registered"} and "registered=2" in output
                        and skip_output.split() == ["test_u", "test_v"],
                        f"退 {exit_code}，{rows}，--skip 名单 {skip_output.split()}，{output.strip()[-200:]}")
        exit_code, rows, output = plan()
        selftest.expect("test-plan：不带 --exclude-registered ⇒ 登记的用例与放量分项也照判，没有 registered",
                        exit_code == 0 and rows == {"test_t": "judge", "test_u": "judge", "test_v": "judge"} and "registered=0" in output, f"退 {exit_code}，{rows}")
        exit_code, _rows, _output = plan("--" + "full")
        selftest.expect("test-plan：带已删的整趟现判参数 ⇒ 当认不出的参数，退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}")
        _exit_code, rows, output = plan("--exclude-registered", changes={BREAK_VARIABLE: "exclude-registered-ignored"})
        selftest.expect("弄坏开关 exclude-registered-ignored 下「登记成 crash-case: 的用例与放量分项记 registered」那一格转红（三条都判）",
                        rows == {"test_t": "judge", "test_u": "judge", "test_v": "judge"}, f"{rows}")
    except AssertionError as error:
        selftest.expect("test-plan 排除登记用例那一格的样本搭得起来", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_plan_skip_names_cells(selftest, module):
    """plan-skip-names（文件头）：门禁分片的一片不起别的片领的测试目标时给 --except-target-roots-file。手写的计划输出八行：tests/started_reuse.rs 里 only_started、shared（reuse），
    tests/started_judge.rs 里 shared（judge，这一片的），tests/dropped.rs（不起的目标）里 only_dropped、shared（registered）与 pair（judge），tests/not_compiled.rs 里 tests::x（not-compiled），
    tests/pair_reuse.rs 里 pair（reuse）：不给选项 ⇒ 名单是 only_dropped、only_started、tests::x（shared、pair 因同名的有现判不收）；给 tests/dropped.rs ⇒ only_started、tests::x
    （不起的目标里的 only_dropped 不收；pair 的现判行在不起的目标里也照旧让 pair_reuse.rs 的 pair 不收：现判的同名看全部目标，与不分片时相同）。
    弄坏开关 skip-names-of-dropped-targets-kept 下这一格红（不起的目标的名字照收）。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-plan-skip-names-")
    try:
        columns = [("only_started", "reuse", "tests/started_reuse.rs"), ("shared", "reuse", "tests/started_reuse.rs"), ("shared", "judge", "tests/started_judge.rs"),
                   ("only_dropped", "registered", "tests/dropped.rs"), ("shared", "registered", "tests/dropped.rs"), ("tests::x", "not-compiled", "tests/not_compiled.rs"),
                   ("pair", "reuse", "tests/pair_reuse.rs"), ("pair", "judge", "tests/dropped.rs")]
        plan_path, except_path = os.path.join(work, "plan.out"), os.path.join(work, "except.txt")
        write_text(plan_path, "".join("\t".join([name, state, "-", "demo:test:" + root, root, name, "样本"]) + "\n" for name, state, root in columns)
                   + "PLAN judge=2 reuse=3 refused=0 not-compiled=1\n")
        write_text(except_path, "tests/dropped.rs\n\n")
        environment = dict(os.environ)

        def listed(*extra, changes=None):
            exit_code, output, _messages = run_in_environment([sys.executable, module, "plan-skip-names", plan_path, *extra], dict(environment, **(changes or {})))
            return exit_code, output.split()

        everything = listed()
        narrowed = listed("--except-target-roots-file", except_path)
        selftest.expect("plan-skip-names：不给选项 ⇒ 全部目标里 reuse、not-compiled、registered 的名字，减去同名而要现判的",
                        everything == (0, ["only_dropped", "only_started", "tests::x"]), f"{everything}")
        selftest.expect("plan-skip-names --except-target-roots-file：不起的目标（tests/dropped.rs）的名字不收，别的照旧，同名而要现判的看全部目标",
                        narrowed == (0, ["only_started", "tests::x"]), f"{narrowed}")
        broken = listed("--except-target-roots-file", except_path, changes={BREAK_VARIABLE: "skip-names-of-dropped-targets-kept"})
        selftest.expect("弄坏开关 skip-names-of-dropped-targets-kept 下「不起的目标的名字不收」那一格转红（照收）", broken == everything, f"{broken}")
        missing = listed("--except-target-roots-file", os.path.join(work, "absent.txt"))
        selftest.expect("plan-skip-names --except-target-roots-file：指的文件不在 ⇒ 退 2、不打名单",
                        missing[0] == EXIT_REGISTRATION_ERROR and not {"only_started", "only_dropped", "tests::x"} & set(missing[1]), f"{missing}")
    except AssertionError as error:
        selftest.expect("plan-skip-names 那一格的样本搭得起来", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_test_plan_closure_key_cells(selftest, module):
    """test-plan 给每条测试函数的键只收它自己的单元闭包：同一设点文件里改 test_t 闭包里的 observer，只 test_t 的键变，test_u、test_v 的键不变。
    弄坏开关 test-key-with-whole-file 下这一格红（每条的键另加整份设点文件，三条都变）；本进程里另开着它跑一遍，核这一格分得出差别。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-test-plan-closure-")
    names = ("test_t", "test_u", "test_v")
    try:
        environment = build_unit_sample_repository(work)

        def planned_keys(label, changes=None):
            plan_file = os.path.join(work, f"closure-{label}.plan")
            run_changes = {THREAD_CAP_VARIABLE: "1", "SINGLEFS_REUSE_SENTINEL_SEED": "1", **(changes or {})}
            exit_code, output, messages = run_in_environment([sys.executable, module, "test-plan", work, "demo-gate.sh:test-plan-closure", "demo", plan_file],
                                                             dict(environment, **run_changes))
            rows = [line.split("\t") for line in output.split("\n") if line.count("\t") >= 3]
            keys = {short_name(columns[0]): columns[2] for columns in rows if columns[1] in ("judge", "reuse", "refused")}
            return exit_code, keys, (output + messages).strip()[-300:]

        broken = {BREAK_VARIABLE: "test-key-with-whole-file"}
        before = planned_keys("before")
        broken_before = planned_keys("broken-before", broken)
        edit_once(os.path.join(work, "crates/demo/tests/setting.rs"), "helper() * 2", "helper() * 2 + 0")
        after = planned_keys("after")
        broken_after = planned_keys("broken-after", broken)

        def changed_names(first, second):
            """两趟都退 0、三条都算出了键时交键变了的那几条（排好序）；不是就交 None。"""
            if first[0] != 0 or second[0] != 0 or any(first[1].get(name, "-") == "-" or second[1].get(name, "-") == "-" for name in names):
                return None
            return sorted(name for name in names if first[1][name] != second[1][name])

        changed = changed_names(before, after)
        selftest.expect("test-plan：同一设点文件里只改 test_t 闭包里的 observer ⇒ 只 test_t 的键变，test_u 与 test_v 的键不变", changed == ["test_t"],
                        f"键变了的 {changed}（None 是有一趟没退 0 或有一条没算出键）；改前「{before[2]}」；改后「{after[2]}」")
        broken_changed = changed_names(broken_before, broken_after)
        selftest.expect("弄坏开关 test-key-with-whole-file 下「同一设点文件里只改 test_t 闭包里的 observer」那一格转红（三条的键都变）",
                        broken_changed == list(names), f"弄坏之后键变了的 {broken_changed}；改前「{broken_before[2]}」；改后「{broken_after[2]}」")
    except AssertionError as error:
        selftest.expect("test-plan 闭包键那一格的样本改得动", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_verdict_library_cells(selftest, module):
    """判定库：头一趟全现判、记绿之后第二趟全复用；改一个被调函数只那一条现判、原因点名变了的单元；全量只写不读；跑的过程中键变了的不记；
    门禁的复用答法：全部有判定退 10、有一条对不上退 0 并点名；H2（改动带着 HEAD 一起前移、staged-green 跟着挪）必须现判，H3（空提交）与 E01（只换工具链）照复用；
    弄坏开关 tree-compare-reuse 下 H2 那一格转红。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-library-")
    try:
        environment = build_unit_sample_repository(work)
        # 复用哨兵钉成默认的 3 条：这几格的库里至多 3 条，都当哨兵、不抽强制现判，现判与复用的条数照下面写的判（哨兵那一格在 run_reuse_three_question_cells）
        environment.update({"SINGLEFS_REUSE_SENTINEL_COUNT": "3", "SINGLEFS_LAST_RUNS_DIRECTORY": os.path.join(work, ".last-runs")})
        requests_file = os.path.join(work, "demo.requests")
        plan_file = os.path.join(work, "demo.plan")
        names_file = os.path.join(work, "demo.names")
        write_text(requests_file, "".join(request.text() + "\n" for request in unit_sample_requests()))

        def admission(*arguments, changes=None):
            return run_in_environment([sys.executable, module, *arguments], dict(environment, **(changes or {})))

        def plan(changes=None):
            exit_code, output, messages = admission("verdict-plan", work, "demo-gate.sh", requests_file, plan_file, changes=changes)
            decisions = {short_name(line.split("\t")[0]): line.split("\t")[1] for line in output.split("\n") if line.count("\t") >= 3}
            return exit_code, decisions, output + messages

        def record(names, changes=None):
            write_text(names_file, "".join(f"crates/demo/tests/setting.rs::{name}\n" for name in names))
            return admission("verdict-record", work, "demo-gate.sh", plan_file, names_file, "--how", "cargo test --release -p demo（自证）", changes=changes)

        exit_code, decisions, output = plan()
        selftest.expect("verdict-plan：库是空的 ⇒ 三条都 judge，末行 PLAN judge=3 reuse=0 refused=0",
                        exit_code == 0 and decisions == {"test_t": "judge", "test_u": "judge", "test_v": "judge"} and "PLAN judge=3 reuse=0 refused=0" in output,
                        f"退 {exit_code}，{decisions}，{output.strip()[-300:]}")
        exit_code, output, _messages = record(["test_t", "test_u", "test_v"])
        selftest.expect("verdict-record：三条判绿都记进库（RECORD recorded=3 changed=0）", exit_code == 0 and "RECORD recorded=3 changed=0" in output,
                        f"退 {exit_code}，{output.strip()}")
        exit_code, decisions, output = plan()
        selftest.expect("verdict-plan：记过之后第二趟三条都 reuse", decisions == {"test_t": "reuse", "test_u": "reuse", "test_v": "reuse"}, f"{decisions}，{output.strip()[-300:]}")
        # 库里的键照这一种类重开一份数（数的是临时服务里的，文件头「判定库」；本地目录不建）
        demo_library = VerdictLibrary(work, "demo-gate.sh")
        record_path = demo_library.directory
        records = demo_library.stored_keys()
        sample_record = (demo_library.load_stored_json(demo_library.path_of_key(records[0])) or {}) if records else {}
        composition = demo_library.composition_of(sample_record.get("key", "")) or []
        local_library_files = [name for _walked, _directories, names in os.walk(os.path.join(work, ".git", VERDICT_LIBRARY_DIRECTORY_NAME)) for name in names]
        selftest.expect("判定库在 prune-server 里这一种类那一份（名字取 git common-dir 的 singlefs-unit-verdicts/<种类>，本地不建），每条记着键、判定日期与跑法，"
                        "组成清单按键另存（compositions/<键>.tsv.gz）",
                        len(records) == 3 and re.fullmatch(r"\d{4}-\d{2}-\d{2}", sample_record.get("judged", "")) is not None
                        and sample_record.get("how", "").startswith("cargo test") and any(line.startswith("fn\t") for line in composition)
                        and not local_library_files,
                        f"{record_path} 里 {records}，样本记录的键 {sorted(sample_record)}，组成清单 {len(composition)} 行，本地判定文件 {local_library_files[:3]}")
        edit_once(os.path.join(work, "crates/demo/src/lib.rs"), "WIDTH + 1", "WIDTH + 2")
        exit_code, decisions, output = plan()
        reason = next((line.split("\t", 3)[3] for line in output.split("\n") if "::test_t\t" in line), "")
        selftest.expect("verdict-plan：改了被调函数 helper ⇒ 只 test_t 现判，原因点名「变了 fn crates/demo/src/lib.rs::helper」",
                        decisions == {"test_t": "judge", "test_u": "reuse", "test_v": "reuse"} and "变了 fn crates/demo/src/lib.rs::helper" in reason,
                        f"{decisions}，原因「{reason}」")
        # 组成清单的回收：此刻库里四份（三条判绿的键各一份，加 test_t 刚算出的 WIDTH + 2 那一把）
        def stored_compositions():
            return {relative for relative, _value in demo_library.store.scan("compositions/")}

        def verdict_files():
            return sorted(relative for prefix in ("keys/", "items/") for relative, _value in demo_library.store.scan(prefix))

        green_compositions = {demo_library.path_of_composition(key) for key in records}
        test_t_name, test_t_green = next(((name, record.get("key", "")) for name, record in demo_library.items() if str(name).endswith("::test_t")), ("", ""))
        after_first_edit = stored_compositions()
        never_green = after_first_edit - green_compositions
        verdicts_before = verdict_files()
        kept_through = []
        for earlier_width, later_width in ((2, 5), (5, 6)):
            edit_once(os.path.join(work, "crates/demo/src/lib.rs"), f"WIDTH + {earlier_width}", f"WIDTH + {later_width}")
            plan()
            kept_through.append(stored_compositions())
        edit_once(os.path.join(work, "crates/demo/src/lib.rs"), "WIDTH + 6", "WIDTH + 7")
        plan()
        after_fourth_key = stored_compositions()
        verdicts_after_plans = verdict_files()
        _exit_code, record_output, _messages = record(["test_t"])
        after_green_moved = stored_compositions()
        old_green_composition = demo_library.path_of_composition(test_t_green)
        selftest.expect(f"组成清单的回收：test_t 再算出两把新键（最近 {LATEST_COMPOSITIONS_KEPT} 把以内）⇒ 一份都不删；算出第四把 ⇒ 只删挤出去、没判绿过的那一把（WIDTH + 2），"
                        "判绿的三把都在、keys/ 与 items/ 一条不少；新键判绿 ⇒ 换下来、已不在最近几把里的原判绿那一把的组成清单才删，别的名字的不动",
                        len(after_first_edit) == 4 and len(never_green) == 1 and test_t_green != ""
                        and [len(found) for found in kept_through] == [5, 6] and all(after_first_edit <= found for found in kept_through)
                        and len(after_fourth_key) == 6 and not (never_green & after_fourth_key) and green_compositions <= after_fourth_key
                        and verdicts_after_plans == verdicts_before
                        and "RECORD recorded=1 changed=0" in record_output
                        and after_green_moved == after_fourth_key - {old_green_composition} and old_green_composition in after_fourth_key
                        and demo_library.read(demo_library.path_of_key(test_t_green)) is not None,
                        f"头一次改后 {len(after_first_edit)} 份（没判绿过的 {len(never_green)} 份），再算两把之后 {[len(found) for found in kept_through]} 份，第四把之后 {len(after_fourth_key)} 份"
                        f"（没判绿过的那一把还在 {len(never_green & after_fourth_key)} 份、判绿的在 {len(green_compositions & after_fourth_key)} 份），"
                        f"判定文件前后 {len(verdicts_before)}/{len(verdicts_after_plans)} 条，判绿后 {len(after_green_moved)} 份；{record_output.strip()}")
        edit_once(os.path.join(work, "crates/demo/src/lib.rs"), "WIDTH + 7", "WIDTH + 2")
        exit_code, _output, _messages = admission("verdict-plan", work, "demo-gate.sh", requests_file, plan_file, "--" + "full")
        selftest.expect("verdict-plan：带已删的整趟现判参数 ⇒ 当认不出的参数，退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}")
        plan()
        edit_once(os.path.join(work, "crates/demo/src/lib.rs"), "WIDTH + 2", "WIDTH + 3")
        exit_code, output, _messages = record(["test_t"])
        selftest.expect("verdict-record：开跑之后被判的函数又改了 ⇒ 不记、点名（changed=1）", "RECORD recorded=0 changed=1" in output, output.strip())
        edit_once(os.path.join(work, "crates/demo/src/lib.rs"), "WIDTH + 3", "WIDTH + 1")

        # 门禁的复用答法：H2 / H3 / staged-green 不读（controls L15）
        def reuse(changes=None, with_requests=True):
            arguments = ["gate-reuse", work, "demo-gate.sh"] + ([requests_file] if with_requests else [])
            return admission(*arguments, changes=changes)

        exit_code, output, _messages = reuse()
        selftest.expect("gate-reuse：登记的三条在库里都有键相同的判定 ⇒ 退 10 可跳过", exit_code == EXIT_GATE_MAY_SKIP, f"退 {exit_code}：{output.strip()}")
        exit_code, output, _messages = reuse(with_requests=False)
        selftest.expect("gate-reuse：不给请求文件（tests-of:demo 展开、按库里记的请求重算）⇒ 同样退 10", exit_code == EXIT_GATE_MAY_SKIP, f"退 {exit_code}：{output.strip()}")
        edit_once(os.path.join(work, "crates/demo/src/lib.rs"), "WIDTH + 1", "WIDTH + 2")
        run_quietly(["git", "-C", work, "commit", "-qam", "改动带着 HEAD 一起前移"], GIT_IDENTITY)
        mark_tree_green(work, staged_tree_of(work))
        exit_code, output, _messages = reuse()
        selftest.expect("gate-reuse：H2（改了 helper 并提交、refs/sop/staged-green 也挪到这一棵）⇒ 退 0 要跑，点名 test_t、报键对不上的 1 条",
                        exit_code == EXIT_GATE_MUST_RUN and "键对不上的 1 条" in output and "::test_t" in output, f"退 {exit_code}：{output.strip()}")
        exit_code, output, _messages = reuse({BREAK_VARIABLE: "tree-compare-reuse"})
        selftest.expect("弄坏开关 tree-compare-reuse 下 H2 那一格转红（按树比，答可跳过）", exit_code == EXIT_GATE_MAY_SKIP, f"退 {exit_code}：{output.strip()}")
        edit_once(os.path.join(work, "crates/demo/src/lib.rs"), "WIDTH + 2", "WIDTH + 1")
        run_quietly(["git", "-C", work, "commit", "-qam", "改回去"], GIT_IDENTITY)
        run_quietly(["git", "-C", work, "commit", "-q", "--allow-empty", "-m", "空提交"], GIT_IDENTITY)
        mark_tree_green(work, staged_tree_of(work))
        exit_code, output, _messages = reuse()
        selftest.expect("gate-reuse：H3（空提交，HEAD 前移、内容回到判过的样子）⇒ 退 10 照复用", exit_code == EXIT_GATE_MAY_SKIP, f"退 {exit_code}：{output.strip()}")
        write_text(os.path.join(work, FAKE_RUSTC_VERSION_FILE), "1.99.0-selftest\n")
        exit_code, output, _messages = reuse()
        os.remove(os.path.join(work, FAKE_RUSTC_VERSION_FILE))
        selftest.expect("gate-reuse：E01（只换工具链）⇒ 退 10 照复用（工具链版本不进键）", exit_code == EXIT_GATE_MAY_SKIP,
                        f"退 {exit_code}：{output.strip()}")
    except AssertionError as error:
        selftest.expect("判定库的样本改得动", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_reuse_fix_cells(selftest, module):
    """记红撤绿、库跟着哪棵仓、改判定域的环境变量进键、同短名的测试函数各算各的键（文件头「判定库」「单元键」「判定请求」）：
    弄坏开关 red-verdict-not-recorded、library-outside-git-tolerated、domain-environment-unkeyed、entry-homonyms-pulled 下各自那一格转红。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-reuse-fix-")
    copy_parent = tempfile.mkdtemp(prefix="admission-selftest-reuse-fix-copy-")
    try:
        environment = build_unit_sample_repository(work)
        environment.update({"SINGLEFS_REUSE_SENTINEL_COUNT": "3", "SINGLEFS_LAST_RUNS_DIRECTORY": os.path.join(work, ".last-runs")})
        requests_file, plan_file = os.path.join(work, "demo.requests"), os.path.join(work, "demo.plan")
        names_file, red_file = os.path.join(work, "demo.names"), os.path.join(work, "demo-red.names")
        write_text(requests_file, "".join(request.text() + "\n" for request in unit_sample_requests()))

        def admission(*arguments, changes=None):
            return run_in_environment([sys.executable, module, *arguments], dict(environment, **(changes or {})))

        def plan(root=work, kind="demo-gate.sh", changes=None):
            exit_code, output, messages = admission("verdict-plan", root, kind, requests_file, plan_file, changes=changes)
            rows = {short_name(line.split("\t")[0]): line.split("\t") for line in output.split("\n") if line.count("\t") >= 3}
            return exit_code, {name: columns[1] for name, columns in rows.items()}, {name: columns[3] for name, columns in rows.items()}, output + messages

        def record(green, red=(), root=work, kind="demo-gate.sh", changes=None):
            write_text(names_file, "".join(f"crates/demo/tests/setting.rs::{name}\n" for name in green))
            write_text(red_file, "".join(f"crates/demo/tests/setting.rs::{name}\n" for name in red))
            return admission("verdict-record", root, kind, plan_file, names_file, "--how", "cargo test --release -p demo（自证）", "--red-names", red_file,
                             changes=changes)

        # 记红、撤绿、下趟现判：三条判绿入库 ⇒ 同一把键 test_t 判红（--red-names）⇒ 下一趟 test_t 现判、原因「上次判红」，别的复用 ⇒ 再判绿 ⇒ 下一趟复用
        plan()
        record(["test_t", "test_u", "test_v"])
        _exit_code, record_output, _messages = record([], ["test_t"])
        _exit_code, decisions, reasons, output = plan()
        selftest.expect("判定库：同一把键判绿入库、后来判红（--red-names）⇒ 下一趟这一条现判、原因以「上次判红」起头",
                        decisions == {"test_t": "judge", "test_u": "reuse", "test_v": "reuse"} and reasons.get("test_t", "").startswith(RED_VERDICT_REASON)
                        and " red=1 " in record_output,
                        f"记红那一趟「{record_output.strip()[-200:]}」，下一趟 {decisions}，test_t 的原因「{reasons.get('test_t')}」")
        record(["test_t"])
        _exit_code, decisions, _reasons, output = plan()
        selftest.expect("判定库：红之后再判绿 ⇒ 下一趟复用", decisions == {"test_t": "reuse", "test_u": "reuse", "test_v": "reuse"}, f"{decisions}")
        # 记红没成（文件头「按条复用失灵」）：样本标记 red-record-failure 下 test_u 判红 ⇒ verdict-record 退 1、打「判红没记进判定库」；plan-failure 带 --red
        #   撤掉 test_u 原有的绿、判红 ⇒ 下一趟 test_u 现判，别的复用。弄坏开关 red-record-failure-swallowed 下这一格转红（只打 !、绿留着、下一趟复用）
        all_reused = {"test_t": "reuse", "test_u": "reuse", "test_v": "reuse"}
        only_u_judged = {"test_t": "reuse", "test_u": "judge", "test_v": "reuse"}
        marker = os.path.join(work, RED_RECORD_FAILURE_SAMPLE_MARKER)
        os.makedirs(os.path.dirname(marker), exist_ok=True)
        write_text(marker, "样本：记红没成\n")
        record_exit, record_output, record_messages = record([], ["test_u"])
        os.unlink(marker)
        record_output_path = os.path.join(work, "red-record-failure.out")
        write_text(record_output_path, record_output + record_messages)
        failure_exit, failure_output, failure_messages = admission("plan-failure", "样本调用方的", "verdict-record", str(record_exit), "--red", work, "demo-gate.sh",
                                                                   plan_file, red_file, record_output_path)
        _exit_code, decisions, _reasons, output = plan()
        selftest.expect("判定库：记红没成（样本标记 red-record-failure）⇒ verdict-record 退 1、plan-failure --red 撤掉原有的绿、下一趟这一条现判",
                        record_exit == 1 and RED_RECORD_FAILURE_HEAD in record_output and " red-unrecorded=1 " in record_output
                        and failure_exit == 1 and "同一把键原有的绿已撤掉 1 条" in failure_output and decisions == only_u_judged,
                        f"verdict-record 退 {record_exit}「{(record_output + record_messages).strip()[-200:]}」；"
                        f"plan-failure 退 {failure_exit}「{(failure_output + failure_messages).strip()[-300:]}」；下一趟 {decisions}")
        # verdict-record 没跑完（被停：退 137、一个字没打）：带 --red 的 plan-failure 照样撤绿、判红 ⇒ 下一趟 test_u 现判；格末把 test_u 判绿记回去（后面几格照三条都绿往下）
        record(["test_u"])
        _exit_code, before, _reasons, output = plan()
        write_text(red_file, "crates/demo/tests/setting.rs::test_u\n")
        killed_output_path = os.path.join(work, "killed-record.out")
        write_text(killed_output_path, "")
        failure_exit, failure_output, failure_messages = admission("plan-failure", "样本调用方的", "verdict-record", "137", "--red", work, "demo-gate.sh",
                                                                   plan_file, red_file, killed_output_path)
        _exit_code, decisions, _reasons, output = plan()
        selftest.expect("判定库：verdict-record 没跑完（退 137、没有 RECORD 那一行）⇒ plan-failure --red 照样撤绿、判红、下一趟这一条现判",
                        before == all_reused and failure_exit == 1 and "没跑完" in failure_output and "同一把键原有的绿已撤掉 1 条" in failure_output
                        and decisions == only_u_judged,
                        f"撤之前 {before}；plan-failure 退 {failure_exit}「{(failure_output + failure_messages).strip()[-300:]}」；下一趟 {decisions}")
        record(["test_u"])
        # 欠撤（文件头「按条复用失灵」）：样本标记 red-record-unavailable 下 test_u 判红 ⇒ verdict-record 退 75；plan-failure --red 照旧退 75、记一笔欠撤（键与种类，
        #   一把键一份，放在 SINGLEFS_LAST_RUNS_DIRECTORY 底下）；下一趟 verdict-plan 开跑先撤 test_u 原有的绿、回读、删掉欠撤 ⇒ test_u 现判，别的复用。
        #   再记一笔欠撤，样本标记 owed-withdrawal-unavailable 下下一趟撤不掉 ⇒ verdict-plan 整道退 75、不打 PLAN、欠撤留着；标记撤掉之后撤成、test_u 现判。
        #   弄坏开关 withdrawal-owed-unrecorded 下两格都转红（不记欠撤、不读欠撤：下一趟照复用 test_u 原有的绿）。每格末把 test_u 判绿记回去
        owed_root = os.path.join(environment["SINGLEFS_LAST_RUNS_DIRECTORY"], OWED_WITHDRAWALS_DIRECTORY)

        def owed_files():
            return sorted(os.path.join(directory, name) for directory, _subdirectories, names in os.walk(owed_root) for name in names if name.endswith(".json"))

        def red_record_unavailable():
            """test_u 判红、记红那一帧服务不可用 ⇒ (verdict-record 的退出码与输出, plan-failure 的退出码与输出)。"""
            marker_path = os.path.join(work, RED_RECORD_UNAVAILABLE_SAMPLE_MARKER)
            write_text(marker_path, "样本：记红那一帧服务不可用\n")
            unavailable_exit, unavailable_output, unavailable_messages = record([], ["test_u"])
            os.unlink(marker_path)
            output_path = os.path.join(work, "red-record-unavailable.out")
            write_text(output_path, unavailable_output + unavailable_messages)
            owed_exit, owed_output, owed_messages = admission("plan-failure", "样本调用方的", "verdict-record", str(unavailable_exit), "--red", work, "demo-gate.sh",
                                                              plan_file, red_file, output_path)
            return unavailable_exit, unavailable_output + unavailable_messages, owed_exit, owed_output + owed_messages

        _exit_code, before, _reasons, output = plan()
        record_exit, record_text, failure_exit, failure_text = red_record_unavailable()
        owed_after_failure = owed_files()
        plan_exit, decisions, _reasons, output = plan()
        selftest.expect("判定库：记红时 prune-server 不可用（样本标记 red-record-unavailable）⇒ plan-failure --red 照旧退 75、记一笔欠撤，下一趟开跑先撤同一把键的绿、回读，这一条现判",
                        before == all_reused and record_exit == PLAN_SERVICE_UNAVAILABLE_EXIT and failure_exit == PLAN_SERVICE_UNAVAILABLE_EXIT
                        and len(owed_after_failure) == 1 and "欠撤（键与种类）记在" in failure_text and plan_exit == 0 and "撤掉原有的绿 1 条" in output
                        and decisions == only_u_judged and not owed_files(),
                        f"撤之前 {before}；verdict-record 退 {record_exit}「{record_text.strip()[-160:]}」；plan-failure 退 {failure_exit}「{failure_text.strip()[-260:]}」；"
                        f"欠撤 {len(owed_after_failure)} 份；下一趟退 {plan_exit}、{decisions}、欠撤剩 {len(owed_files())} 份「{output.strip()[-200:]}」")
        record(["test_u"])
        plan()
        red_record_unavailable()
        owed_after_failure = owed_files()
        withdrawal_marker = os.path.join(work, OWED_WITHDRAWAL_UNAVAILABLE_SAMPLE_MARKER)
        write_text(withdrawal_marker, "样本：撤欠撤那一帧服务不可用\n")
        blocked_exit, blocked, _reasons, blocked_output = plan()
        owed_while_blocked = owed_files()
        os.unlink(withdrawal_marker)
        plan_exit, decisions, _reasons, output = plan()
        selftest.expect("判定库：欠撤撤不掉（样本标记 owed-withdrawal-unavailable）⇒ verdict-plan 整道退 75、不打 PLAN、欠撤留着；撤得掉之后这一条现判",
                        len(owed_after_failure) == 1 and blocked_exit == PLAN_SERVICE_UNAVAILABLE_EXIT and not blocked and "PLAN " not in blocked_output
                        and "撤不掉" in blocked_output and owed_while_blocked == owed_after_failure and plan_exit == 0 and decisions == only_u_judged and not owed_files(),
                        f"欠撤 {len(owed_after_failure)} 份；撤不掉那一趟退 {blocked_exit}、判了 {blocked}、欠撤 {len(owed_while_blocked)} 份「{blocked_output.strip()[-200:]}」；"
                        f"标记撤掉之后退 {plan_exit}、{decisions}、欠撤剩 {len(owed_files())} 份")
        record(["test_u"])
        # 不是 git 工作树的副本（照样本拷一份、不带 .git）里判：verdict-plan 与 verdict-record 都退 2、点名那棵树，一条都不判、不打 PLAN 与 RECORD
        copy = os.path.join(copy_parent, "copy-without-git")   # 放在样本仓外面：放里面 git 照样找得到样本仓的 common-dir
        shutil.copytree(work, copy, ignore=shutil.ignore_patterns(".git"))
        plan_exit, copied, _reasons, copy_output = plan(copy, "copy-gate.sh")
        record_exit, record_output, record_messages = record(["test_t"], root=copy, kind="copy-gate.sh")
        selftest.expect("判定库：不是 git 工作树的副本里判，verdict-plan 与 verdict-record 都退 2、点名那棵树，一条都不判",
                        plan_exit == EXIT_REGISTRATION_ERROR and not copied and "不是 git 工作树" in copy_output and copy in copy_output
                        and "PLAN " not in copy_output and record_exit == EXIT_REGISTRATION_ERROR and "RECORD " not in record_output,
                        f"verdict-plan 退 {plan_exit}（要 {EXIT_REGISTRATION_ERROR}）、判了 {copied}「{copy_output.strip()[-200:]}」；"
                        f"verdict-record 退 {record_exit}「{(record_output + record_messages).strip()[-160:]}」")
        # 改判定域的环境变量进键：一条测试的闭包读按段登记文件与片长两个变量，另一条不读；请求里的 variable: 照 test-functions-request 的取法从环境给
        edit_once(os.path.join(work, "crates/demo/src/lib.rs"), "pub fn unrelated() -> u32 {",
                  'pub fn registered_domain() -> usize {\n    std::env::var("SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY").map_or(0, |path| path.len())\n'
                  '        + std::env::var("SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES").map_or(0, |states| states.len())\n}\n\npub fn unrelated() -> u32 {')
        write_text(os.path.join(work, "crates/demo/tests/domain.rs"),
                   "#[test]\nfn reads_the_registry() {\n    assert!(demo::registered_domain() < 1000);\n}\n\n#[test]\nfn stands_alone() {\n    assert_eq!(1, 1);\n}\n")
        registry = os.path.join(work, ".claude/gate.d/crash-point-replay-segments.tsv")
        write_text(registry, "alpha\talpha_flow\t0\t7\t" + "0" * 64 + "\n")

        def domain_keys(variables):
            tokens = UNIT_SAMPLE_TEST_TOKENS + domain_variable_tokens(variables)
            requests = [Request(f"crates/demo/tests/domain.rs::{name}", f"crates/demo/tests/domain.rs::{name}", tokens) for name in ("reads_the_registry", "stands_alone")]
            return {short_name(name): key for name, key in sample_keys(work, environment, requests).items()}

        light = {"SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY": registry, "SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES": "65536"}
        base = domain_keys(light)
        write_text(registry, "alpha\talpha_flow\t0\t7\t" + "1" * 64 + "\n")
        registry_changed = domain_keys(light)
        sliced = domain_keys(dict(light, SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES="131072"))
        unset = domain_keys({})
        reading = [base["reads_the_registry"], registry_changed["reads_the_registry"], sliced["reads_the_registry"], unset["reads_the_registry"]]
        standing = {base["stands_alone"], registry_changed["stands_alone"], sliced["stands_alone"], unset["stands_alone"]}
        selftest.expect("单元键：闭包读按段登记文件与片长两个变量的那条，登记文件的内容变、片长变、设与没设，键都变；闭包不读的那条键不变",
                        all(reading) and len(set(reading)) == 4 and len(standing) == 1 and None not in standing,
                        f"读的那条四种取值下的键 {[key[:12] if key else key for key in reading]}，不读的那条 {[key[:12] if key else key for key in standing]}")
        _exit_code, request_output, _messages = admission("test-functions-request", "demo-gate.sh", work, changes={"SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES": "65536",
                                                                                                          "SINGLEFS_CRASH_INJECTION_THREADS": "4"})
        selftest.expect("test-functions-request：起它的环境里设了的改判定域的变量打成 --token variable:<名>=<值>，以 THREADS 收尾的不打",
                        "argument\tvariable:SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=65536" in request_output and "THREADS" not in request_output,
                        request_output.strip())
        # 同短名的测试函数各算各的键：同一目标里顶层 twin 与 mod inner 里的 twin（inner 经 use super::* 看得见顶层那一条，定义那一处当引用解就连上它），
        # 改 inner::twin 只它的键变，改顶层 twin 也只它的键变
        twins = os.path.join(work, "crates/demo/tests/twins.rs")
        write_text(twins, "#[test]\nfn twin() {\n    assert_eq!(demo::helper(), 89);\n}\n\nmod inner {\n    use super::*;\n\n    #[test]\n    fn twin() {\n"
                          "        assert_eq!(demo::helper() + 1, 90);\n    }\n}\n")
        twin_requests = [Request(f"crates/demo/tests/twins.rs::{name}", f"crates/demo/tests/twins.rs::{name}", UNIT_SAMPLE_TEST_TOKENS) for name in ("twin", "inner::twin")]
        before = sample_keys(work, environment, twin_requests)
        edit_once(twins, "demo::helper() + 1, 90", "demo::helper() + 2, 91")
        after = sample_keys(work, environment, twin_requests)
        edit_once(twins, "assert_eq!(demo::helper(), 89)", "assert_eq!(demo::helper() - 1, 88)")
        after_top = sample_keys(work, environment, twin_requests)
        top, nested = "crates/demo/tests/twins.rs::twin", "crates/demo/tests/twins.rs::inner::twin"
        selftest.expect("单元键：同一目标里两条短名相同、不在同一模块的测试函数键不同，改其中一条只它的键变",
                        all(before.values()) and before[top] != before[nested] and after[top] == before[top] and after[nested] != before[nested]
                        and after_top[top] != after[top] and after_top[nested] == after[nested],
                        f"改前 {before}，改 inner::twin 之后 {after}，再改顶层 twin 之后 {after_top}")
    except AssertionError as error:
        selftest.expect("记红撤绿、库位置、环境变量进键、同短名那几格的样本改得动", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)
        shutil.rmtree(copy_parent, ignore_errors=True)

def run_reuse_three_question_cells(selftest, module):
    """verdict-plan / verdict-record 的复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」，格名 reuse-three-questions）与复用哨兵（「二」，格名 reuse-sentinel）。
    每一串各用一份自己的临时小仓（单元键那几格的样本）、自己的判定库（种类 reuse-questions-gate.sh，在那份临时仓的 git common-dir 里），哨兵的条数与种子在这里钉死、
    上次红的清单（SINGLEFS_LAST_RUNS_DIRECTORY）指到临时仓里，不读写共用的位置：
      四趟：库是空的（三条都现判，记绿）、同一份输入（①）、设点文件里加一个没人调的函数（闭包外，②）、改 helper（只在 test_t 的闭包里，③ affected test_t）；
      这四趟哨兵数设 1000（键相同的全当哨兵、不抽强制现判），规范输出照 report_three_questions 打。
      哨兵：哨兵 1 条、种子 7：一条当哨兵复用、一条强制现判（verdict-record 判绿之后报与库里同为绿），整趟退 0；
      开着弄坏开关 verdict-lookup-skipped（不读库）再跑同一个种子，退 1、红「复用失灵」（共用库的 REUSE_CHECKS_BREAK=rejudged-sentinel-passes 下这一格判错）。
      另一份临时小仓开着弄坏开关 verdict-key-with-entry-file（键另加入口所在的整份文件）再走那四趟：② 必须红（这一串的规范输出不打）。"""
    checks = reuse_checks_module()
    if checks is None:
        selftest.expect(f"reuse-three-questions：同目录有 {REUSE_CHECKS_LIBRARY}", False, "找不到复用三问的判法，这一格判不了")
        return
    kind = "reuse-questions-gate.sh"
    names = [request.name for request in unit_sample_requests()]

    class Questions:
        """一份临时小仓上的 verdict-plan / verdict-record。"""

        def __init__(self, work, changes):
            self.work = work
            self.environment = build_unit_sample_repository(work)
            self.environment.pop(checks.SENTINEL_SEED_VARIABLE, None)
            self.environment.update({checks.SENTINEL_COUNT_VARIABLE: "1000", checks.LAST_RUNS_VARIABLE: os.path.join(work, ".last-runs")}, **changes)
            self.requests_file, self.plan_file, self.names_file = (os.path.join(work, f"questions.{suffix}") for suffix in ("requests", "plan", "names"))
            write_text(self.requests_file, "".join(request.text() + "\n" for request in unit_sample_requests()))

        def admission(self, *arguments, changes=None):
            return run_in_environment([sys.executable, module, *arguments], dict(self.environment, **(changes or {})))

        def plan(self, changes=None):
            """一趟 verdict-plan：交 (退出码, {项: 复用 | 现判 | 强制现判 | 拒算}, 原样输出)。"""
            exit_code, output, messages = self.admission("verdict-plan", self.work, kind, self.requests_file, self.plan_file, changes=changes)
            states = {}
            for line in output.split("\n"):
                columns = line.split("\t")
                if len(columns) >= 4 and columns[0] in names:
                    decision, reason = columns[1], columns[3]
                    states[short_name(columns[0])] = ("复用" if decision == "reuse" else "拒算" if decision == "refused"
                                                      else "强制现判" if reason.startswith("强制现判") else "现判")
            return exit_code, states, output + messages

        def record(self, green, changes=None):
            write_text(self.names_file, "".join(f"crates/demo/tests/setting.rs::{name}\n" for name in green))
            return self.admission("verdict-record", self.work, kind, self.plan_file, self.names_file, "--how", "cargo test --release -p demo（自证）", changes=changes)

        def four_runs(self):
            """头一趟（记绿）、同一份输入、闭包外改动、闭包内改 helper：交 (四趟的结局, 末趟输出)。"""
            _exit, first, _output = self.plan()
            self.record(sorted(first))
            _exit, second, _output = self.plan()
            edit_once(os.path.join(self.work, "crates/demo/tests/setting.rs"), "#[test]\nfn test_u()",
                      "fn spare_unused() -> u32 {\n    9\n}\n\n#[test]\nfn test_u()")
            _exit, unrelated, _output = self.plan()
            edit_once(os.path.join(self.work, "crates/demo/src/lib.rs"), "WIDTH + 1", "WIDTH + 2")
            _exit, unit_change, unit_output = self.plan()
            return (first, second, unrelated, unit_change), unit_output

    works = [tempfile.mkdtemp(prefix="admission-selftest-reuse-questions-") for _ in range(2)]
    try:
        questions = Questions(works[0], {})
        runs, unit_output = questions.four_runs()
        lines = []
        passed, problems = checks.report_three_questions("scripts/gate/admission.py", *runs, ["test_t"], printer=lines.append)
        for line in lines:
            print(line)
        selftest.expect("reuse-three-questions：verdict-plan 头一趟三条现判、同一份输入第二趟全复用、闭包外加一个没人调的函数全复用、改 helper 只现判 test_t",
                        passed, f"{'；'.join(problems)}；四趟：{' / '.join(map(str, runs))}；末趟输出：{unit_output.strip()[-300:]}")

        # 复用哨兵：库里三条都记绿（键是此刻的），哨兵 1 条、种子 7
        questions.record(["test_t"])
        sentinel_settings = {checks.SENTINEL_COUNT_VARIABLE: "1", checks.SENTINEL_SEED_VARIABLE: "7"}
        green_exit, states, green_output = questions.plan(sentinel_settings)
        forced = [name for name, state in states.items() if state == "强制现判"]
        record_exit, record_output, _messages = questions.record(sorted(states), sentinel_settings)
        lookup_exit, _states, lookup_output = questions.plan(dict(sentinel_settings, **{BREAK_VARIABLE: "verdict-lookup-skipped"}))
        selftest.expect("reuse-sentinel：哨兵 1 条、种子 7 那一趟退 0、种子打进输出、一条强制现判（verdict-record 报与库里同为绿）；不读库那一趟退 1、红「复用失灵」",
                        green_exit == 0 and "种子 7（SINGLEFS_REUSE_SENTINEL_SEED=7 复现）" in green_output and len(forced) == 1
                        and record_exit == 0 and f"强制现判 crates/demo/tests/setting.rs::{forced[0] if forced else '?'} 与库里同为「绿」" in record_output
                        and lookup_exit == 1 and "复用失灵：" in lookup_output,
                        f"照常那一趟退 {green_exit}、结局 {states}；verdict-record 退 {record_exit}：{record_output.strip()[-300:]}；"
                        f"不读库那一趟退 {lookup_exit}：{lookup_output.strip()[-400:]}；照常那一趟输出：{green_output.strip()[-400:]}")

        # 弄坏开关 verdict-key-with-entry-file：另一份临时小仓，键另加入口所在的整份文件，闭包外加函数那一趟必须现判（② 红）
        broken_runs, _output = Questions(works[1], {BREAK_VARIABLE: "verdict-key-with-entry-file"}).four_runs()
        broken_passed, _line, broken_problems = checks.three_questions("scripts/gate/admission.py", *broken_runs, ["test_t"])
        selftest.expect("弄坏开关 verdict-key-with-entry-file 下 reuse-three-questions 的 ② 转红（闭包外的改动也让整份入口文件的三条作废）",
                        not broken_passed and any(problem.startswith("② ") for problem in broken_problems),
                        f"三问{'过了' if broken_passed else '红在：' + '；'.join(broken_problems)}；四趟：{' / '.join(map(str, broken_runs))}")
    except AssertionError as error:
        selftest.expect("reuse-three-questions：样本改得动", False, str(error))
    finally:
        for work in works:
            shutil.rmtree(work, ignore_errors=True)

RESEARCH_SAMPLE_FILES = {
    "scripts/experiment-apparatus/Cargo.toml": ('[package]\nname = "e7-index-bench"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\naes-gcm = "0.11.1"\n\n'
                                       '[lib]\npath = "lib.rs"\n\n[[bin]]\nname = "e23_journal_geom"\npath = "e23_journal_geom.rs"\n\n'
                                       '[[bin]]\nname = "e140-header-alignment"\npath = "e140_header_alignment.rs"\n\n[[bin]]\nname = "e900"\npath = "e900.rs"\n'),
    "scripts/experiment-apparatus/Cargo.lock": ('version = 4\n\n[[package]]\nname = "aes-gcm"\nversion = "0.11.1"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n'
                                       'checksum = "cccc"\n\n[[package]]\nname = "e7-index-bench"\nversion = "0.1.0"\ndependencies = [\n "aes-gcm",\n]\n'),
    "scripts/experiment-apparatus/lib.rs": ("pub struct Emitter;\n\nimpl Emitter {\n    pub fn emit(&self) -> u32 {\n        1\n    }\n}\n\n"
                                   "pub fn lru_new() -> u32 {\n    assert!(true, \"容量\");\n    5\n}\n"),
    "scripts/experiment-apparatus/e23_journal_geom.rs": ("use e7_index_bench::Emitter;\n\nfn simulate_journal_run() -> u64 {\n    let start = 0;\n    start\n}\n\n"
                                                "fn main() {\n    let _ = Emitter.emit();\n    let _ = simulate_journal_run();\n}\n"),
    "scripts/experiment-apparatus/e140_header_alignment.rs": "use aes_gcm::Aes256Gcm;\n\nfn main() {\n    let _ = std::mem::size_of::<Aes256Gcm>();\n}\n",
    "scripts/experiment-apparatus/e900.rs": "fn main() {\n    let _ = 1;\n}\n",
    RESULTS_DIRECTORY + "/e23-2026-09-01.out": "E7RESULT name=e23 rows=1\nE7RESULT name=done emitted=2\n",
    RESULTS_DIRECTORY + "/e140-2026-09-01.out": "E7RESULT name=e140 rows=1\nE7RESULT name=done emitted=2\n",
    RESULTS_DIRECTORY + "/e900-2026-09-01.out": "E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n",
    RESULTS_DIRECTORY + "/e901-2026-09-01.out": "E7RESULT name=e901 rows=1\nE7RESULT name=done emitted=2\n",
    ".claude/kb/decisions/08-sample.md": "# 样本决策\n\n一句。\n",
    REGISTRATION_TABLE: ("# 样本登记表\nE900\tscripts/experiment-apparatus/e900.rs::main\t# 样本实验\n"
                         "E900/second\t@E900\tquestion-row=research/questions.md#6:够判[：:][^（(|]*对照(本身)?是好的 product-field=E900:verdict:control_ok=true\t# 样本第二段\n"
                         "E900/third\t@E900\tsame-run-field=verdict:control_ok=true\t# 样本第二段的同一趟现判写法：前一段这一趟的输出经环境变量交来\n"),
}
RESEARCH_SAMPLE_REPLAY = """#!/usr/bin/env bash
# 自证的复跑表（admission.py --selftest 的样本：只有判一行的几个函数与两个驱动，不跑东西）
TABLE=$(cat <<'TSV'
E23|e23_journal_geom||e23-2026-09-01.out|exact
E140|e140-header-alignment||e140-2026-09-01.out|exact
E900|@driver_e900||e900-2026-09-01.out|exact
E901|@driver_gpu||e901-2026-09-01.out|exact
TSV
)
ADMISSION_ROW_KEYS=$(cat <<'TSV'
TSV
)
replay_judge_fresh_product() {
  diff -q "$1" "$2" >/dev/null
}
strip_admission_header() { sed '/^E7INPUT /d'; }
strip_timing() { sed 's/ elapsed=[0-9.]*//'; }
strip_load_fields() { sed 's/ loadavg1=[0-9.]*//'; }
timing_load_reading_raw() { echo 0; }
timing_load_decide() { echo judged; }
driver_e900() {
  printf 'E7RESULT name=verdict control_ok=true\\nE7RESULT name=done emitted=2\\n'
}
driver_gpu() {
  echo "${CHECKER_TIER_FEATURES[@]}"
  ./scripts/experiment-apparatus/target/release/e900
}
"""

def build_research_sample_repository(work):
    run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
    for path, text in RESEARCH_SAMPLE_FILES.items():
        write_text(os.path.join(work, path), text)
    write_text(os.path.join(work, REPLAY_SCRIPT), RESEARCH_SAMPLE_REPLAY)
    write_questions(work, "开着")
    tools = os.path.join(work, ".tools")
    for name, script in fake_toolchain_scripts(os.path.join(work, FAKE_RUSTC_VERSION_FILE)).items():
        write_executable(os.path.join(tools, name), script)
    os.makedirs(os.path.join(work, "scripts", "gate"), exist_ok=True)
    shutil.copy(ENTRY_SCRIPT, os.path.join(work, "scripts/gate/admission.py"))
    copy_admission_modules(work)
    copy_function_units_library(work)
    link_sop_copy(work)
    write_text(os.path.join(work, ".gitignore"), ".tools/\n.fake-rustc-version\n.claude/singlefs-ai-sop\n.claude/scripts\n")
    run_quietly(["git", "-C", work, "add", "-A"], GIT_IDENTITY)
    run_quietly(["git", "-C", work, "commit", "-qm", "base"], GIT_IDENTITY)
    return environment_without_build_settings({"PATH": tools + os.pathsep + os.environ.get("PATH", ""), "CARGO_HOME": os.path.join(work, ".tools", "cargo-home")})


def one_line(text):
    """自证一格判错的细节里转述了子进程的输出（客户端的「✗ …」「→ 怎么办：…」两行这类）：换行换成「 | 」，整条留在格的那一行里，
    转述里以 ✗ 起头的那几行不另起一行、不被当成别的格（门禁 code-tooling 的 break-switch-replay 格按以 ✗ 起头的行认格；写法同 scripts/gate/verus-proof-check.py 的 one_line）。"""
    rows = str(text).split("\n")
    return " | ".join([rows[0].rstrip(), *(row.strip() for row in rows[1:] if row.strip())])


class Selftest:
    def __init__(self):
        self.failures = 0
        self.checked = 0
        self.names = []
        self.cell = ""   # 正在跑的格（run_selftest 的 cells 的键），失败行点名它（文件头「选格」）

    def expect(self, name, condition, detail):
        self.checked += 1
        self.names.append(name)
        if condition:
            print(f"  ✓ {name}")
        else:
            print(f"  ✗ 自证：{self.cell}：{name}：{one_line(detail)}")  # gate-lint:detail
            self.failures += 1


def run_quietly(command, environment_changes=None, working_directory=None, keep_multi_host_configuration=False):
    """外面设的 ADMISSION_BREAK 照样传给子进程：`ADMISSION_BREAK=skip-unchanged admission.py --selftest` 要看到「输入没变」那几格红。
    本台的多机配置（task-server 把 SINGLEFS_MULTI_HOST_CONFIG 设成派到的那一台树里的 multi-host.env）与起活那一方清场的握手变量不传：
    临时仓的格要配置的经 environment_changes 显式给。keep_multi_host_configuration 只给弄坏开关 multi-host-config-inherited 那一格（不清多机配置变量）。
    继承的 GIT_*（GIT_DIR、GIT_WORK_TREE、GIT_INDEX_FILE 这类）一律不传：临时仓里起的 git 只认 -C 与工作目录给的那个仓；作者身份这类经 environment_changes 显式给（GIT_IDENTITY）。"""
    environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
    cleared = ["SINGLEFS_STAGED_TREE", "GPU_QUIESCE_ORIGIN_READ_BACK"]
    if not keep_multi_host_configuration:
        cleared.append(MULTI_HOST_CONFIGURATION_VARIABLE)
    for variable in cleared:
        environment.pop(variable, None)
    environment.update(environment_changes or {})
    completed = subprocess.run(command, capture_output=True, text=True, errors="replace", env=environment, cwd=working_directory)
    return completed.returncode, completed.stdout, completed.stderr


def write_text(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def write_questions(work, row_six_status):
    write_text(os.path.join(work, "research/questions.md"),
               "| # | 问题 | 状态 |\n|---|---|---|\n"
               "| 5 | 别的 | **够判：能** |\n"
               f"| 6 | 阳性对照是好的还是坏的 | {row_six_status} |\n")


def point_replay_row_at(work, product_name):
    write_text(os.path.join(work, REPLAY_SCRIPT), f"TABLE=$(cat <<'TSV'\nE900|@driver_e900||{product_name}|exact\nTSV\n)\n")


GIT_IDENTITY = {"GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@t", "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@t"}


def write_executable(path, text):
    write_text(path, text)
    os.chmod(path, 0o755)


def staged_tree_of(work):
    run_quietly(["git", "-C", work, "add", "-A"], GIT_IDENTITY)
    _exit_code, output, _messages = run_quietly(["git", "-C", work, "write-tree"])
    return output.strip()


def mark_tree_green(work, tree):
    """照 scripts/gate/gate-staged.sh 的做法：refs/sop/staged-green 指一个包着那棵暂存树的提交。"""
    _exit_code, commit, _messages = run_quietly(["git", "-C", work, "commit-tree", tree, "-m", "green"], GIT_IDENTITY)
    run_quietly(["git", "-C", work, "update-ref", GATE_GREEN_TREE_RECORD, commit.strip()])


def environment_without_build_settings(changes=None):
    """子进程的环境：去掉会进构建环境那一格的变量（CARGO_HOME、RUSTC 与名字对得上的那几类）与握手用的几个，再套上 changes。"""
    environment = {name: value for name, value in os.environ.items()
                   if not (name in BUILD_ENVIRONMENT_EXACT_VARIABLES or name in ("RUSTC", "CARGO_HOME")
                           or any(form.match(name) for form in BUILD_ENVIRONMENT_VARIABLE_FORMS))}
    for variable in ("SINGLEFS_STAGED_TREE",):
        environment.pop(variable, None)   # 调用方的强制全跑与暂存树标记不带进自证的子进程
    environment.update(changes or {})
    return environment


def run_in_environment(command, environment, working_directory=None):
    completed = subprocess.run(command, capture_output=True, text=True, errors="replace", env=environment, cwd=working_directory)
    return completed.returncode, completed.stdout, completed.stderr


class BreakSwitch:
    """在本进程里临时设 ADMISSION_BREAK（给纯函数那几格证明会红），退出时照原样还回去。"""
    def __init__(self, switch_name):
        self.switch_name = switch_name
        self.saved = None

    def __enter__(self):
        self.saved = os.environ.get(BREAK_VARIABLE)
        os.environ[BREAK_VARIABLE] = self.switch_name
        return self

    def __exit__(self, *_exception):
        if self.saved is None:
            os.environ.pop(BREAK_VARIABLE, None)
        else:
            os.environ[BREAK_VARIABLE] = self.saved
        return False


FAKE_TOOLCHAIN_SCRIPTS = fake_toolchain_scripts("/nonexistent/fake-rustc-version")



def build_crash_case_repository(work, registration_rows):
    """一个临时小仓：包 pkg 里有用例 own_case（mod common、mod helper_target）、被注释提到的 other_target、被字符串点名的 string_named。"""
    run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
    write_text(os.path.join(work, "Cargo.toml"), '[workspace]\nmembers = ["crates/pkg"]\n')
    write_text(os.path.join(work, "crates/pkg/Cargo.toml"), '[package]\nname = "pkg"\nversion = "0.0.0"\n')
    write_text(os.path.join(work, "crates/pkg/src/lib.rs"),
               '//! 注释里提到 other_target 不算\npub const PATH: &str = "tests/string_named.rs";\npub fn one() -> u32 { 1 }\n')
    write_text(os.path.join(work, "crates/pkg/tests/own_case.rs"),
               'mod common;\nmod helper_target;\n#[test]\nfn the_case() {}\n')
    write_text(os.path.join(work, "crates/pkg/tests/common/mod.rs"), "pub fn shared() {}\n")
    write_text(os.path.join(work, "crates/pkg/tests/helper_target.rs"), "pub fn helper() {}\n")
    write_text(os.path.join(work, "crates/pkg/tests/other_target.rs"), "#[test]\nfn other() {}\n")
    write_text(os.path.join(work, "crates/pkg/tests/string_named.rs"), "#[test]\nfn named() {}\n")
    write_text(os.path.join(work, REGISTRATION_TABLE), "".join(row + "\n" for row in registration_rows))


ADMISSION_MODULE_FILES = ("admission_base.py", "admission_unit_key.py", "admission_crash_case.py", "admission_commands.py")   # 入口模块级 import 的块；自证块只在 --selftest 里才 import，样本仓里不跑它
ENTRY_SCRIPT = os.path.join(SELFTEST_HERE, ENTRY_SCRIPT_NAME)


def copy_admission_modules(work):
    """样本仓里的 scripts/gate/admission.py 要 import 同目录的块：入口拷过去之后，把这几块一并拷到同一个目录。"""
    for file_name in ADMISSION_MODULE_FILES:
        shutil.copy(os.path.join(SELFTEST_HERE, file_name), os.path.join(work, "scripts/gate", file_name))


FUNCTION_UNITS_MODULE_PREFIX = FUNCTION_UNITS_LIBRARY[:-len(".py")]   # lib_function_units：入口与它拆出的各份（含自证各份）的文件名起头；弄坏开关 function-units-library-by-prefix 只按它列


def function_units_library_names(by_prefix=False):
    """样本仓里要有的算键库文件名：lib_function_units.py 连同它在同目录里 import 的各份，递归收全（lib_same_directory_imports.py：被 import 的不一定以 lib_function_units 起头，
    例 lib_function_units_python_module.py 在模块级 import lib_lag_split_family）。by_prefix 只给弄坏开关 function-units-library-by-prefix 那一格用：只列以 lib_function_units 起头的各份。"""
    if by_prefix:
        return sorted(name for name in os.listdir(SELFTEST_HERE) if name.startswith(FUNCTION_UNITS_MODULE_PREFIX) and name.endswith(".py"))
    return same_directory_import_closure(SELFTEST_HERE, FUNCTION_UNITS_LIBRARY)


def function_units_runtime_data_names(nonliteral_skipped=False):
    """样本仓里要有的、算键库运行期读的数据文件（从仓根起的路径）：function_units_library_names 列的各份模块在模块级写的 RUNTIME_DATA_FILES 的并集
    （lib_same_directory_imports.py 的 runtime_data_files，ast 读、不 import；例 lib_function_units_registry.py 从 LIBRARY_ROOT 读测试服务代码登记表）。
    nonliteral_skipped 只给弄坏开关 runtime-data-nonliteral-skipped 那一格用：写得不对的那一份算零份、不抛。"""
    return runtime_data_files(SELFTEST_HERE, function_units_library_names(), nonliteral_skipped)


def copy_function_units_library(work, by_prefix=False, data_skipped=False):
    """样本仓里的 scripts/gate/lib_function_units.py 在模块级 import 同目录的各份、运行期又读仓根下的数据文件：入口拷过去之后，function_units_library_names 列的各份
    一并拷到同一个目录，function_units_runtime_data_names 列的数据文件拷到样本仓里同一个相对路径；只拷入口、漏一份模块时 admission.py 在样本仓里载不进它、每一条键都拒算，
    漏一份数据文件时载得进、读那一份时拒算（读不了测试服务代码登记表）。data_skipped 只给弄坏开关 function-units-runtime-data-not-copied 那一格用：只拷模块、不拷数据文件。"""
    for file_name in function_units_library_names(by_prefix):
        shutil.copy(os.path.join(SELFTEST_HERE, file_name), os.path.join(work, "scripts/gate", file_name))
    for relative in [] if data_skipped else function_units_runtime_data_names():
        os.makedirs(os.path.dirname(os.path.join(work, relative)), exist_ok=True)
        shutil.copy(os.path.join(SELFTEST_REPOSITORY, relative), os.path.join(work, relative))


RUNTIME_DATA_FIXTURE_MODULES = {
    "runtime_data_entry.py": "import runtime_data_declared\nimport runtime_data_other\nimport runtime_data_plain\n",
    "runtime_data_declared.py": 'RUNTIME_DATA_FILES = ("data/second.tsv", "data/first.tsv")\n',
    "runtime_data_other.py": 'RUNTIME_DATA_FILES: tuple = ("data/first.tsv", "other/third.txt")\n',
    "runtime_data_plain.py": "import os\n",
}
RUNTIME_DATA_FIXTURE_NONLITERAL = {
    "runtime_data_entry.py": "import runtime_data_declared\nimport runtime_data_computed\n",
    "runtime_data_declared.py": 'RUNTIME_DATA_FILES = ("data/first.tsv",)\n',
    "runtime_data_computed.py": 'TABLE = "second.tsv"\nRUNTIME_DATA_FILES = ("data/" + TABLE,)\n',
}


def runtime_data_fixture_union(directory, modules, nonliteral_skipped=False):
    """directory 里写 modules（文件名 → 源码），从 runtime_data_entry.py 起收同目录 import 闭包，交 (它们的 RUNTIME_DATA_FILES 并集, 抛的 RuntimeDataError 或 None)。"""
    os.makedirs(directory)
    for file_name, source in modules.items():
        write_text(os.path.join(directory, file_name), source)
    names = same_directory_import_closure(directory, "runtime_data_entry.py")
    try:
        return runtime_data_files(directory, names, nonliteral_skipped), None
    except RuntimeDataError as error:
        return None, error


def run_function_units_library_copy_cell(selftest):
    """样本仓拷算键库（copy_function_units_library）：拷出来的清单里有 lib_lag_split_family.py（lib_function_units_python_module.py 在模块级 import 它，名字不以 lib_function_units 起头），
    且拷出来的算键库在只有这份拷贝与链进来的 .claude/scripts 的临时仓里载得进（python3 -I，不带仓里别的路径）；
    算键库运行期读的数据文件（模块级 RUNTIME_DATA_FILES）拷到了样本仓里同一个相对路径、拷出来的算键库读得到测试服务代码登记表；
    夹具模块：闭包里每份模块的 RUNTIME_DATA_FILES 取并集排序、没写这个常量的算零份，写成非字面的字符串元组抛 RuntimeDataError、不悄悄跳过。
    弄坏开关 ADMISSION_BREAK=function-units-library-by-prefix：这一格里拷算键库只按前缀列，清单里没有 lib_lag_split_family.py、载入退 1（ModuleNotFoundError）。
    弄坏开关 ADMISSION_BREAK=function-units-runtime-data-not-copied：拷算键库只拷模块、不拷数据文件，样本仓里没有那几个文件、读测试服务代码登记表那一步拒算。
    弄坏开关 ADMISSION_BREAK=runtime-data-nonliteral-skipped：夹具里写成非字面量的那一份算零份、不抛。
    别的格里拷算键库照现算的清单拷（模块与数据文件），不读这三条开关。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-library-copy-")
    try:
        gate_directory = os.path.join(work, "scripts", "gate")
        os.makedirs(gate_directory)
        copy_function_units_library(work, by_prefix=break_is_set("function-units-library-by-prefix"), data_skipped=break_is_set("function-units-runtime-data-not-copied"))
        link_sop_copy(work)   # lib_function_units.py 载入时按相对路径取 .claude/scripts 里的 project_preflight
        copied = sorted(os.listdir(gate_directory))
        selftest.expect("样本仓拷算键库：清单里有它在模块级 import 的 lib_lag_split_family.py", "lib_lag_split_family.py" in copied,
                        f"拷出 {len(copied)} 份：{'、'.join(copied)}；清单按 lib_same_directory_imports.py 照算键库实际 import 的同目录模块现算，不按文件名前缀列")
        code, _output, errors = run_quietly([sys.executable, "-I", "-B", "-c", "import sys; sys.path.insert(0, sys.argv[1]); import lib_function_units", gate_directory])
        selftest.expect("样本仓拷算键库：拷出来的 lib_function_units.py 在只有这份拷贝的目录里载得进", code == 0,
                        f"退 {code}：{errors.strip().splitlines()[-1] if errors.strip() else '无 stderr'}；拷的清单少了它 import 的某一份")
        data_names = function_units_runtime_data_names()
        missing = [name for name in data_names if not os.path.isfile(os.path.join(work, name))]
        selftest.expect("样本仓拷算键库：算键库运行期读的数据文件拷到了样本仓里同一个相对路径", not missing,
                        f"清单 {data_names}，样本仓里没有 {missing}；看算键库模块级的 RUNTIME_DATA_FILES 与 copy_function_units_library")
        # 载不进的那一趟（上一格已红）不再重复判这一格：载得进了，才看拷出来的算键库读不读得到登记表
        rows_code, rows_output, rows_errors = (code, "", errors) if code != 0 else run_quietly(
            [sys.executable, "-I", "-B", "-c", "import sys; sys.path.insert(0, sys.argv[1]); import lib_function_units as library; print(len(library.test_service_code_rows()))", gate_directory])
        selftest.expect("样本仓拷算键库：拷出来的算键库读得到测试服务代码登记表", code != 0 or (rows_code == 0 and rows_output.strip().isdigit() and int(rows_output.strip()) > 0),
                        f"退 {rows_code}：{rows_errors.strip().splitlines()[-1] if rows_errors.strip() else rows_output.strip() or '无输出'}；样本仓里读不到它运行期读的数据文件")
        union, error = runtime_data_fixture_union(os.path.join(work, "runtime-data-union"), RUNTIME_DATA_FIXTURE_MODULES)
        selftest.expect("样本仓拷算键库：闭包里每份模块的 RUNTIME_DATA_FILES 取并集、排序，没写这个常量的算零份",
                        error is None and union == ["data/first.tsv", "data/second.tsv", "other/third.txt"], f"得 {union}、异常 {error}")
        union, error = runtime_data_fixture_union(os.path.join(work, "runtime-data-nonliteral"), RUNTIME_DATA_FIXTURE_NONLITERAL,
                                                  nonliteral_skipped=break_is_set("runtime-data-nonliteral-skipped"))
        selftest.expect("样本仓拷算键库：RUNTIME_DATA_FILES 写成非字面的字符串元组时抛 RuntimeDataError、不悄悄跳过", error is not None and union is None,
                        f"得 {union}、异常 {error}；拼出来的 \"data/\" + TABLE 那一份被当成零份")
    finally:
        shutil.rmtree(work, ignore_errors=True)
