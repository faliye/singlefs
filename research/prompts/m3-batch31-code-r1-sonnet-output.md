# 删除批第 31 批合入的代码：代码轮第一轮云端正推（Sonnet）报告（2026-10-07）

<!-- doc-lint:not-numbers K1 K3 N1 N2 N3 N4 N5 N6 F-L0 F-W F-IT X3 X4 X5 O3 -->

轮名 m3-batch31-code-r1，立场：云端正推，格 K1（N1–N6）与 K3。冻结副本 `~/.local/share/singlefs/work/m3-batch31-code-r1/tree/`（下称「冻结副本」），
五份被判文件的 sha256 与 `research/prompts/m3-batch31-code-r1-snapshot/crates-sha256.txt` 逐份相同（本报告第 6 节贴命令输出）。
引 kb 写 kb 文件自己的行号；引代码写冻结副本全路径:行号。本报告分段落盘，后面各节陆续追加。

目录：第 0 节引文与复跑约定；第 1 节 N1；第 2 节 N2；第 3 节 N3；第 4 节 N4；第 5 节 N5；第 6 节 N6；第 7 节 K3；第 8 节判定一览；第 9 节没做什么与要主 agent 跑的命令。

## 0. 引文与复跑约定

- 带引号的代码引文写冻结副本全路径:行号（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/…`）；kb 引文写 kb 文件自己的行号（`.claude/kb/invariants.md` 等）。
- 无引号的行号指针只写文件名：`walk.rs` = 冻结副本 `crates/singlefs-checker/src/walk.rs`；`crash_judge_checker.wgsl`、`crash_judge_checker_scan.wgsl` = `crates/singlefs-checker-tier/src/` 下同名文件；`checker_known_bad_images.rs` = `crates/singlefs-harness/tests/` 下；`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` = `crates/singlefs-checker-tier/tests/` 下；`mutations.tsv` = `crates/mutations.tsv`。每个指针的行号都在写进来之前由脚本在那份文件里按原文找过。
- 复跑：`research/prompts/m3-batch31-code-r1-sonnet-model/rerun.sh`（文件头写了变量与阶段），只跑 `singlefs-harness` 的 `checker_known_bad_images` 一个测试二进制，不碰 checker-tier、不跑 GPU。编译 6G、跑 4G，经 `run-with-memory-cap.sh`，线程上限 10，同一时刻一条 cargo，编译目录 `<工作目录>/target`。
- 开跑前 `ps` 看到别的会话在跑一条 `cargo test --release -p singlefs-checker-tier … --no-run`（38G，线程 1），没有 `qemu-system`、`fio`、`vm-bench.sh`、`e152-file-system-benchmark`；照定义加 `nice -n 19`，没等锁（我这条 cargo 与它不同目录、各拿各的锁）。
## 1. N1：墓碑树、谱系树层级 ≤ 1 的类型段 0 孩子照判 I-2.1（F-L0）

### 1.1 从条款与今天的实现推出「应该是什么」

- I-2.1 那一行写比长度的读法：「不往下走孩子的几支（墓碑、谱系、inode 三棵树的类型段不认得、孩子自证成另一类），每条位置条目按那一份头里类标签说的那一类的长度比，类标签不在登记表里或头读不出的，16 KiB 与 32 KiB 两种长度都对不上才算不匹配」（`.claude/kb/invariants.md:130`）。
- I-9.23 与 I-9.17 那两行对「一份都没有自证成类型段那一类」的孩子写的是：「一份都没有的不往下走，每份副本照 I-2.1（校验和与内容匹配） 那一行的长度读法判」（`.claude/kb/invariants.md:311`）（谱系树同句在 `.claude/kb/invariants.md:305`）。
- 所以孩子两份都没自证成码 2、父节点层级 ≤ 1、条目类型段 0 的那一形，条款推出来的是：不往下走，每份副本按它头里那一类的长度比，对不上红 I-2.1（I-4.8、I-7.4 跟着），两份都对不上再记走读失败。

### 1.2 代码做了什么（CPU）

| 项 | 冻结副本里的原文 |
|---|---|
| 条件 | 「if node.level <= 1 {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2499`） |
| 新加的调用 | 「self.judge_every_copy_of_a_child_the_walk_does_not_go_into(」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2500`） |
| 之后照旧记一次走读失败 | 「self.walk_failures.push(format!(」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2504`）；说明里写着 「这两棵树里没有条款定的形状」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2505`） |
| 共用函数逐份判 | 「let matches = copy_matches_its_location_checksum_at_the_length_its_header_names(」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2463`）、「self.judgements.judge("I-2.1", matches, || {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2467`） |
| 两份都对不上才记走读失败 | 「if !some_copy_matches {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2475`） |
| 长度读法：头里是登记过的类就只按那一类 | 「.map(|header| MappingKeyUnitClass::of(header[UNIT_HEADER_CLASS_TAG_OFFSET]))」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:4482`）、「Some(unit_bytes) => vec![unit_bytes],」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:4485`） |
| 没登记或头读不出两种长度都试 | 「None => vec![node_bytes(), data_unit_bytes()],」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:4486`）；没登记的类交 None：「MappingKeyUnitClass::Unregistered(_) => None,」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:4393`） |
| 判完不往下走、不登记引用 | 这一支里没有 `note_references_of_a_pointer_followed_by_the_walk` 的调用（`walk.rs:2499-2510` 整段读过）；共用函数自己的说明写「引用由调用方按它定的形状先登记」：「只判这两样：不往下走，引用由调用方按它定的形状先登记。」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2455`） |

- 到这一支之前的认孩子：CPU 先问孩子有没有自证成别类（「if let Some(child_class) = child_unit_class_other_than_the_entry_type_names(」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2337`）），再问另一份是不是类型段那一类（「if some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2346`））；两问都过了才落到 `walk_the_internal_node_below_a_container_tree_entry`（`walk.rs:2385`），所以 F-L0 这一支只接「孩子一份都没自证成别类」与「F-W 判红之后照类型段 0 往下走」两种孩子。

### 1.3 GPU 走读内核同一支（读码，GPU 没跑）

| 项 | CPU | GPU |
|---|---|---|
| 条件 | `walk.rs:2499` | 「if (n.level <= 1u) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3180`） |
| 调共用函数 | `walk.rs:2500` | 「w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3183`） |
| 之后记一次走读失败 | `walk.rs:2504` | 「w_failures = w_failures + 1u;」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3184`） |
| 先认别类 | `walk.rs:2337` | 「var class_of_the_child_of_an_internal_entry = w_child_unit_class_other_than_the_entry_type_names(child, 2u);」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3163`） |
| F-W 判红后放行 | `walk.rs:2346` | 「if (class_of_the_child_of_an_internal_entry != NONE && w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child, 2u)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3165`） |
| 身份那一判在调用之前 | `walk.rs:2376` | 「w_judge(identity_invariant, is_zero64(rd64(n.base, o + 8u))」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3178`） |
| 逐份判 | `walk.rs:2463` | 「if (!first_matches) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3100`）、「w_judge(INVARIANT_I_2_1, second_matches);」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3109`） |
| 两份都对不上才记失败 | `walk.rs:2475` | 「if (!first_matches && !second_matches) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3110`） |
| 登记过的类（码 1–3）只按那一类的长度 | `walk.rs:4485` | 「if (class_in_the_header >= 1u && class_in_the_header <= 3u) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3089`） |
| 没登记两种长度都试 | `walk.rs:4486` | 「return slice_crc_matches(copy, NODE_BYTES, checksum) || slice_crc_matches(copy, DATA_UNIT_BYTES, checksum);」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3092`） |
| 头读不出 | `walk.rs:4482`（读不出就是 None，两种长度都试） | 「if (copy.version == NONE) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3085`）（头读不出直接判不匹配；32 KiB 的读是 16 KiB 的超集，读不出 16 KiB 时 32 KiB 也读不出，两边同果） |
| 引用 | 不登记 | 这一支里没有 `w_note_pointer_references`（`crash_judge_checker.wgsl:3180-3186`） |

判定（N1 CPU 与 GPU 同一支）：一致。GPU 函数头上的说明（`crash_judge_checker.wgsl:3095`）没把第 31 批这一支列进去，CPU 函数说明（`walk.rs:2450-2451`）列了；只是说明没跟上，判法同。

### 1.4 在 harness 上量到的（CPU 池级 checker，探针在 `probe_append.rs`，原样输出在 `expected-compare.txt`）

`DBG` 是复跑时临时打在共用函数调用前的一行打印（`extra_mutations.tsv` 第 4 行，只在副本里，不是被判的代码）：每份副本按「头里那一类的长度」比位置条目校验和，对得上没有。

| 探针 | 造的形 | CPU 红的集合 | 孩子那一处每份副本（`DBG`） |
|---|---|---|---|
| A-tomb-type0-fw-node-copy-at-location-0 | 墓碑树，类型段 0、身份清零；位置条目 0 那一份换成树根的码 2 字节、位置条目 1 那一份是原来的码 3 容器（F-W） | I-2.1, I-4.8, I-7.2, I-7.4, I-9.23 | 盘 0：对得上；盘 1：对得上 |
| A-tomb-type0-fw-node-copy-at-location-1 | 同上，码 2 那一份在位置条目 1 | I-2.1, I-4.8, I-7.2, I-7.4, I-9.23 | 盘 0：对得上；盘 1：对得上 |
| A-lineage-type0-fw-node-copy-at-location-0 | 谱系树，同 A 墓碑那一形，码 2 那一份在位置条目 0 | I-2.1, I-4.8, I-7.2, I-7.4, I-9.17 | 盘 0：对得上；盘 1：对得上 |
| A-lineage-type0-fw-node-copy-at-location-1 | 谱系树，码 2 那一份在位置条目 1 | I-2.1, I-4.8, I-7.2, I-7.4, I-9.17 | 盘 0：对得上；盘 1：对得上 |
| D-tomb-type0-both-copies-valid-node | 墓碑树，类型段 0、身份清零；两份都换成码 2 字节、两条位置条目校验和都改成它的 CRC | I-2.1, I-4.8, I-7.2, I-7.4 | 盘 0：对得上；盘 1：对得上 |
| D-lineage-type0-both-copies-valid-node | 谱系树，同上 | I-2.1, I-4.8, I-7.2, I-7.4 | 盘 0：对得上；盘 1：对得上 |
| E-tomb-type0-one-valid-node-one-unreadable | 墓碑树，D 之后位置条目 1 的盘改成不在池里的 7 | I-2.1, I-4.8, I-7.2, I-7.4 | 盘 0：对得上；盘 7：对不上 |
| E-lineage-type0-one-valid-node-one-unreadable | 谱系树，同上 | I-2.1, I-4.8, I-7.2, I-7.4 | 盘 0：对得上；盘 7：对不上 |
| G-tomb-root-level-one-type0-two-location-checksums-wrong | 墓碑树根层级 1，类型段 0、身份清零、两条位置条目校验和都翻一字节（就是 harness 的 F-L0 坏镜像） | I-2.1, I-4.8, I-7.2, I-7.4 | 盘 0：对不上；盘 1：对不上 |
| G-tomb-root-level-zero-type0-two-location-checksums-wrong | 同上，只把根的层级字节 1 改成 0 | I-2.1, I-4.8, I-7.2, I-7.4 | 盘 0：对不上；盘 1：对不上 |

读法：
- A（F-W、层级 1、类型段 0）：两份都「对得上」，所以孩子那一处没有产出 I-2.1。A 行红集里的 I-2.1 是别处来的：探针把孩子的槽改了写、中央映射条目还指着原来的校验和，第一处违例说明写的是「映射条目指的单元」（复跑时 `probe.txt` 里 `i2_1_first_item=映射条目指的单元`）。
- D（两份都是对得上的码 2）：两份「对得上」，没有多红；E（一份读不出）：读不出那份「对不上」，孩子那一处红 I-2.1，另一份不受影响；A 的位置条目 0 / 1 两种盘序红的集合逐字相同。
- G：层级 1 与层级 0 两种根，两份都「对不上」，孩子那一处红 I-2.1；层级 0 的根下面类型段 0 也走这一支，CPU 与 GPU 都写 `<= 1`（`walk.rs:2499`、`crash_judge_checker.wgsl:3180`）。

### 1.5 冲突：F-W 之后、层级 ≤ 1、类型段 0——另一份按哪一种长度比（两边整句并排）

| 条款 A：I-2.1 那一行（比长度按头里的类） | 条款 B：I-9.23 那一行（另一份按类型段那一类的长度） | 代码与测试怎么做 |
|---|---|---|
| 「不往下走孩子的几支（墓碑、谱系、inode 三棵树的类型段不认得、孩子自证成另一类），每条位置条目按那一份头里类标签说的那一类的长度比，类标签不在登记表里或头读不出的，16 KiB 与 32 KiB 两种长度都对不上才算不匹配」（`.claude/kb/invariants.md:130`） | 「就照类型段那一支往下走那一份，引用登记的槽数按那一类；另一份按同一长度读回比校验和，对不上照红 I-2.1（校验和与内容匹配），I-4.8（近 K 代根校验和自洽）、I-7.4（近 K 代块未被复用） 跟着」（`.claude/kb/invariants.md:311`） | 层级 ≤ 1 这一支：「self.judge_every_copy_of_a_child_the_walk_does_not_go_into(」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2500`），每份按头里的类（A 行：两份都对得上，孩子那一处没有 I-2.1）；GPU 对拍流的说明也这样写：「两份照判 I-2.1（都对得上）、记走读失败，红 I-7.2」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9901`） |

- 同一形在层级 ≥ 2 走的是另一条路：F-W 判红之后落到 `read_index_node`，那里 `read_referenced_unit` 按 16 KiB（类型段那一类的长度）逐份判 I-2.1（「judgements.judge("I-2.1", matches, || {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/image.rs:1530`）），另一份是 32 KiB 容器时按 16 KiB 比对不上，红。读代码得出，没在 harness 上量：harness 里没有层级 2 的 F-W 镜像（m3-batch30 判决 G3 记的缺口）。
- 第 30 批判决进下一批的第 6 条把 B 那一句定成了要写进条款的读法：「F-W 之后别类那一份按类型段长度读对不上时 I-2.1 一族照红」（`research/prompts/m3-batch30-code-r1-main-verification.md:46`）。
- 这一形在第 31 批之前就没有 I-2.1（层级 ≤ 1 只记走读失败），N1 没有改它的答案；改动后答案仍是 A 条款的读法，B 条款的读法在层级 ≤ 1 上没有落到代码。没有任何一条 harness 用例或变异钉着这一形（第 30 批判决 G1 缺口，进第 32 批）。
- 判定：**冲突**（两条 kb 句子对同一形给不同答案：A 读法不红、B 读法红），不替主 agent 选。推翻这条的现象：A 探针行的 `DBG` 出现「对不上」（那就是孩子那一处红了），或 I-9.23 / I-9.17 那一句改成只管层级 ≥ 2。

### 1.6 N1 其余几问

- **引用登记、走读失败次数**：代码里层级 ≤ 1 这一支不登记引用（见 1.2 表末行），走读失败：两份都对不上时共用函数记一次、层级那一句再记一次，共两次；GPU 同样 `w_failures` 加两次（`crash_judge_checker.wgsl:3110` 与 `crash_judge_checker.wgsl:3184`）。正文第一节 N1 行写的是「引用照旧不登记」，与代码一致；kb 没写这种孩子占的槽算不算引用（规则没说）。共用函数自己的说明要调用方先登记（`walk.rs:2455`），这一处没有，说明与调用点不一致——只是观察，探针与 harness 的 F-L0 坏镜像钉的红集不含 I-3.1：「&["I-2.1", "I-4.8", "I-7.2", "I-7.4"],」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:13096`）。
- **长度读法与 I-2.1 那一行**：CPU `walk.rs:4475-4491` 与 GPU `crash_judge_checker.wgsl:3084-3092` 逐项对上 1.2 / 1.3 两张表（登记的类只按那一类、没登记或头读不出两种长度）；与 I-2.1 那一行写的一致。I-2.1 那一行列的「不往下走的几支」是两支（类型段不认得、孩子自证成另一类），第 31 批加了第三支（层级 ≤ 1 的类型段 0）没进那一行：规则没说，写回时补。
- **盘 0 / 盘 1**：共用函数按 `child.locations` 逐条判、不看盘序；A 的两种盘序红集相同（1.4 表）。
- **谱系树**：与墓碑树同一份代码（`name` 不同）；谱系树的 F-L0 坏镜像是 `a_lineage_tree_entry_of_type_zero_…` 那一条，复跑 5 条用例都过（第 4 节）。
- **判定**：N1 的 CPU 实现与 I-9.23 / I-9.17 对「孩子一份都没自证成类型段那一类」的句子：**一致**；与 I-2.1 那一行的长度读法：**一致**；F-W 之后另一份的读法：**冲突**（1.5）；第三支没写进 I-2.1 那一行：**规则没说**。

## 2. N2：GPU 走读内核读 extent 上段根照 CPU 的豁免

### 2.1 条款推出来的

- 「按位置寻址的两棵树（分配记录树、extent 树），节点头里的层级与 key 区间等于它的位置规定的那一层、那一段」（`.claude/kb/invariants.md:26`）；「根之下没有空节点（这一半 checker 由实二五补）。」（`.claude/kb/invariants.md:26`）——「不空」管的是根之下，根自己不在内。
- 「判定 = 树表有条目 ∧ inode 树零条目根（key 区间 `[0, 0]`，照已定项 6）∧ extent 树零条目根（key 区间按位置规定）」（`.claude/kb/decisions/08-核心索引结构.md:469`）：删空之后 extent 树的根就是层级 0 的零条目根，是合法形状。
- 推出来：位置为「根」的那一次读，层级 0 且零条目的节点只判 key 宽（区间由调用方按位置判）；层级 0 非空的根、层级 ≥ 1 的根（空不空都）、所有位置不是根的节点，照旧判「分隔 key 严格递增、节点不空」。

### 2.2 CPU 与 GPU 一一对应

| 情形 | CPU（`walk.rs`） | GPU（`crash_judge_checker.wgsl`） |
|---|---|---|
| 选哪一种读法 | 位置是 `None`（根）取 extent 上段根读法，`Some` 取按位置读法：「KeyRangeReading::PrescribedByThePositionOfTheExtentUpperRootJudgedByTheCaller」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:3592`）、「Some(_) => KeyRangeReading::PrescribedByThePositionJudgedByTheCaller,」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:3594`） | 读之前 `pending_level == NONE` 取 3，否则 2：「if (pending_level == NONE) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2191`）、「key_range_reading_of_this_node = 3u;」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2192`）、「let read = w_read_index_node(pending_pointer, tree, 3u, key_range_reading_of_this_node, true, fsid_low);」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2194`） |
| 层级 0、零条目的根 | 「(KeyRangeReading::PrescribedByThePositionOfTheExtentUpperRootJudgedByTheCaller, 0)」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:1499`） 加 「if view.entries.is_empty() =>」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:1500`），只比 key 宽：「if schema.width() == view.key_width {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:1502`） | 「} else if (key_range_reading == 3u && n.level == 0u && n.entry_count == 0u) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1515`），只比 key 宽：「keys_hold = schema_key_width(schema) == n.key_width;」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1517`） |
| 层级 0 非空 / 层级 ≥ 1 的根 | 「| (KeyRangeReading::PrescribedByThePositionOfTheExtentUpperRootJudgedByTheCaller, _) => {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:1510`） 落到 「check_internal_node_separators(&view, schema)」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:1511`） | 落 else：「keys_hold = w_check_internal_node_separators(n, schema);」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1519`）；该函数里 「if (n.entry_count == 0u) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1438`） 判空红 |
| 孩子节点（位置是 `Some`） | 按位置读法，同上 `_` 一支 | `pending_level` 此时是孩子的层级，传 2，走 else |
| 下段的根与节点 | `walk_extent_lower_node` 一律 「KeyRangeReading::PrescribedByThePositionJudgedByTheCaller,」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:3704`） | 下段走另一个循环，读根也传 2：「let read = w_read_index_node(pending_pointer, tree, 3u, 2u, true, fsid_low);」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2066`） |
| 节点解出来 | 零条目时 `entries` 为空：「let entries = if entry_count == 0 {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/lib.rs:517`）；条目宽小于 key 宽拒：「if declared_length != entry_count * entry_width || entry_width < key_width {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/lib.rs:505`） | 「if (entry_width < key_width) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:1435`）（零条目、条目宽 ≥ key 宽照样解得出） |

`pending_level == NONE` 只在读根那一次成立：
- `w_walk_extent_tree` 里 `pending_level` 声明在 `crash_judge_checker.wgsl:2184`（「var pending_level = NONE;」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2184`）），整个函数里给它赋值的只有一处 「pending_level = level - 1u;」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2314`）（孩子入栈之前）；读根失败时 `depth == 0` 直接 `break`，不会带着 `NONE` 再读别的节点。
- 这个循环只有一个调用点：「w_walk_extent_tree(pointer, tree, fsid_low);」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2977`），CPU 的根读法也只有一个调用点：「self.walk_extent_upper_node(&entry[14..100], None, tree);」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2146`），孩子那一处传 `Some`：「self.walk_extent_upper_node(child_pointer_bytes, Some(child), tree);」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:3678`）。
- 注释写的读法号与 CPU 枚举同名：「3 按位置读 extent 树上段的根」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1476`）。

### 2.3 测试与变异

- 第 31 批没给 N2 加新用例；变异行 `mutations.tsv:3064` 点名的是早先就有的整流对拍（「fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch() {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:5535`）），两个窄根变体流（例 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6211`）没点名。变异把豁免整句关掉（`false && …`），原文在 `crash_judge_checker.wgsl` 里恰好命中一次（`anchors.txt`：`ANCHOR line=3064 … anchor_hits=1`），点名的测试是 `#[test]` 且被 `--exact` 筛得到。
- GPU 一侧没跑：「删到最后一批那条流上第 156–164 个状态 GPU 多红 I-1.1」是实现员写进变异说明的，我复核不了（要卡）。
- 判定：**一致**（读码；条款、CPU、GPU 三边对上）。推翻它的现象：`w_read_index_node` 以 3 被别处调用，或 `pending_level` 在读完根之后还保持 `NONE`（例如把赋值挪到分支里）；也可以是卡上那条整流对拍出现 `mismatches`。

## 3. N3：站点 37 / 38 兜底那一形（候选根自己的链 257 片、最新根的链途经它第 100 片）

### 3.1 流造的是什么、期望写全没有

- 造流的函数在构造时就在 CPU 一侧钉：整条落盘的镜像上最新根那条链第 2 片身份不对，CPU 池级 checker 红 I-1.1：「pool_checker_verdict_on_the_image(&image_with_every_write, "I-1.1"),」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:10260`）（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:10050-10266`）。
- 「257 片」：「InstanceTableChainPages::OneMoreThanTheGpuWalks.count(),」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:10171`），枚举的值是 「Self::OneMoreThanTheGpuWalks => GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED + 1,」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6812`）、测试里那个上限 「const GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: usize = 256;」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6759`），两个内核各自的上限同为 256：「const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 256u;」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1584`）（扫描内核 `crash_judge_checker_scan.wgsl:13`）。
- 「途经第 100 片」：「const PAGE_OF_THE_CANDIDATE_CHAIN_THE_NEWEST_CHAIN_RUNS_INTO: usize = 100;」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:10035`）；最新根那张表第 1 片的链指针记录指到 R 那条链的第 100 片（`chain_record_naming_the_next_page(` 带 `slot_of_the_page_run_into` 那一处在 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:10220-10224`），那一片的容器号是 100、在最新根的链上是第 2 片，身份不对（I-1.1 红、走读失败）。
- 期望：对拍流的期望分三层——判了的状态与 CPU 逐位相同；判不了的原因字只许在三个里（61、37、38，`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:10279-10290`）；至少一个状态的原因字在 37 / 38 里（「states_carrying_a_required_reason > 0,」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9972`））。红项集本身不写（这一条流只比 GPU 与 CPU 一致，不钉红哪几条），所以「期望写全」指原因字与一致性，没有红项清单要补。
- 测试自己把「GPU 会落在 37 / 38」写成推的：「（37 或 38，推的：读码，攻方读码模型 `l2_chain_model.py` 也是这样，没在卡上跑过）」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:10273`）。**我复核不了**：要卡，归提交时主 agent 跑。

### 3.2 读代码推「GPU 会不会走到 37 / 38」（推的，没跑）

1. 走读内核 R 的链到第 100 片停、不记 61：最新根的链先走，第 2 片（R 的第 100 片）先 `w_visit`（走过），再判身份不符、记失败、`break`；之后走 R 的链走到第 100 片 `w_visit` 返回已走过，合并、`break`，循环没走满 256，所以不落到记 61 的那一句。
   - 「if (!w_visit(device_read_from, slot_read_from)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1624`），紧接着 「w_merge_below(pointer.device0, pointer.slot0);」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1625`） 与 `break`（`crash_judge_checker.wgsl:1624-1627`）；身份在 `w_visit` 之后判：「if (!identity_holds) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1649`）；只有走满才记 61：「if (page == INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1728`）。
2. 「走过」的集合是同一个：最新根与候选根都在上下文 0 里（`w_start(0u, true)` 只在入口调一次：「w_start(0u, true);」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3872`），候选阶段 `crash_judge_checker.wgsl:3892` 不重起）；上下文 1 另起一份走过的集合，被抛弃根、F 之下的根、重放那一版用（`w_start(1u, false)` 在 `crash_judge_checker.wgsl:4025`、`crash_judge_checker.wgsl:4162`、`crash_judge_checker.wgsl:4190`、`crash_judge_checker.wgsl:4236`）——R 要是落在候选集外，会在上下文 1 里把 256 片走满记 61、交接不成立，扫描内核不跑，到不了 37 / 38。
3. R 在不在候选集：「return !w_root_abandoned(r.instance, r.txg) && !lt64(r.txg, c_effective_floor);」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:662`）；「被抛弃」是实例表某行的 txg 小于这条根的 txg：「if (lt64(row_txg, txg)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:626`）。R 与最新根同属实例 3，行里实例 3 的 txg 不小于 R 的，所以不被抛弃（推的）；F 生效值要大于 R 的 txg 才踢出，这一条流里没有抬 F 的步骤（推的，没逐步核）。
4. 扫描内核跑不跑：走读内核判完且没记判不了才交接：「if (reached == WALK_REACHED_THE_HAND_OFF && undecidable == 0u) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4653`）；扫描内核入口 「if (sw(H_BASE + H_VALID) == 0u) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3199`） 不满足就什么都不做。走读失败（最新根链第 2 片身份不符）不挡这一步，只是红 I-7.2 等。
5. 扫描内核读 R 的链、读满记 37 / 38：候选根的引用集合那处 「for (var step = 0u; step < INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED; step = step + 1u) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:493`） 走完循环落到 「undecidable = UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED;」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:532`）；一条根自己的全部行那处同形：「undecidable = UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED;」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1178`）（循环在 `crash_judge_checker_scan.wgsl:1131`）。257 片的链在第 256 步之后还有第 257 片，两处都走得到。
6. 扫描内核末尾把 `undecidable` 写进结论字：「if (undecidable != 0u) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3207`）。

### 3.3 第 3065 行把两个常量写成 `0u`，与「删掉那两句」差在哪

- 两个常量只在两处被用：`crash_judge_checker_scan.wgsl:532`、`crash_judge_checker_scan.wgsl:1178`（`grep -rn` 全 `crates/` 的 `.rs` 与 `.wgsl`，除定义行 `crash_judge_checker_scan.wgsl:16`、`crash_judge_checker_scan.wgsl:19` 与测试里手抄的 `3 | (37 << 8)` 形式的镜像常量外，只有这两句赋值）。所以写成 `0u` 等于把这两句变成 `undecidable = 0u;`。
- 与删句相同的情形：进到这两句时 `undecidable` 本来是 0（这条流上的预期）——`= 0u` 与不写一样，扫描结论字走 `reached = 5`。
- 不同的情形：进到这两句时 `undecidable` 已经非零（别处先记了判不了且没立刻退出）。`= 0u` 会把先记的原因抹掉，删句保留它。我没有逐个核 `crash_judge_checker_scan.wgsl` 里每个设 `undecidable` 的站点是不是设完就退，所以这条差别只写到「可能有」。
- 这行同时改两处，点名的测试只要求「至少一个状态落在 37 或 38」（「states_carrying_a_required_reason > 0,」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9972`），要求集见上）：两处各自是不是被这条流走到，这一行分不出。要分，得各加一行只改一处的变异（37 单改、38 单改）；今天没有。
- 变异行的形：6 段、原文在 `crash_judge_checker_scan.wgsl` 里恰好命中一次（`anchors.txt`：`ANCHOR line=3065 … anchor_hits=1`），点名的测试是 `#[test]`、被 `--exact` 筛得到（同行 `named_is_a_test=yes named_selected=yes`）；特性开关 `verdict-store,gpu` 在 `crates/singlefs-checker-tier/Cargo.toml` 里都有。该文件不在被判的 5 份里（第 31 批没改它，只被变异行改坏）。
- 基线风险：这行的「红」依赖基线本身绿。基线要绿，这条流在卡上必须真有状态落到 37 / 38；今天只有读码（3.2）。要是读码错了（例如 R 实际在候选集外，或扫描内核读到第 N 片先因别的原因退出），基线就红，变异行也就没有判别力——这是复核不了、要主 agent 在卡上看的。
- 判定：**一致**（读码，GPU 没跑，推的）；变异行 3065 **成形、筛得到**，两站点各自没被钉（观察）。推翻它的现象：卡上这条对拍在基线上红（`states_carrying_a_required_reason` 为 0 或原因字落在三个之外），或单改一处的变异存活。

## 4. N4：测试尖度（X3 / X4 / X5 与容器树四处调用的对拍流）

判决要补的是：「测试尖度：正推腿三形（登记过的类按两种长度比、没登记只按 32 KiB 比、读不出算对得上）各补一形」（`research/prompts/m3-batch29-code-r1-main-verification.md:40`）；「GPU 容器树三处调用补对拍与变异」（`research/prompts/m3-batch29-code-r1-main-verification.md:40`）（第 31 批的 F-L0 又加了第四处调用）。

### 4.1 CPU 五条用例（harness 档，我复跑了）

复跑（`rerun.sh` 的 `baseline` 阶段，原样见 `expected-compare.txt`）：这 5 条 `test … ok`，`test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 170 filtered out`。变异表第 3054–3057 行四行各自在工作副本里改坏、跑那一行的参数，**四行都红**，每行红的就是它点名的那一条（F-L0 一行同时红墓碑与谱系两条，因为同一处代码）：

| 用例（`checker_known_bad_images.rs`） | 造的形 | 断言 | 变异行 → 复跑 |
|---|---|---|---|
| F-L0 墓碑（13102）与谱系（13115） | 层级 1 根第一条条目类型段 → 0、身份三段清零、子指针两条位置条目的校验和各翻一字节；孩子不动 | 红集恰好 I-2.1、I-4.8、I-7.2、I-7.4（「&["I-2.1", "I-4.8", "I-7.2", "I-7.4"],」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:13096`）），不含 I-9.23 / I-9.17（身份那一条不红）；根层级 「assert_eq!(level, 1, "{scene}：树根直接管容器");」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:13093`） | 3054 去掉那次调用 → 红 2 条（墓碑、谱系）；M2（删走读失败那句）→ 红同样 2 条 |
| X3（13167） | inode 根条目类型段 2 → 7、第一条位置条目校验和改成叶容器前 16 KiB 的 CRC | 红集恰好 I-9.2、I-2.1、I-4.8、I-7.4（`checker_known_bad_images.rs:13189`），I-7.2 不红（第二份对得上不记走读失败） | 3055 登记过的类也按两种长度比 → 红 1 条（X3） |
| X4（13199） | 叶容器类标签改成没登记的 9 再重封到根，根条目类型段 → 7、第一条位置条目校验和 = 改过的叶前 16 KiB 的 CRC | 红集恰好 「["I-1.11", "I-9.2"]」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:13234`）（I-2.1 不红：第一份按 16 KiB 对得上、第二份按 32 KiB 对得上） | 3056 没登记的只按 32 KiB → 红 1 条（X4）；**M1（我加的）没登记的只按 16 KiB → 红 1 条（X4）**，两个方向都钉住 |
| X5（13248） | 根条目类型段 → 7、第二条位置条目的盘改成不在池里的 7 | 红集恰好 I-9.2、I-2.1、I-4.8、I-7.4（`checker_known_bad_images.rs:13264`），I-7.2 不红（第一份对得上） | 3057 读不出算对得上（`is_some_and` → `is_none_or`）→ 红 1 条（X5） |

- 红集是「整集相等」不是「至少含」：F-L0、X3、X5 走辅助函数 「violated_invariant_set(&verdicts),」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:11471`），另验 I-2.1 的说明点名「走读不往下读的孩子」（「InvariantVerdict::Violated(detail) if detail.contains("走读不往下读的孩子")」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:11481`）），列了 I-7.2 的再验它的说明点名两份都对不上（「if detail.contains("（走读不往下读的孩子）两份都与位置条目里的校验和对不上")」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:11491`））；X4 自己内联同样的整集比较（`assert_eq!` 比 `violated_invariant_set`）。这几处让「I-7.2 不红」「I-2.1 红在这个孩子上而不是别处」「X4 的 I-2.1 不红」都落在断言里。
- X3、X4、X5 都用 inode 树的条目（类型段 7）造形，走的是 inode 树那两处调用（`walk.rs:2752`、`walk.rs:2783`）进同一个共用函数；三形钉住的是共用函数里的长度读法与「读不出」那句（「None => vec![node_bytes(), data_unit_bytes()],」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:4486`）、「.is_some_and(|content| crc32_castagnoli_table(&content) == location.checksum)」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:4491`）），墓碑 / 谱系的调用点共用它，所以三形对容器树的调用点也钉到了同一处代码。
- 第 3056 行替换文把原文那一行换成「只比 32 KiB」，M1 换成「只比 16 KiB」：同一个用例（X4）两个方向都红，说明用例确实同时要求「16 KiB 与 32 KiB 两种长度都试」；变异表今天只有 32 KiB 一个方向的一行，没有 16 KiB 那一行（我加的 M1 在 `extra_mutations.tsv` 第 1 行）。
- 判定：**一致**。F-L0 两条、X3、X4、X5 的期望写全，且每一条都有一行变异把它打红；复跑原样输出在 `expected-compare.txt`。

### 4.2 GPU 容器树四处调用各一条对拍流（读码，GPU 没跑）

四处调用与四条流一一对应，四行变异各去掉对应那一处调用（`mutations.tsv` 第 3058–3061 行）：

| 调用点（`crash_judge_checker.wgsl`） | 对拍流 | 流里写的 CPU 要红的集合 | 变异行 |
|---|---|---|---|
| F-L0：第 3183 行 | `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 第 9689 行起 | 「&["I-2.1", "I-4.8", "I-7.2", "I-7.4"],」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9696`） | 3058 |
| 类型段 7：第 3214 行 | `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 第 9722 行起 | 「&["I-9.23", "I-2.1", "I-4.8", "I-7.2", "I-7.4"],」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9729`） | 3059 |
| 类型段 0 指容器：第 3171 行 | `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 第 9756 行起 | 「&["I-9.23", "I-2.1", "I-4.8", "I-7.4"],」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9763`） | 3060 |
| 层级 2 类型段 1 指节点：第 3227 行 | `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 第 9788 行起 | 「&["I-9.23", "I-2.1"],」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9795`） | 3061 |

- 四个调用点各在 4 条流里被走到：F-L0 那条造的是层级 1 根条目类型段 0、孩子两份校验和都坏；类型段 7 那条是 H-J3a 同形；类型段 0 指容器那条第一份位置条目校验和坏、第二份自证成码 3（另一类，第二份对得上、不记走读失败，红集不含 I-7.2）；层级 2 那条先长高到层级 2 再改根第一条条目（孩子是层级 1 的码 2 节点，第一份按 16 KiB 对不上、第二份自证成码 2 别类）。这些形与 1.1 / 1.2 的 CPU 读法一致，我按 CPU 代码把每条的红集推了一遍（读 `walk.rs:2337-2372` 与 `walk.rs:2429-2445`），与流里写的集合相符。
- 流里写的是下界：「comparison.cpu_checker_red_invariants.contains(invariant),」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:5899`）只要求 CPU 红过这些，不要求恰好；真正钉 GPU 的是「comparison.mismatches.is_empty(),」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:5906`）（每个状态逐位相同）。所以「期望写全没有」看两件事：下界集没写错（上面推过，相符）、每个状态 GPU 与 CPU 逐位相同（卡上才知道，复核不了）。
- 观察（不是缺口，判决没要）：GPU 一侧长度读法那个函数（`crash_judge_checker.wgsl:3084`）本身没有 X3 / X4 / X5 三形的对拍流，8 行 GPU 变异也都改在调用点上、没有改这个函数；它与 CPU 的逐项对应靠读码（见 1.3 表）。
- 判定：**一致**（读码；GPU 没跑）。推翻它的现象：卡上任一条对拍出现 `mismatches`，或某一行变异在卡上存活（去掉调用点之后对拍流仍绿）。

## 5. N5：第 30 批留下的两处对拍（容器树两处 F-W、inode 类型段 0 而另一份自证成码 2）

### 5.1 容器树两处 F-W（读码）

| 流 | 造的形 | 流里写的 CPU 要红的集合 | GPU 一侧对应的放行 |
|---|---|---|---|
| 类型段 = 记录类型（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9886`） | 孩子盘 1 那一份自证成码 2、盘 0 那一份是码 3 容器（记录宽 39） | 「&["I-9.23", "I-1.7"],」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9895`） | 「if (class_of_the_child_of_a_container_entry != NONE && w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child, 3u)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3221`） 里的 「class_of_the_child_of_a_container_entry = NONE;」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3223`）（变异 3062 去掉它） |
| 类型段 0（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9904`） | 同上再把根第一条条目类型段 1 → 0 | 「&["I-9.23", "I-7.2"],」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9913`） | 「class_of_the_child_of_an_internal_entry = NONE;」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3167`）（变异 3063 去掉它） |

- 两条流共用一个造法（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9819`）：墓碑容器两份记录宽改 39、重封到根，再把盘序大的那一份的前 16 KiB 换成树根的码 2 字节、父条目那条位置条目的校验和换成那 16 KiB 的 CRC。与 harness 的 `…beside_a_copy_verifying_as_a_node_…` 坏镜像（`checker_known_bad_images.rs`，第 30 批）同一形，录成流。
- 类型段 = 记录类型那条：我按 CPU 代码推：另一份自证成码 2（别类）、有一份是码 3 按 32 KiB 对得上，F-W 放行并判身份红，再按码 3 往下读，容器里记录宽 39 → I-1.7 红；另一份按 32 KiB 读对不上，也红 I-2.1 一族。流里写的 `I-9.23`、`I-1.7` 是这个集合的下界，相符。
- 类型段 0 那条：与 1.4 / 1.5 的 A 探针是同一形——流里的说明写「两份照判 I-2.1（都对得上）」（「两份照判 I-2.1（都对得上）、记走读失败，红 I-7.2」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9901`）），与 A 探针 `DBG` 两份都「对得上」一致，I-7.2 红也一致。**这条流的说明就是 1.5 冲突里 A 读法的落地**：主 agent 要是把读法改成 B（另一份按 16 KiB 比、红 I-2.1），这条流的说明与 CPU / GPU 两边的代码都要跟着动（红集下界不变）。
- 变异 3062 / 3063 的原文各在 `crash_judge_checker.wgsl` 里恰好命中一次，点名的流是 `#[test]`、被 `--exact` 筛得到（`anchors.txt`）；这两行在卡上红不红复核不了。
- 判定：**一致**（读码，GPU 没跑）。

### 5.2 inode 树类型段 0 而另一份自证成码 2：GPU 判不了、原因字 4、回落 CPU

- GPU 一侧：走到「类型段 0 的内部节点」就记判不了：「// 类型段 0 的内部节点（类型段不认得的上面已经判完）：这一版没有这种形状（inode_tree_root_level_is_one），GPU 判不了」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2836`）、「undecidable = UNDECIDABLE_TOO_DEEP;」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2837`）；原因字 4：「const UNDECIDABLE_TOO_DEEP: u32 = 4u;」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:81`），测试里的镜像常量 「const GPU_UNDECIDABLE_TOO_DEEP: u32 = 4;」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9918`）。kb 里写的就是这个行为：「GPU 判器走读内核 inode 树只接类型段 2 的层级 1 条目：类型段 0 而有一份按码 2 自证过的条目落判不了（`UNDECIDABLE_TOO_DEEP`），交回 CPU 判」（`.claude/kb/invariants.md:290`）。
- CPU 一侧在 harness 上量到的（探针 B，两种盘序红集逐字相同）：`I-1.1, I-1.2, I-2.1, I-3.1, I-3.11, I-4.8, I-5.1, I-7.2, I-7.4, I-9.2`（`expected-compare.txt`）。第 30 批判决 O1 写的是这一形「多红 I-1.1、I-1.2、I-3.1、I-3.11、I-5.1、I-7.2」：「inode 那一形多红 I-1.1、I-1.2、I-3.1、I-3.11、I-5.1、I-7.2」（`research/prompts/m3-batch30-code-r1-main-verification.md:34`），探针与它逐项对上。
- 对拍函数的期望：判了的状态与 CPU 逐位相同，判不了的原因字只许是 4，至少一个状态是 4（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:10019`；公共函数 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9923`）。
- 观察：函数最后那一条「判不了就回落 CPU 之后红项与 CPU 相同」在判不了的状态上**按构造恒真**：「let concluded = gpu_verdict_items(gpu_words)」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9959`） 接 「.map_or_else(|| cpu_red_items.clone(), |items| format!("{items:?}"));」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9960`），而 `gpu_verdict_items` 在判不了的状态上交回 `None`：「if bits & (1 << verdict_bits::UNDECIDABLE) != 0 || words[POOL_CHECKER_UNDECIDABLE_WORD] != 0 {」（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:1529`）。真正的内容只有前两条（判了的逐位相同、原因字只许 4 且至少一个状态是）。这一写法在函数里早就有（`…:7471` 与 `…:8874` 两处同形），不是第 31 批新引入；写在这里让读报告的人别把它当成第三条独立检查。
- 这条流没有变异行：第 31 批追加的 12 行里没有一行改 `undecidable = UNDECIDABLE_TOO_DEEP;` 那一句。去掉它的话，基线断言「至少一个状态原因字是 4」会红，所以流本身有判别力，只是没有登记成变异行（观察）。
- 判定：**一致**（读码；CPU 红集在 harness 上量过，GPU 没跑）。推翻它的现象：卡上这条流某个状态判了而与 CPU 位不同，或原因字不是 4。

## 6. N6：变异表追加的 12 行（`mutations.tsv` 第 3054–3065 行）

静态：`crates-mutation-rows.py static-check <冻结副本>`（原样输出，退出码 0）：

```
COPIED	crates Cargo.toml Cargo.lock litmus research/scripts/run-with-memory-cap.sh research/scripts/crates-mutation-rows.py
CHECKED	3060	302	9
static-check rc=0
```

另查：这 12 行与表里别的行没有「同文件、同原文、同替换文」的重复，变异名也不与任何行重名（小脚本，输出「同文件同原文同替换文的组里碰到新行的：[]」「同变异名的组里碰到新行的：[]」）。

| 行 | 改什么 | 原文命中次数 | 点名的是 `#[test]` | 参数筛得到 | 复跑 |
|---|---|---|---|---|---|
| 3054 | F-L0 CPU：去掉层级 ≤ 1 那一处的调用 | 1 | yes | yes（筛出 5 条：2,1,1,1） | **RED**（cargo 退 101，跑 5 条、红 2 条） |
| 3055 | X3：登记过的类也按 16 KiB、32 KiB 两种长度比 | 1 | yes | yes（筛出 5 条：2,1,1,1） | **RED**（cargo 退 101，跑 5 条、红 1 条） |
| 3056 | X4：没登记的类只按 32 KiB 比 | 1 | yes | yes（筛出 5 条：2,1,1,1） | **RED**（cargo 退 101，跑 5 条、红 1 条） |
| 3057 | X5：读不出的副本算对得上 | 1 | yes | yes（筛出 5 条：2,1,1,1） | **RED**（cargo 退 101，跑 5 条、红 1 条） |
| 3058 | GPU F-L0：去掉层级 ≤ 1 那一处的调用 | 1 | yes | yes（筛出 1 条：exact） | GPU 没跑（归提交时） |
| 3059 | GPU：类型段不认得那一处去掉调用 | 1 | yes | yes（筛出 1 条：exact） | GPU 没跑（归提交时） |
| 3060 | GPU：类型段 0、孩子是另一类那一处去掉调用 | 1 | yes | yes（筛出 1 条：exact） | GPU 没跑（归提交时） |
| 3061 | GPU：类型段 = 记录类型、孩子是另一类那一处去掉调用 | 1 | yes | yes（筛出 1 条：exact） | GPU 没跑（归提交时） |
| 3062 | GPU F-W：类型段 = 记录类型那一处不放行 | 1 | yes | yes（筛出 1 条：exact） | GPU 没跑（归提交时） |
| 3063 | GPU F-W：类型段 0 那一处不放行 | 1 | yes | yes（筛出 1 条：exact） | GPU 没跑（归提交时） |
| 3064 | GPU：extent 上段根的豁免整句关掉 | 1 | yes | yes（筛出 1 条：exact） | GPU 没跑（归提交时） |
| 3065 | GPU 扫描内核：站点 37 / 38 的两个原因字写成 0 | 1 | yes | yes（筛出 1 条：exact） | GPU 没跑（归提交时） |

- 第 3054–3057 行的参数是同一串、四个子串（`mutations.tsv:3054`）：`of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong`（命中 2 条：墓碑、谱系）、`first_location_checksum_matches_only_its_first_sixteen_kibibytes`（1）、`of_an_unregistered_class_whose_first_location_checksum_matches_its_first_sixteen_kibibytes`（1）、`over_a_child_whose_second_copy_does_not_read_back`（1），合起来恰好是 N4 的 5 条；每行复跑「跑 5 条」与此一致。
- 四行 CPU 变异我在自己的工作副本里逐行改坏、跑、还原（还原后逐字节核与备份相同），**全红**，每行红的就是它点名的那条；3054 同时红了谱系那条（共用代码，没点名也红）。第 3056 行只钉「没登记只按 32 KiB」一个方向，我加的 M1（只按 16 KiB）也红在同一条用例上。
- 去掉 `if !some_copy_matches` 那句走读失败（M2）红 2 条（墓碑、谱系），说明 F-L0 用例的 I-7.2 说明断言钉着「走读失败是共用函数记的」。
- **层级 0 没人钉**：M3（把 F-L0 那一处的调用收成只在层级恰为 1 时才调）整份 5 条全绿（存活）；探针 G 在 M3 下层级 0 的根红集掉了 I-2.1（`extra-probe.txt`），层级 1 不变。这一形 CPU 与 GPU 都写 `<= 1`，没有测试也没有变异行钉层级 0（K2 攻方的格，这里只记一笔）。
- 3065 与 3064：见 3.3 与 2.3。第 3065 行改的 `crash_judge_checker_scan.wgsl` 不在被判的 5 份里（第 31 批没动它）。
- GPU 8 行（3058–3065）我只能核静态：锚点命中一次、点名的是 `#[test]`、`--exact` 筛得到、特性 `verdict-store,gpu` 存在。红不红要卡，**复核不了**。
- 判定：**一致**（12 行都成形、筛得到；CPU 4 行复跑全红；GPU 8 行静态对、动态没验）。

## 7. K3：被判的 5 份文件各一格

`sha256sum -c research/prompts/m3-batch31-code-r1-snapshot/crates-sha256.txt`（在冻结副本根下跑，原样）：

```
crates/singlefs-checker/src/walk.rs: OK
crates/singlefs-checker-tier/src/crash_judge_checker.wgsl: OK
crates/singlefs-harness/tests/checker_known_bad_images.rs: OK
crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs: OK
crates/mutations.tsv: OK
```

工作区这 5 份与冻结副本 `cmp` 逐份相同；`kb-sha256.txt` 里的 `invariants.md`、`08-核心索引结构.md` 两份对得上（本报告的 kb 行号按它们）。

| 文件 | 判到的格 | 判定 |
|---|---|---|
| `crates/singlefs-checker/src/walk.rs` | N1（1.2–1.6）、N4 复跑（4.1）、N6 CPU 四行（第 6 节） | 一致；F-W 之后另一份的读法冲突（1.5） |
| `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` | N1 GPU（1.3）、N2（第 2 节）、N4 四处调用（4.2）、N5 两处 F-W 与 inode 判不了（第 5 节） | 一致（读码） |
| `crates/singlefs-harness/tests/checker_known_bad_images.rs` | N4 的 F-L0 两条与 X3 / X4 / X5（4.1） | 一致（复跑 5 条过） |
| `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | N3（第 3 节）、N4 GPU 四条流（4.2）、N5 三条流（第 5 节） | 一致（读码，卡上没跑；N3 走到 37 / 38 是推的） |
| `crates/mutations.tsv` | N6 十二行（第 6 节） | 一致 |

## 8. 判定一览

| 格 | 判定 | 一句话（括号里是什么现象会推翻它） |
|---|---|---|
| K1 N1 CPU：孩子一份都没自证成码 2、层级 ≤ 1、类型段 0 | 一致 | 调用在位，与 I-9.23 / I-9.17 那句、I-2.1 长度读法逐项对上；复跑 F-L0 两条过、变异 3054 红（推翻：harness 上这一形 I-2.1 不红） |
| K1 N1 CPU 与 GPU 同一支 | 一致（读码） | 条件、调用、失败次数、先认别类、F-W 放行、长度读法逐项对上，GPU 没跑（推翻：卡上对拍出现 mismatches） |
| K1 N1 长度读法 vs I-2.1 那一行 | 一致 | 登记的类只按那一类、没登记或头读不出两种长度（推翻：头读不出时 CPU 与 GPU 判得不同） |
| K1 N1 第三支（层级 ≤ 1 的类型段 0）没列进 I-2.1 那一行 | 规则没说 | I-2.1 只列两支，第 31 批加了第三支；写回时补 |
| K1 N1 F-W 之后、层级 ≤ 1、类型段 0：另一份按哪种长度比 | **冲突** | I-2.1 那一行（按头里的类）→ 不红，I-9.23 / I-9.17 那一句（另一份按类型段长度）→ 红 I-2.1；代码取前者，探针 `DBG` 两份都「对得上」；层级 ≥ 2 取后者（读码）（推翻：这一形上孩子那一处出现 I-2.1） |
| K1 N1 引用登记与走读失败次数 | 一致（与正文） | 不登记引用、失败记两次，CPU 与 GPU 同；共用函数说明要调用方先登记、这一处没有（观察，kb 没写） |
| K1 N1 层级 0 的根下类型段 0 | 一致（读码 + 探针） | `<= 1` 两边同写；没有测试也没有变异钉层级 0（M3 存活）（观察） |
| K1 N2 GPU 读根那一种读法 | 一致（读码） | `pending_level == NONE` 只在读根那一次成立，与 CPU `position` 为 `None` 一一对应；层级 0 非空、层级 ≥ 1 照旧判分隔 key；没有新测试，变异 3064 点名早先的整流（推翻：卡上那条整流 mismatches） |
| K1 N3 流的造法与期望 | 一致（读码） | CPU 一侧在造流时钉 I-1.1；GPU 落 37 / 38 是推的、卡上没跑，我复核不了（推翻：卡上基线红） |
| K1 N3 变异 3065 | 成形、筛得到 | 常量写 0 与删句只在「先有别的原因」时有差；同改两处，两站点各自没被钉（观察） |
| K1 N4 CPU：F-L0 两条、X3、X4、X5 | 一致（复跑） | 5 条过；变异 3054–3057 四行全红、M1 / M2 红；红集整集相等（推翻：任一行在我的复跑里存活） |
| K1 N4 GPU 容器树四处调用 | 一致（读码） | 四条流各对一处调用，CPU 红集下界与我推的相符；GPU 没跑 |
| K1 N5 容器树两处 F-W | 一致（读码） | 类型段 0 那条的说明正是上面冲突里「不红」读法的落地 |
| K1 N5 inode 类型段 0 而另一份自证成码 2 | 一致（读码，CPU 红集量过） | GPU 记原因字 4；「回落 CPU 同判」那条在判不了的状态上按构造恒真（观察） |
| K1 N6 变异表 12 行 | 一致 | 成形、锚点命中一次、点名的是 `#[test]` 且筛得到；CPU 4 行复跑全红；GPU 8 行只核到静态 |
| K3 5 份文件 | 一致 | sha256 与快照逐份相同，每份一格（第 7 节） |

## 9. 没做什么

- **K2 不是我的格**：没替攻方腿造反例。探针里的 G（层级 0）、D、E 只是读 N1 时顺手量的，不算对 K2 的作答；O3（F-W 之后「走过」的键取 `child.locations[0]`）、第二次抽样（m3-batch30 的 F-W CPU / GPU 读码，m3-batch29 的 L1 一份读不出、L2 链长与候选集里外）都没做。
- **GPU 一律没跑**：第 2、3、4、5 节 GPU 一侧都是读码；GPU 8 行变异只核了静态。N3 的 37 / 38 落点是推的。
- **没编也没跑 `singlefs-checker-tier`**：只编了 `singlefs-harness` 的 `checker_known_bad_images` 一个测试二进制；没跑别的 harness 测试二进制（主 agent 说全套 170 个已绿）；没跑 fmt / clippy。
- **没造层级 2 的 F-W 镜像**来量 1.5 里「层级 ≥ 2 另一份红 I-2.1」，那一句只是读 `read_referenced_unit` 得出（harness 里没有现成的，第 30 批判决 G3 记了缺口）。
- **没逐个核**扫描内核里每个设 `undecidable` 的站点是不是设完就退（3.3 的「可能有」差别因此没坐实）。
- **中央映射噪声**：A / D 探针改了孩子的槽，I-2.1 的第一处违例来自映射条目；孩子那一处有没有产出 I-2.1，我用函数调用前的 `DBG` 逐份看，不用红集推（见 1.4）。
- 不采纳、不判决：1.5 的冲突、3.3 的「各自没被钉」、4.2 与 5.2 的观察，都交主 agent。

### 要主 agent 跑的命令（GPU，提交时）

工作目录 `/home/fy5090/code/singlefs`；每条形如（内存上限与线程数照主 agent 的，下面的 `<上限>` `<N>` 待填；`SINGLEFS_HEAVY_TESTS=commit` 前缀由主 agent 加）：

```
bash research/scripts/run-with-memory-cap.sh <上限> bash research/scripts/capped.sh <N> cargo test -p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact <名>
```

`<名>` 取下面九条之一，跑完看 `test … ok` 与 `CRASH_JUDGE_GPU_LEFT_TO_THE_CPU` 行（后两类流）：

1. `the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_child_whose_two_location_checksums_are_wrong`（变异 3058）
2. `the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong`（3059）
3. `the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_whose_first_location_checksum_is_wrong`（3060）
4. `the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_whose_first_location_checksum_is_wrong`（3061）
5. `the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_container_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node`（3062）
6. `the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_with_a_copy_verifying_as_a_node`（3063）
7. `the_gpu_judge_leaves_to_the_cpu_an_inode_root_entry_of_type_zero_over_a_leaf_with_a_copy_verifying_as_a_node`（无变异行；看原因字 4）
8. `the_gpu_judge_leaves_to_the_cpu_a_candidate_instance_table_chain_longer_than_the_pages_it_scans_once_the_newest_chain_runs_into_it`（3065；**先看基线绿不绿**：红则 3.2 的读码有错，3065 没有判别力）
9. `the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch`（3064；N2 的整流）

### 草稿与编译目录

- 复跑工作目录 `~/.local/share/singlefs/work/m3-batch31-code-r1-sonnet/run6`、`run7`（各自从空目录起、同一份 `rerun.sh`，阶段拆成 3 次调用跑：前台一次不超过 240 秒；两次 `compare.sh` 汇总的 sha256 同为 `bcf344ad029b8f2bcb09ff94295073b358b85d257e7de883282f0d6f01853fd4`，92 行，即 `expected-compare.txt`）。交回前删掉它们的 `tree/` 与 `target/`（编译目录与工作树），只留 `out/`。
- 脚本草稿在 `~/.local/share/singlefs/work/m3-batch31-code-r1-sonnet/build/`（写本报告的小脚本，每句引文写进来之前在被引文件里核过），不入库。

### 模型目录

`research/prompts/m3-batch31-code-r1-sonnet-model/`：`rerun.sh`（阶段与变量写在文件头）、`compare.sh`（排序、去掉耗时字段，不依赖 `SINGLEFS_REPO`）、`apply_mutation.py`（变异表一行的应用 / 还原 / 锚点核对）、`probe_append.rs`（追加在 `checker_known_bad_images.rs` 副本末尾的 CPU 探针，不断言）、`extra_mutations.tsv`（我加的 M1–M3 与打印用的 D1）、`expected-compare.txt`（两次从空目录跑出的汇总，92 行，sha256 `bcf344ad029b8f2bcb09ff94295073b358b85d257e7de883282f0d6f01853fd4`）。`rerun.sh` 在最后一次复跑之后只加了一行文件头注释（阶段分次调用的说明），判法没动。
