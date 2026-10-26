# m3-close-g4-rewrite-r2 辩方腿（Sonnet）报告（2026-10-22）

立场：辩方。复核第一轮判决 `research/prompts/m3-close-g4-rewrite-r1-main-verification.md` 的 V1–V4 与「只有 G4-丙 修得到」，替没被采的候选（照旧、收窄、判据带闸、乙、撤掉 G4 臂）找站得住的理由，答 W1–W4。
没起 checker 档与任何重型测试；没读副本、没读本轮攻方腿与判决（禁读清单）。只读主工作区的代码，开工先核了快照：

```
$ sha256sum -c research/prompts/m3-close-g4-rewrite-r2-snapshot/code-sha256.txt
crates/singlefs-core/src/mount.rs: OK
crates/singlefs-harness/src/mount_transparency.rs: OK
crates/singlefs-checker-tier/src/crash_injection.rs: OK
crates/singlefs-core/src/root_ring.rs: OK
crates/singlefs-checker/invariants/03-journal.md: OK
```

另核 kb 清单（`kb-sha256.txt`）十份全 OK。`transaction.rs`、`make_filesystem.rs`、`history.rs` 等不在快照清单里，`git status --short` 里没有它们的改动（只读，行号现查）。

## 复跑（抽象模型，纯 python，不碰仓）

模型目录 `research/prompts/m3-close-g4-rewrite-r2-sonnet-model/`。它只模拟一块盘的两个系统配置槽，读法抄自 `transaction.rs` 的 `write_system_configuration_slot`，**不调核心层**；有对照臂，证明它分辨得出坏协议。
```
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-close-g4-rewrite-r2-sonnet-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen <任一存在的目录> --work <新的草稿目录>
```
末行打 `name=rerun_compare file=explore.out sha256=… same=yes`（本腿跑过，same=yes）。

| 文件 | sha256 |
|---|---|
| slot_protocol_model.py | 24487daaa04ba5c62335c5ca4519ec18a8cbcfc66b584b9dabf3c8e8e9453a30 |
| explore.py | 1f1924e387e5eb448b65ffc1699427e7b7d63536d9e305ff026aac3687f7f3fb |
| tools/rerun.sh | 22f3bdefab796e42033a720a9728558ef1b7811d849a53e1cc11b585cf2ad636 |
| out/explore.out | 7b6b7514c4f204f9979d30c6717dc44fd0f1aea47664169b75e110899597a094 |

读模型输出的口径：`states_without_a_verified_slot=0` 是「所有崩溃状态上这块盘至少一个槽自证过」。`depth` 是连崩次数（崩、再挂、再崩…），`publishes` 是一次挂载后面跟几次发布末尾轮换。`outcomes=PNT` 是崩溃点重放的枚举域（落了／没落／撕裂），`PN` 是崩溃注入第二、三截的抽样域（见 W1 的第一条观察）。

## 一、一览表

判定的三个词：**站得住**（第一轮这一条我复核后仍成立）；**够不着**（引的证据够不到它说的全称）；**同样打中别的臂**（这条理由对它自己或对别的候选同样成立）。

| 判决条目 | 辩方判定 | 证据（细节在后面各节） |
|---|---|---|
| V1 mkfs 末段留得下两槽全废 | 站得住 | 「mkfs 返回之前的崩溃态不在 G4 的射程」（`.claude/kb/decisions/13-验证路线.md:187`）；同循环写、循环后才屏障：「device.write_at(offset, &slot, WriteDurability::Plain)?;」（`crates/singlefs-core/src/make_filesystem.rs:732`）；模型对照臂 `control_mkfs_last_segment_only_slot_zero_landed_depth1` 有 1 个无槽态 |
| V1 mkfs 之后：取号写两轮中途崩溃 | 站得住（代码加模型，推的；副本上的 20519 态不当已核） | 第二节表第 1 行 |
| V1 mkfs 之后：补写本身中途崩溃 | 够不着：域内补写不执行，协议侧站得住 | 第二节表第 2 行；仓里没有补写的崩溃枚举流 |
| V1 mkfs 之后：二次崩溃 | 站得住（模型连崩 3 次） | 第二节表第 3 行 |
| V1 mkfs 之后：清扫与覆盖写 | 站得住（推的） | 第二节表第 4、5 行 |
| V1 全称到读故障历史 | 够不着（模型链上有反例，域内摆不出） | 第二节「读故障」 |
| V2 空档（G4 只在拒的时候判） | 站得住；但「没人判」说过头，空档比 G4 一臂宽 | 第三节末、第七节 |
| V2「只有 G4-丙 修得到」 | **同样打中 G4-丙 自己**（五处） | 第四节 |
| 候选 照旧（只改注释） | 站得住（第一轮已采） | 第三节末 |
| 候选 收窄 | 站得住：域内与照旧无差，「退步」只在域外盘面 | 第四节 |
| 候选 判据带闸 | 站得住：代价大，且不补空档 | 第四节、第七节 |
| 候选 撤掉 G4 拒绝臂 | 站得住：撤不得 | 第三节末 |
| V3 甲 | 站得住；脆弱点有更稳的盘面可换 | 第六节 |
| V3 乙 | 判别力不比甲多（站得住），但覆盖了别处没有的一支 | 第六节 |
| V4 C670、Q3 不受影响 | 站得住（读阶段与重建先于补写闸，行号现查） | 第五节 |
| 打中二（撤掉补写闸的坏实现被判合法） | 同样打中 G4-丙：域内无盘面分辨它 | 第四节 |

## 一点五、W1–W4 一句话判定

- **W1（G4-丙 误报）**：留不下，没有哪一类历史留得下误报。最强的一层是第二、三截只摆整写子集（第二节观察一）；协议层与模型连崩 3 次相互印证；起战役没做，推的。补写本身这一类域内不执行，G4-丙 罩不到它，仓里也没有补写的崩溃枚举流（要记账）。
- **W2（接法）**：挂两处（第二截首次崩溃镜像、第三截每张二次崩溃镜像），第三截要摆在 `match once` 早退之前；不读补写闸；G4 拒绝臂要留（E171 与 harness 档共用它）；同一张盘面上只有「无槽且闸不过」一形记两条（G4-丙 加臂），C670、Q3 与它们互斥或各记各的事；变异行至少五条。第三截会把第一次崩溃镜像上已有的无槽盘重复记，建议只判新增的（G4-戊）。
- **W3（甲-二）**：不做。开关不是分支纪律要的那种，判官一侧比甲不多罩，唯一多出的「耐受补写闸改动」用换盘面（己）和一行源码替换变异（庚）买得到；采了 G4-丙 之后更不需要。
- **W4（条文）**：D23 第 396 行加一句交叉引用，判官判据不带闸；D13 第 187 行采 G4-丙 则删「与 C670 那条同接法」并重写「不同形」半句，不采则收窄并记账；C873 已被占用。

## 二、W1：G4-丙 的判据在 mkfs 之后每一类历史上留不留得下

结论：正确实现留不下，没有一类历史留得下误报。依据三层，由强到弱：①注入域本身只摆整写子集（下面观察一），正确协议的槽写落与不落都留着旧槽；②代码读出的写协议，加模型连崩 3 次的穷举；③仓里的守卫用例与变异行。**推的**：整个 W1 没在入库装置上跑过崩溃注入（不起重型），副本上的数（攻方第一轮）不当已核。

### 观察一：崩溃注入第二、三截的抽样域里没有撕裂态（第一轮没人看这一处）

- 文件头：「枚举域与崩溃点重放（`crate::crash`）同一个：更早的段整段持久、当前段任意真子集、更晚的段一个都没持久」（`crates/singlefs-checker-tier/src/crash_injection.rs:5`），同一行接着写撕裂并进没持久。
- 抽样只出整写子集：「pub persisted_within_the_segment: Vec<bool>,」（`crates/singlefs-checker-tier/src/crash_injection.rs:286`）；「let subset = draw_a_proper_subset(&mut source, segments[segment_index].len());」（`crates/singlefs-checker-tier/src/crash_injection.rs:4815`）。
- 撕裂扩展住在崩溃点重放那一侧：「pub struct TearableInPlaceOverwrites {」（`crates/singlefs-checker-tier/src/crash.rs:2184`）；现跑：
```
$ grep -c 'Tearable' crates/singlefs-checker-tier/src/crash_injection.rs
0
```
- 后果一：注入域里一块盘要无槽，得有一个**落了的破坏性写**（清零、写错位置），撕裂造不出。模型 `outcomes=PN`：正确协议连崩 3 次无槽态 0（`injection_domain_post_mkfs_depth3 states=75 states_without_a_verified_slot=0`）；攻方的 MUT-B1 形状（同段把另一槽清零）仍出无槽态（`injection_domain_control_mut_b1_depth1 states=10 states_without_a_verified_slot=2`，反例 `seg1:NP->(None, None)`：第一轮的清零已落、第二轮新槽没落而清零落了）。
- 后果二：「少一道屏障、两个槽写并一段」那一类坏协议，在注入域里造不出无槽盘（`injection_domain_control_no_barrier_between_the_two_acquisition_rounds_depth3 … states_without_a_verified_slot=0`），G4-丙 根本看不见它；这一类归守卫用例与变异行（下面第 5 行）。
- 后果三：第一轮攻方量的 42、40、6 个态出自它自己的原型：「崩溃态用 `singlefs_checker_tier::crash_state_enumeration::enumerate_crash_states_with_expansion` 加 `crash::full_expansion` 取」（`research/prompts/m3-close-g4-rewrite-r1-opus-output.md:7`），是枚举域（带撕裂）。数不能搬到第二、三截；定性结论（MUT-B1 形状摆得出无槽盘）模型在抽样域里也复现。

### 逐类

| 类 | 历史与崩溃点 | 代码里的步 | 无槽盘留得下吗 | 证据性质 |
|---|---|---|---|---|
| 取号写两轮中途崩溃 | 重开可写挂载，崩在第一轮或第二轮内、或两轮之间 | 世代号取盘上自证过的最大者加一：「let slot_generation = verified_on_this_device」（`crates/singlefs-core/src/transaction.rs:592`）；落槽：「(slot_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing,」（`crates/singlefs-core/src/transaction.rs:643`）；两轮：「for _round in ACQUISITION_WRITE_ROUNDS {」（`crates/singlefs-core/src/transaction.rs:1235`），第一轮前有屏障、每轮后有屏障：「if let Err(cause) = pool.perform_commit_step(CommitStep::Barrier) {」（`crates/singlefs-core/src/transaction.rs:1178`、`:1254`） | 留不下。mkfs 之后两槽都是世代号 1：世代号 2 落槽 0、世代号 3 落槽 1，每轮覆写的都是另一槽 | 推的（读码加模型：`post_mkfs_publishes0..3_depth1..3` 十二行全 0）；仓里取号写到一半的全枚举流重开可写挂载并在被拒时 panic（位置见表后第一条），补写例外之前这句就是「没有无槽盘」的隐式判官 |
| 补写本身中途崩溃 | 要先有一块两槽全废的盘 | 只有两槽全废、读得出、闸过的盘才补写：「if devices_rewritten_by_the_acquisition.contains(&slots_of_this_device.device) {」（`crates/singlefs-core/src/transaction.rs:920`） | 域内补写**不执行**（域内没有这样的盘），所以没有「补写中途崩溃」这一类历史；脱离域，模型 `rewrite_of_a_device_without_slots_depth1/depth2`：无槽态只在第 0 段（等于起点那一块本来就全废的盘），补写没新造出一块 | 协议侧推的；**仓里没有补写的崩溃枚举流**（`ls crates/singlefs-checker-tier/tests/crash_enumeration_*` 无一条点名补写），补写只有非崩溃的验收用例：「let mounted = outcome.expect("盘 1 过了补写闸：可写挂载做成");」（`crates/singlefs-harness/tests/acquisition_rewrites_a_device_that_lost_both_system_configuration_slots_only_through_the_gates.rs:164`）。G4-丙 罩不到这一类 |
| 二次崩溃（含再崩） | 第一次崩溃镜像加挂载写的一个子集 | 「second_base = crash_image_materialized.clone();」（`crates/singlefs-checker-tier/src/crash_injection.rs:4242`）；每个挂载阶段摆一个：「if writes_of_the_phase.is_empty() {」（`crates/singlefs-checker-tier/src/crash_injection.rs:4105`），阶段六个：「pub const ALL: [WritableMountPhase; 6] = [」（`crates/singlefs-checker-tier/src/writable_mount_phases.rs:38`） | 留不下。第一次崩溃镜像每块盘都有槽，挂载写的每个槽写覆写另一槽，与上一行同一规则 | 模型连崩 3 次穷举（枚举域带撕裂，`post_mkfs_publishes3_depth3 states=363 … =0`） |
| 清扫 | 挂载末尾的抹头加清扫发布 | 抹头只在单元区：「let mut slot = unit_area_start.slot().0;」（`crates/singlefs-core/src/instance_table_sweep.rs:409`），抹之前重读认得出单元头：「unit_head_read_by_sweep(&reread_bytes).is_some_and(」（`crates/singlefs-core/src/instance_table_sweep.rs:575`）；清扫发布末尾是普通轮换 | 留不下：写的偏移落不到系统配置区。划段按偏移：「if offset < system_configuration_end {」（`crates/singlefs-harness/src/segments.rs:80`） | 推的（读码）；没量 |
| 覆盖写、发布末尾轮换、抬 F、取号回卷 | 一次发布的最后三步，抬 F 先写系统配置，取号失败的回卷 | 「发布的最后三步（D16（发布语义） 已定项 7 的持久顺序）：根槽 FUA 写 → 系统配置槽轮换 → 屏障，屏障完成才返回。」（`crates/singlefs-core/src/transaction.rs:1584`）；抬 F 与回卷各在第一个槽写前一道屏障：「if let Err(barrier_error) = self.perform_commit_step(CommitStep::Barrier) {」（`crates/singlefs-core/src/transaction.rs:693`） | 留不下 | 守卫用例判录制流：「判据按录制流量：mkfs 之后切段（屏障切段），没有一段里同一块盘上有两次系统配置槽写。」（`crates/singlefs-harness/tests/system_configuration_slot_is_overwritten_only_after_a_barrier.rs:7`）；变异行 `crates/mutations.tsv` 第 796、797、798 行，例：「实审A1 第19条：取号回卷写之前不过屏障（取号写与回卷写同段）」（`crates/mutations.tsv:798`）。覆盖写那条枚举流态数过上限，攻方没跑，这一格只剩读码与守卫 |
| 起点 | 两种起点 | 「只做过 mkfs、进程已退出：没有可写会话，要先挂载。」（`crates/singlefs-harness/src/history.rs:315`）；基线断言：「崩溃状态摆在整段落在 mkfs 之后的段上：mkfs 的写都已落」（`crates/singlefs-checker-tier/src/crash_injection.rs:2085`） | mkfs 末段摆不进（V1 第一行） | 断言在代码里，现读 |

表后补两条（含竖线的引文不放进表格）：
- 取号写到一半的全枚举流观察者在被拒时 panic：「.unwrap_or_else(|error| panic!("取号写写到一半崩溃之后重开可写挂载被拒：{error:?}"));」（`crates/singlefs-checker-tier/tests/crash_enumeration_crash_in_the_middle_of_the_acquisition_write_stream.rs:196`）。同一档的「只做过 mkfs 的池重开」那条流同样在被拒时 panic：「panic!("C492：崩溃之后重开可写挂载被拒（树表 0 条的一版上没有可写的出路）：{error:?}")」（`crates/singlefs-checker-tier/tests/crash_enumeration_formatted_pool_remounted_without_a_file_stream.rs:164`）。
- 这两条流只断言「至少一块盘有可择的系统配置」：「.expect("取号写只覆写较旧那一槽：每块盘较新那一槽一直读得出，至少一块盘有可择的系统配置");」（`crates/singlefs-checker-tier/tests/crash_enumeration_crash_in_the_middle_of_the_acquisition_write_stream.rs:168`），不数每块盘。

### 读故障

全称到读故障历史够不着。历史里有一种操作在读故障下真的往盘上写：「崩溃注入里唯一一种在读故障下真的往盘上发写的步」（`crates/singlefs-harness/src/history.rs:544`）。模型：这一步排在崩溃之前（历史的一步，崩溃态之后的挂载都不带故障）时无槽态 0（`post_mkfs_fault_in_the_first_run_only_depth1..3`）；若一次撕裂的崩溃**之后**再来一次读故障下的挂载，读故障把唯一自证的槽藏掉，写落在它上面，撕裂就得无槽盘（`stress_fault_in_every_run_depth3 … states_without_a_verified_slot=6`，反例 `seg0:T->(None, 1) | seg0:T->(None, None)`）。这个序列现在摆不出来：第二、三截的挂载起在崩溃镜像上、不带故障层：「盘是不带故障层的内存盘：读到的就是镜像上的字节」（`crates/singlefs-checker-tier/src/crash_injection.rs:3455`）。所以 G4-丙 跟 C670、Q3 一样只判没注入读故障的崩溃态，是对的，不是保守：D13 第 187 行写的正是「无读故障的崩溃态」。哪天历史里出现「撕裂之后再来读故障下的挂载」这类步，这一条要先重判。

**什么现象会推翻 W1 的结论**：①第二、三截的任一崩溃态上，没注入读故障、某块盘两槽都自证不过，而造出它的历史是正确实现（唯一的现查办法是起 checker 档，我没起）；②守卫用例 `system_configuration_slot_is_overwritten_only_after_a_barrier` 在入库装置上红；③抽样域加进撕裂扩展（`crash_injection.rs` 引进 `TearableInPlaceOverwrites`）之后，正确协议上出现无槽态，此时要按模型的 `post_mkfs_*` 行重判（枚举域里模型给的是 0）。

## 三、W2：G4-丙 的接法

### 挂在哪几处、每张盘面判几次

- 第二截：第一次崩溃镜像 `crash_image_materialized`，每个崩溃态一次。现成的位置是挂载那一趟之前：可写挂载那一趟在 `crash_injection.rs:1797` 起，`read_faults` 已在手：「read_faults: read_faults_in_the_crash_state,」（`crates/singlefs-checker-tier/src/crash_injection.rs:1811`）。
- 第三截：每张二次崩溃镜像一次。不要放进「没崩过的那一次挂载做成」才走的路径：`judge_the_remount_after_a_second_crash` 一进门就在没崩过的那一次被拒时返回：「let once_content = match once {」（`crates/singlefs-checker-tier/src/crash_injection.rs:4416`）。G4 拒绝臂在这条早退之后，G4-丙 要摆在它前面，否则继承同一个盲区。
- 放在 `judge_the_remount_after_a_second_crash` 里（早退之前）还是放在调它的循环里，后果不同：放进函数里，8968 起那条红的测试（直接调这个函数）会多一条 G4-丙，它末尾的断言要改：「"第三截记一条：已知红 {:?}，新发现 {:?}",」（`crates/singlefs-checker-tier/src/crash_injection.rs:9093`）从 1 改 2；放在循环里测试不动，但「第三截调用点缺」那一行变异要用重型的 `SecondCrashInputs` 搭才杀得到（见变异行）。
- 要不要读补写闸：不要。判据是盘面上的事实（某块盘无自证槽），与核心层放不放过它无关；读闸要在判官里把五件另写一遍（闸一要所选版本的单元清单），域内零收益。这一问站得住，同意第一轮。
- 第三截里的重复：二次崩溃镜像从第一次崩溃镜像起算：「second_base = crash_image_materialized.clone();」（`crates/singlefs-checker-tier/src/crash_injection.rs:4242`）。第一次崩溃镜像 X 若已经有无槽盘且闸过，挂载补写它，取号阶段那一张二次崩溃镜像上这块盘可能仍然无槽（每阶段摆一个：`crash_injection.rs:4112`）——同一个根因在第二截记一条、第三截再记最多一条。X 无槽而闸不过时拒在写之前，挂载没有写，各阶段都摆不出二次崩溃镜像（`crash_injection.rs:4105`），第三截零条。要不重复，第三截只判「X 没有而这张镜像上有」的无槽盘（新名 G4-戊；零轮）。

### 同一张盘面上各记几条

「记」指 `FindingsOfOneHistory::settle` 收一次（`tally.crash_states_ending_new_finding` 加一；同一截同签名的新发现只留一个，`crash_injection.rs:2655`）。G4 拒绝臂在第二截的一条走 `writable_mount_refusals_on_the_d23_list_illegal`（`crash_injection.rs:3526`），在第三截走「判据为真」那一支（`crash_injection.rs:4743`）。下表全部是读码推的。

| 盘面（没注入读故障） | 可写挂载结局 | G4-丙 | G4 拒绝臂 | C670 | Q3 |
|---|---|---|---|---|---|
| a 盘 0 无槽、补写闸过 | 挂上，补写 | 第二截 1；第三截至多 1（取号阶段那张继承的） | 0（没拒） | 0 | 0 |
| b 盘 0 无槽、闸不过（甲那一形） | 拒，拒因是读阶段判完之后系统配置又读不出，拒在写之前 | 1 | 第二截 1、第三截 1 | 0 | 0 |
| c 盘 0 无槽，同时见证判据为真 | 拒，拒因是判据 N-配置（读阶段，先于逐盘核） | 1 | 0（拒因不同） | 1 | 0 |
| d 盘 0 无槽，同时根环按最新根的实例表一条有效根都没有 | 拒，拒因是重建分配器那一拒（先于 `establish_instance`） | 1 | 0 | 0 | 1 |
| e 每块盘都无槽 | 恢复整池拒，不在单子上 | 1 | 0（第二截按通用「拒了」记 1，不是 G4 的） | 0 | 0 |
| f 盘面全好而核心层误拒（拒因读阶段之后系统配置读不出） | 拒 | 0 | 第二截 1（判据复算为假），第三截 0（只红判据为真的） | 0 | 0 |
| g 注入过读故障 | 任意 | 0（跳过） | 0 | 0 | 0 |

- 拒因互斥（一次挂载只交一个错），所以 b、c、d 里 G4 拒绝臂、C670、Q3 至多一条；G4-丙 独立于拒因，在 b、c、d、e 上各多一条。c、d 里是两件各自成立的事（见证超前或根环全废，加盘 0 无槽），记两条不算重复。
- 顺序现查：读阶段「read_stage_settled_at_most_on_the_one_reread(」在 `crates/singlefs-core/src/mount.rs:8352`，重建分配器「let rebuilt = rebuilt_allocator(」在 `crates/singlefs-core/src/mount.rs:8458`，两者都在「establish_instance(」之前（`crates/singlefs-core/src/mount.rs:8490`）；补写闸在 `establish_instance` 里：「devices_without_the_selected_version(」（`crates/singlefs-core/src/mount.rs:6885`），函数起于「fn establish_instance<Device: BlockDevice>(」（`crates/singlefs-core/src/mount.rs:6674`）。
- **只有 b 是一件事记两条**：G4 拒绝臂自己的说明写的就是同一个事实：「正确实现在它上面永远拒可写；与 C670 那一条同形，拒是对的、造出这份盘面的那一段不对」（`crates/singlefs-harness/src/mount_transparency.rs:2947`）。b 在 G4-丙 之后，臂的红只是 G4-丙 的红的回声。两者的签名不同（臂是 `ModelDisagreement`，G4-丙 若记成违例名就是 `CheckerViolations`，`FailureSignature::of` 在 `history.rs:3093`），会并存成两个新发现；不并存的办法是让 G4-丙 在这张镜像上的拒绝已被臂记过时不再记（第二截可做，第三截的臂有早退，做不到）。我的建议是不合并，并把红测试的断言改成两条（见第八节）。
- **漏一条**的盘面：①注入过读故障的态（按设计）；②二次崩溃镜像每阶段只摆一个，没摆到的阶段上的无槽盘（抽样）；③撕裂造出的无槽盘（观察一，抽样域摆不出）；④补写本身（域内不执行）。

### 采了 G4-丙，G4 拒绝臂还要不要留

**要留。** 留着在崩溃注入里是多余的（b 的回声），撤掉在别处丢判别力：
- 臂的判据是判官 API 的一部分，不止崩溃注入一家用。E171 的装置按枚举成员认它：「SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults =>」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3529`）；撤掉枚举成员 E171 的 `match` 编不过。
- harness 档两条直接判臂的用例、变异行 2406、2407 的杀者都在这个 API 上：「第29批 G4 判官整条不判：没注入读故障的崩溃态上…那一拒的判据复算为真不报（MUT-B1 最小形状）」（`crates/mutations.tsv:2406`）。撤臂等于重写这三条变异与它们的杀者。
- 臂判的是拒绝本身（拒是不是该拒），G4-丙 判的是盘面；补写例外之后这是两件事：盘面无槽而闸过，拒不对；盘面无槽而闸不过，拒对、盘面不该有。把后者的红并进盘面一边，前者留在 U6。

## 四、「只有 G4-丙 修得到」是不是同样打中 G4-丙 自己，与没被采的候选

第一轮的理由原句：「照旧、收窄、收窄加 U6 在这 42 格上都还中，只有 G4-丙 修得到」（`research/prompts/m3-close-g4-rewrite-r1-opus-output.md:106`）。三个候选都要「先有一次拒才进得来」，这一点站得住。**打中 G4-丙 自己的有五处**：

1. **「修得到」是原则上看得见，不是抽得到。**
   - 那 42 格出自枚举域（带撕裂，第二节观察一）；注入域是抽样：第二截每段历史摆 `crash_points_per_history` 个态（调用方给，例：「crash_points_per_history: 16,」`crates/singlefs-checker-tier/src/crash_injection.rs:8118`，战役测试里取 2 和 4），第三截每个第一次崩溃态每阶段摆一个（`crash_injection.rs:4112`）。
   - MUT-B1 的无槽态占 `remount` 流 42/20582（攻方副本的数，枚举域，不当已核）。G4-丙 在注入域能不能抽到，只有起战役才知道，我没起。
   - 对同一个坏实现，仓里已有一个**确定性**杀者：攻方原型的清零写是普通写：「&vec![0u8; slot_bytes.len()],」（`research/prompts/m3-close-g4-rewrite-r1-opus-model/toggles.patch:12`），按偏移归系统配置槽（「if offset < system_configuration_end {」，`crates/singlefs-harness/src/segments.rs:80`），于是同段同盘第二次槽写，守卫用例（第二节表第 5 行）会红。**推的，没跑**；若清零写改走 `write_zeroes_at`，划段把它归清零写：「RecordedOperationKind::WriteZeroes => StepKind::ZeroFill,」（`crates/singlefs-harness/src/segments.rs:71`），守卫看不见，G4-丙 才是唯一的网。所以「只有」只对这一个子形（清零写走 `write_zeroes_at`）成立。
2. **判官读槽的算法与核心层共用。** 复用的谓词读槽走「if let Ok(system_configuration) = SystemConfiguration::parse_slot(&bytes) {」（`crates/singlefs-harness/src/mount_transparency.rs:2374`），`parse_slot` 是核心层的（`crates/singlefs-core/src/system_configuration.rs:646`）；取 fsid 与槽距又经「let Ok(system_configuration) = choose_system_configuration(crash_image) else {」（`crates/singlefs-harness/src/mount_transparency.rs:2422`）。核心层的槽校验放宽（把撕裂槽当自证）时，判官与实现一起看不见。规则是「运行时与 checker 必须用不同的算法。」（`.claude/rules/fs-design.md:35`）。池级 checker（O2）有独立实现的现成报告：「pub fn system_configuration_redundancy_report(」（`crates/singlefs-checker/src/image.rs:1492`），交 `devices_without_a_self_verified_slot`，只依赖格式常量 crate。差别：它不分 fsid（「本池自证不过」那一半（fsid 是别的池的槽）这里不分，`crates/singlefs-checker/src/image.rs:1490`），域内无别的池，不碍事。新名 **G4-丁**：G4-丙 的读槽改调这份报告（零轮）。代价（推的）：它是池级 checker 的调用点，`.claude/rules/verification.md:132` 要求落在判定段里或带 `// gpu-exempt:` 标记，要加一行豁免表。
3. **跳过注入过读故障的态。** 同 C670、Q3，按设计（第二节「读故障」）；于是 `NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS` 那组比重的历史里，崩溃窗口之后的态 G4-丙 一律不看。
4. **空档比 G4 一臂宽，G4-丙 只补崩溃注入一家。** 补写例外之前，「挂不上就 panic」是全枚举流的隐式判官（第二节表后两条；第三条在 floor_raise 那条流里：「Err(error) => panic!("挂载失败：{error:?}"),」（`crates/singlefs-checker-tier/tests/crash_enumeration_floor_raise_pushed_by_admission.rs:237`））；例外之后这些观察者在无槽盘上挂得上、照样绿。D13 第 187 行只写了崩溃注入：「这一性质由 G4 判（崩溃注入第二、三截，与 C670…那条同接法）」（`.claude/kb/decisions/13-验证路线.md:187`）。要罩全部，判断得落在枚举流共用的判段上（调 O2 报告，G4-丁 的另一种落点），不是崩溃注入的两个调用点。**这一处没量**：`grep -ln 'ImageAfterTheObserver' crash_enumeration_*.rs` 命中三条（floor_raise_pushed_by_admission、formatted_pool_remounted_without_a_file_stream、crash_in_the_middle_of_the_acquisition_write_stream），三条都在被拒时 panic（上面三处引文）；别的枚举流有没有同样的隐式判官，没逐条读。
5. **「撤掉补写闸」的坏实现 G4-丙 也分辨不出。** 打中二说「只收窄不动 U6」会把退回丙' 之前的实现判合法。域内（没有无槽盘）这样的坏实现与正确实现逐态相同，没有任何判官分辨得出；脱离域，G4-丙 在无槽盘上无论核心层拒不拒都红，也不分辨「拒对不对」。抓它的是补写验收用例：「let mounted = outcome.expect("盘 1 过了补写闸：可写挂载做成");」（`crates/singlefs-harness/tests/acquisition_rewrites_a_device_that_lost_both_system_configuration_slots_only_through_the_gates.rs:164`），变异行「丙' 取号那一刻的见证读不认过了补写闸的盘（照旧拒）」点名它（`crates/mutations.tsv:5010`）。打中二因此不能当收窄与照旧之间的分辨臂。

### 候选逐个

- **照旧（只改注释）**：站得住。第一轮已采并落了。采 G4-丙 之后仍要它，臂不撤（第三节）。
- **收窄（闸也不过才判不合法）**：域内与照旧逐态相同（没有无槽盘），差别只在手造盘面；攻方说它「撤掉丙' 判合法」，见上第 5 条，那是 U6 不带闸的事，收窄与照旧在这一格上判得一样的错。第一轮说它是退步：「这是退步：这类拒是实现错（该补写而没补），改后放过」（`research/prompts/m3-close-g4-rewrite-r1-sonnet-output.md:103`），这只对手造盘面成立。
- **判据带闸**：站得住的理由是代价与收益：判官要把补写闸五件另写一遍（闸一要所选版本的单元清单，核心层靠重建那一版得出），域内零收益，也不补空档。同意不采。
- **乙（断言再挂做成）**：判别力不比甲多（第一轮结论站得住）。但乙钉住的是别处没有的一支：第三截的「挂成」支在无槽盘上没有 checker 档用例（补写只有非崩溃的验收用例，第二节表第 2 行）。采 G4-丙 之后，这条用例是 G4-丙 要判红的盘面（无槽而挂上），它只能作直接调 `judge_the_remount_after_a_second_crash` 的单元用例（不经循环）存在，并且不能再断言「零发现」。
- **撤掉 G4 臂**：站得住的是「不撤」，理由见第三节末。

## 五、V4 复核

C670、Q3 的拒因出在读阶段与重建分配器，早于补写闸，行号见第三节。站得住。再补一句第一轮没写的：**这两臂与 G4 同形**（只在拒的时候判），空档的形状相同；它们没有出现 G4 那样的空档，只因为这两拒没有「某种盘面放行而挂上」的例外。哪天给它们开例外，同一个空档就开了，写进第八节第 9 条。

## 六、W3：甲-二 的开关与它多罩的东西

### 开关放哪、是不是分支纪律要的那种

- 攻方原型把开关放在补写闸函数的第一句，读环境变量：「if std::env::var_os("SINGLEFS_PROTO_GATE_NEVER_PASSES").is_some() {」（`research/prompts/m3-close-g4-rewrite-r1-opus-model/toggles.patch:26`）。这是草稿副本里的写法，不能入库：产品的可写挂载路径读进程环境，等于多一条发布路径，运行时也报不出走了哪一臂，违反「分支必须可观测」（`.claude/rules/fs-design.md:127`）。
- 入库只有两种形态。①仓里已有的先例：参数枚举。`SpaceAdmission`：「只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：发布与可写挂载判不判空间准入。」（`crates/singlefs-core/src/admission.rs:1046`），有 `ALL` 与 `branch_name`，经「pub fn mount_writable_with_space_admission」（`crates/singlefs-core/src/mount.rs:7771`）穿进可写挂载；`mount.rs` 里提到它的行 63 处（`grep -c 'SpaceAdmission\|space_admission' crates/singlefs-core/src/mount.rs` 现跑 63）。照这个先例开一个补写闸的参数，要穿过 `mount_writable` 家族的公开入口一路到「devices_without_the_selected_version(」（`crates/singlefs-core/src/mount.rs:6885`）。②不开运行时开关，把「补写闸恒不过」写成 `crates/mutations.tsv` 一行源码替换（变异表的机制就是替换原文再跑点名的测试，不需要产品里留开关）。
- **它不是五条硬要求第 2 条要的那种。** 那一条是：「每条分支必须能被测试强制进入。」（`.claude/rules/fs-design.md:122`）。补写闸的两臂（补写、拒）都由盘面数据进得去：补写臂见 `…only_through_the_gates.rs:164`，拒绝臂见「fn a_blank_device_is_not_rewritten_and_the_writable_mount_is_refused_before_any_write() {」（`crates/singlefs-harness/tests/acquisition_rewrites_a_device_that_lost_both_system_configuration_slots_only_through_the_gates.rs:202`）与别池的盘「fn a_device_of_another_pool_is_not_rewritten_and_the_writable_mount_is_refused_before_any_write() {」（同文件 `:214`）。`SpaceAdmission` 的理由是数据进不去那一臂：「在健康的历史里走不到——小盘上式子先拒。关掉准入，那一条拒绝路径」（`crates/singlefs-core/src/admission.rs:1048`）。甲-二 的开关强迫的是**组合**（拒绝臂 × 产品本来会补写的盘面），那是故障注入，不是进入分支。

### 甲-二 比甲多罩什么

判官收到的输入不含闸：「pub fn judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(」（`crates/singlefs-harness/src/mount_transparency.rs:3031`）的参数是崩溃镜像、所选版本、拒因、盘面变没变、只读读出、读故障两个标记。同样的无槽镜像、同样的拒因，在闸不过的盘面（甲）和闸过而被强迫拒（甲-二）上判官的输入逐项相同，所以**判官这一侧甲-二 一格不多罩**。

| 项 | 甲 | 甲-二 | 己加庚（新，零轮） |
|---|---|---|---|
| 判官 G4 臂判红 | 罩 | 罩（同输入） | 罩 |
| 对补写闸改动的耐受 | 脆：盘面押在闸一（单元坏一字节）上，闸一放宽测试又红（第一轮攻方打中四） | 耐：强迫拒与闸无关 | 换成闸一不过且有自己变异行钉着的盘面：空盘（变异行「丙' 补写闸一不看本盘带不带所选那一版的单元」点名，`crates/mutations.tsv:5011`）、别池的盘；放宽闸一时先红的是钉它的用例，原因清楚 |
| 核心层「闸恒不过」的坏实现 | 不罩 | 罩，杀者是强迫拒的新用例 | 罩，杀者是现成的补写验收用例（`…only_through_the_gates.rs:164`，同类变异行 `crates/mutations.tsv:5010`）；新增一行源码替换变异即可，推的 |
| 产品代码多出的东西 | 无 | 一个参数枚举穿过挂载入口 | 无 |
| 采了 G4-丙 之后 | 罩 | 「G4 报」那句断言在这张盘面上 G4-丙 已独立红，剩下只有对闸耐受一格 | 同甲 |

结论：甲-二 比甲多罩的只有「对补写闸改动的耐受」一格，用盘面（己）可以买到，不必开关；「闸恒不过」的坏实现用变异行（庚）罩，现成的验收用例就是杀者。采了 G4-丙 之后甲-二 不要。**推的**：没造己的盘面。己的前提要查：检查档那条红的测试里换成空盘之后，只读恢复仍落到第一个文件那一版、`once` 仍有内容（空盘会拿掉根环里它那一半的根）；补写验收用例里的拒绝用例把丢槽的盘放在盘 1、所选版本的根放盘 0（`VERSION_WITH_ITS_ROOT_ON_DEVICE_ZERO`），检查档那条的盘 0 要换成盘 1。

**什么现象会推翻**：空盘换进检查档那条测试后拒因不再是「读阶段判完之后系统配置又读不出」，或只读恢复落不到那一版，己就不成立，退回甲加「盘面同时让闸一和闸四不过」（攻方第一轮的建议，零轮）。

## 七、W4：条文

### D23 已定项 14 的拒因单子与判官判据要不要带补写例外

- 现状三处并排：D22 写了例外：「某块盘两槽全废（一份本池自证过的系统配置都没有）时，取号那一步补写它的两槽，要这几样都成立，有一样不成立就拒」（`.claude/kb/decisions/22-单元原子性怎么合成.md:181`），变更记录：「某块盘一份自证过的系统配置都没有时取号那一步也拒，用户 2026-10-03 定、2026-10-19 开补写例外」（`.claude/kb/decisions/22-单元原子性怎么合成.md:190`）；D23 没带：「这一步在某块盘一份自证过的系统配置都没有时也拒（用户 2026-10-03 定留着）」（`.claude/kb/decisions/23-journal的角色与格式.md:396`）。
- **要在 D23 那一句后面带一句交叉引用**（补写闸五件都过的盘例外，权威写法在 D22 已定项 8）。理由：两处现在对同一盘面说相反的话，读者只读 D23 会以为过闸的盘也拒；判官里那个枚举成员的文档注释写「正确实现在它上面永远拒可写」（`crates/singlefs-harness/src/mount_transparency.rs:2947`），也是同一个读法。**不要把五件抄进 D23**：五件的权威写法只留在 D22，D23 指过去，免得两份各自漂（引 kb 条款要整条抄，摘句不许）。
- **判官判据（`system_configuration_unreadable_after_the_read_stage_on_the_crash_image`，`crates/singlefs-harness/src/mount_transparency.rs:2419`）不带闸。** 它复算的是「拒因所述的事实成立不成立」（某块盘无槽），不是「该不该拒」；该不该拒是闸的事，闸在核心层。带闸要在判官里独立重写五件（第四节），域内零收益。判据旁边的注释要加一句：这一拒的判据为真不等于正确实现会拒，闸过的盘不拒；不改代码。

### D13 已定项 6 第 187 行那一句

- 采 G4-丙：「这一性质由 G4 判（崩溃注入第二、三截，与 C670…那条同接法）」（`.claude/kb/decisions/13-验证路线.md:187`）里「与 C670 那条同接法」要删，G4 不再挂在拒绝上。建议写成：「这一性质由 G4 判（崩溃注入第二、三截，按盘面判：没注入读故障的崩溃镜像上某块盘一份本池自证过的系统配置都没有就报，不看可写挂载拒没拒，补写例外放过的盘面挂上了也判；C670、Q3 仍挂在拒绝上）」。同一行后半句要重写：「G4 与那条还有一处不同形：择不到系统配置时那条的复算交 `None`、G4 按真，这时读阶段本身已拒、到不了这一格」（`.claude/kb/decisions/13-验证路线.md:187`）——按盘面判的 G4 到得了这一格（每块盘都无槽按真），句子要改或删。「mkfs 返回之前的崩溃态不在 G4 的射程」那一半照旧，另加「抽样域摆整写子集（撕裂并进没持久），无槽盘只会由落了的破坏性写造出」（第二节观察一）。
- 不采：这一句要收窄成「G4 只在可写挂载被拒时判；补写例外放过的盘面挂上了不在 G4 的射程」，空档记账。账记 `.claude/kb/checks-owed.md`，新 C 号：**C873 已被占用**（「隔离记录两盘不对称没人判」，`.claude/kb/checks-owed.md:578`），C872 在 `records/2026-10-13-里程碑四预开发.md:848` 一行里被里程碑五会话的通知提到过；先 `grep -n 'C87[2-9]' .claude/kb/checks-owed.md` 现查再取号，不照背景材料的「从 C873 起」。
- 不论采不采，另有三笔账可记（零轮）：①补写本身没有崩溃枚举流（第二节表第 2 行）；②补写例外之后全枚举流观察者「挂上即绿」不再保证没有无槽盘（第四节第 4 条）；③注入域不摆撕裂，系统配置槽撕裂只在枚举域摆（第二节观察一）。

## 八、改法与条文建议

「被攻过几轮」：第一轮由两条腿提出的算「第一轮提出，被攻 0 轮」；本轮攻方腿的结果和判决我按禁读清单没看，不计。我这一腿是复核，不是攻，我自己提的新名全是零轮。

| # | 建议 | 被攻过几轮 | 依据 |
|---|---|---|---|
| 1 | 采 G4-丙：盘面级；不读补写闸；只判没注入读故障的态；第二截摆在挂载那一趟之前，第三截摆在 `match once` 之前（不继承早退盲区） | 第一轮提出，0 轮 | 第二、三节；W1 误报面我没找到反例（读码加模型；没起战役） |
| 2 | G4-丁：G4-丙 的读槽改调池级 checker 的 `system_configuration_redundancy_report`（只取 `devices_without_a_self_verified_slot`）；不采就在 G4-丙 的注释里写明它与核心层共用 `parse_slot` | 0 轮（新） | 第四节第 2 条；`.claude/rules/fs-design.md:35` |
| 3 | G4-戊：第三截只判「第一次崩溃镜像上没有、这张二次崩溃镜像上有」的无槽盘，免得同一根因重复记 | 0 轮（新） | 第三节「第三截里的重复」 |
| 4 | G4 拒绝臂照旧不动（枚举成员、变异行 2406–2408、E171 的 `match` 都留） | 第一轮判决的「照旧」，0 轮 | 第三节末 |
| 5 | 红测试：甲，盘面换成「闸一不过且有自己变异行钉着」的（空盘、别池的盘），断言按 G4-丙 的摆法改（放函数里则 1 改 2） | 甲：第一轮两腿一致，0 轮；换盘面（己）0 轮（新） | 第六节；己的前提没验 |
| 6 | 不做甲-二 的开关；「补写闸恒不过」写成 `crates/mutations.tsv` 一行源码替换（庚），杀者是现成的补写验收用例 | 甲-二：第一轮提出，0 轮；庚 0 轮（新） | 第六节 |
| 7 | G4-丙 的变异行至少五条，全是 0 轮（推的）：①判定恒假；②「某块盘」改「每块盘」（`.any` 改 `.all`，全废镜像上两者等价，要混合镜像才杀得到）；③不看读故障标记；④第二截调用点缺；⑤第三截调用点缺或戊的「相对第一次崩溃镜像」被删；⑥摆在早退之后。杀者：①② 一条 harness 单测搭混合镜像（盘 0 无槽、盘 1 好）；③ 一条注入过读故障的无槽镜像；④ 检查档库单测直接调第二截函数（要手造 `WritableMountOnTheCrashImage`，字段全公开）；⑤ 8968 那条改过的断言；⑥ 一条 `once` 为被拒、镜像无槽的新用例 | 0 轮 | 第三节 |
| 8 | D23 第 396 行那句后面加交叉引用（补写闸五件都过的盘例外，权威写法在 D22 已定项 8），不抄五件；判官判据不带闸，只在 2419 起那个函数旁加一句注释 | 0 轮 | 第七节 |
| 9 | D13 第 187 行：采 G4-丙 则删「与 C670 那条同接法」、重写「不同形」那半句；不采则收窄成「只在被拒时判」并记账。C670、Q3 与 G4 同形（只在拒的时候判），备注里写「给这两拒开例外时必须同时加盘面级判」 | 0 轮 | 第五、七节 |
| 10 | 记账（`.claude/kb/checks-owed.md`，C873 已占用，取号先现查）：补写本身没有崩溃枚举流；补写例外之后全枚举流观察者「挂上即绿」不再保证没有无槽盘；注入域不摆撕裂。不采 G4-丙 时再加空档一行 | 0 轮 | 第二节、第四节第 4 条 |

## 要主 agent 核的（我没做）

- 模型复跑：`python3 -I research/prompts/m3-close-g4-rewrite-r2-sonnet-model/explore.py | sha256sum` 应等于上面 `out/explore.out` 的 sha256。模型是我对 `write_system_configuration_slot` 的读法，不是核心层本身；模型红绿不替代入库装置的数。
- 若要量第四节第 1 条的「抽得到」：在副本上给核心层加两个坏实现（同段清零写走 `write_at`、走 `write_zeroes_at`），各起一次崩溃注入战役，看 G4-丙 与守卫用例各红不红、红在第几个种子。重型，我没起。
- 第四节第 4 条的另几条全枚举流有没有「挂上即绿」当隐式判官，逐条读。

## 没做什么

- 没起 checker 档、崩溃注入战役与任何重型测试；没编译，没改 `crates/`、变异表、kb；没读副本，没读本轮攻方腿与判决。
- 没在入库装置上量任何一个数：第二节的「留不下」是读码加抽象模型；攻方第一轮的 42、40、6、20519、35 都是它副本上的数，我没复跑也没引作已核。
- 没验己（换稳固盘面）能不能在检查档那条红的测试里成立；没造 G4-丙、G4-丁、G4-戊 的实现，也没数它们在战役里抽到几个态。
- 没判 mkfs 自己的崩溃枚举流（在 D13 第 187 行的射程外）；没核 Q3 第三截计数没有断言、变异表没有行点名那一条（第一轮已记下，不在本轮）。
- 没替主 agent 撤销或维持判决，只交复核结果。

## cite-check 末行

```
$ python3 scripts/agents/cite-check.py research/prompts/m3-close-g4-rewrite-r2-sonnet-output.md
  ✓ 核了 65 处引文、25 处裸行号，对上 65 处，没判 25 处（其中裸行号没有引文可比 22 处；逐处列在上面）
```

草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-g4-rewrite-r2-sonnet` 里只有写者锁与认领文件，没有编译目录与仓副本（模型复跑的临时输出已删）。
