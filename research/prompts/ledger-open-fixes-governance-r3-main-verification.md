# ledger-open-fixes-governance-r3：判决（2026-11-19）

正文 `research/prompts/_ledger-open-fixes-governance-r3-body.md`，开工快照 `research/prompts/ledger-open-fixes-governance-r3-snapshot/`。这是这一题治理文件轮的第三轮，也是最后一轮；前两轮判决 `research/prompts/ledger-open-fixes-governance-r1-main-verification.md`、`research/prompts/ledger-open-fixes-governance-r2-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（第 1 到 5 问） | `research/prompts/ledger-open-fixes-governance-r3-opus-output.md`（sha256 `7d45c151…`） | 第 1 问两趟同时摘已死的 `.taking` 两趟都占到、`.taking` 没有心跳另一台停住超 300 秒就被当死；第 2 问行键带后缀的 timing 行不归实验号、`#[path]` 引的共用模块不判；第 3 问回退到作废值、登记改名、原地改写旧行三处放行，在树里的合规回退误红；第 4 问 assume-unchanged / skip-worktree 下改过的输出被删；第 5 问下层写坏时上层被拦而说明不点名原因；撤回第二轮攻方腿三条改法 |
| 正推（第 6 到 8 问） | `research/prompts/ledger-open-fixes-governance-r3-sonnet-output.md`（sha256 `3dd43060…`） | 六条都做了判决说的事；三处偏离都偏得对；钉值格成功句与格名表那一行没跟着放宽改；嵌套两格只被通用开关证红；`.gz` timing 产物一支没样本；红句写「最新的」取的是最旧的；hook-swap 的 KILL 格没记数 |

核查命令（主 agent 的草稿目录 `~/.local/share/singlefs/work/open-fixes-governance-r3-verify/`，日志在它的 `main-agent-logs/`）：

- 倒推快照：`python3 scripts/agents/snapshot-tree.py ledger-open-fixes-governance-r3 --out <草稿>/r3-tree --repository <仓根>`，结局 `type=succeeded exit=0`，「清单点名 72 份，倒推出 72 份，sha256 对上 72 份」。
- 引文：`python3 scripts/agents/cite-check.py <两份报告> --root <草稿>/r3-tree --background research/prompts/_ledger-open-fixes-governance-r3-background.md`，末行「核了 80 处引文、56 处裸行号，对上 72 处，没判 64 处」，判错 0 处。
- 攻方四份探针拷进 `<草稿>/probe/` 复跑，结局都 `type=succeeded exit=0`。`probe-evidence.sh` 第 26 行与 `probe-pin.sh` 第 42 行起门禁格的那一句在副本里补了范围声明（项目代码闸要求），别的一字没动：
  - `x1-taking.log`：「arm=double-dead-taking-scratch … believers=2」「arm=double-dead-taking-repository … believers=2」，对照「arm=double-dead-taking-sequential … believers=1」；「arm=remover-stall-elsewhere … believers=2」，对照「arm=remover-stall-same-machine … believers=1」；「arm=stale-threshold SCRATCH_LOCK_STALE_SECONDS=300 elsewhere_330s_occupies=False elsewhere_290s_occupies=True」。
  - `x2-evidence.log`：「arm=suffix-timing-row exit=0」，对照「arm=suffix-timing-control exit=1」；「arm=mutation-table-only exit=1」「arm=archived-timing-product exit=1」。
  - `x3-pin.log`：「arm=older-line-rewritten exit=0」（对照「arm=control-base-line-rewritten exit=1」）、「arm=archived-revert-to-superseded exit=0」、「arm=intree-repin-superseded exit=0」、「arm=renamed-registry-pin-kept exit=0」（上一行「deletion_commit_found=1」）、「arm=intree-revert-with-new-line exit=1」、「arm=renamed-registry-live-revised exit=1」。
  - `x4-delete.log`：e07、e56 两份「-assume-unchanged-modified … output_kept=false … reached_build=true」「-skip-worktree-modified … output_kept=false … reached_build=true」「-icase-environment … output_kept=false … reached_build=true」，对照「-control-untracked … output_kept=true refused_by_delete_gate=true」；e53 十臂都 `refused_by_delete_gate=false reached_build=false`、e159 五臂打「输出文件之后写子命令（probe-device 或 cell）」：这两份的臂停在删旧闸之前的参数核对上，作废（探针给的参数不合这两份入口）。
- 主 agent 现查：
  - `.claude/gate.d/doc-experiments.sh:1136` 取 `min(timing_present, …)`，`:1174` 的红句写「最新的 E{number} 产物」。
  - `.claude/gate.d/doc-experiments.sh:4776` 的成功句写「装置没变而钉值改了的 … 份，登记第十二节都新加了写着新钉值的行」，`:32` 格名表那一行写「要在这次改动里新加一行写着新钉值的修订」，都没跟着只改钉值那一支的放宽改。
  - 正推腿给的线索 E184 复跑表 timing 行点名的 `e184-regular-layer-cost-counts-segment-one-2026-10-24.out` 不在盘：`git log --oneline --all` 那份产物两条，最近一条是 `a24379563`（归档提交），是归档出工作区，不是缺陷。
  - 正文写的「过期 360 秒」不对：探针读的常数 `SCRATCH_LOCK_STALE_SECONDS=300`，360 是自证样本拨的年龄。更正写在这里，正文冻结不改。

## 二、判决

坐实，这一轮改（最后一轮，改法都标零轮）：

1. **换上工具 `.taking` 两趟各占一次**（攻方第 1 问，`x1` 四臂 `believers=2`）：两趟同时摘同一份已死的 `.taking`，先读后删不是原子的，两趟都当自己占到，草稿目录与按仓的两份记录都这样；`.taking` 没有心跳，另一台上摘的那一趟停住超过 300 秒就被当成死了，它恢复后删掉别人刚放的记录。文件头「留下的缝」把前一种的后果写成「只是回到没有这一道占用的样子」，按仓的那一份上不成立；后一种没列。
2. **evidence-in-repo 行键带后缀的 timing 行不归实验号**（攻方第 2 问，`x2` 第一臂 `exit=0`）：复跑表行键写成 `E926R2` 这种（与现行 `E16R5`、`E155R2` 同一写法）时，这一格认不到那个实验有 timing 行，第二轮打中的「放一份不相干的文件就绿」原样回来。另两处并进这一条：经 `#[path]` 引进来、住在 `tests/` 下的共用模块这一格不判，文件头「管不到」没写；红句写「最新的」取的是最旧的那份（正推腿，主 agent 现查 `:1136`、`:1174`）。
3. **钉值格四处判错**（攻方第 3 问，`x3`）：第十二节里不带基准钉值的旧行原地改写，三条判据都过（`older-line-rewritten exit=0`）；登记已归档或在树里时，钉回登记里已被后续修订作废的旧值放行（`archived-revert-to-superseded`、`intree-repin-superseded` 都 `exit=0`）；登记被 `git mv` 改名而起跑脚本没跟着改，拿改名之前冻住的那一版判（`renamed-registry-pin-kept exit=0`）；登记在树里时合规的回退（新加一行写回旧值）判红、没有出路（`intree-revert-with-new-line exit=1`）。另外成功句与格名表那一行没跟着只改钉值那一支的放宽改（正推腿，主 agent 现查 `:32`、`:4776`）。
4. **删旧闸被索引标记绕过**（攻方第 4 问，`x4` e07、e56 各三臂 `output_kept=false reached_build=true`）：输出文件带 assume-unchanged 或 skip-worktree 标记、改过没提交时，两条 git 都报「没差」，文件被删；调用环境带 `GIT_ICASE_PATHSPECS=1` 时同样被删。
5. **完工清理闸下层声明写坏时，上层被拦而说明不点名原因**（攻方第 5 问）：下层 `.deliverables` 写坏、被删或换成符号链接，副本照列、上层被拦，拦的说明里没有下层那一份的原因。
6. **自证与样本的四处空白**（正推腿第 8 问）：嵌套两格 `nested-undeclared-copy-blocks`、`nested-lower-handover-main-listed` 只被通用开关证红，没有一条开关只弄坏「认下层声明」那一处；timing 产物只存着 `.gz` 的那一支没有样本；hook-swap 自证 KILL 那一格没算进成功句「查了 N 种」。

认下，这一轮不改：

- 下层声明不论是哪个会话写的，上层都照认（攻方第 5 问）：主 agent 收工时不读下层声明、交接副本照列，后果有界。
- `.taking` 被 TERM 在放 `.taking` 与放草稿目录记录之间时，留下的不超过被 KILL 留下的（攻方第 1 问）。
- evidence-in-repo「每一条 timing 行都要」的出路只有全部重跑一条，14 个有 timing 行的实验里 13 个的 timing 产物今天已归档（攻方第 2 问，`x2` 末两臂 `exit=1`）：这是收严的代价，正推腿判收严成立；装置不再改时不触发。

不采：

- E184 复跑表点名的产物不在盘（正推腿线索）：主 agent 现查是归档提交 `a24379563` 拿出工作区的，与这一格判的「改了装置没有新产物」无关。

攻方腿撤回第二轮攻方腿三条改法（删旧闸只加 `:(literal)`、钉值格「从 git 历史取被删前那一版判」、evidence-in-repo「同一个实验有 timing 行时」那句没写行键怎么归），三条都已被这一轮坐实的第 2、3、4 条罩住。

## 三、改法

| 改法 | 修哪一条 | 谁提的 | 标 |
|---|---|---|---|
| 摘已死的 `.taking` 先排他建一份唯一名的摘取记录（只有一趟建得成），建成的那一趟才删 `.taking`；摘的那一趟持有期间给 `.taking` 刷修改时间（同记录的心跳）；文件头「留下的缝」照改后的判法重写；四臂各做成一格自证与一条弄坏开关 | 第 1 条 | 攻方腿与主 agent | 零轮；缺口量过（`x1` 四臂 `believers=2`） |
| 复跑表行键按「`E<号>` 后接可选的 `R<号>`」归到实验号；文件头「管不到」补 `#[path]` 引进来的共用模块；红句改成「最旧的」；补一份后缀行键的红样本与一份 `.gz` timing 产物的样本 | 第 2 条、第 6 条第二样 | 攻方腿与正推腿 | 零轮；缺口量过（`x2` 第一臂 `exit=0`） |
| 第十二节基准那一版的每一行（不只带基准钉值的）都要原样在；钉值只认第十二节里最后一次出现的那个值（钉回作废值判红、新加一行写回旧值放行）；起跑脚本点名的登记不在树里而 git 历史里它是被改名走的，判红、出路写改脚本里的登记路径；成功句与格名表那一行照只改钉值那一支改；五种写法各一份样本 | 第 3 条 | 攻方腿与正推腿 | 零轮；缺口量过（`x3` 五臂） |
| 删旧闸改成按内容比：工作区那一份的 `git hash-object` 等于 `HEAD` 里那一份的 blob 才算「在 git 里、没改过」；两条 git 起之前清掉 `GIT_ICASE_PATHSPECS`；四份入口同改，各补一格自证与一条开关 | 第 4 条 | 攻方腿 | 零轮；缺口量过（`x4` e07、e56 各三臂） |
| 上层被拦时，说明里点名下层 `.deliverables` 哪一行写坏、被删或换成符号链接；补一条只弄坏「认下层声明」的开关，点名嵌套两格；hook-swap KILL 格记进「查了 N 种」 | 第 5 条、第 6 条第一、三样 | 攻方腿与正推腿 | 零轮 |

这一轮是最后一轮，以上改法改完不再开第四轮；改动照治理文件的换上流程做（工具员在草稿里改、主 agent 经 `scripts/gate/hook-swap.py` 换上、自证与开关复跑逐格核）。

## 交用户的

没有要用户定的岔路。告知：这一题三轮打完，第三轮坐实的六条修补没被再攻过，标零轮。

## 这一轮被判过的文件

- research/prompts/_ledger-open-fixes-governance-r3-body.md：两条腿照它的八问作答；「过期 360 秒」一句写错，判决更正为 300 秒。
- research/prompts/ledger-open-fixes-governance-r2-main-verification.md：正推腿逐条核它「三、改法」六条，代码都做了那一句说的事；三处偏离判得对。
- scripts/gate/hook-swap.py：判了 `.taking` 两趟各占一次与没有心跳两处（第二节第 1 条）、KILL 格没记数（第 6 条）。
- .claude/gate.d/doc-experiments.sh：判了行键带后缀的 timing 行不归实验号、红句与取法对不上（第 2 条），钉值格四处判错与成功句、格名表没跟着改（第 3 条）。
- .claude/hooks/completion-cleanup-guard.sh：判了下层写坏时上层的说明不点名原因（第 5 条）、嵌套两格只被通用开关证红（第 6 条）；下层声明照认、后果有界（认下）。
- scripts/experiment-apparatus/e07-vm-bench-run.sh：判了删旧闸被 assume-unchanged、skip-worktree 与 `GIT_ICASE_PATHSPECS` 绕过（第 4 条）。
- scripts/experiment-apparatus/e53-vm-bench-run.sh：同 e07 那一份的删旧闸（第 4 条），探针臂停在参数核对上作废，按同一段代码判。
- scripts/experiment-apparatus/e159-vm-bench-run.sh：同 e07 那一份的删旧闸（第 4 条），探针臂停在参数核对上作废，按同一段代码判。
- scripts/experiment-apparatus/e56-vm-bench-run.sh：判了删旧闸被索引标记与 `GIT_ICASE_PATHSPECS` 绕过（第 4 条）。
- .claude/gate.d/stage-owners.tsv：判了第 117、130、143 行与换上的判法对得上。
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-crates-bin-green/.gate-cells：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-crates-bin-green/expect：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-crates-bin-green/setup.sh：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-crates-bin-red/.gate-cells：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-crates-bin-red/expect：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-crates-bin-red/setup.sh：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-exact-only-rerun-red/.gate-cells：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-exact-only-rerun-red/expect：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-exact-only-rerun-red/setup.sh：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-timing-rows-green/.gate-cells：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-timing-rows-green/expect：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-timing-rows-green/setup.sh：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-timing-rows-red/.gate-cells：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-timing-rows-red/expect：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-timing-rows-red/setup.sh：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-unrelated-file-red/.gate-cells：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-unrelated-file-red/expect：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-unrelated-file-red/setup.sh：判了它照 timing 行逐份比、认 crates 下装置这一支站得住，没罩到行键带后缀的 timing 行（第二节第 2 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-added-line-without-pin-red/.gate-cells：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-added-line-without-pin-red/expect：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-added-line-without-pin-red/setup.sh：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-archived-history-green/.gate-cells：判了它钉住的是只改钉值与登记已归档两支的现行判法，没罩到回退到作废值与登记改名（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-archived-history-green/expect：判了它钉住的是只改钉值与登记已归档两支的现行判法，没罩到回退到作废值与登记改名（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-archived-history-green/setup.sh：判了它钉住的是只改钉值与登记已归档两支的现行判法，没罩到回退到作废值与登记改名（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-archived-history-red/.gate-cells：判了它钉住的是只改钉值与登记已归档两支的现行判法，没罩到回退到作废值与登记改名（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-archived-history-red/expect：判了它钉住的是只改钉值与登记已归档两支的现行判法，没罩到回退到作废值与登记改名（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-archived-history-red/setup.sh：判了它钉住的是只改钉值与登记已归档两支的现行判法，没罩到回退到作废值与登记改名（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-duplicated-line-red/.gate-cells：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-duplicated-line-red/expect：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-duplicated-line-red/setup.sh：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-inplace-rewrite-red/.gate-cells：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-inplace-rewrite-red/expect：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-inplace-rewrite-red/setup.sh：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-internal-whitespace-red/.gate-cells：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-internal-whitespace-red/expect：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-internal-whitespace-red/setup.sh：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-pin-in-other-section-red/.gate-cells：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-pin-in-other-section-red/expect：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-pin-in-other-section-red/setup.sh：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-pin-prefix-green/.gate-cells：判了它钉住的是只改钉值与登记已归档两支的现行判法，没罩到回退到作废值与登记改名（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-pin-prefix-green/expect：判了它钉住的是只改钉值与登记已归档两支的现行判法，没罩到回退到作废值与登记改名（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-pin-prefix-green/setup.sh：判了它钉住的是只改钉值与登记已归档两支的现行判法，没罩到回退到作废值与登记改名（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-pin-trailing-comment-red/.gate-cells：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-pin-trailing-comment-red/expect：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-pin-trailing-comment-red/setup.sh：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-registry-mentioned-after-another-red/.gate-cells：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-registry-mentioned-after-another-red/expect：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-registry-mentioned-after-another-red/setup.sh：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-registry-unlabeled-red/.gate-cells：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-registry-unlabeled-red/expect：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-registry-unlabeled-red/setup.sh：判了它钉住的写法照判红，没罩到原地改写不带基准钉值的旧行（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-repin-only-green/.gate-cells：判了它钉住的是只改钉值与登记已归档两支的现行判法，没罩到回退到作废值与登记改名（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-repin-only-green/expect：判了它钉住的是只改钉值与登记已归档两支的现行判法，没罩到回退到作废值与登记改名（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-repin-only-green/setup.sh：判了它钉住的是只改钉值与登记已归档两支的现行判法，没罩到回退到作废值与登记改名（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-repin-only-red/.gate-cells：判了它钉住的是只改钉值与登记已归档两支的现行判法，没罩到回退到作废值与登记改名（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-repin-only-red/expect：判了它钉住的是只改钉值与登记已归档两支的现行判法，没罩到回退到作废值与登记改名（第二节第 3 条）
- .claude/gate.d/fixtures/doc-experiments.sh/pinned-apparatus-revision-repin-only-red/setup.sh：判了它钉住的是只改钉值与登记已归档两支的现行判法，没罩到回退到作废值与登记改名（第二节第 3 条）

## 回看决策

不涉及决策：这一轮判的是换上工具、两道门禁格、完工清理闸与四份虚机入口，都是协作工具，不推翻也不确立任何决策。
