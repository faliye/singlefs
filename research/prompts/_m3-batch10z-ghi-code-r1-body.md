# 根环实现第 10z 批第七、八、九块：代码轮第一轮正文（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 G2 H1 H2 H3 H4 F-K1 F-K3 F-H3 F-H4 P3 P5 X1 X7 R C P -->

## 一、题面

这一轮攻三块，碰的文件两两不交：

| 块 | 照什么改 | 落点 |
|---|---|---|
| 第七块 | 第六块合入后验证红的那一条：`next_card_to_start` 不管有没有下一张卡都先问叫停，改成叫停参数是 `impl FnOnce() -> StopOfTheCards`、找到下一张装了状态的卡才问；`print_the_not_run_line_or_refuse` 拆出不打印的 `not_run_line_or_refusal_of_a_lease`，库单测改调它 | `crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-harness/tests/gpu_card_rounds_remember_a_stop_request_removed_mid_run.rs` |
| 第八块 | 第六块代码轮判决 `research/prompts/m3-batch10z-f-code-r1-main-verification.md` 第三节：K1-2 借之前对有没等成借据的卡 `Poll` 一次、`QueueEmpty` 就销，另带 F-K1；K1-3 改形，按丢了的设备每个 `GpuCard` 克隆都放掉才算放掉；H3 抽成 `wait_for_the_device_and_take_in_the_outcome`、展开中不再抛；K4 碰卡用例照 F-H4 改、各项补用例与变异；串里落单的花括号 11 处改成 `\u{7b}` / `\u{7d}` | `gpu_unit_checks.rs`、`gpu_memory_ledger.rs`、`crash_amplification.rs`、两份 GPU 测试 |
| 第九块 | 第四块代码轮判决 `research/prompts/m3-batch10z-d-code-r1-main-verification.md` 第二、三节与用户定案：H2 丙（`placements_in_the_accounts_of_newer_readable_roots` 记录齐那一支交挂载那一刻现行账里仍分配着的落点）；K1-1 新公开 `ring_slots_whose_reread_still_bad_refuses_the_ceiling_by`，`on_device_modes.rs`、e158 改取它；K1-2 空条目照 `Some(空)` 算；K1-3 e158 加阳性对照；K3 不空闲核全部槽、H2 端到端期望独立算 | `crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-checker-tier/src/on_device_modes.rs`、`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs`、`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` |

条款与用户原话：
- G2 岔路单 `research/prompts/m3-batch10z-a-code-r1-forks.md` G2 行（用户 2026-10-08 定「改用乙」）；用户原话「全局的任务仅仅管会不会对指定的电脑造成cpu或者gpu的oom」「不压并发」；GPU 总口径 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md`（D24（后台重活能不能卸给 GPU））。
- `.claude/kb/decisions/16-发布语义.md` 已定项 1 第 37 行「根槽这一次读坏」那一行（H1、H2 与用户 2026-10-08 定的丙）；`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14 第 390、392 行；`.claude/kb/decisions/28-挂载期承诺量.md` 已定项 1 第九项；`.claude/kb/invariants.md` 第 58 行 I-7.4（近 K 代块未被复用）。

改动全文：`research/prompts/_m3-batch10z-ghi-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-r1/batch10z-g/patch/report.md`、`…/batch10z-h/patch/report.md`、`…/batch10z-i/patch/report.md`。

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-ghi-code-r1/tree/`，取在三块都合入之后；被判文件 sha256 在 `research/prompts/m3-batch10z-ghi-code-r1-snapshot/crates-sha256.txt`。底里另有删除线第 45 批、形式化线与删除线当天改的几处装置测试钉值（E165、E168、`new_pool_file_creation_on_device`），不在这一轮被判范围。
- 合后验证：
  - 三块各自合入时 harness 档 186 个测试二进制全绿、fmt、clippy 过；
  - mutation-tables 格 0 行筛不到，static-check `CHECKED 3405 322 10`；
  - checker 档带 gpu 特性 lib 与 12 份挂 gpu 的测试全过，`NOT_RUN_NO_GPU_ADAPTER` 0 行；
  - 拷贝里把借据丢掉时的等改回 Poll，碰卡用例红（改之前照绿）；
  - e158 内联单测 `95 passed`；E171 装置 sha 前 12 位 ab35105f861d；
  - checker 档快档库与 `tests/` 全绿。
- 实现员自定、被攻过零轮的：
  - 第八块第 1 条：借之前 `Poll` 一次的出口；
  - 第八块第 2 条：按 `GpuCard` 克隆都放掉判丢了的设备放掉；
  - 第八块：借之前 poll 要拿池里那一格的锁，卡在起设备时这次借等它起完；
  - 第八块：「wgpu 句柄不从 `GpuCard` 克隆另拿出来、活得更久」这一前提今天没有会红的检查；
  - 第九块：丙的落法与 D28 第九项计数变化（挂载那一刻 18 槽不变，推过第一串抬 F 之后 22 → 46 槽）；
  - 第九块 K1-2：空条目照 `Some(空)` 算——空条目一个来源是坏盘记录，这一来源上照空串算比照读盘少护，条款没区分来源；
  - 第七块：本次未跑那一行的打印用扫源码用例配变异（python 模拟过、不算证红）。
- 已知没做、不在这一轮被判范围的：`crates/singlefs-checker-tier/tests/admission_raises_the_floor_before_refusing.rs` 第 2268 行仍按旧口径传整集（池里没有被抛弃根，结局不变）；丙那一形的崩溃枚举没做；设备丢了、账还没被问到那一下读数多给（攻方模型 `over=6960`，借之前必经重起判）。
- 用户已定、报出来不算打中的：两进程同读一份空闲的竞态、一份 `nvidia-smi -q` 报告里约 1.3 毫秒的窗口；G2 乙本身；H2 丙本身（只攻它的实现与没写到的边角）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　第八块：G2 乙的补法做到没有
1. 等超时那一笔：活做完之后是不是总能销？有没有一形借之前那一下 `Poll` 让账多给，或拿锁把并发压住？
2. 丢了的设备：按 `GpuCard` 克隆都放掉判，有没有一形旧设备还占着卡却判成放掉（多给），或永远判不成放掉（少给出不来）？
3. 展开中不再抛：有没有一形把真错吞掉、让判红变绿？

### K2　第九块：H2 丙与第四块判决那几条做到没有
1. 丙罩不罩得住被抛弃根 R 引用的全部单元？R 引用、却既不是 R 自己写的、也不在挂载那一刻现行账里仍分配着的单元有没有？
2. 丙多护的那一截什么时候放？会不会一直不放、把空间占满而准入白拒？D28 第九项计数变了之后，准入与卸载有没有一形判错？
3. K1-1 新口径：公开函数、`on_device_modes.rs`、e158 改取之后，有效根槽读坏是不是照拒？
4. K1-2 空条目来自坏盘记录时，有没有一形让 I-7.4 红？

### K3　第七块
1. 叫停只在有下一张装了状态的卡时才问：有没有一形该看叫停时没看、叫停之后还起新卡领块？
2. 不打印那一半拆出去之后，产品路径上该打的本次未跑那一行是不是每一处都还打？

### K4　只防 OOM、不压并发两头（新代码引进来的）
1. 有没有一形账给的量大于此刻真能用的？
2. 除用户已认的之外，有没有一形卡放得下却报本次未跑、判红、整趟弃卡，或并发被压住？

### K5　测试判别力
三块新加的用例与变异行，能不能拦住每一处被改回去？有没有一形「改回去」照绿？

## 四、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
