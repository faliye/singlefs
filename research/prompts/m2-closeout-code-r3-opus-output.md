# m2-closeout-code-r3 云端攻方腿（Opus）报告

轮名 `m2-closeout-code-r3`（里程碑二收尾代码三方第三轮），格 X1、X5 ①②③、X6（正文 `research/prompts/_m2-closeout-code-r3-body.md` 第三、四节）。代码一律读快照
`refs/sop/m2-closeout-code-r3-snapshot`（提交 `0cc2a238a57d297f511f9e08cedbbc36c6d56d67`），取到草稿目录后 `crates-sha256.txt` 逐行核过（`sha256sum -c` 对不上 0 行）；用例写在快照副本上另加的原型包里跑，没碰主工作区。
下文「快照 `文件` 第 N 行」都是快照里的行号。日期 2026-09-28。线程上限 8、内存上限 8G（`research/scripts/replay.sh` 的缺省）。
开跑前 `ps` 看到：别的会话的 `cargo test`（harness 档一条、另一台机器上的 checker 档分片经 ssh 一条）、合入后验证二的一条编译；没有 `qemu-system`、`vm-bench.sh`、`e152`、`fio`。
我的第四条内存包装的命令排过一次队（`run-with-memory-cap: 排队（已等 60 秒…）`，账上在跑 4 条），没等文件锁以外的东西。

## 各格判定一览

| 格 | 打中的 / 试的 | 结论（三种之一） | 已知？ |
|---|---|---|---|
| X6-a 落后支 | 发布 P 途中（单元已写、根槽没写）取下的盘 d 旧快照换上来，会话覆盖写 / 建 inode 照常做成，之后池级 checker 红 1 条 I-3.1；58 / 1268 格。抬 F、卸载、回退靠「已知根槽读坏」拒掉，不是落后支拒的 | 和条款说反话（D18 已定项 11 第 314 行「不落后」）；原型 C 修这 58 格、一槽坏 0 误拒（量过，零轮） | 否 |
| X6-b 只剩一槽自证、回退之后建对象写几次 | 1728 串单故障历史 | 兑现了条款（没打中） | — |
| X6-c 四入口核两样、拒在任何写之前 | 956 格拒、0 格拒前写；快照自带 10 条过 | 兑现了条款（没打中） | — |
| X6-d core 与 checker 同一张表 | 21 格两边收拒一致；E 格（371 = 0）两边报的成员名不同 | 兑现了条款（没打中） | — |
| X6-e I-7.14 | 字段互换只红 I-7.14；盘体对调红 I-7.14 与 I-7.1 | 兑现了条款（没打中） | — |
| X1-a 读不出的更新状态与出路 | 新实例写行之后掉电、写行那条根的槽再坏一个字节或一个扇区：每次可写挂载都拒、只读照常；只做过 mkfs 的池同形 | 兑现了条款（D23 已定项 14 第 387 行字面），条款的理由罩不住这一格，与 D16 已定项 7 第 183 行说反话；没有出路；零轮 | 否（不是 C579 那一形） |
| X1-b 已知槽集含挂载后写过的槽 | 代码与 X6-a 同位置的观测 | 兑现了条款 | — |
| X1-c 「自证不过」一半 | 四处走到；三处读根环不分类，单故障下无害（推的 + X5 ① 读故障 0 红） | 替没写的条款做了选择（回退扩写与另两处） | — |
| X1-d 回退扩写拒错或放错 | X5 ① 回退读故障、X6 两条扫 | 没打中 | — |
| X1-e 两条改形用例、水位与 F_生效 取 max 挪到哪 | 改形对；新水位用例在副本上对第 736 行变异判红；变异表第 736 行点名的测试在快照里不存在；树 ID 水位取内存里那一半改形后没有用例罩着（副本上变异 45 条全过） | 兑现了条款；变异表一行靶名没跟着换 | 否 |
| X5 ① Y1 换历史再抽一次 | 种子 1–10（短 6 条、长 4 条，各含回退、抬 F、崩了再挂的组合）× 五个入口：单故障扫 17516 段（盘面 / 重挂 / checker / 读回 / 重发）0 段红（30 段是已知的取号回卷写）；回退 / 抬 F / 卸载一返回就掉电与回退、抬 F 注入之后掉电，111 条流 721617 个崩溃状态 0 红 | 没打中（第二次抽样） | 取号回卷写：已知（D18 已定项 11） |
| X5 ② 轮换之后屏障 litmus | 形态、锚点、步数 35 与根槽之后那四步 | 兑现了条款；litmus 注释里一个路径过期 | 否（路径） |
| X5 ③ 零单元发布屏障报错用例 | 快照上跑过；结局逐样核对 | 兑现了条款（D23 已定项 14 第 407 行的选择）；之后那一步接 X1-a | — |


## 复跑命令与文件哈希

模型目录 `research/prompts/m2-closeout-code-r3-opus-model/`。复跑（在仓根下）：`bash research/prompts/m2-closeout-code-r3-opus-model/rerun.sh <草稿目录>`。它把快照解四份（原样、原型 B、原型 C、第 736 行变异），
把原型包 `opus-r3-proto`（`Cargo.toml`、`lib.rs`、五份 `opus_r3_*.rs`）装进工作区，经 `capped.sh` 与 `run-with-memory-cap.sh 8G` 跑 `checks.sh`、X6 三份、X5 批 2 与批 3（约 75 分钟），末尾 `analyze_x5.py` 分格。原样日志在 `logs/`。

| 文件 | sha256 |
|---|---|
| `analyze_x5.py` | `46ad5c296513b2cda3f15499a6f5fe025d12591c272cb0f837a0f4cbf7b5a20e` |
| `batch_x5.sh` | `3873b2320f5ad6a56be5341fd5b792c54365b19f02c17770711290b3e995cc06` |
| `Cargo.toml` | `34496b954c2bf4647b999e8b4755c96209fe80d9a9b15f3ca542e2d911404541` |
| `checks.sh` | `c71f4a168bed4931a3c220a979643f811272933353f3c701e9f1bcb6383a1e46` |
| `fixB.patch` | `653507aeb0da9a2a2ee62440c9f21fd7e97e78ad3bb2298ef0d7ce353c3fa7ed` |
| `fixC.patch` | `33050ad81d9ca19a909d4cc954dd5b8122313369faddda2052a29faceed64d73` |
| `lib.rs` | `78ac9bfaf015bd84f2e7d5732fc4f1d9917e7f7177e09fbe756969c2c6886880` |
| `logs/bases.txt` | `49e951efd82bd8179fad48a10ca40352e0e67df4bffadea362406eecff0d22e5` |
| `logs/batch2.out` | `7e612ba05e42fa6f60d86bcdcb8f3e22979fd41d11b93de4bebb3d3660731128` |
| `logs/batch3.out` | `e0689208b60498f2c866b0c56960635bd400769114d690c230b52381626098a3` |
| `logs/checks.out` | `89be80f22ffb411c3448ab8b56a1540cd1e5cfeff5c89d508c5be7fbc81eb0f5` |
| `logs/mut736-old-target.log` | `d43a014ba823f4b96bf69051456877f0c7fd7ecd12a9f4fde71c1c66b91cd144` |
| `logs/mut736-watermark.log` | `3afb0fae43415848295eae4718070d56ead275096fc504b69376a2e25c6f67ee` |
| `logs/mutTree.log` | `de82de54a0e11bdead12d258e6e0cdf33fb3440a5acd38df31d92e3d8eb626eb` |
| `logs/repo-entries.log` | `6e2282da5cddc354fdd8d640a9f90707ffd33964e6c24ad57d16ddd44254715c` |
| `logs/repo-i714.log` | `d7298f6bcdf05ca4137d079418c502bb9b8af366a012509c2ace6ebe28de5557` |
| `logs/repo-litmus.log` | `9486820c8cc1ae492431f641e28cfeb42f890981daf7752c9d450656373f0e7f` |
| `logs/repo-watermark.log` | `b5e206204eb36d09974439fe40c49514bd6e23e22adc89dc0d9d8583c09ed59a` |
| `logs/repo-x5-3.log` | `ffff8010cfac4ee88bab02ff827cf0261b0473c4c1133a87c0c475cc6e88d293` |
| `logs/x1-readonly.log` | `f9d83d30082c454f7831ffeb8f3edd5da37e505a79c0f2610150970ad5fee91e` |
| `logs/x5a-1.log` | `dbf2a2dde1b351223d5ecf6c40ad7abfcc4677334fb3e43e67b643a652db6bbd` |
| `logs/x5a-2.log` | `79267434085ea67cb3c8468cc16eb85045cfb05e048748552470f6611c94f86c` |
| `logs/x5a-3.log` | `db7d92f1753db1417365713224a940cf9417df9e940ea9d677ffd4294677d115` |
| `logs/x5-analysis-enum.txt` | `67cde848dc9ffb058125496b503a713052c0e7ad408e31c6f173b0f55c5eeaaa` |
| `logs/x5-analysis-sweep.txt` | `3eee1c0b867ccaa7f9dceb51bcca324bfd553220698558d26fc13e85c55c8b14` |
| `logs/x5b.log` | `52690b9d7522f635ebd447c5acd14094e5e87e2d6dadd4b64355ccd7f4a9dc59` |
| `logs/x5c.log` | `d7f30007aa74bc86958ac21030f951c1250246ade64c7f5ded8ace97ac8e3300` |
| `logs/x5d-1.log` | `f8581f4e9111e13af12e5fc2e5fcefdeaed33620c6cb186c64ba8db1fbe0ef69` |
| `logs/x5d-2.log` | `12c94499744220afb354e6da7e35c5d9ba525cf5fba5bde31d330d4a49091acb` |
| `logs/x5e.log` | `d689ff68fcd0235c09c1d3ea7056a721377f41772380f44521b6f1739244529c` |
| `logs/x5f.log` | `b32649a351d4722de9d0fdd4016f6614b9f5ab67a83a9f83250c792e78f5f7e1` |
| `logs/x5g.log` | `71067d03d6d3fa6022c2f884d4bba91db0de37a9c40c179beb5a255843a6cb32` |
| `logs/x5-probe.log` | `bd826c62e0fcb0ae325e43622efcc6da1a560f69d0e7ec1c54872426ed049c53` |
| `logs/x6-debug.log` | `ed5e0a317d6a0a03f22d8dcd40de95daa293af95043f13ade9f69f5064e0efc8` |
| `logs/x6-fields.log` | `80ac64cdd5a79bcf346c4a20c89fe46e451da96c71e7e1ad4b78412ac3653397` |
| `logs/x6-fixB.log` | `b892ef5442d06c3c38463401b9db751c34c6aadcc68e6bd4d5e893e96b6177df` |
| `logs/x6-fixC.log` | `02e82451f8f1a25a5072954c1c8b2292a277397c90f7eb26d5e85dac9a5c522d` |
| `logs/x6-snapshot.log` | `0d3cb853a15bb0639c795ae7b5218dd949dfe3c9d1e6b1b3514c15edc593bf21` |
| `mut736.patch` | `bc01af4a57a1e929f226b1553b0227bb5c03ded8b26e9ce8956750b547b885df` |
| `opus_r3_x1_single_fault_leaves_the_pool_read_only.rs` | `f52480ede737960bacf8d907ecca29f4cab65ea6a8705a7572b47ea981ac8df1` |
| `opus_r3_x5_resample_single_fault_then_power_loss.rs` | `b4c754d2509ab0ab7bf42bd87a3fc1942123366338be5682a1a58f13bddcedab` |
| `opus_r3_x6_stale_snapshot_mid_publish_and_one_slot_fault.rs` | `b18436b047952057545acaaa28ea0353c768af56789cb8cf98b0139926ee4634` |
| `opus_r3_x6_system_configuration_fields_core_and_checker.rs` | `45ab49adcee34186a99d49dd64f379487eaaa1b9c5063d6e1bb113c8de6146d1` |
| `rerun.sh` | `b7ad0e3fa8e77f6c3116e26284e6e055366c080c1d24ad59e4c326323316ef73` |

主 agent 2026-09-28 两条消息（做到 X5 批 3 时收到）：转述用户看过草稿、定 X1-a 维持今天的判法、X6-a 选改法 C 放到里程碑三，要本腿照已量的写、不为这两格加活；
再转述用户要批 3 现在就停、腾出 CPU 与内存给第五步层 0 全量。本腿照做：批 3 件 3 停下（停在哪见 X5 ①），两格的判定照本腿的量与判写，定案那一句只是转述、本腿没核。

## X6　四入口的落后支、只剩一槽自证、325 / 371 同一张表、I-7.14

### X6-a　落后支「落后且缺现行那一版的单元」：发布途中取下的旧快照混进来，会话照常发布，池级 checker 红 I-3.1（打中）

用例 `opus_r3_x6_stale_snapshot_mid_publish_and_one_slot_fault.rs` 的 `a_stale_snapshot_taken_mid_publish`（原样日志 `logs/x6-snapshot.log`）。
起点：两块单元区 384 槽的小盘，第一个文件之后会话覆盖写三次（实例 1，txg 6），只建一次、每格从内存里那一份拷。
一次发布 P（覆盖写 / 准入抬 F / 管理员回退 / 建一个 inode）录下写流，P 做成之后，把盘 d 换成「起点 + P 写流前 k 步」时盘 d 的样子（k 从 0 扫到末尾，
与 P 做完的盘 d 逐扇区相同的 k 跳过），再做一个用户动作（会话覆盖写、建一个 inode、准入抬 F、正常卸载、管理员回退、崩了再可写挂载，六个都跑，放开扫）。
做成的：盘面池级 checker，再拷一份崩了再可写挂载、checker、读回内容。四个 P × 两块盘 × 全部 k × 六个动作共 1268 格。

**结局**（原样末行）：
```
STALE accepted=312 refused=956 flagged=58
```
58 格全是同一形：盘 d 取在 P 的单元（与 journal 记录）已写、P 的根槽 FUA 还没写的那几步之间，而 P 的根写在盘 d 上；之后的会话覆盖写或建 inode 做成，
盘面池级 checker 红 1 条 I-3.1。按 P 与盘分（数自原样日志）：

| P | 换的盘 | P 的根在哪块盘 | 打中的 k | 格数（会话覆盖写 + 建 inode） |
|---|---|---|---|---|
| 覆盖写（txg 7） | 1 | 1 | 20–26（第 27 步是根槽 FUA） | 14 |
| 建一个 inode（txg 7） | 1 | 1 | 16–22 | 14 |
| 管理员回退到 (1,3)（txg 7） | 1 | 1 | 12–18 | 14 |
| 准入抬 F 到 3（txg 7、8，现行 txg 8） | 0 | 0 | 40–47 | 16 |

一格的原样输出（P = 覆盖写，盘 1，k = 20，六个动作）：
```
STALE P=overwrite stale_device=1 k=20 last_op=Write action=overwrite outcome=Ok wrote=true checker=1 first=Some("I-3.1: 盘 0：记账的已分配 Some(1228800)，遍历全部有效根得到 1032192（其中隔离豁免 0）；机理：根环槽数 24、最新根 txg 8、环里自证过的根槽 8 个、最老的自证过的根
STALE P=overwrite stale_device=1 k=20 last_op=Write action=raise_floor outcome=Err RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { ring_slot: RootRingSlot { region: 1, slot: 2 }, first_reading: NotSelfVerified, reread: NotSelfVeri wrote=false
STALE P=overwrite stale_device=1 k=20 last_op=Write action=unmount outcome=Err NewerStateStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot { ring_slot: RootRingSlot { region: 1, slot: 2 }, first_reading: NotSelfVerified, rerea wrote=false
STALE P=overwrite stale_device=1 k=20 last_op=Write action=rollback outcome=Err CandidateJudgementStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot { ring_slot: RootRingSlot { region: 1, slot: 2 }, first_reading: NotSelfVerifie wrote=false
STALE P=overwrite stale_device=1 k=20 last_op=Write action=crash_remount outcome=Ok chose_1_7 wrote=true checker=0 first=None remount=Ok checker_after=0 first_after=None read=FileRead root=Some((InstanceGeneration(3), CheckpointTxg(13))) content_ok=true
```
（建 inode 那一行与覆盖写同形，I-3.1 的已分配数是 1179648。）三个挂着的入口（抬 F、卸载、回退）拒掉它，靠的不是落后支，是 X1 那条「这个进程写过的根槽读坏重读仍坏就拒」：
P 的根槽是这个进程写过、FUA 返回过的，换上来的盘 1 那一槽是旧内容、自证不过。会话发布不读根环，于是只有它放了进来。
可写挂载（崩了再挂）也放行，择 (1,7)（靠 journal 记录施加），之后 checker 0——可写挂载那一处的落后支本来就按「根落盘之后、轮换之前崩」放行这一形。
会话发布之后的那一份盘面：I-3.1 红（P 那一版的单元仍在记账的已分配里，而环里没有 P 的根、遍历不到它们）；P 的根在盘 d 的根环里缺一条（直到根环转过一圈、盖掉那一槽）；
再崩了再可写挂载一次之后 checker 0、读回内容对（同一行 `checker_after=0 … content_ok=true`）。

**代码**（快照行号）：落后支的判法在快照 `crates/singlefs-core/src/mount.rs` 第 3113 行 `units_of_the_version_missing_on_a_device_behind_it`：
(实例, tail) 落后于现行那一版末条记录的 jsn 之后，只逐个读 `current.units` 的落点，一个都不缺就交回 `None`（放行）。挂着之后的入口经第 3150 行
`devices_behind_the_current_version` 调它，会话在快照 `crates/singlefs-core/src/mounted_session.rs` 第 284 行调它。它不看现行那一版的根在不在这块盘上。

**条款**：`.claude/kb/decisions/18-块里携带什么信息.md:314`「这块盘最新那份自证系统配置的 (实例代号, journal tail) 不落后于现行那一版末条记录的 jsn（只剩一槽自证的盘不误拒」。
同一行放行可写挂载落后支的理由：`.claude/kb/decisions/18-块里携带什么信息.md:314`「放行两种：只落后、单元都在（根落盘之后、轮换之前崩」。

**四句**：
1. 分不分辨臂：分辨。三条臂——A（快照今天的判法：落后且缺单元才拒）、B（第 314 行字面：落后就拒）、C（本腿原型：落后且「缺单元，或现行那一版的根写在这块盘上而这块盘那一槽里不是它」才拒）——
   在这 58 格上 A 放行、B 与 C 拒；在下面 X6-b 的一槽坏 1728 串上 A 与 C 都不拒、B 误拒（量过，见改法表）。
2. 系统当时看得到判别它的东西吗：看得到。现行那一版的根（`current.root` 的实例与 txg）在进程内存里，它住哪个根环槽由 txg 现算，读那一槽一次就知道盘 d 上有没有它；
   可写挂载那一处的「根落盘之后、轮换之前崩」与这里的「根没写上去」在盘上的差别正是这一槽。
3. 满足的是判据字面的哪一个分句：第 314 行「不落后于现行那一版末条记录的 jsn」——这 58 格上盘 d 落后而入口放行，与字面相反；
   括注「只剩一槽自证的盘不误拒」要求的「不误拒」由 A 做到（X6-b），代价就是这 58 格。字面两半（「不落后」与「不误拒一槽盘」）只看系统配置分不开，实现选了「缺单元」去分，这一刀没切在「根在不在」上。
4. 跑前条款给的改法在这几格上还中不中：第二轮判决给的改法就是「四个入口再核两样」，照 A 实现，这 58 格它不修；B 修这 58 格但在一槽坏上误拒；C 两边都不中（只在我的模型上量过、被攻过零轮）。

**可达性**：要换盘（与第二轮 Y2-a 同一类，用户 2026-09-27 定「四个入口都核」时点的就是这一类）。掉电造不出这一形：盘 0 的轮换写在根槽 FUA 返回之后才发，
「盘 0 已轮换而盘 1 上的根槽没写」不是任何一个崩溃状态（推的，按快照 `crates/singlefs-core/src/transaction.rs` 第 1201 行三步的次序）。

**结论**：和条款说反话（第 314 行字面「不落后」）；实现的取舍在「发布途中取下、单元齐而根没写上的旧快照」这一形上让它混进来，会话发布之后 checker 红 I-3.1、P 的根在环里缺一条。
推翻条件：有一个 k 在原型 C 上 checker 仍红，或在一槽坏的某一串上原型 C 拒了。

### X6-b　只剩一槽自证的盘、回退之后建对象写几次：不误拒（没打中）

同一份用例的 `one_system_configuration_slot_fault_after_each_publish`：P（四种）做成之后，盘 d 的一个系统配置槽坏一个字节（两块盘 × 两槽各试，是合法的单故障），
再接三步用户动作（前两步六个动作放开扫，第三步覆盖写 / 建 inode / 卸载），每一步看有没有被「可见」、本盘设备号或落后支拒，三步之后盘面 checker、再可写挂载、读回内容。原样末行：
```
ONE_SLOT sequences=1728 flagged=0
```
1728 串里一步都没有被这三判拒，checker 0，读回内容都对（其中 P = 管理员回退之后接建 inode / 覆盖写的都在内）。第一遍跑时有 96 串报「读回内容不对」，
查下来是装置的错：内容表跨串共用，不同串里同一个 (实例, txg) 是不同的发布；改成每串各拷一份之后 0 串（同一文件的 `content_check_of_the_one_slot_sequences_with_a_crash_remount`
把其中一串单拎出来跑，坏与不坏两种读回都与 P 那一版相同，原样日志 `logs/x6-debug.log`）。快照自带的 `a_device_left_with_one_self_verified_system_configuration_slot_is_not_refused_by_any_entry`
本腿也跑了，过（`logs/repo-entries.log`）。

结论：兑现了条款（第 314 行括注「只剩一槽自证的盘不误拒」）。推翻条件：换一种单故障（一个单元副本坏）加一槽坏之外的单故障、在某一步被落后支拒。

### X6-c　四入口「可见」之后核本盘设备号与落后支、不过在任何写之前拒（没打中）

- X6-a 那 1268 格里被拒的 956 格，没有一格在拒之前写过盘（用例对拒掉的每一格比盘面逐扇区，`refused_after_writing` 0 格）。
- 盘体对调：快照自带 `entries_after_mount_refuse_swapped_or_behind_devices.rs` 10 条（四个入口 × 旧快照、四个入口 × 盘体对调、一槽自证、16 格放开扫）本腿在快照上跑过，10 条全过（`logs/repo-entries.log`）。
- 落后判的级：可写挂载取号之前（快照 `crates/singlefs-core/src/mount.rs` 第 3060 行 `devices_without_the_selected_version`）与挂着之后的入口（第 3150 行）调同一个函数（第 3113 行），比的都是 (实例代号, journal tail) 对那一版末条记录的 jsn，同一级（读代码）。

结论：兑现了条款。推翻条件：一个入口在拒之前发过写。

### X6-d　core 与 checker 收不收同一张表（没打中；一格报的成员不同）

用例 `opus_r3_x6_system_configuration_fields_core_and_checker.rs`（原样日志 `logs/x6-fields.log`）：4 GiB 两盘第一个文件之后那份合法镜像，第二轮 Y4 那五格（A：417 改 +64；B：325 改 1023；C：371 改 65；
D：环长改到 768 MiB − 16 KiB、417 跟着现算成 50175；E：371 改 0）加 F（371 改 128）、G（325 改 2048），每格三种改法（四槽都改、只改盘 1 世代号小的那一槽、只改盘 1 世代号大的那一槽），共 21 格。
每格 core 只读挂载、可写挂载都整池拒（`SystemConfigurationValueRefused`），池级 checker 恰红 1 条 I-7.13、别的 0 条——21 格两边收拒一致，只改一槽的 14 格也一致（core 逐槽判，`mountable_slot_or_refusal`，快照 `crates/singlefs-core/src/recovery.rs` 第 838 行）。
第二轮 Y4 里 E、C 两格 core 照挂、A、D 两格 checker 不红，这一次都对上了。

不同的一处：E 格（371 = 0）core 报 `FixedStructureSlotSpacingOutsideTheFormatRange`，checker 报 `RootRingBaseSlotNotTheFirstVersionConstant`。原样两段：
```
Y4R3 E_371_is_0 every_slot core_read_only=err=Recovery(SystemConfigurationValueRefused { device: DeviceIdentity(0), value: FixedStructureSlotSpacingOutsideTheFormatRange { fixed_structure_slot_spacing: 4096 } })
checker_violations=1 i_7_13=1 list=["I-7.13: 盘 0 偏移 0 的系统配置槽自证过，而池级字段越出这一版读者收的范围（RootRingBaseSlotNotTheFirstVersionConstant）：实现整池拒绝挂载"]
```
core 先判槽距（上界按同一槽自述的根环起点，起点 0 时一档都不收）、后判起点等不等常量；checker 的 `geometry_of` 先判两个起点、后判槽距。
两边都落在 I-7.13 这一条、都整池拒，只是成员名不同：拿成员名对拍 core 与 checker 的用例（合入后验证二第二段的交叉测试）在这一格上要按「同一条、不同成员」写，否则这一格会红或被写成两边不一致。

「逐档找槽 1 仍用常量」（快照 `crates/singlefs-core/src/recovery.rs` 第 684 行 `largest_fixed_structure_slot_spacing_the_format_allows`）与「判一槽的槽距用它自述的根环起点」两处：
371 合法（= 64）时两个上界都是 64 × 16384 − 4096 = 1044480（算术），一收一拒走不到；371 不合法时 core 在收下那一槽之前就按起点整池拒（上面 C、E、F 三格）。另记一条线索（不在这一轮的 diff 里）：
全池没有一个槽 0 可择时 core 逐档找槽 1，checker 的 `system_configuration_slot_readings`（快照 `crates/singlefs-checker/src/image.rs` 第 456 行）只按 4096 读一档，槽距不是 4096 的池上两边读的不是同一槽（读代码，没造镜像）。

结论：兑现了条款（用户定「读字段，不等于常量就整池拒」、I-7.13 补同一张表）；E 格报的成员与 core 注释自称的不一致。推翻条件：21 格里有一格一边收一边拒。

### X6-e　I-7.14 与盘体对调（没打中）

- 快照自带 `devices_whose_own_device_numbers_are_swapped_redden_only_the_own_device_number_invariant` 本腿跑过，过（`logs/repo-i714.log`）：本盘设备号字段互换的坏镜像只红 I-7.14。
- 整块盘体对调（本腿用例 `swapped_device_bodies_on_the_pool_checker_and_both_mounts`，第一个文件之后、覆盖写两次之后各一次）：池级 checker 红 2 条——I-7.14 与 I-7.1（按身份读根环，读到的是另一块盘的区域）；
  core 可写挂载拒成本盘设备号不符，只读挂载报 `NoValidRoot`。原样：
```
Y4R3 swapped_bodies after_the_first_file ... checker_violations=2 i_7_13=0 list=["I-7.1: 根环里一条自证过的根都没有", "I-7.14: 盘 0 上世代 4 的系统配置槽自述本盘设备号 1：与这块盘在池里的身份不同"]
```
  「坏镜像只红 I-7.14」说的是字段互换那一份；盘体对调那一份多红一条 I-7.1，是同一件事的另一面，不是漏判。只读挂载不核本盘设备号（第 314 行的射程是可写挂载与挂着之后的入口），报的是「没有有效根」，拒了、说的原因不对。

结论：兑现了条款。推翻条件：盘体对调的镜像上 I-7.14 不红。

## X1　读不出的根槽怎么算：已知槽集、「自证不过」一半、回退扩写、两处退路、两条改形用例、F_生效 取 max

### X1-a　一条单故障历史让可写挂载永远被拒、条款没给出路（打中；C579 的邻格，不是 C579 那一形）

用例 `opus_r3_x1_single_fault_leaves_the_pool_read_only.rs`（原样日志 `logs/x1-readonly.log`）。历史：第一个文件、会话覆盖写一次（实例 1，txg 4），进程退出；
可写挂载（实例 2）写行那次发布落盘——根槽 FUA、两盘轮换、屏障——之后、第一次暖机的根落盘之前掉电（录制流截在那一串屏障的末尾，是层 0 的一个段边界状态）；
之后单故障：写行那条根（实例 2，txg 5，盘 0）所在的根环槽坏——改一个字节（自证不过），或那一个扇区一直读报错。原样输出：
```
MOUNT instance=2 chosen=(1,4) warm_ups=2
CASE after_the_row_publish cut=30 newest_root=(2,5) on_device=0 offset=7344128 read=FileRead root=(2,5) len=2999
CASE after_the_row_publish fault=none writable=Ok(3)
CASE after_the_row_publish fault=root_slot_not_self_verified attempt=0 writable=Err("NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(4) }, wi") disk_unchanged=true read_only=FileRead root=(1,4) len=2999
CASE after_the_row_publish fault=root_slot_not_self_verified attempt=1 writable=Err("NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(4) }, wi") disk_unchanged=true read_only=FileRead root=(1,4) len=2999
CASE after_the_row_publish fault=root_slot_not_self_verified attempt=2 writable=Err("NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(4) }, wi") disk_unchanged=true read_only=FileRead root=(1,4) len=2999
CASE after_the_row_publish fault=root_sector_unreadable attempt=0 writable=Err("NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(4) }, wi")
CASE after_the_first_warm_up cut=53 newest_root=(2,6) on_device=0 offset=1056768 read=FileRead root=(2,6) len=2999
CASE after_the_first_warm_up fault=root_slot_not_self_verified attempt=0 writable=Ok((3, 5)) disk_unchanged=false read_only=FileRead root=(2,5) len=2999
CASE after_the_last_warm_up fault=root_slot_not_self_verified attempt=0 writable=Ok((3, 6)) disk_unchanged=false read_only=FileRead root=(2,6) len=2999
```
（全部行见日志；「坏扇区」两次尝试与「改一个字节」三次尝试结局逐字相同。）对照：同一截断状态不坏任何槽，可写挂载做成；截在第一次暖机或最后一次暖机落盘之后、坏最新那条根，
可写挂载做成（同一实例里靠 journal 重放追得上）。只有「新实例只有写行那一条根」这一窗里坏它，每一次可写挂载都在取号之前被拒、盘上逐字节不变，只读恢复读回实例 1 的 (1,4)。
只做过 mkfs 的池上同形：第一次可写挂载写行那次（零单元发布）落盘之后掉电、那条根 (1,1) 的槽再坏，三次可写挂载逐字相同地拒、只读恢复读回「没有文件」的 (0,0)（同一文件 `the_same_shape_on_a_pool_that_only_went_through_mkfs`，原样见 X5 ③）。

**条款**：
- 实现照的是 `.claude/kb/decisions/23-journal的角色与格式.md:387`「仍为真就在取号之前拒可写挂载、盘上逐字节不变，只读挂载照常」；这一格里判据走的是「为真」那一支：
  `.claude/kb/decisions/23-journal的角色与格式.md:387`「所选那一版那次发布带末条标志的记录读得出（任一份）⇒ c_见证 > 它的计数器即为真」（实例 1 的末条记录读得出，见证值是实例 2 写行那条的计数器），不是 C579 的「两条都读不出按真」。
- 同一格另一条已定项说的是退回去照常走：`.claude/kb/decisions/16-发布语义.md:183`「在下一次发布之前是单点…那个槽读不出或它所在的盘掉了，恢复退到上一个实例的根，重放按严格前缀停在实例边界」，
  代价写的是丢事务：`.claude/kb/decisions/16-发布语义.md:183`「这次挂载里 fsync 已返回的事务丢，一个故障就够」。而这一窗里实例 2 一次 fsync 都还没返回：
  `.claude/kb/decisions/16-发布语义.md:179`「新实例在本实例写成的根覆盖两块盘之前，fsync 不返回」——退回去一个用户看得见的字节都不丢。
- 出路：两份条款都没写这一形之后怎么回到可写（C579 那一行 `.claude/kb/checks-owed.md:491` 管的是「末条记录两份都读不出」，同一个「拒可写、只读照常」，也没有出路）。

**四句**：
1. 分不分辨臂：分辨「乙-配置续照 D23 第 387 行拒」与「照 D16 第 183 行退回上一个实例的根」两条读法：前者永久只读，后者可写、零字节损失。
2. 系统当时看得到判别它的东西吗：看得到。见证值指的那条记录读得出，它是实例 2 的写行记录；环里、journal 里都没有实例 2 的暖机记录——实例 2 的根没覆盖两块盘、fsync 没返回过
   （D16 第 179 行的返回条件），「读不出的更新状态」里没有用户确认过的东西，这件事挂载时读得到。
3. 满足的是判据字面的哪一个分句：D23 第 387 行「为真…仍为真就在取号之前拒可写挂载…只读挂载照常」，兑现字面；与 D16 第 183 行「恢复退到上一个实例的根」在可写那一半说反话（只读那一半两边一致）。
4. 跑前条款给的改法：C579 用户定维持乙字面，这一形照它就是永久只读；本轮没有条款给改法。本腿想到的两个（都没实现、都是推的、被攻过零轮）：
   ① 见证值指向的那一版属于一个暖机没做完的实例（它只有写行那条记录）时按假，照 D16 第 183 行退回；② 保持拒，另给管理员一个「放弃读不出的那一个实例」的出路。

**结论**：兑现了条款（D23 第 387 行字面），条款的理由罩不住这一格，且与 D16 第 183 行说反话；一个单故障（一个扇区）落在「新实例只有写行那一条根」那一窗里，池就永久只读、条款没给出路。
这一形不是 C579 那一行登记的形（那一行是末条记录两份都读不出），零轮，交用户表。推翻条件：在快照上找到一条不经写盘就让这份镜像重新可写挂载的路径，或 D16 第 183 行已被别处改写。

### X1-b　已知槽集里有没有挂载之后这个进程写过的槽（兑现）

- 代码：快照 `crates/singlefs-core/src/allocator.rs` 第 993 行 `ring_slots_known_to_hold_a_root` 交回根环表里全部住着根的槽；这张表挂载时从盘上读（读得出、自证过的），之后每写一条根由第 1637 行
  `record_root_written_by_this_process` 记一条（带单元的发布在取完落点之后记，零单元发布在根落盘之后记，失败的发布连表一起退回、重发成功时换成冻结着的那一份分配器）。
- 观测：X6-a 那 58 格同一位置上，抬 F、卸载、回退三个入口都拒成 `RootRingSlotKnownToHoldARoot { ring_slot: RootRingSlot { region: 1, slot: 2 }, first_reading: NotSelfVerified, … }`——那一槽是 P（txg 7）的根，
  这个进程挂载之后才写的。挂载之后写过的槽进了集合，读坏重读仍坏就拒。
- 一格边角（读代码，没造）：一次发布落盘途中失败而根槽 FUA 已返回（例如轮换那一写报错）时，分配器连表退回发布之前，这个进程写过且 FUA 返回过的那一槽暂时不在集合里；
  这段时间里抬 F、卸载、回退都先被「冻结着一次没重发」拒（快照 `crates/singlefs-core/src/mount.rs` 第 2247 行、第 4900 行），读不到根环，这个缺口走不到。

### X1-c　「自证不过」那一半四个入口走到了没有（走到四处；另有三处读根环不分类，单故障下无害）

走到的（读坏的槽按已知槽集分两类、已知的重读一次、仍坏就拒，读不出与自证不过一样处置）：准入抬 F 算上限（快照 `mount.rs` 第 1641 行 `roots_of_the_ring_for_the_ceiling`）、
抬 F 与卸载重算影子账（第 2336 行附近经 `readable_roots_rereading_ring_slots_known_to_hold_a_root_once`）、管理员回退判「根环里有它」与算 F_生效（第 4558 行 `rollback_candidate`）。
快照自带的 `raising_the_floor_while_a_root_ring_slot_known_to_hold_a_root_stays_not_self_verified_is_refused_before_any_write` 等四条本腿没单跑（在 `unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs`，本腿只单跑了其中水位那一条）。

没走到的三处（读坏一律当没有根、自证不过不重读）：
- 抬 F 那一串里「新 F 不得低于 F 生效值」那一判（快照 `mount.rs` 第 2278 行，`effective_rollback_floor_rereading_the_newest_instance_table_once` 读根环用 `readable_roots_with_ring_slots`）：
  只拿来挡「往下抬」，产品路径上新 F 取上限或现行 txg，都不低于 F 生效值；F 生效值另取系统配置里的 F 的最大值（快照 `recovery.rs` 第 1687 行，第 1734–1736 行取两者的大者），一个根槽坏压不低它。
- 管理员回退算水位（快照 `mount.rs` 第 5033 行 `readable_roots`）：两个水位都与现行那一版内存里的取 max，一个根槽坏压不低它（快照自带的水位用例本腿跑过、过，见 X1-e）。
- 写系统配置槽之前算 F（快照 `transaction.rs` 第 374 行）：读不出重读一次、自证不过不重读，退路同上取系统配置里 F 的最大值。

X5 ① 的扫描里读故障逐个注入（10034 段里读故障 8843 段，覆盖上面每一处读），盘面 checker、崩了再挂之后的 checker 0 段红，见 X5 ①。结论：兑现了条款（D16 第 37 行只管抬 F 的上限；回退扩写与另两处是实现的选择）。

### X1-d　管理员回退扩写：拒错或放错（没打中）

- X5 ① 里回退入口的读故障：28 段拒成 `ResurrectedUnitCopyUnreadableOrMismatched`（复活单元逐盘读核那一步一次瞬时读错就拒，不重读；在任何写之前，重试就过），
  其余读故障段都做成；第二轮 Y1 的「一次读错报成 NotInRing」这一轮 0 段。
- X6-a、X6-b 的回退动作：旧快照上拒（知道住着根的槽读坏）、一槽坏上照常（168 次做成、360 次 `TargetNotACandidate`——那几串里前一步抬了 F，目标落在 F 之下）。
- 回退扩写唯一的「拒」在这个进程写过的根槽坏的时候；出路是重挂（重挂之后那一槽不在已知集合里、当没有根）。重挂本身被拒的那一形见 X1-a。

### X1-e　两条「挂载后改坏一条根」的用例改成哪一形；水位与 F_生效 取 max 挪到哪（两处改对了；变异表一行靶名没跟着换）

- `rollback_by_a_forward_publish.rs` 与 `reuse_after_raising_the_floor.rs` 那两条都改成「这个进程写过的根槽被改坏 ⇒ 回退判候选时拒成 `CandidateJudgementStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot { 自证不过, 自证不过 })`，
  在任何写之前」，并各钉盘面、分配器、现行版本不动。与 D16 第 37 行全句「读不出或自证不过就重读一次，仍坏就拒」一致（改对了）。
- 水位「取 max(内存里的, 环里读得出的)」挪到 `unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 的
  `the_rollback_takes_the_inode_number_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_its_root`。本腿在快照上跑过、过；把变异表第 736 行那条变异打在副本上再跑，它红（原样）：
```
test the_rollback_takes_the_inode_number_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_its_root ... FAILED
thread 'the_rollback_takes_the_inode_number_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_its_root' (1255971) panicked at crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs:1578:5:
```
  新用例罩得住这条变异。但快照的 `crates/mutations.tsv` 第 736 行点名的必红测试仍是旧名 `the_rollback_takes_the_watermarks_from_the_current_version_in_memory_when_its_root_is_unreadable`、过滤串 `the_rollback_takes_the_watermarks`，
  快照的 `rollback_by_a_forward_publish.rs` 里已经没有这个名字：同一条变异打上去、照第 736 行的参数跑，原样 `running 0 tests` / `test result: ok. 0 passed; … 22 filtered out`（`logs/mut736-old-target.log`）。
  按 `research/scripts/mutate.sh` 的判法（没抓到一个 FAILED 的测试名就记「一个测试都没红」），59 号在快照上会把这一行判成没被看见（推的，没跑 59 号）。新用例里的注释说「`crates/mutations.tsv` 里实三「水位」那一行跟着换靶」，快照里没换。
  树 ID 水位那一半（同一个取 max）新用例只钉 inode 号水位，没钉树 ID 水位；变异表里树 ID 水位那几行（第 319–325 行）靶的是别的用例，没有一行对着回退这一处的 max。
  本腿在副本上把快照 `mount.rs` 第 5042 行 `.fold(current.root.tree_identifier_watermark, u64::max)` 改成 `.fold(0, u64::max)`（只取环里读得出的根），跑 `rollback_by_a_forward_publish.rs` 与
  `unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 两份（含 ignored），原样 `test result: ok. 22 passed; 0 failed` 与 `test result: ok. 23 passed; 0 failed`（`logs/mutTree.log`）：
  回退这一处树 ID 水位取内存里那一半今天没有用例罩着（量过，副本）。改之前它由 `rollback_by_a_forward_publish.rs` 那条「C 的根槽读不出、回退照做」一并钉着，改形之后那一条先拒、根槽改回原样再回退，环里读得到 C 的根，这一判就不再分辨了。
- F_生效「取 max(根上带的, 系统配置里的)」：原来钉在回退那条用例上（改坏载体之后 D 拒成 `BelowEffectiveFloor`），改形之后那一判由两条别的用例守：
  `reuse_after_raising_the_floor.rs` 的 `floor_carried_by_the_system_configuration_takes_effect_on_remount_even_when_one_device_lost_its_root_carrying_it`（变异表第 35 行）与 checker 档的
  `the_floor_takes_effect_from_the_system_configuration_when_every_root_carrying_it_is_unreadable`（第 710 行），两条走的都是重挂那一路；三条路径共用同一个函数（快照 `recovery.rs` 第 1687 行），
  所以共用那一处的变异罩得住。回退判候选这一路上「F 只由系统配置撑着」今天没有用例——那一形挂着的时候造不出（挂载的写行与暖机两块盘都写带 F 的根，推的，用例注释同样说「推的，没造」）。「F_生效 取 max」的新用例在第二段、不在快照里。

结论：两条改形兑现了条款；inode 号水位那一判的新用例罩得住，变异表第 736 行的靶名没跟着换（快照上 59 号会报这一行没被看见，推的）；树 ID 水位那一半改形之后没有用例罩着（量过，副本）。推翻条件：快照的 `mutations.tsv` 里另有一行对着这条变异、点的是新用例名。

## X5　C577 加屏障结清

### X5 ①　第二轮 Y1 换一组历史再抽一次：含回退与抬 F 之后立刻掉电（没打中，第二次抽样）

用例 `opus_r3_x5_resample_single_fault_then_power_loss.rs`，批跑 `batch_x5.sh`（批 2、批 3），分格 `analyze_x5.py`。与第二轮 Y1 不同的地方：
- 历史换了：两块单元区 384 槽的小盘（journal 环 6 MiB），第一个文件之后按种子随机走（覆盖写、建 inode、管理员回退到一个候选、准入抬 F、崩了再可写挂载）。短历史种子 1–6（3–6 步），
  长历史种子 7–10（6–9 步，覆盖写权重高，凑得够 4 个非空版本，F 才抬得离开 0）。每个起点只建一次，之后每段从内存里那一份拷。起点（原样 `BASE` 行，见日志）：
```
BASE seed=1 steps=["crash_remount:Ok(())", "rollback_to_1_3:Ok", "overwrite:Ok(())"] current=(2,7) floor=0 rollback_target=Some((2, 4))
BASE seed=2 steps=["rollback:no_candidate", "overwrite:Ok(())", "crash_remount:Ok(())", "crash_remount:Ok(())", "overwrite:Ok(())"] current=(3,11) floor=0 rollback_target=Some((1, 4))
BASE seed=3 steps=["crash_remount:Ok(())", "overwrite:Ok(())", "create_one_inode:Ok(())", "overwrite:Ok(())", "rollback_to_2_7:Ok", "rollback_to_2_6:Ok"] current=(2,10) floor=0 rollback_target=Some((2, 9))
BASE seed=4 steps=["rollback:no_candidate", "overwrite:Ok(())", "overwrite:Ok(())", "crash_remount:Ok(())"] current=(2,7) floor=0 rollback_target=Some((1, 4))
BASE seed=5 steps=["rollback:no_candidate", "rollback:no_candidate", "raise_floor:AtCeiling_0_0", "raise_floor:AtCeiling_0_0"] current=(1,3) floor=0 rollback_target=None
BASE seed=6 steps=["rollback:no_candidate", "crash_remount:Ok(())", "rollback_to_2_4:Ok"] current=(2,6) floor=0 rollback_target=Some((1, 3))
BASE seed=7 steps=["create_one_inode:Ok(())", "raise_floor:AtCeiling_0_0", "create_one_inode:Ok(())", "overwrite:Ok(())", "overwrite:Ok(())", "raise_floor:Raised_to_4", "create_one_inode:Ok(())"] current=(1,11) floor=4 rollback_target=Some((1, 5))
BASE seed=8 steps=["overwrite:Ok(())", "raise_floor:AtCeiling_0_0", "crash_remount:Ok(())", "raise_floor:AtCeiling_0_0", "overwrite:Ok(())", "overwrite:Ok(())", "crash_remount:Ok(())", "raise_floor:Raised_to_3"] current=(3,13) floor=3 rollback_target=Some((1, 4))
BASE seed=9 steps=["overwrite:Ok(())", "create_one_inode:Ok(())", "rollback_to_1_4:Ok", "crash_remount:Ok(())", "overwrite:Ok(())", "crash_remount:Ok(())"] current=(3,11) floor=0 rollback_target=Some((3, 10))
BASE seed=10 steps=["raise_floor:AtCeiling_0_0", "create_one_inode:Ok(())", "crash_remount:Ok(())", "overwrite:Ok(())", "crash_remount:Ok(())", "overwrite:Ok(())", "overwrite:Ok(())", "raise_floor:Raised_to_4", "crash_remount:Ok(())"] current=(4,16) floor=4 rollback_target=Some((1, 4))
```
- 入口：管理员回退（到起点那一刻随机挑的一个候选）、准入抬 F、正常卸载、会话覆盖写、崩了再可写挂载。每段只注入一次（整池第 n 次读、写或屏障报块设备错）。
- 每段之后两件：(a) 第二轮那一套（冻结的原样重发、盘面 checker、拷一份崩了再可写挂载、checker、读回内容）；(b) 入口一返回立刻掉电：录下的写流按层 0 的枚举域全量展开
  （`singlefs_checker_tier::crash::enumerate_layer0_versions`：屏障与 FUA 切段、段内整写子集逐个、原地覆写三态），每个崩溃状态恢复两遍（看 / 不看 journal）、oracle、池级 checker、记录核对器。

**取样与缩**（跑前按小段估、跑后实数）：
- 先跑一小段（种子 1–2 不注入、五个入口全展开）量到 114827 个状态、其中可写挂载那一格一条流 266271 个状态、每秒约 200 个（8 线程）；照这个速率全量（全部故障段都展开）要几十小时，超过 40 分钟，于是缩：
  (a) 全做；读故障每 7 个取一个、末 300 个逐个（缩前每入口 48–3635 个读序号，缩后可写挂载那一格约 800 个、其余不变多少）；
  (b) 只展开「不注入」的回退 / 抬 F / 卸载（全部种子）、回退逐个写与屏障注入（短历史种子 1、3、4、6）、抬 F 逐个屏障注入（长历史种子，这一件另设 30 万个状态的上限）；
  会话覆盖写那一格一条流 1048583 到 16777223 个状态（原样 `skipped_over_per_stream_limit`），超过一段 10⁶ 的线，不展开；可写挂载那一格不展开。
- 留下的每段，它的崩溃状态一个不落（`states` 与带第三态的精确数 `exact` 逐段相等，否则分析脚本判红）。
- 试跑那一段在批 2 之前停了（跑完的 114827 个加停下时那一条流已跑的 33296 个，共 148123 个状态算进这条腿的总量）。
- 批 3 件 3 没跑完：用户经主 agent 要求腾出 CPU 与内存（第五步层 0 全量要开跑），2026-09-28 我按进程号停了它。停时 x5e 在种子 10 的正常卸载那一条流上（原样末行 `LAYER0_PROGRESS slice=60/64 … finished_states=11580/12330`），
  那一条没跑完、不算进上面的数；x5e 在它之前的 10 条流跑完；x5f（抬 F 逐个屏障注入）只跑完种子 7 的前 4 道屏障（4 段：两段报错之前没写、两段各 9 个状态），其余没跑。
- 这条腿原型跑的崩溃状态总量：试跑 148123、批 2 件 4（含 oracle 版本表修之前那一次 49194）205163、批 3 件 3–4 565648（x5e 368854 加 x5f 18 加 x5g 196776）、停下时丢掉的 11580，合计 930514，没超过约 10⁷ 的线；每一条流都不超过 10⁶。

**结局**（`analyze_x5.py` 原样，节选；全部格见 `logs/x5-analysis.txt`）：
(a) 第二轮那一套，批 2 件 1–3 与批 3 件 1–2（`logs/x5a-*.log`、`logs/x5d-*.log`），读故障做成的那几格（`(…, 'read', 'Ok', 'false')`，合计 15377 段）略去：
```
RUNS 17516
CELL ('crash_remount', 'barrier', 'Err Acquisition(AcquisitionFailed', 'false') 40
CELL ('crash_remount', 'barrier', 'Err Publish(PublishSequenceFailed', 'false') 150
CELL ('crash_remount', 'none', 'Ok', 'false') 10
CELL ('crash_remount', 'write', 'Err Acquisition(AcquisitionFailed', 'false') 20
CELL ('crash_remount', 'write', 'Err Publish(PublishSequenceFailed', 'false') 457
CELL ('raise_floor_to_the_admission_ceiling', 'barrier', 'Err RaiseFloorSequencePublishFailed(PublishSequenceFailed', 'true') 42
CELL ('raise_floor_to_the_admission_ceiling', 'barrier', 'Err RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(RaisedFloorSystemConfigurationWriteFailed', 'false') 12
CELL ('raise_floor_to_the_admission_ceiling', 'none', 'Ok', 'false') 10
CELL ('raise_floor_to_the_admission_ceiling', 'write', 'Err RaiseFloorSequencePublishFailed(PublishSequenceFailed', 'true') 123
CELL ('raise_floor_to_the_admission_ceiling', 'write', 'Err RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(RaisedFloorSystemConfigurationWriteFailed', 'false') 6
CELL ('rollback', 'barrier', 'Err Publish(PublishSequenceFailed', 'true') 54
CELL ('rollback', 'none', 'Ok', 'false') 9
CELL ('rollback', 'read', 'Err ResurrectedUnitCopyUnreadableOrMismatched', 'false') 52
CELL ('rollback', 'write', 'Err Publish(PublishSequenceFailed', 'true') 165
CELL ('session_overwrite', 'barrier', 'Err Publish', 'true') 60
CELL ('session_overwrite', 'none', 'Ok', 'false') 10
CELL ('session_overwrite', 'write', 'Err Publish', 'true') 266
CELL ('unmount', 'barrier', 'Err RaiseFloorSequencePublishFailed(PublishSequenceFailed', 'true') 150
CELL ('unmount', 'barrier', 'Err RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(RaisedFloorSystemConfigurationWriteFailed', 'false') 40
CELL ('unmount', 'none', 'Ok', 'false') 10
CELL ('unmount', 'write', 'Err RaiseFloorSequencePublishFailed(PublishSequenceFailed', 'true') 433
CELL ('unmount', 'write', 'Err RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(RaisedFloorSystemConfigurationWriteFailed', 'false') 20
RED ('crash_remount', 'barrier', 'wrote_after_the_failure_without_freezing') 20
RED ('crash_remount', 'write', 'wrote_after_the_failure_without_freezing') 10
REDLINE SWEEP seed=1 entry=crash_remount fault=Some((Write, 2)) fired=Some((Write, 2, 1, 4096)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的写错" }), rollback: RolledBack }) writes_before=1 writes_after=1 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(2), CheckpointTxg(7))) content_ok=true remount=Ok checker_after_re
REDLINE SWEEP seed=1 entry=crash_remount fault=Some((Barrier, 3)) fired=Some((Barrier, 3, 0, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(2), CheckpointTxg(7))) content_ok=true remount=Ok checker_after_
REDLINE SWEEP seed=1 entry=crash_remount fault=Some((Barrier, 4)) fired=Some((Barrier, 4, 1, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(2), CheckpointTxg(7))) content_ok=true remount=Ok checker_after_
REDLINE SWEEP seed=2 entry=crash_remount fault=Some((Write, 2)) fired=Some((Write, 2, 1, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的写错" }), rollback: RolledBack }) writes_before=1 writes_after=1 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(3), CheckpointTxg(11))) content_ok=true remount=Ok checker_after_remo
REDLINE SWEEP seed=2 entry=crash_remount fault=Some((Barrier, 3)) fired=Some((Barrier, 3, 0, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(3), CheckpointTxg(11))) content_ok=true remount=Ok checker_after
REDLINE SWEEP seed=2 entry=crash_remount fault=Some((Barrier, 4)) fired=Some((Barrier, 4, 1, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(3), CheckpointTxg(11))) content_ok=true remount=Ok checker_after
REDLINE SWEEP seed=3 entry=crash_remount fault=Some((Write, 2)) fired=Some((Write, 2, 1, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的写错" }), rollback: RolledBack }) writes_before=1 writes_after=1 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(2), CheckpointTxg(10))) content_ok=true remount=Ok checker_after_remo
REDLINE SWEEP seed=3 entry=crash_remount fault=Some((Barrier, 3)) fired=Some((Barrier, 3, 0, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(2), CheckpointTxg(10))) content_ok=true remount=Ok checker_after
REDLINE SWEEP seed=3 entry=crash_remount fault=Some((Barrier, 4)) fired=Some((Barrier, 4, 1, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(2), CheckpointTxg(10))) content_ok=true remount=Ok checker_after
REDLINE SWEEP seed=4 entry=crash_remount fault=Some((Write, 2)) fired=Some((Write, 2, 1, 4096)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的写错" }), rollback: RolledBack }) writes_before=1 writes_after=1 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(2), CheckpointTxg(7))) content_ok=true remount=Ok checker_after_re
REDLINE SWEEP seed=4 entry=crash_remount fault=Some((Barrier, 3)) fired=Some((Barrier, 3, 0, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(2), CheckpointTxg(7))) content_ok=true remount=Ok checker_after_
REDLINE SWEEP seed=4 entry=crash_remount fault=Some((Barrier, 4)) fired=Some((Barrier, 4, 1, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(2), CheckpointTxg(7))) content_ok=true remount=Ok checker_after_
REDLINE SWEEP seed=5 entry=crash_remount fault=Some((Write, 2)) fired=Some((Write, 2, 1, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的写错" }), rollback: RolledBack }) writes_before=1 writes_after=1 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(1), CheckpointTxg(3))) content_ok=true remount=Ok checker_after_remou
REDLINE SWEEP seed=5 entry=crash_remount fault=Some((Barrier, 3)) fired=Some((Barrier, 3, 0, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(1), CheckpointTxg(3))) content_ok=true remount=Ok checker_after_
REDLINE SWEEP seed=5 entry=crash_remount fault=Some((Barrier, 4)) fired=Some((Barrier, 4, 1, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(1), CheckpointTxg(3))) content_ok=true remount=Ok checker_after_
REDLINE SWEEP seed=6 entry=crash_remount fault=Some((Write, 2)) fired=Some((Write, 2, 1, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的写错" }), rollback: RolledBack }) writes_before=1 writes_after=1 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(2), CheckpointTxg(6))) content_ok=true remount=Ok checker_after_remou
REDLINE SWEEP seed=6 entry=crash_remount fault=Some((Barrier, 3)) fired=Some((Barrier, 3, 0, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(2), CheckpointTxg(6))) content_ok=true remount=Ok checker_after_
REDLINE SWEEP seed=6 entry=crash_remount fault=Some((Barrier, 4)) fired=Some((Barrier, 4, 1, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(2), CheckpointTxg(6))) content_ok=true remount=Ok checker_after_
REDLINE SWEEP seed=7 entry=crash_remount fault=Some((Write, 2)) fired=Some((Write, 2, 1, 4096)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的写错" }), rollback: RolledBack }) writes_before=1 writes_after=1 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(1), CheckpointTxg(11))) content_ok=true remount=Ok checker_after_r
REDLINE SWEEP seed=7 entry=crash_remount fault=Some((Barrier, 3)) fired=Some((Barrier, 3, 0, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(1), CheckpointTxg(11))) content_ok=true remount=Ok checker_after
REDLINE SWEEP seed=7 entry=crash_remount fault=Some((Barrier, 4)) fired=Some((Barrier, 4, 1, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(1), CheckpointTxg(11))) content_ok=true remount=Ok checker_after
REDLINE SWEEP seed=8 entry=crash_remount fault=Some((Write, 2)) fired=Some((Write, 2, 1, 4096)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的写错" }), rollback: RolledBack }) writes_before=1 writes_after=1 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(3), CheckpointTxg(13))) content_ok=true remount=Ok checker_after_r
REDLINE SWEEP seed=8 entry=crash_remount fault=Some((Barrier, 3)) fired=Some((Barrier, 3, 0, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(3), CheckpointTxg(13))) content_ok=true remount=Ok checker_after
REDLINE SWEEP seed=8 entry=crash_remount fault=Some((Barrier, 4)) fired=Some((Barrier, 4, 1, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(3), CheckpointTxg(13))) content_ok=true remount=Ok checker_after
REDLINE SWEEP seed=9 entry=crash_remount fault=Some((Write, 2)) fired=Some((Write, 2, 1, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的写错" }), rollback: RolledBack }) writes_before=1 writes_after=1 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(3), CheckpointTxg(11))) content_ok=true remount=Ok checker_after_remo
REDLINE SWEEP seed=9 entry=crash_remount fault=Some((Barrier, 3)) fired=Some((Barrier, 3, 0, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(3), CheckpointTxg(11))) content_ok=true remount=Ok checker_after
REDLINE SWEEP seed=9 entry=crash_remount fault=Some((Barrier, 4)) fired=Some((Barrier, 4, 1, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(3), CheckpointTxg(11))) content_ok=true remount=Ok checker_after
REDLINE SWEEP seed=10 entry=crash_remount fault=Some((Write, 2)) fired=Some((Write, 2, 1, 4096)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的写错" }), rollback: RolledBack }) writes_before=1 writes_after=1 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(4), CheckpointTxg(16))) content_ok=true remount=Ok checker_after_
REDLINE SWEEP seed=10 entry=crash_remount fault=Some((Barrier, 3)) fired=Some((Barrier, 3, 0, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(4), CheckpointTxg(16))) content_ok=true remount=Ok checker_afte
REDLINE SWEEP seed=10 entry=crash_remount fault=Some((Barrier, 4)) fired=Some((Barrier, 4, 1, 0)) outcome=Err Acquisition(AcquisitionFailed { cause: InputOutput(Custom { kind: Other, error: "注入的屏障错" }), rollback: RolledBack }) writes_before=2 writes_after=2 frozen=false resend=- checker=0 first=None read=FileRead root=Some((InstanceGeneration(4), CheckpointTxg(16))) content_ok=true remount=Ok checker_afte
```
(b) 立刻掉电全量展开，批 2 件 4 前半与批 3 件 3、4（`logs/x5b.log`、`logs/x5e.log`、`logs/x5f.log`、`logs/x5g.log`）：
```
RUNS 124
ENUM_STATES ('raise_floor_to_the_admission_ceiling', 'barrier') 18
ENUM_STATES ('raise_floor_to_the_admission_ceiling', 'none') 90216
ENUM_STATES ('rollback', 'barrier') 98424
ENUM_STATES ('rollback', 'none') 221292
ENUM_STATES ('rollback', 'write') 98352
ENUM_STATES ('unmount', 'none') 213315
ENUM_STATES_TOTAL 721617
```

**判**：
- (a) 17516 段：入口之后盘面 checker 红 0 段、撤掉故障重挂失败 0 段、重挂之后 checker 红 0 段、读回内容不对 0 段、冻结的发布原样重发失败 0 段；读故障让入口报错而之前已写过盘 0 段。
  读故障做成的 15377 段之外，读故障只让回退报 `ResurrectedUnitCopyUnreadableOrMismatched`（复活单元逐盘读核一次读错就拒，在任何写之前）。
  红的 30 段全是可写挂载取号那一步的写或屏障报错之后的回卷写（`AcquisitionFailed { …, rollback: RolledBack }`），是 D18（块里携带什么信息） 已定项 11 的全或无回卷，第二轮 Y1 同一格，不是「写了再拒」。
- (b) 124 段（其中有写、展开了的 111 条流）、721617 个崩溃状态（每条流 `states` 与精确数相等）：oracle 违例 0、不看 journal 那一遍违例 0、记录核对器两判 0、池级 checker 红的状态 0。其中含回退 / 抬 F / 卸载「不注入、一返回就掉电」的 30 条（种子 1–10；
  抬 F 在有 4 个非空版本的种子 3、7、9 上才真抬、真写），回退逐个写序号与屏障序号注入之后掉电 92 条（短历史种子 1、3、4、6），抬 F 屏障注入之后掉电 2 条（长历史种子 7 的第 1–4 道屏障，都落在「先写系统配置」那一步，两段报错之前没写）。

批 2 件 4 后半截（种子 1 回退逐个写与屏障注入）第一次跑有 4 段 oracle 报「第 8 代根下面没有文件却读出了内容」：那 4 段的故障落在回退那次发布的记录已写、根槽没写之间，
恢复靠记录把 txg 8 那一版施加出来，读回的正是回退目标的内容；装置的版本表只按根槽写登记版本、漏了这一版。版本表补上「现行那一版之后的几个 txg、内容同入口」之后，
批 3 件 4 把种子 1 连同 3、4、6 重跑，0 段。原样第一次那 4 行在 `logs/x5c.log`。

**结论**：没打中（这是 Y1「没打中」的第二次抽样，历史、种子、盘宽都换了，含回退与抬 F 之后立刻掉电）。推翻条件：换一组历史或把读序号的步长放到 1 之后，有一段盘面或之后的 checker 红、或一个崩溃状态 oracle 红。

### X5 ②　两份「轮换之后屏障」litmus（没打中；一处注释路径过期）

- 形态：`litmus/publish-returns-after-the-rotation-barrier.litmus` 的 P0 是「根 → 系统配置 → smp_wmb → returned」，P1 是「读 returned → smp_rmb → 读系统配置」，
  `exists (1:r0=1 /\ 1:r1=0)` 声明 Never；`-nofence` 对照去掉 P0 那一道 smp_wmb、声明 Sometimes。这是消息传递（MP）形，屏障在、坏结局不可能，去掉、可能，
  对照组有判别力（herd7 归提交时 57 号，本腿没跑）。
- 锚点：两行 `singlefs-models:` 指 `crates/singlefs-core/src/transaction.rs::persist_the_root_then_rotate_the_system_configuration` 与
  `crates/singlefs-core/src/recovery.rs::verified_system_configuration_slots`；快照里这两个函数在（快照 `transaction.rs` 第 1201 行、`recovery.rs` 第 2854 行），
  `.claude/scripts/lkmm.sh`（快照版）判锚点只核文件在、`fn <名>` 在。重发那一路（`resend_the_frozen_publish`）经快照 `transaction.rs` 第 1047 行 `persist_publish_writes`
  走同一个 `persist_the_root_then_rotate_the_system_configuration`（第 1066 行），litmus 里「三条发布路径与重发共用」这句属实（读代码）。
- 绑到代码：`a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs` 的第二条比的是只做过 mkfs 的池第一次可写挂载（全是零单元发布）的录制流；
  带文件的发布那一路由 `publish_order_matches_litmus.rs` 钉：步数 35、根槽之后恰是「系统配置、系统配置、屏障、屏障」（快照该文件第 51–79 行）。两份在快照上本腿都跑了，见「快照自带用例单跑」。
- 过期的一处：新 litmus 注释写块层那一半在 `crates/singlefs-harness/tests/a_publish_returns_only_after_the_system_configuration_rotation_is_durable.rs`，
  快照里这份文件在 `crates/singlefs-checker-tier/tests/` 下（别的会话 singlefs-8b 的包重组挪的，写 litmus 的一方沿用了旧路径）。57 号只核 `singlefs-models:` 行，不核注释里的路径，这一处没人会红。

结论：兑现了条款（D16 已定项 7 的最后三步与「屏障完成才返回」）。只是注释路径过期，归属写在上一条。推翻条件：herd7 在提交时判 Never 那一份为 Sometimes，或 `-nofence` 那一份为 Never。

### X5 ③　零单元发布轮换之后屏障报错：用例钉的结局（兑现；钉的只是「全落」那一种盘面，之后接 X1-a）

快照自带的 `a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs` 两条本腿在快照上跑过，都过（`logs/repo-x5-3.log`）。它钉的三样，本腿逐样核：
- 错误成员：`MountError::Publish(PublishSequenceFailed { cause: PublishError::BlockDevice(_) , … })`、落盘成功的发布 0 次、落盘阶段失败 1 次、盘 1 拒了恰好一道屏障——与快照 `crates/singlefs-core/src/transaction.rs` 第 1243 行起
  `publish_without_units` 的错误处置一致（失败记账、原样交回，不冻结），与 `.claude/kb/decisions/23-journal的角色与格式.md:407`「零单元发布…这一支不冻结、整个挂载返回错误（失败经 `PublishError::BlockDevice`）」一致。
- 盘上已落的步骤：取号两写、屏障、空记录、屏障、根槽 FUA、两盘轮换、盘 0 那道屏障；写行那条根 (1,1) 在盘上。
- 之后可写挂载：换回不报错的设备，取号 2、择到 (1,1)。

这一格它钉对了。两处它没钉、本腿补看的：
- 屏障报错时盘 1 的轮换写没等到屏障，「落没落」两种都是可能的盘面；用例只钉了「全落」那一种（文件镜像上写照样在）。轮换没落的那一种是「根槽 FUA 之后、轮换之前崩」，
  与层 0 枚举里的崩溃状态同形，可写挂载照常（推的，这一格本腿没单造）。带文件的池上，可写挂载取号之后那一串（写行与暖机都是带单元的发布）的写错与屏障错：X5 ① 里 361 段
  （`crash_remount` 入口的写故障 271 段、屏障故障 90 段，整个挂载返回 `PublishSequenceFailed`、不冻结），撤掉故障再可写挂载 361 段全做成、之后 checker 0。
- 这一支之后盘上「新实例只有写行那一条根」：此时那一槽再坏，就是 X1-a 那一形。只做过 mkfs 的池上本腿造过（`the_same_shape_on_a_pool_that_only_went_through_mkfs`，原样）：
```
FORMATTED newest_root=(1,1) fault=none writable=Ok(2)
FORMATTED fault=root_slot_not_self_verified attempt=0 writable=Err("NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(0), checkpoint_txg: CheckpointTxg(0) }, wi") read_only=NoFile root=(0,0)
```
  一个刚做完 mkfs、第一次可写挂载就遇到这一串的池，之后永远挂不成可写（三次尝试逐字相同）。

结论：兑现了条款（D23 第 407 行的选择）；用例钉的结局对。它之后那一步的风险在 X1-a。推翻条件：换回设备之后的可写挂载在「盘 1 轮换没落」那一种盘面上被拒。

## 改法表：几个改法各修哪一格

「量过」贴的是副本上的原样末行（`logs/x6-fixB.log`、`logs/x6-fixC.log`、`logs/x6-snapshot.log`）；「推的」是按代码推、没实现没跑。副本上的数不算入库装置上的数。

| 改法 | X6-a 旧快照 58 格 | X6-b 一槽坏 1728 串 | X1-a 写行之后掉电加一槽坏 |
|---|---|---|---|
| A 快照今天（落后且缺单元才拒） | 不修：`STALE accepted=312 refused=956 flagged=58`（量过） | 不误拒：`ONE_SLOT sequences=1728 flagged=0`（量过） | 不修（量过，X1-a 原样） |
| B 第 314 行字面（落后就拒，`fixB.patch`） | 修：`STALE accepted=72 refused=1316 flagged=0`（量过） | 误拒：`ONE_SLOT sequences=1728 flagged=1920`（量过；按「被拒的步」计，一串可以拒几步） | 不相干（推的：它只动挂着之后的入口） |
| C 落后且「缺单元，或现行那一版的根写在这块盘上而那一槽里不是它」才拒（`fixC.patch`） | 修：`STALE accepted=254 refused=1101 flagged=0`（量过；多拒的 58 格正是 A 放进来的那 58 格） | 不误拒：`ONE_SLOT sequences=1728 flagged=0`（量过） | 不相干（推的） |
| D 见证值指向的那一版属于暖机没做完的实例时按假、退回上一个实例的根 | 不相干（推的） | 不相干（推的） | 修（推的，没实现） |
| E 保持拒，另给「放弃读不出的那一个实例」的管理员出路 | 不相干（推的） | 不相干（推的） | 给出路（推的，没实现） |

C、D、E 都只在我的模型上量过或只是推的、被攻过零轮。C 的代价（推的）：每次挂着之后的入口对落后且单元都在的盘多读一个根槽；合法历史里这种盘只有「只剩一槽自证」那一形。
C 只核现行那一版的根；比它更早的几条根缺在旧快照上的那一形（快照取在更早一次发布途中、之后又发过几次）会先被「缺单元」拒，本腿的扫描里没见到漏网的（推的）。

## 没打中的形状

| 形状 | 取样范围 | 结局 |
|---|---|---|
| X5 ① 单故障逐序号扫（第二轮那一套：入口之后盘面 checker、冻结的原样重发、崩了再可写挂载、之后 checker、读回内容） | 短历史种子 1–6、长历史种子 7–10；五个入口；写、屏障逐个，读每 7 个取一个加末 300 个逐个 | 见 X5 ① 的数；红的只有取号失败的全或无回卷写（已知，D18 已定项 11） |
| X5 ① 入口一返回立刻掉电，崩溃状态全量展开 | 见 X5 ① 的表 | oracle、池级 checker、记录核对器 0 |
| X6-a 旧快照在「整次发布之间」取（第二轮 Y2-a 那一形） | 每个 P 的 k = 0 那一格 × 两块盘 × 六个动作 | 挂着的入口都拒成落后支，可写挂载拒成见证判据 |
| X6-a 旧快照在「根槽已写、轮换之前」取 | k ≥ 根槽 FUA 那一步 | 放行，之后 checker 0（这一形等同崩溃状态） |
| X6-b 一个系统配置槽坏之后三步用户动作 | 4 × 2 × 2 × 36 × 3 = 1728 串 | 0 串被这三判拒、0 串 checker 红 |
| X6-d 325 / 371 / 417 越界值只写在一槽里 | 7 格 × 3 种改法 | core 两种挂载都拒、checker 恰红 I-7.13 |
| X1 回退时一次读错报成 NotInRing（第二轮 Y1 的已知格） | X5 ① 回退入口全部读故障 | 0 段 |
| X5 ② litmus 的锚点、步数 | 快照 `lkmm.sh` 的锚点判法、两份绑定用例 | 都对上（只注释路径过期） |

## 这条腿自己的限度

- 读序号按步长 7 取（末 300 个逐个）；写、屏障逐个。崩溃状态只在「回退 / 抬 F / 卸载不注入」「回退逐个写与屏障注入」「抬 F 逐个屏障注入（有上限）」这几件上展开；会话覆盖写与可写挂载两个入口的写流一条就超过 10⁶ 个状态或太慢，没展开。
- 每段只注入一次故障、故障只一次（瞬时）；X1-a 另外造了一直坏的扇区。没有造「写成功而内容坏」的写故障（撕裂只在层 0 的第三态里）。
- X6-a 的旧快照是在内存里按录制流前 k 步拼出来的「盘 d 在第 k 步时的样子」，不是真设备上拔插的结果；换盘这一类是用户 2026-09-27 定要挡的那一类（第二轮 Y2-a），不是掉电造得出的状态。
- X6 两条与 X1 那条的起点都只建了一个（第一个文件之后覆盖写三次 / 一次）；X6-a 的 P 只有四种、每种一次。
- 原型 B、C 只改了挂着之后的入口那一处（`devices_behind_the_current_version`），可写挂载那一处没改；C 的正确性只在本腿这两条扫描上量过。
- 本腿的原型包 `opus-r3-proto` 是在快照副本的工作区里另加的一个成员包，依赖 `singlefs-checker-tier` 的库（用它的 `enumerate_layer0_versions`），`cargo test -p opus-r3-proto` 不跑 checker 档包自己的测试；
  原型里的流都是原型自己造的，没从名字带 layer0 的用例里拷（辅助模块只引了 harness 的 `tests/common_admission/mod.rs`）。
- 模型目录里的 `mut736.patch` 在建好之后用 `sed -i` 改过前两行的路径头（这一轮自己新建的文件，没用 `replace-once.py`）。
- 报告里代码的行号都是快照里的（取法 `git show refs/sop/m2-closeout-code-r3-snapshot:<路径>`），不是主工作区的；代码位置不写成 `路径:行号` 引文，cite-check 不核它们。

## 没做什么

- 不判 X2、X3（本地攻方）与 X4（云端辩方）；X1-e 里变异表第 736 行那一处落在「16 行变异里证红」的边上，只报现象，不算钉值。
- 没跑任何重型测试：checker 档包 `singlefs-checker-tier` 自己的测试、54 / 55 / 57 / 59 号、herd7、变异整表都没跑；X5 ② 的 herd7 判定、X1-e 的「59 号会把第 736 行判成没被看见」都是推的。
- X6-a、X1-a 的改法（原型 C、D、E）都没进入库装置；副本上量出的数不算入库装置上的数，主 agent 要引得在入库装置上重做。
- 第二段（双故障两条、`random_histories.rs` 两条、「F_生效 取 max」新用例、core 与 checker 收同一张表的交叉测试）不在快照里，本腿没看。
- X5 ④（屏障代价）不在这条腿。
- X1-c 列的「没走到」三处只读了代码、配了 X5 ① 的读故障扫描，没有专门造「那一槽一直坏」的历史。
