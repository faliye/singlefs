# m3-close-g4-rewrite-r3 攻方腿（Opus）报告（2026-10-22）

立场：攻前两轮站住的形态与第三轮两个候选（丙-置、丙-I），照正文分工表「云端攻方」那一行；不重攻 mkfs 末段、C670 与 Q3 拒因位置、甲-二 的开关。按 `crates/` 开工时的实现谈：开工时主工作区七份快照文件与 `research/prompts/m3-close-g4-rewrite-r3-snapshot/code-sha256.txt` 逐份 OK，我从主工作区拷了冻结树（`crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`）在草稿目录里编。量过的数都出自冻结树副本上的原型，没在入库装置上跑过。

## 复跑

模型目录 `research/prompts/m3-close-g4-rewrite-r3-opus-model/`：`src/` 是这一轮的原型（从第二轮那一份改出，第二轮的流原样留着），`r2-prototype/` 与 `toggles.patch` 是第二轮模型目录里那两样的逐字节拷贝，`tools/independent_count.py` 是 Y4 那条不共用代码的手算，`out/` 是这一轮的 20 份计数加手算的输出，`out-r2/` 是第二轮 18 条流在这一轮冻结树上的复跑输出。原型只 `cargo build`，编出来的二进制直接执行；崩溃状态取法同第二轮（`enumerate_crash_states_with_expansion` 加 `crash::full_expansion`，段内整写子集与原地覆写的撕裂态都展开，每态跑崩溃点重放自己的恢复与池级 checker，另挂一个观察者）。

    nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/m3-close-g4-rewrite-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-g4-rewrite-r3-opus/frozen --work <新的草稿目录>

脚本拷冻结树、打补丁、编两个原型，第二轮 18 条流用第二轮的二进制跑、与 `out-r2/` 逐字节比，仓里还有第二轮模型目录时另与它的 `out/` 去掉三种行再比；这一轮 20 条流用这一轮的二进制跑、与 `out/` 逐字节比；最后跑手算、与 `out/independent_count.txt` 比。每批 4 件并行，`.rc` 文件数与件数对不上整批作废（退 3）。我交回前在新目录里整趟跑过一次（约 22 分钟，本机，别的会话的 checker 档在跑），汇总原样：

    $ grep 'rerun_compare' rerun-check.log | awk '{print $2, $NF}' | sort | uniq -c
         38 how=byte same=yes
         21 how=stripped same=yes
          1 tag=independent_count same=yes
    $ grep -c 'rerun_batch.*rc=0' rerun-check.log; grep 'rerun_batch' rerun-check.log | grep -vc 'rc=0'
    38
    0

模板自带的 `--selftest` 拿假仓跑 `rerun_steps`，这一份改过的步骤要真编译，自证那一格跑不通，没跑。

| 文件 | sha256 |
|---|---|
| Cargo.toml | 0a7de3318a8459f35b2d768fdeee6d2d4f1d534a1710ef2506b12b4fe13e18f3 |
| out/aff_remount_roll_back.counts | 13ca9d032d780ca9809be011d48f87230848478549c07764bebdca36e953820e |
| out/aff_remount_twice.counts | 2afb9443b5b733650bf79589bef3be25b6a5c3c814a98b09c6ff39e429067ca8 |
| out/aff_remount_twice-SINGLEFS_PROTO_MUT_B1.counts | acafc08f8b46908f694dbcf53fe4c7ec41a4d0225aafa696765377f8a825bc87 |
| out/aff_remount_twice-SINGLEFS_PROTO_MUT_B2.counts | 2345c4f666130fca862a03a51b8eb26f401b1a55479a344eb341c13e51616770 |
| out/aff_remount_zeros_mount.counts | b5b501987b0e138c4e13f5b125155d9c209945e6ced5a3e6e7bcd0e7e0d9380a |
| out/amf_remount.counts | 7a66afaf246fa3b4c21f97fec11e7ac2132c5d0cada3ba034fcc1afedcb7f955 |
| out/amf_remount_publish_without_units.counts | 107fb3252f71786c7d97fe0a246c0affe418431af269f1c0562c6af9788ae7ef |
| out/amf_remount-SINGLEFS_PROTO_MUT_B1.counts | 776475dc54ff9c4fbfbe63685dc17c3fa47572b6010934ae1d1ae6428dbdf303 |
| out/amf_remount-SINGLEFS_PROTO_MUT_B2.counts | 3d2db8cc4739a5e64b743e4993c9fd0feb05a6cb0a0506dbccad62027cccb053 |
| out/amf_remount_twice.counts | 41609f98a84eebcc926b23f59a773e4770560b6a1e2dc7ededa550c014785b03 |
| out/amf_remount_twice-SINGLEFS_PROTO_MUT_B1.counts | 402cdfe6e25dc188248f22132315da31ebde30bad4d7963bcf2dc5d7d1011c96 |
| out/amf_remount_twice-SINGLEFS_PROTO_MUT_B2.counts | 6dea1d7efee5458d8732572da98e20c90ff6268d7dbf802c74295b4a8e0742d7 |
| out/amf_remount_zeros_mount.counts | 284aed057b1fa73d54c848fc03fad9ab40b12e6cd86b67c6bdd9c17ca271b291 |
| out/amf_zeros_mount.counts | 0a22a21dd15807bc0e8fdcf79421bf0c5bcfa63bdf54befcce65f09520935c39 |
| out/hand_made_image_one_slot_0.counts | b4deaf65e5ddbcd8968484753dc1ebf30fb8878b2a4a54c2df441bed2f8f2ed7 |
| out/hand_made_image_one_slot_1.counts | c432ddee20ba3e30f0b91c5ac96d313abd044b5fc68e62111d4f40b4904d319d |
| out/independent_count.txt | 19414c5ece5e1b5c6b0c53f8b97b8dbf349d1a5118a40795a16db8c39c4d11cb |
| out/make_filesystem_stream.counts | 6b7a2ed404da99d2368e6e306958af81020b00b9973457a8f767417d42d7f13d |
| out-r2/cold_start_then_remount.counts | 05feb8ce498b1d40daf4da6e120502f7a64946fbbbe0f6d8642f0755600365ab |
| out-r2/hand_made_image_both_slots_and_a_unit-none.counts | 66e5a07c6bb3e4cd1947c3031767a59dde3850200a3336f4fec36d2948f0bd15 |
| out-r2/hand_made_image_both_slots_and_a_unit-SINGLEFS_PROTO_GATE_NEVER_PASSES.counts | 6f42335c726646c321fabf700aa32b4293a3a62a9e8ed7f4551a7433cf53f27f |
| out-r2/hand_made_image_both_slots-none.counts | 78ed454c582e08c62cb6fd15eaef4b3eb5cd5432ef16c0e89a58469afc6ddfdf |
| out-r2/hand_made_image_both_slots-SINGLEFS_PROTO_GATE_NEVER_PASSES.counts | 91c3010a0a094f463bdff7a7eb94129a6355ba4f226bb6d22998cd849abc2bde |
| out-r2/remount.counts | 5550e120ec9c057d21530bbc972f638b34d9b94bb91bda7e31f32adfa32a01e0 |
| out-r2/remount-SINGLEFS_PROTO_MUT_B1.counts | a555e861af91ad679f53e3e02162a73f8cdf4b907159a53259c08defb1b74baa |
| out-r2/remount-SINGLEFS_PROTO_MUT_B1+SINGLEFS_PROTO_GATE_NEVER_PASSES.counts | 64221cb3b5d5d787aeec4d27beea213924e04b1b584759b9dd3d1c8de143f0a7 |
| out-r2/remount-SINGLEFS_PROTO_MUT_B2.counts | 924b8e9d3da0240529420995b20b374775ecca72c51a6ac1743aa3a0a7ca4689 |
| out-r2/second_crash_base_0.counts | e08c3ca411575d375815a67b426f74f07bca02248325c4c9e288b78707947dac |
| out-r2/second_crash_base_1.counts | 42aebe163ba7b624f16d3f726a50f5dbca74e86a3314c6790c2b89ae6d71f0c0 |
| out-r2/second_crash_base_2.counts | 5067a105388551cb1c3cebf160d6d3c4fcbd0de2392f050192beb1a207ade7a4 |
| out-r2/second_crash_base_3.counts | 889be0d020069bbe7816e4ac670ac191f5639fb186ca29f35ed152933c6dc74f |
| out-r2/second_crash_base_4.counts | e2aab0e5202e45bdd85c0a7821dac1c818a1b8a3829485555cd6e04efa40559e |
| out-r2/second_crash_base_5.counts | a18fe8a01ca05dac2ff1633ae6ec4d4bd425359e9024321ccfaf8e8efe5660c3 |
| out-r2/second_crash_base_6.counts | e9fa3a2ef8e6b25ccac2478f7e28bd4dc96ea10e3474db03ead8cc7fc02346f3 |
| out-r2/second_crash_base_7.counts | b6a998f4a8a4f190615891d9f025d7b782efbc3bb4843cbae276bcee3d024959 |
| out-r2/zeros_mount.counts | 8818b5542fa03f3c424603894ba1bef15dec9ec244e423f283b54f2a0ea0eba0 |
| out/rewrite_after_a_make_filesystem_crash_0.counts | 8ce7247bc2f14c88e065fe88179cf27077c7dcae7d62cefd7d9d786b8d0ac9e9 |
| out/rewrite_after_a_make_filesystem_crash_1.counts | 9bb0a5a40f8a21c49c378eaf2ab08ffdbd31fdf84850d654fa209f3d030f8ee4 |
| out/second_crash_on_a_hand_made_dead_base.counts | cc2266948fd6487539d2ce075c41aba025a074468cf5c0b573502d6b098cc698 |
| r2-prototype/Cargo.toml | f50aa534420cb9781c679b7ad8898570dc0fe2bf91c10b4f331764f1def3cbe3 |
| r2-prototype/src/main.rs | 70f4c2b8acf6f5c69da66e962cb6f691442c7eb75c205cc8be35bad5b6a7105d |
| src/main.rs | 48a633cd44dd32681fb08c421f4339925f7aa8f57bb467edf5cfb346a1873f4f |
| toggles.patch | 8c145a835bbc4688937ad9e5302a50d17be51c1432deacb7f4b5269cffab5e82 |
| tools/independent_count.py | e544f919c4e7d9b71b32c4a9bb4c29855316a5c44ee8f6378d2a66b145c62246 |
| tools/rerun.sh | f069779950566d3084ecd5aa19c0cf4ce36dd302c12f0e33aadf4a474bcd6365 |

## 各格判定一览

| 格 | 判定 | 量的 / 推的 |
|---|---|---|
| Y1 | 甲-二 不做：**撑得住**。G4-丙 出局三处理由：① 撑得住，且罩到丙-I 在随机历史里那一半；② **对丙-置 不成立、对丙-I 原样回来**；③ 两个候选都改读池级 checker 修得掉，留一处口径差（fsid）。第一轮的甲跟着 G4 拒绝那一臂走，不跟候选走。**撤回第二轮我自己的两句**：「崩溃点重放不跑可写挂载」（不对，观察者每态都挂）、「mkfs 写世代号 0」（是 1） | 复跑与读码；撤回那两句是读码 |
| Y2 | **打中三处**：① 丙-I 分不清 mkfs 中途与之后——照字面在 mkfs 自己的 6 态、补写流 3 态上误报；加世代号判别子在 mkfs 之后第一次取号漏 MUT-B1 4 态、MUT-B2 6 态，补写流误报 1 态（判别子观测不到）；② 丙-置 按段认来历在补写流 3 态上误报，要流报来历；③ G4 拒绝那一臂撤与留随候选翻面（丙-置 留、丙-I 撤），前两轮各对一半。**没打中（第二次抽样）**：正确实现在 mkfs 返回之后留下单盘全废——另 8 段历史 168 947 态（撕裂 250 态）0 态。丙-择 两个候选下都多余 | ①② 量的（原型与手算两路一致）；③ 读码加第二轮复跑过的 42/42 |
| Y3 | **打中**：I-8.5 一行「另报哪块盘少于两槽自证过、可写挂载会拒」在没有读故障的盘面上不成立，与补写例外无关（252 张盘面全挂上）；同一句另在 `image.rs` 两处、两条测试的文档注释、欠账表 C461 一行。每个候选顺着写出的 D13 第 187 行、I-8.5 稿各有一句说假话（表在 Y3） | 量的（可写挂载结局）；条文对照读码 |
| Y4 | **核上**：第二轮 `out/` 里 21 份计数（18 条流）在这一轮的冻结树上复跑，去掉老几版原型不打的三种行之后全对上，最后一版那几份逐字节相同；W1 172 366、MUT-B2 16 + 1、MUT-B1 8 / 34 另用手算（不共用代码）核上。**打中第二轮自己的复跑脚本**：照它的比法 8 个 `same=no`，是它比老几版输出的毛病，不是数变了 | 量的 |

## Y1：第二轮「已定」的几条撑不撑得住

| 第二轮已定 | 这一轮的判定 | 量的 / 推的 |
|---|---|---|
| 甲-二 不做 | **撑得住，没打中**。今天的判官不读补写闸，甲与甲-二 两张盘面判官结论逐字段相同（第二轮 W3-②，这一轮复跑对上：`hand_made_image_both_slots-SINGLEFS_PROTO_GATE_NEVER_PASSES` 与 `hand_made_image_both_slots_and_a_unit-none` 两份 `byte_same=yes`）；丙-置、丙-I 都只看两槽，也不读闸。撤掉 G4 拒绝那一臂时甲-二 连断言的对象都没有 | 量的（复跑）+ 推的 |
| G4-丙 原形态出局理由 ①：挂在崩溃注入只看得见整写子集 | **撑得住**，且同一条理由罩到丙-I 在随机历史里的那一半：崩溃注入第一、三截交给定状态表判（`crates/singlefs-checker-tier/src/crash_injection.rs:1306`：「/// [`inject_crashes_into_history`]，第一截与第三截上只读一张镜像的那几样（池级 checker、两遍恢复、记录核对器）交 `given_states_judge` 判」），状态仍是按种子摆的整写子集。MUT-B2 那 16 态（全在撕裂域）丙-I 在随机历史里一样看不到，只在崩溃点重放的枚举流里看得到 | 读码；16 态撕裂是量的（Y4 复跑与手算两路） |
| 理由 ②：与 G4 拒绝那一臂一件事记两条 | **对丙-置 不成立，对丙-I 原样回来**（Y2 第 4 节）。丙-置 是枚举流的观察者，给定状态表那一路没有观察者（`crates/singlefs-checker-tier/src/crash_given_states.rs` 里 `grep -c observer` 为 0），G4 那一臂住崩溃注入第二截，两者不在同一趟里判同一张盘面。丙-I 改的是池级 checker，崩溃注入第一截经给定状态表跑池级 checker；第二截的可写挂载就起在第一截那张镜像上，闸不过时 G4 那一臂再报一次 | 读码；「两条都报」的盘面数用第二轮 MUT-B1 加闸恒不过那 42/42（复跑对上），丙-I 的谓词与那份原型里 G4-丙 的谓词在那 42 态上同值（第二轮 `-o2` 那份 `o2_reading` 0 行，复跑 `byte_same=yes`） |
| 理由 ③：判自证共用核心层算法 | **两个候选都修得掉，留一处口径差**：两者都改读池级 checker 的逐盘数。池级 checker 的「自证过」不分 fsid，也把「自证过而不可择」算进去（`crates/singlefs-checker/src/image.rs:1490`：「那一半（fsid 是别的池的槽）这里不分：数的是 I-8.5 那一行的」，后面接的是「自证过」），核心层补写闸认的「全废」含别的池的槽（`crates/singlefs-core/src/mount.rs:5781`：「只认这一次读得出、自证不过的（撕裂、清零、别的池的）」，说的是两槽「全废」）。这一轮 19 趟流（正确实现与两个坏实现，mkfs 之后与 mkfs 自己）里两种读法只在「两盘全废、判官择不到系统配置」那几态上不同，单盘那一格 0 处不同 | 量的（`o2_reading` 行，见 Y2） |
| 第一轮的甲（测试盘面另坏一个单元，让补写闸闸一不过） | **跟着 G4 拒绝那一臂走，不跟候选走**。那条测试直接调第三截的判法（`crates/singlefs-checker-tier/src/crash_injection.rs:8968`：「fn a_remount_refused_on_a_second_crash_image_where_device_zero_has_both_system_configuration_slots_torn_is_an_illegal_refusal_after_the_read_stage(」，函数体里调 `judge_the_remount_after_a_second_crash`），不经池级 checker、不经崩溃点重放：丙-置、丙-I 采哪个它都照绿。臂留着，它是 `crates/mutations.tsv` 第 2408 行唯一的杀手，要留；臂撤掉，它与第 2406–2408 行一起撤。丙-I 另要改的是池级 checker 那条测试：`crates/singlefs-harness/tests/checker_known_bad_images.rs:8861`：「"盘 0 两槽都还自证过：单盘全废不判违例"」——它的盘面是第一个文件之后的池（世代号大于 mkfs 写的 1），丙-I 照字面与加判别子都判违例，这一句断言要反过来 | 读码 |

### 撤回第二轮我自己的两句（读码与量的）

1. 第二轮报告 `research/prompts/m3-close-g4-rewrite-r2-opus-output.md:219`：「崩溃点重放不跑可写挂载。」——**不对**。崩溃点重放的观察者在每个崩溃状态上起可写挂载，挂不上就红：`crates/singlefs-checker-tier/tests/common_crash_point_observers/mod.rs:194`：「.unwrap_or_else(|error| panic!("每个崩溃状态上可写挂载都做得成：{error:?}"));」；挂完的盘再交池级 checker 判（`crates/singlefs-checker-tier/src/crash_amplification.rs` 的 `pool_checker_of_the_images_after_the_observers`）。登记在 `crates/singlefs-checker-tier/src/judging_units_entries.rs` 的观察者里，起可写挂载的至少有续做、截断续做、压实续做、快照续做、`remount_after_the_crash`、`writable_mount_after_the_crash` 这几族。崩溃注入文件头同一句话也说过了头：`crates/singlefs-checker-tier/src/crash_injection.rs:18`：「//! checker、记录核对器）。崩溃点重放不跑可写挂载，这两截只在这里有。」。对这一题的影响：在挂了这类观察者的流上，补写闸不过的单盘全废态今天已经会被观察者的 panic 判红（不是被 G4 判），补写闸过了的照样无声补上——第一轮那个空档在崩溃点重放里也在。
2. 第二轮报告 `research/prompts/m3-close-g4-rewrite-r2-opus-output.md:112`：「mkfs 把两槽都写成世代号 0」——**不对**：`crates/singlefs-core/src/make_filesystem.rs:52`：「pub const SYSTEM_CONFIGURATION_GENERATION_AT_MKFS: u64 = 1;」。那一段的结论（mkfs 之后的写不丢唯一自证的那一槽）不受影响：两槽同为世代号 1，第一次取号写世代号 2、落槽 0。这个数在这一轮成了丙-I 判别子的门槛（Y2）。

推翻条件：今天的判官改成读补写闸（甲-二 才有对象）；或给定状态表那一路接上观察者（理由 ② 对丙-置 回来）。

## Y2：丙-置 与丙-I

原型 `research/prompts/m3-close-g4-rewrite-r3-opus-model/`（从第二轮那一份改出，第二轮的流与判法原样留着）在每个崩溃状态上另记一行：池级 checker 的逐盘数（`system_configuration_redundancy_report`）有单盘全废时，两个候选各判出什么——丙-I 照字面（`c_i_plain`：单盘全废就报，全池皆废归今天的 I-8.5）、丙-I 加盘面判别子（`c_i_generation`：全池本池自证过的槽里世代号最大的大于 mkfs 写的 1 才报）、丙-置 照字面（`c_place_literal`：流报自己的来历，只判 mkfs 返回之后）、丙-置 按段（`c_place_by_segment`：流不报来历，只认得出 mkfs 自己的写）。判别子是我能找到的最好的盘面量：mkfs 两槽写世代号 1（`crates/singlefs-core/src/make_filesystem.rs:52`：「pub const SYSTEM_CONFIGURATION_GENERATION_AT_MKFS: u64 = 1;」），之后每写 +1（`crates/singlefs-core/src/transaction.rs:572`：「世代号 = 这块盘两槽里自证过的最大世代号 + 1（两槽都读不出时从 1 起）」）。

### Y2 第 1 节：撕裂域里正确实现在 mkfs 返回之后留不留得下单盘全废态——没打中（第二次抽样，量的）

第二轮 W1 只抽过一次（关闭再挂、读故障那一步、冷启动再挂、八个基线）。这一轮换了历史再抽，用户动作放开到下表：

| 历史（起点 / 用户动作） | 状态数 | 撕裂态 | 单盘全废 | 只少一槽的态（全挂上） |
|---|---|---|---|---|
| mkfs 之后 / 可写挂载（`amf_remount`） | 41 | 20 | 0 | 20 |
| mkfs 之后 / 可写挂载两次（`amf_remount_twice`） | 336 | 40 | 0 | 40 |
| mkfs 之后 / 可写挂载、读故障那一步（`amf_remount_zeros_mount`） | 336 | 40 | 0 | 40 |
| mkfs 之后 / 读故障那一步（`amf_zeros_mount`） | 41 | 20 | 0 | 20 |
| mkfs 之后 / 可写挂载、零单元发布（`amf_remount_publish_without_units`） | 53 | 25 | 0 | 25 |
| 第一个文件之后 / 可写挂载、回退（`aff_remount_roll_back`，第二轮交主 agent 没跑的那一条） | 86 066 | 25 | 0 | 25 |
| 第一个文件之后 / 可写挂载两次（`aff_remount_twice`） | 41 037 | 40 | 0 | 40 |
| 第一个文件之后 / 可写挂载、读故障那一步（`aff_remount_zeros_mount`） | 41 037 | 40 | 0 | 40 |

合计 168 947 态、撕裂 250 态，单盘全废 0。原样（每份 `.counts` 里除计数行外没有带 `o2_dead=` 的行；例 `out/aff_remount_roll_back.counts`）：

    name=aff_remount_roll_back domain=torn states count=25
    name=aff_remount_roll_back domain=whole states count=86041

试过而没写盘、不进表的用户动作：第一个文件之后回退（`TargetNotACandidate`）、可写挂载后抬 F 一步（`RollbackFloorAboveCeiling`）、可写挂载后崩溃恢复抛弃最新根（`NewerStateStillUnreadableAfterOneReread`，拒）、可写挂载后零单元发布（`NotApplicable`）；删除（`aff_unlink`）5 242 903 态，过了约 10⁶，没跑。

加上第二轮那一次，这一格两次抽样都没打中。推翻条件：覆盖写、清扫、删除那几条大流（各超过 10⁶），三块盘的池，或别的几何上数出单盘全废。


### Y2 第 2 节：打中——丙-I 分不清 mkfs 中途与之后（量的）

被判的那一句（r3 正文丙-I 的定义）：「改 I-8.5 本身——mkfs 返回之后的盘面上单盘全废进违例通道，旁路报告只留给 mkfs 中途」。池级 checker 判的是一张盘面（D13 已定项 7），「mkfs 返回之后」是历史的事。量了三种流、两种读法：

| 流 | 单盘全废态（整写 / 撕裂） | 丙-I 照字面报 | 丙-I 加判别子报 | 该不该报 |
|---|---|---|---|---|
| mkfs 自己（新盘，`make_filesystem_stream`，4 117 态） | 6 / 0（盘 0、盘 1 各 3） | 6（误报） | 0 | 不该（mkfs 中途，D13 第 187 行划出射程） |
| mkfs 崩在最后一段、一块盘两槽没落，之后那次可写挂载（补写）的全部崩溃态（`rewrite_after_a_make_filesystem_crash_0` / `_1`，各 35 态） | 每条 2 / 1 | 3（误报） | 1（误报：健康那块盘第一轮写了世代号 2，补写那一写没落） | 不该（mkfs 没有返回过） |
| mkfs 之后第一次可写挂载，MUT-B1（`amf_remount` 加 MUT-B1，113 态） | 6 / 36 | 42 | 38（漏 4，全在撕裂域：盘 1 第一轮那一写没落或撕裂，全池最大世代号还是 1） | 该 |
| 同上，MUT-B2（105 态） | 0 / 16（另 1 态两盘全废） | 16 | 10（漏 6，全在撕裂域） | 该 |
| mkfs 之后可写挂载两次，MUT-B1 / MUT-B2（`amf_remount_twice`，各 480 / 464 态） | 14 / 76，0 / 32（另 2 态两盘全废） | 90，32 | 86，26（漏的同上那 4、6 态，都在第一次取号） | 该 |

原样（`out/rewrite_after_a_make_filesystem_crash_1.counts`、`out/amf_remount-SINGLEFS_PROTO_MUT_B1.counts`，第一行是流的基镜像）：

    name=rewrite_after_a_make_filesystem_crash_1 base=[dead_device=1 slots_per_device=[2, 0] o2_dead=[1] highest_generation=Some(1)] first_mount=mounted
    name=rewrite_after_a_make_filesystem_crash_1 domain=torn origin=mount_after_a_make_filesystem_crash o2_dead=[1] highest_generation=Some(1) c_i_plain=fires c_i_generation=silent c_place_literal=not_judged c_place_by_segment=fires count=1
    name=rewrite_after_a_make_filesystem_crash_1 domain=whole origin=mount_after_a_make_filesystem_crash o2_dead=[1] highest_generation=Some(1) c_i_plain=fires c_i_generation=silent c_place_literal=not_judged c_place_by_segment=fires count=1
    name=rewrite_after_a_make_filesystem_crash_1 domain=whole origin=mount_after_a_make_filesystem_crash o2_dead=[1] highest_generation=Some(2) c_i_plain=fires c_i_generation=fires c_place_literal=not_judged c_place_by_segment=fires count=1
    name=amf_remount domain=torn origin=after_make_filesystem_returned o2_dead=[0] highest_generation=Some(1) c_i_plain=fires c_i_generation=silent c_place_literal=fires c_place_by_segment=fires count=4

这几个数换一条路也核过：`tools/independent_count.py`（Y4 那一份手算）对 `amf_remount` 加 MUT-B1、MUT-B2 与补写流给出同样的分法（漏 4、漏 3 + 3、补写流 3 态里判别子报 1 态）：

    name=independent_count flow=amf_remount-SINGLEFS_PROTO_MUT_B1 domain=torn dead=[0] generation_discriminator_silent count=4
    name=independent_count flow=amf_remount-SINGLEFS_PROTO_MUT_B2 domain=torn dead=[0] generation_discriminator_silent count=3
    name=independent_count flow=amf_remount-SINGLEFS_PROTO_MUT_B2 domain=torn dead=[1] generation_discriminator_silent count=3
    name=independent_count flow=rewrite_after_a_make_filesystem_crash_1 domain=whole dead=[1] generation_discriminator_fires count=1

这些态落在哪些入库的流上（读码）：漏报的态在「mkfs 之后第一次取号」那两段的撕裂态里，合表里从 mkfs 之后起、枚举第一次取号的流有 `new-pool-file-creation`、`writable-mount-of-an-io-min-8192-pool`、`formatted-pool-remounted-without-a-file`（`.claude/gate.d/crash-point-replay-items.tsv` 第 12、33、34 行）；照字面误报的 mkfs 自己的态在 `make-filesystem-over-a-used-pool` 那条流上（那条流的断言见 Y3 表）；补写流的误报要等第二轮记账要补的「补写本身中途崩溃」那条流落地才出现。崩溃注入摆不到 mkfs 中途（`crates/singlefs-checker-tier/src/crash_injection.rs:4777`：「/// 起点那一段（取号、暖机、第一个文件）在内——2026-09-20 才放开；mkfs 那几段不在内，与崩溃点重放同一条界：」），所以补写流那一形在随机历史里摆不出来；漏报那几态全在撕裂域，崩溃注入也摆不出来。

四句：
1. 分不分辨臂：分辨。丙-置 照字面在这几条流上逐态判对（`c_place_literal`：mkfs 自己与补写流都是 `not_judged`，坏实现那几条全 `fires`）；丙-I 两种读法都有错的格。
2. 被判的系统看不看得到：**看不到**——这是 `.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` 表里「判别子观测不到」那一形：池级 checker 在判的那一刻只有一张盘面，「mkfs 返回过没有」不在盘面上。世代号是代用量，两个方向都量到了错。
3. 满足的分句：丙-I 定义里「mkfs 返回之后的盘面上」与「旁路报告只留给 mkfs 中途」这两个限定。
4. 改法：丙-I 加世代号判别子，在漏的 10 态与误报的 1 态上照中；丙-置 照字面不中，但要每条流报自己的来历（下一节）。

推翻条件：找到一个池级 checker 在一张盘面上读得出、在上面几条流上逐态分对的量（例如实现往盘上另写一个「mkfs 已返回」的标记，那是改格式，不在这一题里）。


### Y2 第 3 节：打中——丙-置 的「只判 mkfs 返回之后的段」要流报来历，按段认会误报（量的）

丙-置 照正文写的是「只判 mkfs 返回之后的段」。一条流的写表里认得出 mkfs 自己的写（`make-filesystem-over-a-used-pool` 那条流整条都是），认不出「这条流的基镜像是 mkfs 中途的崩溃态」：补写流的写表里只有那次可写挂载的写。上一节的表里，补写流 3 态单盘全废，`c_place_by_segment=fires` 3 态全误报，`c_place_literal=not_judged` 才对。

- 四句：分辨丙-置 的两种接法（按段 / 流报来历）；被判的观察者看得到写表、看不到基镜像怎么来的，按段认就是「判别子观测不到」，流报来历之后看得到；满足的是「只判 mkfs 返回之后的段」里「mkfs 返回之后」；改法：给流加一样来历（mkfs 返回之后 / mkfs 中途 / mkfs 中途崩溃之后的挂载），观察者照它判——只在我的模型上量过，被攻过零轮。
- 这一条今天没有入库的流会碰上：补写流还不存在（第二轮判决把它记成要补的欠账）。补那条流时若照常挂丙-置，3 态红（推的，按上面的量）。

### Y2 第 4 节：挂在哪、判几次、各记几条、动判定库多少（读码为主）

| 问 | 丙-置 | 丙-I |
|---|---|---|
| 住在哪一档、罩哪些状态 | 枚举流的观察者。观察者一条流一条地登记（`crates/singlefs-checker-tier/src/judging_units_entries.rs:123`：「/// 观察者的身份就是这几个入口 fn 的闭包键（同一份文件里两个观察者各自一把）；没登记的观察者取键拒算（照判、不入库）。」）。合表 34 个测试目标里 30 个调枚举入口，4 个不调（`crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library`、`crash_injection_campaign`、`crash_injection_read_faulted_states_never_hold_the_witness_criterion`、`record_checker_judges_absence_by_the_persisted_set`），给定状态表那一路没有观察者 | 池级 checker 跑到的每一处：枚举流的崩溃镜像、观察者交回的盘（`pool_checker_of_the_images_after_the_observers`）、给定状态表（崩溃注入第一、三截，`crates/singlefs-checker-tier/src/crash_given_states.rs:3`：「//! 随机历史两项（崩溃注入、读故障）建表交它判，崩溃点重放的引擎（`crate::crash_amplification::GivenStatesEngine`）实现这个 trait。」）、GPU 判器（I-8.5 在内核里有一份：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:5172`：「// 别的不判；再 I-8.5（全池没有一个自证过的槽）、择系统配置、系统配置与根环那几条、逐盘 journal，根环里有根才进走读。」） |
| 分得清 mkfs 中途与之后 | 流报来历就分得清（第 3 节） | 分不清（第 2 节） |
| 与 G4 拒绝那一臂同一张盘面记几条 | 不在同一趟：臂在崩溃注入第二截，丙-置 不在崩溃注入里。各档各一条 | 留臂：两条（第一截池级 checker 一条、第二截臂一条；第二截的可写挂载起在第一截那张镜像上）。数是第二轮 MUT-B1 加闸恒不过的 42/42（这一轮复跑对上），丙-I 的谓词与那份原型的 G4-丙 在这 42 张盘面上同值（闸只管挂载、不改盘面，不带闸开关的 `-o2` 那一趟 `o2_reading` 0 行） |
| 与 C670、Q3 两臂 | 0 条：两拒在补写闸之前发，第二轮量的 `other_conditions=0` 复跑对上（不重攻拒因位置） | 同左 |
| 观察者交回的盘上再判一次 | 不涉及 | 挂了可写挂载观察者的流：闸过了补写补上，交回的盘上不再全废，一条；交回原镜像的观察者（例 `crash_image_handed_back`）上同一张盘面判两次（推的） |
| 判定库单元键 | 新加一个观察者，只多一把它自己的键；别的块不动（推的，读码同上） | 池级 checker 在「判镜像的定义」那一组的闭包里（`crates/singlefs-checker-tier/src/judging_units_entries.rs:25`：「/// - 判镜像的定义（`judgement_of_the_crash_images`）：CPU 参照判器 `crash::judge_crash_image` 与结论的编码（红项、计数向量）；镜像判定的版本」），改它就换这一组的键，库里每条流的镜像判定块全要重判；内核那一份也要改，`gpu_judge` 一组的键一起换（推的，读码） |
| 判槽自证读谁的读法 | 池级 checker 的逐盘数 | 同左（I-8.5 本来就是它） |


### Y2 第 5 节：丙-择、G4-戊、G4 拒绝那一臂撤与留

**丙-择（择不到系统配置也记一条）：两个候选下都多余（量的）。** 第二轮要它，是因为判官那一份读法择不到系统配置时交 `None`、G4-丙 无从判。两个候选都改读池级 checker 的逐盘数，它在两盘全废那几态上照样点名两块盘（`o2_reading judge=None o2=[0, 1]`）；而且那几态今天就红两处：I-8.5（全池皆废）与 oracle（`no_root_chosen`）。这一轮带两盘全废态的三趟：`amf_remount` 加 MUT-B2 1 态、`amf_remount_twice` 加 MUT-B2 2 态、第二轮 `remount` 加 MUT-B2 1 态（复跑），崩溃点重放自己的判分别是 `violations:1`、`violations:2`、`violations:1`。四句：不分辨丙-置 与丙-I；看得到；满足的是第二轮判决交来的「丙-择」那一项；改法：不做丙-择。

**G4-戊（第三截只判新增的无槽盘）：问题换到丙-I 上原样回来，丙-I 却做不了「只判新增」（量的 + 读码）。** 第三截的二次崩溃镜像也交给定状态表、跑池级 checker（`crates/singlefs-checker-tier/src/crash_injection.rs:1306` 那一句，Y1 抄过）。第一次崩溃镜像上一块盘两槽全废、补写闸过了，可写挂载在取号第一轮补写它；补写那一写落之前崩，盘照旧全废，丙-I 在第三截再记一次同一块盘。原型 `second_crash_on_a_hand_made_dead_base`（第一个文件之后盘 0 两槽各翻一字节当第一次崩溃镜像，挂载补写，枚举这次挂载的全部二次崩溃）：20 519 态里盘 0 仍全废 6 态（整写 2、撕裂 4），整写那 2 态都在取号第一轮（崩溃注入第三截在取号段摆一个二次崩溃）。原样：

    name=second_crash_on_a_hand_made_dead_base base=[slots_per_device=[0, 2] o2_dead=[0] highest_generation=Some(6)] first_mount=mounted
    name=second_crash_on_a_hand_made_dead_base domain=whole origin=after_make_filesystem_returned o2_dead=[0] highest_generation=Some(6) c_i_plain=fires c_i_generation=fires c_place_literal=fires c_place_by_segment=fires count=1
    name=second_crash_on_a_hand_made_dead_base domain=whole origin=after_make_filesystem_returned o2_dead=[0] highest_generation=Some(7) c_i_plain=fires c_i_generation=fires c_place_literal=fires c_place_by_segment=fires count=1

池级 checker 只有这一张盘面，认不出「这块盘在第一次崩溃时就已全废」，G4-戊 那种「只判新增」在丙-I 下写不出来；要去重只能在发现那一层按签名并（推的）。丙-置 不在崩溃注入里，这一格对它不适用。四句：分辨丙-I 与丙-置；丙-I 看不到第一次崩溃镜像（判别子观测不到）；满足的是第二轮交来的 G4-戊 那一项；改法：G4-戊 对丙-I 不可用。这条流崩溃点重放自己的判全红（`violations:20519`，首条 `file_read_where_this_root_has_none`）是原型没给这条流带版本表的装置毛病，第二轮八个基线那几趟同样如此（复跑看到的，第二轮报告没提），不影响观察者那一路的计数。

**G4 拒绝那一臂撤与留：答案随候选翻面（读码为主，数是第二轮的量、这一轮复跑）。**

| | 采丙-置 | 采丙-I |
|---|---|---|
| 留臂 | 不重报（不同档）。臂罩的是：崩溃注入随机历史里、补写闸不过的单盘全废（只整写子集）。丙-置 不进随机历史，这一块只有臂在罩 | 重报：同一张第一次崩溃镜像，第一截丙-I 一条、第二截臂一条（42/42）。臂多罩的：0——它能报的盘面，第一截的池级 checker 先看到了（臂只在没注入读故障的崩溃镜像上判，第一截判的就是那张镜像） |
| 撤臂 | 丢：随机历史里坏实现留下、补写闸又不过的单盘全废（`hand_made_image_both_slots_and_a_unit` 那一形）没人判（正确实现留不下这种盘面，Y2 第 1 节）；补写闸恒不过那一类坏实现在崩溃注入里被判合法（U6 复算为真），只剩 harness 档那条正向测试（第二轮 W3 点名的 `crates/mutations.tsv` 第 5010 行）。E171 装置（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3529`：「RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults => {」）与第 2406–2408 行三条变异要改指或删 | 不丢（推的）：随机历史第一、三截的盘面池级 checker 都判；第 2406–2408 行改指池级 checker 那一处，第 2407 行「注入过读故障的崩溃态也判」对池级 checker 没有对象（它只看盘面，不看读故障），要删或换一条 |

四句：分辨臂（撤与留在两个候选下答案相反）；看得到；满足的是正文「G4 拒绝那一臂撤与留」那一问；改法：辩方第二轮「撤不得」在丙-置 下成立、在丙-I 下不成立；攻方第二轮「撤掉、三行改指」反过来——前两轮各自对了一半，要先定候选再定撤留。

## Y3：未定项清单与条文候选

### 全仓 grep（命令与命中数，量的）

    $ for p in '可写挂载会拒' '单盘全废' '两槽全废' '两槽都废' '一份自证过的系统配置都没有' '一份本池自证过的系统配置都没有' '少于两槽自证过'; do echo "== pattern=$p total_lines=$(grep -rnF --exclude-dir=target --exclude-dir=.git "$p" . | wc -l)"; grep -rnF --exclude-dir=target --exclude-dir=.git "$p" . | sed 's#^\./##' | awk -F/ '{d=$1; if (d==".claude") d=$1"/"$2; print d}' | sort | uniq -c | sort -rn | tr '\n' ';'; echo; done
    == pattern=可写挂载会拒 total_lines=89
         82 research;      5 crates;      1 records;      1 .claude/kb;
    == pattern=单盘全废 total_lines=113
         96 research;      8 crates;      5 scripts;      3 .claude/kb;      1 records;
    == pattern=两槽全废 total_lines=140
        113 research;     20 crates;      5 records;      2 .claude/kb;
    == pattern=两槽都废 total_lines=2
          2 crates;
    == pattern=一份自证过的系统配置都没有 total_lines=132
        105 research;     19 crates;      7 .claude/kb;      1 records;
    == pattern=一份本池自证过的系统配置都没有 total_lines=34
         24 research;      9 crates;      1 .claude/kb;
    == pattern=少于两槽自证过 total_lines=77
         74 research;      2 crates;      1 .claude/kb;

（工作目录是主工作区仓根；`research/` 下的命中几乎全是各轮背景材料与附录抄的同一批条款，不逐条列。）`research/` 之外，说「单盘全废 / 少于两槽 ⇒ 可写挂载会拒」这同一件事、前两轮没点名的（逐处 `grep -nF` 过）：

- `crates/singlefs-checker/src/image.rs:350`：「单盘全废、哪块盘少于两槽自证过（可写挂载会拒）」——`SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION` 的文档注释。
- `crates/singlefs-checker/src/image.rs:1484`：「/// 少于两个槽自证过的盘：可写挂载读阶段读每块盘两槽，有一槽读不出或自证不过就判不出、按真，重读一次仍不成就拒可写」
- `crates/singlefs-harness/tests/checker_known_bad_images.rs:8833`：「/// 单盘全废不进违例通道：盘 1 两槽都翻坏，I-8.5 照样成立（盘 0 有自证过的槽），旁路报告点名盘 1 两槽都废、可写挂载会拒。」，下一行（第 8834 行）说只坏一槽的也「可写挂载会拒」。
- `crates/singlefs-harness/tests/system_configuration_slot_with_a_foreign_node_size_is_not_chosen.rs:95`：「旁路报告把盘 0 记成少一槽自证过（可写挂载会拒）」
- `.claude/kb/checks-owed.md:732`：「另报哪块盘少于两槽自证过、可写挂载会拒」（C461 一行）。
- `crates/singlefs-checker/src/walk.rs:17323`：「单盘全废走旁路报告，不进违例通道」（丙-I 要改的落点之一）。
- `scripts/experiment-apparatus/e170_o3_spec_executor/system_configuration.py:160`：「单盘两槽都不可择不拒池」，后面括注引 I-8.5「单盘全废不判违例」；这一形的文档串在 `scripts/experiment-apparatus/e170_*/system_configuration.py` 共 5 份（O3 规格执行器）；丙-I 改了 I-8.5，它们引的那半句就不在了（它们说的「不拒池」本身还对）。
- `crates/singlefs-checker-tier/src/crash_injection.rs:18`：「崩溃点重放不跑可写挂载，这两截只在这里有。」（Y1 撤回那一条），丙-置 与撤臂的取舍要读它。

### 打中 Y3-①（量的）：「少于两槽自证过、可写挂载会拒」在没有读故障的盘面上不成立，与补写例外无关

被判的那一句（I-8.5 一行）：`crates/singlefs-checker/invariants/03-journal.md:12`：「单盘全废不判违例，走旁路报告：另报哪块盘少于两槽自证过、可写挂载会拒。」；代码注释给的理由：`crates/singlefs-checker/src/image.rs:1484`：「有一槽读不出或自证不过就判不出」。

而 D23 已定项 14 的「判不出」要有读错：`.claude/kb/decisions/23-journal的角色与格式.md:396`：「判不出取 R2，权威写法在 D22（单元原子性怎么合成） 已定项 8：有槽设备报读错，且没有一块盘两槽都自证过」。一槽撕裂、设备不报错，不是判不出，挂载照常。

量的：这一轮每条流的每个崩溃状态上，凡有一块盘只剩一槽自证过、没有盘全废、只读恢复没失败的，都在那张盘面上起一次可写挂载（没有读故障），池级 checker 的旁路报告在每一态都把那块盘列进 `devices_whose_writable_mount_would_be_refused`，挂载结局：

| 流（正确实现） | 这类态数 | 挂上 | 拒 |
|---|---|---|---|
| `amf_remount`（mkfs 之后第一次可写挂载） | 20 | 20 | 0 |
| `amf_remount_twice` / `amf_remount_zeros_mount` | 40 / 40 | 40 / 40 | 0 |
| `amf_zeros_mount` / `amf_remount_publish_without_units` | 20 / 25 | 20 / 25 | 0 |
| `aff_remount_twice` / `aff_remount_zeros_mount` / `aff_remount_roll_back`（第一个文件之后） | 40 / 40 / 25 | 40 / 40 / 25 | 0 |
| 手造：第一个文件之后盘 0 只翻坏槽 0 / 只翻坏槽 1 | 1 / 1 | 1 / 1 | 0 |

原样（`out/hand_made_image_one_slot_0.counts`、`out/amf_remount.counts`）：

    name=hand_made_image_one_slot_0 slots_per_device=[1, 2] o2_report=SystemConfigurationRedundancyReport { devices_without_a_self_verified_slot: [], devices_whose_writable_mount_would_be_refused: [0] } writable_mount=mounted slots_per_device_after_the_mount=[2, 2]
    name=amf_remount domain=torn origin=after_make_filesystem_returned short_of_a_slot o2_would_be_refused=[0] slots_per_device=[1, 2] writable_mount=mounted count=8

两个坏实现的流上这类态也一个都没拒（`aff_remount_twice` 加 MUT-B1 41 066 态、其余几趟见 Y2 的表）。手造那张盘面挂完两槽都补回来了（`slots_per_device_after_the_mount=[2, 2]`）：取号写两轮覆盖了坏的那一槽。

四句：
1. 分不分辨臂：不分辨丙-置 与丙-I——这句话两个候选都要改（丙-置 留着旁路报告，丙-I 也说「旁路报告只留给 mkfs 中途」）。它分辨的是「I-8.5 那一行照抄旧理由」与「改理由」。
2. 被判的系统看不看得到：看得到（盘面上就是一槽自证不过）。
3. 满足的分句：I-8.5 一行「另报哪块盘少于两槽自证过、可写挂载会拒」的后半「可写挂载会拒」；`SystemConfigurationRedundancyReport::devices_whose_writable_mount_would_be_refused` 这个字段名本身也是这句话。
4. 改法：正文给的两个候选都没碰这半句，照中；要改的是把「可写挂载会拒」换成「带读故障时可写挂载可能判不出而拒；没有读故障时挂得上，取号写两轮补回那一槽」，或把这个字段改名、改成只报「少一槽」。

推翻条件：在入库装置上同样的盘面可写挂载被拒；或 D23 已定项 14 的判不出改成不要读错。


### 条文候选，与每一稿哪一句说假话

下面每一稿是照正文那个候选「顺着写」出来的样子（是我写的靶子，不是推荐），被攻过零轮；每格注明说假话的依据是量的还是推的。

| 条款 | 照丙-置 写 | 哪一句说假话 | 照丙-I 写 | 哪一句说假话 |
|---|---|---|---|---|
| I-8.5（`crates/singlefs-checker/invariants/03-journal.md:12`） | 不变量本身不动；把「可写挂载会拒」改成「mkfs 中途留得下（D22 已定项 8）；mkfs 返回之后的由崩溃点重放的观察者判」 | 「由观察者判」：给定状态表那一路（崩溃注入、读故障两项随机历史）没有观察者，随机历史里没人判（推的，读码：`crash_given_states.rs` 里 `observer` 0 处）；臂撤掉时连闸不过的那一形也没人判 | 「mkfs 返回之后的盘面上单盘全废判违例，mkfs 中途的盘面走旁路报告」 | 「mkfs 返回之后的盘面」：池级 checker 只看一张盘面，分不出（Y2 第 2 节，量的）。用世代号作判别子：MUT-B1 漏 4 态、MUT-B2 漏 6 态，补写流误报 1 态 |
| 同上，旁路报告那半句 | 「另报哪块盘少于两槽自证过、可写挂载会拒」若照抄 | 量的：没有读故障时这类盘 0 次被拒（Y3-①） | 同左 | 同左 |
| D13 第 187 行（`.claude/kb/decisions/13-验证路线.md:187`） | 「这一性质由崩溃点重放 mkfs 返回之后的全部状态（第三态在内）上的观察者判」 | ① 「全部」：只罩挂了它的枚举流，34 个合表测试目标里 4 个不是枚举流（崩溃注入战役、读故障、给定状态表随机历史、记录核对器那一项），随机历史整片不在（推的）；② 「mkfs 返回之后」要流自己报来历：补写中途崩溃那条流（第二轮记的欠账要补的就是它）里 mkfs 没有返回过，观察者按段认会误报 3 态（量的，Y2 第 3 节） | 「这一性质由池级 checker 的 I-8.5 判（mkfs 返回之后的盘面）」 | 「mkfs 返回之前的崩溃态不在射程」这半句池级 checker 做不到（量的，Y2 第 2 节）；照字面不加判别子时，崩溃点重放 `make-filesystem-over-a-used-pool` 那条流的断言（`crates/singlefs-checker-tier/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs:272`：「"{invariant} 判违例的状态数（只有整池拒的状态上 I-8.5 红）：{:?}",」）立刻对不上（mkfs 自己的单盘全废态，新盘 6 态量的、用过的盘态数是第一轮的数，这一轮没重量） |
| 同上，「与 C670 那条同接法」 | 留着就说假话（两候选都不再挂在拒绝上；第二轮已攻过，这里不重攻） | — | 同左 | — |
| D13 第 72 行（`.claude/kb/decisions/13-验证路线.md:72`） | 臂撤掉：判官对「读阶段判完之后系统配置又读不出」那一拒只剩 U6 复算（某块盘一份都没有即真），补写闸过了还拒的坏实现被判合法；「照那一条的判据在崩溃镜像上自己复算成立」在 D23 带补写例外之后说的不是同一条判据 | 第二轮 W4 已攻，这里只记：撤臂让这一格从「有 G4 兜着」变成「没人兜」——丙-置 不在崩溃注入里（推的） | 同左 | 丙-I 在崩溃注入第一截照样判那张盘面，「没人兜」不成立，这一格只剩措辞问题（推的） |
| D23 第 396 行（`.claude/kb/decisions/23-journal的角色与格式.md:396`） | 「这一步在某块盘一份自证过的系统配置都没有、又过不了补写闸（D22 已定项 8）时也拒」 | 「自证过」一词两处口径不同：D23 这里（核心层）不认别的池的槽，I-8.5 与旁路报告认（Y1 理由 ③）。两句并排写成同一个词，在用过的盘重做 mkfs 的中途盘面上说的是两件事（推的；单盘那一格在这一轮 19 趟流里两读法 0 处不同，用过的盘那条流没重跑） | 同左 | 同左 |

## Y4：第二轮攻方的数，复跑与换路子核

### 复跑（量的，副本）

第二轮原型 `research/prompts/m3-close-g4-rewrite-r2-opus-model/` 原样拷进草稿目录，打它的 `toggles.patch`，在这一轮开工时拷出的冻结树上编（拷的时候七份快照文件 `sha256sum -c` 全 OK；冻结树 635 个文件的 sha256 清单在草稿目录 `frozen-manifest.sha256`，清单本身的 sha256 是 bc0f146eb1a3b3b86e362dafeea03844207cd37882599aee6f5bfe19b5f492f6）。第二轮那棵冻结树已经删了，这一轮的冻结树就是今天主工作区的 `crates/`；第二轮快照五份代码今天仍逐份 OK。18 条流逐条跑，每份计数与 `out/` 里 21 份同名文件（`-tally`、`-o2` 那三份比它们本来那条流）比：

    name=rerun_compare tag=cold_start_then_remount against=cold_start_then_remount stripped_same=yes byte_same=yes
    name=rerun_compare tag=hand_made_image_both_slots_and_a_unit-none against=hand_made_image_both_slots_and_a_unit-none stripped_same=yes byte_same=yes
    name=rerun_compare tag=hand_made_image_both_slots_and_a_unit-SINGLEFS_PROTO_GATE_NEVER_PASSES against=hand_made_image_both_slots_and_a_unit-SINGLEFS_PROTO_GATE_NEVER_PASSES stripped_same=yes byte_same=yes
    name=rerun_compare tag=hand_made_image_both_slots-none against=hand_made_image_both_slots-none stripped_same=yes byte_same=yes
    name=rerun_compare tag=hand_made_image_both_slots-SINGLEFS_PROTO_GATE_NEVER_PASSES against=hand_made_image_both_slots-SINGLEFS_PROTO_GATE_NEVER_PASSES stripped_same=yes byte_same=yes
    name=rerun_compare tag=remount against=remount stripped_same=yes byte_same=no
    name=rerun_compare tag=remount-SINGLEFS_PROTO_MUT_B1 against=remount-SINGLEFS_PROTO_MUT_B1 stripped_same=yes byte_same=no
    name=rerun_compare tag=remount-SINGLEFS_PROTO_MUT_B1-o2 against=remount-SINGLEFS_PROTO_MUT_B1 stripped_same=yes byte_same=yes
    name=rerun_compare tag=remount-SINGLEFS_PROTO_MUT_B1+SINGLEFS_PROTO_GATE_NEVER_PASSES against=remount-SINGLEFS_PROTO_MUT_B1+SINGLEFS_PROTO_GATE_NEVER_PASSES stripped_same=yes byte_same=no
    name=rerun_compare tag=remount-SINGLEFS_PROTO_MUT_B1-tally against=remount-SINGLEFS_PROTO_MUT_B1 stripped_same=yes byte_same=yes
    name=rerun_compare tag=remount-SINGLEFS_PROTO_MUT_B2 against=remount-SINGLEFS_PROTO_MUT_B2 stripped_same=yes byte_same=no
    name=rerun_compare tag=remount-SINGLEFS_PROTO_MUT_B2-o2 against=remount-SINGLEFS_PROTO_MUT_B2 stripped_same=yes byte_same=yes
    name=rerun_compare tag=second_crash_base_0 against=second_crash_base_0 stripped_same=yes byte_same=no
    name=rerun_compare tag=second_crash_base_1 against=second_crash_base_1 stripped_same=yes byte_same=yes
    name=rerun_compare tag=second_crash_base_2 against=second_crash_base_2 stripped_same=yes byte_same=no
    name=rerun_compare tag=second_crash_base_3 against=second_crash_base_3 stripped_same=yes byte_same=yes
    name=rerun_compare tag=second_crash_base_4 against=second_crash_base_4 stripped_same=yes byte_same=no
    name=rerun_compare tag=second_crash_base_5 against=second_crash_base_5 stripped_same=yes byte_same=yes
    name=rerun_compare tag=second_crash_base_6 against=second_crash_base_6 stripped_same=yes byte_same=no
    name=rerun_compare tag=second_crash_base_7 against=second_crash_base_7 stripped_same=yes byte_same=yes
    name=rerun_compare tag=zeros_mount against=zeros_mount stripped_same=yes byte_same=no

`stripped_same` 去掉 `history=`、`o2_reading`、`judged_tally=` 三种行再比；`byte_same=no` 的 9 份，差的行逐份分过类，只有这三种（第二轮 `out/` 里那几份出自更早几版原型，不打 `judged_tally=`，有的 `history=` 打法不同，MUT-B2 那份不打 `o2_reading`）：

    == remount
          1 < name=remount history=steps_run=
          1 > name=remount history=steps_run=
          1 > name=remount judged_tally=
    == remount-SINGLEFS_PROTO_MUT_B2
          1 > name=remount domain=torn
    == second_crash_base_0
          1 > name=second_crash_base_0 judged_tally=
    == zeros_mount
          1 < name=zeros_mount history=steps_run=
          1 > name=zeros_mount history=steps_run=
          1 > name=zeros_mount judged_tally=

（MUT-B1、MUT-B1 加闸恒不过与 remount 同形；基线 2、4、6 与基线 0 同形。）由此第二轮那几个数今天逐个对上：W1 四组 172 366 态（六个基线各 20 519、两个各 24 626），零发现；MUT-B2 撕裂域单盘全废 16 态（盘 0、盘 1 各 8）、两盘全废 1 态；MUT-B1 整写域 8 态、撕裂域 34 态；MUT-B1 加闸恒不过 42 张拒绝盘面两条都报。

**打中第二轮自己的复跑脚本（量的）**：照第二轮 `tools/rerun.sh` 自己的比法（只去 `history=` 与 `o2_reading` 两种行）比这一轮的输出，18 个标签里 8 个打 `same=no`（remount、MUT-B1、MUT-B1 加闸恒不过、zeros_mount、基线 0、2、4、6），原因就是上面那一种：脚本用的是最后一版原型（打 `judged_tally=`），比的却是更早几版的 `out/`。照那份脚本整趟复跑，主 agent 会看到 8 个 `same=no`，而数其实全对上。这一轮模型目录里的 `tools/rerun.sh` 改成去三种行再比，并另比第二轮那几份。

### 换一条不共用代码的路子核（量的，第二条路是手算）

`tools/independent_count.py`（`python3 -I` 直接跑，不读盘、不调原型与枚举库）只拿 `crates/singlefs-checker-tier/src/crash.rs` 文件头写的枚举规则（每段各写取几态任意组合、去掉整段全持久、流末加一态）与可撕的三条认法（`crates/singlefs-checker-tier/src/crash.rs:2176`：「/// 3. 它罩住的范围里原来有东西：基镜像那一段有非零字节，或写表里更早有一次同一块盘上、带字节（不是整段清零或 discard）的写与它重叠。」），加上每个系统配置槽写落在哪块盘哪一槽、写新槽还是全 0，手算状态数、撕裂态数与单盘全废态数：

    name=independent_count flow=remount states=20519 torn_states=20
    name=independent_count flow=remount-SINGLEFS_PROTO_MUT_B1 states=20582 torn_states=67
    name=independent_count flow=remount-SINGLEFS_PROTO_MUT_B1 domain=torn dead=[0] generation_discriminator_fires count=34
    name=independent_count flow=remount-SINGLEFS_PROTO_MUT_B1 domain=whole dead=[0] generation_discriminator_fires count=8
    name=independent_count flow=remount-SINGLEFS_PROTO_MUT_B2 states=20583 torn_states=75
    name=independent_count flow=remount-SINGLEFS_PROTO_MUT_B2 domain=torn dead=[0] generation_discriminator_fires count=8
    name=independent_count flow=remount-SINGLEFS_PROTO_MUT_B2 domain=torn dead=[0, 1] generation_discriminator_fires count=1
    name=independent_count flow=remount-SINGLEFS_PROTO_MUT_B2 domain=torn dead=[1] generation_discriminator_fires count=8
    name=independent_count flow=second_crash_bases states_each=[20519, 24626] total=172366

六个数（20 519 / 20、20 582 / 67 与 8 + 34、20 583 / 75 与 8 + 8 + 1、172 366）逐个与原型相同。手算里有一处是先验想错、被原型的数逼出来的：MUT-B1 的 8 / 34 分法要「前缀也在同一开关下跑过，所以盘 0 基镜像上已有一槽全 0，第一段那一写罩住的全是 0、不可撕」这一条才对得上（不加这一条手算出 6 / 36，总数同为 42）。这一条是按 `crates/singlefs-checker-tier/src/crash.rs:2176` 那条规则推出来、再用原型的总撕裂态数 67 对上的，不是另量的。

四句（这一格是核数，不判臂）：不分辨臂；不涉及被判系统看得到什么；对的是 r3 正文 Y4 那一格；改法不适用。

推翻条件：在入库装置（checker 档的崩溃点重放用例）上跑同一形状的流，状态数或单盘全废态数与上面不同。

## 自己提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 采丙-置 时，每条枚举流报自己的来历（mkfs 返回之后 / mkfs 自己 / mkfs 中途崩溃之后的挂载），观察者照来历判，不按段认 | Y2 第 3 节（补写流按段认误报 3 态） | 量过：原型里 `c_place_literal` 在 mkfs 流、两条补写流、mkfs 之后各流上逐态判对；接法没实现，推的 |
| 不采丙-I；若一定要在池级 checker 里判，承认它判不了「mkfs 返回之后」，用世代号判别子并在条文里写明它漏的那一形（mkfs 之后第一次取号那两段的撕裂态） | Y2 第 2 节 | 量过：漏 4 / 6 态、误报 1 态（两路一致）；条文写法推的 |
| 撤留臂与候选一起定：采丙-置 留臂（随机历史里只有它），采丙-I 撤臂（第 2406、2408 行改指池级 checker，第 2407 行删或换） | Y2 第 5 节 | 推的（重报 42/42 是第二轮量的、这一轮复跑） |
| I-8.5 一行与 `image.rs` 两处注释、两条测试的文档注释、欠账表 C461 一行里的「可写挂载会拒」改成「带读故障时可能判不出而拒；没有读故障时挂得上，取号写两轮把坏的那一槽补回」；`devices_whose_writable_mount_would_be_refused` 改名 | Y3-① | 量过：没有读故障时这类盘 0 次被拒（正确实现 250 态 + 两张手造盘面）；措辞推的 |
| 不做丙-择 | Y2 第 5 节 | 量过（两盘全废态今天红两处） |
| 第二轮原型的复跑比对去掉 `judged_tally=` 行（这一轮 `tools/rerun.sh` 已这样比） | Y4 | 量过（照第二轮脚本比 8 个 `same=no`，三种行去掉之后 18 个全对上） |

## 没打中的形状

- 正确实现在 mkfs 返回之后留下单盘全废（Y2 第 1 节）：这一轮另取 8 段历史、168 947 态（撕裂 250 态），0 态；连同第二轮那一次，两次抽样都没打中。用户动作放开到：mkfs 之后可写挂载一次 / 两次、读故障那一步、可写挂载后读故障那一步、零单元发布；第一个文件之后可写挂载后回退、再挂一次、读故障那一步。试了而没写盘的：第一个文件之后直接回退、可写挂载后抬 F、可写挂载后崩溃恢复抛弃最新根、第一个文件之后零单元发布、新建 inode（各 1 态，没有写）。
- 两个坏实现在第一个文件之后的历史上躲过世代号判别子：`aff_remount_twice` 加 MUT-B1（90 态单盘全废）、加 MUT-B2（32 态单盘全废）全部被判别子报出——漏报只出在 mkfs 之后第一次取号那两段。
- 「判槽自证读池级 checker」在单盘那一格与判官读法不同：这一轮 19 趟流 0 处（只在两盘全废、判官择不到系统配置那几态上不同，那几态今天已红）。
- 丙-置 在崩溃点重放里与可写挂载观察者重报：可写挂载观察者挂上了就不记，补写闸不过时观察者 panic、整条流红，丙-置 再记一条也只是同一趟里多一行（推的，没造这种流）。
- 取样范围：两块盘、单元区 384 槽的小几何、历史种子 0；三块盘、别的几何与种子没取；覆盖写、删除、清扫几条流各超过约 10⁶ 态，没跑。

## 这条腿自己的限度

- 只抽了一次：这一轮的「没打中」（Y2 第 1 节）算第二次抽样（第一次是第二轮 W1），两次的历史不重；丙-I、丙-置 各格的「打中」是线索，要主 agent 现查坐实。
- 丙-置、丙-I 都是我在原型里照正文描述另写的谓词（`classify_the_candidates`），不是将来的实现；丙-I 的世代号判别子是我挑的、我能找到的最好的盘面量，别的盘面量没逐个试（例如实例代号：取号同一写写进去，推的结论同世代号）。
- 坏实现只有两个（MUT-B1、MUT-B2，第二轮造的，补丁与第二轮逐字节相同），都不在 `crates/mutations.tsv` 里；三块盘的池、只坏一块盘的写序错没造。
- 原型的二次崩溃那几条流（`second_crash_*`、`rewrite_after_*`、`second_crash_on_a_hand_made_dead_base`）不带版本表，崩溃点重放自己的判在这些流上不能用（`second_crash_on_a_hand_made_dead_base` 与第二轮八个基线全红在 `file_read_where_this_root_has_none`）；观察者那一路的计数不受影响。
- 读码的几格（给定状态表没有观察者、判定库单元键动多少、撤臂丢什么）没造流量过。
- 取样：两块盘、单元区 384 槽的小几何、种子 0；覆盖写（1 048 588 态）、删除（5 242 903 态）几条大流超过约 10⁶，没跑。这条腿逐态枚举过的状态合计：这一轮原型 256 633 + 后加的二次崩溃 20 519 + 第二轮原型复跑 295 670 = 572 822，另有复跑脚本整趟自检一遍（同样的量），合计约 1.15 × 10⁶，在约 10⁷ 之内。
- 代码以快照为准：开工时主工作区七份快照文件与快照 sha256 逐份相同，我从主工作区拷了冻结树。跑到中途再核，`crates/singlefs-checker-tier/src/crash_injection.rs` 与 `crates/singlefs-checker/src/image.rs` 在主工作区里已被别的会话改过（`sha256sum -c` 两份 FAILED，`git status` 是 `MM`）；另有 `crates/singlefs-checker/src/walk.rs` 与 `.claude/kb/checks-owed.md`（快照 kb 清单里那一份，`sha256sum -c` FAILED）在开工之后被改过。`crates/` 下这几份的行号一律按冻结树（等于快照）取，主工作区此刻的行号已移位（例「按 draw 摆崩溃状态」那一行冻结树第 4775 行、主工作区第 4790 行）；欠账表第 732 行（C461）是我读的时候工作区里的样子，快照那一版的行号复原不了。核引文请带 `--root <冻结树>`。
- 跑法与负载：本机跑，每条命令经 `scripts/hosts/capped.sh 31`、`run-with-memory-cap.sh 16G` 加 `nice -n 19`；编译与流两次在内存包装里排队（共约 20 分钟，包装报「slice 已占 38–39 GiB / 40.1 GiB」，别的会话的 checker 档与变异在跑）。起重活前的资源提醒写「本机放得下、第二台放得下」；第二台要从带 `scripts/` 的树起（仓里 `scripts/` 7.4G），这一轮没走第二台。批与件：c1（4 件计数）、d1（8 件）、c2（18 件计数）、e1（4 件）、e2（5 件）、e3（2 件），每批数过 `.rc` 文件数与件数相同、退出码全 0；r2 复跑 b1–b4（3、4、4、6 件）同。超过 240 秒被工具挪到后台的四次（编译一次、`aff_remount_twice` 两坏实现一次、二次崩溃那一条连编译一次、复跑脚本整趟一次）都等完成通知或 `proc.py wait` 等到进程退出再读，没有写轮询输出文件的循环。

## 要主 agent 跑的命令

1. 整趟复跑（判决要引这条腿的数时）：工作目录随意，`--frozen` 给一棵与快照逐份相同的树（这一轮的冻结树留在 `/home/fy5090/.local/share/singlefs/work/m3-close-g4-rewrite-r3-opus/frozen`，没删），`--work` 给一个新目录：

       nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/m3-close-g4-rewrite-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-g4-rewrite-r3-opus/frozen --work <新的草稿目录>

   内存上限 16G、线程 31（脚本里经 `capped.sh 31`）。看输出里的 `name=rerun_compare` 行：`how=byte` 的 18 + 20 行、`how=stripped` 的 18 行（仓里还有第二轮模型目录时）、`independent_count` 1 行，全 `same=yes` 算核上；`name=rerun_batch … rc=` 全 0。
2. 引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-g4-rewrite-r3-opus-output.md --root /home/fy5090/.local/share/singlefs/work/m3-close-g4-rewrite-r3-opus/frozen --root /home/fy5090/code/singlefs`（`crates/` 下的引文按冻结树判，别的按仓根判）。

## 没做什么

- 没判正推、辩方那几格，没读禁读清单里的三样，没替主 agent 采纳任何改法；条文候选是靶子，不是推荐。
- 没跑 checker 档包的测试、崩溃点重放入口、崩溃注入、门禁；原型只调 checker 档的枚举库函数，编出来的二进制直接执行。
- 没改 `crates/`、`crates/mutations.tsv` 与 kb；副本上的数不算入库装置上的数，要引得先在入库装置上重做。
- 不重攻的角度：mkfs 末段本身（mkfs 自己的 6 态只拿来量丙-I 的判别子）、C670 与 Q3 拒因位置（只引第二轮量的 `other_conditions=0`）、甲-二 的开关。
- 第二台没用；三块盘、别的几何没取；超过约 10⁶ 态的流没跑。
