# m3-close-sysconf-r1 攻方腿（Opus）报告（2026-10-19）

立场：找反例。代码只读树副本 `/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree`，引代码写那份副本的绝对路径与行号；kb 行号是 kb 文件自己的。副本上量出的数都是攻方副本上的数，不是入库装置上的数。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-close-sysconf-r1-opus-model/`。复跑（整条经内存包装，峰值实测 78 MiB）：

```
bash scripts/hosts/run-with-memory-cap.sh 700M bash research/prompts/m3-close-sysconf-r1-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs \
  --frozen /home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree \
  --work <新的草稿目录>
```

它把冻结副本拷进草稿目录、补上仓根的 `Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`，用 `tools/patch_behind_arms.py` 给 `mount.rs` 的落后判法加三条臂（环境变量 `SINGLEFS_PROBE_BEHIND`：`tail` 今天的判法；`record_both` 改法丁照字面、两半都按「这块盘上有没有那一版末条记录」判落后；`record_units` 改法丁只换缺单元那一半），拷进探针测试 `opus_sysconf_r1_probe.rs`，每条臂跑一遍探针与现有用例文件 `entries_after_mount_refuse_swapped_or_behind_devices`（跳过那一条扫描），末尾逐份与 `out/` 比、打 `same` / `differs`。

```
2f76ac1bc847b7e17ec8d7d38310d4b2c4a7774fab295b3f9d6ce16b0d0c5573  ./opus_sysconf_r1_probe.rs
1d023639fcffb537d97803753d3417b9b91679ffabba85b1f3fde95cb6b1b119  ./out/c585-tests-record_both.txt
725d4231b461ac8f811ef5d434ac5257211e0b69a7052b5b08183d00a0ff6101  ./out/c585-tests-record_units.txt
725d4231b461ac8f811ef5d434ac5257211e0b69a7052b5b08183d00a0ff6101  ./out/c585-tests-tail.txt
bee2c2e28495477c1a456e72a783a39ad720f5dca21f974206099a4202cbf69c  ./out/probe-lines-record_both.txt
5fe5e5ca7e8c5bc860255651623c9ca0b367a3dc3165ae69cecbce2197ed3f43  ./out/probe-lines-record_units.txt
25d99b3c55bf3529801c85a6d537dd63220f60d611451c7a7debe7e76fd1ad59  ./out/probe-lines-tail.txt
c0af2d88eb360e8f91c3780a15e235e8c556e022f689d1c0d7890f0b424c6dc6  ./tools/patch_behind_arms.py
07b631cfb18e9d74879af2fe0716f4c5f5d4f69d858a71f1d77b515bbd257bf0  ./tools/rerun.sh
```

（`sha256sum` 在模型目录里跑；`rerun.sh` 本身在交回之前没有整条复跑过一次，`out/` 里的是我在草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-opus/repo` 上逐臂手跑的输出，见「这条腿自己的限度」。）

探针五条（都只打印、不断言臂的好坏）：
- `probe_behind_and_one_unit_copy_bad_then_writable_mount`：C593 形。起点是现有场景 `root_device_of_the_publish_taken_mid_publish(AfterItsRootSlotWrite, txg 6)`：会话覆盖写到 txg 5，再发 P（txg 6），P 的根住的盘 0 换成切在 P 的根槽 FUA 之后的样子（根已落、轮换没写，等同一次崩溃）；再把盘 0 上 P 的第一个数据单元翻坏一个字节（单故障），连着两次可写挂载。
- `probe_controls_for_the_behind_and_one_unit_copy_bad_shape`：两份对照：上面那块盘面挂载之前的 checker；两块盘都做完 P（都轮换过）、同一单元同样翻坏，可写挂载前后的 checker。
- `probe_mid_publish_snapshot_through_a_session_publish`：C585 形。同一场景切在 P 的根槽 FUA 之前，经会话发布，之后 checker；再可写挂载一次。
- `probe_acquisition_cut_after_units_then_both_newest_slots_bad`：C555 形。会话覆盖写到 txg 5 的盘面（不卸载）上录一次可写挂载（取到 2），切在第一条 journal 记录写之前（单元写了、记录没写），三条故障臂：不加；两块盘带实例 2 的那一槽各翻一个字节；那一槽读错。各自再可写挂载，打印取到的号与 checker。
- `probe_acquisition_formula_when_the_undecidable_surface_is_let_through`：同一切点，读错那一臂上 `next_instance_generation_to_acquire` 算出的号；再模拟「管理员工具拿同盘另一槽重写读错那一槽」之后可写挂载。

## 各格判定一览

| 格 | 候选 | 判定 | 量过 / 推的 |
|---|---|---|---|
| S1 | 管理员工具离线重写 | 打中：照「拿同盘还自证过的那一槽重写读错那一槽」做，C555 那段历史上下一次可写挂载又取到 2（撞号），池级 checker 0 红、看不见 | 量过（副本） |
| S1 | 只读挂载下定点修复 | 越位：与「只读挂载不写盘」的条款相撞；重写的内容同上一格，同样撞号 | 推的 |
| S1 | 可写挂载附加条件放行 | 打中：任何放行 C555 读错形的条件，取号式子都算出 2；单故障下站得住的条件（只有一块盘报读错）判别不了「另一块盘的坏槽是撕裂还是损坏」，把 R2 收掉的混合形丢写重新放出来 | 撞号量过；混合形推的 |
| S1 | 接受永远只读 | 打中定义：一槽持续读错（单故障）加一次普通断电撕裂另一盘的轮换，就永远只读；与 D22 已定项 8 射程把「每盘一槽系统配置坏」写在承诺里相撞 | 拒那一半有现成用例；「永远」推的 |
| S2 | 改法丁照字面（两半都换） | 打中：重开 C585；现有用例文件 41 条里 17 条由绿转红，会话发布照常做成之后池级 checker 红 I-3.1 | 量过（副本） |
| S2 | 改法丁只换缺单元那一半 | 没打中：41 条全绿；C593 形挂得上，挂上之后的 checker 与「系统配置跟得上、只缺一份单元」那份对照逐条相同 | 量过（副本） |
| S2 | 维持拒 | 打中定义：崩溃加一份单元坏（单故障）之后两次可写挂载都拒 | 量过（副本） |
| S3 | fsid 不符算读错 | 打中：一次错向写（单故障）盖掉盘 0 一槽，加一次普通断电撕裂盘 1 的轮换，就永远只读；今天的判法在这段历史上挂得上、不丢写 | 推的 |
| S3 | fsid 不符算自证不过（今天） | 单故障形没打中；内部矛盾：R2 按「设备报不报错」分边，不按「内容丢没丢」分，同一种丢失（最新槽内容没了）翻坏走一边、读错走另一边，结局一个撞号一个拒 | 量过（副本，C555 两臂） |
| S4 | 恢复里自愈 | 越位：恢复对只读挂载也跑 | 推的 |
| S4 | 取号时自愈（与取号重写同一个写） | 打中：两槽全废的盘可能是换进来的别池或空白盘，取号写先把它写成本池的「可见」盘，「可见」与落后两道核从此认它，等于洗白；写失败时每次可写挂载都拒 | 推的 |
| S5 | 照 C583 写「不罩」 | 内部矛盾：D22 已定项 8 射程字面把「每盘一槽系统配置坏」写在承诺里，C583 只把它加上「最新根一时读不出」才排除；C555 没有根读不出，按字面在承诺之内 | 引文核过 |
| S5 | 取号式子并进 journal 环 | 打中：只写了单元就崩的切点上环里没有实例 2 的记录，并进环也取 2；池级 checker 在这一形上 0 红，当不了这一格的判据 | 切点与 checker 量过；并进环那一句推的 |
| S5 | 第三种：取号把新号写进每块盘两个槽 | 我提的，只在推理上、被攻过零轮；它只罩取号到第一次轮换之间那一段窗口 | 推的 |
| S6 | 收（按计数器与实例表认第一条） | 判别子观测不到：实例表行里没有「起始计数器」这一列，崩溃那一刻也还没有本实例的行 | 推的 |
| S6 | 不收 | 可达（推的）：要一次多记录发布、挂载扫环时残留那一槽一次瞬时读错、发布途中崩溃三样；之后 checker 在每次运行里持续红 I-8.6，直到环转一圈；恢复在同一份镜像上照常施加 | 推的 |

最脆弱的一条：S5 写「不罩」与 S1 永远只读都踩在同一处，即 R2 按设备报不报错分边。C555 那段历史量过：同一个「两块盘最新槽内容没了」，翻坏走判得出那一边（撞号、checker 0 红），读错走判不出那一边（永远只读）。S1 的任何出路只要放行读错那一形，就落回翻坏那一形的撞号（量过：取号式子算出 2）。S1 与 S5 不能分开定。

## S1　C596：R2 之后仍判不出的盘面怎么走出只读

今天的判法：「pub fn system_configuration_witness_is_undecidable(」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/recovery.rs:4306`），读错只数设备交空的那一种：「None => slots_reporting_a_read_error += 1,」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/recovery.rs:4267`）。取号式子：「let highest_on_disk = highest_system_configuration_instance(pool).max(highest_root);」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs:941`），只读系统配置槽与根环。

### 打中一：任何出路落在 C555 的读错形上都撞号（量过，副本）

盘面与历史（四步，用户动作只有「挂载」，没有可放开的选择）：
1. 会话覆盖写到 txg 5、不卸载（现有场景 `pool_with_a_snapshot_after_the_first_file`）。
2. 可写挂载，取到 2（`probe=c555_shape first_mount_instance=2`），录下写流，切在第一条 journal 记录写之前：「probe=c555_shape cut_at_operation=26 of=62 unit_writes_before_cut=18」（`research/prompts/m3-close-sysconf-r1-opus-model/out/probe-lines-tail.txt:1`）。这时实例 2 只留在每块盘最新那一槽系统配置与 18 次单元写里。
3. 两块盘带实例 2 的那一槽各一次持续读错。
4. 再可写挂载。

结局（`probe-lines-tail.txt` 原样）：
- 今天：「probe=c555_shape fault=read_error second_mount=refused NewerStateStillUnreadableAfterOneReread」（`research/prompts/m3-close-sysconf-r1-opus-model/out/probe-lines-tail.txt:4`），即 C596 的盘面，之后一直如此。
- 出路放行之后取号会取到的号：「probe=s1_exit_on_c555_read_error formula_next_instance=Ok(2)」（`research/prompts/m3-close-sysconf-r1-opus-model/out/probe-lines-tail.txt:14`）。
- 管理员工具拿同盘另一槽（还自证过的那一槽）重写读错那一槽，再可写挂载：「probe=s1_exit_admin_rewrite_from_sibling second_mount=mounted instance=2 checker_red=[]」（`research/prompts/m3-close-sysconf-r1-opus-model/out/probe-lines-tail.txt:13`）。

实例 2 第二次被取，第一次那 18 次单元写带的写序实例代号也是 2。条款说作废一个 checkpoint 必须换实例代号：「**作废一个 checkpoint 必须换实例代号**」（`.claude/kb/decisions/18-块里携带什么信息.md:318`），重发同一个号，孤儿单元按「i == i_now ⇒ b ≤ 挂载根 txg」判成已发布。池级 checker 在这一形上 0 红（同一行 `checker_red=[]`），它不判没被根引用的孤儿单元，所以当不了这一格的判据（S5 节同一处）。

四句：
- 分不分辨臂：分。管理员工具、只读挂载修复、附加条件放行三条，只要重写或放行之后取号仍按今天的式子，都中；「接受永远只读」不中这一格（它中的是下一格）。
- 系统当时看不看得到判别它的东西：看不到。读错那一槽里本来有实例 2，读不出；根环里没有实例 2 的根；journal 环里没有实例 2 的记录（切在第一条记录之前）。唯一的痕迹是单元头里的写序，取号式子不读单元区。
- 满足的是判据的哪一句：派发提示列的第二种「让两次挂载取到同一个实例号」。
- 改法在这几格上还中不中：S5 的「并进 journal 环」照样中（环里没有）；S5 的第三种（取号把号写进每块盘两槽）不中，因为每块盘另一槽也带 2，读错一槽还看得到（推的，没实现）。

### 打中二：附加条件放行，条件写不全（判别子观测不到，推的）

单故障下站得住的放行条件是「只有一块盘有槽报读错」：另一块盘那一槽要么是这次崩溃撕裂的（在飞的写，不承载确认过的发布），要么是第二个故障。轮换下一次写的槽恒是另一槽（世代号 mod 2），所以撕裂槽恒落在「较新」那个下标上，另一槽是上一次做完的轮换，见证到最后一次确认过的发布。这一推导只在单故障预算下成立。
同一块盘面（一盘一槽报读错、另一盘一槽读得出而自证不过），系统分不出「另一盘那一槽是崩溃撕裂」与「另一盘最新槽被损坏、同时最新根一时读不出」；后一形就是 R2 收掉的混合形。D22 记的界：「丢写格都落在每盘各坏一槽系统配置、同时最新根一时读不出的双故障形里」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）。所以任何只看系统配置槽的放行条件，要么仍拒这一形（出不了只读），要么把 R2 关掉的那几格重新放出来。要分开它们得另读根环与环里的记录：放行条件写成「根环每个槽都读得出（没有设备报读错）」，根环里看不到的更新发布才可能被漏，推的，没量。

### 打中三：「接受永远只读」与单故障承诺相撞

盘面：盘 0 一槽持续读错（单故障），盘 1 最后一次轮换被普通断电撕裂（不算故障）。现有用例 `a_writable_mount_is_refused_when_one_device_reports_a_read_error_and_the_other_reads_back_zeros`（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-harness/tests/system_configuration_witness_is_undecidable_only_when_a_slot_reports_a_read_error_and_no_device_verified_both.rs:139`）钉的就是它拒；只读挂载不写盘，撕裂那一槽只有可写挂载的取号写会重写，于是永远只读。射程句：「容错承诺是单故障：每盘一槽系统配置坏在承诺里」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）。写成已知代价，就要同时改这句射程，否则「在承诺里」只剩「数据不丢」一层意思、不含「还能写」，条款里没有这句区分。

### 只读挂载下的定点修复：越位

C596 那一行把「只读挂载不写盘」列成已有条款：「「只读挂载不写盘」的条款」（`.claude/kb/checks-owed.md:388`）。只读挂载下修，就是推翻它，要另走一轮；修的内容与管理员工具相同，打中一照中。

### 没打中的形状（S1）

- 管理员工具重写时把每块盘两槽都写成「max(全部自证过的槽, 根环, 单元区扫出的写序实例) + 1 之前的号」：读单元区之后打中一不中（推的）；代价是整盘扫单元头，没量。
- 只看撕裂（全零、翻坏）不看读错的放行：今天就放行，不在这一格。

## S2　C593：落后且缺一份单元的盘

今天的判法两半各有一道「落后」：缺单元那一半「if newest_system_configuration_journal_tail >= version_journal_position {」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/mount.rs:5802`），缺根那一半「let is_behind_the_version = newest_system_configuration_journal_tail < version_journal_position;」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/mount.rs:5851`）。改法丁（`yi-j-mount.diff`，c245-r3 那一份，从 git 历史 `dbcc0d2f5` 读）改的是旧代码里合在一起的那一处；今天两半分开了，「落后与否看末条记录」照字面要换两处，只换一处是另一个形态。我把两种移植都实现成臂（`tools/patch_behind_arms.py`），在同一份二进制上用环境变量切。

### 打中：改法丁照字面（两半都换）重开 C585（量过，副本）

盘面与历史：会话覆盖写到 txg 5，再发 P（txg 6，根住盘 0）；盘 0 换成切在 P 的根槽 FUA 之前取下的那一份（P 的单元与记录都在，根没落、轮换没写）；会话再发一次覆盖写。代码注释里这一形本来靠根那一半拒：「旧快照取在一次发布途中（单元与记录已写、根槽 FUA 还没写）、那次发布的根又写在这块盘上的，单元都在，靠根那一半拒」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/mount.rs:5933`）。

- `record_both`：盘 0 带着 P 的末条记录，判成不落后，根那一半不再读根槽，会话发布照常做成，之后池级 checker 红 I-3.1：「probe=c585_shape arm=record_both refused=false checker_red=["I-3.1:盘 0：记账的已分配 Some(1196032)，遍历全部有效根得到 966656」（`research/prompts/m3-close-sysconf-r1-opus-model/out/probe-lines-record_both.txt:6`）。
- `record_units` 与 `tail`：拒，checker 0 红：「probe=c585_shape arm=record_units refused=true checker_red=[]」（`research/prompts/m3-close-sysconf-r1-opus-model/out/probe-lines-record_units.txt:6`）。
- 现有用例文件（跳过那一条扫描，41 条）：「test result: FAILED. 24 passed; 17 failed」（`research/prompts/m3-close-sysconf-r1-opus-model/out/c585-tests-record_both.txt:1`）；`record_units` 与 `tail` 都是「test result: ok. 41 passed; 0 failed」（`research/prompts/m3-close-sysconf-r1-opus-model/out/c585-tests-record_units.txt:1`）。红的 17 条都是根那一半的格：四个入口各四格（「发布途中取下」与「根环转过一圈、根槽里是上一圈的根」两类），加可写挂载「根槽连着两次读错」那一格；名单在 `out/c585-tests-record_both.txt`。

四句：
- 分不分辨臂：分。只中「两半都换」，不中「只换缺单元那一半」。
- 系统看不看得到：看得到。盘 0 的根槽里不是 P，根那一半读得到；照字面的丁不去读。
- 满足哪一句：C585 那一行改法 C 的「或那一版的根写在这块盘上而那一槽读一次重读一次都不是它，才拒」那一半被拿掉；结局是派发提示第三种（恢复与池级 checker 判得不一致：会话照常发布，checker 红 I-3.1）。
- 改法：「只换缺单元那一半」在这 17 格上全绿（量过）。

### 没打中：改法丁只换缺单元那一半（量过，副本）

C593 形（盘 0 切在 P 的根槽 FUA 之后，再翻坏盘 0 上 P 的一个数据单元），两次可写挂载：
- `tail`（今天）：两次都拒，「probe=c593_shape arm=tail round=1 unit=Data(DataUnitIndexInFile(0)) outcome=refused WritableMountRefusedByDevicesWithoutTheSelectedVersion」（`research/prompts/m3-close-sysconf-r1-opus-model/out/probe-lines-tail.txt:11`）。这就是「维持拒」那一臂中的定义：崩溃加单故障之后每次都拒、只读挂载不写盘，永远只读。
- `record_units`：挂上，取 2：「probe=c593_shape arm=record_units round=1 unit=Data(DataUnitIndexInFile(0)) outcome=mounted instance=2」（`research/prompts/m3-close-sysconf-r1-opus-model/out/probe-lines-record_units.txt:11`）。挂上之后 checker 红 I-2.1、I-4.8、I-7.4，三条都指盘 0 槽 50186 那一份坏的单元。
- 对照（两块盘都做完 P、同一单元同样翻坏，今天就放行的「系统配置跟得上、只缺几份单元」）：挂上之后红的是同样三条，「probe=c593_control_caught_up_after_mount arm=tail outcome=mounted instance=2 checker_red=["I-2.1:数据单元 在盘 0 槽 50186 的那一份与位置条目里的校验和对不上"」（`research/prompts/m3-close-sysconf-r1-opus-model/out/probe-lines-tail.txt:9`）；挂载之前的镜像也红这三条（`probe=c593_control_before_mount`），红的是注入的那个坏副本本身。

所以「只换缺单元那一半」把 C593 形带到今天已放行的那条路上，没有新增的红。这三条红在每条臂上都有、与臂无关（打中不分辨臂）：挂上之后最新根（txg 8）仍引用盘 0 上那一份坏的单元，可写挂载不重写它。要不要补写坏副本是共用前提，另记一笔账，不拿它判丁。

### 挂着之后的入口与会话

会话每次改动调的是「devices_behind_the_current_version(」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/mounted_session.rs:1885`），与可写挂载共用 `what_a_device_behind_the_version_lacks`。臂改在共用的那一处，所以四个入口与会话自动跟着改；上面 41 条用例覆盖了四个入口（会话、卸载、抬 F、回退）。「两半都换」在每个入口都红，「只换缺单元那一半」在每个入口都绿（量过）。

### 我还试了、没打中的（S2）

- 有末条记录、却缺 P 的单元的盘：单元写在第一道屏障之前，记录写在第二道之前（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs:1383` 一段），末条记录在盘上就说明 P 的单元已经落过盘；之后再缺只能是损坏（单故障），与对照同一条路。推的，没造。
- 末条记录那一槽一时读错：判成落后，接着读单元；单元都在就放行，缺一个才拒，而且只拒这一次挂载。推的。

## S3　C707：fsid 不符的槽落 R2 哪一边

今天：fsid 不符的槽既不进 `self_verified`、也不进读错计数（只有设备交空才计，`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/recovery.rs:4267`；比 fsid 的是「== *filesystem_identifier」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/recovery.rs:4271`）），也不记进读坏账：只有 magic 或校验和不过的那一支调 `note_that_the_bytes_read_fail_self_verification`（同一个函数里紧跟着的那一支），fsid 不符那一支什么都不做。所以它与撕裂同一边。

### 打中：「算读错」让单故障加一次崩溃永远只读（推的）

盘面与历史：
1. 池照常发布若干次。
2. 一次错向写（D18 已定项 21 自描述的射程就是错向写；例如同一台机器上另一个池的系统配置写落错了块设备）盖掉盘 0 的一槽，读得出、校验和过、fsid 是别的池。单故障。
3. 之后某次发布在轮换途中断电，盘 1 那一槽撕裂。不算故障。

「算读错」时：有槽报读错（盘 0）、没有一块盘两槽都自证过（盘 0 一槽 fsid 不符，盘 1 一槽撕裂）⇒ 判不出，之后每次可写挂载都拒，与 S1 打中三同一个结局。
今天（算自证不过）：判得出，见证值取全部自证过的槽 tail 的最大值。盘 1 撕裂的是在飞的那次轮换，另一槽是上一次做完的轮换，至少见证到最后一次确认过的发布（S1 打中二的那段推导），所以不丢确认过的写；取号写落在每块盘没自证过的那一槽（世代号 mod 2 恒指另一槽），把错向写的那一槽也重写掉。推的，没造镜像量。

四句：分辨（只中「算读错」）；系统看得到 fsid，但看不到「被盖掉的那一槽原来是不是最新的见证」；满足派发提示第四种（单故障之后永远只读）；S1 若定了出路，这一格随之不再永远只读，但出路本身带着 S1 打中一的撞号。

### 内部矛盾：R2 按「设备报不报错」分边，不按「内容丢没丢」分（量过一半）

R2 的理由是「读错那一槽可能藏着更新的见证」。翻坏、全零、fsid 不符三种，被盖掉的那一槽原来的内容同样没了，也可能是更新的见证；R2 却放它们进判得出那一边，只因为设备没报错。C555 那段历史上量到两边结局不同：翻坏「probe=c555_shape fault=flip second_mount=mounted instance=2 checker_red=[]」（`research/prompts/m3-close-sysconf-r1-opus-model/out/probe-lines-tail.txt:2`）撞号；读错（同一行号文件第 4 行）拒。
C707 问 fsid 不符落哪边，R2 自己的理由答不了：按「内容可能丢了」它该算读错，按「设备没报错」它该算自证不过。要答得先选一条原则：
- 原则甲「本池自己的崩溃造不出的坏 ⇒ 判不出」：fsid 不符算判不出（mkfs 已清零 [0, 根环基址)，本池崩溃造不出别池的槽），翻坏与全零仍判得出（撕裂造得出）。这条原则系统看得到（fsid 读得出），但它就是上面打中的那一臂。
- 原则乙「只看设备报不报错」：今天的样子，C555 翻坏形撞号留着。
两条原则都不同时罩住 S3 与 S5，这一格要与 S1、S5 一起定。

### 三种盘面的结局（推的，按今天的代码读）

| 盘面 | 算读错 | 算自证不过（今天） |
|---|---|---|
| 撕裂（全零、翻坏） | 不涉及 fsid，两边同 | 同左 |
| 整盘换成别池的盘（两槽 fsid 都不符） | 这块盘没有自证过的槽，可写挂载在「可见」那一判拒，与 R2 无关 | 同左：取号前「DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness」那一拒（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs:893` 一段） |
| 只换一槽、另一盘两槽都自证过 | 判得出（另一盘两槽都过），照常可写 | 同左 |
| 只换一槽、另一盘只剩一槽自证过 | 判不出，永远只读（上面的打中） | 判得出，取号写把它重写掉 |

### 我还试了、没打中的（S3）

- 「算自证不过」在单故障加崩溃下丢写：找不到。要丢写还得最新根一时读不出，即 C583 的双故障形，D22 已定项 8 写明不罩。
- fsid 不符那一槽不进读坏账（上面那一处）会不会让下一次读拿到缓存里的旧字节：只读阶段与取号各读盘一次，读坏账只影响「下一次读穿到盘上」；fsid 不符的槽下一次读到的还是同样的别池字节，结局不变。推的。

## S4　C463：一块盘两槽全废之后的自愈重写

今天：取号那一刻某块盘一份自证过的都没有就拒（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs:893`），挂着之后的入口在「可见」那一判拒；恢复与只读挂载只报出来。

### 越位：放在恢复里

恢复对只读挂载也跑（只读挂载照样择根、扫环），在恢复里写系统配置槽就违反「只读挂载不写盘」（`.claude/kb/checks-owed.md:388` 那一行列的条款）。只放在可写挂载的恢复里，就是「可写挂载取号时」那一种，不另成一种。

### 打中：放在取号时（与取号重写同一个写），等于洗白一块来历不明的盘（推的）

盘面：盘 1 两槽全废。系统看到的东西在以下两段历史里逐条相同：
- 甲：盘 1 系统配置那一段介质坏（单故障），其余都在。
- 乙：盘 1 被换成一块空白盘，或别池 mkfs 过、又把 [0, 1 MiB) 清掉的盘（运维事故，单一事件）。乙里盘 1 的单元、根、记录全都不在。

今天两段都拒在取号之前。取号时自愈：把盘 1 两槽写成本池的系统配置（tail 写见证值）。写完之后盘 1 「可见」，它的 tail 等于见证值、不落后，落后支两半都不读它的单元与根槽（`what_a_device_behind_the_version_lacks` 先比 tail），于是乙那块空白盘被当成完好的镜像盘：每个单元只剩一份好的，没有任何一道核报出来，直到读到才一份一份地隔离。这是派发提示第三种（之后池级 checker 红 I-2.1 一片，恢复照常挂）。
要分开甲乙，写之前必须核这块盘带不带所选那一版：末条记录在不在（就是改法丁那一读）、单元在不在。核了，甲（介质只坏在系统配置那一段）放行、乙拒；所以 S4 的写之前核什么，答案就是 S2 的判法，两格要一起定。

四句：分辨（只中「写之前不核这块盘带不带那一版」的形态）；系统当时看得到判别子（盘 1 上的记录与单元），前提是去读；满足派发提示第三种；改法「写之前跑一遍丁」在乙上拒、在甲上放行（推的）。

### 写失败怎么报：单故障之后永远只读（推的）

系统配置那一段介质坏得连写也写不进（甲的一种）：取号写在盘 1 报错，按全或无回卷、报失败，这次可写挂载失败；下一次照样。永远只读，与 S1 同一个结局。条款要写的是：写不进去时这块盘按「打开过又掉了」那一类处置（D18 已定项 11「前提」那一段的分法），而不是每次挂载重试同一个写。

### 与 S1 是不是同一个机制

不是同一个前提。S4 的盘面判得出（另一块盘两槽都自证过时），见证值确定，只缺「这块盘是不是本池的那一块」；S1 的盘面判不出，缺的是见证值本身。写的动作可以共用（取号写），写之前核的东西不同：S4 核盘的身份与版本，S1 核见证值有没有被漏掉。另一块盘只剩一槽、这块盘两槽全废，两样同时缺，就同时落在 S1 与 S4（今天先拒在 R2）。

## S5　C555：两盘最新系统配置槽都坏时取号撞号

### 内部矛盾：「照 C583 写不罩」与 D22 已定项 8 的射程字面相撞（引文核过）

射程原句：「容错承诺是单故障：每盘一槽系统配置坏在承诺里；每盘各坏一槽系统配置、同时最新根一时读不出的双故障形不在承诺里」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）。这句把「每盘一槽系统配置坏」算一个故障、写在承诺里；C583 排除的是它再加「最新根一时读不出」。C555 的四步里没有根读不出，只有「两盘最新系统配置槽各翻坏一个字节」，按这句字面就是承诺之内的单故障。
所以「不罩」不是照 C583 的先例，而是收窄已定项 8 的射程，C583 那一行自己写了要罩、要改先改射程：「将来要罩住它，先改那条射程」（`.claude/kb/checks-owed.md:717`），反过来要排除也要先改射程。候选写成「故障模型不收」时，要同时改 178 行那一句，并说清「每盘一槽坏」在 S1（判不出、永远只读那一形）里算不算承诺之内：同一个「每盘一槽坏」，翻坏落 S5（撞号）、读错落 S1（拒），见 S3 的量过的两臂。

### 打中：「max 再并进 journal 环」在只写了单元就崩的切点上照样撞（切点量过，并进环推的）

盘面与历史：S1 打中一那段（会话覆盖写到 txg 5 → 可写挂载取到 2 → 切在第一条 journal 记录写之前 → 两块盘带实例 2 的那一槽各翻一个字节 → 再可写挂载）。
- 不加故障：「probe=c555_shape fault=none second_mount=mounted instance=3 checker_red=[]」（`research/prompts/m3-close-sysconf-r1-opus-model/out/probe-lines-tail.txt:3`）。
- 翻坏：取到 2，`probe-lines-tail.txt` 第 2 行（S3 节引过）。
切点在第一条记录写之前（62 步里的第 26 步，前面有 18 次单元写），环里一条实例 2 的记录都没有，所以「并进 journal 环」在这一格上算出的仍是 2。C555 那一行写着同样的事：「只写了单元就崩的那几格照样撞号」（`.claude/kb/checks-owed.md:362`）。并进单元写序要扫整个单元区的单元头，代价随池长，没量。

### 打中判据：池级 checker 在这一形上 0 红（量过）

翻坏那一臂撞了号，挂载之后池级 checker 一条都不红（同上一行 `checker_red=[]`）。C555 的立账说撞号由「checker 判 I-8.6 红」看出来，那是两段记录都写了的那一形；只写了单元那一形里，第一段实例 2 只留下没被任何根引用的单元，checker 不判它们。所以这一格的「判别力自证」不能只靠池级 checker：要另一个判据，例如按 D18 已定项 11 的已发布谓词判「写序实例等于挂载根实例的单元，诞生代号 ≤ 挂载根 txg ⇒ 已发布」在孤儿单元上判错（推的，没写这条判据）。

### 第三种：取号把新号写进每块盘两个槽（我提的，只在推理上、被攻过零轮）

形态：取号时每块盘先写世代号 g+1 那一槽、过一道屏障、再写 g+2 那一槽（同一份内容，世代号 mod 2 恰好轮到另一槽）、再过一道屏障，之后才动单元。
- 罩住什么：C555 的窗口是「取号之后、第一次发布的轮换之前」，那段时间每块盘只有一槽带新号；第一次轮换之后两槽都带新号（轮换写的是现行实例），本来就不怕一槽坏。两槽都写把窗口关掉：任一块盘任一槽坏，另一槽仍带新号，max 看得到。
- 崩在两次写之间：一槽带新号、一槽带旧号，单元还没写；这时那一槽再坏，重取同一个号，不带任何单元、记录、根，I-7.7（系统配置实例代号不低于根环） 第 ② 句本来就收这一形（推的）。
- 代价：每次可写挂载每块盘多一次 4 KiB 原地写与一道屏障；D22 已定项 16「取号、发布末尾的轮换、取号全或无失败时的回卷三种系统配置写」要加一种，回卷要回卷两槽。
- 我自己能想到的攻法：两次写都在 R2 判得出之后才发；判不出那一形（S1）先拒，所以它不救 S1。它也不救「每盘两槽都坏」（每盘两个故障），那一形落在 S4。

| 改法 | C555 翻坏形，切在第一条记录之前（量过的切点） | C555 读错形（S1 盘面） | 翻坏形，实例 2 的记录也落了 | 标注 |
|---|---|---|---|---|
| 写「不罩」 | 撞号留着（量过） | 永远只读留着（量过） | 撞号留着 | 推的 |
| max 并进 journal 环 | 中（环里没有 2） | 先被 R2 拒 | 不中（环里有 2；C555 那一行记的调查副本 225 格 0 红是这一类） | 切点量过，并进推的 |
| max 并进单元区写序 | 不中 | 先被 R2 拒 | 不中 | 推的，代价没量 |
| 取号写两槽 | 不中 | 先被 R2 拒 | 不中 | 推的，没实现 |

## S6　C569：反向链遇上前一槽的残留记录

### 这一格在今天的写路径上怎么造（推的，没造镜像）

今天新实例的第一条记录编在「环里读得出的最大计数器 + 1」：「let next_counter = records」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/mount.rs:8074`），与条款一致：「前缀末的计数器取环里读得出的最大那一个」（`.claude/kb/decisions/23-journal的角色与格式.md:371`）。所以 D23 已定项 8 说的「恢复之后从「前缀末 + 1」接着写，那正好接在残留序列的前面」（`.claude/kb/decisions/23-journal的角色与格式.md:230`）在今天只对挂载那一刻读不出的残留成立：读得出的残留计数器都比新实例的小，坐不到新实例的槽上。
要让 checker 那一判（「Some(_record_of_another_instance) => ExpectedBackChain::Value(0),」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-checker/src/walk.rs:9058`））碰上新实例的记录，最少要这几步：
1. 实例 A 发一次跨多条记录的发布（计数器 c、c+1，产品路径要点名 65 项以上，测试可用 `JournalRecordNamedEntryCapacity::CappedForTests` 压小），崩在记录那一段里：A:c 没落，A:c+1 只落在盘 1。同一段里的记录之间没有屏障（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs:1383` 那个循环之后才发屏障），这一形可达。
2. 下一次可写挂载扫环时，盘 1 那一槽一次瞬时读错（单故障）：环里读得出的最大计数器是 c−1，实例 B 从 c 起。
3. B 也发一次跨多条记录的发布（c、c+1、c+2），崩在记录那一段里：B:c+1 只落在盘 0，B:c+2 落在两块盘。

镜像上：盘 1 计数器 c+1 那一槽是 A:c+1（残留），c+2 是 B:c+2。checker 逐盘判：盘 1 上 B:c+2 的前一条是别的实例 ⇒ 期望 0，B:c+2 带的是 B:c+1 的链值 ⇒ I-8.6 红。恢复：按 (实例, 计数器) 找 (B, c+1)，在盘 0 上找到、比链对得上（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/recovery.rs:4388` 那个函数按盘次序找第一份解得出、与扫到的那一条相同的镜像），B 那次发布整次施加。两边在同一份镜像上判得不一致：派发提示第三种。

### 打中「不收」：checker 的红在之后每次运行里都在，直到环转一圈（推的）

下一个实例 C 从 max + 1 > c+2 起写，盘 1 的 c+1、c+2 两槽要等环转一圈（默认环 196 608 槽）才被盖掉。「不收」把这一格写成「有意判违例」，就等于承认一个健康池在之后很长时间里每次跑池级 checker 都红 I-8.6；崩溃点重放带读故障的那一类（E165 一族）碰上它也会报红，要另记已知红。

### 打中「收」：判别子观测不到（推的）

候选「按计数器与实例表认本实例第一条」：实例表的行是「`kind` = 0 行记录：`kind 1 | 实例代号 4 | 所选根的 checkpoint_txg 8 | 属于该实例的最大已施加事务号 W 8」（`.claude/kb/decisions/18-块里携带什么信息.md:304`），没有起始计数器这一列；而且行是下一个实例写的（「恢复在所选根指着的那一版表上给 [max(所选根的实例, 1), 新实例) 每个实例写一行」（`.claude/kb/decisions/18-块里携带什么信息.md:312`）），B 崩溃那一刻表里还没有 B 的行。checker 在盘 1 上要分的是「B 从 c+2 起（链应为 0）」与「B 从更小的号起、c+1 那一条在这块盘上没落（链应为 B:c+1 的值）」，实例表给不出。能用的判别子在别处：B:c+2 的「本次发布内序号」不是 1 就不可能是 B 的第一条；或者到另一块盘上找 (B, c+1)。候选要写成「按计数器、发布内序号与两块盘上同实例的记录认」，不是按实例表认。

### 没打中的（S6）

- 环转圈造残留：checker 逐盘按计数器建表（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-checker/src/walk.rs:8975` 那一处 `records.insert(`），上一圈的记录计数器差一整圈，查 c+1 查不到它，判「前一条不在这块盘上」、不判。
- 单记录发布：B:c+1 那次发布做完（屏障、根 FUA）之后才写 B:c+2，B:c+1 两块盘都落了，盖掉了残留。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| S1 | 只撕裂（全零、翻坏）不读错的放行 | 今天就放行，现有用例覆盖，不另跑 |
| S1 | 管理员工具连单元区写序一起取 max | 推的，没实现 |
| S2 | 改法丁只换缺单元那一半：C593 形、C585 形、现有用例文件 41 条 | 一个起点（会话覆盖写到 txg 5，P 是 txg 6，根住盘 0）、一个坏单元（P 的第一个数据单元）、两次连着挂载；用例文件里的格照它自己的取样 |
| S2 | 有末条记录而缺单元、末条那一槽一时读错 | 推的 |
| S3 | 「算自证不过」在单故障加一次崩溃下丢写 | 推的，找不到；要丢写还得根读不出（C583 形） |
| S5 | 取号写两槽的反例 | 推的，只想到「两槽都坏」「判不出先拒」两形，都在别的格 |
| S6 | 环转圈造残留、单记录发布造残留 | 推的 |

放开的用户动作：S1、S5 的历史里用户动作只有「挂载」与「覆盖写」，切点我只取了一个（第一条记录写之前），没有扫别的切点（例如写了一半单元、写了部分记录）；S2 的历史里 P 之后会话只发一次，没扫「之后发 1/2/3 次」（现有用例文件里那一条扫描我跳过了，见下）。

## 这条腿自己的限度

- 只在一个起点上量：会话覆盖写到 txg 5 的两盘 4 GiB 稀疏池。没扫几何、没扫切点，崩溃状态没有走 `enumerate_crash_states`：我的切点都是写流前缀，不是崩溃状态全集。所以「量过」的只是那几个盘面上的结局，不是「这一格上没有别的结局」。
- 用的故障注入是现有脚手架：持续读错用测试目录 `common` 模块的 `UnreadableRange`（`FailingReadsOfARange::Every`），翻坏是直接改盘上字节。
- `entries_after_mount_refuse_swapped_or_behind_devices` 里那一条扫描（`stale_snapshot_substitution_is_refused_by_every_entry_over_the_swept_user_steps`）每条臂都跳过了：本机内存包装的总池当时被别的会话占满（排队），我改用 700M 上限跑，扫描那一条按登记线程数并行、我没估它的内存，没跑。
- 第一次三条臂连着跑的那条命令被移到后台排队，我逐个停掉（`proc.py stop`，先停排队的子进程，循环又起了下一件，再停循环的 shell 与它的子进程），之后改小上限在前台重跑；`out/` 里的是重跑那一次的输出，草稿目录的 `run3-*.log`、`c585b-*.log` 是原件。
- `tools/rerun.sh` 没有整条复跑过；它的步骤与我手跑的逐条相同，差别只在它把内存包装放在最外层。判决复跑时它若打 `differs`，以手跑的原件为准去对。
- S3、S4、S6 的打中都是推的，没造镜像。S6 那一形最值得造：要用 `CappedForTests` 压小每条记录的点名项，造两次跨记录发布、一次挂载时的瞬时读错，再按崩溃状态全集判 checker 与恢复。
- 我提的改法（S5 第三种「取号写两槽」、S6「按发布内序号与两块盘认第一条」、S4「写之前跑一遍丁」）只在推理上，被攻过零轮。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上的数不是入库装置上的数。
- 没读禁读清单里的文件。
- 没跑重型测试，没跑 checker 档包的任何测试，没调崩溃枚举函数。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-opus/` 里留着仓副本 `repo/`（含 `target/`）：判决复跑前主 agent 可以直接在里面按 `SINGLEFS_PROBE_BEHIND` 切臂重跑，省一次编译；不用时整目录删。
