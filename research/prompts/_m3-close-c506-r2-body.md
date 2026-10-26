# m3-close-c506-r2：每区槽数 S 的上界改成格式常量 16，第一轮修补之后的形态站不站得住（推论轮第二轮）正文（2026-10-22）

## 一、题面

第一轮判决 `research/prompts/m3-close-c506-r1-main-verification.md`。被判对象同第一轮：用户 2026-10-22 定案把 D22（单元原子性怎么合成） 已定项 2 字段表「每区槽数 S」那一格的「**上界由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算**」改成「上界 16 是格式常量」。第一轮挂起三处打中：依据段 E50（根环槽数的上下界） 那一行把槽总数 N 读成了 S；读者两侧（core、池级 checker）不判 S × 槽距，攻方在副本上造出 S = 16、槽距 512 KiB 的盘，两个读者都收，暖机挂载 24 次 24 / 24 判 I-7.17 红；区间两条边在 kb 没有 format-const 登记。

这一轮攻的是判决第三节「第二轮攻的形态」（逐字在判决里，下面第三节照抄）。分工照 `.claude/rules/three-way-inference.md` 第二轮：第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方。

## 二、实现今天的样子（主 agent 的观测，2026-10-22，行号按主工作区）

- `crates/singlefs-format/src/lib.rs`：`ROOT_RING_SLOTS_PER_REGION_MINIMUM = 4`、`ROOT_RING_SLOTS_PER_REGION_MAXIMUM = 16`（带 placeholder 注释，指 C506（每区槽数 S 写成编译期常量，条文说它住系统配置））、`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM = 8`。
- `crates/singlefs-core/src/root_ring.rs`：`RootRingSlotsPerRegion` 只判 S 的区间 4..16；`region_start` 按基址 + 区域号 × P × chunk 算区域起点（1 / 4 / 7 MiB）；区域长 = S × 槽距。
- `crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts` 与 `crates/singlefs-checker/src/image.rs` 的 `geometry_of`：判 S 区间与槽距区间，不判 S × 槽距；`grep -n 'overlap\|相叠\|重叠'` 在这两份与 `system_configuration.rs` 里没有一处判。
- `crates/singlefs-core/src/make_filesystem.rs`：`fixed_structure_extents` 两两判不相叠，不分盘，叠了报 `FixedStructuresOverlap`。
- GPU 判器：`crates/singlefs-checker-tier/src/crash_judge_common.wgsl`、`crash_judge_recovery.wgsl`、`crash_judge_checker.wgsl` 各有 `4u` / `16u` 字面量判区间；`crash_judge_common.wgsl` 的 `MAX_ROOTS = 48u`。
- `crates/singlefs-core/src/root_record.rs`：钉槽 `PinnedRingSlot(pub u8)`，`ROOT_RECORD_NO_RING_SLOT_PINNED = 255`。
- 第一轮攻方模型 `research/prompts/m3-close-c506-r1-opus-model/`（主 agent 复跑逐字相同）可以拿来改臂再量；腿的新模型写在自己这一轮的模型目录。

## 三、被判的形态（照抄判决第三节）

D22 已定项 2 字段表「每区槽数 S」那一格，把「**上界由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算**；4..16 之间取值不碰任何一条边。」换成：

> **上界 16 是格式常量**，下界 4 同为格式常量（两条边随系统配置槽落 D15（格式冻结政策） 第 1 层，登记在 layout 一「每区槽数 S」那一行旁）。上界是读者承诺收的最大值，不从 I-7.4（近 K 代块未被复用） 扣住的块数算：那个「K 代」是按 F 与实例表现算的回退候选集，给不出只由池定的数；块复用的界仍是 max(F_生效, 环里最旧有效根) 现算，不是格式常量（已定项 25）。环深上界 3 × 16 进格式，焊死的是管理员回退最多能退的发布数。上界另受两处约束：根记录钉槽 p 一字节、钉值最大 3S − 1、255 是哨兵 ⇒ S ≤ 85；GPU 判器根表容量 48 = 3 × 16，抬上界先抬它。**同一槽另判 S × 固定结构槽距 ≤ P × chunk**（区域不越过下一个区域的起点），越界与 S 越界同一个处置：读者（core、池级 checker、GPU 判器）整池拒、mkfs 拒。mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)，小于 4 拒绝 mkfs；所写的值不取两条边。

依据段 E50 那一行换成：

> E50（根环槽数的上下界）：扫的是槽总数 N = R × S；R = 3 时粗粒度档（13 块/次）S ≤ 25 不过跑前定的 1000 块线，散开档（34 块/次）S ≥ 10 就过线 ⇒ 上界随负载与那条线变，没有只由池定的数；16 是读者承诺，不是 E50 量出的上界。

D16（发布语义） 已定项 1「环深下限」那一格的「S 下界随之抬」改成「冻结前随之抬」。

## 四、格

| 格 | 问什么 |
|---|---|
| W1 | 「S × 槽距 ≤ P × chunk」这一判本身：它罩不罩得住第一轮打中的那一类（同盘区域相叠）的全部形状——区域 2 末端越过 journal 环起点（16 MiB）、区域与系统配置槽相叠、槽距不是 2 的幂时；它会不会错拒 mkfs 今天写得出、盘上并不相叠的几何；它与 I-7.17、I-7.13 的关系（加在哪条不变量下） |
| W2 | 「mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)、所写的值不取两条边」：min 的结果落在 4 或 16 上时与「不取两条边」打不打架；S 随槽距变之后 E50、D28（挂载期承诺量） 环窗口、mkfs 编译期断言这些按「S = 8」写死的地方哪些会错 |
| W3 | 产生第一轮结论的方法：攻方用改字节造盘代替「只照条款判盘长的写者」，这一等价性只在三对 mkfs 两边都收的几何上逐扇区比过；判决把 S = 16 / 512 KiB 那一格当成可达。这一推是否成立；改字节造的盘与合法写者能写出的盘在别的字段上有没有差别、会不会让 I-7.17 红是改字节造成的 |
| W4 | 从未被看过的地方：被判的形态里「环深上界 3 × 16 进格式，焊死的是管理员回退最多能退的发布数」与 D22 已定项 25、D16 已定项 1 管理员回退的条款有没有冲突；「冻结前随之抬」与 D15 已定项 8 冻结之后只能新增的条款是否自洽；GPU 根表 48 与「抬上界先抬它」进不进格式条款（它是 checker 档的实现容量，不是格式） |

## 五、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | W1–W4 | 从附录条款与 `crates/` 今天的代码正推被判的形态每一句该不该这样写；W1 写出这一判在三个读者与 mkfs 里各落在哪个函数 |
| 云端攻方（Opus） | W1–W4 | 找反例：W1 里这一判罩不住的相叠形状与它错拒的合法几何；W2 里 min 公式撞边或撞写死的 8；W3 里改字节造盘与合法写者的差别；W4 里与已定条款打架的句子。不重复第一轮攻过的角度（E50 读错量、读者不判 S × 槽距这件事本身、两条边没登记） |

方案按 `crates/` 今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
