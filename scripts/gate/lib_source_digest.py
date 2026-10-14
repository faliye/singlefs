#!/usr/bin/env python3
# admission: always 判的是这一次给的文件内容与两棵树，上一次的结论不替这一次作保
# run-condition: command bash git
# gate-similar: comment-only-change.py 规范副本里那一份判「一处改动是不是只动了注释」、只认 .rs、不出摘要；这里调它的词法切分算摘要，不另写一份 Rust 词法
# gate-similar: crash_identity.rs crates/singlefs-checker-tier 里崩溃放量库的判法摘要按同一个口径（include_str! 读的取原文、其余取词法单元），住在 Rust 里给判定库的键用，门禁这一侧的 python 调不到它
"""源码摘要：给一份文件算「不随注释与空白变」的摘要。门禁的复用判定共用这一份：scripts/gate/admission.py 的
stage-fingerprint（全绿标记的键）、崩溃枚举用例的输入清单（crash-case-manifest），与 gate-reuse 拿两棵树在登记路径上逐文件比的那一步
（scripts/gate/stage-must-run.sh 转到那里）。实验的输入指纹（admission.py experiment / manifest）不走这里，照旧逐文件 sha256。

摘要怎么算（按仓根起的相对路径的后缀分）：
  .rs        规范副本的 scripts/comment-only-change.py 的 rust_code_tokens_and_documentation_code：去掉注释与空白之后的词法单元（字符串、字符字面量整个算一个），
             外加文档注释里会编成 doctest 的代码行；文档注释与 `// SAFETY:` 注释在不在由它记占位。它只记 SAFETY 注释的占位、不记正文，
             这里另把每条 SAFETY 注释的正文按次序记进来（套在它的 SAFETY_TAG 外面顺手记下）：改 SAFETY 正文也算改了。
             它在规范副本目录里，从仓根按 .claude/singlefs-ai-sop/scripts/ 找；规范副本不随 git 走，临时 worktree 里没有时用主仓那一份
             （.claude/scripts/project_preflight.py 的 shared_preflight_directory，与 preflight 同一种找法）。
  .sh .bash  交给 bash 自己的解析器：`bash -n` 过了、stderr 一个字没有的，整份包进一个函数体，取 `declare -f` 打出来的那一份
             （bash 解析之后重新打印：注释、空行、缩进与多余的空白都不在，here-doc 正文与引号里的照原样），再按次序接上 `#!` 那一行与
             「# <词>:」「# shellcheck」形状的注释（admission:、run-condition:、gate-lint:… 这类有程序读的，引号里长成这样的也算进来，多跑不少跑）。
             `bash -n` 不过、有告警、包不进函数的取原文。
  .py        tokenize 切词：去掉注释（「# <词>:」「# shellcheck」「# noqa」「#!」与 coding 声明这几种留着），去掉行尾空白与只剩空白的行；
             跨行的词（三引号字符串、跨行 f-string）占到的每一行照原样。tokenize 切不了、切出 ERRORTOKEN 的取原文。
  .md .txt   crates/ 下的不进指纹：判据是 crates/ 下的源码、测试与 build.rs 没有一处读它们（2026-10-13 查：
             `git grep -n -E 'invariants|\\.md\\b|\\.txt\\b' -- 'crates/*/src' 'crates/*/tests' 'crates/*/build.rs' 'crates/*/benches' 'crates/*/examples'`
             去掉 // 与 * 开头的注释行之后一处不剩，`git grep -n 'invariants/' -- crates/ ':!crates/*/invariants/*'` 只命中注释）；
             被 include_str! / include_bytes! / include! 以字面路径读的、落在下面「整个 crate 取原文」那种 crate 里的照原文进。
             crates/ 外的取原文（登记表里点名的 .md 是阶段或实验要读的，例 checker-tier-qemu-device-streams 那一行的 .claude/kb/layout/01-first-txn.md）。
  其余       Cargo.toml、Cargo.lock、.tsv、.wgsl、.json 这些取原文。
  被 include_str! / include_bytes! 以字面路径读的文件（路径相对写它的那份 .rs 所在目录，`::core::` / `std::` 前缀也认）一律取原文，本身是 .rs 的也一样：
  读它的代码拿到的是原文（与 crash_identity.rs 文件头同一条）。参数不是字面路径的（`concat!(env!("OUT_DIR"), …)` 读编译产物的除外）、
  用 use 把 include_str / include_bytes 引进来的（改不改名都算），写它的那份文件往上最近的、有 Cargo.toml 的目录下的文件全部取原文；
  找不到这样的目录就这一批全部取原文。一批文件里谁读谁按这一批自己现算（digests_of 不给 context 时）：两棵树比的时候各按各的树算。
取原文的摘要就是文件字节的 sha256（与 sha256sum 相同）；别的摘要在内容前面带一段写法名（DIGEST_FORM 与种类）再算 sha256，与取原文的撞不上。
写法改了（摘要的算法、留哪些注释、哪些后缀进来）就抬 DIGEST_FORM：admission.py 把它写进指纹与全绿标记，旧标记对不上就按没有标记处置。

缓存：.rs 的词法分析、.sh 与 .py 的摘要按「内容的 sha256」记在 git common-dir 的 singlefs-source-digest-cache.tsv，跨进程、跨 worktree 复用
（全量 crates/ 切一遍约 4 秒，崩溃枚举用例每条开跑与跑完各算一次）。一行一条：<命名空间> <种类> <内容 sha256> <值> <校验>，
命名空间 = 这一份库、规范副本那一份 comment-only-change.py、Python 版本（.py）与 bash 版本（.sh）的摘要，任何一样变了旧行不再命中；
校验是另外四列连起来的 sha256、截成 16 位十六进制，对不上的行当没有；至多留 CACHE_ENTRIES_KEPT_AT_MOST 行（先进先出）。写不进去（目录只读、磁盘满）就不记，摘要照算。
弄坏开关（这一份的与规范副本那份脚本的 COMMENT_ONLY_CHANGE_BREAK）也进命名空间：开着时算的与平常的互不命中。

用法：
  lib_source_digest.py digest <根> <路径>…                    逐个打「<写法><制表符><摘要，不进指纹的打 -><制表符><路径>」（读工作区；谁读谁按给的这一批算）
  lib_source_digest.py tree-diff <根> <旧树> <新树> -- <路径…>  两棵树在这些路径（git 路径限定，认 :(exclude)）上比：
                                                                打「changed<制表符><路径>」与「same<制表符><路径>」（blob 不同而摘要相同的）
  lib_source_digest.py --selftest                              自证（格数由成功行现算）
退出码：0 判完；2 用法错；3 算不出（规范副本里找不到 comment-only-change.py、bash 起不来、git 读不出树），原因打在 stderr。

弄坏开关（SOURCE_DIGEST_BREAK=<项>[,<项>…]，只给自证证红用；自证里另逐个打开、核点名的那几格转红）：
  rust-raw                  .rs 取原文：「.rs 加一行 // 注释」「.rs 改文档注释措辞」「.rs 改块注释与缩进」三格红，连带两格「…照样按词法、摘要相同」
                            与「tree-diff：两棵树只差注释」
  literals-dropped          .rs 的字面量（字符串、字符、数字）不进摘要：「.rs 改一个字面量」那一格红，连带「tree-diff：两棵树差一个字面量」
  safety-text-ignored       不记 SAFETY 注释的正文：「.rs 改 SAFETY 注释的正文」那一格红
  doc-code-ignored          不记文档注释里的代码行：「.rs 改文档注释里的代码块」那一格红
  include-target-lexical    被 include_str! 读的文件、整个 crate 取原文的那几份照样按词法算：「被 include_str! 读的 .rs 只改注释」
                            「include_str! 的参数是算出来的：同一个 crate 下的…」两格红，连带「crates/ 下被 #[doc = include_str!] 读的 .md」
                            与「tree-diff：被 include_str! 读的 table.rs」
  shell-raw                 .sh 取原文：「.sh 只改注释与空行」那一格红，连带 tree-diff 里 sample.sh 只差注释的那三格
  shell-naive-comments      .sh 不经 bash，按行把「行首或空白后的 # 到行尾」删掉：「.sh here-doc 里 # 开头的行改了」「.sh 引号里的 # 后面改了」两格红，
                            连带「.sh bash -n 不过的取原文」
  python-raw                .py 取原文：「.py 只改注释与空行」那一格红
  python-naive-comments     .py 不经 tokenize，按行把「# 到行尾」删掉：「.py 字符串里的 # 后面改了」「.py 三引号串里 # 开头的行改了」两格红，
                            连带「.py tokenize 切不了的取原文」
  docs-kept-in-crates       crates/ 下没被读的 .md 照原文进：「crates/ 下没人读的 .md 不进指纹」那一格红，连带 tree-diff 里有 notes.md 的五格
  docs-dropped-everywhere   哪里的 .md 都不进：「crates/ 外的 .md 取原文」那一格红
  cache-unchecked           缓存行的校验不核：「缓存行被改坏了不认」那一格红

管不到的：
  运行期按拼出来的路径读源码原文的测试（fs::read_to_string 读 .rs、判原文里有没有某一串）：只改注释、摘要不变，而它可能因注释里的字变红。
    2026-10-13 现查的例：crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs 的
    this_flow_does_not_reference_the_random_history_scaffolding 读自己的源文件、判原文不含 `singlefs_harness::history`；
    gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs 扫 src/ 下各份 .rs，先去掉注释再判（code_lines），不受影响。
    按文件名、路径在字符串里出现过就取原文太宽（crates/ 里约 100 份被字符串点过名，多是判法闭包按词法读），这里不这么做。
  行号：去掉的注释行让后面代码的行号变（line!()、panic 的位置、编译告警里的行号）；2026-10-13 查 crates/ 里没有 line!() 与 Location::caller。
  变异表（crates/mutations.tsv）的原文按字节在源码里命中：只改注释可能让一条原文不再恰好命中一次，那一条由门禁 code-source-discipline 每轮判，不靠这里。
  bash、python 升级后 declare -f 与 tokenize 的输出可能变：摘要跟着变，多跑一次，不会少跑。
  .sh 里有程序读、又不长成「# <词>:」的注释（例：别的脚本按行抓这份文件里某句中文注释）：改它摘要不变。
"""
import hashlib
import importlib.util
import io
import json
import os
import posixpath
import re
import subprocess
import sys
import tempfile
import tokenize
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight, shared_preflight_directory  # noqa: E402

DIGEST_FORM = "singlefs-source-digest-v1"
BREAK_VARIABLE = "SOURCE_DIGEST_BREAK"
UPSTREAM_BREAK_VARIABLE = "COMMENT_ONLY_CHANGE_BREAK"
UPSTREAM_SCRIPT_NAME = "comment-only-change.py"
CACHE_FILE_NAME = "singlefs-source-digest-cache.tsv"
CACHE_ENTRIES_KEPT_AT_MOST = 40000
SHELL_SUFFIXES = (".sh", ".bash")
DOCUMENT_SUFFIXES = (".md", ".txt")
DOCUMENTS_LEFT_OUT_UNDER = "crates/"
SHELL_TIMEOUT_SECONDS = 600
ZERO_OBJECT = "0" * 40
SPECIAL_MODES = ("120000", "160000")   # 符号链接与子模块：blob 不同就算变了，不按内容算摘要
RAW_READING_MACROS = ("include_str", "include_bytes")
CODE_INCLUDING_MACRO = "include"
OUT_DIRECTORY_INCLUDE = ["concat", "!", "(", "env", "!", "(", '"OUT_DIR"', ")"]
OPENING_BRACKETS = ("(", "[", "{")
CLOSING_AFTER_LITERAL = (")", "]", "}", ",")
KEPT_PYTHON_COMMENT = re.compile(r"^#!|^#.*\bcoding[:=]|^#\s*(?:[A-Za-z][A-Za-z0-9_-]*:|shellcheck\b|noqa\b)")
KEPT_SHELL_COMMENT = re.compile(r"(?:^|[\s;&|()])(#\s*(?:[A-Za-z][A-Za-z0-9_-]*:|shellcheck\b).*)$")
NAIVE_SHELL_COMMENT = re.compile(r"(?:^|(?<=\s))#.*$")
RAW_STRING_LITERAL = re.compile(r'r(#*)"(.*)"\1', re.S)
LITERAL_TOKEN_START = re.compile(r'''^(?:["']|[0-9]|(?:b|c)?r#*"|b["'])''')
SHELL_NORMALIZE_SCRIPT = r'''
for source_path in "$@"; do
  if bash -n "$source_path" 2>"$source_path.check" && [[ ! -s "$source_path.check" ]]; then
    source_text="$(<"$source_path")"
    if eval "__singlefs_source_digest_body() {
$source_text
}" 2>/dev/null; then
      declare -f __singlefs_source_digest_body >"$source_path.normal"
    fi
    unset -f __singlefs_source_digest_body
  fi
done
'''


class DigestError(Exception):
    """算不出摘要：规范副本里找不到 comment-only-change.py、bash 起不来、git 读不出树。调用方按「判不出来」处置（门禁按要跑）。"""


def breaks_set():
    return {name for name in os.environ.get(BREAK_VARIABLE, "").split(",") if name}


def break_is_set(name):
    return name in breaks_set()


def raw_digest(content):
    return hashlib.sha256(content).hexdigest()


def form_digest(kind, text):
    return hashlib.sha256(f"{DIGEST_FORM} {kind}\0".encode("utf-8") + text.encode("utf-8", "surrogateescape")).hexdigest()


def classify(name):
    lower = name.lower()
    if lower.endswith(".rs"):
        return "rust"
    if lower.endswith(SHELL_SUFFIXES):
        return "shell"
    if lower.endswith(".py"):
        return "python"
    if lower.endswith(DOCUMENT_SUFFIXES):
        return "document"
    return "raw"


# ── .rs：调规范副本的 comment-only-change.py ─────────────────────────────────

class SafetyCommentRecorder:
    """套在 comment-only-change.py 的 SAFETY_TAG 外面：它判每条注释是不是 SAFETY 注释时顺手记下正文（它自己只记占位）。"""

    def __init__(self, pattern):
        self.pattern = pattern
        self.bodies = []

    def match(self, body):
        found = self.pattern.match(body)
        if found:
            self.bodies.append(body.strip())
        return found


UPSTREAM = {}


def upstream_module():
    """→ (模块, SAFETY 记录器, 那份脚本的字节)；只载一次。找不到、载不进、缺那两样抛 DigestError。"""
    if "module" not in UPSTREAM:
        directory = shared_preflight_directory()
        path = os.path.join(directory, UPSTREAM_SCRIPT_NAME) if directory else ""
        if not path or not os.path.isfile(path):
            raise DigestError(f"规范副本里找不到 scripts/{UPSTREAM_SCRIPT_NAME}（这棵树与主仓的 .claude/singlefs-ai-sop/ 都没有）")
        saved_path = list(sys.path)
        specification = importlib.util.spec_from_file_location("singlefs_comment_only_change", path)
        module = importlib.util.module_from_spec(specification)
        try:
            specification.loader.exec_module(module)
        except Exception as error:  # 载不进就算不出：交给调用方按判不出来处置，不当「没变」
            raise DigestError(f"载不进 {path}：{type(error).__name__}：{error}") from error
        finally:
            sys.path[:] = saved_path
        if not hasattr(module, "rust_code_tokens_and_documentation_code") or not hasattr(module, "SAFETY_TAG"):
            raise DigestError(f"{path} 里没有 rust_code_tokens_and_documentation_code 或 SAFETY_TAG：它改了写法，这一份要跟着改")
        recorder = SafetyCommentRecorder(module.SAFETY_TAG)
        module.SAFETY_TAG = recorder
        with open(path, "rb") as handle:
            UPSTREAM.update(module=module, recorder=recorder, source=handle.read())
    return UPSTREAM["module"], UPSTREAM["recorder"], UPSTREAM["source"]


def string_literal_value(token):
    """普通字符串字面量与原始字符串的内容；带反斜杠转义的、不是字符串的回 None。"""
    if len(token) >= 2 and token.startswith('"') and token.endswith('"'):
        inner = token[1:-1]
        return None if "\\" in inner else inner
    raw = RAW_STRING_LITERAL.fullmatch(token)
    return raw.group(2) if raw else None


def is_literal_token(token):
    """字符串、字符、数字字面量（只给弄坏开关 literals-dropped 用）；生命周期的单个 ' 不算。"""
    return token[:1].isdigit() or (len(token) > 1 and LITERAL_TOKEN_START.match(token) is not None)


def include_references(tokens):
    """→ (include_str!/include_bytes! 的字面路径, include! 的字面路径, 有没有认不出读哪份的 include_str!/include_bytes!)。"""
    raw_literals, code_literals, unresolved = [], [], False
    for position, token in enumerate(tokens):
        if token not in RAW_READING_MACROS and token != CODE_INCLUDING_MACRO:
            continue
        reads_raw = token in RAW_READING_MACROS
        if tokens[position + 1:position + 2] != ["!"] or tokens[position + 2:position + 3] not in [[bracket] for bracket in OPENING_BRACKETS]:
            unresolved = unresolved or reads_raw   # use core::include_str as grab; 这一类：改名之后读哪份看不出
            continue
        argument = tokens[position + 3] if position + 3 < len(tokens) else ""
        literal = string_literal_value(argument)
        if literal is not None and tokens[position + 4:position + 5] and tokens[position + 4] in CLOSING_AFTER_LITERAL:
            (raw_literals if reads_raw else code_literals).append(literal)
        elif tokens[position + 3:position + 3 + len(OUT_DIRECTORY_INCLUDE)] == OUT_DIRECTORY_INCLUDE:
            continue   # 读 OUT_DIR 里的编译产物，不是仓里的文件
        elif reads_raw:
            unresolved = True
    return raw_literals, code_literals, unresolved


def rust_analysis(content):
    """→ {"d": 摘要, "s": include_str!/include_bytes! 的字面路径, "c": include! 的字面路径, "u": 有没有认不出的}。"""
    module, recorder, _source = upstream_module()
    text = content.decode("utf-8", "surrogateescape")
    recorder.bodies = []
    tokens, doc_code = module.rust_code_tokens_and_documentation_code(text)
    safety_bodies = list(recorder.bodies)
    raw_literals, code_literals, unresolved = include_references(tokens)
    kept_tokens = [token for token in tokens if not is_literal_token(token)] if break_is_set("literals-dropped") else tokens
    sections = ["rust-tokens"] + [f"{len(token)}:{token}" for token in kept_tokens]
    if not break_is_set("doc-code-ignored"):
        sections += ["doc-code"] + [f"{len(line)}:{line}" for line in doc_code]
    if not break_is_set("safety-text-ignored"):
        sections += ["safety-text"] + [f"{len(body)}:{body}" for body in safety_bodies]
    return {"d": form_digest("rust", "\n".join(sections)), "s": raw_literals, "c": code_literals, "u": unresolved}


# ── .py：tokenize ───────────────────────────────────────────────────────────

def python_normal_form(content):
    """去掉注释与只剩空白的行之后的原文；切不了回 None（取原文）。"""
    if break_is_set("python-naive-comments"):
        text = content.decode("utf-8", "surrogateescape")
        rows = [re.sub(r"#.*$", "", row).rstrip() for row in text.split("\n")]
        return "\n".join(row for row in rows if row)
    try:
        encoding, _declaration_lines = tokenize.detect_encoding(io.BytesIO(content).readline)
        text = content.decode(encoding)
        rows = io.StringIO(text).readlines()
        tokens = list(tokenize.generate_tokens(io.StringIO(text).readline))
    except (SyntaxError, tokenize.TokenError, UnicodeDecodeError, LookupError, ValueError):
        return None
    protected, cuts = set(), {}
    for token in tokens:
        if token.type == tokenize.ERRORTOKEN:
            return None
        if token.type in (tokenize.NEWLINE, tokenize.NL, tokenize.ENDMARKER, tokenize.INDENT, tokenize.DEDENT):
            continue
        if token.start[0] != token.end[0]:
            protected.update(range(token.start[0], token.end[0] + 1))
        if token.type == tokenize.COMMENT and not KEPT_PYTHON_COMMENT.match(token.string):
            cuts[token.start[0]] = token.start[1]
    kept = []
    for number, row in enumerate(rows, 1):
        body = row[:-1] if row.endswith("\n") else row
        if number in protected:
            kept.append(body)
            continue
        if number in cuts:
            body = body[:cuts[number]]
        body = body.rstrip(" \t\f\r")
        if body:
            kept.append(body)
    return "\n".join(kept)


# ── .sh：bash 自己的解析器 ──────────────────────────────────────────────────

def kept_shell_comments(text):
    lines = text.split("\n")
    kept = [lines[0]] if lines and lines[0].startswith("#!") else []
    for line in lines:
        found = KEPT_SHELL_COMMENT.search(line)
        if found:
            kept.append(found.group(1))
    return kept


def bash_normal_forms(contents):
    """每份内容 → declare -f 打出来的那一份（str），bash -n 不过、有告警、包不进函数的 → None。bash 起不来抛 DigestError。"""
    with tempfile.TemporaryDirectory(prefix="source-digest-shell-") as directory:
        paths = []
        for index, content in enumerate(contents):
            path = os.path.join(directory, f"{index}.sh")
            with open(path, "wb") as handle:
                handle.write(content)
            paths.append(path)
        environment = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "LC_ALL": "C", "HOME": directory}
        try:
            completed = subprocess.run(["bash", "--norc", "--noprofile", "-c", SHELL_NORMALIZE_SCRIPT, "source-digest", *paths],
                                       cwd=directory, env=environment, capture_output=True, stdin=subprocess.DEVNULL,
                                       timeout=SHELL_TIMEOUT_SECONDS)
        except (OSError, subprocess.TimeoutExpired) as error:
            raise DigestError(f"bash 解析 .sh 没跑完：{error}") from error
        if completed.returncode != 0:
            raise DigestError(f"bash 解析 .sh 退 {completed.returncode}：{completed.stderr.decode('utf-8', 'replace').strip()[:300]}")
        forms = []
        for path in paths:
            normal_path = path + ".normal"
            if os.path.isfile(normal_path):
                with open(normal_path, "rb") as handle:
                    forms.append(handle.read().decode("utf-8", "surrogateescape"))
            else:
                forms.append(None)
        return forms


def naive_shell_form(content):
    rows = [NAIVE_SHELL_COMMENT.sub("", row).rstrip() for row in content.decode("utf-8", "surrogateescape").split("\n")]
    return "\n".join(row for row in rows if row)


# ── 缓存 ────────────────────────────────────────────────────────────────────

def bash_version_text():
    if "bash" not in UPSTREAM:
        try:
            completed = subprocess.run(["bash", "--version"], capture_output=True, text=True, errors="replace", stdin=subprocess.DEVNULL)
            UPSTREAM["bash"] = completed.stdout.split("\n")[0]
        except OSError:
            UPSTREAM["bash"] = "bash 起不来"
    return UPSTREAM["bash"]


def own_source():
    if "own" not in UPSTREAM:
        with open(os.path.realpath(__file__), "rb") as handle:
            UPSTREAM["own"] = handle.read()
    return UPSTREAM["own"]


def namespace_of(kind):
    if kind == "rust":
        extra = upstream_module()[2]
    elif kind == "python":
        extra = sys.version.encode()
    else:
        extra = bash_version_text().encode("utf-8", "replace")
    switches = f"{','.join(sorted(breaks_set()))}|{os.environ.get(UPSTREAM_BREAK_VARIABLE, '')}".encode()
    return hashlib.sha256(own_source() + b"\0" + kind.encode() + b"\0" + extra + b"\0" + switches).hexdigest()[:16]


def cache_line_check(fields):
    return hashlib.sha256("\t".join(fields).encode("utf-8", "surrogateescape")).hexdigest()[:16]


class DigestCache:
    """按内容的 sha256 记 .rs 的分析与 .sh / .py 的摘要；path 为 None 时不读不写。弄坏开关进命名空间：开着时算的与平常的互不命中。"""

    def __init__(self, path):
        self.path = path
        self.enabled = bool(path)
        self.entries = {}
        self.added = []
        self.hits = 0
        self.namespaces = {}
        if self.enabled:
            for fields in self.read_lines():
                self.entries[tuple(fields[:3])] = fields[3]

    def read_lines(self):
        try:
            with open(self.path, encoding="utf-8", errors="surrogateescape") as handle:
                text = handle.read()
        except OSError:
            return []
        lines = []
        for line in text.split("\n"):
            fields = line.split("\t")
            if len(fields) != 5:
                continue
            if not break_is_set("cache-unchecked") and cache_line_check(fields[:4]) != fields[4]:
                continue
            lines.append(fields)
        return lines

    def key(self, kind, content):
        if kind not in self.namespaces:
            self.namespaces[kind] = namespace_of(kind)
        return (self.namespaces[kind], kind, raw_digest(content))

    def get(self, kind, content):
        if not self.enabled:
            return None
        value = self.entries.get(self.key(kind, content))
        if value is None:
            return None
        try:
            parsed = json.loads(value)
        except ValueError:
            return None
        self.hits += 1
        return parsed

    def put(self, kind, content, value):
        if not self.enabled:
            return
        key = self.key(kind, content)
        text = json.dumps(value, ensure_ascii=False, separators=(",", ":"))
        if self.entries.get(key) != text:
            self.entries[key] = text
            self.added.append(list(key) + [text])

    def save(self):
        """新记的行接在磁盘上那一份后面（别的进程同时写的照留），超过上限丢最早的；同目录临时文件写完改名换上。写不了就不记。"""
        if not self.enabled or not self.added:
            return
        lines = [fields[:4] for fields in self.read_lines()] + self.added
        lines = lines[-CACHE_ENTRIES_KEPT_AT_MOST:]
        try:
            handle_number, temporary_path = tempfile.mkstemp(dir=os.path.dirname(self.path), prefix=os.path.basename(self.path) + ".partial.")
            with os.fdopen(handle_number, "w", encoding="utf-8", errors="surrogateescape") as handle:
                handle.write("".join("\t".join(fields + [cache_line_check(fields)]) + "\n" for fields in lines))
            os.replace(temporary_path, self.path)
        except OSError:
            return   # 缓存只省时间：写不进去下一趟照算，摘要不受影响
        self.added = []


def cache_for(root):
    """root 所在仓的 git common-dir 里那一份缓存；不在 git 仓里给一份不读不写的。"""
    completed = subprocess.run(["git", "-C", root, "rev-parse", "--path-format=absolute", "--git-common-dir"],
                               capture_output=True, text=True, stdin=subprocess.DEVNULL)
    common_directory = completed.stdout.strip() if completed.returncode == 0 else ""
    return DigestCache(os.path.join(common_directory, CACHE_FILE_NAME) if common_directory else None)


def cached(cache, kind, content, compute):
    value = cache.get(kind, content) if cache is not None else None
    if value is None:
        value = compute()
        if cache is not None:
            cache.put(kind, content, value)
    return value


# ── 一批文件里谁读谁 ─────────────────────────────────────────────────────────

def resolve_literal(directory, literal):
    if literal.startswith("/"):
        return None
    return posixpath.normpath(posixpath.join(directory, literal))


def crate_root_finder(has_file):
    """名字 → 它往上最近的、有 Cargo.toml 的目录（带结尾斜杠）；找不到回 ""（前缀 "" 罩住整批）。"""
    found = {}

    def crate_root_of(name):
        directory = posixpath.dirname(name)
        while directory not in found:
            if has_file(posixpath.join(directory, "Cargo.toml") if directory else "Cargo.toml"):
                found[directory] = directory + "/" if directory else ""
                break
            if not directory:
                found[directory] = ""
                break
            directory = posixpath.dirname(directory)
        return found[directory]
    return crate_root_of


class DigestContext:
    """一批文件里谁被 include_str! / include_bytes! 读原文、谁被 include! 带进来、哪几个 crate 整个取原文。"""

    def __init__(self):
        self.raw_names = set()
        self.included_names = set()
        self.raw_prefixes = set()

    def add_includer(self, name, analysis, crate_root_of):
        directory = posixpath.dirname(name)
        for literal in analysis["s"]:
            target = resolve_literal(directory, literal)
            if target is None:
                self.raw_prefixes.add(crate_root_of(name))
            else:
                self.raw_names.add(target)
        for literal in analysis["c"]:
            target = resolve_literal(directory, literal)
            if target is not None:
                self.included_names.add(target)
        if analysis["u"]:
            self.raw_prefixes.add(crate_root_of(name))

    def reads_raw(self, name):
        if break_is_set("include-target-lexical"):
            return False
        return name in self.raw_names or any(name.startswith(prefix) for prefix in self.raw_prefixes)


def context_of(named_analyses, crate_root_of):
    context = DigestContext()
    for name, analysis in named_analyses:
        context.add_includer(name, analysis, crate_root_of)
    return context


def digests_of(entries, cache=None, context=None, has_file=None):
    """entries：[(仓根起的名字, 内容字节)] → [(写法, 摘要)]，次序同 entries；不进指纹的是 (None, None)。
    写法是 raw / rust / shell / python。context 不给就按 entries 自己现算；has_file(仓根起的路径) 找 crate 根用，不给就按 entries 里有没有那个名字。"""
    rust = {index: cached(cache, "rust", content, lambda content=content: rust_analysis(content))
            for index, (name, content) in enumerate(entries) if classify(name) == "rust"}
    if context is None:
        names = {name for name, _content in entries}
        context = context_of(((entries[index][0], analysis) for index, analysis in rust.items()),
                             crate_root_finder(has_file or (lambda path: path in names)))
    shell_indices = [index for index, (name, _content) in enumerate(entries) if classify(name) == "shell" and not context.reads_raw(name)]
    shell_results = dict(zip(shell_indices, shell_digests_of([entries[index][1] for index in shell_indices], cache)))
    results = []
    for index, (name, content) in enumerate(entries):
        kind = classify(name)
        if context.reads_raw(name):
            results.append(("raw", raw_digest(content)))
        elif kind == "document":
            left_out = (break_is_set("docs-dropped-everywhere")
                        or (name.startswith(DOCUMENTS_LEFT_OUT_UNDER) and name not in context.included_names
                            and not break_is_set("docs-kept-in-crates")))
            results.append((None, None) if left_out else ("raw", raw_digest(content)))
        elif kind == "rust":
            results.append(("raw", raw_digest(content)) if break_is_set("rust-raw") else ("rust", rust[index]["d"]))
        elif kind == "shell":
            results.append(shell_results[index])
        elif kind == "python":
            results.append(python_digest(content, cache))
        else:
            results.append(("raw", raw_digest(content)))
    if cache is not None:
        cache.save()
    return results


def python_digest(content, cache):
    def compute():
        form = None if break_is_set("python-raw") else python_normal_form(content)
        return {"f": "raw", "d": raw_digest(content)} if form is None else {"f": "python", "d": form_digest("python", form)}
    value = cached(cache, "python", content, compute)
    return (value["f"], value["d"])


def shell_normal_forms(contents):
    """bash_normal_forms，连同弄坏开关：shell-raw 一律 None（取原文），shell-naive-comments 按行删 # 到行尾、不经 bash。"""
    if break_is_set("shell-raw"):
        return [None] * len(contents)
    if break_is_set("shell-naive-comments"):
        return [naive_shell_form(content) for content in contents]
    return bash_normal_forms(contents)


def shell_digests_of(contents, cache):
    results = [cache.get("shell", content) if cache is not None else None for content in contents]
    pending = [index for index, value in enumerate(results) if value is None]
    if pending:
        forms = shell_normal_forms([contents[index] for index in pending])
        for index, form in zip(pending, forms):
            content = contents[index]
            if form is None:
                value = {"f": "raw", "d": raw_digest(content)}
            else:
                kept = kept_shell_comments(content.decode("utf-8", "surrogateescape"))
                value = {"f": "shell", "d": form_digest("shell", form + "\n\0kept-comments\n" + "\n".join(kept))}
            results[index] = value
            if cache is not None:
                cache.put("shell", content, value)
    return [(value["f"], value["d"]) for value in results]


def working_tree_digests(root, names, cache=None):
    """工作区里这几份（仓根起的名字，str）的摘要，次序同 names；谁读谁按这一批算，crate 根按盘上有没有 Cargo.toml 找。读不了抛 OSError。"""
    entries = []
    for name in names:
        with open(os.path.join(root, name), "rb") as handle:
            entries.append((name, handle.read()))
    return digests_of(entries, cache, has_file=lambda path: os.path.isfile(os.path.join(root, path)))


# ── 两棵树 ──────────────────────────────────────────────────────────────────

def git(root, *arguments, accepted=(0,), input_bytes=None):
    try:
        completed = subprocess.run(["git", "-C", root, *arguments], capture_output=True, input=input_bytes,
                                   stdin=None if input_bytes is not None else subprocess.DEVNULL)
    except OSError as error:
        raise DigestError(f"git 起不来：{error}") from error
    if completed.returncode not in accepted:
        raise DigestError(f"git {' '.join(arguments[:3])} … 退 {completed.returncode}：{completed.stderr.decode('utf-8', 'replace').strip()[:300]}")
    return completed


def read_objects(root, object_names):
    """git cat-file --batch：每个对象名 → 内容字节，不在的 → None。"""
    if not object_names:
        return []
    output = git(root, "cat-file", "--batch", input_bytes=b"".join(os.fsencode(name) + b"\n" for name in object_names)).stdout
    contents, position = [], 0
    for _name in object_names:
        header_end = output.index(b"\n", position)
        header = output[position:header_end].split()
        position = header_end + 1
        if len(header) == 2 and header[1] == b"missing":
            contents.append(None)
            continue
        size = int(header[2])
        contents.append(output[position:position + size])
        position += size + 1
    return contents


def tree_of(root, tree_ish):
    return git(root, "rev-parse", "--verify", "-q", f"{tree_ish}^{{tree}}").stdout.decode().strip()


def tree_context(root, tree, pathspecs, cache):
    """这棵树在 pathspecs 上谁读谁：只有出现 include 字样的 .rs 才可能是读的一方（git grep 找）。"""
    found = git(root, "grep", "-z", "-l", "-I", "-E", r"include(_str|_bytes)?", tree, "--", *pathspecs, accepted=(0, 1)).stdout
    prefix = f"{tree}:"
    names = [os.fsdecode(item)[len(prefix):] for item in found.split(b"\0") if item]
    names = [name for name in names if classify(name) == "rust"]
    contents = read_objects(root, [f"{tree}:{name}" for name in names])
    exists = {}

    def has_file(path):
        if path not in exists:
            exists[path] = git(root, "cat-file", "-e", f"{tree}:{path}", accepted=(0, 1, 128)).returncode == 0
        return exists[path]
    analyses = [(name, cached(cache, "rust", content, lambda content=content: rust_analysis(content)))
                for name, content in zip(names, contents) if content is not None]
    return context_of(analyses, crate_root_finder(has_file))


def tree_differences(root, old_tree_ish, new_tree_ish, pathspecs, cache=None):
    """两棵树在 pathspecs 上比：→ (变了的, blob 不同而摘要相同的)，名字按字节序。
    模式变了的、符号链接与子模块 blob 不同的算变了；新增与删掉的按「不进指纹」与「有摘要」比（crates/ 下没人读的 .md 加减不算变）。"""
    old_tree, new_tree = tree_of(root, old_tree_ish), tree_of(root, new_tree_ish)
    output = git(root, "diff", "--raw", "-z", "--no-renames", "--no-abbrev", old_tree, new_tree, "--", *pathspecs).stdout
    fields = output.split(b"\0")
    changed, candidates, position = [], [], 0
    while position + 1 < len(fields) and fields[position]:
        old_mode, new_mode, old_object, new_object, _status = fields[position].decode().lstrip(":").split()[:5]
        name = os.fsdecode(fields[position + 1])
        position += 2
        if (old_mode in SPECIAL_MODES or new_mode in SPECIAL_MODES
                or (old_mode != new_mode and "000000" not in (old_mode, new_mode))):
            changed.append(name)
        else:
            candidates.append((name, old_object, new_object))
    same = []
    if candidates:
        sides = []
        for tree, object_position in ((old_tree, 1), (new_tree, 2)):
            present = [(item[0], item[object_position]) for item in candidates if item[object_position] != ZERO_OBJECT]
            contents = read_objects(root, [object_name for _name, object_name in present])
            digests = digests_of([(name, content) for (name, _object), content in zip(present, contents)], cache,
                                 context=tree_context(root, tree, pathspecs, cache))
            sides.append({name: digest for (name, _object), (_form, digest) in zip(present, digests)})
        for name, _old_object, _new_object in candidates:
            (same if sides[0].get(name) == sides[1].get(name) else changed).append(name)
    return sorted(changed), sorted(same)


# ── 自证 ────────────────────────────────────────────────────────────────────

RUST_BASE = '''//! 模块说明
use std::fmt;

/// 返回一。
///
/// ```
/// assert_eq!(demo::one(), 1);
/// ```
pub fn one() -> u32 {
    1
}

pub fn read(pointer: *const u8) -> u8 {
    // SAFETY: 调用方保证 pointer 指向活的一个字节
    unsafe { *pointer }
}
'''
SHELL_BASE = '''#!/usr/bin/env bash
# admission: always 样本
# 样本说明
set -euo pipefail
name="a # 不是注释"   # 行尾注释
cat <<'EOF'
# 这一行是 here-doc 正文，不是注释
EOF
echo "$name" # 又一个行尾注释
'''
PYTHON_BASE = '''#!/usr/bin/env python3
# admission: always 样本
"""文档串"""
import sys

TEXT = "a # 不是注释"  # 行尾注释
BLOCK = """
# 三引号串里的行
"""


def main():
    # 函数里的注释
    return len(TEXT) + len(BLOCK) + len(sys.argv)
'''


def edited(text, old, new):
    if text.count(old) != 1:
        raise AssertionError(f"自证样本里「{old}」不是恰好一处")
    return text.replace(old, new)


def digest_pair(name, before, after, extra=()):
    """同一个名字的两版内容（连同同一批里的别的文件）各算一次 → (改前的摘要, 改后的摘要)。"""
    others = [(other_name, other_text.encode()) for other_name, other_text in extra]
    earlier = digests_of([(name, before.encode())] + others)[0][1]
    later = digests_of([(name, after.encode())] + others)[0][1]
    return earlier, later


def content_cells():
    """[(格名, 判对了没有 的函数)]：每一格是「一份文件改前改后摘要相同 / 不同」。"""
    rust = "crates/demo/src/lib.rs"
    cargo = [("crates/demo/Cargo.toml", '[package]\nname = "demo"\n')]
    including = 'pub const TABLE: &str = include_str!("table.rs");\npub fn one() -> u32 { 1 }\n'
    table = "pub const ROWS: u32 = 3; // 行数\n"
    computed = 'pub const OTHER: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/other.rs"));\n'
    generated = 'pub const TABLE: &str = include_str!(concat!(env!("OUT_DIR"), "/table.tsv"));\n'
    other = "pub fn other() -> u32 { 7 } // 注释\n"

    def same(name, before, after, extra=()):
        return lambda: (lambda pair: pair[0] is not None and pair[0] == pair[1])(digest_pair(name, before, after, extra))

    def differ(name, before, after, extra=()):
        return lambda: (lambda pair: pair[0] != pair[1])(digest_pair(name, before, after, extra))

    def left_out(name):
        return lambda: digests_of([(name, b"# notes\n")] + [(other_name, other_text.encode()) for other_name, other_text in cargo])[0] == (None, None)

    return [
        (".rs 加一行 // 注释", same(rust, RUST_BASE, edited(RUST_BASE, "    1\n", "    // 新加的一行注释\n    1\n"), cargo)),
        (".rs 改文档注释措辞", same(rust, RUST_BASE, edited(RUST_BASE, "/// 返回一。", "/// 返回常数一，不会 panic。"), cargo)),
        (".rs 改块注释与缩进", same(rust, RUST_BASE, edited(RUST_BASE, "    1\n}", "  /* 块注释 */ 1\n    }"), cargo)),
        (".rs 改一个字面量", differ(rust, RUST_BASE, edited(RUST_BASE, "    1\n", "    2\n"), cargo)),
        (".rs 改 SAFETY 注释的正文", differ(rust, RUST_BASE, edited(RUST_BASE, "调用方保证 pointer 指向活的一个字节", "pointer 由调用方保证非空且对齐"), cargo)),
        (".rs 改文档注释里的代码块", differ(rust, RUST_BASE, edited(RUST_BASE, "assert_eq!(demo::one(), 1);", "assert_eq!(demo::one(), 2);"), cargo)),
        (".rs 删掉 SAFETY 注释的标记", differ(rust, RUST_BASE, edited(RUST_BASE, "// SAFETY: 调用方", "// 调用方"), cargo)),
        ("被 include_str! 读的 .rs 只改注释，摘要不同（读它的拿到的是原文）",
         differ("crates/demo/src/table.rs", table, table.replace("// 行数", "// 行数，改了措辞"), [(rust, including)] + cargo)),
        ("include_str! 的参数是算出来的：同一个 crate 下的 .rs 只改注释也取原文、摘要不同",
         differ("crates/demo/src/other.rs", other, other.replace("// 注释", "// 改了的注释"), [(rust, computed)] + cargo)),
        ("include_str! 的参数是算出来的：别的 crate 下的 .rs 只改注释照样按词法、摘要相同",
         same("crates/elsewhere/src/lib.rs", other, other.replace("// 注释", "// 改了的注释"),
              [(rust, computed), ("crates/elsewhere/Cargo.toml", '[package]\nname = "elsewhere"\n')] + cargo)),
        ("include_str!(concat!(env!(\"OUT_DIR\"), …)) 读编译产物：同一个 crate 下的 .rs 只改注释照样按词法、摘要相同",
         same("crates/demo/src/other.rs", other, other.replace("// 注释", "// 改了的注释"), [(rust, generated)] + cargo)),
        (".sh 只改注释与空行", same("scripts/sample.sh", SHELL_BASE,
                                    edited(edited(SHELL_BASE, "# 样本说明", "# 改过的说明，长一点\n\n   # 再加一行"), "# 又一个行尾注释", "# 改了的行尾注释"))),
        (".sh 改命令", differ("scripts/sample.sh", SHELL_BASE, edited(SHELL_BASE, "set -euo pipefail", "set -eu"))),
        (".sh here-doc 里 # 开头的行改了", differ("scripts/sample.sh", SHELL_BASE, edited(SHELL_BASE, "# 这一行是 here-doc 正文", "# 改了 here-doc 正文"))),
        (".sh 引号里的 # 后面改了", differ("scripts/sample.sh", SHELL_BASE, edited(SHELL_BASE, "a # 不是注释", "a # 改了"))),
        (".sh 改 # admission: 这一类有程序读的注释", differ("scripts/sample.sh", SHELL_BASE, edited(SHELL_BASE, "always 样本", "always 样本改了"))),
        (".sh bash -n 不过的取原文：只改注释也不同", differ("scripts/unparsable.sh", "if true; then\n# 一\n", "if true; then\n# 二\n")),
        (".py 只改注释与空行", same("scripts/sample.py", PYTHON_BASE,
                                    edited(edited(PYTHON_BASE, "# 函数里的注释", "# 改过的注释\n\n    # 再加一行"), "# 行尾注释", "# 改了的行尾注释"))),
        (".py 字符串里的 # 后面改了", differ("scripts/sample.py", PYTHON_BASE, edited(PYTHON_BASE, "a # 不是注释", "a # 改了"))),
        (".py 三引号串里 # 开头的行改了", differ("scripts/sample.py", PYTHON_BASE, edited(PYTHON_BASE, "# 三引号串里的行", "# 三引号串里改了"))),
        (".py 改代码", differ("scripts/sample.py", PYTHON_BASE, edited(PYTHON_BASE, "len(sys.argv)", "len(sys.path)"))),
        (".py tokenize 切不了的取原文：只改注释也不同", differ("scripts/unparsable.py", "x = (\n# 一\n", "x = (\n# 二\n")),
        ("crates/ 下没人读的 .md 不进指纹", left_out("crates/demo/notes.md")),
        ("crates/ 下被 #[doc = include_str!] 读的 .md 取原文：改一个字就不同",
         differ("crates/demo/src/doc.md", "说明\n", "说明改了\n", [(rust, '#[doc = include_str!("doc.md")]\npub fn one() {}\n')] + cargo)),
        ("crates/ 外的 .md 取原文：改一个字就不同", differ("docs/guide.md", "说明\n", "说明改了\n")),
        ("Cargo.toml 只改 # 注释也不同（取原文）", differ("crates/demo/Cargo.toml", '[package]\n# 一\nname = "demo"\n', '[package]\n# 二\nname = "demo"\n')),
        (".wgsl 只改 // 注释也不同（取原文）", differ("crates/demo/src/k.wgsl", "// 一\nfn f() {}\n", "// 二\nfn f() {}\n")),
        ("取原文的摘要与 sha256sum 相同", lambda: digests_of([("Cargo.lock", b"lock\n")])[0] == ("raw", hashlib.sha256(b"lock\n").hexdigest())),
    ]


BREAK_CELLS = [  # (弄坏开关, 打开后该转红的那几格)
    ("rust-raw", (".rs 加一行 // 注释", ".rs 改文档注释措辞", ".rs 改块注释与缩进")),
    ("literals-dropped", (".rs 改一个字面量",)),
    ("safety-text-ignored", (".rs 改 SAFETY 注释的正文",)),
    ("doc-code-ignored", (".rs 改文档注释里的代码块",)),
    ("include-target-lexical", ("被 include_str! 读的 .rs 只改注释，摘要不同（读它的拿到的是原文）",
                                "include_str! 的参数是算出来的：同一个 crate 下的 .rs 只改注释也取原文、摘要不同")),
    ("shell-raw", (".sh 只改注释与空行",)),
    ("shell-naive-comments", (".sh here-doc 里 # 开头的行改了", ".sh 引号里的 # 后面改了")),
    ("python-raw", (".py 只改注释与空行",)),
    ("python-naive-comments", (".py 字符串里的 # 后面改了", ".py 三引号串里 # 开头的行改了")),
    ("docs-kept-in-crates", ("crates/ 下没人读的 .md 不进指纹",)),
    ("docs-dropped-everywhere", ("crates/ 外的 .md 取原文：改一个字就不同",)),
]


def with_break(name, function):
    saved = os.environ.get(BREAK_VARIABLE)
    os.environ[BREAK_VARIABLE] = name
    try:
        return function()
    finally:
        if saved is None:
            del os.environ[BREAK_VARIABLE]
        else:
            os.environ[BREAK_VARIABLE] = saved


def cache_cells(work):
    """[(格名, 对没对, 明细)]：再算一次时从缓存取、取出来的与现算的相同；缓存行被改坏了不认。"""
    cells = []
    path = os.path.join(work, CACHE_FILE_NAME)
    entries = [("crates/demo/src/lib.rs", RUST_BASE.encode()), ("scripts/sample.sh", SHELL_BASE.encode()), ("scripts/sample.py", PYTHON_BASE.encode())]
    fresh = digests_of(entries)
    writing_cache = DigestCache(path)
    written = digests_of(entries, writing_cache)
    reading_cache = DigestCache(path)
    read_back = digests_of(entries, reading_cache)
    cells.append(("缓存：再算一次时三份都从缓存取，取出来的与不用缓存现算的相同",
                  written == fresh and read_back == fresh and reading_cache.hits == 3,
                  f"现算 {fresh}，记缓存那一次 {written}，从缓存取那一次 {read_back}（命中 {reading_cache.hits} 份）"))
    with open(path, encoding="utf-8") as handle:
        lines = handle.read().split("\n")
    forged = []
    for line in lines:
        fields = line.split("\t")
        if len(fields) == 5 and fields[1] == "python":
            fields[3] = json.dumps({"f": "python", "d": "0" * 64}, separators=(",", ":"))
        forged.append("\t".join(fields))
    with open(path, "w", encoding="utf-8") as handle:
        handle.write("\n".join(forged))
    after_forgery = digests_of(entries, DigestCache(path))
    python_position = 2
    cells.append(("缓存行被改坏了不认（校验对不上，照样现算）", after_forgery == fresh,
                  f"现算 {fresh[python_position]}，改坏缓存之后取到 {after_forgery[python_position]}"))
    return cells


def tree_cells(work):
    """[(格名, 对没对, 明细)]：临时仓里两棵树按摘要比。"""
    identity = {"GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@t", "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@t"}
    environment = dict(os.environ, **identity)
    repository = os.path.join(work, "repository")

    def run(*arguments):
        return subprocess.run(["git", "-C", repository, *arguments], capture_output=True, text=True, env=environment, check=True).stdout.strip()

    def write(relative, text, executable=False):
        path = os.path.join(repository, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
        os.chmod(path, 0o755 if executable else 0o644)

    def snapshot():
        run("add", "-A")
        return run("write-tree")
    os.makedirs(repository)
    run("init", "-q", "-b", "master")
    write("crates/demo/Cargo.toml", '[package]\nname = "demo"\n')
    write("crates/demo/src/lib.rs", RUST_BASE + 'pub const TABLE: &str = include_str!("table.rs");\n')
    write("crates/demo/src/table.rs", "pub const ROWS: u32 = 3; // 行数\n")
    write("scripts/sample.sh", SHELL_BASE)
    base = snapshot()
    write("crates/demo/src/lib.rs", edited(RUST_BASE, "    1\n", "    // 新加的一行注释\n    1\n") + 'pub const TABLE: &str = include_str!("table.rs");\n')
    write("scripts/sample.sh", edited(SHELL_BASE, "# 样本说明", "# 改过的说明"))
    write("crates/demo/notes.md", "没人读的说明\n")
    comments_only = snapshot()
    write("crates/demo/src/lib.rs", edited(RUST_BASE, "    1\n", "    2\n") + 'pub const TABLE: &str = include_str!("table.rs");\n')
    literal = snapshot()
    write("crates/demo/src/lib.rs", RUST_BASE + 'pub const TABLE: &str = include_str!("table.rs");\n')
    write("crates/demo/src/table.rs", "pub const ROWS: u32 = 3; // 行数，改了措辞\n")
    write("scripts/sample.sh", SHELL_BASE)
    write("crates/demo/notes.md", "没人读的说明\n")
    included = snapshot()
    write("crates/demo/src/table.rs", "pub const ROWS: u32 = 3; // 行数\n")
    write("scripts/sample.sh", SHELL_BASE, executable=True)
    mode = snapshot()
    pathspecs = ["crates/", "scripts/"]
    cells = []
    for label, tree, want_changed, want_same in (
            ("两棵树只差注释（.rs 加一行注释、sample.sh 改一行注释、crates/ 下新加一份没人读的 .md）：没有变了的", comments_only, [],
             ["crates/demo/notes.md", "crates/demo/src/lib.rs", "scripts/sample.sh"]),
            ("两棵树差一个字面量：lib.rs 算变了", literal, ["crates/demo/src/lib.rs"], ["crates/demo/notes.md", "scripts/sample.sh"]),
            ("被 include_str! 读的 table.rs 只改注释：算变了", included, ["crates/demo/src/table.rs"], ["crates/demo/notes.md"]),
            ("脚本只改了执行位：算变了", mode, ["scripts/sample.sh"], ["crates/demo/notes.md"])):
        got = tree_differences(repository, base, tree, pathspecs)
        cells.append((f"tree-diff：{label}", got == (want_changed, want_same), f"要 {(want_changed, want_same)}，实际 {got}"))
    excluded = tree_differences(repository, base, literal, ["crates/", ":(exclude)crates/demo/src/lib.rs", "scripts/"])
    cells.append(("tree-diff：认 git 路径限定的 :(exclude)（减掉的 lib.rs 不进比对）", excluded == ([], ["crates/demo/notes.md", "scripts/sample.sh"]),
                  f"实际 {excluded}"))
    return cells


def selftest():
    failures, checked, names = [], 0, []
    cells = content_cells()
    by_name = dict(cells)
    for name, judge in cells:
        checked += 1
        names.append(name)
        try:
            ok, detail = judge(), "判反了"
        except DigestError as error:
            ok, detail = False, f"算不出：{error}"
        print(f"  {'✓' if ok else '✗'} {name}" + ("" if ok else f"：{detail}"))  # gate-lint:detail
        if not ok:
            failures.append(name)
    for switch, red_cells in BREAK_CELLS:
        checked += 1
        flipped = [cell for cell in red_cells if not with_break(switch, by_name[cell])]
        ok = flipped == list(red_cells)
        label = f"弄坏开关 {switch} 下「{'」「'.join(red_cells)}」转红"
        print(f"  {'✓' if ok else '✗'} {label}" + ("" if ok else f"：只红了 {flipped}"))  # gate-lint:detail
        if not ok:
            failures.append(label)
    with tempfile.TemporaryDirectory(prefix="source-digest-selftest-") as work:
        extra = cache_cells(work) + tree_cells(work)
        unchecked_work = os.path.join(work, "unchecked-cache")
        os.makedirs(unchecked_work)
        cache_red = with_break("cache-unchecked", lambda: cache_cells(unchecked_work))
        forgery_cell = next(index for index, cell in enumerate(cache_red) if cell[0].startswith("缓存行被改坏了不认"))
        extra.append(("弄坏开关 cache-unchecked 下「缓存行被改坏了不认」转红", not cache_red[forgery_cell][1], f"弄坏之后照样判对：{cache_red[forgery_cell][2]}"))
    for name, ok, detail in extra:
        checked += 1
        print(f"  {'✓' if ok else '✗'} {name}" + ("" if ok else f"：{detail}"))  # gate-lint:detail
        if not ok:
            failures.append(name)
    if failures:
        print(f"  ✗ lib_source_digest.py 自证没过：{len(failures)} 格判错（共 {checked} 格）")
        print("    → 怎么办：照上面每一格改 digests_of / tree_differences 里对应的那一段；SOURCE_DIGEST_BREAK 设着的话点名的那几格本来就该红")
        return 1
    print(f"  ✓ lib_source_digest.py 自证通过：{checked} 格都对（.rs 注释、文档措辞、块注释与缩进、字面量、SAFETY 正文与标记、doctest、include_str! 读的与算出来的参数、"
          ".sh 注释、命令、here-doc、引号、admission 注释、解析不过，.py 注释、字符串、三引号、代码、切不了，.md 在 crates/ 内外，Cargo.toml、.wgsl 取原文，"
          "缓存与改坏的缓存行，两棵树按摘要比、执行位、:(exclude)；弄坏开关 " + "、".join(switch for switch, _cells in BREAK_CELLS)
          + "、cache-unchecked 下各自那几格转红）")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    try:
        if len(arguments) >= 3 and arguments[0] == "digest":
            root = arguments[1]
            results = working_tree_digests(root, arguments[2:], cache_for(root))
            for name, (form, digest) in zip(arguments[2:], results):
                print(f"{form or '-'}\t{digest or '-'}\t{name}")
            return 0
        if len(arguments) >= 6 and arguments[0] == "tree-diff" and arguments[4] == "--":
            root = arguments[1]
            changed, same = tree_differences(root, arguments[2], arguments[3], arguments[5:], cache_for(root))
            for name in changed:
                print(f"changed\t{name}")
            for name in same:
                print(f"same\t{name}")
            return 0
    except (DigestError, OSError) as error:
        print(f"  ✗ 算不出摘要：{error}", file=sys.stderr)
        print("    → 怎么办：看上一句点的是哪一样（规范副本、bash、git、读不了的文件），补齐再跑；门禁按要跑处置，不当没变", file=sys.stderr)
        return 3
    print("  ✗ 用法：lib_source_digest.py digest <根> <路径>… | tree-diff <根> <旧树> <新树> -- <路径…> | --selftest", file=sys.stderr)
    print("    → 怎么办：照文件头「用法」一段给参数", file=sys.stderr)
    return 2


if __name__ == '__main__':
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
