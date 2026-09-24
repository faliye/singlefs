# m2-newq-r1 核查员报告

**你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。**

## 〇、判别力自证

待核引用：opus 报告第 391 行声称 `.claude/kb/decisions/19-块指针的结构与宽度预算.md` 第 99 行是
「1. **释放一律经映射，不经提示**；经映射核到那条映射条目之后，释放之前还要按它位置项里带的单元校验和读盘核一次。**核出对不上时隔离那个槽，发布照成**：逻辑上照样释放（映射条目去掉），物理槽不还回空闲池，记进隔离并计数报出。」

自证：把声称行号人为加 1（99 → 100），按同一条核法（`awk 'NR==行号'` 取该行、逐字比对声称原文）去核第 100 行。

```
$ awk 'NR==100' ".claude/kb/decisions/19-块指针的结构与宽度预算.md"
2. **单元头或提示必须带块 birth**——今天已满足：D18（块里携带什么信息） 已定项 7 的数据单元类身份段本来就含诞生代号 8 字节。
```

第 100 行内容与声称原文完全不同（讲的是「单元头或提示必须带块 birth」，不是「释放一律经映射」），逐字比对不相等 ⇒ **判 ✗**。核查方法能分辨，通过自证；真实的行号 99 核对时逐字相同（见下文表格），判 ✓。

## 一、输入与快照核对（观测，不是判决）

- 三条腿报告 sha256（现算）：
  - `m2-newq-r1-opus-output.md`：`d44c6edaf787d4e42617ea960e358b11d66be9013df85c2c03bd937050bdc348` —— 与主 agent 给的一致。
  - `m2-newq-r1-sonnet-output.md`：`995001527a57eaeb8cc67a7e783a33dc62ca2aa6e36a309ea07810616b822ced` —— 与主 agent 给的一致。
  - 本地攻方 4 份文件（提示、核对表、s1、s2）sha256 与 `m2-newq-r1-local-attack-runlog.md` 里记的逐一现算相同（见第四节）。
- 开工快照 `research/prompts/m2-newq-r1-snapshot/sha256sums.txt` 里 15 个文件，逐个现算：
  - 11 个（6 份 kb 决策 + `verification-build.md` + 2 份 research/prompts 旧材料）与**主工作区现在的内容**逐字节相同。
  - `.claude/kb/checks-owed.md`、`.claude/kb/milestone/02-second-txn.md` 与快照不同（与 `reconstruction.md` 所述一致：主 agent 同日直接改的，不倒推）；两条腿引到这两份文件的地方按当前主树核，逐条见下文，未见 ✗。
  - 4 个 `crates/singlefs-core/src/*.rs`（`transaction.rs`、`recovery.rs`、`allocator.rs`、`mount.rs`）与主树不同；`reconstruction.md` 给的倒推树 `/tmp/claude-1000/m2-newq-r1-reconstructed/crates/` 上，这 4 个文件的 sha256 与 `sha256sums.txt` 逐个重算相同（独立复核，不是照抄 `reconstruction.md` 的结论）：
    ```
    4ac6ea56a91f90ca0241cc05e69057ae67a4592e8e0056a70e0ec4c0b4ced168  crates/singlefs-core/src/transaction.rs
    6c380575ba2d9459fd797ec9015f68fed967328290430ea5dc87549c49d8f832  crates/singlefs-core/src/recovery.rs
    5a49002f14dfa13896ac6c1e7f0e1ca0994a159800e6dbec4787abcde3e45e79  crates/singlefs-core/src/allocator.rs
    96c8d8d8b8ec4f996db9f265cc1dd541d309050a5206cbf81520b442c7482b4d  crates/singlefs-core/src/mount.rs
    ```
  - **重要发现**：倒推树里除这 4 个文件外，`singlefs-format/src/lib.rs`、`singlefs-harness/src/bin/e156_allocation_basis_counts.rs`、`singlefs-harness/src/bin/e158_root_choice_repair.rs` 三个文件也与主树不同——这不在 `reconstruction.md` 列出的「8 份补丁」范围内，是另一个并发会话（`ps` 现查到 `impl-m2-mountfix`、`e156-s4` 等会话）同一时段独立改的。**这意味着「腿引的 crates/ 行号对着倒推树核」这条指令的射程比 `reconstruction.md` 字面列的 4 个文件更宽**：`crates/singlefs-harness/` 下的文件同样可能已经被别的改动错位，逐条见下文——sonnet 报告引的 `scenario.rs`、`first_transaction_on_device.rs`、`first_transaction_step_five_publish.rs` 等文件，按主树核全部不对，按倒推树核全部逐字相同（详见第三节），这正是本轮踩到的一个真实案例。

## 二、云端攻方（Opus）腿核对表

### 2.1 kb 条文引用（第七节「引文」，5 处整行抄）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| D19（块指针的结构与宽度预算）:99，硬规则 1 全文 | ✓ 逐字相同 | `awk 'NR==99' .claude/kb/decisions/19-块指针的结构与宽度预算.md` |
| D23（journal 的角色与格式）:371，D23 已定项14 失败表整段 | ✓ 逐字相同（含嵌套引用 D2 已定项13 第 225 行、及「不是已定项 15」的旁注，现查 D2:225 确实归已定项 13「降级期间只读」） | `awk 'NR==371' ...23-journal的角色与格式.md`；`awk 'NR==225' ...02-RAID条带策略.md` |
| D23:373，切换预留量整段 | ✓ 逐字相同 | `awk 'NR==373' ...` |
| D28（挂载期承诺量）:27，第九项取值句整行 | ✓ 逐字相同 | `awk 'NR==27' ...28-挂载期承诺量.md` |
| D3（空间分配）:123，跨度段字段表整行 | ✓ 逐字相同 | `awk 'NR==123' ...03-空间分配.md` |

### 2.2 `crates/` 代码行引用

第四节「今天的两条判据」明写「主工作区 2026-09-24 04:30 UTC 现查」，即引的是**当时的主树**（补丁已打完），不是这条腿自己的副本基线；第十节「主工作区落了正式实现」同样明写是现查主树。这两处按**当前主树**核（与开工快照/倒推树无关，因为它们本来就不是在描述副本基线）：

| 引用 | 核的结果 |
|---|---|
| `recovery.rs:1038` `data_unit_bytes_in_file_order.push(read_unit_via_locations(` | ✓ |
| `recovery.rs:1127` `let (key, _locations) =` | ✓ |
| `transaction.rs:1493` `pub fn placements_to_release_via_mapping(` | ✓ |
| `transaction.rs:1849` `ReleaseChecksumReadFailedWhoseHandlingIsUndecided {` | ✓ |
| `allocator.rs:320` 隔离注释整行 | ✓ |
| `allocator.rs:858` 断言字符串整行 | ✓ |
| `allocator.rs:1123` `.expect(...)` 整行 | ✓ |
| `allocator.rs:347` `pub fn quarantine_after_release_checksum_mismatch(...)` | ✓ |
| `admission.rs:316` `abandoned_root_exclusive: ...` | ✓ |
| `mount.rs`、`admission.rs` 里 `grep -c quarantin` 各 0 次 | ✓（现查两个文件均为 0） |

### 2.3 模型目录完整性

`M/out*/`、`patch*/` 全部 116 个文件，逐文件重算 sha256，与报告第 36–151 行的表**整表比对**（`diff` 后排序结果一致，0 差异）；`baseline-crates.tgz` 解包后 91 个文件的 sha256 与随附的 `baseline-crates.sha256` 逐一比对，0 差异。

`baseline-crates.tgz`（opus 自称基线 2026-09-24 01:22 UTC）与 `reconstruction.md` 倒推出的 00:52 UTC 快照树整体 diff：**除 `mutations.tsv`（opus 自己声明不含）外，仅 3 个文件不同**——`singlefs-format/src/lib.rs`、`e156_allocation_basis_counts.rs`、`e158_root_choice_repair.rs`（与第一节发现的并发会话改动一致，均非 opus 分析涉及的文件），opus 分析用到的 4 个 `singlefs-core` 文件在两份基线里完全相同。

### 2.4 重点复跑（N1 甲/乙/丙/丁 × F1/F2/F3/F9，N2/N3/准入）

做法：把 `M/patch/apply.py`、`patch/apply2.py`、`patch/m2_newq_attack.rs` 原样拷进自己的草稿目录
`/tmp/claude-1000/m2-newq-r1-verifier/rerun/`，对 `baseline-crates.tgz` 解包出的基线打上这两个补丁（不在腿的原目录里跑），
`nice -n 19 cargo test -p singlefs-harness --test m2_newq_attack --no-run` 编译，再用与报告完全相同的环境变量
（`TMPDIR=/dev/shm/verifier-x NEWQ_OUT=... NEWQ_MAXLEN=4 NEWQ_WORKERS=4`，`/dev/shm` 用独立子目录，未写满共享的 `/dev/shm`）
真跑，比对产物。

- **编译**：`Finished test profile ... in 14.58s`，无警告无错误。
- **F6、F7 两个故障 × 全部 11 个共同臂（甲三变体、乙 any/every、丙 any/every、N3 三变体、off）× 120 条后缀，全部 16 列逐字节比对**：
  ```
  $ diff orig-f6f7.txt mine-f6f7.txt   # 各 2640 行，排序后逐字节比
  (无输出，diff exit 0)
  ```
  完全重现，包括 F7 的「58/120 panic」（驱动记成 `final_mount=skip`）与 F6 的挂载失败模式。这是**从源码重新编译、重新跑出来**的独立复现，不是读现成 tsv。
- **F1、F2、F3、F9 上的数**（本节因 F1–F5 顺序在这次重跑里排在 F6/F7 之后，尚未跑完全部即已达成 F6/F7 的完整交叉验证；F1–F3/F9 的具体计数改用现成产物逐字段现算复核，做法见下）：直接在 `M/out/sweep-len4.tsv`、`M/out3/p4-F{9}-*.tsv`、`M/out4/p4-F{1,2,3,9}--N1D-*.tsv` 上用 `awk`/`grep` 现算（**不是抄报告里的数字，是对存量 tsv 重新算一遍**），逐格结果：

| 候选 × 故障 | 报告称 | 现算结果 | 命令 |
|---|---|---|---|
| 甲 × F1 | 58/120 `isolated_by_check>0` | **58/120** ✓ | `awk -F'\t' 'NR>1 && $1=="F1-transient-eio-both" && $2=="N1A-any-isoAll-mem" && $9+0>0' sweep-len4.tsv \| wc -l` |
| 乙 × F1（发布失败交回后 O 照成） | 定性描述，无数字 | 58 条 `t1@...:publish-failed` 后转 ok，4 条挂载失败 ✓与描述一致 | 见报告第三节引用行 |
| 丙 × F1（switched） | 58/120 switched | **58/120** ✓ | `awk -F'\t' '...$2=="N1C-any" && $14+0>0' \| wc -l` |
| 丁 × F1 | 0/120 隔离 | **0/120** ✓ | `awk -F'\t' '...$9+0>0' p4-F1--N1D-...iso-mem.tsv \| wc -l` |
| 乙 × F2（全程 ReleaseCheckReadFailed） | 58/120 | **58/120** ✓ | `grep -c ReleaseCheckReadFailed`（按行统计） |
| 丙 × F2（failure-table 转只读） | 58 条 | **58/120** ✓ | `grep -c failure-table` |
| 甲/丁 × F2 | 不错；40 条以 O 开头无一步报错 | **40/40 无 err** ✓ | 见第二节 |
| 丁 × F2（隔离位） | 「同甲，58/120 隔离」 | **58/120** ✓（现算，与 F1/F3 对照确认「丁≠恒0」，主 agent 派发提示里的「丁 0 条」对应的是 F3 那一格，非全部故障恒 0——见下） |
| 丁 × F3 | 0/120 隔离 | **0/120** ✓ | 与派发提示「丁 0 条」吻合 |
| 丁 × F9 | 隔离位未在正文单列数字 | **58/120** 隔离（现算，与「不错（发布照成）」定性一致，不矛盾） | |
| 乙 × F9 | 120/120 最终挂不上 | **120/120** ✓ | |
| 丙 × F9 | 120/120（58 条转只读） | **final_mount err 120/120** ✓ | |
| 甲/N1A × F9（"今天"基线，同 N2 今天各自判） | 62/120 | **62/120** ✓ | |
| N2 各自判(今天) × F1/F2/F9 先重挂 | 62/120 挂不上 | **62/120** ✓（用同一份 N1A-any-isoAll-mem 数据） | |
| N2 读法乙 × F8 | 120/120 | **120/120** ✓ | `p4-F8-N2-rebuild-checks-mapping-refuses.tsv` |
| N2 读法乙 × F10 | 40/120 | **40/120** ✓ | `p4-F10-N2-rebuild-checks-mapping-refuses.tsv` |
| N2 今天 × F8 | 58/120 | **58/120** ✓ | `p4-F8-off(today).tsv` |
| N2 今天 × F10 | 0/120 | **0/120** ✓ | `p4-F10-off(today).tsv` |
| N3 只在内存 × F10 tailM | 40/120 | **40/120** ✓ | `tailM-F10-N1A-any-isoAll-mem.tsv` |
| N3 只在内存 × F9 tailM | 58/120（120 减去先重挂的 62） | **58/58 全部 writes_refused>0**，与今天同为 62/120 mount-err ✓ | 见下方「⚠️」 |
| 准入「每段每盘差 2 槽」 | 隔离[2,2]/未回收[0,0]=42、已回收[2,2]=22（只在内存/重挂现算）；「落点」73/656 | **42、22、73、656** 全部 ✓ | `long-F4-N1A-any-isoAll-mem.tsv`、`long-F4-N3-iso-persisted.tsv` |

⚠️ **发现（观测，非判决）**：Section V 表格里「重挂现算」列在 F9 的 tailM（58/120）与 tailFO（32/120）两格，
在模型目录里**找不到对应的产物文件**：`out3/` 下只有 `tailM-F9-{N1A,N3-iso-persisted,N3-keep-allocated,off(today)}.tsv`
与 `tailFO-F9-{同上}.tsv`，唯独没有 `tailM-F9-N3-iso-recompute.tsv` / `tailFO-F9-N3-iso-recompute.tsv`；
仅有的两份含 `N3-iso-recompute` 且故障为 F9 的文件是 `longsmoke-F9-N3-iso-recompute.tsv`（4 行）与
`smoke-F9-N3-iso-recompute.tsv`（6 行），都是小样本探路，不是 120 条系统性 tailM/tailFO 扫描。
报告第 366 行给出了机制性论证（「现算臂无从算起」，释放发布之后盘上没有痕迹可算，因此重挂现算在这个故障上
退化成与「只在内存」相同的行为），这个论证本身可信，但**报告在表格里把这两格的数字用粗体呈现，
格式上与其余测出来的格无法区分**，且没有像第 333 行 F11 那格那样标注「没跑（推的：同）」。
判据：这两格没有产物支持 ⇒ **✗（产物缺失，非数字错误）**；F10 同一位置的「重挂现算」（tailM 0/120、tailFO 0/120）
是**真有**产物的（`out3/tailM-F10-N3-iso-recompute.tsv`、`tailFO-F10-N3-iso-recompute.tsv` 均存在且核对无误）。

另：Section V 准入子表里的行标「落点」与候选定义清单（只在内存/落盘/重挂时现算/记录留在已分配/off）对不上——
数字（73/656/0）核对后精确匹配 `N3-iso-persisted.tsv`（也就是候选「落盘」）的实测值，
**「落点」应为「落盘」的笔误**，不影响数字本身的正确性，但会让读表的人对错候选。

## 三、云端正推（Sonnet）腿核对表

### 3.1 N1–N3、N5–N7 的 kb 引用

| 引用 | 核的结果 |
|---|---|
| D19:105（读盘失败/rebuild_version/隔离跨挂载均「没有条款」） | ✓ 逐字相同，N1/N2/N3 三格共用同一行，三句连续出现在原文同一句里，引用切分准确 |
| D19 已定项 5 范围 87–115 | ✓ 边界准确（87 起 115 止，116 起是已定项 6） |
| D19 已定项 8 范围 160–183 | ✓ 边界准确 |
| D23:386（切换重新读盘择根三件没有条款，Clause F 同一句） | ✓ 逐字相同 |
| D23:369（管理员回退候选集定义） | ✓ 逐字相同 |
| D23 已定项 14 范围 342–406 | ✓ 边界准确 |
| D23:377（实例切换主句） | ✓ 逐字相同 |
| **D23:515（N5 段落称「实例切换」段落「附近」）** | **✗**：第 515 行实为「#### 已定项 21：世代号」小节下的字段表表头（`\| 字段 \| 粒度 \| 用途 \|`），与实例切换完全无关。同一份报告 N6 引用同一段主文字时正确写了 `:377`，N5 却写成 `:515`，是同一处内容的两个不同行号，其中一个必错 |
| D18 已定项 11 范围 248–336 | ✓ 边界准确 |
| checks-owed.md:403（C458） | ✓ 逐字相同 |
| checks-owed.md:281（C318，用于区分「被抛弃根独占量」与「校验和核不上」两类隔离） | ✓ 逐字相同，且引用的区分点（两种隔离不能互相顶替）在原文里确有依据 |
| D28 已定项 1 范围 14–53 | ✓ 边界准确 |
| `research/prompts/c381-r3-main-verification.md:29-34、57-66` | ✓ 内容与转述一致（非逐字整段引用，是复述性归纳，报告本身没有声称是整行抄） |

### 3.2 N4 段落引用（这一段是本轮发现最密集的地方）

**关键方法说明**：N4 涉及的 8 个 `crates/` 文件里，只有 `allocator.rs` 是 `reconstruction.md` 明写的 4 个倒推文件之一；
`scenario.rs`、`first_transaction_on_device.rs`、`first_transaction_step_five_publish.rs`、
`second_transaction_step_one_overwrite.rs`、`second_transaction_supplement_two_commit_generated_fallback.rs`
**没有被 `reconstruction.md` 列出**，但按第一节的发现，这些文件同样被同一批或另一批改动移动过行号。
**按主树核，这 5 个文件的引用全部对不上**；但一一在 `/tmp/claude-1000/m2-newq-r1-reconstructed/crates/` 上核，**全部逐字相同**。
这是任务说明里「腿引的 crates/ 行号对着这棵倒推树核，不对着主树核」这条指令在本轮里最直接对应的一处，
写进这里防止误判——**不核倒推树、只核主树，会把 sonnet 这段全部错判成 ✗，而它其实全对**。

| # | 引用 | 按主树核 | 按倒推树核 |
|---|---|---|---|
| 1 | `allocator.rs:629` `pub policy_mismatches: u64,` | ✗（主树该行是无关函数签名，字段已挪到 802 行） | **✓** |
| 2 | `allocator.rs:655` `policy_mismatches: 0,` | ✗（挪到 831 行） | **✓** |
| 3 | `allocator.rs:1157` 断言 | ✗（挪到 1495 行） | **✓** |
| 4 | `scenario.rs:61` 字段声明 | ✗（该文件在主树上多出一个 `FirstTransactionPathStep` 枚举等约 60 行新代码，字段挪到 62 行） | **✓** |
| 5 | `scenario.rs:133` 赋值 | ✗（挪到 206 行） | **✓** |
| 6 | `first_transaction_on_device.rs:780-781` 打印格式与参数 | ✗（挪到 1091-1092 行） | **✓** |
| 7 | `first_transaction_region_bytes.rs:115,118` | **✓**（两棵树上这个文件都没变） | ✓ |
| 8 | `first_transaction_step_two_data_unit.rs:296` | **✓**（同上） | ✓ |
| 9 | `first_transaction_step_five_publish.rs:1018-1027` 元组断言 | ✗（真实断言在 1034-1035，挪动约 16 行） | **区间偏差**：倒推树上真实断言块是 1021（注释）–1028（分号），报告给的 1018-1027 与真实块有 3-4 行重叠但两端都未对齐，判**分不清：区间划定不精确**，不判满分 ✓ 也不判 ✗ |
| 10 | `second_transaction_step_one_overwrite.rs:169` | ✗（真实在 171） | **✓** |
| 11 | `second_transaction_supplement_two_commit_generated_fallback.rs:144` | ✗（真实在 146） | **✓** |
| 12 | `.claude/gate.d/55-qemu-first-transaction.sh:183` | ✓（不受 crates 补丁影响） | ✓ |
| 13 | `.claude/kb/verification-build.md:76` | ✓ | ✓ |
| 14 | `.claude/kb/checks-owed.md:456`（C515，前置列＝「无」） | ✓（此文件是主 agent 直接改的，按当前主树核） | — |

**结论**：按正确的核对基准（倒推树），14 条里 12 条精确 ✓、1 条区间边界有 3-4 行误差（内容仍指向同一处断言，不是指错文件或指到无关代码）、1 条本就该按主树核（checks-owed.md）也 ✓。**没有一条真正指向错误内容。**

### 3.3 N4 的其余引用与自证命令

| 引用 | 核的结果 |
|---|---|
| D3（空间分配） 已定项 8 范围 154-178，射程句「第一版盘不等大时…」 | 内容 **✓**（现查主树，逐字相同，见下方命令），但报告自己贴出的 `grep -n` 输出**行号是 768，而该文件总共只有 316 行，768 根本不可能存在**；真实行号是 164。这不是「不同基线导致的行号漂移」——`03-空间分配.md` 不属于 crates/，不受补丁影响，两棵树上都是 164 行，**是报告贴出的命令输出本身与现查结果不符** |
| `records/2026-09-19-里程碑二遗留收拢.md:130`（旁证「主 agent 按推荐删掉」） | 内容存在但**行号错**：130 行是问题清单第 5 行（只问不答），真正写「主 agent 按推荐删掉那个计数」的是第 151 行（用户答复表最后一行）|
| `milestone/02-second-txn.md:353` | ✓（此文件是主 agent 直接改的，按当前主树核，逐字相同，含 `allocator.rs` 第 494 行、`verification-build.md` 第 76 行等交叉引用） |
| checks-owed.md:456（C515 全文） | ✓ |
| verification-build.md:76 | ✓ |
| 自证命令 `grep -rln "policy_mismatches" crates/ --include="*.rs" \| grep -v "^./research"` 现查命中数 | 报告称「7 个文件」，**现查命中 8 个**（`allocator.rs`、`scenario.rs`、`first_transaction_on_device.rs`、`first_transaction_region_bytes.rs`、`first_transaction_step_two_data_unit.rs`、`first_transaction_step_five_publish.rs`、`second_transaction_step_one_overwrite.rs`、`second_transaction_supplement_two_commit_generated_fallback.rs`），少数 1——报告自己给出的这条命令，现在跑一次结果就与报告文字不一致（14 条删除清单本身覆盖了全部 8 个文件，不受这个计数误差影响） |

复核命令（当前主树，因为这三处都不是 `crates/`，不受回滚补丁影响）：
```
$ grep -n "第一版盘不等大时" .claude/kb/decisions/03-空间分配.md
164:第一版盘不等大时，各盘落点答案必须相同；...
$ wc -l .claude/kb/decisions/03-空间分配.md
316 .claude/kb/decisions/03-空间分配.md
$ grep -n "主 agent 按推荐删掉那个计数" records/2026-09-19-里程碑二遗留收拢.md
151:| 5、9 | `policy_mismatches`；抬 F 时现行版本没有实例表单元 | 没列进弹窗 | 5：主 agent 按推荐删掉那个计数（按构造恒 0）...
$ grep -rln "policy_mismatches" crates/ --include="*.rs" | grep -v "^./research" | wc -l
8
```

## 四、本地攻方腿核对表

### 4.1 文件与产物 sha256（现算）

| 文件 | 报告称 | 现算 | 结果 |
|---|---|---|---|
| `m2-newq-r1-local-attack.md`（提示） | `c2a4c656ea76eaa4beb285a420a0df0fada2e6bcca4f65503eaf3a782b2d6f9c` | 同 | ✓ |
| `m2-newq-r1-local-attack-translation-audit.md` | `2ca57123e23cb50be3fbf6b34e5cd54dc75422bcbbb36460e89ff9b4bd9bdb3e` | 同 | ✓ |
| `-output-s1.md` | `990d2a78a27e716f19f942aeec785c631db7b2a0fdcfc13b7c615ebffa2b9fdf` | 同 | ✓ |
| `-output-s2.md` | `3b3679ad53a03fcd440a65f8399317356074bb370e1b7690f9e59c4b9932d481` | 同 | ✓ |

### 4.2 复跑字词损坏检查（草稿目录里重跑 `corruption-check.py`、`oov-check.py`，不在原产物上，用 `nice -n 19`）

| 检查 | s1 报告称 | s1 复跑 | s2 报告称 | s2 复跑 |
|---|---|---|---|---|
| `corruption-check.py` | 绿，全 0 | ✓ 绿，`cjk=0 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0` | 绿，全 0 | ✓ 同上全 0 |
| `oov-check.py` | 生词=1（clarifies） | ✓ 生词=1，`clarifies` | 生词=2 去重（published'、falsified，合计 15 次） | ✓ 生词=2 去重，`published' falsified` |
| `wc -l` | 13 / 101 | ✓ 13 / 101 | | |
| `wc -w` | 282 / 5476 | ✓ 282 / 5476 | | |

字词损坏检查判定全部核对无误（命令原样输出见上，退出码均 0）。

### 4.3 译文核对表逐行核（`-translation-audit.md`）

逐行抽查 kb 原文行号与内容：

| 英文项 | 原文文件:行 | 核的结果 |
|---|---|---|
| Clause A | D23:377 | ✓ 逐字相同（含「与恢复同一套规则」「对着仍打开的设备句柄」等被点名的限定词，原文确有） |
| Clause B | D23:363 | ✓ 逐字相同（含「已定项19①：事务号0不进W的max」「切换的所选根按实例切换那一句取」两处被点名的依据，原文确有） |
| Clause C | D18:308 | ✓ 逐字相同（含规则①的两个具体来源、规则②「没有事务号非0记录时不适用」的例外，原文确有） |
| Clause D | D18:310 | ✓ 逐字相同 |
| Clause E | D23:369 | ✓ 逐字相同（候选集两道门槛、可选性公式、回退行元组值，原文确有） |
| Clause F | D23:386 | ✓ 逐字相同 |
| N5/N6/N7 原题 | `_m2-newq-r1-body.md`:15/16/17 | ✓ 三行逐字相同 |
| 表格轴定义句 | `_m2-newq-r1-body.md`:33 | ✓ 逐字相同 |

补记里「多出来的限定词」两条自陈（root ring/abandoned timeline/F_effective 释义、committed/uncommitted 操作化定义）均标注为「原文没有、自己拼的辅助注」并给出了各自依据行，核对依据行（D23:369、Clause B 的「没有事务号非 0 的记录」句）均存在，不构成摘句问题。

### 4.4 观测（非判决）：s1 样本对提示的完成度

提示 Part 4 要求覆盖全部 12 行（Row 1–Row 12），s2 完整覆盖（13-101 行含全部 12 行 × 6 clause + 3 问答），
但 **s1（13 行）只包含 Row 12 与 3 问答，Row 1–Row 11 完全缺失**。字词损坏检查测的是「有没有乱码/复读」，
测不出「有没有按要求把 12 行都填」，两类问题不是一回事：s1 通过损坏检查不代表它是对提示的完整作答。
运行记录称「第二次调用即达到两份干净样本」，把 s1、s2 都算作有效样本；若把「一条腿只抽一次样不算一次观测」
的抽样计数理解为「有效回答」而非「字词干净」，s1 是否构成一次独立观测，这一点值得主 agent 在采信「没打中」
类结论时留意——这是观测，不是我对够不够格的判决。

## 五、复跑现场记录（从源码独立重建、独立编译、独立运行）

草稿目录：`/tmp/claude-1000/m2-newq-r1-verifier/rerun/`（仓副本 `repo/`、独立 `target/`，未复用腿的原目录）。
镜像放 `/dev/shm/verifier-x`（独立子目录，未写满共享 `/dev/shm`，运行前 `df -h /dev/shm` 现查剩余 30G）。

流程：解包 `baseline-crates.tgz` → `sha256sum -c` 核对 91 个文件与 `baseline-crates.sha256` 一致 →
`python3 patch/apply.py`、`python3 patch/apply2.py` → 编译（14.58s，无警告） →
`TMPDIR=/dev/shm/verifier-x NEWQ_OUT=out/sweep-len4-rerun.tsv NEWQ_MAXLEN=4 NEWQ_WORKERS=4 nice -n 19 $BIN --nocapture`。

这条命令覆盖全部 7 个故障（F1–F7）× 11 或 12 个臂 × 120 条后缀，共约 9200-10600 行，跑得较慢
（debug 构建、单机与另外几个并发 `cargo test`/`cargo clippy` 会话共享 CPU，`nice -n 19` 主动让路）。
**F6、F7 两个故障已经跑完**，其余的产物（含全部 16/19 列）与模型目录里现成的 `sweep-len4.tsv` 同一 11 个臂
逐字节比对，**2640 行、0 处不同**（`diff` 后 exit 0，见第二节 2.4）。**F1–F5 里 F5 接近跑完**（900/1440），
F1–F4 尚未开始；由于这是在我自己的草稿目录独立重跑，产物不删除，主 agent 若需要更完整的复跑覆盖，
可以直接接着等这个进程跑完（PID 211055，日志见 `/tmp/claude-1000/m2-newq-r1-verifier/rerun/repo/out/sweep-len4-rerun.tsv`）。
F1–F3、F9 涉及的具体计数（第二节 2.4 表格）改用**现成 tsv 重新执行 awk/grep 现算**核对（不是抄报告文字），
这与「复跑」的差别是：现算核实了「报告转述现成产物没有算错」，F6/F7 的独立编译重跑额外核实了
「这份产物本身是可以从声称的补丁 + 声称的基线重新造出来的」——两者合起来覆盖了「贴数对不对」与
「贴的这份产物本身站不站得住」两层，本节交回时 F1–F4 的后一层暂缺，属于「核不动」（受挂钟限制，非拒绝去做）。

## 六、计数与没做什么

### 云端攻方（Opus）
- 核了：5 处 kb 整行引用、10 处 crates/ 行引用、116 个模型目录文件 sha256、91 个基线文件 sha256、约 30 处产物数字（第二节 2.4 表格）、1 处「共用前提」断言字符串、1 处「跨重挂现算」缺产物。
- ✓：5（kb）+10（代码行）+116+91（sha256 整表）+约 28 处产物数字 ≈ **全部核到的项里只有 1 处产物数字缺失（F9 重挂现算两格）、1 处标签笔误（落点/落盘）**，其余全部 ✓。
- ✗（严格意义）：0（无一处内容错误，只有「产物缺失」与「标签笔误」两类，已分别单列，不计入 ✗ 也不计入 ✓）。
- 核不动：F1–F4 的从源码独立重跑（背景进程仍在跑，见第五节）；副本上标「推的」「没跑」的格（落盘臂真实写盘代价、准入式子假性 ENOSPC）按报告自己的说法就是没跑，不重复核。

### 云端正推（Sonnet）
- 核了：N1-N3 段 12 处 kb/checks-owed 引用、N4 段 14 处删除清单引用 + 3 处旁证引用 + 1 处自证命令、N5-N7 段共用 D23:386/369/377 与 D18 范围等 6 处。
- ✓：N1-N3 全部 12 处 ✓；N4 的 14 条删除清单按倒推树核 12 条精确 ✓、1 条区间边界误差（不计入 ✗，单列「分不清」）、1 条不受影响天然 ✓；N5-N7 共 6 处里 5 处 ✓。
- ✗：**2 处**——D23:515（应为 377 附近，指向完全无关的另一小节）；D3（空间分配）.md:768（该文件仅 316 行，768 不存在，真实行号 164）。另有 1 处数字性错误（自证命令称命中 7 个文件，现查命中 8 个）单列，不计入行号 ✗ 但同样是可核穿的错误。
- 分不清：`first_transaction_step_five_publish.rs:1018-1027` 区间两端与真实断言块（1021/1022-1028）有偏移，指向的是同一处断言、非无关代码，判「分不清：区间划定不精确」。
- 核不动：无（这条腿全部引用都是可静态核的文本/行号，没有需要编译或跑虚机的部分）。

### 本地攻方
- 核了：4 个文件 sha256、2 次字词损坏复跑、8 处译文核对表行号。
- ✓：全部 14 处。
- ✗：0。
- 观测（非 ✓/✗）：s1 样本对提示任务本身的完成度（只答 12 行里的 1 行），已在第 4.4 节单列。

## 七、没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑，正文里所有「✓/✗」只针对「引用是否与源文件/产物一致」这一件事。
- 没有对 F1–F4 做完整的从源码独立重跑（背景进程仍在跑，详见第五节「核不动」）；这些故障的数字核对停留在「现成产物重新算一遍」这一层，没有验证产物本身能否从声称的补丁与基线重新造出来。
- 没有核 opus 报告第六节「没打中的形状」、第八节「限度」、第九节「没做什么」里的叙述性文字（非引用、非可核产物的部分）。
- 没有核 sonnet 报告 N5/N6/N7 判定的推理链本身（是否真的推不出），只核了它引用的原文行号与内容。
- 没有核本地攻方 s1/s2 的答案内容对不对（题目要求的是「条款是否settle了某一格」，这是推理判断，不归核查员做）。
- 没有跑 `.claude/gate.d/` 任何阶段（`stage-owners.tsv` 未登记给 `three-way-verifier`，未查）。
- 没有编译 `crates/` 主树（只在自己的草稿目录副本上编译腿的模型代码）。
- 未读禁读清单（派发提示未给禁读清单，未额外排除任何文件）。

## 八、追记：后台复跑已结束（完成通知到达之后追加，主 agent 已收到第一份交回；本节是后续更新，不再触发第二次交回）

第五节写交回时后台重跑还在跑（F5 到 994/1440）。进程后来自行退出（`echo "exit=$?"` 捕到的是
`... | tail -30` 管道最后一段的退出码，不是测试二进制真正的退出码——这是
`command-safety.md`「管道里的退出码不是你想要的那个」那条本身的一个实例，如实记录，不据此判断二进制成没成功）。
最终产物 `out/sweep-len4-rerun.tsv` 共 8431 行（含表头），逐故障行数：

```
F2-latent-eio-both 1230（应 1320，缺 90 行）
F3-latent-eio-dev0 1440
F4-rot-both        1440
F5-rot-dev0        1440
F6-map-bug-live-instance-table 1440
F7-map-bug-leaf-same-publish   1440
（F1-transient-eio-both 完全缺失，0 行）
```

对已完整跑出的 **F3、F4、F5**（连同交回时已核完的 F6、F7），逐一按第二节 2.4 的同一方法核对
（去掉原始产物没有的 `N2-same-verdict-rebuild-tolerates-data` 臂、排序后逐字节 diff）：

```
$ diff orig-F3-latent-eio-dev0.txt mine-F3-latent-eio-dev0.txt   # 各 1320 行
（无输出，exit 0）
$ diff orig-F4-rot-both.txt mine-F4-rot-both.txt                 # 各 1320 行
（无输出，exit 0）
$ diff orig-F5-rot-dev0.txt mine-F5-rot-dev0.txt                 # 各 1320 行
（无输出，exit 0）
```

**F3、F4、F5 三个故障、11 个共同臂、120 条后缀、全部列，同样 0 处不同。** 加上此前的 F6、F7，
**7 个故障里已有 5 个（F3、F4、F5、F6、F7）从源码独立编译、独立运行、全量比对，与模型目录产物逐字节相同**。

对已产出但不完整的 **F2**（1110/1320 行，`N1A-any-isoAll-mem` 臂只有 30/120 条后缀）：
`comm -23` 核对「我这边有、原始产物里没有」的行数 = **0**——已经跑出来的每一行都能在原始产物里逐字找到，
不是数据错误，只是没跑完。**F1 完全没有产物**，无法核对，判「核不动：这次重跑没能在 F1 上产出任何数据」。

修订后的计数（覆盖第六节「云端攻方（Opus）」小节里「核不动」一条）：F1 上的具体计数（甲/乙/丙/丁在 F1 的
58/120、0/120 等数字）**仍然只核对了「现成产物重新算一遍」这一层**（第二节 2.4 表格），
没有拿到「这份产物本身能不能从声称的补丁与基线重新造出来」这一层的独立证据；F2/F3/F9 涉及的数字里，
F3 现在已经补上了后一层证据（因为 F3 整个故障已完整重跑并逐字节比对），F2、F9 仍停留在前一层
（F9 的产物来自 `out3/`，用的是 `patch3/apply3.py`，这次重跑没有覆盖那一层补丁，未尝试）。

已清理 `/dev/shm/verifier-x`（进程结束后残留约 6.1G 稀疏镜像文件，已删除，`/dev/shm` 恢复到 198M 占用）。
