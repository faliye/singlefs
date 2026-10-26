# m4-exit5-relocation-proof-r3 判决：出口 5 全验那一形（主 agent，2026-10-26，第三轮，最后一轮）

<!-- doc-lint:not-numbers Z1 Z2 Z3 Z4 Z5 G7 P4 -->

同一题的第三轮，也是最后一轮。前两轮判决 `research/prompts/m4-exit5-relocation-proof-r1-main-verification.md`、`research/prompts/m4-exit5-relocation-proof-r2-main-verification.md`。用户 2026-10-26 定层「core 直验，连胶水一起证」、定「里程碑四全验」。这一轮之后不再开第四轮；这一轮里新冒出的零轮形态写进第四节交用户表、标「零轮」。

两条腿：辩方 `research/prompts/m4-exit5-relocation-proof-r3-sonnet-output.md`（sha256 0e2d7904650f9c5e6f35a52d937988b2e6524e69f98a7c7b56b5f295e23cb9de，模型目录 `research/prompts/m4-exit5-relocation-proof-r3-sonnet-model/`），攻方 `research/prompts/m4-exit5-relocation-proof-r3-opus-output.md`（sha256 7e5b7328b7be47a584eaeb8718edda44b04edcc6cb98808c8e3d3035193d87a7，模型目录 `research/prompts/m4-exit5-relocation-proof-r3-opus-model/`）。

## 一、核查

- 开工快照：`python3 scripts/agents/snapshot-tree.py m4-exit5-relocation-proof-r3 --out /tmp/claude-1000/-home-fy5090-code-singlefs/361bb28f-a369-4461-967b-925319d2cf5d/scratchpad/x5r3/tree`，「清单点名 2 份，倒推出 2 份，sha256 对上 2 份」。
- cite-check：`python3 scripts/agents/cite-check.py research/prompts/m4-exit5-relocation-proof-r3-sonnet-output.md research/prompts/m4-exit5-relocation-proof-r3-opus-output.md --root <上一条的树> --root /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --root /home/fy5090/code/singlefs --background research/prompts/_m4-exit5-relocation-proof-r3-background.md`，末行「✓ 核了 15 处引文、60 处裸行号，对上 15 处，没判 60 处（其中裸行号没有引文可比 21 处；逐处列在上面）」，没有对不上的。
- 两条腿模型复跑：见第五节。

## 二、逐格判定

| 格 | 两条腿 | 判定 |
|---|---|---|
| Z1 改后协议 | 两腿一致：第二轮正推的改后协议是两份不组合的模型——根环版有回退候选但只按起点槽号护，区间版 P4 有跨度但没有回退候选这一维；变体一（写进回退目标还在读的槽）被根环协议的 `Write` 门排除，变体二（两槽单元的第二槽）排除不了（攻方 Verus 证出、两份必红各红 1；辩方同）。辩方把根环协议提升成区间（`verus/gc_ring_runs.rs`）后排除变体二，4 verified、三个必红世界都红；攻方提的补法同形（根环协议按区间、映射值带跨度） | **采「根环协议按区间」的形**（两腿各自独立得到，标零轮）；这一格照第二轮判决「两个变体都排除才算关上」现在关上，以入库实现上 Verus 复验为准 |
| Z1 G7 改写 | 辩方：「逐项相同」与「恰多移 k 格」都不成立；按版本号点名时，现行内容相同、回退候选是不整理那份的后缀、至多少 k 个、抬 F 上限恰多 k；全穷举（用户步长度到 5）约 1887 万次运行加随机抽样约 413 万次反例为 0；按内容点名会出 8 个假反例 | **采辩方的措辞**：G7 改成「整理不改现行内容；回退候选集是不整理那份的后缀、至多少 k 个，抬 F 的上限恰多 k（k 为这一段整理占的发布数），按版本号点名」 |
| Z1 前提在 core 里成立吗 | 两腿一致：`set_reuse_window` 拨到 `ForcedToZero` 绕过 F（只供测试、无产品调用者，但它是 pub）。辩方另指两条：「先内存抬 F、持久后才放开复用」那一条协议里没有、靠 core 的扣住机制满足；「门槛蕴涵没被引用」（`gc_crash_proofs.rs:70`）没人证 | **三条都进证明义务**：只供测试的开关挪进默认关的 cargo 特性（与 m4-publish-trigger-r1 判决 P6 同一个做法）；协议补「内存 F / 持久 F」两层；「门槛蕴涵没被引用」写成引理 |
| Z2 信任清单与门禁 | 两腿一致：去掉 `--no-cheating`、信任清单、`axiom` 不许这几条站得住；差分测试判得出写错的 std 规格（辩方：`binary_search` 写成「第一个下标」，360 例红 168 例，真规格 0 例红）。两腿打中：清单与门禁漏 feature、`debug_assertions`、宏里的 cfg 分叉与 `verus!` 外面的 trait impl（Verus 验过、产品违 ensures，辩方用产品构建复现）；「按 `verus_keep_ghost` 分叉判红」只挡一个 cfg 原子；第二轮的 MIR 比对（两次产品编译）看不见 cfg 分叉——宏写法产品构建返回 11、`verus --compile` 编出返回 12 | **判法收严**：被验区域里任何 cfg 原子（不只 `verus_keep_ghost`）都判红，确要的写进信任清单逐条登记；被验类型的 trait impl 一律写在 `verus!` 里面，外面的判红；MIR 比对改成「产品构建」对「`verus --compile` 构建」逐函数比，不是两次产品编译 |
| Z3 射程 | 攻方：三个闭包（836 / 1057 / 654）之外没找到改抽象状态的 core 路径（非穷举）。辩方打中：含回收门槛调用的七个非测试 fn 里三个在闭包外，其中算 `floor` 的两个（挂载重建分配器之后的回收、抬 F 序列里的回收）正是 `floor` 的来历；改盘面字节的写点 18 个，闭包外 3 个：丢弃整段、实例表清扫抹头、`discard_at` 的 trait 默认实现 | **射程加辩方点名的四块**：抬 F 一路、挂载重建分配器之后的回收、丢弃（含 `discard_at` 默认实现）、实例表清扫抹头。攻方的「没打中」只抽了一次样，辩方这一格是打中，以打中为准 |
| Z4 判器键 | 辩方：原型上五种只改证明的写法键不变、三种改执行代码的键变，Verus 的模式检查拒「证明影响执行」的四种写法；判决漏一条：形式化判定的键必须保留规格，否则假绿；原型是文本级、对 `lib.rs` 排版失败，要换 `verus_syn`。攻方：词法剥会认错 exec 与 ghost（推的） | **采辩方的形**：剥规格与证明块只用于判产品行为的那几类判定的键；形式化判定的键保留规格；剥法用 Verus 自己的语法树（`verus_syn`），不用词法 |
| Z4 整库重判代价 | 辩方：判器闭包今天已含 83 个 replay-prefix 单元（23 个证明 fn），在已验 `lib.rs` 里一条引理多一句 `assert(true);` 键就变；一次整库作废约 3.9×10⁹ 到 3.8×10¹⁰ 态（夹出来的、不是量的） | 用 Z4 那一形之后不再整库作废；这个数写进交用户表当没做 Z4 时的代价 |
| Z5 数 | 两腿一致：1073 与 1071 差 2 个 replay-prefix 核 fn，core 新验 1071 个 fn、24958 行；A / B / C / D 与 456 个 std 键、配上 49% 原样复出（A 是上界，名字配上的只有 287）。两腿打中：「11 万–12.5 万行」的倍数取自正推自己新写的两个原型；攻方按三闭包射程（1185 fn、29807 行）同倍数重算约 13.4 万–14.9 万行；辩方按现成核的现数（`relocation_judgement.rs` 2.5 倍、`compaction_plan_proofs.rs` 8.6 倍）重算约 3.9 万到 20.8 万行 | **代价写成区间、标推的**：按这一轮定的射程（三闭包加 Z3 四块）证明代码约 3.9 万到 20.8 万行，倍数随类别差一个数量级；实现第一批量出 A 类的实际倍数后回写 |

## 三、三轮的结论

- 层与射程：core 直验，发布、回退、崩溃后读映射三个闭包，加抬 F 一路、挂载重建分配器之后的回收、丢弃、实例表清扫抹头四块；里程碑四全验（用户定）。
- 抽象状态：`gc_crash_proofs` 的 `logical` 补回退目标视图、多逻辑版本与「内存 F / 持久 F」两层；根环协议按区间、映射值带跨度；区间版 P4。
- G7 照第二节 Z1 第二行改写。
- 门禁：去掉 `--no-cheating`，登记式信任清单（每条点名、规格原文 sha256、差分测试；vstd 按版本整体登一行），`axiom` 不许，被验区域里任何 cfg 原子判红（登记的除外），被验类型的 trait impl 必须在 `verus!` 里，产品构建对 `verus --compile` 构建的 MIR 逐函数比，Verus 与产品的 rustc 版本差进工具链钉表，按全路径逐项验，判法改十一条（第二轮正推 Y4）。
- 判器键：产品行为那几类判定剥规格与证明块（`verus_syn`），形式化判定保留规格。
- 只供测试的开关挪进默认关的 cargo 特性。

## 四、交用户

| 项 | 形态 | 被攻过几轮 | 去向 |
|---|---|---|---|
| 根环协议按区间、映射值带跨度 | 两腿各自独立提 | 零轮 | 写回、进实现第一批 |
| G7 改写措辞 | 辩方提 | 零轮 | 写回 |
| 射程加四块 | 辩方打中 | 一轮 | 写回、进实现 |
| 门禁收严三条（cfg 原子一律判红、trait impl 在 verus! 里、MIR 对 verus --compile 比） | 两腿打中 | 一轮 | 工具实现员 |
| 判器键剥规格只用于产品行为判定、用 verus_syn | 辩方提 | 零轮 | 工具实现员与单元图实现员；没做时的代价是一次整库作废约 3.9×10⁹ 到 3.8×10¹⁰ 态（推的） |
| 代价区间 3.9 万到 20.8 万行证明代码 | 两腿重算 | 一轮 | 第一批量出实际倍数后回写 |

## 五、复跑

第二台没有 Verus，在本机仓外目录跑（`RUN_WITH_MEMORY_CAP_WAIT_SECONDS=7200 bash scripts/hosts/run-with-memory-cap.sh 8G bash ~/.local/share/singlefs/work/m4-exit5-relocation-proof-r3-verify/<腿>-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --work …`，辩方那一份照它报告分五段 `RERUN_STEPS=z1` 到 `z5`）：攻方末行「name=rerun-compare out/summary.txt same=yes」；辩方五段末行都是「name=rerun_result mismatches=0」。两份报告里引到的 verified / errors 与穷举、闭包数都复出。

## 六、写回与后续

- 写回（kb-scribe）：里程碑四出口 5 那一行（射程、层、全验）；判决 G7 那一句的现行写法（m4-gc-r1 判决冻结，写回落在 D13 与里程碑文件里，G7 原文留在判决里）；D13 新立一项「GC 恒等性质与 core 直验」（M1–M4 的主语是 `Recover`，不补进已定项 1）；`.claude/rules/verification.md`「定义与名字」checker 档那一行「Verus 验 `crates/singlefs-replay-prefix`」改成连 `singlefs-core` 被验区域一起，与「门禁的结构」checker-tier-formal-verification 按条那一句。
- 实现：先派工具实现员改判法与门禁（第三节「门禁」那几条）、单元图实现员改判器键（Z4）；同时派第一批 core 直验实现员（A 类、协议与区间 P4、G7 引理、只供测试开关挪进特性），量出实际倍数；之后按 B、C、D 分批派，每批照「写与验分两段」。
- 推论轮到此结束。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 1 | 不影响 | 2026-10-26 不受影响：GC 恒等性质新立一项，M1–M4 不改 |
| D13（验证路线） 已定项 5 | 不影响 | 2026-10-26 不受影响：core 加 vstd 不进池级 checker 的依赖闭包 |
| D13（验证路线） 已定项 14 | 不影响 | 2026-10-26 不受影响：校验和照旧当未解释函数，进信任清单的 D 类 |
| D13（验证路线） 已定项 17 | 支撑 | 2026-10-26 改了：产品行为判定的单元键剥规格与证明块、形式化判定保留，并进写回批 |
