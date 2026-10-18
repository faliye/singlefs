#!/usr/bin/env python3
# admission: always 判的是此刻共用索引与 HEAD 的关系，每做完一次提交都要现算，上一次的结论不替这一次作保
# run-condition: command git python3
# gate-similar: stage-mine.py 它管「几个会话共写一批文件时只暂存这一轮的块」与 --write-index 直接写索引，是提交之前的一半；这里管提交之后共用索引有没有停在旧 HEAD 上，只读、不写任何索引，判的对象与时刻都不同
# gate-similar: gate-staged.sh 它在提交前整轮门禁里用暂存区，不看提交之后共用索引与 HEAD 的差
# gate-overlap:copy-kept host-resources-reminder.sh 自证末尾「逐个弄坏开关、起 --selftest-cells、核红在点名的格」的轮次样板各份各有自己的开关表与格名；抽成共用库要改 host-resources-reminder.sh 的自证，不在这一批的范围
"""提交之后查共用索引有没有停在旧 HEAD 上（C420）：几个会话共写一个仓时，有人用 GIT_INDEX_FILE 指私有索引暂存并提交，共用索引（git 目录里的 index）一个字没动，
提交之后它存的还是旧版内容，`git diff --cached` 把刚进仓的文件全列成「已暂存的改动」，下一个人提交就把它们反向提交回去。

用法：
  shared-index-stale-check.py [--repo <目录>]    # 默认当前目录；检出的只是 HEAD 这一次提交，不改任何索引、工作区与引用
  shared-index-stale-check.py --selftest         # SHARED_INDEX_STALE_CHECK_BREAK=<项> 时必须判红

判法：
  取 HEAD 相对它的第一个父提交（根提交取空树）改到的文件，对其中每个出现在共用索引相对 HEAD 的 `git diff --cached --name-only` 里的文件，
  比共用索引里那一份的（模式，对象）与父提交里的：相同就是「停在旧 HEAD 上」，判红；不同的是别的会话正常暂存的改动，不报。
  父提交里没有而共用索引里也没有（HEAD 新增的文件），或 HEAD 删掉而共用索引里还是父提交那份，同样算相同。冲突中的条目（索引里有 0 以外的阶段）不报。
  共用索引取 git 目录里的 index：环境里的 GIT_INDEX_FILE 一律摘掉再问 git（它指的是私有索引，不是要查的这一份）。
  git 只带 GIT_OPTIONAL_LOCKS=0 地读：不刷新、不写共用索引。

退出码：0 没有停在旧 HEAD 上的文件；1 有，逐个列出文件与出路；77 本次未查（不在 git 仓里、没有 HEAD、HEAD 没改任何文件）。

弄坏开关 SHARED_INDEX_STALE_CHECK_BREAK=<项>（逗号连几个；只在 --selftest 里认，平常跑设了也不认、打一句提醒）。
逐项写开关名、弄坏哪一处、打开后自证哪几格红（格名是「✗ 自证 <格名>：」里的那一段）：
  no-content-compare   不比内容：HEAD 改到而又在 `git diff --cached` 里的文件一律报            others-staged-kept、stale-others-mixed
  trust-env-index      不摘环境里的 GIT_INDEX_FILE，查的是私有索引                           env-index-set
  skip-removed         HEAD 删掉的文件不报                                                  stale-private-index、stale-others-mixed、env-index-set
  skip-added           HEAD 新增的文件不报                                                  stale-private-index、stale-others-mixed、env-index-set、root-commit
  always-green         什么都不报                                                          stale-private-index、stale-others-mixed、env-index-set、root-commit、remedy-text
"""
import concurrent.futures
import os
import shutil
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

BREAK_VARIABLE = "SHARED_INDEX_STALE_CHECK_BREAK"
BREAKS = set()   # 只有 --selftest-cells 往里放；平常跑不认弄坏开关
ABSENT = ("000000", "0")
NOT_RUN = 77
LISTED_OTHERS_LIMIT = 30
KIND_TEXT = {
    "A": "HEAD 新增的文件，共用索引里还没有（列成已删除，下一个提交会把它删掉）",
    "D": "HEAD 删掉的文件，共用索引里还留着旧版（列成已新增，下一个提交会把它加回来）",
    "M": "HEAD 改过的文件，共用索引里还是 HEAD^ 的内容（列成已暂存的改动，下一个提交会把它改回旧版）",
}


def git(repository, arguments, environment, data=None):
    """→ (退出码, 标准输出字节)。只读地问 git：环境里的 GIT_INDEX_FILE 摘掉（trust-env-index 下留着），GIT_OPTIONAL_LOCKS=0。"""
    child_environment = {name: value for name, value in environment.items() if name != "GIT_INDEX_FILE" or "trust-env-index" in BREAKS}
    child_environment["GIT_OPTIONAL_LOCKS"] = "0"
    child_environment["LC_ALL"] = "C"
    completed = subprocess.run(["git", "-C", repository] + arguments, input=data, capture_output=True, env=child_environment, check=False)
    return completed.returncode, completed.stdout


def zero_separated(data):
    return [piece for piece in data.split(b"\0") if piece]


def inspect_repository(repository, environment=None):
    """→ 字典：status 是 "checked" 或 "not-run"；not-run 带 reason；checked 带 head、touched（HEAD 改到的文件数）、stale（[(路径, 种类)]）、others（别的会话暂存的路径）。"""
    environment = os.environ if environment is None else environment
    code, output = git(repository, ["rev-parse", "--verify", "-q", "HEAD^{commit}"], environment)
    if code != 0:
        return {"status": "not-run", "reason": "没有 HEAD（不在 git 仓里，或还没有提交）"}
    head = output.decode().strip()
    code, output = git(repository, ["rev-parse", "--verify", "-q", "HEAD^1^{commit}"], environment)
    if code == 0:
        parent = output.decode().strip()
    else:
        code, output = git(repository, ["hash-object", "-t", "tree", "--stdin"], environment, data=b"")
        if code != 0:
            return {"status": "not-run", "reason": "取不到空树"}
        parent = output.decode().strip()
    code, raw = git(repository, ["diff-tree", "-r", "-z", "--raw", "--no-renames", "--no-abbrev", parent, "HEAD"], environment)
    if code != 0:
        return {"status": "not-run", "reason": "git diff-tree 读不出 HEAD 改到的文件"}
    pieces = zero_separated(raw)
    touched = {}
    for index in range(0, len(pieces) - 1, 2):
        fields = pieces[index].decode().lstrip(":").split(" ")
        touched[pieces[index + 1].decode("utf-8", "surrogateescape")] = (fields[0], fields[2], fields[4][0])
    if not touched:
        return {"status": "not-run", "reason": "HEAD 没改任何文件"}
    code, cached_raw = git(repository, ["diff", "--cached", "--name-only", "-z", "--no-renames"], environment)
    if code != 0:
        return {"status": "not-run", "reason": "git diff --cached 读不出共用索引相对 HEAD 的差"}
    cached = {piece.decode("utf-8", "surrogateescape") for piece in zero_separated(cached_raw)}
    code, entries_raw = git(repository, ["ls-files", "-s", "-z"], environment)
    if code != 0:
        return {"status": "not-run", "reason": "git ls-files 读不出共用索引"}
    entries = {}
    for piece in zero_separated(entries_raw):
        meta, path = piece.decode("utf-8", "surrogateescape").split("\t", 1)
        mode, digest, stage = meta.split(" ")
        entries.setdefault(path, []).append((mode, digest, stage))
    stale = []
    for path in sorted(touched):
        old_mode, old_digest, status = touched[path]
        if "always-green" in BREAKS or path not in cached:
            continue
        if status == "D" and "skip-removed" in BREAKS:
            continue
        if status == "A" and "skip-added" in BREAKS:
            continue
        in_index = entries.get(path)
        if in_index and any(stage != "0" for _, _, stage in in_index):
            continue   # 冲突中的条目
        index_state = (in_index[0][0], in_index[0][1]) if in_index else ABSENT
        parent_state = ABSENT if status == "A" else (old_mode, old_digest)
        if "no-content-compare" in BREAKS or index_state == parent_state:
            stale.append((path, status if status in KIND_TEXT else "M"))
    stale_paths = {path for path, _ in stale}
    return {"status": "checked", "head": head, "touched": len(touched), "stale": stale, "others": sorted(cached - stale_paths)}


def render(result):
    """→ (退出码, 输出行)。"""
    if result["status"] == "not-run":
        return NOT_RUN, [f"  · 本次未查（不是判过了）：{result['reason']}"]
    short = result["head"][:12]
    if not result["stale"]:
        return 0, [f"  ✓ 共用索引没有停在旧 HEAD 上（HEAD {short} 改到 {result['touched']} 个文件，没有一个在共用索引里还是 HEAD^ 的内容）"]
    lines = [f"  ✗ 共用索引停在旧 HEAD 上：HEAD {short} 改到 {result['touched']} 个文件，其中 {len(result['stale'])} 个在共用索引里还是 HEAD^ 的内容；"
             "`git diff --cached` 把它们列成已暂存的改动，下一个人提交就把它们反向提交回去"]
    for path, kind in result["stale"]:
        lines.append(f"      {path}（{KIND_TEXT[kind]}）")   # gate-lint:detail
    others = result["others"]
    lines.append("  → 怎么办：`git read-tree HEAD` 之前先把共用索引里别的会话暂存的文件记下（`git diff --cached --name-only` 存一份），"
                 "重置后逐个 `git add` 回去；拿不准就先问。")
    if others:
        shown = "、".join(others[:LISTED_OTHERS_LIMIT])
        lines.append(f"    共用索引里不在上面名单里的已暂存文件共 {len(others)} 个（重置前要记下的就是它们的名字）：{shown}"
                     + (f" 等" if len(others) > LISTED_OTHERS_LIMIT else ""))
    else:
        lines.append("    共用索引里不在上面名单里的已暂存文件：没有，重置不会丢别的会话的暂存。")
    return 1, lines


# ── 自证 ──
FIXTURE_ENVIRONMENT_DROPPED = ("GIT_INDEX_FILE", "GIT_DIR", "GIT_WORK_TREE", "GIT_OPTIONAL_LOCKS")


def fixture_environment(home):
    environment = {name: value for name, value in os.environ.items() if name not in FIXTURE_ENVIRONMENT_DROPPED and name != BREAK_VARIABLE}
    environment.update({"HOME": home, "GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_NOSYSTEM": "1",
                        "GIT_AUTHOR_NAME": "selftest", "GIT_AUTHOR_EMAIL": "selftest@example.invalid",
                        "GIT_COMMITTER_NAME": "selftest", "GIT_COMMITTER_EMAIL": "selftest@example.invalid"})
    return environment


class Fixture:
    """临时目录里的一个仓，连同造它的 git 环境。给钩子的自证用：它按路径导入这个文件，从这里造同样的几种仓。"""

    def __init__(self, root, name):
        self.root = tempfile.mkdtemp(prefix=f"{name}-", dir=root)   # 同一种局面造几次也各用各的目录
        self.repository = os.path.join(self.root, "repository")
        self.private_index = os.path.join(self.root, "private.index")
        self.environment = fixture_environment(os.path.join(self.root, "home"))
        os.makedirs(self.repository)
        os.makedirs(self.environment["HOME"])
        self.run("init", "-q", "-b", "main")

    def run(self, *arguments, private=False, data=None):
        environment = dict(self.environment)
        if private:
            environment["GIT_INDEX_FILE"] = self.private_index
        completed = subprocess.run(["git", "-C", self.repository, "-c", "commit.gpgsign=false"] + list(arguments), input=data,
                                   capture_output=True, env=environment, check=False)
        if completed.returncode != 0:
            raise RuntimeError(f"git {' '.join(arguments)} 退 {completed.returncode}：{completed.stderr.decode(errors='replace')[:300]}")
        return completed.stdout.decode()

    def write(self, name, text):
        with open(os.path.join(self.repository, name), "w", encoding="utf-8") as handle:
            handle.write(text)

    def stage_content_in_shared_index(self, name, text):
        """别的会话暂存：只往共用索引放一份内容，工作区不动。"""
        digest = self.run("hash-object", "-w", "--stdin", data=text.encode()).strip()
        self.run("update-index", "--add", "--cacheinfo", f"100644,{digest},{name}")

    def baseline(self):
        for name in ("M.txt", "F.txt", "D.txt", "K.txt"):
            self.write(name, f"{name} 基线\n")
        self.run("add", "-A")
        self.run("commit", "-q", "-m", "基线")

    def private_commit(self, root=False):
        """HEAD 相对父提交：改 M.txt 与 F.txt，新增 H.txt，删 D.txt；只用私有索引暂存与提交，共用索引不动。根提交时 H.txt、M.txt、F.txt 都是新增。"""
        if not root:
            self.run("read-tree", "HEAD", private=True)
        self.write("M.txt", "M.txt 私有索引提交的版本\n")
        self.write("F.txt", "F.txt 私有索引提交的版本\n")
        self.write("H.txt", "H.txt 新文件\n")
        paths = ["M.txt", "F.txt", "H.txt"]
        if not root:
            os.remove(os.path.join(self.repository, "D.txt"))
            self.run("rm", "--cached", "-q", "D.txt", private=True)
        self.run("add", *paths, private=True)
        self.run("commit", "-q", "-m", "私有索引提交", private=True)


def make_scenario(root, kind):
    """kind：stale（私有索引提交，共用索引不动）、stale-others（共用索引里先有别的会话暂存的 F.txt 与 G.txt，再私有索引提交）、
    normal（经共用索引的正常提交）、recovered（stale-others 之后按出路重置并把别的会话的暂存加回去）、root（空仓里私有索引做根提交）、
    no-head（只 init）、empty-commit（--allow-empty 的提交）。→ Fixture"""
    fixture = Fixture(root, kind)
    if kind == "no-head":
        return fixture
    if kind == "root":
        fixture.private_commit(root=True)
        return fixture
    fixture.baseline()
    if kind == "empty-commit":
        fixture.run("commit", "-q", "--allow-empty", "-m", "空提交")
    elif kind == "normal":
        fixture.write("M.txt", "M.txt 正常提交的版本\n")
        fixture.run("add", "M.txt")
        fixture.run("commit", "-q", "-m", "正常提交")
    elif kind == "stale":
        fixture.private_commit()
    elif kind in ("stale-others", "recovered"):
        fixture.stage_content_in_shared_index("F.txt", "F.txt 别的会话暂存的版本\n")
        fixture.stage_content_in_shared_index("G.txt", "G.txt 别的会话暂存的新文件\n")
        fixture.private_commit()
        if kind == "recovered":
            fixture.run("read-tree", "HEAD")
            fixture.stage_content_in_shared_index("F.txt", "F.txt 别的会话暂存的版本\n")
            fixture.stage_content_in_shared_index("G.txt", "G.txt 别的会话暂存的新文件\n")
    else:
        raise ValueError(kind)
    return fixture


def selftest_cells(root):
    failures, checked = [], 0

    def check(cell, condition, detail):
        nonlocal checked
        checked += 1
        if not condition:
            failures.append(cell)
            print(f"  ✗ 自证 {cell}：{detail}")   # gate-lint:detail

    def run_check(kind, environment_extra=None):
        fixture = make_scenario(root, kind)
        environment = dict(fixture.environment, **(environment_extra(fixture) if environment_extra else {}))
        result = inspect_repository(fixture.repository, environment)
        code, lines = render(result)
        return code, lines, sorted(path for path, _ in result.get("stale", [])), result

    wanted_stale = ["D.txt", "F.txt", "H.txt", "M.txt"]
    code, lines, stale, _ = run_check("stale")
    check("stale-private-index", code == 1 and stale == wanted_stale,
          f"私有索引提交而不重置共用索引：应当退 1、报 {wanted_stale}（改、新增、删除三种都在）；实际退 {code}，报 {stale}")
    code, lines, stale, result = run_check("stale-others")
    check("stale-others-mixed", code == 1 and stale == ["D.txt", "H.txt", "M.txt"] and "F.txt" in result.get("others", []) and "G.txt" in result.get("others", []),
          f"共用索引里有别人暂存的 F.txt（内容与 HEAD^ 不同）与 G.txt：应当退 1、只报 D.txt、H.txt、M.txt，F.txt 与 G.txt 记在别的会话的暂存里；实际退 {code}，报 {stale}，别的会话的暂存 {result.get('others')}")
    code, lines, stale, _ = run_check("normal")
    check("normal-commit", code == 0 and not stale, f"经共用索引的正常提交：应当退 0、不报；实际退 {code}，报 {stale}")
    code, lines, stale, _ = run_check("recovered")
    check("others-staged-kept", code == 0 and not stale,
          f"按出路重置并把别的会话的 F.txt、G.txt 加回去之后：应当退 0（F.txt 在 HEAD 改到的文件里、也在 git diff --cached 里，但内容与 HEAD^ 不同，不报）；实际退 {code}，报 {stale}")
    code, lines, stale, _ = run_check("stale", environment_extra=lambda fixture: {"GIT_INDEX_FILE": fixture.private_index})
    check("env-index-set", code == 1 and stale == wanted_stale,
          f"环境里还留着指向私有索引的 GIT_INDEX_FILE：应当照样查共用索引、退 1、报 {wanted_stale}；实际退 {code}，报 {stale}")
    code, lines, stale, _ = run_check("root")
    check("root-commit", code == 1 and stale == ["F.txt", "H.txt", "M.txt"],
          f"空仓里私有索引做根提交：应当退 1、报 F.txt、H.txt、M.txt；实际退 {code}，报 {stale}")
    code, lines, stale, _ = run_check("no-head")
    check("no-head", code == NOT_RUN, f"还没有提交的仓：应当退 {NOT_RUN}（本次未查）；实际退 {code}")
    code, lines, stale, _ = run_check("empty-commit")
    check("empty-commit", code == NOT_RUN, f"--allow-empty 的提交没改任何文件：应当退 {NOT_RUN}；实际退 {code}")
    code, lines, stale, _ = run_check("stale")
    shown = "\n".join(lines)
    check("remedy-text", "git read-tree HEAD" in shown and "逐个 `git add` 回去" in shown and "拿不准就先问" in shown and "→ 怎么办" in shown,
          f"红的输出要带出路（read-tree 之前记下别的会话的暂存、逐个 add 回去、拿不准先问）；实际输出：{shown[-300:]}")
    return failures, checked


SWITCH_EXPECTED_CELLS = {
    "no-content-compare": ["others-staged-kept", "stale-others-mixed"],
    "trust-env-index": ["env-index-set"],
    "skip-removed": ["stale-private-index", "stale-others-mixed", "env-index-set"],
    "skip-added": ["stale-private-index", "stale-others-mixed", "env-index-set", "root-commit"],
    "always-green": ["stale-private-index", "stale-others-mixed", "env-index-set", "root-commit"],
}


def cells_in_scratch():
    root = tempfile.mkdtemp(prefix="shared-index-stale-check-selftest-")
    try:
        return selftest_cells(root)
    finally:
        shutil.rmtree(root, ignore_errors=True)


def selftest():
    failures, checked = cells_in_scratch()
    if failures:
        print(f"  ✗ shared-index-stale-check 自证 {len(failures)} 格不对（查了 {checked} 格）：{'、'.join(failures)}")
        print(f"  → 怎么办：照上面逐格看 inspect_repository() 与 render()；{BREAK_VARIABLE} 设着的话本来就该红")
        return 1
    clean = {name: value for name, value in os.environ.items() if name != BREAK_VARIABLE}

    def one(switch):
        completed = subprocess.run([sys.executable, "-B", os.path.realpath(__file__), "--selftest-cells"], capture_output=True, text=True,
                                   env=dict(clean, **{BREAK_VARIABLE: switch}), timeout=600, stdin=subprocess.DEVNULL, check=False)
        return switch, {line.split("：", 1)[0].removeprefix("  ✗ 自证 ") for line in completed.stdout.splitlines() if line.startswith("  ✗ 自证 ")}

    with concurrent.futures.ThreadPoolExecutor(max_workers=len(SWITCH_EXPECTED_CELLS)) as pool:
        results = list(pool.map(one, SWITCH_EXPECTED_CELLS))
    switch_failures = [] if len(results) == len(SWITCH_EXPECTED_CELLS) else [f"开关轮派出 {len(SWITCH_EXPECTED_CELLS)} 个、收回 {len(results)} 个"]
    for switch, red in results:
        print(f"  · 弄坏开关 {switch}：红在 {'、'.join(sorted(red)) or '一格都没有'}")
        missing = [cell for cell in SWITCH_EXPECTED_CELLS[switch] if cell not in red]
        if missing:
            switch_failures.append(f"{switch} 没红在 {'、'.join(missing)}（红的是 {'、'.join(sorted(red)) or '一格都没有'}）")
    for failure in switch_failures:
        print(f"  ✗ 弄坏开关 {failure}")   # gate-lint:detail
    if switch_failures:
        print("  → 怎么办：照上面那几个开关看它在代码里弄坏的那一处还在不在判法上；改了判法就改 SWITCH_EXPECTED_CELLS 与文件头的表")
        return 1
    print(f"  ✓ shared-index-stale-check 自证通过（查了 {checked} 格：私有索引提交而不重置判红、正常提交与按出路重置之后判绿、别的会话暂存的内容不同的文件不报、"
          f"环境里的私有索引不被当成共用索引、根提交判红、没有 HEAD 与空提交本次未查；弄坏开关 {len(results)} 个逐个打开都红在点名的格）")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    if arguments == ["--selftest-cells"]:
        BREAKS.update(item for item in os.environ.get(BREAK_VARIABLE, "").split(",") if item)
        failures, _ = cells_in_scratch()
        return 1 if failures else 0
    repository = os.getcwd()
    if arguments[:1] == ["--repo"] and len(arguments) == 2:
        repository = arguments[1]
    elif arguments:
        print("  ✗ 用法不对：只认 --repo <目录>、--selftest")
        print("  → 怎么办：不带参数（查当前目录的仓）、带 --repo <目录>，或带 --selftest")
        return 2
    if os.environ.get(BREAK_VARIABLE):
        print(f"  ! {BREAK_VARIABLE} 只在 --selftest 里认，这一次不认", file=sys.stderr)
    code, lines = render(inspect_repository(repository))
    print("\n".join(lines))
    return code


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
