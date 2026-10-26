# landing-audit-r1 修补「门禁与扫描」那一份：交回报告（tooling-writer，2026-10-22）

批次：records/2026-10-22-子agent结果落地审计.md#5。判决：research/prompts/landing-audit-r1-main-verification.md「二、逐格判」K2 表、K3-6，「三、判决」第一张表「门禁与扫描」，「四、交用户」（用户定「七处一起并」）。

## 结论（五条各自的出口）

| 条 | 出口 | 状态 |
|---|---|---|
| K2-1 七处并成一份库 | 库有自证与弄坏开关；七处改调它；各处自证与样本照旧绿；开关打开后至少一道门禁的样本红在点名那一格 | 库自证 8 项绿、开关打开 3 项红（实跑）；七处都改调库；样本在草稿里逐段核过绿；开关打开后 code-source-discipline 的 reading-discipline-red 那条 want 找不到（草稿里核的）。stage-selftest 被闸拒（样本与暂存区不同），交主 agent 跑 |
| K2-2 删新格的滞后表 | 删表与四样判法、一个开关，只缩不涨改比登记表的基准，三份样本跟着改 | 做完；红、绿、零三份样本在草稿里核过退出码与 want；七个 comparison-* 开关逐个在草稿里核过 |
| K2-3 复跑表一份读法 | replay_rows 进库，doc-experiments 四处都调它，今天 replay.sh 上四处取到的行与改前逐行相同 | 做完；四处各 254 行逐行相同（命令与输出在下面） |
| K2-5 候选扫描读登记位 | CARRIER_PATTERNS 两行改成读 .claude/kb-directories；开关改成退回写死的 .claude/kb/，自证红在原来那一项 | 做完；自证绿、开关打开红在原来那一项、--benchmark 读数不变（4190 行） |
| K3-6 登记表归执行员、补全单元 | stage-owners 那一行改归执行员；6 个缺的单元逐个判 | 做完；6 个都不是 r5 比对对象表里的比对对象，登记表文件头逐个写了不登记的理由（一个一行），没加登记行 |

推翻条件：
- 主 agent 暂存之后跑 stage-selftest（命令在「要主 agent 跑的命令」）有一份样本判错，或 `LAG_BASELINE_BREAK=unreadable-as-absent` 下 code-source-discipline 判错的样本集合不正好是 {reading-discipline-red}，K2-1 的出口就不算达成。
- 真仓单跑 doc-experiments 的 results-cited、verdict-false-named、quoted-result-lines 三格的读数与改前不同（例如「查了多少」变了），K2-3 不算达成。
- 有人指出 r5 档、r3 合法那 6 个单元其实是某份重跑登记的比对对象（比对对象表里有它们），K3-6 那 6 行不登记的理由就站不住。

## 改过的文件

`git diff --stat`（相对暂存区，只含这一批的改动；四道门禁与 lib_long_run.py、stale-candidates.py 上别的会话已暂存的改动不在里面）原样：

```
 .claude/gate.d/code-source-discipline.sh           |  83 ++++----
 .claude/gate.d/code-tooling.sh                     |  36 ++--
 .claude/gate.d/doc-experiments.sh                  | 233 ++++++++-------------
 .claude/gate.d/doc-process-records.sh              |  32 ++-
 .../.claude/gate.d/replay-comparison-units-lag.tsv |   2 -
 .../replay-comparison-units-green/expect           |   2 +-
 .../replay-comparison-units-green/setup.sh         |   4 +-
 .../.claude/gate.d/replay-comparison-units-lag.tsv |   4 -
 .../replay-comparison-units-red/expect             |   7 +-
 .../replay-comparison-units-red/setup.sh           |  13 +-
 .claude/gate.d/replay-comparison-units-lag.tsv     |   8 -
 .claude/gate.d/replay-comparison-units.tsv         |  11 +-
 .claude/gate.d/stage-owners.tsv                    |   2 +-
 .claude/hooks/lib_long_run.py                      |  35 ++--
 scripts/kb/stale-candidates.py                     |  37 +++-
 15 files changed, 223 insertions(+), 286 deletions(-)
```

另有新建（没进 git，`git status` 显示 `??`）：`.claude/gate.d/lib-lag-baseline.py`，268 行。

删的三份（都已在暂存区，`git show :<路径>` 取得回原样）：`.claude/gate.d/replay-comparison-units-lag.tsv`、两份样本里的 `.claude/gate.d/replay-comparison-units-lag.tsv`（red、green）。

开工与每次写之前 `git diff --stat` 看过：这几份文件在这一批开工时没有未暂存的改动，写的过程中也没有新出现别人的未暂存改动（上表的每个 hunk 都是这一批的）。

改法：门禁、库与 setup.sh 这些已存在的脚本一律经 `scripts/kb/replace-once.py` 定点改（换 inode）；第一处（doc-experiments.sh 加 `LIB_LAG_BASELINE` 那一行）用的 Edit 工具（同样换 inode）；`lib_long_run.py` 经 `scripts/gate/hook-swap.py`（草稿验过、换上、主工作区再验过，见 K2-1 第 7 处）；两份 setup.sh 写同目录 `.new` 再 `mv`；expect、tsv 用 Edit。

## K2-1　`.claude/gate.d/lib-lag-baseline.py`（新）与七处

库（文件头写全了用法、函数、命令行、开关）：
- `content_rows(text)`：表的内容行 [(行号, 各列)]，去掉行首空白后以 # 起头的行与空行不算，行尾 \r 去掉，按制表符切。
- `baseline_text(base, path, cwd=None)` / `baseline_rows(...)`：三种结局 (全文或内容行, None)、(None, None) 基准里没有这张表、(None, BaselineProblem) git 取不到；`BaselineProblem.sentence(stderr_limit=80)` 给「取不到基准 … 里的 …（git ls-tree 退 N：…）」或「读不出基准 …（git show 退 N：…）」，各处原来的句式都保住了（code-source-discipline 的 reading-discipline 用 `sentence(None)`，不带 stderr 那一段，stderr 另起一行列）。
- `replay_rows(path)`（K2-3）。
- 命令行：`show <基准> <路径>`（退 0 全文打 stdout、退 3 基准里没有、退 1 问题句打 stderr）、`replay-rows <路径>`、`--selftest`；开头调 `preflight`（`admission: always`、`run-condition: command git`），所以不用登记进 `.claude/preflight-exclude`（那张表不在我写范围里）。
- 新写的 `# gate-similar:` 两行原样：
  - `# gate-similar: lib-item-ref-status.py 它管按编号取 kb 文件与条目引用的状态，读的是 kb 正文；这一份管「滞后表与基准那一版比」与复跑表的行，读的是登记表与 git 基准，不并进它`
  - `# gate-similar: lib-owed.py 它读欠账表里哪些还开着，不碰 git 基准；这一份取基准那一版的表，不并进它`
- 没有 `# hook-events:`（不是钩子）。

七处，每处删了什么、留了什么（行号是改后现查的）：

| 处 | 删的 | 留的（键与报法） |
|---|---|---|
| 1 doc-experiments.sh multipath-registry 格（`.claude/gate.d/doc-experiments.sh:2013` 起那一段 python） | `lag_rows` 函数、`ls-tree` / `show` 两段 subprocess（14 行）、`import subprocess` | 按实验编号当键；报「滞后表涨了」与问题句后接「，只缩不涨没比」 |
| 2 doc-experiments.sh replay-comparison-units 格 | `rows_of`、`replay_table`、`ls-tree` / `show` 两段，连同 K2-2 删的滞后表判法 | 见 K2-2 |
| 3 doc-process-records.sh 点名一节挂账表（`check_review_pending_table`，`:810` 起） | `rows_of`、`ls-tree` / `show` 两段、`import subprocess` | 第一列判决路径当键；问题句后接「，只缩不涨没比」 |
| 4 code-source-discipline.sh mutation-tables 格的重复行滞后表（`duplicates_lag_judge`） | `parse_rows` 里自己切行的四行、`ls-tree` / `show` 两段、`import subprocess` | 按组（表与名字集合）比；`parse_rows` 只留形状判法，切行调 `content_rows` |
| 5 code-source-discipline.sh reading-discipline 格（`read_lag`、`baseline_counts`） | `read_lag` 自己切行那几行、`baseline_counts` 里 `ls-tree` / `show` 两段与自己切行 | 按文件计数比；`reject` 的句式不变（「读不出基准 HEAD 里的 …（git show 退 128），「只缩不涨」这一条没比：」），两种失败的出路并成一句 |
| 6 code-tooling.sh heavy-thread-defaults 格（bash） | `git ls-tree` / `git show` 那 6 行 | 改调 `python3 "$STAGE_DIRECTORY/lib-lag-baseline.py" show …`，退 0 / 3 / 别的映射成原来的 `base_state` 三种；python 那一段红行直接用库给的问题句 |
| 7 code-tooling.sh long-run-header-exempt 格（bash）与 `.claude/hooks/lib_long_run.py` | bash 里 `ls-tree` / `show` 与两种失败出口（14 行）、`mktemp` 的临时表与 trap；`lib_long_run.py exempt-check` 的 `--base-table <文件>` | bash 只传 `--base "$base_name"`；`exempt_main` 经 `LAG_BASELINE_LIBRARY`（`.claude/hooks/../gate.d/lib-lag-baseline.py`）的 `baseline_text` 在仓根里取，基准里没有不比、取不到退 1 并给出路；按「路径 + sha256」比的判法不动 |

另找到第八处同一写法，没改（不在派发列的七处里）：doc-experiments.sh decision-links 格的「待回填」表 `.claude/decision-links-pending`（`.claude/gate.d/doc-experiments.sh:1069` 那一行 `listed = git('ls-tree', '--name-only', base, '--', pending_rel)`）。要并的话下一批改。

### 库的自证与弄坏开关（实跑，命令与输出原样）

```
$ python3 .claude/gate.d/lib-lag-baseline.py --selftest
  ✓ 表的内容行
  ✓ 基准里有这张表
  ✓ 基准里没有这张表
  ✓ 基准名取不到
  ✓ 基准那一版的对象缺了
  ✓ 命令行三种结局
  ✓ 复跑表的行
  ✓ 认不出的弄坏开关被拒
  ✓ lib-lag-baseline.py 自证：8 项都过
rc=0
$ LAG_BASELINE_BREAK=unreadable-as-absent python3 .claude/gate.d/lib-lag-baseline.py --selftest
  ! 弄坏开关 LAG_BASELINE_BREAK=unreadable-as-absent 生效：取不到基准 no-such-base 里的 lag.tsv（git ls-tree 退 128：fatal: Not a valid object name no-such-base），当成基准里没有这张表
  ! 弄坏开关 LAG_BASELINE_BREAK=unreadable-as-absent 生效：读不出基准 HEAD 里的 lag.tsv（git show 退 128：fatal: bad object HEAD:lag.tsv），当成基准里没有这张表
  ✓ 表的内容行
  ✓ 基准里有这张表
  ✓ 基准里没有这张表
  ✗ 自证失败：基准名取不到：取到 None、None
  ✗ 自证失败：基准那一版的对象缺了：取到 None、None
  ✗ 自证失败：命令行三种结局：有表 0、没有表 3、取不到 3（stderr 'G_BASELINE_BREAK=unreadable-as-absent 生效：读不出基准 HEAD 里的 lag.tsv（git show 退 128：fatal: bad object HEAD:lag.tsv），当成基准里没有这张表'）
  ✓ 复跑表的行
  ✓ 认不出的弄坏开关被拒
  ✗ lib-lag-baseline.py 自证：8 项里 3 项没过
     → 怎么办：照上面点名的那一项修 baseline_text / content_rows / replay_rows；设了 LAG_BASELINE_BREAK 时这几项该红
rc=1
```
（这一份是改成内联 git 调用之前跑的；改了之后又跑了一次不带开关的，末行仍是「✓ lib-lag-baseline.py 自证：8 项都过」。）

开关在门禁里的样本：code-source-discipline.sh 文件头开关表加了一条（mutate.sh 写法，用 code-tooling 的 `BREAK_SWITCH_TABLE_LIBRARY` 解析过：第 124 行、三样不缺、点名的样本 {reading-discipline-red}）：
```
#   LAG_BASELINE_BREAK=unreadable-as-absent  归 lib-lag-baseline.py（…）：git 取不到基准那一版的滞后表、豁免表当成「基准里没有这张表」，只缩不涨静默不比
#       reading-discipline-red（输出里找不到「读不出基准 HEAD 里的 .claude/gate.d/experiment-seed-fold-lag.tsv（git show 退 128）」）
```
草稿里核：把 reading-discipline 那一段 python 从阶段里取出来，在样本拷贝上（跑过 setup.sh）照阶段的参数起，比 expect 的 want（阶段汇总那几条不在 python 输出里，不比）：
```
== reading-discipline-green：退出码 0（expect 写 0），找不到的 want 1 条      ← 那 1 条是「reading-discipline：绿」，阶段汇总行
== reading-discipline-red：退出码 1（expect 写 1），找不到的 want 1 条        ← 同上，「reading-discipline：红（退出 1）」
--- 开关 LAG_BASELINE_BREAK=unreadable-as-absent
== reading-discipline-green：退出码 0（expect 写 0），找不到的 want 1 条
  [缺 want] 读不出基准 HEAD 里的 .claude/gate.d/experiment-seed-fold-lag.tsv（git show 退 128），「只缩不涨」这一条没比
== reading-discipline-red：退出码 1（expect 写 1），找不到的 want 2 条
```
同一道里别的样本不受这个开关影响的依据：开关只改「git 取不到」那一种结局；code-source-discipline 的样本里删对象库的只有 reading-discipline-red、absolute-assertions-red、mutation-tables-red 三份的 setup.sh，absolute-assertions 格不读滞后表，mutation-tables-red 没有重复行滞后表（`rows` 为空不取基准）。这一条是读出来的，要 stage-break-switch-replay 实跑确认（命令在后面）。doc-experiments、doc-process-records、code-tooling 三道没有一份样本会因这个开关红：doc-experiments 的头注用散文写明了这一点（不写成开关表的写法，免得复跑格读到一条没有点名样本的开关）；doc-process-records、code-tooling 两道的头注没加这一句（doc-experiments、code-tooling 在 `stage-break-switch-replay-exempt.tsv` 里本来就整道不复跑）。

### 七处的样本（草稿里逐段核，阶段本身没跑）

做法：`extract_heredoc.py` 从改后的阶段里取出那一段 python，在样本拷贝上（跑过 setup.sh，样本根是 git 仓就把基准写 HEAD）照阶段的参数起，比 expect 的退出码与 want（阶段汇总、「样本只跑…」、出路尾行这几种不在 python 输出里，不比）。脚本在草稿目录：`run_snippet.sh`、`run_cell_sample.sh`、`run_long_run.sh`、`extract_heredoc.py`、`compile_heredocs.py`。

```
== multipath-registry-green：退出码 0（expect 写 0），找不到的 want 0 条
== multipath-registry-red：退出码 1（expect 写 1），找不到的 want 0 条
== verdict-false-named-green：退出码 0（expect 写 0），找不到的 want 0 条
== verdict-false-named-gz-green：退出码 0（expect 写 0），找不到的 want 0 条
== verdict-false-named-gz-red：退出码 1（expect 写 1），找不到的 want 0 条
== verdict-false-named-no-history-red：退出码 1（expect 写 1），找不到的 want 0 条
== verdict-false-named-red：退出码 1（expect 写 1），找不到的 want 0 条
== verdict-false-named-translation-green：退出码 0（expect 写 0），找不到的 want 0 条
== verdict-false-named-zero：退出码 1（expect 写 1），找不到的 want 0 条
== crates-adversarial-review-pending-table-red：退出码 1（expect 写 1），找不到的 want 0 条
== crates-adversarial-review-pending-grown-red：退出码 1（expect 写 1），找不到的 want 0 条
== crates-adversarial-review-pending-green：退出码 0（expect 写 0）        ← 挂账表那一段只判表，没输出；缺的 3 条 want 是那一格别的段打的
== agent-def-adversarial-review-pending-green：退出码 0（expect 写 0）     ← 同上
== long-run-header-exempt-base-green（--base HEAD）：退出码 0（expect 写 0），找不到的 want 0 条
== long-run-header-exempt-green（不带基准）：退出码 0（expect 写 0），找不到的 want 0 条
== long-run-header-exempt-red（--base HEAD）：退出码 1（expect 写 1），找不到的 want 0 条
== long-run-header-exempt-repin-red（--base HEAD）：退出码 1（expect 写 1），找不到的 want 0 条
== long-run-header-exempt-sha256-red（不带基准）：退出码 1（expect 写 1），找不到的 want 0 条
```
long-run 两个开关照头注：`LIB_LONG_RUN_BREAK=exempt-growth-unchecked` 下 -red 缺「豁免表涨了」与「有 5 处不对（共 6 行）」两条、-repin-red 退 0；`exempt-repin-unchecked` 下 -repin-red 退 0、别的不变（实跑输出在草稿里，与 code-tooling.sh 文件头那两条说的一致）。

mutation-tables 的重复行滞后表：在 mutation-tables-duplicates-lag-red 的拷贝上手喂两组重复行，输出里有 `LAG_PROBLEM	.claude/gate.d/mutation-duplicates-lag.tsv:3 相对基准那一版多出一组（scripts/mutation/tables/e1_sample.tsv：M1_样本；M2_同一改法另起的名字）：这张表只缩不涨`、`LAG_BASELINE	与基准 HEAD 那一版按组比，多出 1 组`；基准名写成不在的，出 `取不到基准 no-such-base 里的 …（git ls-tree 退 128：…），只缩不涨没比`。

heavy-thread-defaults 的 bash 映射（heavy-thread-defaults-red 的拷贝上）：
```
HEAD .claude/heavy-thread-exempt -> table（基准表 3 行）
HEAD .claude/not-there -> none（基准表 0 行）
no-such-base .claude/heavy-thread-exempt -> unreadable:取不到基准 no-such-base 里的 .claude/heavy-thread-exempt（git ls-tree 退 128：fatal: Not a valid object name no-such-base）（基准表 0 行）
```

真仓上的读数（草稿里起那一段 python，基准 = `gate_diff_base gate` = e1b1ceb95a8b8c38bcd59e2e3ae9cc921575f865）：
```
  ✓ 登记了路径与共用项的实验页判过了（28 份、144 条路径）；还没写登记节的 9 份在 multipath-registry-lag.tsv 里、每一份都现存且确实还没写，与基准 e1b1ceb95a8b8c38bcd59e2e3ae9cc921575f865 比没涨；逐份补齐后删行
  ✓ 比对对象 14 行（E165）：登记出处都在，单元名在单元表出处里都恰好出现一次；复跑写驱动名的 10 行在 replay.sh 里都有且产物名相符；复跑写 none 的 4 行只缩不涨，基准里还没有这张表（或不是 git 仓），只缩不涨这一条没比
$ python3 .claude/hooks/lib_long_run.py exempt-check /home/fy5090/code/singlefs --base HEAD
  ✓ .claude/long-run-live-hosts-exempt 6 行：每行的脚本都在、登记的 sha256 都对得上，文件头都得靠现算读法才齐（按登记值读不齐）；与基准那一版比没涨
```

语法：四道门禁 `bash -n` 过；内嵌 python 逐段编译（`compile_heredocs.py`）：doc-experiments 8 段、doc-process-records 6 段、code-source-discipline 9 段、code-tooling 20 段，编译不过 0 段；`duplicates_lag_judge`（`$(cat <<'PY'` 那种写法，上面的脚本认不出）另取出来 `py_compile` 过；`lib-lag-baseline.py`、`stale-candidates.py` `py_compile` 过。

`lib_long_run.py` 经 hook-swap 换上（`--hook .claude/hooks/runner-dispatch-guard.sh`，喂法一条：派一个没写「重型测试：不跑」的 clerk 照拒，neutral）：
```
✓ 草稿里验过：2 份文件语法检查、内嵌 python 1 段编译、喂了 1 条
✓ 换上了 1 份（改名换上新 inode、回读一致），主工作区里再验过：2 份文件语法检查、内嵌 python 1 段编译、喂了 1 条
```
换上之前 sha256 441124c4236de9b4fd048c7ab3ec8b08dd554481397e08b20e02bf036fa09626，之后 2118b3bccb9ba46d5053a47d456c3fec94cf5114444c701e1c6acf06a61a5286；反向规格在草稿目录 `hookswap/hook-swap/` 下（换上那一次写的那一份 reverse-*.json，另一份是 --dry-run 写的）。用到 lib_long_run 的钩子自证：`bash .claude/hooks/runner-dispatch-guard.sh --selftest` 退 0，「✓ 自检通过（查了 437 种）」（输出在草稿 `runner-dispatch-guard-selftest.txt`）。改动只在 `exempt_main` 与命令行解析，import 时不多载东西（gate.d 那份库只在 `--base` 时才载入）。

runner 表：`python3 .claude/gate.d/lib-lag-baseline.py --selftest` 加进 code-tooling.sh 的 `SELFTEST_RUNNERS`（末行）。`selftest-registry.tsv` 没加行：登记表把键在 `.claude/gate.d/` 这一层的行一律当门禁阶段的样本档（`lib_heavy_tests.py` 的 `selftest_registry_at`：`stage_row` 只认 samples-light / samples-heavy），写 light 会被当成写坏的行；这份库的自证在 research-script-selftests 格里按「表外的、守卫原样 exec、本次未盯」那一档跑。要它被守卫盯着，得把库挪进 `.claude/gate.d/lib/`（判决点名的是现在这个路径，没挪）。

## K2-2　删新格的滞后表

- 删：`.claude/gate.d/replay-comparison-units-lag.tsv`；新格里滞后表的形状、指到 none 行、none 行都登记、同一对不重复四样判法；开关 `comparison-none-lag-unchecked`（文件头那一行一并删了）。
- 只缩不涨改成：登记表里复跑写 none: 的行，按（登记出处，留存产物名）与 gate 基准那一版登记表里复跑写 none: 的行比，基准里没有的判红（「none 行比基准多：…」，逐行点行号）；基准里没有这张表、不是 git 仓，不比、成功句写明；git 取不到判红（`baseline_rows`）。
- `comparison-lag-growth-unchecked` 改成不比登记表的基准（照样取基准，只是不判涨，绿样本的成功句不变）。
- 跟着改的文字：doc-experiments.sh 文件头样本说明两行、开关表、这一格的出路（`stage_cell` 第二个字符串）、格上方的注释；登记表文件头第 6 列的说明。
- 三份样本：
  - red：删滞后表；setup.sh 改成基准提交里的登记表去掉辛、癸两行 none 行（`grep -v` e991-h.out、e991-j.out），提交后换回；expect 里四条滞后表的 want 换成「none 行比基准多：…」与第 9 行 e991-h.out、第 12 行 e991-j.out 三条。
  - green：删滞后表；setup.sh 只改了注释；expect 成功句换成「复跑写 none 的 1 行只缩不涨，与基准 HEAD 比没涨」。
  - zero：没动。
- 草稿里核（格那一段 python，拷贝跑过 setup.sh）：red 退 1、want 全在；green 退 0；zero 退 77。七个开关逐个：

```
comparison-registration-unchecked   red 缺「登记出处不在」1 条；green 照绿
comparison-unit-unchecked           red 缺「一次都没找到」「出现 2 次」「单元表出处不在」3 条；green 照绿
comparison-unit-count-loose         red 缺「出现 2 次」1 条；green 照绿
comparison-unit-substring           red 不变；green 退 1（期望 0）
comparison-driver-unchecked         red 缺「没有 E991 的驱动行 @driver_absent」1 条；green 照绿
comparison-product-unchecked        red 缺「产物名是 e991-other.out」1 条；green 照绿
comparison-lag-growth-unchecked     red 缺「none 行比基准多」与两行明细共 3 条；green 照绿
```
与文件头开关表写的一致。doc-experiments 整道在 `stage-break-switch-replay-exempt.tsv` 里，复跑格不复跑它。

## K2-3　复跑表一份读法

`replay_rows(path)` 放进同一份库（理由：调它的四处都在 doc-experiments.sh，这一道已经要载入这份库取基准；已有的 `lib-item-ref-status.py` 管按编号取 kb 文件，与复跑表不相干）。读法：整份文件逐行，整行形如 `E<数字>|…|…|…|exact 或 timing` 的算复跑行（三处旧写法的形状，与 experiment-replay 格相同）；新格原来只读 TABLE 那一段、实验号认任意大写字母开头、第 5 列不限，改成同一个形状之后今天的 replay.sh 上取到的行一样（下面的比对）。`scripts/gate/admission.py` 那一份没动。

四处：results-cited 的 sed 换成 `python3 "$LIB_LAG_BASELINE" replay-rows "$REPLAY_TABLE" | cut -f4 | sed 's#.*/##'`；verdict-false-named、quoted-result-lines 两份 `ROW_RE` 读表循环删了，改调 `replay_rows`；新格的 `replay_table()` 删了。

比对命令与输出（改前那四种取法原样搬进 `before/old_extractions.py`，在改之前对 replay.sh 各取一遍；改后 `after/new_extractions.py` 载入库、按各处要的列投影，results-cited 那一处走的是阶段里同一条命令行管道；replay.sh 两次都是 sha256 7d4b8b394c14660b412d0634d8359c342805f2087fe3c619a12020c1b2c8da60）：
```
$ for f in results-cited verdict-false-named quoted-result-lines replay-comparison-units; do printf '%s: ' $f; if cmp -s before/$f.txt after/$f.txt; then echo "逐行相同（$(wc -l < after/$f.txt) 行）"; else echo 不同; fi; done
results-cited: 逐行相同（254 行）
verdict-false-named: 逐行相同（254 行）
quoted-result-lines: 逐行相同（254 行）
replay-comparison-units: 逐行相同（254 行）
```
推翻条件：主 agent 真仓单跑这三格，「查了多少」的数与上一次提交时的读数不同。

## K2-5　候选扫描读登记位

- `scripts/kb/stale-candidates.py`：`CARRIER_PATTERNS` 删了 `^\.claude/kb/.+\.md$` 与 `^crates/singlefs-checker/invariants/[^/]+\.md$` 两行；新加 `kb_directories(repository_root)`（载入 `scripts/kb/quote-kb.py` 调它的 `kb_candidate_roots`，读法只在那一份，同 `scripts/agents/three-way-materials.py` 的载法）与 `carrier_patterns(repository_root)`（登记的每个目录下的 `.md`，子目录也算）；`is_current_state_carrier` 多一个参数，两处调用方各取一次模式。
- 开关 `invariants-carrier-missing` 改成 `kb_directories()` 不读登记位、退回写死的 `.claude/kb`，文件头那一行跟着改；点名的项（失败行原文）没变。
- 自证的临时仓里加写一份 `.claude/kb-directories`（两行，与真仓同形）。
- 读数：
```
$ python3 scripts/kb/stale-candidates.py --selftest | tail -n 1     （退 0）
  ✓ stale-candidates 自证通过：…候选收 records/、crates/singlefs-checker/invariants/ 下的不变量清单与没进 git 的文件…
$ STALE_CANDIDATES_BREAK=invariants-carrier-missing python3 scripts/kb/stale-candidates.py --selftest     （退 5）
  ✗ 自证失败：不变量清单（crates/singlefs-checker/invariants/ 下）里的现状句没进候选表
  ✗ stale-candidates 自证没过
$ python3 scripts/kb/stale-candidates.py --benchmark     （退 0）
  ✓ 阳性对照：已知的 25 处腐烂里 25 处在候选里（下限 25）（盲写事实表 60 行，候选 4190 行、2821 个不同的行）
```
它在 `break-switch-replay-files.tsv` 里，复跑格要的失败行原文在上面的输出里。

## K3-6　登记表归执行员、补全单元

- `.claude/gate.d/stage-owners.tsv` 那一行（doc-experiments.sh / replay-comparison-units）：归属从 `experiment-designer,experiment-runner,implementation-writer` 改成 `experiment-runner,implementation-writer`，理由列写登记行归执行员（实现单元、登记复跑的同一次改动里加），单元表归实现员、复跑行归执行员照旧。
- 登记表文件头加一行「登记行归执行员…」。
- 单元表 `formal_run_units`（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18363`）19 个单元、登记表 13 个不同单元名，缺的 6 个对着 `research/prompts/e165-r5-prereg.md` 第十一节末尾比对对象表（19 行）逐个判：都不是比对对象，登记表文件头逐个写了不登记的理由（一个一行）：「r3 合法」（比对对象表 r3 档只有第 14 行对抗、第 15 行择根；这个单元对的是 `driver_e165_r6_r3_legal`，产物带 r6）；「r5 合法」「r5 对抗」「r5 多盘」「奇数版本的 r5 合法」「奇数版本的 r5 对抗」（r5 档是这份登记第三段新跑的，5.6 第三段那一行列的 segment3-r5-*.out，是被比的一侧）。没加登记行。真仓上这一格的读数见上面「真仓上的读数」。

## 要主 agent 跑的命令

工作目录都是主工作区仓根 `/home/fy5090/code/singlefs`，不起 cargo，内存与线程没有特别要求（stage-break-switch-replay 格自己按每台默认重活线程数并行）。样本档的放行要样本与 HEAD 加暂存区逐字节相同，先暂存这一批（本批文件见「改过的文件」，连同新文件 `.claude/gate.d/lib-lag-baseline.py` 与三份删除）再跑。

1. 四道的样本档（判：末行退 0、没有一份样本判错）：
```
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh、doc-process-records.sh、code-source-discipline.sh、code-tooling.sh 四道的样本档; 范围=4 道阶段; 为什么=这一批改了这四道取基准那一版与读复跑表的写法，新格的三份样本也改了，样本全喂一遍核没改坏' bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item doc-experiments.sh --item doc-process-records.sh --item code-source-discipline.sh --item code-tooling.sh
```
2. 库的开关红在点名那一份（判：code-source-discipline 判错的样本正好是 reading-discipline-red 一份，「输出里找不到「读不出基准 HEAD 里的 .claude/gate.d/experiment-seed-fold-lag.tsv（git show 退 128）…」」）：
```
LAG_BASELINE_BREAK=unreadable-as-absent SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的样本档，设 lib-lag-baseline.py 的弄坏开关; 范围=1 道阶段; 为什么=核库的开关打开后红在头注点名的 reading-discipline-red' bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item code-source-discipline.sh
```
   复跑方那一格（它逐条复跑 code-source-discipline 的头注开关，含新加的这一条；break-switch-replay 复跑 stale-candidates.py 的开关）：
```
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 stage-break-switch-replay、break-switch-replay 两格; 范围=2 格; 为什么=新加的 LAG_BASELINE_BREAK 头注与 stale-candidates.py 改过的开关要按头注点名复跑' bash .claude/gate.d/code-tooling.sh --check stage-break-switch-replay,break-switch-replay
```
3. 真仓单跑改过的格（判：都绿；doc-experiments 三格的「查了多少」与上一次提交时的读数相同；replay-comparison-units 报 14 行、none 4 行）：
```
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 results-cited、verdict-false-named、quoted-result-lines、multipath-registry、replay-comparison-units 五格; 范围=5 格; 为什么=这五格改调 lib-lag-baseline.py' bash .claude/gate.d/doc-experiments.sh --check results-cited,verdict-false-named,quoted-result-lines,multipath-registry,replay-comparison-units
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-process-records.sh 的 crates-adversarial-review、agent-def-adversarial-review 两格; 范围=2 格; 为什么=两格开头的挂账表只缩不涨改调 lib-lag-baseline.py' bash .claude/gate.d/doc-process-records.sh --check crates-adversarial-review,agent-def-adversarial-review
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 mutation-tables、reading-discipline 两格; 范围=2 格; 为什么=两格的滞后表与豁免表只缩不涨改调 lib-lag-baseline.py' bash .claude/gate.d/code-source-discipline.sh --check mutation-tables,reading-discipline
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 heavy-thread-defaults、long-run-header-exempt、research-script-selftests、research-script-selftest-coverage、change-range-single-source、stage-owners 六格; 范围=6 格; 为什么=两格改调 lib-lag-baseline.py，runner 表加了库的自证，归属表改了一行，库里的 git 调用要过 quotepath 那一格' bash .claude/gate.d/code-tooling.sh --check heavy-thread-defaults,long-run-header-exempt,research-script-selftests,research-script-selftest-coverage,change-range-single-source,stage-owners
```
4. 提交前那一趟的 lint 照常会判到的（gate-lint、shell-lint、preflight-lint 判新库与改过的四道；gate-overlap 判新库的 `# gate-similar:` 与七处删掉的重复）：整轮 `scripts/gate/gate-staged.sh` 里跑，不单列。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 broad-mutation.py 起 cargo 的用法、gate.sh、全量 cargo test、check.sh）；没跑门禁阶段与各类 lint（stage-selftest 被重型测试闸拒：样本与暂存区不同，命令在上一节）；没走定义三方（这一批没改定义、共用约束、规则）；没提交，没暂存。
- 命名检查没跑：这一批没改 `crates/`、`scripts/` 下的 `.rs`。
- 第八处同一写法（doc-experiments.sh decision-links 格的 `.claude/decision-links-pending`，`:1069`）没并，不在派发的七处里。
- kb 里还点名删掉的滞后表：`.claude/kb/milestone/03-third-txn.md`（与 `.en.md`、`.ja.md`）有 `replay-comparison-units-lag.tsv` 字样，kb 不在我写范围里，交 kb-scribe（`grep -rln replay-comparison-units-lag` 现查到的现状载体只有这三份，records 与 research 下的是冻结证据）。
- `.claude/batch-scope` 没登记这一批的文件（不在写范围里）。
- `selftest-registry.tsv` 没加行（原因在 K2-1 末段）；库的自证没挪进 `.claude/gate.d/lib/`。
- 执行员定义里「往登记表登记」那一步与写范围表那一行归第 17、18 号任务，没碰；`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md`、`runner-dispatch-guard.sh`、`continuation-guard.sh`、`agent-write-scope.tsv`、`scripts/agents/` 都没碰。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/landing-audit-r1fix-gates/` 里只有小脚本、改前快照（`before/`，含三份删掉的滞后表原样与改前的两份 setup.sh）、改法片段（`edits/`）、hook-swap 的喂法日志与反向规格；没有仓副本与编译目录。
