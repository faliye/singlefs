# 根环实现第 10z 批第一块：代码轮第一轮正文（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 K4 G1 P1 P2 -->

## 一、题面

这一轮攻第 10z 批第一块（显存账）。它修第 10y 批代码轮判决 `research/prompts/m3-batch10y-code-r1-main-verification.md` 第四节第 1 条那几项。第 10z 批第二块（抬 F 照记着的落点，`crates/singlefs-core/src/mount.rs`、`allocator.rs`）要等删除线第 35 批 W 合入之后才开工，文件与这一块不相交，另开一轮。

GPU 用在崩溃放量与对拍上的总口径在 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md`（D24（后台重活能不能卸给 GPU））。

用户原话：
- 「全局的任务仅仅管会不会对指定的电脑造成cpu或者gpu的oom」；
- 「不压并发」；
- 2026-10-07 定 G1 二：不做跨进程账，每次借现读空闲、只记本进程借的，两个进程的竞态靠运行时退 245、报本次未跑兜底（岔路单 `research/prompts/m3-batch10t-code-r1-forks.md`）。

改动全文：`research/prompts/_m3-batch10z-a-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-r1/batch10z-a/patch/report.md`（第二节是它自定的取法）。

| # | 改动 | 落点 |
|---|---|---|
| A | K1-1：不走卡池的设备先记总线号再起（`BusesWithDevicesStartedOutsideTheBooks`）；分不出驱动实占时按 0 算（乙附）；销了账的那一份照早先读数见到的压（丙形） | `crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs` |
| B | K1-3：单元校验和对拍每条核对线程各自接住 Out of Memory（`CheckingTally::stopped_because_the_gpu_memory_was_short`）；Out of Memory 按线程认（`WorkOnTheCardOfThisThread`，编译线程按「这一刻在这台设备上编内核的活」认）；设备丢了重起时 `prepare_the_driver_share_before_starting_the_device` 重新登记驱动那一份 | `gpu_memory_ledger.rs`、`gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs` |
| C | K2-2：P1 早先读数见到的那一份与那一刻活着的借据号；P2 一张卡 Out of Memory 只停那一张、那一块交回队里（`BlocksForTheCards::give_back`），收工后别的卡判完了照常交回 | `gpu_memory_ledger.rs`、`crash_amplification.rs` |
| D | K3：照绿 6 条与 7 处补用例和变异；删一行旧变异 | 三份 src 的库单测、`crates/mutations.tsv` |

## 二、实现今天的样子（主 agent 的观测，2026-10-07）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-a-code-r1/tree/`，取在 10z 第一块合入之后（底含删除线 S、U、T；W、V 没合）；被判文件 sha256 在 `research/prompts/m3-batch10z-a-code-r1-snapshot/crates-sha256.txt`（324 份）。
- 合后验证（`~/.local/share/singlefs/work/m3-r1/merge-10z-a/verify-summary.txt`）：harness 档 lib `140 passed`；checker 档 `--lib` `215 passed`、扫源码 `13 passed`、产品路径 `2 passed`、pipeline `13 passed`；fmt、clippy 退 0；清场后 9 份 GPU 测试 8 份全绿，`NOT_RUN_NO_GPU_ADAPTER` 0，「显存不够」0；对拍那份 53 条红，与 10y 合后逐条相同，是删除线 T 的 F2 只在 CPU 侧（删除线二分坐实，修在它第 36 批 V），不算这一块。
- 变异：harness 档 23 行实现员证红；checker 档 8 行不要卡的主 agent 证红 `✓ 点名 8 条：跑了 8 条，跳过 0 条，跑的都抓到了`。
- 实现员自定、被攻过零轮的取法（报告第二节第 1–5 条）：乙附在不按进程报占用的驱动上多扣一份 2560 MiB；P1 假设一次读数见到的那一份在那几笔借据销账之前不变小；重起时旧设备那一份 D 记进「销了账而卡上可能还没还的」；Out of Memory 归属的残差（没走那一层的线程恰在编内核时报）；P2 交回的块在别的卡都收工后没人领、报本次未跑。
- 已知没做、不在这一轮被判范围的（腿报出来不算打中）：
  - 真卡上的 Out of Memory、真丢设备造不出：注入开关要加在 `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs`，那份在删除线第 36 批 V 手上；
  - `usable_gpu_cards` / `gpu_cards_by_priority` 两处传 `OutsideTheCardPool` 的实参、`start_the_card` 编管线时的 Out of Memory 范围、设备丢失回调、`card_lent_for_the_booking` 那一支没有变异行；
  - 驱动固定开销 2560 MiB 没上卡量；单子外的 9 处过时注释。
- 用户已定、报出来不算打中的：两个进程同时读到同一份空闲的竞态，以及一份 `nvidia-smi -q` 报告里空闲与进程表两次查询之间约 1.3 毫秒的窗口，由运行时兜底。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　第 10y 批判决第四节第 1 条各项做到没有
1. K1-1：不走卡池的设备先记后起，起不来照样记着，会不会让卡池在那一条总线上一直少借或多借？乙附、丙形两种取法下，账给的量是不是都不大于此刻真能用的？
2. K1-3：单元校验和对拍几条线程里任一条报 Out of Memory，这一块与之后的块结局对不对（不入库、本次未跑，判完的块照写）？按线程认之后，有没有一形别的线程的 Out of Memory 仍会把这条线程真正的 panic 说成显存不够，或者反过来，真的显存不够被当成 panic 判红？设备丢了重起，账算得对吗？
3. K2-2：P1 那组数之外，长借据、短借据交替、多张卡时，销了账的那一份会不会压过头或压不够？P2 交回的块会不会判两遍、漏判、或者判完的块没入库？

### K2　只防 OOM、不压并发两头（新代码引进来的）
1. 实现员自定的五条取法（第二节那一条列的），各有没有一形让账给的量大于此刻真能用的？
2. 有没有一形，卡放得下却报本次未跑、判红，或者整趟弃卡？

### K3　测试判别力
新加的单测与变异行，能不能拦住这一批每一处被改回去？有没有一形「改回去」照绿？没补行的几处里，有没有不用真卡也能补的？

## 四、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
