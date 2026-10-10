# 根环实现第 10t–10w 批：代码轮第一轮正文（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K2-1 K4-1 K4-2 K4-3 K4-4 -->

## 一、题面

这一轮攻根环线在第 10o 批代码轮判决之后合进主工作区的四批。判决是 `research/prompts/m3-batch10o-code-r1-main-verification.md`，第三节列了下一批，第五节是用户定案；岔路单 `research/prompts/m3-batch10o-code-r1-forks.md`。

用户原话（2026-10-06）：「全局的任务仅仅管会不会对指定的电脑造成cpu或者gpu的oom」「GPU和CPU的使用只要 不要终止其他的任务都没有问题」「不压并发」。

改动全文：`research/prompts/_m3-batch10t-code-r1-diff.md`，含四份补丁、主 agent 合入时改的钉值与 `image.rs` 一句文档。

| # | 批 | 改动 | 主要落点 | 压着的定案 | 实现员报告 |
|---|---|---|---|---|---|
| A | 10t | 判决 K4 四格：<br>- 容量三者取最小；<br>- 判活看 `/proc/<pid>/stat` 状态位，首次起池驱动开销记一份；<br>- 每次起池现读空闲，估计夹进 [现读, 现读 + 活进程已记]，后开账改小估计时先起的跟着变小；<br>- 不再用 2048 MiB 滤卡，第一次借到才起设备（先登记后建设备）；<br>- 产品路径在起过卡池的进程里用池里的卡；<br>- 扫源码检查改词法剥离 | `crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crash_amplification.rs`、两份测试 | 判决第三节第 1–4 条 | `~/.local/share/singlefs/work/m3-r1/batch10t/patch/report.md` |
| B | 10u | K2-1 甲，I-3.14 第五形：链指针记录有下一片而位置指针头部有一段不是第一版的取值，这一片解不开。<br>- CPU 经 `walk.rs` 的 `chain_record_view`；<br>- GPU 经 `crash_judge_checker_common.wgsl` 的 `w_chain_record_kind`，恢复内核读实例表同判；<br>- 坏镜像两张，对拍一条流 | `crates/singlefs-checker/src/walk.rs`、三份 wgsl、`checker_known_bad_images.rs`、对拍文件、`image.rs` 一句文档 | 岔路单 K2-1「甲」；`.claude/kb/invariants.md` I-3.14 行与 D18 已定项 11（已写回） | `~/.local/share/singlefs/work/m3-r1/batch10u/patch/report.md` |
| C | 10v | 起过卡池的放量产品路径，第 ① 段按装的状态数借草稿区，等满报本次未跑；没配显卡优先级时 3/4 那一份并进最小值 | 同 A 三份 | 10t 报告「要你定的」第 1、2 条，主 agent 定 | `~/.local/share/singlefs/work/m3-r1/batch10v/patch/report.md` |
| D | 10w | 五条：<br>- 接线有会红的有卡用例；<br>- 借不到的卡不装状态、重排；<br>- 第 ② 段与单元校验和缓冲进账；<br>- 只排计划那一趟借不到报本次未跑（Rust 一半）；<br>- 估计变小时在账本锁里重压 | 同 A 三份 + `crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs` | 10v 报告「停下交主 agent」第 1–5 条，主 agent 定 | `~/.local/share/singlefs/work/m3-r1/batch10w/patch/report.md` |

## 二、实现今天的样子（主 agent 的观测，2026-10-07）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10t-code-r1/tree/`，取在 10w 合入之后；被判文件 sha256 在 `research/prompts/m3-batch10t-code-r1-snapshot/crates-sha256.txt`。
- 合入验证（照抄判定行）：
  - 10t 在删除线第 32 批之后重跑：checker 档 `196 passed`、`13 passed`、`5 passed`；清场后 8 份 GPU 测试全绿，对拍 `141 passed; 0 failed; 2 ignored`；`NOT_RUN_NO_GPU_ADAPTER` 0，显存 OOM 0。
  - 10u、10v：harness 档 `109 passed`、`190 passed`、`9 passed`；checker 档 `196 passed`、`13 passed`、`5 passed`；8 份 GPU 测试全绿，对拍 `142 passed`；三个内核展开：恢复 319137、扫描 381545、走读 329390。
  - 10w：见调度记录「10w 合后验证」那一行。
  - 变异：10t checker 档 4 行与 10u 5 行 `✓ 点名 6 条 … 都抓到了`、`✓ 点名 3 条 … 都抓到了`；10v 7 行、10t harness 8 行由实现员证过。
- 主 agent 定的零轮设计（腿判对不对）：
  - 10t 报告第 4 条：外人在两次开账之间占显存、同时有借了还没建缓冲的借据时，估计会偏大，偏大的量不超过两者里小的那个；
  - 10t 报告第 5 条：卡起不来挪到第一次借时判红；
  - 10u 报告第九节第 1 条：第五形那条指针不跟随，所以 I-2.4 不判；
  - 10w 报告第 2 条：取卡登记也照「借不到的卡不装状态」办。
- 已知没做、不在这一轮被判范围的（腿报出来不算打中）：
  - 10u 头部另三段（算法类型、压缩算法码、压后长度）没有坏镜像与变异；
  - 10w：
    - 对拍那一路不重压，要动 `crash_judge_dispatch.rs`；
    - 第 ② 段布局常数抄了 `crash_verify_gpu.rs` 的私有常数；
    - 第 ② 段借卡候选是池里全部卡；
    - `book_on_the_card_waiting_for_room` 只剩测试调；
  - 分项驱动认 `NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment` 那一半，归工具批 t6；
  - 驱动固定开销 2560 MiB 没上卡量（10r 报告 Q6）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　显存账会不会让一台机器 OOM（只防 OOM 那一头）
1. 有没有一形，账给出的容量大于卡上此刻真能用的？看这几样：
   - 估计的夹法 [现读, 现读 + 活进程已记]；
   - 活进程「已记而还没实际占」的那部分；
   - 外人在两次开账之间来去；
   - 第 ② 段与单元校验和的缓冲；
   - 当场起设备的驱动开销。
2. 先登记、后建设备这条，在产品路径、第 ② 段、取卡登记三处都成立吗？
3. 重压与借在账本锁里一气做完，有没有一形在锁外读了估计、在锁里借？

### K2　显存账会不会白白压并发（不压并发那一头）
1. 有没有一形，卡上放得下、账却让它等满 30 分钟报本次未跑，或者判红？
2. 「借不到的卡不装状态、重排到借到的卡上」有没有一形，把本可以分到几张卡的状态全压到一张上，卡空着不用？
3. 僵尸判死、驱动开销只记一份，有没有让账算重或算漏的形状？

### K3　I-3.14 第五形在各读法之间是不是同一个（B）
1. 五段头部判法（MAC / nonce、算法类型、压缩算法码、压后长度、extent 偏移），CPU `chain_record_view`、GPU `w_chain_record_kind`、恢复内核三处与实现 `crates/singlefs-core/src/instance_table.rs` 第 190–228 行逐段一致吗？
2. 解不开之后「行不收、链不往下走、I-9.29 门关」三件事，第五形在每处都做到了吗？
3. I-2.4 不判这条读法，与 `.claude/kb/invariants.md` I-2.4 行站不站得住？

### K4　测试与扫源码的判别力（A、D）
1. 新的有卡用例，能不能分出「产品路径借了」与「没借」？
2. 扫源码检查改成词法剥离之后，还有哪种写法绕得过？
3. 钉值、变异行能不能拦住这几批里任何一处被改回去？

## 四、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
