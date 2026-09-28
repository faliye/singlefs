# E142 第十九次跑续做（第四段）执行员报告

日期 2026-09-28。照用户新定的「同一块盘上连着的屏障并成一道」改独立模型（臂 N19M）、重跑步 ③④。编译、单测、变异与产物都在本机跑（开工时 `run-with-memory-cap.sh --status`：slice 总上限 40.1 GiB、已占 346 MiB，不排队），没用第二台。第二段（层 0 主臂枚举）是重型，没跑。

## 结论

- 问题单第 1 行：Q142.46 的 20 格**全部** `literal_equal=true`，上一段不同的 4 格（暖机、整条流的 `operations=`、`kinds=`）照新规矩与 `crates/` 钉的相同（暖机 20、整条 57，暖机第二次开头那一段 `[journal_record×2,barrier×2]`）。Q142.49 7 项全同（窗口之前 45 = 45）。Q142.47 三态 16777260 与取三态种类同 `crates/`。锚点逐格 145 格 `equal=true`、0 格 `equal=false`。停机 S19-crates 没触发。
- 第 2 行：六个字段在第十八次、臂 N18、N19C、N19M 四份产物里逐字相同（四个 `true`，`control_outcome_matrix_ok=false`、`g7_states_ok=false`，都与第十八次相同）。
- 第 3 行：D_model 34 组、D_total 192 行（D_crates 24、D_model 168、两者 0、找不到 0），分组与行数与上一段逐个相同；N19C → N19M 只差 7 组（见「与 N19C 的差」）。
- 字节：`old-new` `changed=0`；`impl_bytes_equal_summary … equal=29 unequal=0`（S19-bytes 没触发）。
- 推翻条件：`python3 /tmp/claude-1000/e142-r19-runner4/judge/compare_crates.py compare <主产物> N19M <step_five> <crash_segments>` 出现 `literal_equal=false`；或 `judge.py` 出现 `equal=false`；或 `old-new` 的 `changed` 不为 0。

## 登记修订（都在这一段主产物之前）

`research/prompts/e142-r19-prereg.md` 第十二节加两条：
- **R19E-4**（10 条）：派发提示「主 agent 对 S19-crates 的处置」原句整段引入；N19C 记一次输；新读法 **R46** 替换 R40（「任何一次写」取任何一块盘上的写，对面读法「只看这块盘上的写」在这条流上认出的集合相同，推的）；新臂 **N19M** = N19C + ⑫；锚点换值表（D2/C3 暖机 22→20、D2/C5 整条 59→57、C10 47→45、D3 暖机与整条那一段 `barrier×4`→`barrier×2`、G10 两点整条 58→56、暖机屏障步 12→10），新值出自登记 R19C-9 那份脚本（`e25e6d0b…`，现核 sha 与原样输出 `f624b80d…` 都相同）把末段循环全换成 `merge=True` 的一份（`f2f44594…`，输出 22 行 `42ca21e1…` 全文贴进登记）；`point=N19M` 六行与登记原有 `point=N19C_merge_reading` 六行 `diff` 为空，G11/G12 的 merge 行与原行 `diff` 为空；加变异 M202「不合并同一块盘连着的屏障」、M203「只看这块盘上的写」与一条合成单测；Q142.46 任一格不同 ⇒ 停机（派发提示第 2 条）；S19-clause 基线换成 `80e98fbd…`（190 行，与 `4d98ee79…` 只差 D13 已定项 4 的三处：出处行 `:70-84`→`:70-85`、定案末句、依据第 82 行那一条）；沿用臂 N18 产物与 D_crates（`crates/` 今天与 A28 `cmp` 退 0）；这一段的新文件名；在哪台跑。
- **R19E-5**：单测 95 过、变异 182 全抓、冻结值模型 `ebefbe936d5f173420b0d568ca47ddc64069afc724764a676fb03fd561f2a4dd`（9371 行）、变异表 `ec2af965de10fb42fb4d79e141dda75c355273ef822e998ca2ef0274512aa107`（182 行）。主产物落盘之后登记一字没改。

## 单测、变异

- 单测（本机，`research/` 下 `nice -n 19 bash scripts/run-with-memory-cap.sh 8G bash scripts/capped.sh 32 cargo test --release -p e7-index-bench --bin e142-new-pool-file-creation-dry-run`）末行原样：`test result: ok. 95 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.73s`；`grep -c '^test .* ok$'` = 95；`grep -c '^warning'` = 0。
- 变异整表（`MUTATE_JOBS=2 nice -n 19 bash scripts/capped.sh 16 bash scripts/mutate.sh …`，182 条），按行首符号数：✅ 182、⏭ 0、❌ 0、💥 0、⚠️ 0、⏱ 0、🧱 0；收尾原样「计数：内存撞顶 0 条（上限 16G）、超时 0 条」「已还原，基线仍全绿」，退出码 0。**抓到 182 / 无效 0 / 没红 0**；已有 180 条无一要重锚。M202 被五条单测抓到，M203 只被合成单测 `consecutive_barriers_on_one_device_merge_and_any_write_in_between_keeps_them_apart` 抓到（原样：`✅ [M203_only_writes_on_the_same_device_keep_barriers_apart] 红：tests::consecutive_barriers_on_one_device_merge_and_any_write_in_between_keeps_them_apart`）。日志 `research/results/e142_new_pool_file_creation_dry_run-mutate-2026-09-28.log`（sha256 `0e9f8283…`）。
- 命名：`naming-lint.sh` 对新写的名字零命中（文件里原有的 128 处命中不是这一段的；那条 `a_pool_barrier_…` 单字母命中是原有测试名）。clippy `-D warnings` 在这个 bin 上报 25 处，全在原有行，不在这一段改的行。

## 产物（`research/results/`，第一段的产物一个没动）

| 文件 | 说明 |
|---|---|
| `e142-new-pool-file-creation-dry-run-2026-09-28-r19-main-2026-09-28.out` | 主产物，772 行，sha256 `00e7d52b45f6d46ce14521cddbc33474bcd61f27e798dc1016d636028d449f48`，头 `E7INPUT … sha256=11551b1e15cf… files=200`，`name=done` 2 行（第 733 行 `emitted=732`、第 772 行 `emitted=39`），`forced_rerun` 0 行 |
| `e142-region-old-new-independent-2026-09-28.out` | Q142.55 |
| `e142-r19-comparisons-2026-09-28-r2.out` | 比对甲、Q142.47 crates 格、Q142.49、锚点逐格 |
| `e142-r19-controls-2026-09-28-r2.out`、`…-r3.out` | 控制；`-r2` 臂 N18 的 PC-a(ii) 记 `ok=false` 是我的脚本把「恰三格不同」写死、没顾到 N18 没有三态行（测到的两格就是该变的两格），改脚本后重跑存 `-r3`，两份只差这一行 |
| `e142-r19-diff-{arm-n18-to-main,r18-to-main}-2026-09-28-r2.txt` | 186 / 212 行 |
| `e142-r19-crates-sha256-{b,c}-2026-09-28-r2.txt` | `cmp` A28–B、B–C 都退 0 |
| `e142_new_pool_file_creation_dry_run-mutate-2026-09-28.log` | 变异日志 |

模型准入 stderr 原样：「✓ 放行 E142：输入自 research/results/e142-new-pool-file-creation-dry-run-2026-09-28-r19-arm-n18.out 以来变了（那一份 c1c620beadad，今天 11551b1e15cf）」。步 ③(1) 重抄 sha256 `80e98fbd…` = 新基线。跑之前 `ps` 看到别的会话一条 `cargo check --offline --all-targets`（pid 1001338），没等，照常跑。

4b 按行类数（命令 `awk '/name=done/{d++} {match($0,/name=[a-z0-9_]+/); n=substr($0,RSTART+5,RLENGTH-5); side=(d==0?"model":"crates"); if (n!="") c[side":"n]++} END{for(k in c) print c[k], k}' <主产物>`），81 类，与 N19C 那一份逐类同数（`diff` 空）；节选原样：`5 model:segments`、`5 model:segments_three_state`、`13 model:segments_sensitivity`、`1 model:verdict`、`29 model:device_region_bytes`、`29 model:impl_bytes_equal`、`4 model:positive_control_p2`、`2 model:window_segments`、`29 crates:device_region_bytes`、`6 crates:window_barrier`、`1 crates:device_region_bytes_summary`。

## 与 N19C 的差（归一化 diff，92 行，7 组）

`warm_up`（`barriers=` 12→10）、`before_window_summary side=model`（47→45）、`segments path=warm_up`（22→20、那一段 `barrier×4`→`×2`）、`segments path=post_mkfs_stream`（59→57、同上）、`segments_sensitivity` G10a / G10b 整条（58→56）、`header311_write_digest` 34 行（只差 `index=`，每行少 2；去掉 `index=` 之后逐行相同）。都归 ⑫。段序列、两态、三态、判决行、字节一行不变。

## 交回之前第 1 条：整行抄（主产物）

```
E7RESULT name=segments path=mkfs operations=23 segments=12+1+1+1+4 closed_form=4114 kinds=[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]
E7RESULT name=segments path=instance_acquisition operations=2 segments=2 closed_form=4 kinds=[system_configuration_slot×2]
E7RESULT name=segments path=warm_up operations=20 segments=2+1+2+2+1+2 closed_form=15 kinds=[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]
E7RESULT name=segments path=transaction operations=35 segments=24+2+1+2 closed_form=16777223 kinds=[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]
E7RESULT name=segments path=post_mkfs_stream operations=57 segments=2+2+1+2+2+1+2+24+2+1+2 closed_form=16777240 kinds=[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]
E7RESULT name=segments_three_state path=mkfs writes=19 in_place_overwrites=0 in_place_per_segment=0+0+0+0+0 in_place_kinds=none closed_form_two_state=4114 closed_form_three_state=4114
E7RESULT name=segments_three_state path=instance_acquisition writes=2 in_place_overwrites=2 in_place_per_segment=2 in_place_kinds=[system_configuration_slot×2] closed_form_two_state=4 closed_form_three_state=9
E7RESULT name=segments_three_state path=warm_up writes=10 in_place_overwrites=4 in_place_per_segment=0+0+2+0+0+2 in_place_kinds=[system_configuration_slot×4] closed_form_two_state=15 closed_form_three_state=25
E7RESULT name=segments_three_state path=transaction writes=29 in_place_overwrites=2 in_place_per_segment=0+0+0+2 in_place_kinds=[system_configuration_slot×2] closed_form_two_state=16777223 closed_form_three_state=16777228
E7RESULT name=segments_three_state path=post_mkfs_stream writes=41 in_place_overwrites=8 in_place_per_segment=2+0+0+2+0+0+2+0+0+0+2 in_place_kinds=[system_configuration_slot×8] closed_form_two_state=16777240 closed_form_three_state=16777260
E7RESULT name=segments_sensitivity point=G10a swallowed_device=1 path=transaction operations=34 segments=24+5 closed_form=16777247 closed_form_three_state=16777287 in_place_per_segment=0+2 kinds=[unit_write×24,barrier×2]|[journal_record×2,root_record_fua,system_configuration_slot×2,barrier×3]
E7RESULT name=segments_sensitivity point=G10a swallowed_device=1 path=post_mkfs_stream operations=56 segments=2+2+1+2+2+1+2+24+5 closed_form=16777264 closed_form_three_state=16777319 in_place_per_segment=2+0+0+2+0+0+2+0+2 kinds=[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,root_record_fua,system_configuration_slot×2,barrier×3]
E7RESULT name=segments_sensitivity point=G10b swallowed_device=0 path=transaction operations=34 segments=24+3+2 closed_form=16777226 closed_form_three_state=16777231 in_place_per_segment=0+0+2 kinds=[unit_write×24,barrier×2]|[journal_record×2,root_record_fua,barrier]|[system_configuration_slot×2,barrier×2]
E7RESULT name=segments_sensitivity point=G10b swallowed_device=0 path=post_mkfs_stream operations=56 segments=2+2+1+2+2+1+2+24+3+2 closed_form=16777243 closed_form_three_state=16777263 in_place_per_segment=2+0+0+2+0+0+2+0+0+2 kinds=[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,root_record_fua,barrier]|[system_configuration_slot×2,barrier×2]
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

## 判决行逐行读（4c）

`grep -n name=verdict` 原样（这一段新产物只有主产物一份）：

```
732:E7RESULT name=verdict width_mismatches=0 write_list_ok=true recover_full_ok=true layer0_states_ok=not_run layer0_violations=not_run control_states_ok=true control_violations_ok=true control_missed_zero=true control_false_alarm_zero=true control_outcome_matrix_ok=false g6_states_ok=true g6_violations_zero=true g7_states_ok=false g7_violations_zero=false positive_control_main_geometry_ok=true role_label_contradictions_zero=true role_labels_all_defined=true text_number_mismatches_zero=true text_numbers_all_classified=true named_instances_ok=true journal_differing_states=not_run r18_system_configuration_fields_ok=true r18_root_flags_ok=true r18_g8_ok=true r18_g9_ok=true
```

表示「没过」的字段（主产物第 732 行，与第十八次、臂 N18、N19C 同值）：
- `layer0_states_ok=not_run`、`layer0_violations=not_run`、`journal_differing_states=not_run`：层 0 主臂不在默认调用里（第二段，重型，未跑），登记预期内。
- `control_outcome_matrix_ok=false`：第 658 行 `E7RESULT name=layer0_control_outcome_matrix_summary off_diagonal=2`，门槛 `off_diagonal == 0`（第十七次登记）；与第十八次相同，登记 R35 只报值——预期内。
- `g7_states_ok=false`、`g7_violations_zero=false`：第 660 行 `E7RESULT name=layer0_sensitivity point=G7 barriers=none fua_is_boundary=true segments=12+1 states=4097 closed_form=4097 violations=2046 missed=0 false_alarm=0 file_read=3 journal_differing=0`，门槛写死 2050 与 0；登记第四节第 4 条预言 false，预期内。
违例、不匹配、歧义、失败类的整数计数：`width_mismatches=0`。

## Q142.46（N19M，20 格）与 Q142.49、Q142.47 crates 格：比对器原样输出（`research/results/e142-r19-comparisons-2026-09-28-r2.out`）

```
name=extract triples=5 kinds=5 full_three_state=1 tearable=1
name=q46 arm=N19M path=mkfs field=operations model=23 pinned=23 literal_equal=true
name=q46 arm=N19M path=mkfs field=segments model=12+1+1+1+4 pinned=12+1+1+1+4 literal_equal=true
name=q46 arm=N19M path=mkfs field=closed_form model=4114 pinned=4114 literal_equal=true
name=q46 arm=N19M path=mkfs field=kinds model=[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2] pinned=[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_
name=q46 arm=N19M path=instance_acquisition field=operations model=2 pinned=2 literal_equal=true
name=q46 arm=N19M path=instance_acquisition field=segments model=2 pinned=2 literal_equal=true
name=q46 arm=N19M path=instance_acquisition field=closed_form model=4 pinned=4 literal_equal=true
name=q46 arm=N19M path=instance_acquisition field=kinds model=[system_configuration_slot×2] pinned=[system_configuration_slot×2] literal_equal=true
name=q46 arm=N19M path=warm_up field=operations model=20 pinned=20 literal_equal=true
name=q46 arm=N19M path=warm_up field=segments model=2+1+2+2+1+2 pinned=2+1+2+2+1+2 literal_equal=true
name=q46 arm=N19M path=warm_up field=closed_form model=15 pinned=15 literal_equal=true
name=q46 arm=N19M path=warm_up field=kinds model=[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2] pinned=[journal_record×2,bar
name=q46 arm=N19M path=transaction field=operations model=35 pinned=35 literal_equal=true
name=q46 arm=N19M path=transaction field=segments model=24+2+1+2 pinned=24+2+1+2 literal_equal=true
name=q46 arm=N19M path=transaction field=closed_form model=16777223 pinned=16777223 literal_equal=true
name=q46 arm=N19M path=transaction field=kinds model=[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2] pinned=[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[sys
name=q46 arm=N19M path=post_mkfs_stream field=operations model=57 pinned=57 literal_equal=true
name=q46 arm=N19M path=post_mkfs_stream field=segments model=2+2+1+2+2+1+2+24+2+1+2 pinned=2+2+1+2+2+1+2+24+2+1+2 literal_equal=true
name=q46 arm=N19M path=post_mkfs_stream field=closed_form model=16777240 pinned=16777240 literal_equal=true
name=q46 arm=N19M path=post_mkfs_stream field=kinds model=[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuratio
name=q47_crates arm=N19M path=post_mkfs_stream field=closed_form_three_state model=16777260 pinned=16777260 equal=true
name=q47_crates arm=N19M path=post_mkfs_stream field=in_place_kinds model=[system_configuration_slot×8] pinned=[system_configuration_slot×8] equal=true
name=q49 item=i_before_window_operations model=45 crates=45 equal=true
name=q49 item=i_before_window_writes model=31 crates=31 equal=true
name=q49 item=i_before_window_last_five model=device0@16781312+4096,device1@16781312+4096,device0@7340032+512,device0@0+4096,device1@0+4096 crates=device0@16781312+4096,device1@16781312+4096,device0@7340032+512,device0@0+4096,device1@0+4096 equal=true
name=q49 item=ii_transaction_operations model=35 crates=35 equal=true
name=q49 item=ii_transaction_writes model=29 crates=29 equal=true
name=q49 item=ii_transaction_barrier_steps model=6 crates=6 equal=true
name=q49 item=iii_window_segments model=24+2+1+2 crates=24+2+1+2 equal=true
name=row_counts segments=5 segments_three_state=5 segments_sensitivity=13 segments_paths_ok=true three_state_paths_ok=true
```

（行长截到 260 字符；全文在产物里。）Q142.58（臂 N18 对 step_five，只报不判）：20 格里 `literal_equal=true` 10 格，与上一段相同。

## 控制（`research/results/e142-r19-controls-2026-09-28-r3.out` 原样）

```
# PC-c
picked_line=390 steps=[29, 30] region_offset=6 old_byte=02 new_byte=03
synthetic_sha256=c7a807ae88685d502ba3ab7a6345a5ec2a2a2348e855ead80dd00a895e8ae7f7
E7RESULT name=old_new_independent_summary new_regions=29 old_regions=29 matched=29 changed=1 unchanged=28 unmatched_new=0 unmatched_old=0 mapped_regions=0 unmapped_regions=1
E7RESULT name=old_new_independent_summary new_regions=29 old_regions=29 matched=29 changed=0 unchanged=29 unmatched_new=0 unmatched_old=0 mapped_regions=0 unmapped_regions=0
# PC-a PC-b P2 self-proof
name=pc_a_i arm=N19M real_extract="triples=5 kinds=5 full_three_state=1 tearable=1" differing_fields=3 post_mkfs_stream.closed_form post_mkfs_stream.closed_form_three_state transaction.kinds ok=true
name=pc_a_ii arm=N19M self_cells=22 self_equal=22 mutated_equal=19 mutated_unequal=path=transaction field=kinds;path=post_mkfs_stream field=closed_form;path=post_mkfs_stream field=closed_form_three_state; ok=true
name=pc_a_i arm=N18 real_extract="triples=5 kinds=5 full_three_state=1 tearable=1" differing_fields=3 post_mkfs_stream.closed_form post_mkfs_stream.closed_form_three_state transaction.kinds ok=true
name=pc_a_ii arm=N18 self_cells=20 self_equal=20 mutated_equal=18 mutated_unequal=path=transaction field=kinds;path=post_mkfs_stream field=closed_form; three_state_cell=absent_in_this_arm ok=true
name=pc_b arm=N19M copy_sha256=f7de0f9043e066198215febd3beb177cb67ae1279060e917df287f10a5fa0a72 deleted=2 added=2 segments_transaction_lines=2 device_region_bytes_lines=2 e7input_lines=0 ok=true
name=pc_b arm=N18 copy_sha256=c161ab2a5f03bb2f698ad4e5734132fb71e7b196691bad6cfeec35d2d1c31cb3 deleted=2 added=2 segments_transaction_lines=2 device_region_bytes_lines=2 e7input_lines=0 ok=true
name=p2 arm=e142-new-pool-file-creation-dry-run-2026-09-28-r19-main-2026-09-28.out 4 rows; ok_rows=4
name=p2 arm=e142-new-pool-file-creation-dry-run-2026-09-28-r19-arm-n18.out 4 rows; ok_rows=4
name=self_proof n=1 unequal_cells=2 name=q57 point=G10a path=transaction field=segments;name=q57 point=G10a path=post_mkfs_stream field=segments;
name=self_proof n=2 unequal_cells=2 name=q48 arm=N19M path=mkfs field=in_place_per_segment;name=q47 arm=N19M path=mkfs field=in_place_per_segment;
name=self_proof n=3 unequal_cells=1 name=q60 point=G11 path=post_mkfs_stream field=segments;
name=self_proof n=4 unequal_cells=4 name=q48 arm=N19M path=post_mkfs_stream field=segments;name=q48 arm=N19M path=post_mkfs_stream field=closed_form;name=q47 arm=N19M path=post_mkfs_stream field=closed_form_three_state;name=c12 arm=N19M row=layer0 field=segments;
name=self_proof n=0 unequal_cells=0 
```

V19d 没触发。合成文件都在 `/tmp/claude-1000/e142-r19-runner4/controls/`。

## Q142.51（6 × 4）

| 字段 | 第十八次（`-r18-main-2` 第 714 行） | 臂 N18（第 714 行） | N19C（`-r19-main.out` 第 732 行） | N19M（主产物第 732 行） |
|---|---|---|---|---|
| r18_system_configuration_fields_ok | true | true | true | true |
| r18_root_flags_ok | true | true | true | true |
| r18_g8_ok | true | true | true | true |
| r18_g9_ok | true | true | true | true |
| control_outcome_matrix_ok | false | false | false | false |
| g7_states_ok | false | false | false | false |

四行 `name=verdict` 整行逐字相同（上面 4c 节与上一段报告的原样）。F19d、F19f 与上一段相同：两个 `false` 都「与第十八次相同」。

## Q142.52–Q142.54

- D_crates 沿用上一段（`research/results/e142-r19-diff-r18-to-arm-n18-2026-09-28.txt`，6 组，归 B3a-2 / C577，见上一段报告与实验页）。
- D_model：`python3 /tmp/claude-1000/e142-r19-runner4/judge/groups.py <diff>` 对新旧两份 D_model 各跑一次，输出 `diff` 为空（都是 `groups=34`，每组删 / 加行数相同）；归因同上一段，另在暖机、整条两行 `segments`、`warm_up`、`before_window_summary side=model`、`header311_write_digest` 的 `index=` 上加 ⑫（条款依据 D13（验证路线） 已定项 4 定案末句）。
- D_total：`python3 /tmp/claude-1000/e142-r19-runner4/judge/d_total_labels.py <D_total> <D_crates> <D_model>` 原样：`name=d_total_labels total_lines=192 D_crates=24 D_model=168 both=0 NOT_FOUND=0`（V19f 没触发）。
- Q142.55：`E7RESULT name=old_new_independent_summary new_regions=29 old_regions=29 matched=29 changed=0 unchanged=29 unmatched_new=0 unmatched_old=0 mapped_regions=0 unmapped_regions=0`。

## Q142.50 与复跑

- 主产物落盘后立即 `python3 research/scripts/admission.py experiment <仓根> E142` 退 77，stderr 原样「✗ 拒绝开跑 E142：输入自上次产物以来没变，不必重跑——research/results/e142-new-pool-file-creation-dry-run-2026-09-28-r19-main-2026-09-28.out 头上的输入指纹与今天的相同（11551b1e15cf）」。
- `research/scripts/replay.sh` 第 171 行改成 `E142|@driver_e142||e142-new-pool-file-creation-dry-run-2026-09-28-r19-main-2026-09-28.out|exact`。`bash research/scripts/replay.sh E142` 退 0，末段原样「字节一致 0 ／ 仅计时不同 0 ／ 对不上 0 ／ 跑不了 0 ／ 结论断言不中 0 ／ 产物已归档 0 ／ 输入没变没跑 1」。这一段**没有**另起一棵树逐字节复现主产物（模型准入对今天的输入退 77，不设破闸变量就跑不了第二次）。

## 门禁格（`--check` 单格；登记「交回之前」第 3 条要 52、69 两道，今天并进 32、34 号）

```
32 segment-registry exit=1      ✗ 段序列登记表与 E142 产物的 name=segments 行（或第二条流与钉它的用例）对不上，共 5 处
34 index-sync exit=0            ✓ 实验索引行与正文标题一致（索引 162 行、正文 162 份）
34 verdict-false-named exit=0   ✓ 登记产物里判决字段是 false 的都在对应实验页里点了名（点名 15 个）
34 quoted-result-lines exit=0     ✓ 这次改动新增或改写的 kb 正文里整行抄的产物行都在产物里逐字找得到（判了 98 行，对照 158 份登记着的产物）  ——改页前红在实验页三行抄的是 N19C 的旧行，已改成新主产物的行
34 evidence-in-repo exit=1      E142 只剩 `research/prompts/e142-r19-runner-report.md:148` 的 /tmp 引用（第一段报告，不是这一段写的）；E142 的指纹那两行已不在红项里
34 decision-links exit=1        只点 E159「没回看 1」（不是这一段）；E142 0 行（改页前一度报「E142 正文提到 D6」，是我写的「D6（…」被认成决策引用，已改写）
34 results-cited exit=1         只点 E162 与 gate69-refresh 那几份；E142 0 行
```
段序列 5 处（`python3 research/scripts/check-segment-registry.py --root .` 原样首行同上）是预期的：`layout/01-first-txn.md` 第八节没改。

## `layout/01-first-txn.md` 第八节照新产物应当写成什么（没改，交主 agent 派书记员）

四行原样（主产物第 35、37、38、39 行；52 号比的就是这几行）：

```
E7RESULT name=segments path=mkfs operations=23 segments=12+1+1+1+4 closed_form=4114 kinds=[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]
E7RESULT name=segments path=warm_up operations=20 segments=2+1+2+2+1+2 closed_form=15 kinds=[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]
E7RESULT name=segments path=transaction operations=35 segments=24+2+1+2 closed_form=16777223 kinds=[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]
E7RESULT name=segments path=post_mkfs_stream operations=57 segments=2+2+1+2+2+1+2+24+2+1+2 closed_form=16777240 kinds=[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]
```

- **mkfs 种根**：23 次操作、`12+1+1+1+4`、4114，种类串两处 `barrier` → `barrier×2`（与 `crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs` 的 `recorded_stream_matches_the_registered_mkfs_segment_sequence` 钉的逐字相同，同上一段报告）。
- **空发布（暖机）**：20 次操作、`2+1+2+2+1+2`、15，种类串照第 37 行（kb 今天写的是 `[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2]`）。
- **普通发布（新池新建文件）**：35 次操作（kb 今天 31）、`24+2+1+2`、16777223，种类串照第 38 行（末段多 `barrier×2`）。
- **整条流**：57 次操作、`2+2+1+2+2+1+2+24+2+1+2`、16777240（kb 今天 `…+26+…`、67108885），种类串照第 39 行；表后「发布与下一次发布之间没有屏障」那一句在发布尾屏障之后不成立。三态 16777260（第 44 行 `name=segments_three_state path=post_mkfs_stream`）。
- 第二条流那一行不来自 E142，52 号比的是 `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` 钉的数组，这一段没量（上一段报告已列今天的数组）。

## 决策格（7b）

- 37 行都回看了、都加了「（第十九次跑第四段）」一句：`D13（验证路线） 已定项 4` 写「2026-09-28 改了」（`grep -n 'E142（' .claude/kb/decisions/13-验证路线.md` 命中第 78、82 行，第 82 行就是这次合并那一条依据，关系维持「支撑」）；其余 36 行写「不受影响」。
- `D16（发布语义） 已定项 7` 维持「备料」（`grep -n 'E142（' .claude/kb/decisions/16-发布语义.md` 只命中第 210、225 行，不在已定项 7）；**这一格该不该升成支撑，仍交主 agent**（上一段已提）。
- `.claude/decision-links-pending` 是空的，② 不适用。

## 实验页、索引、变更史

- `.claude/kb/experiments/142-新池新建文件的干跑.md`：标题状态、「判决」第 16 行、「第十九次跑的段序列、三态与归因」一段（N19M、新产物名、Q142.46/48/49/50/51/55/57、控制、diff）、判决块三行抄的产物行（`warm_up`、暖机与整条 `segments`）与出处、「它答不了的」第 15 条（R40 分叉那句换成 R46 读法与「相同不是独立佐证」）、「影响的决策」37 行、历史节加「2026-09-28（第十九次跑第四段）」。
- `.claude/kb/experiments.md` E142 索引行状态照标题改；`.claude/kb/experiments-history.md` 加一条「2026-09-28：E142（新池新建文件的干跑） 第十九次跑第四段——同一块盘上连着的屏障并成一道」。

## 岔路表（问题单 `research/prompts/e142-r19-questions.md`）

| # | 状态 | 算出它的命令 | 剩下的量能不能让它翻面 |
|---|---|---|---|
| 1 段数、写数、两态与三态闭式 | **已够判**：Q142.46 20 格全同、Q142.47 与 `crates/` 同、Q142.48 锚点全等、Q142.49 7 项全同、Q142.50 退 77、Q142.57/60/61 全等；PC-a、PC-d（判别力自证 ①–④）过；R41 行数齐（5/5/13） | `compare_crates.py compare …`、`q49.py`、`judge.py`（输出都在 `research/results/e142-r19-comparisons-2026-09-28-r2.out`） | 不能：剩下的只有第二段层 0 枚举，不在第 1 行够判条件里 |
| 2 六个判决字段 | **已够判**（照新产物复核）：四列逐字相同 | `grep -n name=verdict` 四份产物 | 不能 |
| 3 与第十八次逐行比、各因哪一批 | **已够判**（照新产物复核）：D_model 34 组分组与行数同上一段、D_total 192 行全标上（NOT_FOUND 0）；PC-b、PC-c 过；Q142.55 `changed=0` | `groups.py`、`d_total_labels.py` | 不能 |
| 第二段（R19C-8 层 0 主臂） | 未跑：重型，等主 agent 问用户 | — | 不补三行任何一行 |

## 删了什么 / 留着什么

- `replay.sh` 建的输出目录 `/tmp/singlefs-replay-1101661` 已删。草稿目录 `/tmp/claude-1000/e142-r19-runner4/`（约 11M）里没有编译目录与仓副本；脚本、锚点、比对与门禁日志留给主 agent 核。`mutate.sh` 的副本与编译目录由它自己收。

## 没做什么

- 第二段层 0 主臂枚举（重型）没跑；`layout/01-first-txn.md` 第八节没改（不在写范围）；决策文件没改（D16 已定项 7 升不升支撑交主 agent）。
- 没另起一棵树逐字节复现主产物；第二台没用。
- 门禁只跑了上面 7 格单格，没跑整轮、没跑 55 号；没提交；没改问题单状态。
- 没判这个实验的结论能不能推翻或确立决策。
- 这一段的新产物都已被实验页点名（results-cited E142 0 行）。
