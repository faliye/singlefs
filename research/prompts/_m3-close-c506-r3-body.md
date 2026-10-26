# m3-close-c506-r3：每区槽数 S 的上界改成格式常量 16，第二轮修补之后的形态站不站得住（推论轮第三轮，最终轮）正文（2026-10-22）

## 一、题面

前两轮判决：`research/prompts/m3-close-c506-r1-main-verification.md`、`research/prompts/m3-close-c506-r2-main-verification.md`。被判对象同前两轮：用户 2026-10-22 定案把 D22（单元原子性怎么合成） 已定项 2 字段表「每区槽数 S」那一格的「**上界由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算**」改成「上界 16 是格式常量」。第二轮挂起四处打中：读者之间 P（偏移 363）、chunk（偏移 367）读法不一致（core 只写常量不读，池级 checker 与 GPU checker 读盘上字段）、mkfs 的 min 公式在一段槽距上写出下界 4、「环深上界」用了同一格点名不用的词且量纲不同、「焊死管理员回退」与 D22 已定项 25 相反。

这一轮攻的是第二轮判决第三节「第三轮攻的形态」（下面第三节照抄）。这是最后一轮：分工照 `.claude/rules/three-way-inference.md` 第三轮——「已定」项撑不撑得住 / 未定项清单的完整性 / 那一轮报出的数字能不能核。这一轮新冒出来的改法算零轮，写进交用户表，不再开第四轮。

## 二、实现今天的样子（主 agent 的观测，2026-10-22，行号按主工作区）

- `crates/singlefs-format/src/lib.rs`：`ROOT_RING_SLOTS_PER_REGION_MINIMUM = 4`、`ROOT_RING_SLOTS_PER_REGION_MAXIMUM = 16`（带 placeholder 注释）、`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM = 8`；`ROOT_RING_PRIME_STEP`、`ROOT_RING_CHUNK_BYTES` 是编译期常量。
- `crates/singlefs-core/src/system_configuration.rs` 第 593–594 行写系统配置时把 P、chunk 写成常量；core 读者不读这两个字段（`grep -n '363\|367\|PRIME_STEP\|CHUNK_BYTES'` 在 `recovery.rs`、`system_configuration.rs` 读路径里 0 处）。`crates/singlefs-core/src/root_ring.rs` 的 `region_start` 按常量放区域。
- `crates/singlefs-checker/src/image.rs` 第 773–774 行、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 第 173–174 行按盘上字段读 P、chunk。
- `crates/singlefs-core/src/make_filesystem.rs` 的 `check_geometry` / `fixed_structure_extents` 两两判固定结构不相叠，第二轮正推模型 26429 个 (S, 槽距) 与「S × 槽距 ≤ 3 MiB」逐形状相等。
- `crates/singlefs-core/src/root_ring.rs` 的编译期断言只罩常量 8；`AT_MAKE_FILESYSTEM` 有 77 处直接调用（第二轮正推数的）。
- GPU：`crash_judge_common.wgsl` `MAX_ROOTS = 48u`、`crash_judge_checker_common.wgsl` `C_MAX_CANDIDATES = 48u`、三处 `4u` / `16u` 字面量。
- 前两轮的模型可以拿来改臂再量：`research/prompts/m3-close-c506-r1-opus-model/`、`research/prompts/m3-close-c506-r2-opus-model/`、`research/prompts/m3-close-c506-r2-sonnet-model/`（只读，要改拷进自己这一轮的模型目录）。

## 三、被判的形态（照抄第二轮判决第三节）

D22 已定项 2 字段表「每区槽数 S」那一格，把「**上界由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算**；4..16 之间取值不碰任何一条边。」换成：

> **每区槽数上界 16 是格式常量**，下界 4 同为格式常量（两条边随系统配置槽落 D15（格式冻结政策） 第 1 层，登记在 layout 一「每区槽数 S」那一行旁）；环长上界 3 × 16 = 48 个槽。上界是读者承诺收的最大值，不从 I-7.4（近 K 代块未被复用） 扣住的块数算：那个「K 代」是按 F 与实例表现算的回退候选集，给不出只由池定的数；块复用的界仍是 max(F_生效, 环里最旧有效根) 现算，不是格式常量（已定项 25）；一个池上管理员回退能退到哪里由回退候选集（txg ≥ F_生效）与这个池自己的 3S 定。上界另受根记录钉槽 p 的一字节约束：钉值最大 3S − 1、255 是哨兵 ⇒ S ≤ 85。**读者解槽时读 P（偏移 363）与 chunk（偏移 367），与第一版常量（3、1 MiB）不等一律整池拒；再判 S × 固定结构槽距 ≤ P × chunk**（区域不越过下一个区域的起点），越界与 S 越界同一个处置：读者（core、池级 checker、GPU 判器）整池拒，mkfs 拒。mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)，结果小于 5 拒绝 mkfs，所以所写的值不取两条边。

依据段 E50 那一行换成：

> E50（根环槽数的上下界）：扫的是槽总数 N = R × S；R = 3 时粗粒度档（13 块/次）S ≤ 25 不过跑前定的 1000 块线，散开档（34 块/次）S ≥ 10 就过线 ⇒ 上界随负载与那条线变，没有只由池定的数；16 是读者承诺，不是 E50 量出的上界。

D16（发布语义） 已定项 1「环深下限」那一格的「S 下界随之抬」改成「冻结前随之抬；冻结之后 k_tol 不再抬」。

## 四、格

| 格 | 问什么 |
|---|---|
| X1 | 「已定」项撑不撑得住：被判形态引的每一条已定项（D22 已定项 2、16、25，D16 已定项 1，D15 已定项 7、8、9，D2（RAID 条带策略） 已定项 19，I-7.13、I-7.17）逐条读原文，被判形态哪一句与它们字面或量纲打架；「P、chunk 与第一版常量不等一律整池拒」会不会撞上 D2 / D22 里「P 素数且 P > devs、按盘数选 P」一类条款（多盘时合法的 P 不是 3） |
| X2 | 未定项清单的完整性：被判形态与第二轮判决「实现欠」「另登欠账」两张表，漏了哪些该随这一改一起动的地方（读者、mkfs、checker 档 GPU、harness 模型、format-const 标记、三语译文、实验页「影响的决策」回看、layout 字段表）；「mkfs 写 min(8, …) 小于 5 拒」在 D2 已定项 19 的槽距取法下还剩什么取值 |
| X3 | 前两轮报出的数字能不能核：第一轮「24 / 24 判 I-7.17 红、对照 96 / 96 干净」、「S ≤ 85」、「粗粒度 S ≤ 25、散开 S ≥ 10」；第二轮「26429 / 29744 格逐形状相等、错拒 0」、「槽距 629248..786432 写 4」、「有效槽距上界 768 KiB」、「77 处直接调用 AT_MAKE_FILESYSTEM」。逐个用一条独立于原腿的办法复核（换算式、换模型、grep 计数），对不上的写出来 |

## 五、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | X1–X3 | 复核前两轮判决：替被判出局或被改掉的写法辩护（「min(8, …) 小于 4 拒」「环深上界」「焊死管理员回退」「GPU 根表进条款」各自有没有站得住的读法），判两份判决的打中是不是够得着、是不是同样打中所有替代写法；X3 的数字逐个复核 |
| 云端攻方（Opus） | X1–X3 | 找反例：X1 被判形态与已定条款打架的句子，尤其 P、chunk 钉常量与多盘选 P 的条款；X2 漏掉的落点；X3 前两轮数字核不上的。不重复前两轮攻过的角度（E50 读错量、读者不判 S × 槽距这件事本身、两条边没登记、P / chunk 读法不一致这件事本身、min 公式写 4、「环深」「焊死」两处措辞） |

方案按 `crates/` 今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
