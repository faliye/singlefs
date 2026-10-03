# m3-code-r1 云端攻方腿（Opus）报告：Z2（C585 / C586）、Z3（打洞）、Z4（测试）

日期 2026-10-01。被判代码：冻结副本 `/tmp/claude-1000/m3-code-r1/tree/`（基于 fcc4d340 打三份补丁），全部在它的拷贝上编、跑；下文的数都是副本上的，不是入库装置上的。重型测试一样没跑（checker 档包的测试、层 0 门禁、变异整表都没跑；原型里调了 checker 档库的枚举函数跑自己造的小流，状态数见 Z2-③）。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Z2-① 零轮候选造盘面（候选 3、4、5、6） | 没打中 | 多发布旧快照表 200 664 格、转圈表 3 708 格，原样代码 0 格漏拒 |
| Z2-② 可写挂载 `NotWritten` 那一支 | 没打中 | 「写了、读得出、没被择到」造不出；崩溃态表里施加记录的状态全挂上 |
| Z2-③ 误拒（合法崩溃态 + 可写挂载） | 没打中 | 17 条流 364 911 个崩溃态，新判法一个都没拒；撕裂系统配置槽的旧拒因另记 |
| Z2-④ 重读一次 | **打中** | 去掉重读，一次瞬时读错就拒成落后支；harness 全包与 core 单测 94 个二进制全绿；用例原型已写 |
| Z2-⑤ C586 用例换造法 | 没打中（放得过 W1、W3，今天分不出） | 树号只在第一个文件那一版发，水位恒 19 |
| Z3-a 长度 0 | **打中**（今天的调用方够不着） | 文件后端报 `InputOutput(EINVAL)`，`O_DIRECT` 与稀疏盘交回 `Ok` |
| Z3-b 真 errno 分路 | 没打中 | 七个 errno 分路都对；`EINTR` 不重试一条交主 agent |
| Z3-c 不支持打洞的真文件系统 | 没造出 | 用户命名空间被拒，本机可写的只有 ext4 与 tmpfs |
| Z3-d mkfs 打洞用例在不支持时 | **打中**（红之后留文件） | 该红、红得清楚；但红了不删镜像，每次 1.5 GiB 真占用留在 `/tmp` |
| Z3-e wchar 按线程量 | 没打中 | mkfs 不起线程，文件里只有一条用例 |
| Z3-f SAFETY 注释 | **打中一句**（不碍内存安全） | 「即使越界内核也只返回 errno」不成立：越过文件末尾的打洞交回成功 |
| Z4-① 看起来对的错实现 | **打中三个**（A、D、E） | 只查盘 0 的根槽、不重读、自证得过就算是它：新用例与全包都绿，A、E 在旧快照表上漏拒成 checker 红 |
| Z4-② 造盘面的造法 | 没打中 | 「根槽之前」那组不是崩溃盘面，但它钉的行为在真崩溃态上同样成立（T06 在崩溃态表上红 3 态） |
| Z4-③ rustfmt 重排 | 没打中 | 去掉空白后逐字相同 |
| Z4-④ 变异表 14 行 | 没打中 | 14 行都红在点名用例上；打洞第 1、4、6、7 行在副本上证了 |

## 复跑

模型目录 `research/prompts/m3-code-r1-opus-model/`：`proto/` 是原型 crate（放进副本的 `crates/m3r1-opus-proto/`，工作区 `members` 加一行 `"crates/m3r1-opus-proto",`）；`mutrun.py`、`apply-mutation.py` 套改坏；`run-proto-2.sh`…`run-proto-6.sh` 是各批的原样脚本（路径写死在 `/tmp/claude-1000/m3-code-r1-opus/`，换目录改脚本头的 `R=`）；`logs/`、`logs5/`、`logs6/` 是原样日志。复跑：

```
R=/tmp/claude-1000/<新目录>; mkdir -p $R && cp -r research/prompts/m3-code-r1-opus-model $R/model
rsync -a /tmp/claude-1000/m3-code-r1/tree/ $R/work/ && rsync -a /tmp/claude-1000/m3-code-r1/tree/ $R/work2/ && rsync -a $R/model/proto/ $R/work2/crates/m3r1-opus-proto/
# work2/Cargo.toml 的 members 里加 "crates/m3r1-opus-proto",；各 run-proto-*.sh 的 R= 改成 $R
bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 10 bash $R/model/run-proto-3.sh   # 重读一次、多发布旧快照表（旧版）、几个错实现
bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 10 bash $R/model/run-proto-5.sh   # A/D/E 全包、转圈表、崩溃态表、T06
bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/capped.sh 10 bash $R/model/run-proto-6.sh   # 多发布旧快照表（定稿）
CARGO_TARGET_DIR=$R/target bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 10 python3 $R/model/mutrun.py $R/work $R/model/wrong-impls.tsv $R/logs/wrong
```

源文件 sha256（日志的逐个 sha256 在 `SHA256SUMS-logs.txt`，84 行）：

```
95a133695c868c0367a397bea9f9577ec52f242e0840ed6b930077adf2052d8b  apply-mutation.py
e2e9051a14c9bbf50c7584dc9b988ec935312904175d02cc3ca2aa6c5ad700cc  mutrun.py
2224542d8c7f54ef1f9d05a44d0d6ff78d2142695a85923a6481699bf6b1a282  proto/Cargo.toml
2552be16f45189961caf9e3976f3279168001e4052ff773876c2a58d0ceea7a1  proto/src/bin/with_fallocate_errno.rs
d82cf42f16205812f40b35c1baa0bc117fa026a72c4d5887fe44d974a37fb3c8  proto/src/lib.rs
48ba06a5c2b9a26a3001ee4ee763bb1f0b44a4da21edcd34476e0aeed183174d  proto/tests/z2_every_crash_state_of_a_publish_then_a_writable_mount.rs
a5b6f1829b1154a8233652ea3ad4d293dfb8db98a11c3b8f795f0d3a1cc0cd2f  proto/tests/z2_reread_one_transient_root_slot_read_error.rs
3f0801e969cd190359d59381ed2b2d376c0fe7b8f82a22c02f7b33c013f8da2a  proto/tests/z2_stale_snapshot_from_any_publish_of_a_history.rs
d654eb72dbe6ab880e1de980ae3bbfc3672da63a9b9988e191968a1d9085ccf0  proto/tests/z3_zero_fill_on_the_file_backend_against_the_other_backends.rs
c23f6e8931e2145098889edd8192d78fae60c4a0f8821309d4b63e74331525ee  run-proto-1.sh
1aba11fee0fd390566dc5bf4598cae6f39efb80663970dd9fbf3c1ce3e07fcdc  run-proto-2.sh
686bb93b058c123ef429eaba64bfdfe72c5589904ba336991d897bcc916dd808  run-proto-3.sh
c1f5f5a43fd8bbdd22a4058fec37279be8c888ee01ddd988414b0949d846c5ea  run-proto-4.sh
63da1dae2b4724f84a5867c7683ce1c2f092c21d923b622147ace07e51fb49d9  run-proto-5.sh
42cd0faa8c067bad929dddb497c0eda497277e6840efd442a584d39fda8ffff6  run-proto-6.sh
d9ae59e0fb07da9b7bc61d16767977b24115988c84b08058da3cd17658c36d6f  SHA256SUMS-logs.txt
3a8abd3010afb0aa61d75b5e23df04596d66967d12aa1be19a4022698069f8ba  survivors-full-suite-ade.tsv
f585c474d51bd0a8db24aad75e7d3c5cbb1d4516d8a9d2094404ae6de11c7b1d  survivors-full-suite.tsv
eb74d815cbdef48c210dbb9a3487278261bb386f9e8dc440e8a98eb60c200de7  table14.tsv
da42cb4c9f728054132c4a0945ab2a2dca1612fc0d0c07a4d424e0714895ea4f  wrong-impls.tsv
```

`run-proto-1.sh` 那一批四步都没跑成（原型文件没同步进副本、编不过），`run-proto-4.sh` 的原型那一半作废（见末尾「限度」），两份脚本留着只为对得上日志。

### 我提的改法（都只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 收 Z2-④ 原型进 entries：甲那一形的会话发布 n = 1、可写挂载 n = 8，变异行 `let reread = first_reading;` | Z2-④、Z4-① D | 量过：原样绿、变异红（`logs/reread-orig.log`、`logs/reread-no-reread.log`）；搬进 entries 之后没跑 |
| C585 用例加一版 P 的 txg ≡ 1 (mod 3)（根在盘 1） | Z4-① A | 推的：A 在旧快照表上红是量过的，用例没写 |
| C585 用例加「根环转过一圈」那一版、断言 `AnotherRoot` | Z4-① E | 推的：E 在转圈表上红是量过的，用例没写 |
| `write_zeroes_at` 长度 0 直接交回 `Ok` | Z3-a | 推的 |
| mkfs 打洞用例的镜像包 Drop 守卫 | Z3-d | 推的 |
| SAFETY 注释那一句改写 | Z3-f | 推的（越界打洞交回 0 是量过的） |

## Z2　C585 / C586 哪一格会错

起点照 m2-closeout-code-r3 攻方 X6：两块 384 槽单元区的小盘，mkfs、取号、暖机、第一个文件（txg 3），会话里再覆盖写（各表另注）。崩溃态那张表调 checker 档 `enumerate_layer0_in_state_slices`、每一段都 `full_expansion`（原地覆写取三态），每个状态由枚举器跑 checker 与 oracle，观察者里把那个状态落成盘面、可写挂载一次。全部是冻结副本的拷贝上跑的。

### Z2-①　零轮候选逐个造盘面（没打中：原样代码在多发布历史上零漏拒）

模型 `proto/tests/z2_stale_snapshot_from_any_publish_of_a_history.rs`：历史 = 起点之后 1–3 次发布（覆盖写 / 准入抬 F / 管理员回退 / 建一个 inode，全排列 84 段），录下整段历史的写流；旧快照 = 起点 + 整段流前 k 步之后盘 d 的样子，k 扫遍整段流（只留 d 的盘面变了的 k），d 取 0、1；换上来之后接一个动作（6 种），做成的再接第二个（6 种，卸载之后不接）。只固定前缀与故障，用户动作放开扫。判：拒不拒、拒在写之前没有；放过的每一步之后池级 checker、最后崩了再可写挂载、checker、读回内容。

| 表 | 格数（含第二步） | 放过（第一步） | 拒成换盘 / 落后（第一步） | FLAG | 日志 |
|---|---|---|---|---|---|
| 多发布（1–3 次，84 段历史全跑） | 200664 | 1680 | 29205 | 0 | `logs6/stale-m3-shard*.log` 的 SUMMARY 行（8 片相加） |
| 单发布、根环转过一圈（起点再覆盖写 22 次） | 3708 | 80 | 497 | 0 | `logs5/stale-wrap-orig.log` |

- 候选 3「旧快照取在更早一次发布途中、现行那一版复用了那块盘上全部旧单元」：多发布表里旧快照取自非最后一次发布的格，放过的是 0 格：SUMMARY 里 `accepted_from_an_earlier_publish` 那 100 格全在末尾是「准入抬 F 已在上限、一个字节没写」的历史上（例 `HISTORY ["overwrite", "raise_floor", "raise_floor"] operations=104 publish_ends=[31, 104, 104]`，末两项相等），装置把最后那次真写的发布记成了「前一次」。被拒的都拒在缺单元那一半：每次带文件的发布都重写四个固定点单元，而单元在两块盘上同槽镜像（`units_of_the_version_missing_on_a_device_behind_it` 按同一个 `unit.slot` 读每块盘），之后的任何一次发布都会写出旧快照上没有的单元。「复用了全部旧单元」的现行版今天造不出来（推的：零单元发布只发生在树表 0 条的一版上，第一个文件之后没有零单元发布）。
- 候选 4「P 的根住在另一块盘上、被换的盘是 P 途中取下的、单元都在」：在表里（d 取 0、1 两块，k 扫遍 P 的流），放过的格 checker 0、再挂 0、内容对。这个盘面与「崩在根槽之后、盘 0 轮换没落」那个合法崩溃态只差在本盘的轮换，见 Z4-②。
- 候选 5「环转过一圈之后重核」：转圈那张表 0 格 FLAG、新判法没把合法的盘误拒（现行那一版恒是最新的根，它那一槽不会被下一圈盖掉）。
- 候选 6「只比 (实例, txg)」：见 Z4-① B 那一行，没造出。

### Z2-②　可写挂载传 `NotWritten` 那一支（没打中）

- 施加过记录（`effective_root` ≠ `chosen_root`）说明那几次发布的根读阶段没读到：要么没写（崩在根槽 FUA 之前），要么写了而读不出（实现员排除的那一形）。根槽只写在 `target_for_publish(txg)` 那一槽（`transaction.rs` 的 `WriteRootRecordForceUnitAccess` 与 mkfs 都按它算），择根取最新的读得出的根，所以「写了、读得出、又没被择到」造不出来；没写的那一版哪块盘都不缺它的根。
- 量过的：合法崩溃态表（Z2-③）里施加记录才挂上的那几类状态全部 `Ok`；多发布旧快照表的动作里有「崩了再可写挂载」，放过的格 checker 与再挂都过。
- 被放过的盘缺的只能是别的东西（旧快照上的系统配置、更早的根环槽、journal 记录）；这几样在旧快照表里放开扫过，没有一格 checker 红。

### Z2-③　误拒：合法崩溃态 + 可写挂载（没打中：新判法一个合法崩溃态都没拒）

模型 `proto/tests/z2_every_crash_state_of_a_publish_then_a_writable_mount.rs`。流 = 起点（第一个文件之后覆盖写两次）之后 1–2 个动作，枚举最后那个动作的写流。`logs5/crash-part{0,1,2}.log` 的 FLOW 行原样汇总：

| 流 | 崩溃态 | checker 违例 | 可写挂载结局 | 用时 |
|---|---|---|---|---|
| rollback | 4108 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 5, "Ok": 4103}` | 28.5 s |
| create_inode | 65548 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 5, "Ok": 65543}` | 393.7 s |
| crash_remount | 20511 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 15, "Ok": 20496}` | 130.7 s |
| overwrite+raise_floor | 8223 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 15, "Ok": 8208}` | 53.9 s |
| create_inode+raise_floor | 8223 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 15, "Ok": 8208}` | 53.6 s |
| overwrite+rollback | 4108 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 5, "Ok": 4103}` | 30.8 s |
| overwrite+crash_remount | 20511 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 15, "Ok": 20496}` | 138.6 s |
| raise_floor+rollback | 4108 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 5, "Ok": 4103}` | 28.5 s |
| raise_floor+crash_remount | 20511 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 15, "Ok": 20496}` | 133.0 s |
| rollback+rollback | 4108 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 5, "Ok": 4103}` | 27.6 s |
| rollback+crash_remount | 20511 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 15, "Ok": 20496}` | 142.8 s |
| create_inode+rollback | 4108 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 5, "Ok": 4103}` | 32.9 s |
| crash_remount+rollback | 4108 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 5, "Ok": 4103}` | 30.6 s |
| overwrite+create_inode | 65548 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 5, "Ok": 65543}` | 415.4 s |
| create_inode+crash_remount | 20511 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 15, "Ok": 20496}` | 137.0 s |
| crash_remount+create_inode | 65548 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 5, "Ok": 65543}` | 456.6 s |
| crash_remount+crash_remount | 24618 | 0 | `{"NewerStateStillUnreadableAfterOneReread": 20, "Ok": 24598}` | 173.6 s |

- 拒因里没有一格是 `BehindTheSelectedVersionAnd…`（新判法的两个成员）。合计 364 911 个崩溃态，全部 checker 0 违例、记录核对器 0。
- 每条流都有 5、15 或 20 个状态被拒成 `NewerStateStillUnreadableAfterOneReread`：数等于 5 × 那条流里系统配置槽写那几段的段数（每段两块盘各一槽原地覆写，3² − 2² = 5 个状态里至少一槽撕裂；rollback、create_inode 一段，crash_remount 与「…+raise_floor」三段，crash_remount+crash_remount 四段），是撕裂的系统配置槽（原地覆写的第三态）让见证读不出、按 C331 甲拒——这是改动之前就有的拒因，不归新判法，这一轮没判它该不该拒。
- 覆盖写收尾的五条流（每条 `closed_form_state_count=1048583`）没跑：按 rollback 流量的 5.8 ms/态（观察者里那次可写挂载是单线程），一条约 100 分钟，超过估时上限。准入抬 F 收尾而抬 F 本身没写（已在上限）的五条流没有根槽写，跳过。
- 同一套表在变异表 T06（施加过记录也当写进了根环）下：rollback 那条流 3 个状态被拒成 `BehindTheSelectedVersionAndItsRootNotInItsRingSlot`（`logs5/crash-rollback-T06.log`），即这张表分辨得出 T06。

### Z2-④　重读一次（打中：去掉重读的变异今天全包不红；钉它的用例写好了）

模型 `proto/tests/z2_reread_one_transient_root_slot_read_error.rs`（就是要补的那条用例的原型）：会话里覆盖写 P（txg 6，根在区域 0、盘 0）做完；盘 0 落后、单元都在、根槽里就是 P 的根，两种造法——甲：盘 0 换成切在 P 的根槽 FUA 之后的那一截（合法崩溃态的盘面）；乙：不换盘，盘 0 最新那一槽系统配置翻一个字节（持久单故障）。再加一次瞬时读错：盘 0 上 P 的根槽第 n 次读坏，n 扫遍干净跑时这一槽被读的次数（会话发布 3 次、可写挂载 15 次）。

原样（`logs/reread-orig.log`）：每个 n 都做成，`test result: ok. 1 passed`。去掉重读（`let reread = first_reading;`，`logs/reread-no-reread.log`）：

```
SwappedForTheCutAfterTheRootSlotWrite session_publish fail_read=1 outcome=DevicesBehindTheCurrentVersionAndMissingItsUnits { devices: [DeviceBehindTheCurrentVersion { device: DeviceIdentity(0), newest_system_config
SwappedForTheCutAfterTheRootSlotWrite writable_mount fail_read=8 outcome=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg:
NewestSystemConfigurationSlotCorrupted session_publish fail_read=1 outcome=DevicesBehindTheCurrentVersionAndMissingItsUnits { devices: [DeviceBehindTheCurrentVersion { device: DeviceIdentity(0), newest_system_config
```

- 同一个变异（Z4 表 D）在 harness 全包与 core 全部单测下 94 个测试二进制全绿（`logs5/full-suite.out`）。
- 乙那一形是「一个持久单故障 + 一次瞬时读错」：会话发布第一次读那一槽碰上读错，没有重读就把一块健康的盘拒成落后支，会话所有写入口从此被拒。
- 四句：不分辨臂（判的是测试）；判别子看得到（第 1 次读坏、第 2 次读得出，两次读都在被判函数里）；满足跑前判据「Z2 ④ 去掉重读的变异在瞬时读错的盘面上不红 ⇒ 补用例与变异行」；改法：把这个原型搬进 entries 文件（甲的会话发布一格就够，n = 1；可写挂载那一格 n = 8），变异行替换文 `let reread = first_reading;`。只在我的模型上量过，被攻过零轮。
- 乙上可写挂载本来就按 C331 甲拒（干净跑原样 `NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion …`），原型里不扫那一格。

### Z2-⑤　C586 那条用例换造法（放得过两种错实现，今天分不出）

W1（只取内存里现行那一版）、W3（取回退目标那一版的）在 unit_area 与 rollback 两个文件下全绿（Z4 表）。它们与正确实现今天在任何可达历史上给同一个数（Z4 表 W1 那一行的理由，推的），所以这条用例放过它们不算漏钉；若以后第一个文件之外还会发树号（例如第二个文件、或回退到树表 0 条的一版），要补一条「环里有根的水位高于现行那一版」的用例。

## Z3　打洞哪一格会错

模型：`research/prompts/m3-code-r1-opus-model/proto/tests/z3_zero_fill_on_the_file_backend_against_the_other_backends.rs`（三个后端同一组输入；seccomp 让内核对 `fallocate` 真交回点名的 errno，每个 errno 一个新线程）、`proto/src/bin/with_fallocate_errno.rs`（装同一个过滤器后 exec 别的程序，用来跑原样的 mkfs 打洞用例）。日志在 `logs/z3-backends.log`、`logs/mkfs-test-*.log`。

### Z3-a　长度 0 的写零：文件后端报 I/O 错，另两个后端交回成功（打中，接口语义不同；今天的调用方够不着）

`logs/z3-backends.log` 原样（副本上量的）：

```
ZERO_FILL offset=0 length=0 file=InputOutput(raw_os_error=Some(22), kind=InvalidInput) file_counts=ZeroFillLandingCounts { punched_calls: 0, written_as_zero_bytes_calls: 0 } direct=Ok sparse=Ok
ZERO_FILL offset=1048576 length=0 file=InputOutput(raw_os_error=Some(22), kind=InvalidInput) file_counts=ZeroFillLandingCounts { punched_calls: 0, written_as_zero_bytes_calls: 0 } direct=Ok sparse=Ok
ZERO_FILL offset=4096 length=0 file=InputOutput(raw_os_error=Some(22), kind=InvalidInput) file_counts=ZeroFillLandingCounts { punched_calls: 0, written_as_zero_bytes_calls: 0 } direct=Ok sparse=Ok
```

- 机理：`check_request` 放行长度 0（0 是块大小的整数倍、末端不越界），`fallocate(2)` 对 `len == 0` 交回 `EINVAL`，分路判成「别的错」，照报 `InputOutput`。改动之前文件后端走拆块循环，长度 0 循环零次、交回 `Ok`；`O_DIRECT` 后端今天仍是这样（`direct=Ok`），内存稀疏盘也是。
- 被判对象里许可它的那一句：`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:174`「Some(_) | None => HolePunchFailure::OtherFailure,」；同文件头 `/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:4`「两个后端的持久语义相同」。
- 今天够不着：仓里写零的调用方只有 mkfs 两处（根环三段与 journal 环），长度来自几何、不为 0（`grep -rn 'write_zeroes_at(' crates/singlefs-core/src` 只命中 `make_filesystem.rs` 第 497、499 行与单测）。所以这一格是「同一接口三个后端不同答」，不是今天的历史能走到的错。
- 四句：不分辨臂（只有打洞一条臂）；判别子看得到（长度在调用时就在手里）；满足 Z3 判据「造出打洞与真写零在任何可观测量上不同（读回、录制流…）」里「任何可观测量」那一分句（交回值）；改法（推的，零轮）：`write_zeroes_at` 在 `check_request` 之后、打洞之前加「长度 0 直接交回 `Ok`、两个计数都不动」，或在 `check_request` 里对三个后端一起拒长度 0。只在我的模型上量过现状，改法没实现、被攻过零轮。

### Z3-b　真 errno 下的分路（没打中，照用户定案走；`EINTR` 一条交主 agent）

seccomp 让内核交回 errno（不是测试开关，走的是真系统调用的返回），`logs/z3-backends.log` 原样：

```
ERRNO EOPNOTSUPP outcome=Ok counts=ZeroFillLandingCounts { punched_calls: 0, written_as_zero_bytes_calls: 1 } range_zeroed=true range_untouched=false
ERRNO ENOSYS outcome=Ok counts=ZeroFillLandingCounts { punched_calls: 0, written_as_zero_bytes_calls: 1 } range_zeroed=true range_untouched=false
ERRNO ENOSPC outcome=InputOutput(raw_os_error=Some(28), kind=StorageFull) counts=ZeroFillLandingCounts { punched_calls: 0, written_as_zero_bytes_calls: 0 } range_zeroed=false range_untouched=true
ERRNO EINTR outcome=InputOutput(raw_os_error=Some(4), kind=Interrupted) counts=ZeroFillLandingCounts { punched_calls: 0, written_as_zero_bytes_calls: 0 } range_zeroed=false range_untouched=true
ERRNO EPERM outcome=InputOutput(raw_os_error=Some(1), kind=PermissionDenied) counts=ZeroFillLandingCounts { punched_calls: 0, written_as_zero_bytes_calls: 0 } range_zeroed=false range_untouched=true
ERRNO EIO outcome=InputOutput(raw_os_error=Some(5), kind=Uncategorized) counts=ZeroFillLandingCounts { punched_calls: 0, written_as_zero_bytes_calls: 0 } range_zeroed=false range_untouched=true
ERRNO EINVAL outcome=InputOutput(raw_os_error=Some(22), kind=InvalidInput) counts=ZeroFillLandingCounts { punched_calls: 0, written_as_zero_bytes_calls: 0 } range_zeroed=false range_untouched=true
```

- 七个 errno 都按 `classify_hole_punch_failure` 分对：只有 `EOPNOTSUPP`、`ENOSYS` 退回写 0、段里读回全 0；其余报错、两个计数不动、段里原样是旧字节。
- `ENOSPC` 的后果（量过）：mkfs 照报 `BlockDevice(InputOutput(Os { code: 28, kind: StorageFull, ... }))`，见下面 Z3-d 的 ENOSPC 那一行。用户 2026-10-01 已定照报（`records/2026-09-30-里程碑三开工第一轮.md:184`「用户 2026-10-01 复核时定：ENOSPC 照报错，补丁不改。」），不算打中。
- `EINTR`（交主 agent，没打中）：写的路径用 `write_all_at`，std 碰到 `Interrupted` 自己重试；打洞碰到 `EINTR` 直接报 I/O 错。照用户「只在不支持时退」的字面这是对的，但它让「写零被信号打断」与「写被信号打断」两种结局不同。ext4、tmpfs 的打洞路径会不会交回 `EINTR` 没量（推的：不常见）。

### Z3-c　真「不支持打洞」的文件系统没造出来

`unshare -Urm` 起用户命名空间被拒（原样：`unshare: write failed /proc/self/uid_map: Operation not permitted`），挂不了 ramfs；本机能写的文件系统只有 ext4（`/`）与 tmpfs（`/dev/shm`、`/run/user/1000`），两个都支持打洞；`/boot/efi` 是 vfat 但不可写。所以「不支持」只由 seccomp 造的内核返回值与测试开关进入，没在真不支持的文件系统上跑过。

### Z3-d　`mkfs_on_image_files_punches_the_zeroed_ranges` 在不支持的环境里红成什么样（打中：红时留下镜像文件）

| 环境 | 结局（原样） |
|---|---|
| `TMPDIR=/dev/shm/…`（tmpfs） | `test result: ok. 1 passed`（`logs/mkfs-test-on-tmpfs.log`） |
| fallocate 一律 `EOPNOTSUPP` | 红在第 91 行 `left: 1610925568` / `right: 116224`，消息「经 write 交出去的该恰好是 mkfs 真写的 116224 字节；多出来的是写零真写了 0」（`logs/mkfs-test-fallocate-EOPNOTSUPP.log`） |
| fallocate 一律 `ENOSPC` | 红在第 63 行 `mkfs: BlockDevice(InputOutput(Os { code: 28, kind: StorageFull, message: "No space left on device" }))`（`logs/mkfs-test-fallocate-ENOSPC.log`） |

- 该红：不支持打洞时它红在 wchar 那条断言上、消息直说「写零真写了 0」，与文件头写的一致。
- 打中的是红之后：镜像在 `std::env::temp_dir()` 里建、清理在用例最后一行（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/tests/mkfs_on_image_files_punches_the_zeroed_ranges.rs:128`「std::fs::remove_file(path).expect("清理镜像");」），任何一条断言先红就不清。量到的：`EOPNOTSUPP` 那一次在 `/tmp` 留下两个 4 GiB 稀疏镜像、各实占 769M（`du -sh` 原样 `769M	singlefs-step67-punch-zeroed-ranges-1245543-0-dev0.img`）；变异表第 1 行（T08，见 Z4）证红那一次同样留下两个、各 769M。都是我起的，已删。`/tmp` 里另有一对 `singlefs-step67-punch-zeroed-ranges-275910-{0,1}-dev{0,1}.img`（各实占 769M，2026-09-30 建），不是我建的、没动，交主 agent。
- 后果：这条用例的变异行（`crates/mutations.tsv` 打洞第 1 行）每证一次红，`/tmp` 就多 1.5 GiB 真占用的文件；提交时 checker-tier-crates-mutation-replay 复跑整表也一样。
- 四句：不分辨臂；看得到；满足的是 C603「测试与验证的资源浪费」那一类，不是 Z3 跑前判据里写了的分句（跑前判据没有这一格，按「改法碰不到」之外的新格记）；改法（推的，零轮）：镜像路径包一个 Drop 守卫（drop 时删文件），或建完立刻打开再 `unlink`、只留句柄。

### Z3-e　wchar 按本线程量（没打中）

`make_filesystem`、`RecordingBlockDevice`、文件后端里都没有起线程（`grep -rn 'thread::spawn\|rayon\|scope(' crates/singlefs-core/src/make_filesystem.rs crates/singlefs-core/src/block_device.rs crates/singlefs-harness/src/lib.rs` 零命中），mkfs 的写都在测试线程上；这个测试文件只有一条用例。别的线程的写进不来，本线程里 libtest 捕获输出不走 write。没造出误红、误绿。

### Z3-f　`// SAFETY:` 的前提（打中一条：注释里关于内核的一句不成立；内存安全不受影响）

- 「描述符取自 `file`，而 `file` 借用到这次调用返回」：成立（参数是 `&File`，借用跨过调用）。
- 「偏移与长度由上面的 `try_from` 转成非负的 `off_t`，放不下就在转换处拒」：成立（`u64` 转 `i64`，超过 `i64::MAX` 的拒）。
- 「即使越界内核也只返回 errno」不成立：`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:377`「即使越界内核也只返回 errno。返回值在下面查了。」。本机 ext4 上量的（util-linux `fallocate -p` 即 `PUNCH_HOLE | KEEP_SIZE`），1 MiB 文件：

```
punch past EOF rc=0
punch straddling EOF rc=0
size=1048576
fallocate: fallocate failed: File too large
punch off+len>maxbytes rc=1
```

  越过文件末尾的打洞交回成功、什么都不做，不报 errno；只有越过文件系统最大长度才报 `EFBIG`。今天 `check_request` 先挡住越界，所以没有可达的后果；但 SAFETY 注释把「越界」说成内核兜底，而真兜底的是 `check_request`。拿掉 `check_request` 的错实现（Z4 表 P_C）红在 `writing_zeroes_rejects_unaligned_and_out_of_range_ranges_before_touching_the_file`，即用例钉着这件事。改法（推的，零轮）：把那一句改成「越界由调用方先过的 `check_request` 挡住；内核对越过文件末尾的打洞交回成功、不报错」。

## Z4　测试

装置：`research/prompts/m3-code-r1-opus-model/mutrun.py`（逐条套一个改坏、旧串恰好命中一次、跑点名的 cargo test、记哪些红、还原）、`wrong-impls.tsv`（12 个错实现）、`table14.tsv`（变异表新加的 14 行，参数去掉了过滤词、跑整个靶文件）、`survivors-full-suite-ade.tsv`（A、D、E 跑 harness 全包与 core 全部单测）。全部在仓副本上跑，数是副本上的。

### Z4-① 看起来对、其实错的实现（打中三个：A、D、E 新用例全绿而行为错；B、C、W1、W3 放得过但今天的历史分不出）

先跑三个靶文件（entries、`fsync_drop_and_devices_without_the_selected_version`、unit_area；C586 的两个跑 unit_area 与 `rollback_by_a_forward_publish`），`logs/wrong.out` 原样：

```
MUT A_only_device_zero	GREEN	rc=0	failed=[]	results=3
MUT B_txg_only	GREEN	rc=0	failed=[]	results=3
MUT C_mount_never_root	GREEN	rc=0	failed=[]	results=3
MUT D_no_reread	GREEN	rc=0	failed=[]	results=3
MUT E_any_self_verified_root	GREEN	rc=0	failed=[]	results=3
MUT W1_current_only	GREEN	rc=0	failed=[]	results=2
MUT W3_target_only	GREEN	rc=0	failed=[]	results=2
MUT P_A_fallback_first_chunk_only	RED	rc=101	failed=['block_device::tests::an_unsupported_hole_punch_falls_back_to_writing_zero_bytes_across_chunk_boundaries']	results=1
MUT P_B_punch_short_by_a_sector	RED	rc=101	failed=['block_device::tests::punching_ranges_off_the_filesystem_block_boundaries_reads_back_zero_keeps_the_neighbours_and_the_file_length', 'block_device::tests::writing_zeroes_clears_the_whole_range_across_chunk_boundaries_and_nothing_outside_it']	results=1
MUT P_C_no_check_request	RED	rc=101	failed=['block_device::tests::writing_zeroes_rejects_unaligned_and_out_of_range_ranges_before_touching_the_file']	results=1
MUT P_D_punched_not_counted	RED	rc=101	failed=['block_device::tests::punching_ranges_off_the_filesystem_block_boundaries_reads_back_zero_keeps_the_neighbours_and_the_file_length']	results=1
MUT P_E_punch_then_also_write	RED	rc=101	failed=['block_device::tests::punching_ranges_off_the_filesystem_block_boundaries_reads_back_zero_keeps_the_neighbours_and_the_file_length']	results=1
```

A、D、E 再跑 harness 全包与 core 全部单测（94 个测试二进制），`logs5/full-suite.out` 原样（基线不改源码那一次在 `logs/full-suite.out`：`MUT BASE	GREEN	rc=0	failed=[]	results=94`）：

```
MUT A_only_device_zero	GREEN	rc=0	failed=[]	results=94
MUT D_no_reread	GREEN	rc=0	failed=[]	results=94
MUT E_any_self_verified_root	GREEN	rc=0	failed=[]	results=94
```

| 名 | 改哪一行（冻结副本 `mount.rs`） | 新用例 | 它错在哪（量过 / 推的） |
|---|---|---|---|
| A 落后支只查盘 0 | 第 3416 行起取根槽归属盘那两行换成 `region_devices[0]` | harness 全包绿 | **量过**：单发布旧快照表在 A 下 168 格 FLAG（`logs/stale-m1-A_only_device_zero.log`：164 格 I-3.1、4 格 I-3.9），原样 0 格；漏的都是 `d=1`、txg ≡ 1 (mod 3) 的那一版（区域 1 归盘 1：`make_filesystem.rs` 第 63 行 `[DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)];`）。新用例的 P 是 txg 6、根在区域 0、盘 0，A 照拒 |
| D 不重读 | 第 3444 行 `let reread = first_reading;` | harness 全包绿 | **量过**：见 Z2-④，一次瞬时读错就拒成落后支 |
| E 根槽里自证得过就算是它 | 第 3429 行守卫改成恒真 | harness 全包绿 | **量过**：根环转过一圈之后（起点再覆盖写 22 次，环 24 槽）单发布旧快照表在 E 下 226 格 FLAG、原样 0 格（`logs/stale-wrap-E.log` 与 `logs5/stale-wrap-orig.log` 的 SUMMARY 行）。新用例的根槽是 mkfs 清过零的槽，两次读都是 `NotSelfVerified`（entries 第 842 行），分不出「读坏」与「上一圈的旧根」 |
| B 只比 txg 不比实例 | 第 3429 行去掉实例那一半 | 三个靶文件绿 | 单发布表、两发布表都 0 格 FLAG（`logs/stale-m1-B_txg_only.log`）。要分辨得造「同一 txg、不同实例」的两条根落在同一槽：上一个实例写过 txg T 的根、重挂时没择到它（瞬时读不出）、新实例重发 txg T，再把新实例 T 途中取下的盘换上来——**推的**，没造 |
| C 可写挂载一律不核缺根 | 第 3993 行条件前加 `false &&` | 三个靶文件绿 | 两张表都 0 格 FLAG。可写挂载核缺根只在「择到的根就是所选那一版、那一槽读两次都坏」时拒（实现员报告线索第 1 条），分辨它要两次瞬时读错；Z2-④ 第 8 次读那一格是 D 下才拒的单次读错，C 下不拒（推的，没在 C 下跑那张表） |
| W1 回退水位只取内存里现行那一版 | 第 5386 行 `.fold(…, \|kept, _ring\| kept)` | unit_area 与 rollback 两文件绿 | 今天的可达历史分不出（推的）：树 ID 只在第一个文件那一版发号（冻结副本 `transaction.rs` 里 `issued_from_watermark` 在测试模块之外的调用方两处：`publish_first_file`，与 `publish_version_with_unmount_marker` 没有上一版那一支——mkfs 那条流、从 11 发、水位 19），之后每一版照抄 19，回退目标都带文件（树表 0 条的目标被拒成 `VersionWithoutFile`），环里没有根的水位会高过现行那一版 |
| W3 回退水位取目标那一版的 | 同一行换成 `target_version.root.tree_identifier_watermark` | 同上 | 同 W1 |

- A、D、E 的四句：分不分辨臂——这里没有臂，判的是测试；判别子看得到——A 要 P 的根住在盘 1 上（txg ≡ 1 mod 3），D 要一次瞬时读错，E 要根槽里有上一圈的旧根（环转过一圈），三样新用例都没有，盘面在用例手里是看得到的；满足 Z4 判据「写出一种错实现、新用例全绿」；改法（推的，零轮）：A——四个入口的 C585 用例各加一版 P 的 txg ≡ 1 (mod 3)（会话里再覆盖写一次，P 取 txg 7）；D——收 Z2-④ 的用例；E——起点先把环转过一圈（384 槽几何每区 8 槽，覆盖写 22 次够），断言 `first_reading` 是 `AnotherRoot`。
- 打洞那五个错实现都被现有单测红掉。题面举的「打洞一律当成功不调系统调用但仍计数」就是变异表打洞第 4 行（T11），「退路只写第一块」是 P_A，都红。

### Z4-② 造盘面的造法（没打中）

- 切在根槽 FUA 之后那一组（`AfterItsRootSlotWrite`）是真崩溃留得下的盘面：轮换在 `CommitStep::RotateSystemConfigurationSlots` 里对每块盘各写一槽、写完才发屏障（冻结副本 `transaction.rs` 第 319–331 行），同一段里「盘 1 的轮换落了、盘 0 的没落」是一个合法子集。
- 切在根槽 FUA 之前那一组不是：盘 1 停在 P 做完（轮换已落），而轮换在根槽 FUA 那一段关闭之后才发，崩溃留不下「根槽没落而别的盘轮换落了」。它是换盘造出来的盘面——C585 本来就是换盘那一题，四条「拒」的用例用它是对的。
- 「可写挂载照常施加记录挂上」那一条（`a_writable_mount_onto_the_root_device_taken_before_…`）钉在这个非崩溃盘面上，但它钉的行为在真崩溃盘面上也成立、也会被同一行变异打红：合法崩溃态枚举（Z2-③）在 rollback 那条流上，原样 `Ok: 4103`，套变异表 T06（施加过记录也当写进了根环）之后有 3 个崩溃态被拒成 `BehindTheSelectedVersionAndItsRootNotInItsRingSlot`（`logs5/crash-rollback-T06.log` 那一行 FLOW）。所以它钉住的行为不是假的；可以把它换成真崩溃态的造法（崩在记录之后、根槽之前），不靠换盘。

### Z4-③ entries 文件整份 rustfmt（没打中）

把 fcc4d340 的原文件过一遍 `rustfmt --edition 2021`，再与冻结副本比：原文件去掉全部空白之后的 md5 与格式化之后的相同（`9f492ff67e99e57d94c66dce3304ec8a`，两次都是），即格式化只动了空白；格式化之后与新文件比，原有部分只差 5 行，全是新增的 `use` 与 `assert_refused_as_behind` 里 `units_missing` → `lacking` 那一处（改成 `matches!(…UnitsMissing(units_missing) if !units_missing.is_empty())`，比原来更严）。原有用例的语义没被重排改掉。比对产物在 `z4fmt/`。

### Z4-④ 变异表 14 行（都红在点名的用例上；打洞第 1、4、6、7 行在副本上证了）

`logs/table14.out`（T01–T04 替换文相同，只跑 T01，整个 entries 文件里红的正是四个入口那四条）：

| 行 | 点名用例 | 红的（原样摘自日志） | 替换文是不是退回了旧行为 |
|---|---|---|---|
| T01–T04 | 四个入口各一条 | 四条 `…onto_the_root_device_taken_before_the_publish_root_slot_write_is_refused_before_any_write` | 是：`.filter(\|_\| false)` 让缺根那一半恒不拒，等于改动之前「落后且缺单元」 |
| T05 | `…taken_after_the_publish_root_slot_write_is_not_refused` | 只红它 | 不是旧行为，是另一种错（落后而根在这块盘上一律拒），名字照实写了 |
| T06 | `a_writable_mount_onto_…_applies_the_publish_record_and_mounts` | 只红它 | 是另一种错（施加过记录也核缺根），名字照实写了 |
| T07 | C586 那条 | 只红它 | 是：只取环里读得出的根 |
| T08（打洞第 1 行，没证过） | mkfs 打洞用例 | 红在第 91 行 `left: 1610925568` / `right: 116224` | 是：每次退回逐块写 0 |
| T09 | 退路跨块那条 | 只红它 | — |
| T10 | 非对齐打洞那条 | 只红它 | — |
| T11（第 4 行，没证过） | 非对齐打洞那条 | 它与 `writing_zeroes_clears_the_whole_range_across_chunk_boundaries_and_nothing_outside_it`，前者第 874 行 `left: 195` / `right: 0` | 是：交回成功、一个洞都没打 |
| T12 | EIO 那条 | 它与分路表那条 | — |
| T13（第 6 行，没证过） | EIO 那条 | 只红它，第 995 行 | 是 |
| T14（第 7 行，没证过） | 分路表那条 | 只红它，第 1031 行 `left: OtherFailure` / `right: PunchingUnsupported` | 是 |

T08 每证一次就在 `/tmp` 留下两个各实占 769M 的镜像（Z3-d）。

## 没打中的形状

| 试过的形状 | 取样范围 | 结果 |
|---|---|---|
| 旧快照取自 1–3 次发布历史里任一次发布的任一步，换盘 0 或盘 1，接 1–2 个用户动作 | 84 段历史全跑（覆盖写 / 准入抬 F / 管理员回退 / 建一个 inode 全排列），k 扫遍整段写流，动作 6 × 6 | 200 664 格，0 格 FLAG |
| 同上，起点再覆盖写 22 次（根环 24 槽转过一圈） | 单发布 4 种 | 3 708 格，0 格 FLAG |
| 合法崩溃态 + 一次可写挂载 | 17 条流（起点之后 1–2 个动作，最后那个动作的写流全量展开），364 911 态 | 新判法 0 拒；checker 0 |
| 一次瞬时读错在落后、根在本盘、根槽里是它的盘上 | 会话发布每一次读那一槽（3 次）、可写挂载每一次（15 次）、两种落后的造法 | 原样 0 拒 |
| 七个真 errno | `EOPNOTSUPP`、`ENOSYS`、`ENOSPC`、`EINTR`、`EPERM`、`EIO`、`EINVAL` | 分路都对 |
| 打洞的五个错实现 | 少写块、少打扇区、不过 `check_request`、不计数、打了还写 | 都被 core 单测红掉 |
| 14 行变异的点名 | 全部 | 都红在点名用例上 |

## 这条腿自己的限度

- **第四批原型那一半作废**：`apply-mutation.py` 换回原文件用的是 `os.replace`，换回的文件 mtime 停在变异之前，cargo 认为 T06 编出的产物仍新、没重编，第四批的崩溃态三片与转圈表跑的是 T06 下的二进制（那一批里每条 rollback / create_inode 流都多出 3 个 `BehindTheSelectedVersionAndItsRootNotInItsRingSlot`，与第三批原样 rollback 流的 `Ok: 4103` 对不上，才查出来）。第五批起摸新全部源文件强制重编、换回时 `os.utime`；作废的日志没拷进模型目录。第三批在 T06 之前跑的几张表不受影响（每次套变异都写出新 mtime）。
- 单发布旧快照表在 A、B、C、E 下的那几份日志（`logs/stale-m1-*.log`，实际是两发布表）用的是内容表修正之前的模型：第二个位置起的管理员回退会选到 mkfs 的根、被拒成 `VersionWithoutFile`，那几段历史跳过。A、E 的漏拒都出在第一个位置就做完的历史上，不受影响；原样代码的定稿表（`logs6/`）是修正之后跑的，84 段全跑。
- 覆盖写收尾的五条崩溃态流（每条 1 048 583 态）没跑，见 Z2-③；崩溃态表只挂了一次可写挂载，没在挂上之后再发布、再崩。
- 旧快照表只换一块盘、只取「起点 + 流的前缀」，没取「前缀之外再任选子集」的撕裂盘面；动作只放开两步。
- 原型都在两块盘、384 槽单元区或 4 GiB 的几何上；多于两块盘的池（region 归属不是 [0, 1, 0]）没造。
- B（只比 txg）、C（可写挂载不核缺根）的分辨盘面要两次瞬时读错或「同 txg 不同实例」，没造，那两行的「今天分不出」是推的。
- 「不支持打洞」的真文件系统没造出（Z3-c）；ext4 打洞之后的持久性没量（要掉电或虚机）。
- harness 全包只对 A、D、E 跑了；B、C、W1、W3 只跑了三个靶文件。全包基线一次（`logs/full-suite.out`）与 A 的第一次全包（被后台时限停在第 51 个二进制，`logs/full-suite/A_only_device_zero.log`）在第四批；第五批 A、D、E 各跑完 94 个。
- 原型跑的状态合计：崩溃态 364 911（原样，第五批）+ 4 108（T06，第五批）+ 4 108（原样 rollback，第三批）+ 4 108（T06，第三批）+ 作废的第四批 364 911，约 74.2 万，单次最多 176 225，都在约 10⁶ / 10⁷ 之内。
- 第四批被后台时限（默认 30 分钟）停掉一次；第六批第一次起跑被内存包装拒成 252（`RUN_WITH_MEMORY_CAP_WAIT_SECONDS=0`、slice 被别的会话占着），设等待 3600 秒、上限改 4G 后排上。
- `ps` 看到的负载：开工时别的会话的 `cargo test`（`checker_known_bad_images` 等）与广谱变异的 `cargo nextest … -j 31`（24G）在跑，没有性能测量；我这边一律 `nice -n 19`。

## 没做什么

- 不判 Z1（正推那一格）；不替主 agent 采纳；副本上的数不算入库装置上的数。
- 重型测试没跑：checker 档包的测试、层 0 门禁、QEMU、herd7、变异整表、广谱变异。
- 没改主工作区与冻结副本的任何文件。

## 删掉的

- 删了：仓副本 `/tmp/claude-1000/m3-code-r1-opus/work`（11M）、`/tmp/claude-1000/m3-code-r1-opus/work2`（11M）、编译目录 `/tmp/claude-1000/m3-code-r1-opus/target`（12G）、`/tmp/claude-1000/m3-code-r1-opus/target2`（1.9G）、空目录 `images`、`mnt-ramfs`（各 4.0K）；我起的用例红了留在 `/tmp` 的四个镜像（`singlefs-step67-punch-zeroed-ranges-1189842-*`、`-1245543-*` 各实占 769M，`-1245678-*` 实占 0）。
- 留着：草稿目录里的日志、模型与报告分段（模型目录已拷进仓）；`/tmp` 里 `singlefs-step67-punch-zeroed-ranges-275910-{0,1}-dev{0,1}.img`（2026-09-30 建，不是我的）。
