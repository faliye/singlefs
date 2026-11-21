"""task_server_lib.py 拆出的一份：签名、环境变量与命令词的拆分、套娃派发、对仓内目标的绕过、分片声明与分片计划、会话号（总说明在 task_server_lib.py 文件头）。"""
import os
import hashlib
import hmac
import json
import re
import secrets
import tomllib
from task_server_lib_constants import (
    AMBIENT_PREFIXES, AMBIENT_VARIABLES, ASSIGNMENT_WORD, CARD_DECLARATION_MARKS, CARD_DECLARATION_REFUSAL, CARRIED_PREFIXES, CARRIED_VARIABLES,
    COMPILES_DECLARED_WORD, DECLARED_COMPILES_MARK, DECLARED_HEADER_LINES, DECLARED_RESUMABLE_MARK, DECLARED_SHARDS_MARK, INSTALL_COMMAND,
    MAXIMUM_SHARDS, NOT_SHARDED_ARGUMENTS, REFUSED_PREFIXES, REPOSITORY_ROOT, RESOURCE_ARGUMENTS, RETIRED_VARIABLES, SECRET_PATTERN,
    SESSION_VARIABLE, SHARDED_ENTRIES, SHARD_PLAN_KEYS, TAKEN_OVER_PREFIXES, TAKEN_OVER_VARIABLES, TARGET_DIRECTORY_VARIABLES, WRAPPER_OWN_ARGUMENTS,
)
from task_server_lib_configuration import Unavailable, broken, command_word_index, ensure_private_directory, inside, inside_repository


def read_signing_key(path):
    """读签名密钥（64 位小写十六进制、属主自己、600、所在目录 700）；不对就抛 Unavailable。只有服务读它。"""
    try:
        status = os.stat(path)
    except FileNotFoundError:
        raise Unavailable(f"签名密钥 {path} 不在", f"{INSTALL_COMMAND}（它排他新建密钥，已在的不动）") from None
    except OSError as error:
        raise Unavailable(f"签名密钥 {path} 读不了（{error}）", "看它的属主与权限，修好再跑") from None
    if status.st_uid != os.getuid() or status.st_mode & 0o077:
        raise Unavailable(f"签名密钥 {path} 的属主不是自己或权限不是 600（{oct(status.st_mode & 0o777)}）", f"chmod 600 {path}（属主不对就删掉它重装）")
    if os.stat(os.path.dirname(path)).st_mode & 0o077:
        raise Unavailable(f"签名密钥所在目录 {os.path.dirname(path)} 的权限不是 700", f"chmod 700 {os.path.dirname(path)}")
    with open(path, encoding="utf-8") as handle:
        text = handle.read().strip()
    if not SECRET_PATTERN.fullmatch(text):
        raise Unavailable(f"签名密钥 {path} 写坏了（要 64 位小写十六进制）", "删掉它重装；密钥换了，库里旧的行都会核不过，要连库一起清掉")
    return bytes.fromhex(text)


def create_signing_key(path, environment=None):
    """排他新建签名密钥（32 字节随机数的十六进制，600）；已在就不动、交 False。弄坏开关 secrets-overwritten 下已在也重写。"""
    ensure_private_directory(os.path.dirname(path))
    if broken("secrets-overwritten", environment) and os.path.exists(path):
        os.remove(path)
    try:
        descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    except FileExistsError:
        return False
    with os.fdopen(descriptor, "w", encoding="utf-8") as handle:
        handle.write(secrets.token_hex(32) + "\n")
    return True


def row_signature(key, table, row_id, state, session, command_key, record_text):
    message = "\n".join([table, str(row_id), state or "", session or "", command_key or "", record_text])
    return hmac.new(key, message.encode("utf-8"), hashlib.sha256).hexdigest()


def canonical_json(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(",", ":"))


# ── 环境变量名单（文件头「环境变量」）──
def split_environment(environment, broken_environment=None):
    """→ (带进去的 {名: 值}, 拒的 [(名, 为什么)])；会话自带的两边都不进。"""
    carried, refused = {}, []
    for name in sorted(environment):
        if name in AMBIENT_VARIABLES or name.startswith(AMBIENT_PREFIXES):
            continue
        if name in RETIRED_VARIABLES:   # 弄坏开关 retired-variables-carried：照改前带进去
            if broken("retired-variables-carried", broken_environment):
                carried[name] = environment[name]
            else:
                refused.append((name, RETIRED_VARIABLES[name]))
            continue
        if name in CARRIED_VARIABLES or name.startswith(CARRIED_PREFIXES):
            carried[name] = environment[name]
            continue
        if name in TAKEN_OVER_VARIABLES or name.startswith(TAKEN_OVER_PREFIXES):
            if broken("queue-variables-carried", broken_environment):
                carried[name] = environment[name]
            else:
                refused.append((name, TAKEN_OVER_VARIABLES.get(name, "排队、记账与挑机器归 task-server，内存包装与派发器的这一类参数不收")))
            continue
        if broken("environment-unfiltered", broken_environment):
            carried[name] = environment[name]
            continue
        if name.endswith("_BREAK"):
            refused.append((name, "弄坏开关只在各脚本自己的 --selftest 里认"))
            continue
        refused.append((name, "白名单外：不知道它该不该带进任务"))
    return carried, refused


def environment_refusal_lines(refused):
    lines = [f"  ✗ 这几个环境变量 task-server 不收（不静默丢，当场拒）：" + "、".join(name for name, _ in refused)]
    lines += [f"      {name}：{why}" for name, why in refused]
    retired = [name for name, _ in refused if name in RETIRED_VARIABLES]
    lines.append("  → 怎么办：unset 它们再提交；命令真要它的，写进命令里：task-server/task submit -- env <名>=<值> <命令…>（原样跑）；"
                 "该常带的，往 task-server/task_server_lib.py 的 CARRIED_VARIABLES 里加"
                 + (f"；{'、'.join(retired)} 是退役的名字，命令里也不写（照上面它那一行的出路办）" if retired else ""))
    return lines


def retired_assignment(argv):
    """命令外层写了退役变量的「名=值」（env 后面的，与命令位置前的裸赋值；文件头「环境变量」）：交那个词；没有交空串。"""
    index = command_word_index(argv)
    words = list(argv[:index])
    while index < len(argv) and ASSIGNMENT_WORD.match(argv[index]):
        words.append(argv[index])
        index += 1
    return next((word for word in words if "=" in word and word.split("=", 1)[0] in RETIRED_VARIABLES), "")


def nested_dispatch(argv, environment=None):
    """命令自己在命令位置上套了内存包装、派发器或 multi-host-run.sh 的跑法，或在外层写了退役变量：交说明；没有交空串（剥法见 command_word_index）。"""
    if not broken("retired-variable-assignments-accepted", environment):
        retired = retired_assignment(argv)
        if retired:
            return f"{retired}：{RETIRED_VARIABLES[retired.split('=', 1)[0]]}（去掉这个赋值）"
    if broken("nested-dispatch-allowed", environment):
        return ""
    index = command_word_index(argv)
    if index >= len(argv):
        return ""
    name = os.path.basename(argv[index])
    rest = argv[index + 1:]
    if not broken("wrapper-prefixes-allowed", environment):
        for word in argv[:index]:
            if os.path.basename(word) in REFUSED_PREFIXES:
                return f"{word} …：{REFUSED_PREFIXES[os.path.basename(word)]}由服务定，命令里不套它（去掉这一层）"
            if word.startswith("SINGLEFS_THREAD_CAP="):
                return f"{word}：线程一律是登记的每台默认重活线程数，命令里不写它（去掉这个赋值）"
        if name in REFUSED_PREFIXES and rest not in WRAPPER_OWN_ARGUMENTS:
            return f"{argv[index]} …：{REFUSED_PREFIXES[name]}由服务定，命令里不套它（去掉这一层）"
    if not broken("resource-arguments-allowed", environment):
        for word in rest:
            if word in RESOURCE_ARGUMENTS or word.split("=", 1)[0] in RESOURCE_ARGUMENTS and "=" in word:
                return f"{word}：上限、线程与挑哪台由服务定，命令里不写它（去掉这个参数）"
    if name == "run-with-memory-cap.sh" and rest and (re.fullmatch(r"[1-9][0-9]*[KMGT]", rest[0]) or rest[0] == "auto"):
        return f"{argv[index]} {rest[0]} …：内存上限由服务定，命令里不套内存包装"
    if name in ("host-dispatch.py", "multi-host-run.sh") and "--" in rest:
        return f"{argv[index]} … -- …：挑机器、派第二台由服务定，命令里不套派发器"
    return ""


# ── 编译目录（文件头「编译目录」）──
def given_target_directories(environment, argv, cwd):
    """调用方给的编译目录：→ [(从哪来, 绝对路径)]：环境里的两个变量、命令词里的 CARGO_TARGET_DIR=… 赋值与 --target-dir。"""
    found = []
    for name in TARGET_DIRECTORY_VARIABLES:
        if environment.get(name):
            found.append((f"环境里的 {name}", environment[name]))
    for index, word in enumerate(argv):
        for name in TARGET_DIRECTORY_VARIABLES:
            if word.startswith(name + "="):
                found.append((f"命令里的 {name}=", word[len(name) + 1:]))
        if word.startswith("--target-dir="):
            found.append(("命令里的 --target-dir", word[len("--target-dir="):]))
        elif word == "--target-dir" and index + 1 < len(argv):
            found.append(("命令里的 --target-dir", argv[index + 1]))
    return [(source, os.path.normpath(os.path.join(cwd, os.path.expanduser(value)))) for source, value in found if value]


def target_directory_refusal(environment, argv, cwd, broken_environment=None):
    """调用方给的编译目录落在仓里：交说明；没给或在仓外交空串。弄坏开关 in-repository-target-accepted 下一律交空串。"""
    if broken("in-repository-target-accepted", broken_environment):
        return ""
    for source, path in given_target_directories(environment, argv, cwd):
        root = inside_repository(path)
        if root:
            return f"{source}指到仓里（{path}，在 {root} 里）：编译目录不许长在仓里"
    return ""


# ── 取仓根 target/ 下的二进制（task_server.py 文件头 ②「提交时警告」）──
REPOSITORY_TARGET_NAME = "target"
BYPASS_SCRIPT_MAXIMUM_BYTES = 4 * 1024 * 1024   # 读入口脚本正文只读这么大的（推的：本仓的脚本都远小于它，读不全的不判）
BYPASS_HITS_SHOWN = 3   # 警告那一句至多点名几处
# 词里的路径段：引号、命令分隔、重定向与「名=」的等号都切开
BYPASS_WORD_PATH = re.compile(r"[^\s'\"`;|&()<>=]+")
# 入口脚本正文里指向仓根 target/ 的路径字面：前面接仓根变量（shell 的 $<名>、${<名>}、${<名>:-…}，python f 串的 {<名>}；名里带 repo、root 或 toplevel）、
# $(dirname …) 之后往上跳的 /..、$(git rev-parse --show-toplevel)，接 /target/release/ 或 /target/debug/（${CARGO_TARGET_DIR:-$<仓根>/target}/release/ 这种退回仓根的同样算）
ROOTED_TARGET_LITERAL = re.compile(
    r"(?:\$\{?\w*(?:repo|root|toplevel)\w*\}?|\{\w*(?:repo|root|toplevel)\w*\}|dirname[^\n]*?\)\"?(?:/\.\.)+|\$\(git rev-parse --show-toplevel\))"
    r"/target\}?/(?:release|debug)/", re.IGNORECASE)
# 在仓根起的命令，正文里裸写的 target/release/、target/debug/：赋值的值（NAME=、python 的 name = ）、缺省值（${…:-target/…}）与命令位置（行首不带引号）
BARE_TARGET_LITERAL = re.compile(r"(?:^\s*|\w\s*=\s*[\"']?|:-[\"']?)(?:\./)?target/(?:release|debug)/")
# 正文里不读的行：注释、只在弄坏开关下走的那一支（break_is、*_BREAK、broken(）
BYPASS_SKIPPED_LINE = re.compile(r"^\s*(?:#|//)|\bbreak_is\b|_BREAK\b|\bbroken\(")
# 自证那一支：python 名字里带 selftest 或以 cells 收尾的函数体、shell 名字里带 selftest 的函数体（样本里造的假路径，如 memory_peaks_lib.py 的 key_fold_cells）
PYTHON_SELFTEST_DEFINITION = re.compile(r"^(\s*)def\s+(\w*selftest\w*|\w*cells)\s*\(")
SHELL_SELFTEST_DEFINITION = re.compile(r"^\s*(?:function\s+)?[\w-]*selftest[\w-]*\s*\(\)")


def script_target_literals(script_path, bare_counts):
    """入口脚本正文里指向仓根 target/ 的路径字面（ROOTED_TARGET_LITERAL；bare_counts 为真时另认 BARE_TARGET_LITERAL）：→ [(行号, 那一段字面)]。
    注释、弄坏开关那一支（BYPASS_SKIPPED_LINE）与自证那一支的函数体不读；读不了、不是普通文件、超过 BYPASS_SCRIPT_MAXIMUM_BYTES 的交 []。"""
    try:
        if not os.path.isfile(script_path) or os.path.getsize(script_path) > BYPASS_SCRIPT_MAXIMUM_BYTES:
            return []
        with open(script_path, encoding="utf-8", errors="replace") as handle:
            lines = handle.read().splitlines()
    except OSError:
        return []
    found, python_indent, in_shell_selftest = [], None, False
    for number, line in enumerate(lines, 1):
        if in_shell_selftest:
            in_shell_selftest = not line.startswith("}")
            continue
        if python_indent is not None:
            if not line.strip() or len(line) - len(line.lstrip()) > python_indent:
                continue
            python_indent = None
        definition = PYTHON_SELFTEST_DEFINITION.match(line)
        if definition:
            python_indent = len(definition.group(1))
            continue
        if SHELL_SELFTEST_DEFINITION.match(line):
            in_shell_selftest = not line.rstrip().endswith("}")
            continue
        if BYPASS_SKIPPED_LINE.search(line):
            continue
        match = ROOTED_TARGET_LITERAL.search(line) or (BARE_TARGET_LITERAL.search(line) if bare_counts else None)
        if match:
            found.append((number, match.group(0).strip()))
    return found


def repository_target_references(argv, cwd, roots, environment):
    """这一条命令取仓根 target/ 下的东西（task_server.py 文件头 ②「提交时警告」）：→ [那一处的说明…]，没有交 []。
    roots 是「仓根」那几棵树（服务的仓根与主工作树；草稿与临时目录下的仓副本不在里面）。认两样：
    ① 命令里的词（整个词、「名=值」的值、bash -c 词里的每一段；相对路径照工作目录解）落在某个仓根的 target/ 下；调用方给的编译目录（given_target_directories）下的不算；
    ② 命令位置上起的本仓脚本（command_word_index 剥过之后那一个词；不是 -c）正文里有指向仓根 target/ 的路径字面（script_target_literals；工作目录就是仓根时另认裸写的）。
    带 --selftest 起的脚本走自证那一支（样本里的假路径），不读正文。"""
    targets = [os.path.join(os.path.realpath(root), REPOSITORY_TARGET_NAME) for root in roots]
    given = [path for _, path in given_target_directories(environment, argv, cwd)]
    found = []
    for word in argv:
        for piece in BYPASS_WORD_PATH.findall(word):
            if REPOSITORY_TARGET_NAME not in piece or "$" in piece:
                continue
            path = os.path.normpath(os.path.join(cwd, piece))
            if (any(inside(path, target) and os.path.realpath(path) != target for target in targets)
                    and not any(inside(path, directory) for directory in given)):
                found.append(f"命令里的 {piece}（{path}）")
                break
    index = command_word_index(argv)
    if index < len(argv) and argv[index] != "-c" and "--selftest" not in argv[index + 1:]:
        script = os.path.realpath(os.path.join(cwd, argv[index]))
        root = next((os.path.realpath(root) for root in roots if inside(script, root)), "")
        if root and not inside(script, os.path.join(root, REPOSITORY_TARGET_NAME)):
            bare = os.path.realpath(cwd) in (os.path.realpath(each) for each in roots)
            for number, literal in script_target_literals(script, bare):
                found.append(f"入口脚本 {os.path.relpath(script, root)}:{number} 的 {literal}")
    return found


# ── 分片（文件头「分片」）──
def declared_header_words(script_path, mark):
    """脚本文件头前 DECLARED_HEADER_LINES 行里以 mark 起头的那一行写的词（分片、会编、续跑与卡的声明共用）；文件读不了、不是普通文件或没写交 None。"""
    if not os.path.isfile(script_path):
        return None
    try:
        with open(script_path, encoding="utf-8", errors="replace") as handle:
            for _ in range(DECLARED_HEADER_LINES):
                line = handle.readline()
                if not line:
                    break
                if line.startswith(mark):
                    return tuple(line[len(mark):].split())
    except OSError:
        return None
    return None


def declared_shards(script_path):
    """「# task-server-shards: …」写的第一个参数（* 是都分）；没写交 None。"""
    return declared_header_words(script_path, DECLARED_SHARDS_MARK)


def first_non_option(arguments):
    """参数里第一个不以 - 起头的词；没有交空串。"""
    return next((argument for argument in arguments if not argument.startswith("-")), "")


def declaration_matches(arguments, declared):
    """续跑声明的判法：声明了 * 就是任何参数都算，不然第一个非选项参数（first_non_option）要是声明的词之一。"""
    if not declared:
        return False
    return "*" in declared or first_non_option(arguments) in declared


def declared_compiles(script_path, environment=None):
    """命令位置上起的脚本声明了会编没有（文件头「编译目录」的入口声明）：→ (声明了没有, 写坏的那一句)。只认「# task-server-compiles: yes」；写了别的不猜，
    交那一句（服务照 run-condition-unmet 打回）。文件不在、没有那一行交 (False, "")。弄坏开关 compiles-declaration-ignored 下一律当没声明。"""
    if broken("compiles-declaration-ignored", environment):
        return False, ""
    declared = declared_header_words(script_path, DECLARED_COMPILES_MARK)
    if declared is None:
        return False, ""
    if declared == (COMPILES_DECLARED_WORD,):
        return True, ""
    return False, f"{script_path} 文件头的会编声明写成「{' '.join(declared)}」，要写 {COMPILES_DECLARED_WORD}"


def card_declaration_problem(argv, cwd, environment=None):
    """命令位置上起的脚本（command_word_index 剥过之后那一个词，照工作目录解成真路径）文件头前 DECLARED_HEADER_LINES 行里写了 CARD_DECLARATION_MARKS 之一
    （task_server.py 文件头 ②「提交时拒」⑧；异常表 card-declaration）→ {problem, next_step}（点名那份脚本与写了哪几种）；没写、文件不在、不是普通文件交 None。
    只查不跑的写法不另放行：声明写在文件头，跟这一次怎么起无关。弄坏开关 card-declarations-accepted 下一律交 None。"""
    if broken("card-declarations-accepted", environment):
        return None
    index = command_word_index(argv)
    if index >= len(argv):
        return None
    script = os.path.realpath(os.path.join(cwd or REPOSITORY_ROOT, argv[index]))
    found = [mark for mark in CARD_DECLARATION_MARKS if declared_header_words(script, mark) is not None]
    if not found:
        return None
    return {"problem": f"{CARD_DECLARATION_REFUSAL}：{script} 文件头写了 {'、'.join(found)}，服务不认领卡、要卡的声明，这一条不收、不跑",
            "next_step": (f"删掉那份脚本文件头的 {'、'.join(found)} 那一行，脚本里要卡的那一段改成往判定库写待判行、交 GPU 批处理验证服务判"
                          "（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」），再照原命令重交")}


def enclosing_user_unit(cgroup_text):
    """进程的 cgroup（/proc/<进程号>/cgroup 里 cgroup v2 那一行「0::<路径>」）→ 它所在的用户级 systemd 单元名：路径里 user@<uid>.service 之下、离进程最近的那一段
    .service 或 .scope（task_server.py 文件头 ③「套在外层单元里的服务」）。不在用户级管理器之下、或认不出的交空串。"""
    for line in (cgroup_text or "").splitlines():
        if not line.startswith("0::"):
            continue
        parts = [part for part in line[3:].strip().split("/") if part]
        manager = next((index for index, part in enumerate(parts) if part.startswith("user@") and part.endswith(".service")), None)
        if manager is None:
            return ""
        return next((part for part in reversed(parts[manager + 1:]) if part.endswith((".service", ".scope"))), "")
    return ""


def cargo_option_values(arguments):
    """cargo 子命令后面、`--` 之前的参数里认这几样：→ {"-p": [包…], "--features": [特性…], "--profile": 档, "--target": 目标, "--test": [测试目标…], 别的开关: True}。
    -p / --package、-F / --features、--profile、--target、--test 认「选项 值」「--选项=值」「-p值」三种写法；--release（-r）记成 --profile release。"""
    found = {"-p": [], "--features": [], "--test": [], "--profile": "dev", "--target": ""}
    aliases = {"--package": "-p", "-F": "--features"}
    index = 0
    while index < len(arguments) and arguments[index] != "--":
        word = arguments[index]
        name, value = word, None
        if "=" in word and word.startswith("--"):
            name, value = word.split("=", 1)
        elif word[:2] in ("-p", "-F") and len(word) > 2:
            name, value = word[:2], word[2:]
        name = aliases.get(name, name)
        if name in ("-p", "--features", "--test", "--profile", "--target"):
            if value is None:
                index += 1
                value = arguments[index] if index < len(arguments) else ""
            if name == "--features":
                found[name] += [part for part in re.split(r"[,\s]+", value) if part]
            elif name in ("-p", "--test"):
                found[name].append(value)
            else:
                found[name] = value
        elif word in ("--release", "-r"):
            found["--profile"] = "release"
        elif word.startswith("-"):
            found[word] = True
        index += 1
    return found


MANIFEST_VIRTUAL = "virtual"          # cargo_manifest_scope：只有 [workspace]、没有 [package] 的清单，不点名包就编每一个成员
MANIFEST_PACKAGE = "package"          # 包自己的清单：不点名包只编这一个包
MANIFEST_NONE = "none"                # 往上一份 Cargo.toml 都没有：cargo 自己就报错
MANIFEST_UNREADABLE = "unreadable"    # 有清单而读不了、不是合法的 TOML


def cargo_manifest_scope(directory):
    """cargo 在 directory 起、不点名包时编的范围（task_server.py 文件头 ②「提交时拒」⑤）：照 cargo 找清单的走法，directory 起往上第一份 Cargo.toml。
    → (那份清单的路径或 None, MANIFEST_VIRTUAL / MANIFEST_PACKAGE / MANIFEST_NONE / MANIFEST_UNREADABLE 之一, 读不了的原因或空串)。"""
    current = os.path.abspath(directory)
    while True:
        manifest = os.path.join(current, "Cargo.toml")
        if os.path.isfile(manifest):
            try:
                with open(manifest, "rb") as handle:
                    parsed = tomllib.load(handle)
            except (OSError, tomllib.TOMLDecodeError) as error:
                return manifest, MANIFEST_UNREADABLE, f"{type(error).__name__}: {error}"
            return manifest, MANIFEST_VIRTUAL if "workspace" in parsed and "package" not in parsed else MANIFEST_PACKAGE, ""
        parent = os.path.dirname(current)
        if parent == current:
            return None, MANIFEST_NONE, ""
        current = parent


def resumable_command(argv, cwd, environment=None):
    """这条命令登记了续跑没有（文件头「续跑」）：命令位置上起的脚本（剥掉 env、nice、timeout、setsid、bash / sh / python3、capped.sh N 之后，同 sharded_entry）
    文件头写了「# task-server-resumable: <词>…」、第一个非选项参数是其中一个（或写了 *）。弄坏开关 resumable-declaration-ignored 下一律当没登记，
    resume-declaration-any-word 下非选项参数有一个是声明的词就算（不只看第一个）。"""
    if broken("resumable-declaration-ignored", environment):
        return False
    index = command_word_index(argv)
    if index >= len(argv):
        return False
    script = os.path.realpath(os.path.join(cwd or REPOSITORY_ROOT, argv[index]))
    declared = declared_header_words(script, DECLARED_RESUMABLE_MARK)
    if broken("resume-declaration-any-word", environment):
        return bool(declared) and ("*" in declared or any(argument in declared for argument in argv[index + 1:] if not argument.startswith("-")))
    return declaration_matches(argv[index + 1:], declared)


def first_argument_matches(arguments, accepted, environment=None):
    first = arguments[0] if arguments else ""
    unsharded = first in NOT_SHARDED_ARGUMENTS or any(argument in ("--selftest",) for argument in arguments)
    if unsharded and not broken("unsharded-arguments-sharded", environment):
        return False
    if accepted is None or "*" in accepted:
        return True
    return first in accepted


def sharded_entry(argv, cwd, environment=None):
    """这件要不要分片：交起的那份脚本（仓根下的相对路径，或声明了分片的脚本的绝对路径）；不分交空串。弄坏开关 sharding-unrecognized 下一律交空串。"""
    if broken("sharding-unrecognized", environment):
        return ""
    index = command_word_index(argv)
    if index >= len(argv):
        return ""
    word, arguments = argv[index], argv[index + 1:]
    script = os.path.realpath(os.path.join(cwd, word))
    for relative, accepted in SHARDED_ENTRIES.items():
        if (script.endswith("/" + relative) or word == relative) and first_argument_matches(arguments, accepted, environment):
            return relative
    declared = declared_shards(script) if os.path.isfile(script) else None
    if declared and first_argument_matches(arguments, declared, environment):
        return script
    return ""










def over_run_hard_limit(seconds, limit, environment=None):
    """累计用时到没到四小时硬上限（task_server.py 文件头 ⑦「四小时硬上限」）：恰好等于上限算到。弄坏开关 run-hard-limit-exclusive 下等于上限不算，run-hard-limit-loose 下比上限少一秒也算。"""
    if broken("run-hard-limit-exclusive", environment):
        return seconds > limit
    return seconds >= limit - (1 if broken("run-hard-limit-loose", environment) else 0)


def covered_seconds(intervals, environment=None):
    """几段时段 [(起, 止)…] 的并集有多长（秒）：分片的父任务整件的挂钟时长（task_server.py 的 parent_run_hard_limit，只在弄坏开关 shard-whole-by-wall-clock 下算），同时在跑的不重复算，都不在跑的空档不算。
    弄坏开关 run-hard-limit-parent-summed 下取各段之和。"""
    if broken("run-hard-limit-parent-summed", environment):
        return sum(max(0.0, end - start) for start, end in intervals)
    total, reached = 0.0, None
    for start, end in sorted(intervals):
        start = start if reached is None else max(start, reached)
        if end > start:
            total += end - start
            reached = end
    return total








def shard_child_key(base_key_text, role, index, count, environment=None, compiles=False):
    """子任务的命令键原文（交给内存包装的 RUN_WITH_MEMORY_CAP_KEY）：这件的键加「〈task-server 分片 <步> 第<片号>片〉」，只带片号、不带片数：
    计划切出来的片数变了（2 片、64 片），同一片号还是这把键，但读记录时只认片数相同的那几趟（task_server.py 的 record_fits_task；相当的记录一条都没有的片照没有记录的命令定，同一趟的兄弟片与别的键都不借；文件头「资源记录」）。
    计划那一步的键带「这一步编不编」（compiles：服务给不给这一步编译目录，task_server.py 的 child_compiles）：「〈task-server 分片 plan 编〉」或「〈task-server 分片 plan 不编〉」，
    编与不编的计划步各记各的（计划这一步预编测试二进制时的峰值比只算键、核账的大得多，不编的记录不当编的那一种的起点）。
    加的那一段不带 /：键的换算（memory_peaks_lib.py 文件头「键」③）把带 / 的位置参数折成〈文件〉，带了各片就落一个键。
    弄坏开关 shard-key-position-folded 下写成 <片号>/<片数>，shard-key-count-kept 下写成「第<片号>片共<片数>片」（片数一变各片都换新键），
    plan-key-compile-unmarked 下计划那一步不带编不编（编与不编落同一把键）。"""
    if role == "plan" and not broken("plan-key-compile-unmarked", environment):
        position = " 编" if compiles else " 不编"
    elif role != "shard":
        position = ""
    elif broken("shard-key-position-folded", environment):
        position = f" {index}/{count}"
    elif broken("shard-key-count-kept", environment):
        position = f" 第{index}片共{count}片"
    else:
        position = f" 第{index}片"
    return f"{base_key_text} 〈task-server 分片 {role}{position}〉"


def read_shard_plan(path):
    """读计划（文件头「分片」①）：→ ([{"label"}…], 问题)；读得通问题是空串。写了 pin 的拒（片不钉台，哪台由服务挑）；弄坏开关 host-lists-restored 下照改前收
    pin（空串与 local，读了不用）。"""
    restored = broken("host-lists-restored")
    try:
        with open(path, encoding="utf-8") as handle:
            value = json.load(handle)
    except FileNotFoundError:
        return [], f"计划那一步退 0 而没写 {path}"
    except (OSError, ValueError) as error:
        return [], f"{path} 读不通（{type(error).__name__}: {error}）"
    shards = value.get("shards") if isinstance(value, dict) else None
    if not isinstance(shards, list):
        return [], f"{path} 里没有 shards 列表"
    if len(shards) > MAXIMUM_SHARDS:
        return [], f"{path} 写了 {len(shards)} 片，多于 {MAXIMUM_SHARDS} 片"
    parsed = []
    for number, shard in enumerate(shards):
        if isinstance(shard, dict) and "pin" in shard and not restored:
            return [], f"{path} 第 {number} 片写了 pin（{shard['pin']!r}）：片不钉台，哪台由服务挑；计划里去掉 pin"
        if restored and isinstance(shard, dict) and (shard.get("pin") or "") in ("", "local"):
            shard = {name: value for name, value in shard.items() if name != "pin"}
        if not isinstance(shard, dict) or set(shard) - set(SHARD_PLAN_KEYS):
            return [], f"{path} 第 {number} 片不是只带 {'、'.join(SHARD_PLAN_KEYS)} 的对象"
        if not isinstance(shard.get("label", ""), str):
            return [], f"{path} 第 {number} 片写坏了（label 是一句话）"
        parsed.append({"label": shard.get("label", "")})
    return parsed, ""


# ── 身份（文件头「身份」）──
def process_parent(process_id):
    try:
        with open(f"/proc/{process_id}/stat", encoding="utf-8") as handle:
            return int(handle.read().rsplit(")", 1)[1].split()[1])
    except (OSError, IndexError, ValueError):
        return 0


def process_environment(process_id):
    try:
        with open(f"/proc/{process_id}/environ", "rb") as handle:
            raw = handle.read()
    except OSError:
        return {}
    pairs = (item.split(b"=", 1) for item in raw.split(b"\0") if b"=" in item)
    return {name.decode("utf-8", "replace"): value.decode("utf-8", "replace") for name, value in pairs}


def session_of_peer(process_id, boundary_process_id=1, environment=None):
    """调用进程的会话号：从它的父进程起往上，各祖先 /proc 环境里的 CLAUDE_CODE_SESSION_ID 取最外层的那一个（不含调用进程自己；到 init 或 boundary 为止，不含它）。
    读不到交空串。弄坏开关 session-from-client 下取调用进程自己环境里的。"""
    if broken("session-from-client", environment):
        return process_environment(process_id).get(SESSION_VARIABLE, "")
    found, seen = "", set()
    current = process_parent(process_id)
    while current > 1 and current != boundary_process_id and current not in seen:
        seen.add(current)
        value = process_environment(current).get(SESSION_VARIABLE, "")
        if value:
            found = value
        current = process_parent(current)
    return found
