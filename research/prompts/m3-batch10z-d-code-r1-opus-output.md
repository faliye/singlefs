# 第 10z 批第四块代码轮第一轮：云端攻方（Opus）报告（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 H1 H2 R C P -->

轮名 m3-batch10z-d-code-r1。被判的是冻结副本 `~/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/` 里第四块的改动；引代码写冻结副本的绝对路径与行号，引 kb 写 kb 文件自己的行号。数都是我在冻结副本的拷贝上量的，是副本上的数，不是入库装置上的数。

## 复跑

```
SINGLEFS_FROZEN=$HOME/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree SINGLEFS_TOOLS=/home/fy5090/code/singlefs THREADS=6 \
  bash research/prompts/m3-batch10z-d-code-r1-opus-model/rerun.sh <新工作目录> [a1 a2 a3 a4 a5 a6 x x2]
```

末尾每组一行「summary-<组>.txt 全同」，最后一行「退出码不为 0 的件：0」。崩溃枚举那四段（`out/crash.txt`）不在 rerun.sh 里，单跑：在 rerun.sh 建好的副本里设 `OPUS_CRASH=1 OPUS10ZD_ARM=<h2|today> OPUS_KS=3 OPUS_RKINDS=n OPUS_FAULTS=acct OPUS_ACTIONS=Wss,Wsss,Woss,nWss,u,n,nu,Fu`，经 `run-with-memory-cap.sh 4G` 与 `capped.sh 6` 跑 `opus10zd_h2_borrowed` 那份测试二进制的 `opus_sweep`（每臂约 25 分钟）。

模型目录 `research/prompts/m3-batch10z-d-code-r1-opus-model/` 各文件 sha256：

```
0c07fe509e1b814f255aa2486cb0cb3583ef83429ad944e1b4788e7f97601a55  research/prompts/m3-batch10z-d-code-r1-opus-model/out/crash.txt
e8084ea237638e7f0cc3946217553f2ec35e2a565d687be0c3e3d75e58457d4d  research/prompts/m3-batch10z-d-code-r1-opus-model/out/raw-lines.txt
c95206f130dfa8d24d01d0e1d1c2545264b6540d16f23c327df7a64df4ea5460  research/prompts/m3-batch10z-d-code-r1-opus-model/out/summary-a1.txt
968c739b5dec97680dc946ec66172f0367321d3762be6db601f8a8d6dde1ec03  research/prompts/m3-batch10z-d-code-r1-opus-model/out/summary-a2.txt
8a3726e54e6639015eb5200d8d4ea29903e6b008e0158e3f9120fd27ea80e5b6  research/prompts/m3-batch10z-d-code-r1-opus-model/out/summary-a3.txt
b227f75c4a1fe1819d9ae8f8dca6480431249ef9eacce4ed1b877771720f2d3a  research/prompts/m3-batch10z-d-code-r1-opus-model/out/summary-a4.txt
cfa79daaa2cb288b2dd423eb5c71ab0954a19420182a5b028593d1ea69f9f361  research/prompts/m3-batch10z-d-code-r1-opus-model/out/summary-a5.txt
cc8ea34ac8f07286b966cc7bfe29c1110c74928069ebaf63b2e42df0524ae2fd  research/prompts/m3-batch10z-d-code-r1-opus-model/out/summary-a6.txt
715054866d7ef917b2f7a262c59d08db33999891dcc2aaad64a2258247779a0e  research/prompts/m3-batch10z-d-code-r1-opus-model/out/summary-x1.txt
9dff99fba5b86b191d96b3807f2f488192f930ba169a13cabf603d47fa33e271  research/prompts/m3-batch10z-d-code-r1-opus-model/out/summary-x2.txt
28caf6f13841d1bd9b93ca82dd60279740113f85bf54508dcc12863eb0ce2ff4  research/prompts/m3-batch10z-d-code-r1-opus-model/out/summary-z1.txt
cd9816810d4172c1c987251e644c68ace3d3aa353c47db1c1722eec42b67bb42  research/prompts/m3-batch10z-d-code-r1-opus-model/README.md
c3c929868fa5ae19a8f3129f5f54aa6eee9894964813c9af8200a28b5ca70e82  research/prompts/m3-batch10z-d-code-r1-opus-model/rerun.sh
f1c10b7b1eb64418f51b14ffdacc7a8d6364ac3c730f34cb178b87f20e492f4b  research/prompts/m3-batch10z-d-code-r1-opus-model/scripts/agg.py
05783b786ec5dc4efe0f20d34b7127e119ded83d1530598e6390ac71edb4b723  research/prompts/m3-batch10z-d-code-r1-opus-model/scripts/aggx.py
65256423b782ea10d8e1088b783c2e487850df72bc3cbc679871ddc7234a0f99  research/prompts/m3-batch10z-d-code-r1-opus-model/scripts/batch2.sh
8fbb8cec96a1c23d3941f0a6e827155b202d43942bcf79fcaa5ab4df86897dc6  research/prompts/m3-batch10z-d-code-r1-opus-model/scripts/spec-a1.tsv
433092f7416d36b5c0ca61fee5fa6211d6daa605be5366352d60b10471daa067  research/prompts/m3-batch10z-d-code-r1-opus-model/scripts/spec-a2.tsv
a4db78f7cd128f23e8225319e3d5712845b6df94bfe8616b87b7042ddca42c4f  research/prompts/m3-batch10z-d-code-r1-opus-model/scripts/spec-a3.tsv
9f871952d2d55025d9dc76f9b86aa197939478e6f71ab52dbb4243c456800f37  research/prompts/m3-batch10z-d-code-r1-opus-model/scripts/spec-a4.tsv
0c35fe962ff30dabba2a4cf2211b75f5c0447ac0d17ac75283ac1bb0d6a23c69  research/prompts/m3-batch10z-d-code-r1-opus-model/scripts/spec-a5.tsv
46f383092cdb47cbd5cc81b45ffe87061cd497e912c08915979c90afc90ee34c  research/prompts/m3-batch10z-d-code-r1-opus-model/scripts/spec-a6.tsv
1b835a60809d81c5ab1ae024069eca9702bcdaf6f220658e81789d39cd4e43ba  research/prompts/m3-batch10z-d-code-r1-opus-model/scripts/spec-x2.tsv
9bf4a67e21fde9dc684fde7b3d64f3ba67f5e72e6705c629e0e297a56a8f1fc8  research/prompts/m3-batch10z-d-code-r1-opus-model/scripts/spec-x.tsv
180bf4aa81363a31871ff30cd567a40ccc476dbf6b7df186bd5b49545030dd7f  research/prompts/m3-batch10z-d-code-r1-opus-model/src/opus10zd-arms.patch
dd75afbcda0995831f93819361fc6dadfc8cfe07f0dfa1bd55716625716401ec  research/prompts/m3-batch10z-d-code-r1-opus-model/src/opus10zd-probe/Cargo.toml
78ac9bfaf015bd84f2e7d5732fc4f1d9917e7f7177e09fbe756969c2c6886880  research/prompts/m3-batch10z-d-code-r1-opus-model/src/opus10zd-probe/src/lib.rs
fd6dbec28eca8361efd072872ec314dde0f04d4d5f64a05e072663d44f85bc76  research/prompts/m3-batch10z-d-code-r1-opus-model/src/opus10zd-probe/tests/opus10zd_abandoned_slot.rs
b48603c5ec4470489df769910ef1a103f956732fdccb9ab400468567821ff6bf  research/prompts/m3-batch10z-d-code-r1-opus-model/src/opus10zd-probe/tests/opus10zd_h2_borrowed.rs
```

## 各格判定一览

| 格 | 判定 | 依据（详见各节） |
|---|---|---|
| K2-2 H2 借来的 C 罩不罩得住 R 引用的全部单元 | **打中**（分辨臂） | 挂载那一刻 R 的账读不出、记录齐时，C 在 R 那一槽只有 R 自己写的 22 个落点，没有 R 从 P 继承、之后被会话释放的单元；抬 F 借这一份，那几个单元回收、再分配，池级 checker 判 I-7.4 红。H2 臂随机长历史 53 / 150（k=3）、68 / 150（k=5）段在某一步红，today 臂同样历史 0 段红、0 段被拒 |
| K2-2 差一整圈、那一槽 C 只记着别的发布 | 没打中（推的） | 要水位之上没施加的发布跨过一整圈（24 个 txg），F2 只退到见证到的那一版，造不出 |
| K1-2 `ValidRoot` 只比 txg 不比实例 | 没打中（推的） | 抬 F 那一遍从同一槽读出的就是挂载那一刻那一条，这个进程写掉那一槽时表项换类、C 解开 |
| K1-2 两样都没记着时照今天读盘 | 没打中（读码 + 量过） | `.or_else` 交 `None` 之后走 `placements_referenced_by_root`；today 臂入库用例那一条照旧拒，说明开关接对了 |
| K2-3 挂载那一刻被抛弃根槽读回全零、它的单元谁都不护 | **造出来了**，归 C554，不是 D16 已定的代价 | 入库同一造法，挂载那一刻 C 的根槽读回全零（或报读错），第一次覆盖写就写掉 C 的数据单元，I-7.4 红：每组 61 段里 42–48 段在某一步红 |
| K2-1 H1 不进上限、照记着的落点隔离 | 没打中（一次抽样） | 3732 段历史（之前覆盖写 1/3/5 次 × 实例 3 再写 0/2 次 × 读回全零/报读错 × 用户动作长到 3 的全部串）0 段红、0 段被拒 |
| K1-1 抬 F 那一串每一处读根环 | 没打中（读码） | 准入上限、抬 F 重算两处都滤了；F 生效值那一遍本来不分槽；管理员回退那一遍没有 hunk。checker 档两处自算上限还用没滤的那一份（`on_device_modes.rs`、e158），今天的用法里碰不到 |
| K3 改回去照绿 | **部分打中** | 这一批每一处改回去都有变异行抓；但 H2 用例的 R 是覆盖写、C 罩得住它的全部单元，断言只看「借来的那一份不空闲」——K2-2 那一形在冻结副本上跑这份用例 8 条全绿 |

这条腿自己提的三个改法（`diskfirst`、`cplusreleased`、`cplusmountaccount`）都只在我的模型上量过、被攻过零轮。

## 一、K2-2：H2 借来的 C 罩不住 R 从 P 继承的单元（打中）

### 历史（每一步指到许可它的那一句）

1. 第一个文件之后覆盖写 k 次得 P，再经会话录一次发布 R = 建一个 inode（数据单元不动，R 与 P 共用文件的数据单元），崩在 R 的根槽落盘之后、系统配置轮换之前（入库 `common_unwitnessed_publish` 同一录法，只把 R 的种类放开）。
2. 可写挂载那一刻 R 自己写出的树表、中央映射树根、分配记录树根读回全零（只这一次挂载）：R 走读不成，F2 退回 P；R 的记录齐（一条、带末条），所以 F1 不拒，C 的两份账都不加——
   「这两份账读不出时：那次发布读得出的记录齐（本次发布内序号 1..N 连着、带末条）就不加」（`.claude/kb/decisions/23-journal的角色与格式.md:392`），
   代码「读得出的记录齐（序号 1..N 连着、带末条）就不交」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-core/src/mount.rs:1833`）、「记录齐就不交（交 F1 乙）」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-core/src/mount.rs:1916`）。
   C 在 R 那一槽只剩 R 的记录点名的 22 个落点（R 写的那几个单元与新根段四条指针），R 记成有效根。
3. 挂上之后故障撤掉；会话覆盖写到准入推出第一串抬 F（W，13–15 次），覆盖写释放了 P 的数据单元——R 也引用它们。
4. 抬 F：R 按现行实例表被抛弃、表里没记着它的落点，借 C 那一槽——「照 C 那一圈记着的落点算、不读账、读不出不拒」（`.claude/kb/decisions/16-发布语义.md:37`），
   代码「.placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-core/src/mount.rs:4131`）。
   借来的 22 个落点里没有那几个被释放的数据单元；R 被抛弃、不进「环里最旧有效根」，回收门槛取新 F：「let floor_of_the_reclaim = reclaim_floor(new_floor, oldest_valid_root_in_the_ring);」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-core/src/mount.rs:4168`），它们回收。
5. 之后再顺序写两三次（s），分配器把回收出来的槽发出去。R 的根槽还在环里，池级 checker 的被抛弃根那一半判红：「judgements.judge("I-7.4", !walked_into_a_reallocated_or_erased_unit, || {」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-checker/src/walk.rs:8498`）。

原样输出（`research/prompts/m3-batch10z-d-code-r1-opus-model/out/raw-lines.txt`，同一段历史、两个臂）：

```
OPUS10ZD arm=h2 k=3 rkind=n r=(1, 7) p=(1, 6) fault=acct actions=Wss hit=true secs=0.13 selected=(1, 6) borrowed_at_mount=22 recorded_abandoned=-1 faulted_reads=10 W=ok(after=15) s=ok(raises=0) s=ok(raises=0) r_own_units_overwritten=0 first_red_step=- final=I-7.4:被抛弃的根（实例1、txg7）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元在盘0槽1418的那一份与位置条目里的校验和对不上；数据单 crash=not-run
OPUS10ZD arm=today k=3 rkind=n r=(1, 7) p=(1, 6) fault=acct actions=Wss hit=false secs=0.13 selected=(1, 6) borrowed_at_mount=22 recorded_abandoned=-1 faulted_reads=10 W=ok(after=15) s=ok(raises=0) s=ok(raises=0) r_own_units_overwritten=0 first_red_step=- final=- crash=not-run
```

`r_own_units_overwritten=0`：R 自己写的单元一个没被写掉，红的是 R 从 P 继承的数据单元（盘 0 槽 1418）。

### 取样与结果（副本上的数）

用户动作放开：a1 是挂上之后长 1–4 的全部串（字母 o 覆盖写、n 建 inode、s 顺序写两个单元、F 抬到准入上限、W 覆盖写到推出抬 F、b 管理员回退到上一版，u 只在末尾），每组 1813 段；a2 同一批串每一步之后都过 checker；a3 是「W + 随机 3–8 个 o/n/s」150 段（种子 = k，几个臂同一批串）；a4 同 a3、R 的账在挂上之后也一直读回全零。

| 组 | 臂 | k | R 的种类 / 故障 | 段数 | 某一步之后红 | 终态红 | 某一步被拒 |
|---|---|---|---|---|---|---|---|
| a1 | h2 | 3 | n / acct、n / data | 1813、1813 | — | 17、17 | 133、133（都是 b 回退被拒 TargetNotACandidate，各臂相同） |
| a1 | h2 | 5 | n / acct、n / data | 1813、1813 | — | 1、1 | 133、133 |
| a1 | h2 | 3、5 | s / acct、s / data | 各 1813 | — | 0 | 133 |
| a1 | today | 3、5 | 上面四种 | 各 1813 | — | 0 | 133 |
| a2 | h2 | 3、5 | n / acct | 各 1813 | 20、1 | 17、1 | 133 |
| a2 | today、diskfirst | 3、5 | n / acct | 各 1813 | 0 | 0 | 133 |
| a3 | h2 | 1、3、5、8 | n / acct | 各 150 | 98、53、68、10 | 0、10、39、10 | 0 |
| a3 | today、diskfirst | 1、3、5、8 | n / acct | 各 150 | 0 | 0 | 0 |
| a4（故障不撤） | h2、diskfirst | 3、5 | n / acct | 各 150 | 53、68 | 10、39 | 0 |
| a4（故障不撤） | today | 3、5 | n / acct | 各 150 | 0 | 0 | 150、150（第一串推抬 F 时 NewerStateStillUnreadableAfterOneReread） |

（a3 h2 k=1 那一组另有 `data` 故障 150 段，结果与 acct 逐格相同：98 段某一步红、0 段终态红。）
R 是覆盖写或顺序写三个单元（o、s）时 R 不与 P 共用会被释放的单元，没有一段红；入库 H2 用例的 R 正是覆盖写（「PublishRecordedAsR::Overwrite,」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:778`））。
终态红少于某一步红，是 R 的根槽在后面的发布里被转过的环盖掉、R 离开根环之后不再判。

### 四句

1. **分不分辨臂**：分辨。同一段历史 h2 臂红、today 臂（撤掉 H2 的 `.or_else`、照读盘）不红；不撤故障时 today 臂换成每段都拒（a4），h2 臂换成不拒而红。它打的是 H2 这一臂，不是各臂共用的前提。
2. **系统当时看不看得到判别它的东西**：看得到。挂载那一刻重建分配器就知道 R 是比所选那一版新、有效、账读不出而记录齐（C 那两份账都没加）；抬 F 那一刻 R 的账读得出（a2/a3 故障已撤）。今天的代码没把「C 那一槽有没有带上那条根的账」记下来，抬 F 那一遍也不看。
3. **满足的是判据字面的哪一个分句**：D23 已定项 14「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」（`.claude/kb/decisions/23-journal的角色与格式.md:390`）；不变量 I-7.4「它们引用的块在离开根环之前同样不许重新分配、不许抹头」（`.claude/kb/invariants.md:58`）。R 是被这次挂载自己写行抛弃的，不是崩溃恢复抛弃的——这一句的主语按字面套不套得上 R，交主 agent 判；checker 那一半（`judge_blocks_referenced_by_abandoned_roots`）按「被最新根的实例表判抛弃」判，套得上。许可它的是 D16 第 37 行 H2 那一句「照 C 那一圈记着的落点算、不读账、读不出不拒」（`.claude/kb/decisions/16-发布语义.md:37`）。两条条款在这一形上给的不一样，是条款冲突，不是实现没照条款做。
4. **跑前条款给的改法在打中的格上还中不中**：上一轮判决交给 H2 的只有认同条件（那一槽记的是有效根且 txg 相同），这一形里 C 那一槽恰好只有 R 自己的记录，认同条件对了照样中。实现员报告第 3 条的改法「C 按 (实例代号, txg) 存」也中：分得开的是别的发布，R 自己那一份本来就缺账（推的：384 槽盘上这段历史里 R 那一槽只有 R 一次发布的记录）。

### 改法各修哪一格（都只在我的模型上量过、被攻过零轮）

| 改法 | a2 / a3（故障撤了） | a4（故障不撤） | 代价 |
|---|---|---|---|
| 甲 `diskfirst`：借来的 C 只在这一遍读盘读不出时才用 | **量过**：0 红（a2 3626 段、a3 600 段） | **量过**：照样红（53 / 150、68 / 150），与 h2 逐格相同 | **量过**：副本入库用例 8 passed（`out/raw-lines.txt`）；违 H2「不读账」那半句 |
| 乙 `cplusreleased`：C 再并上现行账里释放代大于那条根 txg 的全部已释放落点 | **量过**：0 红（300 段） | **量过**：0 红（300 段） | **量过**：隔离均值 370.2（k=3）、388.2（k=5）个槽，对照 h2 是 4.1、15.8；600 段里 588 段有一步被 NoSpaceAfterRaisingTheFloor 拒。不可用 |
| 丙 `cplusmountaccount`：挂载那一刻较新可读根账读不出而记录齐时，C 那一槽并上那一刻现行账里仍分配着的全部落点 | **量过**：0 红（a2 同一批 1813 段、a3 300 段） | **量过**：0 红（300 段） | **量过**：借来的落点 22 → 44；隔离均值 8.6（k=3）、33.1（k=5）；0 段被拒。副本入库用例在这一臂上没跑；会动 C_NEWER_ROOTS 与第九项的计数，有没有别的用例因此红是**推的**、没量 |

丙的道理：R 建在所选那一版上，它引用的单元要么是它自己写的（记录点名了），要么在所选那一版的账里；后一半挂载那一刻仍分配、本来就豁免，一旦这次挂载释放它们，抬 F 与进程内回收都照没解开的 C 补隔离（「// C694 的 C 只在这里解开」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-core/src/allocator.rs:2179`））。多护的是「挂载之后释放、R 离开根环之前」那一截。

### 崩溃状态

命中的历史都含覆盖写或顺序写：一次覆盖写那一段 24 个写、顺序写那一段 28 个写，按 `closed_form_state_count` 单段就是 2²⁴、2²⁸ 个状态。命中的四段历史整条流是 6.9×10⁸–9.6×10⁸ 个（原样 `crash=skipped(closed_form=694190243)` 等，`research/prompts/m3-batch10z-d-code-r1-opus-model/out/crash.txt`）。照定义「一段历史自己就超过约 10⁶ 的，那段不跑」，**命中的历史一段都没做崩溃枚举**，只判了终态与每一步之后的盘面。交主 agent：要在入库装置上坐实，用平时快档那种按段的展开（`enumerate_layer0_quick_tier_versions`）在主 agent 的跑法里量。

## 二、K2-3：挂载那一刻被抛弃根槽读回全零（造出来了，归 C554）

造法：入库 `pool_whose_newest_root_was_abandoned_by_a_crash_recovery` 同一造法（之前覆盖写 k ∈ {1, 3, 5} 次、实例 3 再写 0 或 2 次），之后可写挂载一次（实例 4），**只在挂载那一刻** C 的根槽读回全零（`slot-zero-at-mount`）或报读错（`slot-err-at-mount`），挂上之后照读；用户动作长 0–2 的全部串（o n s F W，可接 u）。每组 61 段：

| 故障 | 每组某一步之后红（六组） | 终态红（六组） |
|---|---|---|
| none | 0 | 0 |
| slot-zero-at-mount | 48、42、42、42、42、42 | 42、36、39、39、40、40 |
| slot-err-at-mount | 与 zero 逐格相同 | 与 zero 逐格相同 |

原样一行（`research/prompts/m3-batch10z-d-code-r1-opus-model/out/raw-lines.txt`）：

```
OPUS10ZDX k=1 i3=0 c=(2, 8) fault=slot-zero-at-mount actions=o recorded_at_mount=-1 abandoned_slots_in_table=0 faulted_reads=16 o=ok(raises=0) first_red=0o:I-7.4:被抛弃的根（实例2、txg8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元在盘0槽1416的 final=I-7.4:被抛弃的根（实例2、txg8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元在盘0槽1416的
```

挂载那一刻表里没记 C（`recorded_at_mount=-1`、`abandoned_slots_in_table=0`），C 的单元在实例 3 的账里是空闲的，第一次覆盖写就发出去。挂载路径这一批只多传一个 `&[]`，这一形与第 10z 批第四块无关（推的：没在 10z-d 之前的树上跑）。

算不算 D16 已定的代价：**不算**。D16 第 37 行「读坏的槽与读回全零的槽」那一句只定了回收门槛钉在它可能住过的最旧 txg（`.claude/kb/decisions/16-发布语义.md:37`），护的是已释放、在 defer 里的槽，不护空闲槽，也没写「这一形认下」。它是 C554 已记的那一形：「同一形在被抛弃的根所在的槽挂载那一刻读不出、可写挂载里重读也读不出时照样出现」（`.claude/kb/checks-owed.md:426`），D23 已定项 14 射程「挂着时回退的已知边角」② 也点了它（`.claude/kb/decisions/23-journal的角色与格式.md:413`）。读回全零这一变体 C554 的题面写的是「读不出」，没写全零；量出来两者逐格相同，建议并进 C554 的题面。

## 三、K2-1：H1 被抛弃根的槽读坏（没打中，一次抽样）

造法同上一节，故障换成挂上之后 C 的根槽每次读都读回全零（`slot-after`）或报读错（`slot-after-err`），这个进程写到那一槽就撤故障（x2）。用户动作长 0–3 的全部串（o n s F W，可接 u），每组 311 段，k ∈ {1, 3, 5} × 实例 3 再写 {0, 2} × 两种故障 = 3732 段：**0 段某一步之后红，0 段被拒**（`out/summary-x2.txt`）。

先跑的 x1 那一组故障在这个进程写掉那一槽之后不撤，3732 段里有 W、u 被拒（例如 `W=Err@15:FloorRaiseFailedWhilePushingForSpace(…RollbackFloorCeilingRootRin…`）：那一槽已经换成这个进程自己的有效根，读坏重读仍坏照拒，是 H1 另一半「有效根的槽照拒」；是装置的故障层造出来的，不算打中（推的依据：x2 写到那一槽就撤故障，同一批串 0 段被拒）。

读码（推的）：H1 那一圈的豁免与读出来的被抛弃根同一个集合（「// 根槽这一遍读坏的被抛弃根（H1）：读不出它的根记录，照它挂载时记着的落点补隔离，豁免与读出来的被抛弃根同一个集合。」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-core/src/mount.rs:1629`））；记着的落点是挂载那一刻按账或映射算出的全集，凭据放行那一形记的是空串（M7 用例钉着）。不重读那一条：重读读得出也是同一条根、同一份记着的落点，结局相同，不改 `abandoned_roots_unreadable`。

## 四、K1-1：抬 F 那一串读根环（没打中，读码）

- 准入算上限滤了被抛弃根槽「!ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-core/src/mount.rs:3596`）；抬 F 重算同一句（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-core/src/mount.rs:3870`），交回的记着的落点只取这一遍没读出的槽「.filter(|(ring_slot, _)| !ring_slots_read.contains(ring_slot))」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-core/src/mount.rs:3912`）。
- 抬 F 那一遍算 F 生效值「let effective_floor = effective_rollback_floor_rereading_the_newest_instance_table_once(」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-core/src/mount.rs:4023`）本来就不分「知道住着根的槽」，读坏当没有根，这一批前后一样。
- 管理员回退那一遍仍用没滤的那一份「&ring_slots_known_to_hold_a_root_by(allocator),」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-core/src/mount.rs:7588`），`roll_back_by_a_forward_publish` 那一段没有 hunk：没变。
- core 之外有两处自己算上限，还用没滤的那一份：「&ring_slots_known_to_hold_a_root_by(allocator),」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-checker-tier/src/on_device_modes.rs:336`）、「&ring_slots_known_to_hold_a_root_by(&mounted.allocator),」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:1992`）。两处都在不注入读故障的一步里算，今天碰不到被抛弃根槽读坏（推的）；以后在那一步注读故障，它们会比 core 先拒。

## 五、K1-2：认同条件（没打中，推的）

- 只比 txg：「} => *recorded_checkpoint_txg == checkpoint_txg,」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-core/src/allocator.rs:1167`）。抬 F 那一遍问的那条根，是从它 txg 对应的那一槽读出来的；那一槽挂载之后只有这个进程写得掉，写掉就换成 `ValidRoot`（这个进程的根）、C 在那一槽同时解开。所以同一槽、同一 txg 而实例不同的根读不出来，只比 txg 认不错。
- 差一整圈、那一槽 C 只记着别的发布：要水位之上同时有 txg 差 24（3S）的两次没施加的发布，即择到的根比 R 旧一整圈以上。F2 只退到见证到的那一版，系统配置每次发布都轮换，单故障下差一版；x 与 a 两组造法里 C 那一槽从没出现两次发布（推的，没造多故障把见证拉开一圈的盘面）。
- 两样都没记着：`.or_else` 交 `None` 之后走读盘，副本上 today 臂让入库 H2 用例照旧拒（`out/raw-lines.txt`），开关接在这一处。

## 六、K3：改回去照绿（部分打中）

- 这一批改动逐处改回去：准入上限那一道滤、抬 F 重算那一道滤、H1 补隔离那一圈、`.or_else`、认同条件三臂、四类槽交 `None`，变异表追加的 15 行各有一条会红的用例或库单测（实现员证红表，读过没复跑）。没找到一处单独改回去照绿的。
- 但入库 H2 用例造不出 K2-2 这一形：R 是覆盖写（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:778` 那一行），挂载那一刻 R 的账读得出、C 那一槽带着 R 的全部账；断言「assert_none_of_the_placements_is_free(」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:861`）查的是借来的那一份，不是 R 引用的那一份。量过：副本上这份用例 8 passed，同一副本上 K2-2 那一形红。要能判，用例得让 R 与 P 共用之后被释放的单元（R 只建 inode），并在抬 F 之后再写几次、用池级 checker 判 R 引用的全集。

## 没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| R 顺序写三个单元（s），账或账之外读回全零 | k ∈ {3, 5} × 2 种故障 × 两臂，长 1–4 全部串各 1813 段 | 0 红（R 不与 P 共用会被释放的单元） |
| R 覆盖写一次（o） | 只试了 k = 3、Wo 与 Wsss 两段 × 3 种故障 × 两臂（草稿目录 `try1-*.log`） | 0 红；取样很小 |
| H1：被抛弃根槽挂上之后读坏 | 3732 段（第三节） | 0 红、0 拒 |
| 管理员回退夹在中间（b） | a1 每组 1813 段里含 b 的串 | b 被拒 133 次（TargetNotACandidate，上一版是写行或暖机根）；做成的 b 之后红与不红跟着 W 之后的 o/s 走，b 本身不开新的红 |
| 认同条件认错 | 读码 | 见第五节 |

## 这条腿自己的限度

- 只有 384 槽小盘一种几何；R 只放开了三种（o、n、s），P 只有一个文件；没造 4 GiB 档、多片实例表、多层分配记录树。
- 命中的历史一段都没做崩溃枚举（第一节末），只判终态与每一步之后的盘面。做了全量枚举的只有不含覆盖写与顺序写的对照历史（下一节），合计状态数见那里。
- 每组只抽了一次样；随机串种子取 k，三个臂同一批串。
- 三个改法都只在我的模型上量过、被攻过零轮；丙会改挂载那一刻 C 的内容，入库用例在丙上没跑。
- 副本上的数都不是入库装置上的数。

## 做了崩溃全量枚举的历史

挂载起录的整条流，按 `closed_form_state_count` 不超过 10⁶ 的才调 `enumerate_layer0_versions`（每段都展开）。只有不含覆盖写与顺序写的对照历史够得上（k = 3、R 建 inode、挂载那一刻 R 的账读回全零），h2 与 today 两臂各跑一遍（`out/crash.txt`）：

| 动作 | 状态数 | 恢复后 oracle 违例 | I-7.4 | I-4.8 |
|---|---|---|---|---|
| u | 36 947 | 0 | 0 | 0 |
| n | 90 165 | 0 | 0 | 0 |
| nu | 98 387 | 0 | 0 | 0 |
| Fu | 49 276 | 0 | 0 | 0 |

两臂逐格相同；另有一次试跑 u 36 947 个。合计 586 497 个状态，没越过约 10⁷。命中的 Wss、Wsss、Woss、nWss 整条流 6.9×10⁸–9.6×10⁸ 个，照定义不跑，交主 agent。

## 没做什么

- 没在入库装置上重做，没跑 checker 档测试，没碰正推那几格；副本上 LR2 那份 14 条用例在三个改法上都没跑。
- 实现员证红表的 15 行变异只读了，没复跑。
- 命中的历史没有崩溃枚举（上一节）。
- 草稿目录 `~/.local/share/singlefs/work/m3-batch10z-d-code-r1-opus/` 的仓副本 `repo/` 与编译目录 `target/` 交回前删了（`rerun.sh` 从冻结副本重建）；原样日志留在 `runs/`，都小。
