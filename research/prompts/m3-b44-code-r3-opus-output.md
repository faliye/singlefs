# m3-b44-code-r3 攻方腿（Opus）报告（2026-10-18）

攻击面：正文「三、格」L1、L2、L3、L5，L4、L6 只做了读码。量法全在冻结副本的拷贝上（`~/.local/share/singlefs/work/m3-b44-code-r3-opus/tree/`，拷来后按 `research/prompts/m3-b44-code-r3-snapshot/crates-sha256.txt` 核过 118 份全对上）；冻结副本本身没动。下面引代码的 `crates/…:行号` 是冻结副本的行号，被引的九份文件与主工作区此刻逐字节相同（`cmp` 过）。副本上量出的数只算副本的数。

## 复跑

```
bash scripts/hosts/run-with-memory-cap.sh 14G bash research/prompts/m3-b44-code-r3-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs \
  --frozen /home/fy5090/.local/share/singlefs/work/m3-b44-code-r3/repo \
  --work <新的草稿目录>
```

它把冻结副本拷三份：原样（after）；照 `research/prompts/_m3-b44-code-r3-diff.md` 倒着撤回 W13 之前（before，七份文件撤完与 diff 头上 `index` 那一栏的 W13 之前的 blob 逐份相同，`git hash-object` 核过；另撤 harness 库里两处引新错误成员的分支，`patch/before-w13-harness.patch`）；原样加两处打印与一个只供原型的开关（observed，`patch/observed-mount.patch`：打出取号之前预演续做走到哪儿，设 `PROBE_WITHOUT_THE_READ_ERROR_REFUSAL` 时跳过读错那一拒，不设时行为与原样相同）。三份各编一次 release，再跑读迹、点名计划、两版对照扫描、整池读序窗口扫描（单线程顺序跑，本机估 40 分钟以上；我自己是按「批」并行跑的，`tools/run-batch.sh`）。`tools/rerun.sh --selftest` 里「拷进与仓无关的临时目录、传参应当跑通」那一格红：它拿假仓跑 `rerun_steps`，这一段要真的原型文件与 diff，第二轮攻方的 `rerun.sh` 同样红在这一格；另外四格（缺参、缺 `--frozen`、`--repo` 不是仓、`--work` 非空）照模板。

模型目录 `research/prompts/m3-b44-code-r3-opus-model/` 每个文件的 sha256：

```
a87f0de628007fa8fd793a6c00aefde69670c2ef5776968fc9002300fea20d8c  ./attack_shared_reads_probe.rs
6923f261506e98bdc309d602af770d2f6d5c6e40863343557362c08a9ee296a2  ./out/counterfactual.log
bc64fac80ac65b94f2540241f79fe06536a1b46c145fe972265df9b0d4fb187d  ./out/fault-free-trace.log
4b2fee5a5229f3a4d036366b4455c32abde361011e74b950a7f3b490fc03054e  ./out/follow-up.log
59be18a43fff262cce2dee8476e513dfda1167e501fbf9982435afb13cb0ba61  ./out/fsid-fixtures.log
3f9eb4c45fea994442197618041ead7588afbe0c7c81989a52f91f2bb5ea7f4f  ./out/pair-summary.log
52554a525d15217f898e80573ddb4567c8ac3ef4b3754025fec002fdb1840a2b  ./out/rehearsal-endings.log
1bf6e7ec0e662cc0cf598c9d307160d944aa7875dcd4ceb9a177431d3df7bb2b  ./out/sysconf-writes.log
090c9d73207c07607cbb7d874b0078a5021161a9d8904b4cd0d53e6c8ce8a56f  ./out/sysconf.log
092e9e17997a8e23bf7127d6d0f46edff6e728cdc41fb0f0c6c8d86b7ebd10b0  ./out/trace-after-corrupted.txt
cf07c36507f59c0c2e2e9cffee238c5e542949034b1d121fbc836bf12a38289e  ./out/trace-after-unchanged.txt
5e188d6bd1c15c7bd498dea8bd1b28772a67bd41d69f0576a4651961515c03fe  ./out/trace-before-corrupted.txt
62d73145ad318454070c7deddee27eeb2c76d208b3cefc61c889c23d4f9a075d  ./out/trace-before-unchanged.txt
68a4b42da0e90e4521cd94db8de5bb94284ce94ed2c42ed39c265420eebb477d  ./out/window-hits.txt
c5fdc55c79b1a346357e1409f23b765491951d433298f620f6fae94c39c4cc37  ./out/window-summary.log
63204b1cff961ee8175cdea320c58d578731c6b6088b036872bb88c633e5a21c  ./patch/before-w13-harness.patch
34e903f23a2ca8db642bd8c68dda655a9b7aa9f8ce81a7e398856585599f1410  ./patch/observed-mount.patch
bfdb1b91e513bcbbcb32cc3c64b1656c16efb7320c8f53dd7920f15513fefd35  ./tools/rerun.sh
3d04812673c788e655abf01e285233f638798590c849344383d6aa23c4900ed6  ./tools/run-batch.sh
```

镜像就是 W13 用例那张窄盘（第二轮攻方 `narrow_image` 原样：304 槽、第一个文件 16 个单元、删除队列 {1}、一批一个单元、退役容器一片 1 条），「没改坏」与「改坏」（退役树唯一那片号改成 u64::MAX，续做最后一批报 C841）两份。盘是原型自己写的一层 `ProbeDisk`，垫在交给挂载的位置，也就是 `DeviceReadOnceInThisMount` 底下：它数到的读都是穿到盘上的读。故障两种：`fail` 读报块设备错；`garbage` 读报成功、第 0 字节翻最低位（读得出、自证不过）。点法三种：整池第 n 次读起连着 w 次（`w:n:w`）；某块盘某偏移的头 k 次（`o:…`，这一轮没进结论）；两块盘同一偏移在整池第 a 次读之后各自的头 k 次（`p:偏移:a:k`，一个单元的两份）。

## 各格判定一览

| 格 | 判定 | 一句话 | 量的还是推的 |
|---|---|---|---|
| L1 | **打中（两形）** | ① 只进内存那一趟的取号读共用那一份、真做取号读盘：盘 0 两个系统配置槽在这次挂载头一次读时读回坏字节（整池第 303、304 次读，取号之前），预演在它自己的取号上停下、读错账空，交给真做；真做取号读盘读到好的，续做走到最后一批交 C841，19 次发布已落盘。W13 之前同一个故障是取号之前交 C841、盘不变。② 取号那一写算「写哪一槽」的读走共用那一份：盘 0 最新那一槽头一次读回坏字节一次（第 303 次读），取号写把盘 0 **最新**那一槽（世代 10）覆写掉，与 `crates/singlefs-core/src/transaction.rs:1086` 写的「覆写旧槽」相反；W13 之前同一个故障盘面与无故障逐字节相同 | 量的（副本） |
| L1 | 没打中 | 无故障时两张盘可写挂载写出的字节与 W13 之前逐字节相同（摘要 `826326e3fd072b73` 两版一样，写 508 次一样） | 量的 |
| L1 | 量了、不判 | 共用那一份的上界：这张窄盘一次挂载穿到盘上读过或写过的扇区 17320 个（8.9 MB），其中写 14324 个（7.3 MB，续做 16 步每步约 0.46 MB）；挂载交回就丢 | 量的；大删除规模下是推的 |
| L2 | **打中（条款没写的那一形）** | 两份都读回坏字节（不是读错）时 ⓑ 不拒：续做推的抬 F 算上限读一条有效根的树表，两份头一次读都坏，预演停下、读错账空，交给真做；真做取的是记下的坏字节，抬 F 照样失败，交 `FloorRaiseFailedAfterTheMountsPublishes`，写行、暖机已落盘。没改坏的盘上之后不带故障再挂：第 1 次做成（续做 0 步），第 2–5 次都在取号之前拒（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`）。W13 之前同一个故障挂载做成、盘与无故障逐字节相同、之后 5 次都做成 | 量的（副本） |
| L2 | 量了：ⓑ 偏拒 | 没改坏的盘上宽 2 的读错窗口 411 个里 ⓑ 拒 7 个（1.7%），宽 1 的 0 个；关掉这一拒，这 7 个全做成、盘与无故障逐字节相同；拒了的盘不变、再挂一次做成（续做 16 步）。改坏的盘上同 7 个窗口关掉这一拒就是 C841 落盘之后才报：ⓑ 拒的那一刻两张盘看到的一样，分不出 | 量的（副本） |
| L3 | 部分打中 | 坏字节第一次被记下之后，共用那一段里谁再读都取它，只有当场重读那几处穿到盘上；没有当场重读的读（上一行 L2 那一读、L1 ② 那一读）整次挂载都看坏字节，W13 之前真做会自己再读一次盘。W16 没改的两处：根环槽单次读坏（第 295–302 次读）盘面与无故障相同，没打中。默认实现走故障注入那一层（读码：harness 与 checker 档里没有一个 `BlockDevice` 覆写这两个新方法） | 量的 + 读码 |
| L5 | **打中（条款没写的那一形）** | 单元头只带 fsid 的低 8 字节：仓里自己用来当「另一个池」的 `*b"singlefs-earlier"` 与本池 `*b"singlefs-rev-a2b"` 记进单元头都是 `7366656c676e6973`，W14 那一判把它们当本池；仓里 21 个 16 字节字面量（大多是测试池 fsid）有 18 个以 `singlefs` 开头。mkfs 不生成 fsid。误拒本池的没找到 | 量的（8 字节相等）；整张镜像没造 |
| L4、L6 | 没攻 | 只读码：deleted_inodes 写者只写 8 字节条目，`!= 8` 不误拒；`data_unit_payload` 与 `parse_data_unit` 用同一个界 | 读码 |

## 各格答四句的总表（打中的几格）

| 打中 | 分辨臂吗 | 系统当时看得到判别它的东西吗 | 满足判据字面的哪一句 | 跑前条款给的改法在这几格上还中不中 |
|---|---|---|---|---|
| L1 ① | 分辨：只打 W15 收窄 ⓐ 的取号读那一半（真做读盘、预演读共用那一份）；两边都读共用那一份或都读盘就不中 | 看得到：预演的取号读到自证不过的系统配置，读者知道，但没记进读错账 | L1 验收第一句「让预演与真做……走出不同的结局」（预演停在取号，真做走到续做 C841） | 跑前条款只给「改法并进修红批」，没给具体改法；见后面改法表 |
| L1 ② | 分辨：只打「取号写起切回共用的那一份」那一段里算世代号的读 | 看得到：盘上那一槽是好的，取号写之前那几次见证值读刚读过它 | L1 验收第一句后半「照旧该读盘的……取号写之前那几次读……有一次读漏进了共用那一份」——交主 agent 判这一读算不算「取号写之前那几次读」 | 同上 |
| L2 坏字节那一形 | 不分辨 ⓑ 的两种读法：ⓑ 只看读错账，坏字节根本不进账 | 看得到：预演停下、两份都没核过校验和，读者知道 | 不落在 L2 验收的哪一句：拒没发生，不是「拒了之后盘变了」；按跑前条款第三行交用户 | 同上 |
| L5 | 不分辨：任何只比单元头 8 字节的判法都中 | **看不到**：单元头里没有 fsid 的高 8 字节，两个池在头里逐字节相同（「判别子观测不到」那一形） | L5 验收后半「一份别池单元头仍被放过的镜像」 | 改判据是新一轮的事；交用户 |

## L1：读一次两边共用

### 打中 ①：只进内存那一趟的取号读共用那一份，真做取号读盘

历史（改坏的那份窄盘，`follow_up` 计划 `w:303:2:garbage`）：不带故障时整池第 303、304 次读是盘 0 两个系统配置槽（`out/trace-after-corrupted.txt` 第 303、304 行 `dev=0 off=0 len=4096`、`dev=0 off=4096 len=4096`），两版二进制前 306 次读逐行相同（`cmp` 两份读迹：「differ: byte 15250, line 307」，两张镜像都是）。让这两次读回坏字节、别的读照常：

| 一步 | 被判对象里许可它的那一句 |
|---|---|
| 第 303、304 次读是这次挂载头一次读盘 0 两槽（算要取的号），读成了，逐块记进共用那一份——坏字节照记 | `crates/singlefs-core/src/mounted_session.rs:680`：「recorded.insert(block_index, block.to_vec());」（读成就记，不看自证） |
| 只进内存那一趟走取号，取号那几次读经它底下那块盘读共用那一份，两槽都自证不过，取号失败，这一趟停在续做之前 | `crates/singlefs-core/src/mount.rs:6877`：「ReadsOfTheAcquisition::ThroughTheDevicesOfTheRehearsal => {」 |
| 预演结局是「没走到」，读错账空（坏字节不是读错），照旧交给真做 | `crates/singlefs-core/src/mount.rs:5365`：「// 这两趟读盘没出过读错：照旧留给续做（岔路单那一行只定了 C841）。」 |
| 真做取号写之前那几次读穿到盘上，读到两槽好的，取号成功 | `crates/singlefs-core/src/mount.rs:6869`：「ReadsOfTheAcquisition::StraightToTheDiskOfTheRealRun(read_route) => {」 |
| 真做写行、暖机、续做，最后一批开退役容器交 C841 | 第二轮判决 J2 那一形的结局 |

原样输出（`out/rehearsal-endings.log`，observed 那一份；`out/follow-up.log`，after / before 两份）：

```
PROBE_REHEARSAL ending=stopped-before-the-end reads_failed_in_the_two_passes=0
after follow image=corrupted plan=w:303:2:garbage first mount C841-after-writes(persisted=19) disk-other all-before-the-first-write digest_after=fa9ca4e921cac055 fired_at=[(303, 0, 0, 4096), (304, 0, 4096, 4096)] first_write_at=Some(351)
after follow image=corrupted plan=w:303:2:garbage remount 1 without fault C841-before-acquisition disk_changed_by_it=false
after follow image=corrupted plan=w:303:2:fail first mount C841-before-acquisition disk-unchanged all-before-the-first-write(no-write) digest_after=a481abfc0d9b441b fired_at=[(303, 0, 0, 4096), (304, 0, 4096, 4096)] first_write_at=None
before follow image=corrupted plan=w:303:2:garbage first mount C841-before-acquisition disk-unchanged all-before-the-first-write(no-write) digest_after=a481abfc0d9b441b fired_at=[(303, 0, 0, 4096), (304, 0, 4096, 4096)] first_write_at=None
```

整池读序窗口扫描（`out/window-summary.log`，改坏那份 360 个起点）里宽 2 的坏字节窗口有 2 个是这一形（起点 303、305），宽 3 的 4 个（302–305）；同样位置的读错窗口一个都不是（读错不进共用那一份，只进内存那一趟取号时再读一次盘、读到好的）。这几个窗口里注进去的故障全在这次挂载第一个写之前。

四句：分辨臂——只打 W15 收窄 ⓐ 的取号那一半（真做「取号写之前那几次读」读盘、只进内存那一趟的同一段读共用那一份）；两边都读共用那一份时真做取号同样失败、一个写都不发，两边都读盘时就是 W13 之前那一列，都不中。系统看得到——只进内存那一趟取号读到两槽都自证不过，读系统配置的那一层知道，只是没进 `reads_failed_on_the_disk`。满足 L1 验收第一句「让预演与真做……走出不同的结局」：预演停在取号，真做走到续做交 C841；字节也不同（同一槽，预演读的是记下的坏字节，真做读的是盘）。

### 打中 ②：取号那一写算「写哪一槽」的读走共用那一份

历史（没改坏的那份，计划 `w:303:1:garbage`）：只让第 303 次读（盘 0 槽 0，世代 10，这块盘最新那一槽）读回坏字节一次。

| 一步 | 被判对象里许可它的那一句 |
|---|---|
| 坏字节记进共用那一份（同上） | `crates/singlefs-core/src/mounted_session.rs:680`：「recorded.insert(block_index, block.to_vec());」 |
| 真做取号写之前那几次读穿到盘上（见证值读得好） | `crates/singlefs-core/src/mount.rs:6869`：「ReadsOfTheAcquisition::StraightToTheDiskOfTheRealRun(read_route) => {」 |
| 取号写起切回共用那一份 | `crates/singlefs-core/src/mount.rs:6836`：「/// 真做：取号写之前那几次读（重算要取的号、读上一个号、屏障前后各读一次见证值）穿到盘上；取号写起切回共用的那一份」 |
| 写槽之前在这块盘两槽里取自证过的最大世代号，槽号按新世代号 mod 2：共用那一份里槽 0 自证不过，只剩槽 1（世代 9），新世代 10，写槽 0 | `crates/singlefs-core/src/transaction.rs:509`：「/// 一块盘写一次系统配置槽：世代号 = 这块盘两槽里自证过的最大世代号 + 1（两槽都读不出时从 1 起），槽 = 世代号 mod 2；」；`crates/singlefs-core/src/transaction.rs:520`：「let verified_on_this_device = verified_system_configuration_slots(」 |

取号写覆写了盘 0 最新那一槽。这一侧要的是覆写旧槽：`crates/singlefs-core/src/transaction.rs:1085`：「取号写覆写的是这块盘上较旧的那一槽，另一槽可能是上一次发布末尾轮换刚写的」；`crates/singlefs-core/src/transaction.rs:1086`：「两槽轮换要的是…覆写旧槽时新槽已持久」。原样输出（`out/sysconf-writes.log`，按先后列系统配置那两段上的头 4 个写；`out/sysconf.log` 列挂载之后两槽的世代号）：

```
after sysconf-writes image=unchanged plan=none outcome=Ok(steps=16) first_system_configuration_writes=[dev=0 slot=1 len=4096 after_reads=410; dev=1 slot=1 len=4096 after_reads=410; dev=0 slot=0 len=4096 after_reads=410; dev=1 slot=0 len=4096 after_reads=410]
after sysconf-writes image=unchanged plan=w:303:1:garbage outcome=Ok(steps=16) first_system_configuration_writes=[dev=0 slot=0 len=4096 after_reads=410; dev=1 slot=1 len=4096 after_reads=410; dev=0 slot=1 len=4096 after_reads=410; dev=1 slot=0 len=4096 after_reads=410]
after sysconf-writes image=unchanged before system-configuration dev=0 slot0:generation=10,instance=1,tail=23,floor=0 slot1:generation=9,instance=1,tail=22,floor=0
before sysconf-writes image=unchanged plan=w:303:1:garbage outcome=Ok(steps=16) first_system_configuration_writes=[dev=0 slot=1 len=4096 after_reads=1159; dev=1 slot=1 len=4096 after_reads=1190; dev=0 slot=0 len=4096 after_reads=1229; dev=1 slot=0 len=4096 after_reads=1260]
after sysconf image=unchanged plan=w:303:1:garbage after system-configuration dev=0 slot0:generation=30,instance=2,tail=42,floor=21 slot1:generation=31,instance=2,tail=43,floor=21
```

（`after_reads=410` 那一个写就是这次挂载的第一个写，即取号写。）W13 之前同一个故障，第一个写照旧写盘 0 槽 1，挂载之后盘面与无故障逐字节相同（`out/follow-up.log`：「before follow image=unchanged plan=w:303:1:garbage first mount Ok(steps=16) disk-as-fault-free」）。之后的后果是推的、没量：取号写正写到一半时崩溃，盘 0 只剩世代 9 那一槽（tail 22），丢的是上一次轮换见证过的 tail 23；盘 1 那两槽不受影响，池级取 tail 的最大值时还取得到 23。崩溃点枚举要依赖 checker 档包，原型里没有跑，命令写在「要主 agent 跑的命令」。

同一个计划下盘 0 两槽都读回坏字节（`w:303:2:garbage`）时第一个写照旧落在槽 1，但新世代从 1 起（挂载之后盘 0 两槽世代 22 / 21，无故障是 32 / 31，`out/sysconf.log`）：取号写之后、下一次轮换之前，盘 0 按世代号选出来的最新一槽仍是取号之前那一份（世代 10、实例 1）。这一句是按两槽的世代号推的，中间那一刻的盘面没打出来。

四句：分辨臂——只打 ⓐ 里「取号写起切回共用的那一份」这一段里算世代号的读；`crates/singlefs-core/src/mount.rs:6836` 把它划在「写」那一半，它决定写哪一槽，按 `crates/singlefs-core/src/transaction.rs:1085` 那一句它要的是这一刻盘上哪一槽旧。系统看得到——见证值读刚穿到盘上读过这一槽、读到好的。满足 L1 验收「照旧该读盘的……取号写之前那几次读……有一次读漏进了共用那一份」，**这一读算不算「取号写之前那几次读」交主 agent 判**；不算的话，它是 L1 判什么一栏「会不会改掉……挂载写出的字节」在带故障时的那一半，落在条款没写的地方。

### 没打中：无故障时写出的字节

两版二进制、两张镜像不带故障各挂一次（`out/fault-free-trace.log` 原样）：

```
after name=fault_free_trace image=unchanged outcome=Ok(steps=16) reads=411 reads_before_the_first_write=410 writes=508 distinct_offsets_read_before_the_first_write=155 distinct_offsets_read=156 blocks_read=4296 blocks_written=14324 blocks_touched=17320 disk_changed=true digest_before=5163c2b7f7323a1b digest_after=826326e3fd072b73
before name=fault_free_trace image=unchanged outcome=Ok(steps=16) reads=3104 reads_before_the_first_write=1159 writes=508 distinct_offsets_read_before_the_first_write=155 distinct_offsets_read=468 blocks_read=15080 blocks_written=14324 blocks_touched=17320 disk_changed=true digest_before=5163c2b7f7323a1b digest_after=826326e3fd072b73
```

摘要是两块盘非零扇区的 `DefaultHasher`（固定键），同一份工具链编的两版比得了；写次数、写过的扇区数也一样。只在这一种几何、这两段历史上量过。

### 记下那一份占的内存

上一段那一行：这次挂载穿到盘上读过 4296 个扇区、写过 14324 个，合起来 17320 个扇区（512 字节一个，8.9 MB）——共用那一份至多记这么多（读盘那三段读的不记，实际只会更少）。写那一半是真做写的每一块（W13 之前不记），续做 16 步、一批一个单元，每步约 0.46 MB，与第二轮量的只进内存那一趟每步约 0.41 MB 同一个量级，两份同时活着。删大文件时按步数线性长是推的；它以这次挂载碰过的不同块为上界、不超过两块盘的大小，会不会撑爆一次合法挂载取决于续做一次挂载里走多少步，交岔路二的计数实验量。

## L2：预演读错在取号之前拒

### 量过：ⓑ 拒的比例、拒了之后、关掉它会怎样

整池读序窗口（`out/window-summary.log`，没改坏那份 411 个起点 × 宽 1、2、3，读错）：宽 1 的 411 个里 ⓑ 拒 0 个；宽 2 的拒 7 个（起点 352–358）；宽 3 的拒 7 个（同一段起点，`fired=2`：拒在窗口第三次读之前）。这 7 个都是续做推的抬 F 算上限读一条有效根的树表，两份（盘 0、盘 1 同一偏移）连着读错（读迹第 352–358 行是七条有效根的树表，盘 0 那一份）。原样（宽 2 那一行）：

```
window image=unchanged kind=Fail summary 7 × width=2 ReadFailedRefusalBeforeAcquisition disk-unchanged all-before-the-first-write(no-write) fired=2
```

拒了之后：盘逐字节不变（`disk-unchanged`），不带故障再挂一次做成、续做 16 步走到空（`out/follow-up.log`：「after follow image=unchanged plan=p:23101440:306:1:fail remount 1 without fault Ok(steps=16) disk_changed_by_it=true」，之后 4 次都做成）。L2 验收前半「这一拒之后盘变了、或再挂一次不带故障走不完」没打中。

关掉这一拒（observed 那一份设 `PROBE_WITHOUT_THE_READ_ERROR_REFUSAL`，`out/counterfactual.log`）：没改坏那份 7 个窗口 7 个都做成、盘与无故障逐字节相同；改坏那份同 7 个窗口 7 个都是 C841 落盘之后才报。原样各一行：

```
PROBE_REHEARSAL refusal-skipped would_refuse=true
follow image=unchanged plan=w:352:2:fail first mount Ok(steps=16) disk-as-fault-free all-before-the-first-write digest_after=826326e3fd072b73 fired_at=[(352, 0, 23101440, 16384), (353, 1, 23101440, 16384)] first_write_at=Some(397)
follow image=corrupted plan=w:352:2:fail first mount C841-after-writes(persisted=19) disk-other all-before-the-first-write digest_after=613e696b5060267a fired_at=[(352, 0, 23101440, 16384), (353, 1, 23101440, 16384)] first_write_at=Some(397)
```

所以 ⓑ 在这张没改坏的盘上拒的 7 个挂载，真做本来全走得通（L2 验收「ⓑ 偏拒把真做本来走得通的挂载拒了」，比例 7/411，宽 2）；同一读错窗口落在改坏那份上，不拒就是第二轮 J2 那一形。拒的那一刻两份盘上预演停在同一处、读错账同样两条，后面会不会撞 C841 在停下的那一处之后，系统看不到——这是「判别子观测不到」那一形，不是 ⓑ 写错。代价：一次挂载被拒、盘不变、重挂做成。W13 之前同一个故障（`p:<偏移>:306:1:fail`，两版对照见下一节）没改坏那份 61 个偏移全做成。宽 2 以上的读错窗口里「被另一份兜住的那次读错也算」那一句没有多拒出一个（读错被兜住时预演走到底，宽 1 的 0 个）。

### 打中：两份都读回坏字节时 ⓑ 不拒，真做取同一份坏字节、落盘之后失败

历史（计划 `p:23101440:306:1:garbage`：两块盘偏移 23101440 那一份树表，在整池第 306 次读之后各自的头一次读读回坏字节；after 那一版落在第 352、353 次读，before 那一版落在第 961、962 次读，都是这次挂载取号之前头一次读这一份）：

| 一步 | 被判对象里许可它的那一句 |
|---|---|
| 只进内存那一趟抬 F 算上限，两份头一次读都读成、都记下（坏字节照记），两份都核不过校验和 | `crates/singlefs-core/src/mounted_session.rs:680`：「recorded.insert(block_index, block.to_vec());」；`crates/singlefs-core/src/recovery.rs:572`：「if crc32_castagnoli(&bytes) == location.unit_checksum {」 |
| 这一趟停在续做中途，读错账空，照旧交给真做 | `crates/singlefs-core/src/mount.rs:5365`：「// 这两趟读盘没出过读错：照旧留给续做（岔路单那一行只定了 C841）。」 |
| 真做走到同一步，读的是共用那一份里的坏字节，照样失败 | `crates/singlefs-core/src/mount.rs:5302`：「只进内存那一趟与真做读同一份字节、走同一段，它报的错真做走到那一步照样报；」 |

原样（`out/follow-up.log`，前两行 after、后两行 before）：

```
after follow image=unchanged plan=p:23101440:306:1:garbage first mount FloorRaiseFailedAfterTheMountsPublishes disk-other all-before-the-first-write digest_after=0802033225bd3a9a fired_at=[(352, 0, 23101440, 16384), (353, 1, 23101440, 16384)] first_write_at=Some(397)
after follow image=unchanged plan=p:23101440:306:1:garbage remount 2 without fault PlacementRefusedBeforeAcquisitionMountAdmissionUndecided disk_changed_by_it=false
before follow image=unchanged plan=p:23101440:306:1:garbage first mount Ok(steps=16) disk-as-fault-free all-before-the-first-write digest_after=826326e3fd072b73 fired_at=[(961, 0, 23101440, 16384), (962, 1, 23101440, 16384)] first_write_at=Some(1037)
before follow image=unchanged plan=p:23101440:962:2:garbage first mount FloorRaiseFailedAfterTheMountsPublishes disk-other both-sides-of-the-first-write digest_after=0802033225bd3a9a fired_at=[(966, 0, 23101440, 16384), (967, 1, 23101440, 16384), (2405, 0, 23101440, 16384), (2406, 1, 23101440, 16384)] first_write_at=Some(1042)
```

after 那一版之后不带故障再挂 5 次：第 1 次做成（续做 0 步）、第 2–5 次都在取号之前拒，盘不再变。before 那一版同一个故障做成、盘与无故障相同、之后 5 次都做成。before 那一版要走到同一张盘面（摘要都是 `0802033225bd3a9a`），得让真做自己那一读也读坏（最后一行，故障落在第一个写两边）。也就是说：W13 之前只有真做读坏才到这一步，W13 之后取号之前预演读坏一次就到了。

两版对照扫描（`out/pair-summary.log`，计划 `p:<偏移>:306:1:<故障>`，偏移取 after 那一版读迹第 306 行之后读到的每一个：没改坏 61 个、改坏 31 个）原样：

```
after pair image=unchanged kind=Garbage after=306 first=1 summary 7 × FloorRaiseFailedAfterTheMountsPublishes disk-other all-before-the-first-write fired=2
after pair image=unchanged kind=Fail after=306 first=1 summary 7 × ReadFailedRefusalBeforeAcquisition disk-unchanged all-before-the-first-write(no-write) fired=2
after pair image=corrupted kind=Fail after=306 first=1 summary 7 × ReadFailedRefusalBeforeAcquisition disk-unchanged all-before-the-first-write(no-write) fired=2
after pair image=corrupted kind=Garbage after=306 first=1 summary 7 × FloorRaiseFailedAfterTheMountsPublishes disk-other all-before-the-first-write fired=2
before pair image=unchanged kind=Garbage after=306 first=1 summary 36 × Ok(steps=16) disk-as-fault-free all-before-the-first-write fired=2
before pair image=corrupted kind=Fail after=306 first=1 summary 7 × C841-after-writes(persisted=19) disk-other all-before-the-first-write fired=2
before pair image=corrupted kind=Garbage after=306 first=1 summary 7 × C841-after-writes(persisted=19) disk-other all-before-the-first-write fired=2
```

（每组其余的行都是 `Ok(steps=16) disk-as-fault-free` 或 `C841-before-acquisition disk-unchanged`，全文在那份文件里。）读成一张表：

| 一份树表两份头一次读 | 没改坏，W13 之前 | 没改坏，W13 之后 | 改坏，W13 之前 | 改坏，W13 之后 |
|---|---|---|---|---|
| 读错 | 7/7 做成 | 7/7 取号之前拒 | 7/7 C841 落盘之后 | 7/7 取号之前拒 |
| 读回坏字节 | 7/7 做成 | **7/7 落盘之后失败** | 7/7 C841 落盘之后 | **7/7 落盘之后失败** |

W13 把读错那一行修了；坏字节那一行在改坏那份上换了一个落盘之后的错，在没改坏那份上从「做成」变成「落盘之后失败、之后挂不上」。

四句：不分辨 ⓑ 的两种读法（「被兜住的也算」与否）——坏字节根本不进读错账。系统看得到——预演里读这份树表的读者核过两份都对不上，知道这是读坏；同一个文件把「读坏」定义成两种都算：`crates/singlefs-core/src/mounted_session.rs:590`：「/// 共用的那一段里读坏（读不出或自证不过）之后当场重读的那一读」。字面上不落在 L2 验收的哪一句（拒没有发生）；L1 验收要的「不同」也不成立（预演与真做同一份字节、同一个结局）。按跑前条款第三行交用户：用户定的「读一次两边共用」让取号之前一次读坏的字节罩住整次挂载，之后真做照着它落盘再失败；岔路单那一行的翻面观测只写了「取号之后有写改了它」，没写这一形。

### 调用方按 `MountError::Publish` 认的东西

只读码查了 harness 库里按这个成员分路的几处（`model_comparison.rs`、`history.rs`、`mount_transparency.rs`）：都按 `cause` 认名字或理由，新成员在 `model_comparison.rs` 里归「说不出理由」；写账全空的形态与第二轮 J4 查过的那一拒相同。没有逐个调用方造历史。

## L3：读坏当场重读

| 问 | 结果 | 依据 |
|---|---|---|
| 重读之后换掉的那一份会不会让预演与真做看到不同字节 | 这一轮的取样里没造出来：坏字节窗口落在根环槽、释放之前读盘核读的单元上的，没有一个起点让结局与无故障不同（见下一行）。打中的两形（L1 ①、L2 坏字节那一形）都不经当场重读 | `out/window-summary.log`、`out/window-hits.txt` |
| 坏字节第一次被记下、重读又失败时后面那几读看到的是什么 | 共用那一段里谁再读都取记下的坏字节（`read_by_the_route` 只在三种路里分，`SharedBetweenTheRehearsalAndTheRealRun` 那一路先查记下的那一份）；只有当场重读那几处与照旧读盘的三段穿到盘上。后面的读都核自证、都判得出它坏，没有「拿到坏字节而没被判出」的；问题在于没有当场重读的读（L2 那一份树表、L1 ② 那一槽）整次挂载都看坏字节，W13 之前真做那一读本身就是又一次读盘 | `crates/singlefs-core/src/mounted_session.rs:716`：「ReadRouteOfThisMount::StraightToTheDisk => return self.disk.read_at(offset, buffer),」；L1、L2 两节的原样输出 |
| W16 没改的两处是不是真的不用改 | 没打中：根环槽（`dev=0` 的 512 字节读，读迹第 295–302 行）单次读回坏字节，挂载做成、盘面与无故障逐字节相同（宽 1 的坏字节窗口没改坏那份 411 个里只有 3 个盘面不同：起点 27 在读阶段、读错同样不同，起点 303、305 是系统配置槽，归 L1 ②）。E2 进程内重读那一处这一张盘上走不到（没有挂载那一刻读坏的槽），没造 | `out/window-summary.log`：「window image=unchanged kind=Garbage summary 3 × width=1 Ok(steps=16) disk-other all-before-the-first-write fired=1」 |
| 另有一处不在 W16 那张表上、却同样「第一次读坏、整次挂载都坏」 | 写系统配置槽时算世代号的那一读（L1 ②）与抬 F 算上限读有效根树表的那一读（L2）都没有当场重读；W16 的表是按「重读一次」这几个字 grep 出来的，没有重读的读不在上面 | W16 报告「改法」一节那张表 |
| 带读故障注入的设备上默认实现走不走故障注入那一层 | 走：harness 库与 checker 档的源码里没有一个 `BlockDevice` 覆写 `reread_at_after_a_bad_read` / `keep_the_self_verified_reread`（`grep -n` 两个名字在 `crates/singlefs-harness/src/`、`crates/singlefs-checker-tier/src/` 命中 0 次），默认实现调 `self.read_at`，就是故障层自己那一读。原型的 `ProbeDisk` 同样不覆写，它数到的读里含当场重读 | 读码 |

## L5：比 fsid

| 问 | 结果 |
|---|---|
| 本池 fsid 取自哪一份系统配置、那一刻择定没有 | 取 `rebuilt_allocator` 收到的那一份，挂载把它交进去之前已经择定（`crates/singlefs-core/src/mount.rs:7797`：「let system_configuration = choose_system_configuration(&*devices)?;」，之后调 `rebuilt_allocator` 传的就是它）；单元写者用的是挂载参数里的 fsid，可写挂载先核过它与盘上的相同：`crates/singlefs-core/src/mount.rs:6112`：「caller_filesystem_identifier == disk_filesystem_identifier,」。没找到合法写路径写出别的 fsid 的单元头，误拒本池的没打中（读码；格式化、换盘、扩盘、回退这几种操作没有逐个造镜像） |
| 别池的单元头放不放过 | **放过一类**：单元头只记 fsid 的低 8 字节：`crates/singlefs-core/src/unit.rs:45`：「u64::from_le_bytes(filesystem_identifier[..8].try_into().expect("切了 8 字节"))」；W14 那一判只比这 8 字节：`crates/singlefs-core/src/mount.rs:2245`：「(filesystem_identifier_in_the_header == filesystem_identifier_in_unit_headers)」。两个池 fsid 的低 8 字节相同，单元头逐字节分不出 |

仓里自己的「另一个池」就撞上这一形：`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:63`：「const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-rev-a2b";」，同文件 `crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:65`：「const ANOTHER_FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-earlier";」（注释说「更早一次 mkfs 留下的槽用它」）。原型把它们交给 `unit_filesystem_identifier`（`out/fsid-fixtures.log` 原样）：

```
fsid core_review_tree_table_duplicates_and_slot_one_search FILESYSTEM_IDENTIFIER: full=[73, 69, 6e, 67, 6c, 65, 66, 73, 2d, 72, 65, 76, 2d, 61, 32, 62] in_unit_headers=7366656c676e6973
fsid core_review_tree_table_duplicates_and_slot_one_search ANOTHER_FILESYSTEM_IDENTIFIER: full=[73, 69, 6e, 67, 6c, 65, 66, 73, 2d, 65, 61, 72, 6c, 69, 65, 72] in_unit_headers=7366656c676e6973
fsid E142 pool, last 8 bytes changed: full=[5f, 53, 46, 53, 2d, 45, 31, 34, 2d, 61, 6e, 6f, 74, 68, 65, 72] in_unit_headers=3431452d5346535f
```

冻结副本 `crates/` 下 16 字节的 `*b"…"` 字面量（`grep -rhoE '\*b"[^"]{16}"' --include=*.rs . | sort -u`，大多是测试池的 fsid）去重后 21 个，18 个以 `singlefs` 开头；当 fsid 用时记进单元头全是同一个数。mkfs 不生成 fsid：`crates/singlefs-core/src/make_filesystem.rs:531`：「而 fsid 是调用方给的（`MakeFilesystemParameters::filesystem_identifier`，mkfs 自己不生成）⇒」，同一处注释说的那一形（同一个 fsid 在用过的盘上重做 mkfs）在单元区里留下的旧池单元头与本池逐字节相同。W14 的用例把「别的池」造成「本池那 8 字节逐位取反」：`crates/singlefs-harness/tests/newer_unreadable_root_counts_only_units_of_this_pool_as_overwriting_its_pointer_slots.rs:69`：「/// 别的池的：本池那 8 字节逐位取反，与本池的必不相同。」——低 8 字节相同的那一类没有用例。

镜像：低 8 字节相同的别池单元摆进 R 的四个指针槽，单元头与 W14 阳性对照那一份（fsid 不动、放过、做成）在被比的那 8 字节上相同，那一判照阳性对照放过（推的；没有另造一张别池镜像跑，因为被比的只有这 8 字节与实例代号）。

四句：不分辨（任何只比单元头的判法都中）；**系统看不到**判别它的东西（头里没有高 8 字节）——「判别子观测不到」那一形，要改判据（例：单元头加宽、或 mkfs 生成 fsid 并保证低 8 字节在一组盘上不撞），改完是新一轮；满足 L5 验收后半「一份别池单元头仍被放过的镜像」；条款（用户「比 fsid」）没说低 8 字节相同怎么办，按跑前条款第三行交用户。

## L4、L6（没攻，只读码）

- L4：`crates/singlefs-core/src/recovery.rs` 里新加的那一判罩新根段四条指针（树表、映射树根、实例表、分配记录树根），排在 `record.to_bytes()` 之前；没造镜像。
- L6：deleted_inodes 写者（`deleted_inodes_root_entries`）每条只写 `key.0.to_le_bytes()` 8 字节，读者 `!= 8` 不会误拒它写出的根。`data_unit_payload` 新加的越界判与 `parse_data_unit` 用同一个界（`payload_start + 声明长度 > bytes.len()`）。注释与代码对不上的一处：`crates/singlefs-core/src/mounted_session.rs:590` 把「读坏」写成「读不出或自证不过」，`crates/singlefs-core/src/mount.rs:5300` 那一拒只认读错（见 L2）——这是 L2 那一形在文档上的样子，不另算一格。

## 改法（只在我的模型上想过、一个都没实现，被攻过零轮）

| 改法 | L1 ①（系统配置槽坏字节，改坏那份 C841 落盘之后） | L1 ②（取号写覆写最新那一槽） | L2 坏字节那一形（落盘之后失败、之后挂不上） | ⓑ 偏拒（没改坏那份 7/411） |
|---|---|---|---|---|
| 甲：读错账之外再记「读坏」：共用那一段里核自证、核校验和不过的读，读的一方报一声（与 W16 的 `note_that_the_bytes_read_self_verify` 对称），ⓑ 照两本账判 | 中（推的：预演停下、账里有一条，取号之前拒） | 不中（推的：一槽坏时预演取号照样成、走到底，不进 ⓑ 那一判） | 中（推的：两张盘都在取号之前拒） | 不变，没改坏那份另多拒坏字节那 7 个 |
| 乙：自证不过的字节不留在共用那一份里（读的一方报一声，那一层丢掉那几块，下一次读穿到盘上） | 中（推的：预演取号再读一次盘、读到好的，走到 C841、取号之前拒） | 中（推的：算世代号那一读重新读盘） | 没改坏那份中（推的：真做重新读盘）；改坏那份不中（推的：预演停下、账空，交给真做，真做读到好的、走到 C841 落盘之后，回到第二轮 J2 那一形）——要与甲一起 | 不变 |
| 丙：只进内存那一趟的取号读也穿到盘上，与真做对称 | 中（推的） | 不中（推的：写那一半的世代号读照旧共用） | 不中（碰不到那一读） | 不变 |
| 丁：写系统配置槽算世代号的那一读穿到盘上（算进「取号写之前那几次读」，轮换写也照这样） | 不中（推的） | 中（推的） | 不中 | 不变 |
| 戊：预演没走到结局就拒（岔路单「没走完就拒」那一候选，用户没选） | 中（推的） | 不中 | 中（推的） | 变多（推的：凡是停下都拒） |

每一格都是「推的」（按代码推、没实现没跑），没有一格是量过的。能同时罩住四格的组合是甲 + 乙（再加丁罩 L1 ② 的轮换写）；戊与 ⓑ 偏拒那一格方向相反。

## 没打中的形状

| 形状 | 取样范围 | 结局 |
|---|---|---|
| 整池第 n 次读起连着 w 次读错 / 读回坏字节 | 没改坏那份 411 个起点、改坏那份 360 个起点，w = 1、2、3，两种故障，全起点不抽样：4626 次挂载（`awk` 把 `out/window-summary.log` 第 5 栏加起来是 4626） | 打中的只有上面三形；改坏那份没有一个读错窗口落到 C841 落盘之后（第二轮 J2 读错那一形在今天的代码上没了） |
| 一个单元两份在第 306 次读之后的头一次读都坏 | 没改坏 61 个偏移、改坏 31 个偏移，两种故障，两版二进制 | 同上表 |
| 无故障时写出的字节 | 两张镜像，两版 | 相同 |
| W16 没改的两处 | 根环槽单次坏字节（读迹第 295–302 行），宽 1 全起点 | 盘面与无故障相同；E2 那一处这张盘走不到 |
| 别池单元头误拒本池 | 读码：单元写者的 fsid 来源与挂载核参数那一处 | 没找到 |
| 写报成功而没落盘（ⓒ） | 没造（`InjectedFault::WriteIsSwallowed` 在 harness 里有，没接进原型） | — |

## 这条腿自己的限度

- 只有一种几何（窄盘 304 槽、16 个单元、一批一个单元）、两张镜像；坏字节只试了「第 0 字节翻最低位」一种，多块的读只坏第一块。
- 窗口按整池读序数点。故障落在取号之前的那几形，挂载里由用户决定的动作只有挂载本身；之后「挂不上」那一句只放开扫了「不带故障再挂」这一种用户动作（5 次），没试只读挂载、手动抬 F、回退这几种之后能不能救回来——那一句只说明「再挂可写挂不上」。
- before 那一版只把七份被判文件撤回 W13 之前，另撤了 harness 库里两个分支；别的包是冻结副本的样子。两版读迹前 306 次逐行相同是比出来的；两版对照扫描假定「某偏移第 306 次读之后的头一次读」在两版是同一个逻辑读，只对 23101440 那一份按序数核过（after 第 352 次、before 第 961 次，都是只进内存那一趟抬 F 算上限那一读）。
- observed 那一份的打印走 stderr，与原型的结果行按行交错，靠行缓冲认先后。
- 共用那一份的大小只量了上界（底下那一层看到的不同扇区数），没从那一层里面数。
- 崩溃状态一个都没枚举（累计 0 个，约 10⁶ / 10⁷ 的额度都没用）：L1 ② 之后取号写崩在中途的后果是推的。

## 跑法与批（照定义 3b）

不带故障先跑一段：debug 二进制改坏那份 40 个起点 25.8 秒（每次挂载约 0.64 秒），全量 4626 次挂载单线程约 49 分钟，超过 40 分钟，所以按批并行、不缩取样。批一：改坏那份 24 件 × 90 个起点（debug，本机此刻还有别的会话的 crash / 变异在跑，负载约 56），十来分钟；之后换 release，批二：没改坏那份 24 件 × 103 个起点，四分钟以内。正式数取自 release 的批三（改坏，24 件 × 90）、批四（没改坏，24 件 × 103），各四分钟以内；批三与批一、批四与批二的汇总逐行相同（`diff` 过）。批五、批六：两版对照扫描各 4 件。每批都按 `tools/run-batch.sh` 收退出码：起之前删本批 `.rc`、一个不带参数的 `wait`、数 `.rc` 与件数、按件号读，六批 `.rc` 全是 0。没给批设限时。线程：每件单线程，`capped.sh 31` 起，至多 24 件同时跑。

## 要主 agent 跑的命令

1. 复跑（长活，单线程顺序跑估 40 分钟以上，要放后台）：工作目录随意，`bash scripts/hosts/run-with-memory-cap.sh 14G bash research/prompts/m3-b44-code-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-b44-code-r3/repo --work <新的草稿目录>`；它里面的 cargo 与原型经 `capped.sh default` 起。跑完看 `<草稿目录>/out/` 下 `fault-free-trace.log`、`follow-up.log`、`pair-summary.log`、`window-summary.log`、`sysconf.log`、`sysconf-writes.log`、`fsid-fixtures.log` 与模型目录 `out/` 里同名文件逐行比（`window-summary.log` 我这一份是 24 件分片汇总出来的，行的写法与它一件跑全段打出的相同，行序可能不同，按排序后比）。
2. L1 ② 崩溃那一半要在 checker 档里造：录 `w:303:1:garbage` 那一次挂载的写流，照崩溃点重放的枚举域枚举取号写那一段，每个崩溃状态恢复之后跑池级 checker、再挂一次。原型做不了（要依赖 checker 档包），没写成命令。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；副本上的数不算入库装置上的数。
- L4、L6 没造镜像；ⓒ 没造；只一种几何。
- 没跑门禁、checker 档测试、重型测试，没起 GPU，没连第二台。
- 冻结副本一个字节没改；主工作区只写了这份报告与模型目录。

## 草稿与副本

草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-b44-code-r3-opus/`：交回之前删掉三份仓副本（`tree/`、`tree-observed/`、`tree-before-w13/`）与三个编译目录（`target/`、`target-observed/`、`target-before-w13/`）；留下 `out/`、`final/`（原始日志，模型目录里的文件由它们汇总）、`model/`（与入库的模型目录逐字节相同）与几份小脚本。
