# rot-defs-r1 判决：知识腐烂治理改过的八份定义，第一轮（2026-10-17）

被判的对象：`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/kb-scribe.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/tooling-writer.md` 在全仓知识腐烂治理里的改动（正文 `research/prompts/_rot-defs-r1-body.md` 第五节的 diff，28 增 26 删）。开工快照 `research/prompts/rot-defs-r1-snapshot/defs-sha256.txt`（八份定义加 `scripts/gate/merge-and-verify.sh`、`.claude/hooks/run-scope-guard.sh`、`scripts/mutation/mutate.sh`、`.claude/rules/verification.md`、`.claude/skills/crash-test/commit-stages.md` 十三份）与 `kb-sha256.txt`。

<!-- doc-lint:not-numbers K1 K2 K3 A1 A2 A3 A4 A5 A6 A7 A8 A9 B1 B2 B3 R2 -->

## 这一轮被判过的文件

.claude/agent-common.md
.claude/main-agent.md
.claude/agents/experiment-runner.md
.claude/agents/implementation-writer.md
.claude/agents/kb-scribe.md
.claude/agents/mutation-triage.md
.claude/agents/three-way-attack.md
.claude/agents/tooling-writer.md

## 一、证据核对（主 agent）

| 证据 | 核法 | 结论 |
|---|---|---|
| 开工快照 | `sha256sum -c` 两份快照表，两条腿交回之后各跑一次 | 十三份定义与脚本全对上，腿跑着的时候主 agent 没改。kb 快照里 `.claude/kb/decisions-history.md` 对不上：是主 agent 为 rot-decisions-r1 判决插的四条变更史条目（第 7477、8549、14650、14946 行），两条腿都没引这份文件（`grep -c decisions-history` 两份报告各 0） |
| 正推 `research/prompts/rot-defs-r1-sonnet-output.md`（sha256 28429e55…，204 行）、攻方 `research/prompts/rot-defs-r1-opus-output.md`（sha256 a9c4275a…，250 行） | `cite-check.py <两份> --root . --background research/prompts/_rot-defs-r1-background.md` | 「✓ 核了 99 处引文、58 处裸行号，对上 99 处，没判 58 处（其中裸行号没有引文可比 58 处）」；「对不上」0 处 |
| 承重的裸行号 | 主 agent 逐行 `sed -n` 现读 | `.claude/kb/decisions/13-验证路线.md:256`；`.claude/main-agent.md` 第 10、29、50、52、94、96 行；`.claude/agents/kb-scribe.md` 第 22、32、33、35、36 行；`.claude/agents/three-way-attack.md` 第 35、36、53 行；`.claude/agents/experiment-runner.md:31`；`.claude/agents/mutation-triage.md` 第 31、34 行；`.claude/agents/implementation-writer.md` 第 34、40、48 行；`.claude/agents/investigator.md:31`；`.claude/agent-common.md` 第 56、82、87 行；`.claude/hooks/run-scope-guard.sh` 第 27、32、44、46 行；`.claude/hooks/heavy-test-guard.sh` 第 29、35 行；`scripts/gate/merge-and-verify.sh` 第 33、34、44 行；`.claude/skills/crash-test/commit-stages.md` 第 23、24、26 行；`scripts/mutation/broad-mutation.py` 第 12、515、517 行；`.claude/rules/mutation-sampling.md` 第 57、79 行；`.claude/rules/implementation-workflow.md:10`——都与腿引的一致 |
| 攻方模型 `research/prompts/rot-defs-r1-opus-model/` | 整份拷进会话草稿目录，`bash scripts/hosts/run-with-memory-cap.sh 2G bash <副本>/tools/rerun.sh --repo <仓根> --frozen <仓根> --work <新目录>` | 退 0，末两行 `name=scope_probe same_as_model_out=yes`、`name=heavy_probe same_as_model_out=yes`；`scope_probe.out` 与入库那份 `diff` 逐字相同；五个文件的 sha256 与报告开头列的逐个相同。模型只把命令当 hook 输入喂给仓里此刻的 `run-scope-guard.sh` 与 `heavy-test-guard.sh`，不起 cargo |
| 正推腿的实测 | 同一类（hook 输入喂钩子），没有模型目录 | 它报的退出码与拒绝说明首行，攻方模型的对应格（kb-scribe `--write`、`three-way-attack` 原型 `cargo run`、轻型带不带声明）都给出同样的结果，当作第二次抽样 |
| `.claude/product-line-prefixes` | `git ls-files --error-unmatch` | 「Did you forget to 'git add'?」：是这一批新建、还没暂存的文件，与八份定义同一次提交；不是缺文件 |

## 二、逐格判

两条腿没有一格互相矛盾：正推判「一致」的十六处，攻方也没打中它们的字面；正推判「冲突 / 对不上」的四处（第 9、16、17、20 项）与顺带核出的三处（第 14、21、22 项），攻方分别对应 A5、A6、A2、A4，再加它自己量出的 A1、A3、A8、A9 与漏改清单 B1、B2。

| 格 | 判 | 依据 |
|---|---|---|
| K1 一致的十六处（共用约束六处、实验执行员两处、实现员一处、写回员一处、分诊两处、攻方路径一处、工具实现员一处、主 agent 验证段两处） | 站住 | 正推逐处对脚本、钩子、规则现读，两处实测（轻型带声明、子 agent 跑 whole）与攻方模型同结果 |
| A1 写回员照 kb-scribe 第 2、3 步与第 35 行跑 `doc-decisions.sh --write` 被范围声明闸按整轮拒，带不带声明都拒；主 agent 不带声明也拒一次 | 打中（重），主 agent 复跑模型坐实 | 能过的只有 `--write --check decision-items-sync` 加 `种类=items` 声明（模型第三格退 0；`doc-decisions.sh:39` 文件头许可这种写法，主 agent 今天跑 `--write` 的输出也是「写完 1 块：decision-items-sync」）。定义三处没写这种写法 |
| A2 攻方 3b「在自己的原型里这样调枚举函数跑小流不算重型」与 heavy-test-guard 冲突：原型包一链接 `singlefs-checker-tier`，`cargo run` / `cargo test` 就被拒；能过的只有先 `cargo build` 再直接执行二进制，而那正是闸文件头自认「罩不住」的一处 | 打中（重），两条腿各自实测同结果，主 agent 复跑坐实 | 冲突改前就在，这一批只改准了路径。解法要同时改定义与闸（下面第三节） |
| A3 实验执行员 1b 在 `scripts/experiment-apparatus/` 下裸跑 `cargo test` 按整轮拒 | 打中，复跑坐实 | `cargo test --release --bin e<号>-<简称>` 加 target 声明放行（模型第六格退 0） |
| A4 / 正推第 22 项 分诊第 5 步缺「点名的测试 ok 又打了 `NOT_RUN_NO_GPU_ADAPTER` 记本次未跑」一支与「基线不绿」 | 打中 | `.claude/rules/mutation-sampling.md:79` 与门禁 `checker-tier-crates-mutation-replay.sh` 第 90、103 行都有；分诊会把这几条记成没红 |
| 正推第 21 项 分诊第 2 步「这张表不跑」与 `mutate.sh` 按条锚点失效（`mutate.sh:47`、规则第 57 行）矛盾，改后第 5 步的 `⚓` 栏走不到 | 打中 | 同一份定义两处自相矛盾 |
| A5 / 正推第 17 项 main-agent 第 10 行把 D13 已定项 9 的全量弹窗条件摘短了（丢「对不上的键里有判器单元变了」那一半），也没指到数它的入口 | 打中 | `13-验证路线.md:256` 整行主 agent 现读；两条腿都 grep 不到「阶段把作废块数与变了的判器单元清单打出来」的实现（`scripts`、`crates`、`.claude/gate.d`、`.claude/skills`、`.claude/hooks` 零命中）——kb 与实现对不上，另立账 |
| A6 / 正推第 20 项 main-agent 第 94 行「不换树」与驱动文件头、commit-stages 第 2b 步 ③ 反着 | 打中（轻） | 出处写的是「不包暂存树、不换版本」，驱动每趟把 `--source` 那棵树换进库目录的专用树 |
| A7 main-agent 第 52 行「只改注释的不开代码轮」丢了规则的例外（文档注释与 `// SAFETY:` 注释照代码算） | 打中（轻） | 括注指到了规则；只读这一句的会错 |
| A8 main-agent 第 7、29、50、90 行四处合入验证不是同一套：验证段第 1 步无条件 `--skip-apply`，写段只给有依赖的那批打补丁；`merge-and-verify.sh:44`「`--skip-apply` 时照样读给的补丁目录，只是不打」，不核补丁打没打 | 打中（推的，主 agent 读脚本文件头核过第 44 行；没造树真跑） | 一段只有一批、彼此不依赖时，照第 1 步会在没打补丁的树上验出绿 |
| A9 agent-common 第 56 行只给轻型三条补了范围声明，同一句 harness 档的 `cargo test -p <crate> --test …` 没带；实现员第 4 步「合成一条命令」只在开头 export 一份声明会被按种类对不上拒 | 打中（轻），复跑坐实 | 每段各带自己的声明放行（模型第十二格退 0） |
| B1 八份里没改到的命令句：main-agent 第 29、94、96 行，three-way-attack 第 53 行，experiment-runner 第 31 行，implementation-writer 第 34 行（`rerun-failed-tests.py` 打出的命令不带声明） | 打中（轻） | 主 agent 照抄各被拒一次，能补声明再跑；子 agent 照抄被拒后要么自己改对、要么交回 |
| B2 别的八份定义没有这一批改过的旧说法（逐个 grep 0）；只有 investigator 第 31 行「用输入给的命令原样复现」在输入是整包 `cargo test` 时跑不了，定义没说收窄还是交回 | 打中（轻） | 复跑第九格：整包带 whole 声明退 2 |
| 正推第 16 项 main-agent 三处「`--all-packages` 才跑全部」说窄了 | 打中（轻） | `merge-and-verify.sh` 第 33、34 行：没给补丁目录、补丁改到根 `Cargo.toml` / `Cargo.lock` / `.cargo/`、补丁改到不落在任何包里的路径也全跑；补丁只改 checker 档包时测试本次未跑 |
| 正推第 9 项 实验执行员第 7 步「指针指到的那一行要以 `E7RESULT name=<名>` 起头」只有未入库的 `.claude/product-line-prefixes` 撑着、没有门禁格判 | 站住（形式一致）；登记表随这一批入库 | 判它的检查是 C837 欠的那一半，已在欠账表 |
| 正推第 10 项 实现员 7a 没要求报告给合表 `.claude/gate.d/crash-point-replay-items.tsv` 那一行（第四列「轻型取自重型的哪一项」登记的人只能猜） | 打中（轻） | 合表头第 7、8 行要求第四列非空 |
| 正推第 5 项 范围声明闸条目比 hook 实管的窄（漏了驱动脚本逐行判、崩溃放量入口） | 打中（轻） | `run-scope-guard.sh` 文件头「驱动脚本」一段 |
| 正推第 3 项 `E168_THREADS` 的例子多余（`capped.sh` 已设它） | 不改 | 无害，措辞问题 |
| B3 `run-scope-guard.sh:32` 文件头旧句 | 已知，归 tooling-writer 下一批 | 正文第二节已列 |

## 三、判决：这一批定义改动站不站得住，哪些要再改

这一批改动本身（二十处）没有一处把现状写错到让人跑错命令的地步；两条腿打中的都是**同一批改动没改到、或改得不够**的地方：范围声明闸与重型测试闸比定义写的宽，定义里几句命令照抄会被拒（A1、A3、A9、B1、B2），两处与规则或 kb 摘句（A5、A7），两处与驱动字面反着（A6、正推第 16 项），一份定义内部自相矛盾（分诊第 2 步与第 5 步），一处改前就有的定义与闸的冲突（A2）。

要再改的，按文件聚簇，交下一批 tooling-writer 一次改完（定义与它们点名的钩子、脚本同一批，改完走 rot-defs-r2）。改法都只被攻过零轮：

| 文件 | 改什么 |
|---|---|
| `.claude/agents/kb-scribe.md` 第 22、32、33、35 行 | `--write` 的写法改成 `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync`（A1） |
| `.claude/main-agent.md` 第 29、96 行 | 同上的写法（A1、B1）；第 94 行 `--full` 那句补范围声明（B1）；第 10 行改成只指路不摘句：「条件照 D13（验证路线） 已定项 9 那一句（单元键对不上、且对不上的键里有判器单元变了），数法 `bash scripts/crash/crash-point-replay.sh --list-items --with-keys`」（A5）；第 94 行「不换树」改成「不包暂存树、不换版本；判的是主工作区，先照第 2b 步 ① 核它等于暂存区」（A6）；第 52 行「只改注释」收进括注的指路里，不写短版（A7）；验证段第 1 步改成「写段打过的补丁目录带 `--skip-apply`，没打过的不带」（A8）；第 7、29、90 行「`--all-packages` 才跑全部」补上脚本第 33、34 行的另三种全跑与「补丁只改 checker 档包时测试本次未跑」（正推第 16 项） |
| `.claude/agents/three-way-attack.md` 第 35、36、53 行 | 3b 与写范围改成：调枚举函数的原型只 `cargo build`，经内存包装直接执行编出来的二进制（不 `cargo run`、不 `cargo test`）；第 53 行的 `cargo test -p <crate> --test <目标>` 补 target 声明（A2、A9、B1） |
| `.claude/hooks/heavy-test-guard.sh` 文件头第 35 行一带 | 把「攻方原型 `cargo build` 之后直接执行二进制」从「罩不住」改登成许可的用法，并写明原型里的流只许自己造（与 3b 同一句）（A2）；这是闸的改动，归 tooling-writer |
| `.claude/agents/experiment-runner.md` 第 31 行 | 1b 的 `cargo test` 写成 `cargo test --release --bin e<号>-<简称>` 加 target 声明（A3） |
| `.claude/agents/mutation-triage.md` 第 31、33、34 行 | 第 2 步改成「原文不是恰好一次的那几条打 `⚓`、不跑，别的条照跑」（正推第 21 项）；第 4 步补「基线不绿 N 行」要为 0；第 5 步在「那一行出现了」之前插入规则第 79 行那一支（`ok` 又打 `NOT_RUN_NO_GPU_ADAPTER` 记本次未跑，不算抓到、不算没红，另起一行报）（A4） |
| `.claude/agents/implementation-writer.md` 第 34、40、48 行 | 第 34 行：`rerun-failed-tests.py` 打出的命令前面补 target 声明再起（B1）；第 4 步补「串成一条时每一段各带自己的声明，不用一份 export」（A9）；7a 补「报告给合表那一行（项名、第二列键、第三列放量分项、第四列轻型取自重型的哪一项）」（正推第 10 项） |
| `.claude/agent-common.md` 第 56、87 行 | 第 56 行 harness 档那一截写成带 target 声明的样子（A9）；第 87 行范围声明闸条目补「崩溃放量三份入口、整轮门禁、`cargo test` 整包与整工作区都是 whole；命令位置上执行的驱动脚本读进去逐行判，脚本里每条要声明的命令各自带声明」（正推第 5 项） |
| `.claude/agents/investigator.md` 第 31 行 | 补「输入命令是整包、整工作区或整道门禁的，收窄到点名的目标或格再复现；收不窄的写进要主 agent 跑的命令」（B2） |
| `scripts/gate/rerun-failed-tests.py` | 打出的每条命令带 `SINGLEFS_RUN_SCOPE='种类=target; …'`（B1）；归 tooling-writer |
| `scripts/gate/merge-and-verify.sh` | `--skip-apply` 时对每份 `crates.patch` 跑 `git apply --check --reverse`，不过就红（A8）；归 tooling-writer |
| `.claude/hooks/run-scope-guard.sh:32` | 文件头旧句（B3，已在下一批清单） |

另立账（不改定义，记进调度记录延后项）：D13（验证路线） 已定项 9 说「阶段把作废块数与变了的判器单元清单打出来」，仓里没有打这份清单的阶段；`crash-point-replay.sh --list-items --with-keys` 打的是块数与判法单元指纹，不是「变了的判器单元清单」——要么补实现，要么改 kb 那半句；进下一批实现员或 tooling-writer 清单。

不开第二轮：两条腿结论一致，没有要收敛的分歧。下一批改完上面那张表再开 rot-defs-r2（第二轮攻第一轮的修补本身）。

## 四、没核动的

- A8「在没打补丁的树上验出绿」：按脚本文件头第 44 行推的，没造一棵没打补丁的树跑 `merge-and-verify.sh --skip-apply`；要坐实归 tooling-writer 改脚本时的自证。
- 两条腿喂钩子的 hook 输入里 `agent_type` 字段按类型名填的，Claude Code 真送的字段名与值没单独核；仓外检出记录里子 agent 的行带这个字段，相容。
- 「没打中的形状」两条腿各抽一次，不拿来支撑任何「没打中」；这一轮没有靠「没打中」立的结论。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 9 | 不影响 | 2026-10-17 不受影响：这一轮只判 main-agent 第 10 行对它的转述摘没摘句，没动它；「阶段打作废块数与变了的判器单元清单」今天没有实现，记进延后项，另立账。别的决策一条都不涉及：这一轮判的是八份定义的改动与钩子、脚本、规则对不对得上 |
