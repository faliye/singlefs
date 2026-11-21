# merge-verify-slots-governance-r1：判决（2026-11-20）

正文 `research/prompts/_merge-verify-slots-governance-r1-body.md`，开工快照 `research/prompts/merge-verify-slots-governance-r1-snapshot/`。这是这一题的第一轮。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（sonnet） | `research/prompts/merge-verify-slots-governance-r1-sonnet-output.md` | 格 2 成立（「同一棵树」规则里没有判据，靠脚本文件头补），旧规矩该改三处；格 3 六张表 3 行不对、README 一处列举不全、搬迁登记这一批 66 行成立而别人的 14 行不生效；格 4 部分成立，模块改名报错不静默 |
| 攻方（opus） | `research/prompts/merge-verify-slots-governance-r1-opus-output.md` | 第 1 格三项打中（调低上限、`slots=0` 不清、槽里 `repo/target` 不量不删）、一项只报一行（三槽留满）；第 5 格判定不中、文件头三处描述与第二台不符；「已经量的」两句对不上、两句核不到 |

核查（照 `.claude/rules/three-way-inference.md`「核查与判决由主 agent 做，不派子 agent 去核子 agent」）：

- 倒推开工快照的树：`task-server/task submit --cwd ~/.local/share/singlefs/work/merge-verify-slots-r1-verdict -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py merge-verify-slots-governance-r1 --out ~/.local/share/singlefs/work/merge-verify-slots-r1-verdict/merge-verify-slots-governance-r1-tree --repository /home/fy5090/code/singlefs` type=command-failed exit=1：「清单点名 21 份，倒推出 20 份、sha256 对上 20 份；倒推不出 1 份」。倒推不出的是 `scripts/gate/merge-and-verify.sh`：开工之后别的会话补过四条开关的连带点名。攻方腿开工时把那一版冻在草稿目录的 `frozen/merge-and-verify.sh`，sha256 c88d382f… 与快照相同，主 agent 拷进树里补齐。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/merge-verify-slots-governance-r1-sonnet-output.md research/prompts/merge-verify-slots-governance-r1-opus-output.md --root ~/.local/share/singlefs/work/merge-verify-slots-r1-verdict/merge-verify-slots-governance-r1-tree --background research/prompts/_merge-verify-slots-governance-r1-background.md`：「核了 36 处引文、67 处裸行号，4 处对不上，78 处没判」。
  - 4 处对不上都在攻方腿引的 `scripts/gate/merge-and-verify.sh`（第 1156、2419、1128、1134 行），工具报原文在开工版第 2411、1120、1126 行，第 1156 行那一句同样差 8 行。攻方腿报告开头写明它引的是交回那一刻主工作区的行号；四句原文主 agent 在开工版里逐字找到，只差行号，不影响判定。
  - 没判的 78 处里，判决用到的几处主 agent 逐处现查：`.claude/agent-common.md:37`、`.claude/main-agent.md:34`、`scripts/mutation/prove-red.sh:232`、`scripts/crash/crash-library-totals.sh:281`、`.claude/memory-bounded-parallelism.tsv:10` 与 `.claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh:258`、`.claude/gate.d/repository-copy-closure-exempt.tsv:41`（`build_unit_graph` 现在在 `scripts/gate/lib_function_units_unit_graph.py:106`）与第 44 行、`scripts/experiment-apparatus/README.md:7`、`scripts/gate/merge-and-verify-snapshot-lib.sh:315`、`:326`。
- 两条腿的模型都拷到 `~/.local/share/singlefs/work/merge-verify-slots-r1-verdict/` 下的副本里复跑（拷完逐份 sha256 与报告开头记的相同）：
  - 正推：`task-server/task submit -- bash <副本>/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs <快照目录> <运行目录> <副本>` type=succeeded exit=0。读数：「逐行核：86 行，有问题 14 行；生效 72 行」「生效行之间串的 (行,行,前缀种类) 0 处」；逐项弄坏十行，结局与预期逐项相同（M1、M8 FAIL，M3、M7、N1 ERROR，M2、M4、M5、M6 OK），「收尾：两份文件的哈希与开跑时相同」。模板自证 `task-server/task submit -- bash <副本>/tools/rerun.sh --selftest` type=succeeded exit=0「✓ 自检通过（查了 6 格）」。
  - 攻方：`task-server/task submit --cwd <核查目录> -- bash <副本>/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs <冻结副本> <运行目录> <副本>` type=succeeded exit=0，「name=rerun_compare lines=7 verdict=all-lines-match」。
- 门禁五格（正推腿点名）：`bash .claude/gate.d/code-tooling.sh --check repository-copy-closure,memory-bounded-parallelism,heavy-thread-defaults,hardcoded-quota,change-range-single-source` type=command-failed exit=1「5 格里 4 格红」。
  - 这一批改路径的那些行没有一行被判成「涨了」：拆分族认法已接上。
  - repository-copy-closure 格点名第 44 行「锚串……一处被判行都不命中」。
  - change-range-single-source 格点名搬迁登记第 746–759 行「旧前缀……在基准 4319aa2b0 里没有文件」。
  - 其余「涨了」的行都是别的会话新登的。
- 主 agent 另现查：
  - 任务日志里「这一趟不占槽：调用方给了编译目录 CARGO_TARGET_DIR」13 处。
  - 本机 `merge-and-verify-slots/` 下两槽的编译目录 `du -sh` 105G、91G。
  - `slot.kyz05duu/repo/target` 2.5G。
  - `slot.4u3rewvh` 里有 `kept`。

## 二、判决

1. **第 1 格（槽位拦不拦得住）**：
   - 改过的包不重编而判绿：不成立，拷进槽对内容变了的文件写新、修改时刻是此刻，cargo 照修改时刻重编。
   - `slot-changed-sources` 只收 `.rs`：坐实是少核，不致判绿。重编核对不核包里 `.rs` 以外的输入，是 `scripts/gate/lib_rebuilt_packages.py` 本来的射程，不是槽位这一版带来的，这一轮不改。
   - NFS 上两台同时挑槽：此刻走不到，第二台那一层一律给 `CARGO_TARGET_DIR`、从不占槽。
   - cgroup 杀掉的尝试留下的槽被抢错：不成立。
   - 三槽留满之后照一趟一份跑而不报：成立，只打一行 `!`、不进汇总，留着的槽没有期限。攻方腿说 `slot.4u3rewvh`「三槽里已有一槽被一趟作废了的验证占着」不对：mainagent(A)（管拆分与判定键的会话）回告那一槽是它这一批合入后验证有意留着的快照，第 2 步修补还要在上面重交。所以改法只报、不收。
   - 占盘上限的读法与超限处理：成立，三处坐实（模型复跑 lowered-limit、trim-scope、excluded-left-in-slot 三行与推的相同；本机两份编译目录与 `repo/target` 现量）。
2. **第 2 格（规则与技能正文）**：
   - `.claude/rules/test-script-discipline.md:65` 与脚本「槽位」一致，成立。但「同一棵树」规则里没有判据，补一句。
   - `.claude/skills/merge-verify/SKILL.md` 放槽那条命令与 `scripts/gate/merge-and-verify-snapshot-lib.sh:70`、`:330` 是同一条，成立；第 23 行「被下一趟替掉的那一趟的快照」是旧写法，槽位那一种下一趟不替掉它，改措辞。
   - 旧规矩：
     - 该改：`scripts/mutation/prove-red.sh:232` 把旧规则原文当规则名下的说法引；`scripts/crash/crash-library-totals.sh:281` 写「服务给编译目录、任务结束删」，与 `task-server/task_server_lib.py:287` 的「编译目录归树、不归任务」相左；`.claude/agent-common.md:37` 与 `.claude/main-agent.md:34` 写「不留给以后的趟或别的任务复用」，没给槽留例外。
     - 照留：`crash-library-totals.sh:280`、`scripts/mutation/prove_red_selftest.py` 三处是那份脚本自己的做法；`scripts/hosts/multi-host-run.sh:380`、`task-server/task_server_lib.py:1159` 是弄坏开关打开后的样子。
3. **第 3 格（登记行）**：不成立，五处要改。
   - `.claude/gate.d/repository-copy-closure-exempt.tsv` 第 41 行理由指到旧文件。
   - 同表第 44 行（暂存区新登的那一行，不是这一批改的）锚串零命中，门禁判红；那一处现在门禁不再点名，这一行删掉。
   - `.claude/memory-bounded-parallelism.tsv` 第 10 行理由把只在弄坏开关 `replay-jobs-by-memory` 下才走的按内存算并行写成常态。
   - `scripts/experiment-apparatus/README.md:7` 只列 5 个装置目录，现场 31 个。
   - e203 拆出的四份模块（`scripts/experiment-apparatus/e203_admission_scan_cost_counters_*.py`）没登 `.claude/preflight-exclude`，preflight 那 66 处里 12 处落在它们上面。

   `.claude/preflight-exclude` 改过的 31 行、搬迁登记这一批的 66 行、其余六张表改路径的行都成立。搬迁登记第 746–759 行是钩子改名的那 14 行，不是这一批的，交改钩子的会话。
4. **第 4 格（测试那一处）**：成立。用例在模块名换掉时报 `KeyError`、不静默；M2、M4、M5、M6 四种改坏照绿，那几处归 `scripts/agents/agent-watch.py --selftest` 罩，这一轮不加。
5. **第 5 格（多机与不兼容旧东西）**：判定成立。两台都跑得起、判得对；登记读坏退 1、`slots=0` 都不算为旧东西留的路；没找到别的兼容分支。
   - 文件头第 87、98 行与 `scripts/gate/merge-and-verify-slots.conf` 第 4 行说「两台各一份」与实际不符：第二台从不占槽。
   - 「只在本机占槽」照实写；要不要让第二台也占槽，交用户。
6. **「已经量的」**：两句是主 agent 正文写错。
   - 「省盘没量到」：开工前已有三趟占槽真编过，量得到，数在「交用户的」。
   - 「剩 66 处都在拆分前就有的文件上」：12 处在 e203 拆出的四份新文件上。
   - replace-batch 那一句只有调度记录可核，它是直接跑的文本工具、不经服务。
   - 「两台的锁互斥」核不到，这一条路此刻也没走到。

## 三、改法

都是第一轮的改法，改完开第二轮（第二轮派辩方腿复核这份判决，照 `.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」），不碰产品代码，不做 A/B。

1. 槽位（工具员）：
   - 挑槽只在按 `used` 排好的前 `slots` 槽里挑，多出来的不占。
   - `slots` 小于已有槽数（含 0）时照样清多出来的、没被占也没留着的槽，连同这一台的编译目录；占着的、留着的跳过并打一行。
   - 拷进槽之前删掉槽里 `repo/` 下的 `target/`、`target-*/`、`__pycache__/`（`.git` 照留）；放槽那条命令另带删 `repo/target`。
   - 「这一趟不占槽」进汇总，三槽里有留着的槽时逐个打槽名、留了多久、`.owner` 里的会话与放槽命令，只报不收。
   - 文件头第 87、98 行与 `merge-and-verify-slots.conf` 第 4 行改成只在本机占槽，那一行的说法改成「第二台那一层给了编译目录」。
   - 每条配自证格与弄坏开关。换上要等 mainagent(A) 重交的点这份脚本的开关复跑出结局。
2. 正文（工具员，同一件）：
   - `.claude/rules/test-script-discipline.md:65` 补「固定路径、每趟按内容拷进来的那一份算同一棵树」。
   - `.claude/skills/merge-verify/SKILL.md` 第 23 行措辞改。
   - `.claude/agent-common.md:37` 与 `.claude/main-agent.md:34` 给合入后验证的槽留例外，指到 `scripts/gate/merge-and-verify.sh` 文件头「槽位」。
   - `scripts/mutation/prove-red.sh:232` 引规则那一句改成现行规则。
   - `scripts/crash/crash-library-totals.sh:281` 出路改成「服务给编译目录」。
3. 登记行（主 agent 自己改）：
   - `.claude/gate.d/repository-copy-closure-exempt.tsv` 第 41 行理由改指 `scripts/gate/lib_function_units_unit_graph.py` 的 `build_unit_graph`，第 44 行删。
   - `.claude/memory-bounded-parallelism.tsv` 第 10 行理由照代码改。
   - `scripts/experiment-apparatus/README.md:7` 改成不逐个列。
   - `.claude/preflight-exclude` 登 e203 四份模块。
4. 搬迁登记第 746–759 行发给改钩子的会话。

## 交用户的

- 省盘实测（量的，本机）：
  - 改前一趟的编译目录 158G，326 个调试可执行合计 138.2 GB。
  - 槽位这一版两槽的编译目录 105G、91G：
    - `slot.4u3rewvh` 里 debug 可执行 624 个、合计 37.1 GB，平均每个约 59 MB，改前约 424 MB；抽一个，`.debug_*` 占 56%。
    - `incremental/` 53G，是现在最大的一块。
    - 可执行个数比改前多，是同一槽跨趟留下旧哈希的产物。
  - 要不要在槽位的编译里关掉增量（`CARGO_INCREMENTAL=0`），拿编译快慢换这 53G，交用户定。
- 第二台要不要也占槽（此刻第二台每一趟都从头编）：要占得改 `scripts/hosts/multi-host-run.sh` 给编译目录的那一层，另要过 NFS 上锁的那一支；不占就照实写进文件头。交用户定。

## 这一轮被判过的文件

- scripts/gate/merge-and-verify.sh：文件头「槽位」第 80–113 行与挑槽那段 python 判了调低上限、`slots=0`、留着的槽、第二台那几样，打中三处
- scripts/gate/merge-and-verify-snapshot-lib.sh：`choose_slot`、`take_slot`、`refresh_slot_snapshot`、`finish` 槽位那一支判了重编、占盘与清理，打中 `repo/target` 不量不删与 `slots=0` 不清
- scripts/gate/merge-and-verify-selftest-lib.sh：槽位九格对调低上限、`slots=0` 后清理、`repo/target`、第二台四样都罩不到
- scripts/gate/merge-and-verify-slots.conf：读法成立，第 4 行两台的说法与实际不符
- .claude/rules/test-script-discipline.md：第 65 行与脚本一致，「同一棵树」缺判据
- .claude/skills/merge-verify/SKILL.md：放槽命令与脚本同一条，第 23 行措辞是旧写法
- scripts/gate/README.md：第 42、44 行拆出文件的说法对，点名的文件都在
- .claude/gate.d/path-moves.tsv：这一批 66 行生效、实据过、0 处串；第 746–759 行别人的 14 行不生效
- .claude/heavy-thread-exempt：改路径的行锚串都命中、门禁不判涨
- .claude/gate.d/repository-copy-closure-exempt.tsv：第 41 行理由指旧文件，第 44 行锚串零命中
- .claude/gate.d/hardcoded-quota-exempt.tsv：改路径的两行命中、门禁不判涨
- .claude/memory-bounded-parallelism.tsv：第 10 行入口对、理由把弄坏开关那一支写成常态
- .claude/preflight-exclude：删的 17 行与新登的 10 行对，e203 四份模块漏登
- .claude/term-rename-exempt：12 行行内锚恰好 1 行、九条目录写法的目录都在
- scripts/agents/README.md：拆出的三份模块行对
- scripts/hosts/README.md：两份自证行对
- scripts/crash/README.md：自证行对
- scripts/prune-server/README.md：三份模块行对
- scripts/experiment-apparatus/README.md：新表行对，第 7 行只列 5 个装置目录
- scripts/agents/tests/test_agents.py：补丁打在 `agent_watch_alerts` 上，十种改坏结局与推的相同，模块改名报错不静默

## 回看决策

不涉及决策：这一轮判的是合入后验证的槽位、规则与技能正文、几张登记表与一份工具测试，都是治理文件，没有拿依据去推翻或确立 `.claude/kb/decisions.md` 里的任何一条。
