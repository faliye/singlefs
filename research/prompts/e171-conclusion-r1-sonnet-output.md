# e171-conclusion-r1 正推腿报告（Sonnet，正推）

轮名 e171-conclusion-r1（第一轮），格 G1–G5。kb 引文写文件自己的行号；代码引文写冻结副本 `/tmp/claude-1000/e171-conclusion-r1/tree/` 下的路径与行号（报告里的 `crates/…`、`research/e7-index-bench/…` 都指冻结副本，cite-check 用 `--root` 指它）。我自己的两个只读复核脚本在 `research/prompts/e171-conclusion-r1-sonnet-model/`：`diff_ops.py`（逐条 op 比基线与行的导出）、`recount.py`（从比对器产物重数全放行与只靠一条名单）。

## 0. 摘要（第一轮意见，不替主 agent 判）

- G1：12 行全放行（甲乙）里，我从条款原文推出「状态或恢复结果在已定条款下非法」的有 11 行（15、23、27、34、42、112、242、269、270、271、277；其中 27、277 取决于 M4 的等号读法，见 G2），92 一行条款欠一句（里程碑页自己写「它不要求正确」）。三条互不重叠的格子在 D13 已定项 6 表最后一列里只承认了两条（O1、O2），O3 那一格没承认。233 在甲乙里 O1-崩溃判官红，不是三条的共同盲区。
- G2：执行员判「对这一批等价」的 5 行（15、27、34、112、277）一行都站不住：15、34 的落点由 D3 已定项 10 ③「取最低的」定死，112 违反 D23 已定项 14 注 1，27、277 违反 D13 已定项 1 M4「恰是 j*」。它们是「条款非法而伤害落在这批域外」，不是等价。102、233、265 记「不稳定」对。
- G3：A(c) 取自实现自己写的根，坐实；再加一条执行员没写的：O3 根本不调实现的恢复（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/crash_dump.py:4`），所以实现读者一侧的坏它结构上看不见，只有 O1-崩溃判官红得了。这是设计边界，该写进「验不到什么」；D13 O3 行的「判什么」「独立性来自哪里」两格要窄化。
- G4：算术对（我重数：D=39、只靠 O1 的行 10→12、全放行名单不变）。表取自 `mutations.tsv` 对「有」那一边是存在性结论、不受偏向影响，对比例（12/37）有偏；Q3「有一行只靠一条」同理。域外推：只到这两条流、无故障、无重挂载的层 0 无故障态。
- G5：D13 已定项 6 的依据可以引 E171 的读数，不引「N 行真盲区」的推论（还没过三轮）；关系写「支撑 + 窄化」，不写推翻；「全部故事就是这三个 oracle」那一句要改。C407 不能照原改法还清（「关一条仍有别的腿红」在 25 个被抓行里只有 2 行成立），要改写成钉矩阵。

## 1. 复核命令与原样输出（我这一次真跑的）

重数名单（脚本只读产物）：

```
$ python3 research/prompts/e171-conclusion-r1-sonnet-model/recount.py research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out
include_untouched_but_red=False D=37 quiet_rows_alpha=12 15,23,27,34,42,92,112,242,269,270,271,277
   only=O1:10[32,45,108,115,172,233,237,249,250,281] O2:13[8,65,86,87,90,99,102,135,234,238,245,258,265] O3:0[]
include_untouched_but_red=True D=39 quiet_rows_alpha=12 15,23,27,34,42,92,112,242,269,270,271,277
   only=O1:12[32,45,108,115,155,161,172,233,237,249,250,281] O2:13[8,65,86,87,90,99,102,135,234,238,245,258,265] O3:0[]
categories {'compile_failed': 6, 'counted': 37, 'self_crash': 6, 'untouched': 232}
```

补位产物上同一脚本输出的三张名单与上面逐字相同，`categories {'counted': 37, 'self_crash': 6, 'untouched': 238}`；37 个分母行按「被几条 oracle 判红」分：0 条 12 行、1 条 23 行、2 条 2 行（第 11、244 行）、3 条 0 行（我另算的，同一产物批 1）。

坏实现的写流与基线逐条比（`diff_ops.py <基线流> <行流>`，导出在 `research/target/e171-dumps/`）：

```
row 23  流一: first=26  op 26: write dev=0 off=823164928 len=32768 bytes_differing=5 …@53:0x01->0x00
row 15  流一: op 22 起 写落点 822149120 -> 823132160（槽 50180 -> 50240），op 24-45 整体后移一段（开放段也移）
row 34  流二: first=498  op 498: 822083584 -> 822411264（槽 50176 -> 50196）；call 12 raise_floor 是 440-498，call 13 publish 从 498 起
row 42  流二: first=537  op 537: write dev=0 off=0 len=4096 … @481:0x11->0x0b
row 269 流一: first=0    op 0: write dev=0 off=0 len=4096 … @469:0x00->0x01
row 271 流一: first=44   op 44: write dev=0 off=823328768 len=16384 … @92:0xe2->0xa9
row 242 流一: first=8    op 8: base=write_fua dev1 off 4194304 len 512  row=write dev0 off 4096 len 4096
row 270 流一: ops_base=57 ops_row=53   流二: ops_base=595 ops_row=565  first=20 base=barrier row=write
```

第一处不同的 op 序号、不同条数与页上相同（页 `.claude/kb/experiments/171-三oracle合谋测试.md:86`、`:87`、`:89`、`:90`、`:169`、`:170`、`:171`），字节偏移我只核了前几处；行 15、34 的落点数页上也有（`:86`、`:89`），我多做的是把落点换成槽号去对 D3 的字节表。

恢复一侧的差（只读各行的 `impl-verdicts.tsv`，两种读法的第一列）：行 27 与 277 在流一 6 个态、流二 51 个态上基线都比变异新一版（例：`1,2` 对 `1,1`；`1,3`（读回文件）对 `1,2`（没有文件））；行 112 两条流各 3 个态基线 `0,0`、变异 `1,1`。个数与页上相同。

## 2. 我用来判「非法」的两样东西

**判法**：一个态（或一次恢复的结果）「非法」= 某句已定条款（决策正文、`layout/01-first-txn.md` 字节表、`invariants.md` 的判据）写明了不该有这个态或这个结果；不是「读回内容同」「镜像自洽」。里程碑页不算条款：`.claude/kb/milestone/02-second-txn.md:7` 写「它不要求正确」（`.claude/kb/milestone/01-first-txn.md:3` 同句）。

**「共同」二字**：登记的字面是「三条 oracle 对同一行都不红」。D13 已定项 6 表最后一列是：O1「磁盘结构是否自洽；崩溃语义；两份实现对同一条 POSIX 语义的共同误解」（`.claude/kb/decisions/13-验证路线.md:161`），O2「只判结构自洽：一个块被覆盖成另一份合法内容，Merkle 链照样自洽、记账照样对」（`.claude/kb/decisions/13-验证路线.md:162`），O3「规约本身可能写错」（`.claude/kb/decisions/13-验证路线.md:163`）。落到字节级的坏法（容器号、头 ID、tail、落点）：O1、O2 两格已承认，O3 那一格没写它的 A(c) 取自实现自己写的根（见 G3），所以「表已经承认了它」只有三分之二。

## 3. G1　逐行（非法依的条款行、三条为什么放行、补在哪、什么会推翻）

引的条款都整节读过（D8 已定项 6 的「新池新建文件写」「射程」，D16 已定项 1 的字段表与射程，D16 已定项 7 的定案与射程，D23 已定项 3、14、18，D3 已定项 10）。

**行 23（inode 叶容器号 1 → 0）**　条款非法，三条放行属实。
- 非法依：「一个码 3 叶（类型 2、容器号 1、出生代 3、记录数 1、记录宽 140）」（`.claude/kb/decisions/08-核心索引结构.md:203`）；「每个容器号都是某条记录首次插入时的 inode 号」（`.claude/kb/decisions/08-核心索引结构.md:205`）；字节表「容器号 = 1」（`.claude/kb/layout/01-first-txn.md:260`）。diff：op 26/27 偏移 53 的 0x01→0x00，op 28/29 的内部节点条目里身份引用同改（I-9.2 两侧一致，所以内部自洽）。
- O1：`ObservedRoot` 只有 key、journal 计数器、F、文件、实例表、分配记录（`crates/singlefs-harness/src/model.rs:594`），不读盘。O2：I-9.4 只判「打包记录类型 2 的容器内每条记录的 inode 号 ≥ 容器号」与沿叶序严格递增（`.claude/kb/invariants.md:290`；代码「inode >= container」`crates/singlefs-checker/src/walk.rs:1930`、「left.container < right.container」`crates/singlefs-checker/src/walk.rs:1748`），一片叶 0 ≤ 1 成立。O3：A(c) 由录制流里的根槽写建（「加录制流里每一次根槽写按流序各一版」`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/version_table.py:3`），投影不进 inode 树（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/root_ring.py:22`）。
- 补：O2 加一条「容器号 ≥ 1」。但条款只有「第一版的唯一文件取 inode 1（0 无效）」（`.claude/kb/decisions/08-核心索引结构.md:199`），只管第一版的唯一文件；先补一句条款再补 O2（零轮）。分裂之后 I-9.4 会红（推的，没跑；这批域没有分裂，页 `:461` 写「没有分裂」）。
- 推翻：决策正文里另有一句允许容器号为 0。我 grep `0 无效` 在 decisions 与 invariants 里只命中 :199 一处。

**行 15（用户数据落点不从单元区起点扫）**　执行员记「对这一批等价、条款没定」，我判条款非法、三条放行（重判，见 G2）。
- 非法依：「数据单元取最低的、起点 32768 对齐、两槽都空、整跨度都过合取的起点」（`.claude/kb/decisions/03-空间分配.md:204`）；字节表「已选设备内槽号最小、32768 对齐、两槽都空」（`.claude/kb/layout/01-first-txn.md:309`）、t1 的落点「不在开放段 [50240, 50304) 里的最低 32768 对齐空槽对」（`.claude/kb/layout/01-first-txn.md:67`）。diff：流一 op 22 的数据单元槽 50180 → 50240，后面的节点整体后移一段，开放段从 50304 起，与 D3 已定项 10 ⑤ 的落点表（t2 在 50240）不符。页上记「条款没定」是因为 grep 只找了「起扫」「单元区起点」（页 `:86`），漏了「取最低的」。
- O1：`ObservedAllocationRecord`（「pub struct ObservedAllocationRecord {」`crates/singlefs-harness/src/model.rs:651`）只有设备、分配代、是否已释放，没有槽号；模型只数占用槽数（「pub occupied_slots_upper_bound_per_device: u64,」`crates/singlefs-harness/src/model.rs:362`）。O2：`invariants.md` 里没有判落点政策的一条（`grep -n '最低\|落点政策\|first-fit' .claude/kb/invariants.md` 无输出，我跑的）。O3：投影里没有落点（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/root_ring.py:22`）。
- 补：落点取决于之前释放的槽，单镜像的 O2 判不了；放 O1（模型的分配器加槽号，重）或放 O3 的固定流写侧期望（G5）。伤害在空间紧的池（推的）。
- 推翻：找到一句已定条款说「落点只要在单元区内的空槽即可」。

**行 34（抬 F 生效之后不放开扣住的槽）**　同行 15 的重判。
- 非法依：「已释放 ∧ 释放代 ≤ max(F_生效, 一圈窗口里最旧有效根)」（`.claude/kb/decisions/16-发布语义.md:33`）加 D3 已定项 10 ③（`.claude/kb/decisions/03-空间分配.md:204`）。diff：流二 call 12 `raise_floor`（op 440–498）之后 call 13 `publish` 的数据单元，基线落槽 50176（单元区第一个槽），变异落 50196（op 498 起：822083584 → 822411264）。
- 三条放行的理由同行 15（O1 没有槽号，O2 没有落点政策的不变量，O3 投影没有落点）。页上的「会分开的历史」（空间只够复用那几槽的池，O1-对拍会红）合理，但它是伤害，不是非法的依据；非法在这批态里已经成立。补：同行 15。

**行 42（卸载把 F 抬到现行那一版带的 F）**　条款非法，三条放行属实；O2 那一格是条款自己写明的边界。
- 非法依：「推一串空发布把 F 抬到现行那一版的 txg」（`.claude/kb/decisions/16-发布语义.md:42`）；「卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版」（`.claude/kb/decisions/16-发布语义.md:57`）。diff：流二 op 537 偏移 481 的 0x11→0x0b（F 17 → 11）。
- O1：模型没有卸载的答（「模型没有卸载的答，不比；模型停在这里」`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:981`；`model.rs` 里 `answer_*` 一族没有 unmount，`grep -c unmount crates/singlefs-harness/src/model.rs` 输出 0）。但 `ObservedRoot` 已带根的 F（`crates/singlefs-harness/src/model.rs:597`），补一个答就比得出。O2：I-7.9 带卸载记号那一支只判上限（`raised_floor <= newest_root_txg_before_the_raise`，`crates/singlefs-checker/src/walk.rs:4445`），F 没抬的根整条跳过（`crates/singlefs-checker/src/walk.rs:4427`）；条款自己写「它判的是一张镜像上 F 有没有抬过头，不判 F 该不该抬」（`.claude/kb/invariants.md:63`）。O3：投影带 F（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/root_ring.py:30`），但 A(c) 就是这一行写的 F = 11 的根。
- 补：O1 加 `answer_unmount`（`crates/singlefs-harness/src/model.rs`，答案是根上的 F = 现行那一版的 txg），E171 装置 `step_unmount`（`e171_three_oracle_collusion.rs:981`）改成调它。D13 已定项 6 表没承认它：O1 格写的「崩溃语义」不覆盖「卸载这次调用该做什么」。

**行 92（覆盖写照抄上一版的写入时间）**　我判「条款欠一句」，不判真盲区。
- 决策正文只定了位置：「atime / mtime / ctime 秒，各 8」（`.claude/kb/decisions/08-核心索引结构.md:188`）。「写入时间是发布参数」只在里程碑页（`.claude/kb/milestone/02-second-txn.md:103`、`.claude/kb/milestone/01-first-txn.md:103`），而两页自己写「它不要求正确」（`.claude/kb/milestone/02-second-txn.md:7`）；验收句只算线索。回答 G1 末问：里程碑页的验收句不算条款。
- O1：`write_time` 在 `model.rs`、`model_comparison.rs` 零命中（我跑 `grep -c write_time` 两份都输出 0）。O2、O3 不知道这次发布的时间参数（单镜像；投影不进 inode 树）。
- 补：先在 D8 已定项 6（或 8）补一句「覆盖写的 inode 记录的 atime / mtime / ctime 取这次发布的时间参数、不抄上一版」，再给 O1 的 `ObservedFile`（`crates/singlefs-harness/src/model.rs:605` 起）加写入时间。补句之前 92 不进真盲区名单。
- 推翻：在 decisions 里找到这句。我 `grep -n 'atime\|mtime' .claude/kb/decisions/*.md` 只命中 D8 的 :188、:190（字段位置）与不相关的 :167。

**行 242、270（持久顺序）**　条款非法、三条放行属实，但伤害落在这批域外。
- 非法依：「系统配置槽在根槽持久之后再更新，每个 checkpoint 一次」与「fsync 等系统配置轮换持久之后才返回」（`.claude/kb/decisions/16-发布语义.md:181`）；射程「发布返回前加屏障」（`.claude/kb/decisions/16-发布语义.md:186`）。「不合法的态」是层 0 切段模型上数的（页 `:464` 自己写「真机上一次 FUA 写与随后的写怎么排，不在这一批里」），数法用的是 D13 已定项 4「段里每块有写的盘都被自己的屏障或 FUA 放行才关段」（`.claude/kb/decisions/13-验证路线.md:12`）；基线上同一判法数 0（页 `:164`）。数法本身我没重跑（复核不了：要比对器的 `segments_of_the_write_stream`），只核了写流的次序变化（行 242 op 8 由 FUA 变普通写、行 270 的 op 数 57→53、595→565）。
- O1：崩溃判官比的是恢复择到的根是否「模型提交过的某一版」且不比盘上已持久的最新根槽旧（「if chosen < persisted {」`crates/singlefs-harness/src/model.rs:2639`），不看系统配置。O2：I-7.12 只比 F，系统配置落后一次发布在单镜像里合法（`.claude/kb/invariants.md:66`）。O3：系统配置写若在某次根槽写之前、且与前一根写之间隔着屏障，归属记 None（「owners.append(position + 1 if barrier_between == 0 else None)」`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/version_table.py:75`），A(c) 不看它。
- 伤害域：实现自己的注释写明末尾屏障买的是「这次的根槽一时读不出，可写挂载的见证判据……把这次发布当被抛弃」（`crates/singlefs-core/src/transaction.rs:1238-1246` 一段；行号区间我只核了起止），D23 已定项 3 射程同向：「只减轮换次数或不等它持久就返回」（`.claude/kb/decisions/23-journal的角色与格式.md:136`）。即要「根槽读不出」加「再挂载」，E171 两条流没有故障注入、没有崩溃后的可写挂载（登记读法 2，`research/prompts/e171-preregistration.md` 第二节）。
- 补：不补三条之一。归故障注入档与崩溃注入第二截（D13 已定项 7 射程写明可写挂载由崩溃注入覆盖），登记欠账。O3 的 `version_table.py` 已认出系统配置写的位置，加两条异常（系统配置写先于同一次发布的根写、轮换与下一次调用的第一批写之间无屏障）就能红，但那是语法检查、不是规约推导，且伤害在域外，我不把它排在第一件。
- 推翻：层 0 全量里这两类态本来就由别的判官在门禁红掉，并且它们在无故障域里有可见伤害（我没找到）。

**行 269（见证值取成槽世代号）**　条款非法、三条放行属实；伤害在重挂载轴。
- 非法依：「mkfs 写 tail 0，mkfs 之后第一次取号读到 0、写 0：新池新建文件的字节不变」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）；「tail 8 字节存 jsn 的 48 位计数器」（`.claude/kb/decisions/23-journal的角色与格式.md:501`）。diff：流一 op 0/1 偏移 469 的 0x00→0x01，环里还没有记录；之后的发布把它写回正确值，所以只有取号之后到下一次发布之前的态非法（页 `:169`：流一 31 态、流二 78 态）。
- O1：`ObservedRoot` 不带系统配置。O2：读了 tail（「journal_tail: read_u64(slot, SYSTEM_CONFIGURATION_TAIL_OFFSET),」`crates/singlefs-checker/src/lib.rs:285`），`invariants.md` 里没有一条判它的值（`grep -n 'journal_tail' .claude/kb/invariants.md` 无输出，我跑的）。O3：只在锚点断言里读 tail（「anchor_assertion id=A4」`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/executor_main.py:182`），E171 比对器不读 `anchor_assertion`（`grep -n anchor_assertion` 在比对器的 `.py` 里无输出）；12 个全放行行的 `o3.out` 里 `holds=false` 都是 0 条（我跑的）。
- 伤害（推的，没跑）：恢复不先信 tail（「tail 语义上是派生态」`.claude/kb/decisions/23-journal的角色与格式.md:125`），读恢复不受影响；影响在下一次可写挂载取 c_见证 的判据（`.claude/kb/decisions/23-journal的角色与格式.md:392`）：tail = 1 大于环里任一记录的计数器，会被当成「系统配置见证过更新的发布」。
- 补：O2 加一条值域不变量（tail ≤ 环里读得出的最大自证记录计数器；编码照 D23 已定项 18）。环回绕与撕裂的边角没验，零轮。

**行 271（deleted_inodes 的树表条目头 ID 写 0）**　条款非法、三条放行属实，补点最清楚。
- 非法依：「`deleted_inodes` 与墓碑树写 inode 树的树 ID（65537）」（`.claude/kb/decisions/08-核心索引结构.md:250`）。diff：流一 op 44/45 起（树表单元 16 KiB）。
- O1 不读树表；O2 的树表条目切法只有诞生 txg 与种类两个偏移（「const TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8;」`crates/singlefs-checker/src/walk.rs:2953`、「const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 8 + 2;」`crates/singlefs-checker/src/walk.rs:2955`），`invariants.md` 里「头 ID」零命中（我 grep 的）；O3 投影只带树表单元指针。
- 补：O2 加一条「树表条目头 ID 照 D8 已定项 8 那一行的表取值」；条款现成、单镜像可判。

**行 233（只在丙里全放行）**　不算三条的共同盲区。
- 甲乙里 O1-崩溃判官在 45 个态上红（「o1_crash_judge_red_states=45」`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out:4473`），只是丙把崩溃判官拿掉。D13 已定项 6 表 O1 格写「崩溃语义」不归 O1（`.claude/kb/decisions/13-验证路线.md:161`），但项目的接线是把 O1 崩溃判官算进调用点：「崩溃判官的全部四条判据合取」（`.claude/kb/decisions/13-验证路线.md:99`）。照项目自己的接线读，233 是「只靠 O1-崩溃判官」，已在 Q3 名单里。
- 非法依：M2 语义层「其余字段等于所选根」（`.claude/kb/decisions/13-验证路线.md:32`）。批 2 没有施加了记录的态，取样点不敏感，记「不稳定」对（页 `:174`）。

## 4. G2　「对这一批等价」与「取样点不敏感」

**先对口径**。页上的定义是「每个态连同实现在态上的恢复都在条款允许的集合里」（`.claude/kb/experiments/171-三oracle合谋测试.md:82`），再附一条「会分开的历史（推的，没跑）」。`.claude/rules/mutation-sampling.md` 的口径更严：只有「在所有输入上与原式同值」才叫等价（`.claude/rules/mutation-sampling.md:16`），「确认在所有输入上同值 ⇒ 才是等价变异」（`.claude/rules/mutation-sampling.md:90`）；写得出一个会分开的历史，规则归它为「在我选的取样点上恰好同值，换一个取样点就不同」那一类（`.claude/rules/mutation-sampling.md:17`）。所以按规则，只要执行员自己写了「会分开的历史」，那 5 行就不是等价，是取样点不敏感。这是口径问题，下面再看条款。

| 行 | 执行员 | 我 | 依据（条款原文行） |
|---|---|---|---|
| 15 | 等价（条款没定落点） | 条款非法，三条放行 | D3 已定项 10 ③（见 G1），页 `:86` 的 grep 漏了「取最低的」 |
| 34 | 等价（落点不是条款定的值） | 条款非法，三条放行 | D16 `:33` 加 D3 `:204`（见 G1） |
| 27 | 等价（调用没返回，新旧两版都允许） | 恢复结果违反 M4，三条放行 | D13 已定项 1 M4，见下 |
| 277 | 等价（新旧两版都在「根 + journal 任意前缀」里） | 同 27 | 同 27 |
| 112 | 等价（`0,0` 与 `1,1` 都允许） | 恢复结果违反 D23 已定项 14 注 1，三条放行 | D23 `:369`，见下 |

**行 27、277（M4）**。差在恢复一侧：我读两种读法的 `impl-verdicts.tsv`，流一 6 个态、流二 51 个态上基线都比变异新一版（例：`1,2` 对 `1,1`；`1,3`（读回文件）对 `1,2`（没有文件）；行 277 个数同 27；页 `:88`、`:172` 同数）。基线在这些态上多施加了记录，即点名单元两份都验过；变异停在前一版。D13 已定项 1 的 M4 写：「Recover 返回的根恰是 j」且 j 是「记录条数之和不超在飞上限的最远一版」（`.claude/kb/decisions/13-验证路线.md:34`，原文 `j\*` 带反斜杠，我只引到 j）。这些态里最远一版是新版，变异返回旧版，是 M4 要防的「少施加」。页上的论证「调用没返回，新旧两版都在允许集合里」用的是 O1-崩溃判官的弱判据：择到的根是模型提交过的某一版、不比盘上已持久的最新根槽旧（「if chosen < persisted {」`crates/singlefs-harness/src/model.rs:2639`），根槽没落，所以旧版过关。
- 读法分歧（我核不动，交主 agent）：M4 射程句说「M4 连同择根才排除」已确认发布被丢（`.claude/kb/decisions/13-验证路线.md:55`），执行员的读法可以是「M4 只保已确认的」；而定案表的字面是「恰是 j」，不带「已确认」。按字面这些态非法；按「只保已确认」它们合法（调用没返回）。要判的是 M4 那个等号要不要带「已确认」限定，不是读数。
- M4 的落点不在三条里：「E165（恢复重放有界穷举与变异杀伤界） 的最远一版判据在界内判」（`.claude/kb/decisions/13-验证路线.md:34`）。记录核对器也不判它：`check_records`（`crates/singlefs-checker-tier/src/crash.rs:51`）的两条判据是「根在而记录一条都不在」（`crates/singlefs-checker-tier/src/crash.rs:86`）与「恢复自称的 txg ≥ 某次发布、那次发布的某个单元全部副本都缺席」（`crates/singlefs-checker-tier/src/crash.rs:88`），没有「记录齐全而恢复没施加」这一条；E171 里层 0 / 记录核对器一列在 27、277、112 上都是 `layer0_or_record_checker_red=false`（`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out:4442`、`:4446`、`:4451`）。

**行 112（前缀跨实例边界）**。D23 已定项 14 注 1：「所选根是 mkfs 的第 0 代根时它一条记录都不覆盖，之后的记录全属于更新的实例 ⇒ 一条都不施加」（`.claude/kb/decisions/23-journal的角色与格式.md:369`）。这 3 个态里所选根是 `0,0`（mkfs 第 0 代根），变异施加了 1 条实例 1 的记录得 `1,1`，违反这一句。页上的「`0,0` 与 `1,1` 都在允许集合里」（`.claude/kb/experiments/171-三oracle合谋测试.md:92`）是 O1-崩溃判官的读法，不是条款的读法。此行的变异在 `crates/singlefs-replay-prefix/src/lib.rs`（Verus 核所在的文件，`.claude/kb/decisions/13-验证路线.md:31`），点名的测试是层 0 快档计数 `layer0_quick_tier_matches_the_full_tally_shape`（`crates/mutations.tsv` 的第 6 段）。

**「会分开的历史」真会分开吗**：27、277 要「已返回的发布 + 根槽读不出」，即故障轴；O1 的模型对这类历史有答（「pub fn answer_mount_writable_with_the_newest_root_unreadable(」`crates/singlefs-harness/src/model.rs:1636`、「pub fn answer_mount_writable_with_the_roots_rebuilt_from_their_records(」`crates/singlefs-harness/src/model.rs:1669`），所以在故障注入档上 O1-对拍能红（推的，没跑），只是 E171 的两条流没有故障。15、34 的分开史是空间只够复用那几槽的池，模型数占用槽数（`crates/singlefs-harness/src/model.rs:362`）能红（推的）。112 的分开史是环里留着被抛弃时间线的记录，页自己写两条流里没有（`:92`）。

**102、233、265 记「不稳定」**：对。批 2 在流二只取 17 段（页 `:94`：「批 2 在流二只取 17 段」），102、265 的红态落在批 1 的第 60、73 与 59、71 段；233 的红态要「基线施加了记录」的态，批 2 一个都没有。这是 `.claude/rules/mutation-sampling.md` 第六类「几何敏感性」那一格的正确记法；登记读法 3 把批 1 当主判，所以 Q1、Q3 的结论字不翻（页 `:94`、`:175`）。我没找到反例。

**推翻这一节的现象**：(a) 主 agent 判 M4 的等号带「已确认」限定 ⇒ 27、277 回到「条款允许」，但仍是「取样点不敏感」（有会分开的历史），不是等价；(b) 找到一句已定条款允许落点任取 ⇒ 15、34 回到条款允许；(c) 112 没有任何办法回到允许：注 1 的句子直接点名了这个态。

## 5. G3　O3 的 A(c) 取自实现写出的根

**坐实**：版本表 = 基镜像上的现行那一版加录制流里每一次根槽写按流序各一版（「加录制流里每一次根槽写按流序各一版」`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/version_table.py:3`；根槽写由「(prepared.device_identity, prepared.byte_offset) not in root_slot_keys」认，`version_table.py:48`；每一版的投影是「projection_of_root_bytes(prepared.new_content)」，`version_table.py:61`）。A(c) 是流序落在 [ack, begun] 的各版（「A(c) = 版本表里流序落在 [ack(c), begun(c)] 的各版」`version_table.py:91`）；ack 由调用返回点定（「if version.return_point is not None and version.return_point <= largest_index:」`version_table.py:97`）。判法是「landed_is_allowed = recovery.landed is not None and any(version.projection == recovery.landed for version in allowed.allowed)」（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/executor_core.py:127`），`recovery` 是执行器自己对崩溃镜像的恢复（`executor_core.py:124`）。

**执行员没写的一条**：O3 不调实现的恢复，也不读实现的判定：「不读导出给的布局分类，不读实现判定」（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/executor_core.py:1`），`impl-verdicts.tsv` 「这里一律不打开」（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/crash_dump.py:4`）。所以 O3 判的是「实现写出的盘面，在规约的恢复规则下落到哪一版」，不判实现的恢复。读者一侧的坏（行 27、112、277、233，也包括只靠 O1 的 108、115、237、281）O3 结构上红不了；E171 里这些行 O3 一列全是 quiet 不是巧合。

**这一读法下 O3 推导的与不推导的**（文件在 `research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/`）：
- 推导（照 kb 文字另写，不与实现共代码）：系统配置择槽与几何（`system_configuration.py`）；根环读槽、自证、择新，择新 txg 为主、实例代号破平局（「择新在 checkpoint_txg 平局时按它高者赢」`root_ring.py:58`）；扫 journal 环（`journal_ring.py`）；取前缀的五条口径（`replay_prefix.py`）；崩溃镜像的三态写与撕裂字节（`crash_image.py`）；ack / begun。
- 不推导：一版里除六个投影字段之外的任何内容（实例代号、txg、树表单元指针、中央映射树根指针、树 ID 水位、F，`root_ring.py:22`；A(c) 里每一版都是实现写的根槽原样）；系统配置槽的字段值（只拿来择几何与择槽）；树表、叶、节点、分配记录的字节（只在点名单元校验里当黑盒）；调用的语义（调用标签只用来给版本挂返回点，`version_table.py:60`）；实现的恢复。
- 能红的类：实现写出的根、记录、环链、点名单元合起来，让规约恢复落到「实现从没发布过的版」或「比已确认的更旧」。实证在 Q2：O3 在坏镜像上「P 行 26 张红 22、盲测 40 张全红、已知坏镜像 41 张严格红 28」（`.claude/kb/experiments/171-三oracle合谋测试.md:114`）。E171 的 37 个行都是改实现，没有一行把根与它的记录写成互相矛盾，所以 O3 一列 0 红。
- 整类无感的类：实现一致写错的值。页上同一判断：「实现一致地写错的字段（容器号、写入时间、F 的取值、记账行）它照样判」（`.claude/kb/experiments/171-三oracle合谋测试.md:465`），并自己写「这是读执行器代码读出来的，没有另造一行只让 O3 红的坏实现去核」（同行）。我读同样的码得同样的结论，同样没另造反例（造反例是攻方腿的活）。

**边界还是缺陷**：分开答。
1. A(c) 取自实现、O3 不调实现恢复：设计边界。「实现发布过哪几版」是历史的事实，规约文字推不出来，推它要操作语义，那是 O1 / RefFS 的活（「O3（独立规约执行器） 仍是崩溃点集合的独立执行器，职责不变」`.claude/kb/decisions/13-验证路线.md:110`）。但 D13 已定项 6 把 O3 写成「崩溃点截断后的每个磁盘状态是否落在规约允许的集合里」（`.claude/kb/decisions/13-验证路线.md:163`），独立性写成「只照 decisions 与 invariants 的文字重新推导」（同行）：「规约允许的集合」实际是「实现自己发布过的版的集合」。这是文字欠账，该写进「验不到什么」。
2. O3 的版本表代码已经认出系统配置写与根写的相对位置（`version_table.py:66`–`:75`）却不判它，也不比调用的后置条件（卸载后 F、取号后 tail）：同一条 lane 里的缺口（行 42、242、270 的一部分），可补。补的是「规约写侧期望」，范围到版本表这一层为止；叶内字节（行 23、271、15）不在 O3 的廉价范围里，要再写一个文件系统的写侧。
3. 「O3 × RefFS」这一对：D13 已定项 13 写 O3 有代码、能交判定行之前登记为空格（`.claude/kb/decisions/13-验证路线.md:346`）。今天 O3 能交判定行了，但 RefFS 的 (lo, hi) 入参仍由录制流取（D13 已定项 3 射程写「(lo, hi) 与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官盘级下界那一条判据的入参输入同源」，`.claude/kb/decisions/13-验证路线.md:110`），所以这一对也不能把 ack 的来处从实现里拿出来。C447 的第 ② 步（O3 与 RefFS 逐态比）不解决这件事。

**推翻这一节的现象**：造出一行改实现、只让 O3 红的坏实现（写出的根与它的记录矛盾）会补上「整类无感」的另一半，不推翻「一致写错无感」；造出一行「一致写错的值而 O3 红」的坏实现会推翻本节。我没造，不是核过。

## 6. G4　触达定义与测量方法

**155、161 的算术**：对。重数见第 1 节：把两行算进去 D = 37 → 39，只靠 O1 的行 10 → 12（加 155、161），被抓行 25 → 27，全放行名单 12 行不变。两行在批 1、批 2 都是 O1-对拍红、O1-崩溃判官、O2、O3 quiet（`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out:2478`、`:2479`、`:2574`、`:2575`）。对 Q1、Q3 的判不影响：Q1 是「有」，两行不是全放行；Q3 是「有一行只靠一条」，只多了两行。
- 缺口的真实范围：登记读法 4 的前提句「它在这一批上与正常实现逐字节同行为，哪条 oracle 都不该红」（`research/prompts/e171-preregistration.md:40`）在这两行上不成立。页上说别的未触达行有没有同类差异看不出来（「别的未触达行有没有同类差异，装置没记成功调用的交回值，看不出来」`.claude/kb/experiments/171-三oracle合谋测试.md:462`）。我能补的一半：232 个未触达行每行都有批 1、批 2 的 `row_matrix`（我数的：232 行批 1、232 行批 2 都在），O1-对拍在那些行上红的只有 155、161，所以 O1 比得到的返回值字段上没有第三行；比不到的字段（写入时间、模型没有答的调用的返回值）看不出来。
- 方向：这类行被归「未触达」，不进分母也不进全放行名单，所以全放行名单只会被少数、不会被多数；对「有」不影响，对比例有偏。改法：触达加第四样「逐调用成功交回值的摘要与基线不同」，装置把每次成功调用的交回值摘要写进 `call-outcomes.tsv`（今天只在对拍判出不同时记）。

**表取自 `crates/mutations.tsv`**：该文件第一行写明每行带「必须红的测试名」（`crates/mutations.tsv:1`），所以每一行本来就被某条点名的测试抓过。12 行全放行的点名测试（我按行名读第 6 段，分档照第 2 段的 `-p`）：harness 档 6 行（15、34、92、242、269、271）、checker 档 3 行（42、112、270）、核心 / 取前缀的单元测试 3 行（23、27、277）。
- 对 Q1「有」：这是存在性结论，一个态一个态地成立（行 23、15、271 的非法态在批 1 的全持久态上就有），表怎么取都不影响；偏向的是比例（12/37 不是任何总体的比例：点名测试多半是字节级、条款级的小测试，通用 oracle 本来不看那个粒度，表偏向「找得到放行行」）。同一事实的另一面：每个放行行有点名测试罩着，所以「三条的共同盲区」不是「系统的盲区」，D13 已定项 6 的「本工程功能正确性的全部故事就是这三个 oracle」（`.claude/kb/decisions/13-验证路线.md:167`）这一句被这 12 行直接推翻。
- 对 Q3：「有一行只靠一条」也是存在性，23 / 25 个被抓行只靠一条（我另数：被 0 条红 12 行、1 条 23 行、2 条 2 行即第 11、244 行、3 条 0 行），O1 与 O2 只重叠 2 行，O3 一行都没红。偏向：被抓的行是「通用判官看得见」的那些，粒度不同本来就少重叠，所以数偏向「只靠一条」；但表里没有任何一行三条都红，这个不受偏向解释。
- 表之外：mutations.tsv 的行是人为写来证明某条测试会红的，不含「没人写过测试的坏法」，也不含「三条 oracle 同源地误读一句条款」那一类：变异只能测能力重叠，测不了同源误读。E171 因此回答的是「三条在这批坏实现上有没有共同看不见的」，不是「三条是否同源」；D13 已定项 6 的 nonce 恒为 0 那个例子（一类字节级的写错）与这批行同类，没有被换成同源误读。

**域外推到哪里为止**：两条流共 18 次调用（流一 3 次、流二 15 次，`calls=3`、`calls=15`，`research/results/e171-three-oracle-collusion-2026-10-03-s1-baseline-run1.out:27`、`:71`）；层 0 全量态数 16 777 260 与 1 662 648 564（同文件 `:27`、`:71`），批 1 取 53 + 397 = 450 态（`:35`、`:110`），占 450 / 1 679 425 824 ≈ 2.7e-7。没有分裂、没有删除、没有空间紧的池（页 `:461`）、没有故障注入、没有崩溃后的可写挂载。外推规则：「存在一个非法而三条都放行的态」是单调的（域加大不会取消），所以 Q1 的「有」可以外推到任何包含这些态的域；行级的「全放行」不单调（域加大可能多出红态，102、233、265 已是例子），12 与 23 这两个数只在这一批态上成立；「没有」一边（今天没有）不能外推，更不能写成三条独立（登记第六节末尾已写，见 `research/prompts/e171-preregistration.md:462`）。

## 7. G5　D13 已定项 6 与 C407 怎么改（零轮提议，不写回；写回要过二三轮）

**7.1 依据能不能引 E171**。今天的字面：「这一条依据今天还不引它」（`.claude/kb/decisions/13-验证路线.md:173`）。我的意见：可以引读数（观测），不引「N 行真盲区」的推论。能引的：Q1 存在性（有一个条款非法而三条都放行的态，行 23、271、15 在全持久态上就是）、Q3「23 / 25 个被抓行只靠一条」、O3 一列在 37 行上 0 红（重数见第 1、6 节）。不能引的：「真盲区 N 行」这个数（我与执行员在 5 行上相反，M4 的「已确认」读法待主 agent 判，92 条款欠一句）；「三条独立」这个说法（E171 测的是能力重叠，不是同源误读；登记第六节末尾自己写了不许这样写，`research/prompts/e171-preregistration.md:462`）。

**7.2 关系写什么**。支撑 + 窄化，不写推翻。理由：已定项 6 自带一支「若三者全部放行，说明存在共同盲区，回头补 oracle」（`.claude/kb/decisions/13-验证路线.md:166`），E171 是这一支被触发的观测，不是对定案句的反驳；被反驳的只有两处表述：O3 行的「判什么」「验不到什么」（见 7.4），以及「本工程功能正确性的全部故事就是这三个 oracle」（`.claude/kb/decisions/13-验证路线.md:167`）。页 `.claude/kb/experiments/171-三oracle合谋测试.md:478` 那一行今天写「备料」，三轮判决之前保持，判决之后改「支撑」；本轮意见交判决时一起写。

**7.3 「回头补 oracle」今天落成什么**：逐条补的有 5 处（都是零轮，没验过），登记欠账的 2 类，什么都不补的 1 行。

| 行 | 补在哪 | 补什么 | 前置 / 状态 |
|---|---|---|---|
| 23 | O2：`crates/singlefs-checker/src/walk.rs`（I-9.4 旁，`:1930`） | 容器号 ≥ 1 | 先补条款句（D8 已定项 6 现有「0 无效」只管第一版唯一文件，`.claude/kb/decisions/08-核心索引结构.md:199`） |
| 271 | O2：同文件树表条目切法旁（`:2953`） | 头 ID 照 D8 已定项 8 那一行的表取值 | 条款现成（`.claude/kb/decisions/08-核心索引结构.md:250`） |
| 269 | O2：`crates/singlefs-checker/src/lib.rs:285` 旁 | tail ≤ 环里读得出的最大自证记录计数器 | 编码照 D23 已定项 18；环回绕、撕裂的边角没验 |
| 42 | O1：`crates/singlefs-harness/src/model.rs` 加 `answer_unmount`；E171 装置 `step_unmount`（`:981`）改调 | 卸载交回的根上 F = 现行那一版的 txg | 条款现成（D16 `:42`、`:57`） |
| 92 | 先补一句条款，再 O1 的 `ObservedFile` 加写入时间 | 覆盖写取这次发布的时间参数 | 条款欠一句，补句之前不进真盲区 |
| 15、34 | O1 分配器落点（模型要加槽号与「取最低」，重），或 O3 固定流的写侧期望 | 数据单元槽号 | 选哪边要代价数，交岔路 |
| 27、277、112 | 不是三条之一：记录核对器 `check_records`（`crash.rs:51`）加「记录齐全而恢复没施加」；或把 E165 的最远一版判据当第四列接进 E171 | M4 的等号 | 先定 M4 的等号带不带「已确认」；112 不依此（D23 注 1 点名） |
| 242、270 | 登记欠账：归故障注入档与崩溃注入第二截 | 域 | 伤害要「根槽读不出 + 再挂载」，E171 域外 |
| 233 | 不补 | — | 甲乙里 O1-崩溃判官已红 |

O3 一侧的补法只有一种：「写侧期望」，范围到版本表这一层（系统配置写的相对次序、卸载后 F、取号后 tail），不碰叶内字节；表里 15、34 的第二个选项就是它。

**7.4 文字改法（零轮）**
1. D13 已定项 6 的 O3 行第 5 格追加：「A(c) 取自实现自己写出的根槽（版本表 = 基镜像上现行那一版 + 录制流里每一次根槽写），不调实现的恢复；实现一致写错的值（根、系统配置、树表与叶内字段）判『在』，实现读者一侧的错看不见（E171：37 行 0 红）」。依据：第 5 节。
2. 同行第 3 格「规约允许的集合」改成「规约的恢复规则下，实现自己发布过的版的集合」。
3. 已定项 6 的「本工程功能正确性的全部故事就是这三个 oracle」改成「三个 oracle 是独立规约的主干，不是全部：每个坏实现另有 `crates/mutations.tsv` 点名的测试罩着，崩溃一侧另有 E165、Verus 与层 0 的计数判官；三条的共同盲区由合谋测试逐行登记（E171 页）」。依据：12 行全放行行各有点名测试。
4. 依据段「这一条依据今天还不引它」换成引 E171 读数（7.1 能引的三样）加一句「回头补 oracle 的落点逐行见 E171 页」。
5. O1 行第 5 格补一句：没有模型答的调用（卸载）不对拍；登记读法 6 已经是这样做的（见 `research/prompts/e171-preregistration.md:42`）。
6. 欠块 C407 那一句换成 7.5 的改写。
要不要给 O3 的「验不到什么」加句：要，就是第 1 条。

**7.5 C407：改写，不还清**。C407 的改法列写「合谋测试做成门禁阶段：一份已知坏镜像（或坏实现）喂给三条 oracle，至少一条必须红」与「把其中一条 oracle 的判定关掉，必须仍有别的腿红」（`.claude/kb/checks-owed.md:327`）。E171 的读数让这两句不能照字面还：
- 「关一条仍有别的腿红」在 25 个被抓行里只有 2 行成立（第 11、244 行，第 1 节），23 行关掉那一条就全放行；照字面做成门禁一上来就红，且红的不是实现的错。
- 「三条全绿即判红」：把 12 行全放行行放进名单，门禁一上来就红；不放进去，门禁对共同盲区一声不吭。只有钉矩阵同时管这两头。
- 「O3 零代码」那一格前置已过时：O3 有代码（E170），前置现在是 C447（`.claude/kb/checks-owed.md:362`）。
- 今天已有的钉：`research/scripts/replay.sh:262` 把整表读数登记成 exact 复跑；它钉的是当前读数，12 行全放行时它照样绿，不是 C407 要的判官。装置与驱动已写（页 `:20`、`:21`），门禁里没有它（`grep -rln 'e171\|E171\|合谋' .claude/gate.d/*.sh` 只命中 `.claude/gate.d/checker-independence-and-sync.sh:40` 的一句注释，我跑的）。
改写成（提议）：「合谋测试的门禁形态取钉矩阵：计入分母的行（今天 37 行，加没触达却红的 2 行）逐行钉 O1-对拍、O1-崩溃判官、O2、O3 四列的红 / 不红；任一格翻面、或多出一行全放行，门禁红；每个全放行行挂一个补 oracle 的欠账号（7.3 的表）；判别力自证沿用 E171 的阳性对照（四条 oracle 各翻一次）。」费用（算术，没跑）：两条线道每行约 47 秒（5 569.6 s ÷ 117、7 607.9 s ÷ 164，`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out:286`、`:290`），39 行约 31 分钟、只跑 12 + 2 个全放行与没触达却红的行约 11 分钟，属提交时的重阶段，不进 harness 档（D13 已定项 15）。

**推翻本节的现象**：(a) 二三轮把 M4 的「已确认」读法判成限定、再加上 15、34 的落点被判成条款允许，7.3 的表缩成 5 行；(b) 找到 O3 一侧的 anchor / 异常行在比对器里本来就算红（我 grep 比对器 `.py` 零命中）则 7.4 第 1 条的「0 红」要加限定；(c) 门禁里已有别的阶段判了这 12 行的哪一行（我只 grep 了 `.claude/gate.d/*.sh`）则 7.5 的「门禁里没有」要改。

## 8. 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| G1 行 23 | 条款非法，三条放行 | D8 `:203`、layout `:260`；O2 只判 ≥；补 O2，先补一句条款 |
| G1 行 271 | 条款非法，三条放行 | D8 `:250`；O2 不读头 ID；补 O2 一条，条款现成 |
| G1 行 269 | 条款非法，三条放行；伤害在重挂载 | D23 `:392`；O2 读 tail 不判值；补 O2 值域 |
| G1 行 42 | 条款非法，三条放行 | D16 `:42`、`:57`；O1 无卸载答，O2 的 I-7.9 条款自己写明不判该不该抬；补 O1 |
| G1 行 92 | 条款欠一句 | 只有里程碑页，页自己写「不要求正确」；先补条款 |
| G1 行 242、270 | 条款非法，三条放行；伤害在故障 + 再挂载轴 | D16 `:181`、`:186`；E171 无故障；不补三条，登记欠账 |
| G1 行 233 | 不是三条的共同盲区 | 甲乙里 O1-崩溃判官红，D13 `:99` 接线把它算进调用点 |
| G1「共同」二字 | 表只承认 O1、O2 | D13 `:161`–`:163`；O3 那一格没写 A(c) 取自实现 |
| G1 里程碑页算不算条款 | 不算 | `milestone/02-second-txn.md:7` 自己写「它不要求正确」 |
| G2 行 15、34 | 不是等价，条款非法 | D3 `:204`、layout `:309`；执行员的 grep 漏了「取最低的」 |
| G2 行 27、277 | 不是等价，违反 M4 的字面；读法待主 agent 判 | D13 `:34`「恰是 j」；`:55` 的「已确认」限定与否未定 |
| G2 行 112 | 不是等价，违反 D23 注 1 | D23 `:369` 点名「一条都不施加」 |
| G2 行 102、233、265 | 「不稳定」对 | 批 2 只取 17 / 79 段；mutation-sampling 第六类 |
| G3 | 设计边界 + 文字欠账；另有一条同 lane 可补的缺口 | A(c) 取自实现、O3 不调实现的恢复；D13 O3 行两格要窄化 |
| G4 触达 | 执行员算术对 | D 37→39，只靠 O1 10→12，全放行名单不变（我重数） |
| G4 表与域 | 存在性不受偏，比例有偏；外推只到这两条流的无故障态 | 12 行各有点名测试；450 / 1 679 425 824 个态 |
| G5 依据 | 可引读数，不引「N 行真盲区」 | 7.1 |
| G5 关系 | 支撑 + 窄化，不写推翻 | 7.2；「全部故事」一句要改 |
| G5 回头补 | 5 处逐条补、2 类欠账、1 行不补 | 7.3 |
| G5 C407 | 改写成钉矩阵，不还清 | 「关一条仍有别的腿红」只对 2 / 25 行成立 |

## 9. 没做什么

- 没跑整表、装置、层 0、cargo（重型测试与编译不跑）；没重跑 O3 执行器；没造反例行（只让 O3 红 / 一致写错而 O3 红的行，攻方腿的活）。
- 没复核行 242、270 的「条款不许的态」数（要比对器的切段函数 `segments_of_the_write_stream`），只核了写流次序与条数。
- 没验 Verus、E165 在 27、112、277 上红不红（推的）；所有「伤害」一栏都是推的，没跑。
- 没读禁读清单里的文件；没判决、没采纳，M4「已确认」限定交主 agent。
- 没动仓内任何文件：新写的只有本报告、`research/prompts/e171-conclusion-r1-sonnet-model/` 下的两个只读脚本（`diff_ops.py` sha256 `6e8642ecf33d063d99101cb09124706774082d4d6f7c1caf8383919e773bbc2f`，`recount.py` sha256 `e411796a6c07a9c4a3cc2bad3625ce9c348db60116f82e68e0c1e31e3b1bf639`）与草稿目录 `/tmp/claude-1000/e171-conclusion-r1-sonnet/`。

## 10. cite-check 末行

`python3 research/scripts/cite-check.py research/prompts/e171-conclusion-r1-sonnet-output.md --root /tmp/claude-1000/e171-conclusion-r1/tree --root /home/fy5090/code/singlefs --background research/prompts/_e171-conclusion-r1-background.md` 的末行：

```
  ✓ 核了 48 处引文，对上 48 处，没判 0 处（逐处列在上面）
```
