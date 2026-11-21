"""agent-watch.py 的一块：知识同步提醒的仓现状（git 状态与内容摘要的读法、基线落盘与读回、基线之后的变化、定时回报一节的写法），以及测这一块的自证用例与造小仓的工具。入口 agent-watch.py 导入并再导出这里的名字。"""
import argparse
import collections
import contextlib
import hashlib
import io
import json
import math
import os
import re
import shlex
import signal
import stat
import subprocess
import sys
import tempfile
import time
import unicodedata
import lib_session_directory as session_directories  # noqa: E402  会话记录目录的唯一取法（来源仓根 TASK_SERVER_SOURCE_REPOSITORY，不经 $HOME；agent-cost.py、agent-handover.py、watch.sh、move-directory-in-use-check.py 同取它）
from agent_watch_transcripts import (
    BROKEN_DETECTION, BYTES_PER_GIBIBYTE, ENTRY_SCRIPT_PATH, ROOT_FREE_BYTES_VARIABLE, bash_result, bash_use, find_transcript,
    handback_records, record_at, write_transcript
)


GIT_TIMEOUT_VARIABLE = "AGENT_WATCH_GIT_TIMEOUT_SECONDS"   # 环境变量只给 --selftest 用：换掉每条 git 命令的时限
GIT_EXECUTABLE_VARIABLE = "AGENT_WATCH_GIT_EXECUTABLE"     # 环境变量只给 --selftest 用：换掉起的那个 git（一个只睡的假 git、一个不存在的路径），自证造超时与起不来
GIT_TIMEOUT_SECONDS_DEFAULT = 30      # 每条 git 命令的时限；超过算查不成
DIGEST_TIMEOUT_VARIABLE = "AGENT_WATCH_DIGEST_TIMEOUT_SECONDS"   # 环境变量只给 --selftest 用：换掉一次快照里算内容摘要的总时限
DIGEST_TIMEOUT_SECONDS_DEFAULT = 30   # 一次快照里算全部内容摘要的总时限；超过算查不成
REPOSITORY_REPORT_LINES_DEFAULT = 20  # --repository-report-lines 的默认值：知识类文件、新提交、别的变了的路径这几张清单各最多列这么多条
# 从别的 git 命令（钩子、git worktree 里的子进程）里继承来会把查询指到别的仓、别的对象库、别的配置或别的命名空间的变量，起 git 之前去掉：
# git 自己列的仓内变量（git rev-parse --local-env-vars，起来时现取一次，git_local_environment_variables），另加不在那张表里的 GIT_NAMESPACE
GIT_LOCAL_ENVIRONMENT_FALLBACK = ("GIT_ALTERNATE_OBJECT_DIRECTORIES", "GIT_CONFIG", "GIT_CONFIG_PARAMETERS", "GIT_CONFIG_COUNT", "GIT_OBJECT_DIRECTORY",
                                  "GIT_DIR", "GIT_WORK_TREE", "GIT_IMPLICIT_WORK_TREE", "GIT_GRAFT_FILE", "GIT_INDEX_FILE", "GIT_NO_REPLACE_OBJECTS",
                                  "GIT_REPLACE_REF_BASE", "GIT_PREFIX", "GIT_SHALLOW_FILE", "GIT_COMMON_DIR")   # 现取不成时用的这 15 个：与 git rev-parse --local-env-vars 列的同一张、同一个次序
GIT_ENVIRONMENT_ALSO_DROPPED = ("GIT_NAMESPACE",)   # 不在 rev-parse --local-env-vars 里、同样把引用指到别处的
GIT_LOCAL_ENVIRONMENT_NAME = re.compile(r"GIT_[A-Z0-9_]+")   # rev-parse --local-env-vars 一行一个名字的写法；有一行对不上就当没取成
# 弄坏开关用的两张短表：repositoryinheritedcommondir 下只去掉前三个；repositorylocalenvironmentunread、repositorylocalenvironmentfallbackshort 下只认这八个（不含 GIT_CONFIG 这一组）
GIT_ENVIRONMENT_DROPPED_WHEN_BROKEN = ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE")
GIT_ENVIRONMENT_DROPPED_WHEN_UNREAD = ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR", "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES",
                                       "GIT_NAMESPACE", "GIT_PREFIX")
GIT_READ_OPTIONS = ("-c", "core.quotepath=off", "-c", "core.fileMode=false")   # 每条 git 都带：路径不加引号；只改执行位不算改了（git status 不列）
GIT_STATUS_ARGUMENTS = ("status", "--porcelain=v1", "-z", "--untracked-files=normal")   # 读状态：NUL 分隔、不加引号、不上色、不打分支行、未跟踪的照列，不受 status.* 与 color.* 配置影响
COMMAND_INDENT = "      "             # 给人抄的命令每条单独一行、行首这么多空格（command_rows）
WHO_TOUCHED_COMMAND = "python3 scripts/agents/who-touched.py <路径>…"   # 「核手上的文件」那一步给的命令：在仓根跑，<路径> 换成手上正在改或要改的文件
# 落盘的基线：会话目录（<家目录>/.claude/projects/<仓路径改写>/<会话号>，取法在 lib_session_directory.py 文件头）底下这一份 JSON（knowledge_state_path）
KNOWLEDGE_STATE_FILE_NAME = "agent-watch-knowledge-sync.json"
KNOWLEDGE_STATE_FORMAT = "agent-watch.py 知识同步提醒的基线"   # 落盘那一份的 format 字段：对不上就当读不出、重新记
KNOWLEDGE_STATE_TEMPORARY_PREFIX = f".{KNOWLEDGE_STATE_FILE_NAME}."   # 落盘时同目录排他新建的临时文件：这个前缀加随机段加 .tmp
KNOWLEDGE_STATE_TEMPORARY_SUFFIX = ".tmp"
KNOWLEDGE_STATE_TEMPORARY_MAX_AGE_SECONDS = 3600   # 起来时删掉会话目录里比这更旧的那种临时文件（落盘时被杀留下的）；更新的可能是别的看门狗正在写的，不碰
# 知识类文件的范围：到点要主 agent 重读的（项目规矩、技能、定义、知识库）。写 / 结尾的是目录前缀，底下每一份都算；不带 / 的是这一份文件本身
KNOWLEDGE_SCOPE = ("CLAUDE.md", ".claude/main-agent.md", ".claude/agent-common.md", ".claude/rules/", ".claude/skills/", ".claude/agents/",
                   ".claude/kb/", ".claude/singlefs-ai-sop/rules/")
# 点名的先后：表里的条目就是次序（目录前缀底下的按路径排），不在表里的排在后面、按路径排
KNOWLEDGE_PRIORITY = (".claude/kb/milestone/", ".claude/kb/decisions.md", ".claude/kb/checks-owed.md", "CLAUDE.md", ".claude/main-agent.md", ".claude/agent-common.md")
COMMITTED_KNOWLEDGE_TAG = "新提交动到"                         # 基线之后的新提交动到的路径
UNCOMMITTED_KNOWLEDGE_TAG = "工作区改了没提交"                  # 工作区里相对基线变了的路径（两头都记了摘要的摘要变了；有一头没记的状态码变了）
BOTH_KNOWLEDGE_TAG = "两样都有"                               # 上面两样都有
DROPPED_KNOWLEDGE_TAG = "起来时未提交的改动已不在 git status 里"   # 基线里在 git status 里、此刻不在了、新提交又没动到它（撤掉了）
IGNORED_KNOWLEDGE_TAG = "被 git 忽略、内容摘要变了"             # 被 git 忽略的知识类文件（git status 与 git diff 都看不见）：改了、新加或删掉
NEWLY_IGNORED_KNOWLEDGE_TAG = "新进 git 忽略"                  # 基线时在工作区里、没被忽略（git status 里有它），此刻被忽略了
UNIGNORED_KNOWLEDGE_TAG = "移出 git 忽略"                      # 基线时被忽略，此刻不再被忽略（出现在 git status 里，或被它列的未跟踪目录盖住）
APPEARED_PATH_KIND = "新出现"          # 基线之后才出现在 git status 里的路径
CHANGED_PATH_KIND = "状态变了"         # 基线里就在 git status 里、此刻状态码不同的路径
VANISHED_PATH_KIND = "不在了"          # 基线里在 git status 里、此刻不在了的路径
RENAME_ARROW = " -> "                  # 改名那一条（原路径, 新路径）在输出里的写法，也是 git status --short 里改名那一行的分隔
STATUS_CODES_JOINER = "+"              # 同一路径在 git status 里出了几条（例：git rm --cached 之后「D 」与「??」）时，几个状态码在输出里的连写
# 输出里路径与 git 读来的文字要转义的字符（visible_text）：换行、制表、回车写成 \n、\t、\r，别的控制字符、行分隔符与段分隔符写成 \x.. 或 \u....，
# 不是合法 UTF-8 的字节（surrogateescape 读成的代理字符）写成 \x..，反斜杠本身写成 \\
VISIBLE_ESCAPES = {"\\": "\\\\", "\n": "\\n", "\t": "\\t", "\r": "\\r"}
VISIBLE_ESCAPED_CATEGORIES = ("Cc", "Cs", "Zl", "Zp")
# 一次快照：status_by_path 是 {路径元组: 状态码元组}（改名、复制那一条是（原路径, 新路径），别的是（路径,)；状态码元组是这一路径在 git status 里每一条的两位状态码，
# 照出现的次序，通常只有一条）；digest_by_path 是 status 里落在范围内的各路径的内容摘要；ignored_digest_by_path 是范围里被 git 忽略的各文件的内容摘要
RepositorySnapshot = collections.namedtuple("RepositorySnapshot", "branch head_hash head_short_hash head_title status_by_path digest_by_path ignored_digest_by_path")
# 比的基线：snapshot 读不出时是 None、说明在 unreadable_reason；state_path 是落盘那一份（取不到会话目录是 None）；reminded_at 是上次提醒的时刻（没提醒过是记下基线的那一只起来的时刻）；
# note 是「基线：」那一行写的这份基线从哪来
RepositoryBaseline = collections.namedtuple("RepositoryBaseline", "root snapshot unreadable_reason state_path reminded_at note")
# 读回的落盘那一份：snapshot 是 None 的是上一只起来时没查成、只落了上次提醒的时刻
KnowledgeState = collections.namedtuple("KnowledgeState", "snapshot reminded_at recorded_at")
DigestDeadline = collections.namedtuple("DigestDeadline", "at seconds")   # 一次快照里算内容摘要的截止时刻（time.monotonic）与总时限
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))   # 从 scripts/agents/ 往上两级是仓根


class RepositoryUnreadable(Exception):
    """git 查不成：说明写卡在哪一条命令、什么原因。"""


def git_timeout_seconds():
    """每条 git 命令的时限：GIT_TIMEOUT_SECONDS_DEFAULT 秒（环境变量 AGENT_WATCH_GIT_TIMEOUT_SECONDS 只给自证换；写成不是数的抛 ValueError，由两处外层接住）。"""
    return float(os.environ.get(GIT_TIMEOUT_VARIABLE) or GIT_TIMEOUT_SECONDS_DEFAULT)


def git_local_environment_variables(git_executable):
    """git 自己列的仓内变量（git rev-parse --local-env-vars，一行一个名字）；起不来、超时、退出码非 0、一个都没列或有一行不像变量名，交写死的 GIT_LOCAL_ENVIRONMENT_FALLBACK。
    起这一条时同样不继承那 15 个与 GIT_NAMESPACE。弄坏开关 repositorylocalenvironmentfallbackshort：取不成时交只有八个的 GIT_ENVIRONMENT_DROPPED_WHEN_UNREAD。"""
    fallback = GIT_ENVIRONMENT_DROPPED_WHEN_UNREAD if BROKEN_DETECTION == "repositorylocalenvironmentfallbackshort" else GIT_LOCAL_ENVIRONMENT_FALLBACK
    environment = {name: value for name, value in os.environ.items() if name not in GIT_LOCAL_ENVIRONMENT_FALLBACK + GIT_ENVIRONMENT_ALSO_DROPPED}
    try:
        finished = subprocess.run([git_executable, "rev-parse", "--local-env-vars"], capture_output=True, timeout=git_timeout_seconds(), env=environment)
    except (OSError, subprocess.SubprocessError):
        return fallback
    names = tuple(line.strip() for line in finished.stdout.decode("utf-8", "replace").splitlines() if line.strip())
    if finished.returncode != 0 or not names or not all(GIT_LOCAL_ENVIRONMENT_NAME.fullmatch(name) for name in names):
        return fallback
    return names


GIT_DROPPED_BY_EXECUTABLE = {}   # git_local_environment_variables 按起的那个 git 现取一次的结果：第一条 git（起来时记基线）取，之后照用


def dropped_git_environment(git_executable):
    """起 git 之前不继承的变量：git 现列的仓内变量加 GIT_ENVIRONMENT_ALSO_DROPPED。
    弄坏开关 repositoryinheritedcommondir：只去掉 GIT_DIR、GIT_WORK_TREE、GIT_INDEX_FILE；repositorylocalenvironmentunread：不现取，只去掉 GIT_ENVIRONMENT_DROPPED_WHEN_UNREAD 那八个。"""
    if BROKEN_DETECTION == "repositoryinheritedcommondir":
        return GIT_ENVIRONMENT_DROPPED_WHEN_BROKEN
    if BROKEN_DETECTION == "repositorylocalenvironmentunread":
        return GIT_ENVIRONMENT_DROPPED_WHEN_UNREAD
    if git_executable not in GIT_DROPPED_BY_EXECUTABLE:
        GIT_DROPPED_BY_EXECUTABLE[git_executable] = git_local_environment_variables(git_executable) + GIT_ENVIRONMENT_ALSO_DROPPED
    return GIT_DROPPED_BY_EXECUTABLE[git_executable]


def run_git(root, git_arguments):
    """在 root 里跑一条 git（带 GIT_READ_OPTIONS），带时限、不丢标准错误；返回标准输出，起不来、超时、退出码非 0 都抛 RepositoryUnreadable。
    标准输出按 surrogateescape 读（不是合法 UTF-8 的文件名照原字节留着，按它找得到文件），标准错误按 replace 读（只进原因）。
    只读：GIT_OPTIONAL_LOCKS=0 让 git status 不去刷新索引，不和别的会话的提交抢锁。dropped_git_environment 里的变量不继承。
    弄坏开关 knowledgefilemodecounted：不带 core.fileMode=false（只改执行位的照列）；knowledgegitoutputreplaced：标准输出按 replace 读（不是 UTF-8 的字节换成 U+FFFD）。"""
    shown = "git " + " ".join(git_arguments)
    timeout = git_timeout_seconds()
    git_executable = os.environ.get(GIT_EXECUTABLE_VARIABLE) or "git"
    dropped = dropped_git_environment(git_executable)
    environment = {name: value for name, value in os.environ.items() if name not in dropped}
    environment["GIT_OPTIONAL_LOCKS"] = "0"
    read_options = GIT_READ_OPTIONS[:2] if BROKEN_DETECTION == "knowledgefilemodecounted" else GIT_READ_OPTIONS
    try:
        finished = subprocess.run([git_executable, *read_options, "-C", root, *git_arguments], capture_output=True, timeout=timeout, env=environment)
    except subprocess.TimeoutExpired:
        raise RepositoryUnreadable(f"{shown} 超过 {timeout:g} 秒没有返回，被停掉了")
    except OSError as error:
        raise RepositoryUnreadable(f"git 起不来（{error}）")
    if finished.returncode != 0:
        error_lines = [line for line in finished.stderr.decode("utf-8", "replace").splitlines() if line.strip()]
        raise RepositoryUnreadable(f"{shown} 退出码 {finished.returncode}：{error_lines[0] if error_lines else '没有错误输出'}")
    return finished.stdout.decode("utf-8", "replace" if BROKEN_DETECTION == "knowledgegitoutputreplaced" else "surrogateescape")


def parse_status_porcelain(text):
    """git status --porcelain=v1 -z 的输出 → {路径元组: 状态码元组}。每一条是「XY 路径」接一个 NUL；状态码里有 R 或 C（改名、复制）的那一条后面另跟一段
    以 NUL 结尾的原路径，键写成（原路径, 新路径）；别的键是（路径,）。路径原样（不加引号、不转义）。同一个键出了几条的（git rm --cached 之后「D 」与「??」），
    几条的状态码照出现的次序都留着。弄坏开关 knowledgerenamesplitoff：改名、复制那一条只认新路径，原路径那一段丢掉；
    knowledgeduplicatepathmerged：同一个键后一条盖掉前一条，只留最后那一码。"""
    status_by_path = {}
    fields = text.split("\0")
    index = 0
    while index < len(fields):
        entry = fields[index]
        index += 1
        if len(entry) < 4:
            continue
        code, path = entry[:2], entry[3:]
        if "R" in code or "C" in code:
            original = fields[index] if index < len(fields) else ""
            index += 1
            key = (path,) if BROKEN_DETECTION == "knowledgerenamesplitoff" else (original, path)
        else:
            key = (path,)
        earlier = () if BROKEN_DETECTION == "knowledgeduplicatepathmerged" else status_by_path.get(key, ())
        status_by_path[key] = earlier + (code,)
    return status_by_path


def parse_status_short_text(text):
    """只给弄坏开关 repositoryshortstatus 用：git status --short 的输出按换行切 → {路径元组: 状态码元组}，路径照 git 加的引号原样留着、改名那一行按「 -> 」拆。"""
    return {tuple(row[3:].split(RENAME_ARROW)): (row[:2],) for row in text.split("\n") if len(row) >= 4}


def status_key_text(status_key):
    """路径元组在输出里的写法：改名、复制那一条写「原路径 -> 新路径」，每一段过 visible_text。"""
    return RENAME_ARROW.join(visible_text(path) for path in status_key)


def status_codes_text(codes):
    """状态码元组在输出里的写法：一条照写两位状态码，几条用 STATUS_CODES_JOINER 连起来。"""
    return STATUS_CODES_JOINER.join(codes)


def visible_text(text):
    """输出里的路径与 git 读来的文字（分支名、提交标题、原因）照 VISIBLE_ESCAPES 与 VISIBLE_ESCAPED_CATEGORIES 转义成看得见的一行：
    带换行的路径不会拆成两行、造不出像「→」出路的一行，不是 UTF-8 的字节写成 \\x..（打印不抛）。弄坏开关 knowledgepathunescaped：原样交回。"""
    if BROKEN_DETECTION == "knowledgepathunescaped":
        return text
    pieces = []
    for character in text:
        code_point = ord(character)
        if character in VISIBLE_ESCAPES:
            pieces.append(VISIBLE_ESCAPES[character])
        elif 0xDC80 <= code_point <= 0xDCFF:   # surrogateescape 读成的代理字符：原来那一个字节
            pieces.append(f"\\x{code_point - 0xDC00:02x}")
        elif unicodedata.category(character) in VISIBLE_ESCAPED_CATEGORIES:
            pieces.append(f"\\x{code_point:02x}" if code_point <= 0xFF else f"\\u{code_point:04x}")
        else:
            pieces.append(character)
    return "".join(pieces)


def codes_by_path(status_by_path):
    """{路径元组: 状态码元组} → {路径: 状态码元组}：改名那一条的两头各记那一条的状态码。"""
    return {path: codes for key, codes in status_by_path.items() for path in key}


def status_counts(status_by_path):
    """→ (已暂存, 未暂存, 未跟踪) 各几条，从 git status 的每一条原始条目的两位状态码现数（同一路径出了几条的各算各的）：第一位不是空格与 ? 算已暂存，
    第二位不是空格与 ? 算未暂存，「??」算未跟踪；一条两边都有改动的（MM、AM）两边各算一次。弄坏开关 repositorystagedunread：不数第一位，已暂存恒为 0。"""
    codes = [code for entry_codes in status_by_path.values() for code in entry_codes]
    staged = 0 if BROKEN_DETECTION == "repositorystagedunread" else sum(1 for code in codes if code[0] not in " ?!")
    unstaged = sum(1 for code in codes if code[1] not in " ?!")
    untracked = sum(1 for code in codes if code == "??")
    return staged, unstaged, untracked


def matches_scope_entry(path, entry):
    """entry 写 / 结尾的是目录前缀，path 在它底下就算；不带 / 的要整条相等。"""
    return path.startswith(entry) if entry.endswith("/") else path == entry


def in_knowledge_scope(path):
    """path 在 KNOWLEDGE_SCOPE 里。弄坏开关 repositoryruleblind：一份知识类文件都不认。"""
    return BROKEN_DETECTION != "repositoryruleblind" and any(matches_scope_entry(path, entry) for entry in KNOWLEDGE_SCOPE)


def knowledge_sort_key(path):
    """点名的次序：KNOWLEDGE_PRIORITY 里的条目在前、按表里的先后，其余在后，同一档按路径排。弄坏开关 knowledgeorderflat：一律只按路径排。"""
    if BROKEN_DETECTION == "knowledgeorderflat":
        return (0, path)
    for rank, entry in enumerate(KNOWLEDGE_PRIORITY):
        if matches_scope_entry(path, entry):
            return (rank, path)
    return (len(KNOWLEDGE_PRIORITY), path)


def digest_deadline_from_now():
    """一次快照里算内容摘要的截止时刻：此刻加 DIGEST_TIMEOUT_SECONDS_DEFAULT 秒（环境变量 AGENT_WATCH_DIGEST_TIMEOUT_SECONDS 只给自证换）。"""
    seconds = float(os.environ.get(DIGEST_TIMEOUT_VARIABLE) or DIGEST_TIMEOUT_SECONDS_DEFAULT)
    return DigestDeadline(time.monotonic() + seconds, seconds)


def check_digest_deadline(deadline):
    """过了截止时刻抛 RepositoryUnreadable（这一节写「没查成」）。弄坏开关 knowledgedigestuntimed：不看截止时刻。"""
    if BROKEN_DETECTION != "knowledgedigestuntimed" and time.monotonic() >= deadline.at:
        raise RepositoryUnreadable(f"算内容摘要超过 {deadline.seconds:g} 秒，停下了（范围里的文件太大或太多）")


def file_digest(path, deadline):
    """一个目录项的内容摘要：普通文件取内容的 sha256（不跟随符号链接、非阻塞地打开，打开之后再核一遍是普通文件）；符号链接取它指向的路径；
    别的类型（命名管道、套接字、设备）只记类型、不打开；读不出写原因。每读一块核一次截止时刻，过了抛 RepositoryUnreadable。
    弄坏开关 knowledgefiletypeunchecked：不判类型，照普通文件阻塞着打开（命名管道没有写端时一直等）。"""
    check_digest_deadline(deadline)
    try:
        if BROKEN_DETECTION == "knowledgefiletypeunchecked":
            handle = open(path, "rb")
        else:
            mode = os.lstat(path).st_mode
            if stat.S_ISLNK(mode):
                return "链接 " + os.readlink(path)
            if not stat.S_ISREG(mode):
                return f"不是普通文件 {stat.filemode(mode)[0]}"
            descriptor = os.open(path, os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW)
            if not stat.S_ISREG(os.fstat(descriptor).st_mode):
                os.close(descriptor)
                return "不是普通文件"
            handle = os.fdopen(descriptor, "rb")
        with handle:
            digest = hashlib.sha256()
            for block in iter(lambda: handle.read(1 << 20), b""):
                digest.update(block)
                check_digest_deadline(deadline)
        return digest.hexdigest()
    except OSError as error:
        return f"读不出 {error.strerror}"


def content_digest(root, relative_path, deadline):
    """工作区里这个路径此刻的内容摘要：不是目录的照 file_digest；目录（git status 把整个未跟踪的目录写成一条）取底下每个目录项的相对路径与 file_digest 合起来的 sha256
    （不跟随指到目录的符号链接）；不存在写「不在」。弄坏开关 knowledgedirectorydigestblind：目录一律记「目录」，底下变了也看不出。"""
    full_path = os.path.join(root, relative_path)
    try:
        mode = os.lstat(full_path).st_mode
    except FileNotFoundError:
        return "不在"
    except OSError as error:
        return f"读不出 {error.strerror}"
    if not stat.S_ISDIR(mode):
        return file_digest(full_path, deadline)
    if BROKEN_DETECTION == "knowledgedirectorydigestblind":
        return "目录"
    overall = hashlib.sha256()
    for directory, subdirectories, file_names in os.walk(full_path):
        subdirectories.sort()
        for file_name in sorted(file_names):
            file_path = os.path.join(directory, file_name)
            overall.update(f"{os.path.relpath(file_path, full_path)}\0{file_digest(file_path, deadline)}\0".encode("utf-8", "surrogateescape"))
    return overall.hexdigest()


def digest_wanted(status_key):
    """git status 里这一条要不要记内容摘要：牵涉的路径有一个落在知识类文件范围里就记（两头都记）；范围外的只比状态码。弄坏开关 knowledgedigestoutofscope：范围外的也记。"""
    return BROKEN_DETECTION == "knowledgedigestoutofscope" or any(in_knowledge_scope(path) for path in status_key)


def ignored_knowledge_paths(root):
    """范围里被 git 忽略的文件（git status 与 git diff 都看不见，例：整个被 .gitignore 忽略的 .claude/singlefs-ai-sop/rules/）：git ls-files 列出范围底下被忽略的未跟踪文件，
    按路径排；内容由调用方按摘要逐个比，不经 git。弄坏开关 knowledgeignoredblind：一份都不列。"""
    if BROKEN_DETECTION == "knowledgeignoredblind":
        return []
    listed = run_git(root, ["ls-files", "-z", "--others", "--ignored", "--exclude-standard", "--", *KNOWLEDGE_SCOPE])
    return sorted(path for path in listed.split("\0") if path and in_knowledge_scope(path))


def repository_snapshot(root):
    """此刻的分支、HEAD（完整哈希、短哈希、标题）、git status（GIT_STATUS_ARGUMENTS，按 parse_status_porcelain 读），status 里落在知识类文件范围内的各路径此刻的内容摘要，
    与范围里被 git 忽略的各文件此刻的内容摘要；算摘要整次带时限（digest_deadline_from_now）。查不成抛 RepositoryUnreadable。
    弄坏开关 repositoryshortstatus：状态改读 git status --short（按换行切、引号原样，受 status.* 与 color.* 配置影响）。"""
    head_text = run_git(root, ["log", "-1", "--no-show-signature", "--format=%H%x09%h%x09%s"]).strip("\n")
    head_fields = head_text.split("\t", 2)
    if len(head_fields) != 3:
        raise RepositoryUnreadable(f"git log -1 的输出认不出：{head_text[:120]!r}")
    branch = run_git(root, ["branch", "--show-current"]).strip() or "（游离 HEAD）"
    if BROKEN_DETECTION == "repositoryshortstatus":
        status_by_path = parse_status_short_text(run_git(root, ["status", "--short"]))
    else:
        status_by_path = parse_status_porcelain(run_git(root, list(GIT_STATUS_ARGUMENTS)))
    ignored_paths = ignored_knowledge_paths(root)
    deadline = digest_deadline_from_now()
    digest_by_path = {path: content_digest(root, path, deadline) for key in status_by_path if digest_wanted(key) for path in key}
    ignored_digest_by_path = {path: content_digest(root, path, deadline) for path in ignored_paths}
    return RepositorySnapshot(branch, head_fields[0], head_fields[1], head_fields[2], status_by_path, digest_by_path, ignored_digest_by_path)


def repository_root_of(arguments):
    """--repository-root 给了用它；没给取统一任务服务交的来源仓根 TASK_SERVER_SOURCE_REPOSITORY（lib_session_directory.py 的 source_repository，两台同一个值），
    服务外手跑没有这个变量，取脚本所在的仓。来源仓根不是绝对路径时抛 session_directories.SessionDirectoryError。
    弄坏开关 repositoryrootorder：后两级对调，脚本所在的仓先于来源仓根。"""
    if BROKEN_DETECTION == "repositoryrootorder":
        return arguments.repository_root or REPOSITORY_ROOT or session_directories.source_repository()
    return arguments.repository_root or session_directories.source_repository() or REPOSITORY_ROOT


def knowledge_state_path(arguments, agent_ids):
    """落盘基线的路径：会话目录底下的 KNOWLEDGE_STATE_FILE_NAME。会话目录是 --session-dir；没给时取被看的子 agent 会话记录所在的那一层（<会话目录>/subagents/agent-<号>.jsonl）；
    都取不到（--processes-only 没给 --session-dir、会话记录还没有）交 None：基线只记在这一只看门狗里。"""
    session_dir = arguments.session_dir
    if not session_dir:
        for agent_id in agent_ids:
            try:
                transcript_path = find_transcript(agent_id, None)
            except session_directories.SessionDirectoryError:
                transcript_path = None
            if transcript_path:
                session_dir = os.path.dirname(os.path.dirname(transcript_path))
                break
    return os.path.join(session_dir, KNOWLEDGE_STATE_FILE_NAME) if session_dir else None


def knowledge_state_encoding_errors():
    """落盘那一份按 UTF-8 读写；摘要里的链接目标与路径带着 surrogateescape 读成的代理字符（不是合法 UTF-8 的字节）时照原字节写、照原字节读回，写盘不抛。
    弄坏开关 knowledgestatestrictencoding：按严格的 UTF-8 读写（带代理字符的写不了，落盘不成）。"""
    return "strict" if BROKEN_DETECTION == "knowledgestatestrictencoding" else "surrogateescape"


def remove_knowledge_state_temporary(temporary):
    """删掉落盘没换上的那份临时文件；已经不在、删不掉的不管（起来时 remove_stale_knowledge_state_temporaries 再删）。"""
    if temporary is None:
        return
    try:
        os.unlink(temporary)
    except OSError:
        pass


def save_knowledge_state(state_path, root, snapshot, reminded_at, recorded_at):
    """把基线与上次提醒的时刻落盘（snapshot 是 None 时只落上次提醒的时刻：起来时就没查成）；recorded_at 是记下的时刻。同目录里排他新建一份名字带随机段的临时文件
    （tempfile.mkstemp），写完 os.replace 换上，读的一方不会读到半截。落盘成了交 None；不成（含编码出错这类别的异常）删掉那份临时文件、交一句原因；
    中途被停（KeyboardInterrupt、SystemExit）的删掉临时文件再往外抛。"""
    snapshot_record = None if snapshot is None else {
        "branch": snapshot.branch, "head_hash": snapshot.head_hash, "head_short_hash": snapshot.head_short_hash, "head_title": snapshot.head_title,
        "status": [[list(codes), list(key)] for key, codes in snapshot.status_by_path.items()],
        "digests": snapshot.digest_by_path, "ignored_digests": snapshot.ignored_digest_by_path}
    record = {"format": KNOWLEDGE_STATE_FORMAT, "repository": os.path.realpath(root), "reminded_at": reminded_at, "recorded_at": recorded_at, "snapshot": snapshot_record}
    temporary = None
    try:
        descriptor, temporary = tempfile.mkstemp(prefix=KNOWLEDGE_STATE_TEMPORARY_PREFIX, suffix=KNOWLEDGE_STATE_TEMPORARY_SUFFIX, dir=os.path.dirname(state_path))
        with os.fdopen(descriptor, "w", encoding="utf-8", errors=knowledge_state_encoding_errors()) as handle:
            json.dump(record, handle, ensure_ascii=False)
        os.replace(temporary, state_path)
    except Exception as error:   # 写盘不往外抛：OSError 之外还有按 UTF-8 编码出错这类
        remove_knowledge_state_temporary(temporary)
        detail = error.strerror if isinstance(error, OSError) and error.strerror else f"{type(error).__name__}：{error}"
        return f"基线落不了盘（{state_path}：{visible_text(detail)}）"
    except BaseException:
        remove_knowledge_state_temporary(temporary)
        raise
    return None


def remove_stale_knowledge_state_temporaries(state_path):
    """起来时删掉会话目录里本文件名前缀（KNOWLEDGE_STATE_TEMPORARY_PREFIX、KNOWLEDGE_STATE_TEMPORARY_SUFFIX）、最后修改超过 KNOWLEDGE_STATE_TEMPORARY_MAX_AGE_SECONDS 的临时文件
    （落盘时被杀留下的）→ 删了几份。读不了目录、删不掉的不报，下一只再删。弄坏开关 knowledgestaletemporarykept：一份都不删。"""
    if BROKEN_DETECTION == "knowledgestaletemporarykept":
        return 0
    directory = os.path.dirname(state_path)
    try:
        names = os.listdir(directory)
    except OSError:
        return 0
    removed = 0
    for name in names:
        if not (name.startswith(KNOWLEDGE_STATE_TEMPORARY_PREFIX) and name.endswith(KNOWLEDGE_STATE_TEMPORARY_SUFFIX)):
            continue
        path = os.path.join(directory, name)
        try:
            if time.time() - os.lstat(path).st_mtime > KNOWLEDGE_STATE_TEMPORARY_MAX_AGE_SECONDS:
                os.unlink(path)
                removed += 1
        except OSError:
            continue
    return removed


def knowledge_time_from_record(name, value, latest):
    """落盘那一份里的一个时刻（reminded_at、recorded_at）→ float；不是数（布尔、字符串、null 都不算）、不是有限数（NaN、无穷）、晚于 latest 的抛 ValueError（当读不出）。
    弄坏开关 knowledgeremindedatunchecked：照 float() 转得了就收（NaN、无穷、将来的时刻与写成字符串的数都收）。"""
    if BROKEN_DETECTION == "knowledgeremindedatunchecked":
        return float(value)
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f"{name} 是 {value!r}，不是数")
    if not math.isfinite(value):
        raise ValueError(f"{name} 是 {value!r}，不是有限数")
    if value > latest:
        raise ValueError(f"{name} 是 {value!r}，晚于此刻")
    return float(value)


def knowledge_snapshot_from_record(fields):
    """落盘那一份的 snapshot 字段 → RepositorySnapshot；认不出抛 ValueError、KeyError 或 TypeError（当读不出）。"""
    status_by_path = {}
    for codes, paths in fields["status"]:
        if not (isinstance(codes, list) and codes and all(isinstance(code, str) and len(code) == 2 for code in codes)
                and isinstance(paths, list) and paths and all(isinstance(path, str) for path in paths)):
            raise ValueError(f"status 里有一条认不出：{[codes, paths]!r}")
        status_by_path[tuple(paths)] = tuple(codes)
    texts = [fields["branch"], fields["head_hash"], fields["head_short_hash"], fields["head_title"]]
    digests, ignored_digests = fields["digests"], fields["ignored_digests"]
    if not all(isinstance(text, str) for text in texts) or not all(
            isinstance(table, dict) and all(isinstance(key, str) and isinstance(value, str) for key, value in table.items()) for table in (digests, ignored_digests)):
        raise ValueError("分支、HEAD 或摘要表的类型不对")
    return RepositorySnapshot(*texts, status_by_path, dict(digests), dict(ignored_digests))


def load_knowledge_state(state_path, root, now):
    """读落盘的那一份 → (KnowledgeState, None)；读不出、格式不对、时刻不是有限数或晚于此刻（now 与墙上时钟里晚的那个）、记的是别的仓 → (None, 「基线：」那一行写的原因)。
    snapshot 是 null 的（上一只起来时没查成，只落了上次提醒的时刻）照读，KnowledgeState.snapshot 是 None。
    弄坏开关 knowledgestaterepositoryunchecked：不核记的是哪个仓，别的仓的基线也照用。"""
    try:
        with open(state_path, encoding="utf-8", errors=knowledge_state_encoding_errors()) as handle:
            record = json.load(handle)
    except FileNotFoundError:
        return None, f"会话目录里还没有落盘的基线（{state_path}），这一只起来时记了"
    except (OSError, ValueError) as error:
        return None, f"落盘的基线读不出（{state_path}：{visible_text(str(error))}），这一只起来时重新记了"
    latest = max(now, time.time())
    try:
        if record["format"] != KNOWLEDGE_STATE_FORMAT:
            raise ValueError(f"format 是 {record['format']!r}")
        if not isinstance(record["repository"], str):
            raise ValueError("仓的类型不对")
        snapshot = None if record["snapshot"] is None else knowledge_snapshot_from_record(record["snapshot"])
        reminded_at = knowledge_time_from_record("reminded_at", record["reminded_at"], latest)
        recorded_at = knowledge_time_from_record("recorded_at", record["recorded_at"], latest)
    except (KeyError, TypeError, ValueError) as error:
        return None, f"落盘的基线读不出（{state_path}：{visible_text(repr(error))}），这一只起来时重新记了"
    if record["repository"] != os.path.realpath(root) and BROKEN_DETECTION != "knowledgestaterepositoryunchecked":
        return None, f"落盘的基线记的是别的仓（{visible_text(record['repository'])}），这一只起来时按 {root} 重新记了"
    return KnowledgeState(snapshot, reminded_at, recorded_at), None


def unreadable_baseline(root, reason, state_path, kept, started_at, note):
    """起来时没查成的基线：上次提醒的时刻取读回的那一份（kept，读得出的话）、读不出取这一只起来的时刻 started_at，照样落盘（快照照读回的那一份留着，没有就只落时刻），
    退出时满了照样带这一节（写「没查成」）。弄坏开关 knowledgeunreadablenotsaved：不落盘（上次提醒的时刻每只都从起来时重新算）。"""
    reminded_at = started_at if kept is None else kept.reminded_at
    if state_path is not None and BROKEN_DETECTION != "knowledgeunreadablenotsaved":
        problem = save_knowledge_state(state_path, root, None if kept is None else kept.snapshot, reminded_at, started_at)
        if problem:
            note = problem if note is None else f"{note}；{problem}"
    return RepositoryBaseline(root, None, reason, state_path, reminded_at, note)


def take_repository_baseline(arguments, started_at, state_path):
    """看门狗起来时定这一只比的基线。先删会话目录里落盘时被杀留下的旧临时文件；state_path 那一份读得出、记的是这个仓：上次提醒的时刻照它记的，
    快照有、起点 HEAD 核得到就接着比它；快照是 null（上一只起来时没查成）或起点 HEAD 核不成的，此刻重新记一份快照（HEAD、git status、范围内各路径与范围里被 git 忽略的
    各文件的内容摘要）。没有、读不出（含时刻不是有限数或晚于此刻）、对不上仓的，此刻重新记一份、上次提醒的时刻记成 started_at。记了的落盘到 state_path，原因写进 note；
    此刻那一份也查不成，照样落盘（unreadable_baseline），原因记进基线，退出时那一节写「没查成」。state_path 是 None（取不到会话目录）时只记在这一只里。
    --repository-report-lines 写 0 不读不记、返回 None。外层接住全部异常（写成「<异常名>：<原因>」当查不成），不往外抛。
    弄坏开关 repositorylineszero：写 0 也照记；repositorystatenotread：不读落盘的，一律此刻重新记；knowledgebaselineexceptionescapes：外层只接查不成，别的异常往外抛。"""
    if arguments.repository_report_lines <= 0 and BROKEN_DETECTION != "repositorylineszero":
        return None
    try:
        root = repository_root_of(arguments)
    except session_directories.SessionDirectoryError as error:
        return RepositoryBaseline(REPOSITORY_ROOT, None, str(error), state_path, started_at, None)
    kept, note = None, None
    try:
        if state_path is None:
            note = "取不到会话目录，基线只记在这一只看门狗里，下一只接不上"
        else:
            remove_stale_knowledge_state_temporaries(state_path)
            if BROKEN_DETECTION == "repositorystatenotread":
                note = f"会话目录里还没有落盘的基线（{state_path}），这一只起来时记了"
            else:
                kept, note = load_knowledge_state(state_path, root, started_at)
        if kept is not None and kept.snapshot is not None:
            try:
                run_git(root, ["cat-file", "-e", f"{kept.snapshot.head_hash}^{{commit}}"])
            except RepositoryUnreadable as error:
                note = (f"落盘的基线里起点 HEAD {kept.snapshot.head_short_hash} 核不成（{visible_text(str(error))}），"
                        "这一只起来时重新记了快照，上次提醒的时刻照落盘的那一份")
            else:
                return RepositoryBaseline(root, kept.snapshot, None, state_path, kept.reminded_at,
                                          f"接着落盘的那一份比（{state_path}，{(max(started_at, kept.recorded_at) - kept.recorded_at) / 60:.1f} 分钟前记的）")
        elif kept is not None:
            note = f"落盘的那一份只有上次提醒的时刻（{state_path}，上一只起来时没查成），这一只起来时重新记了快照"
        reminded_at = started_at if kept is None else kept.reminded_at
        try:
            snapshot = repository_snapshot(root)
        except RepositoryUnreadable as error:
            return unreadable_baseline(root, str(error), state_path, kept, started_at, note)
        if state_path is not None:
            problem = save_knowledge_state(state_path, root, snapshot, reminded_at, started_at)
            if problem:
                note = f"{note}；{problem}"
        return RepositoryBaseline(root, snapshot, None, state_path, reminded_at, note)
    except Exception as error:   # 记基线这一处外层接住全部异常：当查不成，看门狗照常盯、退出码不变
        if BROKEN_DETECTION == "knowledgebaselineexceptionescapes":
            raise
        return unreadable_baseline(root, f"{type(error).__name__}：{error}", state_path, kept, started_at, note)


def new_commits_since(root, start_snapshot, now_snapshot):
    """基线之后新进的提交（最新的在前，每条「短哈希 标题」）与基线的 HEAD 到此刻 HEAD 之间改动过的路径（git diff --name-only -z，按 NUL 切、路径原样）；HEAD 没动是两张空表。
    弄坏开关 repositorycommitsblind：一概当没有新提交；repositorydiffnewline：改动过的路径改读不带 -z 的输出、按换行切（带双引号的文件名被 git 加上引号）。"""
    if BROKEN_DETECTION == "repositorycommitsblind" or start_snapshot.head_hash == now_snapshot.head_hash:
        return [], []
    commit_rows = run_git(root, ["log", "--no-show-signature", "--format=%h %s", f"{start_snapshot.head_hash}..HEAD"]).splitlines()
    if BROKEN_DETECTION == "repositorydiffnewline":
        changed_paths = run_git(root, ["diff", "--name-only", "--no-renames", start_snapshot.head_hash, "HEAD"]).splitlines()
    else:
        changed_paths = [path for path in run_git(root, ["diff", "--name-only", "-z", "--no-renames", start_snapshot.head_hash, "HEAD"]).split("\0") if path]
    return commit_rows, changed_paths


def worktree_path_changed(path, start, now):
    """工作区里这一个路径相对基线变没变（knowledge_files_changed 用）：基线与此刻都记了内容摘要的，只看摘要（状态码变了而摘要没变的不算，例：只暂存、只改执行位）；
    有一头没记摘要的（基线时不在 git status 里），看状态码变没变。弄坏开关 knowledgedigestblind：一律只看状态码；
    knowledgecodechangenamed：状态码变了也算（摘要没变也算变了）。"""
    start_code, now_code = codes_by_path(start.status_by_path).get(path), codes_by_path(now.status_by_path).get(path)
    start_digest, now_digest = start.digest_by_path.get(path), now.digest_by_path.get(path)
    if BROKEN_DETECTION == "knowledgedigestblind" or start_digest is None or now_digest is None:
        return start_code != now_code
    if BROKEN_DETECTION == "knowledgecodechangenamed":
        return start_code != now_code or start_digest != now_digest
    return start_digest != now_digest


def knowledge_files_changed(start, now, commit_paths):
    """知识类文件里基线之后变过的 → [(路径, 来源标签)]，按 knowledge_sort_key 的次序排，不分提交没提交。三处来源取并集：
    基线之后的新提交动到的路径（commit_paths 里落在范围内的）；工作区里相对基线变了的路径（此刻在 git status 里、照 worktree_path_changed 判变了的），
    基线里在 status 里、此刻不在了的另算「已不在」（撤掉了，或提交了）；范围里被 git 忽略的文件，基线与此刻的内容摘要不同的（改了、新加、删掉）与进出忽略的。
    标签：只被新提交动到、只被工作区动到、两样都有、被 git 忽略的照 ignored_knowledge_tag 分（内容变了、新进忽略、移出忽略）、已不在（新提交又没动到它）。
    范围外的路径只比状态码，不在这里。弄坏开关 knowledgeworktreeblind：工作区那一半不看（被 git 忽略的那一半照看）；knowledgedigestoutofscope：范围外的路径状态码没变而
    内容摘要变了也算；knowledgetagsource：标签恒写「新提交动到」。"""
    committed = {path for path in commit_paths if in_knowledge_scope(path)}
    uncommitted = set()
    dropped = set()
    if BROKEN_DETECTION != "knowledgeworktreeblind":
        start_codes, now_codes = codes_by_path(start.status_by_path), codes_by_path(now.status_by_path)
        for path, code in now_codes.items():
            if in_knowledge_scope(path):
                if worktree_path_changed(path, start, now):
                    uncommitted.add(path)
            elif (path in start.digest_by_path and path in now.digest_by_path and start_codes.get(path) == code
                  and start.digest_by_path[path] != now.digest_by_path[path]):   # 范围外的路径不记摘要，只有弄坏开关 knowledgedigestoutofscope 才走到这里
                uncommitted.add(path)
        dropped = {path for path in start_codes if path not in now_codes and in_knowledge_scope(path)}
    ignored_changed = {path for path in set(start.ignored_digest_by_path) | set(now.ignored_digest_by_path)
                       if start.ignored_digest_by_path.get(path, "不在") != now.ignored_digest_by_path.get(path, "不在")}
    rows = []
    for path in sorted(committed | uncommitted | dropped | ignored_changed, key=knowledge_sort_key):
        if BROKEN_DETECTION == "knowledgetagsource":
            tag = COMMITTED_KNOWLEDGE_TAG
        elif path in committed and path in uncommitted:
            tag = BOTH_KNOWLEDGE_TAG
        elif path in committed:
            tag = COMMITTED_KNOWLEDGE_TAG
        elif path in uncommitted:
            tag = UNCOMMITTED_KNOWLEDGE_TAG
        elif path in ignored_changed:
            tag = ignored_knowledge_tag(path, start, now)
        else:
            tag = DROPPED_KNOWLEDGE_TAG
        rows.append((path, tag))
    return rows


def covered_by_status(path, snapshot):
    """path 在这一份快照的 git status 里：自己就是一条，或落在 git status 写成一条的未跟踪目录（键以 / 结尾）底下。"""
    return any(path == entry or (entry.endswith("/") and path.startswith(entry)) for key in snapshot.status_by_path for entry in key)


def ignored_knowledge_tag(path, start, now):
    """范围里被 git 忽略、基线与此刻不同的那一份的标签，按发生的事分：基线时没被忽略、在 git status 里，此刻被忽略了的是「新进 git 忽略」；基线时被忽略，此刻不再被忽略、
    出现在 git status 里的是「移出 git 忽略」；别的（两头都被忽略而内容变了、新加、删掉）是「被 git 忽略、内容摘要变了」。
    弄坏开关 knowledgeignoredtagflat：一律写「被 git 忽略、内容摘要变了」。"""
    if BROKEN_DETECTION != "knowledgeignoredtagflat":
        if path not in start.ignored_digest_by_path and covered_by_status(path, start):
            return NEWLY_IGNORED_KNOWLEDGE_TAG
        if path not in now.ignored_digest_by_path and covered_by_status(path, now):
            return UNIGNORED_KNOWLEDGE_TAG
    return IGNORED_KNOWLEDGE_TAG


def paths_changed_since(start_status, now_status):
    """基线与此刻两份 git status 的差，只列不在知识类文件范围里的路径（范围里的由 knowledge_files_changed 逐份点名）→ [(类别, 一行说明)]：
    新出现、状态变了、不在了三类依次排，类别内按路径排；只比两位状态码，基线之后又改而状态码没变的看不出。弄坏开关 repositorychangesblind：一概当没变。"""
    if BROKEN_DETECTION == "repositorychangesblind":
        return []

    def outside_scope(status_key):
        return not any(in_knowledge_scope(path) for path in status_key)
    appeared = [(APPEARED_PATH_KIND, f"    {APPEARED_PATH_KIND} [{status_codes_text(codes)}] {status_key_text(key)}")
                for key, codes in sorted(now_status.items()) if key not in start_status and outside_scope(key)]
    changed = [(CHANGED_PATH_KIND, f"    {CHANGED_PATH_KIND} [{status_codes_text(start_status[key])}]→[{status_codes_text(codes)}] {status_key_text(key)}")
               for key, codes in sorted(now_status.items()) if key in start_status and start_status[key] != codes and outside_scope(key)]
    vanished = [(VANISHED_PATH_KIND, f"    {VANISHED_PATH_KIND} [{status_codes_text(codes)}] {status_key_text(key)}")
                for key, codes in sorted(start_status.items()) if key not in now_status and outside_scope(key)]
    return appeared + changed + vanished


def visible_count(total, limit):
    """一张清单列几条：不超过上限全列，超过只列上限那么多。弄坏开关 repositorylimitignored：上限不起作用，全列。"""
    return total if BROKEN_DETECTION == "repositorylimitignored" else min(total, limit)


def git_command_line(root, *git_arguments):
    """给人抄的一条 git 命令：git -c core.quotepath=off -C <仓> <参数…>，每一段按 shell 的写法加引号（shlex.quote），仓的路径带单引号、$、反引号与空格也原样交给 shell 就能跑。
    弄坏开关 repositorymanualcommandsquotedloosely：每一段只在两头套一对单引号、里面的单引号不转义。"""
    if BROKEN_DETECTION == "repositorymanualcommandsquotedloosely":
        return " ".join(["git", "-c", "core.quotepath=off", "-C", *(f"'{part}'" for part in (root, *git_arguments))])
    return " ".join(["git", "-c", "core.quotepath=off", "-C", *(shlex.quote(part) for part in (root, *git_arguments))])


def command_rows(heading, commands):
    """给人抄的命令：先一行说明，再每条命令单独一行（行首 COMMAND_INDENT）。弄坏开关 repositorymanualcommandsjoined：几条命令接在说明后面、用全角分号连成一行；
    repositorymanualcommandssemicolon：每条照旧一行，行尾接一个全角分号。"""
    if BROKEN_DETECTION == "repositorymanualcommandsjoined":
        return [heading + "；".join(commands)]
    if BROKEN_DETECTION == "repositorymanualcommandssemicolon":
        return [heading] + [f"{COMMAND_INDENT}{command}；" for command in commands]
    return [heading] + [f"{COMMAND_INDENT}{command}" for command in commands]


def rows_with_remainder(rows, limit, unit, full_commands):
    """清单行最多列 limit 条，余下的写条数，再一行一条看全部的命令。"""
    shown = visible_count(len(rows), limit)
    if shown == len(rows):
        return list(rows)
    return list(rows[:shown]) + command_rows(f"    …余 {len(rows) - shown} {unit}没列，看全部照下面每行一条跑：", full_commands)


def reminder_start_text(baseline, arguments, now):
    """提醒句的前半句：到点该同步项目最新变动与状态，写出离上次提醒（没提醒过从记下基线）多少分钟。"""
    elapsed_minutes = (now - baseline.reminded_at) / 60
    return (f"知识同步提醒：到点了，该同步项目最新变动与状态了。从上次提醒（没提醒过从记下基线）到此刻这 {elapsed_minutes:.1f} 分钟里"
            f"（离上次提醒满 {arguments.max_minutes:g} 分钟的退出才带这一节）")


def unchecked_section_lines(baseline, arguments, now, reason):
    """查不成时这一节的各行：提醒句写明变过哪些知识类文件这一趟没查成，一行「仓的现状：没查成：<原因>」，再一行一条手动看的命令：git status、最近五条提交、
    范围里被 git 忽略的条目（git status 看不见它们）。弄坏开关 knowledgeignoredcommandomitted：不给列被忽略条目的那一条。"""
    root = baseline.root
    commands = [git_command_line(root, "status", "--short"), git_command_line(root, "log", "-5", "--format=%h %s")]
    if BROKEN_DETECTION != "knowledgeignoredcommandomitted":
        commands.append(git_command_line(root, "ls-files", "--others", "--ignored", "--exclude-standard", "--", *KNOWLEDGE_SCOPE))
    return ([f"{reminder_start_text(baseline, arguments, now)}，哪些知识类文件变了这一趟没查成，先自己照下面的命令看，变过的逐份重读",
             f"仓的现状：没查成：{visible_text(reason)}"]
            + command_rows("   → 自己看（被 git 忽略的条目 git status 不列，最后一条列它们），每行一条照原样跑：", commands))


def repository_status_section(baseline, arguments, moment=None):
    """「知识同步提醒」一节的各行与此刻的快照（文件头「知识同步提醒」）→ (行, 此刻的快照；没查成是 None)。次序：以写给主对话的提醒句起头（该同步项目最新变动与状态、
    照下面点名的文件重读，写出离上次提醒多久、变过几份知识类文件、一份没有写明没变）；逐份点名变过的知识类文件与来源标签；「仓的现状」（分支与 HEAD、这份基线从哪来、
    基线之后的新提交、git status 此刻的计数、基线之后变了的别的路径）；末尾一行「→ 同步：」出路。路径、分支名与提交标题过 visible_text。git 查不成（不是仓、git 起不来、超时、
    算摘要超时）时照 unchecked_section_lines 写「没查成」。baseline 是 None（--repository-report-lines 写 0）时是空。moment 是算「离上次提醒多久」的此刻，不给取墙上时钟（自证换）。
    弄坏开关 repositorysectionoff：一概是空；repositoryheadomitted：不写分支与 HEAD 那一行；repositoryfailuresilent：查不成时也是空、不写没查成；
    knowledgereminderlast：提醒句放在这一节最后；repositorybackwardsunsaid：HEAD 往回挪时不写「HEAD 从 … 移到了 …，不是往前走」；
    knowledgecheckcommandomitted：「→ 同步：」那一行不给查谁动过文件的命令。"""
    if baseline is None or BROKEN_DETECTION == "repositorysectionoff":
        return [], None
    limit = arguments.repository_report_lines
    root = baseline.root
    moment = time.time() if moment is None else moment
    reminder_start = reminder_start_text(baseline, arguments, moment)
    try:
        if baseline.snapshot is None:
            raise RepositoryUnreadable(f"看门狗起来时就没读到：{baseline.unreadable_reason}")
        start = baseline.snapshot
        now = repository_snapshot(root)
        commit_rows, commit_paths = new_commits_since(root, start, now)
    except RepositoryUnreadable as error:
        if BROKEN_DETECTION == "repositoryfailuresilent":
            return [], None
        return unchecked_section_lines(baseline, arguments, moment, str(error)), None
    staged, unstaged, untracked = status_counts(now.status_by_path)
    knowledge = knowledge_files_changed(start, now, commit_paths)
    if knowledge:
        reminder = f"{reminder_start}，知识类文件变过 {len(knowledge)} 份，照下面逐份点名的重读。"
    else:
        reminder = f"{reminder_start}，知识类文件没变，没有要重读的。"
    named = rows_with_remainder([f"    {visible_text(path)}（{tag}）" for path, tag in knowledge], limit, "份",
                                [git_command_line(root, "diff", "--name-only", start.head_short_hash, "HEAD"), git_command_line(root, "status", "--short")])
    lines = [f"仓的现状（{root}）："]
    if BROKEN_DETECTION != "repositoryheadomitted":
        lines.append(f"  分支 {visible_text(now.branch)}，HEAD {now.head_short_hash} {visible_text(now.head_title)}")
    if baseline.note:
        lines.append(f"  基线：{baseline.note}")
    if commit_rows:
        lines.append(f"  基线之后的新提交：{len(commit_rows)} 条")
        lines += rows_with_remainder([f"    {visible_text(row)}" for row in commit_rows], limit, "条",
                                     [git_command_line(root, "log", "--oneline", f"{start.head_short_hash}..HEAD")])
    else:
        moved = ("" if start.head_hash == now.head_hash or BROKEN_DETECTION == "repositorybackwardsunsaid"
                 else f"（HEAD 从 {start.head_short_hash} 移到了 {now.head_short_hash}，不是往前走）")
        lines.append(f"  基线之后的新提交：没有新提交{moved}")
    lines.append(f"  git status --short 此刻：已暂存 {staged} 个、未暂存 {unstaged} 个、未跟踪 {untracked} 个"
                 f"（共 {len(now.status_by_path)} 个路径；一个路径两边都有改动的两边各算一次）")
    changes = paths_changed_since(start.status_by_path, now.status_by_path)
    if changes:
        counts = collections.Counter(kind for kind, _ in changes)
        lines.append(f"  基线之后变了的别的路径（知识类文件上面已逐份点名，这里只比状态码）：{APPEARED_PATH_KIND} {counts[APPEARED_PATH_KIND]} 个、"
                     f"{CHANGED_PATH_KIND} {counts[CHANGED_PATH_KIND]} 个、{VANISHED_PATH_KIND} {counts[VANISHED_PATH_KIND]} 个")
        lines += rows_with_remainder([row for _, row in changes], limit, "个", [git_command_line(root, "status", "--short")])
    else:
        lines.append("  基线之后变了的别的路径：没变")
    who_touched = "" if BROKEN_DETECTION == "knowledgecheckcommandomitted" else f"；查谁改过、提到过，在仓根跑 {WHO_TOUCHED_COMMAND}"
    checking = f"核手上正在改或要改的文件有没有被别的会话改过、提交过（对着上面的新提交与别的变了的路径{who_touched}）"
    restart_first = "还有子 agent 在跑的，先照这份报告最后一行「→ 接着盯」起下一只看门狗"
    if knowledge:
        shown = visible_count(len(knowledge), limit)
        left_out = "" if shown == len(knowledge) else f"（余 {len(knowledge) - shown} 份没列，照上面给的命令看全）"
        arrow = f"   → 同步：{restart_first}；再重读上面点名的 {shown} 份知识类文件{left_out}；再{checking}"
    else:
        arrow = f"   → 同步：{restart_first}；知识类文件没变，不用重读；再{checking}"
    if BROKEN_DETECTION == "knowledgereminderlast":
        return named + lines + [arrow, reminder], now
    return [reminder] + named + lines + [arrow], now


def emit_knowledge_sync_section(baseline, arguments, on_alert, now=None, output=None):
    """退出时（告警退 3、定时回报退 4）带不带「知识同步提醒」一节，带了先打出、flush，再落盘 → 打出的行（不带是空表）。
    满没满：先重读 state_path 那一份，上次提醒的时刻取它与这一只内存里晚的那个（几只看门狗盯同一个会话时，另一只刚提醒过的这一只不再提醒），离它满 --max-minutes 才带，
    提醒句里的分钟数照它算。带了：读到了此刻的仓就把基线换成此刻；没查成的快照不换（照落盘的那一份留着）；两种都把上次提醒的时刻记成此刻、落盘（state_path 是 None 的不落盘）。
    落不了盘的在打出的这一节之后补一行「基线没换成此刻」。算这一节时的异常外层接住（照查不成写「没查成：<异常名>：<原因>」），退出码不因这一节改变。
    now 是退出的此刻、不给取墙上时钟，output 不给取标准输出（自证换）。
    弄坏开关 repositoryonalert：告警退出不看满没满、一律带；repositorydueonalertignored：告警退出一律不带；repositorybaselinekept：带了也不换基线、不记时刻；
    knowledgestatenotreread：不重读落盘的那一份，只看这一只起来时读到的上次提醒；knowledgesavedbeforeprint：先落盘、再打出；
    knowledgeunreadablenotadvanced：没查成的那一次不落盘、上次提醒的时刻不往前推；knowledgesectionexceptionescapes：外层只接查不成，别的异常往外抛。"""
    if baseline is None:
        return []
    now = time.time() if now is None else now
    output = sys.stdout if output is None else output
    reread = None
    if baseline.state_path is not None and BROKEN_DETECTION != "knowledgestatenotreread":
        try:
            reread, _ = load_knowledge_state(baseline.state_path, baseline.root, now)
        except Exception:   # 重读不成照这一只内存里的上次提醒判（同属带这一节这一处外层）
            if BROKEN_DETECTION == "knowledgesectionexceptionescapes":
                raise
    if reread is not None and reread.reminded_at > baseline.reminded_at:
        baseline = baseline._replace(reminded_at=reread.reminded_at)
    due = now - baseline.reminded_at >= arguments.max_minutes * 60
    if on_alert and BROKEN_DETECTION == "repositoryonalert":
        due = True
    if on_alert and BROKEN_DETECTION == "repositorydueonalertignored":
        due = False
    if not due:
        return []
    try:
        lines, now_snapshot = repository_status_section(baseline, arguments, now)
    except Exception as error:   # 带这一节这一处外层接住全部异常：照查不成写，退出码不变
        if BROKEN_DETECTION == "knowledgesectionexceptionescapes":
            raise
        lines, now_snapshot = unchecked_section_lines(baseline, arguments, now, f"{type(error).__name__}：{error}"), None
    if not lines:
        return []
    kept_snapshot = now_snapshot
    if kept_snapshot is None:   # 没查成：快照不换，照落盘的那一份（读不回就照这一只起来时的）
        kept_snapshot = reread.snapshot if reread is not None and reread.snapshot is not None else baseline.snapshot

    def save_this_reminder():
        if (baseline.state_path is None or BROKEN_DETECTION == "repositorybaselinekept"
                or (now_snapshot is None and BROKEN_DETECTION == "knowledgeunreadablenotadvanced")):
            return None
        return save_knowledge_state(baseline.state_path, baseline.root, kept_snapshot, now, now)
    if BROKEN_DETECTION == "knowledgesavedbeforeprint":
        problem = save_this_reminder()
        print_lines(lines, output)
    else:
        print_lines(lines, output)
        problem = save_this_reminder()
    if problem:
        unsaved_line = f"  基线没换成此刻：{problem}"
        print_lines([unsaved_line], output)
        lines = lines + [unsaved_line]
    return lines


def report_deadline_of(watch_started, repository_baseline, arguments):
    """定时回报的时刻 → (时刻, 是不是「离上次知识同步提醒满了」先到)：起来时刻加 --max-minutes，与上次提醒的时刻（repository_baseline.reminded_at，落盘的那一份）
    加 --max-minutes 里早的那个；起来时就已经过了的，第一次整查之后当场退 4。没有基线（--repository-report-lines 写 0）只看起来时刻。
    弄坏开关 knowledgedeadlinefromstart：只看起来时刻。"""
    period = arguments.max_minutes * 60
    started_deadline = watch_started + period
    if repository_baseline is None or BROKEN_DETECTION == "knowledgedeadlinefromstart":
        return started_deadline, False
    reminder_deadline = repository_baseline.reminded_at + period
    if reminder_deadline < started_deadline:
        return reminder_deadline, True
    return started_deadline, False


def timed_report_head(watch_started, arguments, reminder_first):
    """「定时回报：」那一行的前半句：到了起来之后的回报周期，或离上次知识同步提醒满了先到（写这一只看了多久）。"""
    if reminder_first:
        return (f"定时回报：离上次知识同步提醒满 {arguments.max_minutes:g} 分钟，提前回报（这一只看了 {(time.time() - watch_started) / 60:.1f} 分钟）")
    return f"定时回报：已经看了 {arguments.max_minutes:g} 分钟"


def processes_restart_line():
    """--processes-only 退 3、退 4 时最后一行：照抄能起的下一只（watch.sh 认得的写法）。弄坏开关 restarthint：印回这里自己的选项。"""
    return ("   → 接着盯就用 run_in_background 再起一个看门狗："
            f"{'--processes-only' if BROKEN_DETECTION == 'restarthint' else 'bash scripts/agents/watch.sh --processes'}")


def print_lines(lines, output=None):
    """逐行打出、flush（output 不给取标准输出）。"""
    output = sys.stdout if output is None else output
    for line in lines:
        print(line, file=output)
    output.flush()

SELFTEST_GIT_ENVIRONMENT = {"GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_NOSYSTEM": "1"}   # 自证里的 git 不读用户与系统的配置（本机 git 配置那一格另给自造的）
SELFTEST_DUE_SECONDS_AGO = 7200        # 预先落盘的基线把上次提醒写成这么久之前：比各格的 --max-minutes 都长，退出时一定满
SELFTEST_ALERT_WATCH_MINUTES = "1"     # 按告警退出的几只的回报周期：告警在第一次整查就叫醒、远不到这么久，所以没满；告警造不出来时（弄坏开关 timeout、rootfree）到点照样退 4
SELFTEST_FIFO_WATCH_SECONDS = 60       # 命名管道那一格等看门狗退出的时限：到点没退就停它自己那一组、判红
SELFTEST_SHELL_SECONDS = 60            # 给人抄的命令原样写进一份脚本交给 bash，每条的时限
SELFTEST_INJECTED_MARKER = "selftest-injected-marker"   # 带特殊字符的仓路径里反引号那一段要打的字：照抄的命令真执行了它，输出里才有这一串
SELFTEST_SPECIAL_ROOT_DIRECTORY = f"q'uote $HOME `echo {SELFTEST_INJECTED_MARKER}` dir"   # 给人抄的命令那一格的小仓放在这个目录底下：单引号、$、反引号与空格都有
SELFTEST_CHAIN_MAX_WATCHERS = 40       # 一串看门狗在本进程里照退出时刻走（selftest_reminder_chain），最多走这么多只：弄坏开关下每只一起来就退时到这里停、判红
SELFTEST_CHAIN_MAX_MINUTES = 60        # 那一串的 --max-minutes（与 watch.conf 的 max-minutes 同一个数）
SELFTEST_CHAIN_HORIZON_MINUTES = 180   # 那一串走到这么多分钟为止
SELFTEST_CHAIN_SLACK_MINUTES = 0.001   # 比最长间隔时容下的浮点误差
# 范围里每个条目至少一份，次序就是「新提交动到这些时点名的次序」：KNOWLEDGE_PRIORITY 里的先排（里程碑、决策索引、欠账表，再 CLAUDE.md、主 agent 规矩、共用约束），其余按路径排
SELFTEST_KNOWLEDGE_FILE_SAMPLES = {".claude/kb/milestone/sample-milestone.md": "一份里程碑\n", ".claude/kb/decisions.md": "决策索引\n", ".claude/kb/checks-owed.md": "欠账表\n",
                                   "CLAUDE.md": "项目说明\n", ".claude/main-agent.md": "主 agent 的规矩\n", ".claude/agent-common.md": "共用约束\n",
                                   ".claude/agents/sample-agent.md": "一份定义\n", ".claude/kb/other.md": "别的知识库页\n", ".claude/rules/sample-rule.md": "一条规则\n",
                                   ".claude/singlefs-ai-sop/rules/sample-sop-rule.md": "一条上游规则\n", ".claude/skills/sample/SKILL.md": "一份技能\n"}
CHANGED_COMMIT_TITLE = "自证新提交：加普通文件"
PROCESSES_COMMIT_TITLE = "自证新提交：进程那一路加普通文件"
KNOWLEDGE_COMMIT_TITLE = "自证新提交：动知识类文件"
MAIN_AGENT_COMMIT_TITLE = "自证新提交：改主 agent 的规矩"
CAPPED_COMMITS = (("自证新提交：加登记行", "registry-line.txt"), ("自证新提交：改说明", "description.txt"), ("自证新提交：删旧样本", "old-sample.txt"))   # 从旧到新
CAPPED_KNOWLEDGE_FILES = (".claude/kb/capped-registry-note.md", ".claude/kb/capped-report-note.md")   # 前两条提交各带一份；按路径排，列出来的第一个是 capped-registry-note.md
CAPPED_NEW_FILES = ("new-notes.txt", "new-report.txt", "new-table.txt")   # 按文件名排，列出来的第一个是 new-notes.txt
WORKTREE_TRACKED_KNOWLEDGE = ".claude/kb/tracked.md"                  # 基线时干净、之后改了没提交
WORKTREE_STEADY_KNOWLEDGE = ".claude/kb/steady-modified.md"           # 基线时已改（未暂存）、之后不动
WORKTREE_REWRITTEN_KNOWLEDGE = ".claude/kb/already-modified.md"       # 基线时已改（未暂存）、之后又改了内容，状态码不变
WORKTREE_UNTRACKED_KNOWLEDGE = ".claude/kb/already-untracked.md"      # 基线时未跟踪、之后又改了内容，状态码不变
WORKTREE_REVERTED_KNOWLEDGE = ".claude/kb/reverted.md"                # 基线时已改（未暂存）、之后撤回成提交里的内容
WORKTREE_OUTSIDE_FILE = "docs/already-modified.md"                    # 范围外：基线时已改（未暂存）、之后又改了内容，状态码不变
GAP_KNOWLEDGE_FILE = ".claude/kb/gap.md"                              # 两只看门狗之间（前一只退出之后、后一只起来之前）改的那一份
IGNORED_KNOWLEDGE_DIRECTORY = ".claude/singlefs-ai-sop/rules/"        # 被 .gitignore 整个忽略的范围条目（照本仓 .gitignore 的「/.claude/singlefs-ai-sop/」）
KNOWLEDGE_ROW_PATTERN = re.compile(r"^    (\S.*)（(" + "|".join(re.escape(tag) for tag in (
    COMMITTED_KNOWLEDGE_TAG, UNCOMMITTED_KNOWLEDGE_TAG, BOTH_KNOWLEDGE_TAG, IGNORED_KNOWLEDGE_TAG, NEWLY_IGNORED_KNOWLEDGE_TAG, UNIGNORED_KNOWLEDGE_TAG,
    DROPPED_KNOWLEDGE_TAG)) + r")）$")   # 逐份点名的那一行「    <路径>（<标签>）」


def named_knowledge_rows(lines):
    """这一节逐份点名的知识类文件，按出现的次序 → [(路径, 来源标签)]（路径可以带空格）。"""
    return [found.groups() for found in map(KNOWLEDGE_ROW_PATTERN.match, lines) if found]


def selftest_git(directory, *git_arguments):
    """自证在临时目录的小仓里跑的 git：不读用户与系统的 git 配置、不跑钩子、不签名；失败抛 CalledProcessError（自证当场崩，不当成判红）。"""
    return subprocess.run(["git", "-c", "user.name=agent-watch-selftest", "-c", "user.email=selftest@example.invalid", "-c", "commit.gpgsign=false",
                           "-c", f"core.hooksPath={os.devnull}", "-C", directory, *git_arguments],
                          check=True, capture_output=True, text=True, timeout=60, env=dict(os.environ, **SELFTEST_GIT_ENVIRONMENT)).stdout


def write_selftest_file(directory, relative_path, text):
    path = os.path.join(directory, relative_path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def build_selftest_repository(directory):
    """临时目录里造一个小仓：分支 selftest-branch、一条提交（a.txt），之后一个已暂存的新文件 staged.txt 与一个未跟踪的 old-untracked.txt。返回 directory。"""
    os.makedirs(directory)
    selftest_git(directory, "init", "-q")
    selftest_git(directory, "symbolic-ref", "HEAD", "refs/heads/selftest-branch")
    write_selftest_file(directory, "a.txt", "第一行\n")
    selftest_git(directory, "add", "--", "a.txt")
    selftest_git(directory, "commit", "-q", "-m", "自证起点提交")
    write_selftest_file(directory, "staged.txt", "已暂存\n")
    selftest_git(directory, "add", "--", "staged.txt")
    write_selftest_file(directory, "old-untracked.txt", "未跟踪\n")
    return directory


def build_knowledge_repository(directory, committed_files, branch="selftest-branch"):
    """临时目录里造一个小仓：分支 branch，committed_files（{路径: 内容}）一次提交成起点（标题「自证起点提交」）。返回 directory。"""
    os.makedirs(directory)
    selftest_git(directory, "init", "-q")
    selftest_git(directory, "symbolic-ref", "HEAD", f"refs/heads/{branch}")
    for relative_path, text in committed_files.items():
        write_selftest_file(directory, relative_path, text)
    selftest_git(directory, "add", "--", *committed_files)
    selftest_git(directory, "commit", "-q", "-m", "自证起点提交")
    return directory


def build_selftest_worktree_repository(directory):
    """临时目录里造一个带知识类文件的小仓：分支 selftest-branch、一条提交（a.txt、.claude/main-agent.md 与 WORKTREE_* 那几份已提交的文件，内容都是「原文」），
    之后的 git status 是：WORKTREE_STEADY_KNOWLEDGE、WORKTREE_REWRITTEN_KNOWLEDGE、WORKTREE_REVERTED_KNOWLEDGE 与范围外的 WORKTREE_OUTSIDE_FILE 各改了没暂存，
    WORKTREE_UNTRACKED_KNOWLEDGE 未跟踪。返回 directory。"""
    committed_files = ("a.txt", ".claude/main-agent.md", WORKTREE_TRACKED_KNOWLEDGE, WORKTREE_STEADY_KNOWLEDGE, WORKTREE_REWRITTEN_KNOWLEDGE,
                       WORKTREE_REVERTED_KNOWLEDGE, WORKTREE_OUTSIDE_FILE)
    build_knowledge_repository(directory, {relative_path: "原文\n" for relative_path in committed_files})
    for relative_path in (WORKTREE_STEADY_KNOWLEDGE, WORKTREE_REWRITTEN_KNOWLEDGE, WORKTREE_REVERTED_KNOWLEDGE, WORKTREE_OUTSIDE_FILE):
        write_selftest_file(directory, relative_path, "基线时已改\n")
    write_selftest_file(directory, WORKTREE_UNTRACKED_KNOWLEDGE, "基线时未跟踪\n")
    return directory


def commit_selftest_files(directory, title, contents_by_path):
    """写 contents_by_path 里的文件，只把它们提交成一条标题为 title 的提交（索引里别的已暂存文件原样留着）；返回这条提交的短哈希。"""
    for relative_path, text in contents_by_path.items():
        write_selftest_file(directory, relative_path, text)
    selftest_git(directory, "add", "--", *contents_by_path)
    selftest_git(directory, "commit", "-q", "-m", title, "--", *contents_by_path)
    return selftest_git(directory, "log", "-1", "--format=%h").strip()


def write_due_knowledge_state(session_directory, repository):
    """在会话目录里落盘一份此刻的基线，上次提醒写成 SELFTEST_DUE_SECONDS_AGO 秒之前：之后起的看门狗接着它比（回报周期从改完仓、起来之后才算），退出时一定满。
    返回落盘那一份的路径；落不了盘抛 RuntimeError（自证当场崩，不当成判红）。"""
    os.makedirs(session_directory, exist_ok=True)
    state_path = os.path.join(session_directory, KNOWLEDGE_STATE_FILE_NAME)
    problem = save_knowledge_state(state_path, repository, repository_snapshot(repository), time.time() - SELFTEST_DUE_SECONDS_AGO, time.time())
    if problem:
        raise RuntimeError(problem)
    return state_path


def run_selftest_shell_command(command, directory):
    """一条给人抄的命令原样写成一行、存进 directory 底下新建的临时目录里的 manual-command.sh，交给 bash 跑那份文件
    （bash 读那一行与粘进 shell 的那一行切词相同；在它自己的进程组里起，不读用户与系统的 git 配置，SELFTEST_SHELL_SECONDS 秒到点停那一组）→ (退出码, 标准错误的第一行, 标准输出)。
    不写成 bash -c <命令>：命令是这一节现出的文本，算单元键的库折不出 -c 的串、整份拒算（scripts/gate/lib_function_units.py 文件头「拒算」inline-script-not-literal）；
    脚本路径是 tempfile 新建的目录加字面尾巴 manual-command.sh，仓里没有以它收尾的脚本，那个库照「不是仓里的脚本」不连、不拒算（同一份文件头「起代码的写法逐种认」python 的命令那一段）。"""
    with tempfile.TemporaryDirectory(prefix="agent-watch-manual-command-", dir=directory) as scratch:
        script = os.path.join(scratch, "manual-command.sh")
        with open(script, "w", encoding="utf-8", errors="surrogateescape") as handle:
            handle.write(command + "\n")
        shell = subprocess.Popen(["bash", script], cwd=directory, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, errors="replace",
                                 env=dict(os.environ, **SELFTEST_GIT_ENVIRONMENT), start_new_session=True)
        try:
            output, error_output = shell.communicate(timeout=SELFTEST_SHELL_SECONDS)
        except subprocess.TimeoutExpired:
            os.killpg(shell.pid, signal.SIGKILL)   # 自己用新会话起的那一个子进程，只停它自己那一组
            shell.communicate()
            return f"超时（{SELFTEST_SHELL_SECONDS} 秒没退出）", "", ""
    return shell.returncode, (error_output.strip().splitlines() or [""])[0], output


def selftest_knowledge_reading(work, check):
    """「知识同步提醒」读仓与出这一节的自检，在本进程里直接调 take_repository_baseline 与 repository_status_section（不起看门狗，没有时序窗口）：
    工作区路径带空格与双引号、新提交文件名带双引号、本机 git 配置不改读数、范围里被 git 忽略的条目、继承了指到别的仓的 git 变量、给人抄的命令照原样跑得起来、
    核手上的文件给命令、只暂存与只改执行位不点名、读哪个仓的三级取法、知识类文件改名、HEAD 往回挪、未跟踪目录的摘要、整次摘要超时走没查成、
    落盘的基线对不上仓或读不出就重新记、同一路径出两条、路径带换行与控制字符、文件名不是 UTF-8、继承了带配置的 git 变量、起来时干净之后只改执行位不点名、
    没查成给列被忽略条目的命令、被忽略条目的标签按发生的事分。每格各造一个小仓（给人抄的命令那一格的放在路径带单引号、$、反引号与空格的目录底下）。"""
    root = os.path.join(work, "knowledge-reading")
    os.makedirs(root)

    def arguments_of(repository, limit=20):
        return argparse.Namespace(repository_report_lines=limit, repository_root=repository, max_minutes=60)

    def baseline_of(repository, limit=20):
        return take_repository_baseline(arguments_of(repository, limit), time.time(), None)

    def section_of(repository, baseline, limit=20):
        return repository_status_section(baseline, arguments_of(repository, limit))[0]

    def has_reminder(lines):
        return any(line.startswith("知识同步提醒：") for line in lines)

    quoted_tracked, spaced_untracked = '.claude/kb/say"hi".md', ".claude/kb/two words.md"
    spaced = build_knowledge_repository(os.path.join(root, "spaced"), {".claude/main-agent.md": "原文\n", quoted_tracked: "原文\n"})
    baseline = baseline_of(spaced)
    write_selftest_file(spaced, quoted_tracked, "原文\n改了\n")
    write_selftest_file(spaced, spaced_untracked, "新知识页\n")
    lines = section_of(spaced, baseline)
    named = dict(named_knowledge_rows(lines))
    check("工作区路径带空格与双引号", quoted_tracked in named and spaced_untracked in named,
          f"基线之后改了文件名带双引号的 {quoted_tracked}、新建了路径带空格的 {spaced_untracked}，两份都应当逐份点名，实际点名 {named}，这一节 {lines}")

    quoted_committed = '.claude/rules/say"hi".md'
    committed = build_knowledge_repository(os.path.join(root, "quote-committed"), {".claude/main-agent.md": "原文\n"})
    baseline = baseline_of(committed)
    commit_selftest_files(committed, "自证新提交：加一份文件名带双引号的规则", {quoted_committed: "规则\n"})
    lines = section_of(committed, baseline)
    named = dict(named_knowledge_rows(lines))
    check("新提交文件名带双引号", named.get(quoted_committed) == COMMITTED_KNOWLEDGE_TAG,
          f"基线之后的新提交加了文件名带双引号的 {quoted_committed}，应当点名、标「{COMMITTED_KNOWLEDGE_TAG}」，实际点名 {named}，这一节 {lines}")

    def configured_reading(label, config_text):
        """在一个新造的小仓里照同一串改动读一遍（config_text 不是 None 时用户的 git 配置换成它）→ 点名、计数与别的路径那几行（仓的路径换成「<仓>」）。"""
        repository = build_knowledge_repository(os.path.join(root, f"config-{len(readings)}"),
                                                {".claude/main-agent.md": "原文\n", ".claude/kb/k.md": "原文\n", ".claude/rules/old-rule.md": "旧规则\n"})
        saved = os.environ.get("GIT_CONFIG_GLOBAL")
        if config_text is not None:
            config_path = os.path.join(root, f"config-{len(readings)}.gitconfig")
            with open(config_path, "w", encoding="utf-8") as handle:
                handle.write(config_text)
            os.environ["GIT_CONFIG_GLOBAL"] = config_path
        try:
            configured_baseline = baseline_of(repository)
            write_selftest_file(repository, ".claude/kb/k.md", "原文\n改了\n")
            write_selftest_file(repository, ".claude/rules/new-rule.md", "新规则\n")
            write_selftest_file(repository, "other.txt", "范围外\n")
            configured_lines = section_of(repository, configured_baseline)
        finally:
            os.environ["GIT_CONFIG_GLOBAL"] = saved
        kept_prefixes = ("  git status --short 此刻", "  基线之后变了的别的路径", f"    {APPEARED_PATH_KIND} ", f"    {CHANGED_PATH_KIND} ", f"    {VANISHED_PATH_KIND} ")
        readings[label] = [line.replace(repository, "<仓>") for line in configured_lines if KNOWLEDGE_ROW_PATTERN.match(line) or line.startswith(kept_prefixes)]
        return readings[label]
    readings = {}
    reference = configured_reading("没有这些配置", None)
    wanted_named = {".claude/kb/k.md", ".claude/rules/new-rule.md"}
    check("本机 git 配置不改读数", {path for path, _ in named_knowledge_rows(reference)} == wanted_named,
          f"没有这些配置时基线之后改了 .claude/kb/k.md、新建了 .claude/rules/new-rule.md，应当点名这两份，实际 {reference}")
    for label, config_text in (("status.branch=true", "[status]\n\tbranch = true\n"), ("status.showUntrackedFiles=no", "[status]\n\tshowUntrackedFiles = no\n"),
                               ("color.status=always", "[color]\n\tstatus = always\n")):
        configured = configured_reading(label, config_text)
        check("本机 git 配置不改读数", configured == reference, f"用户的 git 配置是 {label} 时，点名、计数与别的路径应当与没有这些配置时逐行相同：{reference}，实际 {configured}")

    ignored = build_knowledge_repository(os.path.join(root, "ignored"), {".gitignore": "/.claude/singlefs-ai-sop/\n", ".claude/main-agent.md": "原文\n"})
    for name in ("changed.md", "deleted.md", "steady.md"):
        write_selftest_file(ignored, IGNORED_KNOWLEDGE_DIRECTORY + name, "上游规则原文\n")
    baseline = baseline_of(ignored)
    write_selftest_file(ignored, IGNORED_KNOWLEDGE_DIRECTORY + "changed.md", "上游规则改了\n")
    write_selftest_file(ignored, IGNORED_KNOWLEDGE_DIRECTORY + "added.md", "上游新规则\n")
    os.remove(os.path.join(ignored, IGNORED_KNOWLEDGE_DIRECTORY + "deleted.md"))
    lines = section_of(ignored, baseline)
    named = dict(named_knowledge_rows(lines))
    wanted = {IGNORED_KNOWLEDGE_DIRECTORY + name: IGNORED_KNOWLEDGE_TAG for name in ("added.md", "changed.md", "deleted.md")}
    check("范围里被 git 忽略的条目", named == wanted,
          f".gitignore 忽略整个 .claude/singlefs-ai-sop/，底下的规则改了一份、新加一份、删了一份、一份没动：应当只点名那三份、都标「{IGNORED_KNOWLEDGE_TAG}」，"
          f"即 {wanted}，实际点名 {named}，这一节 {lines}")

    inherited = build_knowledge_repository(os.path.join(root, "inherited"), {".claude/main-agent.md": "原文\n"})
    other = build_knowledge_repository(os.path.join(root, "inherited-other"), {"x.txt": "别的仓\n"}, branch="other-branch")
    saved = os.environ.get("GIT_COMMON_DIR")
    os.environ["GIT_COMMON_DIR"] = os.path.join(other, ".git")
    try:
        snapshot = repository_snapshot(inherited)
        reading = (snapshot.branch, snapshot.head_title)
    except RepositoryUnreadable as error:
        reading = f"没查成：{error}"
    finally:
        if saved is None:
            os.environ.pop("GIT_COMMON_DIR", None)
        else:
            os.environ["GIT_COMMON_DIR"] = saved
    check("继承了指到别的仓的 git 变量", reading == ("selftest-branch", "自证起点提交"),
          f"环境里带着指到别的仓（分支 other-branch）的 GIT_COMMON_DIR 时，应当照样读出这个仓的分支 selftest-branch 与 HEAD 标题「自证起点提交」，实际 {reading}")

    # 小仓放进路径带单引号、$、反引号与空格的目录：引法写坏了（只套一对单引号、里面的不转义），照抄的命令在这里跑不起来、或执行了反引号里的 echo
    special_parent = os.path.join(root, SELFTEST_SPECIAL_ROOT_DIRECTORY)
    commands_repository = build_knowledge_repository(os.path.join(special_parent, "commands"), {".claude/main-agent.md": "原文\n", ".claude/kb/old.md": "原文\n"})
    unread_lines = repository_status_section(RepositoryBaseline(commands_repository, None, "自证造的：起来时没读到", None, time.time(), None),
                                             arguments_of(commands_repository))[0]
    baseline = baseline_of(commands_repository, 1)
    commit_selftest_files(commands_repository, "自证新提交：动一份知识页与一份普通文件", {".claude/kb/one.md": "页\n", "x.txt": "x\n"})
    commit_selftest_files(commands_repository, "自证新提交：再动一份知识页", {".claude/kb/two.md": "页\n"})
    for name in ("y.txt", "z.txt"):
        write_selftest_file(commands_repository, name, "新出现\n")
    capped_lines = section_of(commands_repository, baseline, 1)
    commands = [line[len(COMMAND_INDENT):] for line in unread_lines + capped_lines if line.startswith(COMMAND_INDENT + "git ")]
    check("给人抄的命令照原样跑得起来", len(commands) == 7 and all("git -c core.quotepath=off -C " in command for command in commands),
          f"没查成那一节给 3 条、--repository-report-lines 1 而知识类文件、新提交、别的路径各超上限时给 2、1、1 条，共 7 条，每条单独一行、行首 {len(COMMAND_INDENT)} 个空格、"
          f"带 -c core.quotepath=off；实际 {commands}，没查成那一节 {unread_lines}，余下没列那一节 {capped_lines}")
    for command in commands:
        exit_code, error_line, output = run_selftest_shell_command(command, work)
        check("给人抄的命令照原样跑得起来", exit_code == 0 and SELFTEST_INJECTED_MARKER not in output + error_line,
              f"仓在 {special_parent!r} 底下，「{command}」原样写进一份脚本交给 bash 应当退 0、不执行路径里反引号那一段（输出里没有 {SELFTEST_INJECTED_MARKER}），"
              f"实际 {exit_code}，标准错误第一行「{error_line}」")

    quiet = build_knowledge_repository(os.path.join(root, "quiet"), {".claude/main-agent.md": "原文\n"})
    quiet_lines = section_of(quiet, baseline_of(quiet))
    arrows = [line for line in capped_lines + quiet_lines if line.startswith("   → 同步：")]
    check("核手上的文件给命令", len(arrows) == 2 and all(WHO_TOUCHED_COMMAND in arrow for arrow in arrows),
          f"「→ 同步：」那一行（知识类文件变了的与没变的各一行）应当给查谁动过文件的命令 {WHO_TOUCHED_COMMAND}，实际 {arrows}")

    # 只改执行位的：读状态带 core.fileMode=false，状态码也不变（照旧「M 」）
    for scenario, directory_name, wanted_code, prepare, change in (
            ("只暂存不点名", "steady-stage-only", ("M ",), lambda repository: None, lambda repository: selftest_git(repository, "add", "--", ".claude/kb/k.md")),
            ("只改执行位不点名", "steady-mode-only", ("M ",), lambda repository: selftest_git(repository, "add", "--", ".claude/kb/k.md"),
             lambda repository: os.chmod(os.path.join(repository, ".claude/kb/k.md"), 0o755))):
        repository = build_knowledge_repository(os.path.join(root, directory_name), {".claude/main-agent.md": "原文\n", ".claude/kb/k.md": "原文\n"})
        write_selftest_file(repository, ".claude/kb/k.md", "基线时已改\n")
        prepare(repository)
        baseline = baseline_of(repository)
        change(repository)
        lines = section_of(repository, baseline)
        named = dict(named_knowledge_rows(lines))
        now_code = codes_by_path(repository_snapshot(repository).status_by_path).get(".claude/kb/k.md")
        check(scenario, has_reminder(lines) and ".claude/kb/k.md" not in named and now_code == wanted_code,
              f"基线时已改着的 .claude/kb/k.md 之后状态码是「{status_codes_text(wanted_code)}」而内容一字没变，这一节照打、不该点名它，实际此刻状态码 {now_code}、点名 {named}，这一节 {lines}")

    explicit_root = os.path.join(root, "explicit-repository")
    source_root = os.path.join(root, "source-repository")
    saved = os.environ.get(session_directories.SOURCE_REPOSITORY_VARIABLE)
    picks = {}
    try:
        os.environ[session_directories.SOURCE_REPOSITORY_VARIABLE] = source_root
        picks["给了 --repository-root"] = repository_root_of(argparse.Namespace(repository_root=explicit_root))
        picks["没给、有来源仓根"] = repository_root_of(argparse.Namespace(repository_root=None))
        os.environ.pop(session_directories.SOURCE_REPOSITORY_VARIABLE)
        picks["没给、也没有来源仓根"] = repository_root_of(argparse.Namespace(repository_root=None))
    finally:
        if saved is None:
            os.environ.pop(session_directories.SOURCE_REPOSITORY_VARIABLE, None)
        else:
            os.environ[session_directories.SOURCE_REPOSITORY_VARIABLE] = saved
    wanted = {"给了 --repository-root": explicit_root, "没给、有来源仓根": os.path.normpath(source_root), "没给、也没有来源仓根": REPOSITORY_ROOT}
    check("读哪个仓的三级取法", picks == wanted, f"--repository-root 先于 TASK_SERVER_SOURCE_REPOSITORY 先于本脚本所在的仓，应当是 {wanted}，实际 {picks}")

    renamed = build_knowledge_repository(os.path.join(root, "renamed"), {".claude/main-agent.md": "原文\n", ".claude/rules/old-name.md": "一条规则\n" * 20})
    baseline = baseline_of(renamed)
    selftest_git(renamed, "mv", "--", ".claude/rules/old-name.md", ".claude/rules/new-name.md")
    lines = section_of(renamed, baseline)
    named = dict(named_knowledge_rows(lines))
    check("知识类文件改名", {".claude/rules/old-name.md", ".claude/rules/new-name.md"} <= set(named),
          f"基线之后 git mv 改名一份规则（git status 里是一条改名），原路径与新路径都应当点名，实际点名 {named}，这一节 {lines}")

    backwards = build_knowledge_repository(os.path.join(root, "backwards"), {".claude/main-agent.md": "原文\n"})
    old_head, old_short = selftest_git(backwards, "log", "-1", "--format=%H %h").split()
    new_short = commit_selftest_files(backwards, "自证新提交：往回挪之前的那一条", {"b.txt": "b\n"})
    baseline = baseline_of(backwards)
    selftest_git(backwards, "update-ref", "HEAD", old_head)
    lines = section_of(backwards, baseline)
    wanted_line = f"  基线之后的新提交：没有新提交（HEAD 从 {new_short} 移到了 {old_short}，不是往前走）"
    check("HEAD 往回挪", wanted_line in lines, f"基线之后 HEAD 从 {new_short} 往回挪到 {old_short}，应当有一行「{wanted_line.strip()}」，这一节 {lines}")

    drafts = build_knowledge_repository(os.path.join(root, "drafts"), {".claude/main-agent.md": "原文\n", ".claude/kb/tracked.md": "原文\n"})
    write_selftest_file(drafts, ".claude/kb/drafts/notes.md", "草稿\n")
    baseline = baseline_of(drafts)
    write_selftest_file(drafts, ".claude/kb/drafts/second-note.md", "又一份草稿\n")
    lines = section_of(drafts, baseline)
    named = dict(named_knowledge_rows(lines))
    check("未跟踪目录的摘要", ".claude/kb/drafts/" in named,
          f"基线时就未跟踪的 .claude/kb/drafts/（git status 写成一条）底下又多一份、状态码没变，目录的摘要变了，应当点名 .claude/kb/drafts/，实际点名 {named}，这一节 {lines}")

    slow = build_knowledge_repository(os.path.join(root, "slow"), {".claude/main-agent.md": "原文\n", ".claude/kb/k.md": "原文\n"})
    write_selftest_file(slow, ".claude/kb/k.md", "改着\n")
    saved = os.environ.get(DIGEST_TIMEOUT_VARIABLE)
    os.environ[DIGEST_TIMEOUT_VARIABLE] = "0"
    try:
        baseline = baseline_of(slow)
        lines = section_of(slow, baseline)
    finally:
        if saved is None:
            os.environ.pop(DIGEST_TIMEOUT_VARIABLE, None)
        else:
            os.environ[DIGEST_TIMEOUT_VARIABLE] = saved
    check("整次摘要超时走没查成", baseline.snapshot is None and "算内容摘要超过 0 秒" in (baseline.unreadable_reason or "")
          and any(line.startswith("仓的现状：没查成：") and "算内容摘要超过 0 秒" in line for line in lines),
          f"算内容摘要的总时限设成 0 秒、范围里有一份改着的文件时，基线应当记成读不出、这一节写「没查成：…算内容摘要超过 0 秒」，"
          f"实际基线 {'读到了' if baseline.snapshot is not None else '没读到：' + str(baseline.unreadable_reason)}，这一节 {lines}")

    first = build_knowledge_repository(os.path.join(root, "state-first"), {".claude/main-agent.md": "原文\n"})
    second = build_knowledge_repository(os.path.join(root, "state-second"), {".claude/main-agent.md": "原文\n"})
    other_state = os.path.join(root, "state-other-repository", KNOWLEDGE_STATE_FILE_NAME)
    os.makedirs(os.path.dirname(other_state))
    save_knowledge_state(other_state, first, repository_snapshot(first), time.time() - SELFTEST_DUE_SECONDS_AGO, time.time())
    garbage_state = os.path.join(root, "state-garbage", KNOWLEDGE_STATE_FILE_NAME)
    write_selftest_file(os.path.dirname(garbage_state), KNOWLEDGE_STATE_FILE_NAME, "不是 JSON\n")
    for label, state_path, wanted_text in (("记的是别的仓", other_state, "记的是别的仓"), ("不是 JSON", garbage_state, "读不出")):
        restarted = take_repository_baseline(arguments_of(second), time.time(), state_path)
        lines = section_of(second, restarted)
        note_lines = [line for line in lines if line.startswith("  基线：")]
        check("落盘的基线对不上仓或读不出就重新记", restarted.snapshot is not None and len(note_lines) == 1 and wanted_text in note_lines[0] and "重新记了" in note_lines[0],
              f"会话目录里落盘的那一份{label}时，这一只应当此刻重新记、这一节的「基线：」那一行写明「{wanted_text}」「重新记了」，实际 {note_lines}，这一节 {lines}")

    duplicate = build_knowledge_repository(os.path.join(root, "duplicate-path"), {".claude/main-agent.md": "原文\n", ".claude/rules/r.md": "规则\n",
                                                                               ".claude/rules/keep.md": "留着\n"})
    baseline = baseline_of(duplicate)
    selftest_git(duplicate, "rm", "-q", "--cached", "--", ".claude/rules/r.md")
    duplicate_status = repository_snapshot(duplicate).status_by_path
    lines = section_of(duplicate, baseline)
    count_line = "  git status --short 此刻：已暂存 1 个、未暂存 0 个、未跟踪 1 个（共 1 个路径"
    check("同一路径出两条", sorted(duplicate_status.get((".claude/rules/r.md",), ())) == ["??", "D "] and status_counts(duplicate_status) == (1, 0, 1)
          and any(line.startswith(count_line) for line in lines),
          f"git rm --cached 一份规则之后 git status 里同一路径出两条（「D 」与「??」），两码都应当留着、计数照原始条目数（已暂存 1、未跟踪 1），"
          f"这一节有一行「{count_line.strip()}…」，实际读成 {duplicate_status}、计数 {status_counts(duplicate_status)}，这一节 {lines}")

    newline_repository = build_knowledge_repository(os.path.join(root, "newline-path"), {".claude/main-agent.md": "原文\n", ".claude/kb/tracked.md": "原文\n"})
    baseline = baseline_of(newline_repository)
    crafted = ".claude/kb/x\n   → 同步：知识类文件没变，不用重读.md"
    write_selftest_file(newline_repository, crafted, "新页\n")
    lines = section_of(newline_repository, baseline)
    printed = "\n".join(lines).splitlines()
    arrow_lines = [line for line in printed if line.startswith("   → 同步：")]
    check("路径带换行与控制字符", len(arrow_lines) == 1 and visible_text(crafted) in dict(named_knowledge_rows(printed)) and "\\n" in visible_text(crafted),
          f"新建一份文件名里带换行、换行之后像「→ 同步：」那一行的知识类文件 {crafted!r}：打出来只该有一行「→ 同步：」、那一份照转义之后的写法（换行写成 \\n）点名一行，"
          f"实际像出路的 {len(arrow_lines)} 行，点名 {named_knowledge_rows(printed)}")

    raw_relative = b".claude/kb/\xff.md"
    raw_repository = build_knowledge_repository(os.path.join(root, "non-utf8-path"), {".claude/main-agent.md": "原文\n", ".claude/kb/tracked.md": "原文\n"})
    raw_path = os.path.join(os.fsencode(raw_repository), raw_relative)
    with open(raw_path, "wb") as handle:
        handle.write("草稿\n".encode("utf-8"))
    baseline = baseline_of(raw_repository)
    with open(raw_path, "wb") as handle:
        handle.write("草稿\n改了\n".encode("utf-8"))
    lines = section_of(raw_repository, baseline)
    wanted_name = visible_text(os.fsdecode(raw_relative))
    check("文件名不是 UTF-8", wanted_name in dict(named_knowledge_rows(lines)),
          f"基线时就未跟踪、文件名带一个不是 UTF-8 的字节（0xff）的知识类文件之后改了内容，应当照原字节找到它、点名 {wanted_name!r}，"
          f"实际点名 {named_knowledge_rows(lines)!r}")

    configured = build_knowledge_repository(os.path.join(root, "config-parameters"), {".claude/main-agent.md": "原文\n", ".claude/kb/tracked.md": "原文\n"})
    excludes_path = os.path.join(root, "config-parameters-excludes")
    with open(excludes_path, "w", encoding="utf-8") as handle:
        handle.write("new.md\n")
    with selftest_environment_set({"GIT_CONFIG_PARAMETERS": f"'core.excludesfile'='{excludes_path}'"}):
        baseline = baseline_of(configured)
        write_selftest_file(configured, ".claude/kb/new.md", "新页\n")
        lines = section_of(configured, baseline)
    named = dict(named_knowledge_rows(lines))
    check("继承了带配置的 git 变量", named.get(".claude/kb/new.md") == UNCOMMITTED_KNOWLEDGE_TAG,
          f"环境里带着 GIT_CONFIG_PARAMETERS（core.excludesfile 指到一份忽略 new.md 的文件）时，它不该继承：基线之后新建的 .claude/kb/new.md 应当标「{UNCOMMITTED_KNOWLEDGE_TAG}」，"
          f"实际点名 {named}，这一节 {lines}")

    chmodded = build_knowledge_repository(os.path.join(root, "chmod-clean-at-baseline"), {".claude/main-agent.md": "原文\n", ".claude/kb/k.md": "原文\n"})
    baseline = baseline_of(chmodded)
    os.chmod(os.path.join(chmodded, ".claude/kb/k.md"), 0o755)
    lines = section_of(chmodded, baseline)
    named = dict(named_knowledge_rows(lines))
    check("起来时干净、之后只改执行位不点名", has_reminder(lines) and ".claude/kb/k.md" not in named,
          f"基线时干净的 .claude/kb/k.md 之后只改了执行位（内容一字没变），这一节照打、不该点名它，实际点名 {named}，这一节 {lines}")

    unchecked = build_knowledge_repository(os.path.join(root, "unchecked-commands"), {".gitignore": "/.claude/singlefs-ai-sop/\n", ".claude/main-agent.md": "原文\n",
                                                                                     ".claude/kb/tracked.md": "原文\n"})
    ignored_rule = IGNORED_KNOWLEDGE_DIRECTORY + "rule.md"
    write_selftest_file(unchecked, ignored_rule, "上游规则\n")
    write_selftest_file(unchecked, ".claude/kb/draft.md", "草稿\n")   # 未跟踪的一份知识类文件：到点要算它的摘要，总时限 0 秒时走「没查成」
    baseline = baseline_of(unchecked)
    write_selftest_file(unchecked, ignored_rule, "上游规则改了\n")
    with selftest_environment_set({DIGEST_TIMEOUT_VARIABLE: "0"}):
        lines = section_of(unchecked, baseline)
    unchecked_commands = [line[len(COMMAND_INDENT):] for line in lines if line.startswith(COMMAND_INDENT + "git ")]
    readings = [run_selftest_shell_command(command, work) for command in unchecked_commands]
    check("没查成给列被忽略条目的命令", any(line.startswith("仓的现状：没查成：") for line in lines) and readings
          and all(exit_code == 0 for exit_code, _, _ in readings) and any(ignored_rule in output for _, _, output in readings),
          f"范围里被 git 忽略的 {ignored_rule} 改了、到点算摘要超时走「没查成」时，下面给的命令照原样跑都该退 0、其中一条列得出 {ignored_rule}，"
          f"实际命令 {unchecked_commands}，退出码 {[exit_code for exit_code, _, _ in readings]}，这一节 {lines}")

    unignored = build_knowledge_repository(os.path.join(root, "ignored-to-untracked"), {".gitignore": "/.claude/singlefs-ai-sop/\n", ".claude/main-agent.md": "原文\n"})
    for name in ("a.md", "b.md"):
        write_selftest_file(unignored, IGNORED_KNOWLEDGE_DIRECTORY + name, "上游规则\n")
    baseline = baseline_of(unignored)
    write_selftest_file(unignored, ".gitignore", "")
    unignored_named = dict(named_knowledge_rows(section_of(unignored, baseline)))
    newly = build_knowledge_repository(os.path.join(root, "untracked-to-ignored"), {".claude/main-agent.md": "原文\n", ".claude/kb/tracked.md": "原文\n"})
    write_selftest_file(newly, ".claude/kb/notes.md", "草稿\n")
    baseline = baseline_of(newly)
    write_selftest_file(newly, ".git/info/exclude", "/.claude/kb/notes.md\n")
    newly_named = dict(named_knowledge_rows(section_of(newly, baseline)))
    wanted_unignored = {IGNORED_KNOWLEDGE_DIRECTORY + name: UNIGNORED_KNOWLEDGE_TAG for name in ("a.md", "b.md")}
    check("被忽略条目的标签按发生的事分", unignored_named == wanted_unignored and newly_named == {".claude/kb/notes.md": NEWLY_IGNORED_KNOWLEDGE_TAG},
          f"只改忽略规则、内容没变：.gitignore 去掉那一行之后底下两份应当标「{UNIGNORED_KNOWLEDGE_TAG}」（{wanted_unignored}），实际 {unignored_named}；"
          f"基线时未跟踪的 .claude/kb/notes.md 之后写进 .git/info/exclude 应当标「{NEWLY_IGNORED_KNOWLEDGE_TAG}」，实际 {newly_named}")


@contextlib.contextmanager
def selftest_environment_set(values):
    """自证在本进程里临时设几个环境变量（{名: 值}），出来时照原样放回（原来没有的删掉）。"""
    saved = {name: os.environ.get(name) for name in values}
    os.environ.update(values)
    try:
        yield
    finally:
        for name, value in saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value


def selftest_reminder_chain(repository, state_path, alerts, restart_delay, base):
    """一串看门狗在本进程里照 run() 的次序走，时刻是 base 加分钟数（假时钟，不等）：每只起来时 take_repository_baseline，定时回报的时刻照 report_deadline_of，
    退出时刻取它与下一次告警（alerts，分钟）里早的那个（起来时就已过的，第一次整查之后当场退），退出时 emit_knowledge_sync_section，隔 restart_delay 分钟起下一只；
    走到 SELFTEST_CHAIN_HORIZON_MINUTES 分钟、或走满 SELFTEST_CHAIN_MAX_WATCHERS 只为止 → (带了这一节的退出时刻（分钟）, 每次带的那一节, 走了几只)。"""
    arguments = argparse.Namespace(repository_report_lines=REPOSITORY_REPORT_LINES_DEFAULT, repository_root=repository, max_minutes=SELFTEST_CHAIN_MAX_MINUTES)
    reminders, sections, start, watchers = [], [], 0.0, 0
    while start < SELFTEST_CHAIN_HORIZON_MINUTES and watchers < SELFTEST_CHAIN_MAX_WATCHERS:
        watchers += 1
        started_at = base + start * 60
        baseline = take_repository_baseline(arguments, started_at, state_path)
        deadline_minutes = (report_deadline_of(started_at, baseline, arguments)[0] - base) / 60
        next_alert = next((moment for moment in alerts if moment > start), None)
        on_alert = next_alert is not None and next_alert < deadline_minutes
        moment = max(start, next_alert if on_alert else deadline_minutes)
        lines = emit_knowledge_sync_section(baseline, arguments, on_alert, now=base + moment * 60, output=io.StringIO())
        if lines:
            reminders.append(moment)
            sections.append(lines)
        start = moment + restart_delay
    return reminders, sections, watchers


def reminder_chain_verdict(reminders, watchers, restart_delay):
    """一串的读数 → (最长间隔（分钟，从 0 起、到 SELFTEST_CHAIN_HORIZON_MINUTES 止）, 走到头之前带了几次, 这一串对不对)：
    最长间隔不超过 --max-minutes 加重起的间隔，次数不多于每满 --max-minutes 一次，没走满 SELFTEST_CHAIN_MAX_WATCHERS 只。"""
    within = [moment for moment in reminders if moment <= SELFTEST_CHAIN_HORIZON_MINUTES]
    points = [0.0] + within + [SELFTEST_CHAIN_HORIZON_MINUTES]
    longest = max(later - earlier for earlier, later in zip(points, points[1:]))
    bounded = (longest <= SELFTEST_CHAIN_MAX_MINUTES + restart_delay + SELFTEST_CHAIN_SLACK_MINUTES
               and len(within) <= SELFTEST_CHAIN_HORIZON_MINUTES // SELFTEST_CHAIN_MAX_MINUTES + 1 and watchers < SELFTEST_CHAIN_MAX_WATCHERS)
    return longest, len(within), bounded


def selftest_knowledge_state_handling(work, check):
    """「知识同步提醒」落盘与按时带的自检，在本进程里直接调 take_repository_baseline、report_deadline_of 与 emit_knowledge_sync_section（时刻用假的、不等）：
    定时回报离上次提醒满了就退、起来时就查不成也按时提醒、之后查不成每满一次才提醒、两只读同一份只提醒一次、先打出这一节再落盘、落盘留下的旧临时文件起来时删掉、
    落盘的时刻不是有限数或晚于此刻就重新记、记基线时与带这一节时的异常照没查成写、链接目标不是 UTF-8 也落得了盘、取不到 git 现列的变量时用写死的。每格各造一个小仓与会话目录。"""
    root = os.path.join(work, "knowledge-state")
    os.makedirs(root)
    base = time.time()

    def arguments_of(repository):
        return argparse.Namespace(repository_report_lines=REPOSITORY_REPORT_LINES_DEFAULT, repository_root=repository, max_minutes=SELFTEST_CHAIN_MAX_MINUTES)

    def state_path_of(name):
        directory = os.path.join(root, f"session-{name}")
        os.makedirs(directory)
        return os.path.join(directory, KNOWLEDGE_STATE_FILE_NAME)

    def minutes(value):
        return base + value * 60

    def stored_record(state_path):
        with open(state_path, encoding="utf-8", errors="surrogateescape") as handle:
            return json.load(handle)

    chained = build_knowledge_repository(os.path.join(root, "chain"), {".claude/main-agent.md": "原文\n"})
    reminders, _, watchers = selftest_reminder_chain(chained, state_path_of("chain"), [59.9], 5.0, base)
    longest, count, bounded = reminder_chain_verdict(reminders, watchers, 5.0)
    check("定时回报离上次提醒满了就退", bounded,
          f"--max-minutes {SELFTEST_CHAIN_MAX_MINUTES}、第 59.9 分钟一次告警退出、每次退出隔 5 分钟重起，{SELFTEST_CHAIN_HORIZON_MINUTES} 分钟里两次提醒的最长间隔"
          f"应当不超过 {SELFTEST_CHAIN_MAX_MINUTES} 加 5 分钟、次数不多于每满一次，实际带这一节的时刻 {reminders}、最长间隔 {longest:.1f} 分钟、{count} 次、走了 {watchers} 只")

    unreadable = build_knowledge_repository(os.path.join(root, "unreadable-from-start"), {".claude/main-agent.md": "原文\n"})
    with selftest_environment_set({GIT_EXECUTABLE_VARIABLE: os.path.join(root, "no-such-git")}):
        reminders, sections, watchers = selftest_reminder_chain(unreadable, state_path_of("unreadable-from-start"), [30.0 * index for index in range(1, 7)], 0.1, base)
    longest, count, bounded = reminder_chain_verdict(reminders, watchers, 0.1)
    check("起来时就查不成也按时提醒", bounded and count >= 2 and all(any(line.startswith("仓的现状：没查成：") for line in lines) for lines in sections),
          f"git 一直起不来（会话目录里先没有落盘的那一份）、每 30 分钟一次告警退出、隔 0.1 分钟重起，{SELFTEST_CHAIN_HORIZON_MINUTES} 分钟里每满 {SELFTEST_CHAIN_MAX_MINUTES} 分钟"
          f"应当带一次写「没查成」的这一节，实际带这一节的时刻 {reminders}、最长间隔 {longest:.1f} 分钟、{count} 次、走了 {watchers} 只")

    later = build_knowledge_repository(os.path.join(root, "unreadable-later"), {".claude/main-agent.md": "原文\n", ".claude/kb/k.md": "原文\n"})
    write_selftest_file(later, ".claude/kb/k.md", "改着\n")   # 范围里有一份改着的：到点要算它的摘要
    later_state = state_path_of("unreadable-later")
    problem = save_knowledge_state(later_state, later, repository_snapshot(later), base, base)
    with selftest_environment_set({DIGEST_TIMEOUT_VARIABLE: "0"}):
        reminders, sections, watchers = selftest_reminder_chain(later, later_state, [10.0 * index for index in range(1, 19)], 0.1, base)
    longest, count, bounded = reminder_chain_verdict(reminders, watchers, 0.1)
    check("之后查不成每满一次才提醒", problem is None and bounded and count >= 2 and all(any(line.startswith("仓的现状：没查成：") for line in lines) for lines in sections),
          f"先落盘了一份查成的基线、之后算摘要一直超时（总时限 0 秒）、每 10 分钟一次告警退出，带了「没查成」也把上次提醒记成此刻、快照不换，"
          f"{SELFTEST_CHAIN_HORIZON_MINUTES} 分钟里应当只在每满 {SELFTEST_CHAIN_MAX_MINUTES} 分钟时带一次，实际落盘 {problem or '成了'}，带这一节的时刻 {reminders}、"
          f"最长间隔 {longest:.1f} 分钟、{count} 次、走了 {watchers} 只")

    pair = build_knowledge_repository(os.path.join(root, "two-watchers"), {".claude/main-agent.md": "原文\n"})
    pair_state = state_path_of("two-watchers")
    save_knowledge_state(pair_state, pair, repository_snapshot(pair), minutes(-120), minutes(-120))
    first = take_repository_baseline(arguments_of(pair), minutes(-1), pair_state)
    second = take_repository_baseline(arguments_of(pair), minutes(-0.5), pair_state)
    first_lines = emit_knowledge_sync_section(first, arguments_of(pair), False, now=minutes(0), output=io.StringIO())
    second_lines = emit_knowledge_sync_section(second, arguments_of(pair), False, now=minutes(0.5), output=io.StringIO())
    check("两只读同一份只提醒一次", bool(first_lines) and not second_lines,
          f"两只看门狗起来时读到同一份上次提醒在两小时前的落盘，前一只在第 0 分钟带了这一节、换成此刻，后一只在第 0.5 分钟退出时重读落盘的那一份、离上次提醒才 0.5 分钟，"
          f"不该再带，实际前一只 {first_lines[:1]}，后一只 {second_lines[:1]}")

    printing = build_knowledge_repository(os.path.join(root, "print-before-save"), {".claude/main-agent.md": "原文\n"})
    printing_state = state_path_of("print-before-save")
    save_knowledge_state(printing_state, printing, repository_snapshot(printing), minutes(-120), minutes(-120))
    printing_baseline = take_repository_baseline(arguments_of(printing), minutes(0), printing_state)

    class StoppedWhilePrinting(Exception):
        """自证造的：打这一节时被停（写标准输出出错）。"""

    class StoppingOutput(io.StringIO):
        def write(self, text):
            raise StoppedWhilePrinting("自证造的：打这一节时被停")
    try:
        emit_knowledge_sync_section(printing_baseline, arguments_of(printing), False, now=minutes(1), output=StoppingOutput())
        outcome = "打完了"
    except StoppedWhilePrinting:
        outcome = "打的时候被停"
    stored_reminded_at = stored_record(printing_state)["reminded_at"]
    check("先打出这一节再落盘", outcome == "打的时候被停" and stored_reminded_at == minutes(-120),
          f"离上次提醒满了、打这一节时被停：这一节没送到，落盘的上次提醒应当照旧（{minutes(-120):.0f}），下一只接着点名，"
          f"实际 {outcome}、落盘的上次提醒是 {stored_reminded_at!r}")

    stale_state = state_path_of("stale-temporaries")
    stale_directory = os.path.dirname(stale_state)
    stale_names = {"old": f"{KNOWLEDGE_STATE_TEMPORARY_PREFIX}selftestold{KNOWLEDGE_STATE_TEMPORARY_SUFFIX}",
                   "fresh": f"{KNOWLEDGE_STATE_TEMPORARY_PREFIX}selftestfresh{KNOWLEDGE_STATE_TEMPORARY_SUFFIX}"}
    for label, name in stale_names.items():
        write_selftest_file(stale_directory, name, "{")
    old_moment = time.time() - 2 * KNOWLEDGE_STATE_TEMPORARY_MAX_AGE_SECONDS
    os.utime(os.path.join(stale_directory, stale_names["old"]), (old_moment, old_moment))
    take_repository_baseline(arguments_of(pair), minutes(1), stale_state)
    left = sorted(name for name in os.listdir(stale_directory) if name.endswith(KNOWLEDGE_STATE_TEMPORARY_SUFFIX))
    check("落盘留下的旧临时文件起来时删掉", left == [stale_names["fresh"]] and os.path.isfile(stale_state),
          f"会话目录里有一份两小时没动、一份刚写的落盘临时文件时，起来时应当只删旧的那一份，实际剩 {left}")

    edited = build_knowledge_repository(os.path.join(root, "hand-edited"), {".claude/main-agent.md": "原文\n"})
    for label, value in (("NaN", float("nan")), ("Infinity", float("inf")), ("-Infinity", float("-inf")), ("year-2100", 4102444800.0), ("string-zero", "0")):
        edited_state = state_path_of(f"hand-edited-{label}")
        save_knowledge_state(edited_state, edited, repository_snapshot(edited), minutes(-120), minutes(-120))
        record = stored_record(edited_state)
        record["reminded_at"] = value
        with open(edited_state, "w", encoding="utf-8") as handle:
            json.dump(record, handle, ensure_ascii=False)
        restarted = take_repository_baseline(arguments_of(edited), minutes(0), edited_state)
        check("落盘的时刻不是有限数或晚于此刻就重新记", "读不出" in (restarted.note or "") and "重新记了" in (restarted.note or "") and restarted.reminded_at == minutes(0),
              f"落盘那一份的上次提醒被改成 {label}（{value!r}）时应当当读不出、此刻重新记（上次提醒记成这一只起来的时刻），"
              f"实际「基线：」{restarted.note!r}、上次提醒 {restarted.reminded_at!r}")

    raising = build_knowledge_repository(os.path.join(root, "baseline-exception"), {".claude/main-agent.md": "原文\n"})
    try:
        with selftest_environment_set({GIT_TIMEOUT_VARIABLE: "不是数"}):
            raised_baseline = take_repository_baseline(arguments_of(raising), minutes(0), state_path_of("baseline-exception"))
        raised_lines = repository_status_section(raised_baseline, arguments_of(raising), minutes(61))[0]
        outcome = f"返回了，原因 {raised_baseline.unreadable_reason!r}"
    except Exception as error:   # 弄坏开关 knowledgebaselineexceptionescapes 下外层不接，这里接住判红
        raised_baseline, raised_lines, outcome = None, [], f"抛出 {type(error).__name__}"
    check("记基线时的异常照没查成记", raised_baseline is not None and raised_baseline.snapshot is None and "ValueError" in (raised_baseline.unreadable_reason or "")
          and any(line.startswith("仓的现状：没查成：") and "ValueError" in line for line in raised_lines),
          f"每条 git 的时限写成不是数（float 抛 ValueError）时，记基线不该往外抛：基线记成没查成、这一节写「没查成：…ValueError…」，实际 {outcome}，这一节 {raised_lines}")

    section_raising = build_knowledge_repository(os.path.join(root, "section-exception"), {".claude/main-agent.md": "原文\n"})
    section_baseline = take_repository_baseline(arguments_of(section_raising), minutes(0), None)
    try:
        with selftest_environment_set({GIT_TIMEOUT_VARIABLE: "不是数"}):
            section_lines = emit_knowledge_sync_section(section_baseline, arguments_of(section_raising), False, now=minutes(61), output=io.StringIO())
        outcome = "返回了"
    except Exception as error:   # 弄坏开关 knowledgesectionexceptionescapes 下外层不接，这里接住判红
        section_lines, outcome = [], f"抛出 {type(error).__name__}"
    check("带这一节时的异常照没查成写", any(line.startswith("仓的现状：没查成：") and "ValueError" in line for line in section_lines),
          f"基线读到了、到点时每条 git 的时限写成不是数（float 抛 ValueError），带这一节不该往外抛、写「没查成：…ValueError…」，实际 {outcome}，这一节 {section_lines}")

    linked = build_knowledge_repository(os.path.join(root, "non-utf8-link"), {".claude/main-agent.md": "原文\n", ".claude/kb/seed.md": "原文\n"})
    os.symlink(b"\xff\xfe-target", os.path.join(os.fsencode(linked), b".claude", b"kb", b"odd-link"))
    linked_state = state_path_of("non-utf8-link")
    linked_baseline = take_repository_baseline(arguments_of(linked), minutes(0), linked_state)
    reloaded = load_knowledge_state(linked_state, linked, minutes(0))[0]
    linked_digest = (linked_baseline.snapshot.digest_by_path if linked_baseline.snapshot else {}).get(".claude/kb/odd-link", "")
    leftovers = sorted(name for name in os.listdir(os.path.dirname(linked_state)) if name.endswith(KNOWLEDGE_STATE_TEMPORARY_SUFFIX))
    check("链接目标不是 UTF-8 也落得了盘", linked_digest.startswith("链接 ") and reloaded is not None and reloaded.snapshot is not None
          and reloaded.snapshot.digest_by_path.get(".claude/kb/odd-link") == linked_digest and not leftovers,
          f"范围里一条目标不是合法 UTF-8 的符号链接：摘要记它指向的路径、落盘与读回不出错、不留临时文件，"
          f"实际摘要 {linked_digest!r}、「基线：」{linked_baseline.note!r}、读回 {'读到了' if reloaded else '读不出'}、剩下的临时文件 {leftovers}")

    listed = git_local_environment_variables(os.path.join(root, "no-such-git"))
    check("取不到 git 现列的变量时用写死的", set(listed) == set(GIT_LOCAL_ENVIRONMENT_FALLBACK) and len(listed) == 15
          and {"GIT_CONFIG", "GIT_CONFIG_PARAMETERS", "GIT_CONFIG_COUNT", "GIT_COMMON_DIR", "GIT_DIR"} <= set(listed),
          f"git 起不来、git rev-parse --local-env-vars 取不成时，不继承的变量应当是写死的那 15 个（含 GIT_CONFIG 这一组），实际 {listed}")


def selftest_repository_report(work, failures, watch_runs, probed_processes):
    """「知识同步提醒」一节的自检（文件头「知识同步提醒」）：先核状态码计数，再照 selftest_knowledge_reading 在本进程里核读仓那一半、照 selftest_knowledge_state_handling
    核落盘与按时带那一半；
    再起看门狗子进程（--agents 与 --processes-only 两条路，--repository-root 指到临时目录里的小仓，每只一个自己的会话目录）。要比改动的那几只先在会话目录里落盘一份
    上次提醒在两小时前的基线、再改仓、改完才起（回报周期从起来之后才算，不靠固定窗口），核退出时带这一节且内容对（提醒句排第一、新提交与改动、新提交动了知识类文件与点名次序、
    工作区改了没提交、基线时已改而之后又改了内容、范围外同样情形不点名、知识类文件没变、什么都没变、清单上限、--processes-only）；查不成（不是仓、git 超时、git 起不来）写「没查成」、
    写 0 不打、告警退出而没满不带、告警退出且已满带这一节并把基线换成此刻（--agents 与 --processes-only 两路，后一路以重起看门狗那一行收尾）、
    每条 git 的时限写成不是数时照常到点退 4 并写「没查成」、两只看门狗之间变的被后一只点名、未跟踪知识类目录里有命名管道照样退出、全部交回退 0 与 report 不打。返回查了几格。
    弄坏开关：文件头开关表里 repository 起头与 knowledge 起头的那几条。"""
    checks = 0
    results = {}

    def check(scenario, condition, detail):
        nonlocal checks
        checks += 1
        if not condition:
            failures.append(f"知识同步提醒（{scenario}）：{detail}")

    def shown(key):
        code, output, error_output = results[key]
        return f"实际退出码 {code}，输出：{output[-900:]}，标准错误：{error_output[-300:]}"

    def arrow_line_of(key):
        return next((line for line in results[key][1].splitlines() if line.startswith("   → 同步：")), "")

    def reminder_line_of(key):
        return next((line for line in results[key][1].splitlines() if line.startswith("知识同步提醒：")), "")

    def line_after_report_of(key):
        """「定时回报：」那一行的下一行。"""
        lines = results[key][1].splitlines()
        report_at = next((index for index, line in enumerate(lines) if line.startswith("定时回报：")), None)
        return "" if report_at is None or report_at + 1 >= len(lines) else lines[report_at + 1]

    def named_files_of(key):
        """输出里逐份点名的知识类文件，按出现的次序 → [(路径, 来源标签)]。"""
        return named_knowledge_rows(results[key][1].splitlines())

    parsed = parse_status_porcelain(" M a.txt\0M  b.txt\0MM c.txt\0?? d.txt\0R  f.txt\0e.txt\0UU g.txt\0")
    check("状态码计数", len(parsed) == 6 and parsed.get(("e.txt", "f.txt")) == ("R ",) and status_counts(parsed) == (4, 3, 1),
          "六条（未暂存、已暂存、两边都有改动、未跟踪、改名、冲突）应当读成 6 个路径元组、改名那一条是（e.txt, f.txt）、已暂存 4 个（b.txt、c.txt、改名那一条、冲突那一条）、"
          f"未暂存 3 个（a.txt、c.txt、冲突那一条）、未跟踪 1 个，实际 {parsed}、计数 {status_counts(parsed)}")
    saved_git_environment = {name: os.environ.get(name) for name in SELFTEST_GIT_ENVIRONMENT}
    os.environ.update(SELFTEST_GIT_ENVIRONMENT)   # 本进程里直接读小仓的几格同样不读用户与系统的 git 配置；收尾放回
    environment = dict(os.environ)
    watchers = {}
    sessions = {}
    group_sleeper = subprocess.Popen(["sleep", "300"], start_new_session=True)   # --processes-only 要有一个盯得住的进程，否则起来就退 0
    probed_processes.append(group_sleeper)

    def session_of(key):
        """这一只看门狗自己的会话目录（落盘的基线各管各的）：三个合成子 agent——在跑的 healthy、工具调用 20 分钟的 longtool、交回了的 finished。"""
        if key not in sessions:
            directory = os.path.join(work, f"repository-session-{key}")
            write_transcript(directory, "healthy", [bash_use(0.2, "t1", "cargo build", "m1")], {"agentType": "experiment-runner", "description": "在跑的"})
            write_transcript(directory, "longtool", [bash_use(20, "t1", "cargo test --release", "m1")], {"agentType": "experiment-runner", "description": "工具调用 20 分钟"})
            write_transcript(directory, "finished", [bash_use(20, "t1", "ls", "m1"), bash_result(19, "t1"), *handback_records(18.5, "h1", "m2"),
                                                     record_at(18, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m3")],
                             {"agentType": "experiment-runner", "description": "跑完的"})
            sessions[key] = directory
        return sessions[key]

    def launch(key, repository, extra_arguments=(), agents="healthy", processes=False, extra_environment=None, minutes="0.03", session_key=None):
        watch_runs.append(["repository-section", key])
        command = [sys.executable, ENTRY_SCRIPT_PATH, "watch"]
        if processes:
            command += ["--processes-only", "--process-root-pid", str(os.getpid())]
        else:
            command += ["--agents", agents, "--process-root-pid", "0"]
        command += ["--session-dir", session_of(session_key or key), "--interval-seconds", "1", "--max-minutes", minutes, "--repository-root", repository, *extra_arguments]
        watchers[key] = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, encoding="utf-8", errors="replace",
                                         env=dict(environment, **(extra_environment or {})), start_new_session=True)

    def collect(key, timeout_seconds):
        """等那一只退出、记下结局；到点没退的停它自己那一组，退出码记成「超时」，照原判据判红。"""
        watcher = watchers[key]
        try:
            output, error_output = watcher.communicate(timeout=max(timeout_seconds, 1))
            results[key] = (watcher.returncode, output, error_output)
        except subprocess.TimeoutExpired:
            os.killpg(watcher.pid, signal.SIGKILL)   # 自己用新会话起的那一只，只停它自己那一组
            output, error_output = watcher.communicate()
            results[key] = (f"超时（{timeout_seconds:.0f} 秒没退出，已停它自己那一组）", output, error_output)

    try:
        selftest_knowledge_reading(work, check)
        selftest_knowledge_state_handling(work, check)
        repositories = {name: build_selftest_repository(os.path.join(work, f"repository-{name}"))
                        for name in ("changed", "rules", "quiet", "capped", "processes", "due-alert", "due-alert-processes")}
        repositories["worktree"] = build_selftest_worktree_repository(os.path.join(work, "repository-worktree"))
        repositories["gap"] = build_knowledge_repository(os.path.join(work, "repository-gap"), {".claude/main-agent.md": "原文\n", GAP_KNOWLEDGE_FILE: "原文\n"})
        repositories["fifo"] = build_knowledge_repository(os.path.join(work, "repository-fifo"), {".claude/main-agent.md": "原文\n"})
        write_selftest_file(repositories["fifo"], ".claude/kb/pipes/notes.md", "草稿\n")
        os.mkfifo(os.path.join(repositories["fifo"], ".claude/kb/pipes/channel"))   # 被跟踪的 .claude/ 底下未跟踪的知识类目录里：一份普通文件（git 才列这个目录）与一个没有写端的命名管道
        empty_directory = os.path.join(work, "not-a-repository")
        os.makedirs(empty_directory)
        sleeping_git = os.path.join(work, "git-that-sleeps")
        with open(sleeping_git, "w", encoding="utf-8") as handle:
            handle.write("#!/bin/sh\nexec sleep 30\n")
        os.chmod(sleeping_git, 0o755)
        due_states = {key: write_due_knowledge_state(session_of(key), repositories[key])
                      for key in ("changed", "rules", "worktree", "capped", "processes", "due-alert", "due-alert-processes")}
        due_reminded_at = load_knowledge_state(due_states["due-alert"], repositories["due-alert"], time.time())[0].reminded_at
        # 基线落了盘：现在改仓，改完才起看门狗。changed：一条新提交、改两个文件（一个基线时就已暂存）、新建一个、删掉基线时就未跟踪的那一个
        changed_head = commit_selftest_files(repositories["changed"], CHANGED_COMMIT_TITLE, {"plain.txt": "普通文件\n"})
        write_selftest_file(repositories["changed"], "a.txt", "第一行\n第二行\n")
        write_selftest_file(repositories["changed"], "staged.txt", "已暂存\n又改了\n")
        write_selftest_file(repositories["changed"], "fresh.txt", "新出现\n")
        os.remove(os.path.join(repositories["changed"], "old-untracked.txt"))
        commit_selftest_files(repositories["rules"], KNOWLEDGE_COMMIT_TITLE, {**SELFTEST_KNOWLEDGE_FILE_SAMPLES, "docs/other.md": "不是知识类文件\n"})
        # worktree：改一份基线时干净的、重写两份基线时已改与未跟踪的（状态码不变）、范围外的也重写一份（状态码不变）、撤回一份基线时已改的；
        # 主 agent 的规矩先提交一条、再改一次没提交；基线时已改之后不动的那一份不碰
        worktree_path = repositories["worktree"]
        write_selftest_file(worktree_path, WORKTREE_TRACKED_KNOWLEDGE, "原文\n改了\n")
        write_selftest_file(worktree_path, WORKTREE_REWRITTEN_KNOWLEDGE, "基线时已改\n之后又改了\n")
        write_selftest_file(worktree_path, WORKTREE_UNTRACKED_KNOWLEDGE, "基线时未跟踪\n之后又改了\n")
        write_selftest_file(worktree_path, WORKTREE_OUTSIDE_FILE, "基线时已改\n之后又改了\n")
        write_selftest_file(worktree_path, WORKTREE_REVERTED_KNOWLEDGE, "原文\n")
        commit_selftest_files(worktree_path, MAIN_AGENT_COMMIT_TITLE, {".claude/main-agent.md": "新内容\n"})
        write_selftest_file(worktree_path, ".claude/main-agent.md", "新内容\n再改了\n")
        for (title, file_name), knowledge_file in zip(CAPPED_COMMITS, (*CAPPED_KNOWLEDGE_FILES, None)):
            capped_head = commit_selftest_files(repositories["capped"], title, {file_name: title + "\n", **({knowledge_file: title + "\n"} if knowledge_file else {})})
        for file_name in CAPPED_NEW_FILES:
            write_selftest_file(repositories["capped"], file_name, "新出现\n")
        processes_head = commit_selftest_files(repositories["processes"], PROCESSES_COMMIT_TITLE, {"plain.txt": "普通文件\n"})
        write_selftest_file(repositories["processes"], "fresh.txt", "新出现\n")
        launched_at = time.time()
        launch("changed", repositories["changed"])
        launch("rules", repositories["rules"])
        launch("worktree", repositories["worktree"])
        launch("capped", repositories["capped"], ["--repository-report-lines", "1"])
        launch("processes", repositories["processes"], processes=True)
        launch("quiet", repositories["quiet"])
        launch("not-repository", empty_directory, extra_environment={"GIT_CEILING_DIRECTORIES": os.path.realpath(work)})
        launch("git-timeout", empty_directory, extra_environment={GIT_EXECUTABLE_VARIABLE: sleeping_git, GIT_TIMEOUT_VARIABLE: "1"})
        launch("git-missing", empty_directory, extra_environment={GIT_EXECUTABLE_VARIABLE: os.path.join(work, "no-such-git")})
        launch("lines-zero", repositories["quiet"], ["--repository-report-lines", "0"])
        launch("alert", repositories["quiet"], agents="longtool", minutes=SELFTEST_ALERT_WATCH_MINUTES)
        launch("alert-processes", repositories["quiet"], ["--root-free-gib-alert", "128"], processes=True, minutes=SELFTEST_ALERT_WATCH_MINUTES,
               extra_environment={ROOT_FREE_BYTES_VARIABLE: str(40 * BYTES_PER_GIBIBYTE)})
        launch("due-alert", repositories["due-alert"], agents="longtool", minutes=SELFTEST_ALERT_WATCH_MINUTES)
        launch("due-alert-processes", repositories["due-alert-processes"], ["--root-free-gib-alert", "128"], processes=True, minutes=SELFTEST_ALERT_WATCH_MINUTES,
               extra_environment={ROOT_FREE_BYTES_VARIABLE: str(40 * BYTES_PER_GIBIBYTE)})
        launch("git-timeout-garbage", repositories["quiet"], extra_environment={GIT_TIMEOUT_VARIABLE: "不是数"})
        launch("gap-first", repositories["gap"], agents="longtool", minutes=SELFTEST_ALERT_WATCH_MINUTES)
        launch("finished", repositories["quiet"], agents="finished")
        launch("fifo", repositories["fifo"])
        for key in list(watchers):
            if key != "fifo":
                collect(key, 120)
        collect("fifo", launched_at + SELFTEST_FIFO_WATCH_SECONDS - time.time())
        # 前一只（gap-first）已经退出：现在改那一份知识类文件，再在同一个会话目录里起后一只
        write_selftest_file(repositories["gap"], GAP_KNOWLEDGE_FILE, "原文\n两只之间改了\n")
        launch("gap-second", repositories["gap"], session_key="gap-first")
        collect("gap-second", 120)
        watch_runs.append(["repository-section", "report"])
        report_run = subprocess.run([sys.executable, ENTRY_SCRIPT_PATH, "report", "--agents", "healthy", "--session-dir", session_of("report"),
                                     "--process-root-pid", "0", "--repository-root", repositories["quiet"]],
                                    capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=120, env=environment)
        results["report"] = (report_run.returncode, report_run.stdout, report_run.stderr)
    finally:
        for process in [*watchers.values(), group_sleeper]:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)   # 每一只都是自己用新会话起的，只停它自己那一组
            process.wait()
        for name, value in saved_git_environment.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value

    changed_output = results["changed"][1]
    section_at = changed_output.find("知识同步提醒：")
    check("有新提交与改动", results["changed"][0] == 4 and 0 <= changed_output.find("定时回报：") < section_at,
          f"到点的定时回报应当退出码 4，「定时回报：」之后跟着「知识同步提醒：」这一节，{shown('changed')}")
    check("有新提交与改动", f"  分支 selftest-branch，HEAD {changed_head} {CHANGED_COMMIT_TITLE}" in changed_output,
          f"应当写分支名与 HEAD 的短哈希、标题，{shown('changed')}")
    check("有新提交与改动", "基线之后的新提交：1 条" in changed_output and f"    {changed_head} {CHANGED_COMMIT_TITLE}" in changed_output,
          f"应当点出基线之后新进的那一条提交（短哈希 标题），{shown('changed')}")
    check("有新提交与改动", "已暂存 1 个、未暂存 2 个、未跟踪 1 个（共 3 个路径" in changed_output,
          f"git status 此刻应当是已暂存 1 个（staged.txt）、未暂存 2 个（a.txt、staged.txt）、未跟踪 1 个（fresh.txt），{shown('changed')}")
    check("有新提交与改动", all(text in changed_output for text in ("新出现 2 个、状态变了 1 个、不在了 1 个", "新出现 [ M] a.txt", "新出现 [??] fresh.txt",
                                                                    "状态变了 [A ]→[AM] staged.txt", "不在了 [??] old-untracked.txt")),
          f"基线之后变了的别的路径应当分新出现、状态变了、不在了三类逐条列出，{shown('changed')}")
    knowledge_paths = list(SELFTEST_KNOWLEDGE_FILE_SAMPLES)
    rules_output = results["rules"][1]
    rules_files = named_files_of("rules")
    check("新提交动了知识类文件", dict(rules_files) == {path: COMMITTED_KNOWLEDGE_TAG for path in knowledge_paths} and len(rules_files) == len(knowledge_paths),
          f"新提交动了范围里每个条目底下各一份（共 {len(knowledge_paths)} 份）、另有一份不在范围里的 docs/other.md，"
          f"应当逐份点名那 {len(knowledge_paths)} 份、都标「{COMMITTED_KNOWLEDGE_TAG}」、不点名 docs/other.md，实际点名：{rules_files}，{shown('rules')}")
    check("新提交动了知识类文件", f"知识类文件变过 {len(knowledge_paths)} 份" in reminder_line_of("rules") and f"重读上面点名的 {len(knowledge_paths)} 份" in arrow_line_of("rules"),
          f"提醒句应当写变过 {len(knowledge_paths)} 份、末尾的出路应当写重读上面点名的 {len(knowledge_paths)} 份，{shown('rules')}")
    check("新提交动了知识类文件", 0 <= rules_output.find(f"    {knowledge_paths[-1]}（") < rules_output.find("仓的现状（"),
          f"逐份点名的清单应当排在「仓的现状（」之前，{shown('rules')}")
    check("点名次序", [path for path, _ in rules_files] == knowledge_paths,
          f"点名次序应当是里程碑、决策索引、欠账表、CLAUDE.md、.claude/main-agent.md、.claude/agent-common.md，其余按路径排，即 {knowledge_paths}，实际 {[path for path, _ in rules_files]}，{shown('rules')}")
    check("什么都没变", results["quiet"][0] == 4 and "基线之后的新提交：没有新提交" in results["quiet"][1]
          and "基线之后变了的别的路径：没变" in results["quiet"][1],
          f"基线之后什么都没变时应当写「没有新提交」「变了的别的路径：没变」，{shown('quiet')}")
    check("什么都没变", "已暂存 1 个、未暂存 0 个、未跟踪 1 个（共 2 个路径" in results["quiet"][1],
          f"基线时就有的一个已暂存、一个未跟踪的路径应当照现状数进计数，{shown('quiet')}")
    for key in ("quiet", "changed"):
        check("知识类文件没变", "知识类文件没变，没有要重读的" in reminder_line_of(key) and "知识类文件没变，不用重读" in arrow_line_of(key) and not named_files_of(key),
              f"基线之后一份知识类文件都没变时，提醒句应当写「知识类文件没变，没有要重读的」、出路写「知识类文件没变，不用重读」、不逐份点名（{key}），{shown(key)}")
    for key in ("changed", "quiet", "rules", "worktree"):
        check("提醒句排第一", line_after_report_of(key).startswith("知识同步提醒：到点了，该同步项目最新变动与状态了。"),
              f"「定时回报：」那一行的下一行应当是写给主对话的提醒句，先于点名的文件与分支、HEAD、计数这些（{key}），{shown(key)}")
    worktree_output = results["worktree"][1]
    worktree_files = dict(named_files_of("worktree"))
    check("工作区改了没提交", results["worktree"][0] == 4 and worktree_files.get(WORKTREE_TRACKED_KNOWLEDGE) == UNCOMMITTED_KNOWLEDGE_TAG,
          f"基线时干净、之后改了没提交的 {WORKTREE_TRACKED_KNOWLEDGE} 应当点名、标「{UNCOMMITTED_KNOWLEDGE_TAG}」，实际点名 {worktree_files}，{shown('worktree')}")
    check("工作区改了没提交", worktree_files.get(".claude/main-agent.md") == BOTH_KNOWLEDGE_TAG,
          f"新提交动过、之后又改了没提交的 .claude/main-agent.md 应当标「{BOTH_KNOWLEDGE_TAG}」，实际点名 {worktree_files}，{shown('worktree')}")
    check("工作区改了没提交", worktree_files.get(WORKTREE_REVERTED_KNOWLEDGE) == DROPPED_KNOWLEDGE_TAG,
          f"基线时已改、之后撤回成提交里的内容的 {WORKTREE_REVERTED_KNOWLEDGE} 应当点名、标「{DROPPED_KNOWLEDGE_TAG}」，实际点名 {worktree_files}，{shown('worktree')}")
    check("起来时已改内容又变", worktree_files.get(WORKTREE_REWRITTEN_KNOWLEDGE) == UNCOMMITTED_KNOWLEDGE_TAG
          and worktree_files.get(WORKTREE_UNTRACKED_KNOWLEDGE) == UNCOMMITTED_KNOWLEDGE_TAG,
          f"基线时已改（未暂存）与未跟踪、之后又改了内容而状态码没变的 {WORKTREE_REWRITTEN_KNOWLEDGE}、{WORKTREE_UNTRACKED_KNOWLEDGE} 应当点名、标「{UNCOMMITTED_KNOWLEDGE_TAG}」，"
          f"实际点名 {worktree_files}，{shown('worktree')}")
    check("起来时已改内容又变", WORKTREE_STEADY_KNOWLEDGE not in worktree_files,
          f"基线时已改、之后内容没动的 {WORKTREE_STEADY_KNOWLEDGE} 不该点名，实际点名 {worktree_files}，{shown('worktree')}")
    check("范围外不点名", results["worktree"][0] == 4 and "知识同步提醒：" in worktree_output and WORKTREE_OUTSIDE_FILE not in worktree_output,
          f"范围外的 {WORKTREE_OUTSIDE_FILE} 基线时已改、之后又改了内容而状态码没变，只比状态码、不该出现在这一节里，{shown('worktree')}")
    capped_output = results["capped"][1]
    capped_newest_title, capped_oldest_title = CAPPED_COMMITS[-1][0], CAPPED_COMMITS[0][0]
    check("清单超过上限", "基线之后的新提交：3 条" in capped_output and f"    {capped_head} {capped_newest_title}" in capped_output
          and capped_oldest_title not in capped_output and "…余 2 条没列" in capped_output,
          f"--repository-report-lines 1、基线之后新进三条提交时，应当只列最新的一条、余下报「余 2 条没列」，{shown('capped')}")
    check("清单超过上限", all(text in capped_output for text in ("新出现 3 个、状态变了 0 个、不在了 0 个", f"新出现 [??] {CAPPED_NEW_FILES[0]}", "…余 2 个没列"))
          and CAPPED_NEW_FILES[1] not in capped_output,
          f"--repository-report-lines 1、基线之后新出现三个未跟踪文件时，应当只列第一个、余下报「余 2 个没列」，{shown('capped')}")
    capped_files = named_files_of("capped")
    check("清单超过上限", [path for path, _ in capped_files] == [CAPPED_KNOWLEDGE_FILES[0]] and "知识类文件变过 2 份" in reminder_line_of("capped")
          and "…余 1 份没列" in capped_output and CAPPED_KNOWLEDGE_FILES[1] not in capped_output,
          f"--repository-report-lines 1、基线之后新提交动了两份知识类文件时，应当只点名第一份 {CAPPED_KNOWLEDGE_FILES[0]}、提醒句写变过 2 份、余下报「余 1 份没列」，"
          f"实际点名 {capped_files}，{shown('capped')}")
    processes_output = results["processes"][1]
    processes_restart_hint = "   → 接着盯就用 run_in_background 再起一个看门狗：bash scripts/agents/watch.sh --processes"
    check("--processes-only 有新提交与改动",
          results["processes"][0] == 4 and 0 <= processes_output.find("定时回报：") < processes_output.find("仓的现状（") < processes_output.find(processes_restart_hint)
          and 0 <= processes_output.find("知识同步提醒：") < processes_output.find(processes_restart_hint),
          f"--processes-only 的定时回报应当退出码 4，「定时回报：」之后依次是这一节（含「知识同步提醒：」与「仓的现状（」）与「{processes_restart_hint.strip()}」，{shown('processes')}")
    check("--processes-only 有新提交与改动", "基线之后的新提交：1 条" in processes_output and f"    {processes_head} {PROCESSES_COMMIT_TITLE}" in processes_output,
          f"--processes-only 的这一节应当点出基线之后新进的那一条提交，{shown('processes')}")
    check("--processes-only 有新提交与改动", "新出现 [??] fresh.txt" in processes_output,
          f"--processes-only 的这一节应当列出基线之后新出现的 fresh.txt，{shown('processes')}")
    check("仓根不是仓", results["not-repository"][0] == 4 and "仓的现状：没查成：" in results["not-repository"][1] and "退出码 128" in results["not-repository"][1],
          f"--repository-root 指到不是仓的目录时，定时回报仍退出码 4，这一节写「没查成：」与 git 退出码和它的错误输出，{shown('not-repository')}")
    check("仓根不是仓", git_command_line(empty_directory, "status", "--short") in results["not-repository"][1],
          f"没查成时应当给手动看的命令，{shown('not-repository')}")
    check("仓根不是仓", "知识同步提醒：" in results["not-repository"][1] and "没查成，先自己照下面的命令看" in reminder_line_of("not-repository"),
          f"没查成时提醒句照样写，并写明哪些知识类文件变了这一趟没查成，{shown('not-repository')}")
    check("git 超时", results["git-timeout"][0] == 4 and "仓的现状：没查成：" in results["git-timeout"][1] and "超过 1 秒没有返回" in results["git-timeout"][1],
          f"git 超过时限（自证设成 1 秒）时，定时回报仍退出码 4，这一节写「没查成：」与哪条命令超了多久，{shown('git-timeout')}")
    check("git 起不来", results["git-missing"][0] == 4 and "仓的现状：没查成：" in results["git-missing"][1] and "git 起不来" in results["git-missing"][1],
          f"git 起不来时，定时回报仍退出码 4，这一节写「没查成：git 起不来」，{shown('git-missing')}")
    check("写 0 不打", results["lines-zero"][0] == 4 and "知识同步提醒" not in results["lines-zero"][1] and "仓的现状" not in results["lines-zero"][1],
          f"--repository-report-lines 0 时定时回报应当退出码 4、不打这一节，{shown('lines-zero')}")
    for key, label in (("alert", "（--agents）"), ("alert-processes", "（--processes-only）"), ("gap-first", "（--agents）：两只之间那一格的前一只")):
        check("告警退出而没满不带", results[key][0] == 3 and "⚠️" in results[key][1] and "知识同步提醒" not in results[key][1] and "仓的现状" not in results[key][1],
              f"有告警叫醒（退出码 3）而离上次提醒没满 {SELFTEST_ALERT_WATCH_MINUTES} 分钟的那一趟不带这一节{label}，{shown(key)}")
    due_output = results["due-alert"][1]
    check("告警退出且已满带这一节", results["due-alert"][0] == 3
          and 0 <= due_output.find("⚠️") < due_output.find("知识同步提醒：") < due_output.find("   → 接着盯就用 run_in_background 再起一个看门狗："),
          f"会话目录里落盘的基线上次提醒在两小时前、有告警叫醒（退出码 3）时，告警之后、重起看门狗那一行之前带这一节，{shown('due-alert')}")
    due_processes_output = results["due-alert-processes"][1]
    check("--processes-only 告警退出且已满带这一节", results["due-alert-processes"][0] == 3
          and 0 <= due_processes_output.find("⚠️") < due_processes_output.find("知识同步提醒：") < due_processes_output.find("   → 同步：")
          < due_processes_output.find(processes_restart_hint),
          f"--processes-only 按根分区余量过低告警退出（退出码 3）、离上次提醒满了时，告警之后带这一节，「→ 同步：」那一行要它照的「{processes_restart_hint.strip()}」排在最后，"
          f"{shown('due-alert-processes')}")
    garbage_output = results["git-timeout-garbage"][1]
    check("记基线时的异常照没查成记", results["git-timeout-garbage"][0] == 4 and "仓的现状：没查成：" in garbage_output and "ValueError" in garbage_output,
          f"每条 git 的时限写成不是数时，看门狗照常盯、到点退出码 4，这一节写「没查成：…ValueError…」，{shown('git-timeout-garbage')}")
    state_after = load_knowledge_state(due_states["due-alert"], repositories["due-alert"], time.time())[0]
    check("打过把基线换成此刻", state_after is not None and state_after.reminded_at >= launched_at > due_reminded_at,
          f"带了这一节之后，落盘的基线里上次提醒的时刻应当换成退出那一刻（不早于这一批看门狗起来的时刻），"
          f"落盘时是 {due_reminded_at:.0f}，此刻读到 {state_after.reminded_at if state_after else '读不出'}，{shown('due-alert')}")
    gap_files = dict(named_files_of("gap-second"))
    check("两只之间变了的被后一只点名", results["gap-second"][0] == 4 and GAP_KNOWLEDGE_FILE in gap_files,
          f"前一只退出之后、后一只起来之前改的 {GAP_KNOWLEDGE_FILE}，后一只接着落盘的基线比、到点应当点名，"
          f"前一只退出码 {results['gap-first'][0]}，后一只点名 {gap_files}，{shown('gap-second')}")
    check("未跟踪知识类目录里有命名管道", results["fifo"][0] == 4 and "知识同步提醒：" in results["fifo"][1],
          f"被跟踪的 .claude/ 底下未跟踪的 .claude/kb/pipes/ 里有一份普通文件与一个没有写端的命名管道时，记基线与到点都不该卡住，"
          f"应当 {SELFTEST_FIFO_WATCH_SECONDS} 秒之内到点退出码 4 并带这一节，{shown('fifo')}")
    check("全部交回与 report 不打", results["finished"][0] == 0 and "被看的子 agent 全部交回或被停" in results["finished"][1]
          and "知识同步提醒" not in results["finished"][1] and "仓的现状" not in results["finished"][1],
          f"被看的子 agent 全部交回（退出码 0）的那一趟不打这一节，{shown('finished')}")
    check("全部交回与 report 不打", results["report"][0] == 0 and "知识同步提醒" not in results["report"][1] and "仓的现状" not in results["report"][1],
          f"report 看一眼的那一趟不打这一节，{shown('report')}")
    return checks
