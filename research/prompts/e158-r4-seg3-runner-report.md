# E158（择根与修复四岔路） 第 4 次跑第三段：执行员报告（2026-09-27）

## 一、结论

- 第三段（`r4-seg3` = H1f + H1g，G0，主族 12 条臂）全部跑完，12 份臂产物加 1 份 `r4-compare`，都以 `name=done` 收尾，没有一份打 `name=stop`。每条命令退出码 0，内存包装没撞顶。从开跑到最后一条臂跑完约 4 分 20 秒。
- 开跑检查全过：`r4_local_constant_check` 与 `r4_arm_code_check` 没有一行不是 `verdict=pass`，7.2 锚点 168/168 行 pass，含每臂「H1f 前缀数 8」「H1g 格数 32」「h1g 每种时长的格数 W 32」三行。
- **丢写格**（`r4-compare` 的 `r4_loss` 行，G0）：
  - H1f 上 -配置 与 -配置续 的 8 条臂（甲、乙、乙-窄读、丙 各两条）都是 0。
  - **H1g 上 -配置 四臂（jia-cfg、yi-cfg、yi-narrow-cfg、bing-cfg）各 16/32，-配置续 四臂各 0/32。** 丢的 16 格全在 a = 2 上：连着两次取号崩溃之后，装置解出的 N-配置 判假（`device_n_cfg=false`），被测挂载交回 K0，回滚。
  - 今天：H1f-cuts 96/4064，H1f-tail 128/128，H1g 32/32。
  - -槽 三臂：H1f-cuts jia-slot 48、yi-slot 64、yi-narrow-slot 64；H1f-tail 64、128、128；H1g 各 16。
- 这一段新出的对比只有一处：**H1g 把 -配置 与 -配置续 分开了，H1f 没分开。** H1f 每一格上装置都判 N-配置 为真（每臂 `n_cfg` 格数 = 格数），所以 H1f 对这两种 N 的判别力，和 H1d 一样落不到它们的差别上。
- 登记推的 ② 成立，F7 没触发：H1g(a = 2) 上 jia-cfg 交回 K0，16/16，没有拒。
- F16 在 H1f 上没触发：yi-cfg 与 yi-cfg-carry 没有被切断的 T1、O1only 格全部 `Ok`，所选根就是被藏那条根（各 32 格）。
  - yi-narrow-cfg 与 yi-narrow-cfg-carry 在 `reads_zeros`、`b=records` 的 T1、O1 格上 K3（cuts 各 8 + 8 格，tail 32 格）。主 agent 认定第 2 项已经认了这一形，照量。F16 点名的是 乙-配置 与 乙-配置续，不含窄读。
- **M3 的臂那一半**：抓到 1、无效 0、没红 0。
  - 基线是产物 jia-cfg H1g 的 a = 1 16 格，全是 K3，不丢写。
  - 改坏之后同 16 格全变 K0，而且回滚；32 格丢写 32。
- 推翻条件：
  - 另跑一遍 `r4-seg3`，`r4_loss` 任一行的数不同（装置是确定性的，同一二进制应逐字节相同）；
  - 在 H1g(a = 2) 的产物行里找到 -配置 四臂有 `device_n_cfg=true` 的格；
  - 或者 -配置续 四臂 H1g 出现 `lost_write_cell=true` 的格。

## 二、跑了什么、怎么跑的

- 负载：开跑前`ps` 看到别的会话在跑 `gate.sh`、几条 `cargo test` / `clippy` / `check`，没有性能测量（qemu、vm-bench、e152、fio）。照常 `nice -n 19` 跑。
  - jia-slot 那条的包装在账上排了 5 秒才起跑（`stderr-jia-slot.log`：「slice 已占 31.8 GiB……排了 5 秒，起跑」）。
- 每条臂的命令照派发提示：进 `arms/<臂>`，带三个环境变量，`capped.sh 3` 套 `run-with-memory-cap.sh 10G`，跑 `./target/release/e158_root_choice_repair r4-seg3`，输出重定向到 `research/results/e158-root-choice-repair-2026-09-27-r4-seg3-<臂>.out`（`noclobber`）。
  - 装置单线程（源码里没有线程变量）。三条道并行，每道依次跑 4 条臂，同时在跑的装置进程不超过 3 个。
  - 脚本 `/tmp/claude-1000/e158-r4-seg3/run_lane.sh`，进度 `progress.md`。
- compare：在 today 副本上跑 `r4-compare research/results/e158-root-choice-repair-2026-09-27-r4-seg3`（传绝对前缀），写 `…-r4-seg3-compare.out`，rc=0，254 行。
- 副本里的 bin 源码 sha256 前 16 位全是 `37a8a312cf9c2d3b`，与主工作区那份 bin 相同（12 份逐个核过）。

| 臂 | 耗时（秒） | 行数 | sha256 前 16 位 |
|---|---|---|---|
| today | 46 | 4427 | 7febe18859ffe0f6 |
| jia-cfg | 4 | 459 | 233e700668450c28 |
| jia-cfg-carry | 4 | 459 | 0bf26350b420276f |
| jia-slot | 30 | 2443 | 2e06d5e3aed70338 |
| yi-cfg | 63 | 4621 | 3ecf24084bedafca |
| yi-cfg-carry | 66 | 4621 | 679662a9a4184c75 |
| yi-slot | 85 | 6605 | 20f8ce1f9bad3296 |
| yi-narrow-cfg | 48 | 3629 | ef604499588d4de0 |
| yi-narrow-cfg-carry | 39 | 3629 | a507b63adbc96038 |
| yi-narrow-slot | 106 | 6605 | c074f1d84fc3cc71 |
| bing-cfg | 29 | 2443 | 404f0509dcdffadd |
| bing-cfg-carry | 27 | 2443 | 42ba4721e1748824 |
| compare | — | 254 | d434a0b37a483413 |

- 完成标记：每份末行 `E7RESULT name=done emitted=<行数−1>`（例：today `E7RESULT name=done emitted=4426`，compare `E7RESULT name=done emitted=253`）。
- 单测数：`grep -c '#\[test\]' crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 输出 `82`。这一段没改装置，单测没重跑，读数是装置执行员那一趟的。

## 三、丢写（产物整行抄，`e158-root-choice-repair-2026-09-27-r4-seg3-compare.out`）

命令：`grep -n 'name=r4_loss' research/results/e158-root-choice-repair-2026-09-27-r4-seg3-compare.out`，原样输出（36 行）：

```
133:E7RESULT name=r4_loss arm=bing-cfg family=h1f-cuts geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=2080 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=2080
134:E7RESULT name=r4_loss arm=bing-cfg family=h1f-tail geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=128 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=128
135:E7RESULT name=r4_loss arm=bing-cfg family=h1g geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 lost_write_cells=16 q1_overwrite_cells=16 q2_rollback_cells=16 q3_lost_cells=0 q3c_lost_cells=8 q3_last_confirmed_cells=32
136:E7RESULT name=r4_loss arm=bing-cfg-carry family=h1f-cuts geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=2080 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=2080
137:E7RESULT name=r4_loss arm=bing-cfg-carry family=h1f-tail geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=128 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=128
138:E7RESULT name=r4_loss arm=bing-cfg-carry family=h1g geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=32
139:E7RESULT name=r4_loss arm=jia-cfg family=h1f-cuts geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=96 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=96
140:E7RESULT name=r4_loss arm=jia-cfg family=h1f-tail geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=128 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=128
141:E7RESULT name=r4_loss arm=jia-cfg family=h1g geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 lost_write_cells=16 q1_overwrite_cells=16 q2_rollback_cells=16 q3_lost_cells=0 q3c_lost_cells=8 q3_last_confirmed_cells=32
142:E7RESULT name=r4_loss arm=jia-cfg-carry family=h1f-cuts geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=96 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=96
143:E7RESULT name=r4_loss arm=jia-cfg-carry family=h1f-tail geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=128 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=128
144:E7RESULT name=r4_loss arm=jia-cfg-carry family=h1g geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=32
145:E7RESULT name=r4_loss arm=jia-slot family=h1f-cuts geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=2080 lost_write_cells=48 q1_overwrite_cells=0 q2_rollback_cells=48 q3_lost_cells=48 q3c_lost_cells=0 q3_last_confirmed_cells=2032
146:E7RESULT name=r4_loss arm=jia-slot family=h1f-tail geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=128 lost_write_cells=64 q1_overwrite_cells=64 q2_rollback_cells=64 q3_lost_cells=0 q3c_lost_cells=32 q3_last_confirmed_cells=128
147:E7RESULT name=r4_loss arm=jia-slot family=h1g geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 lost_write_cells=16 q1_overwrite_cells=16 q2_rollback_cells=16 q3_lost_cells=0 q3c_lost_cells=8 q3_last_confirmed_cells=32
148:E7RESULT name=r4_loss arm=today family=h1f-cuts geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=4064 lost_write_cells=96 q1_overwrite_cells=0 q2_rollback_cells=96 q3_lost_cells=96 q3c_lost_cells=0 q3_last_confirmed_cells=3968
149:E7RESULT name=r4_loss arm=today family=h1f-tail geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=128 lost_write_cells=128 q1_overwrite_cells=128 q2_rollback_cells=128 q3_lost_cells=0 q3c_lost_cells=64 q3_last_confirmed_cells=128
150:E7RESULT name=r4_loss arm=today family=h1g geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 lost_write_cells=32 q1_overwrite_cells=32 q2_rollback_cells=32 q3_lost_cells=0 q3c_lost_cells=16 q3_last_confirmed_cells=32
151:E7RESULT name=r4_loss arm=yi-cfg family=h1f-cuts geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=4096 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=4096
152:E7RESULT name=r4_loss arm=yi-cfg family=h1f-tail geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=256 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=256
153:E7RESULT name=r4_loss arm=yi-cfg family=h1g geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 lost_write_cells=16 q1_overwrite_cells=16 q2_rollback_cells=16 q3_lost_cells=0 q3c_lost_cells=8 q3_last_confirmed_cells=32
154:E7RESULT name=r4_loss arm=yi-cfg-carry family=h1f-cuts geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=4096 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=4096
155:E7RESULT name=r4_loss arm=yi-cfg-carry family=h1f-tail geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=256 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=256
156:E7RESULT name=r4_loss arm=yi-cfg-carry family=h1g geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=32
157:E7RESULT name=r4_loss arm=yi-narrow-cfg family=h1f-cuts geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=3104 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=3104
158:E7RESULT name=r4_loss arm=yi-narrow-cfg family=h1f-tail geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=256 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=256
159:E7RESULT name=r4_loss arm=yi-narrow-cfg family=h1g geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 lost_write_cells=16 q1_overwrite_cells=16 q2_rollback_cells=16 q3_lost_cells=0 q3c_lost_cells=8 q3_last_confirmed_cells=32
160:E7RESULT name=r4_loss arm=yi-narrow-cfg-carry family=h1f-cuts geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=3104 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=3104
161:E7RESULT name=r4_loss arm=yi-narrow-cfg-carry family=h1f-tail geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=256 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=256
162:E7RESULT name=r4_loss arm=yi-narrow-cfg-carry family=h1g geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=32
163:E7RESULT name=r4_loss arm=yi-narrow-slot family=h1f-cuts geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=6080 lost_write_cells=64 q1_overwrite_cells=0 q2_rollback_cells=64 q3_lost_cells=64 q3c_lost_cells=0 q3_last_confirmed_cells=6016
164:E7RESULT name=r4_loss arm=yi-narrow-slot family=h1f-tail geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=256 lost_write_cells=128 q1_overwrite_cells=128 q2_rollback_cells=128 q3_lost_cells=0 q3c_lost_cells=64 q3_last_confirmed_cells=256
165:E7RESULT name=r4_loss arm=yi-narrow-slot family=h1g geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 lost_write_cells=16 q1_overwrite_cells=16 q2_rollback_cells=16 q3_lost_cells=0 q3c_lost_cells=8 q3_last_confirmed_cells=32
166:E7RESULT name=r4_loss arm=yi-slot family=h1f-cuts geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=6080 lost_write_cells=64 q1_overwrite_cells=0 q2_rollback_cells=64 q3_lost_cells=64 q3c_lost_cells=0 q3_last_confirmed_cells=6016
167:E7RESULT name=r4_loss arm=yi-slot family=h1f-tail geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=256 lost_write_cells=128 q1_overwrite_cells=128 q2_rollback_cells=128 q3_lost_cells=0 q3c_lost_cells=64 q3_last_confirmed_cells=256
168:E7RESULT name=r4_loss arm=yi-slot family=h1g geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 lost_write_cells=16 q1_overwrite_cells=16 q2_rollback_cells=16 q3_lost_cells=0 q3c_lost_cells=8 q3_last_confirmed_cells=32
```

## 四、H1g 按 a 分开（-配置 与 -配置续 八臂）

命令（每臂一遍）：`grep 'name=r4_cell family=h1g ' research/results/e158-root-choice-repair-2026-09-27-r4-seg3-<臂>.out | awk '{…取 a、a_class、device_n_cfg、q2_rollback、lost_write_cell…}' | sort | uniq -c`（完整脚本见草稿目录 `h1g-by-a.txt` 生成的那条；输出原样）：

```
     16 jia-cfg a=1 a_class=K3 device_n_cfg=true q2_rollback=false lost_write_cell=false
     16 jia-cfg a=2 a_class=K0 device_n_cfg=false q2_rollback=true lost_write_cell=true
     16 jia-cfg-carry a=1 a_class=K3 device_n_cfg=true q2_rollback=false lost_write_cell=false
     16 jia-cfg-carry a=2 a_class=K3 device_n_cfg=true q2_rollback=false lost_write_cell=false
     16 yi-cfg a=1 a_class=K3 device_n_cfg=true q2_rollback=false lost_write_cell=false
     16 yi-cfg a=2 a_class=K0 device_n_cfg=false q2_rollback=true lost_write_cell=true
     16 yi-cfg-carry a=1 a_class=K3 device_n_cfg=true q2_rollback=false lost_write_cell=false
     16 yi-cfg-carry a=2 a_class=K3 device_n_cfg=true q2_rollback=false lost_write_cell=false
     16 yi-narrow-cfg a=1 a_class=K3 device_n_cfg=true q2_rollback=false lost_write_cell=false
     16 yi-narrow-cfg a=2 a_class=K0 device_n_cfg=false q2_rollback=true lost_write_cell=true
     16 yi-narrow-cfg-carry a=1 a_class=K3 device_n_cfg=true q2_rollback=false lost_write_cell=false
     16 yi-narrow-cfg-carry a=2 a_class=K3 device_n_cfg=true q2_rollback=false lost_write_cell=false
     16 bing-cfg a=1 a_class=K3 device_n_cfg=true q2_rollback=false lost_write_cell=false
     16 bing-cfg a=2 a_class=K0 device_n_cfg=false q2_rollback=true lost_write_cell=true
     16 bing-cfg-carry a=1 a_class=K3 device_n_cfg=true q2_rollback=false lost_write_cell=false
     16 bing-cfg-carry a=2 a_class=K3 device_n_cfg=true q2_rollback=false lost_write_cell=false
```

- -配置 四臂在 a = 2 上 `device_n_cfg=false`：两次取号都写 tail = 0，把系统配置两槽都盖成 0，N-配置 读不出更新的那条根。四臂于是照今天那样交回 K0、回滚。
- -配置续 四臂在 a = 2 上 `device_n_cfg=true`：取号时续写了见证的计数器，照样拒（K3）。bing-cfg-carry 的拒绝成员是 `R3RebuildFromRecordsStoppedBelowTheWitness`，别的三臂是 `R3NewerRootWitnessedByTheSystemConfiguration`（`r4_family_summary` 行的 `tested_classes`）。
- -槽 三臂在 H1g 上丢的 16 格全在 `reads_zeros` 上（a = 1、a = 2 各 8 格，`device_n_slot=false`），与登记推的 ④ 同向。

## 五、产物齐不齐（4b）

- 每臂每族的格数、N_真 为真的格数、装置判 N-配置 / N-槽 为真的格数，以及缺 `hidden` / `q0_destination` 字段的格数，命令：`awk '/name=r4_cell /{…按 family 累计 truth_newer、device_n_cfg、device_n_slot…}' <产物>`（草稿目录里跑的）。结果：
  - 12 臂 × 3 族（h1f-cuts、h1f-tail、h1g）都有格，缺字段 0。每一格 `truth_newer=true`，所以每一族都造出了被测形状。
  - H1g 每臂 32 格，duration 全为 W（登记 5.3「时长 W」）。
  - H1f-tail：乙 各臂 256 格（W、T1 各 128），别的臂 128 格（只有 W）；按装置的 `FourthRunDurationPlan::OfTheArm`，乙 多一档时长。
  - H1f-cuts 的格数随被测挂载的写数变：jia-cfg / jia-cfg-carry 96，jia-slot / bing 两臂 2080，yi-narrow 两条 -配置 3104，today 4064，yi-cfg / yi-cfg-carry 4096，yi-slot / yi-narrow-slot 6080。
  - `device_n_cfg` 为真的格数：H1f 两族在 12 臂上都等于格数；H1g 上 -配置续 四臂 32，其余 16。
- `r4_family_summary` 的检查字段，12 臂 × 3 族全是 0：`void_cells_v2`、`fault_not_reached_cells`、`s4_mismatches`、`f4_failures`、`f6_findings`、`f13_failures`、`f15_findings`、`formula_mismatches`。f13 只在 today 上查了（h1f-cuts 96、h1f-tail 32、h1g 16 格）。f6_checked、f15_checked 在这三族上都是 0：登记只让 F6、F15 核 H1d / H1e 与 H-随 的格。
- `r4_h1g_acquisition_crash`：48 行，72 次崩溃核对都是 `holds=true`（`grep -h … | grep -o 'holds=[a-z]*' | sort | uniq -c` 输出 `     72 holds=true`）。
- `r4_today_base_check`：192 行，`ledger_roots_missing_on_the_image=0` 192 行。compare 的 `r4_base_identity`：12 臂都是 `prefixes=16 differing=0 verdict=pass`。
- 7.3：compare 第 112–119 行 row=2 两行 `mismatches=0`，row=3 六行 `differences=0`，第 14 行 row=4 `nonzero_cells=0`，全是 pass。F11：`E7RESULT name=r4_f11 cells=112 cells_differing_outside_o1_and_cell_22=0 verdict=inference_holds`。

## 六、判决行（4c）

- `grep -c 'name=verdict'` 在这一段 13 份产物上全是 0：装置不打 `name=verdict` 行，判定写在各行的 `verdict=` 字段里。
- 按取值数（`grep -o 'verdict=[a-z_]*' <产物> | sort | uniq -c`）：
  - 12 份臂产物只有 `verdict=pass`：today、jia 三臂、bing 两臂各 153 行，乙 与 乙-窄读 六臂各 185 行。
  - compare 有 33 pass、1 `inference_holds`、12 `not_constructible`。
- 点名的 12 行「没过」，都在 `e158-root-choice-repair-2026-09-27-r4-seg3-compare.out`：
  - 第 100–110 行：11 条非今天臂各一行 `control=PC-extra-refusal sentence=today_ok_on_the_cell tag=today verdict=not_constructible n1=1,2,3`。
  - 第 111 行：`arm=today control=PC-22 sentence=an_m_with_ok_zero_count_zero_isolation tag=today verdict=not_constructible route=F2`。
  - 为什么：PC-多拒 与 PC-22 的格属于第四段（L 族），这一段的产物里没有，compare 在只给 seg3 前缀时照样把这两条对照的句子算一遍，找不到格就记 `not_constructible`。这不是登记里对第三段的预期结局，而是 compare 只喂了第三段的结果。两句都是 [今] 标签，按 V1 ⑤ 不作废任何量。第 253 行 `r4_missed_arm_and_today_sentences count=12` 就是这 12 句。
- 违例、不匹配、歧义、失败类的整数计数全是 0：`r4_family_summary` 的 s4 / f4 / f6 / f13 / f15 / formula 各栏（第五节）、compare 的 `mismatches`、`differences`、`differing`、`nonzero_cells`、`cells_differing_outside_o1_and_cell_22`。
- `r4_void`：84 行都是 `voided=false basis=none`。**注意**：这一段的产物里没有阳性对照（PC-N-环、PC-N-盖 等在第一段），V1 在这里只拿第三段自己的句子算。跨段的 V1（第一段的阳性对照作废了哪几臂的 Q1–Q3，从而作废这一段的 H1f / H1g 丢写）要对全部段一起跑一次 `r4-compare`，这一段没跑。

## 七、变异：M3 的臂那一半

- 表：`research/mutations/e158_r4_arm_mutations.tsv` 的 `M3` 行。臂 jia-cfg，文件 `crates/singlefs-core/src/mount.rs`。把 `.flat_map(|device| { verified_system_configuration_slots(…) })` 换成 `.filter_map(… .into_iter().max_by_key(|slot| slot.quantities.slot_generation))`，即 c_见证 只取每块盘世代号最新的那一槽。取样点 `H1g（a = 1，r4-h1g）`；应当看到「H1g a=1 的被测挂载 甲-配置 由拒（K3）变不拒（K0）」。
- 做法：把 `arms/jia-cfg` 用 `cp -a` 拷到草稿 `mut-M3`（原副本没动），用 python 定点替换，锚点命中 1 次（`anchor hits: 1`）。`diff` 只动了第 3840 行与第 3846 行之后两行。`capped.sh 3 cargo build --release --offline --bin e158_root_choice_repair` 的结果是 build_rc=0、warning 0，耗时 1m 11s。然后照产物的环境变量跑 `r4-h1g`，run_rc=0，末行 `E7RESULT name=done emitted=92`，`r4_arm_code_check … verdict=pass`。
- 结果：基线（产物 jia-cfg）是「16 a=1 a_class=K3 … lost_write_cell=false」「16 a=2 a_class=K0 … lost_write_cell=true」。变异后按同一 awk 数出的原样输出：

```
     16 a=1 a_class=K0 q2=true lost=true
     16 a=2 a_class=K0 q2=true lost=true
```

  family_summary 里 `q2_rollback_cells=32`、`lost_write_cells_q1_q2_q3=32`。**抓到 1 / 无效 0 / 没红 0**，另列的 `💥 ⚠️ ⏱ 🧱` 都是 0（没走 `mutate.sh`：臂表变异改的是 `crates/` 副本，mutate.sh 与 59 号都跑不到）。
- 还原：变异只做在拷贝上，`arms/jia-cfg` 里的锚点原文仍在，基线就是这一段的产物（第三节 jia-cfg 那三行）。`mut-M3` 这份拷贝已删（352M），变异后的输出留在草稿 `mut-M3-r4-h1g-jia-cfg.out`，没有入库：它不是实验产物，是变异的观测；要入库由主 agent 定。
- 登记第九节 M3 的另一半（装置单测：手写两盘两槽，期望 c_见证 = c）是第 3 次跑的装置变异，在 `crates/mutations.tsv` 里，归 59 号，这一段没跑。

## 八、岔路表（岔路单 `research/prompts/c554-fix-forks.md` 第 1 行，登记 11.3 的格式）

这一段只补 H1f、H1g 两族。出局与选哪个由主 agent 按岔路单判，下表只写这一段量到了什么。

| 那一样 | 状态 | 还差什么 | 剩下的量能不能让它翻面 | 算它的命令 |
|---|---|---|---|---|
| 丢写：-配置 四臂（jia-cfg、yi-cfg、yi-narrow-cfg、bing-cfg）在 H1f、H1g | H1f 已量：四臂 cuts、tail 都是 0。H1g 已量：四臂各 16/32（全在 a = 2）。按登记第六节，H1f、H1g 对这几臂算判据；**这一族上丢写 > 0 已经成立** | 这一族不差。实七、实八、H-随（第二段）还没有产物 | 不能：剩下的族只会加格，不会减掉 H1g 上的 16 | `grep -n 'name=r4_loss' research/results/e158-root-choice-repair-2026-09-27-r4-seg3-compare.out`（第 135、141、153、159 行） |
| 丢写：-配置续 四臂（jia-cfg-carry、yi-cfg-carry、yi-narrow-cfg-carry、bing-cfg-carry）在 H1f、H1g | 已量：H1f cuts、tail，H1g，都是 0 | H1d / H1e 对它们不算判据（第六节开头）。实七-甲、实七-乙、实八、H-随 在第二段 | 能：第二段任一族 > 0 就翻 | 同一命令，第 136–138、142–144、154–156、160–162 行 |
| 丢写：今天、-槽 三臂 在 H1f、H1g | 已量：today 96 / 128 / 32；jia-slot 48 / 64 / 16；yi-slot、yi-narrow-slot 64 / 128 / 16（依次是 h1f-cuts、h1f-tail、h1g） | 不差（这几臂在全部族上都算，> 0 已成立） | 不能 | 同一命令，第 145–150、163–168 行 |
| 丢写：跨段作废（V1） | 未判：这一段没有阳性对照，`r4_void` 只拿第三段的句子算，84 行 `voided=false` | 对第一、三段（再加第四段）的前缀一起跑一次 `r4-compare`，看第一段的 PC-N-环 / PC-N-盖 / PC-O / PC-丢写 有没有作废某臂的 Q1–Q3 | 能：某臂的 Q1–Q3 若被第一段的 [测] / [臂] 句作废，这一段那臂的丢写数就不出结论 | `r4-compare <seg1 前缀> <seg3 前缀> …` 的 `name=r4_void quantity=Q1-Q3` 行（没跑） |
| 多拒（Q4） | 不归这一段（第四段） | 第四段 L 族、第二段 H-随全 | 能 | compare 打出的 `r4_q4` 行只按 H1f / H1g 格算（33 行 `extra_refusals=0`），登记没把这两族列为多拒的合法状态族，不当读数 |
| 多读（Q5-同成） | 不归这一段 | 第一、四段（第二段补） | 能 | compare 打出的 52 行 `r4_q5` 取自 H1f / H1g 的故障格，不是第六节 Q5-同成 列的来源族，不当读数 |
| 第 22 条那一格（Q6） | 不归这一段 | 第四段 | 能 | 这一段没有 `r4_q6` 行 |
| 够判条件第二半 | 装置执行员已在快照上核过（登记修订第 3 条） | — | 不能 | 同上 |

岔路单状态栏写着「乙在实七、实八或 H-随上丢写 > 0，就带数再问用户」。这一段量的是 H1f、H1g，不在那三族里。但 **yi-cfg（乙-配置）在 H1g 上丢写 16/32**，yi-cfg-carry（乙-配置续）是 0。用户定的「-配置 一族」同时罩着这两条 N，要不要拿这个数去问用户，交主 agent 定。

## 九、门禁（登记给 experiment-runner 的阶段，主工作区；日志在草稿目录 `gate-*.log`）

这一次不写实验页，所以读实验页的几道也在这一步跑。各阶段的原样末行（截到 250 字）与退出码：

| 阶段 | 退出码 | 末行 | 与这一段的关系 |
|---|---|---|---|
| 33 | 0 | `✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 1132 条的原文各命中源码一次；…` | — |
| 52 | 1 | `再让 research/scripts/replay.sh 里 E142 那一行的入库产物名字跟上。` | 日志里没有 E158 |
| 80 | 0 | `crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs` | — |
| 96 | 0 | `e92_reuse_requirement.rs` | — |
| 27 | 1 | `→ 旧值只许留在「## 历史版本」之后与 *-history.md 里。` | 日志里没有 E158 |
| 34 | 0 | `✓ 实验索引行与正文标题一致（索引 159 行、正文 159 份）` | — |
| 40 | 1 | 末行点名 E160 的旧产物 | 日志点名这一段的 13 份 `r4-seg3` 产物还没被实验页点名，这是预期的（派发要求不写实验页）；另有第一、四段的产物与更早的 E158 产物 |
| 69 | 1 | `原件已经没了，就写明它没了、把还核得动的那部分落进仓里；…` | 日志里没有 E158 |
| 75 | 1 | `支撑 / 推翻的那条分项在「**依据**」段引回这个实验；…` | 日志里没有 E158 |
| 84 | 0 | `✓ 登记产物里判决字段是 false 的都在对应实验页里点了名（点名 3 个）` | — |
| 85 | 0 | `写明「原始输出未留存」不判的 0 页：（没有）` | — |
| 86 | 0 | `扫了 research crates/singlefs-harness/src/bin（target/ 不扫）；不在的目录 0 个：（没有）` | — |
| 88 | 77 | `! 这次改动新增或改写的 kb 正文里没有整行抄的 E7RESULT 行，本阶段无对象可判` | 本次未跑 |
| 99 | 0 | `✓ 登记了路径与共用项的实验页判过了（3 份、14 条路径）；…` | — |

红的几道都没有点名这一段写的文件。40 号点名的 13 份新产物，要等实验页点名才会清；这里写出来交主 agent，另派人去点名。

## 十、没做什么

- 没写实验页、索引行与 `experiments-history.md`；没改 `replay.sh`（派发要求）。这 13 份产物因此还没有 `replay.sh` 登记行，也没被实验页点名（40 号），要主 agent 派人补。
- 没跑跨段的 `r4-compare`（第一、三、四段一起），跨段的 V1 没算（第六节、第八节）。
- 没跑 `mutate.sh`，也没跑 59 号：入库装置那一支归提交时的 59 号；M3 装置单测那一半同样归 59 号。
- 没重跑单测（装置没改）。没跑重型测试，没碰 `crates/`，没提交。
- 没判任何候选出局或胜出，没判这一段的数能不能确立或推翻决策。
- 登记没修订：这一段在产物之前没有要改的判据或臂。

## 十一、草稿目录（`/tmp/claude-1000/e158-r4-seg3/`）

- 删了：`mut-M3`（jia-cfg 臂副本的拷贝，含 release target，删前 `du -sh` 为 352M）。
- 留着的是小文件，没有编译目录，也没有仓副本：
  - `run_lane.sh`、`progress.md`、`rc-*`、`lane-*.rc`、`stderr-*.log`
  - `mut-M3-build.log`、`mut-M3-run.err`、`mut-M3-r4-h1g-jia-cfg.out`（M3 变异后的输出，第七节）
  - `h1g-by-a.txt`、`gate-*.log`、本报告
- `/tmp/claude-1000/e158-r4-device/` 下的臂副本不是我建的，我只读、只执行，没动。
