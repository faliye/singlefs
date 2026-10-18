# C439（gate-ok 前移让同轮早先写的行不再被判）：第二轮正文（2026-10-15）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 -->

## 一、这一轮要判什么

第一轮判决 `research/prompts/c439-r1-main-verification.md`：照今天的外壳，甲、乙、丙、丁、戊与正推加的庚都出局；站着的只有攻方提的两个候选，都被攻过零轮。这一轮只攻这两个，换第一轮没看过的面：

- 庚二：基准取「最后一次整轮绿的那棵树」（`refs/sop/staged-green` 的树）对这次的暂存树做树对树的 diff；里层判的就是外壳 `git write-tree` 的那棵树（不再自己从暂存区重套）；调用方给了 `GATE_BASE`、或这一趟有阶段强制跑过（`--force`）时不前移 staged-green。最后一条守卫是主 agent 在第一轮判决里补的。
- wu_cf：quoted-result-lines 格不按窗口判，改成全量判，加一张存量豁免表；豁免按（路径、行文）计数，只缩不涨的比对基准是引入时的固定基线。

出口：两个候选各判站得住 / 站不住（条款照第一轮正文「五、跑前条款」，原文在第一轮背景材料 `research/prompts/_c439-r1-background.md`）；站得住的给代价数；两个都站得住就写岔路单交用户；都站不住就照第三轮的分工另开。这一轮不改任何脚本。

## 二、实现今天的样子（主 agent 的观测，2026-10-15 现查）

`crates/` 里没有这件事的代码：`grep -rn 'gate-ok\|diff_base\|staged-green' crates/` 0 命中。脚本沿用第一轮的冻结副本 `~/.local/share/singlefs/work/c439-r1/frozen/`（10 份，sha256 在 `research/prompts/c439-r1-snapshot/scripts-sha256.txt`），引行号以它为准。

- 上游 `.claude/singlefs-ai-sop/scripts/gate.sh` 的 `--staged`：第 55 行把 `git diff --cached --binary` 写进补丁，第 58 行 `git worktree add --detach <临时树> HEAD`，第 87 行把补丁 `apply --index` 套上去；第 91–93 行规范副本不在临时树里时从源仓拷进去；第 96–99 行源仓有 cargo 前缀登记而临时树里没有就拒绝；第 102 行先认调用方的 `GATE_BASE`，没有才在源仓上算 `diff_base`。外壳与里层各读一次暂存区，两次之间 HEAD 也可能被别的会话的提交挪动。
- 本仓外壳 `scripts/gate/gate-staged.sh`：第 28 行 `git write-tree`，第 31–33 行把它放进 `SINGLEFS_STAGED_TREE` 再起 `gate.sh --staged`，第 36–46 行退出码 0 且收尾的 `write-tree` 与开跑时同一棵才前移 staged-green（一个包着那棵树、没有父提交的提交）。
- `scripts/gate/stage-must-run.sh`（62 行）：拿 staged-green 那棵树与这一次的暂存树比每道重阶段读的路径，判能不能复用。
- 用 `gate_diff_base gate` 的格不只 quoted-result-lines：doc-experiments 六格（第 58 行 `CHANGE_RANGE_CELLS`）、doc-process-records 五格、doc-text、doc-decisions、doc-registries 的 invariant-anchors 格、checker-independence-and-sync、code-tooling、`scripts/kb/archive-past-rounds.py` 的 `base_of`。
- 共享规则 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」一节写着「正文引产物的检查按「这次改动」判，不按全仓判」，项目里不许就地改（只能在上游改）。
- 第一轮量到的存量（攻方 `research/prompts/c439-r1-opus-output.md`「H4：代价」，复跑逐字相同）：staged-green 停在 2026-09-22，庚二此刻的窗口 1326 行，照窗口判法 355 行红、其中 101 行在任何产物里都找不到；全量 1794 行、455 行红。那 101 行登记为 C837（实验页里抄短的产物行落在窗口外）；外壳前移的树不一定是被判的树登记为 C836（staged-green 挂的树不一定是被判的树）。

## 三、格

| 格 | 判什么 |
|---|---|
| K1 | 庚二的落法：「里层判的就是外壳那棵树」怎么落（上游 `gate.sh --staged` 收一棵树、或外壳自己建临时树再起里层），落法本身会不会再开口子——外壳 write-tree 与里层建出的临时树逐字节是不是同一棵（规范副本的拷贝、cargo 前缀、HEAD 在两步之间被挪动、子模块与符号链接、带过滤器的文件）；守卫（人给 `GATE_BASE`、`--force`）放在外壳还是里层，各漏什么 |
| K2 | 庚二与 `stage-must-run.sh` 的复用：同一条 ref 同时当改动范围的基准与重阶段复用的依据，庚二下复用会不会认错树；只前移「判过的那棵」之后，复用一侧有没有反向的漏（该复用的不复用只是代价，该重跑的被复用才算漏） |
| K3 | 庚二多会话：两个以上会话同时跑外壳、先后前移 staged-green（后写的覆盖先写的），一个会话的绿树里没有另一个会话暂存的行；之后谁的窗口包着谁的行，有没有一段交错让一行永久落在窗口外；人在暂存区留着别的会话的一行红、谁都挪不动 staged-green 时，旁路（`GATE_BASE`、`--force`、绕过门禁提交）走哪一条、守卫拦不拦得住 |
| K4 | 庚二的射程：只给 quoted-result-lines（或只给「新增行」类的格）改基准，还是 `gate_diff_base gate` 整个改；不带 `--staged` 那条路改不改；各自要动 C528（两条门禁路判的改动窗口不同） 现状说明的哪一句，碰不碰用户 2026-09-24 的定案（共用脚本不认 `GATE_DIFF_BASE`）；漏改的那条路留下什么形 |
| K5 | wu_cf 的两处没量的：产物重跑一轮（换登记、重跑写出新的一版）之后，全量判下正文早先抄的行有多少变红，表不许涨时只能当场回改正文，量出要回改多少行（照本仓 `replay.sh` 登记的产物与最近几轮真实的重跑取样）；与共享规则「按「这次改动」判」冲突，wu_cf 要走就得先改上游规则，判这一改的射程（别的引产物的检查跟不跟） |
| K6 | 复核第一轮判决：第一轮出局的丙、戊（集合键、比 HEAD）、门禁写的乙（退 merge-base）、按树挂的丁，打中它们的形是不是同样打中庚二或 wu_cf；第一轮判决第二节 H3 那一格两种代价口径（别的会话先把那行退出暂存再跑绿 / 那行留在暂存里被判过）在庚二下各量一遍 |

## 四、分工

| 腿 | 立场 | 格 |
|---|---|---|
| 云端辩方（Sonnet） | 复核第一轮判决：出局的几个候选是不是被判错了（打中它们的形能不能修、修了是不是就是庚二或 wu_cf）；第一轮判决的 H3 那一格两种口径各量；庚二与 wu_cf 站得住的那一面，替它们答 K1、K4 | K1、K4、K6 |
| 云端攻方（Opus） | 对庚二与 wu_cf 造反例历史与量数，换第一轮没看过的面，不重复第一轮已攻过的形 | K1–K5 |

## 五、跑前条款

照第一轮正文「五、跑前条款」那六行（原文在 `research/prompts/_c439-r1-background.md`），另加：

| 条款 | 什么观测会让它触发 |
|---|---|
| 庚二的某种落法造出一段历史，staged-green 前移到的树里有被判的那棵没有的行 ⇒ 那种落法出局，另一种落法照判 | 一份可复跑的临时仓脚本，落法照写，之后那一行一次都没进窗口 |
| 庚二下复用一侧被造出「该重跑的被复用」 ⇒ 记成庚二的一条漏，与改动范围的漏同等出局 | 临时仓里一道重阶段读的路径在被判的树与 staged-green 的树之间不同，复用判据却判「没变」 |
| wu_cf 在一轮真实取样的重跑之后要回改的行数超过存量本身 ⇒ 记代价，交用户时写明 | K5 的量数 |
| 两条腿对第一轮判决出局的某一个候选给出「打中它的形同样打中留下的那个」 ⇒ 留下的那个同样出局 | K6 的复核，带可复跑的历史 |

⚠️ 打中之后先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 六、交付要求

- 报告先写进 `research/prompts/c439-r2-<腿名>-output.md`（辩方 `-sonnet-output.md`、攻方 `-opus-output.md`），每一次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建，开工后 20 分钟内落第一段，回复只写短句。
- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数；引脚本写冻结副本里的路径与行号。
- 草稿放腿自己的目录 `~/.local/share/singlefs/work/c439-r2-<腿名>/`，临时 git 仓建在那里。不在主仓里写任何 ref、不改主仓的暂存区与工作区，不在冻结副本里改。第一轮两腿的模型目录（`research/prompts/c439-r1-sonnet-model/`、`research/prompts/c439-r1-opus-model/`）可以拷进自己目录改用。造历史的脚本随报告拷进 `research/prompts/c439-r2-<腿名>-model/`（含一份 `tools/rerun.sh`，在一个新的临时目录里从头造仓、打出结果，与 `out/` 逐字比）。
- 文件与报告里不写钟点、时区词与带钟点的时间戳，日期按东京时间写；造历史要固定提交时刻的，那一行写 `# clock-times:allow <理由>`。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 `gate.sh`、`gate-staged.sh`，不跑 cargo；在临时仓里 `source` 冻结副本的 `scripts/gate/changed-paths.sh`、调它的函数可以；要造真的 `gate.sh --staged` 落法的，在临时仓里拷冻结副本的脚本改，不在主仓起。
- 新写的文字不用「层 0」「第一条流 / 第二条流」这几种旧名（`.claude/kb/term-renames.md`）。
