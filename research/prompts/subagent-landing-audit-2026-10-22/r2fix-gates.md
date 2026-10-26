# landing-audit-r2 修补「门禁与扫描」那一份：交回报告（tooling-writer，2026-10-23）

批次：records/2026-10-22-子agent结果落地审计.md#7。判决：research/prompts/landing-audit-r2-main-verification.md「二、逐格判」K2、K4 两张表，「三、判决」第一张表「门禁与扫描」那一行；攻方 research/prompts/landing-audit-r2-opus-output.md（K2-1、K2-2、K4-1、K4-2）。

## 结论（四条各自的出口）

| 条 | 验收 | 状态 |
|---|---|---|
| K2-1（甲） | 库文件头写明四处认法变了、方向是跳过不报形状错；判法不改 | 做完。`content_rows` 一字没动；文件头加了一段（四处点名、两种行形、方向、doc-process-records / doc-decisions / decision-links 三处没变的理由）。攻方探针复跑读数与文件头写的一致 |
| K2-2（甲） | 键改成（实验号，登记出处，单元名，留存产物名）；新借键红样本改后红、暂存区那一版绿；红绿零三份照旧判对；文件头样本表与开关表同步 | 做完。新样本 replay-comparison-units-rekey-red（三种借法各一行、一行原样）：改前工作区那一版退 0，暂存区那一版（加一张基准与工作区相同的滞后表）退 0，改后退 1、三行逐行点名、原样那行不报；新开关 comparison-none-key-pair 打开后只这一份判错 |
| K4-1 与 ⑪ | 两处改调库，取不到判红、句式是 `BaselineProblem.sentence`；各一份样本；`LAG_BASELINE_BREAK=unreadable-as-absent` 下红在点名那一份；grep 里没有自己取基准表的两段 | 做完。doc-decisions 的 `base_ledger_text` 调 `baseline_text`，三格各把问题句并进挂账表的红行；decision-links 调 `baseline_rows`。新样本 evidence-ledger-base-unreadable-red（三格）改前三格都绿、改后三格都红；decision-links-pending-base-unreadable-red 改前改后都红（这一格原来就分三种结局），开关打开后改前不受影响、改后判错。grep 剩下的 9 行都不是取基准表的段（下面逐行列） |
| K4-2 | 规整只在 quote-kb.py 一处；绝对路径与 `..` 出仓的拒、stderr 报那一行与出路；doc-registries 那一份改调它；stale-candidates 加一格；quote-kb 加规整与拒绝两格 | 做完。doc-registries 的 `kb_candidate_roots` 删了自己的 bash 读法，改起一段 python 调 quote-kb.py（没留副本，不写 copy-kept）。stale-candidates 新格改前红、改后绿；quote-kb 两格绿，新开关 kb-directories-raw 下红 |

另做了一件派发没列、改调库时不做就会退步的事：库的 `baseline_text` 里 `git show` 改写成 `<基准>:./<路径>`（ls-tree 本来就按当前目录认路径，show 原来按仓根认，两边不一致；doc-decisions 原来的取法写的就是 `./`，文件头写明「样本根在别的仓里时不会读到那个仓的表」）。加了自证格「子目录里按当前目录取」与开关 `LAG_BASELINE_BREAK=show-root-relative`。在仓根上跑（门禁、样本、--staged 的 worktree）两种写法取到的是同一份。

推翻条件：
- 主 agent 暂存后跑 stage-selftest（「要主 agent 跑的命令」第 1 条）有一份样本判错；或 `LAG_BASELINE_BREAK=unreadable-as-absent` 下 doc-decisions 判错的不正好是 {evidence-ledger-base-unreadable-red}、doc-experiments 判错的不正好是 {decision-links-pending-base-unreadable-red}、code-source-discipline 判错的不正好是 {reading-discipline-red}，K4-1 不算达成。
- `EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-none-key-pair` 下 doc-experiments 判错的不正好是 {replay-comparison-units-rekey-red}，K2-2 不算达成。
- 真仓单跑 doc-registries 的六格（第 5 条）读数与上一次提交时不同，或 stderr 出了「.claude/kb-directories 第 N 行」的拒绝句，K4-2 改坏了真仓的读法。
- stage-break-switch-replay 复跑 doc-decisions 时，新样本出现在头注没点名它的开关下判错（我按读代码推的那几条开关逐个列在下面「没核动的」），头注就点得不全。

## 改过的文件

这一批的改动行数（相对开工时存进草稿 `before/` 的那一份，`diff -u` 数的非空增删行；doc-experiments.sh、stale-candidates.py 上一批 r1fix 的改动还没暂存，`git diff --stat` 里混着它们，所以先给这一份）：

```
.claude/gate.d/lib-lag-baseline.py: +32 -7
.claude/gate.d/doc-experiments.sh: +49 -29
.claude/gate.d/doc-decisions.sh: +51 -30
scripts/kb/quote-kb.py: +82 -10
.claude/gate.d/doc-registries.sh: +14 -12
scripts/kb/stale-candidates.py: +18 -3
```

`git diff --stat -- <这些文件>`（相对暂存区）原样：

```
 .claude/gate.d/doc-decisions.sh                    |  85 +++---
 .claude/gate.d/doc-experiments.sh                  | 292 +++++++++------------
 .claude/gate.d/doc-registries.sh                   |  26 +-
 .../replay-comparison-units-red/expect             |   7 +-
 scripts/kb/quote-kb.py                             |  98 ++++++-
 scripts/kb/stale-candidates.py                     |  56 +++-
 6 files changed, 328 insertions(+), 236 deletions(-)
```

没进 git 的（`git status --short` 原样）：

```
?? .claude/gate.d/fixtures/doc-decisions.sh/evidence-ledger-base-unreadable-red/
?? .claude/gate.d/fixtures/doc-experiments.sh/decision-links-pending-base-unreadable-red/
?? .claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-rekey-red/
?? .claude/gate.d/lib-lag-baseline.py
```

`lib-lag-baseline.py` 是上一批新建的、还没暂存（294 行）；这一批在它上面改。三份新样本：
- `fixtures/doc-experiments.sh/replay-comparison-units-rekey-red/`：`.gate-cells`、`expect`、`setup.sh`、`.claude/gate.d/replay-comparison-units.tsv`、`crates/sample/units.rs`、`research/prompts/e991-registration.md`
- `fixtures/doc-experiments.sh/decision-links-pending-base-unreadable-red/`：`.gate-cells`、`expect`、`setup.sh`（decision-links-green 的 setup.sh 逐字照抄，换了开头三行注释，末尾加三行删基准那一版待回填清单的对象）
- `fixtures/doc-decisions.sh/evidence-ledger-base-unreadable-red/`：`.gate-cells`（三格）、`expect`、`setup.sh`、`.claude/gate.d/decision-evidence-lag.tsv`、`.claude/kb/decisions/2522-账样本乙.md`（照抄 evidence-ledger-grew-red 那一份）
- 改了一份已有样本：`fixtures/doc-experiments.sh/replay-comparison-units-red/expect` 两行 want 换成新的明细写法（`第 9 行 E991「辛」e991-h.out（…）`、`第 12 行 E991「癸」e991-j.out（…）`）。

改法：已存在的文件用 Edit 工具（换 inode，`stat -c %i` 核过：lib-lag-baseline.py 25071182 → 25035639）或 `scripts/kb/replace-once.py`（doc-decisions.sh 头注里两段逐字相同的样本清单、stale-candidates.py 两处）；新文件先 `ls` 确认不在再建。开工时六份文件各存了一份快照进草稿 `before/`；收工时逐份 `diff -u before/<名> <文件>`（草稿 `after-*.diff`），每个 hunk 都是这一批的（doc-decisions 的 17 个 hunk 逐个看过），写的过程中没有别人的改动混进来。

没有新门禁、新钩子，所以没有新写的 `# gate-similar:` / `# hook-events:` 行；`gate-overlap.py --list` 开工时跑过（57 份：门禁 40、钩子 17），没有管同一件事的另一份。doc-registries 那一份读法删了改调 quote-kb.py，不写 `gate-overlap:copy-kept`。

## 草稿里怎么核的样本（阶段本身没跑）

样本档 stage-selftest 要样本与暂存区相同，这一批不暂存，所以照上一批的做法：草稿 `tools/` 下的小脚本从门禁文件里取出那一格的 python heredoc，在样本拷贝上（跑过 setup.sh）照阶段的参数起，比 expect 的退出码与 want；阶段级的 want（「汇总」「样本只跑」「<阶段>.sh：N 格里…」「<格>：红（退出 1）」「→ <格>：」「--check」「本次未跑」）不在格的 python 输出里，跳过并报跳过几条。
- `tools/rcu.sh <门禁文件> <样本> [开关]`：replay-comparison-units（认得改后的与暂存区那一版两种参数形状）
- `tools/dl.sh <门禁文件> <样本> [开关] [LAG_BASELINE_BREAK]`：decision-links，基准与改动范围照阶段用 `scripts/gate/changed-paths.sh` 的 `gate_diff_base gate` / `gate_changed_paths` 取
- `tools/dd.sh <门禁文件> <样本> [格名,…] [DOC_DECISIONS_BREAK] [LAG_BASELINE_BREAK]`：doc-decisions 的三格（DECISION_EVIDENCE_SOURCE 与格的 python 各取一段，`load_diff_base` 同一取法）；多格时「退出码」是「有一格退 1 就记 1」
- `tools/check_wants.sh`、`tools/extract_heredoc.py`、`tools/syntax.py`（与 py_compile 同一个 `compile()`，不写 `__pycache__`）

## K2-1　库文件头：四处的注释行认法

改的只有 `.claude/gate.d/lib-lag-baseline.py` 文件头：「调它的九处」那一段之后加了「注释行的认法并进来时变了（content_rows）」一段（四处点名、缩进的 `#` 行与行首制表符第二列 `#` 的行改前判形状错、并后跳过、方向判得更严、doc-process-records 改前就 lstrip 没变；这一批新并进来的 doc-decisions 挂账表不经 `content_rows`、decision-links 经它之后再过形状正则，认出的行不变）。`content_rows` 的代码一字没动。

该红的输入是攻方的四行表（第 2 行缩进的注释、第 3 行行首制表符），判法有意不改，所以没有「改后红」，只核文件头说的与实际一致。攻方探针复跑（`python3 -I research/prompts/landing-audit-r2-opus-model/probes/lstrip_probe.py /home/fy5090/code/singlefs`）原样：

```
name=content_rows rows=[1, 4]
name=multipath-registry before_rows=[1, 2, 3, 4] after_rows=[1, 4] same=False
name=replay-comparison-units before_rows=[1, 2, 3, 4] after_rows=[1, 4] same=False
name=mutation-tables before_rows=[1, 2, 3, 4] after_rows=[1, 4] same=False
name=reading-discipline before_rows=[1, 2, 3, 4] after_rows=[1, 4] same=False
name=doc-process-records before_rows=[1, 4] after_rows=[1, 4] same=True
```

今天三张表里这两种行形逐张 0 行（`grep -cE '^[[:space:]]+#|^'$'\t'`：multipath-registry-lag.tsv 0、replay-comparison-units.tsv 0、mutation-duplicates-lag.tsv 0）。

## K2-2　replay-comparison-units：none 行的键

改法（`.claude/gate.d/doc-experiments.sh` 这一格的 python）：
- 新函数 `none_identity(实验号, 登记出处, 单元名, 留存产物名)` 交（实验号，登记出处，单元名，留存产物名）；`none_key`（读基准那一版）与 `none_rows`（此刻的）都经它；开关 `comparison-none-key-pair` 让它退回（登记出处，留存产物名）。
- 重复行判法（同一登记出处、同一留存产物名不许两行，`key = (registration, product)`）不随键变，格的注释写明了。
- 红行明细从 `第 N 行 <产物>（<登记出处>）` 改成 `第 N 行 <实验号>「<单元名>」<产物>（<登记出处>）`，出路加一句「换了实验号或单元名就是新的比对对象……不许改一行已有 none 行的实验号或单元名借它的登记」。
- 文件头：样本说明加 rekey-red 一句；开关表 `comparison-lag-growth-unchecked` 下补点名的两份样本，新加 `comparison-none-key-pair` 一条（mutate.sh 写法）；格上方「只缩不涨」的注释改成四样进键与为什么。

新样本 replay-comparison-units-rekey-red：基准登记表四行 E991 的 none 行（丙 e991-c.out、戊 e991-e.out、庚 e991-g.out、甲 e991-a.out），工作区第 3 行换成 E992「丁」（实验号与单元名都换）、第 4 行 E991「己」（只换单元名）、第 5 行 E993「庚」（只换实验号）、第 6 行原样；登记出处与产物名四行都不变。

改前（工作区那一版，键是（登记出处，留存产物名））：
```
  ✓ 比对对象 4 行（E991、E992、E993）：登记出处都在，单元名在单元表出处里都恰好出现一次；复跑写驱动名的 0 行在 replay.sh 里都有且产物名相符；复跑写 none 的 4 行只缩不涨，与…
rc=0
```
对照：暂存区那一版门禁（`git show :.claude/gate.d/doc-experiments.sh`，滞后表 replay-comparison-units-lag.tsv 键同为这一对）。样本原样喂它会红在「滞后表里没有这一行」（它要一张滞后表），所以对照拷贝里加了一张基准与工作区相同的四行滞后表（草稿 `control-staged-rekey/`）：
```
  ✓ 比对对象 4 行（E991、E992、E993）：登记出处都在，单元名在单元表出处里都恰好出现一次；复跑写驱动名的 0 行在 replay.sh 里都有且产物名相符；复跑写 none 的 4 行都登记在 replay
rc=0
```
改后：
```
  ✗ none 行比基准多：replay-comparison-units.tsv 里这几行复跑写 none:，基准 HEAD 那一版里没有这样的行：
     第 3 行 E992「丁」e991-c.out（research/prompts/e991-registration.md）
     第 4 行 E991「己」e991-e.out（research/prompts/e991-registration.md）
     第 5 行 E993「庚」e991-g.out（research/prompts/e991-registration.md）
== 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 4 条）
rc=1
```
四份样本（改后，`samples-doc-experiments-after.txt`）：
```
replay-comparison-units-red: == 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 5 条）
replay-comparison-units-green: == 退出码 0（expect 写 0），找不到的 want 0 条（阶段级 want 跳过 3 条）
replay-comparison-units-zero: == 退出码 77（expect 写 77），找不到的 want 0 条（阶段级 want 跳过 2 条）
replay-comparison-units-rekey-red: == 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 4 条）
```

八个 comparison-* 开关逐个在四份样本上（`switches-rcu.txt` 原样）：
```
--- 开关 comparison-none-key-pair
red: == 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 5 条）
green: == 退出码 0（expect 写 0），找不到的 want 0 条（阶段级 want 跳过 3 条）
zero: == 退出码 77（expect 写 77），找不到的 want 0 条（阶段级 want 跳过 2 条）
rekey-red: == 退出码 0（expect 写 1），找不到的 want 4 条（阶段级 want 跳过 4 条）
--- 开关 comparison-lag-growth-unchecked
red: == 退出码 1（expect 写 1），找不到的 want 3 条（阶段级 want 跳过 5 条）
green: == 退出码 0（expect 写 0），找不到的 want 0 条（阶段级 want 跳过 3 条）
zero: == 退出码 77（expect 写 77），找不到的 want 0 条（阶段级 want 跳过 2 条）
rekey-red: == 退出码 0（expect 写 1），找不到的 want 4 条（阶段级 want 跳过 4 条）
--- 开关 comparison-registration-unchecked
red: == 退出码 1（expect 写 1），找不到的 want 1 条（阶段级 want 跳过 5 条）
green: == 退出码 0（expect 写 0），找不到的 want 0 条（阶段级 want 跳过 3 条）
zero: == 退出码 77（expect 写 77），找不到的 want 0 条（阶段级 want 跳过 2 条）
rekey-red: == 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 4 条）
--- 开关 comparison-unit-unchecked
red: == 退出码 1（expect 写 1），找不到的 want 3 条（阶段级 want 跳过 5 条）
green: == 退出码 0（expect 写 0），找不到的 want 0 条（阶段级 want 跳过 3 条）
zero: == 退出码 77（expect 写 77），找不到的 want 0 条（阶段级 want 跳过 2 条）
rekey-red: == 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 4 条）
--- 开关 comparison-unit-count-loose
red: == 退出码 1（expect 写 1），找不到的 want 1 条（阶段级 want 跳过 5 条）
green: == 退出码 0（expect 写 0），找不到的 want 0 条（阶段级 want 跳过 3 条）
zero: == 退出码 77（expect 写 77），找不到的 want 0 条（阶段级 want 跳过 2 条）
rekey-red: == 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 4 条）
--- 开关 comparison-unit-substring
red: == 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 5 条）
green: == 退出码 1（expect 写 0），找不到的 want 1 条（阶段级 want 跳过 3 条）
zero: == 退出码 77（expect 写 77），找不到的 want 0 条（阶段级 want 跳过 2 条）
rekey-red: == 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 4 条）
--- 开关 comparison-driver-unchecked
red: == 退出码 1（expect 写 1），找不到的 want 1 条（阶段级 want 跳过 5 条）
green: == 退出码 0（expect 写 0），找不到的 want 0 条（阶段级 want 跳过 3 条）
zero: == 退出码 77（expect 写 77），找不到的 want 0 条（阶段级 want 跳过 2 条）
rekey-red: == 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 4 条）
--- 开关 comparison-product-unchecked
red: == 退出码 1（expect 写 1），找不到的 want 1 条（阶段级 want 跳过 5 条）
green: == 退出码 0（expect 写 0），找不到的 want 0 条（阶段级 want 跳过 3 条）
zero: == 退出码 77（expect 写 77），找不到的 want 0 条（阶段级 want 跳过 2 条）
rekey-red: == 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 4 条）
```
与文件头一致：`comparison-none-key-pair` 只让 rekey-red 判错（red 那两行换的是产物名，键照样变，照红）；`comparison-lag-growth-unchecked` 让 red（缺「none 行比基准多」与两行明细，共 3 条）与 rekey-red 判错；另六个开关不碰 rekey-red。doc-experiments 整道在 `stage-break-switch-replay-exempt.tsv` 里（`*`），复跑格不复跑它。

## K4-1 与 ⑪　doc-decisions 的挂账表、decision-links 的待回填清单改调库

### doc-decisions（`.claude/gate.d/doc-decisions.sh`）

- `LIB_LAG_BASELINE="$GATE_DIRECTORY/lib-lag-baseline.py"`（文件开头一行），三格起 python 时经环境变量传进去（与 `LIB_ITEM_REF_STATUS` 同一种传法）。
- DECISION_EVIDENCE_SOURCE 里 `base_ledger_text(diff_base)` 删了自己的 `git show {diff_base}:./{路径}`，改调库的 `baseline_text`，交（正文, 问题句）：不在 git 仓里或基准里没有这张表 (None, None)，取不到 (None, `<BaselineProblem.sentence()>，这张表只缩不涨没比`)。挂账表的行照它自己原来的读法切（不经 `content_rows`），判法不变。
- `ledger_growth(cell, rows, diff_base)` 改成自己取基准、交（涨了的行, 问题句）；三格把问题句并进挂账表的红行（「挂账表 … 有 N 处不对」下面一行），出路各加一句「读不出、取不到基准那一版的，按括号里 git 的报错修好仓库状态……」。`evidence-ledger-growth-unjudged` 开关照旧连基准都不取。
- 文件头：样本清单加 evidence-ledger-base-unreadable-red；开关表 red-swallowed、marker-ignored、check-ignored、cross-decision-status、inference-needs-verdict、three-way-leg-record、evidence-ledger-growth-unjudged 七条各补点名这份样本，新加 `LAG_BASELINE_BREAK=unreadable-as-absent` 一条；inference-needs-verdict 格上方的「存量挂账」与样本两行补了取不到判红。

新样本 evidence-ledger-base-unreadable-red（三格；挂账表与基准逐字相同，setup.sh 提交后删掉基准那一版挂账表的对象）。改前（草稿 `before/doc-decisions.sh`）：
```
  （基准 HEAD；跑了 3 格、红 0 格）
  ✓ 没有把已定的决策说成未定（扫 1 份 kb 文件，比对 1 条决策的状态行）
  ✓ 自陈是推论的依据条目都指到了存在的判决：扫 1 份决策、2 条依据条目，带推论记号的 0 条（指到判决 0 条，挂账 0 条；用户定案的「没走三方」0 条不算）
  ✓ 依据里写了「三方」的条目都有腿况记录：扫 1 份决策、2 条依据条目，写了「三方」的 2 条（指到存在的材料 0 条，写明腿况缺失 0 条，挂账 2 条）
```
改后（三格各一段，这里抄 three-way-leg-record 那一段，另两段同一句）：
```
  （基准 HEAD；跑了 3 格、红 3 格）
── three-way-leg-record
  ✗ 挂账表 .claude/gate.d/decision-evidence-lag.tsv 有 1 处不对
     读不出基准 HEAD 里的 .claude/gate.d/decision-evidence-lag.tsv（git show 退 128：fatal: bad object HEAD:./.claude/gate.d/decision-evidence-lag.tsv），这张表只缩不涨没比
     → 怎么办：这张表只收此刻判红、当场补不上指针的存量，一行一条：罩不住的行删掉（对象改好了就是它该退场），格式写坏的照表头改；
               新判红的条目补指针或腿况，别往表里加行；读不出、取不到基准那一版的，按括号里 git 的报错修好仓库状态（基准要存在、对象库没坏）再跑。
  [three-way-leg-record 退 1]
```
开关逐个（新样本）：
```
DOC_DECISIONS_BREAK= LAG_BASELINE_BREAK=:   （基准 HEAD；跑了 3 格、红 3 格） == 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 4 条） 
DOC_DECISIONS_BREAK=evidence-ledger-growth-unjudged LAG_BASELINE_BREAK=:   （基准 HEAD；跑了 3 格、红 0 格） == 退出码 0（expect 写 1），找不到的 want 3 条（阶段级 want 跳过 4 条） 
DOC_DECISIONS_BREAK=evidence-ledger-unjudged LAG_BASELINE_BREAK=:   （基准 HEAD；跑了 3 格、红 3 格） == 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 4 条） 
DOC_DECISIONS_BREAK=evidence-ledger-ignored LAG_BASELINE_BREAK=:   （基准 HEAD；跑了 3 格、红 3 格） == 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 4 条） 
DOC_DECISIONS_BREAK= LAG_BASELINE_BREAK=unreadable-as-absent:   （基准 HEAD；跑了 3 格、红 0 格） == 退出码 0（expect 写 1），找不到的 want 2 条（阶段级 want 跳过 4 条） 
```
回归：用到挂账表三格的 15 份样本，改前改后三格的输出逐行比（`cmp/` 下，`grep -a` 去掉判定行）：
```
cross-decision-status-ledger-green：改前改后逐行相同（6 行）
cross-decision-status-noun-phrase-red：改前改后逐行相同（11 行）
cross-decision-status-quoted-red：改前改后逐行相同（11 行）
cross-decision-status-red：改前改后逐行相同（11 行）
cross-decision-status-relaxed-green：改前改后逐行相同（6 行）
decision-documents-green：改前改后逐行相同（26 行）
decision-documents-some-cells-not-run：改前改后逐行相同（6 行）
evidence-ledger-base-unreadable-red：不同（29 行）
evidence-ledger-grew-red：不同（2 行）
evidence-ledger-red：不同（5 行）
evidence-pointer-history-green：改前改后逐行相同（9 行）
inference-needs-verdict-green：改前改后逐行相同（5 行）
inference-needs-verdict-red：改前改后逐行相同（10 行）
three-way-leg-record-green：改前改后逐行相同（5 行）
three-way-leg-record-red：改前改后逐行相同（12 行）
```
不同的三份：evidence-ledger-base-unreadable-red 是这一条要的；evidence-ledger-red、evidence-ledger-grew-red 只差出路那一句（`diff -a` 看过，红行与退出码不变）。改后 15 份对各自 expect：两份 decision-documents-* 带别的格、我的脚本只起这三格，不比 expect；其余 13 份退出码对、want 0 条缺。

### doc-experiments 的 decision-links（⑪ 的第八处）

- 起 python 时多传 `LIB_LAG_BASELINE`；python 里载入库，「待回填清单只减不增」那一段删了自己的 `git('ls-tree', …)`、`git('show', …)` 两段与两句自拼的问题句，改成 `lag_baseline.baseline_rows(base, pending_rel, cwd=root)`：问题句是 `<BaselineProblem.sentence()>，「只减不增」这一条没判`（与改前自拼的句子逐字相同）；基准里的行经 `content_rows` 之后照原来的正则 `^(E|D)(\d+)\s` 认。
- 格上方 ⑥ 的注释、样本说明、文件头样本清单与开关表（`LAG_BASELINE_BREAK=unreadable-as-absent` 一条，替掉原来「这一道没有一份样本会红」那句）跟着改；`LIB_LAG_BASELINE` 那一行上方的注释改成三格取基准。

新样本 decision-links-pending-base-unreadable-red（现场与 decision-links-green 相同，基准那一版待回填清单的对象删掉）。这一格改前就分清了三种结局，所以「该红的输入」是库的开关：改前开关够不着这一格，改后够得着。
```
### 改前，不设开关
== 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 3 条）
### 改前，设 LAG_BASELINE_BREAK=unreadable-as-absent
== 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 3 条）
### 改后，不设开关
      [待回填] 读不出基准 HEAD 里的 .claude/decision-links-pending（git show 退 128：fatal: bad object HEAD:.claude/decision-links-pending），「只减不增」这一条没判
== 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 3 条）
### 改后，设 LAG_BASELINE_BREAK=unreadable-as-absent
  ! 弄坏开关 LAG_BASELINE_BREAK=unreadable-as-absent 生效：读不出基准 HEAD 里的 .claude/decision-links-pending（git show 退 128：fatal: bad object HEAD:.claude/decision-links-pending），当成基准里没有这张表
      [待回填] .claude/decision-links-pending 新加了 D2：清单只减不增，新写的实验页与决策当场登记，不进清单（基准 HEAD 里还没有这张清单，这次改动新建了它，整张都算新加）
      [待回填] .claude/decision-links-pending 新加了 E2：清单只减不增，新写的实验页与决策当场登记，不进清单（基准 HEAD 里还没有这张清单，这次改动新建了它，整张都算新加）
  [缺 want] [待回填] 读不出基准 HEAD 里的 .claude/decision-links-pending（git show 退 128
  [缺 want] 「只减不增」这一条没判
  [缺 want] 决策与实验的登记有 1 处问题（待回填 1）
== 退出码 1（expect 写 1），找不到的 want 3 条（阶段级 want 跳过 3 条）
```
（上面两段「改后」是库的 show 改成 `./` 写法之前跑的；改了之后重跑，判定与 want 不变，读数在 `samples-doc-experiments-after.txt`。）
decision-links 九份样本（改后）：
```
decision-links-deleted-product-green: == 退出码 0（expect 写 0），找不到的 want 0 条（阶段级 want 跳过 3 条）
decision-links-formatting-green: == 退出码 0（expect 写 0），找不到的 want 0 条（阶段级 want 跳过 1 条）
decision-links-green: == 退出码 0（expect 写 0），找不到的 want 0 条（阶段级 want 跳过 3 条）
decision-links-no-experiment-green: == 退出码 0（expect 写 0），找不到的 want 0 条（阶段级 want 跳过 3 条）
decision-links-pending-base-unreadable-red: == 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 3 条）
decision-links-progress-mark-green: == 退出码 0（expect 写 0），找不到的 want 0 条（阶段级 want 跳过 3 条）
decision-links-progress-mark-red: == 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 3 条）
decision-links-red: == 退出码 1（expect 写 1），找不到的 want 0 条（阶段级 want 跳过 5 条）
decision-links-translation-green: == 退出码 0（expect 写 0），找不到的 want 0 条（阶段级 want 跳过 3 条）
```
multipath-registry 两份样本（库的 show 改了之后）：green 退 0、red 退 1，want 都不缺。

### 库（`.claude/gate.d/lib-lag-baseline.py`）

文件头「调它的七处」改成九处（加 decision-links 与 doc-decisions 挂账表）、K2-1 那一段、「路径按当前目录算」一段、命令行 `show` 的路径说明；`baseline_text` 的 show 写成 `<基准>:./<路径>`；`KNOWN_BREAKS` 加 `show-root-relative`；自证加「子目录里按当前目录取」一格（临时仓的 `sub/` 下放一份同名表，在 `sub/` 里取，写不写 `./` 都要取到它）。开关表 `unreadable-as-absent` 下补点名两份新样本。自证三种读数（`lib-selftest.txt`）：
```
### LAG_BASELINE_BREAK=
  ✓ 表的内容行
  ✓ 基准里有这张表
  ✓ 子目录里按当前目录取
  ✓ 基准里没有这张表
  ✓ 基准名取不到
  ✓ 基准那一版的对象缺了
  ✓ 命令行三种结局
  ✓ 复跑表的行
  ✓ 认不出的弄坏开关被拒
  ✓ lib-lag-baseline.py 自证：9 项都过
rc=0
### LAG_BASELINE_BREAK=unreadable-as-absent
  ! 弄坏开关 LAG_BASELINE_BREAK=unreadable-as-absent 生效：取不到基准 no-such-base 里的 lag.tsv（git ls-tree 退 128：fatal: Not a valid object name no-such-base），当成基准里没有这张表
  ! 弄坏开关 LAG_BASELINE_BREAK=unreadable-as-absent 生效：读不出基准 HEAD 里的 lag.tsv（git show 退 128：fatal: bad object HEAD:./lag.tsv），当成基准里没有这张表
  ✓ 表的内容行
  ✓ 基准里有这张表
  ✓ 子目录里按当前目录取
  ✓ 基准里没有这张表
  ✗ 自证失败：基准名取不到：取到 None、None
  ✗ 自证失败：基准那一版的对象缺了：取到 None、None
  ✗ 自证失败：命令行三种结局：有表 0、没有表 3、取不到 3（stderr 'BASELINE_BREAK=unreadable-as-absent 生效：读不出基准 HEAD 里的 lag.tsv（git show 退 128：fatal: bad object HEAD:./lag.tsv），当成基准里没有这张表'）
  ✓ 复跑表的行
  ✓ 认不出的弄坏开关被拒
  ✗ lib-lag-baseline.py 自证：9 项里 3 项没过
     → 怎么办：照上面点名的那一项修 baseline_text / content_rows / replay_rows；设了 LAG_BASELINE_BREAK 时这几项该红
rc=1
### LAG_BASELINE_BREAK=show-root-relative
  ✓ 表的内容行
  ✓ 基准里有这张表
  ✗ 自证失败：子目录里按当前目录取：取到 '# 表头注释\n\n甲\t理由一\n  # 缩进的注释\n乙\t理由二\n'、None；写成 ./ 开头取到 '# 表头注释\n\n甲\t理由一\n  # 缩进的注释\n乙\t理由二\n'、None
  ✓ 基准里没有这张表
  ✓ 基准名取不到
  ✓ 基准那一版的对象缺了
  ✓ 命令行三种结局
  ✓ 复跑表的行
  ✓ 认不出的弄坏开关被拒
  ✗ lib-lag-baseline.py 自证：9 项里 1 项没过
     → 怎么办：照上面点名的那一项修 baseline_text / content_rows / replay_rows；设了 LAG_BASELINE_BREAK 时这几项该红
rc=1
```
（文件头改了一行说明之后又跑了一次不带开关的：「✓ lib-lag-baseline.py 自证：9 项都过」，退 0。）

### 验收那条 grep

`grep -n "git', 'show'\|git show" .claude/gate.d/doc-decisions.sh .claude/gate.d/doc-experiments.sh`（仓根上）原样：
```
.claude/gate.d/doc-experiments.sh:97:#       decision-links-pending-base-unreadable-red（输出里找不到「[待回填] 读不出基准 HEAD 里的 .claude/decision-links-pending（git show 退 128」）
.claude/gate.d/doc-experiments.sh:486:    echo "               再 git show <提交>^:<路径> 读回来）；取不回来说明它从没存在过，"
.claude/gate.d/doc-experiments.sh:888:# .claude/decision-links-pending（git show 退 128」；弄坏开关 LAG_BASELINE_BREAK=unreadable-as-absent 下当成基准里没有这张清单，红在「新加了」，那一条 want 找不到。
.claude/gate.d/doc-experiments.sh:1886:    """git show 取出的归档产物字节，按文件名解压（若压过）后逐行去首尾空白；取不出返回 None。"""
.claude/gate.d/doc-experiments.sh:1887:    blob = subprocess.run(['git', 'show', revision_path], capture_output=True)
.claude/gate.d/doc-decisions.sh:72:#                                           三格都红在「读不出基准 HEAD 里的 .claude/gate.d/decision-evidence-lag.tsv（git show 退 128…），这张表只缩不涨没比」
.claude/gate.d/doc-decisions.sh:698:    head = subprocess.run(['git', 'show', f'HEAD:{path}'], capture_output=True, text=True)
.claude/gate.d/doc-decisions.sh:2746:  howto "  项里写结论、依据与改前（改前写一行 git show <提交>:<路径>，不抄原文）；一条改动点名了几条决策，只在标题里点名的第一条决策的节里写全文，别的节写一行「- 见 D<n>（简称） 节同日条目「摘要」」。"
.claude/gate.d/doc-decisions.sh:2767:    git show "$diff_base:$hist" > "$base_copy"
```
逐行：doc-experiments:97、888 与 doc-decisions:72 是这一批写的样本说明；doc-experiments:486、doc-decisions:2746 是出路句里教人取历史；doc-experiments:1887 取归档产物（quoted-result-lines）；doc-decisions:698 取 HEAD 上的决策正文（stale-open-items）；doc-decisions:2767 取基准那一版变更史（history-loss）。没有一行是「取基准那一版的滞后表、挂账表、待回填清单」。decision-links 里另有 `git('show', f'{base}:{rel}')` 两处（doc-experiments:1286、1327），取的是实验页与决策正文的基准那一版做逐行比，不是只缩不涨的表，没动。

## K4-2　`.claude/kb-directories` 的读法只在 quote-kb.py 一处

### scripts/kb/quote-kb.py

- 新函数 `kb_directory_entries(root)` →（登记的目录, 拒掉的行 [(行号, 原文, 为什么)]）：`#` 之后是注释、空行跳过；`os.path.normpath` 规整（开头的 `./`、结尾的 `/`、中间的 `./` 与重复的 `/`）；拒三种：绝对路径、规整后以 `..` 起头（走出仓）、规整后是 `.`（仓根本身，派发没点这一种，同一类「拼出的模式配不上 git 路径」的静默，一起拒了）；目录不在的照旧不算、不报。
- `kb_candidate_roots(root)` 调它，被拒的行逐行打到 stderr（`  ✗ .claude/kb-directories 第 N 行「…」<为什么>，这一行不认，那个目录不算 kb 目录` 下接 `     → 怎么办：写成相对仓根的目录……`），同一个进程里同一行只报一次；其余（排序去重、没登记或都不认就是 `.claude/kb`）不变。
- 开关 `QUOTE_KB_BREAK=kb-directories-raw`：退回改前的读法（原串只去首尾空白与结尾 `/`）。文件头开关表加一条（mutate.sh 写法），I 号那一段的读法说明改成「读法与规整只在这里一份，doc-registries.sh 与 stale-candidates.py 都调它」。
- 自证加两格（`selftest_in` 末尾）：「登记位写成 ./ 开头、/ 结尾、中间重复 / 的，规整成相对仓根的写法、去重」；「登记位写成绝对路径、走出仓、仓根本身的行被拒，stderr 逐行点名并给出路，同一行只报一次」。

改前改后的同一份输入（`tools/kb_roots_probe.py`，改前取的是草稿 `before/quote-kb.py`）：
```
name=规整 改前 roots=['./crates/x/invariants', '.claude//kb'] stderr_lines=0
name=规整 改后 roots=['.claude/kb', 'crates/x/invariants'] stderr_lines=0
name=拒绝 改前 roots=['.', '../outside', '<临时目录>/repo/crates/x/invariants'] stderr_lines=0
name=拒绝 改后 roots=['.claude/kb'] stderr_lines=6
```
自证：
```
$ python3 scripts/kb/quote-kb.py --selftest      （退 0；输出里这两格）
  ✓ 自检：登记位写成 ./ 开头、/ 结尾、中间重复 / 的，规整成相对仓根的写法、去重
  ✓ 自检：登记位写成绝对路径、走出仓、仓根本身的行被拒，stderr 逐行点名并给出路，同一行只报一次
$ QUOTE_KB_BREAK=kb-directories-raw python3 scripts/kb/quote-kb.py --selftest      （退 1）
  ✗ 自检：登记位写成 ./ 开头、/ 结尾、中间重复 / 的三行，应当规整成 ['.claude/kb', 'crates/x/invariants']、一行不拒，实际 ['./crates/x/invariants', '.claude//kb', 
     → 怎么办：看 kb_directory_entries() 的规整（os.path.normpath）；QUOTE_KB_BREAK=kb-directories-raw 设着的话这里本来就该红。
```
（开关下第一格就退 1，拒绝那一格走不到；拒绝那一格的判别力靠上面的改前改后读数：改前四种写法全认。）

### scripts/kb/stale-candidates.py

- 自证加一格：在已有的建候选那一步之后，把临时仓的登记位改写成 `./.claude/kb/` 与 `./crates/singlefs-checker/invariants`，再建一次候选，要求 `crates/singlefs-checker/invariants/01-sample.md:1` 与 `.claude/kb/checks-owed.md:1` 都在候选里，建完把登记位换回去。
- 文件头开关表：`invariants-carrier-missing` 下补点名这一格；新加 `QUOTE_KB_BREAK=kb-directories-raw  归 quote-kb.py` 一条，点名这一格；`kb_directories` 的说明与成功句各补半句。代码里 `kb_directories` 不用改（模式由 quote-kb.py 交来的规整写法拼）。

先加这一格、quote-kb.py 还没改时（该红的输入）：
```
  ✗ 自证失败：登记位写成 ./crates/singlefs-checker/invariants（./ 开头）时不变量清单里的现状句没进候选表：[('F1', 'CLAUDE.md:3'), ('F1', 'records/2026-01-01-log.md:1'), ('F3', 'records/2026-01-01-log.md:1')]
  ✗ stale-candidates 自证没过
rc=5
```
改后逐个开关（`stale-selftest-switches.txt`，✗ 行截 170 字）：
```
### stale-candidates --selftest 
  ✓ stale-candidates 自证通过：变更记录（含改名）与差异片段认对；事实表八种写法各判红…
rc=0
### stale-candidates --selftest STALE_CANDIDATES_BREAK=invariants-carrier-missing
  ✗ 自证失败：不变量清单（crates/singlefs-checker/invariants/ 下）里的现状句没进候选表
  ✗ 自证失败：登记位写成 ./crates/singlefs-checker/invariants（./ 开头）时不变量清单里的现状句没进候选表：[('F1', '.claude/kb/checks-owed
  ✗ stale-candidates 自证没过
     → 怎么办：对照 selftest 里造的小仓，逐格看 new_change_entries、changed_segments、check_facts、b
rc=5
### stale-candidates --selftest QUOTE_KB_BREAK=kb-directories-raw
  ✗ 自证失败：登记位写成 ./crates/singlefs-checker/invariants（./ 开头）时不变量清单里的现状句没进候选表：[('F1', 'CLAUDE.md:3'), ('F1', 
  ✗ stale-candidates 自证没过
     → 怎么办：对照 selftest 里造的小仓，逐格看 new_change_entries、changed_segments、check_facts、b
rc=5
### stale-candidates --selftest STALE_CANDIDATES_BREAK=wide-unchecked
  ✗ 自证失败：命中超过抽查门槛、出处列没写「宽词已抽查」的事实表没被判红
  ✗ stale-candidates 自证没过
     → 怎么办：对照 selftest 里造的小仓，逐格看 new_change_entries、changed_segments、check_facts、b
rc=5
### stale-candidates --selftest STALE_CANDIDATES_BREAK=dirty-worktree-unchecked
  ✗ 自证失败：--changes 不给 --target、工作区有改动应当退 2 且不写变更清单：退 0
  ✗ 自证失败：--changes 不给 --target 的拒绝没写 batch-target.py 的出路：  ✓ 变更清单 /tmp/tmpi4klqfh9/dirty-no-target.md：1 条变更记录
  ✗ 自证失败：--allow-dirty-worktree 应当照跑并在输出头一行写明放行与理由：退 0，  ✓ 变更清单 /tmp/tmpi4klqfh9/dirty-allowed.md：1 条…
  ✗ 自证失败：--allow-dirty-worktree 的理由不到 8 个字应当退 2：退 0
  ✗ stale-candidates 自证没过
     → 怎么办：对照 selftest 里造的小仓，逐格看 new_change_entries、changed_segments、check_facts、b
rc=5
### stale-candidates --selftest STALE_CANDIDATES_BREAK=provenance-unchecked
  ✗ 自证失败：出处已跟踪且有没进这一批的改动的事实表应当退 6
  ✗ 自证失败：出处引的行号被改了的事实表应当退 6
  ✗ 自证失败：出处是暂存区里的新文件的事实表应当退 6
  ✗ 自证失败：出处已跟踪且只暂存区有改动的事实表应当退 6
  ✗ 自证失败：出处已跟踪且有改动的事实表没点名是哪一行哪个路径：  ✓ 事实表罩全了：1 条变更记录都有出处，1 行事实的…
  ✗ 自证失败：出处是未跟踪新文件的事实表没另列一行名单：  ✓ 事实表罩全了：1 条变更记录都有出处，1 行事实的检索词都…
  ✗ stale-candidates 自证没过
     → 怎么办：对照 selftest 里造的小仓，逐格看 new_change_entries、changed_segments、check_facts、b
rc=5
```
`--benchmark`（退 0）：「✓ 阳性对照：已知的 25 处腐烂里 25 处在候选里（下限 25）（盲写事实表 60 行，候选 4190 行、2821 个不同的行）」，与上一批的读数相同。
攻方探针 `probes/kb_directories_probe.py` 改后复跑（`kb-directories-probe-after.txt`，探针把 quote-kb.py 载入两次，拒绝句因此出两遍）：
```
name=form:crates/x/invariants quote_kb_roots=['.claude/kb', 'crates/x/invariants'] stale_candidates_carrier=True
name=form:./crates/x/invariants quote_kb_roots=['.claude/kb', 'crates/x/invariants'] stale_candidates_carrier=True
name=form:<探针根>/crates/x/invariants quote_kb_roots=['.claude/kb'] stale_candidates_carrier=False
name=form:crates/x/invariants// quote_kb_roots=['.claude/kb', 'crates/x/invariants'] stale_candidates_carrier=True
```

### .claude/gate.d/doc-registries.sh

`kb_candidate_roots` 的 bash 读法（13 行）删了，改起一段 python 载入 `$REPOSITORY/scripts/kb/quote-kb.py` 调它的 `kb_candidate_roots`，每行打一个目录；被拒的行由 quote-kb.py 打到 stderr（阶段输出里看得见）；python 起不来时打「✗ 读不出 kb 目录登记位……」与出路到 stderr、函数退 1（调用方都是 `< <(…)` 或 `> 文件`，取不到退出码，候选目录是空的，那几格会因为找不到不变量表而红，不静默变绿）。函数上方加两行注释写读法在哪一份。

改前改后对同一份根取一次（`tools/roots_fn.sh` 把函数原样取出来起；草稿路径换成 `<草稿>`）：
```
### 真仓：改前 / 改后
.claude/kb
crates/singlefs-checker/invariants
.claude/kb
crates/singlefs-checker/invariants
### 登记位：./crates/x/invariants/|.claude/kb
改前: .claude/kb  ./crates/x/invariants
改后: .claude/kb  crates/x/invariants
### 登记位：<草稿>/roots.U6vXfS/crates/x/invariants|../landing-audit-r2fix-gates/roots-outside
改前: <草稿>/roots.U6vXfS/crates/x/invariants
改后: 两行拒绝句（第 1 行是绝对路径、第 2 行规整后走出了仓）各接出路，候选目录 .claude/kb
### 登记位：（空）
改前: .claude/kb
改后: .claude/kb
```
（改前、改后每段末尾都是 rc=0，这里并成一行写。）doc-registries 带登记位的 10 份样本，候选目录改前改后逐个相同（checks-owed-*、inline-sums-*、invariant-counts-translation-*、references-* 八份是「.claude/kb crates/sample/invariants」，invariant-prerequisites-* 两份是「crates/sample/invariants」）。

## 语法检查

```
bash -n .claude/gate.d/doc-experiments.sh：过
bash -n .claude/gate.d/doc-decisions.sh：过
bash -n .claude/gate.d/doc-registries.sh：过
编译了 60 段，编译不过 0 段          ← tools/syntax.py：三份 .py 整份，三道门禁里每段 python heredoc（含 DECISION_EVIDENCE_SOURCE）各一段
bash -n 三份新样本的 setup.sh：过
```
另起过的 light 自证（都退 0）：`lib-lag-baseline.py --selftest`（9 项）、`quote-kb.py --selftest`、`stale-candidates.py --selftest` 与 `--benchmark`、调 quote-kb.py 的 `scripts/agents/three-way-materials.py --selftest`（「✓ 自检通过（查了 10 项）」）、`scripts/agents/checklist-specs.py --selftest`（「查了 4 条」）、`scripts/kb/kb-sections.py --selftest`。命名检查没跑：这一批没改 `crates/`、`scripts/` 下的 `.rs`（新样本里的 `crates/sample/units.rs` 在 `.claude/gate.d/fixtures/` 下）。

## 要主 agent 跑的命令

工作目录都是主工作区仓根 `/home/fy5090/code/singlefs`，不起 cargo，内存与线程没有特别要求。样本档要样本与 HEAD 加暂存区逐字节相同：先暂存这一批（「改过的文件」里六份、一份 expect、三个新样本目录，连同上一批还没暂存的 `lib-lag-baseline.py`）再跑。

1. 三道的样本档（判：末行退 0、没有一份判错）：
```
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh、doc-decisions.sh、doc-registries.sh 三道的样本档; 范围=3 道阶段; 为什么=这一批改了 replay-comparison-units 的键、decision-links 与挂账表三格取基准的写法、kb 目录登记位的读法，加了三份样本、改了一份' bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item doc-experiments.sh --item doc-decisions.sh --item doc-registries.sh
```
2. 库的开关红在点名那几份（判：判错的样本正好是 decision-links-pending-base-unreadable-red、evidence-ledger-base-unreadable-red、reading-discipline-red 三份）：
```
LAG_BASELINE_BREAK=unreadable-as-absent SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh、doc-decisions.sh、code-source-discipline.sh 三道的样本档，设 lib-lag-baseline.py 的弄坏开关; 范围=3 道阶段; 为什么=核库的开关打开后红在三道头注点名的那三份' bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item doc-experiments.sh --item doc-decisions.sh --item code-source-discipline.sh
```
3. 新键的开关（判：判错的样本正好是 replay-comparison-units-rekey-red 一份）：
```
EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-none-key-pair SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的样本档，设 comparison-none-key-pair; 范围=1 道阶段; 为什么=核退回旧键后只有借键那一份判错' bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item doc-experiments.sh
```
4. 复跑方（stage-break-switch-replay 复跑 doc-decisions 头注里补了点名的七条与新加的 LAG_BASELINE_BREAK；break-switch-replay 复跑 stale-candidates.py 头注两条；break-switch-headers 判 quote-kb.py、stale-candidates.py 的新条目三样写全）：
```
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 stage-break-switch-replay、break-switch-replay、break-switch-headers 三格; 范围=3 格; 为什么=doc-decisions 头注七条开关补点名新样本、加 LAG_BASELINE_BREAK 一条，stale-candidates.py 与 quote-kb.py 头注各加开关' bash .claude/gate.d/code-tooling.sh --check stage-break-switch-replay,break-switch-replay,break-switch-headers
```
5. 真仓单跑改过的格与用库的格（判：都绿；「查了多少」与上一次提交时的读数相同；doc-registries 的输出里没有「.claude/kb-directories 第 N 行」拒绝句；replay-comparison-units 报 14 行、none 4 行）：
```
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 decision-links、multipath-registry、replay-comparison-units 三格; 范围=3 格; 为什么=三格取基准经 lib-lag-baseline.py，replay-comparison-units 的键改了' bash .claude/gate.d/doc-experiments.sh --check decision-links,multipath-registry,replay-comparison-units
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 cross-decision-status、inference-needs-verdict、three-way-leg-record 三格; 范围=3 格; 为什么=挂账表取基准改调 lib-lag-baseline.py，取不到判红' bash .claude/gate.d/doc-decisions.sh --check cross-decision-status,inference-needs-verdict,three-way-leg-record
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-registries.sh 用 kb_candidate_roots 的六格; 范围=6 格; 为什么=kb 目录登记位的读法改调 quote-kb.py' bash .claude/gate.d/doc-registries.sh --check audit-contradictions,declared-counts,invariant-count-elsewhere,invariant-anchors,inline-sums,invariant-prerequisites
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 mutation-tables、reading-discipline 两格; 范围=2 格; 为什么=库的 git show 改成按当前目录取，两格是库的调用方' bash .claude/gate.d/code-source-discipline.sh --check mutation-tables,reading-discipline
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-process-records.sh 的 crates-adversarial-review、agent-def-adversarial-review 两格; 范围=2 格; 为什么=库的 git show 改成按当前目录取，两格是库的调用方' bash .claude/gate.d/doc-process-records.sh --check crates-adversarial-review,agent-def-adversarial-review
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 heavy-thread-defaults、long-run-header-exempt、research-script-selftests 三格; 范围=3 格; 为什么=前两格是库的调用方，后一格跑库、quote-kb.py、stale-candidates.py 的自证' bash .claude/gate.d/code-tooling.sh --check heavy-thread-defaults,long-run-header-exempt,research-script-selftests
```
6. 提交前那一趟会判到的（gate-lint、shell-lint、preflight-lint 判改过的三道与两份脚本；gate-overlap 判 doc-registries 删掉的那份读法与新的 python 段）：整轮 `scripts/gate/gate-staged.sh` 里跑，不单列。

## 没核动的（读代码推的，没实跑）

- stage-break-switch-replay 复跑 doc-decisions 时新样本在哪几条开关下判错：我实跑了 evidence-ledger-growth-unjudged、evidence-ledger-unjudged、evidence-ledger-ignored、LAG_BASELINE_BREAK=unreadable-as-absent 四条（只起三格的 python，不经阶段）；red-swallowed、marker-ignored、check-ignored 与 cross-decision-status、inference-needs-verdict、three-way-leg-record 三条「这一格不跑」是照阶段的写法推的（前三条是共用库的行为，后三条跳过的那一格少一行「<格>：红（退出 1）」）；其余开关（KB_* 四条、cross-decision-status-* 六条、evidence-pointer-*、inference-*、three-way-leg-*）按样本内容推的不碰它（单份中文决策、没有「未定」、没有推论记号、腿况那两条都被挂账罩住）。第 4 条命令实跑会核。
- doc-experiments 的 decision-links-pending-base-unreadable-red 在 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK 各条下是否判错没逐条跑（这一道整道在复跑豁免表里）；现场与 decision-links-green 相同，按 green 在各开关下的行为推，会让它判错的是 STAGE_CELLS_BREAK 那几条（红按绿记、不认 .gate-cells、不认 --check）与 LAG_BASELINE_BREAK=unreadable-as-absent（实跑过），EXPERIMENT_PAGES_AND_PRODUCTS_BREAK 各条碰不到它。
- 两份 decision-documents-* 样本带别的格，我的脚本只起挂账表三格，它们的 expect 没比；三格的输出改前改后逐行相同。
- doc-registries 的样本档没在草稿里逐格起（只把 `kb_candidate_roots` 取出来核了 10 份带登记位的样本与真仓的读数）；第 1 条命令实跑会核。
- 库的 `git show` 改成 `<基准>:./<路径>` 之后，code-source-discipline、doc-process-records、code-tooling 三道用库的格没在草稿里重跑样本；它们都在仓根上起，两种写法取到同一份，推的；第 5 条命令与第 1、2 条会核到。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 broad-mutation.py 起 cargo 的用法、gate.sh、全量 cargo test、check.sh）；没跑门禁阶段与各类 lint（样本档要暂存后跑，命令在上一节）；没走定义三方（这一批没改定义、共用约束、规则）；没提交，没暂存。
- `.claude/gate.d/decision-evidence-lag.tsv` 文件头第 9 行写「基准那一版没有这张表、或不在 git 仓里时不比」，没补「git 取不到判红」（不在派发列的文件里）。
- 攻方 K2-1 甲还提到 `research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md` 第 63 行那张「七处」表补一句；research/ 不在我写范围里，没改。
- 攻方 K4-1 另点了 `.claude/gate.d/checker-independence-and-sync.sh:670`（取基准那一版的格式常数，取不到交空串，是不是只缩不涨攻方没核）；doc-decisions 的 history-loss 格 `git show "$diff_base:$hist"`（doc-decisions.sh:2767）取基准变更史；两处都不在这一批，没查也没改。
- 攻方 K4-2 末段「写回员写范围 `crates/singlefs-checker/invariants/*.md` 不跨子目录」那一形没碰（`.claude/hooks/agent-write-scope.tsv` 不在这一批）。
- `.claude/batch-scope` 没登记这一批的文件（不在写范围里）。
- `selftest-registry.tsv` 没加行：库仍在 `.claude/gate.d/` 顶层（原因同上一批报告 K2-1 末段）。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/landing-audit-r2fix-gates/` 里是小脚本（`tools/`）、改前快照（`before/`，六份文件与两道门禁的样本目录）、改前改后的 diff（`after-*.diff`）、对照用的暂存区那一版门禁（`staged/doc-experiments.sh`）、K2-2 对照用的样本拷贝（`control-staged-rekey/`，rekey-red 加一张滞后表，不是仓副本、没有 .git）与各次读数；没有仓副本、工作树与编译目录。
