# 第十二批任务 41：门禁 doc-experiments 与 doc-process-records 新判、claim-experiment 开关（2026-10-23）

批次：records/2026-10-22-子agent结果落地审计.md#12。关的是 `records/2026-09-16-subagent拆分提案.md` 第九十节 ②、③、㉟（㉟ 里 2.19、2.20、2.22 三件，原始出处 `research/prompts/subagent-landing-audit-2026-10-22/b9-gates-b.md` 第 269、271、272 行）。

## 结论

- ② 做了：doc-experiments 的 preregistration-parallelism 格加「挂钟算式」一判。真仓 E186、E187、E197 三份登记各判出 1 条、都在一个数量级以内（绿）；改坏一个数的三份副本逐条判红；认不出态数或单态耗时的登记报「本次未判」并点名、往 `$GATE_NOT_RUN_FILE` 报一行。
- ③ 做了：doc-process-records 加第十格 deferred-items-registered，词表是文件头常量 `DEFERRAL_WORDS`。真仓 `records/` 在 HEAD 与此刻工作区之间新增的行里，判了 162 行、红 107 行（逐行在第五节），交主 agent，`records/` 没改。
- ㉟ stage-owners：replay-comparison-units 一行归属加上 experiment-designer。
- ㉟ claim-experiment：`CLAIM_SKIP_USED_CHECK`、`CLAIM_SKIP_WORKTREES` 改成 `CLAIM_EXPERIMENT_BREAK=used-check-skipped`、`=worktrees-skipped`；平常跑设了退 2 并点名（原有的守卫，现在三个值都走它）；自证失败行带 ✗；登进 `.claude/gate.d/break-switch-replay-files.tsv`（60 秒）。草稿里照复跑格同一段判法只复跑这一份：3 条都对；改前那份 1 条「不符」。
- 什么现象会推翻：主 agent 暂存样本后跑 stage-selftest，新加的四份样本里有判错的；或下面列的弄坏开关打开后判错的样本与头注点名的不一样。

## 改过的文件

`git diff --stat -- <这五份>` 原样（工作区对暂存区，doc-experiments.sh 里含另一个会话没暂存的改动）：

```
 .claude/gate.d/break-switch-replay-files.tsv |   1 +
 .claude/gate.d/doc-experiments.sh            | 236 +++++++++++++++++++++++---
 .claude/gate.d/doc-process-records.sh        | 239 ++++++++++++++++++++++++++-
 .claude/gate.d/stage-owners.tsv              |   3 +-
 scripts/kb/claim-experiment.sh               |  67 ++++----
 5 files changed, 488 insertions(+), 58 deletions(-)
```

只算这一批的改动（对开工时留的底 `~/.local/share/singlefs/work/landing-audit-b12-gates-a/originals/originals/`，`git diff --no-index --numstat`）：doc-experiments.sh +178 −12、doc-process-records.sh +232 −7、stage-owners.tsv +2 −1、claim-experiment.sh +36 −31、break-switch-replay-files.tsv +1 −0。另一个会话在 doc-experiments.sh 里的几处（第 1644、1768、1816、1830 行起的 hunk：`REPRO_RE`、experiment-orphans、quoted-result-lines 历史节）还在，没动。

新建的样本（都还没暂存）：
- `.claude/gate.d/fixtures/doc-experiments.sh/preregistration-parallelism-wallclock-red/`（setup.sh 51 行、expect 9 行、.gate-cells）
- `.claude/gate.d/fixtures/doc-experiments.sh/preregistration-parallelism-wallclock-green/`（setup.sh 63 行、expect 7 行、.gate-cells）
- `.claude/gate.d/fixtures/doc-process-records.sh/deferred-items-registered-red/`（setup.sh 49 行、expect 12 行、.gate-cells）
- `.claude/gate.d/fixtures/doc-process-records.sh/deferred-items-registered-green/`（setup.sh 55 行、expect 4 行、.gate-cells）

没有新门禁文件、新钩子，不写 `# gate-similar:` / `# hook-events:`；两格都是并进已有的门禁（`gate-overlap.py --list` 看过：判跑前登记的只有 doc-experiments，判 `records/` 记录的只有 doc-process-records）。定义、共用约束、规则一份没改。

改法：脚本与门禁都经 `scripts/kb/replace-once.py` / `insert-row.py` 定点改（换新 inode）；claim-experiment.sh 在草稿里改好、拷到同目录临时名再 `mv` 换上。

## 一、② preregistration-parallelism 加判挂钟算式

节号与写法照 `.claude/agents/experiment-designer.md` 现读：第 3a 步「每个量按它的定义现算态数或输入规模与单态耗时……相乘得挂钟」，「登记的固定节名」的 `## 十四、并行与挂钟`。真仓三份 E186 起的登记，态数与单态耗时只写在第十四节「预计挂钟：」那一行里（第六节一条带时间单位的数都没有，草稿里扫过），所以判法只读那一行。

判法（文件头与格函数上方注释写全，读法在格里的 `wallclock_claims`）：那一行按「；」「;」「。」切段，每段认两种写法之一——
- 式子：`<数>[量词] × … × <数><时间单位> [÷ <数>] ≈|=|约 <挂钟>`，乘数里恰好一个带时间单位；
- 每：「每<量词>」后 40 字内的 `<数><时间单位>` 是单态耗时，前面最近的、量词开头相同的 `<数><量词>`（乘出来的一串算一个，有等号取右边）是态数，几对相加，与这一段最后一对之后最后写的时间比。
- 差一个数量级以上（写的上限 ×10 < 算出的下限，或写的下限 > 算出的上限 ×10）判红；一段都认不出的登记报「本次未判」并点名、往 `$GATE_NOT_RUN_FILE` 写一行，不算绿。

真仓读数（草稿里单跑格里那段 python，喂三份真登记，`cell-from-stage.py` 与阶段里嵌的逐字节相同，`cmp` 过）：

```
  ✓ 跑前登记的「并行与挂钟」：查了 3 份（research/prompts/e186-preregistration.md、research/prompts/e187-preregistration.md、research/prompts/e197-preregistration.md），五样都写全了；挂钟算式判了 3 条（3 份）
rc=0
```
三条各是：E186「17 条」乘「每条重编加跑测试约 1 分钟」比写的 5 分钟（算出 1020 秒，写的 300 秒）；E187 式子「960 × 10000 × 0.5 毫秒 ÷ 31 ≈ 155 秒」（算出 154.8 秒）；E197「65 次与包装起」乘「每次约 1 秒」比写的 8 分钟（65 秒对 480 秒）。

样本（逐字抄真仓三节，`setup.sh` 里写明取自哪几行：E186 第 650–658 行、E187 第 637–643 行、E197 第 332–341 行）：
- 绿 `preregistration-parallelism-wallclock-green`：三节原样，另一份 e994 只写散文。草稿里跑（`run-cell.sh`，建临时 git 仓跑 setup.sh、取改动清单、起格里那段 python）：

```
  ! 挂钟算式本次未判 1 份：「预计挂钟：」那一行写不出态数或单态耗时（或写了而没有挂钟可比）：research/prompts/e994-preregistration.md:8
  ✓ 跑前登记的「并行与挂钟」：查了 4 份（…e186…、…e187…、…e197…、…e994…），五样都写全了；号小于 186 不判的 1 份：research/prompts/e100-preregistration.md；挂钟算式判了 3 条（3 份）；挂钟算式本次未判 1 份（写不出态数或单态耗时）：research/prompts/e994-preregistration.md:8
rc=0
not-run file:
doc-experiments.sh preregistration-parallelism：挂钟算式本次未判 1 份（research/prompts/e994-preregistration.md:8）
```
（「…」是这里为短省掉的路径前缀，原样在 expect 里。）
- 红 `preregistration-parallelism-wallclock-red`：同三节各改坏一个数（E186 17 条→1700 条、E187 式子里 0.5 毫秒→50 毫秒、E197 65 次→6500 次）：

```
  ✗ research/prompts/e186-preregistration.md:8：预计挂钟与态数乘单态耗时差一个数量级以上：「1700 条」乘「每条重编加跑测试约 1 分钟」，与写的「5 分钟」比；算出约 28.3 小时，写的约 5 分钟
  ✗ research/prompts/e187-preregistration.md:8：预计挂钟与态数乘单态耗时差一个数量级以上：式子「960 × 10000 × 50 毫秒 ÷ 31 ≈ 155 秒」；算出约 4.3 小时，写的约 2.58 分钟
  ✗ research/prompts/e197-preregistration.md:8：预计挂钟与态数乘单态耗时差一个数量级以上：「6500 次与包装起」乘「每次约 1 秒」，与写的「8 分钟」比；算出约 1.81 小时，写的约 8 分钟
  ✗ 这次改动的跑前登记里 3 处预计挂钟与态数乘单态耗时差一个数量级以上（判了 3 条、3 份）
rc=1
```
- 原有两份样本照旧：`preregistration-parallelism-red` 退 1、「3 处没写全（查了 3 份）」不变（e991、e992 多报本次未判 2 份）；`preregistration-parallelism-green` 退 0，原 want 那一句原样是成功句的前缀，后面接「挂钟算式判了 0 条（0 份）；挂钟算式本次未判 1 份……e993…:12」。

弄坏开关（文件头 `EXPERIMENT_PAGES_AND_PRODUCTS_BREAK`，这一道整道在 `.claude/gate.d/stage-break-switch-replay-exempt.tsv` 里，复跑格不跑它；草稿里四份样本逐个核过）：
- 新 `parallelism-wallclock-unchecked`：wallclock-red 退 0（期望 1）；wallclock-green 少「判了 3 条」「本次未判 1 份」；原有两份不受影响。
- 新 `parallelism-unjudged-as-green`：wallclock-green 少「本次未判 1 份」两条 want；别的三份不受影响。
- 原有两条的点名改准：`parallelism-section-presence-only` 另让 wallclock-red（退 0）、wallclock-green（少「判了 3 条」）判错；`parallelism-old-registrations-judged` 另让 wallclock-green 退 1，原有的 preregistration-parallelism-red 也判错（「3 处、3 份」变成「4 处、4 份」，改前头注漏写了它）。

管不到的（写进格注释）：态数与单态耗时写在第六节或别的行、「预计挂钟：」里没重写的不读；「每」按量词开头配对，配错靠 review（E197 的「每槽」配不上任何态数，不判；「每次」配到最近的「65 次」）；没写除数的并行按串行乘积比。

## 二、③ doc-process-records 新格 deferred-items-registered

- 格名表加一行 `# gate-cell: deferred-items-registered records/ 下这次新增的带延后类词的行，同一行写着指得到的登记（C 号、拆分提案的节号与行首序号或 文件:行）`，`stage_cell` 登记与它逐字相同；`--list` 打出十行。
- 词表：文件头常量 `DEFERRAL_WORDS="延后 下轮开 下一轮开 下一批 留给 下一个题目"`（派发给的五个，加「下轮开」的全称「下一轮开」）。
- 对象：`gate_added_lines "$gate_base" 'records/*.md'`（经 `scripts/gate/changed-paths.sh`，不自己算 diff；新加 `load_added_records_directory_lines` 与父进程预取那一支）。去掉「」与反引号里的字后还带词的行；围栏里的、`records/2026-09-16-subagent拆分提案.md` 里的（登记处本身）不判。
- 判据：同一行至少一处指得到——① `C<数>` 在 `.claude/kb/checks-owed.md` 有「| C<数> |」起头的行；② 拆分提案「第<节号>节 <行首序号>」：那一节在、那一节有行首是那个序号的行（没写「拆分提案」时只认节号后紧跟序号的「九十节 ㉟」写法；写了「拆分提案」只写节号的，那一节没有带序号的行才算）；③ `<路径>:<行号>` 文件在树里、行号不超过行数。写了而都指不到的逐处说为什么，没写的点名「没写」。出路以「照 .claude/main-agent.md「一轮怎么开、怎么收」第 2 条在同一行写登记的编号或 文件:行」起头。
- 无对象退 77；欠账表或拆分提案读不到判红。
- `.claude/gate.d/stage-owners.tsv` 加一行：deferred-items-registered 归 investigator,tooling-writer（记录由主 agent 补，判法与词表归工具实现员）。

样本（草稿里同一个 `run-cell.sh` 跑，基准提交是 HEAD，与 stage-selftest 摘掉 GATE_BASE 后的取法一样）：
- 绿 `deferred-items-registered-green`：

```
  ✓ records/ 下这次新增的带延后类词的行：判了 5 行，都写着指得到的登记（C 号、拆分提案的节号与序号或 文件:行；records/ 新增的 .md 行共 12 行）；不判的：词只在「」或反引号里的 1 行、围栏里的 1 行、拆分提案（登记处本身）里的 1 行
rc=0
```
- 红 `deferred-items-registered-red`：

```
  ✗ records/2026-10-22-样本调度.md:5：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
  ✗ records/2026-10-22-样本调度.md:6：带「下一批」，写的登记指不到——C 号「C99」指不到：.claude/kb/checks-owed.md 里没有「| C99 |」这一行
  ✗ records/2026-10-22-样本调度.md:7：带「下轮开」，写的登记指不到——拆分提案节号与序号「第九十节 ⑤」指不到：records/2026-09-16-subagent拆分提案.md 第 90 节里没有行首是「⑤」的一行
  ✗ records/2026-10-22-样本调度.md:8：带「留给」，写的登记指不到——拆分提案节号「第九十节」指不到：那一节有行首序号（2 行），要写是哪一行
  ✗ records/2026-10-22-样本调度.md:9：带「下一个题目」，写的登记指不到——文件:行「research/notes.md:99」指不到：research/notes.md 只有 30 行
  ✗ records/2026-10-22-样本调度.md:10：带「延后」，写的登记指不到——文件:行「gone.md:3」指不到：gone.md 不在树里（只写文件名的要写从仓根起的路径）
  ✗ records/ 下这次新增的带延后类词的行里 6 行没登记去向（判了 7 行）
     → 怎么办：照 .claude/main-agent.md「一轮怎么开、怎么收」第 2 条在同一行写登记的编号或 文件:行（…）
rc=1
```

弄坏开关 `DEFERRED_ITEMS_BREAK=<项>`（文件头那一段，下一行写判错的样本；草稿里两份样本逐个核过，每条正好让一份判错）：
- `pointers-unresolved-accepted`：red 只剩第 5 行一条，「6 行」与第 6–10 行的 want 找不到；green 不变。
- `quotes-counted`：green 退 1（「判了 6 行」里 1 行没登记）；red 不变。
- `registry-rows-judged`：green 退 1；red 不变。
- `fenced-lines-judged`：green 退 1；red 不变。
共用库那两条（`STAGE_CELLS_BREAK=red-swallowed`、`=marker-ignored`）的点名清单里各补上新样本（red-swallowed 补 red，marker-ignored 补两份），「九格」改「十格」，「用改动范围的六格」改「七格」。

## 三、㉟ stage-owners（2.22）

`.claude/gate.d/stage-owners.tsv` 第 107 行归属改成 `experiment-runner,implementation-writer,experiment-designer`，理由一格前面加「重跑登记里固定形态的比对对象表归设计员（它写表，写完到执行员登记之前这一格红，设计员报告里写明、交执行员登记；表头与七格写法不对的由设计员改）」。`.claude/agents/experiment-designer.md` 在，stage-owners 格的 ③ 判得过（定义文件在）。

## 四、㉟ claim-experiment 开关改名、进复跑表（2.19、2.20）

改名前全仓 `grep -rn 'CLAIM_SKIP_'`（排掉 target、.git）：脚本本身第 25、26、50、116、163 行；别处全是记录与三方材料里引旧名的历史文字（`records/2026-10-22-子agent结果落地审计.md:133`、`records/2026-09-16-subagent拆分提案.md:1506`、`research/prompts/` 下 landing-audit 各轮输出、快照 diff、`tw-gate-scripts.md`、`b9-gates-b.md`）。它们是当时的证据、不在写范围（`research/prompts/` 禁写，`records/` 不在这一批的文件里），没改；代码里引它的只有脚本自己，改完全仓 `scripts/`、`.claude/` 下 0 处。

改法（`scripts/kb/claim-experiment.sh`）：
- 第 52 行 `worktree_roots` 改读 `CLAIM_EXPERIMENT_BREAK == worktrees-skipped`；第 118 行 `claim` 改读 `!= used-check-skipped`。
- 删掉自证里「关掉查号确认判红并退 0」那一支（复跑格认退 0 为「没红」）；自证 18 处失败改调 `selftest_fail`：打「  ✗ selftest: …」与一行「→」再 `return 1`（复跑格只在带 ✗ 的行里找点名的项）。
- 文件头开关表照 mutate.sh 写法逐条三样：

```
#   CLAIM_EXPERIMENT_BREAK=used-check-skipped  claim() 跳过「已经有人用了」那一判：已有源码的号被占成功
#     查号那一格（「✗ selftest: E6 已有源码却没被拒」）
#   CLAIM_EXPERIMENT_BREAK=worktrees-skipped  worktree_roots() 不读工作树清单、只给给定的那一棵：另一棵里用过的号被占成功、--next 少数
#     跨工作树那一格（「✗ selftest: 另一棵工作树用过 E12，本棵 --next 该是 E13」）
#   CLAIM_EXPERIMENT_BREAK=registry-ignored  查号不读目录登记，退回改前写死的那组目录（…）
#     目录登记那一格（「✗ selftest: 目录登记：」）
```
- 平常跑的守卫（第 37 行起，原有）三个值都走它，拒绝说明改成列三个值。
- `.claude/gate.d/break-switch-replay-files.tsv` 加一行 `scripts/kb/claim-experiment.sh	60	# 三条开关逐个跑过：……（单跑一条 0.1 到 0.5 秒，不开关整份 0.5 秒）`。实测最慢 0.46 秒，两倍取整到表里通用的 60 秒。selftest-registry 里它本来就是 light（第 46 行）。

自证实测（主工作区，`nice -n 19`）：

```
== plain rc=0 secs=.445644534
selftest: 通过（--next 取到 E9；已有源码的 E6 被拒；…）
== used-check-skipped rc=1 secs=.098189224
  ✗ selftest: E6 已有源码却没被拒
== worktrees-skipped rc=1 secs=.206034262
  ✗ selftest: 另一棵工作树用过 E12，本棵 --next 该是 E13，拿到 E6
== registry-ignored rc=1 secs=.458602982
  ✗ selftest: 目录登记：入库装置目录里有 E40、登记里还没有 notes，--next 该是 E41，拿到 E1
== guard
  ✗ CLAIM_EXPERIMENT_BREAK=used-check-skipped 只在 --selftest（自证）里认：…
rc=2
```

复跑格判法（code-tooling 的 break-switch-replay 格那段 python 原样抽进草稿 `claim/replay-cell.py`，登记表换成只有这一行的 `one-row.tsv`，不经门禁阶段）：
- 改后（主工作区）：`✓ 脚本与钩子的弄坏开关复跑：复跑了 3 条开关（1 份），自证红且点名的项都在失败行里 3 条、自证自带开关轮 0 条；…` rc=0
- 改前那份（草稿最小树里换回开工时的拷贝）：`✗ scripts/kb/claim-experiment.sh CLAIM_EXPERIMENT_BREAK=registry-ignored：不符，点名的项没在失败行里：「selftest: 目录登记：」` / `✗ …1 处不对（复跑了 1 条、1 份）` rc=1（改前的 `CLAIM_SKIP_*` 不是 `_BREAK` 名字，格认不出，只复跑了一条）
- break-switch-headers 格的读法（`switch_entries`）读新文件头：三条，`incomplete` 都是空表。

## 五、③ 在真仓上的读数（HEAD 与此刻工作区之间 records/ 新增的行）

命令（不经门禁阶段；新增行经共用取法，格里那段 python 原样，`deferral/cell-from-stage.py` 与阶段里嵌的逐字节相同）：

```
bash -c 'source scripts/gate/changed-paths.sh && gate_added_lines HEAD "records/*.md"' > deferral/added-head.tsv   # rc=0，2494 行
python3 deferral/cell.py deferral/added-head.tsv "延后 下轮开 下一轮开 下一批 留给 下一个题目" ""                   # rc=1
```
汇总行原样：
```
  ✗ records/ 下这次新增的带延后类词的行里 107 行没登记去向（判了 162 行）
```
按文件数（`grep '^  ✗ records/2' | sed … | sort | uniq -c`）：
```
      2 records/2026-10-13-代码欠账批.md
      2 records/2026-10-13-复用键与全仓改名.md
     26 records/2026-10-13-里程碑四预开发.md
     40 records/2026-10-18-里程碑三收尾与近期欠账.md
     11 records/2026-10-19-里程碑四GPU垃圾回收开工.md
     10 records/2026-10-21-doc门禁违规修复.md
      1 records/2026-10-21-仓外目录不进仓.md
     14 records/2026-10-22-子agent结果落地审计.md
      1 records/2026-10-23-门禁自证覆盖格修正.md
```
注意：这 2494 行里 1885 行落在 HEAD 里已有的记录文件上（按文件是否在 HEAD 数的，改一行在 diff 里也算新增），609 行落在这一轮新建的记录文件上；红行里有多少是改写旧行带出来的没逐行分。真仓上的红交主 agent 判（补登记、改写那一行，或认定词表、判法要收窄），`records/` 没改。

逐行（红行原样，第一部分）：
- records/2026-10-13-代码欠账批.md:175：带「留给」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-代码欠账批.md:217：带「留给」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-复用键与全仓改名.md:979：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-复用键与全仓改名.md:989：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:227：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:277：带「留给」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:337：带「下轮开」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:706：带「下一批」，写的登记指不到——文件:行「gpu_unit_checks.rs:231」指不到：gpu_unit_checks.rs 不在树里（只写文件名的要写从仓根起的路径）
- records/2026-10-13-里程碑四预开发.md:747：带「下一批」，写的登记指不到——文件:行「crash_enumeration_new_pool_file_creation_stream.rs:363」指不到：crash_enumeration_new_pool_file_creation_stream.rs 不在树里（只写文件名的要写从仓根起的路径）
- records/2026-10-13-里程碑四预开发.md:751：带「留给」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:838：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:861：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:864：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:891：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:896：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:914：带「下一批」，写的登记指不到——拆分提案节号「第八十七节」指不到：那一节有行首序号（1 行），要写是哪一行
- records/2026-10-13-里程碑四预开发.md:917：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:932：带「留给」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:939：带「留给」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:940：带「留给」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:957：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:960：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:962：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:967：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:968：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:971：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:972：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:978：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:986：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-13-里程碑四预开发.md:993：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:179：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:226：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:696：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:708：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:722：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:742：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:758：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:771：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:774：带「下一批」，写的登记指不到——文件:行「crash_injection.rs:9029」指不到：crash_injection.rs 不在树里（只写文件名的要写从仓根起的路径）
- records/2026-10-18-里程碑三收尾与近期欠账.md:780：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:842：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:843：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:903：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:910：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:911：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:958：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:978：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:998：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1024：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1056：带「留给」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1069：带「留给」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1093：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1192：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1193：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1202：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行

逐行（第二部分）：
- records/2026-10-18-里程碑三收尾与近期欠账.md:1216：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1237：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1283：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1289：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1300：带「留给」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1319：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1326：带「延后」「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1338：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1353：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1368：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1376：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1418：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1486：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1525：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-18-里程碑三收尾与近期欠账.md:1546：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-19-里程碑四GPU垃圾回收开工.md:158：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-19-里程碑四GPU垃圾回收开工.md:159：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-19-里程碑四GPU垃圾回收开工.md:160：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-19-里程碑四GPU垃圾回收开工.md:185：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-19-里程碑四GPU垃圾回收开工.md:200：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-19-里程碑四GPU垃圾回收开工.md:273：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-19-里程碑四GPU垃圾回收开工.md:276：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-19-里程碑四GPU垃圾回收开工.md:295：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-19-里程碑四GPU垃圾回收开工.md:304：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-19-里程碑四GPU垃圾回收开工.md:323：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-19-里程碑四GPU垃圾回收开工.md:347：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-21-doc门禁违规修复.md:65：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-21-doc门禁违规修复.md:99：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-21-doc门禁违规修复.md:148：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-21-doc门禁违规修复.md:160：带「下轮开」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-21-doc门禁违规修复.md:166：带「下轮开」，写的登记指不到——拆分提案节号「第八十一节」指不到：那一节有行首序号（7 行），要写是哪一行
- records/2026-10-21-doc门禁违规修复.md:170：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-21-doc门禁违规修复.md:172：带「下轮开」，写的登记指不到——拆分提案节号「第八十一节」指不到：那一节有行首序号（7 行），要写是哪一行
- records/2026-10-21-doc门禁违规修复.md:203：带「下轮开」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-21-doc门禁违规修复.md:204：带「下轮开」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-21-doc门禁违规修复.md:207：带「下轮开」，写的登记指不到——文件:行「crash-amplification-two-hosts.sh:328」指不到：crash-amplification-two-hosts.sh 不在树里（只写文件名的要写从仓根起的路径）
- records/2026-10-21-仓外目录不进仓.md:41：带「下轮开」「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-22-子agent结果落地审计.md:48：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-22-子agent结果落地审计.md:87：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-22-子agent结果落地审计.md:94：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-22-子agent结果落地审计.md:104：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-22-子agent结果落地审计.md:115：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-22-子agent结果落地审计.md:127：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-22-子agent结果落地审计.md:140：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-22-子agent结果落地审计.md:170：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-22-子agent结果落地审计.md:212：带「下轮开」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-22-子agent结果落地审计.md:262：带「下轮开」「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-22-子agent结果落地审计.md:264：带「下轮开」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-22-子agent结果落地审计.md:278：带「留给」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-22-子agent结果落地审计.md:284：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-22-子agent结果落地审计.md:323：带「下轮开」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行
- records/2026-10-23-门禁自证覆盖格修正.md:23：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号或 文件:行

## 六、要主 agent 跑的命令

工作目录都是主工作区仓根。新样本没暂存，钩子拒子 agent 喂它们（拒绝原文：「doc-process-records.sh 的样本与 … HEAD 加暂存区不同（6 处：deferred-items-registered-green/.gate-cells（暂存区里没有）…）」），所以下面这些先暂存这一批的文件再跑：

1. 样本档（两道都登记成 samples-light）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=stage-selftest.sh 喂 doc-experiments.sh 与 doc-process-records.sh 的样本; 范围=2 道阶段的样本档; 为什么=核第十二批任务 41 新加的挂钟算式与 deferred-items-registered 两格' bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item doc-experiments.sh --item doc-process-records.sh`
   看末行「判得都对」；新加的四份与 preregistration-parallelism-red / -green 两份要判对。
2. 新开关逐个打开（每条只该让点名的样本判错；同一条命令前面加开关）：
   - `EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=parallelism-wallclock-unchecked`、`=parallelism-unjudged-as-green`、`=parallelism-section-presence-only`、`=parallelism-old-registrations-judged`，带 `--item doc-experiments.sh`；判错的应当正好是文件头那几行点名的。
   - `DEFERRED_ITEMS_BREAK=pointers-unresolved-accepted` 判错 deferred-items-registered-red；`=quotes-counted`、`=registry-rows-judged`、`=fenced-lines-judged` 各判错 deferred-items-registered-green；带 `--item doc-process-records.sh`。
3. code-tooling 几格（这一批碰到的）：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 stage-owners、gate-structure、break-switch-headers、break-switch-replay、break-switch-outside-selftest、research-gate-lint、fixture-claims 七格; 范围=7 格; 为什么=核第十二批任务 41 改的归属表、格名表、claim-experiment 开关与新样本' bash .claude/gate.d/code-tooling.sh --check stage-owners,gate-structure,break-switch-headers,break-switch-replay,break-switch-outside-selftest,research-gate-lint,fixture-claims`
   break-switch-replay 格要报 claim-experiment 那 3 条都「对」（这一格整表复跑，慢）。
4. 真仓单跑新格看读数（会红，红行就是第五节那些，基准换成 gate 取法后条数可能不同）：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-process-records.sh 的 deferred-items-registered 格; 范围=1 格; 为什么=看真仓 records/ 里延后行登没登记' bash .claude/gate.d/doc-process-records.sh --check deferred-items-registered`
5. 改动的门禁与脚本归提交前那一趟 gate-lint、shell-lint、preflight-lint、gate-overlap（`scripts/gate/gate-staged.sh`）。

## 七、没做什么

- 没跑重型测试（`checker-tier-crash-point-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`）；没走定义三方；没提交、没暂存。
- 没在主工作区跑门禁阶段与 lint；两道阶段的样本档被钩子拒（新样本没暂存），只在草稿里单跑了格里那段 python（与阶段里嵌的逐字节相同），阶段的 shell 那一层（预取、进程替换、stage-cells 汇总行）没跑过，交第六节第 1 条。
- 没跑命名检查：这一批没改 `.rs`。
- 没改 `records/`：真仓 107 行红交主 agent；`records/2026-10-22-子agent结果落地审计.md:133` 与 `research/prompts/` 下引旧名 `CLAIM_SKIP_*` 的历史文字没改（证据，不在写范围）。
- 没改 `.claude/agents/experiment-designer.md`：挂钟算式认的两种写法（式子、「每」）没写进设计员定义；现在三份真登记都认得出，新登记写成散文的只报本次未判、不红。要不要把写法写进定义，交主 agent 定。
- 词表只放了派发给的五个词加「下一轮开」；「推迟」「以后再」这类没加（加之前要先量真仓会多红几行）。
- 「九十节各行」这种写法（序号写在别的格里）不认，真仓红行里有两行是这种（落地审计第 262、321 行），交主 agent 定是改写还是放宽判法。
- 拆分提案第九十节 ㉟ 里别的件（看门狗 ⑭、合入后验证、㉑ 混合大小写、`dispatch-prompt.py` 文件头等）不在这一批，没碰。

## 草稿目录

`~/.local/share/singlefs/work/landing-audit-b12-gates-a/`：`originals/`（开工时五份的拷贝与清单）、`claim/`（claim-experiment 的改法、自证输出、复跑格抽出的判法与读数）、`wallclock/`（挂钟算式的原型、格的 python、样本源、跑样本的脚本）、`deferral/`（新格的 python、样本源、真仓新增行 `added-head.tsv` 与读数 `real-head.out`）、`gate-overlap-list.txt`。没有仓副本、编译目录与库拷贝（建过的最小树与临时 git 仓都在用完时删了）。
