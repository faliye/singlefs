# m3-batch52-54-code-r1 云端正推腿报告（Sonnet）

轮名 m3-batch52-54-code-r1（代码轮第一轮，删除线第 52–54 批）。立场：正文第四节「云端正推」一行——核「代码做的是不是条款说的」，对得上 / 对不上，不替主 agent 判决。
格：正文第三节 K1–K6 全部。判定词只有三个：一致 / 冲突 / 规则没说；「推翻条件」写在每条结论后面。日期 2026-10-10。
记号：下文 `F` = 冻结副本根 `/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo`；代码引文一律写这个绝对路径加那份文件自己的行号（项目根下的 `crates/` 是未打补丁的主工作区，行号对不上）；kb 条款写 kb 文件自己的行号，行号都用 `grep -nF` 现查过。

## 零、做法与环境

- 开工前 `ps -o pid,args -u "$(id -u)"` 查性能测量：`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio` 一个都没有（`ps ... | grep -E 'qemu-system|vm-bench|e152-file|fio ' | wc -l` 输出 `0`）；中途 `host-resources.py` 报本机有别的会话的 `cargo test --release`，我的命令一律 `nice -n 19`、经 `run-with-memory-cap.sh 10G`、`capped.sh 12`，同一时刻只起一条 cargo，没等锁。
- 验证用的拷贝：`~/.local/share/singlefs/work/m3-batch52-54-code-r1-sonnet/repo/`（只拷 crates、litmus、.cargo、Cargo.toml、Cargo.lock），编译目录 `.../m3-batch52-54-code-r1-sonnet/target`。冻结副本一个字没改：`cd F && sha256sum -c research/prompts/m3-batch52-54-code-r1-snapshot/crates-sha256.txt`（清单 25 行，含 `crates/mutations.tsv`）的结果是 `frozen OK 25 FAILED 0`。拷贝里我动过的只有两份（`walk.rs` 末尾加了一个探针单测、独立参照用例加了事件计数），`sha256sum -c` 在拷贝上正好这两份 FAILED、其余 23 份 OK。
- 模型目录 `research/prompts/m3-batch52-54-code-r1-sonnet-model/`：变异与探针的脚本、补丁、各次运行的原样日志，`README.md` 写了怎么复跑，`SHA256SUMS` 是其中每份文件的哈希。
- 只跑 harness 档与核心 / 检查器两个包的库测试（`singlefs-harness`、`singlefs-core --lib`、`singlefs-checker --lib`）。没跑 `singlefs-checker-tier` 的任何测试、没跑门禁阶段。需要 checker 档才能判的格在末尾「没做什么」逐条写。

## 一、K1　恢复里两判的次序（第 52 批）

### K1.1　先核条目宽之后，别的坏法交回的错误成员变了没有；合法镜像被拒没有

条款：`/home/fy5090/code/singlefs/.claude/kb/invariants.md:35`「判定顺序与 I-1.7（打包容器合法与判定顺序） 同一纪律：**先于**按字段表解条目」（I-1.10，码 2 条目宽等于字段表宽，记账条目 34）。

代码（行号都在 F）：
- 新判：「fn every_accounting_entry_is_as_wide_as_its_field_table(」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-core/src/recovery.rs:4771`）逐条调「fn accounting_entry_parsed_at_its_field_table_width(」（同文件 `:4857`），那一条判的是「AccountingEntry::parse(entry_bytes).ok_or(RecoveryFailure::EntryNarrowerThanItsFieldTable {」（同文件 `:4860`）；解析只拒窄于 34：「if bytes.len() < usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34") {」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-core/src/records.rs:329`）。
- 数条目那一判不变，只改了注释：「/// 解不开的条目算进 3 + 6 × 盘数；调用方先过 [`every_accounting_entry_is_as_wide_as_its_field_table`]，走到这里的每条都解得开。」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-core/src/recovery.rs:4782`）。
- 次序（重建 = `rebuild_version_in_the_unit_area_starting_at`，`recovery.rs:2519`）：读完记账树 → 条目宽 `recovery.rs:2916` → 条目数 `:2919` → 逐条解并判代与 seq `:2927`。
- 次序（冷走读 = `walk_to_file_in_the_unit_area_starting_at`，`recovery.rs:5167`）：条目宽 `recovery.rs:5390` → 条目数 `:5393` → 映射条目数 `:5398` → 分配记录代与跨度 `:5409` → 逐条解并判代与 seq `:5411`。

判定：**一致**。推出来的三件事：
1. 被拒的镜像集合不变。改前每一条记账条目都要过逐条解那一判（`recovery.rs:2927`、`:5411`，里面调的就是同一个「accounting_entry_parsed_at_its_field_table_width」，`:4872`），窄条目本来就拒；新判是同一个条件提前到数条目之前，不新增拒的镜像，也不放过拒的镜像。
2. 合法镜像不受影响：合法条目都是 34 字节，新判恒过。只剩台阶行、空发布、回退那次的镜像没有窄条目，两个读者的流程与改前逐步相同。实测：S 集（`checker_known_bad_images` 230、`container_trees_grow_a_level_when_the_root_is_full` 6、`writable_mount_of_a_formatted_pool` 16、`reuse_after_raising_the_floor` 15、`pool_checker_judges_the_deletion_state_of_every_walked_version` 45，均 `--include-ignored`）在冻结副本拷贝上全过，见 `…-sonnet-model/logs/S-pristine.log`。
3. 交回的错误成员只在「有窄条目、同时还有另一处坏」的镜像上变。改前，窄条目要等到逐条解才报，在那之前会先报别的：重建读者里是条目数（`:2919`）；冷走读里是条目数、映射条目数（`:5398`）、分配记录代与跨度（`:5409`），逐条判里还有下标更小的那条「代晚于根或 seq 为 0」。这些组合改前报别的，改后一律先报条目宽。条款方向对：I-1.10 要求宽「先于按字段表解条目」，而数条目要解条目才认得出台阶行（统计量标签与代段在条目里）。我没有造这些组合镜像逐个跑，上面是读码推的。

命令：`cd F && grep -nF 'every_accounting_entry_is_as_wide_as_its_field_table(' crates/singlefs-core/src/recovery.rs` 输出 `2916:`、`4771:fn …`、`5390:` 三个调用 / 定义点（`2916`、`5390` 是调用，`4771` 是定义）。

推翻条件：造出一张镜像，记账条目宽 ≥ 34、两个读者之一却因新判改了结局（合法镜像被拒、或被拒的镜像放行）；或一张「窄条目 + 另一处坏」的镜像，改后报的不是条目宽。

补充（没被这批改，只记一笔）：I-1.10 写的是条目宽「等于」字段表宽，恢复这一侧只拒窄于：记账条目 `records.rs:329` 与码 2 内部节点「if header.entry_width < internal_width {」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-core/src/code_two_tree.rs:1005`）都是 `<`。宽于 34 的记账条目恢复会放行、池级 checker 判红。我没造宽条目镜像验证，只是读到的两处。

### K1.2　重建与冷走读次序一致吗；可写挂载那条路呢

判定：**一致**。两个读者的前三步相同：宽 → 数 → 逐条（上面两串次序）；中间隔的别的判不同是原来就不同，这批没动。可写挂载没有第三份实现：它走 `rebuild_version_in_the_unit_area_starting_at`（「let rebuilt = rebuild_version_in_the_unit_area_starting_at(」，`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-core/src/mount.rs:1236`），失败原样包成「RebuildVersionFailure::Walk(recovery_failure) => MountError::Recovery(recovery_failure),」（同文件 `:1078`）。全 crates 里记账条目只在 `recovery.rs`、`admission.rs`、`transaction.rs` 用，只有 `recovery.rs` 解盘上的条目（`grep -rn 'AccountingEntry' crates/singlefs-core/src crates/singlefs-checker/src` 现查）。

新加的两条用例各钉一个读者：恢复一条「fn a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_recovery() {」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:1543`），可写挂载一条（同文件下一条）。我在拷贝上把两处调用次序各换回去（变异表第 3711、3712 行），原样输出：

```
第52批 冷走读记账条目数排回条目宽之前（台阶行解不开被算进 3 + 6 × 盘数，条目宽坏法报成条目数）
  expected: a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_recovery
  rc 101
   test a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_recovery ... FAILED
第52批 重建记账条目数排回条目宽之前（台阶行解不开被算进 3 + 6 × 盘数，条目宽坏法报成条目数）
  expected: a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_writable_mount
  rc 101
   test a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_writable_mount ... FAILED
```

（每次只跑点名的那一条，另 32 条被筛掉没跑；完整输出 `…-sonnet-model/logs/rows-b52-53.log`。）把调用整个删掉与换次序在这组镜像上等价（数条目会报 16 ≠ 15），不另列。

推翻条件：把两处调用次序换回去、两条用例有一条照绿；或可写挂载在窄条目镜像上交回的不是 `MountError::Recovery(EntryNarrowerThanItsFieldTable …)`。

## 二、K2　I-9.36、I-9.37（第 53 批）

条款现状：I-9.36、I-9.37 还没写回 kb（`/home/fy5090/code/singlefs/.claude/kb/invariants.md:12`「判 90 条」，F 里 `IMPLEMENTED_INVARIANTS` 已是 92：「pub const IMPLEMENTED_INVARIANTS: [&str; 92] = [」，`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker/src/image.rs:111`）。能对的条款是它们照的两条：`/home/fy5090/code/singlefs/.claude/kb/invariants.md:309`（I-9.21）「容器号沿叶序严格递增」…「除最右那一个之外的每个打包记录类型 1 容器记录数 == 815（开放容器满了才关」…「判回退候选集里的每一版」，`:310`（I-9.22）「每个打包记录类型 1 容器记录数 ≤ 旋钮值」…「旋钮值由池级 checker 的新入口给，旧入口当没装旋钮」；以及容器怎么装的条款：
- 墓碑：`/home/fy5090/code/singlefs/.claude/kb/decisions/18-块里携带什么信息.md:578`「3. **开放容器**：有新记录的那次发布才重写，不删除的发布照抄；满了才关；关闭路径加只供测试的容量旋钮，旋钮不落盘。」
- 退役：`/home/fy5090/code/singlefs/.claude/kb/layout/03-third-txn.md:19`「有新记录的那次发布才重写开放容器，没有的发布照抄；满了才关；第一版不回收」。
- 谱系：`/home/fy5090/code/singlefs/.claude/kb/layout/03-third-txn.md:18`「容器号从 1 起，下一个号 = 谱系树里最大号 + 1，零根时 1，0 不用（规则同墓碑容器）」——「满了才关」没写在谱系这一行，括注「规则同墓碑容器」跟在容器号规则后面；D18 已定项 24 第 5 条（`/home/fy5090/code/singlefs/.claude/kb/decisions/18-块里携带什么信息.md:580`）写「管理员回退那次发布追加一条」「一个 32 KiB 容器装 ⌊(32768 − 136) ÷ 24⌋ = 1359 条」，也没明写。**谱系树「非最右装满」是类比墓碑与退役得来的，条款没逐字写：规则没说。**

写侧实现（判的对象）：追加记录时「let open_container_has_room = containers」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-core/src/container_tree.rs:535`）接「.is_some_and(|open| open.records.len() < records_per_container);」（`:537`）——最右那片没满就往里追加，满了才开新片，新号「.map_or(1, |rightmost| rightmost.identity.container + 1);」（`:548`）。谱系、退役、墓碑三棵树的追加都调这一个函数，所以任何一版自己的这三棵树里，非最右的容器恒是满的、号恒是连号从 1 起。

### K2.1　合法镜像上误不误报

判法（CPU，F 的 `walk.rs`）：「fn judge_lineage_or_retirement_container_capacity_of_a_version(」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker/src/walk.rs:11975`）；没装旋钮「None => 2,」（`:11986`）至少两片才判，装了旋钮至少一片；非最右「if is_the_rightmost {」（`:12008`）跳过，其余「judgements.judge(invariant, record_count == product_records, || {」（`:12012`）；装了旋钮「judgements.judge(invariant, record_count <= records_the_knob_allows, || {」（`:12019`）。

| 形 | 判定 | 依据 |
|---|---|---|
| 最右容器没满 | 一致，不报 | `:12008` 最右那片 `continue`；零根 / 一片没有对象，探针单测直接调这个函数：单容器 `[(1,5)]` 无旋钮 → `NotApplicable("这个镜像上没有它判的对象")`（`…-sonnet-model/logs/probe1.log`），经 `into_report` 变成树级默认不适用理由 |
| 根长高之后 | 一致，不报 | 容器号取容器头不取根的条目：「fn containers_of_a_container_tree_without_judging(」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker/src/walk.rs:10380`）按条目序递归、类型段 0 的下一层，叶序出来；`container_trees_grow_a_level_when_the_root_is_full` 6 条（含谱系树回退三次长到第二层）在拷贝上全过 |
| 回退照抄 cur | 一致，不报 | 回退不改退役树 / 谱系树的装法，追加仍走 `container_tree.rs:535` 那一条路；`unlinking_rolling_back_and_unlinking_again_writes_a_second_record_of_the_same_leaf_and_no_invariant_reddens` 在 `checker_known_bad_images`（230 过）里 |
| 崩溃恢复抛弃段里写的容器 | 一致（推的，没造镜像） | 判的是每一版自己那棵树，不跨版；被抛弃的根即使留在候选集里（滤不掉时），它自己的三棵树也是按 `container_tree.rs:535` 追加出来的，形状同上；要出现「非最右没满」只有写侧另有一条不满就开新片的路；`grep -n 'append_record_to_the_open_container\|append_records_to_the_open_container' lineage_tree.rs tombstone_tree.rs retirement_tree.rs`（在 F 的 crates/singlefs-core/src 下）三份各一处调用：lineage_tree.rs:109、tombstone_tree.rs:126、retirement_tree.rs:132，开新容器的只有 container_tree.rs:549 一处 |
| 测试旋钮压小的镜像 | 一致，设计如此 | 新入口按树给旋钮：「pub fn check_pool_image_given_the_container_capacity_knobs(」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker/src/walk.rs:12135`）；旧入口 `check_pool_image` = 「check_pool_image_given_the_container_capacity_knobs(reader, ContainerCapacityKnobs::PRODUCT)」（`:12109`）当三棵都没装旋钮，所以旋钮压小的谱系 / 退役镜像走旧入口会红 I-9.36——两条 GPU 对拍用例正是靠这个造红；旧的墓碑旋钮入口把谱系、退役给 `None`（「lineage: None,」，`:12124`） |
| 旋钮只给了一棵树 | 一致 | 每棵树按自己的旋钮选 I-9.36 或 I-9.37（`:11394`、`:11616`）；`container_trees_grow…`（墓碑 + 谱系装旋钮、退役没装）6 过，`checker_known_bad_images` 的退役旋钮用例（只给退役）过 |
| 谱系与退役混在一起 | 一致 | 两棵树各调一次判法，互不串；两棵都没装时「读不全」那条射程说明落在 I-9.36 上两次（`part_of_the_range_not_judged` 以第一次为准，不重复）|

推翻条件：一张合法镜像（不装旋钮、旧入口）上 I-9.36 红，或带旋钮值的入口给对了旋钮值却红 I-9.37。我没造出来。

### K2.2　坏镜像漏不漏

| 坏法 | 判定 | 依据 |
|---|---|---|
| 提早关（非最右没满） | 一致，拦住 | 变异行 `crates/mutations.tsv` 第 3722 行「第53批 F2 I-9.36：非最右的退役容器」等把 `==` 改成 `<=`，在拷贝上两条都红（`…-sonnet-model/logs/rows-b52-53.log`：退役树那一形、谱系树那一形各一条 FAILED；每次只跑点名的那一条，229 条被筛掉没跑） |
| 中间一片多出一条 | 旋钮下拦住；产品下 I-9.36 自己不单独钉 | 旋钮：把 `<=` 改成 `<= … + 1` 的变异行在拷贝上红（`rows-b52-53.log` 最后一行）。产品容量：`== product` 抓得到 960 对 959，但没有「产品容量下多一条」的坏镜像；我把 `==` 改成 `>=`（提早关仍红、多一条不红，变异 w11）五个 harness 二进制全过（见 K6），I-1.7 的「记录数 × 记录宽 ≤ 声明长度」（`/home/fy5090/code/singlefs/.claude/kb/invariants.md:32`）会同时红这类镜像，所以整体不漏，I-9.36 自己的这一半没单钉 |
| 容器号不连 / 不从 1 起 | **规则没说，实现与 I-9.36 的写法一致，漏** | 判的是严格递增（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker/src/walk.rs:11994`「judgements.judge(invariant, left_container < right_container, || {」）。探针单测直接调判法：谱系 `[(1,1359),(3,5)]` → `Holds`，退役 `[(5,959),(6,5)]`（首号 5）→ `Holds`（`logs/probe1.log` 前两行 `PROBE`）。条款 D18 / layout 写的是「容器号从 1 起，下一个号 = 谱系树里最大号 + 1」（`/home/fy5090/code/singlefs/.claude/kb/layout/03-third-txn.md:18`），连号与从 1 起没有一条不变量判——I-9.21 同形缺这一半，这批照抄 |
| 容器号相等 / 倒序 | 一致，判得到 | 同一探针：`[(2,959),(2,5)]` → `Violated("…容器号不严格递增：容器 2 之后是容器 2")`，`[(3,959),(2,5)]` → `Violated`。**但没有任何测试钉这一句**（K6） |
| 最右那片超过产品容量 | 不归 I-9.36 | 探针 `[(1,1359),(2,1400)]` → `Holds`；由 I-1.7 与 I-9.32（退役「record_count > 959」）管 |

推翻条件：造出一张「中间一片多出一条」的旋钮镜像，带对旋钮值的入口却不红 I-9.37。（探针只调了判法函数，没有整镜像上造「容器号不连」的坏镜像；整镜像上的结局由 `into_report` 把 `Holds` 报成成立，推的。）

### K2.3　CPU 与 GPU 判得一样吗（树表 12 条、容量上限、旋钮怎么传给 GPU）

判定：**读码一致；动态对拍在 checker 档，我复核不了**。

- 同形：GPU「fn d_judge_lineage_or_retirement_container_capacity(product_records: u32) {」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3162`）：至少两片、沿叶序号严格递增（`:3167`「if (!lt64(d_container_number(container_index - 1u), d_container_number(container_index))) {」）、非最右每片 `!= product_records`（`:3172`）；谱系树在读完容器后调一次（`:3031`「d_judge_lineage_or_retirement_container_capacity(DELETION_LINEAGE_CONTAINER_RECORDS);」，在逐条记录的循环和 `D_MAX_LINEAGE_RECORDS` 之前，与 CPU 先判容量后判链同序），退役树调一次（`:3835`）。
- 常量：「const DELETION_LINEAGE_CONTAINER_RECORDS: u32 = 1359u;」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2173`）、「const DELETION_RETIREMENT_CONTAINER_RECORDS: u32 = 959u;」（`:3355`）等于格式常量「pub const LINEAGE_CONTAINER_RECORDS: u64 = 1359;」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-format/src/lib.rs:122`）与「pub const RETIREMENT_CONTAINER_RECORDS: u64 = 959;」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-format/src/lib.rs:130`）。位号「const INVARIANT_I_9_36: u32 = 90u;」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:204`）、「const INVARIANT_I_9_37: u32 = 91u;」（`:206`）是 92 条清单的最后两位，覆盖表也补了 90、91（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker-tier/src/crash_judge_gpu.rs:99`）。
- 树表 12 条：两边都是遍历树表全部条目、按种类码挑谱系 / 退役 / 墓碑，没有写死条数；GPU 的树表上限只有 `C_APPEARANCE_MAX_WALKED_TREE_TABLES`（别的判法），这两条不碰。
- 容量上限：GPU 一棵树最多读 256 片容器：「const D_MAX_CONTAINERS_OF_A_CONTAINER_TREE: u32 = 256u;」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:845`），多于它「if (d_container_count >= D_MAX_CONTAINERS_OF_A_CONTAINER_TREE) {」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2780`）置判不了、交 CPU；CPU 没上限。与 I-9.21 同一个安排，不新增分歧。
- 旋钮怎么传给 GPU：**不传**。GPU 只有没装旋钮那一支，I-9.37 恒不置位——覆盖表文档写明「下标 91 的 I-9.37 照下标 79 的先例恒不置位」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker-tier/src/crash_judge_gpu.rs:93` 起的那段，`:92` 是 90、91 的说明）。所以 GPU ≡ CPU 的**产品入口**，不等于 CPU 的新入口带旋钮；两条 GPU 对拍用例拿旋钮写的镜像 + CPU 旧入口对拍。后果：凡是经 GPU 判的流都不许带谱系 / 退役旋钮而期望绿；现有 checker 档流里带这两个旋钮的只有这两条新用例（`grep -rln 'LineageRecordsPerContainerCappedForTests\|RetirementRecordsPerContainerCappedForTests' crates` 现查：checker-tier 只有 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`）。

kb 一侧的冲突两处（不影响代码，写回 kb 时会撞）：
1. `/home/fy5090/code/singlefs/.claude/kb/invariants.md:32`（I-1.7）「类型 5（谱系）是 16」，D18 已定项 24 第 5 条（`decisions/18-块里携带什么信息.md:580`）「记录 = T_rb 8 + T_old 8 + H_cur 8 = 24 字节」，代码「PACKED_TYPE_LINEAGE => Some(24),」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker/src/walk.rs:210`）：**冲突**，I-1.7 那一行是旧值；该行也没列类型 3（退役，34，`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker/src/walk.rs:212`）。
2. `/home/fy5090/code/singlefs/.claude/kb/invariants.md:12`「判 90 条」与代码 92：写回时要改。

推翻条件：一张不带旋钮的镜像上 GPU 与 CPU（旧入口）对 I-9.36 位不同，或 GPU 覆盖表 / 位号与 `IMPLEMENTED_INVARIANTS` 下标错位（`IMPLEMENTED_INVARIANTS` 末两项 `image.rs:202` 起即 I-9.36、I-9.37）。

## 三、K3　Opt2 环那一项（第 53 批）

条款：`/home/fy5090/code/singlefs/.claude/kb/decisions/23-journal的角色与格式.md:385`（D23 已定项 14「水位」一格）「inode 号水位 = max(cur 那一版内存里的水位, 环里读得出、按实例表没被崩溃恢复抛弃的根的水位)」…「max 里 cur 那一版内存里的那一项不能去：只取环，回退那次就重发号」…「树 ID 水位照旧 = max(cur 那一版内存里的水位, 环里读得出的全部根的水位)」。

### K3.1　抽出前后 `roll_back_by_a_forward_publish` 的行为逐字节相同吗

判定：**一致**（读码；core 库 244 条含三条新单测在拷贝上全过；没有逐字节对拍两棵树的产出，那一步靠「没有副作用的纯重排」推）。

代码（F 的 `mount.rs`）：
- 新函数：「fn inode_number_watermark_of_the_rollback(」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-core/src/mount.rs:7845`），体内先滤「fn ring_roots_not_abandoned_by_crash_recovery(」（`:7829`，改前同一个函数同一个实参），再「ring.max(inode_number_watermark_of_the_current_version)」（`:7856`）。
- 调用点：「let inode_number_watermark = inode_number_watermark_of_the_rollback(」（`:8137`），闭包里调的还是「fn highest_inode_number_watermark_in_the_ring<Device: BlockDevice>(」（`:7802`）；树 ID 水位那一段留在调用点前面没动：「let tree_identifier_watermark = ring_roots」（`:8131`）接「.fold(current.root.tree_identifier_watermark, u64::max);」（`:8134`）。
- 与改前的差别只有求值次序：改前 `current.inode_number_watermark()` 在盘上重建（读盘）之后才求（`map_or` 的缺省实参与闭包里各一次），改后先求一次再传进去。这个读函数是纯的：「pub fn inode_number_watermark(&self) -> u64 {」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-core/src/transaction.rs:3055`）只扫内存里的 `accounting_entries`，缺行时 `expect` panic；盘上重建不写盘。panic 与读盘的先后在「cur 没有水位行」这种发布路径保证不会出现的状态里才有区别（改前读完盘再 panic，改后直接 panic），读盘不留痕，所以输出不变。

命令（拷贝上，未变异；`cmp` 过：拷贝里 inode_tree.rs、mount.rs、recovery.rs 与 F 逐字节相同）：`cargo test --offline -p singlefs-core --lib`（经 `run-with-memory-cap.sh 10G`、`capped.sh 12`）原样输出（同样存在 `…-sonnet-model/logs/core-lib-pristine.log`）：

```
test mount::inode_number_watermark_of_the_rollback_tests::a_ring_root_abandoned_by_the_instance_table_does_not_set_the_watermark ... ok
test mount::inode_number_watermark_of_the_rollback_tests::a_ring_root_not_abandoned_whose_watermark_is_above_the_current_version_sets_the_watermark ... ok
test mount::inode_number_watermark_of_the_rollback_tests::a_ring_root_whose_watermark_is_below_the_current_version_leaves_the_current_watermark ... ok
test result: ok. 244 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.84s
```

回退用例在 harness 档的未变异基线：S 集里的 `reuse_after_raising_the_floor`（15）等全过（`logs/S-pristine.log`）；`rollback_by_a_forward_publish --include-ignored` 25 条的全过是在 K3.2 的 mt1 变异之下跑的，那一趟之下过就意味着这些用例不依赖被改的那一句，**不是**未变异基线，未变异基线我没单跑它。

推翻条件：一段历史上，抽出前后回退那次发布的 `inode_number_watermark` 不同；或发布路径在 `highest_inode_number_watermark_in_the_ring` 之前就改了 `current` 的记账。

### K3.2　三条构造态单测钉的是不是条款说的那件事

判定：**一致于条款前三件，树 ID 水位那一件不在里面（规则说了，三条单测没钉）**。

- 三条用例（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-core/src/mount.rs:9384`「fn a_ring_root_not_abandoned_whose_watermark_is_above_the_current_version_sets_the_watermark() {」、`:9399`「fn a_ring_root_abandoned_by_the_instance_table_does_not_set_the_watermark() {」、`:9414`「fn a_ring_root_whose_watermark_is_below_the_current_version_leaves_the_current_watermark() {」，都在 `/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-core/src/mount.rs`）依次钉：没被抛弃的根算（环 90 > cur 40 → 90）、被抛弃的不算（同一条根、实例表只选到 5 → 40）、cur 那一项不能去（环 25 < cur 40 → 40）。三件正是条款 385 行 inode 号水位前半。我在拷贝上把三条各自的变异行改上去（变异表第 3716–3718 行），每条红了自己点名的那一条（每次只跑那一条，`cargo test` 输出）：

```
第53批 F3 Opt2：回退取 inode 号水位时环那一项一条根都不交（攻方 M5，只剩 cur 内存里的）
  expected: a_ring_root_not_abandoned_whose_watermark_is_above_the_current_version_sets_the_watermark
   test mount::inode_number_watermark_of_the_rollback_tests::a_ring_root_not_abandoned_whose_watermark_is_above_the_current_version_sets_the_watermark ... FAILED
第53批 F3 Opt2：回退取 inode 号水位时被崩溃恢复抛弃的根也算（构造态）
   test mount::inode_number_watermark_of_the_rollback_tests::a_ring_root_abandoned_by_the_instance_table_does_not_set_the_watermark ... FAILED
第53批 F3 Opt2：回退取 inode 号水位只取环、不与 cur 内存里的取 max（构造态）
   test mount::inode_number_watermark_of_the_rollback_tests::a_ring_root_whose_watermark_is_below_the_current_version_leaves_the_current_watermark ... FAILED
```

- **树 ID 水位「照旧取环里全部根」没有任何测试钉**：抽出的函数不带它（它仍在调用点 `:8131`–`:8134`），三条构造态单测也没碰。我在拷贝上把它改成只取没被抛弃的根（把「let tree_identifier_watermark = ring_roots」改成「let tree_identifier_watermark = ring_roots_not_abandoned_by_crash_recovery(&ring_roots, &current_table)」起头，变异 mt1），跑回退相关的 12 个 harness 二进制，**全过**：`rollback_by_a_forward_publish --include-ignored`（25 条，`logs/mt1-ignored.log`）；`raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table`（2）、`rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate`（20）、`presumed_clause_checks`（3）、`core_review_unit_area_start_and_publish_limits`（67）、`unlink_idempotence_criteria_and_rollback_to_the_emptied_version`（1）、`an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor`（7）、`system_configuration_rollback_floor_and_layout_identity`（5），带 `--include-ignored`（`logs/mt1-more.log`）；另外 `unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over`（24）、`rollback_lineage_record_and_its_checkpoint_cost_term`（5）、`container_trees_grow_a_level_when_the_root_is_full`（6）、`pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root`（8）四个只有终端输出、没存日志。变异表里只有「C586：回退那次的树 ID 水位只取环里读得出的根，不与内存里现行那一版的取 max」（第 1504 行）管 cur 那一项，没有一行管「被抛弃的也算」。
- 抛弃判定本身（`abandoned_by_table`，「fn abandoned_by_table(root: &RootRecord, table: &InstanceTableRecords) -> bool {」，`mount.rs:1433`）转给 `recovery::root_is_abandoned_by_the_instance_table`，边界（选到的 txg 等于根的 txg、实例在表里没有行）这三条没钉：它们的实例表里选到 9 或 5，从不取 7。这一层不是这批改的。

推翻条件：有一个 harness 用例在树 ID 水位只取没被抛弃的根时变红（我跑了 12 个相关二进制没找到；全部 181 个 harness 二进制没跑）；或条款把「树 ID 水位照旧取全部」改写成取没被抛弃的。

## 四、K4　M3 的独立参照（第 53 批）

### K4.1　参照用例真不调 `inode_tree.rs` 的合并、借、摘片吗；与实现共用了哪些前提

判定：**不调：一致。共用前提：有三样，另外还有两样没比（下面逐项）。**

不调。用例文件头「参照按条款另写一份，不引 singlefs_core::inode_tree 的合并、借、摘片代码」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs:2`）；`grep -n 'inode_tree' <该文件>` 现查只有文件头那行与结构体字段名 `inode_tree: u64`（第 2、57、76、82、98 行），`use` 只有 `common`、`common_deletion_in_progress`、`TransactionOutput`、`INODE_LEAF_RECORDS`、`SeededRandomSource`（`:16`–`:20`）。被测的是发布路径（`unlink_to_the_end` 经核心的 unlink 调 `inode_tree::remove_record_from_leaf_containers`），参照是文件里自己的 `ReferenceLeafShape`（`:55` 起的结构体与 `remove`/`merge_from`/`borrow_from_the_right_if_below_the_floor`）。

共用的前提：
1. **规则来源同一份（E176 M3）**。D8 已定项 6 把合并时机放在写路径策略里：`/home/fy5090/code/singlefs/.claude/kb/decisions/08-核心索引结构.md:171`「合并时机是写路径策略（单测 / 模型对拍），不进格式」——「先查右邻」「下限 ⌈233÷2⌉」「借 ⌊(右−左)÷2⌋」「删空摘掉后从新左邻起并」「借后从借出的那片起并」这几条**条款没写**（条款只写了「左吸收右与左从右借两样都做」「借的条数取均分」，同一行 `:171`；用户定案「M3 连借一起做」在 `:228`）。参照自己写明规则取自 E176 的 M3 臂：「合并时机照 E176」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs:4`）；实现的文档也点名同一处：「从 `start_position` 那片起先查它与右邻、再查左邻与它，两片之和 ≤ 233 就左吸收右」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-core/src/inode_tree.rs:453`）、「左从右借的下限：一片叶删完一条之后记录数少于它、这次又没并过、它有右邻，就从右邻借」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-core/src/inode_tree.rs:192`）。我把参照的 `merge_from`（`:132`）、借（`:160` 的「let moved_count = (self.records_at(position + 1) - self.records_at(position)) / 2;」）与 E176 `remove`（`/home/fy5090/code/singlefs/research/e7-index-bench/src/bin/e176_inode_leaf_merge_capacity_and_rewrites.rs:278`）逐句对过：同一套规则的第二份转写，不是对条款的独立推导。所以这个参照能证「实现 = E176 M3」，证不了「E176 M3 = 用户要的」；E176 M3 在这三处错了，两边一起错。
2. **容量常量同源**：参照 `use singlefs_format::INODE_LEAF_RECORDS;`（`:19`）与实现用的是同一个常量（`singlefs-format/src/lib.rs:110`，=233，kb 一侧 `/home/fy5090/code/singlefs/.claude/kb/decisions/08-核心索引结构.md:155`「一叶 233 条 140 字节记录」）。借的下限在参照里是「leaf_capacity().div_ceil(2)」（`:69`），实现里是「usize::try_from(INODE_LEAF_RECORDS.div_ceil(2)).expect("117")」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-core/src/inode_tree.rs:196`）——同一条式子各写一遍，E176 的 `BORROW_FLOOR_MAIN = 117`（`/home/fy5090/code/singlefs/research/e7-index-bench/src/bin/e176_inode_leaf_merge_capacity_and_rewrites.rs:26`）是第三份。
3. **读侧同一个结构体**：参照读的是发布路径交出来的内存结构「fn leaves_of_the_version(version: &TransactionOutput) -> Vec<ReferenceLeaf> {」（`:223`）的「.inode_leaf_containers」（`:225`），不是从盘上重读；写盘的编码 / 解码、池级 checker 都不在这个比里（文件头自己写明方向只有这里钉，池级 checker 不红）。
没比的：叶的分隔 key（借完右邻的分隔 key 改成它这一版的最小 key，`inode_tree.rs:510` 起那句赋值）、重写清单与 origins 都不在 `ReferenceLeaf`（只有身份与记录号）。我把那句赋值删掉（变异 m9）：新用例照绿，核心单测与原有 `inode_leaf_absorb_and_borrow_on_the_unlink_path` 红（`logs/inode-tree-mutation-results.txt`）。

推翻条件：E176 M3 的规则与用户定案「M3 连借一起做」的字面不符（则两边一起错，这条用例不会红）；或参照与实现的某个共用常量被改（改 `INODE_LEAF_RECORDS` 两边一起动）。

### K4.2　固定种子的历史覆盖到借后吸收、连锁吸收、先右与先左结局不同的那几形了吗

判定：**只覆盖到「先右与先左结局不同」和「借」各一次；借后吸收、左吸收、删空摘片三形没覆盖；连锁吸收按不变量推不可达**。

量法：给拷贝里的用例加事件计数（补丁 `…-sonnet-model/patches/reference-test-event-counters.patch`，只加计数和结束时一行 `eprintln!`，不改比的逻辑），`cargo test --offline -p singlefs-harness --test inode_leaf_merge_and_borrow_against_an_independent_reference -- --nocapture`，原样输出：

```
EVENTS merge_right=1 merge_left=0 chain>=2=0 borrow=1 borrow_then_merge=0 empty_drop=0 empty_drop_then_merge=0 both_fit=1
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.97s
```

（种子 53，`const SEED_OF_THE_HISTORY: u64 = 53;`，`:27`；事件是整条历史上的次数。）

| 形 | 次数 | 谁钉 |
|---|---|---|
| 先右 / 先左结局不同（两对都装得下） | 1（`both_fit`） | 新用例与核心新单测都钉住；变异 m1（先查左邻）两条都红，原有 harness 的 `inode_leaf_absorb_and_borrow_on_the_unlink_path` 不红（`logs/inode-tree-mutation-results.txt`），所以这一条是新加的真增量 |
| 借 | 1 | 新用例钉：m6（借的条数向上取整）、m7（下限 116）、m8（下限用 `>`）三个变异新用例都红；原有测试也红 |
| 借后吸收 | 0 | 新用例不钉；m2（借完不再并）新用例与核心库 `inode_tree::` 21 条**照绿**，只有原有 harness 的 `the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it` 红 |
| 左吸收（只有左邻那一对装得下） | 0 | 新用例不钉；m5（把左邻那一支整个关掉）新用例、核心库 244 条、原有 `inode_leaf_absorb…` 4 条、再加 10 个相关 harness 二进制（`tools/wide-targets.txt`）**全照绿**。这一形是可达的：我在核心库测试里加了探针，三片各 117 条删最后一片的一条，未变异输出 `PROBE counts=[117, 233] numbers=[1, 234] retired=1`，m5 下是 `PROBE counts=[117, 117, 116] numbers=[1, 234, 467] retired=0`（`patches/inode-tree-probe-left-absorb.patch`） |
| 删空摘片后再并（`empty_drop`） | 0 | 变异 m4（删空后不再并）新用例、原有 `inode_leaf_absorb…` 4 条、核心库 244 条**照绿**（没跑那 10 个相关二进制）；但按下面的不变量推，它不可达，等价变异 |
| 连锁吸收（一次删除并两轮以上） | 0 | 变异 m3（并一轮就停）新用例、原有 `inode_leaf_absorb…`、核心库 244 条、10 个相关 harness 二进制**照绿**；同样推的不可达 |

不可达的推法（推的，没造镜像穷举）：E176 对 M2、M3 断言「任意相邻两片之和 ≥ 234」（`/home/fy5090/code/singlefs/research/e7-index-bench/src/bin/e176_inode_leaf_merge_capacity_and_rewrites.rs:368`「M2、M3 任意相邻两片之和 ≥ 234」），追加只落最右、满了才开新片也满足它。删一条之后能并的一对恰好和 = 233，并完幸存那片 = 233 满，再与任何邻居相加 > 233，不会第二轮；删空的那片删前只有 1 条，所以它的两个邻居删前都是 233 满，摘掉后左右邻之和 466 > 233。借之后左片 + 借出片的和不变（≥ 234），借出片与它右邻可并，并完幸存片再与借方、与更右的邻居的和仍 > 233，同样只有一轮。因此 m3、m4 在可达状态上与原函数等价，不算漏；m5 不是等价变异。

推翻条件：造出一段可达历史，m3 或 m4 的输出与原实现不同（则连锁 / 删空后再并是可达的，参照用例没覆盖）；或 E176 M3 的「相邻两片之和 ≥ 234」在某个操作之后不再成立。

## 五、K5　钉值（第 52、54 批）

做法：每个钉值分「能从原理（常量、条款、算式）算出来」与「抄的实测值」，前者我自己重算，后者查它的构造过程能不能自证；再问变化背后是不是这三批改了产品行为。先说结论：**这三批没有改任何产品行为里会让这些钉值变的东西**——第 52 批改的 `recovery.rs` 只动窄条目镜像的次序，第 53 批改的是新不变量、检查器、GPU 扫描内核（只此一项让展开量 +316）和一个纯重排，第 54 批只动测试；下面每个钉值的变化，来源是更早合入的第 49、50 批（台阶行、退役树、扫描内核变大），这三批只是把钉值追上去。

### K5.1　从原理算的（我重算了）

| 钉值 | 判定 | 重算 |
|---|---|---|
| dispatch：`states_per_scratch_lane` 1674、12 GiB 的 10 044、借卡 `mebibytes_a_judge_needs` 45 个状态 57（第 968、975、999 行） | 一致 | 草稿区字数「pub const SCRATCH_WORDS: usize = 320_688;」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker-tier/src/crash_judge_gpu.rs:146`，这批没改它），每状态 `320 688 × 4`，借卡公式「SCRATCH_WORDS * 4 + 2 * VERDICT_WORDS * 4」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:156`，`VERDICT_WORDS` = 14）。`python3` 原样输出：`bytes_per_state 1282752 lane 1674 12GiB 10044`、`per_state_incl_verdict 1282864 mebibytes45 57`。钉值行：「assert_eq!(states_per_scratch_lane(BINDING_LIMIT), 1674);」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:968`）、「assert_eq!(states_per_dispatch(BINDING_LIMIT, 12 * 1024), 10_044);」（`:975`）、「assert_eq!(mebibytes_a_judge_needs(&tables_of(45), usize::MAX), 57);」（`:999`）。1702 → 1674 是 `SCRATCH_WORDS` 变大的余波，不是这三批的产品改动：这三批的补丁在 `crash_judge_gpu.rs` 里只改文档和覆盖表（改动全文里 `SCRATCH_WORDS` 那行没动），HEAD 里它是 255 504（`git show HEAD:crates/singlefs-checker-tier/src/crash_judge_gpu.rs \| grep -n SCRATCH_WORDS` 输出 `158:pub const SCRATCH_WORDS: usize = 255_504;`），哪一批把它抬到 320 688 我没逐批查 |
| unlink 流全量两个闭式 1_463_812_228、1_463_812_278 | 一致 | 最后一批的单元写 20 → 24：kb 一侧「最后一批（1 → 0，批大小 1） | 12：t2 t4 t5@0.0.61 t5@0.1.61 t5@1.0.0 t5@1.1.0 t5 t6 tr@0 tr t7 t8」（`/home/fy5090/code/singlefs/.claude/kb/layout/03-third-txn.md:58`）= 12 个单元 × 两盘 = 24 写，该行段序列写 `24+2+1+2`；测试里的段向量「22, 2, 1, 2, 24, 2, 1, 2」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:345`）。闭式差 `(1<<24) − (1<<20)`：`python3` 原样输出 `unlink closed forms 1463812228 1463812278`（旧值 1448083588、1448083638 各加 15728640），七个单元段合计旧新都是 `189`（`7 unit segments old/new 189 189`）；钉值行「assert_eq!(closed_form_with_two_states_per_write, 1_463_812_228);」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:1005`）、「assert_eq!(closed_form, 1_463_812_278);」（`:1028`）。段向量那一项另有实测对照：主 agent 改前那趟日志 `~/.local/share/singlefs/work/m3-batch50/merge-copy/checker-quick.log` 第 1051 行 `left` 里正是 `…, 22, 2, 1, 2, 24, 2, 1, 2]`。全量那条 `#[ignore]` 用例没跑（十四亿个状态），两个闭式只是算术 |
| 树 ID 水位 65548 → 65549 | 一致 | 十三棵树从 65536 发，最后一棵退役树 65548，水位 65549：`/home/fy5090/code/singlefs/.claude/kb/layout/03-third-txn.md:21`「树 ID 水位第一个文件版本那次之后 65549」；钉值「(CheckpointTxg(3), 65549),」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs:113`） |
| 新池新建文件流：I-9.31–I-9.34 为 0、I-9.35 为根已落的状态数 9、I-9.36 / I-9.37 为 0 | 一致（推） | 这条流只有 txg 3 一版有 inode 树，退役树、谱系树都是零根，两条新不变量没有对象（K2.1 的表，零根 / 单容器不判）；这些值同时出现在主 agent 的日志 `CHECKER` 行里，是原理与实测两边对上 |

### K5.2　AccountingRootSplit / AccountingLeafSplit 的新形状（行为变了吗）

钉值：「shape_after: "L8 L8 I2",」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker-tier/tests/crash_enumeration_tree_split_streams.rs:201`）、「shape_after: "L5 L5 L6 I3",」（`:214`）。来历是 W3 的台阶行：D5 已定项 2 射程规定记账为水位变了的每次发布留一行台阶行（`/home/fy5090/code/singlefs/.claude/kb/decisions/05-快照-空间记账机制.md:53`「记账在 inode 号水位变了的每次发布给这一统计量留一行台阶行 (g, H)」），随根一直留着。所以被录的那次空发布不删这一行，其余 14 行先删、再按 key 升序插 15 行（水位统计量 12 排最后）。

我用自己写的 Python 重排了一遍（只用规则「满了从中间切、左半留 ⌈n ÷ 2⌉ 条」，出处「fn split_in_the_middle(node: &mut PlanningNode) -> PlanningChild {」（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-core/src/code_two_tree.rs:447`）与「let right_keys = keys.split_off(keys.len().div_ceil(2));」（`:450`））；与实现者的小模拟共用「切法」这一前提，不共用钉值）。原样输出：

```
RootSplit with step row   : [8, 8]
RootSplit without step row: [8, 7] (old: 15 new rows incl. gen-4 watermark row replace all 15)
LeafSplit with step row   : [5, 5, 6]
LeafSplit without step row: [5, 5, 5]
```

有台阶行 = 新钉值，没有 = 旧钉值，两个方向对上。这不是这三批改的行为（产品代码在这两条流上的行为 W3 起就是这样），是钉值追上 W3。**判定：一致，没有把行为变化当成该变；AccountingLeafSplit 与 unlink 全量那几个闭式在 checker 档，我没有跑，只有上面的算术与重排**。

### K5.3　抄实测值的（分片打印行 / 进度文件哈希、展开量）

| 钉值 | 判定 | 我核到哪一步 |
|---|---|---|
| 打印行哈希（第 54 批）`a4643c92…`（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:853`），进度文件哈希 `92ff4595…`（`:867`） | 一致于「只多了 I-9.36、I-9.37 两项」；钉值本身是抄的实测值，**复核不了它与产品行为的对应** | `sha256sum ~/.local/share/singlefs/work/m3-batch54/repin/printed-b53.txt` 输出 `a4643c92e6836ef465aba245ee096127bebc0b65f7342a090fd1570d9c48bea6`；`sed 's/ I-9.36=0\/0\/46 I-9.37=0\/0\/46//g' …printed-b53.txt \| sha256sum` 输出 `4d42c7ca5ca9e8e16e5c7bdde36198f576d40a98ad9a8dfb35fb33947161e354`（上一版钉值）；该文件两行 `GOLDEN_TALLY` 都含 `I-9.36=0/0/46 I-9.37=0/0/46`。进度文件：`/tmp/singlefs-crash-enumeration-sharded-2542539-golden-progress/` 里那份还在，`sha256sum` 输出 `92ff4595a87332b8a915defde97ee09f05db4066dbcd4e58ece5f9214eb4481d`，与钉值相同。主 agent 在钉值之后的那一趟 `~/.local/share/singlefs/work/m3-batch50/merge-copy/sharded-b54.log` 里 `test result: ok. 10 passed`。没核的：那份 /tmp 文件是不是**那一趟红了的运行**留下的，实现者的依据只是进程号相邻（2542539 与日志里的 2542549），我能核的是它的哈希与内容的 CRC，认不出它的来历 |
| 打印行哈希（第 52 批）`4d42c7ca…`、进度文件 `1553ba8e…` | 一致于「只多了六项并换了计划哈希」 | `grep -E '^(LAYER0_\|GOLDEN_)' ~/.local/share/singlefs/work/m3-batch52/fix-reds/sharded-printed.txt \| sha256sum` 输出 `4d42c7ca5ca9e8e16e5c7bdde36198f576d40a98ad9a8dfb35fb33947161e354`；删六项（` I-9.30=9/0/37 … I-9.35=9/0/37`）并把计划哈希换回 `4987883029b8…` 后 `sha256sum` 输出 `4b8b1e84e16b6a4ecd22a8bc28a15c75a5dbef3b99a1cfddc9e5b98703ed04b1`，正是更早的钉值 |
| 展开量 387_470 / 余量 12_530（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1361`） | 复现了数；钉的是测量值 | 实现者草稿工具 `m3-batch53/gaps/naga-check`（`kernel_inlined_size` 的逐句转写，naga =30.0.1）拷到自己目录编好跑冻结副本的内核源码，原样输出 `checker-scan ok inlined_expressions=387470 margin=12530`（另 `recovery … 332666`、`checker-walk … 346278`，与文档里的数相同；`logs/naga-check-frozen.log`）。该测试本身在 checker-tier 里，没跑 |
| 第 54 批 `reuse_after_raising_the_floor` 豁免表加六行（`/home/fy5090/.local/share/singlefs/work/m3-fixes-code-r1/repo/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs:63` 起） | 一致于被测代码的实际输出；表是抄实测的理由文字 | 六行变异（变异表第 3727 起）在拷贝上各红在 `…reuses_it` 那一条（`logs/rows-b54.log`）；这六行证的是「豁免表里少一行这条用例就红」，没证产品代码报的就是表里那句理由——理由取自 `walk` 的公开常量（`NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION` 等），产品代码改了常量时表跟着变，没有独立的钉 |

推翻条件：把 I-9.36 / I-9.37 在新池新建文件流上的评估数改成非 0 的产品改动（比如让零根也评估），上面的打印行哈希就是被行为变化推着变；或者主 agent 在 checker 档重跑 `unsharded_enumeration_prints_and_writes_byte_for_byte_what_it_did_before_sharding` 仍红。

## 六、K6　判别力

### K6.1　三批新加与替换的变异行拦得住「改回去」吗

判定：**harness 档能跑的行全部拦住（逐行在拷贝上复跑点名的那一条用例，都红）；checker 档的行复核不了**。

| 行（`crates/mutations.tsv`） | 结果 |
|---|---|
| 第 52 批两行（第 3711、3712 行） | 红在点名的用例（K1.2 的输出） |
| 第 53 批 F1：「三片各 117 条删中间一条」（核心单测） | 红：`inode_tree::tests::removing_from_the_middle_of_three_half_full_containers_absorbs_the_right_neighbour_first ... FAILED`（`logs/rows-b53-rest.log`） |
| 第 53 批 F1：「随机历史上发布路径与…参照逐次比」 | 行本身在 `rowrun` 里两次超时没跑完（内存包装排队，本机当时被别的会话占满）；同一个替换文（左邻先查）我用 `tools/mut.py` 的 m1 在这条用例上跑过：`every_publish_of_a_random_history_… ... FAILED` |
| 第 53 批 F3 三行 + 两行替换（实三、第 49 批 W3 Y1 Opt2） | 红，各自点名的那一条（K3.2 的输出；两行替换在 `logs/rows-b53-rest.log`） |
| 第 53 批 F4 三行（I-1.7 退役记录宽、I-9.32 两句） | 红在点名的用例（`logs/rows-b52-53.log`） |
| 第 53 批 F2 CPU 三行（I-9.36 退役树、谱系树，I-9.37 加一） | 红在点名的用例（同上） |
| 第 54 批六行（第 3727 起） | 红在 `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it`（`logs/rows-b54.log`）；这六行证的是「豁免表少一行用例就红」，是在改**测试**的表，不是改产品代码，K5.3 已说 |
| 「实审 A3d…」改名行、第 53 批 F2 GPU 两行 | **复核不了**：点名的用例在 `singlefs-checker-tier`（checker 档），我不跑 |

推翻条件：把某行的改动施到产品代码上、点名的用例照绿。（我复跑的这些行没有出现。）

### K6.2　还有哪几种改坏的写法照绿

我在拷贝上做了变异（脚本在模型目录），逐条是「改什么 → 跑了哪些 → 结果」。**照绿的这些，是这三批新加的测试没钉住、也没有别的测试钉住（在我跑的范围内）的写法**：

| 写法 | 跑的范围 | 结果 | 该钉的条款 / 理由 |
|---|---|---|---|
| I-9.36 / I-9.37 的「沿叶序容器号严格递增」整句去掉（w1：`true \|\| left_container < right_container`） | S 集 5 个二进制 | **全过** | I-9.21 同形的前半句；探针单测证明判法本身会红（K2.2），但没有整镜像用例、没有变异行钉这一句，GPU 一侧同样没有 |
| 谱系产品容量写成退役的 959 / 退役写成 1359（w3、w3b） | S 集 | **全过** | 新用例全用旋钮压到 1 条的镜像，产品容量常量从没碰上一棵真有两片满容器的树；这是 I-9.36 里唯一的数 |
| 「非最右装满」只判不足、不判多一条（w11：`==` 改 `>=`） | S 集 | **全过** | 多一条产品下 I-1.7 同时红，整体不漏，I-9.36 自己那半没单钉 |
| 最右那片也要求装满（w7） | S 集 | **全过** | 没有一张合法的产品容量多容器镜像 |
| 谱系树读不全 / 退役树读不全 / 整版树表读不出时不报「射程里判不了」（w4、w5、w6） | S 集 | **全过** | K2.1 里「读不全」那一半没有用例；同一个函数里 I-9.21 的同类也未必有，我没查 |
| 旧的墓碑旋钮入口把谱系旋钮改成 `Some`（w10） | S 集 | **全过** | `lineage: None`（`walk.rs:12124`）这句没人钉 |
| 退役树旋钮 +1（w12） | `checker_known_bad_images` 230 条 | **全过** | I-9.37 退役树这一半只有「旋钮下合法镜像成立」，没有「多于旋钮值」的退役坏镜像；谱系那一半有 |
| M3 左吸收（只有左邻那一对装得下）整支关掉（m5） | 新用例、核心库 244、原有 `inode_leaf_absorb…`、另 10 个相关 harness 二进制 | **全过** | 可达形，探针证明行为不同（K4.2）；新独立参照的固定种子没走到这一形，且没有「某形至少走到一次」的断言（只断言 `both_fit` 见过一次） |
| M3 借完不再并（m2） | 新用例、核心库 `inode_tree::` | 新用例**照绿**，只有原有 harness 用例红 | 新用例没走到借后吸收 |
| M3 借完不改右邻分隔 key（m9） | 新用例 | 新用例**照绿**，核心单测、原有 harness 红 | 新参照不比分隔 key |
| 回退的树 ID 水位也只取没被抛弃的根（mt1） | 回退相关 12 个 harness 二进制 | **全过** | 条款 `/home/fy5090/code/singlefs/.claude/kb/decisions/23-journal的角色与格式.md:385`「树 ID 水位照旧」，K3.2 |
| M3 连锁吸收只并一轮（m3）、删空摘片后不再并（m4） | 同 m5 | 全过 | 按「相邻两片之和 ≥ 234」推等价变异，不算漏（K4.2） |

另外两条没有变异、只是读到的缺口：
1. 恢复里的条目宽判只拒窄于 34（`records.rs:329`），I-1.10 写的是等于，宽条目镜像恢复放行（K1.1 补充）。
2. I-9.36 / I-9.37 只判叶序严格递增，不判连号与从 1 起（K2.2），D18 / layout 写了连号。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1.1 先核宽之后别的坏法的错误成员 / 合法镜像 | 一致 | 被拒的镜像集合不变；只有「窄条目 + 另一处坏」的组合改报条目宽，条款 I-1.10「先于按字段表解条目」同向；合法镜像没有窄条目。补充：I-1.10 写「等于」，恢复只拒窄于（没改、没验） |
| K1.2 两个读者次序 / 可写挂载 | 一致 | 重建与冷走读都是宽 → 数 → 逐条；可写挂载走重建，没有第三份实现；两条新用例各钉一个读者，次序换回去各红自己 |
| K2.1 合法镜像误报 | 一致（谱系「满了才关」规则没说） | 判法只判非最右装满、旋钮下不超；写侧三棵树共用一个追加函数，恒满；谱系那一行条款只在容器号后写「规则同墓碑容器」，装满靠类比 |
| K2.2 坏镜像漏报 | 部分一致 | 提早关、旋钮下多一条拦住；容器号不连 / 不从 1 起不判（规则没说到 I-9.36，D18 / layout 写了）；严格递增判得到但没有任何测试钉 |
| K2.3 CPU / GPU | 读码一致，对拍复核不了 | 位号、常量、判法同形；GPU 一棵树 256 片上限超出交 CPU；旋钮不传 GPU，I-9.37 恒不置位，GPU ≡ CPU 产品入口 |
| K2（kb） | 冲突 | `invariants.md:32`（I-1.7）谱系记录宽 16，D18 与代码 24；`invariants.md:12` 条数待改 |
| K3.1 抽出前后行为 | 一致 | 同一条式子换了地方；差别只在纯函数求值次序；core 库 244 过（含三条新单测） |
| K3.2 三条单测钉的事 | 一致于 inode 号水位三件；树 ID 水位那件没钉 | 三条各拦自己的变异；树 ID 水位改成只取没被抛弃的根，12 个回退相关 harness 二进制全过 |
| K4.1 参照独立性 | 不调 `inode_tree` 一致；共用前提三样 | 规则同出 E176 M3（条款没写合并时机）、容量常量 233 与下限式、读发布路径的内存结构；没比分隔 key |
| K4.2 覆盖 | 部分 | 固定种子只走到先右先左分歧 1 次、借 1 次；借后吸收、左吸收、删空摘片 0 次；左吸收是可达形，关掉它全照绿；连锁与删空后再并按不变量推不可达 |
| K5 钉值 | 一致 | dispatch、unlink 闭式、树 ID 水位、树分裂形状都能从原理重算且对上；打印行 / 进度文件哈希是实测，核了「只多两项」；展开量复现了数；没有把这三批的行为变化当成该变，变化来自更早合入的批 |
| K6.1 变异行拦住改回去 | 一致（harness 档） | 能复跑的行点名的那一条都红；A3d 改名行与 GPU 两行复核不了 |
| K6.2 照绿的写法 | 有 | 严格递增整句、产品容量常量对调、读不全不报判不了、退役旋钮 +1、M3 左吸收关掉、借后吸收 / 分隔 key（新用例）、树 ID 水位也只取没被抛弃的根 |

## 没做什么

- 没跑任何 `singlefs-checker-tier` 的测试、没跑门禁阶段、没跑全量 `cargo test`：所以 GPU 对拍两条新用例、两行 GPU 变异、`crash_enumeration_*` 钉值（树分裂形状、unlink 闭式、分片哈希）、`bad_disk_input_campaign` 里的「A3d」用例、`crash_judge_dispatch` 的展开量测试，我都只能读码与算术核，没有跑出结果。要主 agent 在提交时的 checker 档跑；红了先看是不是本报告 K5 里说的「只有这几处是推 / 抄」。
- 没造整镜像坏镜像：K1.1 的「窄条目 + 另一处坏」组合、K2.2 的「容器号不连」整镜像、K2.1 的「崩溃恢复抛弃段」镜像都是读码推的，探针只直接调了私有判法函数。
- 没跑全部 181 个 harness 二进制，只跑了与这三批相关的几十个（名单在各格与模型目录里）。变异「照绿」只在我点名跑的范围内成立，范围外的测试可能拦住。
- 随机历史（`random_histories`）那一趟本机被别的会话占满，跑到 235 秒超时，没有结果，没作数。
- 没验宽于 34 的记账条目镜像（K1.1 补充）；没核 `m3-batch53/gaps/naga-check` 与 `kernel_inlined_size` 逐句相同之外的事（读了两份源码，结构相同）。
- 没替主 agent采纳或出判决；上面「规则没说」只是指出条款没逐字写，不是说该怎么补。
- 模型目录：`research/prompts/m3-batch52-54-code-r1-sonnet-model/`（README、tools、patches、logs、SHA256SUMS）。
- 草稿 / 编译产物的处置：已删（交回前 `rm -rf` 这三个路径，`ls` 复核没有了）自己建的仓副本 `~/.local/share/singlefs/work/m3-batch52-54-code-r1-sonnet/repo/`、编译目录 `…/target`、`…/target-naga`（会话共用暂存目录没用）；保留的只有该目录下的小文件（`tools/`、`pristine/` 变异前原件、`naga-check/` 源码、各 `.log`），复跑时按模型目录 README 从冻结副本重拷。

交回前 `python3 research/scripts/cite-check.py research/prompts/m3-batch52-54-code-r1-sonnet-output.md` 末行：`✓ 核了 51 处引文，对上 51 处，没判 0 处（逐处列在上面）`。
