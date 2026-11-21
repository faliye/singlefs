"""task_server_lib.py 拆出的一份：自证开关的读法（broken）、cargo 与选条判法、多机配置、数据根、NFS 共享、ssh 复用、读 GPU 服务的报告（总说明在 task_server_lib.py 文件头）。"""
import os
import sys
import contextlib
import importlib.util
import re
import secrets
import shlex
import stat
import subprocess
import time
from dataclasses import dataclass
from task_server_lib_constants import (
    ADDRESSES_LIBRARY, ASSIGNMENT_WORD, BREAK_VARIABLE, BY_HOST_FORWARDED_BREAKS, CAP_GROWTH_FACTOR, CAP_HIT_REAL_PEAK_FRACTION, CONFIGURATION_STEP,
    DATABASE_NAME, DATA_ROOT_KEY, EXPANDED_VARIABLES, GPU_BLOCK_CLAIMS_KEPT, GPU_BLOCK_CLAIMS_LET_GO, GPU_BLOCK_CLAIM_FIELD,
    GPU_BLOCK_NOT_JUDGEABLE_ALERT_KIND, GPU_BLOCK_NOT_JUDGEABLE_KEY_FIELDS, GPU_BLOCK_NOT_JUDGEABLE_OF_A_HOST_PREFIX, GPU_BLOCK_NUMBERS_SHOWN,
    GPU_CARDS_FAILED_READ_PREFIX, GPU_CARDS_FAILED_READ_STATES, GPU_CARDS_FAILED_UNREAD_PREFIX, GPU_CARD_FAILED_ALERT_KIND, GPU_CARD_FIELD,
    GPU_FAILED_CARD_OF_A_HOST_PREFIX, GPU_THIS_HOST_LABEL, HOOKS_DIRECTORY, HOSTS_KEY, INSTALL_COMMAND, INSTALL_PACKAGES, IO_PRESSURE_LIMIT_KEY,
    IO_PRESSURE_SELFTEST_DEFAULT, KEY_NEUTRAL_VARIABLES, MULTI_HOST_ADDRESSES_SCRIPT, MULTI_HOST_CONFIG_NAME, MULTI_HOST_CONFIG_VARIABLE,
    NFS_EXPORTS_FILE, NFS_FILESYSTEM_TYPES, NFS_ITEMS, NFS_MOUNT_PROBE_TEXT, NFS_MOUNT_RECOVERY, NFS_PASSED, NFS_PEER_PROBE_NAME, NFS_PROBE_NAME,
    NFS_PROBE_READ_TRIES, NFS_README, NFS_RECHECK_PAUSE_SECONDS, NFS_ROOTS_KEY, NFS_ROOTS_MISSING_TEXT, NFS_SLOW_GRACE_SECONDS, NFS_SSH_OPTIONS,
    NFS_SSH_SECONDS, OPTIONAL_KEYS, OPTIONAL_KEY_PREFIX, OWN_SERVER_SELFTESTS, PLAN_KEY_NEUTRAL_VARIABLES, REPOSITORY_ROOT, REQUIRED_MODULES,
    RUN_CONDITION_EXIT, SCENES_ROOT_NAME, SELFTEST_MARK_TEXT, SELFTEST_MARK_VARIABLE, SELFTEST_ONLY_VARIABLES, SHARDS_ROOT_NAME, SOCKET_NAME,
    SOCKET_PATH_MAXIMUM_BYTES, SSH_CONTROL_DIRECTORY_NAME, SSH_CONTROL_HASH_CHARACTERS, SSH_CONTROL_PERSIST_SECONDS,
    SSH_CONTROL_TEMPORARY_SUFFIX_CHARACTERS, SSH_CONTROL_VARIABLE, SSH_SHIM_DIRECTORY_NAME, TEMPORARY_SERVICE_BINARY_VARIABLE,
    TEMPORARY_SERVICE_FLAG, TEMPORARY_SERVICE_OPTIONS, TEMPORARY_SERVICE_SCRIPT, USAGE_EXIT, VOLATILE_FILESYSTEMS,
)


def gpu_line_fields(text):
    """一行服务行里的 <键>=<值>：{键: 值}，同一个键取头一次出现的（原因那一栏里可能又写着 card= 这类字样，排在后面）；带双引号的值只取到空白为止。"""
    fields = {}
    for word in text.split():
        name, equals, value = word.partition("=")
        if equals and name.isidentifier():
            fields.setdefault(name, value)
    return fields


def gpu_block_number_order(block):
    """判不了那一条正文里块号的次序：数字的按数排在前，别的按字面排在后。"""
    return (not block.isdigit(), int(block) if block.isdigit() else 0, block)


def gpu_service_report(text, environment=None):
    """gpu-verification-server/control cards-failed 的标准输出：(读到的几样, 没读成的台, 认不出的行)。
    读到的几样是 [(种类, 那一句, 台, 认领)]，按种类排（task_server.py 的 gpu_card_failures 拿种类当服务健康的种类）：
    卡坏的种类 gpu-card-failed:<台>:<进程号>:<卡号>，同一台同一个服务进程同一张卡只算一样，认领写 None；
    判不了的种类 gpu-block-not-judgeable:<台>:<族>:<流键头>，同一台同一条流的几块并成一样，不带进程号（GPU 服务重起之后同一条流再判不了还是这一样）；
    那一句写这条流此刻判不了几块、头 GPU_BLOCK_NUMBERS_SHOWN 块的块号与头一块那一行原样，认领是 (放了的块数, 没放的块数)：
    照每块那一行的 claim=，GPU_BLOCK_CLAIMS_LET_GO 里的算放了、GPU_BLOCK_CLAIMS_KEPT 里的算没放（gpu_card_failures 照它写出路）。
    没读成的台是 GPU_CARDS_FAILED_UNREAD_PREFIX 那几行点名的（task_server.py 照它留着那一台上一回读到的，gpu_service_problems_merged）。
    卡坏行取不到卡号、判不了的行取不到那三栏或认领、认领的值两种都不是的，进认不出的行（报错，不猜：卡号退成「?」会把同一个进程的几张坏卡并成一样、
    后坏的漏报；认领猜错了出路就写错）。弄坏开关 gpu-card-number-defaulted 下取不到卡号的记成「?」照收；gpu-block-not-judgeable-unwatched 下判不了的行不认；
    gpu-block-not-judgeable-keyed-per-block 下判不了的照每块各算一样、种类带进程号（gpu-block-not-judgeable:<台>:<进程号>:<族>:<流键头>:<块号>）。别的行不认。"""
    found, streams, unread, unparsed = {}, {}, set(), []
    for line in text.splitlines():
        words = line.split(" ", 3)
        if words[0] == GPU_CARDS_FAILED_UNREAD_PREFIX and len(words) >= 2 and words[1].startswith("host="):
            unread.add(words[1].partition("=")[2])
            continue
        if len(words) != 4 or words[0] not in (GPU_FAILED_CARD_OF_A_HOST_PREFIX, GPU_BLOCK_NOT_JUDGEABLE_OF_A_HOST_PREFIX):
            continue
        if words[0] == GPU_BLOCK_NOT_JUDGEABLE_OF_A_HOST_PREFIX and broken("gpu-block-not-judgeable-unwatched", environment):
            continue
        host, process = words[1].partition("=")[2], words[2].partition("=")[2]
        fields = gpu_line_fields(words[3])
        where = "这一台" if host == GPU_THIS_HOST_LABEL else f"第二台 {host}"
        if words[0] == GPU_FAILED_CARD_OF_A_HOST_PREFIX:
            card = fields.get(GPU_CARD_FIELD)
            if card is None and broken("gpu-card-number-defaulted", environment):
                card = "?"
            if card is None:
                unparsed.append(line)
                continue
            kind = f"{GPU_CARD_FAILED_ALERT_KIND}:{host}:{process}:{card}"
            found.setdefault(kind, (f"{where}的 gpu-verification-server（进程 {process}）报卡坏：{words[3]}", host, None))
        else:
            named = [fields.get(name) for name in GPU_BLOCK_NOT_JUDGEABLE_KEY_FIELDS]
            claim = fields.get(GPU_BLOCK_CLAIM_FIELD)
            if any(value is None for value in named) or claim not in GPU_BLOCK_CLAIMS_LET_GO + GPU_BLOCK_CLAIMS_KEPT:
                unparsed.append(line)
                continue
            family, stream, block = named
            if broken("gpu-block-not-judgeable-keyed-per-block", environment):
                kind = f"{GPU_BLOCK_NOT_JUDGEABLE_ALERT_KIND}:{host}:{process}:{family}:{stream}:{block}"
            else:
                kind = f"{GPU_BLOCK_NOT_JUDGEABLE_ALERT_KIND}:{host}:{family}:{stream}"
            entry = streams.setdefault(kind, {"host": host, "where": where, "family": family, "stream": stream, "head": words[3], "processes": [], "claims": {}})
            if process not in entry["processes"]:
                entry["processes"].append(process)
            entry["claims"][block] = claim   # 同一块报了几回的照最后那一回的认领
    for kind, entry in streams.items():
        blocks = sorted(entry["claims"], key=gpu_block_number_order)
        let_go = sum(1 for claim in entry["claims"].values() if claim in GPU_BLOCK_CLAIMS_LET_GO)
        shown = "、".join(blocks[:GPU_BLOCK_NUMBERS_SHOWN]) + (f"，还有 {len(blocks) - GPU_BLOCK_NUMBERS_SHOWN} 块" if len(blocks) > GPU_BLOCK_NUMBERS_SHOWN else "")
        found[kind] = (f"{entry['where']}的 gpu-verification-server（进程 {'、'.join(entry['processes'])}）判不了族 {entry['family']} 流键头 {entry['stream']} 的 "
                       f"{len(blocks)} 块（块号 {shown}）；头一块那一行：{entry['head']}", entry["host"], (let_go, len(blocks) - let_go))
    return [(kind, line, host, claims) for kind, (line, host, claims) in sorted(found.items())], unread, unparsed


def gpu_service_hosts_read(text):
    """gpu-verification-server/control cards-failed 的标准输出里读成了的台：每台那一行（GPU_CARDS_FAILED_READ_PREFIX）状态在 GPU_CARDS_FAILED_READ_STATES 里、
    又没有没读成那一行（GPU_CARDS_FAILED_UNREAD_PREFIX）点名的。GPU 没开（只打 why=gpu-off）的一台都不点名。
    task_server.py 的 read_gpu_cards_failed 拿它按台记服务起来之后读成过没有（watch_health 照它判一台名下记着发过的能不能当成好了）。"""
    read, unread = set(), set()
    for line in text.splitlines():
        words = line.split(" ")
        if len(words) < 2 or not words[1].startswith("host="):
            continue
        host = words[1].partition("=")[2]
        if words[0] == GPU_CARDS_FAILED_UNREAD_PREFIX:
            unread.add(host)
        elif words[0] == GPU_CARDS_FAILED_READ_PREFIX and gpu_line_fields(line).get("state") in GPU_CARDS_FAILED_READ_STATES:
            read.add(host)
    return read - unread


def gpu_service_problems_merged(previous, found, unread, environment=None):
    """这一回读到的几样并上上一回的：没读成的台（unread）留着上一回读到的那几样，不当成好了；别的台照这一回的。
    previous、found 都是 [(种类, 那一句, 台, 认领)]（gpu_service_report），交回同样的形，按种类排。弄坏开关 gpu-cards-unread-taken-as-healed 下不留。"""
    merged = {kind: (line, host, claims) for kind, line, host, claims in found}
    if not broken("gpu-cards-unread-taken-as-healed", environment):
        for kind, line, host, claims in previous:
            if host in unread:
                merged.setdefault(kind, (line, host, claims))
    return [(kind, line, host, claims) for kind, (line, host, claims) in sorted(merged.items())]


# ── 只在自证里认的变量 ──
def in_selftest(environment=None):
    environment = os.environ if environment is None else environment
    mark = environment.get(SELFTEST_MARK_VARIABLE, "")
    if not mark:
        return False
    try:
        with open(mark, encoding="utf-8") as handle:
            return handle.read().strip() == SELFTEST_MARK_TEXT
    except OSError:
        return False


def selftest_override(name, environment=None):
    """只在自证里认的变量：自证里交它的值，别处交空串（别处设了由 refuse_selftest_variables_or_continue 拒）。"""
    environment = os.environ if environment is None else environment
    return environment.get(name, "") if in_selftest(environment) else ""


def broken(item, environment=None):
    return item in selftest_override(BREAK_VARIABLE, environment).split(",")


def cap_hit_record_bytes(cap, peak, unresolved, environment=None):
    """撞了上限的那一次尝试进定额度的量（字节；文件头「资源记录」）：task_server.py 的 cap_hit_measurement 与 control drop-history --attempt-records-over-gib
    都只经这一处算，两处是同一个数。峰值到了上限的 CAP_HIT_REAL_PEAK_FRACTION 或量不到的取上限（至少要到这个上限）；之后这把键还没有一次跑完的（unresolved）
    取上限乘 CAP_GROWTH_FACTOR（撞了说明要的比这个上限多，同键下一条新任务从放大过的那一档起；任务 15161 撞 6144M，照上限本身记时重交的 15185 还是 6144M）；
    峰值远低于上限的取峰值（脚本开跑前预判装不下就退 250，任务 10731：上限 30.4G、峰值 1.0G，上限不代表它要这么多）。
    弄坏开关 cap-hit-low-peak-counted-at-cap 下峰值远低于上限的也取上限。"""
    measured = int(cap * CAP_GROWTH_FACTOR) if unresolved else int(cap)
    if (isinstance(peak, int) and not isinstance(peak, bool) and 0 < peak < cap * CAP_HIT_REAL_PEAK_FRACTION
            and not broken("cap-hit-low-peak-counted-at-cap", environment)):
        measured = peak
    return measured


def key_words(command, environment=None, plan_step=False):
    """算命令键用的词（文件头「资源记录」的命令键）：起头的 env 与紧跟的「名=值」里名在 KEY_NEUTRAL_VARIABLES 的剥掉，别的赋值照留（留了的连 env 一起留）；
    plan_step（分片计划那一步的键，task_server.py 的 child_key_text）另剥 PLAN_KEY_NEUTRAL_VARIABLES 里的名；
    一个都没剥掉的（不是 env 起头、env 后面先是选项）照原样。弄坏开关 environment-prefix-kept 下照原样，plan-key-shrunk-domain-kept 下计划那一步也不剥缩域变量。"""
    words = unwrapped_temporary_service(list(command), environment)
    if broken("environment-prefix-kept", environment) or not words or os.path.basename(words[0]) != "env":
        return words
    neutral = KEY_NEUTRAL_VARIABLES + (PLAN_KEY_NEUTRAL_VARIABLES if plan_step and not broken("plan-key-shrunk-domain-kept", environment) else ())
    index, kept = 1, []
    while index < len(words) and ASSIGNMENT_WORD.match(words[index]):
        if words[index].split("=", 1)[0] not in neutral:
            kept.append(words[index])
        index += 1
    if len(kept) == index - 1:
        return words
    return ([words[0]] + kept if kept else []) + words[index:]


# cargo test 的同类键（cargo_test_class_words）照留的几样：带值的选项 → 键里写成哪个名字；不带值的开关 → 键里写成哪个名字
CARGO_TEST_CLASS_VALUE_OPTIONS = {"-p": "-p", "--package": "-p", "--exclude": "--exclude", "--manifest-path": "--manifest-path",
                                  "--features": "--features", "-F": "--features", "--profile": "--profile"}
CARGO_TEST_CLASS_FLAGS = {"--workspace": "--workspace", "--all": "--workspace", "--all-features": "--all-features",
                          "--no-default-features": "--no-default-features", "--release": "--release", "-r": "--release"}


def cargo_test_class_words(words, environment=None):
    """命令键用的词里 cargo test 归同类（文件头「资源记录」的命令键：同包同特性同档就是同类）：命令词之前的前缀照留，cargo test 后面只留包（-p / --package、
    --workspace、--exclude、--manifest-path）、特性（--features / -F 的值按逗号与空格拆开、去重排序合成一个，--all-features、--no-default-features）与档（--release、--profile），
    次序固定；点名的测试目标（--test、--lib、--bin、--example、--bench 与它们的值）、位置参数里的测试名、`--` 之后的整段都不进键：点名不同目标的同类命令共用一份记录。
    别的命令照原样。弄坏开关 cargo-test-key-whole-command 下照原样（点名不同目标就各是一个键）。"""
    words = list(words)
    index = command_word_index(words)
    if (broken("cargo-test-key-whole-command", environment) or index + 1 >= len(words)
            or os.path.basename(words[index]) != "cargo" or words[index + 1] != "test"):
        return words
    arguments = words[index + 2:]
    if "--" in arguments:
        arguments = arguments[:arguments.index("--")]
    values, flags, position = {}, set(), 0
    while position < len(arguments):
        name, joined, value = arguments[position].partition("=")
        if name in CARGO_TEST_CLASS_VALUE_OPTIONS:
            if not joined:
                value = arguments[position + 1] if position + 1 < len(arguments) else ""
                position += 1
            values.setdefault(CARGO_TEST_CLASS_VALUE_OPTIONS[name], set()).update(
                piece for piece in (re.split(r"[,\s]+", value) if name in ("--features", "-F") else [value]) if piece)
        elif arguments[position] in CARGO_TEST_CLASS_FLAGS:
            flags.add(CARGO_TEST_CLASS_FLAGS[arguments[position]])
        position += 1
    kept = sorted(flags)
    for option in sorted(values):
        if option == "--features":
            kept += [option, ",".join(sorted(values[option]))]
        else:
            for value in sorted(values[option]):
                kept += [option, value]
    return words[:index + 2] + kept


# 本仓门禁与入口脚本归同类键（selection_class_words）时连值去掉的选法：点名哪几格、哪几项不同的同一道共用一份记录（用户定「类似这样的参数都应该加入」）
SELECTION_VALUE_OPTIONS = ("--check", "--item", "--only")
# 位置参数挪到选项之后时值留在选项旁边的选项：memory_peaks_lib.command_key 把它们连值去掉（那一份的 SELECTION_VALUE_OPTIONS；自证 selection-key-positionals-after-options 核两份相同）
SELECTION_ATTACHED_VALUE_OPTIONS = ("--fingerprint-of", "--crash-expand-up-to")
SELECTION_SCRIPT_PREFIXES = (".claude/gate.d/", "scripts/", ".claude/scripts/")


def repository_script_word(script, environment=None):
    """命令词是本仓门禁与入口脚本（SELECTION_SCRIPT_PREFIXES 下）时交它在仓内的相对路径，不是交 None：以前缀起头的相对路径照 normpath 交（./ 一类去掉）；
    别的（绝对路径：主工作区、快照、仓副本里的同一份；../ 一类相对路径）取头一处「/<前缀>」之后那一截。同一份脚本写成绝对路径与仓内相对路径落一把命令键（selection_class_words）。
    弄坏开关 entry-script-absolute-path-kept 下后一种照原样交。"""
    path = script if os.path.isabs(script) else os.path.normpath(script)
    if not os.path.isabs(script) and path.startswith(SELECTION_SCRIPT_PREFIXES):
        return path
    starts = [path.find(f"/{prefix}") for prefix in SELECTION_SCRIPT_PREFIXES if f"/{prefix}" in path]
    if not starts:
        return None
    return script if broken("entry-script-absolute-path-kept", environment) else path[min(starts) + 1:]


def selection_class_words(words, environment=None):
    """命令键用的词里本仓门禁与入口脚本归同类（文件头「资源记录」的命令键）：命令词是 SELECTION_SCRIPT_PREFIXES 下的脚本（相对路径，或绝对路径里含 /<前缀>）时，
    命令词换成它在仓内的相对路径（repository_script_word：绝对路径与相对路径同一把键），SELECTION_VALUE_OPTIONS 里的选项（--x <值> 与 --x=<值> 两种写法）
    连同值都去掉，在末尾按选项名排序各接一个「--x *」；一个都没有的只换命令词（整道一个键，与点名的分开）。--fingerprint-of、--crash-expand-up-to 连同值
    在这之后由 memory_peaks_lib.command_key 去掉（它的 SELECTION_VALUE_OPTIONS），这里不另去一遍。
    不以 - 起头的词（位置参数：仓根、补丁目录这类）统一排在选项与「--x *」之后、各自保持原次序，仓根写在选项前还是后落同一把键；
    SELECTION_ATTACHED_VALUE_OPTIONS 里的选项与它的值是一对、留在选项那一段，值不跟着挪。只改键，不改起的命令。
    别的命令照原样。弄坏开关 selection-key-whole-command 下照原样（点名不同格、不同项就各是一个键），selection-positionals-in-place 下位置参数留在原位（仓根的位置不同就各是一个键）。"""
    words = list(words)
    index = command_word_index(words)
    if broken("selection-key-whole-command", environment) or index >= len(words):
        return words
    script = repository_script_word(words[index], environment)
    if script is None:
        return words
    in_place = broken("selection-positionals-in-place", environment)
    kept, seen, positionals, position = words[:index] + [script], set(), [], index + 1
    while position < len(words):
        word = words[position]
        name, joined, _ = word.partition("=")
        if name in SELECTION_VALUE_OPTIONS:
            seen.add(name)
            position += 1 if joined else 2
            continue
        if in_place or word.startswith("-"):
            kept.append(word)
            if name in SELECTION_ATTACHED_VALUE_OPTIONS and not joined and not in_place and position + 1 < len(words) and not words[position + 1].startswith("-"):
                kept.append(words[position + 1])
                position += 1
        else:
            positionals.append(word)
        position += 1
    for name in sorted(seen):
        kept += [name, "*"]
    return kept + positionals


def temporary_service_wrapped(command, environment=None):
    """命令包着临时 prune-server 的那一层（unwrapped_temporary_service 剥得开）交 True：命令键前面另接 TEMPORARY_SERVICE_KEY_MARK（task_server.py 的 command_key）。
    弄坏开关 temporary-service-key-shared 下一律 False（包着的与不包的共键，第十二批甲的样子）。"""
    if broken("temporary-service-key-shared", environment):
        return False
    return unwrapped_temporary_service(list(command), environment) != list(command)


def unwrapped_temporary_service(command, environment=None):
    """剥开临时 prune-server 的包装（TEMPORARY_SERVICE_SCRIPT --with-temporary-service [--server-binary <路径>] -- <原命令>，前面的 python3 一起剥；
    包装前面的 env、nice 这类前缀照留）：交前缀加原命令；不是这一层包装的照原样交。算命令键（key_words）、认登记表行
    （task_server.py 的 verdict_library_refusal）、判包装里会编的（task_server.py 的 temporary_service_heavy_refusal）用它；包装本身照旧整条执行。
    弄坏开关 temporary-service-wrapper-kept 下照原样交。"""
    words = list(command)
    if broken("temporary-service-wrapper-kept", environment):
        return words
    split = temporary_service_split(words)
    if split is None:
        return words
    start, inner, _ = split
    return words[:start] + words[inner:]


def temporary_service_split(words):
    """认临时 prune-server 的包装（TEMPORARY_SERVICE_SCRIPT --with-temporary-service [--server-binary <路径>] -- <原命令>）：不是这一层交 None；
    是交 (包装起头的位置：前面的 python3 算在包装里, 原命令起头的位置, 给了 --server-binary 没有)。unwrapped_temporary_service 与 temporary_service_builds_server 共用。"""
    index = command_word_index(words)
    if index >= len(words) - 1 or os.path.basename(words[index]) != TEMPORARY_SERVICE_SCRIPT or words[index + 1] != TEMPORARY_SERVICE_FLAG:
        return None
    start = index - 1 if index > 0 and os.path.basename(words[index - 1]) in ("python3", "python") else index
    position = index + 2
    binary_given = False
    while position < len(words) and words[position] in TEMPORARY_SERVICE_OPTIONS:
        binary_given = True
        position += 2
    if position >= len(words) - 1 or words[position] != "--":
        return None
    return start, position + 1, binary_given


def own_server_selftest_split(words):
    """自带临时服务的那两份脚本（OWN_SERVER_SELFTESTS）的自证：脚本词后面紧跟它认的自证模式词的交 (起头的位置：前面的 python3 算在里面, 给了 --server-binary 没有)，
    不是这种起法交 None。temporary_service_builds_server 用它；包装那一层另由 temporary_service_split 认。"""
    index = command_word_index(words)
    if index >= len(words) - 1:
        return None
    modes = OWN_SERVER_SELFTESTS.get(os.path.basename(words[index]))
    if modes is None or words[index + 1] not in modes:
        return None
    start = index - 1 if index > 0 and os.path.basename(words[index - 1]) in ("python3", "python") else index
    return start, "--server-binary" in words[index + 2:]


def temporary_service_builds_server(command, task_environment=None, environment=None):
    """命令会让 prune-server 现编一份、又没给二进制（--server-binary，命令前的赋值与任务环境里的 TEMPORARY_SERVICE_BINARY_VARIABLE）交 True：这一层包装
    （temporary_service_split）与自带临时服务的两份脚本的自证（own_server_selftest_split）不带二进制起时，都在服务给的编译目录里现编一份 prune-server
    （prune_server_client_selftest.py 的 resolve_binary），外层这一条按会编的算（task_server.py 的 compiles），两台都由服务给编译目录；不是这两种起法、或给了二进制的交 False。
    弄坏开关 temporary-service-build-unrecognized 下一律 False，temporary-service-own-server-unrecognized 下自带临时服务的那两份不认。"""
    if broken("temporary-service-build-unrecognized", environment):
        return False
    words = list(command)
    split = temporary_service_split(words)
    if split is not None:
        start, binary_given = split[0], split[2]
    else:
        own = None if broken("temporary-service-own-server-unrecognized", environment) else own_server_selftest_split(words)
        if own is None:
            return False
        start, binary_given = own
    if binary_given:
        return False
    given_before = any(word.startswith(TEMPORARY_SERVICE_BINARY_VARIABLE + "=") for word in words[:start])
    return not given_before and not (task_environment or {}).get(TEMPORARY_SERVICE_BINARY_VARIABLE)


def selftest_variable_refusals(environment=None):
    environment = os.environ if environment is None else environment
    if in_selftest(environment):
        return []
    named = [name for name in SELFTEST_ONLY_VARIABLES if environment.get(name)]
    if not named:
        return []
    return ([f"  ✗ {name}={environment[name]} 只在自证（python3 task-server/task_server_lib.py --selftest）里认，平常跑设了它就拒，什么都没做" for name in named]
            + [f"  → 怎么办：unset {' '.join(named)} 再跑；要看弄坏开关弄红哪几格，设着 {BREAK_VARIABLE} 跑 --selftest（开关表在 task_server_lib.py 文件头）"])


def refuse_selftest_variables_or_continue(environment=None):
    refusals = selftest_variable_refusals(environment)
    if refusals:
        for line in refusals:
            print(line, file=sys.stderr)
        sys.exit(USAGE_EXIT)


def fast(environment=None):
    return selftest_override("TASK_SERVER_SELFTEST_FAST", environment) == "1"


# ── 运行条件不满足（退 78）──
class Unavailable(Exception):
    """运行条件不满足：str 是卡在哪，next_step 是出路。"""

    def __init__(self, reason, next_step):
        super().__init__(reason)
        self.next_step = next_step


def report_unavailable_and_exit(error, who):
    print(f"  ✗ {who}：{error}", file=sys.stderr)
    print(f"  → 怎么办：{error.next_step}", file=sys.stderr)
    sys.exit(RUN_CONDITION_EXIT)


def missing_modules(environment=None):
    """必须依赖里此刻导入不了的 python 模块（只查找、不导入）。"""
    forced = selftest_override("TASK_SERVER_SELFTEST_MISSING_MODULES", environment).split(",")
    return [name for name in REQUIRED_MODULES if name in forced or importlib.util.find_spec(name) is None]


def require_modules(environment=None):
    missing = missing_modules(environment)
    if missing:
        raise Unavailable(f"必须依赖的 python 模块不在：{'、'.join(missing)}（task-server 没有它们起不来，不退回直接执行命令）",
                          f"{INSTALL_PACKAGES}；装好之后 {INSTALL_COMMAND}，再 python3 task-server/control check 回读到退 0")


# ── 路径（文件头「路径」）──
@dataclass
class Configuration:
    path: str
    state_directory: str
    runtime_directory: str
    signing_key_file: str
    build_directory: str

    @property
    def database(self):
        return os.path.join(self.state_directory, DATABASE_NAME)

    @property
    def logs_directory(self):
        return os.path.join(self.state_directory, "logs")

    @property
    def attempts_directory(self):
        return os.path.join(self.state_directory, "attempts")

    @property
    def shards_directory(self):
        """分片的计划与各片产物（文件头「分片」）：共享工作根下的 SHARDS_ROOT_NAME，两台照同一个绝对路径读写。"""
        return shards_root()

    @property
    def scenes_directory(self):
        """判红那一趟的现场（文件头「现场」）：共享工作根下的 SCENES_ROOT_NAME，两台照同一个绝对路径读写。"""
        return os.path.join(work_root(), SCENES_ROOT_NAME)

    @property
    def alerts_log(self):
        return os.path.join(self.state_directory, "alerts.jsonl")

    @property
    def operations_log(self):
        return os.path.join(self.state_directory, "operations.jsonl")

    @property
    def socket(self):
        return os.path.join(self.runtime_directory, SOCKET_NAME)


def shards_root():
    """分片的计划与各片产物所在的目录（文件头「分片」）：工作根下的 SHARDS_ROOT_NAME，不归多机配置管。"""
    return os.path.join(work_root(), SHARDS_ROOT_NAME)


def work_root_library():
    return load_module("lib_work_root", os.path.join(HOOKS_DIRECTORY, "lib_work_root.py"))


def data_root_unavailable(error, configuration_path):
    """取不到共享数据根（lib_work_root.py 的 DataRootUnconfigured）→ Unavailable（退 78），出路点名往多机配置加 SHARED_DATA_ROOT 那一行（文件头「共享数据根」）。"""
    return Unavailable(f"取不到共享数据根：{error}",
                       f"往多机配置 {configuration_path}（主工作树根的 multi-host.env）加一行 {DATA_ROOT_KEY}=<数据根的绝对路径>，要落在同一份配置 {NFS_ROOTS_KEY} 的某一段下"
                       f"（{NFS_README}）；改好再起")


def work_root():
    """工作根的真路径（.claude/hooks/lib_work_root.py 的 work_root()，SINGLEFS_WORK_ROOT 可换，没设是共享数据根下的 work；在 NFS 共享根下，两台同一个绝对路径）。
    取不到（共享数据根取不到）抛 Unavailable，不补缺省。"""
    library = work_root_library()
    try:
        return os.path.realpath(library.work_root())
    except library.DataRootUnconfigured as error:
        raise data_root_unavailable(error, multi_host_configuration_path()) from None


def shared_data_root(environment=None):
    """共享数据根的真路径（文件头「共享数据根」）：照 lib_work_root.py 的 shared_data_root 取（SINGLEFS_DATA_ROOT，没设取多机配置的 SHARED_DATA_ROOT），不另算、不补缺省；
    多机配置照这里的取法（SINGLEFS_MULTI_HOST_CONFIG 或主工作树根的 multi-host.env）交给它。取不到抛 Unavailable。"""
    environment = os.environ if environment is None else environment
    path = multi_host_configuration_path(environment)
    library = work_root_library()
    try:
        return os.path.realpath(library.shared_data_root(dict(environment, **{MULTI_HOST_CONFIG_VARIABLE: path})))
    except library.DataRootUnconfigured as error:
        raise data_root_unavailable(error, path) from None


def require_configured_data_root(configuration, environment=None):
    """control install、start、check 判多机配置（文件头「共享数据根」）：只看配置里 SHARED_DATA_ROOT 那一行（lib_work_root.py 的 configured_data_root：写了、绝对路径、
    落在 NFS_SHARED_ROOTS 某一段下），不看环境：服务的单元环境里没有 SINGLEFS_DATA_ROOT，起来时取的就是这一行。→ 数据根；不合抛 Unavailable。
    弄坏开关 data-root-key-unchecked 下不判、交空串。"""
    if broken("data-root-key-unchecked", environment):
        return ""
    library = work_root_library()
    try:
        return library.configured_data_root({MULTI_HOST_CONFIG_VARIABLE: configuration.path})
    except library.DataRootUnconfigured as error:
        raise data_root_unavailable(error, configuration.path) from None


def multi_host_addresses_library():
    """第二台地址的共用取法（文件头「主机」）：照文件路径载入 scripts/hosts/multi_host_addresses.py，一个进程只载入一次。"""
    if not ADDRESSES_LIBRARY:
        ADDRESSES_LIBRARY.append(load_module("multi_host_addresses", MULTI_HOST_ADDRESSES_SCRIPT))
    return ADDRESSES_LIBRARY[0]


def peer_ssh_address(configuration_path, environment=None):
    """第二台的 ssh 地址（文件头「主机」）：→ (地址或 None, 取不到的原因连出路)。调共用取法 multi_host_addresses.py 的 peer_ssh_address（整份配置判过才给地址），
    不另写一份判法。"""
    try:
        library = multi_host_addresses_library()
    except (OSError, ImportError, SyntaxError) as error:
        return None, f"载入不了第二台地址的共用取法 {MULTI_HOST_ADDRESSES_SCRIPT}（{type(error).__name__}: {error}）：看那份文件在不在、改坏没有"
    return library.peer_ssh_address(configuration_path, None if environment is None else dict(environment))


def host_addresses(configuration_path, environment=None):
    """这一台与另一台在 HOSTS 里的地址（文件头「主机」）：→ (Addresses(this, peer) 或 None, 取不到的原因连出路)。调共用取法 multi_host_addresses.py 的 resolve，
    不另写一份判法；task_server.py 照它把 HOSTS 里的地址映到 local / peer（分片交给命令的那一台地址、按台的钉值）。"""
    try:
        library = multi_host_addresses_library()
    except (OSError, ImportError, SyntaxError) as error:
        return None, f"载入不了第二台地址的共用取法 {MULTI_HOST_ADDRESSES_SCRIPT}（{type(error).__name__}: {error}）：看那份文件在不在、改坏没有"
    try:
        return library.resolve(configuration_path, None if environment is None else dict(environment)), ""
    except library.AddressesUnavailable as error:
        return None, f"{error}；{library.HOWTO}"


def require_multi_host_keys(configuration, environment=None):
    """control install、start、check 判多机配置的 HOSTS 与 SHARED_DATA_ROOT（文件头「主机」「共享数据根」）：→ (第二台的地址, 数据根)；不合抛 Unavailable（退 78）。"""
    peer, why = peer_ssh_address(configuration.path, environment)
    if peer is None:
        raise Unavailable(f"取不到第二台的地址（多机配置的 {HOSTS_KEY}）：{why}", f"照 task-server/README.md「## NFS 共享」的「多机配置写哪几键」改 {configuration.path}；改好再起")
    return peer, require_configured_data_root(configuration, environment)


def main_worktree_root(directory=REPOSITORY_ROOT):
    """directory 所在的主工作树的根（git common-dir 的上一层）；不在 git 仓里或查不了交空串。"""
    try:
        completed = subprocess.run(["git", "-C", directory, "rev-parse", "--path-format=absolute", "--git-common-dir"],
                                   capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=10)
    except (OSError, subprocess.TimeoutExpired):
        return ""
    common = completed.stdout.strip()
    return os.path.dirname(common) if completed.returncode == 0 and common else ""


def repository_roots():
    """「仓里」指哪几棵树：这一份脚本所在的树与它的主工作树（去重）。"""
    roots = [REPOSITORY_ROOT]
    main_root = main_worktree_root()
    if main_root and os.path.realpath(main_root) != os.path.realpath(REPOSITORY_ROOT):
        roots.append(main_root)
    return roots


def multi_host_configuration_path(environment=None):
    """多机配置在哪（文件头「路径」）：SINGLEFS_MULTI_HOST_CONFIG，否则主工作树根的 multi-host.env，不在 git 仓里取这一棵树根上的。"""
    environment = os.environ if environment is None else environment
    explicit = environment.get(MULTI_HOST_CONFIG_VARIABLE, "")
    if explicit:
        return explicit
    return os.path.join(main_worktree_root() or REPOSITORY_ROOT, MULTI_HOST_CONFIG_NAME)


def default_values(environment):
    home = environment.get("HOME") or os.path.expanduser("~")
    runtime = environment.get("XDG_RUNTIME_DIR") or f"/run/user/{os.getuid()}"
    state_home = environment.get("XDG_STATE_HOME") or os.path.join(home, ".local", "state")
    config_home = environment.get("XDG_CONFIG_HOME") or os.path.join(home, ".config")
    data_home = environment.get("XDG_DATA_HOME") or os.path.join(home, ".local", "share")
    return {"HOME": home, "XDG_RUNTIME_DIR": runtime, "XDG_STATE_HOME": state_home, "XDG_CONFIG_HOME": config_home, "XDG_DATA_HOME": data_home,
            "TASK_SERVER_STATE_DIRECTORY": os.path.join(state_home, "singlefs", "task-server"),
            "TASK_SERVER_RUNTIME_DIRECTORY": os.path.join(runtime, "singlefs-task-server"),
            "TASK_SERVER_SIGNING_KEY_FILE": os.path.join(config_home, "singlefs", "task-server-signing.key"),
            "TASK_SERVER_BUILD_DIRECTORY": os.path.join(data_home, "singlefs", "task-server-builds")}


def expand_value(text, defaults):
    if text.startswith("~/"):
        text = defaults["HOME"] + text[1:]
    for name in EXPANDED_VARIABLES:
        text = text.replace("${" + name + "}", defaults[name])
    return text


def parse_optional_keys(text, path):
    """多机配置里 TASK_SERVER_ 起头的那几行 → ({键: 值}, [问题])；别的行归多机配置的判法，这里不看。"""
    values, problems = {}, []
    for number, line in enumerate(text.splitlines(), 1):
        stripped = line.strip()
        if not stripped.startswith(OPTIONAL_KEY_PREFIX):
            continue
        name, separator, value = stripped.partition("=")
        name, value = name.strip(), value.strip()
        if len(value) >= 2 and value[0] == value[-1] and value[0] in "'\"":
            value = value[1:-1]
        if not separator or (name not in OPTIONAL_KEYS and name != IO_PRESSURE_LIMIT_KEY):
            problems.append(f"{path}:{number} 认不出（TASK_SERVER_ 起头的键只认 {'、'.join(OPTIONAL_KEYS + (IO_PRESSURE_LIMIT_KEY,))}，一行一个 键=值）")
            continue
        if name in values:
            problems.append(f"{path}:{number} {name} 写了第二遍")
            continue
        if name == IO_PRESSURE_LIMIT_KEY and io_pressure_limit_problem(value) and not broken("io-pressure-limit-unchecked"):
            problems.append(f"{path}:{number} {io_pressure_limit_problem(value)}")
            continue
        values[name] = value
    return values, problems


def filesystem_type(path):
    """path（或它最近一层在的祖先）所在的文件系统类型；/proc/self/mounts 读不了交空串。"""
    probe = os.path.realpath(path)
    while not os.path.exists(probe) and probe != "/":
        probe = os.path.dirname(probe)
    best, kind = "", ""
    try:
        with open("/proc/self/mounts", encoding="utf-8") as handle:
            for line in handle:
                fields = line.split()
                if len(fields) < 3:
                    continue
                mount_point = fields[1].replace("\\040", " ")
                if (probe == mount_point or probe.startswith(mount_point.rstrip("/") + "/")) and len(mount_point) >= len(best):
                    best, kind = mount_point, fields[2]
    except OSError:
        return ""
    return kind


def inside(path, directory):
    path, directory = os.path.realpath(path), os.path.realpath(directory)
    return path == directory or path.startswith(directory.rstrip("/") + "/")


def inside_repository(path):
    """落在仓里的话交那一棵树的根，不在交空串。"""
    return next((root for root in repository_roots() if inside(path, root)), "")


def build_tree_root(directory):
    """编译目录组按哪一棵树分（文件头「编译目录」）：directory 解开符号链接之后，往上找到的第一层有 .git（目录或文件，主仓与工作树都算）的那一层；
    哪一层都没有交解开之后的 directory 本身。只看文件系统、不起 git：排队每一轮对每条在排的会编任务都要判一次。"""
    resolved = os.path.realpath(directory)
    probe = resolved
    while True:
        if os.path.lexists(os.path.join(probe, ".git")):
            return probe
        parent = os.path.dirname(probe)
        if parent == probe:
            return resolved
        probe = parent


def state_placement_problem(state_directory, runtime_directory, build_directory, defaults, environment):
    """几个目录放得对不对（文件头「路径」）：状态目录落在仓里、运行时目录下或 tmpfs / ramfs 上，编译目录落在仓里，交原因；放得对交空串。"""
    if broken("state-placement-unchecked", environment):
        return ""
    root = inside_repository(state_directory)
    if root:
        return f"状态目录 {state_directory} 落在仓里（{root}）：日志与库不进仓，仓根的日志会被清"
    for runtime in (runtime_directory, defaults["XDG_RUNTIME_DIR"]):
        if inside(state_directory, runtime):
            return f"状态目录 {state_directory} 落在运行时目录 {runtime} 下：那里是 tmpfs，重启就清"
    kind = filesystem_type(state_directory)
    if kind in VOLATILE_FILESYSTEMS and not selftest_override("TASK_SERVER_SELFTEST_ALLOW_TMPFS", environment):
        return f"状态目录 {state_directory} 在 {kind} 上：重启就清，日志留不住"
    root = inside_repository(build_directory)
    if root:
        return f"编译目录 {build_directory} 落在仓里（{root}）：每次尝试的编译目录不许长在仓里"
    return ""


def socket_path_problem(socket_path, environment=None):
    """socket 的全路径放不放得进 unix socket 的地址（文件头「路径」）：太长交原因（点名路径、字节数与上限），放得下交空串。"""
    if broken("socket-path-unchecked", environment):
        return ""
    length = len(os.fsencode(socket_path))
    if length <= SOCKET_PATH_MAXIMUM_BYTES:
        return ""
    return (f"socket 路径 {socket_path} 有 {length} 字节，超过 unix socket 地址放得下的 {SOCKET_PATH_MAXIMUM_BYTES} 字节"
            "（Linux 的 sun_path 只有 108 字节、含结尾的零）：服务绑不上、客户端连不上")


def read_configuration(environment=None):
    """读路径（文件头「路径」）：交 Configuration；多机配置不在、TASK_SERVER_ 那几个键写坏、目录放错都抛 Unavailable。"""
    environment = os.environ if environment is None else environment
    path = multi_host_configuration_path(environment)
    defaults = default_values(environment)
    try:
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
    except FileNotFoundError:
        if not broken("config-missing-ignored", environment):
            raise Unavailable(f"多机配置 {path} 不在（task-server 照它调度资源；模板是仓根 multi-host.env.example）",
                              f"{CONFIGURATION_STEP}；不在主工作树里跑的设 {MULTI_HOST_CONFIG_VARIABLE}=<主工作树根>/{MULTI_HOST_CONFIG_NAME}") from None
        text = ""
    except OSError as error:
        raise Unavailable(f"多机配置 {path} 读不了（{error}）", "看它的属主与权限，修好再跑") from None
    values, problems = parse_optional_keys(text, path)
    if problems:
        raise Unavailable("多机配置里 TASK_SERVER_ 那几个键写坏了：" + "；".join(problems), f"照 task-server/task_server_lib.py 文件头「路径」改 {path}，或删掉那几行取缺省")
    resolved = {}
    for name in OPTIONAL_KEYS:
        value = expand_value(values[name], defaults) if values.get(name) else defaults[name]
        if not os.path.isabs(value):
            raise Unavailable(f"多机配置 {path} 里 {name} 展开之后是「{value}」，不是绝对路径", f"写成绝对路径，或删掉这一行取缺省（{defaults[name]}）")
        resolved[name] = os.path.normpath(value)
    problem = state_placement_problem(resolved["TASK_SERVER_STATE_DIRECTORY"], resolved["TASK_SERVER_RUNTIME_DIRECTORY"], resolved["TASK_SERVER_BUILD_DIRECTORY"],
                                      defaults, environment)
    if problem:
        raise Unavailable(problem, f"把 {path} 里那个键改到仓外、不会被清的目录，或删掉那一行取缺省（文件头「路径」）")
    problem = socket_path_problem(os.path.join(resolved["TASK_SERVER_RUNTIME_DIRECTORY"], SOCKET_NAME), environment)
    if problem:
        raise Unavailable(problem, f"把 {path} 里的 TASK_SERVER_RUNTIME_DIRECTORY 改到短一点的目录（接上 /{SOCKET_NAME} 不超过 {SOCKET_PATH_MAXIMUM_BYTES} 字节），"
                          f"或删掉那一行取缺省 {defaults['TASK_SERVER_RUNTIME_DIRECTORY']}（缺省也太长的，把 XDG_RUNTIME_DIR 设成短的目录）")
    return Configuration(path, resolved["TASK_SERVER_STATE_DIRECTORY"], resolved["TASK_SERVER_RUNTIME_DIRECTORY"], resolved["TASK_SERVER_SIGNING_KEY_FILE"],
                         resolved["TASK_SERVER_BUILD_DIRECTORY"])


# ── NFS 共享（文件头「NFS 共享」）──
def configuration_key_missing_text(path, name):
    """configured_value 在多机配置里找不到一个键时交的那一句。"""
    return f"多机配置 {path} 里没有 {name}"


def io_pressure_limit_problem(text):
    """多机配置里 IO_PRESSURE_LIMIT_KEY 的值写得对不对：→ 空串（对），或写坏的原因。要写成大于 0、不超过 100 的十进制数（/proc/pressure/io 的 full avg300 的百分数）。"""
    if not re.fullmatch(r"[0-9]+(\.[0-9]+)?", text) or not 0 < float(text) <= 100:
        return f"{IO_PRESSURE_LIMIT_KEY} 要写成大于 0、不超过 100 的数（/proc/pressure/io 的 full avg300 的百分数，例 50），读到「{text}」"
    return ""


def io_pressure_limit(configuration_path, environment=None):
    """多机配置里磁盘压力阈值（文件头「路径」的 IO_PRESSURE_LIMIT_KEY，现读）：→ (阈值或 None, 读不了或写坏的原因)。没写这一行交 (None, "")：不按磁盘压力排，不补缺省。
    弄坏开关 io-pressure-limit-defaulted 下没写的补 IO_PRESSURE_SELFTEST_DEFAULT。"""
    text, why = configured_value(configuration_path, IO_PRESSURE_LIMIT_KEY)
    if text is None:
        if why == configuration_key_missing_text(configuration_path, IO_PRESSURE_LIMIT_KEY):
            return (IO_PRESSURE_SELFTEST_DEFAULT if broken("io-pressure-limit-defaulted", environment) else None), ""
        return None, why
    problem = io_pressure_limit_problem(text)
    return (None, problem) if problem else (float(text), "")


def configured_value(path, name):
    """多机配置里一个键的值：→ (值或 None, 读不到的原因)。NFS_SHARED_ROOTS、HOSTS 与按台的键（configured_by_host）都照这一套读：
    以「名字=」起头的行里最后一行算数；去掉行尾「 # …」注释与成对的引号。"""
    try:
        with open(path, encoding="utf-8") as handle:
            lines = [line for line in handle.read().splitlines() if re.match(rf"^\s*{re.escape(name)}=", line)]
    except OSError as error:
        return None, f"多机配置 {path} 读不了（{error}）"
    if not lines:
        return None, configuration_key_missing_text(path, name)
    value = re.sub(r"\s+#.*$", "", lines[-1].split("=", 1)[1]).strip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in "'\"":
        value = value[1:-1]
    return value, ""


def configured_by_host(configuration_path, key, environment=None):
    """多机配置里一个按台的键（文件头「路径」按台的三个键）：→ ({地址: 值} 或 None, 读不到或写坏的原因)。HOSTS 与这个键照 configured_value 读，
    切法与「每台恰好一项、地址都在 HOSTS 里」两条判法交给共用取法 multi_host_addresses.py 的 by_host_values，不另写一份。
    弄坏开关 by-host-missing-accepted、by-host-outside-accepted 照 BY_HOST_FORWARDED_BREAKS 转给共用取法的同名开关。"""
    try:
        library = multi_host_addresses_library()
    except (OSError, ImportError, SyntaxError) as error:
        return None, f"载入不了按台的键的共用取法 {MULTI_HOST_ADDRESSES_SCRIPT}（{type(error).__name__}: {error}）：看那份文件在不在、改坏没有"
    hosts_text, why = configured_value(configuration_path, HOSTS_KEY)
    if hosts_text is None:
        return None, why
    text, why = configured_value(configuration_path, key)
    if text is None:
        return None, f"{why}（三个按台的键都要写，仓根 multi-host.env.example）"
    forwarded = ",".join(item for name, item in BY_HOST_FORWARDED_BREAKS if broken(name, environment))
    try:
        return library.by_host_values(text, hosts_text.split(), key, {library.BREAK_VARIABLE: forwarded}), ""
    except library.ByHostValueUnavailable as error:
        return None, f"多机配置 {configuration_path} 写坏了：{error}"


class NfsProbeTimedOut(Exception):
    """control check --probe 现探的总时长到点：str 写卡在哪一项（NfsProbeBudget.stuck_text）。"""


class NfsProbeBudget:
    """control check --probe 现探 NFS 共享用的预算：总时长上限 limit 秒（NFS_LIVE_PROBE_LIMIT_SECONDS 或 --limit），每一项开始、结束各经 progress 打一行（带这一项用了几秒），
    ssh 每一次的时限取 NFS_SSH_SECONDS 与剩下的时间里小的那个；到点抛 NfsProbeTimedOut，写卡在哪一项、已多久。clock 只给自证换。"""

    def __init__(self, limit, progress=None, clock=time.monotonic):
        self.limit, self.progress, self.clock = float(limit), progress, clock
        self.deadline = clock() + self.limit
        self.item, self.item_started = "", None

    def remaining(self):
        return self.deadline - self.clock()

    def check(self):
        if self.remaining() <= 0:
            raise NfsProbeTimedOut(self.stuck_text())

    def seconds_for(self, wanted):
        """一次 ssh 最多等多久：wanted 与剩下的时间里小的；已到点的抛 NfsProbeTimedOut。"""
        self.check()
        return min(float(wanted), self.remaining())

    def begin(self, root, name):
        self.item, self.item_started = f"共享根 {root} 的「{name}」", self.clock()
        self.say(f"  … {self.item}开始")

    def end(self, verdict):
        self.say(f"  … {self.item}结束（{self.clock() - self.item_started:.1f} 秒）：{'过' if verdict == NFS_PASSED else '不过或没判：' + str(verdict)}")
        self.item, self.item_started = "", None

    def say(self, line):
        if self.progress is not None:
            self.progress(line)

    def stuck_text(self):
        where = f"卡在 {self.item}（这一项已 {self.clock() - self.item_started:.0f} 秒）" if self.item else "还没进任何一项"
        return f"NFS 共享现探总时长 {self.limit:g} 秒到点：{where}"


def ssh_control_path_problem(control_directory):
    """控制目录放不放得下 ssh 的控制 socket：<控制目录>/<%C 的 40 个字符> 再接 ssh 建主连接时的临时后缀，不超过 socket 路径的上限（SOCKET_PATH_MAXIMUM_BYTES）。
    → 放得下交空串，放不下交那一句。"""
    longest = len(os.fsencode(control_directory)) + 1 + SSH_CONTROL_HASH_CHARACTERS + SSH_CONTROL_TEMPORARY_SUFFIX_CHARACTERS
    if longest > SOCKET_PATH_MAXIMUM_BYTES:
        return (f"ssh 复用连接的控制目录 {control_directory} 太长：接上 %C 与 ssh 的临时后缀共 {longest} 字节，超过 socket 路径的上限 {SOCKET_PATH_MAXIMUM_BYTES}，"
                "这一次服务起的 ssh 不复用连接")
    return ""


def ssh_multiplex_options(environment=None):
    """服务起的 ssh 复用连接的三项（文件头「主机」）：environment 里记了控制目录（SSH_CONTROL_VARIABLE，enable_ssh_multiplexing 设）的交
    ("-o", "ControlMaster=auto", "-o", "ControlPath=<控制目录>/%C", "-o", "ControlPersist=<SSH_CONTROL_PERSIST_SECONDS>")，没记的交空元组（control 这类不是服务起的）。
    主机名不在这里：照调用方给的（多机配置 HOSTS 里那一台，peer_ssh_address）。弄坏开关 ssh-unmultiplexed 下交空元组。"""
    control = (os.environ if environment is None else environment).get(SSH_CONTROL_VARIABLE, "")
    if not control or broken("ssh-unmultiplexed", environment):
        return ()
    return ("-o", "ControlMaster=auto", "-o", f"ControlPath={control}/%C", "-o", f"ControlPersist={SSH_CONTROL_PERSIST_SECONDS}")


def ssh_shim_text(options):
    """复用层那一份 ssh（enable_ssh_multiplexing 写进 <控制目录>/bin/ssh）的正文：PATH 上它之后头一个可执行的 ssh 接上 options 这几项再起，参数原样（主机名照调用方给的）；
    PATH 上除了它自己没有 ssh 的退 255、标准错误写一句。"""
    words = " ".join(shlex.quote(word) for word in options)
    return ("#!/usr/bin/env bash\n"
            "# task-server 起的 ssh 一律复用连接（task_server_lib.py 的 enable_ssh_multiplexing 每次起服务时照 ssh_shim_text 写这一份）：\n"
            "# PATH 上它之后头一个可执行的 ssh 接上 ControlMaster、ControlPath、ControlPersist 三项再起，参数原样\n"
            'own="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"\n'
            'IFS=: read -r -a entries <<< "${PATH:-}"\n'
            'for entry in "${entries[@]}"; do\n'
            '  [[ -n "$entry" && -f "$entry/ssh" && -x "$entry/ssh" ]] || continue\n'
            '  [[ "$(cd "$entry" 2>/dev/null && pwd -P)" == "$own" ]] && continue\n'
            f'  exec "$entry/ssh" {words} "$@"\n'
            "done\n"
            'echo "task-server 的 ssh 复用层（$own/ssh）：PATH 上除了它自己没有 ssh" >&2\n'
            "exit 255\n")


def enable_ssh_multiplexing(environment, state_directory):
    """服务起的 ssh 一律复用连接（文件头「主机」；serve 起服务时调）：控制目录是状态目录下的 SSH_CONTROL_DIRECTORY_NAME（700），记进 environment 的 SSH_CONTROL_VARIABLE
    （peer_shell_timed 照它接三项），控制目录下 bin/ssh 写复用层（ssh_shim_text；host-dispatch.py 经 peer-host-lib.sh 起的 ssh 由 ssh_multiplexed_environment 把它接在 PATH 前面）。
    控制目录放不下 socket 路径的（ssh_control_path_problem）、建不了目录或写不了复用层的不记、交那一句（这一次不复用，服务照起）；成了交空串。
    尝试的环境照提交时的白名单另建（CARRIED_VARIABLES），不带这一层：主连接留在服务的单元里，不进尝试的单元。"""
    control = os.path.join(state_directory, SSH_CONTROL_DIRECTORY_NAME)
    problem = ssh_control_path_problem(control)
    if problem:
        return problem
    try:
        ensure_private_directory(control)
        shim_directory = os.path.join(control, SSH_SHIM_DIRECTORY_NAME)
        ensure_private_directory(shim_directory)
        shim = os.path.join(shim_directory, "ssh")
        temporary = f"{shim}.{os.getpid()}.partial"
        with open(temporary, "w", encoding="utf-8") as handle:
            handle.write(ssh_shim_text(ssh_multiplex_options({SSH_CONTROL_VARIABLE: control})))
        os.chmod(temporary, 0o700)
        os.replace(temporary, shim)
    except OSError as error:
        return f"ssh 复用连接的控制目录 {control} 建不了或复用层写不了（{type(error).__name__}: {error}），这一次服务起的 ssh 不复用连接"
    environment[SSH_CONTROL_VARIABLE] = control
    return ""


def ssh_multiplexed_environment(base, environment):
    """host-dispatch.py 那一路（Peer.run → peer-host-lib.sh 的 peer_host_run，命令词是 PATH 上的 ssh）的环境接上复用层：environment 里记了控制目录的，
    交 base 的一份拷贝，PATH 前面接 <控制目录>/bin（enable_ssh_multiplexing 写的那一份 ssh）、SSH_CONTROL_VARIABLE 照记；没记的交 base 的一份拷贝原样。
    弄坏开关 ssh-unmultiplexed 下原样交。"""
    merged = dict(base)
    control = (environment or {}).get(SSH_CONTROL_VARIABLE, "")
    if not control or broken("ssh-unmultiplexed", environment):
        return merged
    merged[SSH_CONTROL_VARIABLE] = control
    merged["PATH"] = os.pathsep.join(part for part in (os.path.join(control, SSH_SHIM_DIRECTORY_NAME), merged.get("PATH", "")) if part)
    return merged


def close_ssh_masters(environment, configuration_path):
    """停服务时关掉复用的主连接（serve 收尾调）：控制目录里每一个 socket 起一次 ssh -o ControlPath=<那个 socket> -O exit <第二台>（第二台的地址只从多机配置取：
    peer_ssh_address，有 socket 才取）。服务单元 KillMode=process，停服务不杀单元里别的进程：不关的主连接留到 ControlPersist 到点，control stop 回读 cgroup.procs 不空。
    → 关了几个。控制目录没记、列不出、没有 socket、第二台的地址取不到的交 0。"""
    control = (environment or {}).get(SSH_CONTROL_VARIABLE, "")
    if not control:
        return 0
    try:
        sockets = [path for path in (os.path.join(control, name) for name in sorted(os.listdir(control))) if stat.S_ISSOCK(os.lstat(path).st_mode)]
    except OSError:
        return 0
    host = peer_ssh_address(configuration_path, environment)[0] if sockets else None
    if not host:
        return 0
    ssh = selftest_override("TASK_SERVER_PEER_SSH", environment) or "ssh"
    closed = 0
    for path in sockets:
        try:
            subprocess.run([ssh, "-o", f"ControlPath={path}", "-O", "exit", host], capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=10)
        except (OSError, subprocess.TimeoutExpired):
            continue
        closed += 1
    return closed


def peer_shell_timed(host, script, environment=None, budget=None):
    """经 ssh 在第二台上跑一段 shell（NFS_SSH_OPTIONS，服务起的另接复用连接的三项 ssh_multiplex_options）：→ (退出码, 标准输出, 标准错误, 是不是超时)；ssh 起不来或超时退 255。
    自证里 ssh 换成 TASK_SERVER_PEER_SSH 那一份、时限换成 TASK_SERVER_SELFTEST_NFS_SSH_SECONDS。给了 budget（现探）的，时限取它剩下的时间里小的那个，已到点的抛 NfsProbeTimedOut。"""
    ssh = selftest_override("TASK_SERVER_PEER_SSH", environment) or "ssh"
    seconds = float(selftest_override("TASK_SERVER_SELFTEST_NFS_SSH_SECONDS", environment) or NFS_SSH_SECONDS)
    if budget is not None:
        seconds = budget.seconds_for(seconds)
    try:
        completed = subprocess.run([ssh, *NFS_SSH_OPTIONS, *ssh_multiplex_options(environment), host, script], capture_output=True, text=True,
                                   stdin=subprocess.DEVNULL, timeout=seconds)
    except subprocess.TimeoutExpired as error:
        return 255, "", f"{type(error).__name__}: {error}", True
    except OSError as error:
        return 255, "", f"{type(error).__name__}: {error}", False
    return completed.returncode, completed.stdout, completed.stderr, False


def peer_shell(host, script, environment=None, budget=None):
    """peer_shell_timed 去掉是不是超时那一样：→ (退出码, 标准输出, 标准错误)。"""
    code, output, error, _ = peer_shell_timed(host, script, environment, budget)
    return code, output, error


def peer_shell_rechecked(host, script, environment=None, budget=None):
    """连第二台一次超时先当慢（文件头「NFS 共享」的挂载）：→ (退出码, 标准输出, 标准错误, 慢的那一句)。头一次超时的进复查期（NFS_SLOW_GRACE_SECONDS 秒）：每隔
    NFS_RECHECK_PAUSE_SECONDS 复查一回，哪一回回了照那一回的交，慢的那一句写「慢：头一次 ssh 超时 …，隔 … 秒复查到第 N 回回了」；期满（开始下一回之前算）还超时或连不上的
    交 255、标准错误写复查了几回、最后一回的原因。头一次没超时的照它交、不复查。弄坏开关 peer-health-single-timeout-reported 下不复查（头一次超时就照连不上交），
    peer-recheck-once-only 下复查期里只复查一回。给了 budget（control check --probe 的现探）的，每一回 ssh 的时限与回与回之间的等待都不超过它剩下的时间，到点抛 NfsProbeTimedOut。"""
    code, output, error, timed_out = peer_shell_timed(host, script, environment, budget)
    if not timed_out or broken("peer-health-single-timeout-reported", environment):
        return code, output, error, ""
    seconds = float(selftest_override("TASK_SERVER_SELFTEST_NFS_SSH_SECONDS", environment) or NFS_SSH_SECONDS)
    pause = float(selftest_override("TASK_SERVER_SELFTEST_NFS_RECHECK_PAUSE_SECONDS", environment) or NFS_RECHECK_PAUSE_SECONDS)
    grace = float(selftest_override("TASK_SERVER_SELFTEST_NFS_SLOW_GRACE_SECONDS", environment) or NFS_SLOW_GRACE_SECONDS)
    started, rechecks = time.monotonic(), 0
    while True:
        if budget is not None:
            budget.check()
        time.sleep(pause if budget is None else min(pause, max(0.0, budget.remaining())))
        again, output, second_error, second_timed_out = peer_shell_timed(host, script, environment, budget)
        rechecks += 1
        if again != 255:
            return again, output, second_error, (f"慢：头一次 ssh 超时 {seconds:g} 秒（{last_line(error, '没有输出')}），隔 {pause:g} 秒复查到第 {rechecks} 回回了"
                                                f"（复查了 {time.monotonic() - started:.0f} 秒）")
        last = "又超时" if second_timed_out else f"连不上（{last_line(second_error, '没有输出')}）"
        if time.monotonic() - started >= grace or broken("peer-recheck-once-only", environment):
            return 255, output, f"ssh 超时 {seconds:g} 秒，复查期 {grace:g} 秒里隔 {pause:g} 秒复查了 {rechecks} 回仍不通（最后一回{last}）", ""


def last_line(text, fallback):
    lines = [line.strip() for line in (text or "").splitlines() if line.strip()]
    return lines[-1] if lines else fallback


def exported_roots(environment=None):
    """NFS_EXPORTS_FILE 里各行的第一列（本机此刻导出的目录）：→ (集合, 读不到的原因)。自证里读 TASK_SERVER_NFS_EXPORTS 那一份。"""
    path = selftest_override("TASK_SERVER_NFS_EXPORTS", environment) or NFS_EXPORTS_FILE
    try:
        with open(path, encoding="utf-8") as handle:
            return {line.split()[0] for line in handle if line.split()}, ""
    except OSError as error:
        return set(), f"{path} 读不了（{error}）：本机没装 nfs-kernel-server 或没导出"


def nfs_probe_directory(root, environment=None):
    """两向探针写在哪（文件头「NFS 共享」的两向探针）：根下的 .git 是目录时写进它，不在项目根下留条目；别的根写在根下。弄坏开关 nfs-probe-at-root 下一律写在根下。"""
    git_directory = os.path.join(root, ".git")
    return git_directory if os.path.isdir(git_directory) and not broken("nfs-probe-at-root", environment) else root


def nfs_probe(root, host, environment=None, budget=None):
    """两向探针（文件头「NFS 共享」）：→「过」或一句为什么不过。读完两个探针文件都删（共享根在本机盘上，本机删就是两边都删）。
    弄坏开关 nfs-probe-one-way 下只做本机→第二台那一向。"""
    probe_directory = nfs_probe_directory(root, environment)
    # 每一趟的探针文件名各带一个随机后缀：几个会话的看门狗同时起 control check 时，一趟收尾删文件不会删到另一趟正要读的那一份
    suffix = "" if broken("nfs-probe-shared-name", environment) else f"-{secrets.token_hex(8)}"
    local_file = os.path.join(probe_directory, NFS_PROBE_NAME + suffix)
    peer_file = os.path.join(probe_directory, NFS_PEER_PROBE_NAME + suffix)
    try:
        token = secrets.token_hex(16)
        with open(local_file, "w", encoding="utf-8") as handle:
            handle.write(token)
        reader = (f"for try in $(seq 1 {NFS_PROBE_READ_TRIES}); do cat -- {shlex.quote(local_file)} 2>/dev/null && exit 0; sleep 1; done; exit 1")
        code, seen, error = peer_shell(host, reader, environment, budget)
        if code != 0:
            return f"本机→第二台不过：本机写的 {local_file}，第二台读不到（退 {code}：{last_line(error, '没有输出')}）"
        if seen != token:
            return f"本机→第二台不过：本机写的 {local_file}，第二台读回的不是同一个记号"
        if broken("nfs-probe-one-way", environment):
            return NFS_PASSED
        token = secrets.token_hex(16)
        code, _, error = peer_shell(host, f"printf %s {token} > {shlex.quote(peer_file)}", environment, budget)
        if code != 0:
            return f"第二台→本机不过：第二台写不进 {peer_file}（退 {code}：{last_line(error, '没有输出')}）"
        try:
            with open(peer_file, encoding="utf-8") as handle:
                seen = handle.read()
        except OSError as read_error:
            return f"第二台→本机不过：第二台写的 {peer_file} 本机读不到（{read_error}）"
        return NFS_PASSED if seen == token else f"第二台→本机不过：第二台写的 {peer_file}，本机读回的不是同一个记号"
    except OSError as error:
        return f"本机写探针 {local_file} 出错（{error}）"
    finally:
        for path in (local_file, peer_file):
            with contextlib.suppress(OSError):
                os.remove(path)


def nfs_mount_verdict(root, host, environment=None, slow=None, budget=None):
    """「挂载」那一项（文件头「NFS 共享」）：经 ssh 在第二台上 stat -f 这个根下面的「.」——走进目录会触发 x-systemd.automount 挂上它（自动挂载点闲着时
    只是一层 autofs 的占位，mountpoint -q 照样退 0，不说明 NFS 挂上了），读到的文件系统类型是 nfs 才过。→「过」或一句为什么不过、怎么在第二台恢复。
    ssh 一次超时照 peer_shell_rechecked 复查一次：复查回了照回的判，慢的那一句（带上根）追加进 slow（给了的话）；复查仍不通的写连不上。
    弄坏开关 nfs-mount-placeholder-accepted 下照改前只看 mountpoint -q（占位也算挂上）。"""
    if broken("nfs-mount-placeholder-accepted", environment):
        code, _, error, slowly = peer_shell_rechecked(host, f"mountpoint -q -- {shlex.quote(root)}", environment, budget)
        if slowly and slow is not None:
            slow.append(f"共享根 {root} 的「挂载」{slowly}")
        return NFS_PASSED if code == 0 else (f"第二台 mountpoint -q 退 {code}：没把它挂在同一路径" if code != 255
                                             else f"连不上第二台（{last_line(error, '没有输出')}）")
    code, output, error, slowly = peer_shell_rechecked(host, f"{NFS_MOUNT_PROBE_TEXT} {shlex.quote(root + '/.')}", environment, budget)
    if slowly and slow is not None:
        slow.append(f"共享根 {root} 的「挂载」{slowly}")
    if code == 255:
        return f"连不上第二台（{last_line(error, 'ssh 退 255，没有输出')}）"
    kind = output.strip()
    if code == 0 and kind in NFS_FILESYSTEM_TYPES:
        return NFS_PASSED
    found = f"stat -f 退 {code}：{last_line(error, '没有输出')}" if code != 0 else f"那里的文件系统是 {kind or '（空）'}、不是 NFS"
    return f"第二台读不到挂上的 NFS（{found}）：{NFS_MOUNT_RECOVERY.format(root=root)}"


def budgeted_item(budget, root, key, compute):
    """NFS 共享的一项（NFS_ITEMS 里 key 那一项）：没给 budget 的直接算；给了的开始、结束各经 budget 打一行。"""
    if budget is None:
        return compute()
    budget.begin(root, dict(NFS_ITEMS)[key])
    verdict = compute()
    budget.end(verdict)
    return verdict


def nfs_share_report(configuration_path, environment=None, probe=True, budget=None):
    """NFS 共享判得过没有（文件头「NFS 共享」）：→ {ok, problem, next_step, roots: [{root, directory, export, mount, probe}…], checked_at, slow}。
    slow 是「挂载」那一项 ssh 头一次超时、复查回了的那几句（nfs_mount_verdict；服务只记进服务日志）。
    每一项是「过」或一句为什么不过（没判的写「没判：…」）；problem 是第一处不过的那一句（哪个根、哪一项、为什么）。probe=False 不做两向探针。
    budget（NfsProbeBudget，control check --probe 的现探）给了的，每一项开始、结束各打一行、ssh 受它的总时长管，到点抛 NfsProbeTimedOut。
    弄坏开关 nfs-unchecked 下一项都不判、照过。"""
    environment = os.environ if environment is None else environment
    report = {"ok": False, "problem": "", "next_step": f"照 {NFS_README} 在本机导出、第二台挂在同一绝对路径，修好再起", "roots": [], "checked_at": round(time.time(), 3),
              "slow": []}
    if broken("nfs-unchecked", environment):
        return dict(report, ok=True)
    text, why = configured_value(configuration_path, NFS_ROOTS_KEY)
    if not text:
        report.update(problem=f"{why or f'多机配置 {configuration_path} 里 {NFS_ROOTS_KEY} 是空的'}：{NFS_ROOTS_MISSING_TEXT}（结果要两台都读得到）",
                      next_step=f"照 {NFS_README} 在本机导出、第二台挂在同一绝对路径，再往 {configuration_path} 写一行 {NFS_ROOTS_KEY}=<绝对路径>[:<绝对路径>…]（冒号分隔）")
        return report
    roots = [part.strip() for part in text.split(":")]
    if any(not root or not os.path.isabs(root) for root in roots):
        report.update(problem=f"多机配置 {configuration_path} 的 {NFS_ROOTS_KEY}「{text}」写法不对：要是冒号分隔的绝对路径",
                      next_step=f"改成 {NFS_ROOTS_KEY}=<绝对路径>[:<绝对路径>…]（{NFS_README}）")
        return report
    host, host_why = peer_ssh_address(configuration_path, environment)
    exports, exports_why = exported_roots(environment)
    for root in roots:
        entry = {"root": root}
        entry["directory"] = budgeted_item(budget, root, "directory", lambda: NFS_PASSED if os.path.isdir(root) else "本机没有这个目录")
        entry["export"] = budgeted_item(budget, root, "export", lambda: NFS_PASSED if root in exports else (
            exports_why or f"{selftest_override('TASK_SERVER_NFS_EXPORTS', environment) or NFS_EXPORTS_FILE} 里没有第一列是它的导出行"))
        if not host:
            entry["mount"] = f"{host_why}：连不了第二台"
        else:
            entry["mount"] = budgeted_item(budget, root, "mount", lambda: nfs_mount_verdict(root, host, environment, report["slow"], budget))
        if not probe:
            entry["probe"] = "没判：只在起来时与 control check --probe、install、start 做"
        elif entry["directory"] != NFS_PASSED or entry["mount"] != NFS_PASSED:
            entry["probe"] = "没判：本机目录或第二台挂载没过"
        else:
            entry["probe"] = budgeted_item(budget, root, "probe", lambda: nfs_probe(root, host, environment, budget))
        report["roots"].append(entry)
    for entry in report["roots"]:
        failed = next(((key, name) for key, name in NFS_ITEMS if entry.get(key) != NFS_PASSED and not str(entry.get(key)).startswith("没判")), None)
        if failed:
            report["problem"] = f"共享根 {entry['root']} 的「{failed[1]}」不过：{entry[failed[0]]}"
            return report
    report["ok"] = True
    return report


def require_nfs_share(configuration, environment=None, probe=True):
    """NFS 共享判不过抛 Unavailable（退 78）；判得过交那份结论。"""
    report = nfs_share_report(configuration.path, environment, probe)
    if not report["ok"]:
        raise Unavailable(f"NFS 共享判不过：{report['problem']}", report["next_step"])
    return report


# ── 私有目录与签名密钥 ──
def ensure_private_directory(path):
    os.makedirs(path, mode=0o700, exist_ok=True)
    os.chmod(path, 0o700)


def command_word_index(argv, environment=None):
    """剥掉 env（连同 名=值、-u 名、-C 目录、-S 串）、nice（-n N、-nN、--adjustment N、-N）、timeout（-k 时长、-s 信号与别的开关之后的 <时长>）、setsid、
    bash / sh（-o、-O 带一个值）/ python3（-W、-X 带一个值）起脚本、capped.sh N 之后命令词在 argv 里的位置；剥完了没有交 len(argv)。
    带值的选项值写在下一个词里的连同值一起跳过（不停在选项值上，把值当命令词）。弄坏开关 command-wrapper-values-unskipped 下只认 env 的 -u、nice 的 -n 两种分开写的值。"""
    unskipped = broken("command-wrapper-values-unskipped", environment)
    env_values = ("-u", "--unset") if unskipped else ("-u", "--unset", "-C", "--chdir", "-S", "--split-string")
    index = 0
    while index < len(argv):
        word = argv[index]
        name = os.path.basename(word)
        if name == "env":
            index += 1
            while index < len(argv) and ("=" in argv[index] or argv[index].startswith("-")):
                index += 2 if argv[index] in env_values else 1
            continue
        if name == "nice":
            index += 1
            if unskipped:
                index += 2 if index < len(argv) and argv[index] == "-n" else 0
                continue
            while index < len(argv) and argv[index].startswith("-"):
                index += 2 if argv[index] in ("-n", "--adjustment") else 1
            continue
        if name == "timeout":
            index += 1
            while index < len(argv) and argv[index].startswith("-"):
                index += 2 if argv[index] in ("-k", "--kill-after", "-s", "--signal") and not unskipped else 1
            index += 1
            continue
        if name == "setsid":
            index += 1
            while index < len(argv) and argv[index].startswith("-"):
                index += 1
            continue
        if name in ("bash", "sh", "python3", "python"):
            index += 1
            valued = () if unskipped else (("-W", "-X") if name.startswith("python") else ("-o", "-O"))
            while index < len(argv) and argv[index].startswith("-") and argv[index] != "-c":
                index += 2 if argv[index] in valued else 1
            continue
        if name == "capped.sh":
            index += 2
            continue
        break
    return index


def load_module(name, path):
    specification = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module
