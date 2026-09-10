## E129 小于 io_min 的写在真设备上怎么出事 —— 已跑（2026-09-10，三段真设备，5 轮逐格一致，9 条变异全抓）

**它答两件事**，判据、阈值、作废条款写在 `research/prompts/e129-preregistration.md`，先于任何一行 harness 代码：

1. D2（RAID 条带策略）「写的粒度」那一节末尾逐字「⚠️ 本机无法用故障注入证伪（`dm-flakey` 造不出撕裂），
   按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 记为**机制推导**，证伪等崩溃点重放 harness。」
   ——**这句话今天还成不成立**。
2. C212（固定结构的写小于 io_min） 候选 ② 逐字欠的那条依据：「那要给出为什么固定结构不怕设备内 RMW，
   而 E34（根环槽几何） 的真设备那一轮**只证到 RAID5 写洞这一条机制**」——**除了写洞，还有没有第二条机制**。

### 结论一：位置确定的撕裂注入器造得出，5/5 轮

构造是 `dm-linear` 把一台设备拼成两段：前段落在正常 loop 上、后段是 `dm-error`；
对**跨边界**的一次 O_DIRECT 写，三件事同时成立。产物 `e129-tear-injector-2026-09-10.out` 逐字：

```
E7RESULT name=span round=1 write_rc=1 prefix_persisted=1 suffix_absent=1
E7RESULT name=poscontrol round=1 write_rc=0 all_persisted=1
E7RESULT name=negcontrol round=1 write_rc=1 nothing_persisted=1
E7RESULT name=verdict poscontrol_bad=0 negcontrol_bad=0 span_torn_rounds=5/5
```

⇒ **D2（RAID 条带策略） 那句「本机无法用故障注入证伪」不成立**：换一个构造就造得出，
而且撕裂点是**写死在 dm 表里的**、可复核。

⚠️ **射程要写死，这一条只推翻那句元断言**：D2（RAID 条带策略） 硬要求 1 的**实质**主张是
「设备内部会做 read-modify-write，掉电可能损坏同一映射单元里已经持久的邻居数据」，
撕裂注入器造不出「设备内 RMW」——那要结论二与结论三答。
⚠️ **`dm-flakey` 那半句仍然是对的**：它的 `corrupt_bio_byte` / `random_write_corrupt` 造的是
**字节损坏**，不是撕裂（前缀持久、后缀未持久）。两者不许互相充数。

### 结论二：非 RAID 的 `io_min > physical_block_size` 真设备存在，而且一次 512 字节的写在它上面放大成整块

`dm-thin`，块 64 KiB。块**共享**（有快照）时对块内任意 512 字节的写触发整块 COW。
产物 `e129-thin-rmw-2026-09-10.out` 逐字：

```
E7RESULT name=cell round=1 shared=1 host_sectors=1 data_dev_sectors=129 thin_io_min=65536 thin_pbs=512
E7RESULT name=cell round=1 shared=0 host_sectors=1 data_dev_sectors=1 thin_io_min=65536 thin_pbs=512
E7RESULT name=verdict shared_not_amplified=0 exclusive_amplified=0 cells=10 cells_with_declared_iomin=10 declared_block_bytes=65536
```

三件事一起出来：① 主机发 1 个扇区、数据设备写 **129** 个（整块 128 + 那一个）；
② 块独占时只写 1 个，**两格差 129 倍**，对照分得出差别；
③ `thin_io_min = 65536` 而 `thin_pbs = 512` ⇒ **这是一台 `io_min` 比 `physical_block_size` 大 128 倍的真设备，
而且它不是 RAID**。C212（固定结构的写小于 io_min） 的前件此前只在阵列上被量到过（E34（根环槽几何）），
这一条把它扩到了非阵列。

⇒ **D2（RAID 条带策略） 硬要求 1 说的「向一个设备物理映射单元里发一次更小的写，设备内部会做 read-modify-write」
在真设备上量到了，而且是第二条机制**——它与 RAID5 写洞不同族：没有 parity、不跨腿。

### 结论三：这条 RMW 被精确打断时，同一映射单元里的邻居 5/5 轮逐字节完好

把结论一的撕裂注入器垫到 pool 的**数据设备**下面，让 COW 的后半截落不下去。
产物 `e129-thin-neighbour-2026-09-10.out` 逐字：

```
E7RESULT name=neighbour round=1 inject=1 write_rc=1 read_rc=0 verdict=intact first_diff=-1 cow_sectors=64 pool_data_before=1/256 pool_data_after=2/256
E7RESULT name=neighbour round=1 inject=0 write_rc=0 read_rc=0 verdict=intact first_diff=-1 cow_sectors=129 pool_data_before=1/256 pool_data_after=2/256
E7RESULT name=verdict inject_not_torn=0 negcontrol_damaged=0
```

⚠️ **判别力那一半是承重的，先说它**：光看「邻居完好」分不出「机制是安全的」和「注入根本没打到 COW」。
`cow_sectors=64`（注入格）对 `129`（对照格）证明**注入确实把 COW 撕在了设计好的边界上**——
好段 192 扇区 = 块 0 整块 128 + 块 1 的前 64，COW 正好落进 64 就撞上错误段；
`pool_data 1/256 → 2/256` 两格都涨，说明两格都真的分配了第二个块。
少了这两个数，这一格的「完好」什么也不证明。

⇒ **失败形态是「写失败 + 旧数据完好」，不是「静默的邻居损坏」**：
`dm-thin` 的块内 COW 是**事务性**的——先分配新块、拷完才切映射；拷贝被撕裂时映射不切，
读到的仍是旧的共享块 ⇒ 邻居逐字节完好，而写 A 的那一方**收到错误**（`write_rc=1`）。

### 这三条合起来对 C212（固定结构的写小于 io_min） 意味着什么

**给候选 ② 的是一条正面依据，但不足以让它成立。** 逐条说清：

| 给了什么 | 没给什么 |
|---|---|
| 「所有设备内 RMW 都危险」这个**全称被削弱**：至少有一条（`dm-thin` 块内 COW）是事务性的，打断它不损坏邻居 | 它**只覆盖 `dm-thin` 这一条机制**，不推广到别的 |
| 「`io_min > pbs` 只在阵列上出现」这个印象被推翻：非 RAID 的也有 | SSD FTL 内部对整个映射单元的 RMW **仍然注不进去**（E34（根环槽几何） 已记，E129（小于 io_min 的写在真设备上怎么出事） 没解决）|
| 撕裂态可以按需构造 ⇒ 一整类不变量今天就能验，不必等崩溃点重放 harness | ⚠️ **它不构成「与 RAID5 相反」的一对**：E34（根环槽几何） 真设备那一半逐字是「同 chunk 内邻槽 B（扇区位 1）**10/10 完好**」，故障单元「是「同 stripe 行 × 同扇区位」跨腿，**不是整个 `io_min` 单元**」⇒ **两条已量机制的邻居都完好**，「设备内 RMW 会损坏同一映射单元里的邻居」这句话今天**一条实测支撑都没有** |

⇒ **推论（未走三方，不作数，只记形状）**：D2（RAID 条带策略） 硬要求 1 的理由句
（「设备内部会做 read-modify-write，掉电可能损坏同一映射单元里已经持久的邻居数据」）
在真设备上是**逐机制成立或不成立**的，不是全称。要动那条已定条款得走三方论证。

### 口径与复跑

三段都是真设备（loop + dm/md），**一个字节都不碰 `nvme0n1` 的分区**；
要 `sudo`，口令从 `.env` 读、不打印；设备名全部来自变量，`EXIT` 时按记录下来的名字逐个拆除。

```bash
bash research/scripts/e129-tear-injector.sh      # 结论一
bash research/scripts/e129-thin-rmw.sh           # 结论二
bash research/scripts/e129-thin-neighbour.sh     # 结论三
```

三份原始输出：
`research/results/e129-tear-injector-2026-09-10.out`、
`research/results/e129-thin-rmw-2026-09-10.out`、
`research/results/e129-thin-neighbour-2026-09-10.out`。

⚠️ **复跑不是逐字节比对**：设备名（`/dev/loopN`）与工作目录每次不同。
钉住结论的是三份产物里的 `name=verdict` 行，以及各自的作废条款——
任一对照不符时脚本**自己报 `name=fatal` 并以非 0 退出**。

⚠️ **它造的是盘上状态，不是真的断电**：loop 断不了电。状态等价，路径不等价。
这一句与 E34（根环槽几何） 真设备那一半是同一条限制。
⇒ 它买到的是「能不能构造出撕裂态去验一条不变量」，**不是**「撕裂多久发生一次」。

### 变异

三张表共 9 条，全部被抓：`research/mutations/e129_tear_injector.tsv`（3）、
`research/mutations/e129_thin_rmw.tsv`（3）、`research/mutations/e129_thin_neighbour.tsv`（3）。
被测装置是 shell 探针不是 Rust 二进制，跑法写在各表头部：把「原文」换成「替换文」，
跑一遍脚本，**必须出现 `name=fatal` 且退出码非 0**。

⚠️ **其中一条判为等价变异并留档**：把 `dm-thin` 的块宽从 128 扇区改成 256 抓不到，
**而且不该抓**——`BLK_BYTES` 由 `BLK_SECT` 算出、`io_min` 断言跟着走，
而结论二对块大小是尺度无关的 ⇒ 那是一个合法的别的配置，不是 bug。
按 `.claude/rules/mutation-sampling.md` 换成「让独占格也建快照」，从另一侧考同一个判别子。

⚠️ **补这条断言的过程本身踩了第三类**：`shared_not_amplified` 的阈值 128 是写死的，
**对「块大小往上改」不敏感**，所以才要另钉一条「实测 `io_min` 必须等于本段声明的块宽」。

## 历史版本

E129（小于 io_min 的写在真设备上怎么出事）的历史条目集中在 [experiments-history.md](../experiments-history.md)。
