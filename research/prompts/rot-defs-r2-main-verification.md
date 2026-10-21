# rot-defs-r2 判决：定义修补第二轮与四份已入库没点名的定义（主 agent，2026-10-19）

<!-- doc-lint:not-numbers K1 K2 K3 A1 A2 A3 A4 A5 A6 A7 A8 A9 B1 B2 B3 H1 H3 H4 H5 H6 R1 R2 R3 R4 R5 R6 R7 R8 R9 R10 R11 R12 R13 -->

正文 `research/prompts/_rot-defs-r2-body.md`；辩方腿 `research/prompts/rot-defs-r2-sonnet-output.md`（sha256 7be90d17…）；攻方腿 `research/prompts/rot-defs-r2-opus-output.md`（sha256 740ff5b0…），模型 `research/prompts/rot-defs-r2-opus-model/`。第一轮判决 `research/prompts/rot-defs-r1-main-verification.md`。

## 这一轮被判过的文件

`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/kb-scribe.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`（批 4 的改动）；`.claude/agents/experiment-designer.md`、`.claude/agents/three-way-defense.md`、`.claude/agents/three-way-forward.md`（`04ceaa904` 的改动；`investigator.md` 两处都在）。

## 一、核查

- `python3 scripts/agents/cite-check.py <两份腿报告> --root . --background research/prompts/_rot-defs-r2-background.md`：「✓ 核了 119 处引文、22 处裸行号，对上 119 处，没判 22 处（其中裸行号没有引文可比 22 处）」。
- 开工快照 `research/prompts/rot-defs-r2-snapshot/`：被判的十二份定义 `sha256sum -c` 都对上；不对的是 `.claude/hooks/lib_heavy_tests.py`（快照之后又被改过，两条腿都写明探针与喂钩子用的是此刻那一版）与 kb 两份（`.claude/kb/term-renames.md`、`.claude/kb/checks-owed.md`，别的会话的写回）。腿引这几份的行由 cite-check 对着今天的文件逐字对上；没倒推快照原样。重型闸判法以此刻那一版为准：下面采的几条都在此刻那一版上量过。
- 攻方模型复跑：主 agent 把模型目录拷到 `~/.local/share/singlefs/work/rot-defs-r2-main-checks/model`，经 2G 包装起 `rerun.sh`，末四行原样：

```
name=compare file=probe.out same=yes
name=compare file=probe-deps.out same=yes
name=compare file=quick-sim.out same=yes
name=compare file=capped.out same=yes
```

- 主 agent 现查（对读）：`.claude/main-agent.md` 第 91 行「（今天是 E171（三oracle合谋测试） 修订 8）」，实验页 `.claude/kb/experiments/171-*.md` 有「### 第 1 次重跑·修订 12：流三段长重钉（2026-10-13）」；`.claude/agents/experiment-runner.md` 第 31 行 1b 写 `--bin e<号>-<简称>`、第 34 行第 2 步写 bin 名一律用英文名；`.claude/agents/three-way-forward.md` 第 37 行没有「冻结副本……只读」那一截，`three-way-defense.md` 第 36 行、`three-way-attack.md` 第 53 行有。入库装置单测被重型闸拒：这一批 E181、E182 执行员各撞过一次（看门狗的钩子检出记录），与攻方「入库装置单测」那一格同形。

## 二、逐格判

| 格 | 判定 | 去向 |
|---|---|---|
| K1 第一轮判决第三节表落没落（辩方） | 采：A1–A9、B1–B3 与正推四项全落、没有落偏 | 不改 |
| K1 三处字面出入（辩方） | 采：`agent-common.md` 第 87 行是第一轮判决那一格错（辩方把两份入口的声明各喂给 `run-scope-guard.sh`：whole 退 2、target 退 0），工具员改对；另两处没改偏 | 不改；第一轮判决那一格的错记在这里 |
| 辩方 H1：第一轮 A5 给的数法 `crash-point-replay.sh --list-items --with-keys` 起 cargo，是重型 | 采（辩方喂钩子量的） | R10 |
| 辩方 H3 / 攻方「入库装置单测」：执行员 1b 的单测命令对入库装置被重型闸拒，定义没给出路；bin 名写成简称 | 采（钩子量过，加这一批两次实撞） | R1 |
| 攻方「装置本体放行重放」：直接执行装置本体放行，`e165_replay_bounded_enumeration --pinned-cases` 就是形式化门禁那一格的活 | 采钩子那一半（量过）；「同一份活」是推的，判法收严时只认登记在门禁格里的装置用法 | R2 |
| 攻方「轻型开关」：带 `--quick` 时设弄坏开关照轻型放行，门禁那一侧会照整道跑 | 采钩子那一半（量过），门禁一侧按源码推的 | R3 |
| 攻方「监控行」：第 14 条的 grep 在提交那一趟数出 `✓ 0 ✗ 0`，没有本次未跑与作废的位置 | 采（照门禁 echo 模板造的日志量的；真日志同形是推的） | R4 |
| 攻方「混合补丁」/ 辩方 H4：验证段第 1 步一条命令里打过的与没打过的混写，脚本的 `--skip-apply` 是整条命令一个开关 | 采（读脚本，推的） | R5 |
| 攻方「声明夹进包装」：照 1c 在 `rerun-failed-tests.py` 打出的命令前套 `capped.sh`，三道钩子放行、跑起来退 127 | 采（`capped.sh` 真跑量过） | R6 |
| 攻方 K3「E171 修订」 | 采（现查） | R7 |
| 攻方 K3「产物名用简称」 | 采（现查：`research/results/` 177 份、带非 ASCII 字符 0 份） | R1 |
| 攻方 K3「正推腿冻结副本」 | 采（现查） | R8 |
| 攻方 K3「小节指到段落」 | 采 | R9 |
| 攻方 K3「`04ceaa904` 四处」 | 没打中（攻方量过标题都在）；辩方没判这一格 | 正推腿那一句的缺口另列 R8 |
| 辩方 H5：`.claude/rules/path-moves.md` 第 33 行还是裸 `--write` | 采（不在被判的定义里，规则文件归工具员） | R11 |
| 攻方「两处门禁与钩子的旧句」 | 采（`doc-decisions.sh` 出路句第 254、450、474 行；`heavy-test-guard.sh` 拒绝说明漏 `--check-table`） | R12 |
| 辩方 H6：例子只带声明片段、没带内存包装 | 采轻：例子旁指到 `agent-common.md` 包装那一条，不在每个例子里抄全 | R13 |

## 三、判决：要再改的（交下一批 tooling-writer，按文件聚簇；改法都只被攻过零轮，改完开 rot-defs-r3，最后一轮）

| 号 | 文件 | 改什么 |
|---|---|---|
| R1 | `.claude/agents/experiment-runner.md` 第 1b、2、4 步 | 入库装置（checker 档包 `src/bin/`）的单测与编译写成固定一步：装置写完交主 agent 经 `scripts/hosts/compile-then-swap.py` 后台编、主 agent 带前缀跑单测，执行员只执行编好的二进制；bin 名与产物名一律用登记的英文名（`e<号>_<英文名>`），不写简称 |
| R2 | `.claude/hooks/lib_heavy_tests.py`（经 `hook-swap.py`） | 直接执行装置本体放行收严：形式化门禁红例钉表重放那一格用的装置用法（`--pinned-cases` 一类、登记在门禁格里的）照重型判；别的装置本体照旧放行；自证加该红输入 |
| R3 | 同上 | 带 `--quick` 的轻型三条，环境里设了各自门禁的弄坏开关（`GATE_MUTATION_BREAK`、`FORMAL_VERIFICATION_BREAK` 一类）时不认轻型、照重型判 |
| R4 | `.claude/main-agent.md`「一轮怎么开、怎么收」第 14 条 | 监控行的数法改成与 `.claude/skills/crash-test/commit-stages.md` 同一套（报 ✓、✗、本次未跑三栏，数逐条用例行而不只数 crash-case 前缀），或直接指到那份 skill 的抄法，不另写一份 grep |
| R5 | `.claude/main-agent.md` 验证段第 1 步 | 打过的与没打过的补丁分两条命令（先 `--skip-apply` 那一批，再不带的那一批），写成步骤 |
| R6 | `.claude/agents/implementation-writer.md` 1c、`.claude/agent-common.md` 包装那一条 | 范围声明写在包装的前面（`SINGLEFS_RUN_SCOPE=… bash scripts/hosts/run-with-memory-cap.sh …`），`rerun-failed-tests.py` 打出的命令照这个次序拼；不在声明前面再套包装 |
| R7 | `.claude/main-agent.md` 第 91 行 | 不写修订号：「有实验登记钉死了常量、还没拍树的（以实验页最新一次修订为准）」 |
| R8 | `.claude/agents/three-way-forward.md` 第 37 行、`three-way-defense.md` | 正推腿补「冻结副本只读」那一截；写模型的正推、辩方两份同攻方第 8 条，报告开头给复跑命令与每个文件的 sha256 |
| R9 | `.claude/main-agent.md` 第 89 行 | 引成「`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」一节里「核查与判决由主 agent 做」那一段」 |
| R10 | `.claude/main-agent.md` 第 10 行 | 数法那一句写明它是重型，主 agent 带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request` 跑 |
| R11 | `.claude/rules/path-moves.md` 第 33 行 | `--write` 写成带 `--check decision-items-sync` 与范围声明的那一种 |
| R12 | `.claude/gate.d/doc-decisions.sh` 第 254、450、474 行出路句；`.claude/hooks/heavy-test-guard.sh` 拒绝说明 | 出路句改成只写那一格；拒绝说明补 `--check-table` |
| R13 | `.claude/agent-common.md` 第 56 行、`three-way-attack.md` 第 53 行、`experiment-runner.md` 第 31 行 | 例子旁加一句指到包装那一条，不抄全 |

`.claude/agents/experiment-designer.md`、`investigator.md`、`three-way-defense.md`（除 R8）在 `04ceaa904` 里的改动站得住，不改。

## 四、没核动的

- 攻方「装置本体放行重放」里「同一份活」与「轻型开关」门禁那一侧照整道跑，都是读源码推的，没在门禁上真跑；R2、R3 的改法按钩子一侧量过的形状写。
- 「混合补丁」没真跑 `merge-and-verify.sh` 混写那一条。

## 回看决策

不涉及决策：被判的都是 agent 定义与共用约束的工作流写法，不碰 `.claude/kb/decisions/` 的任何分项。
