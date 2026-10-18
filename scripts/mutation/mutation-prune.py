#!/usr/bin/env python3
# admission: always 只读给它的源码文本与覆盖输出现算，不记上一次的结果；--selftest 判的是此刻这份规则
# run-condition: command python3
"""广谱变异驱动（scripts/mutation/broad-mutation.py）的剪枝规则库：每条变异的单元键、结论能不能沿用、按覆盖挑哪些测试、清单按单元存。

依据：用户 2026-10-15 定（剪枝的单元是函数；广谱变异一条一条按内容算，不按 HEAD 算）；复用键普查对账底稿第一节 L22 与第二节第 6、9 条、
一之三 verify 路的改写；第二轮 design 路报告第三节 3.1；controls 路报告第 5 节（S34、T16、N37、N45、T17 与弄坏开关名）。
清单与结论都按单元键存：每条变异过沿用判据，沿用不了才现判；不带参数就是剪枝，--full 只写不读（驱动那一头）。
没有 --changed-since、--sync-tree、changed-since-tree、notice_if_head_moved，没有「一版」与 git archive <提交> 导出的版本树。
函数外的项各切单元（const、static、struct、impl 头、宏都是单元），切不开当解析缺陷修，不退整文件。判法是判结局那几个函数单元的指纹，没有手写的版本号。

  mutation-prune.py --selftest            自证：controls 路的 S34、T16、N37、N45、T17，verify 路量过的 index_node_entry_capacity 那一格，
                                          与本库自己加的几格（见 SAMPLES），逐个判沿用的决定；另有清单按单元存、跑全部的例外、覆盖文件名归一、
                                          -E 按块切、CPU 限时倍数、runner；不起 cargo，runner 那几格起一个假测试程序
  mutation-prune.py --index <树> [文件…]   打每个文件的单元（种类、限定名、起止行、指纹前 12 位），排查用

单元与闭包都由 scripts/gate/lib_function_units.py 算（Rust 那一份；本库只 import 它，不另切）：
  一条变异的容器 = 包住它的最小的单元（fn、impl 头、const、static、struct、enum、宏……；嵌套 fn 与闭包算外层 fn 的）。
  变异键（verdict key）= sha256（form 一行、容器单元闭包的键、容器里的相对位置、原文、替换文）；闭包的键是 lib_function_units 的 closure_key：
    入口目标的生效 cfg、闭包里每个单元的（种类，指纹）多重集、数据单元的摘要，加调用方给的工具链行（global_lines）与依赖行（dependency_lines）。
    相对位置按容器里去掉注释之后的词法单元下标与单元内偏移记（注释与空白不算）；不带路径、名字、行号、上限、限时。
  依赖行：闭包里的单元真点名的外部 crate 照 Cargo.lock 落到的包（名、版本、来源、checksum），连同传递依赖；取法只在
    scripts/mutation/lib_mutation_units.py 一处（它文件头「依赖版本行」，crates 变异表与 mutate.sh 用同一份）。
  工具链行（驱动给）：rustc -vV、cargo nextest / cargo-mutants / llvm-profdata 的版本、构建环境的生效值（build_environment_lines）、判法单元指纹。
    Cargo.toml 只取解析后的 [profile]、各包的 edition 与生效特性（特性在闭包的 cfg 行里），不取原文；.cargo/ 下的配置取解析后的值；排除表不进。
  测试键 = 抓到它那条测试的单元闭包键（入口是那条测试函数）；认不出是哪条测试、或闭包拒算，交 None 与原因（那一条现判，不退整包、不退只比测试函数）。
  覆盖者键 = 静态闭包里有容器单元的每一条测试（第一段的、第二段慢档用例、checker 档的、被变异包的文档测试，排除表里的不算）各自测试键的多重集：
    闭包是那条测试执行得到的单元的上界，集合与各自的键不变、也没有新测试的闭包碰到它，没红就沿用。
沿用判据（carry_decision）：键相同的那一行；抓到的另要测试键与现算的相同、那条测试此刻不在排除表里（升档判的抓到带档位，另要覆盖者键相同：超时、撞顶是在碰得到它的那几条测试跑着时出的，它们变了、或有新测试碰到它就现判）；
  没红的要覆盖者键相同；无效的键相同就沿用（键里有容器与它点到的单元、依赖、工具链）；超时、内存撞顶、驱动错不是结论，照选中现判；
  拒算的（容器或它的闭包里有认不出的形态）不沿用，每趟现判。
清单按单元存（list_unit_key）：cargo-mutants 给的每一条变异记在它的容器单元下（相对位置、原文、替换文、变异名去掉位置的那一段），
  键 = cargo-mutants 版本、.cargo/mutants.toml 解析后的值、文件路径（examine_globs 按路径挑文件）、单元的种类与指纹、cfg 条件链、外层单元的指纹；
  被变异包里每一个单元都记（没有变异的记空表），提交门禁照这张表在此刻的源码上复原清单，不起 cargo。

规则（每条后面是弄坏开关 BROAD_MUTATION_BREAK=<项>，只在 --selftest 与驱动的自证里生效）：
  catching-test-own-function-only   测试键只取测试函数自己的指纹，不取它的闭包                    T16（「✗ 自证：样本 T16 判成 carry」）
  mutated-function-only             变异键只取容器单元自己的指纹，不取它的闭包（依赖行跟着不进）  INDEXU、MACRO、S04、Y8、FC、FD、Y9、Q1、DEP（判成 carry）
  container-fingerprint-dropped     变异键里的闭包去掉容器单元自己                                S34U、Y5、FF（判成 carry）
    抓到的结论另有测试键守着（测试的闭包里有被测函数与它调到的），S34、INDEX 要同开两个开关才红：
    container-fingerprint-dropped,catching-test-own-function-only 红在 S34，mutated-function-only,catching-test-own-function-only 红在 INDEX
  defining-file-in-key              变异键另收闭包碰到的每份文件里全部单元的指纹                  N37、N45、T17、FA（判成 rejudge）
  relative-position-dropped         相对位置不进变异键                                            TWO（判成 carry）
  relative-position-counts-comments 相对位置连注释一起数                                          COMMENT（判成 rejudge）、清单那一格（「✗ 自证：清单的一条没复原回去」）
  dependency-lines-dropped          依赖行不进键                                                  DEP（判成 carry）
  dependency-whole-lock             依赖行取整份 Cargo.lock 的全部包                              DEP-OTHER（判成 rejudge）
  profile-ignored                   构建环境不取 Cargo.toml 解析后的 [profile]                    Y7（判成 carry）
  manifest-text-in-global-key       构建环境另收 Cargo.toml 原文的 sha256                         MANIFEST（判成 rejudge）
  exclusions-in-global-key          工具链行另收排除表的 sha256                                   EXCLUDE-OTHER（判成 rejudge）
  catching-test-exclusion-ignored   抓到它的测试进了排除表也沿用                                  EXCLUDE-CATCHER（判成 carry）
  unviable-rejudged                 无效的条键相同也现判                                          UNVIABLE（判成 rejudge）
  missed-coverers-ignored           没红的条不比覆盖者键                                          N1（判成 carry）
  missed-always-rejudged            没红的条键相同也现判                                          MISSED（判成 rejudge）
  escalated-not-carried             升档判的抓到键相同也现判                                      ESCALATED（判成 rejudge）
  escalated-coverers-ignored        升档判的抓到不比覆盖者键                                      ESCALATED-COVERERS（判成 carry）
  list-cache-context-dropped        清单的单元键不收 cfg 条件链                                   清单那一格（「✗ 自证：单元挪进 #[cfg(test)] 之后清单的单元键没变」）
  outside-mutation-selected         不在 fn 单元里的变异也按覆盖挑测试                            K1（「✗ 自证：K1：…没判成跑全部」）
  const-context-runs-selected       常量上下文里用到的 const fn 也按覆盖挑测试                    X3（「✗ 自证：X3：…没判成跑全部」）
  spawning-binary-selected          起子进程的测试二进制一个都不认                                X1b（「✗ 自证：X1b：…没认出来」）
  coverage-paths-unnormalized       覆盖文件名不拼不截                                            覆盖文件名归一（「✗ 自证：覆盖文件名没归一或行列没对上」）
  nextest-filter-single-argument    挑测试的 -E 拼成一个参数                                      -E 按块切（「✗ 自证：挑测试的 -E 没按块切」）
  limit-below-smt                   不核 CPU 限时倍数的下限                                       CPU 限时倍数（「✗ 自证：CPU 限时的倍数或上限算错」）
  cpu-limit-ignored                 RUNNER_SCRIPT 在 limit 模式不设 CPU 上限、只看墙钟             runner（「✗ 自证：runner 的限时不对」）
  弄坏开关打开后本库 --selftest 退 1，汇总行「✗ mutation-prune.py --selftest：<格数> 格里 N 格判错」；驱动 broad-mutation.py 的 --selftest 打开同一个开关时红在哪写在它自己的文件头。

gate-similar: broad-mutation.py 它是逐条驱动（库目录、续跑、逐步命令），这里是它载入的纯函数库，判的是同一件事的剪枝那一半，分开是为了自证不碰库目录
gate-similar: lib_function_units.py 它切单元、收闭包、算闭包键；这里只调它，加上广谱变异自己的那几样（容器、相对位置、测试键、覆盖者键、清单的单元键、依赖行）
"""
import hashlib
import importlib.util
import json
import os
import re
import sys
import tomllib

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY, ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402

BREAKS = frozenset()    # 驱动在自证里设；本库 --selftest 自己从 BROAD_MUTATION_BREAK 读
UNITS_LIBRARY_PATH = os.path.join(REPOSITORY, "scripts", "gate", "lib_function_units.py")
MUTATION_UNITS_LIBRARY_PATH = os.path.join(REPOSITORY, "scripts", "mutation", "lib_mutation_units.py")   # 依赖版本行与 crates 变异表、mutate.sh 共用这一份
VERDICT_KEY_FORM = "broad-mutation-verdict-v1"
LIST_KEY_FORM = "broad-mutation-list-v1"
NEXTEST_FILTER_CHUNK_BYTES = 32768
SINGLE_ARGUMENT_LIMIT = 131072
MINIMUM_CPU_MULTIPLIER = 2.0
DEFAULT_CPU_MULTIPLIER = 3.0       # 盖住 SMT 2.0 倍 × 12 线程散到 1.26 倍（第三轮攻方 c4-cpu-time.txt）
DEFAULT_CPU_FLOOR_SECONDS = 30.0
DEFAULT_WALL_MULTIPLIER = 10.0
DEFAULT_WALL_FLOOR_SECONDS = 300.0
ESCALATED_LABELS = ("caught-by-timeout", "caught-by-memory")
SPAWN_PATTERNS = re.compile(r"\bcurrent_exe\b|CARGO_BIN_EXE_|\bCommand::new\b|process::Command\b")
# 构建环境里取生效值的环境变量（名字逐个、前缀一族）：编译器、rustdoc 的标志，cargo 的 build / profile / target 配置，换编译器的那几个
BUILD_VARIABLES = ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS", "RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER",
                   "RUSTDOC")
BUILD_VARIABLE_PREFIXES = ("CARGO_BUILD_", "CARGO_PROFILE_", "CARGO_TARGET_")
PROGRAM_VARIABLES = ("RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTDOC")
CARGO_CONFIG_TABLES = ("build", "profile", "target", "env", "unstable")
CARGO_CONFIG_PROGRAMS = (("build", "rustc"), ("build", "rustc-wrapper"), ("build", "rustc-workspace-wrapper"), ("build", "rustdoc"))


def broken(name):
    return name in BREAKS


def digest_of(parts):
    return hashlib.sha256("\x1f".join(parts).encode("utf-8", "surrogateescape")).hexdigest()


def digest_of_lines(lines):
    return hashlib.sha256("".join(line + "\n" for line in lines).encode("utf-8", "surrogateescape")).hexdigest()


UNITS_MODULE = None


def units_library():
    """scripts/gate/lib_function_units.py（切单元、收闭包、算闭包键只认它一份）。"""
    global UNITS_MODULE
    if UNITS_MODULE is None:
        specification = importlib.util.spec_from_file_location("lib_function_units_for_mutation_prune", UNITS_LIBRARY_PATH)
        UNITS_MODULE = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(UNITS_MODULE)
    return UNITS_MODULE


MUTATION_UNITS_MODULE = None


def mutation_units_library():
    """scripts/mutation/lib_mutation_units.py（依赖版本行只认它一份：dependency_lines、read_lock_packages、lock_line）。"""
    global MUTATION_UNITS_MODULE
    if MUTATION_UNITS_MODULE is None:
        specification = importlib.util.spec_from_file_location("lib_mutation_units_for_mutation_prune", MUTATION_UNITS_LIBRARY_PATH)
        MUTATION_UNITS_MODULE = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(MUTATION_UNITS_MODULE)
    return MUTATION_UNITS_MODULE


# ── 切词：只给「容器里的相对位置」与「原文」用。注释（连文档注释）与空白去掉，串、原始串、字符、生命周期各一个单元 ────────────────

def identifier_start(character):
    return character == "_" or character.isalpha()


def identifier_continue(character):
    return character == "_" or character.isalnum()


def tokenize(text, keep_comments=False, documents=True):
    """[(类, 文字, 起, 止)]，起止是字符下标。类：ident、lifetime、literal、punct；documents 时文档注释另成 doc（旧切法 FileIndex 要它：
    文档注释是属性、带着文档测试），keep_comments 时每段注释（连文档注释）另成 comment；两样都不要时注释一律去掉（变异的相对位置用这一种）。"""
    tokens, length, position = [], len(text), 0

    def end_quoted(start, quote):
        index = start + 1
        while index < length:
            if text[index] == "\\":
                index += 2
                continue
            if text[index] == quote:
                return index + 1
            index += 1
        return length

    def end_raw(start):
        index, hashes = start, 0
        while index < length and text[index] == "#":
            hashes += 1
            index += 1
        if index >= length or text[index] != '"':
            return None
        closing = '"' + "#" * hashes
        found = text.find(closing, index + 1)
        return length if found < 0 else found + len(closing)

    while position < length:
        current = text[position]
        following = text[position + 1] if position + 1 < length else ""
        if current.isspace():
            position += 1
            continue
        if current == "/" and following == "/":
            end = text.find("\n", position)
            end = length if end < 0 else end
            body = text[position:end].rstrip()
            if keep_comments:
                tokens.append(("comment", text[position:end], position, end))
            elif documents and ((body.startswith("///") and not body.startswith("////")) or body.startswith("//!")):
                tokens.append(("doc", body, position, end))
            position = end
            continue
        if current == "/" and following == "*":
            depth, index = 0, position
            while index < length:
                if text.startswith("/*", index):
                    depth, index = depth + 1, index + 2
                elif text.startswith("*/", index):
                    depth, index = depth - 1, index + 2
                    if depth == 0:
                        break
                else:
                    index += 1
            body = text[position:index]
            if keep_comments:
                tokens.append(("comment", body, position, index))
            elif documents and ((body.startswith("/**") and not body.startswith("/***") and body != "/**/") or body.startswith("/*!")):
                tokens.append(("doc", body, position, index))
            position = index
            continue
        if current == '"':
            end = end_quoted(position, '"')
            tokens.append(("literal", text[position:end], position, end))
            position = end
            continue
        if current == "'":
            if following == "\\":
                end = end_quoted(position, "'")
                tokens.append(("literal", text[position:end], position, end))
            elif position + 2 < length and text[position + 2] == "'":
                end = position + 3
                tokens.append(("literal", text[position:end], position, end))
            else:
                end = position + 1
                while end < length and identifier_continue(text[end]):
                    end += 1
                tokens.append(("lifetime", text[position:end], position, end))
            position = end
            continue
        if identifier_start(current):
            end = position + 1
            while end < length and identifier_continue(text[end]):
                end += 1
            word = text[position:end]
            if end < length and word in ("r", "br", "cr") and text[end] in '#"':
                if word == "r" and text[end] == "#" and end + 1 < length and identifier_start(text[end + 1]):
                    close = end + 1
                    while close < length and identifier_continue(text[close]):
                        close += 1
                    tokens.append(("ident", text[position:close], position, close))
                    position = close
                    continue
                close = end_raw(end)
                if close is not None:
                    tokens.append(("literal", text[position:close], position, close))
                    position = close
                    continue
            if end < length and word in ("b", "c") and text[end] == '"':
                close = end_quoted(end, '"')
                tokens.append(("literal", text[position:close], position, close))
                position = close
                continue
            if end < length and word == "b" and text[end] == "'":
                close = end_quoted(end, "'")
                tokens.append(("literal", text[position:close], position, close))
                position = close
                continue
            tokens.append(("ident", word, position, end))
            position = end
            continue
        if current.isdigit():
            end = position + 1
            while end < length and identifier_continue(text[end]):
                end += 1
            if end + 1 < length and text[end] == "." and text[end + 1].isdigit():
                end += 1
                while end < length and identifier_continue(text[end]):
                    end += 1
            tokens.append(("literal", text[position:end], position, end))
            position = end
            continue
        tokens.append(("punct", current, position, position + 1))
        position += 1
    return tokens


def match_pairs(tokens):
    """括号配对：{开的下标: 关的下标}；配不上交 None。"""
    pairs, stack = {}, []
    closers = {")": "(", "]": "[", "}": "{"}
    for index, token in enumerate(tokens):
        if token[0] != "punct":
            continue
        if token[1] in "([{":
            stack.append(index)
        elif token[1] in ")]}":
            if not stack or tokens[stack[-1]][1] != closers[token[1]]:
                return None
            pairs[stack.pop()] = index
    return None if stack else pairs


# ── 按文件切 fn 项（只留给 scripts/crash/cpu-judging-sites-check.py 认判器调用点用：它读 tokens、text 与 functions；广谱变异的容器、键与覆盖都不用它） ──

class Function:
    __slots__ = ("qualified", "name", "first", "last", "start", "end", "fingerprint", "is_const", "body_first", "module_path")

    def __init__(self, qualified, name, first, last, start, end, is_const, body_first, module_path):
        self.qualified, self.name, self.first, self.last = qualified, name, first, last
        self.start, self.end, self.is_const, self.body_first, self.module_path = start, end, is_const, body_first, module_path
        self.fingerprint = ""


class FileIndex:
    """一个 .rs 文件按 fn 项切开（tokens、functions）。ok 为假：括号配不上，一个 fn 项都不交。"""

    def __init__(self, path, text):
        self.path, self.text = path, text
        self.tokens = tokenize(text)
        self.line_starts = [0] + [index + 1 for index, character in enumerate(text) if character == "\n"]
        self.functions, self.macro_definitions, self.ok = [], set(), True
        self.pairs = self.match_groups()
        if self.ok:
            try:
                self.scan(0, len(self.tokens), [], [])
            except (KeyError, IndexError):
                self.ok, self.functions = False, []
        self.disambiguate()
        for function in self.functions:
            function.fingerprint = self.function_fingerprint(function)

    def text_of(self, index):
        return self.tokens[index][1] if index < len(self.tokens) else ""

    def match_groups(self):
        pairs, stack = {}, []
        closers = {")": "(", "]": "[", "}": "{"}
        for index, token in enumerate(self.tokens):
            if token[0] != "punct":
                continue
            if token[1] in "([{":
                stack.append(index)
            elif token[1] in ")]}":
                if not stack or self.tokens[stack[-1]][1] != closers[token[1]]:
                    self.ok = False
                    return {}
                opener = stack.pop()
                pairs[opener] = index
        if stack:
            self.ok = False
        return pairs

    def function_fingerprint(self, function):
        return digest_of([token[1] for token in self.tokens[function.first:function.last]])

    def disambiguate(self):
        seen = {}
        for function in self.functions:
            seen[function.qualified] = seen.get(function.qualified, 0) + 1
            if seen[function.qualified] > 1:
                function.qualified += f"#{seen[function.qualified]}"

    def skip_group_or_token(self, index):
        return self.pairs[index] + 1 if index in self.pairs else index + 1

    def scan(self, low, high, context, module_path):
        index = low
        while index < high:
            item_start = index
            while index < high:
                token = self.tokens[index]
                if token[0] == "doc":
                    index += 1
                elif token[1] == "#" and self.text_of(index + 1) == "[":
                    index = self.pairs[index + 1] + 1
                elif token[1] == "#" and self.text_of(index + 1) == "!" and self.text_of(index + 2) == "[":
                    index = self.pairs[index + 2] + 1
                    item_start = index
                else:
                    break
            if index >= high:
                return
            keyword_at = index
            if self.text_of(keyword_at) == "pub":
                keyword_at += 1
                if self.text_of(keyword_at) == "(":
                    keyword_at = self.pairs[keyword_at] + 1
            is_const = False
            while keyword_at < high:
                word, following = self.text_of(keyword_at), self.text_of(keyword_at + 1)
                if word == "const" and following in ("fn", "unsafe", "async", "extern"):
                    is_const = True
                    keyword_at += 1
                elif word in ("async", "default", "safe") and following in ("fn", "unsafe", "async", "extern", "const", "impl"):
                    keyword_at += 1
                elif word == "unsafe" and following in ("fn", "extern", "impl", "trait", "async"):
                    keyword_at += 1
                elif word == "extern" and following == "fn":
                    keyword_at += 1
                elif word == "extern" and self.tokens[keyword_at + 1][0] == "literal" and self.text_of(keyword_at + 2) in ("fn", "unsafe"):
                    keyword_at += 2
                else:
                    break
            keyword = self.text_of(keyword_at)
            if keyword == "fn" and keyword_at + 1 < high:
                name = self.text_of(keyword_at + 1)
                cursor = keyword_at + 2
                body_first = None
                while cursor < high:
                    text = self.text_of(cursor)
                    if text in ("(", "["):
                        cursor = self.pairs[cursor] + 1
                        continue
                    if text == "{":
                        body_first = cursor
                        cursor = self.pairs[cursor] + 1
                        break
                    if text == ";":
                        cursor += 1
                        break
                    cursor += 1
                self.functions.append(Function("::".join(context + [name]), name, item_start, cursor, self.tokens[item_start][2],
                                               self.tokens[cursor - 1][3], is_const, body_first, list(module_path)))
                if body_first is not None:
                    # 函数体里的嵌套 fn 也记下（容器仍取最外层的，见 container_at；只有弄坏开关 innermost-function-key 取它）
                    self.scan(body_first + 1, self.pairs[body_first], context + [name], module_path)
                index = cursor
                continue
            if keyword in ("impl", "trait") or (keyword == "auto" and self.text_of(keyword_at + 1) == "trait") or \
                    (keyword == "extern" and (self.text_of(keyword_at + 1) == "{" or self.text_of(keyword_at + 2) == "{")):
                brace = keyword_at
                while brace < high and self.text_of(brace) not in ("{", ";"):
                    brace = self.skip_group_or_token(brace) if self.text_of(brace) in ("(", "[") else brace + 1
                if brace >= high or self.text_of(brace) == ";":
                    index = brace + 1
                    continue
                label = "<" + " ".join(self.text_of(position) for position in range(keyword_at, brace)) + ">"
                self.scan(brace + 1, self.pairs[brace], context + [label], module_path)
                index = self.pairs[brace] + 1
                continue
            if keyword == "mod" and self.text_of(keyword_at + 2) == "{":
                name = self.text_of(keyword_at + 1)
                self.scan(keyword_at + 3, self.pairs[keyword_at + 2], context + [name], module_path + [name])
                index = self.pairs[keyword_at + 2] + 1
                continue
            if keyword == "macro_rules" and self.text_of(keyword_at + 1) == "!":
                self.macro_definitions.add(self.text_of(keyword_at + 2))
                opener = keyword_at + 3
                index = self.pairs[opener] + 1 if opener in self.pairs else opener + 1
                if self.text_of(index) == ";":
                    index += 1
                continue
            cursor = keyword_at
            braced_item = keyword in ("struct", "enum", "union")
            while cursor < high:
                text = self.text_of(cursor)
                if text in ("(", "[", "{") and cursor in self.pairs:
                    close = self.pairs[cursor]
                    if text == "{" and (braced_item or self.text_of(cursor - 1) == "!"):
                        cursor = close + 1
                        if self.text_of(cursor) == ";":
                            cursor += 1
                        break
                    cursor = close + 1
                    continue
                cursor += 1
                if text == ";":
                    break
            index = max(cursor, index + 1)


# ── 一棵树的单元：lib_function_units 的 RustWorkspace 加广谱变异要的几样 ─────────────────────────────────────────

class UnitTree:
    """一棵树（库目录里的专用树、或提交门禁判的工作树）上的单元、闭包与各种键，按单元缓存。"""

    def __init__(self, root):
        self.root = os.path.abspath(root)
        self.library = units_library()
        self.workspace = self.library.RustWorkspace(self.root)
        self.files = {rust_file.path: rust_file for rust_file in self.workspace.all_files()}
        self.texts, self.tokens, self.line_starts = {}, {}, {}
        self.closures, self.dependency_cache, self.test_keys = {}, {}, {}
        self.lock = None
        self.const_context = None

    # 原文与切词

    def text(self, path):
        if path not in self.texts:
            with open(os.path.join(self.root, path), encoding="utf-8", errors="surrogateescape") as handle:
                self.texts[path] = handle.read()
            text = self.texts[path]
            self.line_starts[path] = [0] + [index + 1 for index, character in enumerate(text) if character == "\n"]
        return self.texts[path]

    def file_tokens(self, path):
        keep = broken("relative-position-counts-comments")
        if (path, keep) not in self.tokens:
            self.tokens[(path, keep)] = tokenize(self.text(path), keep_comments=keep, documents=False)
        return self.tokens[(path, keep)]

    def offset(self, path, line, column):
        """1 起的行、列（列按字符）→ 字符下标。"""
        self.text(path)
        return self.line_starts[path][line - 1] + column - 1

    def line_column(self, path, offset):
        self.text(path)
        starts = self.line_starts[path]
        line = max(index for index, start in enumerate(starts) if start <= offset)
        return line + 1, offset - starts[line] + 1

    def offset_of_byte_column(self, path, line, column):
        """llvm-cov 的列按字节：换成字符下标。"""
        text = self.text(path)
        starts = self.line_starts[path]
        start = starts[line - 1]
        end = starts[line] - 1 if line < len(starts) else len(text)
        prefix = text[start:end].encode("utf-8")[:max(0, column - 1)]
        return start + len(prefix.decode("utf-8", "ignore"))

    # 单元

    def code_units(self, path):
        """一份文件里能当容器的单元（不算数据单元与文档测试）。"""
        rust_file = self.files.get(path)
        if rust_file is None:
            return []
        return [unit for unit in rust_file.units if not unit.data and unit.kind != "doctest"]

    def container(self, path, start, end):
        """包住 [起, 止)（各是 (行, 列)）的最小单元；找不到交 None（切不开：当解析缺陷修，不退整文件）。"""
        candidates = [unit for unit in self.code_units(path) if unit.first_line <= start[0] and end[0] <= unit.last_line]
        if not candidates:
            return None
        return min(candidates, key=lambda unit: ((unit.last_token or 0) - (unit.first_token or 0), unit.last_line - unit.first_line))

    def unit_start_offset(self, unit):
        """单元第一个词法单元在原文里的字符下标：单元起行上、与 lib 切出的第一个词法单元同文字的那一个。"""
        rust_file = self.files[unit.file]
        first_text = rust_file.texts[unit.first_token] if unit.first_token is not None else ""
        line_start = self.offset(unit.file, unit.first_line, 1)
        line_end = self.offset(unit.file, unit.first_line + 1, 1) if unit.first_line < len(self.line_starts[unit.file]) else len(self.text(unit.file))
        for token in self.file_tokens(unit.file):
            if line_start <= token[2] < line_end and token[1] == first_text:
                return token[2]
        return line_start

    def unit_token_range(self, unit):
        """单元在本库切词里的下标范围 [起, 止)。"""
        tokens = self.file_tokens(unit.file)
        start = self.unit_start_offset(unit)
        end = self.offset(unit.file, unit.last_line + 1, 1) if unit.last_line < len(self.line_starts[unit.file]) else len(self.text(unit.file))
        first = next((index for index, token in enumerate(tokens) if token[2] >= start), len(tokens))
        last = next((index for index, token in enumerate(tokens) if token[2] >= end), len(tokens))
        return first, last

    def relative(self, unit, start, end):
        """变异在容器里的相对位置：「起词.偏移-止词.偏移」，词按容器里去掉注释之后的词法单元数，偏移是那个词里的字符偏移。"""
        tokens = self.file_tokens(unit.file)
        first, last = self.unit_token_range(unit)
        start_offset, end_offset = self.offset(unit.file, *start), self.offset(unit.file, *end)
        inside = list(range(first, last))
        if not inside:
            return f"{start_offset}-{end_offset}"
        begin = next((number for number, index in enumerate(inside) if tokens[index][3] > start_offset), len(inside) - 1)
        finish = max((number for number, index in enumerate(inside) if tokens[index][2] < end_offset), default=begin)
        return f"{begin}.{max(0, start_offset - tokens[inside[begin]][2])}-{finish}.{end_offset - tokens[inside[finish]][2]}"

    def absolute(self, unit, relative):
        """relative 的反算：→ ((起行, 起列), (止行, 止列))；算不回去交 None。"""
        found = re.fullmatch(r"(\d+)\.(\d+)-(\d+)\.(\d+)", relative)
        if not found:
            return None
        tokens = self.file_tokens(unit.file)
        first, last = self.unit_token_range(unit)
        begin, begin_offset, finish, finish_offset = (int(group) for group in found.groups())
        if first + begin >= last or first + finish >= last:
            return None
        start_offset = tokens[first + begin][2] + begin_offset
        end_offset = tokens[first + finish][2] + finish_offset
        return self.line_column(unit.file, start_offset), self.line_column(unit.file, end_offset)

    def original(self, path, start, end):
        """变异区间里的原文：区间里的词法单元用 0x1f 连起来（注释与空白不算）；区间落在一个词法单元里面时取原样那一段。"""
        start_offset, end_offset = self.offset(path, *start), self.offset(path, *end)
        inside = [token[1] for token in self.file_tokens(path) if token[0] != "comment" and token[2] >= start_offset and token[3] <= end_offset]
        return "\x1f".join(inside) if inside else self.text(path)[start_offset:end_offset]

    def owners(self, unit):
        targets = [target for target in self.workspace.targets.values() if unit.file in target.files]
        targets.sort(key=lambda target: (self.library.TARGET_PREFERENCE.index(target.kind), target.id))
        return targets

    def closure(self, unit, target_id=None):
        """单元闭包（缓存）：→ (闭包或 None, 拒算形态)。入口在默认 cfg 下不编时按 test 再试一次（#[cfg(test)] 里的单元）。"""
        cache_key = (id(unit), target_id)
        if cache_key in self.closures:
            return self.closures[cache_key]
        found = (None, ["entry-not-in-any-target"])
        if unit.kind == "doctest":
            try:
                entry, doctest_target = self.workspace.find_entry(unit.file, unit.name, None)
                found = (self.library.rust_closure_of(self.workspace, entry, doctest_target), [])
            except (self.library.UsageError, self.library.FunctionUnitsError) as error:
                found = (None, [f"entry-unusable:{error}"])
        else:
            targets = [target_id] if target_id else [target.id for target in self.owners(unit)]
            for identifier in targets[:1]:
                for cfg in (None, ["test"]):
                    try:
                        found = (self.library.rust_closure_of(self.workspace, unit, identifier, cfg), [])
                        break
                    except self.library.UsageError:
                        found = (None, ["entry-cfg-inactive"])
        self.closures[cache_key] = found
        return found

    def crate_of(self, unit):
        return self.workspace.crate_of_path(unit.file)

    # 依赖行：闭包真引到的外部包，照 Cargo.lock 解析（连同传递依赖）

    def lock_packages(self):
        if self.lock is None:
            self.lock = mutation_units_library().read_lock_packages(self.root)
        return self.lock

    def dependency_lines(self, closure):
        """→ (依赖行, 拒算形态)。取法只在 lib_mutation_units.py 的 dependency_lines 一处（它文件头「依赖版本行」），排好序；
        落不到包的形态是 lock-dependency-unresolved。"""
        if broken("dependency-lines-dropped"):
            return [], []
        packages = self.lock_packages()
        if broken("dependency-whole-lock"):
            return sorted(mutation_units_library().lock_line(entry) for entry in packages if entry.get("source")), []
        lines, refusals = mutation_units_library().dependency_lines(self.workspace, closure.order, packages)
        return lines, sorted({refusal.split(" ", 1)[0] for refusal in refusals})

    def edition_lines(self, closure):
        editions = []
        directories = {self.workspace.crate_of_path(member.file).directory for member in closure.order
                       if member.language == "rust" and self.workspace.crate_of_path(member.file) is not None}
        for crate in self.workspace.crates:
            if crate.directory in directories:
                editions.append(f"edition {crate.manifest.get('package', {}).get('edition', '2015')}")
        return sorted(editions)

    # 键

    def closure_key(self, unit, global_lines, target_id=None):
        """单元闭包的键：→ (键, 组成清单, 拒算形态)。拒算时照样算出看得见的那一部分的键（allow_refused），拒算形态交回调用方。"""
        closure, refusals = self.closure(unit, target_id)
        if closure is None:
            return None, [], refusals
        dependencies, dependency_refusals = self.dependency_lines(closure)
        toolchain = list(global_lines) + self.edition_lines(closure)
        if broken("container-fingerprint-dropped"):
            kept = [member for member in closure.order if member is not unit]
            closure = _ClosureView(closure, kept)
        key, closure_refusals = self.library.closure_key(closure, toolchain=toolchain, dependencies=dependencies, allow_refused=True)
        composition = [member.text() for member in closure.units()] + [f"dependency\t{line}" for line in dependencies] + [f"toolchain\t{line}" for line in toolchain]
        forms = sorted({refusal.form for refusal in closure_refusals}) + dependency_refusals
        return key, composition, forms

    def all_units_of_files(self, paths):
        return sorted(f"{unit.kind}:{unit.fingerprint}" for path in sorted(paths) for unit in self.code_units(path))


class _ClosureView:
    """弄坏开关 container-fingerprint-dropped 用：成员少一个容器单元的闭包，别的照原样。"""

    def __init__(self, closure, kept):
        self.order, self.refusals, self.cfg_lines, self.programs = kept, closure.refusals, closure.cfg_lines, closure.programs
        self.members = {id(member): member for member in kept}

    def units(self):
        return sorted(self.order, key=lambda member: (member.file, member.first_line, member.kind, member.name))


def mutation_key(tree, item, global_lines):
    """一条变异（{file, start, end, replacement}）此刻的键：{key, refused, composition, unit, relative, locator, original}。
    key 是 None：找不到容器单元（切不开），这一条拒算、每趟现判。"""
    path = item["file"]
    start, end = tuple(item["start"]), tuple(item["end"])
    unit = tree.container(path, start, end)
    if unit is None:
        return {"key": None, "refused": ["no-container-unit"], "composition": [], "unit": None, "relative": "", "original": "",
                "locator": f"{path}::<切不开>::{start[0]}:{start[1]}::{item['replacement']}"}
    relative = tree.relative(unit, start, end)
    original = tree.original(path, start, end)
    if broken("mutated-function-only"):
        closure_key, composition, refused = digest_of_lines([f"unit\t{unit.kind}\t{unit.fingerprint}"] + list(global_lines)), [unit.text()], []
    else:
        closure_key, composition, refused = tree.closure_key(unit, global_lines)
    lines = [f"form\t{VERDICT_KEY_FORM}", f"closure\t{closure_key}", f"original\t{hashlib.sha256(original.encode('utf-8', 'surrogateescape')).hexdigest()}",
             f"replacement\t{item['replacement']}"]
    if not broken("relative-position-dropped"):
        lines.append(f"relative\t{relative}")
    if broken("defining-file-in-key"):
        closure, _forms = tree.closure(unit)
        files = {member.file for member in (closure.order if closure else [unit]) if member.language == "rust"}
        lines += [f"file-unit\t{entry}" for entry in tree.all_units_of_files(files)]
    key = digest_of_lines(lines) if closure_key is not None else None
    # 位置索引（不进键）：同一处变异在别的版本源码上也落在这个索引下（文件、容器限定名、变异名去掉位置的那一段），作废时拿它找上一回那把键比组成清单
    named = NAME_LOCATION.match(item.get("name", ""))
    description = named.group("description") if named else f"{relative} {item['replacement']}"
    return {"key": key, "refused": refused, "composition": composition + lines[1:], "unit": unit, "relative": relative, "original": original,
            "locator": f"{path}::{unit.name}::{description}"}


# ── 测试：静态列出、认名字、测试键、覆盖者键 ─────────────────────────────────────────────────────────────────

def binary_identity(package, kind, relative):
    """照 cargo nextest 的二进制 id：tests/<名>.rs 是 <包>::<名>，lib 是 <包>，src/bin/<名>.rs 是 <包>::bin/<名>；再交文件自己的模块路径。"""
    parts = relative[:-3].split("/")
    if kind == "tests":
        binary, module = f"{package}::{parts[0]}", parts[1:]
    elif parts == ["main"]:
        binary, module = f"{package}::bin/{package}", []
    elif parts[0] == "bin" and len(parts) > 1:
        binary, module = f"{package}::bin/{parts[1]}", parts[2:]
    else:
        binary, module = package, ([] if parts == ["lib"] else parts)
    if module and module[-1] in ("main", "mod"):
        module = module[:-1]
    return binary, module


def target_binary(target):
    package = target.crate.package
    if target.kind == "lib":
        return package
    if target.kind == "bin":
        return f"{package}::bin/{target.name}"
    return f"{package}::{target.name}"


def unit_attributes(tree, unit):
    """单元起头那几个外属性里的标识符（#[test]、#[ignore = "…"] 这一类）。"""
    rust_file = tree.files[unit.file]
    names, index = set(), unit.first_token
    while index is not None and index < unit.last_token and rust_file.texts[index] == "#":
        close = rust_file.pairs.get(index + 1) if index + 1 < rust_file.count and rust_file.texts[index + 1] == "[" else None
        if close is None:
            break
        if index + 2 < close:
            names.add(rust_file.texts[index + 2])
        index = close + 1
    return names


def test_entries(tree, packages, exclusions=()):
    """静态列出这几个包里的测试：[{binary, test, unit, target, ignored}]（排除表里的不算）。名字照 nextest：<二进制 id> <模块路径::fn 名>。"""
    excluded = {(binary, test) for binary, test, *_rest in exclusions}
    found = []
    for target in tree.workspace.targets.values():
        if target.crate.package not in packages or target.kind not in ("lib", "test", "bin"):
            continue
        binary = target_binary(target)
        for path, (module_path, _chain) in sorted(target.files.items()):
            for unit in tree.code_units(path):
                if unit.kind != "fn":
                    continue
                attributes = unit_attributes(tree, unit)
                if "test" not in attributes:
                    continue
                name = "::".join(list(module_path) + [unit.name])
                if (binary, name) in excluded:
                    continue
                found.append({"binary": binary, "test": name, "unit": unit, "target": target.id, "ignored": "ignore" in attributes})
    return found


def doctest_entries(tree, packages):
    found = []
    for path, rust_file in tree.files.items():
        crate = tree.workspace.crate_of_path(path)
        if crate is None or crate.package not in packages:
            continue
        for unit in rust_file.units:
            if unit.kind == "doctest":
                found.append({"binary": f"{crate.package}::doctest", "test": f"{path} - {unit.name}", "unit": unit, "target": None, "ignored": False})
    return found


DOCTEST_NAME = re.compile(r"^(?P<path>\S+) - (?P<item>.*) \(line (?P<line>\d+)\)$")


def resolve_test(tree, caught_by):
    """抓到它的测试（「<二进制 id> <用例名>」）→ (单元, 目标号, "") 或 (None, None, 为什么)。"""
    binary, _space, test = caught_by.partition(" ")
    if not test:
        return None, None, "catching-test-without-binary"
    package, _sep, rest = binary.partition("::")
    try:
        crate = tree.workspace.crate_by_package(package)
    except tree.library.UsageError:
        return None, None, "catching-test-package-unknown"
    if rest == "doctest":
        found = DOCTEST_NAME.match(test)
        if not found:
            return None, None, "catching-doctest-name-unrecognised"
        line = int(found.group("line"))
        for path in (found.group("path"), f"{crate.directory}/{found.group('path')}"):
            rust_file = tree.files.get(path)
            if rust_file is None:
                continue
            hits = [unit for unit in rust_file.units if unit.kind == "doctest" and unit.first_line <= line <= unit.last_line + 1]
            if len(hits) == 1:
                return hits[0], None, ""
        return None, None, "catching-doctest-not-found"
    if not rest:
        target_id = f"{crate.label}:lib:{crate.lib_name}"
    elif rest.startswith("bin/"):
        target_id = f"{crate.label}:bin:{rest[4:]}"
    else:
        target_id = next((identifier for identifier in (f"{crate.label}:{kind}:{rest}" for kind in ("test", "bench", "example"))
                          if identifier in tree.workspace.targets), f"{crate.label}:test:{rest}")
    target = tree.workspace.targets.get(target_id)
    if target is None:
        return None, None, "catching-test-target-unknown"
    hits = []
    for path, (module_path, _chain) in target.files.items():
        for unit in tree.code_units(path):
            if unit.kind == "fn" and "::".join(list(module_path) + [unit.name]) == test:
                hits.append(unit)
    if len(hits) != 1:
        return None, None, f"catching-test-found-{len(hits)}"
    return hits[0], target_id, ""


def test_key(tree, unit, target_id, global_lines):
    """一条测试的测试键（它的单元闭包键）：→ (键或 None, 拒算形态)。弄坏开关 catching-test-own-function-only：只取测试函数自己的指纹。"""
    cache_key = (id(unit), tuple(global_lines), broken("catching-test-own-function-only"))
    if cache_key in tree.test_keys:
        return tree.test_keys[cache_key]
    if broken("catching-test-own-function-only"):
        found = (digest_of_lines([f"test-unit\t{unit.kind}\t{unit.fingerprint}"] + list(global_lines)), [])
    else:
        key, _composition, refused = tree.closure_key(unit, global_lines, target_id)
        found = (None if refused or key is None else key, refused)
    tree.test_keys[cache_key] = found
    return found


def catching_test_key(tree, caught_by, global_lines, exclusions=()):
    """抓到它的测试此刻的测试键：→ (键或 None, 为什么)。在排除表里（不再跑）的交 None。"""
    binary, _space, test = caught_by.partition(" ")
    if (binary, test) in {(entry[0], entry[1]) for entry in exclusions} and not broken("catching-test-exclusion-ignored"):
        return None, "catching-test-excluded"
    unit, target_id, why = resolve_test(tree, caught_by)
    if unit is None:
        return None, why
    key, refused = test_key(tree, unit, target_id, global_lines)
    if key is None:
        return None, "catching-test-refused:" + ",".join(refused)
    return key, ""


def coverers_key(tree, unit, tests, global_lines):
    """覆盖者键：静态闭包里有这个容器单元的每一条测试，各自测试键的多重集取 sha256。→ (键或 None, 条数, 为什么)。"""
    keys = []
    for entry in tests:
        closure, _forms = tree.closure(entry["unit"], entry["target"])
        if closure is not None and id(unit) not in closure.members:
            continue
        key, refused = test_key(tree, entry["unit"], entry["target"], global_lines)
        if key is None:
            return None, len(keys), f"coverer-refused:{entry['binary']} {entry['test']}:" + ",".join(refused or ["closure"])
        keys.append(key)
    return digest_of_lines(["coverers-v1"] + sorted(keys)), len(keys), ""


# ── 结论能不能沿用 ─────────────────────────────────────────────────────────────────────────────────────────

def carry_decision(record, current, test_key_now="", test_reason="", coverers_now="", coverers_reason=""):
    """record 是判定库里键相同的那一行（dict：outcome、caught_by、test_key、coverage_key），current 是 mutation_key 交的。→ (能不能沿用, 理由)。"""
    if record is None:
        return False, "no-record"
    if current.get("key") is None or record.get("key") != current["key"]:
        return False, "key-changed"
    if current.get("refused"):
        return False, "refused:" + ",".join(current["refused"])
    outcome = record.get("outcome")
    if outcome == "caught":
        if record.get("caught_by") in ESCALATED_LABELS:
            if broken("escalated-not-carried"):
                return False, "escalated-rejudged"
            if broken("escalated-coverers-ignored"):
                return True, "carried"
            if not coverers_now:
                return False, coverers_reason or "coverers-unknown"
            if record.get("coverage_key") != coverers_now:
                return False, "coverers-changed"
            return True, "carried"
        if not test_key_now:
            return False, test_reason or "catching-test-unknown"
        if record.get("test_key") != test_key_now:
            return False, "catching-test-changed"
        return True, "carried"
    if outcome == "missed":
        if broken("missed-always-rejudged"):
            return False, "missed-rejudged"
        if broken("missed-coverers-ignored"):
            return True, "carried"
        if not coverers_now:
            return False, coverers_reason or "coverers-unknown"
        if record.get("coverage_key") != coverers_now:
            return False, "coverers-changed"
        return True, "carried"
    if outcome == "unviable":
        return (False, "unviable-rejudged") if broken("unviable-rejudged") else (True, "carried")
    return False, f"outcome-{outcome}"


# ── 全局那几行（每条判定的键里都有）：工具链、工具版本、构建环境的生效值、判法单元指纹 ─────────────────────────────

def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def program_digest(program, directory):
    """指一个程序的值（RUSTC_WRAPPER 这一类）：那个程序的内容 sha256；找不到写「不在」。"""
    if not program:
        return "-"
    candidate = program if os.path.isabs(program) else os.path.join(directory, program)
    if not os.path.isfile(candidate):
        for place in os.environ.get("PATH", "").split(os.pathsep):
            if os.path.isfile(os.path.join(place, program)):
                candidate = os.path.join(place, program)
                break
    if not os.path.isfile(candidate):
        return "不在"
    with open(candidate, "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()


def cargo_config_files(root, environment):
    """cargo 会读的配置：从树往上每一层的 .cargo/config(.toml)，再加 CARGO_HOME 下的；近的在前。"""
    found, directory = [], os.path.abspath(root)
    while True:
        for name in ("config.toml", "config"):
            path = os.path.join(directory, ".cargo", name)
            if os.path.isfile(path):
                found.append(path)
        parent = os.path.dirname(directory)
        if parent == directory:
            break
        directory = parent
    home = environment.get("CARGO_HOME") or os.path.join(environment.get("HOME", os.path.expanduser("~")), ".cargo")
    for name in ("config.toml", "config"):
        path = os.path.join(home, name)
        if os.path.isfile(path) and path not in found:
            found.append(path)
    return found


def toolchain_files(root):
    found, directory = [], os.path.abspath(root)
    while True:
        for name in ("rust-toolchain.toml", "rust-toolchain"):
            path = os.path.join(directory, name)
            if os.path.isfile(path):
                found.append(path)
        parent = os.path.dirname(directory)
        if parent == directory:
            return found
        directory = parent


def build_environment_lines(root, environment):
    """构建环境的生效值（算工具链的一部分，底稿第二节第 4 条）：一行一样，不带路径、不取配置文件原文。
    编译变量与它们指的程序（内容 sha256）；cargo 配置里 build / profile / target / env / unstable 这几张表解析后的值与配置里指的程序；
    rust-toolchain 解析后的值；工作区 Cargo.toml 解析后的 [profile]。读不了的配置抛 ValueError（调用方当拒绝办）。"""
    lines = []
    for name in sorted(environment):
        if name in BUILD_VARIABLES or name.startswith(BUILD_VARIABLE_PREFIXES):
            lines.append(f"variable {name}={environment[name]}")
            if name in PROGRAM_VARIABLES or name.endswith(("_RUNNER", "_LINKER")):
                lines.append(f"program {name} {program_digest(environment[name].split(' ', 1)[0], root)}")
    for depth, path in enumerate(cargo_config_files(root, environment)):
        try:
            with open(path, "rb") as handle:
                parsed = tomllib.load(handle)
        except (OSError, tomllib.TOMLDecodeError) as error:
            raise ValueError(f"读不出 cargo 配置 {path}：{error}") from error
        relevant = {table: parsed[table] for table in CARGO_CONFIG_TABLES if table in parsed}
        lines.append(f"cargo-config {depth} {canonical(relevant)}")
        for table, field in CARGO_CONFIG_PROGRAMS:
            value = parsed.get(table, {}).get(field)
            if isinstance(value, str):
                lines.append(f"program {table}.{field} {program_digest(value, os.path.dirname(os.path.dirname(path)))}")
    for depth, path in enumerate(toolchain_files(root)):
        with open(path, "rb") as handle:
            content = handle.read()
        try:
            parsed = tomllib.loads(content.decode("utf-8", "replace"))
        except tomllib.TOMLDecodeError:
            parsed = {"channel": content.decode("utf-8", "replace").strip()}
        lines.append(f"rust-toolchain {depth} {canonical(parsed)}")
    manifest = os.path.join(root, "Cargo.toml")
    if os.path.isfile(manifest):
        with open(manifest, "rb") as handle:
            content = handle.read()
        try:
            parsed = tomllib.loads(content.decode("utf-8", "replace"))
        except tomllib.TOMLDecodeError as error:
            raise ValueError(f"读不出 {manifest}：{error}") from error
        if not broken("profile-ignored"):
            lines.append(f"profile {canonical(parsed.get('profile', {}))}")
        if broken("manifest-text-in-global-key"):
            lines.append(f"manifest {hashlib.sha256(content).hexdigest()}")
    return lines


def global_lines(toolchain_text, tool_lines, build_lines, judging_key, exclusions_digest=""):
    """每条判定的键里都有的那几行（全局键）：只留工具链（rustc -vV 各行）、nextest / cargo-mutants / llvm-profdata 版本、构建环境的生效值、判法单元指纹。
    排除表不进（加一条排除只让被排除的那几条测试不再算数，弄坏开关 exclusions-in-global-key 下才进）。"""
    lines = [f"rustc {line}" for line in toolchain_text.strip().splitlines()]
    lines += [f"tool {line}" for line in tool_lines]
    lines += [f"build {line}" for line in build_lines]
    lines.append(f"judging-units {judging_key}")
    if broken("exclusions-in-global-key"):
        lines.append(f"exclusions {exclusions_digest}")
    return lines


# ── 清单按单元存 ───────────────────────────────────────────────────────────────────────────────────────────

def mutants_configuration_digest(root):
    """.cargo/mutants.toml 解析后的值（不取原文：注释不算）；没有这份写 none。"""
    path = os.path.join(root, ".cargo", "mutants.toml")
    if not os.path.isfile(path):
        return "none"
    with open(path, "rb") as handle:
        return hashlib.sha256(canonical(tomllib.load(handle)).encode()).hexdigest()


def list_unit_key(tree, unit, mutants_version, configuration_digest):
    """清单的单元键：cargo-mutants 对这个单元生成哪几条，取决于它的版本、配置、文件路径（examine_globs 按路径挑）、单元的限定名（变异名里有它）、
    单元本身与它所在的 cfg 条件与外层单元。这是清单的索引，不是判定的键：判定的键里没有路径与名字。"""
    owners = tree.owners(unit)
    chain = [raw for _tree, raw, _line in (owners[0].files.get(unit.file, ((), []))[1] if owners else [])]
    conditions = [] if broken("list-cache-context-dropped") else [raw for _tree, raw, _line in (unit.conditions or [])] + chain
    containers, probe = [], unit.container
    while probe is not None:
        containers.append(f"{probe.kind}:{probe.fingerprint}")
        probe = probe.container
    return digest_of_lines([f"form\t{LIST_KEY_FORM}", f"mutants\t{mutants_version}", f"configuration\t{configuration_digest}", f"file\t{unit.file}",
                            f"name\t{unit.name}", f"unit\t{unit.kind}\t{unit.fingerprint}"] + [f"cfg\t{condition}" for condition in conditions] + [f"container\t{entry}" for entry in containers])


NAME_LOCATION = re.compile(r"^(?P<file>[^:]+):(?P<line>\d+):(?P<column>\d+): (?P<description>.*)$", re.S)


def list_entry(tree, unit, item):
    """一条变异记进清单的样子（相对它的容器单元）：{relative, original, replacement, description}。"""
    found = NAME_LOCATION.match(item["name"])
    return {"relative": tree.relative(unit, tuple(item["start"]), tuple(item["end"])), "original": tree.original(item["file"], tuple(item["start"]), tuple(item["end"])),
            "replacement": item["replacement"], "description": found.group("description") if found else item["name"]}


def restore_item(tree, unit, entry):
    """清单里一条的复原：→ {file, start, end, replacement, name}；复原不回去（单元里没有那几个词、原文对不上）交 None。"""
    span = tree.absolute(unit, entry["relative"])
    if span is None:
        return None
    start, end = span
    if tree.original(unit.file, start, end) != entry["original"]:
        return None
    return {"file": unit.file, "start": list(start), "end": list(end), "replacement": entry["replacement"],
            "name": f"{unit.file}:{start[0]}:{start[1]}: {entry['description']}"}


# ── 跑全部的例外、起子进程的二进制、覆盖 ──────────────────────────────────────────────────────────────────────

def is_const_fn(tree, unit):
    rust_file = tree.files[unit.file]
    index = unit.first_token
    while index is not None and index < unit.last_token and rust_file.texts[index] == "#":
        close = rust_file.pairs.get(index + 1)
        index = (close + 1) if close is not None else index + 1
    words = rust_file.texts[index:min(unit.last_token, index + 8)] if index is not None else []
    return "const" in words and "fn" in words and words.index("const") < words.index("fn")


def const_context_functions(tree):
    """常量上下文（const / static 项的初始化式、const 块、数组长度、枚举判别值）里出现的名字，沿 const fn 的函数体闭包，交 const fn 单元的 id 集合。"""
    if tree.const_context is not None:
        return tree.const_context
    names, const_functions = set(), {}
    for path, rust_file in tree.files.items():
        for unit in tree.code_units(path):
            if unit.kind in ("const", "static", "enum"):
                names |= set(unit.plain or ())
            elif unit.kind == "fn" and is_const_fn(tree, unit):
                const_functions.setdefault(unit.short, []).append(unit)
        tokens = tree.file_tokens(path)
        pairs = match_pairs(tokens) or {}
        for position, token in enumerate(tokens):
            if token[1] == "const" and position + 1 < len(tokens) and tokens[position + 1][1] == "{" and position + 1 in pairs:
                names.update(found[1] for found in tokens[position + 2:pairs[position + 1]] if found[0] == "ident")
            elif token[1] == "[" and position in pairs:
                close, cursor = pairs[position], position + 1
                while cursor < close and tokens[cursor][1] != ";":
                    cursor = pairs[cursor] + 1 if cursor in pairs else cursor + 1
                if cursor < close:
                    names.update(found[1] for found in tokens[cursor + 1:close] if found[0] == "ident")
    found, frontier = set(), names & set(const_functions)
    while frontier:
        found |= frontier
        reached = set()
        for name in frontier:
            for unit in const_functions[name]:
                reached |= set(unit.plain or ())
        frontier = (reached & set(const_functions)) - found
    tree.const_context = {id(unit) for name in found for unit in const_functions[name]}
    return tree.const_context


def run_everything_reason(tree, current):
    """按覆盖挑测试的例外：这条变异要跑全部测试，交理由；按覆盖挑就交 None。"""
    unit = current.get("unit")
    if (unit is None or unit.kind != "fn") and not broken("outside-mutation-selected"):
        return "not-in-a-function"
    if unit is not None and id(unit) in const_context_functions(tree) and not broken("const-context-runs-selected"):
        return "const-fn-in-const-context"
    return None


def spawning_binaries(tree, packages, profile_counts=None):
    """起子进程的测试二进制（读源码：current_exe、CARGO_BIN_EXE_、Command::new），加覆盖那一遍里有测试写出不止一份 profraw 的二进制。"""
    if broken("spawning-binary-selected"):
        return set()
    found = set()
    for package in packages:
        for kind in ("src", "tests"):
            base = f"crates/{package}/{kind}/"
            for path in tree.files:
                if path.startswith(base) and SPAWN_PATTERNS.search(tree.text(path)):
                    found.add(binary_identity(package, kind, path[len(base):])[0])
    for (binary, _test), count in (profile_counts or {}).items():
        if count > 1:
            found.add(binary)
    return found


def normalise_coverage_path(name, tree_root, known):
    """llvm-cov 里的文件名 → 树内相对路径；不在 known（树里的文件集合）里交 None。"""
    if broken("coverage-paths-unnormalized"):
        return name if name in known else None
    full = os.path.normpath(name if os.path.isabs(name) else os.path.join(tree_root, name))
    root = os.path.normpath(tree_root)
    if full.startswith(root + os.sep):
        relative = os.path.relpath(full, root)
        if relative in known:
            return relative
    marker = full.rfind(os.sep + "crates" + os.sep)
    if marker >= 0:
        relative = full[marker + 1:]
        if relative in known:
            return relative
    return None


def covered_units(export, tree):
    """llvm-cov export 的 JSON（dict）→ 执行到的单元 [(文件, 限定名)]：计数大于 0 的函数，按它第一段代码区域落在哪个最小单元里。"""
    found = set()
    for data in export.get("data", []):
        for function in data.get("functions", []):
            if function.get("count", 0) <= 0:
                continue
            regions = [region for region in function.get("regions", []) if len(region) > 5 and region[5] == 0]
            filenames = function.get("filenames") or []
            if not regions or not filenames:
                continue
            path = normalise_coverage_path(filenames[0], tree.root, tree.files)
            if path is None:
                continue
            line, column = min((region[0], region[1]) for region in regions)
            if line > len(tree.text(path).split("\n")):
                continue
            unit = tree.container(path, tree.line_column(path, tree.offset_of_byte_column(path, line, column)), (line, column))
            if unit is not None:
                found.add((path, unit.name))
    return sorted(found)


def fingerprints_of(tree, identities):
    """[(文件, 限定名)] → 那几个单元此刻的（种类:指纹）集合；树里没有的略过。"""
    wanted = {tuple(identity) for identity in identities}
    return sorted({f"{unit.kind}:{unit.fingerprint}" for path in {identity[0] for identity in wanted} for unit in tree.code_units(path) if (path, unit.name) in wanted})


def select_tests(records, fingerprint, always=()):
    """覆盖记录 {(二进制, 用例): {种类:指纹}} → 执行到这个单元的用例，加 always 里的二进制的全部用例；按名字排。"""
    chosen = {test for test, covered in records.items() if fingerprint in covered}
    chosen |= {test for test in records if test[0] in always}
    return sorted(chosen)


def nextest_filters(tests):
    """[(二进制 id, 用例名)] → 若干条 -E 的值（nextest 对多条 -E 取并集），每条不超过 NEXTEST_FILTER_CHUNK_BYTES 字节。"""
    terms = [f"(binary_id(={binary}) & test(={test}))" for binary, test in tests]
    if broken("nextest-filter-single-argument"):
        return [" | ".join(terms)] if terms else []
    chunks, current, size = [], [], 0
    for term in terms:
        if current and size + len(term) + 3 > NEXTEST_FILTER_CHUNK_BYTES:
            chunks.append(" | ".join(current))
            current, size = [], 0
        current.append(term)
        size += len(term.encode()) + 3
    if current:
        chunks.append(" | ".join(current))
    return chunks


# ── 限时（C4） ─────────────────────────────────────────────────────────────────────────────────────────────

def check_cpu_multiplier(multiplier):
    """倍数要盖住 SMT 兄弟被占的 2.0 倍；交 None 或一句拒绝。"""
    if multiplier < MINIMUM_CPU_MULTIPLIER and not broken("limit-below-smt"):
        return f"CPU 限时倍数 {multiplier} 小于 {MINIMUM_CPU_MULTIPLIER}：盖不住 SMT 兄弟核被占时 CPU 时间涨的 2.0 倍（第三轮攻方 c4-cpu-time.txt 第 9 行）"
    return None


def limits_table(times, cpu_multiplier, cpu_floor, wall_multiplier, wall_floor):
    """基线逐条的 (CPU 秒, 墙钟秒) → {(二进制文件名去掉哈希, 用例): (CPU 上限, 墙钟上限)}，另加 ("*", "*") 取全表最大，认不出的用例用它。"""
    table = {}
    for key, (cpu, wall) in times.items():
        table[key] = (cpu_multiplier * cpu + cpu_floor, wall_multiplier * wall + wall_floor)
    if table:
        table[("*", "*")] = (max(value[0] for value in table.values()), max(value[1] for value in table.values()))
    else:
        table[("*", "*")] = (cpu_floor, wall_floor)
    return table


RUNNER_SCRIPT = r'''#!/usr/bin/env python3
# 广谱变异驱动写的测试 runner（CARGO_TARGET_<三元组>_RUNNER，nextest 与 cargo test 都经它起测试二进制）。
# BROAD_MUTATION_RUNNER_MODE：空或 --list 照原样 exec；measure 记这一条（或这个二进制）的 CPU 与墙钟秒；limit 按表设 CPU 上限（RLIMIT_CPU）、墙钟兜底。
import os, re, resource, signal, sys, time
arguments = sys.argv[1:]
mode = os.environ.get("BROAD_MUTATION_RUNNER_MODE", "")
if not arguments:
    sys.exit(2)
if mode not in ("measure", "limit") or "--list" in arguments[1:]:
    os.execv(arguments[0], arguments)
breaks = os.environ.get("BROAD_MUTATION_BREAK", "").split(",")
stem = re.sub(r"-[0-9a-f]{16}$", "", os.path.basename(arguments[0]))
test = arguments[arguments.index("--exact") + 1] if "--exact" in arguments[1:-1] else "*"
cpu_limit = wall_limit = None
if mode == "limit":
    limits = {}
    with open(os.environ["BROAD_MUTATION_RUNNER_LIMITS"], encoding="utf-8") as handle:
        for line in handle:
            fields = line.rstrip("\n").split("\t")
            if len(fields) == 4:
                limits[(fields[0], fields[1])] = (float(fields[2]), float(fields[3]))
    cpu_limit, wall_limit = limits.get((stem, test)) or limits.get((stem, "*")) or limits[("*", "*")]
    if "cpu-limit-ignored" in breaks:
        cpu_limit = None
resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
started = time.monotonic()
child = os.fork()
if child == 0:
    try:
        os.setpgid(0, 0)
        if cpu_limit is not None:
            soft = max(1, int(cpu_limit + 0.999))
            resource.setrlimit(resource.RLIMIT_CPU, (soft, soft + 5))
        os.execv(arguments[0], arguments)
    finally:
        os._exit(127)
wall_hit = []
def forward(signal_number, _frame):
    try:
        os.killpg(child, signal_number)
    except OSError:
        pass
def wall_expired(_signal_number, _frame):
    wall_hit.append(True)
    try:
        os.killpg(child, signal.SIGKILL)
    except OSError:
        pass
for name in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
    signal.signal(name, forward)
if wall_limit is not None:
    signal.signal(signal.SIGALRM, wall_expired)
    signal.setitimer(signal.ITIMER_REAL, wall_limit)
_pid, status, usage = os.wait4(child, 0)
signal.setitimer(signal.ITIMER_REAL, 0)
cpu, wall = usage.ru_utime + usage.ru_stime, time.monotonic() - started
if mode == "measure":
    line = f"{stem}\t{test}\t{cpu:.3f}\t{wall:.3f}\t{status}\n".encode()
    descriptor = os.open(os.environ["BROAD_MUTATION_RUNNER_TIMES"], os.O_WRONLY | os.O_APPEND | os.O_CREAT, 0o644)
    os.write(descriptor, line)
    os.close(descriptor)
if wall_hit:
    print(f"BROAD_MUTATION_LIMIT kind=wall test={stem} {test} wall_seconds={wall:.1f} limit={wall_limit:.1f} cpu_seconds={cpu:.1f}", file=sys.stderr, flush=True)
    sys.exit(124)
if os.WIFSIGNALED(status):
    number = os.WTERMSIG(status)
    if number == signal.SIGXCPU or (cpu_limit is not None and number == signal.SIGKILL and cpu >= cpu_limit):
        print(f"BROAD_MUTATION_LIMIT kind=cpu test={stem} {test} cpu_seconds={cpu:.1f} limit={cpu_limit:.1f}", file=sys.stderr, flush=True)
    signal.signal(number, signal.SIG_DFL)
    os.kill(os.getpid(), number)
    sys.exit(128 + number)
sys.exit(os.WEXITSTATUS(status))
'''


def read_times(path):
    """runner 在 measure 模式写的表 → {(文件名, 用例): (CPU 秒, 墙钟秒)}；同一个键取最大。"""
    times = {}
    if not os.path.exists(path):
        return times
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            fields = line.rstrip("\n").split("\t")
            if len(fields) >= 4:
                key = (fields[0], fields[1])
                cpu, wall = float(fields[2]), float(fields[3])
                old = times.get(key, (0.0, 0.0))
                times[key] = (max(old[0], cpu), max(old[1], wall))
    return times


def write_limits(path, table):
    with open(path + ".partial", "w", encoding="utf-8") as handle:
        for (stem, test), (cpu, wall) in sorted(table.items()):
            handle.write(f"{stem}\t{test}\t{cpu:.1f}\t{wall:.1f}\n")
    os.replace(path + ".partial", path)


# ── 自证：样本树上逐个判沿用的决定 ───────────────────────────────────────────────────────────────────────────
# 每个样本：(名字, 期望, 守它的弄坏开关, 第一版的文件, 第二版换掉的文件, 变异 (文件, 原文, 替换文, 第几处), 判定库里那一行的结局, 别的选项)。
# 期望 rejudge：沿用不了，必须现判（开关打开时那一格判成 carry、红）；期望 carry：键与沿用判据都对得上，照沿用（开关打开时判成 rejudge、红）。
# 选项：record 第一版判的是哪一条（默认同一条）、label 抓到的是升档判的（标签）、exclusions 第二版的排除表多出的行。
# 样本树：工作区两个包，mcore（被变异，lib.rs 由各模块文件现生成 pub mod）与 mharness（tests/t.rs 里的 catches 是抓到它的那条测试）。
SAMPLE_TEST = "mharness::t catches"
SAMPLE_WORKSPACE = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/mcore", "crates/mharness"]\nresolver = "2"\n',
    "crates/mcore/Cargo.toml": '[package]\nname = "mcore"\nversion = "0.1.0"\nedition = "2021"\n',
    "crates/mharness/Cargo.toml": '[package]\nname = "mharness"\nversion = "0.1.0"\nedition = "2021"\n\n[dev-dependencies]\nmcore = { path = "../mcore" }\n',
    "crates/mharness/src/lib.rs": "",
}
T = "crates/mharness/tests/t.rs"


def test_calling(expression, expected):
    return f"#[test]\nfn catches() {{\n    assert_eq!({expression}, {expected});\n}}\n"


DEPENDENCY_MANIFEST = '[package]\nname = "mcore"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\nextdep = "1.0"\nother = "2.0"\n'


def dependency_lock(extdep_version, other_version):
    return ('version = 3\n\n[[package]]\nname = "extdep"\nversion = "%s"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "%s"\n\n'
            '[[package]]\nname = "mcore"\nversion = "0.1.0"\ndependencies = [\n "extdep",\n "other",\n]\n\n[[package]]\nname = "mharness"\nversion = "0.1.0"\n'
            'dependencies = [\n "mcore",\n]\n\n[[package]]\nname = "other"\nversion = "%s"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n'
            'checksum = "%s"\n') % (extdep_version, hashlib.sha256(extdep_version.encode()).hexdigest(), other_version, hashlib.sha256(other_version.encode()).hexdigest())


MACROS = "#[macro_export]\nmacro_rules! bump {\n    ($e:expr) => {\n        $e + 1\n    };\n}\npub fn unrelated() -> u64 {\n    5\n}\n"
INDEX = ("pub fn index_node_header_bytes(key_width: u64) -> u64 {\n    8 + key_width\n}\n"
         "pub fn index_node_entry_capacity(key_width: u64, entry_width: u64) -> u64 {\n    (4096 - index_node_header_bytes(key_width)) / entry_width\n}\n")
Y9 = {"crates/mcore/src/y9_type.rs": "pub struct W(pub u64);\n", "crates/mcore/src/y9.rs": "use crate::y9_type::W;\npub fn make() -> W {\n    W(1)\n}\n",
      T: test_calling("mcore::y9::make().0", 1)}
ROOT_ONE = 'pub fn check_root_slot(slot: u64) -> u64 {\n    let checked = slot.checked_add(1).expect("457");\n    checked * 2\n}\n'
NESTED = "pub fn outer(value: u64) -> u64 {\n    fn inner(input: u64) -> u64 {\n        input + 1\n    }\n    inner(value) * 2\n}\n"
OFFSETS = "pub const OFFSET: u64 = 95;\npub fn offset_of(n: u64) -> u64 {\n    n + OFFSET\n}\n"
USER = "pub fn user(n: u64) -> u64 {\n    bump!(n) * 3\n}\n"
CHECK = "pub fn check(n: u64) -> u64 {\n    n + 1\n}\n"
RARELY = "pub fn rarely(n: u64) -> u64 {\n    n + 7\n}\n"
SAMPLES = [
    # controls 路第 5 节点名的五格（L22）。S34 抓到的那一条有两把键守着（变异键里的容器单元、测试键里的被测函数），只拿掉一把照样现判：
    # 守它的写成「两个开关同开」（+ 连起来），单个开关红在 S34U（无效的结论没有测试键，只靠变异键）
    ("S34", "rejudge", "container-fingerprint-dropped+catching-test-own-function-only",
     {"crates/mcore/src/root.rs": ROOT_ONE, T: test_calling("mcore::root::check_root_slot(1)", 4)},
     {"crates/mcore/src/root.rs": ROOT_ONE.replace('"457"', '"458"')}, ("crates/mcore/src/root.rs", "*", "/", 0), "caught", {}),
    ("S34U", "rejudge", "container-fingerprint-dropped",
     {"crates/mcore/src/root.rs": ROOT_ONE, T: test_calling("mcore::root::check_root_slot(1)", 4)},
     {"crates/mcore/src/root.rs": ROOT_ONE.replace('"457"', '"458"')}, ("crates/mcore/src/root.rs", "*", "/", 0), "unviable", {}),
    ("T16", "rejudge", "catching-test-own-function-only",
     {"crates/mcore/src/pool.rs": "pub fn publish(n: u64) -> u64 {\n    n + 1\n}\n",
      T: "fn build_pool(tag: u64) -> u64 {\n    tag * 3\n}\n\n#[test]\nfn catches() {\n    assert_eq!(mcore::pool::publish(build_pool(1)), 4);\n}\n"},
     {T: "fn build_pool(tag: u64) -> u64 {\n    tag * 4\n}\n\n#[test]\nfn catches() {\n    assert_eq!(mcore::pool::publish(build_pool(1)), 4);\n}\n"},
     ("crates/mcore/src/pool.rs", "+", "-", 0), "caught", {}),
    ("N37", "carry", "defining-file-in-key",
     {"crates/mcore/src/check.rs": CHECK, T: test_calling("mcore::check::check(1)", 2)},
     {"crates/mcore/src/check.rs": "use std::collections::BTreeMap;\n" + CHECK}, ("crates/mcore/src/check.rs", "+", "-", 0), "caught", {}),
    ("N45", "carry", "defining-file-in-key",
     {"crates/mcore/src/check.rs": CHECK, T: test_calling("mcore::check::check(1)", 2)},
     {"crates/mcore/src/check.rs": CHECK + "pub fn never_called() -> u64 {\n    7\n}\n"}, ("crates/mcore/src/check.rs", "+", "-", 0), "caught", {}),
    ("T17", "carry", "defining-file-in-key",
     {"crates/mcore/src/macros.rs": MACROS, "crates/mcore/src/user.rs": USER, T: test_calling("mcore::user::user(1)", 6)},
     {"crates/mcore/src/macros.rs": MACROS.replace("    5\n", "    6\n")}, ("crates/mcore/src/user.rs", "*", "/", 0), "caught", {}),
    # verify 路量过的漏判（index_node_entry_capacity 调的 index_node_header_bytes 改了，旧判法仍沿用）：抓到的那一条两把键都守着，单个开关红在 INDEXU
    ("INDEX", "rejudge", "mutated-function-only+catching-test-own-function-only",
     {"crates/mcore/src/index.rs": INDEX, T: test_calling("mcore::index::index_node_entry_capacity(8, 16)", 255)},
     {"crates/mcore/src/index.rs": INDEX.replace("8 + key_width", "16 + key_width")}, ("crates/mcore/src/index.rs", "/", "*", 0), "caught", {}),
    ("INDEXU", "rejudge", "mutated-function-only",
     {"crates/mcore/src/index.rs": INDEX, T: test_calling("mcore::index::index_node_entry_capacity(8, 16)", 255)},
     {"crates/mcore/src/index.rs": INDEX.replace("8 + key_width", "16 + key_width")}, ("crates/mcore/src/index.rs", "/", "*", 0), "unviable", {}),
    # 被变异函数用到的宏、常量、类型别名、枚举判别值、include_str! 读的文件、类型改了：都在它的闭包里（无效的结论只靠变异键）
    ("MACRO", "rejudge", "mutated-function-only",
     {"crates/mcore/src/macros.rs": MACROS, "crates/mcore/src/user.rs": USER, T: test_calling("mcore::user::user(1)", 6)},
     {"crates/mcore/src/macros.rs": MACROS.replace("$e + 1", "$e + 2")}, ("crates/mcore/src/user.rs", "*", "/", 0), "unviable", {}),
    ("S04", "rejudge", "mutated-function-only",
     {"crates/mcore/src/offsets.rs": OFFSETS, T: test_calling("mcore::offsets::offset_of(1)", 96)},
     {"crates/mcore/src/offsets.rs": OFFSETS.replace("95", "91")}, ("crates/mcore/src/offsets.rs", "+", "-", 0), "unviable", {}),
    ("Y8", "rejudge", "mutated-function-only",
     {"crates/mcore/src/y8.rs": 'pub fn y8() -> usize {\n    include_str!("y8_k.txt").len() + 1\n}\n', "crates/mcore/src/y8_k.txt": "abc\n", T: test_calling("mcore::y8::y8()", 5)},
     {"crates/mcore/src/y8_k.txt": "abcd\n"}, ("crates/mcore/src/y8.rs", "+", "-", 0), "unviable", {}),
    ("FC", "rejudge", "mutated-function-only",
     {"crates/mcore/src/word.rs": "pub type Word = u16;\n", "crates/mcore/src/pack.rs": "use crate::word::Word;\npub fn pack(high: u64, low: u64) -> Word {\n    ((high << 8) | low) as Word\n}\n",
      T: test_calling("mcore::pack::pack(1, 1)", 257)},
     {"crates/mcore/src/word.rs": "pub type Word = u8;\n"}, ("crates/mcore/src/pack.rs", "<<", ">>", 0), "unviable", {}),
    ("FD", "rejudge", "mutated-function-only",
     {"crates/mcore/src/kind.rs": "pub enum Kind {\n    Data = 1,\n    Meta = 3,\n}\n", "crates/mcore/src/tag.rs": "use crate::kind::Kind;\npub fn tag(kind: Kind, count: u64) -> u64 {\n    (kind as u64) * count\n}\n",
      T: test_calling("mcore::tag::tag(mcore::kind::Kind::Data, 2)", 2)},
     {"crates/mcore/src/kind.rs": "pub enum Kind {\n    Data = 2,\n    Meta = 3,\n}\n"}, ("crates/mcore/src/tag.rs", "*", "+", 0), "unviable", {}),
    ("Y9", "rejudge", "mutated-function-only", Y9, {"crates/mcore/src/y9_type.rs": "#[derive(Default)]\npub struct W(pub u64);\n"},
     ("crates/mcore/src/y9.rs", "W(1)", "Default::default()", 0), "unviable", {}),
    # 容器单元自己变了（签名、嵌套 fn 那一段）：变异键里的容器单元守着
    ("Y5", "rejudge", "container-fingerprint-dropped",
     {"crates/mcore/src/y5.rs": "pub fn widen(value: u32) -> u64 {\n    value as u64 + 1\n}\n", T: test_calling("mcore::y5::widen(1)", 2)},
     {"crates/mcore/src/y5.rs": "pub fn widen(value: u16) -> u64 {\n    value as u64 + 1\n}\n"}, ("crates/mcore/src/y5.rs", "+", "-", 0), "unviable", {}),
    ("FF", "rejudge", "container-fingerprint-dropped",
     {"crates/mcore/src/nested.rs": NESTED, T: test_calling("mcore::nested::outer(1)", 4)},
     {"crates/mcore/src/nested.rs": NESTED.replace("inner(value) * 2", "(value + 1) * 2")}, ("crates/mcore/src/nested.rs", "+", "-", 0), "unviable", {}),
    # 全局那几行：profile 的生效值进键、Cargo.toml 的注释不进、排除表不进
    ("Y7", "rejudge", "profile-ignored",
     {"Cargo.toml": SAMPLE_WORKSPACE["Cargo.toml"] + "\n[profile.dev]\noverflow-checks = true\n", "crates/mcore/src/y7.rs": "pub fn y7(n: u8) -> u8 {\n    n + 200\n}\n",
      T: test_calling("mcore::y7::y7(1)", 201)},
     {"Cargo.toml": SAMPLE_WORKSPACE["Cargo.toml"] + "\n[profile.dev]\noverflow-checks = false\n"}, ("crates/mcore/src/y7.rs", "+", "*", 0), "caught", {}),
    ("MANIFEST", "carry", "manifest-text-in-global-key",
     {"crates/mcore/src/y7.rs": "pub fn y7(n: u8) -> u8 {\n    n + 200\n}\n", T: test_calling("mcore::y7::y7(1)", 201)},
     {"Cargo.toml": "# 只改注释：工作区的成员与 profile 一样没变\n" + SAMPLE_WORKSPACE["Cargo.toml"]}, ("crates/mcore/src/y7.rs", "+", "*", 0), "caught", {}),
    ("EXCLUDE-OTHER", "carry", "exclusions-in-global-key",
     {"crates/mcore/src/check.rs": CHECK, T: test_calling("mcore::check::check(1)", 2)},
     {}, ("crates/mcore/src/check.rs", "+", "-", 0), "caught", {"exclusions": [("mharness::t", "some_other_case", "自证：排除表加了一条别的测试")]}),
    ("EXCLUDE-CATCHER", "rejudge", "catching-test-exclusion-ignored",
     {"crates/mcore/src/check.rs": CHECK, T: test_calling("mcore::check::check(1)", 2)},
     {}, ("crates/mcore/src/check.rs", "+", "-", 0), "caught", {"exclusions": [("mharness::t", "catches", "自证：抓到它的那条测试进了排除表")]}),
    # 无效、没红、升档判的抓到都进可沿用库
    ("UNVIABLE", "carry", "unviable-rejudged", Y9, {"crates/mcore/src/zother.rs": "pub fn other() -> u64 {\n    3\n}\n"},
     ("crates/mcore/src/y9.rs", "W(1)", "Default::default()", 0), "unviable", {}),
    ("N1", "rejudge", "missed-coverers-ignored",
     {"crates/mcore/src/n1.rs": RARELY, "crates/mcore/src/check.rs": CHECK, T: test_calling("mcore::check::check(1)", 2)},
     {"crates/mharness/tests/n1.rs": "use mcore::n1::rarely;\n#[test]\nfn now_covered() {\n    assert_eq!(rarely(1), 8);\n}\n"},
     ("crates/mcore/src/n1.rs", "+", "-", 0), "missed", {}),
    ("MISSED", "carry", "missed-always-rejudged",
     {"crates/mcore/src/n1.rs": RARELY, "crates/mcore/src/check.rs": CHECK, T: test_calling("mcore::check::check(1)", 2)},
     {T: test_calling("mcore::check::check(2)", 3)}, ("crates/mcore/src/n1.rs", "+", "-", 0), "missed", {}),
    ("ESCALATED", "carry", "escalated-not-carried",
     {"crates/mcore/src/check.rs": CHECK, T: test_calling("mcore::check::check(1)", 2)},
     {"crates/mcore/src/zother.rs": "pub fn other() -> u64 {\n    3\n}\n"}, ("crates/mcore/src/check.rs", "+", "-", 0), "caught", {"label": ESCALATED_LABELS[0]}),
    ("ESCALATED-COVERERS", "rejudge", "escalated-coverers-ignored",
     {"crates/mcore/src/check.rs": CHECK, T: test_calling("mcore::check::check(1)", 2)},
     {"crates/mharness/tests/escalated.rs": "use mcore::check::check;\n#[test]\nfn now_also_covers() {\n    assert_eq!(check(4), 5);\n}\n"},
     ("crates/mcore/src/check.rs", "+", "-", 0), "caught", {"label": ESCALATED_LABELS[0]}),
    # 同文件里别的函数改了不作废；同一个函数里两处同样的替换、两个 impl 里一样的方法各是各的；注释不改相对位置
    ("FA", "carry", "defining-file-in-key",
     {"crates/mcore/src/slots.rs": 'pub fn slot_of(offset: u64) -> u64 {\n    offset / 4096\n}\n\npub fn describe(offset: u64) -> String {\n    format!("slot-{}", slot_of(offset))\n}\n',
      T: test_calling("mcore::slots::slot_of(8192)", 2)},
     {"crates/mcore/src/slots.rs": 'pub fn slot_of(offset: u64) -> u64 {\n    offset / 4096\n}\n\npub fn describe(offset: u64) -> String {\n    format!("slot-{}", offset / 4096)\n}\n'},
     ("crates/mcore/src/slots.rs", "/", "%", 0), "caught", {}),
    ("Q1", "rejudge", "mutated-function-only",
     {"crates/mcore/src/q1.rs": "pub struct A;\npub struct B;\nimpl A {\n    pub fn value(&self) -> u64 {\n        7 + 1\n    }\n}\nimpl B {\n    pub fn value(&self) -> u64 {\n        7 + 1\n    }\n}\n",
      T: test_calling("mcore::q1::A.value() + mcore::q1::B.value()", 16)},
     {}, ("crates/mcore/src/q1.rs", "+", "-", 1), "caught", {"record": ("crates/mcore/src/q1.rs", "+", "-", 0)}),
    ("TWO", "rejudge", "relative-position-dropped",
     {"crates/mcore/src/two.rs": "pub fn twice(n: u64) -> u64 {\n    (n + 1) * (n + 1)\n}\n", T: test_calling("mcore::two::twice(1)", 4)},
     {}, ("crates/mcore/src/two.rs", "+", "-", 1), "caught", {"record": ("crates/mcore/src/two.rs", "+", "-", 0)}),
    ("COMMENT", "carry", "relative-position-counts-comments",
     {"crates/mcore/src/comment.rs": "pub fn commented(n: u64) -> u64 {\n    let m = n * 2;\n    m + 1\n}\n", T: test_calling("mcore::comment::commented(1)", 3)},
     {"crates/mcore/src/comment.rs": "pub fn commented(n: u64) -> u64 {\n    // 只加一行注释\n    let m = n * 2; /* 行尾也加一段 */\n    m + 1\n}\n"},
     ("crates/mcore/src/comment.rs", "+", "-", 0), "caught", {}),
    # 依赖：闭包真点了名的外部包的版本进键（无效的结论只靠变异键），别的包换版本不作废
    ("DEP", "rejudge", "dependency-lines-dropped",
     {"crates/mcore/Cargo.toml": DEPENDENCY_MANIFEST, "Cargo.lock": dependency_lock("1.0.0", "2.0.0"),
      "crates/mcore/src/dep.rs": "pub fn dep(n: u64) -> u64 {\n    extdep::shift(n) + 1\n}\n", T: test_calling("mcore::dep::dep(1)", 2)},
     {"Cargo.lock": dependency_lock("1.0.1", "2.0.0")}, ("crates/mcore/src/dep.rs", "+", "-", 0), "unviable", {}),
    ("DEP-OTHER", "carry", "dependency-whole-lock",
     {"crates/mcore/Cargo.toml": DEPENDENCY_MANIFEST, "Cargo.lock": dependency_lock("1.0.0", "2.0.0"),
      "crates/mcore/src/dep.rs": "pub fn dep(n: u64) -> u64 {\n    extdep::shift(n) + 1\n}\n", T: test_calling("mcore::dep::dep(1)", 2)},
     {"Cargo.lock": dependency_lock("1.0.0", "2.0.1")}, ("crates/mcore/src/dep.rs", "+", "-", 0), "caught", {}),
]
# 按覆盖挑测试的例外（第三轮 X1b、X3 与 K1）：期望「跑全部」
C2_SOURCES = {
    "crates/mharness/tests/golden.rs": '#[test]\nfn golden_parent() {\n    let output = std::process::Command::new(std::env::current_exe().unwrap()).arg("child").output().unwrap();\n    assert!(output.status.success());\n}\n',
    "crates/mcore/src/mixed.rs": "pub const fn square(x: u64) -> u64 {\n    x * x\n}\npub const fn helper(x: u64) -> u64 {\n    square(x) + 0\n}\npub fn runtime_only(x: u64) -> u64 {\n    x + 1\n}\n",
    "crates/mharness/tests/consts.rs": "const STRONG: u64 = mcore::mixed::helper(3);\n#[test]\nfn strong() {\n    assert_eq!(STRONG, 9);\n}\n",
    "crates/mcore/src/limits.rs": "pub const LIMIT: u64 = 1 << 8;\n",
}


def write_tree(root, files):
    """写一棵样本树：files 之外补齐 SAMPLE_WORKSPACE，mcore 的 lib.rs 照 src/ 下的模块文件现生成 pub mod。"""
    everything = dict(SAMPLE_WORKSPACE, **files)
    modules = sorted(os.path.basename(path)[:-3] for path in everything if path.startswith("crates/mcore/src/") and path.endswith(".rs")
                     and path.count("/") == 3 and not path.endswith("/lib.rs"))
    everything.setdefault("crates/mcore/src/lib.rs", "".join(f"pub mod {module};\n" for module in modules))
    for path, text in everything.items():
        full = os.path.join(root, path)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w", encoding="utf-8") as handle:
            handle.write(text)


def item_for(files, mutation):
    path, old, replacement, occurrence = mutation
    text, offset = files[path], -1
    for _ in range(occurrence + 1):
        offset = text.index(old, offset + 1)

    def line_column(position):
        line = text.count("\n", 0, position) + 1
        return [line, position - (text.rfind("\n", 0, position) + 1) + 1]
    return {"file": path, "start": line_column(offset), "end": line_column(offset + len(old)), "replacement": replacement,
            "name": f"{path}:{line_column(offset)[0]}:{line_column(offset)[1]}: replace {old} with {replacement}"}


def sample_globals(root, exclusions):
    exclusions_digest = digest_of_lines(["\t".join(row) for row in exclusions])
    return global_lines("rustc 0.0.0-selftest\nhost: x86_64-unknown-linux-gnu", ["cargo-nextest 0.0.0-selftest"], build_environment_lines(root, {}),
                        "selftest-judging-units", exclusions_digest)


def sample_decision(work, sample):
    """交 (决定, 理由)：决定是 carry 或 rejudge。"""
    name, _expected, _switch, first, second, mutation, outcome, options = sample
    files_one = dict(first)
    files_two = dict(files_one, **second)
    exclusions_two = list(options.get("exclusions", []))
    roots = [os.path.join(work, name, "v1"), os.path.join(work, name, "v2")]
    for root, files in zip(roots, (files_one, files_two)):
        write_tree(root, files)
    trees = [UnitTree(root) for root in roots]
    globals_one, globals_two = sample_globals(roots[0], []), sample_globals(roots[1], exclusions_two)
    packages = ("mcore", "mharness")
    recorded = mutation_key(trees[0], item_for(files_one, options.get("record", mutation)), globals_one)
    test_one, _why = catching_test_key(trees[0], SAMPLE_TEST, globals_one)
    coverers_one = coverers_key(trees[0], recorded["unit"], test_entries(trees[0], packages), globals_one)[0] if recorded["unit"] else ""
    record = {"key": recorded["key"], "outcome": outcome, "caught_by": options.get("label", SAMPLE_TEST) if outcome == "caught" else "",
              "test_key": test_one if outcome == "caught" else "", "coverage_key": coverers_one if outcome == "missed" or options.get("label") else ""}
    current = mutation_key(trees[1], item_for(files_two, mutation), globals_two)
    # 样本本身要算得出键：两版的变异键、第一版抓到它那条测试的测试键都不许是 None（不然「现判」可能只是算不出，不是判出来的）
    if recorded["key"] is None or current["key"] is None or (outcome == "caught" and not options.get("label") and not test_one):
        return "error", f"样本算不出键（第一版 {recorded['key'] and recorded['key'][:12]} {recorded['refused']}，第二版 {current['key'] and current['key'][:12]} {current['refused']}，测试键 {test_one}）"
    test_two, test_why = catching_test_key(trees[1], SAMPLE_TEST, globals_two, exclusions_two)
    coverers_two, _count, coverers_why = (coverers_key(trees[1], current["unit"], test_entries(trees[1], packages, exclusions_two), globals_two)
                                          if current["unit"] else ("", 0, "no-unit"))
    carry, reason = carry_decision(record if record["key"] == current["key"] else None, current, test_two or "", test_why, coverers_two or "", coverers_why)
    return ("carry" if carry else "rejudge"), reason


def selftest_list_cells(work, check):
    """清单按单元存：单元键随 cfg 条件链变（同一个单元、同一个名字，挂它的 mod 声明加了 #[cfg(test)]）；清单的一条记成相对单元的样子之后
    在此刻的源码上复原得回去（加了注释行也一样）。"""
    plain = {"crates/mcore/src/ctx.rs": "pub fn helper() -> u64 {\n    1 + 1\n}\n"}
    gated = dict(plain, **{"crates/mcore/src/lib.rs": "#[cfg(test)]\npub mod ctx;\n"})
    commented = {"crates/mcore/src/ctx.rs": "// 头上加一行注释\npub fn helper() -> u64 {\n    // 体里也加一行\n    1 + 1\n}\n"}
    keys, restored = [], []
    for name, files in (("plain", plain), ("gated", gated), ("commented", commented)):
        root = os.path.join(work, "list", name)
        write_tree(root, files)
        tree = UnitTree(root)
        item = item_for(files, ("crates/mcore/src/ctx.rs", "+", "-", 0))
        unit = tree.container(item["file"], tuple(item["start"]), tuple(item["end"]))
        keys.append(list_unit_key(tree, unit, "cargo-mutants 0.0.0-selftest", "none") if unit else "")
        if name == "plain":
            entry = list_entry(tree, unit, item)
        restored.append(restore_item(tree, unit, entry) if unit else None)
    check(keys[0] and keys[1] and keys[0] != keys[1] and keys[0] == keys[2],
          f"单元挪进 #[cfg(test)] 之后清单的单元键没变、或只加注释就变了（{[key[:12] for key in keys]}）",
          "看 list_unit_key：cfg 条件链进键（挪进 cfg(test) 的单元 cargo-mutants 不生成），注释不进（单元指纹本来就去注释）")
    check(restored[0] is not None and restored[0]["start"] == [2, 7] and restored[2] is not None and restored[2]["start"] == [4, 7]
          and restored[2]["name"].startswith("crates/mcore/src/ctx.rs:4:7: "),
          f"清单的一条没复原回去（原样那份 {restored[0]}；加了注释的那份 {restored[2]}）",
          "看 list_entry 与 restore_item：相对位置按去掉注释的词法单元数记，反算回此刻源码里的行列，原文要逐字对上")


def run_selftest():
    import shutil
    import tempfile
    cells, failures = [0], [0]

    def check(condition, wrong, howto):
        cells[0] += 1
        if not condition:
            failures[0] += 1
            print(f"  ✗ 自证：{wrong}", file=sys.stderr)
            print(f"     → 怎么办：{howto}", file=sys.stderr)

    work = tempfile.mkdtemp(prefix="mutation-prune-selftest.")
    try:
        for sample in SAMPLES:
            try:
                decision, reason = sample_decision(work, sample)
            except Exception as error:  # 样本树切不开、库接口变了：照判错记，不让后面的格看不见
                decision, reason = "error", f"{type(error).__name__}: {error}"
            check(decision == sample[1], f"样本 {sample[0]} 判成 {decision}（{reason}），该是 {sample[1]}（守它的开关 {sample[2]}）",
                  f"看文件头「规则」里开关 {sample[2]} 那一条与 carry_decision、mutation_key、catching_test_key、coverers_key")
        selftest_list_cells(work, check)
        root = os.path.join(work, "c2")
        write_tree(root, C2_SOURCES)
        tree = UnitTree(root)
        check("mharness::golden" in spawning_binaries(tree, ["mharness"]),
              f"X1b：起本二进制当子进程的测试二进制没认出来（认出的 {sorted(spawning_binaries(tree, ['mharness']))}）",
              "看 spawning_binaries：源码里有 current_exe / CARGO_BIN_EXE_ / Command::new 的测试二进制每次整个跑（E1）")
        square = mutation_key(tree, item_for(C2_SOURCES, ("crates/mcore/src/mixed.rs", "x * x", "x + x", 0)), [])
        runtime = mutation_key(tree, item_for(C2_SOURCES, ("crates/mcore/src/mixed.rs", "x + 1", "x - 1", 0)), [])
        check(run_everything_reason(tree, square) == "const-fn-in-const-context" and run_everything_reason(tree, runtime) is None,
              f"X3：常量上下文里（经 helper 一跳）用到的 const fn 没判成跑全部（square {run_everything_reason(tree, square)}，runtime_only {run_everything_reason(tree, runtime)}）",
              "看 const_context_functions：const / static 初始化式里的名字沿 const fn 函数体闭包（E1）")
        constant = mutation_key(tree, item_for(C2_SOURCES, ("crates/mcore/src/limits.rs", "<<", ">>", 0)), [])
        check(run_everything_reason(tree, constant) == "not-in-a-function" and constant["unit"] is not None and constant["unit"].kind == "const",
              f"K1：常量初始化式里的变异没判成跑全部（{run_everything_reason(tree, constant)}，容器 {constant['unit'] and constant['unit'].kind}）",
              "看 run_everything_reason：容器不是 fn 单元（const、static、impl 头……）的变异覆盖记不到，一律跑全部")
        # 覆盖文件名归一（别处一份副本的绝对路径、带 .. 的相对路径），列按字节（行里有中文）
        write_tree(root, dict(C2_SOURCES, **{"crates/mcore/src/wide.rs": '// 中文注释\npub fn wide() -> &\'static str { "宽字符" }\npub fn after() -> u64 { 1 }\n'}))
        tree = UnitTree(root)
        export = {"data": [{"functions": [
            {"count": 3, "filenames": ["/somewhere/else/copy/crates/mcore/src/mixed.rs"], "regions": [[7, 1, 9, 2, 3, 0, 0, 0]]},
            {"count": 1, "filenames": ["crates/mharness/tests/../../mcore/src/wide.rs"], "regions": [[3, 1, 3, 30, 1, 0, 0, 0]]},
            {"count": 0, "filenames": [os.path.join(root, "crates/mcore/src/mixed.rs")], "regions": [[1, 1, 3, 2, 0, 0, 0, 0]]},
            {"count": 5, "filenames": ["/rustc/abc/library/core/src/num.rs"], "regions": [[1, 1, 2, 2, 5, 0, 0, 0]]}]}]}
        covered = covered_units(export, tree)
        fingerprints = fingerprints_of(tree, covered)
        check(covered == [("crates/mcore/src/mixed.rs", "runtime_only"), ("crates/mcore/src/wide.rs", "after")] and len(fingerprints) == 2,
              f"覆盖文件名没归一或行列没对上（得 {covered}，指纹 {len(fingerprints)} 个）",
              "看 normalise_coverage_path（相对路径按树根拼、绝对路径截到 /crates/）、UnitTree.offset_of_byte_column（llvm-cov 的列按字节）与 covered_units")
        # 挑测试的 -E 按块切：第三轮正推量的全名 133 410 字节
        names = [("singlefs-harness::random_histories", f"module_{index:04d}::a_rather_long_test_name_that_pads_the_filter_out") for index in range(1500)]
        chunks = nextest_filters(names)
        total = sum(len(chunk.encode()) for chunk in chunks)
        check(total > SINGLE_ARGUMENT_LIMIT and all(len(chunk.encode()) <= NEXTEST_FILTER_CHUNK_BYTES for chunk in chunks)
              and sum(chunk.count("test(=") for chunk in chunks) == len(names),
              f"挑测试的 -E 没按块切（{len(chunks)} 块，最长 {max(len(chunk.encode()) for chunk in chunks)} 字节，合计 {total}）",
              f"看 nextest_filters：单个参数不许超过 {SINGLE_ARGUMENT_LIMIT} 字节（MAX_ARG_STRLEN），每块不超过 NEXTEST_FILTER_CHUNK_BYTES")
        # C4：倍数盖住 SMT 2.0 倍
        table = limits_table({("bin", "t"): (10.0, 12.0)}, 3.0, 30.0, 10.0, 300.0)
        check(check_cpu_multiplier(1.5) is not None and check_cpu_multiplier(DEFAULT_CPU_MULTIPLIER) is None and table[("bin", "t")] == (60.0, 420.0)
              and table[("*", "*")] == (60.0, 420.0),
              f"CPU 限时的倍数或上限算错（1.5 倍{'没' if check_cpu_multiplier(1.5) is None else ''}拒；表 {table}）",
              "看 check_cpu_multiplier 与 limits_table：上限 = 倍数 × 基线 CPU 秒 + 下限，倍数不小于 2.0")
        selftest_runner(work, check)
    finally:
        shutil.rmtree(work, ignore_errors=True)
    if failures[0]:
        print(f"  ✗ mutation-prune.py --selftest：{cells[0]} 格里 {failures[0]} 格判错", file=sys.stderr)
        print(f"     → 怎么办：照上面每一格的出路改，再跑 python3 {os.path.join(SCRIPT_DIRECTORY, 'mutation-prune.py')} --selftest", file=sys.stderr)
        return 1
    print(f"  ✓ mutation-prune.py --selftest：{len(SAMPLES)} 个样本的沿用决定（S34、T16、N37、N45、T17、INDEX 在内）、清单按单元存、X1b / X3 / K1 跑全部、"
          f"覆盖文件名归一、-E 按块切、CPU 限时倍数、runner 量时与限时，{cells[0]} 格都对")
    return 0


def selftest_runner(work, check):
    """runner：measure 记 CPU 秒；limit 下吃 CPU 的被 CPU 上限停、只睡的被墙钟兜底停；--list 照原样 exec。起的是假测试程序（python），不起 cargo。
    假测试程序按 CPU 秒忙等（time.process_time()，short 0.3 秒、burn 2.5 秒），不按墙钟：机器负载高时按墙钟等拿不到那么多 CPU，
    量到的 CPU 秒掉到下限 0.15 以下、burn 也碰不到 1 秒的 CPU 上限，这两格跟开关无关地红（2026-10-01 负载 46 时实测 0.105 CPU 秒）。"""
    import subprocess
    runner = os.path.join(work, "runner.py")
    fake = os.path.join(work, "fake-test-0123456789abcdef")
    with open(runner, "w") as handle:
        handle.write(RUNNER_SCRIPT)
    with open(fake, "w") as handle:
        handle.write("#!/usr/bin/env python3\nimport sys, time\nif '--list' in sys.argv:\n    print('burn: test')\n    sys.exit(0)\n"
                     "name = sys.argv[sys.argv.index('--exact') + 1]\ncpu_budget = 2.5 if name == 'burn' else 0.3\n"
                     "if name == 'sleep':\n    time.sleep(5)\nwhile time.process_time() < cpu_budget:\n    pass\n")
    os.chmod(runner, 0o755)
    os.chmod(fake, 0o755)
    times, limits = os.path.join(work, "times.tsv"), os.path.join(work, "limits.tsv")
    base = {key: value for key, value in os.environ.items() if not key.startswith("BROAD_MUTATION_RUNNER")}
    base["BROAD_MUTATION_BREAK"] = ",".join(sorted(BREAKS))

    def run(mode, test, extra=None):
        environment = dict(base, BROAD_MUTATION_RUNNER_MODE=mode, BROAD_MUTATION_RUNNER_TIMES=times, BROAD_MUTATION_RUNNER_LIMITS=limits, **(extra or {}))
        return subprocess.run([sys.executable, runner, fake, "--exact", test, "--nocapture"], capture_output=True, text=True, env=environment, timeout=60)
    measured = run("measure", "short")
    recorded = read_times(times)
    check(measured.returncode == 0 and 0.15 < recorded.get(("fake-test", "short"), (0, 0))[0] < 2.0,
          f"runner 的 measure 没记下 CPU 秒（退 {measured.returncode}，记的 {recorded}）：{measured.stderr}",
          "看 RUNNER_SCRIPT：wait4 取子进程的用户加系统时间，二进制名去掉 -<16 位哈希>")
    write_limits(limits, {("fake-test", "burn"): (1.0, 30.0), ("fake-test", "sleep"): (30.0, 1.0), ("*", "*"): (30.0, 30.0)})
    burned = run("limit", "burn")
    slept = run("limit", "sleep")
    listed = subprocess.run([sys.executable, runner, fake, "--list"], capture_output=True, text=True, timeout=60,
                            env=dict(base, BROAD_MUTATION_RUNNER_MODE="limit", BROAD_MUTATION_RUNNER_LIMITS=limits))
    check(burned.returncode != 0 and "kind=cpu" in burned.stderr and slept.returncode == 124 and "kind=wall" in slept.stderr
          and listed.returncode == 0 and "burn: test" in listed.stdout,
          f"runner 的限时不对（吃 CPU 的退 {burned.returncode}：{burned.stderr.strip()[-200:]}；只睡的退 {slept.returncode}：{slept.stderr.strip()[-200:]}；--list 退 {listed.returncode}）",
          "看 RUNNER_SCRIPT：limit 模式给子进程设 RLIMIT_CPU（SIGXCPU 停了报 kind=cpu），墙钟到点停整个进程组报 kind=wall、退 124；--list 不限时")


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:2] == ["--index"] and len(sys.argv) >= 3:
        tree = UnitTree(sys.argv[2])
        for path in sys.argv[3:] or sorted(tree.files):
            print(f"FILE\t{path}\tunits={len(tree.code_units(path))}")
            for unit in tree.code_units(path):
                print(f"UNIT\t{path}:{unit.first_line}-{unit.last_line}\t{unit.kind}\t{unit.name}\t{unit.fingerprint[:12]}")
        sys.exit(0)
    if sys.argv[1:] != ["--selftest"]:
        print("  ✗ 用法：mutation-prune.py --selftest | --index <树> [文件…]", file=sys.stderr)
        print("  → 怎么办：看文件头的用法；这份库平时由 scripts/mutation/broad-mutation.py 载入", file=sys.stderr)
        sys.exit(2)
    BREAKS = frozenset(name for name in os.environ.get("BROAD_MUTATION_BREAK", "").split(",") if name)
    sys.exit(run_selftest())
