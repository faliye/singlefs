# C439（gate-ok 前移让同轮早先写的行不再被判）：第一轮正文（2026-10-14）

<!-- doc-lint:not-numbers H1 H2 H3 H4 H5 -->

## 一、这一轮要判什么

`.claude/kb/checks-owed.md` C439（gate-ok 前移让同轮早先写的行不再被判）：门禁里按「这次改动新增或改写的行」判的格（doc-experiments 的 quoted-result-lines 格是来处，同一个基准还有别的格在用），基准怎么取，才不让一行写下之后从没被判过的错引文永久落在窗口外；几个会话共用一个仓（一个 HEAD、一个暂存区、一套 `refs/sop/`）时，这个起点由谁定。

出口：判决给出一种基准取法，连同它在 H1 每种历史上的判定、多会话下的行为、代价、会红的样本形状；收敛就写回 C439 的改法一列；两种以上站得住就写岔路单交用户，每条带代价数；动用户 2026-09-24 定案（共用脚本不认 `GATE_DIFF_BASE`）的单列、交用户。这一轮不改任何脚本。

## 二、实现今天的样子（主 agent 的观测，2026-10-14 现查）

`crates/` 里没有这件事的代码：`grep -rn 'gate-ok\|diff_base\|staged-green' crates/` 0 命中；方案按 `scripts/`、`.claude/` 今天的实现来谈。被引的脚本冻结在 `~/.local/share/singlefs/work/c439-r1/frozen/`（保留仓内路径），10 份的 sha256 在 `~/.local/share/singlefs/work/c439-r1/frozen-sha256.txt`；引行号以冻结副本为准。

- 项目共用库 `scripts/gate/changed-paths.sh` 的 `gate_diff_base`（第 67 行起）：`gate` 取法是 `GATE_BASE` 是提交就用它；否则 `@{upstream}` 的 merge-base；都没有就 `HEAD`。`head` 取法是 `HEAD`。文件头第 9–14 行写明不认上游导出的 `GATE_DIFF_BASE`（用户 2026-09-24 定）与两条路窗口不同（C528（两条门禁路判的改动窗口不同））。
- 用 `gate` 取法的：doc-experiments 六格（第 58 行 `CHANGE_RANGE_CELLS`，quoted-result-lines 在内）、doc-process-records 五格、doc-text、doc-decisions、doc-registries 的 invariant-anchors 格、checker-independence-and-sync、code-tooling，以及 `scripts/kb/archive-past-rounds.py` 的 `base_of`；用 `head` 取法的只有 doc-process-records 的 batch-scope 格。
- 上游 `.claude/singlefs-ai-sop/scripts/lib.sh` 的 `diff_base`（第 144 行起）：设了 `GATE_BASE` 用它；在默认分支上 merge-base 等于 HEAD 时，依次取与上游的 merge-base（不等于 HEAD 时）、`refs/sop/gate-ok`（不等于 HEAD 且是 HEAD 的祖先时）、`HEAD~1`。
- 上游 `.claude/singlefs-ai-sop/scripts/gate.sh`：`--staged` 时第 102 行在源仓上算 `diff_base`，第 115 行设成里层的 `GATE_BASE`；第 119–123 行 `--staged` 这一轮不前移 gate-ok；第 769–782 行不带 `--staged`、`GATE_BASE` 为空、整轮全绿、没有强制跑的阶段时 `update-ref refs/sop/gate-ok` 到开跑时的 HEAD。
- 本仓外壳 `scripts/gate/gate-staged.sh` 第 40–50 行：`gate.sh --staged` 整轮全绿、跑的过程中暂存区没变，就把 `refs/sop/staged-green` 前移到一个包着这次暂存树的提交（没有父提交）；`scripts/gate/stage-must-run.sh` 第 16、33 行拿它那棵树判六道重阶段能不能复用。
- 提交前跑门禁的流程：`.claude/skills/gate/staged-gate.md` 第 16 行（只用 `gate-staged.sh`），第 22 行列门禁自己的 git 写（gate-ok、staged-green）。`.git/hooks/pre-commit` 只是提交时间窗闸（本地、不入库），不跑门禁。
- 现场：本仓没有 `@{upstream}`；`refs/sop/gate-ok` 不存在；`refs/sop/staged-green` 指 `e2ae1837`，提交日期 2026-09-22，它的树 `9b1d9d06` 与 HEAD 往回 60 个提交的树都对不上。此刻上游 `diff_base` 交出 `f0b94fef`（HEAD~1），项目 `gate_diff_base gate` 交出 `f111c3f3`（HEAD）。
- 提交历史：`e2ae1837` 的提交时刻之后 HEAD 上 131 个提交（`git rev-list --count --since=<那个时刻> HEAD`），提交说明里带 `no-verify` 的 70 个，其中 27 个写着「提交时绕过门禁（--no-verify），没跑 gate-staged 与 checker 档」；2026-10-11 有 13 个提交在 13 分钟之内相继落下（一次暂存、分类型提交）。
- C439（gate-ok 前移让同轮早先写的行不再被判） 的来处：正推腿报告 `archive-rename-r1-sonnet-output.md` 已归档，取 `git show ad787cc2^:research/prompts/archive-rename-r1-sonnet-output.md`；同族欠账 C528（两条门禁路判的改动窗口不同）、C454（--staged 跑绿不让基准前移，而且不说），用户 2026-09-24 的定案在 `research/prompts/gate-fix-forks-r2-main-verification.md`「T11　共用脚本认不认 `GATE_DIFF_BASE`」一节。这一轮只判 C439（gate-ok 前移让同轮早先写的行不再被判） 的基准取法；判出的取法若同时改变 C528（两条门禁路判的改动窗口不同） 的现状说明，写明改哪句。

## 三、格

| 格 | 判什么 |
|---|---|
| H1 | 漏判的形：今天的取法下，哪几种历史会让一行新增的错引文永远不在窗口里——C439（gate-ok 前移让同轮早先写的行不再被判） 原形（同一轮里先写、整轮全绿让 gate-ok 前移、再跑）；绕过门禁的提交；一次暂存分几次提交；另一个会话在两次门禁之间提交；全推出去之后。逐种给出 git 历史的形状、今天带与不带 `--staged` 两条路各算出的基准、那一行在不在窗口里 |
| H2 | 候选，至少五个：甲，基准取上一次提交（带不带 `--staged` 都取 HEAD）；乙，开轮或前移时记下这一轮的起点（谁写：开轮的会话、门禁、还是用户）；丙，基准取最后一次整轮绿的那棵树（`refs/sop/staged-green` 的树与这次的暂存树做树对树的 diff，不要求它是 HEAD 的祖先）；丁，门禁绿的标记挂在提交上（git notes 或别的），基准取最近一个带标记的祖先，没带标记的提交一律在窗口里；戊，这几格不按窗口判，改成全量判，加一张只缩不涨的存量豁免表。可以加别的。逐个给出在 H1 每种历史上的判定 |
| H3 | 多会话：一个 HEAD、一个暂存区、一套 `refs/sop/` 时，每个候选的起点由谁定、会不会被别的会话的写挪动；两个会话交错跑门禁与提交时，会不会漏判，会不会把别的会话的行算进这一次的窗口；提交时整个暂存区一起提交、不分是谁改的，这一条对每个候选意味着什么 |
| H4 | 代价：每个候选要改哪些文件（上游的、本仓的），动不动用户 2026-09-24 的定案与 C528（两条门禁路判的改动窗口不同） 的现状说明；窗口变宽之后存量红有多少（数得出的就数，例如丙在此刻让 quoted-result-lines 多判多少行：对冻结副本的拷贝调 `gate_added_lines` 量）；与 `stage-must-run.sh` 复用判据是不是同一个起点 |
| H5 | 会红的样本与判别力自证的形状：照选定取法，临时 git 仓里造「整轮全绿之后同一轮里早先写下的错引文」与「绕过门禁提交的错引文」两种历史，选定取法下必须进窗口；把取法换回今天的，同样两份样本必须漏掉 |

## 四、分工

| 腿 | 立场 | 格 |
|---|---|---|
| 云端正推（Sonnet） | 从脚本与 git 语义推每个候选在 H1 每种历史上的基准与判定、H3 的多会话行为、H4 的代价，给出哪一个站得住 | H1–H5 |
| 云端攻方（Opus） | 对每个候选造反例历史：在自己目录下的临时 git 仓里造，找让候选永久漏判一行、把起点挪到别的会话的未判改动之后、或在交错下判错窗口的那一段；H1 尽量在临时仓里复现今天的漏判，造不出写试过的形 | H1–H4 |

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| H1 两条腿都造不出今天的漏判 ⇒ C439（gate-ok 前移让同轮早先写的行不再被判） 降为只在不带 `--staged` 的整轮全绿时成立，处置照判但写明射程 | 两条腿各写试过的形，没有一段历史让错引文落在窗口外 |
| 某候选被造出一段历史，一行新增的错引文永久不在窗口里 ⇒ 那个候选出局 | 一份可复跑的临时仓脚本：照流程跑门禁、提交、再跑门禁，那一行一次都没进 `gate_added_lines` 的输出 |
| 某候选在两个会话交错时把起点挪到另一个会话还没判过的改动之后 ⇒ 那个候选出局 | 同一份临时仓里两组操作交错，选定取法下另一组写的错引文漏掉 |
| 某候选把别的会话已经判绿过的行算进这一次的窗口 ⇒ 记代价，不单凭这一条出局 | 交错样本里那几行在窗口里，且它们在之前某一次整轮绿里判过 |
| 留下的候选要动用户 2026-09-24 的定案 ⇒ 交用户，不由判决定 | 站得住的候选里有要共用脚本认 `GATE_DIFF_BASE` 或改两条路窗口关系的 |
| 两个以上候选站得住 ⇒ 写岔路单交用户，每个候选带代价数（窗口宽度、存量红行数） | 两条腿对剩下的候选都给不出出局的反例 |

⚠️ 打中之后先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 六、交付要求

- 报告先写进 `research/prompts/c439-r1-<腿名>-output.md`（正推 `-sonnet-output.md`、攻方 `-opus-output.md`），每一次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建，回复只写短句。
- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数；引脚本写冻结副本里的路径与行号。
- 草稿放腿自己的目录 `~/.local/share/singlefs/work/c439-r1-<腿名>/`；临时 git 仓建在那里。不在主仓里写任何 ref、不改主仓的暂存区与工作区，不在冻结副本里改。造历史的脚本随报告拷进 `research/prompts/c439-r1-<腿名>-model/`（含一份 `rerun.sh`，在一个新的临时目录里从头造仓、打出各候选的基准与 `gate_added_lines` 输出）。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 `gate.sh`、`gate-staged.sh`，不跑 cargo；在临时仓里 `source` 冻结副本的 `scripts/gate/changed-paths.sh`、调它的函数可以。
- 新写的文字不用「层 0」「第一条流 / 第二条流」这几种旧名（`.claude/kb/term-renames.md`）。
