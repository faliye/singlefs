"""长活声明的判法：预计挂钟超过 2 小时的活，要说清分片单元、续跑入口、机器与额度取自多机配置与现查三样；分不了的写一行缺哪一段。
判据：.claude/singlefs-ai-sop/rules/test-script-discipline.md「用上现有的硬件加速」与「判定可复用：按输入存，按前缀剪」。
三处用这一份：research/scripts/detached-run.sh 的 start（--wall-clock over-2h）与 long-run-check 判被起脚本的文件头；
.claude/hooks/runner-dispatch-guard.sh 判派 experiment-designer、experiment-runner 的提示；research/scripts/host-dispatch.py 判派法占几台（started_script、script_header_declaration、judge）。

声明的四个键（脚本文件头 / 派发提示的写法）：
  shards       # long-run-shards: <分片单元：按什么切、几片怎么定>                         分片：<…>
  resume       # long-run-resume: <续跑入口：同一条命令重起只补没做完的，怎么认做完的>       续跑：<…>
  hosts        # long-run-hosts: <机器与额度怎么取>                                         多机：<…>
  unshardable  # long-run-unshardable: <缺哪一段、为什么>                                   分不了：<…>
派发提示另有一行「预计挂钟：2 小时内」或「预计挂钟：超过 2 小时」（后面可接逗号与推的依据）。

判法（judge）：
  - 三样里值不空、不是「无 / 没有 / -」、不是 <…> 占位的算写了。
  - hosts 写了的：要点名机器清单的读法 --emit-hosts（research/scripts/layer0-shard-configuration-check.sh）与现查余量的 host-resources.py，
    不许写死额度（HARD_CODED_QUOTA：数字后面跟线程、核、台、K/M/G/T，或 --threads N、-j N、--memory-cap N、--shard k/n、THREADS=N 这类）。
  - 三样缺的，要有 unshardable 一行：至少 UNSHARDABLE_MINIMUM_CHARACTERS 个字，并逐个点名缺的那几样的词（分片 / 续跑 / 多机）。
脚本文件头：#! 之后连着的注释块（# 起头的行与空行，到第一行别的为止）；同一个键写了几行，按次序接起来。
被起的脚本（started_script）：命令的词里，含 / 或以 .sh / .py 结尾、按命令的当前目录解得到普通文件、以 .sh / .py 结尾或开头是 #! 的，
  跳过包装（WRAPPER_SCRIPTS）之后的第一份；一份都没有的（bash -c '…' 一长串、直接起 cargo 或编出来的二进制）判不出。

用法：
  python3 lib_long_run.py script <命令的当前目录> -- <命令与参数…>
      判被起脚本的声明：齐了退 0、stdout 打一行；不齐退 1、stderr 逐条 ✗ 加 → 出路；用法错与弄坏开关写错退 2
  python   import lib_long_run（把 .claude/hooks 放进 sys.path），调 prompt_declaration、judge

弄坏开关 LIB_LONG_RUN_BREAK=<项>（逗号连几个；只给用它的两份的自证判红用，认不出的值抛 ValueError）。打开后红在哪几格写在两份的文件头：
  partial-accepted            三样写了一样就当齐了
  hosts-unchecked             hosts 一行不看内容（不核两份现查、不核写死的额度）
  unshardable-unchecked       unshardable 一行不看字数、不看点没点名缺的那几样
  script-unresolved-accepted  判不出被起的脚本时当成齐了（只 detached-run.sh 用）
  wrappers-not-skipped        不跳过包装，第一份脚本就算被起的那份（只 detached-run.sh 用）
"""
import os
import re
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
import lib_shell_words as shell_words  # noqa: E402

BREAK_VARIABLE = "LIB_LONG_RUN_BREAK"
KNOWN_BREAK_MODES = ("partial-accepted", "hosts-unchecked", "unshardable-unchecked", "script-unresolved-accepted", "wrappers-not-skipped")
WALL_CLOCK_THRESHOLD_HOURS = 2
WALL_CLOCK_WITHIN, WALL_CLOCK_OVER = "within-2h", "over-2h"
REQUIRED_KEYS = ("shards", "resume", "hosts")
KEY_WORDS = {"shards": "分片", "resume": "续跑", "hosts": "多机"}   # unshardable 一行要点名的词
HEADER_KEYS = {"long-run-shards": "shards", "long-run-resume": "resume", "long-run-hosts": "hosts", "long-run-unshardable": "unshardable"}
HEADER_SPELLING = {key: f"# {name}:" for name, key in HEADER_KEYS.items()}
PROMPT_LABELS = {"分片": "shards", "续跑": "resume", "多机": "hosts", "分不了": "unshardable"}
PROMPT_SPELLING = {key: f"{label}：" for label, key in PROMPT_LABELS.items()}
WALL_CLOCK_LABEL = "预计挂钟"
HOST_READS = ("--emit-hosts", "host-resources.py")
UNSHARDABLE_MINIMUM_CHARACTERS = 8
NONE_VALUES = {"", "无", "无。", "没有", "-", "—", "空", "none"}
PLACEHOLDER = re.compile(r"^<[^>\n]*>$")
HARD_CODED_QUOTA = re.compile(
    r"(?<![A-Za-z0-9_./\-])\d+(?:\.\d+)?\s*(?:个线程|线程|核|台|[KMGTkmgt]i?[Bb]?(?![A-Za-z]))"
    r"|(?:--threads|--jobs|--workers|--lanes|--memory-cap|--memory-max|(?<![A-Za-z\-])-j)[=\s]\s*\d+"
    r"|--shard[=\s]\s*\d+/\d+"
    r"|\b[A-Z_]*(?:THREADS|JOBS|MEMORY|CAP)[A-Z_]*=\d+")
LINE_START = r"^[ \t>*_\-]*"
WALL_CLOCK_LINE = re.compile(LINE_START + WALL_CLOCK_LABEL + r"[* \t]*[：:][* \t]*(.*)$", re.M)
WALL_CLOCK_WITHIN_VALUE = re.compile(r"^2\s*小时内")
WALL_CLOCK_OVER_VALUE = re.compile(r"^超过\s*2\s*小时")
# 包一层再起命令的脚本：跳过它们找被起的那份（lib_shell_words 认的两个带一个参数的包装，加上把命令挪到别处跑、清场后跑、脱离会话跑的三个）
WRAPPER_SCRIPTS = set(shell_words.WRAPPERS_TAKING_ONE_ARGUMENT) | {"multi-host-run.sh", "gpu-quiesce-run.sh", shell_words.DETACHED_RUN_SCRIPT_NAME}
SCRIPT_SUFFIXES = (".sh", ".py")
HEADER_READ_BYTES = 65536


def breaks():
    modes = {mode for mode in os.environ.get(BREAK_VARIABLE, "").split(",") if mode}
    unknown = modes - set(KNOWN_BREAK_MODES)
    if unknown:
        raise ValueError(f"{BREAK_VARIABLE} 认不出 {'、'.join(sorted(unknown))}（认得的：{'、'.join(KNOWN_BREAK_MODES)}）")
    return modes


def is_written(value):
    if value is None:
        return False
    text = value.strip()
    return text not in NONE_VALUES and not PLACEHOLDER.match(text)


def judge(declared, spelling):
    """declared：键 → 值（没写的键不在里面）；spelling：键 → 那一行的写法（报问题时用）。交回问题的列表，空的是齐了。"""
    modes = breaks()
    problems = []
    written = {key for key in REQUIRED_KEYS if is_written(declared.get(key))}
    if "partial-accepted" in modes and written:
        written = set(REQUIRED_KEYS)
    hosts = declared.get("hosts")
    if "hosts" in written and is_written(hosts) and "hosts-unchecked" not in modes:
        unnamed = [read for read in HOST_READS if read not in hosts]
        if unnamed:
            problems.append(f"{spelling['hosts']} 一行没点名 {'、'.join(unnamed)}：机器清单只从 layer0-shard-configuration-check.sh --emit-hosts 读，每台的线程与内存从 host-resources.py --json 现算")
        quotas = [match.group(0).strip() for match in HARD_CODED_QUOTA.finditer(hosts)]
        if quotas:
            problems.append(f"{spelling['hosts']} 一行写死了额度：{'、'.join(quotas)}（台数、线程、内存要按多机配置与现查算，不写常量）")
    missing = [key for key in REQUIRED_KEYS if key not in written]
    if missing:
        unshardable = declared.get("unshardable")
        names = "、".join(f"{KEY_WORDS[key]}（{spelling[key]}）" for key in missing)
        if not is_written(unshardable):
            problems.append(f"缺 {names}，也没有 {spelling['unshardable']} 一行写明缺哪一段")
        elif "unshardable-unchecked" not in modes:
            text = unshardable.strip()
            if len(text) < UNSHARDABLE_MINIMUM_CHARACTERS:
                problems.append(f"{spelling['unshardable']} 一行只有 {len(text)} 个字，至少 {UNSHARDABLE_MINIMUM_CHARACTERS} 个字写明缺哪一段、为什么")
            unnamed = [KEY_WORDS[key] for key in missing if KEY_WORDS[key] not in text]
            if unnamed:
                problems.append(f"{spelling['unshardable']} 一行没点名缺的 {'、'.join(unnamed)}（缺的 {names} 每一样都要在这一行里写出它的词）")
    return problems


def script_header_declaration(path):
    """脚本文件头注释块里的四个键：键 → 值（同一个键几行按次序用空格接起来）。读不了抛 OSError。"""
    with open(path, "rb") as handle:
        text = handle.read(HEADER_READ_BYTES).decode("utf-8", errors="replace")
    declared = {}
    for index, line in enumerate(text.split("\n")):
        stripped = line.strip()
        if index == 0 and stripped.startswith("#!"):
            continue
        if stripped and not stripped.startswith("#"):
            break
        match = re.match(r"#\s*(long-run-[a-z]+)\s*:\s*(.*)$", stripped)
        if match and match.group(1) in HEADER_KEYS:
            key = HEADER_KEYS[match.group(1)]
            declared[key] = (declared[key] + " " + match.group(2)).strip() if key in declared else match.group(2).strip()
    return declared


def starts_with_shebang(path):
    try:
        with open(path, "rb") as handle:
            return handle.read(2) == b"#!"
    except OSError:
        return False


def started_script(words, directory):
    """命令的词里被起的那份脚本的绝对路径；判不出交 None（写法在文件头「被起的脚本」）。"""
    skip_wrappers = "wrappers-not-skipped" not in breaks()
    for word in words:
        if not ("/" in word or word.endswith(SCRIPT_SUFFIXES)):
            continue
        path = os.path.normpath(word if os.path.isabs(word) else os.path.join(directory, word))
        if not os.path.isfile(path):
            continue
        if not (word.endswith(SCRIPT_SUFFIXES) or starts_with_shebang(path)):
            continue
        if skip_wrappers and os.path.basename(os.path.realpath(path)) in WRAPPER_SCRIPTS:
            continue
        return path
    return None


def prompt_declaration(prompt):
    """派发提示里的预计挂钟与四行：交回 (挂钟, 声明)。挂钟是 None（没写那一行）、WALL_CLOCK_WITHIN、WALL_CLOCK_OVER，或认不出时那一行的原值。"""
    wall_clock = None
    match = WALL_CLOCK_LINE.search(prompt)
    if match:
        value = match.group(1).strip()
        wall_clock = (WALL_CLOCK_WITHIN if WALL_CLOCK_WITHIN_VALUE.match(value)
                      else WALL_CLOCK_OVER if WALL_CLOCK_OVER_VALUE.match(value) else value)
    declared = {}
    for label, key in PROMPT_LABELS.items():
        line = re.search(LINE_START + label + r"[* \t]*[：:][* \t]*(.*)$", prompt, re.M)
        if line:
            declared[key] = line.group(1).strip()
    return wall_clock, declared


def script_main(directory, words):
    """命令行 script 子命令：退 0 齐了，1 不齐或判不出。"""
    script = started_script(words, directory)
    if script is None:
        if "script-unresolved-accepted" in breaks():
            print("  ✓ 长活声明：判不出被起的脚本，照齐了算（弄坏开关 script-unresolved-accepted）")
            return 0
        print("  ✗ 长活声明：判不出被起的是哪份脚本（-- 之后没有一个词是脚本文件；bash -c '…' 一长串、直接起 cargo 或编出来的二进制都判不出），"
              "超过 2 小时的活读不到分片、续跑、多机三样的声明", file=sys.stderr)
        print("     → 怎么办：把这条命令写进一份脚本，文件头写 # long-run-shards: / # long-run-resume: / # long-run-hosts:（分不了的写 # long-run-unshardable: 点名缺哪一段），"
              "改成 -- bash <那份脚本> 起；确实 2 小时内跑得完的写 --wall-clock within-2h", file=sys.stderr)
        return 1
    try:
        declared = script_header_declaration(script)
    except OSError as error:
        print(f"  ✗ 长活声明：读不了被起的脚本 {script}：{error}", file=sys.stderr)
        print("     → 怎么办：核对 -- 之后那份脚本的路径与读权限再起", file=sys.stderr)
        return 1
    problems = judge(declared, HEADER_SPELLING)
    if problems:
        for problem in problems:
            print(f"  ✗ 长活声明（{script}）：{problem}", file=sys.stderr)  # gate-lint:detail
        print(f"     → 怎么办：在 {script} 文件头的注释块里补齐（判据 .claude/singlefs-ai-sop/rules/test-script-discipline.md「用上现有的硬件加速」「判定可复用：按输入存，按前缀剪」）："
              "# long-run-shards: 分片单元；# long-run-resume: 同一条命令重起只补没做完的；"
              "# long-run-hosts: 台数从 layer0-shard-configuration-check.sh --emit-hosts 读、额度从 host-resources.py --json 现算；"
              "分不了的写 # long-run-unshardable: <点名缺的分片 / 续跑 / 多机与为什么，至少 8 个字>", file=sys.stderr)
        return 1
    how = "；".join(f"{KEY_WORDS.get(key, '分不了')}：{declared[key]}" for key in (*REQUIRED_KEYS, "unshardable") if is_written(declared.get(key)))
    print(f"  ✓ 长活声明齐了（{script}，判了 {len(declared)} 个键）：{how}")
    return 0


def main(argv):
    try:
        breaks()
    except ValueError as error:
        print(f"  ✗ {error}", file=sys.stderr)
        print(f"     → 怎么办：去掉 {BREAK_VARIABLE}，或只写认得的项", file=sys.stderr)
        return 2
    if len(argv) >= 3 and argv[0] == "script" and argv[2] == "--" and len(argv) > 3:
        return script_main(argv[1], argv[3:])
    print("  ✗ 用法：lib_long_run.py script <命令的当前目录> -- <命令与参数…>", file=sys.stderr)
    print("     → 怎么办：照文件头「用法」一段写；平常经 research/scripts/detached-run.sh long-run-check -- <命令…> 调", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
