"""形式化证明判法（入口 verus-proof-check.py）的一块：常量与弄坏开关集合 BREAK / FORMAL_BREAK、Red / NotRun 与报告函数、登记表 verus-pins.tsv 的读法、起 Verus 与解析它的输出、产品 rustc 与信任清单的读取、词法小工具。
入口 verus-proof-check.py 把这一块的名字再导出。
"""

import glob
import importlib.util
import json
import os
import re
import shutil
import signal
import subprocess
import threading
import tomllib
from lib_function_units import rust_lex  # noqa: E402

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))


REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
REGISTRY = ".claude/gate.d/verus-pins.tsv"
# 复用三问的规范输出与复用哨兵认的脚本名（scripts/gate/lib_reuse_checks.py 文件头「一」「二」）
SCRIPT_KEY = "scripts/gate/verus-proof-check.py"
EXIT_GREEN, EXIT_RED, EXIT_USAGE, EXIT_NOT_RUN = 0, 1, 2, 77
# 登记表没有 rlimit 行时 Verus 命令行上写的数：Verus 0.2026.09.20 自己的默认值（登记表 rlimit 行的依据写在那一行上面的注释里）
DEFAULT_RLIMIT = 10
RLIMIT_EXHAUSTED = "Resource limit (rlimit) exceeded"
# 必红世界表第五段的唯一形态：proof_errors=<证明错条数>（proof_error_messages 数的那些，不含 rlimit 用尽）
EXPECTED_PROOF_ERRORS = re.compile(r"proof_errors=([1-9][0-9]*)")
EXPECTATION_HOWTO = ("必红世界表第五段一律写 proof_errors=M：M 是改坏的副本在登记的 rlimit 上 Verus 报的证明错条数"
                     "（error 起头的报错，除去含 Resource limit (rlimit) exceeded 的、以 aborting due to 起头的与 cargo 那一种自己打的 could not compile），至少 1")
# task-server 起任务时套的内存包装给命令设的外层上限（run-with-memory-cap.sh 文件头「套在包装里」）：几处同时验不按它收（parallel_locations），
# 自证里那一格造它、别的格开跑前从环境里清掉
OUTER_MEMORY_CAP_VARIABLE = "RUN_WITH_MEMORY_CAP_OUTER_CAP"
# Verus 被信号停了（直接起时负的信号号，经 bash 起时 128 + 信号号）那一处的出路：是外面停的，不写修证明（killed_signal）
KILLED_HOWTO = ("这一次是外面停了 Verus：查是谁在那一刻停了它（同一时刻的 ps、journalctl --user、看门狗的记录），那件事不再发生之后重跑这一格；"
                "整条命令撞了内存上限由 task-server 放大重排、不在这里判；不改证明、核或必红世界表")
# tool-versions 除 verus 外另哈希的安装文件：verus 只是起 rust_verify 的外壳，判定还取决于 rust_verify、z3 与 vstd
TOOL_VERSION_FILES = ("rust_verify", "z3", "vstd.vir", "libvstd.rlib")
# Verus 的 launcher 写死 rustup run <version.json 的 toolchain> 起 rust_verify；rust_verify 从那条工具链的 lib/ 读 librustc_driver 与 libLLVM，
# 从 lib/rustlib/<三元组>/lib/ 读 std、core 这些 crate 的 rlib 与 rmeta（rustup_toolchain_fields）
RUSTUP_DEFAULT_HOME = os.path.join(os.path.expanduser("~"), ".rustup")
EXCLUDED_PACKAGE_FILES = {"Cargo.toml", "Cargo.toml.orig", ".cargo_vcs_info.json", ".cargo-ok"}
# 核里一律不许的标记（不带 --no-cheating 之后这几样 Verus 不拒，由这里判）：admit(…)、assume(…)、verifier::external（让整个条目对 Verus 不可见）。
# assume_specification、external_body、uninterp 这几样不在这里：照信任清单逐条登记（TRUST_KINDS）；axiom 另判（region_problems，按词法单元认）
CHEATING_MARKER = re.compile(r"\badmit\s*\(|\bassume\s*\(|verifier\s*::\s*external\b|verifier\s*\(\s*external\s*\)")
# 信任清单：被验区域里每一处可信声明一行，vstd 按版本整体一行；格式写在表自己的文件头
TRUST_LIST = ".claude/gate.d/verus-trust-list.tsv"
TRUST_KINDS = ("assume_specification", "external_body", "uninterp", "external_fn_specification", "external_trait_specification",
               "external_type_specification", "external_trait_extension")
CFG_KIND = "cfg"
TRUST_ROW_HOWTO = (f"照 {TRUST_LIST} 文件头写一行（制表符分隔）：种类、文件、点名的条目、规格原文的 sha256、一条差分测试的测试名、理由；"
                   "红句里给了这一处现算的 sha256 与整行的样子，测试名写那条把这份规格与真实现逐值比的 #[test]（没有就先写它）")
# verus! 外面的 trait impl 的登记式放过表（只缩不涨；格式写在表自己的文件头）：与 gate 基准那一版比、切内容行，都用滞后表的共用读法 lib-lag-baseline.py
TRAIT_IMPLS_OUTSIDE_LAG = ".claude/gate.d/verus-trait-impls-outside-lag.tsv"
LAG_BASELINE_LIBRARY = os.path.join(REPOSITORY_ROOT, ".claude", "gate.d", "lib-lag-baseline.py")
TRAIT_IMPL_OUTSIDE_HOWTO = ("把这个 impl 挪进 verus! 里（方法照 trait 的 requires / ensures 证出来）；确是不该被验的类型，改名或挪出被验区域。"
                            f"{TRAIT_IMPLS_OUTSIDE_LAG} 只缩不涨：gate 基准那一版里还没有这张表时照它的文件头初次登记，之后只删不加，新写的 impl 不往里登记")
# 函数（或模块）上的 rlimit 属性：#[verifier::rlimit(N)] 与 #[verifier(rlimit(N))] 都盖过命令行的 --rlimit（Verus 0.2026.09.20 两种写法都认、--no-cheating 不拒）
RLIMIT_ATTRIBUTE = re.compile(r"verifier\s*::\s*rlimit\b|verifier\s*\(\s*rlimit\b")
# 一行 fn 的开头（空真取插入点的说明用：requires 往上最近的一行）；const fn 也认（core 的 SlotNumber::to_device_offset 就是 const fn）
FUNCTION_LINE = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?(open\s+|closed\s+)?(proof\s+|spec\s+|exec\s+)?(?:const\s+)?fn\s+(\w+)")
# 必红世界表第六段（落点）的唯一形态：landing=<落点>[,<落点>…]；落点是证明错报在哪个函数里（一个 fn 名），或两个占位：
# （函数外）证明错的位置不在任何 fn 的行里，（无位置）输出里这一条证明错没带 --> 位置。条数会随求解环境漂，落点不会（C654）
LANDING_OUTSIDE, LANDING_NO_LOCATION = "（函数外）", "（无位置）"
LANDING_TOKEN = rf"(?:[A-Za-z_][A-Za-z0-9_]*|{LANDING_OUTSIDE}|{LANDING_NO_LOCATION})"
EXPECTED_LANDING = re.compile(rf"landing=({LANDING_TOKEN}(?:,{LANDING_TOKEN})*)")
LANDING_HOWTO = ("必红世界表第六段一律写 landing=<函数名>[,<函数名>…]：证明错报在哪个 fn 里（Verus 报错 --> 位置所在的函数），多个用逗号、顺序随意；"
                 "不是为了数条数：同一个 fn 里多报一条循环不变式、条数跳了，落点不变就不红")
# 一行 fn 的开头（verus! 里 spec、proof、exec、open、closed、broadcast 这些修饰都认）：证明错的位置往上找最近一行这样的，就是它落在哪个函数里
FUNCTION_HEADING = re.compile(r"^\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?(?:(?:open|closed|broadcast|proof|spec(?:\([a-z]+\))?|exec|axiom|const|unsafe)\s+)*fn\s+(\w+)")
DIAGNOSTIC_LOCATION = re.compile(r"^\s*-->\s+(\S+?):(\d+):(\d+)")
BREAK = {item for item in os.environ.get("VERUS_PROOF_CHECK_BREAK", "").split(",") if item}
if "const-fn-heading-unrecognized" in BREAK:   # 弄坏开关：FUNCTION_LINE 不认 const fn
    FUNCTION_LINE = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?(open\s+|closed\s+)?(proof\s+|spec\s+|exec\s+)?fn\s+(\w+)")
# cargo 那一种（cargo verus focus）编不过时 cargo 自己打的那一句「error: could not compile `<包>` (lib) due to N previous error(s)」：不是证明错，
# 与 rustc 的「aborting due to」一样不数、不进落点（proof_error_messages、diagnostic_blocks）
CARGO_GAVE_UP = "could not compile"
# 门禁 formal-verification 那一道的弄坏开关（文件头「被验区域与多 crate 的弄坏开关」一段）：判法这一半在这份脚本里，两份文件头各列一份
FORMAL_BREAK = {item for item in os.environ.get("FORMAL_VERIFICATION_BREAK", "").split(",") if item}
# 这份脚本读的那几项（自证的开关轮逐项打开；门禁那一道自己读的另几项不在这里）
FORMAL_BREAK_ITEMS = ("single-crate-registry", "cargo-mode-ignored", "methods-refused", "attribute-mode-unrecognized", "no-cheating-kept",
                      "trust-list-unchecked", "trust-digest-unchecked", "trust-test-unchecked", "trust-vstd-unchecked", "axiom-accepted",
                      "cfg-atom-accepted", "cfg-macro-unchecked", "trait-impl-outside-accepted", "mir-compare-skipped", "rustc-pin-unchecked",
                      "reuse-key-without-trust-list", "cargo-fingerprint-kept")
# 门禁给的一份文件：每格判完往里追加一行「格<TAB>现判数<TAB>复用数<TAB>原因分栏」（lib_verdict_library.Tally.append_to），门禁整道相加
ITEM_TALLY_FILE_VARIABLE = "FORMAL_ITEM_TALLY_FILE"
# 门禁给的另一份文件：每项判完（现判或复用）往里追加一行「项<TAB>这一项自己的挂钟秒数<TAB>现判条数<TAB>复用条数<TAB>退出码」，门禁收尾转成 ITEM_DONE 行
ITEM_DONE_FILE_VARIABLE = "FORMAL_ITEM_DONE_FILE"
# list-items --key-mismatches 往 stderr 逐项打的一行前缀：「ITEM_KEY item=<项名> state=<键相同|键对不上|无记录|拒算>」（门禁 --list-items --with-keys 转成 ITEM_PLAN 行）
ITEM_KEY_PREFIX = "ITEM_KEY"


class Red(Exception):
    """这一格判红：message 是卡在哪，howto 是下一步。"""

    def __init__(self, message, howto):
        super().__init__(message)
        self.message, self.howto = message, howto


class NotRun(Exception):
    """这一格本次未跑（核还没进仓）。"""


def report_red(message, howto):
    print(f"  ✗ {message}")
    print(f"     → 怎么办：{howto}")


def one_line(text):
    """自证一格判错的细节行（以 ✗ 起头、带格名）里转述了被测那一格的输出：换行换成「 | 」，整条留在这一行里，
    转述里以 ✗ 起头的那几行不另起一行、不被当成别的格（门禁 code-tooling 的 break-switch-replay 格按以 ✗ 起头的行认格）。"""
    rows = text.split("\n")
    return " | ".join([rows[0].rstrip(), *(row.strip() for row in rows[1:] if row.strip())])


def report_cell(problems):
    """一格的问题逐条打（(卡在哪, 下一步)）。"""
    for message, howto in problems:
        report_red(message, howto)


def strip_line_comment(line):
    position = line.find("//")
    return line if position < 0 else line[:position]


# ── 登记表 ──

class Registry:
    def __init__(self):
        self.tag = self.commit = self.zip_sha256 = self.install_relative = None
        self.packages = []
        self.crate_directory = None
        self.verified_count = None
        self.kernels = []
        self.must_red_table = None
        self.rlimit = None
        self.rustc = None            # rustc 行：(Verus 用的工具链, 产品 rustc 的版本, 产品 rustc 的 commit-hash)；只有 verus-toolchain-pinned 一格要它
        self.cargo_mode = False      # crate 行第四段写 cargo：经 cargo verus focus 验（可以有别的依赖），不写是直接起 verus（只依赖 vstd）
        self.crates = []             # 每个登记的 crate 一份（CRATE_FIELDS 换成那个 crate 的），只登记一个 crate 时就是 [自己]
        self.all_crate_directories = []


# 每个 crate 自己的几样（per_crate_view 换成那个 crate 的）；别的（release、package、rlimit、rustc）各 crate 共用
CRATE_FIELDS = ("crate_directory", "verified_count", "kernels", "must_red_table", "cargo_mode")
# 登记表各种行的段数；crate 行三段或四段（第四段只许写 cargo）
REGISTRY_WIDTHS = {"release": (5,), "package": (6,), "crate": (3, 4), "kernel": (2,), "must-red-worlds": (2,), "rlimit": (2,), "rustc": (4,)}


def per_crate_view(registry, crate):
    """登记的一个 crate 的那一份登记：共用的几样照抄，CRATE_FIELDS 换成这个 crate 的。"""
    view = Registry()
    view.__dict__.update(registry.__dict__)
    for name in CRATE_FIELDS:
        setattr(view, name, crate[name])
    view.crates = []
    return view


def crate_identifier(registry, manifest=None):
    """crate 的 Rust 名（Cargo.toml 的 package.name，- 换成 _）：项名里全路径的头一段，与 Verus 的 func-details 同一种写法。"""
    if manifest is None:
        try:
            manifest = tomllib.load(open(os.path.join(registry.root, registry.crate_directory, "Cargo.toml"), "rb"))
        except (OSError, tomllib.TOMLDecodeError, TypeError, AttributeError):
            manifest = {}
    return (manifest.get("package", {}).get("name") or os.path.basename(registry.crate_directory or "crate")).replace("-", "_")


def read_registry(root):
    path = os.path.join(root, REGISTRY)
    if not os.path.isfile(path):
        raise Red(f"登记表 {REGISTRY} 不在 {root} 里", "它随仓走；被删了从 git 找回，格式写在它自己的文件头")
    registry = Registry()
    registry.root = root
    problems = []
    crates, kernels, tables = [], [], []
    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        if not raw.strip() or raw.startswith("#"):
            continue
        fields = raw.split("\t")
        kind, widths = fields[0], REGISTRY_WIDTHS.get(fields[0])
        if widths is None or len(fields) not in widths:
            problems.append(f"第 {number} 行认不出（第一段 {kind!r}、{len(fields)} 段）")
            continue
        if kind == "release":
            if registry.tag is not None:
                problems.append(f"第 {number} 行：release 行不止一行")
            registry.tag, registry.commit, registry.zip_sha256, registry.install_relative = fields[1:]
            if not re.fullmatch(r"[0-9a-f]{40}", registry.commit) or not re.fullmatch(r"[0-9a-f]{64}", registry.zip_sha256):
                problems.append(f"第 {number} 行：commit 要 40 位、zip 的 sha256 要 64 位小写十六进制")
        elif kind == "package":
            name, version, sha1, path_in_vcs, fingerprint = fields[1:]
            if not re.fullmatch(r"[0-9a-f]{40}", sha1) or not re.fullmatch(r"[0-9a-f]{64}", fingerprint):
                problems.append(f"第 {number} 行：sha1 要 40 位、内容指纹要 64 位小写十六进制")
            registry.packages.append({"name": name, "version": version, "sha1": sha1, "path_in_vcs": path_in_vcs, "fingerprint": fingerprint})
        elif kind == "crate":
            directory = fields[1].rstrip("/")
            if any(crate["crate_directory"] == directory for crate in crates):
                problems.append(f"第 {number} 行：crate {directory} 登记了两次")
            if len(fields) == 4 and fields[3] != "cargo":
                problems.append(f"第 {number} 行：crate 行第四段只许写 cargo（经 cargo verus focus 验），读到 {fields[3]!r}")
            if not fields[2].isdigit() or int(fields[2]) == 0:
                problems.append(f"第 {number} 行：verified 的登记数要是正整数，读到 {fields[2]!r}")
            crates.append({"crate_directory": directory, "verified_count": int(fields[2]) if fields[2].isdigit() else None,
                           "kernels": [], "must_red_table": None,
                           # 弄坏开关 cargo-mode-ignored：第四段 cargo 不认，照直接起 verus 判（有别的依赖就红）
                           "cargo_mode": len(fields) == 4 and fields[3] == "cargo" and "cargo-mode-ignored" not in FORMAL_BREAK})
        elif kind == "kernel":
            kernels.append((number, fields[1]))
        elif kind == "rlimit":
            if registry.rlimit is not None:
                problems.append(f"第 {number} 行：rlimit 行不止一行")
            if not fields[1].isdigit() or int(fields[1]) == 0:
                problems.append(f"第 {number} 行：rlimit 要是正整数，读到 {fields[1]!r}")
            else:
                registry.rlimit = int(fields[1])
        elif kind == "rustc":
            if registry.rustc is not None:
                problems.append(f"第 {number} 行：rustc 行不止一行")
            if not all(fields[1:]) or not re.fullmatch(r"[0-9a-f]{40}", fields[3]):
                problems.append(f"第 {number} 行：rustc 行三段是 Verus 用的工具链、产品 rustc 的版本、产品 rustc 的 commit-hash（40 位小写十六进制）")
            registry.rustc = tuple(fields[1:])
        else:
            tables.append((number, fields[1]))
    if len(crates) > 1 and "single-crate-registry" in FORMAL_BREAK:
        crates = crates[:1]   # 弄坏开关：只认登记表里第一个 crate
    for number, kernel in kernels:
        owner = [crate for crate in crates if kernel.startswith(crate["crate_directory"] + "/")]
        if not owner:
            if "single-crate-registry" not in FORMAL_BREAK:
                problems.append(f"核 {kernel} 不在登记的 crate {'、'.join(crate['crate_directory'] for crate in crates) or '（一个都没有）'} 里")
            continue
        owner[0]["kernels"].append(kernel)
    for number, table in tables:
        owner = [crate for crate in crates if table.startswith(crate["crate_directory"] + "/")] if len(crates) > 1 else crates
        if not owner:
            if "single-crate-registry" not in FORMAL_BREAK:
                problems.append(f"第 {number} 行：必红世界表 {table} 不在登记的哪个 crate 的目录里（登记了几个 crate 时，每个 crate 的表放在它自己的目录里）")
            continue
        if owner[0]["must_red_table"] is not None:
            problems.append(f"第 {number} 行：crate {owner[0]['crate_directory']} 的 must-red-worlds 行不止一行")
        owner[0]["must_red_table"] = table
    if registry.tag is None or not crates or not registry.packages or any(not crate["kernels"] or crate["must_red_table"] is None for crate in crates):
        problems.append("release 恰好一行，crate 至少一行、每个 crate 至少一行 kernel、恰好一行 must-red-worlds，package 至少一行")
    if problems:
        raise Red(f"登记表 {REGISTRY} 写坏了：" + "；".join(problems), "照它文件头的格式改；一行一条、制表符分隔")
    if registry.rlimit is None:
        registry.rlimit = DEFAULT_RLIMIT
    registry.all_crate_directories = [crate["crate_directory"] for crate in crates]
    if len(crates) == 1:
        for name in CRATE_FIELDS:
            setattr(registry, name, crates[0][name])
        registry.crates = [registry]
    else:
        registry.crates = [per_crate_view(registry, crate) for crate in crates]
    return registry


def kernel_files(root, registry):
    """登记的核都在就交回它们；核没进仓 NotRun；核不在而 crate 已经接了 Verus 判红。"""
    missing = [kernel for kernel in registry.kernels if not os.path.isfile(os.path.join(root, kernel))]
    if not missing:
        # 不依赖登记数：新增模块的契约即使未验证，也能被已登记函数使用。
        registered = {os.path.realpath(os.path.join(root, path)) for path in registry.kernels}
        for directory, _subdirectories, filenames in os.walk(os.path.join(root, registry.crate_directory, "src")):
            for filename in filenames:
                if not filename.endswith(".rs"):
                    continue
                path = os.path.join(directory, filename)
                _kinds, words, _lines, _docs = rust_lex(open(path, encoding="utf-8").read())
                # verus! 宏包的，与属性写法（#[verus_verify]、#[verus_spec(…)]）标的，都是被验区域
                attribute_mode = "attribute-mode-unrecognized" not in FORMAL_BREAK and any(word in ("verus_verify", "verus_spec") for word in words)
                if (any(words[index:index + 2] == ["verus", "!"] for index in range(len(words) - 1)) or attribute_mode) and os.path.realpath(path) not in registered:
                    raise Red(f"未登记的 Verus 核：{os.path.relpath(path, root)}" + ("（属性写法 #[verus_verify] / #[verus_spec]）" if attribute_mode else ""),
                              f"把该文件加入 {REGISTRY} 的 kernel 行；所有被使用的证明契约都必须验证")
        return [os.path.join(root, kernel) for kernel in registry.kernels]
    crate = os.path.join(root, registry.crate_directory)
    evidence = []
    manifest = os.path.join(crate, "Cargo.toml")
    if os.path.isfile(manifest) and re.search(r"^\s*vstd\s*=", open(manifest, encoding="utf-8").read(), re.M):
        evidence.append(f"{registry.crate_directory}/Cargo.toml 依赖 vstd")
    for directory, _subdirectories, names in os.walk(os.path.join(crate, "src")):
        for name in sorted(names):
            if name.endswith(".rs") and "verus!" in open(os.path.join(directory, name), encoding="utf-8", errors="replace").read():
                evidence.append(os.path.relpath(os.path.join(directory, name), root) + " 里有 verus!")
    if (not evidence and len(missing) == len(registry.kernels)) or "absent-kernel-not-run" in BREAK:
        raise NotRun(f"登记的核 {' '.join(missing)} 还没进仓（crate 不依赖 vstd、src 里没有 verus!）：Verus 这一格本次无对象可判")
    raise Red(f"登记的核 {' '.join(missing)} 不在，而 crate 已经接了 Verus（{'；'.join(evidence) or '另一份登记的核在'}）",
              f"核被删了或改了名：找回它，或者照新路径改 {REGISTRY} 的 kernel 行；核的删除不许让这一格变成本次未跑")


# ── Verus 安装、crate、外部依赖 ──

# 登记表 release 行第五列相对的是数据根的上一层：起头的 singlefs/ 是数据根本身，再下一层 formal-tools 是数据根下的共享名（lib_work_root.py 的 SHARED_DATA_ENTRIES）
INSTALL_SHARED_ENTRY = "formal-tools"
INSTALL_PREFIX = "singlefs/" + INSTALL_SHARED_ENTRY + "/"


def work_root_library():
    """.claude/hooks/lib_work_root.py（数据根只在那一份里定义；按文件路径导入，与 scripts/kb/pdf-text.py 同法）。读不到判红。"""
    path = os.path.join(REPOSITORY_ROOT, ".claude", "hooks", "lib_work_root.py")
    try:
        specification = importlib.util.spec_from_file_location("lib_work_root_for_verus_proof_check", path)
        library = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(library)
    except (OSError, ImportError, SyntaxError, AttributeError) as error:
        raise Red(f"读不到 {path}（{error}），取不到数据根", "在带着 .claude/hooks/lib_work_root.py 的仓里跑，或用 --verus-root、SINGLEFS_VERUS_ROOT 指到 Verus 的安装目录") from error
    return library


def verus_directory(registry, override):
    """Verus 安装目录：--verus-root，再 SINGLEFS_VERUS_ROOT，再共享数据根下的登记位置（formal-tools 在共享名单里，两台同一个绝对路径读同一份）。
    不经 ~ 与 XDG_DATA_HOME：第二台的家目录是另一个。登记的第五列是绝对路径就是它自己（门禁样本把它改成替身安装）。
    弄坏开关 default-install-by-home 下取 ${XDG_DATA_HOME:-~/.local/share}/<第五列>。"""
    if override:
        return os.path.abspath(override)
    if os.environ.get("SINGLEFS_VERUS_ROOT"):
        return os.path.abspath(os.environ["SINGLEFS_VERUS_ROOT"])
    if "default-install-by-home" in BREAK:
        data_home = os.environ.get("XDG_DATA_HOME") or os.path.join(os.path.expanduser("~"), ".local", "share")
        return os.path.join(data_home, registry.install_relative)
    if os.path.isabs(registry.install_relative):
        return registry.install_relative
    if not registry.install_relative.startswith(INSTALL_PREFIX):
        raise Red(f"{REGISTRY} 的 release 行第五列 {registry.install_relative} 不以 {INSTALL_PREFIX} 起头，认不出它在共享数据根下的位置",
                  f"第五列写成 {INSTALL_PREFIX}<发布目录>，或写绝对路径，或用 --verus-root、SINGLEFS_VERUS_ROOT 指到安装目录")
    library = work_root_library()
    try:
        shared_directory = library.shared_data_path(INSTALL_SHARED_ENTRY)
    except ValueError as error:
        raise Red(f"取不到数据根下的 {INSTALL_SHARED_ENTRY}：{error}",
                  f"{getattr(library, 'DATA_ROOT_HOWTO', '配好数据根')}；或用 --verus-root、SINGLEFS_VERUS_ROOT 指到 Verus 的安装目录") from error
    return os.path.join(shared_directory, registry.install_relative[len(INSTALL_PREFIX):])


def read_version_json(directory):
    path = os.path.join(directory, "version.json")
    if not os.path.isfile(path):
        return None
    try:
        return json.load(open(path, encoding="utf-8")).get("verus", {})
    except (ValueError, OSError):
        return None


def crate_manifest(root, registry):
    path = os.path.join(root, registry.crate_directory, "Cargo.toml")
    if not os.path.isfile(path):
        raise Red(f"{registry.crate_directory}/Cargo.toml 不在", f"crate 目录改了就照新路径改 {REGISTRY} 的 crate 行")
    try:
        return tomllib.load(open(path, "rb"))
    except tomllib.TOMLDecodeError as error:
        raise Red(f"{registry.crate_directory}/Cargo.toml 读不出：{error}", "先让 cargo build 认得这份清单")


def check_dependencies(registry, manifest):
    """直接起的 Verus 只带它自带的 vstd：被验的 crate 除 vstd 之外还有依赖就编不过，开跑之前判红、说清下一步。"""
    if registry.cargo_mode:
        return   # 经 cargo verus focus 验：依赖照 Cargo.toml 由 cargo 编（copy_crate 连路径依赖一起拷）
    others = sorted(name for name in manifest.get("dependencies", {}) if name != "vstd")
    if others:
        raise Red(f"{registry.crate_directory}/Cargo.toml 除 vstd 之外还依赖 {' '.join(others)}：直接起的 Verus 只带它自带的 vstd，编不了别的 crate",
                  f"确要别的依赖（singlefs-core 这类），把 {REGISTRY} 里这个 crate 那一行写成四段、第四段 cargo：经 cargo verus focus 验，路径依赖连同 crate 一起拷进副本；"
                  "只依赖 vstd 的核照旧三段、直接起 verus")


def path_dependencies(root, crate_directory, seen=None):
    """crate（相对仓根的目录）与它经 Cargo.toml 的 path 依赖（dependencies、build-dependencies，一层一层往下）连到的 crate 目录，相对仓根、排好序。
    dev-dependencies 不收（cargo verus focus 不编测试）。路径出了仓根、或那份 Cargo.toml 不在的判红。"""
    seen = set() if seen is None else seen
    if crate_directory in seen:
        return seen
    seen.add(crate_directory)
    manifest_path = os.path.join(root, crate_directory, "Cargo.toml")
    try:
        manifest = tomllib.load(open(manifest_path, "rb"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise Red(f"{crate_directory}/Cargo.toml 读不出：{error}", "先让 cargo build 认得这份清单")
    for section in ("dependencies", "build-dependencies"):
        for name, specification in sorted(manifest.get(section, {}).items()):
            if not isinstance(specification, dict) or "path" not in specification:
                continue
            target = os.path.normpath(os.path.join(crate_directory, specification["path"]))
            if target.startswith(os.pardir) or os.path.isabs(target) or not os.path.isfile(os.path.join(root, target, "Cargo.toml")):
                raise Red(f"{crate_directory}/Cargo.toml 的路径依赖 {name} = {specification['path']!r} 不在仓里或没有 Cargo.toml",
                          "被验的 crate 的路径依赖要都在仓里（cargo verus focus 在副本里连它们一起编）")
            path_dependencies(root, target.replace(os.sep, "/"), seen)
    return seen


def copy_crate(root, registry, work):
    """一处判定一份副本：直接起 verus 的拷 crate 自己，交 crate 副本的目录（verus 在那里起）；cargo 那一种另拷它的路径依赖、仓根的 Cargo.lock，
    在副本根写一份只列这几个成员的 Cargo.toml（整个工作区的 cargo metadata 在离线机器上过不了），交副本根（cargo verus focus 在那里起）。
    每处判定都在一份新目录里：cargo 的编译指纹不看 --verify-* 参数，同一个目录里换参数再验拿到的是上一趟的结果、没有判定行。"""
    if not registry.cargo_mode:
        source = os.path.join(root, registry.crate_directory)
        destination = os.path.join(work, registry.crate_directory)
        shutil.copytree(source, destination, ignore=shutil.ignore_patterns("target"), symlinks=True)
        return destination
    members = sorted(path_dependencies(root, registry.crate_directory))
    for member in members:
        shutil.copytree(os.path.join(root, member), os.path.join(work, member), ignore=shutil.ignore_patterns("target"), symlinks=True)
    with open(os.path.join(work, "Cargo.toml"), "w", encoding="utf-8") as handle:
        handle.write("# verus-proof-check.py 拷的副本：只列被验的 crate 与它的路径依赖\n[workspace]\nresolver = \"2\"\nmembers = [\n"
                     + "".join(f"    {json.dumps(member)},\n" for member in members) + "]\n")
    if os.path.isfile(os.path.join(root, "Cargo.lock")):
        shutil.copy2(os.path.join(root, "Cargo.lock"), os.path.join(work, "Cargo.lock"))
    return work


RUST_IDENTIFIER = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def kernel_module(registry, manifest, relative):
    """核 relative（相对仓根）所在的模块：是 crate 的入口文件（Cargo.toml 的 lib.path，默认 src/lib.rs）交 None（点 --verify-root）；
    入口所在目录下的 <名>.rs 与 <名>/mod.rs 交 <名>，更深的用 :: 接起来（Verus 0.2026.09.20 的 --verify-only-module 认 outer::inner，文件头「只验一个函数」一段）。
    不在入口所在目录下、或有一段不是 Rust 标识符的判红。弄坏开关 submodule-verify-root 下一律交 None。"""
    if "submodule-verify-root" in BREAK:
        return None
    entry = os.path.normpath(manifest.get("lib", {}).get("path", "src/lib.rs"))
    crate_relative = os.path.normpath(os.path.relpath(relative, registry.crate_directory))
    if crate_relative == entry:
        return None
    rest = os.path.relpath(crate_relative, os.path.dirname(entry) or ".")
    parts = rest[:-len(".rs")].split(os.sep) if rest.endswith(".rs") and not rest.startswith(os.pardir + os.sep) else []
    if parts and parts[-1] == "mod":
        parts = parts[:-1]
    if not parts or not all(RUST_IDENTIFIER.fullmatch(part) for part in parts):
        raise Red(f"核 {relative} 认不出是 {registry.crate_directory} 的哪个模块：要是入口 {entry}，"
                  f"或入口所在目录 {os.path.dirname(entry) or '.'} 下的 <名>.rs、<名>/mod.rs（更深的照样一层一层往下）",
                  f"把核挪到入口所在目录下、照文件路径用 mod 声明接上，并照新路径改 {REGISTRY} 的 kernel 行；确要别的写法（#[path]、内联 mod），"
                  "先给 scripts/gate/verus-proof-check.py 的 kernel_module 补上认法")
    return "::".join(parts)


def verus_arguments(registry, manifest, threads, function=None, module=None):
    """Verus 的命令行；function 给了就只验这一个函数（证明逐函数一项、空真只验它所属的那一个函数）：module 是 None（核是 crate 的入口）点 --verify-root，
    不是 None 点 --verify-only-module <module>（kernel_module），再 --verify-function <名>。--verify-function 只在点的那一个模块里找；
    Verus 0.2026.09.20 拒 --verify-module 与 --verify-function 一起给、拒 --verify-root 与 --verify-only-module 一起给（文件头「只验一个函数」一段）。
    Verus 0.2026.09.20 按子串点名、有恰好同名的就取它（名字是别人子串的四个都退 0、没报 more than one match），
    所以 verus! 里的 fn 同名判红（verus_functions），名字是别人子串的照常用；点名在表里是不是恰好命中一项，起 Verus 之前由 point_name_problems 判。"""
    package = manifest.get("package", {})
    entry = manifest.get("lib", {}).get("path", "src/lib.rs")
    # 不带 --no-cheating：assume_specification、external_body、uninterp 这几样照信任清单逐条登记（trust_list_problems），
    # admit、assume(、axiom、verifier::external 在被验区域里一律判红（region_problems），都在起 Verus 之前判
    arguments = [] if registry.cargo_mode else ["--crate-type=lib", f"--edition={package.get('edition', '2021')}", "--crate-name", package.get("name", "crate").replace("-", "_")]
    arguments += ["--num-threads", str(threads), "--output-json"]
    if "rlimit-unpinned" not in BREAK:
        arguments += ["--rlimit", str(registry.rlimit)]
    if "no-cheating-kept" in FORMAL_BREAK:
        arguments.insert(0, "--no-cheating")   # 弄坏开关：照旧带 --no-cheating（登记了的 assume_specification 也被 Verus 拒）
    if function:
        arguments += ["--verify-root"] if module is None else ["--verify-only-module", module]
        arguments += ["--verify-function", function]
    return arguments if registry.cargo_mode else arguments + [entry]


def forget_crate_fingerprints(target, package):
    """cargo 那一种几处同时验共用一个编译目录：起之前删掉被验的 crate 在这个编译目录里的指纹目录（<编译目录>/**/.fingerprint/<包>-*），让它每处都现编、出判定行；
    cargo 的编译指纹不看 --verify-* 参数，换一份副本、改了文件时间也照样当它没变。依赖的指纹不动、照旧复用。
    弄坏开关 FORMAL_VERIFICATION_BREAK=cargo-fingerprint-kept 下不删（第二处起拿不到判定行）。"""
    if "cargo-fingerprint-kept" in FORMAL_BREAK:
        return
    for path in glob.glob(os.path.join(target, "**", ".fingerprint", f"{package}-*"), recursive=True):
        shutil.rmtree(path, ignore_errors=True)


def verus_program(registry, executable):
    """起 Verus 的程序：直接起的是 verus（在 crate 副本里起）；cargo 那一种是 Verus 安装里 verus 旁边的 cargo-verus（在副本根起）。"""
    if not registry.cargo_mode:
        return executable
    return os.path.join(os.path.dirname(executable), "cargo-verus")


def verus_program_arguments(registry, arguments, build=False):
    """交给 verus_program 的参数：直接起的照原样；cargo 那一种是 verus focus -p <包> --offline -- <参数…>（build 时是 verus build，MIR 比对用）。"""
    if not registry.cargo_mode:
        return list(arguments)
    return ["verus", "build" if build else "focus", "-p", crate_package_name(registry), "--offline", "--", *arguments]


def crate_package_name(registry):
    try:
        manifest = tomllib.load(open(os.path.join(registry.root, registry.crate_directory, "Cargo.toml"), "rb"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise Red(f"{registry.crate_directory}/Cargo.toml 读不出：{error}", "先让 cargo build 认得这份清单")
    return manifest.get("package", {}).get("name") or os.path.basename(registry.crate_directory)


def diagnostic_path(registry, relative):
    """Verus 报错 --> 后面的路径怎么写这份文件（relative 相对仓根）：直接起的相对 crate 目录（在 crate 副本里起），cargo 那一种相对副本根（就是相对仓根）。"""
    if registry.cargo_mode:
        return relative
    return os.path.relpath(relative, registry.crate_directory)


def located_file(registry, path):
    """Verus 报错 --> 后面的路径交回相对仓根的那一份（diagnostic_path 的反过来）。"""
    return os.path.normpath(path if registry.cargo_mode else f"{registry.crate_directory}/{path}")


def kernel_modules(root, registry, manifest, kernels):
    """每份核（相对仓根的路径）点哪个模块：{相对路径: kernel_module}。认不出的判红（开跑之前）。"""
    modules = {}
    for kernel in kernels:
        relative = os.path.relpath(kernel, root).replace(os.sep, "/")
        modules[relative] = kernel_module(registry, manifest, relative)
    return modules


def run_in_own_process_group(command, **popen_options):
    """起 command，放进它自己的进程组（start_new_session），stdout 与 stderr 接管道、两条线程边跑边收；它退了先停组里留下的
    （拿着管道的子进程不让读的一头等到 EOF；进程组号就是这里起的那一个的进程号，组里已经没有进程了不算错），再收齐输出，交 (process, stdout, stderr)。
    起与停写在同一个函数里：终止只发给这里自己起的那一组（.claude/hooks/bash-command-detector.sh 文件头 ⑥ 认的写法）。
    弄坏开关 group-leftovers-kept 下不停留下的（读的一头等到它们自己退）。popen_options 照交给 Popen（cwd、env）。"""
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, errors="replace", start_new_session=True,
                               **popen_options)
    collected = {}
    readers = [threading.Thread(target=lambda name=name, stream=stream: collected.__setitem__(name, stream.read()), daemon=True)
               for name, stream in (("stdout", process.stdout), ("stderr", process.stderr))]
    for reader in readers:
        reader.start()
    process.wait()
    if "group-leftovers-kept" not in BREAK:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except (ProcessLookupError, PermissionError):
            pass
    for reader in readers:
        reader.join()
    return process, collected.get("stdout", ""), collected.get("stderr", "")


def run_verus(program, arguments, crate_copy, extra_environment=None):
    """起一次 Verus（program 是 verus_program 交的、arguments 是 verus_program_arguments 交的；MIR 比对的产品构建交 cargo 与它的参数）；交回 {exit, json, diagnostics, stderr}。
    直接起：不套内存包装、capped.sh 与 nice，线程数在 Verus 自己的 --num-threads 上（verus_arguments），整条命令的内存额度由 task-server 给；不限时，卡住由 task-server 判。
    放进它自己的进程组，输出与收尾照 run_in_own_process_group。extra_environment 另设进环境（cargo 那一种的 CARGO_TARGET_DIR）。"""
    environment = dict(os.environ, **(extra_environment or {}))
    process, stdout, stderr = run_in_own_process_group([program, *arguments], cwd=crate_copy, env=environment)
    outcome = {"exit": process.returncode, "json": None, "diagnostics": stderr + "\n" + stdout, "stderr": stderr}
    try:
        outcome["json"] = json.loads(stdout) if stdout.strip().startswith("{") else None
    except ValueError:
        outcome["json"] = None
    return outcome


def diagnostic_blocks(text):
    """stderr 里的「error: <说什么>」与它下面第一处「--> 文件:行:列」配成一对。"""
    blocks, pending = [], None
    for line in text.split("\n"):
        if line.startswith("error"):
            pending = line.split(":", 1)[1].strip() if ":" in line else line
            if pending.startswith("aborting due to") or (pending.startswith(CARGO_GAVE_UP) and "could-not-compile-counted" not in BREAK):
                pending = None
            continue
        match = DIAGNOSTIC_LOCATION.match(line)
        if match and pending is not None:
            blocks.append((pending, match.group(1), int(match.group(2))))
            pending = None
    return blocks


def proof_error_messages(text, count_rlimit=False):
    """输出里 error 起头的报错，除去含 rlimit 用尽的、以「aborting due to」起头的与 cargo 自己那一句「could not compile」之后剩下的（证明错）；
    必红世界表第五段的 M 数的就是它们，只剩 rlimit 用尽的那一行不作数。count_rlimit 只给弄坏开关 rlimit-counted-in-proof-errors 用：rlimit 用尽也算进去。
    弄坏开关 could-not-compile-counted：cargo 那一句照算（cargo 那一种每行必红世界多一条、落点多一个（无位置））。"""
    messages = []
    for line in text.split("\n"):
        if not line.startswith("error"):
            continue
        message = line.split(":", 1)[1].strip() if ":" in line else line
        if (RLIMIT_EXHAUSTED in message and not count_rlimit) or message.startswith("aborting due to"):
            continue
        if message.startswith(CARGO_GAVE_UP) and "could-not-compile-counted" not in BREAK:
            continue
        messages.append(message)
    return messages


def integer_counts(results):
    """JSON 的 verified 与 errors 都是整数才交 True：缺一个是 vir 错没走到验证，或 Verus 换了 JSON 的写法。"""
    return all(isinstance(results.get(key), int) and not isinstance(results.get(key), bool) for key in ("verified", "errors"))


def outcome_problem(outcome, options):
    """这一次 Verus 没得出可判的结局时交回一句（被信号停了、没出 JSON），得出了交 None。"""
    signal_number = killed_signal(outcome)
    if signal_number is not None:
        tail = " | ".join(line for line in outcome["diagnostics"].strip().split("\n")[-4:] if line.strip())
        return f"Verus 被信号停了（退 {outcome['exit']}，信号 {signal_number}），这一次的输出不算判定{'（' + tail[:300] + '）' if tail else ''}"
    if outcome["json"] is None or "verification-results" not in outcome["json"]:
        tail = " | ".join(line for line in outcome["diagnostics"].strip().split("\n")[-6:] if line.strip())
        return f"Verus 没出判定（退 {outcome['exit']}，编不过或起不来）：{tail}"
    return None


def outcome_judged(outcome):
    """这一次 Verus 出没出能记进判定库的结局（红的记红、撤绿用）：被信号停了交 False（这一次的输出不算判定）；
    别的（Verus 报错、没出 JSON）交 True。"""
    return killed_signal(outcome) is None


def problem_howto(outcome, howto):
    """outcome_problem 报了问题时给的出路：被信号停了给 KILLED_HOWTO（是外面停的，不写修证明），别的给这一格自己的 howto。"""
    if killed_signal(outcome) is not None:
        return KILLED_HOWTO
    return howto


def killed_signal(outcome):
    """Verus 被信号停了就交信号号：直接起时 subprocess 交负的信号号，经 bash 起时退 128 + 信号号（129–192）；
    别的退出码交 None。弄坏开关 killed-as-proof-red 下一律交 None。"""
    code = outcome["exit"]
    if "killed-as-proof-red" in BREAK:
        return None
    if code < 0:
        return -code
    return code - 128 if 128 < code <= 128 + 64 else None


def product_rustc_version(root, options):
    """产品构建的 rustc（在被判的仓根起 rustc -vV，rustup 照那里的设定选工具链）交 (release, commit-hash)；起不了交 (None, 原因)。
    样本给 options["rustc_version_file"]（.formal-sample/rustc-version）时读那份文件当 rustc -vV 的输出。"""
    if options.get("rustc_version_file"):
        text = open(options["rustc_version_file"], encoding="utf-8").read()
    else:
        try:
            completed = subprocess.run(["rustc", "-vV"], cwd=root, capture_output=True, text=True)
        except OSError as error:
            return None, f"起不了 rustc -vV：{error}"
        if completed.returncode != 0:
            return None, f"rustc -vV 退 {completed.returncode}：{completed.stderr.strip()[-200:]}"
        text = completed.stdout
    fields = dict(line.split(": ", 1) for line in text.split("\n") if ": " in line)
    return (fields.get("release"), fields.get("commit-hash")), None


def rustc_pin_problems(root, registry, version, options, problems):
    """Verus 与产品的 rustc 版本差进工具链钉表：登记表 rustc 行三段——Verus 用的工具链（version.json 的 toolchain 第一段）、
    产品 rustc 的 release、commit-hash，都与此刻的相同。两边不同不判红（今天 Verus 1.98.1、产品 1.98.0），判的是「差」被钉住：哪一边换了都要改这一行，
    要改这一行时先在新的一对上把 MIR 比对跑一遍。问题追加进 problems，交一句给成功句。弄坏开关 FORMAL_VERIFICATION_BREAK=rustc-pin-unchecked 下不核。"""
    if "rustc-pin-unchecked" in FORMAL_BREAK:
        return "Verus 与产品的 rustc 没核（弄坏开关 rustc-pin-unchecked）"
    if registry.rustc is None:
        problems.append((f"登记表 {REGISTRY} 没有 rustc 行：Verus 与产品的 rustc 版本差没钉住",
                         "加一行 rustc<TAB><Verus 的 version.json 里 toolchain 的第一段><TAB><产品 rustc -vV 的 release><TAB><它的 commit-hash>"))
        return ""
    verus_toolchain = ((version or {}).get("toolchain") or "").split(" ")[0]
    if verus_toolchain != registry.rustc[0]:
        problems.append((f"Verus 用的工具链是 {verus_toolchain or '（version.json 里没有）'}，登记表 rustc 行写的是 {registry.rustc[0]}",
                         "换了 Verus 就照它 version.json 的 toolchain 改 rustc 行第二段，改之前在新工具链上把 MIR 比对（verus-proof 整格）跑一遍"))
    parsed, failure = product_rustc_version(root, options)
    release, commit = parsed if parsed else (None, None)
    if failure:
        problems.append((f"产品的 rustc 版本取不到：{failure}", "让 rustc -vV 在仓根跑得通（rustup 装着产品的工具链）"))
    elif (release, commit) != (registry.rustc[1], registry.rustc[2]):
        problems.append((f"产品的 rustc 是 {release}（commit-hash {commit}），登记表 rustc 行写的是 {registry.rustc[1]}（{registry.rustc[2]}）",
                         "产品的工具链换了：照 rustc -vV 改 rustc 行第三、四段，改之前在新的一对上把 MIR 比对（verus-proof 整格）跑一遍"))
    return f"Verus 的工具链 {registry.rustc[0]} 与产品的 rustc {registry.rustc[1]} 都与 rustc 行相同"


def cheating_markers(root, kernels):
    found = []
    for kernel in kernels:
        for number, line in enumerate(open(kernel, encoding="utf-8").read().split("\n"), 1):
            match = CHEATING_MARKER.search(strip_line_comment(line))
            if match:
                found.append(f"{os.path.relpath(kernel, root)}:{number}：{match.group(0)}")
    return found


def rlimit_attributes(root, kernels):
    found = []
    for kernel in kernels:
        for number, line in enumerate(open(kernel, encoding="utf-8").read().split("\n"), 1):
            match = RLIMIT_ATTRIBUTE.search(strip_line_comment(line))
            if match:
                found.append(f"{os.path.relpath(kernel, root)}:{number}：{match.group(0)}")
    return found


# ── 被验区域：信任清单、axiom、cfg 原子、verus! 外面的 trait impl（起 Verus 之前判）──

def read_trust_list(root):
    """信任清单 → (vstd 那几行 [(版本, 内容指纹, 行号)], 条目行 [dict], 写坏的行 [说明])；表不在交 None。
    条目行六段：种类（TRUST_KINDS 之一或 cfg）、文件（相对仓根）、点名的条目、规格原文的 sha256、差分测试的测试名、理由。vstd 行四段：vstd、版本、内容指纹、理由。"""
    path = os.path.join(root, TRUST_LIST)
    if not os.path.isfile(path):
        return None
    vstd_rows, rows, problems = [], [], []
    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        if not raw.strip() or raw.startswith("#"):
            continue
        fields = raw.split("\t")
        if fields[0] == "vstd":
            if len(fields) != 4 or not all(fields[1:]) or not re.fullmatch(r"[0-9a-f]{64}", fields[2]):
                problems.append(f"{TRUST_LIST} 第 {number} 行：vstd 行四段（vstd、版本、内容指纹 64 位小写十六进制、理由）")
                continue
            vstd_rows.append((fields[1], fields[2], number))
        elif fields[0] in TRUST_KINDS + (CFG_KIND,):
            if len(fields) != 6 or not all(fields[1:]) or not re.fullmatch(r"[0-9a-f]{64}", fields[3]) or len(fields[5]) < 4:
                problems.append(f"{TRUST_LIST} 第 {number} 行：条目行六段（种类、文件、点名的条目、规格原文的 sha256、差分测试名、理由至少 4 个字）")
                continue
            rows.append({"number": number, "kind": fields[0], "file": fields[1], "point": fields[2], "digest": fields[3], "test": fields[4]})
        else:
            problems.append(f"{TRUST_LIST} 第 {number} 行：第一段「{fields[0]}」不是 vstd、cfg 或 {'、'.join(TRUST_KINDS)} 之一")
    return vstd_rows, rows, problems


_LAG_BASELINE = []


def lag_baseline_library():
    """滞后表「只缩不涨」的共用读法 .claude/gate.d/lib-lag-baseline.py（content_rows、gate_base_of、baseline_rows），取这份脚本所在的仓里那一份，载一次。"""
    if not _LAG_BASELINE:
        import importlib.util
        specification = importlib.util.spec_from_file_location("lib_lag_baseline_for_verus_proof_check", LAG_BASELINE_LIBRARY)
        library = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(library)
        _LAG_BASELINE.append(library)
    return _LAG_BASELINE[0]


def read_trait_impls_outside_lag(root):
    """登记式放过表 → (行 [dict], 写坏的行 [说明])；表不在交 ([], [])：一行都没登记。
    五段（制表符分隔）：trait 的全路径（<crate>::<模块>::<trait>）、实现类型（impl 头里 for 后面那个类型的名字，不带路径与泛型参数）、
    所在文件（相对仓根）、哪一簇会把它挪进 verus!、为什么此刻放过（至少 8 个字）。"""
    path = os.path.join(root, TRAIT_IMPLS_OUTSIDE_LAG)
    if not os.path.isfile(path):
        return [], []
    rows, problems = [], []
    for number, fields in lag_baseline_library().content_rows(open(path, encoding="utf-8").read()):
        fields = [field.strip() for field in fields]
        if len(fields) != 5 or not all(fields) or "::" not in fields[0] or not RUST_IDENTIFIER.fullmatch(fields[1]) or len(fields[4]) < 8:
            problems.append(f"{TRAIT_IMPLS_OUTSIDE_LAG} 第 {number} 行：要五段（trait 的全路径 <crate>::<模块>::<trait>、实现类型的名字、所在文件、"
                            "哪一簇会把它挪进 verus!、为什么此刻放过至少 8 个字）")
            continue
        rows.append({"number": number, "trait": fields[0], "type": fields[1], "file": fields[2], "cluster": fields[3], "reason": fields[4]})
    return rows, problems


# 阶段只跑了一部分时跳过的逐项报进门禁汇总的「本次未跑」（.claude/singlefs-ai-sop/rules/show-me-test.md「门禁不许假装通过」）：门禁给的那份文件，不在时不写
NOT_RUN_FILE_VARIABLE = "GATE_NOT_RUN_FILE"


def report_passed_not_run(options, registry):
    """放过表放过的每一处（region_problems 记进 options["trait_impls_passed"]）往 $GATE_NOT_RUN_FILE 追加一行本次未跑：那个 impl 的契约这一次没人验，只写在成功句里不算报了。
    变量不在时不写（单跑与喂样本时它不在），交一句给成功句说明报了几行或为什么没报。弄坏开关 trait-impl-lag-not-run-unreported 下不写。"""
    passed = options.get("trait_impls_passed") or []
    if not passed:
        return ""
    path = os.environ.get(NOT_RUN_FILE_VARIABLE)
    if "trait-impl-lag-not-run-unreported" in BREAK or not path:
        return f"；放过的 {len(passed)} 处没往 ${NOT_RUN_FILE_VARIABLE} 报本次未跑（{'弄坏开关 trait-impl-lag-not-run-unreported' if path else '这个变量不在：单跑或喂样本'}）"
    with open(path, "a", encoding="utf-8") as handle:
        for row, where in passed:
            handle.write(f"formal-verification 的 verus-proof 格（{registry.crate_directory}）：verus! 外面的 {row['trait']} for {row['type']}"
                         f"（{where.split(' ', 1)[0]}）登记在 {TRAIT_IMPLS_OUTSIDE_LAG} 里放过，契约本次未验（靠信任，归「{row['cluster']}」挪进 verus!）\n")
    return f"；放过的 {len(passed)} 处逐个往 ${NOT_RUN_FILE_VARIABLE} 报了本次未跑"


def trust_list_text(root, registry):
    """进每一项键的「信任清单」那一样：vstd 行与这个 crate 里的条目行（去掉理由），排好序。表不在交空串。"""
    listed = read_trust_list(root)
    if listed is None:
        return ""
    vstd_rows, rows, _problems = listed
    lines = [f"vstd\t{version}\t{fingerprint}" for version, fingerprint, _number in vstd_rows]
    lines += [f"{row['kind']}\t{row['file']}\t{row['point']}\t{row['digest']}\t{row['test']}" for row in rows
              if row["file"].startswith(registry.crate_directory + "/")]
    return "\n".join(sorted(lines))


TEST_FUNCTION = re.compile(r"#\s*\[\s*test\s*\]\s*(?:#\s*\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)")


def test_functions(root):
    """仓里 crates/ 下每份 .rs 里标了 #[test] 的函数名（差分测试的测试名照它核：写全路径的取最后一段）。"""
    names = set()
    for path in glob.glob(os.path.join(root, "crates", "**", "*.rs"), recursive=True):
        try:
            names.update(TEST_FUNCTION.findall(open(path, encoding="utf-8", errors="replace").read()))
        except OSError:
            continue
    return names


def innermost_unit(rust_file, index):
    """词法单元 index 落在的最里层那个单元（数据单元不算）；不在任何单元里交 None。"""
    best = None
    for unit in rust_file.units:
        if unit.data or unit.first_token is None or unit.last_token is None or not unit.first_token <= index <= unit.last_token:
            continue
        if best is None or unit.last_token - unit.first_token < best.last_token - best.first_token:
            best = unit
    return best


def joined_tokens(rust_file, low, high):
    """词法单元 [low, high) 用一个空格连起来（去了注释与空白：只改注释、换行的不变）。"""
    return " ".join(rust_file.texts[low:high])


def statement_end(rust_file, index, limit):
    """从 index 往后找同一层的 ;（跳过配对的括号），交它的下标；找不到交 limit。"""
    position = index
    while position < limit:
        if rust_file.texts[position] in ("(", "[", "{") and position in rust_file.pairs:
            position = rust_file.pairs[position] + 1
            continue
        if rust_file.texts[position] == ";":
            return position
        position += 1
    return limit


def body_open(rust_file, unit):
    """fn 单元函数体的 {：lib_function_units 切出的单元是半开区间 [first_token, last_token)，单元里最后一个词法单元（last_token - 1）是 } 时，
    交与它配对、落在单元里的那个 {；最后一个词是 ;（trait 里只有签名与契约的声明）或配不上的，交 None。只按切出的单元与括号配对认，
    签名与 ensures 里的花括号（结构体字面量）不算函数体。external_body 的规格原文（trusted_declarations）与 trait 里的声明不成项（proof_items）都用它。"""
    if unit is None or unit.last_token is None:
        return None
    last = unit.last_token - 1
    if last < unit.first_token or rust_file.texts[last] != "}":
        return None
    return next((opening for opening, closing in rust_file.pairs.items() if closing == last and opening >= unit.first_token), None)


def body_open_past_unit_end(rust_file, unit):
    """弄坏开关 body-open-past-unit-end 下 external_body 的规格原文用的取法：把 last_token 本身当单元最后一个词，
    一个 impl 里最后一个 fn 落在 impl 的 } 上、配到 impl 的 {，签名那一截取成空串；后面还接着别的条目的 fn 配不上，取成整个 fn 再多一个词。"""
    if unit is None or unit.last_token is None or unit.last_token >= rust_file.count or rust_file.texts[unit.last_token] != "}":
        return None
    return next((opening for opening, closing in rust_file.pairs.items() if closing == unit.last_token), None)


def whole_entry_end(unit, cfg_atom=False):
    """取整个条目当规格原文时的上界（uninterp、external_*_specification、带 external_body 的类型规格，与 cfg 原子的「所在条目整个」）：
    单元是半开区间 [first_token, last_token)，取到 last_token 为止、不连下一个条目的头一个词。
    弄坏开关 whole-entry-past-unit-end 下几处都多取 last_token 那一个词；cfg-entry-past-unit-end 下只 cfg 原子那一处多取（cfg_atom 为真的调用）。"""
    if "whole-entry-past-unit-end" in BREAK or cfg_atom and "cfg-entry-past-unit-end" in BREAK:
        return unit.last_token + 1
    return unit.last_token


def assume_specification_parts(rust_file, index, limit):
    """index 是一处 assume_specification 那个词：交 (点名的条目, 方括号 ] 的下标或 None, 同一层的 ; 的下标)。点名的条目是方括号里那一项的词法单元连写
    （例 <[T]>::binary_search、singlefs_format::SLOT_BYTES），认不出方括号时是「（认不出方括号里的那一项）」。信任清单（trusted_declarations）与证明成项（proof_items）共用。"""
    texts = rust_file.texts
    opening = next((position for position in range(index + 1, min(index + 40, rust_file.count)) if texts[position] == "["), None)
    closing = rust_file.pairs.get(opening) if opening is not None else None
    point = "".join(texts[opening + 1:closing]) if closing is not None else "（认不出方括号里的那一项）"
    return point, closing, statement_end(rust_file, index, limit)


ENTRY_SCRIPT = os.path.join(SCRIPT_DIRECTORY, "verus-proof-check.py")   # 入口脚本：块文件里的 __file__ 是块自己的文件，要起入口的地方用这个
