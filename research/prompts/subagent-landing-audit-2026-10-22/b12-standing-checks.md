# 第十二批任务 49：派发提示常设要求的检查（tooling-writer，2026-10-23）

批次：records/2026-10-22-子agent结果落地审计.md#12。这是接续：前一个同类 agent 把三份脚本与共用约束第 90 行都换上了，没写报告就停了。这一趟核了现场（三份文件与草稿里最后一版逐字节相同、`lib_work_root.py` 与 `lib_long_run.py` 与开工基线相同），在主工作区把三份自证、全部弄坏开关、真提示新拒新放、各轮快照读数重跑了一遍，补写这份报告。没有再改文件。

## 结论

- 八条都做完了。派发闸 `.claude/hooks/runner-dispatch-guard.sh` 加了 ⑭「复用：」一行与「分片：」「续跑：」字数的判法，加了 ⑱ 一组（批次、草稿目录落点、写死台数与额度、自证在哪跑），⑰ 加了三方腿要行首「报告：」一行。骨架 `scripts/agents/dispatch-prompt.py` 给十种类型多打一行「复用：<填 …>」。`scripts/agents/snapshot-tree.py` 加了两种最后的取法。`.claude/agent-common.md` 第 90 行补写了这些拒。
- 第 2 条：2026-10-14 起的真派发提示里，「草稿目录：」一行落在过渡期旧根 `/tmp/claude-1000` 下的有 **0 份**（1065 份里 1048 份落在工作根下，17 份没有这一行），所以派发闸对新派发不再认旧根。只改了派发闸里这一判，`lib_work_root.py` 没动。
- 三份自证在主工作区都是绿的：派发闸「查了 609 种」，骨架「查了 110 项」，snapshot-tree「查了 26 项」。弄坏开关照门禁 code-tooling 的 break-switch-replay 格的读法逐条复跑：派发闸 128 条、骨架 20 条、snapshot-tree 9 条，每一条都是点名了却没红的 0、红了却没点名的 0。
- 真提示 3484 份（会话记录里子 agent 的首条提示 3463 份，加钉住的样本 21 份）上比改前改后：新放 0；新拒 1545 条，其中 2026-10-14 起 192 条。逐份核过，都照新规矩该拒，判法不用再收窄。B4 那一判在量的时候已经收窄过一次（宽认法在 2026-10-14 起的提示里撞 38 份，全是观测与说明；收窄后整个语料只撞 1 份，就是 E171 #21 那份原句）。
- 第 6 条：骨架落地之后派的三方腿里新拒 3 份，与任务 48 交回的 3 份是同一批：`af3d69f5f57afdbcd`、`a18d37da1ea404019`、`a1c5dbc404e8fffa8`，都是 2026-10-22 的 three-way-attack。另有钉住的样本 `a3652270eb80e4fd0` 那 3 份（landing-audit-r1 攻方腿，在骨架落地之前，只在句中写「。报告：…」）也照新判法拒。都算已知的新拒。
- 第 7 条改后：landing-audit-r4、r5 两轮变成全部对上（退 0），r1、r2、r3 各多对上 11、17、19 份。逐轮的读数在第 7 条下面。

什么现象会推翻这些结论：主工作区里这三份文件的 sha256 与下一节列的对不上，说明有人在这之后又改过；骨架照打出来的提示经 `dispatch-prompt.py --check` 退了非 0；真提示里 2026-10-14 起出现了草稿目录落在 `/tmp/claude-1000` 下的派发；或者在 `compare3.txt` 的新拒名单里找到一份照常设要求本该放行的。

## 改过的文件

| 文件 | 此刻 sha256 | 相对开工基线（兄弟任务 48 换上的那一版，`originals/` 下） |
|---|---|---|
| `.claude/hooks/runner-dispatch-guard.sh` | `15023592ab9306ba8671d8029bde5edf6cbe514d451f2a5344b671568a2f9dbf` | +413 −40 |
| `scripts/agents/dispatch-prompt.py` | `dd2a4d9486c607d681c8e8a5c3a8a8d6619b6dca8249b877ed87f3dd3cc2336a` | +96 −5 |
| `scripts/agents/snapshot-tree.py` | `52b7b3518087c0ff79d940e9ae0708921876a85a84542ddb4f0f200a8ccb83e7` | +95 −16 |
| `.claude/agent-common.md` | `7f2336ef17889bf3cdd85a582e24041017b20301e5f0b68631b5b884b6bdda19` | +1 −1（只改了第 90 行，别处与基线逐字相同） |

钩子与两份脚本都是经 `scripts/gate/hook-swap.py` 换上的：先在草稿里验，再改名换上新 inode、回读一致，再在主工作区验。派发闸换了五次，`dispatch-prompt.py` 两次，`snapshot-tree.py` 两次，每次的读数在 `swapA.txt` 到 `swapI.txt`。共用约束用 `scripts/kb/replace-once.py` 定点替换：命中 1 次，换上新 inode，回读确认过。`.claude/hooks/fixtures/runner-dispatch-guard/` 下没有加文件，新判法的样本都写在派发闸 `--selftest` 里面。

`git diff --stat -- <这些文件>` 原样（比的是暂存区，里面也有别的会话没提交的改动）：

```
 .claude/agent-common.md                |   2 +-
 .claude/hooks/runner-dispatch-guard.sh | 453 ++++++++++++++++++++++++++++++---
 scripts/agents/dispatch-prompt.py      | 101 +++++++-
 scripts/agents/snapshot-tree.py        | 111 ++++++--
 4 files changed, 605 insertions(+), 62 deletions(-)
```

## 逐条：改了什么、样本红绿、自证格与弄坏开关、真提示读数

样本都是派发闸 `--selftest` 里的格（「」里是格名），红的退 2，绿的退 0。弄坏开关的复跑读数在「自证与复跑」一节。

### 第 1 条 B6：判定复用（⑭）

- 骨架：十种类型在「多机：」那一行后面多打一行 `复用：<填 超过 2 小时才写：判定按什么入库、键相同的怎么沿用，至少 8 个字；这一件不产出判定的写「无判定：」后面接为什么；2 小时内删掉这一行>`（`REUSE_LINE`）。
- 派发闸（`long_run_line_value_problems`）：预计挂钟超过 2 小时时，要有一行「复用：」，不能是占位，不能少于 8 个字，同一标签写第二行拒，「分不了：」顶不了它。闭合了的围栏里的行不算；标签行在两种围栏认法下遮不遮不一致的拒（`LONG_RUN_LABEL_LINE` 里加了「复用」，与 ⑭ 别的标签走同一套）。
- 红：「长活:超过 2 小时没写复用一行拒」「长活:复用一行是尖括号占位拒」「长活:复用一行不到 8 个字拒」「长活:复用一行写了两行拒」「长活:分不了顶不了复用一行拒」；拒绝说明的格：「长活:缺复用一行的拒绝说明写明复用一行与无判定的写法」。
- 绿：「长活:复用写无判定与为什么放行」「长活:代码块里的复用示例行不算重复放行」「长活:那几行带粗体与列表记号也认」。
- 骨架自证加的格：十种类型各一格「设计员与带重活的类型的骨架应当带一行「复用：<填 …>」（<类型>）」、kb-scribe 不带这一行；`--check` 的三格：没写复用一行退 3、复用一行只写几个字退 3、写「无判定：」与为什么退 0。
- 弄坏开关：`RUNNER_DISPATCH_GUARD_BREAK=long-run-reuse-unchecked`（红 6 项对 6 行）；`DISPATCH_PROMPT_BREAK=reuse-line-dropped`（红 10 项对 10 行）；原有的 `long-run-duplicate-label-first-wins`、`long-run-codeblock-judged` 两条补了复用那一格的点名。
- 真提示：新拒 128 份（2026-10-14 起 121 份），全部写了「预计挂钟：超过 2 小时」又没有「复用：」一行，没有一份写了这一行却判错（`复用` 行 0 份）。其中 109 份改前就因为别的条被拒。2026-10-14 起按类型：implementation-writer 75、tooling-writer 28、three-way-attack 7、experiment-runner 6、three-way-defense 3、three-way-forward 2。这是新要求，都该拒。

### 第 2 条 B7：草稿目录不落开机会清空的目录（⑱ 草稿目录落点）

- 先数的：2026-10-14 起 1065 份真提示里，「草稿目录：」一行（认法用 `lib_prompt_scratch_line.py` 的 `scratch_line_values`）落在工作根 `/home/fy5090/.local/share/singlefs/work` 下 1048 份，落在旧根 `/tmp/claude-1000` 下 **0 份**，没有这一行的 17 份。所以这一判对新派发不认旧根。
- 派发闸（`scratch_line_root_verdict`）：值照 `lib_work_root.py` 的 `expand_written_path` 展开（`$SINGLEFS_WORK_ROOT/`、`~/`、`$HOME/`），要落在 `configured_work_root()` 下面；工作根本身、相对路径、工作根之外的都拒。`LEGACY_ROOT_IN_TRANSITION` 没动，④ ⑧ ⑯ ⑰ 与别的钩子照旧认旧根。
- 红：「草稿目录落点:落在过渡期旧根拒」「草稿目录落点:落在工作根之外拒」「草稿目录落点:写相对路径拒」「草稿目录落点:写工作根本身拒」。
- 绿：「草稿目录落点:~ 起头、落在工作根下放行」「草稿目录落点:$SINGLEFS_WORK_ROOT/ 起头放行」「草稿目录落点:闭合围栏里的旧根示例行不判」。
- 弄坏开关：`scratch-line-root-unchecked`（4 对 4）、`scratch-line-legacy-root-accepted`（1 对 1）。原有的 `writer-lock-root-locked` 那一格「写者锁:草稿目录写成工作根本身不建锁」现在先被这一判拒掉，开关打开也不红，所以文件头那一条删了这一项、写明了原因。
- 真提示：新拒 609 份，全部早于 2026-10-14，2026-10-14 起 0 份。

### 第 3 条 B4：不写死台数与额度（⑱ 写死台数与额度）

- 派发闸（`host_quota_body` + `host_quota_verdict`，只判十种类型）：先从正文里去掉闭合了的围栏代码块、「」『』“”引文、行首「线程上限：」「线程上限已判：」「内存上限：」「多机：」四种行（「多机：」一行里的额度归 ⑭ 判）、「多机已判：」那一行与紧跟它的非空行（到空行为止）。剩下的正文里只认这几种字面（`HOST_QUOTA`）：本机与第二台成对的「N 线程」；「每台、两台各、各台 N 线程或 NG」；「两台合计 NG」「内存合计 NG」。出路照派发写：「删掉，照「多机：」规范句与「线程上限：」一行；用户给了逐台参数的写「多机已判：」」。
- 先量后收窄：宽认法（`HOST_QUOTA_WIDE`：任何「N 线程 / N 核 / N 台」「两台」「每台 N」「合计 NG」）在去掉上面那几样之后的正文里，2026-10-14 起撞 38 份，整个语料撞 129 份。撞到的类别是：「丁 2 核心」这类名字（9 份）、「两台机器与 GPU 都占着」这类观测、单写一台的「31 线程」、硬件说明「32 核」、磁盘「du 合计 1159G」、「CPU 只用 1 到 4 核」。都是观测与说明，不是派给子 agent 的额度。收窄成上面那几种字面之后，整个语料只撞 1 份：`a51808a134923474f`（2026-10-06，experiment-runner，E171 #21 原句「本机 31 线程、第二台 31 线程，内存两台合计 80G」），2026-10-14 起 0 份。不收窄的话正文整份照判会撞 822 份。读数在 `resume-quota-survey.txt`。
- 红：「写死台数:正文写本机与第二台各几线程、两台合计内存拒（E171 #21 的原句）」「写死台数:正文写每台 N 线程拒」「写死台数:多机已判那一段之后空一行再写的照判」；拒绝说明的格：「写死台数:拒绝说明点名写死的数、给删掉与多机已判的出路」。
- 绿：「写死台数:「」里转述的原话不判」「写死台数:多机已判后面逐台参数那一段不判」「写死台数:内存上限一行不判」「写死台数:闭合围栏里的样本不判」「写死台数:单写一台的线程数与核数的硬件说明不判」「写死台数:不在十种类型里的不判」。
- 弄坏开关：`host-quota-unchecked`（4 对 4）、`host-quota-whole-prompt`（4 对 4）、`host-quota-wide`（2 对 2）。
- 「N 核」与单写一台的线程数没判，派发点名的「N 核」这一样也就没拦。依据是真提示：这两种在 2026-10-14 起撞到的都是观测与硬件说明。

### 第 4 条 A2：批次（⑱ 批次）

- 派发闸（`batch_verdict`）：行首「批次：」一行（闭合了的围栏里的不算）的值要形如 `records/<文件>.md#<批号>`。批号不能空、不能是占位，写法不限数字（真提示里有「#批6」「#第 3 批后半」这类，只认数字的话会误拒）；批号后面接句号、逗号、括注与别的话的照认。文件要在本仓仓根或登记过的某棵工作树里（`lib_three_way_legs.py` 的 `registered_trees`），路径里不许有 `..`。十种类型没有这一行就拒；别的类型写了才判。骨架原来就打这一行。
- 红：「批次:十种类型没写批次一行拒」「批次:点名的调度记录不在拒」「批次:别的类型写了批次一行照判点名的调度记录」「批次:不是 records/<文件>.md#<批号> 的写法拒」「批次:路径里带 .. 拒」「批次:闭合围栏里的批次一行不算」；拒绝说明的格：「批次:拒绝说明点名不在的调度记录、给照抄的一行」；骨架 `--check` 的格：「工具员骨架批次一行点名不在的调度记录，--check 应当退 3 并点名那份记录」。
- 绿：「批次:别的类型没写批次一行不判」「批次:批号写批名放行」「批次:批号后面接句号与别的话放行」。
- 弄坏开关：`batch-unchecked`（7 对 7）、`batch-line-optional`（2 对 2）、`batch-file-unchecked`（3 对 3）、`batch-label-digits-only`（2 对 2）。
- 真提示：新拒 570 份，2026-10-14 起 3 份：`afea6a9d9d7029750`、`a14f4df4f73faaf49`、`adf41ba5939a72c36`，都是 2026-10-14 的 tooling-writer，拒的原因都是「没有「批次：」一行」，都该拒。点名的文件不在、写法认不出的新拒是 0。

### 第 5 条 B2b、B10：分片 / 续跑字数，自证在哪跑

- 「分片：」「续跑：」：超过 2 小时、写了的值不能少于 8 个字（`long_run_line_value_problems`；缺行与占位照旧归 `lib_long_run.py` 的 judge 判）。红：「长活:分片一行不到 8 个字拒」「长活:续跑一行不到 8 个字拒」；开关 `long-run-values-unchecked`（2 对 2）。改的时候把原有样本里「续跑：已有的行跳过」补长了，放行的格照旧绿。
- 「自证在哪跑：」（`selftest_location_verdict`）：值里点名了工作根或旧根下的路径（`SCRATCH_MENTION_PREFIX` 认的那几种起头）就拒；「本仓主工作区」「不跑」放行；不带路径的说法认不出，不判；占位交给 ⑬。红：「自证在哪跑:指到工作根下的仓副本拒」「自证在哪跑:指到过渡期旧根拒」；绿：「自证在哪跑:本仓主工作区放行」「自证在哪跑:不跑放行」；开关 `selftest-location-unchecked`（2 对 2）。原有的 `SINGLEFS_WORK_ROOT_BREAK=legacy-root-unknown`、`new-root-unknown` 补了这一格的点名。
- 真提示：这两样新拒都是 0（`compare3.txt` 里没有「⑭分片续跑字数」「自证在哪跑」两类）。

### 第 6 条：三方腿要行首「报告：」一行（⑰）

- 派发闸（`leg_report_line_verdict`，类型取定义 frontmatter 登记了 `leg: three-way` 的）：没有行首「报告：」一行（`LINE_START` 起头，闭合了的围栏里的不算）就拒。句中写的「报告写进 …」「。报告：…」都不算这一行。出路是照骨架写「报告：research/prompts/<轮>-<模型>-output.md」。⑰ 原有的形态判法不变。
- 红：「三方报告:没有行首「报告：」一行、句中写报告写进拒」「三方报告:「报告：」一行只在闭合围栏里拒」；骨架那一格：开关 `three-way-report-default-dropped` 下多红一行「✗ 派 three-way-attack 的提示里没有行首「报告：」一行（句中写的「报告写进 …」不算这一行）。」，已经补进文件头。
- 绿：「三方报告:行首「报告：」一行放行」，加上原有的三方腿放行格。原有那批只判形态的格（句中写报告、只写报告路径、禁读清单那几格）用 `ignored=("三方腿报告行",)` 只判它们自己那一条。
- 弄坏开关：`leg-report-line-optional`（2 对 2）。
- 真提示：新拒 237 份，2026-10-14 起 68 份。骨架落地（任务 48 取的时刻，2026-10-22）之后派的三方腿里新拒 3 份，就是任务 48 交回第 6 条那 3 份：`af3d69f5f57afdbcd`、`a18d37da1ea404019`（m3-close-reuse-closure 两轮）、`a1c5dbc404e8fffa8`，都是 2026-10-22 的 three-way-attack，报告只写在句中「报告写进 …-output.md」。钉住的样本 `a3652270eb80e4fd0.txt` 与它的 `-section-colon`、`-delivery-colon` 两份（landing-audit-r1 攻方腿，句中写「。报告：research/prompts/landing-audit-r1-opus-output.md。」）也新拒。这 6 份都是已知的新拒。派发闸 `--selftest` 的真提示格只判草稿目录与报告名，不判整份放不放，所以照旧绿。

### 第 7 条：`snapshot-tree.py` 再加两种取法

- 改法（`fallback_content` + `failed_or_fallback`）：照原样与照搬迁登记都倒推不出或对不上之后，按下面的次序再试两样。第一样：diff 头记的结果 blob（index 行 `..` 后面那个）还在对象库里、sha256 与快照相同的，取它。第二样：工作区此刻那一份（仓根下清单点名的路径，普通文件）sha256 与快照相同的，取它。两样都是逐字节比 sha256。取到的铺进树，那一行说明里写取的是哪一种、照原样为什么不行；汇总行加一句「对上的里有 N 份取 diff 头记的结果 blob、N 份取工作区此刻那一份」。
- 自证格（26 项，改前 23 项）：「结果 blob 还在对象库里」（r6：基底取不到，结果 blob 经 `git hash-object -w` 写进对象库，退 0，那一行来源写结果 blob）；「工作区那一份对得上」（r7：一份工作区里 sha256 对得上的被取到，一份工作区里改过的照报倒推不出，退 1，改过的那份不铺进树）。
- 弄坏开关：`SNAPSHOT_TREE_BREAK=result-blob-ignored`（3 对 3）、`worktree-ignored`（4 对 4）；原有的 `head-only`、`no-unquote`、`index-diff-ignored`、`path-moves-ignored`、`diff-markers-kept` 照新的失败行改了点名，复跑点名了却没红的 0、红了却没点名的 0。
- 各轮读数（`python3 scripts/agents/snapshot-tree.py <轮> --out <草稿>/snap2/<轮>-tree`，原样输出在 `snap2/<轮>.txt`；「任务 48 交回」一栏抄自 `b12-dispatch-followup.md` 第 8 条那张表）：

| 轮 | 清单点名 | 任务 48 交回 对上 | 这一批改后 对上 | 其中取结果 blob | 其中取工作区那一份 | 改后倒推不出 / 对不上 | 退出码 |
|---|---|---|---|---|---|---|---|
| landing-audit-r1 | 51 | 30 | 41 | 0 | 11 | 10 / 0 | 1 |
| landing-audit-r2 | 63 | 36 | 53 | 0 | 17 | 10 / 0 | 1 |
| landing-audit-r3 | 74 | 46 | 65 | 0 | 19 | 9 / 0 | 1 |
| landing-audit-r4 | 80 | 77 | 80 | 3 | 0 | 0 / 0 | 0 |
| landing-audit-r5 | 27 | 26 | 27 | 1 | 0 | 0 / 0 | 0 |
| landing-audit-r6 | 40 | 40 | 40 | 0 | 0 | 0 / 0 | 0 |
| apparatus-rename-r1 | 14 | 12 | 12 | 0 | 0 | 2 / 0 | 1 |
| apparatus-rename-r2 | 15 | 0 | 2 | 0 | 2 | 13 / 0 | 1 |

  r4 汇总行原样：「✓ 快照 landing-audit-r4：清单点名 80 份，倒推出 80 份，sha256 对上 80 份（基底取自 HEAD 26 份、diff 头记的 blob 0 份、新文件 51 份；diff 用的是 judged-vs-head.diff；对上的里有 13 份照搬迁登记换回旧路径；对上的里有 3 份取 diff 头记的结果 blob、0 份取工作区此刻那一份）；树在 …/snap2/landing-audit-r4-tree」。
  r1 到 r3 剩下没对上的 10、10、9 份里，有 28 份的失败行是「倒推不出：diff 里没有它，此刻 HEAD 里也没有」，两种新取法也取不到。这一类就是任务 48 说的「暂存了没提交的新文件」，结果 blob 不在对象库里，工作区那一份又对不上或者已经不在（推的：没逐份核工作区是改了还是删了）。剩下 1 份是 r3 的 `.claude/kb/checks-owed.md`，属于任务 48 第 3 种（暂存区另有改动，那一版没留下）。任务 48 记 apparatus-rename-r1 有 1 份（`.claude/gate.d/path-moves.tsv`）工作区那一份对得上，这一趟没有对上，推的：那份登记表在那之后又改过。

### 第 8 条：`.claude/agent-common.md` 第 90 行派发闸那一条

用 `scripts/kb/replace-once.py` 定点替换，旧串是这一条原来的结尾「「草稿目录：」点名的目录里写者锁的持有者还在跑，又没写「写者锁已判：<理由>」（⑯）。」，命中 1 次。替换之后与开工基线比，只有第 90 行不同。补写的只有拒什么：
- ⑯：「草稿目录：」一行照 CommonMark 认（没收尾的围栏遮到文末）与照逐行配围栏认遮不遮不一致（任务 48 第 3 条）。
- ⑰ 报告名：报告行点名的、落在本提示草稿目录下或不带目录的文件名里带 report（不分大小写），或以 SUMMARY、FINDINGS、ANALYSIS 开头的 .md（前缀与扩展名都不分大小写；任务 39、48 第 1 条）。
- ⑰ 三方腿：没有行首「报告：」一行（这一件第 6 条）；没有一处本腿的报告 `research/prompts/<轮>-<模型>-output.md`（报告行点名的，或不在引号与禁读清单里、名字以本腿轮名起头的；`$SINGLEFS_WORK_ROOT/` 起头的照 `lib_work_root.py` 展开，任务 48 第 2 条）；报告行点名草稿目录下的文件或裸文件名；禁读清单里点到本腿自己的报告（任务 39）。
- ⑭：超过 2 小时缺「复用：」一行、这一行是占位、不到 8 个字、写了两行，或「分片：」「续跑：」不到 8 个字（第 1、5 条）。
- ⑱：批次一行的写法、文件在不在、十种类型缺这一行（第 4 条）；草稿目录不在工作根底下（第 2 条）；十种类型正文写死主机资源的数（第 3 条）；「自证在哪跑：」点名工作根或旧根下的路径（第 5 条）。

## 真提示上的新拒新放

- 语料：会话记录 `~/.claude/projects/-home-fy5090-code-singlefs/*/subagents/agent-*.jsonl` 里每个子 agent 的首条提示，共 3463 份；加上 `.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/` 下的 21 份，合计 3484 份，2026-10-14 起 1065 份。2026-10-23 这一趟重新抽了一遍（`corpus2.jsonl`）。
- 判法：两份派发闸各在进程里跑 `judge_dispatch(..., check_only=True)`，与 `--check-prompt` 走同一条路。改前是开工基线 `originals/.claude/hooks/runner-dispatch-guard.sh`，改后是此刻主工作区这一份（拷到 `draft/guard-current-for-judge.sh` 再判）。分 6 片并行，12 件的退出码文件都是 0，判出错的 0 份。工具：`tools/extract_corpus.py`、`tools/judge_corpus.py`、`tools/compare.py`。全部新拒逐份的名单在 `compare3.txt`。
- `compare3.txt` 开头原样：

```
提示 3484 份（2026-10-14 起 1065 份）
新拒：1545 条（按条名）
  ⑭复用：128 份，其中 2026-10-14 起 121 份
  三方腿报告行：237 份，其中 2026-10-14 起 68 份
  写死台数与额度：1 份，其中 2026-10-14 起 0 份
  批次：570 份，其中 2026-10-14 起 3 份
  草稿目录落点：609 份，其中 2026-10-14 起 0 份
新放：0 条（按条名）
```

- 2026-10-14 起的新拒逐份列在下面（日期、子 agent 号、类型）。2026-10-14 之前的 1353 条在 `compare3.txt` 里逐份列着：草稿目录落点 609 份全落在旧根，批次 567 份没有这一行，三方腿报告行 169 份在骨架之前，⑭复用 7 份，写死台数 1 份（`a51808a134923474f`）。
新拒 ⑭复用（2026-10-14 起 121 份）
  2026-10-14 a98357774c4c42040 implementation-writer
  2026-10-14 a2e438ae401656464 tooling-writer
  2026-10-14 a7c31dd918efb9d2e implementation-writer
  2026-10-14 ac8fdb802ef70e3eb implementation-writer
  2026-10-14 a2d334198157214d9 implementation-writer
  2026-10-14 a3710aafc429e4302 tooling-writer
  2026-10-14 ad645c2b93650a816 tooling-writer
  2026-10-14 a46cfe3d4abbf4f0d tooling-writer
  2026-10-14 a53c20c1ec7d3d9f6 tooling-writer
  2026-10-14 a658ad4f4485aae50 tooling-writer
  2026-10-14 ae4802e8ffb3e8733 experiment-runner
  2026-10-14 aa36a9879953e1e03 implementation-writer
  2026-10-14 a74470d57633dbaeb implementation-writer
  2026-10-14 aee297c2817afd59b implementation-writer
  2026-10-15 a2edfc721a2f26056 implementation-writer
  2026-10-15 ad382e73e38f29533 implementation-writer
  2026-10-15 aebd047b8a78d799d tooling-writer
  2026-10-15 a7c947481bebb703a implementation-writer
  2026-10-15 a5b80904ef9d2903f implementation-writer
  2026-10-15 ae2cf6cb79dce006f implementation-writer
  2026-10-15 ac444c41efaba339b implementation-writer
  2026-10-15 abd28133ddf221a97 implementation-writer
  2026-10-15 a03e547801f697e93 implementation-writer
  2026-10-15 aef9d9c646543f7ce implementation-writer
  2026-10-15 ad6c2e083a1e3358f tooling-writer
  2026-10-15 ad276b43686bf3cdb tooling-writer
  2026-10-15 adf67955eaefa441e tooling-writer
  2026-10-15 ac4ea1564b70fa4c5 tooling-writer
  2026-10-15 a71befa194ca9b9c4 implementation-writer
  2026-10-15 a028b96f139ab0bc1 tooling-writer
  2026-10-15 ab20ee3bf53977caa implementation-writer
  2026-10-15 a3d1834688f7b024f tooling-writer
  2026-10-15 ad5b589e11f82aa53 implementation-writer
  2026-10-15 a5c763279c02072af tooling-writer
  2026-10-15 af3b7434ab9fcef44 tooling-writer
  2026-10-15 a29ca7df0f9aa56bc implementation-writer
  2026-10-15 af9125a06669fe21d implementation-writer
  2026-10-15 a51c4a7bb50b69883 tooling-writer
  2026-10-15 a64c18bf073613589 tooling-writer
  2026-10-15 adc00c31b05927361 tooling-writer
  2026-10-15 a7cbf6a75997dbe8c implementation-writer
  2026-10-15 a59369cac6e041e8d tooling-writer
  2026-10-15 adfab145ae912ddd8 tooling-writer
  2026-10-15 acc6505c8acbfe5b3 tooling-writer
  2026-10-15 af48af68778936bda implementation-writer
  2026-10-15 aece3b70829ba5ce5 three-way-attack
  2026-10-15 a823ae170ca3a7b37 three-way-forward
  2026-10-15 a8f5ec8ba6204e83a implementation-writer
  2026-10-15 ad93dadb1a865eefd implementation-writer
  2026-10-15 ab75e8fde50976dad implementation-writer
  2026-10-15 acfba9bf973a7e11b implementation-writer
  2026-10-16 ab35e8deabc2eb5fc implementation-writer
  2026-10-16 a40e173fc3b80dc0f three-way-attack
  2026-10-16 aefab183b90cda084 three-way-defense
  2026-10-16 a8dcdbcb953359856 implementation-writer
  2026-10-16 a764f125f798d42bd three-way-attack
  2026-10-16 affe8ef85c18cf398 three-way-defense
  2026-10-16 a1b734e0d4e6b8e95 implementation-writer
  2026-10-16 a7b472ae1b1515c81 implementation-writer
  2026-10-16 ae7ff5ead0f0c92a9 implementation-writer
  2026-10-16 a2adbe251b3465c9f implementation-writer
  2026-10-17 a70ad565379360ef4 implementation-writer
  2026-10-17 a5793100f108eb07a experiment-runner
  2026-10-18 a788f431db074f846 implementation-writer
  2026-10-18 aef9f3f1debc30836 implementation-writer
  2026-10-18 a728f14bbc23a8fdb implementation-writer
  2026-10-18 a542546a7883a91db implementation-writer
  2026-10-18 a8490f51798802e11 implementation-writer
  2026-10-18 a1fa2eaffb7f7f3e8 implementation-writer
  2026-10-18 aa014bdf68bad54cb tooling-writer
  2026-10-18 ad95dc2d10c83e9b9 experiment-runner
  2026-10-18 a2ecb2367e5bc1f02 experiment-runner
  2026-10-18 ac9b9ef5e6dc060de experiment-runner
  2026-10-18 a5c0f8cd69ef445c5 implementation-writer
  2026-10-19 a5a8c321517e6654d experiment-runner
  2026-10-19 addae744baac83dd0 implementation-writer
  2026-10-19 aa76c428cce9b4ce7 implementation-writer
  2026-10-19 a3679821a8edca0f8 implementation-writer
  2026-10-19 aefa1667cc7e6f7e5 implementation-writer
  2026-10-19 a47a6e6f9b0e25950 implementation-writer
  2026-10-19 a2c29a35f9f9567ea implementation-writer
  2026-10-19 a8ebedfe8fa675f59 implementation-writer
  2026-10-19 a9373a1bdd336e9c6 implementation-writer
  2026-10-19 a4337757c0a0e2189 implementation-writer
  2026-10-19 afa34c9b55ae7bd3f implementation-writer
  2026-10-20 a44b80a56169ea025 tooling-writer
  2026-10-20 ad156bede96eb6ea8 implementation-writer
  2026-10-21 a6da5bbea8d19d7bf tooling-writer
  2026-10-21 a79a6dae23ce48f9a three-way-attack
  2026-10-21 a059f71f99ac8c52c three-way-defense
  2026-10-21 a0ba4466defcb0a7e implementation-writer
  2026-10-21 a0c2df320382c14bd implementation-writer
  2026-10-22 a84c85b58b9f07337 implementation-writer
  2026-10-22 adb4b342c53463f74 tooling-writer
  2026-10-22 adbb3ad2e4301f725 three-way-attack
  2026-10-22 a4e68d8e365461f29 three-way-attack
  2026-10-22 af1ed53b7173c4f39 three-way-forward
  2026-10-22 a3fb10ce83955f774 implementation-writer
  2026-10-22 ab7bcc632ae000dd7 three-way-attack
  2026-10-22 ac59343e8f41db069 implementation-writer
  2026-10-22 a21d15f91b9bc333c implementation-writer
  2026-10-22 a51a99f7e448b8c83 implementation-writer
  2026-10-22 a9b89b5e8f6f14cff implementation-writer
  2026-10-22 a4c15da80d700df15 implementation-writer
  2026-10-22 a567b63eeaaeb38cb implementation-writer
  2026-10-22 a8bac87681bca82f2 implementation-writer
  2026-10-22 a564e644c8df1540b implementation-writer
  2026-10-22 a1da34b66db32161b implementation-writer
  2026-10-22 ab223d4e3014ce887 implementation-writer
  2026-10-22 a993fda3eb837343d tooling-writer
  2026-10-22 a55de88df0f612b0b tooling-writer
  2026-10-22 a8eacf46d70af59dd implementation-writer
  2026-10-22 a6576d6bc9000480f implementation-writer
  2026-10-22 a6e8bd09dcf72f5e0 tooling-writer
  2026-10-22 aa6c0cead2f03f08e implementation-writer
  2026-10-22 acf04b49293b12bba implementation-writer
  2026-10-22 a84e51c003c8c60bd implementation-writer
  2026-10-22 a21722bc9e821a995 implementation-writer
  2026-10-22 ad78da731d96813af implementation-writer
  2026-10-22 a011035aff62edc42 implementation-writer
  2026-10-23 a391f3778dde9e885 implementation-writer
新拒 三方腿报告行（2026-10-14 起 68 份）
  2026-10-14 ac4e3ba58ba9cc219 three-way-attack
  2026-10-14 aa2e0010e89f52933 three-way-attack
  2026-10-14 a2c672cd1c579e8a9 three-way-attack
  2026-10-14 a236d6bda6812358f three-way-attack
  2026-10-14 a3adaff8184055a3b three-way-attack
  2026-10-14 a621bb3c92922f171 three-way-defense
  2026-10-15 a21a59c66c0993a6a three-way-attack
  2026-10-15 a9e9af22cb0006e5e three-way-forward
  2026-10-15 a23f01174e1eedb10 three-way-attack
  2026-10-15 aae9b0d5968a12114 three-way-forward
  2026-10-15 a68d6ac4aac8aa4fa three-way-attack
  2026-10-16 a7ffb36ae50f60924 three-way-attack
  2026-10-16 ac9b3e510ee361343 three-way-forward
  2026-10-16 a28a25d325012853c three-way-attack
  2026-10-16 a47da288974952a2c three-way-attack
  2026-10-16 a2e0cc6ef56fad5c8 three-way-attack
  2026-10-17 a5c60e03b6a25e6c8 three-way-forward
  2026-10-17 a8f6fe9f3bff590d1 three-way-attack
  2026-10-17 af82b7635471f218f three-way-attack
  2026-10-17 a7d14a698679766e7 three-way-forward
  2026-10-18 a8341d19c49cdd69e three-way-attack
  2026-10-18 a49f828dc940984c0 three-way-defense
  2026-10-18 a3d3757fffd6e91a2 three-way-attack
  2026-10-18 a3dc7af43d9f231ed three-way-defense
  2026-10-18 a546570e162def915 three-way-attack
  2026-10-19 a992667cf3967f3de three-way-attack
  2026-10-19 a92f64094b3c19beb three-way-attack
  2026-10-19 a12e849aeb20b1463 three-way-attack
  2026-10-19 ad04282558e3cb04f three-way-attack
  2026-10-19 a9d454ba090f7f24b three-way-attack
  2026-10-19 a0d4e2dc603c11e52 three-way-attack
  2026-10-19 a3c328c9365553767 three-way-attack
  2026-10-19 a3299d166f66b9eb9 three-way-attack
  2026-10-19 a1b52b1b838e1a44d three-way-attack
  2026-10-19 a366e637128225bf7 three-way-attack
  2026-10-19 a028ec0c5ee1bd7bb three-way-attack
  2026-10-19 a3a3385bf1eb85b09 three-way-attack
  2026-10-19 a85f888241a80a1e8 three-way-attack
  2026-10-19 ab8d954ef238cb628 three-way-attack
  2026-10-19 a063ef4cff4377e09 three-way-defense
  2026-10-20 affff11036594ee42 three-way-attack
  2026-10-20 a82d92d119f3319b9 three-way-attack
  2026-10-20 aa0e273563254db19 three-way-attack
  2026-10-20 ac3d289709ab7d454 three-way-attack
  2026-10-20 aa1c8d5fbdf6bec57 three-way-attack
  2026-10-20 a357d2cac1b1524f1 three-way-attack
  2026-10-20 a3435c9a3a5875e30 three-way-attack
  2026-10-20 a646eca7ce75ce550 three-way-attack
  2026-10-20 a420550f0069007a8 three-way-attack
  2026-10-20 aa8a38f0232dab4e5 three-way-attack
  2026-10-21 a3403bc65f50c224c three-way-attack
  2026-10-21 a241e665af71117c0 three-way-attack
  2026-10-21 aadd0c365b3da2113 three-way-attack
  2026-10-21 a96381f44beab50ff three-way-attack
  2026-10-21 a0d286439e256b124 three-way-attack
  2026-10-21 a79a6dae23ce48f9a three-way-attack
  2026-10-21 a059f71f99ac8c52c three-way-defense
  2026-10-21 a597c61e413e638b4 three-way-attack
  2026-10-22 afd6cbb6e192cb293 three-way-attack
  2026-10-22 a68335bb0a021de91 three-way-attack
  2026-10-22 ae25ed4e143e8e3f3 three-way-attack
  2026-10-22 a3652270eb80e4fd0 three-way-attack
  2026-10-22 a753fc916bf4864f3 three-way-attack
  2026-10-22 a9b82cd4cd97c6839 three-way-attack
  2026-10-22 aebc9d7bfa16c60e7 three-way-attack
  2026-10-22 af3d69f5f57afdbcd three-way-attack
  2026-10-22 a18d37da1ea404019 three-way-attack
  2026-10-22 a1c5dbc404e8fffa8 three-way-attack
新拒 写死台数与额度（2026-10-14 起 0 份）
新拒 批次（2026-10-14 起 3 份）
  2026-10-14 afea6a9d9d7029750 tooling-writer
  2026-10-14 a14f4df4f73faaf49 tooling-writer
  2026-10-14 adf41ba5939a72c36 tooling-writer
新拒 草稿目录落点（2026-10-14 起 0 份）

上面 ⑭复用 121 份全部写了「预计挂钟：超过 2 小时」又没有「复用：」一行；三方腿报告行 68 份全部没有行首「报告：」一行，其中骨架落地之后的只有 `af3d69f5f57afdbcd`、`a18d37da1ea404019`、`a1c5dbc404e8fffa8` 三份；批次 3 份全部缺这一行。没有找到不该拒的，所以除了 B4 那一次，判法都没再收窄。

## 自证与复跑（这一趟在主工作区跑，都登记为 light，`nice -n 19`）

开跑前用 `ps` 看过负载：有别的会话的 checker 档 `cargo test`（经 `gpu-quiesce-run.sh`、`capped.sh 31`）和三条 harness `cargo test` 在跑，没有性能测量在跑，所以在本机加 `nice -n 19` 照常跑。

| 命令 | 退出码 | 汇总行开头（全文在草稿目录） |
|---|---|---|
| `bash .claude/hooks/runner-dispatch-guard.sh --selftest` | 0 | 「✓ 自检通过（查了 609 种）：没写「重型测试：不跑」一行的拒…」（`resume-guard.txt`，✗ 行 0；单跑一次约 24 秒） |
| `python3 scripts/agents/dispatch-prompt.py --selftest` | 0 | 「✓ 自检通过（查了 110 项）：有定义的类型按 required-inputs 逐组一行…」（`resume-dp.txt`，✗ 行 0） |
| `python3 scripts/agents/snapshot-tree.py --selftest` | 0 | 「✓ snapshot-tree 自检通过（查了 26 项）：快照之后有提交的按 diff 头记的基底 blob 倒推…」（`resume-snap.txt`，✗ 行 0） |
| `bash -n .claude/hooks/runner-dispatch-guard.sh` | 0 | |
| `python3 -m py_compile scripts/agents/dispatch-prompt.py scripts/agents/snapshot-tree.py` | 0 | |

弄坏开关复跑（`tools/replay.py`：开关表与「」项的读法从 `.claude/gate.d/code-tooling.sh` 的 `PY_BREAK_SWITCH_TABLE` 现取，别的 `_BREAK` 变量与 `SINGLEFS_HEAVY_TESTS` 摘掉，逐条设上跑 `--selftest`；判据与门禁那一格一样，点名了却没红的 0、红了却没点名的 0），原样：

```
派发闸 1/4：switches 32; exact 32; missing>0 0; unnamed>0 0; count-differs 0; exit0 0
派发闸 2/4：switches 32; exact 32; missing>0 0; unnamed>0 0; count-differs 0; exit0 0
派发闸 3/4：switches 32; exact 32; missing>0 0; unnamed>0 0; count-differs 0; exit0 0
派发闸 4/4：switches 32; exact 32; missing>0 0; unnamed>0 0; count-differs 0; exit0 0
骨架：switches 20; exact 13; missing>0 0; unnamed>0 0; count-differs 7; exit0 0
snapshot-tree：switches 9; exact 7; missing>0 0; unnamed>0 0; count-differs 2; exit0 0
```

骨架有 7 条、snapshot-tree 有 2 条项数与失败行数不等，但点名了却没红的与红了却没点名的都是 0：一项照到了几行（骨架一格失败时把派发闸的输出也带进失败行，任务 48 交回第 7 条说过同一回事）。门禁那一格按这两样判，不比条数。新开关逐条（项 / 失败行）：派发闸 `long-run-reuse-unchecked` 6/6、`long-run-values-unchecked` 2/2、`batch-unchecked` 7/7、`batch-line-optional` 2/2、`batch-file-unchecked` 3/3、`batch-label-digits-only` 2/2、`scratch-line-root-unchecked` 4/4、`scratch-line-legacy-root-accepted` 1/1、`host-quota-unchecked` 4/4、`host-quota-whole-prompt` 4/4、`host-quota-wide` 2/2、`selftest-location-unchecked` 2/2、`leg-report-line-optional` 2/2；骨架 `reuse-line-dropped` 10/10；snapshot-tree `result-blob-ignored` 3/3、`worktree-ignored` 4/4。读数在 `resume-replay-guard-q1..q4.jsonl`、`resume-replay-dp.jsonl`、`resume-replay-snap.jsonl`。三份脚本都登记在 `.claude/gate.d/break-switch-replay-files.tsv` 里，表没动。

照新骨架打的提示（`python3 scripts/agents/dispatch-prompt.py tooling-writer --batch 'records/2026-10-22-子agent结果落地审计.md#12'`，把占位填成真值）过 `--check`：
- 2 小时内（删掉分片、续跑、多机、复用四行）：退 0，「✓ 派发闸照这份提示放行（tooling-writer，查了 21 行提示）…」。
- 超过 2 小时（四行都填，多机用规范句，复用写「判定按单元键入库，键相同的照库里的结论沿用」）：退 0，「…查了 25 行提示…」。
- 超过 2 小时、删掉复用一行：退 3，「缺 复用（复用：）：超过 2 小时的活要写明判定按什么入库…」「→ 派发闸照这份提示会拒 1 处（预计挂钟与长活声明）…」。
- 十种真定义的骨架填完都过 `--check`，这一样由骨架自证里原有的那几格判，照旧绿。

## 新写的 `# gate-similar:` / `# hook-events:` 行

没有：没有新钩子，也没有新门禁阶段。派发闸原有的 `# gate-similar:` 几行没改。

## 第二种活改过的定义、共用约束与规则

- `.claude/agent-common.md` 第 90 行（派发闸那一条），交主 agent 开定义三方（门禁 doc-process-records 的 agent-def-adversarial-review 格）。别的定义与规则没动。

## 要主 agent 跑的命令

没有。三份自证都登记为 light，这一趟已经在主工作区跑绿。没有改 `.rs`，不用跑命名检查。

## 没做什么

- 依据派发提示，不做的几条（主 agent 照这几条记进拆分提案第九十节 ⑤）：
  - B8 续派指到前一份报告：只有执行员有续做判据，派发闸续派那一判已经核了（「上一段岔路表里还差：」）。
  - B9 内存上限：门禁与装置都有缺省上限（`REPLAY_MEMORY_CAP` 这一类，经 `run-with-memory-cap.sh`），派发时不判。B4 那一判也不看「内存上限：」一行。
  - D3 不碰的文件：撞文件由 `apply-writer-patch.py` 与写范围表 `.claude/hooks/agent-write-scope.tsv` 拦。
  - E2 弄坏开关名：工具员定义第 2 步要它自己起名。
  - C4 先装置后分段的派法：这是派发的次序，不写在任何一份提示里，派发闸看不到。
  - A5「关的是哪几条」是不是已经抓到的问题：这是内容，判不了。
- B4 没判「N 核」与单写一台的线程数（第 3 条写了依据：真提示里撞到的都是观测与硬件说明）。写死的是派法还是观测，钩子分不出，只认那几种成对与合计的字面，写成别的说法的照样漏。
- 「自证在哪跑：」只认带路径的写法；「副本里」这类不带路径的说法认不出，不判。
- 没往 `.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/` 加样本，新判法的样本都写在自证里。钉住的样本 `a3652270eb80e4fd0` 那 3 份照新判法会整份拒，自证里的真提示格不判整份放不放，没改它们的期望。
- 没改 `.claude/hooks/lib_work_root.py`（第 2 条数出来是 0 份，用不着改库）；没碰 `.claude/gate.d/`。
- 没跑重型测试（`checker-tier-crash-point-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`）；没跑门禁阶段与 lint（开工时按定义跑过 `gate-overlap.py --list`，读数在 `gate-overlap-list.txt`）；没走定义三方；没提交。

## 清理

草稿目录里没有仓副本、编译目录与库拷贝。`snap2/` 下各轮铺出的树（约 30M）记完读数就删了，只留 `<轮>.txt`。留下的是开工基线备份 `originals/`（4 份文件加两份库）、各次换上前后的草稿 `draft/`、语料与判读 `corpus*.jsonl`、`judged*.jsonl`、`compare*.txt`，还有 `tools/` 下的辅助脚本，都是普通文件。
