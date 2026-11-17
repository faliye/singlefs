# m4-one-copy-behind-refusal-r1 攻方腿报告（读数改判，2026-11-10）

批次：records/2026-10-13-里程碑四预开发.md#收口修补二批。承接 `research/prompts/m4-one-copy-behind-refusal-r1-opus-output.md`（下面简称「前份」，「前份 N」指它的第 N 行）。这一份不重新攻，只拿读数对前份写死的预期。

读数来源（量的，任务 11676，主 agent 交的；服务日志 `name=model rc=0`，四臂 `guards` 都 `rc=0`，这两句是派发提示转述的，我没读服务日志）。都在 `/home/fy5090/.local/share/singlefs/work/m4-one-copy-behind-refusal-r1-opus/rerun-2.attempt.Ggzsl1/` 下。**这些都是副本上的数（打了臂的 `mount.rs`），不是入库装置上的数，判决要引的由主 agent 复跑。**

```
c79bf06a299af4c742377c260dc97f6c671581e1a9f67f3299b95e21db8e56f0  model.txt
a2fa8745ff7f7e8e88eddeac4ce6c8c60ea5d430a9a3737c5e1fcf6c4da0394f  guards-c1.txt
343abd3048ef084af1b010c3950fc3e06d053dc782bdcf3786e0e070b5f0bed2  guards-c2.txt
441eafa118a30ec4cb811e9ec115626b602802897c00316066f211bd8add2b77  guards-c1-same.txt
e9b4773642d6a9af043ae8b23a5ab9c5e877785c9189ea44db6acb7d4fb3f550  guards-c3.txt
8889498e838e680d573b9244489f45615484ceab7bc4ecf7892d91b63afa7ae9  src/apply_arms_patch.py（前份模型目录，这一次现算，与前份第 226 行相同）
44784dd8f5e88552560096726a59f32371cd5218cf897cdc19470719d0c7a0e9  src/one_copy_behind_refusal_model.rs（同上）
cea901ef65836050a7aac4484e722d24d9cea2d3648e3c9aa2ca62718eea7967  tools/rerun.sh（同上）
```

下面整行都是 `grep -n` 对 `model.txt` 的原样，`NNN:` 是它的行号。第一节的「中 / 没中」由脚本 `/home/fy5090/.local/share/singlefs/work/m4-one-copy-behind-refusal-r1-opus-2/judge.py` 按写死的预期判，不手判；它只判 `MODEL_ARM` 整行。

## 各格判定一览

逐格 152 行在第一节（「量的」计数：中 123、没中 26、没写死 3，每组末行有脚本打的计数）。这张表只放决定判定的那几行。

| 格 | 前份判定 | 改判（量的） | 决定它的整行 |
|---|---|---|---|
| 不换盘的历史上放掉真缺单元 | 没打中（一次抽样） | **没中**。不换盘的形状上，`differing` 都等于手工弄坏的份数。手挑盘面，一次抽样，不支撑「没打中」的结论 | `72:MODEL_ARM older-unit-on-0-record-on-1 arm=c1 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []`<br>`107:MODEL_ARM every-data-unit-on-0-record-on-1 arm=c1 mounted units=22 differing_after_mount_on_0=7 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0 read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 候选 1 字面放掉真缺单元 | 打中，换盘分叉的等号格 | **中，但只有 5 个等号格里的 1 格**。`fork-5-3` 只有 `c1` 放、缺 4 个。`fork-2-0` 也只有 `c1` 放，`differing` 是 0/0，按前份 81 的判据不算（第三节）。另 3 格五臂一起放，今天也放，不是 `c1` 独有 | `495:MODEL_ARM fork-5-3 arm=today refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(4), checkpoint_txg: CheckpointTxg(16) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(4), counter: 22 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c`<br>`496:MODEL_ARM fork-5-3 arm=c1 mounted units=22 differing_after_mount_on_0=4 on_1=0`<br>`498:MODEL_ARM fork-5-3 arm=c1-same refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(4), checkpoint_txg: CheckpointTxg(16) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(4), counter: 22 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c`<br>`328:MODEL_ARM fork-2-0 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=0`<br>`383:MODEL_ARM fork-3-1 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 候选 2 该放行的还拒 | 打中：多记录、差二 | **多记录中；差二没中**。`lag-*` 没走到前份说的盘面，今天与 `c2` 都放（第三节） | `158:MODEL_ARM multi-record-older-on-0 arm=c2 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 21 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c`<br>`141:MODEL_ARM lag-older-on-0 arm=today mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []`<br>`143:MODEL_ARM lag-older-on-0 arm=c2 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 窗口比字节在要修的那一形上起不起作用 | 不起作用 | **中**。点名单元坏时五臂都挂上。所选版本退到上一版（这一半只有 `MODEL_STATE` 第 77 行有，`CheckpointTxg(11)`） | `78:MODEL_ARM named-unit-on-0-record-on-1 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []`<br>`79:MODEL_ARM named-unit-on-0-record-on-1 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 「tail 证明本盘收过」 | 字面不成立，离线出路破它 | **中，但后果比前份写的轻**。离线出路之后今天就放、缺 4 个；接着一次改动全换下，`violations=0` | `192:MODEL_ARM stale-disk-after-alpha arm=today mounted units=22 differing_after_mount_on_0=4 on_1=0 change=ok relocation=Relocated { mapping_keys: [[2, 2, 0, 1, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0], [2, 2, 0, 1, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 放行之后坏副本由谁换 | 盘 1 那一份接不上 | **中**。盘 0 那一份 `Relocated` 且绿；盘 1 那一份 `NotNeeded`，改动之后还差，`violations=3`。四臂逐字相同 | `72:MODEL_ARM older-unit-on-0-record-on-1 arm=c1 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []`<br>`86:MODEL_ARM older-unit-on-1-record-on-0 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 判真盘（放行三态与其余 150 态） | 没读数 | **有读数：四臂都 `ok`**（第三节末尾贴 `guards-*.txt`）。这条流弄坏的只有盘 0，不罩盘 1 那一形；用例不分辨臂 | 见第三节 |
| 放行有没有份数上限 | 没有；今天 tail 跟得上那一支同样没有 | **中**。7 个数据单元全坏放行且绿，与今天全落盘那一态逐字相同 | `107:MODEL_ARM every-data-unit-on-0-record-on-1 arm=c1 mounted units=22 differing_after_mount_on_0=7 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0 read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []`<br>`113:MODEL_ARM every-data-unit-on-0-all-persisted arm=today mounted units=22 differing_after_mount_on_0=7 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0 read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 候选 3 | 打不中第二、三行，不罩记录也坏的两故障 | **中**。多记录放且绿；分叉拒；两故障拒。只在我的模型上量过、被攻过零轮 | `160:MODEL_ARM multi-record-older-on-0 arm=c3 mounted units=27 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []`<br>`499:MODEL_ARM fork-5-3 arm=c3 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(4), checkpoint_txg: CheckpointTxg(16) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(4), counter: 22 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c`<br>`124:MODEL_ARM root-landed-record-bad-on-0-older-on-0 arm=c3 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 同一单元两份都坏 | 建上一版读不出，拒在逐盘核之前 | **没中，撤回**。五臂都挂上，读文件时才拒，`violations=4`。五臂相同，不是候选带来的 | `99:MODEL_ARM same-unit-both-copies-record-on-1 arm=today mounted units=21 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_refused=Read(DataUnitChecksumMismatchEverywhere { unit_index_in_file: DataUnitIndexInFile(0), hinted_slot: SlotNumber(50182) }) differing_after_change_on_0=0 on_1=0 unmount=ok violations=4 ["I-2.1:数据单元 在盘 0 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.2:最新的根走不完：数据单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]`<br>`100:MODEL_ARM same-unit-both-copies-record-on-1 arm=c1 mounted units=21 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_refused=Read(DataUnitChecksumMismatchEverywhere { unit_index_in_file: DataUnitIndexInFile(0), hinted_slot: SlotNumber(50182) }) differing_after_change_on_0=0 on_1=0 unmount=ok violations=4 ["I-2.1:数据单元 在盘 0 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.2:最新的根走不完：数据单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| （新）今天就放的真缺单元 | 前份只点了同实例分叉与离线出路 | **读数比前份多一族**：跨实例分叉 49 格里 10 格五臂都放，缺的在盘 1；同实例分叉 4 格五臂都放 | `383:MODEL_ARM fork-3-1 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=4`<br>`199:MODEL_ARM same-instance-fork-1 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=2` |

## 撤回与收窄

- 撤回前份 153：「同一单元两份都坏」不拒。读码也对得上：逐盘核跳过字节是空的单元，「`.filter(|unit| !unit.bytes.is_empty())`」（`crates/singlefs-core/src/mount.rs:7076`），`units=21` 比别的形状少 1 是这个（推的，由 `units=` 与这一行对出来，没单独量）。
- 撤回前份 173 的 `today`、`c2` 两格，前份 42、140 的「`lag-older-on-0` 上 `c2` 拒」，前份 11 的「两次断电的 tail 差二」：那一形**没造到**，不算反例，也不算没反例。
- 收窄前份 10 与前份 176：打中只有 `fork-5-3` 一格；「等号那几格今天拒」只对其中 2 格成立。
- 收窄前份 115 的判据：写的是 `differing_after_mount_on_0>0`，字面只有 `same-instance-fork-2` 中；另 3 格差的 2 个在盘 1（第 199、213、220 行）。今天挡不住这一族这句话不变，判据写窄了，是判据的错，不是形状的错。

## 四句（打中的格）

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到 | 满足的分句 | 改法在这几格上 |
|---|---|---|---|---|
| `c1` 放 `fork-5-3`（第 496 行） | 分辨：只有 `c1` 放 | 看得到：tail 实例 3、所选实例 4，判的那一刻两个代号都在手上，`c1-same` 就靠它退回 | 背景第四节第 1 问前半句「放行之后盘 0 真的缺这一版要的、而 tail 之后又没点名的单元」 | `c1-same`、`c2`、`c3` 都拒（量的，第 498、497、499 行） |
| `c2` 拒 `multi-record-older-on-0`（第 158 行） | 分辨 | 看得到 (tail, 所选位置) | 第 1 问后半句「或者该放行的还拒」 | `c1`、`c1-same`、`c3` 都放且绿（量的，第 157、159、160 行） |
| 四臂放 `older-unit-on-1-record-on-0` 之后红（第 86–89 行） | **不分辨**：四臂逐字相同，今天全落盘那一态也相同 | 看得到：逐盘核读到了盘 1 那一份不对，臂把它从缺的清单里删了 | 第 3 问「坏的那一份由谁换」；不是第 1 问（坏的是手工弄坏的那一份，不是从没收过） | 四个臂都还中，只改放行条件的改法碰不到这一格（第二节） |
| 今天就放的真缺单元（第 383、199、192 行这一族） | **不分辨**：五臂相同 | 跨实例那一族看得到实例不同，但「`if newest_system_configuration_journal_tail >= version_journal_position {`」（`crates/singlefs-core/src/mount.rs:7061`）先返回了 | 第 1 问前半句，今天的判也中 | 四个候选都不修。另立一笔账，不拿它判臂 |

## 一、逐格改判

验收口径：每格引 `MODEL_ARM` 整行，判只看这一行。「前份行」是预期写在前份的第几行。

### 改法表（前份 171–178，`fork` 那一行单列在下一小节）

| 前份行 | 形状 | 臂 | 跑前写死 | 量的 | 整行（`grep -n` 原样） |
|---|---|---|---|---|---|
| 171 | older-unit-on-0-record-on-1 | today | 拒 | 中 | `71:MODEL_ARM older-unit-on-0-record-on-1 arm=today refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 171 | older-unit-on-0-record-on-1 | c1 | 放 | 中 | `72:MODEL_ARM older-unit-on-0-record-on-1 arm=c1 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 171 | older-unit-on-0-record-on-1 | c1-same | 放 | 中 | `74:MODEL_ARM older-unit-on-0-record-on-1 arm=c1-same mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 171 | older-unit-on-0-record-on-1 | c2 | 放 | 中 | `73:MODEL_ARM older-unit-on-0-record-on-1 arm=c2 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 171 | older-unit-on-0-record-on-1 | c3 | 放 | 中 | `75:MODEL_ARM older-unit-on-0-record-on-1 arm=c3 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 172 | multi-record-older-on-0 | today | 拒 | 中 | `156:MODEL_ARM multi-record-older-on-0 arm=today refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 21 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 172 | multi-record-older-on-0 | c1 | 放 | 中 | `157:MODEL_ARM multi-record-older-on-0 arm=c1 mounted units=27 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 172 | multi-record-older-on-0 | c1-same | 放 | 中 | `159:MODEL_ARM multi-record-older-on-0 arm=c1-same mounted units=27 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 172 | multi-record-older-on-0 | c2 | 拒 | 中 | `158:MODEL_ARM multi-record-older-on-0 arm=c2 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 21 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 172 | multi-record-older-on-0 | c3 | 放 | 中 | `160:MODEL_ARM multi-record-older-on-0 arm=c3 mounted units=27 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 173 | lag-older-on-0 | today | 拒 | **没中** | `141:MODEL_ARM lag-older-on-0 arm=today mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 173 | lag-older-on-0 | c1 | 放 | 中 | `142:MODEL_ARM lag-older-on-0 arm=c1 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 173 | lag-older-on-0 | c1-same | 放 | 中 | `144:MODEL_ARM lag-older-on-0 arm=c1-same mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 173 | lag-older-on-0 | c2 | 拒 | **没中** | `143:MODEL_ARM lag-older-on-0 arm=c2 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 173 | lag-older-on-0 | c3 | 放 | 中 | `145:MODEL_ARM lag-older-on-0 arm=c3 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 174 | root-landed-record-bad-on-0-older-on-0 | today | 拒 | 中 | `120:MODEL_ARM root-landed-record-bad-on-0-older-on-0 arm=today refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 174 | root-landed-record-bad-on-0-older-on-0 | c1 | 放 | 中 | `121:MODEL_ARM root-landed-record-bad-on-0-older-on-0 arm=c1 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 174 | root-landed-record-bad-on-0-older-on-0 | c1-same | 放 | 中 | `123:MODEL_ARM root-landed-record-bad-on-0-older-on-0 arm=c1-same mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 174 | root-landed-record-bad-on-0-older-on-0 | c2 | 放 | 中 | `122:MODEL_ARM root-landed-record-bad-on-0-older-on-0 arm=c2 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 174 | root-landed-record-bad-on-0-older-on-0 | c3 | 拒 | 中 | `124:MODEL_ARM root-landed-record-bad-on-0-older-on-0 arm=c3 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 175 | root-landed-record-bad-on-0-named-on-0 | today | 拒 | 中 | `127:MODEL_ARM root-landed-record-bad-on-0-named-on-0 arm=today refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 175 | root-landed-record-bad-on-0-named-on-0 | c1 | 拒 | 中 | `128:MODEL_ARM root-landed-record-bad-on-0-named-on-0 arm=c1 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 175 | root-landed-record-bad-on-0-named-on-0 | c1-same | 拒 | 中 | `130:MODEL_ARM root-landed-record-bad-on-0-named-on-0 arm=c1-same refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 175 | root-landed-record-bad-on-0-named-on-0 | c2 | 拒 | 中 | `129:MODEL_ARM root-landed-record-bad-on-0-named-on-0 arm=c2 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 175 | root-landed-record-bad-on-0-named-on-0 | c3 | 拒 | 中 | `131:MODEL_ARM root-landed-record-bad-on-0-named-on-0 arm=c3 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 177 | stale-disk-after-alpha | today | 放 | 中 | `192:MODEL_ARM stale-disk-after-alpha arm=today mounted units=22 differing_after_mount_on_0=4 on_1=0 change=ok relocation=Relocated { mapping_keys: [[2, 2, 0, 1, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0], [2, 2, 0, 1, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 177 | stale-disk-after-alpha | c1 | 放 | 中 | `193:MODEL_ARM stale-disk-after-alpha arm=c1 mounted units=22 differing_after_mount_on_0=4 on_1=0 change=ok relocation=Relocated { mapping_keys: [[2, 2, 0, 1, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0], [2, 2, 0, 1, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 177 | stale-disk-after-alpha | c1-same | 放 | 中 | `195:MODEL_ARM stale-disk-after-alpha arm=c1-same mounted units=22 differing_after_mount_on_0=4 on_1=0 change=ok relocation=Relocated { mapping_keys: [[2, 2, 0, 1, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0], [2, 2, 0, 1, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 177 | stale-disk-after-alpha | c2 | 放 | 中 | `194:MODEL_ARM stale-disk-after-alpha arm=c2 mounted units=22 differing_after_mount_on_0=4 on_1=0 change=ok relocation=Relocated { mapping_keys: [[2, 2, 0, 1, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0], [2, 2, 0, 1, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 177 | stale-disk-after-alpha | c3 | 放 | 中 | `196:MODEL_ARM stale-disk-after-alpha arm=c3 mounted units=22 differing_after_mount_on_0=4 on_1=0 change=ok relocation=Relocated { mapping_keys: [[2, 2, 0, 1, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0], [2, 2, 0, 1, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 178 | same-instance-fork-1 | today | 放 | 中 | `199:MODEL_ARM same-instance-fork-1 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 178 | same-instance-fork-1 | c1 | 放 | 中 | `200:MODEL_ARM same-instance-fork-1 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 178 | same-instance-fork-1 | c1-same | 放 | 中 | `202:MODEL_ARM same-instance-fork-1 arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 178 | same-instance-fork-1 | c2 | 放 | 中 | `201:MODEL_ARM same-instance-fork-1 arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 178 | same-instance-fork-1 | c3 | 放 | 中 | `203:MODEL_ARM same-instance-fork-1 arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 178 | same-instance-fork-2 | today | 放 | 中 | `206:MODEL_ARM same-instance-fork-2 arm=today mounted units=22 differing_after_mount_on_0=2 on_1=0` |
| 178 | same-instance-fork-2 | c1 | 放 | 中 | `207:MODEL_ARM same-instance-fork-2 arm=c1 mounted units=22 differing_after_mount_on_0=2 on_1=0` |
| 178 | same-instance-fork-2 | c1-same | 放 | 中 | `209:MODEL_ARM same-instance-fork-2 arm=c1-same mounted units=22 differing_after_mount_on_0=2 on_1=0` |
| 178 | same-instance-fork-2 | c2 | 放 | 中 | `208:MODEL_ARM same-instance-fork-2 arm=c2 mounted units=22 differing_after_mount_on_0=2 on_1=0` |
| 178 | same-instance-fork-2 | c3 | 放 | 中 | `210:MODEL_ARM same-instance-fork-2 arm=c3 mounted units=22 differing_after_mount_on_0=2 on_1=0` |
| 178 | same-instance-fork-3 | today | 放 | 中 | `213:MODEL_ARM same-instance-fork-3 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 178 | same-instance-fork-3 | c1 | 放 | 中 | `214:MODEL_ARM same-instance-fork-3 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 178 | same-instance-fork-3 | c1-same | 放 | 中 | `216:MODEL_ARM same-instance-fork-3 arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 178 | same-instance-fork-3 | c2 | 放 | 中 | `215:MODEL_ARM same-instance-fork-3 arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 178 | same-instance-fork-3 | c3 | 放 | 中 | `217:MODEL_ARM same-instance-fork-3 arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 178 | same-instance-fork-4 | today | 放 | 中 | `220:MODEL_ARM same-instance-fork-4 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 178 | same-instance-fork-4 | c1 | 放 | 中 | `221:MODEL_ARM same-instance-fork-4 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 178 | same-instance-fork-4 | c1-same | 放 | 中 | `223:MODEL_ARM same-instance-fork-4 arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 178 | same-instance-fork-4 | c2 | 放 | 中 | `222:MODEL_ARM same-instance-fork-4 arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 178 | same-instance-fork-4 | c3 | 放 | 中 | `224:MODEL_ARM same-instance-fork-4 arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=2` |
<!-- changes: 行数=50 中=48 没中=2 没写死=0 -->

### 分叉的等号格（前份 176 与前份 81 的判据；只放 5 个等号格，全 49 格的分类在第三节）

| 前份行 | 形状 | 臂 | 跑前写死 | 量的 | 整行（`grep -n` 原样） |
|---|---|---|---|---|---|
| 176 | fork-2-0 | today | 拒 | 中 | `327:MODEL_ARM fork-2-0 arm=today refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(4), checkpoint_txg: CheckpointTxg(13) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(4), counter: 19 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 176 | fork-2-0 | c1 | 放 | 中 | `328:MODEL_ARM fork-2-0 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=0` |
| 176 | fork-2-0 | c1-same | 拒 | 中 | `330:MODEL_ARM fork-2-0 arm=c1-same refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(4), checkpoint_txg: CheckpointTxg(13) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(4), counter: 19 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 176 | fork-2-0 | c2 | 拒 | 中 | `329:MODEL_ARM fork-2-0 arm=c2 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(4), checkpoint_txg: CheckpointTxg(13) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(4), counter: 19 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 176 | fork-2-0 | c3 | 拒 | 中 | `331:MODEL_ARM fork-2-0 arm=c3 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(4), checkpoint_txg: CheckpointTxg(13) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(4), counter: 19 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 176 | fork-3-1 | today | 拒 | **没中** | `383:MODEL_ARM fork-3-1 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 176 | fork-3-1 | c1 | 放 | 中 | `384:MODEL_ARM fork-3-1 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 176 | fork-3-1 | c1-same | 拒 | **没中** | `386:MODEL_ARM fork-3-1 arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 176 | fork-3-1 | c2 | 拒 | **没中** | `385:MODEL_ARM fork-3-1 arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 176 | fork-3-1 | c3 | 拒 | **没中** | `387:MODEL_ARM fork-3-1 arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 176 | fork-4-2 | today | 拒 | **没中** | `439:MODEL_ARM fork-4-2 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 176 | fork-4-2 | c1 | 放 | 中 | `440:MODEL_ARM fork-4-2 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 176 | fork-4-2 | c1-same | 拒 | **没中** | `442:MODEL_ARM fork-4-2 arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 176 | fork-4-2 | c2 | 拒 | **没中** | `441:MODEL_ARM fork-4-2 arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 176 | fork-4-2 | c3 | 拒 | **没中** | `443:MODEL_ARM fork-4-2 arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 176 | fork-5-3 | today | 拒 | 中 | `495:MODEL_ARM fork-5-3 arm=today refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(4), checkpoint_txg: CheckpointTxg(16) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(4), counter: 22 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 176 | fork-5-3 | c1 | 放 | 中 | `496:MODEL_ARM fork-5-3 arm=c1 mounted units=22 differing_after_mount_on_0=4 on_1=0` |
| 176 | fork-5-3 | c1-same | 拒 | 中 | `498:MODEL_ARM fork-5-3 arm=c1-same refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(4), checkpoint_txg: CheckpointTxg(16) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(4), counter: 22 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 176 | fork-5-3 | c2 | 拒 | 中 | `497:MODEL_ARM fork-5-3 arm=c2 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(4), checkpoint_txg: CheckpointTxg(16) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(4), counter: 22 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 176 | fork-5-3 | c3 | 拒 | 中 | `499:MODEL_ARM fork-5-3 arm=c3 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(4), checkpoint_txg: CheckpointTxg(16) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(4), counter: 22 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 176 | fork-6-4 | today | 拒 | **没中** | `551:MODEL_ARM fork-6-4 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 176 | fork-6-4 | c1 | 放 | 中 | `552:MODEL_ARM fork-6-4 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 176 | fork-6-4 | c1-same | 拒 | **没中** | `554:MODEL_ARM fork-6-4 arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 176 | fork-6-4 | c2 | 拒 | **没中** | `553:MODEL_ARM fork-6-4 arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 176 | fork-6-4 | c3 | 拒 | **没中** | `555:MODEL_ARM fork-6-4 arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 81 | fork-2-0 | c1 | 放 且 differing_after_mount_on_0>0 | **没中** | `328:MODEL_ARM fork-2-0 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=0` |
| 81 | fork-3-1 | c1 | 放 且 differing_after_mount_on_0>0 | **没中** | `384:MODEL_ARM fork-3-1 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 81 | fork-4-2 | c1 | 放 且 differing_after_mount_on_0>0 | **没中** | `440:MODEL_ARM fork-4-2 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=4` |
| 81 | fork-5-3 | c1 | 放 且 differing_after_mount_on_0>0 | 中 | `496:MODEL_ARM fork-5-3 arm=c1 mounted units=22 differing_after_mount_on_0=4 on_1=0` |
| 81 | fork-6-4 | c1 | 放 且 differing_after_mount_on_0>0 | **没中** | `552:MODEL_ARM fork-6-4 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=4` |
<!-- fork: 行数=30 中=14 没中=16 没写死=0 -->

### 其余写死的预期（前份 31、53、111、115、137–142、151–154）

| 前份行 | 形状 | 臂 | 跑前写死 | 量的 | 整行（`grep -n` 原样） |
|---|---|---|---|---|---|
| 31 | named-unit-on-0-record-on-1 | today | 放 | 中 | `78:MODEL_ARM named-unit-on-0-record-on-1 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 31 | named-unit-on-0-record-on-1 | c1 | 放 | 中 | `79:MODEL_ARM named-unit-on-0-record-on-1 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 31 | named-unit-on-0-record-on-1 | c1-same | 放 | 中 | `81:MODEL_ARM named-unit-on-0-record-on-1 arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 31 | named-unit-on-0-record-on-1 | c2 | 放 | 中 | `80:MODEL_ARM named-unit-on-0-record-on-1 arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 31 | named-unit-on-0-record-on-1 | c3 | 放 | 中 | `82:MODEL_ARM named-unit-on-0-record-on-1 arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 53 | older-unit-on-1-all-persisted | today | 放 且 violations>0 | 中 | `163:MODEL_ARM older-unit-on-1-all-persisted arm=today mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 53 | older-unit-on-1-all-persisted | c1 | 放 且 violations>0 | 中 | `164:MODEL_ARM older-unit-on-1-all-persisted arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 53 | older-unit-on-1-all-persisted | c1-same | 放 且 violations>0 | 中 | `166:MODEL_ARM older-unit-on-1-all-persisted arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 53 | older-unit-on-1-all-persisted | c2 | 放 且 violations>0 | 中 | `165:MODEL_ARM older-unit-on-1-all-persisted arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 53 | older-unit-on-1-all-persisted | c3 | 放 且 violations>0 | 中 | `167:MODEL_ARM older-unit-on-1-all-persisted arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 53 | older-unit-on-1-record-on-0 | today | — | 没写死；量的 拒 | `85:MODEL_ARM older-unit-on-1-record-on-0 arm=today refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(1), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 53 | older-unit-on-1-record-on-0 | c1 | 放 且 relocation=NotNeeded 且 violations>0 | 中 | `86:MODEL_ARM older-unit-on-1-record-on-0 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 53 | older-unit-on-1-record-on-0 | c1-same | 放 且 relocation=NotNeeded 且 violations>0 | 中 | `88:MODEL_ARM older-unit-on-1-record-on-0 arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 53 | older-unit-on-1-record-on-0 | c2 | 放 且 relocation=NotNeeded 且 violations>0 | 中 | `87:MODEL_ARM older-unit-on-1-record-on-0 arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 53 | older-unit-on-1-record-on-0 | c3 | 放 且 relocation=NotNeeded 且 violations>0 | 中 | `89:MODEL_ARM older-unit-on-1-record-on-0 arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 111 | stale-disk-control | today | 拒 | 中 | `177:MODEL_ARM stale-disk-control arm=today refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(13) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 19 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 111 | stale-disk-control | c1 | 拒 | 中 | `178:MODEL_ARM stale-disk-control arm=c1 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(13) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 19 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 111 | stale-disk-control | c1-same | 拒 | 中 | `180:MODEL_ARM stale-disk-control arm=c1-same refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(13) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 19 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 111 | stale-disk-control | c2 | 拒 | 中 | `179:MODEL_ARM stale-disk-control arm=c2 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(13) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 19 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 111 | stale-disk-control | c3 | 拒 | 中 | `181:MODEL_ARM stale-disk-control arm=c3 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(13) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 19 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 111 | stale-disk-after-alpha | today | 放 且 differing_after_mount_on_0>0 | 中 | `192:MODEL_ARM stale-disk-after-alpha arm=today mounted units=22 differing_after_mount_on_0=4 on_1=0 change=ok relocation=Relocated { mapping_keys: [[2, 2, 0, 1, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0], [2, 2, 0, 1, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 115 | same-instance-fork-1 | today | 放 且 differing_after_mount_on_0>0 | **没中** | `199:MODEL_ARM same-instance-fork-1 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 115 | same-instance-fork-2 | today | 放 且 differing_after_mount_on_0>0 | 中 | `206:MODEL_ARM same-instance-fork-2 arm=today mounted units=22 differing_after_mount_on_0=2 on_1=0` |
| 115 | same-instance-fork-3 | today | 放 且 differing_after_mount_on_0>0 | **没中** | `213:MODEL_ARM same-instance-fork-3 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 115 | same-instance-fork-4 | today | 放 且 differing_after_mount_on_0>0 | **没中** | `220:MODEL_ARM same-instance-fork-4 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=2` |
| 137 | control-record-on-1-no-damage | today | 放 且 violations=0 | 中 | `64:MODEL_ARM control-record-on-1-no-damage arm=today mounted units=22 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 137 | control-record-on-1-no-damage | c1 | 放 且 violations=0 | 中 | `65:MODEL_ARM control-record-on-1-no-damage arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 137 | control-record-on-1-no-damage | c1-same | 放 且 violations=0 | 中 | `67:MODEL_ARM control-record-on-1-no-damage arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 137 | control-record-on-1-no-damage | c2 | 放 且 violations=0 | 中 | `66:MODEL_ARM control-record-on-1-no-damage arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 137 | control-record-on-1-no-damage | c3 | 放 且 violations=0 | 中 | `68:MODEL_ARM control-record-on-1-no-damage arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 138 | older-unit-on-0-record-on-1 | c1 | 放 且 change=ok | 中 | `72:MODEL_ARM older-unit-on-0-record-on-1 arm=c1 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 138 | older-unit-on-0-record-on-1 | c1-same | 放 且 change=ok | 中 | `74:MODEL_ARM older-unit-on-0-record-on-1 arm=c1-same mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 138 | older-unit-on-0-record-on-1 | c2 | 放 且 change=ok | 中 | `73:MODEL_ARM older-unit-on-0-record-on-1 arm=c2 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 138 | older-unit-on-0-record-on-1 | c3 | 放 且 change=ok | 中 | `75:MODEL_ARM older-unit-on-0-record-on-1 arm=c3 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 139 | lag-older-on-0 | today | 放 | 中 | `141:MODEL_ARM lag-older-on-0 arm=today mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 139 | lag-older-on-0 | c1 | 放 | 中 | `142:MODEL_ARM lag-older-on-0 arm=c1 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 139 | lag-older-on-0 | c1-same | 放 | 中 | `144:MODEL_ARM lag-older-on-0 arm=c1-same mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 139 | lag-older-on-0 | c2 | 放 | 中 | `143:MODEL_ARM lag-older-on-0 arm=c2 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 139 | lag-older-on-0 | c3 | 放 | 中 | `145:MODEL_ARM lag-older-on-0 arm=c3 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 142 | older-unit-on-0-record-on-1 | c1 | 放 且 relocation=Relocated | 中 | `72:MODEL_ARM older-unit-on-0-record-on-1 arm=c1 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 142 | older-unit-on-0-record-on-1 | c1-same | 放 且 relocation=Relocated | 中 | `74:MODEL_ARM older-unit-on-0-record-on-1 arm=c1-same mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 142 | older-unit-on-0-record-on-1 | c2 | 放 且 relocation=Relocated | 中 | `73:MODEL_ARM older-unit-on-0-record-on-1 arm=c2 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 142 | older-unit-on-0-record-on-1 | c3 | 放 且 relocation=Relocated | 中 | `75:MODEL_ARM older-unit-on-0-record-on-1 arm=c3 mounted units=22 differing_after_mount_on_0=1 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 142 | older-unit-on-1-record-on-0 | c1 | 放 且 relocation=NotNeeded | 中 | `86:MODEL_ARM older-unit-on-1-record-on-0 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 142 | older-unit-on-1-record-on-0 | c1-same | 放 且 relocation=NotNeeded | 中 | `88:MODEL_ARM older-unit-on-1-record-on-0 arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 142 | older-unit-on-1-record-on-0 | c2 | 放 且 relocation=NotNeeded | 中 | `87:MODEL_ARM older-unit-on-1-record-on-0 arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 142 | older-unit-on-1-record-on-0 | c3 | 放 且 relocation=NotNeeded | 中 | `89:MODEL_ARM older-unit-on-1-record-on-0 arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 151 | every-data-unit-on-0-record-on-1 | today | — | 没写死；量的 拒 | `106:MODEL_ARM every-data-unit-on-0-record-on-1 arm=today refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 151 | every-data-unit-on-0-record-on-1 | c1 | 放 | 中 | `107:MODEL_ARM every-data-unit-on-0-record-on-1 arm=c1 mounted units=22 differing_after_mount_on_0=7 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0 read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 151 | every-data-unit-on-0-record-on-1 | c1-same | 放 | 中 | `109:MODEL_ARM every-data-unit-on-0-record-on-1 arm=c1-same mounted units=22 differing_after_mount_on_0=7 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0 read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 151 | every-data-unit-on-0-record-on-1 | c2 | 放 | 中 | `108:MODEL_ARM every-data-unit-on-0-record-on-1 arm=c2 mounted units=22 differing_after_mount_on_0=7 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0 read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 151 | every-data-unit-on-0-record-on-1 | c3 | 放 | 中 | `110:MODEL_ARM every-data-unit-on-0-record-on-1 arm=c3 mounted units=22 differing_after_mount_on_0=7 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0 read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 151 | every-data-unit-on-0-all-persisted | today | 放 | 中 | `113:MODEL_ARM every-data-unit-on-0-all-persisted arm=today mounted units=22 differing_after_mount_on_0=7 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0 read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 151 | every-data-unit-on-0-all-persisted | c1 | 放 | 中 | `114:MODEL_ARM every-data-unit-on-0-all-persisted arm=c1 mounted units=22 differing_after_mount_on_0=7 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0 read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 151 | every-data-unit-on-0-all-persisted | c1-same | 放 | 中 | `116:MODEL_ARM every-data-unit-on-0-all-persisted arm=c1-same mounted units=22 differing_after_mount_on_0=7 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0 read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 151 | every-data-unit-on-0-all-persisted | c2 | 放 | 中 | `115:MODEL_ARM every-data-unit-on-0-all-persisted arm=c2 mounted units=22 differing_after_mount_on_0=7 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0 read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 151 | every-data-unit-on-0-all-persisted | c3 | 放 | 中 | `117:MODEL_ARM every-data-unit-on-0-all-persisted arm=c3 mounted units=22 differing_after_mount_on_0=7 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0 read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 152 | older-on-0-other-on-1-record-on-1 | today | — | 没写死；量的 拒 | `92:MODEL_ARM older-on-0-other-on-1-record-on-1 arm=today refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 152 | older-on-0-other-on-1-record-on-1 | c1 | 放 且 改动之后盘 1 还差 (on_1>0) | 中 | `93:MODEL_ARM older-on-0-other-on-1-record-on-1 arm=c1 mounted units=22 differing_after_mount_on_0=1 on_1=1 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50184 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 19）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 19）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 152 | older-on-0-other-on-1-record-on-1 | c1-same | 放 且 改动之后盘 1 还差 (on_1>0) | 中 | `95:MODEL_ARM older-on-0-other-on-1-record-on-1 arm=c1-same mounted units=22 differing_after_mount_on_0=1 on_1=1 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50184 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 19）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 19）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 152 | older-on-0-other-on-1-record-on-1 | c2 | 放 且 改动之后盘 1 还差 (on_1>0) | 中 | `94:MODEL_ARM older-on-0-other-on-1-record-on-1 arm=c2 mounted units=22 differing_after_mount_on_0=1 on_1=1 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50184 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 19）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 19）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 152 | older-on-0-other-on-1-record-on-1 | c3 | 放 且 改动之后盘 1 还差 (on_1>0) | 中 | `96:MODEL_ARM older-on-0-other-on-1-record-on-1 arm=c3 mounted units=22 differing_after_mount_on_0=1 on_1=1 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0]], units_exempt_from_the_mapping_replaced: [], insta read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50184 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 19）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 19）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 153 | same-unit-both-copies-record-on-1 | today | 拒 | **没中** | `99:MODEL_ARM same-unit-both-copies-record-on-1 arm=today mounted units=21 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_refused=Read(DataUnitChecksumMismatchEverywhere { unit_index_in_file: DataUnitIndexInFile(0), hinted_slot: SlotNumber(50182) }) differing_after_change_on_0=0 on_1=0 unmount=ok violations=4 ["I-2.1:数据单元 在盘 0 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.2:最新的根走不完：数据单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 153 | same-unit-both-copies-record-on-1 | c1 | 拒 | **没中** | `100:MODEL_ARM same-unit-both-copies-record-on-1 arm=c1 mounted units=21 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_refused=Read(DataUnitChecksumMismatchEverywhere { unit_index_in_file: DataUnitIndexInFile(0), hinted_slot: SlotNumber(50182) }) differing_after_change_on_0=0 on_1=0 unmount=ok violations=4 ["I-2.1:数据单元 在盘 0 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.2:最新的根走不完：数据单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 153 | same-unit-both-copies-record-on-1 | c1-same | 拒 | **没中** | `102:MODEL_ARM same-unit-both-copies-record-on-1 arm=c1-same mounted units=21 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_refused=Read(DataUnitChecksumMismatchEverywhere { unit_index_in_file: DataUnitIndexInFile(0), hinted_slot: SlotNumber(50182) }) differing_after_change_on_0=0 on_1=0 unmount=ok violations=4 ["I-2.1:数据单元 在盘 0 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.2:最新的根走不完：数据单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 153 | same-unit-both-copies-record-on-1 | c2 | 拒 | **没中** | `101:MODEL_ARM same-unit-both-copies-record-on-1 arm=c2 mounted units=21 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_refused=Read(DataUnitChecksumMismatchEverywhere { unit_index_in_file: DataUnitIndexInFile(0), hinted_slot: SlotNumber(50182) }) differing_after_change_on_0=0 on_1=0 unmount=ok violations=4 ["I-2.1:数据单元 在盘 0 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.2:最新的根走不完：数据单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 153 | same-unit-both-copies-record-on-1 | c3 | 拒 | **没中** | `103:MODEL_ARM same-unit-both-copies-record-on-1 arm=c3 mounted units=21 differing_after_mount_on_0=0 on_1=0 change=ok relocation=NotNeeded read_refused=Read(DataUnitChecksumMismatchEverywhere { unit_index_in_file: DataUnitIndexInFile(0), hinted_slot: SlotNumber(50182) }) differing_after_change_on_0=0 on_1=0 unmount=ok violations=4 ["I-2.1:数据单元 在盘 0 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.2:最新的根走不完：数据单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 154 | root-landed-record-bad-on-0-named-on-0 | today | 拒 | 中 | `127:MODEL_ARM root-landed-record-bad-on-0-named-on-0 arm=today refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 154 | root-landed-record-bad-on-0-named-on-0 | c1 | 拒 | 中 | `128:MODEL_ARM root-landed-record-bad-on-0-named-on-0 arm=c1 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 154 | root-landed-record-bad-on-0-named-on-0 | c1-same | 拒 | 中 | `130:MODEL_ARM root-landed-record-bad-on-0-named-on-0 arm=c1-same refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 154 | root-landed-record-bad-on-0-named-on-0 | c2 | 拒 | 中 | `129:MODEL_ARM root-landed-record-bad-on-0-named-on-0 arm=c2 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
| 154 | root-landed-record-bad-on-0-named-on-0 | c3 | 拒 | 中 | `131:MODEL_ARM root-landed-record-bad-on-0-named-on-0 arm=c3 refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(0), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |
<!-- others: 行数=72 中=61 没中=8 没写死=3 -->

## 二、`older-unit-on-1-record-on-0` 放行之后 `violations=3`：新坏，还是今天同样的盘面

**判：不是放行带来的新一种坏。读数与 tail 跟得上时今天的读数逐字相同；放行多出来的是入口：这一态今天拒、盘不变，放行之后走进同一个缺口。**

| 对照哪一行 | 相同或不同 | 整行 |
|---|---|---|
| `older-unit-on-1-all-persisted arm=today`（第 163 行，盘 1 那一份坏、全落盘，这是真正的镜像对照） | **相同**：去掉形状名与臂之后逐字相同（下面的比对输出）。同样 3 条违例、同样槽 50182、同样 txg 17 | `86:MODEL_ARM older-unit-on-1-record-on-0 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]`<br>`163:MODEL_ARM older-unit-on-1-all-persisted arm=today mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| `every-data-unit-on-0-all-persisted arm=today`（第 113 行，派发提示点名的） | **不同**：它是盘 0 的形状，`Relocated`、`violations=0`。比它不能回答盘 1 这一问。它对的是盘 0 那一边：第 107 行与它逐字相同 | `113:MODEL_ARM every-data-unit-on-0-all-persisted arm=today mounted units=22 differing_after_mount_on_0=7 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0 read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []`<br>`107:MODEL_ARM every-data-unit-on-0-record-on-1 arm=c1 mounted units=22 differing_after_mount_on_0=7 on_1=0 change=ok relocation=Relocated { mapping_keys: [[1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0 read_equal=true differing_after_change_on_0=0 on_1=0 unmount=ok violations=0 []` |
| 前份 142「坏副本由谁换」一格 | **相同**：盘 1 那一份 `relocation=NotNeeded`，改动之后 `on_1=1`，那一格写的「接不上」量到了 | `86:MODEL_ARM older-unit-on-1-record-on-0 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=1 change=ok relocation=NotNeeded read_equal=true differing_after_change_on_0=0 on_1=1 unmount=ok violations=3 ["I-2.1:数据单元 在盘 1 槽 50182 的那一份与位置条目里的校验和对不上", "I-4.8:最新根（txg 17）出发的遍历有单元对不上或读不出", "I-7.4:最新根（txg 17）引用的单元已被复用或抹头（校验和对不上或头用不了）"]` |
| 同一盘面 `arm=today`（第 85 行） | **不同**，这就是入口：今天拒，`disk_unchanged=true` | `85:MODEL_ARM older-unit-on-1-record-on-0 arm=today refused disk_unchanged=true refusal=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(12) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(3), counter: 18 }, devices: [DeviceWithoutTheSelectedVersion { device: DeviceIdentity(1), lacking: BehindTheSelectedVersionAndMissingItsUnits { newest_system_c` |

比对（把 `MODEL_ARM <形状> arm=<臂> ` 去掉，比后面整段字符串；这一次现跑）：

```
older-unit-on-1-record-on-0 arm=c1  vs  older-unit-on-1-all-persisted arm=today: 去掉形状名与臂之后逐字相同=True
older-unit-on-1-record-on-0 arm=c3  vs  older-unit-on-1-all-persisted arm=today: 去掉形状名与臂之后逐字相同=True
every-data-unit-on-0-record-on-1 arm=c1  vs  every-data-unit-on-0-all-persisted arm=today: 去掉形状名与臂之后逐字相同=True
every-data-unit-on-0-record-on-1 arm=c3  vs  every-data-unit-on-0-all-persisted arm=today: 去掉形状名与臂之后逐字相同=True
older-unit-on-0-record-on-1 arm=c1  vs  older-unit-on-0-all-persisted arm=today: 去掉形状名与臂之后逐字相同=True
older-unit-on-1-record-on-0 arm=c1  vs  every-data-unit-on-0-all-persisted arm=today: 去掉形状名与臂之后逐字相同=False
older-on-0-other-on-1-record-on-1 arm=c1  vs  older-unit-on-1-all-persisted arm=today: 去掉形状名与臂之后逐字相同=False
older-unit-on-1-record-on-0 四臂放行行逐字相同= True
older-on-0-other-on-1-record-on-1 四臂放行行逐字相同= True
every-data-unit-on-0-record-on-1 四臂放行行逐字相同= True
older-unit-on-0-record-on-1 四臂放行行逐字相同= True
```

这个「相同」的边界：

- 比的是打出来的字段（`relocation` 截到 160 字符，违例文字每条截到 100 字符），**不是卸载之后盘的字节**，也没比别的不变量的结论文字。
- 起点不是同一块盘：tail 是 (3,17) 与 (3,18)，盘 0 差着一条记录这一点两边不同。
- 前份 53 的跑前条款写「两边都红，就是另一笔账，候选不背」，字面成立。我要补一句前份没写的：**放行之前这一态进不了那个缺口，放行之后进得了**，「候选不背」说的是病根，不是说放行对这一态没影响。
- `older-on-0-other-on-1-record-on-1`（第 93 行，违例落在槽 50184、txg 19）没有全落盘的对照形状，同一笔账是推的（盘 1 那一份、同样 3 条违例），不是量的。

**这三行不能判候选 1 字面失败。** 候选 1 还有后半句「这些之外的单元缺一份按读到一份坏处理」（背景第三节第 1 条）。模型的臂没实现它，只把窗口外的删掉：「`units_missing.retain(|(_, slot)| slots_named_by_the_window.contains(slot));`」（`research/prompts/m4-one-copy-behind-refusal-r1-opus-model/src/apply_arms_patch.py:128`）。所以读数说的是「放行而不接那半句就红」。接上那半句，第 86 行这一态能不能绿，**没量**；就算能绿（推的），第 163 行那一态 tail 跟得上、不走这条判，照样红，那是今天的账，放行条件碰不到。

交主 agent 定：盘 1 这一笔要不要在放行之前先修。不修就放行，等于把「放行」的验收写成「判真盘绿只罩盘 0」。这条里我没核有没有现成的欠账行。

什么现象推翻这一判：第 163 行在今天的臂上变绿而第 86 行还红；或者两边卸载之后的盘逐字节比，放行那一边多出今天那一边没有的违例。

## 三、臂 × 形状

单元格写「判定 行号」。判法（写死在这里，不随读数改）：

- **放绿**：挂上，接着一次改动、读回相等、卸载，`violations=0`。
- **放红**：挂上，卸载之后 `violations>0`；坏的是手工弄坏的那几份。
- **放错**：挂上，`differing` 里有不是手工弄坏的单元（分叉、旧快照的盘，这块盘没收过）。**这是这张表里唯一算「放掉真缺单元」的一种。**
- **放（后果没量）**：挂上，模型没接着走（`follow_on=false`），只有 `differing`，没有 checker 读数。
- **拒错**：拒，而同一盘面上别的臂放了且 `violations=0`，读数说放得下。
- **拒**：拒，没有臂给出放得下的读数。

| 形状 | 真坏的是什么 | today | c1 字面 | c1-same | c2 | c3 |
|---|---|---|---|---|---|---|
| `older-unit-on-0-record-on-1` | 手工 1 份 | 拒错 71 | 放绿 72 | 放绿 74 | 放绿 73 | 放绿 75 |
| `multi-record-older-on-0` | 手工 1 份 | 拒错 156 | 放绿 157 | 放绿 159 | **拒错 158** | 放绿 160 |
| `every-data-unit-on-0-record-on-1` | 手工 7 份 | 拒错 106 | 放绿 107 | 放绿 109 | 放绿 108 | 放绿 110 |
| `root-landed-record-bad-on-0-older-on-0` | 手工 1 份加盘 0 记录坏 | 拒错 120 | 放绿 121 | 放绿 123 | 放绿 122 | **拒错 124** |
| `root-landed-record-bad-on-0-named-on-0` | 记录坏加它点名的单元坏 | 拒 127 | 拒 128 | 拒 130 | 拒 129 | 拒 131 |
| `older-unit-on-1-record-on-0` | 手工 1 份在盘 1 | 拒 85 | 放红 86 | 放红 88 | 放红 87 | 放红 89 |
| `older-on-0-other-on-1-record-on-1` | 手工两盘各 1 份 | 拒 92 | 放红 93 | 放红 95 | 放红 94 | 放红 96 |
| `same-unit-both-copies-record-on-1` | 手工同单元两份 | 放红 99 | 放红 100 | 放红 102 | 放红 101 | 放红 103 |
| `named-unit-on-0-record-on-1` | 手工 1 份，退上一版 | 放绿 78 | 放绿 79 | 放绿 81 | 放绿 80 | 放绿 82 |
| `lag-older-on-0` | 手工 1 份；tail 实例新于所选 | 放绿 141 | 放绿 142 | 放绿 144 | 放绿 143 | 放绿 145 |
| `older-unit-on-0-all-persisted` | 手工 1 份 | 放绿 170 | 放绿 171 | 放绿 173 | 放绿 172 | 放绿 174 |
| `every-data-unit-on-0-all-persisted` | 手工 7 份 | 放绿 113 | 放绿 114 | 放绿 116 | 放绿 115 | 放绿 117 |
| `older-unit-on-1-all-persisted` | 手工 1 份在盘 1 | 放红 163 | 放红 164 | 放红 166 | 放红 165 | 放红 167 |
| 对照 `control-record-on-1-no-damage` | 无 | 放绿 64 | 放绿 65 | 放绿 67 | 放绿 66 | 放绿 68 |
| 对照 `lag-control` | 无 | 放绿 134 | 放绿 135 | 放绿 137 | 放绿 136 | 放绿 138 |
| 对照 `multi-record-control` | 无 | 放绿 149 | 放绿 150 | 放绿 152 | 放绿 151 | 放绿 153 |
| `stale-disk-control` | 旧快照 | 拒 177 | 拒 178 | 拒 180 | 拒 179 | 拒 181 |
| `stale-disk-dead-slots-before-alpha` | 旧快照，两槽全废 | 拒 184 | 拒 185 | 拒 187 | 拒 186 | 拒 188 |
| `stale-disk-after-alpha` | 旧快照，tail 被离线出路写上去，缺 4 个 | 放错，接着全换下、绿 192 | 同左 193 | 同左 195 | 同左 194 | 同左 196 |
| `same-instance-fork-1`、`-3`、`-4` | 分叉，盘 1 差 2 个 | 放错 199、213、220 | 放错 200、214、221 | 放错 202、216、223 | 放错 201、215、222 | 放错 203、217、224 |
| `same-instance-fork-2` | 分叉，盘 0 差 2 个 | 放错 206 | 放错 207 | 放错 209 | 放错 208 | 放错 210 |
| `fork-5-3` | 跨实例分叉，盘 0 缺 4 个 | 拒 495 | **放错 496** | 拒 498 | 拒 497 | 拒 499 |
| `fork-2-0` | 跨实例分叉；今天判缺单元，挂后量到 0/0 | 拒 327 | **放（缺的挂后量不到）328** | 拒 330 | 拒 329 | 拒 331 |
| 跨实例分叉 10 格，五臂都放（下面整行） | 跨实例分叉，盘 1 差 4 个 | 放错 | 放错 | 放错 | 放错 | 放错 |
| 跨实例分叉 37 格，五臂都拒（下面分类） | — | 拒 | 拒 | 拒 | 拒 | 拒 |

同实例分叉 4 行与跨实例分叉的 `follow_on=false`：「`every_arm(&format!("fork-{k_older}-{k_newer}"), &composite, &[], false);`」（`research/prompts/m4-one-copy-behind-refusal-r1-opus-model/src/one_copy_behind_refusal_model.rs:500`）。所以这几行的「放错」只到「挂上时缺」，后果没量。唯一接着走过的邻居是 `stale-disk-after-alpha`，同样盘 0 缺 4 个，改动之后全换下、`violations=0`。**`fork-5-3` 的份量因此没定**：它在判据上打中，放行之后坏不坏没有读数。

按臂收：

| 臂 | 独有的放错 | 拒错 | 和别的臂共有的 |
|---|---|---|---|
| today | 无 | 普通断电加坏一份的 3 形（71、156、106），两故障 1 形（120） | 放错：同实例 4 格、跨实例 10 格、离线出路 |
| c1 字面 | `fork-5-3`（496）；`fork-2-0`（328）判据上不算 | 无 | 放红：盘 1 那两形；放错同上 |
| c1-same | 无 | 无 | 同上 |
| c2 | 无 | 多记录（158） | 同上 |
| c3 | 无 | 两故障（124） | 同上 |

在这张表的形状上，`c1-same` 不差于别的任一臂。这句只到这张表：手挑形状、一次抽样，`c1-same` 与 `c3` 都只在我的模型上量过、被攻过零轮。什么现象推翻：任一条盘面上 `c1-same` 放而 `today` 拒，`differing` 里有不是手工弄坏的单元。

`fork-2-0` 为什么挂后 0/0：`differing` 比的是挂载自己发布之后的现行那一版，不是逐盘核判的那一版（`walk` 在 `MountedSession::of_the_mount` 之后才量）。`k_newer=0` 那条历史只多挂载自己写的单元，再挂一次又重写一遍（推的，拒因整行截到 420 字符，看不到缺的是哪几个）。所以 0/0 不说明当时不缺。今天那一判确实报了 `BehindTheSelectedVersionAndMissingItsUnits`。

`lag-*` 为什么全放：tail 被取号写成 (4,17)，所选还停在实例 3（下面两行是解释，不是判定依据），(4,17) 字典序不落后于 (3,18)，第 7061 行直接返回。这正好是前份 139 那一行「取号之后、第一次发布之前断电」，那一行中。前份 140「写记录时断电」要的盘面（tail (新实例, n−1)、所选 (新实例, n+1)）没造到：模型取的是挂载流里最早那一对记录写，「`let lagging = crash_with_the_record_only_on(&both_records, &remount.writes_of_the_whole_run(), DEVICE_ONE);`」（`research/prompts/m4-one-copy-behind-refusal-r1-opus-model/src/one_copy_behind_refusal_model.rs:582`），只落一块盘之后所选没有前进到实例 4，为什么没前进我没查。

```
139:MODEL_TAILS lag-older-on-0 0:(4,17) 1:(4,17)
140:MODEL_STATE lag-older-on-0 selected=Some((InstanceGeneration(3), CheckpointTxg(12)))
```

跨实例分叉全 49 格的分类（`k_older`、`k_newer` 各取 0 到 6，只把盘 0 换成另一条历史；`放x/y` 是 `differing_after_mount_on_0`/`on_1`；脚本 `/home/fy5090/.local/share/singlefs/work/m4-one-copy-behind-refusal-r1-opus-2/fork_classes.py` 现跑）：

```
共 49 格，5 类
14 格 today=拒:WritableMountRefusedByDevicesWithoutTheSelectedVersion c1=拒:WritableMountRefusedByDevicesWithoutTheSelectedVersion c1-same=拒:WritableMountRefusedByDevicesWithoutTheSelectedVersion c2=拒:WritableMountRefusedByDevicesWithoutTheSelectedVersion c3=拒:WritableMountRefusedByDevicesWithoutTheSelectedVersion
   fork-0-0 fork-0-3 fork-0-6 fork-1-0 fork-1-3 fork-1-6 fork-2-3 fork-2-6 fork-3-3 fork-3-6 fork-4-3 fork-4-6 fork-5-6 fork-6-6
23 格 today=拒:NewerStateStillUnreadableAfterOneReread c1=拒:NewerStateStillUnreadableAfterOneReread c1-same=拒:NewerStateStillUnreadableAfterOneReread c2=拒:NewerStateStillUnreadableAfterOneReread c3=拒:NewerStateStillUnreadableAfterOneReread
   fork-0-1 fork-0-2 fork-0-4 fork-0-5 fork-1-1 fork-1-2 fork-1-4 fork-1-5 fork-2-1 fork-2-2 fork-2-4 fork-2-5 fork-3-2 fork-3-4 fork-3-5 fork-4-4 fork-4-5 fork-5-0 fork-5-1 fork-5-2 fork-5-4 fork-5-5 fork-6-5
1 格 today=拒:WritableMountRefusedByDevicesWithoutTheSelectedVersion c1=放0/0 c1-same=拒:WritableMountRefusedByDevicesWithoutTheSelectedVersion c2=拒:WritableMountRefusedByDevicesWithoutTheSelectedVersion c3=拒:WritableMountRefusedByDevicesWithoutTheSelectedVersion
   fork-2-0
10 格 today=放0/4 c1=放0/4 c1-same=放0/4 c2=放0/4 c3=放0/4
   fork-3-0 fork-3-1 fork-4-0 fork-4-1 fork-4-2 fork-6-0 fork-6-1 fork-6-2 fork-6-3 fork-6-4
1 格 today=拒:WritableMountRefusedByDevicesWithoutTheSelectedVersion c1=放4/0 c1-same=拒:WritableMountRefusedByDevicesWithoutTheSelectedVersion c2=拒:WritableMountRefusedByDevicesWithoutTheSelectedVersion c3=拒:WritableMountRefusedByDevicesWithoutTheSelectedVersion
   fork-5-3
```

五臂都放的 10 格里，等号格 3 格的整行在第一节；另 7 格的整行（`grep -nE` 原样）：

```
376:MODEL_ARM fork-3-0 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=4
377:MODEL_ARM fork-3-0 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=4
378:MODEL_ARM fork-3-0 arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=4
379:MODEL_ARM fork-3-0 arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=4
380:MODEL_ARM fork-3-0 arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=4
425:MODEL_ARM fork-4-0 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=4
426:MODEL_ARM fork-4-0 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=4
427:MODEL_ARM fork-4-0 arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=4
428:MODEL_ARM fork-4-0 arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=4
429:MODEL_ARM fork-4-0 arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=4
432:MODEL_ARM fork-4-1 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=4
433:MODEL_ARM fork-4-1 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=4
434:MODEL_ARM fork-4-1 arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=4
435:MODEL_ARM fork-4-1 arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=4
436:MODEL_ARM fork-4-1 arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=4
523:MODEL_ARM fork-6-0 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=4
524:MODEL_ARM fork-6-0 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=4
525:MODEL_ARM fork-6-0 arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=4
526:MODEL_ARM fork-6-0 arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=4
527:MODEL_ARM fork-6-0 arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=4
530:MODEL_ARM fork-6-1 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=4
531:MODEL_ARM fork-6-1 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=4
532:MODEL_ARM fork-6-1 arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=4
533:MODEL_ARM fork-6-1 arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=4
534:MODEL_ARM fork-6-1 arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=4
537:MODEL_ARM fork-6-2 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=4
538:MODEL_ARM fork-6-2 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=4
539:MODEL_ARM fork-6-2 arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=4
540:MODEL_ARM fork-6-2 arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=4
541:MODEL_ARM fork-6-2 arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=4
544:MODEL_ARM fork-6-3 arm=today mounted units=22 differing_after_mount_on_0=0 on_1=4
545:MODEL_ARM fork-6-3 arm=c1 mounted units=22 differing_after_mount_on_0=0 on_1=4
546:MODEL_ARM fork-6-3 arm=c2 mounted units=22 differing_after_mount_on_0=0 on_1=4
547:MODEL_ARM fork-6-3 arm=c1-same mounted units=22 differing_after_mount_on_0=0 on_1=4
548:MODEL_ARM fork-6-3 arm=c3 mounted units=22 differing_after_mount_on_0=0 on_1=4
```

### 用例（四臂，`guards-*.txt`，量的）

用例 `every_persisted_set_…_mounts_writable_and_is_green_after_the_unmount` 四臂整行（`grep -nF` 原样，每块前面是文件名）与 `test result` 行：

```
guards-c1.txt
11:test every_persisted_set_of_the_batches_on_the_disk_with_the_damage_still_on_it_mounts_writable_and_is_green_after_the_unmount ... ok
13:test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 396.75s
61:test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s
1338:test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 67.09s
1347:test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
guards-c1-same.txt
10:test every_persisted_set_of_the_batches_on_the_disk_with_the_damage_still_on_it_mounts_writable_and_is_green_after_the_unmount ... ok
12:test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 351.01s
60:test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s
1339:test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 96.93s
1348:test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
guards-c2.txt
10:test every_persisted_set_of_the_batches_on_the_disk_with_the_damage_still_on_it_mounts_writable_and_is_green_after_the_unmount ... ok
12:test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 353.26s
60:test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s
1337:test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 70.95s
1346:test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
guards-c3.txt
10:test every_persisted_set_of_the_batches_on_the_disk_with_the_damage_still_on_it_mounts_writable_and_is_green_after_the_unmount ... ok
12:test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 340.55s
60:test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
1337:test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 70.39s
1346:test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

每臂 `ok` 与 `FAILED` 的行数（`grep -cE` 现数）：

```
guards-c1.txt ok=67 FAILED=0
guards-c1-same.txt ok=67 FAILED=0
guards-c2.txt ok=67 FAILED=0
guards-c3.txt ok=67 FAILED=0
```

这几行能说什么、不能说什么：

- 能说：四个测试目标在四臂下都没红，放行臂没有弄坏这 67 个用例。坏盘那条流上四臂都挂上、卸载之后绿。
- **这四个目标不分辨臂。** `c1` 在 `fork-5-3` 的放错、`c2` 在多记录的拒错、盘 1 那一形的红，都不在它们里面（模型才量到）。拿它们当放行的验收，挡不住这几格。
- `today` 臂这一次没跑这四个目标。用例在今天的实现上红，来自背景第二节转述的任务 11560，我没读那份日志，按推的算。
- 日志不打状态数，「153 态」是背景的数，这一次的 `guards-*.txt` 里数不出来。
- 这条用例读的不是跑的那棵树（`rerun-2.attempt.Ggzsl1/` 下只剩 5 份文本，树已不在），读的是同名用例：「`| GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => continue,`」（`/home/fy5090/.local/share/singlefs/work/m4-close-r3-fault-campaign-writer/copy/crates/singlefs-verification/tests/crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs:744`）。表归别的机器判或只录那一支会跳过断言，日志看不出走了哪一支，所以「卸载之后绿」要主 agent 在复跑时核一下。挂载本身每臂都现跑，判定按卸载之后盘的字节进键，臂不进键，只影响同字节的盘，不把别的臂的结论带过来（推的，读码）。

### 没枚举的范围

| 范围 | 取了什么 | 没取的 |
|---|---|---|
| 崩溃状态 | 手挑盘面，不枚举。`MODEL_STREAM writes=29 segment_sizes=[24, 2, 1, 2] closed_form_states=16777223`（`model.txt` 第 61 行）超约 10⁶，按规矩那段不跑 | 记录写段的整写子集全域、原地覆写撕裂。四个用例目标只走它们自己那条流与取法 |
| 几何与负载 | 两块盘、一套参数，一个 7 单元的文件，发布是建 inode 与一次顺序写（`MODEL_FILE data_units=7`、`MODEL_MULTI record_writes=8`） | 别的区域归属、盘数、片数、整理、删除、抬 F 之后的历史 |
| 弄坏的对象 | 手工 flip 一个字节；`older-*` 只坏数据单元；`named-*` 坏记录点名的 `named[0]` | 映射树叶、实例表这些豁免单元的手挑盘面（只在坏盘用例那条流上有）；撕裂 |
| 坏在哪块盘 | 盘 1 只有 `older-unit-on-1-*` 与 `older-on-0-other-on-1-*` | 多记录、差二、两故障、离线出路的盘 1 镜像 |
| 跨实例分叉 | `k_older`、`k_newer` 各 0 到 6，49 格，只把盘 0 换成旧历史 | 换盘 1；k 超 6；分叉之后接着改动、卸载、判 checker（`follow_on=false`） |
| 同实例分叉 | 两边改动数相等，1 到 4，tail 相等 | **两边改动数不等**：tail 落后、窗口非空，这才是 `c1-same`、`c2` 放宽真正生效的那一支。推的：窗口里那条记录点名的单元这块盘也没有，多半照拒（前份 2.3 节），没量。所以「`c1-same`、`c2` 在同实例分叉上不比今天差」**不算量到**；这一步由用户决定的动作没放开扫，定义第 3 步要求放开，这里没做到 |
| 候选 1 后半句 | 没实现，四个放行臂都是删掉窗口外的 | 「按读到一份坏处理」接上之后的全部读数 |
| 放行之后 | 一次挂载、一次建 inode、读回、卸载、判 checker | 下一次崩溃（第 3 问那一半）：放行之后再断电，只有 `lag-*` 这一个不是从放行走过来的形状；放行之后再发一次、只落一份断电（前份 185）没跑 |
| 取号之后写记录时断电 | 没造到 | 前份 140 要的盘面 |
| 判器 | 模型只用池级 checker（CPU） | GPU 判器在这些手挑盘面上没跑 |

## 没打中的形状

这一份没有新攻。前份列的形状照读数：不换盘的 `control`、`older`、`named`、`multi-record`、`lag`、`root-landed` 上，`differing` 都不多于手工弄坏的份数，没有放掉真缺单元。手挑、一次抽样，不支撑「没打中」的结论。

## 这条腿自己的限度

- 只读读数，没编、没跑、没改模型。读数是主 agent 在副本上跑的一次，没有第二次。
- `judge.py` 里的预期是我从前份抄进去的。有三处是我的读法：前份 137 写的是「待跑」，我按同一行「这个洞不是候选带来的」读成 `violations=0`；前份 142 读成 `relocation=` 字段；三行 `today` 前份没写死（前份 53、151、152），标「没写死」，不计中或没中。
- 第 86–89 行的红只说明「不接后半句就红」，第二节写了。`fork-5-3` 的份量没量。
- `differing` 量的是挂载之后的现行版本，不是被判的那一版，`fork-2-0` 就栽在这里。前份 81 的判据建在它上面，这是判据写得不够准。
- 我自提的 `c3` 与 `c1-same` 都是零轮，这张表不改变这一点。

## 要主 agent 跑的命令

这一份没有新命令。判决要引的数，照前份第 213 行那条复跑（主 agent 交，模型 sha256 见开头）。

要定 `fork-5-3` 的份量、造差二那一形、放开同实例分叉的改动数，都要改模型（`follow_on` 改真、取挂载之后第一次发布的那一对记录、`same-instance-fork` 两边 k 分开取）。超出这一次的出口，没做，要不要再派由主 agent 定。

## 没做什么

- 没跑任何东西，没写模型，没动前份与它的模型目录。
- 判决要引的数没复跑。这份里的数全部是任务 11676 副本上的读数，派发提示转述的 `rc=` 没核服务日志。
- 没判正推那几格；不替主 agent 采纳。
- 没读禁读清单里的腿。
- `mutations.tsv` 那 2 行在候选下还红不红，前份没核，这一次也没核。
- 没查盘 1 这一形有没有现成的欠账行；没查 `lag-*` 那条记录为什么没施加。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-one-copy-behind-refusal-r1-opus-2` 里只有 `judge.py`、`expand.py`、`fork_classes.py`、两份模板与编译缓存，没有仓副本与编译目录，无须删。
