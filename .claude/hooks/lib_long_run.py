"""长活声明的判法：预计挂钟超过 2 小时的活，要说清分片单元、续跑入口、机器与额度取自多机配置与现查三样；分不了的写一行缺哪一段。
判据：.claude/singlefs-ai-sop/rules/test-script-discipline.md「用上现有的硬件加速」与「判定可复用：按输入存，按前缀剪」。
三处用这一份：scripts/hosts/detached-run.sh 的 start（--wall-clock over-2h）与 long-run-check 判被起脚本的文件头；
.claude/hooks/runner-dispatch-guard.sh 判派 experiment-designer 与带重活的几种类型（线程上限那张类型表）的提示；scripts/hosts/host-dispatch.py 判派法占几台（started_script、script_header_declaration、judge_script_header）。

声明的四个键（脚本文件头 / 派发提示的写法）：
  shards       # long-run-shards: <分片单元：按什么切、几片怎么定>                         分片：<…>
  resume       # long-run-resume: <续跑入口：同一条命令重起只补没做完的，怎么认做完的>       续跑：<…>
  hosts        # long-run-hosts: <机器与额度怎么取>                                         多机：<…>
               （脚本文件头与派发提示都按「登记值」读：这一行逐字等于 CANONICAL_HOSTS_SENTENCE 那一句规范写法（可带句末句号），后面不许接任何说明，说明另起一行写；不再用现算词表；
                 只有 LIVE_HOSTS_EXEMPT_TABLE 登记的脚本（sha256 对得上的那一版）文件头按「现算」读：点名 --emit-hosts 与 host-resources.py，见 judge 的 hosts_reading）
  unshardable  # long-run-unshardable: <缺哪一段、为什么>                                   分不了：<…>
派发提示另有一行「预计挂钟：2 小时内」或「预计挂钟：超过 2 小时」（后面可接逗号与推的依据）。

判法（judge）：
  - 三样里值不空、不是「无 / 没有 / -」、不是 <…> 占位的算写了。
  - hosts 写了的，按 hosts_reading 读：
      "registered"（默认：派发提示与脚本文件头走这一种，用户 2026-10-12 定、2026-10-13 把文件头也并过来）：这一行逐字等于 CANONICAL_HOSTS_SENTENCE 那一句（台数点名 --emit-hosts、
        每台线程点名 .claude/heavy-thread-default 的登记值），可带句末句号（「。」或「.」），后面不许接任何说明（「；……」「，……」都拒）；别的写法一律拒，不用现算词表去拦。
        要说明的另起一行写：脚本文件头里写成不带 long-run-hosts: 的普通注释行（同一个键写第二行会按次序接在后面，接出来就不再逐字等于规范句），
        派发提示里写成不以「多机：」起头的普通正文行。
        规范句写在这一处，骨架（scripts/agents/dispatch-prompt.py）、scripts/hosts/detached-run.sh 与 scripts/hosts/host-dispatch.py 的说明引它，不另抄词表。
        文件头写的读法与脚本做的事是否一致，静态判法看不到，由写文件头的人保证。
      "live"（只有 LIVE_HOSTS_EXEMPT_TABLE 登记、sha256 也对得上的脚本文件头走这一种，judge_script_header 查表）：要点名机器清单的读法 --emit-hosts（scripts/crash/crash-point-replay-shard-configuration-check.sh）与现查余量的 host-resources.py。
    两种都不许写死额度（HARD_CODED_QUOTA：数字后面跟线程、核、台、K/M/G/T，或 --threads N、-j N、--memory-cap N、--shard k/n、THREADS=N 这类）。
  - 三样缺的，要有 unshardable 一行：至少 UNSHARDABLE_MINIMUM_CHARACTERS 个字，并逐个点名缺的那几样的词（分片 / 续跑 / 多机）。
脚本文件头：#! 之后连着的注释块（# 起头的行与空行，到第一行别的为止）；同一个键写了几行，按次序接起来。
脚本文件头的读法（judge_script_header）：脚本相对本库所在仓根的路径在 LIVE_HOSTS_EXEMPT_TABLE（.claude/long-run-live-hosts-exempt，一行「路径  sha256  # 理由」）里、
  而且表里登记的 sha256 等于这份脚本现在文件的 sha256 的，按 "live" 读，别的一律 "registered"；在表里而 sha256 对不上的（脚本改过了）按 "registered" 读，另加一条 sha256 对不上的问题（判红）。
  表里只有按现算写、被 sha256 钉住不能改的脚本；指向不存在的路径、sha256 对不上、脚本已经按登记值写齐、按现算读也不齐、比基准多出的行、
  基准那一版里同一路径的 sha256 变了的行（改了脚本正文再把表里的 sha256 重钉，也算涨），都由 exempt-check 判红（只缩不涨按「路径 + sha256」成对比）。
被起的脚本（started_script）：命令的词里，含 / 或以 .sh / .py 结尾、按命令的当前目录解得到普通文件、以 .sh / .py 结尾或开头是 #! 的，
  跳过包装（WRAPPER_SCRIPTS）之后的第一份；一份都没有的（bash -c '…' 一长串、直接起 cargo 或编出来的二进制）判不出。

用法：
  python3 lib_long_run.py script <命令的当前目录> -- <命令与参数…>
      判被起脚本的声明：齐了退 0、stdout 打一行；不齐退 1、stderr 逐条 ✗ 加 → 出路；用法错与弄坏开关写错退 2
  python3 lib_long_run.py exempt-check <仓根> [--base-table <基准那一版的豁免表>]
      判 LIVE_HOSTS_EXEMPT_TABLE：写法、路径在不在、登记的 sha256 对不对得上、脚本是不是真要靠现算读法才齐，给了 --base-table 的再比只缩不涨（按路径 + sha256 成对比：基准那一版里没有的路径、同一路径的 sha256 换了的都判红；
      基准那一版可以还是不带 sha256 的旧写法，那样的行只比路径）；
      好退 0、stdout 打一行（带行数）；有问题退 1、stderr 逐条 ✗ 加 → 出路；仓根下没有这张表退 77；用法错与弄坏开关写错退 2（门禁 code-tooling 的 long-run-header-exempt 格调它）
  python   import lib_long_run（把 .claude/hooks 放进 sys.path），调 prompt_declaration、duplicate_prompt_problems（派发提示里「分片：」「续跑：」「多机：」同一标签写了两行就判红，只有派发闸用）、judge、judge_script_header

弄坏开关 LIB_LONG_RUN_BREAK=<项>（逗号连几个；只给用它的几份的自证判红用，认不出的值抛 ValueError）。打开后红在哪几格写在各份的文件头：
  partial-accepted            三样写了一样就当齐了
  hosts-unchecked             hosts 一行不看内容（不核两份现查、不核写死的额度）
  hosts-canonical-unchecked   "registered" 读法下，hosts 一行不核是不是规范句（写了就放，只剩写死的额度那一条）（派发闸与脚本文件头都红）
  hosts-suffix-accepted       "registered" 读法下，规范句后面接任何说明都放行（只核以规范句起头）（派发闸与脚本文件头都红）
  header-live-reading         脚本文件头退回按 "live" 读（点名 host-resources.py 才齐；登记值那一句反而拒）
  live-exempt-ignored         不查 LIVE_HOSTS_EXEMPT_TABLE，登记了的脚本也按 "registered" 读
  exempt-path-unchecked       exempt-check 不核表里的路径在不在（只 code-tooling 的 long-run-header-exempt 格用）
  exempt-needless-unchecked   exempt-check 不核脚本是不是已经按登记值写齐、按现算读是不是齐（只 long-run-header-exempt 格用）
  exempt-sha256-unchecked     豁免表里登记的 sha256 不比（exempt-check 不核、judge_script_header 在表里的一律按 "live" 读）（exempt-check 的格与 detached-run.sh 的自证用）
  exempt-growth-unchecked     exempt-check 不比基准（路径涨了与 sha256 重钉两样都不比；只 long-run-header-exempt 格用）
  exempt-repin-unchecked      exempt-check 比基准时只比路径、不比 sha256（基准里同一路径的 sha256 换了也放；只 long-run-header-exempt 格用）
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
import lib_heavy_tests  # noqa: E402

file_sha256 = lib_heavy_tests.file_sha256   # 文件内容的 sha256：共用重型测试判定模块那一份，不另写

BREAK_VARIABLE = "LIB_LONG_RUN_BREAK"
KNOWN_BREAK_MODES = ("partial-accepted", "hosts-unchecked", "unshardable-unchecked", "script-unresolved-accepted", "wrappers-not-skipped",
                     "hosts-canonical-unchecked", "hosts-suffix-accepted", "header-live-reading",
                     "live-exempt-ignored", "exempt-path-unchecked", "exempt-needless-unchecked", "exempt-growth-unchecked", "exempt-repin-unchecked", "exempt-sha256-unchecked")
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
HOSTS_READING_LIVE, HOSTS_READING_REGISTERED = "live", "registered"
# 登记值读法下多机一行的规范写法（逐字）：派发提示的「多机：」行与脚本文件头的 # long-run-hosts: 只认这一句（可带句末句号），后面不许接任何说明，说明另起一行写。
# 规范句只写在这一处：骨架、detached-run.sh、host-dispatch.py 的说明与自证样本引它，不另抄一份
CANONICAL_HOSTS_SENTENCE = "台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量"
HOSTS_SENTENCE_ENDINGS = ("", "。", ".")   # 规范句本身可带的句末句号（全角或半角）；别的结尾都算接了说明
LIVE_HOSTS_EXEMPT_TABLE = ".claude/long-run-live-hosts-exempt"   # 文件头仍按现算写、不能改的脚本：一行「路径  sha256  # 理由」，路径相对仓根，sha256 是被豁免那份脚本文件的 sha256（64 位小写十六进制）
LIVE_HOSTS_EXEMPT_ROW = re.compile(r"^(?P<path>[^\s#]+)[ \t]{2,}(?P<sha256>[0-9a-f]{64})[ \t]{2,}#[ \t]*(?P<reason>\S.*?)\s*$")
LIVE_HOSTS_EXEMPT_BASE_ROW = re.compile(r"^(?P<path>[^\s#]+)[ \t]{2,}(?:(?P<sha256>[0-9a-f]{64})(?:[ \t]|$))?")   # 基准那一版：路径必取；sha256 有就取（基准可能还是不带 sha256 的旧写法，那样的行只比路径）
EXEMPT_REASON_MINIMUM_CHARACTERS = 4
PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))   # 本库在 <仓根>/.claude/hooks/ 下
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


def canonical_hosts_problem(hosts, modes=frozenset()):
    """登记值读法下 hosts 一行是不是规范写法：逐字等于 CANONICAL_HOSTS_SENTENCE，可带句末句号（HOSTS_SENTENCE_ENDINGS），后面不许接任何说明。
    合返回 None，不合返回一句原因（接在「<…> 一行」后面）。"""
    text = hosts.strip()
    if "hosts-suffix-accepted" in modes and text.startswith(CANONICAL_HOSTS_SENTENCE):
        return None
    if text in {CANONICAL_HOSTS_SENTENCE + ending for ending in HOSTS_SENTENCE_ENDINGS}:
        return None
    if not text.startswith(CANONICAL_HOSTS_SENTENCE):
        return (f"不是规范写法「{CANONICAL_HOSTS_SENTENCE}」（逐字）：写的是「{text[:80]}」；登记值读法下这一行只认这一句（可带句末句号），"
                "别的说法（台数线程的取法换了词、加了括注、写了现算或折算）一律不认")
    rest = text[len(CANONICAL_HOSTS_SENTENCE):]
    return (f"在规范句「{CANONICAL_HOSTS_SENTENCE}」后面接了说明「{rest[:60]}」：这一行逐字等于规范句（可带句末句号），后面不许接任何说明；"
            "说明另起一行写（脚本文件头里写成不带 long-run-hosts: 的普通注释行，派发提示里写成不以「多机：」起头的普通正文行）")


def judge(declared, spelling, hosts_reading=HOSTS_READING_REGISTERED):
    """declared：键 → 值（没写的键不在里面）；spelling：键 → 那一行的写法（报问题时用）；hosts_reading：hosts 一行按哪种读（HOSTS_READING_REGISTERED 或 HOSTS_READING_LIVE）。
    交回问题的列表，空的是齐了。"""
    if hosts_reading not in (HOSTS_READING_LIVE, HOSTS_READING_REGISTERED):
        raise ValueError(f"hosts_reading 只认 {HOSTS_READING_LIVE}、{HOSTS_READING_REGISTERED}，给的是 {hosts_reading!r}")
    modes = breaks()
    problems = []
    written = {key for key in REQUIRED_KEYS if is_written(declared.get(key))}
    if "partial-accepted" in modes and written:
        written = set(REQUIRED_KEYS)
    hosts = declared.get("hosts")
    if "hosts" in written and is_written(hosts) and "hosts-unchecked" not in modes:
        if hosts_reading == HOSTS_READING_REGISTERED:
            canonical = None if "hosts-canonical-unchecked" in modes else canonical_hosts_problem(hosts, modes)
            if canonical:
                problems.append(f"{spelling['hosts']} 一行{canonical}")
        else:
            unnamed = [read for read in HOST_READS if read not in hosts]
            if unnamed:
                problems.append(f"{spelling['hosts']} 一行没点名 {'、'.join(unnamed)}：机器清单只从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台的线程与内存从 host-resources.py --json 现算")
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


def live_hosts_exempt_rows(root=PROJECT_ROOT):
    """LIVE_HOSTS_EXEMPT_TABLE 的行：[(行号, 路径, sha256, 理由)]，写法不对的行 路径 与 sha256 是 None、理由 是原行；表不在是 None。"""
    table = os.path.join(root, LIVE_HOSTS_EXEMPT_TABLE)
    if not os.path.isfile(table):
        return None
    rows = []
    with open(table, encoding="utf-8") as handle:
        for number, line in enumerate(handle.read().split("\n"), 1):
            if not line.strip() or line.lstrip().startswith("#"):
                continue
            found = LIVE_HOSTS_EXEMPT_ROW.match(line)
            if found and len(found.group("reason")) >= EXEMPT_REASON_MINIMUM_CHARACTERS and not os.path.isabs(found.group("path")):
                rows.append((number, os.path.normpath(found.group("path")), found.group("sha256"), found.group("reason")))
            else:
                rows.append((number, None, None, line))
    return rows


def header_exemption(script, root=PROJECT_ROOT):
    """脚本在豁免表里的情形：(None, None) 不在表里（或表读不了）；(登记的 sha256, 这份脚本现在文件的 sha256) 在表里。现在的读不了是 (登记的, None)。"""
    try:
        rows = live_hosts_exempt_rows(root)
    except OSError:
        return None, None
    relative = os.path.relpath(os.path.realpath(script), os.path.realpath(root))
    for _number, path, registered, _reason in rows or []:
        if path == relative:
            try:
                return registered, file_sha256(os.path.realpath(script))
            except OSError:
                return registered, None
    return None, None


def header_hosts_reading(script, root=PROJECT_ROOT):
    """脚本文件头的 # long-run-hosts: 按哪种读：登记在 LIVE_HOSTS_EXEMPT_TABLE 里、登记的 sha256 又等于这份脚本现在的 sha256 的按 "live"，别的一律 "registered"
    （在表里而 sha256 对不上的也按 "registered" 读，judge_script_header 另报 sha256 对不上）。"""
    modes = breaks()
    if "header-live-reading" in modes:
        return HOSTS_READING_LIVE
    if "live-exempt-ignored" in modes:
        return HOSTS_READING_REGISTERED
    registered, current = header_exemption(script, root)
    if registered is None:
        return HOSTS_READING_REGISTERED
    if "exempt-sha256-unchecked" in modes or registered == current:
        return HOSTS_READING_LIVE
    return HOSTS_READING_REGISTERED


def judge_script_header(script, declared, root=PROJECT_ROOT):
    """被起脚本文件头的声明按 header_hosts_reading 判：detached-run.sh 的 start 与 long-run-check、host-dispatch.py 文件头 ⓪ 都调这一个。
    脚本在豁免表里而登记的 sha256 对不上（脚本改过了）：按登记值读法判，另加一条 sha256 对不上的问题，一定判红。"""
    problems = judge(declared, HEADER_SPELLING, header_hosts_reading(script, root))
    modes = breaks()
    if "header-live-reading" not in modes and "live-exempt-ignored" not in modes and "exempt-sha256-unchecked" not in modes:
        registered, current = header_exemption(script, root)
        if registered is not None and registered != current:
            problems.insert(0, f"{os.path.relpath(os.path.realpath(script), os.path.realpath(root))} 在 {LIVE_HOSTS_EXEMPT_TABLE} 里登记的 sha256 是 {registered}，"
                               f"这份脚本现在的是 {current or '（读不了）'}：豁免只给登记的那一版文件，脚本改过就不再按现算读；文件头改按登记值读法写（多机一行用规范句），或把改过的脚本连新 sha256 交主 agent 问用户")
    return problems


def exempt_table_problems(root, base_table=None):
    """LIVE_HOSTS_EXEMPT_TABLE 的问题：[(说什么, 怎么办)] 与行数。base_table 是基准那一版的表文本（None 不比只缩不涨）。表不在返回 (None, 0)。"""
    rows = live_hosts_exempt_rows(root)
    if rows is None:
        return None, 0
    modes = breaks()
    problems = []
    for number, path, registered_sha256, reason in rows:
        if path is None:
            problems.append((f"第 {number} 行不是「<相对仓根的路径>  <被豁免脚本的 sha256，64 位小写十六进制>  # 理由」（各段之间至少两个空格，理由至少 {EXEMPT_REASON_MINIMUM_CHARACTERS} 个字，路径不写绝对路径）：{reason[:120]}",
                             "照这个写法改；表里只放文件头仍按现算写、被 sha256 钉住不能改的脚本，sha256 取 sha256sum <脚本> 的前一栏"))
            continue
        full = os.path.join(root, path)
        if not os.path.isfile(full):
            if "exempt-path-unchecked" not in modes:
                problems.append((f"第 {number} 行指向不存在的路径：{path}", "脚本搬了就改路径，删了或改成按登记值写就删掉这一行；豁免只缩不涨"))
            continue
        if "exempt-sha256-unchecked" not in modes:
            try:
                current_sha256 = file_sha256(full)
            except OSError as error:
                problems.append((f"第 {number} 行的 {path} 读不了：{error}", "看它的读权限"))
                continue
            if current_sha256 != registered_sha256:
                problems.append((f"第 {number} 行的 {path} 现在的 sha256 是 {current_sha256}，表里登记的是 {registered_sha256}（脚本改过了，豁免只给登记的那一版文件）",
                                 "这份脚本不该再改：把改动撤回去；确要改它，文件头按登记值读法写（多机一行用规范句）并删掉这一行，或交主 agent 问用户再改表里的 sha256"))
                continue
        if "exempt-needless-unchecked" in modes:
            continue
        try:
            declared = script_header_declaration(full)
        except OSError as error:
            problems.append((f"第 {number} 行的 {path} 读不了：{error}", "看它的读权限"))
            continue
        if not judge(declared, HEADER_SPELLING, HOSTS_READING_REGISTERED):
            problems.append((f"第 {number} 行的 {path} 文件头已经按登记值读法写齐了，不需要豁免", "删掉这一行"))
        elif judge(declared, HEADER_SPELLING, HOSTS_READING_LIVE):
            problems.append((f"第 {number} 行的 {path} 文件头按现算读法也不齐：{'；'.join(judge(declared, HEADER_SPELLING, HOSTS_READING_LIVE))}",
                             "豁免只放行按现算读法写齐的脚本；补齐它的文件头，或删掉这一行并改成登记值读法"))
    if base_table is not None and "exempt-growth-unchecked" not in modes:
        base_rows = {}   # 路径 → 基准那一版里这个路径登记的 sha256 的集合；旧写法（不带 sha256）的行记一个 None，那样的路径只比路径
        for line in base_table.split("\n"):
            found = LIVE_HOSTS_EXEMPT_BASE_ROW.match(line)
            if found and not line.lstrip().startswith("#"):
                base_rows.setdefault(os.path.normpath(found.group("path")), set()).add(found.group("sha256"))
        for number, path, registered_sha256, _reason in rows:
            if path is None:
                continue
            if path not in base_rows:
                problems.append((f"第 {number} 行 {path} 是基准那一版里没有的（豁免表涨了）", "豁免只缩不涨：新脚本的文件头按登记值写；确实要加的，交主 agent 问用户再加"))
            elif "exempt-repin-unchecked" not in modes and None not in base_rows[path] and registered_sha256 not in base_rows[path]:
                problems.append((f"第 {number} 行 {path} 登记的 sha256 {registered_sha256} 与基准那一版里的 {'、'.join(sorted(base_rows[path]))} 不同（豁免表重钉了：改了脚本正文再换表里的 sha256 也算涨）",
                                 "豁免只缩不涨，表里的 sha256 不能换：把脚本的改动撤回去（sha256 对回基准那一版）；文件头已能按登记值读法写的，改写文件头并删掉这一行；确要保留改动并重钉的，交主 agent 问用户"))
    return problems, len(rows)


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


def duplicate_prompt_problems(prompt):
    """派发提示里「分片：」「续跑：」「多机：」（REQUIRED_KEYS 那三样）哪一个写了两行及以上：prompt_declaration 只读第一行，第二行写什么都没人判，所以同一标签出现两次判红。
    交回问题的列表（每个重复的标签一条，点名每一行的开头），空的是没有重复。判的是传进来的整段文字：要先去掉代码块的，调用方先去（runner-dispatch-guard.sh 的 without_code_blocks）。"""
    problems = []
    for label, key in PROMPT_LABELS.items():
        if key not in REQUIRED_KEYS:
            continue
        values = [found.group(1).strip() for found in re.finditer(LINE_START + label + r"[* \t]*[：:][* \t]*(.*)$", prompt, re.M)]
        if len(values) >= 2:
            shown = "；".join(f"第 {order} 处「{value[:30]}」" for order, value in enumerate(values[:4], 1))
            problems.append(f"「{label}：」写了 {len(values)} 行（{shown}）：只写一行，只有第一行会被读；要补充的另起一行，写成不以「{label}：」起头的普通正文")
    return problems


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
    problems = judge_script_header(script, declared)
    if problems:
        for problem in problems:
            print(f"  ✗ 长活声明（{script}）：{problem}", file=sys.stderr)  # gate-lint:detail
        print(f"     → 怎么办：在 {script} 文件头的注释块里补齐（判据 .claude/singlefs-ai-sop/rules/test-script-discipline.md「用上现有的硬件加速」「判定可复用：按输入存，按前缀剪」）："
              "# long-run-shards: 分片单元；# long-run-resume: 同一条命令重起只补没做完的；"
              f"# long-run-hosts: {CANONICAL_HOSTS_SENTENCE}（逐字这一句，可带句末句号，后面不接任何说明；要说明另起一行写普通注释）；"
              "分不了的写 # long-run-unshardable: <点名缺的分片 / 续跑 / 多机与为什么，至少 8 个字>"
              f"（{LIVE_HOSTS_EXEMPT_TABLE} 里登记的脚本只要 sha256 对得上就按现算读，那张表只缩不涨，不是给新脚本加的）", file=sys.stderr)
        return 1
    how = "；".join(f"{KEY_WORDS.get(key, '分不了')}：{declared[key]}" for key in (*REQUIRED_KEYS, "unshardable") if is_written(declared.get(key)))
    print(f"  ✓ 长活声明齐了（{script}，判了 {len(declared)} 个键）：{how}")
    return 0


def exempt_main(root, base_table_file):
    """命令行 exempt-check 子命令：退 0 好，1 有问题，77 仓根下没有这张表。"""
    base_table = None
    if base_table_file is not None:
        try:
            with open(base_table_file, encoding="utf-8") as handle:
                base_table = handle.read()
        except OSError as error:
            print(f"  ✗ 读不了基准那一版的豁免表 {base_table_file}：{error}", file=sys.stderr)
            print("     → 怎么办：核对 --base-table 的路径；基准里还没有这张表就不给 --base-table", file=sys.stderr)
            return 1
    try:
        problems, count = exempt_table_problems(root, base_table)
    except OSError as error:
        print(f"  ✗ 读不了 {os.path.join(root, LIVE_HOSTS_EXEMPT_TABLE)}：{error}", file=sys.stderr)
        print("     → 怎么办：看它的读权限", file=sys.stderr)
        return 1
    if problems is None:
        print(f"  ⊘ 本次无对象可判：{root} 下没有 {LIVE_HOSTS_EXEMPT_TABLE}")
        return 77
    if problems:
        print(f"  ✗ {LIVE_HOSTS_EXEMPT_TABLE} 有 {len(problems)} 处不对（共 {count} 行）：", file=sys.stderr)  # gate-lint:summary
        for what, _howto in problems:
            print(f"     {what}", file=sys.stderr)  # gate-lint:detail
        print(f"     → 怎么办：{'；'.join(sorted({howto for _what, howto in problems}))}", file=sys.stderr)
        return 1
    note = "与基准那一版比没涨" if base_table is not None else "没给基准（基准那一版里还没有这张表，或不在 git 仓里），只缩不涨这一条没比"
    print(f"  ✓ {LIVE_HOSTS_EXEMPT_TABLE} {count} 行：每行的脚本都在、登记的 sha256 都对得上，文件头都得靠现算读法才齐（按登记值读不齐）；{note}")
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
    if len(argv) == 2 and argv[0] == "exempt-check":
        return exempt_main(argv[1], None)
    if len(argv) == 4 and argv[0] == "exempt-check" and argv[2] == "--base-table":
        return exempt_main(argv[1], argv[3])
    print("  ✗ 用法：lib_long_run.py script <命令的当前目录> -- <命令与参数…>；lib_long_run.py exempt-check <仓根> [--base-table <文件>]", file=sys.stderr)
    print("     → 怎么办：照文件头「用法」一段写；平常经 scripts/hosts/detached-run.sh long-run-check -- <命令…> 调", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
