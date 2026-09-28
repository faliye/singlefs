# m2-closeout-code-r2 本地攻方腿 提示翻译核对表

栏位：英文项 / 原文文件:行 / 首稿缺的 / 定稿

## 派发提示原句（分工表 / 事实表六项）

原文：「按事实表逐格算数、与仓里钉的数比：C577 之后第一条流的段序列 2+2+1+2+2+1+2+24+2+1+2 与两态闭式 16777240；隔离看第一个事务 33 → 35 步、暖机 18 → 20 步；σ 段闭式 2^18 → 2^16；单元区起点在 768 MiB 环（50176）、6 MiB 环（1408）、159 MiB 环（11200）上的值与「在 64 槽段边界上」；1 MiB 环的在飞上限 85；I-7.13 各字段的界」

1. 英文项：任务开场句「fill in every cell ... show the arithmetic step ... state whether it matches the pinned value」
   原文文件:行：主 agent 派发提示「每格让模型算出数、与钉的数比，答不一致的给出它的算式」
   首稿缺的：无
   定稿：同首稿

## FACT SET A（第一条流段序列与闭式）

2. 英文项：A1「First stream: 2+2+1+2+2+1+26+2+1+2 (67108885) becomes 2+2+1+2+2+1+2+24+2+1+2. Closed form = 1 + (3+3+1+3+3+1+3+3+1+3) + (2^24 - 1) = 16777240; after the change, the in_place_overwrite test prints exactly 16777240.」
   原文文件:行：research/prompts/m2-impl-c577-barrier-implementer-report.md:75「第一条流：`2+2+1+2+2+1+26+2+1+2`（67108885）变成 `2+2+1+2+2+1+2+24+2+1+2`。闭式 = 1 + (3+3+1+3+3+1+3+3+1+3) + (2^24 − 1) = 16777240，in_place_overwrite 那条改后打出的正是 16777240。」
   首稿缺的：无，逐句照译，数字与算式原样保留
   定稿：同首稿

3. 英文项：A2「closed form = 1 + sum over each segment of (2^(segment length) - 1)」
   原文文件:行：crates/singlefs-harness/src/crash.rs:621（函数 closed_form_state_count 的文档注释「E77（发布的持久顺序） 的闭式：1 + Σ(2^|段| − 1)，每次写只取两态（没持久 / 持久）时的全量」）与 :623-628（函数体 `1 + segments.iter().map(|segment| (1u64 << segment.len()) - 1).sum::<u64>()`）
   首稿缺的：首稿原先误标成 m2-impl-c577-barrier-implementer-report.md:71，那一行是空行、:75 只有代入数字之后的算式（1 + (3+3+...) + (2^24-1) = 16777240），不含符号形式的 Σ(2^|段|−1)；符号形式的原文出处是 crash.rs 那条函数注释，改标到那里
   定稿：同已改的行

## FACT SET B（隔离路径步数）

4. 英文项：B1「Every publish adds one pool barrier after the rotation; the recording stream logs it per device: two more steps overall (one Barrier per disk, two disks).」
   原文文件:行：research/prompts/m2-impl-c577-barrier-implementer-report.md:72「每次发布在轮换之后多一道池屏障，录制流按设备记，一次多 2 步（两块盘各一条 Barrier）。」
   首稿缺的：无
   定稿：同首稿

5. 英文项：B2「Isolated-view path: the first transaction's 24+2+1+2 segment sequence is unchanged, the last segment's write kinds gain barrier x2, step count 33 becomes 35. The warm-up path's step count 18 becomes 20; its segment sequence 2+1+2+2+1+2 is unchanged.」
   原文文件:行：research/prompts/m2-impl-c577-barrier-implementer-report.md:76「隔离看的路径：第一个事务 `24+2+1+2` 段序列不变，末段种类多 `barrier×2`，步数 33 变 35。暖机路径步数 18 变 20，段序列 `2+1+2+2+1+2` 不变。」
   首稿缺的：无
   定稿：同首稿

## FACT SET C（σ 段闭式）

6. 英文项：C1「sigma (record_checker): (18, 16) becomes (16, 16); the sigma segment's closed form 2^18 becomes 2^16. The crash-case row c561-sigma-full's registered note 'closed form 2^18' must change along with it.」
   原文文件:行：research/prompts/m2-impl-c577-barrier-implementer-report.md:77「σ（record_checker）：`(18, 16)` 变 `(16, 16)`，σ 段闭式 2^18 变 2^16。登记在 crash-case 的 `c561-sigma-full` 那一行的「闭式 2^18」也要跟着变。」
   首稿缺的：无，"record_checker" 与登记行名 c561-sigma-full 原样保留
   定稿：同首稿

## FACT SET D（单元区起点）

7. 英文项：D1「Under the default 768 MiB ring, the start is still 50176.」
   原文文件:行：research/prompts/m2-rev-a3b-implementer-report.md:30「默认环 768 MiB 下起点仍是 50176。」（同一行后半段讲 CRC-32C 核对，与本题算术无关，未译入）
   首稿缺的：无
   定稿：同首稿

8. 英文项：D2「I take a 6 MiB ring: start = 1024 + 384 = 1408 = 22 x 64, on the segment boundary.」
   原文文件:行：research/prompts/m2-rev-a3b-implementer-report.md:84「我取 6 MiB 环：起点 1024 + 384 = 1408 = 22 × 64，在段边界上。」（同一行后半段讲三档盘宽与在飞上限 512 条，与本题算术无关，未译入）
   首稿缺的：无
   定稿：同首稿

9. 英文项：D3「Change the ring to 159 MiB: start = 1024 + 159 x 64 = 11200 = 50176 - 3 x 12992. 12992 = lcm(812, 64), so the start's position within its leaf and within its segment is the same as 50176's: 168 slots before leaf 13's last slot 11367, same as the default ring's 168 slots before leaf 61's last slot 50343. 159 MiB <= 1 GiB / 4.」
   原文文件:行：research/prompts/m2-rev-a3b-fallout-1-implementer-report.md:132「环改成 159 MiB：起点 1024 + 159 × 64 = 11200 = 50176 − 3 × 12992。12992 = lcm(812, 64)，所以起点在叶内、在段内的位置与 50176 相同：离叶 13 的末槽 11367 是 168 槽，同默认环离叶 61 末槽 50343 的 168 槽。159 MiB ≤ 1 GiB ÷ 4。」
   首稿缺的：无，逐句照译
   定稿：同首稿

10. 英文项：D4「the general form you should use, read off from D2 and D3's own arithmetic: start = 1024 + (ring size in MiB) x 64」
    原文文件:行：无对应原文单句——这是从 D2、D3 两处各自的算式（1024+384、1024+159×64）里挑出的公共系数 64，本轮派发提示第 4 条本身给的口径是「单元区起点 = 1024 + 环长 ÷ 16384」，与「1024 + MiB 数 × 64」是同一件事（1 MiB ÷ 16384 字节 = 64），派发提示给的是字节除法形式，D2、D3 原文给的是「MiB 数 × 64」形式；这里选用 D2、D3 自己写出来的系数形式，不再另译派发提示的字节除法句，因为两种写法数值等价且 D2、D3 已经把「× 64」摆在原文里，选贴合原文写法的一种，不是在原文之外新造一个公式
    首稿缺的：见上，此项本身就是「多出来的」，另在下面第 12 条单列
    定稿：D4 保留「read off from D2 and D3's own arithmetic」这句限定，不称它是引用某一行原文

## FACT SET E（1 MiB 环在飞上限）

11. 英文项：E1「pub const JOURNAL_SAFETY_FACTOR: u64 = 3;」
    原文文件:行：crates/singlefs-format/src/lib.rs:221（同一行原样，代码常量声明，非中文，逐字抄）
    首稿缺的：无
    定稿：同首稿

12. 英文项：E2「The per-publish record-count-limit test is now measured against a 1 MiB ring's in-flight limit of 85: 85 records are admitted, 86 are rejected; the write-row cell is 85 instance-table shards plus the allocation-record tree's nodes.」
    原文文件:行：research/prompts/m2-rev-a3b-fallout-1-implementer-report.md:12「按主 agent 的定（Q6：不为测试开口子），段边界外的小环一条都没留，全换成 1 MiB 整数倍的环。一次发布记录条数上限的用例改在 1 MiB 环的在飞上限 85 上测：85 条放行，86 条拒；写行那一格是 85 片实例表加上分配记录树的节点。没有「内存或时长撑不住」的格。」
    首稿缺的：首稿漏译了句首「按主 agent 的定（Q6：不为测试开口子），段边界外的小环一条都没留，全换成 1 MiB 整数倍的环」——判定：这句是背景理由（为什么改用 1 MiB 环），不是本题要算的算术输入，本题只需要「85 条放行、86 条拒」与「写行那一格是 85 片实例表加上分配记录树的节点」两句，故不译入 E2 正文；句尾「没有「内存或时长撑不住」的格」同样是背景说明，不译入
    定稿：E2 只保留「1 MiB 环的在飞上限 85：85 条放行、86 条拒；写行那一格是 85 片实例表加上分配记录树的节点」这一句的英译，句首、句尾两处背景说明不译，理由记于此

13. 英文项：E3「the formula, read off from E2's own label 'in-flight limit': in_flight_limit = floor(ring_bytes / 4096 / JOURNAL_SAFETY_FACTOR), where a 1 MiB ring is 1048576 bytes and JOURNAL_SAFETY_FACTOR is E1's constant.」
    原文文件:行：主 agent 派发提示第 5 条「1 MiB 环的在飞上限 = 环长 ÷ 4096 ÷ F，F = `JOURNAL_SAFETY_FACTOR` = 3」
    首稿缺的：无，公式与常量名原样保留；floor 语义是本轮任务要求模型自己做除法时补的运算细节，不是原文没有的限定词，是把「÷」在整数上下文里该取的语义显式化
    定稿：同首稿

## FACT SET F（I-7.13 字段界）

14. 英文项：F1「Fixed structure slot spacing >= 4096 bytes (FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES) and slot 1's whole slot falls before the root ring base address; physical_block_size in [457 (ROOT_RECORD_BYTES), slot spacing]; journal ring bytes / 4096 / F (JOURNAL_SAFETY_FACTOR) >= 1 and the ring's end does not cross the unit-area start slot number.」
    原文文件:行：.claude/kb/invariants.md:64「固定结构槽距 ≥ 4096 字节（`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`）且槽 1 整槽落在根环基址之前、`physical_block_size` ∈ [457（`ROOT_RECORD_BYTES`）, 槽距]、journal 环长 ÷ 4096 ÷ F（`JOURNAL_SAFETY_FACTOR`）≥ 1 且环末端不越过单元区起始槽号」
    首稿缺的：无，逐词照译；这一行本身只给槽距的下界（≥4096）与一条位置性上界（槽 1 落在根环基址之前），没有把这条位置性上界换算成具体字节数——换算成字节数的句子在 F2，不在这一行，首稿没有在 F1 里编造一个数字上界，是忠实的
    定稿：同首稿

15. 英文项：F2「When the reader selects a system configuration, it judges the root slot width and the fixed structure slot spacing: fixed structure slot spacing in [4096, 1 MiB - 4096] (the upper bound is that slot 1's whole slot falls before the root ring base address, root_ring::region_start(0) minus the 4096-byte system-configuration slot width); root slot width (physical_block_size) in [457-byte root record width (item 7), slot spacing]; out-of-range values reject the whole pool at mount.」
    原文文件:行：.claude/kb/decisions/22-单元原子性怎么合成.md:72「读者择系统配置时判根槽宽与固定结构槽距：固定结构槽距 ∈ [4096, 1 MiB − 4096]（上界是槽 1 整槽落在根环基址之前，`root_ring::region_start(0)` 减系统配置槽宽 4096）；根槽宽（`physical_block_size`）∈ [根记录宽 457 字节（已定项 7）, 槽距]；越界整池拒绝挂载，与每区槽数 S 越界同一个处置（……）」
    首稿缺的：首稿漏译了行尾「与每区槽数 S 越界同一个处置（实审 A3a 落地：……；池级 checker 报 I-7.13 违例、该池其余不变量一律报不适用……）」这一整段落地与处置细节——判定：这段讲的是「越界之后系统怎么处置」的实现落点，不是本题要算的数字界本身，本题只需要区间 [4096, 1 MiB − 4096] 与 [457, 槽距] 两个数字区间，故不译入 F2；F2 保留到「越界整池拒绝挂载」这句为止
    定稿：F2 只保留区间数字与「越界整池拒绝挂载」，落地实现细节不译，理由记于此。这一条本身是本轮追加的引用（主 agent 只点名 invariants.md:64，追这一行里的指针「D22 已定项 2」找到的），在下面第 16 条单列说明为什么加

16. 英文项：F2 整条（追加来源）
    原文文件:行：.claude/kb/invariants.md:64 本身没有写出 [4096, 1 MiB − 4096] 这个数字上界，只写了「≥ 4096」与「槽 1 整槽落在根环基址之前」这条位置性描述；主 agent 派发提示第 6 条给的事实是「固定结构槽距 ∈ [4096, 1 MiB − 4096]」，这个数字上界的原文出处是 invariants.md:64 里的指针「读者收的界在 D22（单元原子性怎么合成） 已定项 2 与已定项 16」指向的 .claude/kb/decisions/22-单元原子性怎么合成.md:72
    首稿缺的：首稿如果只抄 invariants.md:64 会缺这个数字上界的字面出处；补法是追这一行自己给的指针，把 D22 已定项 2 那一句（decisions/22:72）作为 F2 另列一条来源，不是在 invariants.md:64 之外凭空造一个数字
    定稿：F1 只忠实抄 invariants.md:64（不含数字上界字面），F2 另起一条抄 decisions/22:72（含数字上界字面），两条都进提示，模型看到的是两条来源叠加起来的完整界，不是我自己拼出来的一句
