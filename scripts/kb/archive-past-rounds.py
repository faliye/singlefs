#!/usr/bin/env python3
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: command git
"""上一轮及更早的实验记录归档进版本库：删掉工作区里的那批，保留本轮的。

规则见 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」之下那三段：
冻结只在本轮内有效；这一轮提交之后，下一次提交把上一次留下的那批删掉；本次的照旧留着，本次的验证照旧做得了；
要查删掉的那些，去版本库的历史里找。

用法：
  archive-past-rounds.py --check          还留着上一轮的记录就判红（门禁用）
  archive-past-rounds.py --apply          删掉；同时把指向被删文件的引用改成不带路径的说法
  archive-past-rounds.py --apply --keep-registered   照删别的，被登记表点名的那几份留下、退 0（「把那份留下」这条出路）
  archive-past-rounds.py --selftest

退出码：0 没有该删的、或删完了；1 --check 判红、取不到基准、删完有 include 指空；2 --apply 没给 GATE_BASE；
        3 --apply 时有被登记表点名的上一轮产物：别的照删，点名的那几份没删，逐份列出哪张表哪一行点名。

弄坏开关（只给 --selftest 证红用）：
  ARCHIVE_PAST_ROUNDS_BREAK=registry-check-skipped   named_by_registries() 不查登记表，一份都不报
      「✗ 自检失败：门禁登记表点名的上一轮产物」那一格红
  ARCHIVE_PAST_ROUNDS_BREAK=gate-directory-default-only   registry_trees() 不读 .claude/gate-directory，门禁目录一律取默认
      「✗ 自检失败：登记了别的门禁目录」那一格红
  ARCHIVE_PAST_ROUNDS_BREAK=gitlinks-kept   past_round_files() 不滤模式位 160000，gitlink 当文件进候选
      「✗ 自检失败：gitlink 进了候选」那一格红（--apply 对它 os.remove 抛 IsADirectoryError 的那条也跟着红）

判据（机械，不靠人记）：
  删 —— `research/results/` 与 `research/prompts/` 下，**在 HEAD 里有、且这一轮没碰过**的文件；
        「这一轮」的起点与门禁 doc-process-records 的 knowledge-sync 格同一个基准：`scripts/gate/changed-paths.sh` 的 `gate_diff_base gate`
        （`GATE_BASE`，否则 `@{upstream}` 的 merge-base，都没有就 HEAD），见 base_of 的说明。
        没改过 = 它是上一次提交固化下来的，属于上一轮及更早。
  留 —— 工作区里新加或改过的（本轮在产生的）、三方判决 `*-main-verification.md`（kb 的依据指着它）、
        `abandoned-rounds.tsv`（登记表，门禁 doc-process-records 的 abandoned-rounds 格的输入）、还被代码当输入的产物（still_an_input）、
        被检查读的登记表非注释行点名的产物（named_by_registries：门禁目录下的 .tsv，门禁目录照 .claude/gate-directory 登记、没登记取 .claude/gate.d；
        scripts/ 与 crates/ 下的 .tsv，钉表、变异表 scripts/mutation/tables/、crates/mutations.tsv 都在里面）。
        登记表点名的那几份 --apply 不删、退 3，逐份列出哪张表哪一行；出路是先改登记，或者把那份留下（--keep-registered）。
  ⚠️ 删文件会让别处指向它的链接指空，所以 --apply 同时把那些引用**只留文件名、去掉路径**，
     不留一个指空的路径——共享 `gate.sh` 的「链接指向」阶段（`.claude/singlefs-ai-sop/scripts/link-targets.py`）判的就是这个。「找不到就去 git 历史里看」写在 `.claude/agent-common.md`，不逐处重复。
"""
import os
import re
import subprocess
import sys
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight, shared_preflight_module  # noqa: E402

DIRS = ("research/results", "research/prompts")
KEEP = re.compile(r"-main-verification\.md$|abandoned-rounds\.tsv$")
LINK = re.compile(r"(?:research/results|research/prompts)/[^\s`)（）\]\n，。、；]+")
SCAN = (".claude/kb", "records", ".claude/rules", "briefs")
# 会把留存产物当**输入**的地方：门禁阶段按名字读它去比对、装置源码 include_str! 在编译期读它。
# 这些不是「记录」，删了不是「去 git 历史里查」，是**这道检查没得比 / 这个 crate 编不过**。
INPUT_TREES = (".claude/gate.d", ".claude/scripts", "scripts", "crates")
INPUT_SUFFIX = (".sh", ".py", ".rs", ".toml")
# 被检查读的登记表：门禁目录（照 .claude/gate-directory 登记，见 registry_trees）与这两处下的 .tsv——钉表、变异表（scripts/mutation/tables/、
# crates/mutations.tsv）、crate 里的登记表（verus-must-red-worlds.tsv 这类）。删之前查它们非注释行点名的路径
# （`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」：删之前查一遍被检查读的登记表点名的路径）。
REGISTRY_TREES_BESIDE_GATE = ("scripts", "crates")
REGISTRY_SUFFIX = ".tsv"
REGISTRY_SKIPPED_DIRECTORIES = ("fixtures", "target", ".git")
# 登记表一格里的产物路径：到空白、引号、括号、冒号（「文件:行号」）、井号、通配符为止；跟着通配符的是射程（e165-*-red-examples.tsv），不是点名
REGISTRY_PATH = re.compile(r"research/(?:results|prompts)/[^\s`'\"<>|()（）\[\]\\,，。、；：:#*?]+")
BROKEN = os.environ.get("ARCHIVE_PAST_ROUNDS_BREAK", "")


def git(*args, root="."):
    return subprocess.run(["git", "-c", "core.quotepath=false", *args], cwd=root,
                          capture_output=True, text=True, check=True).stdout


class BaseUnavailable(Exception):
    """取不到「这一轮」的起点：判不了哪些是上一轮的，不许当成「全是这一轮的」或「全是上一轮的」。"""


def base_of(root="."):
    """「这一轮」的起点。与门禁 doc-process-records 的 knowledge-sync 格（改了规则、agent、hook、门禁、脚本或实现之后有没有写阶段同步记录）
    取同一个基准——两道对同一批文件判相反的事，基准必须是同一个。取法不在这里另写一份，
    直接调那一格用的那一份：`scripts/gate/changed-paths.sh` 的 `gate_diff_base gate`
    （GATE_BASE，否则 @{upstream} 的 merge-base，都没有就 HEAD）。只认「GATE_BASE，否则 HEAD」时，
    没设 GATE_BASE 又有没推的提交，两道的窗口就不一样（C450 那一形）。

    ⚠️ **基准不一致会让两道直接打架**（C450 实测 2026-09-21，同一天撞了两次）：
    knowledge-sync 格（当时是单独的一道门禁）的改动范围取 `GATE_BASE`（门禁跑时给的是 `diff_base`，`refs/sop/gate-ok` 不存在时退回 `HEAD~1`），
    这一处原先写死 `HEAD`。差这一格的后果是**每次提交之后，上一轮的同步记录必然同时满足
    「这一道说该删」（它已进仓、相对 HEAD 没再改）与「knowledge-sync 格说要留」（它点名的触发文件相对 `HEAD~1` 还在范围里）**，
    不是偶发。实测那天两份 sync 记录点名的触发文件相对 `HEAD~1` 分别还有 14 个和 7 个在范围里。
    取同一个基准之后，上一轮的记录能正常退场。
    """
    library = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "gate", "changed-paths.sh")
    shown = subprocess.run(["bash", "-c", 'source "$1" && gate_diff_base gate', "base-of", library],
                           cwd=root, capture_output=True, text=True)
    base = shown.stdout.strip()
    if shown.returncode != 0 or not base:
        raise BaseUnavailable(f"{library} 的 gate_diff_base gate 退 {shown.returncode}：{shown.stderr.strip()[:200]}")
    return base


def past_round_files(root="."):
    # gitlink（模式位 160000，子模块）不是文件：--apply 对它 os.remove 抛 IsADirectoryError，后面的 stale_exclusions 与 missing_code_inputs 一条都跑不到（C437）。
    # 只有 ls-files --stage 带模式位；冲突时同一路径有几个阶段，去重。
    tracked = []
    for entry in git("ls-files", "--stage", *DIRS, root=root).splitlines():
        meta, _, path = entry.partition("\t")
        if not path or (meta.split()[0] == "160000" and BROKEN != "gitlinks-kept"):
            continue
        tracked.append(path)
    tracked = sorted(set(tracked))
    dirty = set(p for p in git("diff", "--name-only", base_of(root), "--", *DIRS, root=root).splitlines() if p)
    return sorted(p for p in tracked if p not in dirty and not KEEP.search(p))


def still_an_input(root, candidates):
    """候选里还被代码当输入的那些：门禁阶段按名字读它去比对、装置源码 include_str! 在编译期读它。
    返回 {相对路径: [点名它的源文件]}。这些**不删**——它们不是上一轮的记录，是这一轮还在跑的东西的输入。

    2026-09-21 实测两次，都是删完才发现：① 段序列登记表那一道门禁（段序列登记表与 E142 产物逐字比对，已搬进 doc-registries 的 segment-registry 那一格）连同它的自证一起塌；
    ② 归档删掉四份跨实验断言读的产物之后，research 整个编不过（e134 读 e133、e136 读 e134 与 e109、e137 读 e136），
    而 HEAD 上就是这个状态。第一次只把射程补到门禁脚本与 scripts，漏掉装置源码，于是第二次照样发生——
    所以这里按「哪些后缀是代码」扫，不按「我记得哪些目录会引」扫。"""
    by_name = {}
    for relative in candidates:
        by_name.setdefault(os.path.basename(relative), []).append(relative)
    found = {}
    for tree in INPUT_TREES:
        for directory, _, names in os.walk(os.path.join(root, tree)):
            if "fixtures" in directory.split(os.sep):
                continue                        # 样本目录里的路径是造给检查自证用的，不是真引用
            for name in names:
                if not name.endswith(INPUT_SUFFIX) or name == os.path.basename(__file__):
                    continue
                source = os.path.join(directory, name)
                if os.path.relpath(source, root) in candidates:
                    continue                    # 自己点名自己不算被谁当输入
                try:
                    text = open(source, encoding="utf-8", errors="ignore").read()
                except OSError:
                    continue
                # 只看非注释行：代码注释里写「依据见 research/prompts/xxx」是**记录引用**，
                # 归档要做的正是把它改成不带路径的说法；非注释行点名才是把它当数据读。
                # 不收这一刀，几乎每份产物都会被某处注释保下来，归档规则整个失效。
                text = "\n".join(line for line in text.splitlines()
                                 if not line.lstrip().startswith(("#", "//", "//!", "/*", "*")))
                for base, relatives in by_name.items():
                    if base in text:
                        for relative in relatives:
                            found.setdefault(relative, []).append(os.path.relpath(source, root))
    return found


def registry_trees(root):
    """要查的登记表在哪几棵树下：门禁目录 + scripts/ + crates/，相对仓根，嵌在别的树里的去掉。
    门禁目录从项目根 .claude/gate-directory 登记读（上游 preflight.py 的 project_gate_directory，与门禁、lib.sh 读同一份），没登记取默认 .claude/gate.d。"""
    if BROKEN == "gate-directory-default-only":
        gate_directory = ".claude/gate.d"
    else:
        gate_directory = shared_preflight_module(__file__).project_gate_directory(root)
    trees = []
    for tree in (gate_directory, *REGISTRY_TREES_BESIDE_GATE):
        tree = os.path.normpath(tree)
        if any(tree == kept or tree.startswith(kept + os.sep) for kept in trees):
            continue
        trees = [kept for kept in trees if not kept.startswith(tree + os.sep)] + [tree]
    return trees


def named_by_registries(root, candidates):
    """候选里被登记表非注释行点名的：{相对路径: ["<表>:<行号>", …]}。这些 --apply 不删。
    点名只认整条文件路径，或 research/ 下的子目录（带不带结尾的 / 都算，目录下的候选都算点到）；research/results/、research/prompts/ 本身与带通配符的是射程，不算点名。
    注释行（# 起头）里写的「依据见 research/…」是记录引用，不算——与 still_an_input 跳注释同一个理由。"""
    if BROKEN == "registry-check-skipped":
        return {}
    candidates = set(candidates)
    found = {}
    for tree in registry_trees(root):
        for directory, subdirectories, names in os.walk(os.path.join(root, tree)):
            subdirectories[:] = sorted(name for name in subdirectories if name not in REGISTRY_SKIPPED_DIRECTORIES)
            for name in sorted(names):
                if not name.endswith(REGISTRY_SUFFIX):
                    continue
                table = os.path.join(directory, name)
                try:
                    lines = open(table, encoding="utf-8", errors="ignore").read().splitlines()
                except OSError:
                    continue
                for number, line in enumerate(lines, 1):
                    if not line.strip() or line.lstrip().startswith("#"):
                        continue
                    for match in REGISTRY_PATH.finditer(line):
                        if line[match.end():match.end() + 1] in ("*", "?"):
                            continue                # 通配：射程，不是点名
                        target = match.group(0)
                        directory_prefix = target.rstrip("/") + "/"
                        hits = [path for path in candidates if path == target or path.startswith(directory_prefix)]
                        naming = "%s:%d" % (os.path.relpath(table, root), number)
                        for path in hits:
                            if naming not in found.setdefault(path, []):   # 同一行点名两次只列一次
                                found[path].append(naming)
    return found


def text_files(root="."):
    for base in SCAN:
        for directory, _, names in os.walk(os.path.join(root, base)):
            for name in names:
                if name.endswith((".md", ".tsv")):
                    yield os.path.relpath(os.path.join(directory, name), root)


def rewrite_links(root, doomed):
    """把指向被删文件的引用改成不带路径的说法，别留指空的路径。"""
    doomed = set(doomed)
    changed = 0
    touched = 0
    for relative in text_files(root):
        path = os.path.join(root, relative)
        try:
            text = open(path, encoding="utf-8").read()
        except (UnicodeDecodeError, OSError):
            continue
        hits = 0

        def fix(match):
            nonlocal hits
            target = match.group(0)
            if target not in doomed:
                return target
            hits += 1
            # 只留文件名，不留路径、也不逐处写「已归档」——那句话在 .claude/agent-common.md 里写一处就够
            return os.path.basename(target)

        new = LINK.sub(fix, text)
        if hits:
            # 反引号里的路径换掉之后那对反引号就多余了，留着不碍事；括注重复的收一下
            open(path, "w", encoding="utf-8").write(new)
            changed += hits
            touched += 1
    return changed, touched


def missing_code_inputs(root):
    """装置源码 include_str! 点名、而磁盘上不在的留存产物：那个 crate 编不过。
    不限于这一轮删的——上一轮删错了，这一轮照样红。

    2026-09-21 实测：上一次提交归档时删掉了四份跨实验断言用 include_str! 读的产物
    （e134 读 e133、e136 读 e134 与 e109、e137 读 e136），于是 HEAD 上 research 整个编不过，
    而当时的检查只在**删的那一刻**看、只扫门禁脚本，一个字都没说。

    ⚠️ 只认 include 这一族。别的形态（shell 把产物 cat 出来比、python 读它）**这里不判**：
    实测按「非注释行出现过这个名字」扫，49 处命中里只有 0 处是真的——样本目录 fixtures/ 造的假路径、
    模板占位串 `eNN-xxx-YYYY-MM-DD.out`、println! 里写的那句复跑说明，全会中。
    宁可射程窄而准：那些形态的脚本读不到文件时自己会失败，编译期输入不会，它只在 build 时塌。"""
    missing = []
    include = re.compile(r"include(?:_str|_bytes)?!\s*\(\s*\"([^\"]+)\"")
    for tree in INPUT_TREES:
        for directory, _, names in os.walk(os.path.join(root, tree)):
            if "fixtures" in directory.split(os.sep):
                continue
            for name in names:
                if not name.endswith(".rs"):
                    continue
                source = os.path.join(directory, name)
                try:
                    lines = open(source, encoding="utf-8", errors="ignore").read().splitlines()
                except OSError:
                    continue
                for number, line in enumerate(lines, 1):
                    if line.lstrip().startswith(("//", "/*", "*")):
                        continue
                    for target in include.findall(line):
                        if os.path.exists(os.path.normpath(os.path.join(directory, target))):
                            continue
                        missing.append("%s:%d  ← %s" % (os.path.relpath(source, root), number, target))
    return sorted(set(missing))


def stale_exclusions(root):
    """删完之后指不到东西的排除项：排除表指向不存在的路径时，那道检查自己会判红
    （`.claude/singlefs-ai-sop/rules/show-me-test.md`「排除的每一份都要登记」）。
    2026-09-21 实测：归档删掉 research/results/e153-s1-probe-tests/ 之后，naming-lint 整道红。"""
    import glob as _glob
    stale = []
    for table in sorted(_glob.glob(os.path.join(root, ".claude", "*-exclude"))):
        for number, line in enumerate(open(table, encoding="utf-8"), 1):
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            target = line.split("#")[0].strip()
            if target and not os.path.exists(os.path.join(root, target)):
                stale.append("%s:%d  %s" % (os.path.relpath(table, root), number, target))
    return stale


def run(root, apply_changes, keep_registered=False):
    # `--apply` 不给基准就拒绝跑，不是提醒、是拒绝。
    # 为什么非拒绝不可：门禁跑的时候 GATE_BASE 是设好的，人照着出路抄到自己的 shell 里就没了，
    # 于是它按 HEAD 算「这一轮」，把**这一次提交要带的**同步记录也当上一轮的删掉——删完门禁 doc-process-records 的 knowledge-sync 格
    # 当场红（触发文件没人点名），而删掉的文件没进过任何提交就找不回来。
    # 2026-09-22 实测：这么删掉两份，白跑一趟 20 分钟的全量门禁。
    # 文件头写着 base_of 取 GATE_BASE 拦不住这件事：写下来的提醒不会在动手那一刻拦人（sop-first.md）。
    # `--check` 不拦：它只读不写，而门禁就是不带 GATE_BASE 单跑它也要能报出现状。
    if apply_changes and not os.environ.get("GATE_BASE", ""):
        print("  ✗ --apply 不许在没给 GATE_BASE 的时候跑：不给基准它按 HEAD 算，会把这一次提交要带的同步记录也当上一轮的删掉")
        print("     → 怎么办：跑 GATE_BASE=<门禁那一行「diff 基准 <sha>」报的那个提交> python3 scripts/kb/archive-past-rounds.py --apply；")
        print("               基准要与门禁跑的时候一样（`gate.sh` 开头打的那一行「diff 基准 <sha>」就是它）。")
        print("               真要按 HEAD 算（只有手里一个提交都没暂存时才成立），显式写 GATE_BASE=HEAD。")
        return 2
    try:
        doomed = past_round_files(root)
    except BaseUnavailable as error:
        print(f"  ✗ 取不到「这一轮」的起点（{error}），分不出哪些实验记录是上一轮的")
        print("     → 怎么办：在仓根跑、确认 scripts/gate/changed-paths.sh 在；基准取法与门禁 doc-process-records 的 knowledge-sync 格是同一份，那一份坏了两边一起坏。")
        return 1
    inputs = still_an_input(root, doomed)
    doomed = [p for p in doomed if p not in inputs]
    registered = named_by_registries(root, doomed)
    doomed = [p for p in doomed if p not in registered]
    if inputs:
        print("      留下 %d 份还被代码当输入的产物（门禁按名字读它比对、装置 include_str! 编译期读它）：" % len(inputs))
        for relative in sorted(inputs):                              # gate-lint:detail
            print("        %s  ← %s" % (relative, "、".join(sorted(set(inputs[relative])))))
    if registered:
        print("      留下 %d 份被检查读的登记表点名的产物（删了那一行就指空，读它的检查当场红；查的树：%s）："
              % (len(registered), "、".join(registry_trees(root))))
        for relative in sorted(registered):                          # gate-lint:detail
            print("        %s  ← %s" % (relative, "、".join(registered[relative])))
    if not doomed:
        if apply_changes and refuse_registered(registered, keep_registered):
            return 3
        # 数磁盘上真有的，不数 git 索引：删过还没提交时索引里仍跟踪着那些文件，
        # 拿索引数报「现有多少份」会报出一个磁盘上不成立的数。
        kept = sum(len(names) for base in DIRS for _, _, names in os.walk(os.path.join(root, base)))
        print("  ✓ 没有上一轮留下的实验记录（%s 下磁盘上现有 %d 份，都是本轮的或按规则保留的；「这一轮」的起点 %s，取法同门禁 doc-process-records 的 knowledge-sync 格：changed-paths.sh 的 gate_diff_base gate）"
              % ("、".join(DIRS), kept, base_of(root)))
        print("     保留的四类：三方判决 *-main-verification.md（kb 的依据指着它）、abandoned-rounds.tsv（门禁 doc-process-records 的 abandoned-rounds 格的输入）、"
              "还被代码当输入的产物（这一次 %d 份）、被检查读的登记表点名的产物（这一次 %d 份），后两类逐个列在上面" % (len(inputs), len(registered)))
        return 0
    if not apply_changes:
        by_dir = {}
        for p in doomed:
            by_dir[p.split("/")[1]] = by_dir.get(p.split("/")[1], 0) + 1
        for name, count in sorted(by_dir.items(), key=lambda item: -item[1]):
            print("      research/%s/  %d 份" % (name, count))       # gate-lint:detail
        print("  ✗ 还留着上一轮及更早的实验记录 %d 份（本轮的与判决不算）" % len(doomed))
        # 出路里把基准一起打出来：门禁跑的时候 GATE_BASE 是设好的，人照着抄到自己的 shell 里就没了，
        # 于是它按 HEAD 算「这一轮」，把**这一次提交要带的**同步记录也当上一轮的删掉——删完门禁 doc-process-records 的 knowledge-sync 格
        # 当场红（触发文件没人点名），实测 2026-09-22 多删两份、白跑一轮全量门禁。
        given_base = os.environ.get("GATE_BASE", "")
        shown_base = given_base if given_base else "<门禁那一行「diff 基准 <sha>」报的那个提交>"
        print("     → 怎么办：跑 GATE_BASE=%s python3 scripts/kb/archive-past-rounds.py --apply 删掉，" % shown_base)
        print("               这个基准要与门禁跑的时候一样（`gate.sh` 开头打的那一行「diff 基准 <sha>」就是它）；")
        print("               不给基准它会按 HEAD 算，把这一次提交要带的同步记录也当上一轮的删掉，删完门禁 doc-process-records 的 knowledge-sync 格当场红；")
        print("               它同时把别处指向这些文件的引用改成只留文件名、不留路径；")
        print("               要查删掉的内容去 git 历史里找，有疑问就重新验证、不翻旧证据。")
        return 1
    changed, touched = rewrite_links(root, doomed)
    for relative in doomed:
        os.remove(os.path.join(root, relative))
    stale = stale_exclusions(root)
    losing = missing_code_inputs(root)
    print("  ✓ 删了 %d 份上一轮及更早的实验记录；%d 份文件里 %d 处引用去掉了路径、只留文件名" % (len(doomed), touched, changed))
    refused = refuse_registered(registered, keep_registered)
    if losing:
        print("  ✗ 代码非注释行点名了这些产物，而磁盘上没有——那道检查没得比、那个 crate 编不过：")
        for entry in losing:                                       # gate-lint:detail
            print("      %s" % entry)
        print("     → 怎么办：把它从 git 取回（git log --all --diff-filter=D --name-only 找删它的提交，")
        print("               再 git show <提交>^:<路径> 取出来）；确实不该再读它，就改那一行代码。")
        return 1
    if stale:
        print("  ! 删完之后这几条排除项指不到东西了，各自那道检查会判红：")
        for entry in stale:                                        # gate-lint:detail
            print("      %s" % entry)
        print("     → 怎么办：把它们从各自的排除表里删掉——排除只缩不涨，指不到的排除会让人以为那批文件已经被绕开了。")
    print("     → 下一步：在仓库根单跑共享「链接指向」那一道 `python3 .claude/singlefs-ai-sop/scripts/link-targets.py` 确认没有指空的链接，再跑一次本脚本 --check 判绿。")
    return 3 if refused else 0


def refuse_registered(registered, keep_registered):
    """--apply 时被登记表点名、因而没删的上一轮产物：没带 --keep-registered 就判拒（调用方退 3）。哪张表哪一行已经在「留下 … 登记表点名」那几行逐份列了。"""
    if not registered or keep_registered:
        return False
    print("  ✗ 有 %d 份上一轮的产物被检查读的登记表点名，没删（别的照删；哪张表哪一行见上面「登记表点名」那几行）" % len(registered))
    print("     → 怎么办：① 先改登记：那一行不再要这份产物的，删掉那一行或改指向（与归档同一次提交），再跑一次 --apply 把它删掉；")
    print("               ② 把那份留下：读它的检查还要它，跑 --apply --keep-registered（别的照删、这几份留着、退 0），")
    print("                  或者把它搬出 research/、放到读它的检查旁边并改登记指过去（.claude/rules/directory-layout.md「测试的定义与输入跟着测试放」）。")
    return True


def selftest():
    import tempfile
    # 自检自己控制 GATE_BASE，先清掉外面传进来的那一个。
    # 门禁跑这一条自证时 GATE_BASE 是 export 好的（门禁 code-tooling 在 gate.sh 底下跑），
    # 不清掉，「不给基准的 --apply 必须被拒绝」那一格在门禁里永远不触发、当场判红，
    # 而单独手敲它又是绿的——`command-safety.md`「握手用的环境变量漏给子进程」那一格。
    inherited_base = os.environ.pop("GATE_BASE", None)
    try:
        code = selftest_registry_check()
        if code != 0:
            return code
        code = selftest_in_temporary_directory()
        if code != 0:
            return code
        return selftest_gitlink()
    finally:
        if inherited_base is not None:
            os.environ["GATE_BASE"] = inherited_base


def selftest_registry_check():
    """删之前查被检查读的登记表：点名的不删、--apply 退 3 并列出表与行；没点名的照删；--keep-registered 留下退 0；
    --check 不把点名的算成该删；登记了别的门禁目录（.claude/gate-directory）时照那个目录找。"""
    import contextlib
    import io
    import tempfile
    failures, checked = [], 0
    with tempfile.TemporaryDirectory() as work:
        def write(relative, text):
            path = os.path.join(work, relative)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            open(path, "w", encoding="utf-8").write(text)

        def exists(name):
            return os.path.exists(os.path.join(work, "research/results", name))

        def commit(message):
            git("add", "-A", root=work)
            subprocess.run(["git", "commit", "-qm", message], cwd=work, check=True)

        def apply(keep_registered=False):
            buffer = io.StringIO()
            os.environ["GATE_BASE"] = "HEAD"
            try:
                with contextlib.redirect_stdout(buffer):
                    code = run(work, True, keep_registered)
            finally:
                os.environ.pop("GATE_BASE", None)
            return code, buffer.getvalue()

        git("init", "-q", root=work)
        subprocess.run(["git", "config", "user.email", "t@t"], cwd=work, check=True)
        subprocess.run(["git", "config", "user.name", "t"], cwd=work, check=True)
        for name in ("e5-named", "e6-unnamed", "e8-glob", "e11-mutation", "e12-in-directory/a"):
            write(f"research/results/{name}.out", "上一轮\n")
        write(".claude/gate.d/sample-ledgers.tsv",
              "# 注释行里提到不算点名：research/results/e6-unnamed.out\n"
              "research/results/e5-named.out\t门禁读的账\n"
              "research/results/e8-*.out\t带通配的是射程，不是点名\n")
        write("crates/mutations.tsv", "条目\tresearch/results/e11-mutation.out:3\n")
        write("scripts/mutation/tables/sample.tsv", "说明（research/results/e12-in-directory/）\t旧串\t新串\n")
        commit("上一轮")
        code, output = apply()
        checked += 1
        if code != 3 or not exists("e5-named.out") or ".claude/gate.d/sample-ledgers.tsv:2" not in output:
            failures.append(("门禁登记表点名的上一轮产物应当不删、--apply 退 3 并列出 .claude/gate.d/sample-ledgers.tsv:2，"
                             f"实际退 {code}、那份{'还在' if exists('e5-named.out') else '删了'}：{output[-600:]}",
                             "看 named_by_registries() 与 run() 里 doomed 减去 registered、refuse_registered() 那几步"))
        checked += 1
        if exists("e6-unnamed.out") or exists("e8-glob.out"):
            failures.append(("没点名的（只在注释行里提到的、通配射程里的）应当照删", "看 named_by_registries() 跳注释行与跳通配那两处"))
        checked += 1
        if (not exists("e11-mutation.out") or not exists("e12-in-directory/a.out")
                or "crates/mutations.tsv:1" not in output or "scripts/mutation/tables/sample.tsv:1" not in output):
            failures.append(("变异表里「文件:行号」写法点名的、按子目录点名的应当不删并列出表与行", f"看 REGISTRY_PATH 与目录前缀那一步：{output[-600:]}"))
        code_keep, output_keep = apply(keep_registered=True)
        checked += 1
        if code_keep != 0 or not exists("e5-named.out"):
            failures.append((f"--apply --keep-registered 应当留下点名的那几份、退 0，实际退 {code_keep}：{output_keep[-300:]}",
                             "看 refuse_registered() 的 keep_registered 分支"))
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_check = run(work, False)
        checked += 1
        if code_check != 0:
            failures.append((f"--check 不该把登记表点名、按规则留下的算成该删的，实际退 {code_check}：{buffer.getvalue()[-300:]}",
                             "看 run() 里 doomed 减去 registered 是不是在 --check 那一支之前"))
        for name in ("e13-old-gate-directory", "e14-registered-gate-directory"):
            write(f"research/results/{name}.out", "上一轮\n")
        write(".claude/gate-directory", "ops/gates  # 样本：门禁目录搬了家\n")
        write("ops/gates/moved.tsv", "research/results/e14-registered-gate-directory.out\t搬家后的门禁登记表\n")
        write(".claude/gate.d/stale.tsv", "research/results/e13-old-gate-directory.out\t不再是门禁目录的旧位置\n")
        commit("门禁目录搬家")
        code_moved, output_moved = apply(keep_registered=True)
        checked += 1
        if not exists("e14-registered-gate-directory.out") or exists("e13-old-gate-directory.out") or "ops/gates/moved.tsv:1" not in output_moved:
            failures.append(("登记了别的门禁目录（.claude/gate-directory 写 ops/gates）时应当照那个目录找登记表、旧的默认目录不再算，"
                             f"实际退 {code_moved}：{output_moved[-400:]}",
                             "看 registry_trees() 是不是经 project_gate_directory 读的登记"))
    for failure, howto in failures:
        print(f"  ✗ 自检失败：{failure}")                 # gate-lint:detail
        print(f"     → 怎么办：{howto}；ARCHIVE_PAST_ROUNDS_BREAK 设着的话这里本来就该红")
    if failures:
        return 1
    print(f"  ✓ 自检（登记表，查了 {checked} 格）：门禁登记表、变异表点名的不删且 --apply 退 3 列出表与行、注释行与通配不算点名照删、"
          "--keep-registered 留下退 0、--check 不算它们、登记了别的门禁目录时照那个目录找")
    return 0


def selftest_in_temporary_directory():
    import tempfile
    with tempfile.TemporaryDirectory() as work:
        os.makedirs(os.path.join(work, "research/results"))
        os.makedirs(os.path.join(work, "research/prompts"))
        os.makedirs(os.path.join(work, ".claude/kb"))
        git("init", "-q", root=work)
        subprocess.run(["git", "config", "user.email", "t@t"], cwd=work, check=True)
        subprocess.run(["git", "config", "user.name", "t"], cwd=work, check=True)
        open(os.path.join(work, "research/results/e1-old.out"), "w").write("old\n")
        open(os.path.join(work, "research/prompts/r1-main-verification.md"), "w").write("判决\n")
        # 上一轮的产物，但装置源码在编译期读它：它不是记录，删了那个 crate 编不过
        open(os.path.join(work, "research/results/e3-input.out"), "w").write("input\n")
        os.makedirs(os.path.join(work, "scripts/experiments"))
        open(os.path.join(work, "scripts/experiments/probe.rs"), "w").write(
            '//! 依据见 research/results/e1-old.out\n'
            'const P: &str = include_str!("../../research/results/e3-input.out");\n')
        open(os.path.join(work, ".claude/kb/page.md"), "w").write(
            "产物 `research/results/e1-old.out`，判决 `research/prompts/r1-main-verification.md`，"
            "输入 `research/results/e3-input.out`\n")
        git("add", "-A", root=work)
        subprocess.run(["git", "commit", "-qm", "round one"], cwd=work, check=True)
        open(os.path.join(work, "research/results/e2-new.out"), "w").write("new\n")
        if run(work, False) != 1:
            print("  ✗ 自检失败：有上一轮记录时 --check 没判红")
            print("     → 怎么办：看 past_round_files() 的 ls-files 与 diff 组合。")
            return 1
        # 不给基准的 --apply 必须当场拒绝（退出码 2），而且一个文件都不许删
        if run(work, True) != 2 or not os.path.exists(os.path.join(work, "research/results/e1-old.out")):
            print("  ✗ 自检失败：没给 GATE_BASE 的 --apply 没被拒绝，或者它已经动手删了")
            print("     → 怎么办：看 run() 开头那道 GATE_BASE 闸，它要在 past_round_files() 之前 return 2。")
            return 1
        os.environ["GATE_BASE"] = "HEAD"
        try:
            run(work, True)
        finally:
            os.environ.pop("GATE_BASE", None)
        if os.path.exists(os.path.join(work, "research/results/e1-old.out")):
            print("  ✗ 自检失败：上一轮的产物没删掉")
            print("     → 怎么办：看 run() 的 os.remove 那一段。")
            return 1
        if not os.path.exists(os.path.join(work, "research/prompts/r1-main-verification.md")):
            print("  ✗ 自检失败：判决被删了，它该留")
            print("     → 怎么办：看 KEEP 那条正则。")
            return 1
        if not os.path.exists(os.path.join(work, "research/results/e2-new.out")):
            print("  ✗ 自检失败：本轮新产生的被删了")
            print("     → 怎么办：未跟踪的文件不在 ls-files 里，看 past_round_files() 为什么把它算进去了。")
            return 1
        if not os.path.exists(os.path.join(work, "research/results/e3-input.out")):
            print("  ✗ 自检失败：装置源码 include_str! 读的产物被当成上一轮的记录删了")
            print("     → 怎么办：看 still_an_input() 扫的树与后缀，它该在删之前把这一份挑走。")
            return 1
        page = open(os.path.join(work, ".claude/kb/page.md"), encoding="utf-8").read()
        if "research/results/e3-input.out" not in page:
            print("  ✗ 自检失败：没删的产物，指向它的路径却被改掉了")
            print("     → 怎么办：rewrite_links() 只改真删掉的那些，看 doomed 是不是没减去输入集。")
            return 1
        # 「代码点名了磁盘上没有的产物」这条自己也要证明会红：造一处非注释行的引用
        gate_dir = os.path.join(work, ".claude", "gate.d")
        os.makedirs(gate_dir, exist_ok=True)
        sample = os.path.join(work, "scripts/experiments/gone.rs")
        open(sample, "w").write('const G: &str = include_str!("../../research/results/e9-gone.out");\n')
        if not missing_code_inputs(work):
            print("  ✗ 自检失败：装置源码 include_str! 点名了磁盘上没有的产物，missing_code_inputs 却一条都没报")
            print("     → 怎么办：看 include 那条正则与它按 directory 解相对路径那一步。")
            return 1
        # 同一行挪进注释就该不报：注释里的引用是记录，归档改的正是它
        open(sample, "w").write('// const G: &str = include_str!("../../research/results/e9-gone.out");\n')
        if missing_code_inputs(work):
            print("  ✗ 自检失败：注释行里的 include 被当成了输入")
            print("     → 怎么办：看 missing_code_inputs 跳注释那一步。")
            return 1
        os.remove(sample)
        # 产物在的时候不许报：只证明会红不够，还要证明它分得出差别
        if missing_code_inputs(work):
            print("  ✗ 自检失败：include 指的产物都在，missing_code_inputs 还是报了")
            print("     → 怎么办：看它解出来的路径与磁盘上的实际位置差在哪。")
            return 1
        page = open(os.path.join(work, ".claude/kb/page.md"), encoding="utf-8").read()
        if "research/results/e1-old.out" in page:
            print("  ✗ 自检失败：指向被删文件的引用还留着指空的路径")
            print("     → 怎么办：看 rewrite_links() 的 LINK 正则与 doomed 集合。")
            return 1
        if "research/prompts/r1-main-verification.md" not in page:
            print("  ✗ 自检失败：指向判决的引用被误改了，判决没删就不该改它的链接")
            print("     → 怎么办：rewrite_links() 只改 doomed 里的路径，看它为什么碰了别的。")
            return 1
        if run(work, False) != 0:
            print("  ✗ 自检失败：删完 --check 仍判红")
            print("     → 怎么办：看 past_round_files() 删除之后还认出了什么。")
            return 1
        # ── 基准对齐：上一轮的记录已进仓，而它点名的触发文件相对 HEAD~1 还在改动范围里 ──
        # 这一格分的是「这一轮」取哪个起点。取 HEAD（旧行为）时那份记录被判成「上一轮的、该删」，
        # 而门禁 doc-process-records 的 knowledge-sync 格按 GATE_BASE（= HEAD~1）判它「还要着」，两边打架（C450 实测撞过两次）。
        # 取同一个 GATE_BASE 之后它落进「这一轮碰过的」，不再被判该删。
        open(os.path.join(work, "research/prompts/r1-sync.md"), "w").write("<!-- knowledge-sync -->\n同步记录\n")
        git("add", "-A", root=work)
        subprocess.run(["git", "commit", "-qm", "上一轮：写记录并提交"], cwd=work, check=True)
        one_commit_ago = git("rev-parse", "HEAD~1", root=work).strip()
        os.environ["GATE_BASE"] = one_commit_ago
        try:
            still_past = past_round_files(work)
        finally:
            os.environ.pop("GATE_BASE", None)
        if any(name.endswith("r1-sync.md") for name in still_past):
            print("  ✗ 自检失败：给了 GATE_BASE 之后，这一轮碰过的记录仍被判成上一轮的")
            print("     → 怎么办：看 base_of —— 它要取 GATE_BASE（与门禁 doc-process-records 的 knowledge-sync 格同一个基准），")
            print("               写死 HEAD 会让两道对同一份记录判相反的事。")
            return 1
        # ── 没设 GATE_BASE、有没推的提交：基准取 @{upstream} 的 merge-base（changed-paths.sh 的 gate 取法，与 knowledge-sync 格同一份）──
        # 只认「GATE_BASE，否则 HEAD」时，没推的那次提交里写的记录会被当成上一轮的删掉，而 knowledge-sync 格按 merge-base 判它还要着。
        pushed_tip = git("rev-parse", "HEAD", root=work).strip()
        subprocess.run(["git", "branch", "upstream-sample", pushed_tip], cwd=work, check=True)
        subprocess.run(["git", "branch", "-q", "--set-upstream-to=upstream-sample"], cwd=work, check=True)
        open(os.path.join(work, "research/prompts/r2-sync.md"), "w").write("<!-- knowledge-sync -->\n没推的那次提交里写的记录\n")
        git("add", "-A", root=work)
        subprocess.run(["git", "commit", "-qm", "没推的一次提交"], cwd=work, check=True)
        if any(name.endswith("r2-sync.md") for name in past_round_files(work)):
            print("  ✗ 自检失败：没设 GATE_BASE、有没推的提交时，那次提交里写的记录被判成上一轮的")
            print("     → 怎么办：看 base_of —— 它要调 scripts/gate/changed-paths.sh 的 gate_diff_base gate（与门禁 doc-process-records 的 knowledge-sync 格同一份），")
            print("               那一份没设 GATE_BASE 时取 @{upstream} 的 merge-base；只认「GATE_BASE，否则 HEAD」就会在这里判错。")
            return 1

    print("  ✓ 自检：不给 GATE_BASE 的 --apply 当场拒绝且一个文件没删、有旧记录判红、删完判绿、判决与本轮新产物不被删、装置 include_str! 读的产物不被删且链接不被改、"
          "指向被删文件的引用改成不带路径的说法、include 指空的产物报得出来、注释里的 include 与产物都在时不误报、"
          "没设 GATE_BASE 时基准取上游的 merge-base（与门禁 doc-process-records 的 knowledge-sync 格同一份取法）")
    return 0


def selftest_gitlink():
    """gitlink（子模块，模式位 160000）不是文件：不进候选、--apply 不对它 os.remove，stale_exclusions 与 missing_code_inputs 照常跑到（C437）。
    gitlink 用 git update-index --cacheinfo 160000 造，不用真子模块，工作区里放一个空目录当它的检出位置。
    两个场景：删完有指不到的排除项（打「排除项指不到东西」）、删完有 include 指空的产物（打「代码非注释行点名了这些产物」）。"""
    import contextlib
    import io
    import tempfile
    failures, checked = [], 0
    gitlink = "research/results/sub-module"
    for scene in ("排除项指不到", "include 指空"):
        with tempfile.TemporaryDirectory() as work:
            def write(relative, text):
                path = os.path.join(work, relative)
                os.makedirs(os.path.dirname(path), exist_ok=True)
                open(path, "w", encoding="utf-8").write(text)

            git("init", "-q", root=work)
            subprocess.run(["git", "config", "user.email", "t@t"], cwd=work, check=True)
            subprocess.run(["git", "config", "user.name", "t"], cwd=work, check=True)
            write("research/results/e1-old.out", "上一轮\n")
            if scene == "排除项指不到":
                write(".claude/sample-exclude", "research/results/e7-gone-directory/  # 样本：删完指不到\n")
            else:
                write("scripts/experiments/gone.rs", 'const G: &str = include_str!("../../research/results/e9-gone.out");\n')
            git("add", "-A", root=work)
            subprocess.run(["git", "commit", "-qm", "上一轮"], cwd=work, check=True)
            tip = git("rev-parse", "HEAD", root=work).strip()
            git("update-index", "--add", "--cacheinfo", f"160000,{tip},{gitlink}", root=work)
            subprocess.run(["git", "commit", "-qm", "gitlink"], cwd=work, check=True)
            os.makedirs(os.path.join(work, gitlink))        # 子模块检出位置：一个目录，os.remove 对它抛 IsADirectoryError
            os.environ["GATE_BASE"] = "HEAD"
            try:
                checked += 1
                if gitlink in past_round_files(work):
                    failures.append((f"gitlink 进了候选（{scene}）：{gitlink} 是子模块不是文件，--apply 对它 os.remove 会抛 IsADirectoryError",
                                     "看 past_round_files() 是不是读 ls-files --stage、把模式位 160000 的滤掉了"))
                checked += 1
                buffer = io.StringIO()
                try:
                    with contextlib.redirect_stdout(buffer):
                        code = run(work, True)
                except Exception as error:
                    failures.append((f"有 gitlink 时 --apply（{scene}）抛了 {type(error).__name__}：{error}，后面的 stale_exclusions 与 missing_code_inputs 没跑到",
                                     "看 past_round_files() 滤 gitlink 那一步，与 run() 里 os.remove 之前的候选"))
                    continue
                output = buffer.getvalue()
                checked += 1
                if os.path.exists(os.path.join(work, "research/results/e1-old.out")) or not os.path.isdir(os.path.join(work, gitlink)):
                    failures.append((f"有 gitlink 时 --apply（{scene}）该删 e1-old.out、不动 {gitlink}：{output[-300:]}", "看 past_round_files() 与 run() 的 os.remove 那一段"))
                checked += 1
                if scene == "排除项指不到" and (code != 0 or "排除项指不到东西" not in output or ".claude/sample-exclude:1" not in output):
                    failures.append((f"有 gitlink 时 --apply 该跑到 stale_exclusions 并列出 .claude/sample-exclude:1，实际退 {code}：{output[-400:]}",
                                     "看 run() 里 os.remove 之后 stale_exclusions() 那一句"))
                if scene == "include 指空" and (code != 1 or "代码非注释行点名了这些产物" not in output or "e9-gone.out" not in output):
                    failures.append((f"有 gitlink 时 --apply 该跑到 missing_code_inputs 并列出 e9-gone.out、退 1，实际退 {code}：{output[-400:]}",
                                     "看 run() 里 os.remove 之后 missing_code_inputs() 那一句"))
            finally:
                os.environ.pop("GATE_BASE", None)
    for failure, howto in failures:
        print(f"  ✗ 自检失败：{failure}")                 # gate-lint:detail
        print(f"     → 怎么办：{howto}；ARCHIVE_PAST_ROUNDS_BREAK=gitlinks-kept 设着的话这里本来就该红")
    if failures:
        return 1
    print(f"  ✓ 自检（gitlink，查了 {checked} 格）：research/results/ 下的 gitlink 不进候选、--apply 不抛异常也不动它、"
          "删完照常跑到 stale_exclusions 与 missing_code_inputs")
    return 0


def main(argv):
    if "--selftest" in argv:
        return selftest()
    positional = [a for a in argv if not a.startswith("--")]
    root = os.path.abspath(positional[0]) if positional else os.path.abspath(
        os.path.join(os.path.dirname(__file__), "..", ".."))
    return run(root, "--apply" in argv, "--keep-registered" in argv)


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
