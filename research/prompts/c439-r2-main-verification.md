# c439-r2 判决：C439（gate-ok 前移让同轮早先写的行不再被判）第二轮（2026-10-15）

被判的对象：第一轮判决 `research/prompts/c439-r1-main-verification.md` 留下的两个零轮候选——庚二（基准取 `refs/sop/staged-green` 的树对这次的暂存树；里层判的就是外壳 `git write-tree` 那棵；调用方给了 `GATE_BASE` 不前移）与 wu_cf（quoted-result-lines 改全量判，豁免按（路径、行文）计数，只缩不涨比引入时的固定基线）；连同第一轮出局的丙、戊、门禁写的乙、按树挂的丁判得对不对。脚本沿用第一轮的冻结副本 `~/.local/share/singlefs/work/c439-r1/frozen/`。`crates/` 不涉及。

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 -->

## 一、证据核对（主 agent）

| 证据 | 核法 | 结论 |
|---|---|---|
| 冻结副本 | `sha256sum -c research/prompts/c439-r2-snapshot/scripts-sha256.txt`（与第一轮同一份） | 10 份全对上 |
| 辩方 `research/prompts/c439-r2-sonnet-output.md`（sha256 64419a0a…）、攻方 `research/prompts/c439-r2-opus-output.md`（sha256 f9224e91…） | `cite-check.py <两份> --root <仓根> --background research/prompts/_c439-r2-background.md` | 「✓ 核了 74 处引文、18 处裸行号，对上 74 处，没判 18 处」；没判里 5 处是腿在临时仓里造的样本页（`900-x.md`、`901-y.md`、`two.txt`），不是仓里的路径 |
| 钟点 | 两腿产物按钟点、时区词与带钟点的时间戳扫 | 没有命中 |
| 复跑 | 两腿模型目录拷进会话草稿目录，经 `run-with-memory-cap.sh 16G` 跑各自的 `tools/rerun.sh` | 辩方 5 份（`k6-examples.txt`、`k1-tree-identity.txt`、`h3-cost.txt`、`small-checks.txt`、`summary-L2.txt`）、攻方 12 份（`k1-binary.txt`、`k1-break.txt`、`k1-force-preflight.txt`、`k1-tree-identity.txt`、`k2-reuse.txt`、`k3-real-window.txt`、`k3-sweep-summary.txt`、`k3-sweep.txt`、`k4-archive.txt`、`k4-scope.txt`、`k5-rerun.txt`、`tools-sha256.txt`）全部 `same=yes`，两条退出码 0。辩方长度 3 的扫描（`out/summary-L3.txt`）不在 `rerun.sh` 里，没复跑 |
| 外壳摘掉 `--force` | 读 `.claude/singlefs-ai-sop/scripts/preflight.sh` 第 29–33 行与冻结的 `scripts/gate/gate-staged.sh` 第 20、33 行 | 坐实（两腿都报了）：外壳的 preflight 把 `--force` 摘进 `PREFLIGHT_FORCE_GIVEN`，`PREFLIGHT_ARGUMENTS` 里不再有它，第 33 行起里层时收不到。第一轮判决与 C836（staged-green 挂的树不一定是被判的树） 里「带 `--force` 时外壳同样前移」那半句照字面走不到，改掉 |
| NUL 字节 | 读 `.claude/gate.d/doc-text.sh` 文件头第 29、73 行与第 674 行 `HEAD_BYTES` | 坐实攻方的前提：前 8 KiB 里有 NUL 的文件按二进制整份跳过；`.claude/gate.d/` 里没有一格拒绝 kb 页带 NUL。`gate_added_lines` 对二进制只得到 `Binary files … differ` 那一句（攻方报告第 229、269 行，模型里量过） |
| 弄坏开关在平常跑时也认 | 读冻结的 `scripts/gate/changed-paths.sh` 第 76 行 | 坐实：`gate_diff_base` 不看是不是在自证里，环境里留着 `LIB_CHANGED_PATHS_BREAK=fallback` 就不认 `GATE_BASE` |

## 二、逐格判

| 格 | 辩方 | 攻方 | 判 |
|---|---|---|---|
| K6 丙 | 判对；补两处（里层判外壳那棵树、人给 `GATE_BASE` 不前移）就是庚二，缺一处不行：守规矩续作里一次都没判到的，丙 984/2274、只补前一处 636、只补后一处 606、两处都补 258，258 条全是每一趟都带 `GATE_BASE=HEAD` 的续作 | 没判 | **第一轮判对**。庚二的两处补丁各自必要，辩方量过 |
| K6 戊（集合键、比 HEAD） | 判对；`wu_set` 在两形上 135/135 漏，`wu_cf` 0/135 | 没判 | **第一轮判对** |
| K6 门禁写的乙 | 判对；记号改挂被判的树就是庚二；弱修 `yi_clean` 窗口只增不缩 | 没判 | **第一轮判对** |
| K6 按树挂的丁 | 判对，理由要换：补上丙的两处之后与庚二覆盖逐段相同（23/23），重判更多、多一族 notes，是被庚二支配，不是继承丙的漏 | 没判 | **第一轮判对，出局理由改成「被庚二支配」** |
| K6 wu_cf 的残余 | 已豁免的存量行在同页搬位置，wu 一族四个变体都漏（135/135），庚二与今天都判到；标判不了 | 没判 | **不算打中**：跑前条款第二行要的是「一行新增的错引文」，搬位置的那一行仍是豁免表里登着的存量，不是新错。记成 wu_cf 的一条代价：存量行搬家不再被看一眼 |
| K1 庚二的落法 | 站得住：`read-tree --reset -u <外壳那棵树>` 20 案 20/20 同棵，今天的建法 14/20；外壳用私有 `GIT_INDEX_FILE` 不行（`gate.sh` 第 20 行清掉）；守卫放外壳 | 8 种树形（新页、`add -N`、符号链接、规范副本、eol、ident、gitlink、cargo 前缀）逐字节相同，没打中；今天的重套建法在第 56、59 行之间挪 HEAD 就判错树 | **庚二的落法站得住**，两腿各自量过、形不重合。落法要改上游 `gate.sh --staged`（收一棵树或由外壳建临时树），归上游 |
| K1 窗口判法：页里带 NUL | 没判 | 打中：`gate_added_lines` 交不出新增行，永久判不到；wu_cf 全量判当场红 | **打中不分辨臂**（今天、庚二与一切窗口类候选一起中），照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」那一条，共用前提另立一笔账：C838（kb 页里的 NUL 让按改动判的格当二进制跳过）。不拿它判庚二出局 |
| K1 环境里留着弄坏开关 | 没判 | 打中：`LIB_CHANGED_PATHS_BREAK=fallback` 留在环境里，格退 77，staged-green 前移 | **打中不分辨臂**，另立 C839（共用库的弄坏开关平常跑时也认）：`.claude/singlefs-ai-sop/rules/test-script-discipline.md` 要求「开关只给自证用，平常跑时不认」 |
| K2 复用一侧 | 没判 | 两形都打中、都不分辨臂：lkmm 登记的 `.claude/singlefs-ai-sop/scripts/lib.sh` 不在任何一棵树里，换了也判可跳过；一趟里有变异记本次未跑而整道退 0，staged-green 照样前移，下一趟判可跳过；`GATE_REPLAY_FULL=1` 经外壳被复用吞掉（推的） | **并进 C836（staged-green 挂的树不一定是被判的树）**：都是「staged-green 认证了它没判全的东西」 |
| K3 多会话 | 两种代价口径在庚二下：退出暂存口径每条续作重判 1 次，留在暂存口径 0 次；两种都不漏 | 两会话交错 3750 段全格扫 0 漏（第一次扫出 193 段是模型自己的并行竞争，改成各进程各抽一份格脚本之后 0 段，第一次的输出不当证据） | **庚二在多会话下站得住** |
| K4 射程 | 窄射程站得住：只给 doc-experiments 的 quoted-result-lines、results-cited、evidence-in-repo 三格单取一份新增行（读 staged-green，读不到取空树）；不带 `--staged` 那条路必须一起改，只改带 `--staged` 一路漏 789/2274，两路都改 258 | 整个 `gate_diff_base gate` 换掉时 archive-past-rounds 格的判法实际失效、改动路径 239 → 16359 | **窄射程**。两条路一起改，改的是两条路窗口的关系，字面不碰用户 2026-09-24 定案（共用脚本不认 `GATE_DIFF_BASE`），照跑前条款第五行交用户 |
| K5 wu_cf 的代价 | 没判 | 20 个重跑事件里登记表读得到的 5 个，E142（新池新建文件的干跑） r24→r25 那次换登记让 73 行在登记对照下变红、同一提交没回改；按全部产物对照 20 个事件共 1 行；用同一句「按这次改动判」的还有 results-cited 与 invariant-anchors | **wu_cf 站得住，代价是改上游规则**：共享规则「正文引产物的检查按「这次改动」判，不按全仓判」要先在上游改；重跑一轮可能要当场回改几十行 |
| 存量 | 没复跑 | 庚二此刻（staged-green 停在 2026-09-22，之后 143 个提交）这一格真判法判红 361 行 | 两个候选上线前都要先清这批：庚二是 361 行（照真判法），wu_cf 是全量 455 行（第一轮攻方，照窗口判法）。其中 101 行是 C837（实验页里抄短的产物行落在窗口外） |

## 三、处置

收敛到两个候选，各被攻过一轮（第二轮），打中它们的形都不分辨臂、已另立欠账；两轮结论一致，不开第三轮。照跑前条款第六行「两个以上候选站得住 ⇒ 写岔路单交用户」，岔路单 `research/prompts/c439-r2-forks.md`。另有一件跑前条款第五行要交用户：庚二要两条路一起改，改了两条路窗口的关系。

两个候选都要先做的（不论用户选哪个）：C838（kb 页里的 NUL 让按改动判的格当二进制跳过）、C839（共用库的弄坏开关平常跑时也认）、C836（staged-green 挂的树不一定是被判的树） 并进 K2 两形并去掉 `--force` 那半句、C837（实验页里抄短的产物行落在窗口外）。

## 回看决策

不涉及决策：C439（gate-ok 前移让同轮早先写的行不再被判） 管的是门禁的改动范围取法，`.claude/kb/decisions/` 下没有一条决策管它。
