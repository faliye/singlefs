#!/usr/bin/env bash
# 门禁 doc-text 的格函数库：out-of-project-names、utf8-valid、dated-status、comment-history。
# 由 .claude/gate.d/doc-text.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释；读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取。

cell_out_of_project_names() {
  local base="" changed changed_file common_directory main_root python_rc=0
  if [[ "$text_notations_break" == out-of-project-names-unjudged ]]; then
    echo "  ! 弄坏开关 out-of-project-names-unjudged：这一格不判，记本次无对象可判"
    exit 77
  fi
  cd "$ROOT" || exit 1
  private_value_sources
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "  ! 被判的根不在 git 仓里，划不出「这一次改动新加的行」，这一格本次无对象可判（这不是通过）"
    exit 77
  fi
  # 仓自己与它的上一级目录：取 git common-dir 的上一层（主工作树的根，--staged 的临时 worktree 里也是主工作树），同 private_value_sources
  main_root="$ROOT"
  if common_directory="$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$common_directory" ]]; then
    main_root="$(dirname "$common_directory")"
  fi
  changed_file="$(mktemp "${TMPDIR:-/tmp}/out-of-project-names.XXXXXX")" || { echo "  ✗ 建不了放改动文件清单的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
  if (( ! out_of_project_all_lines )); then
    base="$(gate_diff_base gate)"
    changed="$(text_changed_paths "$base")" || { rm -f "$changed_file"; echo "  ✗ 取不到这一轮碰了哪些文件（基准 ${base:-空}）"; echo "     → 怎么办：看 git 是不是可用、基准提交在不在；改动范围只经 scripts/gate/changed-paths.sh 取。"; exit 1; }
    printf '%s\n' "$changed" > "$changed_file"
  fi
  python3 - "$base" "$changed_file" "$text_notations_break" "$PRIVATE_VALUES_LIBRARY" "$PRIVATE_CONFIGURATION_FILE" "$PRIVATE_HOSTNAME_FILE" \
    "$PRIVATE_IN_SAMPLE" "$out_of_project_all_lines" "${GATE_NOT_RUN_FILE:-}" "${GATE_OUT_OF_PROJECT_PARENT:-}" "$main_root" "$text_diff_cache" <<'PY' || python_rc=$?
import errno, os, re, subprocess, sys, types

(base, changed_path, broken, private_values_source, configuration_path, hostname_path,
 in_sample, all_lines, not_run_file, parent_override, main_root, diff_cache_path) = sys.argv[1:13]
in_sample, all_lines = in_sample == "1", all_lines == "1"
private_values = types.ModuleType("lib_private_values")      # 读改动的新加行与路径打码的共用函数（doc-text.sh 的 PRIVATE_VALUES_LIBRARY，与别的格同一份）
exec(compile(private_values_source, "private-values-library", "exec"), private_values.__dict__)
masking_targets = []
if configuration_path:
    try:
        masking_targets = private_values.load_private_targets(configuration_path, hostname_path, broken)[0]
    except OSError:
        masking_targets = []
HEAD_BYTES = 8192
NAME_CHARACTERS = "A-Za-z0-9._+-"
NAME_CHARACTER = re.compile(f"[{NAME_CHARACTERS}]")
TABLE_PATH = ".claude/out-of-project-paths"
PRIVATE_LIST = os.path.expanduser("~/.config/singlefs/out-of-project-names")
MINIMUM_REASON_CHARACTERS = 8
SOURCE_LABELS = {"home": "本机家目录", "parent": "仓的上一级目录", "private": "第二台私有名单", "root": "根下一层的系统目录"}
CONTEXT_LABELS = {"path": "名字类·路径段", "backtick": "名字类·反引号", "capacity": "名字类·带容量", "root": "容量类·根下系统目录带容量"}
CONTEXT_ORDER = ["path", "backtick", "capacity", "root"]
# 容量：数加单位（419G、16 GiB、3.5T、（82G） 里的 82G）；单位后面不能再接字母数字
CAPACITY = re.compile(r"(?<![A-Za-z0-9.])\d+(?:\.\d+)?[ \t]?(?:[KMGT]i?B|[KMGT])(?![A-Za-z0-9])")
# 名字与容量之间许夹的：收尾的反引号、空白、一个左括号或冒号或「约」
CAPACITY_GAP = re.compile(r"`?[ \t]*(?:[（(：:]|约)?[ \t]*$")
BACKTICK = re.compile(r"`([^`\n]+)`")
# 只由小写 ASCII 字母组成的名字（多半是普通英文词）不在「反引号里只有这个名字」的上下文里判，路径段与带容量照判
LOWERCASE_WORD = re.compile(r"[a-z]+")


def fail(what, howto):
    print(f"  ✗ {what}")
    print(f"     → 怎么办：{howto}")
    sys.exit(1)


def shown_path(path):
    return private_values.shown_path(path, masking_targets, broken)


def git(*arguments):
    return subprocess.run(["git", "-c", "core.quotepath=false", *arguments], capture_output=True)


def entry_names(directory):
    try:
        return sorted(os.listdir(directory))
    except OSError as error:
        fail(f"列不出 {directory} 下一层的条目（{errno.errorcode.get(error.errno, error.errno)}）", "看这个目录在不在、有没有读权限；这一格要现读它下一层的条目名来比。")


def sample_listing(name):
    """样本根里的替身：第一行是那个目录的路径（.out-of-project-root 没有这一行），其余一行一个条目名；文件不在交回 None。"""
    try:
        lines = [line.strip() for line in open(name, encoding="utf-8").read().split("\n")]
    except FileNotFoundError:
        return None
    return [line for line in lines if line and not line.startswith("#")]


# ── 名字从哪来 ──
if in_sample:
    home_listing = sample_listing(".out-of-project-home") or ["/nonexistent-sample-home"]
    parent_listing = sample_listing(".out-of-project-parent") or ["/nonexistent-sample-parent"]
    home, home_names = home_listing[0], home_listing[1:]
    parent, parent_names = parent_listing[0], parent_listing[1:]
    root_names = sample_listing(".out-of-project-root") or []
    private_names = sample_listing(".out-of-project-private-names")
    repository_name = os.path.basename(os.getcwd())
else:
    home = os.path.expanduser("~")
    parent = parent_override or os.path.dirname(main_root)
    home_names, parent_names = entry_names(home), entry_names(parent)
    root_names = [name for name in entry_names("/") if os.path.isdir(os.path.join("/", name))]
    try:
        private_names = [line.strip() for line in open(PRIVATE_LIST, encoding="utf-8").read().split("\n")
                         if line.strip() and not line.strip().startswith("#")]
    except FileNotFoundError:
        private_names = None
    repository_name = os.path.basename(main_root)
if broken == "out-of-project-names-home-unread":            # 弄坏：不读本机家目录
    home_names = []
if broken == "out-of-project-names-parent-unread":          # 弄坏：不读仓的上一级目录
    parent_names = []
if broken == "out-of-project-names-private-unread":         # 弄坏：不读第二台私有名单（当它在、却是空的）
    private_names = [] if private_names is not None else None


def excluded(name):
    """singlefs 自己的：singlefs、singlefs- 开头的（上游 singlefs-ai-sop-* 也在内）与仓自己。"""
    if broken == "out-of-project-names-singlefs-unexcluded":   # 弄坏：不排掉 singlefs 自己的
        return name in {".", ".."}
    return name in {".", "..", "singlefs", repository_name} or name.startswith("singlefs-")


sources_of = {}             # 名字 → {来源}
for source, names in (("home", home_names), ("parent", parent_names), ("private", private_names or [])):
    for name in names:
        if not excluded(name):
            sources_of.setdefault(name, set()).add(source)
home_side = sorted((name for name, sources in sources_of.items() if sources & {"home", "private"}), key=len, reverse=True)
parent_side = sorted((name for name, sources in sources_of.items() if sources & {"parent", "private"}), key=len, reverse=True)
all_names = sorted(sources_of, key=len, reverse=True)
repository_top_names = set()
if broken != "out-of-project-names-repository-names-judged":   # 弄坏：仓里自己也有的顶层名照样按反引号、带容量判
    repository_top_names = set(os.listdir("."))
# ../<名> 里名字也是仓根下一层的条目的（../.claude/kb、include_str!("../../../.claude/…")、crates/../.claude），是仓里自己的相对路径，不判
relative_repository_names = set()
if broken != "out-of-project-names-relative-repository-judged":   # 弄坏：../ 后面接仓根下一层也有的名字照判
    relative_repository_names = set(os.listdir("."))
loose_names = sorted((name for name in all_names if name not in repository_top_names), key=len, reverse=True)
root_side = sorted(set(root_names), key=len, reverse=True)

# ── 像目录的上下文：路径前缀 ──
BOUNDARY = f"(?<![{NAME_CHARACTERS}])"
home_alternatives = [BOUNDARY + "~", r"\$HOME", r"\$\{HOME\}", BOUNDARY + re.escape(home), BOUNDARY + r"/home/[^/\s`'\"]+"]
HOME_PREFIX = "(?:" + "|".join(home_alternatives) + ")/"
# 不带边界的家目录写法：扫路径前缀（边界在 prefix_boundary 里逐处补判）与放行表的 XDG 回退写法（${XDG_CACHE_HOME:-$HOME/.cache}/singlefs 里 :- 紧接着家目录）共用
home_bare = "(?:~|\\$HOME|\\$\\{HOME\\}|" + re.escape(home) + "|/home/[^/\\s`'\"]+)/"
parent_alternatives = [BOUNDARY + re.escape(parent)]
if parent.startswith(home.rstrip("/") + "/"):
    parent_alternatives.append(HOME_PREFIX + re.escape(parent[len(home.rstrip("/")) + 1:]))
elif not in_sample and re.match(r"^/home/[^/]+/.+", parent):
    parent_alternatives.append(HOME_PREFIX + re.escape(parent.split("/", 3)[3]))
PARENT_PREFIX = "(?:" + "|".join(parent_alternatives) + ")/|(?<![A-Za-z0-9_+-])\\.\\./"
# singlefs 自己的路径（仓自己、上游 singlefs-ai-sop-*、家目录下 singlefs- 开头的）整段放过：~/<上一级>/singlefs 里那个上一级目录名不算命中
own_names = "singlefs[" + NAME_CHARACTERS + "]*" + ("" if repository_name.startswith("singlefs") else "|" + re.escape(repository_name))
OWN_SPAN_RE = re.compile(f"(?:{HOME_PREFIX}|{PARENT_PREFIX})(?:{own_names})(?![{NAME_CHARACTERS}])")


NAME_END = f"(?![{NAME_CHARACTERS}])"
# XDG 基目录与它的变量：放行表里 ~/<基目录>/… 那一条另认 ${<变量>:-<家目录>/<基目录>}/… 的回退写法
XDG_BASES = (("XDG_CACHE_HOME", ".cache"), ("XDG_CONFIG_HOME", ".config"), ("XDG_DATA_HOME", ".local/share"), ("XDG_STATE_HOME", ".local/state"))


def allow_pattern(path):
    """放行表里一条路径的命中范围，后面不能紧接名字字符。~/ 开头的这几种写法都当成同一条 ~/… 路径认：
    家目录写成 ~/、$HOME/、${HOME}/、家目录的绝对路径或 /home/<任一用户>/；XDG 回退 ${XDG_CACHE_HOME:-$HOME/.cache}/singlefs（另三个基目录同样）；
    python 拼接 os.path.expanduser("~/.local/share"), "singlefs"（家目录那一段的字串后面逐段跟着引号括起的各段）；
    末段是 singlefs 的，singlefs-<后缀>（~/.cache/singlefs-golden）也算 singlefs 自己的。XDG 基目录本身、停在半中间的路径不放。"""
    if not path.startswith("~/"):
        return re.compile(BOUNDARY + re.escape(path) + NAME_END)
    home_prefix, home_loose = HOME_PREFIX, home_bare
    if broken == "out-of-project-names-other-user-home-unrecognized":   # 弄坏：/home/<别的用户>/ 不当家目录认
        home_prefix = "(?:" + "|".join(home_alternatives[:-1]) + ")/"
        home_loose = "(?:~|\\$HOME|\\$\\{HOME\\}|" + re.escape(home) + ")/"
    segments = path[2:].split("/")
    escaped = [re.escape(segment) for segment in segments]
    if segments[-1] == "singlefs" and broken != "out-of-project-names-singlefs-suffix-unrecognized":   # 弄坏：singlefs-<后缀> 不认
        escaped[-1] += f"(?:-[{NAME_CHARACTERS}]*)?"
    forms = [home_prefix + "/".join(escaped) + NAME_END]
    for variable, base in XDG_BASES:
        depth = base.count("/") + 1
        if segments[:depth] != base.split("/") or len(segments) == depth:
            continue
        if broken != "out-of-project-names-xdg-fallback-unrecognized":   # 弄坏：XDG 回退写法不认
            forms.append(r"\$\{" + variable + ":-" + home_loose + "/".join(escaped[:depth]) + r"\}/" + "/".join(escaped[depth:]) + NAME_END)
        if broken == "out-of-project-names-xdg-base-allowed":   # 弄坏：XDG 基目录底下一律放过
            forms.append(home_prefix + "/".join(escaped[:depth]) + NAME_END)
    if broken != "out-of-project-names-python-join-unrecognized":   # 弄坏：python 拼接不认
        for split in range(1, len(segments)):
            forms.append(home_prefix + "/".join(escaped[:split]) + "[\"']\\)?"
                         + "".join("\\s*,\\s*[\"']" + segment + "[\"']" for segment in escaped[split:]))
    return re.compile("|".join(f"(?:{form})" for form in forms))


# ── 放行表 ──
allow_entries, table_problems = [], []      # (行号, 路径, 编译好的范围) / (行号, 哪里不对)
table_present = os.path.isfile(TABLE_PATH)
if table_present and broken != "out-of-project-names-allow-ignored":
    for number, raw in enumerate(open(TABLE_PATH, encoding="utf-8", errors="replace").read().split("\n"), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        found = re.fullmatch(r"(\S+)\s+#\s*(.*?)", line)
        if broken == "out-of-project-names-table-format-unchecked":   # 弄坏：不核写法，取行首一段当路径
            allow_entries.append((number, line.split()[0], allow_pattern(line.split()[0])))
            continue
        if not found:
            table_problems.append((number, "不是「<路径>  # 理由」的写法（路径后面要有 # 与理由）"))
        elif not (found.group(1).startswith("~/") or found.group(1).startswith("/")):
            table_problems.append((number, "路径要以 ~/ 或 / 开头（写 singlefs 读的那条仓外路径本身）"))
        elif len(found.group(2)) < MINIMUM_REASON_CHARACTERS:
            table_problems.append((number, f"理由不到 {MINIMUM_REASON_CHARACTERS} 个字"))
        else:
            allow_entries.append((number, found.group(1), allow_pattern(found.group(1))))

unreferenced = []           # (行号)：放行表里这一条没有一份 git 跟踪的脚本字面引到
if allow_entries and broken != "out-of-project-names-references-unchecked":
    listing = git("ls-files", "-z", "--", "*.sh", "*.py", "*.rs")
    if listing.returncode != 0:
        fail("列不出 git 跟踪的脚本", "看 git 是不是可用；放行表每一条都要有一份 git 跟踪的 .sh、.py 或 .rs 字面引到。")
    waiting = {number: pattern for number, _path, pattern in allow_entries}
    # 预筛的字面：路径（~/ 之后那一段）的每一段都要在；XDG 回退与 python 拼接把路径拆开写，不认整段字面
    tails = {number: [segment for segment in (path[2:] if path.startswith("~/") else path).split("/") if segment] for number, path, _pattern in allow_entries}
    if broken == "out-of-project-names-references-literal-tail":   # 弄坏：预筛认整段字面，拆开写的引用认不到
        tails = {number: [path[2:] if path.startswith("~/") else path] for number, path, _pattern in allow_entries}
    for script in listing.stdout.decode("utf-8", "replace").split("\0"):
        if not waiting:
            break
        if not script or script == TABLE_PATH or os.path.islink(script) or not os.path.isfile(script):
            continue
        try:
            text = open(script, encoding="utf-8", errors="replace").read()
        except OSError:
            continue
        # 先认路径各段的字面在不在，在了才用正则认几种写法：整仓几千份脚本逐份试正则太慢
        for number in [number for number, pattern in waiting.items() if all(segment in text for segment in tails[number]) and pattern.search(text)]:
            del waiting[number]
    unreferenced = sorted(waiting)


NAME_SET = frozenset("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789._+-")
TOKEN_AT = re.compile(f"[{NAME_CHARACTERS}]+")
TOKEN_LIMIT = 256           # 往回认名字最多看这么多个字，再长的不当名字
# 扫路径前缀用不带后顾断言的写法（整份大文件里逐处试后顾断言太慢），边界在 prefix_boundary 里逐处补判
HOME_SCAN_RE = re.compile(home_bare)
parent_bare = [re.escape(parent)]
if parent.startswith(home.rstrip("/") + "/"):
    parent_bare.append(home_bare + re.escape(parent[len(home.rstrip("/")) + 1:]))
PARENT_SCAN_RE = re.compile("(?:" + "|".join(parent_bare) + ")/|\\.\\./")


def prefix_boundary(text, start):
    """前缀前面那个字：~、家目录、/home/ 与上一级目录前面不能紧接名字字符，../ 前面不能紧接名字字符里除 . 之外的，$HOME 不看。"""
    if start == 0 or text[start] == "$":
        return True
    before = text[start - 1]
    if text.startswith("../", start):
        return before == "." or before not in NAME_SET
    return before not in NAME_SET


def odd(names):
    """名字里有名字字符之外的字（空白、非 ASCII 这类）的，切 token 认不出来，逐个比。"""
    return [name for name in names if not TOKEN_AT.fullmatch(name)]


def name_at(text, position, name_set, odd_names):
    """position 起头、后面不紧接名字字符的那个名字；没有交回 None。"""
    found = TOKEN_AT.match(text, position)
    if found and found.group(0) in name_set:
        return found.group(0)
    for name in odd_names:
        end = position + len(name)
        if text.startswith(name, position) and (end == len(text) or text[end] not in NAME_SET):
            return name
    return None


def name_ending(text, end, name_set, odd_names):
    """在 end 收尾、前面不紧接名字字符的那个名字：(起点, 名字)；没有交回 None。"""
    start, limit = end, max(0, end - TOKEN_LIMIT)
    while start > limit and text[start - 1] in NAME_SET:
        start -= 1
    if start < end and (start == 0 or text[start - 1] not in NAME_SET) and text[start:end] in name_set:
        return start, text[start:end]
    for name in odd_names:
        start = end - len(name)
        if start >= 0 and text.endswith(name, 0, end) and (start == 0 or text[start - 1] not in NAME_SET):
            return start, name
    return None


ROOT_LEADING = frozenset(NAME_SET | set("/~$}"))
home_set, parent_set, loose_set, root_set = set(home_side), set(parent_side), set(loose_names), set(root_side)
home_odd, parent_odd, loose_odd, root_odd = odd(home_side), odd(parent_side), odd(loose_names), odd(root_side)
allow_names = {segment for _number, path, _pattern in allow_entries for segment in path.split("/")}
# 家目录或仓的上一级目录前缀收尾在这一处（~/、$HOME/、/home/<用户>/、<上一级>/、../）：它后面那个名字是第一段，不是深层段
FIRST_SEGMENT_PREFIX_RE = re.compile("(?:" + home_bare + "|(?:" + "|".join(parent_bare) + ")/|\\.\\./)\\Z")
FIRST_SEGMENT_WINDOW = len(home) + len(parent) + TOKEN_LIMIT


def deep_segment(text, start):
    """start 起头的名字是不是一条路径的深层段：前面紧挨着 /，/ 前面还有一段（名字字符或 / ~ $ }），
    而这个 / 不是家目录或仓的上一级目录前缀的收尾（~/<名>、$HOME/<名>、/home/<用户>/<名>、<上一级>/<名>、../<名> 里的名字是第一段，不算深层）。"""
    if start < 2 or text[start - 1] != "/" or text[start - 2] not in ROOT_LEADING:
        return False
    position = max(0, start - FIRST_SEGMENT_WINDOW)
    while True:
        found = FIRST_SEGMENT_PREFIX_RE.search(text, position, start)
        if not found:
            return True
        if prefix_boundary(text, found.start()):
            return False
        position = found.start() + 1


def line_allowed_spans(line):
    """一行里的放行范围（行内偏移）：singlefs 自己的路径与放行表每一条的命中范围。"""
    spans = []
    if broken != "out-of-project-names-singlefs-unexcluded":
        spans.extend(found.span() for found in OWN_SPAN_RE.finditer(line))
    for _number, path, pattern in allow_entries:
        spans.extend(found.span() for found in pattern.finditer(line))
    return spans


def judge(text):
    """一段文字（一行，或 --all-lines 时整份文件）里的命中：[(起点, 终点, 上下文, 名字)]，落在放行范围里的去掉；另交回放过了几处。
    放行范围只在有命中的那一行里现算（放行表的写法都在一行之内）。"""
    hits = []
    if broken != "out-of-project-names-path-unjudged":
        for scan_re, name_set, odd_names in ((HOME_SCAN_RE, home_set, home_odd), (PARENT_SCAN_RE, parent_set, parent_odd)):
            for found in scan_re.finditer(text):
                if not prefix_boundary(text, found.start()):
                    continue
                name = name_at(text, found.end(), name_set, odd_names)
                if name and found.group(0) == "../" and name in relative_repository_names:
                    continue
                if name and found.group(0) == "../" and broken == "out-of-project-names-relative-all-unjudged":   # 弄坏：../ 一律不判
                    continue
                if name:
                    hits.append((found.end(), found.end() + len(name), "path", name))
    if broken != "out-of-project-names-backtick-unjudged":
        for found in BACKTICK.finditer(text):
            content = found.group(1).strip()
            content = content[:-1] if content.endswith("/") else content
            if LOWERCASE_WORD.fullmatch(content) and broken != "out-of-project-names-lowercase-backtick-judged":   # 弄坏：只有小写字母的名字在反引号里也判
                continue
            if content in sources_of and content not in repository_top_names:
                hits.append((found.start(1), found.end(1), "backtick", content))
    for found in CAPACITY.finditer(text):
        gap = CAPACITY_GAP.search(text, max(0, found.start() - 12), found.start())
        end = gap.start() if gap else found.start()
        if broken != "out-of-project-names-capacity-unjudged":
            named = name_ending(text, end, loose_set, loose_odd)
            if named and broken == "out-of-project-names-lowercase-everywhere-unjudged" and LOWERCASE_WORD.fullmatch(named[1]):   # 弄坏：小写词带容量也不判
                named = None
            # 路径的深层段（~/.local/share/singlefs/work/<轮>/bin 里的 bin）不按名字类判，那条路径归不归本项目交路径段按第一段判
            if named and broken != "out-of-project-names-deep-segment-judged" and deep_segment(text, named[0]):   # 弄坏：路径深层段照按名字类判
                named = None
            if named:
                hits.append((named[0], end, "capacity", named[1]))
        if broken != "out-of-project-names-root-unjudged":
            root_end = end - 1 if end > 0 and text[end - 1] == "/" else end
            named = name_ending(text, root_end, root_set, root_odd)
            if named and named[0] > 0 and text[named[0] - 1] == "/" and (named[0] == 1 or text[named[0] - 2] not in ROOT_LEADING):
                hits.append((named[0] - 1, root_end, "root", named[1]))
    if broken == "out-of-project-names-root-without-capacity":   # 弄坏：根下系统目录不带容量也判
        for name in root_side:
            for found in re.finditer(r"(?<![" + NAME_CHARACTERS + r"/~$}])/" + re.escape(name) + f"(?![{NAME_CHARACTERS}])", text):
                hits.append((found.start(), found.end(), "root", name))
    kept, allowed, spans_by_line = [], 0, {}
    for start, end, context, name in sorted(set(hits)):
        if context != "root":
            if broken == "out-of-project-names-allow-whole-name":   # 弄坏：按名字放行（放行表路径里出现过的名字到处都放过）
                if name in allow_names:
                    allowed += 1
                    continue
            else:
                line_start = text.rfind("\n", 0, start) + 1
                if line_start not in spans_by_line:
                    line_end = text.find("\n", start)
                    spans_by_line[line_start] = line_allowed_spans(text[line_start: len(text) if line_end < 0 else line_end])
                if any(span_start <= start - line_start and end - line_start <= span_end for span_start, span_end in spans_by_line[line_start]):
                    allowed += 1
                    continue
        kept.append((start, end, context, name))
    return kept, allowed


def labels(hits):
    """一行命中的类别与来源：「名字类·路径段（本机家目录）」这样，不带名字本身。"""
    by_context = {}
    for _start, _end, context, name in hits:
        by_context.setdefault(context, set()).update(sources_of.get(name, {"root"}) if context != "root" else {"root"})
    order = ["home", "parent", "private", "root"]
    return "、".join(f"{CONTEXT_LABELS[context]}（来源：{'、'.join(SOURCE_LABELS[source] for source in order if source in by_context[context])}）"
                    for context in CONTEXT_ORDER if context in by_context)


def shown_names(hits):
    if broken == "out-of-project-names-print-names":            # 弄坏：红行把命中的名字也打出来
        return f"（名字：{'、'.join(sorted({name for _s, _e, _c, name in hits}))}）"
    return "（名字不打印）"


untracked_listing = git("ls-files", "-z", "--others", "--exclude-standard")
if untracked_listing.returncode != 0:
    fail("列不出未跟踪的文件", "看 git 是不是可用；改动范围只经 scripts/gate/changed-paths.sh 取。")
untracked = {name for name in untracked_listing.stdout.decode("utf-8", "replace").split("\0") if name}
if all_lines:
    tracked_listing = git("ls-files", "-z")
    if tracked_listing.returncode != 0:
        fail("列不出 git 跟踪的文件", "看 git 是不是可用；--all-lines 逐行判的是 git ls-files 列出的每一份。")  # change-range:exempt 出路文字点名逐行判的文件从哪条命令列出，这一行不跑 git
    judged_paths = [name for name in tracked_listing.stdout.decode("utf-8", "replace").split("\0") if name]
else:
    judged_paths = [name for name in open(changed_path, encoding="utf-8").read().split("\n") if name]

diff_segments = private_values.load_diff_segments(diff_cache_path, base, broken)   # 阶段预取的整份 diff 按文件切开；没预取到的文件 added_lines 退回逐份 git diff
findings, per_file, unreadable = [], {}, []     # (路径, 行号, 命中) / 路径 → 处数 / (路径, 错误)
files_judged, lines_judged, allowed_total, binary_files, links_skipped = 0, 0, 0, [], 0
for path in judged_paths:
    if os.path.islink(path):
        links_skipped += 1
        continue
    if not os.path.isfile(path):
        continue
    try:
        with open(path, "rb") as handle:
            head = handle.read(HEAD_BYTES)
        if b"\0" in head:
            binary_files.append(path)
            continue
        files_judged += 1
        if all_lines:
            text = open(path, "rb").read().decode("utf-8", "replace")
            lines_judged += text.count("\n") + (0 if text.endswith("\n") or not text else 1)
            hits, allowed = judge(text)
            allowed_total += allowed
            if hits:
                per_file[path] = len({(start, end) for start, end, _context, _name in hits})
            continue
        whole_file = path in untracked or broken == "out-of-project-names-whole-file"
        for number, line in private_values.added_lines(path, whole_file, base, git, fail, shown_path, diff_segments):
            lines_judged += 1
            hits, allowed = judge(line)
            allowed_total += allowed
            if hits:
                findings.append((path, number, hits))
    except OSError as error:
        unreadable.append((path, error))

private_note = None
if private_names is None:
    where = "样本根里的 .out-of-project-private-names" if in_sample else PRIVATE_LIST
    private_note = f"第二台私有名单（{where}）不在，第二台的目录名本次未查（不判红）"
    if broken != "out-of-project-names-private-missing-silent":
        print(f"  ! {private_note}；建它：一行一个名字，# 开头是注释，放第二台家目录与仓的上一级目录里的条目名")
        if not_run_file and not in_sample:
            with open(not_run_file, "a", encoding="utf-8") as handle:
                handle.write(f"doc-text.sh 的 out-of-project-names 格：{private_note}\n")

if findings:
    print(f"  ✗ 这次改动新加的 {len(findings)} 行写了不归本项目的目录名或根下系统目录的占用，在 {len({path for path, _, _ in findings})} 份文件里；只列文件、行号、类别与来源，不打印名字：")   # gate-lint:summary
    for path, number, hits in findings[:200]:
        print(f"      {shown_path(path)}:{number}  {labels(hits)}{shown_names(hits)}")   # gate-lint:detail
    if len(findings) > 200:
        print(f"      （另有 {len(findings) - 200} 行没列出）")   # gate-lint:detail
    print("     → 怎么办：改成不指名的说法（例「根分区其余不归 singlefs」）；要说空间只说本项目自己的目录与总的可用量；")
    print(f"               singlefs 自己读的仓外路径登记进放行表 {TABLE_PATH}（一行「<路径>  # 理由」，要有 git 跟踪的脚本引到它），只放行那条路径本身。")
if per_file:
    print(f"  ✗ 全仓 git 跟踪的 {files_judged} 份文本文件里 {len(per_file)} 份有不归本项目的目录名或根下系统目录的占用，共 {sum(per_file.values())} 处；只列每份的处数，不打印名字与行号：")   # gate-lint:summary
    for path in sorted(per_file):
        print(f"      {shown_path(path)}  {per_file[path]} 处")   # gate-lint:detail
    print(f"     → 怎么办：逐份改成不指名的说法（例「根分区其余不归 singlefs」），或把 singlefs 自己读的那条仓外路径登记进放行表 {TABLE_PATH}；")
    print("               已入库的冻结证据（research/prompts/ 这类）照批次定的不改；这一格平时只判这次改动新加的行，回扫后留着不改的旧行不会让它红。")
if table_problems:
    print(f"  ✗ 放行表 {TABLE_PATH} 有 {len(table_problems)} 行写得不对（只列行号，不打印路径）：")   # gate-lint:summary
    for number, what in table_problems:
        print(f"      {TABLE_PATH}:{number}  {what}")   # gate-lint:detail
    print(f"     → 怎么办：一行一条「<路径>  # 理由」，路径以 ~/ 或 / 开头，理由至少 {MINIMUM_REASON_CHARACTERS} 个字（写哪份脚本读它、读来做什么）。")
if unreferenced:
    print(f"  ✗ 放行表 {TABLE_PATH} 里 {len(unreferenced)} 条没有一份 git 跟踪的脚本（.sh、.py、.rs）字面引到（只列行号，不打印路径）：")   # gate-lint:summary
    for number in unreferenced:
        print(f"      {TABLE_PATH}:{number}  没有脚本引到这条路径（认 ~/、$HOME/、${{HOME}}/、家目录绝对路径、/home/<任一用户>/、XDG 回退与 python 拼接几种写法）")   # gate-lint:detail
    print("     → 怎么办：放行表只登记 singlefs 自己的脚本真读的仓外路径；没有脚本用它就删掉这一行，改成不指名的说法。")
if unreadable:
    print(f"  ✗ {len(unreadable)} 个文件读不了（权限这一类），这一格没法判：")   # gate-lint:summary
    for path, error in unreadable:
        print(f"      {shown_path(path)}  {errno.errorcode.get(error.errno, error.errno)}（{error.strerror}）")   # gate-lint:detail
    print("     → 怎么办：给自己补上读权限（chmod u+r）再跑；确实不该留的删掉。")
if findings or per_file or table_problems or unreferenced or unreadable:
    sys.exit(1)
if lines_judged == 0:
    print(f"  ! 这次改动没有新加的行（判了 {len(judged_paths)} 个路径，二进制 {len(binary_files)} 份），这一格本次无对象可判（这不是通过）")
    sys.exit(77)
private_count = "本次未查" if private_names is None else f"{len(private_names)} 个条目名"
scope = f"全仓 git 跟踪的 {files_judged} 份文本文件的 {lines_judged} 行" if all_lines else f"这次改动里 {files_judged} 份文本文件新加的 {lines_judged} 行"
print(f"  ✓ 查了{scope}：对着本机家目录 {len(home_names)} 个条目名、仓的上一级目录 {len(parent_names)} 个条目名、第二台私有名单 {private_count}、根下一层 {len(root_side)} 个系统目录，没有一处不归本项目的目录名与占用")
print(f"     放过的：落在放行表路径或 singlefs 自己的路径里的 {allowed_total} 处；放行表 {len(allow_entries)} 条{'都有 git 跟踪的脚本引到' if allow_entries else '（表不在或是空的）'}；没判的：二进制 {len(binary_files)} 份、符号链接 {links_skipped} 个")
PY
  rm -f "$changed_file"
  exit "$python_rc"
}

cell_utf8_valid() {
  local base changed changed_file python_output python_rc=0 utf8_break=""
  cd "$ROOT" || exit 1
  private_value_sources
  if [[ "$text_notations_break" == utf8-valid-* ]]; then
    if ((PRIVATE_IN_SAMPLE)); then
      utf8_break="$text_notations_break"
    else
      echo "  ! 弄坏开关 $text_notations_break 只在样本根里认，这一趟被判的根不是样本，没理它，照常判"
    fi
  fi
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "  ! 被判的根不在 git 仓里，划不出「这一次改动」，这一格本次无对象可判（这不是通过）"
    exit 77
  fi
  base="$(gate_diff_base gate)"
  changed="$(text_changed_paths "$base")" || { echo "  ✗ 取不到这一轮碰了哪些文件（基准 $base）"; echo "     → 怎么办：看 git 是不是可用、基准提交在不在；改动范围只经 scripts/gate/changed-paths.sh 取。"; exit 1; }
  changed_file="$(mktemp "${TMPDIR:-/tmp}/utf8-valid.XXXXXX")" || { echo "  ✗ 建不了放改动文件清单的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
  printf '%s\n' "$changed" > "$changed_file"
  python_output="$(python3 - "$changed_file" "$utf8_break" "$base" <<'PY'
import io, os, subprocess, sys

changed_path, broken, base = sys.argv[1:4]
HEAD_BYTES = 8192                            # 前 8 KiB 里有 NUL 的文件按二进制整份跳过
CHUNK_BYTES = 16 * 1024 * 1024               # 一块读多少；块边界上被切开的多字节序列带到下一块
LISTED_SKIPPED = 10                          # 「没查的」逐个列名的上限，其余只报个数
BINARY_SUFFIXES = (".gz", ".xz", ".zst", ".bz2", ".zip", ".tar", ".png", ".jpg", ".jpeg", ".gif", ".ico", ".pdf", ".bin", ".img", ".qcow2",
                   ".so", ".a", ".o", ".rlib", ".rmeta", ".wasm", ".db", ".parquet")


def git(*arguments):
    return subprocess.run(["git", "-c", "core.quotepath=false", *arguments], capture_output=True)


def fail(what, howto):
    print(f"  ✗ {what}")
    print(f"     → 怎么办：{howto}")
    sys.exit(1)


def locate_invalid_utf8(handle):
    """handle 里第一处不合法的字节：None（整份合法）或 (字节偏移从 0 起算, 行号从 1 起算, 从那里起最多 4 个字节的十六进制)。
    弄坏开关 utf8-valid-lenient：用 replace 解码，永远不报错。"""
    offset = lines_before = 0
    carry = b""
    errors = "replace" if broken == "utf8-valid-lenient" else "strict"
    while True:
        block = handle.read(CHUNK_BYTES)
        final = not block
        data = carry + block
        carry = b""
        try:
            data.decode("utf-8", errors)
        except UnicodeDecodeError as error:
            if not final and error.reason == "unexpected end of data" and error.end == len(data) and broken != "utf8-valid-chunk-carry-dropped":
                carry, data = data[error.start:], data[:error.start]          # 块尾被切开的多字节序列：带到下一块再判
            else:
                bad = " ".join("0x%02X" % byte for byte in data[error.start:error.start + 4])
                return offset + error.start, lines_before + data.count(b"\n", 0, error.start) + 1, bad
        if final:
            return None
        lines_before += data.count(b"\n")
        offset += len(data)


changed_paths = [name for name in open(changed_path, encoding="utf-8").read().split("\n") if name]
index_entries = {}                           # 路径 → (模式位, blob 哈希)，只收普通文件
listing = git("ls-files", "-z", "--stage")
if listing.returncode != 0:
    fail("列不出暂存区的文件", "看 git 是不是可用；改动范围只经 scripts/gate/changed-paths.sh 取。")
for record in listing.stdout.split(b"\0"):
    meta, _, name = record.partition(b"\t")
    fields = meta.split()
    if name and len(fields) == 3 and fields[0] in (b"100644", b"100755"):
        index_entries[os.fsdecode(name)] = (fields[0], fields[1].decode())
drift = git("diff-files", "-z", "--name-only")
if drift.returncode != 0:
    fail("列不出工作区与暂存区不同的文件", "看 git 是不是可用。")
differs_from_index = {os.fsdecode(name) for name in drift.stdout.split(b"\0") if name}
staged = git("diff", "--cached", "--name-only", "-z", base)          # 暂存区相对基准有改动的路径：提交带走的是暂存区里的这一版
if staged.returncode != 0:
    fail(f"列不出暂存区相对基准 {base[:12]} 有改动的文件", "看基准提交在不在、git 是不是可用；改动范围的基准只经 scripts/gate/changed-paths.sh 的 gate_diff_base 取。")
staged_changed = {os.fsdecode(name) for name in staged.stdout.split(b"\0") if name}

findings = []                                # (路径, 哪一版, 偏移, 行号, 坏字节)
judged_working, judged_index = 0, 0
skipped = []                                 # (路径, 为什么)
for path in changed_paths:
    if os.path.islink(path):
        skipped.append((path, "符号链接"))
        continue
    if os.path.isfile(path):
        if path.lower().endswith(BINARY_SUFFIXES):
            skipped.append((path, "二进制后缀"))
        else:
            try:
                with open(path, "rb") as handle:
                    head = handle.read(HEAD_BYTES)
                    if b"\0" in head:
                        skipped.append((path, "二进制（前 8 KiB 里有 NUL）"))
                    else:
                        handle.seek(0)
                        judged_working += 1
                        found = locate_invalid_utf8(handle)
                        if found:
                            findings.append((path, "工作区") + found)
            except OSError as error:
                skipped.append((path, f"读不了（{error.strerror}）"))
    elif path not in index_entries:
        skipped.append((path, "路径已删"))
    if path in index_entries and path in staged_changed and path in differs_from_index and broken != "utf8-valid-index-unjudged" and not path.lower().endswith(BINARY_SUFFIXES):
        blob = git("cat-file", "blob", index_entries[path][1])
        if blob.returncode != 0:
            fail(f"读不出暂存区里 {path} 的内容（blob {index_entries[path][1][:12]}）", "看 git 是不是可用、暂存区有没有损坏（git fsck）。")
        if b"\0" in blob.stdout[:HEAD_BYTES]:
            skipped.append((path + "（暂存区）", "二进制（前 8 KiB 里有 NUL）"))
            continue
        judged_index += 1
        found = locate_invalid_utf8(io.BytesIO(blob.stdout))
        if found:
            findings.append((path, "暂存区") + found)

if findings:
    print(f"  ✗ 这次改动里 {len(findings)} 份文本文件不是合法 UTF-8（字节偏移从 0 起算、行号从 1 起算，坏字节是从偏移处起最多 4 个字节）：")   # gate-lint:summary
    for path, side, offset, number, bad in findings:
        print(f"      {path}（{side}）  字节偏移 {offset}（第 {number} 行），坏字节 {bad}")   # gate-lint:detail
    print("     → 怎么办：对着行号和偏移看那一处是不是多字节字符被拦腰切断了（shell 的 paste -d、cut -c、head -c 这类按字节切的命令切多字节字符会这样），"
          "用 Edit 或 replace-once.py 按字符定点改回来；暂存区里坏而工作区已修好的，git add 重新暂存那一份；")
    print("               确实是二进制的，补进本格 BINARY_SUFFIXES（.claude/gate.d/lib/doc-text-content-and-history-cells.sh 里 cell_utf8_valid 的后缀名单），不要放过文本。")
    sys.exit(1)
if judged_working + judged_index == 0:
    print(f"  ! 这次改动里没有文本文件可判（改动范围 {len(changed_paths)} 个路径，没查 {len(skipped)} 份），这一格本次无对象可判（这不是通过）")
    sys.exit(77)
print(f"  ✓ 查了这次改动里 {judged_working} 份工作区文本文件、{judged_index} 份暂存区里与工作区不同的版本：整份按 UTF-8 严格解码，没有一处非法字节")
listed = "；".join(f"{path}（{why}）" for path, why in skipped[:LISTED_SKIPPED])
extra = f"；…另有 {len(skipped) - LISTED_SKIPPED} 份" if len(skipped) > LISTED_SKIPPED else ""
print(f"     没查的 {len(skipped)} 份：{listed or '无'}{extra}")
PY
)" || python_rc=$?
  rm -f "$changed_file"
  if [[ -n "$python_output" ]]; then
    # 红行与「没查的」里的路径经 private-values 那一份打码（shown_lines_main 的 lines：带 / 的一段当路径，别的当行文）
    printf '%s\n' "$python_output" | python3 -c 'import sys, types; library = types.ModuleType("lib_private_values"); exec(compile(sys.argv[1], "private-values-library", "exec"), library.__dict__); library.shown_lines_main(sys.argv[2:], sys.stdin, sys.stdout)' \
      "$PRIVATE_VALUES_LIBRARY" "$PRIVATE_CONFIGURATION_FILE" "$PRIVATE_HOSTNAME_FILE" "$text_notations_break" lines
  fi
  exit "$python_rc"
}

# 判法与滞后表的写法在文件头「dated-status」一段。kb 目录登记只经上游 lib.sh 的 kb_directories 读（它 set -euo pipefail，放进单独的 bash 里起，不带进这一道）。
cell_dated_status() {
  local base="" registry_errors kb_listing python_rc=0
  cd "$ROOT" || exit 1
  private_value_sources
  registry_errors="$(mktemp "${TMPDIR:-/tmp}/dated-status.XXXXXX")" || { echo "  ✗ 建不了放 kb 目录登记报错的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
  if ! kb_listing="$(bash -c 'source "$1" && kb_directories "$2"' dated-status "$REPO_ROOT/.claude/singlefs-ai-sop/scripts/lib.sh" "$(pwd)" 2>"$registry_errors")"; then
    echo "  ✗ 读不出 kb 目录登记 .claude/kb-directories（上游 lib.sh 的 kb_directories 报错，原文在下面）"   # gate-lint:summary
    sed 's/^/      /' "$registry_errors"
    echo "     → 怎么办：照报错改 .claude/kb-directories：一行一条「<相对项目根的目录>  # 为什么」，目录要在，理由不许省（.claude/singlefs-ai-sop/rules/kb-discipline.md 开头一段）。"
    rm -f "$registry_errors"
    exit 1
  fi
  rm -f "$registry_errors"
  if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    base="$(gate_diff_base gate)"
  fi
  python3 - "$(pwd)" "$kb_listing" "$base" "$STAGE_DIR/lib-lag-baseline.py" "$text_notations_break" "$PRIVATE_VALUES_LIBRARY" "$PRIVATE_CONFIGURATION_FILE" "$PRIVATE_HOSTNAME_FILE" <<'PY' || python_rc=$?
import importlib.util, os, re, sys, types

root, kb_listing, base, lag_baseline_path, broken, private_values_source, configuration_path, hostname_path = sys.argv[1:9]
private_values = types.ModuleType("lib_private_values")      # 路径与片段打码的共用函数（doc-text.sh 的 PRIVATE_VALUES_LIBRARY，与别的格同一份）
exec(compile(private_values_source, "private-values-library", "exec"), private_values.__dict__)
masking_targets = []
if configuration_path:
    try:
        masking_targets = private_values.load_private_targets(configuration_path, hostname_path, broken)[0]
    except OSError:
        masking_targets = []

def shown_path(path):
    return private_values.shown_path(path, masking_targets, broken)

def shown_snippet(line, start, end):
    start, end = private_values.snippet_bounds(line, max(0, start), min(len(line), end), masking_targets)
    return private_values.shown_text(line[start:end].strip(), masking_targets)

LAG_TABLE = ".claude/gate.d/dated-status-lag.tsv"
DEFAULT_KB_DIRECTORY = ".claude/kb"                       # 没有 .claude/kb-directories 时的 kb 目录（kb-discipline.md 开头一段）
HISTORY_HEADING = re.compile(r"^## 历史版本\s*$")
NOT_JUDGED_NAMES = ("decisions-history.md", "experiments-history.md", "INDEX.md")
TRANSLATION_SUFFIXES = (".en.md", ".ja.md")
DATED_STATUS_FORMS = (                                   # 两种带日期的现状句，红行报第一处命中的那一种
    ("现状（日期", re.compile(r"现状(?:\*\*)?\s*[（(]\s*[0-9]{4}-[0-9]{2}-[0-9]{2}")),
    ("日期 现状", re.compile(r"(?<![0-9])[0-9]{4}-[0-9]{2}-[0-9]{2} ?(?:\*\*)?现状")),
)
MINIMUM_ANCHOR_CHARACTERS = 12
MINIMUM_REASON_CHARACTERS = 4
LISTED_FINDINGS = 200

# ── 扫哪些页 ──
kb_directories = [os.path.relpath(line, root) for line in kb_listing.split("\n") if line.strip()]
if not kb_directories and os.path.isdir(DEFAULT_KB_DIRECTORY):
    kb_directories = [DEFAULT_KB_DIRECTORY]
judged_files, skipped_translations, skipped_named = [], 0, 0
for directory in kb_directories:
    for current, subdirectories, names in os.walk(directory):
        subdirectories.sort()
        for name in sorted(names):
            if not name.endswith(".md"):
                continue
            if name.endswith(TRANSLATION_SUFFIXES):
                skipped_translations += 1
                continue
            if name in NOT_JUDGED_NAMES:
                skipped_named += 1
                continue
            path = os.path.normpath(os.path.join(current, name))
            if os.path.isfile(path) and path not in judged_files:
                judged_files.append(path)

# ── 逐份找命中：只看「## 历史版本」之前 ──
hits = []            # (路径, 行号, 那一行, 形态名, 命中起点, 命中终点)
lines_judged = 0
unreadable = []
for path in judged_files:
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            text_lines = handle.read().split("\n")
    except OSError as error:
        unreadable.append((path, error))
        continue
    for number, line in enumerate(text_lines, 1):
        if HISTORY_HEADING.match(line):
            break
        lines_judged += 1
        for form_name, pattern in DATED_STATUS_FORMS:
            found = pattern.search(line)
            if found:
                hits.append((path, number, line, form_name, found.start(), found.end()))
                break

# ── 滞后表：读、核写法、与 gate 基准那一版比 ──
table_problems = []  # (表的行号, 一句问题)
growth = []          # (表的行号, 路径, 锚)
lag_rows = []        # 写对了的登记 (表的行号, 路径, 锚)
table_rows = 0
base_note = ""
spec = importlib.util.spec_from_file_location("lib_lag_baseline", lag_baseline_path)
lag_baseline = importlib.util.module_from_spec(spec)
spec.loader.exec_module(lag_baseline)
file_lines_cache = {}
def lines_of(path):
    if path not in file_lines_cache:
        with open(path, encoding="utf-8", errors="replace") as handle:
            file_lines_cache[path] = handle.read().split("\n")
    return file_lines_cache[path]

if broken == "dated-status-lag-unread":
    print("  ! 弄坏开关 dated-status-lag-unread：不读滞后表，命中的一律判红")
    base_note = "弄坏开关 dated-status-lag-unread：没读滞后表"
elif os.path.isfile(LAG_TABLE):
    with open(LAG_TABLE, encoding="utf-8", errors="replace") as handle:
        current_rows = lag_baseline.content_rows(handle.read())
    table_rows = len(current_rows)
    seen_keys = {}
    for row_number, columns in current_rows:
        if len(columns) != 3:
            table_problems.append((row_number, f"有 {len(columns)} 列，要三列：<路径><TAB><行内锚><TAB># <为什么还没改>"))
            continue
        path, anchor, reason = columns[0].strip(), columns[1].strip(), columns[2].strip()
        if not reason.startswith("#") or len(reason.lstrip("#").strip()) < MINIMUM_REASON_CHARACTERS:
            table_problems.append((row_number, f"第三列要以 # 起头、写为什么还没改（至少 {MINIMUM_REASON_CHARACTERS} 个字）"))
            continue
        if len(anchor) < MINIMUM_ANCHOR_CHARACTERS:
            table_problems.append((row_number, f"锚只有 {len(anchor)} 个字，要那一行里至少 {MINIMUM_ANCHOR_CHARACTERS} 个字的原文"))
            continue
        if (path, anchor) in seen_keys:
            table_problems.append((row_number, f"与第 {seen_keys[(path, anchor)]} 行登记的是同一条（路径与锚都相同）"))
            continue
        seen_keys[(path, anchor)] = row_number
        if not os.path.isfile(path):
            table_problems.append((row_number, f"登记的路径不在：{shown_path(path)}"))
            continue
        try:
            anchored = [number for number, line in enumerate(lines_of(path), 1) if anchor in line]
        except OSError as error:
            table_problems.append((row_number, f"读不了登记的文件 {shown_path(path)}（{error.strerror}）"))
            continue
        if len(anchored) != 1:
            table_problems.append((row_number, f"锚在 {shown_path(path)} 里命中 {len(anchored)} 行，要恰好一行" + (f"（第 {'、'.join(map(str, anchored[:5]))} 行）" if anchored else "")))
            continue
        lag_rows.append((row_number, path, anchor))
    if broken == "dated-status-growth-unjudged":
        base_note = "弄坏开关 dated-status-growth-unjudged：没与 gate 基准比"
    elif not base:
        base_note = "被判的根不在 git 仓里，滞后表没与 gate 基准比"
    else:
        base_rows, problem = lag_baseline.baseline_rows(base, LAG_TABLE, cwd=root)
        if problem is not None:
            table_problems.append((0, f"与 gate 基准比不了：{problem.sentence()}"))
        elif base_rows is None:
            base_note = f"gate 基准 {base} 里还没有滞后表（初次登记），不比"
        else:
            base_keys = {(columns[0].strip(), columns[1].strip()) for _number, columns in base_rows if len(columns) >= 2}
            for row_number, columns in current_rows:
                if len(columns) >= 2 and (columns[0].strip(), columns[1].strip()) not in base_keys:
                    growth.append((row_number, columns[0].strip(), columns[1].strip()))
            base_note = f"与 gate 基准 {base} 那一版比没涨（基准 {len(base_rows)} 行，此刻 {table_rows} 行）"
else:
    base_note = "没有滞后表"

lagged, findings = [], []
for hit in hits:
    path, number, line = hit[0], hit[1], hit[2]
    if any(row_path == path and anchor in line for _row, row_path, anchor in lag_rows):
        lagged.append(hit)
    else:
        findings.append(hit)
hit_lines = {(hit[0], hit[1]) for hit in hits}
stale_rows = [row for row in lag_rows if (row[1], next(number for number, line in enumerate(lines_of(row[1]), 1) if row[2] in line)) not in hit_lines]

failed = False
if findings:
    failed = True
    print(f"  ✗ {len(findings)} 行带日期的现状句不在滞后表里，在 {len({hit[0] for hit in findings})} 份 kb 中文页的历史节之前：")   # gate-lint:summary
    for path, number, line, form_name, start, end in findings[:LISTED_FINDINGS]:
        print(f"      {shown_path(path)}:{number}  {form_name}  …{shown_snippet(line, start - 10, end + 24)}…")   # gate-lint:detail
    if len(findings) > LISTED_FINDINGS:
        print(f"      （另有 {len(findings) - LISTED_FINDINGS} 行没列出）")   # gate-lint:detail
    print("     → 怎么办：删掉日期，把那一句的事实现查一次（evidence-discipline.md）改成现值；那一天的样子要留就写进「## 历史版本」（只有登记在 .claude/history-carriers 的文件有这一节），")
    print("               没登记的文件不留历史，经过靠 git。规则：.claude/singlefs-ai-sop/rules/kb-discipline.md「带日期的现状快照也是历史，会停在那一天」。")
if table_problems:
    failed = True
    print(f"  ✗ 滞后表 {LAG_TABLE} 有 {len(table_problems)} 处写得不对：")   # gate-lint:summary
    for row_number, sentence in table_problems:
        print(f"      {'第 ' + str(row_number) + ' 行：' if row_number else ''}{sentence}")   # gate-lint:detail
    print(f"     → 怎么办：一行「<路径><TAB><那一行里至少 {MINIMUM_ANCHOR_CHARACTERS} 个字、在文件里只出现一次的原文><TAB># <为什么还没改，至少 {MINIMUM_REASON_CHARACTERS} 个字>」；")
    print("               路径不在或锚找不到的多半是那一行已经改好或文件搬了家，删掉这一行；锚命中几行的换一段更长、只出现一次的原文。")
if growth:
    failed = True
    print(f"  ✗ 滞后表比 gate 基准 {base} 那一版多 {len(growth)} 行（只缩不涨；路径或锚改了也算新加）：")   # gate-lint:summary
    for row_number, path, anchor in growth:
        print(f"      第 {row_number} 行：{shown_path(path)}  …{private_values.shown_text(anchor, masking_targets)}…")   # gate-lint:detail
    print("     → 怎么办：新写的带日期现状句当场改成现值，不往滞后表里加；那一行改过字、锚对不上了的，在同一次改动里把那一句改成现值、删掉它那一行，不改锚。")
if unreadable:
    failed = True
    print(f"  ✗ {len(unreadable)} 份 kb 中文页读不了，这几份没判：")   # gate-lint:summary
    for path, error in unreadable:
        print(f"      {shown_path(path)}（{error.strerror}）")   # gate-lint:detail
    print("     → 怎么办：给自己补上读权限（chmod u+r）再跑这一格。")
for row_number, path, anchor in stale_rows:
    print(f"  ! 滞后表第 {row_number} 行指的那一行（{shown_path(path)}）已经不是带日期的现状句，删掉这一行（只缩不涨，删了不用补）")
if failed:
    sys.exit(1)
if not judged_files:
    print(f"  ! 登记的 kb 目录（{'、'.join(kb_directories) or '没有'}）下没有中文 .md，这一格本次无对象可判（这不是通过）")
    sys.exit(77)
print(f"  ✓ 带日期的现状句：扫了 {len(kb_directories)} 个 kb 目录里 {len(judged_files)} 份中文页历史节之前的 {lines_judged} 行，命中 {len(hits)} 行，不在滞后表里的 0 行；滞后 {len(lagged)} 行（滞后表 {table_rows} 行，{base_note}）")
print(f"     没扫的：译文 {skipped_translations} 份、两份变更史与 INDEX.md {skipped_named} 份")
PY
  exit "$python_rc"
}

# ── comment-history ──
# 判法整份在 scripts/gate/comment-history-check.py（文件头「comment-history」一段写判据、两张表的写法与弄坏开关）；这一格只起它、退出码原样交回。
# 被判的根给它当参数；环境里的弄坏开关 COMMENT_HISTORY_CHECK_BREAK 原样带进去（stage_cells_run 开头已拦真仓）。
cell_comment_history() {
  local python_rc=0
  cd "$ROOT" || exit 1
  python3 "$REPO_ROOT/scripts/gate/comment-history-check.py" "$ROOT" || python_rc=$?
  exit "$python_rc"
}

# text_changed_paths <基准>：交 gate_changed_paths <基准> untracked 的结果。下面「预取」一段在父进程里取过同一个基准的，直接给取好的（各格各取一遍要起两条 git diff --name-only）；没取过的现取。
# 取不到的退 1，各格照旧各自判红；预取过的取不到时，把预取时记下的 git 报错（text_prefetched_error）先打到标准错误，各格的日志里就有 git 的原文。
# 弄坏开关 changed-paths-prefetch-empty：预取过的一律交空表。
text_changed_paths() {
  if [[ -n "$text_prefetched_base" && "$1" == "$text_prefetched_base" ]]; then
    if ((text_prefetched_status != 0)); then
      [[ "$text_notations_break" == prefetch-error-dropped ]] || printf '%s\n' "${text_prefetched_error:-  ! 预取改动路径失败，gate_changed_paths 没有留下报错}" >&2
      return 1
    fi
    [[ "$text_notations_break" == changed-paths-prefetch-empty ]] && return 0
    [[ -z "$text_prefetched_changed" ]] || printf '%s\n' "$text_prefetched_changed"
    return 0
  fi
  gate_changed_paths "$1" untracked || return 1
}
