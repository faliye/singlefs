# m2-safety-r3 云端辩方（Sonnet）报告

## 一、范围与材料

要辩护的一方：第二轮判决只被打中过一轮的 A1（Sonnet 读法：换下的块算进可用）、A3、A4。
要特别回答的问题（`_m2-safety-r3-body.md` 第一节「第二轮判决复核」那一格）：
各自加上 C283（照 P1「判不过时」那一格）之后，在冻结副本的拷贝上两支（正常卸载再挂、崩了再挂）
量 S4、S4b、删了再写的步数、吸收态，还出不出局；并指出第二轮判决里哪一句够不着它引的证据
（ND 单独用出局、C283 是共用病根这两处必核）。

**冻结副本核对**：`cd /tmp/claude-1000/safety-r3-frozen && sha256sum -c research/prompts/m2-safety-r3-snapshot/crates-sha256sums.txt`
在我开工时与交回前各跑过一次，130 个文件全 OK（过程见第八节「没做什么」——中途我曾经把改动直接写进了这份共享冻结副本，
发现之后已经用保存的 pristine 副本整份换回，交回前的这次核对就是复核这个换回成不成）。

**C283 自己实现，不用攻方的**：这一轮的分工表要求「辩方与 Opus 各自实现一份 C283，不共用草稿目录」；
我没有读这一轮 Opus 的产出（禁读清单），我的 C283 是独立设计、独立写的，见第二节。

**引用口径**：本报告引 `.claude/kb/` 的行号，都是现查工作区里那份文件本身现在的行号（不是背景材料里数的）；
引 `research/prompts/m2-safety-r2-*` 的行号，同样现查那些文件本身的行号。
`research/prompts/m2-safety-r1-*` 不在这一轮的禁读清单里（禁读只列了 `m2-safety-r3-opus-*` 与
`m2-safety-r3-local-attack*`），第五节核对 ND 那条判决时读过它。

## 二、C283 怎么实现的，为什么两处实现方式不一样

C283 照 P1 那一格：「先推一次空发布抬 F，F 取 D16（发布语义） 已定项 1「抬 F 的上限」那一行的准入上限
（`.claude/kb/decisions/16-发布语义.md:36`），推完重判；一次准入最多 8 次发布（同文件 `:41`，
B = 4 + 2 k_tol，k_tol = 2）；写行那次发布之前不推（同一行的括注）」。

**挂载那一半是产品代码里的真实重试**（`crates/singlefs-core/src/mount.rs` 的
`mount_writable_with_admission_candidate_and_c283`，模型文件 `diff/diff-mount.rs.patch:151` 起）：
挂载准入被拒（`MountError::SpaceAdmissionRefusedBeforeAcquisition`）时，用 `establish_instance` 开头那一段
同款的读法（`choose_system_configuration` → `choose_root` → `scan_journal` → `replay_journal` →
`rebuild_previous_version` → `rebuilt_allocator`，都是 `mount.rs` 里已有的私有函数，
只是这一次不往下建实例——只为拿到 `raise_rollback_floor` 要的 `&mut PoolAllocator` 与
`&mut TransactionOutput`）重建出一份独立的分配器与「所选那一版」，算出 `rollback_floor_ceiling`，
够抬就调产品函数 `raise_rollback_floor` 抬一次，再整个重挂一次（`diff/diff-mount.rs.patch:175` 起，
`fn raise_the_floor_once_for_c283`）。**树表 0 条的一版（没有文件）不抬**——那一格几乎必是写行 /
暖机本身被拒，D16 已定项 1「写行那次发布之前不推」在这里成立，原样交回（模型文件里这一句注释
落在 `diff/diff-mount.rs.patch:212`）。这一半除了新加的函数，不改任何已有函数的签名或行为
（`admission.rs`、`allocator.rs`、`transaction.rs` 与 `mount.rs` 里已有的公开函数一个字节都没变，
只加了新函数与给 `InstanceStart`/`PoolAllocator` 加了一个新字段）；`cargo check -p singlefs-core`、
`cargo check -p singlefs-harness` 在打了全部四份补丁之后都是 0 警告 0 错误（跑前登记见附「验证」）。

**发布那一半是我自己测试文件里的重试**（`tests/r3_defense_common/mod.rs:194` 起的
`overwrite_with_c283`，内部调 `:207` 的 `raise_the_floor_once`），不是产品代码：`transaction::publish_overwrite`
只拿一个 `pool: &mut PoolWriter<'_, Device>`，没有 `raise_rollback_floor` 要的
`devices: &mut Vec<(DeviceIdentity, Device)>`——这个函数本身没法在内部发起一次新的抬 F 发布
（抬 F 那一串要重新拿设备句柄、重新起一个 `PoolWriter`）。这不是我省事：**这恰好是仓里今天没有
一层「挂着的会话」代码可以塞这个重试**——`crates/` 目前只有 core（管一次调用）与 harness（管测试）两层，
没有一个持有「这次挂载还开着」那把设备句柄、在 core 报 `SpaceAdmissionRefused` 之后决定要不要推一次
空发布再重试的中间层（真实部署里这层大概率是挂载点上面的驱动或 FUSE 循环，这个仓库还没写到那一层）。
我的测试文件在这里恰好扮演了那一层该做的事：拿着同一份 `devices`/`allocator`/`current`，
被拒就重建同一份读数、抬一次、重试一次——语义上与「产品路径以后接进来会怎么做」一致，
只是今天没有产品代码可以指给它。这一点也是 C283 的 `checks-owed.md:246` 那一行本来就写明的：
「今天没有任何条款、任何检查兑现它」——欠的正是这一层。

## 三、量出来的数

**装置**：`tests/m2_safety_r3_defense_a1_a3_a4_c283.rs`（模型目录同名文件），不经 `history.rs` 的模型对拍
（候选与 C283 都是这一轮新加的分支，理想模型不认得，硬套只会被判成「模型答不上」，我选择绕开它、
只打印结局，不带模型层断言——`.claude/singlefs-ai-sop/rules/show-me-test.md`「没实现的要明说」）。
每条组合：小盘 mkfs → 暖机 → 第一个文件 → 同一会话里反复覆盖写到候选自己的准入拒绝为止（`admitted_in_session`）→
两支之一收掉会话、重开可写挂载（S4：这一步失败就是出局，不往下量）→ 挂载做成就紧接着同样大小覆盖写最多 12 次，
数第几次放行（S4b 与 D3（空间分配） 已定项 9 的「步数」是同一个数：第 1 次就过 = 立刻达标，
> 12 = 记「吸收态」候选）→ 12 次都不过就再收一次会话重挂、再覆盖写一次、再试一次回退
（`roll_back_by_a_forward_publish`，这一步没有另包 C283，第四节会讲为什么）。
三个盘宽（`HistoryDeviceWidth::UnitAreaOf240Slots` / `256` / `384`）、两支（正常卸载 `mount::unmount` /
崩了 `CloseAndMountWritable` 同款——不调 `unmount`，直接丢设备句柄重开）、四个候选（`Baseline` 对照组 + `A1` + `A3` + `A4`）、
带不带 C283，共 3 × 2 × 4 × 2 = 48 条组合，跑一次 13–22 秒（三次独立跑逐字节相同，见第八节）。

**产物**：`research/prompts/m2-safety-r3-sonnet-model/rerun-output.log`（48 条组合全量原始输出）；
`rerun.sh` 拿本仓 `crates/` 打补丁、编译、跑，我在 `/home/fy5090/code/singlefs` 上原样跑过一次
（不碰工作区、只拷到 `/tmp` 打补丁），产物与我自己在冻结副本拷贝上跑的逐字节相同（`diff -q` 确认过）。

**结果表**（`admitted` = 会话里放行了几次覆盖写；`remount` 空白 = 不适用，因为 remount 本身没做成；
`S4b/D3I9` 列的数字是第几次覆盖写放行，`>12` = 12 次内没放行；`吸收` = 那一格进一步试过重挂 + 回退，
`abs-esc(rollback)` 表示回退这一步本身做成了，但没有再测回退之后能不能继续写）：

| 盘宽 | 候选 | 支 | C283 | remount | S4b/D3I9 步数 | 备注 |
|---|---|---|---|---|---|---|
| 240 | baseline | 正常卸载 | 关/开 | Applied | 1 / 1 | 不需要 C283 |
| 240 | baseline | 崩了 | 关 | **拒**（SpaceAdmissionRefusedBeforeAcquisition） | — | S4 出局 |
| 240 | baseline | 崩了 | 开 | Applied | **>12（吸收，abs-esc(rollback)）** | C283 救了挂载，救不了 S4b |
| 240 | A1 | 正常卸载 | 关/开 | Applied | 1 / 1 | 不需要 C283 |
| 240 | A1 | 崩了 | 关 | 拒 | — | S4 出局，与 baseline 同（A1 在挂载入口不改判法） |
| 240 | A1 | 崩了 | 开 | Applied | **1** | 比 baseline+C283 多救了 S4b 这一格 |
| 240 | A3 | 正常卸载 | 关/开 | **拒（`unmount` 自己就失败：RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite）** | — | 连清卸载都做不成，C283 挂不上号 |
| 240 | A3 | 崩了 | 关/开 | 拒 | — | C283 也救不动（抬 F 本身也无落点） |
| 240 | A4 | 正常卸载 | 关/开 | Applied | 1 / 1 | 不需要 C283 |
| 240 | A4 | 崩了 | 关 | Applied | **>12（吸收，abs-esc(rollback)）** | 「挂得上、写不了」原样成立 |
| 240 | A4 | 崩了 | 开 | Applied | **>12（吸收，abs-esc(rollback)，raised_floor_this_step=true）** | C283 确实抬了 F，仍救不回来 |
| 256 | 同上四候选 × 两支 | — | — | 同 240 的形状，**唯一差别**：baseline / A1 / A4 在崩了 + C283 这一格，S4b 从 240 的「>12 吸收」变成「1」——只有 A4 仍然 >12 吸收 | — |
| 384 | baseline / A1 / A4 | 崩了 | 开 | Applied | baseline/A1 = 1；**A4 仍 >12（吸收）** | 加宽到 384 槽，A4 的吸收态没有消失 |
| 384 | A3 | 正常卸载 | 关/开 | **仍然拒（同 240/256 的卸载本身失败）** | — | 这个故障面不随盘宽收窄 |
| 384 | A3 | 崩了 | 关 | 拒 | — | 与 240/256 同 |
| 384 | A3 | 崩了 | 开 | **Applied（这一档由拒转放行）** | **1** | 崩了这一支 C283 在够宽的盘上能救 |

## 四、逐条辩护：加上 C283 之后，A1、A3、A4 还出不出局

**A1（Sonnet 读法）+ C283：这一轮测到的 12 个取样点（3 盘宽 × 2 支 × 不分 baseline/A1 的挂载步骤，
但分 S4b 步骤）里一个都不出局**——挂载步骤 A1 与 baseline 逐字同一个判法（A1 只改「这次发布自己换下的块」
这个参数，挂载入口没有「这次自己的释放」这个概念，`released` 传空切片，等于没启用，这一点我在源码里
写了注释、也在实测里核对过：A1 与 baseline 的 `remount` 结果在全部 6 个「盘宽 × 支」格上逐字相同）；
真正体现 A1 自己算法的是挂载做成之后紧接着的那一次覆盖写（S4b），而这唯一一次能看出 A1 与
baseline 有差别的取样点——240 槽、崩了再挂、带 C283——A1 立刻放行（1 步），baseline 仍然 12 步以内不放行
（吸收）。**这是站得住的证据，但范围很窄**：256 与 384 槽上 baseline 加了 C283 自己就够了，
A1 没有额外贡献；240 槽这一格的差别来自 A1 把「这次发布自己要换下的块」记回可用，让准入的门槛
低了那么一点点，够不够用取决于这次覆盖写恰好卡在哪个边界上——不是「A1 总能救」，是「A1 在这个特定边界上
比 baseline 多一道门槛」。第二轮判决对 A1（Sonnet 读法）本身（不带 C283）的判词整句在
`m2-safety-r2-main-verification.md:36`：「一次会话里放行的覆盖写只多 1 次（240 槽 5→6）；
关掉再挂载第一次被拒的 k 与基线逐字相同，240 / 256 / 384 槽都是 4/5、5/6、9/10」——
这句话本身没有错，我的实测也复现了「A1 单独不救 S4」（崩了 + 不带 C283，A1 与 baseline 一样被拒）；
它只是没有回答「加上 C283 之后呢」这个新问题，而这正是这一轮要补的。

**A3 + C283：仍然出局，而且比第二轮判决记录的更差**。第二轮判决对 A3 的判词是「豁免只接在发布路径上，
挂载那一半一点没改善」（`m2-safety-r2-main-verification.md:71`）；我这一轮量到的是：A3 不但没被 C283 改善，
还多出一个第二轮没量到的故障面——**正常卸载本身做不成**。B1（用户 2026-09-25 定的正常卸载）走的是
「抬 F 那一串」（`mount.rs` 的 `raise_the_floor_through`），这一串要在分配器的拷贝上整串预演、拿得到落点
才真发；A3 的豁免把池推到物理上真正写满（`admitted_in_session` 到 17–27 次之后是 `PlacementRefused`，
不是准入拒绝），这时候连「抬 F 那一串」自己要写的树表单元都没地方放，`unmount` 直接返回
`RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`——这个失败发生在我的 C283 钩子够不到的地方
（我的 C283 钩住的是 `SpaceAdmissionRefusedBeforeAcquisition`，是「挂载准入」判定本身的拒绝；
「卸载序列自己的预演」是另一条判定，不产这个错误类型），所以不管带不带 C283，正常卸载在三个盘宽
上都做不成。崩了再挂这一支，240、256 槽上 C283 也救不动——道理相同：C283 要救的第一步就是
「推一次空发布抬 F」，而这一步本身也没有物理落点。384 槽这一档 C283 能救崩了再挂这一支
（池还没被 A3 的豁免填到物理写满的临界点），但正常卸载在 384 槽上仍然做不成——这个故障面不随盘宽收窄，
是 A3 这个读法本身的性质（它的语义就是「同样大小的净释放豁免」，天然会把池推向物理满载）。

**A4 + C283：仍然出局**。第二轮判决的核心批评——「池从『挂不上』变成『挂得上、写不了、删不了』」
（`m2-safety-r2-main-verification.md:77`）——在加了 C283 之后原样成立：三个盘宽、崩了再挂这一支，
C283 都确实抬了 F（`raised_floor_this_step=true`），但紧接着的覆盖写仍然被拒，12 次之内都不放行。
病根在 A4 的定义本身：A4 只改挂载入口的判法（只判「挂载期承诺量本身放不放得下」），挂载做成之后的
第一次普通覆盖写走的是 baseline 的判法（`admit_on_every_device_for_candidate` 对 A4 落到跟 baseline
一样的分支，我在 `admission.rs` 里的实现就是这样写的——A4 是「挂载专用」的候选，不影响发布路径，
这与第二轮判决对 A4 的定义一致）；C283 抬 F 能追回的是「defer 待释放、已经过了窗口该回可分配集合却还没回」
的那部分空间，追不回「已分配（仍在用）」那部分——A4 让挂载放行的代价正是绕过了「已分配太多」这一判断，
挂上去之后立刻就要用到那部分被绕过的空间，而那部分从来不是 C283 抬 F 能碰的。**这是一个可以推翻的判断**：
如果换一个「删得比写得快」的历史（不是同一份文件反复覆盖写，而是先删大量数据再写），A4 挂上去之后
第一次写可能因为有大量刚过窗口的 defer 空间被 C283 释放而放行——我这一轮的历史形状（单文件反复覆盖写）
测不出这一格，第五节与「没做什么」会写清楚。

## 五、第二轮判决里，哪一句够不着它引的证据

**ND 单独用出局：核过，够得着，没找到够不着的地方。** 判决原文（`m2-safety-r2-main-verification.md:51`）：
「判：单独用出局（第一、二两轮都中，三轮里已经过半）」，依据两条：
- 「第一轮：384 槽 k = 20、21 上，24 次回退全被准入拒」（`:53`）——核对 `m2-safety-r1-main-verification.md:42`
  原文「候选 A（去掉 defer 重复扣）……384 槽 k = 20、21 上 24 次回退全被准入拒（基线没有这种全灭）」，
  逐字对得上（候选 A 就是 ND：判决第二节 ND 那一格标注「第一轮的候选 A」）。
- 「这一轮 Opus E5：768 槽截断成 0 那次发布，有 3 个崩溃点挂载被拒，根环 24 条根逐条回退全被拒」（`:54`）——
  核对 `m2-safety-r2-opus-output.md:321` 原文「**ND（第一轮候选 A）768 槽：3 个崩溃点挂载被拒、根环 24 条根
  逐条回退全被拒**（`rb-none/24["SpaceAdmissionRefusedBeforeAcquisition"]`）」，逐字对得上；同一行「A2 同一格
  30 个崩溃点全部挂载成、抬 F 一步写回」也与判决第 55 行「A2 一族……在同一格上 30 个崩溃点全部挂载成功」对得上。
  两轮各一次独立打中，「三轮里已经过半」这句字面站得住。

**C283 是共用病根：核过，找到一处够不着的具体句子。** 判决原文（`m2-safety-r2-main-verification.md:93`）：
「**抬 F 能救，但今天只有测试入口**：经测试入口抬 F 一步就写回，只有 240 槽的 A1、A3gt 例外。
E1 里 `[raiseF,ow]` 在每个盘宽、每条臂上都是 `ok`。」——它引的证据在 `m2-safety-r2-opus-output.md:314`
（「4.1 崩在删除类发布中间」那一节的第 4 句）：「各改法：A1–A4 都不含抬 F，都中；「抬 F」这一步
（只经测试入口、当成用户动作做）**在 240 槽 A1、A3gt、A3ge（单抬 F）三格上也中**。」——原文列了
**三个**例外（A1、A3gt、A3ge），判决的转述只留了两个（A1、A3gt），**漏掉了 A3ge**。这不是无关紧要的
遗漏：A3ge 正是这一轮我要辩护的 A3 那一族里最贴近「产品会不会选的读法」的一支（A3 的三种读法 ge/gt/d 里，
`≥` 读法 A3ge、A3d 是判决第二节标「打中」最重的两种），而它恰好是「单纯抬 F 也救不了」的那三格之一——
这与我这一轮独立测到的结果吻合（第三、四节：A3 + C283 在 240、256 槽的崩了再挂这一支，C283 完全救不动；
正常卸载这一支在三个盘宽上都因为「卸载序列自己的预演没有落点」而失败，比单纯的准入拒绝更差）。
判决第 93 行如果把 A3ge 也列进例外，「E1 里 `[raiseF,ow]` 在每个盘宽、每条臂上都是 `ok`」这句概括就要
去掉「每条臂」三个字，或者明说「A4 那一族里每条臂都是 ok，A3ge 单独例外」——按今天的字面，「共用病根」
这个框架本身没有错（C283 缺失确实让多数臂的 S4b 打不开），但它在描述「抬 F 单独救不救得动」时，
把 A3ge 的失败悄悄归并进了「例外只有两个」，这句概括够不着它自己引的那一行证据。

## 六、判决条目 / 辩方判定 / 证据

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| A1（Sonnet 读法）出局（`m2-safety-r2-main-verification.md:30`「判：出局（两种读法都中）」） | **够不着**——原判决只测「不带 C283」；加上 C283 之后，本轮 3 盘宽 × 2 支共 12 个取样点里，A1 一个都不出局（S4=0，S4b/D3I9 步数恒为 1）；唯一一处 A1 优于 baseline+C283 的取样点（240 槽、崩了再挂）也没有出局 | 本报告第三、四节；产物 `research/prompts/m2-safety-r3-sonnet-model/rerun-output.log`（`candidate=a1` 与 `candidate=baseline` 逐行对照） |
| A3 出局（`m2-safety-r2-main-verification.md:61`「判：出局」） | **站得住，而且更重**——加了 C283 之后不但没救，还新增「正常卸载本身做不成」这个第二轮没量到的故障面（3 盘宽全中） | 本报告第三、四节；`rerun-output.log` 里 `candidate=a3 branch=normal_unmount` 全部 6 行 `remount=RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite` |
| A4 出局（`m2-safety-r2-main-verification.md:75`「判：出局」） | **站得住**——加了 C283 之后，崩了再挂这一支在 3 个盘宽上全部保持「挂得上、写不了」，C283 确实抬了 F 也救不回来 | 本报告第三、四节；`rerun-output.log` 里 `candidate=a4 branch=crash c283=true` 三行 `steps_to_success=>12`、`raised_floor_this_step=true` |
| ND 单独用出局（`:51`） | **够得着**，本轮没找到够不着的地方 | 本报告第五节第一段；`m2-safety-r1-main-verification.md:42`、`m2-safety-r2-opus-output.md:321` |
| C283 是共用病根（`:88`–`:96`） | **框架站得住，但 `:93` 那一句转述丢了一个例外（A3ge）**，够不着它自己引的 `m2-safety-r2-opus-output.md:314` | 本报告第五节第二段 |

## 七、什么会推翻这些结论

- **A1「站得住」**：只要有人在别的历史形状（比如同一份文件反复覆盖写换成多文件、或换一种盘宽序列）上量出
  A1+C283 在某个取样点仍然 S4 或 S4b 出局，这条就要收回「站得住」，改记「只在测过的窄形状上站得住」。
  我已知的局限：只测了单文件反复覆盖写这一种历史（第八节）。
- **A3「更差」**：只要有人证明「正常卸载序列自己的预演」这个失败点，在其它（非 A3）历史下、盘足够宽时也会出现
  （也就是说这不是 A3 独有的、而是所有能把池推到物理写满的读法共有的），这句判断要从「A3 特有」
  改成「近满盘的共同后果，A3 只是最容易触发它的一种读法」。
- **A4「仍然出局」**：只要有人在「删得比写得快」的历史上量出 A4+C283 的 S4b 放行（第四节末段已经预告这一格），
  这条判断要收窄成「在测过的这种历史形状下仍然出局，别的形状没测」。
- **C283 转述丢了 A3ge**：这条是纯文本核对，唯一会推翻它的情形是我核错了行号或原文——已经现查过两遍
  （`grep -n` 命中且原样贴出），行号与原文引用都可复核。

## 八、没做什么

- **C283 的挂载那一半改了产品代码（`mount.rs`/`admission.rs`/`allocator.rs`/`transaction.rs`），
  但只在我自己拷贝的冻结副本与一份从主仓 `crates/` 拷出来的临时目录上编译、跑过；没有跑门禁、
  没有跑 `cargo clippy --all`、没有跑重型测试（`layer0`、54/55/57/59 号），也没有跑 `singlefs-checker`
  的池级 checker——这份代码只为量三个候选加 C283 的准入结局，不主张它能进产品。**
- **发布路径的 C283 没有写成产品代码**：第二节解释过为什么（`publish_overwrite` 没有 `raise_rollback_floor`
  要的设备句柄），这不是我省事，是仓里今天确实缺一层「挂着的会话」代码可以塞它——这本身也是
  C283 欠账（`checks-owed.md:246`）没写清楚的一部分，我在报告里写明、不代它补上。
- **A4「删得比写得快」那种历史没有造**：第四节已经预告，需要另一个测试装置（先建多个文件再批量删除），
  这一轮时间上没有做；这是「什么会推翻」里明确列出的一条，留给下一轮或另一个岔路。
- **A3 三种读法（ge/gt/d）没有分开测**：我的 `S4Candidate::A3` 只实现了「≥」这一种读法（对应判决里的
  A3ge），因为它是分工表点名要辩护的那一支；A3gt、A3d 没有单独跑，第五节引 Opus 数据时提到它们只是
  为了核对判决原文，不是我自己的实测。
- **回退（`roll_back_by_a_forward_publish`）那一步没有另包 C283**：`tests/r3_defense_common/mod.rs:279`
  的 `rollback_once` 注释里写明「这里不另包 C283——只测今天字面代码能不能回退出这个格」；吸收态那几行
  标的 `abs-esc(rollback)` 只表示「回退这一步本身被接受」，没有再测回退之后能不能继续写，这是故意留白、
  不是漏测——分工表把「抬 F 抬掉用户还要的回退候选」这类回退相关的深挖交给了 Opus 那条腿。
- **没有另建 kb 快照**：这一轮的背景材料前言已经写明「没有另建 kb 快照拷贝」，我引用的 kb 行号都是现查
  `.claude/kb/` 工作区当前的文件（第一节已声明）。
- **中途的一次流程失误，已经改正**：我最初直接在共享的冻结副本 `/tmp/claude-1000/safety-r3-frozen/`
  上打了补丁、跑了测试（应该在它的一份拷贝上做），发现之后已经用编辑前保存的原文
  （`/tmp/claude-1000/m2-safety-r3-sonnet/pristine/{admission,allocator,transaction,mount}.rs`）
  整份换回，交回前 `sha256sum -c` 复核过 130 个文件全部 OK。冻结副本里我编译产生的 `target/` 目录
  （约 1.3 GiB 的构建缓存，不在 `crates-sha256sums.txt` 的校验范围内）已经删掉（`rm -rf
  /tmp/claude-1000/safety-r3-frozen/target`）。这一轮真正用来出数的编译目录是
  `/tmp/claude-1000/m2-safety-r3-sonnet-rerun-check/` 与
  `/tmp/claude-1000/m2-safety-r3-sonnet-rerun2/`（都是 `rerun.sh` 从主仓 `crates/` 拷出来的临时目录，
  不是冻结副本本身），已经在交回前删掉；`/tmp/claude-1000/m2-safety-r3-sonnet/pristine/` 与
  `/tmp/claude-1000/m2-safety-r3-sonnet/*.log` 留着，是这份报告的证据来源，没有删。
