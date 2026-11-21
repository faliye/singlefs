# merge-anchor-check-tooling-r1：判决（2026-11-21）

正文 `research/prompts/_merge-anchor-check-tooling-r1-body.md`，开工快照 `research/prompts/merge-anchor-check-tooling-r1-snapshot/`。这一题治理文件轮第一轮，攻换进主工作区的合入后验证「这一批补丁新弄断的变异锚点」核（四份 `scripts/gate/merge-and-verify*.sh`）。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/merge-anchor-check-tooling-r1-sonnet-output.md` | 四份改动做的就是正文说的，读码推出拦得住那一形；第二节五条成立，出路两处小缺口（没留快照那一趟、只列 20 处）；代价会明显拖长 |
| 攻方（Opus） | `research/prompts/merge-anchor-check-tooling-r1-opus-output.md` | 打中五处：开跑之后别的会话只追加的行、补丁前就红的 RELEASE 行被判成新红、补丁改登记表带来的新红看不见、`--skip-apply` 那一条路一次都不核、出路三处做不成；自证分不出三处改坏；代价与注释对不上 |

核查：
- 开工快照的六份里 `scripts/gate/merge-and-verify.sh` 在腿跑着时被别的会话改了（只在文件头开关表的注释里给四条旧开关补点名，16 行），排队的 `snapshot-tree.py` 两台满载起不来、取消；主 agent 照 `.claude/skills/three-way/SKILL.md`「代码轮派腿之前记一份开工快照」手工倒推：那一份取工具员草稿 `~/.local/share/singlefs/work/merge-anchor-static-check/new/scripts/gate/merge-and-verify.sh`（换上的就是它），别的五份取工作区此刻，`sha256sum -c` 六份都 OK。
- `python3 scripts/agents/cite-check.py`（两份腿报告，`--root` 倒推的树，`--background` 正文）：「核了 64 处引文、41 处裸行号，对上 46 处，没判 59 处」，没判的是树之外的文件（变异判法与打补丁脚本的库，这一轮没改）；`--root` 换成仓根时 21 处对不上，都落在被改的那一份第 431 行以后（行号往后挪了）。
- 正推腿的真判法探针（拷进 `~/.local/share/singlefs/work/merge-anchor-check-tooling-r1-main/anchor-real-probe.sh`，sha256 与腿报告相同；主 agent 把「运行目录已在就拒」改成删了重建，服务撞内存上限放大重来时才续得上）在真仓上跑两趟：break 那一趟 `type=succeeded exit=0`「✓ break：判红，点名 逐盘核退回只核总数 命中 0 次」，第 4 次尝试用时 423 秒、内存峰值 15.20 GiB；control 那一趟 `type=succeeded exit=0`「变异锚点静态核（static-check）：CHECKED 8680 1996 13」「✓ control：判绿」。
- 攻方腿的样本补丁（`attack-anchor-check.patch`，sha256 与腿报告相同）打在主 agent 建的副本 `~/.local/share/singlefs/work/merge-anchor-check-tooling-r1-main/copy` 上，三份脚本与腿草稿 `new/` 逐份相同，串行交七条自证：
  - R0（样本格开、不改坏）`exit=1`：「✗ 自检：」0 行；「✗ 攻方样本 concurrent-append：开跑之后别的会话只追加了一行锚在这一批改掉的原文上，合进主工作区那一段的核照样退 0，合进去之后那一行命中 0 次」；「✗ 攻方样本 release-shift：补丁之前就红的 RELEASE 行 attack-release-red-row 只因这一批删了它前面一行、行号前移，被判成这一批带来的新红」；own-only、own-with-code、created 三格是「·」行。
  - R1 / R2 / R3（三处改坏、样本格开）都 `exit=1`、「✗ 自检：」0 行，各自红在 own-only / own-only 与 own-with-code / created。
  - R4 / R5 / R6（三处改坏、样本格关）都 `exit=0`「✓ 自检通过（查了 167 种）」：改坏了现有自证照样绿。
- 两条新开关证红（`code-tooling.sh --check break-switch-replay --item scripts/gate/merge-and-verify.sh`）`type=command-failed exit=1`「4 处不对（复跑了 153 条、1 份）」：新加两条对上，四条旧开关的文件头漏点名，已由别的会话补上（第一条核查那 16 行）。

## 二、判决

| # | 腿报的 | 判 | 依据 |
|---|---|---|---|
| 1 | 开跑之后别的会话只追加、锚在这一批改掉的原文上的变异行，合进去才断，哪一道核都报不出 | **采，量过** | R0 concurrent-append；`scripts/gate/merge-and-verify.sh` 文件头「开跑那一刻的指纹」写明登记表与 `crates/mutations.tsv` 不进起点；真合那一份不跑锚点核（`scripts/gate/apply-writer-patch.py` 的 `apply_planned` 只在 `--dry-run` 那一支跑） |
| 2 | 补丁前就红的 RELEASE 行因这一批删了它前面一行、行号前移，被判成新红 | **采，量过** | R0 release-shift（样本用假 static-check，真仓此刻有没有这种行没量）；`RELEASE` 不在去掉行号再比的那几种里 |
| 3 | static-check 的 RELEASE 与拷贝范围读主工作区那一份登记表，补丁新登记的看不见 | **采，读码，不归这一题改** | 改的是 `scripts/mutation/` 下变异判法读登记表那几处，那几份归另一个会话在改（变异键那一件）；发给它 |
| 4 | 带 `--skip-apply` 那一条路一次都不核 | **采，读码** | `scripts/gate/merge-and-verify.sh` 那一趟补丁目录为空，进不了核；主 agent 自己的探索性改动走这一条 |
| 5 | 出路三处做不成：只点名前 20 处、锚着的代码整段删掉的没写删行、没留快照那一趟点的快照已删 | **采** | 正推腿也报了后两处中的两处；不改会在弄断多行时多走一轮 |
| 6 | 自证分不出三处改坏（只改变异表的不核、变异表不换回原样、新建的不删） | **采，量过** | R4–R6 三条改坏下「✓ 自检通过（查了 167 种）」；R1–R3 攻方腿的新格各红在点名那一格 |
| 7 | 代价与注释对不上 | **采，量过** | 注释写两趟 static-check「各约半分钟到一分钟」；真跑 break 那一趟 423 秒、峰值 15.20 GiB |
| 8 | 判法复用 `static_check_report`、补丁之前的树、多份合成之后再核、核没做成一律判红、红了不打合进主工作区的命令 | **没打中**（两条腿都判） | 腿报告各自第二节 |

## 三、改法

都在 `scripts/gate/` 下改，交工具员一次改完；这一轮的改法在第二轮攻。

1. 判决第 1 条：开跑那一刻另记「`crates/mutations.tsv` 里锚在这一批改到的文件上的变异行」（攻方腿 F2），合进主工作区那一段多一条核：重算这一份，多出来或变了的退非 0、不合，出路写以合入那一刻为基线整批重交。
2. 判决第 2 条：`RELEASE` 进去掉行号再比的那几种（攻方腿 F3），`--dry-run` 那一支同一处一起改。
3. 判决第 4 条：带 `--skip-apply` 的那一趟用倒打出的原样摆补丁之前的树，照样核（攻方腿 F5）。
4. 判决第 5 条：新红全数写进 `anchor-check.log`（标准输出照截前若干处并写明一共几处）；出路补「锚着的代码被整段删掉的写进 `mutations-delete.txt`」与「没留快照的带 `MERGE_AND_VERIFY_KEEP_COPY=1` 重交」（攻方腿 F6、正推腿第三节）。
5. 判决第 6 条：自证收攻方腿写的 own-only、own-with-code、created、release-shift 四格（F7，样本补丁在 `~/.local/share/singlefs/work/merge-anchor-check-tooling-r1-opus/attack-anchor-check.patch`），concurrent-append 那一格照改法第 1 条改成必绿；各配弄坏开关。
6. 判决第 7 条：注释的耗时换成量到的数（两趟 static-check 一共约 7 分钟、峰值约 15 GiB，真仓一趟）。

## 交用户的

无。判决第 3 条发给管变异键那一件的会话。

## 这一轮被判过的文件

- scripts/gate/merge-and-verify.sh：判了开跑那一刻的指纹不罩变异表（判决第 1 条打中）、`--skip-apply` 那一趟不核（判决第 4 条）与内联程序摆补丁之前的树
- scripts/gate/merge-and-verify-steps-lib.sh：判了 `anchor_check_new_breaks` 的出路三处（判决第 5 条）与耗时注释（判决第 7 条）
- scripts/gate/merge-and-verify-items-lib.sh：判了调用点在算好合入补丁之后、红了记一项不打合入命令
- scripts/gate/merge-and-verify-selftest-lib.sh：判了两格样本分不出三处改坏（判决第 6 条）
- scripts/gate/apply-writer-patch.py：判了复用的 `static_check_report` 不把 RELEASE 去行号再比（判决第 2 条）、真合那一份不跑锚点核
- research/prompts/_merge-anchor-check-tooling-r1-body.md：判了题面量到的缺口与第二节五条，两条腿逐条答过

## 回看决策

不涉及决策：这一题只改合入后验证这一道协作工具，不碰文件系统的设计条款。
