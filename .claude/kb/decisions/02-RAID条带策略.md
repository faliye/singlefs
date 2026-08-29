## D2 RAID 条带策略 —— 半定（策略已定，两项未定）

**条带宽度可变，每次写都是全条带写，永不 read-modify-write。**

write hole 不是被修好的，是被设计成不存在。ZFS RAID-Z 已验证此路可行。

代价：空间效率略降，小写有浪费。接受。

### 写的粒度：不发出小于设备物理映射单元的写

**已定（2026-08-28）：本决策的「永不 read-modify-write」要扩到设备侧。**

原有措辞管的是「**我们**不做 RMW」，管不到「**我们不诱发设备做 RMW**」——
向一个设备物理映射单元里发一次更小的写，设备内部会做 read-modify-write，
**掉电可能损坏同一映射单元里已经持久的邻居数据**。

⇒ 两条硬要求：

1. **不发出小于设备物理映射单元的写。** 在 Linux 上这个量是 **`io_min`**，不是
   `physical_block_size`。⚠️ **两者是不同的量，此前本决策把它们当成了同一个。**

   | 量 | 内核里怎么来的 | 它是什么 |
   |---|---|---|
   | `io_min` | `lim->io_min = phys_bs`，其中 `phys_bs = bs * (1 + npwg)`（NPWG = Namespace Preferred Write Granularity） | **不诱发设备 RMW 的最小写宽度** |
   | `physical_block_size` | `lim->physical_block_size = min(phys_bs, atomic_bs)` | **设备承诺的掉电原子宽度**（`atomic_bs` 由 NAWUPF 定，未声明则等于逻辑块） |

   （2026-08-28 现查 `drivers/nvme/host/core.c:2126-2143`，Linux 7.2.0。
   那个 `min` 上方的注释原文只讲原子性：「Linux filesystems assume writing a single
   physical block is an atomic operation. Hence limit the physical block size to the
   value of the Atomic Write Unit **Power Fail** parameter.」）

   ⇒ **`physical_block_size ≤ io_min`，且设备不声明 NAWUPF 时前者会塌到逻辑块大小。**
   按 `physical_block_size` 发写**照样可能低于 NPWG 而诱发 RMW**。
   ⇒ 本硬要求的量取 **`io_min`**；原子宽度那件事另用 `physical_block_size` 表述，
   两个量不许再合成一句话。
2. **不让两个生命周期不同的对象共享同一个物理映射单元。**
   共享则一个对象的写会把另一个对象拖进同一次 RMW 的风险窗口。

依据：D22（单元原子性怎么合成） 反推 C 暴露的一处——「我们不做 read-modify-write」这句话
覆盖不到「我们不诱发设备做 read-modify-write」这一侧。
⚠️ **这条在本机无法用故障注入证伪**（`dm-flakey` 造不出撕裂），按
`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 记为**机制推导**，证伪等崩溃点重放 harness。

### 未定项

1. **「宽度可变」的粒度从没写明。**
2. **对设备异质性的前提从没写明**（各盘要不要等大）。

#### 未定项 1：「宽度可变」的粒度

**每次写都可能是不同的宽度，还是同一个分配组内固定、跨组才变？**

两种读法给出不同的格式：前者要求每个物理位置逐个列出且各带设备身份；
后者可以走 btrfs 那种「逻辑地址 + chunk 映射表」，指针里不必有设备身份。

⇒ **这是 D19（块指针的结构与宽度预算） 未定项 1（指针带不带 dev）的真前置**，
本决策定案时未曾涉及。

⚠️ **本决策也从未写明对设备异质性的前提**（各盘要不要等大）。
md/raid0 那次跨 5 年的静默数据损坏，触发条件正是各盘不等大
（见 D12（目标介质） 未定项 3 那张表）。

**存在第二个已知好答案（2026-08-26 调查补充，未采纳，备查）**：
bcachefs 在**后台对整个 bucket 做纠删码**，而不是把前台写切成条带——
前台正常多副本写，后台攒够候选 bucket 再算一次 parity，然后原子更新所有指针。
parity 只对不可变数据算一次，因此同样不存在 write hole。
代价是 stripe 里只要还有活数据，其中的 bucket 就不能复用。
两条路都成立，本决策维持变宽全条带；若将来 D8（核心索引结构） 采用 bucket 模型，此项应重新权衡。

**与 D9（加密）的连锁**：parity 对密文算，多副本是同一份密文写 N 遍——加密 N 次会得到 N 份不同的字节。

## 历史版本

本决策的历史条目集中在 [decisions-history.md](../decisions-history.md)。
