# m3-close-c506-r1：每区槽数 S 的上界改成格式常量 16，站不站得住（推论轮第一轮）正文（2026-10-22）

## 一、题面

D22（单元原子性怎么合成） 已定项 2 字段表「每区槽数 S」那一格今天写「**上界由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算**」，实现没这么做：`crates/singlefs-format/src/lib.rs` 的 `ROOT_RING_SLOTS_PER_REGION_MAXIMUM` 写死 16、标着 placeholder（C506（每区槽数 S 写成编译期常量，条文说它住系统配置））。用户 2026-10-22 弹窗定案（原话见变更史，选项原文「改条款：16 是格式常量（推荐）」）：把那一句改成「上界 16 是格式常量」。这一句是已定条款，改它要走三方（`.claude/rules/three-way-inference.md`「适用范围」）。

这一轮要回答：

1. 把上界从「挂载时由 I-7.4 扣住的块数算」改成「格式常量 16」，原条款那一句要保护的东西（为什么要有上界、上界防的是什么）会不会因此丢掉；丢的话，丢在哪一种池、哪一种历史上。
2. 改成格式常量之后，S 的上界落 D15（格式冻结政策） 已定项 7 的哪一层、改它要不要 incompat 位；下界 4 是否同样登记成格式常量（今天下界已有 `format-const` 标记）。
3. 16 这个数本身：E50（根环槽数的上下界） 说 S 的上下界都在 4..16 之外、S 不是要精算的量（D22 依据段那一行）。16 与 mkfs 写的 8、系统配置那一字节的宽、根环几何（R × S 个根槽、区域 chunk 1 MiB、槽距）之间有没有哪一条约束会被 16 撞到。

## 二、实现今天的样子（主 agent 的观测，2026-10-22，行号按主工作区，HEAD 841b3beab 加未提交的改动）

- `crates/singlefs-format/src/lib.rs` 第 296–309 行附近：`ROOT_RING_SLOTS_PER_REGION_MINIMUM = 4`（带 `format-const` 标记），`ROOT_RING_SLOTS_PER_REGION_MAXIMUM = 16`（带 placeholder 注释，写明「那个数今天算不出来：I-7.4 的『K 代』2026-09-13 起就指回退候选集本身……在它算得出来之前按 16 收着」），`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM = 8`。
- `crates/singlefs-core/src/root_ring.rs` 的 `RootRingSlotsPerRegion`（第 24–41 行附近）：字段私有，只从 `from_system_configuration_field` 或 `AT_MAKE_FILESYSTEM` 进来，挂载时判区间 4..16，越界由 `RootRingSlotsPerRegionOutOfRange` 报、整池拒绝挂载。
- `crates/singlefs-core/src/system_configuration.rs` 第 58 行 `ROOT_RING_SLOTS_PER_REGION_OFFSET = 362`：S 住系统配置槽偏移 362 那一字节，mkfs 写、挂载读回来判。
- 欠账登记：`.claude/kb/checks-owed.md` 里 `| C506 |` 那一行（今天写了用户定案与去向里程碑三二·4）。

## 三、格

| 格 | 问什么 |
|---|---|
| V1 | 原条款「上界由 I-7.4 扣住的块数挂载时算」要防的是什么：S 过大时哪一条不变量、哪一条已定项会被破坏（读 I-7.4 与回退候选集的条款原文，写出 S 与候选集、F_生效、实例表之间的关系）；改成常量 16 之后，在哪一种池几何、哪一种历史上会有 S ≤ 16 却破坏原条款要防的那件事；没有的话写出为什么没有 |
| V2 | 常量 16 落 D15 已定项 7 的哪一层（照 `.claude/kb/freeze-layer-membership.md` 的写法），改它要不要 incompat 位；区间两条边是否都登记 `format-const`（与 C506 登记行「区间两条边在 kb 里没有 format-const 登记位」对上） |
| V3 | 16 与根环几何的其余约束：R × S 个根槽能不能都落进一个区域 chunk（1 MiB）里，槽距取最大值（io_min 的整数倍，上界见 D22 已定项 2 射程那一条）时 S = 16 放不放得下；系统配置那一字节的取值域；mkfs 写的 8 |
| V4 | 改条款的写法：D22 已定项 2 那一格改成什么（逐字给候选句），依据段要不要加一条（E50 那一行够不够撑），C506 销账还欠什么（区间两边的 `format-const` 登记、`placeholder` 注释删掉、一条越界被拒的用例在不在） |

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | V1–V4 | 从附录里的条款与 `crates/` 今天的代码正推：先答 V1（原条款防什么、常量会不会丢它），再推 V2–V4 |
| 云端攻方（Opus） | V1–V4 | 找反例：V1 里 S ≤ 16 而原条款要防的那件事发生的池与历史；V2 落层落错的地方；V3 里 16 撞到的几何约束；V4 候选句与别的已定项打架的地方 |

方案按 `crates/` 今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
