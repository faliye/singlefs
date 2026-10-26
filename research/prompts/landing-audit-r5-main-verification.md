# landing-audit-r5 判决：子 agent 落地审计第九、十批改动第二轮（主 agent，2026-10-23）

正文：`research/prompts/_landing-audit-r5-body.md`。两条腿：辩方（Sonnet，K1）`research/prompts/landing-audit-r5-sonnet-output.md`（sha256 `9a83b8885a6c519b246615f18aac685d34927d2cd6564a86c4183907c1f92080`，模型目录 `research/prompts/landing-audit-r5-sonnet-model/`）、攻方（Opus，K2、K3）`research/prompts/landing-audit-r5-opus-output.md`（sha256 `07b972c2877013bd99c9dafeb3849e7f0d599d51c8e70c147ae2f5993b5af356`，模型目录 `research/prompts/landing-audit-r5-opus-model/`）。

## 这一轮被判过的文件

- `.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/implementation-workflow.md`：第十批写的合入后验证那一步的例外与调度表两行；攻方轻的一条打中例外说法比 `scripts/gate/merge-and-verify.sh` 自己给的重跑出路窄。
- `.claude/hooks/heavy-test-guard.sh`：`MERGE_VERIFY_QUICK_STEP` 与两处拒绝说明；没人打中别的。
- `.claude/hooks/handback-guard.sh`：交回闸 ⑤ 新判法；两条腿都打中 `timeout` 包着、后面接 `; echo rc=$?` 的真跑完的命名检查被误拒（攻方 K2-2a、辩方额外发现），攻方 K2-2b 打中工具退出码不是命名检查自己的那几种照收；交回闸 ② 把仓外的报告认成仓里同名的那一份（攻方轻的一条）。
- `.claude/hooks/runner-dispatch-guard.sh`、`.claude/hooks/lib_prompt_scratch_line.py`、`.claude/hooks/completion-cleanup-guard.sh`（认草稿目录那一行）：收严成只认行首「草稿目录：」；两条腿都量了真提示上的代价（攻方 K2-1a、辩方 K1 第 4 条）；⑳ 读的 14 份真提示下一次归档会被删（攻方 K2-3a）；新库的围栏判法与 CommonMark 不一致、相对路径按钩子起跑时的当前目录解析（攻方轻的两条）。
- `.claude/hooks/lib_three_way_legs.py`、`.claude/hooks/selftest-registry.tsv`：见交回闸 ② 那一条；没人打中别的。
- `.claude/gate.d/code-tooling.sh`：break-switch-replay 格按字节收；攻方说没做到九十节 ㉛ 原文的「容错解码」，主 agent 判不采（「二、逐格判」K2 表）。
- `.claude/gate.d/break-switch-replay-files.tsv`、`.claude/gate.d/replay-comparison-units.tsv`、新样本 `.claude/gate.d/fixtures/code-tooling.sh/break-switch-replay-undecodable-red/`：没人打中。
- `scripts/agents/subagent_transcript.py`：攻方轻的一条打中开关值漏进环境时续做闸平常跑也会改判（续做闸在 `.claude/gate.d/break-switch-outside-selftest-pending.tsv` 里）。
- `scripts/agents/agent-watch.py`（自证四行）：没人打中；攻方 K3 打中的是 `progress_file_of`（九十节 ㉚ 那一行），不在被判那四行里。
- `.claude/kb/checks-owed.md` 三语的 C879（E165 根选择与杀伤界正式跑没有单元）那一句：辩方判站得住。
- `records/2026-09-16-subagent拆分提案.md` 第九十节 ㉘ 到 ㉛、调度记录节「第九批收尾」「第十批」：辩方判 ㉙ 里 ⑬ 那一件没有对象。

## 一、核查

- 开工快照：`sha256sum -c research/prompts/landing-audit-r5-snapshot/judged-sha256.txt` 27 份里只有 `.claude/gate.d/code-tooling.sh` 对不上，是腿跑着时别的会话改的。主 agent 取 HEAD 那一份、打上快照里 `judged-vs-head.diff` 的那一段，倒推出开工那一版，sha256 `99dd70c4dfddf000…` 与快照相同；它与写判决时主工作区那一份的差存在 `research/prompts/landing-audit-r5-snapshot/code-tooling-changed-after-snapshot.diff`（15 段）。两份腿报告引这份文件只有攻方第 283 行一处（`.claude/gate.d/code-tooling.sh:4217`「return "坏字节", (f"{stream} 里有不合法的 UTF-8」），在倒推出的开工那一版第 4217 行逐字找得到；`cite-check.py` 判它「那一行不含这句、原文在第 4268 行」，是行号随后来的改动挪了，不是腿引错。kb 快照里 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.en.md` 被别的会话改过，两份腿报告引它 0 处。
- 引文：`python3 scripts/agents/cite-check.py <两份腿报告> --root . --background research/prompts/_landing-audit-r5-background.md` 末行「✗ 核了 60 处引文、21 处裸行号，1 处对不上，21 处没判」，对不上的那一处就是上一条倒推核过的那一处。
- 攻方模型复跑：模型目录拷到草稿目录，`bash scripts/hosts/run-with-memory-cap.sh 4G bash <副本>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <新草稿目录> --model <副本>` 退 0，「抽了 3343 份（共 3343 份记录）」；18 份输出里 17 份与腿交回的 `out/` 逐字节相同；`leg_handbacks.out` 一行不同：「两边相同：715 次」变「716 次」，是腿交回之后真交回又多了一次（这一轮的腿自己的交回），判法相同。`watch_progress` 取此刻的文件系统，不比，复跑照样报「21 份」。
- 判决引的两个数在复跑输出里逐行核过：`report_verdicts.out` 改前拒、改后放四行 18、282、88、14 份，合计 402 份，其中 2026-10-14 之前 388 份、2026-10-14 到 10-21 14 份；`naming_real.out`「2026-10-22 起：改前放、改后拒：4 份」，例 `a7aa1608c74c3e21e`、`aa87dd9fe761d727f`、`aa68d151c2fa72bb7`。
- 辩方模型没复跑：腿报告写整条 `rerun.sh` 要一小时以上、最后四处改动之后只单跑了轻探针。判决只引两条腿一致的那几样（4 份误拒，辩方给的四个号里三个与攻方相同、另一个 `ab6dd5f35228122b7`），辩方独有的数（61 份、874 份、967 次、1464 份）不引成判决的依据，只作复核的旁证。

## 二、逐格判

### K1（辩方：第一轮判决够不够得着）

| 条 | 腿判 | 主 agent 判 |
|---|---|---|
| ⑯ 前一半 | 一半：结论站得住，理由够不着（加键后八次三方腿派发都在更早开的会话里，分不出读的是新定义还是缓存；拆分提案第 534 行写同一会话里改 frontmatter 不立刻生效），靠的是先例：三份三方定义自 2026-09-27 起带未知键 `required-inputs:`，之后照常派出 | 采：第一轮判决「这一轮派发坐实」那半句改成「靠先例」，结论不变 |
| ⑯ 后一半 | 站得住，「全部定义类型」那几个字不准（两张表 9 种与 10 种，定义 14 份） | 采措辞更正，结论不变 |
| 「没打中」几处 | 结局账 `.txt`、目录登记位置、交回闸 ⑤ 与 rsync 一支、merge-and-verify 闭包站得住；⑬ 站得住但记进九十节 ㉙ 的那一件没有对象（本机峰值表最小 1 MiB、峰值为 0 的 0 条）；⑭ 没复核 | 采：㉙ 里 ⑬ 那一件删掉；⑭ 两轮都只读了代码，第三轮点名量 |
| K2-4 并 K3-2 收严 | 一半：「收严不丢罩住的对象」字面不成立、当时就该算进代价，按今天的做法代价是 0 | 采，与攻方 K2-1a 合判，见 K2 表 |
| 交回闸 ⑤ 认退 1 | 站得住：只认退 0 反被第一轮攻方自己的对照打中（两条都真跑、lint 判红退 1） | 采，主 agent 原判维持 |
| K2-6 只补互指 | 站得住：删那五行，比对对象表 #15 到 #19 既没登记又没点名，格判红 | 采，主 agent 原判维持 |

### K2（攻方：第十批的修补本身）

| 条 | 腿报 | 主 agent 判 |
|---|---|---|
| K2-1a（重） | ⑰ 收严之后 402 份真提示从拒变放行，抽看 12 份全是子 agent 自己的 `report.md` | 采为事实、不回退收严：复跑核得 402 份里 388 份在 2026-10-14 之前、14 份在 2026-10-14 到 10-21、2026-10-22 起 0 份，都是没有行首「草稿目录：」一行的提示；照骨架派的提示今天都有这一行，代价落在不照骨架写的旧提示上。第一轮判决「收严不丢罩住的对象」那句不成立，改成「收严丢了不照骨架写的那一类，照骨架写的提示不受影响」；九十节 ⑳ 那一行点名要拒的 `a9717cf07fb6e7c21` 那一类从此不罩，写进 ⑳ 的去向列 |
| K2-2a（重） | 2026-10-22 起 4 份真跑完的命名检查交回被新 ⑤ 全拒，写法是 `timeout 230` 包着、后面接 `; echo rc=$?` | 采（两条腿各自量到、主 agent 复跑核过 4 份）。改法：`timeout` 包着的照收，只拒 `timeout` 杀掉的那一种（命名检查自己的退出码是 124 或 137，或输出里有 `rc=124` / `rc=137`）；进第十一批 |
| K2-2b（中） | 工具退出码不是命名检查自己的那几种照收：接 `| head` 被 SIGPIPE 杀掉（实测 141）、`; echo rc=$?`、`|| true` | 采，与 K2-2a 同一处改：认命名检查自己的退出码（`rc=` 一行或管道最后一段之外的写法不认），141 不算跑完；进第十一批 |
| K2-3a（中） | ⑳ 读的 14 份真提示在 `research/prompts/landing-audit-r3-opus-model/`，下一次提交归档时会被删，删后自证 507 种变 493 种、外加一行「!」照样退 0 | 采：那 14 份拷进钩子自己的样本目录、按 sha256 钉住，拷不到或对不上判红；进第十一批 |
| 交回闸 ② 仓外报告（轻） | 交回闸 ② 把仓外的报告认成仓里同名的那一份，真交回 46 次 | 采，与 K2-5 那一处认法同一个函数，进第十一批 |
| 例外说法窄（轻） | 定义里的例外比 `merge-and-verify.sh` 自己给的重跑出路窄 | 采措辞，进第十一批 |
| 新库围栏与 CommonMark（轻） | 不一致，真提示里 0 份撞上 | 记九十节，下一批工具改动 |
| 相对路径按钩子当前目录（轻） | 推的 | 记九十节，下一批工具改动 |
| ㉛ 严格解码 | 九十节 ㉛ 原文要「容错解码」，实现是严格解码 | 不采：㉛ 那一行的改法写的是「坏字节那一份记一条红并点名路径与开关，别的照判」，实现照做了（任务 34 交回的红样本读数），解码方式是手段不是验收 |
| 没打中 3 处 | 见腿报告 | 记没打中；单条腿一次 |

### K3（攻方：产生结论的方法与没看过的地方）

| 条 | 腿报 | 主 agent 判 |
|---|---|---|
| 第一轮打中的形状在真记录里 | K3-1 那五种在 142 条真命名检查里 0 次；K3-2 转述误拦 0 份；第十批在真记录里新拒的只有 ⑤ 那 4 份，全是误拒 | 采为观测：第一轮两条打中都只是造出来的输入上会误判，真记录里没有撞上；第十批收严在真记录里没拦到一处真问题，带来了 4 份误拒。这一条写进调度记录，作为「钩子收严之前先拿真记录量一遍」的数据 |
| 看门狗取别的子 agent 的 progress.md（中） | 2026-10-22 起此刻找得到 progress.md 的 58 份里 21 份取到的是别的子 agent 的 | 采：这是九十节 ㉚ 那一行（`agent-watch.py` 的 `progress_file_of`），有真影响，从「下一批」提到第十一批 |
| 开关值漏进环境改判续做闸（轻） | 推的 | 记九十节，下一批工具改动 |

## 三、判决

1. 第一轮站住的形态，第二轮复核后仍站住的：⑯（理由改成靠先例）、交回闸 ⑤ 认退 1、K2-6 互指、四处定义里的例外（措辞要放宽一处）。
2. 收严成只认行首「草稿目录：」保留，第一轮判决里「收严不丢罩住的对象」那句不成立，代价是 2026-10-21 以前不照骨架写的 402 份（复跑数）；九十节 ⑳ 的去向列写明 `a9717cf07fb6e7c21` 那一类不再罩。
3. 采的修补并进第十一批：K2-2a 与 K2-2b（交回闸 ⑤ 认 `timeout` 包着且跑完的、只认命名检查自己的退出码）、K2-3a（⑳ 的真提示拷进钩子样本目录钉住）、交回闸 ② 仓外报告、例外措辞、九十节 ㉚ 里看门狗 `progress_file_of` 那一处。记成九十节 ㉜。改法都是腿与主 agent 提的，被攻过零轮；第十一批改完开 landing-audit-r6，第三轮、最后一轮。
4. 记进九十节、下一批工具改动的：新库围栏判法、相对路径、开关值漏进环境改判续做闸；㉙ 里 ⑬ 那一件删掉。
5. 方法上的一条：钩子收严之前先拿真会话记录量一遍新拒与新放，第十批两处收严都是在造出来的输入上定的。记进九十节 ㉜ 的形态列，作为第十一批改法的验收之一。

## 四、交用户

无设计岔路。第十一批要不要现在开，照「一轮怎么开、怎么收」第 12 条收尾时问。

## 五、没核动的

- 辩方模型整条 `rerun.sh` 没复跑（一小时以上），辩方独有的数不引成依据。
- ⑭ 两轮都只读了代码，没量。
- 攻方报告第 284 行「与改前不逐条相同的另两处（读代码，推的）」没核。

## 回看决策

不涉及决策：被判的都是 agent 定义、共用约束、规则、钩子、门禁与协作脚本的工作流写法，加上两份调度记录与欠账表 C879 一句指向。
