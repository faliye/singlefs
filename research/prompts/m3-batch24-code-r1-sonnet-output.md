# m3-batch24-code-r1 正推腿报告（Sonnet，格 K1 与 K3）

轮：m3-batch24-code-r1（删除批第 24 批合入的代码，代码轮第一轮）。立场：正推（从条款与 `crates/` 今天的实现推出「应该是什么」，再与被判代码逐格比）。
被判：冻结副本 `/tmp/claude-1000/m3-batch24-code-r1/tree/` 的 10 份。下文引代码一律写冻结副本全路径加行号，引 kb 写 kb 文件自己的行号（都是此刻 `grep -nF` 现取的）。
模型与脚本：`research/prompts/m3-batch24-code-r1-sonnet-model/`；`rerun.sh` 在空目录 `rerun3` 里从头跑通，末行 `SUMMARY_MATCHES lines=393`（见第 9 节）。

## 0 开工核对

- 冻结副本 10 份 sha256 与快照清单逐份相同（我的仓副本与三次空目录复跑都核了）：「SHA256_OK_OF_10 10」（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:2`）。
- 开跑前 `ps -o pid,args -u "$(id -u)"` 没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`；有别的会话的 `cargo` 在跑，我的命令都加 `nice -n 19` 并经 `run-with-memory-cap.sh`（编译 6G、跑 4G，显式写，不用 auto），线程 10。
- 每份相对第 24 批开工前的行数变化（用 K、G 实现员留下的 `before` 副本，`git diff --no-index --shortstat`；`before` 在实现员草稿目录里，不进模型）：walk.rs +183 −11；e142 bin +70 −31；crash_judge_checker.wgsl +81 −1；crash_judge_checker_scan.wgsl +13 −3；crash_judge_gpu.rs +5 −2（去掉 `///` 行后 diff 为 0 行，纯文档）；GPU 对拍用例 +499 −29；checker_known_bad_images.rs +532；pool_checker_judges… +215 −1；reuse_after_raising_the_floor.rs +57 −2。mutations.tsv 见 N7。

## 1 N1　CPU：容器树与 inode 树按孩子头里的类标签认孩子

### 1.1 条款与判决推出的「应该是什么」

- I-9.2：「inode 树内部节点的每条条目：类型段 ∈ {0, 2}；类型段 0 ⇒ 子单元偏移 6 = 2 且身份引用的出生树、容器号、出生代三段为 0；类型段 2 ⇒ 子单元偏移 6 = 3」（`.claude/kb/invariants.md:289`）。
- I-9.17：「谱系树（D18（块里携带什么信息） 已定项 24）的每条条目：类型段 ∈ {0, 5}；类型段 0 ⇒ 子单元是码 2」（`.claude/kb/invariants.md:304`）；I-9.23 同形（`.claude/kb/invariants.md:310`）；两行都写着「身份不符单独判定，不再记走读失败」（`.claude/kb/invariants.md:304`）。
- 上一轮判决第三节第 1 条：「判孩子类标签只读单元头；类标签与类型段不符判 I-9.17 / I-9.23、不再往下走、不记走读失败」（`research/prompts/m3-batch22-code-r1-main-verification.md:39`）。
- 由此推出：①类标签与类型段说的类不同，且孩子按它自己那一类的长度读回、整单元校验和对得上 ⇒ 只红目标（I-9.2 / I-9.17 / I-9.23），不连带 I-2.1 与走读失败；②类标签坏到按自己那一类的长度对不上校验和 ⇒ 不能算「另一类」，读不出归 I-2.1；③类型段不在 {0,2}/{0,1}/{0,5} 里 ⇒ 判红。

### 1.2 实现今天的样子（CPU）

- 认类的函数：「fn child_unit_class_other_than_the_entry_type_names(」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:4086`）。逐条位置条目：头按 16 KiB 读；「if Some(class_in_the_header) == class_the_entry_type_names {」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:4098`）交回 None 走老路；另一类登记过的码（1、3 按 32 KiB，2 按 16 KiB）且「.is_some_and(|content| crc32_castagnoli_table(&content) == location.checksum);」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:4106`）才交回那一类；否则看下一条位置条目。
- 容器树（墓碑、谱系）：「if let Some(class_the_entry_type_names) = class_the_entry_type_names {」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2184`）里调它，不符就走「fn judge_a_container_tree_entry_whose_child_is_of_another_class(」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2258`）：按孩子那一类占的槽登记引用、判 `identity_invariant` 红、`continue`，函数里没有 `walk_failures.push`。类型段不认得的那支「ContainerTreeEntryType::Unregistered(_) => None,」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2182`）不进新判，仍按「let span_in_slots = if node.level >= 2 { 1 } else { 2 };」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2240`）按层级登记。
- inode 树：「InodeInternalEntryType::Unregistered(_) => None,」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2488`），其后「if let Some(child_class) = child_unit_class_other_than_the_entry_type_names(」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2490`）；不符判「self.judgements.judge("I-9.2", false, || {」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2503`）、按孩子那一类登记、`continue`。类型段不认得时没有「应是」的类，孩子自证得过就算。

### 1.3 比对：我自己造的实测矩阵（CPU，harness 档）

探针 `probes/zs_probe_tail.rs`：四种树（墓碑层级 1、谱系层级 1、inode 层级 1、墓碑层级 2）× 根第一条条目类型段（0 且身份清零、0 且身份不动、本树容器类型、7、另一棵树的容器类型、容器类型且身份清零）× 孩子头里类标签（不动、1、2、3、9、0；改 6 号字节、不重封头、整单元校验和按孩子实际长度沿引用链补）= 144 格。每格先由条款推「目标该不该红」「孩子自证得过时是不是只红目标」，再与 `check_pool_image` 实际比。

- 144 格里 117 格与条款推的一致；「ZS_SUMMARY rows=144 deviations=27」（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:257`）。
- 27 格偏离分两类，我都追到了根：
  - 18 格 `DEVIATION_EXTRA_RED`，全是类标签改成 1 的格，多红的只有 I-1.11（映射 key 里孩子类是 3、头里成了 1，那一条自己的判定）。不是本批引入，也不是 N1 的错。例（墓碑层级 1、类型段 1、标签 1）：「ZS_CPU TombstoneLevelOne type=1 zero_identity=false tag=Some(1) verifies_as=Some(1) expected_target_red=Some(true) reds=["I-1.11", "I-9.23"] DEVIATION_EXTRA_RED」（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:123`）。
  - 9 格 `DEVIATION_TARGET`，全在 inode 树：类型段 0（身份不清零）、7、1 三种，孩子类标签改成 2、9、0（孩子按自己的类标签读不出）时，I-9.2 不红，只红 I-2.1 与走读失败那一串。例：「ZS_CPU InodeLevelOne type=0 zero_identity=false tag=Some(2) verifies_as=None expected_target_red=Some(true) reds=["I-1.11", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.2", "I-7.4"] DEVIATION_TARGET」（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:192`）；类型段 7 同形（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:204`）。同一组格在墓碑树上 I-9.23 红：「ZS_CPU TombstoneLevelOne type=7 zero_identity=false tag=Some(2) verifies_as=None expected_target_red=Some(true) reds=["I-1.11", "I-9.23"]」（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:130`）。
- 机理（读码，不是推测）：容器树类型段 0 那一支先用条目自己的身份三段判「0 / 0 / 0」，孩子读不出时那一小句放过——「let entry_holds = birth_tree == 0」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2208`），类型段不认得的那支不看孩子就判红（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2246`）。inode 树先读孩子，读不出就记走读失败 `continue`：「.push("inode 树的孩子两份都读不到对得上的".to_string());」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2529`），I-9.2 的判（身份三段、类型段不认得）都排在读成功之后。所以 inode 树里「身份不为 0 的类型段 0」「类型段 7」两种条目，只要孩子读不出，I-9.2 不判。这一格 GPU 与 CPU 一样（见 N5），且老代码就是这样，不是第 24 批引入的；但 N1 写「inode 树 I-9.2 同法改」，这一格没被「同法」罩到。条款字面（`.claude/kb/invariants.md:289`「类型段 ∈ {0, 2}」「三段为 0」）这两种条目该判红。

### 1.4 N1 判定

- 一致：类标签与类型段不符、孩子按自己那一类自证得过的所有格（容器树、inode 树、三种类型段、孩子类 1 / 2 / 3），CPU 只红目标、登记引用按孩子的类、不记走读失败。矩阵里这些格 0 偏离（类标签 1 的 I-1.11 是另一条自己的判定）。
- 一致：类标签就是类型段说的那一类、或孩子按自己的长度自证不过，都回老路，读不出归 I-2.1（矩阵里 tag=2/9/0 的墓碑与谱系层级 1 行红 I-1.11、I-4.8、I-7.2、I-7.4，不红 I-2.1：容器树类型段 0 那一支读孩子用的是不判任何不变量的 `read_unit_without_judging`（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2206`），走读失败来自「if node.level <= 1 {」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2296`）；这是老路的样子）。
- 线索（不改判定）：inode 树读不出孩子时 I-9.2 不判（上一段，9 格），与容器树不对称。
- 推翻 N1「一致」的现象：在冻结副本上造一张「孩子自证得过、类与类型段不符」的镜像，CPU 多红 I-2.1 / I-4.8 / I-7.2 / I-7.4 或少红目标。我的矩阵里没有这样的格；`rerun.sh matrix` 重跑可复现。

## 2 N2　坏镜像与变异（CPU 侧的 harness 档）

### 2.1 每份新坏镜像是不是只红目标

- 新加 11 条（checker_known_bad_images）与 6 条（pool_checker_judges…）都在我的仓副本跑绿：「test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 110 filtered out」（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:53`）、「test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 39 filtered out」（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:62`）。除 `an_inode_leaf_whose_class_tag_alone_says_index_node_is_read_at_the_length_of_its_entry_type` 一条外（它断言 I-9.2 违例说明含「（类 2）不符」且 I-3.1 不红），每条的断言是 `violated_invariants(..) == [目标]`（M2、M6、M7 三条按名排序后与带连带的集合比，连带项就是条款推出的 I-9.10、I-9.29、I-9.20），没有一条只断言「目标红」。
- 数目核对：新加的 `#[test]` 数用命令数：`grep -c '^#\[test\]'` 在 before 与冻结副本 checker_known_bad_images.rs 上是 110 → 121（+11），pool_checker… 是 39 → 45（+6），GPU 对拍是 97 → 102（+5），reuse 是 16 → 16。K 的报告写「新加 12 条用例」，数出来是 11 条；K 报告点名的形状（N4-a、N4-i、N4-c、谱系类型段不认得、inode 两形算两条、N4-f、N4-j、M1、M5、只改类标签的叶容器）合起来也是 11 条，所以「12」是数错了，没有漏交的用例。
- 队列同号两条归 I-1.1：坏镜像只红 I-1.1、I-9.24 报成立（用例 `a_deletion_queue_holding_one_number_twice_reddens_only_the_key_order_invariant` 跑绿）。机理：主走读按「pub fn check_index_node_keys(view: &IndexNodeView, schema: KeySchema) -> Result<(), Verdict> {」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/lib.rs:607`）逐条判严格递增、挂在 I-1.1 上。条款这一侧：I-1.1 那一行（`.claude/kb/invariants.md:26`）里没有「码 2 节点条目按 key 严格递增」这句（`sed -n 26p .claude/kb/invariants.md | grep -c '条目按 key 严格递增'` 输出 0；同一行里 `deleted_inodes` 也是 0 次）；kb 里只有 I-9.16 那一行顺口提到「与 checker 在树表上判的 I-1.1（块头自述逻辑地址） 那组 key 严格递增是同一个谓词」（`.claude/kb/invariants.md:303`）。判决第三节第 3 条写「查清，没有就记欠账」（`research/prompts/m3-batch22-code-r1-main-verification.md:41`）：实现这一侧查清了（有一个谓词管），条款这一侧没有一句话把 `deleted_inodes` 同号两条归给 I-1.1。判：规则没说（半）。

### 2.2 每行新变异改坏的是不是名字说的那一处、红在哪条断言

K 的 21 行里 harness 档 19 行，我逐条施加到副本、跑点名的测试、还原（还原后 10 份 sha256 复核全对）。19 行全红（`exit=101`），红的位置全是名字说的那一处，下表「变异后实际红集」是 `left`，「用例期望」是 `right`（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:88-106`）：

| 序 | 名字说的那一处 | 红在哪条断言：变异后实际 → 用例期望 |
|---|---|---|
| 0 | 容器树不认孩子的类（整段新判关掉） | 只红集合断言：[I-4.8, I-7.2, I-7.4, I-9.23] → [I-9.23] |
| 1 | 类型段 0 那一支不认孩子的类（身份清零，墓碑） | [I-4.8, I-7.2, I-7.4] → [I-9.23]（I-9.23 报成立，正是上一轮坐实的漏判形） |
| 2 | 同上，谱系侧 | [I-4.8, I-7.2, I-7.4, I-9.17] → [I-9.17] |
| 3、4 | 类型段 = 容器类型那一支不认孩子的类（根环转没转） | [I-2.1, I-3.1, I-3.11, I-4.8, I-5.1, I-7.2, I-7.4] → [I-9.23] |
| 5、6 | inode 树类型段 0 当孩子该是码 3 / 不符当相符 | 同上七条 → [I-9.2] |
| 7 | 孩子的类不验整单元校验和 | I-9.2 违例说明要含「（类 2）不符」那一断言（不是集合断言） |
| 8、9 | M1：类型段 0 身份恒真 / 谱系类型段不认得不判红 | [] → [I-9.23] / [] → [I-9.17] |
| 10 | M5：容器号沿叶序递增恒真 | [] → [I-9.22] |
| 11 | M2：I-9.29 不收墓碑的出生代 | [I-9.10] → [I-9.10, I-9.29] |
| 12、13 | M3：死亡 txg 上界 / M4：倒序放过 | [] → [I-9.20] / [] → [I-9.4] |
| 14、15 | M6 出生代 / M7 整对象区间 | 少了 I-9.18：[I-9.10, I-9.29] → [I-9.10, I-9.18, I-9.29]；[I-9.20] → [I-9.18, I-9.20] |
| 16 | 队列同号：key 只要求不减 | [] → [I-1.1] |
| 17、18 | 新九条理由：I-9.23 / I-9.18 不写专门理由 | NotApplicable("这个镜像上没有它判的对象") → 专门那一句 |

- 序 1 与序 2（`crates/mutations.tsv:2878`、`crates/mutations.tsv:2879`）、序 3 与序 4 各共用同一条「原文 → 替换文」，只换点名的测试；这是两棵树、两种根环状态各一条证红，不是重复。
- 序 7 红在细节串而不是集合：它防的是「只改类标签、不重封的叶容器被当成码 2 孩子」，用例 `an_inode_leaf_whose_class_tag_alone_says_index_node_is_read_at_the_length_of_its_entry_type` 的第二条断言只要求 I-3.1 不红，不是「只红目标」；名字也没写只红。
- K 的另 2 行（E142 导出 bin 的内联单测，`crates/mutations.tsv:2894`、`crates/mutations.tsv:2895`）与 G 的 6 行不在这 19 行里：前两行属 checker 档、后六行要显卡，见 N4、N7。
- 判定：一致（19 行 19 红、红在名字说的那一处）；队列同号两条的归属条款没说。推翻：任何一行在冻结副本上变异后点名测试仍绿；`rerun.sh mut` 可复现。

## 3 N3　新九条的专门不适用理由与慢档用例豁免

### 3.1 应该是什么

- 缺省理由那一句：「.unwrap_or("这个镜像上没有它判的对象")」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/image.rs:358`）；条款要求不适用必须带理由：「不适用（必须带理由，不许静默算成立）」（`.claude/kb/invariants.md:6`）。缺省理由分不开「没有对象」与「一次都没判」，所以九条里一次都没判到时要有各自专门的一句；用例豁免要每条只认那一句、别的不适用（含缺省）照红，I-5.5 那一支原来的专门理由不能改回去。

### 3.2 八条（十句）各在哪一支报（冻结副本 walk.rs）

| 不变量 | 常量 | 在哪报 |
|---|---|---|
| I-9.18 | `NO_QUEUED_INODE_IN_ANY_WALKED_VERSION`（定义 9289） | 墓碑与谱系那一遍末尾（`judge_tombstones_and_the_lineage_of_the_versions`，9597 起）：9686 |
| I-9.19 | `NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD` | 同一处：9690 |
| I-9.20 | `NO_TOMBSTONE_IN_ANY_WALKED_VERSION` | 同一处：9694 |
| I-9.29 | `NO_INODE_RECORD_OR_TOMBSTONE_ON_THE_LINEAGE_TO_COMPARE` | 同一处：9698 |
| I-9.21 | `NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER`（旧入口）/ `PRODUCT_TOMBSTONE_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOB`（新入口给了旋钮） | 同一处的 `match capacity_knob`：9704 / 9714 |
| I-9.22 | `NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY` / `NO_TOMBSTONE_CONTAINER_UNDER_THE_KNOB` | 同一处：9708 / 9718 |
| I-9.17 | `NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION` | `check_pool_image_given_the_tombstone_container_capacity_knob` 里：10945 |
| I-9.23 | `NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION` | 同一处：10949 |
| I-9.24 | 没动：「"走读到的每一版树表里都没有 deleted_inodes 条目（树表 0 条）：没有队列可判"」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:8805`） | 形式化线 34-1 改的那一段里，内联 |

- 十个新常量的文字一个字没改：我用 K 的 `before` 副本逐句数，每一句在 before 的 walk.rs 里恰好作为字面量出现 1 次、在冻结副本里恰好作为常量字面 1 次（一次性核对，输出 11 行 `before_hits= 1 now_literal_hits= 1`，含形式化线那条 `NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK`）。harness 里 `checker_known_bad_images.rs` 自己还留着这几句的字面（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:1891` 起），常量被改字会在那里红。
- `Judgements::not_applicable` 不改已评估过的：「这个镜像上整条不适用（例如第 0 代根下面没有记账树）。已经评估过的不改。」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/image.rs:307`），所以判到过的版本照判定报，专门理由只在一次都没判到时出现。

### 3.3 抬 F 用例的豁免

- 豁免表是六对（不变量 → 只认的那一句）：「const DELETION_INVARIANTS_WITHOUT_AN_OBJECT_ON_A_SESSION_WITHOUT_DELETION: [(&str, &str); 6] = [」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs:49`），I-9.18、I-9.19、I-9.20、I-9.21（旧入口那句）、I-9.22（旧入口那句）、I-9.23；比较用 `assert_eq!`，不是 `contains`：「InvariantVerdict::NotApplicable(reason),」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs:381`）。I-9.17、I-9.24、I-9.29 不在表里，落到末尾的 `Holds` 断言：用例注释写明「回退 D 写出了谱系树，I-9.17 在 E 之后的镜像上真被评估过、要成立」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs:378`）。
- I-5.5 那一支没动：「if *invariant == "I-5.5" {」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs:315`）仍只认「singlefs_checker::walk::NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs:326`）；第 24 批的 diff 只在 `I-4.4` 那支之后插入豁免块，没碰 I-5.5 的行。
- 实测：抬 F 那条用例与同文件另 15 条（`--include-ignored`）全绿：「test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out」（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:81`）；去掉 I-9.23 或 I-9.18 的专门理由两行变异都红在「报缺省理由」那条断言（上表序 17、18）。

### 3.4 合法历史上新九条有没有落到缺省理由上（正推的一格）

探针 `zs_new_nine_never_fall_to_the_default_reason_on_the_legal_histories`：12 个合法历史镜像（单文件、三单元文件发完意图、第一批后、空 inode 发完意图、三单元文件删到底、第 3 号删完根环转一圈、回退后再删且根环转一圈、回退到第一条根、旋钮下 1/2/3 个 inode 删完各入口）× 九条（I-9.17–I-9.24、I-9.29，I-9.22 在两个入口都看），每条要么 Holds、要么 NotApplicable 且理由是上表某一句（I-9.24 是它自己那一句）。结果：「ZS_REASON_SUMMARY cases=12 flagged=1」（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:380`）。那 1 个标出的是我的用法：旋钮下才合法的镜像用旧入口判，I-9.21 红（按产品容量判），这一形 GPU 对拍文档里早写明「旋钮下合法、CPU 判器按旧入口判就红」，不是缺省理由、也不是合法历史上的红：「ZS_REASON two-inodes-one-tombstone-per-container-old-entry entry=old I-9.21 VIOLATED:根（实例 1、txg 7）：墓碑树（树 65545）里不是最右的容器 1 装了  FLAGGED」（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:345`）。其余 11 个镜像、两个入口全部没有缺省理由，也没有别的理由。
- 判定：一致。推翻：任一合法历史上这九条里有一条报「这个镜像上没有它判的对象」或别的不在表里的理由；`rerun.sh reasons` 可复现。线索归属：合法历史的广谱扫是攻方腿 K2 的格，我这 12 个是手挑的，不是抽样。

## 4 N4　E142 导出：拒绝判据改按算术

### 4.1 应该是什么

本批题面：拒绝判据按算术（checked 乘法，两块盘空闲图合起来上界 1 GiB，上界是推的）；退出码 1 = 命令行不对或每盘字节数解得开而空闲图合起来超上界或算不出，0 = 正常，101 = mkfs 收了后面没走通（文件头文档，「两块盘内存里的单元区空闲图合起来超过定死的上界或算不出」，`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:20`）。

### 4.2 实现与我的独立算术

- 常量：「const FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST: u64 = 4;」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:140`）与「const FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST: u64 = 1 << 30;」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:145`）。每槽 4 字节我对着 `DeviceFreeMap` 的字段数了：三个 `Vec<bool>`（`allocated`、`isolated`、`held_until_floor_takes_effect`）3 字节，四个 `Vec<u64>` 每 64 槽一格（`used_per_segment`、`isolated_per_segment`、`held_per_segment`、`unblocked_slot_pairs_per_segment`）= 0.5 字节，3.5 ≤ 4（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-core/src/allocator.rs:253`、`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-core/src/allocator.rs:383` 起）。
- 判据：槽数 = 每盘字节 ÷ 16384 − 50176（单元区起点，默认环 768 MiB）；盘末在起点之前不在这里判（「let Ok(unit_area_slots) = unit_area_slots_of_device_starting_at(device_bytes, unit_area_start)」，`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:160`，放过交 mkfs），槽数 × 4 × 2 用 checked 乘（「.checked_mul(FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST)」，`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:170`），超 1 << 30 拒：「if free_map_bytes_of_the_two_devices > FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST {」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:173`）；拒绝走 `std::process::exit(EXIT_CODE_OF_A_REFUSED_COMMAND_LINE)`，值是 1（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:70`）。
- 用整数算术（`bin_arith.py`，Python 大整数，不起 bin）复算：上界恰好对应每盘 2²⁷ 槽（2 TiB 的单元区）：「SLOTS_AT_BOUND 134217728 (= 2^27 True) DEVICE_BYTES_AT_BOUND 2199845339136」（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:382`）；4 GiB、8 GiB 放过，0 字节放过（交 mkfs 退 1），2⁶⁴ − 1 与 2⁴³ 拒，恰在上界放过、多一槽拒（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:383-389`）。2⁶⁴ − 1 字节的盘每盘约 2⁵⁰ 槽、乘出来 2⁵³ 字节，不溢出，所以 checked 乘法的溢出那一支今天够不到（脚本末行 `fits u64: True`，`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:393`），K 报告里也这么写。
- 两行变异的算术复算：「INLINE_TESTS mutant18(> u64::MAX)   largest_device_bytes_test=RED at_the_bound_test=RED」（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:391`）与「INLINE_TESTS mutant19(>=)           largest_device_bytes_test=pass at_the_bound_test=RED」（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:392`）：K 的两条内联单测对这两行变异各翻红（`crates/mutations.tsv:2894`、`crates/mutations.tsv:2895`）。
- 本批对 bin 的改动只在拒绝函数、文件头文档、main 里的调用名与两条内联单测（`git diff --no-index --shortstat` 70 增 31 删，见第 0 节），输出路径一行没碰，所以「P8G 输出与产物逐字节相同、不带参数只差第 1 行」这条主 agent 的观测与 K 的自报，读 diff 看没有能让输出变的改动；字节比对本身我复核不了（见「没做什么」）。

### 4.3 N4 判定

- 一致：退出码（拒绝退 1）、算术（checked 乘、上界 2²⁷ 槽 / 盘）、放过的几档、变异两行的翻红。
- 上界是推的：2²⁷ 槽 / 盘的 1 GiB 只保证「明显超大的盘」在建盘前被拒；上界以下、4 GiB 以上的每盘字节数（例如 1 TiB，两盘空闲图 512 MiB）在这台机器上会不会在后面的 mkfs、取号、暖机里因别的分配过不去，没人量过——K 的文档注释自己写了「这个数是推的，没量过哪一档开始分配不出」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:143`，同一段）。推翻「上界够用」的现象：某个 ≤ 2199845339136 的 `--device-bytes` 起 bin 时进程 abort（退 134）或被杀。

## 5 N5　GPU：走读内核读孩子类标签同 N1（逐支对 CPU）

GPU 不跑（显卡与 checker 档归主 agent）；这一格是读码，用第 1 节 CPU 实测矩阵当期望。违例位的下标常量在 `crash_judge_checker_common.wgsl`（本批 diff 没动这份）：I-9.2 = 39、I-9.17 = 74、I-9.23 = 80、I-9.29 = 82（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:87`、`:165`、`:177`、`:181`），与 G 报告写的四个数相同。

| 支 | CPU | GPU | 同不同 |
|---|---|---|---|
| 认类的函数 | `child_unit_class_other_than_the_entry_type_names`（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:4086`） | 「fn w_child_unit_class_other_than_the_entry_type_names(child: Ptr, class_the_entry_type_names: u32) -> u32 {」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2942`） | 同：两条位置条目依次，头按 16 KiB 读 |
| 头里就是类型段说的类 | 「if Some(class_in_the_header) == class_the_entry_type_names {」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:4098`）交回 None | 「if (class_in_the_first_header == class_the_entry_type_names) {」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2946`）交回 NONE | 同，且不看这份校验和对不对 |
| 另一类登记过的码 | 码 1、3 按 32 KiB、码 2 按 16 KiB 读回再比位置条目校验和（walk.rs:4101-4107） | 「if (w_unit_verifies_at_the_length_of_its_class(first_copy, class_in_the_first_header, child.sum0)) {」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2949`） | 同（GPU 比的是版本表里预先算好的 16 KiB / 32 KiB 两个 CRC，32 KiB 那个要版本够 32 KiB） |
| 类标签不在登记表 / 读不出 / 校验和对不上 | 下一条位置条目，都不行交回 None 走老路 | 同，交回 NONE | 同 |
| 容器树类型段 0 | 判红、`continue`，排在 `node.level <= 1` 的走读失败之前（walk.rs:2184-2198） | 「let class_of_the_child_of_an_internal_entry = w_child_unit_class_other_than_the_entry_type_names(child, 2u);」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3014`）、`continue` 在 3018，层级判在 3025 之后 | 同 |
| 容器树类型段 = 记录类型 | 同上，named 是码 3 | 「let class_of_the_child_of_a_container_entry = w_child_unit_class_other_than_the_entry_type_names(child, 3u);」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3060`） | 同 |
| 容器树类型段不认得 | 不进新判，按层级登记（walk.rs:2240） | 「span_of_the_unregistered_child = 1u;」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3053`）按层级 | 同 |
| inode 树 | 类型段 2 应码 3、0 应码 2、不认得时没有「应是」：「InodeInternalEntryType::Unregistered(_) => None,」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2488`）起到 2509 | 「class_the_inode_entry_type_names = 3u;」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2611`）、`= 2u` 在 2613、「if (class_of_the_inode_child != NONE) {」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2616`） | 同 |
| 红的是哪一位、登记几槽 | 判红恒为 false，槽数按孩子那一类 | 「w_judge(INVARIANT_I_9_2, class_of_the_inode_child == class_the_inode_entry_type_names);」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2618`）；容器树两处 `== 2u`（3017）、`== 3u`（3063）：函数在相等时已交回 NONE，所以走到这里恒为 false | 同 |
| inode 树类型段不认得、孩子读不出 | 读不出记走读失败 `continue`，I-9.2 不判（walk.rs:2529） | 「let s = w_read_referenced_unit(child, length);」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2622`）失败记 `w_failures`、`continue` | 同（第 1.3 节那 9 格 GPU 与 CPU 一样不判 I-9.2） |

- 不一致的支：GPU 的 inode 走读只做根（层级 1），类型段 0 的孩子真是码 2 节点时「undecidable = UNDECIDABLE_TOO_DEEP;」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2737`）整个状态判不了，这是老行为；判不了的状态对拍里不比，不是误判。本批没引入新的不一致支。
- G 对齐的那一版（c9884ecc…）与 K 交回的那一版（2a95615f…）之间的差别有没有落在这一判上：**我复核不了**。我在 `/tmp/claude-1000` 下找到 354 份 `singlefs-checker` 的 walk.rs，sha256 前缀 c9884ecc 与 2a95615f 一份都没有（`grep -cE '^(c9884ecc|2a95615f)' walk-copies.txt` 输出 0），K 的副本 `/tmp/claude-1000/m3-batch24/k/repo` 已不在（`ls: cannot access`）。替代证据：G 报告第一节写出它对齐的 CPU 读法，每一支（头按 16 KiB、类标签相同交回 NONE、另一类登记且按自己长度自证得过才算、类型段不认得交回孩子自证的类、inode 同形）我都在冻结副本的 CPU 终版里找到对应行，GPU 与终版 CPU 也逐支一致（上表）；所以「终版 CPU 与 G 描述的 CPU 有没有差别」在这一判上我看不出差别，但这是对描述的核对，不是对两版 walk.rs 的 diff。
- 判定：一致（读码，GPU 未跑）；版本差别那一问复核不了。推翻：对拍里 N4 那几条（第 24 批新加 5 条与改了断言的 N4-a）出现 `CRASH_JUDGE_GPU_MISMATCH`；或者拿到 c9884ecc 那一版 walk.rs 与终版 diff 出落在认类函数、`judge_a_container_tree_entry_whose_child_is_of_another_class` 或 inode 那一段上的差。

## 6 N6　GPU：I-9.29 最新根实例表读不出时不置位

- 条款：「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」（`.claude/kb/invariants.md:316`）。
- CPU 在 I-9.29 上判不了有两类。本批改的是第一类：最新根自己指着的实例表沿链读不出：「None => AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheNewestRootsInstanceTableDoesNotReadBack,」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:10567`），由 `instance_table_rows_of_root_without_judging`（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:5718`）定；这时跨版本那张表只收号、不比，报 `partially_judged`（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:9673` 一带），没有任何路径在这一支上判 I-9.29 红。第二类是版本级缺口（一版的树表读不出、墓碑树读不全等）：I-9.29 列在「const TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS: [&str; 4] = [」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:9321`）里走 `part_of_the_range_not_judged`，GPU 对这类缺口本来就一格不置位（扫描内核注释：「CPU 记成射程里判不了的那几格」，`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2618`），老行为、本批没动。所以两个「不置位」条件，除上面第 ② 处上限外，现在是同一个集合：第二类早就一致，第一类这一批补齐。
- GPU 不置位的条件：「d_object_births_are_compared = r_instance_rows(checker_root_of(c_newest_index).base);」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3095`），为假时「if (!d_object_births_are_compared) {」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2820`）直接 `return`，I-9.29 只在 `d_note_object_birth` 里判，所以一个号都不记、不置位。
- 两个读法是不是同一个：`r_instance_rows`（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1093`）逐项对 CPU 的链读法：身份 (0, 4, 片序号, 0)、记录宽 88、记录数 × 宽 ≤ 声明长度、声明长度 ≤ 32768 − 136、第一条之前的记录行种类 0、链指针三种结局、零条记录（「if (count == 0u) {」，`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1112`，对 CPU `records.split_last()?`）都对得上。差别只有两处，都是 `r_instance_rows` 的上限：①行数超 256，GPU 记「undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (32u << 8u);」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1121`）整个状态判不了，CPU 无上限——判不了的状态对拍不比，不会 GPU 红而 CPU 不红；②链走 256 片封顶，到顶直接返回 false、不记 undecidable——一条实例表链超过 256 片而总行数不到 256（每片都几乎没有行）时 CPU 照读照比、GPU 不比。②推的是合法写者造不出（写者每片装满才换页，我没造过这样的镜像），只在手造镜像上 GPU 少红；这两处老行为（`r_instance_rows` 本批没动，I-3.12 一直在用它），②在这一批才连到 I-9.29 的置位上。
- 新的 `r_instance_rows` 调用会不会改坏别人用的草稿：它写「const C_R_ROWS: u32 = 205168u;」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:439`）起的 256 × 3 字，终点正好是下一块「const C_VALID_BEFORE: u32 = 205936u;」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:443`）的起点，不与删除态的 D_* 区重叠（「const D_OBJECT_BIRTHS: u32 = 303088u;」，`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:820`）；用 `C_R_ROWS` 的另外两个调用（`crash_judge_checker_scan.wgsl:1329`、`:1910`）都是先调 `r_instance_rows` 再读，不依赖上一次留下的行。
- 对拍补的一形：流 `record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_unreadable`（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6187`）与用例 `the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_object_birth_is_not_compared_once_the_newest_instance_table_does_not_read_back`（同文件 6615）：CPU 一侧的预设（翻坏前整条落盘 I-9.29 成立、翻坏后报部分没判且说明以「最新根的实例表读不出」起头）我用同一段历史的 harness 版（第 23 批已有的坏镜像）复跑：「test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 120 filtered out」（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:85`）。GPU 与 CPU 在每个状态上逐位相同要显卡，没跑。
- `crash_judge_gpu.rs` 只改了文档（去掉 `///` 行后 diff 为 0 行）；判法摘要罩着内核源码：「message.extend_from_slice(GPU_JUDGE_CHECKER_SCAN_SOURCE.as_bytes());」（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_gpu.rs:189`），版本前缀不抬不会让旧判定被当成新判器的。
- 判定：一致（读码）；GPU 对拍复核不了。推翻：对拍那条新流上 GPU 与 CPU 在任一状态不同位；或某个镜像上 `instance_table_rows_of_root_without_judging` 与 `r_instance_rows` 一个读得出一个读不出（我对两处逐项核了，没找到）。

## 7 N7　变异表（K 追加 21 删 1，G 追加 6）

- 行数用命令数：`grep -c '^第24批\|^第 24 批' crates/mutations.tsv` 在冻结副本上输出 27，K 的 21 行在 2877–2897、G 的 6 行在 2898–2903（`crates/mutations.tsv:2877`、`crates/mutations.tsv:2903`）。
- 锚点：27 行逐行核了六段、原文在源文件里恰好命中一次、替换文与原文不同、点名的测试 `fn` 在点名的文件里：「A_OK 27/27」（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:33`）。被删的那行不在表里：「B_DELETED_ROW_PRESENT False」（`research/prompts/m3-batch24-code-r1-sonnet-model/expected-summary.txt:34`），表里没有任何行的原文含它的锚点 `.try_reserve_exact(free_map_bytes_per_device)`（同文件 35 行）。全表 2898 行里原文命中不是恰好一次的只有 1 行，是 litmus 那行的源文件不在冻结副本里（副本只含 `crates/`），不是锚点腐化（同文件 36–37 行）。
- 红不红：K 的 19 行 harness 档我全施加跑了（N2 表）；K 的 E142 两行用算术复算翻红（N4）；G 的 6 行（显卡 + checker 档）没跑，只核了锚点与测试名存在。G 的 6 行只有 4 个不同的改坏点：内部条目那处判（两行共用）、容器条目那处判（两行共用）、inode 那处判（一行）、`d_object_births_are_compared = true`（一行）；每个改坏点是否在对拍里红，要显卡。
- 判定：一致（锚点、删行、K 的 harness 档 19 行）；G 的 6 行的红复核不了。推翻：任一行的原文在目标文件里命中 0 或 ≥ 2 次；`rerun.sh anchors` 可复现。

## 8 K3　被判的 10 份，每份一格

sha256 前缀取自 `research/prompts/m3-batch24-code-r1-snapshot/crates-sha256.txt`，与冻结副本逐份相同（第 0 节）。

| # | 文件（`crates/` 下） | sha256 前缀 | 这一格判到了什么 | 判定 |
|---|---|---|---|---|
| 1 | `mutations.tsv` | 46a04b62 | 27 行锚点与测试名（N7）；19 行 harness 档逐条红（N2）；删行 | 一致；G 的 6 行红复核不了 |
| 2 | `singlefs-checker/src/walk.rs` | 0d74ca43 | 认类函数、容器树两支、inode 一支、八条专门理由（N1、N3）；CPU 实测 144 格矩阵 | 一致；线索：inode 树读不出孩子时 I-9.2 不判（9 格） |
| 3 | `singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs` | f2a0c022 | 拒绝判据、常量、退出码、两条内联单测（N4） | 算术一致；bin 起不得（checker 档）；上界是推的 |
| 4 | `singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` | 29ba1b9e | I-9.29 的 `d_object_births_are_compared` 一处；去掉整行注释后与改前 diff 共 6 行（两边合计）（N6） | 读码一致；GPU 未跑 |
| 5 | `singlefs-checker-tier/src/crash_judge_checker.wgsl` | 7ce14c1f | 认类三处（容器树两支、inode 一支）与三个新函数（N5）；去掉整行注释后与改前 diff 共 65 行（两边合计） | 读码一致；GPU 未跑 |
| 6 | `singlefs-checker-tier/src/crash_judge_gpu.rs` | 7e5aa35f | 只改文档注释，去掉 `///` 行后与改前 diff 为 0 行；判法摘要罩着内核源码（N6） | 一致 |
| 7 | `singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 7bf535f7 | 新 5 条 + 改断言 1 条的对拍；`INVARIANTS_A_WALK_FAILURE_REDDENS`（「const INVARIANTS_A_WALK_FAILURE_REDDENS」，`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6522`）四条在每个状态上 CPU 都不红；CPU 一侧预设我用 harness 版复核（N6）。逐状态 GPU 对 CPU 逐位相同要显卡 | CPU 一侧一致；GPU 一侧复核不了 |
| 8 | `singlefs-harness/tests/checker_known_bad_images.rs` | f5c6b8ac | 新 11 条，全绿、各只红目标；11 条与变异序 0–10 一一对应，各有 1 行变异红在目标断言（N2） | 一致 |
| 9 | `singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs` | b36f6b4e | 新 6 条（M2、M3、M4、M6、M7、队列同号）全绿、各 1 行变异红（N2） | 一致；队列同号归 I-1.1：条款没说 |
| 10 | `singlefs-harness/tests/reuse_after_raising_the_floor.rs` | 3a963356 | 六对豁免、I-5.5 一支原样、16 条全绿、两行变异红（N3） | 一致 |

- 第 8 行的对应：序 0 → N4-a、1 → N4-i、2 → N4-c、3 → N4-f、4 → N4-j、5、6 → inode 两形、7 → 只改类标签的叶容器、8 → M1 类型段 0 容器号、9 → M1 谱系类型段不认得、10 → M5；点名的 11 条新用例各有且只有 1 行变异，没有无变异罩着的新用例。

## 9 模型与复跑

- 位置：`research/prompts/m3-batch24-code-r1-sonnet-model/`（`rerun.sh`、`anchors.py`、`mutate_rows.py`、`bin_arith.py`、`probes/zs_probe_tail.rs`、`probes/zs_probe_tail2.rs`、`expected-summary.txt` 393 行）。
- 用法：`SINGLEFS_REPO=<仓> bash rerun.sh <空目录> [anchors|tests|mut|matrix|reasons|arith|summary …]`；编译 6G、跑 4G 都经 `run-with-memory-cap.sh`（显式上限，不用 auto），线程 10，编译目录 `<目录>/target`；`compare` 比之前把并行测试的 `test … ok` 行折成测试名并排序，自己不依赖 `SINGLEFS_REPO`（比的是脚本旁的 `expected-summary.txt`）。
- 空目录从头跑通：`rerun1` 产出 `expected-summary.txt`，`rerun2`、`rerun3` 各在空目录里从头跑（三个目录交回前都删了，见第 12 节），`rerun3` 末行 `SUMMARY_MATCHES lines=393`，`sha256sum` 与 `expected-summary.txt` 同为 `2d2fa3cf131b164e224537e1cf41fa5f29e0dacb2e0b14b50a477d2cb514fe79`。
- 前台一次放不下（单条命令 4 分钟上限）：分三次给步名：`anchors tests`、`mut`、`matrix reasons arith summary`（矩阵约 100 秒、变异约 2.5 分钟）。
- 探针不改任何 `crates/` 文件：`zs_probe.rs` 是 `checker_known_bad_images.rs` 去掉 `//!` 行的副本接上探针尾巴，放在我的仓副本的 harness 测试目录里。

## 10 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 N1 | 一致（附线索） | CPU 类标签认孩子、按孩子那一类登记、只红目标、不记走读失败；144 格矩阵 117 格与条款推的一致，27 格偏离全追到根（18 格 I-1.11 是另一条自己的判定；9 格是 inode 树读不出孩子时 I-9.2 不判，老行为，N1「同法」没罩到） |
| K1 N2 | 一致；队列同号归属规则没说 | 新 11 + 6 条全绿，K 的 19 行 harness 变异 19 红在名字说的那一处；I-1.1 行没写 `deleted_inodes` 同号两条，只有 I-9.16 一句旁证；K 报告「12 条」应为 11 条 |
| K1 N3 | 一致 | 八条十句在九处报、文字未改；豁免六对 `assert_eq!` 只认那一句、I-5.5 一支原样；12 个合法历史没有落到缺省理由 |
| K1 N4 | 算术一致；bin 与产物比对复核不了；上界是推的 | 退出码 1、2²⁷ 槽 / 盘上界、放过与拒绝各档、两行变异翻红都对；上界以下大盘会不会在别处失败没人量过 |
| K1 N5 | 读码一致；G 对齐版与 K 终版之差复核不了 | GPU 认类逐支对 CPU（含 inode 读不出孩子那 9 格同样不判）；两个 sha256 的 walk.rs 都找不到 |
| K1 N6 | 读码一致；对拍复核不了 | 两个「读不出」读法逐项对上，只差 `r_instance_rows` 的两处上限（256 行判不了；链 256 片到顶 GPU 不比，合法写者造不出，推的）；CPU 一侧预设的 harness 版跑绿 |
| K1 N7 | 一致；G 的 6 行红复核不了 | 27 行锚点与测试名全对、被删行不在、K 的 19 行 harness 档 19 红 |
| K3 | 10 份各一格 | 见第 8 节 |

## 11 给 K2 / 主 agent 的线索（不是我的格，不替它们判）

- inode 树读不出孩子时 I-9.2 不判（9 格，矩阵 `expected-summary.txt:192`、`:204` 一带，CPU 与 GPU 一样）：与容器树不对称；这是老行为，不是第 24 批引入。
- 容器树类型段不认得那一支仍按层级登记引用（`walk.rs:2240`，GPU `crash_judge_checker.wgsl:3053`），而 inode 树同一支按孩子自己的类登记；形状一致时没差别（矩阵里这几格没有 I-3.1），形状不一致（层级 1 的节点下类型段 7 指着码 2 节点）会多登一槽——我没造这一形。
- GPU 按「落点」读（要有一个起点正好在那里的已写落点），CPU 按镜像字节读（任何偏移）。指针指到一个单元中间、整单元校验和又手算对上的镜像上两边可能判得不同；合法历史上不会出现，只在手造镜像上。
- 认类函数遇到「第一份头里类标签就是类型段说的类、第二份才是另一类自证得过」时返回 None（看第一份）；CPU 与 GPU 判得一样，但与「任一份自证为另一类就红」的读法不同；两份内容不同的镜像是手造的。
- GPU 的 `r_instance_rows` 链走 256 片封顶到顶返回 false 而不记 undecidable（N6 第 ② 处）：I-9.29 的置位现在连着它。

## 12 没做什么

- 没跑 GPU，没跑 checker 档：G 的 6 行变异、GPU 对拍用例（新 5 条 + 改的 N4-a）、E142 导出 bin 的两条内联单测与 K 报告里第 18、19 行变异（`crates/mutations.tsv:2894`、`:2895`）、checker 档快档与 `crash_enumeration_sharded_across_processes.rs` 两个钉值。E142 bin 本身也没起：直接执行 `src/bin/` 下的装置属 checker 档，子 agent 不跑；所以退出码与「`--device-bytes 8589934592` 输出与产物逐字节相同、不带参数只差第 1 行」只做了读码与整数算术，没做字节比对。
- 没拿到 c9884ecc 与 2a95615f 两版 walk.rs（N5）。
- 没替主 agent 采纳或出判决；M8（去重键里的 txg）、第 25 批的 I-9.29 末片门、别的线的改动不在我的格里。
- 没攻：K2 是攻方腿的格。第 1.3 节的矩阵是核「实现是否按条款」的正推核对，不是对抗搜索，也没有用户决定的那几步放开扫。
- 仓内文件我没写；写了的只有报告、模型目录与我的草稿目录 `/tmp/claude-1000/m3-batch24-code-r1-sonnet/`。
- 交回前删了自己建的仓副本与编译目录（删前 `du -sh`）：`/tmp/claude-1000/m3-batch24-code-r1-sonnet/target` 1.8G、`/tmp/claude-1000/m3-batch24-code-r1-sonnet/work` 20M、`/tmp/claude-1000/m3-batch24-code-r1-sonnet/rerun1` 1.7G、`/tmp/claude-1000/m3-batch24-code-r1-sonnet/rerun2` 1.7G、`/tmp/claude-1000/m3-batch24-code-r1-sonnet/rerun3` 1.7G（每个里都有 `work` 与 `target`）。复跑要重编，用 `rerun.sh`。
- 没删（都是小文件，留作出处）：`/tmp/claude-1000/m3-batch24-code-r1-sonnet/rerun3-summary.txt`（`rerun3` 的 `summary.txt`，与 `expected-summary.txt` 同 sha256）、`/tmp/claude-1000/m3-batch24-code-r1-sonnet/walk-copies.txt`（N5 里找 walk.rs 副本的清单，354 行）、`/tmp/claude-1000/m3-batch24-code-r1-sonnet/zs1.log`、`zs2.log`（探针首次输出）、`mut-all.log`（首次变异输出）。

## 13 要主 agent 跑的命令（我这里不能跑的：GPU 与 checker 档）

工作目录仓根；都带 `SINGLEFS_HEAVY_TESTS=commit`（或 `=user-request`），经 `bash scripts/hosts/run-with-memory-cap.sh <上限> bash scripts/hosts/capped.sh <N> …`，上限与线程数我没有依据，沿用主 agent 提交时的取法。

1. GPU 对拍（要显卡）：`cargo test --offline -p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- <第 24 批新加的 5 条与改了断言的 N4-a>`，用例名就是 `crates/mutations.tsv:2898`–`:2903` 六行最后一列的六个名字（五条新加，加改了断言的 N4-a `the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_carries_an_identity`）。看哪几行：每条 `ok`，没有 `CRASH_JUDGE_GPU_MISMATCH`；没有显卡时用例自己报「没跑」，那不算通过。
2. G 的 6 行变异：`scripts/mutation/mutate.sh` 对 `crates/mutations.tsv` 的 2898–2903 行；看哪几行：6 行都红在各自最后一列的用例上。
3. E142 导出 bin 两条内联单测与 K 的两行变异：`cargo test --offline -p singlefs-checker-tier --bin e142_new_pool_file_creation_write_dump -- the_largest_device_bytes_parse_but_their_free_maps_exceed_the_bound_and_the_bin_ends_as_a_refused_command_line device_bytes_at_the_free_map_bound_pass_and_one_slot_more_is_refused`（看 2 条 ok）；变异 `crates/mutations.tsv:2894`、`:2895` 各红。
4. E142 bin 的退出码与产物：`--device-bytes 18446744073709551615` 应退 1 且 stderr 含「超过上界」，`--device-bytes 0` 应退 1（mkfs 的错），`--device-bytes 8589934592` 的输出与 `research/results/e142-r24-p8g-crates-dump-2026-10-05.out` 逐字节相同；再选一两个 1 TiB 上下的 `--device-bytes` 看会不会 abort（N4 第 4.3 节的推翻条件）。
