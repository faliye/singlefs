# 根环实现第 10z 批第二十、二十一块：代码轮第一轮正文（2026-10-09）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 M1 M2 M7 M12 R C P Q F A -->

## 一、题面

这一轮攻两块：

| 块 | 照什么改 | 落点 |
|---|---|---|
| 第二十块（core） | 判决 `research/prompts/m3-batch10z-pqrs-code-r1-main-verification.md` 第二、三节；用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」：报 `StuckBehindTheWideProtection` 之前在分配器拷贝上预演推环，再按推完的根环表算上限、预演一次这次改动的准入（连同它会推的抬 F），放得下才列推环、放不下只列卸载再挂；推环入口先判冻结；C 记着落点而一个槽都没隔离时不报宽护、推环入口在任何写之前拒；出路一按 R 有没有根分两种说法并写明 R 的账永久读不出的后果；出路二写推几次、写掉几槽，N 等于推完隔离实际降了多少；补用例拦住攻方 M1、M2、M7、M12、正推活下来的变异与第九块 loop-span1 那一行 | `crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` |
| 第二十一块（checker 档测试） | 同一份判决 K4 checker 档、K5：卡池扫源码 `!=` 当宏的误判与另几类合法写法；崩溃放量扫源码名单外三种写法；`brace_depth_at_the_start_of_each_line` 遇原始字符串 | `crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs` 的测试模块 |

条款与用户原话：
- `.claude/kb/decisions/16-发布语义.md` 已定项 1 第 29 行（回退候选集保留最近 4 个可退到的不同状态）、第 36 行「抬 F 的上限」那一行、第 37 行「根槽这一次读坏」那一行；`.claude/kb/decisions/28-挂载期承诺量.md` 已定项 1 第 27 行第九项；`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14（冻结那一道）；`.claude/kb/invariants.md` 第 58 行 I-7.4（近 K 代块未被复用）；欠账 `.claude/kb/checks-owed.md` C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）、C762（抬 F 给根环表没记着的被抛弃根补的隔离一直不放）。
- 用户 2026-10-08 弹窗原话「显式推环命令加带出路的错」：不自动推环；推环只在用户显式调用时、调用即确认回退候选集掉到少于 4 个不同状态。用户 2026-10-09 弹窗原话「甲：预演推环与准入，推不通就不列」（弹窗里另三条：乙推完把 F 抬到最后一版、越过 D16 第 36 行；丁去掉推环入口；丙只改措辞）。

改动全文：`research/prompts/_m3-batch10z-tu-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-r1/batch10z-t/patch/report.md`（第二十块）、`~/.local/share/singlefs/work/m3-r1/batch10z-u/patch/report.md`（第二十一块）。上一轮两腿报告：`research/prompts/m3-batch10z-pqrs-code-r1-opus-output.md`、`research/prompts/m3-batch10z-pqrs-code-r1-sonnet-output.md`；攻方原型 `research/prompts/m3-batch10z-pqrs-code-r1-opus-model/`（`atk_part*.rs` 拼进那份 harness 测试的搭建函数，`assemble.sh`、`rerun.sh` 是拼法）。

## 二、实现今天的样子（主 agent 的观测，2026-10-09）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-tu-code-r1/tree/`，取在两块都合入、第 3553 行变异改过之后；被判文件 sha256 在 `research/prompts/m3-batch10z-tu-code-r1-snapshot/crates-sha256.txt`，与合入目录 `~/.local/share/singlefs/work/m3-r1/merge-10z-tu/after.sha256` 逐个相同。
- 第二十块关键位置（冻结副本 `crates/singlefs-core/src/mounted_session.rs`）：第 188 行 `WideProtectionAtTheCeiling`、第 208 行 `StuckBehindTheWideProtection`、第 221 行字段 `pushing_the_root_ring`、第 228 行 `RootRingPushRehearsal`、第 248 行 `ways_out` 交 `Vec<String>`；第 293 行 `DeviceKeepingWritesInMemory`（盘外一层，写记在内存、读没写过的块穿到底下的盘；第 315 行 `check_aligned_and_in_range` 照 `crates/singlefs-core/src/block_device.rs` 文件后端那一份抄的，原函数私有）；第 478 行 `rehearse_the_push_then_the_remainder_of_the_change`；第 622 行 `wide_protection_at_the_ceiling`；第 684 行 `RootRingPushRefused`（新成员 `PublishFrozenAfterAWriteFailureIsNotResentYet`）；第 1096 行推环入口、第 1123 行 `push_the_root_ring_on`。`mount.rs` 没动：预演里走的是抬 F 那一串自己，它调私有的上限函数，读的是盖着推环那几次写的盘。
- 合后验证（`~/.local/share/singlefs/work/m3-r1/merge-10z-tu/summary.txt`）：merge-and-verify 编译、186 个 harness 档测试二进制全绿、fmt、clippy 过（它报 code-source-discipline 红，是第二十一块一行变异替换文里的 `\u{7b}`，主 agent 改成 `token.as_bytes() == [0x7b_u8]` 后 mutation-tables 格绿、那一行单跑 prove-red 抓到）；static-check `CHECKED 3558 322 10`；e158 `95 passed`；e168、E171 装置编过，E171 sha 前 12 位 ab35105f861d；`raise_floor_takes_…` `--include-ignored` 55 passed，另两份慢档 23 / 16 passed、第九项相关 10 份全绿；checker 档快档 61 份 0 失败；带 gpu 的 lib 与 12 份挂 gpu 的测试全过、NOT_RUN 0；合后副本上 checker 档 13 行 prove-red 全抓到，harness 档整行替换的 6 行 prove-red 全抓到。
- 实现员自定、被攻过零轮的：
  - 第二十块：预演用「盘外包一层只记写的设备，原样走一遍推环入口与这次改动剩下的发布」，不另写一份预演；预演结局放成字段、不改错误成员；C 有落点无隔离复用 `NothingHeldUntilThisProcessWritesARingSlot`；N 取预演里推完前后每块盘隔离数之差；几条用例是注入级的（C 有落点无隔离、记着空串、预算、取不到落点、两槽 C 的 N、loop-span1），自然挂载造不出那几形；带被抛弃根那一形的 N 没做；unlink 之后走空被卡那一种没造出；
  - 第二十一块：use 整棵树读、按引进来的名字判，通配、条目层宏、另起模块三类故意拒；崩溃放量按叫停类型找层；数花括号顺带认块注释；没编进 gpu 那一份 `judge_my_blocks_on_gpu` 进名单整句钉住。
- 已知没做、不在这一轮被判范围的：C762、C554；预演每报一次整串推一遍环，大盘上的时间与内存代价没量；抄的那一份对齐判下一批改成共用。
- 用户已定、报出来不算打中的：tri 的判法本身与它在「不知道」那一支上多护的那一截；显式推环会让回退候选集掉到少于 4 个不同状态；不选乙（推完不抬 F，推环在长写上仍可能不够，只是不再列给用户）；两进程同读一份空闲的竞态；G2 乙。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　第二十块 预演定出路

1. 预演与真结局：有没有一形预演说推完放得下、照推环走完同一个写却被拒，或预演说放不下而推完其实放得下？攻方上一轮那几形（`atk_k1_ways_out_sweep` 9 形、实现员三形长写、两槽 C、环里另有被抛弃根）在新实现上逐格对一遍。
2. 预演不落盘：`DeviceKeepingWritesInMemory` 有没有一条路把写穿到真盘（屏障、写零、越界、对齐、读回自己写过的块的拼接）？预演里出错（读盘错、抬 F 那一串在预演里被拒、取不到落点）时报的是什么，会不会把一个真能放下的改动报成卡死，或把会话与分配器弄动？
3. 预演的代价：每报一次整串推一遍环加这次改动剩下的发布，内存与时间随什么涨？有没有一形报错这一步本身就失败或慢到不可用？

### K2　第二十块 出路措辞、冻结与无隔离

1. 两种 R 的出路一说法、出路二的次数与槽数、N 与推完隔离降了多少：有没有对不上的形（多槽 C、C 与被抛弃根重叠、有盘读坏的槽）？
2. 推环入口先判冻结：冻结之外还有没有一种状态让预演过、真发在第一次就被拒？
3. C 记着落点而一个槽都没隔离时的两处处置，有没有该报宽护而被这一判吞掉的形？

### K3　判别力

第二十块追加 25 行、整行替换 6 行，第二十一块追加 13 行：能不能拦住改回去？注入级用例的那几形，在自然历史里存不存在、用例钉的是不是条款说的那件事？还有哪几种改坏的写法照绿？

### K4　第二十一块 扫源码

1. 判绿的写法里有没有真能遮住钉住路径、绕过叫停的（`use … as`、两层花括号、`self` 组、`extern "C"`）？
2. 新的找层与数 `let` 的判法、块注释与原始字符串的处理，还有哪几种写法骗得过、哪几种合法产品写法会被判红？

### K5　抄的那一份对齐判

`DeviceKeepingWritesInMemory::check_aligned_and_in_range` 与 `crates/singlefs-core/src/block_device.rs` 文件后端那一份有没有不同；不同会让预演与真发在哪一形上分叉？

## 四、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。读码推出的「没打中」尽量换成造出来的反例尝试。
- 本机内存 60G、几条线同时在跑：每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 12G 的上限，同时起的件数乘上限不超过 12G；编出来的测试二进制也经它跑，不直接执行。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试。
