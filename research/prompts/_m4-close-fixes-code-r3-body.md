# m4-close-fixes-code-r3：r2 判决第四节与 P2a，第三轮，最终轮（2026-11-09）

<!-- doc-lint:not-numbers T1 T2 T3 T4 P1 P2 P2a P3 -->

## 一、题面

`research/prompts/m4-close-fixes-code-r2-main-verification.md` 第四节五条（读全部接上计数、映射 key 集合不等就拒、账的三处、名字与死链、测试）与用户 2026-11-09 定的 P2a（固定点每轮把释放链走到闭包，E206（发布固定点轮数的代价））已打进主工作区，都被攻过零轮。这是这一题的第三轮，也是最终轮，之后不再开第四轮。

照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第三轮分工：「已定」项撑不撑得住、未定项清单的完整性、那一轮报出的数字能不能核。这一轮只攻前两轮站住的形态与这批改动；这一轮里新冒出来的零轮形态写进判决的交用户表，不再为它开一轮。

改动全文：`research/prompts/_m4-close-fixes-code-r3-diff.md`（`crates/singlefs-core` 7 个文件块）。实现员报告：`~/.local/share/singlefs/work/m4-close-fixes-r2-writer/result.md`、`result-reds-and-closure.md`、`result-unkilled-rows.md`。

## 二、实现今天的样子（主 agent 的观测，2026-11-09）

- 被判的产品代码：`crates/singlefs-core/src/` 的 `allocator.rs`、`compaction_plan.rs`、`instance_table_sweep.rs`、`mount.rs`、`mounted_session.rs`、`recovery.rs`、`transaction.rs`（开工快照 `research/prompts/m4-close-fixes-code-r3-snapshot/crates-sha256.txt`）。这几份里不是这批的块（别的会话的改动）当上下文读，腿只攻 diff 里有的块。
- 验证到的数（量的）：主工作区 clippy 任务 11330 退 0；带 gpu：`unit_read_…` 37 条过、`crash_points_…` 4 条过、`states_judged_by_the_gpu_here=182`（任务 11331）；P2a 目标 `allocation_record_tree_settling_walks_the_release_chain_to_its_closure` 1 条过（任务 11332）；崩溃枚举用例缩域到 5：`states=177 … exhaustive=false violations=0`（任务 11334，缩过域不算门禁通过）。变异在副本上都证过红（任务 11220、11288、11307、11311）；「释放处不走闭包」红在 `覆盖写 inode 1 的固定点走了 29 轮`。全量域 GPU、不指定范围的门禁都没跑。
- 压着的条款：D4（校验和位置） 已定项 4，用户定案「按字面全接上」（`research/prompts/m4-close-fixes-code-r1-main-verification.md` 第三节）；映射 key 不等就拒（r2 判决第三节）；P2a 的定义 `research/prompts/e206-preregistration.md` 第五节 P2a 那一行；E206 产物与判定 `.claude/kb/experiments/206-发布固定点轮数的代价.md`（P1 逐轮读数按作废条款 1 作废，只作线索）。
- 方案按 `crates/` 今天的实现来谈。

## 三、攻击面

1. **P2a 的前提**（登记写「被并进去的节点都在 P1 的不动点里」）：`merge_into_the_rewrite_set_walking_the_release_chain` 一带，走闭包会不会并进 P1 不动点之外的节点（多写单元），读上一版指针的时机对不对。量到的只有 16 个形状（第二段 33 项没跑），测试只有一段历史；什么形状 P2a 仍走满上界 64 拒。实现员报告写了写行那一份（树表 0 条那一版）两处并入没有证红，判这个盲区。
2. **映射 key 集合不等就拒**（`recovery.rs` 一带）：有没有合法的盘被拒——留存条目、搬迁中崩溃、回退之后、快照销毁、整理半途这些状态，key 集合的等式都成立吗；GPU 判器少判的一形（实现员报告第 2 条）；只读路径不判留下什么。
3. 读全部接上之后：穷举表（实现员报告）里没接的 `transaction.rs:668`、`mount.rs:1407` 一带的依据站不站得住；接了线而没有变异行的 6 处（`instance_table_sweep.rs:743`、`:921`，`mounted_session.rs:4115`，`mount.rs:10789`、`:10901`、`:5350`）。
4. 账的口径统一之后：去重的范围、「只试一次」写实之后那一句，与 D4（校验和位置） 已定项 4 对不对得上。
5. 未定项清单：r1、r2 判决与实现员「停下交主 agent」里没有去向的条目，有没有漏；攒批会话读不到待发写（r2 判决第五节）要不要进条款。
6. 数字能不能核：正文第二节每个数对产物；E206 页引的行。

## 四、检查单

### 检查单：走不到的理由二次查实

实现员报告原样：「补丁在产品代码里没有新加 `todo!`、`unreachable!`、`assert!`，也没有以 Undecided、Unsupported、OutsideTheFirstVersion 收尾的错误成员（`grep` 补丁的加行，量的）。删掉的那一支（`transaction.rs` 旧 `:11953`）的理由写在第 4 条。」第 4 条：「账上的组都是这次发布之前读到的」。P2a 那一批报告写「走不到的理由：0 处」。

### 检查单：类型与名字

新加的名字由腿看 diff；`merge_into_the_rewrite_set_walking_the_release_chain`、`PoolWriter::rounds_of_the_last_allocation_record_tree_settling`（只读观测）、映射 key 集合那一判的拒因、改名后的开关。`expect` 依赖的不变量逐处看。

### 检查单：不兼容旧代码与旧格式

主 agent 没看到认旧版本的分支；P2a 不留开关。腿现跑 `grep -nE 'legacy|old_format|migrat' research/prompts/_m4-close-fixes-code-r3-diff.md`。

### 检查单：不新建判定库

本轮改动没有这一类：产品代码不写判定。

### 检查单：所有任务多机可用

产品代码没有这一类；新测试目标由腿看读不读本机状态。

### 检查单：测试优先多线程

P2a 目标单历史逐次判，次序就是被测对象；由腿核。

### 检查单：带 --item 的命令与项

带 `--item` 的 crates-mutation-replay 这批没跑（任务 11017 要记 45.6 GiB、等整台腾空，取消），整表归提交时；变异行都走 `prove-red.sh` 证过。

### 检查单：新崩溃流多罩了什么

`crash_points_of_the_repair_of_the_units_exempt_from_the_mapping` 182 个状态加「坏盘 × 部分落盘」与卸载之后判真盘；GPU 缩域新流多罩建上一版的下界与映射 key 集合。腿核是不是真比已有的多。

## 五、格

1. 代码做的是不是条款说的。
2. 第三节六条攻击面。
3. 第四节八格逐格回「成立」「不成立」或「推不出」，推不出的算打中。
