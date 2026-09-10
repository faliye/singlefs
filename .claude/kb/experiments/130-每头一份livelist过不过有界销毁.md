## E130 每头一份 livelist 过不过有界销毁 —— 已跑（2026-09-10）

**问的是**：D6（快照实现模型） 未定项 2 的候选甲（每头一份 livelist）过不过
D6（快照实现模型） 九条判据第 5 条「**有界销毁**：可分批、可续做，
每批最坏空间需求有界且可在批前预留」。

**为什么要跑**：2026-09-10 那一轮三方把丙与乙判出局、丁的代价重估，只剩甲存活；
而反推腿自己明写「我的结论只到『乙过不了』，**不到『甲过得了』**」——
甲有条目数，可「每批最坏空间需求」的算式与「可 condense」的代价仓里一个数都没有。

**跑前写死**：`research/prompts/e130-preregistration.md`（判据、阈值、作废条款、
以及「结果反过来我接不接受」都写在测量代码之前）。

**复跑命令**（登记成 `exact` 模式，与留存产物逐字节比对）：

```bash
cd research && bash scripts/replay.sh E130
# 直接跑装置：
cd research && cargo run --release --bin e130_livelist_bounded_destroy
```

代码 `research/e7-index-bench/src/bin/e130_livelist_bounded_destroy.rs`，原始输出 `research/results/e130-livelist-bounded-destroy-2026-09-10.out`（一次运行，无参数扫描）。

### 这是计数模型，不是实现——它证明不了什么

没有文件 I/O、没有并发、没有随机源 ⇒ 同一个二进制跑 N 遍必然逐字节一致。
**「N 轮一致」说明的是没有隐藏状态，不是统计上稳定**
（`.claude/singlefs-ai-sop/rules/test-discipline.md`）。
证据强度来自 **21 个单测 + 15 条变异全部被抓**，不来自轮数。

**不回答**：真实 I/O 次数、condense 自身的挂钟代价、崩溃一致性；
「可续做」那半没有建模——它骑在 D8（核心索引结构） 已定项 5 的意图机制上，
本装置一个字都没验。

### 臂

| 代号 | 定义 |
|---|---|
| `naive` | 每次 ALLOC 追加一条、每次 FREE 追加一条，**从不 condense** |
| `condense` | 每次 ALLOC 追加一条、每次 FREE 追加一条，且结构条目数 > 2 × 净活块数时抵消一次，只留净 ALLOC |
| `no_structure` | 乙 的形态，**对照臂**（2026-09-10 已出局，放进来是给「甲多付了什么」一个减数）|

⚠️ **`condense` 的触发判据改过一次，改在任何测量跑起来之前**：第一版写「FREE 条目占比 ≥ 1/2」，
而这族负载里 `frees / (allocs + frees) = c / (1 + 2c)` 对任何有限 `c` 都严格小于 1/2
⇒ 前件恒假、该臂一次也不触发，等于 `naive` 的复制品——
`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 点名的**稻草人对照臂**，
也是 `.claude/singlefs-ai-sop/rules/test-discipline.md`「失败条款的前件可以写反」的形态。
是本装置的单测照出来的，不是产物。写反那一版由单测 `the_first_condense_predicate_never_fires` 留档。

### 结论：甲过判据 5，代价两笔

产物逐行抄自 `research/results/e130-livelist-bounded-destroy-2026-09-10.out`。

| 跑前写死的判据 | 产物 | 判 |
|---|---|---|
| 1. 每批最坏空间需求只随批大小走 | `name=criterion1 arm=naive distinct_worst_batch_bytes=1 value=221184`，`condense` 同值，`no_structure` 是 `122880`；三条臂各自扫遍 24 个负载格，取值集合大小都是 1 | **过**。221184 = 4096 × (30 + 24) |
| 2. 批前预留算得出，且不读活树 | 单测 `livelist_pre_reserve_does_not_read_the_live_tree`：净活块数相同、活树规模差一倍的两格，两条 livelist 臂的 `pre_reserve_bytes` 相等，而 `no_structure` 的 `destroy_reads` 不等 | **过** |
| 3. 高估比有没有上界 | `naive n=65536 churn=16` 逐字 `entries=2162688 net_alloc=65536 pre_reserve_bytes=36507222016 true_pending_bytes=2147483648 overestimate=17.0000`；`condense` 同格逐字 `entries=65536 ... overestimate=1.0000` | naive 随 churn 线性发散（C=16 时 17×），**condense 恒 1.0000** |
| 4. 阳性对照 | `name=positive_control arm=naive n=8192 entries_c0=8192 entries_c16=270336 ratio=33` | **不触发作废**，装置分得出差别 |
| 5. condense 之后回不回得到 O(净活块数) | `condense` 在全部 12 个 `share=0.0` 格上 `entries` 逐格等于 `net_alloc`（1024 / 8192 / 65536），与 churn 无关 | **过** |
| 6. 释放路径的额外预付 | 两条 livelist 臂逐格 `free_path_extra_bytes=24`，`no_structure` 逐格 `0` | **> 0**，落 C260（livelist 条目的预付没算进解开自己那个量） |

⇒ **甲过 D6（快照实现模型） 判据 5**，但**只在带 condense 的形态下**：
`naive` 的条目数随写次数无界增长——`n=65536 churn=16` 那格逐字 `entries=2162688`，
而净活块数只有 65536，**33 倍**。

### 两笔要还的账

1. **甲必须带 condense，「可 condense」不是可选项**。没有它，
   条目数与写次数同阶，与活块数无关；判据 5 字面仍然过（上界照样算得出），
   而准入会在盘还很空时就拒绝写入——`n=65536 churn=16` 那格上界报 36507222016 字节（34 GiB），
   真实待删只有 2147483648 字节（2 GiB）。
2. **释放路径每释放一个块多预付 24 字节**（类型标签 2 + 位置条目 14 + birth txg 8）。
   D3（空间分配） 的原则逐字「COW 系统必须保证『释放空间』这个操作本身不需要申请空间」，
   而 D3（空间分配） 已定项 2 的解法是「墓碑就是『解开自己』的一部分，计入那个量」，
   在写入那一刻预付。**livelist 条目要走同一条路，就得把它加进「解开自己需要多少」这个量**，
   而那是 2026-08-31 用户定案的条款 ⇒ 落 C260（livelist 条目的预付没算进解开自己那个量）。

### 变异

15 条，**全部被抓**，表在 `research/mutations/e130_livelist_bounded_destroy.tsv`。

⚠️ **其中一条第一轮没被抓，按 `.claude/rules/mutation-sampling.md` 三分判为第三类
「取样点不敏感」，补了取样点而不是记成等价变异**：M8（condense 判据放宽成恒触发）
只在 `raw / net ∈ (1, 2]` 这一段上与原式不同，而这族负载里
`raw / net = 1 + 2c` 只取得到 1、3、9、33，**取不到那一段**；
`c = 0` 时 `raw` 与 `net` 同值，断言看不见差别。
处置是把判据抽成 `condense_entries(raw, net)` 单独可测，
补三个取样点（150/100、200/100、201/100），M8 当场被抓。
负载族够不到那一段这件事本身由单测 `the_load_family_cannot_reach_the_sensitive_ratio` 留档。

## 历史版本

无。
