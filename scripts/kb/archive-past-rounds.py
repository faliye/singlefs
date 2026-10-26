#!/usr/bin/env python3
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: command git
"""上一轮及更早的实验记录归档进版本库：删掉工作区里的那批，保留本轮的。

规则见 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」之下那三段：
冻结只在本轮内有效；这一轮提交之后，下一次提交把上一次留下的那批删掉；本次的照旧留着，本次的验证照旧做得了；
要查删掉的那些，去版本库的历史里找。

用法：
  archive-past-rounds.py --check          还留着上一轮的记录就判红（门禁用）
  archive-past-rounds.py --apply          删掉；同时把指向被删文件的引用改成 `git show <删之前的提交>:<原路径>`（可重跑：磁盘上已删的跳过并报数）
  archive-past-rounds.py --apply --keep-registered   照删别的，被登记表点名的那几份留下、退 0（「把那份留下」这条出路）
  archive-past-rounds.py --selftest

退出码：0 没有该删的、或删完了；1 --check 判红、取不到基准、删完有 include 指空、登记表点名的路径已经不在（指空的登记）；2 --apply 没给 GATE_BASE；
        3 --apply 时有被 .tsv 登记表点名的上一轮产物：别的照删，点名的那几份没删，逐份列出哪张表哪一行点名。

弄坏开关（只给 --selftest 证红用）：
  ARCHIVE_PAST_ROUNDS_BREAK=registry-check-skipped   named_by_registries() 不查登记表，一份都不报
      「✗ 自检失败：门禁登记表点名的上一轮产物」那一格红
      连带红的格：「✗ 登记表点名的 3 处路径已经不在（文件早删了，或这一趟刚删），读那张表的检查拿到的是指空的路径」、「✗ 自检失败：--apply --keep-registered」、「✗ 自检失败：--check 不该把登记表点名、按规则留下的算成该删的」、「✗ 自检失败：变异表里…写法点名的、按子目录点名的」、「✗ 自检失败：登记了别的门禁目录（.claude/gate-directory 写 ops/gates）时」
  ARCHIVE_PAST_ROUNDS_BREAK=gate-directory-default-only   registry_trees() 不读 .claude/gate-directory，门禁目录一律取默认
      「✗ 自检失败：登记了别的门禁目录」那一格红
  ARCHIVE_PAST_ROUNDS_BREAK=gitlinks-kept   past_round_files() 不滤模式位 160000，gitlink 当文件进候选
      「✗ 自检失败：gitlink 进了候选」那一格红（--apply 对它 os.remove 抛 IsADirectoryError 的那条也跟着红）
      连带红的格：「✗ 自检失败：有 gitlink 时 --apply（include 指空）抛了 IsADirectoryError」、「✗ 自检失败：有 gitlink 时 --apply（排除项指不到）抛了 IsADirectoryError」
  ARCHIVE_PAST_ROUNDS_BREAK=glob-range-protects   named_by_registries() 把带通配的射程当点名（按 fnmatch 配候选），射程配到的不删
      「✗ 自检失败：没点名的（只在注释行里提到的、通配射程里的）应当照删」那一格红（自证停在头一组出错的；单测 selftest_registry_namings 时
      『✗ 自检失败：通配射程配到的那一份应当照删』那一格同样红）
  ARCHIVE_PAST_ROUNDS_BREAK=dangling-registry-unreported   dangling_registry_namings() 一条都不报（登记表点名、文件已经不在的那几行）
      「✗ 自检失败：登记表点名、文件已经不在的那几行」那一格红
      连带红的格：「✗ 自检失败：平铺登记表点名的那几份 --apply」、「✗ 这一趟删掉的 2 份上一轮产物被平铺登记表点名（排除表、豁免表不保护它点到的产物，删了那一行就指空）」
  ARCHIVE_PAST_ROUNDS_BREAK=flat-registries-unread   registry_namings() 不读平铺登记表（.claude/term-rename-exempt、.claude/*-exclude）
      「✗ 自检失败：这一趟要删、被平铺登记表点名的那几份」那一格红（它们指空之后的那几行也跟着报不出来）
      连带红的格：「✗ 登记表点名的 2 处路径已经不在（文件早删了，或这一趟刚删），读那张表的检查拿到的是指空的路径」、「✗ 自检失败：平铺登记表点名的那几份 --apply」、「✗ 自检失败：登记表点名、文件已经不在的那几行（.claude/gate.d/stage.tsv:1、.claude/term-rename…」、「✗ 还留着上一轮及更早的实验记录 4 份（本轮的与判决不算）」
  ARCHIVE_PAST_ROUNDS_BREAK=mutation-replacement-counted   registry_namings() 对变异表也按整行认（原文、替换文里的代码文本也算点名）
      「✗ 自检失败：登记表点名、文件已经不在的那几行」那一格红（变异表替换文里写着的 e27-gone-prereg.md 也报成指空）
  ARCHIVE_PAST_ROUNDS_BREAK=exclusion-colon-as-path   stale_exclusions() 把排除项整行当路径，<路径>:<串> 的写法也要整串存在（改前的样子）
      「✗ 自检失败：<路径>:<串> 写法的排除项，路径在就不该报指不到东西」那一格红
  ARCHIVE_PAST_ROUNDS_BREAK=rerun-crashes   run() --apply 不把磁盘上已删的挑出来，对着它们 os.remove（改前的样子）
      「✗ 自检失败：第二次 --apply 抛了 FileNotFoundError」那一格红
  ARCHIVE_PAST_ROUNDS_BREAK=check-counts-index   run() --check 把磁盘上已删、索引里还跟踪着的也算待归档（改前的样子）
      「✗ 自检失败：磁盘上已删、索引里还跟踪着的不该算待归档」那一格红
  ARCHIVE_PAST_ROUNDS_BREAK=rewrite-basename-only   rewrite_links() 把引用改成只留文件名（改前的样子），不写 git show <提交>:<原路径>
      「✗ 自检失败：改引用后 history-consolidate.py loss-check 要退 0」那一格红（产物名里的日期随目录一起丢了；后面紧跟的『引用没改成 git show』那一格被它挡在前面）
      连带红的格：「✗ 1 个日期块有找不到的数或编号」
  ARCHIVE_PAST_ROUNDS_BREAK=history-carriers-ignored   rewrite_links() 不读 .claude/history-carriers，登记的变更史也照改（改前的样子）
      「✗ 自检失败：登记在 .claude/history-carriers 的文件与它们的变更史，--apply 之后要逐字不变」那一格红
      连带红的格：「✗ 自检失败：--apply」
  ARCHIVE_PAST_ROUNDS_BREAK=still-input-by-basename   still_an_input() 按文件名认「还被代码当输入」（改前的样子），不按代码里写的完整路径
      「✗ 自检失败：只有一个被完整路径引到，另一个同名的该删」那一格红

判据（机械，不靠人记）：
  删 —— `research/results/` 与 `research/prompts/` 下，**在 HEAD 里有、且这一轮没碰过**的文件；
        「这一轮」的起点与门禁 doc-process-records 的 knowledge-sync 格同一个基准：`scripts/gate/changed-paths.sh` 的 `gate_diff_base gate`
        （`GATE_BASE`，否则 `@{upstream}` 的 merge-base，都没有就 HEAD），见 base_of 的说明。
        没改过 = 它是上一次提交固化下来的，属于上一轮及更早。
  留 —— 工作区里新加或改过的（本轮在产生的）、三方判决 `*-main-verification.md`（kb 的依据指着它）、
        `abandoned-rounds.tsv`（登记表，门禁 doc-process-records 的 abandoned-rounds 格的输入）、还被代码当输入的产物（still_an_input）、
        被检查读的 .tsv 登记表非注释行点名的产物（named_by_registries：门禁目录下的 .tsv，门禁目录照 .claude/gate-directory 登记、没登记取 .claude/gate.d；
        scripts/ 与 crates/ 下的 .tsv，钉表、变异表 scripts/mutation/tables/、crates/mutations.tsv 都在里面）。
        .tsv 登记表点名的那几份 --apply 不删、退 3，逐份列出哪张表哪一行；出路是先改登记，或者把那份留下（--keep-registered）。
  报 —— 被检查读的登记表（上面那几张 .tsv，加平铺登记表 .claude/term-rename-exempt 与 .claude/*-exclude：一行「<路径>  # 理由」，# 前面那一段是点名）
        点名的 research/ 路径，逐条报两种（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」：
        删之前查一遍被检查读的登记表点名的路径，点到的不删，或者在同一次提交里改掉那几行登记）：
        ① 已经不在（文件早删了）：dangling_registry_namings 逐条列「表:行  路径」，--check 与 --apply 都判红（退 1）；
        ② 这一趟要删、被平铺登记表点名的：平铺登记表是排除表与豁免表，不保护它点到的产物——它点到的冻结证据删了，那一行就该一起删；
           --check 逐条列出、照样判红，--apply 照删，删完它们落进 ①，判红、出路写同一次提交里删掉那几行登记。
  不认 —— 通配（`research/results/e165-*-red-examples.tsv` 这种）是射程，不点名任何一份，也不保护任何一份；代码里拼出来的路径
        （目录加一个从别的文件名换后缀得来的名字）这里看不见，也不保护。代码把 research/ 下的文件当输入本身就是要消掉的依赖
        （research/ 整个可删，.claude/rules/directory-layout.md「research/：可以整个删掉的研究目录」）：读它的检查要改成读跟着测试放的登记，
        或者文件不在时报本次未跑；still_an_input 按字面文件名留下的那几份是还没改完的旧依赖，留着是免得这一趟当场塌，不是该保护的对象。
        「还被代码当输入」按代码非注释行里写出的**完整路径**（如 research/prompts/leg-a/src/main.rs）认，不按文件名：四个腿目录里各有一份 src/main.rs，
        只有一个被完整路径引到时，其余三个照删（2026-10-18 实测：按文件名把 4 个腿模型目录里的 src/main.rs 全留下了）。
  ⚠️ 删文件会让别处指向它的链接指空，所以 --apply 同时把那些引用改成 `git show <删之前的提交>:<原路径>`（反引号包着，提交取删之前的 HEAD），
     不留一个指空的路径——共享 `gate.sh` 的「链接指向」阶段（`.claude/singlefs-ai-sop/scripts/link-targets.py`）判的就是这个。
     不剥成文件名：产物名里的目录与日期随路径一起丢了，门禁 doc-decisions 的 history-loss 格（`scripts/kb/history-consolidate.py loss-check`）判变更史丢了产物名；
     `git show <提交>:<路径>` 里路径原样在，那一格认得出。已经写成 `git show <提交>:<路径>` 的不再改（重跑不套娃）；
     markdown 链接目标 `](路径)` 里放不下空格，仍只留文件名。
     不改的：登记在 `.claude/history-carriers` 的文件（读那张表：一行一个目录或文件，目录带结尾的 /），与登记项的变更史
     （登记项 `<目录>/` 或 `<名>.md` 旁的 `<名>-history.md` 及它的 `.en.md`、`.ja.md` 译文：登记 `.claude/kb/decisions/` 的旁边是 `.claude/kb/decisions-history.md`）。
     变更史记的是当时的原文，指向被删产物的路径在 git 历史里取得回；改写它既丢了原样，又让门禁 doc-decisions 的 history-loss 格当场红（2026-10-18 实测）。
     --apply 报出跳过了几份文件、几处引用；登记表不在就一份都不跳。
  可重跑：上一趟 --apply 删了文件、还没暂存或提交时，索引里仍跟踪着它们。这类「磁盘上已删、索引里还跟踪着」的不再算待归档（--check 不为它们判红）、
  --apply 不再对它们 os.remove，只报数；它们的引用照改（上一趟被打断时没改完的补上）。
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
HISTORY_CARRIERS_FILE = ".claude/history-carriers"   # 哪些 kb 文件带「## 历史版本」节：一行一个目录或文件，# 之后是理由（.claude/rules/changelog-format.md）
# 会把留存产物当**输入**的地方：门禁阶段按路径读它去比对、装置源码 include_str! 在编译期读它。
# 这些不是「记录」，删了不是「去 git 历史里查」，是**这道检查没得比 / 这个 crate 编不过**。
INPUT_TREES = (".claude/gate.d", ".claude/scripts", "scripts", "crates")
INPUT_SUFFIX = (".sh", ".py", ".rs", ".toml")
# 被检查读的登记表：门禁目录（照 .claude/gate-directory 登记，见 registry_trees）与这两处下的 .tsv——钉表、变异表（scripts/mutation/tables/、
# crates/mutations.tsv）、crate 里的登记表（verus-must-red-worlds.tsv 这类）。删之前查它们非注释行点名的路径
# （`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」：删之前查一遍被检查读的登记表点名的路径）。
REGISTRY_TREES_BESIDE_GATE = ("scripts", "crates")
REGISTRY_SUFFIX = ".tsv"
REGISTRY_SKIPPED_DIRECTORIES = ("fixtures", "target", ".git")
# 登记表一格里的产物路径：到空白、引号、括号、冒号（「文件:行号」）、井号、通配符为止；跟着通配符的是射程（e165-*-red-examples.tsv），不是点名，
# 也不保护它配得到的任何一份（文件头「不认」一段）
REGISTRY_PATH = re.compile(r"research/(?:results|prompts)/[^\s`'\"<>|()（）\[\]\\,，。、；：:#*?]+")
# 平铺登记表：一行一条「<路径>[:「行内锚」]  # 理由」，空白加 # 之后是理由、不算点名。门禁 doc-text 的 term-renames 格读的豁免表，
# 与各道检查从 .claude/*-exclude 读的排除表（门禁 code-source-discipline 的 format-constants 格读 .claude/format-constants-exclude）
FLAT_REGISTRY_FILES = (".claude/term-rename-exempt",)
FLAT_REGISTRY_GLOB = ".claude/*-exclude"
FLAT_REGISTRY_REASON = re.compile(r"\s+#")
# 变异表（每行「变异名、文件、原文、替换文……」：scripts/mutation/tables/*.tsv、crates/mutations.tsv、Verus 必红世界表）只认前两列：
# 第三列起是被改坏的代码与替换上去的代码，里面注释提到的 research/ 路径是代码文本，不是登记的输入
MUTATION_TABLE_NAMING_COLUMNS = 2


def is_mutation_table(table):
    """table 是不是变异表（registry_namings 对它只认前 MUTATION_TABLE_NAMING_COLUMNS 列）。"""
    return (table.startswith("scripts/mutation/tables/") or os.path.basename(table) == "mutations.tsv"
            or os.path.basename(table).endswith("must-red-worlds.tsv"))
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


def written_as_path(text, relative):
    """text 里有没有把 relative 整条路径写出来：后面不能紧接文件名字符（research/results/e1.out 不算 e1.output 的点名）。"""
    start = text.find(relative)
    while start >= 0:
        following = text[start + len(relative):start + len(relative) + 1]
        if not (following.isascii() and (following.isalnum() or following in ("_", "-"))):
            return True
        start = text.find(relative, start + 1)
    return False


def still_an_input(root, candidates):
    """候选里还被代码当输入的那些：门禁阶段按路径读它去比对、装置源码 include_str! 在编译期读它。
    返回 {相对路径: [点名它的源文件]}。这些**不删**——它们不是上一轮的记录，是这一轮还在跑的东西的输入。

    2026-09-21 实测两次，都是删完才发现：① 段序列登记表那一道门禁（段序列登记表与 E142 产物逐字比对，已搬进 doc-registries 的 segment-registry 那一格）连同它的自证一起塌；
    ② 归档删掉四份跨实验断言读的产物之后，research 整个编不过（e134 读 e133、e136 读 e134 与 e109、e137 读 e136），
    而 HEAD 上就是这个状态。第一次只把射程补到门禁脚本与 scripts，漏掉装置源码，于是第二次照样发生——
    所以这里按「哪些后缀是代码」扫，不按「我记得哪些目录会引」扫。

    只认代码里字面写出的完整路径，不按文件名（文件头「不认」一段）：四个腿目录里各有一份 src/main.rs，按文件名认就全留下了。
    代码里拼出来的路径（例：按红例账的文件名换后缀配同一目录的杀伤界产物）这里看不见、不保护，
    那份产物照删；读它的那一道要改成不读 research/（跟着测试放的登记，或文件不在报本次未跑），不是在这里补一条认法。"""
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
                if BROKEN == "still-input-by-basename":
                    for relative in candidates:
                        if os.path.basename(relative) in text:
                            found.setdefault(relative, []).append(os.path.relpath(source, root))
                    continue
                if "research/" not in text:
                    continue
                for relative in candidates:
                    if written_as_path(text, relative):
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


def registry_tables(root):
    """被检查读的登记表：[(相对路径, 种类)]，种类是 "tsv"（门禁目录、scripts/、crates/ 下的 .tsv）或 "flat"（平铺登记表）。"""
    tables = []
    for tree in registry_trees(root):
        for directory, subdirectories, names in os.walk(os.path.join(root, tree)):
            subdirectories[:] = sorted(name for name in subdirectories if name not in REGISTRY_SKIPPED_DIRECTORIES)
            tables += [(os.path.relpath(os.path.join(directory, name), root), "tsv") for name in sorted(names) if name.endswith(REGISTRY_SUFFIX)]
    if BROKEN != "flat-registries-unread":
        import glob as _glob
        flat = set(FLAT_REGISTRY_FILES) | {os.path.relpath(path, root) for path in _glob.glob(os.path.join(root, FLAT_REGISTRY_GLOB))}
        tables += [(relative, "flat") for relative in sorted(flat) if os.path.isfile(os.path.join(root, relative))]
    return tables


def registry_namings(root):
    """登记表非注释行点名的 research/ 路径：[(表, 行号, 路径, 是不是射程, 种类)]。
    .tsv 算到头一个「制表符加 #」之前（表的理由列照 stage-inputs.tsv 那种写法以 # 起头，里面的「依据见 research/…」是记录引用），
    之前的各格都算（一格里写什么由读它的检查定，这里不分列），变异表例外：只算变异名与文件两列（is_mutation_table）；平铺登记表只算空白加 # 之前那一段（之后是理由）。
    注释行（# 起头）里写的「依据见 research/…」是记录引用，不算——与 still_an_input 跳注释同一个理由。
    跟着通配符的是射程：照样列出（是不是射程那一项为真），点名、保护、指空三样都不算它。"""
    namings = []
    for table, kind in registry_tables(root):
        try:
            lines = open(os.path.join(root, table), encoding="utf-8", errors="ignore").read().splitlines()
        except OSError:
            continue
        for number, line in enumerate(lines, 1):
            if not line.strip() or line.lstrip().startswith("#"):
                continue
            named_part = FLAT_REGISTRY_REASON.split(line, 1)[0] if kind == "flat" else line.split("\t#", 1)[0]
            if kind == "tsv" and is_mutation_table(table) and BROKEN != "mutation-replacement-counted":
                named_part = "\t".join(named_part.split("\t")[:MUTATION_TABLE_NAMING_COLUMNS])
            for match in REGISTRY_PATH.finditer(named_part):
                is_range = named_part[match.end():match.end() + 1] in ("*", "?")
                namings.append((table, number, match.group(0), is_range, kind))
    return namings


def named_by_registries(root, candidates, kind="tsv"):
    """候选里被登记表点名的：{相对路径: ["<表>:<行号>", …]}。kind="tsv" 的那几份 --apply 不删（保护）；kind="flat" 的照删、只报（文件头「报」②）。
    点名只认整条文件路径，或 research/ 下的子目录（带不带结尾的 / 都算，目录下的候选都算点到）；research/results/、research/prompts/ 本身与带通配符的是射程，不算点名。
    弄坏开关 glob-range-protects 下射程按 fnmatch 配候选、配到的也算点名。"""
    if BROKEN == "registry-check-skipped":
        return {}
    import fnmatch as _fnmatch
    candidates = set(candidates)
    found = {}
    for table, number, target, is_range, table_kind in registry_namings(root):
        if table_kind != kind:
            continue
        if is_range:
            if BROKEN != "glob-range-protects":
                continue                # 通配：射程，不是点名，也不保护
            pattern = target + "*"
            hits = [path for path in candidates if _fnmatch.fnmatch(path, pattern)]
        else:
            directory_prefix = target.rstrip("/") + "/"
            hits = [path for path in candidates if path == target or path.startswith(directory_prefix)]
        naming = "%s:%d" % (table, number)
        for path in hits:
            if naming not in found.setdefault(path, []):   # 同一行点名两次只列一次
                found[path].append(naming)
    return found


def path_present(root, target):
    """点名的路径还在：是文件，或是底下至少还有一份文件的目录（删光了文件只剩空目录的不算：git 不记空目录，换一棵树就没了）。"""
    path = os.path.join(root, target.rstrip("/"))
    if os.path.isdir(path):
        return any(names for _, _, names in os.walk(path))
    return os.path.lexists(path)


def dangling_registry_namings(root):
    """登记表点名、而磁盘上已经不在的 research/ 路径（文件头「报」①）：["<表>:<行号>  <路径>", …]。射程不算。
    读那张表的检查拿到的是指空的路径；出路是同一次提交里删掉或改掉那几行登记。"""
    if BROKEN == "dangling-registry-unreported":
        return []
    dangling = []
    for table, number, target, is_range, _kind in registry_namings(root):
        if is_range or path_present(root, target):
            continue
        entry = "%s:%d  %s" % (table, number, target)
        if entry not in dangling:
            dangling.append(entry)
    return dangling


def report_dangling(dangling):
    """dangling_registry_namings 的结果打出来（判红那一种）；没有就什么都不打。"""
    if not dangling:
        return
    print("  ✗ 登记表点名的 %d 处路径已经不在（文件早删了，或这一趟刚删），读那张表的检查拿到的是指空的路径：" % len(dangling))
    for entry in dangling:                                         # gate-lint:detail
        print("      %s" % entry)
    print("     → 怎么办：同一次提交里删掉那几行登记，或把那一格的路径改成仓里在的文件（.claude/singlefs-ai-sop/rules/evidence-discipline.md「原样保存的证据不许事后改」）；")
    print("               那一格只是记录引用的，改成只留文件名、不带路径（登记表的格子放不下 git show 写法）；读它的检查还要那份文件的，")
    print("               改成读跟着测试放的登记或文件不在报本次未跑（.claude/rules/directory-layout.md），不去把产物找回来。")


def report_gone(root, gone, apply_changes):
    """磁盘上已删、索引里还跟踪着的那几份：只报数。--apply 时它们的引用另在 rewrite_links 里照改。"""
    if not gone:
        return
    print("      另有 %d 份磁盘上已删、索引里还跟踪着（上一趟 --apply 删的，这批删除暂存或提交之后不再出现）：不算待归档，%s"
          % (len(gone), "这一趟不再删它们、只补改它们的引用" if apply_changes else "不为它们判红"))


def report_history_kept(kept_files, kept_hits):
    """登记在 .claude/history-carriers 的文件与它们的变更史里没改的引用（文件头「⚠️」一段）：只报数，路径在 git 历史里取得回。"""
    if kept_files:
        print("      没改 %d 份登记在 %s 的文件与它们的变更史里指向被删产物的 %d 处引用（变更史记当时的原文；路径在 git 历史里取得回）"
              % (kept_files, HISTORY_CARRIERS_FILE, kept_hits))


def report_flat_named(flat_named, apply_changes):
    """这一趟要删、被平铺登记表点名的那几份（文件头「报」②）。"""
    if not flat_named:
        return
    verb = "删掉" if apply_changes else "要删"
    print("  ✗ 这一趟%s的 %d 份上一轮产物被平铺登记表点名（排除表、豁免表不保护它点到的产物，删了那一行就指空）：" % (verb, len(flat_named)))
    for relative in sorted(flat_named):                            # gate-lint:detail
        print("        %s  ← %s" % (relative, "、".join(flat_named[relative])))
    print("     → 怎么办：在归档的同一次提交里删掉那几行登记（.claude/singlefs-ai-sop/rules/evidence-discipline.md「原样保存的证据不许事后改」：")
    print("               删之前查一遍被检查读的登记表点名的路径，点到的不删，或者在同一次提交里改掉那几行登记）；那一行豁免的东西没了，留着只会让那道检查红。")


def text_files(root="."):
    for base in SCAN:
        for directory, _, names in os.walk(os.path.join(root, base)):
            for name in names:
                if name.endswith((".md", ".tsv")):
                    yield os.path.relpath(os.path.join(directory, name), root)


GIT_SHOW_BEFORE = re.compile(r"git show [0-9a-f]{7,40}:$")


def commit_before_deletion(root):
    """删之前的提交（HEAD）的短哈希；改写后的引用 `git show <它>:<原路径>` 读得到被删的文件。"""
    return git("rev-parse", "--short=12", "HEAD", root=root).strip()


def history_carriers(root):
    """.claude/history-carriers 登记的目录或文件（一行一条，# 之后是理由；相对仓根，目录带结尾的 /）。表不在返回空。"""
    if BROKEN == "history-carriers-ignored":
        return []
    try:
        lines = open(os.path.join(root, HISTORY_CARRIERS_FILE), encoding="utf-8").read().splitlines()
    except OSError:
        return []
    return [entry for entry in (line.split("#", 1)[0].strip() for line in lines) if entry]


def history_text_guard(root):
    """→ 函数 relative -> 是不是登记在 history-carriers 的文件，或登记项的变更史。
    登记项 `<目录>/`（目录下的文件都算）与 `<文件>`（逐字）；变更史是登记项旁的 `<名>-history.md` 与它的 `.en.md`、`.ja.md` 译文
    （`.claude/kb/decisions/` 旁是 `.claude/kb/decisions-history.md`）：变更史自己不带「## 历史版本」，登记表里没有它，改写它却一样丢当时的原文。"""
    entries = history_carriers(root)
    prefixes = tuple(entry for entry in entries if entry.endswith("/"))
    exact = {entry for entry in entries if not entry.endswith("/")}
    history_stems = set()
    for entry in entries:
        stem = entry.rstrip("/")
        history_stems.add((stem[:-3] if stem.endswith(".md") else stem) + "-history")

    def guarded(relative):
        if relative in exact or relative.startswith(prefixes):
            return True
        return any(relative == stem + ".md" or (relative.startswith(stem + ".") and re.fullmatch(r"[a-z]{2}\.md", relative[len(stem) + 1:]))
                   for stem in history_stems)
    return guarded


def rewrite_links(root, doomed, commit=None):
    """把指向被删文件的引用改成 `git show <删之前的提交>:<原路径>`，别留指空的路径，也别把路径剥成文件名（产物名里的目录与日期会丢）。
    已经写成 `git show <提交>:<路径>` 的不再改；markdown 链接目标 `](路径)` 里放不下空格，只留文件名。
    登记在 `.claude/history-carriers` 的文件与它们的变更史（history_text_guard）整份不改，只数跳过了几份、几处。
    → (改了几处, 动了几份文件, 跳过的变更史份数, 跳过的处数)"""
    doomed = set(doomed)
    changed = 0
    touched = 0
    kept_files = 0
    kept_hits = 0
    guarded = history_text_guard(root)
    if doomed and commit is None and BROKEN != "rewrite-basename-only":
        commit = commit_before_deletion(root)
    for relative in text_files(root):
        path = os.path.join(root, relative)
        if guarded(relative):
            try:
                kept = sum(1 for match in LINK.finditer(open(path, encoding="utf-8").read()) if match.group(0) in doomed)
            except (UnicodeDecodeError, OSError):
                kept = 0
            if kept:
                kept_files += 1
                kept_hits += kept
            continue
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
            before = text[:match.start()]
            if GIT_SHOW_BEFORE.search(before[-60:]):
                return target                   # 已经是 git show <提交>:<路径>，再改就套娃了
            hits += 1
            if BROKEN == "rewrite-basename-only" or before.endswith("]("):
                return os.path.basename(target)
            # 反引号里的路径原地换成 git show 写法（外面那对反引号照用）；不在反引号里的，自己包一对
            inside_backticks = before.endswith("`") and text[match.end():match.end() + 1] == "`"
            shown = f"git show {commit}:{target}"
            return shown if inside_backticks else f"`{shown}`"

        new = LINK.sub(fix, text)
        if hits:
            # 反引号里的路径换掉之后那对反引号就多余了，留着不碍事；括注重复的收一下
            open(path, "w", encoding="utf-8").write(new)
            changed += hits
            touched += 1
    return changed, touched, kept_files, kept_hits


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
            if not target or os.path.exists(os.path.join(root, target)):
                continue
            # 排除项有两种写法：整份文件「<路径>」，和文件里的一条「<路径>:<串>」（门禁 code-source-discipline 的 data-names 格那张表）。
            # 后一种要看冒号前的路径还在不在；整行当路径会把存量里两千多条都报成指不到东西。
            path_part = target.split(":", 1)[0]
            if BROKEN != "exclusion-colon-as-path" and path_part != target and os.path.exists(os.path.join(root, path_part)):
                continue
            stale.append("%s:%d  %s" % (os.path.relpath(table, root), number, path_part if BROKEN != "exclusion-colon-as-path" else target))
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
        # 磁盘上已删、索引里还跟踪着的（上一趟 --apply 删的，还没暂存或提交）：不再算待归档，--apply 也不再删它们
        splitting = BROKEN != ("rerun-crashes" if apply_changes else "check-counts-index")
        gone = [p for p in doomed if splitting and not os.path.lexists(os.path.join(root, p))]
        doomed = [p for p in doomed if p not in gone]
    except BaseUnavailable as error:
        print(f"  ✗ 取不到「这一轮」的起点（{error}），分不出哪些实验记录是上一轮的")
        print("     → 怎么办：在仓根跑、确认 scripts/gate/changed-paths.sh 在；基准取法与门禁 doc-process-records 的 knowledge-sync 格是同一份，那一份坏了两边一起坏。")
        return 1
    inputs = still_an_input(root, doomed)
    doomed = [p for p in doomed if p not in inputs]
    registered = named_by_registries(root, doomed)
    doomed = [p for p in doomed if p not in registered]
    flat_named = named_by_registries(root, doomed, kind="flat")    # 照删，只报（文件头「报」②）
    dangling = dangling_registry_namings(root)                      # 删之前就已经指空的（文件头「报」①）
    if inputs:
        print("      留下 %d 份还被代码当输入的产物（门禁按路径读它比对、装置 include_str! 编译期读它）：" % len(inputs))
        for relative in sorted(inputs):                              # gate-lint:detail
            print("        %s  ← %s" % (relative, "、".join(sorted(set(inputs[relative])))))
    if registered:
        print("      留下 %d 份被检查读的登记表点名的产物（删了那一行就指空，读它的检查当场红；查的树：%s）："
              % (len(registered), "、".join(registry_trees(root))))
        for relative in sorted(registered):                          # gate-lint:detail
            print("        %s  ← %s" % (relative, "、".join(registered[relative])))
    if not doomed:
        refused = apply_changes and refuse_registered(registered, keep_registered)
        report_gone(root, gone, apply_changes)
        if apply_changes and gone:
            changed, touched, kept_files, kept_hits = rewrite_links(root, gone)
            print("      补改了上一趟没改完的引用：%d 份文件 %d 处" % (touched, changed))
            report_history_kept(kept_files, kept_hits)
        if dangling:
            report_dangling(dangling)
            return 1
        if refused:
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
        report_gone(root, gone, False)
        report_flat_named(flat_named, False)
        report_dangling(dangling)
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
        print("               它同时把别处指向这些文件的引用改成 `git show <删之前的提交>:<原路径>`（不剥成文件名）；")
        print("               要查删掉的内容去 git 历史里找，有疑问就重新验证、不翻旧证据。")
        return 1
    report_flat_named(flat_named, True)
    report_gone(root, gone, True)
    commit = commit_before_deletion(root)
    changed, touched, kept_files, kept_hits = rewrite_links(root, doomed + gone, commit)   # 引用在删之前改，写的是删之前的提交
    for relative in doomed:
        os.remove(os.path.join(root, relative))
    dangling = dangling_registry_namings(root)                      # 删完再算一遍：这一趟删掉的平铺登记表点名的那几份落进来
    dangling_namings = {entry.split("  ", 1)[0] for entry in dangling}
    stale = [entry for entry in stale_exclusions(root) if entry.split("  ", 1)[0] not in dangling_namings]   # 同一行不报两遍
    losing = missing_code_inputs(root)
    print("  ✓ 删了 %d 份上一轮及更早的实验记录；%d 份文件里 %d 处引用改成了 `git show <删之前的提交>:<原路径>`" % (len(doomed), touched, changed))
    report_history_kept(kept_files, kept_hits)
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
    if dangling:
        report_dangling(dangling)
        return 1
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
        code = selftest_registry_namings()
        if code != 0:
            return code
        code = selftest_in_temporary_directory()
        if code != 0:
            return code
        code = selftest_rerun_and_exclusions()
        if code != 0:
            return code
        code = selftest_history_carriers_and_full_paths()
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


def selftest_registry_namings():
    """文件头「报」「不认」两段：平铺登记表（.claude/term-rename-exempt、.claude/*-exclude）点名的照删、--check 逐条报；
    登记表点名而文件已经不在的逐条报、判红；通配射程与代码里拼出来的路径不保护任何一份。"""
    import contextlib
    import io
    import tempfile
    failures, checked = [], 0
    with tempfile.TemporaryDirectory() as work:
        def write(relative, text):
            path = os.path.join(work, relative)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            open(path, "w", encoding="utf-8").write(text)

        def exists(relative):
            return os.path.exists(os.path.join(work, relative))

        def run_captured(apply_changes):
            buffer = io.StringIO()
            if apply_changes:
                os.environ["GATE_BASE"] = "HEAD"
            try:
                with contextlib.redirect_stdout(buffer):
                    code = run(work, apply_changes)
            finally:
                os.environ.pop("GATE_BASE", None)
            return code, buffer.getvalue()

        git("init", "-q", root=work)
        subprocess.run(["git", "config", "user.email", "t@t"], cwd=work, check=True)
        subprocess.run(["git", "config", "user.name", "t"], cwd=work, check=True)
        for relative in ("research/results/e20-term.out", "research/prompts/e21-model/a.rs", "research/results/e22-glob-a.out",
                         "research/results/e23-red-examples.tsv", "research/results/e23-kill-bounds.out"):
            write(relative, "上一轮\n")
        write(".claude/term-rename-exempt", "research/results/e20-term.out:「旧名」  # 样本：按行豁免\n"
                                            "research/results/e24-gone.out  # 样本：文件早删了，这一行指空\n")
        write(".claude/format-constants-exclude", "research/prompts/e21-model/  # 样本：冻结证据目录\n")
        write(".claude/gate.d/ledgers.tsv", "research/results/e22-*.out\t通配射程，不点名也不保护\n")
        # 变异表只认变异名与文件两列：替换文里注释提到的路径是代码文本（不报），文件那一列指空照报
        write("scripts/mutation/tables/arms.tsv", "臂一\tcrates/x.rs\t原文\t// 依据 research/prompts/e27-gone-prereg.md 5.2\t-p x\tt\n"
                                                  "臂二\tresearch/results/e28-gone.out\t原文\t替换文\t-p x\tt\n")
        write(".claude/gate.d/stage.tsv", "E1\tquestion-row=research/prompts/e25-gone-questions.md#6:够判\n"
                                          "E2\t-\t-\t# 理由列里的记录引用不算点名：依据见 research/prompts/e26-cited-only.md\n")
        # 账的文件名在代码里写成字面（still_an_input 认得、留下），杀伤界产物的路径是换后缀拼出来的（看不见、照删）
        write("scripts/experiment-apparatus/pin.py", 'ledger = "research/results/e23-red-examples.tsv"\n'
                                            'bounds = ledger.replace("-red-examples.tsv", "-kill-bounds.out")\n')
        git("add", "-A", root=work)
        subprocess.run(["git", "commit", "-qm", "上一轮"], cwd=work, check=True)

        code_check, output_check = run_captured(False)
        checked += 1
        if (code_check != 1 or "登记表点名的 3 处路径已经不在" not in output_check
                or "scripts/mutation/tables/arms.tsv:2  research/results/e28-gone.out" not in output_check
                or "e27-gone-prereg" in output_check
                or ".claude/gate.d/stage.tsv:1  research/prompts/e25-gone-questions.md" not in output_check
                or ".claude/term-rename-exempt:2  research/results/e24-gone.out" not in output_check
                or "e26-cited-only" in output_check):
            failures.append((f"登记表点名、文件已经不在的那几行（.claude/gate.d/stage.tsv:1、.claude/term-rename-exempt:2、scripts/mutation/tables/arms.tsv:2）--check 应当逐条报、退 1，"
                             f"理由列里的记录引用（stage.tsv:2）与变异表替换文里的路径（arms.tsv:1）不报，实际退 {code_check}：{output_check[-700:]}",
                             "看 dangling_registry_namings() 与 run() 里 report_dangling() 那几处"))
        checked += 1
        if ("research/results/e20-term.out  ← .claude/term-rename-exempt:1" not in output_check
                or "research/prompts/e21-model/a.rs  ← .claude/format-constants-exclude:1" not in output_check
                or "这一趟要删的 2 份上一轮产物被平铺登记表点名" not in output_check):
            failures.append((f"这一趟要删、被平铺登记表点名的那几份 --check 应当逐份列出表与行：{output_check[-700:]}",
                             "看 registry_tables() 读平铺登记表、named_by_registries(kind=\"flat\") 与 report_flat_named()"))
        code_apply, output_apply = run_captured(True)
        checked += 1
        if exists("research/results/e22-glob-a.out"):
            failures.append(("通配射程配到的那一份应当照删（射程不点名、不保护）", "看 named_by_registries() 跳射程那一处"))
        checked += 1
        if exists("research/results/e23-kill-bounds.out") or not exists("research/results/e23-red-examples.tsv"):
            failures.append(("代码里拼出来的路径应当看不见、照删（e23-kill-bounds.out），字面写着的那一份应当留下（e23-red-examples.tsv）",
                             "看 still_an_input() 只按字面文件名认那一段"))
        checked += 1
        if (code_apply != 1 or exists("research/results/e20-term.out") or exists("research/prompts/e21-model/a.rs")
                or ".claude/term-rename-exempt:1  research/results/e20-term.out" not in output_apply
                or ".claude/format-constants-exclude:1  research/prompts/e21-model/" not in output_apply):
            failures.append((f"平铺登记表点名的那几份 --apply 应当照删，删完那几行落进「已经不在」逐条报、退 1，实际退 {code_apply}：{output_apply[-700:]}",
                             "看 run() 里删完再算一遍 dangling_registry_namings() 那一句"))
    for failure, howto in failures:
        print(f"  ✗ 自检失败：{failure}")                 # gate-lint:detail
        print(f"     → 怎么办：{howto}；ARCHIVE_PAST_ROUNDS_BREAK 设着的话这里本来就该红")
    if failures:
        return 1
    print(f"  ✓ 自检（登记表点名的路径，查了 {checked} 格）：登记表点名而文件已经不在的逐条报、判红；平铺登记表（.claude/term-rename-exempt、.claude/*-exclude）"
          "点名的这一趟要删的逐条报、--apply 照删之后落进「已经不在」判红；通配射程与代码里拼出来的路径不保护任何一份")
    return 0


def selftest_in_temporary_directory():
    import shutil
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
        os.makedirs(os.path.join(work, "scripts/experiment-apparatus"))
        open(os.path.join(work, "scripts/experiment-apparatus/probe.rs"), "w").write(
            '//! 依据见 research/results/e1-old.out\n'
            'const P: &str = include_str!("../../research/results/e3-input.out");\n')
        open(os.path.join(work, ".claude/kb/page.md"), "w").write(
            "产物 `research/results/e1-old.out`，判决 `research/prompts/r1-main-verification.md`，"
            "输入 `research/results/e3-input.out`\n")
        # 变更史里引了带日期目录的产物：改引用时路径要原样留在 git show <提交>:<路径> 里，门禁 doc-decisions 的 history-loss 格才认得出产物名没丢
        os.makedirs(os.path.join(work, "research/prompts/sample-2026-10-15"))
        open(os.path.join(work, "research/prompts/sample-2026-10-15/r2-words-report.md"), "w").write("第二节：D44、D45 两行\n")
        history_path = os.path.join(work, ".claude/kb/decisions-history.md")
        open(history_path, "w").write("# 决策变更史\n\n## D1（样本）\n\n### 2026-10-15\n\n- 判决 `research/prompts/sample-2026-10-15/r2-words-report.md` 第二节 D44、D45 两行，数 44 与 45\n")
        history_before = os.path.join(work, "history-before.md")
        shutil.copyfile(history_path, history_before)
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
        sample = os.path.join(work, "scripts/experiment-apparatus/gone.rs")
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
        history_after = open(os.path.join(work, ".claude/kb/decisions-history.md"), encoding="utf-8").read()
        loss = subprocess.run([sys.executable, os.path.join(os.path.dirname(os.path.abspath(__file__)), "history-consolidate.py"), "loss-check",
                               history_before, os.path.join(work, ".claude/kb/decisions-history.md"), "--repo", work], capture_output=True, text=True)
        if loss.returncode != 0 or "git show" not in history_after:
            print("  ✗ 自检失败：改引用后 history-consolidate.py loss-check 要退 0（产物名里的目录与日期一并留在 git show 写法里），实际退 %d：%s"
                  % (loss.returncode, (loss.stdout + loss.stderr)[-400:]))
            print("     → 怎么办：看 rewrite_links()：不要把引用剥成文件名，门禁 doc-decisions 的 history-loss 格判变更史丢了产物名。")
            return 1
        if re.search(r"(?<!git show [0-9a-f]{12}:)research/results/e1-old\.out", page):
            print("  ✗ 自检失败：指向被删文件的引用还留着指空的路径")
            print("     → 怎么办：看 rewrite_links() 的 LINK 正则与 doomed 集合。")
            return 1
        if not re.search(r"`git show [0-9a-f]{12}:research/results/e1-old\.out`", page):
            print("  ✗ 自检失败：引用没改成 `git show <删之前的提交>:<原路径>`：" + page.strip())
            print("     → 怎么办：看 rewrite_links() 的 fix()：反引号里的路径原地换成 git show 写法，提交取删之前的 HEAD。")
            return 1
        os.environ["GATE_BASE"] = "HEAD"
        try:
            again = run(work, True)                  # 第二遍：已改成 git show 写法的不套娃，磁盘上已删的跳过
        finally:
            os.environ.pop("GATE_BASE", None)
        if again != 0 or open(os.path.join(work, ".claude/kb/decisions-history.md"), encoding="utf-8").read() != history_after or \
                open(os.path.join(work, ".claude/kb/page.md"), encoding="utf-8").read() != page:
            print("  ✗ 自检失败：第二遍改引用把已经写成 git show 的又改了一遍，或退出码不是 0（%s）" % again)
            print("     → 怎么办：看 rewrite_links() 的 GIT_SHOW_BEFORE：路径前面已经是 git show <提交>: 的不再改。")
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
          "指向被删文件的引用改成 `git show <删之前的提交>:<原路径>`（再跑 history-consolidate.py loss-check 退 0、重跑不套娃）、include 指空的产物报得出来、注释里的 include 与产物都在时不误报、"
          "没设 GATE_BASE 时基准取上游的 merge-base（与门禁 doc-process-records 的 knowledge-sync 格同一份取法）")
    return 0


def selftest_rerun_and_exclusions():
    """三格：① 排除项的 <路径>:<串> 写法只看冒号前的路径还在不在；② 第二次 --apply（上一趟删了、还没暂存，索引里仍跟踪着）不抛 FileNotFoundError、已删的跳过并报数；
    ③ --check 不把「磁盘上已删、索引里还跟踪着」的算成待归档。"""
    import contextlib
    import io
    import tempfile
    failures, checked = [], 0
    with tempfile.TemporaryDirectory() as work:
        def write(relative, text):
            path = os.path.join(work, relative)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            open(path, "w", encoding="utf-8").write(text)

        def captured(apply_changes, base="HEAD"):
            buffer = io.StringIO()
            os.environ["GATE_BASE"] = base
            try:
                with contextlib.redirect_stdout(buffer):
                    code = run(work, apply_changes)
            except Exception as error:
                return error, buffer.getvalue()
            finally:
                os.environ.pop("GATE_BASE", None)
            return code, buffer.getvalue()

        git("init", "-q", root=work)
        subprocess.run(["git", "config", "user.email", "t@t"], cwd=work, check=True)
        subprocess.run(["git", "config", "user.name", "t"], cwd=work, check=True)
        # 起点提交：两份记录在它之后才进仓。第一趟 --apply 取 HEAD 当基准把它们删了；之后另一个基准（门禁单跑 --check 取的上游 merge-base 就是这一种）
        # 取在它们进仓之前，磁盘上没有它们、diff 也不报，索引里却还跟踪着——候选从索引取，就把已删的也算进来
        write("README.md", "起点\n")
        git("add", "-A", root=work)
        subprocess.run(["git", "commit", "-qm", "起点"], cwd=work, check=True)
        starting_commit = git("rev-parse", "HEAD", root=work).strip()
        write("research/results/e1-old.out", "上一轮\n")
        write("research/results/e2-old.out", "上一轮\n")
        write(".claude/kb/page.md", "产物 `research/results/e1-old.out` 与 research/results/e2-old.out\n")
        write("docs/exists.md", "存量里的一条\n")
        # 数据名排除表：<路径>:<串> 写法（路径在、串在文件里的）与整份文件写法，各一条指得到、一条指不到
        write(".claude/data-name-lint-exclude", "# 样本\ndocs/exists.md:some_name  # 样本：路径在，不该报\ndocs/gone.md:other_name  # 样本：路径不在，该报\n"
                                                 "docs/exists.md  # 样本：整份文件，在\ndocs/gone-whole.md  # 样本：整份文件，不在\n")
        git("add", "-A", root=work)
        subprocess.run(["git", "commit", "-qm", "上一轮"], cwd=work, check=True)
        code, output = captured(True)
        checked += 1
        stale_lines = [line for line in output.splitlines() if line.strip().startswith(".claude/data-name-lint-exclude:")]
        if code != 0 or len(stale_lines) != 2 or any("docs/exists.md" in line for line in stale_lines) \
                or not any("docs/gone.md" in line and "other_name" not in line for line in stale_lines) or not any("docs/gone-whole.md" in line for line in stale_lines):
            failures.append(("<路径>:<串> 写法的排除项，路径在就不该报指不到东西（路径不在的才报，且报的是路径）；整份文件写法照旧，"
                             f"应当只报 docs/gone.md 与 docs/gone-whole.md 两条，实际退 {code}：{stale_lines}",
                             "看 stale_exclusions()：冒号前的路径还在，这一条就不算指不到"))
        checked += 1
        if os.path.exists(os.path.join(work, "research/results/e1-old.out")):
            failures.append(("第一趟 --apply 应当删掉 e1-old.out", "看 run() 的 os.remove 那一段"))
        # 第二趟：删了没暂存，索引里还跟踪着两份
        code_check, output_check = captured(False, starting_commit)
        checked += 1
        if code_check != 0 or "2 份磁盘上已删、索引里还跟踪着" not in output_check:
            failures.append((f"磁盘上已删、索引里还跟踪着的不该算待归档：--check 应当退 0 并报「2 份磁盘上已删、索引里还跟踪着」，实际退 {code_check}：{output_check[-400:]}",
                             "看 run() 里 gone 那一步：past_round_files() 的结果要把磁盘上已不在的分出去"))
        code_again, output_again = captured(True, starting_commit)
        checked += 1
        if isinstance(code_again, Exception):
            failures.append((f"第二次 --apply 抛了 {type(code_again).__name__}：{code_again}", "看 run() 里 os.remove 之前有没有把磁盘上已删的分出去"))
        elif code_again != 0 or "2 份磁盘上已删、索引里还跟踪着" not in output_again:
            failures.append((f"第二次 --apply 应当退 0 并报「2 份磁盘上已删、索引里还跟踪着」，实际退 {code_again}：{output_again[-400:]}",
                             "看 report_gone() 与 run() 里 gone 那一步"))
        page = open(os.path.join(work, ".claude/kb/page.md"), encoding="utf-8").read()
        checked += 1
        if page.count("git show") != 2:
            failures.append((f"两趟 --apply 之后两处引用各改成一次 git show 写法（不套娃），实际：{page.strip()}", "看 rewrite_links() 的 GIT_SHOW_BEFORE"))
    for failure, howto in failures:
        print(f"  ✗ 自检失败：{failure}")                 # gate-lint:detail
        print(f"     → 怎么办：{howto}；ARCHIVE_PAST_ROUNDS_BREAK 设着的话这里本来就该红")
    if failures:
        return 1
    print(f"  ✓ 自检（可重跑与排除项，查了 {checked} 格）：<路径>:<串> 写法的排除项看冒号前的路径、--check 不把磁盘上已删而索引里还跟踪着的算待归档、"
          "第二次 --apply 不抛异常并报数、引用不套娃")
    return 0


def selftest_history_carriers_and_full_paths():
    """两格：① --apply 改写引用时跳过登记在 .claude/history-carriers 的文件与登记项的变更史（逐字不变），没登记的照改；
    ② 「还被代码当输入」按完整路径认：三个腿目录各有一份 src/main.rs，只有一份被代码用完整路径写出来，只留那一份（另一份同名的只在代码里出现文件名，照删）。"""
    import contextlib
    import io
    import tempfile
    failures, checked = [], 0
    with tempfile.TemporaryDirectory() as work:
        def write(relative, text):
            path = os.path.join(work, relative)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            open(path, "w", encoding="utf-8").write(text)

        def read(relative):
            return open(os.path.join(work, relative), encoding="utf-8").read()

        git("init", "-q", root=work)
        subprocess.run(["git", "config", "user.email", "t@t"], cwd=work, check=True)
        subprocess.run(["git", "config", "user.name", "t"], cwd=work, check=True)
        reference = "产物 `research/results/e50-old.out`，腿 `research/prompts/leg-b/src/main.rs`\n"
        for leg in ("leg-a", "leg-b", "leg-c"):
            write("research/prompts/%s/src/main.rs" % leg, "fn main() {}\n")
        write("research/results/e50-old.out", "上一轮\n")
        write("scripts/experiment-apparatus/runner.py", 'LEG = "research/prompts/leg-a/src/main.rs"\n# 依据 research/prompts/leg-c/src/main.rs\n')
        write("scripts/experiment-apparatus/other.py", 'NAME = "main.rs"\n')
        write(".claude/history-carriers", "# 样本：哪些 kb 文件带历史版本节\n.claude/kb/experiments/  # 实验页\n")
        write(".claude/kb/experiments/01-x.md", reference + "\n## 历史版本\n")
        write(".claude/kb/experiments-history.md", "## E1（样本）\n\n### 2026-10-15\n\n- " + reference)
        write(".claude/kb/other.md", reference)
        write("records/2026-10-15-样本.md", reference)
        git("add", "-A", root=work)
        subprocess.run(["git", "commit", "-qm", "上一轮"], cwd=work, check=True)
        carrier_texts = {relative: read(relative) for relative in (".claude/kb/experiments/01-x.md", ".claude/kb/experiments-history.md")}
        buffer = io.StringIO()
        os.environ["GATE_BASE"] = "HEAD"
        try:
            with contextlib.redirect_stdout(buffer):
                code = run(work, True)
        finally:
            os.environ.pop("GATE_BASE", None)
        output = buffer.getvalue()
        checked += 1
        if code != 0:
            failures.append((f"--apply 应当退 0，实际退 {code}：{output[-500:]}", "看 run() 的输出里哪一步判了红"))
        checked += 1
        changed_carriers = [relative for relative, text in carrier_texts.items() if read(relative) != text]
        if changed_carriers:
            failures.append((f"登记在 .claude/history-carriers 的文件与它们的变更史，--apply 之后要逐字不变，实际改了：{changed_carriers}",
                             "看 rewrite_links() 开头的 history_text_guard()：登记项的目录前缀、逐字的文件、旁边的 <名>-history.md 都要跳过"))
        checked += 1
        if "没改 2 份登记在 .claude/history-carriers 的文件" not in output:
            failures.append((f"--apply 应当报「没改 2 份登记在 .claude/history-carriers 的文件…」，输出：{output[-500:]}", "看 report_history_kept() 与 rewrite_links() 的返回值"))
        checked += 1
        for relative in (".claude/kb/other.md", "records/2026-10-15-样本.md"):
            if not re.search(r"`git show [0-9a-f]{12}:research/results/e50-old\.out`", read(relative)):
                failures.append((f"没登记的 {relative} 里指向被删产物的引用应当照改成 git show 写法：{read(relative).strip()}",
                                 "看 history_text_guard() 有没有把没登记的也跳过了"))
        checked += 1
        if not os.path.exists(os.path.join(work, "research/prompts/leg-a/src/main.rs")):
            failures.append(("被代码用完整路径 research/prompts/leg-a/src/main.rs 引到的那一份该留下", "看 still_an_input() 的 written_as_path()"))
        checked += 1
        gone_wrongly = [leg for leg in ("leg-b", "leg-c") if os.path.exists(os.path.join(work, "research/prompts/%s/src/main.rs" % leg))]
        if gone_wrongly:
            failures.append((f"只有一个被完整路径引到，另一个同名的该删（代码里只有文件名 main.rs 与注释里的路径不算引到），实际还留着：{gone_wrongly}",
                             "看 still_an_input()：按候选的完整相对路径认，不按 os.path.basename"))
    for failure, howto in failures:
        print(f"  ✗ 自检失败：{failure}")                 # gate-lint:detail
        print(f"     → 怎么办：{howto}；ARCHIVE_PAST_ROUNDS_BREAK=history-carriers-ignored 或 still-input-by-basename 设着的话这里本来就该红")
    if failures:
        return 1
    print(f"  ✓ 自检（变更史与完整路径，查了 {checked} 格）：登记在 .claude/history-carriers 的文件与它们的变更史 --apply 之后逐字不变、没登记的照改；"
          "「还被代码当输入」按完整路径认，同名的 src/main.rs 只留被完整路径引到的那一份")
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
                # 排除项指 research/ 之外：research/ 下指空的由 dangling_registry_namings 报（判红），这一格只证 stale_exclusions 跑到
                write(".claude/sample-exclude", "docs/e7-gone-directory/  # 样本：删完指不到\n")
            else:
                write("scripts/experiment-apparatus/gone.rs", 'const G: &str = include_str!("../../research/results/e9-gone.out");\n')
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
