"""一条命令是不是重型测试、属于哪一类。

heavy-test-guard.sh（执行前拒绝）与看门狗（research/scripts/agent-watch.py，在进程这一层）共用这一份判定，各自按文件路径用 importlib 导入；
同目录的 lib_shell_words.py（切词、剥前缀与包装）由这份自己导入，调用方从 `shell_words` 属性拿同一份。
谁能跑哪一类、要带什么前缀，是 heavy-test-guard.sh 的事，不在这里。

入口：
  classify(words, directory) -> HeavyTest | None
      一条已经剥掉前缀与包装的命令：words[0] 是命令词（照写的样子，没取 basename），其后是参数；directory 是它的当前目录，认不出时 None。
  classify_process(argv, cwd) -> list[HeavyTest]
      一个在跑的进程：argv 是 /proc/<pid>/cmdline 按 NUL 切开的那一串，cwd 是 /proc/<pid>/cwd 指向的目录。
      先照 lib_shell_words 剥掉 bash / sh 起脚本、bash -c、nice / timeout / env / taskset、capped.sh N 这类包装，再逐条 classify；空列表就是不重型。
  runs_compiled_code(words, directory) -> str | None
      一条已经剥掉前缀与包装的命令会不会跑编译出来的代码：cargo test / t / run / r / bench（test、bench 带 --no-run 的只编不跑，不算），
      或直接执行 cargo 编出来的二进制（测试二进制，与名字里带 target 的目录底下 debug / release 里的）；会就交一句说明。
      heavy-test-guard.sh 拿它判子 agent 跑这一类经没经 research/scripts/run-with-memory-cap.sh（与重型不重型无关）。
  judged_by_name(word, directory) -> bool
      这个命令词是不是按名字判的仓内脚本（.claude/gate.d/ 下的阶段、KNOWN_SCRIPT_LOCATIONS 里的脚本在它们的仓内位置上）：
      算不算重型、带什么参数才算，都在名字那一格判完了，要读脚本正文的一方不再读进去。
  python3 lib_heavy_tests.py --selftest

认的输入：cargo 命令行（test 与 run）、按名字认的脚本与门禁阶段、qemu-system-*、herd7，
以及直接执行的测试二进制 `<target 目录>/[<目标三元组>/]<profile>/deps/<名字>-<16 位十六进制哈希>`：按 <名字> 判，名字含 layer0 的算层 0。
"""
import fnmatch, glob, importlib.util, os, re, shlex, shutil, sys, tempfile, tomllib
from typing import NamedTuple


def load_sibling_module(module_name):
    spec = importlib.util.spec_from_file_location(module_name, os.path.join(os.path.dirname(os.path.abspath(__file__)), module_name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


shell_words = load_sibling_module("lib_shell_words")


class HeavyTest(NamedTuple):
    kind: str      # 细分的用法（layer0-cargo、gate-sh……），heavy-test-guard.sh 按它定谁能跑
    category: str  # 报给人看的类名（层 0、全量测试……），KIND_CATEGORY 的值
    detail: str    # 这一条为什么算：认出的是什么


# 每一种重型用法：kind → 它属于哪一类
KIND_CATEGORY = {
    "layer0-stage": "层 0", "layer0-cargo": "层 0", "layer0-binary": "层 0",
    "qemu-stage": "QEMU", "qemu-system": "QEMU", "vm-bench": "QEMU",
    "herd7-stage": "herd7", "lkmm": "herd7", "herd7": "herd7",
    "crates-mutation-stage": "crates 变异整表", "crates-mutation-mutate": "crates 变异整表",
    "full-cargo": "全量测试", "check-sh": "全量测试",
    "gate-sh": "整轮门禁", "gate-staged": "整轮门禁", "replay-all-stage": "全部实验复跑",
    "e152": "E152 装置",
}
# 只有这几道阶段是重型；.claude/gate.d/ 下其余阶段谁都能跑
STAGE_KIND = {"54": "layer0-stage", "55": "qemu-stage", "57": "herd7-stage", "59": "crates-mutation-stage", "87": "replay-all-stage"}
# 按名字判的仓内脚本：名字 → 它在仓里的位置。命令词是这个名字、又落在这个位置上，judged_by_name 为真
KNOWN_SCRIPT_LOCATIONS = {
    "gate.sh": ".claude/scripts/gate.sh", "check.sh": ".claude/scripts/check.sh", "lkmm.sh": ".claude/scripts/lkmm.sh",
    "gate-staged.sh": "research/scripts/gate-staged.sh", "mutate.sh": "research/scripts/mutate.sh",
    "vm-bench.sh": "research/scripts/vm-bench.sh", "e152-run.sh": "research/scripts/e152-run.sh",
    "capped.sh": "research/scripts/capped.sh", "run-with-memory-cap.sh": "research/scripts/run-with-memory-cap.sh",
}


def heavy_test(kind, detail):
    return HeavyTest(kind, KIND_CATEGORY[kind], detail)


def manifest_sections(path):
    try:
        with open(path, "rb") as handle:
            return tomllib.load(handle)
    except (OSError, ValueError):  # TOMLDecodeError 与编码错都是 ValueError
        return None


def nearest_manifest(directory):
    while directory:
        candidate = os.path.join(directory, "Cargo.toml")
        if os.path.isfile(candidate):
            return candidate
        parent = os.path.dirname(directory)
        if parent == directory:
            return None
        directory = parent
    return None


def workspace_root_manifest(manifest_path):
    directory = os.path.dirname(manifest_path)
    while True:
        candidate = os.path.join(directory, "Cargo.toml")
        sections = manifest_sections(candidate) if os.path.isfile(candidate) else None
        if sections and "workspace" in sections:
            return candidate, sections
        parent = os.path.dirname(directory)
        if parent == directory:
            return None, None
        directory = parent


def workspace_packages(root_manifest, root_sections):
    """工作区成员：包名 → 包目录。"""
    root = os.path.dirname(root_manifest)
    packages = {}
    for pattern in (root_sections.get("workspace") or {}).get("members") or []:
        for directory in sorted(glob.glob(os.path.join(root, pattern))):
            sections = manifest_sections(os.path.join(directory, "Cargo.toml"))
            name = ((sections or {}).get("package") or {}).get("name")
            if name:
                packages[name] = os.path.normpath(directory)
    return packages


def layer0_test_targets(package_directory):
    """包里名字含 layer0 的集成测试目标：tests/*.rs、tests/<名>/main.rs 与 [[test]] 的 name。"""
    names = set()
    for path in glob.glob(os.path.join(package_directory, "tests", "*.rs")):
        names.add(os.path.basename(path)[:-3])
    for path in glob.glob(os.path.join(package_directory, "tests", "*", "main.rs")):
        names.add(os.path.basename(os.path.dirname(path)))
    for target in (manifest_sections(os.path.join(package_directory, "Cargo.toml")) or {}).get("test") or []:
        if isinstance(target, dict) and target.get("name"):
            names.add(target["name"])
    return sorted(name for name in names if "layer0" in name)


CARGO_GLOBAL_OPTIONS_WITH_VALUE = {"--color", "--config", "-Z"}
CARGO_TEST_OPTIONS_WITH_VALUE = {"-p", "--package", "--exclude", "--test", "--bin", "--example", "--bench", "-F", "--features",
                                 "--target", "--target-dir", "--manifest-path", "-j", "--jobs", "--profile", "--color",
                                 "--message-format", "--config", "-Z", "--lockfile-path"}
NARROWING_SELECTORS = {"--test", "--lib", "--bin", "--bins", "--example", "--examples", "--bench", "--benches", "--doc"}
GLOB_CHARACTERS = set("*?[")


def cargo_subcommand(arguments, directory):
    """cargo 的全局选项（+工具链、--config、-C 目录……）跳过之后的子命令：交 (子命令, 它后面的参数, 按 -C 换过之后的目录)；没有子命令交 None。"""
    index = 0
    while index < len(arguments):
        argument = arguments[index]
        if argument.startswith("+"):
            index += 1
        elif argument in CARGO_GLOBAL_OPTIONS_WITH_VALUE:
            index += 2
        elif argument == "-C":
            directory = shell_words.resolve_path(directory, arguments[index + 1]) if index + 1 < len(arguments) else directory
            index += 2
        elif argument.startswith("-"):
            index += 1
        else:
            break
    if index >= len(arguments):
        return None
    return arguments[index], arguments[index + 1:], directory


def cargo_use(arguments, directory):
    """cargo 这一条算不算重型：返回 HeavyTest 或 None。"""
    found = cargo_subcommand(arguments, directory)
    if found is None:
        return None
    subcommand, rest, directory = found
    packages, test_names, selectors, binaries = [], [], set(), []
    whole_workspace, manifest_argument = False, None
    position = 0
    while position < len(rest):
        argument = rest[position]
        if argument == "--":
            break
        if argument.startswith("--") and "=" in argument:
            option, value = argument.split("=", 1)
            position += 1
        elif argument.startswith("-p") and len(argument) > 2 and not argument.startswith("--"):
            option, value = "-p", argument[2:].lstrip("=")
            position += 1
        elif argument in CARGO_TEST_OPTIONS_WITH_VALUE:
            option = argument
            value = rest[position + 1] if position + 1 < len(rest) else ""
            position += 2
        else:
            option, value = argument, None
            position += 1
        if option in ("-p", "--package"):
            packages.append(value)
        elif option == "--test":
            test_names.append(value)
            selectors.add(option)
        elif option in ("--bin", "--example", "--bench"):
            selectors.add(option)
            if option == "--bin":
                binaries.append(value)
        elif option == "--manifest-path":
            manifest_argument = value
        elif option in ("--workspace", "--all"):
            whole_workspace = True
        elif option in ("--lib", "--bins", "--examples", "--tests", "--benches", "--all-targets", "--doc"):
            selectors.add(option)
    if subcommand == "run":
        if "e152-file-system-benchmark" in binaries:
            return heavy_test("e152", "cargo run --bin e152-file-system-benchmark")
        return None
    if subcommand not in ("test", "t"):
        return None
    if whole_workspace:
        return heavy_test("full-cargo", "cargo test 带 --workspace / --all")
    named_layer0 = [name for name in test_names if "layer0" in name]
    if named_layer0:
        return heavy_test("layer0-cargo", f"cargo test --test {named_layer0[0]}")
    manifest_path = shell_words.resolve_path(directory, manifest_argument) if manifest_argument else (nearest_manifest(directory) if directory else None)
    sections = manifest_sections(manifest_path) if manifest_path else None
    if sections is None:
        return None
    is_virtual_workspace_root = "workspace" in sections and "package" not in sections
    narrowed = bool(selectors & NARROWING_SELECTORS)
    if not packages and is_virtual_workspace_root and not narrowed:
        return heavy_test("full-cargo", f"在工作区根（{os.path.dirname(manifest_path)}）上不带 -p / --test / --lib / --bin 的 cargo test")
    root_manifest, root_sections = workspace_root_manifest(manifest_path)
    members = workspace_packages(root_manifest, root_sections) if root_manifest else {}
    if packages:
        scope = [members[name] for name in packages if name in members]
    elif is_virtual_workspace_root:
        scope = list(members.values())
    else:
        scope = [os.path.normpath(os.path.dirname(manifest_path))]
    if not narrowed and members and set(scope) == set(members.values()):
        return heavy_test("full-cargo", (f"cargo test 不挑目标，而包的范围是整个工作区（{os.path.dirname(root_manifest)} 的全部 "
                                         f"{len(members)} 个成员），等于全量"))
    layer0_targets = sorted({name for package_directory in scope for name in layer0_test_targets(package_directory)})
    if not layer0_targets:
        return None
    if not selectors or selectors & {"--tests", "--all-targets"}:
        return heavy_test("layer0-cargo", f"cargo test 不挑目标，会跑到名字含 layer0 的测试二进制（{'、'.join(layer0_targets)}）")
    for pattern in test_names:
        if GLOB_CHARACTERS & set(pattern):
            matched = fnmatch.filter(layer0_targets, pattern)
            if matched:
                return heavy_test("layer0-cargo", f"cargo test --test {pattern} 命中名字含 layer0 的测试二进制（{'、'.join(matched)}）")
    return None


GATE_STAGE_PATH = re.compile(r"(?:^|/)\.claude/gate\.d/(\d+)-[^/]+\.sh$")
# cargo 编出来的测试二进制：<target 目录>/[<目标三元组>/]<profile>/deps/<名字>-<16 位十六进制的元数据哈希>，名字里的 - 已换成 _
TEST_BINARY_PATH = re.compile(r"(?:^|/)deps/([A-Za-z0-9_]+)-[0-9a-f]{16}$")


def gate_stage_number(word, directory):
    for candidate in (word, shell_words.resolve_path(directory, word) or ""):
        match = GATE_STAGE_PATH.search(candidate)
        if match:
            return match.group(1)
    return None


def test_binary_name(word):
    """直接执行的是 cargo 编出来的测试二进制时交回它的名字（去掉哈希），不是时交 None。"""
    match = TEST_BINARY_PATH.search(word)
    return match.group(1) if match else None


# 跑编译出来的代码的 cargo 子命令（t、r 是 test、run 的内置简写）；只编不跑的 build、check、clippy 不在内
COMPILED_CODE_SUBCOMMANDS = {"test", "t", "run", "r", "bench"}
# cargo 编出来的二进制直接执行：<名字里带 target 的目录>/[<目标三元组>/]<debug|release>/[deps/]<名字>；名字里不许有点（排掉 .d、.rlib、.so）。
# CARGO_TARGET_DIR 设成名字里不带 target 的目录、再直接执行里面的二进制，认不出（测试二进制按 TEST_BINARY_PATH 照认）
BUILT_BINARY_PATH = re.compile(r"(?:^|/)[^/]*target[^/]*/(?:[^/]+/)?(?:debug|release)/(?:deps/)?[A-Za-z0-9_][A-Za-z0-9_-]*$")


def runs_compiled_code(words, directory):
    """一条剥掉前缀与包装的命令会不会跑编译出来的代码（子 agent 要经 run-with-memory-cap.sh 跑的那一类）：会就交一句说明，不会交 None。"""
    if os.path.basename(words[0]) == "cargo":
        found = cargo_subcommand(words[1:], directory)
        if found is None or found[0] not in COMPILED_CODE_SUBCOMMANDS:
            return None
        subcommand, rest, _directory = found
        options = rest[:rest.index("--")] if "--" in rest else rest
        if "--no-run" in options:
            return None
        return f"cargo {subcommand}"
    if TEST_BINARY_PATH.search(words[0]) or BUILT_BINARY_PATH.search(words[0]):
        return f"直接执行 cargo 编出来的二进制 {words[0]}"
    return None


def classify(words, directory):
    """一条已经剥掉前缀与包装的命令：返回 HeavyTest 或 None。"""
    name, arguments = os.path.basename(words[0]), words[1:]
    stage = gate_stage_number(words[0], directory)
    if stage is not None:
        return heavy_test(STAGE_KIND[stage], f"门禁 {stage} 号（{name}）") if stage in STAGE_KIND else None
    binary = test_binary_name(words[0])
    if binary is not None:
        return heavy_test("layer0-binary", f"直接执行名字含 layer0 的测试二进制（{binary}）") if "layer0" in binary else None
    if name.startswith("qemu-system"):
        return heavy_test("qemu-system", name)
    if name == "vm-bench.sh":
        return heavy_test("vm-bench", "research/scripts/vm-bench.sh（--selftest 也起虚机）")
    if name == "lkmm.sh":
        return heavy_test("lkmm", ".claude/scripts/lkmm.sh")
    if name == "herd7":
        return heavy_test("herd7", "herd7")
    if name == "mutate.sh":
        for argument in arguments:
            resolved = shell_words.resolve_path(directory, argument) or argument
            if argument.endswith("crates/mutations.tsv") or resolved.endswith("/crates/mutations.tsv"):
                return heavy_test("crates-mutation-mutate", "research/scripts/mutate.sh 跑 crates/mutations.tsv 整表")
        return None
    if name == "check.sh":
        return heavy_test("check-sh", "check.sh（里面是全量 cargo test）")
    if name == "gate.sh":
        return heavy_test("gate-sh", " ".join(["gate.sh", *arguments]))
    if name == "gate-staged.sh":
        return None if "--selftest" in arguments else heavy_test("gate-staged", "research/scripts/gate-staged.sh（跑 gate.sh --staged）")
    if name in ("e152-file-system-benchmark", "e152-run.sh"):
        return heavy_test("e152", name)
    if name == "cargo":
        return cargo_use(arguments, directory)
    return None


def classify_process(argv, cwd):
    """一个在跑的进程（argv 与 cwd）里认出的重型用法；包装（bash 起脚本、bash -c、nice、timeout、env、capped.sh……）照共用切词剥掉。"""
    if not argv:
        return []
    found = []
    for command in shell_words.commands_at_command_position(shlex.join(argv), cwd).commands:
        test = classify(command.words, command.directory)
        if test:
            found.append(test)
    return found


def judged_by_name(word, directory):
    """按名字判的仓内脚本（门禁阶段、KNOWN_SCRIPT_LOCATIONS 里的脚本在仓内位置上）：判定在名字那一格做完，不再读脚本正文。"""
    if gate_stage_number(word, directory) is not None:
        return True
    location = KNOWN_SCRIPT_LOCATIONS.get(os.path.basename(word))
    if location is None:
        return False
    return any(candidate == location or candidate.endswith("/" + location)
               for candidate in (os.path.normpath(word), shell_words.resolve_path(directory, word) or ""))


def build_sample_workspace(work):
    """仓根与 research/ 两个虚工作区，harness 里有一个名字含 layer0 的测试目标。"""
    def write(relative, text):
        path = os.path.join(work, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
    write("Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["crates/singlefs-core", "crates/singlefs-harness"]\nexclude = ["research"]\n')
    write("crates/singlefs-core/Cargo.toml", '[package]\nname = "singlefs-core"\nversion = "0.0.0"\n')
    write("crates/singlefs-core/tests/core_contract.rs", "")
    write("crates/singlefs-harness/Cargo.toml", '[package]\nname = "singlefs-harness"\nversion = "0.0.0"\n')
    write("crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs", "")
    write("crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs", "")
    write("research/Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["e7-index-bench"]\n')
    write("research/e7-index-bench/Cargo.toml", '[package]\nname = "e7-index-bench"\nversion = "0.0.0"\n')


def selftest():
    """在临时工作区里判 cargo 命令行、直接执行的测试二进制与按名字判的脚本；返回退出码。"""
    work = tempfile.mkdtemp(prefix="lib-heavy-tests-")
    results = []
    try:
        build_sample_workspace(work)
        harness = os.path.join(work, "crates", "singlefs-harness")
        layer0_binary = "first_transaction_step_seven_layer0-0123456789abcdef"
        # (说明, 进程的 argv, 进程的 cwd, 该认出的 kind；None 是不重型)
        cargo_cases = [
            ("cargo test --all", ["cargo", "test", "--all"], work, "full-cargo"),
            ("argv[0] 是 cargo 的全路径", ["/home/user/.rustup/toolchains/stable/bin/cargo", "test", "--workspace"], work, "full-cargo"),
            ("在工作区根裸跑", ["cargo", "test"], work, "full-cargo"),
            ("-p harness 点名层 0 目标", ["cargo", "test", "--release", "-p", "singlefs-harness", "--test", "first_transaction_step_seven_layer0"], work, "layer0-cargo"),
            ("在 harness 里不挑目标", ["cargo", "test", "--release"], harness, "layer0-cargo"),
            ("经 nice 包一层", ["/usr/bin/nice", "-n", "19", "cargo", "test", "--workspace"], work, "full-cargo"),
            ("bash -c 里", ["bash", "-c", "cd crates/singlefs-harness && cargo test"], work, "layer0-cargo"),
            ("bash 起 54 号", ["bash", ".claude/gate.d/54-layer0-replay.sh", "--full", "/tmp/wt"], work, "layer0-stage"),
            ("cargo run E152", ["cargo", "run", "--release", "--bin", "e152-file-system-benchmark"], os.path.join(work, "research"), "e152"),
            ("一个不是层 0 的测试目标", ["cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], work, None),
            ("clippy --all-targets", ["cargo", "clippy", "--all-targets"], work, None),
            ("bash 起轻阶段 12 号", ["bash", ".claude/gate.d/12-no-prime-marks.sh"], work, None),
        ]
        binary_cases = [
            ("绝对路径的层 0 测试二进制", [os.path.join(work, "target", "release", "deps", layer0_binary), "--test-threads", "4"], work, "layer0-binary"),
            ("相对路径的层 0 测试二进制", [os.path.join("target", "debug", "deps", layer0_binary)], work, "layer0-binary"),
            ("带目标三元组、自定的 target 目录", ["/tmp/target-elsewhere/x86_64-unknown-linux-gnu/release/deps/some_layer0_stream-fedcba9876543210"], work, "layer0-binary"),
            ("经 timeout 包一层", ["timeout", "600", os.path.join(".", "target", "release", "deps", layer0_binary), "--exact", "case"], work, "layer0-binary"),
            ("名字不含 layer0 的测试二进制", [os.path.join(work, "target", "release", "deps", "second_transaction_step_one_overwrite-0123456789abcdef")], work, None),
            ("deps 下的 .d 依赖文件不是二进制", [os.path.join(work, "target", "release", "deps", layer0_binary + ".d")], work, None),
            ("哈希不是 16 位", [os.path.join(work, "target", "release", "deps", "first_transaction_step_seven_layer0-0123abcd")], work, None),
            ("名字只当参数", ["grep", "-c", "x", os.path.join("target", "release", "deps", layer0_binary)], work, None),
        ]
        for label, argv, cwd, want in cargo_cases + binary_cases:
            found = classify_process(argv, cwd)
            results.append((label, want, found[0].kind if found else None))
        name_cases = [
            ("门禁阶段按名字判", ".claude/gate.d/12-no-prime-marks.sh", work, True),
            ("仓内位置上的 gate-staged.sh 按名字判", "research/scripts/gate-staged.sh", work, True),
            ("从 research 里相对着写的 mutate.sh", "scripts/mutate.sh", os.path.join(work, "research"), True),
            ("仓外的同名 mutate.sh 不按名字判", "/tmp/claude-1000/somewhere/mutate.sh", work, False),
            ("没登记的脚本", "run-chain.sh", work, False),
        ]
        for label, word, directory, want in name_cases:
            results.append((label, want, judged_by_name(word, directory)))
        # 跑不跑编译出来的代码（与重型不重型无关）：真 / 假
        compiled_cases = [
            ("cargo test 一个测试目标", ["cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], True),
            ("cargo t 简写", ["cargo", "t", "--lib"], True),
            ("cargo run 实验二进制", ["cargo", "run", "--release", "--bin", "e160-random-small-read-share"], True),
            ("cargo r 简写", ["cargo", "r"], True),
            ("cargo bench", ["cargo", "bench"], True),
            ("全局选项在子命令前", ["cargo", "+stable", "--offline", "-C", "crates/singlefs-core", "test"], True),
            ("cargo test --no-run 只编不跑", ["cargo", "test", "--no-run", "-p", "singlefs-core"], False),
            ("-- 之后的 --no-run 是测试二进制的参数", ["cargo", "test", "--", "--no-run"], True),
            ("cargo build", ["cargo", "build", "--release"], False),
            ("cargo b 简写是 build", ["cargo", "b"], False),
            ("cargo clippy", ["cargo", "clippy", "--all-targets"], False),
            ("cargo 只带全局选项", ["cargo", "--version"], False),
            ("直接执行测试二进制", [os.path.join("target", "debug", "deps", "second_transaction_step_one_overwrite-0123456789abcdef")], True),
            ("直接执行 research 里的实验二进制", ["research/target/release/e142_region_diff_independent"], True),
            ("直接执行名字里带 target 的自定编译目录里的二进制", ["/tmp/singlefs-crates-mutation-target/release/tiny"], True),
            ("带目标三元组的", ["./target/x86_64-unknown-linux-musl/release/first_transaction_on_device"], True),
            ("deps 下的 .d 依赖文件", [os.path.join("target", "release", "deps", "tiny.d")], False),
            ("名字只当参数", ["ls", "target/release/tiny"], False),
            ("不在 target 底下的程序", ["./prog"], False),
        ]
        for label, words, want in compiled_cases:
            results.append((f"跑编译出来的代码：{label}", want, runs_compiled_code(words, work) is not None))
    finally:
        shutil.rmtree(work)
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ lib_heavy_tests 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 classify()、cargo_subcommand()、cargo_use()、test_binary_name()、runs_compiled_code()、classify_process()、judged_by_name() "
              "与 TEST_BINARY_PATH / BUILT_BINARY_PATH / COMPILED_CODE_SUBCOMMANDS / KNOWN_SCRIPT_LOCATIONS 几张表")
        return 1
    print(f"  ✓ lib_heavy_tests 自检通过（查了 {len(results)} 种：cargo 与包装过的命令行 {len(cargo_cases)} 种、"
          f"直接执行的测试二进制 {len(binary_cases)} 种、按名字判的脚本 {len(name_cases)} 种、跑不跑编译出来的代码 {len(compiled_cases)} 种）")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    print("用法：python3 .claude/hooks/lib_heavy_tests.py --selftest（判定入口按文件路径导入这份模块再调，见文件头）", file=sys.stderr)
    sys.exit(2)
