# m3-close-g4-rewrite-r1 攻方腿（Opus）报告（2026-10-22）

立场：找反例。攻击面三样：过度外推与越位、内部矛盾、哪一条最脆弱。按 `crates/` 今天的实现谈：主工作区的五份代码与开工快照 `research/prompts/m3-close-g4-rewrite-r1-snapshot/code-sha256.txt` 逐份 sha256 相同（`sha256sum -c` 没有一行不 OK）。量过的数都出自草稿副本上的原型，没有在入库装置上跑过。

## 复跑

模型目录 `research/prompts/m3-close-g4-rewrite-r1-opus-model/`。原型是一个工作区成员 crate，只 `cargo build`，编出来的二进制直接执行。崩溃态用 `singlefs_checker_tier::crash_state_enumeration::enumerate_crash_states_with_expansion` 加 `crash::full_expansion` 取，每一段都展开，原地覆写的撕裂态也算在内；每个态上崩溃点重放照常跑自己的恢复与池级 checker，另加一个观察者。`toggles.patch` 给核心层加两个只在副本里认的运行时开关：`SINGLEFS_PROTO_MUT_B1`（盘表下标 0 那块盘写完系统配置新槽之后，同一段里把另一槽清成全 0）、`SINGLEFS_PROTO_GATE_NEVER_PASSES`（补写闸恒不过，等于撤掉丙'）。

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-close-g4-rewrite-r1-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen <开工时那棵树（crates/、Cargo.toml、Cargo.lock、.cargo、.claude/heavy-thread-default）> --work <新的草稿目录>
```
每条流的计数写进 `<草稿目录>/out/<流>.counts`，与 `out/` 里同名文件比，打 `name=rerun_compare ... same=yes|no`。编译用 `-j 4`：8G 上限下 31 个并行的 rustc 撞过一次 OOM（包装报 `Result=oom-kill`，命令退出码 143）。

| 文件 | sha256 |
|---|---|
| Cargo.toml | b9a147cc3c044d17de6789ae106b0470ffce818df38c962f4273238582913a14 |
| src/main.rs | c1dd19ea64b02995becf36ce1b9c12a5e48705d0630053ee6bceb6ac167305d7 |
| toggles.patch | 83613ec3919602ec1e21852ba667ce6b8d6b6787f066100ea19ffe132662bcc1 |
| tools/rerun.sh | a8f1957cb1f4c38ca6acadf46cb1007f4b894439399498717e1e3cb6073940f5 |
| out/hand_made_image_both_slots.counts | 53e720b0aff24ab07b7c8966d0931e8c0679476a9aba83bc209369adb1ff9d04 |
| out/hand_made_image_both_slots_and_a_unit.counts | cd2137662d235eb4178cbe9d4c9a4166ecddaf553da9fdf637fe4b35a2e839a9 |
| out/hand_made_image_both_slots-SINGLEFS_PROTO_GATE_NEVER_PASSES.counts | 71be2634ca48e2cd9a566824b62aecf53138072cec52ecd516794a327a3c4837 |
| out/make_filesystem_on_fresh_devices.counts | 40c8edfb7dcc269d0efef60450fe82f72a2a6ceb0947c1d56debff3b1eb3facc |
| out/make_filesystem_on_fresh_devices-SINGLEFS_PROTO_GATE_NEVER_PASSES.counts | c451ca48f75683f815aeb4d98dedb7b25b2d891af156eb5f60bad04783136dd8 |
| out/make_filesystem_over_a_used_pool.counts | 0294d25ffbcbf19a261ede2580b4d9f36bb634ce315be1937a1efea2698c7129 |
| out/overwrite.counts | af3a11ae7a98fd77063f0f281b31b7dde1b9763881d8aa7739e59fef7fa6c3af |
| out/remount.counts | e18125066d4bad8d4d8a9ed1227d788f87c83da165ef0d8d31f1d2da19a7362c |
| out/remount-SINGLEFS_PROTO_MUT_B1.counts | 5ded0444788bca7b49b83260e8dfa90fb974e79981621bd7f3deacf6770353b2 |
| out/remount-SINGLEFS_PROTO_MUT_B1+SINGLEFS_PROTO_GATE_NEVER_PASSES.counts | 0ed3e398edcee4ec7c39e3c13c73934c6a5dc29ea6574932b1284601e3ad2c31 |
| out/rewrite_after_a_make_filesystem_crash.counts | 23d97d05b24064fdbea6a7d520492f40f92427ed1d1a88650f8a956a31ac2c0d |

## 各格判定一览

| 格 | 判定 | 依据 |
|---|---|---|
| V1 | **打中（量过）**：没有读故障的历史留得下两槽全废，来源只有一个：mkfs 最后一段（新盘 6 态，用过的盘 40 态）。mkfs 之后，取号写两轮、补写中途崩溃这两种位置都没有新造出全废的盘（量过）；发布轮换是推的 | 打中一 |
| V2 | **打中两处**：① 只收窄 G4、不把闸并进 U6 的复算，会把「撤掉丙'」那种坏实现判成合法（量过，分辨臂）。② 丙' 让 MUT-B1 那一类坏实现挂得上，照旧、收窄两种判法都摸不到它，D13 第 187 行交给 G4 的那条性质已经没人判（量过，不分辨臂）。在正确实现上，照旧与收窄没有一张可达的盘面判得不一样（推的） | 打中二、三 |
| V3 | **打中（推的）**：乙等于「甲 + 一条手造的闸不过盘面的新测试」，2408 的判别力不比甲多，反倒把打中三的空当钉成了期望行为；甲最脆弱：三条变异全押在「闸一因一处单元坏而不过」上 | 打中四 |
| V4 | **没打中（推的）**：C670、Q3 两拒都在补写闸之前发，丙' 不改它们会不会被调到；同形的空当（只在拒的时候判）照样在 | V4 一节 |

自己提的改法（G4-乙、G4-丙、甲-二）都只在我的模型上量过或推过，**被攻过零轮**：

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| G4-乙：收窄 G4，同时把判官自己复算的五件闸并进 U6 那条判据 | 打中二（撤掉丙' 的坏实现判合法） | 推的（没实现；它要改掉的「只判 U6 判合法」是量过的） |
| G4-丙：对盘面判，无读故障的崩溃态上某块盘没有自证槽就记发现，不管拒没拒 | 打中三（MUT-B1 那 42 态） | 推的（没实现；42 态全挂上是量过的） |
| 甲-二：甲加一条反向测，再加一行变异「补写闸恒不过」 | 打中二在测试与变异表里的那一格 | 推的 |
| 甲的盘面改成两件闸一起不过 | 打中四 | 推的 |
## V1：没有读故障、只有崩溃的历史留不留得下「某块盘两槽都自证不过」

**判定：留得下，但只有一个来源——mkfs 最后一段。**（量过）mkfs 之后的历史里，三种崩溃位置都摆不出新的两槽全废的盘（取号写两轮量过，补写中途崩溃量过，发布轮换按代码推的）。所以 G4 注释那句「正确实现在没有读故障的历史里留不下它」照字面说过头了：它漏了 mkfs 中途崩溃，而 D22 已定项 8 已经把这种盘面写成合法盘面。缩到「mkfs 返回之后的历史」这句才成立，D13 写 G4 射程时已经这样缩了。

### 打中一：mkfs 最后一段（新盘、用过的盘都中）

- 历史：mkfs 写到最后一段崩溃。最后一段是每块盘两个系统配置槽各一写，同段里没有屏障（`crates/singlefs-core/src/make_filesystem.rs:730` `slot_index * u64::from(parameters.geometry.fixed_structure_slot_spacing),`，写在同一个循环里，循环后才逐盘屏障）。三条第 0 代根在这一段之前已经由 FUA 各成一段，单元写在更早的屏障之前。所以段内子集「盘 1 两槽落了、盘 0 两槽都没落」摆得出：盘 0 两槽全 0，盘 1 两槽自证过，根和单元都在。
- 量过（原型 `make_filesystem_on_fresh_devices` / `make_filesystem_over_a_used_pool`，崩溃态用 `enumerate_crash_states_with_expansion` + `full_expansion` 全部展开）：

| 流 | 状态数 | 两块盘都没有自证槽 | 只有一块盘没有（盘 0 / 盘 1） | 这些态上可写挂载（丙' 之后） | 补写闸恒不过时（丙' 之前），G4 判官 / 只判 U6 的判官 |
|---|---|---|---|---|---|
| 新盘 mkfs | 4117 | 4102（恢复 `NoValidSystemConfiguration`） | 3 / 3（恢复读回第 0 代空池） | 6 个全挂上，挂上后每块盘都有自证槽 | 6 个都以「读阶段判完之后系统配置又读不出」拒；G4：判不合法（只差 G4 那一条），U6：合法 |
| 用过的盘重做 mkfs | 31195 | 31122（整池拒）+ 8（作废没持久，读回旧池） | 20 / 20 | 40 个全挂上，挂上后每块盘都有自证槽 | 没跑（新盘那一条已经分出两种判官） |

- 原样输出（`out/make_filesystem_on_fresh_devices.counts` 里的几行）：
  `name=make_filesystem_on_fresh_devices devices_without_a_self_verified_slot=[0] recovery=no_file remount=mounted devices_without_a_self_verified_slot_after_the_mount=0 count=3`
  `name=make_filesystem_on_fresh_devices devices_without_a_self_verified_slot=[0] recovery=no_file remount=refused reason=SystemConfigurationUnreadableAfterTheReadStage verdict_with_g4=illegal[SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults] verdict_u6_only=legal count=3`（补写闸恒不过）
- 条文：`.claude/kb/decisions/22-单元原子性怎么合成.md:181` 写了这一形是合法盘面（「后一种里系统配置的冗余可以缺，缺新槽的那块盘的可写挂载按现行补写条款处置」）。`.claude/kb/decisions/13-验证路线.md:187` 把这一形划出 G4 的射程（「mkfs 返回之前的崩溃态不在 G4 的射程」）。
- 四句：
  1. 分不分辨臂：不分辨。丙' 之后这 46 个态都挂上，判官到不了 G4，照旧与收窄在这里判得一样。丙' 之前（量过）G4 会把合法的 mkfs 盘面判成「造出盘面的那一段不对」，U6 判合法。
  2. 系统看不看得到：看得到。判官看不到历史，但 D13 用「崩溃注入的候选段都取在 mkfs 之后」把这一形挡在外面，靠的是调用方，不是判据本身。
  3. 满足哪一个分句：G4 注释那句全称句「正确实现在没有读故障的历史里留不下它」（`crates/singlefs-harness/src/mount_transparency.rs:3064`）。D13 第 187 行带射程的那句不中。
  4. 改法还中不中：照旧、收窄都不中（到不了判官）。如果以后崩溃注入从 mkfs 中途起，两种都要先判「所选那一版是第 0 代、盘面是 mkfs 留下的」；判法与改法对这一格的作用是推的。

### 没打中：mkfs 之后的三种崩溃位置

- **取号写两轮中途崩溃**（量过）：流 `remount`（第一个文件之后关闭、再可写挂载：取号两轮、写行、暖机；段形 `[2, 2, 14, 2, 1, 2, 12, 2, 1, 2]`），20519 个态里，某块盘没有自证槽的是 0 个（`name=remount states count=20519`，没有别的计数行）。机制：每块盘一次只写较旧的那一槽，世代号 = 本盘自证过的最大值 + 1，槽号 = 世代号 mod 2（`crates/singlefs-core/src/transaction.rs:643` `(slot_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing,`）；两轮之间有一道池屏障。同一段里同一块盘只有一个系统配置写。
- **补写本身中途崩溃**（量过）：流 `rewrite_after_a_make_filesystem_crash`。盘面是新盘 mkfs 崩在最后一段、盘 0 两槽都没落的那一张；在它上面可写挂载一次（补写盘 0），录下这次挂载的写，枚举它的崩溃态。段形 `[2, 2, 2, 1, 2, 2, 1, 2]`，35 个态，盘 0 仍然全废的 3 个（都在取号第一轮之内），3 个再挂都挂上：`name=rewrite_after_a_make_filesystem_crash devices_without_a_self_verified_slot=[0] recovery=no_file remount=mounted devices_without_a_self_verified_slot_after_the_mount=0 count=3`。盘 1 一个态都没有变成全废。补写只是把原来就全废的那块盘留着，不会新造出一块。
- **发布轮换**（推的）：发布写序是单元 → 屏障 → 记录 → 屏障 → 根 FUA → 轮换 → 屏障，轮换每盘一写，两次轮换之间至少隔着记录那一道屏障。覆盖写那条流有 1048588 个态，过了约 10⁶ 的上限，没跑（见「这条腿自己的限度」）。
- 读故障下同样推不出全废：直读把较新的那一槽藏掉时，写会落在较新那一槽上，较旧那一槽还在。这一条是推的，不在 V1 的射程里。

## V2：G4 那一臂今天的判法与各个改法在哪张盘面上判错

先说 G4 什么时候会被调到：只有可写挂载以「读阶段判完之后系统配置又读不出」拒了，判官才进 G4（`crates/singlefs-harness/src/mount_transparency.rs:3064` 起那一臂）。没有读故障时，这个拒因只有一个来源：某块盘两槽全废，而且补写闸不过（`crates/singlefs-core/src/mount.rs:5797` `Some(_gate_not_met) => return Err(device),`）。闸过了就把那块盘记进补写（`crates/singlefs-core/src/mount.rs:5794`），挂载往下走，判官根本不调。

### 打中二（量过，分辨臂）：只收窄 G4、不动 U6 的复算，会把「退回丙' 之前」的坏实现判成合法

- 盘面：红的那条测试同形的手造盘面（第一个文件之后，盘 0 两槽各在偏移 2048 翻一个字节）。另有 MUT-B1 在 `remount` 流上摆出的 42 个崩溃态。
- 坏实现：补写闸恒不过（原型开关 `SINGLEFS_PROTO_GATE_NEVER_PASSES`，也就是把丙' 撤掉）。
- 量过：
  `name=hand_made_image_both_slots remount=refused reason=SystemConfigurationUnreadableAfterTheReadStage verdict_with_g4=illegal[SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults] verdict_u6_only=legal count=1`
  `name=remount devices_without_a_self_verified_slot=[0] recovery=file_read remount=refused reason=SystemConfigurationUnreadableAfterTheReadStage verdict_with_g4=illegal[SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults] verdict_u6_only=legal count=42`
- 各候选在这张盘面上判什么：
  - 照旧：不合法（G4）。抓得到，但写的原因是错的：它说「造出这份盘面的那一段不对」，其实错的是这次拒本身（条文要求闸过就补写）。
  - 收窄到「补写闸也不过」：判官自己复算闸，闸是过的，所以 G4 不报；U6 那条判据（`system_configuration_unreadable_after_the_read_stage_on_the_crash_image`，`crates/singlefs-harness/src/mount_transparency.rs:2419`）里没有闸，复算为真，结论是**合法**。撤掉丙' 这种退化就漏了。这一格的判定是推的：结论 = 只判 U6 的判官的结论，而那个结论是量过的。
  - 收窄 G4，同时把闸并进 U6 的复算（判据 = 判不出 ∨ 某块盘全废且判官自己复算的闸不过；建议起新名，例如 G4-乙）：闸过而拒，U6 判「判据复算为假」，不合法；闸不过而拒，G4 判不合法（历史错）。这张盘面上判对。推的，没实现，被攻过零轮。代价是判官要独立再写一份五件闸（`.claude/rules/fs-design.md`「运行时与 checker 必须用不同的算法」），闸三那份「比所选那一版新」的读法在被抛弃的时间线残留上，判官与实现可能各取一边（sysconf-r3 判决 U4 那一行已记过「照字面会把被抛弃时间线的残留算成更新」）。
- 四句：分辨臂（照旧、只收窄、收窄加 U6 三条臂判得都不一样）；系统看得到（判官手里有崩溃镜像，能自己复算闸）；满足的是 D23 已定项 14 拒因单子那一条「判据在崩溃镜像上复算成立」的分句（U6 第二条）；只收窄这个改法在这一格上照样中。

### 打中三（量过，不分辨臂）：丙' 让 G4 原本要抓的那一类坏实现摸不到判官，几种改法一起失效

- 坏实现 MUT-B1（盘 0 写完新槽后，同一段里把另一槽清成全 0；原型开关 `SINGLEFS_PROTO_MUT_B1`，补丁在模型目录 `toggles.patch`）。G4 当初就是为它加的（`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 文件头第 12 行起那一段）。
- 流 `remount`，补写闸正常（丙' 在）：20582 个态，盘 0 两槽全废 42 个，**42 个都挂上**，挂上之后每块盘都有自证槽：
  `name=remount devices_without_a_self_verified_slot=[0] recovery=file_read remount=mounted devices_without_a_self_verified_slot_after_the_mount=0 count=42`
  同一次枚举里崩溃点重放自己的判（只读恢复加池级 checker）：`violations: 0`，`first_violation: None`。跟正确实现那一趟相比，只有 I-7.7 在 42 个态上改报不适用。
- 结论：丙' 落地以后，MUT-B1 这一类在崩溃点重放和第二、三截里都没人报（第二、三截那一半是推的：原型只量到「挂上了」，第三截的幂等判没跑）。照旧、收窄、收窄加 U6 都要先有一次拒才进得来，所以三个改法一起失效。D13 第 187 行（`.claude/kb/decisions/13-验证路线.md:187`）把「无读故障的崩溃态上每块盘至少一份自证过的系统配置」这条性质交给 G4 判，这句话在丙' 之后已经不成立：这条性质只在闸不过的那几张盘面上有人判。池级 checker 对单盘全废只走旁路报告（`crates/singlefs-checker/invariants/03-journal.md:12` 「单盘全废不判违例，走旁路报告」，函数 `crates/singlefs-checker/src/image.rs:1492`），这个报告 checker 档与 harness 档没有人读（`grep -rn system_configuration_redundancy_report` 只命中定义那一份）。
- 能分辨的候选（起新名 G4-丙，推的，被攻过零轮）：不再挂在拒因上，改成对盘面判：崩溃态没注入读故障、某块盘一份本池自证过的系统配置都没有，就记一条发现，不管可写挂载拒了还是挂上了。依据是 V1：mkfs 之后的历史留不下这种盘面，所以它出现就说明某一段写序错了。挂在第二截的崩溃镜像与第三截的二次崩溃镜像上，判一次。它在三张盘面上判得与今天不同：MUT-B1 那 42 个态（今天什么都不报，G4-丙 报）；手造的两槽全废、闸过的盘面（今天挂上不报，G4-丙 报）；mkfs 最后一段那 46 个态不进第二、三截，判不到。
- 四句：不分辨照旧与收窄（病根在两者共用的前提「G4 只在拒的时候判」，照证据纪律那张表另立一笔账）；系统看得到（只读崩溃镜像的两槽）；满足的是 D13 第 187 行写的那条性质，不是 G4 注释的全称句；照旧、收窄、收窄加 U6 在这 42 格上都还中，只有 G4-丙 修得到。

### 照旧 vs 收窄：在正确实现上没有一张可达的盘面判得不一样

由 V1 加上闸的代码（推的）：mkfs 之后，没有读故障的崩溃态上不会出现两槽全废，所以正确实现不会以这个拒因拒，G4 永远不被调。两种判法只在三种地方判得不同：坏实现（打中二）、手造盘面（V3）、mkfs 留下的盘面（第二、三截够不着）。

## V3：甲、乙两条测试改法，改完之后 2406–2408 三条变异各由谁杀

先查现状（`sed -n '2406,2408p' crates/mutations.tsv`）：2406「G4 判官整条不判」与 2407「注入过读故障的崩溃态也判」都由 harness 档 `writable_mount_refusal_on_a_crash_state_without_read_faults` 杀，那两条测试的盘面已经走了甲：盘 0 上第一个文件的树表那一份翻一个字节，闸一不过（同一文件第 142 行 `fn corrupt_the_last_unit_copy_of_the_first_file_on_device_zero`）。只有 2408「只判第二截不判第三截」靠 checker 档那条红的测试杀。第三截里被变异的那一臂在 `crates/singlefs-checker-tier/src/crash_injection.rs:4730`。

量过的两张盘面（harness 档直接调核心层的可写挂载与判官，没经第三截）：

| 盘面 | 挂载 | 带 G4 的判官 | 只判 U6 |
|---|---|---|---|
| 两槽各翻一字节（红的测试今天的盘面） | 挂上，挂上后每块盘都有自证槽 | 没调 | 没调 |
| 两槽各翻一字节，加盘 0 树表那一份翻一字节（甲） | 以「读阶段判完之后系统配置又读不出」拒 | 不合法，只差 G4 | 合法 |

原样输出：`name=hand_made_image_both_slots remount=mounted devices_without_a_self_verified_slot_after_the_mount=0 count=1`；`name=hand_made_image_both_slots_and_a_unit remount=refused reason=SystemConfigurationUnreadableAfterTheReadStage verdict_with_g4=illegal[SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults] verdict_u6_only=legal count=1`。

| 候选 | 2406 | 2407 | 2408 | 测的东西与判法对不对得上 |
|---|---|---|---|---|
| 甲（盘面加一处单元坏，断言不变） | harness 469 行那条，不变 | harness 445 行那条，不变 | 改过的 8964 那条：第三截照旧以 G4 拒，变异把 G4 计数清掉就红（推的：第三截没跑，盘面与判官那一半量过） | 照旧与收窄（闸一在判官里也不过）都对得上。和 G4-丙 对不上的地方只有这一处：G4-丙 在这张盘面上另记一条发现，「第三截记一条」要改成两条 |
| 乙（断言再挂做成、两槽补写、I-8.4 判同；G4 另找盘面） | 不变 | 不变 | 要另写一条测试。闸不过的盘面在 mkfs 之后的历史里摆不出来（V1），只能手造，最省事的还是甲那张。所以乙等于「甲 + 一条新测试」，2408 的判别力不比甲多 | 新断言「闸过的全废盘面上第三截零发现」把打中三那个空当钉成了期望行为：以后采 G4-丙，这条测试就红。照旧与收窄下对得上 |
| 甲加一条反向测（起新名 甲-二，推的，零轮）：同一张闸过的盘面，在补写闸恒不过的副本上断言 G4 报 | 不变 | 不变 | 同甲 | 补上打中二那一格：变异「补写闸恒不过」今天在 `crates/mutations.tsv` 里没有对应行，加一行，点名 harness 档那条测试 |

**哪一条最先丢判别力（打中四，推的）：甲。** 甲的 2408 判别力全押在「盘 0 上某个单元坏了，闸一就不过」。只要闸一放宽（例如改成单元在任一盘验得过就补写，并顺手修单元），或者 `units_of_the_selected_version` 不再含被翻的那个单元（树表单元换位、带文件那一版的清单改成只列这次重写的那几个），盘面就重新过闸、挂上，测试会像今天这样再红一次，2408 又没人杀。harness 档 2406、2407 的那两条测试押的是同一处，三条一起丢。乙的新断言不押闸的件数，但它的 2408 也得靠同一类手造的闸不过盘面，所以只是丢得一样快，不是更慢。能缓一点的写法：盘面同时让两件闸不过（闸一，再加单元区里写一个写序实例更大的本池单元，触发闸四），单改一件闸不会让它过闸。这是推的，没造过。
- 四句：分辨臂（甲、乙押同一类盘面，甲多押一件闸）；系统看得到（测试自己造盘面）；满足的是「`crates/mutations.tsv` 一行必须由点名的测试判红」那条登记口径；甲、乙两个改法在「闸放宽」这一格上都还中，只有两件闸一起不过的盘面修得到。

## V4：C670 那一臂、Q3 那一臂会不会因丙' 一样失效

**没打中（按代码推的，没量）。** 两臂的拒都排在补写闸前面，丙' 不改它们会不会被调到：

- 写序：重建分配器（Q3 那一拒在 `rebuilt_allocator` 里，调用在 `crates/singlefs-core/src/mount.rs:8458` `let rebuilt = rebuilt_allocator(`）与判据 N-配置 那一拒（C670，`mount.rs` 第 8100–8170 行那几处 `NewerStateStillUnreadableAfterOneReread`）都在 `establish_instance` 之前（调用在 `crates/singlefs-core/src/mount.rs:8490`），补写闸在 `establish_instance` 里面（`crates/singlefs-core/src/mount.rs:5786` 起）。闸放过一块盘，只影响后面的取号与发布，不改这两拒发不发。
- C670 判据：c_见证 取全部自证槽 tail 的最大值。补写写进去的 tail 是取号那一刻读到的同一个见证值（`write_the_witnessed_acquisition` 每轮都带 `witnessed_journal_tail`），所以补写盘不会让 c_见证 变大。补写中途崩溃，最多是补写盘多出一槽带同一个 tail，C670 判据的值不变（推的）。
- Q3 判据只看根环，与系统配置槽无关。
- 同形的空当照样有：C670 与 Q3 跟 G4 一样，只在拒的时候判。哪天也有一条「闸」让它们的拒变成做成，就会出现打中三那样的失效。今天没有这样的例外，所以不算打中。

顺带（不是 V4 的格）：`crates/singlefs-checker/invariants/03-journal.md:12` I-8.5 那一行写「另报哪块盘少于两槽自证过、可写挂载会拒」。丙' 之后，闸过的盘不会拒，这半句已经和实现对不上（打中三的旁证）。

## 没打中的形状

- 取号写两轮中途崩溃，正确实现：`remount` 流，20519 个态，没有一块盘变成没有自证槽。
- 补写本身中途崩溃：35 个态，只留下原来就全废的盘 0，盘 1 没有一个态变成全废。
- 用过的盘重做 mkfs，补写闸四（单元区里有写序实例更大的单元）会不会被旧池的单元误触：没有。40 个单盘全废的态全挂上，旧池单元的 fsid 不同，扫描时跳过（`system_configuration_repair.rs` 的 `write_order_and_birth_in_the_unit_area_of_the_device` 按 fsid 筛）。fsid 前 8 字节撞上的那一形没造（概率按 2⁻⁶⁴ 算，推的）。
- C670、Q3 两臂在丙' 之后会不会被调到：按代码次序推，两拒都在补写闸之前（V4）。
- 取样范围：两块盘、单元区 384 槽的小几何，历史种子 0，起点是第一个文件之后（mkfs 两条流从 mkfs 起）。三块盘、默认几何、别的种子都没取。
- 用户决定的动作：只放开了「关闭再可写挂载」这一步。回退、抬 F、建 inode 这几步没放开扫，所以「mkfs 之后摆不出两槽全废」在这些步上是推的（依据是轮换的写法：每盘一次一写、写较旧的那一槽，与操作种类无关）。

## 这条腿自己的限度

- 缩了的取样：覆盖写那条流（第一个文件之后一次覆盖写）有 1048588 个态，过了约 10⁶ 的单次上限，没跑；内容改成空长度还是 1048588（单元写那一段固定 20 个写，`out/overwrite.counts` 只记了状态数与「没跑」）。零单元发布在第一个文件之后一个写都不发（状态数 1），没用。最后用的是 `remount`（20519 态）。各流的态数：新盘 mkfs 4117 ×2 趟，用过的盘 mkfs 31195（有一趟因为原型把挂载参数的 fsid 给错了作废重跑，作废那一趟也是 31195），`remount` 20519，MUT-B1 下 20582 ×2 趟，补写流 35。原型跑过的全部态合计约 13 万，在 10⁷ 之内。挂钟：mkfs 那几条 13–66 秒，`remount` 那几条 73–125 秒（`out/` 里的原始输出带 `seconds=`）；预估 40 分钟以内，没有先缩取样。
- 第二、三截没跑：那两截的入口在 checker 档测试模块里、不是公开的，原型只在每个崩溃镜像上起一次可写挂载，再直接调 harness 的两个判官。所以「MUT-B1 在第二、三截零发现」与「甲改完之后第三截照旧记一条」都是推的：盘面、挂载结局、判官结论这一半是量过的。
- 判官两个调用里「只读读出在允许集合里」都直接交了 `InsideTheAllowedSet`（与红的那条测试同一个交法），没有逐态核。
- MUT-B1 是我按 harness 测试文件头的描述自己写的一份（盘 0 写完新槽，同一段清另一槽），不是当年攻方 `apply_mut_b.py` 那一份，两者是不是逐字节一样没核。
- 引文行号按主工作区现查，那几份代码与开工快照 sha256 相同。
- 草稿目录里建的编译目录 `target`（6.1G）、仓副本 `repo`（35M）与 11 个崩溃库目录（合计约 18M）交回前都删了；留下的只有构建日志、四份章节草稿与 `out/` 原始输出（几百 KB）。

## 没做什么

- 没判正推腿那几格，没读禁读清单里的文件，没替主 agent 采纳任何改法。
- 没跑 checker 档包的测试（红的那条测试在内），没跑门禁，没改 `crates/` 与 `crates/mutations.tsv`。
- 副本上的数不算入库装置上的数；要引得先在入库装置上重做。
