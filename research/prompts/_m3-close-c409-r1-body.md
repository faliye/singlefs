# m3-close-c409-r1：崩溃点重放的常规层、大负载层怎么立（推论轮第一轮）正文（2026-10-18）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 K1 K2 K3 K4 -->

## 一、题面

D13（验证路线） 已定项 9 把崩溃点重放分三层（冒烟层、常规层、大负载层），射程写「常规层、大负载层在门禁里没有阶段」，欠块挂 C409（常规层、大负载层在门禁里没有阶段）。用户 2026-10-18 弹窗定「立」（原话即选项标签，记在 `records/2026-10-18-里程碑三收尾与近期欠账.md`「用户弹窗定案」）。这一轮要给出立法的候选与各自的代价要先量什么，交用户选；不判「立不立」。

- Q1　常规层的负载取什么：已定项 9 定案表写「中等规模负载（写请求数上千），精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore」。候选至少三条，各写负载怎么生成、写请求数怎么控在上千、与已登记的崩溃枚举用例与放量分项重叠多少。
- Q2　大负载层的负载取什么：定案表写「常规层负载 + 大规模负载 + 多设备档位交叉，N≥5 轮，精确全量」。今天多设备只有两块盘的镜像（三块盘以上已挪出里程碑三），「多设备档位交叉」在今天的实现上能取哪几档、取不到的怎么写。
- Q3　落在哪：新立门禁阶段，还是并进已有的崩溃点重放入口（`scripts/crash/crash-point-replay.sh`）与门禁 `checker-tier-crash-point-replay` 当新的项（`.claude/kb/milestone/03-third-txn.md`「待优化」第 13 行：同一件事不另立第二套装置）。
- Q4　触发：定案表写常规层「每次提交」。提交时六道重阶段已按函数单元键复用（D13 已定项 17），常规层每次提交跑的挂钟与第一次跑的挂钟各是什么量级、要先量什么；全量档「用户要求时跑」那一条与常规层「每次提交」怎么并存。
- Q5　C409 关账判据：立到哪一步算还清（每层至少一条负载登记、门禁一格判、判别力样本各一）。

## 二、实现今天的样子（主 agent 的观测，2026-10-18）

方案按 `crates/` 今天的实现来谈（主工作区现状，含里程碑四会话没提交的改动）。读过的路径与看到的事实：

- `.claude/gate.d/stage-inputs.tsv`：`crash-case:` 登记 29 条（`grep -c '^crash-case:'` 现数），其中 `crash-case:random-histories-and-fault-injection-given-states` 是随机历史七段取样与故障注入快档每张镜像建成给定状态表判，标 ignore，「单跑以十分钟计」（那一行第四段）。
- `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs`：固定脚本到 E 再正常卸载 `FULL_STATES_THROUGH_THE_UNMOUNT = 1_678_377_204` 个状态（第 104 行常量）。
- `scripts/crash/crash-amplification.sh`：`FULL_EXPAND_UP_TO=30`，`--list-items` 现列 20 项。
- `crates/singlefs-harness/src/history.rs`：随机历史的操作种类 17 种（`HistoryOperationKind`）；随机历史不做全量崩溃点重放。
- `crates/singlefs-core/src/make_filesystem.rs`：设备数取自交进来的设备身份个数；今天建池与测试只用两块盘（三块盘以上的欠账已挪出里程碑三，`.claude/kb/milestone/03-third-txn.md` 五·5）。
- `scripts/crash/crash-point-replay.sh` 文件头：入口不带参数是剪枝跑全部项、`--full` 全量、`--quick` 轻型；2026-10-18 起重型三种跑法快档红即退 1。
- 崩溃判定库按块键与判器单元键复用（`crates/singlefs-checker-tier/src/crash_identity.rs`、`verdict_store.rs`），提交时六道按键只判键对不上的（`.claude/rules/implementation-workflow.md`「复用重型门禁的判定」）。

## 三、格

### K1　候选与重叠（正推）
每个 Q 给至少三条候选（Q5 给一条判据即可），每条写：负载怎么生成、规模、跟已登记的用例重叠的部分、判法沿用哪一套（池级 checker、记录核对器、oracle、观察者）、要动 `crates/` 与 `scripts/` 的哪几处。

### K2　代价要先量什么（正推）
每条候选写出挂钟、状态数、判定库增量三样里哪样今天没有数，要建什么计数或实测才有数（`.claude/rules/three-way-inference.md`「岔路交用户定之前，每条路要有代价数」）。

### K3　哪条候选名不副实（攻方）
攻每条候选：它跑出来的东西是不是 D13 已定项 9 定案表那一行说的那一层（「精确全量」「写请求数上千」「N≥5 轮」「多设备档位交叉」逐个对）；有没有哪条只是把已登记的冒烟层用例换个名字再跑一遍。

### K4　并进还是另立（攻方）
攻 Q3：并进已有入口与门禁会不会让提交时那一趟挂钟失控、复用键罩不住；另立会不会撞「待优化」第 13 行。

## 四、分工

正推腿（Sonnet）答 K1、K2；攻方腿（Opus）答 K3、K4，并对正推交的每条候选攻一遍（交叉看对方的报告不行，两条腿各自独立作答）。主 agent 判决，交用户的岔路写进 `research/prompts/m3-close-c409-r1-forks.md`。
