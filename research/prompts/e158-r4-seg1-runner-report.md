# E158 第 4 次跑 第一段 执行员报告（2026-09-27 JST）

## 一、结论

- 十二臂的 `r4-seg1` 都跑完了，`r4-compare` 也跑了。13 份产物都在 `research/results/`，开头第一行都是 `E7INPUT name=crates_snapshot key=E158 sha256=bb5ca9bef0b09eb957906317adf7d2b47567d901ef341e85c14b6fe23a37cf4b`，最后一行都是 `name=done`。没有 `name=stop` 行。开跑检查（`r4_local_constant_check`、`r4_arm_code_check`、`r4_section_seven_two_anchor`）每臂 52 或 53 条，全部 `verdict=pass`，没有触发 V3。停机条款 S0–S4 在这一段都没打中：各族 `s4_mismatches=0`，7.3 第二至四行都是 `pass`。
- 丢写（H1d、H1e，G0）：
  - 今天：304 / 64 格。
  - 甲-槽：152 / 32 格。
  - 乙-槽：288 / 64 格。
  - 乙-窄读-槽：288 / 64 格。
  - -配置 八臂（甲、乙、乙-窄读、丙 各两条）：全部为 0。登记第六节开头写明，-配置 臂在 H1d、H1e 上的 0 不算判据。
- 阳性对照里有三处没按构造出来，都是 [臂] 句（V1 ④）：
  1. **PC-多读 (ii)，-槽 三臂。** 登记预言 +24，量到 0（24 格 fail）。这是装置报告第二节第 1 条预先写过的，主 agent 认定第 1 项已定：走 F18，这三臂的 Q5-同成 作废。compare 已照此打出作废。
  2. **PC-N-盖，乙-窄读-配置 与 乙-窄读-配置续。** `reads_zeros`、T1 格上 R 轮之后照样拒（K3），三句 [臂] 都是 fail。这一处主 agent 认定第 2 项已定为「照量」。按 V1 ④，这两臂的 Q1–Q3 作废，要走 F16 的三步。
  3. **新发现，事先没有预言：PC-多读 (iii)「乙-X 第一遍的读 = 甲-X 整次」。** 六条乙臂在 PC-N-环 上都差 1：-配置 臂 1573 对 1574，-槽 臂 1569 对 1570（10 句 fail）。按 V1 ④，这六臂的 Q5-拒 作废（Q5-拒 是附带量）；Q5-同成 不受影响。这一处要走 F18 的三步，交主 agent。成因我是读代码推的，没有量过，见第五节。
- 这一段让岔路单第 1 行够判的只有一处：今天、甲-槽、乙-槽、乙-窄读-槽 在 H1d / H1e 上丢写 > 0，后面的量翻不回 0。-配置 各臂的丢写、多拒、第 22 条那一格都还差（岔路表在第八节）。
- 推翻条件：
  - 拿同一批副本重跑 `r4-seg1`，产物不逐字节相同（这是确定性装置，同一个二进制跑 N 遍应当一样）；
  - 或者对全部段一起跑 `r4-compare`，作废范围与本段不同。第二种不算推翻，是作废范围扩大，照新的来。

## 二、跑了什么、产物

- 负载：开跑前 `ps` 看到别的会话在跑 `gate.sh` 与 `cargo test`、`cargo build`，没有 qemu / vm-bench / e152 / fio。内存包装 `--status` 显示 slice 上限 40.1 GiB，账上有 2 条。各臂都没有排队等待，也没有撞内存顶，退出码都是 0，stderr 都是空的。
- 命令：每一臂在各自的 `/tmp/claude-1000/e158-r4-device/arms/<臂>/` 下执行。环境变量与派发提示给的相同，经 `capped.sh 3 … run-with-memory-cap.sh 10G ./target/release/e158_root_choice_repair r4-seg1` 跑；用 `xargs -P 3` 同时跑三臂（装置是单线程，grep 里没有 `thread::` / `rayon`）。compare 在仓根执行：`… run-with-memory-cap.sh 10G /tmp/claude-1000/e158-r4-device/arms/today/target/release/e158_root_choice_repair r4-compare research/results/e158-root-choice-repair-2026-09-27-r4-seg1`，退出码 0。
- 时长：10:22:27 JST 开跑，10:23:42 JST 全部跑完。单臂 1–39 秒，逐臂的数在 `/tmp/claude-1000/e158-r4-seg1/progress.md`。这一段是 H1d 全部断点，之前没有实测过，现在量到了：乙-槽 这一臂最长，39 秒。
- 产物（行数 / sha256 前 16 位）：

| 文件（`research/results/e158-root-choice-repair-2026-09-27-r4-seg1-` 后缀） | 行 | sha256 |
|---|---|---|
| today.out | 1903 | a6da12a19b76a468 |
| jia-cfg.out | 271 | db681aac3e8dbdab |
| jia-cfg-carry.out | 271 | 0a579567ba64406d |
| jia-slot.out | 1087 | 79000c3af5c7a974 |
| yi-cfg.out | 2098 | e6a2c9a64237848f |
| yi-cfg-carry.out | 2098 | 3fdda6618376a88d |
| yi-slot.out | 2866 | 51f0ed37c90df1bc |
| yi-narrow-cfg.out | 1666 | 273307e2a385b86d |
| yi-narrow-cfg-carry.out | 1666 | c2c1654ba149b1f0 |
| yi-narrow-slot.out | 2866 | bc1043b9079913e5 |
| bing-cfg.out | 1135 | 63cffa62cab43805 |
| bing-cfg-carry.out | 1135 | 6ba7321b7899ceba |
| compare.out | 469 | ba321a3e7ce81513 |

- 单测：`grep -c '#\[test\]' crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 数出 82。变异：这一段没有跑。抓到 / 无效 / 没红的数取自装置报告 `research/prompts/e158-r4-device-runner-report.md` 第三、八节：装置变异 11 / 0 / 0，臂表变异 8 / 0 / 0。这些都是它在同一份 bin（sha256 `37a8a312…445b`）上跑出来的；我核过，主工作区的 bin 与四份臂副本里的源码 `cmp` 结果相同。

## 三、齐不齐（第 4b 步）

命令（awk 脚本在草稿目录 `completeness2.awk`，对每条 `r4_cell` 核 `q0_destination q2_rollback overwrite_cell q3_read_back lost_write_cell form duration b n1 c` 在不在，另数 `a_class` 与 `a_outcome` 都没有的格、`truth_newer=true` 的格）：
`cat research/results/e158-root-choice-repair-2026-09-27-r4-seg1-{today,jia-cfg,…,bing-cfg-carry}.out | awk -f /tmp/claude-1000/e158-r4-seg1/completeness2.awk | sort`，原样输出（只取汇总行）：

```text
bing-cfg-carry h1d cells=912 missing_field_cells=0 truth_newer_cells=48 unintercepted_nonempty=0 no_class_no_outcome=0
bing-cfg-carry h1e cells=64 missing_field_cells=0 truth_newer_cells=64 unintercepted_nonempty=0 no_class_no_outcome=0
bing-cfg h1d cells=912 missing_field_cells=0 truth_newer_cells=48 unintercepted_nonempty=0 no_class_no_outcome=0
bing-cfg h1e cells=64 missing_field_cells=0 truth_newer_cells=64 unintercepted_nonempty=0 no_class_no_outcome=0
jia-cfg-carry h1d cells=48 missing_field_cells=0 truth_newer_cells=48 unintercepted_nonempty=0 no_class_no_outcome=0
jia-cfg-carry h1e cells=64 missing_field_cells=0 truth_newer_cells=64 unintercepted_nonempty=0 no_class_no_outcome=0
jia-cfg h1d cells=48 missing_field_cells=0 truth_newer_cells=48 unintercepted_nonempty=0 no_class_no_outcome=0
jia-cfg h1e cells=64 missing_field_cells=0 truth_newer_cells=64 unintercepted_nonempty=0 no_class_no_outcome=0
jia-slot h1d cells=864 missing_field_cells=0 truth_newer_cells=48 unintercepted_nonempty=0 no_class_no_outcome=0
jia-slot h1e cells=64 missing_field_cells=0 truth_newer_cells=64 unintercepted_nonempty=0 no_class_no_outcome=0
today h1d cells=1680 missing_field_cells=0 truth_newer_cells=48 unintercepted_nonempty=0 no_class_no_outcome=0
today h1e cells=64 missing_field_cells=0 truth_newer_cells=64 unintercepted_nonempty=0 no_class_no_outcome=0
yi-cfg-carry h1d cells=1792 missing_field_cells=0 truth_newer_cells=64 unintercepted_nonempty=0 no_class_no_outcome=0
yi-cfg-carry h1e cells=128 missing_field_cells=0 truth_newer_cells=128 unintercepted_nonempty=0 no_class_no_outcome=0
yi-cfg h1d cells=1792 missing_field_cells=0 truth_newer_cells=64 unintercepted_nonempty=0 no_class_no_outcome=0
yi-cfg h1e cells=128 missing_field_cells=0 truth_newer_cells=128 unintercepted_nonempty=0 no_class_no_outcome=0
yi-narrow-cfg-carry h1d cells=1360 missing_field_cells=0 truth_newer_cells=64 unintercepted_nonempty=0 no_class_no_outcome=0
yi-narrow-cfg-carry h1e cells=128 missing_field_cells=0 truth_newer_cells=128 unintercepted_nonempty=0 no_class_no_outcome=0
yi-narrow-cfg h1d cells=1360 missing_field_cells=0 truth_newer_cells=64 unintercepted_nonempty=0 no_class_no_outcome=0
yi-narrow-cfg h1e cells=128 missing_field_cells=0 truth_newer_cells=128 unintercepted_nonempty=0 no_class_no_outcome=0
yi-narrow-slot h1d cells=2560 missing_field_cells=0 truth_newer_cells=64 unintercepted_nonempty=0 no_class_no_outcome=0
yi-narrow-slot h1e cells=128 missing_field_cells=0 truth_newer_cells=128 unintercepted_nonempty=0 no_class_no_outcome=0
yi-slot h1d cells=2560 missing_field_cells=0 truth_newer_cells=64 unintercepted_nonempty=0 no_class_no_outcome=0
yi-slot h1e cells=128 missing_field_cells=0 truth_newer_cells=128 unintercepted_nonempty=0 no_class_no_outcome=0
```

- 断点格（`cut≠none`）没有 `truth_newer` 字段，这是装置的写法：N_真 只在不断的那一格上算。我第一版 awk 把它算成缺字段，改成上面的判法之后是 0。每臂每族都造出了被测形状（`truth_newer_cells` > 0）。V2 的 `void_cells_v2` 与 `fault_not_reached_cells` 在 24 行 `r4_family_summary` 上全是 0。
- 阳性对照逐臂的条数（每臂一行，按 control 与 verdict 数）：每一臂 PC-N-环 4、PC-N-盖 4、PC-O 1、PC-丢写 1、PC-N0 8，与 7.2 的锚点（`r4_section_seven_two_anchor`，每臂 17 条 pass）相符。PC-只读 的条数：今天 4 条都是 `not_applicable`（今天不拒）；-槽 三臂各 2 条 pass、2 条 `not_applicable`（`reads_zeros` 上 N-槽 判假，不拒）；其余八臂各 4 条 pass。
- 跨臂的起始镜像：`r4_base_identity` 十二臂都是 `prefixes=8 differing=0 verdict=pass`。

## 四、判决行（第 4c 步）

`grep -n 'name=verdict' <产物>` 在 13 份里都是 0 行。这台装置不打 `name=verdict` 行，判决写在各行的 `verdict=` 字段和 `sentence_*=` 字段里，下面按这两种字段点名。`grep -o 'verdict=[a-z_]*' … | sort | uniq -c` 的结果：各臂产物里只有 `pass`，另有 `not_applicable`（今天 4 条，三条 -槽 臂各 2 条，都是 PC-只读 那一臂没拒）和 `fail`（乙-窄读 两臂各 1 条）；compare 里是 `pass` 255、`fail` 34、`not_constructible` 12、`inference_holds` 1。名字里带 mismatch / failures / findings / differing / differences / violations / ambiguous / not_reached / void 的整数计数共 813 个，大于 0 的一个都没有。

没过的，逐个点名：

| 产物:行 | 字段 = 取值 | 为什么 | 登记预期内？ |
|---|---|---|---|
| `…-r4-seg1-yi-narrow-cfg.out:71`、`…-yi-narrow-cfg-carry.out:71` | PC-N-盖 `form=reads_zeros duration=T1 verdict=fail`：`sentence_arm_ok_after_one_round_chosen_hidden_no_rollback=fail`、`sentence_arm_row_publish_txg_is_hidden_txg_plus_one=fail`、`sentence_arm_destination_chosen_or_applied=fail`，`missed_routes=F16,F16,F16` | 被藏那次发布的记录读回全零，窄读把它们当 journal 空槽收进缓存；E 那次发布的末条计数器因此找不到，N-配置 判不出、按真处置，R 轮之后拒（`a_class=K3`、`a_arm_observation=…,rounds:1,…`、`a_arm_caches_zeroed_journal_slots=true`） | 是：装置报告第二节第 2 条、登记修订第 11 条已经写过，主 agent 认定第 2 项「照量」 |
| compare:123–130、281–288、313–320 | PC-多读 (ii) `point_value[n1=… c=…] tag=arm verdict=fail measured_extra_read_calls=Some(0)expected=Some(24)`（甲-槽、乙-窄读-槽、乙-槽 各 8 格） | -槽 臂的读阶段按同一槽序每槽读一次，替换掉择根那一遍，读数与今天相同，所以是 +0 | 是：修订第 10 条、主 agent 认定第 1 项（走 F18，Q5 作废） |
| compare:152、155、185、188、218、221、251、254 | PC-多读 (iii) `first_pass_reads_equal_jia_total[control=PC-N-ring form=read_fails / reads_zeros] verdict=fail before=Some(1573)jia_total=Some(1574)`（乙-配置、乙-配置续、乙-窄读-配置、乙-窄读-配置续） | 见第五节，推的 | **否**：事先没有预言 |
| compare:289、321 | 同上，`form=read_fails before=Some(1569)jia_total=Some(1570)`（乙-窄读-槽、乙-槽） | 同上 | **否** |
| compare:327–337 | PC-多拒 `today_ok_on_the_cell tag=today verdict=not_constructible n1=1,2,3`（十一臂各一行） | 第一段的产物里没有 L 族，compare 在 L2 / L3 上找不到今天那一格 | 是（段落的形态造成的）：PC-多拒 归第四段；[今] 句，V1 ⑤ 不作废。五段一起跑 compare 时要重看 |
| compare:338 | PC-22 `an_m_with_ok_zero_count_zero_isolation tag=today verdict=not_constructible route=F2` | 同上，第一段没有 L5 | 同上，归第四段 |
| compare:415、457、464 | `r4_void … quantity=Q5-both-mounted voided=true`（甲-槽、乙-窄读-槽、乙-槽） | 上面 (ii) 那几格 | 是 |
| compare:430、437、444、451、458、465 | `r4_void … quantity=Q5-refused voided=true`（六条乙臂） | 上面 (iii) 那几格 | 否（跟着新发现来的） |
| compare:440、447 | `r4_void … quantity=Q1-Q3 voided=true`（乙-窄读-配置、乙-窄读-配置续） | 上面 PC-N-盖 那一格 | 是（认定第 2 项之下 V1 ④ 的字面结果） |

`compare:347 name=r4_f11 cells=48 cells_differing_outside_o1_and_cell_22=0 verdict=inference_holds` 不算没过。

## 五、三样的读数（整行抄自产物）

丢写（`grep -n 'name=r4_loss' research/results/e158-root-choice-repair-2026-09-27-r4-seg1-compare.out`，24 行原样；lost_write_cells = Q1 ∪ Q2 ∪ Q3 按格去重）：

```text
360:E7RESULT name=r4_loss arm=bing-cfg family=h1d geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=912 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=912
361:E7RESULT name=r4_loss arm=bing-cfg family=h1e geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=64 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=64
362:E7RESULT name=r4_loss arm=bing-cfg-carry family=h1d geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=912 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=912
363:E7RESULT name=r4_loss arm=bing-cfg-carry family=h1e geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=64 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=64
364:E7RESULT name=r4_loss arm=jia-cfg family=h1d geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=48 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=48
365:E7RESULT name=r4_loss arm=jia-cfg family=h1e geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=64 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=64
366:E7RESULT name=r4_loss arm=jia-cfg-carry family=h1d geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=48 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=48
367:E7RESULT name=r4_loss arm=jia-cfg-carry family=h1e geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=64 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=64
368:E7RESULT name=r4_loss arm=jia-slot family=h1d geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=864 lost_write_cells=152 q1_overwrite_cells=134 q2_rollback_cells=24 q3_lost_cells=144 q3c_lost_cells=4 q3_last_confirmed_cells=720
369:E7RESULT name=r4_loss arm=jia-slot family=h1e geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=64 lost_write_cells=32 q1_overwrite_cells=32 q2_rollback_cells=32 q3_lost_cells=0 q3c_lost_cells=16 q3_last_confirmed_cells=64
370:E7RESULT name=r4_loss arm=today family=h1d geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=1680 lost_write_cells=304 q1_overwrite_cells=268 q2_rollback_cells=48 q3_lost_cells=288 q3c_lost_cells=8 q3_last_confirmed_cells=1392
371:E7RESULT name=r4_loss arm=today family=h1e geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=64 lost_write_cells=64 q1_overwrite_cells=64 q2_rollback_cells=64 q3_lost_cells=0 q3c_lost_cells=32 q3_last_confirmed_cells=64
372:E7RESULT name=r4_loss arm=yi-cfg family=h1d geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=1792 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=1792
373:E7RESULT name=r4_loss arm=yi-cfg family=h1e geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=128 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=128
374:E7RESULT name=r4_loss arm=yi-cfg-carry family=h1d geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=1792 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=1792
375:E7RESULT name=r4_loss arm=yi-cfg-carry family=h1e geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=128 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=128
376:E7RESULT name=r4_loss arm=yi-narrow-cfg family=h1d geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=1360 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=1360
377:E7RESULT name=r4_loss arm=yi-narrow-cfg family=h1e geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=128 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=128
378:E7RESULT name=r4_loss arm=yi-narrow-cfg-carry family=h1d geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=1360 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=1360
379:E7RESULT name=r4_loss arm=yi-narrow-cfg-carry family=h1e geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=128 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=128
380:E7RESULT name=r4_loss arm=yi-narrow-slot family=h1d geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=2560 lost_write_cells=288 q1_overwrite_cells=264 q2_rollback_cells=32 q3_lost_cells=272 q3c_lost_cells=5 q3_last_confirmed_cells=2288
381:E7RESULT name=r4_loss arm=yi-narrow-slot family=h1e geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=128 lost_write_cells=64 q1_overwrite_cells=64 q2_rollback_cells=64 q3_lost_cells=0 q3c_lost_cells=32 q3_last_confirmed_cells=128
382:E7RESULT name=r4_loss arm=yi-slot family=h1d geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=2560 lost_write_cells=288 q1_overwrite_cells=264 q2_rollback_cells=32 q3_lost_cells=272 q3c_lost_cells=5 q3_last_confirmed_cells=2288
383:E7RESULT name=r4_loss arm=yi-slot family=h1e geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=128 lost_write_cells=64 q1_overwrite_cells=64 q2_rollback_cells=64 q3_lost_cells=0 q3c_lost_cells=32 q3_last_confirmed_cells=128
```

每一臂在这两族上算不算判据（登记第六节开头）：
- 今天、甲-槽、乙-槽、乙-窄读-槽：算。四臂在 H1d、H1e 上都 > 0。-槽 三臂的丢写只落在读回全零那一种造法上。命令：`grep 'name=r4_cell ' <臂产物> | grep 'lost_write_cell=true' | grep -o 'family=[a-z0-9]* \|form=[a-z_]*' | paste -d' ' - - | sort | uniq -c`，结果如下：甲-槽 h1d 152、h1e 32，乙-槽 与 乙-窄读-槽 h1d 288、h1e 64，全部是 `form=reads_zeros`；今天两种造法各占一半（h1d 152 + 152，h1e 32 + 32）。
- 甲-配置、甲-配置续、乙-配置、乙-配置续、丙-配置、丙-配置续：不算，只核 F6。
- 乙-窄读-配置、乙-窄读-配置续：登记第六节没有点名这两臂（它们是后来另立的），但判据同样是 N-配置，按同一个理由也不算（这是我推的）。况且这两臂的 Q1–Q3 已经按 V1 ④ 作废。

Q0 被藏那条根的去向（`r4_family_summary` 行的 `q0_destinations` 字段，命令：`grep 'name=r4_family_summary' <臂产物> | sed -E 's/.*family=([a-z0-9]+) .* q0_destinations=(\{[^}]*\}).*/\1 \2/'`）：
- 今天 h1d `{"in_ring_and_abandoned": 32, "left_as_it_was": 1632, "root_slot_overwritten": 16}`，h1e `{"in_ring_and_abandoned": 32, "root_slot_overwritten": 32}`。今天那一臂「环在且被抛弃」∪「根槽被盖」= 48 / 64 > 0，所以 F8 不触发。
- -配置 各臂只出现 `left_as_it_was` 与 `chosen_or_applied`：乙-配置 h1d 32 / 1760，h1e 64 / 64；丙 h1d 16 / 896。
- -槽 各臂四类都有，例如乙-槽 h1d `{"chosen_or_applied": 16, "in_ring_and_abandoned": 20, "left_as_it_was": 2512, "root_slot_overwritten": 12}`。

F 条款（只列这一段查得到的）：
- 今天 h1d 的 `r4_family_summary`：`f4_failures=0 f6_checked=48 f6_findings=0 f13_checked=48 f13_failures=0 f15_checked=8 f15_findings=0`；h1e：`f6_checked=16 f6_findings=0 f13_checked=16 f13_failures=0 f15_checked=8 f15_findings=0`。F4、F6、F13、F15 都不触发。
- F17：PC-N-环 今天四格的 `sentence_today_hidden_in_ring_and_abandoned=pass`，`q0_destination=in_ring_and_abandoned`，不触发。PC-N-盖 今天四格的 [今] 句都是 pass（`row_publish_txg=4 row_publish_slot=1:1 q0_destination=root_slot_overwritten`），F15 不触发。
- F16（乙-配置、乙-配置续）：H1d 不断格与 H1e m=1 格上，T1 与 O(1 只) 两种造法全部 `K0` 且所选根 = 被藏那条根（各 8 格），T2 与 W 是 K3。不触发。
- 乙-窄读 两臂：`reads_zeros` 的 T1 / O(1 只) 上，b=records 的 4 格 K3，b=unit_first_copy 的 4 格 K0。登记的 F16 没有点名窄读臂；这就是认定第 2 项说的那个差别。
- 命令：`grep 'name=r4_cell ' <臂产物> | awk …`（H1d 取 `cut=none`、H1e 取 `m=1`），按 family / form / duration / b / a_class / 所选根是否 = hidden 分组，原样输出在 `/tmp/claude-1000/e158-r4-seg1/f16-tally.txt`（144 行）。在里面对乙-配置、乙-配置续数 T1 与 O(1 只)：`K0 chosen=hidden` 的行 24 行，别的 0 行。

多读（compare 里的 `r4_q5` 行，34 行，只抄同成栏；拒 那一栏在 compare 第 `grep -n refused_minus_today` 的行里）：

```text
17:E7RESULT name=r4_q5 arm=bing-cfg column=both_mounted family=h1d@GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=16 positive_cells=16 total_extra_read_calls=24772 total_extra_read_bytes=109838336 minimum_extra_read_calls=1540 maximum_extra_read_calls=1573
42:E7RESULT name=r4_q5 arm=bing-cfg-carry column=both_mounted family=h1d@GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=16 positive_cells=16 total_extra_read_calls=24836 total_extra_read_bytes=110100480 minimum_extra_read_calls=1544 maximum_extra_read_calls=1577
107:E7RESULT name=r4_q5 arm=jia-slot column=both_mounted family=h1d@GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=24 positive_cells=0 total_extra_read_calls=0 total_extra_read_bytes=0 minimum_extra_read_calls=0 maximum_extra_read_calls=0
108:E7RESULT name=r4_q5 arm=jia-slot column=both_mounted family=h1e@GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=8 positive_cells=0 total_extra_read_calls=0 total_extra_read_bytes=0 minimum_extra_read_calls=0 maximum_extra_read_calls=0
133:E7RESULT name=r4_q5 arm=yi-cfg column=both_mounted family=h1d@GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 positive_cells=32 total_extra_read_calls=50136 total_extra_read_bytes=207872000 minimum_extra_read_calls=1488 maximum_extra_read_calls=1639
166:E7RESULT name=r4_q5 arm=yi-cfg-carry column=both_mounted family=h1d@GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 positive_cells=32 total_extra_read_calls=50264 total_extra_read_bytes=208396288 minimum_extra_read_calls=1492 maximum_extra_read_calls=1643
199:E7RESULT name=r4_q5 arm=yi-narrow-cfg column=both_mounted family=h1d@GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=24 positive_cells=23 total_extra_read_calls=942 total_extra_read_bytes=5744640 minimum_extra_read_calls=-38 maximum_extra_read_calls=111
232:E7RESULT name=r4_q5 arm=yi-narrow-cfg-carry column=both_mounted family=h1d@GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=24 positive_cells=23 total_extra_read_calls=1038 total_extra_read_bytes=6137856 minimum_extra_read_calls=-34 maximum_extra_read_calls=115
265:E7RESULT name=r4_q5 arm=yi-narrow-slot column=both_mounted family=h1d@GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=40 positive_cells=15 total_extra_read_calls=496 total_extra_read_bytes=3289088 minimum_extra_read_calls=-46 maximum_extra_read_calls=103
266:E7RESULT name=r4_q5 arm=yi-narrow-slot column=both_mounted family=h1e@GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=8 positive_cells=0 total_extra_read_calls=-4 total_extra_read_bytes=-16384 minimum_extra_read_calls=-1 maximum_extra_read_calls=0
297:E7RESULT name=r4_q5 arm=yi-slot column=both_mounted family=h1d@GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=40 positive_cells=16 total_extra_read_calls=24928 total_extra_read_bytes=103362560 minimum_extra_read_calls=-1 maximum_extra_read_calls=1631
298:E7RESULT name=r4_q5 arm=yi-slot column=both_mounted family=h1e@GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=8 positive_cells=0 total_extra_read_calls=-4 total_extra_read_bytes=-16384 minimum_extra_read_calls=-1 maximum_extra_read_calls=0
```

PC-多读 (ii)（L0 八格上 Q5-同成 的点值，compare 里的 `r4_cross_arm_sentence … control=PC-extra-reads-ii`，按臂数）：甲-配置、乙-配置、乙-窄读-配置 +4；三条「续」+8；丙-配置 −21，丙-配置续 −17，与 `expected` 相等（p = 1 片），这 64 格都是 pass；-槽 三臂量到 0、登记预言 24，这 24 格 fail。PC-多读 (i) 今天减自己：`compare:14 … row=4 arm=today cells=64 nonzero_cells=0 verdict=pass`。

多拒：H1d、H1e 上 N_真 为真，这里只有「拦下」，没有多拒。`r4_q4` 22 行都是 `extra_refusals=0`，拦下按时长分：甲-配置 h1d `intercepted="O=16,T=16,W=16"`，乙-配置 `"W=16"`，乙-窄读-配置 `"O=4,T=4,W=16"`，丙-配置 `"T=16,W=16"`，甲-槽 `"O=8,T=8,W=8"`，乙-槽 `"W=8"`（整行在 compare 的 `name=r4_q4` 行）。PC-N0：十一臂各 8 格 `same_as_today` pass（compare 共 88 句）。

7.3：compare 第 339–346 行，第二行（`device_n_cfg`↔甲-配置、`device_n_slot`↔甲-槽）各 `cells=64 mismatches=0 verdict=pass`；第三行今天、甲 三臂、丙 两臂各 `cells=16 differences=0 verdict=pass`；第四行见上。第一行：每份 `r4_family_summary` 与 `r4_cross_checks` 的 `s4_mismatches=0`。

### PC-多读 (iii) 差 1 的成因（读代码推的，没量过）

乙 的读阶段经 `R3ReadCache` 读盘（副本 `arms/yi-cfg/crates/singlefs-core/src/mount.rs` 第 3999 行起的 `r3_reread_then_refuse`，第一轮也经 cache）。甲 在第 3936 行起的 `RefuseWritable` 那一支直接用 `reader`。第一遍里如果有同一个落点被读了两次，而且那个落点读得出、不是全零，乙 第二次就从缓存拿，不打到盘上，于是比甲少 1。能对上这个推断的读数：差值只出现在 PC-N-环（b = `unit_first_copy`），PC-N-盖（b = `records`）上相等（-配置 1572 = 1572，-槽 1568 = 1568，compare 第 158、161、191、194、224、227、257、260、292、324 行，共 10 句 pass）；两种造法差的都是 1。登记 5.4 那一句 [臂] 的前提「两臂第一遍同一串读」，由乙 的定义（缓存「这次挂载内有效」）推不出来。这属于「做完会怎样推不出」那一类，要走 F18 的三步（记一次输、只许收严、写明收严在哪）。这不归执行员改，交主 agent。要证实这个推断，需要在副本里记下第一遍每次读的落点，这一段没有做。

## 六、V1 作废范围（compare 的 `r4_void`，84 行，其中 `voided=true` 11 行）

- Q5-同成：甲-槽、乙-槽、乙-窄读-槽 作废（PC-多读 (ii)）。
- Q5-拒：乙-配置、乙-配置续、乙-槽、乙-窄读-配置、乙-窄读-配置续、乙-窄读-槽 作废（PC-多读 (iii)）。
- Q1–Q3：乙-窄读-配置、乙-窄读-配置续 作废（PC-N-盖）。
- 其余 73 行 `voided=false`。今天、甲-配置、甲-配置续、乙-配置、乙-配置续、丙 两臂的 Q1–Q3 与 Q5-同成 都没有作废。

这张作废表只按第一段的对照算。PC-多拒、PC-22、PC-554、PC-随 都不在这一段，所以 Q4、Q6、H-随 那一栏的 `voided=false` 不代表它们的对照已经过了。五段齐了之后要对全部前缀一起跑一次 `r4-compare`，拿那一次的作废表算数。

## 七、登记修订

没有。产物跑完之前没有发现要补的；产物跑完之后按定义不改登记。第五节 PC-多读 (iii) 那一处交主 agent。

## 八、岔路表（岔路单 `research/prompts/c554-fix-forks.md` 第 1 行，逐样）

| 那一样 | 状态 | 还差什么 | 剩下的量能不能让它翻面 | 算它的命令 |
|---|---|---|---|---|
| 丢写：今天 | 已够判：> 0（H1d 304、H1e 64） | — | 不能：丢写格只会增加 | `grep -n 'name=r4_loss' …-r4-seg1-compare.out` |
| 丢写：甲-槽、乙-槽、乙-窄读-槽 | 已够判：> 0（甲-槽 152 / 32；乙-槽、乙-窄读-槽 各 288 / 64）；这三臂的 Q1–Q3 没有作废 | — | 不能 | 同上 |
| 丢写：甲-配置、甲-配置续、乙-配置、乙-配置续、丙-配置、丙-配置续 | 未够判：H1d、H1e 上都是 0，但按第六节开头不算判据 | 实七-甲、实七-乙、实八、H-随（第二段，装置还没写）；H1f、H1g（第三段） | 能：后面任一族 > 0 就出局 | 同上；后面看各段 compare 的 `r4_loss` |
| 丢写：乙-窄读-配置、乙-窄读-配置续 | 未够判：这两族上是 0，但 Q1–Q3 已按 V1 ④ 作废（PC-N-盖 [臂] 句），F16 的三步还没有走 | 主 agent 定 F16 三步怎么落；然后同上一行，还差第二、三段 | 能 | compare:440、447 |
| 多拒（Q4） | 未够判：H1d、H1e 上各臂 `extra_refusals=0`（N_真 为真，只有拦下）；PC-N0 十一臂 88 格与今天逐项相同 | L0–L6、PC-多拒（第四段），H-随全（第二段） | 能 | compare `name=r4_q4` |
| 多读（Q5-同成） | 未够判：L0 点值 -配置 臂 +4 / +8，丙 −21 / −17；H1d 不断格的同成栏见第五节；-槽 三臂按 F18 作废 | L 族（第四段）、实七 / 实八 / H-随（第二段）；要等两个候选都不丢写，才比到这一样 | 能 | compare `name=r4_q5 … column=both_mounted` 与 `control=PC-extra-reads-ii` |
| 多读（Q5-拒，附带） | 六条乙臂作废（第五节新发现），其余有数 | 不进判定，够判后不跑 | — | compare `column=refused_minus_today` |
| 第 22 条那一格（Q6） | 未够判：这一段没有 L5 | 第四段 L5、PC-22 | 能 | — |
| 够判条件第二半 | 已够判（登记修订第 3 条：S0 在快照上逐条核过） | — | 不能 | 登记第十二节修订第 3 条里的行号 |

每一格剩下的量能不能让它翻面，都是按登记第六节「翻面取值」那一栏读的。出不出局，由主 agent 按岔路单判。

## 九、门禁（登记给 experiment-runner 的阶段；这一段不写实验页，读页的那几道放在最后跑）

原样日志在草稿目录 `gate-<阶段>.log`。

| 阶段 | 退出码 | 末行（原样，截到 200 字）/ 与这一段的关系 |
|---|---|---|
| 33 | 0 | `✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 1132 条的原文各命中源码一次；…` |
| 52 | 1 | `再让 research/scripts/replay.sh 里 E142 那一行的入库产物名字跟上。` 与 E158 无关（日志里 0 处 E158） |
| 80 | 0 | 末行列文件名 `crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs` |
| 96 | 0 | 末行列文件名 `e92_reuse_requirement.rs` |
| 27 | 1 | `→ 旧值只许留在「## 历史版本」之后与 *-history.md 里。` 日志里 0 处 E158 |
| 34 | 0 | `✓ 实验索引行与正文标题一致（索引 159 行、正文 159 份）` |
| 40 | 1 | 点名这一段的 13 份新产物还没被实验页点名（日志第 13–25 行），另有 r2 / r3 的旧产物、并行几段的产物、E160 等。**这一段按派发不写页；这 13 个文件名交给五段齐之后写页的那个执行员去点名** |
| 69 | 1 | 日志里 0 处 E158。装置报告第九节里 69 号点名的那条 E158 的红，这次没有再出现；清掉它的是这一段的产物，还是并行几段的产物，没有分开查 |
| 75 | 1 | 日志里 0 处 E158 |
| 84 | 0 | `✓ 登记产物里判决字段是 false 的都在对应实验页里点了名（点名 3 个）` |
| 85 | 0 | `写明「原始输出未留存」不判的 0 页：（没有）` |
| 86 | 0 | `扫了 research crates/singlefs-harness/src/bin（target/ 不扫）；不在的目录 0 个：（没有）` |
| 88 | 77 | 本次未跑：`! 这次改动新增或改写的 kb 正文里没有整行抄的 E7RESULT 行，本阶段无对象可判` |
| 99 | 0 | `✓ 登记了路径与共用项的实验页判过了（3 份、14 条路径）；…` |

## 十、草稿目录与没做什么

- 草稿目录 `/tmp/claude-1000/e158-r4-seg1/` 里只有日志、计数脚本（`completeness*.awk`）、计数输出、`f16-tally.txt`、`progress.md`、`rc/` 与这份报告，没有编译目录，也没有仓副本。臂副本 `/tmp/claude-1000/e158-r4-device/arms/` 是装置执行员建的，另外几段还在用，我没有动。
- 没做：
  - 没写实验页、索引行与 `experiments-history.md`，没改 `replay.sh`，没碰 `crates/mutations.tsv`（派发这样定的）。所以 40 号点名的 13 份新产物还没被页引用。
  - 没跑变异，没跑单测：装置执行员在同一份 bin 上跑过，数见第二节。
  - 没判任何候选出局或胜出。
  - 没去证实第五节那个差 1 的推断。
  - 没跑重型测试，没提交。
  - 没跑跨段的 `r4-compare`，要等五段齐了再跑。
