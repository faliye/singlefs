# m3-formal-code-b17-r1 云端攻方（Opus）报告（2026-10-04）

攻击面：K2（I-5.5 认法与判据）、K4（8 行变异）、兼 K1（GPU 与 CPU 同判）。代码引文读冻结副本 `/tmp/claude-1000/m3-formal-code-b17-r1/tree/`（快照清单逐文件 sha256 核过：`sha256sum -c --quiet research/prompts/m3-formal-code-b17-r1-snapshot/crates-sha256.txt` 在副本里无输出、退 0）。本报告里所有量出来的数都是**攻方副本上的数**（冻结树拷进草稿目录，harness 档，内存稀疏盘 4 GiB × 2），主 agent 要在入库装置上重做才能引。checker 档、GPU、WGSL 一行都没跑，那几格写「推的」。

## 复跑

```
bash research/prompts/m3-formal-code-b17-r1-opus-model/tools/rerun.sh /tmp/claude-1000/<新的工作目录> 8
```

工作目录：仓根；它自己经 `research/scripts/run-with-memory-cap.sh`（原型 2G、变异行 3G）与 `research/scripts/capped.sh <线程>` 起每一次 cargo；没带重型测试（只编、只跑 `singlefs-harness` 的测试目标）。跑完读 `<工作目录>/out/summary.txt`，每行一格，与下面「各格判定一览」里的计数逐项对。按攻方副本上各次单跑相加估约 25 分钟（没有整份连跑过一遍，见「这条腿自己的限度」）。

| 文件 | sha256 |
|---|---|
| `opus_b17_first_data_unit_attack.rs`（原型：形 A–E，harness 测试目标，名字不带 layer0） | 1829481ae0ca8153bd1ef75987cecdcb32276c87a1beee40ef48b46bcb5689aa |
| `walk-fix-arms.patch`（副本 walk.rs 上两条改法臂，环境变量 `OPUS_I55_ARM=f1/f2` 选，不设等于入库那一版） | cf664d1996d980bf7d3a3b807ffb9cf17c645617992a6adbda62e579cb7d4d1e |
| `rows.tsv`（diff 第 1496–1503 行那 8 行变异原样） | 7aa72e7e79b1628ed91b113b2e12e1cebbd9a1ebdd7998616e0850102f8ea4fe |
| `rows-h1.tsv`（攻方自造坏实现 H1：用户数据从 U + 4 起扫） | 76a4ea72511957b88140d25a2404a318b6dbdd2c6171069389591b2c80283c7c |
| `rows-h2.tsv`（攻方自造坏实现 H2：U、U + 1 一律当空；自崩，见下） | 27c50dc2c4d27c4c7eda2b87fbe70955dd884b86edfbe593e61c5b9852e0362d |
| `apply_row.py` | badae629ed1a44229729b7fdf0139dcac3eb8392453720f4f1a0b072c62cff1d |
| `run_row.sh` | b98d1b9b7f03c963f645cc120486da2e701e6d71e919a70b6aaae1d6a5130fdd |
| `tools/rerun.sh` | 03580bf36dcdfd48d8380adf129aac10fc8bcc90f5fc9f39dd784a93113f6fe8 |
| `out/counts.txt`（各格计数，从 out/ 下日志数出来） | 151f4a05b7f371cd0fdc3b25f52678f018b4e3fa66055ae45cbc992f53322f99 |

产物在 `research/prompts/m3-formal-code-b17-r1-opus-model/out/`（33 份日志是攻方这一次跑出的原样输出，`counts.txt` 是从它们数出的计数）。

## 各格判定一览

| 格 | 判定 | 量过 / 推的 | 一句话 |
|---|---|---|---|
| K2-1 合法历史误红：重挂多次之后的第一次建树 | **打中** | 量过（副本） | 只做过 mkfs 的池可写挂载 10 次（每次只写行、暖机）再写第一个文件：那一版认出、数据落 U（50176），I-5.5 判红，别的不变量一条不红；默认环与 128 MiB 环同形，k = 10..20 全红（22 / 40 格） |
| K2-2 同上，同一次挂载里推零单元发布 | **打中** | 量过（副本） | 实例 2 里推 z ≥ 22 次零单元发布再写第一个文件：19 / 41 格红在 I-5.5 一条上 |
| K2-3 非默认环 | 不另红 | 量过（副本） | 128 MiB 环 k = 1..9 成立；k ≥ 10 与默认环同红（K2-1 那一形） |
| K2-4 零长文件 | 不另红 | 量过（副本） | `publish_first_file` 零长也写一个数据单元，k = 1、2 成立；k = 10 同 K2-1 红 |
| K2-5 崩溃落在第一个文件版本中途再建一次 | 没跑（状态数超限），推的不红 | 推的 | 那段发布的闭式全量 16 777 223 态，超过约 10⁶ 的单段上限，按派发约束不跑；推的理由在 K2-5 节 |
| K2-6 两遍读环不一致时 t1 被隔离 | 没打中 | 量过（副本，76 格） | 挂载那一刻读不出的根槽，影子账整次挂载都当没有根，隔离数恒 0；「水位那一遍漏看、隔离那一遍看见」凑不出来 |
| K2-7 坏实现漏判：E171 第 15 行（A2c） | 不漏 | 量过（副本） | 形 A 24 格全红（三条臂都红） |
| K2-8 坏实现漏判：t1 挪两槽 | 不漏 | 推的 + 现有用例 | 现有坏镜像与 GPU 坏流就是这一形；判据是逐盘 `==` |
| K2-9 坏实现漏判：写死 U + 4（H1，攻方自造） | **漏**（且与 K2-1 互为反面） | 量过（副本） | 形 A 24 格 I-5.5 全成立：k ≥ 10 时正确实现被判红、写死 U + 4 的坏实现被判成立 |
| K2-10 不适用那几格过宽 | 没打中 | 推的 | 五种判不了的格在今天的实现写出的合法镜像上都碰不到；判不了不吞违例 |
| K4 8 行变异 | 8 / 8 红 | 量过（副本） | 第 3、4 行与第 2 行是同一处、同一改法，只是点名用例不同；第 1 行点名的第二个测试二进制（`checker_known_bad_images`）在这一行下 69 条全过，红的只在高水位那一条 |
| K1 GPU 与 CPU 同判 | 没找到不同 | 推的（没跑 GPU） | 跳过条件、逐盘比较、容量越界置判不了，逐条对得上 |

## K2：I-5.5 的认法与判据

### K2-1 打中：可写挂载 10 次之后的第一个文件版本，正确实现被判红

**历史**（形 A，`shape_a_remount_count_sweep_then_first_file`；用户决定的那一步「挂几次」放开扫 k = 1..20，两种环长）：

1. mkfs（默认 768 MiB 环，U = 50176；或 128 MiB 环，U = 9216）。
2. 可写挂载 k 次，每次挂上就丢（每次挂载只做写行与暖机，不写文件）。实例 2 的写行把实例表 COW 走，U、U + 1 记成已释放、释放代 3（形 B 产物 `research/prompts/m3-formal-code-b17-r1-opus-model/out/runB1.log:28`：「low_allocation_records_dev0=["50176+2R@3", "50178+1R@26", "50180+2A@26"]」）。
3. 第 k 次挂载上接着现行那一版（树表 0 条）发第一个文件版本，从 mkfs 水位 65536 发树。

每一步都是用户可做的动作：可写挂载、写第一个文件是产品入口；第 2 步回收 U、U + 1 是实现按可再分配谓词做的（「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」（`crates/singlefs-core/src/allocator.rs:1799`）），第 3 步落点是实现按 D3 已定项 8 第 1 条取的最低空槽对（「let mut candidate = self.unit_area_start.slot().0;」（`crates/singlefs-core/src/allocator.rs:742`），「&& self.is_free(SlotNumber(candidate))」（`crates/singlefs-core/src/allocator.rs:749`）），条款原句「起点槽号最小、起点 32768 对齐、跨度内两槽都空」（`.claude/kb/decisions/03-空间分配.md:163`）。

**读数**（原样，`out/runA1.log`）：

- k = 9：「smallest_slot_per_device=[(0, 50180), (1, 50180)] i55=holds red=[]」（`research/prompts/m3-formal-code-b17-r1-opus-model/out/runA1.log:15`）
- k = 10：「smallest_slot_per_device=[(0, 50176), (1, 50176)] i55=VIOLATED(根（实例 10、txg 29）（从 mkfs 水位第一次建树的那一版）：盘 0 上落点最小的数据单元在槽 50176，不是单元区起始槽号 50176 + 4 = 50180) red=["I-5.5"]」（`research/prompts/m3-formal-code-b17-r1-opus-model/out/runA1.log:17`）
- 128 MiB 环 k = 10：「smallest_slot_per_device=[(0, 9216), (1, 9216)]」（`research/prompts/m3-formal-code-b17-r1-opus-model/out/runA1.log:37`）

计数：40 格（两种环 × k = 1..20），I-5.5 成立 18、判红 22（k ≥ 10 全红），除 I-5.5 之外红的格 0：「runA1 cells=40 holds=18 na=0 violated=22 other_red=0 no_judge_line=0」（`research/prompts/m3-formal-code-b17-r1-opus-model/out/counts.txt:1`；计数命令同 `tools/rerun.sh` 的 `count`）。

翻转点落在 k = 9 → 10 之间，与回收谓词逐格对得上（推的算术）：第一个文件版本 txg = 29 时，写它之前环里最旧有效根是 29 − 24 = 5 ≥ 3，U、U + 1 已回收；k = 9（txg 26）时 26 − 24 = 2 < 3，还没回收。

### K2-2 打中：同一次挂载里推零单元发布

形 B（`shape_b_zero_unit_publish_count_sweep_then_first_file`；z 放开扫 0..40）：mkfs → 实例 1 → 实例 2 → 在实例 2 里连推 z 次零单元发布（检查档那条 `crash_points_tree_identifier_watermark_orphans` 把它当挂载之后的用户动作）→ 第一个文件版本。

- z = 21：「smallest_slot_per_device=[(0, 50180), (1, 50180)] low_allocation_records_dev0=["50176+2R@3", "50178+1R@26", "50180+2A@26"] i55=holds red=[]」（`research/prompts/m3-formal-code-b17-r1-opus-model/out/runB1.log:28`）
- z = 22：「smallest_slot_per_device=[(0, 50176), (1, 50176)] low_allocation_records_dev0=["50176+2A@27", "50178+1R@27"]」（`research/prompts/m3-formal-code-b17-r1-opus-model/out/runB1.log:29`），同一行 I-5.5 判红、`red=["I-5.5"]`。

计数：「runB1 cells=41 holds=22 na=0 violated=19 other_red=0 no_judge_line=0」（`research/prompts/m3-formal-code-b17-r1-opus-model/out/counts.txt:2`）。

### K2-1、K2-2 打中之后的四句

1. **分不分辨臂**：不分辨代码与条款。代码照 I-5.5 那一行字面判（「let expected_slot = unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START;」（`crates/singlefs-checker/src/walk.rs:4139`）），代码注释与条款共用同一个前提「mkfs 的实例表占 U、U + 1，树表占 U + 2，第一个 32768 对齐的空槽对从 U + 4 起」（`crates/singlefs-checker/src/walk.rs:3371`，条款原句同字在 `.claude/kb/decisions/03-空间分配.md:171`）。这个前提在「实例表被 COW 走、U、U + 1 被回收」之后不成立。按 evidence-discipline「打中不分辨臂」办：病根在共用前提里，先另立一笔账修前提，不拿这一格判代码对错。
2. **被判的系统看不看得到判别它的东西**：部分看得到。checker 在那一版的分配记录里看得到 U、U + 1 是「已释放 R@3」还是被这一版自己的数据占了（K2-2 两行对比）；但「R@3 此刻回收没回收」取决于写这一版那一刻的回收门槛 max(F_生效, 环里最旧有效根)，盘上只留下部分（这一版根里的 F、txg、根环槽数），被抛弃根与挂载那一刻读不出的槽 checker 看不到。K2-9 的 H1 就落在这条缝里（见下）。
3. **满足的是判据字面哪一句**：I-5.5 那一行的认法 ①②（树表非空、extent 树 65536、每条诞生 txg 等于这一版 txg）全满足，「每块盘上落点槽号最小的那一个，槽号等于 U + 4」（`.claude/kb/invariants.md:199`）不成立。射程句「这一类含新池新建文件，也含 mkfs 之后重挂、实例 1、2、3 从 mkfs 水位写的第一个文件」（`.claude/kb/invariants.md:199`）只列到实例 3，但「这一类」由 ①② 定义，实例 10 字面在类里。D3 那一句「偏离它写出的这一类镜像不合法」（`.claude/kb/decisions/03-空间分配.md:171`）因此把一张按同一项第 1 条取落点的镜像判成不合法：**D3 已定项 8 第 1 条与它自己的射程句在这段历史上不能同真**。
4. **跑前条款给的改法还中不中**：正文第五节只写「打中而坐实 ⇒ 改法并进下一批实现员」，没列具体改法，没有可逐个核的。下面「改法表」里两条是攻方自己提的，只在攻方副本上量过、被攻过零轮。

### K2-3 非默认环、K2-4 零长文件：不另红

- 128 MiB 环：U 现算为 9216（单元区起点必须落在 64 槽聚簇段边界上，「if !slot.0.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {」（`crates/singlefs-core/src/allocator.rs:221`），所以 U 恒偶、U + 4 恒 32768 对齐）；k = 1..9 成立，k ≥ 10 与默认环同红（上面 `runA1.log:37`）。
- 零长：`publish_first_file` 零长也写一个数据单元（「data_units=1」），k = 1、2 成立，k = 10 同 K2-1 红：「name=shape_e length=0 k=10 data_units=1 expected=50180」（`research/prompts/m3-formal-code-b17-r1-opus-model/out/runE.log:8`）。长度 32768、32769、100000 这条入口拒（「ContentExceedsDataUnit」，9 格），多数据单元的第一个文件版本这一轮没造出来。

### K2-5 崩溃落在第一个文件版本中途再建一次：没跑，推的不红

形 D（`shape_d_crash_inside_the_first_file_version_then_build_again`，攻方自己造的流：mkfs → 取号 → 暖机 → 第一个文件版本，录制写流按 harness 的切段函数切段，逐段展开段内持久写的每个子集）只数了状态数：「name=shape_d_count writes=29 segment_sizes=[24, 2, 1, 2] closed_form_state_count=16777223」（`research/prompts/m3-formal-code-b17-r1-opus-model/out/counts.txt:21`）。第 0 段 24 次单元写（12 个单元 × 两盘），单段就超过约 10⁶，按派发约束整段不跑，也不抽样（崩溃状态不在可缩之列）；原型里写了超过 10⁶ 就只报数的闸。

推的（没量）：那 24 次单元写都在记录之前，任一子集落盘都只留孤儿；挂载择到暖机根，分配器从那一版的分配记录重建，孤儿不在账里，U + 4 照样是最低空槽对，再建一次落 U + 4、成立。记录落了而根没落的态，那一版由记录施加出来，树表、数据指针与发布时相同，落点仍是 U + 4。所以这一形不另红；但它与 K2-1 叠起来（崩溃之后再挂 ≥ 10 次才写第一个文件）照样红，病根同 K2-1。checker 档 `crash_points_tree_identifier_watermark_orphans` 那条流挂载之后只做六种动作、最多重挂一次，碰不到 K2-1 的翻转点（推的，没跑）。

### K2-6 两遍读环不一致、t1 被隔离：没打中

形 C（`shape_c_transient_read_masks_then_first_file`）：实例 1 发第一个文件版本 (1, 3)（数据 U + 4），崩在根槽 FUA 之后、系统配置轮换之前；重挂时 txg 3 的根槽按掩码 S 瞬时读坏、那次发布的两份 journal 记录按掩码 J 读坏，挂上之后撤掉故障、接着发第一个文件版本。两批：① S 只放开第 1 次读（坏 / 不坏两种）× J ∈ {0、0x1、0x3、全坏}，8 格（`out/runC0.log`）；② S 取「前 16 次读里恰好坏一次」16 种、「恰好好一次」16 种、全好、全坏，共 34 种 × J ∈ {0x1、全坏}，68 格（`out/runC1-*.log`；一次挂载里这一槽被读 14–20 次）。共 76 格，带隔离字段的 37 行隔离数全是 0：「runC isolated_nonzero_lines=0」（`research/prompts/m3-formal-code-b17-r1-opus-model/out/counts.txt:17`），I-5.5 一格都没红（counts.txt 第 14–16 行 violated=0）。

机理（推的，与读数相容）：根槽在读阶段读坏，这次挂载就把它当没有根，后面几遍读再读得出也不进影子账；读阶段读得出，它就被择中或经记录施加，不是被抛弃根。「水位那一遍漏看它、影子账那一遍把它当被抛弃根隔离 U + 4」要两件同时成立，这 76 格里一格都没出现。实现的注释承认几遍读之间看见的根可以不一样（「影子账、回收门槛与分配器那张根环表读的是同一遍根环（每槽读一次）」（`crates/singlefs-core/src/mount.rs:1782`）），但那一遍之外的分歧落在水位上，不落在隔离上。

旁见（不归这条腿判）：S = 「除第 2 次读之外全坏」那一格 I-7.4 红、I-5.5 成立（`out/runC1-ffffffffffffffff.log:12`），是根与记录都没被水位那一遍看见、数据槽被复用的那一形，与「根与它那条记录都读不出的那一格仍然盖不住。」（`crates/singlefs-core/src/recovery.rs:1337`）同族（C554 / C342 一类），7 行 I-7.4 红都在这一族里。

### K2-7、K2-8、K2-9 坏实现漏不漏

- **A2c（E171 第 15 行那一形，变异表第 2 行的改法）**：形 A 24 格（k = 1..12 × 两种环）全红，三条臂都是 24 / 24（`out/counts.txt` 第 7–9 行）。落点是 U + 64 起的那几个槽（「smallest_slot_per_device=[(0, 50240)」等，`out/runA-a2c-orig.log`）。不漏。
- **t1 挪两槽**：现有的 `checker_known_bad_images` 坏镜像与 GPU 对拍坏流就是这一形，判据是逐盘 `==`（「judgements.judge(invariant, smallest_slot == expected_slot, || {」（`crates/singlefs-checker/src/walk.rs:4224`）），挪任何非零槽数都红。推的，没另造。
- **H1（攻方自造：用户数据落点从 U + 4 起扫，不看 U、U + 1 回收没回收）**：形 A 24 格 I-5.5 **全成立**：「runA-h1-orig cells=24 holds=24 na=0 violated=0 other_red=0 no_judge_line=0」（`research/prompts/m3-formal-code-b17-r1-opus-model/out/counts.txt:10`）。k ≥ 10 时它违反 D3 已定项 8 第 1 条（有更低的空槽对不用），I-5.5 判它成立；同一格上正确实现被判红（K2-1）。**在 k ≥ 10 上 I-5.5 的判定与第 1 条正好相反**。
- H2（U、U + 1 一律当空，释放了没回收也复用）在 k = 1 就撞实现自己的断言 panic（`out/runA-h2-orig.log:9`），不是安静的坏实现，不计。

### K2-10 不适用那几格是不是过宽：没打中（推的）

五种判不了（树表读不出、条目宽不到 200、种类 1 不止一条、extent 树读不全、某块盘上没有数据单元）在今天的实现写出的合法镜像上碰不到：树表条目定宽 200、只有一棵 extent 树、两盘镜像。前两种与第四种只在单元读坏、撕裂的崩溃态上出现，那时 I-1.x / I-2.x 先红。判不了也不吞违例：报告时先看违例，「let verdict = if let Some(detail) = self.first_violation.get(invariant) {」（`crates/singlefs-checker/src/image.rs:256`），后看「判不了」（第 258 行）。真正的过宽不在这五格，在 K2-9：类里那一版按 U + 4 判，而不是按「那一刻真正的最低空槽对」判。

## 改法表（攻方自己提的，只在攻方副本上量过、被攻过零轮）

两条都是 checker 一侧的改法（`walk-fix-arms.patch`，副本里用 `OPUS_I55_ARM` 选），都要先改条款：I-5.5 那一行与 D3 已定项 8 射程句由用户 2026-10-04 定，改 checker 不改条款就是 checker 替条款定。

- **f1（收窄认法）**：认法加 ③「这一版的分配记录里 U、U + 1 两槽都还被占着（已分配且分配代早于这一版，或已释放）」，不满足就不判。
- **f2（落点现算）**：每块盘上该在的槽不写死 U + 4，按这一版的分配记录现算：从 U 起第一个 32768 对齐、两槽都不被「已释放、或分配代早于这一版」的记录罩着的槽对。

| 格 | 入库那一版 | f1 | f2 | 依据 |
|---|---|---|---|---|
| K2-1 形 A 合法历史（24 格，k = 1..12） | 6 格误红（k = 10..12；k = 1..20 的 40 格里 22 格） | 0 红（6 格不判） | 0 红 | 量过：`out/counts.txt` 第 1、3、4 行 |
| K2-2 形 B 合法历史（31 格，z = 0..30） | 9 格误红（41 格里 19 格） | 0 红（9 格不判） | 0 红 | 量过：`out/counts.txt` 第 2、5、6 行 |
| 现有 `pool_checker_judges_…` 五条 | 绿 | 绿 | 绿 | 量过：「test result: ok. 5 passed; 0 failed」（`research/prompts/m3-formal-code-b17-r1-opus-model/out/arm-f1-pool_checker.log:13`），f2 同（`out/arm-f2-pool_checker.log` 第 12 行） |
| 现有 I-5.5 坏镜像（`the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target`） | 红得出 | 红得出 | 红得出 | 量过：两条臂下这条用例照过（`out/arm-f1-checker_know.log` 第 9 行、`out/arm-f2-checker_know.log` 第 8 行） |
| A2c 坏实现（形 A 24 格） | 24 红 | 24 红 | 24 红 | 量过：`out/counts.txt` 第 7–9 行 |
| H1 坏实现（写死从 U + 4 起扫，形 A 24 格） | 0 红（漏） | 0 红（漏） | 0 红（漏） | 量过：`out/counts.txt` 第 10–12 行 |
| 「U、U + 1 释放了还没到回收门槛就被复用」那一形 | 红（≠ U + 4） | 不判（那一版的记录里 U、U + 1 已是这一版自己的已分配） | 成立（同理） | 推的：H2 自崩、没造出安静的版本；这一形要回收门槛之前复用，I-7.4 应当先红（推的，没量） |

f1、f2 都修掉 K2-1、K2-2 的误红，都不丢 A2c，都抓不到 H1。H1 漏的原因同四句里的第 2 句：f2 把「已释放」一律当占着，而 H1 那一格 U、U + 1 在记录里仍是「R@3」（已回收但没被重新分配，记录不变），f2 读不出它已过回收门槛。要抓 H1，判据得复算写这一版那一刻的回收门槛 max(F_生效, 环里最旧有效根)（「f3」，推的、没实现）：这一版根里的 F 与 txg、根环槽数盘上都有，被抛弃根与挂载那一刻读不出的槽盘上没有，f3 只能近似。三条之外还有一条出路是不改 checker、改条款：把 I-5.5 的类收回到「mkfs 的实例表仍在 U、U + 1」或「新池新建文件那一条流」，这要用户重定 2026-10-04 那一项（推的，没量）。

## K4：8 行变异各自证不证得到它点名的那一处（量过，攻方副本）

逐行：打变异（第 3 列原串在被改文件里恰好命中 1 次才打）、照第 5 列跑 harness 档测试、从冻结树还原（`run_row.sh`）。

| 行 | 改哪一处 | 点名用例 | 结果 | 引 |
|---|---|---|---|---|
| 1 | transaction.rs 头 ID 非 0 一律写 65537（M1） | 高水位那一条 | 红；第 5 列另列的 `checker_known_bad_images` 69 条全过，红只靠高水位那一条 | 「test a_first_file_version_issuing_its_trees_from_the_ring_watermark_after_a_recovery_keeps_the_head_identifier_relation ... FAILED」（`research/prompts/m3-formal-code-b17-r1-opus-model/out/row1.log:87`） |
| 2 | allocator.rs 用户数据从 U + 64 起扫（A2c） | 默认环新池 | 红，落 50240 | `out/row2.log` 第 7 行 |
| 3 | 同第 2 行（同一原串、同一改法） | 128 MiB 环新池 | 红，落 9280 | `out/row3.log` 第 8 行；「盘 0 上落点最小的数据单元在槽 9280，不是单元区起始槽号 9216 + 4 = 9220」（`research/prompts/m3-formal-code-b17-r1-opus-model/out/row3.log:27`） |
| 4 | 同第 2 行 | 实例 2 第一个文件 | 红，落 50304 | `out/row4.log` 第 10 行；「根（实例 2、txg 5）」（`research/prompts/m3-formal-code-b17-r1-opus-model/out/row4.log:34`） |
| 5 | walk.rs 认法 ② 后面并一个恒真（不看诞生 txg） | 覆写一圈之后不判 | 红 | `out/row5.log` 第 10 行 |
| 6 | walk.rs I-5.5 判成恒成立 | 已知坏镜像总表 | 红 | 「I-5.5 那份坏镜像应当判违例，实际 Holds」（`research/prompts/m3-formal-code-b17-r1-opus-model/out/row6.log:14`） |
| 7 | walk.rs 种类 1/9/10 不比 inode 树 ID | 指针 / journal / 代坏镜像表 | 红，I-9.27 那份坏镜像不红 | `out/row7.log` 第 7 行 |
| 8 | walk.rs 种类 1/9/10 拿自己的树 ID 比 | 高水位那一条 | 红；同文件 5 条全红（每张带文件的镜像都红） | `out/row8.log` 第 11 行 |

判别力上的两点（量过之外是推的）：

- 第 2、3、4 行是同一个变异体（第 3、4 列逐字相同），三行证的是「同一处改动被三条用例各自抓到」，不是三处改动；三条用例在这一个变异体下都红（`out/row3.log` 第 19、27、34 行三处 Violated）。登记成三行没有错，但计数时别当成三个独立的判别力证据。
- 8 行里没有一行碰得到 K2-1 那一格（k ≥ 10）：形 A、形 B 不在任何点名用例里，所以 K2-1 的误红在现有用例与变异表下看不见（推的：harness 现有用例都在 k ≤ 3 上）。
- 正文第三节 K4 提到的 `crates/mutations.tsv` 第 1605、1809 行（17-4 那一条用例的旧变异）这一轮没复跑（不在 diff 抄的 8 行里，没读那两行），写进「没做什么」。

## K1（兼）：GPU 与 CPU 同不同判（推的，GPU 一行没跑）

- I-9.27 读不出头 ID 时两边都整张表不判：CPU「.all(|entry| entry.len() >= TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8);」（`crates/singlefs-checker/src/walk.rs:3494`），GPU「if (n.entry_width < TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1466`）。一个节点里条目定宽，两句等价。
- I-5.5 跳过条件逐条对得上：GPU「if (!tt.ok || tt.entry_count == 0u || tt.entry_width < 200u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1784`）、「if (extent_entry_count != 1u || !every_birth_is_this_version」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1800`）对 CPU 的读不出 / 宽不到 200 / 种类 1 不止一条（`crates/singlefs-checker/src/walk.rs:4178`）/ 认法 ②（`crates/singlefs-checker/src/walk.rs:4189`）；逐盘比较「if (r_has_data_unit_on_device[device_index] == 1u && !eq64(r_smallest_data_unit_slot_on_device[device_index], expected_slot)) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1820`）与 CPU 第 4224 行同义（没有数据单元的盘两边都不置违例）。
- extent 树走读越过内核容量时 GPU 置判不了、交 CPU：「undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (19u << 8u);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:90`），不会悄悄当成「读不全、不判」而与 CPU 分叉。
- I-5.5 那一遍把 `r_noting` 置假之后不还原（`crash_judge_checker_scan.wgsl` 第 1810 行），后面每一遍用它之前都自己设（如「r_noting = every;」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:628`）），不串到别的判据。
- K2-1 在 GPU 上同样误红：内核用的也是写死的「const FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START: u32 = 4u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:161`）。改法 f1、f2 要两边一起改，GPU 那一边还得把这一版的分配记录树读进扫描内核（推的工作量，没估）。
- I-9.27 在攻方跑的全部 378 格里一格没红（有 `red=[` 字段的结果行 378 行，含 `"I-9.27"` 的日志 0 份），含形 C 那几格树 ID 65549 的历史。这是对 K1「合法历史不误红」的一点旁证，不是攻 I-9.27 的结果。

## 没打中的形状

| 形状 | 取样 | 结果 |
|---|---|---|
| 非默认环 | 默认、128 MiB；k = 1..20 | 只在 K2-1 那一形上红 |
| 零长、1 字节第一个文件 | k ∈ {1, 2, 10} | 同上 |
| 两遍读环不一致、t1 被隔离 | 76 格（根槽读坏掩码 × 记录读坏掩码） | 隔离数恒 0，I-5.5 不红 |
| 崩溃落在第一个文件版本中途再建一次 | 没跑（单段 16 777 216 + 7 态） | 推的不红 |
| A2c 漏判 | 形 A 24 格 × 三条臂 | 不漏 |
| 不适用过宽 | 读代码 | 推的不过宽 |
| GPU 与 CPU 分叉 | 读 WGSL | 推的不分叉 |

## 这条腿自己的限度

- 全部数都在攻方副本上（冻结树拷贝、harness 档）；checker 档、GPU、naga 没跑。
- `tools/rerun.sh` 没有整份连跑过一遍：里面每一步都在副本上单独跑过（命令与输出在 `out/`），拼成一份脚本之后没再跑。
- 形 C 的读坏掩码只放开了「恰好坏一次 / 恰好好一次」与前 1 次读的全部组合，没扫一次挂载里 14–20 次读的全部 2^n 组合；也没放开数据单元那两份的读坏。
- 形 D 按单段 ≥ 10⁶ 的约束没跑，K2-5 是推的。
- 形 A、B 的用户动作只放开了「挂几次」「推几次零单元发布」；挂载之间夹覆盖写、抬 F、管理员回退的组合没扫（那些一旦有了文件就不再是这一类，推的不影响 K2-1）。
- f1、f2 是攻方自己提的，被攻过零轮；f3 只是推的方向。
- 多于两块盘的池、io_min 8192 的几何没造。

## 要主 agent 跑的命令

1. 复跑攻方的量（入库装置上重做之前先核腿的数）：工作目录仓根，`bash research/prompts/m3-formal-code-b17-r1-opus-model/tools/rerun.sh /tmp/claude-1000/<新目录> 8`，内存上限在脚本里（原型 2G、变异行 3G），线程 8；看 `<新目录>/out/summary.txt`，与 `out/counts.txt` 第 1–21 行逐项比（rerun 的 summary 多 H1 三行与 8 行变异摘要）。
2. 在入库装置上坐实 K2-1：把形 A 那一段（mkfs → 可写挂载 10 次 → 第一个文件版本 → 池级 checker）做成入库用例，判据「I-5.5 判红、别的不红」，再按判决定是改条款还是改 checker。

## 没做什么

- 没跑任何 checker 档测试、GPU 对拍、WGSL 校验、门禁阶段、crates 变异整表。
- 没复跑 `crates/mutations.tsv` 第 1605、1809 行（17-4 那一条用例的旧变异）。
- 没攻 K3（GPU 覆盖表断言），那一格分给正推腿。
- 没读禁读清单里的文件。
- 草稿目录 `/tmp/claude-1000/m3-formal-code-b17-r1-opus/` 只剩日志与小脚本（约 296 KiB）；仓副本与它的编译目录（`/tmp/claude-1000/m3-formal-code-b17-r1-opus/repo`，1.5 GiB）交回前已删。

## cite-check 末行

`python3 research/scripts/cite-check.py research/prompts/m3-formal-code-b17-r1-opus-output.md --root /tmp/claude-1000/m3-formal-code-b17-r1/tree --root . --background research/prompts/_m3-formal-code-b17-r1-background.md`：

```
  ✓ 核了 39 处引文，对上 39 处，没判 0 处（逐处列在上面）
```
