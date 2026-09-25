# 里程碑二收尾：HEAD 之后打进的四份补丁，代码轮第一轮正文（2026-09-25）

<!-- doc-lint:not-numbers Z1 Z2 Z3 Z4 Z5 Z6 -->

## 一、这一轮要判什么

HEAD（`e980a219`）之后打进主工作区的四份补丁，一轮三方都没走过（`.claude/rules/implementation-workflow.md` 三步里的第 2 步）：

| 补丁 | 实现员报告 | 收口表的行 |
|---|---|---|
| 实十八：抬 F 失败的账、取号前在分配器副本上预演、211 行变异 | `research/prompts/m2-followups2-implementer-report.md` | 58、39 |
| 实十四：I-7.9、I-9.15（blocks）、重建与回退 | `research/prompts/m2-checker3-implementer-report.md` | 26、46、13（N2） |
| 实二十：末条标志、末条再跨记录、锚点读法乙、I-8.9、层 0 流 L8 | `research/prompts/m2-lastflag-implementer-report.md` | 36、T6 |
| 实十六接续：释放前读盘核（N1）、核出坏的留在已分配（N3）、删政策不一致计数（C515）、实例表第二片写路径 | `research/prompts/m2-writepath-implementer-report.md` | 13、5、38 |

这一轮攻的是**写好的代码与它的测试**：代码做的是不是条款说的。不重判设计。

共用问句：

> **这一处代码的行为，是从哪条已定分项推出来的？推不出的那些，它是在替一条没写的条款做选择吗？如果是，这个选择今天有没有会红的东西钉着？**

三种结论，每格只能落一种：

| 结论 | 什么样 | 要交出什么 |
|---|---|---|
| **兑现了条款** | 行为是某条已定分项的字面后果 | 那条分项的**整段原文**，加从原文到这段代码的那一步 |
| **替没写的条款做了选择** | 不同实现会做出不同的、都说得通的选择 | 那个选择是什么、它影响哪些字节或哪条可达历史、今天有没有会红的东西钉着 |
| **和条款说反话** | 代码做的与某条已定分项的字面相反 | 两边各自的原文，以及这次差异在哪个字节 / 哪条历史上看得出来 |

## 二、实现今天的样子（主 agent 的观测，2026-09-25 01:29 JST 现查）

- 腿读代码一律读冻结副本 `/tmp/claude-1000/m2-final-code-r1/tree/crates/`（主工作区在腿跑着的时候还会被实十九的补丁改）；它的 `src/*.rs` 共 48 份，sha256 在 `research/prompts/m2-final-code-r1-snapshot/crates-src-sha256.txt`。
- HEAD 之后 `crates/*/src/*.rs` 里改过的 19 份（`git diff --stat HEAD`）：
  - checker：`image.rs`、`lib.rs`、`walk.rs`；
  - core：`allocator.rs`、`instance_table.rs`、`journal.rs`、`mount.rs`、`mounted_read.rs`、`recovery.rs`、`transaction.rs`、`write_accounting.rs`；
  - harness：`bin/e158_root_choice_repair.rs`、`bin/first_transaction_on_device.rs`、`bin/first_transaction_region_bytes.rs`、`fault_injection.rs`、`history.rs`、`model.rs`、`model_comparison.rs`、`scenario.rs`。
- `bin/e158_root_choice_repair.rs` 是 E158 的实验装置，它对不对由那个实验页的变异表与单测判，不在这一轮的六格里。判决里只按路径点名，写明为什么不判。
- 实现员在各自副本里合过三次主工作区（报告各自第八节）。合并点是这一轮最容易出错的地方：`mount.rs` 的 `placements_of_the_publishes_after_acquisition_on_a_copy`、`placements_taken_by`、`establish_instance` 里的「拷贝上取的与真发的相同」那条断言，三份补丁都动过。
- 已知、用户已定、实现还没落的（**不算打中**，腿打中这几格时写「已知」并点名去向）：
  - 只一块盘那一份核出坏时，今天在写之前拒绝；用户 2026-09-25 定「两块盘一起留」，实现随实二二；
  - checker 的 I-3.11、I-3.1 对隔离记录的豁免读法，用户 2026-09-25 定「核出坏的才豁免」，实现随实二二；
  - 树表 0 条那一版写行，片数 ≥ 67 时一条记录装不下、`ByteWriter` 越界，照 D23（journal 的角色与格式） 已定项 17 接再跨记录，随实二二；
  - 读者规则两格（锚点读得出时首条序号不是 1、末条标志之后还有同 txg 的记录）当损坏断链，用户 2026-09-24 定，实现随实二二；
  - 发布失败原样重发，用户 2026-09-24 定，实现随实二二；
  - 写行那次的释放核验挪到取号之前，用户 2026-09-24 定，实现随实二三。

## 三、六格

### Z1　末条标志、末条再跨记录、锚点读法乙（实二十）

`transaction.rs` 的 `roles_named_by_each_record_of_the_publish`、`transaction_offset_of_each_record_of_the_publish` 与写记录那一段；`recovery.rs` 按标志认发布边界、按读法乙取锚点、一次发布之内三种断链；`journal.rs` 解析器把序号 0 与标志其余位非 0 当损坏。压着 D23（journal 的角色与格式） 已定项 4、7、14（注 1、第六条）、17，I-8.9（一次发布的记录序号连续且只有末条带标志）。

### Z2　释放前读盘核与隔离（实十六接续 N1、N3）

`transaction.rs` 的 `copies_failing_the_release_checksum_check`（判定次序、重读一次、清单怎么传进发布）；`allocator.rs` 的 `release_leaving_the_record_allocated_on`。压着 D19（块指针的结构与宽度预算） 已定项 5「硬规则 1 的读盘核读不出、核出对不上时怎么办」那一段。

### Z3　实例表多片写路径（实十六接续）

`instance_table.rs` 分片与链指针；`transaction.rs` 的 `instance_table_page_roles_in_bump_order`、`build_instance_table_chain`、`instance_table_chain_to_release`；`mount.rs` 取号之前按片数的准入与逐片核旧链。压着 D3（空间分配） 已定项 10 ⑤、D18（块里携带什么信息） 已定项 11（尾片先、一片写满再开下一片、写行整条链 COW 重写）。

### Z4　取号之前在分配器副本上预演、抬 F 失败的账（实十八）

`mount.rs` 的 `placements_of_the_publishes_after_acquisition_on_a_copy` 与 `establish_instance` 的断言；抬 F 那一串发布失败时的错误成员与已落盘次数。压着 D18（块里携带什么信息） 已定项 11「可写挂载的顺序」（第五个合取：副本上预演取得到全部落点）、D16（发布语义） 已定项 1、C516（抬 F 那一串发布被拒时前面几次已落盘）。

### Z5　checker 的 I-7.9、I-9.15 与重建回退（实十四）

`walk.rs` 里 I-7.9（回退下界 F 不高于抬 F 的上限）、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉） 的判定；`recovery.rs` 的 `rebuild_version` 在回退之后怎么取上一版。压着 `.claude/kb/invariants.md` 那两行、D8（核心索引结构） 已定项 6、D23（journal 的角色与格式） 已定项 14。

### Z6　三份补丁的合并点

同一段 `mount.rs` 被实十八、实二十、实十六接续三次改过，实现员在副本里做过语义合并（实十六接续报告第三节末「合并新底时补的一处」）。问：合并之后，拷贝上预演的落点序列与真发的是不是在**每一种**写行形态上都相同——带文件的一版与树表 0 条的一版、单片与多片、单条记录与末条再跨记录、回退到环里最旧的根。找一种形态让 `establish_instance` 那条断言在合法历史上 panic，或让它在不该相同时判相同。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端攻方（Opus） | Z1、Z4、Z6 | 找反例：造一条合法历史让代码的行为与条款字面不同，在冻结副本上写用例跑出来 |
| 云端正推（Sonnet） | Z2、Z3、Z5 | 逐格核「代码做的是不是条款说的」，三种结论每格落一种 |
| 本地攻方 | 算术 | 按事实表逐格填数：一片装几行（369）、一条记录装几个点名项（⌊(4096 − 311) ÷ 56⌋）、多少行时树表 0 条那一版的写行记录装不下、末条再跨记录时第 N 个事务落到第几条记录；每格写出推导。攻击面只在算术上，与云端攻方不重叠 |

## 五、交付

腿的报告写 `research/prompts/m2-final-code-r1-<腿名>-output.md`，分段落盘；攻方的用例与模型放 `research/prompts/m2-final-code-r1-opus-model/`。引 kb 条款写 kb 文件自己的行号。方案按 `crates/` 今天的实现来谈。
