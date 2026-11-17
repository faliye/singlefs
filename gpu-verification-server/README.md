# gpu-verification-server

GPU 批处理验证服务：崩溃判定的后半段（卡上判）。前半段（录入、建表）在 CPU 上，由统一任务服务跑；两个服务之间只经判定库连：判段把要判的块写成待判行，本服务取走、判完在一帧里带守卫写回结论、删掉待判行。本服务管这一台的卡：起来照多机配置开全部点名的卡、一直持着，不经统一任务服务领卡。卡上判没有状态，服务只管按每张卡给服务的显存额度定每一波派多少、怎么分，不让卡 OOM。待判的块跨片、跨流进同一个池连续取；池里的态够把每张空着的卡各填满一次派活（水位线）、或最老的待判等过配置的限时，就发一波。

仓根工作区的成员，与验证包同一份 `Cargo.lock` 与 profile（判器组键从仓根这两份读，`crates/singlefs-verification/build.rs`），path 依赖 `../crates/singlefs-verification`（`gpu` 特性常开）。clippy 的编码纪律那几条照仓根工作区走（`.claude/singlefs-ai-sop/scripts/check.sh` 的 `CODE_DISCIPLINE_LINTS`）。

| 文件 | 做什么 |
|---|---|
| `src/main.rs` | 进程本体 `gpu-verification-server serve --state-directory <目录>`：解配置、算编进来的两组版本键、连判定库、照多机配置开这一台点名的全部卡（开不成退 78）、把「在跑」那一行（进程号、认领的持有者）写进状态目录下的 `serving`，之后跑取活循环直到被停 |
| `src/library_pending_pool.rs` | 待判池的判定库那一份：版本对账（认不认领只核 GPU 判器组键，结论照待判行里写的结论版本键存）、在库上现算判到了哪一步（强制现判的不问库；判完而可复用的不取、带守卫删待判行）、认领与续租、一帧写结论（判定库服务核认领、结论块那一行、删的待判行是不是读到的那一份）、守卫不过照种类重编重读再写、没写成的结论留下报红；验证包那一侧经接口 `VerificationPackageSide` |
| `src/verification_package_side.rs` | 验证包那一侧的真的那一份：待判行（表 `1D`）的扫与重读、读回装判器的材料、现算判到了哪一步、编结论帧，一律调验证包交出的函数，服务不另写一份 |
| `src/cards_of_the_service.rs` | 服务持着的那一组卡的接口（真卡与替身卡各一份实现） |
| `src/serving_from_the_library.rs` | 取活循环：起来开卡、一直持着；扫库挑块、认领、入服务主循环、判完写回；停的时候才放掉卡 |
| `control` | 装、起、停、查（`install`、`start`、`stop`、`check`；`unit-state` 给发起的那一台问第二台）与它自己的 `--selftest`；起任何一台的单元之前，发起的那一台清一次场（复用 `scripts/crash/gpu-quiesce-run.sh`）、`check` 另判单元环境里的 `CARGO` 与版本对不上；用法、退出码与弄坏开关在文件头 |
| `singlefs-gpu-verification-server.service.in` | 用户级 systemd 单元模板（`control install` 照它写）：开机不自起、被杀由 systemd 拉起；带 `Environment=CARGO=<这一台的 cargo>` |
| `src/batch_scheduling.rs` | 批次调度（纯函数）：一波是给每张空卡排的一轮，驻留优先、新装均分、不饿死、水位线；算法写在文件头 |
| `src/card_memory_ledger.rs` | 一个判器占多少显存（与验证包 `mebibytes_a_judge_needs` 同一个算法）、一张卡的显存账：越界的判器进不了账，计划只经账造 |
| `src/pending_pool.rs` | 待判池的接口 `PendingPool` 与内存实现（服务主循环与批次调度只看它；判定库里取来的块认领之后入这一份） |
| `src/card_worker.rs` | 每张卡一条工作线程：照计划换出、装上、按批判，结论按批交回 |
| `src/gpu_judges_on_a_card.rs` | 真卡上按流的键缓存 `GpuCrashJudge`；照多机配置（`GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED`，没写取 `GPU_CARDS_BY_PRIORITY`）开这一台点名的全部卡 |
| `src/service_loop.rs` | 服务主循环与客户端（同一进程里经通道） |
| `src/service_configuration.rs`、`service-configuration.txt` | 服务的配置（凑批的限时、扫库间隔、认领租期），编译时读进来 |
| `src/break_switches.rs` | 弄坏开关 `GPU_VERIFICATION_SERVER_BREAK` |

## 起停

多机配置里点名了卡的每一台各一个进程（`GPU_CARDS_BY_PRIORITY`、`GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED` 里有它的台标签）；在这一台跑 `control`，第二台的那一份经 ssh 在同一绝对路径的仓里做。

- 装：先编（编译目录归任务服务、任务结束就删，所以编和拷到共享根在同一条提交命令里做完，整条写法在 `control` 文件头的编法与 `install` 缺二进制时的出路里），再 `python3 gpu-verification-server/control install --binary <共享根下的二进制>`。没开 GPU（`ENABLE_GPU=0`）时 `install` 退 78。
  单元带 `Environment=CARGO=<多机配置 CARGO_BIN_DIRECTORY_BY_HOST 里装的那一台那一项>/cargo`（写结论时验证包要从 `CARGO` 认编这个二进制的工具链，没有就在写结论时崩；两台各取各的，取不到 `install` 退 78）。
  用户级 systemd 的单元 drop-in 目录 `singlefs-gpu-verification-server.service.d/` 下手写过设 `CARGO` 的 drop-in（例 `cargo.conf`）的，装完那一份多余：`install` 点名它，删法是 `rm` 那份文件、再 `systemctl --user daemon-reload`。
- 起：`python3 gpu-verification-server/control start`。没开 GPU 时一台都不起、退 0，所以随任务服务与判定库服务一起起的那一步可以一律调它；开了 GPU 而它没起来是服务健康的毛病，不退回 CPU。
- 清场：`start` 与 `install` 在起任何一台的单元之前，只在发起的那一台跑一趟 `bash scripts/crash/gpu-quiesce-run.sh -- true`（查卡、清场、回读），罩住要起的各台点名的卡；过了才起各台，没过一台都不起、退 78，出路点名那一趟的清场日志（默认在 `<工作根>/gpu-quiesce-logs`）。多机配置的清场命令是在起活的那一台跑、两台都管的，第二台上可能没有那份私有脚本，所以经 ssh 转去第二台的那一条不清场。第二台在不在跑经 ssh 起 `control unit-state` 问；单元已在跑的那一台不清它的卡（卡在它手里），都在跑就不清场。清场之后不复原。清场放在起单元之前、不套在服务进程外面的理由写在 `control` 文件头「开卡之前清场」。
- 停：`python3 gpu-verification-server/control stop`；没装的那一台照退 0。
- 查：`python3 gpu-verification-server/control check`。开了 GPU 时点名了卡的每一台都要单元环境里有 `CARGO`、它指的文件在那台在且能执行，且「在跑」（`is-active` 打 `active`，且 `serving` 写的进程号等于单元此刻的主进程号），缺一样退 78、出路写全；在跑的再读单元此刻的进程近四小时的日志，有 `GPU_VERIFICATION_SERVER_VERSION_MISMATCH` 行（待判行期待的 GPU 判器组键这一份二进制判不出）就退 1，打期待的键、这一份二进制的键与出路（用此刻的主工作区重编 release 二进制，`install --binary`、`stop`、`start`）；没开 GPU 退 0。
- 开机不自起（单元没有 `[Install]`、不 `enable`）；起来之后被杀由 systemd 隔 `RestartSec` 秒拉起（这一次不清场：清过的场不复原，一直清着），`control stop` 停的不拉起。

## 测试

全部经统一任务服务交，工作目录是带着验证包的那棵树里的 `gpu-verification-server/`（工作区成员的目录里起的 cargo 只编这一个包）。

| 测试目标 | 判什么 | 要卡 |
|---|---|---|
| `batch_scheduling_requirements` | 批次调度要满足的八条，一条一个测试 | 不要 |
| `card_memory_matches_the_verification_package` | 调度算的判器显存与验证包借卡算的逐个相同；流的键随表与源码变 | 不要 |
| `service_loop_with_judges_off_the_card` | 池 → 调度 → 工作线程 → 客户端整条路在替身判器上；卡坏了报拒；登记与入池的拒 | 不要 |
| `serve_process_marks_itself_serving_and_waits_to_be_stopped` | 进程本体（连测试进程里起的临时判定库）：多机配置没点名这一台的卡退 78、不写 `serving`；参数认不出退 2、状态目录不在退 1，都不写 `serving` | 不要 |
| `library_pending_pool_on_a_temporary_library` | 判定库那一份待判池对着临时判定库服务（待判行照验证包的写法读写，验证包另外三样是替身）：别人持着不取、GPU 判器组键对不上不认领并点名、结论版本键这份二进制算不出的照认领并照行里的键写回、强制现判不问库且写凭据行、写结论那一帧被拒时结论留下报红、判完而可复用的待判行带守卫删、判着时待判行被升成强制现判的重读再写带凭据、结论块那一行被别人改过的重编再写（两份都绿而不同报红）；取活循环起来就开卡、池空着照持着、停的时候才放 | 不要 |
| `gpu_service_path_agrees_with_the_direct_judge` | 服务路径与直接用 `GpuCrashJudge` 判逐态相同；两条流（同一个展开上限，窄的那条少展开一段）同卡交替派与各自独占逐态相同 | 要，带 `--features gpu`（任务服务照这个特性分卡、清场）；没卡打「本次未跑」 |
| `library_service_path_agrees_with_the_direct_judge` | 同一条小流切成几块写成待判行，取活循环取走、上卡判、写回，读回的结论字与直接判逐态相同（验证包那一侧是替身，结论字原样存） | 要，带 `--features gpu`；没卡打「本次未跑」 |

判段写待判行、本服务判完写回、判段读回的结论与直接判逐态相同那一条端到端对拍不在这里：判段写行等判的入口（验证包 `judge_my_blocks_on_gpu`）只在验证包里可调，见「还没做的」。

`control` 的自证不起 cargo、不起真 systemctl、ssh、journalctl 与清场脚本（换成自证根里的假脚本）：`python3 gpu-verification-server/control --selftest`，弄坏开关 `GPU_VERIFICATION_SERVER_CONTROL_BREAK`（表在 `control` 文件头）。

## 弄坏开关

`GPU_VERIFICATION_SERVER_BREAK=<项>[,<项>…]`，只有测试读（`BreakSwitches::of_the_environment`），服务进程交的是空的。每一项打开后点名的测试目标里那一条必须红（表与 `BreakSwitch::red_test_target`、`BreakSwitch::red_test` 逐项相同）。

测试目标 `batch_scheduling_requirements`：

| 项 | 弄坏哪一处 | 红的测试 |
|---|---|---|
| `memory-ledger-unchecked` | 显存账收越界的判器 | `judges_resident_on_a_card_never_exceed_the_quota_less_the_driver_share` |
| `scratch-split-ignores-kept-judges` | 新装的判器分草稿区时不扣留着的判器 | 同上 |
| `carved-states-not-consumed` | 切过的态下一批又切 | `every_pending_state_is_dispatched_exactly_once_and_the_rest_stays_in_the_pool` |
| `pool-drops-the-rest-of-a-partly-dispatched-block` | 内存池把没派的那一截也当派了 | 同上 |
| `watermark-ignored` | 池里有态就发 | `a_wave_goes_out_when_every_card_fills_once_or_the_oldest_waited_the_limit_and_never_from_an_empty_pool` |
| `wait-limit-ignored` | 等多久都不发 | 同上 |
| `batch-not-capped-by-the-state-count` | 一次派活的态数不按流的态数压 | `streams_sharing_a_card_split_the_remaining_memory_and_a_short_stream_takes_its_own_state_count` |
| `scratch-split-by-entry-order` | 新装的几条流先到先拿、不均分 | 同上 |
| `batches-of-a-stream-back-to-back` | 一条流的批在一波里连着派完 | `batches_of_streams_sharing_a_card_alternate_from_wave_to_wave` |
| `pool-order-ignored` | 流按键排、不按入池先后 | `streams_are_taken_in_entry_order_and_a_short_stream_is_not_left_behind_a_long_one` |
| `starving-streams-not-rescued` | 等过限时也不腾卡 | 同上 |
| `residence-ignored` | 驻留的判器当新流重装 | `a_resident_stream_goes_back_to_its_card_and_only_judges_without_pending_states_are_evicted` |
| `every-resident-judge-evicted` | 腾显存时有待判态的也换出 | 同上 |
| `input-order-followed` | 照调用方给的次序走 | `the_same_inputs_give_the_same_plan_whatever_order_they_come_in` |

测试目标 `library_pending_pool_on_a_temporary_library`：

| 项 | 弄坏哪一处 | 红的测试 |
|---|---|---|
| `claim-held-by-others-ignored` | 认领时别的持有者持着也当认领到 | `a_block_claimed_by_another_holder_is_left_to_it` |
| `version-mismatch-ignored` | 不对版本，期待的 GPU 判器组键不同也取 | `a_row_expecting_another_gpu_judge_is_not_claimed_and_is_named` |
| `conclusion-version-checked` | 认领时另要期待的结论版本键是这个二进制算得出的 | `a_row_whose_conclusion_version_this_build_does_not_compute_is_claimed_and_written_under_the_row_key` |
| `conclusion-written-under-this-build-key` | 写回时结论帧换成这个二进制算的那一把结论版本键 | 同上 |
| `forced-block-reads-the-library` | 强制现判的也先问库，判完的不取 | `a_forced_block_is_judged_without_reading_the_library_and_leaves_a_rejudged_row` |
| `rejudged-row-not-written` | 强制现判的写回那一帧不写凭据行 | 同上 |
| `refused-conclusions-dropped` | 那一帧没写成时结论不留 | `conclusions_refused_at_the_write_are_kept_and_reported_red` |
| `cards-opened-only-with-work` | 起来不开卡，库里有了能取的块才开 | `the_cards_are_opened_at_start_and_held_while_the_pool_idles` |
| `cards-let-go-when-the-pool-idles` | 池空一圈就放掉卡、再来活再开 | 同上 |
| `reusable-block-pending-row-kept` | 判完而可复用的块只不取、待判行不删 | `a_reusable_block_is_not_taken_and_its_pending_row_is_deleted_under_the_guard` |
| `pending-row-change-not-reread` | 写结论那一帧因待判行读完之后变了被拒时不重读、当场报红 | `a_pending_row_raised_to_forced_while_judged_is_reread_and_written_with_its_credential` |
| `guard-row-change-not-reread` | 写结论那一帧因结论块那一行编帧之后变了被拒时不重编、当场报红 | `a_conclusion_changed_under_the_guard_is_compared_again_before_writing` |

## 做到哪

- 批次调度、显存账、内存待判池、每卡工作线程、服务主循环：写好，经上面几个测试目标判（上卡的那一条要卡）。
- 工作区：仓根工作区的成员。
- 进程与起停：进程本体、用户级单元、`control` 的 `install`、`start`、`stop`、`check`。
- 从判定库取活（协议 `~/.local/share/singlefs/work/batch53-gpu-service-real-tasks/protocol.md`）：待判行与强制现判凭据照验证包的表 `1D`、`1E`，
  认领与续租、版本对账、现算判到了哪一步、读回材料、编结论帧调验证包交出的函数；一帧写结论由判定库服务核认领、结论块那一行与删的待判行。
  经 `library_pending_pool_on_a_temporary_library` 与 `library_service_path_agrees_with_the_direct_judge` 判（编结论帧那一样是替身）。
- 卡：`control` 起单元之前对这一台点名的卡清场（不复原），起来照多机配置开这一台点名的全部卡，一直持着到进程停。
- 版本对账：认不认领只核 GPU 判器组键（服务真在跑的只有 GPU 判器那一组）。结论版本键跟着编二进制时依赖包开的特性走（判段的测试二进制与服务的 GPU 判器组键相同、结论版本键可以不同），不参与认不认领，结论照待判行里写的那一把存。
- 版本对不上（GPU 判器组键不同）：服务不认领、每行在日志里点名一次（这一份二进制算得出的结论版本键也打在那一行，只给人看）；`control check` 读单元此刻的进程近四小时的日志，有这一行就判不健康、退 1。
## 还没做的

依据：`research/prompts/gpu-judge-service-r2-main-verification.md`「三、改法」与上面那份协议。

- 端到端对拍：判段写待判行、本服务判完写回、判段读回的结论与直接判逐态相同。判段写行等判的入口（验证包 `judge_my_blocks_on_gpu`）与把一条小流录进库的办法
  只在验证包里可调，这个包的测试调不到；要么验证包交出这两样，要么服务装上之后对一条小的崩溃枚举用例开 GPU 跑一趟。
- 版本对不上时判段那一侧不知道：服务不认领那几块，判段的片一直等；只有 `control check` 看得见。判段那一侧要得知，得改验证包与协议。
- `check` 只看近四小时：一块只点名一次，进程跑过四小时之后更早点名、还没人取的块 `check` 不再报。
- 被 systemd 拉起的那一次不清场：清过的场一直清着，用户手动复原了清场停下的服务之后，服务被杀再拉起时不再清一次。
- 一次性库：服务只连全仓那一份库；判段的库是一次性库时待判行它看不见（验证包 `PENDING_ROWS_IN_A_DISPOSABLE_LIBRARY_ARE_UNSUPPORTED`）。
- 读回的材料与登记过的流一直留在进程里，不按库里还有没有它的待判行放掉。
