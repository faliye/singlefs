#!/usr/bin/env python3
"""上一轮及更早的实验记录归档进版本库：删掉工作区里的那批，保留本轮的。

规则见 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」之下那三段：
冻结只在本轮内有效；这一轮提交之后，下一次提交把上一次留下的那批删掉；本次的照旧留着，本次的验证照旧做得了；
要查删掉的那些，去版本库的历史里找。

用法：
  archive-past-rounds.py --check          还留着上一轮的记录就判红（门禁用）
  archive-past-rounds.py --apply          删掉；同时把指向被删文件的引用改成不带路径的说法
  archive-past-rounds.py --selftest

判据（机械，不靠人记）：
  删 —— `research/results/` 与 `research/prompts/` 下，**在 HEAD 里有、且这一轮没碰过**的文件；
        「这一轮」的起点取 `GATE_BASE`（与门禁 68 号同一个基准，见 base_of 的说明），没给就取 HEAD。
        没改过 = 它是上一次提交固化下来的，属于上一轮及更早。
  留 —— 工作区里新加或改过的（本轮在产生的）、三方判决 `*-main-verification.md`（kb 的依据指着它）、
        `abandoned-rounds.tsv`（登记表，门禁 66 号的输入）。
  ⚠️ 删文件会让别处指向它的链接指空，所以 --apply 同时把那些引用**只留文件名、去掉路径**，
     不留一个指空的路径——共享 `gate.sh` 的「链接指向」阶段判的就是这个。「找不到就去 git 历史里看」写在 `.claude/agent-common.md`，不逐处重复。
"""
import os
import re
import subprocess
import sys

DIRS = ("research/results", "research/prompts")
KEEP = re.compile(r"-main-verification\.md$|abandoned-rounds\.tsv$")
LINK = re.compile(r"research/(?:results|prompts)/[^\s`)（）\]\n，。、；]+")
SCAN = (".claude/kb", "records", ".claude/rules", "briefs")
# 会把留存产物当**输入**的地方：门禁阶段按名字读它去比对、装置源码 include_str! 在编译期读它。
# 这些不是「记录」，删了不是「去 git 历史里查」，是**这道检查没得比 / 这个 crate 编不过**。
INPUT_TREES = (".claude/gate.d", ".claude/scripts", "research", "crates")
INPUT_SUFFIX = (".sh", ".py", ".rs", ".toml")


def git(*args, root="."):
    return subprocess.run(["git", "-c", "core.quotepath=false", *args], cwd=root,
                          capture_output=True, text=True, check=True).stdout


def base_of(root="."):
    """「这一轮」的起点。与门禁 68 号（改了规则、agent、hook、门禁、脚本或实现之后有没有写阶段同步记录）
    取同一个基准——两道对同一批文件判相反的事，基准必须是同一个。

    ⚠️ **基准不一致会让两道直接打架**（C450 实测 2026-09-21，同一天撞了两次）：
    68 号的改动范围取 `GATE_BASE`（门禁跑时给的是 `diff_base`，`refs/sop/gate-ok` 不存在时退回 `HEAD~1`），
    这一处原先写死 `HEAD`。差这一格的后果是**每次提交之后，上一轮的同步记录必然同时满足
    「这一道说该删」（它已进仓、相对 HEAD 没再改）与「68 号说要留」（它点名的触发文件相对 `HEAD~1` 还在范围里）**，
    不是偶发。实测那天两份 sync 记录点名的触发文件相对 `HEAD~1` 分别还有 14 个和 7 个在范围里。
    取同一个基准之后，上一轮的记录能正常退场。
    """
    base = os.environ.get("GATE_BASE", "")
    if base:
        r = subprocess.run(['git', '-C', root, 'rev-parse', '--verify', '-q', base + '^{commit}'],
                           capture_output=True, text=True)
        if r.returncode == 0:
            return base
    return "HEAD"


def past_round_files(root="."):
    tracked = [p for p in git("ls-files", *DIRS, root=root).splitlines() if p]
    dirty = set(p for p in git("diff", "--name-only", base_of(root), "--", *DIRS, root=root).splitlines() if p)
    return sorted(p for p in tracked if p not in dirty and not KEEP.search(p))


def still_an_input(root, candidates):
    """候选里还被代码当输入的那些：门禁阶段按名字读它去比对、装置源码 include_str! 在编译期读它。
    返回 {相对路径: [点名它的源文件]}。这些**不删**——它们不是上一轮的记录，是这一轮还在跑的东西的输入。

    2026-09-21 实测两次，都是删完才发现：① 门禁 52 号（段序列登记表与 E142 产物逐字比对）连同它的自证一起塌；
    ② 归档删掉四份跨实验断言读的产物之后，research 整个编不过（e134 读 e133、e136 读 e134 与 e109、e137 读 e136），
    而 HEAD 上就是这个状态。第一次只把射程补到门禁脚本与 research/scripts，漏掉装置源码，于是第二次照样发生——
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


def run(root, apply_changes):
    # `--apply` 不给基准就拒绝跑，不是提醒、是拒绝。
    # 为什么非拒绝不可：门禁跑的时候 GATE_BASE 是设好的，人照着出路抄到自己的 shell 里就没了，
    # 于是它按 HEAD 算「这一轮」，把**这一次提交要带的**同步记录也当上一轮的删掉——删完门禁 68 号
    # 当场红（触发文件没人点名），而删掉的文件没进过任何提交就找不回来。
    # 2026-09-22 实测：这么删掉两份，白跑一趟 20 分钟的全量门禁。
    # 文件头写着 base_of 取 GATE_BASE 拦不住这件事：写下来的提醒不会在动手那一刻拦人（sop-first.md）。
    # `--check` 不拦：它只读不写，而门禁就是不带 GATE_BASE 单跑它也要能报出现状。
    if apply_changes and not os.environ.get("GATE_BASE", ""):
        print("  ✗ --apply 不许在没给 GATE_BASE 的时候跑：不给基准它按 HEAD 算，会把这一次提交要带的同步记录也当上一轮的删掉")
        print("     → 怎么办：跑 GATE_BASE=<门禁那一行「diff 基准 <sha>」报的那个提交> python3 research/scripts/archive-past-rounds.py --apply；")
        print("               基准要与门禁跑的时候一样（`gate.sh` 开头打的那一行「diff 基准 <sha>」就是它）。")
        print("               真要按 HEAD 算（只有手里一个提交都没暂存时才成立），显式写 GATE_BASE=HEAD。")
        return 2
    doomed = past_round_files(root)
    inputs = still_an_input(root, doomed)
    doomed = [p for p in doomed if p not in inputs]
    if inputs:
        print("      留下 %d 份还被代码当输入的产物（门禁按名字读它比对、装置 include_str! 编译期读它）：" % len(inputs))
        for relative in sorted(inputs):                              # gate-lint:detail
            print("        %s  ← %s" % (relative, "、".join(sorted(set(inputs[relative])))))
    if not doomed:
        # 数磁盘上真有的，不数 git 索引：删过还没提交时索引里仍跟踪着那些文件，
        # 拿索引数报「现有多少份」会报出一个磁盘上不成立的数。
        kept = sum(len(names) for base in DIRS for _, _, names in os.walk(os.path.join(root, base)))
        print("  ✓ 没有上一轮留下的实验记录（%s 下磁盘上现有 %d 份，都是本轮的或按规则保留的）" % ("、".join(DIRS), kept))
        print("     保留的两类：三方判决 *-main-verification.md（kb 的依据指着它）、abandoned-rounds.tsv（门禁 66 号的输入）")
        return 0
    if not apply_changes:
        by_dir = {}
        for p in doomed:
            by_dir[p.split("/")[1]] = by_dir.get(p.split("/")[1], 0) + 1
        for name, count in sorted(by_dir.items(), key=lambda item: -item[1]):
            print("      research/%s/  %d 份" % (name, count))       # gate-lint:detail
        print("  ✗ 还留着上一轮及更早的实验记录 %d 份（本轮的与判决不算）" % len(doomed))
        # 出路里把基准一起打出来：门禁跑的时候 GATE_BASE 是设好的，人照着抄到自己的 shell 里就没了，
        # 于是它按 HEAD 算「这一轮」，把**这一次提交要带的**同步记录也当上一轮的删掉——删完门禁 68 号
        # 当场红（触发文件没人点名），实测 2026-09-22 多删两份、白跑一轮全量门禁。
        given_base = os.environ.get("GATE_BASE", "")
        shown_base = given_base if given_base else "<门禁那一行「diff 基准 <sha>」报的那个提交>"
        print("     → 怎么办：跑 GATE_BASE=%s python3 research/scripts/archive-past-rounds.py --apply 删掉，" % shown_base)
        print("               这个基准要与门禁跑的时候一样（`gate.sh` 开头打的那一行「diff 基准 <sha>」就是它）；")
        print("               不给基准它会按 HEAD 算，把这一次提交要带的同步记录也当上一轮的删掉，删完门禁 68 号当场红；")
        print("               它同时把别处指向这些文件的引用改成只留文件名、不留路径；")
        print("               要查删掉的内容去 git 历史里找，有疑问就重新验证、不翻旧证据。")
        return 1
    changed, touched = rewrite_links(root, doomed)
    for relative in doomed:
        os.remove(os.path.join(root, relative))
    stale = stale_exclusions(root)
    losing = missing_code_inputs(root)
    print("  ✓ 删了 %d 份上一轮及更早的实验记录；%d 份文件里 %d 处引用去掉了路径、只留文件名" % (len(doomed), touched, changed))
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
    return 0


def selftest():
    import tempfile
    # 自检自己控制 GATE_BASE，先清掉外面传进来的那一个。
    # 门禁跑这一条自证时 GATE_BASE 是 export 好的（门禁 47 号在 gate.sh 底下跑），
    # 不清掉，「不给基准的 --apply 必须被拒绝」那一格在门禁里永远不触发、当场判红，
    # 而单独手敲它又是绿的——`command-safety.md`「握手用的环境变量漏给子进程」那一格。
    inherited_base = os.environ.pop("GATE_BASE", None)
    try:
        return selftest_in_temporary_directory()
    finally:
        if inherited_base is not None:
            os.environ["GATE_BASE"] = inherited_base


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
        os.makedirs(os.path.join(work, "research/bench/src"))
        open(os.path.join(work, "research/bench/src/probe.rs"), "w").write(
            '//! 依据见 research/results/e1-old.out\n'
            'const P: &str = include_str!("../../results/e3-input.out");\n')
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
        sample = os.path.join(work, "research/bench/src/gone.rs")
        open(sample, "w").write('const G: &str = include_str!("../../results/e9-gone.out");\n')
        if not missing_code_inputs(work):
            print("  ✗ 自检失败：装置源码 include_str! 点名了磁盘上没有的产物，missing_code_inputs 却一条都没报")
            print("     → 怎么办：看 include 那条正则与它按 directory 解相对路径那一步。")
            return 1
        # 同一行挪进注释就该不报：注释里的引用是记录，归档改的正是它
        open(sample, "w").write('// const G: &str = include_str!("../../results/e9-gone.out");\n')
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
        # 而门禁 68 号按 GATE_BASE（= HEAD~1）判它「还要着」，两道打架（C450 实测撞过两次）。
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
            print("     → 怎么办：看 base_of —— 它要取 GATE_BASE（与门禁 68 号同一个基准），")
            print("               写死 HEAD 会让两道对同一份记录判相反的事。")
            return 1

    print("  ✓ 自检：不给 GATE_BASE 的 --apply 当场拒绝且一个文件没删、有旧记录判红、删完判绿、判决与本轮新产物不被删、装置 include_str! 读的产物不被删且链接不被改、"
          "指向被删文件的引用改成不带路径的说法、include 指空的产物报得出来、注释里的 include 与产物都在时不误报")
    return 0


def main(argv):
    if "--selftest" in argv:
        return selftest()
    positional = [a for a in argv if not a.startswith("--")]
    root = os.path.abspath(positional[0]) if positional else os.path.abspath(
        os.path.join(os.path.dirname(__file__), "..", ".."))
    return run(root, "--apply" in argv)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
