#!/usr/bin/env python3
# admission: always 只读给它的源码文本与覆盖输出现算，不记上一次的结果；--selftest 判的是此刻这份规则
# run-condition: command python3
"""广谱变异驱动（research/scripts/broad-mutation.py）的剪枝规则库：判「这次改到的」、结论能不能沿用、版本内按覆盖挑哪些测试。

依据（三轮设计判决，后一轮覆盖前一轮的同一格）：research/prompts/tooling-broad-mutation-prune-r1-main-verification.md（B1–B8）、
…-r2-main-verification.md（方向：只判改到的函数 + 版本内按覆盖挑测试）、…-r3-main-verification.md（C1–C6 加 E1–E6）。

  mutation-prune.py --selftest            自证：判决点名的样本（第一轮 S1、S2、S4，第二轮 FA、FA2、FF、FB、FC、FD、FH，第三轮 X1b、X3、X7、Y1–Y9）
                                          与本库自己加的 T1、N1、K1、Q1、OUT，逐个判剪枝的决定；不起 cargo、不起子进程（runner 那几格起一个假测试程序）
  mutation-prune.py --index <树> [文件…]   打每个文件的切法：函数项（限定名、起止行、词法单元指纹）、函数外部分的指纹、依赖指纹（排查用）

为什么不并进驱动：驱动是库目录、逐步命令、续跑那一层（两千多行）；这里只是纯函数（切词、切函数项、算键与指纹、读覆盖、挑测试、算限时），
  自证不碰库目录也不起 cargo，单独跑得快；驱动按文件路径载入它（importlib），重型测试闸只按驱动的名字认，这份库自己不起 cargo。

规则（每条后面是弄坏开关 BROAD_MUTATION_BREAK=<项>，只在 --selftest 与驱动的自证里生效）：
  E2 「函数改没改」比整个 fn 项（属性与文档注释、签名、where、函数体）的词法单元序列；普通注释与空白不算，串字面量逐字算
     （regex-function-text：退回正则去注释折空白；body-only-function-text：只比函数体）。
  C1 变异的容器取包住它的最外层 fn 项（嵌套 fn、闭包算外层的）（innermost-function-key）；文件里函数外面的部分按词法单元比，
     变了整个文件的变异都重判（outside-changes-ignored）；切不开的文件退回按整个文件比。新加的函数在结论表里没有键，自然重判。
  E3 宏：文件里调到的宏名，凡在树里有 macro_rules! 定义的，定义它的文件的指纹进这个文件的依赖指纹（macro-dependencies-ignored）；
     include_str! / include_bytes! / include! 读的文件的 sha256 进依赖指纹，参数不是单个串字面量的，取整棵树的指纹（include-dependencies-ignored）。
     抓到它的那条测试的 fn 项指纹存进结论表，变了或没了就重判（test-fingerprint-ignored）。
  E4 全局键（驱动算）变了整版重判（global-key-ignored）；无效的条每次重判（unviable-carried）；没红的条每次重判（missed-carried，本库加的：
     提交门禁对任何没红都红，沿用会把新加的、能杀它的测试挡在外面）。
  E5 键 =（文件、限定名、函数内相对起止、替换文）；限定名带 mod 路径与 impl / trait 头，相对起止按词法单元下标记（key-by-bare-name：退回裸函数名）。
  C2/E1 版本内按覆盖挑测试的例外，一律跑全部：变异不在任何 fn 项里（常量初始化式等，outside-mutation-selected）；容器是常量上下文里用到的
     const fn（按名字沿常量上下文与 const fn 函数体闭包，const-context-runs-selected）；起子进程的测试二进制每次整个跑（spawning-binary-selected）；
     文档测试每次跑（doc-tests-skipped，驱动那一头）。
  B7 覆盖文件名归一到树内相对路径：相对路径按树根拼、绝对路径先按树根截，截不了按最后一个 /crates/ 截，截出来的不在树里的不认（coverage-paths-unnormalized）。
  选测试的 -E 按块切，每块不超过 NEXTEST_FILTER_CHUNK_BYTES（nextest-filter-single-argument：拼成一个参数，单参数上限 131072 字节）。
  C4 单条测试限时 = 倍数 × 基线 CPU 秒 + 下限，倍数不小于 MINIMUM_CPU_MULTIPLIER（第三轮攻方量的 SMT 兄弟被占 2.0 倍）；墙钟兜底另算
     （limit-below-smt：倍数下限不核；cpu-limit-ignored：runner 不设 CPU 上限只看墙钟）。

弄坏开关 BROAD_MUTATION_BREAK=<项>（逗号分隔，只给 --selftest 证红用：本库在 --selftest 时读它、RUNNER_SCRIPT 写出的 runner 从环境读它；驱动只在它自己 --selftest 起的子进程里认、真跑时设了就拒，runner 因此也只在自证里见得到它）。
逐项写开关名、弄坏哪一处、打开后本库 --selftest 哪几格红（项名是那一格：样本名、X1b / X3 / K1，或格的说法；括号里是那条「✗ 自证：」的开头；红了退 1）。
驱动 broad-mutation.py 的 --selftest 打开同一个开关时红在哪，写在它自己的文件头：
  BROAD_MUTATION_BREAK=regex-function-text  FileIndex.function_fingerprint() 退回正则去注释折空白，不按词法单元序列比
      样本 Y1（「✗ 自证：样本 Y1 判成 carry」）、样本 Y2（「✗ 自证：样本 Y2 判成 carry」）、样本 Y3（「✗ 自证：样本 Y3 判成 carry」）、
      样本 X7（「✗ 自证：样本 X7 判成 carry」）
  BROAD_MUTATION_BREAK=body-only-function-text  function_fingerprint() 与 mutation_key() 只比函数体，属性、文档注释、签名、where 不算
      样本 Y5（「✗ 自证：样本 Y5 判成 carry」）、样本 X7（「✗ 自证：样本 X7 判成 carry」）
  BROAD_MUTATION_BREAK=innermost-function-key  TreeIndex 取容器时取包住变异的最里层 fn 项，不取最外层
      样本 FF（「✗ 自证：样本 FF 判成 carry」）
  BROAD_MUTATION_BREAK=outside-changes-ignored  mutation_key() 的指纹不带函数外部分、changed_since() 不看函数外部分变没变
      样本 OUT（「✗ 自证：样本 OUT 判成 carry」）
  BROAD_MUTATION_BREAK=macro-dependencies-ignored  依赖指纹不收调到的 macro_rules! 定义所在文件
      样本 Y4（「✗ 自证：样本 Y4 判成 carry」）
  BROAD_MUTATION_BREAK=include-dependencies-ignored  依赖指纹不收 include_str! / include_bytes! / include! 读的文件
      样本 Y8（「✗ 自证：样本 Y8 判成 carry」）
  BROAD_MUTATION_BREAK=test-fingerprint-ignored  carry_decision() 不比抓到它的那条测试的 fn 项指纹
      样本 T1（「✗ 自证：样本 T1 判成 carry」）
  BROAD_MUTATION_BREAK=global-key-ignored  carry_decision() 不比全局键
      样本 Y7（「✗ 自证：样本 Y7 判成 carry」）
  BROAD_MUTATION_BREAK=unviable-carried  carry_decision() 把无效的条照沿用
      样本 Y9（「✗ 自证：样本 Y9 判成 carry」）
  BROAD_MUTATION_BREAK=missed-carried  carry_decision() 把没红的条照沿用
      样本 N1（「✗ 自证：样本 N1 判成 carry」）、样本 Q1（「✗ 自证：样本 Q1 判成 carry」）
  BROAD_MUTATION_BREAK=key-by-bare-name  mutation_key() 的键退回裸函数名，不带 mod 路径与 impl / trait 头
      样本 Q1（「✗ 自证：样本 Q1 判成 carry」）
  BROAD_MUTATION_BREAK=doc-comments-dropped  tokenize() 把文档注释（///、//!、/** */、/*! */）当普通注释丢掉
      样本 X7（「✗ 自证：样本 X7 判成 carry」）
  BROAD_MUTATION_BREAK=outside-mutation-selected  run_everything_reason() 不在任何 fn 项里的变异也按覆盖挑测试
      K1（「✗ 自证：K1：常量初始化式里的变异没判成跑全部」）
  BROAD_MUTATION_BREAK=const-context-runs-selected  run_everything_reason() 常量上下文里用到的 const fn 也按覆盖挑测试
      X3（「✗ 自证：X3：常量上下文里（经 helper 一跳）用到的 const fn 没判成跑全部」）
  BROAD_MUTATION_BREAK=spawning-binary-selected  spawning_binaries() 一个都不认
      X1b（「✗ 自证：X1b：起本二进制当子进程的测试二进制没认出来」）
  BROAD_MUTATION_BREAK=coverage-paths-unnormalized  normalise_coverage_path() 不拼不截，只认原样就在树里的名字
      覆盖文件名归一（「✗ 自证：覆盖文件名没归一或行列没对上」）
  BROAD_MUTATION_BREAK=nextest-filter-single-argument  nextest_filters() 拼成一个参数，不按块切
      -E 按块切（「✗ 自证：挑测试的 -E 没按块切」）
  BROAD_MUTATION_BREAK=limit-below-smt  check_cpu_multiplier() 不核倍数下限
      CPU 限时倍数（「✗ 自证：CPU 限时的倍数或上限算错」）
  BROAD_MUTATION_BREAK=cpu-limit-ignored  RUNNER_SCRIPT 在 limit 模式不设 CPU 上限、只看墙钟
      runner 量时与限时（「✗ 自证：runner 的限时不对」）

gate-similar: broad-mutation.py 它是逐条驱动（库目录、续跑、逐步命令），这里是它载入的纯函数库，判的是同一件事的剪枝那一半，分开是为了自证不碰库目录
"""
import hashlib
import json
import os
import re
import sys

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY, ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402

BREAKS = frozenset()    # 驱动在自证里设；本库 --selftest 自己从 BROAD_MUTATION_BREAK 读
NEXTEST_FILTER_CHUNK_BYTES = 32768
SINGLE_ARGUMENT_LIMIT = 131072
MINIMUM_CPU_MULTIPLIER = 2.0
DEFAULT_CPU_MULTIPLIER = 3.0       # 盖住 SMT 2.0 倍 × 12 线程散到 1.26 倍（第三轮攻方 c4-cpu-time.txt）
DEFAULT_CPU_FLOOR_SECONDS = 30.0
DEFAULT_WALL_MULTIPLIER = 10.0
DEFAULT_WALL_FLOOR_SECONDS = 300.0
OUTSIDE = "<函数外>"
WHOLE_FILE = "<整个文件>"
SPAWN_PATTERNS = re.compile(r"\bcurrent_exe\b|CARGO_BIN_EXE_|\bCommand::new\b|process::Command\b")


def broken(name):
    return name in BREAKS


def digest_of(parts):
    return hashlib.sha256("\x1f".join(parts).encode()).hexdigest()


# ── 切词：串、原始串、字符、生命周期各一个单元，普通注释与空白去掉，文档注释留着（它是属性，也带着文档测试） ─────────────

def identifier_start(character):
    return character == "_" or character.isalpha()


def identifier_continue(character):
    return character == "_" or character.isalnum()


def tokenize(text):
    """[(类, 文字, 起, 止)]，起止是字符下标。类：ident、lifetime、literal、doc、punct。"""
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
            if ((body.startswith("///") and not body.startswith("////")) or body.startswith("//!")) and not broken("doc-comments-dropped"):
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
            if ((body.startswith("/**") and not body.startswith("/***") and body != "/**/") or body.startswith("/*!")) and not broken("doc-comments-dropped"):
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


def regex_normalised(text):
    """弄坏开关 regex-function-text 用的旧读法：正则去块注释与行注释、空白折成一个空格（不认串）。"""
    text = re.sub(r"/\*.*?\*/", " ", text, flags=re.S)
    text = re.sub(r"//[^\n]*", " ", text)
    return re.sub(r"\s+", " ", text).strip()


# ── 切函数项 ───────────────────────────────────────────────────────────────────────────────────────────────

class Function:
    __slots__ = ("qualified", "name", "first", "last", "start", "end", "fingerprint", "is_const", "body_first", "module_path")

    def __init__(self, qualified, name, first, last, start, end, is_const, body_first, module_path):
        self.qualified, self.name, self.first, self.last = qualified, name, first, last
        self.start, self.end, self.is_const, self.body_first, self.module_path = start, end, is_const, body_first, module_path
        self.fingerprint = ""


class FileIndex:
    """一个 .rs 文件的切法。ok 为假：括号配不上，退回按整个文件。"""

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
        texts = [token[1] for token in self.tokens]
        self.file_fingerprint = digest_of(texts)
        covered = set()
        for function in self.functions:
            covered.update(range(function.first, function.last))
            function.fingerprint = self.function_fingerprint(function)
        self.outside_indices = [index for index in range(len(self.tokens)) if index not in covered]
        self.outside_fingerprint = digest_of([texts[index] for index in self.outside_indices]) if self.ok else self.file_fingerprint
        self.invoked_macros = {self.tokens[index][1] for index in range(len(self.tokens) - 1)
                               if self.tokens[index][0] == "ident" and self.tokens[index + 1][1] == "!" and self.tokens[index][1] != "macro_rules"}
        self.includes = self.find_includes()

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
        if broken("regex-function-text"):
            return hashlib.sha256(regex_normalised(self.text[function.start:function.end]).encode()).hexdigest()
        first = function.body_first if broken("body-only-function-text") and function.body_first is not None else function.first
        return digest_of([token[1] for token in self.tokens[first:function.last]])

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

    def find_includes(self):
        found = []
        for index, token in enumerate(self.tokens):
            if token[1] in ("include_str", "include_bytes", "include") and self.text_of(index + 1) == "!" and self.text_of(index + 2) == "(":
                inside = self.tokens[index + 3:self.pairs.get(index + 2, index + 3)]
                literal = inside[0][1] if len(inside) == 1 and inside[0][0] == "literal" else None
                path = None
                if literal is not None:
                    match = re.fullmatch(r'r#*"(.*)"#*|"(.*)"', literal, re.S)
                    path = (match.group(1) if match.group(1) is not None else match.group(2)) if match else None
                found.append(path)
        return found

    def offset(self, line, column):
        """1 起的行、列（列按字符）→ 字符下标。"""
        return self.line_starts[line - 1] + column - 1

    def offset_of_byte_column(self, line, column):
        """llvm-cov 的列按字节：换成字符下标。"""
        start = self.line_starts[line - 1]
        end = self.line_starts[line] - 1 if line < len(self.line_starts) else len(self.text)
        prefix = self.text[start:end].encode("utf-8")[:max(0, column - 1)]
        return start + len(prefix.decode("utf-8", "ignore"))

    def container_at(self, start_offset, end_offset):
        """包住 [起, 止) 的最外层 fn 项；不在任何 fn 项里交 None。弄坏开关 innermost-function-key：交最里层的。"""
        found = [function for function in self.functions if function.start <= start_offset and end_offset <= function.end]
        if not found:
            return None
        return max(found, key=lambda function: function.start) if broken("innermost-function-key") else min(found, key=lambda function: function.start)


# ── 整棵树：宏、include、常量上下文、测试的指纹 ───────────────────────────────────────────────────────────────

def rust_files(root):
    found = []
    base = os.path.join(root, "crates")
    for directory, names, files in os.walk(base):
        names[:] = sorted(name for name in names if name not in ("target", "mutants.out", "mutants.out.old"))
        for name in sorted(files):
            if name.endswith(".rs"):
                found.append(os.path.relpath(os.path.join(directory, name), root))
    return found


class TreeIndex:
    def __init__(self, root, manifest=None):
        self.root = root
        self.manifest = manifest
        self.files = {}
        for path in rust_files(root):
            with open(os.path.join(root, path), encoding="utf-8", errors="replace") as handle:
                self.files[path] = FileIndex(path, handle.read())
        self.macro_files = {}
        for path, index in self.files.items():
            for name in index.macro_definitions:
                self.macro_files.setdefault(name, []).append(path)
        self._tree_fingerprint = None
        self._dependency_cache = {}
        self.const_context_functions = self.compute_const_context_functions()

    def tree_fingerprint(self):
        if self._tree_fingerprint is None:
            if self.manifest is not None:
                self._tree_fingerprint = digest_of([f"{path} {self.manifest[path]}" for path in sorted(self.manifest)])
            else:
                parts = []
                for directory, names, files in os.walk(self.root):
                    names[:] = sorted(name for name in names if name not in ("target",))
                    for name in sorted(files):
                        path = os.path.join(directory, name)
                        if os.path.isfile(path) and not os.path.islink(path):
                            with open(path, "rb") as handle:
                                parts.append(f"{os.path.relpath(path, self.root)} {hashlib.sha256(handle.read()).hexdigest()}")
                self._tree_fingerprint = digest_of(parts)
        return self._tree_fingerprint

    def dependency_fingerprint(self, path):
        """E3：这个文件调到的、树里有定义的宏，定义文件的指纹；它 include 的文件的 sha256。"""
        if path in self._dependency_cache:
            return self._dependency_cache[path]
        index = self.files.get(path)
        entries = []
        if index is not None:
            if not broken("macro-dependencies-ignored"):
                for name in sorted(index.invoked_macros):
                    for definer in sorted(self.macro_files.get(name, [])):
                        entries.append(f"macro {name} {definer} {self.files[definer].file_fingerprint}")
            if not broken("include-dependencies-ignored"):
                for included in index.includes:
                    if included is None:
                        entries.append(f"include-unresolved {self.tree_fingerprint()}")
                        continue
                    full = os.path.normpath(os.path.join(self.root, os.path.dirname(path), included))
                    try:
                        with open(full, "rb") as handle:
                            entries.append(f"include {included} {hashlib.sha256(handle.read()).hexdigest()}")
                    except OSError:
                        entries.append(f"include {included} missing")
        self._dependency_cache[path] = digest_of(entries)
        return self._dependency_cache[path]

    def compute_const_context_functions(self):
        """E1：常量上下文（const / static 项的初始化式、const 块、数组长度、枚举判别值）里出现的名字，沿 const fn 的函数体闭包，交 const fn 的裸名集合。"""
        names, const_bodies = set(), {}
        for index in self.files.values():
            tokens = index.tokens
            for function in index.functions:
                if function.is_const:
                    const_bodies.setdefault(function.name, []).append({token[1] for token in tokens[function.first:function.last] if token[0] == "ident"})
            for position, token in enumerate(tokens):
                text = token[1]
                if text in ("const", "static") and position + 1 < len(tokens):
                    following = index.text_of(position + 1)
                    if following == "{" and position + 1 in index.pairs:
                        names.update(t[1] for t in tokens[position + 2:index.pairs[position + 1]] if t[0] == "ident")
                        continue
                    if following in ("fn", "unsafe", "async", "extern"):
                        continue
                    cursor = position + 1
                    while cursor < len(tokens) and tokens[cursor][1] not in ("=", ";", "{"):
                        cursor = index.pairs[cursor] + 1 if cursor in index.pairs and tokens[cursor][1] in "([" else cursor + 1
                    if cursor < len(tokens) and tokens[cursor][1] == "=":
                        end = cursor + 1
                        while end < len(tokens) and tokens[end][1] != ";":
                            end = index.pairs[end] + 1 if end in index.pairs else end + 1
                        names.update(t[1] for t in tokens[cursor + 1:end] if t[0] == "ident")
                elif text == "[" and position in index.pairs:
                    close, cursor = index.pairs[position], position + 1
                    while cursor < close and tokens[cursor][1] != ";":
                        cursor = index.pairs[cursor] + 1 if cursor in index.pairs else cursor + 1
                    if cursor < close:
                        names.update(t[1] for t in tokens[cursor + 1:close] if t[0] == "ident")
                elif text == "enum":
                    cursor = position + 1
                    while cursor < len(tokens) and tokens[cursor][1] not in ("{", ";"):
                        cursor += 1
                    if cursor in index.pairs:
                        close, inside = index.pairs[cursor], cursor + 1
                        while inside < close:
                            if tokens[inside][1] == "=":
                                end = inside + 1
                                while end < close and tokens[end][1] != ",":
                                    end = index.pairs[end] + 1 if end in index.pairs else end + 1
                                names.update(t[1] for t in tokens[inside + 1:end] if t[0] == "ident")
                                inside = end
                            inside = index.pairs[inside] + 1 if inside in index.pairs else inside + 1
        found, frontier = set(), names & set(const_bodies)
        while frontier:
            found |= frontier
            reached = set()
            for name in frontier:
                for body in const_bodies[name]:
                    reached |= body
            frontier = (reached & set(const_bodies)) - found
        return found

    def package_fingerprint(self, package_directory):
        return digest_of([f"{path} {index.file_fingerprint}" for path, index in sorted(self.files.items())
                          if path.startswith(package_directory.rstrip("/") + "/")])


def mutation_key(tree, item):
    """一条变异的键与它此刻的指纹：{key: [文件, 容器, 相对起止, 替换文], container_kind, fingerprint, dependency, const_context}。
    容器：包住它的最外层 fn 项的限定名；不在 fn 项里是 OUTSIDE；文件切不开是 WHOLE_FILE。相对起止按容器里的词法单元下标与单元内偏移。"""
    path = item["file"]
    index = tree.files.get(path)
    if index is None:
        return {"key": [path, WHOLE_FILE, f"{item['start'][0]}:{item['start'][1]}-{item['end'][0]}:{item['end'][1]}", item["replacement"]],
                "container_kind": "file", "fingerprint": "unindexed", "dependency": "", "const_context": False, "function": None}
    start, end = index.offset(*item["start"]), index.offset(*item["end"])
    if not index.ok:
        positions, container, kind, fingerprint, function = list(range(len(index.tokens))), WHOLE_FILE, "file", index.file_fingerprint, None
    else:
        function = index.container_at(start, end)
        if function is None:
            positions, container, kind, fingerprint = index.outside_indices, OUTSIDE, "outside", index.outside_fingerprint
        else:
            body_only = broken("body-only-function-text") and function.body_first is not None
            positions = list(range(function.body_first if body_only else function.first, function.last))
            container, kind = function.qualified, "function"
            fingerprint = function.fingerprint + " " + index.outside_fingerprint
    if broken("outside-changes-ignored") and kind == "function":
        fingerprint = function.fingerprint
    relative = relative_span(index, positions, start, end)
    name = container
    if broken("key-by-bare-name") and function is not None:
        name = function.name
    return {"key": [path, name, relative, item["replacement"]], "container_kind": kind, "fingerprint": fingerprint,
            "dependency": tree.dependency_fingerprint(path),
            "const_context": function is not None and function.is_const and function.name in tree.const_context_functions,
            "function": function}


def relative_span(index, positions, start, end):
    if not positions:
        return f"{start}-{end}"
    tokens = index.tokens
    first = next((number for number, position in enumerate(positions) if tokens[position][3] > start), len(positions) - 1)
    last = max((number for number, position in enumerate(positions) if tokens[position][2] < end), default=first)
    return f"{first}.{max(0, start - tokens[positions[first]][2])}-{last}.{end - tokens[positions[last]][2]}"


# ── 结论能不能沿用（C3 + E2–E5） ───────────────────────────────────────────────────────────────────────────

CARRIED_OUTCOMES = ("caught",)


def carry_decision(record, current, global_key, test_fingerprint_now):
    """record 是结论表里这个键最近的一行（dict），current 是 mutation_key 交的；交 (能不能沿用, 理由)。"""
    if record is None:
        return False, "no-record"
    if record["global_key"] != global_key and not broken("global-key-ignored"):
        return False, "global-key-changed"
    if record["outcome"] == "unviable" and not broken("unviable-carried"):
        return False, "unviable-rejudged"
    if record["outcome"] == "missed" and not broken("missed-carried"):
        return False, "missed-rejudged"
    if record["outcome"] not in CARRIED_OUTCOMES + (("unviable",) if broken("unviable-carried") else ()) + (("missed",) if broken("missed-carried") else ()):
        return False, f"outcome-{record['outcome']}"
    if record["fingerprint"] != current["fingerprint"]:
        return False, "function-or-outside-changed"
    if record["dependency"] != current["dependency"]:
        return False, "macro-or-include-changed"
    if record["outcome"] == "caught" and record["test_fingerprint"] != test_fingerprint_now and not broken("test-fingerprint-ignored"):
        return False, "catching-test-changed"
    return True, "carried"


def changed_since(old_tree, new_tree, item, current):
    """C1：对着旧树判这条变异算不算「这次改到的」：交理由或 None（没改）。"""
    old_index = old_tree.files.get(item["file"])
    if old_index is None:
        return "file-new"
    if current["container_kind"] == "file" or not old_index.ok:
        return None if old_index.file_fingerprint == new_tree.files[item["file"]].file_fingerprint else "file-changed"
    new_index = new_tree.files[item["file"]]
    if old_index.outside_fingerprint != new_index.outside_fingerprint and not broken("outside-changes-ignored"):
        return "outside-changed"
    if old_tree.dependency_fingerprint(item["file"]) != current["dependency"]:
        return "macro-or-include-changed"
    if current["container_kind"] == "outside":
        return None
    function = current["function"]
    old = next((candidate for candidate in old_index.functions if candidate.qualified == function.qualified), None)
    if old is None:
        return "function-new"
    return None if old.fingerprint == function.fingerprint else "function-changed"


# ── 测试的指纹（E3）：(二进制 id, 用例名) → 那条测试 fn 项的指纹 ─────────────────────────────────────────────

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


def test_fingerprints(tree, packages):
    """{(二进制 id, 用例名): 指纹}，用例名照 nextest（模块路径::fn 名）；另交 {包: 整包指纹}（认不出的测试退回它）。"""
    found, package_fallback = {}, {}
    for package in packages:
        directory = f"crates/{package}"
        package_fallback[package] = tree.package_fingerprint(directory)
        for kind in ("src", "tests"):
            base = f"{directory}/{kind}/"
            for path, index in tree.files.items():
                if not path.startswith(base):
                    continue
                binary, module = binary_identity(package, kind, path[len(base):])
                for function in index.functions:
                    if any(part.startswith("<") for part in function.qualified.split("::")):
                        continue
                    found[(binary, "::".join(module + function.module_path + [function.name]))] = function.fingerprint
    return found, package_fallback


def catching_test_fingerprint(fingerprints, fallbacks, caught_by):
    """caught_by 是结果行里的「抓到它的第一条测试」（nextest 的「二进制 id 用例名」，或 stage 前缀的别的写法）。"""
    binary, _, test = caught_by.partition(" ")
    if (binary, test) in fingerprints:
        return fingerprints[(binary, test)]
    package = binary.split("::", 1)[0]
    return "package:" + fallbacks.get(package, digest_of(sorted(fallbacks.values())))


# ── 覆盖（C2、B7） ─────────────────────────────────────────────────────────────────────────────────────────

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


def covered_containers(export, tree):
    """llvm-cov export 的 JSON（dict）→ 执行到的容器集合 {(文件, 容器)}：计数大于 0 的函数，按它第一段代码区域落在哪个最外层 fn 项里。"""
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
            index = tree.files[path]
            line, column = min((region[0], region[1]) for region in regions)
            if line > len(index.line_starts):
                continue
            offset = index.offset_of_byte_column(line, column)
            container = index.container_at(offset, offset) if index.ok else None
            found.add((path, WHOLE_FILE if not index.ok else container.qualified if container else OUTSIDE))
    return found


def run_everything_reason(current, spawning_selected=False):
    """C2 的例外：这条变异要跑全部测试，交理由；按覆盖挑就交 None。"""
    if current["container_kind"] != "function" and not broken("outside-mutation-selected"):
        return "not-in-a-function"
    if current["const_context"] and not broken("const-context-runs-selected"):
        return "const-fn-in-const-context"
    return None


def spawning_binaries(tree, packages, profile_counts=None):
    """E1：起子进程的测试二进制（读源码：current_exe、CARGO_BIN_EXE_、Command::new），加覆盖那一遍里有测试写出不止一份 profraw 的二进制。"""
    if broken("spawning-binary-selected"):
        return set()
    found = set()
    for package in packages:
        for kind in ("src", "tests"):
            base = f"crates/{package}/{kind}/"
            for path, index in tree.files.items():
                if path.startswith(base) and SPAWN_PATTERNS.search(index.text):
                    found.add(binary_identity(package, kind, path[len(base):])[0])
    for (binary, _test), count in (profile_counts or {}).items():
        if count > 1:
            found.add(binary)
    return found


def select_tests(coverage, container, spawning):
    """覆盖表 {(二进制, 用例): {(文件, 容器)}} → 覆盖到 container 的用例，加起子进程的二进制里的全部用例；按名字排。"""
    chosen = {test for test, covered in coverage.items() if container in covered}
    chosen |= {test for test in coverage if test[0] in spawning}
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


# ── 自证：判决点名的样本，逐个判剪枝的决定 ─────────────────────────────────────────────────────────────────
# 每个样本：(名字, 期望, 守它的弄坏开关, 第一版的文件, 第二版换掉的文件, 变异 (文件, 原文, 替换文, 第几处), 结论表里那一行的结局)。
# 期望 rejudge：剪枝必须判「重判」，判成沿用就红（开关打开时那一格红）。期望 carry：用户定的「别处改动」那一类（第二轮判决第四节、第三轮判决第二节第 5 行），
# 提交时照沿用，靠里程碑末全量（驱动 --milestone-full 列翻面的）兜底；这几格钉的是「提交时的规则看不见它」这件事本身，驱动自证的里程碑那一格钉兜底。
# 量过的翻面：第一轮 S1、S2、S4a、S4b 在 research/prompts/tooling-broad-mutation-prune-r1-opus-model/run-output.txt，第二轮 FA–FH 在 …-r2-opus-model/run-output.txt，
# 第三轮 Y1–Y9 在 …-r3-opus-model/run-output.txt 第 71–138 行；这里的源码是照它们缩写的同形样本。
SAMPLE_BASE = {"Cargo.toml": "[workspace]\n", "crates/mharness/tests/t.rs": "#[test]\nfn catches() {\n    assert_eq!(mcore::value(), 2);\n}\n"}
SAMPLE_TEST = "mharness::t catches"
SAMPLES = [
    ("Y1", "rejudge", "regex-function-text", {"crates/mcore/src/y1.rs": 'pub fn label(n: u64) -> String {\n    let pad = "a  ";\n    format!("{}{}", pad, n + 1)\n}\n'},
     {"crates/mcore/src/y1.rs": 'pub fn label(n: u64) -> String {\n    let pad = "a ";\n    format!("{}{}", pad, n + 1)\n}\n'}, ("crates/mcore/src/y1.rs", "n + 1", "n - 1", 0), "caught"),
    ("Y2", "rejudge", "regex-function-text", {"crates/mcore/src/y2.rs": 'pub fn y2(n: u64) -> u64 {\n    let open = "/*";\n    let m = n + 1;\n    let close = "*/";\n    m + open.len() as u64 + close.len() as u64\n}\n'},
     {"crates/mcore/src/y2.rs": 'pub fn y2(n: u64) -> u64 {\n    let open = "/*";\n    let m = n + 2;\n    let close = "*/";\n    m + open.len() as u64 + close.len() as u64\n}\n'}, ("crates/mcore/src/y2.rs", "m + open", "m - open", 0), "caught"),
    ("Y3", "rejudge", "regex-function-text", {"crates/mcore/src/y3.rs": 'pub fn y3(n: u64) -> u64 {\n    let s = "//"; let m = n + 1;\n    m * 2 + s.len() as u64\n}\n'},
     {"crates/mcore/src/y3.rs": 'pub fn y3(n: u64) -> u64 {\n    let s = "//"; let m = n + 2;\n    m * 2 + s.len() as u64\n}\n'}, ("crates/mcore/src/y3.rs", "m * 2", "m / 2", 0), "caught"),
    ("Y5", "rejudge", "body-only-function-text",
     {"crates/mcore/src/y5.rs": "pub trait Weigh {\n    fn weigh(&self) -> u64;\n}\npub trait Measure {\n    fn weigh(&self) -> u64;\n}\npub fn total<T: Weigh>(item: &T) -> u64 {\n    item.weigh() + 1\n}\n"},
     {"crates/mcore/src/y5.rs": "pub trait Weigh {\n    fn weigh(&self) -> u64;\n}\npub trait Measure {\n    fn weigh(&self) -> u64;\n}\npub fn total<T>(item: &T) -> u64 where T: Measure {\n    item.weigh() + 1\n}\n"},
     ("crates/mcore/src/y5.rs", "item.weigh() + 1", "item.weigh() - 1", 0), "caught"),
    ("Y4", "rejudge", "macro-dependencies-ignored",
     {"crates/mcore/src/y4_macros.rs": "macro_rules! bump {\n    ($e:expr) => {\n        $e + 1\n    };\n}\n", "crates/mcore/src/y4.rs": "pub fn y4(n: u64) -> u64 {\n    bump!(n) * 3\n}\n"},
     {"crates/mcore/src/y4_macros.rs": "macro_rules! bump {\n    ($e:expr) => {\n        $e + 2\n    };\n}\n"}, ("crates/mcore/src/y4.rs", "* 3", "/ 3", 0), "caught"),
    ("Y8", "rejudge", "include-dependencies-ignored", {"crates/mcore/src/y8.rs": 'pub fn y8() -> usize {\n    include_str!("y8_k.txt").len() + 1\n}\n', "crates/mcore/src/y8_k.txt": "abc\n"},
     {"crates/mcore/src/y8_k.txt": "abcd\n"}, ("crates/mcore/src/y8.rs", "+ 1", "- 1", 0), "caught"),
    ("Y7", "rejudge", "global-key-ignored", {"Cargo.toml": "[workspace]\n[profile.dev]\noverflow-checks = true\n", "crates/mcore/src/y7.rs": "pub fn y7(n: u8) -> u8 {\n    n + 200\n}\n"},
     {"Cargo.toml": "[workspace]\n[profile.dev]\noverflow-checks = false\n"}, ("crates/mcore/src/y7.rs", "n + 200", "n * 200", 0), "caught"),
    ("Y9", "rejudge", "unviable-carried", {"crates/mcore/src/y9_type.rs": "pub struct W(pub u64);\n", "crates/mcore/src/y9.rs": "use crate::y9_type::W;\npub fn make() -> W {\n    W(1)\n}\n"},
     {"crates/mcore/src/y9_type.rs": "#[derive(Default)]\npub struct W(pub u64);\n"}, ("crates/mcore/src/y9.rs", "W(1)", "Default::default()", 0), "unviable"),
    ("FF", "rejudge", "innermost-function-key", {"crates/mcore/src/nested.rs": "pub fn outer(value: u64) -> u64 {\n    fn inner(input: u64) -> u64 {\n        input + 1\n    }\n    inner(value) * 2\n}\n"},
     {"crates/mcore/src/nested.rs": "pub fn outer(value: u64) -> u64 {\n    fn inner(input: u64) -> u64 {\n        input + 1\n    }\n    (value + 1) * 2\n}\n"},
     ("crates/mcore/src/nested.rs", "input + 1", "input - 1", 0), "caught"),
    ("X7", "rejudge", "doc-comments-dropped", {"crates/mcore/src/doc.rs": "/// ```\n/// assert_eq!(mcore::doc::halve(10), 5);\n/// ```\npub fn halve(n: u64) -> u64 {\n    n / 2\n}\n"},
     {"crates/mcore/src/doc.rs": "/// ```\n/// assert_eq!(mcore::doc::halve(12), 6);\n/// ```\npub fn halve(n: u64) -> u64 {\n    n / 2\n}\n"}, ("crates/mcore/src/doc.rs", "n / 2", "n * 2", 0), "caught"),
    ("T1", "rejudge", "test-fingerprint-ignored", {"crates/mcore/src/t1.rs": "pub fn value() -> u64 {\n    1 + 1\n}\n"},
     {"crates/mharness/tests/t.rs": "#[test]\nfn catches() {\n    assert!(mcore::value() > 0);\n}\n"}, ("crates/mcore/src/t1.rs", "1 + 1", "1 * 1", 0), "caught"),
    ("N1", "rejudge", "missed-carried", {"crates/mcore/src/n1.rs": "pub fn rarely(n: u64) -> u64 {\n    n + 7\n}\n"},
     {"crates/mharness/tests/n1.rs": "#[test]\nfn now_covered() {\n    assert_eq!(mcore::n1::rarely(1), 8);\n}\n"}, ("crates/mcore/src/n1.rs", "n + 7", "n - 7", 0), "missed"),
    ("Q1", "rejudge", "key-by-bare-name",
     {"crates/mcore/src/q1.rs": "pub struct A;\npub struct B;\nimpl A {\n    pub fn value(&self) -> u64 {\n        7 + 1\n    }\n}\nimpl B {\n    pub fn value(&self) -> u64 {\n        7 + 1\n    }\n}\n"},
     {}, ("crates/mcore/src/q1.rs", "7 + 1", "7 - 1", 1), "missed"),
    ("OUT", "rejudge", "outside-changes-ignored", {"crates/mcore/src/out.rs": "const STEP: u64 = 1;\npub fn out(n: u64) -> u64 {\n    n + STEP\n}\n"},
     {"crates/mcore/src/out.rs": "const STEP: u64 = 2;\npub fn out(n: u64) -> u64 {\n    n + STEP\n}\n"}, ("crates/mcore/src/out.rs", "n + STEP", "n - STEP", 0), "caught"),
    # 「别处改动」：提交时照沿用，里程碑末全量兜底
    ("S1", "carry", "-", {"crates/fmtc/src/lib.rs": "pub const CAPACITY: u64 = 4;\n", "crates/mcore/src/split.rs": "pub fn pieces(length: u64, capacity: u64) -> u64 {\n    (length + capacity - 1) / capacity\n}\n"},
     {"crates/fmtc/src/lib.rs": "pub const CAPACITY: u64 = 1;\n"}, ("crates/mcore/src/split.rs", "(length + capacity - 1)", "(length + capacity + 1)", 0), "caught"),
    ("S2", "carry", "-", {"crates/fmtc/src/lib.rs": "pub const SLOTS: u64 = 8;\n", "crates/mcore/src/layout.rs": "pub const REGION_SLOTS: u64 = fmtc::SLOTS;\n",
                          "crates/mcore/src/ring.rs": "use crate::layout::REGION_SLOTS;\npub fn wrap(publish: u64) -> bool {\n    if publish >= REGION_SLOTS { true } else { false }\n}\n"},
     {"crates/fmtc/src/lib.rs": "pub const SLOTS: u64 = 9;\n"}, ("crates/mcore/src/ring.rs", "publish >= REGION_SLOTS", "publish > REGION_SLOTS", 0), "caught"),
    ("S4", "carry", "-", {"crates/mcore/src/extra.rs": "pub fn route(n: u64) -> u64 {\n    n\n}\n", "crates/mcore/src/render.rs": "pub fn render(number: u64) -> u64 {\n    number + 1\n}\n"},
     {"crates/mcore/src/extra.rs": "pub fn route(n: u64) -> u64 {\n    crate::render::render(n)\n}\n"}, ("crates/mcore/src/render.rs", "number + 1", "number - 1", 0), "caught"),
    ("FA", "carry", "-", {"crates/mcore/src/slots.rs": 'pub fn slot_of(offset: u64) -> u64 {\n    offset / 4096\n}\n\npub fn describe(offset: u64) -> String {\n    format!("slot-{}", slot_of(offset))\n}\n'},
     {"crates/mcore/src/slots.rs": 'pub fn slot_of(offset: u64) -> u64 {\n    offset / 4096\n}\n\npub fn describe(offset: u64) -> String {\n    format!("slot-{}", offset / 4096)\n}\n'},
     ("crates/mcore/src/slots.rs", "offset / 4096", "offset % 4096", 0), "caught"),
    ("FA2", "carry", "-", {"crates/mcore/src/thing.rs": 'pub struct Thing(pub u64);\npub trait Describe {\n    fn describe(&self) -> String;\n}\nimpl Describe for Thing {\n    fn describe(&self) -> String {\n        format!("thing-{}", self.0 + 1)\n    }\n}\nimpl Thing {\n    pub fn new(value: u64) -> Self {\n        Thing(value)\n    }\n}\n'},
     {"crates/mcore/src/thing.rs": 'pub struct Thing(pub u64);\npub trait Describe {\n    fn describe(&self) -> String;\n}\nimpl Describe for Thing {\n    fn describe(&self) -> String {\n        format!("thing-{}", self.0 + 1)\n    }\n}\nimpl Thing {\n    pub fn new(value: u64) -> Self {\n        Thing(value)\n    }\n\n    pub fn describe(&self) -> String {\n        format!("thing-{}", self.0 + 1)\n    }\n}\n'},
     ("crates/mcore/src/thing.rs", "self.0 + 1", "self.0 - 1", 0), "caught"),
    ("FB", "carry", "-", {"crates/mcore/src/codec.rs": "pub use crate::codec_strict::check;\n", "crates/mcore/src/frame.rs": "pub fn encode_len(length: u64) -> u64 {\n    length + 1\n}\n"},
     {"crates/mcore/src/codec.rs": "pub use crate::codec_lenient::check;\n"}, ("crates/mcore/src/frame.rs", "length + 1", "length - 1", 0), "caught"),
    ("FC", "carry", "-", {"crates/mcore/src/word.rs": "pub type Word = u16;\n", "crates/mcore/src/pack.rs": "use crate::word::Word;\npub fn pack(high: u64, low: u64) -> Word {\n    ((high << 8) | low) as Word\n}\n"},
     {"crates/mcore/src/word.rs": "pub type Word = u8;\n"}, ("crates/mcore/src/pack.rs", "high << 8", "high >> 8", 0), "caught"),
    ("FD", "carry", "-", {"crates/mcore/src/kind.rs": "pub enum Kind {\n    Data = 1,\n    Meta = 3,\n}\n", "crates/mcore/src/tag.rs": "use crate::kind::Kind;\npub fn tag(kind: Kind, count: u64) -> u64 {\n    (kind as u64) * count\n}\n"},
     {"crates/mcore/src/kind.rs": "pub enum Kind {\n    Data = 2,\n    Meta = 3,\n}\n"}, ("crates/mcore/src/tag.rs", "(kind as u64) * count", "(kind as u64) + count", 0), "caught"),
    ("FH", "carry", "-", {"crates/mcore/src/big.rs": "pub fn scale(value: u64) -> u64 {\n    value * 3\n}\n", "crates/mcore/src/route.rs": "pub fn route(value: u64) -> u64 {\n    if value > 100 { crate::big::scale(value) } else { value * 2 }\n}\n"},
     {"crates/mcore/src/big.rs": "pub fn scale(value: u64) -> u64 {\n    value * 2\n}\n"}, ("crates/mcore/src/route.rs", "value > 100", "value < 100", 0), "caught"),
    ("Y6", "carry", "-", {"crates/mcore/src/y6_trait.rs": "pub trait Score {\n    fn score(&self) -> u64 {\n        1 + 1\n    }\n}\n", "crates/mcore/src/y6_impl.rs": "pub struct P;\nimpl crate::y6_trait::Score for P {}\n"},
     {"crates/mcore/src/y6_impl.rs": "pub struct P;\nimpl crate::y6_trait::Score for P {\n    fn score(&self) -> u64 {\n        2\n    }\n}\n"}, ("crates/mcore/src/y6_trait.rs", "1 + 1", "1 - 1", 0), "caught"),
]
# 版本内挑测试的例外（第三轮 X1b、X3 与本库加的 K1）：期望「跑全部」
C2_SOURCES = {
    "crates/mharness/tests/golden.rs": '#[test]\nfn golden_parent() {\n    let output = std::process::Command::new(std::env::current_exe().unwrap()).arg("child").output().unwrap();\n    assert!(output.status.success());\n}\n',
    "crates/mcore/src/mixed.rs": "pub const fn square(x: u64) -> u64 {\n    x * x\n}\npub const fn helper(x: u64) -> u64 {\n    square(x) + 0\n}\npub fn runtime_only(x: u64) -> u64 {\n    x + 1\n}\n",
    "crates/mharness/tests/consts.rs": "const STRONG: u64 = mcore::mixed::helper(3);\n#[test]\nfn strong() {\n    assert_eq!(STRONG, 9);\n}\n",
    "crates/mcore/src/limits.rs": "pub const LIMIT: u64 = 1 << 8;\n",
}


def write_tree(root, files):
    for path, text in files.items():
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
    return {"file": path, "start": line_column(offset), "end": line_column(offset + len(old)), "replacement": replacement}


def sample_decision(work, sample):
    """交 (决定, 理由)：决定是 carry 或 rejudge。"""
    name, _expected, _switch, first, second, mutation, outcome = sample
    files_one = dict(SAMPLE_BASE, **first)
    files_two = dict(files_one, **second)
    roots = [os.path.join(work, name, "v1"), os.path.join(work, name, "v2")]
    for root, files in zip(roots, (files_one, files_two)):
        write_tree(root, files)
    tree_one, tree_two = TreeIndex(roots[0]), TreeIndex(roots[1])
    store = {}
    for tree, files in ((tree_one, files_one),):
        fingerprints, fallbacks = test_fingerprints(tree, ["mharness"])
        test_now = catching_test_fingerprint(fingerprints, fallbacks, SAMPLE_TEST)
        global_key = digest_of([files["Cargo.toml"]])
        occurrences = [mutation] if name != "Q1" else [mutation, (mutation[0], mutation[1], mutation[2], 0)]
        for found_mutation, found_outcome in zip(occurrences, [outcome, "caught"]):
            current = mutation_key(tree, item_for(files, found_mutation))
            store[tuple(current["key"])] = {"outcome": found_outcome, "global_key": global_key, "fingerprint": current["fingerprint"],
                                            "dependency": current["dependency"], "test_fingerprint": test_now}
    item = item_for(files_two, mutation)
    current = mutation_key(tree_two, item)
    fingerprints, fallbacks = test_fingerprints(tree_two, ["mharness"])
    carry, reason = carry_decision(store.get(tuple(current["key"])), current, digest_of([files_two["Cargo.toml"]]),
                                   catching_test_fingerprint(fingerprints, fallbacks, SAMPLE_TEST))
    changed = changed_since(tree_one, tree_two, item, current)
    return ("carry", reason) if carry and changed is None else ("rejudge", changed or reason)


def run_selftest():
    import shutil
    import subprocess
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
            decision, reason = sample_decision(work, sample)
            check(decision == sample[1], f"样本 {sample[0]} 判成 {decision}（{reason}），该是 {sample[1]}（守它的开关 {sample[2]}）",
                  f"看文件头「规则」里开关 {sample[2]} 那一条；期望 carry 的是用户定的「别处改动」那一类，变了要同时改驱动里程碑那一格的样本")
        root = os.path.join(work, "c2")
        write_tree(root, dict(SAMPLE_BASE, **C2_SOURCES))
        tree = TreeIndex(root)
        check("mharness::golden" in spawning_binaries(tree, ["mharness"]),
              f"X1b：起本二进制当子进程的测试二进制没认出来（认出的 {sorted(spawning_binaries(tree, ['mharness']))}）",
              "看 spawning_binaries：源码里有 current_exe / CARGO_BIN_EXE_ / Command::new 的测试二进制每次整个跑（E1）")
        square = mutation_key(tree, item_for(C2_SOURCES, ("crates/mcore/src/mixed.rs", "x * x", "x + x", 0)))
        runtime = mutation_key(tree, item_for(C2_SOURCES, ("crates/mcore/src/mixed.rs", "x + 1", "x - 1", 0)))
        check(run_everything_reason(square) == "const-fn-in-const-context" and run_everything_reason(runtime) is None,
              f"X3：常量上下文里（经 helper 一跳）用到的 const fn 没判成跑全部（square {run_everything_reason(square)}，runtime_only {run_everything_reason(runtime)}）",
              "看 TreeIndex.compute_const_context_functions：const / static 初始化式里的名字沿 const fn 函数体闭包（E1）")
        constant = mutation_key(tree, item_for(C2_SOURCES, ("crates/mcore/src/limits.rs", "<<", ">>", 0)))
        check(run_everything_reason(constant) == "not-in-a-function", f"K1：常量初始化式里的变异没判成跑全部（{run_everything_reason(constant)}）",
              "看 run_everything_reason：不在任何 fn 项里的变异覆盖记不到，一律跑全部（第一轮 B2）")
        # B7：覆盖文件名归一（别处一份副本的绝对路径、带 .. 的相对路径），列按字节（行里有中文）
        write_tree(root, {"crates/mcore/src/wide.rs": '// 中文注释\npub fn wide() -> &\'static str { "宽字符" }\npub fn after() -> u64 { 1 }\n'})
        tree = TreeIndex(root)
        export = {"data": [{"functions": [
            {"count": 3, "filenames": ["/somewhere/else/copy/crates/mcore/src/mixed.rs"], "regions": [[7, 1, 9, 2, 3, 0, 0, 0]]},
            {"count": 1, "filenames": ["crates/mharness/tests/../../mcore/src/wide.rs"], "regions": [[3, 1, 3, 30, 1, 0, 0, 0]]},
            {"count": 0, "filenames": [os.path.join(root, "crates/mcore/src/mixed.rs")], "regions": [[1, 1, 3, 2, 0, 0, 0, 0]]},
            {"count": 5, "filenames": ["/rustc/abc/library/core/src/num.rs"], "regions": [[1, 1, 2, 2, 5, 0, 0, 0]]}]}]}
        covered = covered_containers(export, tree)
        check(covered == {("crates/mcore/src/mixed.rs", "runtime_only"), ("crates/mcore/src/wide.rs", "after")},
              f"覆盖文件名没归一或行列没对上（得 {sorted(covered)}）",
              "看 normalise_coverage_path（相对路径按树根拼、绝对路径截到 /crates/）与 FileIndex.offset_of_byte_column（llvm-cov 的列按字节）")
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
    print(f"  ✓ mutation-prune.py --selftest：{len(SAMPLES)} 个样本的剪枝决定、X1b / X3 / K1 跑全部、覆盖文件名归一、-E 按块切、CPU 限时倍数、runner 量时与限时，{cells[0]} 格都对")
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
        tree = TreeIndex(sys.argv[2])
        for path in sys.argv[3:] or sorted(tree.files):
            index = tree.files[path]
            print(f"FILE\t{path}\tok={index.ok}\toutside={index.outside_fingerprint[:12]}\tdependency={tree.dependency_fingerprint(path)[:12]}")
            for function in index.functions:
                line = index.text.count("\n", 0, function.start) + 1
                print(f"FN\t{path}:{line}\t{function.qualified}\t{function.fingerprint[:12]}\tconst_context={function.is_const and function.name in tree.const_context_functions}")
        sys.exit(0)
    if sys.argv[1:] != ["--selftest"]:
        print("  ✗ 用法：mutation-prune.py --selftest | --index <树> [文件…]", file=sys.stderr)
        print("  → 怎么办：看文件头的用法；这份库平时由 research/scripts/broad-mutation.py 载入", file=sys.stderr)
        sys.exit(2)
    BREAKS = frozenset(name for name in os.environ.get("BROAD_MUTATION_BREAK", "").split(",") if name)
    sys.exit(run_selftest())
