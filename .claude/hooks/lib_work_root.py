"""工作根：子 agent 的草稿目录、报告、补丁与冻结副本，钩子与看门狗的状态和检出记录，放在哪只在这一份里定义。

工作根：环境变量 SINGLEFS_WORK_ROOT 设了就用它，没设是 DEFAULT_WORK_ROOT（~/.local/share/singlefs/work，在 $HOME 下，开机不清）。
旧根 /tmp/claude-<uid>：过渡期照样认，写范围闸、派发闸、交回正文闸、Bash 检出 hook 与看门狗认路径时工作根与旧根都放行。
  撤旧根的条件：在跑的子 agent 与别的会话的子 agent 都交回之后，由主 agent 撤。
  撤法：LEGACY_ROOT_IN_TRANSITION 改成 False；用它的几份自证里「过渡期旧根下」那几格的应得改成拒（各份文件头的弄坏开关一段点了名）。

用法：
  python  按文件路径导入（importlib.util.spec_from_file_location，与 lib_shell_words.py 的导入法相同），或把 .claude/hooks 放进 sys.path 再 import
  shell   work_root="$(python3 <仓根>/.claude/hooks/lib_work_root.py)"：打一行展开后的工作根，不建目录
  写范围表 .claude/hooks/agent-write-scope.tsv 的路径模式以 $SINGLEFS_WORK_ROOT/ 起头的，由 expand_scope_pattern 展开成工作根与旧根各一条

弄坏开关（只给用它的钩子与脚本的自证判红用）：SINGLEFS_WORK_ROOT_BREAK=
  new-root-unknown     work_root() 交回旧根，scratch_roots() 与 mention_prefix_pattern() 只认旧根：各份自证里「工作根下」那几格红
  legacy-root-unknown  scratch_roots() 与 mention_prefix_pattern() 不认旧根：各份自证里「过渡期旧根下」那几格红
不认识的值抛 ValueError，免得拼错的开关被当成「没开」。
"""
import os
import re
import sys

WORK_ROOT_VARIABLE = "SINGLEFS_WORK_ROOT"
DEFAULT_WORK_ROOT = "~/.local/share/singlefs/work"
SCOPE_TOKEN = "$" + WORK_ROOT_VARIABLE
LEGACY_ROOT_IN_TRANSITION = True
BREAK_VARIABLE = "SINGLEFS_WORK_ROOT_BREAK"
KNOWN_BREAK_MODES = ("new-root-unknown", "legacy-root-unknown")
DETECTIONS_FILE_NAME = "agent-hook-detections.jsonl"


def break_mode():
    mode = os.environ.get(BREAK_VARIABLE, "")
    if mode and mode not in KNOWN_BREAK_MODES:
        raise ValueError(f"{BREAK_VARIABLE}={mode} 不认识，只认 {'、'.join(KNOWN_BREAK_MODES)}")
    return mode


def legacy_root():
    """过渡期旧根 /tmp/claude-<uid>（开机被清空）。"""
    return f"/tmp/claude-{os.getuid()}"


def home_directory():
    return os.path.normpath(os.path.expanduser("~"))


def configured_work_root():
    """配置的工作根（SINGLEFS_WORK_ROOT 或默认值，展开成绝对路径），不看弄坏开关：自证拿它造「工作根下」的样本路径。"""
    return os.path.normpath(os.path.abspath(os.path.expanduser(os.environ.get(WORK_ROOT_VARIABLE) or DEFAULT_WORK_ROOT)))


def work_root():
    """展开后的工作根（绝对路径，不带末尾的 /）；不建目录。"""
    if break_mode() == "new-root-unknown":
        return legacy_root()
    return configured_work_root()


def scratch_roots():
    """认作草稿与报告位置的根：工作根在前，过渡期旧根在后（去重）。"""
    mode = break_mode()
    roots = [] if mode == "new-root-unknown" else [work_root()]
    if LEGACY_ROOT_IN_TRANSITION and mode != "legacy-root-unknown":
        roots.append(legacy_root())
    return tuple(dict.fromkeys(roots))


def expand_written_path(path):
    """文字里写的路径换成绝对写法：$SINGLEFS_WORK_ROOT/、${SINGLEFS_WORK_ROOT}/ 换成工作根，~/、$HOME/、${HOME}/ 换成家目录；别的原样。"""
    for prefix in (SCOPE_TOKEN, "${" + WORK_ROOT_VARIABLE + "}"):
        if path == prefix or path.startswith(prefix + "/"):
            return work_root() + path[len(prefix):]
    for prefix in ("$HOME", "${HOME}"):
        if path == prefix or path.startswith(prefix + "/"):
            return home_directory() + path[len(prefix):]
    return os.path.expanduser(path)


def is_under_scratch_root(path):
    """path（文字里的写法也认）落在工作根或过渡期旧根底下（含根本身）。"""
    absolute = os.path.normpath(os.path.abspath(expand_written_path(path)))
    return any(absolute == root or absolute.startswith(root + "/") for root in scratch_roots())


def mention_prefix_pattern():
    """正则片段：文字里一条路径落在工作根或旧根底下时的开头那一截（含根后面的 /）。
    工作根认四种写法：展开后的绝对路径、$SINGLEFS_WORK_ROOT/（与 ${…}/），在家目录下时另认 ~/…、$HOME/…、${HOME}/…；旧根认 /tmp/claude-<数字>/。"""
    mode = break_mode()
    forms = []
    if mode != "new-root-unknown":
        root = work_root()
        written = [root + "/", SCOPE_TOKEN + "/", "${" + WORK_ROOT_VARIABLE + "}/"]
        home = home_directory()
        if root.startswith(home + "/"):
            relative = root[len(home) + 1:]
            written += [f"~/{relative}/", f"$HOME/{relative}/", "${HOME}/" + relative + "/"]
        forms += [re.escape(form) for form in written]
    if LEGACY_ROOT_IN_TRANSITION and mode != "legacy-root-unknown":
        forms.append(r"/tmp/claude-\d+/")
    return "(?:" + "|".join(forms) + ")" if forms else "(?!)"


def expand_scope_pattern(pattern, anchored=True):
    """写范围表的一条路径模式：以 $SINGLEFS_WORK_ROOT/ 起头的展开成每个草稿根各一条，别的原样（交回列表）。
    anchored 为假只给写范围闸的弄坏开关用：根后面不带 /，同名前缀的兄弟目录也被放行。"""
    if not pattern.startswith(SCOPE_TOKEN + "/"):
        return [pattern]
    rest = pattern[len(SCOPE_TOKEN):] if anchored else pattern[len(SCOPE_TOKEN) + 1:]
    return [root + rest for root in scratch_roots()]


def default_detections_path():
    """钩子检出记录的默认位置（各钩子与看门狗另认环境变量 AGENT_HOOK_DETECTIONS）。"""
    return os.path.join(work_root(), DETECTIONS_FILE_NAME)


def state_directory(name):
    """钩子与看门狗的状态目录：工作根下的 <name>。"""
    return os.path.join(work_root(), name)


if __name__ == "__main__":
    try:
        print(work_root())
    except ValueError as error:
        print(f"  ✗ {error}", file=sys.stderr)
        print(f"  → 怎么办：弄坏开关只给自证判红用，平时不设 {BREAK_VARIABLE}", file=sys.stderr)
        sys.exit(2)
