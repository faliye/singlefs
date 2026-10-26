# m3-close-c506-r2 正推腿（Sonnet）报告（2026-10-23）

被判形态：`research/prompts/_m3-close-c506-r2-body.md` 第三节。立场：从 kb 条款与 `crates/` 今天的代码正推「每一句该不该这样写」，再与被判的句子比。方案按 `crates/` 今天的实现来谈。引 kb 条款的行号都是在 kb 文件里现查的（`grep -nF`），不取自背景材料。

## 复跑与模型

模型是纯 Python 标准库，不读 `crates/`、不编译；模型里的式子从条款与 layout 字节表写，入库实现的判法（mkfs 的六段两两比）读过代码后照式子另写一份，两边相等才记「相等」。

```text
$ cd /home/fy5090/code/singlefs
$ SINGLEFS_RUN_SCOPE='种类=model; 跑什么=m3-close-c506-r2 正推腿 Python 纯算模型复跑; 范围=模型目录与草稿目录, 不碰 crates; 为什么=核模型输出与落盘 out/model.out 逐行相同' \
  bash scripts/hosts/run-with-memory-cap.sh 1G bash research/prompts/m3-close-c506-r2-sonnet-model/tools/rerun.sh --repo . --frozen research/prompts/m3-close-c506-r2-snapshot --work <新的草稿目录> --model research/prompts/m3-close-c506-r2-sonnet-model
name=rerun_compare same=1 lines=51 sha256=d6e9cb44e300d7bdf165a87f5dc758f37b3410040cb135e5d099464dd41f58df
```

```text
0458636bef3cc572ee09d634bd512db7e79c02cdaf959ca42260fd2fd7a53b9f  research/prompts/m3-close-c506-r2-sonnet-model/src/geometry_model.py
d6e9cb44e300d7bdf165a87f5dc758f37b3410040cb135e5d099464dd41f58df  research/prompts/m3-close-c506-r2-sonnet-model/out/model.out
2e1409685336b42fa425a0625e3194e48190a23acecd5940afda6c6a8d5cc651  research/prompts/m3-close-c506-r2-sonnet-model/tools/rerun.sh
```

模型会红的证明：把判的右边改成 `2 * stride` 再跑，自带的 `selftest` 断言（S = 8、槽距 384 KiB + 512 必须拒）当场失败，退出 1（草稿目录 `mut/broken.py`，已复跑）。模型里另有四条反例控制（不加这一判、右边换成 6 MiB、右边取盘上字段、`<` 代替 `<=`），输出行见 W1。

## 判定一览（先给表，推导在后）

| 格 | 判定 | 一句话 |
|---|---|---|
| W1 同盘相叠的形状 | 条款「S × 槽距 ≤ P × chunk」罩得住，但右边必须是第一版常量；今天 checker 读盘上字段，句子没说 | 冲突（读者之间）+ 规则没说 |
| W1 错拒 | 对入库 mkfs 今天写得出的几何错拒 0 个；对盘上并不相叠的几何多拒 8052 个（模型枚举），入库 mkfs 本来就拒它们 | 一致（对 mkfs）；读者侧收严 |
| W1 落点 | core `system_configuration_values_this_reader_accepts`，checker `geometry_of`，GPU 两个解槽函数；mkfs 一行不用加（已等价） | 一致 |
| W1 挂哪条不变量 | 挂 I-7.13（整池拒），不挂 I-7.17 | 一致 |
| W2 min 公式与「不取两条边」 | 16 取不到；4 在槽距自由取值时能取到（629248..786432），按 D2 已定项 19 取槽距且过对齐判时只会写 6 | 句子的条件没写全 |
| W2 按 S = 8 写死的地方 | E50 不依赖 8；D28 环窗口按 S 现算；错的是 mkfs 编译期断言只罩常量 8，以及 `crates/singlefs-format/src/lib.rs` 与 `system_configuration.rs` 注释里「mkfs 写 8」的几句 | 部分会错 |
| W3 改字节造盘的等价 | 对 I-7.17 的判定成立，理由是读代码（mkfs 输出对 S 的依赖只有 4 处），不是三对几何；可达性是「只照 kb 的写者」不是「只照第 3 句判盘长」 | 成立，需改一处措辞 |
| W4 3 × 16 进格式 | 与已定项 25 字面不冲突，措辞要避开「焊死」并引 E92 | 一致（轻微） |
| W4 冻结前随之抬 | 自洽；「抬上界先抬它」也要带「冻结前」 | 一致 |
| W4 GPU 根表 48 | 不进格式条款；p 一字节（S ≤ 85）进 | 一致 |

## W1：「S × 槽距 ≤ P × chunk」这一判本身

### 1.1 今天的条款与代码里这一判在哪

kb 里没有一句要求区域之间不相叠。条款只要求 mkfs 判盘长：`.claude/kb/decisions/22-单元原子性怎么合成.md:350`：「越界拒绝 mkfs」；槽距上界只由系统配置槽 1 定：`.claude/kb/decisions/22-单元原子性怎么合成.md:74`：「固定结构槽距 ∈ [4096, 1 MiB − 4096]（上界是槽 1 整槽落在根环基址之前」。区域起点：`.claude/kb/layout/01-first-txn.md:44`：「区域 r 在 基址 + r × P × chunk，基址 = 系统配置」。

入库 mkfs 已经判了，只是条款没写：`crates/singlefs-core/src/make_filesystem.rs:173`：「固定结构槽距的上界就由它定：两槽要落在根环基址之前、区域长 S × 槽距不能越过区域间距 P × chunk。」判法是 `fixed_structure_extents`（`crates/singlefs-core/src/make_filesystem.rs:387`）列六段，区域长取 `region_length_in_bytes`（`crates/singlefs-core/src/root_ring.rs:140`，式子 `crates/singlefs-core/src/root_ring.rs:144`：「slots_per_region.count() * u64::from(fixed_structure_slot_spacing)」），再两两比（`crates/singlefs-core/src/make_filesystem.rs:532`：「if first_in_layout_order.overlaps(*second_in_layout_order) {」），不分盘。

读者两侧今天只判槽距区间与 S 区间，不判乘积（主 agent 观测；我复核：`system_configuration_values_this_reader_accepts` 的槽距判在 `crates/singlefs-core/src/recovery.rs:1117`：「if fixed_structure_slot_spacing < FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES」，S 判在 `crates/singlefs-checker/src/image.rs:734`：「if !(ROOT_RING_SLOTS_PER_REGION_MINIMUM..=ROOT_RING_SLOTS_PER_REGION_MAXIMUM)」，两处之后都没有乘积）。

### 1.2 这一判在三个读者与 mkfs 里各落在哪个函数

| 处 | 函数（文件:行） | 加什么 | 注意 |
|---|---|---|---|
| core | `system_configuration_values_this_reader_accepts`（`crates/singlefs-core/src/recovery.rs:1095`） | 在槽距判（1117）与根环起点判（1139）、区域数判（1157）之后加一格；新成员加在 `SystemConfigurationValueOutsideWhatThisReaderAccepts`（成员 `crates/singlefs-core/src/recovery.rs:979` 那一排） | S 在更早的 `parse_slot` 就进了带不变量的类型（`crates/singlefs-core/src/system_configuration.rs:670`），乘积判不能放进 `RootRingSlotsPerRegion`（它看不见槽距） |
| 池级 checker | `geometry_of`（`crates/singlefs-checker/src/image.rs:717`） | 槽距判（`crates/singlefs-checker/src/image.rs:750`）之后加一格；新 `Verdict` 成员要同时登进 `value_refused_by_this_reader`（`crates/singlefs-checker/src/image.rs:961`，穷举写全，漏登编译不过） | I-7.13 那一行写「共八项」（`crates/singlefs-checker/invariants/02-根环.md:20`：「共八项」），要改成九项 |
| GPU 判器（恢复一侧） | `parse_system_configuration`（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:943`） | 槽距判（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:980`）之后加一格，拒收码顺延一个 | WGSL 是 u32：S × 槽距在槽距还没被区间判住时会溢出，写成 `spacing <= 3145728u / S` |
| GPU 判器（checker 一侧） | `checker_read_system_configuration_slot`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:87`） | 槽距判（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:126`：「var spacing_holds = spacing >= 4096u;」）之后 | 同上 |
| mkfs | `check_geometry`（`crates/singlefs-core/src/make_filesystem.rs:448`）已经判 | 一行代码不用加；加的是 kb 条款与注释 | 见 1.4 |

四处次序要一致：两个读者的既有测试把五格「core 与 checker 报哪个成员」钉成表（`crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs`，D22 已定项 16 引它；该项自己写了根环起点写 0 那一格两边报的成员本来就不同），新格要放在同一位置：core、checker、GPU 两个函数同位，否则同一张坏盘上报的成员各不相同，那张表也要加一格。

### 1.3 它罩不罩得住第一轮打中的那一类

模型：区域 r 在盘 `[0, 1, 0][r]` 上（`.claude/kb/decisions/22-单元原子性怎么合成.md:351`：「根环三个区域的设备归属第一版写死 0 / 1 / 0」），区域长 S × 槽距，系统配置槽 0、槽 1，journal 环 16 MiB 起（`crates/singlefs-format/src/lib.rs:261`：「pub const JOURNAL_RING_START_SLOT: u64 = 1024;」）。枚举 S ∈ 4..16 × 槽距（512 的整数倍，读者收的整个区间 4096..1044480）共 26429 个形状。

```text
name=w1_counts shapes=26429 rule_accept=9411 mkfs_today_accept=9411 physical_clean_full=17463 physical_clean_byte_exact=18733
name=w1_cover rule_accepts_but_physically_overlaps=0 rule_accepts_but_byte_exact_overlaps=0
name=w1_equals_mkfs_today rule_minus_mkfs=0 mkfs_minus_rule=0
name=w1_control_no_rule accepted=26429 physically_overlapping=8966 includes_16_512KiB=True
name=w3_slot_alias_vs_rule pbs=512 shapes_with_a_root_slot_aliased=12 of_them_accepted_by_rule=0 extent_overlap_shapes=8966 extent_overlap_without_slot_alias=8954 alias_spacings_not_power_of_two=[786432]
```

逐形状：

- 区域 2 末端越过 journal 环：区域 2 起点 7 MiB，到 16 MiB 有 9 MiB；这一判把区域长压到 3 MiB，末端不超过 10 MiB，罩住，且是被「区域间距」顺带罩住的（`w1_largest_S` 行：槽距 1044480 时 S ≤ 9 才不撞 journal，而 S ≥ 4 在这个槽距上已被这一判拒）。**前提是右边取第一版常量 3 MiB，见 1.5。**
- 区域与系统配置槽相叠：槽 1 的末端不越过根环基址早已由槽距上界判住（core `crates/singlefs-core/src/recovery.rs:1117` 起，checker `crates/singlefs-checker/src/image.rs:750`，GPU 两处各一），这一判不增不减。
- 槽距不是 2 的幂：这一判是整数不等式，与 2 的幂无关。真有根槽字节相叠的形状只有 12 个（`pbs=512`），其中非 2 的幂槽距有一个：768 KiB、S ≥ 9（6 MiB ÷ 768 KiB = 8 = 区域 0 与区域 2 的起点差），这 12 个全被拒。
- 同盘区域相叠但根槽并不撞的形状：8954 个（`extent_overlap_without_slot_alias`）。这些形状里区域长 S × 槽距相叠的只是槽与槽之间的空隙字节，根槽本身不撞；这一判把它们也拒了，因为判的是区域长，与入库 mkfs 清零的区间一致（`crates/singlefs-core/src/make_filesystem.rs:671`：「device.write_zeroes_at(region_start(region_to_clear), root_ring_region_bytes)?;」）。

判定：罩得住第一轮打中的那一类的全部形状，前提见 1.5。**什么现象会推翻它**：模型之外找到一个 S × 槽距 ≤ 3 MiB、同盘上两个根槽字节相交的 (S, 槽距)（`geometry_model.py` 的 `w1_cover` 行不是 0）；或者入库的「区域起点」公式不再是 基址 + r × 3 MiB。

### 1.4 它会不会错拒

- 对入库 mkfs 今天写得出的几何：错拒 0 个（`w1_equals_mkfs_today rule_minus_mkfs=0 mkfs_minus_rule=0`）。理由是代码，不是模型：mkfs 的区域间两两判在区域 r、r + 1 之间等价于 S × 槽距 ≤ 3 MiB，区域 2 对 journal 在这个上界下恒不相叠，槽 1 对区域 0 就是已有的槽距上界。模型没算对齐判与设备大小，所以这一条只在「槽距区间 × S 区间」的 26429 格上成立。
- 对「盘上并不相叠」的几何：多拒 8052 个（`w1_over_rejection_full count=8052 min_region_bytes=3146240 max_region_bytes=6291456`），区域长在 (3, 6] MiB 之间，0 / 1 / 0 归属下区域 0 与区域 2 相距 6 MiB、区域 1 在另一块盘上。第一轮打中的「mkfs 比盘上要的严」就是这一类。这些几何入库 mkfs 今天就拒，所以不会拒掉一个已写出的盘；新增的后果只落在「别的写者写出的盘」上。归属换成 0 / 0 / 1 时 6 MiB 的判会放进 8052 个真相叠的形状（`w1_control_same_disk_rule_on_0_0_1 physically_overlapping=8052`），所以取 3 MiB 这个不分盘的上界是对的：归属不是格式常量以后它依然成立。
- 判 `<` 不判 `≤`：错拒 5 个入库 mkfs 收的首尾相接形状（`w1_control_strict_less_than wrongly_rejects_mkfs_accepted=5`），其中一个是 S = 8、槽距 384 KiB，入库测试钉着它（`crates/singlefs-harness/tests/core_review_geometry_back_chain_and_empty_inode.rs:439`：「fn root_ring_regions_longer_than_the_region_step_are_refused_and_regions_that_just_touch_are_not() {」）。条款要写 `≤`。
- 副作用：加上这一判之后，槽距在 (786432, 1044480] 的盘一个也建不出、读不了（S ≥ 4 时 4 × 786944 已超 3 MiB，`w2_min_over_free_spacing` 行 `spacing_giving_below_4=786944..1044480`）。已定项 2 那一句仍写槽距 ∈ [4096, 1 MiB − 4096]（`.claude/kb/decisions/22-单元原子性怎么合成.md:74`），有效上界其实是 768 KiB；两句并存不冲突，但会让读者以为 1 MiB − 4096 是可用的，条款要加一句。

### 1.5 右边的 P × chunk 是常量还是盘上字段：三个读者今天各读各的（句子没说）

- core 不读盘上的 P（偏移 363）与 chunk（偏移 367），只写：写在 `crates/singlefs-core/src/system_configuration.rs:593`：「writer.put_u32(u32::try_from(ROOT_RING_PRIME_STEP).expect("P"));」与 `crates/singlefs-core/src/system_configuration.rs:594`：「writer.put_u32(u32::try_from(ROOT_RING_CHUNK_BYTES).expect("chunk"));」；区域起点取常量：`crates/singlefs-core/src/root_ring.rs:96`：「ROOT_RING_BASE_SLOT * SLOT_BYTES + region * ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES,」。恢复一侧的 GPU 判器同样取常量：`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:113`：「const ROOT_RING_REGION_STRIDE: u32 = 3145728u;     // P × chunk = 3 × 1 MiB」。
- 池级 checker 与 GPU checker 读盘上字段：`crates/singlefs-checker/src/image.rs:773`：「let prime_step = u64::from(read_u32(slot, 363));」、`crates/singlefs-checker/src/image.rs:774`：「let chunk_bytes = u64::from(read_u32(slot, 367));」，取区域起点处 `crates/singlefs-checker/src/image.rs:1306`：「+ region * geometry.prime_step * geometry.chunk_bytes」；GPU：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:173`：「c.prime_step = rd32(base, 363u);」。入库测试明说这是有意的（`crates/singlefs-harness/tests/system_configuration_slots_per_region.rs` 那条「区域步长与区块字节……是盘上读来的数」的注释，只防溢出）。
- 条款没有哪一句要求两个字段等于 3 与 1 MiB：`.claude/kb/decisions/22-单元原子性怎么合成.md:348`：「读者解槽时读这个字段与 journal 环起点字段（偏移 325），与第一版常量（64、1024）不等一律整池拒」只点了基址与 journal 起点；I-7.13 的八项里也没有这两个字段。
- 后果（模型 `w1_control_disk_stride`）：右边取盘上字段时，盘上写 P × chunk = 6 MiB 的池，checker 一侧放行 17463 个形状，其中 8052 个撞 journal 环；写 5 MiB 一个也不撞，写 9 MiB 全撞。而 core 在同一张盘上仍按 3 MiB 读，两个读者读的字节根本不是同一批。

```text
name=w1_control_disk_stride p_times_chunk=3145728 rule_accepts=9411 overlapping_with_that_stride=0 overlapping_journal=0
name=w1_control_disk_stride p_times_chunk=5242880 rule_accepts=15221 overlapping_with_that_stride=0 overlapping_journal=0
name=w1_control_disk_stride p_times_chunk=6291456 rule_accepts=17463 overlapping_with_that_stride=8052 overlapping_journal=8052
name=w1_control_disk_stride p_times_chunk=9437184 rule_accepts=22316 overlapping_with_that_stride=22316 overlapping_journal=22316
```

判定：被判的句子写「S × 固定结构槽距 ≤ P × chunk」不够——要写成「≤ 3 MiB（第一版 P × chunk 的常量）」，并且 I-7.13 的表要加 P、chunk 两个字段等于第一版常量（与基址、journal 起点同一个处置），否则这一判在 checker 一侧罩不住 journal。这是第二轮的新缺口，不在第一轮打中的三处里。**什么现象会推翻它**：`crates/singlefs-core` 里有一处读偏移 363 或 367（我 `grep -rn 'PRIME_STEP_OFFSET\|CHUNK_BYTES_OFFSET' crates` 只命中 harness 测试一处）；或某个读者在读之前已把这两个字段判成常量（`grep -n '363\|367' crates/singlefs-checker/src/image.rs` 命中一处注释、读两处、取区域起点的乘法、测试镜像写两处，没有一处判它等于常量）。

### 1.6 加在哪条不变量下

加在 I-7.13（系统配置池级字段在读者收的范围里），不加 I-7.17（根槽位置等于 txg 算出的位置）、不加 I-7.5（根槽按判定宽度对齐）。理由：

- 这一判的输入是系统配置里的两个字段加格式常量，输出是整池拒、其余不变量「不适用」，与 I-7.13 的处置完全同形（`.claude/kb/decisions/22-单元原子性怎么合成.md:352`：「读者判固定结构槽距：≥ 4096，且槽 1 整槽落在同一槽自述的根环起点之前」那一句的下一格就是越界即拒）。
- I-7.17 判的是「一条自证过的根住在哪一槽」，输入是根；第一轮的 24 / 24 次红正是它在相叠盘上的症状（S 那一格今天只判区间，见 1.1）。I-7.13 加了这一格之后，过了 I-7.13 的池里 I-7.17 不会再因为区域相叠而红，它回到只抓错位的根；两条不是替代关系。
- I-7.5 是对齐，不判长度。

## W2：mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)

### 2.1 今天的实现里没有这个式子

mkfs 不算 S：S 是调用方放进 `geometry` 的输入（`crates/singlefs-core/src/system_configuration.rs:237`：「pub root_ring_slots_per_region: RootRingSlotsPerRegion,」），槽距同样是输入（`crates/singlefs-core/src/system_configuration.rs:231`：「pub fixed_structure_slot_spacing: u32,」）。入库里 44 个文件、77 处直接填 `RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM`（`grep -rn 'RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM' crates --include=*.rs | wc -l` 得 77，`grep -rl` 得 44）；`crates/` 里没有一处按槽距算 S（`grep -rn 'PRIME_STEP \* \|ROOT_RING_REGION_STRIDE' crates --include=*.rs` 只命中区域起点公式与装置里的复写，没有除法）。所以这一句要落成一个新的构造入口，放在 `SystemImmutableSizes::slot_spacing_for`（`crates/singlefs-core/src/system_configuration.rs:243`）旁边，返回 `Result`。

### 2.2 结果落在 4 或 16 上，与「所写的值不取两条边」打不打架

- 16：`min(8, …)` 的结果不超过 8，取不到。
- 小于 4：只在槽距 ∈ [786944, 1044480] 时发生；这些槽距上读者今天收、加了 W1 那一判之后任何 S ≥ 4 都被拒（见 1.4），所以「小于 4 拒绝 mkfs」没有独立的取值域，是 W1 那一判的另一个说法。
- 恰好 4：槽距 ∈ [629248, 786432]（512 的整数倍）。槽距是自由参数时能走到（4096 的整数倍里过对齐判、结果为 4 的有 39 个，第一个 630784），模型：

```text
name=w2_min_over_free_spacing written_S_histogram=[(3, 504), (4, 308), (5, 204), (6, 147), (7, 109), (8, 761)] spacing_giving_exactly_4=629248..786432 spacing_giving_below_4=786944..1044480 spacing_giving_16=none
name=w2_free_spacing_reaching_4 count=39 examples=[630784, 634880, 638976]
```

  按 D2 已定项 19 取槽距（4096 向上取整到 io_min 的整数倍，实现的算式在 `crates/singlefs-core/src/system_configuration.rs:248`：「floor.div_ceil(minimum_input_output_bytes) * minimum_input_output_bytes」）并且过 mkfs 的对齐判时，能走到的 io_min 只有 11 个，全是 2 的幂，写出的 S 只有 8 和一个 6（io_min = 512 KiB）：

```text
name=w2_conforming_io_min_passing_alignment count=11 io_min_values=[524288] written_S_not_8=[(524288, 6)] any_written_S_is_4_or_below=False
name=w2_conforming_io_min_all_powers_of_two value=True
name=w2_non_power_of_two_io_min io_min=786432 spacing=786432 reader_range=True alignment_pass=False written_S=4
```

  对齐判之所以把非 2 的幂 io_min 全挡掉：区域起点 1 / 4 / 7 MiB 要是判定宽度（max(physical_block_size, io_min)，`crates/singlefs-core/src/system_configuration.rs:255` 起）的整数倍，而 1 MiB 的因子只有 2 的幂。
- 没有任何代码把槽距绑到 io_min：池级 checker 里偏移 425 只用来算对齐判定宽度（`crates/singlefs-checker/src/image.rs:1515`：「.max(geometry.declared_minimum_input_output_bytes)」），core 的 `recovery.rs` 里没有这个字段的判；mkfs 收槽距参数而不核它等于 `slot_spacing_for(io_min)`（`crates/singlefs-core/src/make_filesystem.rs:448` 的 `check_geometry` 里没有这一判）。

判定：「所写的值不取两条边」在槽距按 D2 已定项 19 取的前提下成立（今天只会写 8 或 6），在槽距是自由参数的前提下不成立（会写 4）。这个前提条款没写、代码也不强制。要么在句子里改成「写 min(8, …)，所写的值不取上界」，要么 mkfs 加一判「槽距必须等于 4096 向上取整到 io_min 的整数倍」再保留原话；我不替主 agent 选。**什么现象会推翻它**：`make_filesystem.rs` 里有一处把槽距核成 `slot_spacing_for(io_min)`；或对齐判之外还有一条把槽距限成 2 的幂。

### 2.3 编译期断言

`RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM`（`crates/singlefs-core/src/root_ring.rs:44`：「pub const AT_MAKE_FILESYSTEM: Self = Self::at_make_filesystem_checked_at_compile_time(」）在编译期断言 8 严格落在 4 与 16 之间（`crates/singlefs-core/src/root_ring.rs:50`：「slots_per_region > ROOT_RING_SLOTS_PER_REGION_MINIMUM」），报错文字还写着旧条款「4..16 之间取值不碰任何一条边」（`crates/singlefs-core/src/root_ring.rs:52`）。新式子的值是运行时的，编译期断言只罩得住常量 8，罩不到 min 的结果；运行时入口只能走 `from_system_configuration_field`（`crates/singlefs-core/src/root_ring.rs:62`），它收两条边。所以「不取两条边」落成代码要在新构造入口里加运行时判，不是改这个常量。常量 8 仍是 min 的第一项，要留。

### 2.4 按「S = 8」写死的地方，哪些会错

| 处 | 判 | 依据 |
|---|---|---|
| E50 | 不依赖 8 | 扫的是槽总数 N（`.claude/kb/experiments/50-根环槽数的上下界.md:83`：「I-7.4（近 K 代块未被复用） 扣住的块数 = N × 每次 fsync 写的块数」）；被判的新 E50 一行的算术我复算：R = 3、13 块 / 次，S = 25 时 975 块、S = 26 时 1014 块；34 块 / 次，S = 9 时 918 块、S = 10 时 1020 块——都在线性式上推出，E50 页只在 N = 8、64 与「首个够线的 N」（256 / 32）取过样（N = 8 时 104 与 272 块，见 `.claude/kb/experiments/50-根环槽数的上下界.md:87`），S ≤ 25、S ≥ 10 是推的，不是量的 |
| D28 环窗口 | 不错 | kb 写成 S 的式子（`.claude/kb/decisions/28-挂载期承诺量.md:31`：「环窗口 = (根环槽数 3 × S + 1) × W，S 是系统配置每区槽数（S = 4 / 8 / 16 时环窗口 13W / 25W / 49W）」），实现读系统配置的 S（`crates/singlefs-core/src/admission.rs:500`：「let roots_in_the_ring_plus_one = ROOT_RING_REGIONS * slots_per_region.count() + 1;」）；S = 6 时是 19W，式子照用 |
| layout 一的 S 行与「根环参数一览」 | 对新池新建文件的字节表仍对（槽距 4096 ⇒ min = 8），对 io_min = 512 KiB 的池不对 | S 行在 `.claude/kb/layout/01-first-txn.md:140`（值列写 8）；`.claude/kb/layout/01-first-txn.md:164`：「根环参数一览：R = 3、S = 8、P = 3、chunk = 1 MiB、基址 1 MiB、槽宽 512、槽距 4096」 |
| 77 处填 `AT_MAKE_FILESYSTEM` 的调用 | 槽距 ≤ 384 KiB 的池不变；我只核了 `fixed_structure_slot_spacing:` 右边的种类（4096 共 35 处，`slot_spacing_for(512)` 共 15 处，带别的参数的 `slot_spacing_for(` 共 7 处，别的是转抄），没逐个核每个测试给的槽距值 | `grep -rhn 'fixed_structure_slot_spacing:' crates --include=*.rs` |
| `crates/singlefs-checker-tier/src/bin/` 下 E168、E180、E183 的本地常量 8 | 只会在装置模拟 io_min ≥ 512 KiB 的池时错；今天不模拟 | `grep -rn 'AT_MAKE_FILESYSTEM' crates/singlefs-checker-tier/src/bin/` |
| mkfs 编译期断言 | 见 2.3，罩不到 | 同上 |
| 根环一圈 3S 与钉槽 p | 不错 | S = 6 时钉值最大 17，模型 `w2_ring_depth S=6 … max_pin_value=17` |

**什么现象会推翻「D28 不错」**：`admission.rs` 之外有一处把 `3 * 8` 或 25 直接写进环窗口。我只 grep 了 `crates/singlefs-core/src/admission.rs` 与 `transaction.rs`（`grep -rn '25 \*\|\* 25'`），只命中 `admission.rs` 里一条用例（`slots(25 * 386)`，它传的就是常量 `AT_MAKE_FILESYSTEM`，是式子在 S = 8 的取值，不是写死）；别的文件没核。

### 2.5 另一个「S 随槽距变」之后会变的量

S × 槽距 = 区域长。S = 6、槽距 512 KiB 时区域长恰好 3 MiB，区域首尾相接，零空隙（`w1_touching_shapes_rule_accepts count=5`）；min 公式在 io_min = 512 KiB 上写出的就是「首尾相接」的一个，与 W1 取 `≤` 配套。

## W3：产生第一轮结论的方法（改字节造盘）

### 3.1 这一推成不成立：成立，理由是读代码，不是那三对几何

第一轮攻方的做法：mkfs 收的那一格（S = 4、io_min = 512 KiB）建池，把四个系统配置槽里偏移 362 那一字节改成 16、整槽校验和重封（`research/prompts/m3-close-c506-r1-opus-output.md:138` 起的「造法」一段）；等价性只在三对几何上逐扇区比过。三对只能说明「改字节」与「直接写」在那三格上同字节，推不到 S = 16、槽距 512 KiB 那一格（对方自己写了「外推」）。能把它推过去的是 mkfs 输出对 S 的依赖面，我在 `make_filesystem.rs` 里 grep 了 `root_ring_slots_per_region` 的全部使用：

```text
$ grep -n 'root_ring_slots_per_region\|slots_per_region' crates/singlefs-core/src/make_filesystem.rs
401:            geometry.root_ring_slots_per_region,
429:        (0..geometry.root_ring_slots_per_region.count()).map(move |slot| {
496:        geometry.root_ring_slots_per_region,
653:        parameters.geometry.root_ring_slots_per_region,
805:        parameters.geometry.root_ring_slots_per_region,
```

五处：区域长（401，判重叠用）、对齐判（429，逐槽）、盘内末端（496）、清零长度（653 → `crates/singlefs-core/src/make_filesystem.rs:671`）、盘外内存里的占用表（805，不落盘）。落盘的 S 依赖只有两样：系统配置偏移 362 那一字节（由 `SystemConfiguration::to_slot` 写，`crates/singlefs-core/src/system_configuration.rs:590` 起），和清零的长度。第 0 代根的落点 `genesis_root_ring_slots`（`crates/singlefs-core/src/make_filesystem.rs:790`）与 S 无关（区域 r 的槽 0），根记录里钉槽 p 写 255（`crates/singlefs-core/src/make_filesystem.rs:730` 一带），不依赖 S。所以「改一字节」与「直接按 S = 16 写」的盘只差清零的长度。入库里已有同一手法的先例：`crates/singlefs-harness/tests/system_configuration_slots_per_region.rs:119`：「fn declare_slots_per_region_on_disk(」。

### 3.2 改字节造的盘与合法写者的盘在别的字段上有没有差别、红是不是改字节造成的

- 差别只有一处：清零。S = 16 的写者会把每个区域清 8 MiB（`crates/singlefs-core/src/make_filesystem.rs:671`），改字节那一份只清过 S = 4 的 2 MiB。在全零的盘（第一轮用的稀疏盘）上两者逐字节相同；在非全零的盘上，多出来的那几段留着旧字节：读回来是「读坏」还是「全零」要分开记，两类在回收门槛与抬 F 上限里处置不同（`.claude/kb/decisions/16-发布语义.md:39` 那一行写了「读坏的并进准入抬 F 的上限，读回全零的不并」）。这对 I-7.17 的判没有影响，对空间后果有影响，第一轮没量，我也没量（复核不了：要跑盘）。
- I-7.17 红不是改字节造成的：红来自暖机发布按系统配置里的 S 写根（槽 = (txg ÷ 3) mod S），区域 0 槽 12 与区域 2 槽 0 同一段字节，模型 `w3_cell_16_512KiB_aliases` 行：

```text
name=w3_cell_16_512KiB_aliases pairs=d0:r0s12=r2s0;d0:r0s13=r2s1;d0:r0s14=r2s2;d0:r0s15=r2s3
name=w3_genesis_seed_scanned_elsewhere S=16 spacing=524288 hits=['seed_r2s0_is_scanned_as_r0s12']
name=w3_genesis_seed_scanned_elsewhere S=8 spacing=524288 hits=none
```

  由此推出（推的，没跑）：第 0 代种在区域 2 槽 0（7 MiB）的根，在 S = 16 下同时是区域 0 槽 12 的字节，I-7.17 对第 0 代只判槽号等于 0（`crates/singlefs-checker/invariants/02-根环.md:24`：「`checkpoint_txg` == 0（mkfs 把第 0 代种进全部区域」），所以 mkfs 刚写完、一次都没挂载的盘就该红；第一轮的历史只从第 1 次挂载之后读，没看这一格。这一条同时说明：合法写者与改字节造的盘在 mkfs 刚写完那一刻字节相同，红的时机也相同。
- 其余字段：偏移 362 之外，系统配置槽里没有字段由 S 派生（环长三字段由环长派生，`crates/singlefs-core/src/recovery.rs` 的 `journal_ring_geometry_fields_recompute_from_the_ring_length` 只看环长）。

### 3.3 「判决把 S = 16 / 512 KiB 那一格当成可达」成不成立

成立，但判决里的限定语要换。第一轮写的是「写得出它的是只照 D22 已定项 16 第 3 句判盘长的写者」。更准确的是：一个照 kb 现有全部条款写的 mkfs 写得出它。四个前提都是条款：

1. 槽距 = 4096 向上取整到 io_min 的整数倍（`.claude/kb/decisions/22-单元原子性怎么合成.md:63` 那一格，指向 D2 已定项 19），io_min = 512 KiB（md 默认 chunk，外部事实，未在本项目验证）⇒ 槽距 512 KiB；
2. 对齐：512 KiB 整除 1 / 4 / 7 MiB，过 I-7.5；
3. 盘长：基址 + (R−1) × P × chunk + S × 槽距 = 1 + 6 + 8 = 15 MiB ≤ 设备可用字节（`.claude/kb/decisions/22-单元原子性怎么合成.md:350` 那一句）；
4. 读者区间：S = 16 在 4..16 内、槽距在 [4096, 1 MiB − 4096] 内。

入库 mkfs 拒它只因为它多判了一样条款里没有的东西（1.1）。所以这一格是条款缺口，不是只存在于「别的写者」想象里。**什么现象会推翻它**：kb 里有一句已经要求区域不相叠（我 `grep -rn '相叠\|不重叠\|两两' .claude/kb/decisions/22-单元原子性怎么合成.md .claude/kb/layout/01-first-txn.md` 没有命中）。

### 3.4 方法上的限度

- 三对几何比对是在零盘上做的，不能证明非零盘上同字节。
- 红的时机（第 1 次挂载之后）与我推出的「mkfs 刚写完就红」不一致，两者没有冲突：他们的历史从挂载开始读，不是说挂载之前不红。我没跑，标推的。

## W4：从未被看过的地方

### 4.1 「环深上界 3 × 16 进格式，焊死的是管理员回退最多能退的发布数」与已定项 25、已定项 1

- 字面不冲突。已定项 25 禁的是把块复用的界做成格式常量：`.claude/kb/decisions/22-单元原子性怎么合成.md:495`：「块复用的界不能是格式常量」；被判的句子自己写了「块复用的界仍是 max(F_生效, 环里最旧有效根) 现算」。环深上界是对 S 这个系统配置字段的取值域的承诺，不是 K。
- 但读起来有两处拉扯：第一，已定项 25 把「焊死」当坏词（同一行：「格式常量把一个纯可用性参数焊死，没有任何好处」），被判的句子又用「焊死」说环深上界；第二，已定项 25 开头把根环限定成「只承担本代根写坏了还能读到上一代」（D22 第 75 行转述的就是它：根环只承担本代根写坏了还能读到上一代，不是可以回滚 K 代），而「管理员回退最多能退的发布数」恰是根环深度的另一个消费者。这个消费者在已定项 25 自己的依据里有（`.claude/kb/decisions/22-单元原子性怎么合成.md:507`：「E92（最坏块重用延迟需求）：九个消费者分三类计价，恢复择根回退恰 2 代、管理员回退要整个根环深度」），在 D16 里也有（`.claude/kb/decisions/16-发布语义.md:37`：「平时不动，窗口就是整个根环」；`.claude/kb/decisions/16-发布语义.md:41`：「回退候选集 | 按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效」）。
- 「发布数」的口径对：根环一个 txg 一条根，上界 3 × 16 = 48 个 txg；回退候选集里「4 个不同状态」是 F 抬到盘紧时的结果，不是环深。
- 建议写法（不改立场）：把「焊死」换成「环深上界 3 × 16 进格式：管理员回退的候选取自根环（E92），所以它也是回退最多能退的发布数；块复用的界不受它约束」，并在句子里点名 E92。**什么现象会推翻它**：已定项 25 的任何一句要求环深在格式层面可以随 F 或负载变；我没有读到。

### 4.2 「冻结前随之抬」与 D15 已定项 8

- 自洽。D15 已定项 8：`.claude/kb/decisions/15-格式冻结政策.md:171`：「已冻结字段的宽度 / 存在性 / 含义不能再改；只能新增，退役字段走「保留读支持、停止写」」，且 `.claude/kb/decisions/15-格式冻结政策.md:177`：「第一个外部用户出现之前格式是软的，这张表在那之前一条都不生效」。S 的下界 4 与它的取值域属于第 1 层（`.claude/kb/decisions/15-格式冻结政策.md:140`：「第 1 层：系统配置 + 块头（自身地址 / generation / 树 ID / fsid / magic）」）。冻结之前抬下界是改正文（软格式），冻结之后抬下界会让旧镜像里合法的 S = 4 变成整池拒，不是「新增」，所以改成「冻结前随之抬」是对的。D16 那一格的原话：`.claude/kb/decisions/16-发布语义.md:46`：「每区槽数 S ≥ k_tol + 2 = 4（D22（单元原子性怎么合成） 已定项 2 的 S 下界随之抬）」，把括号里的「S 下界随之抬」改成「冻结前随之抬」就够。
- 同一道理下，被判句子里的「抬上界先抬它（GPU 根表）」也只在冻结前有意义：冻结后抬上界是读者收的取值域变宽，D15 已定项 9 的判定表没有这一行（第一轮已挂账，`.claude/kb/decisions/15-格式冻结政策.md:192` 那一行是「改变已有字段含义」，不管取值域）。要在句子里带上「冻结前」。
- D16 已定项 1 射程的 S = 4 一格违反 24 次（`.claude/kb/decisions/16-发布语义.md:60`：「S = 4、c₀ = 5、3 次失败那一格违反 24 次」）说明下界 4 是边界余量为零的取值，格式常量把它钉死后 k_tol 也钉死在 2；这是已知后果，不是矛盾。

### 4.3 GPU 根表 48 与「抬上界先抬它」进不进格式条款

- 不进。48 = R × S 上界只是 GPU 判器根表的实现容量，今天只有字面量（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:241`：「const MAX_ROOTS: u32 = 48u;」），没有一处 `.rs` 里的 const 与它对账（`grep -rn 'MAX_ROOTS' crates --include=*.rs` 零命中）；超容时 GPU 判器报「不可判」而不是判错（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:1316`：「undecidable = UNDECIDABLE_TOO_MANY_ROOTS;」），所以它是失败即关的容量，不是格式承诺。
- 「上界另受两处约束」那一句数得不全：还有 harness 里按上界 48 位排的 `u64` 位图（`crates/singlefs-harness/src/fault_injection.rs:212`：「/// 每区域至多 `ROOT_RING_SLOTS_PER_REGION_MAXIMUM` 个槽 = 48 位，一个槽占一位，装得进 `u64`，」），它在 S 上界超过 21 时放不下；`root_record.rs` 与 `mount.rs` 的 `expect` 消息里写着「3S ≤ 48」（`crates/singlefs-core/src/root_record.rs:93`、`crates/singlefs-core/src/mount.rs:2310`）。这些都是实现容量。条款里只留 p 一字节那一项（格式，S ≤ 85）：钉值 = 3·槽 + 区域（`.claude/kb/decisions/22-单元原子性怎么合成.md:150`：「钉值 = 3·槽 + 区域」），最大 3S − 1，255 是哨兵 ⇒ S ≤ 85（模型 `w2_pin_bound max_S_with_pin_below_255=85`）；GPU 容量与 harness 位图写进 checker 档的欠账或实验页，不写进已定项 2。
- 另：格式常量门禁（format-const）只对 Rust 的 `pub const` 对账，WGSL 里的 `4u` / `16u` / `48u` 不在其内（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:960`：「if (slots_per_region < 4u || slots_per_region > 16u) {」与 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:111` 同写），登记两条边之后 GPU 三处字面量仍不受钉（第一轮已记，我没有新证据）。

## 按正推，被判形态每一句该怎么写（正推结果，不是判决）

| 被判的句子 | 正推结论 | 依据 |
|---|---|---|
| 「同一槽另判 S × 固定结构槽距 ≤ P × chunk」 | 要保留；右边写成第一版常量 3 MiB（P = 3、chunk = 1 MiB），判 `≤` 不判 `<`；P、chunk 两个字段并进 I-7.13 的「等于第一版常量」一类，与基址、journal 起点同一处置；登记在 I-7.13 下，不登 I-7.17 | 1.3、1.5、1.6 |
| 「读者（core、池级 checker、GPU 判器）整池拒、mkfs 拒」 | 读者三处落点见 1.2（GPU 是两个函数）；mkfs 不用加代码，条款补一句「mkfs 今天已经这样判」即可 | 1.2、1.4 |
| 「mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)，小于 4 拒绝 mkfs；所写的值不取两条边」 | 「小于 4 拒绝」没有独立取值域；「不取两条边」要么改成「不取上界」，要么 mkfs 加槽距 = `slot_spacing_for(io_min)` 的判；需要新的运行时构造入口 | 2.1、2.2、2.3 |
| 「环深上界 3 × 16 进格式，焊死的是管理员回退最多能退的发布数」 | 站住；去掉「焊死」，点名 E92，保留「块复用的界仍现算」 | 4.1 |
| 「上界另受两处约束：钉槽 p …… S ≤ 85；GPU 判器根表容量 48 = 3 × 16，抬上界先抬它」 | p 一字节留在条款里（格式）；48 与「先抬它」移出，写进 checker 档的欠账或实验页；两句都带「冻结前」 | 4.2、4.3 |
| 「D16 已定项 1 环深下限『S 下界随之抬』改『冻结前随之抬』」 | 站住 | 4.2 |
| 依据段 E50 一行 | 算术站住（S = 25 时 975 块、S = 26 时 1014 块；S = 9 时 918 块、S = 10 时 1020 块），都是线性式上推的，不是在这些 N 上量的，要标「推的」 | 2.4 |
| （没写）槽距上界 | 加一句：S ≥ 4 时有效的槽距上界是 768 KiB，不是 1 MiB − 4096 | 1.4 |

## 没做什么

- 不判别的腿的格，不替主 agent 采纳或出判决；2.2 与 1.5 里的「怎么改」只列选项，没有选。
- 没编译 Rust、没跑任何测试与重型测试；没造盘。`crates/` 里的判断全是读代码与 grep；非全零盘上的空间后果（3.2）、io_min = 512 KiB 是不是 md 默认 chunk（3.3）复核不了。
- 没逐个核 77 处 `AT_MAKE_FILESYSTEM` 调用点的槽距；只核了 `fixed_structure_slot_spacing:` 右边的种类（4096 共 35 处、`slot_spacing_for(512)` 15 处、带别的参数的 `slot_spacing_for(` 7 处）。
- 没核 D28 之外有没有别处把 25 或 `3 * 8` 写死（2.4 末行写了「没核」）。
- 没读禁读清单里的三份（第二轮攻方的产出、模型与主 agent 核验文件）。
