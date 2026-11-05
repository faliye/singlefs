# m3-close-b19-code-r3 正推腿（Sonnet）报告（2026-10-30）

被判：批 19 验证段代码轮第三轮（最终轮）V1–V5。方案按 `crates/` 今天的实现来谈；代码引文用开工快照里的路径与行号（开工先核了 `research/prompts/m3-close-b19-code-r3-snapshot/crates-sha256.txt` 与 `kb-sha256.txt`：`sha256sum -c` 13 份全 OK，所以主工作区今天的行号就是快照的行号）。kb 的行号是我在 kb 文件里现查的。

## 〇、先看这几条

- V1：剔读坏账的认法对现有每个「记读坏」的调用点是精确的，不会剔多；但**剔少**：只撕一个系统配置槽的盘（不是补写盘，单故障承诺之内）上，同一个机理照旧盖住拒因（四形实测，第二节）。认法三个条件各自没有用例钉（三条恒真变异全部存活）。读错账不剔的取舍成立，理由要改写。
- V2：逐盘核在预演之前这一形，在本仓能造的两盘上站得住（探针复跑对上）；多盘（两块以上补写盘）本仓造不出（单元的两份只落在身份最小的两块盘上，闸一只有那两块盘能过），按读码答，没造。
- V3：与 C874 的补写闸条文、D22 的补写那一句没有矛盾；有一处精度：「闸三、四、五」的编号只在代码注释和判决里，kb 条文没有编号。
- V4：前两轮「条款没写」与改了行为的句子，今天一句都还没写进 kb（下面七个关键词在 checks-owed、D16、D18、D22、D23 五份里 `grep -c` 全 0）。比正文清单多出三类（第五节）。
- V5：第一轮攻方五行、第二轮辩方 U1 与 U6、T5 的第 366 次挂载都在今天的树上复现；U2 的修补后数对上，修补前数用一条恒不剔变异复现。

## 一、怎么跑的（复跑命令与文件）

- 本机排不上：`run-with-memory-cap.sh 1G` 排了 120 秒，打出「slice 已占 40.1 GiB，这一条要 1.0 GiB…总上限 40.1 GiB；账上在跑 2 条」，我停了自己那一条（量的）；`ps` 看到的是别的会话的 checker 档 cargo test 与变异覆盖编译，没有性能测量（qemu、vm-bench、e152、fio）在跑。改经第二台跑。
- 复跑命令（工作目录仓根；第二台那棵树就是仓根的拷贝，所以 `--repo .`、`--frozen .`；开跑时主工作区与快照 13 份同哈希）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=第三轮正推腿的复跑脚本 rerun.sh…; 范围=…; 为什么=…' bash scripts/hosts/multi-host-run.sh --peer --memory-cap 4G --label <新标签> --fetch work-r3/out -- bash research/prompts/m3-close-b19-code-r3-sonnet-model/tools/rerun.sh --repo . --frozen . --work work-r3
```

- 最后一趟（标签 `m3-close-b19-code-r3-sonnet-run3`）退 0，`peer-command` 用 62.9 秒（量的，输出行 `MULTI_HOST_RUN_STEP step=peer-command event=end`）。汇总行落 `research/prompts/m3-close-b19-code-r3-sonnet-model/out/rerun-summary-lines.txt`（14 行）。
- 没跑 `rerun.sh --selftest`：单独的 `--selftest` 被重型测试闸按脚本正文判（正文里有 cargo test），套内存包装又排不上，经第二台的驱动带 `--selftest` 同样被拒。`rerun.sh` 相对模板只改了 `rerun_steps` 与它前面的两个比对函数。
- 变异施加在第二台那棵树的拷贝里，跑完 `patch -R` 还原；还原后 `mount.rs` 前缀 `61275a92c211254c` 回到开头的值（输出行 `name=mutations_reverted`）。
- 每个文件的 sha256 见文末「模型目录文件」一节。

## 二、V1　剔读坏账的改法撑不撑得住

**结论**：认法不会剔多（别的块、别的盘的系统配置槽）；会**剔少**——单故障承诺之内的、只撕一个系统配置槽的盘（它不是补写盘）上，第二轮 U2 那个机理原样还在；认法的三个条件各自没有用例钉；读错账不剔的取舍成立，但「一定是新故障」只有一半对。

### 2.1 认法剔不剔多（读码，加探针旁证）

- 认法是 `mount.rs` 里这三个条件的合取：「devices_rewritten_by_the_acquisition.contains(&read_failing_self_verification.device)」（`crates/singlefs-core/src/mount.rs:5755`）、读长等于一个系统配置槽、偏移是 `slot_index * slot_spacing_in_bytes`；它在判两本账时被 `.find` 套上：「.find(|read_failing_self_verification| {」（`crates/singlefs-core/src/mount.rs:5709`）。
- 现有「读成而自证不过」的上报点只有四处（量的，`grep -rnE '\.note_that_the_bytes_read_fail_self_verification\(' crates/singlefs-core/src/*.rs` 命中 4 行）：`recovery.rs:296`（转发包装）、`recovery.rs:710`（按位置项读单元）、`recovery.rs:4930`（系统配置槽）、`transaction.rs:5239`（释放前读盘核）。读单元的长度是单元字节数：「UNIT_CLASS_INDEX_NODE => Some(usize::try_from(NODE_BYTES).expect("16384")),」（`crates/singlefs-core/src/recovery.rs:671`）与「Some(usize::try_from(DATA_UNIT_BYTES).expect("32768"))」（`crates/singlefs-core/src/recovery.rs:669`），没有 4096；所以「读长 4096 且偏移在 0 或槽距」今天只可能是系统配置槽那一处：「for offset in [0, slot_spacing_in_bytes] {」（`crates/singlefs-core/src/recovery.rs:4914`）。盘条件再把它限在补写盘上。读码上不会剔掉补写盘上别的块、别的盘的槽。
- 槽距用的是写入口的参数：「let slot_spacing_in_bytes = u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);」（`crates/singlefs-core/src/mount.rs:5677`），读槽用的是择到的系统配置的参数（`mount.rs:6126`）；两者靠调用方参数与盘上一致那道核（`mount.rs:6859`–`6895`）相等，没有用例换槽距钉它（没查，读码）。
- 探针旁证：补写盘（盘 1 两槽写 0）加树表每次读坏，拒因点名树表（偏移 25067520、长 16384），不点名盘 1 的槽（`out/v1-torn-single-slot.txt` 的 `control_…` 行；本批用例 `a_tree_table_reading_back_garbage_every_time_is_named_by_the_rehearsal_refusal_instead_of_the_slots_of_a_device_rewritten_by_the_acquisition` 同样 ok，`out/v5-new-tests-of-the-batch.txt`）。注意控制实验里点名的是盘 0 的那份树表（故障读两块盘上同偏移的两份、盘 0 的先读），没有隔出「补写盘自己的非槽块不被剔」；那一条只靠读长条件，没有探针。

### 2.2 剔少：只撕一个槽的盘（新，零轮）

- 机理：取号读见证每次读**每块盘**的两槽（`highest_system_configuration_instance` 一次、`self_verified_system_configurations_of_every_device` 屏障前后各一次：「let previous_instance = highest_system_configuration_instance(pool);」（`crates/singlefs-core/src/transaction.rs:1206`）），读成而自证不过的槽一律记读坏；只有补写盘的被剔。一块盘只撕了一槽（读得出、自证不过，另一槽自证过）逐盘核判它「有自证槽」、不交补写，但取号读它那一槽照样记读坏。
- 探针（`tests/r3_probe_v1_a_torn_single_slot_on_a_device_the_acquisition_does_not_rewrite.rs`，窄盘镜像 `narrow_image()`，树表每一次读都读回坏字节，同第二轮辩方 U2 的施加法）：

| 镜像 | 不带读故障可写挂载 | 树表读坏时的拒因（量的，`out/v1-torn-single-slot.txt`） |
|---|---|---|
| 对照：盘 1 两槽写 0（补写盘） | 做成，`devices_rewritten=[DeviceIdentity(1)]` | 盘 0、偏移 25067520、长 16384（树表） |
| 盘 1 槽 0 写 0 | 做成，补写盘表为空 | 盘 1、偏移 0、长 4096（撕裂的槽） |
| 盘 1 槽 1 写 0 | 做成，补写盘表为空 | 盘 1、偏移 4096、长 4096 |
| 盘 0 槽 0 写 0 | 做成，补写盘表为空 | 盘 0、偏移 0、长 4096 |
| 盘 0 槽 1 写 0 | 做成，补写盘表为空 | 盘 0、偏移 4096、长 4096 |

  四形都 `writes_reaching_the_disk=0`（取号之前拒）。这与 U2（`research/prompts/m3-close-b19-code-r2-main-verification.md:19`「拒因报成盘 1 槽 0」）是同一个缺陷，只是撕裂槽属于不被剔的盘。
- 第二个后果（读码推的，没造）：预演停在非读类错上（真做会照样报它）而读坏账里只有这一条撕裂槽时，补写盘的同形走「(None, None, _) => return Ok(())」（`crates/singlefs-core/src/mount.rs:5724`）留给续做；撕裂槽的同形账不空，重走两趟（「const CONTINUATION_REHEARSALS_BEFORE_ACQUISITION_AT_MOST: usize = 2;」，`crates/singlefs-core/src/mount.rs:5626`）后在取号之前拒成「预演读坏」。同样的机理，两种盘面一个放行一个拒。
- 候选（只列，不采；零轮）：认法去掉盘条件（取号会重写每块盘的两槽，任一盘的槽读坏在预演里都是预期的）。实测：恒真盘条件的变异（`patches/mutation-device-condition-always-true.patch`）下，上表四形全部改点名树表（`out/mutation-device-v1-probe.txt`），本批用例仍 3 个全 ok（`out/mutation-device-target.txt`）。是否采、是否改文字，由主 agent 判；我没判。
- 什么现象会推翻：在今天的树上造出单槽撕裂盘叠上树表读坏、而拒因点名的是树表。

### 2.3 认法的三个条件有没有用例钉

- 本批只有一条整体变异：`crates/mutations.tsv` 第 6259 行「C874 预演判两本账不剔补写盘系统配置槽的读坏」（`true || !` 套住整个认法）。
- 我对三个条件各做一条「恒真」变异（`patches/mutation-device-condition-always-true.patch`、`mutation-length-condition-always-true.patch`、`mutation-offset-condition-always-true.patch`），跑本批的认法用例整份（`writable_mount_rehearsal_takes_the_devices_rewritten_by_the_acquisition_from_the_check_of_every_device`，3 个用例）：三条都是 3 ok、0 FAILED（量的，`out/mutation-device-target.txt`、`mutation-length-target.txt`、`mutation-offset-target.txt`）。恒不剔变异（`patches/mutation-filter-never-applies.patch`，第 6259 行同一形）红 1 个：`a_tree_table_reading_back_garbage_every_time_is_named_by_the_rehearsal_refusal_…` 红在 `…_from_the_check_of_every_device.rs:173` 的 `assert_eq!`（量的，`out/mutation-filter-target.txt`）。
- 所以认法放宽成任意盘、任意读长或任意偏移，本批用例都不红；要钉住需要一条「补写盘上别的读坏（读长不是 4096，或偏移不是槽）不被剔」的用例，和（如果采 2.2 的候选）「非补写盘的槽读坏也被剔」的用例。
- 什么现象会推翻：三条变异里任何一条让本批用例红。

### 2.4 读错账不剔

- 前半句成立：逐盘核对一块盘的两槽，「if slots_reporting_a_read_error > 0 {」（`crates/singlefs-core/src/mount.rs:6135`）这一支在没有自证槽时就拒，所以进补写盘表的盘，核那一刻两槽都读得出（读码）；核读盘走「ReadRouteOfThisMount::StraightToTheDisk => return self.disk.read_at(offset, buffer),」（`crates/singlefs-core/src/mounted_session.rs:1015`），不记账，预演每一趟开头还清账（「reads_failing_self_verification.borrow_mut().clear();」，`crates/singlefs-core/src/mount.rs:5682`；读错账同样清）。所以预演里这两槽的读错，不是核那一刻已有的状态，是之后发生的事——说它「新」站得住。
- 后半句「新故障所以要点名」只是策略，不是必然：读错落在补写盘的槽上，取号并不因此停下——见证判不出要「有槽读错且没有一块盘两槽都自证过」，另一块盘两槽都好时判得出，补写盘空表项放行（`crates/singlefs-core/src/transaction.rs:957` 那一支）。读错账里的第一条读错会像撕裂槽一样盖住后面真出故障的那一读。区别在于读坏在补写盘槽上是**由盘面确定会发生**的（逐盘核判过它们两槽自证不过），读错不是。把「读错账不剔」写成「一个读错的槽可能是更新的见证，不能当没发生」（`mount.rs:6133` 起的注释就是这个口径）比「一定是新故障」稳。
- 没探：单个读错叠上树表读坏要两个故障规格，`mount_writable_on_recording_devices` 只收一个，我没改共用模块。读码结论，推的。
- 什么现象会推翻：核通过之后、补写盘的槽上出现的读错，在同一镜像上被证明是确定性的（核那一刻也会读错）。

## 三、V2　逐盘核在预演之前这一形撑不撑得住

**结论**：在本仓能造的两盘镜像上站得住，探针复跑对上；多盘（两块以上补写盘）本仓造不出，按读码答；条文要写准「先于续做预演」，不是「先于一切预演」。

- 次序（读码，行号都是 `crates/singlefs-core/src/mount.rs`）：准入「let admission_before_acquisition = admission_of_the_writable_mount_before_acquisition(」（`mount.rs:7120`）→ 写行与暖机的预演「} = publishes_after_acquisition_rehearsed(」（`mount.rs:7131`）→ 逐盘核「devices_rewritten_by_the_acquisition_after_checking_each_device(」（`mount.rs:7146`）→ 续做预演「refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range(」（`mount.rs:7165`）。注释写的是「逐盘核排在续做预演之前」（`mount.rs:7142`），与代码一致；前两步的拒（`RowPublishAdmissionRefusedBeforeAcquisition` 一族）仍先于逐盘核那一拒，这一点第一轮修补之前就是这样，没变。
- 逐盘核读盘不进预演的两本账：核整圈包在「} = with_reads_straight_to_the_disk(read_route, || {」（`mount.rs:7407`）里，那一路读错、读坏都不记（`mounted_session.rs:1015`），预演每趟开头清两本账（`mount.rs:5681`–`5682`）。
- 两种拒同时成立：第二轮辩方 U1 探针在今天的树上与留存输出逐行相同（`SAME name=v5-r2-u1 lines=5`，`out/v5-r2-u1.txt`，三条：C841 镜像不动 → C841；加盘 1 两槽写 0 → 仍 C841；加盘 1 整块写 0 → 报逐盘核那一拒）；本批用例 `a_whole_zeroed_device_on_an_image_whose_continuation_opens_a_container_past_the_top_of_its_range_is_refused_by_the_check_of_every_device`（`…_from_the_check_of_every_device.rs:81`）ok（`out/v5-new-tests-of-the-batch.txt`）。对应的变异行是 `crates/mutations.tsv:6260`「C874 逐盘核放回续做预演之后」，我没重跑它（第二轮判决 `m3-close-b19-code-r2-main-verification.md:18` 记了辩方量过旧次序下红）。
- 多盘怎么走（读码）：补写盘表是 `Vec`，认法与取号读都用 `contains`；逐盘那一圈遇到第一块失败的就整个返回，所以多块盘同时有问题时报盘表次序第一块、它第一件没过的（注释：「只报盘表次序第一块这样的盘、它第一件没过的」，`mount.rs:375`）。预演与真做用同一份表：「devices_rewritten_by_the_acquisition: devices_rewritten_by_the_acquisition.clone(),」（`mount.rs:7159`）装进计划，真做取号读与预演取号读都按它放行。
- 两块以上补写盘本仓造不出：一个单元的两份位置项固定放在身份最小的两块盘上——「device: sorted[0],」（`crates/singlefs-core/src/make_filesystem.rs:379`）、「device: sorted[1],」（`make_filesystem.rs:384`）——而补写闸一要求这块盘带所选那一版的全部单元（D22 的 185 行：「这块盘带所选那一版的全部单元」，`.claude/kb/decisions/22-单元原子性怎么合成.md:185`）。补写盘因此只可能是身份最小的那两块；两块都补写还得有第三块盘供系统配置。harness 的窄盘镜像只有两盘：「[DeviceIdentity(0), DeviceIdentity(1)]」（`crates/singlefs-harness/tests/common_admission/mod.rs:108`）。我没去造三盘池（没查三盘 mkfs 的参数），所以「两块补写盘」「补写盘与读坏盘不同、盘数多于两块」这两形没量。两盘池里补写盘至多一块（另一块要供系统配置），「补写盘与读坏盘不同」的两盘形就是 2.2 的对照行：补写盘是盘 1，被点名的树表读坏落在盘 0。
- 精度：写进条文时写成「在续做预演之前，在准入与写行、暖机预演之后」；只写「先于预演」会让人以为准入的拒也排在逐盘核之后。落点见第五节第 4 行。
- 什么现象会推翻：造出一个盘面，逐盘核与续做预演的拒同时成立而报的是预演那一拒（v5-r2-u1 的第三条会红）；或在三盘池里补写盘表含两块盘时，预演的取号读把其中一块当成「一份自证过的都没有」而停住。

## 四、V3　C929 照今天不判

**结论**：与 C874 已定的五件闸、D22 的补写那一句没有矛盾；有两处精度要在写回的那一句里带上。读码加本批回卷用例，没另造盘面。

- D22 的补写条款以「两槽全废」为前提：「某块盘两槽全废（一份本池自证过的系统配置都没有）时，取号那一步补写它的两槽」（`.claude/kb/decisions/22-单元原子性怎么合成.md:185`），失败处置是「补写失败照取号每块盘回卷一次」（同一行）。回卷写写的是旧号与取号那一刻的见证值：「回卷写带的 tail 是取号那一写带的同一个见证值」（`crates/singlefs-core/src/transaction.rs:711`），另一槽留着新号：「另一槽留着新号（下一次取号跳过它）」（`crates/singlefs-core/src/transaction.rs:710`）。回卷之后盘 1 两槽都自证得过，不再满足「两槽全废」这个前提，条文的五件闸没有被违反——它们只管「全废」那一支。
- 与 C874 的文字：「补写闸五件判官不复算，明写成不判」（`.claude/kb/checks-owed.md:540`）说的是崩溃测试的观察者不复算闸，与回卷之后下一次挂载走哪一支无关；没有矛盾。
- 与落后支（D18 的逐盘核一行，`.claude/kb/decisions/18-块里携带什么信息.md:322`）：有自证槽的盘走落后支，落后支只在「(实例代号, journal tail) 小于所选那一版末条记录的 jsn」时才看单元与根槽那一半。回卷后盘 1 的最新槽带取号那一刻的见证值，通常不落后，于是**连闸一（带全部单元）与闸二（根槽那一半）也不再判**——正文说「补写闸三、四、五不再判」低估了。闸三（更新的根或记录）、四（单元区更新写序）、五（环槽读错）落后支本来就不判。写进条款的那一句要按内容写，不按编号写：补写闸的编号（闸一、闸五、闸三、闸四、闸二的判定次序）只在代码注释里（`crates/singlefs-core/src/mount.rs:375`），D22 的 185 行没有编号。
- 本批回卷用例钉了这一支：下一次挂载做成、取号 3、补写盘表为空——「devices_rewritten_by_the_next_mount.is_empty(),」（`crates/singlefs-harness/tests/acquisition_rewriting_a_device_without_a_self_verified_system_configuration_rolls_back_on_a_write_failure.rs:230`），盘 1 回卷后两槽是 (世代号 1, 号 2) 与 (世代号 2, 号 1)（同文件 `:218` 的 `(2, INSTANCE_BEFORE_THE_ACQUISITION)`）。今天整份 ok（`out/v5-new-tests-of-the-batch.txt`：`second_round_write_failing_…` ok、`first_round_write_failing_…` ok）。变异行 `crates/mutations.tsv:6261` 在，没重跑。
- 候选一「照今天不判」会错的那一形：回卷之后有人把盘 1 的单元区改了。这与「一块有自证槽的普通成员盘事后被改」同形，落后支（或根本不落后时）本来也不判；第二轮判决说的「同形、不是回卷独有」（`m3-close-b19-code-r2-main-verification.md:22`）读码上成立。我没造这个盘面。
- 回卷用例没覆盖的一个邻格：第一轮补写那一写就失败（`first_round_write_failing_…`）时盘 1 仍两槽全废、下一次挂载仍过闸补写，这一格已有用例钉，与候选一不冲突。
- 什么现象会推翻：造出回卷之后下一次挂载放行了一块**本该由补写闸一或二拒**的盘，并且同样的改动对一块普通成员盘会被落后支拒（那就说明回卷盘被区别对待了）。

## 五、V4　未定的清单完不完整

**结论**：正文那几句列得对，但不全：多出三类（表里第 8、9、11 行），另有两处落点我认为判决点名的不是最合适的那份（第 5 行）。今天 kb 里一句都还没写：下面七个关键词在 checks-owed、D16、D18、D22、D23 五份里 `grep -c` 全 0——`not_rewritten_because`、`逐盘核.*续做预演`、`只报盘表次序第一块`、`区域在前、槽号在后`、`槽号 0 的钉值`、`解冻`、`读坏账`（量的）。每份 kb 另有 `.en.md`、`.ja.md` 两份译文（`ls .claude/kb/decisions | grep '^22\|^18'` 列出三份），写回要三语同插。

「落点」是我在 kb 文件里现查的行；「今天有没有」是对着开工快照里的 kb（与今天的 kb 同哈希）查的。

| # | 要写的句子 | 来源 | 落点（kb 文件:行） | 今天有没有 |
|---|---|---|---|---|
| 1 | 钉槽 p 存的是槽编码 3·槽+区域，不是钉值；三个区域的槽 0 钉值都取第 0 代根的 txg 0；平局按（钉值, 区域, 槽号）取小。代码：「(pin.pinned_txg, pin.ring_slot));」（`crates/singlefs-core/src/allocator.rs:2793`）、「fn oldest_txg_a_ring_slot_bad_at_mount_can_have_held(」（`crates/singlefs-core/src/mount.rs:2993`） | 第一轮判决 `m3-close-b19-code-r1-main-verification.md:19` | `.claude/kb/decisions/16-发布语义.md:39`（「根槽这一次读坏」表行）与 `:96`（「钉也落盘」一句）；`.claude/kb/checks-owed.md:481`（C773 行） | 没有 |
| 2 | 准入抬 F 上限里同名 `Recovery(NoValidRoot)` 在本进程准入路径走不到，两条前提：(a) 这个进程写出的每条现行根都经记根进根环表（破了断言失败，不往上交）；(b) 现行根带的 F 不高于它自己的 txg；隐含前提 (c)：现行根不被它自己的实例表判成被抛弃（正推腿提出）。调用处三个：真做写行之后与续做里推、取号之前只进内存那一趟预演、挂着的会话发布被拒时推。公开入口 `raise_rollback_floor_to_the_admission_ceiling`（`mount.rs:4478`）与只供测试的 `raise_rollback_floor`（`mount.rs:4239`）交回 `Err(NoValidRoot)`，不 panic | 第一轮 `:20`；第二轮 `m3-close-b19-code-r2-main-verification.md:20` | `.claude/kb/decisions/23-journal的角色与格式.md:455`（「重建分配器读根环两遍并起来没有有效根的拒因」那条）与 `:391`（「在任何写之前拒」单子行）；`.claude/kb/checks-owed.md:439`（C724 行，它的「仍欠」要改成已写明走不到） | 没有（`C724` 在 D23 里 0 命中）。**代码里已经在引**：「（D23（journal 的角色与格式） 已定项 14 射程两条前提之二，C724）」（`mount.rs:4415`）、「…两条前提之一，C724」（`mount.rs:4678`）——kb 里还没有这句话，是悬空引用 |
| 3 | 冻结解除（重发成功）后积下的 discard，下一次进三个 discard 调用点（`mounted_session.rs:2136`、`:2307`、`:2498`）就发、不丢；重发先失败再成功也不丢（第二轮辩方量）；冻结着时 `discard_the_segments_reusable_as_a_whole` 与整理入口都先判 `frozen_publish()`（`mounted_session.rs:2931`、`:2798`） | 第一轮 `:21`；第二轮 `:21` | `.claude/kb/checks-owed.md:591`（C928 行，它的「今天推的会红」「改法候选」已过时）；`.claude/kb/decisions/23-journal的角色与格式.md:409`（「这一版的失败处置」冻结那一段） | 没有（`解冻` 0 命中） |
| 4 | C874 取法与字段名：`MountOutput::devices_rewritten_by_the_acquisition`（`mount.rs:1080`）；没补写的原因装在拒因载荷里：`DeviceNotRewrittenByTheAcquisition`（`mount.rs:6071`）、`not_rewritten_because`（`mount.rs:379`）；只报盘表次序第一块、它第一件没过的（`mount.rs:375` 注释），管理员修好那一件再挂可能撞上同一块盘的下一件或下一块盘 | 第一轮 `:22` | `.claude/kb/checks-owed.md:540`（C874 行）与 `.claude/kb/decisions/22-单元原子性怎么合成.md:185` | 没有（C874 行里 `devices_rewritten_by_the_acquisition` 0 次） |
| 5 | 逐盘核的位置：在准入与写行、暖机预演之后，续做预演之前；两种拒同时成立时报逐盘核那一拒 | 第二轮 `:18` | 判决点名「D22 那一行与 C874」。D22 的 185 行讲的是补写条件，不讲取号之前的顺序；**建议主落点放 `.claude/kb/decisions/18-块里携带什么信息.md:321`（「可写挂载的顺序」，它已列预演，没列逐盘核的位置）与 `:322`（逐盘核一条），D22:185 与 C874 行各留一句指路**（我的建议，不是判） | 没有 |
| 6 | C929：回卷之后下一次挂载按有自证槽的盘走落后支，补写盘表为空；补写闸的各件不再判，落后支只在盘落后时才看单元与根槽那一半；这一形对所有有自证槽的盘同形。要带精度：按内容写、不按编号写（第四节） | 第一轮 `:25`；第二轮 `:22` | `.claude/kb/checks-owed.md:592`（C929 行，「检查」栏的「先定条款」改成已定、用例已落）与 D22:185 补一句 | 没有 |
| 7 | C690：产品默认阈值 365（用户 2026-10-29）取代占位 369，每片仍装 369 行；越过 365 而不清扫有三形：现行那一版树表 0 条（「NoRowDeletedThisRound::SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion,」，`instance_table_sweep.rs:550`）、清扫这一轮一行不删（`NoRowDeletedThisRound`，`:270`）、开关交给后台清扫；一次挂载写几行也越过。第二轮辩方量的：420 次挂载，第 366 次首扫，扫后 2 行（第七节复现） | 第一轮 `:23`；第二轮 `:23` | `.claude/kb/checks-owed.md:360`（C690 行）与 `.claude/kb/decisions/18-块里携带什么信息.md:318`（「阈值是配置项，默认值另定」）；E183 页 H5 一条 | 没有：C690 行仍写「阈值是占位值 369」（`checks-owed.md:360`） |
| 8 | **不在正文清单里**：取号之前预演判两本账时，读坏账里读补写盘系统配置槽的那几条不算进去，读错账不剔（第二轮修补改的行为，`mount.rs:5709`）。这一条所在的整套机制（读错账与读坏账两本账、读坏的块从共用那一份里丢掉、重走至多两趟、只有 C841 不看账就拒）在 kb 里没有任何一句：唯一出处是已归档的岔路单，代码文档还在引它：「岔路单 `research/prompts/m3-b44-code-r3-forks.md` 岔路二「没走完就重走」那一行，用户定」（`mount.rs:5623`），该文件已被 59ce5a865 删（`git show 59ce5a865^:research/prompts/m3-b44-code-r3-forks.md`） | 第二轮 `:19`；record 里「读错账要不要一起剔并进第三轮 V1」 | 建议 `.claude/kb/decisions/18-块里携带什么信息.md:321`（预演那一句）与 `.claude/kb/checks-owed.md:526`（C858 行，它的欠账正是「预演里的读错、读坏不在取号之前报」） | 没有（`读坏账` 0 命中） |
| 9 | **不在正文清单里**：第二轮判决 P4 的「差出来那一形要一份坏根记录，整张已红，不算 Q9 的漏」写进 C937 射程 | 第二轮 `:25` | `.claude/kb/checks-owed.md:600`（C937 行） | 没有 |
| 10 | C939（整理扣下的段取走不放回）登一笔 | 第二轮 `:21` | `.claude/kb/checks-owed.md:602` | **已登** |
| 11 | **本腿新增（V1）**：撕裂单槽盘上拒因被撕裂槽盖住；认法三个条件无用例钉。产品代码不动的话要登一笔欠账（下一个未用号：`grep -c '^\| C940 '` 为 0），动的话记进 C874 行「检查」栏 | 第二节 | `.claude/kb/checks-owed.md`（新行）或 `:540` | 没有 |

- 已落、不必再写的两处代码之外的改动（量的）：`grep -c PLACEHOLDER_UNTIL_C690 crates/mutations.tsv` 为 0，`grep -rn PLACEHOLDER_UNTIL_C690 crates --include=*.rs` 零命中（第一轮判决要改的第 2266 行已改指新常量）。
- 什么现象会推翻这张表的「没有」：对着今天的 kb 再 `grep` 上面七个关键词出现命中（说明有别的会话已写回）。

## 六、V5　前两轮的数能不能核

**结论**：挑的四个数里，三个原样复现，一个（U2 修补前的拒因）在今天的树上按设计已经变了，用一条恒不剔变异把它复现回来。比对口径：两边按行排序再逐行比（第一轮攻方的留存输出是当年的输出次序）；过滤掉耗时与路径。汇总行在 `research/prompts/m3-close-b19-code-r3-sonnet-model/out/rerun-summary-lines.txt`。

| 判决里的数 | 在哪份判决 | 今天的树上（量的） | 结果 |
|---|---|---|---|
| 第一轮攻方探针五行输出（带逐盘核挪位） | `m3-close-b19-code-r1-main-verification.md:12`「name=probe_output_with_the_reorder_identical lines=5」 | 排序后逐行相同，`SAME name=v5-r1-attack-probe lines=5`（五行见下） | 复现 |
| T5 的 420 次挂载、第 366 次触发 | `m3-close-b19-code-r2-main-verification.md:23`「辩方 T5 量 420 次挂载清扫第一次在第 366 行触发」 | `PROBE name=u6_first_sweep mount_number=366 last_not_swept_rows=365 rows_after_the_mount=2`；`u6_summary mounts=420 not_swept=419 rows_deleted=1 … max_rows_after_a_mount=365 mounts_with_rows_over_one_page=0`；`SAME name=v5-r2-u6 lines=4` | 复现 |
| 第二轮辩方 13 步（我核了其中 3 步：U1、U6、U2） | `m3-close-b19-code-r2-main-verification.md:11` | U1 `SAME name=v5-r2-u1 lines=5`、U6 `SAME name=v5-r2-u6 lines=4`、U2 见下一行；其余 10 步没跑（见下） | 核的 3 步复现，10 步没核 |
| U2 拒因前后的偏移与长度 | `m3-close-b19-code-r2-main-verification.md:19`「拒因报成盘 1 槽 0」 | 修补之后：盘 0、偏移 25067520、长 16384（树表）；`DIFF name=v5-r2-u2`，差异只有 `zeroed_garbage_every_time` 那一行（留存 `device=1 offset=0 length=4096`，今天 `device=0 offset=25067520 length=16384`）。用恒不剔变异（`patches/mutation-filter-never-applies.patch`，同 `crates/mutations.tsv:6259` 那一形）回到修补之前：`SAME name=v5-r2-u2-with-the-filter-off lines=6`，即盘 1、偏移 0、长 4096 | 修补后的数与修补员报告对上；修补前的数靠变异复现 |
| 修补员「两个改到的测试二进制整份退 0」 | 正文 `research/prompts/_m3-close-b19-code-r3-body.md` 第二节 | 本批认法用例整份 3 个 ok，回卷用例 2 个 ok，抬 F 上限公开入口用例 2 个 ok（`out/v5-new-tests-of-the-batch.txt` 的三行 `test result: ok`） | 复现 |

第一轮攻方五行（`out/v5-r1-attack-probe.txt`，原样）：

```
PROBE unchanged_image mounted instance=InstanceGeneration(2) devices_rewritten_by_the_acquisition=[] continuation_steps=16
PROBE zeroed_after_the_continuation mounted instance=InstanceGeneration(3) devices_rewritten_by_the_acquisition=[DeviceIdentity(1)] continuation_steps=0
PROBE zeroed_image_space_admission_judged devices_unchanged=false
PROBE zeroed_image_space_admission_judged mounted instance=InstanceGeneration(2) devices_rewritten_by_the_acquisition=[DeviceIdentity(1)] continuation_steps=16
PROBE zeroed_image_space_admission_skipped mounted instance=InstanceGeneration(2) devices_rewritten_by_the_acquisition=[DeviceIdentity(1)] continuation_steps=15
```

- 复现不了的：(1) 第一轮攻方探针**不带逐盘核挪位**的那份留存输出（`probe-output.txt`，拒成「预演读坏」那一版）在今天的树上复现不了——逐盘核已经挪到预演之前，那是修补前的形态；我没把 `fix-reorder-the-device-check-before-the-rehearsal.diff` 反向打回去再跑。(2) 第二轮辩方 13 步里我没核的 10 步：`old-u1`、`new-u4`、`new-u5`、`new-pinned-and-new-tests`、`old-pinned-and-t4`、`new-root-record-target`、`new-core-lib-root-record`、三条变异（`new-t1-mutation-6144`、`new-t2-mutation-6248`、`new-t2-mutation-6155`）。(3) 「第二台跑过」那一句的第二台与我跑的是同一台，不是独立的两台。
- 什么现象会推翻「复现」：同一份模型目录在同一棵树上再跑，汇总行里 `SAME` 变 `DIFF`；或 `mount.rs` 前缀不再是 `61275a92c211254c`（树变了）。

## 七、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| V1　认法剔多 | 一致 | 现有「读成而自证不过」的上报点只四处，读长 4096 加偏移 0 或槽距只可能是系统配置槽，再限在补写盘上；补写盘加树表读坏时拒因点名树表 |
| V1　认法剔少 | 冲突（新，零轮） | 只撕一个系统配置槽的盘（盘 1 或盘 0，槽 0 或槽 1）加树表读坏，拒因点名撕裂的槽，与 U2 同一缺陷；四形实测 |
| V1　认法三条件有没有用例钉 | 规则没说（无用例钉） | 盘、读长、偏移三条恒真变异本批用例全 3 ok；只有整体恒不剔变异红 1 个 |
| V1　读错账不剔 | 一致（理由要改写） | 核通过的盘两槽都读得出，之后的读错确是新事件；但「一定是新故障」不成立，该写成「读错的槽可能装着更新的见证，不能当没发生」；叠加形没探 |
| V2　两盘上逐盘核先于预演 | 一致 | U1 探针 `SAME`，本批用例 ok；核读盘不进两本账（读码） |
| V2　多盘（两块以上补写盘、补写盘与读坏盘不同） | 复核不了 | 单元两份只落身份最小的两块盘，两盘 harness 造不出两块补写盘；读码上表是 `Vec`、整圈遍历后报第一块 |
| V2　条文的先后措辞 | 规则没说 | 要写「续做预演之前、准入与写行暖机预演之后」，不是「先于预演」 |
| V3　C929 与 C874 五件闸、D22 补写那句 | 一致 | D22 补写条件以「两槽全废」起头，回卷后不满足；C874 的「不复算」说的是观察者；没有矛盾 |
| V3　「补写闸三、四、五不再判」的措辞 | 规则没说 | 回卷后连闸一、闸二也不再判（除非盘落后）；编号只在代码注释里，要按内容写 |
| V4　清单完整性 | 冲突 | 正文清单少三类（第 8、9、11 行）；今天 kb 里七个关键词全 0 命中；C939 已登 |
| V4　逐盘核先后的落点 | 规则没说 | 判决点名 D22 与 C874；D22:185 不讲顺序，建议主落点放 D18 的 321、322 行 |
| V5　第一轮攻方五行 | 一致 | 排序后逐行相同 |
| V5　T5 的 420 次挂载第 366 次 | 一致 | `mount_number=366`、`mounts=420`、扫后 2 行 |
| V5　U2 拒因前后 | 一致 | 修补后盘 0、25067520、16384；修补前（恒不剔变异）盘 1、0、4096，与留存逐行相同 |
| V5　辩方 13 步其余 10 步 | 复核不了 | 没跑 |

## 八、要主 agent 跑的命令

- 判决要引复跑数：把 `research/prompts/m3-close-b19-code-r3-sonnet-model/` 拷到草稿目录，在副本里经 `scripts/hosts/multi-host-run.sh --peer`（本机排不上时）跑 `tools/rerun.sh --repo . --frozen . --work <新目录>`，命令形态见第一节；跑完看汇总行与 `out/rerun-summary-lines.txt` 逐行比。
- 我没起重型测试，也没起轻型三条。

## 九、没做什么

- 没判别的腿的格，没替主 agent 采纳或出判决；2.2 的候选只列。
- 没读禁读清单里的文件（攻方第三轮报告与模型、主 agent 第三轮核查记录）。
- 没造三盘池（V2 多盘）；没造「撕裂槽加预演停在非读类错上」（V1 第二个后果）；没造读错账叠加形；没核 U3 的第三个调用处在预演里的 panic 前提（读码上成立，推的）。
- 没跑 `rerun.sh --selftest`、没跑第二轮辩方 13 步里的 10 步、没重跑 `crates/mutations.tsv` 6260 与 6261 两行。
- 没改任何 `crates/`、kb、规则；产出只在报告与模型目录。
- 更正我自己前面的两处措辞：2.4 里「一定是新故障」只有前半成立（已在 2.4 说明）；2.1 的「读码上不会剔掉…」对「补写盘自己的非槽块」只靠读长条件，没有探针隔开（已在 2.1 说明）。
- 清理（量的，`du -sh`）：删了我的冻结副本 44M、变异副本目录 3.6M、驱动取回目录两个共 264K；第二台那棵树与编译目录由驱动删（`step=remove-tree … exit=0`）。草稿目录 `~/.local/share/singlefs/work/m3-close-b19-code-r3-sonnet/` 里只剩 6 个小文本（检查输出与第二台驱动的原始输出），不含仓副本。

## 十、模型目录文件（sha256）

`research/prompts/m3-close-b19-code-r3-sonnet-model/` 下 36 个文件（`for f in $(find . -type f | LC_ALL=C sort); do sha256sum "$f"; done`，在模型目录里跑）：

```
c3061d75458538992f67922b5d1034de7eddd6515b72e1e4d2323e042b1017a7  expected/r1-probe-output-with-the-reorder.txt
270df18b5fb9b24e4bf0989238da065949a96fe6f62f0406c6c649c71d60c9b4  expected/r2-new-u1.txt
dbb03c28fa12c81ed2b1d1dfd157486160b0c2cbfa1d59ebcc0bf8fdea2658b9  expected/r2-new-u2.txt
54f53506c2748750d4441a5c91810ed3ba834398eca944a5e91122665e04682d  expected/r2-new-u6.txt
33b8c6452fe2a120daa4db8491d55559148edf8e5cb9f3e558f9cb2e32086bd0  out/mutation-device-apply.txt
e9ac0c2d1c56cc6cab871c5a0c269f7ce138feaf8cfcc260875e4983483a40e9  out/mutation-device-target.txt
53d196acb5650b5453f596717bacef0e6e4d39b909ce4a4aac29b1af984dc94d  out/mutation-device-v1-probe.txt
33b8c6452fe2a120daa4db8491d55559148edf8e5cb9f3e558f9cb2e32086bd0  out/mutation-filter-apply.txt
e358bb9f397e5c909ab2fa07eba64c21560a56e8bd5e4f1fcb1a2861be0e7317  out/mutation-filter-target.txt
33b8c6452fe2a120daa4db8491d55559148edf8e5cb9f3e558f9cb2e32086bd0  out/mutation-length-apply.txt
e9ac0c2d1c56cc6cab871c5a0c269f7ce138feaf8cfcc260875e4983483a40e9  out/mutation-length-target.txt
33b8c6452fe2a120daa4db8491d55559148edf8e5cb9f3e558f9cb2e32086bd0  out/mutation-offset-apply.txt
e9ac0c2d1c56cc6cab871c5a0c269f7ce138feaf8cfcc260875e4983483a40e9  out/mutation-offset-target.txt
f14e7d68b467afba955064a36ba0f532badf17e6efd2ce40ab68d0353a476ed9  out/rerun-summary-lines.txt
afa479287ac6d097c4bb619f4f8926d23958ea9e4cffae1f212a305a7b254ca1  out/v1-torn-single-slot.txt
324baf6d224fdbfe2dbe645e1e688a8be82fef1be293f7b9cdf3746a138330a5  out/v5-new-tests-of-the-batch.txt
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  out/v5-r1-attack-probe.diff
e835c6a7394b53a79a91cab59833ecd59ee5ee58839013e852abcdfa8facd427  out/v5-r1-attack-probe.txt
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  out/v5-r2-u1.diff
270df18b5fb9b24e4bf0989238da065949a96fe6f62f0406c6c649c71d60c9b4  out/v5-r2-u1.txt
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  out/v5-r2-u2-with-the-filter-off.diff
dbb03c28fa12c81ed2b1d1dfd157486160b0c2cbfa1d59ebcc0bf8fdea2658b9  out/v5-r2-u2-with-the-filter-off.txt
f2e7a0913197d530188824bc9e8d9435272774e61d13ff04381f78669a5e235f  out/v5-r2-u2.diff
aa227f8788733bf40c58c27918677cd2306c539861a7ba009e49deadcee1c67e  out/v5-r2-u2.txt
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  out/v5-r2-u6.diff
54f53506c2748750d4441a5c91810ed3ba834398eca944a5e91122665e04682d  out/v5-r2-u6.txt
b1a8b82317095c1703d642fa09a8bb6de36c819905ce31b63517a4eabbfcaa76  patches/mutation-device-condition-always-true.patch
c8a8bd537652e1487695509a23ba31e3a2ecb68bf56e6c8f316175505f8b95bd  patches/mutation-filter-never-applies.patch
81acd65f42f89d5a7ea3993e7faeaa6e09dac9687ddc702344045e0df9016cae  patches/mutation-length-condition-always-true.patch
0066d74c134af247a3f1678fcf92f7d9e631099249b36c27e2656f5ddbc8754c  patches/mutation-offset-condition-always-true.patch
24881305e3a8ce08ff09145874cbe5ab5ac384be5fd5a1dea04c4728fbbc0f51  tests/r1_attack_probe_rehearsal_reads_a_device_without_any_self_verified_system_configuration.rs
11f95c8a332b943c9046baebf0a4416f82e41801fc6f33a41cbd940150fc1814  tests/r2_probe_u1_check_before_rehearsal.rs
7b5ebab71674da4a7e4594d4c054d06bf93c44c0964125f9a79155bf11c165a6  tests/r2_probe_u2_zeroed_slots_with_read_faults_in_the_rehearsal.rs
400e55d5efdbd383878f1bc3c30632f24b3c2cac8deca7551ff92973af86e791  tests/r2_probe_u6_instance_table_sweep_threshold_by_mounting_many_times.rs
8be9a800e9356a3daa963b334897427742c9d080838c8c5d10763f700734ac47  tests/r3_probe_v1_a_torn_single_slot_on_a_device_the_acquisition_does_not_rewrite.rs
e03fa79001431ddf6a1c06e475fe5978fff381f6f1a7f602738f540acc7656fe  tools/rerun.sh
```
