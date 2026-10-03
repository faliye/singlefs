# m3-code-r3 云端攻方腿（Opus）报告：W2（新用例放不放得过错实现）、W3（两块盘以外的几何）

日期 2026-10-01。被判代码：冻结副本 `/tmp/claude-1000/m3-code-r3/tree/`（开工时对 `research/prompts/m3-code-r3-snapshot/crates-sha256.txt` 的 `sha256sum -c` 不 OK 的 0 行），整份拷进草稿目录的副本上编、跑；下文的数都是副本上的，不是入库装置上的。只跑了 harness 档测试二进制与 `singlefs-core --lib`；checker 档、层 0、变异整表、广谱变异一样没跑；原型没调层 0 的枚举函数（盘面都是换盘与注入读错，崩溃态 0 个）。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| W2-① 第二轮攻方原样的六个错实现 | 没打中（都被新用例抓到） | RG 红 J1 那 1 条；RP 红 J2 那 3 条；EC 红 J3 那 6 条；AC 红 J3 txg 25 那 3 条；Z0 红 core 长度 0 单测与稀疏盘对拍各 1 条；E1 红 core k = 2 那 1 条 |
| W2-② 稀疏盘那一行 Z0 证红 | 证红 | `prove-red.sh` 在副本上「跑了 1 条，跳过 0 条，跑的都抓到了」，红在点名用例 |
| W2-③ 新错实现 RU「缺根那一半只在第一次读不出时重读」 | **打中**（量过） | entries 31 条、稀疏盘 2 条、core 139 条、harness 全包 90 个二进制全绿；根槽一次读回字节坏一位时，注入真发生的 146 格原样全做成，RU 下其中 21 格把健康盘拒成落后支缺根（五个入口加可写挂载、txg 6/7/8 都中） |
| W2-④ 新错实现 RGC「卸载、准入抬 F、回退那条路上把区域号当盘号」 | **打中**（量过） | 同样全绿；J1 只在会话发布上取了区域 2，三个入口只取了区域 0、1；区域 2 且根环转过一圈（txg 26/29/32）时三个入口 10 格做成、池级 checker 红 I-3.1 |
| W3-① 三块盘时 txg 25、27 的根落哪块盘 | 今天造不出三块盘 | mkfs 交三块盘 panic（三种归属都是），两块盘归属不是 0 / 1 / 0 拒，挂着的池多交一块盘四个入口都拒；归属取 0 / 1 / 2 时 txg 25 → 盘 1、txg 27 → 盘 0、txg 8 → 盘 2（按 `target_for_publish` 现算） |
| W3-② 用例取的 txg 在三块盘上还敏不敏感 | 不算第三类（取样域里没有三块盘）；记一笔零轮 | 改过 mkfs 断言的三盘副本上量：AC 照样只被 txg 25 抓、txg 27 抓不到；RG 在 0 / 1 / 2 上与对的同值；两块盘上与对的同值的 MOD2、FV 在三块盘上让会话发布在 txg 8 做成、区域 2 转圈那几格五个动作全做成——J1–J3 取的 txg 在两块盘上一个都分不出这两种 |

## 复跑

模型目录 `research/prompts/m3-code-r3-opus-model/`；脚本里的路径写死在 `/tmp/claude-1000/m3-code-r3-opus/`（副本 `work/`、三盘副本 `work3/`、证红副本 `work-pr/`，编译目录 `target/`、`target3/`、`target-pr/`）。`wrongs_r2.py` 是第二轮攻方 `wrongs.py` 的原样拷贝（`cmp` 逐字节相同），`wrongs3.py` 从它取 RG、RP、EC、AC、Z0、E1 六条原样、另加 RU、RGC、MOD2、FV；`wrongrun3.py apply|restore <副本> <冻结副本> <名>` 套上与写回。

```
D=/tmp/claude-1000/m3-code-r3-opus; R=research/scripts; M=research/prompts/m3-code-r3-opus-model
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="m3-code-r3 攻方腿与同会话别的子 agent 各跑各的"
mkdir -p $D/rc && rsync -a /tmp/claude-1000/m3-code-r3/tree/ $D/work/ && cp -r $M $D/model
T=$D/work/crates/singlefs-harness/tests
cat $T/entries_after_mount_refuse_swapped_or_behind_devices.rs $D/model/proto_r3opus_append.rs $D/model/proto_r3opus_w3_append.rs > $T/r3opus_entries_proto.rs
# W2：六个原样错实现 + RU、RGC，各跑 entries、稀疏盘对拍、core --lib
bash $R/run-with-memory-cap.sh 8G bash $R/capped.sh 12 bash $D/model/run-wrongs3.sh BASE RG_region_number_as_device RP_no_reread_in_the_three_other_entries EC_any_self_verified_root_in_the_three_other_entries AC_only_region_zero_device_in_the_three_other_entries Z0_zero_length_before_the_request_check E1_reinterrupted_punch_reported RU_reread_only_after_a_read_error RGC_region_number_as_device_in_the_three_other_entries
# 后果原型（BASE 全跑；RU 只跑 ru、RGC 只跑 rgc）与 harness 全包
bash $R/run-with-memory-cap.sh 8G bash $R/capped.sh 12 bash $D/model/run-proto3.sh 200 BASE all
bash $R/run-with-memory-cap.sh 8G bash $R/capped.sh 12 bash $D/model/run-proto3.sh 201 RU_reread_only_after_a_read_error ru
bash $R/run-with-memory-cap.sh 8G bash $R/capped.sh 12 bash $D/model/run-proto3.sh 202 RGC_region_number_as_device_in_the_three_other_entries rgc
bash $R/run-with-memory-cap.sh 8G bash $R/capped.sh 12 bash $D/model/run-full-par3.sh 300 COMBO_RU_RGC
# 稀疏盘那一行 Z0 证红（单独一份副本、单独的编译目录）
rsync -a /tmp/claude-1000/m3-code-r3/tree/ $D/work-pr/
CARGO_TARGET_DIR=$D/target-pr PROVE_RED_LOG_DIRECTORY=$D/logs/prove-red bash $R/prove-red.sh --copy $D/work-pr --memory 1300M singlefs-harness 'm3-code-r2 J4：文件后端长度 0 的早退挪到请求判据之前（稀疏盘对拍：越过末尾的长度 0 清零文件后端交 Ok、稀疏盘拒）'
# W3 三盘副本：打 w3-three-device-patch.diff（mkfs 断言放开到 3、起点按 R3_THREE_DEVICES / R3_LAYOUT 建池、切盘那一步按镜像里的盘建表）
rsync -a /tmp/claude-1000/m3-code-r3/tree/ $D/work3/ && (cd $D/work3 && patch -p5 < $D/model/w3-three-device-patch.diff)   # diff 是 -ruN 两个绝对路径，-p5 在拷好的冻结副本上 --dry-run 过
T3=$D/work3/crates/singlefs-harness/tests
cat $T3/entries_after_mount_refuse_swapped_or_behind_devices.rs $D/model/proto_r3opus_w3_three_append.rs > $T3/r3opus_w3_three.rs
bash $R/run-with-memory-cap.sh 8G bash $R/capped.sh 12 bash $D/model/run-w3-three.sh 100 BASE AC_only_region_zero_device_in_the_three_other_entries RG_region_number_as_device RGC_region_number_as_device_in_the_three_other_entries MOD2_region_modulo_two FV_first_version_layout_hard_coded EC_any_self_verified_root_in_the_three_other_entries
python3 $D/model/sum-w3.py
```

模型目录里每个源文件的 sha256（日志逐份的 sha256 在 `SHA256SUMS-logs.txt`，193 行）：

```
a7072bfcc4cf2ad70fe92032c4e7bcedf838943a95b0ae70c10ad767a0d6be1e  sum-w3.py
e009a5f471bc4d24d609100a6cec672f0b73951291057ef530f2ae79c53a24ce  wrongrun3.py
7255993f1179fba965e941da9f29ab965f41a87bbb0ec262be01e7992bd2986d  wrongs3.py
1b9ef610e54fe6c7b1065b24d689c625ccda4b713230d2507fa8853cf355d286  wrongs_r2.py
8e68b47b5f1395aa45b585b93a9fa6636571769d8188e9c17b3d67741dac6e01  run-full-par3.sh
3974a8fb25b80287bd137134da50eef8af0e7f679dbc09d613fbc24233b46ada  run-proto3.sh
7b980bfa003b8bedda830a5d991778bf1a2ecbd41e090627333d479d392489e0  run-w3-three.sh
2030ce61a8c6e7f918d2ab7e9b62ea6c9fef2aeadc71c25e157903eab4edae1a  run-wrongs3.sh
4d98c9fe9235c2bd411608dd9597f720ed175a2539552d91cfc00d85efc2295d  proto_r3opus_append.rs
13deff9f7219a0d597776be629517eb96717562bf94de9ced30f1416b89a22a3  proto_r3opus_w3_append.rs
e62ff3b65ae32a9a8ccf83fc16561d16618c37d2879196d3aa9c782961eb953f  proto_r3opus_w3_three_append.rs
da7e000d96859210ddbf3c223f90fefc8af0d4d626b809fbb6d6edd7a3b2613b  w3-three-device-patch.diff
923d1a92d32c58024561a41b8dc3a97ee3f9e729408fe610312ba4748051af95  SHA256SUMS-logs.txt
```

`wrongs_r2.py` 的 sha256 与第二轮攻方报告里 `wrongs.py` 那一行（`1b9ef610…`）相同。

## W2　新用例放不放得过错实现

装置：冻结副本整份拷进 `/tmp/claude-1000/m3-code-r3-opus/work/`，错实现由 `wrongs3.py` 列出、`wrongrun3.py` 套上，跑完从冻结副本写回（每批末尾 `diff -rq` 只剩原型文件 `r3opus_entries_proto.rs`）。每个错实现跑三样：entries 整个文件（31 条）、稀疏盘对拍那份（2 条）、`singlefs-core --lib`（139 条）。

### W2-①　第二轮攻方原样的六个错实现（没打中：都被新用例抓到）

六条的原文在这一轮冻结副本里各恰好命中一次（`wrongrun3.py apply` 先核）。基线与红了的那几行（`logs/w2-batch1.out` 原样摘行，命令 `grep -E '^WRONG (BASE|RG_|RP_|EC_|AC_|Z0_|E1_)' … | grep -v seconds | grep -E 'rc=101|BASE'`）：

```
WRONG BASE entries_after_mount_refuse_swapped_or_behind_devices rc=0 test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2 failed=[]
WRONG BASE sparse_devices_refuse_requests_outside_the_device_like_the_file_backend rc=0 test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0. failed=[]
WRONG BASE core-lib rc=0 test result: ok. 139 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in  failed=[]
WRONG RG_region_number_as_device entries_after_mount_refuse_swapped_or_behind_devices rc=101 test result: FAILED. 30 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished  failed=[a_session_publish_onto_device_zero_holding_the_publish_root_in_region_two_taken_before_its_root_slot_write_is_refused_before_any_write ]
WRONG RP_no_reread_in_the_three_other_entries entries_after_mount_refuse_swapped_or_behind_devices rc=101 test result: FAILED. 28 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished  failed=[a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot ]
WRONG EC_any_self_verified_root_in_the_three_other_entries entries_after_mount_refuse_swapped_or_behind_devices rc=101 test result: FAILED. 25 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished  failed=[a_normal_unmount_onto_device_one_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write a_normal_unmount_onto_the_root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write a_floor_raise_onto_device_one_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write a_floor_raise_onto_the_root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write an_administrator_rollback_onto_device_one_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write an_administrator_rollback_onto_the_root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write ]
WRONG AC_only_region_zero_device_in_the_three_other_entries entries_after_mount_refuse_swapped_or_behind_devices rc=101 test result: FAILED. 28 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished  failed=[a_normal_unmount_onto_device_one_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write a_floor_raise_onto_device_one_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write an_administrator_rollback_onto_device_one_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write ]
WRONG Z0_zero_length_before_the_request_check sparse_devices_refuse_requests_outside_the_device_like_the_file_backend rc=101 test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished i failed=[a_sparse_block_device_accepts_and_refuses_every_request_exactly_like_the_file_backend ]
WRONG Z0_zero_length_before_the_request_check core-lib rc=101 test result: FAILED. 138 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished failed=[block_device::tests::a_zero_length_zero_fill_ends_the_same_on_the_file_backend_and_both_arms_of_the_direct_backend ]
WRONG E1_reinterrupted_punch_reported core-lib rc=101 test result: FAILED. 138 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished failed=[block_device::tests::a_hole_punch_interrupted_by_a_signal_twice_in_a_row_is_punched_a_third_time_without_falling_back_or_reporting_an_error ]
```

同一批里没列出的格都是 `rc=0` 全过：RG、RP、EC、AC 在稀疏盘与 core 上全过；Z0 在 entries 上全过；E1 在 entries 与稀疏盘上全过。逐条对第二轮判决要的「必须红」：

| 错实现 | 判决要它红 | 副本上红的 |
|---|---|---|
| RG 把区域号当盘号 | J1 | J1 那 1 条（txg 8，区域 2） |
| RP 卸载、抬 F、回退不重读 | J2 | J2 那 3 条 |
| EC 那条路上自证得过就算是它 | J3 | J3 的 6 条（txg 27、25 各三个入口） |
| AC 那条路上只查区域 0 那块盘 | J3 | J3 txg 25 那 3 条（txg 27 的根就住区域 0 那块盘，抓不到，用例文件头已写明） |
| Z0 长度 0 早退挪到请求判据之前 | J4 | core 长度 0 单测 1 条、稀疏盘对拍 1 条 |
| E1 被打断第二次就报错 | J5 | core k = 2 那 1 条 |

### W2-②　稀疏盘那一行 Z0 证红

单独一份副本 `work-pr`、单独的编译目录，`prove-red.sh` 施加 `crates/mutations.tsv` 里那一行（名字见复跑），原样输出（`logs/prove-red.out`）：

```
m3-code-r2 J4：文件后端长度 0 的早退挪到请求判据之前（稀疏盘对拍：越过末尾的长度 0 清零文件后端交 Ok、稀疏盘拒）	抓到	a_sparse_block_device_accepts_and_refuses_every_request_exactly_like_the_file_backend 红了（日志 /tmp/claude-1000/m3-code-r3-opus/logs/prove-red/001.log）
✓ 点名 1 条：跑了 1 条，跳过 0 条，跑的都抓到了
rc=0
```

红的那一处（`logs/prove-red/001.log` 第 14–17 行原样）：

```
thread 'a_sparse_block_device_accepts_and_refuses_every_request_exactly_like_the_file_backend' (1453942) panicked at crates/singlefs-harness/tests/sparse_devices_refuse_requests_outside_the_device_like_the_file_backend.rs:158:9:
assertion `left == right` failed: 清零：偏移 2097152 长度 0
  left: OutOfRange { offset: DeviceOffsetInBytes(2097152), length: 0, device_size: 1048576 }
 right: Accepted
```

前两次没跑成、一行没跑：第一次退 252（同一会话已有一条日常的在跑，补设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`），第二、三次退 251（上限超过峰值表里实测峰值的 2 倍；第二次之后峰值表被我自己的基线更新成 674M，上限降到 1300M 才过）；第一次的输出我重跑前删了，第二、三次的留在 `logs/prove-red-1/`（上限 8G）、`logs/prove-red-2/`（上限 1700M）。

### W2-③　新错实现 RU：缺根那一半只在第一次「读不出」时重读（打中，量过）

改哪里：冻结副本 `/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-core/src/mount.rs:3444`「let reread = reading_other_than_the_version_root()?;」换成「第一次读到的是 `Bad(Unreadable)` 才重读，否则 `reread = first_reading`」（替换文在 `wrongs3.py`）。会话发布、可写挂载、卸载、准入抬 F、回退走的是同一处。

它错在哪：被判函数自己的文档写「读那一槽一次，不是它（读坏、或自证得过而是另一条根）就重读一次」（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-core/src/mount.rs:3396`），D16（发布语义）已定项 1 同一行写「这一次读不出或自证不过就重读一次」（`.claude/kb/decisions/16-发布语义.md:37`）。同一条款的兄弟实现在历史上就错成过这一形：「合入后验证一收窄成只管读不出、之后由改法 B 还原成全句」（`.claude/kb/decisions/16-发布语义.md:70`）。

为什么新用例放得过：J2 与 H1 注入的都是 `InjectedFault::ReadFails`（entries 用例文件第 1177 行，`with_reads_of_the_publish_root_slot_failing` 写死的），第一次读到的恒是 `Unreadable`，RU 照样重读；切在根槽之前那几条两次读到的本来就相同（`NotSelfVerified` 或 `AnotherRoot`），RU 不重读、把第一次抄给 `reread`，断言逐字相同。

量过的（`logs/w2-batch1.out` 原样摘行）：

```
WRONG RU_reread_only_after_a_read_error entries_after_mount_refuse_swapped_or_behind_devices rc=0 test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2 failed=[]
WRONG RU_reread_only_after_a_read_error sparse_devices_refuse_requests_outside_the_device_like_the_file_backend rc=0 test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0. failed=[]
WRONG RU_reread_only_after_a_read_error core-lib rc=0 test result: ok. 139 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in  failed=[]
FULL COMBO_RU_RGC streams=4 binaries=90 rc_files=90 nonzero=[] core_lib=test result: ok. 139 passed
FULL COMBO_RU_RGC failed_tests=[]
```

`COMBO_RU_RGC` 是 RU 与 RGC 一起套（改的行不重叠），harness `tests/` 下 90 个测试二进制（原型那份除外）4 路并行、各 `--skip r3opus --test-threads 3`，加 core 单测，全过；用时 249 秒。组合全绿推得出单套也全绿，除非两者互相掩盖同一条用例的红（推的）。

后果原型 `r3opus_ru_one_corrupted_read_of_the_publish_root_slot`：P 的 txg 取 6、7、8（根住盘 0、1、0），P 的根住的那块盘切在根槽 FUA 之后（落后、单元都在、根槽里是 P，健康盘），整池第 n 次读 P 的根槽时读回的字节翻一位（`InjectedFault::ReadReturnsCorruptedBytes`，字节 100 位 0，只这一次），n 从 1 扫到那一槽读不到第 n 次为止（≤ 12）；用户动作放开：会话发布、卸载、准入抬 F、回退（目标取第一个文件那一版与现行前一版两种）、可写挂载。每行之后跑池级 checker。命令数出来的汇总：

```
BASE cells=158 ok=158 refused_behind=0 checker_nonzero=0
RU_reread_only_after_a_read_error cells=158 ok=137 refused_behind=21 checker_nonzero=0
BASE fired1=146 fired0=12
RU_reread_only_after_a_read_error fired1=146 fired0=12
```

（158 行里 12 行是 n 已超过那一槽被读的次数、注入没发生的收尾行。）RU 下被拒的 21 格：三个 txg × {会话发布 n = 1、卸载 n = 1、准入抬 F n = 1 与 n = 3、回退两种目标 n = 1、可写挂载 n = 8}，都是健康盘被拒成落后支缺根；原样同一格照常做成、checker 0。同一格两边各一行（`logs/proto/BASE/ru-6.log`、`logs/proto/RU_reread_only_after_a_read_error/ru-6.log`）：

```
R3RU txg=6 root_dev=0 action=Entry(NormalUnmount, FirstFileVersion) n=1 fired=1 outcome=做成了 checker=0 ms=238
R3RU txg=6 root_dev=0 action=Entry(NormalUnmount, FirstFileVersion) n=1 fired=1 outcome=DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { devices: [DeviceBehindTheCurrentVersion { device: DeviceIdentity(0), newest_system_configuration_journal_tail: JournalSequenceNumber { instance_generation: InstanceGeneration(1), counter: 5 }, current checker=0 ms=156
```

准入抬 F 在 n = 1、n = 3 两处都中：这个入口核两遍落后支（准入抬 F 先核一遍，`raise_rollback_floor` 里再核一遍），第二遍是那一槽的第 3 次读（哪两处是按代码与注释推的；n = 3 也中是量过的）。

四句：
- 分不分辨臂：没有臂，判的是 J2（与 H1）的用例够不够。
- 被判系统当时看得到判别它的东西：看得到。第一次读回的字节坏一位、自证不过，第二次读得出 P——原样代码重读就是为这一形；用例手里有 `ReadReturnsCorruptedBytes` 这个注入，造得出。
- 满足判据字面的哪一句：正文第五节「W2 原样错实现或新错实现放得过新用例 ⇒ 打中」的「新错实现放得过」。后果是合法盘被拒（假阳性），不坏盘面，与第二轮判 RP 打中是同一类。
- 改法（推的，零轮，只在我的模型上量过错实现的行为）：J2 那三格与 H1 那两格各加一版注入 `ReadReturnsCorruptedBytes`（一次、落在缺根那一半第一次读上），断言照常做成、注入真发生 1 次、之后 checker 不红；配一行变异（RU 的替换文）。RU 下这几格拒成缺根是量过的，用例本身没写。另一半——第一次读到的是 `AnotherRoot` 而重读读得出 P——今天的注入造不出（翻一位只会自证不过），推的，不提改法。

### W2-④　新错实现 RGC：卸载、准入抬 F、回退那条路上把区域号当盘号（打中，量过）

改哪里：第二轮攻方 AC、EC 的同一种限定（`caller_inputs_agreeing_with_the_disk` 唯一那次调用前后置线程标志），标志置着时 `/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-core/src/mount.rs:3416` 那一句算出的盘换成 `DeviceIdentity(区域号)`；会话发布与可写挂载照原样。即第二轮的 RG 只做在那三个入口上。

为什么新用例放得过：J1 只在会话发布上取了区域 2（txg 8，entries 用例文件第 760 行的常量，唯一用它的是会话发布那一条）；三个入口上缺根那一半的用例取的是区域 0（txg 6、27）与区域 1（txg 25），区域 2 一格都没有。原样摘行（`logs/w2-batch1.out`）：

```
WRONG RGC_region_number_as_device_in_the_three_other_entries entries_after_mount_refuse_swapped_or_behind_devices rc=0 test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2 failed=[]
WRONG RGC_region_number_as_device_in_the_three_other_entries sparse_devices_refuse_requests_outside_the_device_like_the_file_backend rc=0 test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0. failed=[]
WRONG RGC_region_number_as_device_in_the_three_other_entries core-lib rc=0 test result: ok. 139 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in  failed=[]
```

harness 全包与 core 单测见上一节 `COMBO_RU_RGC` 那两行（全过）。

后果原型 `r3opus_rgc_region_two_through_every_entry`：P 取区域 2 的 txg 8（槽里没写过）与 26、29、32（根环转过一圈，槽里是上一圈的根），P 的根住的那块盘（区域 2 归盘 0）切在根槽 FUA 之前，每个入口各走一次（回退两种目标），入口之后跑池级 checker、再拿拷贝可写挂载一次。命令数出来的汇总：

```
BASE cells=20 ok=0 checker_nonzero=0
RGC_region_number_as_device_in_the_three_other_entries cells=20 ok=10 checker_nonzero=10
```

- 原样：20 格全拒成落后支缺根、两块盘不变、checker 0。
- RGC：会话发布 4 格照拒（那条路没改）；txg 8 的三个入口拒成别的成员（`NewerStateStillUnreadableAfterOneReread`、`RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`、`CandidateJudgementStillUnreadableAfterOneReread`：槽里没写过，有「重读仍坏」那一道兜底）；txg 26、29、32 三个入口做成 10 格（txg 29、32 回退到第一个文件那一版是 `TargetNotACandidate`，与盘无关），入口之后 checker 红 1–2 条。一行原样（`logs/proto/RGC_region_number_as_device_in_the_three_other_entries/rgc-26.log`）：

```
R3RGC txg=26 region=2 slot=0 root_dev=0 action=Entry(NormalUnmount, FirstFileVersion) outcome=做成了 unchanged=false checker=2 first_violation=Some("I-3.1: 盘 0：记账的已分配 Some(557056)，遍历全部有效根得到 425984（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 24、最新根 txg 28、环里自证过的根槽 24 个、最老的自证过的根 ") remount=Ok(InstanceGeneration(2))
```

同一格原样（`logs/proto/BASE/rgc-26.log`，太长，只取前 200 个字符，命令 `cut -c1-200`）：

```
R3RGC txg=26 region=2 slot=0 root_dev=0 action=Entry(NormalUnmount, FirstFileVersion) outcome=DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { devices: [DeviceBehindTheCurrentVersion { devi
```

四句：
- 分不分辨臂：没有臂，判的是 J1 的取样（三个入口没取区域 2）。
- 被判系统当时看得到判别它的东西：看得到；区域 2 转圈那一形用 J3 现成的辅助函数（`root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through`）就造得出，只换 txg 与上一圈那条根。
- 满足判据字面的哪一句：同上「新错实现放得过」；后果是坏盘面（checker 红 I-3.1），比 RU 重。它与第二轮攻方 Y3 判的「转圈那一形上缺根那一半是三个入口唯一的一道」同一个机理，换的是区域。
- 改法（推的，零轮，只在我的模型上量过错实现的行为）：卸载、准入抬 F、回退各加一格 txg 26（区域 2、槽 0、住盘 0，槽里是上一圈暖机第二条根，按 `target_for_publish` 算是实例 1、txg 2——这条读数我没从日志里取到，日志里那一段被我截断了，用例写时要现核），回退取现行前一版；配一行变异（RGC 的替换文）。RGC 下这三格做成、checker 红是量过的，用例本身没写。

### W2 另试过、放不过新用例的（没打中）

| 试过的 | 结局 |
|---|---|
| 第二轮六个原样错实现 | 见 W2-① |
| E1 的「被打断第三次才报错」（上界 3） | 没跑。k = 1、2 两条单测放得过它（推的）；有限个 k 的单测都放得过「上界大于 k」的错实现，这是「不设上界」这条要求本身测不全，加 k = 3 只把界挪一格，不提改法 |
| 长度 0 早退放在对齐判据之后、越界判据之前 | 没跑。越过末尾一块那一格（对齐、越界）会让 core 单测与稀疏盘对拍红（推的，按用例取的偏移推） |

## W3　两块盘以外的几何（另成一节）

### W3-①　三块盘今天造不出；三块盘时 txg 25、27 的根落哪块盘

今天的实现把池钉在两块盘：mkfs 断言盘数是 2（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-core/src/make_filesystem.rs:422`「"第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）"」），两块盘时归属必须是 0 / 1 / 0（同一文件第 406 行「if devices.len() == 2 && parameters.region_devices != FIRST_VERSION_REGION_DEVICES {」）；harness 的起点也写死两块（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-harness/tests/common_admission/mod.rs:98`「let mut devices: Vec<(DeviceIdentity, Device)> = [DeviceIdentity(0), DeviceIdentity(1)]」）。在原样副本上量（`logs/proto/BASE/w3.log` 原样摘行）：

```
W3MKFS layout=three-012 devices=3 outcome=panic(assertion `left == right` failed: 第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9） |   left: 3 |  right: 2)
W3MKFS layout=three-010 devices=3 outcome=panic(assertion `left == right` failed: 第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9） |   left: 3 |  right: 2)
W3MKFS layout=three-021 devices=3 outcome=panic(assertion `left == right` failed: 第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9） |   left: 3 |  right: 2)
W3MKFS layout=two-011 devices=2 outcome=Err(RegionDevicesNotTheFirstVersionLayout { region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(1)] })
W3THIRD third=blank entry=SessionPublish outcome=DeviceTableOtherThanTheOneOfTheMount { device_table_of_the_mount: [DeviceIdentity(0), DeviceIdentity(1)], device_table_handed_in: [DeviceIdentity(0), D
W3THIRD third=blank entry=NormalUnmount outcome=CallerParametersDisagreeWithTheSelectedSystemConfiguration { disagreeing_fields: [], disagreeing_device_table: [DeviceCountDiffersFromTheSystemConfigura
W3THIRD third=blank entry=FloorRaiseToTheAdmissionCeiling outcome=CallerParametersDisagreeWithTheSelectedSystemConfiguration { disagreeing_fields: [], disagreeing_device_table: [DeviceCountDiffersFrom
W3THIRD third=blank entry=AdministratorRollback outcome=CallerInputsDisagreeWithTheDisk(ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration { disagreeing_fields: [], disagreeing_device_t
W3THIRD third=copy-of-device-one entry=SessionPublish outcome=DeviceTableOtherThanTheOneOfTheMount { device_table_of_the_mount: [DeviceIdentity(0), DeviceIdentity(1)], device_table_handed_in: [DeviceI
W3THIRD third=copy-of-device-one entry=NormalUnmount outcome=CallerParametersDisagreeWithTheSelectedSystemConfiguration { disagreeing_fields: [], disagreeing_device_table: [DeviceCountDiffersFromTheSy
W3THIRD third=copy-of-device-one entry=FloorRaiseToTheAdmissionCeiling outcome=CallerParametersDisagreeWithTheSelectedSystemConfiguration { disagreeing_fields: [], disagreeing_device_table: [DeviceCou
W3THIRD third=copy-of-device-one entry=AdministratorRollback outcome=CallerInputsDisagreeWithTheDisk(ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration { disagreeing_fields: [], disagre
```

（`W3THIRD` 八行太长，命令 `cut -c1-200` 只取前 200 个字符：挂着的两块盘的池，盘表里多交一块身份 2 的盘——空盘或盘 1 盘体的拷贝——会话发布拒成盘表不是挂载时那一份，另三个入口拒成盘数与系统配置不符。）

三块盘的归属今天没有条款给值（D2 已定项 7 只写了第一版两块盘的 0 / 1 / 0）。txg 落哪个区域、哪个槽只看 txg（`target_for_publish`：区域 = txg mod 3，槽 = (txg / 3) mod 8），落哪块盘看归属。用核心层的 `target_for_publish` 现算（`logs/proto/BASE/w3.log` 原样摘行）：

```
W3MAP layout=two-010 txg=6 region=0 slot=2 root_dev=0 lap=false wrongs_told_apart=[]
W3MAP layout=two-010 txg=7 region=1 slot=2 root_dev=1 lap=false wrongs_told_apart=["AC"]
W3MAP layout=two-010 txg=8 region=2 slot=2 root_dev=0 lap=false wrongs_told_apart=["RG"]
W3MAP layout=two-010 txg=25 region=1 slot=0 root_dev=1 lap=true wrongs_told_apart=["AC"]
W3MAP layout=two-010 txg=26 region=2 slot=0 root_dev=0 lap=true wrongs_told_apart=["RG"]
W3MAP layout=two-010 txg=27 region=0 slot=1 root_dev=0 lap=true wrongs_told_apart=[]
W3MAP layout=three-012 txg=6 region=0 slot=2 root_dev=0 lap=false wrongs_told_apart=[]
W3MAP layout=three-012 txg=7 region=1 slot=2 root_dev=1 lap=false wrongs_told_apart=["AC"]
W3MAP layout=three-012 txg=8 region=2 slot=2 root_dev=2 lap=false wrongs_told_apart=["AC", "MOD2", "FIRSTVERSION"]
W3MAP layout=three-012 txg=25 region=1 slot=0 root_dev=1 lap=true wrongs_told_apart=["AC"]
W3MAP layout=three-012 txg=26 region=2 slot=0 root_dev=2 lap=true wrongs_told_apart=["AC", "MOD2", "FIRSTVERSION"]
W3MAP layout=three-012 txg=27 region=0 slot=1 root_dev=0 lap=true wrongs_told_apart=[]
```

- 归属取 0 / 1 / 2：txg 25 的根落盘 1（区域 1、槽 0），txg 27 的根落盘 0（区域 0、槽 1），J1 的 txg 8 落盘 2。转圈与否只看 txg（S × 3 = 24 槽），与盘数无关。
- 末列是四种「根槽住哪块盘」的错算法里哪几种在这个 txg 上算出的盘与对的不同（只换 P 的根住的那块盘时，查错了盘就放过）：RG = 区域号当盘号；AC = 只查区域 0 那块；MOD2 = 区域号对 2 取模再查归属；FIRSTVERSION = 写死 0 / 1 / 0、不读盘上的归属。两块盘上 MOD2、FIRSTVERSION 在每个 txg 上都与对的同值。

### W3-②　J1–J3 取的 txg 在三块盘上还敏不敏感

装置（只在副本上、造的几何）：冻结副本另拷一份 `work3/`，打 `w3-three-device-patch.diff`——mkfs 的断言放开到 2 或 3 块；用例起点在 `R3_THREE_DEVICES=1` 时建三块盘、归属取 `R3_LAYOUT`（默认 0,1,2）；切盘那一步按镜像里的盘建盘表（原来写死两块）。核心层别的代码一行没动。原型 `r3opus_w3_three_device_sweep` 对 txg 6、7、8、25、26、27、32 各把 P 的根住的那块盘切在根槽 FUA 之前，五个用户动作（会话发布 S、卸载 U、准入抬 F、回退到第一个文件那一版 R1、回退到现行前一版 Rp）各走一次。7 种代码（原样与 6 个错实现）× 7 个 txg × 5 个动作 = 245 格，每种 3 件并行、每件 62.77–81.87 秒（21 份日志的 `finished in` 最小与最大）。

⚠️ 三盘池上池级 checker 不可用：原样代码拒在任何写之前、盘一个字节没动，checker 照样报 2 条（盘 2 的 I-3.1，`logs/w3/probe-txg6.log`），checker 本身不认三盘池。所以下表只比结局与盘变没变，括号里的 c 是入口之后 checker 的条数，只作参考。

`python3 sum-w3.py` 原样输出（`DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot` 是落后支；`ok(changed,cN)` 是做成、盘被写了）：

```
AC_only_region_zero_device_in_the_three_other_entries-012 txg=6 region=0 root_dev=0 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
AC_only_region_zero_device_in_the_three_other_entries-012 txg=7 region=1 root_dev=1 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:NewerStateStillUnreadableAfterOneReread F:RollbackFloorCeilingRootRingSlotStillBadAfterOneReread R1:CandidateJudgementStillUnreadableAfterOneReread Rp:CandidateJudgementStillUnreadableAfterOneReread
AC_only_region_zero_device_in_the_three_other_entries-012 txg=8 region=2 root_dev=2 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:NewerStateStillUnreadableAfterOneReread F:RollbackFloorCeilingRootRingSlotStillBadAfterOneReread R1:CandidateJudgementStillUnreadableAfterOneReread Rp:CandidateJudgementStillUnreadableAfterOneReread
AC_only_region_zero_device_in_the_three_other_entries-012 txg=25 region=1 root_dev=1 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:ok(changed,c3) F:ok(changed,c2) R1:ok(changed,c2) Rp:ok(changed,c2)
AC_only_region_zero_device_in_the_three_other_entries-012 txg=26 region=2 root_dev=2 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:ok(changed,c3) F:ok(changed,c2) R1:ok(changed,c2) Rp:ok(changed,c2)
AC_only_region_zero_device_in_the_three_other_entries-012 txg=27 region=0 root_dev=0 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
AC_only_region_zero_device_in_the_three_other_entries-012 txg=32 region=2 root_dev=2 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:ok(changed,c3) F:ok(changed,c2) R1:TargetNotACandidate Rp:ok(changed,c2)
BASE-012 txg=6 region=0 root_dev=0 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
BASE-012 txg=7 region=1 root_dev=1 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
BASE-012 txg=8 region=2 root_dev=2 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
BASE-012 txg=25 region=1 root_dev=1 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
BASE-012 txg=26 region=2 root_dev=2 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
BASE-012 txg=27 region=0 root_dev=0 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
BASE-012 txg=32 region=2 root_dev=2 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
EC_any_self_verified_root_in_the_three_other_entries-012 txg=6 region=0 root_dev=0 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
EC_any_self_verified_root_in_the_three_other_entries-012 txg=7 region=1 root_dev=1 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
EC_any_self_verified_root_in_the_three_other_entries-012 txg=8 region=2 root_dev=2 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
EC_any_self_verified_root_in_the_three_other_entries-012 txg=25 region=1 root_dev=1 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:ok(changed,c3) F:ok(changed,c2) R1:ok(changed,c2) Rp:ok(changed,c2)
EC_any_self_verified_root_in_the_three_other_entries-012 txg=26 region=2 root_dev=2 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:ok(changed,c3) F:ok(changed,c2) R1:ok(changed,c2) Rp:ok(changed,c2)
EC_any_self_verified_root_in_the_three_other_entries-012 txg=27 region=0 root_dev=0 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:ok(changed,c3) F:ok(changed,c2) R1:ok(changed,c2) Rp:ok(changed,c2)
EC_any_self_verified_root_in_the_three_other_entries-012 txg=32 region=2 root_dev=2 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:ok(changed,c3) F:ok(changed,c2) R1:TargetNotACandidate Rp:ok(changed,c2)
FV_first_version_layout_hard_coded-012 txg=6 region=0 root_dev=0 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
FV_first_version_layout_hard_coded-012 txg=7 region=1 root_dev=1 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
FV_first_version_layout_hard_coded-012 txg=8 region=2 root_dev=2 S:ok(changed,c2) U:NewerStateStillUnreadableAfterOneReread F:RollbackFloorCeilingRootRingSlotStillBadAfterOneReread R1:CandidateJudgementStillUnreadableAfterOneReread Rp:CandidateJudgementStillUnreadableAfterOneReread
FV_first_version_layout_hard_coded-012 txg=25 region=1 root_dev=1 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
FV_first_version_layout_hard_coded-012 txg=26 region=2 root_dev=2 S:ok(changed,c6) U:ok(changed,c3) F:ok(changed,c2) R1:ok(changed,c2) Rp:ok(changed,c2)
FV_first_version_layout_hard_coded-012 txg=27 region=0 root_dev=0 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
FV_first_version_layout_hard_coded-012 txg=32 region=2 root_dev=2 S:ok(changed,c6) U:ok(changed,c3) F:ok(changed,c2) R1:TargetNotACandidate Rp:ok(changed,c2)
MOD2_region_modulo_two-012 txg=6 region=0 root_dev=0 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
MOD2_region_modulo_two-012 txg=7 region=1 root_dev=1 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
MOD2_region_modulo_two-012 txg=8 region=2 root_dev=2 S:ok(changed,c2) U:NewerStateStillUnreadableAfterOneReread F:RollbackFloorCeilingRootRingSlotStillBadAfterOneReread R1:CandidateJudgementStillUnreadableAfterOneReread Rp:CandidateJudgementStillUnreadableAfterOneReread
MOD2_region_modulo_two-012 txg=25 region=1 root_dev=1 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
MOD2_region_modulo_two-012 txg=26 region=2 root_dev=2 S:ok(changed,c6) U:ok(changed,c3) F:ok(changed,c2) R1:ok(changed,c2) Rp:ok(changed,c2)
MOD2_region_modulo_two-012 txg=27 region=0 root_dev=0 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
MOD2_region_modulo_two-012 txg=32 region=2 root_dev=2 S:ok(changed,c6) U:ok(changed,c3) F:ok(changed,c2) R1:TargetNotACandidate Rp:ok(changed,c2)
RGC_region_number_as_device_in_the_three_other_entries-012 txg=6 region=0 root_dev=0 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
RGC_region_number_as_device_in_the_three_other_entries-012 txg=7 region=1 root_dev=1 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
RGC_region_number_as_device_in_the_three_other_entries-012 txg=8 region=2 root_dev=2 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
RGC_region_number_as_device_in_the_three_other_entries-012 txg=25 region=1 root_dev=1 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
RGC_region_number_as_device_in_the_three_other_entries-012 txg=26 region=2 root_dev=2 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
RGC_region_number_as_device_in_the_three_other_entries-012 txg=27 region=0 root_dev=0 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
RGC_region_number_as_device_in_the_three_other_entries-012 txg=32 region=2 root_dev=2 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
RG_region_number_as_device-012 txg=6 region=0 root_dev=0 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
RG_region_number_as_device-012 txg=7 region=1 root_dev=1 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
RG_region_number_as_device-012 txg=8 region=2 root_dev=2 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
RG_region_number_as_device-012 txg=25 region=1 root_dev=1 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
RG_region_number_as_device-012 txg=26 region=2 root_dev=2 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
RG_region_number_as_device-012 txg=27 region=0 root_dev=0 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
RG_region_number_as_device-012 txg=32 region=2 root_dev=2 S:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot U:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot F:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot R1:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot Rp:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot
```

读法：
- 原样代码 35 格全拒成落后支、盘不变（三块盘上这一判照样成立）。
- AC：两块盘上 J3 用的 txg 25 照样抓到（三个入口做成）；txg 27 照样抓不到（根就住区域 0 那块盘），与两块盘一样。三块盘上区域 2 的 txg 26、32 也抓得到。
- EC：txg 25、26、27、32 都抓到，与几何无关（只看转圈）。
- RG：归属 0 / 1 / 2 时区域号就是盘号，35 格与原样逐格相同——RG 在这个几何上不是错。J1 的 txg 8 在两块盘上抓 RG，到三块盘上没有可抓的。
- MOD2、FIRSTVERSION：两块盘上与对的同值（上一节那张表，`wrongs_told_apart` 一格都没有它俩），三块盘上会话发布在 txg 8 做成、盘被写（入口之后 checker 报的第一条换成盘 0 的 I-3.1，原样是盘 2 的那条），三个入口在 txg 8 被「重读仍坏」那一道兜住、拒因换了；区域 2 转圈的 txg 26、32 五个动作全做成（txg 32 回退到第一个文件那一版是 `TargetNotACandidate`）。
- RGC：与 RG 同理，三块盘上同值。

四句：
- 分不分辨臂：没有臂，判的是用例取样。
- 被判系统看得到判别它的东西：今天看不到——三块盘的池在今天的实现里建不出来（W3-①），MOD2、FIRSTVERSION 与对的实现在今天所有建得出的池上同值。
- 满足判据字面的哪一句：正文第五节「W3 在三块盘上取样不敏感 ⇒ 补一格用例」。按 `.claude/rules/mutation-sampling.md` 那张表，第三类是「在我选的取样点上恰好同值，换一个取样点就不同」，取样点得是今天的实现收得下的输入；三块盘不在里面，MOD2、FIRSTVERSION 在今天的取样域上是等价变异，不是第三类。我判这一格不中，写成一笔零轮。
- 改法（推的，零轮）：今天不补。等三块盘开线（D2 已定项 9 那条线），J1 要换成「区域 2 落在与区域 0 不同的盘上」的一格（归属 0 / 1 / 2 时 txg 8、26），三个入口也各一格；那时 RG 不再是错，J1 现在这一格对 RG 的意义随之没了。上面这些三块盘上的结局是量过的，但几何是我造的，不进 kb。

另一个几何旋钮：每区槽数 S。转圈的 txg（25、27）只在 S = 8 时是转圈；S 在盘上可以读到 4–16（`RootRingSlotsPerRegion::from_system_configuration_field`），但今天的 mkfs 只写 `AT_MAKE_FILESYSTEM`（8），别的 S 只能来自别人写的盘。没量，判同上：不在今天的取样域里。

## 我提的改法（都只在我的模型上量过错实现的行为，用例本身没写成入库形态，被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| J2 三格与 H1 两格各加一版「根槽第一次读回字节坏一位、重读读得出」（`ReadReturnsCorruptedBytes`，一次），配 RU 那一行变异 | W2-③ RU | 推的（用例没写）；RU 下这几格把健康盘拒成缺根、原样做成，是量过的（21 格 / 146 格注入真发生） |
| 卸载、准入抬 F、回退各加一格区域 2 转圈的 txg 26，回退取现行前一版，配 RGC 那一行变异 | W2-④ RGC | 推的（用例没写，槽里上一圈那条根的读数要现核）；RGC 下这三个入口做成、checker 红，是量过的 |
| 三块盘开线时，J1 换成区域 2 落在第三块盘上的一格、三个入口也各一格 | W3-② MOD2、FIRSTVERSION | 推的；今天不补。三盘副本上那几格的结局是量过的，几何是造的 |

两个改法互不碰：RU 的那一格只动注入的种类，RGC 的那一格只动 txg；RU 下 txg 26 那一格不受影响（槽里是上一圈的根，第一次读到 `AnotherRoot`，RU 不重读、两次读数照样相同，用例照样拒——推的），RGC 下注入那一格也不受影响（txg 6 在区域 0，RGC 只改区域 2 的盘——推的）。

## 没打中的形状

| 试过的形状 | 取样范围 | 结果 |
|---|---|---|
| 第二轮攻方原样六个错实现，套在这一轮冻结副本上 | entries 31 条、稀疏盘 2 条、core 139 条 | 都红，红在判决点名的那几条 |
| 原样代码，健康落后盘的根槽一次读回字节坏一位 | txg 6、7、8 × 六个用户动作 × n 1 到读不到为止（≤ 12） | 146 格注入真发生，全做成、checker 0 |
| 原样代码，区域 2 的 P、切在根槽之前、四个入口（回退两种目标） | txg 8、26、29、32，20 格 | 全拒成落后支缺根、盘不变、checker 0 |
| 原样代码，三块盘的池 | mkfs 三种归属、两块盘非 0 / 1 / 0、挂着的池多交一块盘 × 四个入口 | 三块盘 mkfs 都 panic；非 0 / 1 / 0 拒；多交一块 8 格都拒 |
| 三盘副本（造的）上原样与 RG、RGC | 7 个 txg × 5 个动作 | 35 格逐格相同（全拒成落后支） |

## 这条腿自己的限度

- 一条腿抽一次样；「没打中」的格不支撑「没有错」。
- RU、RGC 在 harness 全包上是组合着套的，单套全绿是推的；harness 的 `--lib` 单测没在错实现下跑；全包没另跑不套错实现的基线（W2 那三样的基线跑了、全过）。
- RU 的后果只在 txg 6、7、8 三个不转圈的 P 上扫了，一次只翻一位、固定在第 100 字节第 0 位；第一次读到 `AnotherRoot`、重读读得出 P 的那一形今天造不出，没量。
- RGC 的后果只扫了区域 2 的四个 txg；前缀全是覆盖写，没有抬 F、回退、建 inode 夹在中间。
- W3 的三块盘是改了 mkfs 断言造的几何，三盘池上 checker 不可用（原样也报 2 条），那一节只比结局；三块盘只量了归属 0 / 1 / 2 一种，别的归属（如 0 / 2 / 1）只在 mkfs 那一步试过（今天照样 panic）。
- 原型的崩溃态 0 个（没调层 0 枚举），这一轮的盘面都是换盘、切流与注入读错。
- 跑的过程：内存包装三次没跑成、一行没跑——同会话已有一条在跑（252，之后设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`）、证红上限超峰值 2 倍两次（251）；W3 三盘副本第一次编也撞了那个 252，补设之后编过。开工时 `ps` 看到别的会话的 `cargo clippy`、`cargo build` 与一条 checker 档 `e168_delete_ring_window_borrow` 在跑，没有性能测量；我这边一律 `nice -n 19`、经 `capped.sh 12`、上限 8G（证红 1300M）。

## 没做什么

- 不判 W1（正推那一格）；不替主 agent 采纳；副本上的数不算入库装置上的数。
- 重型测试没跑：checker 档、层 0 门禁、QEMU、herd7、变异整表、广谱变异。变异表只证了稀疏盘那一行 Z0；这一轮新加的另外 12 行没证（W1 的事）。
- 没改主工作区、冻结副本与 merge-wt 的任何文件；写的只有本报告、模型目录与草稿目录。

## 删掉的

- 删了整个草稿目录 `/tmp/claude-1000/m3-code-r3-opus/`，其中：仓副本 `work`（11M）、`work3`（11M，三盘副本）、`work-pr`（11M，证红副本），编译目录 `target`（12G）、`target3`（833M）、`target-pr`（665M），第二轮模型的拷贝 `r2model`（2.1M）；日志与脚本先整份拷进了模型目录（`diff -rq` 相同、`SHA256SUMS-logs.txt` 逐份 OK）再删。
