# 根环实现第 10z 批第十、十一块：代码轮第一轮正文（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 G2 H2 R C P Q S X1 X2 X4 X4b M1 M2 M6 -->

## 一、题面

这一轮攻两块，碰的文件不交：

| 块 | 照什么改 | 落点 |
|---|---|---|
| 第十块（core） | 第七、八、九块代码轮判决 `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 两行与第四节用户定案「丙继续补洞」：洞一「丙并中间」（走丙那一支的根 R 那一槽并上所选那一版 P 与 R 之间各槽记着的落点）；洞二「回退复活」（R 是回退发布时，靠 R 记录点名的谱系片认出回退目标 R_old，并上 R_old 账里仍分配、P 的账里已释放的落点） | `crates/singlefs-core/src/mount.rs`、`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` |
| 第十一块（GPU） | 同一份判决第二节 K1-1、K4-2、K5 三行：列卡时把照旧记着的借据号一起取走，poll 交回队列空了只销列出来的；`DeviceStartedOnDemand` 拆成起设备的 `starting` 与取留着的 `started` 两把锁；K5 各项补用例与变异 | `crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs` |

条款与用户原话：
- `.claude/kb/decisions/16-发布语义.md` 已定项 1 第 37 行「根槽这一次读坏」那一行（H1、H2 与用户 2026-10-08 定的丙、「丙继续补洞」）；`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14 第 390、392 行；`.claude/kb/decisions/28-挂载期承诺量.md` 已定项 1 第 27 行第九项；`.claude/kb/invariants.md` 第 58 行 I-7.4（近 K 代块未被复用）；回退那次发布追加谱系记录见 `.claude/kb/decisions/18-块里携带什么信息.md` 已定项 24。
- G2 岔路单 `research/prompts/m3-batch10z-a-code-r1-forks.md` G2 行（用户 2026-10-08 定「改用乙」）；用户原话「全局的任务仅仅管会不会对指定的电脑造成cpu或者gpu的oom」「不压并发」；GPU 总口径 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md`（D24（后台重活能不能卸给 GPU））。

改动全文：`research/prompts/_m3-batch10z-jk-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-r1/batch10z-j/patch/report.md`（第十块，含第三节攻方取法在入库代码上的扫、第四节回退复活三臂代价、第五节 D28 第九项读数、第六节停下交主 agent 的六条）、`…/batch10z-k/patch/report.md`（第十一块）。实现员的量法日志在 `~/.local/share/singlefs/work/m3-r1/batch10z-j/runs/`、`keep/` 与 `~/.local/share/singlefs/work/m3-r1/batch10z-k/models/`。

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-jk-code-r1/tree/`，取在两块都合入之后；被判文件 sha256 在 `research/prompts/m3-batch10z-jk-code-r1-snapshot/crates-sha256.txt`。底里另有删除线、形式化线当天合入的改动（`rollback_by_a_forward_publish.rs` 第 1149 行期望 0 → 14、E165、E168 装置），不在这一轮被判范围。
- 第十块关键位置（冻结副本 `crates/singlefs-core/src/mount.rs`）：`placements_in_the_accounts_of_newer_readable_roots` 第 1931 行，走丙那一支的根记在第 2015 行；`PlacementsInTheAccountsOfNewerReadableRoots` 第 2049 行；并中间那几槽的循环第 2073 行；`placements_a_rollback_publish_revived_released_in_the_current_account` 第 2118 行；`rollback_target_txg_named_by_the_records_of_a_publish` 第 2152 行；`placements_still_allocated_in_the_account` 第 2174 行；调用处第 2908、2933 行。新用例在测试文件第 1567、1803 行。
- 第十一块关键位置（冻结副本）：`crates/singlefs-harness/src/gpu_memory_ledger.rs` 第 904、1407、1440、1607、1662、1699、1740 行；`crates/singlefs-checker-tier/src/gpu_unit_checks.rs` 第 1336 行 `take_in_the_devices_found_idle`。
- 合后验证：
  - 第十一块合入（`~/.local/share/singlefs/work/m3-r1/merge-10z-k/summary.txt`）：harness 档 186 个测试二进制全绿、fmt、clippy 过；mutation-tables 格 0 行筛不到，static-check `CHECKED 3416 322 10`；checker 档带 gpu lib 与 12 份挂 gpu 的测试全过，`NOT_RUN_NO_GPU_ADAPTER` 0 行；拷贝里借据丢掉时改回 Poll，碰卡用例照红。
  - 第十块合入（`~/.local/share/singlefs/work/m3-r1/merge-10z-j/summary.txt`）：merge-and-verify rc=0；mutation-tables 格 0 行筛不到点名的测试，static-check `CHECKED 3426 322 10`；e158 内联单测 `95 passed`；E171 装置单编退 0、sha 前 12 位 ab35105f861d；harness 档慢档 `--include-ignored` 三份全绿（`raise_floor_takes_…` 13 passed、`rollback_by_a_forward_publish` 23 passed、`recovery_and_both_mounts_…` 16 passed）；checker 档快档库与 `tests/` 61 份结果 0 失败。
- 实现员自定、被攻过零轮的：
  - 第十块洞一取攻方 `chain` 形：挂载那一刻合成 C 时并，不在分配器解开时并（理由在第十块报告第一节）；
  - 第十块洞二取窄形：认回退靠 R 记录点名的打包单元里、打包记录类型是谱系的那条 rollback_txg = R 的谱系记录；回退目标按最新根的实例表有效、txg = T_old 的根找；并的是 R_old 账里仍分配、P 的账里已释放的全部落点（384 槽盘上多隔离 24 槽，其中 4 槽是复活的数据单元，其余是 R_old 自己的固定点单元）；
  - D28 第九项读数：丙并中间三个读数逐格不变（52 / 76.1 / 80）；回退复活只在 R 是回退时变，挂载那一刻 18 → 42 槽、推过第一串抬 F 46 → 70 槽；
  - 第十一块 K1-1 的列卡带借据号、只销列出来的；前提「借据先于缓冲丢」今天没有会红的检查；
  - 第十一块 K4-2 两把锁；借之前碰上这张卡正在重起时不等、不 poll，这一次不销这张卡；
  - 第十一块 K5：叫停闭包、借之前 poll 那一句用扫源码用例钉（checker 档 4 行变异只在草稿模型上红过）；攻方 M6 判等价，不加用例。
- 已知没做、不在这一轮被判范围的：
  - Q 的记录全读不出、或 Q 只有一条记录而那一条读不出那一形，归 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）；
  - 两条新用例不枚举崩溃状态，那一步层 0 全量超过 2.6 × 10⁸ 个状态，没跑；
  - 1536 槽盘上 34 步之内准入没推出抬 F。
- 用户已定、报出来不算打中的：两进程同读一份空闲的竞态，一份 `nvidia-smi -q` 报告里约 1.3 毫秒的窗口；G2 乙本身；H2 丙本身。
- **交用户的残余，报出来不单算打中，要量**：认不出回退时（R 记录点名的谱系片读不出、根环里读不出 R_old、R_old 的账读不出），补法交空，照丙原样不护、不拒，复活的单元照旧会被发出去（实现员报告第六节第 1 条，`mount.rs` 第 2118 行文档注释末句）。要的是这一形的可达面：哪几种故障组合、哪几种回退目标走得到它，各自红多少。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　洞一 丙并中间

1. 罩不罩得住：(P, R) 之间不止一次发布、中间某一槽的根已不在环里或读不出、R 之后还有较新的根也走丙（或不走丙）时，R 从中间那几次发布继承的单元是不是都在 R 那一槽里？有没有 R 引用、而 C 里哪一槽都没有的单元？
2. 放的时机：并进 R 那一槽的落点什么时候放？有没有一形一直不放，让第九项一直多算、准入白拒？有没有一形放早（R 那一槽写掉之前、R 还在环里时就回收）？

### K2　洞二 回退复活

1. 认回退：谱系片读坏、校验和对不上、R 记录点名的谱系片里有不止一条谱系记录、R 不是回退却点名了更早一次回退的谱系片、回退目标不在一圈窗口里（`rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate` 那一形）时，认得对不对？有没有认错（并进不相干的落点）或认漏？
2. 罩不罩得住：R 复活的单元是不是都在「R_old 账里仍分配、P 的账里已释放」那一截里？回退到更早一版、中间隔了几次覆写、或 R_old 本身也是一次回退时呢？
3. 认不出回退那一形（第二节「交用户的残余」）：可达面多大？在入库代码上能造出的形各红多少？有没有一条读得出的信息（不改格式）能让它认得出？
4. 多护的那一截（R_old 自己的固定点单元）什么时候放？会不会一直不放、把空间占满而准入白拒？D28 第九项计数变了之后，准入与卸载有没有一形判错？

### K3　第十一块 K1-1 列卡带借据号

1. 列卡之后才记成没等成的那一笔，会不会在下一次列卡、poll 交回队列空了之前被销？「借据先于缓冲丢」那一前提不成立时，有没有一形多给？
2. 列出来的那几笔、poll 的那一台、重起换下来的那一台三者不是同一台时，有没有多销或永远销不掉？

### K4　第十一块 K4-2 拆锁

1. 两把锁有没有一形死锁、或同一张卡上两条线程各起一台设备？
2. 借之前碰上正在重起时不等、不 poll：有没有一形账多给？有没有一形把这张卡上本该销的永远销不掉（少给出不来）？
3. 换下来的旧设备在放开 `started` 之后才丢：丢它时 wgpu 要等设备空了，会不会压住别的线程的提交？

### K5　测试判别力

两块新加的用例与变异行，能不能拦住每一处被改回去？有没有一形「改回去」照绿？特别是：
- 第十块 10 行变异里只证了 4 行，其余 6 行读码判；
- 第十一块 checker 档 4 行（X1、X2、X4、X4b）只在草稿扫源码模型上红过；
- `a_second_start_on_the_same_card_waits_for_the_restart_in_progress_and_takes_its_device` 的变异要第二条线程 500 毫秒之内被排上才红。

### K6　只防 OOM、不压并发两头（第十一块新代码引进来的）

1. 有没有一形账给的量大于此刻真能用的？
2. 除用户已认的之外，有没有一形卡放得下却报本次未跑、判红、整趟弃卡，或并发被压住？

## 四、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试。
