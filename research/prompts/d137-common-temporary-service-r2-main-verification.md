# d137-common-temporary-service-r2 判决（第二轮，2026-11-05）

被判：`research/prompts/_d137-common-temporary-service-r2-body.md` 第一节——第一轮判决（`research/prompts/d137-common-temporary-service-r1-main-verification.md`）与照它改过的 `.claude/agent-common.md` 第 64 行那一句。腿：辩方腿 `research/prompts/d137-common-temporary-service-r2-sonnet-output.md`（R 格），攻方腿 `research/prompts/d137-common-temporary-service-r2-opus-output.md`（A 格）。

## 核查怎么做的

- 开工快照（`research/prompts/d137-common-temporary-service-r2-snapshot/files-sha256.txt`，8 份）：两条腿都报 7 份相同、`scripts/prune-server/prune_server_client_selftest.py` 对不上（快照 4d35b619，腿读的期间几次变动，交回时 a0b77c18）。那一份是另一个会话照第一轮 H1 改的（拆分提案一百九十六节 ①），腿按函数名与此刻行号重对；下面复跑的几条都在它是 a0b77c18 时交的（交前现量）。
- 引文：辩方腿「核了 53 处引文、47 处裸行号，对上 53 处」，攻方腿「核了 33 处引文、9 处裸行号，对上 33 处」。
- 腿交回的命令，主 agent 经服务复跑（包装此刻 sha256 a0b77c18，这几条都派到第二台）：
  - 不带二进制起包装（任务 6834、6835）都退 0，日志「PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1」——第二台服务给了编译目录（第一轮 6780 同）。
  - 去掉 `CARGO_TARGET_DIR` 与 `PRUNE_SERVER_BINARY` 再起（任务 6838）退 2，原话「✗ 临时 prune-server 起不来：没给 --server-binary 与 PRUNE_SERVER_BINARY，环境里也没有任务服务给的编译目录（CARGO_TARGET_DIR）：不现编」。派到本机时服务不给编译目录（第一轮 6778 的服务记录 `"compiles": false`），所以同一条不带二进制的命令在本机会照 6838 退 2（推的：这一轮复跑都没落到本机）。
  - `timeout 1800` 写在包装外：提交时当场拒，原话「命令自己套了一层：timeout …：限时由服务定，命令里不套它」；写在 `--` 后面（任务 6836）收下、退 0。
  - 经包装起一份 `.sh` 的 required 脚本（任务 6837，`crash-point-replay-shard-run.sh --selftest`）退 0：「crash-point-replay-shard-run 自证通过：15 格绿」。

## 逐条判

| 条 | 腿 | 判 | 主 agent 现查 |
|---|---|---|---|
| R：第一轮 H1「采一半」站不住，前提已变 | 辩方 | 采，H1 改判「采」 | 包装此刻在没有二进制、没有编译目录时退 2（6838）；不带二进制的写法按派到哪台分红绿，与 `.claude/agent-common.md:40`「所有任务都要多机可用」相顶 |
| R：第一轮拿「编译目录 无」那一栏证本机，分不出机器 | 辩方 | 采 | 那一栏 6778 与 6780 逐字相同而 6780 进程里有 `CARGO_TARGET_DIR`；证据改引 6778 的 `"compiles": false` |
| R：H5、H7 的证据该引真起的，不是只喂钩子的 6777 | 辩方 | 采 | 改引 6778–6780 与 6837（`.sh`、`.py` 各起过一份） |
| R：H3 的旧钩子名在表头第 2、6、10 行 | 辩方 | 采 | 第一轮只写了 2、6；交改钩子名那一路时连第 10 行 |
| R：H4、正推缺口 3 不采站得住，理由换 | 辩方 | 采理由 | H4：24 行里只有 `crash-point-replay-shard-run.sh` 有第二种自证用法，孤例；缺口 3：读法在共用约束第 65 行，统一的「看 ✗ 行」会错判（6778 退 0 的日志里有一行 ✗） |
| A1：表头 `[--server-binary <二进制>]` 写不写都有一台会错 | 攻方 | 采 | 不写：本机退 2、第二台能跑（6834、6835、6838）；写：句子、表头、拒绝说明都没给出子 agent 可写的路径。根在服务没把这一层认成会编、本机不给编译目录：服务照会编的给编译目录，两台就都能不带二进制起。交做统一任务服务的会话与 prune-server 那一路 |
| A2：例外只管外层，`--` 后面那条的包装服务不拦 | 攻方 | 采 | 6836 里层收下；句子写明 `--` 后面那条同样不带包装前缀 |
| A3：仓副本里改过的 required 脚本照这一句读成不归表 | 攻方 | 采 | 表头第 9 行键只认本仓路径；句子写明仓副本里的同一份也算 |
| A4：写法交给表头注释，没有检查绑住，已有三处与包装对不上 | 攻方 | 采，交两处 | 表头第 7 行 own-server 那一句「裸跑不退 2」、拒绝说明「省去现编」、判档「要现编」都随包装这一次改动过时；表头随 A1 的根一起改（交 prune-server 那一路），绑住表头写法与包装用法的检查登账本，归工具员 |
| A5：`scripts/agents/dispatch-prompt.py:282`「自证在哪跑」占位写裸的 `<脚本> --selftest` | 攻方 | 采，记账 | 骨架另一行已拼整条包装写法；占位那一句登账本，下一次动 `dispatch-prompt.py` 时并进 |

## 改法（零轮）

被判句改成：

> 登在 `.claude/hooks/selftest-temporary-service.tsv` 里是 required 的脚本（仓副本里的同一份也算），写进报告的自证命令照那张表文件头 required 一条给的整条写法套临时服务（`<解释器>`：`.py` 用 `python3`，别的用 `bash`；这一层本身不算「交给服务的命令不带包装前缀」那一串里的包装，`--` 后面那条照样不带）

| 改法 | 零轮 | 去向 |
|---|---|---|
| 被判句照改（A2、A3） | 零轮 | 主 agent 改；第三轮（最终轮）等 A1 的根落地、表头稳定之后开 |
| A1 的根：服务把这一层认成会编、本机也给编译目录；表头随之写清带不带二进制 | — | 交做统一任务服务的会话与 prune-server 那一路 |
| A4 绑住表头写法与包装用法的检查、A5 骨架占位 | — | 登进拆分提案，归工具员 |
| H3 第 10 行 | — | 交改钩子名的那一路 |

## 回看决策

不涉及决策：这一轮判的是共用约束里的一句与配套的登记表、钩子、骨架、包装，没有动 `.claude/kb/decisions/` 里的任何分项。
