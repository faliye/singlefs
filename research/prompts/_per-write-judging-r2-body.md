# 崩溃状态逐写独立判：判据与成立条件（第二轮正文，2026-11-06）

## 一、这一轮要判什么

第一轮判决 `research/prompts/per-write-judging-r1-main-verification.md` 没有立判据。这一轮照第二轮分工（第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方）接着判同一件事：「逐写独立判」与全组合枚举判出同一结论，需要哪些条件；判据写成引擎读段表就能逐段判定的形式；要保的是「红不红」还是「红项」。

第一轮问题、候选、共用问句照第一轮正文 `research/prompts/_per-write-judging-r1-body.md` 第一节，这一轮不改。

## 二、实现今天的样子

方案按 `crates/` 今天的实现来谈。读过的 `crates/` 路径与看到的事实，照第一轮正文第二节（`crates/singlefs-verification/src/crash.rs:1932`、`:2052`、`:2173`、`:2190`、`:1937`，`crates/singlefs-checker/src/walk.rs:11252`–`11256`，`crates/singlefs-checker/src/image.rs:2076`，`crates/singlefs-verification/src/crash.rs:148`、`:182`）与调查员报告 `/home/fy5090/.local/share/singlefs/work/per-write-judging-facts/result.md`。第一轮两腿另引到 `crates/singlefs-core/src/transaction.rs:10564`（搬迁副本不盖戳）、`crates/singlefs-checker/src/walk.rs:17931`（子集规则），这一轮要核。

## 三、第一轮留下的（量的与没量的）

量的：

- 任务 8321：删除流段 12、16、20 不动时，单写、成对、只差一个、随机 2000 个子集逐态与 S=∅ 比，`full_differs=0`；阳性对照两份副本都盖时 `red_items_differ=true`，且 `PROBE_PAIRS_NOT_UNION pairs=1`（两份都盖的红项不等于两个单写红项的并）。阳性对照那两份写是 `over_an_earlier_write=true`。
- 任务 8593：记录核对器 C561 四格测试绿。
- 任务 8686：攻方腿的原型复跑退 0，但每段都被 `写数超过 19` 跳过，`PROTO_PLANNED_STATES total=0`，一个状态都没判。

没量的（第一轮挂起或核不动）：攻方①之外的攻方论点、反例 A 与 B、两腿补的破坏条件（攻方 5 类、正推 2 类）、正推腿判据 D1–D5、候选乙、观察者分三类。

## 四、各腿要回答的

1. 攻方：把原型改成在写数多的段上按子集取样判（单写、成对、只差一个、随机），在反例 A（副本写晚落进下一个单元段）、反例 B（记录先写、点名的单元后写）那几条臂上真判状态，报出逐写判（候选甲、乙）与全组合判在红不红、红项两层的差，每格带取样数。改完的命令写成整条交主 agent。
2. 辩方：复核正推腿判据 D1–D5 与候选乙，对着第一轮判决「逐条判」表里挂起的每一条答：判据哪里要收严、收严之后删除流段 12、16、20 与对拍三条流各剩多少态；替被攻的候选乙辩护，判反例 A、B 是不是同样打中所有替代方案。
3. 两腿都从正推腿列的 8 类敏感取样点里至少挑 3 类，写成能跑的命令。
