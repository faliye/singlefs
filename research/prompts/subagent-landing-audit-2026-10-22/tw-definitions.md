# 子 agent 结果落地审计第三批任务 11：定义与共用约束的文字（七条）

批次：records/2026-10-22-子agent结果落地审计.md#3。只改文字，没动钩子与脚本。

## 改过的文件（git diff --stat，对暂存区）

```
 .claude/agent-common.md                 | 5 +++--
 .claude/agents/experiment-designer.md   | 5 +++--
 .claude/agents/experiment-runner.md     | 4 +++-
 .claude/agents/implementation-writer.md | 2 +-
 .claude/agents/kb-scribe.md             | 7 ++++---
 .claude/agents/sweep.md                 | 1 +
 .claude/main-agent.md                   | 8 +++++---
 7 files changed, 20 insertions(+), 12 deletions(-)
```

sha256（收尾时）：
```
67336726d48029d5f38ed048ea3134835ca0538a0ed0758facf452ca486634f4  .claude/agent-common.md
0de095867e9cbb1942c2ae492b9f1fd17bd593693e5ea2108eedfe7560564d69  .claude/main-agent.md
0c9ba8f6b140de9771d75d8b9c02d9f44411ec2d74db04d67bb37b09a005d303  .claude/agents/experiment-runner.md
fcba610b32af36330158caf2d3073458f266308a097c5bed8383d5977d8469e1  .claude/agents/experiment-designer.md
6038d9e855406fd1d7e18f4a4b32eed32ef15172187760fcf845f050aa46d68e  .claude/agents/sweep.md
adcbdf786ec0cc29e1566c4b93d7f006aa543b144696a95f401620faf5d0dff8  .claude/agents/kb-scribe.md
1355b7467a9c5753fb1927536e818b4a8a9485fe91d42d1ccad43cc592bd0c8e  .claude/agents/implementation-writer.md
```

## 每条改了什么、该红的输入、现有检查红不红

样本都在 `~/.local/share/singlefs/work/landing-audit-tw-definitions/samples/`。

### 1. K4 执行员逐条自查
- 改：experiment-runner 做什么新增 3c（登记每条读法、量、停机与作废条款列成清单，逐条写函数与会红的用例，写不出停下交回）；产出新增「登记落地清单」一节。
- 该红的输入：`k4-prereg-unlanded.md`（读法 7、守护定义、停机条款 S1）配 `k4-device-unlanded.rs`（只有读法 7 一个函数一条用例）与 `k4-checklist-missing-rows.md`（清单缺两行）。
- 现有检查：不会红。doc-experiments.sh 的 preregistration-parallelism 格文件头自己写「管不到：写的数对不对、装置是不是照它实现」；code-source-discipline 的 absolute-assertions、reading-discipline 两格只判有没有钉绝对值断言、种子折叠与恒为 0 的读数，不读登记。全仓 `grep -rn "登记落地清单"` 在 gate.d、scripts/kb、scripts/agents、hooks 里 0 处。
- 要立的检查：判登记第六、七、十一节的每个量与条款号，在实验页（或报告落盘处）的「登记落地清单」表里各有一行，行里的函数名与用例名在装置源码里 `grep` 得到、用例带 `#[test]`；读 `research/prompts/e<号>-preregistration.md` 与对应实验页和装置源码。

### 2. K3 设计员核可跑性与判别力
- 改：experiment-designer 做什么新增 3a（每个量现算态数或规模乘单态耗时得挂钟与内存；每个对照点写预期差与测量分辨率；登记点上的触发事件写触发条件并代入几何；算不出的写进第一节末尾「未定项：」一段，不写死）；固定节名表第一节一行与产出一行同步。
- 该红的输入：`k3-prereg-infeasible.md`（末段 1207959552 态每态约 10 毫秒却写「预计挂钟：2 小时内」；预期差 0.3 毫秒而计数每 1 毫秒前进一次；环容量 4096 写入总量 1000 却要搬空发布）。
- 现有检查：不会红。preregistration-parallelism 格只判五个标签行在、冒号后不少于 8 个字、挂钟与核数两行带数字（doc-experiments.sh 该格注释，判据与「管不到」两段），样本里五行齐全且带数字。
- 要立的检查：判登记第十四节「预计挂钟：」的数与第六节各量写出的「态数 × 单态耗时」乘积差一个数量级以上判红；第六节写了「分辨率」的量，其预期差小于分辨率判红；读 `e<号>-preregistration.md` 的第六、十四节。后两项形态不固定，先立固定形态（量表加「态数」「单态耗时」「分辨率」「预期差」列）再立格。

### 3. K1 交回前跑命名检查
- 改：agent-common 门禁一节新增「命名检查交回前跑」一条（code-source-discipline 的 vague-names、test-file-names、ordinal-names、data-names 四格带范围声明，加 naming-lint.sh；只修命中在自己改动文件里的）；原「不跑各类 lint」那句加括注指到它。
- 该红的输入：implementation-writer、experiment-runner、tooling-writer 各发这条新命令。
- 现有钩子读数（把 hook 输入 JSON 喂 `.claude/hooks/*.sh`，三种 agent_type 一样）：
  - `heavy-test-guard.sh` 对 code-source-discipline 那条：exit=2，「提交时才跑的检查被拒：门禁阶段 code-source-discipline.sh」（三种类型都是）。
  - `heavy-test-guard.sh` 对 naming-lint.sh 那条：exit=0。
  - `run-scope-guard.sh` 对写了「范围=4 格」的声明：exit=0；改成「范围=3 格」：exit=2，拒绝说明「范围：4 格」。
- 结论：第 12 号任务让 heavy-test-guard 放行这条命令（子 agent 对这四格的 `--check`）之前，子 agent 照新条文跑会被拒。文字里的范围声明已与 run-scope-guard 对得上。

### 4. 延后项当时就登记
- 改：main-agent「一轮怎么开、怎么收」第 2 条加「延后的当时就登记」。
- 该红的输入：`k4b-dispatch-record-deferred.md`（一行写延后、没有编号或 文件:行）。
- 现有检查：不会红。gate.d、hooks、scripts/gate、scripts/kb、scripts/agents 的 `.sh` 与 `.py` 里含「延后」的文件数为 0（`grep -rln '延后' … | wc -l` 得 0）；doc-registries 的 table-shape、closeout-collects-open 与 doc-process-records 的 milestone-close-owed 读的是欠账表与收口表，不读调度记录。
- 要立的检查：判 `records/` 新增或改动的调度记录里，含「延后」的行同一行（或同一列表项）带 `C<数字>`、`checks-owed.md:<行>`、收口表行号或决策号之一，且它指到的行存在；读 `records/*.md` 的新增行与 `.claude/kb/checks-owed.md`。

### 5. 回扫的 `crates/` 行有人接
- 改：sweep 产出新增「交实现员」一节（目标在 `crates/` 下判要改、要补的行，`crates/singlefs-checker/invariants/` 下的 `.md` 除外，没有也写 0 行）；main-agent 派发表「一批阶段任务结束」一行写明并进第 4 条下一批交 implementation-writer，不交写回员。
- 该红的输入：`k5-candidates.tsv`、`k5-report-gap.md`（逐行判定里有一行 crates 下的要改，没有「交实现员」）。
- 现有检查：`python3 scripts/kb/stale-candidates.py --check-report samples/k5-candidates.tsv samples/k5-report-gap.md` 输出「✓ 报告判全了：1 行候选都有逐行判定」，exit=0，不红。`grep -c "交实现员" scripts/kb/stale-candidates.py` 为 0。
- 要立的检查：判回扫报告「逐行判定」里判定「要改」「要补」且目标路径以 `crates/` 开头（不含 invariants 下的 `.md`）的每一行，在「交实现员」一节里出现；读 `--check-report` 的报告。放在 `--check-report` 里最省。

### 6. 写回员遇到目标文件不在
- 改：kb-scribe 开头加规格目标不存在时整份不写、报告「找不到的目标」一节（含 `git log --diff-filter=DR` 与 grep 查到的新位置）、不跳过；写范围加 `crates/singlefs-checker/invariants/*.md`（只限 `.md`）；输入一节的闸说明与产出同步；main-agent 派发表「书记员不改 `crates/`」改成带例外。
- 该红的输入：`k6-moved-target-spec.json`（目标 `.claude/kb/invariants.md`，已拆进 invariants 目录）。
- 现有检查：`python3 scripts/kb/kb-spec-check.py samples/k6-moved-target-spec.json` exit=1，「第 1 条（.claude/kb/invariants.md）：目标文件不在」，规格阶段已红。写闸：喂 `write-guard.sh` 一条 kb-scribe 的 Edit 到 `crates/singlefs-checker/invariants/README.md`，exit=2，「kb-scribe 的写范围不含…」，要等第 12 号任务加写范围表那一行。
- 要立的检查：无新增（规格检查已覆盖规格阶段）；「找不到的目标」一节有没有、条数对不对，没有现成输入可读（报告在工作根，不进仓）。

### 7. 报告文件名
- 改：agent-common「报告」一节两处（默认路径 `report.md` 改 `result.md`；加「报告文件名一律 `result.md`，不用名字里带 report 的文件」）；main-agent「派发提示怎么写」加一段；implementation-writer 补丁目录里的 `report.md` 改 `result.md`。
- 该红的输入：子 agent 用 Write 建 `report.md`。读数：本次真试了一次，工具层返回「Subagents should return findings as text, not write report files.」；`write-guard.sh` 对 `patch/report.md` 与 `patch/result.md` 都是 exit=0（钩子不管这个名字）。
- 现有检查对改名的反应：`scripts/gate/apply-writer-patch.py` 第 15、218 行读 `<补丁目录>/report.md`，补丁动 `crates/singlefs-checker/src/` 而该文件不在时报「report.md 里却没有「受影响的层 0 流与崩溃枚举用例」一节」。实现员改叫 `result.md` 之后，动 checker 的补丁会被它拒。
- 交主 agent：`scripts/gate/apply-writer-patch.py` 不在第 10、12、13 号任务的文件清单里，需要有人改成认 `result.md`（或两个名字都认）；同类还有 `scripts/agents/cite-check.py` 第 312、323 行与 `.claude/hooks/handback-guard.sh` 第 261–283 行的 `report.md`（这两处在第 12 号清单里）。

## 文字检查（门禁格交主 agent 跑，我这边被拒）

tooling-writer 起 rules-lint、doc-lint 与门禁阶段被 `heavy-test-guard.sh` 拒（共用约束「门禁」一节也不让），没有绕。替代：把 rules-lint 的解释性半句、词法说明、带日期三类判据与 doc-lint 的位置指代、自称判据（`doc-lint.sh` 455–510 行的 grep 式，原样抄）分别用 grep 与 python 对我新增的 20 行跑一遍：
- 位置指代与自称：0 处命中；同一组式子对「写法见上文，如下：」「本节只写做法」命中 2 行（红样本）。
- rules-lint 三类（`$W/prose-check.py`）：新增 20 行 0 处命中；红样本 `bad-lines.tsv`（「，因为…（日期）」「实测 8 次，所以…」）命中 2 行。
- 这只是对判据的复刻，不等于那两道门禁绿。要主 agent 在提交前整轮里读 `规则纪律（项目本地）` 与 `文档铁律`（D-1、D-2）两格的读数。

## 要主 agent 跑的命令
- 提交前整轮门禁里的「规则纪律（项目本地）」「文档铁律」，看这七份文件有没有命中。
- 定义三方（门禁 doc-process-records 的 agent-def-adversarial-review 格）：这七份都要点名。

## 没做什么
- 没改 `.claude/hooks/`、`scripts/`：heavy-test-guard 放行命名检查、写范围表加 kb-scribe 的 invariants 行、报告名在钩子与脚本里的认法（含 apply-writer-patch.py）、stale-candidates.py 的载体模式，都留给第 10、12 号任务与上面「交主 agent」那条。
- 没跑门禁阶段与各类 lint；没走定义三方；没提交。
- sweep 的「交实现员」我加了 invariants 下 `.md` 不进该节的括注（让它与第 6 条写回员能写那批清单一致）；规格没写这半句，主 agent 觉得多余可删。
- 3c 里「用例靠改坏函数一处证红的照 show-me-test 办」是引已有规则，不是新条款。
- 要推翻这些结论的现象：第 12 号任务改完钩子后，上面 K1 的两条命令仍被 heavy-test-guard 拒；或 `--check-report` 对缺「交实现员」的报告报红（说明已有人立了检查）。
