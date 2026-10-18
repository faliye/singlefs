# C794 盘上隔离位：没定的六处条款，第一轮正文（2026-10-15）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 -->

## 一、这一轮要判什么

用户 2026-10-15 在 `research/prompts/c794-r1-forks.md` 两条候选里选了「盘上隔离位」（改用户 2026-09-25「核出坏的才豁免」，`records/2026-10-13-代码欠账批.md`）。实现员已在副本里把它做了一遍（报告 `~/.local/share/singlefs/work/c794-isolation-bit-measure/report.md`，补丁全文 `research/prompts/_c794-bit-r1-diff.md`，不合入），碰到八处条款没写、先按最保守读法做的（报告「七、设计上没定的」）。这一轮判其中六处条款问题；Q6（GPU 内核里旧豁免留下的没人读的表与阶段）照 `.claude/singlefs-ai-sop/rules/code-discipline.md`「用不到的代码删掉」删、Q7（坏盘输入「跨度越过单元区末尾」最大跨度改 0x3FFF）是测试装置的细节，两处主 agent 定，不在射程里。

出口：六处各定一条条款（判据、射程），连同会红的用例与变异的形状；没收敛的写岔路单交用户。判完与用户这次定案一起写回 D3（空间分配） 已定项 7、D19（块指针的结构与宽度预算） 已定项 5、I-3.1、I-3.11，补丁交实现员补证红、合入、走代码轮。

## 二、实现今天的样子（主 agent 的观测，2026-10-15 现查）

冻结副本 `~/.local/share/singlefs/work/c794-bit-r1/repo/`：主工作区 2026-10-15 的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`litmus/`、`.claude/heavy-thread-default` **打上隔离位补丁之后**的样子（实现员副本原样拷来）。`crates/*/src/` 下 `.rs` 与 `.wgsl` 100 份的 sha256 在 `research/prompts/c794-bit-r1-snapshot/crates-sha256.txt`。行号按冻结副本。方案按 `crates/` 这份实现来谈；改前的样子看补丁的减行。

- 写者：隔离时在单元每块盘上的分配记录里置隔离位（借分配记录跨度段次高位，`ALLOCATION_RECORD_RELEASED_FLAG` 仍是最高位），置位 1 处、清位 3 处（实现员报告「三」那一节按类现数：147 处里置位 1、清位 3）。
- 池级 checker：`crates/singlefs-checker/src/walk.rs` 里按读盘核判豁免的那一套删掉，改成按隔离位豁免（报告「checker 豁免」12 处）。GPU 判器三份 `.wgsl` 与 `crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 跟着改。
- 验收（实现员在副本里量的）：瞬时隔离的镜像一条不变量都不红；同一份清掉隔离位、写者漏释放、悬空单元、活单元记成已释放四份都红。证红、harness 全量、checker 档测试都没跑。
- 判决 `research/prompts/c794-r1-main-verification.md`：两腿独立量到瞬时隔离与写者漏释放两份镜像逐字节相同；盘上隔离位只丢「写者谎报隔离」一种检出。

## 三、格

| 格 | 判什么（实现员先做的读法写在括号里） |
|---|---|
| Q1 | 已释放位与隔离位同时置着的记录怎么认（写者不写这一组合；core 读者在任何写之前拒成 I-1.1，GPU 恢复内核同一失败点码；池级 checker 当已释放、不豁免、不另判红）。三处读者对同一组合的判法一致吗，该不该一致；拒的射程（坏盘才走得到）合不合 D22（单元原子性怎么合成） 的失败处置 |
| Q2 | 回退重新引用隔离单元时清不清位（清，随回退那次发布落盘）。不清的读法「checker 只在没有根引用时看位」留下什么盲点；清的那一写在回退的哪一步、是不是在「任何写之前拒」那一判之后（D23（journal 的角色与格式） 已定项 14） |
| Q3 | 池级 checker 要不要把「隔离位置在有根引用的记录上」判违例（没加，只是不豁免）。加了能抓到哪一种写者缺陷、会不会误判 Q2 那一写落盘前的崩溃状态 |
| Q4 | 回退判「隔离单元」那一处（`crates/singlefs-core/src/mount.rs` 的 `resurrection_against_the_current_account`，今天按「每块盘仍分配、同分配代、现行那一版不引用」推断）要不要改成看位（没改）。改了之后不带位的那一类（只能是写者漏释放）的去向；与代码轮 `research/prompts/code-backlog-b2-code-r1-main-verification.md` 的 C572 判决对不对得上 |
| Q5 | 豁免按记录还是按单元（按记录：带位的那一条豁免，同一单元另一块盘上不带位的照判）。I-3.1、I-3.11 原文按单元（主 agent 2026-09-25 定，为「读盘只核出一份坏」时两块盘的账对称）；写者两块盘一起置位之后两种读法在写者写出的镜像上是不是同判，在崩溃状态（只置了一块盘就崩）上是不是同判 |
| Q8 | 理想模型（`crates/singlefs-harness/src/model.rs` 的 `ObservedAllocationRecord`）要不要观测隔离位、判活单元的记录不带位（没加）。加了在随机历史与故障注入里能抓到哪一类漏清，代价多大 |

## 四、分工

| 腿 | 立场 | 格 |
|---|---|---|
| 云端正推（Sonnet） | 从条款与冻结副本的实现推出每一格该取哪个读法，逐格核它与相邻条款（D19 已定项 5、D23 已定项 14、I-3.1、I-3.11）一致 | Q1–Q5、Q8 |
| 云端攻方（Opus） | 对实现员先做的读法与它的对面读法各造反例：在冻结副本的拷贝上造镜像、崩溃状态或历史，找让读法判错（漏判真泄漏、误判合法状态、随崩溃点变）的那一段 | Q1–Q5、Q8 |

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格两条腿都站实现员的读法、攻方给不出反例 ⇒ 照实现员的读法定 | 攻方报告那一格写试过的形、没打中 |
| 一格的某个读法被造出判错的段 ⇒ 那个读法出局 | 一段可复现的镜像、崩溃状态或历史 |
| 一格两个读法都站得住 ⇒ 写岔路单交用户 | 两条腿对两个读法都给不出反例 |

⚠️ 打中之后先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 六、交付要求

- 报告先写进 `research/prompts/c794-bit-r1-<腿名>-output.md`（正推 `-sonnet-output.md`、攻方 `-opus-output.md`），每一次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建，回复只写短句；早点开写。
- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数；引代码写冻结副本里的路径与行号。
- 草稿放腿自己的目录 `~/.local/share/singlefs/work/c794-bit-r1-<腿名>/`；量法拷冻结副本，不在冻结副本里改。模型与驱动随报告拷进 `research/prompts/c794-bit-r1-<腿名>-model/`（含 `tools/rerun.sh`，从 `scripts/agents/three-way-rerun-template.sh` 拷来、只换 `rerun_steps` 那一段；复跑经 `scripts/hosts/run-with-memory-cap.sh`）。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 24G 的上限（librocksdb-sys 在 16G 下会 oom），同一时刻只起一条 cargo；用自己的编译目录，不从别处拷编译产物。前台命令预计超过 4 分钟的拆成小段。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
- 新写的文字不用「层 0」「第一段 / 第二段」「第一条流 / 第二条流」这几种旧名（`.claude/kb/term-renames.md`）。
