# m3-batch26-code-r1 正推腿（Sonnet）报告：K1、K3

立场：正推。格：K1（H1–H3 与测试）、K3（5 份每份一格）。方案按 `crates/` 今天的实现来谈。
引码写冻结副本里的 `crates/…` 路径与行号（`--root /tmp/claude-1000/m3-batch26-code-r1/tree`），引 kb 写 `.claude/kb/invariants.md` 自己的行号。

## 0. 核对输入与这一轮的事

- 被判 5 份与快照 sha256 逐份对上（冻结副本，复跑数在第 6 节）：`cd /tmp/claude-1000/m3-batch26-code-r1/tree && sha256sum -c research/prompts/m3-batch26-code-r1-snapshot/crates-sha256.txt` 五行 OK（我自己的拷贝施加变异、还原之后再核一次，也是 5 个 OK，第 6 节贴）。
- 仓工作区这 5 份与冻结副本 `cmp` 相同（cmp-worktree=same，五份都是），所以冻结副本即今天的实现。
- 线程上限：读码阶段收到「线程上限改成 10」，此后所有 cargo 都经 `capped.sh 10`（编译 `run-with-memory-cap.sh 6G`、跑 `4G`）。读码之前没起过 cargo。
- 内存包装对「上限超过峰值表实测峰值 2 倍」会拒（退 251），本轮派发要求显式写 4G，所以带 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON=派发提示要求跑上限显式写 4G` 跑；它不改变测试结果。
- 起跑前 `ps` 没有 `qemu-system`、`vm-bench`、`e152-file-system-benchmark`、`fio`。

## 1. 条款与判决的出处（先写要比对的「应该是什么」）

- kb 条款：`.claude/kb/invariants.md:289`：「类型段 0 ⇒ 子单元偏移 6 = 2 且身份引用的出生树、容器号、出生代三段为 0」；同一行：「类型段 2 ⇒ 子单元偏移 6 = 3」；同一行：「其余判该条目损坏」。
  I-9.17 `.claude/kb/invariants.md:304`：「与 I-9.2（条目身份与子头相符） 同形」。I-9.23 `.claude/kb/invariants.md:310`：「与 I-9.2（条目身份与子头相符） 同形」。
  kb 三条都没有写「孩子读不出时怎么判」，也没有写「头里的类等于类型段那一类时要不要自证」：这两处「规则没说」，由判决定。
- 判决：`research/prompts/m3-batch24-code-r1-main-verification.md:25`（K2 N1 两故障 H1 那一行）：「条目坏（N4-i、N4-f、inode 类型段 2 → 0）且孩子在盘 0 那一份陈旧、陈旧那份头里的类恰等于类型段说的那一类时」。
  第三节第 1 条 `research/prompts/m3-batch24-code-r1-main-verification.md:35`：「1. H1：读孩子类标签时，头里的类等于类型段那一类也按那一类长度验整单元校验和，验不过看下一份」。
  第三节第 2 条 `research/prompts/m3-batch24-code-r1-main-verification.md:36`：「2. inode 树读不出孩子时 I-9.2 的判法与容器树对齐」。
  K1 N1 那一行 `research/prompts/m3-batch24-code-r1-main-verification.md:23`：「inode 树读不出孩子时 I-9.2 不判进下一批」。
- 所以「应该是什么」两条：
  (H1) 读孩子类标签时，每份位置条目先按头里那一类的长度验整单元校验和，验不过看下一份；两份都验不过交回「不认得别的类」（照类型段那一支走）。
  (H3) inode 树孩子两份都读不出时，I-9.2 里不看孩子就能判的小句照判，判法与容器树逐支相同。

## 2. K1 · H1：CPU 与 GPU 自证的长度、次序、两份都不过的交回

### 2.1 CPU（`crates/singlefs-checker/src/walk.rs`）

- 逐条位置条目，次序是 `child.locations` 的次序：`crates/singlefs-checker/src/walk.rs:4113`：「for location in &child.locations {」。
- 头按 16 KiB 读，取类标签，类标签不在登记表（1、2、3）里就看下一份：`crates/singlefs-checker/src/walk.rs:4108`起；登记表 `MappingKeyUnitClass::of` 只认 1 / 2 / 3（`crates/singlefs-checker/src/walk.rs:808` 起的那段，我读过）。
- 按头里那一类的长度读回整单元、比位置条目里的校验和：`crates/singlefs-checker/src/walk.rs:4123`：「let verifies_at_the_length_of_its_class = reader」；验不过 `continue`：`crates/singlefs-checker/src/walk.rs:4126`：「if !verifies_at_the_length_of_its_class {」。
- 自证过之后才比类：`crates/singlefs-checker/src/walk.rs:4129`：「if Some(class_in_the_header) == class_the_entry_type_names {」，等就交回 None，不等交回那一类：`crates/singlefs-checker/src/walk.rs:4132`：「return Some(class_in_the_header);」。两份都过不了落到函数末尾交回 None。
- 对「应该是什么」(H1)：一致。比改前多的恰是「同类也先自证」这一步，次序、长度、两份都不过的交回都和判决字面相同。

### 2.2 GPU（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`）

- 第一份 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2964`：「if (w_unit_verifies_at_the_length_of_its_class(first_copy, class_in_the_first_header, child.sum0)) {」，里面先比类再交回；第二份 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2974`：「if (w_unit_verifies_at_the_length_of_its_class(second_copy, class_in_the_second_header, child.sum1)) {」；两份都过不了交回 NONE（函数末尾 `return NONE;`，我读的 2960–2985 行）。
- 自证函数 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2944`：「fn w_unit_verifies_at_the_length_of_its_class(s: Slice, unit_class: u32, checksum: u32) -> bool {」：码 2 用 16 KiB、码 1 与码 3 用 32 KiB、别的类恒假。这一函数与 CPU 的 `unit_bytes_of_the_class` 登记表同形（1、2、3 有长度，其余没有）。
- 对 CPU 逐项对照：

| 项 | CPU | GPU | 同不同 |
|---|---|---|---|
| 位置条目次序 | `child.locations` 次序（槽 0 先） | `device0/slot0/sum0` 先、`device1/slot1/sum1` 后（`parse_node_pointer` 同样按这个次序，`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:555`） | 同 |
| 头读多长 | `node_bytes()` 16 KiB | `NODE_BYTES` | 同 |
| 登记的类 | 1、2、3 | `w_unit_verifies_at_the_length_of_its_class` 里 1、2、3 | 同 |
| 自证长度 | 头里那一类的长度 | 同 | 同 |
| 自证过且类等于类型段那一类 | 交回 None | 交回 NONE | 同 |
| 自证过且是另一类 | 交回那一类 | 交回那一类 | 同 |
| 两份都不过 | None | NONE | 同 |
| 类型段不认得（names 为 None / NONE） | 交回自证过的类 | `class_the_inode_entry_type_names` 缺省 NONE，类永不等于 NONE，交回自证过的类 | 同 |

- 读码的限度：CPU 的「读不出」是 `reader.read` 返回 None，GPU 的是 `read_at` 返回 `version == NONE`（`read_at` 要起点是落点起点、长度不超过落点，`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:470`）；两边「读得出」的判据我按读码认为相同，**没有**在 GPU 上跑对拍（GPU 不跑，提交时由主 agent 跑）。
- 推翻 2.1 / 2.2 的现象：一个状态上 GPU 的 `w_child_unit_class_other_than_the_entry_type_names` 与 CPU 的 `child_unit_class_other_than_the_entry_type_names` 交回不同（照 GPU 逐位对拍的 `mismatches` 非空）；或 `read_at` 与 `reader.read` 在某一落点上一个读得出一个读不出。

### 2.3 第 24 批 K 那两行变异在新写法上还红不红（读码结论 + 我实跑）

- 行 2883（`crates/mutations.tsv:2883`，「第24批 K I-9.2 同形：孩子头里的类与类型段说的不符也当相符」）：原文 `        if Some(class_in_the_header) == class_the_entry_type_names {` 在新 walk.rs 里恰好 1 次（`crates/singlefs-checker/src/walk.rs:4129`），替换成 `!=`。读码：改成 `!=` 之后类相等的反而走「交回那一类」、类不等的交回 None（照类型段那一支读），名叫「inode 类型段 0、身份清零、孩子是叶容器」那条坏镜像就不再判出 I-9.2「类标签不符」。实跑：在我的拷贝里施加、只跑该行点名的测试：退出码 101、点名的 `an_inode_tree_entry_of_type_zero_with_a_zero_identity_over_a_leaf_container_reddens_only_the_inode_entry_invariant` FAILED（`expected-summary.txt` 第 1 行 `2883	1	101	1`）。**仍红**。
- 行 2884（`crates/mutations.tsv:2884`，「第24批 K 孩子的类不验整单元校验和」）：原文 `.is_some_and(|content| crc32_castagnoli_table(&content) == location.checksum);` 在新 walk.rs 里恰好 1 次（`crates/singlefs-checker/src/walk.rs:4125`：「.is_some_and(|content| crc32_castagnoli_table(&content) == location.checksum);」），替换成 `.is_some();`。实跑退出码 101，点名的 `an_inode_leaf_whose_class_tag_alone_says_index_node_is_read_at_the_length_of_its_entry_type` FAILED（`2884	1	101	1`）。**仍红**。
- 这两行的原文锚点在新写法下都还恰好命中一次（`scripts/mutate.sh` 的「命中数必须为 1」那一关过得去；我用自己的 `mutate_rows.py` 同样要求恰好 1，输出第 2 列都是 1）。
- 推翻：把这两行换到整张表一起跑，门禁 59 号报「没红」或「锚点命中数不是 1」。

## 3. K1 · H2：GPU 同改与对拍补的一形

- 同改点就是 2.2 的函数，没有别的 GPU 路径自带一份「类相等就早退」：GPU 里调这个函数的只有三处——`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2624`（inode 根）、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3032` 与 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3078`（墓碑 / 谱系树），三处都经同一个函数，所以一处改全覆盖。
- 对拍补的一形：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6855`：「fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_without_an_identity_names_a_leaf_container_whose_first_copy_is_stale(」。
  - 造流的函数 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:5876`：新池新建文件的流，inode 根第一条条目类型段 2 → 0、身份三段清零、重封到根，叶容器盘序小的那一份（`last_unit_copies(.., true)` 取设备号小的那一份，`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:5899`：「let first_leaf_copy = last_unit_copies(&flow, name, &inode_leaf_container, true)[0];」）前 16 KiB 换成改之前 inode 根的原样字节（头里类标签 2，恰是类型段 0 说的类），盘序大的不动。
  - 断言：CPU 红 I-9.2、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6873`：「!comparison.cpu_checker_red_invariants.contains("I-7.2"),」，每个状态 GPU 与 CPU 逐位一致（`mismatches.is_empty()`）。
- 这一形只钉「陈旧副本在盘序小的那一份」。**GPU 里第二份副本那一支的自证**（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2974`）没有哪一条对拍流或变异行钉住：要让第二份的早退（旧写法）露出来，要陈旧的在盘序大的那一份、盘序小的那一份完好或自证不过——这一形对拍里没有。读码推的：若把第二份那一行改成 `class_in_the_second_header == class_the_entry_type_names || w_unit_verifies_…`，当前这条对拍流上结果不变（盘序小的那一份是陈旧的、第二份是完好的，完好的第二份本来就自证过）。这一条是**测试射程缺口**，不是代码错；CPU 侧是同一个循环，两种盘序都被 harness 的坏镜像钉住（见第 5 节）。
  - 推翻：任何一条现有 GPU 对拍或变异行在第二份早退上红（读 `crates/mutations.tsv` 全表：第 26 批 GPU 那一行只改第一份，`crates/mutations.tsv:2936`）。
- GPU 对拍与 GPU 那一行变异按分工提交时由主 agent 跑，我没跑。

## 4. K1 · H3：inode 树读不出孩子，与容器树逐支比

调用点：`crates/singlefs-checker/src/walk.rs:2535`：「judge_an_inode_entry_whose_child_does_not_read_back(」，紧跟其后记走读失败 `crates/singlefs-checker/src/walk.rs:2541`：「.push("inode 树的孩子两份都读不到对得上的".to_string());」。新函数 `crates/singlefs-checker/src/walk.rs:4142`：「fn judge_an_inode_entry_whose_child_does_not_read_back(」。

「读不出」的定义两边同形：inode 树是 `child_unit_class_other_than_the_entry_type_names` 没认出另一类、`read_referenced_unit` 按条目类型段定的长度（类型段 2 读 32 KiB，其余读 16 KiB，`crates/singlefs-checker/src/walk.rs:2517`：「let (length, span_in_slots) = match entry_type {」）两份都读不到对得上的；容器树是 `child_unit_class_other_than_…` 没认出另一类、`read_unit_without_judging(.., node_bytes())`（类型段 0，`crates/singlefs-checker/src/walk.rs:2213`：「let child_class = read_unit_without_judging(self.reader, &child, node_bytes())」）或 `read_referenced_unit(.., data_unit_bytes(), ..)`（类型段 = 打包记录类型，`crates/singlefs-checker/src/walk.rs:2363`：「.push(format!("{name}的容器两份都读不到对得上的"));」那一支）读不到。

逐支：

| 类型段 | 容器树读不出孩子（`walk_the_entries_of_a_container_tree_node`，`crates/singlefs-checker/src/walk.rs:2157`） | inode 树读不出孩子（新函数） | 同不同 |
|---|---|---|---|
| 0 | `child_class` 为 None，`is_none_or` 让「子单元是码 2」那一小句成立（`crates/singlefs-checker/src/walk.rs:2218`：「&& child_class.is_none_or(…class == 2);」），判身份三段为 0；往下读节点读不出另记 I-2.1 / 走读失败 | `crates/singlefs-checker/src/walk.rs:4151`：「let identity_is_zero = birth_tree == 0 && container == 0 && container_birth == 0;」，`crates/singlefs-checker/src/walk.rs:4152`：「judgements.judge("I-9.2", identity_is_zero, …{」；调用方记走读失败 | 同（只判三段为 0；类标签那一句孩子读不出不判） |
| 容器 / 叶容器（容器树类型段 = 打包记录类型；inode 类型段 2） | 不判，记「容器两份都读不到对得上的」走读失败 | `crates/singlefs-checker/src/walk.rs:4148`：「InodeInternalEntryType::LeafContainer => {}」，不判，调用方记走读失败 | 同 |
| 不认得 | `crates/singlefs-checker/src/walk.rs:2253`：「self.judgements.judge(identity_invariant, false, …{」，无论孩子读不读得出都判红 | `crates/singlefs-checker/src/walk.rs:4158`：「InodeInternalEntryType::Unregistered(unregistered_type) => {」判 I-9.2 假 | 同（结论相同；inode 树该支读得出孩子时仍走原来的「类型段不认得」那一支，判假） |

细节差：容器树不认得那一支在读之前按层级登记引用（层级 ≥ 2 一槽、否则两槽），inode 树该支一律按一槽登记（`crates/singlefs-checker/src/walk.rs:2495`：「InodeInternalEntryType::Unregistered(_) => None,」那一段之后的 `(node_bytes(), 1)`）。这是实现员报告「交主 agent」第 1 条已排第 27 批的事，不在这一轮，只记一笔。

GPU 同判：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2632`：「if (s.version == NONE) {」之内：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2635`：「if (record_type == 0u) {」判身份三段为 0；`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2637`：「} else if (record_type != 2u) {」判 I-9.2 假；类型段 2 不判；之后 `w_failures` 加 1。身份三段读法 `rd64(root.base, o + 8u)`、`o + 18u`、`o + 26u` 与 CPU 的 `read_u64(entry, 8/18/26)` 同。与 CPU 逐支一致。

GPU 这一支没有对拍形（已排第 27 批），我没法用 GPU 结果印证，只能读码；读码一致。

- 推翻 H3：一张镜像上容器树读不出孩子给出的 I-9.2 结论与 inode 树读不出孩子给出的不同（同一类型段、同一身份三段）；或 GPU 与 CPU 在读不出孩子这一支上 I-9.2 的位不同（要在 GPU 上跑才能看到）。

## 5. K1 · 测试：6 条坏镜像

我在自己的拷贝（冻结副本复制）里编译并跑了 `cargo test -p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back`：

```
test an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant ... ok
test an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant ... ok
test an_inode_tree_entry_of_type_zero_over_a_leaf_container_with_a_stale_copy_of_the_child_reddens_the_inode_entry_invariant_on_either_device ... ok
test a_tombstone_tree_entry_of_type_zero_over_a_container_with_a_stale_copy_of_the_child_reddens_the_tombstone_entry_invariant_on_either_device ... ok
test a_tombstone_tree_entry_of_the_container_type_over_a_node_with_a_stale_copy_of_the_child_reddens_the_tombstone_entry_invariant_on_either_device ... ok
test a_lineage_tree_entry_of_type_zero_over_a_container_with_a_stale_copy_of_the_child_reddens_the_lineage_entry_invariant_on_either_device ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 124 filtered out;
```

（上面是 `/tmp/claude-1000/m3-batch26-code-r1-sonnet/run-base.log` 的原样测试行；复跑后同样的 6 行在 `expected-base.txt`。）这一筛选串恰好选中 6 条，不多不少（`--list` 输出 `6 tests, 0 benchmarks`）。

H1 四形共用的判法：`crates/singlefs-harness/tests/checker_known_bad_images.rs:10682`：「fn assert_only_the_entry_invariant_adds_to_the_stale_copy_on_either_device(」。逐盘序：
- 先判「只有陈旧副本、条目不改」那张对照：`crates/singlefs-harness/tests/checker_known_bad_images.rs:10696`：「!violated_by_the_stale_copy_only.contains(entry_invariant),」；
- 红的集合恰好等于对照红的集合并上目标：`crates/singlefs-harness/tests/checker_known_bad_images.rs:10706`：「violated, expected,」；
- 不连带 I-7.2：`crates/singlefs-harness/tests/checker_known_bad_images.rs:10710`：「!violated.contains("I-7.2"),」；
- 两种盘序红的集合相同：`crates/singlefs-harness/tests/checker_known_bad_images.rs:10716`：「violated_on_each_device[0], violated_on_each_device[1],」。
与正文 K1 要求的「= 只有陈旧副本那张对照红的 ∪ 目标、不连带 I-7.2、两种盘序相同」逐字对上。

四形各一条：墓碑 N4-i `crates/singlefs-harness/tests/checker_known_bad_images.rs:10724`、层级 2 墓碑 N4-f `crates/singlefs-harness/tests/checker_known_bad_images.rs:10762`、谱系 `crates/singlefs-harness/tests/checker_known_bad_images.rs:10810`、inode `crates/singlefs-harness/tests/checker_known_bad_images.rs:10849`，与判决第三节第 1 条要的「第 391 / 392 行那一对与 T2、inode 那两形各做成坏镜像（两种盘序都只红目标）」同数（N4-i 一对 + T2（N4-f）+ inode + 谱系）。

读不出孩子两形：类型段 0、身份不动 `crates/singlefs-harness/tests/checker_known_bad_images.rs:10912`、类型段 7 `crates/singlefs-harness/tests/checker_known_bad_images.rs:10937`，共用 `crates/singlefs-harness/tests/checker_known_bad_images.rs:10887`：「fn image_whose_inode_root_entry_names_a_child_that_does_not_read_back(」，把叶容器头里类标签写成 9（不登记）。断言查到违例说明里的原文（`crates/singlefs-harness/tests/checker_known_bad_images.rs:10928`：「if detail.contains("孩子两份都读不出") && detail.contains("（要三段为 0）")」），不只查红。

## 6. K1 · 7 行变异：点名的测试与参数筛不筛得到、会不会红

表在 `crates/mutations.tsv` 2930–2936 行（冻结副本）。我在自己的拷贝里逐行施加（原文必须恰好命中一次，施加后还原），跑该行 `-p … --test … -- <筛选串>`，数「FAILED」的测试，并核该行第 6 列点名的那条测试是不是在红的里面：

| 行 | 开头 | 命中 | cargo 退出码 | 红的测试数 | 点名的测试红了 |
|---|---|---|---|---|---|
| 2930 | `crates/mutations.tsv:2930`：「第26批 H1 头里的类就是类型段那一类的副本不验整单元校验和（墓碑树 N4-i」 | 1 | 101 | 4 | 是 |
| 2931 | `crates/mutations.tsv:2931`：「第26批 H1 头里的类就是类型段那一类的副本不验整单元校验和（层级 2 墓碑树 N4-f」 | 1 | 101 | 4 | 是 |
| 2932 | `crates/mutations.tsv:2932`：「第26批 H1 头里的类就是类型段那一类的副本不验整单元校验和（谱系树类型段 0」 | 1 | 101 | 4 | 是 |
| 2933 | `crates/mutations.tsv:2933`：「第26批 H1 头里的类就是类型段那一类的副本不验整单元校验和（inode 树类型段 0」 | 1 | 101 | 4 | 是 |
| 2934 | `crates/mutations.tsv:2934`：「第26批 inode 树读不出孩子时 I-9.2 不判（类型段 0 身份三段不为 0的条目」 | 1 | 101 | 2 | 是 |
| 2935 | `crates/mutations.tsv:2935`：「第26批 inode 树读不出孩子时 I-9.2 不判（类型段不认得的条目」 | 1 | 101 | 2 | 是 |
| 2936（GPU） | `crates/mutations.tsv:2936`：「第26批 GPU 判器 H1：第一份副本头里的类就是类型段那一类时不验整单元校验和」 | 1（wgsl 里原文恰好 1 次，我用 python 数的） | 没跑（GPU 不跑） | — | 没跑 |

- 2930–2933 四行是同一个替换（`if !verifies_… { continue; }` 改成「类等于类型段那一类就不 continue」，等价于改之前的早退），点名的测试各不相同；施加一次，四条 H1 坏镜像都红，所以每条坏镜像单独就分得出这一变异。2934 与 2935 同理是同一个替换（拿掉调用），两条读不出孩子的坏镜像都红。
- 第 6 列点名的测试与参数筛不筛得到：每行的筛选串 `stale_copy_of_the_child child_that_does_not_read_back` 选中的就是上面 6 条（`--list` 6 条，且第 6 列六个名字都在里面）；GPU 行的筛选串是那条对拍测试的完整函数名，`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 里这个名字只有那一个函数加它自己里面的 `test_name` 字符串（`grep -c` 为 2：函数名一次、字符串一次），选得到一条。
- 复跑数：`research/prompts/m3-batch26-code-r1-sonnet-model/expected-summary.txt` 前 8 行（含第 24 批 K 的 2883、2884）。
- 推翻：把我的 `mutate_rows.py` 换成 `scripts/mutate.sh` 在这几行上有一行「没红」。

### 6.1 我自己加的额外变异（探测测试够不够尖，不是第 26 批的行）

在同一份拷贝上加了 9 条（`extra-rows.tsv`），结果在 `expected-summary.txt` 的 `extra-` 行：

| 额外行 | 改法 | 筛选 | 结果 |
|---|---|---|---|
| 1 A | 读不出孩子、类型段不认得那一臂 `false` 改 `true` | 6 条筛选 | 红 1 条（不认得那一条） |
| 2 B | 读不出孩子、类型段 0 身份判定恒真 | 6 条筛选 | 红 1 条（身份那一条） |
| 3 C | 头自证不过就 `return None`（不看下一份） | 6 条筛选 | 红 4 条（四条 H1 坏镜像） |
| 4 D | 类型段 0 的身份只判出生树 | 6 条筛选 | **没红**（退出码 0） |
| 5 D2 | 只判容器号 | 6 条筛选 | **没红** |
| 6 D3 | 只判出生代 | 6 条筛选 | **没红** |
| 7 E | 类型段 2（叶容器）读不出时也判 I-9.2 红（过判） | 6 条筛选 | **没红** |
| 8 E-file | 同 E | 整份 `checker_known_bad_images`（130 条） | **没红**（130 passed） |
| 9 D2-file | 同 D2 | 整份 `checker_known_bad_images` | **没红** |

结论：
- 读不出孩子这支，类型段 0 那条坏镜像只改了类型段、身份三段原样（容器号、出生代不为 0，出生树也非 0），所以三段「全为 0」里任一段单独丢了判定（D / D2 / D3）这条测试分不出来；类型段 2 那一支「读不出时不判」（E）整份 `checker_known_bad_images.rs` 里也没有一条钉它。
- 这是**测试射程缺口**，不是代码错：读码上新函数四支与容器树逐支相同（第 4 节）。缺口的后果是：以后有人把 `identity_is_zero` 改成只判一段、或让类型段 2 读不出时也判红，这批坏镜像不会响。
- 我没跑整张 harness（170 个测试二进制）去看别的测试文件有没有钉住 E 或 D 系，所以「整个仓都分不出」写不得；只写「`checker_known_bad_images.rs` 这一份分不出」。
- 推翻：别的 harness 测试文件里有一条在 E / D2 / D3 上红（要主 agent 在整张表上跑，或我再对 E 跑整个 harness 档的 `--tests` 范围）。

## 7. K3 · 被判的文件名单（5 份各一格）

名单 `research/prompts/m3-batch26-code-r1-snapshot/crates-sha256.txt`，冻结副本逐份 `sha256sum -c` 五行 OK；每份判到的格：

| # | 文件 | 判到的格 | 判定 |
|---|---|---|---|
| 1 | `crates/singlefs-checker/src/walk.rs` | H1 CPU 第 2.1 节；H3 CPU 第 4 节；第 24 批 K 两行的锚点与红（2.3） | 一致 |
| 2 | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` | H1 GPU 第 2.2 节、H3 GPU 第 4 节；GPU 没跑 | 读码一致；没对拍 |
| 3 | `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 第 3 节：对拍补的一形（造流、断言）；没编、没跑 | 读码一致；射程缺口一处（第二份副本那一支） |
| 4 | `crates/singlefs-harness/tests/checker_known_bad_images.rs` | 第 5 节 6 条坏镜像（编、跑、变异都做了）；第 6.1 节射程缺口 | 一致；射程缺口两处（D 系、E） |
| 5 | `crates/mutations.tsv` | 第 6 节 7 行：6 行 CPU 实跑都红、点名的测试都在红里面、筛选串选得到；GPU 行读码；表格式（只有开头五行注释不是六列，新追加 7 行都是六列，我用 awk `NF!=6` 数得 5 行）；7 行与实现员的追加文件行数相同 | 一致；GPU 行没跑 |

K3 不是替主 agent 采纳或出判决，只是每份一格判到。

## 8. 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 H1 CPU 对判决 | 一致 | 位置条目次序、按头里那一类长度自证、验不过看下一份、两份都不过交回 None，与判决第三节第 1 条字面相同（`crates/singlefs-checker/src/walk.rs:4126`） |
| K1 H1 GPU 对 CPU | 读码一致 | 长度、次序、两份都不过交回 NONE 逐项相同；GPU 没跑，没对拍 |
| K1 第 24 批 K 两行变异 | 仍红 | 2883、2884 在新写法上锚点各命中 1 次，点名的测试都红（实跑） |
| K1 H2 对拍补的一形 | 一致；有射程缺口 | 只钉「陈旧副本在盘序小的那一份」；GPU 第二份副本那一支的自证没有对拍流也没有变异行钉住（推的） |
| K1 H3 CPU 对容器树 | 一致 | 类型段 0 / 容器 / 不认得三支与容器树读不出孩子逐支相同；引用登记的槽数差一处属第 27 批 |
| K1 H3 GPU | 读码一致 | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2632` 之内三支同判；没有对拍形（已排第 27 批） |
| K1 测试：6 条坏镜像 | 一致 | 实跑 6 条全绿；红的集合 = 对照红的 ∪ 目标、不连带 I-7.2、两种盘序相同 |
| K1 测试：7 行变异 | CPU 6 行红；GPU 没跑 | 点名的测试都在红里面、筛选串恰好选 6 条 |
| K1 测试的尖度 | 缺口两处 | 读不出孩子那两条坏镜像分不出「身份三段只判一段」与「类型段 2 读不出也判红」（我加的额外变异 D / D2 / D3 / E 没红） |
| K3 5 份 | 5 份各一格 | 第 7 节表 |

## 9. 没做什么

- 没跑 GPU：对拍 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 与 GPU 变异行 2936 提交时由主 agent 跑；我也没编 `singlefs-checker-tier` 的测试（没跑它的编译，所以「第 3 节的对拍测试能编」靠主 agent 的合入验证「170 个测试二进制全绿」，我没独立核）。
- 没攻（K2 归攻方腿）；没找新反例；GPU 与 CPU 判得不一样的一形这里只做了读码对照，没做反例搜索。
- 没替主 agent 采纳或出判决；上面的「缺口」是线索，由主 agent 现查。
- 没跑整张 harness 去验额外变异 E / D 系有没有别的测试文件钉住（第 6.1 节）。
- 没对 kb 写回：`.claude/kb/invariants.md:289` / `:304` / `:310` 没有「读不出孩子怎么判」与「同类也自证」的字句，这是「规则没说」的两处，是否要写回归主 agent。
- 自己的两份冻结副本拷贝与两个编译目录（`/tmp/claude-1000/m3-batch26-code-r1-sonnet/{tree,target}`、`/tmp/claude-1000/m3-batch26-code-r1-sonnet-rerun/{tree,target}`）交回前已删；日志留在 `/tmp/claude-1000/m3-batch26-code-r1-sonnet/` 与 `/tmp/claude-1000/m3-batch26-code-r1-sonnet-rerun/logs/`（报告引的 `run-base.log` 在前者里；不入库：只是跑的原始输出，要核就跑 `rerun.sh`）。
