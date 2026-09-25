# 里程碑二收尾：代码轮第一轮之后打进的三份补丁与定义改动，代码轮第二轮正文（2026-09-25）

<!-- doc-lint:not-numbers Z7 Z8 Z9 Z10 Z11 Z12 B1 B2 B3 B5 B6 Q2 Q3 Q4 Q5 Q6 -->

## 一、这一轮要判什么

代码轮第一轮（`research/prompts/m2-final-code-r1-main-verification.md`）冻结之后，又有三份补丁打进主工作区，外加一批 agent 定义改动。这些都还没走过三方：

| 补丁 | 实现员报告 | 收口表的行 |
|---|---|---|
| 实十九：记账树与中央映射树分裂、挂载态读多层映射、checker 按父条目核多层树 | `research/prompts/m2-treesplit-implementer-report.md` | 28 |
| 实二二三：发布失败原样重发、读者规则两格、释放核验两块盘一起留、按单元豁免、树表 0 条写行再跨记录、取号前准入挪前、回退见证表 | `research/prompts/m2-batch223-implementer-report.md` | 36、13、39、① |
| 实二四与实二四续：真设备二进制的抬 F 模式 | `research/prompts/m2-vm-raise-floor-implementer-report.md`、`research/prompts/m2-vm-raise-floor2-implementer-report.md` | 58 |
| agent 定义：广播规则变更、交回之后不许续做、重型测试清单与 hook 对齐 | 无实现员报告，diff 见附录 | 提案 `records/2026-09-16-subagent拆分提案.md` 第四十节 15–21 行 |

这一轮攻的是写好的代码、它的测试和定义文字。问的是它们做的是不是条款说的，不重判设计。三种结论，每格只能落一种。共用问句与三种结论的定义照第一轮正文 `research/prompts/_m2-final-code-r1-body.md` 第一节，逐字适用。

## 二、实现今天的样子（主 agent 的观测，2026-09-25 04:40 JST 现查）

- **冻结副本**：腿读代码一律读 `/tmp/claude-1000/m2-final-code-r2/tree/crates/`，别读主工作区——实二一正在主工作区里改 `allocator.rs`、`transaction.rs`、`lib.rs`、`singlefs-format/src/lib.rs`，新建 `allocation_record_tree.rs`、`extent_tree.rs`。
  - 冻结副本就是实二二三合入时的整棵树，比主工作区只少实二一的改动。
  - `src/*.rs` 共 50 份，sha256 在 `/tmp/claude-1000/m2-final-code-r2/crates-src-sha256.txt`。
- **这一轮的改动范围**：第一轮冻结树 `/tmp/claude-1000/m2-final-code-r1/tree/crates/` 到这一轮冻结树的 diff，在 `/tmp/claude-1000/m2-final-code-r2/r1-to-r2.diff`，共 43 个文件，其中 `src/*.rs` 22 份：
  - checker：`image.rs`、`lib.rs`、`walk.rs`；
  - core：`allocator.rs`、`code_two_tree.rs`（新）、`journal.rs`、`lib.rs`、`make_filesystem.rs`、`mounted_read.rs`、`mount.rs`、`recovery.rs`、`rollback_witness.rs`（新）、`system_configuration.rs`、`transaction.rs`、`write_accounting.rs`；
  - format：`lib.rs`；
  - harness：`bin/e158_root_choice_repair.rs`、`bin/first_transaction_device_log_check.rs`、`bin/first_transaction_on_device.rs`、`history.rs`、`model_comparison.rs`、`model.rs`、`on_device_modes.rs`。
- `bin/e158_root_choice_repair.rs` 是 E158 的实验装置，由那个实验页判。判决里只点名、写明不判的理由。
- **定义冻结副本**：`/tmp/claude-1000/m2-final-code-r2/defs/`（8 份），相对 HEAD 的 diff 在 `/tmp/claude-1000/m2-final-code-r2/defs-vs-head.diff`。
- **已知、已定或已派、实现还没落的**（不算打中，腿打中这几格时写「已知」并点名去向）：
  - 空间准入没接进发布与挂载路径；抬 F 被拒时扣住的槽不放开——都归实二五（C363 (b) 判决 `research/prompts/c363b-r1-main-verification.md` 第四节）；
  - 核出对不上先重读一次再隔离还没实现，故障注入两条用例 `fast_tier`、`one_fixed_history` 因此是红的——实二五；
  - 分配记录树、extent 树的多层与按位置寻址，以及树表 0 条那一版分配记录条数撞墙（C544（树表 0 条那一版写行，分配记录条数撞墙之后池挂不上可写））——实二一；
  - 原样重发把一次失败放大成整条发布流阻塞——C541，用户定照今天；
  - 实十九的 B1（一次发布先删后插）、B3（多层映射只判层级字节）、B5（重建出的分隔 key 坏了在取号前拒）、B6（叶条目宽由各读者报错），正在写成条款；B2（记账树每次发布整批换代）记欠账；
  - 回退见证的五样实现取法（删除规则、写满拒绝、前缀第五条读法、读哪几槽、落点 481 起 753 字节）正在写成条款，都被攻过零轮——这几格**欢迎攻**，打中照常算。

## 三、六格

### Z7　记账树与中央映射树分裂（实十九）

代码：`code_two_tree.rs` 整份、`transaction.rs` 里规划这一版树形那一段（`plan_the_tree_after_this_publish` 及它的调用点）、`mounted_read.rs` 读多层映射、`walk.rs` 的 `walk_code_two_subtree`、format 的两个内部条目宽常量。

压着的条款：
- D8（核心索引结构） 已定项 11「多层码 2 树怎么长、怎么收」；
- D18（块里携带什么信息） 已定项 2（子树覆盖区间）；
- D19（块指针的结构与宽度预算） 已定项 5「挂载态怎么读映射」；
- I-1.1、I-1.10。

问：
- 分裂点、分隔 key 的维护、层级字节，与条款字面一不一致；
- checker 按父条目核的四样（层级、分隔 key 递增、区间、树高），能不能在一棵合法树上误红，或在一棵坏树上放过。

### Z8　发布失败原样重发、读者规则两格、再跨记录（实二二三 a、b、f）

代码：
- `allocator.rs` 的 `freeze_publish`、`refuse_while_a_publish_is_frozen`、`resend_the_frozen_publish`；
- `transaction.rs` 写记录那一段（树表 0 条写行跨多条记录）；
- `recovery.rs` 两格断链；
- `journal.rs`。

压着的条款：D23（journal 的角色与格式） 已定项 4、14（这一版的失败处置、第六条）、17。

问：
- 冻结之后原样重发，是不是逐字节原样——checkpoint_txg、计数器、序号、标志、单元位置与字节。
- 冻结期间有没有别的写路径绕过闸。
- 读者对两格断链的判法，会不会把一条合法历史切断。

### Z9　回退见证（实二二三 g）

代码：
- `rollback_witness.rs` 整份；
- `system_configuration.rs`、`make_filesystem.rs` 的落点；
- `mount.rs` 回退挂载写见证（post）与删除规则；
- `recovery.rs` 择根跳过被抛弃的根、前缀第五条；
- `walk.rs` 的 `judge_rollback_witness_tables`、`judge_rollback_witness_against_the_chosen_roots_table`。

压着的条款：
- D23（journal 的角色与格式） 已定项 14「回退见证」（用户定 witness (a)、post、上界 R × S − 1）；
- D22（单元原子性怎么合成） 已定项 9。

问：
- 见证写在 post，崩在写见证之前、之间、之后，各盘各槽一新一旧时，择根是不是每一种都落到条款要的根；
- 删除规则会不会删掉一条还护着某条根的条目；
- 表满之前能不能先因别的原因挂不上。

### Z10　真设备抬 F 模式（实二四、实二四续）

代码：`on_device_modes.rs`、`bin/first_transaction_on_device.rs`、`bin/first_transaction_device_log_check.rs`，以及 `.claude/gate.d/55-qemu-first-transaction.sh` 里的 raise 模式（主工作区现行版）。

压着的条款：
- D23（journal 的角色与格式） 已定项 14（管理员回退、抬 F）；
- D18（块里携带什么信息） 已定项 11「可写挂载的顺序」。

问：
- 设备侧录制与程序录制的比对，在这一档判的是不是「发布 D 之后 F 抬到 3、冷重开择 (2, 11)」这件事本身；
- 宿主检查会不会在 F 没抬的镜像上也判绿。

### Z11　定义改动

材料：`defs-vs-head.diff` 的 8 份。

对照的东西：
- `.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/continuation-guard.sh`、`.claude/hooks/bash-command-detector.sh` 的现行行为（它们的 `--selftest` 能跑，不是重型测试）；
- `.claude/singlefs-ai-sop/rules/rules-discipline.md` 第 1、7 条。

问：
- 改过的定义之间，以及定义与 hook 之间，有没有说反话——比如重型测试清单与 hook 的判法不一致、「交回之后不许续做」与续做闸的判据不一致；
- 有没有一条指令照着做会撞上 hook 的拒绝；
- 有没有一句写的是为什么、而不是怎么做。

### Z12　合并点：取号之前的预演与真发

`mount.rs` 取号之前在分配器副本上预演的落点序列，与真发的落点序列，在这一轮新加的每一种形态上是不是都相同：
- 写行那次记账树或映射树分裂；
- 树表 0 条那一版末条再跨记录；
- 回退挂载写见证；
- 冻结着一次失败时挂载。

找一种形态，让 `establish_instance` 里「拷贝上取的与真发的相同」那条断言在合法历史上 panic，或者在不该相同时判相同。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端攻方（Opus） | Z8、Z9、Z12 | 找反例：造一条合法历史，让代码的行为与条款字面不同；在冻结副本的一份拷贝上写用例跑出来 |
| 云端正推（Sonnet） | Z7、Z10、Z11 | 逐格核「代码做的是不是条款说的」，三种结论每格落一种 |
| 本地攻方 | 算术 | 按事实表逐格填数，每格写出推导。攻击面只在算术上，与云端攻方不重叠。要填的格：<br>① 见证表 1 + 47 × 16 = 753、481 + 753 ≤ 4096；<br>② R × S − 1 在 R、S 取条款上界时是不是 47；<br>③ 记账树叶 / 内部、映射树叶 / 内部的每节点条目数，从节点 16384 字节、头宽、条目宽 34 / 108 / 55 / 113 推；<br>④ 两块盘的池记账行数 3 + 6 × 2 = 15 时树高是几、多少块盘起分裂；<br>⑤ 一条记录点名项上限 ⌊(4096 − 311) ÷ 56⌋ |

## 五、交付

- 腿的报告写 `research/prompts/m2-final-code-r2-<腿名>-output.md`，分段落盘。
- 攻方的用例与模型放 `research/prompts/m2-final-code-r2-opus-model/`。
- 引 kb 条款时，写 kb 文件名加那份文件自己的行号。
- 方案按 `crates/` 今天的实现来谈，也就是冻结副本。
