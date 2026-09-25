# 里程碑二收尾：实二一（分配记录树与 extent 树按位置寻址）与第二轮之后的定义改动，代码轮第三轮正文（2026-09-25）

<!-- doc-lint:not-numbers Z13 Z14 Z15 Z16 Z17 Z18 K1 K2 K4 -->

## 一、这一轮要判什么

| 改动 | 报告 | 收口表的行 |
|---|---|---|
| 实二一：分配记录树按绝对槽号按位置寻址（用户 K1）、extent 树两段按位置寻址（用户 K2）、挂载整棵读分配记录树、extent 树按需读（用户 K4）、C544（树表 0 条那一版分配记录条数撞墙）解开、15 个测试二进制按新布局重钉 | `research/prompts/m2-keyspace-implementer-report.md` | 28 |
| 第二轮之后改过的定义：`agent-common.md`（检测器拒绝的写法写成四种）、`main-agent.md`（删一句论证、看门狗叫醒条件补一条、拒绝清单补一条）、`crash-verifier.md`（旧小节名）、`implementation-workflow.md`（标题） | 第二轮判决 `research/prompts/m2-final-code-r2-main-verification.md` 第三节 Z11；diff 见附录 | — |

这一轮攻的是写好的代码、它的测试和定义文字，问它们做的是不是条款说的，不重判用户已定的 K1、K2、K4。

- **共用问句与三种结论**：照第一轮正文 `research/prompts/_m2-final-code-r1-body.md` 第一节，逐字适用。
- **可以攻的**：实现员自己取的值与做法，已由主 agent 写成 D8（核心索引结构） 已定项 14「实现取值」（被攻过零轮）。这一格欢迎攻，打中照常算。

## 二、实现今天的样子（主 agent 的观测，2026-09-25 07:50 JST 现查）

- **冻结副本**：腿读代码一律读 `/tmp/claude-1000/m2-final-code-r3/tree/crates/`（实二一交回那一刻的整棵树），不读主工作区。
  - 实二五此刻正在主工作区改 `transaction.rs`、`mount.rs`、`allocator.rs`、`recovery.rs`、`rollback_witness.rs`、`walk.rs`。
  - `src/*.rs` 53 份，sha256 在 `/tmp/claude-1000/m2-final-code-r3/crates-src-sha256.txt`；`tests/*.rs` 67 份，在 `crates-tests-sha256.txt`。
- **改动范围**：第二轮冻结树到这一轮冻结树的 diff 在 `/tmp/claude-1000/m2-final-code-r3/r2-to-r3.diff`，51 个文件、约 16800 行。
  - 新文件：`crates/singlefs-core/src/allocation_record_tree.rs`、`crates/singlefs-core/src/extent_tree.rs`、`crates/singlefs-checker/src/position_addressed.rs`、`crates/singlefs-harness/tests/second_transaction_position_addressed_trees_layer0.rs`。
  - `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 是 E158 的实验装置，由那个实验页判。
- **定义冻结副本**：`/tmp/claude-1000/m2-final-code-r3/defs/`，相对第二轮冻结那一份的 diff 在 `defs-r2-to-r3.diff`。
- **已知、不算打中的**（腿打中这几格写「已知」并点名去向）：
  - 第一个事务从 8 个单元变 12 个。`layout/01-first-txn.md` 的写清单、E142 装置、`first_transaction_regions.rs` 还没跟上，排在 E142 重跑之后；`first_transaction_region_bytes` 红一条。
  - 实二二三留下的四条红（formatted_pool 1、checker_known_bad_images 1、step_four_rollback 2）与 checker 没判「根之下没有空节点」：归实二五。
  - 已有层 0 流钉着旧布局的数：提交时崩溃验证员跑。
  - C549（分配记录树固定点 64 轮的上限没有依据）、C550（对拍模型里分配记录树节点数的上界偏松）、C551（`build_pool` 的分配器没有根环表）：已立欠账。

## 三、六格

### Z13　分配记录树按位置寻址（K1）

- **代码**：`allocation_record_tree.rs` 整份；`transaction.rs` 的 `settle_the_allocation_record_tree`、`settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file`、`build_allocation_record_tree`；checker 的 `position_addressed.rs` 与 `walk.rs` 走这棵树那一段。
- **压着的条款**：D8（核心索引结构） 已定项 14（K1 与实现取值）、D18（块里携带什么信息） 已定项 2（节点头 key 区间）、I-1.1、I-3.1、I-3.11。
- **问**：
  - 根层公式在每一种盘面上（小盘、4 GiB、1 TiB、各盘大小不同）取得对不对；
  - 缺席即全空闲，在记录删光一片叶、又重新长出来时对不对；
  - 固定点会不会在合法历史上不收拢，或者收拢到一个与真发不同的重写集；
  - checker 自己那份几何与实现的几何，是不是真的独立（只共享格式常量）。

### Z14　extent 树两段（K2）

- **代码**：`extent_tree.rs` 整份；`transaction.rs` 的 `build_extent_tree`、`plan_the_extent_tree_after_this_publish`；标签 0 / 1 / 2 的读写。
- **压着的条款**：D8（核心索引结构） 已定项 14（K2 与实现取值）。
- **问**：
  - 单单元内联 ↔ 两个及以上建下段，来回切换时旧节点是不是都释放了；
  - 下段长到两层再缩回一层，层级与区间对不对；
  - 下段 key 的字节偏移 = 单元号 × 32634，与数据单元净荷容量的口径是不是同一个。

### Z15　挂载怎么读（K4）与 C483 ②

- **代码**：`recovery.rs` 整棵读分配记录树；`mounted_read.rs` 的 `open_file`（多一个 `reader` 参数）按位置走 extent 树；位置提示过期时经映射回退。
- **压着的条款**：D8（核心索引结构） 已定项 14「挂载怎么读」、D19（块指针的结构与宽度预算） 已定项 5。
- **问**：打开文件时读几个节点的自报数，与块层数到的是不是同一个口径；提示过期、映射回退那一路，在下段多层时走不走得通。

### Z16　取号之前的预演与真发走同一段

- **代码**：`mount.rs` 的 `dry_run_of_the_publishes_after_acquisition` 与发布路径共用的 prepare；`establish_instance` 的「预演取的落点与真发逐次相同」断言。
- **问**：
  - 预演与真发共用代码之后，这条断言还剩什么判别力；
  - 找一种形态让预演放行、真发被拒，或者反过来：
    - 树表 0 条那一版写行、66 片实例表；
    - 回退到环里最旧的根；
    - 分配记录树在这一次长一层；
    - 冻结着一次失败时挂载。
  - 第二轮 Z12 偏薄的那一半（带文件、小容量）墙拆了之后，可以接着攻。

### Z17　15 个测试二进制重钉的数

实二一按新布局改了 15 个没动过的测试二进制，钉死的槽号、单元数、释放和隔离槽数、写调用数、字节数都动了，报告里写了每个数从哪来。

- **问**：这些数是按条款独立推出来的，还是照今天的实现输出抄的？后者等于自己给自己发答案。
- **怎么攻**：挑出其中取值最多的几个（第一个事务 12 个单元与 50240..=50252、`step_six_recovery` 的树表槽 50252、`many_inodes`、`admission_formula`），按 D8 已定项 14 的实现取值与 D3（空间分配） 已定项 10 的 bump 次序独立算一遍，和测试里钉的数比。

### Z18　第二轮之后的定义改动

- **材料**：`defs-r2-to-r3.diff`，以及冻结副本里的 `bash-command-detector.sh`（检测器新加的第四种拒绝）。
- **问**：
  - 改过的定义之间、定义与 hook 之间，有没有说反话；
  - 有没有一句写的是为什么、不是怎么做；
  - 检测器第四种拒绝在 `agent-common.md`、`main-agent.md` 里的说法，与 hook 的判据是不是同一条。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端攻方（Opus） | Z13、Z16、Z17 | 找反例：在冻结副本的一份拷贝上造合法历史写用例跑出来；Z17 独立算数 |
| 云端正推（Sonnet） | Z14、Z15、Z18 | 逐格核「代码（定义）做的是不是条款说的」，三种结论每格落一种 |
| 本地攻方 | 算术 | 按事实表逐格填数，写出推导：<br>① W = (16384 − 135) ÷ 20 取整；<br>② 分配记录树内部扇出 (16384 − 135) ÷ 96；<br>③ 根层公式在 240 槽、4 GiB × 2、1 TiB × 2 上各取几；<br>④ extent 上段叶 (16384 − 163) ÷ 113、下段叶 (16384 − 163) ÷ 112、内部扇出 (16384 − 163) ÷ 110；<br>⑤ 145 个单元的文件下段几层 |

## 五、交付

- 腿的报告写 `research/prompts/m2-final-code-r3-<腿名>-output.md`，分段落盘。
- 攻方的用例与模型放 `research/prompts/m2-final-code-r3-opus-model/`。
- 引 kb 条款写 kb 快照里那份文件自己的行号。
- 方案按 `crates/` 今天的实现来谈，也就是冻结副本。
