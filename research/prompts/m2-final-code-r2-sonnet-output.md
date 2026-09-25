# 三方论证正推腿（Sonnet）：`m2-final-code-r2`，判 Z7 / Z10 / Z11

三种结论定义照 `research/prompts/_m2-final-code-r1-body.md` 第一节：兑现了条款 / 替没写的条款做了选择 / 和条款说反话。

代码读冻结副本 `/tmp/claude-1000/m2-final-code-r2/tree/crates/`；Z10 的 gate 脚本、Z11 的三个 hook 读主工作区现行版（派发提示指定）；kb 引文行号取自 `/tmp/claude-1000/m2-final-code-r2/kb-snapshot/`，逐条 `grep -nF` 核过。

## Z7　记账树与中央映射树分裂（实十九）

### 分裂点、分隔 key 维护、层级字节

D8（核心索引结构） 已定项 11「多层码 2 树怎么长、怎么收」原文（`.claude/kb/decisions/08-核心索引结构.md:321`）：

> **多层码 2 树怎么长、怎么收**（记账树、中央映射树，以及照已定项 14 仍用码 2 btree 的树）：① 一次发布里分裂新建的节点与别的单元落在同一段，整条根到叶的路径照常 COW，不加写序步骤、不加屏障；② 叶装不下时从中间切；③ 收缩只摘空节点，根只剩一个孩子时降高一层，不做「低于一半合并」与借条目；④ 分隔 key 跟着维护：插到最左分隔 key 之下时把它压低，条目在兄弟间挪动之后把右边那个孩子的分隔 key 改成它新的最小 key；逐层可判：分隔 key ≤ 孩子节点头的最小 key，且 > 左邻孩子节点头的最大 key（D18（块里携带什么信息） 已定项 18 的 key 区间）；⑤ 树高 = 根节点码 2 头里的层级 + 1。

逐条对代码（`code_two_tree.rs`）：

- ① 由 `insert_below` / `delete_below`（117 行起）标记 `mark_rewritten`、写序交发布路径统一走，与条款字面一致。
- ② `split_in_the_middle`（`code_two_tree.rs:434`，`split_off` 那一行在 `code_two_tree.rs:437`）：`let right_keys = keys.split_off(keys.len().div_ceil(2));`——左半 `⌈n÷2⌉`、右半其余，函数注释自己写「从中间切：左半留 ⌈n ÷ 2⌉ 条」（`code_two_tree.rs:433`）。**兑现了条款**。
- ③ `delete_at_the_root`（`code_two_tree.rs:555`）：根只剩一个孩子时 `*root = only_child;`（`code_two_tree.rs:574`）、孩子不重写；删空的节点在 `delete_below` 里 `children.remove(child_index)`（`code_two_tree.rs:546`），没有借条目或合并逻辑。**兑现了条款**。
- ④ `route_for_insertion`（`code_two_tree.rs:412`）：找不到分隔 key ≤ 目标 key 的孩子时 `children[0].separator_key = key.clone();`（`code_two_tree.rs:419`），即「插到最左分隔 key 之下时把它压低」；`split_in_the_middle` 交回的右半 `separator_key: right_keys[0].clone()`（`code_two_tree.rs:438-439`），即「右边那个孩子的分隔 key 改成它新的最小 key」。**兑现了条款**。
- ⑤ `CodeTwoTreeShape::height`（`code_two_tree.rs:195`）注释自抄「树高 = 根的层级 + 1（D8（核心索引结构） 已定项 11 ⑤）」，实现是 `root.level + 1`（现读，未贴全函数体，属直接对应，未见偏差）。

### 两个内部条目宽常量

D8 原文续行（`.claude/kb/decisions/08-核心索引结构.md:327`）：

> **内部节点的条目 = 本树 key + 子指针 86**：记账树 22 + 86 = 108、中央映射树 27 + 86 = 113。

`singlefs-format/src/lib.rs:131` `ACCOUNTING_INTERNAL_ENTRY_BYTES: u64 = 108`（注释「分隔 key 22 + 子指针 86」）、`lib.rs:134` `CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES: u64 = 113`（注释「分隔 key 27 + 子指针 86」），与 `code_two_tree.rs:118` 的 `internal_entry_width_in_bytes(key_width) = key_width + 86` 三处对得上、逐字节相等。**兑现了条款**。


### checker 按父条目核的「四样」——实测只有三样，没有独立的树高检查

Z7 的问句写「checker 按父条目核的四样（层级、分隔 key 递增、区间、树高），能不能在一棵合法树上误红，或在一棵坏树上放过」。核对 `walk.rs` 的 `walk_code_two_subtree`（`crates/singlefs-checker/src/walk.rs:616`），文档注释（`walk.rs:605-611`）自己列的是三条：

> ① 孩子头里的层级 = 父层级 − 1（层级 0 是叶）；
> ② 分隔 key_i ≤ 第 i 个孩子头里的最小 key，且 > 第 i − 1 个孩子头里的最大 key（它管的区间就是父条目给的那一段）；
> ③ 内部节点头里的区间 = [第一个孩子头里的最小 key, 最后一个孩子头里的最大 key]。

`grep -rin 'height' crates/singlefs-checker/` 在冻结副本上零命中（现查命令：`grep -rin 'height' /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-checker/`，无输出）。坏镜像回归测试 `the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root`（`crates/singlefs-harness/tests/second_transaction_supplement_two_tree_split.rs:702`）列的 7 个用例（`second_transaction_supplement_two_tree_split.rs:712-750` 起）覆盖的是层级、两条分隔 key 不等式、区间、I-1.10 条目宽，同样没有「树高」这一类。`crates/mutations.tsv` 里以「树分裂 checker」开头的 5 行（`mutations.tsv:577-581`）逐条列的是这三样加 I-1.10，也没有树高。

树高本身没有独立存储位置：它就是根节点头里的层级 + 1（已定项 11 ⑤自身的定义），全仓没有第二个字段记录「这棵树该是几层」供 checker 去对照（`root_record.rs`、树表条目的字段表都没有 height 字段，见现查命令：`grep -n 'height\|层级' crates/singlefs-core/src/root_record.rs` 零命中）。**这是一个转述判断（不是引文）**：checker 的三条判定（层级链、分隔 key 单调、区间覆盖）已经蕴含了树高的自洽性——只要每一层的层级都等于父层级减一、最终落到层级 0 的叶，读到的根层级本来就是这棵树实际的层数；没有外部的「声明树高」字段可以与之打架，所以不存在「树高被误判」的独立攻击面。**Z7 问句本身的「四样」与代码不符，实际是三样**，这一点算作对材料问句的更正，不计入 Z7 的三种结论判定（判的是代码对不对得上条款，不是问句本身对不对）。

**推翻条件**：若日后在树表条目、根记录或别处新增一个独立的「树高」字段（例如缓存的层数），而 checker 没有加一条 judge 核对它与 `根层级 + 1` 相等，则「没有独立树高检查」的观察失效，需要重新判定那条新检查是不是也在 walk.rs 里。

### 挂载态读多层映射（D19 已定项 5）

D19（块指针的结构与宽度预算） 已定项 5「挂载态怎么读映射」原文（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:103`）：

> **挂载态怎么读映射**：第一版挂载态打开池时把中央映射整片读进挂载态（`crates/singlefs-core/src/mounted_read.rs` 的 `central_mapping_entries`），之后的解引用查这份内存里的条目、不再发映射读；中央映射长成多层时，挂载态打开池时把整棵映射树读进挂载态，解引用照旧只查内存、不为映射发设备读；……「提示过期的一次解引用发 3 次设备读」这个读数就骑在这一条上，这个读数是根兼叶时量的，多层落地时由用例重量……

`mounted_read.rs:358-367` 的 `open_pool_for_read` 用 `read_code_two_tree` 整棵读入 `mapping_tree`（同一函数管根兼叶与多层两种情形，不是分叉实现），注释（`mounted_read.rs:354-357`）逐字写「多层时整棵读进挂载态（D19（块指针的结构与宽度预算） 已定项 5「挂载态怎么读映射」）……提示过期的一次解引用 = 两条提示各试一次 + 映射落点一次 = 3，不随映射树有几层变」。**兑现了条款**。

## Z10　真设备抬 F 模式（实二四、实二四续）

### 问 1：设备侧录制与程序录制的比对，判的是不是「F 抬到 3、冷重开择 (2, 11)」这件事本身

场景本身在 `first_transaction_on_device.rs:2407-2413` 的测试注释里写死：

> `raise-rollback-floor` 模式在宿主上照同一条路跑一遍……发布 C（txg 8）之后同一次挂载里发布 D（txg 9，第四版），再把 F 抬到上限。……上限 = min(每块盘上最新的有效根, 第 4 新的非空有效根) = min(盘 1 的 txg 7, txg 3) = 3……冷恢复择 (2, 11) 读回第四版。

对应的内存内测试（`first_transaction_on_device.rs:2415` 起）里，F=3 与 ceiling=3 由 `assert_eq!(field(raise_line, "requested_floor"), Some("3"), …)`（`first_transaction_on_device.rs:2451-2455`）与 `assert_eq!(raised.current_version.root.rollback_floor, CheckpointTxg(3));`（`first_transaction_on_device.rs:2456`）核；冷重开择 (2,11) 由 `assert_eq!(root, (InstanceGeneration(2), CheckpointTxg(11)));`（`first_transaction_on_device.rs:2593`）核。**这两组断言读的是程序自己的内存状态（`raised.current_version.root`）与冷启动 `recover()` 的返回值，不是设备侧录制。**

设备侧录制与程序录制的比对在另一份文件 `first_transaction_device_log_check.rs`：`raise_rollback_floor_mode_compares_the_raise_window_and_is_red_there_when_one_step_changed`（`first_transaction_device_log_check.rs:796`）的文档注释（`first_transaction_device_log_check.rs:792-796`）写「发布 D 与抬 F 两段真在比对里……每一段里改一步（少一个写、少一个 FLUSH、最后一个写的内容变了）两块盘都判红」——它比的是**写请求与 FLUSH 的字节序列**（`compare_one_device`），不重新断言 F 的值或冷重开选中的根。

**结论**：设备侧录制与程序录制的比对本身，判的是「程序自称发出的 I/O」与「设备实际收到的 I/O」逐字节相符，不是「F 抬到 3、冷重开择 (2,11)」这件事本身；后者由另外两组独立的语义断言（`requested_floor`/`root.rollback_floor` 的进程内断言，和冷重开 `recover()` 的进程内断言）分别核。**这是替一个没写清楚的问句做的区分，不是代码违反条款**，落「和条款说反话」还是「兑现了条款」都不准确——把 Z10 问句字面拆开看，「这一档」（设备侧-程序比对）不判「这件事本身」（F=3、根=(2,11)），这两件事在真设备档的验证链上是分开的两组断言，互不覆盖。

**推翻条件**：若 `compare_one_device` 或它调用的 `events_by_window`/`device_log_that_received` 除了比较写/FLUSH 字节序列之外，还独立解析并断言了 `root.rollback_floor` 字段或 recover 返回的根，则以上判断不成立；现查未见（`grep -n 'rollback_floor' first_transaction_device_log_check.rs` 零命中）。

### 问 2：宿主检查会不会在 F 没抬的镜像上也判绿

`recover_cold` 打印行的字段表（`first_transaction_on_device.rs:1547`）：

> "name=recover_cold outcome={outcome} root={root} content_matches={content_matches} valid_records={} above_water={} applied={} verification_passed={} mapping_fallbacks={}"

字段里没有 `rollback_floor`；`.claude/gate.d/55-qemu-first-transaction.sh` 对冷重开的判据（`.claude/gate.d/55-qemu-first-transaction.sh:212`）只 grep `name=recover_cold outcome=file_read root=$cold_root content_matches=true`，同样不看 `rollback_floor`。gate 脚本对 F 值的判据落在另一行：`.claude/gate.d/55-qemu-first-transaction.sh:84` 的 `name=raise_rollback_floor floor_before=0 requested_floor=3 ceiling=3 …`——这一行由 guest 进程自己在真实设备写入之后打印（`first_transaction_on_device.rs:1153-1155`，取值 `current_version.root.rollback_floor.0`，即**同一次写入之后进程内存里的值**，不是重新独立读盘、重新 `recover()` 之后再核对的值）。

**结论**：gate 55 对「F 抬到 3」的判据是 guest 进程自己写完之后的自报行，冷重开检查（`recover_cold`）不核 `rollback_floor` 字段。若磁盘上 `rollback_floor` 字段实际编码/持久化出错（例如序列化写错偏移，但内存里的 `current_version.root.rollback_floor` 局部变量本身没错），只要 F 出错不影响本次冷重开选中哪个根（普通冷重开走「候选集里 txg 最大的有效根」，不需要读 F 就能选中 (2,11)，选根逻辑本身不依赖 rollback_floor 字段），这样的镜像会让 gate 55 的全部现有断言判绿——`requested_floor=3` 一行来自内存自报、不受盘上编码错误影响，`recover_cold` 一行不读 `rollback_floor`。这道阶段自己的注释也承认这个缺口（`.claude/gate.d/55-qemu-first-transaction.sh:3-4`）：「这一道今天只有真实负载与设备侧录制、没有崩溃注入」，即它不做 checker 全绿式的盘上语义复核。

**补一层现查**：checker 确实有一条独立核 `rollback_floor` 的检查——`judge_rollback_floor_raises_against_their_ceilings`（`crates/singlefs-checker/src/walk.rs:2509`），核 I-7.9（回退下界 F 不高于抬 F 的上限），但它只在 F **真的比上一版大**时才判上界（`if raising_root.rollback_floor <= floor_before_the_raise { continue; }`，`walk.rs:2531`）——F 该抬而没抬（例如镜像里仍是 0）时这个分支直接跳过，判不出「该抬没抬」，只判得出「抬多了」。而且这条检查属于池级 checker（`check_pool_image`），gate 55 的注释（`.claude/gate.d/55-qemu-first-transaction.sh:3-4`）自己写明这一档不跑 checker。**结论不变，而且更确定**：gate 55 现有的判据链上，一个「F 没抬」的真实设备镜像会判绿。

**推翻条件**：若 gate 55 之后接上池级 checker 全量跑这份镜像，`judge_rollback_floor_raises_against_their_ceilings` 也不会挡住「F 该抬没抬」这一类（上面已证明它的判据只单向管上界），除非另加一条判 `raising_root.rollback_floor == 预期值` 或判「抬 F 的发布之后 F 必须严格大于抬之前」的检查；现查 `walk.rs` 里没有第二条判 rollback_floor 的规则（`grep -n judge.*rollback\|check.*rollback crates/singlefs-checker/src/walk.rs` 只命中这一处 judge 函数）。

## Z11　定义改动（`defs-vs-head.diff` 的 8 份）

### 重型测试清单：6 份 agent 定义 + `main-agent.md` + `implementation-workflow.md` 与 `heavy-test-guard.sh` 逐项核

`heavy-test-guard.sh` 头部注释自己列的重型测试八类（`.claude/hooks/heavy-test-guard.sh` 现读，行号见下）与放行表：

- 八类：层 0、QEMU、herd7、crates 变异整表、全量测试、全部实验复跑（87 号）、整轮门禁、E152 装置（`heavy-test-guard.sh:26-33`）。
- 放行表：主 agent（输入里没有 `agent_type`）放行全部八类；`crash-verifier` 放行层 0、55 号、herd7、crates 变异整表，拒 vm-bench.sh / 全量测试 / 整轮门禁 / E152；`gate-triage` 放行整轮门禁与 87 号；其余子 agent 一律拒（`heavy-test-guard.sh:34-38`）。

对照 `implementation-workflow.md` 新增「重型测试只在提交时跑」一节（`/tmp/claude-1000/m2-final-code-r2/defs/.claude/rules/implementation-workflow.md:14`）：同样八类、同样的角色划分（主 agent 全跑、崩溃验证员 54/55/57/59、门禁分诊员整轮 + 87、其余子 agent 一律不跑），逐项核对一致；`agent-common.md` 新增的重型测试段（`/tmp/claude-1000/m2-final-code-r2/defs/.claude/agent-common.md:46`，即 diff 里那一行）同样八类、同样角色划分，与 hook 一致；`crash-verifier.md`（`crash-verifier.md:19`「只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）」）、`gate-triage.md`（`gate-triage.md:19`「54、55、57、59 这几道重阶段……你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）」）都点名会被同一个 hook 拒，且拒的范围与 hook 放行表相符。**这一批：兑现了条款**（hook 是既有实现、定义改动是对齐它写的说明，逐项比对没有找到不一致）。

**一处不完整、不算说反话**：`main-agent.md` 新增第 3 步（`/tmp/claude-1000/m2-final-code-r2/defs/.claude/main-agent.md:20`）：

> 3. **重型测试**（层 0 全量、QEMU、herd7、`crates` 变异整表、全量测试、整轮门禁）提交之外任务确实要跑，先弹窗问用户，同意了才跑（`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」）。

这里给的清单只有六类，漏了「87 号全部实验复跑」与「E152 装置」，且没有用「等」这类非穷尽标记（对照同一份文件第 14 行「**禁止在subagent中跑重型测试**（层 0 全量、QEMU、herd7、`crates` 变异整表、全量测试、整轮门禁）**等**」——那一行有「等」）。这句同一行末尾点了 `implementation-workflow.md`「重型测试只在提交时跑」，指向的是权威的八类全集，所以字面上不算「和条款说反话」（没有断言「只有这六类」），但六项清单本身与它引用的权威清单不完全相同，若人只读这一句而不点开链接，可能漏判 87 号与 E152 装置也要弹窗。**推翻条件**：若这句本意就是「举例，不穷尽」（如第 14 行那样），只是漏写了「等」，那么这不是缺陷而是笔误；分辨不了，两种读法都成立，故不落三种结论中的任何一种，只记差异。

### 「交回之后不再给它发消息」与续做闸判据是不是一致

`main-agent.md`（`/tmp/claude-1000/m2-final-code-r2/defs/.claude/main-agent.md:41`）新写：

> 交回按子 agent 的 SubagentHandback 消息判。**交回之后不再给它发消息**（续做闸 `.claude/hooks/continuation-guard.sh` 拒绝）……

现读 `.claude/hooks/continuation-guard.sh`（主工作区现行版）的判法（`continuation-guard.sh:10-16`）：两条任一成立就拒——① 交回过（会话记录里有一条不出错的 `SubagentHandback` 调用及其 tool_result）；② 中断过（最近一次任务通知是 failed 或 killed）。**main-agent.md 这句话说的正是条件①，与 hook 实现逐字对得上**。**兑现了条款**。

### `bash-command-detector.sh` 拒绝的「两种写法」

`agent-common.md` 改动后的句子（diff 第 29 行）：

> 项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）对两种写法在执行前拒绝：前台没有超时的等待循环、把活放出追踪（`disown`、`nohup … &` 这类）；run_in_background 里的等待循环与按模式找进程这类命令只记检出，交给主 agent 判断。

现读 `bash-command-detector.sh:2`：「起看门狗的错误写法、前台没超时的等待循环与把活放出追踪的写法在执行前拒绝，其余只记不拦」；同一份文件第 30 行自己的小标题写「拒绝三种（退出 2，stderr 写原因与出路；拒了不记检出）」——**hook 自己认三种拒绝（起看门狗的错误写法、前台无超时等待循环、放出追踪），agent-common.md 只写了后两种、没提「起看门狗的错误写法」**。这不是「说反话」（agent-common.md 没有断言「只有两种」），但同样是一处不完整枚举，与「重型测试清单」那一处是同一种毛病（省略了一类，用「两种」这个数词把省略坐实了——「两种」是一个具体计数，不像「等」那样留了缺口）。**推翻条件**：若「起看门狗的错误写法」在 agent-common.md 别处（未纳入这次 diff、我没有检查射程）另有交代，这处省略就不算漏。

### 有没有一句写的是为什么、而不是怎么做（rules-discipline.md 第 1 条）

`rules-discipline.md` 第 1 条「正文只写四样」的表格（`.claude/singlefs-ai-sop/rules/rules-discipline.md:13`）「不写」一列写着「为什么这么定：论证、取舍、推导」。`main-agent.md` 新增一句（`/tmp/claude-1000/m2-final-code-r2/defs/.claude/main-agent.md:31`）：

> 改了定义、共用约束或规则之后，当场给每个在跑的子 agent 发消息：写明改了哪一条、它手上哪一步要停掉或改做——它们沿用派发那一刻的定义，看不到后来的改动。

破折号前半「写明改了哪一条、它手上哪一步要停掉或改做」是判据 / 怎么做，破折号后半「它们沿用派发那一刻的定义，看不到后来的改动」是解释**为什么**要当场发消息（子 agent 拿到的是派发那一刻的定义快照）——按第 1 条的判据表，这半句该删、不该进正文；按第 2 条「同一段里两样都有时，把判据那一句留在正文，其余整段删掉」，这半句该删掉。**这是一句和条款字面不合的新增文字，落「和条款说反话」**：条款字面禁止的东西，改动里新写了一句。

**推翻条件**：若这半句能被读成「射程」而非「论证」——即「它们沿用派发那一刻的定义」是在划定这条规则管哪一批 agent（射程属于「不写」表里没列的、允许写的「做什么、不做什么」一类）——则不算违反。但字面上这句紧跟在「——」后面、句式是「因为 X，所以要 Y」的倒装（先说 Y 的做法，破折号引出为什么），更贴近「论证」而非「射程声明」；这一判断带有主观成分，已在正文标出。

### 位置指代与自称（rules-discipline.md 第 7 条）

对 `defs-vs-head.diff` 里全部新增行（`^+` 开头）做过 `上述|如下|本节|本表|本条|本文档|见上|见下|前面提到|同上` 的整体 grep，零命中。**没有找到违反第 7 条的新增文字**。

### 有没有一条指令照着做会撞上 hook 的拒绝

逐条核对 8 份改动里新写的、会被 Bash 执行的命令样式：`crash-verifier.md`「`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`」（`crash-verifier.md:19`）、`gate-triage.md`「`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`」（`gate-triage.md:19`）、`mutation-triage.md`「crates 那张表你不跑……由崩溃验证员跑门禁 59 号」（不产生新命令）。三条命令的 agent 身份与前缀值都落在 `heavy-test-guard.sh` 放行表内（`heavy-test-guard.sh:34-38`，crash-verifier 放行 54/55/57/59、gate-triage 放行整轮门禁），**没有找到一条指令照着写会被 hook 拒绝**。

**推翻条件**：`gate-triage.md` 第 2 步原文还有一句「用户要求时两处都换成 `=user-request`」——若某处遗漏把前缀换成 `=user-request` 而仍写 `=commit`，在非提交场合执行会被拒（hook 只认 `commit`/`user-request` 两个值且要求与场合匹配，见 `heavy-test-guard.sh` 的「谁、带什么才放行」一段）；这是使用者按提示填错值的风险，不是定义文字本身与 hook 矛盾，不计入判定。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Z7　分裂点/中间切 | 兑现了条款 | `split_in_the_middle` 左半 ⌈n÷2⌉、右半其余，与已定项 11 ② 逐字对得上（`code_two_tree.rs:434-437`） |
| Z7　分隔 key 维护 | 兑现了条款 | 插到最左压低（`code_two_tree.rs:419`）、右半分隔 key 取新最小 key（`code_two_tree.rs:438-439`），与已定项 11 ④ 一致 |
| Z7　收缩/降高 | 兑现了条款 | 只摘空节点、根剩一个孩子降高不重写（`code_two_tree.rs:546,574`），与已定项 11 ③ 一致 |
| Z7　两个内部条目宽常量 | 兑现了条款 | 108／113 与已定项 11 续行「记账树 22+86=108、中央映射树 27+86=113」逐字节相等（`08-核心索引结构.md:327`） |
| Z7　checker「四样」 | 转述/更正 | 代码与坏镜像测试只覆盖三样（层级、分隔 key 两条不等式、区间），没有独立的树高检查；不落三种结论，是对问句本身的更正 |
| Z7　挂载态读多层映射 | 兑现了条款 | `mounted_read.rs` 整棵读入、注释自引 D19 已定项 5，3 次设备读的口径不随层数变（`mounted_read.rs:354-357`） |
| Z10　设备-程序比对判的是不是「F=3、择(2,11)」本身 | 转述/更正 | 该比对只判写/FLUSH 字节序列相符，F 与冷重开根的语义断言在另外两组独立断言里（`first_transaction_on_device.rs:2451-2456,2593`）；问句把两件事当一件事问 |
| Z10　宿主检查会不会在 F 没抬的镜像上判绿 | 打中（转述判断） | `recover_cold` 字段表没有 `rollback_floor`（`first_transaction_on_device.rs:1547`），gate 55 判据只读程序自报行（`55-qemu-first-transaction.sh:84`）；checker 唯一相关检查只在 F 真的变大时才判上界（`walk.rs:2531`），F 该抬没抬时不触发 |
| Z11　重型测试八类清单 | 兑现了条款 | 6 份定义 + `implementation-workflow.md` + `heavy-test-guard.sh` 的八类与角色划分逐项一致 |
| Z11　`main-agent.md` 第 3 步六类清单 | 转述判断 | 同一文件另一行有「等」、这一行没有，且指向的权威清单是八类；两种读法都成立，不落三种结论 |
| Z11　「交回之后不再发消息」 | 兑现了条款 | 与 `continuation-guard.sh` 条件①（交回过即拒）逐字对得上 |
| Z11　「两种写法在执行前拒绝」 | 和条款说反话 | `bash-command-detector.sh` 自称「拒绝三种」（含「起看门狗的错误写法」），`agent-common.md` 只写了两种 |
| Z11　rules-discipline 第 1 条 | 和条款说反话 | `main-agent.md:31`「它们沿用派发那一刻的定义，看不到后来的改动」是论证句，第 1 条「不写」栏明令不写这一类 |
| Z11　rules-discipline 第 7 条 | 兑现了条款 | 全部新增行 grep 位置指代与自称词零命中 |
| Z11　照做会不会撞 hook | 兑现了条款 | 三条新命令样式都落在 `heavy-test-guard.sh` 放行表内，没找到会被拒的指令 |

## 没做什么

- 不判别的腿的格（Z8、Z9、Z12 归云端攻方；本地攻方的算术格）。
- 不替主 agent 采纳或出判决，「打中」「转述判断」等标注只是我这条腿自己的三种结论落格，不是最终结论。
- Z7：`code_two_tree.rs` 里 `read_code_two_tree`（读盘重建一棵树、按父条目核层级与区间）与 `flatten`/`flatten_read` 的按层摊平细节没有逐行核对，只读了文档注释与关键函数签名；`transaction.rs` 的 `plan_the_tree_after_this_publish` 调用点（`transaction.rs:633` 附近，写落点、装字节那一段）没有走读，因为它落在实二一正在改的文件旁边、这一格的问句聚焦规划逻辑本身。
- Z7：「实十九的 B1（一次发布先删后插）」按背景材料「已知、已定或已派」清单标「正在写成条款」，我只指出 `code_two_tree.rs:622-624` 的注释引了已定项 13（状态折叠）作为先删后插排序的依据、两者字面上讨论的不是同一件事，没有把这一点单独判成三种结论之一，按材料要求写「已知」处理。
- Z10：D18（块里携带什么信息） 已定项 11「可写挂载的顺序」只读了原文（`18-块里携带什么信息.md:314`），没有对照 `on_device_modes.rs` 里 `second-instance` 模式的取号/写行/暖机代码逐条核对，因为 Z10 的「问」聚焦 `raise-rollback-floor` 那一档、可写挂载顺序主要覆盖 `second-instance` 档（第一轮 Z3、Z4 的射程），这一轮不重判。
- Z10：没有实际起 QEMU 复跑 `.claude/gate.d/55-qemu-first-transaction.sh`，只静态读了脚本与 `first_transaction_on_device.rs` / `first_transaction_device_log_check.rs` 的源码；「F 没抬的镜像会判绿」是从判据链的静态结构推出来的，没有构造一份真的「F 没抬」镜像去实测（这类构造与实测按分工属于云端攻方 Z8/Z9/Z12 或本地攻方的活，我的角色是逐格核代码与条款，不建反例装置）。
- Z11：`runner-dispatch-guard.sh`（implementation-workflow.md 提到但材料没有点名要我对照的第四个 hook）没有读；`.claude/rules/three-way-inference.md` 里关于续做闸的另一句表述（只提条件②「failed/killed」）不在材料点名的对照范围内，没有据此认定 `main-agent.md` 与它冲突——那句话所在的文件不是 Z11 的「对照的东西」清单成员。
- Z11：8 份改动里 `experiment-runner.md`、`implementation-writer.md`、`mutation-triage.md` 的改动逐行读过，没有发现与三个 hook 或 rules-discipline 第 1、7 条相关的新问题，因此报告没有单独列出它们的段落。
