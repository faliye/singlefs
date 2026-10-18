# c439-r1 判决：C439（gate-ok 前移让同轮早先写的行不再被判）第一轮（2026-10-15）

被判的对象：`.claude/kb/checks-owed.md` C439（gate-ok 前移让同轮早先写的行不再被判）那一行的改法——门禁里按「这次改动新增或改写的行」判的格，基准怎么取、多会话下起点由谁定。涉及的脚本冻结在 `~/.local/share/singlefs/work/c439-r1/frozen/`（`scripts/gate/changed-paths.sh`、`scripts/gate/gate-staged.sh`、`scripts/gate/stage-must-run.sh`、`.claude/singlefs-ai-sop/scripts/lib.sh`、`.claude/singlefs-ai-sop/scripts/gate.sh` 等 10 份）。`crates/` 不涉及。

<!-- doc-lint:not-numbers H1 H2 H3 H4 H5 -->

## 一、证据核对（主 agent）

| 证据 | 核法 | 结论 |
|---|---|---|
| 冻结副本 | `sha256sum -c ~/.local/share/singlefs/work/c439-r1/frozen-sha256.txt`（与 `research/prompts/c439-r1-snapshot/scripts-sha256.txt` 同一份） | 10 份全对上 |
| kb 快照 | `sha256sum -c research/prompts/c439-r1-snapshot/kb-sha256.txt` | `.claude/kb/checks-owed.md`、`.claude/kb/term-renames.md` 两份对不上（别的会话的提交与改动）；腿引的 C439（gate-ok 前移让同轮早先写的行不再被判）、C528（两条门禁路判的改动窗口不同）、C454（--staged 跑绿不让基准前移，而且不说） 三行逐字仍在附录里（`grep -qF` 三行都中），判决用到的条文没变 |
| 正推 `research/prompts/c439-r1-sonnet-output.md`（交回时 sha256 0f57df71…）、攻方 `research/prompts/c439-r1-opus-output.md`（sha256 4360f51b…） | `cite-check.py <两份> --root <仓根> --background research/prompts/_c439-r1-background.md` | 「✓ 核了 44 处引文、19 处裸行号，对上 44 处，没判 19 处」；19 处裸行号里撑结论的逐处人工核：`scripts/gate/gate-staged.sh` 第 28、33、38–46 行，`.claude/singlefs-ai-sop/scripts/gate.sh` 第 102、769、774、781 行，`.claude/singlefs-ai-sop/scripts/lib.sh` 第 175–181 行，都与腿的读法一致 |
| 两腿产物里的钟点 | 别的会话转达 doc-text 的 clock-times 格判红；照用户 2026-10-15「时刻一定不能出现」 | 主 agent 改了三处，改前 sha256 存在 `~/.local/share/singlefs/work/c439-r1/sonnet-before-clock-fix.sha256` 与 `opus-before-clock-fix.sha256`：正推报告第 71、73、75 行 `git log --since=` 的时间戳字面换成同一时刻的取法 `"$(git log -1 --format=%cI e2ae1837)"`（在 `f111c3f3` 上数出 131）；正推 `histories.sh` 第 144 行、攻方 `c439-model.sh` 第 25 行造历史用的固定提交时刻加 `# clock-times:allow`。数与判定一个字没动 |
| 复跑 | 两腿模型目录整份拷进会话草稿目录，经 `run-with-memory-cap.sh 16G` 跑各自的 `tools/rerun.sh`，冻结副本照原路径给 | 正推：`h1h3-summary.txt`、`h1h3-details.txt`、`h5.txt`、`h5-broken.txt` 逐字相同；`h4-measure.txt` 随 HEAD 变（HEAD 已从 `f111c3f3` 移到 `c1e9b324`），腿自己写明不逐行比。攻方：`examples.txt`、`h1-today.txt`、`grid-L2.txt`、`real.txt` 四份 `same=yes`。两条退出码 0。长度 3、4 的扫描不在 `rerun.sh` 里，没复跑（核不动的一条：挂钟以小时计） |
| 背景材料的错 | 主 agent 现查 `git config --get branch.master.remote` 与 `git rev-parse -q --verify '@{upstream}'` | 背景写「本仓没有 `@{upstream}`」不成立：上游是 `origin/master`，取样那一刻与 HEAD 同一个提交；两条路的基准（不带 `--staged` 取 HEAD、带 `--staged` 取 HEAD~1）照旧对，走的是「已全推」那一支。两腿都报了这一条 |
| 攻方的本仓实例 | `.claude/kb/experiments/164-崩溃放量新设计的小范围功能测试.md` 第 35 行对 `git show '31128273^:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out'` 里 `name=w3_scan` 那一行 | 属实：页里那一行止于 `test_file_renames=169`，产物原行后面还有 `head=…`、`crates_raw_digest=…` 两段。攻方报的 101 行（E164（崩溃放量新设计的小范围功能测试） 页 100、E168（删除借保留量的环窗口） 页 1）只核了这一行 |
| 丙的交错与人给基准两形 | 读冻结的 `scripts/gate/gate-staged.sh` 与 `gate.sh` | 坐实：外壳第 28 行 `git write-tree` 记下要前移的树，第 33 行才起 `gate.sh --staged`，里层第 56 行另取一次 `git diff --cached` 套到临时树上（第 87 行），两次读暂存区之间改走再改回，第 38–39 行的比树看不出；里层第 102 行先认调用方的 `GATE_BASE`，外壳第 36–46 行只看退出码与树就前移 `refs/sop/staged-green`，没有 `gate.sh` 第 774 行那一条守卫；`gate.sh` 文件头第 8 行写明强制跑过的阶段不让退出码变非 0，外壳同样照前移 |

## 二、逐格判

| 格 | 正推 | 攻方 | 判 |
|---|---|---|---|
| H1 今天 | 14 段里 9 段永久漏；C439（gate-ok 前移让同轮早先写的行不再被判） 原形复现（`h1a_original` 对 `h1a_nomove`） | 打中：绕过门禁提交一行、之后再有一次提交，两条路永远判不到（长度 4 内 85/85，已推 156/156）；C439（gate-ok 前移让同轮早先写的行不再被判） 原形是其中一种；本仓此刻 101 行在任何产物里都找不到 | **漏判成立，射程比 C439（gate-ok 前移让同轮早先写的行不再被判） 写的宽**：病根是「已提交而没判过的行」只要基准挪过它就永不再判，gate-ok 前移只是挪基准的一种。跑前条款第一行不触发。本仓 2026-09-22 之后有 27 个提交写着绕过门禁 |
| H2 甲（取 HEAD） | 站不住，13 段全漏 | 打中，永久（135/135） | **出局**（条款 2） |
| H2 乙（记起点） | 站不住：另一会话开轮覆盖共用 ref 就漏 | 会话写的两种被条款 3 打中；门禁写 gate-ok、退 HEAD~1 的被 amend 与 revert 打中；退 merge-base 的被「记号挂在 HEAD、HEAD 里有被判的树删掉的行」打穿（114/114） | **出局**（条款 3、条款 2）。C439（gate-ok 前移让同轮早先写的行不再被判） 改法一列给的两个改法（「改成上一次提交」「前移时记下当轮起点」）在打中的格上都照中 |
| H2 丙（staged-green 的树对暂存树） | 站得住：只改带 `--staged` 漏 1 段（只跑不带 `--staged` 的整轮），两条路都改 0 漏；提了要补「带 `--force` 不前移」的守卫；把外壳的两次读暂存区之间的空隙列为没造出样本的残余 | 打中，永久：交错改走又改回暂存区（182/182）、人给 `GATE_BASE` 跑外壳（182/182） | **照今天的外壳出局**（条款 3、条款 2）：打中的两形都由第一节的代码读法坐实，正推自己列的残余就是攻方造出的那一形。病不在「拿树对树当基准」，在 staged-green 挂的树不一定是被判的树；补上之后的形是庚二，见下 |
| H2 丁（标记挂提交） | 站不住（带条件）：改写历史时漏 1 段 | 挂在开跑 HEAD 的与乙同形打中；开 `notes.rewriteRef` 的被 amend 打中；按树挂的继承丙的两形 | **出局** |
| H2 戊（全量判加豁免表） | 站得住，代价高：约 100 行存量豁免，与 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「正文引产物的检查按「这次改动」判」冲突 | 按（路径、行文）集合认的被「同一页再抄一遍」打中；只缩不涨比 HEAD 的被「绕过门禁的提交同时加错行与豁免行」打中 | **照这两种豁免键出局**；还与共享规则「正文引产物的检查按「这次改动」判，不按全仓判」冲突，要走就得先改上游规则 |
| H2 庚二（攻方提，零轮） | 没判 | 丙，再让里层判的就是外壳 write-tree 的那棵，再加「`GATE_BASE` 是人给的就不前移」：19 段历史长度 3（含 G）与长度 4 下，续作里只要有一次正常门禁就判到 | **被攻过零轮**，第二轮的主攻对象。主 agent 补一条：强制跑过（`--force`）也不前移，正推第 4 条 |
| H2 wu_cf（攻方提，零轮） | 没判 | 戊，豁免按（路径、行文）计数、只缩不涨比引入时的固定基线：两形都接住，长度 3 内没漏 | **被攻过零轮**；与共享规则的冲突不随豁免键消失 |
| H2 庚（正推提：给不带 `--staged` 那条路补一档 gate-ok） | 站不住，7 段漏 | 没判 | **出局** |
| H3 多会话 | 丙、丁、戊不漏，今天、甲、乙、庚各有漏法；丙会把别的会话判绿过的行算进窗口 | 别的会话判绿过的行再进窗口：今天 155/310，门禁写的乙 310/310，丙、按树挂的丁、庚二 0/310 | 两腿的「丙会 / 不会把别人判过的行算进窗口」对不上：正推的 `m2_lastwriter` 是别的会话先把那行退出暂存再跑绿，那一行确实没在最新的绿树里判过，攻方的 h14 是那行留在暂存里被判过。两种都不漏判，只是代价（条款 4）的口径不同；第二轮的提示写明两种都量 |
| H4 代价 | 丙此刻多判 1326 行，至多 351 条找不到（上界，粗估）；戊约 100 行豁免 | 丙此刻 1326 行，照窗口判法 355 行红、其中 101 行在任何产物里都找不到；戊全量 1794 行、455 行红 | 两腿 1326 行同数；红行数两腿口径不同，取攻方量的（复跑 `real.txt` 逐字相同），正推的 351 是它自己写明的上界。不论哪个候选，staged-green 停在 2026-09-22，头一趟要先清掉或登记这几百行 |
| H5 会红的样本 | 丙两版下 `h1a_original`、`h1b_bypass` 进窗口、今天漏，6/6；退回今天红 4/6 | 没写 H5 | 形状可用，第二轮对庚二照同一形补交错与人给基准两份 |

## 三、处置

不收敛：照今天的外壳，五个候选与正推加的庚都出局；站着的只有攻方提的庚二与 wu_cf，都被攻过零轮。照规则开第二轮 `c439-r2`：

- 攻方（Opus）只攻庚二（含主 agent 补的「强制跑过不前移」）与 wu_cf，换攻击面：庚二的「里层判的就是外壳那棵树」怎么落到上游 `gate.sh --staged`（它今天自己从暂存区重套，第 56、87 行），落法本身会不会再开一个口子；`stage-must-run.sh` 的复用一侧（用同一条 ref）在庚二下会不会认错树；两个以上会话同时跑外壳；wu_cf 在产物重跑一轮之后表不许涨的那一形，量出要回改多少行。
- 辩方（Sonnet）复核这份判决：出局的丙、戊、门禁写的乙、按树挂的丁，打中它们的形是不是同样打中庚二与 wu_cf；第二节 H3 那一格两种代价口径各量一遍。

不论哪个候选胜出都要做的，记进欠账表：

- C836（staged-green 挂的树不一定是被判的树）：今天的 `stage-must-run.sh` 复用判据就读这条 ref，交错改暂存区或人给 `GATE_BASE`、`--force` 跑外壳时，staged-green 挂上没判过的树，重阶段会被判「输入没变、可复用」。这一条与 C439（gate-ok 前移让同轮早先写的行不再被判） 走哪个候选无关。
- C837（实验页里抄短的产物行落在窗口外）：攻方在 `f111c3f3` 上量到 E164（崩溃放量新设计的小范围功能测试） 页 100 行、E168（删除借保留量的环窗口） 页 1 行整行抄的产物行在任何一版产物里都找不到，主 agent 核了一行属实；它们在今天两条路的窗口外。

## 回看决策

不涉及决策：C439（gate-ok 前移让同轮早先写的行不再被判） 管的是门禁的改动范围取法，`.claude/kb/decisions/` 下没有一条决策管它。
