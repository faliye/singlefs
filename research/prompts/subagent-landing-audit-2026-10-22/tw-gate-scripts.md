# 第三批任务 10 报告：比对对象机读登记与一格、占号跨工作树、候选扫描认不变量清单

日期：2026-10-22。结论：三条的改动都落了，脚本两份的 `--selftest` 在本机跑到绿、弄坏开关打开后红在点名那一项；
门禁格 replay-comparison-units 的判法本体（格里嵌的 python）用草稿里抽出来的壳逐份喂样本验过，
但「门禁阶段那一层」（stage-selftest 喂样本、真仓单跑新格）被闸拒了，交主 agent 跑，命令在末尾「要主 agent 跑的命令」。

## 改过与新建的文件

| 文件 | 状态 |
|---|---|
| `.claude/gate.d/doc-experiments.sh` | 改（暂存区里原有改动保留，在上面加）：新格 replay-comparison-units、格名表、header、八个弄坏开关 |
| `.claude/gate.d/stage-owners.tsv` | 改一行：新格的归属（code-tooling 的 stage-owners 格要求门禁文件头每格都有一行，没在派发提示的文件清单里，是新格带出来的） |
| `.claude/gate.d/replay-comparison-units.tsv` | 新建：机读登记，六列，文件头写列义 |
| `.claude/gate.d/replay-comparison-units-lag.tsv` | 新建：滞后表（名字我起），三列 |
| `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-{red,green,zero}/` | 新建：三份样本 |
| `scripts/kb/claim-experiment.sh` | 改 |
| `scripts/kb/stale-candidates.py` | 改（暂存区里原有改动保留） |

`git diff --stat`（工作区相对暂存区，只含已跟踪的）原样：

```
 .claude/gate.d/doc-experiments.sh | 225 ++++++++++++++++++++++++++++++++++++--
 .claude/gate.d/stage-owners.tsv   |   1 +
 scripts/kb/claim-experiment.sh    | 101 +++++++++++++----
 scripts/kb/stale-candidates.py    |  10 +-
 4 files changed, 310 insertions(+), 27 deletions(-)
```

新建的未跟踪文件（`git status --short`）：两张 tsv 与三个样本目录，需主 agent 暂存。

sha256（交回时）：
```
1d8e5050d00a37eb3482e284e785b95fa2906e14f66ff1ff83cdcd629150aa6c  .claude/gate.d/doc-experiments.sh
e19108a0b36c155764e203ff1f8c5b6fdfb618352d3b026298295c5d7e798294  scripts/kb/claim-experiment.sh
5dc7e442e3a979c13c80eaed3f047c2e9cecfbb73b00007531be017a71b3f1f1  scripts/kb/stale-candidates.py
1b0142061c1ed7ddb3937ab4852b82287dfc77b256da70b374d36f91076f6353  .claude/gate.d/replay-comparison-units.tsv
7daf74bf350f43bded45a12e6bc6e8c22c916ec3b05e0f94ec1bf2d11d119f66  .claude/gate.d/replay-comparison-units-lag.tsv
```

## 一、重跑比对对象机读登记与 doc-experiments 一格

### 登记表的列（第 11、12 号任务按这个写，列没动）

`replay-comparison-units.tsv` 六列、制表符分隔：实验号、登记出处、单元名、单元表出处、留存产物名、复跑。
列义写在文件头。留存产物名一列我补了读法：复跑写驱动名的，等于 replay.sh 复跑表那个驱动那一行的第 4 列；复跑写 none: 的，写登记里这条正式跑的重跑产物名。

### 起步登记了什么

E165 r5 比对对象表里有单元的 14 行（`grep -vc '^#'` 得 14，每行 6 列）：
- 复跑写驱动名 10 行：`driver_e165`、`driver_e165_light`（同一个单元「r1 合法」，各登记一行）、`driver_e165_adversarial`、`driver_e165_r2_segment3_multi_disk`、`…_segment4_table_both_{legal,adversarial,multi_disk}`、`…_segment4_reversed_disk_order_{legal,multi_disk}`、`driver_e165_r3_segment1_adversarial`。
- 复跑写 none: 4 行：「r2 合法」「r2 对抗」「奇数版本的 r2 合法」「奇数版本的 r2 对抗」，同时进滞后表（4 行，起步）。

### r1 至 r4 为什么不登记

r5 的比对对象表（`research/prompts/e165-r5-prereg.md` 里「**比对对象**」那一段，19 行）把 r1 到 r4 各档要比的正式跑都重列了一遍
（r1 档三行、r2 档若干行、r3 档、r4 档），r1 至 r4 四份登记本身的比对对象表写法各不相同
（`research/prompts/subagent-landing-audit-2026-10-22/doc-experiments-cell-stop.md` 现查过），被 r5 那份取代，不再登记。

### 没登记的 5 行（要主 agent 知道）

r5 比对对象表的第 15 至 19 行（根选择两行、杀伤界三行）在单元表 `formal_run_units`（19 个单元）里没有对应单元：
单元表只放带判据对照的家族正式跑，根选择有自己的 `RootChoiceCriterion`，杀伤界是另一种模式。
我没有给它们编单元名、也没有用 none: 把「没有单元」混进复跑那一列：这一格的第二项判据在这 5 行上现在就是红，登记进去等于要再立一张「单元缺口」滞后表，超出派发提示的列与滞后表形态。
这 5 行是真缺口还是本来就不该有单元，交主 agent 判；要登记就得扩格（单元名列允许 `none:理由` 并另立滞后），先别由我定。

### 现查到的两处不一致（影响留存产物名怎么填）

- 单元表注释（`e165_replay_bounded_enumeration.rs`）写「r1 合法」「r1 对抗」的产物是 `…2026-10-04-r5-segment1-r1-…out`，这几份在提交 04ceaa904 归档时从工作区删掉了，`replay.sh` 的 `driver_e165`、`driver_e165_light`、`driver_e165_adversarial` 三行的期望产物仍是 2026-10-01 的原产物。我按 `replay.sh` 填（那三份在 research/results/ 里在），否则第三项判据就红。
- 比对对象表第 4 至 7 行的「比对对象」列写的是 2026-10-02 的旧产物，其中两份在 research/results/ 里的名字少了 `-2026-10-02` 后缀、反奇偶两份不在树里；留存产物名填的是 r5 重跑产物（单元表注释点名的那四份，树里都在）。

### 新格 replay-comparison-units 判什么

登记 0 行退 77。每行判三样：
- 登记出处文件存在；
- 单元名作为带双引号的字符串字面量在单元表出处里恰好出现一次（0 次、两次以上都红；「r2 合法」不会在「奇数版本的 r2 合法」里被数到）；
- 复跑写驱动名的，`replay.sh` 的 `TABLE=$(cat <<'TSV'` 那一段里有实验号相同、第 2 列是 `@驱动名` 的行，且其中一行第 4 列等于留存产物名。

复跑写 none: 的（理由至少 4 个字）不判第三项，改判滞后表：滞后表三列（登记出处、留存产物名、依据）每行指到登记表里复跑写 none: 的行，
登记表里复跑写 none: 的每行都在滞后表里，键不重复，只缩不涨（与基准提交里的滞后表比，基准里没有的键不许出现；基准里没有这张表时不比；git 取不到基准判红）。
另判行形状（六列、实验号 E<数字>、复跑列是驱动名或 none:理由）。形态照 multipath 格的滞后表判法，多一列是因为同一个留存产物可能被两份登记各列一次，键要带登记出处。

管不到的（写在格的注释里）：登记散文里的比对对象表有没有照实抄进机读表、留存产物名指到的文件在不在、单元表里和 replay.sh 里没登记的单元与驱动行（反向）。

### 样本与弄坏开关（开关值都给 `EXPERIMENT_PAGES_AND_PRODUCTS_BREAK`，写在 doc-experiments.sh 文件头）

三份样本在 `.claude/gate.d/fixtures/doc-experiments.sh/`：
- `replay-comparison-units-red`（setup.sh 提交基准，基准滞后表一行、工作区再加两行）：十一处各一种红，期望退 1。
- `replay-comparison-units-green`（单元名「甲」「乙甲」并存，驱动行两行、none 行一行加滞后表一行，与基准相同）：期望退 0。
- `replay-comparison-units-zero`（只有注释）：期望退 77。

| 开关值 | 弄坏哪一处 | 打开后红在哪 |
|---|---|---|
| comparison-registration-unchecked | 不查登记出处 | red 里「登记出处不在：research/prompts/e991-missing.md」找不到 |
| comparison-unit-unchecked | 不查单元表出处与单元名 | red 里「一次都没找到」「出现 2 次」「单元表出处不在」三条找不到 |
| comparison-unit-count-loose | 出现一次以上都算找得到 | red 里「出现 2 次，不是恰好一次」找不到 |
| comparison-unit-substring | 单元名不带引号按子串数 | green 判错（期望退 0 实测 1，「甲」数到两次） |
| comparison-driver-unchecked | 不查驱动名在不在 replay.sh | red 里「没有 E991 的驱动行 @driver_absent」找不到 |
| comparison-product-unchecked | 不比第 4 列产物名 | red 里「不是留存产物名 e991-g.out」找不到 |
| comparison-none-lag-unchecked | none 行可以不进滞后表 | red 里「滞后表里没有这一行（… e991-h.out）」找不到 |
| comparison-lag-growth-unchecked | 不与基准比只缩不涨 | red 里「滞后表涨了」与「e991-j.out」两条找不到 |

### 实际读数（门禁阶段那一层没跑，见下；这里是把格里嵌的 python 抽出来在样本拷贝上喂的读数）

壳是草稿目录里的 `cell.py`（从 doc-experiments.sh 里抽出来的）加 `cellrun.sh`（拷样本、跑 setup.sh、喂 python、对 expect 的 exit 与 want，壳层的汇总行与出路行不判）。
没有起 doc-experiments.sh，也没起 stage-selftest.sh（闸拒了，原文见「没做什么」）。

无开关：
```
replay-comparison-units-red    exit=1  RESULT right（expect 里非壳层的 want 全中）
replay-comparison-units-green  exit=0  RESULT right
replay-comparison-units-zero   exit=77 RESULT right
```
八个开关逐个打开（完整输出 `breaks.log`，共 44 行）：red 或 green 上判错的位置就是上表「打开后红在哪」，别的样本不受影响（`breaks.log` 里 RESULT 行：
registration-unchecked、unit-unchecked、unit-count-loose、driver-unchecked、product-unchecked、none-lag-unchecked、lag-growth-unchecked 七个只让 red 判错；unit-substring 只让 green 判错）。

真仓读数（python 本体在真仓根上喂，基准为空即「基准里还没有这张表」，不是门禁单跑）：
```
  ✓ 比对对象 14 行（E165）：登记出处都在，单元名在单元表出处里都恰好出现一次；复跑写驱动名的 10 行在 replay.sh 里都有且产物名相符；复跑写 none 的 4 行都登记在 replay-comparison-units-lag.tsv 里，基准里还没有这张表（或不是 git 仓），只缩不涨这一条没比
exit=0
```
同一份登记在 `comparison-unit-substring` 下数到「r2 合法」「r2 对抗」「r2 多盘」3 个单元名各出现 2 次以上（子串会被「奇数版本的…」冒充），说明带引号的数法是真仓需要的。

`bash .claude/gate.d/doc-experiments.sh --list` 能跑（闸放行），末行是新格 `replay-comparison-units` 及判什么，格名登记通过；`bash -n` 过。

## 二、占号跨工作树（K9）：`scripts/kb/claim-experiment.sh`

改了什么：查号对 `git worktree list --porcelain` 列出的每一棵工作树都查（目录不在的跳过），`--next` 取全部工作树最大号加一，占号时别的工作树用过的号一样被拒，占号文件仍建在本棵。
不在 git 仓里（临时目录）只查给定的那一棵；在 git 仓里而 `git worktree list` 失败则拒绝给号（`--next` 退 1，占号退 3，stderr 写出路），不退回只查本棵。
自证加一格两棵工作树：临时 git 仓（本棵用过 E5）加 `git worktree add` 出来的另一棵（只在那里用过 E12）：本棵 `--next` 该是 E13、另一棵 `--next` 该是 E13、本棵占 E12 被拒、本棵占 E13 成、另一棵再占 E13 被拒、另一棵占 E14 成。
弄坏开关 `CLAIM_SKIP_WORKTREES=1`（只查给定的那一棵），写在文件头。

读数（原样）：
```
$ nice -n 19 bash scripts/kb/claim-experiment.sh --selftest
selftest: 通过（--next 取到 E9；已有源码的 E6 被拒；E9 第一次占成、第二次被拒；两棵工作树：另一棵的 E12 在本棵被拒、--next 取两棵最大号加一、E13 只有一棵占得成）
exit=0
$ CLAIM_SKIP_WORKTREES=1 bash scripts/kb/claim-experiment.sh --selftest
selftest: 另一棵工作树用过 E12，本棵 --next 该是 E13，拿到 E6
exit(skip-worktrees)=1
$ CLAIM_SKIP_USED_CHECK=1 bash scripts/kb/claim-experiment.sh --selftest      （旧开关，行为不变）
selftest: 关掉查号确认判红（已有源码的 E6 被占成功）
exit(skip-used)=0
```
真仓读数（`--next` 只读；`m5-dev` 工作树只用 `CLAIM_ROOT` 指过去读，没写）：
```
real --next: E197
main only (CLAIM_SKIP_WORKTREES=1): E188
m5-dev only (CLAIM_SKIP_WORKTREES=1 CLAIM_ROOT=m5-dev): E197
m5-dev all (CLAIM_ROOT=m5-dev): E197
```
即改前主工作区会给 E188，而 `m5-dev` 工作树里已用到 E196，正是 E185 到 E187 撞号的那种局面。
读不出工作树清单那一支，用一个对 `worktree list` 假失败的 git 垫片（草稿目录 `fakebin/git`）手测：`--next` 退 1 并打出路，真仓占号退 3 且没建文件，非 git 临时目录照旧占成。这一支没进 `--selftest`。
自证跑完 `/tmp` 里的条目数前后相同（2365 对 2365），没留临时目录。
登记表 `.claude/hooks/selftest-registry.tsv` 里这份脚本是 light，理由写「临时目录里摆几处已用的号」；现在自证另起临时 git 仓与它的 worktree，仍只起 git，仍是 light；理由文字过时了，该行在 `.claude/hooks/` 下、派发提示点名不碰，交主 agent 或第 12 号任务顺手改。

## 三、候选扫描认不变量清单：`scripts/kb/stale-candidates.py`

改了什么：`CARRIER_PATTERNS` 加 `^crates/singlefs-checker/invariants/[^/]+\.md$`（不变量清单 2026-10-11 拆出来之后的载体）。
自证加一格：基准提交里在 `crates/singlefs-checker/invariants/01-sample.md` 放一句「层 0 只有一次挂载」，候选表里必须有 `('F1', 'crates/singlefs-checker/invariants/01-sample.md:1')`。
弄坏开关 `STALE_CANDIDATES_BREAK=invariants-carrier-missing`（`is_current_state_carrier` 把该目录当不是载体），写在文件头开关一览里。
基准阳性对照（`--benchmark` 的 `b1c8cef~1` 到 `00c9d4f`）不受影响：`git ls-tree 00c9d4f crates/singlefs-checker/invariants` 为空，那时这个目录还不存在；没跑 `--benchmark` 本身。

读数：
```
$ nice -n 19 python3 -B scripts/kb/stale-candidates.py --selftest
  ✓ stale-candidates 自证通过：…候选收 records/、crates/singlefs-checker/invariants/ 下的不变量清单与没进 git 的文件、…
exit=0
$ STALE_CANDIDATES_BREAK=invariants-carrier-missing python3 -B scripts/kb/stale-candidates.py --selftest
  ✗ 自证失败：不变量清单（crates/singlefs-checker/invariants/ 下）里的现状句没进候选表
  ✗ stale-candidates 自证没过
exit(break)=5
```
`python3 -m py_compile` 过；`bash -n` 过（claim-experiment.sh、doc-experiments.sh）。

## 要主 agent 跑的命令

被闸拒的原文（heavy-test-guard）：`stage-selftest.sh` 拒的原因是「doc-experiments.sh 的样本与 HEAD 加暂存区不同（16 处：replay-comparison-units-green/.gate-cells（暂存区里没有）…）」；
直接起 `bash .claude/gate.d/doc-experiments.sh --check replay-comparison-units`（带 `SINGLEFS_RUN_SCOPE`）拒的原因是「tooling-writer 不跑提交时才跑的检查」。我没换写法绕过去（没有起 doc-experiments.sh，也没把样本拷到别处用 stage-selftest 的逻辑喂它）。

1. 先暂存新文件：三个样本目录 `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-{red,green,zero}/`、`.claude/gate.d/replay-comparison-units.tsv`、`.claude/gate.d/replay-comparison-units-lag.tsv`（stage-selftest 要样本与暂存区相同）。
2. 仓根：`bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item doc-experiments`。基准是改前 52 个用例全绿（草稿里 `baseline-selftest.log`），这次应是 55 个用例，三个新用例 `replay-comparison-units-{green,red,zero}` 判得对。
3. 八个开关逐个：`EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=<开关值> bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item doc-experiments`，红在上表点名的那一项，其余用例不变。开关值：comparison-registration-unchecked、comparison-unit-unchecked、comparison-unit-count-loose、comparison-unit-substring、comparison-driver-unchecked、comparison-product-unchecked、comparison-none-lag-unchecked、comparison-lag-growth-unchecked。
4. 新格在真仓上单跑：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 replay-comparison-units 格; 范围=1 格; 为什么=新加的格在真仓上的读数' bash .claude/gate.d/doc-experiments.sh --check replay-comparison-units`。期望成功句：「比对对象 14 行（E165）…复跑写驱动名的 10 行…复跑写 none 的 4 行都登记在 replay-comparison-units-lag.tsv 里，与基准 … 比没涨」或「基准里还没有这张表…没比」（取决于基准里有没有滞后表，滞后表没提交时是后者）。
5. 门禁文件头结构、gate-lint、shell-lint、gate-overlap 这类归提交前那一趟，我没跑；`gate-overlap.py --list` 开工前跑了（`overlap-list.txt`），没有别的门禁管「登记 ↔ 单元表 ↔ replay.sh」这件事。

## 没做什么

- 没跑 stage-selftest、没起 doc-experiments.sh、没跑 gate-lint / shell-lint / gate-structure-check / gate-overlap 判法：都是提交前主 agent 的活，或被闸拒。门禁格的 python 本体用抽出来的壳验过，shell 外壳（取基准、`in_git_work_tree` 判断、`exit 77`、`stage_cell` 出路的打印）没有端到端跑过，端到端交 1 至 4。
- 没登记 r5 比对对象表第 15 至 19 行（见一）；没登记 r1 至 r4。
- 没改 `.claude/hooks/selftest-registry.tsv`（该文件不在这一单的改动范围）；claim-experiment.sh 那一行的理由文字过时，见二。
- 没加「留存产物名指到的文件在不在」「单元表与复跑表里没登记的行」的反向判（超出派发提示的三样）。
- `stage-owners.tsv` 里新格的归属（experiment-designer、experiment-runner、implementation-writer）是我按「登记行由写登记的人加、单元表在 crates/、复跑表在 replay.sh」定的，第 11、12 号任务改写范围时可能要跟着调；这一行的措辞里写了「没有合并之前的旧归属」。
- 没读、没改 `m5-dev` 工作树（只用 `CLAIM_ROOT` 指向它让脚本读它的目录）。
- 没提交，没跑重型测试。

推翻本结论的现象：端到端 stage-selftest 里三个新用例任何一个判错（先看是壳层行如「样本只跑 .gate-cells 点名的格」的 want 对不上，还是格自己的 want）；
真仓单跑新格红（先看 `replay.sh` 复跑表是否在我登记之后改了驱动行的期望产物）。
