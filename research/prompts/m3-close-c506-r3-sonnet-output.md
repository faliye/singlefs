# m3-close-c506-r3 辩方腿（Sonnet）报告（2026-10-23）

被复核的判决：`research/prompts/m3-close-c506-r1-main-verification.md`、`research/prompts/m3-close-c506-r2-main-verification.md`。被判形态是正文 `research/prompts/_m3-close-c506-r3-body.md` 第三节。立场：替被判出局或被改掉的写法找站得住的读法，判两份判决的打中够不够得着、是不是同样打中所有替代写法；X3 的数用独立于原腿的办法重算。方案按 `crates/` 今天的实现谈。没读禁读清单里的三份。kb 条款一律写 kb 文件名加那份文件自己的行号（现查）。

## 复跑与模型

模型 `research/prompts/m3-close-c506-r3-sonnet-model/`：纯 Python 标准库，不编译 Rust。式子从 kb 条款和 `crates/` 源码读来另写（没拷前两轮的模型），源码常量先回读对账（`name=source_check`）。它读的是冻结副本（`crates/` 与 `scripts/experiment-apparatus/e50_ring_slots.rs`，拷自主工作区，下列六份与主工作区逐字节相同），所以复跑要自备一份冻结副本。

```text
$ SINGLEFS_RUN_SCOPE='种类=model; 跑什么=m3-close-c506-r3 辩方腿 Python 纯算模型复跑; 范围=模型目录与草稿目录, 只读 frozen 下的 crates; 为什么=核模型输出与落盘 out/model.out 逐行相同' \
  nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 1G bash research/prompts/m3-close-c506-r3-sonnet-model/tools/rerun.sh \
  --repo . --frozen <冻结副本> --work <新的草稿目录> --model research/prompts/m3-close-c506-r3-sonnet-model
name=rerun_compare same=1 lines=54 sha256=75d5d21481e91c1a89cbab84dc6d695ce0952142505b7c71ceb00fc28cd89551
```

```text
d98afb149f9b7b08ee1b44caa31dde6e6dc4cc3405e6aa3a5900b23eef6f329d  research/prompts/m3-close-c506-r3-sonnet-model/src/model.py
75d5d21481e91c1a89cbab84dc6d695ce0952142505b7c71ceb00fc28cd89551  research/prompts/m3-close-c506-r3-sonnet-model/out/model.out
5cf230f40d72e9987b4bfb46de272542ceaccf9d31d10308f81092b55dbe4b1f  research/prompts/m3-close-c506-r3-sonnet-model/tools/rerun.sh
4b6d8e2fe67bfeec4e2be286ecd0d8533136d417c20203cd267dbf6158b620d4  crates/singlefs-core/src/make_filesystem.rs
428cbdfa553d8173521586025b174bc5822e7d43e49a1db48634bc428785a9ef  crates/singlefs-core/src/root_ring.rs
2d498cf43d8a23817dcfc79fa1328b29454336542a98dbe073f8c3d720e905ef  crates/singlefs-core/src/system_configuration.rs
332d17875b0c36e06653948ce63660528674483c183e4142e07b0830d871330a  crates/singlefs-checker/src/image.rs
acc136cabc833b9db977e18b73d30cb30862df416ed58ad4c633526df7da726f  crates/singlefs-format/src/lib.rs
bf63f8a3dab057ff7106b47211404e9b49b7fbba69527dae909f2e440dee5229  scripts/experiment-apparatus/e50_ring_slots.rs
```

模型会红的证明：设 `C506_R3_BREAK=rule_right_side_doubled`（读者新判的右边翻倍），同一份模型退出 1，5 条 claim 判 `match=false`（读者收的格数 9411 变 17463、与 mkfs 不等 9059 等），见 X3 末。

## 判定一览（先给表，推导在后）

判定取值：站得住（判决的打中成立）/ 够不着（打中碰不到被判形态）/ 同样打中别的臂（打中成立，但替代写法一样中，不分辨）。

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| r1 V1 依据段 E50 读错量 | 站得住；同样打中别的臂：旧条款与新条款共用 D22 第 83 行那一句，不分辨「格式常量 / 现算」。辩方补一条：对 D25 定的粗粒度档，那句「上界在 4..16 之外」其实对（精确越线 S ≥ 26，页上取样越线 N = 256），错的是没分档、页上结论句把 N 当 S | X3 e50 各行；X1 第 5 条 |
| r1 V1 `disk_stuck` 射程外 | 站得住（不打中被判对象，不分辨两臂） | 主 agent 判决第二节原样，辩方无异议 |
| r1 V2 两条边没登记 | 站得住 | `.claude/kb/layout/01-first-txn.md:140` 无标记；`:142` 只有 chunk 有标记，`:141` 的 P 没有（X2 第 4 条） |
| r1 V2 S ≤ 85 | 站得住 | X3 `pin_bound` 行 |
| r1 V2 D15 判定表缺「放宽读者取值域」一行 | 站得住；同样打中所有放宽上界的写法 | `.claude/kb/decisions/15-格式冻结政策.md:199` 判据是旧读者会怎样，九行里没有这一行（读九行所得） |
| r1 V3 S × 槽距：24/24 红、对照 96/96 | 站得住；同样打中所有不加读者新判的写法（含旧条款） | X3 `alias` 各行：我换成字节别名模型，73/73 个状态红、四条对照 0 红 |
| r1 V3 mkfs 不分盘判相叠（欠账） | 站得住；与 16 无关，不分辨臂 | X3 `shapes_over_rejection`：8052 重算对上 |
| r2 W1 P、chunk 读法不一致 | 站得住；钉成第一版常量是唯一不被攻方四格打中的写法 | X1 第 1 条；X2 第 1 条；`crates/singlefs-checker/src/image.rs:773-774` |
| r2 W1 错拒 0、26429 / 29744 逐形状相等 | 站得住（重算对上，口径见 X3） | X3 `shapes`、`claim` 各行 |
| r2 W2 min 公式撞下界 4 | 站得住（按今天的内层入口槽距是自由输入）；「小于 4 拒」有一个站得住的读法，但前提（槽距由 io_min 现算）条款没写；改「小于 5」不被同一格打中 | X1 第 2 条；X3 `min_formula`、`threshold`、`d2_19_conforming` |
| r2 W3 改字节造盘的射程 | 站得住（非阻塞） | 不涉被判形态，辩方未另验 |
| r2 W4「环深上界」 | 站得住，没有站得住的读法；换成「每区槽数上界 16 / 环长上界 3 × 16 = 48 个槽」后不再中 | X1 第 3 条 |
| r2 W4「焊死管理员回退」 | 站得住（有一个「上限」读法，被已定项 25 的坏词和定案句挡住）；同样打中任何保留「焊死」的写法 | X1 第 3 条 |
| r2 W4「GPU 根表进条款」 | 容量 48 拿出条款：站得住；「GPU 判器」留在读者名单里：站得住（I-7.13 行写了 GPU 判器同序同判） | X1 第 4 条 |
| r2 W4「冻结前随之抬；冻结之后 k_tol 不再抬」 | 站得住 | X1 第 6 条 |

## X1 「已定」项撑不撑得住

结论：被判形态里没有一句与它引到的已定项字面或量纲打架。有一处读法要点名（P、chunk 钉常量与「P 素数且 P > devs」，够不着）、两处措辞风险（不是冲突）。逐条：

1. **「P、chunk 与第一版常量不等一律整池拒」会不会撞「P 素数且 P > devs」：够不着。**
   - 条款确实有这句：`.claude/kb/decisions/22-单元原子性怎么合成.md:58`：「区域 r 落在 r × P × chunk，P 素数且 P > devs」（已定项 2 参数表第二行）。
   - 但同一已定项自己说这条约束已没有理由：`.claude/kb/decisions/22-单元原子性怎么合成.md:70`：「P 只剩「区域位置」这一个用途」，同一行：「约束本身还要不要，仓里一处都没写过」（欠账 C217，仍开着）。已定项 16 把话说死：`.claude/kb/decisions/22-单元原子性怎么合成.md:350`：「P = 3、chunk = 1 MiB 是第一版的 mkfs 常量」，同一行：「取 3 不依赖那条约束」。
   - 第一版只做两盘：`crates/singlefs-core/src/make_filesystem.rs:578`：「第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）」（`devices.len()` 断言为 2）；`.claude/kb/decisions/02-RAID条带策略.md:152`：「第一版不跑单盘，第一个可运行目标跑 2 块盘」。归属不靠 P 算：`.claude/kb/decisions/22-单元原子性怎么合成.md:351`：「根环三个区域的设备归属第一版写死 0 / 1 / 0」。所以「多盘时合法的 P 不是 3」在第一版不存在；P = 3 在两盘下还恰好满足 P > devs。
   - 同形先例就在同一已定项：`.claude/kb/decisions/22-单元原子性怎么合成.md:348`：「与第一版常量（64、1024）不等一律整池拒」，R 也是同一处置。P、chunk 加进来是同一个形状。
   - 残余（交主 agent 知道，不是这一句独有）：C217 若将来裁成「约束仍在」并开 3 盘以上，「第一版常量」这一句要跟着放宽，与 R = 3、基址、journal 起点同一类；放宽读者收的取值域落 D15 判定表缺的那一行（`.claude/kb/decisions/15-格式冻结政策.md:191` 九行里没有）。
   - 替代写法各自被打中的格（取自 r2 攻方报告 W1 节抄的 `out/probe.out` 原样行与改法表；那份 Rust 模型我没复跑，只读了报告）：读者按盘上 P × chunk 判 ⇒ chunk = 4 MiB、P = 5 两格判据值仍放行（checker 只读到区域 0 的根，一格不红）；core 也改成读盘上字段 ⇒ 要动 `crates/singlefs-core/src/root_ring.rs:96` 的区域起点（今天是常量）整条读路径；钉常量 ⇒ 四格都不中（r2 攻方自己标「推的：没实现」）。只有钉常量不被已有的反例打中。

2. **mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)、小于 5 拒：站得住；「小于 4 拒」的读法与它的前提。**
   - D2 已定项 19 的槽距取法：`.claude/kb/decisions/02-RAID条带策略.md:315`：「槽距 = 4096 向上取整到 mkfs 时探测的 io_min 的整数倍」；代码式子 `crates/singlefs-core/src/system_configuration.rs:248`：「floor.div_ceil(minimum_input_output_bytes) * minimum_input_output_bytes」。
   - 今天的内层入口槽距是调用方的自由输入：`crates/singlefs-core/src/system_configuration.rs:231`：「pub fixed_structure_slot_spacing: u32,」；`check_geometry` 里没有「槽距等于式子」这一判（`crates/singlefs-core/src/make_filesystem.rs:532` 起只有固定结构两两相叠）；已有用例故意传不走取整的槽距：`crates/singlefs-harness/tests/core_review_geometry_back_chain_and_empty_inode.rs:420`：「槽距 1024（没走 io_min 的取整）：系统配置两槽 [0, 4096) 与 [1024, 5120) 重叠」，以及 `crates/singlefs-harness/tests/core_review_geometry_back_chain_and_empty_inode.rs:439` 的 S = 8、槽距 384 KiB、io_min 512。
   - 站得住的读法（辩护）：「小于 4 拒」是读者下界的镜像（mkfs 不写读者不收的 S），「所写的值不取两条边」说的是按 D2 已定项 19 取槽距时实际写出的值。我重算这个前提下的集合：io_min 取 0 与 512..2 MiB 每 512 字节一档，被对齐判与相叠判收下的只有 12 档（0 与 512 到 512 KiB 的 2 的幂），写出的 S 只有 {8, 6}，6 只出现在 io_min = 512 KiB；io_min = 1 MiB 时公式给 3，小于 4、小于 5 两个门槛一样拒。所以那一支不是「一次都走不到」（r2 攻方的说法只对被收下的 io_min 成立）：它在 io_min ≥ 1 MiB 上走得到，正是 D2 已定项 19 那句「io_min 大到固定结构互相重叠的设备 mkfs 拒」的构造入口版（`.claude/kb/decisions/02-RAID条带策略.md:317`：「io_min 大到固定结构互相重叠的设备 mkfs 拒、不支持」）。
   - 这个读法的前提（槽距由 io_min 现算）条款没写、内层入口不强制，所以 r2 的打中在今天的入口上成立：槽距 629248..786432 公式写 4 而 mkfs 收 S = 4（X3 `min_formula`）。**够得着。**
   - 替代写法：改「小于 5」不被这一格打中（X3 `threshold` 行 `refuse_below=5 … touches_edge_4=False`），代价是 mkfs 拒掉读者收的 S = 4 几何，而这些几何在 D2 已定项 19 取法下对齐判先拒，写不出，所以没有实际代价。另一条「mkfs 另判槽距 = 式子」同样不被打中，但会撞上面那两条故意传自由槽距的现有用例，且 77 处直接调用里槽距实参的种类是 35 处写 4096、15 处 `slot_spacing_for(512)`、7 处别的 `slot_spacing_for(`（我 `grep -rhn 'fixed_structure_slot_spacing:'` 重数对上）。辩方认为 r2 取「小于 5」是代价更小的那条，不翻。
   - 一处与 D2 已定项 19 的连带（X2 第 2 条）：同一行写的是「S = 8 时槽距 384 KiB 收、512 KiB 起拒」（`.claude/kb/decisions/02-RAID条带策略.md:317`），公式一进来 io_min = 512 KiB 的盘 mkfs 写 6、收下，那一句要改。

3. **「环深上界」与「焊死管理员回退」：打中站得住，没有站得住的读法；换成的写法不再中。**
   - 环深：同一格前半句（被判形态不动它）写着：`.claude/kb/decisions/22-单元原子性怎么合成.md:60`：「出过一次事，这里不用它」（指「环深」这个词，C131）。D16 的同名格量的是 S：`.claude/kb/decisions/16-发布语义.md:46`：「每区槽数 S ≥ k_tol + 2 = 4」。原句「环深上界 3 × 16」量的是 R × S。想把它读成 D16 那种口径（环深 = S）只能读成「环深上界 16」，而「3 × 16」就是另一个量，两种读法都过不了。被判形态写「每区槽数上界 16」与「环长上界 3 × 16 = 48 个槽」，没有用那个词，量纲各自一致。一个小提醒：「环长」在 D22 里还指 journal 环（`.claude/kb/decisions/22-单元原子性怎么合成.md:348`：「环长现算的起点」），根环的 3S 也叫环长（`.claude/kb/decisions/22-单元原子性怎么合成.md:150`：「小于环长 3S」）；被判形态靠「个槽」的单位区分，够用，不构成冲突。
   - 焊死：站得住的读法只有「48 是各池 3S 的上确界」这一个事实（E92 记着管理员回退用整个根环：`.claude/kb/decisions/22-单元原子性怎么合成.md:507`：「管理员回退要整个根环深度」），但已定项 25 的定案句把「焊死」点成坏词：`.claude/kb/decisions/22-单元原子性怎么合成.md:495`：「而格式常量把一个纯可用性参数焊死，没有任何好处」，管理员回退能退多深正是每池按 F 和自己的 3S 现定的量（`.claude/kb/decisions/16-发布语义.md:41`：「回退候选集 | 按现行那一版的实例表判仍然有效」，`.claude/kb/decisions/16-发布语义.md:37`：「平时不动，窗口就是整个根环」）。被判形态改写成「一个池上管理员回退能退到哪里由回退候选集（txg ≥ F_生效）与这个池自己的 3S 定」，与这三处逐字对得上。同样打中别的臂：任何保留「焊死」或「环深」的候选句都中同一格。

4. **GPU 根表：容量 48 不进条款站得住；「GPU 判器」留在读者名单里也站得住。**
   - 名单：`crates/singlefs-checker/invariants/02-根环.md:20`：「GPU 判器走读内核」（I-7.13 行写 GPU 判器同序同判），所以被判形态把 GPU 判器列成读者有出处。
   - 容量：`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:241`：「const MAX_ROOTS: u32 = 48u;」超容时 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:1316`：「undecidable = UNDECIDABLE_TOO_MANY_ROOTS;」，是失败即关的实现容量；同批内核另有候选根表 `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:567`：「const C_MAX_CANDIDATES: u32 = 48u;」，harness 里还有 48 位的 u64 位图 `crates/singlefs-harness/src/fault_injection.rs:212`；点名一张表的条款会漏另外两处，进格式就得全列。格式层面真有的上界约束只有钉槽 p 一字节：`.claude/kb/decisions/22-单元原子性怎么合成.md:150`：「钉值 = 3·槽 + 区域」，同一行：「没有钉着的槽写 255」。同样打中别的臂：点名 GPU 表的写法一律中同一格。
   - 补一处落点：S 区间的字面量在 GPU 有三处（我 grep 的原样输出见 X3 `grep_wgsl_s_interval_and_48`），第三处 `crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:906` 只在「自证过」那一层（`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:896`：「recovery::system_configuration_slots_of_one_device_read_once 收进 self_verified 的那一判」），不是新判的落点；新判落 `parse_system_configuration`（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:943`）与 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的读槽函数两处。r2 判决的落点表写两处，对。

5. **依据段 E50 那一行：新写法站得住；旧句有一个被判决漏看的站得住的读法。**
   - 新写法的数：粗粒度 S = 25 是 975 块、S = 26 是 1014 块，散开 S = 9 是 918、S = 10 是 1020（X3 `e50_values`）。E50 自己的源码就写着：`scripts/experiment-apparatus/e50_ring_slots.rs:167`：「粗粒度下要 77 槽才够到 1000 块这条线」，`scripts/experiment-apparatus/e50_ring_slots.rs:168`：「散开下 30 槽就够到」，77 ÷ 3 向上取整 26、30 ÷ 3 = 10，与新句逐数一致，不是 r2 腿「推的」，源码注释里就有。
   - 旧句 `.claude/kb/decisions/22-单元原子性怎么合成.md:83`：「它量出的 S 下界与上界都在 4..16 之外」：对 D25 定的粗粒度档，上界在 S ≈ 26 以外（页上取样越线 N = 256 即 S ≈ 85），确在 16 之外；页上 `.claude/kb/experiments/50-根环槽数的上下界.md:92`：「⚠️ 散开负载下 32 槽就够到了」才是它漏的一档，页上结论 `.claude/kb/experiments/50-根环槽数的上下界.md:118`：「S 在 1..16 这个区间里怎么取都不碰任何一条边」把 N 当 S 写，r1 的打中打在这里。所以 r1 的「读错量」打中成立，但打到的是没分档的措辞，不是「16 在界外」的方向；旧条款（挂载时现算）引同一行，同样中，不分辨两臂。

6. **D16「冻结前随之抬；冻结之后 k_tol 不再抬」：站得住。**
   - k_tol 不在格式里，是实现常量：`crates/singlefs-core/src/mount.rs:4347`：「一次准入最多推几次发布：B = 4 + 2 k_tol，k_tol = 2 ⇒ 8」。D15 冻结后下界 4 抬不动：`.claude/kb/decisions/15-格式冻结政策.md:171`：「已冻结字段的宽度 / 存在性 / 含义不能再改；只能新增」；D16 的保证本来就带射程：`.claude/kb/decisions/16-发布语义.md:60`：「最少保留 4 个状态只在一次处置里根槽写失败 ≤ k_tol 次时兑现」。S = 4 的池只兑现 k_tol ≤ 2，所以冻结后抬 k_tol 的承诺对已有 S = 4 池兑现不了，补这一句是如实写，不是新约束。

7. **两处措辞风险（不是字面冲突，交主 agent 看是否顺手改）。**
   - mkfs 公式进了格式格：被判形态不动那一格后半句，`.claude/kb/decisions/22-单元原子性怎么合成.md:60`：「具体取多少不是格式决策」（同一行：格式承诺的是「S 是系统配置字段 + 挂载时判区间」）。公式 min(8, …) 放在它前面，D15 冻结后又有 `.claude/kb/decisions/15-格式冻结政策.md:173`：「已冻结组件的选取规则本身也被钉死」。读法一：公式只是第一版 mkfs 的取法；读法二：格式承诺。建议写一句「这是第一版 mkfs 的取法，不是格式承诺」，并把那句「格式承诺的是……」补上「及 S × 槽距 ≤ P × chunk」，否则它与新判并排读起来少了一半。
   - 「上界是读者承诺收的最大值」之后紧接着有乘积判，16 只在槽距 ≤ 192 KiB 时收得下（3 MiB ÷ 16 = 192 KiB）；建议写成「在 S × 槽距 ≤ P × chunk 的前提下」。

## X2 未定项清单的完整性

范围：r2 判决第三节「实现欠」「另登欠账」两张表加 r1 判决第三节「另登欠账」。下面是我现查、两张表里没有（或只写了一半）的落点；每条写在哪、凭什么。表里已有而我核过落点还在的不重列：读者三处与 I-7.13、mkfs 门槛、harness 模型 `crates/singlefs-harness/src/model.rs:1815` 的 S 取常量、`crates/singlefs-format/src/lib.rs` 两条 placeholder、C506 行、`root_ring.rs` 里引「已定项 1」的消息（`crates/singlefs-core/src/root_ring.rs:52`）都在。

| 漏项 | 现查证据 | 两张表里有没有 |
|---|---|---|
| 层 layout 一第 45 行：S 的上界写成「由 E50 给出」 | `.claude/kb/layout/01-first-txn.md:45`：「上界由 E50（根环槽数的上下界） 给出）」。新的依据段那一句恰恰说 16 不是 E50 量出的上界，两句并排对不上 | 没有（表里只列了 layout 一第 140 行的登记位） |
| D22 已定项 16 的读者名单与「已实现」段、I-7.13 行的项数 | `.claude/kb/decisions/22-单元原子性怎么合成.md:348`：「与第一版常量（64、1024）不等一律整池拒」是 P、chunk 钉常量的同形先例，名单与「已实现」段不动就落后；`.claude/kb/decisions/22-单元原子性怎么合成.md:352`：「读者判固定结构槽距：≥ 4096，且槽 1 整槽落在同一槽自述的根环起点之前」少了乘积判；`crates/singlefs-checker/invariants/02-根环.md:20` 写「共八项」；两边同表的用例 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 是五格一表 | 只写了「挂 I-7.13、越界用例」，没写已定项 16 本身要改 |
| 既有用例与已有判会因新判的位置变红或变死 | `crates/singlefs-harness/tests/system_configuration_slots_per_region.rs:182` 的用例把 P、chunk 都顶满 u32，期望 `Some(Verdict::RootRingSlotOffsetOverflowsTheOffsetSpace),`（`:193`）；checker 里那一判在 `crates/singlefs-checker/src/image.rs:786` 且排在 P、chunk 读出之后。若等于常量的新判按 r2 说的放在槽距判（`crates/singlefs-checker/src/image.rs:750`）之后，这一格先吃到新成员，用例要改，溢出那一判随之成死码。要不要留作纵深防御，两张表没提 | 没有 |
| 常量标记不对称 | `.claude/kb/layout/01-first-txn.md:142` 的 chunk 带 `format-const: ROOT_RING_CHUNK_BYTES`，`.claude/kb/layout/01-first-txn.md:141` 的 P 没有，`crates/singlefs-format/src/lib.rs:311` 的 `ROOT_RING_PRIME_STEP` 也没有；读者一旦把 P 钉成常量，P 该和 chunk 同样登记。GPU 恢复内核另有字面量 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:113`：「const ROOT_RING_REGION_STRIDE: u32 = 3145728u;」（= P × chunk），门禁只扫 `.rs`，钉不住 | 只列「两条边」的标记 |
| D2 已定项 19 与 C568 的「512 KiB 起拒」 | `.claude/kb/decisions/02-RAID条带策略.md:317`：「S = 8 时槽距 384 KiB 收、512 KiB 起拒」；mkfs 一写公式，io_min = 512 KiB 的盘写 6、收下（X3 `d2_19_conforming`），这一句和 C568 行（`.claude/kb/checks-owed.md:841` 的 C568 行）的措辞要跟着改；io_min = 1 MiB 仍拒（X3 `io_min_1MiB`），C568 的十三条用例不受影响 | 没有 |
| 引旧句的别处 | `crates/singlefs-core/src/recovery.rs:501` 的文档注释引「上界由 I-7.4 扣住的块数挂载时算」；`.claude/kb/prior-art.md:659` 同引；`records/2026-09-28-里程碑三第十项清单.md:336` 的 C506 行同引（records 是历史记录，是否回头改归主 agent）；S 格的「具体取多少不是格式决策——格式承诺的是……」被抄在三处：`crates/singlefs-format/src/lib.rs:296`、`crates/singlefs-core/src/root_ring.rs:19`、`crates/singlefs-harness/tests/system_configuration_slots_per_region.rs:2`，格里那句补了「及 S × 槽距」之后这三处跟着 | 表里只有 lib.rs 的 placeholder 与 `root_ring.rs` 的「已定项 1」消息 |
| 三语译文 | 同一批文件都有 `.en.md` / `.ja.md`：D22（第 60、83 行所在的那两格）、D16 第 46 行那格、D2 第 317 行、layout 一第 45 与 140 行、`checks-owed` 的 C506 行与新欠账；I-7.13 所在的 `crates/singlefs-checker/invariants/02-根环.md` 没有译文（`ls` 只有中文） | 没有 |
| 实验页「影响的决策」回看 | 列了 D22 已定项 2 的实验页有 26 份（`grep -rln 'D22.*已定项 2' .claude/kb/experiments/*.md` 去掉 `.en` `.ja` 后数出 26）。真受影响的是 E50：`.claude/kb/experiments/50-根环槽数的上下界.md:118` 的结论句「S 在 1..16 这个区间里怎么取都不碰任何一条边」还写着，页尾 D22 已定项 2 一行回看写的是「支撑」，现在只支撑「S 不是一个要精算的量」；E139 页的 `disk_stuck` S = 16 一行（r1 已记射程外）建议回看写一句「S 上界改成格式常量，不分辨」。其余 24 份本轮不碰 S 的值，回看写「不受影响」 | 没有 |
| 编译期断言的运行时对应物 | `crates/singlefs-core/src/root_ring.rs:44`、`:50` 的断言只罩常量 8；公式的结果是运行时值，要新入口（r2 正推已写）。另外：77 处直接调用不用动（上限 8 不变），但 `crates/singlefs-core/src/make_filesystem.rs:578` 的两盘断言后面谁来调新入口（对外入口还是只给装置）两张表没定 | 部分（写了新入口，没写谁调） |

**「mkfs 写 min(8, …)、小于 5 拒」在 D2 已定项 19 的槽距取法下还剩什么取值。** 我的模型（X3 `d2_19_conforming`、`threshold`）：槽距按 4096 向上取整到 io_min 的整数倍，io_min 取 0 与 512 到 2 MiB 每 512 字节一档，被 mkfs 的相叠判和对齐判收下的 12 档全是 0 或 2 的幂，最大 512 KiB；写出的 S 只有 8（io_min ≤ 256 KiB，含不到 4096 的）和 6（io_min = 512 KiB）。io_min = 1 MiB 公式给 3，被「小于 5」拒（「小于 4」也拒）。所以「小于 5」在这个取法下只多拒了一类槽距（629248..786432），而这一类对齐判先拒，没有收下的盘受影响；公式的 5 和 7 只在槽距自由取值时才会写出（X3 `threshold` 行：自由槽距上写出 {5, 6, 7, 8}，不取 4 也不取 16）。**什么现象会推翻这条：** 对齐判放宽到收非 2 的幂 io_min（那时 4 与 5 会在 D2 取法下出现），或槽距取法改成不是 io_min 的整数倍。

## X3 前两轮报出的数：逐个独立复核

办法（都独立于原腿，不拷前两轮的模型）：
- **换模型**：纯 Python 重写。r1 的 24/24、96/96 改成「按字节别名重算」——不挂载，只摆根槽的物理字节段，找同盘上相交的两个（区域, 槽），再逐个 txg 状态按 I-7.17 的两种读法判（一条根在物理槽里，按它所在的每个（区域, 槽）核「区域 = txg mod 3 且槽 = (txg div 3) mod S」，第 0 代只核槽号，见 `crates/singlefs-checker/invariants/02-根环.md:24`：「checkpoint_txg == 0」）。r2 的 26429 / 29744 格改成「读 `make_filesystem.rs` 的 `fixed_structure_extents` 与 `check_geometry` 后另写的六段两两比（`crates/singlefs-core/src/make_filesystem.rs:532`：「if first_in_layout_order.overlaps(*second_in_layout_order) {」）」对「S × 槽距 ≤ 3 MiB」逐格比，再按 0 / 1 / 0 归属逐盘算真叠的字节，用区域整段、只算写过的字节、只算根槽宽三种口径。
- **换算式**：S ≤ 85（3S − 1 < 255）、E50 越线（块数常量从 E50 源码读，13 与 34）、槽距 629248..786432 写 4（⌊3 MiB ÷ 槽距⌋ = 4）、有效槽距上界 768 KiB（3 MiB ÷ 4）。
- **grep 计数**：77 处直接调用与 44 个文件。

局限：mkfs 那一判是我读源码另写的 Python，不是编译出来的 mkfs；r2 攻方是用入库 mkfs 真跑的，所以两边相等说明「我读的与他跑的一致」，不说明我读对了 mkfs 以外的东西。别名模型共用前两轮的前提（区域归属 0 / 1 / 0、区域起点 1 / 4 / 7 MiB、根槽宽 = pbs、journal 环 16 MiB 起），不是三条互不共享前提的路径；本轮没有两边互调的结论，不需要。

| 数（出处） | 我的办法 | 结果 |
|---|---|---|
| r1 攻方：S = 16、512 KiB，24 个挂载状态 24 / 24 判 I-7.17 红 | 字节别名 + 逐 txg 状态判 | 核上：别名 4 对（区域 0 的槽 12..15 = 区域 2 的槽 0..3），txg 0..72 共 73 个状态 73 / 73 红，r1 报的几个选到的根 txg（2、37、46、70）与另外 20 个状态共 24 个都红。r2 攻方多报的那一格（一次都没挂就红）我的 txg 0 也红 |
| r1 攻方：四条对照臂 96 / 96 干净 | 同上，四臂各 24 状态 | 核上：四臂别名 0 对，96 / 96 干净 |
| r1：S ≤ 85 | 3S − 1 < 255，哨兵值从源码读（255） | 核上：S = 85 钉值 254，S = 86 钉值 257 |
| r1：粗粒度 S ≤ 25 不过 1000 块线、散开 S ≥ 10 过线 | 块数常量与线从 E50 源码读，现算 | 核上：S = 25 是 975、S = 26 是 1014；S = 9 是 918、S = 10 是 1020；精确越线 N = 77 与 30 |
| r2：26429 个 (S, 槽距) 形状 | 2033 档槽距（4096 到 1044480，每 512）× 13 | 核上：26429；pbs = 4096 一档 255 × 13 = 3315，合计 29744 |
| r2：读者新判与 mkfs 逐形状相等、错拒 0 | 另写的六段两两比 | 核上：9411 / 9411、1184 / 1184，两边只一边收的形状 0 个，错拒 0 |
| r2：罩不住的相叠 0 | 三种口径逐盘算真叠 | 核上：区域整段、只算写过的字节、根槽宽三种口径下「新判收而真叠」都是 0 |
| r2 正推：8966 / 8052 / 7696（18733）| 同上的三种口径 | 核上：区域整段真叠 8966、新判拒而真不叠 8052、写过字节口径真叠 7696；r2 攻方 13784 与 1717（根槽宽口径）也核上 |
| r2：槽距 629248..786432 写 4 | 换算 | 核上：⌊3 MiB ÷ 槽距⌋ = 4 的 512 倍数槽距恰是 629248 到 786432；pbs = 4096 是 630784 起 |
| r2：有效槽距上界 768 KiB | 换算 | 核上：3 MiB ÷ 4 = 786432 = 768 KiB；`.claude/kb/decisions/22-单元原子性怎么合成.md:74` 的 1 MiB − 4096 在 S ≥ 4 时用不满 |
| r2：77 处直接调用 | grep 计数 | 核上：77 处、44 个文件 |

没重核（要盘或要 Rust）：r2 攻方「用过的盘上两种造法差 5 个扇区」「I-7.7 红」，P、chunk 各格的 checker 红与否，GPU u32 绕回（推的），r1 的 E139 `disk_stuck` 产物行（主 agent 已逐字核过）。**核不上的：无。**

模型输出全文（`research/prompts/m3-close-c506-r3-sonnet-model/out/model.out`，复跑 `same=1`）：

```text
name=source_check result=ok S_min=4 S_max=16 S_mkfs=8 P=3 chunk=1048576 R=3 base=1048576 journal=16777216 syscfg=4096 sentinel=255 sp_min=4096 stride=3145728
name=alias arm=rewritten_S16_512KiB S=16 spacing=524288 aliased_pairs=4 red_states=73_of_73 subset24_red=24_of_24
name=alias arm=S16_4KiB S=16 spacing=4096 aliased_pairs=0 red_states=0_of_73 subset24_red=0_of_24
name=alias arm=S4_512KiB S=4 spacing=524288 aliased_pairs=0 red_states=0_of_73 subset24_red=0_of_24
name=alias arm=S16_256KiB S=16 spacing=262144 aliased_pairs=0 red_states=0_of_73 subset24_red=0_of_24
name=alias arm=S8_512KiB S=8 spacing=524288 aliased_pairs=0 red_states=0_of_73 subset24_red=0_of_24
name=alias_control_clean_states_per_24x4 clean=96 expected_if_96_of_96=96
name=pin_bound max_pin_value_formula=3S-1 largest_S_with_pin_below_255=85 pin_at_85=254 pin_at_86=257
name=e50_constants coarse_blocks=13 scattered_blocks=34 pinned_matters=1000
name=e50_crossing coarse_first_S_reaching_line=26 (S_le_25_does_not) scattered_first_S_reaching_line=10 coarse_exact_N=77 scattered_exact_N=30
name=e50_values S25_coarse=975 S26_coarse=1014 S9_scattered=918 S10_scattered=1020 S16_scattered=1632 S16_coarse=624
name=e50_page_row first_sampled_N_coarse=256 first_sampled_N_scattered=32 N8_coarse=104 N8_scattered=272 N64_coarse=832 N64_scattered=2176
name=shapes pbs=512 spacings=2033 shapes=26429 rule_accepts=9411 mkfs_accepts=9411 rule_xor_mkfs=0 false_rejects_vs_mkfs=0 rule_accepts_but_extent_overlap=0 rule_accepts_but_slot_overlap=0
name=shapes_physical pbs=512 extent_overlap_shapes=8966 slot_overlap_shapes=3234 rule_refuses=17018 rule_refuses_and_no_slot_overlap=13784 trimmed_overlap_shapes=7696 rule_accepts_but_trimmed_overlap=0
name=shapes_over_rejection pbs=512 rule_refuses_but_extent_clean=8052 rule_refuses_but_trimmed_clean=9322 rule_accepts_exactly_touching=5
name=shapes pbs=4096 spacings=255 shapes=3315 rule_accepts=1184 mkfs_accepts=1184 rule_xor_mkfs=0 false_rejects_vs_mkfs=0 rule_accepts_but_extent_overlap=0 rule_accepts_but_slot_overlap=0
name=shapes_physical pbs=4096 extent_overlap_shapes=1124 slot_overlap_shapes=414 rule_refuses=2131 rule_refuses_and_no_slot_overlap=1717 trimmed_overlap_shapes=967 rule_accepts_but_trimmed_overlap=0
name=shapes_over_rejection pbs=4096 rule_refuses_but_extent_clean=1007 rule_refuses_but_trimmed_clean=1164 rule_accepts_exactly_touching=5
name=shapes_total total=29744
name=min_formula pbs=512 histogram=[(3, 504), (4, 308), (5, 204), (6, 147), (7, 109), (8, 761)] spacing_giving_4=629248..786432 spacing_giving_below_4=786944..1044480 any_16=False
name=min_formula pbs=4096 histogram=[(3, 63), (4, 39), (5, 25), (6, 19), (7, 13), (8, 96)] spacing_giving_4=630784..786432 spacing_giving_below_4=790528..1044480 any_16=False
name=bound_arithmetic S4_largest_spacing=786432=768KiB real_4_interval_bytes=629146..786432 KiB_of_upper=768.0 reader_range_top=1044480
name=threshold refuse_below=4 written_values_over_free_spacing=[(4, 308), (5, 204), (6, 147), (7, 109), (8, 761)] touches_edge_4=True touches_edge_16=False
name=threshold refuse_below=5 written_values_over_free_spacing=[(5, 204), (6, 147), (7, 109), (8, 761)] touches_edge_4=False touches_edge_16=False
name=d2_19_conforming pbs=512 io_min_range=0..2097152 accepted_io_min_count=12 accepted_all_power_of_two_or_zero=True largest_accepted_io_min=524288 written_S_values=[6, 8] io_min_giving_6=[524288] written_S_equals_4_or_below_among_accepted=False
name=d2_19_conforming pbs=4096 io_min_range=0..2097152 accepted_io_min_count=12 accepted_all_power_of_two_or_zero=True largest_accepted_io_min=524288 written_S_values=[6, 8] io_min_giving_6=[524288] written_S_equals_4_or_below_among_accepted=False
name=io_min_1MiB spacing=1048576 formula_gives=3 refused_by_threshold_4=True refused_by_threshold_5=True mkfs_extent_overlap_at_S4=True reader_range_top=1044480 spacing_over_range=True
name=grep_at_make_filesystem direct_calls=77 files=44
name=grep_reader_of_P_chunk file=crates/singlefs-core/src/recovery.rs hits=[]
name=grep_reader_of_P_chunk file=crates/singlefs-core/src/system_configuration.rs hits=[(12, 'ROOT_RING_CHUNK_BYTES, ROOT_RING_PRIME_STEP, ROOT_RING_REGIONS, SLOT_BYTES,'), (593, 'writer.put_u32(u32::try_from(ROOT_RING_PRIME_STEP).expect("P"));'), (594, 'writer.put_u32(u32::try_from(ROOT_RING_CHUNK_BYTES).expect("chunk"));')]
name=grep_reader_of_P_chunk file=crates/singlefs-checker/src/image.rs hits=[(773, 'let prime_step = u64::from(read_u32(slot, 363));'), (774, 'let chunk_bytes = u64::from(read_u32(slot, 367));')]
name=grep_wgsl_s_interval_and_48 sites=[('crash_judge_checker.wgsl', 111), ('crash_judge_checker_common.wgsl', 567, '48u'), ('crash_judge_common.wgsl', 241, '48u'), ('crash_judge_common.wgsl', 960), ('crash_judge_recovery.wgsl', 906)]
name=claim r1_alias_24_of_24_red reported=24/24 computed=24/24 match=true
name=claim r1_controls_96_of_96_clean reported=96/96 computed=96/96 match=true
name=claim r1_S_le_85 reported=85 computed=85 match=true
name=claim r1_coarse_S_le_25_not_over_line reported=25 computed=25 match=true
name=claim r1_scattered_S_ge_10_over_line reported=10 computed=10 match=true
name=claim r2_shapes_pbs512 reported=26429 computed=26429 match=true
name=claim r2_shapes_total reported=29744 computed=29744 match=true
name=claim r2_rule_accepts_pbs512 reported=9411 computed=9411 match=true
name=claim r2_rule_accepts_pbs4096 reported=1184 computed=1184 match=true
name=claim r2_rule_xor_mkfs_total reported=0 computed=0 match=true
name=claim r2_false_rejects_total reported=0 computed=0 match=true
name=claim r2_uncovered_overlap_total reported=0 computed=0 match=true
name=claim r2_extent_overlap_pbs512 reported=8966 computed=8966 match=true
name=claim r2_refused_no_slot_overlap_pbs512_pbs4096 reported=13784+1717 computed=13784+1717 match=true
name=claim r2_over_rejection_extent_pbs512 reported=8052 computed=8052 match=true
name=claim r2_trimmed_overlap_pbs512_is_26429_minus_18733 reported=7696 computed=7696 match=true
name=claim r2_spacing_giving_4_from reported=629248 computed=629248 match=true
name=claim r2_spacing_giving_4_to reported=786432 computed=786432 match=true
name=claim r2_effective_spacing_upper_KiB reported=768 computed=768 match=true
name=claim r2_at_make_filesystem_direct_calls reported=77 computed=77 match=true
name=claim r2_at_make_filesystem_files reported=44 computed=44 match=true
name=claims_summary claims=21 mismatched=0
```

弄坏开关的原样输出（设 `C506_R3_BREAK=rule_right_side_doubled`，`grep -E 'match=false|claims_summary'`，退出码 1）：

```text
name=claim r2_rule_accepts_pbs512 reported=9411 computed=17463 match=false
name=claim r2_rule_accepts_pbs4096 reported=1184 computed=2191 match=false
name=claim r2_rule_xor_mkfs_total reported=0 computed=9059 match=false
name=claim r2_refused_no_slot_overlap_pbs512_pbs4096 reported=13784+1717 computed=5732+710 match=false
name=claim r2_over_rejection_extent_pbs512 reported=8052 computed=0 match=false
name=claims_summary claims=21 mismatched=5
```

## 每格结论与推翻条件

- **X1 结论：被判形态与它引的已定项（D22 已定项 2、16、25，D16 已定项 1，D15 已定项 7、8、9，D2 已定项 19，I-7.13、I-7.17）没有字面或量纲上的冲突；P、chunk 钉第一版常量不撞「P 素数且 P > devs」；r1、r2 的打中凡成立的都够得着，且替代写法里只有被判形态（加上「小于 5」）不再被已有反例打中。** 推翻它的现象：① 找到一份条款要求第一版 mkfs 在 3 盘或更多盘上选 P ≠ 3（我读 D22 第 58、70、350 行、D2 第 99、152 行没有，`make_filesystem.rs` 断言两盘）；② 有读者之外的写者合法写出 P ≠ 3 或 chunk ≠ 1 MiB 的池（我 `grep` 写系统配置的两行只见常量）；③ 模型之外找到一个 S × 槽距 ≤ 3 MiB 而同盘根槽字节相交的形状（X3 `rule_accepts_but_*` 三个口径都是 0）。
- **X2 结论：两张表漏了八类落点（表中第一列），其中 layout 一第 45 行「上界由 E50 给出」与新依据句直接对不上，D22 已定项 16 的名单与 I-7.13 行没改，既有溢出用例会变红或变死，D2 已定项 19 的「512 KiB 起拒」会过期。** 推翻它的现象：门禁或主 agent 现查发现这些行已在随这一改动的补丁里（我查的是主工作区此刻的文件，不是补丁）。
- **X3 结论：前两轮报出的 11 个数（含它们的子数）全部用独立办法核上，核不上 0 个。** 推翻它的现象：换一份与主工作区不同的冻结副本重跑，`name=rerun_compare same=0`；或 `make_filesystem.rs` 的相叠判与我读到的不同（那时我的 mkfs 一侧作废，r2 攻方用真 mkfs 的数仍在）。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果。
- 没编译 Rust、没造盘、没跑任何重型测试与门禁阶段；mkfs、读者三处的判法都是读源码另写的 Python，不是跑实现。
- 没重跑 r1、r2 两条腿的 Rust 模型（要编译）；它们的原样行只读了报告。
- 没核 E139 产物与 E50 页的原跑数，只核了 E50 源码常量与注释。
- 没做 s2 抽样：本报告的「核上」是一次独立办法的观测，「站得住」是读条款和代码的推导。
- 没读禁读清单里的三份（r3 攻方产出、攻方模型目录、主 agent 核验文件）。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-c506-r3-sonnet/`：`tree/`（即 `/home/fy5090/.local/share/singlefs/work/m3-close-c506-r3-sonnet/tree`）是冻结副本，37 MiB，只含 `crates/` 与 `scripts/experiment-apparatus/e50_ring_slots.rs`，复跑的 `--frozen` 指它，所以不删；`scratch/` 只有脚本与草稿；复跑用的 `rerun1/` 已删；没建编译目录、没建仓副本。
