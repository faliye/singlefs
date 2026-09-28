# unreadable-at-mount-r1 云端攻方（Opus）报告：W1、W2（2026-09-28）

攻击面：正文「W1　C331 候选在变体与多拒上的结局」「W2　C393 候选在复用与多拒上的结局」。全部数只在冻结副本的拷贝上量过（副本上的数，不是入库装置上的数）。
候选的改法与我加的原型都是我自己的实现：只在我的副本上量过、被攻过零轮。

## 复跑

模型目录 `research/prompts/unreadable-at-mount-r1-opus-model/`。复跑命令（从冻结副本重建六份副本、打 diff、编译、跑、汇总）：

```
bash research/prompts/unreadable-at-mount-r1-opus-model/rerun.sh /tmp/claude-1000/unreadable-at-mount-r1/tree <新的工作目录>
```

每条 cargo 经 `capped.sh 16` 与 `run-with-memory-cap.sh 16G`，只跑 `singlefs-harness` 的四个测试目标（`--release`）。
我这一次的挂钟：C331 网格每份 142–268 秒，C393 网格每份 11–13 秒（`run-all.log` 与 `run_c393.sh` 的输出）。

模型目录每个文件的 sha256（`cd` 到模型目录后 `find . -type f | sort | xargs sha256sum`）：

```
c5e059ae88fb43e17a8a8e1a765ad9cc6c98718b3d634bdd76a206672815a8a0  ./c331_candidates_grid.rs
118a620c94049b6265bdee5d1b7ade9f990e53131a56615be2c226b468c23f41  ./c331-summary.txt
97ae24af50170c1454faa9e1d5a4b7cb045596379c800f2f7c4b1fb37000fd62  ./c393_candidates_grid.rs
e4c615303f2d8402c2f6a1ae62745f487c54aed26f9f724bed5e50600cbf3201  ./c393-summary.txt
d8a55b37406ae7a224f8037148f196c6013da1a2c10a6557472804715a586b4a  ./candidate-a.diff
d30121664bd8e047b36200abb4fa45ef8cafdd563d61dc0bbdc0aaae36ce4d3f  ./candidate-bing.diff
f18dafaea0f93ca3520919028b2269e65925551a58b719a7b32823c34fccd73d  ./candidate-d.diff
ff7db8eaa90956bb1f83483610adfc86bc6ec4304439331291b6b128ddbf804e  ./candidate-jia.diff
3d8b88f96a6bbc1d4cd9f6d96de20f8a6207124d6563ba7c2bceeab380a02569  ./candidate-yi.diff
adcaaf6700723dfa0275230734145832267bc1a2446606078f6299cb226c29c3  ./common-mod.rs.diff
74a1f142296a16aeb5d1158ee1ba8dcd707e7502c38c8c21a1ac38f773a650f3  ./r1-opus-crash-proto/Cargo.toml
91c39cd64dd31e2c5c73d065e4a464c55c29ce0523604bfc73b3a0dd6f6e4549  ./r1-opus-crash-proto/src/main.rs
54e679c3015c4878507bfe9b1d16c7df62867d4acbfdeff0d271be00c0b9649b  ./rerun.sh
89a6ec7976a263049d1a5b172b7820e3590c43ec28bfd755290bf46030bc34b5  ./runs/a-c393_abandoned_root_account_unreadable_at_remount.log
b07976293d93c3cd72cb8b1d16f539bd7891d0e3ac17361dadc70f8d1e5256ac  ./runs/a-c393_candidates_grid-v2.log
d2f7121c33b4b528b58b2c92f8873769cb9ebf87bfb62b7a8b1bb7762657430b  ./runs/base-c331_candidates_grid.log
396fe53e2ec74769012e0384a26bd34f353835ebbd8db5e9e822f3be17ce4a45  ./runs/base-c331_newer_instance_roots_unreadable_then_readable_again.log
f163abd8578fda4566419488f289184d592b4ea419bf20112fd4881dd8d3451a  ./runs/base-c393_abandoned_root_account_unreadable_at_remount.log
10eac755693ebb8634d6274f6262a87a5fc575e5baccf854b7383af7d58907cc  ./runs/base-c393_candidates_grid-v2.log
76bfcda0542617920214f3f897207cd924f5ad531ef5a6d9e3451f8b5cad25bb  ./runs/base-crashproto.log
6121e69e2257e594a2bdcef6b03455f7095f76e72834134c98ce127a541210c5  ./runs/bing-c331_candidates_grid.log
a9bc8e817559252d6bd1c9b8b1e7ee5e99698cf5759038ddfaca975705b60bfb  ./runs/bing-c331_newer_instance_roots_unreadable_then_readable_again.log
ca1b2a11d862b8066382abf51b2aa6cb1ab404c539d3fd52e377c4d9325b40d4  ./runs/d-c393_abandoned_root_account_unreadable_at_remount.log
50a895e8e6ccc801469063775362d60735844b429c32075465b7c34d119ccc1e  ./runs/d-c393_candidates_grid-v2.log
a802a1e9c22647d9a37680fb3b67d27b68710ae3e9761003073f9ff9374342f4  ./runs/jia-c331_candidates_grid.log
a56b57e44491c8e03ce5244bad48e4396d1092a709ded9a806ae44d99380e637  ./runs/jia-c331_newer_instance_roots_unreadable_then_readable_again.log
6feeb75b87ef1ddbbdb4989521003d8051279ee6fcd118691996bd95274aa96d  ./runs/yi-c331_candidates_grid.log
a92729a51a89daa963f5b525bc199e25286fb55b800ca6948369cb2d04630ae3  ./runs/yi-c331_newer_instance_roots_unreadable_then_readable_again.log
a5c7e48c09e40af921053f716b1a29a08c5170f90587b0180c4550b1ab22e89f  ./summarize_c331.py
798df188e2e11ca82e4e20f5ad1d7aa2dd8cffd74ebfd2dce365359d1db36f2e  ./summarize_c393.py
```

`candidate-<副本>.diff` 是相对冻结副本 `crates/` 的改动（`patch -p1` 在副本根打）；`common-mod.rs.diff` 是 `tests/common/mod.rs` 的改动（含调查员那两个读窗口变体，另加 `NthThroughMth` 与 `DeviceWithUnreadableRanges::inner_device`），六份副本都打。
`runs/` 是这一次的原样输出；`c331-summary.txt`、`c393-summary.txt` 是 `summarize_c331.py`、`summarize_c393.py` 在 `runs/` 上的输出。

## 各格判定一览

| 格 | 候选 | 丢写格 / 复用格 | 多拒格 | 判定 |
|---|---|---|---|---|
| W1 | 今天（丁） | 240 / 1959 | — | 出局（V1、V2、V3、V4 都丢） |
| W1 | 甲 | 0 / 1959 | 719 / 1959，其中 3 格是**零读故障**的合法崩溃状态 | 丢写为 0；多拒里有「系统配置槽轮换撕裂」这一合法状态，拒了之后永远拒（推的） |
| W1 | 乙 | 100 / 1959 | 192 / 1959 | 出局：最短反例 V3 形（两块盘最新那一槽持续读不出 + C 的根与一份数据单元读不出） |
| W1 | 丙 | 212 / 1959 | 0 / 1959 | 出局：V1（C331 本身）照丢，V3、V4 照丢 |
| W1 V4 | 已有的「取号那一刻某块盘一份自证过的都没有：拒」 | 冻结副本里已实现，V4 在 144 格里仍丢 20 格 | — | 罩不住 V4（量过）；乙的取号规则就是它，同样罩不住；甲的扩展罩得住 |
| W2 | 今天 | 70 / 98 复用、I-7.4 红 | — | — |
| W2 | (a) 拒可写，重读一次 | 0 / 98 | 28 格拒可写（都是今天会复用的格；今天可写且不复用的格一格没拒） | 复用为 0；拒了之后被抛弃根离不开根环，推的是永远拒 |
| W2 | (d) 从 journal 记录取「写过的槽」 | 16 / 98 | 0 | 出局：被抛弃那次发布的记录两份也读不出时隔离 0、txg 14 复用 |
| W2 | (b)、(c) | 没做成副本 | — | 定义不全，缺的定义见 W2 第 4 节 |

「丢写」「多拒」的口径见 W1 第 1 节；W2 的「复用」按逐字节比被抛弃根那次发布写出的单元、再加池级 checker 的 I-7.4，见 W2 第 1 节。

## W1　C331 候选在变体与多拒上的结局

### 1. 候选怎么做成副本、历史怎么造、怎么判

候选（`candidate-jia.diff`、`candidate-yi.diff`、`candidate-bing.diff`）：

- **甲**：`newer_publish_witness` 里只要有一块盘两槽不全自证过（读不出或自证不过），比较一支就是 `Undecidable`（按真），读阶段重读一遍、仍判不出就拒；取号那一写读见证值时同一条：`self_verified_system_configurations_of_every_device` 从「一槽都没有」收严成「少一槽」就拒。今天那一句是「`if slots_of_this_device.is_empty() {`」（`crates/singlefs-core/src/transaction.rs:654`）。
- **乙**：同一处，一块盘两槽都读不出或自证不过才 `Undecidable`；取号照今天（今天的取号规则就是按盘判）。
- **丙**：重读那一遍的 c_见证 取两遍读到的最大，按重读那一遍的所选那一版与记录重比；取号照今天。今天重读那一遍用的是「`let reread = read_stage(&cache, devices, system_configuration)?;`」（`crates/singlefs-core/src/mount.rs:4249`）读出的那一遍自己的见证。
- **丁**：维持今天，就是 `base` 那一份。

原型用例 `c331_candidates_grid.rs`（四份副本同一份文件），22 个测试、每份 1959 段历史：

- 起点状态五个，每个只建一次池、转成内存稀疏盘，之后每段从内存那一份拷：`standard`（调查员那段：A (1,3)；实例 2 写行 4、暖机 5、B (2,6)、C (2,7)，全部确认返回）、`long`（同上，实例 2 覆盖写四次到 (2,9)）、三个「D 的系统配置轮换撕裂」的崩溃状态（C 之后实例 2 发 D (2,8)，根与记录落盘，轮换那两写按层 0 的第三态撕裂或没持久：`torn_d0` 盘 0 撕、盘 1 没持久；`torn_both` 两块都撕；`torn_d0_d1_persisted` 盘 0 撕、盘 1 持久）。D 没确认返回。
- 每段历史：一组系统配置槽（无、四槽、两块盘最新那一槽、盘 0 最新那一槽、盘 0 较旧那一槽、盘 1 两槽）按一种时机读坏（每次、只第 3 次、只第 4 次、第 3 到第 4 次、第 5 次起；第 3、4 次是两遍见证读，第 5 次起在读阶段之后），再叠一组根 / 记录 / 数据单元（无、实例 2 的全部根与记录、实例 2 的全部根、最新那条根与它的记录、最新那条根与它一份数据单元）按一种时机读坏（每次、只第一次、除第 2 次外每次），读回按块设备错与全 0 各一遍。
- 由用户决定的那几步不写死：挂载做成之后撤故障，新实例接着确认 k 次覆盖写，k = 1、2、3 每一次确认之后都当场「崩溃」、拿那一刻的镜像冷重开只读判一次。
- V4 另有 4 个测试、144 段：第 k 次挂载时一组系统配置槽按一种时机读坏（每次、第 5/6/7/8 次起、第 5 到第 8 次），取号之后第一个非系统配置写报错（取号写落盘、写行之前崩溃）；第 k + 1 次挂载按上面三种藏法之一藏 C，之后同上。

判法：

- **丢写**：挂载选中的那一版 txg 小于最新一次确认返回的发布（`LOST_older_acknowledged`），或新实例某次确认返回的写在当场崩溃冷重开之后只读读不回（`LOST_new_instance`）；两样都有记 `LOST_both`。
- **多拒**：今天那一份 `writable_no_loss`、候选那一份 `refused` 的同一段历史（按整行键对齐，四份副本的键一一对上：`key_mismatch=0`）。
- 撕裂态：层 0 的判据把系统配置槽写认作原地覆写，「`if is_copy_on_write || write.length_in_bytes() <= SECTOR_BYTES {`」（`crates/singlefs-checker-tier/src/crash.rs:1580`）之后要么是基镜像那一段有非零字节、要么更早有重叠的写，都满足，所以取三态。我在原型 crate `r1-opus-crash-proto` 里把 D 那条发布录成流、交 `TearableInPlaceOverwrites::of` 判了一遍（`runs/base-crashproto.log` 原样行）：

```
name=crashproto-segment stream=d_publish segment=3 writes=2 kinds=31:SystemConfigurationSlot@d0+4096:tearable,32:SystemConfigurationSlot@d1+4096:tearable
```

D 的流最后一段就是这两写，所以「前三段全持久 + 这两写各取 没持久 / 撕裂 / 持久」都是层 0 全量枚举会给出的崩溃状态，三个 `torn_*` 起点是其中三个。撕裂字节我按 `torn_image_of_in_place_overwrite` 的算法手抄了一份（前一半新、后一半旧），没调它本身。
整条流的层 0 状态数超过约 10⁶（见「这条腿自己的限度」），没有逐状态枚举。

### 2. 结果（`c331-summary.txt` 原样行）

```
today=base scenarios=1959 verdicts={'refused': 814, 'writable_no_loss': 905, 'writable_LOST_both': 18, 'writable_LOST_older_acknowledged': 222}
== candidate=jia scenarios=1959 key_mismatch=0 verdicts={'refused': 1725, 'writable_no_loss': 234}
   lost_cells=0 over_refused_cells=719 today_lost_now_not_lost=240
== candidate=yi scenarios=1959 key_mismatch=0 verdicts={'refused': 1106, 'writable_no_loss': 753, 'writable_LOST_older_acknowledged': 100}
   lost_cells=100 over_refused_cells=192 today_lost_now_not_lost=140
== candidate=bing scenarios=1959 key_mismatch=0 verdicts={'refused': 842, 'writable_no_loss': 905, 'writable_LOST_both': 12, 'writable_LOST_older_acknowledged': 200}
   lost_cells=212 over_refused_cells=0 today_lost_now_not_lost=28
```

按起点拆（同一文件原样行）：

```
   over_refused by prefix: {'long': 240, 'torn_d0_d1_persisted': 15, 'standard': 240, 'torn_both': 71, 'torn_d0': 153}
   lost by prefix: {'long': 40, 'standard': 40, 'v4': 20}
   over_refused by prefix: {'torn_both': 24, 'standard': 52, 'long': 52, 'torn_d0': 64}
   lost by prefix: {'torn_both': 24, 'standard': 84, 'long': 84, 'v4': 20}
```

（依次是甲的多拒、乙的丢写、乙的多拒、丙的丢写。）

调查员的 11 条用例在四份副本上（`runs/<副本>-c331_newer_instance_roots_unreadable_then_readable_again.log`）：今天 11 条全过；甲 5 条红（V1、V2、V3、V4 四条钉今天坏结局的翻成拒，外加「盘 1 两槽读不出」那条仍拒、只是拒因里比较一支变成 `Undecidable`、按原文比的断言红）；乙 3 条红（V1、V2 翻成拒，「盘 1 两槽」那条拒因变了），V3、V4 两条照过——照旧丢；丙 1 条红（只有 V2 翻成拒），V1、V3、V4 照过——照旧丢。

### 3. 每个候选的反例（最短的那几段）

**乙丢写**：最少四段读故障。两块盘见证 C 的那一槽持续读不出（V3），C 的根与它盘 0 那一份数据单元读不出；乙按盘判时每块盘都还有一槽自证过（tail 6），照今天取 max = 6 = B 的末条，判假，从 B 可写挂载，已确认的 C 丢了。今天、乙、甲同一段历史（`runs/base-…`、`runs/yi-…`、`runs/jia-c331_candidates_grid.log` 原样行）：

```
base: name=c331grid prefix=standard slots=NewestOnBoth slot_timing=Every hidden=NewestRootAndOneDataCopy hidden_timing=Every read_back=DeviceError verdict=writable_LOST_older_acknowledged effective=(2,6) newest_acknowledged=(2,7) read_stage=first[sel(2,6)w6:last_record=6] k1=(3,11)->kept(3,11) k2=(3,12)->kept(3,12) k3=(3,13)->kept(3,13)
yi: name=c331grid prefix=standard slots=NewestOnBoth slot_timing=Every hidden=NewestRootAndOneDataCopy hidden_timing=Every read_back=DeviceError verdict=writable_LOST_older_acknowledged effective=(2,6) newest_acknowledged=(2,7) read_stage=first[sel(2,6)w6:last_record=6] k1=(3,11)->kept(3,11) k2=(3,12)->kept(3,12) k3=(3,13)->kept(3,13)
jia: name=c331grid prefix=standard slots=NewestOnBoth slot_timing=Every hidden=NewestRootAndOneDataCopy hidden_timing=Every read_back=DeviceError verdict=refused mount=Err(NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheS)
```

藏 C 的记录代替藏数据单元（五段）结局相同；这一形在 `standard`、`long` 两个起点、读回块设备错与全 0、藏法各时机下都丢（乙 100 格丢写，都是 `LOST_older_acknowledged`）。

**丙丢写**：C331 本身（V1）照丢。四槽只在第一遍见证读那一次读不出，实例 2 的全部根与记录读不出：丙只合并两遍，第一遍判假就不重读，没有第二遍可合并（`runs/bing-c331_candidates_grid.log`）：

```
name=c331grid prefix=standard slots=AllFour slot_timing=OnlyTheNth(3) hidden=AllRootsAndRecordsOfTheSecondInstance hidden_timing=Every read_back=DeviceError verdict=writable_LOST_both effective=(1,3) newest_acknowledged=(2,7) read_stage=first[sel(1,3)w0:nothing] k1=(3,6)->LOST_final(2,7) k2=(3,7)->kept(3,7) k3=(3,8)->kept(3,8)
```

这一行也是「用户动作不写死」的例子：新实例只确认一次（k1）时 E 被 C 压过，确认两次以上（k2、k3）时新实例的 txg 追过 C、E 活下来。把后缀写死成「写两次」的装置会在这一段上看不到丢写。`long` 起点（实例 2 藏六条根）k1、k2、k3 三次全丢：

```
name=c331grid prefix=long slots=AllFour slot_timing=OnlyTheNth(3) hidden=AllRootsAndRecordsOfTheSecondInstance hidden_timing=Every read_back=DeviceError verdict=writable_LOST_both effective=(1,3) newest_acknowledged=(2,9) read_stage=first[sel(1,3)w0:nothing] k1=(3,6)->LOST_final(2,9) k2=(3,7)->LOST_final(2,9) k3=(3,8)->LOST_final(2,9)
```

**甲**：1959 段里丢写 0。取号扩展那一半在 V4 上起作用：第 k 次挂载的读阶段读得出（最新那一槽第 8 次读起才坏，正落在取号那一读），今天、乙、丙都让取号写把 tail 6 写进见证 C 的那一槽，下一次挂载丢 C；甲在取号之前拒，tail 7 留着（`runs/yi-…`、`runs/jia-c331_candidates_grid.log`）：

```
yi: name=c331v4 mount_k_slots=NewestOnBoth mount_k_slot_timing=FromTheNthOnward(8) mount_k_read_back=DeviceError mount_k=[Publish(PublishSequenceFailed { cause: BlockDevice(InputOutput(Custom ] tails_after_k=[(0, 10, 6), (0, 9, 6), (1, 10, 6), (1, 9, 6)] || name=c331grid prefix=v4_after_mount_k slots=None slot_timing=Never hidden=NewestRootAndRecord hidden_timing=Every read_back=DeviceError verdict=writable_LOST_older_acknowledged effective=(2,6) newest_acknowledged=(2,7) read_stage=first[sel(2,6)w6:last_record=6] k1=(4,9)->kept(4,9) k2=(4,10)->kept(4,10) k3=(4,11)->kept(4,11)
jia: name=c331v4 mount_k_slots=NewestOnBoth mount_k_slot_timing=FromTheNthOnward(8) mount_k_read_back=DeviceError mount_k=[WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_vers] tails_after_k=[(0, 10, 7), (0, 9, 6), (1, 10, 7), (1, 9, 6)] || name=c331grid prefix=v4_after_mount_k slots=None slot_timing=Never hidden=NewestRootAndRecord hidden_timing=Every read_back=DeviceError verdict=refused mount=Err(NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheS)
```

（`jia:` 那一行的 `WritableMountRefusedByDevicesWithoutTheSelectedVersion` 是取号那一读判不出、经 `acquire_expected_instance` 映射过来的拒因，`tails_after_k` 里 tail 7 还在。）

**甲多拒**：719 格。最短的三格一段读故障都没有，是 D 的轮换撕裂的合法崩溃状态：盘上一块盘有一槽自证不过，甲按「判不出」处理、重读一遍仍自证不过、拒可写。今天、乙、丙都从 D (2,8) 照常可写挂载、三次写都读得回（`runs/base-…`、`runs/jia-c331_candidates_grid.log`）：

```
base: name=c331grid prefix=torn_d0 slots=None slot_timing=Never hidden=None hidden_timing=Never read_back=DeviceError verdict=writable_no_loss effective=(2,8) newest_acknowledged=(2,7) read_stage=first[sel(2,8)w7:last_record=8] k1=(3,11)->kept(3,11) k2=(3,12)->kept(3,12) k3=(3,13)->kept(3,13)
base: name=c331grid prefix=torn_both slots=None slot_timing=Never hidden=None hidden_timing=Never read_back=DeviceError verdict=writable_no_loss effective=(2,8) newest_acknowledged=(2,7) read_stage=first[sel(2,8)w7:last_record=8] k1=(3,11)->kept(3,11) k2=(3,12)->kept(3,12) k3=(3,13)->kept(3,13)
base: name=c331grid prefix=torn_d0_d1_persisted slots=None slot_timing=Never hidden=None hidden_timing=Never read_back=DeviceError verdict=writable_no_loss effective=(2,8) newest_acknowledged=(2,7) read_stage=first[sel(2,8)w8:last_record=8] k1=(3,11)->kept(3,11) k2=(3,12)->kept(3,12) k3=(3,13)->kept(3,13)
jia: name=c331grid prefix=torn_d0 slots=None slot_timing=Never hidden=None hidden_timing=Never read_back=DeviceError verdict=refused mount=Err(NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheS)
jia: name=c331grid prefix=torn_both slots=None slot_timing=Never hidden=None hidden_timing=Never read_back=DeviceError verdict=refused mount=Err(NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheS)
jia: name=c331grid prefix=torn_d0_d1_persisted slots=None slot_timing=Never hidden=None hidden_timing=Never read_back=DeviceError verdict=refused mount=Err(NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheS)
```

一段读故障的多拒：盘 0 较旧那一槽持续读不出、别的都读得出（`runs/base-…`、`runs/jia-c331_candidates_grid.log`）：

```
base: name=c331grid prefix=standard slots=OlderOnDevice0 slot_timing=Every hidden=None hidden_timing=Never read_back=DeviceError verdict=writable_no_loss effective=(2,7) newest_acknowledged=(2,7) read_stage=first[sel(2,7)w7:last_record=7] k1=(3,11)->kept(3,11) k2=(3,12)->kept(3,12) k3=(3,13)->kept(3,13)
jia: name=c331grid prefix=standard slots=OlderOnDevice0 slot_timing=Every hidden=None hidden_timing=Never read_back=DeviceError verdict=refused mount=Err(NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheS)
```

这两类拒都在取号之前、盘上一个字节没写，所以下一次挂载看到的盘面一模一样、照样拒（推的：按拒的位置在取号之前推，没在副本上连挂两次）。撕裂的那一槽只有可写挂载的下一次轮换写会盖掉它，可写挂载又一直被拒，于是池只剩只读。零读故障的那三格超出了 C579（判据在单故障合法状态上也拒可写） 那一格（`.claude/kb/checks-owed.md:523`）用户认下的射程：那一格要所选根自己那条记录两份读不出，这里一个读故障都没有。

乙的多拒最短一段：`torn_d0` 盘 0 那块已经撕了一槽，另一槽再在两遍见证读上各坏一次，盘 0 两槽都不在，乙判不出、拒（`c331-summary.txt` 里乙 `over_refused: fewest_fault_ranges=1` 下列的两格）。丙没有多拒。

### 4. 打中之后的四句

| 打中 | 分不分辨臂 | 系统当时看不看得到判别它的东西 | 满足判据字面的哪一个分句 | 跑前条款给的改法在这几格上还中不中 |
|---|---|---|---|---|
| 乙丢写（V3 形） | 分：甲拒、乙与丙与今天丢 | 看不到：挂载那一刻每块盘只剩 tail 6 那一槽，C 的根读不出；「最新那一槽读不出（C 已确认）」与「C 的轮换两块盘都撕了（C 没确认）再加 C 的根读不出」逐项相同（推的，后一形没在副本上造） | 正文第五节 C331 第一条「丢写不为 0 的出局，写明最短反例」 | 跑前条款对丢写只给「出局」一条，没有改法可核 |
| 丙丢写（V1、V3、V4） | 分 | V1：看得到（第一遍见证读四槽都读不出，读得出就不会判「没见证」），丙只是没在第一遍上用它 | 同上 | 同上 |
| 甲多拒（撕裂轮换，零读故障） | 分：只有甲拒 | 看得到撕裂这一件事（那一槽自证不过），看不到它是撕裂还是读坏：同一块盘上「较旧那一槽撕了（合法）」与「最新那一槽读回垃圾（V3 的全 0 读回）」都是一槽过、一槽不过 | 正文第五节 C331 第一条「进表的候选各报多拒格数」：甲进表，报 719 | 条款没给改法；我提的收窄见第 5 节，零轮 |
| V4：已有的取号拒罩不住 | 分：甲罩住，今天、乙、丙不罩 | 看不到：取号那一刻每块盘都有一槽自证过（tail 6），已有的那一句只看「一份都没有」 | 正文第五节 C331 V4「W1 与 W3 都判罩得住才算罩住；有一方判不住就单列成一条候选交用户」：W1 判罩不住 | —— |

### 5. 我提的改法（只在我的模型上量过、被攻过零轮）

- **甲-收窄（推的，没实现，零轮）**：见证读有槽过不了时，只有读阶段同时碰到读不出的根槽、记录或点名单元，才按判不出处理；读阶段别的都读得出就照今天取 max。理由：C 的根、记录、点名单元都读得出时择根就择到 C（或 C 那一版经重放施加），不会压过任何已确认的写，见证值用不上。它想放回的是撕裂轮换那三格。反例我还没找：它对「第 k 次挂载一切都读得出、取号那一读漏掉最新那一槽」的 V4 形不起作用，要另靠取号那一半（甲的取号扩展）挡；而取号扩展在撕裂轮换态上同样拒（取号那一读也看到那一槽过不了），所以只收窄读阶段放不回撕裂那三格。要放回，取号那一半也得跟着改，改成什么我没想好。
- 改法与格对照：

| 改法 | V1 | V2 | V3 | V4 | 撕裂轮换零故障 | 单槽持久读不出 |
|---|---|---|---|---|---|---|
| 甲 | 拒（量过） | 拒（量过） | 拒（量过） | 拒（量过） | 拒，多拒（量过） | 拒，多拒（量过） |
| 乙 | 拒（量过） | 拒（量过） | 丢（量过） | 丢（量过） | 放行（量过） | 放行（量过） |
| 丙 | 丢（量过） | 拒（量过） | 丢（量过） | 丢（量过） | 放行（量过） | 放行（量过） |
| 甲-收窄 | 拒（推的） | 拒（推的） | 拒（推的） | 靠取号扩展拒（推的） | 仍拒，取号那一读（推的） | 仍拒，取号那一读（推的） |

「量过」一律指 `runs/<副本>-c331_candidates_grid.log` 或调查员 11 条用例在副本上的原样结局。

## W2　C393 候选在复用与多拒上的结局

### 1. 候选怎么做成副本、历史怎么造、怎么判

- **(a) 拒可写并报告**（`candidate-a.diff`）：影子账读一条被抛弃根的账，`placements_referenced_by_root` 交 `None` 时先调「重读一次」的钩子、再读一次，仍 `None` 就在取号之前拒可写（新成员 `StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot`）；挂着时抬 F 重算影子账那一路同一条，拒的时候分配器换回抬之前那一份（那一路我没造历史去跑）。今天那一支只做「`unreadable += 1;`」（`crates/singlefs-core/src/mount.rs:1253`）然后跳过。`singlefs-harness` 里三处穷举 `match` 补了新成员，别的没动。
- **(d) 保守隔离**（`candidate-d.diff`）：账读不出时，「写过的槽」取自 journal 环：`scan_journal` 扫出的、实例代号等于这条被抛弃根的全部记录，每条记录点名的每个单元的两条位置条目（记录结构里的「`pub named: Vec<NamedUnit>,`」（`crates/singlefs-core/src/journal.rs:163`））；数据单元与打包单元跨 2 槽、索引节点 1 槽。照今天窄读法减掉候选集里的根与现行那一版还引用的，其余按被抛弃根的落点隔离、记进根环表，这条根离开根环时照今天清；计数照旧加一；不拒挂载。读不出的记录、被环绕掉的记录点名的单元不在其内。
- (b)、(c) 没做成副本，见第 4 节。

原型用例 `c393_candidates_grid.rs`（三份副本同一份文件），14 个测试、每份 98 段：

- 起点两个，每个只建一次：`single` 是调查员那段（A、B → 取号 2 → C (2,8) → 崩溃恢复抛弃 C、落到 (2,7)，实例 3 写行 9、暖机 10 → C 写回）；`double` 在它后面先干净地重开一次（C 读得出、影子账隔离它），再发 D、第二次崩溃恢复抛弃 D，两条被抛弃根。两个起点的原样行（`runs/base-c393_candidates_grid-v2.log`）：

```
start c393grid-double: abandoned[0]=(2,8) units already overwritten at start: []
start c393grid-double: abandoned[1]=(4,14) units already overwritten at start: []
start c393grid-double: C=(2,8) abandoned, newest now (3,10)
start c393grid-double: clean remount instance 4 isolated [(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)]
start c393grid-double: D=(4,14) abandoned, newest now (5,16)
start c393grid-single: abandoned[0]=(2,8) units already overwritten at start: []
start c393grid-single: C=(2,8) abandoned, newest now (3,10)
```

  第一次写 `double` 时没做那次干净重开，D 直接复用了 C 的单元，连「读得出」对照臂都在 txg 17 复用、I-7.4 红（`runs/` 之外的草稿 `base-c393_candidates_grid.log`，已作废不引）；那是 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 一族，不是这一格，所以改了起点。
- 每段：读不出的账（树表、分配记录树根、分配记录树根之下的一个节点、树表与分配记录树根两样），两份按一种时机读坏（不坏、每次、每次直到钩子被调、各只坏第一次），读回按块设备错与全 0；另叠「那次被抛弃的发布的记录两份也读不出」「那个实例的全部记录两份也读不出」两种；`double` 上分别让 C、D、两条都读不出。
- 挂载（实例 4 或 5）之后照调查员的办法一直覆盖写到 txg 31，每写一次逐字节比被抛弃根那次发布写出的每个单元；有一个变了就记「复用」与那一刻池级 checker 的 I-7.4。「分配记录树多层」这一形不用另造：4 GiB 的盘上分配记录树本来就不止一层，`AllocationRecordTreeNodeBelowTheRoot` 那一臂每次都真碰到了两段读故障（`fault_ranges=2`）。

### 2. 结果（`c393-summary.txt` 原样行）

```
TOTAL base: cells=98 {'ok_no_reuse': 28, 'REUSE': 70}
TOTAL a: cells=98 {'ok_no_reuse': 70, 'refused': 28}
TOTAL d: cells=98 {'ok_no_reuse': 82, 'REUSE': 16}
```

(a) 与 (d) 分格（`c393-summary.txt` 里每行是「键 | 今天 | (a) | (d)」；下面按格数计，命令是对那份文件的 `awk`）：

```
今天可写且复用、(a) 拒可写: 28
今天可写且复用、(a) 可写不复用: 42
今天可写不复用、(a) 拒可写: 0
(d) 复用: 16，其中键里带 rec=No 的: 0
```

### 3. 反例与四句

**(d) 复用**：16 格，全是「被抛弃那次发布的记录两份也读不出」或「那个实例的全部记录也读不出」的格，最短是四段读故障（账两份 + 那次发布的记录两份）。记录读不出，(d) 就不知道那次发布写过哪几个槽，隔离 0，txg 14 把 C 的数据单元写掉，I-7.4 红。同一段历史三份副本（`runs/<副本>-c393_candidates_grid-v2.log`）：

```
base: name=c393grid start=c393grid-single abandoned_roots_unreadable_account=[0] account=TreeTable unreadability=EveryRead read_back=DeviceError records_also_unreadable=TheAbandonedPublish fault_ranges=4 writable=ok instance=4 isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] abandoned_roots_unreadable=1 hook_calls=0 failed_reads=4 reuse_txg=14 overwritten=[(0, [(50184, "t1")])] I-7.4=Violated("被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 50184 的那一份与位置条目里的校验和对不上；数据单元两份都读不到对得上的；映射条目指的单元两份都读不到 red=["I-7.4"]
a: name=c393grid start=c393grid-single abandoned_roots_unreadable_account=[0] account=TreeTable unreadability=EveryRead read_back=DeviceError records_also_unreadable=TheAbandonedPublish fault_ranges=4 writable=refused hook_calls=1 failed_reads=6 error=NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot { abandoned_root: RollbackTarget { instance: InstanceGeneration(2), checkpoint_txg: CheckpointT
d: name=c393grid start=c393grid-single abandoned_roots_unreadable_account=[0] account=TreeTable unreadability=EveryRead read_back=DeviceError records_also_unreadable=TheAbandonedPublish fault_ranges=4 writable=ok instance=4 isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] abandoned_roots_unreadable=1 hook_calls=0 failed_reads=6 reuse_txg=14 overwritten=[(0, [(50184, "t1")])] I-7.4=Violated("被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 50184 的那一份与位置条目里的校验和对不上；数据单元两份都读不到对得上的；映射条目指的单元两份都读不到 red=["I-7.4"]
```

同一形去掉记录那两段（只有账读不出）时 (d) 隔离 14、到 txg 31 不复用，与读得出的对照臂一样：

```
d: name=c393grid start=c393grid-single abandoned_roots_unreadable_account=[0] account=TreeTable unreadability=EveryRead read_back=DeviceError records_also_unreadable=No fault_ranges=2 writable=ok instance=4 isolated=[(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)] abandoned_roots_unreadable=1 hook_calls=0 failed_reads=2 reuse=none_through_txg=31 I-7.4=Holds red=[]
```

**(a)**：98 格里复用 0。拒可写的 28 格全是今天会复用的格（「每次都读不出」「读回全 0」各账各起点）；今天可写且不复用的格 (a) 一格没拒。拒在取号之前，盘上不写；被抛弃根只有等后面的发布把它的根槽盖掉才离开根环，而可写挂载一直被拒，所以账持续读不出时池就一直只能只读挂载（推的：按拒的位置推，没在副本上连挂两次）。D23（journal 的角色与格式） 已定项 14 第 385 行要求被抛弃时间线的根离开根环之前它引用的单元不许重新分配，(a) 用「永远不发布」满足它。

| 打中 | 分不分辨臂 | 系统当时看不看得到判别它的东西 | 满足判据字面的哪一个分句 | 改法在这几格上还中不中 |
|---|---|---|---|---|
| (d) 记录也读不出时复用 | 分：(a) 拒、(d) 与今天复用 | 看得到「账读不出、记录也读不出」，看不到那次发布写过哪几个槽：(d) 定义里「写过的槽」只有记录这一个来源 | 正文第五节 C393 那一条「W2 报的复用为 0 且 W3 判不与已定分项冲突，才进交用户的表」：复用 16，不为 0 | 跑前条款没给 (d) 的改法；给 (d) 补「记录也读不出就拒」就成了 (a) 在这几格上的结局（推的） |
| (a) 持续读不出就永远只读 | 不分辨复用：(a) 复用为 0，这一格说的是代价 | 看得到：拒之前重读过一次 | 不是丢写或复用判据的分句，是正文第五节没写的一样东西：拒可写之后能不能再可写 | —— |

钩子那一臂（今天钩子 0 次、(a) 调到了钩子）的原样行，证明 (a) 那一支真走了重读（`runs/a-c393_candidates_grid-v2.log`）：

```
a: name=c393grid start=c393grid-single abandoned_roots_unreadable_account=[0] account=TreeTable unreadability=EveryReadUntilTheRereadHook read_back=DeviceError records_also_unreadable=No fault_ranges=2 writable=ok instance=4 isolated=[(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)] abandoned_roots_unreadable=1 hook_calls=1 failed_reads=2 reuse=none_through_txg=31 I-7.4=Holds red=[]
```

调查员的 9 条用例（`runs/a-…`、`runs/d-c393_abandoned_root_account_unreadable_at_remount.log`）：(a) 与 (d) 各 5 条红，都是钉「隔离 0、计数 1、txg 14 复用」这一今天坏结局的那 5 条；对照、阳性对照、盘上真坏、分配记录树只坏第一次那 4 条照过。

### 4. (b)、(c) 缺哪一样定义（没做成副本）

- **(b) 就地重建，树表那一指称**：树表是权威态（D21（权威态与派生态的分界） 已定项 9），被抛弃根的树表两份读不出时仓里没有第二个来源：那次发布的 journal 记录里只有新树表的指针（`new_tree_table`），没有树表的内容。缺的定义：「从哪算」。定义不全。
- **(b) 就地重建，分配记录树那一指称**：可以写出一个定义——从被抛弃根的树表出发，把它那一版的各棵树（inode 树、extent 树、记账树、中央映射树）连同实例表、树表走一遍，走到的单元落点就是「它引用的」，只在内存里用、不写盘（写回要给被抛弃根的账换落点，而那条根的根槽不许动，写回没有意义）。今天能借的走读是 `rebuild_version_in_the_unit_area_starting_at`，它自己要读分配记录，读不出就失败，所以要另写一份不读分配记录的走读；我没写，没做成副本。它罩不住「树表也读不出」那一格（树表是走读的起点），那一格落回树表那一指称的缺口。
- **(c) 交给 scrub**：`crates/` 里没有 scrub（C393（被抛弃根的账读不出时修复没有条款） 那一行 `.claude/kb/checks-owed.md:327` 记的 grep）。缺的定义：scrub 什么时候跑、它对一条账读不出的被抛弃根做什么、「scrub 之前的复用怎么挡」。第三样一旦写成「挡」，就是 (a)（拒）或 (d)（隔离）之一换了个名字；写成「不挡」，就是今天。定义不全。

## 没打中的形状

- 甲的丢写：1959 段、五个起点（含三个撕裂轮换的合法崩溃状态）、六组系统配置槽 × 五种时机 × 五种藏法 × 三种藏法时机 × 两种读回，外加 144 段两次挂载的 V4，新实例确认 1–3 次，每一次确认之后都判——丢写 0 格。没有试的：一块盘的池（冻结副本的可写挂载要求两块盘，`admit_the_writable_device_count` 拒一块盘，没法造）；读阶段之后、取号之前别的直接读盘的几处（`first_txg_of_new_instance`、影子账择根）单独读坏；新实例确认 4 次以上；实例 2 覆盖写 3 次、5 次以上的起点。
- 丙的多拒：0 格，在同一批 1959 段里。
- (a) 的复用：98 段里 0 格；没试挂着时抬 F 重算影子账那一路（补丁打了，没造历史跑）。
- (a) 在「今天可写且不复用」的格上的多拒：0 格（98 段里今天可写不复用的 28 格全是读得出的对照或只坏第一次、重读读得出的格）。
- 三种新形态（被抛弃根多条、分配记录树多层、读回全零）今天都复用、I-7.4 红，(a) 在其上都拒，(d) 在记录读得出时都不复用。

## 这条腿自己的限度

- **层 0 枚举没跑成**：原型 crate `r1-opus-crash-proto` 把 D 那条发布、V4 第 k 次挂载那条流录下来，先用 `layer0_state_count_with_torn_in_place_overwrites` 算状态数，都超过约 10⁶，照派发提示那条规矩不跑（`runs/base-crashproto.log` 原样行）：

```
name=crashproto-skip stream=d_publish reason=layer0_state_count_over_10^6 layer0_state_count_with_torn=268435468
name=crashproto-skip stream=v4_mount_k_newest_slots_every_read reason=layer0_state_count_over_10^6 layer0_state_count_with_torn=4325418
```

  一次发布第一段 28 次 COW 单元写，单这一段就是 2²⁸ 个子集；我没找到更短、又能走到这几格的发布流。所以这一腿的原型全量状态合计 0 个；撕裂轮换那三格是我按层 0 的第三态算法手造的状态，属于层 0 会枚举到的集合（段结构与 `:tearable` 标记见 W1 第 1 节那一行），但没有经 `enumerate_layer0*` 逐状态走过。
- 只在一组几何上量：两块盘、区域归属 [0, 1, 0]、4 GiB 盘、默认环长、每次覆盖写一条记录、单文件。
- 读故障只有「块设备错」「读回全 0」两种；没造读回别的垃圾。
- 我的候选实现是我对正文候选文字的一种读法：甲的「读不出或自证不过」我按「一块盘两槽没全过」实现（fsid 不同也算不过）；乙按「一块盘两槽都没过」；丙只动重读那一遍；(d) 的「写过的槽」取 journal 记录。别的读法没量。
- 副本上的数不是入库装置上的数；要引，得在入库装置上重做。

## 没做什么

- 没判 W3、W4 两格；没替主 agent 采纳。
- 没跑重型测试：没跑 checker 档包的测试、层 0 用例、54/55/57/59/87 号；原型里调的是 checker 档库的计数与撕裂判据函数，没有跑它的测试二进制，也没有枚举状态。
- 没在副本上连挂两次去坐实「甲、(a) 拒了之后永远拒」，那两句是推的。
- (b) 分配记录树那一指称的走读没实现。
- 交回前删了：编译目录 `/tmp/claude-1000/unreadable-at-mount-r1/opus/target-base`（1.9G）、`target-jia`（362M）、`target-yi`（362M）、`target-bing`（360M）、`target-a`（354M）、`target-d`（354M）；仓副本 `/tmp/claude-1000/unreadable-at-mount-r1/opus/base`（9.8M）、`jia`（9.8M）、`yi`（9.8M）、`bing`（9.8M）、`a`（9.7M）、`d`（9.7M）；临时目录 `tmp`（4.0K）。草稿目录里留着日志、脚本与汇总，模型目录里有复跑要的全部文件。
