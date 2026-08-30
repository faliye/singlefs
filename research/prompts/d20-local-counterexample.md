# Background: singlefs D20 corollary 3, questions 1 and 2

D20（承重面：单元的原子性与自包含）已定「承重面 = 单元原子性 + 自包含」，
但推论三的三问只答了第 3 问的一半。现在要定第 1、2 问：

1. 原子的宽度是多少——块？扇区？某个声明值？
2. 谁保证它——设备声明，还是核心层合成？

## 已核事实（逐字现查，可直接当前提）

| # | 事实 | 出处 |
|---|---|---|
| 1 | 本机 NVMe：atomic_write_unit_min = atomic_write_unit_max = logical_block_size = physical_block_size = 512 | /sys/block/nvme0n1/queue/ 现读 |
| 2 | Linux 上 physical_block_size 已经就是「设备承诺的掉电原子宽度」 | drivers/nvme/host/core.c：lim->physical_block_size = min(phys_bs, atomic_bs) |
| 3 | 内核只认 Power Fail 变体（AWUPF / NAWUPF），对 AWUN / NAWUN 零引用 | nvme_configure_atomic_write() |
| 4 | 「设备声明 1 块」与「设备什么也没声明」不可区分 | nvme_configure_atomic_write()：设备不声明时 atomic_bs 落到逻辑块 |
| 5 | 块层从未承诺「一个 bio 不会被撕裂」 | Documentation/ABI/stable/sysfs-block 的四个 atomic_write_* 条目没有一处提到 crash / power fail |
| 6 | 现役实现全部自己合成撕裂检测，没有一家靠设备 | ZFS 根用 4 label × 128K uberblock 环、槽号 = txg % 槽数、SHA-256 内嵌块尾；ZFS 普通块把校验和放父指针；XFS 日志按扇区盖 cycle 号；jbd2 有 jbd2_commit_block_csum_verify_partial() 显式接受「只有头部有效、其余为零」 |

## 本轮新量到的（E23 journal 几何，2026-08-29）

要求「记录头完整落在一个原子单元内」的对齐代价：

| 原子宽度 | 点名 1 项（140 B） | 点名 10 项（644 B） | 点名 100 项（5684 B） |
|---|---|---|---|
| 512 | 512 B（3.7 倍） | 1024 B（1.6 倍） | 6144 B（1.1 倍） |
| 4096 | 4096 B（29.3 倍） | 4096 B（6.4 倍） | 8192 B（1.4 倍） |

不对齐、让多条记录挤同一个原子单元则空间不浪费，但除第一条外都判不了撕裂。

journal 记录头字段合计 84 字节（magic 4 + 类型 2 + 算法类型 1 + 填充 1 +
自述长度 4 + 点名项数 4 + jsn 8 + checkpoint_txg 8 + tail_lsn 8 + nonce 12 + 头部校验和 32）。

## 相关已定决策（原文，不是转述）

- D2（RAID 条带策略）硬要求 1：不发出小于设备物理映射单元的写，那个量是 io_min 不是
  physical_block_size；physical_block_size <= io_min，且设备不声明 NAWUPF 时前者塌到逻辑块。
- D4（校验和位置）：校验和内联在父指针里（Merkle）。根没有父，所以根必须自证。
- D22（单元原子性怎么合成）：根必须自证 = 整单元校验和 + 代号 + 槽轮换。
- D9（加密）：加密开启时 D4 那个字段就是 MAC，满 128 位。

## 尚未做的

- 崩溃点重放没有实现，本仓对「失败原子性宽度 / 撕裂 / 重排序」目前零覆盖。
- 没有在真机上验证过任何撕裂假设。

# Your task: find counterexamples

A candidate answer has been drafted. Attack it.

Candidate answer: "The core layer synthesizes atomicity; the device declaration is never
relied upon. The atomic width is therefore not a device property at all but a format
constant: the unit over which we can detect tearing by our own means. Every production
implementation already does this, and the device declaration is unusable anyway because
declaring one block is indistinguishable from declaring nothing."

Answer these, numbered 1 to 6:

1. Give a concrete case where synthesizing tear detection in the core layer is not enough,
   that is, where a torn write passes our own check. Be specific about the mechanism.

2. If the atomic width becomes a format constant, some device will have a real atomic width
   smaller than our constant. Work out what breaks, and whether it is detected or silent.

3. The candidate says the device declaration is unusable because declaring one block is
   indistinguishable from declaring nothing. Attack that step: is there any way to tell them
   apart, or any case where it does not matter?

4. Per sector stamping and a whole unit checksum are two different mechanisms. Say precisely
   what each one can detect that the other cannot, and whether one of them makes the other
   redundant.

5. The alignment cost table shows 29.3 times waste for a small record at 4096 byte alignment.
   Find a design that keeps per record tear detection without paying that. If none exists,
   say so and say why.

6. Which single sentence in this whole picture is most likely to be wrong, and what
   observation would show it?

Language: answer in English only. Do not use Chinese anywhere in your answer.

Formatting: answer in plain prose. Do NOT use bold, italics, backticks or any markdown
emphasis anywhere in your answer. Number each answer 1 to 6 and nothing else.
