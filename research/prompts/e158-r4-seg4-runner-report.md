# E158 第 4 次跑 第四段 执行员报告（2026-09-27 JST 10:2x）

## 一、结论

- 12 条主族臂的 `r4-seg4` 与跨臂 `r4-compare` 都跑完，13 份产物进 `research/results/`，每份末行 `name=done`，没有一份出 `name=stop`。开跑检查（本地常量 34 × 12 = 408 行、臂探针 12 行、第七节锚点 18 × 12 = 216 行）全部 `pass`。
- **Q4 多拒**（L 族 663 格全是 N_真 为假的合法状态）：甲-配置 / 甲-配置续 / 乙-配置 / 乙-配置续 / 乙-窄读-配置 / 乙-窄读-配置续 各 22；丙-配置 / 丙-配置续 各 16；甲-槽 / 乙-槽 / 乙-窄读-槽 各 19。同一种 N 里甲与乙（含窄读）逐族相等；丙 比甲、乙少的 6 格全在 L5。
- **Q5 同成**：甲-配置 每格 +4、甲-配置续 +8；乙-配置与乙-窄读-配置每格 +3 或 +4（合计 2472，甲-配置 2516，同是 629 格）；-槽 各臂 0 或 −1；丙-配置 −21 / −22、丙-配置续 −17 / −18（每一格都比今天少读）。
- **Q6**：今天那一臂 72 格里有 3 格没罩住（`not_covered`）；另外 11 条臂没罩住的格数都是 0。**甲 三臂的 Q6 按 V1 ④ 作废**：PC-22 里「甲在那个 m 上拒」的 O(m 只) 那一半（`refused_only`，[臂]）在甲 三臂上都是 `fail`。原因见第四节：O(2 只) 只打坏一份的一次读，另一份照样读得出，所以这一格上甲的做法（照常挂载、隔离 28 = 孪生）其实合它的定义；打不中的是登记里这一句的构造。
- **L5 分不出乙与甲**：L5 那 72 格里，乙 三臂没有一格是靠重读挂上的（重读过的 6 格全拒）。第 22 条那一格上乙的重读这一段没量到（第四节末）。
- PC-多拒 36 句全 `pass`；PC-554 12 臂全 `pass`（F1 没触发）；PC-22 今天那一句 `pass`（F2 没触发）；F5（L6 8 格）、F14（每臂 4 次卸载，共 48 行）全 `pass`。
- 什么现象会推翻上面这些：拿同一批臂副本重跑，`r4_q4` / `r4_q5` / `r4_q6` 行变了；或者按 V1 跨段合算（第一、三段 compare 一起）时，某臂的 Q4 / Q5 因为别的段的对照打不中而作废。本段的 compare 只读了第四段的产物，第一段里管 Q4（PC-N0）和 Q5（PC-多读）的对照不在它的输入里。

## 二、怎么跑的

- 负载（01:22 UTC）：有别的会话在跑 `cargo build --offline --all-targets`、一条 `cargo test -p singlefs-harness --test a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable`、另一个仓的 `gate.sh`；没有 qemu、vm-bench、e152、fio。按规定加 `nice -n 19` 照跑，没有等锁。
- 脚本 `/tmp/claude-1000/e158-r4-seg4/run_seg4_v2.sh`：12 条臂逐个在 `arms/<臂>` 下跑，环境变量与命令都照派发提示（`capped.sh 3` 加 `run-with-memory-cap.sh 10G`）；compare 在仓根下跑，用 `arms/today` 那份 bin，参数是相对路径 `research/results/e158-root-choice-repair-2026-09-27-r4-seg4`（这样产物里的 `path=` 字段写的是仓内相对路径）。
- 原样日志（`run_seg4.log`）：

```
arm=today rc=0 seconds=8 lines=816 last=E7RESULT name=done emitted=815
arm=jia-cfg rc=0 seconds=6 lines=808 last=E7RESULT name=done emitted=807
arm=jia-cfg-carry rc=0 seconds=6 lines=808 last=E7RESULT name=done emitted=807
arm=jia-slot rc=0 seconds=5 lines=808 last=E7RESULT name=done emitted=807
arm=yi-cfg rc=0 seconds=6 lines=808 last=E7RESULT name=done emitted=807
arm=yi-cfg-carry rc=0 seconds=6 lines=808 last=E7RESULT name=done emitted=807
arm=yi-slot rc=0 seconds=7 lines=808 last=E7RESULT name=done emitted=807
arm=yi-narrow-cfg rc=0 seconds=6 lines=808 last=E7RESULT name=done emitted=807
arm=yi-narrow-cfg-carry rc=0 seconds=7 lines=808 last=E7RESULT name=done emitted=807
arm=yi-narrow-slot rc=0 seconds=6 lines=808 last=E7RESULT name=done emitted=807
arm=bing-cfg rc=0 seconds=5 lines=808 last=E7RESULT name=done emitted=807
arm=bing-cfg-carry rc=0 seconds=5 lines=808 last=E7RESULT name=done emitted=807
compare rc=0 seconds=1 lines=432 last=E7RESULT name=done emitted=431
```

- 每份产物第一行都是 `E7INPUT name=crates_snapshot key=E158 sha256=bb5ca9bef0b09eb957906317adf7d2b47567d901ef341e85c14b6fe23a37cf4b`。臂副本里的 bin 源码 sha256 `37a8a312…445b`，和主工作区那份一样。
- 装置是确定性的：today 那一臂在草稿目录里重跑一次（`rerun-today.out`），和入库那份 `cmp` 逐字节相同。这只说明没有隐藏状态，不是统计上稳定。
- 产物 sha256（`sha256sum research/results/e158-root-choice-repair-2026-09-27-r4-seg4-*.out`）：

```
27cb69914ce5f7eed76fa3560941dcfe8370e569ba39dad9f5df8b38053e703c  …-r4-seg4-bing-cfg-carry.out
ab77271541a695abb81561daef7f09ff949784a5036f9780ef3f8ac9cede6419  …-r4-seg4-bing-cfg.out
82163aa7a5d8bf7c8b6a901a6f1ae12362ccae8dade50c9cf35d6a0e553c601b  …-r4-seg4-compare.out
6e1681cf9ac644fb73ecf85bc5f146e69128eb1c8aaeabab7e819fedd557c1a0  …-r4-seg4-jia-cfg-carry.out
429743a831c4fd7d049978f512f1ea925fc6806625286ebf318e0d533f0dce84  …-r4-seg4-jia-cfg.out
44468610b8940c20890472a93df19c547d6fc06c8f30bec40b532cb01e62fa39  …-r4-seg4-jia-slot.out
139242fb18ec6c02ab6fb2c7f356a97c66ea3c7a7541c6a03a20d1c49e16b16d  …-r4-seg4-today.out
171c8f8e300bbb60d3fe1c18be26708e75910bb632f9cc9691254d3401e31039  …-r4-seg4-yi-cfg-carry.out
b809c2d367c0dfbbdea0ef98c04bec856ae3e1c55f1b0a24869b0f74c8a47211  …-r4-seg4-yi-cfg.out
74644f3247ca3c712e935e94ed9732ee4c191bbe3a3c9afd9827e2d9867798b7  …-r4-seg4-yi-narrow-cfg-carry.out
2e530e0064b4e5239270ba864ed86b9994c389029b82d6e7934dd18d859996cc  …-r4-seg4-yi-narrow-cfg.out
72e8a97747b7c845271ea9113c7088648a76ff471c94f66c1023c25286fa8f37  …-r4-seg4-yi-narrow-slot.out
1596007f9bd889b576495b4af35c883b2f5d103d44b5b6d32f0eaf0e9bfef0b6  …-r4-seg4-yi-slot.out
```

## 三、产物齐不齐、7.2 的 L 族格数

各臂产物的行名计数（`grep -o 'name=[a-z0-9_]*' <产物> | sort | uniq -c`）：12 份臂产物都是 `r4_l_cell` 663、`r4_q6_cell` 72、`r4_l5_twin` 6、`r4_today_base_check` 6、`r4_positive_control` 1（PC-554）、`r4_section_seven_one_f14` 4、`r4_section_seven_two_anchor` 18、`r4_local_constant_check` 34、`r4_arm_code_check` 1、`r4_cross_checks` 1；只有 today 另有 `r4_section_seven_one_f5` 8。compare：`r4_q4` 110、`r4_q5` 135、`r4_q6` 12、`r4_void` 84、`r4_cross_arm_sentence` 54、`r4_section_seven_three` 9、`r4_base_identity` 12、`r4_f11` 1、`r4_missed_arm_and_today_sentences` 1。

today 那一臂的 L 族分族格数（`tables.sh`「L 族每族格数」）：`8 family=l0`、`116 family=l1a`、`205 family=l1b`、`197 family=l1c`、`16 family=l2`、`24 family=l3`、`16 family=l4`、`72 family=l5`、`8 family=l6`、`1 family=pc554`；`663  truth_newer=false`（没有一格 N_真 为真，所以 Q4 的「拦下」栏在 L 族上本来就是空的）。

7.2 钉死的格数，today 那一臂的原样行（其余 11 臂各有同样的 5 行，都是 `pass`）：

```
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-today.out:810:E7RESULT name=r4_section_seven_two_anchor arm=today geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) item="L5 起始镜像 3×2" computed=6 registered=6 verdict=pass
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-today.out:812:E7RESULT name=r4_section_seven_two_anchor arm=today item="l0 格数" computed=8 registered=8 verdict=pass
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-today.out:813:E7RESULT name=r4_section_seven_two_anchor arm=today item="l3 格数" computed=24 registered=24 verdict=pass
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-today.out:814:E7RESULT name=r4_section_seven_two_anchor arm=today item="l4 格数" computed=16 registered=16 verdict=pass
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-today.out:815:E7RESULT name=r4_section_seven_two_anchor arm=today item="l6 格数" computed=8 registered=8 verdict=pass
```

L1、L2、L5 的 m 不钉（r3 登记 7.2 原文）。L5 每份起始镜像 M = 6（`r4_l5_twin` 的 `today_page_reads=[6]`，6 份都是）；6 × 6 × 2 = 72 格。

检查项里只核了 0 项的，照没判写：compare 的 `r4_base_identity` 12 行都是 `prefixes=0 differing=0 verdict=pass`（第四段没有 `r4_prefix` 行，L 族读的是今天那一臂造好的起始镜像）；`r4_section_seven_three row=3` 6 行都是 `cells=0`（这一段没有 T(τ) 格）。7.3 真正核了的是 row=2（`device_n_cfg` 对 jia-cfg、`device_n_slot` 对 jia-slot，各 663 格，`mismatches=0`）与 row=4（today 663 格，`nonzero_cells=0`）；各臂的 `r4_cross_checks` 都是 `s4_mismatches=0`，today `f13_checked=651 f13_failures=0`。

## 四、阳性对照与 F 条款（PC-多拒、PC-22、PC-554；F1、F2、F5、F14）

**PC-多拒**（compare 第 260–292 行，36 句）：11 条臂各 3 句（`today_ok_on_the_cell` [今]、`arm_refuses` [臂]、`counted_as_extra_refusal` [测]），全部 `verdict=pass`，都在 n1 = 1 上（没有顺延），`truth_newer=false`。举两行原样：
```
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-compare.out:262:E7RESULT name=r4_cross_arm_sentence arm=bing-cfg control=PC-extra-refusal sentence=counted_as_extra_refusal tag=measure verdict=pass n1=1 truth_newer=false
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-compare.out:274:E7RESULT name=r4_cross_arm_sentence arm=jia-slot control=PC-extra-refusal sentence=counted_as_extra_refusal tag=measure verdict=pass n1=1 truth_newer=false
```

**PC-554**（每臂产物各 1 行，12 行全 `verdict=pass`，`missed_routes=none`）。today（F1 那一句）与 jia-slot（-槽 拒那一句）原样：
```
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-today.out:705:E7RESULT name=r4_positive_control arm=today control=PC-554 n1=1 b=unit_first_copy verdict=pass sentences=1 sentence_today_ok_count_zero_isolated_zero=pass missed_routes=none abandoned_root=1:4 exclusive_slots=24 truth_newer_on_the_twin=false a_class=K0 a_error=none a_writes=71 a_reads=2090 a_read_bytes=8617984 a_waits=0 a_chosen=2:7 a_effective=2:7 a_abandoned_roots_unreadable=0 a_isolated=0 a_arm_observation=none a_reads_before_first_wait=none a_reads_after_first_wait=0 a_faults_intercepted_before_first_wait=none a_zero_reads_after_first_wait=0 a_arm_rounds_limit=none a_arm_caches_zeroed_journal_slots=none device_chosen=2:7 device_effective=2:7 device_n_cfg=false device_c_witness=7 device_c_e=7 device_undecidable=false device_n_slot=true device_unreadable_root_slots=1 truth_newer=false truth_orders_disagree=false truth_by_records_only=false
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-jia-slot.out:697:E7RESULT name=r4_positive_control arm=jia-slot control=PC-554 n1=1 b=unit_first_copy verdict=pass sentences=1 sentence_arm_slot_arm_refused=pass missed_routes=none abandoned_root=1:4 exclusive_slots=24 truth_newer_on_the_twin=false a_class=K3 a_error=R3UnreadableRootRingSlot a_writes=0 a_reads=1568 a_read_bytes=6336512 a_waits=0 a_chosen=none a_effective=none a_abandoned_roots_unreadable=none a_isolated=none a_arm_observation=judged:true,newer:true,tail:0,last:none,undecidable:false,unreadable_slots:1,rounds:0,rebuilt:false,table_unreadable:false,table_rereads:0 a_reads_before_first_wait=none a_reads_after_first_wait=0 a_faults_intercepted_before_first_wait=none a_zero_reads_after_first_wait=0 a_arm_rounds_limit=0 a_arm_caches_zeroed_journal_slots=false device_chosen=2:7 device_effective=2:7 device_n_cfg=false device_c_witness=7 device_c_e=7 device_undecidable=false device_n_slot=true device_unreadable_root_slots=1 truth_newer=false truth_orders_disagree=false truth_by_records_only=false
```
今天那一臂：`a_abandoned_roots_unreadable=0 a_isolated=0`，而 `exclusive_slots=24`（被抛弃根独占 24 个槽，一个都没隔离）。**F1 没触发**。-配置、-配置续 各臂与今天同一结局（K0、计数 0、隔离 0）；jia-slot、yi-slot、yi-narrow-slot 拒（K3，`R3UnreadableRootRingSlot`），这一格同时记进它们的 Q4（pc554 那一栏各 1）。

**PC-22**（compare 第 293–313 行）：
```
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-compare.out:293:E7RESULT name=r4_cross_arm_sentence arm=today control=PC-22 sentence=an_m_with_ok_zero_count_zero_isolation tag=today verdict=pass n1=1,b=unit_first_copy,page=0,m=2
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-compare.out:294:E7RESULT name=r4_cross_arm_sentence arm=jia-cfg control=PC-22 sentence=refused_only tag=arm verdict=fail n1=1,b=unit_first_copy,page=0,m=2
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-compare.out:295:E7RESULT name=r4_cross_arm_sentence arm=jia-cfg control=PC-22 sentence=refused_from tag=arm verdict=pass n1=1,b=unit_first_copy,page=0,m=2
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-compare.out:296:E7RESULT name=r4_cross_arm_sentence arm=jia-cfg-carry control=PC-22 sentence=refused_only tag=arm verdict=fail n1=1,b=unit_first_copy,page=0,m=2
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-compare.out:297:E7RESULT name=r4_cross_arm_sentence arm=jia-cfg-carry control=PC-22 sentence=refused_from tag=arm verdict=pass n1=1,b=unit_first_copy,page=0,m=2
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-compare.out:298:E7RESULT name=r4_cross_arm_sentence arm=jia-slot control=PC-22 sentence=refused_only tag=arm verdict=fail n1=1,b=unit_first_copy,page=0,m=2
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-compare.out:299:E7RESULT name=r4_cross_arm_sentence arm=jia-slot control=PC-22 sentence=refused_from tag=arm verdict=pass n1=1,b=unit_first_copy,page=0,m=2
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-compare.out:300:E7RESULT name=r4_cross_arm_sentence arm=yi-cfg control=PC-22 sentence=refused_from tag=arm verdict=pass n1=1,b=unit_first_copy,page=0,m=2
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-compare.out:301:E7RESULT name=r4_cross_arm_sentence arm=yi-cfg control=PC-22 sentence=ok_only_with_twin_isolation tag=arm verdict=pass n1=1,b=unit_first_copy,page=0,m=2
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-compare.out:312:E7RESULT name=r4_cross_arm_sentence arm=bing-cfg control=PC-22 sentence=page_read_once_less_and_same_isolation_as_today tag=arm verdict=pass n1=1,b=unit_first_copy,page=0,m=2
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-compare.out:431:E7RESULT name=r4_missed_arm_and_today_sentences count=3 list=jia-cfg:PC-22:refused_only(arm),jia-cfg-carry:PC-22:refused_only(arm),jia-slot:PC-22:refused_only(arm)
```
- 今天那一句 [今] `pass`（m = 2，取的是 O(2 起) 那一格：今天 `Ok`、`abandoned_roots_unreadable=0`、隔离 0，孪生镜像隔离 28）。**F2 没触发**。
- 乙、乙-窄读 六臂两句都 `pass`；丙 两臂 `pass`；甲 三臂 `refused_from` `pass`、**`refused_only` `fail`**。
- 那一格的原样行（today 与 jia-cfg 的 O(2 只)、O(2 起)）：
```
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-today.out:684:E7RESULT name=r4_l_cell family=l5 arm=today geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) n1=1 b=unit_first_copy page=0 m=2 duration=O2only base_fingerprint=f63c57aa261c0df9 write_fingerprint=242b328be4595fca faults=2 unintercepted=instance_table_page0(device=1,slot=50304) refused=false truth_newer_on_the_twin=false a_class=K0 a_error=none a_writes=71 a_reads=2098 a_read_bytes=8765440 a_waits=0 a_chosen=2:7 a_effective=2:7 a_abandoned_roots_unreadable=0 a_isolated=28 a_arm_observation=none a_reads_before_first_wait=none a_reads_after_first_wait=0 a_faults_intercepted_before_first_wait=none a_zero_reads_after_first_wait=0 a_arm_rounds_limit=none a_arm_caches_zeroed_journal_slots=none device_chosen=2:7 device_effective=2:7 device_n_cfg=false device_c_witness=7 device_c_e=7 device_undecidable=false device_n_slot=false device_unreadable_root_slots=0 truth_newer=false truth_orders_disagree=false truth_by_records_only=false twin_isolated=28
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-today.out:686:E7RESULT name=r4_l_cell family=l5 arm=today geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) n1=1 b=unit_first_copy page=0 m=2 duration=O2from base_fingerprint=f63c57aa261c0df9 write_fingerprint=242b328be4595fca faults=2 unintercepted= refused=false truth_newer_on_the_twin=false a_class=K0 a_error=none a_writes=71 a_reads=2102 a_read_bytes=8896512 a_waits=0 a_chosen=2:7 a_effective=2:7 a_abandoned_roots_unreadable=0 a_isolated=0 a_arm_observation=none a_reads_before_first_wait=none a_reads_after_first_wait=0 a_faults_intercepted_before_first_wait=none a_zero_reads_after_first_wait=0 a_arm_rounds_limit=none a_arm_caches_zeroed_journal_slots=none device_chosen=2:7 device_effective=2:7 device_n_cfg=false device_c_witness=7 device_c_e=7 device_undecidable=false device_n_slot=false device_unreadable_root_slots=0 truth_newer=false truth_orders_disagree=false truth_by_records_only=false twin_isolated=28
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-jia-cfg.out:676:E7RESULT name=r4_l_cell family=l5 arm=jia-cfg geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) n1=1 b=unit_first_copy page=0 m=2 duration=O2only base_fingerprint=f63c57aa261c0df9 write_fingerprint=242b328be4595fca faults=2 unintercepted=instance_table_page0(device=1,slot=50304) refused=false truth_newer_on_the_twin=false a_class=K0 a_error=none a_writes=71 a_reads=2102 a_read_bytes=8781824 a_waits=0 a_chosen=2:7 a_effective=2:7 a_abandoned_roots_unreadable=0 a_isolated=28 a_arm_observation=judged:true,newer:false,tail:7,last:7,undecidable:false,unreadable_slots:0,rounds:0,rebuilt:false,table_unreadable:false,table_rereads:0 a_reads_before_first_wait=none a_reads_after_first_wait=0 a_faults_intercepted_before_first_wait=none a_zero_reads_after_first_wait=0 a_arm_rounds_limit=0 a_arm_caches_zeroed_journal_slots=false device_chosen=2:7 device_effective=2:7 device_n_cfg=false device_c_witness=7 device_c_e=7 device_undecidable=false device_n_slot=false device_unreadable_root_slots=0 truth_newer=false truth_orders_disagree=false truth_by_records_only=false twin_isolated=28
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-jia-cfg.out:678:E7RESULT name=r4_l_cell family=l5 arm=jia-cfg geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) n1=1 b=unit_first_copy page=0 m=2 duration=O2from base_fingerprint=f63c57aa261c0df9 write_fingerprint=cbf29ce484222325 faults=2 unintercepted= refused=true truth_newer_on_the_twin=false a_class=K3 a_error=R3NewestInstanceTableUnreadableForTheShadowLedger a_writes=0 a_reads=1715 a_read_bytes=6889472 a_waits=0 a_chosen=none a_effective=none a_abandoned_roots_unreadable=none a_isolated=none a_arm_observation=judged:true,newer:false,tail:7,last:7,undecidable:false,unreadable_slots:0,rounds:0,rebuilt:false,table_unreadable:true,table_rereads:0 a_reads_before_first_wait=none a_reads_after_first_wait=0 a_faults_intercepted_before_first_wait=none a_zero_reads_after_first_wait=0 a_arm_rounds_limit=0 a_arm_caches_zeroed_journal_slots=false device_chosen=2:7 device_effective=2:7 device_n_cfg=false device_c_witness=7 device_c_e=7 device_undecidable=false device_n_slot=false device_unreadable_root_slots=0 truth_newer=false truth_orders_disagree=false truth_by_records_only=false twin_isolated=28
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-yi-cfg.out:676:E7RESULT name=r4_l_cell family=l5 arm=yi-cfg geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) n1=1 b=unit_first_copy page=0 m=2 duration=O2only base_fingerprint=f63c57aa261c0df9 write_fingerprint=242b328be4595fca faults=2 unintercepted=instance_table_page0(device=1,slot=50304) refused=false truth_newer_on_the_twin=false a_class=K0 a_error=none a_writes=71 a_reads=2102 a_read_bytes=8781824 a_waits=0 a_chosen=2:7 a_effective=2:7 a_abandoned_roots_unreadable=0 a_isolated=28 a_arm_observation=judged:true,newer:false,tail:7,last:7,undecidable:false,unreadable_slots:0,rounds:0,rebuilt:false,table_unreadable:false,table_rereads:0 a_reads_before_first_wait=none a_reads_after_first_wait=0 a_faults_intercepted_before_first_wait=none a_zero_reads_after_first_wait=0 a_arm_rounds_limit=1 a_arm_caches_zeroed_journal_slots=false device_chosen=2:7 device_effective=2:7 device_n_cfg=false device_c_witness=7 device_c_e=7 device_undecidable=false device_n_slot=false device_unreadable_root_slots=0 truth_newer=false truth_orders_disagree=false truth_by_records_only=false twin_isolated=28
research/results/e158-root-choice-repair-2026-09-27-r4-seg4-yi-cfg.out:678:E7RESULT name=r4_l_cell family=l5 arm=yi-cfg geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) n1=1 b=unit_first_copy page=0 m=2 duration=O2from base_fingerprint=f63c57aa261c0df9 write_fingerprint=cbf29ce484222325 faults=2 unintercepted= refused=true truth_newer_on_the_twin=false a_class=K3 a_error=R3NewestInstanceTableUnreadableForTheShadowLedger a_writes=0 a_reads=1717 a_read_bytes=6955008 a_waits=1 a_chosen=none a_effective=none a_abandoned_roots_unreadable=none a_isolated=none a_arm_observation=judged:true,newer:false,tail:7,last:7,undecidable:false,unreadable_slots:0,rounds:0,rebuilt:false,table_unreadable:true,table_rereads:1 a_reads_before_first_wait=1715 a_reads_after_first_wait=2 a_faults_intercepted_before_first_wait=2 a_zero_reads_after_first_wait=0 a_arm_rounds_limit=1 a_arm_caches_zeroed_journal_slots=false device_chosen=2:7 device_effective=2:7 device_n_cfg=false device_c_witness=7 device_c_e=7 device_undecidable=false device_n_slot=false device_unreadable_root_slots=0 truth_newer=false truth_orders_disagree=false truth_by_records_only=false twin_isolated=28
```

**甲 `refused_only` 打不中，按「判据自己也会写错」四问**（只陈述产物，不改判据）：
- ① 分不分辨臂：只打中甲 三臂。乙、乙-窄读 在同一格（O(2 只)）也是 `Ok`、隔离 28 = 孪生，它们那一句「O(m 只) 下 `Ok` 且隔离 = 孪生」判 `pass`。
- ② 判别子看得到：看得到。O(2 只) 那一格今天与各臂都是 `unintercepted=instance_table_page0(device=1,slot=50304)`，而且都是 `table_unreadable:false,table_rereads:0`。这说明第 2 次读只打坏了盘 0 那一份，盘 1 那一份照常读出，那张实例表在这次挂载里其实读得出，没有哪条臂走到了「第二次读实例表失败」那一格。
- ③ 字面上满足的是哪一分句：登记 r3 第 404 行写的是「在那个 m 上：甲 三臂拒」，装置把两种时长各打一句。O(m 只) 那一格上没有第 22 条那一形，甲照定义不该拒；它交回 `Ok`，隔离 28 = 孪生。
- ④ 跑前给的处置在这一格上中不中：V1 ④ 作废甲 三臂的 Q6，并「走那一句点名的 F 条款的三步」。可是 PC-22 的 [臂] 句在登记修订第 12 条的对应表里没有点名 F 条款（装置的 `missed_routes` 也没给），**三步走哪一条要主 agent 定**。
- 连带一件：乙那句 O(m 只) 的 `pass` 也没走到乙的重读（`table_rereads:0`、`a_waits=0`）。所以 PC-22 这一次没有证明乙的「重读后读出」在第 22 条那一格上生效；见第七节末。

**F5**（today L6 8 行，全 `pass`，`effective` = `hidden`，例 `research/results/e158-root-choice-repair-2026-09-27-r4-seg4-today.out:606:E7RESULT name=r4_section_seven_one_f5 arm=today n1=1 form=read_fails verdict=pass effective=1:4 hidden=1:4`）：没触发。
**F14**（每臂 4 行，n1 = 0..3、c = U，12 臂共 48 行全 `pass`，数法 `grep -n 'name=r4_section_seven_one_f14' …-r4-seg4-*.out | grep -c 'verdict=pass'` → `48`；例 `research/results/e158-root-choice-repair-2026-09-27-r4-seg4-today.out:173:E7RESULT name=r4_section_seven_one_f14 arm=today geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) n1=1 c=U verdict=pass txg_before_unmount=4 floors=[Some(4), Some(4)] unmount_publishes=3 expected_publishes=3`）：没触发。登记 7.1 那一行还点了 L1(c)；L1(c) 是卸载写序列上的崩溃状态，没有完整走完的卸载，所以这一段只有 L0 那 4 次卸载可核。
**F11**：`E7RESULT name=r4_f11 cells=590 cells_differing_outside_o1_and_cell_22=0 verdict=inference_holds`（compare 第 322 行）。

## 五、Q4 多拒（L 族，G0）

命令 `bash /tmp/claude-1000/e158-r4-seg4/tables.sh` 的「Q4」一段，原样：

```
arm                     l0   l1a   l1b   l1c    l2    l3    l4    l5    l6 pc554 total mounts
bing-cfg                 0     0     0     0     0    16     0     0     0     0    16    663
bing-cfg-carry           0     0     0     0     0    16     0     0     0     0    16    663
jia-cfg                  0     0     0     0     0    16     0     6     0     0    22    663
jia-cfg-carry            0     0     0     0     0    16     0     6     0     0    22    663
jia-slot                 0     0     0     0     8     0     0     6     4     1    19    663
yi-cfg                   0     0     0     0     0    16     0     6     0     0    22    663
yi-cfg-carry             0     0     0     0     0    16     0     6     0     0    22    663
yi-narrow-cfg            0     0     0     0     0    16     0     6     0     0    22    663
yi-narrow-cfg-carry      0     0     0     0     0    16     0     6     0     0    22    663
yi-narrow-slot           0     0     0     0     8     0     0     6     4     1    19    663
yi-slot                  0     0     0     0     8     0     0     6     4     1    19    663
```

- 「拦下」栏（`intercepted`）110 行都是空的，因为 L 族 663 格都是 N_真 为假。
- 结局类（`tables.sh`「结局类」）：today K0 651 / K3 12；-配置 各臂（甲、乙、乙-窄读）K3 34；丙 K3 28；-槽 K3 31。臂的 K3 数减去今天的 12，正好等于上表的合计。今天那 12 格 K3 是 L5 的 m = 1（6 份起始镜像 × 2 种时长），各臂在这 12 格上也拒，不算多拒。
- **L5 那 6 格多拒是哪几格**：甲、乙、乙-窄读 各臂（-配置 与 -槽 都一样）在每份起始镜像的 m = 2、O(2 起) 上拒（`R3NewestInstanceTableUnreadableForTheShadowLedger`）。6 格里一半在 b = `records`（今天 `Ok`、隔离 0 = 孪生 0，今天在这一格罩住了），一半在 b = `unit_first_copy`（今天 `Ok`、隔离 0 < 孪生 28，就是今天没罩住的那 3 格）。照 Q4 的字面定义（臂拒 ∧ 今天不拒 ∧ N_真 为假），这 6 格都算多拒。其中 3 格正是第 22 条那一格罩住的代价；这 3 格该不该算进「选哪个」的多拒，交主 agent（第十节第 2 项）。丙 在这 6 格上 `Ok`、隔离 = 孪生（用第一次读出的那张表），所以没有多拒。
- 同一种 N 里，甲与乙（含窄读）的多拒在 L 族上逐族相等；按这一段的量，多拒这一样分不出甲、乙。

## 六、Q5 多读

「同成」一栏（`tables.sh`「Q5 同成」，原样）：

```
bing-cfg-carry       cells=635 positive_cells=0 total_extra_read_calls=-10831 min=-18 max=-17
bing-cfg             cells=635 positive_cells=0 total_extra_read_calls=-13371 min=-22 max=-21
jia-cfg-carry        cells=629 positive_cells=629 total_extra_read_calls=5032 min=8 max=8
jia-cfg              cells=629 positive_cells=629 total_extra_read_calls=2516 min=4 max=4
jia-slot             cells=632 positive_cells=0 total_extra_read_calls=0 min=0 max=0
yi-cfg-carry         cells=629 positive_cells=629 total_extra_read_calls=4988 min=7 max=8
yi-cfg               cells=629 positive_cells=629 total_extra_read_calls=2472 min=3 max=4
yi-narrow-cfg-carry  cells=629 positive_cells=629 total_extra_read_calls=4988 min=7 max=8
yi-narrow-cfg        cells=629 positive_cells=629 total_extra_read_calls=2472 min=3 max=4
yi-narrow-slot       cells=632 positive_cells=0 total_extra_read_calls=-40 min=-1 max=0
yi-slot              cells=632 positive_cells=0 total_extra_read_calls=-40 min=-1 max=0
```

- 分族的原样行在 compare 的 `name=r4_q5` 行（135 行，分族表见草稿 `q5-table.txt`）。各臂同成的格数不同（丙 635、-配置 各臂 629、-槽 各臂 632），因为拒的格不同。合计只能在格数相同的臂之间比：甲-配置 2516 对乙-配置 2472（都是 629 格）。乙 在 L1a / L1b / L1c 的一部分格上和 L6 的全部 8 格上每格 +3，比甲的 +4 少一次（原因推测是读缓存命中，没核）。
- L0 那 8 格的每格差（PC-多读 (ii) 的点值，那条对照归第一段判，这里只报数）：甲-配置 +4、甲-配置续 +8、乙-配置 +4、乙-配置续 +8、甲-槽 / 乙-槽 / 乙-窄读-槽 0（登记写 +3S = 24；主 agent 认定第 1 项：走 F18、那几臂的 Q5 按 V1 作废，由第一段的 compare 判）、丙-配置 −21 = 4 − 24 − 1、丙-配置续 −17 = 8 − 24 − 1（即减 3S + p，p = 1；算式是我套的）。
- 「拒」一栏（附带）：丙 在 L3 上减今天每格 +1032..+1113；乙-配置 / 乙-配置续 在 L3 上 +1049..+1133；乙-窄读-配置 在 L3 上 −481..−397；乙-槽 在 L2 / L6 / pc554 上 +1017..+1130；乙-窄读-槽 在同几族上 −505..−400。原样行在 compare。
- 本段 compare 的 `r4_void` 里，Q5-both-mounted 与 Q5-refused 12 臂都是 `voided=false`。本段的输入里没有管 Q5 的 PC-多读，所以这个 `false` 不代表跨段合算之后也不作废。

## 七、Q6 第 22 条那一格

compare 第 335–346 行原样：

```
E7RESULT name=r4_q6 arm=bing-cfg geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=0 verdict=covered
E7RESULT name=r4_q6 arm=bing-cfg-carry geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=0 verdict=covered
E7RESULT name=r4_q6 arm=jia-cfg geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=0 verdict=covered
E7RESULT name=r4_q6 arm=jia-cfg-carry geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=0 verdict=covered
E7RESULT name=r4_q6 arm=jia-slot geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=0 verdict=covered
E7RESULT name=r4_q6 arm=today geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=3 verdict=not_covered
E7RESULT name=r4_q6 arm=yi-cfg geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=0 verdict=covered
E7RESULT name=r4_q6 arm=yi-cfg-carry geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=0 verdict=covered
E7RESULT name=r4_q6 arm=yi-narrow-cfg geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=0 verdict=covered
E7RESULT name=r4_q6 arm=yi-narrow-cfg-carry geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=0 verdict=covered
E7RESULT name=r4_q6 arm=yi-narrow-slot geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=0 verdict=covered
E7RESULT name=r4_q6 arm=yi-slot geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=0 verdict=covered
```

- 作废（compare `r4_void`，12 × 7 = 84 行里 `voided=true` 的只有这 3 行）：`E7RESULT name=r4_void arm=jia-cfg quantity=Q6 voided=true basis=PC-22:refused_only(arm)`，jia-cfg-carry、jia-slot 各一行，写法相同。
- 今天没罩住的 3 格就是 n1 = 1、2、3，b = `unit_first_copy`，m = 2，O(2 起)（`ok=true iso=0 twin=28 aru=0`）。
- 按第六节开头的读法，Q6 对候选臂「不算判据、只核臂的实现做到了定义」，判别力落在今天那一臂上：今天没罩住，PC-22 的 [今] 句 `pass`。
- **L5 里乙的重读没有生效的格**（`tables.sh`「L5 里乙 / 甲」）：乙-配置、乙-窄读-配置、乙-槽 都是 54 × `K0 table_unreadable:false,table_rereads:0`、12 × `K3 table_unreadable:false,table_rereads:0`、6 × `K3 table_unreadable:true,table_rereads:1`；甲-配置 除最后一类的 `table_rereads` 是 0 之外完全相同。L5 没有一格是乙重读之后读出、挂上的。O(m 只) 只打坏两份合计的第 m 次读，另一份照样读得出，走不到第 22 条那一格；O(m 起) 从第 m 次起一直读坏，重读也读坏。所以在第 22 条那一格上，L5 这个构造分不出乙与甲。要分出来，得有「两份都读坏一次、重读时读得出」那种时长。这一格登记里没有，要不要加交主 agent（第十节第 3 项）。

## 八、判决行逐行点名（第 4c 步）

- `grep -n 'name=verdict' <产物>` 在 13 份产物上都是 0 行：这个装置不打 `name=verdict` 行，判决分散在各行的 `verdict=` 字段里。按字段数（`tables.sh`「判决类取值」，原样）：

```
     12 name=r4_arm_code_check verdict=pass
     12 name=r4_base_identity verdict=pass
     12 name=r4_compare_input verdict=pass
      3 name=r4_cross_arm_sentence verdict=fail
     51 name=r4_cross_arm_sentence verdict=pass
      1 name=r4_f11 verdict=inference_holds
    408 name=r4_local_constant_check verdict=pass
     12 name=r4_positive_control verdict=pass
     11 name=r4_q6 verdict=covered
      1 name=r4_q6 verdict=not_covered
     48 name=r4_section_seven_one_f14 verdict=pass
      8 name=r4_section_seven_one_f5 verdict=pass
      9 name=r4_section_seven_three verdict=pass
    216 name=r4_section_seven_two_anchor verdict=pass
```

- 表示「没过」的，逐个点名：
  1. compare 第 294、296、298 行：`r4_cross_arm_sentence` 的 `verdict=fail`，出现在 jia-cfg、jia-cfg-carry、jia-slot 的 PC-22 `refused_only`。原因见第四节。**不在登记预期内**（登记预期甲拒）。
  2. compare 第 340 行：`r4_q6 arm=today … not_covered_cells=3 verdict=not_covered`。今天那一臂没罩住第 22 条，**在登记预期内**（PC-22 [今] 句要的就是这一形）。
  3. compare 第 431 行：`r4_missed_arm_and_today_sentences count=3` 是上面第 1 项的汇总，整数计数 3。
  4. compare 里 3 行 `r4_void … quantity=Q6 voided=true`（jia 三臂），是第 1 项的后果。
- 名字表示违例、不匹配、失败的整数字段（`grep -o '[a-z_]*\(mismatch\|violation\|fail\|ambig\|differ\|missed\|not_run\|stop\)[a-z_]*=[^ ]*'`）：`_mismatches=0` 12、`_failures=0` 12、`differing=0` 12、`differences=0` 6、`mismatches=0` 2、`missed_routes=none` 12。全是 0 或 none。`not_run` / `not_constructible` 0 处。

## 九、岔路表（岔路单 `research/prompts/c554-fix-forks.md` 第 1 行）

| 那一样 | 状态 | 还差什么 | 剩下的量能不能让它翻面 | 算它的命令 |
|---|---|---|---|---|
| 丢写（每个候选） | 这一段不量 | 第一段（H1d、H1e）、第二段（实七、实八、H-随，装置还没写）、第三段（H1f、H1g） | 能 | 各段 compare 的 `name=r4_loss` 行 |
| 多拒（Q4），L 族部分 | **L 族在 G0 上已量完**：甲-配置 / 甲-配置续 / 乙-配置 / 乙-配置续 / 乙-窄读-配置 / 乙-窄读-配置续 各 22，丙 两臂各 16，-槽 三臂各 19；甲与乙逐族相等 | 第二段的实七、实八与 H-随全（登记 5.5 第 9 项认定，那一族的多拒另算）；跨段 V1：第一段 PC-N0 管 Q4，那条对照本段 compare 没读；L5 那 3 格「罩住第 22 条的代价」算不算多拒要主 agent 定 | 能：H-随全 68 段与实七、实八的合法挂载可能让甲、乙分开，也可能改变丙的 −6 | `bash /tmp/claude-1000/e158-r4-seg4/tables.sh` 的 Q4 段，数自 `…-r4-seg4-compare.out` 的 `name=r4_q4` 行 |
| 多读（Q5 同成） | L 族已量：每格差 甲-配置 +4、乙-配置 +3..+4、丙-配置 −22..−21；-配置续 甲 +8、乙 +7..+8、丙 −18..−17；-槽 各 0 / −1 | 第一段 PC-多读 (i)(ii) 的判定与跨段 V1（-槽 三臂按主 agent 认定第 1 项可能作废）；第二段的实七、实八、H-随 | 能：本段的 L 族合计里甲、乙只差 44 次（2516 对 2472），第二段的格可能改变先后 | `tables.sh` 的 Q5 段，数自 compare 的 `name=r4_q5 … column=both_mounted` 行 |
| 第 22 条那一格（Q6） | 今天没罩住 3 格（PC-22 [今] `pass`）；乙、乙-窄读、丙 各臂 0 格没罩住；**甲 三臂 Q6 按 V1 ④ 作废**（PC-22 `refused_only` `fail`） | 甲那一句打不中之后走哪一条 F 的三步，要主 agent 定；L5 分不出乙的重读（第七节末），要不要补「两份各读坏一次」的时长，要主 agent 定 | 不能让「罩住」翻面：候选各臂按定义罩住，本段 0 格没罩住。能不能分出乙与甲：L5 这个构造分不出 | compare 的 `name=r4_q6` 行、`r4_cross_arm_sentence … control=PC-22`、`r4_void … quantity=Q6` |
| 够判条件第二半 | 已由装置执行员在快照上核过（登记第十二节修订第 3 条） | — | 不能 | 同装置报告第七节 |

11.3 够判：这一段结束后，第 1 行还开着（丢写还差第一、二、三段；多拒与多读还差第二段和跨段 V1）。这一段没有能判「够判后未跑」的量。

## 十、要主 agent 定的

1. **甲 三臂的 PC-22 `refused_only`（[臂]）打不中**：Q6 已经按 V1 ④ 作废（compare 的 `r4_void`）。V1 ④ 要求再走「那一句点名的 F 条款的三步」，但 PC-22 的 [臂] 句没有点名任何 F 条款。产物显示打不中的原因在构造：O(m 只) 那一格上实例表读得出，甲不该拒。怎么记这次输、要不要收严，由主 agent 定。
2. **L5 的 6 格多拒里，有 3 格是罩住第 22 条的代价**：今天在那 3 格上 `Ok` 但隔离 0 < 孪生 28。照 Q4 的字面定义它们算多拒；甲、乙、-槽 各臂都有，丙 没有。要不要在「选哪个」时单列，由主 agent 定。
3. **L5 分不出乙的重读**：乙 的重读在 L5 上没有一格是读出之后挂上的。要在第 22 条那一格上比出乙与甲的差别，得加一种「两份各读坏一次、重读读得出」的时长。这是新加的一格，要主 agent 定（按登记规矩只能在新一次跑或修订里加）。
4. 本段的 compare 只读第四段的产物，V1 的跨段作废没算：第一段的 PC-N0（管 Q4）和 PC-多读（管 Q5）打不中的话，本段的 Q4 / Q5 在那几臂上也要作废。最后要拿全部段的前缀一起跑一次 `r4-compare`（装置报告第五节已写这一句）。

## 十一、门禁（登记给 experiment-runner 的阶段，主工作区，JST 10:2x）

各阶段的退出码取自这一次跑的原样输出，末行用 `tail -n 1` 从各自的日志抄（日志在草稿目录 `gate-<阶段>.log`）：

```
33-mutation-tables.sh rc=0 |   ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 1132 条的原文各命中源码一次；没有同名实验二进制、锚点不在本阶段射程的表 7 张：research/mutations/e125_zoned_wp.tsv research/mutations/e129_tear_injector.tsv research/mutations/e129_thin_neighbour.tsv research/mutations/e129_thin_rmw.tsv research/mutations/e158_arms.tsv research/mutations/e158_r3_arm_mutations.tsv research/mutations/e158_r4_arm_mutations.tsv （本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）
52-segment-registry.sh rc=1 |        再让 research/scripts/replay.sh 里 E142 那一行的入库产物名字跟上。
80-absolute-assertions.sh rc=0 |       crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs
96-experiment-source-discipline.sh rc=0 |         e92_reuse_requirement.rs
27-format-constants.sh rc=1 |      → 旧值只许留在「## 历史版本」之后与 *-history.md 里。
34-experiment-index-sync.sh rc=0 |   ✓ 实验索引行与正文标题一致（索引 159 行、正文 159 份）
40-results-cited.sh rc=1 |        E160：e160-e152-input-mirror-2026-09-15.out e160-e152-input-single-2026-09-15.out e160-random-small-read-share-segment1-2026-09-24.out e160-random-small-read-share-segment1-2026-09-24-realweight.out
69-evidence-in-repo.sh rc=1 |                原件已经没了，就写明它没了、把还核得动的那部分落进仓里；只是在说做法（草稿放在哪），把句子里的依据词去掉。
75-decision-experiment-links.sh rc=1 |             支撑 / 推翻的那条分项在「**依据**」段引回这个实验；待回填清单只能删行，新实验页当场写全这张表。
84-verdict-false-named.sh rc=0 |   ✓ 登记产物里判决字段是 false 的都在对应实验页里点了名（点名 3 个）
85-repro-command.sh rc=0 |     写明「原始输出未留存」不判的 0 页：（没有）
86-experiment-orphans.sh rc=0 |     扫了 research crates/singlefs-harness/src/bin（target/ 不扫）；不在的目录 0 个：（没有）
88-quoted-result-lines.sh rc=77 |   ! 这次改动新增或改写的 kb 正文里没有整行抄的 E7RESULT 行，本阶段无对象可判
99-multipath-registry.sh rc=0 |   ✓ 登记了路径与共用项的实验页判过了（3 份、14 条路径）；还没写登记节的 9 份在 multipath-registry-lag.tsv 里、每一份都现存且确实还没写，与基准 faf255e235300d129ede6d6f85af31d686a88519 比没涨；逐份补齐后删行
```

- 33 号这次是绿的：补丁已经打上，`crates/mutations.tsv` 的 1132 条都各命中一次。
- 红的几道里，只有 40 号点到这一段：`grep -c 'r4-seg4' gate-40-results-cited.sh.log` 输出 `13`，就是这一段的 13 份新产物还没被实验页点名。这是预期内的，派发提示写明这一次不写实验页，**「这 13 份 `…-r4-seg4-*.out` 还没被实验页点名」交主 agent 派人点名**。40 号还点了第 3 次跑与第 4 次跑第一、三段的产物，不是这一段写的。
- 27、52、69、75 号的日志里 `grep -ci e158` 都是 0，红在别处，照写不修。69 号这次没再点 E158 的变异表（各段产物已经入库）。
- 88 号退出码 77，是本次没跑（没有对象），不算通过。

## 十二、没做什么

- 没写实验页、索引行、`experiments-history.md`；没改 `replay.sh`（派发提示写明不做）。所以没有复跑登记。本段 13 份产物的复跑命令就是第二节那条脚本（`run_seg4_v2.sh`）。
- 没碰 `crates/`；没改登记（产物跑过之后不改登记；第十节的事交主 agent）。
- 没判任何候选出局或胜出；没跑重型测试、没跑门禁全量、没提交。
- 没做变异（这一段的装置变异已由装置执行员做过；M7、M9 没在这一段单跑）。
- 第二段的量（实七、实八、H-随、H-随全、PC-随）不在这一段。

## 十三、草稿目录（`/tmp/claude-1000/e158-r4-seg4/`）

没有建编译目录、工作树或仓副本；只有脚本、日志、表和本报告：`run_seg4_v2.sh`、`run_seg4.log`、`tables.sh`、`tables.out`、`q5-table.txt`、`rerun-today.out`（确定性核对用的，与入库产物逐字节相同，不入库）、`gate-*.log`、`report.md`。臂副本在 `/tmp/claude-1000/e158-r4-device/arms/`，那不是我建的，我没动。
