# E142 第十九次跑续做（第三段）执行员报告

日期 2026-09-28。这一段做了登记步 ①（在今天的 `crates/` 上重做）、③、④；第二段（R19C-8，层 0 主臂枚举）是重型，没跑。编译、单测、两条臂的产物与变异整表都在第二台跑（主 agent 追加输入，派发中途收到：本机那条单测包装在内存队里排了约 15 分钟，按指示停掉，包装退 143、命令一行没跑，不算结果）。

## 结论

- 问题单三行都够判（岔路表在末尾）。
- 第 1 行：step_five 钉的 20 格里 16 格字面相同，4 格不同（暖机、整条流的 `operations=` 与 `kinds=`），差得恰如登记附表 M 的「R40 分叉」：模型照 R40 把连着的两道屏障都记，`crates/singlefs-harness/src/lib.rs` 第 179 行 `fn push` 把同一块盘的第二道并掉。这四格模型都等于 7.4 锚点 ⇒ 登记的停机 S19-crates（模型、锚点都不改，交主 agent）。三态 16777260 与取三态种类与 `crates/` 钉的相同；锚点 145 格全等；13 行取样点全等。
- 第 2 行：六个字段在第十八次、臂 N18、N19C 三列逐字相同（四个 `true`，`control_outcome_matrix_ok=false`、`g7_states_ok=false`，都与第十八次相同）。
- 第 3 行：D_crates 6 组、D_model 34 组全部归因，D_total 192 行全部落在两份分 diff 里（NOT_FOUND 0）；改名那一批带来的行组 0 个。
- 字节：`old-new` `changed=0`；S19-bytes 没触发（`impl_bytes_equal_summary … equal=29 unequal=0`）。
- 推翻条件：`python3 /tmp/claude-1000/e142-r19-runner3/judge/compare_crates.py compare <主产物> N19C <step_five> <crash_segments>` 出现上面四格之外的 `literal_equal=false`；或 `judge.py` 出现 `equal=false`；或 `old-new` 的 `changed` 不为 0。

## 登记修订（写在任何产物之前）

`research/prompts/e142-r19-prereg.md` 第十二节加 R19E-3（9 条）：① 冻结值记成「`41ef20b5…` 经登记改名后的 `5f882741…`」（复核命令与 sha256 写在里面；8 对替换里测试名那一对不在 `term-renames.md` 里，照实记）；② 臂 N18 源码 `0676ed9b…`（git `7a3c8e1e`）经 `forward_rename.py` 得 `f326f72a…`，`reverse_rename.py` 回到 `0676ed9bc6f21fd9a6627eb55543021c63e02d0c5c74d9452c0305d0cd3f7ba2`；③ S19-clause 基线换成 `4d98ee79…`，三类差异逐类列（用例路径 5 对、产物文件名 1 处、D16 已定项 7 射程里删掉的钟点 1 处），步 ③(1) 重抄 = `4d98ee79…`；④ 旧臂 N18 产物认；⑤ 步 ① 在今天的 `crates/` 上重做（快照 A28，V19c 比 A28–B、B–C）；⑥ 路径今天的位置；⑦ 独立 bin `311ce77c…` 只差注释；⑧ R40 不改；⑨ 这一段在第二台跑，附两台工具链逐字相同的原样输出。主产物落盘之后登记一字没改。

## 单测、变异

- 单测（第二台，`run-with-memory-cap.sh 8G … cargo test --release -p e7-index-bench --bin e142-new-pool-file-creation-dry-run`）末行原样：`test result: ok. 94 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.61s`；`grep -c '^test .* ok$'` = 94。
- 变异整表（第二台，今天的源码 `5f882741…` 与表 `0d91aafd…`，`MUTATE_JOBS=2`，`capped.sh 16`），按行首符号数：✅ 180、⏭ 0、❌ 0、💥 0、⚠️ 0、⏱ 0、🧱 0；收尾原样「计数：内存撞顶 0 条（上限 16G）、超时 0 条」「已还原，基线仍全绿」，退出码 0。**抓到 180 / 无效 0 / 没红 0**。日志 `research/results/e142_new_pool_file_creation_dry_run-mutate-2026-09-28-r19.log`（sha256 `bdb9fc6a…`）。没有要分类的。

## 产物（都在 `research/results/`）

| 文件 | sha256 前 12 位 | 说明 |
|---|---|---|
| `e142-new-pool-file-creation-dry-run-2026-09-28-r19-main.out` | 5cdff990e060 | 主产物，772 行，`name=done` 2 行（`emitted=732`、`emitted=39`），头 `E7INPUT … sha256=5ffcf9958648… files=200`，`forced_rerun` 0 行 |
| `e142-new-pool-file-creation-dry-run-2026-09-28-r19-arm-n18.out` | 23ee405fbd5b | 臂 N18，754 行，`name=done` 2 行 |
| `e142-region-old-new-independent-2026-09-28-r19.out` | — | Q142.55 |
| `e142-r19-comparisons-2026-09-28.out` / `e142-r19-controls-2026-09-28.out` | — | 比对甲与锚点逐格 / 控制 |
| `e142-r19-diff-{r18-to-arm-n18,arm-n18-to-main,r18-to-main}-2026-09-28.txt` | — | 28 / 186 / 212 行 |
| `e142-r19-crates-sha256-{a,b,c}-2026-09-28.txt`、`e142-r19-crates-status-2026-09-28.txt` | — | 179 行各一份，`cmp` A28–B、B–C 都退 0 |

两臂准入（第二台那棵树里）stderr 原样：N18「✓ 放行 E142：输入自 research/results/e142-new-pool-file-creation-dry-run-2026-09-26-r18-main.out 以来变了（那一份 f62e18e1695d，今天 c1c620beadad）」；主臂「…（那一份 f62e18e1695d，今天 5ffcf9958648）」。第二台主臂树的指纹与本机当时 `admission.py` 算的 `5ffcf9958648…` 相同。主产物在第二台同一棵树里再跑一次，`cmp` 退 0。

4b 按行类数（命令：`awk '/name=done/{d++} {match($0,/name=[a-z0-9_]+/); …}' <主产物>`，原样节选）：`5 model:segments`、`5 model:segments_three_state`、`13 model:segments_sensitivity`、`1 model:verdict`、`29 model:device_region_bytes`、`29 model:impl_bytes_equal`、`60 model:header311_write_digest`、`4 model:positive_control_p2`、`2 model:window_segments`、`29 crates:device_region_bytes`、`6 crates:window_barrier`、`1 crates:device_region_bytes_summary`。R41 / V19e（13 行）齐。

## 判决行逐行读（4c）

`grep -n name=verdict` 原样（三份产物各 1 条）：

```
== e142-new-pool-file-creation-dry-run-2026-09-26-r18-main-2.out
714:E7RESULT name=verdict width_mismatches=0 write_list_ok=true recover_full_ok=true layer0_states_ok=not_run layer0_violations=not_run control_states_ok=true control_violations_ok=true control_missed_zero=true control_false_alarm_zero=true control_outcome_matrix_ok=false g6_states_ok=true g6_violations_zero=true g7_states_ok=false g7_violations_zero=false positive_control_main_geometry_ok=true role_label_contradictions_zero=true role_labels_all_defined=true text_number_mismatches_zero=true text_numbers_all_classified=true named_instances_ok=true journal_differing_states=not_run r18_system_configuration_fields_ok=true r18_root_flags_ok=true r18_g8_ok=true r18_g9_ok=true
== e142-new-pool-file-creation-dry-run-2026-09-28-r19-arm-n18.out
714:E7RESULT name=verdict width_mismatches=0 write_list_ok=true recover_full_ok=true layer0_states_ok=not_run layer0_violations=not_run control_states_ok=true control_violations_ok=true control_missed_zero=true control_false_alarm_zero=true control_outcome_matrix_ok=false g6_states_ok=true g6_violations_zero=true g7_states_ok=false g7_violations_zero=false positive_control_main_geometry_ok=true role_label_contradictions_zero=true role_labels_all_defined=true text_number_mismatches_zero=true text_numbers_all_classified=true named_instances_ok=true journal_differing_states=not_run r18_system_configuration_fields_ok=true r18_root_flags_ok=true r18_g8_ok=true r18_g9_ok=true
== e142-new-pool-file-creation-dry-run-2026-09-28-r19-main.out
732:E7RESULT name=verdict width_mismatches=0 write_list_ok=true recover_full_ok=true layer0_states_ok=not_run layer0_violations=not_run control_states_ok=true control_violations_ok=true control_missed_zero=true control_false_alarm_zero=true control_outcome_matrix_ok=false g6_states_ok=true g6_violations_zero=true g7_states_ok=false g7_violations_zero=false positive_control_main_geometry_ok=true role_label_contradictions_zero=true role_labels_all_defined=true text_number_mismatches_zero=true text_numbers_all_classified=true named_instances_ok=true journal_differing_states=not_run r18_system_configuration_fields_ok=true r18_root_flags_ok=true r18_g8_ok=true r18_g9_ok=true
```

表示「没过」的字段（主产物第 732 行；臂 N18 第 714 行同值）：
- `layer0_states_ok=not_run`、`layer0_violations=not_run`、`journal_differing_states=not_run`：层 0 主臂不在默认调用里（第二段，重型，未跑），登记预期内。
- `control_outcome_matrix_ok=false`：同一产物第 658 行 `E7RESULT name=layer0_control_outcome_matrix_summary off_diagonal=2`，门槛 `off_diagonal == 0`；F13（一盘无暖机时重放不跨实例边界，2 个合法状态合法的子类不同）。与第十八次相同，登记 R35 说这一格不改门槛、只报值——预期内（值没变），「该不该是 false」拿不准的那一半仍是第十七次登记的事。
- `g7_states_ok=false`、`g7_violations_zero=false`：第 660 行 `E7RESULT name=layer0_sensitivity point=G7 barriers=none fua_is_boundary=true segments=12+1 states=4097 closed_form=4097 violations=2046 missed=0 false_alarm=0 file_read=3 journal_differing=0`，门槛写死 2050 与 0；登记第四节第 4 条预言 false，预期内。
违例、不匹配、歧义、失败类的整数计数：`width_mismatches=0`。

## 交回之前第 1 条：整行抄（主产物）

```
E7RESULT name=segments path=mkfs operations=23 segments=12+1+1+1+4 closed_form=4114 kinds=[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]
E7RESULT name=segments path=instance_acquisition operations=2 segments=2 closed_form=4 kinds=[system_configuration_slot×2]
E7RESULT name=segments path=warm_up operations=22 segments=2+1+2+2+1+2 closed_form=15 kinds=[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]
E7RESULT name=segments path=transaction operations=35 segments=24+2+1+2 closed_form=16777223 kinds=[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]
E7RESULT name=segments path=post_mkfs_stream operations=59 segments=2+2+1+2+2+1+2+24+2+1+2 closed_form=16777240 kinds=[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]
E7RESULT name=segments_three_state path=mkfs writes=19 in_place_overwrites=0 in_place_per_segment=0+0+0+0+0 in_place_kinds=none closed_form_two_state=4114 closed_form_three_state=4114
E7RESULT name=segments_three_state path=instance_acquisition writes=2 in_place_overwrites=2 in_place_per_segment=2 in_place_kinds=[system_configuration_slot×2] closed_form_two_state=4 closed_form_three_state=9
E7RESULT name=segments_three_state path=warm_up writes=10 in_place_overwrites=4 in_place_per_segment=0+0+2+0+0+2 in_place_kinds=[system_configuration_slot×4] closed_form_two_state=15 closed_form_three_state=25
E7RESULT name=segments_three_state path=transaction writes=29 in_place_overwrites=2 in_place_per_segment=0+0+0+2 in_place_kinds=[system_configuration_slot×2] closed_form_two_state=16777223 closed_form_three_state=16777228
E7RESULT name=segments_three_state path=post_mkfs_stream writes=41 in_place_overwrites=8 in_place_per_segment=2+0+0+2+0+0+2+0+0+0+2 in_place_kinds=[system_configuration_slot×8] closed_form_two_state=16777240 closed_form_three_state=16777260
E7RESULT name=segments_sensitivity point=G10a swallowed_device=1 path=transaction operations=34 segments=24+5 closed_form=16777247 closed_form_three_state=16777287 in_place_per_segment=0+2 kinds=[unit_write×24,barrier×2]|[journal_record×2,root_record_fua,system_configuration_slot×2,barrier×3]
E7RESULT name=segments_sensitivity point=G10a swallowed_device=1 path=post_mkfs_stream operations=58 segments=2+2+1+2+2+1+2+24+5 closed_form=16777264 closed_form_three_state=16777319 in_place_per_segment=2+0+0+2+0+0+2+0+2 kinds=[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,root_record_fua,system_configuration_slot×2,barrier×3]
E7RESULT name=segments_sensitivity point=G10b swallowed_device=0 path=transaction operations=34 segments=24+3+2 closed_form=16777226 closed_form_three_state=16777231 in_place_per_segment=0+0+2 kinds=[unit_write×24,barrier×2]|[journal_record×2,root_record_fua,barrier]|[system_configuration_slot×2,barrier×2]
E7RESULT name=segments_sensitivity point=G10b swallowed_device=0 path=post_mkfs_stream operations=58 segments=2+2+1+2+2+1+2+24+3+2 closed_form=16777243 closed_form_three_state=16777263 in_place_per_segment=2+0+0+2+0+0+2+0+0+2 kinds=[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,root_record_fua,barrier]|[system_configuration_slot×2,barrier×2]
E7RESULT name=segments_sensitivity point=G11 trailing_barrier=off path=mkfs operations=23 segments=12+1+1+1+4 closed_form=4114 closed_form_three_state=4114 in_place_per_segment=0+0+0+0+0 kinds=[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]
E7RESULT name=segments_sensitivity point=G11 trailing_barrier=off path=instance_acquisition operations=2 segments=2 closed_form=4 closed_form_three_state=9 in_place_per_segment=2 kinds=[system_configuration_slot×2]
E7RESULT name=segments_sensitivity point=G11 trailing_barrier=off path=warm_up operations=18 segments=2+1+2+2+1+2 closed_form=15 closed_form_three_state=25 in_place_per_segment=0+0+2+0+0+2 kinds=[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]
E7RESULT name=segments_sensitivity point=G11 trailing_barrier=off path=transaction operations=33 segments=24+2+1+2 closed_form=16777223 closed_form_three_state=16777228 in_place_per_segment=0+0+0+2 kinds=[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]
E7RESULT name=segments_sensitivity point=G11 trailing_barrier=off path=post_mkfs_stream operations=53 segments=2+2+1+2+2+1+26+2+1+2 closed_form=67108885 closed_form_three_state=150994980 in_place_per_segment=2+0+0+2+0+0+2+0+0+2 kinds=[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[unit_write×24,system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]
E7RESULT name=segments_sensitivity point=G12a trailing_barrier=off swallowed_device=1 path=transaction operations=32 segments=24+5 closed_form=16777247 closed_form_three_state=16777287 in_place_per_segment=0+2 kinds=[unit_write×24,barrier×2]|[journal_record×2,root_record_fua,system_configuration_slot×2,barrier]
E7RESULT name=segments_sensitivity point=G12a trailing_barrier=off swallowed_device=1 path=post_mkfs_stream operations=52 segments=2+2+1+2+2+1+26+5 closed_form=67108909 closed_form_three_state=150995039 in_place_per_segment=2+0+0+2+0+0+2+2 kinds=[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[unit_write×24,system_configuration_slot×2,barrier×2]|[journal_record×2,root_record_fua,system_configuration_slot×2,barrier]
E7RESULT name=segments_sensitivity point=G12b trailing_barrier=off swallowed_device=0 path=transaction operations=32 segments=24+3+2 closed_form=16777226 closed_form_three_state=16777231 in_place_per_segment=0+0+2 kinds=[unit_write×24,barrier×2]|[journal_record×2,root_record_fua,barrier]|[system_configuration_slot×2]
E7RESULT name=segments_sensitivity point=G12b trailing_barrier=off swallowed_device=0 path=post_mkfs_stream operations=52 segments=2+2+1+2+2+1+26+3+2 closed_form=67108888 closed_form_three_state=150994983 in_place_per_segment=2+0+0+2+0+0+2+0+2 kinds=[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[unit_write×24,system_configuration_slot×2,barrier×2]|[journal_record×2,root_record_fua,barrier]|[system_configuration_slot×2]
E7RESULT name=write_list writes=29 barriers=6 fua=1 named=12 units=t1@50180x32768,t2@50240x16384,t3@50242x32768,t4@50244x16384,t5@50245x16384,t6@50246x16384,t7@50247x16384,t8@50248x16384,t9@50249x16384,t10@50250x16384,t11@50251x16384,t12@50252x16384
E7RESULT name=layer0 arm=settled_two_devices segments=2+2+1+2+2+1+2+24+2+1+2 states=not_run closed_form=16777240 skipped=true reason=附带_够判后未跑
E7RESULT name=layer0_fua_not_boundary segments=2+2+3+2+3+24+2+3 closed_form=16777249
```

## Q142.46（N19C，20 格）与 Q142.58（N18，附带）：比对器原样输出

```
name=extract triples=5 kinds=5 full_three_state=1 tearable=1
name=q46 arm=N19C path=mkfs field=operations model=23 pinned=23 literal_equal=true
name=q46 arm=N19C path=mkfs field=segments model=12+1+1+1+4 pinned=12+1+1+1+4 literal_equal=true
name=q46 arm=N19C path=mkfs field=closed_form model=4114 pinned=4114 literal_equal=true
name=q46 arm=N19C path=mkfs field=kinds model=[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2] pinned=[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2] liter
name=q46 arm=N19C path=instance_acquisition field=operations model=2 pinned=2 literal_equal=true
name=q46 arm=N19C path=instance_acquisition field=segments model=2 pinned=2 literal_equal=true
name=q46 arm=N19C path=instance_acquisition field=closed_form model=4 pinned=4 literal_equal=true
name=q46 arm=N19C path=instance_acquisition field=kinds model=[system_configuration_slot×2] pinned=[system_configuration_slot×2] literal_equal=true
name=q46 arm=N19C path=warm_up field=operations model=22 pinned=20 literal_equal=false
name=q46 arm=N19C path=warm_up field=segments model=2+1+2+2+1+2 pinned=2+1+2+2+1+2 literal_equal=true
name=q46 arm=N19C path=warm_up field=closed_form model=15 pinned=15 literal_equal=true
name=q46 arm=N19C path=warm_up field=kinds model=[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2] pinned=[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[
name=q46 arm=N19C path=transaction field=operations model=35 pinned=35 literal_equal=true
name=q46 arm=N19C path=transaction field=segments model=24+2+1+2 pinned=24+2+1+2 literal_equal=true
name=q46 arm=N19C path=transaction field=closed_form model=16777223 pinned=16777223 literal_equal=true
name=q46 arm=N19C path=transaction field=kinds model=[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2] pinned=[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2] literal_equal=true
name=q46 arm=N19C path=post_mkfs_stream field=operations model=59 pinned=57 literal_equal=false
name=q46 arm=N19C path=post_mkfs_stream field=segments model=2+2+1+2+2+1+2+24+2+1+2 pinned=2+2+1+2+2+1+2+24+2+1+2 literal_equal=true
name=q46 arm=N19C path=post_mkfs_stream field=closed_form model=16777240 pinned=16777240 literal_equal=true
name=q46 arm=N19C path=post_mkfs_stream field=kinds model=[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,b
name=q47_crates arm=N19C path=post_mkfs_stream field=closed_form_three_state model=16777260 pinned=16777260 equal=true
name=q47_crates arm=N19C path=post_mkfs_stream field=in_place_kinds model=[system_configuration_slot×8] pinned=[system_configuration_slot×8] equal=true
name=q49 item=i_before_window_operations model=47 crates=45 equal=false
name=q49 item=i_before_window_writes model=31 crates=31 equal=true
name=q49 item=i_before_window_last_five model=device0@16781312+4096,device1@16781312+4096,device0@7340032+512,device0@0+4096,device1@0+4096 crates=device0@16781312+4096,device1@16781312+4096,device0@7340032+512,device0@0+4096,device1@0+4096 equal=true
name=q49 item=ii_transaction_operations model=35 crates=35 equal=true
name=q49 item=ii_transaction_writes model=29 crates=29 equal=true
name=q49 item=ii_transaction_barrier_steps model=6 crates=6 equal=true
name=q49 item=iii_window_segments model=24+2+1+2 crates=24+2+1+2 equal=true
```

Q142.58（臂 N18 对 step_five，只报不判）：20 格里 `literal_equal=true` 的有 10 格（取号四格与 mkfs、暖机、事务的段序列与两态），其余都差在屏障步数与发布尾屏障。

锚点逐格（`judge.py`，Q142.47/48/57/60/61、C10、C12）：145 格 `equal=true`、0 格 `equal=false`；`name=row_counts segments=5 segments_three_state=5 segments_sensitivity=13 segments_paths_ok=true three_state_paths_ok=true`；`write_list_ok=true`。

## Q142.51（6 × 3）

| 字段 | 第十八次（`-r18-main-2` 第 714 行） | 臂 N18（第 714 行） | N19C（第 732 行） |
|---|---|---|---|
| r18_system_configuration_fields_ok | true | true | true |
| r18_root_flags_ok | true | true | true |
| r18_g8_ok | true | true | true |
| r18_g9_ok | true | true | true |
| control_outcome_matrix_ok | false | false | false |
| g7_states_ok | false | false | false |

N18 = 第十八次（V19a 没触发）；N19C = N18（F19d 没触发）。F19f：两个 `false` 都「与第十八次相同」。

## Q142.52–Q142.54 答案表（R38；命令 `python3 /tmp/claude-1000/e142-r19-runner3/judge/groups.py <diff>`）

段序列那几组先列：

| 份 | 行组 | 删 / 加 | 归因与证据 |
|---|---|---|---|
| D_model | segments path=mkfs | 1 / 1 | ① 屏障按设备（`operations` 21→23、`barrier`→`barrier×2`）、④ `zero_fill` 排前 |
| D_model | segments path=warm_up | 1 / 1 | ① + ⑧（14→22） |
| D_model | segments path=transaction | 1 / 1 | ① + ⑧（31→35） |
| D_model | segments path=post_mkfs_stream | 1 / 1 | ① + ⑧（47→59；⑧ 把 26 拆成 2+24，67108885→16777240） |
| D_model | segments_three_state × 5 路径 | 0 / 5 | ③ |
| D_model | segments_sensitivity G10a、G10b（各 2） | 0 / 4 | ⑥ |
| D_model | segments_sensitivity G11（5）、G12a、G12b（各 2） | 0 / 9 | ⑩ |
| D_model | layer0 arm=settled_two_devices | 1 / 1 | ⑧（段序列与闭式 16777240） |
| D_model | layer0_fua_not_boundary | 1 / 1 | ⑧（134217754→16777249，= 7.4 C13） |
| D_model | warm_up、write_list、write_list_summary | 各 1 / 1 | `barriers=` 按录制步数：①⑤⑧⑨（4→12、2→6、2→6） |
| D_model | before_window_summary side=model | 1 / 1 | ①⑧（37→47） |
| D_model | header311_write_digest | 48 / 48 | 只变 `index=`（窗口步号里屏障按设备、多发布尾）：①⑧ |
| D_model | device_region_bytes、write_list_row、impl_bytes_equal | 各 5 / 5 | 只变 `step=`：①⑧ |
| D_model | self_check_text_numbers | 1 / 1 | 新加行带来的计数（③⑥⑩） |
| D_model | done | 1 / 1 | 行数 |
| D_crates | impl_config | 1 / 1 | 只变 `mkfs_operations` 21→23：实审 B3a-2（`research/prompts/m2-rev-b3a2-implementer-report.md` 第二节首条 `crates/singlefs-harness/src/lib.rs`；今天第 179 行 `fn push` 按设备记屏障） |
| D_crates | before_window_summary | 1 / 1 | 37→45：B3a-2（同上）+ 实 C577（`research/prompts/m2-impl-c577-barrier-implementer-report.md`「这一轮写过的文件」首条 `crates/singlefs-core/src/transaction.rs`；今天第 1201 行 `fn persist_the_root_then_rotate_the_system_configuration`、第 1216 行 `CommitStep::Barrier`）；暖机第一次末尾那道被 `push` 并进第二次开头（B3a-2），所以是 45 不是 47 |
| D_crates | window_barrier | 1 / 5 | B3a-2（按设备）+ C577（step 33、34 两步发布尾） |
| D_crates | device_region_bytes | 5 / 5 | 只挪 `step=`，`sha256=` 不变：同上两份 |
| D_crates | device_region_bytes_summary | 1 / 1 | 31/2→35/6：同上两份 |
| D_crates | done | 1 / 1 | 多 4 行 `window_barrier` |

两份报告点名的 `crates/` 文件在第十八次快照与 A28 之间都变了：`lib.rs` `edc0799bd1d4`→`405138c688d8`，`transaction.rs` `5abb54e5bdcd`→`af33d518a584`（第十八次快照 `research/results/e142-r18-crates-sha256-after-2026-09-26.txt` 带 `./` 前缀）。D_total（212 行，192 条 ±）按两份分 diff 机械标：D_model 168、D_crates 24、NOT_FOUND 0（`/tmp/claude-1000/e142-r19-runner3/step3/d_total_labels.out`）。改名带来的行组 0 个：三份产物新字段名的行各 7、旧名 0；D_crates 里含 `new_pool_file_creation` 的行 0，D_model 里 2 行是 `warm_up` 那一对，只变 `barriers=`。

## 控制（`research/results/e142-r19-controls-2026-09-28.out` 原样）

```
# PC-c
picked_line=390 steps=[29, 30] region_offset=6 old_byte=02 new_byte=03
synthetic_sha256=c7a807ae88685d502ba3ab7a6345a5ec2a2a2348e855ead80dd00a895e8ae7f7
E7RESULT name=old_new_independent_summary new_regions=29 old_regions=29 matched=29 changed=1 unchanged=28 unmatched_new=0 unmatched_old=0 mapped_regions=0 unmapped_regions=1
E7RESULT name=old_new_independent_summary new_regions=29 old_regions=29 matched=29 changed=0 unchanged=29 unmatched_new=0 unmatched_old=0 mapped_regions=0 unmapped_regions=0
# PC-a PC-b P2 self-proof
name=pc_a_i real_extract="name=extract triples=5 kinds=5 full_three_state=1 tearable=1" differing_fields=3 line5.kinds line6.closed_form line7.closed_form_three_state ok=true
name=pc_a_ii arm=N19C self_cells=22 self_equal=22 mutated_equal=19 mutated_unequal=path=transaction field=kinds;path=post_mkfs_stream field=closed_form;path=post_mkfs_stream field=closed_form_three_state; ok=true
name=pc_a_ii arm=N18 self_cells=20 self_equal=20 mutated_equal=18 mutated_unequal=path=transaction field=kinds;path=post_mkfs_stream field=closed_form; three_state_cell=absent_in_arm_n18 ok=true
name=pc_b arm=N19C copy_sha256=22ac278ddd4693582fbbb7f4cb2e29f8df2166abc24c81ed5c350856bbbbb0c9 deleted=2 added=2 segments_transaction_lines=2 device_region_bytes_lines=2 e7input_lines=0 ok=true
name=pc_b arm=N18 copy_sha256=c161ab2a5f03bb2f698ad4e5734132fb71e7b196691bad6cfeec35d2d1c31cb3 deleted=2 added=2 segments_transaction_lines=2 device_region_bytes_lines=2 e7input_lines=0 ok=true
name=p2 arm=e142-new-pool-file-creation-dry-run-2026-09-28-r19-main.out 4 rows; ok_rows=4
name=p2 arm=e142-new-pool-file-creation-dry-run-2026-09-28-r19-arm-n18.out 4 rows; ok_rows=4
name=self_proof n=1 unequal_cells=2 name=q57 point=G10a path=transaction field=segments;name=q57 point=G10a path=post_mkfs_stream field=segments;
name=self_proof n=2 unequal_cells=2 name=q48 arm=N19C path=mkfs field=in_place_per_segment;name=q47 arm=N19C path=mkfs field=in_place_per_segment;
name=self_proof n=3 unequal_cells=1 name=q60 point=G11 path=post_mkfs_stream field=segments;
name=self_proof n=4 unequal_cells=4 name=q48 arm=N19C path=post_mkfs_stream field=segments;name=q48 arm=N19C path=post_mkfs_stream field=closed_form;name=q47 arm=N19C path=post_mkfs_stream field=closed_form_three_state;name=c12 arm=N19C row=layer0 field=segments;
name=self_proof n=0_restored unequal_cells=0
```

PC-a：臂 N18 没有三态行，自比 20 格、改两格（登记写的是每条臂 21 格改三格，N18 那一格不存在，照实记）；N19C 自比 22 格（多比了 `in_place_kinds`）、改三格。合成文件的 sha256：step_five 拷贝 `253c511f…`、crash_segments 拷贝 `86577a1e…`、PC-b 拷贝 `22ac278d…`（N19C）/ `c161ab2a…`（N18）、PC-c 合成 `c7a807ae…`，都在 `/tmp/claude-1000/e142-r19-runner3/controls/`。V19d 没触发。

## 复跑

- `research/scripts/replay.sh` 第 171 行改成 `E142|@driver_e142||e142-new-pool-file-creation-dry-run-2026-09-28-r19-main.out|exact`。
- `bash research/scripts/replay.sh E142` 退 0，末段原样：「字节一致 0 ／ 仅计时不同 0 ／ 对不上 0 ／ 跑不了 0 ／ 结论断言不中 0 ／ 产物已归档 0 ／ 输入没变没跑 1」——主产物头的指纹就是当时的输入，准入不重跑。逐字节复现改在第二台同一棵树里再跑一次 `driver_e142` 两条命令核：与主产物 `cmp` 退 0。
- Q142.50：主产物落盘后立即 `python3 research/scripts/admission.py experiment <仓根> E142` 退 77，stderr 原样「✗ 拒绝开跑 E142：输入自上次产物以来没变，不必重跑——research/results/e142-new-pool-file-creation-dry-run-2026-09-28-r19-main.out 头上的输入指纹与今天的相同（5ffcf9958648）」。
- ⚠️ 之后别的会话改了准入输入 `.claude/kb/decisions/23-journal的角色与格式.md`（「欠」那几行，mtime 晚于主产物约 7 分钟，不是我改的）：现在准入又放行（今天 `62f1ad71f8a4`），门禁 69 号第一类对 E142 又红。另一处：69 号取的是 mtime 最新的 E142 产物，那是臂 N18（它头上是第二台 N18 树的指纹 `c1c620be…`，按定义永远对不上主工作区）——主产物之后再落任何别的 E142 产物都会这样。登记第四节第 10 条与主 agent 认定第 4 条已定「接受事后补跑一次 `driver_e142`」，补跑之后最新的就是它。

## 门禁（登记给执行员的 14 道，写完实验页之后跑；原样末行）

```
27-format-constants exit=0 |   ✓ 格式常量同步（51 个已登记，51 个在源码里被钉住）
33-mutation-tables exit=0 |   ✓ 153 个实验二进制都有成形的变异表，1747 条变异的原文各命中源码一次；crates/mutations.tsv 1401 条的原文各命中源码一次（…）
34-experiment-index-sync exit=0 |   ✓ 实验索引行与正文标题一致（索引 162 行、正文 162 份）
40-results-cited exit=1 |        E163：e163-gpu-multicard-crc32c-2026-09-27-r1-merge.out e163-gpu-multicard-crc32c-2026-09-27-r1.out
52-segment-registry exit=1 |        再让 research/scripts/replay.sh 里 E142 那一行的入库产物名字跟上。
80-absolute-assertions exit=0 |       crates/singlefs-checker-tier/src/bin/new_pool_file_creation_region_bytes.rs
84-verdict-false-named exit=0 |   ✓ 登记产物里判决字段是 false 的都在对应实验页里点了名（点名 15 个）
85-repro-command exit=0 |     写明「原始输出未留存」不判的 0 页：（没有）
86-experiment-orphans exit=0 |     扫了 research crates/singlefs-checker-tier/src/bin（target/ 不扫）；不在的目录 0 个：（没有）
88-quoted-result-lines exit=0 |        research/perf-by-milestone.md:353,354
69-evidence-in-repo exit=1 |                原件已经没了，就写明它没了、把还核得动的那部分落进仓里；只是在说做法（草稿放在哪），把句子里的依据词去掉。
75-decision-experiment-links exit=1 |             支撑 / 推翻的那条分项在「**依据**」段引回这个实验；待回填清单只能删行，新实验页当场写全这张表。
96-experiment-source-discipline exit=0 |         e92_reuse_requirement.rs
99-multipath-registry exit=0 |   ✓ 登记了路径与共用项的实验页判过了（5 份、21 条路径）；…
```

红的四道：40 号只点 E163（E142 这一次的新文件都被点名了，日志里 E142 0 行）；52 号是预期的（`layout/01` 第八节没改，见下一节）；69 号 E142 两行是上一条说的指纹问题，另有 E101、E141、`research/prompts/e142-r19-runner-report.md:148` 的 /tmp 引用，都不是这一次写的；75 号只点 E159（「没回看 1」），E142 的 37 行回看都写了 2026-09-28。日志在 `/tmp/claude-1000/e142-r19-runner3/gates/`。

## `layout/01-first-txn.md` 第八节该写成什么（没改，交主 agent 派书记员）

`python3 research/scripts/check-segment-registry.py --root <仓根>` 今天退 1，报 5 处（原样首行「✗ 段序列登记表与 E142 产物的 name=segments 行（或第二条流与钉它的用例）对不上，共 5 处」；输出在 `/tmp/claude-1000/e142-r19-runner3/step3/seg-registry.out`）。

- **mkfs 种根那一行**：按门禁 52 号它比的是 `crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs` 的 `recorded_stream_matches_the_registered_mkfs_segment_sequence`（钉 `"[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]"`）；新主产物 mkfs 那一行与它逐字相同，原样（主产物第 35 行）：
  `E7RESULT name=segments path=mkfs operations=23 segments=12+1+1+1+4 closed_form=4114 kinds=[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]`
  ⇒ 那一行的「21 次操作」改 23，种类串两处 `barrier` 改 `barrier×2`。
- **暖机、普通发布、整条流三行**：照主产物第 37–39 行（整行在「交回之前第 1 条」一节）。注意这三行里暖机与整条流的 `operations=`、`kinds=` 是模型的（不合并连着的屏障），与 step_five 钉的（20、57、`[journal_record×2,barrier×2]`）差在 R40 分叉；52 号比的是 E142 产物，照产物写 52 号就绿、而与 `crates/` 钉值不同——写哪一边是 R40 那条岔的决定，交主 agent。表后「发布与下一次发布之间没有屏障」那一句 C577 之后不成立（整条流 `…+2+24+…`）。
- **第二条流那一行**：它不来自 E142 产物（E142 只录第一条流），52 号比的是 `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` 第 432 行 `Script::ReuseAfterRaisingFloorThenNormalUnmount` 那一臂钉的数组，今天原样是 `2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2, 16, 2, 1, 2, 24, 2, 1, 2, 20, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2`（78 段、写数和 477，同一文件第 96 行 `FULL_STATES_WITH_TWO_STATES_PER_WRITE_THROUGH_THE_UNMOUNT: u64 = 1_662_648_449`）。kb 里的 61 段、6649413746、快档 232 与探针行都是 C577 之前的；快档数与探针行这一次没量，要由跑那条用例的一方给。

## 决策格（7b）

- 37 行都回看了，都写「2026-09-28 不受影响：…」。没有写支撑 / 推翻的新格。
- `D16（发布语义） 已定项 7` 维持「备料」：`grep -n 'E142（' .claude/kb/decisions/16-发布语义.md` 只命中第 210、225 行（不在已定项 7）。这一次的 N19C 照它的定案句加发布尾屏障、段序列与闭式与 `crates/` 相同——**这一格该不该升成支撑交主 agent**。
- `D13（验证路线） 已定项 4` 维持「支撑」（依据第 78 行引 E142）。R40 分叉说明定案句没写「连着的屏障合不合并」，要不要补一句交主 agent。
- `.claude/decision-links-pending` 今天是空的（`grep -v '^#'` 0 行），② 不适用。

## 岔路表（问题单 `research/prompts/e142-r19-questions.md`）

| # | 状态 | 算出它的命令 | 剩下的量能不能让它翻面 |
|---|---|---|---|
| 1 段数、写数、两态与三态闭式 | **已够判**：Q142.46 16 同 4 不同（R40 分叉，S19-crates 交主 agent）、Q142.47–49、Q142.57、Q142.60、Q142.61 全判完，Q142.50 退 77，PC-a、PC-d 过，R41 十行齐 | `compare_crates.py compare …`、`judge.py …`（输出在 `research/results/e142-r19-comparisons-2026-09-28.out`） | 不能：剩下的只有第二段层 0 枚举，它不在第 1 行的够判条件里；「相同 / 不同」只会因主 agent 对 R40 的决定改写法而变 |
| 2 六个判决字段 | **已够判**：6 × 3 表三列齐，N19C 与第十八次无一不同；Q142.56 过 | `grep -n name=verdict` 三份产物 | 不能 |
| 3 与第十八次逐行比、各因哪一批 | **已够判**：D_crates 6 组归 B3a-2 / C577，D_model 34 组归 ①–⑪，D_total 192 行全标上；改名行组 0；PC-b、PC-c 过；Q142.55 `changed=0` | `groups.py <diff>`、D_total 标注脚本（`/tmp/claude-1000/e142-r19-runner3/step3/d_total_labels.out`） | 不能 |
| 第二段（R19C-8 层 0 主臂） | 未跑：重型，等主 agent 问用户 | — | 不补三行任何一行 |

## 删了什么

- 本机仓副本 `/tmp/claude-1000/e142-r19-runner3/n18-repo`（703M，本机 N18 编译用，改到第二台之后删）。
- 第二台上这一趟的两棵树与文件（各 1.2G 的 `e142-r19-main`、`e142-r19-n18` 与它们旁边的日志、脚本）、`mutate.sh` 在第二台新建的两个编译目录（各 65M，建于这一次变异跑的那一刻）。
- `replay.sh` 建的本机输出目录。草稿目录 `/tmp/claude-1000/e142-r19-runner3/`（18M）里没有编译目录与仓副本，脚本、比对输出、门禁日志留给主 agent 核。

## 没做什么

- 第二段层 0 主臂枚举（重型）没跑。`layout/01-first-txn.md` 第八节没改（不在写范围）。门禁 69 号 E142 那两行要补跑一次 `driver_e142`（D23 被别人改过）才转绿，没补跑（登记之后改输入的事由主 agent 定）。
- 没判这个实验的结论能不能推翻或确立决策；没跑门禁全量、doc-lint；没提交；没改问题单状态。
