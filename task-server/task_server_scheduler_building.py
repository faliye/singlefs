"""task_server.py 拆出的一份：Scheduler 的一部分方法：编译目录池、取消后删产物、现场目录、收尾与分片、排队次序、派活、提交时拒的各种判据（Scheduler 在 task_server.py，经继承取到它们）。"""
import os
import task_server_lib as lib
import contextlib
import re
import shlex
import shutil
import sqlite3
import subprocess
import threading
import time
import traceback
from typing import NamedTuple
from task_server_constants import (
    ACTIVE_ATTEMPT_STATES, BUILD_POOL_FULL_PREFIX, BUILD_POOL_SLOTS_PER_TREE, BUILD_POOL_SWEEP_SECONDS, COPY_PROOF_COPY_ADVICE,
    COPY_PROOF_STATE_TEXT, HOOK_SWAP_ENTRY, HOST_INDEPENDENT_PATHS, LOCAL, MERGE_AND_VERIFY_ENTRY, PEER, PEER_REPASS_SECONDS, PEER_TRIP_BUILT_MARKS,
    PEER_TRIP_READ_BACK_MARK, PEER_TRIP_REMOVED_MARK, QUOTED_WORD_PATH, REGISTRY_NO_LIBRARY_PREFIXES, REGISTRY_SERVICE_MARK, RESTORED_HOST_LISTS,
    SHELL_WORD_PATH_WITH_EXPANSION_TAILS, UNPOOLED_SWEEP_SUMMARY_MARK, VERDICT_LIBRARY_REGISTRY, VOLATILE_ROOT, WHOLE_FRACTION, WHOLE_WORKSPACE_FLAGS, WHOLE_WORKSPACE_NEXT_STEP,
    WHOLE_WORKSPACE_SUBCOMMANDS, WORKSPACE_READ_ONLY_ARGUMENTS, WORKSPACE_WRITERS, append_log_lines, cancelled_artifact_lines, directory_bytes,
    log_tail, read_json_file, say,
)
from task_server_store import StoreRefused
from task_server_machines import (
    HEAD_BLOCKED, HEAD_NOT_STARTED, HEAD_STARTED, HeadVerdict, MACHINE_ORDER, PassView, Placement, shell_script_word_indexes, shell_word_paths,
)

# 包在里面的写工作区工具（wrapped_workspace_commands）：命令位置上起的脚本正文只读这么大的（推的：仓里与草稿目录里的串行脚本都远小于它），更大的不读、照不认
WRAPPED_WRITER_SCRIPT_MAXIMUM_BYTES = 4 * 1024 * 1024
# 脚本正文里一条命令开头不是命令词的词（script_body_workspace_commands 剥掉之后再交 command_word_index）：shell 的保留字与只改起法的前缀
SCRIPT_COMMAND_PREFIX_WORDS = ("if", "then", "else", "elif", "do", "while", "until", "!", "exec", "command", "time", "{")
# 切不开的一行（引号不配对：python 的撇号、跨行的引号）照这个取词：命令分隔单列一个词，引号与重定向切开
UNLEXED_LINE_WORD = re.compile(r"[;&|()]+|[^\s'\"`;&|()<>]+")
# 切出的词两头剥掉的括号、逗号与引号：python 正文里 ["python3", "scripts/…"] 切成 [python3, 这一种，剥完是 python3
WORD_EDGE_CHARACTERS = "[](){},'\"`"
# shell_text_pieces 记在分隔里的三个记号：每一行起头一个换行，切不开、照 UNLEXED_LINE_WORD 取词的那一行的每一条命令前一个问号，heredoc 正文那几行的每一条命令前一个 <<
PIECE_LINE_START, PIECE_UNLEXED, PIECE_HEREDOC_BODY = "\n", "?", "<<"
# 花括号组的两头（命令位置上单独一个词）：shell_text_pieces 照分隔原样记进分隔，shell_text_runs 当一层数（接在管道或 & 上的整组跑在子 shell 里，组里的 cd 留不到组外）
SHELL_GROUP_BRACES = ("{", "}")
# 切不开的那一行里的 heredoc（<< 或 <<- 带定界词，<<< 不算）：定界词可带一对引号或一个反斜杠；切得开的行照 shlex 切出的 << 与紧跟的词认
HEREDOC_OPERATOR = re.compile(r"(?<!<)<<(?!<)(-?)\s*\\?(['\"]?)([^\s'\";&|()<>]+)\2")
# shell_text_runs 认的换当前目录的命令词：平铺着跑的 cd、pushd 带一个算得出的目录照算，别的一律让整段的当前目录算不出
SHELL_DIRECTORY_WORDS = ("cd", "pushd", "popd")
# 读进或拼出别的代码的命令词与 case（模式里的右括号数不清层）：出现了，整段的当前目录与变量都算不出（shell_text_runs）
SHELL_OPAQUE_WORDS = ("source", ".", "eval", "case")
# 开一层、关一层的保留字（shell_text_runs 数层：层里的 cd 与赋值不照算）
SHELL_BLOCK_OPENERS = ("if", "for", "while", "until", "select")
SHELL_BLOCK_CLOSERS = ("fi", "done")
# 赋值前面可带的声明词（shell_text_runs：`export 名=值` 与 `名=值` 一样照算）
SHELL_DECLARATION_WORDS = ("export", "readonly", "declare", "typeset", "local")
# 平铺着跑：一条命令前面的分隔只许是行首与分号；后面多许 && 与 ||（这一条本身照跑，后面接的只管它成没成）。管道、放后台、括号与花括号组里的跑在子 shell 里或层里，
# heredoc 正文的行不是这一层跑的命令（分隔里带 PIECE_HEREDOC_BODY），都不算平铺
SHELL_STRAIGHT_BEFORE = (PIECE_LINE_START, ";")
SHELL_STRAIGHT_AFTER = (PIECE_LINE_START, ";", "&&", "||")
# 变量引用 $名字 与 ${名字}（剥过 WORD_EDGE_CHARACTERS 的词末尾的右花括号已去掉，词末的 ${名字 照认）；位置参数、$@、${名字:-默认} 这类不认，代入之后还留着 $ 就是算不出
SHELL_VARIABLE_REFERENCE = re.compile(r"\$\{([A-Za-z_][A-Za-z0-9_]*)(?:\}|$)|\$([A-Za-z_][A-Za-z0-9_]*)")
# 一个词里不是变量引用的名字（前面不是 $、${ 与字母数字）：shell_text_runs 数每个名字在整段里这样出现几次，赋值、for、read、unset 都算一次
SHELL_NAME_MENTION = re.compile(r"(?<![\w$])(?<!\$\{)([A-Za-z_][A-Za-z0-9_]*)")
# 定义了函数的 shell 文字（function 关键字、名字()）：函数体在哪儿被调到认不出，整段的当前目录与变量都算不出（shell_text_runs）
SHELL_FUNCTION_DEFINITION = re.compile(r"(?:^|[\s;&|(])function\s|[A-Za-z_][\w-]*\s*\(\s*\)")


class ShellRun(NamedTuple):
    """shell_text_runs 交的一条命令。"""
    words: list          # 切出的词（同 shell_text_commands）
    expanded: list       # 每个词代入前面认得出的赋值之后的样子；代入不了的（没赋过、赋法认不出、位置参数、命令替换、~ 起头）是 None
    directory: object    # 这一条起时的当前目录（照前面认得出的 cd 算的绝对路径），算不出是 None；这一条自己经 env -C / --chdir 换了目录的也是 None


class WorkspaceWriter(NamedTuple):
    """认到的一条写工作区工具的命令（workspace_writer_commands、wrapped_workspace_commands 交的每一项）。"""
    words: list          # 那一条的词：命令那一层的整条，或 -c 串、脚本正文里切出的那一条（脚本正文的剥过开头的前缀与「名=值」）
    cwd: str             # 任务的工作目录：认它是不是写工具、把词折成相对仓根的路径都照它（modifies_workspace 的判法不跟 cd）
    expanded: list       # 每个词代入认得出的赋值之后的样子，代入不了的是 None（ShellRun.expanded；命令那一层的照原样）
    directory: object    # 这一条真起时的当前目录（ShellRun.directory；命令那一层的是任务的工作目录），算不出是 None


def shell_directory_option(words):
    """命令里起 shell 或脚本之前的那几个词有没有换起的目录（env 的 -C / --chdir）：有的话里面的当前目录从一开始就算不出。"""
    return any(word in ("-C", "--chdir") or word.startswith("--chdir=") for word in words)


def shell_text_pieces(text, environment=None):
    """一段 shell 文字（bash -c 的串、脚本正文）里的每一条命令，连同它与上一条之间的分隔 → [(词表, 分隔)…]，词表不空。
    先接上反斜杠续行、按行切；每一行照 shlex（posix，标点单列）切成 shell 词，# 起的注释到行尾不读，按 ; & | ( ) 与它们连写的（&&、||、;;）切成几条；
    重定向连同它的目标（2>&1 的 2 一并）去掉；每个词两头剥掉 WORD_EDGE_CHARACTERS。切不开的那一行按 UNLEXED_LINE_WORD 取词（以 # 起头的整行不读）。
    不展开变量与命令替换：$(…) 里的命令按括号切成单独一条，$X/… 这类词原样留着。
    分隔是元组：每一行起头记一个 PIECE_LINE_START，标点照切出的原样，切不开的那一行每条命令前记一个 PIECE_UNLEXED；空的命令不交，它前后的分隔并进下一条的。
    命令位置上单独一个词的 { 与 }（SHELL_GROUP_BRACES，花括号组的两头）照标点原样记进分隔，不当词。
    heredoc（<<、<<- 带定界词；切得开的行认 shlex 切出的 << 与紧跟的词，切不开的照 HEREDOC_OPERATOR）：之后到定界词那一行（<<- 的先剥掉行首制表符再比）之前的行是正文，
    正文的行照样切成命令（喂给 bash、python 的正文里写着的命令照认），每一条命令前多记一个 PIECE_HEREDOC_BODY；正文的行里的 << 不再认，定界词那一行不交。
    算术里的 <<（$((1 << 2))）同样当 heredoc 认，之后的行当正文（偏向算不出，shell_text_runs）。
    弄坏开关 brace-group-not-layered 下 { 与 } 照词两头的边缘字符剥掉、不记；heredoc-body-as-commands 下不认 heredoc，正文的行照这一层的命令切、不记。"""
    braces = not lib.broken("brace-group-not-layered", environment)
    heredocs = not lib.broken("heredoc-body-as-commands", environment)
    pieces, joint, pending = [], [], []   # pending：认到了、还没读到定界词那一行的 heredoc，[(定界词, 是不是 <<-)…]，照先后
    for line in text.replace("\\\n", " ").splitlines():
        if pending and (line.lstrip("\t") if pending[0][1] else line) == pending[0][0]:
            pending.pop(0)
            continue
        body = bool(pending)
        try:
            lexer = shlex.shlex(line, posix=True, punctuation_chars=True)
            lexer.whitespace_split = True
            tokens, unlexed = list(lexer), False
        except ValueError:
            tokens = [] if line.lstrip().startswith("#") else UNLEXED_LINE_WORD.findall(line)
            unlexed = bool(tokens)
            if heredocs and not body and tokens:
                pending += [(match.group(3), bool(match.group(1))) for match in HEREDOC_OPERATOR.finditer(line)]
        line_marks = ([PIECE_HEREDOC_BODY] if body else []) + ([PIECE_UNLEXED] if unlexed else [])
        joint.append(PIECE_LINE_START)
        joint += line_marks
        # heredoc_dash：上一个词是 heredoc 的 << 时记 False，<< 之后单独一个 - 时记 True（<<- 与定界词之间隔了空白），别的时候是 None
        current, skip_target, heredoc_dash = [], False, None
        for token in tokens:
            if skip_target:   # 重定向的目标不是命令的词
                skip_target = False
                if heredoc_dash is False and token == "-":
                    skip_target, heredoc_dash = True, True
                    continue
                if heredoc_dash is not None:
                    attached = heredoc_dash is False and token.startswith("-")   # <<-EOF 切成 << 与 -EOF
                    delimiter = token[1:] if attached else token
                    if delimiter:
                        pending.append((delimiter, heredoc_dash or attached))
                    heredoc_dash = None
                continue
            if token and set(token) <= set("&|;()"):
                if current:
                    pieces.append((current, tuple(joint)))
                    joint = list(line_marks)
                joint.append(token)
                current = []
            elif braces and token in SHELL_GROUP_BRACES and not current:
                joint.append(token)
            elif token and set(token) <= set("<>&") and ("<" in token or ">" in token):
                skip_target = True
                if heredocs and not body and token == "<<":
                    heredoc_dash = False
                if current and current[-1].isdigit():   # 2>&1 的 2 是文件描述符
                    current.pop()
            else:
                word = token.strip(WORD_EDGE_CHARACTERS)
                if word:
                    current.append(word)
        if current:
            pieces.append((current, tuple(joint)))
            joint = []
    return pieces


def shell_text_commands(text):
    """一段 shell 文字里的每一条命令 → [[词…]…]（切法在 shell_text_pieces，不要分隔）。"""
    return [words for words, _ in shell_text_pieces(text)]


def shell_command_head(words):
    """一条命令剥掉开头的 SCRIPT_COMMAND_PREFIX_WORDS 与「名=值」之后的头一个词；剥完没有交空串。"""
    start = 0
    while start < len(words) and (words[start] in SCRIPT_COMMAND_PREFIX_WORDS or lib.ASSIGNMENT_WORD.match(words[start])):
        start += 1
    return words[start] if start < len(words) else ""


def shell_expanded_word(word, variables):
    """一个词代入 variables（{名字: 值}，值是 None 的是算不出）里的变量之后的样子；有引用到没有或算不出的变量、代入之后还留着 $ 或反引号、以 ~ 起头的交 None。"""
    missing = []

    def substitute(match):
        value = variables.get(match.group(1) or match.group(2))
        if value is None:
            missing.append(match.group(0))
            return ""
        return value

    result = SHELL_VARIABLE_REFERENCE.sub(substitute, word)
    if missing or "$" in result or "`" in result or result.startswith("~"):
        return None
    return result


def shell_assignments(words):
    """这一条只是赋值（`名=值…`，前面可带 SHELL_DECLARATION_WORDS 之一与它的选项）时 → [(名字, 值)…]；带命令词的（`名=值 命令`）与别的交 []。"""
    rest = list(words)
    if rest and rest[0] in SHELL_DECLARATION_WORDS:
        rest = [word for word in rest[1:] if not word.startswith("-")]
    if not rest or not all(lib.ASSIGNMENT_WORD.match(word) for word in rest):
        return []
    return [tuple(word.split("=", 1)) for word in rest]


def shell_text_runs(text, directory, opaque=False, environment=None):
    """一段 shell 文字（bash -c 的串、脚本正文）里的每一条命令连同它起时认得出的当前目录与变量 → [ShellRun…]，与 shell_text_commands(text) 一条对一条。
    directory 是这段文字起时的当前目录（算不出给 None）；opaque 给 True 的（python 正文：它的 os.chdir 与变量这里读不懂）当前目录与变量一概算不出。
    只照平铺着跑的那几条算（前后的分隔是 SHELL_STRAIGHT_BEFORE、SHELL_STRAIGHT_AFTER，不在 if、for、while、until、select、括号与花括号组里，
    不是 heredoc 的正文，那一行切得开；切法与分隔里的记号在 shell_text_pieces）：
    - cd、pushd 带一个目录（选项 -P、-L、-e、-@、-- 跳过），目录代入之后算得出、此刻是一个在的目录的，之后的当前目录是它；带不了、算不出的之后算不出。popd 之后算不出。
      整段里有一处 cd、pushd、popd 不是平铺着跑的这种写法（层里、花括号组里、子 shell 里、heredoc 正文里、&& 之后、当参数），整段的当前目录都算不出：
      循环里的 cd 让循环体前面的命令后续迭代也换了目录，接在管道或 & 上的花括号组跑在子 shell 里，heredoc 正文喂给谁这里认不出。
    - 只是赋值的一条（`名=值`、`export 名=值` 这类，shell_assignments）照先后记下值（值先代入前面记下的变量）。一个名字在整段里不当引用出现（SHELL_NAME_MENTION：
      赋值、for 与 read 的变量、unset 与当参数都算）不止一次的，记成算不出：改它的写法认不全，按名字整段不信。
    - 一条命令起命令词之前带 env 的 -C / --chdir（shell_directory_option 看 command_word_index 之前的词）：它自己起时的当前目录算不出，之后的照前面的算（换的只是它起的那个进程的目录）。
    整段有函数定义（SHELL_FUNCTION_DEFINITION）、source、.、eval、case（SHELL_OPAQUE_WORDS，命令词照 shell_command_head 取），或层数数不平（末尾的 ) 与 } 后面没有命令的同样数不平），
    当前目录与变量一概算不出。单引号里的 $名字 照变量代入（切词时引号已去掉，分不出），这一形判不到。
    弄坏开关 brace-group-not-layered、heredoc-body-as-commands 见 shell_text_pieces；string-env-chdir-unread 下带 env -C 的那一条照前面算出的当前目录交。"""
    pieces = shell_text_pieces(text, environment)
    heads = [shell_command_head(words) for words, _ in pieces]
    straight, depth = [], 0
    for index, (words, joint) in enumerate(pieces):
        depth += sum(token.count("(") + token.count("{") - token.count(")") - token.count("}") for token in joint)
        if words[0] in SHELL_BLOCK_OPENERS:
            depth += 1
        following = pieces[index + 1][1] if index + 1 < len(pieces) else ()
        straight.append(depth == 0 and all(token in SHELL_STRAIGHT_BEFORE for token in joint)
                        and (not following or following[0] in SHELL_STRAIGHT_AFTER))
        if words[0] in SHELL_BLOCK_CLOSERS:
            depth -= 1
        opaque = opaque or depth < 0
    opaque = (opaque or depth != 0 or bool(SHELL_FUNCTION_DEFINITION.search(text)) or any(head in SHELL_OPAQUE_WORDS for head in heads))
    mentions = {}
    for words, _ in pieces:
        for word in words:
            for name in SHELL_NAME_MENTION.findall(word):
                mentions[name] = mentions.get(name, 0) + 1
    directory_known = not opaque and all(
        not any(word in SHELL_DIRECTORY_WORDS for word in words)
        or (straight[index] and words[0] in SHELL_DIRECTORY_WORDS and sum(word in SHELL_DIRECTORY_WORDS for word in words) == 1)
        for index, (words, _) in enumerate(pieces))
    current = os.path.normpath(directory) if directory is not None and directory_known else None
    chdir_read = not lib.broken("string-env-chdir-unread", environment)
    variables, runs = {}, []
    for index, (words, _) in enumerate(pieces):
        expanded = [shell_expanded_word(word, variables) for word in words]
        moved_itself = chdir_read and shell_directory_option(words[:lib.command_word_index(words, environment)])
        runs.append(ShellRun(list(words), expanded, None if moved_itself else current))
        if opaque or not straight[index]:
            continue
        if directory_known and words[0] in SHELL_DIRECTORY_WORDS:
            targets = [value for word, value in zip(words[1:], expanded[1:]) if word not in ("-P", "-L", "-e", "-@", "--")]
            target = targets[0] if words[0] != "popd" and len(targets) == 1 and targets[0] not in (None, "-") else None
            moved = None if target is None or (current is None and not os.path.isabs(target)) else os.path.normpath(os.path.join(current or "", target))
            current = moved if moved is not None and os.path.isdir(moved) else None
        for name, value in shell_assignments(words):
            variables[name] = shell_expanded_word(value, variables) if mentions.get(name) == 1 else None
    return runs


def script_body_text(path):
    """命令位置上起的脚本的正文（script_body_workspace_commands）：超过 WRAPPED_WRITER_SCRIPT_MAXIMUM_BYTES、读出错、带 NUL 字节（编出来的二进制）的交 None。"""
    try:
        if os.path.getsize(path) > WRAPPED_WRITER_SCRIPT_MAXIMUM_BYTES:
            return None
        with open(path, encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError:
        return None
    return None if "\0" in text else text


class SchedulerBuilding:
    """Scheduler 的一部分方法：编译目录池、取消后删产物、现场目录、收尾与分片、排队次序、派活、提交时拒的各种判据（Scheduler 在 task_server.py，经继承取到它们）。"""

    def held_build_slots(self):
        """此刻占着的组里那几个（build_slots 的行号）：在起与在跑的尝试记着的 build_slot，连同拿到了、尝试还没插进库的（build_slot_reservations）与扫树正在删的
        （build_slots_removing）。调用方拿着 build_pool_lock。弄坏开关 build-slot-held-shared 下只认正在删的（占着的照空着的给出去）。"""
        removing = set(self.build_slots_removing)
        if lib.broken("build-slot-held-shared", self.environment):
            return removing
        running = {attempt.get("build_slot") for attempt in self.store.running_attempts() if attempt.get("build_slot")}
        return running | set(self.build_slot_reservations) | removing

    def tree_build_slots(self, tree, host):
        """这棵树在这台的组：→ 校验过的那几行（核不过的不认、不拿、不删）。弄坏开关 build-slot-tree-ignored 下这台上各棵树的都算（不分树）。"""
        if lib.broken("build-slot-tree-ignored", self.environment):
            rows = self.store.select("build_slots", "state = 'present'")
        else:
            rows = self.store.select("build_slots", "command_key = ? AND state = 'present'", (tree,))
        return [row for row, valid in rows if valid and row.get("host") == host]

    def build_pool_problem(self, task, host):
        """服务给编译目录的这一条在这台上拿不拿得到组里一个（文件头 ②「编译目录组」）：组里有空着的、或还没满 BUILD_POOL_SLOTS_PER_TREE 个交空串，
        都占着又满了交排队原因（点名组满）。弄坏开关 build-slot-per-attempt 下不看组。"""
        if not self.server_gives_build_directory(task) or lib.broken("build-slot-per-attempt", self.environment):
            return ""
        tree = lib.build_tree_root(task["cwd"])
        with self.build_pool_lock:
            slots = self.tree_build_slots(tree, host)
            held = self.held_build_slots()
        if len(slots) < BUILD_POOL_SLOTS_PER_TREE or any(row["id"] not in held for row in slots):
            return ""
        return (f"{BUILD_POOL_FULL_PREFIX}：这棵树（{tree}）在{lib.HOST_NAMES.get(host, host)}的 {len(slots)} 个编译目录都占着（一棵树一台至多 {BUILD_POOL_SLOTS_PER_TREE} 个），"
                "等其中一条跑完放回")

    def previous_build_slot(self, task, number, host):
        """这条任务上一次尝试（第 number − 1 次）在这台用的组里那一个的行号；没有交 None（撞上限重排接着编那一个）。"""
        if number <= 1:
            return None
        for attempt, valid in self.store.attempts_of(task["id"]):
            if valid and int(attempt.get("number") or 0) == number - 1 and (attempt.get("host") or LOCAL) == host:
                return attempt.get("build_slot")
        return None

    def take_build_slot(self, task, host, label, number):
        """这一次尝试拿这棵树在这台的组里一个编译目录（文件头「编译目录」）：→ (组里那一行, 是不是组里已有的)；组满、都占着交 (None, False)。
        树是任务 --cwd 解开符号链接后所在的仓根（lib.build_tree_root）。拿法：这条任务上一次尝试在同一台用的那一个空着就拿它（撞上限重排接着编），
        不然拿空着的里最近用过的那一个，都没有而组没满就新开一个（Machine.build_directory 起名：本机的不带标签与任务号，第二台的照这一次的标签；第二台那一个要等 multi-host-run.sh 报了留着才算可接着用）。
        空着而已不在的（本机目录没了、第二台那一个没报过留着）当场从组里删掉、不拿。拿到的记进 build_slot_reservations（占着），尝试插进库或没起成之后
        release_build_slot_reservation 放掉。弄坏开关 build-slot-per-attempt 下一律新开、不看组满。"""
        machine = self.machines[host]
        tree = lib.build_tree_root(task["cwd"])
        now = round(time.time(), 3)
        with self.build_pool_lock:
            chosen = None
            if not lib.broken("build-slot-per-attempt", self.environment):
                held = self.held_build_slots()
                slots = self.tree_build_slots(tree, host)
                free = []
                for row in slots:
                    if row["id"] in held:
                        continue
                    if row.get("ready") and machine.build_directory_exists(row.get("directory") or ""):
                        free.append(row)
                    else:
                        self.store.delete_row("build_slots", row["id"])   # 空着而已不在的：从组里删掉
                previous = self.previous_build_slot(task, number, host)
                chosen = next((row for row in free if row["id"] == previous), None) or max(free, key=lambda row: row.get("used_at") or 0, default=None)
                if chosen is None and len([row for row in slots if row["id"] in held]) >= BUILD_POOL_SLOTS_PER_TREE:
                    return None, False
            if chosen is None:
                directory, _ = machine.build_directory(label, tree)
                if not directory:
                    return None, False
                chosen = self.store.insert("build_slots", {"task_id": 0, "state": "present", "session": "", "command_key": tree, "tree": tree, "host": host,
                                                           "directory": directory, "opened_by": label, "opened_at": now, "used_at": now, "ready": machine.name == LOCAL})
                reused = False
            else:
                chosen = self.store.change("build_slots", chosen["id"], {"used_at": now})
                reused = True
            self.build_slot_reservations.add(chosen["id"])
        return chosen, reused

    def release_build_slot_reservation(self, slot_id):
        """拿到之后的那一段过去了（尝试插进了库、在跑尝试记着它，或没起成）：放掉 build_slot_reservations 里这一行。"""
        with self.build_pool_lock:
            self.build_slot_reservations.discard(slot_id)

    def settle_build_slot(self, slot_id, kept, measured_bytes):
        """一次尝试收尾（finish_attempt）：留着的（本机一律算留着；第二台是 multi-host-run.sh 报了留着的）记成可接着用，量到了占盘（measured_bytes，字节）的照它记下
        （build_disk_problem 照它扣）；第二台没报留着的（⑥ 连它一起删了）从组里删掉这一行。"""
        with self.build_pool_lock:
            if not kept:
                self.store.delete_row("build_slots", slot_id)
                return
            changes = {"ready": True, "used_at": round(time.time(), 3)}
            if isinstance(measured_bytes, int) and not isinstance(measured_bytes, bool) and measured_bytes >= 0:
                changes["bytes"] = measured_bytes
            with contextlib.suppress(StoreRefused):
                self.store.change("build_slots", slot_id, changes)

    def free_build_slot_bytes(self, task, host):
        """这一条在这台上会接着用的组里那一个已占的盘（字节；build_disk_problem 从占盘预测里扣掉它，接着用的那一个已占的盘在余量里扣过了）：
        这棵树在这台的组里空着、可接着用的几个里记下的占盘取最大；不给编译目录的、组里没有空着的交 0。
        弄坏开关 build-disk-reuse-uncredited 下一律交 0（接着用的照从零编算），build-slot-per-attempt 下同样交 0（一律新开）。"""
        if (not self.server_gives_build_directory(task) or lib.broken("build-disk-reuse-uncredited", self.environment)
                or lib.broken("build-slot-per-attempt", self.environment)):
            return 0
        machine = self.machines.get(host) or self.machines[LOCAL]
        tree = lib.build_tree_root(task["cwd"])
        with self.build_pool_lock:
            held = self.held_build_slots()
            free = [row for row in self.tree_build_slots(tree, host)
                    if row["id"] not in held and row.get("ready") and machine.build_directory_exists(row.get("directory") or "")]
        return max((int(row.get("bytes") or 0) for row in free), default=0)

    def start_build_pool_sweep(self, periodic):
        """在后台线程里起一回扫树（sweep_vanished_tree_pools；删第二台那一个要连 ssh，不挡排队）：periodic 的离上一回不到 BUILD_POOL_SWEEP_SECONDS 不起，
        上一回还没完的不起。服务起来时起一回（periodic=False），排队线程每一轮问一次。"""
        now = time.monotonic()
        if periodic and self.build_pool_swept_at and now - self.build_pool_swept_at < BUILD_POOL_SWEEP_SECONDS:
            return
        if not self.build_pool_sweep_lock.acquire(blocking=False):
            return
        self.build_pool_swept_at = now
        threading.Thread(target=self.sweep_vanished_tree_pools, name="task-server-build-pool-sweep", daemon=True).start()

    def sweep_vanished_tree_pools(self):
        """树不在了的整组删（文件头「编译目录」；用户：树删了编译目录跟着删）：组里每一行的树此刻不是目录了、那一行空着的，删它的编译目录
        （remove_vanished_tree_slot）；占着的等放回之后下一回再扫。扫完组接着扫本机编译根下不在任何组里、照尝试标签起名的目录（sweep_unpooled_attempt_label_directories）。
        拿着 build_pool_sweep_lock 起（start_build_pool_sweep），扫完放掉。弄坏开关 vanished-tree-pool-kept 下组里一个都不删（编译根那一扫照做，它有自己的开关）。"""
        try:
            doomed = []
            if not lib.broken("vanished-tree-pool-kept", self.environment):
                with self.build_pool_lock:
                    held = self.held_build_slots()
                    doomed = [row for row, valid in self.store.select("build_slots", "state = 'present'")
                              if valid and row["id"] not in held and not os.path.isdir(row.get("tree") or "")]
                    self.build_slots_removing.update(row["id"] for row in doomed)
            for row in doomed:
                try:
                    self.remove_vanished_tree_slot(row)
                finally:
                    with self.build_pool_lock:
                        self.build_slots_removing.discard(row["id"])
            self.sweep_unpooled_attempt_label_directories()
        except Exception:   # 一回扫出事不停服务：打出来、下一回再扫
            say("  ! 扫树删编译目录组出了异常（下一回再扫）：\n" + traceback.format_exc())
        finally:
            self.build_pool_sweep_lock.release()

    def remove_vanished_tree_slot(self, row):
        """树不在了的组里那一个：本机就地删、第二台经 multi-host-run.sh --remove-build-directory（Machine.remove_build_directory，有进程开着里面的文件就不删），
        删了、回读不在了才从组里删掉那一行；没删成的写告警 build-directory-left，那一行留着、下一回再扫。本机目录已不在、第二台那一个没报过留着的只删那一行。"""
        host = row.get("host") or LOCAL
        machine = self.machines.get(host) or self.machines[LOCAL]
        directory = row.get("directory") or ""
        where = lib.HOST_NAMES.get(host, host)
        if row.get("ready") and directory and machine.build_directory_exists(directory):
            removed, lines = machine.remove_build_directory(directory)
        else:
            removed, lines = True, []
        if not removed:
            self.alert("build-directory-left", 0, f"树 {row.get('tree')} 不在了，它在{where}的编译目录 {directory} 没删成：" + "；".join(line.strip() for line in lines),
                       machine.removal_howto(directory))
            return
        self.store.delete_row("build_slots", row["id"])
        say(f"  · 树 {row.get('tree')} 不在了：删了它在{where}的编译目录 {directory or '（没记目录）'}、组里那一行")

    def sweep_unpooled_attempt_label_directories(self):
        """本机编译根下不在任何组里、名字是尝试标签那一种（task-<这一个库的标签>-<任务号>-<第几次>）的目录（task_server_lib.py 文件头「编译目录」的「删」）：
        本机组里新开的一律照 build_directory_name 起 tree-… 的名字，这种名字的目录不在组里就没有哪一次尝试接着用它。组里任何一行（核不过的也算）与在起、在跑的尝试记着的
        编译目录不动，别的名字、别的库标签的、符号链接不动。删法同 remove_vanished_tree_slot 的本机那一支（LocalMachine.remove_build_directory：有进程开着里面的文件就不删），
        删了、回读不在了才算；没删成的写告警 build-directory-left、下一回再扫。sweep_vanished_tree_pools 扫完组接着调它，同在 build_pool_sweep_lock 底下。
        弄坏开关 unpooled-label-directories-kept 下一个都不删，unpooled-label-directory-open-unchecked 下删之前不核开着的文件，unpooled-label-directories-pool-unread 下组里记着的也删，
        unpooled-label-directories-running-unread 下在起、在跑的尝试记着的（组里没有那一行的）也删。"""
        if lib.broken("unpooled-label-directories-kept", self.environment):
            return
        root = self.configuration.build_directory
        try:
            names = sorted(os.listdir(root))
        except OSError:   # 编译根还没建（没起过会编的任务）：没有要扫的
            return
        label_name = re.compile(rf"task-{re.escape(self.store.tag)}-\d+-\d+")
        running_unread = lib.broken("unpooled-label-directories-running-unread", self.environment)
        with self.build_pool_lock:
            pooled = set() if lib.broken("unpooled-label-directories-pool-unread", self.environment) else (
                {os.path.realpath(row["directory"]) for row, _ in self.store.select("build_slots") if row.get("directory")}
                | ({os.path.realpath(attempt["build_directory"]) for attempt in self.store.running_attempts() if attempt.get("build_directory")}
                   if not running_unread else set()))
        machine = self.machines[LOCAL]
        removed_count, left_count = 0, 0
        for name in names:
            directory = os.path.join(root, name)
            if not label_name.fullmatch(name) or os.path.islink(directory) or not os.path.isdir(directory) or os.path.realpath(directory) in pooled:
                continue
            if lib.broken("unpooled-label-directory-open-unchecked", self.environment):
                removed, lines = self.build_directories.remove_run_build_directory(directory, check_open=False)
                removed = removed and not os.path.lexists(directory)
            else:
                removed, lines = machine.remove_build_directory(directory)
            if not removed:
                left_count += 1
                self.alert("build-directory-left", 0, f"本机编译根下不在任何组里的 {directory}（照尝试标签起的名字）没删成：" + "；".join(line.strip() for line in lines),
                           machine.removal_howto(directory))
                continue
            removed_count += 1
            say(f"  · 本机编译根下不在任何组里的 {directory}（照尝试标签起的名字）删了、回读不在了")
        if removed_count or left_count:
            say(f"  · {UNPOOLED_SWEEP_SUMMARY_MARK}：删了 {removed_count} 个、没删成 {left_count} 个（没删成的写了告警 build-directory-left，下一回再扫）")

    def cancel_artifact_sizes(self, task_id):
        """取消一开始记（文件头「接口」的取消）：这条任务（分片的连同子任务）此刻在起与在跑的尝试是哪几次。
        → {"active": {尝试记录的 id…}}；名下有校验不过的子任务行交空的（取消那一侧照 record-tampered 办，一份都不删）。"""
        before = {"active": set()}
        try:
            owners = [task_id] + [child["id"] for child in self.children_of(task_id)]
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            return before
        for owner in owners:
            for attempt, valid in self.store.attempts_of(owner):
                if valid and attempt.get("state") in ACTIVE_ATTEMPT_STATES:
                    before["active"].add(attempt.get("id"))
        return before

    def remove_cancelled_artifacts(self, task_id, before):
        """取消收尾之后删产物（文件头「接口」的取消；用户：「取消的任务要自己删掉产物 不要残留」「这个加入流程中 让task server去做」）：这条任务（分片的连同子任务）
        每一次尝试的现场目录（设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的也删），派到第二台那一趟的本地目录照日志里 multi-host-run.sh ⑥ 那一行认（cancelled_attempt_artifacts）。
        本机组里的编译目录不删：它归这棵树的组，尝试停了就放回组里；派到第二台、中途被停的那一趟，⑥ 连它用的组里那一个一起删，收尾时组里删掉那一行（task_server_lib.py 文件头「编译目录」）。
        before 是 cancel_artifact_sizes 在取消一开始记的。
        逐份写进每条任务最后一次尝试的日志；没删成的写告警 build-directory-left。
        → {"removed": [份…], "left": [份…], "lines": [给回话与日志的行…]}。弄坏开关 cancelled-artifacts-kept 下不删、交空的。"""
        if lib.broken("cancelled-artifacts-kept", self.environment):
            return {"removed": [], "left": [], "lines": []}
        try:
            owners = [task_id] + [child["id"] for child in self.children_of(task_id)]
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            return {"removed": [], "left": [], "lines": [f"  ✗ {self.children_refused(task_id, refused, '取消之后一份产物都不删')}"]}
        removed, left, every_line, handled = [], [], [], set()
        for owner in owners:
            attempts = [attempt for attempt, valid in self.store.attempts_of(owner) if valid]
            lines = []
            for attempt in attempts:
                for item in self.cancelled_attempt_artifacts(attempt, before, handled):
                    (removed if item["gone"] else left).append(item)
                    lines += cancelled_artifact_lines(item)
                    if not item["gone"]:
                        self.alert("build-directory-left", owner, f"任务 {owner} 取消之后{item['path']} 没删掉：{item['why']}", item["howto"])
            if attempts and lines:
                append_log_lines(attempts[-1].get("log", ""), lines + [f"  · 取消之后删产物：这条任务删了 {sum(1 for line in lines if line.startswith('  · '))} 份、"
                                                                       f"没删成 {sum(1 for line in lines if line.startswith('  ✗ '))} 份"])
            every_line += lines
        every_line.append(f"  · 取消之后删产物：删了 {len(removed)} 份、没删成 {len(left)} 份" if removed or left
                          else "  · 取消之后删产物：各次尝试都没留下现场目录与第二台这一趟的本地目录，没有要删的（编译目录归这棵树的组，放回、不删）")
        return {"removed": removed, "left": left, "lines": every_line}

    def cancelled_attempt_artifacts(self, attempt, before, handled):
        """一次尝试取消之后的产物（remove_cancelled_artifacts 用）：→ [{task, attempt, host, kind, path, bytes_before, bytes_after, gone, how, why, howto}]。
        现场目录（共享工作根下 SCENES_ROOT_NAME/<尝试标签>，两台同一个绝对路径）：在的量了大小就地删、回读；
        取消时在起、在跑的第二台那一次：日志里有 PEER_TRIP_REMOVED_MARK 那一行（「回读不在了」）算 multi-host-run.sh 删了这一趟的本地目录与 runs/ 下三个文件；
        没有那一行的先读这一次的清理回执（peer_cleanup_file）：还是 not-started、日志里也没有 PEER_TRIP_BUILT_MARKS 那两行的，multi-host-run.sh 还没建这一趟的本地目录，
        当已清理、不列（与 confirm_peer_cleanup 认 not-started 同一口径）；回执到过 pending（pending、confirmed）、读不了、没有回执，或建过目录的，算没删成。
        这一趟的本地目录是组里那一个的（接着用组里已有的那一次）就是那一个：⑥ 没等到退出码连它一起删，收尾时组里删掉那一行（settle_build_slot）。
        弄坏开关 cancel-not-started-receipt-unread 下不读回执，没有那一行一律算没删成。handled 记删过的现场目录，只报一次。"""
        items = []
        host = attempt.get("host") or LOCAL
        base = {"task": attempt.get("task_id"), "attempt": attempt.get("number"), "host": host}
        scene = attempt.get("scene_directory")
        if scene and os.path.lexists(scene) and ("scene", scene) not in handled:   # 现场目录在共享工作根下，两台同一个绝对路径，就地删（task_server_lib.py 文件头「现场」）
            handled.add(("scene", scene))
            size = directory_bytes(scene)
            shutil.rmtree(scene, ignore_errors=True)
            gone = not os.path.lexists(scene)
            items.append(dict(base, kind="现场目录", path=scene, bytes_before=size, bytes_after=0 if gone else directory_bytes(scene), gone=gone,
                              how="这一次取消删的、回读不在了" if gone else "", why="" if gone else "rm 之后还在（权限不对或有进程在往里写）",
                              howto=f"停掉往里写的进程，再 rm -r {shlex.quote(scene)}"))
        if host == PEER and attempt.get("id") in before["active"]:
            label = attempt.get("label") or "?"
            local_name = os.path.basename(attempt.get("build_directory") or "") or label
            trip = f"第二台这一趟的本地目录 builds/{local_name} 与 runs/ 下 {label}.pid、.exit、.log"
            log_lines = log_tail(attempt.get("log", "")).splitlines()
            reported = any(PEER_TRIP_REMOVED_MARK in line and PEER_TRIP_READ_BACK_MARK in line for line in log_lines)
            receipt = read_json_file(attempt["peer_cleanup_file"]) if attempt.get("peer_cleanup_file") else None
            receipt_status = receipt.get("status") if isinstance(receipt, dict) else None
            built = any(mark in line for line in log_lines for mark in PEER_TRIP_BUILT_MARKS)
            if not reported and receipt_status == "not-started" and not built and not lib.broken("cancel-not-started-receipt-unread", self.environment):
                return items   # 第二台这一趟的本地目录还没建（同 confirm_peer_cleanup 认 not-started）：没有要删的，不列
            receipt_text = ((f"清理回执 {receipt_status}" if receipt_status else "清理回执读不到" if attempt.get("peer_cleanup_file") else "这一次没有清理回执")
                            + ("、日志里有建了这一趟的本地目录之后才打的那一行" if built else ""))
            items.append(dict(base, kind="本地目录", path=trip, bytes_before=attempt.get("build_peak"), bytes_after=None if not reported else 0, gone=reported,
                              how="multi-host-run.sh 的 ⑥ 删了、回读不在了" if reported else "",
                              why="" if reported else (f"这一次尝试的日志里没有 multi-host-run.sh 打的「{PEER_TRIP_REMOVED_MARK} …{PEER_TRIP_READ_BACK_MARK}」那一行"
                                                       f"（{receipt_text}），第二台那一份删没删判不了"),
                              howto=(f"到第二台多机配置 BUILD_ROOT_BY_HOST 里它那一项的目录下核 builds/{local_name} 与 runs/{label}.pid / .exit / .log，"
                                     "在的话先停开着它们的进程（python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>）再删")))
        return items

    def remove_scenes(self, task_id):
        """删这条任务（分片的连同子任务）各次尝试的现场目录（task_server_lib.py 文件头「现场」）：→ (删了的 [{path, bytes}], 没删成的 [{path, bytes, kind, why}])。"""
        removed, left = [], []
        try:
            owners = [task_id] + [child["id"] for child in self.children_of(task_id)]
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            return [], [{"path": self.scenes_root, "bytes": 0, "kind": "现场", "why": self.children_refused(task_id, refused, "一份现场都不删")}]
        for owner in owners:
            for attempt, _ in self.store.attempts_of(owner):
                scene = attempt.get("scene_directory")
                if not scene or not os.path.lexists(scene):
                    continue
                size = directory_bytes(scene)
                shutil.rmtree(scene, ignore_errors=True)
                if os.path.lexists(scene):
                    left.append({"path": scene, "bytes": size, "kind": "现场", "why": "rm 之后还在（权限不对或有进程在往里写）"})
                else:
                    removed.append({"path": scene, "bytes": size})
        return removed, left

    def scene_due(self, task_id, now):
        """这条任务的现场到没到该删的时候（task_server_lib.py 文件头「现场」）：库里没有了的到了；校验不过的不动；分片的子任务照父任务；
        要提交方 task wait 取走结局满 scene_keep_seconds，没有会话的（人手 --manual，不一定起 wait）按结局时刻算；提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的只由 task clean 删（弄坏开关 scene-keep-ignored 下照删）。
        弄坏开关 scene-sessionless-kept 下没有会话的也要等取走。"""
        found = self.store.task(task_id)
        if found is not None and found[1] and found[0].get("parent_id"):
            found = self.store.task(found[0]["parent_id"])
        if found is None:
            return True
        task, valid = found
        if not valid or task.get("state") != "finished":
            return False
        if not task.get("collected") and (task.get("session") or lib.broken("scene-sessionless-kept", self.environment)):
            return False
        if (task.get("environment") or {}).get("SINGLEFS_KEEP_TEST_TEMPORARY_FILES") == "1" and not lib.broken("scene-keep-ignored", self.environment):
            return False
        return now - float(task.get("collected_at") or task.get("finished_at") or now) >= self.scene_keep_seconds

    def sweep_scenes(self):
        """排队线程每一轮看一遍现场根（task_server_lib.py 文件头「现场」）：这一个库的标签的那几个目录（task-<库标签>-<任务号>-<第几次>）到了该删的就删；
        别的标签的不动。弄坏开关 scene-kept-after-collect 下一个都不删。"""
        if lib.broken("scene-kept-after-collect", self.environment):
            return
        try:
            names = os.listdir(self.scenes_root)
        except OSError:
            return
        prefix, now = f"task-{self.store.tag}-", time.time()
        for name in names:
            task_text = name[len(prefix):].rsplit("-", 1)[0] if name.startswith(prefix) else ""
            if task_text.isdigit() and self.scene_due(int(task_text), now):
                shutil.rmtree(os.path.join(self.scenes_root, name), ignore_errors=True)

    def finalize(self, task, outcome, attempt_exit, note="", overwrite=False, push_parent=True, **fields):
        """记下这条任务的结局（一个事务里读—判—写，只并进结局那几栏与 fields；已结了局的不再改，弄坏开关 finished-cancel-overwrites 那一支给 overwrite）。
        → 记下了没有。子任务记下之后驱动它的父任务（drive_parent）；正在推父任务的那一步自己撤兄弟片时给 push_parent=False，不再套一层推。"""
        changes = dict(fields, state="finished", outcome=outcome, exit_code=lib.final_exit_code(outcome, attempt_exit), finished_at=round(time.time(), 3), note=note)
        try:
            record = self.store.change("tasks", task["id"], changes,
                                       guard=None if overwrite else (lambda current: "已结了局" if current.get("state") == "finished" else ""))
        except StoreRefused as refused:
            if refused.kind == "tampered":
                self.tampered(task["id"], task, "任务记录")
            elif refused.kind != "guard":
                raise
            return False
        task.clear()
        task.update(record)
        self.forget_prediction(task["id"])   # 收尾就删掉存的那一份预测（文件头 ②「预测每条只算一次」）
        # 各次尝试用的本机组里的编译目录不删：归这棵树的组，尝试停了就放回；第二台没等到退出码的那一趟已由 ⑥ 删了、收尾时从组里去掉（task_server_lib.py 文件头「编译目录」）
        self.notify_finished(task["id"])
        if lib.broken("scene-removed-at-finish", self.environment):   # 弄坏开关：现场照「有结局就删」那一刻删（task_server_lib.py 文件头「现场」）
            self.remove_scenes(task["id"])
        say(f"  · 任务 {task['id']} 结局 {outcome}（尝试 {task.get('attempt_count', 0)} 次{'；' + note if note else ''}）")
        if task.get("parent_id") and push_parent:
            self.crash_point(f"child-finished-{task.get('role')}")
            self.advance_parent(task)
        return True

    def advance_parent(self, child):
        """子任务记下结局之后推它的父任务：照库里父任务的阶段与子任务集合现推（drive_parent）；这一步没走到的（服务没了、抛错）由每一轮对账补推。"""
        self.drive_parent(child["parent_id"])

    # ── ④ 分片（文件头「分片」与 task_server_lib.py 文件头「分片」）──
    def children_of(self, parent_id):
        """父任务名下的子任务记录（照关联列 task_id 挑）。有一行校验不过（关联列与正文对不上、写进之后被改过，TS-05）抛 StoreRefused("tampered")，
        说明里点名那几行：调用方照 record-tampered 办（不认、不动、写告警、回 409），不当它没事。弄坏开关 child-rows-unverified 下跳过校验位、整份交回。"""
        rows = self.store.select("tasks", "task_id = ?", (parent_id,))
        unverified = [record.get("id") for record, valid in rows if not valid]
        if unverified and not lib.broken("child-rows-unverified", self.environment):
            raise StoreRefused("tampered", f"任务 {parent_id} 名下有校验不过的子任务记录（行 {unverified}，关联列与正文对不上或写进之后被改过）")
        return [record for record, _ in rows]

    def children_refused(self, parent_id, refused, consequence):
        """children_of 判出名下有校验不过的子任务行（refused）：照 record-tampered 的处理写告警（tampered_cancel_refusal 的写法），交回话里那一句
        「<哪几行校验不过>：<consequence>」；调用方不认那几行、不往下做，回 409。"""
        detail = f"{refused}：{consequence}"
        if (parent_id, detail) not in self.alerted_child_refusals:   # 同一件同一句只告警一次（status 每读一次都会走到这里）
            self.alerted_child_refusals.add((parent_id, detail))
            self.alert("record-tampered", parent_id, detail, "查是谁直接写了库（只有 task-server 读写它）；那几行对应的单元与目录要动由人照记录核过再动")
        return detail

    def shard_directory(self, parent_id):
        return os.path.join(lib.shards_root(), f"{self.store.tag}-{parent_id}")

    def shard_binaries_removal(self, parent_id, session, process_id):
        """删分片父任务计划目录下的 lib.SHARD_PLAN_BINARIES_NAME（计划那一步预先编好、各片与合并共用的测试二进制；文件头 ④）：父任务有了结局（finalize_parent）与 task clean 时调。
        → (删了的 [{path, bytes}], 没删的 [{path, bytes, kind, why}])；那个子目录不在的交 ([], [])。只删这一个子目录：照真路径要恰好是分片根下 <库标签>-<号>/plan 里的那一份，
        是符号链接或上面哪一层指到别处、真路径对不上的不删，服务日志打一行。删了的写份数（文件个数）与字节数、不删的写原因，各记一行操作日志（action shard-binaries）。
        弄坏开关 shard-binaries-kept 下一律不删，shard-binaries-path-unchecked 下不核路径、照真路径删。"""
        target = os.path.join(self.shard_directory(parent_id), "plan", lib.SHARD_PLAN_BINARIES_NAME)
        if lib.broken("shard-binaries-kept", self.environment) or not os.path.lexists(target):
            return [], []
        resolved = os.path.realpath(target)
        own = os.path.join(os.path.realpath(lib.shards_root()), f"{self.store.tag}-{parent_id}", "plan", lib.SHARD_PLAN_BINARIES_NAME)
        if resolved != own and not lib.broken("shard-binaries-path-unchecked", self.environment):
            why = f"真路径 {resolved} 不是这一条任务自己的计划目录下的那一份（{own}），不删；核过它是什么之后人手删 {target}"
            say(f"  ! 分片的任务 {parent_id} 计划目录下的 {lib.SHARD_PLAN_BINARIES_NAME}：{why}")
            self.operation("shard-binaries", parent_id, session, process_id, f"没删 {target}：{why}")
            return [], [{"path": target, "bytes": 0, "kind": "测试二进制", "why": why}]
        files = sum(len(names) for _, _, names in os.walk(resolved))
        size = directory_bytes(resolved)
        shutil.rmtree(resolved, ignore_errors=True)
        if os.path.lexists(resolved):
            why = "rm 之后还在（权限不对或有进程开着里面的文件）"
            self.operation("shard-binaries", parent_id, session, process_id, f"没删成 {target}（{files} 份、{size} 字节）：{why}")
            return [], [{"path": target, "bytes": size, "kind": "测试二进制", "why": why}]
        self.operation("shard-binaries", parent_id, session, process_id, f"删了 {target}：{files} 份、{size} 字节")
        say(f"  · 分片的任务 {parent_id} 删了计划目录下的 {lib.SHARD_PLAN_BINARIES_NAME}（{files} 份、{size} 字节）")
        return [{"path": target, "bytes": size}], []

    def find_child(self, children, role, index=0):
        return next((child for child in children if child.get("role") == role and int(child.get("shard_index") or 0) == index), None)

    @staticmethod
    def child_compiles(parent, role):
        """这一步子任务服务给不给编译目录（create_child 记进子任务的 compiles；计划那一步的键照它带编不编，lib.shard_child_key）：
        计划、各片、合并跑的是父命令，父任务会编的照给。"""
        return bool(parent.get("compiles")) and role in lib.SHARD_ROLES

    def child_key_text(self, parent, role, index, count):
        """这一步子任务的命令键原文（create_child 建子任务照它）：父任务点了 lib.KEY_VARIABLE 的取它，不然父命令照词折叠，再接这一步那一段
        （计划那一步的那一段带这一步编不编，child_compiles；计划那一步的父命令另剥缩域变量 lib.PLAN_KEY_NEUTRAL_VARIABLES，各片的不剥）。"""
        compiles = self.child_compiles(parent, role)
        base = parent["environment"].get(lib.KEY_VARIABLE)
        if not base and lib.broken("shard-key-from-joined-command", self.environment):
            base = " ".join(parent["command"])   # 弄坏开关：用空格接父命令的词，条目名里带空格的后半截漏进键里
        elif not base:
            # 父命令照词折叠（memory_peaks_lib 的 command_key 收词表：带空格的条目名、路径仍是一个词），再接这一步那一段；
            # 截短照这一段的长度先截父任务那一截，接的那一段不被键长上限截掉（截掉了各片落一个键）
            marker = len(lib.shard_child_key("", role, index, count, self.environment, compiles=compiles))
            words = lib.key_words(parent["command"], self.environment, plan_step=role == "plan")   # 计划那一步另剥缩域变量（lib.PLAN_KEY_NEUTRAL_VARIABLES）：缩域深度不同的父命令落同一把计划键
            base = self.peaks.command_key(lib.selection_class_words(lib.cargo_test_class_words(words, self.environment), self.environment))[:max(0, self.peaks.KEY_LENGTH_LIMIT - marker)]
        return lib.shard_child_key(base, role, index, count, self.environment, compiles=compiles)

    def create_child(self, parent, role, index=0, count=0, label=""):
        """建这一步的子任务；同一父任务同一步同一片已有的交已有的那一个（库里 (parent_id, role, shard_index) 唯一，TS-02）。计划、各片与合并都不钉台，哪台由服务挑。
        父任务 task promote 提前过的，子任务带上同一份 promoted（弄坏开关 promote-shards-refused 下不带）。"""
        existing = self.find_child(self.children_of(parent["id"]), role, index)
        if existing is not None:
            return existing
        command = parent["command"]
        key_text = self.child_key_text(parent, role, index, count)
        environment = dict(parent["environment"], **{lib.KEY_VARIABLE: key_text})
        child = {"command": command, "cwd": parent["cwd"], "environment": environment, "session": parent.get("session", ""),
                 "submitter_pid": parent.get("submitter_pid", 0), "submitted_at": round(time.time(), 3),
                 "command_key": self.command_key(command, key_text), "state": "queued", "outcome": "", "exit_code": None,
                 "collected": True, "next_cap": None, "next_host": "", "outcome_counts": {}, "note": "", "cancel_requested": False, "attempt_count": 0,
                 "task_id": parent["id"], "parent_id": parent["id"], "role": role, "shard_index": index, "shard_count": count,
                 "label": label, "compiles": self.child_compiles(parent, role)}   # 计划、各片、合并跑的是父命令：父任务会编的照给编译目录
        if parent.get("promoted") and not lib.broken("promote-shards-refused", self.environment):
            child["promoted"] = dict(parent["promoted"])   # task promote 提前过的父任务：计划、各片、合并照样排最前（promote）
        try:
            child = self.store.insert("tasks", child)
        except sqlite3.IntegrityError:   # 库里已有同一步同一片：交那一个；按关联列找不到它的（关联列改过，TS-05）当记录改过
            existing = self.find_child(self.children_of(parent["id"]), role, index)
            if existing is None:
                raise StoreRefused("tampered", f"任务 {parent['id']} 的{role} {index} 已在库里、关联列却不指它") from None
            return existing
        step = {"plan": "计划", "merge": "合并"}.get(role, f"第 {index}/{count} 片")
        say(f"  · 任务 {parent['id']} 的{step}排进队：子任务 {child['id']}")
        return child

    def finalize_parent(self, parent, child, outcome, child_exit, note=""):
        failed = {}
        if outcome != "succeeded" and child is not None and not lib.broken("shard-failure-unnamed", self.environment):
            failed = {"task": child["id"], "role": child.get("role"), "index": child.get("shard_index", 0), "count": child.get("shard_count", 0),
                      "maximum_host": child.get("maximum_host", ""), "maximum_cap_text": child.get("maximum_cap_text", "")}
        try:
            parent = self.store.change("tasks", parent["id"], dict(state="finished", outcome=outcome, exit_code=lib.final_exit_code(outcome, child_exit),
                                                                   finished_at=round(time.time(), 3), note=note or (child or {}).get("note", ""), failed_step=failed),
                                       guard=lambda current: "已结了局" if current.get("state") == "finished" else "")
        except StoreRefused as refused:
            if refused.kind == "tampered":
                self.tampered(parent["id"], parent, "任务记录")
            elif refused.kind != "guard":
                raise
            return
        try:   # 有了结局就删计划那一步放的测试二进制（成、败、取消都算；文件头 ④），删完再叫醒 wait；删不成不挡记结局与叫醒
            self.shard_binaries_removal(parent["id"], parent.get("session", ""), 0)
        except (OSError, lib.Unavailable) as error:
            say(f"  ! 分片的任务 {parent['id']} 计划目录下的 {lib.SHARD_PLAN_BINARIES_NAME} 这一回没删成（{type(error).__name__}: {error}）：task clean {parent['id']} 再删")
        self.notify_finished(parent["id"])
        say(f"  · 分片的任务 {parent['id']} 结局 {outcome}{'（' + note + '）' if note else ''}")
        if outcome == "succeeded":   # 整件成了：各片的产物删掉，计划目录留着（它下面的测试二进制上面已删）
            directory = self.shard_directory(parent["id"])
            with contextlib.suppress(OSError):
                for name in os.listdir(directory):
                    if name.startswith("output-"):
                        shutil.rmtree(os.path.join(directory, name), ignore_errors=True)

    def drive_sharded_parents(self):
        """每一轮对账：还没结局的分片父任务各现推一遍（drive_parent；TS-02：子任务收尾与父任务推进之间服务没了、推进半途抛错的，在这里补推）。
        一条推不动（抛错）只写告警、不挡别的。弄坏开关 sharded-parents-unreconciled 下不推（只在子任务收尾那一刻推一次）。"""
        if lib.broken("sharded-parents-unreconciled", self.environment):
            return
        for parent, valid in self.store.select("tasks", "state = 'sharded'"):
            try:
                self.selftest_advance_fault()
                self.drive_parent(parent["id"])
                self.note_advance_recovered(parent["id"])
            except Exception as error:
                self.note_advance_failure(parent["id"], f"{type(error).__name__}: {error}")   # wait 叫醒提交方（文件头 ①「要处置的提醒」）
                say(f"  ✗ 分片的任务 {parent['id']} 这一轮推不动（{type(error).__name__}: {error}）：下一轮再推\n" + traceback.format_exc())
                self.alert("shard-advance-failed", parent["id"], f"分片的任务 {parent['id']} 推下一步时出错：{type(error).__name__}: {error}",
                           "服务每一轮对账照库里的阶段再推；一直推不动就看 journalctl --user -u singlefs-task-server.service 里这一句前后的栈")

    def drive_parent(self, parent_id):
        """照库里父任务的阶段（phase：plan、shards、merge、cancelling）与它的子任务集合现推下一步（文件头 ④；TS-02）。幂等：缺的子任务照计划补建
        （同一步同一片只一个），该收尾的收尾，已结了局的不动。计划读通之后片的计划（shard_plan）写进父任务，各片照它补建。"""
        found = self.store.task(parent_id)
        if found is None:
            return
        parent, valid = found
        if not valid:
            self.tampered(parent_id, parent, "任务记录")
            return
        if parent.get("state") != "sharded":
            return
        try:
            children = self.children_of(parent_id)
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            self.tampered(parent_id, parent, "子任务记录")
            return
        try:
            self.drive_parent_phase(parent, children)
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            self.tampered(parent_id, parent, "子任务记录")

    def drive_parent_phase(self, parent, children):
        """drive_parent 的那一步（文件头 ④）：照阶段推，读到的子任务集合是 children（都核过）。"""
        parent_id = parent["id"]
        phase = parent.get("phase") or "plan"
        if phase == "cancelling":   # 取消没停干净（TS-04）：子任务都有了结局才记整件 cancelled
            if all(child.get("state") == "finished" for child in children):
                self.finalize_parent(parent, None, "cancelled", None, "提交它的会话取消了整件（在跑的子任务都停了）")
            return
        if phase != lib.RUN_HARD_LIMIT_PHASE:
            over = self.parent_run_hard_limit(parent, children)
            if over is not None:
                try:
                    parent = self.store.change("tasks", parent_id, {"phase": lib.RUN_HARD_LIMIT_PHASE, "run_hard_limit_stop": over},
                                               guard=lambda current: "" if current.get("state") == "sharded" and current.get("phase") != "cancelling" else "阶段已变")
                except StoreRefused as refused:
                    if refused.kind == "tampered":
                        raise
                    return
                say(f"  · 分片的任务 {parent_id} {over['how']}，到了 {self.run_hard_limit_seconds:.0f} 秒的硬上限：在排的撤、在跑的停，打回")
                phase = lib.RUN_HARD_LIMIT_PHASE
        if phase == lib.RUN_HARD_LIMIT_PHASE:   # 整件满了四小时硬上限（文件头 ⑦「四小时硬上限」）：在跑的由各自的 check_stall 停，都收尾了才记结局，不合并
            over = parent.get("run_hard_limit_stop") or {}
            self.cancel_queued_children(children, "hard-limit", f"整件到了四小时硬上限，没起：{over.get('how') or '没记'}")
            if all(child.get("state") == "finished" for child in children):
                self.finalize_parent(parent, None, lib.RUN_HARD_LIMIT_OUTCOME, None,
                                     f"{over.get('how') or '没记'}，到了 {self.run_hard_limit_seconds:.0f} 秒的硬上限：在排的子任务撤了、在跑的停了，不合并，--why 放不过")
            return
        if phase == "plan":
            plan = self.find_child(children, "plan")
            if plan is None:   # 父任务记下了、计划那一步还没建（TS-02 的「父任务插入后」）
                self.create_child(parent, "plan")
                self.wake()
                return
            if plan.get("state") != "finished":
                return
            if plan.get("outcome") != "succeeded" and not lib.broken("plan-failure-proceeds", self.environment):   # 弄坏开关：计划没成照拆片
                self.finalize_parent(parent, plan, plan.get("outcome") or "launcher-lost", plan.get("exit_code"), f"计划那一步（子任务 {plan['id']}）{plan.get('outcome')}")
                return
            shards, problem = lib.read_shard_plan(os.path.join(self.shard_directory(parent_id), "plan", lib.PLAN_FILE))
            if problem:
                self.finalize_parent(parent, plan, "shard-plan-refused", None, problem)
                return
            try:
                parent = self.store.change("tasks", parent_id, dict(phase="shards", shard_count=len(shards), shard_plan=shards),
                                           guard=lambda current: "" if current.get("state") == "sharded" and (current.get("phase") or "plan") == "plan" else "阶段已变")
            except StoreRefused:
                return
            phase = "shards"
        if phase == "shards":
            planned = parent.get("shard_plan") or []
            created = False
            for index, shard in enumerate(planned):
                if self.find_child(children, "shard", index) is None:
                    children.append(self.create_child(parent, "shard", index, len(planned), label=shard.get("label", "")))
                    created = True
                    self.crash_point("shard-inserted")
            shards = [self.find_child(children, "shard", index) for index in range(len(planned))]
            failed = self.first_failed_shard(shards)
            if failed is not None:
                note = f"第 {failed.get('shard_index')}/{failed.get('shard_count')} 片（子任务 {failed['id']}）{failed.get('outcome')}"
                if lib.broken("shard-siblings-abandoned", self.environment):   # 弄坏开关：父任务当场收尾，在跑的兄弟片没人收
                    self.finalize_parent(parent, failed, failed.get("outcome") or "launcher-lost", failed.get("exit_code"), note)
                    self.cancel_queued_children(shards, "sibling", f"同一件的子任务 {failed['id']} 没成，不起")
                    return
                # 文件头「分片」：兄弟片照排、照跑到底，结局照收（判过的块照样入库），都收齐了父任务才记那一片的结局，不合并
                if all(shard.get("state") == "finished" for shard in shards):
                    self.finalize_parent(parent, failed, failed.get("outcome") or "launcher-lost", failed.get("exit_code"), note)
                return
            done = [shard for shard in shards if shard.get("state") == "finished" and shard.get("outcome") == "succeeded"]
            skipped = [shard for shard in shards if self.cancelled_alone(shard)]
            if not (len(done) + len(skipped) == len(shards) or (done and lib.broken("merge-before-all-green", self.environment))):
                if created:
                    self.wake()
                return
            self.write_outputs(parent, done)
            missing = sorted(int(shard.get("shard_index") or 0) for shard in skipped)
            try:
                parent = self.store.change("tasks", parent_id, {"phase": "merge", "missing_shards": missing},
                                           guard=lambda current: "" if current.get("state") == "sharded" and current.get("phase") == "shards" else "阶段已变")
            except StoreRefused:
                return
            self.crash_point("merge-outputs-written")
            phase = "merge"
        if phase == "merge":
            merge = self.find_child(children, "merge")
            if merge is None:   # 进了合并那一步、合并的子任务还没建（outputs.tsv 照各片最后一次尝试重写）
                if not os.path.exists(os.path.join(self.shard_directory(parent_id), "outputs.tsv")):
                    self.write_outputs(parent, [child for child in children if child.get("role") == "shard" and child.get("outcome") == "succeeded"])
                self.create_child(parent, "merge", 0, parent.get("shard_count", 0))
                self.crash_point("merge-inserted")
                self.wake()
                return
            if merge.get("state") != "finished":
                return
            if merge.get("outcome") == "succeeded" or lib.broken("merge-failure-succeeds", self.environment):   # 弄坏开关：合并没成照记成了
                self.finalize_parent(parent, merge, "succeeded", merge.get("exit_code"))
            else:
                self.finalize_parent(parent, merge, merge.get("outcome") or "launcher-lost", merge.get("exit_code"), f"合并那一步（子任务 {merge['id']}）{merge.get('outcome')}")

    @staticmethod
    def cancelled_alone(shard):
        """人手单独取消的那一片（cancel 记的 cancel_origin=session，结局 cancelled）：不算没成，合并照起、点名缺它（文件头「分片」）。"""
        return shard.get("state") == "finished" and shard.get("outcome") == "cancelled" and shard.get("cancel_origin") == "session"

    def first_failed_shard(self, shards):
        """最先没成的那一片：有了结局、不是 succeeded，人手单独取消的与兄弟片没成被服务撤掉的（cancel_origin=sibling）不算；照记下结局的先后、同一刻取片号小的。没有交 None。"""
        failed = [shard for shard in shards if shard.get("state") == "finished" and shard.get("outcome") != "succeeded" and not self.cancelled_alone(shard)
                  and not (shard.get("outcome") == "cancelled" and shard.get("cancel_origin") == "sibling")]
        return min(failed, key=lambda shard: (float(shard.get("finished_at") or 0), int(shard.get("shard_index") or 0)), default=None)

    def cancel_queued_children(self, children, origin, note):
        """撤掉 children 里还在排的那几条（兄弟片没成：origin=sibling；弄坏开关 parent-cancel-spares-running 下整件被取消也走这里）：拿那一条的锁、
        在一个事务里判还 queued 才记 cancel_requested 与 cancel_origin（起单元那一侧拿同一把锁、claim_attempt 判 cancel_requested，记上之后就不会再起），
        再记 cancelled；判的时候已经起了的不动，照跑到底。记了的那一份字典就地换成库里的新样子。"""
        for child in children:
            if child.get("state") != "queued":
                continue
            with self.store.task_lock(child["id"]):
                try:
                    current = self.store.change("tasks", child["id"], {"cancel_requested": True, "cancel_origin": origin},
                                                guard=lambda row: "" if row.get("state") == "queued" else "不在排")
                except StoreRefused as refused:
                    if refused.kind == "tampered":
                        raise
                    continue
                if self.finalize(current, "cancelled", None, note, push_parent=False):
                    child.clear()
                    child.update(current)

    def write_outputs(self, parent, shards):
        """合并那一步（文件头「分片」③）读的 outputs.tsv：各片最后一次尝试的产物目录（先写临时文件再改名，重写也是同一份）。
        子任务收尾那一侧（advance_parent）与每一轮对账（drive_sharded_parents）可能同时推到这一步：每个线程写自己名下的临时文件再改名，内容相同、后改名的覆盖先改名的，
        谁也不会找不到临时文件。弄坏开关 outputs-temporary-name-shared 下各线程共用一个固定的临时文件名，后改名的那一个抛 FileNotFoundError。"""
        lines = []
        for shard in sorted(shards, key=lambda record: record.get("shard_index", 0)):
            attempts = [attempt for attempt, _ in self.store.attempts_of(shard["id"])]
            last = attempts[-1] if attempts else {}
            lines.append(f"{shard.get('shard_index', 0)}\t{last.get('host', '')}\t{last.get('shard_output', '')}")
        directory = self.shard_directory(parent["id"])
        lib.ensure_private_directory(directory)
        outputs = os.path.join(directory, "outputs.tsv")
        temporary = outputs + ".partial" if lib.broken("outputs-temporary-name-shared", self.environment) else f"{outputs}.{os.getpid()}.{threading.get_ident()}.partial"
        with open(temporary, "w", encoding="utf-8") as handle:
            handle.write("".join(line + "\n" for line in lines))
        os.replace(temporary, outputs)

    def whole_threshold(self, host, snapshot):
        """整片的界（字节）：这台可用上限（总上限减这台此刻的余量，host_capacity；booked_at_cap 收到的那个数）× WHOLE_FRACTION（oversized-budget 提醒的界）。
        只对这一轮派得了的台调（place 先滤掉了余量量不到的台）；量不到的抛 ValueError，不退回别的数。"""
        capacity, why = self.host_capacity(host, snapshot)
        if capacity is None:
            raise ValueError(f"整片的界算不了：{why}")
        return int(capacity * WHOLE_FRACTION)

    def is_whole(self, booked, host, snapshot):
        return int(booked or 0) >= self.whole_threshold(host, snapshot)

    def aged(self, task, now):
        """提交满 aged_seconds 还在排：高优先（文件头「排队」）。task promote 提前过的不在这里：它排最前（queue_order）、不过 CPU 闸（cpu_judgement），内存放得下就起，不挡后面
        （弄坏开关 promote-aged-blocks-queue 下提前过的排满时限照算，起不来就挡住后面全部）。"""
        if task.get("promoted") and not lib.broken("promote-aged-blocks-queue", self.environment):
            return False
        return now - float(task.get("submitted_at") or now) >= self.aged_seconds and not lib.broken("aged-task-not-prioritized", self.environment)

    def queue_order(self, queued, now):
        """排队的先后：task promote 提前过的最前，再是高优先的，各自按原来的次序（排序是稳定的）。弄坏开关 promote-queue-order-ignored 下不看提前过没有。"""
        promoted_ignored = lib.broken("promote-queue-order-ignored", self.environment)
        return sorted(queued, key=lambda row: (promoted_ignored or not row[0].get("promoted"), not self.aged(row[0], now)))

    def promote(self, task_id, session, process_id):
        """task promote <号>（客户端那一条执行前由钩子弹窗要用户确认）→ (HTTP 状态码, 回话)：只许提交它的会话；任务要在排（queued）；
        记 promoted {at}，下一轮排队起它排最前（queue_order）、不过 CPU 闸（cpu_judgement，内存放得下就起）。
        提前过的再提前照 200、不改。分片确认过的父任务（sharded）同样许提前：父任务记 promoted，此刻在排的子任务各记一份，之后建的计划、各片、合并照父任务继承（create_child）。
        改完当场叫醒排队线程（wake），不干等下一轮。
        弄坏开关 promote-session-unchecked 下不看是不是提交它的会话，promote-unwoken 下不叫醒，promote-shards-refused 下分片的父任务照「不在排」拒。"""
        found = self.store.task(task_id)
        if found is None:
            return 404, {"detail": f"没有任务 {task_id}"}
        task, valid = found
        if not valid:
            self.tampered(task_id, task, "任务记录")
            return 409, {"detail": f"任务 {task_id} 的记录校验不过，已记 record-tampered，不提前"}
        if task.get("session") != session and not lib.broken("promote-session-unchecked", self.environment):
            self.operation("promote-refused", task_id, session, process_id, "不是提交它的会话")
            return 403, {"detail": f"任务 {task_id} 不是这个会话提交的，不许提前"}
        if task.get("promoted"):
            return 200, {"state": task.get("state"), "detail": f"任务 {task_id} 原来就提前过"}
        promotable = ("queued",) if lib.broken("promote-shards-refused", self.environment) else ("queued", "sharded")
        mark = {"promoted": {"at": round(time.time(), 3)}}
        try:
            self.store.change("tasks", task_id, mark, guard=lambda current: "" if current.get("state") in promotable else "不在排")
        except StoreRefused as refused:
            if refused.kind != "guard":
                raise
            return 409, {"detail": f"任务 {task_id} 此刻 {task.get('state')}、不在排，不用提前"}
        if task.get("state") == "sharded":   # 分片的父任务：此刻在排的子任务各记一份（之后建的照父任务继承，create_child）
            for child in self.children_of(task_id):
                with contextlib.suppress(StoreRefused):
                    self.store.change("tasks", child["id"], mark, guard=lambda current: "" if current.get("state") == "queued" and not current.get("promoted") else "不在排")
        self.operation("promote", task_id, session, process_id, "提前到队头")
        if not lib.broken("promote-unwoken", self.environment):
            self.wake()   # 当场排一轮，不干等 TICK_SECONDS
        return 200, {"state": task.get("state"), "detail": f"任务 {task_id} 提前到队头：下一轮排队起排最前、不过 CPU 闸，内存放得下就起"}

    def positions_in(self, ordered, now):
        """排好先后的在排任务各排第几：{任务号: (排第几（从 1 起）, 在排共几条, 排在它前面而已排满时限的几条)}。
        先后就是 queue_order 给的（排满时限的在前、各自按任务号）；排满时限还起不来的那一条挡住后面全部（文件头 ②「排队先后」①），所以第三个数另报。"""
        positions, aged_ahead = {}, 0
        for index, row in enumerate(ordered):
            positions[row[0]["id"]] = (index + 1, len(ordered), aged_ahead)
            if self.aged(row[0], now):
                aged_ahead += 1
        return positions

    def queue_position_reply(self, task):
        """status 里排着的任务的全局排位：{position, queued, aged_ahead}；不在上一轮的排队表里（刚进队还没排过一轮、不在 queued）交 None。"""
        found = self.queue_positions.get(task["id"]) if task.get("state") == "queued" else None
        return None if found is None else {"position": found[0], "queued": found[1], "aged_ahead": found[2]}

    def head_judged_apart(self, pass_state=None):
        """只给弄坏开关 health-head-judged-apart 与 health-judges-head：/health 自己另判一套排头放不放得下，交 HeadVerdict（队空交 None）。
        另判的只看内存（judged_on_host），不看 CPU 闸与编译目录占盘（任务 7568 卡在 CPU 闸、2365 等一件 /health 不认得的事，都被报成放得下却没起）。
        pass_state 给了照它取排头、用它的余量；没给（health-judges-head 下 /health 现调）就现读库与余量。"""
        if pass_state is None:
            queued = [row for row in self.store.select("tasks", "state = 'queued'", order="id") if row[1]]
            view = None
        else:
            queued = sorted((row for row in pass_state["queued"] if row[1] and row[0]["id"] not in pass_state["gone"]), key=lambda row: row[0]["id"])
            view = pass_state["view"]
        if not queued:
            return None
        head = self.queue_order(queued, time.time())[0][0]
        view = view or PassView(self, "health")
        snapshot = view.snapshot()
        if snapshot.hold:
            return HeadVerdict(head["id"], HEAD_BLOCKED, "另判：自证的假余量写了 hold")
        prediction = self.prediction_of(head)
        for host in self.candidate_hosts(head, snapshot):
            if host not in snapshot.reports or self.host_limit(prediction, host, snapshot)[0] is None:
                continue   # 余量量不到的台这一轮不派（文件头 ②「余量」）
            verdict = self.judged_on_host(head, prediction, host, snapshot, view.pending_of(host))[1]
            if verdict.fits:
                return HeadVerdict(head["id"], HEAD_NOT_STARTED, "另判：内存放得下")
        return HeadVerdict(head["id"], HEAD_BLOCKED, "另判：放不下")

    def starved_problem(self, now, fresh=False):
        """派活判排头能起却连着 starved_seconds 没起成：交一句问题，不然交空串。只读派活这一轮给排头的结论（head_verdict，place_queued 记下），不另判放不放得下：
        派活判不能起的（等负载、等占盘、排在排满时限的任务后面、第二台这一轮起满、起之前复核没过，原因都在排队原因里）与判不了的一律不报，不认排队原因里的字样。
        服务暂停着（文件头 ⑤「暂停」）排着不起是要的，交空串；弄坏开关 pause-judges-starved 下另判（暂停时排队线程不判，没有结论可读）。
        弄坏开关 health-head-judged-apart 下另判一套（head_judged_apart，用这一轮的余量）；health-judges-head 下由 /health 现调、fresh 为真：另判一套且现读库与余量；
        blocked-head-counted-as-starved 下派活判不能起的排头也当成判能起没起成。"""
        if self.pause is not None:
            if not lib.broken("pause-judges-starved", self.environment):
                self.fits_since = {}
                return ""
            fresh = True
        if fresh or lib.broken("health-head-judged-apart", self.environment):
            try:
                verdict = self.head_judged_apart(None if fresh else self.pass_state)
            except (OSError, ValueError, KeyError, subprocess.SubprocessError):
                return ""
        else:
            verdict = self.head_verdict
        startable = verdict is not None and (verdict.kind == HEAD_NOT_STARTED
                                             or (verdict.kind == HEAD_BLOCKED and lib.broken("blocked-head-counted-as-starved", self.environment)))
        if not startable:
            self.fits_since = {}
            return ""
        since = self.fits_since.get(verdict.task_id, now)
        self.fits_since = {verdict.task_id: since}
        if now - since > self.starved_seconds and not lib.broken("starved-head-unreported", self.environment):
            return f"任务 {verdict.task_id} 排在最前、派活判它此刻放得下，却 {now - since:.0f} 秒没起成（{verdict.why}）"
        return ""

    def progress_problem(self, now):
        """看进展（文件头「接口」；派活判法自己判错时要有人看得到）：有排着的任务、某台此刻照现行公式
        （room_by_the_formula）放得下排着的最小那条的记账，而这台连着 progress_seconds 一条都没起，交一句点名哪台、照公式还能记多少、最小那条是哪条；不然交空串。
        不读派活的结论（head_verdict、排队原因、Placement），只读这一轮排队读的余量（PassView，不另读）、库里在排的任务与起没起（progress_started_hosts）。
        派活本来就该等的几种各自照余量另外不算（progress_smallest_fitting）。服务暂停着不判（排着不起是要的），判不了交空串。
        弄坏开关 progress-unwatched 下一律交空串。"""
        try:
            started = self.progress_started_hosts()
            if self.pause is not None or lib.broken("progress-unwatched", self.environment):
                self.progress_since = {}
                return ""
            for host in started:
                self.progress_since.pop(host, None)   # 这台这一轮起过：从此刻重新算
            view = (self.pass_state or {}).get("view")
            queued = [task for task, valid in self.store.select("tasks", "state = 'queued'", order="id") if valid] if view is not None else []
            if not queued:
                self.progress_since = {}
                return ""
            human = self.host_dispatch.human
            problems = []
            for host in MACHINE_ORDER:
                fitting = self.progress_smallest_fitting(host, queued, view)
                if fitting is None:
                    self.progress_since.pop(host, None)
                    continue
                since = self.progress_since.setdefault(host, now)
                if now - since >= self.progress_seconds:
                    room, task_id, booked = fitting
                    problems.append(f"{self.machines[host].label}照公式（{self.formula_label()}）还能记 {human(room)}、排着的最小那条任务 {task_id}（记 {human(booked)}）放得下，"
                                    f"这台却 {now - since:.0f} 秒一条都没起（看进展，不看派活的结论）")
            return "；".join(problems)
        except (OSError, ValueError, KeyError, subprocess.SubprocessError, sqlite3.Error, StoreRefused):
            return ""

    def progress_started_hosts(self):
        """看进展：尝试号比上一轮数到的最大号大的那几次尝试起在哪几台（起了又收尾的也数得到），交台的集合；头一回调只记下此刻的最大号、交空集。"""
        seen = self.progress_attempt_seen
        top, hosts = seen or 0, set()
        for attempt, _ in self.store.select("attempts", "id > ?", (seen if seen is not None else 0,)):
            top = max(top, int(attempt["id"]))
            if seen is not None:
                hosts.add(attempt.get("host"))
        self.progress_attempt_seen = top
        return hosts

    def progress_smallest_fitting(self, host, queued, view):
        """看进展的一台：照公式这台还能记的（room_by_the_formula）放得下排着的最小那条的记账，交 (还能记的, 那条的任务号, 它在这台的记账)，放不下或不判交 None。
        不判的：自证的 hold、这台余量量不到或连不上；这台 slice 兜底停派着（slice_floor_problem）。
        挑最小那条时跳过派不到这台的、这台给不出上限的、预测峰值比这台大而别处可能装得下的（hosts_short_of_record：这台不派它），与本来就该等的：
        这台负载过 CPU 闸（cpu_problem）、编译目录的盘不够（build_disk_problem）、slice 实测不到下限（slice_floor_problem）。
        弄坏开关 progress-gates-ignored 下照公式放得下就算：不看 CPU、占盘与 slice 兜底。"""
        snapshot = view.snapshot()
        if snapshot.hold or host not in snapshot.reports or not snapshot.reachable.get(host):
            return None
        room = self.room_by_the_formula(host, snapshot)
        if room is None:
            return None
        gated = not lib.broken("progress-gates-ignored", self.environment)
        if gated and (self.slice_floor_problem(host, snapshot) or self.machine_memory_problem(host, snapshot)):
            return None   # slice 兜底、整机内存线停派着（slice_floor_problem、machine_memory_problem）：本来就该等
        smallest = None
        for task in queued:
            if host not in self.candidate_hosts(task, snapshot):
                continue
            prediction = self.prediction_of(task)
            if self.host_limit(prediction, host, snapshot)[0] is None:
                continue
            if host in self.hosts_short_of_record(prediction, self.candidate_hosts(task, snapshot), snapshot):
                continue   # 这台不派它：预测峰值比这台大、别处可能装得下（judge_placement 照同一处判）
            if gated:
                reason = self.waiting_reasons.get(task["id"], "")
                if "已排满时限" in reason or "不插队" in reason:
                    continue   # 照排队规则本来就该等的：排在排满时限的任务后面，或前面有没起的而它不许插队（place_queued_into、jump_refusal）
                if (self.cpu_problem(task, host, snapshot) or self.build_disk_problem(task, host, snapshot, view) or self.build_pool_problem(task, host)
                        or self.io_pressure_problem(task, host, snapshot)):
                    continue   # 等负载、等占盘、等编译目录组里空出一个、等磁盘压力降下来
            booked = int(self.booked_at_cap(prediction, host, snapshot).memory_booked or 0)
            if smallest is None or booked < smallest[1]:
                smallest = (task["id"], booked)
        if smallest is None or smallest[1] > room:
            return None
        return room, smallest[0], smallest[1]

    def place_queued(self):
        """排一轮；排队原因写进一张新表、这一轮排完才换上（status 读不到排了一半的表）。派活给这一轮每条的结论（HeadVerdict）同样记下，
        排头（这一轮头一条判了的：排队的先后，收尾了的、记录坏了的不算）那一条换进 head_verdict，/health 判「判能起却没起成」只读它。"""
        waiting, verdicts = {}, {}
        try:
            self.place_queued_into(waiting, verdicts)
        finally:
            self.waiting_reasons = waiting
            self.head_verdict = next(iter(verdicts.values()), None)

    def place_queued_into(self, waiting, verdicts):
        """排一轮的本体：waiting 收每条排着的在等什么，verdicts 照排队的先后收派活给每条的结论（{任务号: HeadVerdict}，每条只记头一个）。
        自证的 TASK_SERVER_SELFTEST_PLACEMENT_PAUSED 那份文件在时照判、判能起的不起（记判能起却没起成）；TASK_SERVER_SELFTEST_PLACEMENT_REFUSED 那份文件在时
        每条都判成不能起（演派活判法判错，看进展那一道该报）。"""
        self.pass_state = None   # 这一轮没排就是 None
        # 第二台起满了才隔 PEER_REPASS_SECONDS 再排（repass_soon）：每一轮开头先清，队空、暂停着提前回的那几轮不再照上一轮的留着、主循环不每 2 秒空转
        # （loop 审核发现 12）；弄坏开关 repass-kept-when-idle 下照队里有活的那几轮才清
        if not lib.broken("repass-kept-when-idle", self.environment):
            self.repass_soon = False

        def judged(task, kind, why=""):
            verdicts.setdefault(task["id"], HeadVerdict(task["id"], kind, why))

        pause = self.pause_in_force()
        queued = self.store.select("tasks", "state = 'queued'", order="id DESC" if lib.broken("queue-reversed", self.environment) else "id")
        # 全队（暂停着也是全队）：排位（queue_positions）与留着的预测（keep_predictions_of）照它算，别的会话排着的任务排位不丢、预测不清。
        # 弄坏开关 pause-queue-positions-filtered 下照放行的那几条算
        whole = queued
        if pause is not None:   # 服务暂停着（文件头 ⑤「暂停」）：只排放行的会话交的，别的会话的这一轮不起；排着的任务在等什么由 waiting_text 照暂停写
            queued = [row for row in queued if self.allowed_during_pause(row[0], pause)]
            if lib.broken("pause-queue-positions-filtered", self.environment):
                whole = queued
            if not queued:
                return
        if not queued:
            self.pass_state = {"queued": [], "gone": set(), "view": None}
            self.selftest_read("pass view=- queued=0")   # 队空的一轮也记（自证的 next_pass 等它）
            return
        now = time.time()
        self.repass_soon = False
        started_on = {}   # {台: 这一轮在那台真起了几条（尝试在 starting 或 running）}：有的台一轮至多起 Machine.starts_per_pass 条
        queued = self.queue_order(queued, now)
        self.queue_positions = self.positions_in(self.queue_order(whole, now), now)
        awaiting = {task["id"] for task, _ in self.store.select("tasks", "state = ?", (lib.AWAITING_CONFIRMATION_STATE,))}   # 确认单算过的预测留着，进队之后照用（文件头 ⑦）
        self.keep_predictions_of({task["id"] for task, _ in whole} | awaiting)   # 不在队里了的（起了、收尾了）删掉存的预测
        view = PassView(self, "pass")   # 这一轮的余量：每台读一次、各条共用（文件头 ②「余量一轮只读一次」）
        gone = set()   # 这一轮起了或收尾了的任务号（判排头时跳过它们）
        self.pass_state = {"queued": queued, "gone": gone, "view": view}
        self.selftest_read(f"pass view={view.number} queued={len(queued)}")
        # 排队只照两条规则：排满时限还起不来的挡住后面全部；前面有没起的，后面的只拦争同一样资源的（jump_refusal）

        def behind(position, why):
            for later, _ in queued[position + 1:]:
                waiting[later["id"]] = why

        earlier_blocked = []   # 这一轮判过、还没起的任务号（照排队先后）：它后面的任务要起就是插队，照 jump_refusal 判
        pool_waiters = {}      # {任务号: 仓根}：earlier_blocked 里只缺这棵树的编译目录名额的那几条（pool_only_tree）；别的资源它们不争，jump_refusal 只拦同一棵树也要编译目录的
        for position, (task, valid) in enumerate(queued):
            if not valid:
                self.tampered(task["id"], task, "任务记录")
                gone.add(task["id"])
                continue
            try:
                if self.placement_refused and os.path.exists(self.placement_refused):
                    placement = Placement("blocked", why="自证：TASK_SERVER_SELFTEST_PLACEMENT_REFUSED 那份文件在，派活一律判不能起（演派活判法判错）")
                else:
                    placement = self.place(task, view)
            except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
                say(f"  ! 任务 {task['id']} 这一轮判不了放不放得下（{type(error).__name__}: {error}），留在队里")
                waiting[task["id"]] = f"这一轮判不了放不放得下（{type(error).__name__}: {error}），下一轮再判"
                self.idle_short_judged.discard(task["id"])
                judged(task, HEAD_BLOCKED, waiting[task["id"]])
                if lib.broken("judge-error-blocks-queue", self.environment):
                    behind(position, f"排在任务 {task['id']} 后面：它这一轮判不了放不放得下")
                    return   # 弄坏开关：一条判不了就挡住整队
                continue   # 只跳过它这一轮，后面的照判（一条记录坏了不该把整队挡住）
            if placement.kind == "never":
                self.finalize(task, "never-fits", None, placement.why)
                gone.add(task["id"])
                continue
            if placement.kind == "unmet":
                self.finalize(task, "run-condition-unmet", None, placement.why)
                gone.add(task["id"])
                continue
            self.budget_check(task, view)   # 文件头 ②「预算」：记账贴着某一台整片界的记一条 oversized-budget
            if placement.kind == "blocked":
                waiting[task["id"]] = placement.why
                judged(task, HEAD_BLOCKED, placement.why)
                if placement.idle_short:
                    # 在每一台上就算服务在那台跑的任务全收尾、整机空闲也不够要的（idle_short_text）：它等的是整机自己空下来，排在前面的哪条跑完都帮不上它，
                    # 不算「排在前面还没起的」，排满时限也不挡后面的；记一条提醒 booking-beyond-idle-machine。弄坏开关 idle-short-blocks-later 下照一般放不下的那样挡后面
                    self.note_idle_short(task, placement)
                    self.note_idle_short_judged(task, placement)
                    if not lib.broken("idle-short-blocks-later", self.environment):
                        continue
                else:
                    self.idle_short_notices.pop(task["id"], None)
                    self.note_idle_short_judged(task, placement)
                if self.blocks_later(task):
                    earlier_blocked.append(task["id"])
                    if placement.build_tree and not lib.broken("jump-refused-any-resource", self.environment):
                        pool_waiters[task["id"]] = placement.build_tree
                if self.aged(task, now) and not lib.broken("aged-blocked-lets-others-start", self.environment):
                    # 排满时限（两小时）还起不来的这一条挡住排在它后面的全部任务：它起来之前，后面的哪一台都不起。
                    # 不这样，线以下一直有要得少的任务补位，要得多的这一条永远够不着（任务 14789 要 31 线程，排了三个半小时一次没起）。
                    behind(position, f"排在任务 {task['id']} 后面：它已排满时限（{self.aged_seconds / 3600:.1f} 小时）还没起，它起来之前后面的都不起")
                    return
                if lib.broken("backfill-disabled", self.environment):
                    behind(position, f"排在任务 {task['id']} 后面")
                    return   # 弄坏开关：退回严格排队，排头放不下后面的都等
                continue
            self.note_idle_short_judged(task, placement)   # 放得下：「在每一台上都起不来」判过、不是（起之前复核、插队、一轮起的条数再把它留在队里，oversized-budget 照报）
            machine = self.machines[placement.host]
            if (machine.starts_per_pass is not None and started_on.get(placement.host, 0) >= machine.starts_per_pass
                    and not lib.broken("peer-starts-unthrottled", self.environment)):
                waiting[task["id"]] = f"{machine.label}这一轮已起 {machine.starts_per_pass} 条（一齐起太多 ssh 会被重置），{PEER_REPASS_SECONDS:.0f} 秒后再排"
                judged(task, HEAD_BLOCKED, waiting[task["id"]])
                self.repass_soon = True
                continue
            if self.placement_paused and os.path.exists(self.placement_paused):
                judged(task, HEAD_NOT_STARTED, "自证：TASK_SERVER_SELFTEST_PLACEMENT_PAUSED 那份文件在，排队线程判完不起（起之前复核也不做）")
                continue   # 自证：照判、不起
            placement, why = self.launch_recheck(task, placement, view)
            if placement is None:   # 起之前现读的数放不下：留在队里（文件头 ②「起之前复核」）
                waiting[task["id"]] = f"起之前复核没过（现算的上限、现读的余量）：{why}"
                judged(task, HEAD_BLOCKED, waiting[task["id"]])
                if self.blocks_later(task):
                    earlier_blocked.append(task["id"])
                if self.aged(task, now) and not lib.broken("aged-blocked-lets-others-start", self.environment):
                    behind(position, f"排在任务 {task['id']} 后面：它已排满时限（{self.aged_seconds / 3600:.1f} 小时）还没起，它起来之前后面的都不起")
                    return
                continue
            if earlier_blocked and not lib.broken("jump-length-unchecked", self.environment):
                # 排在它前面的任务还没起，这一条要起就是插队，只拦争同一样资源的（jump_refusal）：前面缺内存、CPU 这类的，只许有上次运行时长、
                # 且不超过这台在跑的任务剩余时长的插；前面的都只缺一棵树的编译目录名额，只拦同一棵树也要编译目录的
                jump, tree = self.jump_refusal(task, placement.host, earlier_blocked, pool_waiters, now)
                if jump:
                    waiting[task["id"]] = jump
                    judged(task, HEAD_BLOCKED, jump)
                    earlier_blocked.append(task["id"])
                    if tree:
                        pool_waiters[task["id"]] = tree   # 它也在等这棵树的编译目录名额：它后面的照同一条判，不当缺内存、CPU 的
                    continue
            attempt, held = self.start_attempt_unless_paused(task, placement)
            if held:
                return   # 判完放得下、起之前服务暂停了（pause 与这一处共锁）：这一轮到此为止，一条都不再起
            gone.add(task["id"])
            self.selftest_read(f"start task={task['id']} host={placement.host} view={view.number}")   # 自证：哪一轮在哪台起的（peer-starts-throttled 照它数每轮起几条）
            if attempt is not None and attempt.get("state") in ACTIVE_ATTEMPT_STATES:
                judged(task, HEAD_STARTED)
                view.book(placement.host, placement.booked, placement.disk_booked)   # 这一轮后面的照加过的判（文件头 ②「余量一轮只读一次」）
                started_on[placement.host] = started_on.get(placement.host, 0) + 1   # 只数真起了的：没起成的（取消了、systemd-run 起不来）不占这一轮的名额
            else:
                judged(task, HEAD_NOT_STARTED, f"起尝试没成：{'没记下尝试' if attempt is None else '尝试 ' + str(attempt.get('state')) + '、' + str(attempt.get('outcome'))}")
                if lib.broken("failed-start-counted", self.environment):   # 弄坏开关：没起成的也算进这一轮在那台起了几条
                    started_on[placement.host] = started_on.get(placement.host, 0) + 1

    def blocks_later(self, task):
        """这一轮判过、没起的这一条算不算「排在前面还没起的」（place_queued_into 的 earlier_blocked：它后面的要起就是插队，争同一样资源的照 jump_refusal 拦）：
        task promote 提前过的不算（它排最前，起不来不挡后面）。弄坏开关 promote-blocked-counts-as-earlier 下提前过的也算。"""
        return not task.get("promoted") or lib.broken("promote-blocked-counts-as-earlier", self.environment)

    def jump_refusal(self, task, host, blocked, pool_waiters, now):
        """插队规则：blocked（排在前面还没起的任务号，照排队先后）里有没起的，这一条想先在 host 上起。→ (许插交空串、不许交排队原因, 拦它的是哪棵树的编译目录名额，不是它的交空串)。
        只拦争同一样资源的：前面的任务按缺的资源分开判。
        ·只缺一棵树的编译目录名额的（pool_waiters）：名额是那棵树在那一台的编译目录，这一条起来不占它，不拦；只拦同一棵树也要服务给编译目录的（它起来要拿同一个组里的名额），
         排队原因写那一棵树。
        ·缺的是内存、CPU 这类的（blocked 里别的）：这一条起来占的正是它们等的资源，照长度判：在跑的都算不出剩余时长时，有上次运行时长的许插；算得出时两条都满足才许：
         这一条有上次的运行时长（last_run_seconds：这把命令键上次真跑完用了多久，是记录、不估）；这个时长不超过 host 上在跑的任务剩余时长里最长的那个
         （剩余 = 那条任务上次的运行时长减已跑；没有上次时长、或已经跑过上次时长的不算数）。这样插进去的在挡着排头的那几条跑完之前就结束，不耽误排头。"""
        tree = lib.build_tree_root(task["cwd"]) if self.server_gives_build_directory(task) else ""
        for head in blocked:
            if tree and pool_waiters.get(head) == tree:
                return f"排在任务 {head} 后面、它还没起：它等的是这棵树（{tree}）的编译目录名额，这一条也要这棵树的编译目录、争同一样资源，不插队", tree
        others = [head for head in blocked if head not in pool_waiters]
        if not others:
            return "", ""   # 前面的都只缺别的树的编译目录名额：这一条不争它们等的资源
        head = others[0]
        waits_for = "它缺的是内存、CPU 这类这一条起来也要占的资源（不是编译目录名额），"
        estimate = self.last_run_seconds(task)[0]   # 这把命令键上次真跑完用了多久：记录，不是估的
        if estimate is None:
            return f"排在任务 {head} 后面、它还没起：{waits_for}这一条没有上次的运行时长，不插队", ""
        remaining = []
        for attempt in self.booking_attempts():
            if attempt.get("host") != host:
                continue
            found = self.store.task(attempt.get("task_id"))
            expected = self.last_run_seconds(found[0])[0] if found else None
            elapsed = max(0.0, now - float(attempt.get("started_at") or now))
            if expected is not None and expected > elapsed:
                remaining.append(expected - elapsed)
        label = self.machines[host].label
        if not remaining:
            return "", ""   # 在跑的都算不出剩余时长（第一次跑、没有上次运行时长）：有上次运行时长的这一条许插；队头排满时限之后照样把后面全挡住
        if estimate > max(remaining):
            return (f"排在任务 {head} 后面、它还没起：{waits_for}这一条上次跑了 {estimate:.0f} 秒，超过{label}在跑的任务剩余的时长（最长 {max(remaining):.0f} 秒），不插队"), ""
        return "", ""

    def host_eligible(self, task, host, snapshot):
        """这一条派不派得到这一台（task_server_lib.py 文件头「挑机器」）：只看这一台此刻连不连得上（Snapshot.reachable），余量由 place 判；共享根外的路径、共享根、
        判定库三种在提交时就拒了（文件头 ②「提交时拒」），这里不再判。没有只派本机的名单：哪个脚本派到第二台跑红了由脚本那一侧修。
        第二台在同一棵共享的树里、同一个绝对路径上跑：改主工作区的工具照样派得过去。"""
        if not snapshot.reachable.get(host):
            return False
        if self.machines[host].always_reachable:
            return True   # 下面两个弄坏开关只派服务所在这一台
        if self.restored_local_only(task):
            return False   # 弄坏开关 host-lists-restored：点到只在本机的名单时不派第二台
        if lib.broken("workspace-writers-local-only", self.environment) and self.modifies_workspace(task):
            return False   # 弄坏开关：把改主工作区的工具只派本机
        return True

    def restored_local_only(self, task):
        """弄坏开关 host-lists-restored 下：命令点到删之前只在本机的那几张名单（RESTORED_HOST_LISTS 的目录与入口；bash -c 这类整条写在一个词里的按词里出现的目录认）交 True；
        开关没开一律 False。"""
        if not lib.broken("host-lists-restored", self.environment):
            return False
        root = os.path.realpath(self.peer_root)
        cwd = os.path.realpath(task["cwd"])
        directories = RESTORED_HOST_LISTS["local_directories"]
        prefixes = [os.path.join(root, directory) + os.sep for directory in directories]
        if (any(os.path.normpath(os.path.join(cwd, word)).startswith(prefix) for word in task["command"] for prefix in prefixes)
                or any(re.search(r"(?:^|[\s\x27\x22=;|&(<>/])" + re.escape(directory) + "/", word) for word in task["command"] for directory in directories if " " in word)):
            return True
        relative = [os.path.relpath(os.path.normpath(os.path.join(cwd, word)), root) for word in task["command"] if not word.startswith("-")]
        return any(path in RESTORED_HOST_LISTS["local_entries"] for path in relative)

    def modifies_workspace(self, task):
        """这条命令改不改主工作区（改了跑第二遍结果就不同，撞上限不原样重排）：WORKSPACE_WRITERS 里的工具，命令那一层的词、bash -c 的串与命令位置上起的脚本正文里的都认
        （workspace_swap_tool）；合入后验证（MERGE_AND_VERIFY_ENTRY）不算，
        弄坏开关 merge-and-verify-counted-as-writer 下照算（撞上限不重排、中断了报 workspace-write-interrupted）；
        hook-swap.py 带 --feed-only 的只喂不换、不写文件，不算（任务 2461：喂钩子撞了 44M 被当成改工作区打回）。弄坏开关 feed-only-counted-as-writer 下照算。
        带了 WORKSPACE_READ_ONLY_ARGUMENTS 之一的只读不写，不算（apply-writer-patch.py --dry-run 只在临时树上核：任务 7442 撞 476M、7444 撞 4096M，都被当成改工作区打回；
        自证、列项、读结局同理：任务 7543、7549）。弄坏开关 read-only-arguments-counted-as-writer 下照算。"""
        relative = self.command_paths_in_repository(task)
        if (any(argument in WORKSPACE_READ_ONLY_ARGUMENTS for argument in task["command"])
                and not lib.broken("read-only-arguments-counted-as-writer", self.environment)):
            return False
        # 合入后验证（merge-and-verify.sh）不算改主工作区：它只在自己拷的快照上打补丁与验，主工作区一个字节不写（它文件头「快照（不拿锁）」），撞了上限照常放大重排
        if MERGE_AND_VERIFY_ENTRY in relative and lib.broken("merge-and-verify-counted-as-writer", self.environment):
            return True
        return self.workspace_swap_tool(task)

    def command_paths_in_repository(self, task):
        """命令里不带 - 的词当路径，照任务的工作目录折成相对本仓根的路径（modifies_workspace、workspace_swap_tool 认脚本用）。"""
        root = os.path.realpath(self.peer_root)
        return self.repository_paths(task["command"], os.path.realpath(task.get("cwd") or root))

    def repository_paths(self, words, cwd):
        """词表里不带 - 的词当路径，照 cwd 折成相对本仓根的路径：命令那一层（command_paths_in_repository）与 bash -c 的串、脚本正文里切出的命令共用这一种折法。"""
        root = os.path.realpath(self.peer_root)
        return [os.path.relpath(os.path.normpath(os.path.join(cwd, word)), root) for word in words if not word.startswith("-")]

    def workspace_swap_tool(self, task):
        """命令是不是 WORKSPACE_WRITERS 里的工具在换文件、打补丁（换钩子 hook-swap.py、打补丁 apply-writer-patch.py、stage-mine.py）：这几个工具把结局文件写到一半被杀，之后所有会话的换上都被挡
        （workspace_writer_floor 只给它们留余量）。不含合入后验证 merge-and-verify（不写主工作区，它的内存是编译与测试的，照这把键的记录定上限）。
        先判命令那一层（writer_command_verdict）：带 WORKSPACE_READ_ONLY_ARGUMENTS 之一的只读不写、hook-swap.py 带 --feed-only 的只喂不换，都不算，也不往里看；
        折成相对仓根的路径里有写工作区工具的算。那一层判不出的，再看包在里面的（wrapped_workspace_commands：bash -c 的串、命令位置上起的脚本正文）。
        认到的是哪几条由 workspace_writer_commands 交出，这里只看认没认到。"""
        return bool(self.workspace_writer_commands(task))

    def command_workspace_writer(self, task):
        """这条任务命令那一层的整条当一条写工作区工具（workspace_write_judged 认不到包着的那一条时照它）：词照原样，工作目录与起时的当前目录都是任务的。"""
        cwd = task.get("cwd") or self.peer_root
        command = list(task.get("command") or [])
        return WorkspaceWriter(command, cwd, list(command), cwd)

    def workspace_writer_commands(self, task):
        """这条任务里认到的每一条写工作区工具的命令 → [WorkspaceWriter…]，没认到交 []。workspace_swap_tool 判有没有；writer_is_hook_swap（换钩子退 7 找上一趟）
        与 workspace_write_judged（被杀时逐条定写工具和要核的路径）都取这一份，不另切词。命令那一层（writer_command_verdict）判出写的交整条命令，判出只读、只喂的交 []、不往里看；
        判不出的交包在里面认到的、不是只读也不是只喂的每一条（wrapped_workspace_commands）。认它与把词折成相对仓根的路径都照任务的工作目录。"""
        verdict = self.writer_command_verdict(task["command"], self.command_paths_in_repository(task))
        if verdict is not None:
            return [self.command_workspace_writer(task)] if verdict else []
        return [writer for writer, writes in self.wrapped_workspace_commands(task) if writes]

    def writer_command_verdict(self, words, relative):
        """一条命令（交上来的整条，或 bash -c 的串、脚本正文里切出的一条）照它自己的词判（relative 是它的词折成相对仓根的路径，repository_paths）：
        带 WORKSPACE_READ_ONLY_ARGUMENTS 之一的、hook-swap.py 带 --feed-only 的交 False（只读、只喂，不写）；路径里有 WORKSPACE_WRITERS 之一的交 True；都不是交 None（这一条判不出）。
        弄坏开关 read-only-arguments-counted-as-writer、feed-only-counted-as-writer 下各不认那一种例外（照算改工作区）。"""
        if (any(argument in WORKSPACE_READ_ONLY_ARGUMENTS for argument in words)
                and not lib.broken("read-only-arguments-counted-as-writer", self.environment)):
            return False
        if HOOK_SWAP_ENTRY in relative and "--feed-only" in words and not lib.broken("feed-only-counted-as-writer", self.environment):
            return False
        if any(path in WORKSPACE_WRITERS for path in relative):
            return True
        return None

    def wrapped_workspace_commands(self, task):
        """命令那一层没有写工作区工具时，包在里面认到的每一条写工作区工具 → [(WorkspaceWriter, 写不写)…]，次序照下面先 ① 后 ②、各自照先后：
        ① shell 的 -c 串（shell_script_word_indexes：-c、-ec 这类，套在 env 这类包装里的照认）照 shell 词法切成几条命令（shell_text_runs），每一条照命令那一层同一种判法
           （writer_command_verdict）：词折成相对仓根的路径里有写工作区工具的交出，带只读参数或 --feed-only 的记「不写」（只免它自己那一条，同一个串里别的条照判）；
        ② 命令位置上起的脚本（命令那一层与 ① 里判不出的每一条各看一次，script_body_workspace_commands）正文里命令位置上写着的，同样交出、同样记写不写。
        每一项的 expanded 与 directory 照 shell_text_runs 算：-c 串从任务的工作目录起（前面有 env -C 的算不出，shell_directory_option；串里那一条自己带 env -C 的，
        它自己的算不出，shell_text_runs），脚本正文从起它的那一条的当前目录起。
        弄坏开关 shell-string-writers-unread 下不切 -c 的串，script-body-writers-unread 下不读脚本正文（都只看命令那一层）；
        wrapped-writers-first-only 下认到头一条写的就停、后面的不交；wrapped-writer-paths-literal 下 expanded 照切出的原样、directory 一律是任务的工作目录。"""
        command = list(task["command"])
        given_cwd = task.get("cwd") or self.peer_root
        cwd = os.path.realpath(given_cwd)
        literal = lib.broken("wrapped-writer-paths-literal", self.environment)
        first_only = lib.broken("wrapped-writers-first-only", self.environment)
        found = []
        launch_index = lib.command_word_index(command)
        launched = [(command, given_cwd if literal or not shell_directory_option(command[:launch_index]) else None)]
        if not lib.broken("shell-string-writers-unread", self.environment):
            for index in sorted(shell_script_word_indexes(command)):
                start = given_cwd if literal or not shell_directory_option(command[:index]) else None
                for run in self.shell_runs(command[index], start, given_cwd, literal):
                    relative = self.repository_paths(run.words, cwd)
                    verdict = self.writer_command_verdict(run.words, relative)
                    if verdict or (verdict is False and any(path in WORKSPACE_WRITERS for path in relative)):
                        found.append((WorkspaceWriter(run.words, given_cwd, run.expanded, run.directory), bool(verdict)))
                        if verdict and first_only:
                            return found
                    if verdict is None:   # 只读、只喂的那一条起的脚本也不读正文
                        launched.append((run.words, run.directory))
        if lib.broken("script-body-writers-unread", self.environment):
            return found
        for words, directory in launched:
            for writer, writes in self.script_body_workspace_commands(words, cwd, given_cwd, directory, literal):
                found.append((writer, writes))
                if writes and first_only:
                    return found
        return found

    def shell_runs(self, text, directory, given_cwd, literal, opaque=False):
        """一段 shell 文字的每一条命令 → [ShellRun…]（shell_text_runs，从 directory 起，opaque 同它）；literal（弄坏开关 wrapped-writer-paths-literal）下词照原样、当前目录一律是 given_cwd。"""
        if literal:
            return [ShellRun(words, list(words), given_cwd) for words in shell_text_commands(text)]
        return shell_text_runs(text, directory, opaque, self.environment)

    def script_body_workspace_commands(self, words, cwd, given_cwd, directory, literal):
        """命令位置上起的脚本（command_word_index 剥过 env、bash、python3 这类之后那一个词：bash <脚本>、python3 <脚本>、直接起的可执行脚本）是本仓或工作根下的普通文件时，
        读它的正文（script_body_text），切出的每一条命令（shell_runs，从起它的那一条的当前目录 directory 起；python 正文的当前目录与变量一概算不出）
        剥掉开头的 SCRIPT_COMMAND_PREFIX_WORDS 与「名=值」，命令位置上的词（command_word_index）照 cwd 折成相对仓根是写工作区工具的，
        交 [(WorkspaceWriter（剥过之后那一条的词）, 写不写（writer_command_verdict：只读、只喂的不写）)…]；没有的交 []。
        只认命令位置：注释、引号包着的整句（echo "python3 …"）、别的命令的参数里提到的不算（门禁与合入后验证的正文里处处写着这几个工具的名字）；
        路径要字面写着，经变量拼的（"$root/scripts/…"、sys.executable 之后的常量）认不出；正文里再起的脚本不往下读；合入后验证（MERGE_AND_VERIFY_ENTRY）不读：它只打自己拷的快照。
        读不了的（不在、不是普通文件、不在本仓与工作根下、读出错、太大或是二进制，script_body_text 交 None）照不认。"""
        index = lib.command_word_index(words)
        if index >= len(words) or words[index].startswith("-"):
            return []
        script = os.path.realpath(os.path.join(cwd, words[index]))
        root = os.path.realpath(self.peer_root)
        # 套在假调度器上的格（object.__new__ 起、不经 __init__）没有 work_root：那几格只认仓里的
        roots = [root] + ([self.work_root] if getattr(self, "work_root", "") else [])
        if (not os.path.isfile(script) or os.path.relpath(script, root) == MERGE_AND_VERIFY_ENTRY
                or not any(lib.inside(script, each) for each in roots)):
            return []
        text = script_body_text(script) or ""
        python_body = (script.endswith(".py") or any(os.path.basename(word).startswith("python") for word in words[:index])
                       or (text.startswith("#!") and "python" in text.split("\n", 1)[0]))
        runs = self.shell_runs(text, directory, given_cwd, literal, opaque=python_body)
        found = []
        for run in runs:
            inner = run.words
            start = 0
            while start < len(inner) and (inner[start] in SCRIPT_COMMAND_PREFIX_WORDS or lib.ASSIGNMENT_WORD.match(inner[start])):
                start += 1
            spoken = inner[start:]
            position = lib.command_word_index(spoken)
            relative = self.repository_paths([spoken[position]], cwd) if position < len(spoken) else []
            if relative and relative[0] in WORKSPACE_WRITERS:
                found.append((WorkspaceWriter(spoken, given_cwd, run.expanded[start:], run.directory), bool(self.writer_command_verdict(spoken, relative))))
        return found

    def placement_refusal(self, command, cwd, environment):
        """提交时拒（文件头 ②「提交时拒」）：依次判 ① 共享根外的路径、② 共享根、③ 判定库、④ 临时 prune-server 包装里会编的、⑤ 不点名包的全仓编译，
        命中头一种交 {problem: 那一句, next_step: 出路}，都没命中交 None。
        environment 是这一条带进任务的环境（split_environment 收下的那几个）。五个弄坏开关各关一种。"""
        if not lib.broken("outside-path-accepted", self.environment):
            refusal = self.find_outside_shared_path(command, cwd)
            if refusal:
                return refusal
        if not lib.broken("shared-roots-outside-accepted", self.environment):
            refusal = self.shared_roots_problem(environment)
            if refusal:
                return refusal
        if not lib.broken("verdict-library-family-accepted", self.environment):
            refusal = self.verdict_library_refusal(command, cwd)
            if refusal:
                return refusal
        if not lib.broken("temporary-service-heavy-accepted", self.environment):
            refusal = self.temporary_service_heavy_refusal(command, cwd, environment)
            if refusal:
                return refusal
        if not lib.broken("whole-workspace-cargo-accepted", self.environment):
            return self.whole_workspace_cargo_refusal(command, cwd, environment)
        return None

    def manual_refusal(self, manual, session):
        """人手交的认法（异常表 manual-from-session、sessionless-unmarked）：会话照进程树读（session_of_peer），客户端说了不算。带 --manual 而进程树上有会话的拒，
        会话不靠它躲开推送；没有会话又没带 --manual 的拒，随手填的收件地址换不来进队。合的交 None。
        弄坏开关 manual-from-session-accepted、sessionless-unmarked-accepted 下各不判。"""
        if manual and session and not lib.broken("manual-from-session-accepted", self.environment):
            return {"problem": f"带了 --manual，而这条命令是会话 {session} 起的：这一条不收、不跑", "next_step": lib.anomaly_advice("manual-from-session", "", self.environment)}
        if not manual and not session and not lib.broken("sessionless-unmarked-accepted", self.environment):
            return {"problem": "这条命令不是从 Claude Code 会话起的，也没带 --manual：这一条不收、不跑", "next_step": lib.anomaly_advice("sessionless-unmarked", "", self.environment)}
        return None

    def full_gate_refusal(self, command, cwd, full_gate):
        """全量门禁标识（task submit --full-gate）只认提交时的门禁、不带 --check 与 --item、也不是只查不跑的写法（lib.commit_gate_entry，只查不跑照 lib.query_only_entry）：
        别的命令带它交 {problem, next_step}；没带或合的交 None。
        弄坏开关 full-gate-any-command-accepted 下不判。"""
        if not full_gate or lib.commit_gate_entry(list(command), cwd) or lib.broken("full-gate-any-command-accepted", self.environment):
            return None
        return {"problem": f"带了 --full-gate，而 {shlex.join(command)[:200]} 不是不指定范围的提交时门禁：这一条不收、不跑",
                "next_step": lib.anomaly_advice("full-gate-not-commit-entry", "", self.environment)}

    def whole_package_refusal(self, command, cwd, environment, confirmation_path, full_gate=False):
        """⑥ 整包测试与不指定范围的跑法没带确认标识（文件头 ②「提交时拒」⑥）：给了确认文件的照 lib.whole_package_confirmation_problem 判形式，不合交 {problem, next_step}；
        没给而命令是整包测试（cargo_runs 认的 cargo test 照 lib.whole_package_cargo_test，命令位置上的入口照 lib.whole_package_script）、
        或是不带选条参数的门禁与入口（lib.unscoped_run_entry），又没带全量门禁标识（full_gate，full_gate_refusal 先核过）的，交 {problem, next_step}；别的交 None。
        弄坏开关 whole-package-unconfirmed-accepted 下没给标识的不判。
        不指定范围的提交时门禁（lib.commit_gate_entry）没带全量门禁标识的先拒，整包确认标识顶替不了它（全量门禁必须带 --full-gate）；
        弄坏开关 commit-gate-without-full-gate-accepted 下不判。"""
        if (not full_gate and lib.commit_gate_entry(list(command), cwd)
                and not lib.broken("commit-gate-without-full-gate-accepted", self.environment)):
            return {"problem": f"不指定范围的提交时门禁 {shlex.join(command)[:200]} 没带 --full-gate：全量门禁都带它交（整包确认标识顶替不了），这一条不收、不跑",
                    "next_step": "照原命令加上全量标识重交：task-server/task submit --full-gate -- <命令>；只想跑一部分的，改成指定范围（--check <格>、--item <项>）"}
        if confirmation_path:
            problem = lib.whole_package_confirmation_problem(confirmation_path, self.containing_shared_root)
            return {"problem": f"{problem}：这一条不收、不跑", "next_step": lib.WHOLE_PACKAGE_NEXT_STEP} if problem else None
        if lib.broken("whole-package-unconfirmed-accepted", self.environment) or full_gate:
            return None
        reasons = [lib.whole_package_script(list(command), cwd, self.environment)]
        reasons += [lib.whole_package_cargo_test(run.rest) for run in self.cargo_runs({"command": list(command), "cwd": cwd, "environment": environment})
                    if run.subcommand == "test"]
        found = next((reason for reason in reasons if reason), "")
        if found:
            return {"problem": f"整包测试没带确认标识：{found}；日常开发的测试只点名改到的测试目标与测试函数，这一条不收、不跑", "next_step": lib.WHOLE_PACKAGE_NEXT_STEP}
        unscoped = lib.unscoped_run_entry(list(command), cwd, self.environment)
        if unscoped:
            return {"problem": f"不指定范围的跑法没带确认标识：{unscoped}；不指定范围的只在提交时跑，这一条不收、不跑", "next_step": lib.UNSCOPED_NEXT_STEP}
        return None

    def unnamed_run_refusal(self, command, cwd, environment, confirmation_path, full_gate=False):
        """⑦ 启动测试与验证没点名（文件头 ②「提交时拒」⑦）：提交的命令与 bash -c 里命令位置上的每一条（lib.shell_inner_commands）照 lib.unnamed_run_problem 判，
        头一处没点名的交 {problem, next_step}，都点了名交 None。带 --full-gate、带了 --whole-package-confirmed（⑥ 先核过形式）的照那个函数放行。
        自证的小世界里 lib.UNNAMED_RUN_SELFTEST_VARIABLE 写 off 的不判；弄坏开关 unnamed-run-refusal-skipped 下不判。"""
        if lib.broken("unnamed-run-refusal-skipped", self.environment) or lib.selftest_override(lib.UNNAMED_RUN_SELFTEST_VARIABLE, self.environment) == "off":
            return None
        for one in [list(command)] + lib.shell_inner_commands(list(command), self.environment):
            found = lib.unnamed_run_problem(one, cwd, full_gate, bool(confirmation_path), self.environment)
            if found:
                return {"problem": f"{found['problem']}：这一条不收、不跑", "next_step": found["next_step"]}
        return None

    def card_declaration_refusal(self, command, cwd, environment):
        """⑧ 入口文件头写了卡的声明（文件头 ②「提交时拒」⑧；异常表 card-declaration）：提交的命令与 bash -c 里命令位置上的每一条（lib.shell_inner_commands）
        起的脚本照 lib.card_declaration_problem 判，头一处写了的交 {problem, next_step}，都没写交 None。卡归 GPU 批处理验证服务，服务不认这几种声明、不分卡、不清场，
        带着它们的入口当场拒，不照没写那样收（.claude/agent-common.md「里程碑 17 收尾之前不兼容任何旧东西」）。弄坏开关在 lib.card_declaration_problem。"""
        for one in [list(command)] + lib.shell_inner_commands(list(command), self.environment):
            found = lib.card_declaration_problem(one, cwd, self.environment)
            if found:
                return found
        return None

    def copy_proof_roots(self, command, cwd, environment):
        """这条命令在哪些目录里编译（真路径；文件头 ②「提交时拒」⑨）：命令位置上会编的 cargo 起的目录（cd、-C、--manifest-path 之后的，认不出目录的算工作目录）；
        一条会编的 cargo 都认不出的（入口脚本里面才起 cargo、临时 prune-server 包装现编的、不编译的）就是工作目录。"""
        roots = [os.path.realpath(run.directory or cwd) for run in self.cargo_runs({"command": list(command), "cwd": cwd, "environment": environment})
                 if run.subcommand in self.cargo_lock.BUILDING_SUBCOMMANDS]
        return list(dict.fromkeys(roots)) or [os.path.realpath(cwd)]

    def copy_proof_active_tasks(self):
        """在排、在跑、待确认的任务（分片的子任务照父任务，不单列），按任务号。"""
        return [task for task, valid in self.store.select("tasks", "state IN ('queued', 'running', 'sharded', ?)", (lib.AWAITING_CONFIRMATION_STATE,), order="id")
                if valid and not task.get("parent_id")]

    def copy_proof_refusal(self, command, cwd, environment):
        """⑨ 证红进行中的副本里编译、有会编译的任务的副本上证红（文件头 ②「提交时拒」⑨；异常表 copy-under-proof）→ {problem, next_step} 或 None。
        提交的是证红（lib.prove_red_copies 点名了副本）：副本里已有会编译的任务在排、在跑、待确认的拒（copy_proof_start_refusal）；
        提交的是会编译的命令：它编译的目录落在某份副本里，而这份副本上留着证红的标记、或有证红任务在排在跑的拒（copy_proof_marker_refusal、copy_proof_build_refusal）；
        不编译的命令、别的副本里的都不判。只在交任务那一刻判（submit_in_thread），不进 submit_refusal：重核在排的任务时两条互相看见会一起打回。
        弄坏开关 copy-proof-start-accepted、copy-proof-build-accepted 下各不判一头，copy-proof-every-command-judged 下不看会不会编译，
        copy-proof-every-copy-judged 下不比副本（copy_proof_start_refusal、copy_proof_build_refusal 里）。"""
        copies = lib.prove_red_copies(command, cwd, self.environment)
        if copies:
            return None if lib.broken("copy-proof-start-accepted", self.environment) else self.copy_proof_start_refusal(copies)
        if lib.broken("copy-proof-build-accepted", self.environment):
            return None
        if not lib.broken("copy-proof-every-command-judged", self.environment) and not self.compiles({"command": list(command), "cwd": cwd, "environment": environment}):
            return None
        roots = self.copy_proof_roots(command, cwd, environment)
        return self.copy_proof_marker_refusal(roots) or self.copy_proof_build_refusal(roots)

    def copy_proof_marker_refusal(self, roots):
        """roots 里有一个往上走碰到证红的标记（lib.copy_proof_marker：还原日志、300 秒内刷新过的副本锁持有者记录）的，交 {problem, next_step}，没有交 None。"""
        for root in roots:
            found = lib.copy_proof_marker(root)
            if found is None:
                continue
            copy, name, meaning = found
            leftover = name == lib.COPY_PROOF_PENDING_NAME
            return {"problem": (f"副本 {copy} 上有证红进行中（{name}：{meaning}）：这一条会在这份副本里编译，编到的是打了变异的源码、不是这份副本本来的源码；这一条不收、不跑"),
                    "next_step": ((f"重跑上一趟被停的那条证红命令（scripts/mutation/prove-red.sh 开头照 {name} 把源码写回、删掉它），它出结局之后再交；" if leftover
                                   else "等那条证红出结局（task-server/task status 看是哪一条）再交；") + f"等不起的{COPY_PROOF_COPY_ADVICE}，在新副本里编")}
        return None

    def copy_proof_build_refusal(self, roots):
        """roots 里有一个落在某条在排、在跑、待确认的证红任务点名的副本里的，交 {problem, next_step}（点名那条证红任务），没有交 None。弄坏开关 copy-proof-every-copy-judged 下不比副本。"""
        every_copy = lib.broken("copy-proof-every-copy-judged", self.environment)
        for other in self.copy_proof_active_tasks():
            for copy in lib.prove_red_copies(other["command"], other["cwd"], self.environment):
                if not every_copy and not any(lib.inside_directory(root, copy) for root in roots):
                    continue
                return {"problem": (f"副本 {copy} 上有证红（任务 {other['id']}，{COPY_PROOF_STATE_TEXT.get(other.get('state'), other.get('state'))}）："
                                    "证红在这份副本里就地改源码再还原，这一条会在这份副本里编译，编到的是打了变异的源码；这一条不收、不跑"),
                        "next_step": f"等任务 {other['id']} 出结局（task-server/task wait {other['id']}）再交；等不起的{COPY_PROOF_COPY_ADVICE}，在新副本里编"}
        return None

    def copy_proof_start_refusal(self, copies):
        """提交的证红点名的副本 copies 里已有会编译的任务在排、在跑、待确认的，交 {problem, next_step}（点名那条任务），没有交 None；别的证红不算（副本锁自己拒）。
        弄坏开关 copy-proof-every-copy-judged 下不比副本，copy-proof-every-command-judged 下不看会不会编译。"""
        every_copy, every_command = lib.broken("copy-proof-every-copy-judged", self.environment), lib.broken("copy-proof-every-command-judged", self.environment)
        for other in self.copy_proof_active_tasks():
            if lib.prove_red_copies(other["command"], other["cwd"], self.environment) or not (every_command or other.get("compiles")):
                continue
            roots = self.copy_proof_roots(other["command"], other["cwd"], other.get("environment") or {})
            for copy in copies:
                if not every_copy and not any(lib.inside_directory(root, copy) for root in roots):
                    continue
                return {"problem": (f"副本 {copy} 里有会编译的任务 {other['id']}（{COPY_PROOF_STATE_TEXT.get(other.get('state'), other.get('state'))}，"
                                    f"命令 {shlex.join(other['command'])[:120]}）：证红在这份副本里就地改源码再还原，那条任务编到的是打了变异的源码；这一条证红不收、不跑"),
                        "next_step": f"等任务 {other['id']} 出结局（task-server/task wait {other['id']}）再交证红；等不起的{COPY_PROOF_COPY_ADVICE}，在新副本里证红"}
        return None

    def submit_refusal(self, command, cwd, environment, confirmation_path="", full_gate=False):
        """提交时拒的全部（文件头 ②「提交时拒」）：交任务那一刻（POST /tasks）与服务起来时重核在排的（revalidate_queued）都照这一份判，加一种提交时拒只改这里。
        environment 是交上来的环境（交任务那一刻是请求里的，重核时是任务带的那一份）。依次判：环境变量名单（split_environment）、套了包装或派发器（nested_problem）、
        编译目录指到仓里（target_directory_refusal）、①–⑤（placement_refusal）、⑥ 整包测试与不指定范围的跑法（whole_package_refusal）、
        ⑦ 启动测试与验证没点名（unnamed_run_refusal）、⑧ 入口文件头写了卡的声明（card_declaration_refusal），命中头一种就停。
        → (带进任务的环境, None 或 (回话 detail 的键, 那一项的值, 给人看的一句))。"""
        carried, refused = lib.split_environment(environment, self.environment)
        if refused:
            return carried, ("refused", [list(pair) for pair in refused], "环境变量 " + "、".join(name for name, _ in refused) + " 现在不收")
        nested = self.nested_problem(command, cwd)
        if nested:
            return carried, ("nested", nested, str(nested))
        target = lib.target_directory_refusal(carried, command, cwd, self.environment)
        if target:
            return carried, ("target", target, str(target))
        for refusal in (self.placement_refusal(command, cwd, carried), self.full_gate_refusal(command, cwd, full_gate),
                        self.whole_package_refusal(command, cwd, carried, confirmation_path, full_gate),
                        self.unnamed_run_refusal(command, cwd, carried, confirmation_path, full_gate),
                        self.card_declaration_refusal(command, cwd, carried)):
            if refusal:
                return carried, ("placement", refusal, f"{refusal['problem']}（出路：{refusal['next_step']}）")
        return carried, None

    def whole_workspace_cargo_refusal(self, command, cwd, environment):
        """⑤ 不点名包的全仓编译（文件头 ②「提交时拒」⑤）：命令位置上直接起的 WHOLE_WORKSPACE_SUBCOMMANDS 那几种 cargo（cargo_runs 认的，入口脚本里面的不读）
        带 --workspace / --all 的，或没带 -p / --package 而照 lib.cargo_manifest_scope 落到虚拟清单、起的目录认不出、清单读不了的，交 {problem, next_step}；
        都点了包、落到包自己的清单、往上没有清单的交 None。"""
        for run in self.cargo_runs({"command": list(command), "cwd": cwd, "environment": environment}):
            if run.subcommand not in WHOLE_WORKSPACE_SUBCOMMANDS:
                continue
            values = lib.cargo_option_values(run.rest)
            named = f"cargo {run.subcommand}"
            flags = [flag for flag in WHOLE_WORKSPACE_FLAGS if values.get(flag)]
            if flags:
                problem = f"{named} 带了 {'、'.join(flags)}：编整个工作区的每一个包"
            elif values["-p"]:
                continue
            elif run.directory is None:
                problem = f"{named} 没点名包（-p / --package），起的目录认不出，判不了它编不编整个工作区"
            else:
                manifest, kind, why = lib.cargo_manifest_scope(run.directory)
                if kind in (lib.MANIFEST_PACKAGE, lib.MANIFEST_NONE):
                    continue
                problem = (f"{named} 没点名包（-p / --package），在 {run.directory} 起，清单 {manifest} 读不了（{why}），判不了它编不编整个工作区"
                           if kind == lib.MANIFEST_UNREADABLE else
                           f"{named} 没点名包（-p / --package），在 {run.directory} 起、照工作区清单 {manifest} 编它的每一个成员")
            return {"problem": f"{problem}：不点名包的全仓编译一条就占一整台，这一条不收、不跑",
                    "next_step": WHOLE_WORKSPACE_NEXT_STEP}
        return None

    def temporary_service_heavy_refusal(self, command, cwd, environment):
        """④ 临时 prune-server 包装里会编的（文件头 ②「提交时拒」④）：剥不开（不是这层包装）的交 None；剥出的原命令照 compiles 有会编的 cargo 的，
        交 {problem, next_step}（点名原命令），不是交 None。剥开只用来判，收下的照旧整条执行。"""
        words = lib.unwrapped_temporary_service(command, self.environment)
        if words == list(command):
            return None
        if not self.compiles({"command": words, "cwd": cwd, "environment": environment}):
            return None
        return {"problem": (f"临时 prune-server 的包装 {lib.TEMPORARY_SERVICE_SCRIPT} {lib.TEMPORARY_SERVICE_FLAG} 里套的原命令有会编的 cargo："
                            "这层包装只给脚本自证用，服务只照外层那一条给编译目录，包装里的认不进来；这一条不收、不跑"),
                "next_step": (f"去掉包装重交：task-server/task submit -- {shlex.join(words)}"
                              "（会编的命令对真服务判，验证包的 cargo test 不套这层包装）")}

    def find_outside_shared_path(self, command, cwd):
        """① 共享根外的路径（文件头 ②「提交时拒」）：只认整个 shell 词是绝对路径的。命令的每个词：以 / 起头的整个词、「名字=绝对路径」「--选项=绝对路径」的值；
        shell 的 -c 脚本那一个词（shell_script_word_indexes）照 shell 词切开（shell_word_paths），每个以 / 起头的 shell 词与「名字=」的值；
        别的词中间夹着的 /（参数值里的「（接的是 /）」这类）不算。此刻在的、以 / 起头的整个词照整个词判；两台相同的系统路径（HOST_INDEPENDENT_PATHS）不判。
        相对路径照工作目录解，工作目录在共享根下它们也在。→ 头一个不在的 {problem, next_step}，都在交 None；落在 VOLATILE_ROOT 下的另说一句重启会清。
        紧跟在 ${…}、$(…) 后面的 /… 是展开之后的尾巴（编译目录这类运行时才给的），不当绝对路径；弄坏开关 expansion-tail-as-path 下也当。
        弄坏开关 quoted-word-paths-ignored 下不认词里的路径；path-check-substring 下每个带空白的词都按子串（QUOTED_WORD_PATH）认，词中间夹着的 / 也当路径。"""
        found = ""
        real_cwd = os.path.realpath(cwd)
        if not self.containing_shared_root(real_cwd):
            found, named = cwd, f"工作目录 {real_cwd}"
        else:
            scripts = shell_script_word_indexes(command)
            for index, word in enumerate(command):
                if word.startswith("/") and os.path.lexists(word):
                    candidates = [word]
                elif lib.broken("path-check-substring", self.environment) and any(character.isspace() for character in word):
                    candidates = QUOTED_WORD_PATH.findall(word)
                elif index in scripts:
                    candidates = ([] if lib.broken("quoted-word-paths-ignored", self.environment)
                                  else shell_word_paths(word, SHELL_WORD_PATH_WITH_EXPANSION_TAILS) if lib.broken("expansion-tail-as-path", self.environment)
                                  else shell_word_paths(word))
                else:
                    value = word if word.startswith("/") else word.partition("=")[2]
                    candidates = [value] if value.startswith("/") else []
                found = next((path for path in candidates
                              if not any(path == entry or (entry.endswith("/") and path.startswith(entry)) for entry in HOST_INDEPENDENT_PATHS)
                              and not self.containing_shared_root(os.path.realpath(path))), "")
                if found:
                    named = f"命令里点到的 {found}"
                    break
        if not found:
            return None
        roots = "、".join(self.shared_roots) or "一个根都没写"
        volatile = any(os.path.normpath(path) == VOLATILE_ROOT or os.path.normpath(path).startswith(VOLATILE_ROOT + os.sep) for path in (found, os.path.realpath(found)))
        lost = (f"；它落在 {VOLATILE_ROOT} 下：重启会清掉 {VOLATILE_ROOT}，任务做完了、重启之后那里的产物就没了" if volatile else "")
        return {"problem": (f"{named} 不在多机配置 {lib.NFS_ROOTS_KEY} 的任何一段下（{roots}）：任务两台都要跑得起，第二台照同一个绝对路径读写，"
                            f"读不到本机那一份、写出的本机也读不到{lost}；这一条不收、不跑"),
                "next_step": f"把它改放 {lib.NFS_ROOTS_KEY} 某一段下（{roots}；要留的产物放共享工作根 {self.work_root} 下），再提交"}

    def verdict_library_refusal(self, command, cwd):
        """③ 判定库（文件头 ②「提交时拒」）：带不带判定库只照登记表 .claude/gate.d/verdict-library-scripts.tsv 判（registry_families），不另问别的脚本：
        命令里（临时 prune-server 的包装剥开、照原命令认）有一个词点到登记表里带判定库的一行（第三列不以「不算：」「无判定库：」起头），而那一行第三列不以 prune-server 起头的，
        交 {problem, next_step}，点名那一族（登记的路径）；登记表读不了、写坏了的同样拒（判不了）。都不是交 None。
        词的认法（registry_family_of）：整个词，词里有空白的（bash -c 那一段）按空白拆开的每一段；照写的、照工作目录解开的、解开符号链接的，等于登记的路径或以「/登记的路径」收尾。
        弄坏开关 registry-service-column-ignored 下第三列写 prune-server 的也拒。"""
        families, problem = self.registry_families()
        if problem:
            return {"problem": f"判不了这条命令带不带判定库：{problem}；这一条不收、不跑",
                    "next_step": f"把登记表 {VERDICT_LIBRARY_REGISTRY} 修好（列的写法在它文件头），再提交"}
        words = lib.unwrapped_temporary_service(command, self.environment)
        found = self.registry_family_of(words, cwd, families)
        if not found:
            return None
        if families[found].startswith(REGISTRY_SERVICE_MARK) and not lib.broken("registry-service-column-ignored", self.environment):
            return None
        return {"problem": (f"这条命令带判定库，那一族是登记表 {VERDICT_LIBRARY_REGISTRY} 的 {found}，第三列还没写 {REGISTRY_SERVICE_MARK}："
                            "判定写在按台的库里，派到哪台就落在哪台，两台读不到同一份；这一条不收、不跑"),
                "next_step": (f"把 {found} 那一族的判定库读写接进 prune-server（.claude/rules/verification.md「判定库只有一份」），"
                              f"再把登记表那一行第三列翻成 {REGISTRY_SERVICE_MARK}，然后提交")}

    def registry_family_of(self, words, cwd, families):
        """words 里头一个点到登记表带判定库那一行的词：交登记的路径，没有交空串（认法见 verdict_library_refusal）。展开不了的词（带 $ 或反引号）不认。"""
        root = os.path.realpath(self.peer_root)
        library = [path for path, where in families.items() if not where.startswith(REGISTRY_NO_LIBRARY_PREFIXES)]
        for word in words:
            for piece in (word.split() if any(character.isspace() for character in word) else [word]):
                piece = piece.strip("'\"")
                if not piece or piece.startswith("-") or "$" in piece or "`" in piece:
                    continue
                piece = piece.partition("=")[2] if "=" in piece and not piece.startswith(("/", ".")) else piece
                if "/" not in piece:
                    continue
                resolved = os.path.normpath(os.path.join(cwd, os.path.expanduser(piece)))
                candidates = {os.path.normpath(piece), resolved, os.path.realpath(resolved)}
                for path in library:
                    registered = os.path.join(root, path)
                    if any(candidate in (path, registered, os.path.realpath(registered)) or candidate.endswith("/" + path) for candidate in candidates):
                        return path
        return ""

    def registry_families(self):
        """登记表 .claude/gate.d/verdict-library-scripts.tsv（相对仓根）：→ ({第一列的路径: 第三列（判定库放哪）}, 读不了或写坏了的那一句；读得了是空串)。
        第三列以「prune-server」起头的那一族判定库读写全在服务里、两台读的是同一份，提交时照收（判定库服务化的会话定的规矩）；以「不算：」「无判定库：」起头的没有判定库。
        表的修改时刻没变就用上一次读的。"""
        path = os.path.join(os.path.realpath(self.peer_root), VERDICT_LIBRARY_REGISTRY)
        try:
            stamp = os.stat(path).st_mtime_ns
        except OSError as error:
            return {}, f"登记表 {path} 读不了（{error}）"
        cached = getattr(self, "registry_cache", None)
        if cached and cached[0] == stamp:
            return cached[1], ""
        families = {}
        try:
            with open(path, encoding="utf-8") as handle:
                for number, line in enumerate(handle, 1):
                    if not line.strip() or line.startswith("#"):
                        continue
                    columns = line.rstrip("\n").split("\t")
                    if len(columns) < 3:
                        return {}, f"登记表 {path} 第 {number} 行不到三列"
                    families[columns[0].strip()] = columns[2].strip()
        except (OSError, UnicodeDecodeError) as error:
            return {}, f"登记表 {path} 读不了（{error}）"
        self.registry_cache = (stamp, families)
        return families, ""

    def containing_shared_root(self, real):
        """real（真路径）落在哪个 NFS 共享根下：交那个根，不在交空串（find_outside_shared_path 与 shared_roots_problem 共用）。"""
        return next((root for root in self.shared_roots if real == root or real.startswith(root.rstrip(os.sep) + os.sep)), "")

    def attempt_shared_roots(self, task):
        """这一次交给命令的两个共享根（task_server_lib.py 文件头「资源记录」后的「共享根」）：{变量名: 值}；任务自己带了的（白名单里的）照任务带的。"""
        values = {lib.DATA_ROOT_VARIABLE: self.data_root, lib.WORK_ROOT_VARIABLE: self.work_root}
        for name in lib.SHARED_ROOT_VARIABLES:
            if task["environment"].get(name):
                values[name] = task["environment"][name]
        return values

    def shared_roots_problem(self, environment):
        """② 共享根（文件头 ②「提交时拒」）：这一次要交给命令的两个共享根（attempt_shared_roots：服务自己的，任务带了的照任务带的）有不在 NFS_SHARED_ROOTS 某一段下的
        交 {problem, next_step}（点名变量与值），都在交 None。相对路径算不在（第二台照它自己的目录解）。"""
        outside = [f"{name}={value}" for name, value in self.attempt_shared_roots({"environment": environment}).items()
                   if not os.path.isabs(value) or not self.containing_shared_root(os.path.realpath(value))]
        if not outside:
            return None
        roots = "、".join(self.shared_roots) or "一个根都没写"
        return {"problem": (f"{'、'.join(outside)} 不在多机配置 {lib.NFS_ROOTS_KEY} 的任何一段下（{roots}）：命令在第二台经它写的，两台读不到同一份；"
                            "这一条不收、不跑"),
                "next_step": (f"配多机配置 {self.configuration.path}：{lib.DATA_ROOT_KEY} 写在 {lib.NFS_ROOTS_KEY} 某一段下、服务环境里的 {lib.WORK_ROOT_VARIABLE} "
                              f"也放在那下面（或往 {lib.NFS_ROOTS_KEY} 加上它所在的根），改好重起服务；是任务自己带的那个变量，就去掉它或改成共享根下的路径，再提交")}
