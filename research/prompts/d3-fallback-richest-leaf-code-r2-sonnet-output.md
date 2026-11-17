# d3-fallback-richest-leaf-code-r2 辩方腿报告：用户数据落点换算法与第一轮判决的改法（2026-11-13）

立场：辩方。复核第一轮判决（`research/prompts/d3-fallback-richest-leaf-code-r1-main-verification.md`）里没采与只记欠账的条目够不够得着，核这一批修补做的是不是判决第三节那五条，逐格回检查单，答正文第五节第 3 条。
没跑项目代码、没交任务、没写模型：没有模型目录，也就没有复跑命令与模型文件的 sha256。下面凡是读码推的都标「推的」，没有「量的」。要主 agent 跑的在末尾「要主 agent 跑的命令」一节。

开工核：`sha256sum -c research/prompts/d3-fallback-richest-leaf-code-r2-snapshot/crates-sha256.txt` 三份（`allocator.rs`、`transaction.rs`、`admission.rs`）全部 OK，对不上 0 份。下面的行号都取此刻主工作区；kb 行号取 kb 文件自己的行号。

## 先说结论（四条）

1. 第一轮判决里没采与只记欠账的七条（攻击 1c、固定点终止、C1017 到 C1021 里的另外四条），判决给的理由在此刻主工作区上都仍成立；这一批修补没有让任何一条变得更够得着。没有一条该采而没采。但读码读出三处不影响结论、要改的缺口：欠账 C1017 的理由字面把「装置走不到」写成了「所以没采」（该写的是不分辨新旧政策，且零轮改法要先做 A/B）；C1019 的「推得出」那一句漏了两个角（单元区尾巴不满一段、活 `R` 那一段）；判决第 11 条的注释清单漏了一处（`admission_checkpoint_cost_per_device_paths.rs:1096`）。见第一节。
2. 判决第三节五条改法：第 1、2、3、5 条落全了，没有越位的多做；第 4 条三处注释与 `model.rs` 文档落了，同类第四处没改，`model.rs` 那句「推得出」读码推不全。第 5 条的「答案相同的依据」三样，一样由类型保证（`R`），一样由类型加构造函数保证（单元区起点），一样靠唯一写入点保证、没有类型也没有断言（聚簇段起点）。见第二节。
3. 检查单八格：成立 5 格（走不到的理由、不兼容旧代码、不新建判定库、多机可用、新崩溃流多罩了什么），不成立 2 格（类型与名字：`allocator.rs:1345` 一处裸 `expect("64")`；测试优先多线程：`allocator.rs:5573` 一条用例两个场景），推不出 1 格（带 --item 的命令与项，形式上成立、实质缺两项崩溃枚举用例）。推不出的算打中，三格共 3 处。见第三节。
4. 正文第五节第 3 条：攻击 1c 与固定点终止在这一批之后仍然够不着，这一批没有让它们变得够得着；一条相反方向的观察：换算法之后近满池那一档的历史跑得动了，欠账 C1016 能用实验还了。见第四节。

## 判决条目复核表（主表）

「辩方判定」三种取值：站得住（判决的理由此刻仍成立）、够不着（攻击或改法够不到打中的格）、同样打中别的臂（打中不分辨臂）。

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| 判决第二节第 4 条：攻击 1c 不采，记欠账 | 站得住；理由里「装置走不到」不是不采的理由，「不分辨臂」才是；同样打中别的臂 | 第一节 A |
| 判决第二节第 3 条：固定点终止没被打穿也没被证明，上界 64 留着 | 站得住；这一批没动固定点循环与落点 | 第一节 B |
| 欠账 C1017（准入只数槽不数对） | 站得住；同样打中别的臂；欠账原文的因果写反了 | 第一节 C |
| 欠账 C1018（每次回落按叶走一遍，代价随单元区线性） | 站得住；这一批之后它是近满池档里剩下的最大线性走法（推的） | 第一节 D |
| 欠账 C1019（`model.rs` 重写节点上界在隔离撑起的非空段一角） | 站得住，但射程小了：「推得出」那一句还有两个推不出的角 | 第一节 E |
| 欠账 C1020（新池新建文件预演走空重写集） | 站得住；预演现在也走新取法，答案同旧 | 第一节 F |
| 欠账 C1021（挑真走过回落的崩溃枚举用例没有机器办法） | 站得住；这一批的人工挑项缺两项 | 第一节 G、第三节第 7 格 |
| 判决第二节第 5、6 条（增量维护、演练与真发同一串落点） | 站得住；这一批多加的集合守同一个单一写入点、克隆同拷 | 第二节末 |
| 判决第三节第 1 到 5 条（改法） | 第 1、2、3、5 条落全；第 4 条落了三处，同类第四处与 `model.rs` 一句推不全 | 第二节 |

## 一、第一轮判决里没采与只记欠账的条目：此刻还成不成立、这一批有没有让它们够得着

### A 攻击 1c（空槽最多的叶拆掉全池最后一对相邻空槽）：站得住，同样打中别的臂

- 判决给了两层理由：「装置走不到，不采，也不算没打中」（`research/prompts/d3-fallback-richest-leaf-code-r1-main-verification.md:34`）；攻方自己写了这一形不分辨新旧政策，病根在准入只数槽。
- 第二层此刻仍成立：「② 只数槽：两槽的 inode 叶容器要成对」（`crates/singlefs-core/src/admission.rs:1054`）；准入的判只比数据单元的槽对数与没挡槽数减去数据单元占的槽数（转述，`crates/singlefs-core/src/admission.rs:1077-1103`），两槽的提交内生块要不要成对没人数，对每一条臂一样。
- 第一层不是不采的理由：攻方锻造盘没过准入，说明它的形状没量到，不说明产品里走不到。读码推（没量）：过准入要每块盘没挡的槽不少于整棵树的最坏节点数（`crates/singlefs-core/src/admission.rs:1189` 起的文档写「拿得准的上界只有几何上整棵树」），同时全池没有一对留给两槽容器，需要大量散落的单槽、一对不剩；近满而老化的池子里理论上走得到，没量过。
- 不采的站得住的理由有两层：这一形不分辨臂（攻方自己推的镜像形上，改前的两条臂先被打中，`research/prompts/d3-fallback-richest-leaf-code-r1-opus-output.md:83`「病根在各臂共用的前提」）；攻方的收严是零轮改法，规则要求「写进产品代码之前先在探针里对它做 A/B」（`.claude/rules/three-way-inference.md:112`），A/B 没做就不能写进产品代码。
- 这一批没有让它变得够得着：1c 的三个前提——数据单元取最低一对（`crates/singlefs-core/src/allocator.rs:2923` 起的 `try_allocate_user_data`，产品调用点 `crates/singlefs-core/src/transaction.rs:2502`）、单槽先于两槽取落点、回落先取重写集再取单槽最多的叶（`crates/singlefs-core/src/allocator.rs:3106-3180`）——这一批没改逻辑，只换名字与类型；用户数据落点答案同旧（第二节第 5 条）。新拒绝成员（`crates/singlefs-core/src/allocator.rs:297`）只在「每块盘都答回落、池里取不到每块盘都空的落点」时换名；1c 里每块盘各自就取不到一对，走的是 `NoFreeSlotOnAnyDevice`（`crates/singlefs-core/src/allocator.rs:3056`），不进新成员。
- 后果的上界：发布层的 `PlacementRefused` 被会话层当空间不够，先推一串抬 F 再重判（「PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,」，`crates/singlefs-core/src/mounted_session.rs:2120`），不是直接报假性 ENOSPC；推抬 F 之后仍取不到才到头（推的）。
- 没有该采而没采的。什么会推翻：一段准入放行、没有全空段的真历史上，两槽容器报 `PlacementRefused`，而同一段历史在改前政策下不拒（分辨臂），那时 1c 转成该采，并先做零轮改法的 A/B。一次都没量到过。

### B 固定点终止没被证明、上界 64 留着：站得住，这一批没动

- 判决：「上界 64 留着，终止论证记欠账。」（`research/prompts/d3-fallback-richest-leaf-code-r1-main-verification.md:33`）。
- 此刻：上界常量「pub const ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT: usize = 64;」（`crates/singlefs-core/src/transaction.rs:14166`）；固定点循环从 `crates/singlefs-core/src/transaction.rs:14031` 起；这一批在 `transaction.rs` 只动了两处：导入与 `crates/singlefs-core/src/transaction.rs:2533` 的叶序号包装（`crates/singlefs-core/src/transaction.rs:2523` 的类型），循环体没动。文档仍写「推的、没证」（`crates/singlefs-core/src/transaction.rs:13980`）。落点答案同旧，所以每轮取到的落点、轮数同旧。每轮克隆分配器（「let mut rehearsal = allocator.clone();」，`crates/singlefs-core/src/transaction.rs:14076`）现在多拷一个至多段数项的集合，只是内容，不改语义。
- 判决没写的严重度：不收敛的结局 `AllocationRecordTreeRewriteSetDidNotSettle` 不属于「空间不够」那两种，归在推抬 F 也不会变的一类（`crates/singlefs-core/src/mounted_session.rs:2135`，函数头 `crates/singlefs-core/src/mounted_session.rs:2118`），所以不推抬 F、原样交回，同样的输入重试得同样的结果。登记欠账时建议把这一条写上（推的）。
- 反方向的一条观察：E206 量到至多 5 轮的取样点，文档自己写「这几个点都有全空段、走不到回落」（`crates/singlefs-core/src/transaction.rs:13983`），回落档的轮数没量过；换算法前，近满池一档的历史被用户数据扫描的代价拖得跑不动（调查报告 `~/.local/share/singlefs/work/e207-slow-item-investigator/result.md` 第 5 行起），换算法之后这一档可测，欠账 C1016 可以用实验还。
- 什么会推翻：回落档的一份真历史量到轮数逼近 64，或模型里现行实现那一臂报 did-not-settle-after-64-rounds。

### C 欠账 C1017（准入只数槽不数对）：站得住，同样打中别的臂；欠账原文的因果写反了

- 理由此刻仍成立（同 A）。
- 欠账行把「装置走不到」连到「所以没采」（「在空间准入就被拒、没进固定点，所以没采」，`.claude/kb/checks-owed.md:620`），而判决里不采的依据是不分辨臂（`research/prompts/d3-fallback-richest-leaf-code-r1-main-verification.md:34`）。装置没走到只说明没量到。写回员把那一行的因果改成「不分辨臂；零轮改法未做 A/B；装置未量到」，不然以后有人照字面把「装置走不到」当成放过它的理由。
- 什么会推翻：同 A。

### D 欠账 C1018（每次回落按叶走一遍，代价随单元区线性）：站得住；这一批之后它是近满池档里剩下的线性走法（推的）

- 判决只写这一句，没量。此刻有两处线性：池级按叶走（`crates/singlefs-core/src/allocator.rs:3154` 起的循环，先读每片叶增量维护的没挡槽数剪枝），与每块盘自己答的回落落点从单元区起点逐候选（「while candidate + footprint.slots() <= unit_area_end {」，`crates/singlefs-core/src/allocator.rs:1444`）。后一处实现员第一份报告第八节第 3 条已记，判决没列。
- 回落只在每块盘没有全空段时走（`crates/singlefs-core/src/allocator.rs:1405` 起只在整段里找）；用户数据那一处换成按段之后，同一档（近满池）里这两处是剩下的线性走法。调查报告说那个慢项回落走不到、`fallback_slot_linear_scan` 在量窗内零行（调查报告第 67 到 69 行，量窗内量的、历史上推的）。所以没有证据说它现在慢，欠账不改站得住。
- 什么会推翻：一份近满池历史上，回落次数乘单元区槽数的量级到 1e8 以上。

### E 欠账 C1019（`model.rs` 重写节点上界在隔离撑起的非空段一角）：站得住，射程小了两个角

- 文档此刻写：「每块盘上一个全空段都没有时才走，那时单元区里每一段至少有一槽被挡」（`crates/singlefs-verification/src/model.rs:4212`），并只把「有的段只靠影子账隔离或抬 F 扣住的槽撑成非空」那一角写成推不出（`crates/singlefs-verification/src/model.rs:4214`）。
- 读码推（没跑）：「没有全空段」不等于「每一段至少有一槽被挡」。开段与回落的触发在 `lowest_empty_segment_outside_the_region`，它只在整段里找（「let full_segments = self.allocated.len() / per_segment;」，`crates/singlefs-core/src/allocator.rs:1405`）并排除活 `R` 那一段（`crates/singlefs-core/src/allocator.rs:1411-1413`）。于是还有两个角：活 `R` 那一段可以整段空而回落照走；单元区末尾不满 64 槽的那截不是段、可以整截空。这两种情形下单元区可以长到 `64 × P + 64 + 63` 槽，回落落在 `64 × P` 之外，`model.rs` 的可达范围罩不住。默认两块 4 GiB 盘单元区正好整段（262144 减 50176 是 64 的 3312 倍），尾巴角只在别的容量出现；`R` 角只在有活整理意图时出现。
- 判：欠账站得住；判决第三节第 4 条要的「新政策下推得出的那一句」读码推不全，建议把这两个角并进文档与 C1019（写回员的活）。什么会推翻：模型里这两个角的几何上构造不出「没有全空段」的状态（比如整理意图总在有全空段时才签发）。

### F 欠账 C1020（新池新建文件的预演走空重写集）：站得住；预演现在也走新取法，答案同旧

- 预演调用「rehearsing.allocate_user_data(new_pool_file_creation_txg)」（`crates/singlefs-core/src/recovery.rs:6614`），建的空闲图只到单元区开头两段、起点是传进来的 `UnitAreaStart`（`crates/singlefs-core/src/recovery.rs:6592` 起）。新取法对这张图的答案同旧：段集合起步是每段槽对数不是 0 的段（`crates/singlefs-core/src/allocator.rs:660-666`），第一个数据单元落单元区起点加 4，与 kb 登记的取值一致（「新池新建文件那一次的取值是 t1@50180」，`.claude/kb/decisions/03-空间分配.md:175`）。
- 欠账的缺口（预演取提交内生块走空重写集）这一批没碰。站得住。什么会推翻：非默认环、单元区小于两个整段的几何上，预演的记录数与真发差一片叶（攻方第一轮 2d 的形状），且第一个文件能过合取 7。

### G 欠账 C1021（挑真走过回落的崩溃枚举用例没有机器办法）：站得住；这一批靠人工挑项，缺两项

- 没有机器办法这件事此刻没变。这一批改的是每个落用户数据的流都经过的分配器，实现员按改到的测试目标挑，挑出零项：「这批目标都没登记崩溃枚举」（实现员第一份报告第七节开头，`~/.local/share/singlefs/work/user-data-placement-scan-writer/result.md` 第 171 行）。主 agent 另点了两项缩域重放（近满池一项、快照下截另一个 inode 一项）。
- 落用户数据且最该点的两条流：整理搬迁（目的地数据单元走 `try_allocate_user_data`，「) => allocator.try_allocate_user_data(relocation.birth_txg),」，`crates/singlefs-core/src/transaction.rs:13806`，整理类的几条流里 `R` 是活的，新取法里「段起点在 `R` 里就跳」在崩溃枚举里只有这一类流用到，主 agent 点的近满池与截断那两项里有没有活 `R`，我没核，推的是没有）与新池新建文件（落点 U 加 4 由 I-5.5 在盘面上判）。登记表里它们是 `crash-case:compaction-relocation-stream` 与 `crash-case:new-pool-file-creation-stream`（`.claude/gate.d/stage-inputs.tsv`）。见第三节第 7 格与「要主 agent 跑的命令」。

## 二、判决第三节五条改法逐条对被判改动

diff 行指 `research/prompts/_d3-fallback-richest-leaf-code-r2-diff.md` 的行；文件行取此刻主工作区。

| 改法 | 落在哪几处 | 落全了没有 |
|---|---|---|
| 1 改名，池级取不到时换说实情的错误成员 | 成员 `FallBack`：diff 53-61、`crates/singlefs-core/src/allocator.rs:197`；计数字段与访问器：diff 448-480、`crates/singlefs-core/src/allocator.rs:2314`、`:2362`、`:2659`；引用处 `:3029`、`:3045`、`:3079`、`:3083`；验证包 `crates/singlefs-verification/src/model_comparison.rs:1778`、`crates/singlefs-verification/tests/unequal_devices.rs:388`、`crates/singlefs-verification/tests/allocator_fallback_count_on_small_devices_after_repeated_mounts_and_the_first_file.rs:129`；变异行原文 `crates/mutations.tsv:131` 等；新成员 `crates/singlefs-core/src/allocator.rs:297`，分法 `:305`（调用 `:3064`），名字表 `crates/singlefs-verification/src/history.rs:4579`，理由映射 `crates/singlefs-verification/src/model_comparison.rs:1322`，单测 `crates/singlefs-core/src/allocator.rs:5270` | 落全。旧名 `FallBackToLowestFreeSlot` 与 `fell_back_to_the_lowest_free_slot` 在 `crates`、`scripts` 下各 0 命中（我跑的 `grep -rnF … \| wc -l`）。不是越位：换成员只在 `AnswersDiffer` 一支，拒不拒、在动状态之前拒都不变 |
| 2 叶序号、每片叶槽数各立 newtype；回落那一片的槽号用 `SlotNumber`；`expect` 写清不变量 | `AllocationRecordTreeLeafIndex`：diff 64-87、`crates/singlefs-core/src/allocator.rs:203`；`SlotCountInALeaf`：diff 89-91、`:227`；`RewriteSetLeaves` 字段与交集：`:234`、`:248`；每片叶没挡槽数字段：`:498`；`crates/singlefs-core/src/transaction.rs:2523`、`:2533`；槽号 `SlotNumber`：`:839`（`is_whole_span_open_for_commit_generated` 的候选）与叶内找落点；新 `expect` 五处 | 落了，缺一处：`crates/singlefs-core/src/allocator.rs:1345` 的 `expect("64")` 没写靠哪条不变量（第三节第 2 格）。其余四处写了依据（diff 289、298、411、625 对应的 `allocator.rs` 行）。段号没立类型，判在第三节第 2 格 |
| 3 按参数循环的单测拆成三条 | `crates/singlefs-core/src/allocator.rs:4980`、`:5002`、`:5024`，形态前提收进 `:4953`（diff 705-820） | 落全。同一类毛病在新加的 `crates/singlefs-core/src/allocator.rs:5573` 上又出现一次（第三节第 6 格） |
| 4 三处注释改现状；`model.rs` 上界改成新政策下推得出的一句、推不出的一角指到欠账 | `crates/singlefs-core/src/admission.rs:981`、`:1189`（diff 12-31）；`crates/singlefs-verification/src/model.rs:4204-4217` | 三处落了。缺口两个：同类第四处 `crates/singlefs-verification/tests/admission_checkpoint_cost_per_device_paths.rs:1096`（「情形 ②（没有全空段，回落到最低空槽、一个单元一处）」）没改，实现员第八节第 4 条看到了、没在判决清单里就没改；`model.rs` 那句「推得出」读码推不全（第一节 E）。前一条是判决清单漏的，不是这一批的错 |
| 5 用户数据落点换算法、落点不变 | 字段 `crates/singlefs-core/src/allocator.rs:485`、起步 `:659-666`、`:704`；维护 `:748-786`；新取法 `:1320-1337`、段内 `:1341-1361`；参照取法（只在测试）`:1363-1390`；调用 `:2935`、薄包装 `:1307` | 落全；落点不变靠对拍用例：`crates/singlefs-core/src/allocator.rs:5456`、`:5478`、`:5500`（三种几何）、`:5522`（定形取样点）、`:5573`（增量集合对扫）。证据见下 |

多做的（改法之外）：没有越位的。看到的几样都在改法的射程里：`AllocationRecordTreeLeafIndex` 的三个辅助方法（`of_slot`、`first_slot`、`past_the_last_slot`，`crates/singlefs-core/src/allocator.rs:209` 起）属于第 2 条；`refusal_of_commit_generated_answers_that_differ` 属于第 1 条；对拍用例、逐对扫的辅助函数与 `#[cfg(test)]` 的参照取法属于第 5 条的「落点不变」要有证；`crates/mutations.tsv` 的新增与改写是改名与新测试必须跟着改的。两处值得记：两个新类型的字段都是 `pub`，别处可以直接用裸 `u64` 造，类型只挡住了混用、挡不住造错；`#[cfg(test)]` 的参照取法是逐候选的旧算法原样留着，它是对拍的参照，不是为旧数据留的兼容（第三节第 3 格）。

### 第 5 条的「答案相同的依据」三样，在此刻主工作区里各由什么保证

实现员第一份报告第二节末段（`~/.local/share/singlefs/work/user-data-placement-scan-writer/result.md` 第 69 行）写的三样：一对不跨段；聚簇段起点只来自开段的答案、是单元区起点加 64 的整数倍；`R` 的起点是 64 的倍数。新旧两个取法的差只在「候选在不在聚簇段或 `R` 里」怎么问：旧的问候选落在哪个区间，新的问候选所在那一段的起点在不在；两者相同要三样都落在同一套 64 槽格子上。

| 样 | 保证它的东西 | 文件:行 | 保证的层次 |
|---|---|---|---|
| 单元区起点是 64 的倍数、一对不跨段 | 单元区起点只能由 `UnitAreaStart::following_the_journal_ring` 造出，它拒不在段边界上的槽（「if !slot.0.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {」）；元组字段私有，造值处全仓只有 `Self(slot)` 一处（我 `grep -rn 'UnitAreaStart(' crates` 只命中定义行 `crates/singlefs-core/src/allocator.rs:375`）。段长常量是 64（「pub const CLUSTER_SEGMENT_SLOTS: u64 = 64;」）。成对：位图下标 0 对着单元区起点，下标奇偶就是槽号奇偶；成对判取 `index - index % 2` 起的两槽，末尾落单的一槽答否（`crates/singlefs-core/src/allocator.rs:740-745`）；段内从段首按 2 步进（`crates/singlefs-core/src/allocator.rs:1349-1353`），段首下标是 64 的倍数，一对的两槽必在同一段 | `crates/singlefs-core/src/allocator.rs:393`、`:399`、`:375`；`crates/singlefs-format/src/lib.rs:324` | 类型加唯一构造函数，是真保证 |
| 聚簇段起点只来自开段的答案、是 64 的倍数 | `cluster_segments` 全仓唯一的插入点是「self.cluster_segments.insert(segment_start);」，插的值取自各盘一致的 `OpenEmptySegment` 答案；这个答案只在「Some(CommitGeneratedDeviceAnswer::OpenEmptySegment(segment_start))」处造出，值来自 `lowest_empty_segment_outside_the_region` 的 `单元区起点 + 段号 × 64`（`crates/singlefs-core/src/allocator.rs:1415-1420`），由上一行是 64 的倍数。字段私有，对外只有只读访问器（「pub fn cluster_segments(&self) -> &BTreeSet<SlotNumber> {」，`crates/singlefs-core/src/allocator.rs:3752`）。别处造 `OpenEmptySegment(` 字面量的只有两个测试里拿来比较的值（`crates/singlefs-verification/tests/unequal_devices.rs:384`、`crates/singlefs-verification/src/model_comparison.rs:1774`），不进 `cluster_segments` | `crates/singlefs-core/src/allocator.rs:3073`、`:3070`、`:1475` | 靠唯一写入点，没有类型，新函数对传进来的任意 `&BTreeSet<SlotNumber>` 也不判对齐（`crates/singlefs-core/src/allocator.rs:1320-1323` 是 `pub`）。邻居函数 `pairs_inside_the_cluster_segments` 判了：「"聚簇段起点在段边界上：{segment_start:?}"」（`crates/singlefs-core/src/allocator.rs:593`，断言 `:591-594`），准入读它（`crates/singlefs-core/src/admission.rs:995-1000`），所以带普通分配的发布里错位的段起点在落点之前先被断言抓到；准入关掉的只供测试开关下抓不到（推的） |
| `R` 的起点是 64 的倍数 | `CompactionRegion` 字段私有，运行时唯一的造值处在判过之后：先判「if !start_slot.0.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {」再 `Ok(Self { device, start_slot })`；类型不变量由 Verus 钉（`crates/singlefs-core/src/logged_operations.rs:137-140`）。`R` 恒是整一段，槽数不存（`crates/singlefs-core/src/logged_operations.rs:114-118`），所以「`R` 跨段」「`R` 不满一段」造不出来。`contains_slot` 是「slot.0 >= self.start_slot.0 && slot.0 < self.start_slot.0 + CLUSTER_SEGMENT_SLOTS」，起点与段起点同在 64 格子上时「段起点在 `R` 里」等价于「两个起点相等」 | `crates/singlefs-core/src/logged_operations.rs:211`、`:217`、`:274` | 类型不变量加 Verus，三样里最强 |

依据之外还要成立的两件事，一并核了：

- 集合与每段槽对数同源。三张挡位位图只经 `set_blocking_bit` 写（`crates/singlefs-core/src/allocator.rs:748-786`），它的调用点六处（`:1086`、`:1128`、`:1142`、`:1172`、`:1193`、`:1287`）；空闲图字面量只在 `with_unit_area_start` 里（`:667`），那里按每段槽对数起步建集合（`:660-666`）。「集合里有这一段 ⇔ 这一段槽对数大于 0」由起步加两个转移（0 到正加入 `:782`，1 到 0 摘掉 `:775-777`）归纳成立。挂载重建是 `PoolAllocator::new` 再逐条 `mark_allocated`（`crates/singlefs-core/src/allocator.rs:2668` 起的 `rebuild_from_records`），也经同一处；克隆是 derive（`:429`）整份拷。
- 尾巴。末段不满 64 槽时：起步槽对数是 `min(剩余, 64) / 2`（`:633-640`），只剩一槽的尾巴不进集合；段内上界 `min(…, allocated.len())`（`:1347-1348`）；旧取法的循环条件「while candidate + 1 < end {」（`crates/singlefs-core/src/allocator.rs:1374`）同样不把落单的一槽当候选。两边在尾巴上答案相同。末段只会经 `R` 被跳（聚簇段只在整段里开，`:1405`），两边都按「在 `R` 的 64 槽里」跳。

结论：三样都成立；我没有找到一条能把不对齐的起点放进 `cluster_segments` 或 `R` 的路径。对照攻击面第 1 条的几个角：挂载重建后聚簇段集合是空的（`:2355`）；整理意图续做的 `R` 经 `of_one_cluster_segment` 解码（`crates/singlefs-core/src/logged_operations.rs:738` 起的解码处，推的：只读到这一行）；盘的块数不进新函数（按盘各取，`crates/singlefs-core/src/allocator.rs:2929-2940`），三块以上的盘不改等价性。什么会推翻：有一条路径往 `cluster_segments` 插不是 `lowest_empty_segment_outside_the_region` 给的值，或另有 `UnitAreaStart` 的造值处。

对拍用例的覆盖（攻击面第 2 条，读码判）：历史生成器（`crates/singlefs-core/src/allocator.rs:5295` 起）走到六个写位的入口各一个——分配数据单元与提交内生块（`mark_allocated`）、回收（`mark_reclaimed`）、抬 F 扣住与放开（`hold_until_floor_takes_effect`、`release_holds`）、影子账隔离与清隔离、`R` 签发与撤回、克隆；三种几何（385 与 385、385 与 1000、2000 与 2000，尾巴分别是 1、40、16 槽）。扣住与隔离叠在同一槽上：转移判的是三位的「或」前后（`:749-757`），与哪一位先后无关。没覆盖的形态（三块以上的盘、挂载重建后接着走）按上面的归纳不产生新的分叉；我没找到会分叉的形态。

## 三、正文第四节检查单八格

推不出的算打中。结果：成立 5 格，不成立 2 格，推不出 1 格。

### 1 走不到的理由二次查实：成立

- 三份实现员报告「走不到的理由」都写无。我在 diff 的产品代码部分（diff 第 1 到 660 行）现核：新加的 `+` 行里没有 `todo!`、`unreachable!`、`assert!`，也没有名字以 `Undecided`、`Unsupported`、`OutsideTheFirstVersion` 收尾的成员（`grep -nE` 只命中五处 `expect(`，归第 2 格）。
- 新错误成员 `CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice`（`crates/singlefs-core/src/allocator.rs:297`）条款有字面：「池里取不到每块盘都空的落点时，照各盘自己的答案判一致，不一致拒绝」（`.claude/kb/decisions/03-空间分配.md:173`），所以不是条款没定的成员。它在动状态之前返回：交回在 `crates/singlefs-core/src/allocator.rs:3064`，之后才动 `open_segment`、`cluster_segments` 与记录（`:3070` 起）；单测断言记录、回落计数、开放段一样没动（`crates/singlefs-core/src/allocator.rs:5270`）。判定与后面的写读同一份输入：发布路径在分配器的拷贝上走固定点（「let mut rehearsal = allocator.clone();」，`crates/singlefs-core/src/transaction.rs:14076`），真发取同一串落点（第一轮判决第二节第 6 条）。
- `lowest_unblocked_slot_pair_in_the_segment` 找不到对时（`crates/singlefs-core/src/allocator.rs:1341-1361`）：产品路径上走不到——集合里的段槽对数大于 0 由单一写入点归纳成立（第二节），那一段里一定有一对。真走到了，交回 `None`，`find_map` 接着走下一段（`crates/singlefs-core/src/allocator.rs:1327-1336`），答案不变，只多看至多 32 对。判成立。附一句：这一支是静默的，没有断言，维护出了错答案照样对；只有拿集合与逐对扫比的白盒用例看得出（`crates/singlefs-core/src/allocator.rs:5587-5591`）。
- 什么会推翻：集合里留下一段槽对数为 0 而取落点答出了不同的结果（那就是 `None` 之外还有别的支）。

### 2 类型与名字：不成立（一处）

- 打中：`crates/singlefs-core/src/allocator.rs:1345` 的「let slots_per_segment = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");」在新的产品函数 `lowest_unblocked_slot_pair_in_the_segment` 里，消息只有「64」，没写靠哪条不变量。同一个写法在这份文件里已有十一处（`:770`、`:951`、`:1087`、`:1129`、`:1143`、`:1162`、`:1194`、`:1215`、`:1288`、`:1404`，加这一处），前十处是这一批之前就有的，按字面这一处仍算打中。
- 别的新 `expect`（diff 289、298、411、625）消息都写了依据；只写一两个词的（「段号」「盘号」「个数」「下标」）逐处分开：`crates/singlefs-core/src/allocator.rs:4125`、`:4127`、`:4846`、`:5217`、`:5345`、`:5347`、`:5381`、`:5561` 都在 `#[cfg(test)] mod tests`（`:3757` 起）里；产品代码里的两处「段号」（`:1207`、`:1418`）是这一批之前就有的。
- 段号用裸 `usize`（`BTreeSet<usize>`，`crates/singlefs-core/src/allocator.rs:485`）该不该立类型：不算打中。同文件早有同形的段号集合（「segments_reclaimed_not_yet_discarded: BTreeSet<usize>,」，`crates/singlefs-core/src/allocator.rs:501`），`segment_start_slot` 等函数也吃裸段号；混用的风险是真的（`set_blocking_bit` 里位图下标、叶下标、段号三个 `usize` 在二十行内：`crates/singlefs-core/src/allocator.rs:758`、`:770`），但新集合的两处读（`:1328`、`:1332`）都把段号交给吃段号的函数，混成位图下标会被对拍抓到。要立就连 `segments_reclaimed_not_yet_discarded`、`segment_start_slot` 一并立，那是另一题，不归这一批。
- 什么会推翻：`:1345` 那句补上依据后这一格转成立；段号若出现一处把位图下标当段号传的写法，则段号也算打中。

### 3 不兼容旧代码与旧格式：成立

- `grep -nE '^\+.*(legacy|old_format|migrat)'` 对 diff 零命中（主 agent 跑过，我对 diff 的 `+` 行另加了此前、原先、起初、最初、后来、改成了、换成了、修过、原为、以前、判决、第二轮、攻方这几个历史词，零命中）。旧名在 `crates` 与 `scripts` 下零命中（第二节第 1 行）。
- `#[cfg(test)]` 的逐候选参照取法（`crates/singlefs-core/src/allocator.rs:1363-1390`）算不算为旧代码留的兼容：不算。它不读任何旧盘面、旧格式、旧字段，不认旧版本；调用处全在测试里（全部命中行号都在 `:5441` 以后）；它的角色是对拍的参照取法，规则（`.claude/agent-common.md`「里程碑 17 收尾之前不兼容任何旧东西」）点名的是认旧版本的分支、旧版本号常量、迁移、造旧形态输入的测试，这一条都不是。
- 什么会推翻：产品代码里出现调用它的地方，或它被拿去读旧形态输入。

### 4 不新建判定库：成立

diff 只动 `crates/singlefs-core/src/` 三份文件，新测试是分配器的内联单测，不存判定。我对 diff 的 `+` 行搜了 `std::fs`、`File::`、`std::env`、`PathBuf`、`tempdir`、`rocksdb`、`sqlite`、`hostname`，零命中。什么会推翻：出现任何一处落盘或另起的存放处。

### 5 所有任务多机可用：成立

同上一条的搜索零命中；新测试在进程里造空闲图与定种子的伪随机历史（种子写死并打印：`crates/singlefs-core/src/allocator.rs:5457` 起的几条），不读本机独有状态，不写主机名。副本上的任务派到第二台跑过（主 agent 的观测），我没复核。什么会推翻：测试读了环境变量或路径。

### 6 测试优先多线程：不成立（一处）

- 三条对拍（`crates/singlefs-core/src/allocator.rs:5456`、`:5478`、`:5500`）一条一个几何、各建各的状态，由测试框架并行；历史生成器里的步进循环不是该拆的参数循环：每一步读上一步留下的状态，操作的次序就是被测对象，用例旁写了为什么（「不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。」，`crates/singlefs-core/src/allocator.rs:5454`、`:5476`、`:5498`、`:5571`）。定形取样点单独一条（`crates/singlefs-core/src/allocator.rs:5522`）。
- 打中：`crates/singlefs-core/src/allocator.rs:5573` 的 `segments_with_an_unblocked_slot_pair_are_maintained_incrementally_and_match_a_scan` 在同一条用例里先建一块盘走定形取样点（`:5574-5607`），再另建一个池走 1500 步随机历史（`:5609-5625`）：两个场景、两份状态。判决第二节第 9 条为「后半段又换重写集再建一次」判过打中，标准一样。「一条用例一个场景……确是一个场景的（同一份状态上按次序做几步），在用例旁写一行注释说明为什么」（`.claude/rules/test-script-discipline.md:13`），这条用例旁的注释（`:5566-5571`）只说随机段不并行，没说定形段为什么与它同条。出路：把定形取样点那一段拆成单独一条用例（它自己建盘、自己断言，不用并进随机段）。
- 什么会推翻：定形段与随机段读同一份状态；不是（定形段用的是 `:5574` 的 `device_map`，随机段用的是 `random_history_for_the_user_data_placement` 里新建的池）。

### 7 带 --item 的命令与项：推不出（形式上成立，实质缺两项）

- 形式：这一批的 `--item` 命令是变异门禁的变异行，实现员三份报告各点名一批（第一份追加 7 行、替换 6 行；第二份追加 2 行、替换 8 行；第三份追加 1 行；后两份与第一份的替换行有重叠，三份报告的变异行清单我没逐行与主 agent 交的 22 行对，推的是一致，背景材料第二节写的是 22 行）；没有新项要被 `--list-items` 列出。按那一格的字面（命令缺项或新项列不出）成立，22 行对不对留给主 agent 逐行核。
- 实质：用户数据落点换实现，每个落用户数据的流都经过它；实现员按「改到的测试目标有没有被 `crash-case:` 行点到」挑，挑出零项（第一节 G）。主 agent 另点的两项是近满池与快照下截另一个 inode。`.claude/gate.d/stage-inputs.tsv` 里 `crash-case:` 共 108 行，其中整理搬迁一类（`crash-case:compaction-relocation-stream`）里 `R` 是活的，数据单元目的地走 `try_allocate_user_data`（`crates/singlefs-core/src/transaction.rs:13806`），新取法里「段起点在 `R` 里就跳」只有这一类在崩溃枚举里用到；新池新建文件（`crash-case:new-pool-file-creation-stream`）的落点 U 加 4 由 I-5.5 在盘面上判。这两项没点。
- 实现员一条没列算不算缺项：算，缺的是上面这两项；但崩溃枚举对新取法能多罩的只有「落点答案与旧不同」这一种，集合只住内存（第 8 格），所以风险小、缺项补两条缩域重放即可。依据推的。命令在末尾。
- 什么会推翻：这两条流的缩域重放全绿，且全域那一趟（提交时）也绿，则这一格转成立。

### 8 新崩溃流多罩了什么：成立

本轮没有新崩溃流、没有新崩溃枚举用例。新增的集合是内存里的量：`DeviceFreeMap` 只 derive `Clone` 与 `Debug`（`crates/singlefs-core/src/allocator.rs:429`），不落盘；挂载重建是 `PoolAllocator::new` 加逐条 `mark_allocated`（`crates/singlefs-core/src/allocator.rs:2668` 起），挂载后第一次取落点读到的集合只由盘上记录经同一批写位入口推出，不带出新的盘上状态、没有新写；所以崩溃状态集不会比已有流多。什么会推翻：集合被写进盘上任何一处（记账行、记录、根）。

## 四、正文第五节第 3 条：攻击 1c 与固定点终止，在这一批之后是不是仍然够不着

答：仍然够不着，这一批没有让它们变得够得着。三条依据：

1. 这一批对两者的前提零触碰。固定点循环与上界没动（`crates/singlefs-core/src/transaction.rs:14031`、`:14166`，这一批在 `transaction.rs` 只动导入、`:2523` 的类型与 `:2533` 的包装）；落点答案同旧（第二节），所以每一轮取到的落点、轮数同旧。1c 的三个前提（数据单元取最低一对、单槽先于两槽、回落先重写集再最富叶）逻辑没改，准入只数槽也没改（`crates/singlefs-core/src/admission.rs:1054`、`:1077-1103`）。
2. 新加的东西各自的作用面都碰不到它们：段号集合只被用户数据落点读（`crates/singlefs-core/src/allocator.rs:1325`）；新拒绝成员只在「每块盘都答回落、池里取不到每块盘都空的落点」时换名（`:305-325`，调用 `:3064`），1c 报的是 `NoFreeSlotOnAnyDevice`（`:3056`）；newtype 是编译期的。
3. 另一个会话的 `SlotPairCount` 替换（`crates/singlefs-core/src/allocator.rs:420`）与集合维护的交叉：翻过 0 的判法是对 `.0` 的 u64 比较（`:775` 的减一与摘段、`:664` 的起步过滤 `pairs.0 > 0`），没被改写；准入那一侧用 `slot_pairs_for_data_units: SlotPairCount`（`crates/singlefs-core/src/admission.rs:980`），1c 的两槽容器取落点不经它。没找到交叉打坏的地方。

这一批没有让它们更够得着，但改了一件会影响「怎么还欠账」的事：用户数据落点扫描换成按段之后，近满池一档的历史（回落真走起来、固定点真有机会逼近上界的那一档）不再被每次 O(候选数乘聚簇段数) 的扫描拖得跑不动，欠账 C1016 能用实验还，1c 的形状也能在真路径上量。同时「装置走不到」不再能当放过 1c 的理由（第一节 C）。没有该采而没采的。

附：攻击面第 3 条的另外几问，读码答（推的）：

- 集合的每条维护路径：写位只在 `set_blocking_bit`，调用点六处，空闲图字面量一处（第二节）；回退（复活）与回收分别是 `mark_released` 与 `mark_reclaimed`，前者不动位图、后者经 `:1193`；整理搬走之后的槽也是经这些入口。克隆（固定点演练、发布前后的换回，`crates/singlefs-core/src/transaction.rs:2038`、`:14076`）整份拷，演练与真发读同一份状态拷出来的集合。
- 头一趟没红的那条变异：任务 14606 那一行是「槽对放开时不把那一段加回有槽对的段集合」（`:782`，实现员第三份报告标题与第一节），判成取样点不敏感判得对：改坏后，一段从槽对数 0 回到正数时它不回集合，取落点会跳过它，在「一段占满再放开一对」的输入上结果与原式不同；随机历史走不到「某一段 32 对占满」，所以没红；定形取样点（`crates/singlefs-core/src/allocator.rs:5574-5607`）补上之后这个方向的转移被钉住。主 agent 派发里写的「挡上一侧不摘段」是另一行（`:775-777`，第三份报告新追加）：它与原式在落点答案上同值——集合里多留一段没有槽对的段，`lowest_unblocked_slot_pair_in_the_segment` 在那一段里找不到对，交回 `None`，`find_map` 接着走下一段（`:1327-1336`），产品路径上交出的答案不变，只多看一段。所以它不是取样点不敏感，是在落点上等价、只在集合上可见；抓它的是定形取样点里的集合对扫（`:5587-5591`），不是任何落点对拍。副作用是代价上界：多留的陈旧段让「至多走过聚簇段数加 2 段」不再成立，没有测这个代价的用例（推的）。

## 攻击面第 4 条：新拒绝成员按成员分流的每一处

读全仓对 `PlacementRefusal` 的匹配与传递，没有一处按成员分流，全是原样带或 `{ .. }`：

| 处 | 行为 | 文件:行 | 与换前 |
|---|---|---|---|
| 发布层 | `PlacementRefusal` 原样装进 `PublishError::PlacementRefused` | `crates/singlefs-core/src/transaction.rs:2510`、`:13828` | 相同 |
| 会话层抬 F 那一判 | 只认 `PlacementRefused { .. }`，一律当空间不够推一串（「PublishError::SpaceAdmissionRefused(_) \| PublishError::PlacementRefused { .. } => true,」） | `crates/singlefs-core/src/mounted_session.rs:2120` | 相同 |
| 挂载层 | 只认 `{ .. }`，或把 `refusal` 原样带进 `MountError` | `crates/singlefs-core/src/mount.rs:6358`、`:8155` | 相同 |
| 清扫 | 只认 `{ .. }` | `crates/singlefs-core/src/instance_table_sweep.rs:807` | 相同 |
| 验证包名字表 | 多一臂，输出新字符串 | `crates/singlefs-verification/src/history.rs:4579` | 记录里的成员名字符串变了，行为不变 |
| 模型对拍的映射 | 与旧成员同映射成 `Unexplained` | `crates/singlefs-verification/src/model_comparison.rs:1319-1322` | 相同 |

模型对拍把它映射成「说不清」会不会吞掉一条真分歧：不会，方向相反。`Unexplained` 在 `judge_and_advance` 里一律走「对不上」那一支（「ObservedRefusalReason::Unexplained => None,」，`crates/singlefs-verification/src/model.rs:8703`，之后 `:8705` 起交回 `ModelDisagreement`），所以产品报这个成员的历史会让对拍判红、不会被当成模型允许的拒绝；换成员之前旧成员也是同一个映射。模型的池是两块等大的盘、两盘同样地变（`crates/singlefs-verification/src/model_comparison.rs:1300` 起的文档），模型里没有这条理由，出现就该红。名字字符串的消费方我也 grep 了（`crates`、`scripts`、`.claude/gate.d`、`.claude/kb` 下含两个成员名的文件只有 `crates/mutations.tsv`、`unequal_devices.rs`、`model_comparison.rs`、`history.rs`、`allocator.rs`），名字只由 `history.rs` 的 `placement_refusal_member` 产出（`crates/singlefs-verification/src/history.rs:4613`、`:4849`），进的是对拍交回的「拒了：{member}」描述文字，没有按字符串分流的。什么会推翻：存在一处按成员名字符串分流的消费者（上面的 grep 范围外）。

## 什么现象会推翻上面的结论（汇总）

- 第一节 A、C：一段准入放行的真历史上两槽容器报 `PlacementRefused`、同一段历史在改前政策下不拒。
- 第一节 B：回落档真历史量到轮数逼近 64，或模型里现行实现那一臂报 did-not-settle-after-64-rounds。
- 第一节 D：近满池历史上回落次数乘单元区槽数到 1e8 以上。
- 第一节 E：两个角的几何上构造不出「没有全空段」的状态。
- 第二节第 5 条：有一条路径往 `cluster_segments` 插非开段答案的值，或另有造 `UnitAreaStart` 的地方；三条对拍或定形取样点用例在主工作区上红。
- 第三节：见各格末行。

## 要主 agent 跑的命令

我没写模型、没拷装置，没有复跑命令与模型文件；下面是第三节第 7 格建议补点的两项崩溃枚举用例，其余读数都在本报告里标了「推的」。

1. 整理搬迁流与新池新建文件流的缩域重放：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crash-point-replay.sh 的 crash-case-replay 格点名的 crash-case:<用例名>; 范围=3 项（1 格加点名的 2 项）; 为什么=用户数据落点换成按段取之后，整理搬迁流里活 R 的排除与新池新建文件的落点 U 加 4 各在盘面上验一次；缩到展开深度 5 当快速反馈，全域归提交时不指定范围的那一趟' task-server/task submit --cwd /home/fy5090/code/singlefs -- env SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5 bash .claude/gate.d/crash-point-replay.sh --check crash-case-replay --crash-cases-only --item crash-case:compaction-relocation-stream --item crash-case:new-pool-file-creation-stream /home/fy5090/code/singlefs`
   - 在哪个目录起：仓根 `/home/fy5090/code/singlefs`（`--cwd` 已写）。
   - 看：`task-server/task log <号>` 里两项各自的判定行与收尾计数，`TASK_SERVER_RESULT` 末行 `type=` 为成功、`exit=0`；任一项带 `✗` 行即红；缩过域的不算门禁通过。

交主 agent 判断后再派实现员的两件（不是命令，是改动）：

- 把 `crates/singlefs-core/src/allocator.rs:5573` 那条用例的定形取样点拆成单独一条（第三节第 6 格）；给 `:1345` 的 `expect("64")` 补上依据（第三节第 2 格）。
- 写回员：改 `.claude/kb/checks-owed.md:620` 的 C1017 因果（第一节 C）；C1019 与 `crates/singlefs-verification/src/model.rs:4204-4217` 的文档并进「活 `R` 那一段、单元区尾巴」两个角（第一节 E）；`crates/singlefs-verification/tests/admission_checkpoint_cost_per_device_paths.rs:1096` 的「回落到最低空槽」改成现状（第二节第 4 行）；C1016 登记时写上「不收敛不推抬 F、原样交回」（第一节 B）。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 没读禁读清单里的文件（`research/prompts/d3-fallback-richest-leaf-code-r2-opus-output.md`）。
- 没编译、没跑测试、没证红；「112 passed」「证红 23 行全抓」「任务 14606」等读数是背景材料里主 agent 写的，我没复核；对拍用例会不会红、变异行 22 行的清单对不对，都留给主 agent 的任务结果与逐行核。
- 没读验证包（`crates/singlefs-verification/`）改动的全文，只对点名的行（`history.rs`、`model_comparison.rs`、`model.rs`、变异行、测试文件里的旧名）现读与 grep；没复核任务 15053、15055 的结局。
- 没写模型，没拷装置，没起样本进程，草稿目录 `~/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r2-sonnet` 里没有留文件（认领过，没写进东西）。
- 没核 `crates/mutations.tsv` 里那 22 行的原文是否逐行与锚点相符（静态核由主 agent 跑过，任务 14784）。
