# m3-intent-r1 攻方计数模型：跑前写死（2026-10-01，写于第一次运行之前）

模型 `gc_intent_model.py`：一块盘、NSEG 段 × SEGSZ 槽、一个单元 = 一个槽；每个操作 = 一次原子发布，
崩溃态 = 历史的任一前缀，所以枚举里每个状态都判一遍（不跑 layer0 枚举：crates/ 里没有整理代码，这里没有写流）。
用户动作全放开（写、unlink、删除批、签发整理意图、整理批、回退到环里任一旧根），只限深度与对象数。

臂（每条臂一个名字，跑前定义）：
- carrier：AB = unlink 走 deleted_inodes（随 R_old），整理意图住 logged_ops；C = unlink 条目与整理意图同住 logged_ops。
- lops_rb：回退时 logged_ops 取 R_old（old）还是随现行（cur）。
- cursor：skip = 续做点走到 R 末尾就删意图；recheck = 删意图前全量核 R，核不过回到 R 起点（C364 第二轮的收严）。
- batch：naive = 删除批只核「要删的单元还在」；nlink = 另核对象 nlink = 0，不是就丢掉那条队列项。
- admission：plain = 今天的式子（可用 = 全部可回收空槽 − 环窗口）；rterm = 再减活意图 R 里的空槽；
  rterm_issue = rterm 且签发整理意图时也判「签发后 R 外空槽 ≥ 环窗口」。
- nogc：不许签发整理意图（对照：环窗口模型自己对不对）。
- 变异：gc_forget_release（整理批搬完不释放旧槽）、rb_no_resurrect（回退不复活 R_old 引用的已释放槽）。

判据（每个状态都判）：live_lost / del_lost（对象引用的槽不在已分配）、leak / dangling / double_ref（已分配槽与被引用槽两集合不等）、
queue_has_live / zero_not_queued（删除队列与 nlink = 0 集合不等）、gc_del（整理意图被删的那一刻 R 内还有分配代 ≤ c0 的已分配槽）、
rb_refused（回退被拒，按原因）、事件：false_enospc（写过了式子而落点给不出）、unlink_stuck / delete_stuck（删除类发布落点给不出）、
cont_blocked（队列非空且 R 外没有空槽：挂载里的续做走不下去）、gc_livelock（只跑整理批 2·SEGSZ+2 步还没完成、也没被空间挡住）。

跑前预测（结果反过来我照改结论）：
1. nogc 对照：delete_stuck、unlink_stuck、cont_blocked 全 0。不是 0 ⇒ 环窗口模型错，先修模型、不判别的。
2. AB+old+recheck+nlink：全部判据 0；rb_refused 0。
3. AB+cur+skip：gc_del > 0；AB+cur+recheck：gc_del 0。
4. AB+old+skip：gc_del 0。
5. C+cur（naive 与 nlink 两种）：zero_not_queued > 0；naive 另有 live_lost > 0。C+old：全 0。
6. plain：false_enospc > 0、cont_blocked > 0；rterm：false_enospc 0、cont_blocked > 0（签发在填满之后）；rterm_issue：两样都 0。
7. 两个变异：gc_forget_release ⇒ leak > 0；rb_no_resurrect ⇒ live_lost > 0（判据自己会红）。
推翻条件：2、4、5 的 C+old 出现任何非 0 ⇒ 「回退取 R_old」不成立；6 的 rterm_issue 非 0 ⇒ 我提的改法不够。

## 修订（冒烟跑之后、正式跑之前，2026-10-01）

- 冒烟跑（深度 5–7，几何 3×4、环 3）发现两处模型缺口，改了模型、判据原样保留：
  1. 回退候选集收了 mkfs 那一版（一个分配记录都没有的根），D23 已定项 14 的候选集只收带文件的根 ⇒ 去掉那一种目标。
  2. naive 删除批删活对象时只是把对象的数据单元截短、引用与分配记录照样对得上，live_lost 判不出 ⇒ 对象带上写入时的数据单元数，
     加判据 live_truncated（nlink ≥ 1 的对象数据单元数少于写入时）。预测 5 改成：C+cur+naive 出 live_truncated > 0。
- 冒烟读数（不作结论，只说明为什么改）：C-cur-recheck-naive 深度 7 有 queue_has_live、zero_not_queued 而 live_lost 0。

## 修订二（第一次正式跑之后，2026-10-01）

第一次正式跑（`model-output-run1.txt`）推翻了预测 6 的后半句：adm-rterm_issue 不是 0（几何 3×4、环 3 深度 8 cont_blocked 24，深度 9 cont_blocked 226、unlink_stuck 14、delete_stuck 38）；
plain 臂的 false_enospc 在探到的深度上是 0，只在回退随现行的臂上出现。输一次照记（报告第八节）。
加一条新臂 adm-delexempt（准入照 plain，删除类发布的落点可以落进活整理意图的 R），跑前预测：cont_blocked、unlink_stuck、delete_stuck 全 0；
理由：nogc 对照全 0，而 rterm_issue 剩下的反例都是「被删对象的槽在 R 里，释放之后回到 R、不回到 R 外」。不是 0 ⇒ 这一读法错，照实写。
