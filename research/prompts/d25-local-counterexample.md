# Background: singlefs D25 target-workload priority

## 已核事实（本轮从 E16 原始输出与源码逐条核过）

E16 是确定性模型实验，数的是块数不是字节数。参数：FANOUT=128，HEIGHT=4（内部层，不含叶），
BLOCK=4096，ROOT_SLOT_BLOCKS=1，20 万次操作，checkpoint 间隔 1000。
被测臂 `intent`（甲）= 本工程已定的方案：每次 fsync 写脏叶 + 全部祖先 + 根槽 + 一条记录。

四条负载的生成器（源码 `e16_journal.rs:122-152` 逐行核过）：

| 负载 | 一次操作触到的叶子 |
|---|---|
| seq | 8 个连续叶子 |
| rand | 1 个全域随机叶子 |
| metaheavy | 2 个热区随机叶子（热区 = 全部叶子的 1/1024） |
| multistream | 1 个叶子；16 条流轮流，每条流在自己的子树里顺序推进，子树在根的子节点那层就分开 |

`wa` = 总写块数 / 用户写块数。实测（arm=intent，ckpt_interval=1000）：

| 负载 | 用户块/操作 | 不 fsync 的 wa | 每 10 次 fsync 的 wa | 每次 fsync 的 wa | 每次 fsync 的固定开销（块） |
|---|---|---|---|---|---|
| seq | 8 | 1.0086 | 1.0813 | 1.7500 | 5.93 |
| rand | 1 | 3.1010 | 4.2651 | 7.0000 | 3.90 |
| metaheavy | 2 | 1.6449 | 2.7763 | 4.9681 | 6.65 |
| multistream | 1 | 1.0587 | 4.3000 | 7.0000 | 5.94 |

「每次 fsync 的固定开销」= (每次 fsync 的总块数 − 不 fsync 的总块数) / 操作数。

## 已定的相关决策（原文，不是转述）

- D23 轴一：每次 fsync 发一个根。轴二取甲：祖先不延后。
  轴二是被轴一结构性排除的（延后祖先 ⇒ 新根不可能自洽 ⇒ 发不出根），与收益多少无关。
- D10：把「数据库和虚拟机镜像上的随机写」记为 COW 文件系统的共同软肋。
- D25（本决策）：目标负载优先级从未定过，是产品取舍不是可测量。

## 另外两个已测的、口径不同的实验

- E20（CPU 缓存）：条目 40 字节时，4 KiB 节点点查慢 1.98 倍（树深 3→4），
  16 KiB 节点只慢 1.14 倍（树深仍 3）。⇒ 大节点吸收指针变宽。
- E7（设备 I/O）：小节点在两端都赢。
  E20 与 E7 指向相反方向，两者量的是不同资源，不矛盾。

## 尚未做的

- 没有任何实验量过字节数，只量过块数。降低树高要靠增大扇出，而节点变大后每块的字节也变多。
- 没有任何实验量过屏障数（一次 fsync 要几次 flush），只量过块数。
- 没有真机、没有事务层，以上全部是模型层数字。

# Your task: find counterexamples

A claim has been derived from the table above. Attack it.

Claim T: "Single-stream versus multi-stream is not the variable. The per-fsync fixed cost is
the same for seq and multistream (5.93 vs 5.94 blocks). The 4x write-amplification gap between
them comes entirely from the denominator: how many user blocks one fsync carries. Therefore
D25 should not ask 'single or multi stream', it should ask 'what is the fsync granularity of
the target workload, in blocks'."

Answer these, numbered 1 to 6:

1. Give a concrete workload where Claim T predicts the wrong answer. Be specific about the
   access pattern and say what the table would predict versus what would really happen.

2. The rand row breaks the pattern: its per-fsync fixed cost is 3.90 blocks, not about 5.9.
   Explain why, and say whether that undermines Claim T or is consistent with it.

3. Claim T counts blocks. Reducing tree height requires raising fanout, which makes each node
   bigger. Work out whether Claim T survives if the metric is bytes instead of blocks. Show
   the arithmetic.

4. Claim T ignores barriers. A single fsync at queue depth 1 may cost more in flush barriers
   than in blocks. Say how many barriers the described design needs per fsync, and whether
   the ranking of the four workloads changes when barriers rather than blocks are counted.

5. If fsync granularity really is the only variable that matters, name one design decision
   that would be settled differently under coarse granularity (8 blocks per fsync) versus
   fine granularity (1 block per fsync). Be concrete.

6. Which single sentence in this whole picture is most likely to be wrong, and what
   observation would show it?

Formatting: answer in plain prose. Do NOT use bold, italics, backticks or any markdown
emphasis anywhere in your answer. Number each answer 1 to 6 and nothing else.
