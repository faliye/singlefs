# rot-defs-r3：定义修补第三轮（最后一轮）正文（2026-10-19）

<!-- doc-lint:not-numbers K1 K2 K3 R1 R2 R3 R4 R5 R6 R7 R8 R9 R10 R11 R12 R13 -->

## 一、题面

第二轮判决 `research/prompts/rot-defs-r2-main-verification.md` 第三节列了 R1–R13 十三条要再改的，工具员照它改完，报告 `/home/fy5090/.local/share/singlefs/work/rot-defs-r2-fixes/report.md`（sha256 def6bad5…；第一节逐条改动、第二与第三节钩子的验证与喂探针原样输出、第五节改过的定义清单、第六节留给主 agent 的五条）。这一轮是第三轮，也是最后一轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」）：只攻前两轮站住的形态；这一轮里新冒出来的零轮形态不再为它开一轮，写进判决的交用户表。

被判的文件（第二轮判决之后改过的定义、共用约束与规则）：`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/three-way-defense.md`、`.claude/agents/three-way-attack.md`、`.claude/rules/path-moves.md`。前两轮判过、这一轮没再改的 `investigator.md`、`kb-scribe.md`、`mutation-triage.md`、`experiment-designer.md` 一并在 diff 里，只当上下文，不再判。

## 二、实现今天的样子（主 agent 的观测，2026-10-19）

方案按 `crates/` 与 `scripts/` 今天的实现来谈（主工作区现状）。这一轮被判的定义句子引的钩子与脚本：

- `.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`：R2、R3 经 `hook-swap.py` 换上（工具员报告第二节）：直接执行 `e165_replay_bounded_enumeration` 带 `--pinned-cases` 照重型判（装置名与选项现读 `scripts/experiment-apparatus/e165-pin-red-examples.py` 的 `DEVICE_BINARY`、`DEVICE_FLAG`）；带 `--quick` 的轻型三条在环境里设了各自文件头列的 `*_BREAK` 开关时照重型判。注意：另一位工具员此刻正在改这两份里的 `cargo run --bin` 漏网一形与 `.claude/gate.d/doc-decisions.sh` 另 9 处出路句、`.claude/rules/verification.md` 一句（不在这一轮被判之列），腿读到的若与快照不同，以今天的为准、在报告里写明。
- `scripts/hosts/compile-then-swap.py`：草稿编过（`cargo build` 与 `cargo test --no-run`）才整份换进主工作区，回读 sha256（R1 改后的 `experiment-runner.md` 指到它）。
- `scripts/gate/rerun-failed-tests.py`：打出的命令次序是「范围声明在前、内存包装在后」（工具员报告第一节 R6）。
- `.claude/skills/crash-test/commit-stages.md`：监控行的抄法（R4 改后的第 14 条指到它）。
- `scripts/gate/merge-and-verify.sh`：`--skip-apply` 是整条命令一个开关（R5）。

开工快照：`research/prompts/rot-defs-r3-snapshot/defs-sha256.txt`（被判与上下文的十二份定义、被引的钩子与脚本）、`research/prompts/rot-defs-r3-snapshot/defs-vs-head.diff`（`git diff HEAD` 那十二份，266 行，sha256 af25ca31…；含批 4 与 R1–R13 两批未提交的改动）。

## 三、格

### K1　第二轮判决的十三条落没落、判决本身够不够得着（辩方）

- R1–R13 逐条对着 diff 与工具员报告第一节：落了没有、落得是不是判决那一格要的东西；工具员报告第六节五条（`cargo run --bin` 没收严、对没展开的词偏严、R3 子脚本开关偏严、verification.md 缺一句、doc-decisions 另 9 处）各判：是判决那一格本身没写到，还是工具员改窄了。
- 第二轮判决里有没有一格同样把钩子现状说错（第二轮判决第二节 K1 那一格记过第一轮判决错了一格，照同一个办法查第二轮的）。

### K2　前两轮站住的形态撑不撑得住（攻方）

- 前两轮判「不改」「站得住」的那几格（第二轮判决第二节 K1 前两行、「`04ceaa904` 四处」没打中那一格、第三节末一句「`experiment-designer.md`、`investigator.md`、`three-way-defense.md`（除 R8）站得住」），换一个子 agent 照改后的定义走一遍：会不会被钩子拒、会不会漏做、会不会做到本该归主 agent 的重活。不重复前两轮攻过的角度（第一轮攻 2026-10-17 那二十处改动与漏改的句；第二轮攻批 4 修补本身、产生结论的方法与 `04ceaa904` 四份）。
- R1 改后「执行员只执行编好的二进制」与 R2「直接执行装置本体放行收严」两句放在一起：执行员照 R1 跑产物时会不会撞 R2 的拒（装置本体不是 e165 的那一类放行，是 e165 的那一类拒）；被拒时定义有没有给出路。

### K3　前两轮报出的数与「未定项」清单能不能核（攻方）

- 第二轮判决第一节报的数（「核了 119 处引文、22 处裸行号」、攻方模型复跑四份 `same=yes`）与工具员报告第二、三节的喂探针输出（41 条喂法、各开关红几格）：照报告里的命令在副本上能不能复出来。
- 第二轮判决第四节「没核动的」两条（R2、R3 门禁一侧照整道跑是读源码推的；混合补丁没真跑）在这一轮改后的定义里还要不要靠它：改后的句子有没有把推的写成了事实。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | K1 | 复核第二轮判决与 R1–R13 的修补：判决那一格够不够得着、修补落没落在那一格上 |
| 云端攻方（Opus） | K2、K3 | 找前两轮站住的形态里撑不住的、前两轮报的数里复不出来的；不重复前两轮攻过的角度 |

## 五、被判的改动

`research/prompts/rot-defs-r3-snapshot/defs-vs-head.diff`（`git diff HEAD` 那十二份定义）。只看这一轮新改的一层：对着 `research/prompts/rot-defs-r2-snapshot/defs-sha256.txt` 记的那一版与今天的差，即工具员报告第四节的 diff。
