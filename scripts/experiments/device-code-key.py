#!/usr/bin/env python3
# admission: always 每次照此刻的源码、清单、Cargo.lock 与工具链现算代码键，上一次算出的键不替这一次作保
# run-condition: none 要的 unit-graph 由它自己按 --unit-graph、SINGLEFS_UNIT_GRAPH、<仓根>/target/release/unit-graph 的次序找，找不到退 78（出路 cargo build --release -p singlefs-unit-graph）；run 里的 cargo 与装置由它自己起、自己判
"""实验装置的代码键：装置入口函数的单元闭包键，加工具链行与闭包真引到的外部依赖行。
E178 第四段与 E174 父进程的续跑键都由它取（登记 research/prompts/e178-preregistration.md 修订 4 之五、之六与修订 5，
research/prompts/e174-preregistration.md 修订 2 之五与修订 3）；装置拿它交的代码键再拼上一项（一轮）自己的参数，算项键（轮键）。

用法（<仓根> 是装着 scripts/experiments 的那一棵树）：
  device-code-key.py key <仓根> --binary <bin 名> --entry <fn 路径>… [--feature <特性>]… [--unit-graph <程序>] [--threads <N>] [--host 本机|第二台]
      标准输出打组成清单（行格式见「组成清单」），每个入口一行 code_key。闭包有拒算：照打拒算行、不打 code_key、退 3。
  device-code-key.py run <仓根> <与 key 相同的选项> [--when-refused=<装置参数>]… -- <装置参数…>
      先取键；再在 <仓根>/scripts/experiments 里 cargo build --release --bin <bin 名> [--features <特性,…>]；再起装置
      （<CARGO_TARGET_DIR 或 target>/release/<bin 名>，装置参数后面接 --code-key-file <组成清单> --written-list <文件>）；
      装置退出后再取一次键：有一个入口的代码键与第一次不等，删掉装置在 --written-list 里逐行列出的这一趟写进库的文件、退 1
      （红句「源码在编译与跑之间变了」）；都相等就交装置的退出码。
      键算不出（拒算、工具链起不来）：照编、照起装置，只是不给 --code-key-file 与 --written-list、改接 --when-refused 给的参数
      （E178 给 --no-reuse，E174 给 --without-library：每项现判、不读不写库），装置退 0 时这里退 3。
      两种起法都给装置的环境另放 DEVICE_CODE_KEY_DEVICE_BINARY=<起的那一份装置的路径>：装置要起自己的子进程照它起（E174 的 run），不读自己的路径
      （单元键认不出 std::env::current_exe 这种读自己路径的起法，整个 main 闭包拒算）。
  device-code-key.py --selftest

键的组成（每个入口一把代码键；代码键 = unit-graph key 打的 key 那一个 hex，它把单元、工具链行、依赖行一起算进去）：
  单元     unit-graph closure 与 key：<仓根>/scripts <入口> --target bin:<bin 名> [--cfg feature="<特性>"]… --threads <N>
  工具链行  scripts/gate/admission.py 的 KeyContext(<仓根>).rust_toolchain_lines(<仓根>/scripts/experiments) 的每一行（rustc -vV、cargo -V、
           构建环境的生效值）；另加「profile release」一行、清单 [profile.release] 照 tomllib 解析后逐键摊开的 profile.release.<键>=<值>
           （admission.py 的 flattened_setting_lines，排好序）、「features none」或「features <特性,…>」一行、
           「host: <三元组>」一行（rustc -vV 的 host:，unit-graph 拿它定目标平台的 cfg；closure 与 key 两趟都给）
  依赖行   闭包里的 Rust 单元点到的外部包与它们的传递依赖（名、版本、来源、checksum）：每个单元的原文（unit-graph 交的「文件:起-止」那几行）
           经 scripts/gate/lib_function_units.py 的 rust_lex 与 token_references 取出点到的名字（裸名与路径的头一段），交给 admission.py 的
           KeyContext.dependency_lines 那一份收窄法（清单写的外部依赖里代码名被点到的才收，再顺 Cargo.lock 收传递依赖）
  不进键：入口名、bin 名、路径、行号、--threads、--host、日期。

组成清单（制表符分隔，一行一项；装置照原样存进库的 codes/<代码键>.txt，只给「键对不上时变了哪些单元」用，不进键）：
  form<TAB>device-code-key-v1
  judged<TAB><YYYY-MM-DD>        取键那天（东京时间），装置写进项文件的 meta 行
  host<TAB><本机|第二台>
  code_key<TAB><入口><TAB><hex>  每个入口一行，排在别的行前面（只在 form 之后）
  toolchain<TAB><行>
  dependency<TAB><入口><TAB><行>
  unit<TAB><入口><TAB><unit-graph 的单元行：种类、限定名、文件:起-止、指纹>
  refused<TAB><入口><TAB><unit-graph 的拒算行去掉开头的 refused 那一段>

复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）：自证 reuse-three-questions 一格，两个入口各一项、照装置的存法在内存里按代码键记判过没有，
  打规范输出「REUSE_SELFTEST script=scripts/experiments/device-code-key.py …」。复用哨兵不在这里：判定库在装置里（E174、E178 的库），哨兵要接在装置的取库那一段。

退出码：0 成；1 run 里编不过、两次键不等、起不了装置；2 用法错、入口找不到；3 键算不出；78 unit-graph 找不到。
  run 里装置自己退非 0 时交装置的退出码（键算不出的那一趟也是）。

弄坏开关 DEVICE_CODE_KEY_BREAK=<项>[,<项>…]：只在 --selftest 起的那几趟里认（自证给子进程设 DEVICE_CODE_KEY_SELFTEST_RUN=1），
平常跑设了也不认。自证逐个打开，看红在点名的那一格：
  raw-source-digest              代码键改取闭包里那几份源文件整份原文的 sha256（不经 unit-graph 的去注释指纹）  →「只加一行注释键不变」「加一个没人调的函数键不变」两格红（整份原文动一个字节键就变）；
                                 reuse-three-questions 也红（② 改没人调的函数两项都现判、③ 两项都现判）
  toolchain-lines-dropped        不给工具链行（只留 host: 那一行）                                             →「profile.release 改一个值键变」格红
  host-line-dropped              工具链行里不加 host: <三元组> 那一行（改前的样子）                            →「工具链行带一行 host: <三元组>」格红（取键的几格跟着红）
  dependency-lines-dropped       不给依赖行                                                                   →「闭包点到的包在 Cargo.lock 里抬一版键变」格红
  dependency-lines-not-narrowed  依赖行收清单写的全部外部依赖（不按闭包点到的收窄）                            →「闭包没点到的包抬一版键不变」格红
  bracket-skipped                run 跑完装置不再取第二次键                                                     →「编译与跑之间改了入口调到的函数」格红
  device-binary-unset            起装置不放 DEVICE_CODE_KEY_DEVICE_BINARY                                        →「拒算：退 3…」「前后两次键相等…」两格红
"""
import datetime
import hashlib
import os
import shutil
import subprocess
import sys
import tempfile
import tomllib
import types

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "gate"))
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "hosts"))
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "..", ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402
import admission  # noqa: E402
import lib_function_units  # noqa: E402
from heavy_threads import default_heavy_threads  # noqa: E402

PACKAGE_DIRECTORY = os.path.join("scripts", "experiments")
SCRIPT_KEY = "scripts/experiments/device-code-key.py"   # 复用三问规范输出认的脚本名
UNIT_GRAPH_ROOT = "scripts"
FORM_LINE = "form\tdevice-code-key-v1"
RUSTC_HOST_PREFIX = "rustc -vV: host: "
DEVICE_BINARY_VARIABLE = "DEVICE_CODE_KEY_DEVICE_BINARY"
UNIT_GRAPH_VARIABLE = "SINGLEFS_UNIT_GRAPH"
DEFAULT_UNIT_GRAPH = os.path.join("target", "release", "unit-graph")
BREAK_VARIABLE = "DEVICE_CODE_KEY_BREAK"
SELFTEST_RUN_VARIABLE = "DEVICE_CODE_KEY_SELFTEST_RUN"
BREAK_SWITCHES = ("raw-source-digest", "toolchain-lines-dropped", "host-line-dropped", "dependency-lines-dropped", "dependency-lines-not-narrowed", "bracket-skipped",
                  "device-binary-unset")
HOST_LABELS = ("本机", "第二台")
REFUSAL_SHOWN = 3
TOKYO = datetime.timezone(datetime.timedelta(hours=9))
EXIT_FAILED, EXIT_USAGE, EXIT_REFUSED, EXIT_UNIT_GRAPH_MISSING = 1, 2, 3, 78


class UsageError(Exception):
    """命令行写错、入口找不到：退 2。"""


class UnitGraphMissing(Exception):
    """unit-graph 程序找不到：退 78。"""


def break_is_set(switch):
    if os.environ.get(SELFTEST_RUN_VARIABLE) != "1":
        return False
    return switch in os.environ.get(BREAK_VARIABLE, "").split(",")


def today_in_tokyo():
    return datetime.datetime.now(TOKYO).date().isoformat()


def find_unit_graph(root, given):
    """--unit-graph、SINGLEFS_UNIT_GRAPH、<仓根>/target/release/unit-graph，取第一个给了的；给了而不是可执行文件也算找不到。"""
    candidate = given or os.environ.get(UNIT_GRAPH_VARIABLE) or os.path.join(root, DEFAULT_UNIT_GRAPH)
    if os.path.isfile(candidate) and os.access(candidate, os.X_OK):
        return candidate
    raise UnitGraphMissing(f"unit-graph 找不到：{candidate}（--unit-graph、{UNIT_GRAPH_VARIABLE}、<仓根>/{DEFAULT_UNIT_GRAPH} 依次找）")


def toolchain_lines(root, features):
    """工具链行（文件头「键的组成」）。工具链起不来、rustc -vV 交的 host: 不是恰好一行，抛 admission.UnitKeyError（调用方按键算不出办）。"""
    package_directory = os.path.join(root, PACKAGE_DIRECTORY)
    lines = list(admission.KeyContext(root).rust_toolchain_lines(package_directory))
    hosts = [line[len(RUSTC_HOST_PREFIX):] for line in lines if line.startswith(RUSTC_HOST_PREFIX)]
    if len(hosts) != 1:
        raise admission.UnitKeyError(f"rustc -vV 交了 {len(hosts)} 行 host:（要恰好一行：unit-graph 拿它定目标平台的 cfg）")
    triple = [] if break_is_set("host-line-dropped") else [f"host: {hosts[0]}"]
    if break_is_set("toolchain-lines-dropped"):
        return triple
    with open(os.path.join(package_directory, "Cargo.toml"), "rb") as handle:
        manifest = tomllib.load(handle)
    flattened = []
    admission.flattened_setting_lines("profile.release", (manifest.get("profile") or {}).get("release") or {}, flattened)
    lines += ["profile release"] + sorted(flattened)
    lines.append(f"features {','.join(sorted(features)) if features else 'none'}")
    return lines + triple


def package_name(root):
    with open(os.path.join(root, PACKAGE_DIRECTORY, "Cargo.toml"), "rb") as handle:
        return ((tomllib.load(handle).get("package") or {}).get("name")) or ""


def unit_line_source_text(root, unit_line, file_cache):
    """单元行「种类 限定名 文件:起-止 指纹」那几行原文；数据单元、WGSL 单元、读不到的交 None。"""
    columns = unit_line.split("\t")
    if len(columns) != 4 or columns[0] == "data" or columns[0].startswith("wgsl"):
        return None
    path, _colon, span = columns[2].rpartition(":")
    start_text, _dash, end_text = span.partition("-")
    if not path.endswith(".rs") or not start_text.isdigit() or not end_text.isdigit():
        return None
    if path not in file_cache:
        try:
            with open(os.path.join(root, path), encoding="utf-8") as handle:
                file_cache[path] = handle.read().split("\n")
        except OSError:
            file_cache[path] = None
    lines = file_cache[path]
    if lines is None:
        return None
    return "\n".join(lines[int(start_text) - 1:int(end_text)])


def referenced_names(root, unit_lines):
    """闭包里的 Rust 单元点到的名字：裸名与「限定::名」的头一段（R-裸名与 R-路径读的那几样，lib_function_units.py 的 token_references）。"""
    names, file_cache = set(), {}
    for unit_line in unit_lines:
        text = unit_line_source_text(root, unit_line, file_cache)
        if text is None:
            continue
        kinds, texts, _lines, _documents = lib_function_units.rust_lex(text)
        plain, _methods, qualified, _dyn_traits = lib_function_units.token_references(kinds, texts, 0, len(texts))
        names |= set(plain) | {pair[0] for pair in qualified}
    return names


def dependency_lines(root, unit_lines):
    """闭包点到的外部包与它们的传递依赖：照 admission.py 的 KeyContext.dependency_lines 收（它要的闭包与工作区只用到下面这几个属性，这里给一份只装这几样的）。"""
    if break_is_set("dependency-lines-dropped"):
        return []
    closure_root = os.path.join(root, PACKAGE_DIRECTORY)
    context = admission.KeyContext(root)
    names = referenced_names(root, unit_lines)
    if break_is_set("dependency-lines-not-narrowed"):
        names |= set(admission.manifest_dependency_names(closure_root, ".", False, context.host_triple(closure_root)))
    crate = types.SimpleNamespace(directory=".", package=package_name(root))
    workspace = types.SimpleNamespace(crate_of_path=lambda _path: crate, targets={})
    unit = types.SimpleNamespace(language="rust", data=False, file=PACKAGE_DIRECTORY, plain=names, qualified=())
    closure = types.SimpleNamespace(order=[unit], cfg_lines=[])
    return context.dependency_lines(closure_root, workspace, closure, None)


def unit_graph_arguments(root, command, entry, binary, features, threads, toolchain=(), dependencies=()):
    arguments = [command, os.path.join(root, UNIT_GRAPH_ROOT), entry, "--target", f"bin:{binary}"]
    for feature in sorted(features):
        arguments += ["--cfg", f'feature="{feature}"']
    arguments += ["--threads", str(threads)]
    for line in toolchain:
        arguments += ["--toolchain-line", line]
    for line in dependencies:
        arguments += ["--lock-line", line]
    return arguments


def call_unit_graph(unit_graph, arguments):
    """→ (退出码, 标准输出的行)。退 2（用法错、入口找不到）抛 UsageError。"""
    completed = subprocess.run([unit_graph] + arguments, capture_output=True, text=True)
    if completed.returncode == EXIT_USAGE:
        raise UsageError(f"unit-graph {' '.join(arguments[:3])} 退 2：{completed.stderr.strip()}")
    if completed.returncode not in (0, EXIT_REFUSED):
        raise UsageError(f"unit-graph {' '.join(arguments[:3])} 退 {completed.returncode}：{completed.stderr.strip()}")
    return completed.returncode, [line for line in completed.stdout.split("\n") if line]


def raw_source_key(root, unit_lines, toolchain, dependencies):
    """弄坏开关 raw-source-digest：闭包里那几份源文件整份原文（带注释）的 sha256，再接工具链行与依赖行。"""
    digest = hashlib.sha256()
    for path in sorted({line.split("\t")[2].rpartition(":")[0] for line in unit_lines if len(line.split("\t")) == 4}):
        with open(os.path.join(root, path), "rb") as handle:
            digest.update(handle.read())
    digest.update("\n".join(list(toolchain) + list(dependencies)).encode())
    return digest.hexdigest()


def compute_code_keys(root, options):
    """每个入口一把代码键。→ (组成清单的行, {入口: 代码键或 None}, 拒算行)；键算不出的入口交 None，拒算行写明是哪一处。"""
    unit_graph = find_unit_graph(root, options.unit_graph)
    head = [FORM_LINE, f"judged\t{today_in_tokyo()}", f"host\t{options.host}"]
    code_keys, refusals, body = {}, [], []
    try:
        toolchain = toolchain_lines(root, options.features)
    except admission.UnitKeyError as error:
        refusals = [f"refused\t{entry}\ttoolchain-unavailable\t-\t{error}" for entry in options.entries]
        return head + refusals, {entry: None for entry in options.entries}, refusals
    body += [f"toolchain\t{line}" for line in toolchain]
    for entry in options.entries:
        code, closure = call_unit_graph(unit_graph, unit_graph_arguments(root, "closure", entry, options.binary, options.features, options.threads, toolchain))
        unit_lines = [line for line in closure if not line.startswith(("key\t", "refused\t"))]
        entry_refusals = [f"refused\t{entry}\t{line.partition(chr(9))[2]}" for line in closure if line.startswith("refused\t")]
        body += [f"unit\t{entry}\t{line}" for line in unit_lines]
        if code == EXIT_REFUSED or entry_refusals:
            refusals += entry_refusals or [f"refused\t{entry}\tunit-graph-exit-3\t-\t-"]
            code_keys[entry] = None
            continue
        dependencies = dependency_lines(root, unit_lines)
        body += [f"dependency\t{entry}\t{line}" for line in dependencies]
        if break_is_set("raw-source-digest"):
            code_keys[entry] = raw_source_key(root, unit_lines, toolchain, dependencies)
            continue
        code, keyed = call_unit_graph(unit_graph, unit_graph_arguments(root, "key", entry, options.binary, options.features, options.threads,
                                                                       toolchain, dependencies))
        key_lines = [line.partition("\t")[2] for line in keyed if line.startswith("key\t")]
        if code == EXIT_REFUSED or len(key_lines) != 1:
            refusals += [f"refused\t{entry}\t{line.partition(chr(9))[2]}" for line in keyed if line.startswith("refused\t")] or \
                [f"refused\t{entry}\tunit-graph-key-line\t-\t交了 {len(key_lines)} 行 key"]
            code_keys[entry] = None
            continue
        code_keys[entry] = key_lines[0]
    key_rows = [f"code_key\t{entry}\t{code_keys[entry]}" for entry in options.entries if code_keys[entry] is not None]
    return head + key_rows + body + refusals, code_keys, refusals


def refusal_summary(refusals):
    shown = "；".join(" ".join(line.split("\t")[2:4]) for line in refusals[:REFUSAL_SHOWN])
    more = f"；另 {len(refusals) - REFUSAL_SHOWN} 处" if len(refusals) > REFUSAL_SHOWN else ""
    return f"拒算 {len(refusals)} 处：{shown}{more}"


def run_key(root, options):
    lines, _code_keys, refusals = compute_code_keys(root, options)
    sys.stdout.write("".join(f"{line}\n" for line in lines))
    if refusals:
        print(f"device-code-key：键算不出（{refusal_summary(refusals)}）", file=sys.stderr)
        return EXIT_REFUSED
    return 0


def device_binary_path(root, binary):
    package_directory = os.path.join(root, PACKAGE_DIRECTORY)
    target_directory = os.environ.get("CARGO_TARGET_DIR") or "target"
    return os.path.join(package_directory, target_directory, "release", binary)


def device_environment(root, options):
    """起装置的环境：另放 DEVICE_CODE_KEY_DEVICE_BINARY=<编好、要起的那一份装置的路径>，装置起自己的子进程照它起（E174 的 run），不读自己的路径。
    弄坏开关 device-binary-unset 下不放。"""
    environment = dict(os.environ)
    if not break_is_set("device-binary-unset"):
        environment[DEVICE_BINARY_VARIABLE] = device_binary_path(root, options.binary)
    return environment


def build_device(root, options):
    command = ["cargo", "build", "--release", "--bin", options.binary]
    if options.features:
        command += ["--features", ",".join(sorted(options.features))]
    completed = subprocess.run(command, cwd=os.path.join(root, PACKAGE_DIRECTORY))
    if completed.returncode != 0:
        print(f"device-code-key：{' '.join(command)} 退 {completed.returncode}，没起装置", file=sys.stderr)
    return completed.returncode == 0


def remove_written_files(written_list_path):
    """装置在 --written-list 里逐行列的这一趟写进库的文件，删掉（只删存在的普通文件）；→ 删了几份。"""
    try:
        with open(written_list_path, encoding="utf-8") as handle:
            paths = [line.strip() for line in handle if line.strip()]
    except OSError:
        return 0
    removed = 0
    for path in paths:
        if os.path.isfile(path) and not os.path.islink(path):
            os.remove(path)
            removed += 1
    return removed


def run_device(root, options):
    composition_lines, code_keys, refusals = compute_code_keys(root, options)
    if refusals:
        print(f"device-code-key：键算不出（{refusal_summary(refusals)}），这一趟装置每项现判、不读不写库", file=sys.stderr)
        if not build_device(root, options):
            return EXIT_FAILED
        completed = subprocess.run([device_binary_path(root, options.binary)] + options.device_arguments + options.when_refused,
                                   env=device_environment(root, options))
        return EXIT_REFUSED if completed.returncode == 0 else completed.returncode
    work_directory = tempfile.mkdtemp(prefix="device-code-key-")
    try:
        composition_path = os.path.join(work_directory, "composition.txt")
        written_list_path = os.path.join(work_directory, "written.txt")
        with open(composition_path, "w", encoding="utf-8") as handle:
            handle.write("".join(f"{line}\n" for line in composition_lines))
        open(written_list_path, "w", encoding="utf-8").close()
        if not build_device(root, options):
            return EXIT_FAILED
        try:
            completed = subprocess.run([device_binary_path(root, options.binary)] + options.device_arguments
                                       + ["--code-key-file", composition_path, "--written-list", written_list_path],
                                       env=device_environment(root, options))
        except OSError as error:
            print(f"device-code-key：起不了装置 {device_binary_path(root, options.binary)}：{error}", file=sys.stderr)
            return EXIT_FAILED
        if not break_is_set("bracket-skipped"):
            _after_lines, after_keys, _after_refusals = compute_code_keys(root, options)
            changed = [entry for entry in options.entries if after_keys.get(entry) != code_keys[entry]]
            if changed:
                removed = remove_written_files(written_list_path)
                detail = "；".join(f"{entry}：{code_keys[entry]} → {after_keys.get(entry)}" for entry in changed)
                print(f"device-code-key：✗ 源码在编译与跑之间变了（{detail}），删掉这一趟写进库的 {removed} 份文件", file=sys.stderr)
                return EXIT_FAILED
        print("device-code-key：代码键 " + "，".join(f"{entry} {code_keys[entry]}" for entry in options.entries) + "（编译之前与装置退出之后相同）",
              file=sys.stderr)
        return completed.returncode
    finally:
        shutil.rmtree(work_directory, ignore_errors=True)


def parse_options(command, arguments):
    """key / run 的参数 → (仓根, 选项)。写错抛 UsageError。"""
    options = types.SimpleNamespace(binary=None, entries=[], features=[], unit_graph=None, threads=None, host=HOST_LABELS[0],
                                    when_refused=[], device_arguments=[])
    if "--" in arguments:
        separator = arguments.index("--")
        arguments, options.device_arguments = arguments[:separator], arguments[separator + 1:]
        if command != "run":
            raise UsageError("只有 run 带 -- <装置参数…>")
    elif command == "run":
        raise UsageError("run 要写 -- <装置参数…>")
    if not arguments or arguments[0].startswith("--"):
        raise UsageError(f"{command} 要先给 <仓根>")
    root = os.path.abspath(arguments[0])
    valued = {"--binary": "binary", "--unit-graph": "unit_graph", "--threads": "threads", "--host": "host"}
    index = 1
    while index < len(arguments):
        flag = arguments[index]
        if flag.startswith("--when-refused=") and command == "run":
            options.when_refused.append(flag.partition("=")[2])
            index += 1
            continue
        if flag not in valued and flag not in ("--entry", "--feature"):
            raise UsageError(f"认不得的参数：{flag}")
        if index + 1 >= len(arguments):
            raise UsageError(f"{flag} 后面缺值")
        value = arguments[index + 1]
        if flag == "--entry":
            options.entries.append(value)
        elif flag == "--feature":
            options.features.append(value)
        else:
            setattr(options, valued[flag], value)
        index += 2
    if not options.binary or not options.entries:
        raise UsageError("--binary 与至少一个 --entry 都要给")
    if len(set(options.entries)) != len(options.entries):
        raise UsageError("同一个 --entry 给了两次")
    if options.host not in HOST_LABELS:
        raise UsageError(f"--host 只认 {' / '.join(HOST_LABELS)}")
    if options.threads is None:
        try:
            options.threads = default_heavy_threads(os.environ)
        except Exception as error:  # 登记文件读不出数：照用法错退 2，出路写给 --threads
            raise UsageError(f"没给 --threads，登记的重活默认并行度又读不出（{error}）") from error
    elif not str(options.threads).isdigit() or int(options.threads) < 1:
        raise UsageError("--threads 要正整数")
    if not os.path.isdir(os.path.join(root, PACKAGE_DIRECTORY)):
        raise UsageError(f"<仓根> 下没有 {PACKAGE_DIRECTORY}：{root}")
    return root, options


USAGE = ("用法：device-code-key.py key <仓根> --binary <bin 名> --entry <fn 路径>… [--feature <特性>]… [--unit-graph <程序>] [--threads <N>] [--host 本机|第二台]\n"
         "      device-code-key.py run <仓根> <同上选项> [--when-refused=<装置参数>]… -- <装置参数…>\n"
         "      device-code-key.py --selftest")


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    if not arguments or arguments[0] not in ("key", "run"):
        print(USAGE, file=sys.stderr)
        return EXIT_USAGE
    try:
        root, options = parse_options(arguments[0], arguments[1:])
        return run_key(root, options) if arguments[0] == "key" else run_device(root, options)
    except UsageError as error:
        print(f"device-code-key：{error}\n{USAGE}", file=sys.stderr)
        return EXIT_USAGE
    except UnitGraphMissing as error:
        print(f"  ✗ device-code-key：{error}", file=sys.stderr)
        print("     → 怎么办：cargo build --release -p singlefs-unit-graph（在仓根），或用 --unit-graph / SINGLEFS_UNIT_GRAPH 指到编好的 unit-graph",
              file=sys.stderr)
        return EXIT_UNIT_GRAPH_MISSING


# ── 自证：假的 unit-graph、rustc、cargo 与装置，在临时目录里造一棵只有清单、Cargo.lock 与一份源码的小树，不编译 ──

SELFTEST_UNIT_GRAPH = r'''
import hashlib, os, re, sys
arguments = sys.argv[1:]
command, root, entry = arguments[0], arguments[1], arguments[2]
toolchain = [arguments[index + 1] for index, word in enumerate(arguments) if word == "--toolchain-line"]
locks = [arguments[index + 1] for index, word in enumerate(arguments) if word == "--lock-line"]
if sum(1 for line in toolchain if line.startswith("host: ")) != 1:
    print("unit-graph：缺目标三元组：--toolchain-line 里要有一行 host: <三元组>（rustc -vV）或 target: <三元组>（cargo --target）", file=sys.stderr)
    sys.exit(2)
if os.environ.get("FAKE_UNIT_GRAPH_REFUSE") == "1":
    print('refused\titem-level-include\tscripts/experiments/device.rs:1\tinclude ! ( "x.rs" ) ;')
    sys.exit(3)
lines = open(os.path.join(root, "experiments", "device.rs"), encoding="utf-8").read().split("\n")
functions, position = {}, 0
while position < len(lines):
    match = re.match(r"fn (\w+)\(", lines[position])
    if not match:
        position += 1
        continue
    last = position
    while lines[last] != "}":
        last += 1
    body = [re.sub(r"//.*", "", line).strip() for line in lines[position:last + 1]]
    functions[match.group(1)] = (position + 1, last + 1, "\n".join(line for line in body if line))
    position = last + 1
name = entry.rpartition("::")[2]
if name not in functions:
    print(f"入口 {entry} 找不到", file=sys.stderr)
    sys.exit(2)
closure, todo = set(), [name]
while todo:
    current = todo.pop()
    if current in closure:
        continue
    closure.add(current)
    todo += [other for other in functions if re.search(r"\b" + other + r"\b", functions[current][2])]
digest = lambda text: hashlib.sha256(text.encode()).hexdigest()
units = sorted(f"fn\tfake::{unit}\tscripts/experiments/device.rs:{functions[unit][0]}-{functions[unit][1]}\t{digest(functions[unit][2])}" for unit in closure)
key = digest("\n".join(sorted(unit.split("\t")[3] for unit in units) + sorted("toolchain " + line for line in toolchain) + sorted("lock " + line for line in locks)))
if command == "closure":
    print("\n".join(units))
print(f"key\t{key}")
'''

SELFTEST_RUSTC = r'''
import sys
print("rustc 0.0.0 (fake 2026-01-01)\nbinary: rustc\nhost: x86_64-unknown-linux-gnu\nrelease: 0.0.0" if sys.argv[1:] == ["-vV"] else "rustc 0.0.0 (fake 2026-01-01)")
'''

SELFTEST_CARGO = r'''
import os, sys
if sys.argv[1:] == ["-V"]:
    print("cargo 0.0.0 (fake 2026-01-01)")
    sys.exit(0)
with open(os.environ["FAKE_CARGO_LOG"], "a", encoding="utf-8") as handle:
    handle.write(" ".join(sys.argv[1:]) + "\n")
sys.exit(int(os.environ.get("FAKE_CARGO_EXIT", "0")))
'''

SELFTEST_DEVICE = r'''
import os, shutil, sys
arguments = sys.argv[1:]
with open(os.environ["FAKE_DEVICE_LOG"], "a", encoding="utf-8") as handle:
    handle.write("\t".join(arguments) + "\n")
library = os.environ["FAKE_DEVICE_LIBRARY"]
os.makedirs(library, exist_ok=True)
with open(os.path.join(library, "device-binary.txt"), "w", encoding="utf-8") as handle:
    handle.write(os.environ.get("DEVICE_CODE_KEY_DEVICE_BINARY", "没设"))
if "--written-list" in arguments:
    item = os.path.join(library, "item.out")
    with open(item, "w", encoding="utf-8") as handle:
        handle.write("judged\n")
    with open(arguments[arguments.index("--written-list") + 1], "a", encoding="utf-8") as handle:
        handle.write(item + "\n")
    shutil.copyfile(arguments[arguments.index("--code-key-file") + 1], os.path.join(library, "composition.txt"))
edited = os.environ.get("FAKE_DEVICE_EDIT_SOURCE")
if edited:
    with open(edited, encoding="utf-8") as handle:
        text = handle.read()
    with open(edited, "w", encoding="utf-8") as handle:
        handle.write(text.replace("compute(1)", "compute(2)"))
sys.exit(int(os.environ.get("FAKE_DEVICE_EXIT", "0")))
'''

SELFTEST_SOURCE = """fn entry_function() {
    helper_function();
}

fn helper_function() {
    let value = fake_external::compute(1);
}
"""

SELFTEST_MANIFEST = """[package]
name = "fake-package"
version = "0.1.0"
edition = "2021"

[dependencies]
fake-external = "1.0"
unused-external = "2.0"

[profile.release]
opt-level = 3
overflow-checks = true
"""

SELFTEST_LOCK = """version = 4

[[package]]
name = "fake-external"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aaaa"

[[package]]
name = "fake-package"
version = "0.1.0"
dependencies = [
 "fake-external",
 "unused-external",
]

[[package]]
name = "unused-external"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bbbb"
"""


def selftest_tree(directory):
    """一棵小树：scripts/experiments 下的清单、Cargo.lock、装置源码与编好的假装置；fake-bin 下假 rustc、cargo；tools 下假 unit-graph。→ 环境变量。"""
    package = os.path.join(directory, "tree", PACKAGE_DIRECTORY)
    os.makedirs(os.path.join(package, "target", "release"))
    for name, text in (("Cargo.toml", SELFTEST_MANIFEST), ("Cargo.lock", SELFTEST_LOCK), ("device.rs", SELFTEST_SOURCE)):
        with open(os.path.join(package, name), "w", encoding="utf-8") as handle:
            handle.write(text)
    programs = {os.path.join(directory, "fake-bin", "rustc"): SELFTEST_RUSTC, os.path.join(directory, "fake-bin", "cargo"): SELFTEST_CARGO,
                os.path.join(directory, "tools", "unit-graph"): SELFTEST_UNIT_GRAPH,
                os.path.join(package, "target", "release", "fake-device"): SELFTEST_DEVICE}
    for path, text in programs.items():
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(f"#!{sys.executable}\n{text}")
        os.chmod(path, 0o755)
    environment = {name: value for name, value in os.environ.items() if not name.startswith(("CARGO", "RUST", "FAKE_"))}
    environment.update({"PATH": os.path.join(directory, "fake-bin") + os.pathsep + os.environ.get("PATH", ""),
                        "CARGO_HOME": os.path.join(directory, "cargo-home"), UNIT_GRAPH_VARIABLE: os.path.join(directory, "tools", "unit-graph"),
                        "FAKE_CARGO_LOG": os.path.join(directory, "cargo.log"), "FAKE_DEVICE_LOG": os.path.join(directory, "device.log"),
                        "FAKE_DEVICE_LIBRARY": os.path.join(directory, "library"), SELFTEST_RUN_VARIABLE: "1"})
    return environment


def selftest_call(directory, environment, arguments):
    """在小树上起这份脚本自己。→ (退出码, 标准输出, 标准错误)。"""
    command = [sys.executable, os.path.realpath(__file__)] + arguments[:1] + [os.path.join(directory, "tree")] + arguments[1:]
    completed = subprocess.run(command, capture_output=True, text=True, env=environment)
    return completed.returncode, completed.stdout, completed.stderr


KEY_ARGUMENTS = ["--binary", "fake-device", "--entry", "fake::entry_function", "--threads", "1"]


def selftest_key(directory, environment, edit=None):
    """取一次代码键；edit = (相对 scripts/experiments 的文件, 原文, 替换文)，原文要恰好命中一次。→ (代码键或 None, 说明)。"""
    if edit is not None:
        path = os.path.join(directory, "tree", PACKAGE_DIRECTORY, edit[0])
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        if text.count(edit[1]) != 1:
            return None, f"{edit[0]} 里 {edit[1]!r} 命中 {text.count(edit[1])} 次"
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text.replace(edit[1], edit[2]))
    code, stdout, stderr = selftest_call(directory, environment, ["key"] + KEY_ARGUMENTS)
    keys = [line.split("\t")[2] for line in stdout.split("\n") if line.startswith("code_key\t")]
    return (keys[0] if code == 0 and len(keys) == 1 else None), f"退 {code}，{stderr.strip()[:300]}"


def key_cell(edit, should_change):
    """改一处之前、之后各取一次代码键：should_change 为真要不等，为假要相等。"""
    with tempfile.TemporaryDirectory(prefix="device-code-key-selftest-") as directory:
        environment = selftest_tree(directory)
        before, before_detail = selftest_key(directory, environment)
        after, after_detail = selftest_key(directory, environment, edit)
        if before is None or after is None:
            return False, f"键没取出来：改前 {before_detail}；改后 {after_detail}"
        return (before != after) == should_change, f"改前 {before[:16]}，改后 {after[:16]}"


def run_cell(arguments, extra_environment, check):
    """在小树上跑一次 run；check(退出码, 标准错误, 装置日志, 库目录) → (过不过, 说明)。"""
    with tempfile.TemporaryDirectory(prefix="device-code-key-selftest-") as directory:
        environment = selftest_tree(directory)
        environment.update({name: value.replace("<tree>", os.path.join(directory, "tree")) for name, value in extra_environment.items()})
        code, _stdout, stderr = selftest_call(directory, environment, ["run"] + arguments)
        log_path = environment["FAKE_DEVICE_LOG"]
        device_log = open(log_path, encoding="utf-8").read() if os.path.isfile(log_path) else ""
        return check(code, stderr, device_log, environment["FAKE_DEVICE_LIBRARY"])


def device_binary_seen(library):
    """假装置看到的 DEVICE_CODE_KEY_DEVICE_BINARY 与它自己的路径（小树里编好的那一份）：→ (相等不相等, 看到的)。"""
    path = os.path.join(library, "device-binary.txt")
    seen = open(path, encoding="utf-8").read() if os.path.isfile(path) else "假装置没起"
    expected = os.path.join(os.path.dirname(library), "tree", PACKAGE_DIRECTORY, "target", "release", "fake-device")
    return seen == expected, seen


def check_refused(code, stderr, device_log, library):
    calls = [line.split("\t") for line in device_log.split("\n") if line]
    binary_passed, binary_seen = device_binary_seen(library)
    passed = code == EXIT_REFUSED and calls == [["segment-four", "--no-reuse"]] and "键算不出" in stderr and binary_passed
    return passed, f"退 {code}，装置被起成 {calls}，装置看到的 {DEVICE_BINARY_VARIABLE}={binary_seen}"


def check_kept(code, stderr, device_log, library):
    composition = open(os.path.join(library, "composition.txt"), encoding="utf-8").read() if os.path.isfile(os.path.join(library, "composition.txt")) else ""
    binary_passed, binary_seen = device_binary_seen(library)
    passed = code == 0 and os.path.isfile(os.path.join(library, "item.out")) and "--code-key-file" in device_log \
        and "code_key\tfake::entry_function\t" in composition and FORM_LINE in composition and binary_passed
    return passed, (f"退 {code}，项文件{'在' if os.path.isfile(os.path.join(library, 'item.out')) else '不在'}，"
                    f"装置看到的 {DEVICE_BINARY_VARIABLE}={binary_seen}，{stderr.strip()[-200:]}")


def check_device_exit(code, _stderr, _device_log, _library):
    return code == 4, f"退 {code}"


def check_removed(code, stderr, _device_log, library):
    passed = code == EXIT_FAILED and not os.path.exists(os.path.join(library, "item.out")) and "源码在编译与跑之间变了" in stderr
    return passed, f"退 {code}，项文件{'还在' if os.path.exists(os.path.join(library, 'item.out')) else '删了'}，{stderr.strip()[-200:]}"


def selftest():
    """每种判定至少一格；弄坏开关打开时（DEVICE_CODE_KEY_BREAK，文件头逐个列了红在哪一格）照样跑，红在点名的那一格。"""
    cells = [
        ("只加一行注释键不变", lambda: key_cell(("device.rs", "    helper_function();\n", "    // 注释\n    helper_function();\n"), False)),
        ("改入口调到的函数键变", lambda: key_cell(("device.rs", "compute(1)", "compute(2)"), True)),
        ("加一个没人调的函数键不变", lambda: key_cell(("device.rs", "fn helper_function() {", "fn unrelated_function() {\n}\n\nfn helper_function() {"), False)),
        ("profile.release 改一个值键变", lambda: key_cell(("Cargo.toml", "opt-level = 3", "opt-level = 2"), True)),
        ("闭包点到的包在 Cargo.lock 里抬一版键变", lambda: key_cell(("Cargo.lock", 'version = "1.0.0"', 'version = "1.0.1"'), True)),
        ("闭包没点到的包抬一版键不变", lambda: key_cell(("Cargo.lock", 'version = "2.0.0"', 'version = "2.0.1"'), False)),
        ("拒算：退 3、装置被起成 --no-reuse、不给代码键", lambda: run_cell(KEY_ARGUMENTS + ["--when-refused=--no-reuse", "--", "segment-four"],
                                                                {"FAKE_UNIT_GRAPH_REFUSE": "1"}, check_refused)),
        ("unit-graph 找不到：退 78", lambda: (lambda result: (result[0] == EXIT_UNIT_GRAPH_MISSING, f"退 {result[0]}"))(
            run_missing_unit_graph())),
        ("前后两次键相等：项文件留着、交装置的退出码", lambda: run_cell(KEY_ARGUMENTS + ["--", "segment-four"], {}, check_kept)),
        ("装置退 4：这里也退 4", lambda: run_cell(KEY_ARGUMENTS + ["--", "segment-four"], {"FAKE_DEVICE_EXIT": "4"}, check_device_exit)),
        ("编译与跑之间改了入口调到的函数：删掉这一趟写的项文件、退 1",
         lambda: run_cell(KEY_ARGUMENTS + ["--", "segment-four"], {"FAKE_DEVICE_EDIT_SOURCE": f"<tree>/{PACKAGE_DIRECTORY}/device.rs"}, check_removed)),
        ("工具链行带一行 host: <三元组>，closure 与 key 两趟都给（unit-graph 缺它退 2）", host_line_cell),
        ("reuse-three-questions", three_questions_cell),
        ("用法错：没给 --entry 退 2", lambda: run_usage_error(["key", "--binary", "fake-device"])),
        ("用法错：run 没写 -- 退 2", lambda: run_usage_error(["run"] + KEY_ARGUMENTS)),
    ]
    failures = 0
    for name, cell in cells:
        try:
            passed, detail = cell()
        except Exception as error:  # 自证里一格炸了记红、接着跑别的格
            passed, detail = False, f"{type(error).__name__}：{error}"
        failures += 0 if passed else 1
        print(f"  {'✓' if passed else '✗'} {name}" + ("" if passed else f"：{detail}"))
    print(f"device-code-key 自证：{len(cells)} 格，红 {failures} 格")
    return 1 if failures else 0


SELFTEST_TWO_ENTRY_SOURCE = """fn entry_function() {
    helper_function();
}

fn helper_function() {
    let value = fake_external::compute(1);
}

fn other_entry() {
    other_helper();
}

fn other_helper() {
    let value = 2;
}

fn unused_function() {
    let value = 3;
}
"""


def three_questions_cell():
    """复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）：装置按代码键存判定（库在装置里），这里照装置的存法在内存里记「代码键 → 判过」，
    两个入口（entry_function、other_entry）各一项：头一趟、同一份输入、改没人调的 unused_function（闭包外）、改只有 other_entry 调到的 other_helper（闭包内）各取一趟键，
    键在库里就是复用、不在就现判并记下；打规范输出。已有同义的格在这里点名：「只加一行注释键不变」「加一个没人调的函数键不变」是 ②，「改入口调到的函数键变」是 ③。"""
    reuse_checks = load_reuse_checks()
    entries = ["fake::entry_function", "fake::other_entry"]
    with tempfile.TemporaryDirectory(prefix="device-code-key-selftest-") as directory:
        environment = selftest_tree(directory)
        source = os.path.join(directory, "tree", PACKAGE_DIRECTORY, "device.rs")
        with open(source, "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_TWO_ENTRY_SOURCE)
        library, runs, problems = set(), [], []
        for edit in (None, None, ("fn unused_function() {\n    let value = 3;\n}", "fn unused_function() {\n    let value = 4;\n}"),
                     ("    let value = 2;\n", "    let value = 5;\n")):
            if edit is not None:
                with open(source, encoding="utf-8") as handle:
                    text = handle.read()
                if text.count(edit[0]) != 1:
                    return False, f"device.rs 里 {edit[0]!r} 命中 {text.count(edit[0])} 次"
                with open(source, "w", encoding="utf-8") as handle:
                    handle.write(text.replace(edit[0], edit[1]))
            arguments = ["key", "--binary", "fake-device", "--threads", "1"] + [word for entry in entries for word in ("--entry", entry)]
            code, stdout, stderr = selftest_call(directory, environment, arguments)
            keys = {line.split("\t")[1]: line.split("\t")[2] for line in stdout.split("\n") if line.startswith("code_key\t")}
            if code != 0 or sorted(keys) != entries:
                return False, f"取键退 {code}、交了 {sorted(keys)}：{stderr.strip()[:300]}"
            runs.append({entry: reuse_checks.REUSED if keys[entry] in library else reuse_checks.JUDGED for entry in entries})
            library |= set(keys.values())
        passed, problems = reuse_checks.report_three_questions(SCRIPT_KEY, *runs, ["fake::other_entry"])
        return passed, "；".join(problems)


def load_reuse_checks():
    """scripts/gate/lib_reuse_checks.py（复用三问的判法与规范输出全仓一份）。"""
    import lib_reuse_checks
    return lib_reuse_checks


def host_line_cell():
    """取一次键：退 0、组成清单里有 toolchain<TAB>host: <假 rustc -vV 的三元组> 那一行（假 unit-graph 照真的缺它退 2）。"""
    with tempfile.TemporaryDirectory(prefix="device-code-key-selftest-") as directory:
        environment = selftest_tree(directory)
        code, stdout, stderr = selftest_call(directory, environment, ["key"] + KEY_ARGUMENTS)
        passed = code == 0 and "toolchain\thost: x86_64-unknown-linux-gnu" in stdout.split("\n")
        return passed, f"退 {code}，{stderr.strip()[:300]}"


def run_missing_unit_graph():
    with tempfile.TemporaryDirectory(prefix="device-code-key-selftest-") as directory:
        environment = selftest_tree(directory)
        environment[UNIT_GRAPH_VARIABLE] = os.path.join(directory, "tools", "not-there")
        return selftest_call(directory, environment, ["key"] + KEY_ARGUMENTS)


def run_usage_error(arguments):
    with tempfile.TemporaryDirectory(prefix="device-code-key-selftest-") as directory:
        environment = selftest_tree(directory)
        code, _stdout, stderr = selftest_call(directory, environment, arguments)
        return code == EXIT_USAGE, f"退 {code}，{stderr.strip()[:200]}"


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
