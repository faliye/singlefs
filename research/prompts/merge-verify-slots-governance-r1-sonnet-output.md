# 合入后验证槽位与拆分登记行：正推腿报告（治理文件轮，第一轮）

日期 2026-11-20。判格 2、3、4；格 1、5 归攻方腿，这里不判。方案按 `crates/` 今天的实现来谈：这一批没改 `crates/`，被判的是槽位脚本、规则与技能正文、六张排除表、搬迁登记、六份 README、一处工具测试；测试那一处读的是 `scripts/agents/` 的实现（`agent-watch.py` 与拆出的三份模块）。

## 开头：模型、复跑、哈希

- 模型目录 `research/prompts/merge-verify-slots-governance-r1-sonnet-model/`。都是只读脚本：前四份手算登记表与搬迁登记，第五份起仓副本逐项弄坏 `progress_note`，量用例红不红。
- 复跑（整条见下面「要主 agent 跑的命令」第 1 条）：`rerun-from-a-clean-work-directory.sh` 先删那一个运行目录，再调同一个模型目录的 `tools/rerun.sh`。
- 每个文件的 sha256（`sha256sum` 现算）：

| 文件 | sha256 |
|---|---|
| `tools/check_all_rows.py` | `0effb4b7d1ea0b3bbbc5ffdcfd145b2bfa9b34005879a1c45810288ff0197028` |
| `tools/check_moves.py` | `14ce1afb9e1e99edce348de5ee429a46f019babb85db08cfdabfd04575351cb5` |
| `tools/check_tables.py` | `65571c58278966414705aaffcfa6a4af46c131caf521fd5fe431215bc0886dda` |
| `tools/growth.py` | `3f92b98ef05cd8bf46db87addeee264299fee83c284c6030d74aedfbf71582a3` |
| `tools/progress_note_mutations.py` | `9becc3cf9ed561373f5f9bd351638fec461693a0f597da949b74cf18f835825c` |
| `tools/rerun.sh` | `5c33305ac01c1969c6b7ecd5d24eb92e310a85f7cb5130c6e685ecfd1430aef8` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |

- 我自己跑过的只有只读命令与五份脚本里读文件的那几份（它们只读仓、只写草稿目录，不起项目代码）；起仓副本、起测试的那一份（`progress_note_mutations.py`）我只跑了 `--dry-run`（每一项的旧串在副本里恰好命中、改动非空，退 0），真跑没有，格 4 里凡是「红不红」的结局都是推的，待主 agent 跑。
- 探针的内存与时间：没量。推的：一趟 `minimal-copy.py` 建副本拷出约 111 MiB（我建过一次，`du -sh` 记 111M，已删），之后 10 次起同一个 `unittest` 用例，每次是一个 python 进程、几秒、百 MiB 以内；这些是推的数。
- 被判文件的开工快照核对：`sha256sum -c research/prompts/merge-verify-slots-governance-r1-snapshot/judged-sha256.txt` 只有 `scripts/gate/merge-and-verify.sh` 一行 FAILED（别的 19 份 OK）；这一份在开工之后被别的会话改过（背景材料「分工」前已写明在补开关点名），所以下面引它的行号是此刻的行号，不是快照那一刻的。

## 问题 2：规则与技能正文

### 2a `.claude/rules/test-script-discipline.md` 第 65 行新句

新句整行（`git diff HEAD` 看改的就这一行）：

> 一趟只编译一次：这一趟的几个项共用这一趟的一份构建。不同的树不共用编译目录（经链接或环境变量指到别的树的编译目录同样算共用）；同一棵树先后几趟可以接着用它那一份增量编。按键存结论的判定库与外部依赖的预编库不是编译目录，照留。

（行在 `.claude/rules/test-script-discipline.md:65`。）

对 `.claude/singlefs-ai-sop/rules/rules-discipline.md`（只写怎么做）逐条：

- 「## 1. 正文只写四样」（该文件第 9 行）：新句是射程与例外（「不同的树不共用」）加一个允许（「可以接着用」），没有论证、日期、旧值、改动经过；「## 7. 正文不许用位置指代，也不许自称」（第 48 行）：没有「本节」类自称。判：成立。
- 「## 2. 判据留下，论证搬走」（第 18 行）：照它能判出该不该做的才是判据。「不同的树」读得出：旧句「每一趟新建、跑完删」删了，换成「树」这一个量；但「同一棵树」没有判据——规则正文没写什么算同一棵、槽里每趟打不同补丁拷进来的快照算不算。这一点规则单独推不出，要靠脚本文件头与用户原话补。判：对「怎么做」成立，对「同一棵树指什么」推不出。

与 `scripts/gate/merge-and-verify.sh` 文件头「槽位」的行为是不是一回事：

- 脚本文件头第 80 行自己把话说死：
  > 槽位（快照放在固定路径的槽里、一槽一个编译目录接着用：.claude/rules/test-script-discipline.md「编译与算法都要快」只管不同的树不共用编译目录，一槽是一棵树）：
- 拷法在第 100 行：「拷进槽（refresh_slot_snapshot）」，`rsync --recursive --links --checksum --no-times`，内容变了的文件修改时刻是此刻、没变的留旧的。与第 66 行「拷完把内容变了的文件的修改时刻改到此刻」同一件事，不冲突。
- 用户原话（`records/2026-11-13-全量提交门禁前收尾调度.md:1507-1508`）：合入后验证「每一趟都是『主工作区加这一批补丁』，前后几趟可以接着用同一个编译目录，只要能保证改过内容的文件一定被判成要重编」；第 1508 行 ③ 写「规矩 `.claude/rules/test-script-discipline.md:65` 那一句改成只管「不同的树不共用编译目录」」。
- 所以「同一棵树」在脚本与用户那里是「同一个固定路径、按内容拷进来的那一份」，槽里每一趟的快照算同一棵；规则句本身没写这一读法。判：与脚本行为一致（成立），条件是按上面这样读。
- 两处小偏差，都不构成冲突：① 第 1508 行 ③ 说「只管」不同的树不共用，新句多加了半句允许（「可以接着用」）；方向与脚本一致，超出了用户原话的字面。② 规则句与第 66 行合起来才有「改过的必重编」的保证（第 66 行：核改过的每个包有 `Compiling` 行），脚本里对应的是 `slot-changed-sources` 加重编核对；这一条的灵敏度归攻方腿格 1。

反例输入（会让这一句出歧义的读法）：把「树」读成「同一个提交」的读者。槽里趟 N 与趟 N+1 打不同的补丁、主工作区中间还可能变过，按这一读法它们是两棵树，规则要求各给编译目录——与脚本「一槽一个编译目录接着用」冲突。什么现象会推翻我的「成立」：规则审读者（或 `scripts/rules-lint.sh` 之外的 review）对这一句给出「树 = 提交」的读法且用户认；或者用户说「按内容拷进来的新快照不算同一棵」。

### 2b `.claude/skills/merge-verify/SKILL.md` 第 22、23、43 行放槽的写法与脚本收尾那一行

- SKILL 第 23 行：
  > 槽里的放槽（`rm -f -- <槽>/kept <槽>/.owner <槽>/.deliverables`，快照与编译目录留着，下一趟占到它就接着编）
- 脚本收尾打的那一行（`scripts/gate/merge-and-verify-snapshot-lib.sh:70`）：
  > echo "  → 用完放槽：rm -f -- '$SLOT_DIRECTORY/kept' '$SLOT_DIRECTORY/.owner' '$SLOT_DIRECTORY/.deliverables'（快照与编译目录留给下一趟接着编；要腾盘另 rm -rf -- '$SLOT_DIRECTORY'，这一台的编译目录在下一趟占槽时跟着删）"
- 合进主工作区那一段末尾（`merge-and-verify-snapshot-lib.sh:330`）：
  > SNAPSHOT_RELEASE_TEXT="放这一趟留着的槽：rm -f -- '$SLOT_DIRECTORY/kept' '$SLOT_DIRECTORY/.owner' '$SLOT_DIRECTORY/.deliverables'（快照与编译目录留给下一趟接着编）"
  它在 `scripts/gate/merge-and-verify-items-lib.sh:865` 经 `${SNAPSHOT_RELEASE_TEXT:-删这一趟留下的快照 $snapshot}` 打进「合完」那一行。
- 三处都是同一条命令（三个路径、`rm -f --`），只差引号与占位写法。第 43 行写的是「照『合进主工作区』那一段末尾放这一趟留着的槽」，指到的就是上面第三处，命令同一条。判：成立。
- SKILL 第 22 行说路径「在『拷好快照』那一行与日志目录里的 `slot`」：`scripts/gate/merge-and-verify.sh:2437` 打「拷好快照：…」，自证第 1087 行读 `$scratch/logs-slot-$1/slot`，两处对得上。「打了『这一趟不占槽』的那一趟照旧在日志目录的 `repo/`」：与文件头第 80–113 行「没占到槽的打一行『这一趟不占槽：<为什么>』，照一趟一份那一种跑」一致。
- 两点不构成冲突、记一笔：① 脚本第 70 行多一句「要腾盘另 rm -rf -- 槽」，SKILL 没抄，只写了放槽；② SKILL 第 23 行的「被下一趟替掉的那一趟的快照」措辞是旧写法的残影：槽位下一趟不替掉上一趟的快照，占的是另一个槽（留着的槽别的趟不占，文件头第 109 行），它要表达的「交下一趟之前先放槽」是对的。

什么现象会推翻：脚本哪天改了收尾那一行（比如多删 `.owner` 之外的文件）而 SKILL 没跟。自证只钉住这一行的开头：`scripts/gate/merge-and-verify-selftest-lib.sh:1120` 判输出里有「→ 用完放槽：rm -f -- '<槽>/kept'」，后面的 `.owner`、`.deliverables` 两个路径变了自证不红，所以 SKILL 与脚本之间这一半没有机器守着。

### 2c 「已知还在说旧规矩的几处」逐处：该改还是照留

我重跑了主 agent 的 grep，范围加宽到仓里除 `research/`、`records/`、`.git` 之外的全部文件：`grep -rnF '编译目录归这一趟' . --exclude-dir=research --exclude-dir=records --exclude-dir=.git --exclude-dir=target` 与 `grep -rnF '每一趟新建、跑完删' scripts task-server .claude`。命中比背景材料列的多（背景只列了五处）。判法：说的是那份脚本自己的做法，照留；在引已改的规矩、或说已改的服务做法，要改。

| 位置 | 判 | 依据 |
|---|---|---|
| `scripts/crash/crash-library-totals.sh:280` | 照留 | 句子「读库程序要编，编译目录归这一趟，不照仓根的 target 编」是这份脚本自己的拒绝条件（没有 `CARGO_TARGET_DIR` 就退 2），没引规则文件。 |
| `scripts/crash/crash-library-totals.sh:281` | 该改 | 同一个出路句写「（服务给编译目录、任务结束删）」，说的是任务服务的旧做法：`task-server/task_server_lib.py:264` 写同一棵树先后交的任务接着用那一组编译目录，`:287` 写「编译目录归树、不归任务」。这一句不是这一批的规则句，是任务服务按树给编译目录那一批留下的尾巴，归那一批。 |
| `scripts/mutation/prove_red_selftest.py:544`、`:570`、`:575`（另 `:601`、`:1077`、`scripts/mutation/prove_red_core.py:275`、`:365`、`:770`、`scripts/mutation/prove-red.sh:116`–`:122`、`:284`） | 照留 | 都是「文件头『判法与次序』的『编译目录归这一趟』」这一节自己的名字与自证格名：证红脚本每一趟自己建、跑完删编译目录，是它的做法；新规则句允许接着用、不要求每趟新建，两者不冲突。 |
| `scripts/mutation/prove-red.sh:232` | 该改 | 同一节的文件头把「每一趟新建、跑完删」记到规则名下：`编译目录归这一趟（.claude/rules/test-script-discipline.md「编译与算法都要快」：每一趟新建、跑完删，不跨趟、跨副本、跨脚本共用）`。那一条规则现在写的是 `test-script-discipline.md:65` 的新句，没有这一串；括号里的出处与引文现在对不上（背景材料没列这一处）。改法：括号里只留脚本自己的做法，或改引新句。 |
| `scripts/hosts/multi-host-run.sh:380` | 照留 | 在弄坏开关 `peer-build-not-kept` 的说明里：`带 --keep-build-directory 也照删这一趟的本地目录（第二台每一趟新建、跑完删）`，括号讲开关打开后的世界；平常路径是留着（同一行下一句「留着 --keep-build-directory 跑完留着这一趟的编译目录」）。 |
| `task-server/task_server_lib.py:1159` | 照留 | 同样是弄坏开关 `peer-retry-build-directory-fresh` 的说明，写的是打开开关后退回「第二台每一趟新建、跑完删」；平常路径在同文件 `:274`–`:275`（派到第二台的尝试带 `--keep-build-directory`、接着用组里那一个）。 |
| `scripts/crash/crash-amplification.sh:746`、`scripts/hosts/run-build-directory.sh:2`、`.claude/kb/tooling.md:222`（`replay.sh`）、`scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh:293` | 照留（转述） | 各自讲自己的运行目录或编译目录每趟新建、跑完删，是脚本自己的做法；`e165…sh:293` 说的是状态目录，不是编译目录。 |
| `.claude/agent-common.md:37`、`.claude/main-agent.md:34` | 该改（推的） | 两句通用句：`**用完就删**：副本、编译目录与运行目录里的中转库…用完就删，不留给以后的趟或别的任务`（`agent-common.md:37`）；`**用完就删**：本会话建的仓副本、编译目录与运行目录里的中转库，哪一步用完就在那一步删…不留给以后的趟复用`（`main-agent.md:34`）。合入后验证的槽是主 agent 的任务产物：快照与这一台的编译目录在放槽之后「留给下一趟接着编」（`merge-and-verify-snapshot-lib.sh:70`），字面上就是「留给以后的趟复用」。完工清理闸放得过（槽放掉之后 `.owner` 已删，`.claude/hooks/completion-cleanup-guard.sh:37` 只认带本会话 `.owner` 的层），所以现状不红；规则正文没给槽留例外，读的人会二选一。主 agent 判：加一句例外，或在 SKILL 写明槽不适用「用完就删」。 |

反例输入（推翻「该改」）：用户说「槽是脚本自己的资产、不归主 agent 的『本会话建的』」，那两句通用句不用动。

## 问题 3：登记行指得对不对

### 3a 六张排除表（`git diff HEAD -- <表>` 与现场表比）

方法：对每张表，取相对 HEAD 新增的行（`tools/check_tables.py`），按「路径:锚串」数锚串在路径文件里命中几行；再整张表数一遍（`tools/check_all_rows.py`）；再把新增行按（锚串、理由）与 HEAD 的行对，分出「只换路径」与「新登记」（`tools/growth.py`）。命中要求：`heavy-thread-exempt`、`repository-copy-closure-exempt`、`hardcoded-quota-exempt` 三张的文件头写「那份文件里含锚串的被判行都放行」「锚串一处被判行都不命中，判红」，所以至少 1 行；`term-rename-exempt` 的行内锚要求恰好 1 行（文件头第 4 行）。「被判行」是门禁格自己扫出的行，我只数子串命中行数，扫得出哪几行归格自己判，那一半待主 agent 在拆分族认法换上之后跑格（见「要主 agent 跑的命令」）。

| 表 | 现场总行数 | 相对 HEAD 新增 | 其中只换路径 | 新登记 | 命中 0 的行 |
|---|---|---|---|---|---|
| `.claude/heavy-thread-exempt` | 54 | 32 | 22 | 10 | 0 |
| `.claude/gate.d/repository-copy-closure-exempt.tsv` | 46 | 25 | 15（另 2 行理由改字） | 8 | 1（第 44 行） |
| `.claude/gate.d/hardcoded-quota-exempt.tsv` | 7 | 6 | 2 | 4 | 0 |
| `.claude/memory-bounded-parallelism.tsv` | 8 | 4 | 3（含 1 行理由重写） | 1 | 入口都在 |
| `.claude/term-rename-exempt` | 334 | 12 | 12 | 0 | 12 行都对（整表 1 行不对，见下） |

（`memory-bounded-parallelism.tsv` 的「命中」是入口在不在：写成路径的要是一份文件，`cargo:<包>:<目标>` 要在 `crates/*/Cargo.toml` 里有包、包里有目标，判法在 `.claude/gate.d/lib/code-tooling-source-scan-cells.sh:1080`。）

只换路径的行（共 54 行，含 `memory-bounded-parallelism` 3、`term-rename-exempt` 12）：锚串都在新路径文件里命中 1 行或更多，理由原样成立；其中三处值得点名：

1. `heavy-thread-exempt` 第 10 行 `scripts/experiment-apparatus/e162_crash_verdict_block_store/timed_writes_and_segment_runs.rs:available_parallelism`：命中 2 行，理由「只把本机核数打进 processors / config 行，不起线程」两行都成立（子串命中，我没逐行读第二处）。
2. 同表第 18 行与第 17 行：同一个锚串 `"nproc"` 先后登在 `scripts/gate/admission_selftest_cells.py` 与 `scripts/gate/admission_base.py`，原先一份 `admission.py` 的一行拆成两行，各命中 1 行，对。
3. `term-rename-exempt` 九行整份豁免改成目录写法（第 50–54、72、74、232、233 行）：文件头第 1 行允许「<相对路径或目录>」；九个目录都在（`exists=True dir=True`）。理由「实验装置：源码 sha256 登记在实验页或跑前登记，印出来的串进产物」对目录整体仍成立：目录里的子模块也是装置源码。三条行内锚（第 285、286、330 行）各命中恰好 1 行。

### 3b 不对的行与「理由」对不上的行

1. `.claude/gate.d/repository-copy-closure-exempt.tsv:44`，锚串命中 0 行（`tools/check_all_rows.py` 数这张表整表，出的唯一一行）。这一行是新登记（HEAD 里没有），不属「只换路径」。行里写的锚串：
   > SHARED_INPUTS = ("Cargo.toml", "Cargo.lock", ".cargo", "rust-toolchain"
   而它指的 `.claude/gate.d/lib/code-tooling-unit-verdict-and-entry-cells.sh` 里同名常量在第 747 行是：
   > SHARED_INPUTS = ("Cargo.toml", ".claude/gate.d/stage-inputs.tsv",
   前几项就对不上，门禁格会按「锚串一处被判行都不命中」判红。
2. `.claude/gate.d/repository-copy-closure-exempt.tsv:41`：路径与锚串命中 1 行，但理由指错了文件。理由写：
   > 与 lib_function_units.py 的 build_unit_graph 一起改
   `build_unit_graph` 现在定义在 `scripts/gate/lib_function_units_unit_graph.py:106`（`def build_unit_graph(repository, source, thread_registry, lock_file, cache, directory, program):`），`grep -rn 'def build_unit_graph' scripts .claude` 只命中那一处。同一张表第 21 行的理由只写「build_unit_graph」、没带文件名，不受影响。
3. `.claude/memory-bounded-parallelism.tsv:10`（入口 `.claude/gate.d/code-tooling.sh`，新写的理由）：入口在，但理由里新加的半句与代码对不上。理由写：
   > break-switch-replay 与 stage-break-switch-replay 两格按内存上限与每趟内存算同时跑几条
   而代码里这两格平常按线程数并行：`.claude/gate.d/lib/code-tooling-break-switch-replay-cells.sh:128` 写「Admission 不带内存份额，只数在跑的趟」；按内存算的 `jobs_for` 只在弄坏开关下走：`.claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh:258` 写「SAFE_SHARE_PERCENT = 75 …（只在弄坏开关 replay-jobs-by-memory、cells-alone-jobs-by-memory 下）」，`code-tooling-break-switch-replay-cells.sh:146` 写「弄坏开关 CODE_TOOLING_BREAK=replay-jobs-by-memory：并行条数照内存上限算」。这一行要登记是对的（门禁反方向会把 `jobs_for` 那几行认成「按内存定并发的写法」，入口不登记判红），但理由该写成弄坏开关那一支，不是常态做法。同一行前半句（`research-script-selftests` 格、`SELFTEST_RUNNERS_MEMORY_BOUND`）没动，不在这里判。
4. `.claude/term-rename-exempt:100`（`.claude/agents/implementation-writer.md:「列一节「受影响的层 0 流与崩溃枚举用」`）命中 0 行：与背景材料「HEAD 里那一句也不在」一致，不是这一批改的，也不是拆分带出的；这里只记一笔，不算这一批的不对。
5. `.claude/gate.d/repository-copy-closure-exempt.tsv:13`（e171 `row_driver.py`）：背景材料说拆分前就对不上、没动。现场此刻命中 1 行，行已改成与源码同一串（锚串里带 `*CONFIGURATION_CHECK_ENTRIES)`，`git show HEAD:…row_driver.py | grep -c CONFIGURATION_CHECK_ENTRIES` 数出 5，HEAD 里的那一行确实少了这一段）：这一行现在是对的，背景材料那一句已过时。理由「E171 冻结装置：线道树同步的范围照跑前登记写死，不改」仍成立：`git diff HEAD` 看 `row_driver.py` 只改了 3 行，都是包名从 `singlefs-verification` 换成 `singlefs-implementation-experiments`。

不在「改路径」之内、但落在这些表的 `git diff HEAD` 里的新登记行：`heavy-thread-exempt` 第 63–68、71–74 行（共 10 行）、`repository-copy-closure-exempt.tsv` 第 13、44–46、48–51 行（8 行；第 13 行是锚串随源码改字，HEAD 里那一串对不上了）、`hardcoded-quota-exempt.tsv` 第 16–19 行（4 行）。逐行锚串命中 1 行或 2 行（第 44 行除外，见上；第 48 行命中 2 行），理由都说得通（门禁阶段的 `STAGE_CELLS_JOBS` 缺省取登记值、自证造的假值、只读不拷）。它们对「只缩不涨」是涨，要不要登要看各格怎么判，不是这一轮判的「改过的行指得对不对」。

### 3c `.claude/preflight-exclude`

方法：删的 21 行、新增的 14 行逐行核（`git diff HEAD -U0`）；再读判法 `.claude/singlefs-ai-sop/scripts/preflight-lint.py` 的文件头（第 15 行：「每个目录只看它自己这一层；子目录要判就另登记一行（嵌套阶段是这一条唯一的例外）」）与 `.claude/preflight-dirs`（第 2 行起登记三个目录，各「只算它自己那一层」）。

- 删的 21 行里 17 行是真删（9 行 `scripts/experiment-apparatus/e1xx_….rs`，8 行 `crates/singlefs-verification/src/bin/…rs`），4 行是换路径（`e142_…_write_dump` 两条、`new_pool_file_creation_region_bytes`、`deletion_write_dump`，新路径 `crates/singlefs-implementation-experiments/src/bin/<名>.rs` 都还是一份 `.rs` 文件、在被扫的那一层，新行 `exists=True`）。
- 17 行真删对：它们指的 `.rs` 文件现在不在（`git diff` 逐个 `test -f` 为 n），入口已成 `<名>/main.rs` 目录（9 行脚本装置与 `new_pool_file_creation_on_device`、`e165_replay_bounded_enumeration` 直接 `test -f <名>/main.rs` 为 Y，其余 6 行 crates 装置各有 `path-moves.tsv` 的搬迁行，实据过，见 3d），目录不在被扫的那一层，不需要排除；留着只会让「指向不存在的路径」判红。`new_pool_file_creation_on_device` 那一行的理由「在虚机的 busybox initramfs 里跑的静态二进制，那里没有 python3，preflight 判不了条件」随入口一起不再被扫到；这是判法只认一层带来的覆盖变化，不是理由不成立。
- 反方向核：被扫的两个 `src/bin` 层与 `scripts/experiment-apparatus` 层里，`.rs` 文件要么在表里、要么自己带 `preflight`。我对 `crates/singlefs-verification/src/bin/*.rs`、`crates/singlefs-implementation-experiments/src/bin/*.rs` 逐份数：7 份 `.rs` 里 6 份在表里、1 份（`e184_regular_layer_cost_counts_write_dump.rs`）自己带 `preflight`（文件里 `use write_dump::{…, preflight, …}`），没有漏登的。
- 新登的 10 行「同目录 source 的函数库、没有自己的入口」：
  - 八份 `replay-*.sh`（`.claude/preflight-exclude:154`–`:161`）：`scripts/experiment-apparatus/replay.sh` 在第 192、264、266、328、671、2355、2356、2541 行逐个 `source "$REPLAY_LIBRARY_DIRECTORY/replay-….sh"`，八份都有（例：`scripts/experiment-apparatus/replay.sh:2541` `source "$REPLAY_LIBRARY_DIRECTORY/replay-dispatch.sh"`）；八份都没有执行位、没有 `preflight` 调用。成立。
  - 两份 e165 模块（`.claude/preflight-exclude:162`、`:163`）：`scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh:331` 起 `from e165_replay_mutation_kill_bounds_driver import`（`:334` 同样导入 selftest 那份）；两份里 `__name__` 只出现在 `function.__name__` 取函数名（没有 `if __name__ == "__main__"`），没有入口。成立。
  - 理由句里「被 `scripts/experiment-apparatus/replay.sh` 在同目录 source」「被 `…e165-replay-mutation-kill-bounds.sh` 载入」都核到了。
- 没做的：`scripts/experiment-apparatus/e203_admission_scan_cost_counters_*.py` 四份拆出的模块没有 `admission:` 文件头、表里也没有行（`git ls-tree HEAD` 里入口 `e203_admission_scan_cost_counters.py` 在 HEAD 就没登）；这是拆分前就有的红（背景材料「剩 66 处」），我没逐份核它们红不红，只记「这一批没给它们登行」。

判：17 行删对、4 行换路径对、10 行新登对；不对的行 0。共核 31 行（17 真删、4 换路径、10 新登）。

### 3d `.claude/gate.d/path-moves.tsv`（照 `.claude/gate.d/lib-lag-baseline.py` 文件头「搬迁登记」）

方法：`tools/check_moves.py` 对现场表相对 HEAD 新增的每一行核：写法（四列、日期、`records/<文件>.md`）、出处文件在、此刻生效（`lib-lag-baseline.py:64`「新行必须生效」：基准里旧前缀下有文件、新前缀下没有文件）、搬迁实据（第 92 行「此刻生效的每一行要有搬迁实据」：旧前缀下每份文件在新前缀同相对路径上都在、被跟踪或未被忽略）、旧前缀此刻不在盘上；再核生效的行之间串不串（第 91 行「任一行的旧前缀或新前缀不许等于、包含或落在另一行…下面」）。基准取 HEAD：现查 `git rev-parse HEAD` 与 `git merge-base HEAD @{u}` 都是 `4319aa2b035b2d8238d64aeb68b3ceb7674cc723`，所以 gate 基准就是 HEAD。

结果（`tools/check_moves.py` 的输出）：

```
新表内容行 754，HEAD 里没有的（新增或改过） 86
逐行核：86 行，有问题 14 行；生效 72 行
生效行之间串的 (行,行,前缀种类) 0 处
```

- 现场相对 HEAD 新增 86 行，不是背景材料说的 66 行。分来源（出处列）：`records/2026-11-19-crates代码拆分.md` 49 行（`X.rs → X/main.rs`，含两行样本档）、`records/2026-11-19-产品GPU包测试与E202装置归位.md` 23 行（其中 17 行是被拆分再指到 `<名>/main.rs` 的那 17 行，6 行是 `.rs → .rs` 与一行目录的包搬迁）、`records/2026-11-17-subagent账本清欠调度记录.md` 14 行（`.claude/hooks/` 下连字符改下划线）。
- 这一批的 66 行（49 + 17）：66 行都生效、实据全过、旧前缀此刻都不在盘上、新前缀在基准里没有文件、出处文件在；生效的 72 行之间一对串的也没有。判：满足。另外 6 行包搬迁也满足，不在这 66 行里。
- 不满足的 14 行（第 746–759 行，`.claude/hooks/write-guard-decisions.py → .claude/hooks/write_guard_decisions.py` 起，到 `.claude/hooks/bash-command-detector-selftest-entries-and-main.py → …_main.py`）：旧前缀在 HEAD 里一个文件都没有（这些钩子模块是这一轮才新建的，HEAD 里没有连字符那一版），照第 64 行判红「这一行不是一次搬迁」。它们不是这一批改的，是梳理钩子的会话加的，归它；我只报这 14 行进了被判的 `path-moves.tsv`。
- 一点要点明、不算不成立：`exact_rename`（`.claude/gate.d/lib-lag-baseline.py:545`）写「单个文件登记（old_path 正是旧前缀）的，文件名（最后一段）还要不变」。`X.rs → X/main.rs` 这 66 行最后一段从 `X.rs` 变成 `main.rs`，所以对「改动范围剔除只差搬迁」那一层（`move_source`、`move_target`）这 66 行交 `None`，不起作用；起作用的是登记表自己的判（实据与不串）和 `baseline_text` 把基准表的旧前缀换成新前缀（`rewrite_moved_paths`）。这与文件头第 100 行「④ 换了实现的不登记：落点逐份在、被跟踪就过实据，换了内容的那几份文件与新增行照算改动」一致：拆出来的 `X/main.rs` 内容本来就不同于 `X.rs`。
- 依赖（推的，没跑）：六张表里把锚串改到拆出的部分文件的行（如 `heavy-thread-exempt` 第 10 行指 `…/e162_crash_verdict_block_store/timed_writes_and_segment_runs.rs`），基准里没有这个「路径:锚串」；搬迁登记只能把基准行换到 `…/main.rs`，换不到部分文件。`scripts/gate/lib_lag_split_family.py:19` 写「搬迁登记一个旧前缀只能换一个新前缀…一份旧文件拆成几份新文件它换算不了，所以由比法这里认族」，那一份此刻在索引里（`A  scripts/gate/lib_lag_split_family.py`），背景材料说调用方还没接到这几张表。所以这些行「路径指对了」，但「只缩不涨」那一格会红到族认法接上为止（与背景材料「已转给管这几格的会话」一致）。

反例输入（推翻「66 行满足」）：在 `4319aa2b0` 之后有人把 `crates/singlefs-verification/src/bin/e158_root_choice_repair.rs` 这类旧路径在基准里已不存在（提交进基准）：这一行对门禁不再起作用；或旧前缀下又在盘上放回一份同名文件：实据判红。

### 3e 六份 README 新表行

- 点名的文件在不在：`git diff HEAD -U0 -- <README>` 取新增的表行，取第一列反引号里的名字，对着该 README 所在目录 `test -e`：六份共 94 个名字，89 个在；5 个「不在」是 `scripts/gate/README.md` 里 `git-hooks/` 下的 `pre-commit`、`pre-merge-commit`、`prepare-commit-msg`、`pre-applypatch`、`pre-push`（行里写的是「`git-hooks/` 下的…」，不在 README 同目录），`ls scripts/gate/git-hooks/` 五份都在。不在的：0。
- 用途对不对（抽到源头核）：
  - `scripts/agents/README.md` 新行（`agent_watch_transcripts.py`、`agent_watch_repository.py`、`agent_watch_alerts.py`）：`scripts/agents/agent-watch.py:605`–`:621` 逐个 `from … import` 并 `import *` 再导出，行里写的三块职责与 `agent-watch.py:573`–`:575` 文件头的说明一致。对。
  - `scripts/hosts/README.md` 两行（`run-with-memory-cap-selftest.sh`、`multi-host-run-selftest.sh`）：`scripts/hosts/run-with-memory-cap.sh:1593` 与 `scripts/hosts/multi-host-run.sh:1549` 在 `--selftest` 那一支 `source`，行里「只在 `--selftest` 那一支载入」对。
  - `scripts/crash/README.md` 两行：`crash-point-replay-selftest.sh` 在 `scripts/crash/crash-point-replay.sh:1665` 那一支 `source`，对；`crash-library-totals.sh` 文件头第 6 行「起一次只读读库程序 crash_library_reader keys」、用法「经 task-server/task submit -- bash scripts/crash/crash-library-totals.sh 交」与行里写的相符，对。
  - `scripts/prune-server/README.md` 一行：`prune_server_client_selftest.py:73`–`:76` 文件头写了同样三块与「导入并再导出」，对。
  - `scripts/gate/README.md`：这一批相关的三行对（`merge-and-verify.sh` 与 `merge-and-verify-slots.conf` 那一行写「按内容拷进一个槽当快照…槽数与槽的编译目录占盘上限登记在 `merge-and-verify-slots.conf`」，`merge-and-verify-snapshot-lib.sh` 等四份库那一行，`apply_writer_patch_plan.py` 那一行），文件都在。别的会话加的几十行（`lib_function_units_*.py`、`hook_swap_*.py`、`crates-split-pure-move.py` 等）只核了文件在，没逐行核用途。
  - `scripts/experiment-apparatus/README.md`：`replay-*.sh` 八份、e165 两份的行与 3c 核到的一致，对；`scripts/experiment-apparatus/README.md:7` 那一行「分成多份模块的装置各占一层子目录」后面只列了五个目录（`e139_tightened_floor/` 等），而现场有 31 个带 `main.rs` 的装置目录（`ls -d scripts/experiment-apparatus/*/main.rs` 数出 31，每个都有不止一份 `.rs`，`e10_aging_compaction` 起到 `e210_capacity_grow_reader_rules`），读起来像穷举、其实只列了五个；同一行说「单测在 `<名>/tests.rs`」，`e203_admission_scan_cost/` 里除 `tests.rs` 还有 10 份 `<名>_tests.rs`（`ls` 数出），写法不全。用途大意对、列举不全（该行没写「例如」）。
- 判：新行点名的文件 94 个名字里不在的 0；用途不对的 0；列举不全的 1 行（experiment-apparatus 第 7 行）。

## 问题 4：`WatchTests.test_progress_deleted_before_read_is_reported`

改后的用例（`scripts/agents/tests/test_agents.py:291` 起）：先 `load_script('agent-watch.py')`，再取 `alerts = sys.modules['agent_watch_alerts']`（第 294 行），把 `alerts.progress_file_of` 打成返回一个不存在的路径，调 `alerts.progress_note(SimpleNamespace(), …)`，断言结果里有「已消失或不可读」。

### 4a 读进度文件那一支被改坏时红不红（读代码推的，真跑待主 agent）

被测的函数 `scripts/agents/agent_watch_alerts.py:167` 起：第 171 行 `path = progress_file_of(transcript)`（被打补丁的那一个，在同一模块的全局名里查），第 176 行 `moment = readable_mtime(path)`（`readable_mtime` 在第 138 行，路径不存在时 `except OSError` 交 `None`），第 178 行 `if moment is None:`，第 179 行返回「；progress.md 在检查期间已消失或不可读」；第 183 行 `except OSError:`，第 184 行返回同一句。用例给的路径不存在，所以只走「第 178 行为真 → 第 179 行返回」这一条，第 180–186 行与第 183–184 行都走不到。

背景材料的两条实测与此一致：补丁打在入口模块上（`mock.patch.object(module, 'progress_file_of')`，`module` 是 `agent-watch.py`）时，补丁改的是入口命名空间里那一个名字，`progress_note` 的全局名在 `agent_watch_alerts` 里不受影响，真的 `progress_file_of(SimpleNamespace())` 经 `own_scratch_directories` 读 `transcript.dispatch_prompt`（`scripts/agents/agent_watch_alerts.py:135`），`SimpleNamespace()` 上没有这个属性，抛 `AttributeError`，与背景材料记的结局（用例 errors=1）一致；打在 `agent_watch_alerts` 上才对。

逐项弄坏的预期（`tools/progress_note_mutations.py` 写死的十项，改动都只加在副本上）：

| 项 | 弄坏什么 | 预期 | 为什么 |
|---|---|---|---|
| M0 | 不弄坏 | OK | 基线 |
| M1 | 第 179 行那一句改字 | FAIL | 走的就是这一行，断言找不到「已消失或不可读」 |
| M2 | 第 184 行那一句改字 | OK（用例看不见） | 第 183–184 行走不到 |
| M3 | 去掉 `if moment is None:`（改成 `if False:`） | ERROR | 往下 `datetime.fromtimestamp(None)` 抛 `TypeError`，不是 `OSError`，不被第 183 行接住 |
| M4 | `readable_mtime` 读不到时交 `0.0` 不交 `None` | OK（用例看不见） | 第 178 行为假，往下 `open` 抛 `OSError`，第 184 行返回同一句话 |
| M5 | 第 183 行 `except OSError` 改成 `except KeyError` | OK（用例看不见） | 走不到 |
| M6 | 第 182 行取末几行改成取头几行（平常路径） | OK（用例看不见） | 文件不在，走不到 |
| M7 | `path` 不再取 `progress_file_of` | ERROR（脚本里 FAIL 也算对得上） | 补丁不起作用，`path` 为 `None` 后第 172 行读 `own_scratch_directories(SimpleNamespace())`，抛 `AttributeError` |
| M8 | `AGENT_WATCH_BREAK=progressnote`（第 169 行） | FAIL | 返回空串 |
| N1 | `agent-watch.py` 里模块名换成 `agent_watch_alert_handlers`（文件跟着改名） | ERROR | 见 4b |

结论：这个用例钉住的是「文件在检查期间没了，告警里要写已消失或不可读」这一件事的外部表现，红在第 179 行那句话被改、第 171 行取路径被换、或第 178 行判定被去掉；读文件那一支的别的部分（`readable_mtime` 判 `None`、`except OSError`、取末几行）它看不见，M2、M4、M5、M6 四项预期照样绿。理由是两个出口说同一句话，用例分不出走了哪个。平常路径（「改于…前，末 N 行：…」）由 `agent-watch.py --selftest` 的格罩着：`scripts/agents/agent_watch_alerts.py:1349` 判「整点询问应当附上 progress.md 的末几行」，`agent-watch.py:404`–`:405` 写弄坏开关 `progressnote` 红在这一格；单元测试里没有对第 185–186 行的断言（`grep -rn '改于' scripts/agents` 只命中源码第 185 行）。这是推的；我没真跑，待主 agent 跑 `progress_note_mutations.py` 核。

什么现象会推翻：M2、M4、M5、M6 任一项实测红了（说明我读漏了一条走到那几行的路）；或 M1 实测绿。

### 4b `agent-watch.py` 换一种载入方式、模块名不叫 `agent_watch_alerts` 时

- 现在的载入：`scripts/agents/agent-watch.py:602` 把自己所在目录放进 `sys.path`，第 613 行 `from agent_watch_alerts import (`，第 621 行 `from agent_watch_alerts import *  # noqa: F401,F403`。普通 `import`，模块在 `sys.modules` 里的键就是 `agent_watch_alerts`。
- 模块名换了（上表 N1）：`load_script('agent-watch.py')` 能跑完，但第 294 行 `sys.modules['agent_watch_alerts']` 取不到，抛 `KeyError`，用例 ERROR。是报错，不是静默过。补丁的目标属性不在（`progress_file_of` 挪走）同样：`mock.patch.object` 抛 `AttributeError`。`progress_note` 挪到别的模块、`agent_watch_alerts` 里只剩再导出：`alerts.progress_note` 在原模块的全局里查 `progress_file_of`，补丁打不到，真函数收到 `SimpleNamespace()`，断言红或抛错，也不静默过。
- 唯一静默过的路：进程里已经有一个同名 `agent_watch_alerts` 模块（别人先载入的），而 `agent-watch.py` 自己载入的叫别的名字。那时用例给这个旧模块打补丁、测这个旧模块，`agent-watch.py` 自己那一份 `progress_note` 坏了也不红。现状下进程里没有第二个来源：`grep -rnE 'agent_watch_alerts|agent-watch' --include='*.py' --include='*.sh' scripts .claude/hooks .claude/gate.d` 在被测文件之外，只有 `scripts/kb/move-directory-in-use-check.py:92` 与 `scripts/hosts/clean-session-scratch.py:89` 按路径载入 `agent-watch.py`（模块名分别是 `agent_watch_for_move_check`、`agent_watch_for_clean_session_scratch`，各自再经 `agent-watch.py` 里同样的 `from agent_watch_alerts import` 载入同一个键），不是第二个同名来源；`test_agents.py` 里只有第 292 行一处 `load_script('agent-watch.py')`。所以现状下是「报错」那一支。
- 另一处改前后的差别：旧用例经 `module.progress_note`（`agent-watch.py` 命名空间里的再导出）调，钉住了「入口把 `progress_note` 再导出」；新用例直接调 `alerts.progress_note`，不再钉这一点。`grep -rn 'progress_note' scripts .claude/hooks .claude/gate.d` 在 `agent_watch_alerts.py` 之外没有别的按名字取它的调用方（只有测试与 `agent-watch.py:404` 的说明），所以现在没有实际损失；将来有人经 `agent-watch.py` 取它，用例看不见。
- 什么现象会推翻：N1 实测用例绿；或 `grep` 发现进程里有第二个来源把 `agent_watch_alerts` 放进 `sys.modules`。

## 要主 agent 跑的命令

我没有往任务服务交任何任务。下面每条一项：命令、在哪个目录起、看哪几行。

1. 复跑整份模型（四份登记表与搬迁登记的只读核对，加仓副本里逐项弄坏 `progress_note`；副本用完在脚本里删）。在仓根起：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=模型目录 tools 里的登记表与搬迁登记核对加 progress_note 逐项弄坏; 范围=1 个测试函数乘 10 项弄坏，另加 4 份只读核对; 为什么=复核本报告问题 3 与问题 4 的读数，量补丁改打之后用例对读进度文件那一支与模块名改动灵不灵敏' task-server/task submit -- bash research/prompts/merge-verify-slots-governance-r1-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/merge-verify-slots-governance-r1-snapshot /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r1-sonnet-run /home/fy5090/code/singlefs/research/prompts/merge-verify-slots-governance-r1-sonnet-model`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。运行目录名以 `-run` 收尾、在工作根下，脚本开头先删它。
   - 看哪几行：结局行 `TASK_SERVER_RESULT` 的 `type=succeeded` 与 `exit=0`；日志里 `==` 开头的六行汇总（4a 的表对应 3a 的数）、`新表内容行 754，HEAD 里没有的（新增或改过） 86` 与 `逐行核：86 行，有问题 14 行；生效 72 行` 与 `生效行之间串的 … 0 处`；逐项弄坏的十行每行末尾不带 `DIFFERENT`、末行 `收尾：两份文件的哈希与开跑时相同`、`name=progress_note_mutations exit=0`。有 `DIFFERENT` 的以实测为准，4a 的表对应行作废。
   - 注意：`check_*.py` 与 `growth.py` 读的是 `/home/fy5090/code/singlefs` 此刻的工作区与 HEAD，别的会话再改这些表，行数会变；引数前先核表的 `sha256sum` 与背景材料快照里记的是否还是同一份。
2. 模板自证（我只改了 `rerun_steps`，其余是模板原文；自证里的假仓走 `RERUN_TEMPLATE_SELFTEST_MARK` 早退那几行）。在仓根起：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=模型目录 tools/rerun.sh 的模板自证; 范围=5 格; 为什么=核我改 rerun_steps 之后模板自带的五格自证没被带坏' task-server/task submit -- bash research/prompts/merge-verify-slots-governance-r1-sonnet-model/tools/rerun.sh --selftest`
   - 看哪几行：`type=succeeded`、`exit=0`，日志里 `✓ 自检通过（查了 5 格）`。
3. 判到「登记行路径指对了、但门禁格会不会判红」那一半，要等拆分族认法接上之后再跑；每条在仓根起，一条一项，种类 `items`，看日志里 `✗` 行是否点名我在 3b 列的那几行（第一条应当点名 `repository-copy-closure-exempt.tsv` 第 44 行锚串一处都不命中）：
   - `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling 的 repository-copy-closure 格; 范围=1 格; 为什么=核 3b 点名的第 44 行锚串不命中会判红' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check repository-copy-closure`
   - `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling 的 memory-bounded-parallelism 格; 范围=1 格; 为什么=核新登的 e171 与 code-tooling.sh 行入口在、反方向罩得住' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check memory-bounded-parallelism`
   - `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling 的 heavy-thread-defaults 格; 范围=1 格; 为什么=核拆分族认法接上之后改路径的行不再判成涨' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check heavy-thread-defaults`
   - `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling 的 hardcoded-quota 格; 范围=1 格; 为什么=核拆分族认法接上之后改路径的行不再判成涨' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check hardcoded-quota`
   - `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling 的 change-range-single-source 格; 范围=1 格; 为什么=核搬迁登记 86 行里 14 行钩子改名不生效会判红' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check change-range-single-source`
   - 前四条各自的退出码与「✗」点名行读出来，与 3a、3b 的行号对一遍；第五条应当点名第 746–759 行「这一行不是一次搬迁」，并且不点名这一批的 66 行。
4. 我没有要交的写段命令，没有补丁。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 2a 第 65 行新句对 `rules-discipline.md` | 成立 | 只写射程与允许、无论证无日期；「同一棵树」没有判据（推不出槽里每趟快照算不算），靠脚本文件头第 80 行与用户原话补 |
| 2a 第 65 行新句对脚本「槽位」 | 成立（带读法条件） | 「同一棵树」读成「固定路径、按内容拷进来的那一份」才与一槽一树一致；新句比用户原话多了「可以接着用」半句 |
| 2b SKILL 第 22、23、43 行放槽写法 | 成立 | 与 `merge-and-verify-snapshot-lib.sh:70`、`:330` 的 `rm -f --` 是同一条命令；自证只钉住开头，后两个路径没有机器守 |
| 2c 旧规矩逐处 | 该改 3 处、照留 其余 | 该改：`prove-red.sh:232`（引规则的旧串）、`crash-library-totals.sh:281`（说服务任务结束删）、`agent-common.md:37` 与 `main-agent.md:34`（推的，没给槽留例外）；`multi-host-run.sh:380`、`task_server_lib.py:1159` 是弄坏开关说明，照留 |
| 3a 六张表 | 不成立（3 行不对） | 新增 79 行里：`repository-copy-closure-exempt.tsv:44` 锚串 0 命中；`:41` 理由指错文件；`memory-bounded-parallelism.tsv:10` 理由把弄坏开关那一支写成常态；其余命中 1 行或更多、term-rename 12 行行内锚都恰好 1 行 |
| 3a 背景材料「第 13 行拆分前就对不上」 | 已过时 | 现场该行命中 1 行，已改成与源码同串 |
| 3b 搬迁登记的 66 行（49 + 17） | 成立 | 生效、实据过、旧前缀不在盘、生效行间 0 处串；`X.rs → X/main.rs` 文件名变了，对「只差搬迁」那一层不起作用，只起表行换算与实据 |
| 3b 搬迁登记里别人的 14 行 | 不成立 | 第 746–759 行旧前缀在 HEAD 里没有文件，不生效，照 `lib-lag-baseline.py:64` 判红；不是这一批改的 |
| 3c `.claude/preflight-exclude` | 成立 | 17 行真删对（目录不在被扫的那一层）、4 行换路径对、10 行新登对（库真被 source、模块无入口），共核 31 行 |
| 3e 六份 README 新表行 | 成立（1 行列举不全） | 94 个名字全在，用途对到源头；`experiment-apparatus/README.md:7` 只列 5 个目录、现场 31 个 |
| 4a 读进度文件那一支被改坏 | 部分成立（推的） | 用例钉住「文件没了就报已消失或不可读」；改第 179 行句子、取路径、去掉 `moment is None` 会红，`readable_mtime`、`except OSError`、取末几行四项会绿 |
| 4b 模块名不叫 `agent_watch_alerts` | 成立（报错，不静默） | `sys.modules['agent_watch_alerts']` 取不到抛 `KeyError`；只有进程里另有同名旧模块才静默，现状没有第二个来源 |

## 没做什么

- 没判格 1、5（攻方腿的格），没核「已经量的」那一节；没替主 agent 采纳或出判决。
- 没往统一任务服务交任务；判决要引的数（登记表命中数、搬迁登记 86/72/14、逐项弄坏的结局）没有复跑，命令在「要主 agent 跑的命令」一节；4a 的红不红全是读代码推的。
- 没跑任何门禁格，所以 3a 里「被判行」的那一半、拆分族认法接上之后哪几行判红，都没量。
- 3c 里 `e203_admission_scan_cost_counters_*.py` 四份拆出的模块红不红没逐份核；`scripts/gate/README.md` 里别的会话加的几十行只核了文件在、没核用途。
- 清理：建过一份 111M 的仓副本（`minimal-copy.py` 拷出，`du -sh` 记 111M），已删；草稿目录 `/home/fy5090/.local/share/singlefs/work/merge-verify-slots-r1-sonnet` 剩下五份 `.py` 草稿、两份文本输出，共约 140K，没有要交接的副本。
