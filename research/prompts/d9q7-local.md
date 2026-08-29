# Background: singlefs D9 open item 7

## 问题

D9（加密）未定项 5 已定：无密钥侧的 GC 能「搬块或回收块」。
但「回收」这一半与 D21（权威态与派生态的分界）冲突，而这个岔路从未被显式选过。

冲突的推导（两条都是已定项）：
- D21 已定：权威态 = 单元 + 记账 + 根。⇒ 记账是权威态。
- D9 方向已定：除超级块外全部加密。⇒ 记账加密 ⇒ 无密钥侧写不了记账。

⇒ 无密钥侧没有安全的手段真正完成「把空间标记为可分配」。

## 两条岔路（原文）

- 甲：无密钥只在明文映射里标「死」，不触碰加密记账。可逆，等密钥回来再核对。
  代价：回收其实没发生。
- 乙：另设一份明文空闲表。代价：一个从未出现在 D9 / D19 / D21 里的新结构。

## 一条已立但判不出成立的命题（原文）

命题 4：无密钥侧基于明文信息发起的搬运或回收，不能在密钥侧核对之前造成
物理地址别名（两个权威结构在同一时刻声称占有同一物理位置）。

它依赖「无密钥侧如何安全获知某个位置空闲」，而那个东西还不存在。
搬运不构成威胁（搬到哪都解不开，且 D2 / D9 本来就允许逐字节复制）；
回收才构成，且它的后果是不可逆的数据丢失，不是「读出」。

## 相关已定项

- D9 未定项 5 已定：反向索引 + bucket 代号留明文。
- D9 未定项 8 已定（2026-08-29）：fsid 进 KDF；超级块因 nonce 水位而必须被
  主密钥派生的 MAC 覆盖，无密钥侧不许写它；整卷回滚宣布为已知不防。
- D1（数据可移动性 / 反向索引）已定：数据可移动是硬需求。
- D3（空间分配）已定：常驻后台整理；「释放空间这个操作本身不需要申请空间」。
- 本仓已记录一句：锁盘状态下整理瘫痪会让 D1 的缩容与移除设备、D3 的碎片整理
  在最需要的时候一起失效。
- fs-design.md 纪律：运行时决策路径不许靠遍历；审计与被审计不许用同一段代码。

## 尚未做的

- 崩溃点重放没有实现；没有真机；没有事务层。
- 命题 4 目前既不成立也判不出成立。

# Your task: find counterexamples

A candidate answer has been drafted. Attack it.

Candidate: "Take fork A. The keyless side marks blocks dead only in the plaintext map and
never touches the encrypted accounting. Fork B introduces a second structure that claims to
know what is free, and two structures claiming the same thing is exactly the alias that
proposition 4 forbids. The cost of A, that reclamation does not actually happen while the
volume is locked, is acceptable because reclamation can be completed the moment the key
returns."

Answer these, numbered 1 to 6:

1. Give a concrete situation where fork A's cost is not acceptable, that is, where the volume
   must reclaim space while locked or something breaks. Be specific about what breaks.

2. Attack the claim that fork B necessarily creates an alias. Is there a way to build the
   plaintext free table so that it cannot disagree with the encrypted accounting? If yes,
   describe it; if no, say precisely why not.

3. Under fork A, the plaintext map accumulates dead marks that are never acted on. Work out
   what bounds that growth, and what happens if the volume stays locked for a very long time.

4. Proposition 4 says moving blocks is not a threat but reclaiming is. Attack that split:
   find a case where moving alone causes the same harm as reclaiming.

5. The candidate says reclamation can be completed when the key returns. Work out what that
   completion step has to do, and whether it can fail or be interrupted halfway.

6. Which single sentence in this whole picture is most likely to be wrong, and what
   observation would show it?

Language: answer in English only. Do not use Chinese anywhere in your answer.

Formatting: answer in plain prose. Do NOT use bold, italics, backticks or any markdown
emphasis anywhere in your answer. Number each answer 1 to 6 and nothing else.
