"""三方腿类型表：哪几种子 agent 类型是三方腿，只登记在各份定义的 frontmatter 里（`leg: three-way` 一行），这一份只管读它。

四处用这一份，不各抄一份类型名：
  .claude/hooks/runner-dispatch-guard.sh   ⑰ 三方腿的报告形态只对这几种判
  .claude/hooks/handback-guard.sh          ② 交回里点名的 research/prompts/…-output.md 交 cite-check.py，只对这几种判
  scripts/agents/dispatch-prompt.py        骨架的「报告：」一行，这几种填 research/prompts/<填 轮>-<填 模型>-output.md 占位
  scripts/agents/agent-cost.py             outcome 的两个三方腿桶，只收这几种

读法（three_way_leg_types）：<仓根>/.claude/agents/ 下每份 .md，开头一行是 `---`，到下一行 `---` 为止是 frontmatter；
其中 `leg:` 那一行的值（去掉行尾 # 注释与首尾空白）等于 `three-way` 的，文件名去掉 .md 就是一种三方腿类型。
正文里写的 `leg: three-way` 不算；目录不在、一份都没登记，交回空集（调用方各自决定空集怎么办）。在而读不了的定义跳过，unreadable_definitions 交出它们，
两份钩子照「放行但必记检出」记一处没判全。
每处的自证各有一格，拿样本仓与本仓两份登记核它们读到的集合与 frontmatter 逐个相同；「frontmatter 里写着什么」取 three_way_leg_types_scanned
（不拆键值，在 frontmatter 那一段里按整行找），四处的自证共用它，不各写一份。

三方腿的报告路径也只在这一份里认，三处认法都经同一个函数 leg_report_mentions_in_trees（判决 research/prompts/landing-audit-r4-main-verification.md K2-5，
拆分提案第九十节 ㉗：原来派发闸报告行那一支只去仓根前缀、整份提示那一支与交回闸 ② 各认各的），两份钩子调它：
  .claude/hooks/runner-dispatch-guard.sh   ⑰ 报告行点名的那一个词整个是不是这一形态、解到哪一棵（leg_report_written_tree），整份提示里那几处（leg_report_mentions）
  .claude/hooks/handback-guard.sh          ② 交回里点名的报告（leg_report_mentions_in_trees），逐份交 cite-check.py
  leg_report_written_tree 与 leg_report_mentions 都只是在 leg_report_mentions_in_trees 外面包一层，不另写认法；is_leg_report_path 只认从仓根起的写法、不解树，
  两份钩子都不再调它，只给派发闸的弄坏开关 leg-report-line-root-only（退回改前报告行那一支）用。
形态：research/prompts/<名>-output.md，<名> 里没有空白、/、尖括号、引号与反引号、中文标点与括号（LEG_REPORT_NAME）：
  没填的占位（带 <、>，「<轮>-opus-output.md」）不认，子目录下的（research/prompts/<子目录>/…-output.md）不认；名字里的中文照认（骨架自证把占位填成「已填」）。
leg_report_mentions(文字, 仓根) 在一段文字里找（LEG_REPORT_MENTION）：前面不连着字母、数字、_ . -，后面不接字母、数字、_ -，也不接「.<字母数字>」；
  再只留解到本仓某一棵工作树的 research/prompts/ 那一层的（判决 landing-audit-r5「交回闸 ② 仓外报告」，拆分提案第九十节 ㉜）：往前连着的那一截路径
  （遇到空白、引号、反引号、尖括号、中英文标点与括号、= 与 : 就停，LEG_REPORT_PREFIX）是空的、./、某一棵工作树的根（绝对路径，或 ~、$HOME、${HOME} 起头展开后是它），
  或从仓根起的相对写法解到其中一棵（../<仓根目录名>/ 这类）；不在任何一棵里的仓外副本、带别的变量的不认。
  工作树（registered_trees）：仓根，加上 `git -C <仓根> worktree list --porcelain` 列出的每一棵（bare 的、目录不在的不收）；取不到列表（不是 git 仓、
  git 起不来、超时、退非 0）时只有仓根，原因交给调用方（交回闸写进拒绝说明）。同一个仓根一个进程里只取一次。
leg_report_mentions_in_trees(文字, 仓根[, 工作树]) 是同一处认法、另交回每处解到的那一棵（交回闸 ② 按那一棵当根核引文）；leg_report_mentions 只是去掉那一棵。
leg_report_written_tree(写法, 仓根[, 工作树])：一个词（派发闸报告行点名的那一个，首尾的反引号与句末标点由调用方去掉）整个是这一形态时交回它解到的那一棵，
  不是交 None：拿 leg_report_mentions_in_trees 在这个词里认，要恰好一处、从词首连到词尾（前面那一截整个是 LEG_REPORT_PREFIX 认的路径），
  所以 ./、仓根或某一棵工作树的绝对路径、~ / $HOME / ${HOME} 起头的、../<仓根目录名>/ 这类相对写法都认，占位、子目录下的、仓外副本里的都不认，与整份提示那一支、交回闸 ② 判得一样。
"""
import glob
import os
import re
import subprocess

LEG_LINE = re.compile(r"(?m)^leg:[ \t]*three-way[ \t]*(?:#.*)?$")
LEG_REPORT_NAME = r"[^\s/<>`'\"，。；：、（）()「」『』“”,;*|]+"
LEG_REPORT_PATH = re.compile(r"research/prompts/" + LEG_REPORT_NAME + r"-output\.md")
LEG_REPORT_MENTION = re.compile(r"(?<![A-Za-z0-9_.\-])research/prompts/" + LEG_REPORT_NAME + r"-output\.md(?![A-Za-z0-9_\-]|\.[A-Za-z0-9_])")
LEG_REPORT_PREFIX = re.compile(r"[^\s<>`'\"，。；：、（）()「」『』“”,;*|=:\[\]{}]*\Z")
LEG_REPORT_PREFIX_WINDOW = 4096
HOME_WRITINGS = ("~/", "$HOME/", "${HOME}/")
WORKTREE_LIST_TIMEOUT_SECONDS = 10
# 钩子在 git 钩子或 git 起的子进程里跑时，这几个变量会把 git -C 指的仓换成别的
GIT_LOCATION_VARIABLES = ("GIT_DIR", "GIT_WORK_TREE", "GIT_COMMON_DIR", "GIT_INDEX_FILE")
REGISTERED_TREES = {}

LEG_KEY = "leg"
THREE_WAY_LEG = "three-way"


def frontmatter_fields(path):
    """一份定义的 frontmatter → {键: 值}（值去掉行尾 # 注释与首尾空白）；读不了或开头不是 `---` 交回 None。"""
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
    except OSError:
        return None
    if not lines or lines[0].strip() != "---":
        return None
    fields = {}
    for line in lines[1:]:
        if line.strip() == "---":
            break
        if ":" in line:
            key, value = line.split(":", 1)
            fields[key.strip()] = value.split("#", 1)[0].strip()
    return fields


def three_way_leg_types(project_root):
    """<project_root>/.claude/agents/ 里 frontmatter 写了 `leg: three-way` 的类型名 → frozenset。"""
    found = set()
    for path in glob.glob(os.path.join(project_root, ".claude", "agents", "*.md")):
        fields = frontmatter_fields(path)
        if fields is not None and fields.get(LEG_KEY) == THREE_WAY_LEG:
            found.add(os.path.basename(path)[: -len(".md")])
    return frozenset(found)


def unreadable_definitions(project_root):
    """<project_root>/.claude/agents/ 里在而读不了的定义 → [路径]（排好序）：three_way_leg_types 跳过它们，三方腿类型表这一回读不全；
    两份钩子拿它照「放行但必记检出」记一处没判全（交回闸 ②、派发闸 ⑰ 的 leg-definitions 那一处）。"""
    unreadable = []
    for path in sorted(glob.glob(os.path.join(project_root, ".claude", "agents", "*.md"))):
        try:
            with open(path, encoding="utf-8") as handle:
                handle.read(1)
        except OSError:
            unreadable.append(path)
    return unreadable


def three_way_leg_types_scanned(project_root):
    """自证用的另一种读法，当「frontmatter 里写着什么」：每份定义开头是 `---` 的，取到下一处换行接 `---` 为止的那一段，
    整行是 `leg: three-way`（可带行尾 # 注释）的算登记了。不拆键值，与 three_way_leg_types 走的不是同一条路 → 排好序的列表。"""
    found = set()
    for path in glob.glob(os.path.join(project_root, ".claude", "agents", "*.md")):
        try:
            with open(path, encoding="utf-8") as handle:
                head = handle.read().split("\n---", 1)[0]
        except OSError:
            continue
        if head.startswith("---") and LEG_LINE.search(head):
            found.add(os.path.basename(path)[: -len(".md")])
    return sorted(found)


def is_leg_report_path(relative):
    """relative（从仓根起的写法）整个是 research/prompts/<名>-output.md 这一形态：占位与子目录都不算。不解树；两份钩子都不再调它（见文件头）。"""
    return bool(LEG_REPORT_PATH.fullmatch(relative or ""))


def same_tree(path, tree):
    return os.path.normpath(path) == os.path.normpath(tree) or os.path.realpath(path) == os.path.realpath(tree)


def registered_trees(project_root):
    """本仓的工作树 → (树, 错误)：树是 [仓根, 再照 git worktree list --porcelain 的次序排登记过的别的各棵]，都是 normpath 的绝对路径；
    bare 的、登记了而目录不在的不收，与已收的是同一棵的不重收。取不到列表时树只有仓根，错误写原因（取到了是 None）。"""
    root = os.path.normpath(os.path.abspath(project_root))
    if root in REGISTERED_TREES:
        return REGISTERED_TREES[root]
    trees, error = [root], None
    environment = {key: value for key, value in os.environ.items() if key not in GIT_LOCATION_VARIABLES}
    try:
        listed = subprocess.run(["git", "-C", root, "worktree", "list", "--porcelain"], capture_output=True, text=True,
                                timeout=WORKTREE_LIST_TIMEOUT_SECONDS, env=environment)
    except (OSError, subprocess.SubprocessError) as caught:
        error = f"git worktree list 起不来：{caught}"
    else:
        if listed.returncode != 0:
            error = f"git worktree list 退 {listed.returncode}：{listed.stderr.strip()[-200:]}"
        for entry in ([] if error else listed.stdout.split("\n\n")):
            lines = entry.splitlines()
            if not lines or not lines[0].startswith("worktree ") or "bare" in lines[1:]:
                continue
            path = os.path.normpath(lines[0][len("worktree "):])
            if os.path.isdir(path) and not any(same_tree(path, tree) for tree in trees):
                trees.append(path)
    REGISTERED_TREES[root] = (trees, error)
    return trees, error


def prefix_tree(prefix, project_root, trees):
    """报告路径前面连着的那一截（research/prompts/ 之前）解到 trees 里的哪一棵 → 那一棵，都不是交 None：
    空的与 ./ 是仓根；~ / $HOME / ${HOME} 起头的先展开；相对写法从仓根起算；展开之后还带 $ 或 ~ 的不认。"""
    if prefix in ("", "./"):
        return trees[0]
    written = prefix
    for home in HOME_WRITINGS:
        if written.startswith(home):
            written = os.path.join(os.path.expanduser("~"), written[len(home):])
            break
    if "$" in written or "~" in written:
        return None
    candidate = os.path.normpath(written if os.path.isabs(written) else os.path.join(os.path.normpath(project_root), written))
    return next((tree for tree in trees if same_tree(candidate, tree)), None)


def leg_report_mentions_in_trees(text, project_root, trees=None):
    """一段文字里点名的、解到某一棵工作树 research/prompts/ 那一层的三方腿报告路径（写法见文件头）→ [(re.Match, 那一棵)]，按出现次序；
    trees 不给就取 registered_trees(project_root)。仓外副本里的不认。"""
    if trees is None:
        trees = registered_trees(project_root)[0]
    text = text or ""
    kept = []
    for match in LEG_REPORT_MENTION.finditer(text):
        window = text[max(0, match.start() - LEG_REPORT_PREFIX_WINDOW):match.start()]
        tree = prefix_tree(LEG_REPORT_PREFIX.search(window).group(0), project_root, trees)
        if tree is not None:
            kept.append((match, tree))
    return kept


def leg_report_mentions(text, project_root):
    """同 leg_report_mentions_in_trees，只交回 [re.Match]；调用方要位置（去掉引号里的）就看 match.start()。"""
    return [match for match, _ in leg_report_mentions_in_trees(text, project_root)]


def leg_report_written_tree(written, project_root, trees=None):
    """一个词整个是三方腿报告路径时交回它解到的那一棵，不是交 None（写法见文件头）：认法就是 leg_report_mentions_in_trees，
    要恰好认到一处、那一处到词尾为止、前面那一截整个是路径（LEG_REPORT_PREFIX 从词首认到那一处）。"""
    written = written or ""
    found = leg_report_mentions_in_trees(written, project_root, trees)
    if len(found) != 1:
        return None
    match, tree = found[0]
    if match.end() != len(written) or LEG_REPORT_PREFIX.fullmatch(written[:match.start()]) is None:
        return None
    return tree
