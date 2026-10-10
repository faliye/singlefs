# 根环实现第 10o–10s 批：代码轮第一轮正文（2026-10-06）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 T1 T2 S1 Q2 Q3 Q5 Q6 -->

## 一、题面

这一轮攻根环线 2026-10-06 合进主工作区的五批改动，连同 #9 钉值重取：

- 第 10i 批代码轮第一轮判决：`research/prompts/m3-batch10i-code-r1-main-verification.md`。
- 用户定案：同一份判决第六节；岔路单 `research/prompts/m3-batch10i-code-r1-forks.md`。T1 定甲；T2 照今天拒，下一轮先量。
- 用户 2026-10-06 定 I-3.14 另立：`.claude/kb/decisions-history.md` D18 节 2026-10-06 块。
- 用户关于 GPU 与资源的原话：「全局的任务仅仅管会不会对指定的电脑造成cpu或者gpu的oom」「GPU和CPU的使用只要 不要终止其他的任务都没有问题」「不压并发」。

改动全文分两份：
- 10o、10p：`research/prompts/_m3-batch10o-code-r1-diff.md`。
- 10q、10r、10s 与 #9：`research/prompts/_m3-batch10o-code-r1-diff-2.md`。

| # | 批 | 改动 | 主要落点 | 压着的条款或定案 | 实现员报告 |
|---|---|---|---|---|---|
| A | 10p（= 10l + 10m 重做） | T1 甲：根环表 `RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses` 记整条 `RootRecord`；抬 F 那一串（准入上限、影子账、回收门槛）读这一槽读坏、重读仍坏时，拿记着的那条根顶上。有效根照它本身进上限与候选集；被抛弃根照它记着的落点进影子账、不进上限。回退与 F_生效 照旧拒（T2） | `crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` | `.claude/kb/decisions/16-发布语义.md` 已定项 1 第 37 行（「根槽这一次读坏」，2026-10-06 已写回新类）；10i 判决 K1、K2 两格 | `~/.local/share/singlefs/root-ring-backup/batch10p/report.md` |
| B | 10o（= 10j + 10n 重做，含 10k） | I-3.14（实例表每片记录形态合法），四形当这一片解不开：kind 不是 0 或 1、链指针记录不在最后一条、最后一条不是合法链指针记录、行 flags 非 0。行不收、链不往下走、I-9.29 门关。只读读法与引用集合那一遍（CPU 与 GPU）同口径；H-b「走过」的键按读回、自证过的那一份；`IMPLEMENTED_INVARIANTS` 84 条；GPU 走读内核判 I-3.14，覆盖表补下标 83 | `crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crash_judge_checker_scan.wgsl`、`crash_judge_recovery.wgsl`、`crash_judge_gpu.rs`、`crash_injection.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs` 等 | `.claude/kb/decisions/18-块里携带什么信息.md` 已定项 11（kind 那一句）；`.claude/kb/invariants.md` I-3.14 行、I-3.8 行 | `~/.local/share/singlefs/root-ring-backup/batch10o/report.md` |
| C | 10q | 扫描内核 7 个函数的多处调用并成一处，展开 692201 → 378232（预算 400000）；判定次序与副作用不变（实现员逐函数读码核） | `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` | `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` 的 `LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL` 与拒绝那一段 | `~/.local/share/singlefs/root-ring-backup/batch10q/patch/report.md` |
| D | 10r | GPU 对拍按量借卡：卡池照 `GPU_CARDS_BY_PRIORITY`、每卡一份账、草稿区按 `state_count` 封顶、每卡一个设备、内核按「设备 + 三份源码摘要」只编一次、驱动 2560 MiB 每卡记一份、先填优先的卡 | `crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crash_judge_dispatch.rs`、`crates/singlefs-harness/src/gpu_memory_ledger.rs`、对拍文件 | `.claude/singlefs-ai-sop/rules/test-script-discipline.md`「用上现有的硬件加速」；用户原话 | `~/.local/share/singlefs/root-ring-backup/batch10r/patch/report.md` |
| E | 10s | 五样：<br>① 扫描内核调用点数与三个内核展开余量的库单测；<br>② 卡池配置读法挪进 `gpu_card_configuration.rs`（只开 `gpu` 也编得进）；<br>③ 另 7 份 GPU 测试改走卡池，扫源码检查纳进来；<br>④ `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES` 当每张卡账的上限（取它与清场后空闲减驱动开销两者的较小值）；<br>⑤ 账做成跨进程：按卡一份账本、`flock`、放工作根，按进程号与启动时刻判活，死进程那份当没记，等满上限 30 分钟 | `crash_judge_dispatch.rs`、`gpu_card_configuration.rs`、`gpu_unit_checks.rs`、`crash_amplification.rs`、`gpu_memory_ledger.rs`、7 份测试文件 | ④⑤ 是主 agent 照用户原话替实现员定的设计（零轮） | `~/.local/share/singlefs/work/m3-r1/batch10s/patch/report.md` |
| F | #9 钉值重取 | `crash_enumeration_sharded_across_processes.rs` 打印行钉值 `4fc9cde7…` → `4b8b1e84…`，进度文件内容 `11977571…` → `1d92a7ad…`，来历注释补一句 | `crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs` | 主 agent 核：新 58 行删掉两趟 `GOLDEN_TALLY` 各一项 ` I-3.14=46/0/0` 之后整段 SHA-256 等于旧钉值 | 调度记录「第 10s 批交回」那一行 |

## 二、实现今天的样子（主 agent 的观测，2026-10-06）

- 冻结副本：`~/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/`。取自删除与并发那条线第 31 批合入之后的主工作区，被判文件的 sha256 在 `research/prompts/m3-batch10o-code-r1-snapshot/crates-sha256.txt`。
- 合入验证（照抄日志里的判定行）：
  - 10q、10r：harness 档 `--lib` `test result: ok. 90 passed; 0 failed; 1 ignored`；checker 档 `--release --features gpu,verdict-store --lib` `test result: ok. 194 passed; 0 failed; 2 ignored`；fmt、clippy 退 0。
  - 10s 与 #9：harness 档 `96 passed`；checker 档 `--lib` 与扫源码、#9 两份分别 `196 passed`、`2 passed`、`10 passed`；三种特性 `--no-run` 退 0；清场后 8 份 GPU 测试里，对拍那份 `127 passed; 3 failed`，其余 7 份全绿，`NOT_RUN_NO_GPU_ADAPTER` 0、显存 OOM 0。
  - 对拍红的 3 条是删到最后一批那条流：GPU 走读内核读 extent 上段根时少了 CPU `walk.rs` 的零条目豁免，10q 之前就有，删除线第 31 批改了。
  - 10o、10q、10r 落在 checker 档的 21 行变异全部证红：`✓ 点名 21 条：跑了 21 条，跳过 0 条，跑的都抓到了`。
- GPU 与 CPU 二分：上一趟对拍红的 I-5.4 两条，是编译目录里 CPU 一侧用了第 36 批之前的 `walk.rs`。`rsync -a` 保留了旧 mtime，cargo 认它已编。重新编过之后两条都过。报告 `~/.local/share/singlefs/root-ring-backup/gpu-cpu-disagreement/report.md`。
- 主 agent 认的读法与零轮设计（腿判对不对）：
  - D16 第 37 行写回时，把新类算进「这个进程知道住着根的槽」，依据 `crates/singlefs-core/src/allocator.rs` 第 1090–1108 行 `ring_slots_known_to_hold_a_root`。
  - 10s ④⑤ 两处设计。
  - 10s 实现员先定的四处跨进程账细节（报告 S2），主 agent 认了：
    - 后起的进程照账本记的清场后空闲算容量；
    - 别的活进程各记 2560 MiB 驱动开销；
    - 等满了打 `NOT_RUN_NO_GPU_ADAPTER`；
    - 旧方法 `lease_card_waiting_for_room` 等满判红。
- 已知没做、不在这一轮被判范围的（腿报出来不算打中）：
  - 10r 报告 Q6：驱动固定开销 2560 MiB 没上卡量。
  - 10s 报告 S1：对拍文件里设了额度变量就 `assert!` 那一支，归删除线第 32 批删。
  - 走读内核钉值随删除线第 31 批由 323759 变 326147，归删除线第 32 批改。
  - F-零槽：GPU `read_at` 对没写过的单元区槽交回全零切片，归删除线第 32 批。
  - C741：带一次覆盖写的层 0。
  - T2「回退不拒」那一臂。这一轮要量，见 K1 第 3 条，量出的数交用户，不算打中。
- 用户已认的代价（腿报出来不算打中，只核「代价是不是只有这么大」）：
  - 读坏槽与读回全零的槽把回收门槛钉在它可能住过的最旧 txg，I-3.1 在合法盘面上红在记账多的一边，到钉解开为止（`.claude/kb/invariants.md` I-3.1 行）。
  - 无故障历史也走得到全零钉（I-3.1 回退线索调查员，`~/.local/share/singlefs/root-ring-backup/i31-after-rollback/report.md`），同属这一条。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　T1 甲撑不撑得住（A）

1. 抬 F 那一串拿记着的根顶上时，有没有一形让：
   - 回收门槛越过盘上真住着、可回退到的有效根；
   - 被抛弃的根掉出影子账（10i 判决 K1-1 那一形，I-7.4 红）；
   - 准入上限被一条已经不在盘上的旧记录卡住（K1-2 那一形）。
2. 「写掉那一槽就换类」在每条写路径上都成立吗？有没有写了那一槽、记录还留着，或者没写就被清掉的路径？「记整条根记录」在多盘、整环重读、记根时重读三处是不是同一份？
3. T2「回退不拒」那一臂：在 T1 甲的实现上加一条臂，回退判候选、算 F_生效 时也拿记着的根顶上。在「抬 F 照钉住推过之后再回退」那组历史上，两臂各量回退结局与 checker 红。量到的数交用户，不算打中。
4. 下一轮的攻击面（10i 判决「下一轮」那几行）：
   - K1-d 留洞的环：环上有洞时，新类与洞那一槽里留着的上一圈的根并存；
   - H-b 下的 I-5.1：10k 修后 I-5.1 仍红那张坏镜像。主 agent 判它是那张镜像自身的毛病，推的，要核。

### K2　I-3.14 的口径在各读法之间是不是同一个（B）

1. 四形判「这一片解不开」，在四处读法里逐形都一样吗？四处是主走读、只读读法、引用集合那一遍（CPU）、GPU 走读、扫描、恢复内核。
2. 解不开之后「行不收、链不往下走、I-9.29 门关」三件事，在每处都做到了吗？有没有一处照收、照走，或者门没关？
3. H-b「走过」的键按读回、自证过的那一份：有没有一形，链上两片读回同一份而键不同，或者读回不同而键相同，让链被当成走过而少判一片？
4. I-3.8 报不适用的理由与 I-3.14 那一片对不对得上。

### K3　扫描内核单调用点改法有没有改了判定（C）

1. 7 个函数的次序、副作用、提前返回，与并之前逐个对。
2. 上卡对拍：V1（新编 10q+10r）128 条里只红删除那 3 条，V3（不打 10q）同形。这能不能说明 10q 没改判定？哪一类状态是对拍的流没撞到、而改法会让 GPU 与 CPU 不同的？
3. 10s 的调用点单测钉的是今天的数，它能不能拦住「换一种写法又把展开推回超预算」？

### K4　显存账会不会让一台机器 OOM，或者白白压并发（D、E）

1. 账上的容量会不会大于卡上真能用的？容量 = 清场后空闲 − 每卡 2560 MiB，有额度就取较小值。看这几种情况：同一张卡几个设备、种假违例位那条另编一套管线、驱动开销比 2560 大。
2. 跨进程账：
   - `flock` 的范围、读写次序对吗？
   - 判活靠进程号加启动时刻，进程号复用会不会误判？
   - 进程被 SIGKILL 时它那份要等多久才回收？
   - 两个进程同时首次起池时，谁记清场后空闲？
3. 有没有一形，账放得下而实际撞显存（与「只防 OOM」相反）？或者实际放得下而账让它等满 30 分钟、报本次未跑（与「不压并发」相反）？
4. 7 份改走卡池的测试：每条用例借的量与它真开的缓冲对得上吗？扫源码检查能不能被换个写法绕过？

### K5　#9 钉值重取（F）

主 agent 的核法：删两项 I-3.14 之后哈希等于旧钉值。这能不能证明「只多了 I-3.14」？进度文件内容那个值，是在只换了打印行钉值的副本里取的，有没有可能把别的变化一起钉进去？

## 四、交付要求

- 引 kb 条款，写 kb 文件名加那份文件自己的行号；行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
