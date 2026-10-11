# 第 10z 批第十、十一块代码轮第一轮 攻方腿报告（m3-batch10z-jk-code-r1-opus，2026-10-08）

攻的是第十块（core 丙并中间、回退复活）与第十一块（GPU 列卡带借据号、拆锁、判别力），重点在实现员自定、被攻过零轮的那几处（正文第二节）。
这一轮的数都是在冻结副本的拷贝上量的，不进 kb。

## 复跑与 sha256

- 模型目录只有一份 `research/prompts/m3-batch10z-jk-code-r1-opus-model/rerun.sh`（sha256 fb3518bd3ad6cd9787d8e9c29b861790a90435924a24a475252fa837c1ff73f5）：没有新写的模型源码，这条腿把入库 `mount.rs` 逐条套上第十块 10 行变异、跑点名的测试看各自红（K5 判别力的独立复跑）。
- 复跑：`bash research/prompts/m3-batch10z-jk-code-r1-opus-model/rerun.sh <冻结副本tree> <空的CARGO_TARGET_DIR>`。每一件在 rerun.sh 里经 `bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 15 cargo …` 起。
- 被判文件在冻结副本 `~/.local/share/singlefs/work/m3-batch10z-jk-code-r1/tree/` 的 sha256：
  - `crates/singlefs-core/src/mount.rs` 73d353c452a564e4b74faefcc4b13d467699cea09022ea2c77a4a1cd55dea145
  - `crates/singlefs-harness/src/gpu_memory_ledger.rs` c9fdf242204f586da983cd7f663c604baad5ce865f55e4ecad7f472ed9f4a105
  - `crates/singlefs-checker-tier/src/gpu_unit_checks.rs` 3b3606e102c1a1c6760074b5bda8885cb67e7e0d5bc96e47f1a07cf192aae54b
  - `crates/singlefs-checker-tier/src/crash_amplification.rs` 0b3006f0d0660547b3dd80ee9d7673bef3e957b548c42e235cf0fee6704fb81a

## 各格判定一览

| 格 | 判 |
|---|---|
| K1-1 丙并中间罩不罩得住 | 没打中（走丙那一支的根 R 罩得住）。唯一漏的是 Q 的记录全读不出那一形，实现员已报（第六节第 3 条）、归 C554，不单算打中 |
| K1-2 丙并中间放的时机 | 没打中：并进 R 那一槽的落点随 R 那一槽写掉解开，不会一直不放，也不放早（中间那一槽 Q 写掉时 R 那一槽还罩着） |
| K2-1 认回退认错或认漏 | 没打中：txg 全局单调唯一（`mount.rs:1295`），谱系记录按 txg 找不会认错；R 不是回退却点名更早回退的谱系片时 find 交 None、照丙，不认错 |
| K2-2 回退复活罩不罩得住 | 没打中：丙（P 仍分配）∪ revived（R_old 引用 ∩ P 已释放）覆盖了 R 引用的全部落点；R_old 自己是回退、回退到更早一版都走同一份读法 |
| K2-3 认不出回退的可达面 | 交用户的残余，量了：reachable，红率 ≈ `today` 臂（实现员数：384 槽 39/60、1536 槽 29/60 随机历史红）。直接「谱系片读不出」的故障没单独造，但变异 8（打包单元只看非谱系片）就是它的等价，实测红 |
| K2-4 多护的那一截何时放 | 命中一处可达的过护：实现员自报多隔离 24 槽、其中 20 槽 R 不引用；不会一直不放，但 R 在环里时一直把第九项抬高 20 槽，紧盘上可致准入白拒（没量到，推的，实现员第六节第 2 条交主 agent） |
| K3-1 列卡之后才没等成的那一笔 | 没打中：列卡取快照，之后记成没等成的不在 listed 里；设备号单调（`gpu_memory_ledger.rs`）挡住重起换台后的误销 |
| K3-2 三者不是同一台 | 没打中：设备号单调 + 重起记账 `take_back` 把旧台借据全销（`gpu_memory_ledger.rs:869`），不多销也不永远销不掉 |
| K4-1 两把锁死锁 / 双起 | 没打中：锁序只有 starting→started 一向；拿到 starting 再看一次留着的挡住双起 |
| K4-2 借之前碰上正在重起 | 没打中：`started_now` 不等（`gpu_memory_ledger.rs:1740`），重起时交旧台、是丢了的就跳过 poll，只少给、不永久少给 |
| K4-3 换下来的旧设备在放开锁之后才丢 | 没打中：`drop(replaced)` 在锁外，别的线程不被压住 |
| K5 判别力 | 没打中：第十块 10 行变异（含实现员没单证的 3、4、5、7、8、9）在我的拷贝上逐条独立复跑，全红（下文第五节原样）。判别力的缺口是场景覆盖、不是「改回去照绿」 |
| K6 只防 OOM 不压并发 | 没打中：K3/K4 之外没找到账多给的新形；少给是 K4-2 那一处暂时的 |

## 一、K1 丙并中间

`with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots`（冻结副本 `crates/singlefs-core/src/mount.rs:2064`）把走丙那一支的根 R 那一槽并上 (P, R) 之间每个 txg 对应的槽在 C 里记着的落点，源头是三份并好的 `named_by_ring_slot`。

K1-1 罩不罩得住：R 走丙（记录齐、账读不出）时 R 自己的单元被 R 的记录点名、进 C 里 R 那一槽；R 从中间发布 Q 继承的单元被 Q 的记录点名、进 Q 那一槽，丙并中间把 Q 那一槽并进 R 那一槽。两条根相差超过一圈时 `.take(ring_length_in_txgs)`（`mount.rs:2078`）取的那一圈 txg 覆盖全部 3S 个环槽，而 `named_by_ring_slot` 对同一槽的多条记录是 `.extend` 累加、不覆盖，所以大跨度也不漏。没造出「R 引用、而 C 里哪一槽都没有」的单元——除实现员已报的那一形：Q 的记录全读不出（Q 的单元连 Q 那一槽都没记），挂上第一次覆写就红、违例的根是 Q，归 C554，实现员第六节第 3 条已报，不单算打中。

K1-2 放的时机：并进 R 那一槽的落点随 R 那一槽被这个进程写掉时解开（`record_root_written_by_this_process` 清隔离前先算 `placements_the_isolation_still_protects`，R 那一槽还在就还罩着）。
- 不放早：中间的 Q 那一槽先写掉（Q.txg < R.txg，Q 的槽先轮到），那一刻 `still_protected_by_the_isolation` 把 Q 的单元按 R 那一槽算作还罩着，不清。实测 `units_a_root_inherits_…`「有一步之后 Q 已离开根环而 R 还在」走到了这一窗口、全程不红（第五节基线绿）。
- 不一直不放：R 那一槽至多一圈根环发布之内写掉，解开。

K1 的变异（第十块 1、2、3、4、5、10 行）在拷贝上逐条独立复跑全红（第五节），判别力挡得住把这几处改回去。

## 二、K2 回退复活

认回退：`rollback_target_txg_named_by_the_records_of_a_publish` 读 R 记录点名的谱系打包单元，`.find(|lineage| lineage.rollback_txg == checkpoint_txg_of_the_publish)`（`mount.rs:2169`）取 T_old；回退目标 `.filter(|root| !is_abandoned(root) && root.checkpoint_txg == rollback_target_txg)`（`mount.rs:2141`），只并 `released_in_the_current_account.contains`（`mount.rs:2146`）的那几个。

K2-1 认错 / 认漏：两处都只按 txg 比、不按实例代号比。查 txg 会不会撞：`新实例的第一次发布的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1`（`mount.rs:1295`）——txg 全局单调、不按实例重置，环里不会有两条根同 txg，所以按 txg 找 T_old 只命中一条、不认错。管理员回退「不新增实例、不取号」「checkpoint_txg 照常加一」（`.claude/kb/decisions/23-journal的角色与格式.md:378`:「管理员回退是挂着时的一次向前发布」），只有真回退 R 会写一条 rollback_txg = R.txg 的谱系记录（`.claude/kb/decisions/18-块里携带什么信息.md:576`:「管理员回退那次发布追加一条」）；R 不是回退却点名了更早回退的谱系片（谱系树随现行那一版、只增不删）时，find 找不到 rollback_txg = R.txg 的记录、交 None，照丙原样、不认错。没打中。

K2-2 罩不罩得住：R 走丙时 R 那一槽的隔离 = 丙 `placements_still_allocated_in_the_account`（P 仍分配）∪ revived（R_old 引用 ∩ P 已释放）。R 引用的落点要么 R 自己写、被 R 记录点名（进 R 那一槽），要么是 R_old 的用户可见单元：在 P 里仍分配的被丙罩住，在 P 里已释放的必是 R_old 引用、被 revived 罩住。回退到更早一版、R_old 本身也是回退都走 `placements_referenced_by_root(R_old)` 同一份读法，没找到缺口。没打中。

K2-4 多护的那一截何时放：revived 取 `placements_referenced_by_root(R_old)` 的全部（含 R_old 的固定点单元：实例表、树表、分配记录树节点），再滤 P 已释放。实现员第六节第 2 条自报：384 槽盘多隔离 24 槽，只有 4 槽是 R 真正复活的数据单元，其余 20 槽是 R_old 自己的固定点单元、R 并不引用。
- 命中一处可达的过护：这 20 槽是纯多护，与别的 C 落点同一个时机放（R 那一槽写掉），不会一直不放；但 R 在环里的整段时间里把第九项（D28 已定项 1）抬高 20 槽。紧盘上这 20 槽可致准入白拒——`today` 臂（不护）在实现员的 384 槽盘上 `refused_histories=0`，但那是宽几何；更紧的单元区没量。这一处只在实现员的模型上量过、被攻过零轮，白拒是推的，实现员已交主 agent（还能再窄：只护 R 引用的那一截）。

## 三、K2-3 认不出回退的可达面（交用户的残余，量了）

认不出回退有三条路（`placements_a_rollback_publish_revived_released_in_the_current_account` 交空、照丙原样）：
1. R 记录点名的谱系片读坏 / 校验和对不上 / 里面没有 rollback_txg = R.txg 的记录 ⇒ `rollback_target_txg_named_by_the_records_of_a_publish` 交 None；
2. 根环里没读出 R_old（R_old 已离开根环，或它的槽读坏）⇒ `.filter(txg == rollback_target_txg)` 空；
3. R_old 的账读不出 ⇒ `placements_referenced_by_root(R_old)` 交空。

**可达的故障组合与回退目标**：三条都要「R 走丙」先成立（R 的账三指针读回全零、R 的记录齐）。在此之上：
- 路 1 reachable：故障模型把 R 写的谱系打包单元那两份也读回全零（数据区零化、journal 记录槽完好）——与 R 走丙的故障同一类（都是数据/账区零化），只是多零化一个单元。
- 路 2 reachable：回退目标 R_old 的 txg 比挂载后环里最旧有效根还旧（会话多写几次把 R_old 挤出根环），而 R（走丙、在环里）还引用着复活的那几份。
- 路 3 reachable：R_old 的账指针那几份读回全零。

**各红多少**：认不出之后补法交空 = `today` 臂行为（代码：revived 为空，placements 只剩丙）。所以红率就是 `today` 臂的红率。实现员量到（副本，`runs/cost-summary.txt`，副本上的数、不进 kb）：`rollback-cost` 384 槽 `today` hit=39/60、1536 槽 hit=29/60（随机 20–34 步历史，约 48%–65% 红）。我在拷贝上没单独造「谱系片读回全零」的故障，但变异 8（打包单元只看非谱系片 `!= PACKED_TYPE_LINEAGE`）让 find 必然交 None，正是路 1 的等价，实测 `units_a_rollback_…` 第 1849 行红（第五节）——认不出 ⇒ 复活的落点不进 C ⇒ I-7.4 红，坐实了这条路可达且红。

**有没有一条读得出的信息（不改格式）能让它认得出**：没有。回退那次发布事务号写 0（与空发布同，`roll_back_by_a_forward_publish` 的注释），这个信号能看出 R「可能是回退或空发布」，但给不出目标 T_old；T_old 只在谱系记录里，谱系片读不出就无从得知回退到哪一版，也就无法复活正确的那几份。所以不改格式、不另读别处，认不出就是认不出。

四句（打中之后先答）：这一格是交用户的残余、不单算打中，但照四句过一遍——
- 分不分辨臂：分辨。认不出只在「谱系/根环/R_old 账读不出」时触发，补法与丙在这一截行为相同、在读得出时不同。
- 判别子观测不到：是（这正是它交给用户的理由）。读不出谱系片时，补法看得到的东西与一次普通发布逐条相同，分不出 R 是不是回退。
- 满足判据哪一分句：I-7.4（`.claude/kb/invariants.md:58`:「近 K 代块未被复用」）——被抛弃根 R 引用的复活单元被重新分配。
- 跑前改法在这几格还中不中：实现员给的两条出路（拒可写 / 退到正推的对照不滤已释放）在这几格上都能改掉红（拒就不发出去，对照多护约 100 槽把复活的也护上）；记欠账（同 C554）那一条不改红、照发出去。

## 四、K3 / K4 / K6 GPU（读码，没打中）

K3-1：`leases_kept_after_a_failed_wait_on_each_card` 列卡时克隆快照 `listed`，`return_the_listed_leases_kept_after_a_failed_wait_on` 只销 listed 里、此刻还记着的；列卡之后才记成没等成的不在 listed、不销。前提「借据先于缓冲丢」成立：记成没等成只在借据（booking）被丢时（`take_in_the_wait_before_returning` 的 TheWaitFailed 支），那一刻缓冲随 drop 交回 wgpu，poll 之后才提交的新笔不在这一批。没造出多给。

K3-2 / K4-2：设备号 `next_device_number` 单调递增，重起换台给新号。重起记账 `take_back_the_driver_share_of_a_lost_device` 里 `for kept_lease in std::mem::take(&mut self.leases_kept_after_a_failed_wait).into_keys()`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:869`）把旧台的借据全销。列卡→poll→销三者跨了重起时：poll 的是 `started_now()` 交回的新台（旧台号 ≠ 新台号），`return_the_listed…` 按 `*listed_on_the_device == device` 比，旧台借据不被新台的 poll 误销（且已被重起记账销掉），`.is_some()` 守卫挡住重复销。没多销、没永远销不掉。

K4-1：两把锁 `started`、`starting`。`started_or_start` 先无锁看一次留着的、好就返回；否则拿 `starting`、再看一次、再 `start()`、最后拿 `started` 换一份。持 `started` 的几段都先放开再拿 `starting`，锁序只有 starting→started 一向，不死锁。拿到 starting 再看一次留着的挡住同一张卡上双起（变异 673 去掉这一看、实现员报里红）。

K4-2：`started_now`（`gpu_memory_ledger.rs:1740`）只取克隆、不拿 starting，不等正在起的设备；重起时交起之前留着的那一台（丢了的），`take_in_the_devices_found_idle` 见它 is_lost 就跳过 poll，到重起完下一轮再 poll——少给是暂时的，不永久少给。K4-3：换下来的旧设备 `drop(replaced)` 在放开 started 锁之后，wgpu 等设备空了的那一下不压别的线程。K6：K3/K4 之外没找到账多给的新形。

## 五、K5 判别力：第十块 10 行变异在拷贝上独立复跑（原样）

拷贝 `~/.local/share/singlefs/work/m3-batch10z-jk-code-r1-opus/tree/`（= 冻结副本，sha 对上），`CARGO_TARGET_DIR` 另置，逐条 `sed` 改 `mount.rs`、跑点名测试、还原。基线绿：
- 库 `cargo test -p singlefs-core --lib -- placements_of_the_publishes_in_between_chained_onto_the_newer_roots_tests`：`test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 203 filtered out`
- harness 两条新用例：`test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 38.60s`

实现员单证过的（1、2、6、10）此处不复，补测它没单证的 6 行，每条单独套上、单独红：

| 变异行 | 改法 | 测试结果 |
|---|---|---|
| 3 丙并中间 中间那几槽从第二个 txg 起（`+1`→`+2`） | harness `units_a_root_inherits_…` | 红 `panicked …:1603`「缺 26 份」 |
| 4 中间那几槽只取紧挨 R 的（`take(ring_length)`→`take(1)`） | 库 | `FAILED. 1 passed; 1 failed; … 203 filtered out` |
| 5 R 那一槽已有的落点也再并一次（`insert(..)`→`.. \|\| true`） | 库 | `FAILED. 1 passed; 1 failed; … 203 filtered out` |
| 7 回退目标取仍分配、不取已释放（`contains`→`!contains`） | harness `units_a_rollback_…` | 红 `panicked …:1849`；`FAILED. 0 passed; 1 failed; … 12 filtered out` |
| 8 打包单元只看非谱系片（`== LINEAGE`→`!= LINEAGE`） | harness | 红 `panicked …:1849`；`FAILED. 0 passed; 1 failed; … 12 filtered out` |
| 9 回退目标按被抛弃的根找（`!is_abandoned`→`is_abandoned`） | harness | 红 `panicked …:1849`；`FAILED. 0 passed; 1 failed; … 12 filtered out` |

库 4、5 与 10 另各单独套、单独红（都 `1 passed; 1 failed`，还原后 diff 对回基线 SAME）。所以第十块 10 行变异逐条有判别力，「改回去照绿」没找到。

判别力的缺口是**场景覆盖不是改回去照绿**：两条 harness 用例各只跑一个场景（丙并中间：k=3、Q 顺序写、R 建 inode、一组故障；回退复活：回退到第一个文件那一版、一组故障）。回退到非相邻版、R_old 本身是回退、(P,R) 跨度超过一圈、认不出回退那几形都没有用例（实现员「不枚举崩溃状态」，层 0 全量超 2.6×10⁸ 没跑）。这些是覆盖欠账、不是这 10 行改回去能照绿。

第十一块 checker 档 4 行（X1、X2、X4、X4b）要 gpu 特性、是重型测试，这条腿不跑（按定义）；读码看：X4/X4b（借之前 poll 那一句注释掉 / 包 `if false`）钉在 `gpu_unit_checks.rs` 的扫源码用例，X1/X2 钉在 `crash_amplification.rs:4783` 起那条扫源码用例（`== "&stop_is_requested,"`）。两处都只判源码里那一行在不在，改回注释/永不叫停能被它判出——但只在那一份源码文本上，真跑在卡上的判别没验（重型，不归这条腿）。

## 六、没打中的形状

- K1 走丙那一支：试了大跨度（take(ring_length) 边界）、R 之后还有较新根、两条根都走丙，`named_by_ring_slot` 的 `.extend` 累加都兜住。Q 记录全读不出那一形红，但实现员已报、归 C554。
- K2-1 认错：试了 R 不是回退却点名更早回退的谱系片、多条谱系记录、txg 跨实例——txg 全局单调唯一都挡住。
- K2-2：回退到更早一版、R_old 是回退、中间多次覆写，都走 `placements_referenced_by_root` 同一读法，丙∪revived 覆盖。
- GPU：列卡 / poll / 重起三者错台、两把锁死锁与双起、started_now 等不等——设备号单调 + 重起记账全销 + 锁序单向都挡住。
- 取样范围：库变异在全量状态空间上判（纯内存结构），harness 两条各一个场景一个几何；GPU 全是读码，没跑（账模型要 gpu 特性或并发压测，K4 的 500 毫秒窗口要真并发）。

## 七、这条腿自己的限度

- 第十一块的真判别（checker 档在卡上判、K4 的并发时序、账在真卡上多不多给）没跑：重型测试、按定义不跑；GPU 结论全是读码推的。
- K2-3 的红率用实现员 `today` 臂的副本数当代理，没在入库装置上重做；直接「谱系片读回全零」的故障没造，用变异 8 作等价。要坐实红率得主 agent 在入库装置上重跑。
- K2-4 的准入白拒是推的：没在紧单元区上量到 `implemented` 臂比 `today` 臂多拒的历史。
- 两条 harness 用例不枚举崩溃状态（层 0 全量超 2.6×10⁸），崩溃点那一维没覆盖。
- 副本上量的数（第五节的红/绿）不进 kb；判决要引得主 agent 在入库装置上重做。
