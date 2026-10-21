# rot-defs-r3 判决：定义修补第三轮（最后一轮）（主 agent，2026-10-19）

<!-- doc-lint:not-numbers K1 K2 K3 R1 R2 R3 R4 R5 R6 R7 R8 R9 R10 R11 R12 R13 S1 S2 S3 S4 S5 S6 S7 S8 Z1 Z2 Z3 Z4 Z5 Z6 Z7 Z8 Z9 U1 U2 U3 U4 U5 U6 -->

正文 `research/prompts/_rot-defs-r3-body.md`；辩方腿 `research/prompts/rot-defs-r3-sonnet-output.md`（sha256 b6878d20…）；攻方腿 `research/prompts/rot-defs-r3-opus-output.md`（sha256 0a80f73e…），模型 `research/prompts/rot-defs-r3-opus-model/`。第二轮判决 `research/prompts/rot-defs-r2-main-verification.md`。

## 这一轮被判过的文件

`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/three-way-defense.md`、`.claude/agents/three-way-attack.md`、`.claude/rules/path-moves.md`（R1–R13 的改动）。

## 一、核查

- `python3 scripts/agents/cite-check.py <两份腿报告> --root . --background research/prompts/_rot-defs-r3-background.md`：「✓ 核了 109 处引文、7 处裸行号，对上 109 处，没判 7 处（其中裸行号没有引文可比 7 处；逐处列在上面）」。
- 开工快照 `research/prompts/rot-defs-r3-snapshot/defs-sha256.txt`：被判的八份定义与规则 `sha256sum -c` 都对上；不对的三份是 `.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/gate.d/doc-decisions.sh`，开工后另一位工具员（报告 `~/.local/share/singlefs/work/m3-close-w4-tooling-leftovers/report.md`）改了它们（`cargo run --bin … --pinned-cases`、10 处出路句），正文第二节已写明；两条腿都写了以快照版为准、今天版另比一遍（辩方：两版只差 A8、A9 两个用例）。不在被判之列，没倒推原样。
- 主 agent 现查：
  - `.claude/agents/experiment-runner.md` 第 32 行入库装置那一支原文：「执行员不对它起 `cargo test`、`cargo run`（checker 档，重型闸拒）……主 agent 带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）跑的单测命令；主 agent 编、换、跑完单测之后续派，执行员只执行编好的二进制（`target/release/e<号>_<英文名>`……）」。这一批主 agent 换装置都用 `compile-then-swap.py … --scratch <草稿> --release`，单测与产物都从 `<草稿>/compile-then-swap/target/release/` 起（调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md` 这一批的 E182、E183、E184 几行），主工作区 `target/release/` 下没人为它编。两条腿报的「本体没人编」与主 agent 自己的做法一致。
  - `.claude/hooks/heavy-test-guard.sh` 拒绝说明那一段：「提交之外任务确实要跑的：主 agent 先弹窗问用户，用户同意了才带 `SINGLEFS_HEAVY_TESTS=user-request` 跑」，与第 32 行的 `=commit` 对不上。
  - `.claude/main-agent.md` 第 14 条现文：「N、M、K 照 `.claude/skills/crash-test/commit-stages.md` 第 3 步的抄法数」。
- 攻方模型复跑：攻方报 11 行比对 10 行 same=yes，`r2-rerun-pre-swap` 一行 same=no（别的会话当时在主工作区重编 e165）。主 agent 没再复跑：要采的几格都在定义原文与钩子说明上现查过，不靠模型的数。

## 二、逐格判

| 格 | 判定 | 去向 |
|---|---|---|
| K1 R1–R13 落没落（辩方） | 采：十三条都落在判决点名的地方 | — |
| 「本体没人编」（攻方 K2-1、辩方 Z1） | 采（现查与主 agent 实际做法一致） | S1 |
| 「单测前缀」（攻方 K2-2、辩方 Z2） | 采：第 32 行与 `heavy-test-guard.sh` 的说明对不上；用哪个前缀是流程定案 | U1 交用户 |
| 「cargo run 括注」（攻方 K2-3、辩方 Z3） | 采（两腿都喂钩子量过：除 e152、e165 登记用法外 `cargo run` 放行） | S2 |
| 「循环变量当钉表」（攻方 K2-4） | 采：拒对，但拒绝说明给不出出路 | S3 |
| 「钉表重放走轻型」（攻方 K2-5） | 采现象（`--quick --item pinned:<N>` 放行、全点名也放行）；是不是漏洞关乎轻型「点名的项在轻型的域上跑」那条设计 | U2 交用户（零轮） |
| R3 拒绝说明把推的写成事实（攻方 K3、辩方 Z5） | 采：「弄坏开关下 `--quick` 不只跑轻型」只对两个开关值成立 | S4 |
| 不叫 `*_BREAK` 的规模变量（攻方 K3-7） | 采现象 | U3 交用户（零轮） |
| R4 指针偏半步、三栏没有数法（辩方 Z7） | 采指针那一半；数法 | S5；数法 U4 交用户（零轮） |
| R5 同形没点（辩方 Z6）、R9 同形没点（辩方 Z8）、第 10 行缩写 D13 已定项 9（辩方 Z9） | 采（读一遍即可核） | S6 |
| R2 的 `cd` 后 `./名`、拷贝改名两形（辩方 Z4） | 采现象 | U5 交用户（零轮） |
| K3 第二轮的数能不能核（攻方） | 采：引文数只能靠悬空 blob 复原（快照只存哈希）；第二轮模型的 same=yes 有两个没写下的条件 | U6 交用户（零轮） |
| 修补报告第六节五条（辩方） | 采：都是第二轮判决没写到，不是工具员改窄 | 1、4、5 已由另一位工具员补上；2、3 并进 S4 |
| R5 混合补丁（攻方） | 没打中 | — |

## 三、判决：要改的（交下一批 tooling-writer；第三轮之后不再开轮，这几处改法都被攻过零轮）

| 号 | 文件 | 改什么 |
|---|---|---|
| S1 | `.claude/agents/experiment-runner.md` 第 32 行与第 2 步 ④ | 「编好的二进制」写落点：主 agent 经 `compile-then-swap.py … --scratch <草稿> --release` 编过换上，二进制在 `<草稿>/compile-then-swap/target/release/e<号>_<英文名>`，续派提示写明这个绝对路径；那个编译目录留到这一段产物跑完再 `--clean` |
| S2 | 同上第 32 行 | 括注改成照钩子现状说：`cargo run` 只有 e152、e165 登记的重放用法被拒，入库装置一律不由执行员起 |
| S3 | `.claude/hooks/heavy-test-guard.sh`（经 `hook-swap.py`） | 命令词没展开、按可能是钉表重放拒的那一种，拒绝说明给出路：把变量展开成字面装置名与参数再起，或交主 agent |
| S4 | `.claude/hooks/lib_heavy_tests.py` 与 `heavy-test-guard.sh` 的拒绝说明、`.claude/rules/verification.md`「两档：轻型与重型」那一句 | 只写事实：「设了门禁或入口文件头列的弄坏开关时，`--quick` 的结果不作轻型证据，照重型拒」；删「不只跑轻型、照整道算」这类没量过的话 |
| S5 | `.claude/main-agent.md` 第 14 条 | 指到 `commit-stages.md` 第 3、4 步（「作废」在第 4 步） |
| S6 | `.claude/main-agent.md` 第 10 条（「打过的与没打过的」分两条命令，同验证段第 1 步）、派发表「要建计数实验」那一行与 `.claude/agents/experiment-designer.md` 第 20 行（引小节写全名「交岔路时写岔路单」所在那一节的全称，照 R9 的写法）、第 10 行（D13（验证路线） 已定项 9 的条件整句照抄 `.claude/kb/decisions/13-验证路线.md` 那一句，不缩写） | 照括号里各自改 |

## 四、交用户（零轮，不再开轮）

| 号 | 问题 | 候选 |
|---|---|---|
| U1 | 入库装置的单测在实验当中由主 agent 跑，带哪个前缀 | 甲：照 `heavy-test-guard.sh` 说明，属「提交之外任务确实要跑」，带 `=user-request`（用户已给过「授权所有操作」时不另弹窗、调度记录写依据）；乙：算提交那一趟的一部分，带 `=commit`（今天的写法与这一批的实际做法） |
| U2 | 轻型点名项能不能把重型的活整份做掉（形式化 `--quick --item pinned:<N>` 全点名、变异 `--quick --item` 全点名） | 甲：轻型点名设上限（条数或只许点核心部分之外 N 项）；乙：照现在，轻型点名本来就允许，靠范围声明事后看 |
| U3 | 改轻型规模的变量（`SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES` 一类）设了时轻型还算不算轻型 | 甲：照弄坏开关办，照重型拒；乙：只登记、照报 |
| U4 | 第 14 条三栏（✓、✗、本次未跑）怎么数 | 写进 `commit-stages.md` 一份数法（逐条用例行），第 14 条指过去 |
| U5 | 重放装置的 `cd` 后 `./名`、拷贝改名两形放行 | 甲：钩子按换到的目录解析相对名、按选项判；乙：记欠账 |
| U6 | 三方开工快照只存哈希，事后复原靠悬空 blob | 快照同时存被判文件的拷贝（或 `git stash create` 的对象名），`three-way-materials.py` 一并做 |

## 五、没核动的

- 攻方模型 `r2-rerun-pre-swap` 那一行 same=no 的原因（别的会话在重编）没复现。
- U2 门禁一侧（全点名的轻型是不是与重型判得一样）没真跑。

## 回看决策

不涉及决策：被判的都是 agent 定义、共用约束与规则的工作流写法，不碰 `.claude/kb/decisions/` 的任何分项。
