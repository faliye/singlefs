#!/usr/bin/env python3
"""task-server 的服务：唯一读写库的进程。FastAPI 跑在 uvicorn 上，只开 unix socket（${运行时目录}/server.sock，目录 700、socket 600），不开 TCP。
用户级 systemd 服务 singlefs-task-server.service 起它（python3 task-server/control install 装）；人与 agent 不直接起它，也不直接读写库。
路径、错误类型、重试、资源记录、编译目录、分片、环境变量名单与身份写在 task_server_lib.py 文件头。

  task_server.py serve      常驻；收到 TERM / INT 停下（在跑的任务在各自的单元里接着跑，不杀；服务自己起的探测子进程停掉再退，下面 ③「停服务」）、退 0。
                            同一个 socket 上已有一份在回话退 3；必须依赖、多机配置、签名密钥、每台默认重活线程数不在退 78；socket 路径太长或绑不上退 78
                            （点名那个路径；太长的另报它有多少字节、上限多少，task_server_lib.py 文件头「路径」）；只在自证里认的变量设在了别处退 2；
                            NFS 共享判不过（task_server_lib.py 文件头「NFS 共享」：多机配置没写 NFS_SHARED_ROOTS、哪个根的目录、导出、挂载或两向探针不过）退 78，
                            一句话写明哪个根、哪一项，出路指 task-server/README.md「## NFS 共享」；这一判在 Scheduler 建起之前。
                            NFS 判过之后取共享数据根与工作根（task_server_lib.shared_data_root、work_root：照 .claude/hooks/lib_work_root.py，环境里的 SINGLEFS_DATA_ROOT，
                            没设取多机配置的 SHARED_DATA_ROOT，要落在 NFS_SHARED_ROOTS 某一段下）：取不到退 78，出路点名往 multi-host.env 加 SHARED_DATA_ROOT 那一行；不补缺省。

接口（HTTP over unix socket，JSON；调用方的进程号与 uid 由 SO_PEERCRED 拿，uid 不是服务自己的拒 403；会话号照 task_server_lib.session_of_peer 从进程树上读）：
  GET  /health                    {ok, problem, database_writable, scheduler_seconds_since_tick, tag, queued, running}；problem 只报四样：库写不进；派活判排头能起却连着三轮没起成
                                  （用户：「查清楚为什么总是误报」「根改」：/health 不另判放不放得下，排队线程每一轮排完记下派活给排头的结论（HeadVerdict：判能起而起了、
                                  判能起却没起成及原因、判不能起及原因），只有判能起却没起成的才算，判不能起的（等负载、排在排满时限的任务后面、第二台一轮起满、起之前复核没过……）一律不报，
                                  不认排队原因里的字样；starved_problem）；看进展：某台照公式（room_by_the_formula）放得下排着的最小那条、却连着 PROGRESS_STALL_SECONDS（10 分钟）
                                  一条都没起（不读派活的结论，兜派活自己判错；等负载、等占盘、照排队先后那两条规则等着的不算；progress_problem）；
                                  本机读不到在跑尝试的单元状态（下面 ①「读不到单元状态」：对账记下的那一段，unit_unreadable_report；另带 unit_unreadable 栏逐次列出
                                  {attempt, task, number, host, unit, seconds, presence, detail}；弄坏开关 unit-unreadable-health-unreported 下不报）。
                                  两样都由排队线程每一轮排完算好、存下（starved_text、progress_text），/health 只读存的那一句，不现读余量、不起子进程；queued、running 只数行
                                  （弄坏开关 health-judges-head 下照改前现判排头，health-head-judged-apart 下照改前另判一套，progress-unwatched 下不看进展）；
                                  另带 nfs（每个共享根的四项结论：起来时判的，前三项每 NFS_RECHECK_SECONDS 由复核线程更新）与 peer（第二台用不用得了、原因、从哪一刻起、
                                  最近一次那一趟没跑成；note_peer 记的）：两栏都只读存的，不让 ok 变 false（第二台用不了满一段时间由服务自己发消息给会话，下面 ⑥「服务健康」）；
                                  再带 booking_drift（记账对账判出的漂移：memory 是 {台: {since, rounds, booked, used, attempts: [{label, task, booked, used, cap, why}…], text}}；
                                  没有漂移是空表；只读存的，不让 ok 变 false；下面 ②「内存」的记账对账）；
                                  再带 booking_ledger（在跑记账的逐条明细，记账对账每一轮换上，refresh_booking_ledger：{台: {at, read, slice_total, margin, caps, used, booked, room, slice_room,
                                  rows: [{task, number, label, cap, used, booked, basis, at_cap, text}…], title, total}}；basis 是 stopped、no-scope、measured，说为什么记这么多；
                                  只读存的，不让 ok 变 false；control bookings 逐台打出来，不连第二台）；
                                  再带 paused（暂停着没有）与 pause（{reason, since, allow_session, other_running, text}，text 是「暂停中（原因：…，起于…）、在跑 N 个」，放行了一个会话的接
                                  「、放行会话 <号>」与「、别的会话在跑 M 个」，没暂停是 null）：暂停不让 ok 变 false，暂停中不判「判能起却没起成」，也不看进展（下面 ⑤「暂停」）
  POST /tasks                     {command: [词…], cwd: 绝对路径, environment: {名: 值}, inbox: {socket, token}, whole_package_confirmed: 确认文件}
                                  → {id, session, state, sharded, compiles, inbox, warnings, confirmation, confirm_timeout_seconds}；收下的任务记待确认（state=awaiting-confirmation）、不进队，
                                  confirmation 是确认单（下面 ⑦）；inbox 是会话的收件地址（task submit 读会话环境里的 CLAUDE_CODE_MESSAGING_SOCKET、CLAUDE_CODE_MESSAGING_TOKEN，
                                  读不到的客户端当场拒；下面 ⑥），两样都在才记在任务上，回话的 inbox 只说记没记；令牌不进日志、status、/tasks/<号> 的回话（summary 只留 socket）；环境变量照名单再核、命令里套了包装或派发器的拒、
                                  命令外层与 bash / sh -c 里写了退役变量「名=值」的拒（task_server_lib.py 文件头「环境变量」）、
                                  调用方给的编译目录指到仓里的拒、派不到第二台的三种、临时 prune-server 包装里会编的与不点名包的全仓编译拒（400；这五种的 detail 是
                                  {placement: {problem, next_step}}，下面 ②「提交时拒」；整包测试与不指定范围的跑法没带确认标识的同样 400 {placement}，②「提交时拒」⑥）。
                                  这些拒全在 Scheduler.submit_refusal 一处判，服务起来时重核在排的（revalidate_queued）照同一份判
  POST /tasks/{号}/confirm        {items: 项数, tasks: 这个会话在排在跑的条数} → {confirmed, state, detail}：只许提交它的会话；任务要在待确认；两个数都与服务此刻算的相同才进队，
                                  对不上回 409、detail 是 {mismatch, items, tasks, lines, problem, next_step}（服务算的两个数与确认单）；不在待确认的回 409（下面 ⑦）
  POST /tasks/{号}/long-run-confirm  {why: 理由} → {state, detail}：只许提交它的会话；任务要在跑、累计满两小时、没带 --full-gate；--why 去掉空白不到 12 个字的回 409（下面 ⑦「跑满两小时」）
  GET  /tasks/{号}                任务与它每一次尝试的记录；待确认的带 confirmation（此刻现算的确认单）；排着的带
                                  waiting_for（上一轮判的在等什么）与 queue_position（上一轮全局排第几：position、在排共 queued 条、前面排满时限的 aged_ahead 条）；分片的父任务带 children（各子任务与它们的尝试）；
  GET  /tasks/{号}/wait?timeout=秒  至多等这么久（不超过 60）：结局有了回 {finished: true, …}，没有回 {finished: false}；结局交给提交它的会话时记成已取；
                                  在跑的那一次正疑似卡住、这一段还没报给提交它的会话的回 {finished: false, suspect: {…}}（下面 ①「疑似卡住」）；
                                  这条任务（分片的连同子任务）有还没报给提交它的会话的要处置的提醒（或又跨过一道提醒门槛）的回 {finished: false, attention: [{kind, threshold, task, basis, why, next}…]}，
                                  结局有了的同样在 {finished: true, …} 里带 attention（下面 ①「要处置的提醒」）
  POST /tasks/{号}/cancel         只许提交它的会话：在队里的当场记 cancelled；在跑的 systemctl --user stop 它的单元、回读 ActiveState 与 cgroup.procs 停了，
                                  再等对账写回结局（至多 cancel_settle_seconds）：结局是 cancelled 才回 {stopped: true}；取消到的时候命令已自己结束、写回的是别的结局
                                  （succeeded 这类）回 409 {stopped: false, outcome}，等不到回 500，都不报停了（cancel_outcome_reply；弄坏开关 cancel-outcome-unchecked）。
                                  拿这条任务的锁判（下面 ③「起」）：判放得下之后、记尝试之前到的不起，记了尝试之后到的等单元起完再停、回读（TS-01）。
                                  分片的一片（子任务号）单独取消：只撤那一片，别的片照跑、合并照起并点名缺它（task_server_lib.py 文件头「分片」的「只取消一片」）。
                                  名下有校验不过的尝试或子任务记录（关联列改过，TS-05）、或任务已因校验不过记了 record-tampered 的，一个单元都不停、回 409，不报停了。
                                  分片的父任务连同它在排、在跑的子任务一起取消：父任务先记 cancel_requested、阶段改 cancelling（不先记结局），在排的子任务当场收尾、在跑的逐个停、
                                  回读，再等每一条停了的在库里记成 finished（同不分片的那一种等法，几条共用一个 cancel_settle_seconds；cancel_children），
                                  都到了才给整件记 cancelled、删产物、回 stopped：整件的结局时刻不早于子任务；有没停的、等不到结局的回 500（取消已记下，accepted，点名哪几条子任务），
                                  再取消照样重查、重停活动的子任务；父任务已有结局而子任务还在跑的同样停它们（TS-04）。弄坏开关 parent-finalized-before-children 下停了单元不等结局。
                                  取消的不留产物（用户：「取消的任务要自己删掉产物 不要残留」；remove_cancelled_artifacts）：回读到单元停了、回 stopped 之前，在同一次取消里删这条任务
                                  （分片的连同子任务）每一次尝试的编译目录与现场目录（「这个加入流程中 让task server去做」；设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的也删）；
                                  派到第二台的，照那一次尝试日志里 multi-host-run.sh
                                  的 ⑥ 打的「⑥ 删了第二台这一趟的本地目录 …（回读不在了）」认第二台本地那一份与 runs/ 下三个文件删了（取消时在跑的那一次没有这一行算没删成；
                                  清理回执 peer-cleanup.json 还是 not-started、日志里也没有建了这一趟的本地目录之后才打的两行的，是还没建，不算没删成、不列，
                                  同 confirm_peer_cleanup 认 not-started；弄坏开关 cancel-not-started-receipt-unread 下照算没删成），
                                  上一次带 --keep-build-directory 留着的经 --remove-build-directory 删。删前的大小在取消一开始量（第二台本地盘上的量不到，照写），删后回读不在了才算删了；
                                  逐份写进每条任务最后一次尝试的日志，回话带 artifacts {removed, left, lines}，没删成的进 left、写告警 build-directory-left，task cancel 照它退 5。
                                  它与记下结局时删（remove_finished_build_directories）共一把锁（build_removal_lock），先到的删、后到的回读。弄坏开关 cancelled-artifacts-kept 下不删、不报。
                                  task clean 删编译目录与现场拿同一把锁、锁里现读各次尝试；记下结局时先删完编译目录再叫醒 wait（拿到结局的会话接着 clean 不与这一侧同时删；
                                  弄坏开关 clean-races-finished-removal 下先叫醒、clean 不拿锁）。三处删完记「已删」都只并进 build_directory_removed 那一栏（mark_build_directory_removed、
                                  Store.change），不拿先读的那份尝试整行写回；Store.update 另守库里已 ended 的尝试不写回别的状态（弄坏开关 attempt-row-overwritten 下两样都不守）
  GET  /sessions/self             这个会话在排、在跑与完成了没人取的任务（子任务不单列，列在父任务名下）
  POST /history/import            照服务自己的环境找峰值表与用时表、导进 history 表（task_server_lib.py 文件头「资源记录」）→ {peaks: {read, added, skipped}, durations: {read, added}, paths}
  POST /pins                      {command: [词…], pinned: true|false} → {command_key, pinned, changed, source, bytes_by_host, problem}：给这条命令（键照 task_server_lib.py 文件头「资源记录」的命令键换算）
                                  钉上或撤掉内存上限的钉值（下面 ②「内存」的钉值；control pin-cap / unpin-cap 调它，服务在跑时也能用，不绕过服务写库）；
                                  哪一台那一项减那台此刻的余量不大于 0 的回 422、detail 写哪一台与出路，不钉
  POST /pins/show                 {command: [词…]} → 同上（只读：钉没钉、钉值此刻多少）
  POST /pause                     {reason: 一句话（可空）, allow_session: 会话号（可空）} → {paused, changed, reason, since, running, other_running, allow_session, allow_before, allow_changed}：
                                  暂停排队（下面 ⑤「暂停」）；给了 allow_session，暂停时这个会话交的任务照常排、照常起；已经暂停着的不改原因与起点，changed 为 false，
                                  给了的 allow_session 与现在放行的不同就换（allow_changed 为 true、allow_before 是换之前的）；不给 allow_session 的不动放行；
                                  只认服务自己的 uid（peer_of，同别的写操作）；control pause 调它
  POST /resume                    {} → {paused, changed, reason, since}：放开暂停（reason、since 是放开之前的）；同样只认服务自己的 uid；control resume 调它

排队（一个后台线程，每 TICK_SECONDS 秒一轮，提交与取消时当场叫醒）：
  ① 对账：每一次在跑的尝试读它的结果（attempts/<标签>/result.json，attempt_runner.py 写）。有结果：照 classify_attempt 定类型、照 retry_decision 定重排还是收尾（撞了 250 的那一次下一次的上限乘 1.5，     到或超过那台总上限的照「重试」打回；脚本永远不报自己要多少内存（用户 2026-11-13 定「脚本永远不要报自己要多少」）：上限只照命令键的记录定、撞了乘 1.5，命令打的任何「要多少」行一律不认）。
     没有结果：每 UNIT_CHECK_SECONDS 秒读一次单元的 LoadState 与 ActiveState（照输出判）：在跑的照等；
     状态还是「起单元之前」、起跑标记不在、单元不在（LoadState=not-found）的，是服务在写了单元名与起单元之间没了的那一次：起它一次（同名单元在跑时 systemd-run 自己拒，不会起第二份）；
     别的单元不在了又没留结果的，过 LOST_GRACE_SECONDS 秒记 launcher-lost。
     读不到单元状态（unit_states 交 None：systemctl --user show 起不来、超时或没打 ActiveState）时判不出单元在不在：留了 result.json 的照上面照它的退出码收尾；
     没留的不杀、不收尾、不记结局，尝试照记 running，不设到点就收尾的时限，改用不经 systemd 的办法核这一次还有没有进程（unit_unreadable_gone、attempt_presence）：
     本机读单元那一层（attempt_runner 与它起的包装都在本机的单元里，派到第二台的也是）——起跑标记 started 记的 runner 进程号在 /proc 里在不在，
     本机用户级管理器的 cgroup 树（/sys/fs/cgroup/user.slice/user-<uid>.slice/user@<uid>.service）里名字是这个单元的目录与 attempt_runner 写的 scope 文件点名的目录、
     名字以 MEMORY_CAP_SCOPE_PREFIX 起头而里面有进程的环境带这一次尝试标签的 scope，各自的 cgroup.procs 读出来空不空；派到第二台、清理回执 pending 的，
     第二台照同一段（task_server_lib.attempt_presence_readings；第二台经多机配置的读法 Peer.run 送过去起，同进展读数）读 multi-host-run.sh 记的进程号文件与 scope 单元名文件
     （runs/<标签>.pid、runs/<标签>.scope）；回执 not-started、confirmed 的第二台没有这一次的进程。核法只读 /proc 与 cgroupfs，不起 systemctl、不发信号；
     cgroup 目录不在而上一层列得出算空（已收掉），读不了、起跑标记不在、清理回执读不到、第二台连不上都算核不出。
     核在后台线程里做（probe_unit_presence，每 unit_check_seconds 至多一回；第二台那一趟带 ssh，排队线程不等它），对账照最近一回定（lib.attempt_presence_verdict）：
     有一个进程号在、或有一个 cgroup 里有进程，是还在（照记 running）；都读得到、都空、记下的进程号都不在，连着满 LOST_GRACE_SECONDS 才照没留结果收尾（再读一次 result.json，
     没有的记 launcher-lost；收尾时留下的进程不再经 systemctl 回读，照这一回核的认，stop_leftovers）；有一样核不出、又没有在的，一直等。
     读不到的那一段提醒发起任务的会话（要处置的提醒 unit-unreadable，下面「要处置的提醒」），/health 的 problem 与 unit_unreadable 栏报本机读不到哪几次尝试的单元状态
     （control check 照 /health 不健康退 78）；头一回读不到、之后每跨过一道提醒门槛各往服务日志打一行
     「TASK_UNIT_UNREADABLE time=… attempt=<尝试记录号> task=<任务号> number=<第几次> host=<尝试派在哪台> unit=<单元> seconds=<连着读不到几秒> presence=<alive|gone|unverified|unprobed> detail=…」，
     核到都不在、照没留结果收尾的那一刻再打一行（presence=gone、detail 写核到的）。读到了单元状态就清掉这一段的记账。
     弄坏开关 unit-unreadable-exit-file-unread 下读不到的那一段不认 result.json（核到没了也照没留结果收），unit-unreadable-absent-cgroup-unconfirmed 下 cgroup 目录不在不算空、算核不出，
     unit-unreadable-lost-unverified 下读不到就当场照没留结果收尾（不核进程，收尾照常经 systemctl 回读、停留下的进程），unit-unreadable-unreminded 下不报要处置的提醒，
     unit-unreadable-health-unreported 下 /health 不报。
     续起「起单元之前没了」的那一次拿这条任务的锁、现读任务：
     取消了的照取消收尾、不起（resume_starting）。起单元回执超时过的（start_uncertain_at，下面 ③「起」）单元不在、起跑标记也没有的，过 START_UNCERTAIN_GRACE_SECONDS
     还是这样才确认没起、记 unit-start-failed；单元在跑的照常 running。
     每一轮另把还没结局的分片父任务各现推一遍（drive_sharded_parents，下面 ④；弄坏开关 sharded-parents-unreconciled 下不推）。
     第二台的结局（task_server_lib.py 文件头「错误类型」）：日志里有 multi-host-run.sh 打的「MULTI_HOST_RUN_STEP step=peer-command event=end … exit=<码>」的，
     照那个码（第二台那条命令自己的退出码）定结局，命令自己退 255 记 command-failed、不重排；没有这一行才是 multi-host-run.sh 那一层没跑成（peer-run-failed）。
     弄坏开关 peer-step-exit-ignored 下不看这一行。第二台那一层的内存峰值与编译目录占盘峰值照日志里的 MULTI_HOST_RUN_PEER_MEMORY_PEAK、MULTI_HOST_RUN_PEER_BUILD_PEAK
     记进那一次尝试（不打的记「没量到」），同本机一样进定额度（弄坏开关 peer-memory-peak-ignored 下不读内存峰值那一行）。
     留下的进程（stop_leftovers）：尝试正常结束（有结果、没被停）之后，单元里的由 attempt_runner 退出之前自己停掉、回读，记进 result.json 的 unit_leftovers
     （它一退 systemd 就收单元、当场杀掉剩下的，服务那时已回读不到）；外层 scope（结果里记的 scope，或按尝试标签在 slice 里找到的）由服务回读 cgroup.procs，
     还有进程就对这一次自己的 scope 名发一次 systemctl --user kill --signal=SIGKILL，至多等 LEFTOVER_STOP_SECONDS 秒回读 cgroup 空了没有；
     结局（与重排）的说明前面写留下了几个、哪几个进程号、回读空了没有。弄坏开关 leftover-processes-kept 下两处都不停（runner 照 spec 的 leftovers_kept）。
     收尾回读还有进程或读不了时，尝试保持 starting/running 并写 cleanup_pending，资源占用继续保留，后续对账重查。
     单元已停先放记账（release_stopped_booking）：取消那一侧回读到单元停了（is-active 的输出不在跑、cgroup.procs 读到而且空）、本机尝试收尾一开头同样读到停了的那一刻起，
     这一次尝试不再进 CPU 与内存的在跑记账（unit_stopped、booking_attempts：running_threads、room_by_the_formula、beside_whole_capped、server_pending）；删目录、取峰值、
     落结局、第二台的清理回执照旧做完，记 ended 时清掉这一条；清理没确认（cleanup_pending）的照旧数。读不到状态或 cgroup.procs 的不算停（决不当它停了）。
     弄坏开关 unit-stopped-still-booked 下不放。
     被外部信号杀掉（external_kill_note、kill_log_lines）：命令被不是服务发的 TERM / KILL 终止（构建工具报「signal: 15, SIGTERM」而自己退 1、或退 128 + 信号号），照退出码记的结局是 command-failed、
     服务这一次没有发过停的信号（service_stop_step 空）、单元自己也没收到停的信号（result 的 stopped_signal 为 0）时，读这一台尝试开跑前几秒到结束后 EXTERNAL_KILL_LOG_AFTER_SECONDS 秒的
     earlyoom 日志（journalctl -t earlyoom：「sending SIGTERM to process <号> …」）与内核日志（journalctl -k：「Killed process <号> (…」），点名的进程号在这一次尝试进程树里见过的
     进程号（attempt_runner.py 每轮采样记进 result.json 的 process_ids；Machine.process_ids_seen）里的，记 killed-externally（尝试记 external_kill_note：哪份日志的哪一行、哪个进程号）、
     同一上限重排至多 EXTERNAL_KILL_RETRIES 次（task_server_lib.py 的 retry_decision；改主工作区的命令不重排）。日志读不到、没有对上的进程号仍记 command-failed，不猜。
     第二台的尝试服务这一侧拿不到进程号，由 scripts/hosts/multi-host-run.sh 在第二台那一侧记下这一趟进程树的进程号，命令退非 0 之后读第二台的 earlyoom 与内核日志、
     点名这些进程号之一被杀的打一行「MULTI_HOST_RUN_PEER_EXTERNAL_KILL source=<earlyoom|kernel> pid=<号> line=<日志原行>」（PEER_EXTERNAL_KILL_LINE，log_content 记进 peer_external_kill，
     不算日志在涨），服务读到这一行照本机那一路记 killed-externally、同一上限重排（peer_external_kill_note）；没有这一行的仍记 command-failed。
     内核那一行是「Memory cgroup out of memory」（task_server_lib.py 的 MEMORY_CGROUP_KILL_TEXT：某个内存组到了它自己的上限，不是整机内存紧张）的两台都不认：
     本机的 external_kill_pid 不交进程号，第二台那一行 multi-host-run.sh 照打、服务按 line= 的内容分；照原来的路判（命令自己退非 0，或这一条自己的 scope 撞了上限的照撞上限）。
     弄坏开关 memory-cgroup-kill-counted 下这一种也认。
     进程树里的进程每秒采样一次，活得比一秒短的进程号可能没记到。弄坏开关 external-kill-unrecognized 下不读日志、不认这一行，一律照退出码记，peer-external-kill-line-ignored 下只不认第二台那一行。
     新的远端尝试另有 peer-cleanup.json：起远端命令之前先落 pending，确认进程停下并核目录没有持有者后才写 confirmed；
     失联保留远端目录和 PID/启动时间/退出码，包装已退出时对账经 multi-host-run.sh --recover-cleanup 重查。升级前无此回执的尝试按旧协议处理。
     同一次尝试同一时刻只起一趟 --recover-cleanup（对账、停单元的后台线程与取消那一侧都会来确认；confirm_peer_cleanup 按尝试号拿锁），后到的等它放下再读回执判
     （弄坏开关 peer-cleanup-recovered-concurrently 下各起各的）。
     收尾不在排队线程里做（finish_off_loop）：对账判出该收尾的那一次（有结果、对不回来、起单元之前取消了的），交一个后台线程做
     finish_attempt：第二台的清理回执（--recover-cleanup 经 ssh）、留下的进程回读、记结局时删编译目录都在那个线程里，
     排队线程只看结果，照常对账别的尝试、派活；同一次尝试同一时刻只一个线程在做（finishing），对账与补收尾（settle_ended_attempts）见到它在做就跳过，做完放下、叫醒排队。
     状态机不变：清理没确认的照旧保持 starting/running、记 cleanup_pending。弄坏开关 finish-in-scheduler 下照在排队线程里当场做。
     跑得久而失败（finished_preface）：结局不是 succeeded、这一次跑超过 task_server_lib.LONG_RUN_SECONDS、命令又没声明续跑的，说明前面写「这条跑了 N 分钟、没有续跑：
     重交会从头跑」（task_server_lib.py 文件头「续跑」）。
     命令退 75（prune-server 不可用）：记 prune-server-unavailable，另起一个线程在仓根跑一次 python3 prune-server/control check（PRUNE_SERVER_CHECK_SECONDS），
     不通写告警 prune-server-unavailable；只报，不起、不停 prune-server（check_prune_server）。
     卡住（用户：「就是距离上一次输出日志10分钟没有日志输出」，又选「日志与 CPU 都不涨才停」，又说「至少要延迟十分钟后在判定吧」）：尝试起来后先宽限
     STALL_GRACE_SECONDS（600）秒不判；宽限过了，日志距上一次增长满 STALL_SECONDS（600）秒、
     这一个窗口里 scope 的 CPU 平均不到 STALL_CPU_CORES 个核，服务停掉它、记 stalled（CPU 在涨的是在算，照跑，再看下一个窗口）、
     照常经 wait 叫醒提交它的会话；CPU 本机读 attempt_runner 写的 cpu_usec（各 scope 合计），第二台读日志里 multi-host-run.sh 每轮打的 MULTI_HOST_RUN_PEER_CPU 一行，
     这一行、MULTI_HOST_RUN_PEER_MEMORY 行（下面「顶在上限上卡住」）、MULTI_HOST_RUN_PEER_IO_WAIT 行（下面「卡在 IO 上」）、MULTI_HOST_RUN_PEER_MEMORY_STAT 行（下面「内存最高那一刻」）、MULTI_HOST_RUN_PEER_GPU 行（下面「第二台的显存」）、MULTI_HOST_RUN_PEER_THREADS 行（② 「CPU 闸」的可跑线程数）与 MULTI_HOST_RUN_STEP 行不算日志在涨。已跑多久、日志多久没涨、到此刻平均在用核数
     只写进 task status（running_progress），核数过少不停；头一回量一次尝试（服务刚起来对账在跑的、或宽限刚过）时「日志上一次涨」取日志文件的修改时刻
     （读不到取此刻；服务重启不把日志多久没涨清零），CPU 的窗口照旧从此刻起量。服务不照用时停任务（没有「远超上次用时就停」这一条；用户：「没有预计用时这个东西」）。
     有进展的不停（judge_stall_stop、note_activity、activity_text）：日志没涨的每一轮另读进程内存与峰值（Machine.memory_now）；
     窗口满时进程内存或峰值这一窗口涨了至少 ACTIVITY_MEMORY_BYTES 的不停，从此刻再看一个窗口（贴着上限的不算，照下面「顶在上限上卡住」）。
     弄坏开关 progress-ignored-for-stall 下不看。
     命令自己按固定间隔打的心跳行（lib.HEARTBEAT_LOG_MARK「TASK_SERVER_HEARTBEAT <说明>」；task_server_lib.py 文件头「命令自己打的两种登记行」）不算日志在涨：
     编译、等锁一阵子不出输出的命令靠它在任务日志里留活着的痕迹，真卡死时只剩心跳行，满窗口、CPU 也不涨照样停；夹着真输出的不停。弄坏开关 heartbeat-counted-as-log 下心跳行照算在涨。
     服务重启不杀在跑的任务（它们在各自的单元里），重启之后照这一步对回来。
     顶在上限上卡住（judge_stall_stop；任务 1866 第 4 次：上限 13824M、峰值 13.50 GiB、日志 1288 秒没涨）：照上面两条判卡住的那一刻，这一次尝试的内存
     此刻或峰值已到这一次上限的 STALL_AT_CAP_FRACTION（98%）的，照样停掉，但不记 stalled：记 memory-cap-hit、照重试规矩乘 1.5 重排（钉值的打回 memory-cap-at-maximum），
     结局与重排的说明前面写「顶在上限上卡住：…」；两样都读不到（还没采到 scope、第二台那一层这一轮没打那一行）的照记 stalled。
     判卡住停的是判它的那一次尝试：起的后台线程把那一次的尝试号交给 stop_running、只停那一个单元（线程起来到读库之间那一次自己退了、
     重排起了下一次的，下一次照跑）；取消那一侧照旧按任务停全部。弄坏开关 stall-stop-by-task 下后台线程按任务号停在跑的全部。
     内存只算进程用的（memory.stat 的 anon 加 shmem，memory_peaks_lib.py 的 process_memory_bytes；同 task-server/README.md「算这台还放得下多少」）：页缓存撑满 scope 上限时
     内核回收它、不因此杀进程（任务 8853：anon 921M、file 22163M、memory.peak 顶到上限 50176M），只有进程内存会撞 OOM，撞上限照旧看 memory.events 的 oom_kill 与包装的退出码。
     内存两台同一段判，按台不同的只在读法（Machine.memory_now：本机此刻读外层 scope（尝试目录的 scope 文件记着）的 anon 加 shmem，峰值读 attempt_runner 每轮写进尝试目录的
     memory_peak（它见过的最大）；第二台读日志里 multi-host-run.sh 每轮打的最近一行
     「MULTI_HOST_RUN_PEER_MEMORY poll=<轮数> current_bytes=<字节> peak_bytes=<字节> read=<yes|no>」（PEER_MEMORY_LINE；那一层内存包装 scope 这一轮的 anon 加 shmem 与
     见过的最大，read=no 是这一轮没读到：此刻交 None、峰值照留上一次读到的））。弄坏开关 memory-peak-with-page-cache 下本机照改前读 memory.current 与 memory.peak
     （attempt_runner 记的峰值同样照 memory.peak，attempt_runner.py 文件头 ⑥）。
     贴着上限而 CPU 在涨的同样算卡住（任务 7723 第 2 次：上限 6144M，scope 的 memory.current 6442393600、memory.max 6442450944，memory.pressure full avg300=2.62，
     日志 37702 秒没涨，CPU 平均 2.526 核在回收里打转，跑了 37775 秒没人停）：日志满 STALL_SECONDS 没涨、此刻内存不低于上限的 STALL_AT_CAP_FRACTION 的，
     CPU 在涨、读不到都不算在算，照这一条停掉、记 memory-cap-hit、乘 1.5 重排；此刻内存离上限远的、读不到此刻内存的照上面看 CPU（在涨的照跑）。
     弄坏开关 at-cap-busy-cpu-kept-running 下贴着上限而 CPU 在涨的照跑（改前的样子），at-cap-fraction-ignored 下读得到此刻内存就当贴着上限、不比上限，
     peer-memory-line-unread 下第二台不读 MULTI_HOST_RUN_PEER_MEMORY 那一行（此刻与峰值都交 None；那一行照旧不算日志在涨）。
     卡在 IO 上（任务 8608、8609、8611、8619：本机根盘被一条全目标编译写到盘忙 84%、/proc/pressure/io some avg10=82.44，本机导出的 NFS 跟着慢，第二台上 find、git diff、
     往工作根 mv 的进程都在 D 状态、wchan 是 rpc_wait_bit_killable / nfs_wait_bit_killable，卡了十几分钟，日志不涨、CPU 20 分钟只涨 1.9 秒，被当成卡住停掉；它们是在等 IO）：
     日志没涨的每一轮另读一次这一次尝试此刻在 D 状态（不可中断的 IO 等待）的进程数（note_io_wait；两台同一段，按台不同的只在读法 Machine.io_blocked：本机数尝试目录 started
     记的 runner 进程号连同它的子孙与外层 scope 的 cgroup.procs 里各进程 /proc/<号>/stat 的状态字段；第二台读日志里 multi-host-run.sh 每轮打的
     「MULTI_HOST_RUN_PEER_IO_WAIT poll=<轮数> blocked=<D 状态的进程数> read=<yes|no>」（PEER_IO_WAIT_LINE，这一行不算日志在涨；上一回之后读到几行取最少的，有一行 read=no 算这一回读不到））。
     日志与 CPU 都满窗口不涨、而这一个窗口里每一轮都读到了、都有进程在 D 状态的，不判 stalled、不停，从此刻再看一个窗口；有一轮没有 D 状态的进程或读不到的照原判法判。
     贴着上限那一条照停（贴着上限时回收本身也让进程进 D 状态，不当在等 IO）。每一轮都有 D 状态的进程满 IO_WAIT_ATTENTION_MINUTES 分钟的另报要处置的提醒 io-wait（下面）。
     弄坏开关 io-wait-ignored 下不看 D 状态（改前的样子）。
     内存最高那一刻（撞上限那一次尝试的结局说明与推送写它；任务 10279 峰值 45.46 GiB，包装退出时读的 memory.stat 只剩 anon 311296，进程已被杀、内存已释放）：
     本机由 attempt_runner.py 每轮读外层 scope 的 memory.current，比见过的最高还高就存一份 memory.stat 的 anon、file、file_dirty、file_writeback、shmem，随 result.json 的 memory_stat_at_peak 交回；
     第二台读日志里 multi-host-run.sh 每轮打的「MULTI_HOST_RUN_PEER_MEMORY_STAT poll=<轮数> current_bytes=<字节> anon=… file=… file_dirty=… file_writeback=… shmem=… read=<yes|no>」
     （PEER_MEMORY_STAT_LINE，不算日志在涨），留 current_bytes 最大的那一行（log_content 记进 peer_memory_stat）。finish_attempt 照它写 peak_memory_note（lib.peak_memory_note）。
     弄坏开关 peak-memory-stat-read-at-exit 下两台都留最后读到的那一份（进程被杀之后读到的）。
     跑满两小时提醒（提交时的门禁之外一条任务跑过两小时要提交它的会话确认一次；long_run_unconfirmed_items，判法在下面「要处置的提醒」的 long-run-unconfirmed）：这条任务各次尝试在跑的时长累加
     （结束了的取记下的用时，在跑的取此刻已跑的；排队等的不算；撞了内存上限、照大一档重来的计时归零，只从最后一次撞上限之后那一次起算；run_limit_accumulated），
     满 RUN_LIMIT_SECONDS（7200 秒）服务不停它、不改任务，记要处置的提醒 long-run-unconfirmed；分片的子任务按自己的尝试各自累计（long_run_confirmed_of）。
     不提醒的只有交的时候带了全量门禁标识的（task submit --full-gate：只认提交时的六道门禁、scripts/crash/crash-amplification.sh 或 scripts/gate/gate-staged.sh、不带 --check 与 --item，lib.commit_gate_entry，
     别的命令带它提交时拒；分片的子任务照父任务）与确认过的（下面 ⑦「跑满两小时」）；任务 9604 那种带 --check 的门禁、merge-and-verify、入口脚本、实验都提醒。
     弄坏开关 long-run-unconfirmed-silent 下不提醒，long-run-cap-hits-counted 下撞上限的那几次也累加，
     full-gate-flag-ignored 下带了也照提醒，long-run-confirmed-still-reminded 下确认过的照提醒。
     四小时硬上限（下面 ⑦「四小时硬上限」）：累计的算法同这一条（run_limit_accumulated），满 RUN_HARD_LIMIT_SECONDS（14400 秒）服务停这一次尝试、记结局 over-four-hours 打回；确认过的同样停，只有带 --full-gate 的不判。
     疑似卡住（用户：「不自己终止，告诉负责的对话 自己检查」；check_suspect）：同一宽限过后，整个单元的 CPU 满 STALL_SECONDS 秒平均不到
     STALL_CPU_CORES 个核，不管日志涨不涨：不停它，尝试记 suspect（依据）、写告警 suspected-stall、叫醒在等的 wait（它打 TASK_SERVER_SUSPECT 一行、退 task_server_lib.SUSPECT_EXIT，任务照跑，
     再起 wait 接着等）、task status 写「疑似卡住」与依据。一段只报给提交它的会话一次（服务内存里记报过的段号）；之后 CPU 又涨了（相邻两次读之间平均不少于 STALL_CPU_CORES 个核）
     这一段结束、从此刻再量下一段；读不到 CPU 窗口作废重量。这一次照上面两条停了的不再报疑似。
     要处置的提醒（用户在弹窗里选「task wait 自己报」；三方判决 research/prompts/watchdog-task-half-r1-main-verification.md「三、怎么办」岔路表甲；attention_items）：
     删掉看门狗盯任务那一半之后没人报的五种情形，加改主工作区的命令没正常结束、日志长期不涨而服务不停它、卡在 IO 上而服务不停它、卡上有它的计算进程而服务不停它那四种，
     再加预算贴着整台、跑得久而核没用起来、按给的上限开并行那三种与连着撞上限还在重排那一种（用户：「既然加了给对应对话发消息的功能 就就该把异常和修改意见交给对应的对话」：依据写量到的数、why 写为什么、next 写修改意见），服务不停、不改任务，只在 wait 那一侧交给提交它的会话。
     每一种怎么判、动作、修改意见与没处置时照 task_server_lib.py 文件头「异常表」那一行：next 的头一段是那一行的修改意见（attention_item 经 lib.anomaly_advice 取，
     代码里不另写出路句），后面接这一回的具体数；时长判的那几种在依据末尾写下一回提醒的门槛（「排满 X 秒再提醒一回」这类）；下面每一种只写判法与依据。分片的任务连同它的子任务一起判（提醒点名的 task 是子任务号）：
       queued-too-long   任务在 queued：从提交（重排的从上一次尝试结束）起排满 QUEUE_ATTENTION_MINUTES 分钟，依据写已排多久与排队原因（waiting_reasons），出路照异常表 queued-too-long 那一行（核「在等：…」点名的是哪一台、哪一样）；
       quiet-log-busy    在跑、这一次尝试的日志（不算起法那一层的进度行，log_content）距上一次增长满 QUIET_ATTENTION_MINUTES 分钟，而服务最近一个窗口没停它是因为
                         CPU 在涨或读不到 CPU（上面「卡住」：日志与 CPU 都不涨才停；log_growth 记的免停缘由是 SPARED_CPU_BUSY、SPARED_CPU_UNREAD）
                         （任务 7723 那样在回收里打转的能跑上十个钟头）；按 D 状态（io-wait）免停的、
                         还没判过一个窗口的不报这一条；依据写日志多久没涨，出路照异常表 quiet-log-busy 那一行（核停在哪一步、是不是把输出攒着不打）；弄坏开关 attention-quiet-busy-unreported 下不报，
                         quiet-log-busy-any-exemption 下照改前不看免停缘由；
                         日志不涨而有进展的不报（任务 17334 顶层日志 66200 秒没涨，测试进程的标准输出另指到运行目录里的 record.log 在涨）：日志满 stall_seconds 没涨之后，
                         每隔 stall_seconds 的一半读一回这一趟进程树里各进程打开写着的普通文件与大小（顶层日志不算）（note_attempt_progress；
                         两台同一段 task_server_lib.attempt_progress_readings，第二台经多机配置的读法 Peer.run 送过去起，进程树从清理回执点名的 runs/<标签>.pid 往下），
                         写着的文件比上一回大了算有进展；多久没涨从日志上一次涨与最近一回读到进展里晚的那一刻起算（弄坏开关 quiet-log-busy-progress-ignored 下不看进展，
                         open-file-growth-ignored 下不看文件，open-file-write-mode-unchecked 下只读打开的文件也算写着的）；
       io-wait           在跑、这一次尝试日志没涨的每一轮都读到有进程在 D 状态（上面「卡在 IO 上」，io_waits 记的从哪一刻起），连着满 IO_WAIT_ATTENTION_MINUTES 分钟
                         （中间有一轮没有、读不到或日志涨了就从头算）；依据写最近一轮 D 状态的进程数与本机此刻 /proc/pressure/io 的 some avg60，出路照异常表 io-wait 那一行
                         （核那台的盘与 NFS：/proc/pressure/io、谁在大量写盘、第二台的挂载），IO 缓过来它自己接着跑；弄坏开关 attention-io-wait-unreported 下不报；
       cleanup-pending   在跑的尝试 cleanup_pending（命令已退出、收尾回读没收干净，资源占用保留；finish_attempt）；
       unit-unreadable   在跑的尝试对账连着读不到它的单元状态（上面「读不到单元状态」，unit_unreadable 记的从哪一刻起）满 UNIT_UNREADABLE_ATTENTION_SECONDS（60 秒），之后每翻一倍再报；
                         依据写尝试派在哪台、本机读不到哪个单元的状态、不经 systemd 最近一回核到的（还有进程在、进程都不在、核不出、还没核完，连同点名的进程号与 cgroup），
                         出路照异常表 unit-unreadable 那一行（核本机用户级 systemd、核依据里点名的进程）；服务不杀、不收尾、照记 running（unit_unreadable_items）。
                         弄坏开关 unit-unreadable-unreminded 下不报；
       shard-advance-failed  drive_sharded_parents 推父任务抛错（note_advance_failure 记在内存里、整份另记进库，推通了删；换服务后照库接着数这一回与开始过几回，
                             弄坏开关 advance-episodes-memory-only 下只记内存）；
       workspace-write-interrupted  改主工作区的命令（modifies_workspace）有了结局而没正常结束：结局是 WORKSPACE_WRITER_INTERRUPTED_OUTCOMES 之一（撞上限、被挤掉、
         第二台没跑成、撞顶、取消、卡住被停、对不回来、满四小时硬上限被停；进队那一刻就打回、一次都没起的 over-four-hours 不算）或命令被信号杀（command-failed 而退出码 128 + 信号号，SIGNAL_EXIT_CODES），可能写了一半
         （用户：「这个问题很严重 因为可能会导致补丁没有正常写入」）；依据写哪一次尝试、什么结局，出路点名要核的文件或补丁目录与 git status。
         apply-writer-patch.py 有结局文件（补丁目录里的 PATCH_WRITER_OUTCOME_NAME）的照它判：服务起一次它的 --read-outcome，打全了（退 0）、已退回（退 1）的不报，
         别的报、依据带它打的那几行；结局文件不在的不报（它在动手换之前写开始与文件行）；带 --dry-run 这类只读参数的不报（workspace_write_item，结了局的只核一次）。
         只报给有结局的任务，所以这一行打在 TASK_SERVER_REASON 之前、退码仍是任务的（下面「wait 读到还没报的提醒」）。
         弄坏开关 workspace-write-interrupted-unreported 下不报，apply-outcome-ignored 下不读结局文件、一律照可能写了一半报。
       oversized-budget  在排、在跑，排队那一刻判出这一条在某一台上的记账不小于那台的整片界（下面 ②「预算」，budget_check 记进 budget_notices）：每条只报一次；
                         依据写各台的记账、可用上限、整片界与记账从哪来（记录峰值几次、没有记录的缺省、钉值……），出路照异常表 oversized-budget 那一行（核实范围、分片与同时起的个数）；钉了上限的照样报。
                         弄坏开关 oversized-budget-silent 下不记；
       long-run-idle-cores  在跑、这一次尝试跑满 LONG_RUN_ATTENTION_SECONDS（600 秒）、到此刻平均在用核数不到给的线程数的 LONG_RUN_IDLE_CORE_FRACTION（24/31，线程数 31 时即不到 24 核）：
                         每次尝试只报一次（提醒门槛是尝试的次数），不随时长再报、不再推，依据写跑了多久、给几线程、平均核数与峰值，出路照异常表 long-run-idle-cores 那一行（资源运用是否合理、
                         脚本能不能并行或切片）；只提醒、照跑，读不到 CPU 的不报（long_run_idle_items）。
                         弄坏开关 long-run-idle-silent 下不报，long-run-idle-quarter-threshold 下界是四分之一；
       threads-over-registered  在跑的尝试，服务每一轮读的线程读数（note_threads，② 「CPU 闸」的可跑线程数那一路）里线程总数超过登记数的 THREADS_OVER_REGISTERED_FACTOR（2）倍、
                         其中可跑的（状态 R）也超过登记数、占线程最多的那一个进程自己的线程数也超过登记数的 2 倍（一堆进程加起来多的不报），连着 THREADS_OVER_REGISTERED_ROUNDS（3）轮（同一个采样号读过的不再算一轮，有一轮不超就从头数）：
                         只起一堆不干活线程的编译器、链接器（rustc、ld 的线程多半在睡）可跑的到不了登记数，不报；每次尝试只推一回（提醒门槛是尝试的次数，判出之后不再改），
                         依据写那一轮的线程总数、可跑的个数、登记数与占线程最多的进程名，出路照异常表 threads-over-registered 那一行；只提醒、照跑、不停它。
                         弄坏开关 threads-over-registered-silent 下不报，threads-over-registered-idle-counted 下不看可跑的超没超；
       long-run-split    任务（分片的一片按自己的；分片的父任务没有自己的尝试，不判整件的挂钟）出结局，服务记的用时（各次尝试的 seconds；撞了内存上限、照大一档重来的，从最后一次撞上限之后那一次起算，
                         settled_run_seconds）超过 lib.LONG_RUN_SPLIT_SECONDS（7200 秒）、又不在 .claude/long-run-split-exempt 里（表里的项名等于命令的某个词，long_run_split_exempt）：依据写这是按片或按整条任务的用时判的
                         （.claude/rules/long-run-split.md「判据」：日志里取不出那一项的起止时拿片或整条任务的用时代替），出路指到那份规则「怎么做」；出结局的任务由 push_outcomes 发结局之前那一次与 wait 的结局回话带出；
                         只推一回、不再推。弄坏开关 long-run-split-silent 下不报，long-run-split-cap-hits-counted 下撞上限的那几次也累加，long-run-split-exempt-ignored 下不看豁免表；
       workspace-write-blocked  换钩子的命令（scripts/gate/hook-swap.py，不带 --feed-only）退 7（上一趟换上没写完成，拒绝再换、一个字节没写）：报给被挡的这个会话，依据点名上一趟疑似是库里更早的哪条
                         被杀的换钩子任务（workspace_write_blocked_items），出路照异常表那一行；只推一回、不再推。弄坏开关 workspace-write-blocked-unreported 下不报；
       long-run-unconfirmed  在跑、累计满两小时（上面 ①「跑满两小时提醒」）、没确认、不是带 --full-gate 的：提醒门槛是 1 加撞过上限的次数（撞了上限归零，再满界算跨过一道新门槛），
                         依据写累计与各次用时、命令、范围声明、日志末几行，出路照异常表 long-run-unconfirmed 那一行（核范围与用时，不对的 task cancel 改范围重交，对的照整行 task confirm <号> --why 确认）；
                         只提醒、照跑；没确认的照 ANOMALY_REPUSHED_KINDS 再推。没有会话的（人手 --manual）不推，这一次尝试的日志接一行 lib.LONG_RUN_LOG_MARK 起头的（long_run_sessionless_note，每次尝试一行），status 打一行「长跑待确认：…」，
                         同样可以确认。弄坏开关 long-run-unconfirmed-silent 下不报，long-run-sessionless-unnoted 下人手交的不写日志那一行；
       fills-any-cap     在排、在跑，这条命令键最近 FILLS_CAP_RUNS（3）次跑成的尝试峰值都不低于各自上限的 FILLS_CAP_FRACTION（90%）（任务 8979：按给的上限开工作进程，
                         7 行也记了 45.6 GiB）：依据写三次的上限与峰值，出路照异常表 fills-any-cap 那一行（「这条命令按给的上限开并行，核实」；fills_any_cap_items；只读进程内存口径的行）。
                         撞了上限的那几次不算：撞的那一次峰值本来就贴着上限，照上限一档档撞上去的（任务 9616 那条静态核 6.0、9.0、13.5 GiB）不是按上限开并行。
                         上限照记下的峰值定的那几次（尝试的 prediction 一栏是 record）也不算：峰值贴着上限是定法的结果。
                         弄坏开关 fills-any-cap-silent 下不报，fills-any-cap-counts-cap-hits 下撞了上限的也算进那几次，fills-any-cap-counts-record-caps 下照记录定上限的也算。
       repeated-cap-hits 在排、或下一次在跑（刚收尾而还没定去向的那一刻不判），这条任务末尾连着 lib.LOOP_LEAK_REMINDER_CAP_HITS（3）次已收尾的尝试结局都是 memory-cap-hit
                         （lib.trailing_cap_hits；任务 9868 四次撞上限峰值 1.51、5.50、6.88、9.57 GiB，9731 六次一路到 31G）：每条任务只报一回（不在 ANOMALY_REPUSHED_KINDS 里，
                         第四次再撞也不再报）；依据写连着撞了几次、第几次到第几次，项另带 loop_leak（lib.loop_leak_check_text：要查的两类与各次上限、峰值、是不是一次比一次大、
                         是不是都顶满、最后一次内存最高那一刻的明细），wait 那一行在 basis= 之后打 loop_leak=、推给会话的正文在「出路：」之后另起一行；出路照异常表 repeated-cap-hits 那一行，
                         任务照旧乘 1.5 重排（repeated_cap_hits_item）。弄坏开关 repeated-cap-hits-silent 下不报。
     提醒门槛（reminder_threshold，项里的 threshold）：时长判的那几种，满上面那个时长提醒一回，满它的 2 倍、4 倍……每翻一倍再提醒一回，threshold 是跨过了几道门槛，
     另带 reminded_after_seconds（这一回跨过的那道门槛，秒）；cleanup-pending 的门槛是尝试的次数，shard-advance-failed 的是开始推不动过几回。
     同一条任务同一种提醒、同一道门槛只报一次：报给提交它的会话就记下报到了哪道门槛（别的会话来等照样看得到、不记），跨过下一道门槛才再叫。
     报到的门槛记进库里提醒说的那一条任务的行（正文的 reminded_thresholds：{种类: 报到的最高门槛}，报过的那几回在 reminder_history；经 Store.change 只并进这两栏；note_attention_reported），
     服务重启后照库认、不重发（会话 46 的任务 8819 那 30 片发过一轮，主 agent 重启服务换批之后同一道门槛又发一轮）；服务内存里另留一份照读（reminded_thresholds），库写不进时只记内存、照旧不重报。
     任务行里没有这两栏的当没报过：照此刻跨过的门槛报一回，第几回从 1 数起。
     弄坏开关 attention-reported-memory-only 下照改前只记在内存里，服务重启后再报一次。
     排队过久那一种，排队原因还没判出来（服务刚起来、排队线程这一轮还没判到它；waiting_text 交空串）的这一轮不报、不发，等判出来那一轮再报，原因不空
     （上面那一回重启后的第二轮原因写的是「排队原因此刻读不到」）；弄坏开关 attention-queue-reasonless-sent 下照改前照报、原因写读不到。
     wait 读到还没报的提醒就返回：任务没结局的回 {finished: false, attention}，客户端打 TASK_SERVER_ATTENTION 一行、退 ATTENTION_EXIT；结局已有的（出结局时才判的那几种）
     {finished: true, attention} 打在理由行之前，退码仍是任务的。阈值取值的理由写在常量上（QUEUE_ATTENTION_MINUTES、QUIET_ATTENTION_MINUTES、IO_WAIT_ATTENTION_MINUTES）。
     TASK_SERVER_ATTENTION 那一行的 why= 超过 lib.ATTENTION_WHY_LIMIT（300）个字符的按字符截到那么长、尾巴接「…」（lib.attention_line）；发给会话的那一条另有一行「原因：」写全文，
     全文在那一条里只出现这一回；task status 的「在等：」不截。弄坏开关 attention-why-unclipped 下照全文写。
     这几种与疑似卡住、结局，服务另直接发消息给提交它的会话（下面 ⑥），与 wait 同一份记账：发过的 wait 不再报，wait 报过的不再发。
     服务健康（库写不进、排头能起却一直没起成、看进展、单元不在、NFS、第二台用不了）不归 wait，服务自己盯、发给每个有任务在排在跑的会话（下面 ⑥「服务健康」）；
     服务自己挂了发不了：task wait 连续连不上满 60 秒就退出报错，task submit 连不上当场报。
     弄坏开关 attention-queue-unreported、attention-cleanup-unreported、attention-shard-advance-unreported 各关一种，
     attention-finished-unreported 下结局有了的 wait 不带提醒，attention-reported-every-wait 下报过的照报。
  ② 排队：在排的任务按任务号从小到大逐个看（同一个队，轻重不分，不另设延迟；重排的回到原来的位置；分片的子任务与普通任务同一个队）。行核不过的不起、记 record-tampered、写告警。
     两台同一段逻辑（用户 2026-11-02：「两台型号都是一样的 根本没有必要写 if 第一台 else 第二台这种代码」）：记账、判放不放得下、判卡住与 CPU 用量、记峰值、
       上限收法、编译目录、挑台与盘余量对每台走同一段，台是参数（多机配置 HOSTS 里的地址、那台的总上限与余量、编译根）；按台不同的只在「这一台怎么读、怎么起」的接口
       Machine 里（LocalMachine：服务所在这一台，直接读 cgroup、起内存包装；PeerMachine：HOSTS 里另一台，经 multi-host-run.sh --peer 起、经 ssh 探测），
       台的先后是 MACHINE_ORDER。只给自证证红的弄坏开关里照改前的样子分台的那几处（host-lists-restored、workspace-writers-local-only、peer-overflow-blocked、
       margin-local-for-both、pinned-cap-this-host-only）不算。
     机器：照 task_server_lib.py 文件头「挑机器」：放得下的几台里挑放下之后剩余占比大的（按余量均衡）；撞上限重排不钉台，到了这台最大才换到总上限更大的那台；
       没有只派一台的名单（用户：「【只能在本机跑】 是不能接受的。必须尽快解决 让错误暴露错来」）：计划、合并与各片也不钉台，哪台由余量定；派到第二台跑红的照红，
       由脚本那一侧修，服务不替它挡（弄坏开关 host-lists-restored 下照改前放回 RESTORED_HOST_LISTS 那几张名单与计划的 pin）。第二台在同一棵共享的树里、同一个绝对路径上跑
       （仓、工作根与会话记录目录经 NFS 两台挂在同一绝对路径）；派不派哪一台只看那一台此刻用不用得了与余量（host_eligible），派不过去的三种在提交时就拒了（下面「提交时拒」）。
       改主工作区的工具（hook-swap.py、apply-writer-patch.py、stage-mine.py、scripts/kb/ 下的）派得到第二台（改的就是同一棵树）；合入后验证（merge-and-verify.sh）只在自己拷的快照上
       打补丁与验、主工作区一个字节不写（它文件头「快照（不拿锁）」），不算改主工作区（modifies_workspace）；
       同一件的兄弟片不避开彼此（task_server_lib.py 文件头「挑机器」）：放不放得下只看内存额度，同一台能同时跑几片。
     提交那一路的进出日志（SubmitTrace）：POST /tasks 在事件循环上记下收到的时刻，活在线程池里做（submit_in_thread），回话时打一行（以 SUBMIT_TRACE_TEXT 起头）：
       收到的时刻、结局（收下、拒了哪一种、出错），与进线程、提交时拒、认会话、认任务、记库、提交时警告、确认单、回话各做完时距收到多少秒；平常一条提交只这一行。
       一步本身用了 SUBMIT_TRACE_SLOW_SECONDS（5 秒）以上的当场另打一行：客户端等回话等满 30 秒退 78 时，日志里看得到慢在哪一步。弄坏开关 submit-trace-silent 下不打。
     提交时拒（用户：「改成提交时当场拒（推荐）：不再悄悄改派本机：碰到这三种就在提交时报错、点名是哪个路径或哪一族、出路写怎么改，任务不跑。」；placement_refusal）：
       POST /tasks 收到时判，命中头一种就回 400、不进库、不起尝试，task submit 退 3、打那一句与出路；分片的子任务不另判（父任务提交时判过）。
       每一种在 task_server_lib.py 文件头「异常表」有一行（lib.SUBMIT_REFUSAL_KINDS；提交时警告两种是 lib.SUBMIT_WARNING_KINDS）。
       ① 共享根外的路径（find_outside_shared_path）：工作目录与命令里的绝对路径（只认整个 shell 词：以 / 起头的整个词、「名字=绝对路径」「--选项=绝对路径」的值、
          shell -c 脚本词照 shell 词切开之后的每一个；别的词中间夹着的 / 不算；两台相同的系统路径 HOST_INDEPENDENT_PATHS 不判）有不在多机配置 NFS_SHARED_ROOTS 任何一段下的：
          点名那个路径，出路是改放某一段下。
          落在 VOLATILE_ROOT（/tmp）下的同样拒，那一句另说重启会清 /tmp（问题 93，用户：「防止重启后 内容丢失，但是被认为做完了」）。弄坏开关 outside-path-accepted 下不判。
       ② 共享根（shared_roots_problem）：这一次要交给命令的 SINGLEFS_DATA_ROOT、SINGLEFS_WORK_ROOT（服务自己的，或任务带的白名单变量）有不在 NFS 共享根下的：
          点名变量与值，出路是配多机配置。弄坏开关 shared-roots-outside-accepted 下不判。
       ③ 判定库（verdict_library_refusal）：带不带判定库只照登记表 .claude/gate.d/verdict-library-scripts.tsv 判（registry_families，不另问别的脚本）：
          命令里有词点到登记表里带判定库的一行、而那一行第三列不以 prune-server 起头的：点名那一族（登记的路径），出路是那一族接进 prune-server、把第三列翻成 prune-server；
          登记表读不了、写坏了的同样拒（判不了）。临时 prune-server 的包装剥开、照原命令认。弄坏开关 verdict-library-family-accepted 下不判。
       ④ 临时 prune-server 包装里会编的（temporary_service_heavy_refusal；用户：「旧脚本不兼容 我再说一次 如果错了让 对方重新提交」）：这层包装
          （python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <路径>] -- <原命令>）只给脚本自证用；
          照 task_server_lib.unwrapped_temporary_service 剥出原命令（只用来判，收下的照旧整条执行），原命令照 compiles 的认法有会编的 cargo 的，
          点名原命令，出路是去掉包装重交（验证包的 cargo test 对真服务判）。服务不剥包装去迁就：编译目录只照外层那一条给，
          包装里的认不进来（任务 4229 包装里套 cargo test --release -p singlefs-verification --features gpu，记成不编，编进了仓根 target/）。
          包装自己要现编的另算（量到的：任务 6778 派在本机、退 0，跑的是改包装之前的旧版，里层命令与 6834、6835 不同，不能当本机的样本；
          任务 6834、6835 同一条命令派到第二台、退 0，第二台的编译目录由 multi-host-run.sh 照例给了。推的、没有任务实测过：compiles 为 false 的包装派本机，
          服务不给编译目录，包装的 resolve_binary 没有 CARGO_TARGET_DIR 就退 2）：
          不带 --server-binary 的包装在服务给的编译目录里现编 prune-server，compiles 把外层这一条认成会编的（lib.temporary_service_builds_server），两台都由服务给编译目录；
          带了 --server-binary 的照旧不编。弄坏开关 temporary-service-build-unrecognized 下不认。
          自带临时服务的两份脚本单独的 --selftest（prune_server_client_selftest.py 还有 --selftest-families、prune-server-load-simulation.py）不带 --server-binary 与 PRUNE_SERVER_BINARY 时
          同样自己现编，同样按会编的算（lib.own_server_selftest_split），两台都给编译目录；带了二进制的照旧不编。弄坏开关 temporary-service-own-server-unrecognized 下这两份不认。
          弄坏开关 temporary-service-heavy-accepted 下不判（改前的样子：照外层收下）。
       ⑤ 不点名包的全仓编译（whole_workspace_cargo_refusal；用户：「8105打回 让他重新写重新确认范围」：任务 8105 不带 -p 的 cargo build --offline --all-targets
          预测 49.0 GiB，同一天 8275、8325 同一条，8104、8324 不带 -p 的 cargo clippy --all-targets --all-features，各会话撤掉后都带 -p 重交）：
          提交的这一条在命令位置上直接起的、会编的 cargo（WHOLE_WORKSPACE_SUBCOMMANDS：build、check、clippy、test、run、bench、doc 与它们的单字母别名；认法同 compiles，
          bash -c 与 cd 跟着走，入口脚本与门禁里面起的 cargo 不读、不判；fmt、metadata、tree、clean 不编，不判）带 --workspace / --all 的拒；没带 -p / --package 的，
          照 cargo 找清单的走法（lib.cargo_manifest_scope：起的目录往上第一份 Cargo.toml）落到虚拟清单（只有 [workspace]、没有 [package]：不点名就编每一个成员）的拒，
          起的目录认不出、清单读不了的同样拒（判不了）；落到包自己的清单（scripts/experiment-apparatus、prune-server 这类自成工作区的包，或 cd 进了一个成员的目录）只编那一个包，照收；
          往上一份清单都没有的 cargo 自己就报错，不判。那一句点名哪一条 cargo、在哪个目录起、照哪一份清单；出路是点名要编的包（-p <包>，验证包 -p singlefs-verification），
          整仓要跑的走 merge-and-verify 或提交时的门禁。弄坏开关 whole-workspace-cargo-accepted 下不判。
       ⑥ 整包测试与不指定范围的跑法没带确认标识（whole_package_refusal；.claude/agent-common.md「日常开发的测试不跑整个包」；.claude/main-agent.md「一轮怎么开、怎么收」；
          异常表 whole-package-unconfirmed）：照命令实际跑什么判（判法在 task_server_lib.py 的 whole_package_cargo_test、whole_package_script、unscoped_run_entry）——
          整包测试：命令位置上直接起的 cargo test 不点名 --test、--bin、带测试名的 --lib，或带 --lib --tests、--workspace（带 --workspace 的先被 ⑤ 拒）；
          .claude/gate.d/crash-point-replay.sh 的 --check 点了 product-gpu-real-card-tests、点了 crash-case-replay 而不带 --crash-cases-only、或点了 feature-gated-tests
          而没带 feature-gated:<文件名> 项，点哪一格、点几格都算（--check crash-case-replay --crash-cases-only --item crash-case:<名> 只判点名的崩溃枚举用例，
          --check feature-gated-tests --item feature-gated:<文件名> 只编、只跑点名的挂特性文件，都不算）；
          不给补丁目录、或带 --all-packages 的 scripts/gate/merge-and-verify.sh（带 --all-packages 的那份脚本自己也退 2；补丁改到工作区根的 Cargo.toml、Cargo.lock、.cargo/
          或不落在任何包里的路径的，它判不出测试目标、测试那一步不跑、退 3，不整包跑，这里不另判）。
          不指定范围的跑法：lib.UNSCOPED_RUN_ENTRIES 表里的门禁与入口不带选条参数——提交时的六道门禁与 gate-staged.sh 不带 --check、--item；scripts/crash/crash-point-replay.sh、
          scripts/crash/crash-amplification.sh 不带 --item；scripts/experiment-apparatus/replay.sh 不给实验号（--admission-only 只问准入，不算）；加一种照表加一行。
          带全量门禁标识的（task submit --full-gate，同 ① 的跑满两小时提醒）不判这一种。COMMIT_GATE_ENTRIES（含 scripts/crash/crash-amplification.sh）里不带选条参数而没带
          全量门禁标识的先拒，整包确认标识顶替不了（弄坏开关 commit-gate-without-full-gate-accepted 下不判）。用户点头要在提交之外跑的别的入口照同一个确认标识交。
          弄坏开关 whole-package-cells-all-three-only 下 --check 要三格全点才算整包，merge-all-packages-accepted 下 --all-packages 不算，unscoped-gate-accepted 下不指定范围的照收，
          whole-package-feature-items-uncounted 下点了挂特性文件的 feature-gated-tests 照算整包，whole-package-any-item-exempts 下 feature-gated-tests 带了任何一个 --item 都不算。
          没带 task submit --whole-package-confirmed <确认文件> 的拒；带了的只判形式（lib.whole_package_confirmation_problem：绝对路径、在共享根下、读得到、
          有 ①②③ 起头的三段、每段去掉空白至少 12 个字），不合的同样拒、点名缺哪一段。核过照收，任务记 whole_package_confirmed（那份文件的路径），
          task status、列表、推给会话的正文与日志头打一行「整包已确认：<文件>」（flag_lines）；带它照样受两小时提醒（① 的跑满两小时提醒）。
          形式不合的那一句不论是不是整包测试都拒（给了标识就判它）。弄坏开关 whole-package-unconfirmed-accepted 下整包测试没带标识也照收；
          whole-package-confirmed-exempts-two-hours 下带标识的不提醒。
       ⑦ 启动测试与验证没点名（unnamed_run_refusal；records/2026-11-17-subagent账本自上而下清欠.md 第二百二十九节「收口要做的」；异常表 unnamed-run）：
          判据只一份，task_server_lib.py 的 unnamed_run_problem（范围声明闸 .claude/hooks/run-scope-guard.sh 调同一个函数），对提交的命令与 bash -c 里命令位置上的每一条
          （lib.shell_inner_commands）各问一回：cargo test 没在 `--` 之后带 --exact 与测试全名；scripts/gate/merge-and-verify.sh、scripts/crash/crash-point-replay.sh、
          scripts/crash/crash-amplification.sh 没带 --item；scripts/mutation/prove-red.sh 没给变异名；.claude/gate.d/ 的门禁阶段 --item、--check 都没带，或 --check 点了
          起测试的格（lib.TEST_STARTING_GATE_CELLS）而没带 --item；门禁 crash-point-replay 带了 --item 而照它要跑的格里还有整格跑的（lib.crash_gate_whole_package_cells：
          没给 --check 而点了崩溃枚举用例的照旧九格，crash-case-replay 不带 --crash-cases-only，feature-gated-tests 没点 feature-gated:<文件名> 项，product-gpu-real-card-tests
          不收点名）——拒，说明写哪一种没点名与照这条命令改成点名的样子（门禁 crash-point-replay 那一种写加 --check crash-case-replay --crash-cases-only）。只查不跑的写法（--list、--list-items、--selftest、
          --no-run、`-- --list` 这类）不判。带 --full-gate（提交时那一趟，full_gate_refusal 先核过）或 --whole-package-confirmed（⑥ 先核过确认文件的形式）的不判。
          入口脚本与门禁里面起的不读（只判提交的这一条）。弄坏开关 unnamed-run-refusal-skipped 下不判；判据里的开关见 unnamed_run_problem。
          自证的小世界照 lib.UNNAMED_RUN_SELFTEST_VARIABLE 写 off 不判（各格演别的规矩），判它的 unnamed-* 几格照真的判。
       ⑧ 入口文件头写了领卡、要卡的声明（card_declaration_refusal；.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」；异常表 card-declaration）：
          提交的命令与 bash -c 里命令位置上的每一条起的脚本，文件头前 lib.DECLARED_HEADER_LINES 行里写了 lib.CARD_DECLARATION_MARKS 之一（# task-server-card-lease:、
          # task-server-gpu-cards:、# task-server-gpu-arguments:）的拒，说明头一句是「卡归 GPU 批处理验证服务，任务不领卡」、点名那份脚本与写了的那几种，出路写删掉那一行、
          要卡的那一段交 GPU 服务判（判法 lib.card_declaration_problem）。服务不认这几种声明、不照没写那样收（.claude/agent-common.md「里程碑 17 收尾之前不兼容任何旧东西」）。
          弄坏开关 card-declarations-accepted 下照收。
     提交时警告（用户：「绕过的 那个要做成提醒 谁做错了严厉警告」；repository_target_bypass）：照收照跑、不拒，服务当场打警告，警告进三处——提交回复（客户端打到标准错误）、
       这条任务每一次尝试的日志头（start_attempt 在起单元之前写进日志，分片的子任务照父任务的）、status（task status <号> 与不带号的会话列表）。头一行以「✗ 严厉警告：」起头，
       点名交任务的会话号与任务号、绕过了什么；第二行以「→ 怎么办：」起头写该怎么改。判在 POST /tasks 收下之后、进库之前，记进任务的 target_bypass（几处说明），
       三处照它与会话号、任务号现拼（bypass_warning_lines）。
       ① 取仓根 target/ 下的二进制，绕过服务给的编译目录（任务 7937–7944：长跑脚本默认取仓根 target/ 下合并前的旧二进制，跑出缺包的红）：
          仓根是服务的仓根与主工作树（草稿与临时目录下的仓副本不算）；命令里的词（整个词、「名=值」的值、bash -c 词里的每一段，相对的照工作目录解）落在仓根 target/ 下的，
          或命令位置上起的本仓脚本正文里有指向仓根 target/ 的路径字面（前面接仓根变量、$(dirname …)/../.. 这类拼法、$(git rev-parse --show-toplevel)，接 target/release/ 或
          target/debug/；工作目录就是仓根的另认裸写的 target/release/、target/debug/），就算命中（lib.repository_target_references，读法同读脚本头的声明：command_word_index 剥过之后那一个词）。
          不算：服务给的编译目录与调用方给的 CARGO_TARGET_DIR 指的目录（都在仓外）、草稿与临时目录下的 target/、正文里的注释与弄坏开关那一支、
          自证那一支（带 --selftest 起的不读正文；名字带 selftest 或以 cells 收尾的函数体不读：memory_peaks_lib.py 的 key_fold_cells 那种样本里造的假路径）。
          弄坏开关 repository-target-bypass-unwarned 下不判。
       ② 会话名下有结局没人取（用户同一句；unclaimed_before、unclaimed_warning_lines）：判在 POST /tasks 收下之后、进库之前：交任务的会话名下有父任务（子任务不算）
          结局满 UNCLAIMED_WARNING_MINUTES（30 分钟）还没交给它（collected 为假：服务发的结局消息没送到，也没起 wait 取），记进新任务的 unclaimed_at_submit，
          三处照它现拼：头一行「✗ 严厉警告：会话 <号> 有 <N> 条任务跑完了没人取（<号>、…）：…」，第二行「→ 怎么办：task-server/task wait <号> 取走（…）…」。
          没有会话的（人手）不判。弄坏开关 unclaimed-unwarned 下不判。
       三处都经 submit_warning_lines 取 ①、② 两样（分片的子任务照父任务的）。
       第二台用不了要告警（note_peer）：余量那一轮判出它用不了（连不上、余量查不成、多机配置判不过、NFS 复核判出没挂上共享根、盘余量不到 DISK_FLOOR_BYTES）、
       或那一趟没跑成（peer-run-failed：ssh 被重置、第二台那边的准备没过），写告警 peer-unavailable（用得了变用不了、原因变了才再写一条；盘不够这一种按 DISK_FLOOR_KIND 只写一条），
       /health 的 peer 一栏写明原因，满 PEER_UNAVAILABLE_NOTIFY_SECONDS 服务发「服务健康」给会话（下面 ⑥）；这时只在本机排着的任务排队原因带「第二台此刻用不了：<那一句>」。
       盘余量（Resources.disk_problem；task_server_lib.py 文件头「挑机器」的「盘余量」）：两台同一个下限。读每台余量的那一次（同一个缓存周期）另判那台编译目录所在盘
       （本机是 TASK_SERVER_BUILD_DIRECTORY 所在盘；第二台是多机配置 BUILD_ROOT_BY_HOST 里它那一项，multi-host-run.sh 在它下面建每一趟的本地目录）的余量：
       报告里有 DISK_REPORT_KEY 照它，没有就照那一台的接口现量（Machine.disk_free：本机读盘，第二台起 PEER_DISK_PROBE_TEXT）；不到 DISK_FLOOR_BYTES（50G）或查不成，
       这一轮不给那一台的报告、不派它，原因写「<台>盘只剩 X（下限 Y）」；够了下一轮自己认回。弄坏开关 disk-floor-unchecked 下不判。
     内存：上限与记账只有一条定法，只照服务自己库里的尝试记录（task_server_lib.py 文件头「资源记录」）；脚本与调用方不报内存量，父任务与同一趟的兄弟片的记录不借；自己这把键没有相当的记录的（片照片数），
     与没有记录的命令一样从 4G 起，别的键的记录不借。
       同一命令键最近 RECORD_WINDOW 次跑完的尝试（退 0 的记量到的峰值，撞了这一条上限的记那一次的上限，命令退 250 之前报了要多少的再取不小于报的量加余量，cap_hit_measurement；别的结局不进、不占名额；recorded_measurements）
       取最大，预测、上限与记账都等于它（上限按 MiB 向上取整，不乘 1.5；recorded_prediction）。库里没有这把键跑完的尝试的，预测、上限与记账都取
       task_server_lib.UNRECORDED_MEMORY_DEFAULT_BYTES（4G；unrecorded_prediction）。撞了 250 由 retry_decision 乘 1.5 重排，重排的那一次照它定的上限（next_cap；computed_prediction）。
       钉值先于这一条（pinned_prediction）：分片的各片吃父任务的钉值，计划与合并不吃。
       弄坏开关 recorded-cap-floored 下把下限抬到 4G，钉值之外低于 4G 的上限与记账都抬到 4G（有记录的头一次、重排都兜；memory_floor、memory_floor_applied）。
       内存上限的下限 lib.MEMORY_FLOOR_BYTES（1 GiB；task_server_lib.py 文件头「资源记录」的内存上限下限）：记录的峰值只算进程内存，上限还要装页缓存，峰值很小的命令
       （任务 11127 峰值 5 MiB，上限 3M、5M、8M 都退 250）照记录定不出装得下的上限；computed_prediction 把钉值之外的预测（有记录的头一次、重排、没有记录的）上限与记账都取到
       不低于它（memory_floor_applied），预测的峰值照原样；记账照这个上限（booked_at_cap），收到台的上限为止，所以下限不超过台能给的最大上限；钉值不经这里（钉值低于下限也照钉值）。
       弄坏开关 cap-floor-skipped 下不设下限。换文件、打补丁的工具（workspace_swap_tool：换钩子、打补丁）的下限另取 WORKSPACE_WRITER_FLOOR_BYTES（4 GiB，workspace_writer_floor）：
       撞上限被杀会把换到一半的结局文件留在主工作区、之后所有会话的换上都被挡，所以不照峰值抠；合入后验证不在其内（照这把键的记录定上限）；自证里换了内存下限的世界不加这一条。弄坏开关 workspace-writer-cap-unpadded 下不加。
       读尝试记录只读峰值口径一栏是「进程内存」的行（current_metric；task_server_lib.py 文件头「资源记录」的峰值口径一栏），弄坏开关 old-metric-peaks-read 下连旧行一起读。
     余量（用户：「这部分最大上限看起来要动态计算」「2gb 要动态计算 … 向上取整」；host_margin、host_capacity、host_limit）：每台单条能给到的最大上限 =
       那台 slice 的总上限（多机配置 SLICE_MEMORY_CAP_BY_HOST 里那台那一项，配多少就是多少；第二台另取它那一项的小者）− 那台此刻的余量。
       余量是那台此刻 singlefs-heavy.slice 里回收不掉、又不归任何在跑任务的那一截，按 1 GiB 向上取整（scripts/hosts/memory_peaks_lib.py 文件头「余量」：怎么量、
       读哪几项）；它随那台的余量报告一起读（Resources.snapshot，第二台照 host-dispatch.py 的 probe_peer 在第二台上量），两台各用各的，不存、不跨轮缓存：
       派活、记账、整片的界、钉值、撞上限重排与「上限到了总上限」、起之前复核都照判的那一刻那一份报告里的那一台的数。配置里没有余量的键（写了照多机配置判法 crash-point-replay-shard-configuration-check.sh 认不得的键判不过，不另认）。
       哪一台量不到余量（没报告、cgroup 读不出、第二台探测失败）、或总上限不比余量大、或钉了的命令在那台的钉值不大于 0：那一台这一轮不派，排队原因写哪一台、为什么；
       能派的每一台都这样：不起，结局记 run-condition-unmet（退 78），说明写每一台的原因与出路。不退回固定的数或旧值，也没有让它退回的配置键与运行开关（下面几个弄坏开关只给自证证红）。
       弄坏开关 margin-fallback-fixed 下量不到的退回 2G；margin-local-for-both 下第二台也用本机的余量；margin-unmeasured-waits 下每一台都量不到时照排着等；
       booked-up-to-slice-total 下单条的上限不减余量（照改前收到总上限；钉值照减）。
       准入记账（用户：「当然， 谁超了杀谁 不要连累别人」「检查内存分配 不得大于配置中的设定」；booked_at_cap）：交给 judge_host 的记账量 = 这一次在那台上的上限；
       预测峰值、上限与记账都收到那台总上限减那台此刻的余量为止，钉值、记录、缺省、撞上限重排四种来历一样收，记录比整台大的也不判永远放不下。
       预测峰值比某一台的可用上限大、别处可能装得下（能派的台里有装得下的、有这一轮给不出可用上限的，或多机配置里有一台此刻连不上）的，那一台照永远放不下判、不派
       （hosts_short_of_record，judge_placement、预算那一判与看进展照同一处判），任务去装得下的那台或排着等；能派的每一台都比它小、也没有别处的，
       照判的那一台收到的数起跑，收到最大还撞上限的照 retry_decision 打回 memory-cap-at-maximum。
       弄坏开关 booking-over-host-limit 下钉值之外峰值比可用上限大的照原样记（judge_host 判永远放不下）；record-over-every-host-never-fits 下每一台都比它小时照永远放不下判。
       在跑的记账（running_booked、running_booking）：只照此刻实测记。报告里有它的 scope 的记 min(它的上限, scope 此刻回收不掉的用量)，不加固定余量、不看任何预先估的数；
       报告里没有它的 scope 的（刚起、读不了）记整份上限。各自的上限（memory.max）照旧，超了只杀它自己。
       公式（room_by_the_formula）= 配置的总上限 − 余量 − 在跑的记账之和：记账之和不超过配置的总上限；在跑的上限之和多大都不进公式。
       每条的上限（host_limit）照总上限判（收到总上限减余量为止，上面准入记账）。看进展与记账明细都照同一个总额。
       内存放不下的排队原因（memory_wait_text；judge_host 那一句照它换，memory_verdict_reworded）写明两个数各是什么：账面还能记多少（总上限、余量、在跑的记账之和）、
       slice 实测还放得下多少（总上限减 slice 里进程用的，再减余量与刚起还没读数的那几条之后剩多少），整机空闲减待涨量不够要的空闲的另写一句，
       末尾点名这台记账最多的至多 MEMORY_WAIT_TOP_RUNNING 条在跑任务与各自的记账、实占。弄坏开关 memory-wait-text-unnamed 下照 judge_host 与公式原来那一句写。
       弄坏开关 running-booked-at-cap 下在跑的一律照上限记，running-booked-with-margin 下有 scope 的在实测上加 1 GiB 记（不超过上限）。
       记账对账（audit_bookings，booking_audit_loop）：服务每 BOOKING_AUDIT_SECONDS（60 秒）逐台比一回
       「这台在跑的尝试（booking_attempts）的内存记账之和」与「对账的基准：这台实测的用量（报告里各 scope 回收不掉的用量之和）」；
       记账之和不少于基准的 BOOKING_DRIFT_RATIO（3）倍、而且多出至少 BOOKING_DRIFT_BYTES（8 GiB），连着 BOOKING_DRIFT_ROUNDS（3）轮，判「记账漂移」：
       漂移只来自报告里没有它的 scope、记整份上限的那几条；起跑不满两分钟、照规矩记整份上限的那几条（running_booking 的 young）对账时照实占算、不算漂移
       （弄坏开关 booking-audit-young-counted 下照整份上限算）；
       逐条列各尝试记多少、实占多少、为什么记这么多（取的是此刻实测、实测到了上限，还是报告里没有它的 scope 记整份上限，booking_row），写进 /health 的 booking_drift 一栏（memory：{台: 明细}）与 control check 的输出（「记账漂移」行），并推给被它挡住、还在排的任务所在的会话
       （排队原因里写着「在跑的记账之和」的；push_booking_drift）：同一件事对同一个会话判出来当场推一回，之后照 reminder_threshold（门槛从 BOOKING_DRIFT_PUSH_SECONDS 起、每翻一倍一道）每跨过一道门槛再推一回，同一道门槛不重推，
       对得上了忘掉、再漂移再推。只报不停任务。弄坏开关 booking-audit-skipped 下不对账。
       slice 兜底（slice_floor_problem）：那台 slice 实测还放得下的（报告的 slice_room：总上限 − slice 里进程用的内存）要不小于 SLICE_ROOM_FLOOR_BYTES：
       不到下限的这一轮不派新任务（slice_floor_verdict），排队原因以 SLICE_FLOOR_TEXT 起头、写明是实测余量不够（内存那一判也放不下的接在后面；看进展不算它）。弄坏开关 slice-floor-ignored 下不判。
       每条的上限从哪来（computed_prediction）：只认这把命令键自己的记录（recorded_prediction：近 RECORD_WINDOW 次最大）；自己没有记录的从 4096M 起，别的键、同一趟的兄弟片都不借，
       撞了上限乘 CAP_GROWTH_FACTOR 重来。
       整机内存线（machine_memory_problem）：余量报告（scripts/hosts/memory_peaks_lib.py 的 headroom）带这台整机的 mem_total、mem_available 与 swap_total、swap_free；
       可用内存占总量的比例与 swap 空闲占总量的比例都低于 MACHINE_MEMORY_LINE_FRACTION（12%，earlyoom 在两样都低于 10% 时才杀进程，这一道在它之前停派）的，这一轮不往这台派新任务，
       排队原因以 MACHINE_MEMORY_LINE_TEXT 起头、写两个比例与两个字节数；只挡新派，不停在跑的（看进展不算它）。没有 swap 的机器 swap 空闲按 0% 算（可用内存低于线就停派），报告里读不到这几项的不判。
       弄坏开关 machine-memory-line-ignored 下不判。
       判放不放得下照 host-dispatch.py 的 judge_host，减去的「还没涨到量」那一截 = 服务自己账上这台在跑的每一次尝试 max(0, 它的记账 − 它的 scope 此刻回收不掉的用量)
       加派发器账上刚派出去的那几笔（server_pending）。两台同一段：用量照这一轮读那一台余量的那一份报告（probe_local / probe_peer 量的
       memory_peaks_lib.SCOPES_REPORT_KEY，第二台是同一次 ssh；scope 照里面进程环境的 TASK_SERVER_ATTEMPT_LABEL——每次尝试两台都给，task_server_lib.ATTEMPT_LABEL_VARIABLE——
       对到尝试的 label 上，memory_peaks_lib.py 文件头「在跑的 scope」；几个 scope 带同一个标签（命令里再套内存包装另开的平级 scope）取用量最小的那一份，不往少里记待涨量）。
       报告里没有这一条的 scope（还没起来、读不了、这一轮那台没报告）记整份上限，不记 0。第二台那一层内存包装不排队、不记账（run-with-memory-cap.sh 文件头「排队、记账、挑机器、
       定额度都归仓根 task-server/」那一句、multi-host-run.sh 文件头 ⑤），记账只在这一处，不重复扣。准入只在这里判一处：
       放得下才起；放不下的留在库里排着，不起包装进程去空等；排头放不下时后面的照下面「排队先后」两条规则；
       没满时限的排着的一律不留账（用户：「现在都在数据库里面 记什么账？？？？  每一条能不能启动 是启动时候才查的 记账做什么？」）。
       哪台都等多久也放不下记 never-fits；点名第二台而它用不了同样记 never-fits。
     排队先后（place_queued_into、jump_refusal；用户定「1 超过两个小时的任务阻塞后续的任务。2 插队的任务的时长不得超过正在运行的任务剩余的时长。没有上次运行时间的任务不得插队」）：
       提交满 AGED_WAIT_SECONDS（两小时）还在排的排到最前、先判（Scheduler.aged、queue_order）。只有两条规则，不交替、不封组、不插空、不留位：
       ① 排满时限还起不来的任务（内存、CPU，不论等的是哪一样）挡住排在它后面的全部任务：它起来之前后面的都不起，排队原因写「排在任务 N 后面：它已排满时限」。
          弄坏开关 aged-blocked-lets-others-start 下后面的照常判、照常起。
       ② 前面有没起的任务（没排满时限）时，后面的只许这样的先起：它有上次的运行时长（last_run_seconds：这把命令键上次真跑完用了多久，是记录、不是估的），
          这个时长不超过那台在跑的任务剩余时长里最长的那个（剩余 = 那条任务上次的运行时长减已跑；没有上次时长、或已跑过上次时长的不算数）；
          在跑的都算不出剩余时长时，有上次运行时长的许先起（用户定）；没有上次运行时长的不插队，排队原因写「不插队」。弄坏开关 jump-length-unchecked 下放得下就起。
       服务不预计用时（用户：「没有预计用时这个东西。我们整个项目现在不预计用时。仅仅预测内存。」）：内存照记录定上限、撞了乘 1.5；
       一项实际跑满四小时停掉打回、跑满两小时提醒拆分（下面 ⑦；带 --full-gate 的提交门禁不受这两条限）。
     预算（用户：「一个任务预算是 50GB 明显是有问题的」；budget_check）：排队每一轮判过 never 与 unmet 之后，这一条在派得了的某一台上的记账（booked_at_cap：收到那台
       可用上限为止，上面准入记账）不小于那台整片界的照常排，同时记一条要处置的提醒 oversized-budget（上面 ①）；预测峰值比那台大、别处可能装得下而不派那台的
       （hosts_short_of_record）不算那一台。记账收到可用上限为止，没有超过可用上限的记账，这一判不打回。判不了的这一轮不判。
     钉值（用户：「变异的内存上限直接按照 根目录设置的50gb走」「50 这个是读取的配置 不是随便给的」；pinned_prediction）：库的 history 表里 state=pin 的一行按命令键
       （task_server_lib.py 文件头「资源记录」的命令键）记「这条命令的上限取多机配置 SLICE_MEMORY_CAP_BY_HOST 里派到的那一台那一项」，不记字面的数；定额度先认它：
       预测、上限与记账都取此刻现读的那一项（configured_slice_caps：按台的键照 task_server_lib.py 的 configured_by_host 切、判，地址照 Resources.addresses 映到本机与第二台；
       每一项正整数加 K / M / G / T，写法同 run-with-memory-cap.sh 读它的那一句），判哪一台放不放得下就换成哪一台那一项减那一台此刻的余量
       （booked_at_cap、pinned_on_host，Prediction.memory_by_host 存的是配置里那一项，余量每一轮现量；上面「余量」），
       学到的峰值、撞上限重排的 next_cap 都不顶掉它；撞了上限不放大，照 memory-cap-at-maximum 打回；
       哪一台那一项不比那台此刻的余量大（减完不大于 0）：control pin-cap 当场拒（POST /pins 回 422、说哪一台、出路是改大多机配置里那一项），不钉；
       已钉的命令派活时那一台照「余量」那一段不派、写原因，不兜底、不取下限（弄坏开关 pinned-nonpositive-accepted 下照钉、照派）；
       配置读不到（没有这个键、写法不对、少一台、多了不在 HOSTS 里的地址、认不出 HOSTS 里哪一项是这一台、文件读不了）的这一条不起，结局记 run-condition-unmet（退 78），
       说明写读不到的原因与出路（改对多机配置的 SLICE_MEMORY_CAP_BY_HOST，或 python3 task-server/control unpin-cap <命令…> 撤掉钉值，再重交）；不退回学到的记录。
       弄坏开关 pin-unreadable-predicted 下照改前这一回照常预测、说一句；pinned-cap-this-host-only 下两台都取服务所在这一台那一项；
       pinned-cap-at-slice-total 下照那一项记满、不减余量。
       python3 task-server/control pin-cap / unpin-cap <命令…> 经 POST /pins 写（服务在跑时也能用）；status 写明这一条是钉值。
     预测每条只算一次（用户：「每个任务在运行前读一次 来确认可用的资源量才是对的吧」）：一条在排任务的内存预测（prediction_of 的结果，上面「内存」那一套）
       第一次算出后存下，之后每一轮直接用，不再读峰值表、不再扫记录；next_cap 变了（撞上限放大重排）作废重算；收尾、不在队里了就删掉存的那一份。
       同一条拿它自己那一把锁算（prediction_task_locks），扫记录、读钉值不拿存预测那几张表的锁（prediction_lock）：排队线程算这一条时，确认单与别的任务取预测不等它。
     CPU 闸（cpu_problem；调度溢出审核 CPU-1、CPU-2）：max(那台此刻的负载（1 分钟负载与可跑进程数的大者）, 服务在那台在跑的尝试的线程之和) + 这一条的线程 ≤ 核数 × CPU_OVERSUBSCRIPTION（3.4375，32 核的台线是 110）
       才派（弄坏开关 cpu-oversubscription-three 下照 CPU_OVERSUBSCRIPTION_LOOSE 倍）；在跑的线程一直记着（在 D 状态等 IO 的负载里看不出），与负载取大、不相加。在跑的每一条记几线程（running_threads）：
       起跑满 CPU_MEASURED_AFTER_SECONDS、服务量满了一段的，记它最近量满的那一段的在用核数（latest_running_cores：任务忙完一段闲下来，账上跟着降）：服务每一轮（check_stall）与算账时各记一回
       Machine.cpu_usec 的累计读数（note_running_sample，窗口 CPU_MEASURED_AFTER_SECONDS 秒、留起点那一端），两端相隔满 CPU_MEASURED_AFTER_SECONDS 的算一段的在用核数（两端读数之差 ÷ 相隔秒数），
       取最近那一段，向上取整、至少 1、不超过登记数。各段取最大的那一份（峰值段核数，note_peak_segment、peak_segment_running_cores）只给收尾记的 peak_segment_cores、cpu_gate_threads
       与名额让不让（small_slot_released）用，向上取整、至少 1、
       不超过登记数（弄坏开关 cpu-running-cores-over-registered 下不限；cpu-cores-by-average 下照到此刻的平均在用核数记）：前半段编译、录制用核少，后半段满核的任务按后半段记，不按整次平均记低。
       可跑线程数（机器挤满时在用核数被压低：几条各要 31 核的挤在一台上各只拿到七到十核，照在用核数记，下一条同键的就照十核放进来）：服务每一轮（check_stall 的 note_threads）
       读一回这一次尝试此刻的线程读数（Machine.thread_reading → ThreadReading：本机读 attempt_runner.py 每轮采样写进尝试目录的 lib.THREADS_FILE_NAME，那一条进程树里
       /proc/<号>/task/<线程>/stat 状态是 R 的线程数、线程总数与占线程最多的进程；第二台读日志里 multi-host-run.sh 每轮打的
       「MULTI_HOST_RUN_PEER_THREADS poll=<轮数> runnable=<R 的线程数> total=<线程总数> top=<线程数>:<进程名> read=<yes|no>」（PEER_THREADS_LINE，这一行不算日志在涨；
       上一回读之后接到的几行里 read=yes 的取最大，一行 read=yes 都没有的这一回读不到，一行都没接到不动））。在跑的那一支记
       min(登记数, max(最近那一段的在用核数向上取整, 最近一回读到的可跑线程数))（running_latest_runnable；收尾记的峰值段那一份照 running_peak_runnable：各段取大再取大）；
       读不到线程读数的那一轮只照在用核数。
       收尾时记进尝试 cpu_gate_threads（CPU_GATE_THREADS_COLUMN，finish_cpu_gate_threads）= min(登记数, max(收尾记的峰值段核数, 读到过的可跑线程数的最大))，峰值段核数没记的不记；
       记录读这一栏，没有这一栏的旧尝试不读（等同没有记录）。弄坏开关 cpu-runnable-threads-unread 下不记可跑线程数、只看在用核数。
       靠名额起的让不让出名额（small_slot_released）与收尾记的 peak_segment_cores、cpu_gate_threads 只看峰值段那一支（连同可跑线程数）。
       起跑已满 CPU_MEASURED_AFTER_SECONDS 而服务还没量满一段的（刚重启）照到此刻的累计 CPU ÷ 已跑秒数记（cgroup 记的累计重启不丢；弄坏开关 cpu-restart-unmeasured-by-record 下照下一句）。
       起跑没满 CPU_MEASURED_AFTER_SECONDS 的、量不到的与这一条自己要起的取法相同（threads_by_record）：
       同一命令键近 RECORD_WINDOW 次跑到头的（跑成，或命令自己退非 0 而不是被信号杀的；被停的、撞上限的、被外部信号杀的不算）CPU 闸线程数（尝试收尾时记的 cpu_gate_threads，finish_cpu_gate_threads）
       取最大、向上取整、至少 1、不超过登记数，没有记录（没记 cpu_gate_threads 的尝试不读，这一栏之前的旧尝试同样不读）、尝试没有命令键的量到之前先按登记的线程数记（用户定）；
       在跑的那一条按尝试号存下头一次取的（running_threads_cache），收尾时删；起之前的那一条存在 admission_threads_cache，
       跟内存预测一起清。弄坏开关 threads-record-succeeded-only 下只认跑成的，cpu-wanted-by-registered-threads 下起之前的一律照登记数，cpu-running-unmeasured-registered 下在跑却没实测到的一律照登记数，
       cpu-unrecorded-by-one-core 下没有记录的先按 1 核，cpu-cores-by-average 下记录照尝试的平均在用核数（cores）取平均，
       cpu-threads-registered-only 下在跑的一律照登记数记（32 核的台同时至多 2 条）。
       要核数少的名额（cpu_judgement）：超线的这一条要的核数不超过 CPU_SMALL_TASK_CORES（2）时照样放行，每台同时在跑的、靠名额起的尝试不超过 CPU_SMALL_TASK_SLOTS（4）；
       名额满了照排队，原因写「CPU_SMALL_SLOT_TEXT N 个已用满」。靠名额起的尝试记 cpu_small_slot（一句说明，起的时候照 judge_placement 与起之前复核那一份余量判，cpu_small_slot_note），
       名额照库里在跑的尝试数（booking_attempts），服务重启后照库接着认；起跑那一行（「尝试起了」）写明靠名额起的。读不到核数或负载的那一支不走名额。
       靠名额起的起跑满 CPU_MEASURED_AFTER_SECONDS、峰值段核数（peak_segment_running_cores，与 running_threads 同一个取法）超过 CPU_SMALL_TASK_CORES 的不再占名额（small_slot_released：
       没有核数记录的重活先按 1 核过了闸，量出来不小就把名额让给真要一两个核的）；尝试上的 cpu_small_slot 与起跑那一行照留，量不到 CPU、还没量满一段的照占着。
       弄坏开关 cpu-small-slots-unlimited 下名额不设上限，cpu-small-slots-withheld 下不给名额，cpu-small-slot-heavy-still-counted 下实测超了的也照占。
       余量报告里读不到核数或负载的那一台这一轮不派，排队原因以 CPU_UNREAD_TEXT 起头。弄坏开关 cpu-threads-settling-only 下只记刚起 CPU_SETTLE_BREAK_SECONDS 秒的、与负载相加，cpu-unread-passed 下读不到的放行。
       排队原因里的负载拆开写：余量报告的 CPU 一项带 blocked_threads（此刻状态 D、多半在等 IO 的线程数；本机读 /proc 逐个线程的 stat，第二台走同一份报告，memory_peaks_lib 的 cpu_report），
       原因写成「负载 N（其中等 IO 的线程 M）」；闸的判法不看这一项。弄坏开关 cpu-load-io-unsplit 下不拆开写。
     第二台的余量不在排队线程里读（loop 审核发现 3；Resources.peer_probe_loop）：一个后台线程（task-server-peer-probe）每 PEER_PROBE_REFRESH_SECONDS 连第二台读一回（多机配置的判法、探测与 df
       各带 PEER_PROBE_TIMEOUT_SECONDS 的时限），派活、NFS 复核取它最近一份（peer_snapshot_entry），不在自己的线程里等 ssh；读不成的那一回记下，PEER_PROBE_FAILURE_HOLD_SECONDS 之内不再去连、
       当第二台此刻用不了；最近一份比 PEER_PROBE_STALE_SECONDS 旧的同样当用不了。起之前复核照旧现读那一台（带同样的时限，
       还在不再去连的那一段里的不连）。尝试收尾时对第二台的清理回执与删编译目录仍在排队线程里做。弄坏开关 peer-probe-in-scheduler 下在调用方的线程里现连。
     排队一轮的读数（loop 审核发现 1、6、8、12）：在跑的尝试库没变就不重读（Store.running_attempts）；
       预算那一判的「记账从哪来」只在记 oversized-budget 时算（budget_check）；每一轮开头先清 repass_soon，队空、暂停着的那一轮主循环照 TICK_SECONDS 睡。
       弄坏开关 running-attempts-reread、pass-estimates-recomputed、budget-source-every-pass、repass-kept-when-idle 各退回那一处。
     余量一轮只读一次（PassView）：每台的余量（Resources.snapshot）、服务账上还没涨到量的那一截（server_pending）在这一轮第一次用到时读，
       这一轮里各条共用；这一轮起了一条，它的记账加进这一轮那台的待涨量，后面的照加过的判，同一份余量不判给两条。/health 判排头另读一份。
     编译目录占盘（records/2026-10-31-task-server巡检.md 问题 76；build_disk_problem）：会编的、或这条命令量过编译目录占盘的，那一台盘余量（本机是
       TASK_SERVER_BUILD_DIRECTORY 所在盘，第二台是多机配置 BUILD_ROOT_BY_HOST 里它那一项所在盘，报告里都是 DISK_REPORT_KEY）减去服务账上
       那台在跑的尝试编译目录还要涨的（起的时候记的占盘 − 此刻量到的，量不到的照记的算；server_disk_pending、Machine.build_bytes）要放得下「这条命令的占盘预测 + BUILD_DISK_FLOOR_BYTES」，
       放不下就排着、排队原因点名「编译目录占盘」；预测是同一命令键最近 RECORD_WINDOW 次跑成而量到了占盘峰值的最大值（取法同内存，build_disk_prediction），
       起了的尝试记 disk_booked。弄坏开关 build-disk-unpredicted 下不判（改前只看第二台的下限）。
     起之前复核：判出要起的那一条，起之前现算一次它的预测、现读一次它要去的那台的余量与待涨量（缓存作废），按现读的数再判一次那台放不放得下；
       放得下才起，上限与记账照现算的；放不下的留在队里，排队原因写「起之前复核没过」与现读的那一句；现读的那份换进这一轮，后面的照它判。
       现读的那一台照排队那一轮同样判：盘余量不到下限或查不成的这一轮不派（Resources.recheck 调 disk_problem；第二台的探测不带盘余量，现量的那一份记进报告，
       编译目录占盘照它判；弄坏开关 recheck-disk-unchecked 下不判）；现读的那份过一遍 peer_view（复核这一刻第二台没挂上共享根的当它此刻用不了；
       弄坏开关 recheck-nfs-unchecked 下不过）；现读的那份换进这一轮（PassView.refresh）。
     卡（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：卡归 GPU 批处理验证服务（gpu-verification-server/），它照多机配置开卡、一直持着；这个服务只调配 CPU：
       不按命令或脚本名字认谁要卡、不分卡、不领卡、不清场、不收卡，派活只看内存、CPU 闸与编译目录占盘。入口文件头写了领卡、要卡的声明的提交时就拒（上面「提交时拒」⑧）；
       服务自己环境里带着的选卡变量（task_server_lib.CARD_CHOICE_VARIABLES）不交给尝试（start_attempt；弄坏开关 card-variables-passed-to-attempt 下照交）；
       起的那一条外面不套清场包装（attempt_command）。
     不认合入锁（用户 2026-11-02：「可以 按这个做 不要拿锁了」；验证在快照上跑）：主工作树正在合入时，会编的与派第二台的照常排、照常起。
  ③ 起：每次尝试一个脱离服务的用户级单元 singlefs-task-<库标签>-<任务号>-<第几次>.service（systemd-run --user --collect，KillMode=mixed），先把单元名写进库再起，只起一次。
     照现读的任务（不拿调用方手里的旧字典）备好目录、环境与说明之后拿这条任务的锁（Store.task_lock），claim_attempt 一个事务里判 queued 且没取消、父任务没结局也没在取消整件（admission 审核 F10）、
     分尝试号、插尝试、改任务（TS-01、TS-03；库里 (task_id, number) 唯一），之后起单元，起完才放锁；判不过的不起、备好的目录删掉，取消了的记 cancelled。
     弄坏开关 cancel-race-unguarded 下照改前不拿锁、先插尝试再拿旧字典整行写回任务；attempt-claim-split 下照改前插尝试与改任务分两次提交。
     systemd-run 超时（START_TIMEOUT_SECONDS）只算「起没起不确定」（TS-06：它是发给用户级 systemd 的请求，调用端超时不说明管理器没收下）：照预先定好的单元名
     读回状态与起跑标记，起了的照常 running；判不了的尝试留在 starting、账照记、写告警 start-uncertain，交给对账（上面 ①）。弄坏开关 start-timeout-as-failure 下照改前当没起。
     单元里跑 attempt_runner.py，它照说明起这一条：
       本机：bash scripts/hosts/run-with-memory-cap.sh <上限> bash <仓根>/scripts/hosts/capped.sh <线程数> <命令…>，环境里设 RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER=<任务号>.<第几次>
             （内存包装不再排第二遍，① 每条上限与 ② slice 总上限照旧做内核层兜底）；会编的 cargo 另设这一次的 CARGO_TARGET_DIR（task_server_lib.py 文件头「编译目录」）
       第二台：bash scripts/hosts/multi-host-run.sh --peer --memory-cap <上限> --label <标签> -- env <白名单变量>=<值>… bash <仓根>/scripts/hosts/capped.sh <线程数> <命令…>，
               在任务的工作目录里起：multi-host-run.sh 在第二台同一个绝对路径里 env -C 起（仓与工作根在 NFS 共享根下，两台同一棵树；multi-host-run.sh 文件头 ③ ⑤），
               不送、不取回、不改写路径、不留第二台的树；写出的文件就在共享的树与共享根下，两台都读得到。编译目录由 multi-host-run.sh 在第二台本地盘上这一趟新建、
               跑完删（服务不设、不留，task clean 不管它）。分片那几片同样：计划目录与产物目录在共享工作根下（task_server_lib.py 文件头「分片」），交绝对路径。
               env 那一段带任务的白名单变量与下面「环境」点名的那几个；本机的路径与编译目录、按台取值的（PEER_HOST_SPECIFIC_VARIABLES）、值是本机绝对路径而不在共享根下的不带。
               这一层的环境里另设 MULTI_HOST_RUN_ADMITTED_BY_TASK_SERVER=<任务号>.<第几次>（task_server_lib.py 的 PEER_ADMITTED_VARIABLE）：multi-host-run.sh --peer
               只收服务直接起的这一次，任务里面再带 --peer（两台都一样）它拒、退 255（那份文件头 ①）；env 那一段不带它，本机的尝试不设。弄坏开关 peer-admission-unmarked 下照改前不设。
               弄坏开关 peer-tree-copied 下照改前拷树：加 --send <工作目录> <树里的落点>、命令换到落点里起（multi-host-run.sh 已不认 --send，退 2）。
       只认服务直接起的那一层：内存包装、capped.sh 与 multi-host-run.sh --peer 各认一个准入标记（task_server_lib.py 的 ADMITTED_VARIABLE、
       THREAD_ADMITTED_VARIABLE、PEER_ADMITTED_VARIABLE，值都是 <任务号>.<第几次>），各自读掉、不交给命令，
       于是任务里面的脚本再套这几样当场被拒（退 2 或那一份的用法错退出码），出路是删掉那一层。capped.sh 的那个两台都设（第二台的经 env 那一段带过去）；
       内存包装那个本机设在环境里，第二台的由 multi-host-run.sh 照它拿到的那一份设给第二台那一层内存包装。
     环境：服务自己的环境，加上任务带的白名单变量、TASK_SERVER_TASK、TASK_SERVER_ATTEMPT、TASK_SERVER_ATTEMPT_LABEL（这一次尝试的标签，两台都给：scope 照它对到尝试上）、TASK_SERVER_SOURCE_REPOSITORY（提交方那一份仓的根，本机绝对路径，第二台也给、
       拼两台一致的路径用）、SINGLEFS_DATA_ROOT 与 SINGLEFS_WORK_ROOT（服务这一侧的共享数据根与工作根的真路径，任务自己带了的照任务的；第二台原样带本机的值，
       脚本不经 ~ 与 $HOME 取：第二台的家目录是另一个）、TASK_SERVER_MEMORY_CAP_BYTES（这一次的内存上限，字节、十进制；
       第二台的经 multi-host-run.sh 那一段 env 带过去）、CLAUDE_CODE_SESSION_ID（提交它的会话，读得到才设）、
       分片那几步的 TASK_SERVER_SHARD_* 与 RUN_WITH_MEMORY_CAP_KEY（task_server_lib.py 文件头「分片」）、给内存包装的 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT（退出时峰值，
       task_server_lib.py 文件头「资源记录」）与 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY（现场目录，同一份文件头「现场」；两样都只给包装那一层，包装读了就 unset、
       第二台的由 multi-host-run.sh 设给那边那一层，env 那一段不带）。服务自己环境里的选卡变量不交（上面「卡」）。systemd-run 退非 0 记 unit-start-failed。
     套在外层单元里的服务（自证起的临时服务，与起在一次尝试里的服务；任务 11815 被取消后第二台留着它的自证里临时服务起的 3 个尝试单元，只能等它们自己超时退）：
       服务在自证里（lib.in_selftest）或自己的环境里带着尝试标签（ATTEMPT_LABEL_VARIABLE）时，起来时照 /proc/self/cgroup 认出自己所在的用户级单元
       （task_server_lib.py 的 enclosing_user_unit：user@<uid>.service 之下离它最近的那一段 .service 或 .scope；nested_outer_unit），起每个尝试单元都带 -p BindsTo=<那个单元> -p After=<那个单元>：
       外层单元一停（取消、卡住被停、撞上限，外层的内存包装停它的 scope），systemd 当场停这些尝试单元，不靠它们自己超时；两台同一段（第二台的外层是 multi-host-run.sh 在那边开的 scope）。
       独立的服务单元（真服务）不带：尝试单元本来就要在服务重启之后接着跑。认不出外层单元（不在用户级管理器之下）的照不带起、服务日志打一句。
       弄坏开关 nested-units-unbound 下不带。
     停服务（收到 TERM / INT）：排队线程与 NFS 复核线程停下（各至多 STOP_JOIN_SECONDS）之后，服务自己起的子进程还在的（去第二台探测余量、复核挂载的 bash、ssh、python3 这类，
       KillMode=process 下它们留在服务单元的 cgroup 里，control stop 回读不空）逐个按进程号 TERM、至多等 STOP_CHILD_GRACE_SECONDS 再 KILL，停完才退
       （stop_own_children；弄坏开关 no-detach 下起成子进程的尝试照旧只发 TERM、不算在内）。弄坏开关 probe-children-left 下照改前不等、不停。
  ④ 分片：父任务不进队，子任务（计划、各片、合并）进队；子任务收尾时（advance_parent）与每一轮对账 drive_parent 照库里父任务的阶段与子任务集合现推下一步：计划成了照 plan.json 建各片，
     片都成了（人手单独取消的那几片不算缺，合并时点名，记在父任务的 missing_shards）建合并，合并成了整件成；
     有一步没成整件照那一步的类型收尾、点名是哪一步；一片没成时兄弟片照排照跑，各片收齐了才收尾（task_server_lib.py 文件头「分片」）。
     子任务记录经 children_of 读，有一行校验不过的抛 StoreRefused("tampered")：推父任务的记 record-tampered，status、clean、取消整件写告警、不认那几行（children_refused）。父任务的阶段（phase：plan、shards、merge、cancelling）与片的计划
     （shard_plan，计划读通之后写进父任务）都在库里，推法幂等：缺的子任务照阶段与计划补建（库里 (parent_id, role, shard_index) 唯一，同一步同一片只一个），
     子任务终态提交之后、建了一片之后、写了 outputs.tsv 之后、建了合并之后服务没了的，重启之后与每一轮对账照库里的样子接着推（TS-02）。
     计划那一步往计划目录下放的测试二进制（lib.SHARD_PLAN_BINARIES_NAME 那个子目录；.claude/gate.d/crash-point-replay.sh 文件头「分片」plan）：父任务有了结局（成、败、取消、打回都算；
     finalize_parent）与提交它的会话 task clean 时服务删（shard_binaries_removal）。只删这一个子目录，计划目录别的文件留着；它照真路径要恰好是这一条父任务自己的计划目录下的那一份，
     是符号链接、真路径落在别处的不删，服务日志打一行。删了几份、多大与不删的原因写进操作日志（action shard-binaries）。
     弄坏开关 shard-binaries-kept 下不删，shard-binaries-path-unchecked 下不核路径、照真路径删。
  ⑤ 暂停（control pause / resume；换 prune-server 的二进制、清判定库旧行这类要一段没有任务在跑的空档的活用）：暂停着，服务照收提交、照答 status / wait / log / cancel / health，
     排队线程不起任何新尝试（place_queued_into 开头与每一次 start_attempt 之前都判；分片的子任务、重试重排回队的都是新尝试，一样不起），
     对账照做（在跑的照跑到结局，写回结局、重排回队、分片父任务照推，只是回到队里的不起）；续起「起单元之前没了」的那一次（resume_starting）是在跑的尝试的延续，不算新尝试，照起。
     放行一个会话（control pause --allow-session <会话号>；管 prune-server 的会话要在空档里经 task submit 交清判定库的命令，暂停着起不来，放开之后再起又会和别的会话新起的任务撞）：
     暂停着，任务记的会话（POST /tasks 时 session_of 取的，task status 里「会话 …」那一栏，tasks 表 session 列）等于放行的会话的，照常排、照常起——排队线程只排这些、
     start_attempt_unless_paused 只放这些（Scheduler.allowed_during_pause）；分片的子任务继承父任务的会话，重试重排回队的还是同一条任务，所以都随提交它的会话。
     别的会话（含没有会话的人手任务）照旧排着不起。放行的会话是空串就是不放行谁；人手任务的会话也是空串，空串从不匹配。已暂停着再 pause 带不同的 --allow-session 就换放行的会话
     （原因与起点不改，写一条操作日志 pause，备注「换放行会话 旧 → 新」）；不带 --allow-session 的 pause 不动放行；resume 连放行一起清掉（放行记在暂停态里，暂停态没了放行就没了）。
     暂停态记在库的 meta 表（名 pause，正文 {reason, since, allow_session} 连同签名，放行的会话在签名里；Store.pause_record），服务重启后照旧暂停、照旧放行，直到 resume；
     签名对不上的（绕过服务改过，改放行的会话也算）按暂停算、不放行谁、说明写校验不过。
     pause 与 start_attempt 共用一把锁（Scheduler.start_gate）：pause 返回时，已经开始的起尝试都已记下（running 数得到它们），之后一条新的都不起——
     「等 control check 报在跑 0 个」才不会被一条正在起的挤进来。pause / resume 各写一条操作日志（操作 pause、resume，备注原因）。
     暂停中：/health 与 control check 写「暂停中（原因：…，起于…）、在跑 N 个」，ok 不变；放行了会话的写「暂停中（原因：…，起于…）、放行会话 <号>、在跑 N 个、别的会话在跑 M 个」
     （M 是 state 为 running、会话不是放行的那个的任务数；等 M 为 0 才做空档的活）；排着的任务 status 写「在等：服务暂停中（原因：…）」（Scheduler.waiting_text，
     wait 的排队过久提醒的依据同它），放行会话排着的任务照常写它真在等什么；不判「判能起却没起成」（starved_problem），也不看进展（progress_problem）。
     弄坏开关 pause-queue-ignored（排队线程当没暂停）、pause-start-ungated（pause 与起尝试不共锁）、pause-halts-reconcile（暂停时对账也停）、pause-state-memory-only（暂停态不写库）、
     resume-leaves-cache-paused（resume 只清库不清排队线程读的那份）、pause-health-unreported（/health 不带暂停）、pause-waiting-unreported（排着的任务不写暂停的原因）、
     pause-judges-starved（暂停中照判放得下没起）、pause-allow-ignored（放行的会话也不起）、pause-allow-matches-any-session（放行了谁都起）、
     pause-allow-waiting-paused（放行会话排着的任务也写暂停）、pause-allow-change-ignored（已暂停再带不同的放行会话不换）、resume-keeps-allowed-session（resume 之后下一次 pause 沿用旧放行）、
     pause-allow-memory-only（放行的会话不写库）、pause-allow-unsigned（放行的会话不进签名）、pause-allow-unreported（/health 与 check 不写放行的会话）、
     pause-other-running-miscounted（别的会话在跑数把放行会话的也数进去）各关一种。
  ⑥ 发消息给会话（用户：「不是你通知 而是 task-server 通知发起的对话」「包括报错 长时间没有响应 都可以这样通知」；inbox_loop，一个后台线程，
     每 INBOX_ROUND_SECONDS 一轮，记下结局时 notify_finished 当场叫醒它）：task submit 交来的收件地址记在任务上（上面「接口」POST /tasks），出事往会话的收件 socket 发一条。
     收件地址（inbox_socket_of）：每次发之前按任务记的会话号读会话登记（~/.claude/sessions/*.json，一份一个进程；读法照
     scripts/agents/lib_session_registry.py，lib.session_inbox_socket），取会话号相同、进程还活着的那几份里启动最晚的一份的 messagingSocketPath，一轮里同一个会话只读一次；
     令牌取交任务时带的。会话重启换了 socket、交时的 socket 已经不在的，照新的发；找不到的（会话已关、登记读不了）照下面「发不出去」写告警 inbox-unreachable。
     提醒、疑似卡住与服务健康只发给此刻找得到 socket 的（inbox_reachable；找不到的这一轮不发、不记发失败，wait 照取），结局照发、发不出写告警。
     任务之外的进程空转（watch_runaway_processes，runaway_loop 每 RUNAWAY_SCAN_SECONDS 一轮、两台都扫）：不在任务 slice 里、不是内核线程、累计 CPU 满 RUNAWAY_MINIMUM_CPU_SECONDS 且 CPU 累计占已跑时长不低于 RUNAWAY_MINIMUM_FRACTION 的进程（lib.runaway_processes），
     写告警 runaway-process，只发给它所属的会话（顺着父进程找到登记的 Claude Code 进程；地址取那个会话最近一条找得到收件 socket 的任务；谁的进程提醒谁，不发给别的会话），每 RUNAWAY_RENOTIFY_SECONDS 再报一次；认不出所属的（第二台的、发起的会话已退出的）不等、直接按号终止，写告警 runaway-process-stopped，不发给任何会话；不碰的（lib.runaway_unowned_stoppable：不在登录会话的 scope 里的，也就是 systemd 单元与系统服务；RUNAWAY_PROTECTED_COMMANDS 点名的；本机登记的会话进程与它们的祖先）只写告警。
     认得出所属会话的本机进程报满 RUNAWAY_REPORTS_BEFORE_STOP 次还在，下一次到点服务按进程号停掉（先 TERM、过 RUNAWAY_STOP_GRACE_SECONDS 还在发 KILL，回读 /proc）并告诉那个会话；所属会话发不到（没有任务可取地址）的照样数次数、到次数照样停。只扫这个账号的进程。报过几次、停没停整份另记进库（RESTART_SURVIVING_STATES），换服务后照库接着数（弄坏开关 runaway-reports-memory-only 下只记内存）；上次报它之后才起的同号进程是另一个，从零数（弄坏开关 runaway-reused-pid-carried 下照号接着数）。弄坏开关 runaway-unowned-not-stopped 下认不出所属的不停，runaway-protected-stopped 下不碰的也停， runaway-process-unreported 下不报，runaway-slice-counted 下任务 slice 里的也算，runaway-process-not-stopped 下只报不停，runaway-sent-to-every-session 下发给服务健康的那几个会话。
     弄坏开关 inbox-socket-from-submit-only 下照交任务时记的那一个 socket。往那个 unix socket 发一条：
     先发认证行 {"type":"auth","token":<会话令牌>}，再发一行 {"type":"user","message":{"role":"user","content":<正文>}}，两行先备好再连、一次写完（post_to_inbox；
     行的写法与来源在 task-server/README.md「## 发消息给会话」，文档页没写行格式，升级 Claude Code 之后要复核）。收方空着时 Claude Code 照它起新的一轮。
     正文第一行先写类别与任务此刻的状态，只看第一行就分得出是什么：「task-server：结局（任务已结束）任务 <号> <中文名>（<种类>，退 <码>；第 1 回）」（outcome_headline）、
     「task-server：提醒（任务还在排 / 还在跑，不是结局）任务 <号> <中文名>（<种类>；第 <N> 回，上一回在 <日期>）」（reminder_headline）、
     「task-server：疑似卡住（任务还在跑、服务没停它）任务 <号> 疑似卡住（suspected-stall，第 <段> 段；第 1 回）」（suspect_headline）、或「task-server：服务健康 <哪一样>」；每个种类带中文名（lib.KIND_NAMES），
     英文种类名在括号里；提醒说的是子任务的，状态照那一片的记录（分片的父任务一直是 sharded，三片都在排的提醒写还在排）；第 N 回是这一种提醒这条任务报给会话的第几回（任务行的 reminder_history 记着回数与时刻），重推的写上一回在哪一天。后面只写事实与出路（照 wait 打的那几行：依据、原因、出路；
     确认标识那几行「长跑已确认」「整包已确认」「全量门禁」在依据与出路之后；再是日志），末尾是 wait 那一行原样一个字不动。
     弄坏开关 inbox-headline-state-omitted、inbox-headline-round-omitted、inbox-headline-name-omitted 下首行各不写类别与状态、第几回、中文名，inbox-flag-lines-before-evidence 下确认标识的行排在依据之前。
     发哪几样：
       结局（push_outcomes）：父任务有了结局（wait 会打 TASK_SERVER_RESULT 的每一种）就发，先发还没报的提醒（改主工作区的没正常结束这类），再发结局；
         只发一次；发出去了记成已取（同 wait 把结局交给提交它的会话），会话已经用 wait 取了的不发。这一轮处理到某一条时抛错（库读不了、写不进这类），
         那一条与排在它后面还没处理的放回待发、下一轮接着发，已经发出去记了已取的不放回（不是「发不出去」：那一种照下面不再发）。
         弄坏开关 pushed-outcome-not-collected 下发出去了也不记已取，inbox-pending-dropped-on-error 下抛错时照改前不放回。
       在跑进度（push_progress；用户定的）：任务（分片的连同子任务）有尝试在跑，从最早那一次起跑满 PROGRESS_PUSH_MINUTES（58 分钟）给提交它的会话发一条进度
         （起跑以来多久、子任务几条结束几条在跑几条在排、每一次在跑的尝试在哪一台、起跑以来多久、平均在用几核、
         日志多久没涨（日志不涨时读到过进展的接着写哪份文件在涨；note_attempt_progress）与日志末行；progress_attempt_line），之后每满 58 分钟再发一条；只报、不停、不用处置。
         上一回的时刻记在任务行（progress_pushed_at、progress_pushed_rounds），服务重启后不重发同一回。弄坏开关 progress-not-pushed 下不发，
         progress-line-progress-unnamed 下不写读到的进展。
       结局没人取（repush_unclaimed_outcomes；异常表 unclaimed-outcome）：父任务的结局满 UNCLAIMED_WARNING_MINUTES（30 分钟）还没交给提交它的会话的（发结局的那一条没送到、
         也没起 wait），再推一回，之后每翻一倍（60、120 分钟…）再推；正文第一行「task-server：结局（任务已结束，这条结局还没人取）任务 <号> <中文名>（<种类>，退 <码>；第 <回> 回，上一回在 <日期>）」（上一回的时刻记在任务行的 outcome_repushed_at），写这条结局还没人取、
         此刻还差什么（那一行的修改意见），后面接结局那一条整段；推出去了记成已取。推到第几回记进任务行的 outcome_repushed，服务重启后不重推同一回。
         弄坏开关 unclaimed-outcome-not-repushed 下不再推。
       要处置的提醒与疑似卡住（push_reminders）：在排、在跑的任务每一轮照 attention_items 与尝试的 suspect 判，还没报的发；记账与 wait 同一份（reminded_thresholds、
         suspect_reported）：同一条任务同一种提醒同一道门槛只发一次，发过的 wait 不再报，wait 报过的不再发。弄坏开关 inbox-reported-every-round 下报过的照发。
         没处置时（attention_repush；异常表那几行的「没处置时」）：时长判的每翻一倍跨过一道门槛再推；门槛不随时长涨的（ANOMALY_REPUSHED_KINDS）同一道门槛报过之后还在的，
         这道门槛头一回报出去满 ANOMALY_REPUSH_SECONDS（30 分钟）再推第 1 回、之后每翻一倍再推，第一行写「（<种类>，没处置、第 <回> 回再推；第 <N> 回）」，接两行：
         「之前推过：…」（任务行 reminder_history 记的最近几回：隔了多久、当时的依据）与「此刻还差：…」（这一回的修改意见）；只发消息，wait 那一侧不再报。
         弄坏开关 anomaly-not-repushed 下不再推。
         出路是「确要这么多就照跑」的那几种（ACKNOWLEDGED_KINDS：oversized-budget、fills-any-cap）由提交它的会话 task-server/task confirm <号> --why '<为什么>' 记照跑
         （任务行 anomaly_acknowledged {种类: {why, at}}，不论跑没跑满两小时的界），记过的那一种不再再推；弄坏开关 anomaly-acknowledgement-ignored 下照推。
         认的是此刻开着的与库里记着报过的（budget_notices 只在内存，服务换上之后在跑的那一条没有开着的 oversized-budget）；弄坏开关 anomaly-acknowledge-open-only 下只认开着的。
         分片的子任务那几项按父任务、种类、提醒门槛、原因合成一条（attention_groups；任务 8819 拆成 30 片同时排满 30 分钟，逐片各发一条是 30 条
         只差子任务号的消息）：第一行「task-server：提醒（…，不是结局）任务 <父任务号> <中文名>（<种类>，子任务 <号的区间>，共 <几> 片；第 <N> 回）」，依据、出路与末尾那一行照号最小的那一片
         （merged_attention_text）；只有一片的照旧单发。说子任务的头一轮先压一轮（各片各记提交时刻，相差几毫秒的跨过一轮的边界会拆开），下一轮连同新跨过门槛的一起发；
         发结局之前那一次不压。task wait 那一侧照旧逐片各报一行。弄坏开关 inbox-attention-per-shard 下照改前逐片各发一条、不压。
       服务健康（watch_health、health_problems）：库写不写得进（试写一行）、派活判排头能起却一直没起成与看进展（/health 的 problem 读的那两句）、自己的单元在不在
         （systemctl --user is-active 的输出不在 RUNNING_UNIT_STATES）、NFS 共享前三项（nfs_loop 复核的那一份）、第二台用不了满 PEER_UNAVAILABLE_NOTIFY_SECONDS（180 秒）；
         NFS 复核连第二台的 ssh 一次超时先当「慢」：进复查期（NFS_SLOW_GRACE_SECONDS 秒）隔一会儿复查一回（task_server_lib.py 文件头「NFS 共享」的挂载，peer_shell_rechecked），
         期里通了的那一次只往服务日志记一行「第二台慢（超时、复查通了）：…」、记进 /health 的 nfs 一栏（slow_last：{at, slow}），不推、不当第二台用不了、照派第二台；
         复查期满还不通才算「挂载」不过，正文写「连不上第二台（ssh 超时 …，复查期 … 秒里隔 … 秒复查了 … 回仍不通…）」
         （三个会话各报过一次 30 秒超时的「用不了」，同一刻手跑 ssh 秒回、ping 0.09 ms，第二台 load average 约 60）。弄坏开关 peer-health-single-timeout-reported 下头一次超时就报，peer-recheck-once-only 下复查期里只复查一回；
         查出一样，往每个名下有在排、在跑任务又记着收件地址的会话发一条，出路是 HEALTH_NEXT_STEP；同一样对同一个会话只发一次，这一轮查不出了（好了）就忘掉，再坏再发；
         没有要发给的会话时不查。服务自己挂了发不了：task wait 连续连不上满 60 秒退出报错、task submit 连不上当场报，不另加。
         弄坏开关 health-unwatched 下不查不发，health-alerted-every-round 下每一轮照发。
     发不出去（照会话号找不到、socket 不在、连不上、会话已关；send_to_session）：不重试到死，写一行告警 inbox-unreachable（服务日志、告警记录）、往那条任务最后一次尝试的日志接一行
     lib.INBOX_LOG_MARK 起头的（不算日志在涨），任务照旧；那一样不再发（结局另照「结局没人取」满时限再推）。客户端读不到收件地址的当场拒（task 的 submit；异常表 inbox-missing），
     没记会话令牌的任务（人手经接口直交的）服务日志记一行「这条任务没有会话的收件地址」，出事只经 wait 取。
     要处置的提醒的记账与 wait 同一份，记在库里（上面 ①「要处置的提醒」），服务重启后照库认、不重发；疑似卡住报过的段、服务健康每一样发给了哪几个会话、记账漂移每一件推给每个会话推到哪道提醒门槛，
     都记在换服务不丢的那几份里（RESTART_SURVIVING_STATES 的 suspect_reported、health_notified、booking_drift_notified；弄坏开关 suspect-reported-memory-only、health-notified-memory-only、
     booking-drift-notified-memory-only 下只记内存），服务重启后同一件不再发；结局没人取推到第几回在任务行（outcome_repushed、outcome_repushed_at）；发失败过的（inbox_failed）记在服务内存里，
     那一件没送到过，重启后再试不算重发；
     重启前记下了结局而还没发的不补发（wait 照取，满 30 分钟没人取的照「结局没人取」再推，交任务时照上面 ②「提交时警告」②警告）。
     弄坏开关 inbox-not-sent 下一条都不发，inbox-auth-omitted 下不发认证行。
  ⑦ 交任务的二次确认（提交它的会话确认这一趟的范围与个数对不对；submit、confirmation、confirm、expire_unconfirmed）：
     POST /tasks 收下的任务记 state=awaiting-confirmation（lib.AWAITING_CONFIRMATION_STATE）、awaiting_state 记确认之后该进的状态（queued，分片的 sharded），
     不进队、不建子任务、不起尝试；回话带确认单（confirmation）：整条命令、SINGLEFS_RUN_SCOPE 的四段原文、服务从命令里认出的项数 N（lib.named_scope：--item、--check 的每一格、
     点名的 --test、--bin、--lib 与 --exact 之后的测试名各几个，一个都没点的：整包测试、不带选条参数的门禁与入口、别的 cargo 写「整包、不指定范围」，别的写「范围由脚本里面定，服务算不出」，scope_is_whole 判，弄坏开关 scope-inside-script-unnamed 下一律写前一句；命令是 bash / sh -c 的，外层一个都没点时读脚本里每一条命令位置上点的与起的入口
     （lib.shell_inner_commands，不读被起脚本的正文；弄坏开关 shell-inner-scope-unread 下不读）、
     崩溃用例数与域（lib.crash_scope_text，弄坏开关 crash-scope-unreported 下不打：点名的 crash-case 项数与 --crash-expand-up-to 的值，合入后验证与崩溃点重放入口没点的写「范围由脚本里面定，服务算不出」）、这个会话此刻在排在跑的父任务条数 M（不算它自己与别的待确认的）
     与其中命令键相同或点的项交叠（同一条入口里点了同一项）的逐条列号、预测内存与记录来源（prediction_of、budget_source；算过的预测留着，进队之后照用）、
     异常表里交的时候就判得出的几种（budget_check：贴着整片界的 oversized-budget；fills_any_cap_items），连同那一行的修改意见。
     同一会话的确认串行（session_lock，按会话号的锁）：核两个数与改状态在同一把锁里，后到的照先到的进队之后的条数核。
     确认写 task-server/task confirm <号> --items <N> --tasks <M>（lib.confirm_command）：只许提交它的会话；两个数都与服务此刻现算的相同才进队（拿这条任务的锁改 awaiting_state、
     建子任务；任务号是提交时刻的，照它排位置），对不上回 409、打出服务算的两个数与确认单（异常表 confirm-counts-mismatch）。分片的父任务确认一次，子任务由服务建、不再要确认。
     满 CONFIRM_TIMEOUT_SECONDS（1800 秒）没确认的服务取消（结局 cancelled，说明以「没确认，已取消」起头），照结局发给会话（异常表 unconfirmed）。待确认的取消当场收尾。
     task status <号> 给待确认的打此刻现算的确认单。
     弄坏开关 confirm-skipped 下交了就进队（不等确认），confirm-counts-unchecked 下不核两个数，unconfirmed-kept 下满时限也不取消。
     打回建新号：凡打回的结局，异常表修改意见与推给会话的出路都提醒「不要使用原来的任务号，用 task-server/task submit 创建新的任务号」（lib.BOUNCE_NEW_TASK_REMINDER；弄坏开关 bounce-new-id-reminder-omitted 下不带）。
     跑满两小时（task-server/task confirm <号> --why '<为什么要跑这么久、范围为什么是这些>'，lib.long_run_confirm_command；confirm_long_run）：提醒 long-run-unconfirmed 的出路写两条：
     不对的 task-server/task cancel <号>、改范围重交；对的照整行确认。确认只许提交它的会话（没有会话的人手交的同样照会话号比，空对空）、只对在跑、累计满界、没带 --full-gate 的任务；
     --why 去掉空白至少 LONG_RUN_WHY_MINIMUM_CHARACTERS（12）个字（lib.long_run_confirmation_problem）；不合回 403 或 409，客户端退 3。确认了记 long_run_confirmed {why, at}，之后不再提醒，
     卡住、内存上限、占卡不用、封队列这些判法照旧。标识在 task status、列表、推给会话的正文与日志头都打一行「长跑已确认：<why>」（long_run_lines，经 submit_warning_lines 进前三处），
     确认那一刻在跑的那一次尝试的日志另接一行 lib.LONG_RUN_LOG_MARK 起头的；分片的子任务各确认各的，父任务的 status 连同各子任务打。
     弄坏开关 long-run-confirmed-flag-hidden 下不打标识那一行，long-run-short-why-accepted 下不判字数，long-run-confirm-unchecked 下不核会话、在不在跑与满没满界。
     四小时硬上限（异常表 over-four-hours；judge_run_hard_limit、run_hard_limit_verdict、parent_run_hard_limit）：带 --full-gate 的之外一律判，填过 why、确认过的（long_run_confirmed）同样判，--why 放不过；没有别的放宽。判点：
     跑着的：check_stall 每一轮（连同宽限里，重排之后刚起的那一次照样）算累计（run_limit_accumulated，撞了内存上限的归零），满 RUN_HARD_LIMIT_SECONDS（14400 秒）记 run_hard_limit_stop、只停这一次尝试（stop_over_limit_attempt，没停成的下一轮再停，服务重启了同样接着判、接着停），
     它停下来记结局 lib.RUN_HARD_LIMIT_OUTCOME（这一会儿命令自己退了的照它自己的结局走），settle_attempt 不重排、照结局 over-four-hours 收尾，退 229，推给提交它的会话。服务重启时已在跑的任务同样照这一条判。
     分片（用户定「分片的四小时硬上限按每一片自己的累计用时算」）：各片、计划与合并的子任务各按自己的尝试累计（第一个判点），整件不另判（parent_run_hard_limit 交 None）：
     排队等内存、等 CPU 把片一片片串起来的挂钟不算进谁的账。一片满界的照第一个判点停那一片、记 over-four-hours，兄弟片照排、照跑到底，收齐了照「一步没成」收：父任务记那一片的 over-four-hours，不合并。
     弄坏开关 shard-whole-by-wall-clock 下父任务另算整件有片在跑的挂钟（时段取并集，lib.covered_seconds；片之和不算、全都在排的时段不算、撞了内存上限的那一片它那几次不算），
     满界记 phase=lib.RUN_HARD_LIMIT_PHASE 与 run_hard_limit_stop：在排的子任务当场 cancelled（cancel_origin=hard-limit），在跑的由各自的 check_stall 停（run_hard_limit_verdict 看父任务的阶段），都收尾了父任务才记 over-four-hours。
     改主工作区的命令同样停：结局在 WORKSPACE_WRITER_INTERRUPTED_OUTCOMES 里，wait 与推送照 workspace-write-interrupted 提醒核现场；进队那一刻就打回、一次都没起的主工作区没动，不报。
     被停的这一次不进上次运行时长的记录（不在 lib.RUN_COMPLETED_OUTCOMES）与内存记录（不在 lib.RECORD_COMPLETED_OUTCOMES）。
     弄坏开关 run-hard-limit-skipped 下跑着的两个判点（各次尝试与父任务）都不判，run-hard-limit-full-gate-judged 下带 --full-gate 的也判，run-hard-limit-confirmed-spared 下确认过的不停，run-hard-limit-writers-spared 下改主工作区的不停，
     run-hard-limit-children-spared 下子任务不按自己的累计判，run-hard-limit-parent-phase-ignored 下子任务不看父任务的阶段，run-hard-limit-parent-summed 下整件的挂钟（shard-whole-by-wall-clock 那一支）取片之和，
     run-hard-limit-outcome-unmarked 下停的那一次不记 lib.RUN_HARD_LIMIT_OUTCOME，run-hard-limit-writer-unreminded 下 over-four-hours 不报 workspace-write-interrupted，
     run-hard-limit-shard-sum-judged 下分片的父任务的片之和也算，
     run-hard-limit-sheet-silent 下确认单不写，run-hard-limit-exclusive 下恰好等于上限不算到，run-hard-limit-loose 下比上限少一秒也算到。
弄坏开关读在 task_server_lib.py 的 broken（开关表在那份文件头）。
"""
import os
import sys

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
sys.path.insert(0, SCRIPT_DIRECTORY)
import task_server_lib as lib  # noqa: E402

import asyncio  # noqa: E402
import contextlib  # noqa: E402
import dataclasses  # noqa: E402
import fnmatch  # noqa: E402
import hmac  # noqa: E402
import itertools  # noqa: E402
import json  # noqa: E402
import math  # noqa: E402
import re  # noqa: E402
import resource  # noqa: E402
import secrets  # noqa: E402
import shlex  # noqa: E402
import shutil  # noqa: E402
import signal  # noqa: E402
import socket  # noqa: E402
import sqlite3  # noqa: E402
import struct  # noqa: E402
import subprocess  # noqa: E402
import threading  # noqa: E402
import time  # noqa: E402
import traceback  # noqa: E402
import types  # noqa: E402
from dataclasses import dataclass, field  # noqa: E402

TICK_SECONDS = 30.0             # 用户（records/2026-10-30-编译绕过内存总上限.md）：「每秒循环太浪费资源了 我30s一次就好了」；提交、取消、尝试结束（runner 调 /wake）都当场叫醒，这个间隔只兜底
PEER_STARTS_PER_PASS = 4         # 一轮至多在第二台起这么多条：一齐起几十条时第二台的 sshd 会重置新连接（实测 100 条一齐交时两条 ssh 被重置）
PEER_REPASS_SECONDS = 2.0        # 第二台这一轮起满了：隔这么久再排一轮，不等满 TICK_SECONDS
UNIT_CHECK_SECONDS = 5.0
LOST_GRACE_SECONDS = 10.0
PEER_RECHECK_SECONDS = 60.0
RESOURCE_CACHE_SECONDS = 2.0
# 第二台的余量由探测线程读（Resources.peer_probe_loop；文件头 ②「第二台的余量不在排队线程里读」）：派活只读它最近一份，不在排队线程里等 ssh
PEER_PROBE_REFRESH_SECONDS = 2.0      # 探测线程隔这么久读一回（同 RESOURCE_CACHE_SECONDS：派活读到的至多旧这么多加一回探测的用时）
FAST_PEER_PROBE_REFRESH_SECONDS = 0.1
PEER_PROBE_TIMEOUT_SECONDS = 20       # 探测、df 与读多机配置（Peer）各一回的时限；超时的当这一回连不上
PEER_PROBE_FAILURE_HOLD_SECONDS = 30.0   # 读不成的记下原因，这么久之内不再去连（派活照这一句当第二台此刻用不了），过了再试
PEER_PROBE_STALE_SECONDS = 30.0       # 最近一份比这更旧（探测线程卡在第二台上）：当第二台此刻用不了，不照旧的那份派
START_TIMEOUT_SECONDS = 60
LEFTOVER_STOP_SECONDS = 10.0     # 尝试正常结束之后停掉单元与 scope 里留下的进程，至多等这么久回读 cgroup 空了没有（stop_leftovers；SIGKILL 当场生效，推的）
UNIT_STOP_SECONDS = 150          # 单元被停时给命令收尾的时限：multi-host-run.sh 停第二台那条命令至多等 30 秒再删第二台这一趟的本地目录，内存包装停自己的 scope，留足（推的）
CANCEL_SETTLE_MARGIN_SECONDS = 20.0   # 取消停了单元之后等对账写回结局：没留结果的过 lost_grace_seconds 才记，再加两次读单元的间隔与这一截（推的；cancel_outcome_reply）
CANCEL_SETTLE_POLL_SECONDS = 0.1      # 等结局写回时隔这么久现读一次任务、叫醒排队线程
STARVED_ROUNDS = 3               # 派活判排头能起而连着这么多轮（秒数按一轮的长短乘）没起成，/health 回不健康；判不能起的、队空不动不算
PROGRESS_STALL_SECONDS = 600.0   # 看进展（Scheduler.progress_problem）：一台照公式放得下排着的最小那条、却这么久一条都没起，/health 回不健康。取 10 分钟的依据：与 STALL_SECONDS
                                 # 同一个数（用户定一条任务日志满 10 分钟不涨才算没进展，服务自己没进展用同一把尺）；派活故意让排头等的几种都短得多（第二台一轮起满等 PEER_REPASS_SECONDS
                                 # 2 秒），等负载（CPU 闸）、等占盘、slice 兜底与照排队先后那两条规则等着的另外不算；队列卡死 4 个多小时那一次 10 分钟就报得出来
STALL_SECONDS = 600.0            # 用户定：一条任务的日志距上一次增长满 10 分钟、这段时间 CPU 也几乎没涨，服务停掉它、记 stalled
PEER_CPU_LINE = re.compile(r"^MULTI_HOST_RUN_PEER_CPU poll=([0-9]+) usage_usec=([0-9]+) read=(yes|no)$")   # multi-host-run.sh 文件头 ⑤ 每轮一行
# 第二台那一层内存包装 scope 这一轮进程用的内存（memory.stat 的 anon 加 shmem）与见过的最大，multi-host-run.sh 每轮一行（文件头 ①「顶在上限上卡住」；不算日志在涨）
PEER_MEMORY_LINE = re.compile(r"^MULTI_HOST_RUN_PEER_MEMORY poll=([0-9]+) current_bytes=([0-9]+) peak_bytes=([0-9]+) read=(yes|no)$")
# 第二台那条命令进程树里这一轮在 D 状态（不可中断的 IO 等待）的进程数，multi-host-run.sh 每轮一行（文件头 ①「卡在 IO 上」；不算日志在涨）
PEER_IO_WAIT_LINE = re.compile(r"^MULTI_HOST_RUN_PEER_IO_WAIT poll=([0-9]+) blocked=([0-9]+) read=(yes|no)$")
# multi-host-run.sh 文件头 ⑤「线程」每轮一行：第二台那一趟进程树里状态 R 的线程数、线程总数、占线程最多的进程（线程数:进程名，名字里别的字换成 _）；文件头 ②「CPU 闸」的可跑线程数
PEER_THREADS_LINE = re.compile(r"^MULTI_HOST_RUN_PEER_THREADS poll=([0-9]+) runnable=([0-9]+) total=([0-9]+) top=([0-9]+):([A-Za-z0-9._-]+) read=(yes|no)$")
# 第二台那一层内存包装 scope 这一轮的 memory.current 与 memory.stat 的五样，multi-host-run.sh 每轮一行（文件头 ①「内存最高那一刻」；不算日志在涨）
PEER_MEMORY_STAT_LINE = re.compile(r"^MULTI_HOST_RUN_PEER_MEMORY_STAT poll=([0-9]+) current_bytes=([0-9]+) anon=([0-9]+) file=([0-9]+) file_dirty=([0-9]+) file_writeback=([0-9]+) shmem=([0-9]+) read=(yes|no)$")
PEER_EXTERNAL_KILL_LINE = re.compile(r"^MULTI_HOST_RUN_PEER_EXTERNAL_KILL source=(earlyoom|kernel) pid=([0-9]+) line=(.*)$")   # multi-host-run.sh 命令退非 0 之后读第二台日志打的一行（文件头 ① 与那份文件头 ⑤「被外部信号杀掉」）
SELF_CGROUP_PATH = "/proc/self/cgroup"   # 套在外层单元里的服务认自己所在单元读的那一份（文件头 ③「套在外层单元里的服务」）
IO_PRESSURE_PATH = "/proc/pressure/io"   # 本机此刻的 IO 压力（io-wait 提醒的依据写它的 some avg60；读不到照写读不到）
# 判卡住的窗口里算「有进展」的量（note_activity、activity_text；文件头 ①「卡住」的有进展）：进程内存或这一次的内存峰值这一窗口涨了至少这么多。
# 取 64 MiB（推的）：读数自己的抖动在几 MiB 以内，分配器一回拿的块在 2 MiB 到几十 MiB
ACTIVITY_MEMORY_BYTES = 64 * 1024 ** 2
QUEUE_ATTENTION_MINUTES = 30     # 排队过久的提醒门槛（分钟；文件头 ①「要处置的提醒」）：排满这么久 wait 叫醒一次，之后每翻一倍（60、120 分钟…）再叫一次。取 30 的理由（推的）：常见排队几分钟到十几分钟，
                                 # 满 30 分钟多半是第二台用不了、余量被大任务占着或预测过大；比 AGED_WAIT_SECONDS（两小时升高优先）早，叫醒时人还来得及改命令重交
QUIET_ATTENTION_MINUTES = 30     # 日志长期不涨而服务没停它的提醒门槛（分钟；文件头 ①「要处置的提醒」的 quiet-log-busy）：日志满这么久没涨 wait 叫醒一次，之后每翻一倍（60、120 分钟…）再叫一次。
                                 # 取 30 的理由（推的）：项目规则要单项跑得超过 5 分钟的每 5 分钟至少打一行进度（.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」），
                                 # 满 30 分钟没涨是漏了六回；任务 7723 日志 37702 秒没涨，服务一直不停也没人叫
IO_WAIT_ATTENTION_MINUTES = 30   # 卡在 IO 上而服务没停它的提醒门槛（分钟；文件头 ①「要处置的提醒」的 io-wait）：每一轮都有进程在 D 状态满这么久 wait 叫醒一次，之后每翻一倍再叫一次。
LONG_RUN_ATTENTION_SECONDS = 600   # 跑得久而核没用起来（文件头 ①「要处置的提醒」的 long-run-idle-cores）：一次尝试跑满这么久才看它平均在用几个核
LONG_RUN_IDLE_CORE_FRACTION = 24 / 31   # long-run-idle-cores 的界：到此刻平均在用核数不到给的线程数的这么多（线程数 31 时即不到 24 核）才报，只提醒、照跑
LONG_RUN_IDLE_QUARTER_BREAK = 1 / 4     # 弄坏开关 long-run-idle-quarter-threshold 下那一种的界
FILLS_CAP_RUNS = 3                  # fills-any-cap：同一命令键最近这么多次跑成的尝试（只读进程内存口径的行；撞了上限的不算）都贴着各自上限才报
FILLS_CAP_FRACTION = 0.9            # 那一种的「贴着」：峰值不低于那一次上限的这么多（90%）
RUNAWAY_SCAN_SECONDS = 300.0       # 任务之外的进程空转（文件头 ⑥）每隔这么久扫一次两台
FAST_RUNAWAY_SCAN_SECONDS = 0.2    # 自证里的间隔
RUNAWAY_MINIMUM_CPU_SECONDS = 1800  # 累计 CPU 满这么多秒才算
RUNAWAY_MINIMUM_FRACTION = 0.5     # CPU 累计占已跑时长不低于这么多才算
RUNAWAY_RENOTIFY_SECONDS = 1800.0  # 还在的每隔这么久再报一次
RUNAWAY_REPORTS_BEFORE_STOP = 2    # 认得出所属会话的本机进程报过这么多次还在，下一次到点服务按号停掉（共享规则 command-safety.md：报了没响应，报告的一方可以终止）
RUNAWAY_STOP_GRACE_SECONDS = 10.0  # 先 TERM，过这么久还在发 KILL
PROCESS_LIST_SECONDS = 30          # 读一台的进程表至多等这么久；第二台按号停的时限是两段宽限再加这么久
INBOX_ROUND_SECONDS = 30.0       # 发消息那一路（Scheduler.inbox_loop；文件头 ⑥）每隔这么久扫一遍提醒、疑似卡住与服务健康（同排队线程一轮的长短）；结局一有就当场叫醒它，这个间隔只兜底
FAST_INBOX_ROUND_SECONDS = 0.5   # 自证里的间隔（TASK_SERVER_SELFTEST_FAST=1）
INBOX_SEND_SECONDS = 5.0         # 连会话的收件 socket、写完两行的时限：消息先备好再连，一次写完（Claude Code 30 秒里收不到一整行就关连接，cross-session-messaging 文档「The session's inbox socket」）
INBOX_DRAIN_SECONDS = 0.5        # 写完、关了写的那一半之后等对面收完关连接至多这么久（同 socat 读到输入完了之后的缺省等法）
PEER_UNAVAILABLE_NOTIFY_SECONDS = 180.0   # 第二台用不了满这么久才发「服务健康」：余量查不成这类一两轮就好的不叫醒人（note_peer 当场写告警）
FAST_PEER_UNAVAILABLE_NOTIFY_SECONDS = 1.0
PROGRESS_PUSH_MINUTES = 58   # 在跑进度（文件头 ⑥「在跑进度」；用户定的）：任务（分片的连同子任务）有尝试在跑满这么久，给提交它的会话发一条进度，之后每满这么久再发一条
UNCLAIMED_WARNING_MINUTES = 30   # 交任务时会话名下有结局满这么久没交给它（消息没送到、也没起 wait）的，打严厉警告（文件头 ②「提交时警告」②）：消息当场就发，满 30 分钟还没人取的是发不出去而会话也没取；
                                 # 结局满这么久没人取的服务再推一回、之后每翻一倍再推（文件头 ⑥「结局没人取」，异常表 unclaimed-outcome）
CONFIRM_TIMEOUT_SECONDS = 1800   # 交了没确认满这么久服务取消、推「没确认，已取消」（文件头 ⑦；异常表 unconfirmed）：与没人取的结局同一个时限
ANOMALY_REPUSH_SECONDS = 1800    # 门槛不随时长涨的提醒同一道门槛推过之后满这么久还在的再推一回、之后每翻一倍再推（文件头 ⑥「没处置时」；异常表那几行的「没处置时」）
ANOMALY_REPUSHED_KINDS = ("oversized-budget", "fills-any-cap", "long-run-unconfirmed", "cleanup-pending", "shard-advance-failed")   # 提醒门槛不随时长涨的那几种（lib.ANOMALY_TABLE_HEAD 那一段同名）
ACKNOWLEDGED_KINDS = ("oversized-budget", "fills-any-cap")   # 出路是「确要这么多就照跑」的：task confirm <号> --why 记照跑（anomaly_acknowledged），记过的不再再推
ATTENTION_HISTORY_KEPT = 8       # 任务行里每一种提醒记下的推过的那几回（reminder_history）至多留这么多回：再推的正文照它写前几次推了什么
STALL_GRACE_SECONDS = 600.0      # 用户：「至少要延迟十分钟后在判定吧」「如果准备时间或者编译时间长 这不是很容易被杀掉」：尝试起来后先宽限这么久不判，之后才量窗口
CPU_OVERSUBSCRIPTION = 3.4375    # 32 核的台就是 110（用户定：140 降到 110）。CPU 闸：一台此刻的负载（1 分钟负载与可跑进程数的大者）与服务在那台在跑的尝试的线程之和取大，加这一条的线程，不超过核数的这个倍数（cpu_problem）
CPU_OVERSUBSCRIPTION_LOOSE = 6   # 只给弄坏开关 cpu-oversubscription-three：CPU 闸照这个倍数判
PEAK_SEGMENT_CORES_COLUMN = "peak_segment_cores"   # 尝试记录里峰值段核数那一栏（finish_peak_segment_cores 记；文件头 ②「CPU 闸」）
# 尝试记录里 CPU 闸线程数那一栏：min(登记数, max(峰值段核数, 可跑线程数的最大))（finish_cpu_gate_threads 记，threads_by_record 读；没有这一栏的旧尝试不读；文件头 ②「CPU 闸」）
CPU_GATE_THREADS_COLUMN = "cpu_gate_threads"
# 线程总数远超登记数的提醒（文件头 ①「要处置的提醒」的 threads-over-registered；note_threads）：线程总数超过登记数的这么多倍、其中可跑的也超过登记数、占线程最多的那一个进程自己也超过登记数的这么多倍，连着这么多轮才推。
# 两个数是定的、没量过（推的：登记数的两倍以内算开线程时多算了几个辅助线程；连着三轮排除一次编译或链接冒出来的一阵）
THREADS_OVER_REGISTERED_FACTOR = 2
THREADS_OVER_REGISTERED_ROUNDS = 3
CPU_SETTLE_BREAK_SECONDS = 5     # 只给弄坏开关 cpu-threads-settling-only：只记起了不到这么久的尝试的线程
# CPU 闸里在跑的那一条起跑满这么久、量满一段才照实测的峰值段核数记（running_threads），峰值段每段也是这么长（note_peak_segment）；没满的照命令键的核数记录记（threads_by_record）。
# 取 120 秒（推的）：编译前解依赖、等包缓存锁一般在一分钟里（任务 13065 那一次从起到退 25 秒），留一倍余地；各段分开量、取最大那一段，前后用核不一样的任务按用得多的那一段记
CPU_MEASURED_AFTER_SECONDS = 120.0
# CPU 闸里没有跑到头的核数记录的命令（新起的这一条、在跑而没满 CPU_MEASURED_AFTER_SECONDS 的）先按这么多核记（threads_by_record）；起满 CPU_MEASURED_AFTER_SECONDS 之后照实测
# 要核数少的名额（文件头 ②「CPU 闸」；cpu_judgement）：超线的这一条要的核数不超过 CPU_SMALL_TASK_CORES 时照样放行，每台同时在跑的、靠名额起的尝试不超过 CPU_SMALL_TASK_SLOTS；
# 靠名额起的起满 CPU_MEASURED_AFTER_SECONDS、实测超过 CPU_SMALL_TASK_CORES 核的不再占名额（small_slot_released）。
# 两个数是定的、没量过（推的：负载几百的台上一两核的脚本多起几条只多占几个核，名额封顶让排满的台不被小任务越堆越多）
CPU_SMALL_TASK_CORES = 2
CPU_SMALL_TASK_SLOTS = 4
CPU_SMALL_SLOT_TEXT = "要核数少的名额"   # 名额那一半句与起跑说明里的字样（自证照它认）
STALL_CPU_CORES = 0.01           # 「CPU 几乎没涨」：这一个窗口里平均不到一个核的 1%（用户选「日志与 CPU 都不涨才停」）
RUN_HARD_LIMIT_SECONDS = 14400   # 四小时硬上限（文件头 ⑦「四小时硬上限」）：带 --full-gate 的之外累计满这么多秒服务停这一次、记结局 over-four-hours、不重排
RUN_LIMIT_SECONDS = 7200         # 跑满两小时提醒（文件头 ①「跑满两小时提醒」）：从第一次尝试起跑各次尝试在跑的时长累加到这么多秒，提交时的门禁之外记要处置的提醒 long-run-unconfirmed，服务不停
# 日志满窗口没涨而服务没停它的缘由（judge_stall_stop 记进 log_growth；quiet_log_items 只对 SPARED_CPU_UNREAD、SPARED_CPU_BUSY 报 quiet-log-busy，文件头 ①「要处置的提醒」）
SPARED_CPU_UNREAD = "cpu-unread"   # 窗口开头或此刻读不到 CPU
SPARED_CPU_BUSY = "cpu-busy"       # CPU 在涨
SPARED_IO_WAIT = "io-wait"         # 每一轮都有进程在 D 状态（文件头 ①「卡在 IO 上」）
SPARED_ACTIVITY = "activity"       # 进程内存在涨（文件头 ①「卡住」的有进展；activity_text）
STALL_AT_CAP_FRACTION = lib.STALL_AT_CAP_FRACTION   # 判卡住那一刻顶在上限上的那一档（98%；文件头 ①「顶在上限上卡住」）：定义与依据在 task_server_lib.py，
                                                    # 客户端拼打回理由的 loop_leak= 那一段拿同一档判每一次撞上限是不是顶满
AGED_WAIT_SECONDS = 7200.0       # 提交满这么久还在排的升成高优先：排到最前、先判（用户在 records/2026-10-30-编译绕过内存总上限.md 里定）；放不下时不挡后面、不留份（文件头 ②「满时限」）
PEER_SCOPE_LEFTOVERS_MARK = "第二台这一趟内存包装的 scope 停了之后还留着进程："   # confirm_peer_cleanup 写进 cleanup_pending 的开头；cleanup-pending 提醒照它换修改意见
PEER_SCOPE_LEFTOVERS_SHOWN = 10   # 那一句至多列这么多个留下的进程（multi-host-run.sh 记进清理回执的 scope_leftovers）
# 第二台清理恢复按尝试号互斥的锁表（confirm_peer_cleanup；keyed_lock 管，放下就删）：服务进程里只有一个调度器，锁表放在模块里，只备几样的假调度器上照样拿得到
PEER_CLEANUP_LOCKS, PEER_CLEANUP_LOCKS_GUARD = {}, threading.Lock()
# 读日志按尝试号互斥的锁表（log_content；keyed_lock 管，放下就删）：排队线程判卡住与收尾线程都读同一份日志、接着同一个位置读
LOG_CONTENT_LOCKS, LOG_CONTENT_LOCKS_GUARD = {}, threading.Lock()
# 整机内存线（文件头 ②「内存」的整机内存线；machine_memory_problem）：这台整机可用内存占总量的比例与 swap 空闲占总量的比例都低于它，这一轮不往这台派新任务。
# 取 earlyoom 的线（可用内存与 swap 都低于 10% 才杀进程）再加两个百分点（推的：留一截让在跑的涨回去时先撞这一条的停派，不撞 earlyoom）
MACHINE_MEMORY_LINE_FRACTION = 0.12
MACHINE_MEMORY_LINE_TEXT = "整机内存线"   # machine_memory_problem 那一句的起头（自证照它认）
WORKSPACE_WRITER_FLOOR_BYTES = 4 * 1024 ** 3   # 换文件、打补丁的工具内存上限的下限（workspace_writer_floor）
RUNNING_BOOKING_MARGIN_BREAK_BYTES = 1024 ** 3   # 只给弄坏开关 running-booked-with-margin：有 scope 的在跑尝试在此刻实测上加这么多记（running_booking）
# 在跑的记账取的是哪一个（running_booking 交、记账明细的 basis 与 control bookings 打的那一栏）：此刻实测、实测到了它的上限、报告里没有它的 scope 记整份上限
BOOKED_FROM_USAGE, BOOKED_FROM_CAP, BOOKED_WITHOUT_SCOPE, BOOKED_WHILE_YOUNG = "measured", "cap", "no-scope", "young"
RUNNING_PEAK_SAMPLE_SECONDS = 5   # note_running_sample 里同一条尝试这么久之内读到的几回并成一条：窗口里至多攒 窗口秒数 / 5 条
# slice 兜底（文件头 ②「内存」的 slice 兜底；slice_floor_problem）：那台 slice 实测还放得下的（总上限 − slice 里进程用的内存）不到这么多，这一轮不派新任务。
# 取 4 GiB（推的：一次 31 线程编译几十秒里的突增在几 GiB 量级，留出这一截让在跑的涨回峰值时先撞自己的 memory.max，不撞 slice）
SLICE_ROOM_FLOOR_BYTES = 4 * 1024 ** 3
SLICE_FLOOR_TEXT = "slice 兜底"   # slice_floor_problem 那一句的起头（自证照它认）
MEMORY_WAIT_TOP_RUNNING = 2   # 内存放不下的排队原因里点名这台记账最多的几条在跑任务（memory_wait_text）
JUDGE_HOST_MEMORY_PREFIX = "内存：要的空闲"   # host-dispatch.py 的 judge_host 内存放不下那一句的起头：服务照 memory_wait_text 换成写明两个数的那一句（memory_verdict_reworded）
# 记账对账（文件头 ②「内存」的记账对账；audit_bookings）：每隔 BOOKING_AUDIT_SECONDS 逐台比一回「这台在跑尝试的内存记账之和」与「这台实测的用量（报告里各 scope 回收不掉的用量之和）」，
# 记账之和不少于实测的 BOOKING_DRIFT_RATIO 倍、而且多出至少 BOOKING_DRIFT_BYTES，连着 BOOKING_DRIFT_ROUNDS 轮都这样，判「记账漂移」（只报不停任务）。
# 推的：一分钟一轮、三倍、8 GiB、三轮（一次编译冲高之后几十秒回落不报；一台上两三条各多记十来 GiB 的挡着排队的才报）
BOOKING_AUDIT_SECONDS = 60.0
BOOKING_DRIFT_RATIO = 3.0
BOOKING_DRIFT_BYTES = 8 * 1024 ** 3
BOOKING_DRIFT_ROUNDS = 3
BOOKING_DRIFT_PUSH_SECONDS = 600.0   # 判出之后推给被挡住的会话：头一回当场推，之后照 reminder_threshold 以这个数为门槛、每翻一倍再推一回（同一件事同一道门槛只推一次）
BOOKING_DRIFT_ROWS_SHOWN = 8         # 推给会话的正文里逐条列的尝试数上限（/health 与 control check 列全）
RUNNING_BOOKED_SUM_TEXT = "在跑的记账之和"   # 公式那一句里的字样（verdict_within_the_formula、progress_problem；自证照它认）
CPU_UNREAD_TEXT = "CPU：这台此刻读不到"   # cpu_problem 读不到核数或负载时那一句的起头（自证照它认）
WHOLE_FRACTION = 0.85            # 记账贴着整台的界（oversized-budget 提醒，budget_check）：记账不小于那台可用上限（总上限减那台此刻的余量，host_capacity）的这么多就报
WAIT_LIMIT_SECONDS = 60.0
BREAK_FIXED_MARGIN_BYTES = 2 * 1024 ** 3   # 只给弄坏开关 margin-fallback-fixed：哪台量不到余量时照改前退回的固定数（文件头 ②「余量」）
RECORD_WINDOW = 5
LOG_TAIL_BYTES = 65536
PROGRESS_PUSH_LISTED_ATTEMPTS = 6     # 在跑进度那一条里逐次列的在跑尝试至多这么多次，多的只报个数（Scheduler.push_progress）
PROGRESS_PUSH_TAIL_CHARACTERS = 200   # 在跑进度那一条里每次尝试的日志末行至多这么多个字
LOG_SCAN_BYTES = 4 * 1024 * 1024   # log_content 接着读日志时一次至多读这么多
FAST_TICK_SECONDS = 0.1          # 自证的定时（TASK_SERVER_SELFTEST_FAST=1）
FAST_UNIT_CHECK_SECONDS = 0.3
FAST_LOST_GRACE_SECONDS = 1.0
FAST_STARVED_SECONDS = 1.0
FAST_PROGRESS_STALL_SECONDS = 3.0   # 自证里看进展的时限：比 FAST_STARVED_SECONDS 长，短于各格盯 /health 的 4 秒
NFS_RECHECK_SECONDS = 60.0       # 服务起来之后隔这么久复核一次 NFS 共享的前三项（task_server_lib.py 文件头「NFS 共享」；与 PEER_RECHECK_SECONDS 重读第二台同一个间隔）
FAST_NFS_RECHECK_SECONDS = 0.5
ACTIVE_ATTEMPT_STATES = ("starting", "running")
CLEANUP_RETRY_SECONDS = 5.0   # 清理没确认（cleanup_pending）的那一次隔这么久再重查（finish_attempt）
# 读不到单元状态的提醒门槛（秒；文件头 ①「要处置的提醒」的 unit-unreadable）：对账连着读不到一次尝试的单元状态满这么久推一回，之后每翻一倍再推。
# 取 60 秒（推的）：systemctl --user show 一回至多等 30 秒（unit_states），用户管理器重载、D-Bus 忙这类短时失灵在一两回里就恢复，不为它推；连着一分钟读不到就要人去核用户级 systemd
UNIT_UNREADABLE_ATTENTION_SECONDS = 60.0
FAST_UNIT_UNREADABLE_ATTENTION_SECONDS = 2.0   # 自证里的提醒门槛（TASK_SERVER_SELFTEST_FAST=1）
UNIT_STATES_UNREADABLE_FILE = "unit-states-unreadable"   # 只在自证里：TASK_SERVER_SELFTEST_WORLD 下有这份文件时 unit_states 交 None（演 systemctl --user show 读不到）
UNIT_UNREADABLE_HEALTH_TEXT = "本机读不到在跑尝试的单元状态"   # /health 报读不到单元状态那一句的起头（unit_unreadable_report；自证照它认）
UNIT_UNREADABLE_HEALTH_SHOWN = 5   # /health 那一句至多点名这么多次尝试，余下的报次数（unit_unreadable 栏照列全）
UNIT_CGROUPS_UNREADABLE_FILE = "unit-cgroups-unreadable"   # 只在自证里：TASK_SERVER_SELFTEST_WORLD 下有这份文件时本机核进程的 cgroup 根换成世界目录里不在的一处（演 cgroup 也读不到；LocalMachine.attempt_presence_readings）
ENDED_TAMPERED_FIELDS = ("number", "host", "unit", "label", "started_at", "session", "command_key")   # 核不过而记 ended 的那一次正文只留这几样（end_tampered_attempt）
RESIDENT_RETENTION_SECONDS = 7 * 24 * 3600   # 按任务号记在内存里的那几张表（forget_settled_entries）：有了结局没人取的，结局满这么久也删
RESIDENT_PRUNE_SECONDS = 600.0               # 至多隔这么久清一次那几张表（对账每一轮问一次）
FAST_RESIDENT_PRUNE_SECONDS = 0.5
# forget_settled_entries 清的按任务号记的表：{Scheduler 上的名字: 键里任务号的位置（None 是整个键就是任务号）}
RESIDENT_TASK_TABLES = {"alerted_child_refusals": 0, "reminded_thresholds": 0, "reminder_histories": 0, "workspace_write_checks": None,
                        "advance_failures": None, "advance_episodes": None, "unclaimed_repushed": None}
PAUSE_META_NAME = "pause"        # 暂停态在库 meta 表里的名字（文件头 ⑤「暂停」）
RESTART_SURVIVING_META_PREFIX = "restart-surviving:"   # 换服务不丢的调度判断在库 meta 表里的名字前缀（Store.restart_surviving_state）
# 换服务不丢的调度判断：{库里的名字: 只记内存的弄坏开关}；三样共用一份读写（Scheduler.persist_restart_surviving_state、restore_restart_surviving_state），
# 各自的内存表与库里那一份的互换在 Scheduler.restart_surviving_record、adopt_restart_surviving_record
RESTART_SURVIVING_STATES = {
    "runaway_notified": "runaway-reports-memory-only",        # 任务之外的空转进程报过几次、停没停（watch_runaway_processes）
    "shard_advance": "advance-episodes-memory-only",          # 分片父任务推不动的这一回与开始过几回（note_advance_failure）
    "suspect_reported": "suspect-reported-memory-only",       # 疑似卡住报给会话的是哪一段（push_suspects、unreported_suspect）
    "health_notified": "health-notified-memory-only",         # 服务健康每一样发给了哪几个会话（watch_health）
    "booking_drift_notified": "booking-drift-notified-memory-only",   # 记账漂移每一件推给每个会话推到了哪道提醒门槛（push_booking_drift）
}
PAUSE_REASON_CHARACTERS = 200    # 暂停原因收的字数（一句话；多的截掉）
LISTING_ONLY_ARGUMENTS = ("--list-items", "--help", "-h")   # 带了就只列项、只打帮助，不编不跑（compiles_declared）
# 判定库读写接完 prune-server 的族：登记表第三列（判定库放哪）写「prune-server」的行（registry_families）照收；别的带判定库的提交时拒（文件头 ②「提交时拒」③）
VERDICT_LIBRARY_REGISTRY = os.path.join(".claude", "gate.d", "verdict-library-scripts.tsv")
REGISTRY_SERVICE_MARK = "prune-server"   # 第三列以它起头的行：那一族的判定库读写全在服务里
REGISTRY_NO_LIBRARY_PREFIXES = ("不算：", "无判定库：")   # 第三列以这两种起头的没有判定库（登记表文件头）
# 只派一台的名单全删了（用户：「我再重申一次 【只能在本机跑】 是不能接受的。必须尽快解决 让错误暴露错来」）：任何命令都派得到两台，哪台由余量定，
# 派到第二台跑红的由脚本那一侧修。下面这几张是删之前的样子，只给弄坏开关 host-lists-restored 照改前放回用（文件头 ②「机器」）
RESTORED_HOST_LISTS = {
    "local_directories": ("scripts/hosts", ".claude/gate.d"),   # 点到这几个目录的只在本机
    "local_entries": (".claude/gate.d/doc-registries.sh", "scripts/crash/crash-point-replay-shard-configuration-check.sh", "prune-server/control",
                      "scripts/experiment-apparatus/e180-reading-shards.sh", "scripts/experiment-apparatus/e183-sweep-history-shards.sh"),   # 读本机才有的东西的入口
    "peer_entries": ("scripts/experiment-apparatus/e185-r2-segment-run.sh",),   # 只在第二台跑的入口
}
VOLATILE_ROOT = "/tmp"   # 重启会清掉的目录：提交时拒的共享根外路径落在它下面的，拒绝说明另说一句重启会清（文件头 ②「提交时拒」①）
# 按子串认词里的绝对路径（find_outside_shared_path：shell -c 脚本词切不开时，与弄坏开关 path-check-substring 下每个带空白的词）；前面紧挨着 \w、/（$VAR/…、a/b 的后半）、
# }（${X:?}/…）、)（$(cmd)/…）的是变量展开、命令替换之后的尾巴，真路径运行时才由展开给，不是命令写死的，不算；前面是空白、引号、=、: 一类的照判
QUOTED_WORD_PATH = re.compile(r"(?<![\w/}).])/[^\s'\"`;|&()<>]+")
# shell 的 -c 脚本那一个词（shell_script_word_indexes）：命令词名（basename）是这几个之一、选项里有带 c 的短选项串（-c、-ec、-lc）的，选项之后头一个词是脚本
SHELL_COMMAND_NAMES = ("bash", "sh", "dash", "zsh", "ksh")
SHELL_OPTIONS_WITH_VALUE = ("-o", "+o", "-O", "+O", "--rcfile", "--init-file")   # 后面跟一个值的 shell 选项（shell_script_word_indexes 跳过那个值）
# 一个 shell 词里的绝对路径（shell_word_paths）：在词头，或紧跟在 =（名字=、--选项= 的值）、挨着写的 ;|& 与重定向 <>、开子 shell 的 ( 后面，到挨着写的 ;|&()<>` 为止；
# 跟在 )、}、$… 后面的（$(pwd)/…、${X:?}/… 展开之后的尾巴）与词中间夹着的不算
SHELL_WORD_PATH = re.compile(r"(?:^|(?<=[=;|&<>(]))/[^;|&()<>`]+")
SHELL_WORD_PATH_WITH_EXPANSION_TAILS = re.compile(r"(?:^|(?<=[=;|&<>()}]))/[^;|&()<>`]+")   # 弄坏开关 expansion-tail-as-path 下用的：跟在 )、} 后面的展开之后的尾巴也取
# 两台相同的系统路径（find_outside_shared_path 不判它们在不在共享根下；与 scripts/hosts/multi-host-run.sh 的 MULTI_HOST_HOST_INDEPENDENT_PATHS 同一组）：以 / 收尾的是整个目录
HOST_INDEPENDENT_PATHS = ("/dev/null", "/dev/stdin", "/dev/stdout", "/dev/stderr", "/dev/fd/", "/usr/", "/bin/")
# 多机配置 SLICE_MEMORY_CAP_BY_HOST 里一台那一项的值（写法同 scripts/hosts/run-with-memory-cap.sh 读它的那一句：正整数加 K / M / G / T）与单位
SLICE_CAP_VALUE = re.compile(r"^([1-9][0-9]*)([KMGT])$")
SIZE_UNITS = {"K": 1024, "M": 1024 ** 2, "G": 1024 ** 3, "T": 1024 ** 4}
# 改主工作区的工具（modifies_workspace：跑第二遍结果就变，撞上限这类本会重排的结局一律打回、不重排；派哪台不看它，第二台改的就是同一棵共享的树）
LONG_RUN_SPLIT_EXEMPT_FILE = ".claude/long-run-split-exempt"   # 用户点名豁免拆分的项（.claude/rules/long-run-split.md「豁免」）；long_run_split_exempt 读
WORKSPACE_BLOCKED_EXIT_CODE = 7   # scripts/gate/hook-swap.py 的退出码 7：上一趟换上没写完成，拒绝再换
HOOK_SWAP_ENTRY = "scripts/gate/hook-swap.py"   # 带 --feed-only 的只喂不换（modifies_workspace 不算它改工作区）
WRITER_PATCH_ENTRY = "scripts/gate/apply-writer-patch.py"   # 带 --dry-run 的只在临时树上核（modifies_workspace 不算它改工作区）
WORKSPACE_READ_ONLY_ARGUMENTS = ("--selftest", "--list-items", "--dry-run", "--read-outcome", "--pending-status")   # 改工作区的工具带了这几样之一只读不写（自证、列项、干跑、读结局），modifies_workspace 不算（任务 7543、7549）
WORKSPACE_WRITERS = {"scripts/gate/hook-swap.py", "scripts/gate/apply-writer-patch.py", "scripts/gate/stage-mine.py"}
# 提交时拒 ⑤（文件头 ②「提交时拒」⑤）：这几种 cargo 不点名包就编整个工作区；fmt、metadata、tree、clean 不编，不在里面
WHOLE_WORKSPACE_SUBCOMMANDS = {"build", "b", "check", "c", "clippy", "test", "t", "run", "r", "bench", "doc", "d"}
WHOLE_WORKSPACE_FLAGS = ("--workspace", "--all")   # 带了就编整个工作区，点不点名包都一样
WHOLE_WORKSPACE_NEXT_STEP = "点名要编的包：-p <包>（验证包写 -p singlefs-verification）；整仓要跑的走 scripts/gate/merge-and-verify.sh 或提交时的门禁"
MERGE_AND_VERIFY_ENTRY = "scripts/gate/merge-and-verify.sh"   # 合入后验证：只在自己拷的快照上打补丁与验，不算改主工作区（modifies_workspace）
WORKSPACE_WRITER_NO_RETRY_OUTCOMES = ("memory-cap-hit", "squeezed-out", "peer-run-failed", lib.EXTERNAL_KILL_OUTCOME)   # 改主工作区的命令遇到这几种本会重排的结局一律打回、不重排
# 改主工作区的命令以这几种结局收尾（连同被信号杀的 command-failed）：可能写了一半，wait 报 workspace-write-interrupted（文件头 ①「要处置的提醒」）
WORKSPACE_WRITER_INTERRUPTED_OUTCOMES = WORKSPACE_WRITER_NO_RETRY_OUTCOMES + ("memory-cap-at-maximum", "cancelled", "stalled", "launcher-lost", lib.RUN_HARD_LIMIT_OUTCOME)
EXTERNAL_KILL_LOG_AFTER_SECONDS = 60.0   # 读杀进程的日志：尝试结束之后再往后看这么久（被杀的进程退了、构建工具收尾、命令才退）
EXTERNAL_KILL_LOG_SECONDS = 15.0         # 每读一次日志（journalctl）至多等这么久
SIGNAL_EXIT_CODES = range(129, 129 + 64)   # 128 + 信号号：attempt_runner 记被信号杀的命令就是这个码（attempt_runner.py 文件头 ⑥），bash 起的命令被信号杀同样退它
PATCH_WRITER_OUTCOME_NAME = "apply-outcome.tsv"   # WRITER_PATCH_ENTRY 的结局文件：补丁目录里这一份，--read-outcome 判打全了没有
PATCH_WRITER_READ_ONLY_ARGUMENTS = ("--dry-run", "--pending-status", "--read-outcome", "--selftest")   # 带这几样的不写主工作区
PATCH_WRITER_CHECK_SECONDS = 120   # 起一次 --read-outcome 至多等这么久（它只读结局文件、对着主工作区算几份文件的 sha256）
PATCH_WRITER_VERDICTS = {0: "打全了", 1: "已退回", 2: "结局文件读不了", 3: "没打完", 4: "打完之后文件又被改了"}   # --read-outcome 的退出码（apply-writer-patch.py 文件头「结局文件」）
PEER_HOST_SPECIFIC_VARIABLES = ("ROCKSDB_LIB_DIR", "ROCKSDB_STATIC", "SNAPPY_LIB_DIR", "SNAPPY_STATIC")   # 按台取值（灰度会话报 359：第二台的预编库核对红）
MAXIMUM_LAUNCHER_DEPTH = 4
# 盘余量（task_server_lib.py 文件头「挑机器」的「盘余量」）：一台编译目录所在盘不到这么多就这一轮不派那一台，两台同一个下限。一趟的编译目录量过 200G 级
#（records/2026-10-31-task-server巡检.md 问题 72：任务 2665 的合入后验证在第二台留了一份 213G 的编译目录），余量取不到一次合入后验证的四分之一，
# 那一趟编译会在中途写满那台的盘（本机是 TASK_SERVER_BUILD_DIRECTORY 所在盘，第二台是多机配置 BUILD_ROOT_BY_HOST 里它那一项所在盘，multi-host-run.sh 文件头 ⑤）
DISK_FLOOR_BYTES = 50 * 1024 ** 3
DISK_REPORT_KEY = "disk_available"   # 每台余量报告里它编译目录所在盘的余量（字节；Machine.probe 带上，报告里没有时 Machine.disk_free 现量）
PEER_DISK_PROBE_TEXT = "df -B1 --output=avail -- {directory}"   # 在第二台的编译根（BUILD_ROOT_BY_HOST 里它那一项）上起：末行是余量的字节数
PEER_BUILD_REMOVE_SECONDS = 120   # multi-host-run.sh --remove-build-directory（第二台留着的编译目录）至多等这么久：核持有者、rm、回读各一次 ssh（推的，同 --recover-cleanup 的量级）
DISK_FLOOR_KIND = "disk-floor"   # note_peer 去重的原因种类：盘不够这一种只告警一次，余量的数变了不另写
# 编译目录占盘（task_server_lib.py 文件头「资源记录」的「编译目录占盘」；records/2026-10-31-task-server巡检.md 问题 76）：派到一台之前那台的盘余量要放得下
# 这条命令量过的编译目录峰值加 BUILD_DISK_FLOOR_BYTES（盘余量照上面的 DISK_REPORT_KEY）
BUILD_DISK_FLOOR_BYTES = DISK_FLOOR_BYTES
# 启动回执超时（TS-06，records/2026-11-01-task-server深入审核.md）：systemd-run 超时只算「起没起不确定」，照预先定好的单元名读回；
# 读回单元不在、起跑标记也没有，过了这么久才确认没起、记 unit-start-failed
START_UNCERTAIN_GRACE_SECONDS = 120.0
FAST_START_UNCERTAIN_GRACE_SECONDS = 2.0
SELFTEST_PAUSE_SECONDS = 60.0   # 自证的停顿点至多等这么久放行（TASK_SERVER_SELFTEST_PAUSE_POINT）
PRUNE_SERVER_CHECK_SECONDS = 60   # 命令退 75（prune-server 不可用）时起一次 prune-server/control check 的时限
SCENE_KEEP_AFTER_COLLECT_SECONDS = 3600.0   # 现场在 task wait 取走结局之后再留这么久（task_server_lib.py 文件头「现场」：提交方读日志、照日志去读现场，一小时够，推的）
FAST_SCENE_KEEP_AFTER_COLLECT_SECONDS = 1.0
STOP_JOIN_SECONDS = 10.0          # 停服务时等排队线程与 NFS 复核线程停下的时限（两个合计；单元的 TimeoutStopSec 是 30 秒，文件头 ③「停服务」）
STOP_CHILD_GRACE_SECONDS = 3.0    # 停服务时给自己起的子进程发 TERM 之后至多等这么久再 KILL


def say(text):
    print(text, file=sys.stderr, flush=True)


SUBMIT_TRACE_TEXT = "提交进出"       # 一条提交走完打的那一行的字样（SubmitTrace；自证照它认）
SUBMIT_TRACE_SLOW_SECONDS = 5.0      # 提交那一路的一步本身用了这么久，当场另打一行（不等回话）


class SubmitTrace:
    """一条提交（POST /tasks）在服务里走过的几步与各自的时刻：收到（事件循环上，排进线程之前）、进线程、各步做完、回话。
    回话时打一行（以 SUBMIT_TRACE_TEXT 起头）：收到的时刻、每一步做完时距收到多少秒；平常一条提交只这一行。
    一步本身（距上一步）用了 SUBMIT_TRACE_SLOW_SECONDS 以上的，当场另打一行：后面卡住、客户端等不到回话时，日志里也看得到前面慢在哪一步。
    弄坏开关 submit-trace-silent 下一行都不打。"""

    def __init__(self, environment):
        self.silent = lib.broken("submit-trace-silent", environment)
        self.received_wall, self.received = time.time(), time.monotonic()
        self.previous = self.received
        self.steps = []          # [(这一步叫什么, 做完时距收到多少秒)]
        self.task_id = None

    def step(self, name):
        now = time.monotonic()
        self.steps.append((name, now - self.received))
        if now - self.previous >= SUBMIT_TRACE_SLOW_SECONDS and not self.silent:
            say(f"  ! {SUBMIT_TRACE_TEXT}：{name}这一步用了 {now - self.previous:.1f} 秒（收到之后 {now - self.received:.1f} 秒），回话还没出去")
        self.previous = now

    def finish(self, outcome):
        """回话这一刻：记「回话」一步，打那一行；outcome 写这一条的结局（收下、拒了哪一种、出错）。"""
        self.step("回话")
        if self.silent:
            return
        received = time.strftime("%H:%M:%S", time.localtime(self.received_wall)) + f".{int(self.received_wall % 1 * 1000):03d}"
        steps = "、".join(f"{name} +{seconds:.3f}s" for name, seconds in self.steps)
        task = f"任务 {self.task_id}，" if self.task_id is not None else ""
        say(f"  · {SUBMIT_TRACE_TEXT}（{task}{outcome}）：{received} 收到；{steps}")


def append_private_line(path, line):
    lib.ensure_private_directory(os.path.dirname(path))
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
    with os.fdopen(descriptor, "a", encoding="utf-8") as handle:
        handle.write(line.rstrip("\n") + "\n")


def nofile_property():
    """尝试单元的打开文件数上限：软、硬都取服务自己的硬上限（用户级 systemd 给单元的软上限是 1024，崩溃判定库开不了；
    records/2026-10-30-编译绕过内存总上限.md #16）。"""
    hard = resource.getrlimit(resource.RLIMIT_NOFILE)[1]
    value = "infinity" if hard == resource.RLIM_INFINITY else str(hard)
    return f"LimitNOFILE={value}:{value}"


def directory_bytes(path):
    """一个目录底下文件的字节数之和（task clean 报大小用；读不到的跳过）。"""
    total = 0
    for root, _, names in os.walk(path):
        for name in names:
            with contextlib.suppress(OSError):
                total += os.lstat(os.path.join(root, name)).st_size
    return total


def earlier_reading(kept, latest):
    """note_running_sample 并读数用：累计量挨得近的两回读数，留先到的（时刻与值对得上）。"""
    return kept


def cpu_reading(value):
    """累计 CPU 读数（Machine.cpu_usec）核成 float：是数（不是布尔）、有限、不小于 0 的交 float，别的（None、接口交来的别的东西）交 None，照读不到办。"""
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0:
        return None
    return float(value)


def read_json_file(path):
    try:
        with open(path, encoding="utf-8") as handle:
            return json.load(handle)
    except (OSError, ValueError):
        return None


def write_text_atomically(path, text):
    temporary = f"{path}.{os.getpid()}.partial"
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(descriptor, "w", encoding="utf-8") as handle:
        handle.write(text)
    os.replace(temporary, path)


def write_json_atomically(path, value):
    write_text_atomically(path, json.dumps(value, ensure_ascii=False))


def log_tail(path):
    try:
        with open(path, "rb") as handle:
            handle.seek(0, os.SEEK_END)
            handle.seek(max(0, handle.tell() - LOG_TAIL_BYTES))
            return handle.read().decode("utf-8", "replace")
    except OSError:
        return ""


# multi-host-run.sh 的 ⑥ 删了第二台这一趟的本地目录与 runs/ 下三个文件、回读过之后打的那一行（它的 multi_host_remove_peer_run）里的两段原文：
# 取消时在跑的第二台那一次照它认第二台那一份删了（Scheduler.cancelled_attempt_artifacts）
PEER_TRIP_REMOVED_MARK = "⑥ 删了第二台这一趟的本地目录"
PEER_TRIP_READ_BACK_MARK = "回读不在了"
# multi-host-run.sh 建好第二台这一趟的本地目录 builds/<标签>/ 之后才打的两行（起第二台那条命令那一步的开头、⑤ 开跑）：日志里有其中一行就是那一趟的本地目录建过了
PEER_TRIP_BUILT_MARKS = ("MULTI_HOST_RUN_STEP step=peer-command event=begin", "⑤ 第二台开跑")


def artifact_size_text(byte_count, host):
    """取消之后删产物那几行里的大小：量到的写多大（连同字节数），第二台本地盘上量不到的照写。"""
    if isinstance(byte_count, int):
        return f"{lib.size_text(byte_count)}（{byte_count} 字节）"
    return "量不到（第二台本地盘，服务这一侧看不到）" if host == PEER else "量不到"


def cancelled_artifact_lines(item):
    """取消之后删的一份产物（Scheduler.cancelled_attempt_artifacts 交的）在回话与日志里的行：删了的一行，没删成的「✗」一行接「→ 怎么办：」一行。"""
    where = f"任务 {item['task']} 第 {item['attempt']} 次尝试在{lib.HOST_NAMES.get(item['host'], item['host'])}的{item['kind']} {item['path']}"
    sizes = f"删前 {artifact_size_text(item['bytes_before'], item['host'])}"
    if item["gone"]:
        return [f"  · 取消之后删了{where}：{sizes}、删后 不在（{item['how']}）"]
    return [f"  ✗ 取消之后没删掉{where}：{sizes}、删后 {artifact_size_text(item['bytes_after'], item['host'])}：{item['why']}",
            f"  → 怎么办：{item['howto']}"]


def append_log_lines(path, lines):
    """往一次尝试的日志末尾接几行（取消之后删产物、提交时警告的日志头）；日志路径空着或写不进不报错（回话里照样带这几行）。"""
    if not path:
        return
    with contextlib.suppress(OSError):
        with open(path, "a", encoding="utf-8") as handle:
            handle.write("".join(line + "\n" for line in lines))


def post_to_inbox(socket_path, token, text, environment):
    """往会话的收件 socket 发一条消息（文件头 ⑥）：先发认证行 {"type":"auth","token":…}，再发一行 {"type":"user","message":{"role":"user","content":<正文>}}。
    两行先备好再连、一次写完，写完关掉写的那一半，等对面收完关连接至多 INBOX_DRAIN_SECONDS。→ 发不出去的原因（socket 不在、连不上、写不进），发出去了交空串。
    行的写法取自 Claude Code 二进制自己的调试日志（task-server/README.md「## 发消息给会话」），文档页没写，升级 Claude Code 之后要复核。
    弄坏开关 inbox-auth-omitted 下不发认证行。"""
    lines = [] if lib.broken("inbox-auth-omitted", environment) else [json.dumps({"type": "auth", "token": token}, ensure_ascii=False)]
    lines.append(json.dumps({"type": "user", "message": {"role": "user", "content": text}}, ensure_ascii=False))
    payload = "".join(line + "\n" for line in lines).encode("utf-8")
    connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    try:
        connection.settimeout(INBOX_SEND_SECONDS)
        connection.connect(socket_path)
        connection.sendall(payload)
        connection.shutdown(socket.SHUT_WR)
        connection.settimeout(INBOX_DRAIN_SECONDS)
        with contextlib.suppress(OSError):
            while connection.recv(4096):
                pass
    except OSError as error:
        return f"{type(error).__name__}: {error}"
    finally:
        connection.close()
    return ""


def own_unit_state(environment):
    """服务自己那个用户级单元 is-active 打的状态（照输出判，同 control 的 unit_active_state；自证里 TASK_SERVER_CONTROL_SYSTEMCTL 换成假的）。"""
    systemctl = lib.selftest_override("TASK_SERVER_CONTROL_SYSTEMCTL", environment) or "systemctl"
    try:
        completed = subprocess.run([systemctl, "--user", "is-active", lib.UNIT_NAME], capture_output=True, text=True, env=environment, stdin=subprocess.DEVNULL, timeout=30)
    except (OSError, subprocess.TimeoutExpired) as error:
        return f"读不到（{type(error).__name__}: {error}）"
    lines = completed.stdout.strip().splitlines()
    return lines[-1].strip() if lines else "（没打状态）"


INBOX_HEADLINE = "task-server："   # 每条消息正文第一行的开头（文件头 ⑥）


def push_round_tail(round_number, previous_at):
    """首行括号里最后一段的「；第 N 回」；重推的（N 大于 1）另写「，上一回在 <日期>」（previous_at 是上一回推出去的时刻，读不到就不写日期）。
    弄坏开关 inbox-headline-round-omitted 下交空串。"""
    if lib.broken("inbox-headline-round-omitted"):
        return ""
    tail = f"；第 {round_number} 回"
    if round_number > 1 and previous_at:
        tail += f"，上一回在 {time.strftime('%Y-%m-%d', time.localtime(float(previous_at)))}"
    return tail


def task_state_phrase(task):
    """推给会话的消息首行里任务此刻的状态：已结束、还在排（没进队的也算）、还在跑；任务记录里没有状态的当还在跑。"""
    state = (task or {}).get("state")
    if state == "finished":
        return "任务已结束"
    if state in ("queued", lib.AWAITING_CONFIRMATION_STATE):
        return "任务还在排"
    return "任务还在跑"


def kind_label(kind):
    """首行里一个种类的中文名（括号里的英文种类名由调用方接）；没登记的退回英文种类名（自证 headline-names-cover-kinds 核登记的种类一个不缺）。
    弄坏开关 inbox-headline-name-omitted 下只写英文种类名。"""
    return kind if lib.broken("inbox-headline-name-omitted") else lib.KIND_NAMES.get(kind, kind)


def headline_class(category, state):
    """首行开头的类别与任务此刻的状态：「结局（任务已结束）」「提醒（任务还在跑，不是结局）」「疑似卡住（任务还在跑、服务没停它）」；弄坏开关 inbox-headline-state-omitted 下不写这一段。"""
    return "" if lib.broken("inbox-headline-state-omitted") else f"{category}（{state}）"


def outcome_headline(task_id, outcome, exit_code, round_number=1, previous_at=None, unclaimed=False):
    """结局那一条的首行：先写类别「结局」与任务此刻的状态（已结束），再写任务号、中文名（英文种类名，退出码；第 N 回）；
    结局没人取而再推的（unclaimed）在状态里另写这条结局还没人取。"""
    state = "任务已结束，这条结局还没人取" if unclaimed else "任务已结束"
    return f"{INBOX_HEADLINE}{headline_class('结局', state)}任务 {task_id} {kind_label(outcome)}（{outcome}，退 {exit_code}{push_round_tail(round_number, previous_at)}）"


def reminder_headline(task, item, round_number, previous_at, tail="", subject=None):
    """要处置的提醒那一条的首行：先写类别「提醒」与任务此刻的状态（还在排 / 还在跑，不是结局；任务其实已结束的写结局另有一条），再写任务号、中文名（英文种类名<tail>；第 N 回）。
    tail 是种类后面接的（没处置、第几回再推，子任务号，片数）。状态照提醒说的那一条记录（subject，说的是子任务的是子任务那一行）：分片的父任务一直是 sharded，
    片都在排的提醒写还在排；subject 不给的照 task。"""
    state = task_state_phrase(subject or task)
    note = "，结局另有一条" if state == "任务已结束" else "，不是结局"
    return (f"{INBOX_HEADLINE}{headline_class('提醒', state + note)}任务 {task['id']} {kind_label(item['kind'])}"
            f"（{item['kind']}{tail}{push_round_tail(round_number, previous_at)}）")


def suspect_headline(task, segment, part=""):
    """疑似卡住那一条的首行：类别「疑似卡住」与任务此刻的状态（还在跑、服务没停它），再写任务号、中文名（英文种类名，第 N 段；第 1 回：一段只推一回）。"""
    return (f"{INBOX_HEADLINE}{headline_class('疑似卡住', '任务还在跑、服务没停它')}任务 {task['id']} {kind_label('suspected-stall')}"
            f"（suspected-stall，第 {segment} 段{part}{push_round_tail(1, None)}）")


HEALTH_NEXT_STEP = (f"python3 task-server/control check 看卡在哪；起不来看 journalctl --user -u {lib.UNIT_NAME} -n 100；NFS 共享与第二台挂载照 {lib.NFS_README} 修，"
                    "第二台用不了照上面那一句修（连法、双机开关）；在跑的任务在各自的单元里接着跑，修好之后服务对回它们")   # 服务健康那一条的出路


# ── 库 ──
@contextlib.contextmanager
def keyed_lock(table, guard, key, keep=False):
    """拿着锁表 table（{键: [锁, 拿着与在等的个数]}，改它要拿 guard）里 key 那一把跑 with 里那一段（Store.task_lock 与 confirm_peer_cleanup 用）：
    拿着与在等的都算一个；最后一个放下时把它从表里删掉（在等的那几个手里是同一把，删之前没有人能另建一把顶替它），表的大小不随用过的键数涨。keep 为真时放下也不删。"""
    with guard:
        entry = table.setdefault(key, [threading.Lock(), 0])
        entry[1] += 1
    try:
        with entry[0]:
            yield
    finally:
        with guard:
            entry[1] -= 1
            if entry[1] == 0 and not keep:
                del table[key]


def association_id(value):
    """正文里的关联号（task_id、parent_id）：是正整数的交它，别的（0、空、不是整数的）交 0。校验不过的行判不出归哪一件时照 0 写，不挂在任何一件名下。"""
    return value if isinstance(value, int) and not isinstance(value, bool) and value > 0 else 0


class StoreRefused(Exception):
    """Store.change / claim_attempt 判出这一次不能写（行校验不过、状态不对、取消了）：str 是为什么，kind 是哪一种。"""

    def __init__(self, kind, why):
        super().__init__(why)
        self.kind = kind


class Store:
    """库（task_server_lib.py 文件头「库」）：每个线程一条连接，写都在 write_lock 里、BEGIN IMMEDIATE（write_transaction：出事先回滚再抛，连接不留在写事务里）。
    任务的状态转换（task_server_lib.py 文件头「库」的「状态转换」）：change 在一个事务里读当前那一行、核校验、判 guard、只并进给的几栏再写，
    不拿调用方手里的旧字典整行覆盖；结了局的任务不回到别的状态。claim_attempt 把「判 queued 且没取消、分尝试号、插尝试、改任务」放进同一个事务。
    task_lock 是每条任务一把锁：起单元与取消都拿它，取消要么在起之前（没起），要么等起完再停、回读。
    锁表 task_locks 只留此刻有人拿着或在等的那几把（{任务号: [锁, 拿着与在等的个数]}）：个数回到 0 就删，任务有了结局、没人再拿它的锁之后表里没有它；
    表的大小不随跑过的任务数涨。弄坏开关 task-locks-kept 下照改前一把都不删。"""
    SIGNED_TABLES = ("tasks", "attempts", "history")
    LOG_TABLES = ("operations", "alerts")
    # (task_id, number) 与 (parent_id, role, shard_index) 两条业务唯一（TS-02、TS-03）：照正文里的那几栏建索引；task_id 是 0 的尝试（自证记进库的那几次）不归任务、不判
    UNIQUE_INDEXES = (
        "CREATE UNIQUE INDEX IF NOT EXISTS attempts_task_number ON attempts(task_id, json_extract(record, '$.number')) WHERE task_id > 0",
        "CREATE UNIQUE INDEX IF NOT EXISTS tasks_child_identity ON tasks(json_extract(record, '$.parent_id'), json_extract(record, '$.role'), "
        "json_extract(record, '$.shard_index')) WHERE json_extract(record, '$.parent_id') > 0",
    )
    # 查得勤的几种照列建索引（task_server_lib.py 文件头「库」的索引）：按关联列 task_id 取一条任务的尝试与一件的子任务（attempts_of、children_of），
    # 按命令键取结束了的尝试（recorded_measurements、上次运行时长、占盘预测），按会话取任务；没有它们每一次都是整表扫。弄坏开关 lookup-indexes-skipped 下不建（建过的照留）
    LOOKUP_INDEXES = (
        "CREATE INDEX IF NOT EXISTS attempts_task ON attempts(task_id)",
        "CREATE INDEX IF NOT EXISTS tasks_task ON tasks(task_id)",
        "CREATE INDEX IF NOT EXISTS attempts_key_state ON attempts(command_key, state, id)",
        "CREATE INDEX IF NOT EXISTS tasks_session ON tasks(session, state)",
    )

    def __init__(self, path, key, environment):
        self.path = path
        self.key = key
        self.environment = environment
        self.local = threading.local()
        self.write_lock = threading.Lock()
        self.task_locks = {}
        self.task_locks_guard = threading.Lock()
        self.write_generation = 0   # 本进程写过几行（write_row、delete_row、delete_attempt 各涨一）：running_attempts 的那一份照它与 PRAGMA data_version 作废
        self.tag = ""
        with self.write_lock:
            self.initialize()

    def connection(self):
        connection = getattr(self.local, "connection", None)
        if connection is None:
            connection = sqlite3.connect(self.path, timeout=30, isolation_level=None, check_same_thread=True)
            connection.execute("PRAGMA journal_mode=WAL")
            connection.execute("PRAGMA busy_timeout=30000")
            self.local.connection = connection
        return connection

    def initialize(self):
        connection = self.connection()
        connection.execute("BEGIN IMMEDIATE")
        connection.execute("CREATE TABLE IF NOT EXISTS meta(name TEXT PRIMARY KEY, value TEXT NOT NULL)")
        for table in self.SIGNED_TABLES:
            connection.execute(f"CREATE TABLE IF NOT EXISTS {table}(id INTEGER PRIMARY KEY AUTOINCREMENT, task_id INTEGER NOT NULL DEFAULT 0, state TEXT NOT NULL, "
                               "session TEXT NOT NULL, command_key TEXT NOT NULL, record TEXT NOT NULL, signature TEXT NOT NULL)")
            connection.execute(f"CREATE INDEX IF NOT EXISTS {table}_state ON {table}(state)")
        for table in self.LOG_TABLES:
            connection.execute(f"CREATE TABLE IF NOT EXISTS {table}(id INTEGER PRIMARY KEY AUTOINCREMENT, record TEXT NOT NULL, signature TEXT NOT NULL)")
        for statement in self.UNIQUE_INDEXES:
            try:
                connection.execute(statement)
            except sqlite3.IntegrityError as error:
                connection.execute("ROLLBACK")
                raise lib.Unavailable(f"库 {self.path} 里已有违反业务唯一的行（同一任务同一次号的两次尝试，或同一父任务同一步同一片的两个子任务）：{error}",
                                      "停着服务时 python3 task-server/control clear-queue（先备份库）清掉任务与尝试，再起；这一版不认这种库") from None
        if not lib.broken("lookup-indexes-skipped", self.environment):
            for statement in self.LOOKUP_INDEXES:
                connection.execute(statement)
        row = connection.execute("SELECT value FROM meta WHERE name = 'tag'").fetchone()
        if row is None:
            self.tag = secrets.token_hex(4)
            connection.execute("INSERT INTO meta(name, value) VALUES('tag', ?)", (self.tag,))
        else:
            self.tag = row[0]
        connection.execute("INSERT OR REPLACE INTO meta(name, value) VALUES('health', '0')")
        connection.execute("COMMIT")
        for suffix in ("", "-wal", "-shm"):
            with contextlib.suppress(FileNotFoundError):
                os.chmod(self.path + suffix, 0o600)

    def signature(self, table, row_id, state, session, command_key, record_text):
        return lib.row_signature(self.key, table, row_id, state, session, command_key, record_text)

    def verified(self, table, row):
        """row 是 (id, state, session, command_key, record, signature, task_id)：校验列对得上、正文与几列对得上才交 True。
        决定关联的那一列 task_id（attempts_of、children_of 照它挑）不在签名里，读的时候与正文的 task_id 比，对不上就是改过（TS-05）；
        子任务的 parent_id 另要与 task_id 相同。弄坏开关 association-column-unchecked 下不比（改前的样子）。"""
        if lib.broken("signature-unchecked", self.environment):
            return True
        row_id, state, session, command_key, record_text, given, column_task_id = row
        if not hmac.compare_digest(given or "", self.signature(table, row_id, state, session, command_key, record_text)):
            return False
        try:
            record = json.loads(record_text)
        except ValueError:
            return False
        if not (isinstance(record, dict) and record.get("id") == row_id and record.get("state", "") == state and record.get("session", "") == session
                and record.get("command_key", "") == command_key):
            return False
        if table in self.SIGNED_TABLES and not lib.broken("association-column-unchecked", self.environment):
            if (record.get("task_id") or 0) != (column_task_id or 0):
                return False
            if table == "tasks" and record.get("parent_id") and record.get("parent_id") != record.get("task_id"):
                return False
        return True

    def decoded(self, table, row):
        try:
            record = json.loads(row[4])
        except ValueError:
            record = {}
        if not isinstance(record, dict):
            record = {}
        record["id"] = row[0]   # 校验不过的行正文里的 id 不可信：一律取行号，照它写回、收尾都落在这一行上
        return record, self.verified(table, row)

    @contextlib.contextmanager
    def write_transaction(self):
        """一个写事务（调用方拿着 write_lock）：BEGIN IMMEDIATE，with 里那一段做完 COMMIT；那一段或 COMMIT 抛了任何异常（SQLITE_BUSY、盘满、IO 错、StoreRefused、
        KeyboardInterrupt 都算）先 ROLLBACK（SQLite 已自己回滚了的再回滚报错，不盖住原来那个异常）再照原样抛。连接不留在没结束的写事务里：
        留着的话这一条线程下回 BEGIN 报「事务里不能再开事务」，别的连接等满 busy_timeout 报 locked，整个服务写不进库。
        弄坏开关 store-transaction-left-open 下出事不回滚。"""
        connection = self.connection()
        connection.execute("BEGIN IMMEDIATE")
        try:
            yield connection
            connection.execute("COMMIT")
        except BaseException:
            if not lib.broken("store-transaction-left-open", self.environment):
                with contextlib.suppress(sqlite3.Error):
                    connection.execute("ROLLBACK")
            raise

    def insert(self, table, record):
        with self.write_lock, self.write_transaction() as connection:
            if table in self.SIGNED_TABLES:
                cursor = connection.execute(f"INSERT INTO {table}(task_id, state, session, command_key, record, signature) VALUES(?, '', '', '', '{{}}', '')",
                                            (record.get("task_id", 0),))
            else:
                cursor = connection.execute(f"INSERT INTO {table}(record, signature) VALUES('{{}}', '')")
            record["id"] = cursor.lastrowid
            self.write_row(connection, table, record)
        return record

    def update(self, table, record, force=False):
        """整行写（尝试、钉值这类行）。任务行改用 change，尝试行只改一两栏的（记编译目录已删这类）同样用 change；这里另守两条：库里已结了局的任务不让写回别的状态，
        库里已是 ended 的尝试不让写回别的状态（HTTP 线程手里先读的那份 running 的字典，在对账记了 ended、结局与峰值之后整行写回，会把它们抹掉、尝试永远算在跑），
        抛 StoreRefused("finished") 与 StoreRefused("ended")；弄坏开关 attempt-row-overwritten 下尝试那一条不守
        （force 只给弄坏开关 cancel-race-unguarded 照改前整行覆盖的那一支）。"""
        with self.write_lock, self.write_transaction() as connection:
            if table == "tasks" and record.get("state") != "finished" and not force:
                current = connection.execute("SELECT state FROM tasks WHERE id = ?", (record["id"],)).fetchone()
                if current is not None and current[0] == "finished":
                    raise StoreRefused("finished", f"任务 {record['id']} 已结了局，不写回 {record.get('state')}")
            if (table == "attempts" and record.get("state") != "ended" and not force
                    and not lib.broken("attempt-row-overwritten", self.environment)):
                current = connection.execute("SELECT state FROM attempts WHERE id = ?", (record["id"],)).fetchone()
                if current is not None and current[0] == "ended":
                    raise StoreRefused("ended", f"尝试 {record['id']} 已 ended，不写回 {record.get('state')}")
            self.write_row(connection, table, record)

    def read_row(self, connection, table, row_id):
        """事务里读一行：→ (正文, 校验过没有)；不在交 (None, False)。"""
        row = connection.execute(f"SELECT id, state, session, command_key, record, signature, task_id FROM {table} WHERE id = ?", (row_id,)).fetchone()
        return self.decoded(table, row) if row is not None else (None, False)

    def change(self, table, row_id, changes, guard=None, reopen=False):
        """在一个事务里读当前那一行、核校验、判 guard（交空串放行，交一句就不写、抛 StoreRefused("guard", 那一句)）、只并进 changes 那几栏、写回；→ 写好的正文。
        行不在抛 StoreRefused("missing")，校验不过抛 StoreRefused("tampered")；结了局的任务要写回别的状态（reopen 没给）抛 StoreRefused("finished")。
        changes 可以是函数（拿当前正文、交要并进的几栏）。"""
        with self.write_lock, self.write_transaction() as connection:
            record, valid = self.read_row(connection, table, row_id)
            if record is None:
                raise StoreRefused("missing", f"{table} 里没有 {row_id}")
            if not valid:
                raise StoreRefused("tampered", f"{table} 的 {row_id} 校验不过")
            why = guard(record) if guard is not None else ""
            if why:
                raise StoreRefused("guard", why)
            fields = changes(record) if callable(changes) else changes
            was_finished = record.get("state") == "finished"
            record.update(fields)
            if table == "tasks" and was_finished and record.get("state") != "finished" and not reopen:
                raise StoreRefused("finished", f"任务 {row_id} 已结了局，不回到 {record.get('state')}")
            self.write_row(connection, table, record)
        return record

    def claim_attempt(self, task_id, number, attempt, task_changes, between=None, split=False):
        """起一次尝试的那一个事务（TS-01、TS-03）：读任务、判 queued 且没取消、号是 attempt_count + 1、父任务（有的话）还没结局，插尝试、改任务，一起提交。
        父任务记了 cancel_requested（整件正在取消）的子任务同样不起。
        判不过抛 StoreRefused（kind 是 missing、tampered、cancelled、state、number、parent-finished、parent-cancelling）；between 在插了尝试、改任务之前调（自证的崩溃点）。
        split（弄坏开关 attempt-claim-split）照改前分两次提交：先插尝试、提交，再改任务。→ (尝试正文, 改过的任务正文)。"""
        with self.write_lock:
            connection = self.connection()
            connection.execute("BEGIN IMMEDIATE")
            try:
                task, valid = self.read_row(connection, "tasks", task_id)
                if task is None:
                    raise StoreRefused("missing", f"没有任务 {task_id}")
                if not valid:
                    raise StoreRefused("tampered", f"任务 {task_id} 的记录校验不过")
                if task.get("cancel_requested"):
                    raise StoreRefused("cancelled", f"任务 {task_id} 判放得下之后、起之前被取消了")
                if task.get("state") != "queued":
                    raise StoreRefused("state", f"任务 {task_id} 此刻是 {task.get('state')}，不是 queued")
                if int(task.get("attempt_count") or 0) + 1 != number:
                    raise StoreRefused("number", f"任务 {task_id} 已起过 {task.get('attempt_count')} 次，这一次的号 {number} 对不上")
                parent_id = task.get("parent_id")
                if parent_id:
                    parent, parent_valid = self.read_row(connection, "tasks", parent_id)
                    if parent is None or not parent_valid or parent.get("state") == "finished":
                        raise StoreRefused("parent-finished", f"子任务 {task_id} 的父任务 {parent_id} 已结了局（或记录校验不过），不起")
                    # 整件正在取消（cancel_sharded 先给父任务记 cancel_requested，再逐个停、撤子任务）：这时还排着的子任务不起，
                    # 不在停兄弟片腾出内存的那一会儿被排队线程起出来（admission 审核 F10）。弄坏开关 parent-cancel-unchecked-at-claim 下不判
                    if parent.get("cancel_requested") and not lib.broken("parent-cancel-unchecked-at-claim", self.environment):
                        raise StoreRefused("parent-cancelling", f"子任务 {task_id} 的父任务 {parent_id} 整件正在取消，不起")
                cursor = connection.execute("INSERT INTO attempts(task_id, state, session, command_key, record, signature) VALUES(?, '', '', '', '{}', '')", (task_id,))
                attempt = dict(attempt, id=cursor.lastrowid, task_id=task_id, number=number)
                self.write_row(connection, "attempts", attempt)
                if split:
                    connection.execute("COMMIT")
                    connection.execute("BEGIN IMMEDIATE")
                if between is not None:
                    between()
                task.update(task_changes, attempt_count=number)
                self.write_row(connection, "tasks", task)
                connection.execute("COMMIT")
            except BaseException:
                with contextlib.suppress(sqlite3.Error):
                    connection.execute("ROLLBACK")
                raise
        return attempt, task

    @contextlib.contextmanager
    def task_lock(self, task_id):
        """拿着这条任务的那把锁跑 with 里那一段（起单元、取消、恢复时续起都拿它）：锁表 task_locks 照 keyed_lock 管，最后一个放下时删掉那一项。
        弄坏开关 task-locks-kept 下放下也不删。"""
        with keyed_lock(self.task_locks, self.task_locks_guard, task_id, keep=lib.broken("task-locks-kept", self.environment)):
            yield

    def write_row(self, connection, table, record):
        """写一行、重签。签名表连关联列 task_id 一起照正文写：重签的那一行关联列与正文对得上（verified 的 TS-05 那一判），核得过。
        弄坏开关 tampered-rows-left-unrepaired 下不写关联列，关联列被改过的行重签之后照样核不过。
        每写一行 write_generation 涨一（running_attempts 的那一份照它作废）。"""
        self.write_generation += 1
        text = lib.canonical_json(record)
        if table in self.SIGNED_TABLES:
            state, session, command_key = record.get("state", ""), record.get("session", ""), record.get("command_key", "")
            signature = self.signature(table, record["id"], state, session, command_key, text)
            if lib.broken("tampered-rows-left-unrepaired", self.environment):
                connection.execute(f"UPDATE {table} SET state = ?, session = ?, command_key = ?, record = ?, signature = ? WHERE id = ?",
                                   (state, session, command_key, text, signature, record["id"]))
            else:
                connection.execute(f"UPDATE {table} SET task_id = ?, state = ?, session = ?, command_key = ?, record = ?, signature = ? WHERE id = ?",
                                   (record.get("task_id") or 0, state, session, command_key, text, signature, record["id"]))
        else:
            connection.execute(f"UPDATE {table} SET record = ?, signature = ? WHERE id = ?",
                               (text, self.signature(table, record["id"], "", "", "", text), record["id"]))

    def delete_row(self, table, row_id):
        with self.write_lock:
            self.write_generation += 1
            with self.write_transaction() as connection:
                connection.execute(f"DELETE FROM {table} WHERE id = ?", (row_id,))

    def count(self, table, state):
        """这张表里这个 state 的行数（/health 用：只数、不解、不核校验列）。"""
        return self.connection().execute(f"SELECT COUNT(*) FROM {table} WHERE state = ?", (state,)).fetchone()[0]

    def count_outside_session(self, table, state, session):
        """这张表里这个 state、会话列不是 session 的行数（/health 的「别的会话在跑 M 个」：没有会话的人手任务也是别的会话）。"""
        return self.connection().execute(f"SELECT COUNT(*) FROM {table} WHERE state = ? AND session != ?", (state, session)).fetchone()[0]

    def delete_attempt(self, attempt_id):
        with self.write_lock:
            self.write_generation += 1
            with self.write_transaction() as connection:
                connection.execute("DELETE FROM attempts WHERE id = ?", (attempt_id,))

    def select(self, table, where="1", parameters=(), order="id"):
        rows = self.connection().execute(f"SELECT id, state, session, command_key, record, signature, task_id FROM {table} WHERE {where} ORDER BY {order}",
                                         parameters).fetchall()
        return [self.decoded(table, row) for row in rows]

    def task(self, task_id):
        found = self.select("tasks", "id = ?", (task_id,))
        return found[0] if found else None

    def attempts_of(self, task_id):
        return self.select("attempts", "task_id = ?", (task_id,))

    def running_attempts(self):
        """此刻在起与在跑（state 是 starting 或 running）而校验过的尝试的正文：内存与盘的记账、CPU 闸的线程、被挤掉的那一句都只取这一份
        （task_server_lib.py 文件头「库」：核不过的行不认）。核不过的不在里面，由对账记 ended（Scheduler.end_tampered_attempt）。
        弄坏开关 running-attempts-unverified 下连校验不过的一起交。
        同一轮里各处要读几百遍（task_server_lib.py 文件头「库」的在跑尝试读一遍）：每个线程照自己那条连接记一份，库没变过（本进程没写过——write_generation 没涨——
        别的连接也没提交过——PRAGMA data_version 没变）就交那一份的拷贝（每条一份浅拷贝：调用方改了不写回的不染到下一次），变过才现读。
        弄坏开关 running-attempts-reread 下每次现读。"""
        unverified = lib.broken("running-attempts-unverified", self.environment)
        if lib.broken("running-attempts-reread", self.environment):
            return [record for record, valid in self.select("attempts", "state IN ('starting', 'running')") if valid or unverified]
        stamp = (self.connection().execute("PRAGMA data_version").fetchone()[0], self.write_generation)
        cached = getattr(self.local, "running_cache", None)
        if cached is None or cached[0] != stamp:
            cached = (stamp, [record for record, valid in self.select("attempts", "state IN ('starting', 'running')") if valid or unverified])
            self.local.running_cache = cached
        return [dict(record) for record in cached[1]]

    def health_write(self):
        """库写不写得进：试写一行 meta（自证的 TASK_SERVER_SELFTEST_FAIL_HEALTH_WRITE 指的文件在时当失败）。交 (写进了没有, 原因)。"""
        failing = lib.selftest_override("TASK_SERVER_SELFTEST_FAIL_HEALTH_WRITE", self.environment)
        if failing and os.path.exists(failing):
            return False, "自证造的写不进"
        try:
            with self.write_lock, self.write_transaction() as connection:
                connection.execute("UPDATE meta SET value = ? WHERE name = 'health'", (str(time.time()),))
        except sqlite3.Error as error:
            return False, f"{type(error).__name__}: {error}"
        return True, ""

    # ── 暂停态（文件头 ⑤「暂停」）：meta 表里名为 pause 的一行，正文是 {reason, since, allow_session, signature}；没有这一行就是没暂停 ──
    def pause_signature(self, reason, since, allow_session):
        """签名盖住原因、起点和放行的会话（弄坏开关 pause-allow-unsigned 下放行的会话不进签名，改它改不出校验不过）。"""
        body = {"reason": reason, "since": since}
        if not lib.broken("pause-allow-unsigned", self.environment):
            body["allow_session"] = allow_session
        return self.signature("meta", 0, PAUSE_META_NAME, "", PAUSE_META_NAME, lib.canonical_json(body))

    def pause_record(self):
        """库里记的暂停态：没暂停交 None，暂停着交 {reason, since, allow_session, valid}；签名对不上（绕过服务改过）或读不出的按暂停算、不放行谁，valid 为 False。"""
        row = self.connection().execute("SELECT value FROM meta WHERE name = ?", (PAUSE_META_NAME,)).fetchone()
        if row is None:
            return None
        try:
            stored = json.loads(row[0])
            reason, since, allow_session, given = stored["reason"], stored["since"], stored["allow_session"], stored["signature"]
            valid = (isinstance(reason, str) and isinstance(since, (int, float)) and isinstance(allow_session, str) and isinstance(given, str)
                     and hmac.compare_digest(given, self.pause_signature(reason, since, allow_session)))
        except (ValueError, KeyError, TypeError):
            valid = False
        if not valid:
            return {"reason": "暂停记录校验不过（库被绕过服务改过），按暂停算", "since": 0.0, "allow_session": "", "valid": False}
        return {"reason": reason, "since": since, "allow_session": allow_session, "valid": True}

    def write_pause(self, reason, since, allow_session):
        """记暂停态（放行的会话连同原因、起点一起签名）；换放行的会话也是整行重写。弄坏开关 pause-allow-memory-only 下放行的会话不写进库。"""
        if lib.broken("pause-allow-memory-only", self.environment):
            allow_session = ""
        value = lib.canonical_json({"reason": reason, "since": since, "allow_session": allow_session, "signature": self.pause_signature(reason, since, allow_session)})
        with self.write_lock, self.write_transaction() as connection:
            connection.execute("INSERT OR REPLACE INTO meta(name, value) VALUES(?, ?)", (PAUSE_META_NAME, value))

    def clear_pause(self):
        with self.write_lock, self.write_transaction() as connection:
            connection.execute("DELETE FROM meta WHERE name = ?", (PAUSE_META_NAME,))

    # ── 换服务不丢的调度判断（RESTART_SURVIVING_STATES）：meta 表里名为 restart-surviving:<名> 的一行，正文 {state, signature}；没有这一行就是没记过 ──
    def restart_surviving_signature(self, name, state):
        meta_name = RESTART_SURVIVING_META_PREFIX + name
        return self.signature("meta", 0, meta_name, "", meta_name, lib.canonical_json(state))

    def restart_surviving_state(self, name):
        """库里记的这一份 → (正文, 认不认)：没记过交 (None, True)；签名对得上交 (正文, True)；签名对不上（绕过服务改过）或读不出交 (None, False)。
        弄坏开关 restart-surviving-signature-unchecked 下签名对不上的也认。"""
        row = self.connection().execute("SELECT value FROM meta WHERE name = ?", (RESTART_SURVIVING_META_PREFIX + name,)).fetchone()
        if row is None:
            return None, True
        try:
            stored = json.loads(row[0])
            state, given = stored["state"], stored["signature"]
            valid = ((isinstance(given, str) and hmac.compare_digest(given, self.restart_surviving_signature(name, state)))
                     or lib.broken("restart-surviving-signature-unchecked", self.environment))
        except (ValueError, KeyError, TypeError):
            return None, False
        return (state, True) if valid else (None, False)

    def write_restart_surviving_state(self, name, state):
        """整行重写这一份（state 是能写成 JSON 的值，连同签名）。"""
        value = lib.canonical_json({"state": state, "signature": self.restart_surviving_signature(name, state)})
        with self.write_lock, self.write_transaction() as connection:
            connection.execute("INSERT OR REPLACE INTO meta(name, value) VALUES(?, ?)", (RESTART_SURVIVING_META_PREFIX + name, value))


# ── 两台的余量 ──
LOCAL, PEER = "local", "peer"   # 服务记尝试的视角（task_server_lib.HOST_NAMES）：本机是服务所在这一台，第二台是多机配置 HOSTS 里另一台
MACHINE_ORDER = (LOCAL, PEER)   # 挑台时放下之后剩余占比一样大取排在前面的；分片的各片倒过来（candidate_hosts）


@dataclass
class Snapshot:
    reports: dict                                 # {台: 余量报告}：这一轮量得到、能派的台
    memory_caps: dict                             # {台: 那台总上限另收到的数（字节）}：Machine.configured_cap 交得出的台才有
    reachable: dict                               # {台: 此刻连不连得上}（服务所在这一台恒为真；另一台照多机配置判法与 NFS 复核）
    down: dict                                    # {台: 这一轮没给报告的原因}
    hold: bool = False                            # 只在自证里：假余量里写了 hold，一条都不起
    expected: dict = field(default_factory=dict)  # {台: 用不了要不要告警}（note_peer）：只有自证里没给那一台的格是 False
    down_kind: dict = field(default_factory=dict)  # {台: down 那一句是哪一种}（note_peer 按它去重）：盘余量不到下限是 DISK_FLOOR_KIND，别的没有（按原因的字面去重）

    def total(self, host):
        """这台的总上限（字节）：那台 slice 的总上限，另有收到的数（memory_caps）取小者。减那台此刻的余量在 Scheduler.host_capacity。"""
        total = int(self.reports[host]["memory"].get("slice_total") or 0)
        cap = self.memory_caps.get(host)
        return min(total, cap) if cap else total

    def maxima(self):
        """每台的总上限（total）：连得上而这一轮没报告的是 None（查不到），连不上的不在里面。减那台此刻的余量在 Scheduler.usable_maxima（retry_decision 照它比）。"""
        result = {}
        for host in MACHINE_ORDER:
            if host in self.reports:
                result[host] = self.total(host)
            elif self.reachable.get(host):
                result[host] = None
        return result


def peer_build_root(declarations):
    """第二台的编译根：多机配置 BUILD_ROOT_BY_HOST 里第二台那一项（host-dispatch.py 的 Peer 读出的 declare -p 那几行里的 PEER_HOST_BUILD_ROOT，
    peer-host-lib.sh 按地址取），去掉末尾的 /；读不到交空串。"""
    found = re.search(r"PEER_HOST_BUILD_ROOT=\"?([^\"\n]+?)\"?$", declarations or "", re.MULTILINE)
    return found.group(1).rstrip("/") if found else ""


def df_available_bytes(output):
    """df -B1 --output=avail 的输出（表头一行、数一行）：末行的字节数；认不出交 None。"""
    lines = [line.strip() for line in (output or "").splitlines() if line.strip()]
    return int(lines[-1]) if lines and lines[-1].isdigit() else None


def gibibytes_text(byte_count):
    return f"{byte_count / lib.GIBIBYTE:.1f}G"


def reminder_threshold(value, first):
    """要处置的提醒跨过了几道提醒门槛（文件头 ①「要处置的提醒」）：门槛是 first、2 × first、4 × first……；value 不到 first 交 0，到了 first 交 1，
    到了 2 × first 交 2，到了 4 × first 交 3……"""
    if first <= 0 or value < first:
        return 0
    crossed = 1
    while value >= first * 2 ** crossed:
        crossed += 1
    return crossed


def next_reminder_seconds(crossed, first):
    """跨过 crossed 道提醒门槛（reminder_threshold）之后，下一回提醒的那道门槛：first × 2 的 crossed 次方。"""
    return first * 2 ** crossed


def attention_event(item):
    """发消息那一路认一项提醒的键（inbox_failed、attention_held）：(任务号, 种类, 提醒门槛)；再推的那一回另接第几回（同一道门槛再推的每一回各算一件，发失败的那一回不连累下一回）。"""
    key = (item["task"], item["kind"], item["threshold"])
    return key + (item["repush"],) if item.get("repush") else key


def task_id_ranges(task_ids):
    """几个任务号写成区间（合成的那一条提醒点名几片子任务，Scheduler.merged_attention_text）：从小到大，连着的写「头–尾」、单个的写号，几段用「、」接。"""
    numbers = sorted({int(task_id) for task_id in task_ids})
    pieces, start = [], 0
    for index, number in enumerate(numbers):
        if index + 1 == len(numbers) or numbers[index + 1] != number + 1:
            pieces.append(str(numbers[start]) if start == index else f"{numbers[start]}–{number}")
            start = index + 1
    return "、".join(pieces)


# ── 两台：这一台怎么读、怎么起（文件头 ②「两台同一段逻辑」）──
@dataclasses.dataclass
class BuildDirectory:
    """交给那一台起法的这一次编译目录（Machine.command 的 build）。"""
    directory: str   # target 的上一层
    reused: bool     # 接着用上一次尝试留下的那一份（文件头「编译目录」的重试复用）


@dataclass(frozen=True)
class ThreadReading:
    """一次尝试某一轮的线程读数（Machine.thread_reading；文件头 ②「CPU 闸」的可跑线程数）：sample 是那一侧的采样号（本机是 attempt_runner 的轮数，第二台是 poll=），
    runnable 是状态 R 的线程数，total 是线程总数，top_threads、top_name 是占线程最多的那个进程的线程数与进程名。"""
    sample: int
    runnable: int
    total: int
    top_threads: int
    top_name: str


def thread_reading_of(value):
    """本机尝试目录里 lib.THREADS_FILE_NAME 那一份（attempt_runner.py 写的 JSON）→ ThreadReading；几样不是非负整数、进程名不是字符串的交 None（这一轮读不到）。"""
    if not isinstance(value, dict):
        return None
    numbers = [value.get(name) for name in ("sample", "runnable", "total", "top_threads")]
    if not all(isinstance(number, int) and not isinstance(number, bool) and number >= 0 for number in numbers) or not isinstance(value.get("top_name"), str):
        return None
    return ThreadReading(*numbers, value["top_name"])


class Machine:
    """一台机器在服务这一侧的接口：记账、判放不放得下、判卡住、记峰值、上限、编译目录、挑台与盘余量对两台走同一段（Scheduler 里），
    按台不同的只在这里——怎么连上它、怎么量它的余量与盘、怎么读一次尝试此刻的 CPU 与编译目录占盘、怎么起那一次、它的编译目录由谁建。
    两台型号相同（用户 2026-11-02：「两台型号都是一样的 根本没有必要写 if 第一台 else 第二台这种代码」）：这里只有摸到那台的路不同，判法不分台。"""
    name = ""
    always_reachable = False     # 服务所在这一台总连得上；另一台要照多机配置判法与 NFS 复核
    starts_per_pass = None       # 一轮至多在这台起几条（None 不限）
    admission_variable = ""      # 这一台最外层那一层只认服务起的准入标记（task_server_lib.py 的 ADMITTED_VARIABLE / PEER_ADMITTED_VARIABLE）
    admission_break = None       # 只给自证证红：照改前不设这一台准入标记的弄坏开关

    def __init__(self, scheduler):
        self.scheduler = scheduler

    @property
    def label(self):
        return lib.HOST_NAMES[self.name]

    def address(self, addresses):
        """这一台在多机配置 HOSTS 里的地址（Addresses 是 task_server_lib.host_addresses 交的）。"""
        raise NotImplementedError

    def reachable(self, resources):
        """→ (此刻连不连得上, 连不上的原因)。"""
        raise NotImplementedError

    def probe(self, resources):
        """→ (这台此刻的余量报告或 None, 查不成的原因)；报告里另带 DISK_REPORT_KEY（这台编译目录所在盘的余量，量得到才带）。不读卡：卡归 GPU 批处理验证服务。"""
        raise NotImplementedError

    def disk_free(self, resources):
        """报告里没带盘余量时现量：→ (退出码, 输出, 错误)，输出末行是字节数（同 df -B1 --output=avail）。"""
        raise NotImplementedError

    def disk_advice(self):
        raise NotImplementedError

    def configured_cap(self, resources):
        """这台的总上限另收到的数（字节）：多机配置 SLICE_MEMORY_CAP_BY_HOST 里这台那一项照它现读得到的；没有另收的交 None。"""
        return None

    def cpu_usec(self, attempt):
        """这一次尝试此刻累计用了多少 CPU（微秒）；读不到交 None。"""
        raise NotImplementedError

    def build_bytes(self, attempt):
        """这一次尝试的编译目录此刻的占盘（字节）；量不到交 None（记账照起的时候记的算）。"""
        return None

    def memory_now(self, attempt):
        """这一次尝试外层那一层内存包装 scope 此刻的内存与峰值（字节）：→ (此刻, 峰值)，读不到的那一样交 None（文件头 ①「顶在上限上卡住」）。"""
        return None, None

    def process_ids_seen(self, attempt, result):
        """这一次尝试进程树里见过的进程号（集合）：本机是 attempt_runner.py 每轮采样记进 result.json 的 process_ids；拿不到交 None（第二台的进程号服务这一侧读不到，
        external_kill_note 对第二台的尝试不下判）。"""
        return None

    def attempt_progress_readings(self, attempt):
        """日志不涨时的进展读数（lib.attempt_progress_readings；文件头 ①「要处置的提醒」的 quiet-log-busy）：→ (读数或 None, 读不成的那一句)。"""
        return None, "这一台不读进展"

    def io_blocked(self, attempt):
        """这一次尝试此刻在 D 状态（不可中断的 IO 等待）的进程数；读不到交 None（文件头 ①「卡在 IO 上」：照原判法判）。"""
        return None

    def thread_reading(self, attempt):
        """这一次尝试最近一轮的线程读数（ThreadReading；文件头 ②「CPU 闸」的可跑线程数）；读不到交 None（这一轮 CPU 闸只照在用核数）。"""
        return None

    def attempt_presence_readings(self, request):
        """读不到单元状态时不经 systemd 核一次尝试在这一台上还有没有进程（文件头 ①「读不到单元状态」）：在这一台上跑 task_server_lib.attempt_presence_readings
        → (读数或 None, 读不成的那一句)。两台跑同一段，只是摸到那台的路不同。"""
        raise NotImplementedError

    def command_exit(self, exit_code, tail):
        """→ (定结局用的退出码, classify_attempt 的 host, 起法那一层自己退的码或 None)。"""
        return exit_code, self.name, None

    def memory_stat_at_peak(self, attempt, result):
        """这一次尝试内存最高那一刻的 memory.stat（文件头 ①「内存最高那一刻」）：本机是 attempt_runner.py 交回的 memory_stat_at_peak；没采到交 None。"""
        return (result or {}).get("memory_stat_at_peak")

    def ended_peaks(self, attempt, tail):
        """尝试结束时起法那一层打进日志的峰值：{memory_peak, build_peak}（没打的不带）。"""
        return {}

    def kept_build_label(self, tail):
        """尝试结束时起法那一层报的「这一次的编译目录留着」（第二台是 multi-host-run.sh 打的 PEER_BUILD_KEPT_MARK 那一行）：→ 留着的那一份的标签或 None。"""
        return None

    def command(self, task, placement, label, environment, capped, build):
        """build 是这一次的编译目录（BuildDirectory；服务不给编译目录的是 None）。"""
        raise NotImplementedError

    def build_directory(self, label):
        """这一次尝试的编译目录：→ (目录, 是不是服务在本机建、由服务删)。目录是 target 的上一层；给不出交空串。"""
        raise NotImplementedError

    def build_directory_exists(self, directory):
        """一次尝试记着、没记成已删的编译目录此刻还在不在（文件头「编译目录」的重试复用与收尾删）。"""
        raise NotImplementedError

    def remove_build_directory(self, directory):
        """删一次尝试留下的编译目录：→ (删了、回读不在了, 没删成时的说明行)。有进程开着里面的文件就不删。"""
        raise NotImplementedError

    def removal_howto(self, directory):
        """没删成时告警的出路。"""
        raise NotImplementedError


class LocalMachine(Machine):
    """服务所在这一台：余量照 host-dispatch.py 的 probe_local 现读，CPU 读 attempt_runner 写的 cpu_usec，起法是内存包装。"""
    name = LOCAL
    always_reachable = True
    admission_variable = lib.ADMITTED_VARIABLE

    def address(self, addresses):
        return addresses.this

    def reachable(self, resources):
        return True, ""

    def probe(self, resources):
        report, why = resources.host_dispatch.probe_local(resources.settings, False)
        if report is not None:
            with contextlib.suppress(OSError):   # 本机编译目录所在盘（TASK_SERVER_BUILD_DIRECTORY）的余量
                report[DISK_REPORT_KEY] = shutil.disk_usage(resources.build_directory).free
        return report, why

    def disk_free(self, resources):
        try:
            return 0, f"Avail\n{shutil.disk_usage(resources.build_directory).free}\n", ""
        except OSError as error:
            return 1, "", f"{type(error).__name__}: {error}"

    def disk_advice(self):
        return "先 task-server/task clean 删掉跑完留下的编译目录（多机配置 TASK_SERVER_BUILD_DIRECTORY 所在盘）与别的大目录"

    def cpu_usec(self, attempt):
        return self.scheduler.scope_cpu_usec(attempt)

    def process_ids_seen(self, attempt, result):
        ids = (result or {}).get("process_ids")
        return {process_id for process_id in ids if isinstance(process_id, int) and not isinstance(process_id, bool)} if isinstance(ids, list) else None

    def memory_now(self, attempt):
        return self.scheduler.scope_memory_current(attempt), self.scheduler.scope_memory_peak(attempt)

    def io_blocked(self, attempt):
        return self.scheduler.scope_blocked_processes(attempt)

    def thread_reading(self, attempt):
        return self.scheduler.scope_thread_reading(attempt)

    def build_bytes(self, attempt):
        with contextlib.suppress(OSError, ValueError):
            with open(os.path.join(attempt.get("directory", ""), "build_bytes"), encoding="utf-8") as handle:
                return int(handle.read().strip())
        return None

    def attempt_progress_readings(self, attempt):
        # 服务就在这一台：直接调。进程树从起跑标记记的 runner 进程号往下，顶层日志不算（它的大小里有起法那一层的进度行，服务照 log_content 判）。
        # 只在自证里：/proc 换成 TASK_SERVER_SELFTEST_PROC_ROOT
        scheduler = self.scheduler
        environment = scheduler.environment
        runner, problem = recorded_text(scheduler.attempt_paths(attempt)[1], "起跑标记")
        if problem or not runner.isdigit():
            return None, problem or "起跑标记还没写进程号，读不了进程树"
        request = {"root_process_ids": [int(runner)], "excluded_files": [attempt["log"]] if attempt.get("log") else [],
                   "proc_root": lib.selftest_override("TASK_SERVER_SELFTEST_PROC_ROOT", environment) or "/proc",
                   "file_limit": lib.ATTEMPT_PROGRESS_FILE_LIMIT,
                   "write_mode_unchecked": lib.broken("open-file-write-mode-unchecked", environment)}
        try:
            return lib.attempt_progress_readings(request), ""
        except Exception as error:   # 读法自己出事照读不成办：这一回没有读数
            return None, f"本机读进展出了异常（{type(error).__name__}: {error}）"

    def attempt_presence_readings(self, request):
        # 服务就在这一台：直接调。只在自证里：世界目录里有 UNIT_CGROUPS_UNREADABLE_FILE 时 cgroup 根换成世界目录里不在的一处（演 cgroup 也读不到）
        world = lib.selftest_override("TASK_SERVER_SELFTEST_WORLD", self.scheduler.environment)
        if world and os.path.exists(os.path.join(world, UNIT_CGROUPS_UNREADABLE_FILE)):
            request = dict(request, cgroup_root=os.path.join(world, UNIT_CGROUPS_UNREADABLE_FILE + "-root"))
        try:
            return lib.attempt_presence_readings(request), ""
        except Exception as error:   # 读法自己出事照读不成办：核不出，下一回再核
            return None, f"本机核进程出了异常（{type(error).__name__}: {error}）"

    def command(self, task, placement, label, environment, capped, build):
        # 编译目录（连同重排接着用的那一份）经 environment 的 CARGO_TARGET_DIR 交给命令（start_attempt），这里不用 build
        return ["bash", self.scheduler.memory_wrapper, lib.mebibytes_text(placement.cap)] + capped + task["command"]

    def build_directory(self, label):
        return os.path.join(self.scheduler.configuration.build_directory, label), True

    def build_directory_exists(self, directory):
        return os.path.isdir(directory)

    def remove_build_directory(self, directory):
        removed, lines = self.scheduler.build_directories.remove_run_build_directory(directory)
        return removed and not os.path.lexists(directory), lines

    def removal_howto(self, directory):
        return "照上面那句停掉开着它的进程，再 python3 scripts/hosts/run_build_directory.py remove " + directory


class PeerMachine(Machine):
    """多机配置 HOSTS 里的另一台：经 host-dispatch.py 的 Peer（多机配置判法）连、probe_peer 在那台上量余量；CPU 与内存读日志里 multi-host-run.sh 每轮打的
    MULTI_HOST_RUN_PEER_CPU 与 MULTI_HOST_RUN_PEER_MEMORY；起法是 multi-host-run.sh --peer，编译目录由它在那台本地盘上这一趟新建、跑完删（multi-host-run.sh 文件头 ⑤）。"""
    name = PEER
    starts_per_pass = PEER_STARTS_PER_PASS
    admission_variable = lib.PEER_ADMITTED_VARIABLE
    admission_break = "peer-admission-unmarked"

    def address(self, addresses):
        return addresses.peer

    def reachable(self, resources):
        return resources.peer.available, resources.peer.why

    def probe(self, resources):
        return resources.host_dispatch.probe_peer(resources.peer, False, timeout=PEER_PROBE_TIMEOUT_SECONDS)   # 查不成的原因留给 note_peer 告警

    def disk_free(self, resources):
        directory = peer_build_root(resources.peer.declarations)
        if not directory:
            return 1, "", "多机配置 BUILD_ROOT_BY_HOST 里读不到第二台那一项"
        return resources.peer.run(PEER_DISK_PROBE_TEXT.format(directory=shlex.quote(directory)), timeout=PEER_PROBE_TIMEOUT_SECONDS)

    def disk_advice(self):
        return ("先删第二台的编译根（多机配置 BUILD_ROOT_BY_HOST 里它那一项）下 builds/ 里留下的本地目录（multi-host-run.sh 没删成的那几个，它当时报了原因）"
                "与别的大目录")

    def configured_cap(self, resources):
        return resources.peer.memory_cap_bytes if resources.peer is not None else None

    def cpu_usec(self, attempt):
        return self.scheduler.log_offsets.get(attempt.get("id"), (0, 0, None))[2]

    def attempt_progress_readings(self, attempt):
        # 经多机配置的读法（host-dispatch.py 的 Peer.run，同 attempt_presence_readings）在第二台起同一段 attempt_progress_readings：进程树从清理回执点名的
        # runs/<标签>.pid 记的进程号往下（multi-host-run.sh 在第二台起的那一层；它的标准输出经 ssh 接回本机，不是第二台上的文件）
        resources = self.scheduler.resources
        if resources.fake_path:   # 自证不连第二台（Resources 的假余量世界里 self.peer 一直是 None）
            return None, "自证里不连第二台"
        path = attempt.get("peer_cleanup_file") or ""
        receipt = read_json_file(path) if path else None
        pid_file = (receipt or {}).get("pid_file")
        if (receipt or {}).get("status") != "pending" or not isinstance(pid_file, str) or not pid_file.startswith("/"):
            return None, f"清理回执 {path or '（这一次没有）'} 不是 pending 或没点名进程号文件，第二台那一趟读不了进展"
        resources.refresh_peer()
        peer = resources.peer
        if peer is None or not peer.available:
            return None, f"第二台连不上（{peer.why if peer is not None else '多机配置里的第二台没读出来'}）"
        request = {"root_process_id_files": [pid_file], "file_limit": lib.ATTEMPT_PROGRESS_FILE_LIMIT,
                   "write_mode_unchecked": lib.broken("open-file-write-mode-unchecked", self.scheduler.environment)}
        status, stdout, stderr = peer.run(lib.attempt_progress_probe_text(request), timeout=lib.ATTEMPT_PROGRESS_PEER_SECONDS)
        readings = lib.attempt_progress_probe_reply(stdout)
        if status != 0 or readings is None:
            return None, f"第二台读进展没成（退 {status}：{stderr.strip()[-200:]}）"
        return readings, ""

    def attempt_presence_readings(self, request):
        # 经多机配置的读法（host-dispatch.py 的 Peer.run，同 attempt_progress_readings）在第二台起同一段 attempt_presence_readings
        resources = self.scheduler.resources
        if resources.fake_path:   # 自证不连第二台（Resources 的假余量世界里 self.peer 一直是 None）
            return None, "自证里不连第二台"
        cooling = resources.peer_cooling_down()   # 上一回读不成还在冷却里：不去连
        if cooling:
            return None, cooling
        resources.refresh_peer()
        peer = resources.peer
        if peer is None or not peer.available:
            return None, f"第二台连不上（{peer.why if peer is not None else '多机配置里的第二台没读出来'}）"
        status, stdout, stderr = peer.run(lib.attempt_presence_probe_text(request), timeout=lib.ATTEMPT_PRESENCE_PEER_SECONDS)
        readings = lib.attempt_presence_probe_reply(stdout)
        if status != 0 or readings is None:
            return None, f"第二台核进程没成（退 {status}：{stderr.strip()[-200:]}）"
        return readings, ""

    def memory_stat_at_peak(self, attempt, result):
        # 日志里 multi-host-run.sh 每轮打的 MULTI_HOST_RUN_PEER_MEMORY_STAT 里 current_bytes 最大的那一行（log_content 读进 peer_memory_stat；文件头 ①「内存最高那一刻」）
        return self.scheduler.peer_memory_stat.get(attempt.get("id"))

    def memory_now(self, attempt):
        # 日志里 multi-host-run.sh 每轮打的最近一行 MULTI_HOST_RUN_PEER_MEMORY（log_content 读进 peer_memory）；弄坏开关 peer-memory-line-unread 下不读
        if lib.broken("peer-memory-line-unread", self.scheduler.environment):
            return None, None
        return self.scheduler.peer_memory.get(attempt.get("id"), (None, None))

    def io_blocked(self, attempt):
        # 日志里 multi-host-run.sh 每轮打的 MULTI_HOST_RUN_PEER_IO_WAIT（log_content 接进 peer_io_rounds）：上一次读之后接到的几行取最少的，有一行 read=no 记 None
        return self.scheduler.merge_peer_io_rounds(attempt)

    def thread_reading(self, attempt):
        # 日志里 multi-host-run.sh 每轮打的 MULTI_HOST_RUN_PEER_THREADS（log_content 接进 peer_thread_rounds）：上一次读之后接到的几行里 read=yes 的取最大
        return self.scheduler.merge_peer_thread_rounds(attempt)

    def command_exit(self, exit_code, tail):
        # 第二台那条命令自己的退出码（multi-host-run.sh 打的 MULTI_HOST_RUN_STEP step=peer-command event=end … exit=<码>）：有这一行照它定结局，
        # 命令自己退 255 记 command-failed；没有这一行才是 multi-host-run.sh 那一层没跑成（task_server_lib.py 文件头「错误类型」）
        command_exit = lib.peer_command_exit(tail, self.scheduler.environment) if exit_code is not None else None
        if command_exit is None:
            return exit_code, self.name, None
        return command_exit, lib.PEER_COMMAND_HOST, exit_code

    def ended_peaks(self, attempt, tail):
        # multi-host-run.sh 在第二台那条命令结束时打的 MULTI_HOST_RUN_PEER_MEMORY_PEAK 与 MULTI_HOST_RUN_PEER_BUILD_PEAK；读不到不打
        environment = self.scheduler.environment
        peaks = {"memory_peak": None if lib.broken("peer-memory-peak-ignored", environment) else lib.peer_marked_bytes(tail, lib.PEER_MEMORY_PEAK_MARK),
                 "build_peak": lib.peer_marked_bytes(tail, lib.PEER_BUILD_PEAK_MARK)}
        return {name: value for name, value in peaks.items() if value is not None}

    def kept_build_label(self, tail):
        return lib.peer_kept_build_label(tail)

    def build_directory_exists(self, directory):
        # 第二台本地盘上的那一份服务这一侧看不到：记着留下了就当在（multi-host-run.sh 接着用之前在第二台上核它在不在，不在退 255）
        return True

    def remove_build_directory(self, directory):
        # multi-host-run.sh --remove-build-directory 在第二台上核没有进程开着里面的文件、删、回读不在了才退 0（它文件头「留着的编译目录」）；
        # 第二台上一回读不成还在冷却里的不去连（Resources.peer_cooling_down），照没删成交回
        cooling = self.scheduler.resources.peer_cooling_down()
        if cooling:
            return False, [cooling]
        try:
            completed = subprocess.run(["bash", self.scheduler.peer_runner, "--remove-build-directory", os.path.basename(directory)], capture_output=True, text=True,
                                       env=lib.ssh_multiplexed_environment(self.scheduler.environment, self.scheduler.environment),   # 服务起的 ssh 复用连接（task_server_lib.py 文件头「主机」）
                                       stdin=subprocess.DEVNULL, timeout=PEER_BUILD_REMOVE_SECONDS)
        except (OSError, subprocess.TimeoutExpired) as error:
            return False, [f"multi-host-run.sh --remove-build-directory 起不了或超过 {PEER_BUILD_REMOVE_SECONDS} 秒：{error}"]
        return completed.returncode == 0, (completed.stdout + completed.stderr).splitlines()[-5:]

    def removal_howto(self, directory):
        return ("照上面那句到第二台停掉开着它的进程（python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>），再 bash scripts/hosts/multi-host-run.sh --remove-build-directory "
                + os.path.basename(directory))

    def command(self, task, placement, label, environment, capped, build):
        scheduler = self.scheduler
        # 两个共享根照来源仓根那样原样带过去（task_server_lib.py 文件头「共享根」）；弄坏开关 shared-roots-not-passed 下照改前不带
        shared_roots = () if lib.broken("shared-roots-not-passed", scheduler.environment) else lib.SHARED_ROOT_VARIABLES
        carried_names = ("TASK_SERVER_TASK", "TASK_SERVER_ATTEMPT", lib.ATTEMPT_LABEL_VARIABLE, lib.THREAD_ADMITTED_VARIABLE, lib.MEMORY_CAP_VARIABLE,
                         lib.SOURCE_REPOSITORY_VARIABLE, lib.SHARD_PHASE_VARIABLE, lib.SHARD_MAXIMUM_VARIABLE, lib.SHARD_VARIABLE, lib.SHARD_PLAN_VARIABLE, lib.SHARD_OUTPUT_VARIABLE, lib.SHARD_HOST_VARIABLE,
                         lib.SHARD_OUTPUTS_VARIABLE, lib.SHARD_MISSING_VARIABLE, *shared_roots)   # 合并那一步派到第二台也读得到各片的清单（那份 TSV 在共享根下的分片根里）与缺了哪几片
        if lib.broken("shard-outputs-not-carried", scheduler.environment):   # 弄坏开关：照改前不带各片的清单（task_server_lib.py 文件头「分片」③）
            carried_names = tuple(name for name in carried_names if name != lib.SHARD_OUTPUTS_VARIABLE)
        pairs = [f"{name}={value}" for name, value in sorted(environment.items()) if name in task["environment"] or name in carried_names]
        # 本机的路径与编译目录，第二台用它自己的（编译目录由 multi-host-run.sh 在第二台本地盘上建）
        pairs = [pair for pair in pairs if not pair.startswith(("PATH=", "CARGO_HOME=", "RUSTUP_HOME=", "TMPDIR=", *(name + "=" for name in lib.TARGET_DIRECTORY_VARIABLES)))]
        if not lib.broken("host-specific-variables-sent-to-peer", scheduler.environment):
            # 按台取值的（预编库这类）不带；值是本机绝对路径的只带落在共享根下的（第二台照同一个绝对路径读得到：来源仓根、两个共享根、分片的计划与产物目录），别的第二台用它自己的
            pairs = [pair for pair in pairs if pair.split("=", 1)[0] not in PEER_HOST_SPECIFIC_VARIABLES
                     and (not pair.split("=", 1)[1].startswith("/") or pair.split("=", 1)[0] in (lib.SOURCE_REPOSITORY_VARIABLE, *shared_roots)
                          or scheduler.containing_shared_root(os.path.realpath(pair.split("=", 1)[1])))]
        # 判定库服务的地址不带过去：第二台照 multi-host-run.sh 带过去的 SINGLEFS_MULTI_HOST_CONFIG 读同一份多机配置里的 PRUNE_SERVER_TCP_ADDRESS；
        # 配置里没写，第二台上的判定库客户端退 3、点名缺的是哪一样（scripts/prune-server/prune_server_client.py 文件头「连哪一处」），服务不从别处补
        command = list(task["command"])
        copied = []
        if lib.broken("peer-tree-copied", scheduler.environment):   # 弄坏开关：照改前把工作目录拷进第二台的一棵树、在那份拷贝里起
            copied = ["--send", task["cwd"], ".task-server-tree-copy"]
            command = ["env", "-C", ".task-server-tree-copy", *command]
        # 服务给编译目录的（会编的）叫 multi-host-run.sh 跑完留着这一份、重排到这一台的那一次接着用上一次留下的（文件头「编译目录」的重试复用，两台同一段）；
        # 弄坏开关 peer-retry-build-directory-fresh 下照改前不留、不接着用
        kept = []
        if build is not None and not lib.broken("peer-retry-build-directory-fresh", scheduler.environment):
            kept = ["--keep-build-directory"] + (["--reuse-build-directory", os.path.basename(build.directory)] if build.reused else [])
        return (["bash", scheduler.peer_runner, "--peer", "--memory-cap", lib.mebibytes_text(placement.cap), "--label", label, *kept, *copied, "--"]
                + (["env", *pairs] if pairs else []) + capped + command)

    def build_directory(self, label):
        # multi-host-run.sh 在这台的编译根（多机配置 BUILD_ROOT_BY_HOST 里它那一项）下 builds/<标签>/ 新建、跑完删（带 --keep-build-directory 的留着，服务删）；服务只记下它在哪
        resources = self.scheduler.resources
        root = resources.build_root_of_peer()
        return (os.path.join(root, "builds", label) if root else ""), False


class Resources:
    """两台的余量：平常经各台的 Machine.probe 现查（缓存 RESOURCE_CACHE_SECONDS 秒，尝试结束时与起之前复核时作废）；自证里读 TASK_SERVER_FAKE_RESOURCES 那份 JSON（每次现读）。
    两台同一套判法：每台照它自己的接口（Machine）连、量，盘余量照同一个下限判（disk_problem）。"""

    def __init__(self, environment, host_dispatch, build_directory, configuration_path):
        self.environment = environment
        self.host_dispatch = host_dispatch
        self.build_directory = build_directory   # 本机编译目录所在盘（TASK_SERVER_BUILD_DIRECTORY）：余量记进本机报告的 DISK_REPORT_KEY
        self.configuration_path = configuration_path   # 多机配置：两台在 HOSTS 里的地址照它认（addresses）
        self.addresses_read = None   # (配置文件的修改时刻与大小, 读的时刻, Addresses 或 None, 问题)
        self.fake_path = lib.selftest_override("TASK_SERVER_FAKE_RESOURCES", environment)
        self.slow_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_SLOW_RESOURCES_SECONDS", environment) or 0)
        self.settings = host_dispatch.settings_from_environment()
        self.settings.say = lambda line: None
        # host-dispatch.py 经 peer-host-lib.sh 起的 ssh 接上复用层（task_server_lib.py 文件头「主机」）：PATH 前面接控制目录下的 bin/ssh
        self.settings.environment = lib.ssh_multiplexed_environment(self.settings.environment, environment)
        self.cached = None   # (读的时刻, Snapshot)：本机那一份的缓存
        self.peer = None
        self.peer_read_at = 0.0
        self.machines = {}   # {台: Machine}：Scheduler 建好之后放进来（两台各一份接口）
        # 第二台的余量（peer_probe_loop 读，snapshot 只取最近一份；文件头 ②「第二台的余量不在排队线程里读」）
        self.peer_lock = threading.Lock()
        self.peer_reading = None   # 最近一份 PeerReading
        self.peer_failed = None   # (失败的时刻 monotonic, 原因)：读不成的那一回之后 PEER_PROBE_FAILURE_HOLD_SECONDS 之内不再去连
        self.peer_refresh_seconds = FAST_PEER_PROBE_REFRESH_SECONDS if lib.fast(environment) else PEER_PROBE_REFRESH_SECONDS

    def invalidate(self):
        """作废本机的缓存（尝试结束、起之前复核时）；第二台的那一份归探测线程（peer_probe_loop），不跟着清。"""
        self.cached = None

    def build_root_of_peer(self):
        """第二台的编译根：真服务读多机配置里它那一项（peer_build_root）；自证读假余量 JSON 的 peer_build_root（自证不连真的第二台、self.peer 永远是 None，
        假 multi-host-run.sh 在那个目录下 builds/<标签>/ 建这一趟的编译目录）。读不到交空串。"""
        if self.fake_path:
            return (read_json_file(self.fake_path) or {}).get("peer_build_root") or ""
        return peer_build_root(self.peer.declarations) if self.peer is not None else ""

    def addresses(self):
        """(Addresses(this, peer) 或 None, 问题)：服务所在这一台与另一台在 HOSTS 里的地址。配置文件没改过、又没过 PEER_RECHECK_SECONDS 的照上一次的，不每轮都起判法。"""
        try:
            status = os.stat(self.configuration_path)
            stamp = (status.st_mtime_ns, status.st_size)
        except OSError:
            stamp = None
        cached = self.addresses_read
        if cached is not None and cached[0] == stamp and time.monotonic() - cached[1] < PEER_RECHECK_SECONDS:
            return cached[2], cached[3]
        addresses, problem = lib.host_addresses(self.configuration_path, self.environment)
        self.addresses_read = (stamp, time.monotonic(), addresses, problem)
        return addresses, problem

    def peer_cooling_down(self):
        """第二台上一回读不成、还在 PEER_PROBE_FAILURE_HOLD_SECONDS 之内（peer_failed）→ 那一句；不在交空串。收尾线程、核进程线程、空转线程这几处连第二台之前先问它，
        冷却里直接交「连不上」、不去连（不在第二台断着时每一处各等满自己的 ssh 时限）。弄坏开关 peer-cooldown-ignored-off-scheduler 下一律交空串。"""
        failed = self.peer_failed
        if failed is None or lib.broken("peer-cooldown-ignored-off-scheduler", self.environment) or time.monotonic() - failed[0] >= PEER_PROBE_FAILURE_HOLD_SECONDS:
            return ""
        return f"第二台上一回读不成（{failed[1]}），{PEER_PROBE_FAILURE_HOLD_SECONDS:.0f} 秒之内不再去连"

    def refresh_peer(self):
        """多机配置里的另一台（host-dispatch.py 的 Peer：判法里要连第二台，带 PEER_PROBE_TIMEOUT_SECONDS 的时限）至多每 PEER_RECHECK_SECONDS 重读一次。
        只在探测线程里调（peer_probe_round），排队线程不等它。"""
        if self.peer is None or time.monotonic() - self.peer_read_at > PEER_RECHECK_SECONDS:
            self.peer = self.host_dispatch.Peer(self.settings, timeout=PEER_PROBE_TIMEOUT_SECONDS)
            self.peer_read_at = time.monotonic()

    def read_peer(self, machine):
        """连第二台读一回（探测线程里，或还一份都没有时头一回）→ PeerReading：多机配置判得过才探测，探测到的照 disk_problem 判盘余量（同 snapshot 里本机那一路）。"""
        self.refresh_peer()
        reachable, why = machine.reachable(self)
        report, kind = None, ""
        if reachable:
            report, why = machine.probe(self)
        if report is not None:
            disk_problem = self.disk_problem(machine, report)
            if disk_problem:
                report, why, kind = None, disk_problem, DISK_FLOOR_KIND
        return PeerReading(time.monotonic(), report, why, kind, bool(reachable))

    def peer_probe_round(self):
        """探测线程的一回（文件头 ②「第二台的余量不在排队线程里读」）：上一回读不成的 PEER_PROBE_FAILURE_HOLD_SECONDS 之内不再去连。
        读不成（不是盘余量不到下限）记进 peer_failed。"""
        machine = self.machines.get(PEER)
        if machine is None or self.fake_path:
            return
        if self.peer_failed is not None and time.monotonic() - self.peer_failed[0] < PEER_PROBE_FAILURE_HOLD_SECONDS:
            return
        reading = self.read_peer(machine)
        with self.peer_lock:
            self.peer_reading = reading
        failed = None
        if reading.report is None and reading.kind != DISK_FLOOR_KIND and reading.reachable:
            failed = (time.monotonic(), reading.why)
        self.peer_failed = failed

    def peer_probe_loop(self, stopping):
        """探测线程：每 peer_refresh_seconds 一回（peer_probe_round）；一回出事不停服务，打出来、下一回再读。"""
        while not stopping.wait(self.peer_refresh_seconds):
            try:
                self.peer_probe_round()
            except Exception:
                say("  ! 这一回读第二台的余量出了异常（下一回再读）：\n" + traceback.format_exc())

    def peer_snapshot_entry(self, machine):
        """snapshot 里第二台那一项 → (连不连得上, 报告的拷贝或 None, 原因, down_kind)：取探测线程最近一份（peer_probe_loop），不在调用方的线程里连第二台。
        一份都还没有（服务刚起）的这一回照 read_peer 读一次；上一回读不成、还在 PEER_PROBE_FAILURE_HOLD_SECONDS 之内的照那一句当此刻用不了；
        最近一份比 PEER_PROBE_STALE_SECONDS 旧（探测线程卡在第二台上）的同样当此刻用不了。弄坏开关 peer-probe-in-scheduler 下每次在调用方的线程里现连、读不成不记。"""
        if lib.broken("peer-probe-in-scheduler", self.environment):
            reading = self.read_peer(machine)
            return reading.reachable, (dict(reading.report) if reading.report is not None else None), reading.why, reading.kind
        with self.peer_lock:
            reading = self.peer_reading
        failed = self.peer_failed
        now = time.monotonic()
        if failed is not None and now - failed[0] < PEER_PROBE_FAILURE_HOLD_SECONDS:
            return False, None, f"{machine.label}上一回读不成（{failed[1]}），{PEER_PROBE_FAILURE_HOLD_SECONDS:.0f} 秒之内不再去连", ""
        if reading is None:
            reading = self.read_peer(machine)
            with self.peer_lock:
                self.peer_reading = reading
            if reading.report is None and reading.kind != DISK_FLOOR_KIND and reading.reachable:
                self.peer_failed = (time.monotonic(), reading.why)
        elif now - reading.at > PEER_PROBE_STALE_SECONDS:
            return False, None, f"{machine.label}的余量最近一份是 {now - reading.at:.0f} 秒以前读的（探测线程卡在{machine.label}上），这一轮不派", ""
        return reading.reachable, (dict(reading.report) if reading.report is not None else None), reading.why, reading.kind

    def fake_report(self, fake, machine):
        """自证：假余量 JSON 里这一台的报告（{台: 报告}；down 里写了这一台的演它此刻用不了）→ (报告或 None, 原因)。"""
        why = (fake.get("down") or {}).get(machine.name)
        if fake.get(machine.name) and not why:
            return fake[machine.name], ""
        return None, why or f"自证里没给{machine.label}"

    def snapshot(self):
        fake = None
        if self.fake_path:
            if self.slow_seconds:   # 自证：演余量读得慢（/health 不该跟着慢）
                time.sleep(self.slow_seconds)
            fake = read_json_file(self.fake_path) or {}
        else:
            cached = self.cached
            if cached is not None and time.monotonic() - cached[0] < RESOURCE_CACHE_SECONDS:
                return cached[1]
        reports, reachable, down, down_kind, expected, caps = {}, {}, {}, {}, {}, {}
        for host, machine in self.machines.items():
            judged = False   # 第二台那一项已在 read_peer 里判过盘余量
            if fake is not None:   # 假余量里写了 down（{台: 一句原因}）的演那台用不了：照真的那样要告警；没给那一台的格不当它用不了
                report, why = self.fake_report(fake, machine)
                reachable[host] = machine.always_reachable or report is not None
                expected[host] = host in fake or bool((fake.get("down") or {}).get(host))
                cap = (fake.get("memory_caps") or {}).get(host)
            elif host == PEER:   # 取探测线程最近一份（peer_snapshot_entry），不在这个线程里连第二台
                reachable[host], report, why, kind = self.peer_snapshot_entry(machine)
                if kind:
                    down_kind[host] = kind
                judged = True
                expected[host] = True
                cap = machine.configured_cap(self)
            else:
                reachable[host], why = machine.reachable(self)
                report = None
                if reachable[host]:
                    report, why = machine.probe(self)
                expected[host] = True
                cap = machine.configured_cap(self)
            if cap:
                caps[host] = int(cap)
            if report is not None and not judged:
                disk_problem = self.disk_problem(machine, report, fake)
                if disk_problem:   # 盘余量不到下限或查不成：这一轮不给这台的报告、不派它（同查不成余量的那一种，够了下一轮自己认回）
                    report, why, down_kind[host] = None, disk_problem, DISK_FLOOR_KIND
            if report is None:
                down[host] = why
            else:
                reports[host] = report
        snapshot = Snapshot(reports, caps, reachable, down, bool((fake or {}).get("hold")), expected=expected, down_kind=down_kind)
        if fake is None:
            self.cached = (time.monotonic(), snapshot)
        return snapshot

    def disk_problem(self, machine, report, fake=None):
        """这台编译目录所在盘的余量够不够（task_server_lib.py 文件头「挑机器」的「盘余量」；两台同一个下限 DISK_FLOOR_BYTES）：→ 不到下限或查不成的那一句，够了交空串。
        report 里有 DISK_REPORT_KEY 照它；没有就照这台的接口现量（Machine.disk_free：本机读编译目录所在盘，第二台在它的编译根上起 df）。
        自证里（fake 是假余量那份 JSON）没有这一项读 fake 的 df 里这一台那一项（演 df 的输出），都没写的当盘够。弄坏开关 disk-floor-unchecked 下一律交空串。"""
        if lib.broken("disk-floor-unchecked", self.environment):
            return ""
        available = report.get(DISK_REPORT_KEY)
        if not isinstance(available, int) or isinstance(available, bool):
            if fake is not None:
                shown = (fake.get("df") or {}).get(machine.name)
                if shown is None:
                    return ""
                status, output, error = 0, str(shown), ""
            else:
                status, output, error = machine.disk_free(self)
            available = df_available_bytes(output) if status == 0 else None
            if available is None:
                return f"{machine.label}盘余量查不成（退 {status}：{(error or output).strip()[-200:]}）"
            report[DISK_REPORT_KEY] = available   # 记进报告：派活时照它判这一条的编译目录占盘放不放得下（文件头 ②「编译目录占盘」）
        if available < DISK_FLOOR_BYTES:
            return f"{machine.label}盘只剩 {gibibytes_text(available)}（下限 {gibibytes_text(DISK_FLOOR_BYTES)}）：编译会在中途写满，{machine.disk_advice()}"
        return ""

    def recheck(self, host, base):
        """起之前现读一次那一台（文件头 ②「起之前复核」）：缓存作废，交一份新的 Snapshot，这台的余量是现读的，别的照 base；
        现读到的报告照 snapshot 那样判盘余量（disk_problem：不到下限或查不成的当这台这一轮读不到，down_kind 记 DISK_FLOOR_KIND；第二台的探测不带盘余量，
        这里现量的那一份记进报告，编译目录占盘照它判）。这台读不到的，交的那份里没有它的报告、down 写原因。
        自证里读假余量 JSON，里面写了 recheck 的（{台: 余量}，写 null 的演这台这一次读不到）这一次照它。弄坏开关 recheck-disk-unchecked 下不判盘余量。"""
        self.invalidate()
        machine = self.machines[host]
        fake = None
        if self.fake_path:
            fake = read_json_file(self.fake_path) or {}
            report = dict(fake, **(fake.get("recheck") or {})).get(host)
            why = "" if report is not None else f"自证里复核时没给{machine.label}"
        elif host == PEER and self.peer_failed is not None and time.monotonic() - self.peer_failed[0] < PEER_PROBE_FAILURE_HOLD_SECONDS:
            # 第二台上一回读不成、还在不再去连的那一段里（peer_snapshot_entry）：复核同样不连，照那一句当这一次读不到
            report, why = None, f"{machine.label}上一回读不成（{self.peer_failed[1]}），{PEER_PROBE_FAILURE_HOLD_SECONDS:.0f} 秒之内不再去连"
        else:
            if host == PEER and self.peer is None:
                self.refresh_peer()
            reachable, why = machine.reachable(self)
            report = None
            if reachable:
                report, why = machine.probe(self)   # 第二台的探测与 df 都带 PEER_PROBE_TIMEOUT_SECONDS 的时限
        down = {name: text for name, text in base.down.items() if name != host}
        down_kind = {name: kind for name, kind in base.down_kind.items() if name != host}
        if report is not None and not lib.broken("recheck-disk-unchecked", self.environment):
            disk_problem = self.disk_problem(machine, report, fake)
            if disk_problem:   # 同 snapshot：盘余量不到下限或查不成，这一轮不给这台的报告、不派它
                report, why, down_kind[host] = None, disk_problem, DISK_FLOOR_KIND
        reports = {name: value for name, value in base.reports.items() if name != host}
        if report is not None:
            reports[host] = report
        else:
            down[host] = why or f"起之前复核读不到{machine.label}"
        return dataclasses.replace(base, reports=reports, down=down, down_kind=down_kind)


class PassView:
    """一轮排队读的余量（文件头 ②「余量一轮只读一次」）：每台的余量（Resources.snapshot）、服务账上还没涨到量的那一截（server_pending）
    在这一轮第一次用到时读，这一轮里各条共用；这一轮起了一条，它的记账加进那台的待涨量（book），后面的照加过的判。
    弄坏开关 pending-read-per-task 下每次用到都现读，pass-booking-not-carried 下起了的不加。"""

    def __init__(self, scheduler, source):
        self.scheduler = scheduler
        self.source = source              # pass（排队那一轮）、health（/health 判排头）
        self.number = next(scheduler.view_numbers)
        self.latest_snapshot = None       # 这一轮读进来的那一份 Snapshot：待涨量照它里面第二台的报告对 scope（server_pending）
        self.pending = {}                 # {台: 内存字节}：读到的待涨量
        self.disk_pending = {}            # {台: 字节}：服务账上这台在跑的尝试编译目录还要涨的占盘，连同这一轮起了的那几条的记账
        self.carried = {}                 # {台: 内存字节}：这一轮起了的那几条的记账

    def read_each_time(self):
        return lib.broken("pending-read-per-task", self.scheduler.environment)

    def snapshot(self):
        if self.latest_snapshot is None or self.read_each_time():
            self.scheduler.selftest_read(f"read view={self.number} source={self.source} what=snapshot")
            self.latest_snapshot = self.scheduler.peer_view(self.scheduler.resources.snapshot())
        return self.latest_snapshot

    def read_pending(self, host, source):
        """现读这台的待涨量；读进来的已含库里此刻在跑的每一次尝试，这一轮起了的那几条不再另加。第二台的用量照这一轮最近读进来的那一份余量报告
        （latest_snapshot；这一轮还没读余量时没有报告，第二台在跑的记整份上限），不为它另读一次余量。"""
        self.scheduler.selftest_read(f"read view={self.number} source={source} what=pending host={host}")
        report = self.latest_snapshot.reports.get(host) if self.latest_snapshot is not None else None
        self.pending[host] = self.scheduler.server_pending(host, report)
        self.carried.pop(host, None)

    def pending_of(self, host):
        """内存字节：这台还没涨到量的那一截，连同这一轮起了的那几条的记账。"""
        if host not in self.pending or self.read_each_time():
            self.read_pending(host, self.source)
        return self.pending[host] + self.carried.get(host, 0)

    def book(self, host, memory, disk=0):
        """这一轮在这台起了一条：把它的记账加进这一轮那台的待涨量。"""
        if lib.broken("pass-booking-not-carried", self.scheduler.environment):
            return
        self.carried[host] = self.carried.get(host, 0) + int(memory or 0)
        self.disk_pending[host] = self.disk_pending_of(host) + int(disk or 0)

    def disk_pending_of(self, host):
        """这台在跑的尝试的编译目录还要涨的占盘（server_disk_pending，这一轮第一次用到时读），连同这一轮起了的。"""
        if host not in self.disk_pending:
            self.disk_pending[host] = self.scheduler.server_disk_pending(host)
        return self.disk_pending[host]

    def refresh(self, host, snapshot):
        """起之前复核现读过的：现读的余量换进这一轮，这台的待涨量也现读（复核那一读记 source=recheck）。"""
        self.latest_snapshot = snapshot
        self.read_pending(host, "recheck")


@dataclass
class PeerReading:
    """探测线程读第二台的一回（Resources.read_peer）。"""
    at: float          # 读完的时刻（time.monotonic）
    report: object     # 余量报告；读不成或盘余量不到下限是 None
    why: str           # 读不成的原因
    kind: str          # down_kind：盘余量不到下限是 DISK_FLOOR_KIND，别的是空串
    reachable: bool    # 多机配置判得过、第二台用得上


@dataclass
class Placement:
    kind: str                  # start、blocked、never、unmet
    host: str = ""
    cap: int = 0
    booked: int = 0
    disk_booked: int = 0       # 编译目录占盘的记账（字节；文件头 ②「编译目录占盘」）
    history: str = ""
    why: str = ""
    cpu_small_slot: str = ""   # 靠 CPU 闸要核数少的名额起的那一句（cpu_small_slot_note）：起的时候记进尝试，cpu_small_slots_used 照它数


HEAD_STARTED = "started"                      # 派活这一轮判它能起，起了
HEAD_NOT_STARTED = "startable-not-started"    # 派活这一轮判它能起，却没起成（why 写为什么）：/health 只报连着几轮这样的
HEAD_BLOCKED = "not-startable"                # 派活这一轮判它不能起（why 写在等什么，与排队原因同一句）或判不了


@dataclass
class HeadVerdict:
    """派活这一轮给一条排着的任务的结论（Scheduler.place_queued 记，排头那一条换进 head_verdict；文件头「接口」的 /health）。"""
    task_id: int
    kind: str                  # HEAD_STARTED、HEAD_NOT_STARTED、HEAD_BLOCKED
    why: str = ""


@dataclass
class SuspectWindow:
    """判疑似卡住的一个 CPU 窗口（Scheduler.check_suspect）：时刻是 time.monotonic() 的钟。"""
    started_at: float
    started_cpu: int
    last_at: float
    last_cpu: int
    suspected: bool           # 这一段已报疑似卡住、还没结束


@dataclass
class IoWaitWindow:
    """判卡住的一个 CPU 窗口里每一轮读到的 D 状态进程数（Scheduler.note_io_wait；文件头 ①「卡在 IO 上」）：时刻是 time.monotonic() 的钟。"""
    window_started_at: float  # 这一个窗口开头的时刻（log_growth 记的那一个）：窗口换了从头记
    polls: int                # 这个窗口里读了几轮
    every_poll_blocked: bool  # 这个窗口里每一轮都读到了、都有进程在 D 状态
    blocked_now: object       # 最近一轮读到的 D 状态进程数（int；读不到是 None）
    blocked_since: object     # 从哪一刻起每一轮都有进程在 D 状态（float；跨窗口接着算，有一轮没有或读不到是 None；日志涨了整份删）


@dataclass
class ActivityWindow:
    """判卡住的一个窗口里这一次尝试的进程内存（Scheduler.note_activity；文件头 ①「卡住」的有进展）：窗口换了从头记。"""
    window_started_at: float  # 这一个窗口开头的时刻（log_growth 记的那一个）
    memory_first: object      # 这个窗口里头一回读到的进程内存（字节，Machine.memory_now 的此刻；读不到是 None）
    peak_first: object        # 同一回读到的这一次尝试的内存峰值（字节；读不到是 None）
    memory_last: object       # 最近一回读到的进程内存（读不到的那一回不盖掉）
    peak_last: object         # 最近一回读到的内存峰值（同上）


def signal_name(number):
    """信号号的名字（SIGTERM 这类）；认不出的写「信号 <号>」。"""
    try:
        return signal.Signals(number).name
    except ValueError:
        return f"信号 {number}"


def activity_window_after(previous, window_started_at, memory, peak):
    """ActivityWindow 记上这一轮读到的进程内存 memory 与内存峰值 peak（字节）：窗口换了（或头一回）从这一轮起记，
    读不到（None）的那一样不盖掉已记的。→ 新的那一份。"""
    if previous is None or previous.window_started_at != window_started_at:
        return ActivityWindow(window_started_at, memory, peak, memory, peak)
    return ActivityWindow(window_started_at,
                          previous.memory_first if previous.memory_first is not None else memory,
                          previous.peak_first if previous.peak_first is not None else peak,
                          memory if memory is not None else previous.memory_last, peak if peak is not None else previous.peak_last)


def activity_text(window, window_started_at):
    """这一窗口里有没有进展（文件头 ①「卡住」的有进展；judge_stall_stop 用）：进程内存或内存峰值涨了至少 ACTIVITY_MEMORY_BYTES，
    交给人看的那半句；都没有、窗口不是这一个或没记过交空串。"""
    if window is None or window.window_started_at != window_started_at:
        return ""
    for name, first, last in (("进程内存", window.memory_first, window.memory_last), ("内存峰值", window.peak_first, window.peak_last)):
        if first is not None and last is not None and last - first >= ACTIVITY_MEMORY_BYTES:
            return f"{name}这一窗口从 {first >> 20} MiB 涨到 {last >> 20} MiB"
    return ""


def descendant_processes(root_process_id, directory="/proc"):
    """root 的子孙进程号（照 <directory>/<号>/stat 的父进程号一层层往下，不按名字找；读不到的那几个跳过）。directory 只在自证里换（TASK_SERVER_SELFTEST_PROC_ROOT）。"""
    children = {}
    for name in os.listdir(directory):
        if not name.isdigit():
            continue
        try:
            with open(os.path.join(directory, name, "stat"), encoding="utf-8", errors="replace") as handle:
                parent = int(handle.read().rsplit(")", 1)[1].split()[1])
        except (OSError, IndexError, ValueError):
            continue
        children.setdefault(parent, []).append(int(name))
    found, frontier = set(), [root_process_id]
    while frontier:
        for child in children.get(frontier.pop(), []):
            if child not in found:
                found.add(child)
                frontier.append(child)
    return found


def process_is_zombie(process_id):
    """这个进程号已退出、只剩等父进程收的僵尸（/proc/<号>/stat 的状态是 Z）；读不到交 False（调用方另看 /proc/<号> 在不在）。"""
    try:
        with open(f"/proc/{process_id}/stat", encoding="utf-8", errors="replace") as handle:
            return handle.read().rsplit(")", 1)[1].split()[0] == "Z"
    except (OSError, IndexError):
        return False


def process_run_state(process_id, directory="/proc"):
    """<directory>/<号>/stat 的状态字段（R、S、D、Z 这类一个字母；进程名里有右括号也照最后一个右括号切）；读不到交空串。"""
    try:
        with open(os.path.join(directory, str(process_id), "stat"), encoding="utf-8", errors="replace") as handle:
            return handle.read().rsplit(")", 1)[1].split()[0]
    except (OSError, IndexError):
        return ""


def io_pressure_text(path=IO_PRESSURE_PATH):
    """本机此刻 /proc/pressure/io 的 some avg60（「some avg60=12.34」）；文件不在、读不了、认不出交「/proc/pressure/io 读不到」。"""
    try:
        with open(path, encoding="utf-8") as handle:
            for line in handle:
                found = re.match(r"^some .*\bavg60=([0-9.]+)", line)
                if found:
                    return f"some avg60={found.group(1)}"
    except OSError:
        pass
    return f"{path} 读不到"


def shell_script_word_indexes(command):
    """命令里 shell 的 -c 脚本那一个词的下标（find_outside_shared_path 照 shell 词读它）：命令词名（basename）在 SHELL_COMMAND_NAMES 里的词后面，
    选项（- 或 + 起头）里有一个带 c 的短选项串（-c、-ec、-lc）的，选项（与收尾的 --）之后头一个词；SHELL_OPTIONS_WITH_VALUE 后面那个词是选项的值，跳过。
    套在包装里的（env、timeout、临时 prune-server 的 -- 后面）照认：每个命令词名对得上的词都往后看。"""
    indexes = set()
    for index, word in enumerate(command):
        if os.path.basename(word) not in SHELL_COMMAND_NAMES:
            continue
        cursor, script = index + 1, False
        while cursor < len(command) and command[cursor][:1] in ("-", "+") and command[cursor] != "--":
            option = command[cursor]
            if option in SHELL_OPTIONS_WITH_VALUE:
                cursor += 1
            elif not option.startswith("--") and "c" in option[1:]:
                script = True
            cursor += 1
        if cursor < len(command) and command[cursor] == "--":
            cursor += 1
        if script and cursor < len(command):
            indexes.add(cursor)
    return indexes


def shell_word_paths(script, pattern=SHELL_WORD_PATH):
    """shell 脚本（-c 那一个词）里的绝对路径（find_outside_shared_path）：照 shlex（posix）切成 shell 词，每个词里照 SHELL_WORD_PATH 取：以 / 起头的整个词、
    「名字=绝对路径」「--选项=绝对路径」的值，与挨着 ;|&<>( 写、没隔空白的下一个词（cmd;/x、>/x、(/x）。引号里的整段是一个词（"（接的是 /）" 里的 / 跟在空白后面，不算），
    词中间夹着的 / 不算；${X:?}/…、$(pwd)/… 这类展开之后的尾巴不算（弄坏开关 expansion-tail-as-path 下调用方给 pattern=SHELL_WORD_PATH_WITH_EXPANSION_TAILS，尾巴也取）。
    # 不当注释（注释里写的路径照认）。切不开的（引号不配对）照 QUOTED_WORD_PATH 按子串认：多拒不漏。"""
    try:
        lexer = shlex.shlex(script, posix=True)
        lexer.whitespace_split = True
        lexer.commenters = ""
        words = list(lexer)
    except ValueError:
        return QUOTED_WORD_PATH.findall(script)
    return [path for word in words for path in pattern.findall(word)]


# ── 单元 ──
def unit_states(unit, environment):
    """systemctl --user show 读的 (LoadState, ActiveState)（照输出判，不看退出码）；查不了交 None。
    只在自证里：TASK_SERVER_SELFTEST_WORLD 下有 UNIT_STATES_UNREADABLE_FILE 时交 None（unit-unreadable- 起头的那四格演一直读不到）。"""
    world = lib.selftest_override("TASK_SERVER_SELFTEST_WORLD", environment)
    if world and os.path.exists(os.path.join(world, UNIT_STATES_UNREADABLE_FILE)):
        return None
    try:
        completed = subprocess.run(["systemctl", "--user", "show", "-p", "LoadState", "-p", "ActiveState", unit], capture_output=True, text=True,
                                   env=environment, stdin=subprocess.DEVNULL, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        return None
    values = dict(line.split("=", 1) for line in completed.stdout.splitlines() if "=" in line)
    if "ActiveState" not in values:
        return None
    return values.get("LoadState", ""), values["ActiveState"]


def unit_processes(unit, environment):
    """单元 cgroup 里此刻的进程号（读 cgroup.procs 的内容）；单元已经没了交空表。"""
    try:
        completed = subprocess.run(["systemctl", "--user", "show", "-p", "ControlGroup", "--value", unit], capture_output=True, text=True,
                                   env=environment, stdin=subprocess.DEVNULL, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        return None
    if completed.returncode:
        states = unit_states(unit, environment)
        return [] if states is not None and states[0] == "not-found" else None
    group = completed.stdout.strip()
    if not group:
        return []
    return cgroup_processes("/sys/fs/cgroup" + group)


def recorded_text(path, what):
    """读尝试目录里记一行字的小文件（起跑标记 started、attempt_runner 写的 scope 文件；文件头 ①「读不到单元状态」）→ (去掉首尾空白的内容, "")；
    文件不在而尝试目录在交 ("", "")（还没写过），读不了、尝试目录也不在交 ("", 那一句)。"""
    try:
        with open(path, encoding="utf-8") as handle:
            return handle.read().strip(), ""
    except FileNotFoundError:
        if os.path.isdir(os.path.dirname(path)):
            return "", ""
        return "", f"{what} {path} 不在，尝试目录也不在，核不出"
    except OSError as error:
        return "", f"{what} {path} 读不了（{type(error).__name__}: {error}），核不出"


def cgroup_processes(group_path):
    """一个 cgroup 目录此刻的进程号（读 cgroup.procs）：目录已经没了交空表，读不了交 None。"""
    try:
        with open(os.path.join(group_path, "cgroup.procs"), encoding="utf-8") as handle:
            return handle.read().split()
    except FileNotFoundError:
        return None if os.path.isdir(group_path) else []
    except OSError:
        return None


def registered_threads(environment):
    """每台默认重活线程数（task_server_lib.py 文件头「线程」）；取不到抛 Unavailable（不退回按核数算）。"""
    heavy_threads = lib.load_module("heavy_threads", os.path.join(lib.HOSTS_DIRECTORY, "heavy_threads.py"))
    try:
        return int(heavy_threads.default_heavy_threads(environment))
    except (heavy_threads.HeavyThreadDefaultError, ValueError) as error:
        raise lib.Unavailable(f"取不到每台默认重活线程数（{error}）", "修好 .claude/heavy-thread-default（或 unset 写坏的 SINGLEFS_THREAD_CAP）再起服务") from None


class Scheduler:
    def __init__(self, configuration, store, environment, threads, nfs_report):
        self.configuration = configuration
        self.store = store
        self.environment = environment
        self.threads = threads
        self.host_dispatch = lib.load_module("host_dispatch", os.path.join(lib.HOSTS_DIRECTORY, "host-dispatch.py"))
        self.peaks = lib.load_module("memory_peaks_lib", os.path.join(lib.HOSTS_DIRECTORY, "memory_peaks_lib.py"))
        self.project_code_runs = lib.load_module("lib_project_code_runs", os.path.join(lib.HOOKS_DIRECTORY, "lib_project_code_runs.py"))
        self.cargo_lock = lib.load_module("lib_workspace_cargo_lock", os.path.join(lib.HOOKS_DIRECTORY, "lib_workspace_cargo_lock.py"))
        self.build_directories = lib.load_module("run_build_directory", os.path.join(lib.HOSTS_DIRECTORY, "run_build_directory.py"))
        self.resources = Resources(environment, self.host_dispatch, configuration.build_directory, configuration.path)
        self.machines = {LOCAL: LocalMachine(self), PEER: PeerMachine(self)}   # 两台各一份「这一台怎么读、怎么起」（文件头 ②「两台同一段逻辑」）
        self.resources.machines = self.machines
        speedy = lib.fast(environment)
        self.tick_seconds = FAST_TICK_SECONDS if speedy else TICK_SECONDS
        override = lib.selftest_override("TASK_SERVER_TICK_SECONDS", environment)   # 自证量「30 秒一轮也不慢」那一格用
        if override:
            self.tick_seconds = float(override)
        self.finished_listeners = []   # 记下结局时叫醒在等的 wait（文件头「通知」）
        self.finishing = set()   # 收尾此刻在后台线程里做着的尝试记录 id（finish_off_loop）
        self.finishing_lock = threading.Lock()
        self.unit_check_seconds = FAST_UNIT_CHECK_SECONDS if speedy else UNIT_CHECK_SECONDS
        self.lost_grace_seconds = FAST_LOST_GRACE_SECONDS if speedy else LOST_GRACE_SECONDS
        self.unit_unreadable_attention_seconds = FAST_UNIT_UNREADABLE_ATTENTION_SECONDS if speedy else UNIT_UNREADABLE_ATTENTION_SECONDS
        self.cancel_settle_seconds = self.lost_grace_seconds + 2 * self.unit_check_seconds + CANCEL_SETTLE_MARGIN_SECONDS
        self.alerted_child_refusals = set()   # children_refused 已告警过的 (父任务号, 那一句)
        self.start_uncertain_grace_seconds = FAST_START_UNCERTAIN_GRACE_SECONDS if speedy else START_UNCERTAIN_GRACE_SECONDS
        self.scene_keep_seconds = FAST_SCENE_KEEP_AFTER_COLLECT_SECONDS if speedy else SCENE_KEEP_AFTER_COLLECT_SECONDS
        self.build_disk_predictions = {}   # {任务号: 编译目录占盘的预测（字节）}：与内存预测一样每条在排任务只算一次（文件头 ②「编译目录占盘」）
        self.admission_threads_cache = {}   # {任务号: (线程数, 照什么记的)}：CPU 闸里这一条自己要起的记几线程（admission_threads），每条在排任务只算一次
        self.running_threads_cache = {}     # {尝试 id: (线程数, 照什么记的)}：在跑却还没实测到核数的尝试在 CPU 闸里记几线程（unmeasured_threads），每次尝试只算一次
        # 记下结局时删编译目录（remove_finished_build_directories）与取消之后删产物（remove_cancelled_artifacts）共用：同一份目录不两边一齐删（文件头「接口」的取消）
        self.build_removal_lock = threading.RLock()
        self.starved_seconds = FAST_STARVED_SECONDS if speedy else STARVED_ROUNDS * self.tick_seconds
        self.aged_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_AGED_SECONDS", environment) or AGED_WAIT_SECONDS)
        # slice 兜底的下限（slice_floor_problem）：自证的小世界由 lib.SLICE_ROOM_FLOOR_VARIABLE 换（各格写 0：不判；那几格写数照它判）
        floor_override = lib.selftest_override(lib.SLICE_ROOM_FLOOR_VARIABLE, environment)
        self.slice_room_floor = int(floor_override) if floor_override.isdigit() else SLICE_ROOM_FLOOR_BYTES
        self.placement_paused = lib.selftest_override("TASK_SERVER_SELFTEST_PLACEMENT_PAUSED", environment)
        self.placement_refused = lib.selftest_override("TASK_SERVER_SELFTEST_PLACEMENT_REFUSED", environment)
        self.progress_seconds = FAST_PROGRESS_STALL_SECONDS if speedy else PROGRESS_STALL_SECONDS
        self.stall_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_STALL_SECONDS", environment) or STALL_SECONDS)
        self.stall_grace_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS", environment) or STALL_GRACE_SECONDS)
        # {尝试号: (日志字节数, 上一次看到它变的时刻, 这一个 CPU 窗口开头的时刻, 那时的 CPU, 上一个窗口满了为什么没停：SPARED_* 之一，没判过或日志涨了是空串)}（判卡住）
        self.log_growth = {}
        self.log_offsets = {}   # 第二台的尝试：{尝试号: (日志读到哪, 算在涨的字节数, 最近的 usage_usec)}
        self.hard_limit_stops = set()   # 照四小时硬上限判停、正在停的尝试号（stop_over_limit_attempt）
        self.peer_memory = {}   # 第二台的尝试：{尝试号: (最近一行 MULTI_HOST_RUN_PEER_MEMORY 的此刻内存或 None, 读到过的峰值或 None)}（PeerMachine.memory_now）
        self.peer_memory_stat = {}   # 第二台的尝试：{尝试号: MULTI_HOST_RUN_PEER_MEMORY_STAT 里 current_bytes 最大的那一行 {current, anon, file, file_dirty, file_writeback, shmem}}（文件头 ①「内存最高那一刻」）
        self.peer_external_kill = {}   # 第二台的尝试：{尝试号: (来源, 被杀的进程号, 日志原行)}，multi-host-run.sh 命令退非 0 之后打的 MULTI_HOST_RUN_PEER_EXTERNAL_KILL 那一行（external_kill_note 读）
        self.peer_io_wait = {}   # 第二台的尝试：{尝试号: PeerMachine.io_blocked 上一次读时并起来的几行 MULTI_HOST_RUN_PEER_IO_WAIT 里最少的 blocked，有一行 read=no 是 None}
        self.peer_threads = {}   # 第二台的尝试：{尝试号: PeerMachine.thread_reading 上一次读时并起来的几行 MULTI_HOST_RUN_PEER_THREADS 里 read=yes 的取最大的 ThreadReading，都是 read=no 是 None}
        # 第二台的尝试：{尝试号: [行…]}，log_content 每一回读到的 MULTI_HOST_RUN_PEER_IO_WAIT（blocked 或 None）与 MULTI_HOST_RUN_PEER_THREADS（ThreadReading 或 None）
        # 接在后面，PeerMachine.io_blocked、thread_reading 读时并起来、清空：两次读之间不论 log_content 被调几回、哪个线程调的，读到的行一行不丢
        self.peer_io_rounds = {}
        self.peer_thread_rounds = {}
        self.thread_overruns = {}   # 线程总数远超登记数（note_threads）：{尝试号: {sample: 上一轮的采样号, streak: 连着超了几轮, flagged: 判出那一轮的 (ThreadReading, 登记数) 或 None}}
        self.io_waits = {}   # 判卡住：{尝试号: IoWaitWindow}（文件头 ①「卡在 IO 上」；日志涨了就删）
        self.activity_windows = {}   # 判卡住：{尝试号: ActivityWindow}（文件头 ①「卡住」的有进展；窗口换了从头记）
        # 日志不涨时的进展读数（note_attempt_progress；文件头 ①「要处置的提醒」的 quiet-log-busy）：上一回读的时刻（monotonic）、上一回读到的写着的文件与大小、
        # 最近一回读到进展的 (时刻（monotonic）, 给人看的那半句)
        self.progress_read_at = {}
        self.progress_files = {}
        self.progress_noted = {}
        self.service_stopped_attempts = {}   # {尝试号: stop_running 对它发停的时刻}：收尾时 signal_note 照它说服务发没发过停的信号（finish_attempt 删）
        self.proc_root = lib.selftest_override("TASK_SERVER_SELFTEST_PROC_ROOT", environment) or "/proc"   # 本机读进程状态的目录（只在自证里换成假的）
        self.cpu_windows = {}   # 判疑似卡住：{尝试号: SuspectWindow}（文件头 ①「疑似卡住」）
        self.suspect_reported = {}   # {尝试号: 报给提交它的会话的那一段的段号}：一段只报一次
        self.suspect_lock = threading.Lock()
        # 要处置的提醒（文件头 ①「要处置的提醒」）：提醒门槛的起点；报到的门槛记进库里任务的行、内存里另留一份（reminded_thresholds）；分片父任务推不动的现状由 drive_sharded_parents 记
        self.queue_attention_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS", environment) or QUEUE_ATTENTION_MINUTES * 60)
        self.quiet_attention_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_QUIET_ATTENTION_SECONDS", environment) or QUIET_ATTENTION_MINUTES * 60)
        self.io_wait_attention_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_IO_WAIT_ATTENTION_SECONDS", environment) or IO_WAIT_ATTENTION_MINUTES * 60)
        self.long_run_attention_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_LONG_RUN_ATTENTION_SECONDS", environment) or LONG_RUN_ATTENTION_SECONDS)
        # {任务号: oversized-budget 那一项}：排队那一刻判出记账不小于某一台整片界的（budget_check），attention_of_record 照它报；收尾时删（forget_prediction），
        # 只在内存里：服务重启后下一轮排队照现算的预测再判一次，报到的门槛在库里（reminded_thresholds），不重发
        self.budget_notices = {}
        self.reminded_thresholds = {}   # {(任务号, 种类): 报给提交它的会话的最高提醒门槛}：同一种同一道门槛只报一次；库里那一份（任务行的 reminded_thresholds）读过的也并进来
        self.attention_held = set()    # {(任务号, 种类, 提醒门槛)}：说的是子任务、发消息那一路头一轮先压着的提醒（push_attention；发了或发失败就删）
        self.reminder_histories = {}  # {(任务号, 种类): [{threshold, repush, at, basis, round}…]}：报过的那几回（库里任务行的 reminder_history 另存一份；reminder_history_of 读）
        self.workspace_write_checks = {}   # {任务号: workspace-write-interrupted 那一项或 None}：结了局的改工作区命令只核一次（workspace_write_item）
        self.attention_lock = threading.Lock()
        # 这两张另记进库（RESTART_SURVIVING_STATES 的 shard_advance），换服务后照库接着数（restore_restart_surviving_state）
        self.advance_failures = {}   # {分片父任务号: {"episode": 第几回开始推不动, "why": 最近一次的错, "count": 这一回连着推不动几轮}}：推通了就删
        self.advance_episodes = {}   # {分片父任务号: 推不动开始过几回}：推通之后再推不动是新的一回（提醒门槛加一）
        self.journalctl = lib.selftest_override("TASK_SERVER_JOURNALCTL", environment) or "journalctl"   # 读本机的 earlyoom 与内核日志（文件头 ①「被外部信号杀掉」）
        self.pass_state = None   # 这一轮排队留下的（看进展读它的余量；弄坏开关下另判排头也复用：在排的、起了或收尾了的号、这一轮的余量）
        self.head_verdict = None   # 派活这一轮给排头的结论（HeadVerdict；队空、这一轮没排是 None）：/health 判「判能起却没起成」只读它
        self.starved_text = ""   # 排队线程每一轮算好的「派活判排头能起却一直没起成」（/health 只读它，文件头「接口」）
        self.progress_text = ""   # 排队线程每一轮算好的看进展那一句（progress_problem；/health 只读它）
        self.progress_since = {}   # 看进展：{台: 这台从哪一刻起照公式放得下排着的最小那条、又一条都没起}
        self.progress_attempt_seen = None   # 看进展：上一轮数到的最大尝试号（新起的尝试照号认，起了又收尾的也数得到）
        self.pause = store.pause_record()   # 暂停态（文件头 ⑤「暂停」）：库里记的那一份，起来时读；排队线程与接口读这一份（None 是没暂停），pause_service、resume_service 改库同时改它
        self.start_gate = threading.Lock()  # 起尝试与 pause_service 共用：持锁判没暂停才起，pause 返回之后一条新的都不起
        self.kept_allow_session = ""   # 只在弄坏开关 resume-keeps-allowed-session 下用：上一次 resume 时放行的会话
        self.predictions = {}   # {任务号: (算它时的 next_cap, 预测)}：每条在排任务的内存预测只算一次（文件头 ②「预测每条只算一次」）
        self.pin_unreadable_reasons = {}   # {任务号: 钉值读不到的那一句}：pinned_prediction 记、place 照它记 run-condition-unmet（文件头 ②「内存」的钉值）
        self.prediction_lock = threading.Lock()   # 管 predictions 这几张存预测的表：只在取、存、删的那一下拿着，现算（扫库、读钉值）不在这把锁里
        self.prediction_task_locks = {}   # 每条任务一把锁（keyed_lock 管，放下就删）：排队线程、确认单与弄坏开关 health-judges-head 下 /health 的另判排头同一条不算两遍，别的任务不等它算完
        self.prediction_generation = 0   # 存的预测整表作废（钉值改了）一回涨一：作废之前开算的那一份算完不存（store_prediction）
        self.view_numbers = itertools.count(1)   # PassView 的编号（自证的读数记录里分各轮）
        world = lib.selftest_override("TASK_SERVER_SELFTEST_WORLD", environment)
        self.reads_log = os.path.join(world, "scheduler-reads") if world else ""   # 只在自证里：排队读预测与余量的每一次记一行（selftest_read）
        self.reads_lock = threading.Lock()
        self.repass_soon = False   # 第二台这一轮起满了 PEER_STARTS_PER_PASS 条：主循环隔 PEER_REPASS_SECONDS 再排
        self.waiting_reasons = {}   # 上一轮判的每条排着的任务在等什么：{任务号: 一句}（status 打出来）
        self.queue_positions = {}   # 上一轮排队的先后：{任务号: (排第几, 在排共几条, 前面排满时限的几条)}（status 打出来；全局的，不分会话）
        self.fits_since = {}   # 派活从哪一刻起一直判排头能起却没起成：{任务号: 时刻}（/health 判「判能起却没起成」）
        self.memory_wrapper = lib.selftest_override("TASK_SERVER_MEMORY_WRAPPER", environment) or lib.MEMORY_WRAPPER
        self.peer_runner = lib.selftest_override("TASK_SERVER_PEER_RUNNER", environment) or lib.PEER_RUNNER
        self.systemd_run = lib.selftest_override("TASK_SERVER_SYSTEMD_RUN", environment) or "systemd-run"
        self.peer_root = lib.selftest_override("TASK_SERVER_FAKE_REPOSITORY_ROOT", environment) or lib.REPOSITORY_ROOT
        # 工作根与共享数据根（task_server_lib.py 文件头「资源记录」后的「共享根」）：照 .claude/hooks/lib_work_root.py 取（不另算、不补缺省），交给每次尝试；
        # serve 在建 Scheduler 之前已判过取得到（取不到退 78）
        self.work_root = lib.work_root()
        self.scenes_root = os.path.join(self.work_root, lib.SCENES_ROOT_NAME)   # 判红那一趟的现场（task_server_lib.py 文件头「现场」；serve 起来时建好）
        self.data_root = lib.shared_data_root(environment)
        self.boundary = int(lib.selftest_override("TASK_SERVER_SESSION_BOUNDARY_PID", environment) or 1)
        self.wake_event = threading.Event()
        self.stopping = threading.Event()
        self.last_tick = time.time()
        self.unit_checked_at = {}
        self.unit_gone_since = {}
        # 读不到单元状态的那一段（文件头 ①「读不到单元状态」；unit_unreadable_gone 记，读到了单元状态、收尾时 forget_unit_unreadable 清）
        self.unit_unreadable = {}   # {尝试记录 id: {since: 从哪一刻起（monotonic）连着读不到, task, number, host, unit}}（提醒与 /health 读它）
        self.unit_unreadable_noted = {}   # {尝试记录 id: 这一段已往服务日志打到的最高提醒门槛}
        self.unit_presence = {}   # {尝试记录 id: (lib.PRESENCE_*, 说明, 核完的时刻 monotonic)}：不经 systemd 最近一回核到的（probe_unit_presence 写）
        self.unit_presence_probing = set()   # 此刻在后台核着的尝试记录 id
        self.unit_presence_lock = threading.Lock()
        self.children = []
        self.thread = threading.Thread(target=self.loop, name="task-server-scheduler", daemon=True)
        # NFS 共享（task_server_lib.py 文件头「NFS 共享」）：起来时判过的那一份，前三项由 nfs_loop 每 nfs_recheck_seconds 复核；/health 的 nfs 一栏读它
        self.nfs = nfs_report
        self.nfs_slow_last = None   # 最近一回复查期里连得上的「慢」（{at, slow: [那几句]}）：nfs_loop 记，/health 的 nfs 一栏带 slow_last；只进服务日志与 /health，不推
        self.nfs_lock = threading.Lock()
        self.shared_roots = [os.path.realpath(entry["root"]) for entry in nfs_report.get("roots", [])]   # 第二台照同一绝对路径读写的目录（提交时拒的 ① ②）
        self.mount_problem = ""   # 复核判出第二台没挂上共享根的那一句：这时当第二台此刻用不了（peer_view）
        self.nfs_recheck_seconds = FAST_NFS_RECHECK_SECONDS if speedy else NFS_RECHECK_SECONDS
        self.nfs_thread = threading.Thread(target=self.nfs_loop, name="task-server-nfs-recheck", daemon=True)
        # 第二台用不用得了（note_peer）：/health 的 peer 一栏读它；peer_alerted 是上一条 peer-unavailable 告警写的原因（同一个原因不再写）
        self.peer_status = {"available": True, "why": "", "since": None, "last_run_failure": None}
        self.peer_alerted = ""
        self.peer_status_lock = threading.Lock()
        # 发消息给会话（文件头 ⑥）：一个后台线程；结局记下时 notify_finished 把任务号放进 inbox_pending、叫醒它
        self.inbox_round_seconds = FAST_INBOX_ROUND_SECONDS if speedy else INBOX_ROUND_SECONDS
        self.peer_unavailable_notify_seconds = FAST_PEER_UNAVAILABLE_NOTIFY_SECONDS if speedy else PEER_UNAVAILABLE_NOTIFY_SECONDS
        self.unclaimed_warning_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_UNCLAIMED_WARNING_SECONDS", environment) or UNCLAIMED_WARNING_MINUTES * 60)
        self.confirm_timeout_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_CONFIRM_TIMEOUT_SECONDS", environment) or CONFIRM_TIMEOUT_SECONDS)
        self.run_limit_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS", environment) or RUN_LIMIT_SECONDS)
        self.run_hard_limit_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_RUN_HARD_LIMIT_SECONDS", environment) or RUN_HARD_LIMIT_SECONDS)
        self.anomaly_repush_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS", environment) or ANOMALY_REPUSH_SECONDS)
        self.progress_push_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_PROGRESS_PUSH_SECONDS", environment) or PROGRESS_PUSH_MINUTES * 60)
        self.progress_pushed = {}   # {任务号: 上一回发在跑进度的时刻}（push_progress；任务行的 progress_pushed_at 另存一份，服务重启后取两份里晚的）
        self.unclaimed_repushed = {}   # {任务号: 结局没人取再推到第几回}（repush_unclaimed_outcomes；内存里一份，任务行的 outcome_repushed 另存一份，服务重启后取两份大的、不重推同一回）
        self.inbox_sockets = {}        # 这一轮发消息照会话号找到的收件 socket：{会话号: (路径, 找不到时的那一句)}（inbox_socket_of；每一轮开头清掉）
        self.inbox_pending = set()   # 记下了结局、还没发给会话的任务号
        self.inbox_failed = set()    # 发不出去过的事件（("outcome", 号)、(号, 种类, 提醒门槛)、("suspect", 尝试号, 段号)）：不重试到死，写过告警就不再发
        self.resident_prune_seconds = FAST_RESIDENT_PRUNE_SECONDS if speedy else RESIDENT_PRUNE_SECONDS
        self.resident_pruned_at = 0.0   # forget_settled_entries 上一回清的时刻（time.time）
        self.session_locks = {}   # session_lock 的锁表：{会话号: [锁, 拿着与在等的个数]}
        self.session_locks_guard = threading.Lock()
        self.tamper_lock = threading.RLock()   # tampered：同一行几个线程同时判出核不过，只头一个告警、写回
        self.health_notified = {}    # {服务健康的种类: 已发过的会话}：同一样只发一次，好了（这一轮不再查出）就删，再坏再发
        self.inbox_lock = threading.Lock()
        self.inbox_event = threading.Event()
        self.inbox_thread = threading.Thread(target=self.inbox_loop, name="task-server-inbox", daemon=True)
        self.runaway_scan_seconds = FAST_RUNAWAY_SCAN_SECONDS if speedy else RUNAWAY_SCAN_SECONDS
        # {(台, 进程号): {reported_at, count, alerted, stopped}}：任务之外的进程空转（watch_runaway_processes），这一轮不在了的忘掉；
        # 另记进库（RESTART_SURVIVING_STATES），换服务后照库接着数
        self.runaway_notified = {}
        self.runaway_renotify_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_RUNAWAY_RENOTIFY_SECONDS", environment) or RUNAWAY_RENOTIFY_SECONDS)
        self.class_durations = (0.0, {})   # (算的时刻, {类: [跑成的用时…]})：class_average_seconds 的缓存，只有排队线程读写
        self.running_peak_samples = {}   # {尝试 id: {first_seen, samples: [(时刻, 累计 CPU 微秒)…]}}：峰值段核数的读数（note_peak_segment），窗口 CPU_MEASURED_AFTER_SECONDS、起点之前留最新的一条
        self.running_peak_cores = {}     # {尝试 id: 到此刻各段在用核数的最大}：note_peak_segment 记，finish_attempt 记 ended 时删
        self.running_latest_cores = {}     # {尝试 id: 最近量满的那一段的在用核数}：note_peak_segment 记，latest_running_cores 读（CPU 闸里在跑的那一条照此刻的记，不照到此刻的最大）
        self.running_latest_runnable = {}  # {尝试 id: 最近一回读到的可跑线程数}：note_threads 记，latest_running_cores 读
        self.running_peak_runnable = {}  # {尝试 id: 到此刻读到过的可跑线程数的最大}：note_threads 记，peak_segment_running_cores、finish_cpu_gate_threads 读，finish_attempt 记 ended 时删
        self.running_usage_lock = threading.Lock()
        # 记账对账（audit_bookings，booking_audit_loop 每 booking_audit_seconds 一轮）：booking_audit_state 记每一项连着几轮不对，booking_drift 是判出来的（/health 的 booking_drift 一栏读），
        # booking_drift_notified 记每一件推给每个会话推到了哪道提醒门槛；三张表都由 booking_drift_lock 管
        self.booking_audit_seconds = BOOKING_AUDIT_SECONDS
        self.booking_audit_state = {}
        self.booking_drift = {"memory": {}}
        self.booking_drift_notified = {}
        self.booking_drift_lock = threading.Lock()
        self.booking_ledger = {}   # {台: 在跑记账的逐条明细}：记账对账每一轮换上（refresh_booking_ledger），/health 的 booking_ledger 一栏读（booking_drift_lock 管）
        self.booking_audit_thread = threading.Thread(target=self.booking_audit_loop, name="task-server-booking-audit", daemon=True)
        self.unit_stopped = set()   # 单元已确认停了（release_stopped_booking）而收尾还没做完的尝试号：booking_attempts 不数它们（CPU 与内存的在跑记账）
        self.runaway_thread = threading.Thread(target=self.runaway_loop, name="task-server-runaway", daemon=True)
        # 第二台的余量（Resources.peer_probe_loop；文件头 ②「第二台的余量不在排队线程里读」）：自证的假余量不连第二台，不起它
        self.peer_probe_thread = threading.Thread(target=self.resources.peer_probe_loop, args=(self.stopping,), name="task-server-peer-probe", daemon=True)
        self.outer_unit = self.nested_outer_unit()   # 套在外层单元里的服务（文件头 ③）：尝试单元照它 BindsTo；真服务是空串
        self.restart_surviving_lock = threading.Lock()   # 换服务不丢的那几份：取内存表、比上次写的、写库在同一把锁里，后改的不被先取的那一份盖掉
        self.restart_surviving_written = {}   # {库里的名字: 上次写进库的正文（canonical_json）}：没变的不重写
        self.restore_restart_surviving_state()

    # ── 记录 ──
    def operation(self, action, task_id, session, process_id=0, note=""):
        if lib.broken("operations-unlogged", self.environment):
            return
        record = {"time": round(time.time(), 3), "action": action, "task": task_id, "session": session, "pid": process_id, "note": note}
        self.store.insert("operations", dict(record))
        with contextlib.suppress(OSError):
            append_private_line(self.configuration.operations_log, lib.canonical_json(record))

    def alert(self, kind, task_id, line, next_step):
        record = {"time": round(time.time(), 3), "kind": kind, "task": task_id, "line": line, "next_step": next_step}
        self.store.insert("alerts", dict(record))
        with contextlib.suppress(OSError):
            append_private_line(self.configuration.alerts_log, lib.canonical_json(record))
        say(f"  ✗ 告警（{kind}）任务 {task_id}：{line}")
        say(f"     → 怎么办：{next_step}")

    # ── 换服务不丢的调度判断（RESTART_SURVIVING_STATES）：内存表照旧是判的那一份，每次改了整份记进库（Store.write_restart_surviving_state），服务起来时读回 ──
    def restart_surviving_record(self, name):
        """这一份内存表写进库的正文：键是元组与整数的摊成按键排好的列表，没变的两次取出来逐字相同。"""
        if name == "runaway_notified":
            return [[host, pid, dict(entry)] for (host, pid), entry in sorted(list(self.runaway_notified.items()))]
        if name == "shard_advance":
            with self.attention_lock:
                return {"failures": [[parent_id, dict(failure)] for parent_id, failure in sorted(self.advance_failures.items())],
                        "episodes": [[parent_id, episode] for parent_id, episode in sorted(self.advance_episodes.items())]}
        if name == "suspect_reported":
            with self.suspect_lock:
                return [[attempt_id, segment] for attempt_id, segment in sorted(self.suspect_reported.items())]
        if name == "health_notified":
            return [[kind, sorted(sessions)] for kind, sessions in sorted(list(self.health_notified.items()))]
        if name == "booking_drift_notified":
            with self.booking_drift_lock:
                return [[key[0], key[1], session, crossed] for (key, session), crossed in sorted(self.booking_drift_notified.items(), key=lambda entry: (entry[0][0][0], str(entry[0][0][1]), entry[0][1]))]
        raise AssertionError(f"RESTART_SURVIVING_STATES 里的 {name} 在这里没有一支：新加一份要在这里与 adopt_restart_surviving_record 各写一支")

    def adopt_restart_surviving_record(self, name, state):
        """库里读回的这一份并进内存表（restore_restart_surviving_state 调）：写法不对抛 ValueError，内存表一项都不动。"""
        def number(value):
            return isinstance(value, (int, float)) and not isinstance(value, bool)

        def whole(value):
            return isinstance(value, int) and not isinstance(value, bool)

        if name == "runaway_notified":
            if not isinstance(state, list):
                raise ValueError("空转进程的账应当是列表")
            restored = {}
            for row in state:
                if not (isinstance(row, list) and len(row) == 3 and row[0] in MACHINE_ORDER and whole(row[1]) and isinstance(row[2], dict)
                        and number(row[2].get("reported_at")) and whole(row[2].get("count")) and isinstance(row[2].get("alerted"), bool)):
                    raise ValueError(f"空转进程的账应当每行是 [台, 进程号, {{reported_at, count, alerted…}}]，有一行是 {str(row)[:120]}")
                restored[(row[0], row[1])] = row[2]
            self.runaway_notified.update(restored)
            return
        if name == "shard_advance":
            failures, episodes = (state.get("failures"), state.get("episodes")) if isinstance(state, dict) else (None, None)
            if not (isinstance(failures, list) and isinstance(episodes, list)):
                raise ValueError("分片父任务推不动的账应当是 {failures, episodes}")
            restored_failures, restored_episodes = {}, {}
            for row in failures:
                if not (isinstance(row, list) and len(row) == 2 and whole(row[0]) and isinstance(row[1], dict)
                        and whole(row[1].get("episode")) and whole(row[1].get("count")) and isinstance(row[1].get("why"), str)):
                    raise ValueError(f"推不动的这一回应当每行是 [父任务号, {{episode, why, count}}]，有一行是 {str(row)[:120]}")
                restored_failures[row[0]] = row[1]
            for row in episodes:
                if not (isinstance(row, list) and len(row) == 2 and whole(row[0]) and whole(row[1])):
                    raise ValueError(f"推不动开始过几回应当每行是 [父任务号, 回数]，有一行是 {str(row)[:120]}")
                restored_episodes[row[0]] = row[1]
            with self.attention_lock:
                self.advance_failures.update(restored_failures)
                self.advance_episodes.update(restored_episodes)
            return
        if name == "suspect_reported":
            if not (isinstance(state, list) and all(isinstance(row, list) and len(row) == 2 and whole(row[0]) and (row[1] is None or whole(row[1])) for row in state)):
                raise ValueError("疑似卡住报过的段应当每行是 [尝试号, 段号]")
            with self.suspect_lock:
                self.suspect_reported.update({row[0]: row[1] for row in state})
            return
        if name == "health_notified":
            if not (isinstance(state, list) and all(isinstance(row, list) and len(row) == 2 and isinstance(row[0], str) and isinstance(row[1], list)
                                                    and all(isinstance(session, str) for session in row[1]) for row in state)):
                raise ValueError("服务健康发过的账应当每行是 [种类, [会话…]]")
            self.health_notified.update({row[0]: set(row[1]) for row in state})
            return
        if name == "booking_drift_notified":
            if not (isinstance(state, list) and all(isinstance(row, list) and len(row) == 4 and row[0] == "memory" and isinstance(row[1], str)
                                                    and isinstance(row[2], str) and whole(row[3]) for row in state)):
                raise ValueError("记账漂移推过的账应当每行是 [memory, 台, 会话, 提醒门槛]")
            with self.booking_drift_lock:
                self.booking_drift_notified.update({((row[0], row[1]), row[2]): row[3] for row in state})
            return
        raise AssertionError(f"RESTART_SURVIVING_STATES 里的 {name} 在这里没有一支：新加一份要在这里与 restart_surviving_record 各写一支")

    def persist_restart_surviving_state(self, name):
        """这一份内存表记进库（改了它的地方改完就调）：与上次写进库的逐字相同就不写；库写不进的只记在内存里、打一行，下一次调再试。
        弄坏开关（RESTART_SURVIVING_STATES 里这一份的只记内存）下不写。调用方不拿着 attention_lock（shard_advance 那一份取内存表时要拿它）。"""
        if lib.broken(RESTART_SURVIVING_STATES[name], self.environment):
            return
        with self.restart_surviving_lock:
            state = self.restart_surviving_record(name)
            text = lib.canonical_json(state)
            if self.restart_surviving_written.get(name) == text:
                return
            try:
                self.store.write_restart_surviving_state(name, state)
            except sqlite3.Error as error:
                say(f"  ! 换服务不丢的那一份 {name} 没记进库（{type(error).__name__}: {error}），只记在内存里：下一次改动再写，换服务之前没写上的会丢")
                return
            self.restart_surviving_written[name] = text

    def restore_restart_surviving_state(self):
        """服务起来时（Scheduler.__init__ 末尾）把库里那几份（RESTART_SURVIVING_STATES）读回内存表：
        空转进程报过几次、停没停（watch_runaway_processes）与分片父任务推不动的这一回、开始过几回（note_advance_failure）接着数。
        签名对不上、读不出或写法不对的那一份不认、写告警 record-tampered，那一份照空的起。弄坏开关（这一份的只记内存）下不读。"""
        for name, switch in RESTART_SURVIVING_STATES.items():
            if lib.broken(switch, self.environment):
                continue
            state, valid = self.store.restart_surviving_state(name)
            problem = "" if valid else "签名对不上或读不出"
            if valid and state is not None:
                try:
                    self.adopt_restart_surviving_record(name, state)
                except ValueError as error:
                    problem = str(error)
            if problem:
                self.alert("record-tampered", 0, f"库里换服务不丢的那一份 {name} 不认（{problem}），这一份照空的起",
                           "查是谁直接写了库（只有 task-server 读写它）；这一份从零起：空转进程与推不动的回数从头数")

    def tampered(self, task_id, record, what):
        """校验不过（文件头「库」）：不起、不认，任务记 record-tampered 收尾，写告警。重签写回的那一行自己核得过，之后读到它不再判出校验不过、不再告警：
        关联列照正文写（Store.write_row），正文的 task_id 照 parent_id 写（association_id：parent_id 不是正整数的写 0，判不出它归哪一件就不挂在任何一件名下）。
        弄坏开关 tampered-rows-left-unrepaired 下正文的 task_id 不动、关联列不写，关联列或 parent_id 对不上的那一行重签之后照样核不过，每读一次告警一次。
        几个线程同时读到同一行核不过（排队那一轮与 status 这类）：拿 tamper_lock 现读，已由别的线程记成 record-tampered、核得过的不再告警、不再写。"""
        unrepaired = lib.broken("tampered-rows-left-unrepaired", self.environment)
        with self.tamper_lock:
            current = self.store.task(task_id)
            if not unrepaired and current is not None and current[1] and current[0].get("outcome") == "record-tampered":
                return
            if not lib.broken("tamper-unalerted", self.environment):
                self.alert("record-tampered", task_id, f"库里任务 {task_id} 的{what}校验不过（绕过服务直接写进库，或写进之后被改过），不起、不认",
                           "查是谁直接写了库（只有 task-server 读写它）；任务照 task-server/task submit 重新提交")
            record = dict(record, id=task_id, state="finished", outcome="record-tampered", exit_code=lib.final_exit_code("record-tampered", None),
                          finished_at=round(time.time(), 3), note=f"{what}校验不过，一次都不起")
            if not unrepaired:
                record["task_id"] = association_id(record.get("parent_id"))
            for name in ("session", "command_key"):
                record[name] = record.get(name) if isinstance(record.get(name), str) else ""
            self.store.update("tasks", record)
        self.forget_prediction(task_id)
        if isinstance(record.get("parent_id"), int) and record["parent_id"] != task_id:
            self.advance_parent(record)

    def selftest_read(self, line):
        """只在自证里：排队读预测与余量的每一次（与 stop_running 停了哪几次尝试）往 TASK_SERVER_SELFTEST_WORLD/scheduler-reads 记一行（task_server_lib.py 文件头「只在自证里认的变量」）。"""
        if not self.reads_log:
            return
        with self.reads_lock, open(self.reads_log, "a", encoding="utf-8") as handle:
            handle.write(line + "\n")

    # ── 暂停（文件头 ⑤「暂停」）──
    def pause_in_force(self):
        """排队线程此刻要照着不起新尝试的暂停态（None 是不用）；弄坏开关 pause-queue-ignored 下排队线程当没暂停。"""
        return None if lib.broken("pause-queue-ignored", self.environment) else self.pause

    def allowed_during_pause(self, task, pause):
        """暂停着，这条任务是不是放行的会话交的（文件头 ⑤「暂停」的放行一段）：放行的会话是空串谁都不是，会话是空串的（人手任务）也不匹配。
        弄坏开关 pause-allow-ignored 下谁都不放行，pause-allow-matches-any-session 下放行了会话就谁都放行。"""
        allowed = pause.get("allow_session", "")
        if lib.broken("pause-allow-ignored", self.environment):
            return False
        if allowed and lib.broken("pause-allow-matches-any-session", self.environment):
            return True
        return bool(allowed) and task.get("session", "") == allowed

    def pause_holds(self, task):
        """这条任务此刻要不要留在队里不起：排队线程要照着暂停、而它又不是放行的会话交的 → 那份暂停态，不然 None。"""
        pause = self.pause_in_force()
        if pause is None or self.allowed_during_pause(task, pause):
            return None
        return pause

    def running_outside_allowed(self, pause, running):
        """暂停着在跑的任务里不是放行会话的有几个（等它为 0 才做空档的活）；没放行谁，在跑的都是别的会话的。弄坏开关 pause-other-running-miscounted 下把放行会话的也数进去。"""
        allowed = pause.get("allow_session", "")
        if not allowed or lib.broken("pause-other-running-miscounted", self.environment):
            return running
        return self.store.count_outside_session("tasks", "running", allowed)

    def pause_text(self, pause, running, other_running):
        """暂停态的一句话（/health 的 pause.text，control pause / check 照打）：暂停中（原因：…，起于…）、在跑 N 个；放行了会话的另写放行会话与别的会话在跑几个。
        弄坏开关 pause-allow-unreported 下不写放行的会话。"""
        started = time.strftime("%Y-%m-%d %H:%M:%S", time.localtime(pause["since"])) if pause["since"] else "不知道什么时候"
        text = f"暂停中（原因：{pause['reason'] or '没写'}，起于 {started}）"
        allowed = pause.get("allow_session", "")
        if allowed and not lib.broken("pause-allow-unreported", self.environment):
            return f"{text}、放行会话 {allowed}、在跑 {running} 个、别的会话在跑 {other_running} 个"
        return f"{text}、在跑 {running} 个"

    def pause_reply(self, changed, pause, allow_before="", allow_changed=False):
        """接口 POST /pause、/resume 的回话：pause 是这一回涉及的那一份暂停态（没有交 None）；running 是此刻 state 为 running 的任务数，
        other_running 是其中不是放行会话的；allow_session 是这一份里放行的会话，allow_before 是换之前的，allow_changed 是这一回换了没有。"""
        running = self.store.count("tasks", "running")
        current = self.pause
        return {"paused": current is not None, "changed": changed, "reason": pause["reason"] if pause else "", "since": pause["since"] if pause else None,
                "running": running, "other_running": self.running_outside_allowed(current, running) if current is not None else running,
                "allow_session": pause.get("allow_session", "") if pause else "", "allow_before": allow_before, "allow_changed": allow_changed}

    def pause_service(self, reason, allow_session, session, process_id):
        """暂停（接口 POST /pause）：拿 start_gate 改库里的暂停态、再改排队线程读的那一份，之后一条新尝试都不起（放行的会话交的除外）；
        已经暂停着的不改原因与起点，changed 为 false，给了不同的 allow_session 就换放行的会话（allow_changed 为 true），不给的不动放行。
        暂停态校验不过的（valid 为 False）不换放行。弄坏开关 pause-start-ungated 下不拿锁，pause-state-memory-only 下不写库，
        pause-allow-change-ignored 下已暂停着不换放行的会话，resume-keeps-allowed-session 下没给 allow_session 的沿用上一次 resume 时的放行。"""
        gate = contextlib.nullcontext() if lib.broken("pause-start-ungated", self.environment) else self.start_gate
        allow_before, allow_changed = "", False
        with gate:
            if self.pause is not None:
                allow_before = self.pause["allow_session"]
                if (not allow_session or allow_session == allow_before or not self.pause["valid"]
                        or lib.broken("pause-allow-change-ignored", self.environment)):
                    return self.pause_reply(False, self.pause, allow_before)
                self.store.write_pause(self.pause["reason"], self.pause["since"], allow_session)
                self.pause = dict(self.pause, allow_session=allow_session)
                allow_changed = True
            else:
                if not allow_session and lib.broken("resume-keeps-allowed-session", self.environment):
                    allow_session = self.kept_allow_session
                since = round(time.time(), 3)
                if not lib.broken("pause-state-memory-only", self.environment):
                    self.store.write_pause(reason, since, allow_session)
                self.pause = {"reason": reason, "since": since, "allow_session": allow_session, "valid": True}
        if allow_changed:
            self.operation("pause", 0, session, process_id, f"换放行会话 {allow_before or '没有'} → {allow_session}")
        else:
            self.operation("pause", 0, session, process_id, reason + (f"（放行会话 {allow_session}）" if allow_session else ""))
        self.wake()
        return self.pause_reply(not allow_changed, self.pause, allow_before, allow_changed)

    def resume_service(self, session, process_id):
        """放开暂停（接口 POST /resume）：清库里的暂停态（放行的会话在里面，一并没了）、再清排队线程读的那一份，叫醒排队线程；本来没暂停的 changed 为 false。
        弄坏开关 resume-leaves-cache-paused 下只清库，resume-keeps-allowed-session 下记下放行的会话、下一次不带 --allow-session 的 pause 照它放行。"""
        with self.start_gate:
            was = self.pause
            if was is None:
                return self.pause_reply(False, None)
            self.store.clear_pause()
            if not lib.broken("resume-leaves-cache-paused", self.environment):
                self.pause = None
            if lib.broken("resume-keeps-allowed-session", self.environment):
                self.kept_allow_session = was["allow_session"]
        self.operation("resume", 0, session, process_id, was["reason"] + (f"（放行会话 {was['allow_session']}）" if was["allow_session"] else ""))
        self.wake()
        return self.pause_reply(True, was)

    def waiting_text(self, task):
        """这条任务在等什么：排着而服务暂停着，是暂停的原因（放行会话交的除外，它写自己真在等什么）；不然是上一轮判的排队原因（status、列表与 wait 的排队过久提醒读它）。
        弄坏开关 pause-waiting-unreported 下不写暂停，pause-allow-waiting-paused 下放行会话排着的也写暂停。"""
        pause = self.pause
        if (pause is not None and task.get("state") == "queued" and not lib.broken("pause-waiting-unreported", self.environment)
                and (lib.broken("pause-allow-waiting-paused", self.environment) or not self.allowed_during_pause(task, pause))):
            return f"服务暂停中（原因：{pause['reason'] or '没写'}）"
        reason = self.waiting_reasons.get(task["id"], "")
        return reason

    def start_attempt_unless_paused(self, task, placement):
        """起这一次尝试，除非此刻暂停着而它不是放行的会话交的 → (尝试, 因暂停没起)。拿 start_gate 判过没暂停才起，pause_service 同一把锁：pause 返回之后一条新的都不起，
        已经开始的在它返回之前都记下了。弄坏开关 pause-start-ungated 下判了不拿锁。"""
        if lib.broken("pause-start-ungated", self.environment):
            if self.pause_holds(task) is not None:
                return None, True
            return self.start_attempt(task, placement), False
        with self.start_gate:
            if self.pause_holds(task) is not None:
                return None, True
            return self.start_attempt(task, placement), False

    # ── 主循环 ──
    def wake(self):
        self.wake_event.set()

    def notify_finished(self, task_id):
        """记下结局之后叫醒在等这条任务的 wait（文件头「通知」）与发消息那一路（文件头 ⑥：任务号进 inbox_pending）；弄坏开关 wait-notify-skipped 时不叫 wait，wait 只能等兜底那一轮。"""
        with self.inbox_lock:
            self.inbox_pending.add(task_id)
        self.inbox_event.set()
        if lib.broken("wait-notify-skipped", self.environment):
            return
        for listener in list(self.finished_listeners):
            with contextlib.suppress(Exception):
                listener(task_id)

    def loop(self):
        while not self.stopping.is_set():
            self.last_tick = time.time()
            try:
                if not (self.pause is not None and lib.broken("pause-halts-reconcile", self.environment)):   # 暂停着对账照做（在跑的照跑到结局）；弄坏开关下对账也停
                    self.reconcile()
                self.expire_unconfirmed()   # 交了满时限没确认的取消（文件头 ⑦）
                self.place_queued()
                self.sweep_scenes()   # 取走结局满一段时间的现场删掉（task_server_lib.py 文件头「现场」）
                if not lib.broken("health-judges-head", self.environment):
                    self.starved_text = self.starved_problem(time.time())   # /health 只读这一份（文件头「接口」）
                self.progress_text = self.progress_problem(time.time())   # 看进展：/health 同样只读这一份
            except Exception:   # 一轮出事不停服务：打出来、下一轮再判
                say("  ! 这一轮排队出了异常（下一轮再判）：\n" + traceback.format_exc())
            self.wake_event.wait(PEER_REPASS_SECONDS if self.repass_soon else self.tick_seconds)
            self.wake_event.clear()

    def stop(self):
        self.stopping.set()
        self.wake()
        self.inbox_event.set()
        if lib.broken("probe-children-left", self.environment):   # 弄坏开关：照改前只等排队线程、不管自己起的子进程
            self.thread.join(timeout=30)
        else:   # 文件头 ③「停服务」：各后台线程合计至多等 STOP_JOIN_SECONDS，再停还在的子进程
            deadline = time.monotonic() + STOP_JOIN_SECONDS
            # 第二台的余量探测线程同样等（它起的 ssh 在停子进程之前收完；弄坏开关 peer-probe-thread-unjoined 下不等，停完子进程之后它还能再起一个 ssh 留在服务单元里）
            probe = () if lib.broken("peer-probe-thread-unjoined", self.environment) else (self.peer_probe_thread,)
            for thread in (self.thread, self.nfs_thread, self.inbox_thread, self.runaway_thread, self.booking_audit_thread) + probe:
                if thread.is_alive():
                    thread.join(timeout=max(0.0, deadline - time.monotonic()))
        if lib.broken("no-detach", self.environment):
            for child in self.children:   # 弄坏开关 no-detach：起成服务的子进程的尝试随服务一起停（单元的 cgroup 被停时就是这样）
                with contextlib.suppress(ProcessLookupError):
                    child.send_signal(signal.SIGTERM)
        if not lib.broken("probe-children-left", self.environment):
            self.stop_own_children()

    def stop_own_children(self):
        """停服务时还在的、服务自己起的子进程（文件头 ③「停服务」）：照 /proc 的父进程号从自己往下找，逐个按进程号先 TERM、至多等 STOP_CHILD_GRACE_SECONDS 再 KILL；
        弄坏开关 no-detach 下起成子进程的尝试（self.children 与它们的子孙）不算在内。"""
        kept = set()
        for child in self.children:
            kept |= {child.pid} | descendant_processes(child.pid)
        leftover = descendant_processes(os.getpid()) - kept
        if not leftover:
            return
        say(f"  · 停服务：自己起的子进程还有 {len(leftover)} 个（{'、'.join(str(pid) for pid in sorted(leftover))}），逐个停")
        for signal_number in (signal.SIGTERM, signal.SIGKILL):
            for process_id in sorted(leftover):
                with contextlib.suppress(ProcessLookupError, PermissionError):
                    os.kill(process_id, signal_number)
            deadline = time.monotonic() + STOP_CHILD_GRACE_SECONDS
            while time.monotonic() < deadline:
                leftover = {pid for pid in leftover if os.path.exists(f"/proc/{pid}") and not process_is_zombie(pid)}
                if not leftover:
                    return
                time.sleep(0.05)

    # ── NFS 共享与第二台用不用得了（task_server_lib.py 文件头「NFS 共享」；本文件头 ②「机器」）──
    def nfs_loop(self):
        """每 nfs_recheck_seconds 复核一次 NFS 共享的前三项（两向探针只在起来时做，结论照留）：第二台没挂上的根记进 mount_problem，peer_view 当第二台此刻用不了；
        随后读一次第二台的余量（队空时排队线程不读），第二台用不了照 note_peer 告警。"""
        while not self.stopping.wait(self.nfs_recheck_seconds):
            try:
                report = lib.nfs_share_report(self.configuration.path, self.environment, probe=False)
            except Exception:   # 复核出事不停服务：打出来、下一回再判
                say("  ! 这一回 NFS 复核出了异常（下一回再判）：\n" + traceback.format_exc())
                continue
            with self.nfs_lock:
                probes = {entry["root"]: entry.get("probe") for entry in self.nfs.get("roots", [])}
            for entry in report["roots"]:
                entry["probe"] = probes.get(entry["root"], entry.get("probe"))
            for slow in report.get("slow") or []:   # 第二台一次超时、复查期里回了（task_server_lib.py 文件头「NFS 共享」的挂载）：只记进服务日志与 /health，不推提醒、照派第二台
                say(f"  · 第二台慢（超时、复查通了）：{slow}")
            if report.get("slow"):
                self.nfs_slow_last = {"at": round(time.time(), 3), "slow": list(report["slow"])}
            unmounted = next((entry for entry in report["roots"] if entry.get("mount") != lib.NFS_PASSED), None)
            self.mount_problem = (f"第二台没挂上共享根 {unmounted['root']}（{unmounted['mount']}；{lib.NFS_README}）" if unmounted else "")
            with self.nfs_lock:
                self.nfs = dict(report, probe_checked_at=self.nfs.get("probe_checked_at", self.nfs.get("checked_at")), slow_last=self.nfs_slow_last)
            try:   # 队空时排队线程不读余量：第二台用不了也在这里认出来、告警（note_peer）
                self.peer_view(self.resources.snapshot())
            except Exception:
                say("  ! 这一回读第二台的余量出了异常（下一回再判）：\n" + traceback.format_exc())

    def nfs_snapshot(self):
        with self.nfs_lock:
            return json.loads(json.dumps(self.nfs))

    def peer_view(self, snapshot):
        """这一轮读的余量照 NFS 复核改：第二台没挂上共享根的当它此刻用不了；顺带记第二台用不用得了（note_peer）。"""
        if self.mount_problem and snapshot.reachable.get(PEER):   # 共享根由本机导出、第二台挂载（文件头「NFS 共享」）：挂载复核只判第二台
            snapshot = dataclasses.replace(snapshot, reachable=dict(snapshot.reachable, **{PEER: False}), down=dict(snapshot.down, **{PEER: self.mount_problem}),
                                           down_kind={host: kind for host, kind in snapshot.down_kind.items() if host != PEER},
                                           reports={host: report for host, report in snapshot.reports.items() if host != PEER})
        if snapshot.expected.get(PEER):
            available = bool(snapshot.reachable.get(PEER)) and PEER in snapshot.reports
            self.note_peer(available, "" if available else (snapshot.down.get(PEER) or "第二台的余量查不成"), kind="" if available else snapshot.down_kind.get(PEER, ""))
        return self.unstarted_slices_filled(snapshot)

    def unstarted_slices_filled(self, snapshot):
        """slice 还没起的台（重启之后还没起过内存包装；报告里 memory_peaks_lib.SLICE_STARTED_KEY 为假，slice_total 是按整机内存估的）：总上限换成多机配置
        SLICE_MEMORY_CAP_BY_HOST 里那台那一项（包装第一次起就把 slice 设成它），slice 放得下照它减 slice 里已用的；两台同一段。配置读不到的照报告原样。
        弄坏开关 unstarted-slice-estimated 下照原样用估的。"""
        if lib.broken("unstarted-slice-estimated", self.environment):
            return snapshot
        unstarted = [host for host, report in snapshot.reports.items() if (report.get("memory") or {}).get(self.peaks.SLICE_STARTED_KEY) is False]
        if not unstarted:
            return snapshot
        caps, _ = self.configured_slice_caps()
        reports = dict(snapshot.reports)
        for host in unstarted:
            total = int((caps or {}).get(host) or 0)
            if total > 0:
                memory = dict(reports[host]["memory"], slice_total=total)
                memory["slice_room"] = total - int(memory.get("slice_used") or 0)
                reports[host] = dict(reports[host], memory=memory)
        return dataclasses.replace(snapshot, reports=reports)

    def note_peer(self, available, why, task_id=None, kind=""):
        """第二台用不用得了（本文件头 ②「机器」的告警）：从用得了变成用不了、或用不了的原因变了，写一条告警 peer-unavailable；/health 的 peer 一栏读 peer_status（why 照最新的写）。
        kind 给了的按它去重（盘余量不到下限：余量的数每轮在变，不每轮另写一条），没给的按原因的字面去重。弄坏开关 peer-unavailable-silent 下不记、不告警。"""
        if lib.broken("peer-unavailable-silent", self.environment):
            return
        with self.peer_status_lock:
            status = self.peer_status
            if available:
                if not status["available"]:
                    say(f"  · 第二台又用得了（之前：{status['why']}）")
                status.update(available=True, why="", since=None)
                self.peer_alerted = ""
                return
            if status["available"]:
                status.update(available=False, since=round(time.time(), 3))
            status["why"] = why
            if self.peer_alerted == (kind or why):
                return
            self.peer_alerted = kind or why
        self.alert("peer-unavailable", task_id, f"第二台此刻用不了：{why}；派得到第二台的任务这时只派本机",
                   f"照那一句修第二台（连法、多机配置的双机开关、NFS 挂载照 {lib.NFS_README}），修好之后服务下一轮自己认回；用不了满 {self.peer_unavailable_notify_seconds:.0f} 秒服务发消息给有任务在排、在跑的会话")

    def note_peer_run_failure(self, task, attempt):
        """那一趟没跑成（peer-run-failed：树建不成、ssh 被重置、那边的准备没过）：每一次写一条告警 peer-unavailable，/health 的 peer 一栏记最近一次。"""
        if lib.broken("peer-unavailable-silent", self.environment):
            return
        why = f"任务 {task['id']} 第 {attempt.get('number')} 次在第二台那一趟没跑成（multi-host-run.sh 退 255）：{lib.log_last_line(log_tail(attempt.get('log', '')))}"
        with self.peer_status_lock:
            self.peer_status["last_run_failure"] = {"time": round(time.time(), 3), "task": task["id"], "why": why}
        self.alert("peer-unavailable", task["id"], why, f"看 task-server/task log {task['id']}；连不上、树建不成照那一句修第二台，服务照重试的规矩重排")

    def peer_health(self):
        with self.peer_status_lock:
            return dict(self.peer_status)

    def check_prune_server(self, task, attempt):
        """命令退 75（prune-server 不可用，task_server_lib.py 文件头「错误类型」）：另起一个线程在仓根跑一次 python3 prune-server/control check
        （至多 PRUNE_SERVER_CHECK_SECONDS 秒），不通（退非 0、起不来、超时、那份文件不在）写一条告警 prune-server-unavailable；只报，不起、不停 prune-server。
        自证里 check 换成 TASK_SERVER_PRUNE_SERVER_CHECK 那一份可执行文件。"""
        fake = lib.selftest_override("TASK_SERVER_PRUNE_SERVER_CHECK", self.environment)
        command = [fake] if fake else [sys.executable, "-B", lib.PRUNE_SERVER_CONTROL, "check"]
        shown = "python3 prune-server/control check"

        def run():
            if not fake and not os.path.isfile(lib.PRUNE_SERVER_CONTROL):
                code, text = None, f"仓里没有 {lib.PRUNE_SERVER_CONTROL}"
            else:
                try:
                    completed = subprocess.run(command, cwd=lib.REPOSITORY_ROOT, capture_output=True, text=True, env=self.environment, stdin=subprocess.DEVNULL,
                                               timeout=PRUNE_SERVER_CHECK_SECONDS)
                    code, text = completed.returncode, completed.stderr + completed.stdout
                except (OSError, subprocess.TimeoutExpired) as error:
                    code, text = None, f"{type(error).__name__}: {error}"
            if code == 0:
                say(f"  · 任务 {task['id']} 第 {attempt.get('number')} 次命令退 75，{shown} 通（只报，不另做什么）")
                return
            self.alert(lib.PRUNE_SERVER_ALERT_KIND, task["id"],
                       f"任务 {task['id']} 第 {attempt.get('number')} 次命令退 75（prune-server 不可用）；{shown} {'退 ' + str(code) if code is not None else '没跑成'}："
                       f"{lib.last_line(text, '没有输出')[:300]}",
                       "照 check 那一句修 prune-server（task-server 不起、不停它：python3 prune-server/control start / install 由主 agent 起）；修好之后重交这条任务")
        threading.Thread(target=run, name=f"prune-server-check-{task['id']}", daemon=True).start()

    # ── ① 对账 ──
    def attempt_paths(self, attempt):
        directory = attempt["directory"]
        return os.path.join(directory, "result.json"), os.path.join(directory, "started"), os.path.join(directory, "scope")

    def settle_ended_attempts(self):
        """任务还 running、它最新那一次尝试却已经 ended 的（上一次定去向半途抛错，或服务在记尝试与定去向之间没了）：补定去向（settle_attempt）。
        任务 2461 就是这样留了两小时的孤儿：本机因此一直算有任务在跑。弄坏开关 ended-attempts-unsettled 下不补（改前的样子）。"""
        if lib.broken("ended-attempts-unsettled", self.environment):
            return
        for task, valid in self.store.select("tasks", "state = 'running'"):
            if not valid:
                continue
            attempts = [record for record, record_valid in self.store.attempts_of(task["id"]) if record_valid]
            if not attempts or attempts[-1].get("state") != "ended" or int(attempts[-1].get("number") or 0) != int(task.get("attempt_count") or 0):
                continue
            attempt = attempts[-1]
            if self.finishing_now(attempt.get("id")):   # 收尾线程记了 ended、正在定去向（finish_off_loop）：不是半途断了的，不补
                continue
            say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试结局 {attempt.get('outcome')} 已记、任务没收尾：现在补收尾")
            self.settle_attempt(task, attempt, None)

    def reconcile(self):
        self.settle_ended_attempts()
        self.forget_settled_entries(time.time())
        unrepaired = lib.broken("tampered-rows-left-unrepaired", self.environment)
        for attempt, valid in self.store.select("attempts", "state IN ('starting', 'running')"):
            if self.finishing_now(attempt.get("id")):   # 收尾在后台线程里做着（finish_off_loop）：这一轮不看它
                continue
            owner = association_id(attempt.get("task_id"))
            found = self.store.task(owner) if owner else None
            if not unrepaired and (not valid or (found is not None and not found[1])):   # 这一行或它那条任务核不过：收尾这一次（end_tampered_attempt）
                self.end_tampered_attempt(attempt, found, "尝试记录" if not valid else "任务记录")
                continue
            if found is None:
                continue
            task, task_valid = found
            if task.get("state") == "finished":
                continue
            if not valid or not task_valid:   # 弄坏开关 tampered-rows-left-unrepaired：只给任务记 record-tampered，尝试行留在在跑里
                self.tampered(task["id"], task, "尝试记录" if not valid else "任务记录")
                continue
            result_path, started_path, _ = self.attempt_paths(attempt)
            if attempt["state"] == "starting" and lib.broken("start-twice-after-restart", self.environment) and not attempt.get("restarted"):
                attempt["restarted"] = True   # 弄坏开关：不看结果、单元与起跑标记，再起一份
                self.launch(task, attempt)
                continue
            # 弄坏开关 unit-unreadable-exit-file-unread：读不到单元状态的那一段不认 result.json（核到没了也照没留结果收）
            exit_file_unread = lib.broken("unit-unreadable-exit-file-unread", self.environment) and attempt["id"] in self.unit_unreadable
            result = None if exit_file_unread else read_json_file(result_path)
            if result is not None:
                self.finish_off_loop(task, attempt, result)
                continue
            now = time.monotonic()
            if now - self.unit_checked_at.get(attempt["id"], 0.0) < self.unit_check_seconds:
                continue
            self.unit_checked_at[attempt["id"]] = now
            if attempt.get("unit"):
                states = unit_states(attempt["unit"], self.environment)
                if states is None:   # 判不出单元在不在（文件头 ①「读不到单元状态」）：不杀、不收尾、不记结局，不经 systemd 核到都没了才照没留结果收尾
                    # 弄坏开关 unit-unreadable-lost-unverified：读不到就当场照没留结果收尾（不核进程，收尾照常经 systemctl 回读、停留下的进程）
                    if lib.broken("unit-unreadable-lost-unverified", self.environment) or self.unit_unreadable_gone(attempt, now):
                        self.finish_off_loop(task, attempt, None if exit_file_unread else read_json_file(result_path))
                    continue
                self.forget_unit_unreadable(attempt["id"])
                load_state, active_state = states
            else:   # 弄坏开关 no-detach：起成子进程的那几次，看进程号在不在
                alive = any(child.pid == attempt.get("child_pid") and child.poll() is None for child in self.children)
                load_state, active_state = ("loaded", "active") if alive else ("not-found", "inactive")
            if active_state in lib.RUNNING_UNIT_STATES:
                self.unit_gone_since.pop(attempt["id"], None)
                self.unit_stopped.discard(attempt["id"])
                self.check_stall(task, attempt)
                if attempt["state"] == "starting":
                    attempt["state"] = "running"
                    self.store.update("attempts", attempt)
                continue
            if attempt["state"] == "starting" and load_state == "not-found" and not os.path.exists(started_path):
                self.resume_starting(task, attempt, now)
                continue
            gone_since = self.unit_gone_since.setdefault(attempt["id"], now)
            if now - gone_since < self.lost_grace_seconds or lib.broken("lost-not-recorded", self.environment):
                continue
            self.finish_off_loop(task, attempt, read_json_file(result_path))
        self.drive_sharded_parents()

    def unit_unreadable_gone(self, attempt, now):
        """对账读不到这一次的单元状态（unit_states 交 None）、又没留 result.json 时调（文件头 ①「读不到单元状态」）：记下从哪一刻（now，monotonic）起连着读不到，
        照不经 systemd 最近一回核到的（unit_presence_reading，后台线程核）定：核到这一次在两台上的 cgroup 都读得到且空、记下的进程号都不在，连着满 lost_grace_seconds
        交 True（由对账照没留结果收尾、记 launcher-lost；attempt 记上 unit_unreadable_gone，stop_leftovers 照它认）；还在的、核不出的、还没核完的交 False
        （不杀、不收尾、不记结局，尝试照记 running，不设到点就收尾的时限）。头一回读不到、之后每跨过一道提醒门槛（同 unit-unreadable 提醒），与照没留结果收尾的那一刻，各往服务日志打一行 TASK_UNIT_UNREADABLE。"""
        entry = self.unit_unreadable.setdefault(attempt["id"], {"since": now, "task": attempt.get("task_id"), "number": attempt.get("number"),
                                                                "host": attempt.get("host"), "unit": attempt.get("unit")})
        reading = self.unit_presence_reading(attempt, now)
        verdict, detail = (reading[0], reading[1]) if reading is not None else (lib.PRESENCE_UNPROBED, lib.PRESENCE_UNPROBED_TEXT)
        crossed = reminder_threshold(now - entry["since"], self.unit_unreadable_attention_seconds)
        if crossed > self.unit_unreadable_noted.get(attempt["id"], -1):
            self.unit_unreadable_noted[attempt["id"]] = crossed
            self.note_unit_unreadable(attempt, now - entry["since"], verdict, detail)
        if verdict != lib.PRESENCE_GONE:
            self.unit_gone_since.pop(attempt["id"], None)
            return False
        if now - self.unit_gone_since.setdefault(attempt["id"], now) < self.lost_grace_seconds:
            return False
        attempt["unit_unreadable_gone"] = detail
        self.note_unit_unreadable(attempt, now - entry["since"], verdict, f"核到都不在，照没留结果收尾：{detail}")
        return True

    def note_unit_unreadable(self, attempt, seconds, verdict, detail):
        """读不到单元状态的那一段往服务日志打的一行（文件头 ①「读不到单元状态」）。"""
        say(f"TASK_UNIT_UNREADABLE time={time.time():.6f} attempt={attempt['id']} task={attempt.get('task_id')} number={attempt.get('number')} "
            f"host={attempt.get('host')} unit={attempt.get('unit')} seconds={seconds:.0f} presence={verdict} detail={detail}")

    def forget_unit_unreadable(self, attempt_id):
        """清掉一次尝试读不到单元状态的那一段记账（对账读到了单元状态、收尾、核不过而记 ended 时；文件头 ①「读不到单元状态」）。"""
        self.unit_unreadable.pop(attempt_id, None)
        self.unit_unreadable_noted.pop(attempt_id, None)
        with self.unit_presence_lock:
            self.unit_presence.pop(attempt_id, None)

    def unit_presence_reading(self, attempt, now):
        """不经 systemd 最近一回核到的 (lib.PRESENCE_*, 说明, 核完的时刻)，还没核完交 None；上一回比 unit_check_seconds 旧、又没在核的，另起一个后台线程再核
        （probe_unit_presence；第二台那一趟带 ssh，排队线程不等它）。"""
        key = attempt["id"]
        with self.unit_presence_lock:
            reading = self.unit_presence.get(key)
            if key not in self.unit_presence_probing and (reading is None or now - reading[2] >= self.unit_check_seconds):
                self.unit_presence_probing.add(key)
                threading.Thread(target=self.probe_unit_presence, args=(dict(attempt),), name=f"unit-presence-{key}", daemon=True).start()
        return reading

    def unit_presence_now(self, attempt_id):
        """提醒与 /health 读的：(lib.PRESENCE_*, 说明)，还没核完的交 PRESENCE_UNPROBED。"""
        with self.unit_presence_lock:
            reading = self.unit_presence.get(attempt_id)
        return (reading[0], reading[1]) if reading is not None else (lib.PRESENCE_UNPROBED, lib.PRESENCE_UNPROBED_TEXT)

    def probe_unit_presence(self, attempt):
        """unit_presence_reading 起的后台线程：核一回（attempt_presence），写进 unit_presence；这一段已清掉（读到了、收尾了）的不写回。
        核法自己出事照核不出办：不收尾，下一回再核。"""
        try:
            verdict, detail = self.attempt_presence(attempt)
        except Exception as error:
            verdict, detail = lib.PRESENCE_UNVERIFIED, f"核进程出了异常（{type(error).__name__}: {error}）"
        with self.unit_presence_lock:
            self.unit_presence_probing.discard(attempt["id"])
            if attempt["id"] in self.unit_unreadable:
                self.unit_presence[attempt["id"]] = (verdict, detail, time.monotonic())

    def attempt_presence(self, attempt):
        """不经 systemd 核这一次尝试还有没有进程（文件头 ①「读不到单元状态」）→ (lib.PRESENCE_*, 说明)。本机读单元那一层（attempt_runner 与它起的包装都在本机的单元里，
        派到第二台的也是）：起跑标记记的 runner 进程号、单元的 cgroup、attempt_runner 写的 scope 文件点名的目录、带这一次尝试标签的内存包装 scope；
        派到第二台、清理回执 pending 的另读第二台那一趟（multi-host-run.sh 记的进程号文件与 scope 单元名文件）。两台跑同一段（Machine.attempt_presence_readings），
        判在 lib.attempt_presence_verdict。起跑标记不在或读不了、scope 文件读不了、清理回执读不到或认不出的，那一台照核不出算。
        弄坏开关 unit-unreadable-absent-cgroup-unconfirmed 下两台都把不在的 cgroup 目录算核不出（请求里带 absent_cgroup_unconfirmed）。"""
        _, started_path, scope_path = self.attempt_paths(attempt)
        label = attempt.get("label") or ""
        common = {"label_entry": f"{lib.ATTEMPT_LABEL_VARIABLE}={label}" if label else "", "label_scope_prefix": lib.MEMORY_CAP_SCOPE_PREFIX,
                  "tree_depth": lib.ATTEMPT_PRESENCE_TREE_DEPTH, "absent_cgroup_unconfirmed": lib.broken("unit-unreadable-absent-cgroup-unconfirmed", self.environment)}
        runner, problem = recorded_text(started_path, "起跑标记")
        if problem or not runner.isdigit():
            return lib.attempt_presence_verdict([(LOCAL, None, problem or f"起跑标记 {started_path} 不在或不是进程号（attempt_runner 还没起，或单元没起成），核不出")])
        scope, problem = recorded_text(scope_path, "scope 文件")
        if problem:
            return lib.attempt_presence_verdict([(LOCAL, None, problem)])
        # scope 文件只认真的 cgroup 目录（同 stop_leftovers：自证里 attempt_runner 读的是假 scope 目录，那一份不是 cgroup）
        recorded_scope = [scope] if scope.startswith("/sys/fs/cgroup/") and scope.endswith(".scope") else []
        local_request = dict(common, process_ids=[runner], process_id_files=[], cgroup_directories=recorded_scope, cgroup_names=[attempt["unit"]], cgroup_name_files=[])
        parts = [(LOCAL, *self.machines[LOCAL].attempt_presence_readings(local_request))]
        if attempt.get("host") != LOCAL:
            parts.append(self.peer_presence_part(attempt, common))
        return lib.attempt_presence_verdict(parts)

    def peer_presence_part(self, attempt, common):
        """派到第二台的那一次在第二台那一趟的读数（attempt_presence）：→ (台, 读数或 None, 读不成的那一句)。清理回执 not-started（那一趟没起过）、confirmed（已收干净）的
        第二台没有这一次的进程；pending 的照回执里的进程号文件（runs/<标签>.pid）与它旁边的 scope 单元名文件（runs/<标签>.scope，名字不带 .scope）在第二台上核。"""
        host = attempt["host"]
        path = attempt.get("peer_cleanup_file") or ""
        receipt = read_json_file(path) if path else None
        status = (receipt or {}).get("status")
        if receipt is None:
            return host, None, f"清理回执 {path or '（这一次没有）'} 读不到，第二台那一趟核不出"
        if status in ("not-started", "confirmed"):
            return host, {"processes": {}, "cgroups": {}, "files": {}, "problems": [], "note": f"清理回执 {status}，第二台没有这一次的进程"}, ""
        pid_file = receipt.get("pid_file")
        if status != "pending" or not isinstance(pid_file, str) or not pid_file.startswith("/") or not pid_file.endswith(".pid"):
            return host, None, f"清理回执 {path} 认不出（状态 {status}、进程号文件 {pid_file}），第二台那一趟核不出"
        request = dict(common, process_ids=[], process_id_files=[pid_file], cgroup_directories=[], cgroup_names=[],
                       cgroup_name_files=[pid_file[:-len(".pid")] + ".scope"], cgroup_name_suffix=".scope")
        return (host, *self.machines[host].attempt_presence_readings(request))

    def end_tampered_attempt(self, attempt, found, what):
        """对账判出在跑的这一次核不过（尝试行或它那条任务的行校验不过；task_server_lib.py 文件头「库」）：它那条任务还没结局的照 tampered 记 record-tampered、
        告警一条；任务已有结局、或库里没有它的，告警写这一行。这一行记 ended、结局 record-tampered，正文只留认得出是哪一次的几样（ENDED_TAMPERED_FIELDS），
        路径、峰值与记账都不留（正文不可信，服务不照它删、不照它记账），关联列与正文的 task_id 照 association_id 写、重签：之后它核得过、
        不在在跑的尝试里（Store.running_attempts），不再占内存、盘与线程。同一任务同一次号已有一行的（关联列被挪过来顶了号），task_id 写 0。
        记完之后后台线程照 stop_tampered_unit 停它的单元、回读、报 tampered-unit-stop（不算占用的单元留着跑，这台的内存会被多派）。
        弄坏开关 tampered-unit-left-running 下不停。"""
        owner = association_id(attempt.get("task_id"))
        if found is not None and found[0].get("state") != "finished":
            self.tampered(found[0]["id"], found[0], what)
        elif not lib.broken("tamper-unalerted", self.environment):
            self.alert("record-tampered", owner, f"库里在跑的尝试记录（行 {attempt['id']}）或它那条任务的记录校验不过：服务不认这一次，记 ended、不再算在跑",
                       f"查是谁直接写了库（只有 task-server 读写它）；它的单元 {attempt.get('unit') or '（没记）'} 服务接着停，结果另报 tampered-unit-stop")
        ended = {name: attempt[name] for name in ENDED_TAMPERED_FIELDS if isinstance(attempt.get(name), (str, int, float)) and not isinstance(attempt.get(name), bool)}
        ended.update(id=attempt["id"], task_id=owner, state="ended", outcome="record-tampered", ended_at=round(time.time(), 3),
                     note=f"{what}校验不过：服务不认这一次，记 ended、不再算在跑（单元由服务停，结果另报 tampered-unit-stop）")
        for name in ("session", "command_key"):
            ended[name] = ended.get(name) if isinstance(ended.get(name), str) else ""
        try:
            self.store.update("attempts", ended)
        except sqlite3.IntegrityError:   # (task_id, number) 唯一：那一号已有一行
            self.store.update("attempts", dict(ended, task_id=0))
        self.unit_checked_at.pop(attempt["id"], None)
        self.unit_gone_since.pop(attempt["id"], None)
        self.forget_unit_unreadable(attempt["id"])
        if not lib.broken("tampered-unit-left-running", self.environment):
            task = found[0] if found is not None and found[1] else None   # 任务行也核不过的不照它记的会话与令牌发消息，只写告警
            threading.Thread(target=self.stop_tampered_unit, args=(attempt.get("unit"), owner, task), daemon=True).start()

    def stop_tampered_unit(self, unit, owner, task):
        """停核不过的那一次的单元、回读，报 tampered-unit-stop（end_tampered_attempt 起的后台线程，排队线程不等 systemctl）。行的正文不可信，只停这两样都合的：
        名字是这个服务起的单元（singlefs-task-<库的标>-<任务号>-<次号>.service），又不是校验过的在跑尝试的单元；别的不动、照实报，由人核。
        第二台的那一趟这里停的是本机包装单元，第二台那边由包装收到 TERM 时清（multi-host-run.sh）。"""
        name = unit if isinstance(unit, str) else ""
        if not re.fullmatch(rf"singlefs-task-{re.escape(self.store.tag)}-[0-9]+-[0-9]+\.service", name):
            line = f"核不过的尝试记的单元 {name or '（没记）'} 不是这个服务起的单元名：没停"
        elif any(record.get("unit") == name for record in self.store.running_attempts()):
            line = f"核不过的尝试记的单元 {name} 是另一次校验过的在跑尝试的：没停"
        else:
            with contextlib.suppress(OSError, subprocess.TimeoutExpired):
                subprocess.run(["systemctl", "--user", "stop", name], capture_output=True, env=self.environment, stdin=subprocess.DEVNULL, timeout=UNIT_STOP_SECONDS + 30)
            states, processes = unit_states(name, self.environment), unit_processes(name, self.environment)
            if states is None or states[1] in lib.RUNNING_UNIT_STATES or processes is None or processes:
                line = (f"核不过的尝试的单元 {name} 停了之后回读没停干净：状态 {states[1] if states else '读不到'}、"
                        f"cgroup 里进程 {'读不到' if processes is None else len(processes)} 个")
            else:
                line = f"核不过的尝试的单元 {name} 已停（回读不在跑、cgroup 里没有进程）"
        next_step = lib.anomaly_advice("tampered-unit-stop", owner, self.environment)
        self.alert("tampered-unit-stop", owner, line, next_step)
        if task is not None and (task.get("inbox") or {}).get("token"):   # 人手交的没有令牌、没有会话可发
            self.send_to_session(task, "\n".join([f"{INBOX_HEADLINE}任务 {owner} 核不过的尝试：{line}", f"出路：{next_step}"]), ("tampered-unit-stop", owner, name))

    def forget_settled_entries(self, now):
        """按任务号记在内存里的那几张表（RESIDENT_TASK_TABLES）与按尝试号记的疑似卡住（suspect_reported、inbox_failed 里的 suspect 事件）随跑过的任务涨：
        对账每一轮问一次、至多隔 resident_prune_seconds 清一回。删掉任务有了结局并被取走（collected）、结局满 RESIDENT_RETENTION_SECONDS 没人取、
        或库里已没有的那几条任务的项；按尝试号记的删掉不在在跑的尝试里（Store.running_attempts）的项。库里另存了一份的（报到的提醒门槛、结局再推到第几回）
        删了照库里那份接着算；删了之后同一件再判出同一样的，照新的一件再告警、再报一次。弄坏开关 resident-entries-kept 下一项都不删。"""
        if lib.broken("resident-entries-kept", self.environment) or now - self.resident_pruned_at < self.resident_prune_seconds:
            return
        self.resident_pruned_at = now

        def task_of_event(event):   # inbox_failed 的事件归哪条任务：("outcome", 号)、(号, 种类, 提醒门槛)；("suspect", 尝试号, 段号) 归尝试，交 None
            if not isinstance(event, tuple) or not event or event[0] == "suspect":
                return None
            return event[1] if event[0] == "outcome" and len(event) > 1 else event[0]

        def task_of_key(key, position):
            return key[position] if position is not None and isinstance(key, tuple) and len(key) > position else (key if position is None else None)

        named = {task_of_key(key, position) for name, position in RESIDENT_TASK_TABLES.items() for key in list(vars(self)[name])}
        named |= {task_of_event(event) for event in list(self.inbox_failed)}
        candidates = sorted(task_id for task_id in named if association_id(task_id))
        present = {}
        for start in range(0, len(candidates), 500):
            chunk = candidates[start:start + 500]
            present.update((record["id"], record) for record, _ in self.store.select("tasks", f"id IN ({', '.join('?' * len(chunk))})", tuple(chunk)))
        settled = set()
        for task_id in candidates:
            record = present.get(task_id)
            if record is None or (record.get("state") == "finished" and (record.get("collected")
                                                                           or now - float(record.get("finished_at") or now) >= RESIDENT_RETENTION_SECONDS)):
                settled.add(task_id)
        running = {attempt.get("id") for attempt in self.store.running_attempts()}
        for name, position in RESIDENT_TASK_TABLES.items():
            container = vars(self)[name]
            for key in list(container):
                if task_of_key(key, position) in settled:
                    if isinstance(container, dict):
                        container.pop(key, None)
                    else:
                        container.discard(key)
        self.persist_restart_surviving_state("shard_advance")   # advance_failures、advance_episodes 删掉的项，库里那一份跟着删（一项没删的与上次写的相同，不重写）
        with self.suspect_lock:
            for attempt_id in list(self.suspect_reported):
                if attempt_id not in running:
                    self.suspect_reported.pop(attempt_id, None)
        self.persist_restart_surviving_state("suspect_reported")
        for event in list(self.inbox_failed):
            suspect_gone = isinstance(event, tuple) and len(event) > 1 and event[0] == "suspect" and event[1] not in running
            if suspect_gone or task_of_event(event) in settled:
                self.inbox_failed.discard(event)

    def resume_starting(self, task, attempt, now):
        """starting、单元不在、起跑标记也没有的那一次（文件头 ①）：拿这条任务的锁、现读任务再定——取消了的照取消收尾（不起）；
        起单元的回执超时过（start_uncertain_at，TS-06）的过了 start_uncertain_grace_seconds 还是这样，确认没起、记 unit-start-failed；
        别的是服务在写了单元名与起单元之间没了：起它一次。"""
        with self.store.task_lock(task["id"]):
            found = self.store.task(task["id"])
            current = found[0] if found is not None and found[1] else task
            if current.get("cancel_requested") or current.get("state") == "finished":
                say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试还没起、任务已取消：不起，照取消收尾")
                self.finish_off_loop(current, attempt, None)
                return
            uncertain_at = attempt.get("start_uncertain_at")
            if uncertain_at:
                if time.time() - float(uncertain_at) < self.start_uncertain_grace_seconds:
                    return
                why = (f"systemd-run 回执超时之后照单元名 {attempt.get('unit')} 读回 {self.start_uncertain_grace_seconds:.0f} 秒，单元一直不在、起跑标记也没有："
                       "确认没起")
                attempt.update(state="ended", outcome="unit-start-failed", ended_at=round(time.time(), 3), note=why)
                self.store.update("attempts", attempt)
                self.unit_checked_at.pop(attempt["id"], None)
                self.finalize(current, "unit-start-failed", None, why)
                return
            say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试的单元名写进库之后、起单元之前服务没了：现在起它（只这一次）")
            self.launch(current, attempt)

    def check_stall(self, task, attempt):
        """日志距上一次增长满 stall_seconds、这段时间 CPU 也几乎没涨就停掉这一条（文件头「卡住」）。两台同一段：日志里除起法那一层的进度行之外的字节数（log_content），
        CPU 照那一台的接口读（Machine.cpu_usec：本机读尝试目录的 cpu_usec，第二台读日志里最近一行 MULTI_HOST_RUN_PEER_CPU 的 usage_usec）。
        这一回没停它的，再判疑似卡住（check_suspect：只通知、不停）。
        人手交的跑满两小时另记一行进日志（long_run_sessionless_note）。
        头一步判四小时硬上限（judge_run_hard_limit，文件头 ⑦「四小时硬上限」）：不看宽限，重排之后刚起的那一次累计到了照停。"""
        if self.judge_run_hard_limit(task, attempt):
            return
        usage = self.fresh_cpu_usec(attempt)
        if usage is not None:   # CPU 闸的峰值段核数（note_peak_segment）：每一轮记一回累计读数，宽限里的也记，头一段从起跑那一会儿量起
            self.note_peak_segment(attempt, time.time(), usage)
        self.note_threads(attempt)   # CPU 闸的可跑线程数与线程总数远超登记数（文件头 ②「CPU 闸」、①「要处置的提醒」的 threads-over-registered）：每一轮读一回，宽限里的也读
        if attempt.get("stalled"):
            return
        self.long_run_sessionless_note(task, attempt)
        elapsed = time.time() - float(attempt.get("started_at") or time.time())
        if elapsed < self.stall_grace_seconds and not lib.broken("stall-grace-skipped", self.environment):
            self.log_growth.pop(attempt["id"], None)   # 宽限里不判、不量：宽限过了从那一刻起量窗口
            self.cpu_windows.pop(attempt["id"], None)
            self.io_waits.pop(attempt["id"], None)
            self.activity_windows.pop(attempt["id"], None)
            self.selftest_read(f"stall-grace task={task['id']} attempt={attempt['number']} elapsed={elapsed:.3f}")   # 自证：宽限里判过一次（stall-waits-for-grace 照它放行）
            return
        size = self.log_content(attempt)
        now = time.monotonic()
        cpu = self.machines[attempt["host"]].cpu_usec(attempt)
        if not self.judge_stall_stop(task, attempt, size, cpu, now):
            self.check_suspect(task, attempt, cpu, now)
            self.note_attempt_progress(attempt, now)

    def fresh_cpu_usec(self, attempt):
        """check_stall 每一轮开头读的这一次尝试累计 CPU 微秒：先把日志读到底（log_content：第二台的 CPU、线程数都在日志里，读到的是这一刻之前的最近一行），
        再照那一台的接口读（Machine.cpu_usec）；宽限里不读日志的轮、第二台的尝试同样是现读的。弄坏开关 stall-cpu-read-before-log 下不先读日志（第二台交上一回解析的旧读数）。"""
        if not lib.broken("stall-cpu-read-before-log", self.environment):
            self.log_content(attempt)
        return self.machines[attempt["host"]].cpu_usec(attempt)

    def note_attempt_progress(self, attempt, now):
        """日志不涨时读一回这一次尝试的进展（文件头 ①「要处置的提醒」的 quiet-log-busy；check_stall 每一轮调，now 是 time.monotonic()）：日志（log_content 算的那部分）
        满 stall_seconds 没涨、距上一回读满 stall_seconds 的一半才读（第二台每读一回起一次 ssh），照那一台的接口读（Machine.attempt_progress_readings：
        这一趟进程树里打开写着的文件与大小）。写着的文件比上一回读的大了（子进程的输出另存在运行目录里的 record.log 这类），
        记进 progress_noted（时刻与给人看的那半句：哪份文件从多少涨到多少字节）；quiet_log_items 与进度推送只读它，不另去读。
        上一回没读过的这一回只记大小（头一回读不出涨没涨）。弄坏开关 open-file-growth-ignored 下不看文件涨没涨。"""
        seen = self.log_growth.get(attempt["id"])
        if seen is None or now - seen[1] < self.stall_seconds:
            return
        last = self.progress_read_at.get(attempt["id"])
        if last is not None and now - last < self.stall_seconds / 2:
            return
        self.progress_read_at[attempt["id"]] = now
        readings, _ = self.machines[attempt["host"]].attempt_progress_readings(attempt)
        if readings is None:
            return
        files = {path: size for path, size in (readings.get("files") or {}).items() if isinstance(size, int) and not isinstance(size, bool)}
        previous = self.progress_files.get(attempt["id"])
        self.progress_files[attempt["id"]] = files
        pieces = []
        if previous is not None and not lib.broken("open-file-growth-ignored", self.environment):
            grown = sorted((path for path, size in files.items() if path in previous and size > previous[path]), key=lambda path: previous[path] - files[path])
            if grown:
                more = f"（另有 {len(grown) - 1} 份在涨）" if len(grown) > 1 else ""
                pieces.append(f"子进程写着的 {grown[0]} 从 {previous[grown[0]]} 涨到 {files[grown[0]]} 字节{more}")
        if pieces:
            self.progress_noted[attempt["id"]] = (now, "；".join(pieces))

    def judge_stall_stop(self, task, attempt, size, cpu, now):
        """文件头 ①「卡住」那几条停的规矩：日志与 CPU 都满窗口不涨、日志满窗口不涨而此刻内存贴着上限（CPU 在涨也停）；停了交 True。
        停的那一刻内存此刻或峰值到了上限的 STALL_AT_CAP_FRACTION 的记 memory-cap-hit、照重试规矩重排（顶在上限上卡住）。内存照那一台的接口读（Machine.memory_now）。
        日志与 CPU 都满窗口不涨、而这个窗口里每一轮都有进程在 D 状态的不停（卡在 IO 上；note_io_wait 记，D 状态照那一台的接口读 Machine.io_blocked）。"""
        if lib.broken("stall-ignored", self.environment):
            return False
        seen = self.log_growth.get(attempt["id"])
        if seen is None:
            # 头一回量（服务刚起来对账、或宽限刚过）：日志上一次涨的时刻取日志文件的修改时刻，不当成此刻（服务重启不把「日志多久没涨」清零）；
            # CPU 的窗口照旧从此刻起量（卡住判法不变）。读不到修改时刻、弄坏开关 stall-clock-reset-on-restart 下取此刻
            self.log_growth[attempt["id"]] = (size, self.log_changed_at(attempt, now), now, cpu, "")
            self.io_waits.pop(attempt["id"], None)
            return False
        if seen[0] != size:
            self.log_growth[attempt["id"]] = (size, now, now, cpu, "")
            self.io_waits.pop(attempt["id"], None)   # 日志涨了：卡在 IO 上那一段从头算
            return False
        self.note_io_wait(attempt, seen[2], now)   # 日志没涨的每一轮读一次 D 状态（文件头 ①「卡在 IO 上」）
        self.note_activity(task, attempt, seen[2])   # 日志没涨的每一轮读一次进程内存（文件头 ①「卡住」的有进展）
        if now - seen[2] < self.stall_seconds:
            return False
        elapsed = time.time() - float(attempt.get("started_at") or time.time())
        # 此刻贴着上限（任务 7723：上限 6144M，memory.current 6442393600、日志 37702 秒没涨，CPU 平均 2.526 核在回收里打转、跑了 37775 秒）：CPU 在涨、读不到都不算在算，
        # 往下走顶在上限上卡住那一支（记 memory-cap-hit、放大重排）。弄坏开关 at-cap-busy-cpu-kept-running 下不认，at-cap-fraction-ignored 下读得到就算贴着
        current, peak = self.machines[attempt["host"]].memory_now(attempt)
        cap = int(attempt.get("memory_cap") or 0)
        pinned_at_cap = (current is not None and cap > 0 and not lib.broken("at-cap-busy-cpu-kept-running", self.environment)
                         and (current >= STALL_AT_CAP_FRACTION * cap or lib.broken("at-cap-fraction-ignored", self.environment)))
        # 有进展（进程内存或内存峰值这一窗口在涨）：不停，从此刻再看一个窗口；贴着上限的不算（那是在回收里打转）。
        # 弄坏开关 progress-ignored-for-stall 下不看
        progress_ignored = lib.broken("progress-ignored-for-stall", self.environment)
        activity = "" if pinned_at_cap or progress_ignored else activity_text(self.activity_windows.get(attempt["id"]), seen[2])
        if activity:
            say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试的日志 {now - seen[1]:.0f} 秒没涨，而{activity}：有进展，不停，再看一个窗口")
            self.log_growth[attempt["id"]] = (size, seen[1], now, cpu, SPARED_ACTIVITY)
            return False
        if not pinned_at_cap and (seen[3] is None or cpu is None) and not lib.broken("cpu-unread-counted-as-idle", self.environment):
            # 窗口开头或此刻读不到 CPU（刚起、第二台那一趟还在起、换上读法之前起的尝试没有 cpu_usec）：读不到不等于不涨，不停，从此刻再看一个窗口
            self.log_growth[attempt["id"]] = (size, seen[1], now, cpu, SPARED_CPU_UNREAD)
            return False
        cores = (cpu - seen[3]) / 1e6 / (now - seen[2]) if cpu is not None and seen[3] is not None else None
        if (not pinned_at_cap and cores is not None and cores >= STALL_CPU_CORES
                and not lib.broken("cpu-ignored-for-stall", self.environment)):
            self.log_growth[attempt["id"]] = (size, seen[1], now, cpu, SPARED_CPU_BUSY)   # 日志不涨而 CPU 在涨：在算，不停，从此刻再看一个窗口
            return False
        waited = self.io_waits.get(attempt["id"])
        if (not pinned_at_cap and waited is not None and waited.polls > 0 and waited.every_poll_blocked
                and not lib.broken("io-wait-ignored", self.environment)):
            # 日志与 CPU 都不涨而这个窗口每一轮都有进程在 D 状态：在等盘或 NFS，不判 stalled、不停，从此刻再看一个窗口（满 io_wait_attention_seconds 由 io_wait_items 报 io-wait）
            say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试的日志 {now - seen[1]:.0f} 秒没涨、CPU 这 {now - seen[2]:.0f} 秒也几乎没涨，"
                f"而这 {waited.polls} 轮每一轮都有进程在 D 状态（最近一轮 {waited.blocked_now} 个）：在等 IO，不停，再看一个窗口")
            self.log_growth[attempt["id"]] = (size, seen[1], now, cpu, SPARED_IO_WAIT)
            return False
        attempt["stalled"] = True
        attempt["stalled_after"] = round(now - seen[1], 3)
        peak_at_cap = peak is not None and cap > 0 and peak >= STALL_AT_CAP_FRACTION * cap
        if (pinned_at_cap or peak_at_cap) and not lib.broken("stalled-at-cap-not-retried", self.environment):
            # 顶在上限上卡在回收里：不记 stalled，停掉之后照撞上限办（finish_attempt 记 memory-cap-hit、照重试规矩乘 1.5 重排）
            attempt["stalled_at_cap"] = True
            measured = "、".join(text for text in (f"此刻内存 {lib.mebibytes_text(current)}" if current is not None else "",
                                                   f"内存峰值 {lib.mebibytes_text(peak)}" if peak is not None else "") if text)
            busy = f"，CPU 这 {now - seen[2]:.0f} 秒平均 {cores:.2f} 个核（在回收里打转，不算在算）" if pinned_at_cap and cores is not None and cores >= STALL_CPU_CORES else ""
            attempt["stall_note"] = (f"顶在上限上卡住：{measured}、上限 {attempt.get('memory_cap_text') or lib.mebibytes_text(cap)}"
                                     f"（不低于 {STALL_AT_CAP_FRACTION:.0%}），日志 {now - seen[1]:.0f} 秒没涨{busy}")
            self.store.update("attempts", attempt)
            say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试{attempt['stall_note']}：停掉它，按撞上限办")
            threading.Thread(target=self.stop_running, args=(task["id"],), kwargs=self.judged_attempt_only(attempt), daemon=True).start()
            return True
        attempt["stall_note"] = f"日志 {now - seen[1]:.0f} 秒没涨、CPU 这 {now - seen[2]:.0f} 秒也几乎没涨"   # 判卡住的依据记进尝试，结局说明接上它
        self.store.update("attempts", attempt)
        say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试的{attempt['stall_note']}：停掉它，记 stalled")
        threading.Thread(target=self.stop_running, args=(task["id"],), kwargs=self.judged_attempt_only(attempt), daemon=True).start()
        return True

    def judged_attempt_only(self, attempt):
        """判卡住（judge_stall_stop）起的后台线程交给 stop_running 的那几个参数：只停判它的那一次（attempt_id）。
        线程起来到读库之间那一次自己退了、照重试规矩重排起了下一次的，下一次不归它停。弄坏开关 stall-stop-by-task 下不给，按任务号停在跑的全部。"""
        return {} if lib.broken("stall-stop-by-task", self.environment) else {"attempt_id": attempt["id"]}

    def run_hard_limit_verdict(self, task, attempt):
        """这一次尝试要不要照四小时硬上限停（文件头 ⑦「四小时硬上限」）→ 要记进尝试的 {seconds, how} 或 None：带 --full-gate 的不判；累计（run_limit_accumulated）到了 run_hard_limit_seconds 的停；
        子任务另看父任务整件满界了没有（phase=lib.RUN_HARD_LIMIT_PHASE）。各弄坏开关见文件头 ⑦ 那一段。"""
        environment = self.environment
        if lib.broken("run-hard-limit-skipped", environment):
            return None
        # 提交门禁（交的时候带 --full-gate、弹窗由用户确认过的那几道）不受四小时上限管：它判的是整棵树，本来就要跑这么久，拆不了。
        # 别的任务实际用时满四小时一律停，逼提交它的会话拆小再交。
        if self.full_gate_of(task) and not lib.broken("run-hard-limit-full-gate-judged", environment):
            return None
        if self.long_run_confirmed_of(task) and lib.broken("run-hard-limit-confirmed-spared", environment):
            return None
        if lib.broken("run-hard-limit-writers-spared", environment) and self.modifies_workspace(task):
            return None
        total, current, earlier, _ = self.run_limit_accumulated(task, attempt)
        spared_child = task.get("parent_id") and lib.broken("run-hard-limit-children-spared", environment)
        if lib.over_run_hard_limit(total, self.run_hard_limit_seconds, environment) and not spared_child:
            each = "、".join(f"第 {number} 次 {seconds:.0f} 秒" for number, seconds in sorted(earlier)) + ("、" if earlier else "") + f"第 {attempt.get('number')} 次 {current:.0f} 秒（在跑）"
            return {"seconds": round(total, 3), "how": f"各次尝试在跑的时长累计 {total:.0f} 秒（{each}）"}
        if task.get("parent_id") and not lib.broken("run-hard-limit-parent-phase-ignored", environment):
            parent = self.store.task(task["parent_id"])
            if parent is not None and parent[1] and parent[0].get("phase") == lib.RUN_HARD_LIMIT_PHASE:
                return {"seconds": round(total, 3), "how": f"父任务 {task['parent_id']} 整件满界：{(parent[0].get('run_hard_limit_stop') or {}).get('how') or '没记'}"}
        return None

    def judge_run_hard_limit(self, task, attempt):
        """四小时硬上限判这一次尝试（文件头 ⑦「四小时硬上限」）→ 这一轮归它管了没有（管了 check_stall 不再往下判）：判停过的接着停，这一轮判到的记下、停这一次。"""
        if attempt.get("run_hard_limit_stop"):
            self.stop_over_limit_attempt(task, attempt)
            return True
        record = self.run_hard_limit_verdict(task, attempt)
        if record is None:
            return False
        self.note_run_hard_limit_stop(task, attempt, record)
        return True

    def note_run_hard_limit_stop(self, task, attempt, record):
        """这一次尝试判了四小时硬上限：record 记进尝试的 run_hard_limit_stop（结局说明照它写），停这一次。"""
        attempt["run_hard_limit_stop"] = record
        self.store.update("attempts", attempt)
        say(f"  · 任务 {task['id']} 第 {attempt.get('number')} 次尝试{record['how']}，到了 {self.run_hard_limit_seconds:.0f} 秒的硬上限：停这一次，打回")
        self.stop_over_limit_attempt(task, attempt)

    def settle_run_hard_limit(self, task, attempt, exit_code, counts):
        """照四小时硬上限停的那一次尝试停下来了（结局 lib.RUN_HARD_LIMIT_OUTCOME）：任务不重排，照 over-four-hours 收尾（文件头 ⑦「四小时硬上限」）。"""
        record = attempt.get("run_hard_limit_stop") or {}
        self.finalize(task, lib.RUN_HARD_LIMIT_OUTCOME, exit_code,
                      (f"{self.finished_preface(task, attempt, lib.RUN_HARD_LIMIT_OUTCOME)}{record.get('how') or '没记'}，到了 {self.run_hard_limit_seconds:.0f} 秒的硬上限："
                       "服务停了这一次尝试、不重排，--why 放不过"), outcome_counts=counts)

    def parent_run_hard_limit(self, parent, children):
        """分片的父任务整件判不判四小时硬上限（文件头 ⑦「四小时硬上限」）→ 要记进父任务的 {seconds, how} 或 None。整件不另判（用户定「按每一片自己的累计用时算」）：
        各片、计划、合并各按自己的尝试累计（run_hard_limit_verdict），一片满界停那一片、记 over-four-hours，别的片照跑，收齐了整件照那一片的结局收尾（drive_parent_phase
        的「一步没成」）；排队把片一片片串起来的挂钟不算进谁的账。交 None。
        弄坏开关 shard-whole-by-wall-clock 下整件按有片在跑的挂钟算：有片在跑的时段取并集（lib.covered_seconds），片之和不算、全都在排的时段不算，撞了内存上限的那一片
        只从它最后一次撞上限之后那一次起算（同 run_limit_accumulated），带 --full-gate 的不判；run-hard-limit-skipped 下不算，long-run-cap-hits-counted 下撞上限的那几次也算。"""
        environment = self.environment
        if not lib.broken("shard-whole-by-wall-clock", environment) or lib.broken("run-hard-limit-skipped", environment):
            return None
        if self.full_gate_of(parent) and not lib.broken("run-hard-limit-full-gate-judged", environment):
            return None
        now = time.time()
        intervals = []
        for child in children:
            attempts = [record for record, valid in self.store.attempts_of(child["id"]) if valid and record.get("started_at")]
            hits = [int(record.get("number") or 0) for record in attempts if record.get("state") == "ended" and record.get("outcome") in lib.RECORD_CAP_HIT_OUTCOMES]
            floor = max(hits) if hits and not lib.broken("long-run-cap-hits-counted", environment) else 0
            intervals += [(float(record["started_at"]), float(record.get("ended_at") or now)) for record in attempts if int(record.get("number") or 0) > floor]
        seconds = lib.covered_seconds(intervals, environment)
        if not lib.over_run_hard_limit(seconds, self.run_hard_limit_seconds, environment):
            return None
        return {"seconds": round(seconds, 3), "how": f"分片的整件有片在跑的挂钟时长 {seconds:.0f} 秒（片之和不算）"}

    def stop_over_limit_attempt(self, task, attempt):
        """停照四小时硬上限判停的这一次尝试（只停这一次，judged_attempt_only）：后台线程起 stop_running，这一次还在停的不再起；没停成的（stop_running 退了、单元还在跑）下一轮 check_stall 再起，
        服务重启之后 check_stall 照尝试记的 run_hard_limit_stop 接着停。弄坏开关 hard-limit-stop-once 下线程退了也不让再起。"""
        attempt_id = attempt["id"]
        if attempt_id in self.hard_limit_stops:
            return
        self.hard_limit_stops.add(attempt_id)

        def stop():
            try:
                self.stop_running(task["id"], **self.judged_attempt_only(attempt))
            finally:
                if not lib.broken("hard-limit-stop-once", self.environment):
                    self.hard_limit_stops.discard(attempt_id)
        threading.Thread(target=stop, daemon=True).start()

    def long_run_lines(self, task):
        """这条任务（分片的父任务连同各子任务）长跑的几行（task status、列表、推给会话的正文与日志头都打）：确认过的「长跑已确认：<why>」，在跑而满界没确认的「长跑待确认：…」。
        弄坏开关 long-run-confirmed-flag-hidden 下不打确认过的那一行。"""
        records = [task]
        if task.get("sharded_entry"):
            with contextlib.suppress(StoreRefused):
                records += self.children_of(task["id"])
        lines = []
        for record in records:
            part = f"（子任务 {record['id']}）" if record.get("id") != task.get("id") else ""
            flag = record.get("long_run_confirmed")
            if flag and not lib.broken("long-run-confirmed-flag-hidden", self.environment):
                lines.append(f"长跑已确认{part}：{flag.get('why')}")
            elif not flag and record.get("state") == "running":
                attempts = [attempt for attempt, valid in self.store.attempts_of(record["id"]) if valid]
                if self.long_run_unconfirmed_items(record, attempts):
                    lines.append(f"长跑待确认{part}：累计跑满 {self.run_limit_seconds:.0f} 秒没确认，服务照跑、满 {self.run_hard_limit_seconds:.0f} 秒的硬上限照停；范围对、确是要跑这么久的 {lib.long_run_confirm_command(record['id'])}，"
                                 f"不对的 task-server/task cancel {record['id']}、改了范围重交")
        return lines

    def full_gate_of(self, task):
        """这条任务（子任务照父任务）带没带全量门禁标识（task submit --full-gate）。"""
        if task.get("full_gate"):
            return True
        if task.get("parent_id"):
            parent = self.store.task(task["parent_id"])
            return parent is not None and parent[1] and bool(parent[0].get("full_gate"))
        return False

    def long_run_confirmed_of(self, task):
        """这条任务确认过要跑过两小时：自己确认过，或（子任务）父任务确认过（文件头「跑满两小时提醒」）。
        弄坏开关 long-run-why-children-reminded 下子任务不看父任务。"""
        if task.get("long_run_confirmed"):
            return True
        if task.get("parent_id") and not lib.broken("long-run-why-children-reminded", self.environment):
            parent = self.store.task(task["parent_id"])
            return parent is not None and parent[1] and bool(parent[0].get("long_run_confirmed"))
        return False

    def whole_package_of(self, task):
        """这条任务（子任务照父任务）带的「整包已确认」标识：确认文件的路径或空串（文件头 ②「提交时拒」⑥）。"""
        if task.get("whole_package_confirmed"):
            return task["whole_package_confirmed"]
        if task.get("parent_id"):
            parent = self.store.task(task["parent_id"])
            if parent is not None and parent[1]:
                return parent[0].get("whole_package_confirmed") or ""
        return ""

    def flag_lines(self, task):
        """确认标识的行（task status、列表、推给会话的正文与日志头都打）：「整包已确认：<文件>」、「全量门禁」与长跑那几行（long_run_lines），没有的不打。"""
        whole = self.whole_package_of(task)
        return ([f"整包已确认：{whole}"] if whole else []) + (["全量门禁"] if task.get("full_gate") else []) + self.long_run_lines(task)

    def run_limit_accumulated(self, task, attempt):
        """这条任务（分片的子任务按自己的）各次尝试在跑的时长累加（文件头 ①「跑满两小时提醒」）：撞了内存上限重来的计时归零、只从最后一次撞上限之后那一次起算
        （结束了的取记下的 seconds，这一次取此刻已跑的；排队等的不算）。→ (累计秒, 这一次已跑秒, 算进去的前几次 [(第几次, 秒)…], 撞过上限的次数)。
        弄坏开关 long-run-cap-hits-counted 下撞上限的那几次也累加。"""
        current = max(0.0, time.time() - float(attempt.get("started_at") or time.time()))
        ended = sorted((int(record.get("number") or 0), float(record.get("seconds") or 0), record.get("outcome")) for record, valid in self.store.attempts_of(task["id"])
                       if valid and record.get("state") == "ended" and record.get("id") != attempt.get("id"))
        cap_hit_numbers = [number for number, _, outcome in ended if outcome in lib.RECORD_CAP_HIT_OUTCOMES]
        if cap_hit_numbers and not lib.broken("long-run-cap-hits-counted", self.environment):
            ended = [entry for entry in ended if entry[0] > max(cap_hit_numbers)]
        earlier = [(number, seconds) for number, seconds, _ in ended]
        return current + sum(seconds for _, seconds in earlier), current, earlier, len(cap_hit_numbers)

    def run_limit_exempt(self, task):
        """满界不提醒的（文件头 ①「跑满两小时提醒」）：确认过的（long_run_confirmed_of）、交的时候带了全量门禁标识的（full_gate_of，full_gate_refusal 核过）。
        弄坏开关 long-run-unconfirmed-silent 下一律不提醒，long-run-confirmed-still-reminded 下确认过的照提醒，full-gate-flag-ignored 下带了也照提醒，
        whole-package-confirmed-exempts-two-hours 下整包已确认的不提醒。不指定范围的提交时门禁没带全量标识的提交时就被拒（whole_package_refusal），走不到这里。"""
        if lib.broken("long-run-unconfirmed-silent", self.environment):
            return True
        if self.long_run_confirmed_of(task) and not lib.broken("long-run-confirmed-still-reminded", self.environment):
            return True
        if self.whole_package_of(task) and lib.broken("whole-package-confirmed-exempts-two-hours", self.environment):
            return True
        # 提交门禁（带 --full-gate 的）同样不推两小时的拆分提醒：理由同四小时上限那一处（run_hard_limit_verdict）
        return bool(self.full_gate_of(task)) and not lib.broken("full-gate-flag-ignored", self.environment)

    def long_run_unconfirmed_items(self, record, attempts):
        """跑满两小时没确认（文件头 ①「要处置的提醒」的 long-run-unconfirmed）：在跑的那一次尝试连同算进去的前几次累计满 run_limit_seconds、又不在 run_limit_exempt 里的交一项，
        提醒门槛是 1 加撞过上限的次数（撞了上限归零，再满界算跨过一道新门槛）；依据写累计与各次用时、命令、范围声明、日志末几行。只提醒、照跑。"""
        active = [attempt for attempt in attempts if attempt.get("state") in ACTIVE_ATTEMPT_STATES]
        if not active or self.run_limit_exempt(record):
            return []
        attempt = active[-1]
        total, current, earlier, cap_hits = self.run_limit_accumulated(record, attempt)
        if total < self.run_limit_seconds:
            return []
        each = "、".join(f"第 {number} 次 {seconds:.0f} 秒" for number, seconds in sorted(earlier)) + ("、" if earlier else "") + f"第 {attempt['number']} 次 {current:.0f} 秒（在跑）"
        scope = lib.scope_declaration_segments(record.get("environment") or {}, record.get("command") or [])
        tail = [line for line in lib.log_lines(log_tail(attempt.get("log", ""))) if line.strip()][-3:]
        return [self.attention_item(
            "long-run-unconfirmed", 1 + cap_hits, record["id"],
            (f"累计跑了 {total:.0f} 秒（{total / 60:.0f} 分钟），满 {self.run_limit_seconds:.0f} 秒的界：各次 {each}；命令：{shlex.join(record.get('command') or [])}；"
             f"范围声明：{'；'.join(scope) if scope else '没带 SINGLEFS_RUN_SCOPE'}；日志末几行：{' ⏎ '.join(tail) if tail else '日志读不到'}"),
            f"实际用时过了两小时：建议现在就把这一件拆小（点名范围：--item、--check 少几格、点名测试目标，或照服务分片），拆完重交；此刻服务不停它、不改任务；"
            f"累计满 {self.run_hard_limit_seconds:.0f} 秒的硬上限服务直接停掉这一次、记 over-four-hours，那时只能拆了再交")]

    def long_run_sessionless_note(self, task, attempt):
        """没有会话的（人手 --manual）跑满两小时没确认（文件头 ①「要处置的提醒」的 long-run-unconfirmed）：不推，往这一次尝试的日志接一行 lib.LONG_RUN_LOG_MARK 起头的（不算日志在涨），
        每次尝试一行（记 long_run_noted）。弄坏开关 long-run-sessionless-unnoted 下不写。"""
        if task.get("session") or attempt.get("long_run_noted") or lib.broken("long-run-sessionless-unnoted", self.environment):
            return
        items = self.long_run_unconfirmed_items(task, [attempt])
        if not items:
            return
        attempt["long_run_noted"] = True
        self.store.update("attempts", attempt)
        append_log_lines(attempt.get("log", ""), [f"{lib.LONG_RUN_LOG_MARK} 长跑待确认：{items[0]['basis']}；出路：{items[0]['next']}"])

    def note_io_wait(self, attempt, window_started_at, now):
        """日志没涨的这一轮读一次这一次尝试此刻在 D 状态的进程数（Machine.io_blocked），记进 io_waits（文件头 ①「卡在 IO 上」）：
        窗口（window_started_at，log_growth 记的那一个）换了从头记这个窗口里是不是每一轮都有；从哪一刻起每一轮都有跨窗口接着算，有一轮没有或读不到就清掉。"""
        blocked = self.machines[attempt["host"]].io_blocked(attempt)
        previous = self.io_waits.get(attempt["id"])
        fresh = previous is None or previous.window_started_at != window_started_at
        every = (True if fresh else previous.every_poll_blocked) and bool(blocked)
        since = None
        if blocked:
            since = previous.blocked_since if previous is not None and previous.blocked_since is not None else now
        self.io_waits[attempt["id"]] = IoWaitWindow(window_started_at, (0 if fresh else previous.polls) + 1, every, blocked, since)

    def note_activity(self, task, attempt, window_started_at):
        """日志没涨的这一轮读一次这一次尝试的进程内存与峰值（Machine.memory_now），
        记进 activity_windows（activity_window_after；文件头 ①「卡住」的有进展）。"""
        memory, peak = self.machines[attempt["host"]].memory_now(attempt)
        self.activity_windows[attempt["id"]] = activity_window_after(self.activity_windows.get(attempt["id"]), window_started_at, memory, peak)

    def check_suspect(self, task, attempt, cpu, now):
        """疑似卡住（文件头 ①「疑似卡住」）：整个单元的 CPU 满 stall_seconds 平均不到 STALL_CPU_CORES 个核，
        不管日志涨不涨：不停，记进尝试的 suspect、写告警、叫醒在等的 wait（它打 lib.SUSPECT_MARK 那一行、退 lib.SUSPECT_EXIT）。
        一段只报一次；疑似之后 CPU 又涨了（相邻两次读之间平均不少于 STALL_CPU_CORES 个核），这一段结束，从此刻再量下一段。读不到 CPU 不当不涨：窗口作废重量。
        弄坏开关 suspect-unreported 下不判。"""
        if lib.broken("suspect-unreported", self.environment):
            return
        if cpu is None:
            self.cpu_windows.pop(attempt["id"], None)
            return
        window = self.cpu_windows.get(attempt["id"])
        if window is None or cpu < window.last_cpu:
            self.cpu_windows[attempt["id"]] = SuspectWindow(now, cpu, now, cpu, False)
            return
        if window.suspected:
            if (cpu - window.last_cpu) / 1e6 / max(now - window.last_at, 1e-6) >= STALL_CPU_CORES:   # 这一段结束：CPU 又涨了
                self.end_suspect(task, attempt, "CPU 又涨了")
                self.cpu_windows[attempt["id"]] = SuspectWindow(now, cpu, now, cpu, False)
                return
            self.cpu_windows[attempt["id"]] = dataclasses.replace(window, last_at=now, last_cpu=cpu)
            return
        if now - window.started_at < self.stall_seconds:
            self.cpu_windows[attempt["id"]] = dataclasses.replace(window, last_at=now, last_cpu=cpu)
            return
        cores = (cpu - window.started_cpu) / 1e6 / max(now - window.started_at, 1e-6)
        if cores >= STALL_CPU_CORES:   # 在算：从此刻再量一个窗口
            self.cpu_windows[attempt["id"]] = SuspectWindow(now, cpu, now, cpu, False)
            return
        self.cpu_windows[attempt["id"]] = dataclasses.replace(window, last_at=now, last_cpu=cpu, suspected=True)
        previous = attempt.get("suspect") or {}
        attempt["suspect"] = {"active": True, "segment": int(previous.get("segment") or 0) + 1, "at": round(time.time(), 3),
                              "window_seconds": round(now - window.started_at, 3), "cores": round(cores, 6), "threshold_cores": STALL_CPU_CORES,
                              "why": f"整个单元的 CPU 这 {now - window.started_at:.0f} 秒平均 {cores:.4f} 个核，不到 {STALL_CPU_CORES} 个核（日志涨不涨都不看）"}
        # 先写告警、再记进尝试：wait 那一侧读到 suspect 时告警已在
        self.alert("suspected-stall", task["id"], f"任务 {task['id']} 第 {attempt['number']} 次尝试疑似卡住：{attempt['suspect']['why']}；服务不停它",
                   f"提交它的会话自己查（task-server/task log {task['id']}、task-server/task status {task['id']}），还在算就再起 task-server/task wait {task['id']} 接着等，"
                   f"真卡住了 task-server/task cancel {task['id']}")
        self.store.update("attempts", attempt)
        self.notify_finished(task["id"])   # 叫醒在等的 wait：它照 suspect 打一行、退出

    def unreported_suspect(self, task_id, session):
        """wait 那一侧调：这条任务在跑的那一次正疑似卡住、这一段还没报给提交它的会话，交那一段（连同第几次、哪台、日志），不然交 None。
        报给提交它的会话就记下这一段报过了（记在服务的内存里：只有排队线程写尝试记录，不和它抢着写）；别的会话来等照样看得到、不记。
        弄坏开关 suspect-reported-every-wait 下报过的照报。"""
        found = self.store.task(task_id)
        if found is None:
            return None
        for attempt, valid in self.store.attempts_of(task_id):
            suspect = attempt.get("suspect") or {}
            if not valid or attempt.get("state") not in ACTIVE_ATTEMPT_STATES or not suspect.get("active"):
                continue
            with self.suspect_lock:
                if self.suspect_reported.get(attempt["id"]) == suspect.get("segment") and not lib.broken("suspect-reported-every-wait", self.environment):
                    continue
                recorded = found[0].get("session") == session
                if recorded:
                    self.suspect_reported[attempt["id"]] = suspect.get("segment")
            if recorded:
                self.persist_restart_surviving_state("suspect_reported")
            return dict(suspect, attempt=attempt.get("number"), host=attempt.get("host"), log=attempt.get("log"))

    def end_suspect(self, task, attempt, why):
        suspect = dict(attempt.get("suspect") or {}, active=False, ended_at=round(time.time(), 3), ended_because=why)
        attempt["suspect"] = suspect
        self.store.update("attempts", attempt)
        say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试的疑似卡住（第 {suspect.get('segment')} 段）结束：{why}")

    def attention_items(self, task_id):
        """这条任务（分片的连同子任务）此刻要处置的提醒（文件头 ①「要处置的提醒」）：每项 {kind, threshold, task, basis, why, next}（时长判的另有 reminded_after_seconds），task 是提醒说的那一条
        （子任务的是子任务号）。不改任务；十九种判法在 attention_of_record（预算贴着整台读 budget_check 排队时记下的那一项）与下面的分片推不动（改主工作区的那一种另起一次 apply-writer-patch.py --read-outcome，workspace_write_item），提醒门槛按 reminder_threshold。
        弄坏开关 attention-shard-advance-unreported 下不报分片父任务推不动（别几种的开关在 attention_of_record）。"""
        found = self.store.task(task_id)
        if found is None or not found[1]:
            return []
        records = [found[0]]
        if found[0].get("sharded_entry"):
            try:
                records += self.children_of(task_id)
            except StoreRefused:   # 名下有校验不过的子任务行：不认那几行（summary 与 cancel 已写告警）
                pass
        now = time.time()
        items = []
        for record in records:
            items += self.attention_of_record(record, now)
        failure = dict(self.advance_failures.get(task_id) or {})
        if failure and found[0].get("state") == "sharded" and not lib.broken("attention-shard-advance-unreported", self.environment):
            items.append(self.attention_item("shard-advance-failed", int(failure.get("episode") or 1), task_id,
                                             f"推下一步连着出错 {failure.get('count')} 轮：{failure.get('why')}",
                                             "服务每一轮对账都重推，一直推不动整件就停在原地"))
        return items

    def attention_item(self, kind, threshold, task_id, basis, why, follow="", reminded_after=None):
        """要处置的提醒的一项（文件头 ①「要处置的提醒」）：threshold 是跨过的提醒门槛；时长判的那几种给 reminded_after（这一回跨过的那道门槛，秒），记成 reminded_after_seconds。
        next 是异常表里这一种的修改意见（lib.anomaly_advice），follow 给了的接在后面（这一回的具体数）。每一种都只经这里拼 next，不另写出路句。"""
        advice = lib.anomaly_advice(kind, task_id, self.environment)
        item = {"kind": kind, "threshold": threshold, "task": task_id, "basis": basis, "why": why, "next": f"{advice}；{follow}" if follow else advice}
        if reminded_after is not None:
            item["reminded_after_seconds"] = round(reminded_after)
        return item

    def quiet_log_items(self, record, attempts):
        """日志长期不涨而服务没停它（文件头 ①「要处置的提醒」的 quiet-log-busy）：在跑的那一次尝试的日志（log_content 算的那部分）距上一次增长
        满 quiet_attention_seconds 交一项，不然交空表。日志多久没涨照判卡住记的那一份（log_growth，宽限里还没量的不报）。
        只在服务最近一个窗口因 CPU 在涨或读不到 CPU 没停它时报（log_growth 记的缘由是 SPARED_CPU_BUSY、SPARED_CPU_UNREAD）：按 D 状态免停的由
        io_wait_items 报，还没判过一个窗口的不报。弄坏开关 quiet-log-busy-any-exemption 下不看缘由。
        日志不涨而有进展的不算：多久没涨从日志上一次涨与最近一回读到进展（progress_noted：子进程写着的文件在涨；note_attempt_progress 记）里晚的那一刻起算，
        读到进展之后不满门槛的不报；弄坏开关 quiet-log-busy-progress-ignored 下不看进展、只从日志上一次涨起算。"""
        task_id = record["id"]
        active = [attempt for attempt in attempts if attempt.get("state") in ACTIVE_ATTEMPT_STATES]
        seen = self.log_growth.get(active[-1]["id"]) if active else None
        if seen is None:
            return []
        if seen[4] not in (SPARED_CPU_BUSY, SPARED_CPU_UNREAD) and not lib.broken("quiet-log-busy-any-exemption", self.environment):
            return []
        now = time.monotonic()
        noted = None if lib.broken("quiet-log-busy-progress-ignored", self.environment) else self.progress_noted.get(active[-1]["id"])
        quiet = now - (max(seen[1], noted[0]) if noted is not None else seen[1])
        crossed = reminder_threshold(quiet, self.quiet_attention_seconds)
        if not crossed:
            return []
        since_progress = f"，上一回读到进展（{noted[1]}）是 {now - noted[0]:.0f} 秒前" if noted is not None else ""
        return [self.attention_item(
            "quiet-log-busy", crossed, task_id,
            (f"第 {active[-1].get('number')} 次尝试的日志 {now - seen[1]:.0f} 秒（{(now - seen[1]) / 60:.0f} 分钟）没涨{since_progress}，"
             f"日志与进展满 {next_reminder_seconds(crossed, self.quiet_attention_seconds):.0f} 秒没动再提醒一回"),
            (f"日志不涨而{'读不到 CPU' if seen[4] == SPARED_CPU_UNREAD else 'CPU 在涨'}的服务不停它：日志与 CPU 都不涨才停"),
            (f"还在算就再起 task-server/task wait {task_id}，"
             f"在空转或在回收里打转 task-server/task cancel {task_id}、改了重交"),
            self.quiet_attention_seconds * 2 ** (crossed - 1))]

    def io_wait_items(self, record, attempts):
        """卡在 IO 上而服务没停它（文件头 ①「要处置的提醒」的 io-wait）：在跑的那一次尝试日志没涨的每一轮都读到有进程在 D 状态（io_waits 的 blocked_since），
        连着满 io_wait_attention_seconds 交一项，不然交空表。依据写最近一轮 D 状态的进程数与本机此刻 /proc/pressure/io 的 some avg60。"""
        task_id = record["id"]
        active = [attempt for attempt in attempts if attempt.get("state") in ACTIVE_ATTEMPT_STATES]
        waited = self.io_waits.get(active[-1]["id"]) if active and not active[-1].get("stalled") else None   # 已判停的那一次在等停下来，不再报
        if waited is None or waited.blocked_since is None:
            return []
        blocked_for = time.monotonic() - waited.blocked_since
        crossed = reminder_threshold(blocked_for, self.io_wait_attention_seconds)
        if not crossed:
            return []
        host = lib.HOST_NAMES.get(active[-1].get("host"), active[-1].get("host"))
        return [self.attention_item(
            "io-wait", crossed, task_id,
            (f"第 {active[-1].get('number')} 次尝试（{host}）日志没涨，连着 {blocked_for:.0f} 秒（{blocked_for / 60:.0f} 分钟）每一轮都有进程在 D 状态"
             f"（不可中断的 IO 等待），最近一轮 {waited.blocked_now} 个；本机此刻 /proc/pressure/io {io_pressure_text()}；"
             f"连着满 {next_reminder_seconds(crossed, self.io_wait_attention_seconds):.0f} 秒再提醒一回"),
            "日志不涨而每一轮都有进程在等 IO（D 状态）：多半在等盘或 NFS，服务不因为日志不涨判它卡住、不停它，IO 缓过来它自己接着跑",
            (f"这一次在{host}：等得起再起 task-server/task wait {task_id}，"
             f"盘与 NFS 修不好 task-server/task cancel {task_id}、IO 缓过来再重交"),
            self.io_wait_attention_seconds * 2 ** (crossed - 1))]

    def unit_unreadable_items(self, record, attempts):
        """读不到单元状态（文件头 ①「要处置的提醒」的 unit-unreadable）：在跑的那几次尝试里对账连着读不到单元状态满 unit_unreadable_attention_seconds 的各交一项
        （提醒门槛按连着读不到多久），依据写尝试派在哪台、本机读不到哪个单元的状态、不经 systemd 最近一回核到的；不然交空表。"""
        task_id, now, items = record["id"], time.monotonic(), []
        for attempt in attempts:
            entry = self.unit_unreadable.get(attempt.get("id")) if attempt.get("state") in ACTIVE_ATTEMPT_STATES else None
            if entry is None:
                continue
            elapsed = now - entry["since"]
            crossed = reminder_threshold(elapsed, self.unit_unreadable_attention_seconds)
            if not crossed:
                continue
            verdict, detail = self.unit_presence_now(attempt["id"])
            host = lib.HOST_NAMES.get(attempt.get("host"), attempt.get("host"))
            items.append(self.attention_item(
                "unit-unreadable", crossed, task_id,
                (f"第 {attempt.get('number')} 次尝试（派在{host}）：本机读不到它的单元 {attempt.get('unit')} 的状态（systemctl --user show 起不来、超时或没打 ActiveState），"
                 f"连着 {elapsed:.0f} 秒；不经 systemd 最近一回核到{lib.PRESENCE_WORDS[verdict]}：{detail}；"
                 f"读不到的连着满 {next_reminder_seconds(crossed, self.unit_unreadable_attention_seconds):.0f} 秒再提醒一回"),
                "读不到单元状态时服务判不出它在不在：不杀、不收尾、不记结局，尝试照记 running；核到 cgroup 读得到且空、记下的进程号都不在才记 launcher-lost，核不出就一直等",
                f"这一次的单元是 {attempt.get('unit')}", self.unit_unreadable_attention_seconds * 2 ** (crossed - 1)))
        return items

    def cores_short_of_threads(self, cores, threads):
        """到此刻平均在用 cores 个核算不算没用满 threads 线程（LONG_RUN_IDLE_CORE_FRACTION）。弄坏开关 long-run-idle-quarter-threshold 下界是四分之一。"""
        fraction = LONG_RUN_IDLE_QUARTER_BREAK if lib.broken("long-run-idle-quarter-threshold", self.environment) else LONG_RUN_IDLE_CORE_FRACTION
        return cores < threads * fraction

    def long_run_idle_items(self, record, attempts, now):
        """跑得久而核没用起来（文件头 ①「要处置的提醒」的 long-run-idle-cores）：在跑的那一次尝试跑满 long_run_attention_seconds、到此刻平均在用核数（Machine.cpu_usec 除以已跑秒数，
        同 running_progress）没用满给的线程数（threads_of；cores_short_of_threads），交一项，提醒门槛是尝试的次数（每次尝试只报一次，不随时长再报）；读不到 CPU 的不报。只提醒、照跑。"""
        active = [attempt for attempt in attempts if attempt.get("state") in ACTIVE_ATTEMPT_STATES]
        if not active:
            return []
        attempt = active[-1]
        elapsed = now - float(attempt.get("started_at") or now)
        if elapsed < self.long_run_attention_seconds:
            return []
        machine = self.machines.get(attempt.get("host"))
        usage = machine.cpu_usec(attempt) if machine is not None else None
        threads = int(self.threads_of(record) or 0)
        if usage is None or threads <= 0:
            return []
        cores = usage / 1e6 / elapsed
        if not self.cores_short_of_threads(cores, threads):
            return []
        peak = machine.memory_now(attempt)[1]
        task_id = record["id"]
        return [self.attention_item(
            "long-run-idle-cores", max(1, int(attempt.get("number") or 1)), task_id,
            (f"第 {attempt.get('number')} 次尝试已跑 {elapsed:.0f} 秒（{elapsed / 60:.0f} 分钟），给 {threads} 线程、平均只用 {cores:.1f} 核"
             f"（不到 {threads * LONG_RUN_IDLE_CORE_FRACTION:.0f} 核），进程内存峰值 {lib.size_text(peak)}"),
            f"跑满 {self.long_run_attention_seconds / 60:.0f} 分钟而核没用满：资源运用是否合理、脚本能不能并行或切片？（服务照跑，不停它）",
            f"照跑就再起 task-server/task wait {task_id}")]

    def fills_any_cap_items(self, record):
        """按给的上限开并行（文件头 ①「要处置的提醒」的 fills-any-cap）：这条命令键最近 FILLS_CAP_RUNS 次跑成的尝试（只读进程内存口径的行，current_metric）
        峰值都不低于各自上限的 FILLS_CAP_FRACTION，交一项（每条任务只报一次）；不到这么多次的不报。撞了上限的那几次不算：撞的那一次峰值本来就贴着上限，
        照上限一档档撞上去的（任务 9616 那条静态核 6.0、9.0、13.5 GiB）不是按上限开并行。上限本来就照记下的峰值定的那几次（尝试记的 prediction 一栏是 lib.RECORD_HISTORY）也不算：
        峰值贴着上限是定法的结果，命令未必读上限。弄坏开关 fills-any-cap-counts-cap-hits 下撞了上限的也算进那几次，fills-any-cap-counts-record-caps 下照记录定的上限也算。"""
        counted = lib.RECORD_COMPLETED_OUTCOMES + (lib.RECORD_CAP_HIT_OUTCOMES if lib.broken("fills-any-cap-counts-cap-hits", self.environment) else ())
        runs = []
        for attempt, valid in self.store.select("attempts", "command_key = ? AND state = 'ended'", (record.get("command_key"),), order="id DESC LIMIT 50"):
            if not valid or not self.current_metric(attempt):
                continue
            if attempt.get("prediction") == lib.RECORD_HISTORY and not lib.broken("fills-any-cap-counts-record-caps", self.environment):
                continue
            outcome, peak, cap = attempt.get("outcome"), attempt.get("memory_peak"), attempt.get("memory_cap")
            if outcome not in counted or not isinstance(peak, int) or not isinstance(cap, int) or cap <= 0:
                continue
            runs.append((float(attempt.get("ended_at") or 0), peak, cap))
        runs = sorted(runs, reverse=True)[:FILLS_CAP_RUNS]
        if len(runs) < FILLS_CAP_RUNS or any(peak < cap * FILLS_CAP_FRACTION for _, peak, cap in runs):
            return []
        human = self.host_dispatch.human
        task_id = record["id"]
        shown = "、".join(f"上限 {human(cap)} 峰值 {human(peak)}" for _, peak, cap in runs)
        return [self.attention_item(
            "fills-any-cap", 1, task_id, f"这条命令键最近 {FILLS_CAP_RUNS} 次：{shown}（每次都不低于上限的 {FILLS_CAP_FRACTION:.0%}）",
            "给多少就用满多少：命令多半照服务交的上限（TASK_SERVER_MEMORY_CAP_BYTES）定同时起的个数，记录跟着上限涨，起步一次比一次大",
            f"照跑就再起 task-server/task wait {task_id}")]

    def repeated_cap_hits_item(self, record, state, attempts):
        """连着撞上限还在重排（文件头 ①「要处置的提醒」的 repeated-cap-hits）：在排、或下一次在跑，末尾连着 lib.LOOP_LEAK_REMINDER_CAP_HITS 次已收尾的尝试结局都是
        memory-cap-hit 的交一项（每条任务只报一次），不然交空表。在跑而一次在跑、在起的尝试都没有的（刚收尾、settle_attempt 还没定重排还是打回）不判：
        那一刻打回的照结局的 loop_leak= 报，不另报这一条。项另带 loop_leak（lib.loop_leak_check_text），attention_text 与 lib.attention_line 照它另写一段。"""
        if state == "running" and not any(attempt.get("state") in ACTIVE_ATTEMPT_STATES for attempt in attempts):
            return []
        cap_hits = lib.trailing_cap_hits(attempts)
        if len(cap_hits) < lib.LOOP_LEAK_REMINDER_CAP_HITS:
            return []
        task_id = record["id"]
        numbers = f"第 {cap_hits[0].get('number')}–{cap_hits[-1].get('number')} 次"
        item = self.attention_item(
            "repeated-cap-hits", 1, task_id, f"末尾连着 {len(cap_hits)} 次尝试撞了内存上限（{numbers}），服务照乘 1.5 重排、还没到两台最大",
            "要的内存一次比一次多的，多半是代码在循环里越攒越多或把全量整份读进内存，不是范围大；放大上限只是推迟到撞两台最大才打回",
            f"服务照旧乘 1.5 重排，照跑就再起 task-server/task wait {task_id}")
        return [dict(item, loop_leak=lib.loop_leak_check_text(cap_hits))]

    def current_metric(self, attempt):
        """这一次尝试的峰值是不是这一版的口径（task_server_lib.py 文件头「资源记录」的峰值口径一栏）：peak_metric 是「进程内存」的才算（task_server_lib.py 的 current_peak_metric，control key-stats 同用）。
        弄坏开关 old-metric-peaks-read 下照改前不看这一栏，旧行也算。"""
        return lib.current_peak_metric(attempt) or lib.broken("old-metric-peaks-read", self.environment)

    def attention_of_record(self, record, now):
        """一条任务记录（父任务或子任务）此刻要处置的提醒：排队过久、日志长期不涨而服务没停它、卡在 IO 上而服务没停它、
        读不到单元状态、清理没收尾。
        弄坏开关 attention-queue-unreported、attention-quiet-busy-unreported、attention-io-wait-unreported、
        attention-cleanup-unreported、repeated-cap-hits-silent、unit-unreadable-unreminded 各关一种。"""
        task_id, state = record["id"], record.get("state")
        attempts = [attempt for attempt, valid in self.store.attempts_of(task_id) if valid]
        items = []
        if state == "queued" and not lib.broken("attention-queue-unreported", self.environment):
            since = max([float(record.get("submitted_at") or now)] + [float(attempt.get("ended_at") or 0) for attempt in attempts])   # 重排的从上一次尝试结束起算
            waited = now - since
            crossed = reminder_threshold(waited, self.queue_attention_seconds)
            why = self.waiting_text(record)
            # 排队原因还没判出来（服务刚起来、这一轮还没判到它）的这一轮不报，等判出来那一轮再报（文件头 ①「要处置的提醒」）；弄坏开关 attention-queue-reasonless-sent 下照改前照报
            if crossed and (why or lib.broken("attention-queue-reasonless-sent", self.environment)):
                items.append(self.attention_item(
                    "queued-too-long", crossed, task_id,
                    f"已排 {waited:.0f} 秒（{waited / 60:.0f} 分钟），排满 {next_reminder_seconds(crossed, self.queue_attention_seconds):.0f} 秒再提醒一回",
                    why or "排队原因此刻读不到（这一轮还没判到它）",
                    f"等得起再起 task-server/task wait {task_id}", self.queue_attention_seconds * 2 ** (crossed - 1)))
        if state == "running" and not lib.broken("attention-quiet-busy-unreported", self.environment):
            items += self.quiet_log_items(record, attempts)
        if state == "running" and not lib.broken("attention-io-wait-unreported", self.environment):
            items += self.io_wait_items(record, attempts)
        if state == "running" and not lib.broken("unit-unreadable-unreminded", self.environment):
            items += self.unit_unreadable_items(record, attempts)
        if state in ("queued", "running") and task_id in self.budget_notices:   # 排队那一刻判出的预算贴着整台（budget_check）
            items.append(dict(self.budget_notices[task_id]))
        if state == "running" and not lib.broken("long-run-idle-silent", self.environment):
            items += self.long_run_idle_items(record, attempts, now)
        if state == "running":
            items += self.threads_over_registered_items(record, attempts)
        if state == "running":
            items += self.long_run_unconfirmed_items(record, attempts)
        if state in ("queued", "running") and not lib.broken("fills-any-cap-silent", self.environment):
            items += self.fills_any_cap_items(record)
        if state in ("queued", "running") and not lib.broken("repeated-cap-hits-silent", self.environment):
            items += self.repeated_cap_hits_item(record, state, attempts)
        if not lib.broken("attention-cleanup-unreported", self.environment):
            for attempt in attempts:
                if attempt.get("cleanup_pending") and attempt.get("state") in ACTIVE_ATTEMPT_STATES:
                    left_children = PEER_SCOPE_LEFTOVERS_MARK in attempt["cleanup_pending"]   # 第二台停了 scope 还留着进程（confirm_peer_cleanup）
                    items.append(self.attention_item(
                        "cleanup-pending", max(1, int(attempt.get("number") or 1)), task_id,
                        f"第 {attempt.get('number')} 次尝试的命令已退出，清理没收尾：{attempt['cleanup_pending']}",
                        ("命令退出之后它那一层内存包装的 scope 里还有进程，停了 scope 也没清掉；服务保留这一次的资源占用、每隔几秒重查" if left_children
                         else "服务保留这一次的资源占用、每隔几秒重查，查清之前任务不记结局"),
                        ("这条命令退出后还留着子进程（依据里列着第二台 scope 里留下的进程）" if left_children
                         else "留下的也可能是第二台的远端连接：清理回执没收尾的看日志里 multi-host-run.sh ⑥ 那几行")))
        if state == "finished" and not record.get("parent_id") and not lib.broken("workspace-write-interrupted-unreported", self.environment):
            item = self.workspace_write_item(record, attempts)
            if item is not None:
                items.append(item)
        if state == "finished" and not lib.broken("long-run-split-silent", self.environment):
            items += self.long_run_split_items(record, attempts)
        if state == "finished" and not record.get("parent_id") and not lib.broken("workspace-write-blocked-unreported", self.environment):
            items += self.workspace_write_blocked_items(record, attempts)
        return items

    def settled_run_seconds(self, attempts):
        """一条出了结局的任务（分片的一片按自己的）服务记的用时（.claude/rules/long-run-split.md「判据」）：收尾了的各次尝试记下的 seconds 相加，
        撞了内存上限、照大一档重来的，从最后一次撞上限之后那一次起算（撞上限的那几次和更早的不算；同 run_limit_accumulated）。→ (秒, 算进去的几次, 撞过上限的次数)。
        弄坏开关 long-run-split-cap-hits-counted 下撞上限的那几次也累加。"""
        ended = sorted((int(attempt.get("number") or 0), float(attempt.get("seconds") or 0), attempt.get("outcome"))
                       for attempt in attempts if attempt.get("state") == "ended")
        cap_hit_numbers = [number for number, _, outcome in ended if outcome in lib.RECORD_CAP_HIT_OUTCOMES]
        if cap_hit_numbers and not lib.broken("long-run-split-cap-hits-counted", self.environment):
            ended = [entry for entry in ended if entry[0] > max(cap_hit_numbers)]
        return sum(seconds for _, seconds, _ in ended), len(ended), len(cap_hit_numbers)

    def long_run_split_exempt(self, record):
        """这条任务在不在 .claude/long-run-split-exempt（用户点名豁免的项）里：表里每行 `<项名>  # <理由>` 的项名，等于这条命令的某个词（--item 后面的项、--test 后面的测试名、
        `--item=项` 这种带等号的等号后面、逗号分隔的每一段）就算。表读不到当没有。弄坏开关 long-run-split-exempt-ignored 下不看这张表。"""
        if lib.broken("long-run-split-exempt-ignored", self.environment):
            return False
        path = lib.selftest_override("TASK_SERVER_SELFTEST_LONG_RUN_SPLIT_EXEMPT", self.environment) or os.path.join(self.peer_root, LONG_RUN_SPLIT_EXEMPT_FILE)
        try:
            with open(path, encoding="utf-8") as handle:
                names = {line.split("#", 1)[0].strip() for line in handle if line.strip() and not line.lstrip().startswith("#")} - {""}
        except OSError:
            return False
        words = set()
        for word in record.get("command") or []:
            words.add(word)
            for part in re.split(r"[=,]", word):
                words.add(part)
        return bool(names & words)

    def long_run_split_items(self, record, attempts):
        """出结局时用时超过 lib.LONG_RUN_SPLIT_SECONDS 的要拆（文件头 ①「要处置的提醒」的 long-run-split；.claude/rules/long-run-split.md）：任务（分片的一片按自己的，
        分片的父任务没有自己的尝试、不判整件的挂钟）settled_run_seconds 超过界、又不在豁免表（long_run_split_exempt）里的交一项，只推一回。
        依据写这是按片或按整条任务的用时判的（日志里取不出那一项的起止，拿片或整条任务的用时代替）。弄坏开关 long-run-split-silent 下不报。"""
        seconds, counted, cap_hits = self.settled_run_seconds(attempts)
        if seconds <= lib.LONG_RUN_SPLIT_SECONDS or self.long_run_split_exempt(record):
            return []
        what = f"第 {record.get('shard_index')} 片（子任务 {record['id']}）" if record.get("role") == "shard" else (
            f"{record.get('role')} 子任务 {record['id']}" if record.get("parent_id") else f"整条任务 {record['id']}")
        return [self.attention_item(
            "long-run-split", 1, record["id"],
            (f"{what}服务记的用时 {seconds:.0f} 秒（{seconds / 3600:.1f} 小时），超过 {lib.LONG_RUN_SPLIT_SECONDS} 秒；这是按{'片' if record.get('role') == 'shard' else '整条任务'}的用时判的，"
             f"不是日志里某一项的起止（取不出那一项的起止，拿片或整条任务的用时代替）；算了 {counted} 次尝试，撞内存上限重来的 {cap_hits} 次从最后一次撞上限之后那一次起算"),
            "规则 .claude/rules/long-run-split.md「判据」：一项一次运行的用时超过 7200 秒就要拆；撞没撞四小时上限、结局是绿是红、填没填过 --why、确认没确认长跑都不改这条判据",
            "这一件已经跑完、结局照常用；不豁免的由用户点名，登记在 .claude/long-run-split-exempt")]

    def workspace_write_blocked_items(self, record, attempts):
        """换钩子被上一趟没写完成的挡住（文件头 ①「要处置的提醒」的 workspace-write-blocked）：命令是 scripts/gate/hook-swap.py（不带 --feed-only）、最后一次尝试结局 command-failed 退 7
        （hook-swap.py 退出码 7：上一趟换上没写完成，拒绝再换、一个字节没写）的，交一项报给被挡的这个会话，依据点名上一趟疑似是哪条任务（库里更早的、改主工作区的换钩子命令里，
        结局是被杀这类没正常结束的，取最近的一条）；日志里的拒绝说明逐份点名状态。弄坏开关 workspace-write-blocked-unreported 下不报。"""
        if record.get("outcome") != "command-failed" or record.get("exit_code") != WORKSPACE_BLOCKED_EXIT_CODE or "--feed-only" in (record.get("command") or []):
            return []
        root = os.path.realpath(self.peer_root)
        cwd = os.path.realpath(record.get("cwd") or root)
        if not any(os.path.relpath(os.path.normpath(os.path.join(cwd, word)), root) == HOOK_SWAP_ENTRY for word in record.get("command") or [] if not word.startswith("-")):
            return []
        earlier = None
        for other, valid in self.store.select("tasks", "state = 'finished' AND id < ?", (record["id"],), order="id DESC LIMIT 100"):
            if (valid and not other.get("parent_id") and self.modifies_workspace(other)
                    and (other.get("outcome") in WORKSPACE_WRITER_INTERRUPTED_OUTCOMES or (other.get("outcome") == "command-failed" and other.get("exit_code") in SIGNAL_EXIT_CODES))
                    and HOOK_SWAP_ENTRY in [os.path.relpath(os.path.normpath(os.path.join(os.path.realpath(other.get("cwd") or root), word)), root)
                                            for word in other.get("command") or [] if not word.startswith("-")]):
                earlier = other
                break
        who = (f"上一趟疑似是任务 {earlier['id']}（结局 {earlier.get('outcome')}，退 {earlier.get('exit_code')}）" if earlier is not None
               else "上一趟是哪条任务服务在库里没查到（可能不是经服务起的，或更早）")
        return [self.attention_item(
            "workspace-write-blocked", 1, record["id"],
            f"换钩子的命令退 7：拒绝再换、一个字节没写；{who}",
            "hook-swap.py 起手先读结局文件：上一趟换上没写完成（被杀或回滚不成，几份文件停在半截），之后所有会话的换上都被挡，直到有人补写完成或退回",
            f"这一条的日志（task-server/task log {record['id']}）里拒绝说明逐份点名状态与怎么退回")]

    def workspace_write_item(self, record, attempts):
        """改主工作区的命令（modifies_workspace）没正常结束：结局是 WORKSPACE_WRITER_INTERRUPTED_OUTCOMES 之一、或命令被信号杀（command-failed 而退出码在 SIGNAL_EXIT_CODES 里）时，
        可能写了一半，交一项 workspace-write-interrupted（点名要核的文件或补丁目录）；正常结束的、不改工作区的交 None。用户：「这个问题很严重 因为可能会导致补丁没有正常写入」。
        apply-writer-patch.py 有结局文件（补丁目录里的 apply-outcome.tsv）的照它判：起 --read-outcome，打全了、已退回的不报，没打完、打完之后又被改了、读不了的报（连同它打的那几行）；
        结局文件不在的不报（它在动手换之前写开始与文件行，不在就是还没开始换）；带 --dry-run 这类只读参数的不报。结了局的任务只核一次（workspace_write_checks）。
        弄坏开关 apply-outcome-ignored 下不读结局文件，一律照可能写了一半报。"""
        task_id = record["id"]
        if task_id in self.workspace_write_checks:
            return self.workspace_write_checks[task_id]
        outcome, exit_code = record.get("outcome"), record.get("exit_code")
        interrupted = outcome in WORKSPACE_WRITER_INTERRUPTED_OUTCOMES or (outcome == "command-failed" and exit_code in SIGNAL_EXIT_CODES)
        if outcome == lib.RUN_HARD_LIMIT_OUTCOME and (not attempts or lib.broken("run-hard-limit-writer-unreminded", self.environment)):
            interrupted = False   # 自己一次尝试都没起的（分片的父任务照片收尾），主工作区没动（文件头「四小时硬上限」）
        item = None
        if interrupted and self.modifies_workspace(record):
            item = self.workspace_write_judged(record, attempts, outcome, exit_code)
        self.workspace_write_checks[task_id] = item
        return item

    def workspace_write_judged(self, record, attempts, outcome, exit_code):
        task_id, command, cwd = record["id"], list(record.get("command") or []), record.get("cwd") or self.peer_root
        root = os.path.realpath(self.peer_root)
        positions = [index for index, word in enumerate(command) if not word.startswith("-")
                     and os.path.relpath(os.path.normpath(os.path.join(os.path.realpath(cwd), word)), root) in WORKSPACE_WRITERS]
        position = positions[0] if positions else 0
        writer = command[position] if positions else (command[0] if command else "")
        after = command[position + 1:]
        named = [word for index, word in enumerate(after) if not word.startswith("-") and (index == 0 or after[index - 1] != "--root")]
        number = max([int(attempt.get("number") or 0) for attempt in attempts] or [0])
        basis = f"第 {number} 次尝试结局 {outcome}（退 {exit_code}）" + ("，命令被信号杀" if outcome == "command-failed" else "")
        status_hint = f"git -C {shlex.quote(cwd)} status --short 与 git -C {shlex.quote(cwd)} diff --stat 看主工作区此刻改了哪几份"
        next_step = (f"要核的是{('这几处：' + '、'.join(named)) if named else '它改的文件'}；{status_hint}；"
                     f"task-server/task log {task_id} 看它停在哪一步")
        if os.path.relpath(os.path.normpath(os.path.join(os.path.realpath(cwd), writer)), root) == WRITER_PATCH_ENTRY:
            if any(word in PATCH_WRITER_READ_ONLY_ARGUMENTS for word in after):
                return None   # 只读的用法（--dry-run、--pending-status 这类）不写主工作区
            if not named:   # --finish-pending、--rollback-pending：照记下的暂存换，看暂存还在不在
                next_step = f"python3 {shlex.quote(writer)} --pending-status 看有没有没换完的，照它打的出路接着换完或退回；{status_hint}"
            else:
                outcome_file = os.path.join(cwd, named[0], PATCH_WRITER_OUTCOME_NAME)
                if not os.path.exists(outcome_file) and not lib.broken("apply-outcome-ignored", self.environment):
                    return None   # 结局文件在动手换之前写：不在就是还没开始换，主工作区没动
                read_back = [sys.executable, os.path.join(cwd, writer) if not os.path.isabs(writer) else writer, "--read-outcome", outcome_file]
                if "--root" in after[:-1]:
                    read_back += ["--root", after[after.index("--root") + 1]]
                if not lib.broken("apply-outcome-ignored", self.environment):
                    try:
                        completed = subprocess.run(read_back, cwd=cwd, capture_output=True, text=True, env=self.environment, stdin=subprocess.DEVNULL,
                                                   timeout=PATCH_WRITER_CHECK_SECONDS)
                        verdict, lines = completed.returncode, (completed.stdout + completed.stderr).strip().splitlines()
                    except (OSError, subprocess.TimeoutExpired) as error:
                        verdict, lines = 2, [f"--read-outcome 起不来或没在 {PATCH_WRITER_CHECK_SECONDS} 秒里退：{type(error).__name__}: {error}"]
                    if verdict in (0, 1):
                        return None   # 打全了、已退回：主工作区不在半截上
                    basis += f"；结局文件 {outcome_file} 判{PATCH_WRITER_VERDICTS.get(verdict, f'退 {verdict}')}：" + " / ".join(line.strip() for line in lines[-6:])
                next_step = (f"python3 {shlex.quote(writer)} --read-outcome {shlex.quote(outcome_file)} 逐个文件看已换上与没换上，"
                             f"再照 python3 {shlex.quote(writer)} --pending-status 的出路接着换完或退回；补丁目录 {named[0]}；{status_hint}")
        return self.attention_item("workspace-write-interrupted", 1, task_id, basis, f"这条命令改主工作区（{writer}），没正常结束，可能写了一半", next_step)

    def stored_reminded_thresholds(self, task_id):
        """库里这一条任务的行记着的报到的提醒门槛（正文的 reminded_thresholds，{种类: 报到的最高门槛}；文件头 ①「要处置的提醒」）：行不在、校验不过、写法不对的那几项交空。
        弄坏开关 attention-reported-memory-only 下不读库、交空（改前的样子）。"""
        if lib.broken("attention-reported-memory-only", self.environment):
            return {}
        found = self.store.task(task_id)
        stored = found[0].get("reminded_thresholds") if found is not None and found[1] else None
        if not isinstance(stored, dict):
            return {}
        return {kind: threshold for kind, threshold in stored.items() if isinstance(kind, str) and isinstance(threshold, int) and threshold > 0}

    def reminder_history_of(self, task_id, kind):
        """这条任务这一种提醒报过的那几回 [{threshold, repush, at, basis, round}…]（旧的在前）：内存里有的照内存，没有的读库里任务行的 reminder_history（服务重启之后）；都没有交 []。"""
        with self.attention_lock:
            remembered = list(self.reminder_histories.get((task_id, kind)) or [])
        if remembered or lib.broken("attention-reported-memory-only", self.environment):
            return remembered
        found = self.store.task(task_id)
        stored = found[0].get("reminder_history") if found is not None and found[1] else None
        entries = stored.get(kind) if isinstance(stored, dict) else None
        return [entry for entry in entries if isinstance(entry, dict)] if isinstance(entries, list) else []

    def attention_round(self, item):
        """这一项（attention_items 的项）这一次推出去是第几回、上一回是什么时刻 → (回数, 时刻或 None)：这条任务这一种提醒报过的最后一回（reminder_history_of）的回数加一，没报过的是第 1 回。"""
        history = self.reminder_history_of(item["task"], item["kind"])
        if not history:
            return 1, None
        return int(history[-1].get("round") or len(history)) + 1, history[-1].get("at")

    def attention_repush(self, item):
        """提醒门槛不随时长涨的提醒（ANOMALY_REPUSHED_KINDS）同一道门槛报过之后还在（文件头 ⑥「没处置时」）：这道门槛头一回报出去满 anomaly_repush_seconds 再推第 1 回、
        之后每翻一倍再推（reminder_threshold），→ 此刻该推第几回（不该推交 0）；报过的那几回照 reminder_history_of。弄坏开关 anomaly-not-repushed 下交 0。"""
        if lib.broken("anomaly-not-repushed", self.environment):
            return 0
        history = [entry for entry in self.reminder_history_of(item["task"], item["kind"]) if entry.get("threshold") == item["threshold"]]
        if not history:
            return 0
        first = min(float(entry.get("at") or 0) for entry in history)
        due = reminder_threshold(time.time() - first, self.anomaly_repush_seconds)
        done = max(int(entry.get("repush") or 0) for entry in history)
        return due if due > done else 0

    def anomaly_acknowledged(self, item):
        """这一项（attention_items 的项）那条任务行里记了照跑（anomaly_acknowledged 有这一种；confirm --why 记的）。弄坏开关 anomaly-acknowledgement-ignored 下一律当没记。"""
        if item["kind"] not in ACKNOWLEDGED_KINDS or lib.broken("anomaly-acknowledgement-ignored", self.environment):
            return False
        found = self.store.task(item["task"])
        return bool(found and found[1] and item["kind"] in (found[0].get("anomaly_acknowledged") or {}))

    def acknowledge_anomalies(self, task_id, session, process_id, stated):
        """confirm --why 时这条任务（分片的连同子任务）此刻开着的、出路是照跑的提醒（ACKNOWLEDGED_KINDS）逐条记进那条任务行的 anomaly_acknowledged，→ 记了的种类（排过序）。"""
        kinds = set()
        # 开着的提醒，连同库里记着报过的（budget_notices 只在内存：服务换上之后在跑的那一条就没有开着的 oversized-budget，报过的照样认）
        reported = [] if lib.broken("anomaly-acknowledge-open-only", self.environment) else [
            {"task": task_id, "kind": kind} for kind in self.stored_reminded_thresholds(task_id) if kind in ACKNOWLEDGED_KINDS]
        for item in self.attention_items(task_id) + reported:
            if item["kind"] not in ACKNOWLEDGED_KINDS:
                continue
            with self.store.task_lock(item["task"]):
                found = self.store.task(item["task"])
                if not (found and found[1]):
                    continue
                marks = dict(found[0].get("anomaly_acknowledged") or {})
                marks.setdefault(item["kind"], {"why": stated, "at": round(time.time(), 3)})
                self.store.change("tasks", item["task"], {"anomaly_acknowledged": marks})
            kinds.add(item["kind"])
        if kinds:
            self.operation("anomaly-acknowledge", task_id, session, process_id, f"{'、'.join(sorted(kinds))}：{stated[:200]}")
        return sorted(kinds)

    def reminded_threshold_of(self, item):
        """这一项（attention_items 的项）报给提交它的会话的最高提醒门槛：内存里那一份与库里那一份取大的，读到库里的并进内存（服务重启之后头一次读库）。"""
        key = (item["task"], item["kind"])
        with self.attention_lock:
            remembered = self.reminded_thresholds.get(key, 0)
        if remembered >= item["threshold"]:
            return remembered
        stored = self.stored_reminded_thresholds(item["task"]).get(item["kind"], 0)
        with self.attention_lock:
            self.reminded_thresholds[key] = max(self.reminded_thresholds.get(key, 0), stored)
            return self.reminded_thresholds[key]

    def note_attention_reported(self, items):
        """这几项报给了提交它的会话：记进内存，再逐条任务记进库里那一行的 reminded_thresholds 与 reminder_history（Store.change 只并进这两栏、门槛只升不降；文件头 ①「要处置的提醒」）。
        库写不进（行不在、校验不过、库出错）的说一句、只记内存。弄坏开关 attention-reported-memory-only 下只记内存（改前的样子）。"""
        reached_by_task, logged = {}, {}
        now = round(time.time(), 3)
        rounds = {(item["task"], item["kind"]): self.attention_round(item)[0] for item in items}
        with self.attention_lock:
            for item in items:
                key = (item["task"], item["kind"])
                self.reminded_thresholds[key] = max(self.reminded_thresholds.get(key, 0), item["threshold"])
                reached_by_task.setdefault(item["task"], {})[item["kind"]] = self.reminded_thresholds[key]
                # 这一回报了什么（reminder_history：再推的正文照它写前几次推了什么，attention_repush 照它算再推到第几回）
                entry = {"threshold": item["threshold"], "repush": int(item.get("repush") or 0), "at": now, "basis": str(item.get("basis") or "")[:200], "round": rounds[key]}
                self.reminder_histories[key] = (self.reminder_histories.get(key, []) + [entry])[-ATTENTION_HISTORY_KEPT:]
                logged.setdefault(item["task"], {}).setdefault(item["kind"], []).append(entry)
        if lib.broken("attention-reported-memory-only", self.environment):
            return
        for task_id, reached in reached_by_task.items():
            def merged(record, reached=reached, entries=logged.get(task_id, {})):
                stored = record.get("reminded_thresholds") if isinstance(record.get("reminded_thresholds"), dict) else {}
                history = record.get("reminder_history") if isinstance(record.get("reminder_history"), dict) else {}
                return {"reminded_thresholds": dict(stored, **{kind: max(stored[kind] if isinstance(stored.get(kind), int) else 0, threshold) for kind, threshold in reached.items()}),
                        "reminder_history": dict(history, **{kind: (list(history.get(kind) or []) + added)[-ATTENTION_HISTORY_KEPT:] for kind, added in entries.items()})}
            try:
                self.store.change("tasks", task_id, merged)
            except (StoreRefused, sqlite3.Error) as error:
                say(f"  ! 任务 {task_id} 报到的提醒门槛没记进库（{error}），只记在内存里：服务重启之后这几道门槛会再报一次")

    def unreported_attention(self, task_id, session):
        """wait 那一侧调：这条任务此刻要处置的提醒里还没报给提交它的会话、或又跨过一道提醒门槛的，交这些项（attention_items 的项）；没有交空表。
        报给提交它的会话就记下这一种报到了这道门槛（内存里与库里各一份，note_attention_reported；服务重启后照库认、不再报）；别的会话来等照样看得到、不记。
        内存里那一份的判与记在同一把锁里：两个 wait 一起来只报一次。弄坏开关 attention-reported-every-wait 下报过的照报。"""
        found = self.store.task(task_id)
        if found is None:
            return []
        owner = found[0].get("session") == session
        reported, noted = [], []
        for item in self.attention_items(task_id):
            key = (item["task"], item["kind"])
            self.reminded_threshold_of(item)   # 先把库里那一份并进内存
            with self.attention_lock:
                if self.reminded_thresholds.get(key, 0) >= item["threshold"] and not lib.broken("attention-reported-every-wait", self.environment):
                    continue
                if owner:
                    self.reminded_thresholds[key] = max(self.reminded_thresholds.get(key, 0), item["threshold"])
                    noted.append(item)
            reported.append(item)
        if noted:
            self.note_attention_reported(noted)
        return reported

    # ── ⑥ 发消息给会话（文件头 ⑥）──
    def inbox_loop(self):
        """发消息那一路：每 inbox_round_seconds 一轮（notify_finished 当场叫醒）：先发记下了结局的，再扫在排、在跑的任务的疑似卡住与要处置的提醒，最后盯服务健康。
        一轮出事不停服务：打出来、下一轮再判。"""
        while not self.stopping.is_set():
            self.inbox_event.clear()
            with self.inbox_lock:
                self.inbox_sockets.clear()   # 收件 socket 每一轮照会话登记现找（inbox_socket_of）：会话重启换了 socket 的下一轮就照新的发
            try:
                self.push_outcomes()
                self.repush_unclaimed_outcomes()
                self.push_reminders()
                self.watch_health()
            except Exception:
                say("  ! 这一轮发消息出了异常（下一轮再判）：\n" + traceback.format_exc())
            self.inbox_event.wait(self.inbox_round_seconds)

    def task_log_path(self, task):
        """这条任务最后一次尝试的日志（分片的父任务自己没有尝试，取子任务里最后起的那一次）；一次都没起交空串。"""
        attempts = [attempt for attempt, valid in self.store.attempts_of(task["id"]) if valid]
        if not attempts and task.get("sharded_entry"):
            with contextlib.suppress(StoreRefused):
                attempts = [attempt for child in self.children_of(task["id"]) for attempt, valid in self.store.attempts_of(child["id"]) if valid]
        return max(attempts, key=lambda attempt: float(attempt.get("started_at") or 0)).get("log", "") if attempts else ""

    def inbox_socket_of(self, task):
        """这条任务此刻该往哪个收件 socket 发（文件头 ⑥「收件地址」）→ (路径, 找不到时的那一句)：按任务记的会话号读会话登记（lib.session_inbox_socket：
        会话号相同、进程还活着的那一份的 messagingSocketPath），一轮里同一个会话只读一次（inbox_sockets，inbox_loop 每一轮开头清掉）；令牌照交任务时带的。
        弄坏开关 inbox-socket-from-submit-only 下照交任务时记的那一个 socket（会话重启换了 socket 就发不到）。"""
        if lib.broken("inbox-socket-from-submit-only", self.environment):
            submitted = (task.get("inbox") or {}).get("socket", "")
            return submitted, ("" if submitted else "交任务时没记收件 socket")
        session = task.get("session") or ""
        with self.inbox_lock:
            known = self.inbox_sockets.get(session)
        if known is None:
            known = lib.session_inbox_socket(session, self.environment)
            with self.inbox_lock:
                self.inbox_sockets[session] = known
        return known

    def inbox_reachable(self, task):
        """这条任务记着会话令牌、此刻照会话号找得到收件 socket：提醒、疑似卡住与服务健康只发给这样的（找不到的不算一回发失败，wait 照取；结局另照 push_outcomes 发、发不出写告警）。"""
        return bool((task.get("inbox") or {}).get("token")) and bool(self.inbox_socket_of(task)[0])

    def send_to_session(self, task, text, event):
        """往提交 task 的会话的收件 socket（inbox_socket_of 照会话号现找的，令牌是任务记着的）发 text → 发出去了没有。发不出去（找不到活着的会话、socket 不在、连不上、
        会话已关）写一行告警：服务日志、告警记录与这条任务最后一次尝试的日志里 lib.INBOX_LOG_MARK 起头的一行；event 记进 inbox_failed，不重试到死；任务照旧。
        弄坏开关 inbox-not-sent 下一条都不发、交 False。"""
        if lib.broken("inbox-not-sent", self.environment):
            return False
        inbox = task.get("inbox") or {}
        headline = text.splitlines()[0]
        socket_path, missing = self.inbox_socket_of(task)
        problem = missing or post_to_inbox(socket_path, inbox.get("token", ""), text, self.environment)
        session = task.get("session")
        if not problem:
            say(f"  · 发给会话 {session}：{headline}")
            return True
        if event is not None:
            with self.inbox_lock:
                self.inbox_failed.add(event)
        line = f"往会话 {session} 的收件 socket {socket_path or '（照会话号没找到）'} 发「{headline}」没发出去（{problem}）；不再重试，任务照旧"
        next_step = f"会话已关的不用管；会话还在的用 task-server/task wait {task['id']} 取结局、task-server/task status {task['id']} 看在等什么"
        append_log_lines(self.task_log_path(task), [f"{lib.INBOX_LOG_MARK} ✗ {line}；→ 怎么办：{next_step}"])
        try:
            self.alert("inbox-unreachable", task["id"], line, next_step)
        except Exception:   # 库写不进（服务健康里的那一样）时告警记录写不进：服务日志照打
            say(f"  ✗ 告警（inbox-unreachable）任务 {task['id']}：{line}")
            say(f"     → 怎么办：{next_step}")
        return False

    def push_outcomes(self):
        """记下了结局的（inbox_pending）：发给提交它的会话，先发还没报的提醒（改主工作区的没正常结束这类，push_attention），再发结局；
        结局发出去了记成已取（同 wait 把结局交给提交它的会话）。结局只发一次（处理完的任务号不再放回：发出去记了已取的、发不出去写了告警的，同文件头 ⑥ 不重试）；
        会话已经用 wait 取了的不发；子任务不发（父任务的结局发）。处理某一条时抛错（读库、summary、push_attention、记已取），那一条与排在它后面还没处理的号放回
        inbox_pending，异常照旧交给 inbox_loop，下一轮接着发（unsent 记还没处理完的）。
        弄坏开关 pushed-outcome-not-collected 下发出去了也不记已取；inbox-pending-dropped-on-error 下抛错时照改前不放回。"""
        with self.inbox_lock:
            pending, self.inbox_pending = self.inbox_pending, set()
        unsent = sorted(pending)
        try:
            while unsent:
                task_id = unsent[0]
                found = self.store.task(task_id)
                task = found[0] if found is not None and found[1] else None
                if (task is not None and not task.get("parent_id") and (task.get("inbox") or {}).get("token")
                        and task.get("state") == "finished" and not task.get("collected")):
                    self.push_attention(task, final=True)
                    if (self.send_to_session(task, self.outcome_text(self.summary(task_id)), ("outcome", task_id))
                            and not lib.broken("pushed-outcome-not-collected", self.environment)):
                        self.mark_collected(task_id)
                unsent.pop(0)
        finally:
            if unsent and not lib.broken("inbox-pending-dropped-on-error", self.environment):
                with self.inbox_lock:
                    self.inbox_pending.update(unsent)

    def repush_unclaimed_outcomes(self):
        """结局没人取（文件头 ⑥「结局没人取」；异常表 unclaimed-outcome；首行 outcome_headline 写第 N 回与上一回在哪一天，上一回的时刻记在任务行的 outcome_repushed_at）：父任务记着会话令牌、结局满 unclaimed_warning_seconds 还没交给提交它的会话（collected 为假：
        发结局的那一条没送到，也没起 wait 取）的再推一回，之后每翻一倍再推（门槛照 reminder_threshold）；正文写这条结局还没人取、它是什么（outcome_text 整段接在后面）。
        推出去了记成已取（同 push_outcomes）；推到第几回记进任务行的 outcome_repushed（内存里另留一份 unclaimed_repushed），服务重启后不重推同一回。
        收件 socket 照会话号现找（send_to_session），会话重启换了 socket 的这一回照新的推。弄坏开关 unclaimed-outcome-not-repushed 下不再推。"""
        if lib.broken("unclaimed-outcome-not-repushed", self.environment):
            return
        now = time.time()
        for task, valid in self.store.select("tasks", "state = 'finished' AND json_extract(record, '$.collected') = 0"):
            if not valid or task.get("parent_id") or task.get("collected") or not (task.get("inbox") or {}).get("token"):
                continue
            waited = now - float(task.get("finished_at") or now)
            crossed = reminder_threshold(waited, self.unclaimed_warning_seconds)
            done = max(self.unclaimed_repushed.get(task["id"], 0), int(task.get("outcome_repushed") or 0))
            if crossed <= done:
                continue
            self.unclaimed_repushed[task["id"]] = crossed
            previous_at = task.get("outcome_repushed_at") or task.get("finished_at")   # 上一回推出去的时刻：头一回再推的上一回是结局那一条（记结局的时候发的）
            with contextlib.suppress(StoreRefused, sqlite3.Error):
                self.store.change("tasks", task["id"], {"outcome_repushed": crossed, "outcome_repushed_at": round(now, 3)})
            reply = self.summary(task["id"])
            text = "\n".join([outcome_headline(task["id"], task.get("outcome"), task.get("exit_code"), crossed + 1, previous_at, unclaimed=True),
                              f"依据：结局记下 {waited:.0f} 秒（{waited / 60:.0f} 分钟）了，还没交给提交它的会话（发结局的那一条没送到，也没起 task wait 取）",
                              f"此刻还差：{lib.anomaly_advice('unclaimed-outcome', task['id'], self.environment)}（之后每翻一倍再推一回）",
                              "这条结局是：", self.outcome_text(reply)])
            if (self.send_to_session(task, text, ("unclaimed-outcome", task["id"], crossed))
                    and not lib.broken("pushed-outcome-not-collected", self.environment)):
                self.mark_collected(task["id"])

    def push_reminders(self):
        """在排、在跑（连同分片的父任务）而此刻照会话号找得到收件 socket 的任务（inbox_reachable）：疑似卡住与要处置的提醒里还没报的，逐项发给提交它的会话。"""
        for task, valid in self.store.select("tasks", "state != 'finished'"):
            if not valid or task.get("parent_id") or not self.inbox_reachable(task):
                continue
            self.push_suspects(task)
            self.push_attention(task)
            self.push_progress(task, time.time())

    def push_progress(self, task, now):
        """在跑进度（文件头 ⑥「在跑进度」）：这条任务（分片的连同子任务）有尝试在跑，从最早那一次起跑（发过的从上一回发）满 progress_push_seconds（58 分钟），
        给提交它的会话发一条：已跑多久、分片的各片几片结束几片在跑几片在排、每一次在跑的尝试在哪一台、已跑多久、平均在用几核、日志多久没涨与日志末行。
        只报、不停、不要会话处置；发没发出去都记这一回的时刻（发不出去的不每轮重发）。弄坏开关 progress-not-pushed 下不发。"""
        if lib.broken("progress-not-pushed", self.environment):
            return
        try:
            children = self.children_of(task["id"])
        except StoreRefused:
            return
        names = {task["id"]} | {child["id"] for child in children}
        running = sorted((attempt for attempt in self.store.running_attempts() if attempt.get("task_id") in names and attempt.get("started_at")),
                         key=lambda attempt: float(attempt["started_at"]))
        if not running:
            return
        first = float(running[0]["started_at"])
        last = max(first, self.progress_pushed.get(task["id"], 0.0), float(task.get("progress_pushed_at") or 0.0))
        if now - last < self.progress_push_seconds:
            return
        rounds = int(task.get("progress_pushed_rounds") or 0) + 1
        self.progress_pushed[task["id"]] = now
        with contextlib.suppress(StoreRefused, sqlite3.Error):
            self.store.change("tasks", task["id"], {"progress_pushed_at": round(now, 3), "progress_pushed_rounds": rounds})
        lines = [f"{INBOX_HEADLINE}{headline_class('进度', '任务还在跑，不是结局')}任务 {task['id']} 起跑以来 {(now - first) / 60:.0f} 分钟"
                 f"（第 {rounds} 回，每 {self.progress_push_seconds / 60:.0f} 分钟报一回；只报进度，不用处置）"]
        if children:
            finished = sum(1 for child in children if child.get("state") == "finished")
            started = {attempt.get("task_id") for attempt in running} - {task["id"]}
            lines.append(f"子任务 {len(children)} 条：{finished} 条已结束、{len(started)} 条在跑、{len(children) - finished - len(started)} 条在排")
        for attempt in running[:PROGRESS_PUSH_LISTED_ATTEMPTS]:
            lines.append(self.progress_attempt_line(task, attempt, now))
        if len(running) > PROGRESS_PUSH_LISTED_ATTEMPTS:
            lines.append(f"· 另有 {len(running) - PROGRESS_PUSH_LISTED_ATTEMPTS} 次在跑的没列")
        lines.append(f"要看细的：task-server/task status {task['id']}、task-server/task log {task['id']}")
        self.send_to_session(task, "\n".join(lines), None)

    def progress_attempt_line(self, task, attempt, now):
        """在跑进度那一条里一次在跑的尝试那一行（push_progress）：在哪一台、起跑以来多久、
        平均在用几核、日志多久没涨（日志不涨时读到过进展的接着写读到的是哪份文件在涨与多少秒前读的：running_progress 的 progress_note）、日志末行。
        弄坏开关 progress-line-progress-unnamed 下不写读到的进展。"""
        progress = self.running_progress(task, attempt, now)
        cores, quiet = progress.get("cores"), progress.get("log_quiet_seconds")
        tail = [line.strip() for line in log_tail(attempt.get("log", "")).splitlines() if line.strip()]
        note, age = progress.get("progress_note"), progress.get("progress_age_seconds")
        note_text = (f"（日志不涨而有进展：{note}，{age:.0f} 秒前读到）" if note and age is not None and not lib.broken("progress-line-progress-unnamed", self.environment)
                     else "")
        return (f"· 任务 {attempt.get('task_id')} 在 {attempt.get('host')}：起跑以来 {progress['elapsed_seconds'] / 60:.0f} 分钟、"
                f"平均在用 {'量不到' if cores is None else f'{cores:.1f}'} 核、日志 {'量不到多久' if quiet is None else f'{quiet:.0f} 秒'}没涨{note_text}；"
                f"日志末行：{tail[-1][:PROGRESS_PUSH_TAIL_CHARACTERS] if tail else '（日志还是空的）'}")

    def push_attention(self, task, final=False):
        """这条任务（分片的连同子任务）要处置的提醒里还没报给提交它的会话、也没发失败过的，发给它；记账与 wait 同一份 reminded_thresholds
        （同一种同一道门槛只报一次：这里发过的 wait 不再报，wait 报过的这里不再发）。说的是子任务的那几项按 (种类, 提醒门槛, 原因) 合成一条（attention_groups，
        文件头 ⑥「要处置的提醒」）：发出去了每一项都记报过，发不出去每一项都记发失败过。合成的那一组头一轮先压一轮（attention_held）：各片各记提交时刻、相差几毫秒，
        同一刻跨过门槛的兄弟片跨过一轮的边界会拆成两条，下一轮连同新跨过门槛的一起发（组里有一项压过就发）；final（发结局之前那一次，push_outcomes）不压，没有下一轮了。
        弄坏开关 inbox-reported-every-round 下报过的照发。"""
        per_shard = lib.broken("inbox-attention-per-shard", self.environment)
        pending = []
        for item in self.attention_items(task["id"]):
            reported = self.reminded_threshold_of(item) >= item["threshold"] and not lib.broken("inbox-reported-every-round", self.environment)
            if reported and item["kind"] in ANOMALY_REPUSHED_KINDS and not self.anomaly_acknowledged(item):   # 门槛不随时长涨的：同一道门槛推过之后一直还在，满时限再推（attention_repush）；记了照跑的不再推
                repush = self.attention_repush(item)
                if repush:
                    item, reported = dict(item, repush=repush), False
            with self.inbox_lock:
                failed = attention_event(item) in self.inbox_failed
            if not (reported or failed):
                pending.append(item)
        for group in self.attention_groups(task, pending):
            events = [attention_event(item) for item in group]
            if group[0].get("task") != task["id"] and not per_shard and not final:
                with self.attention_lock:
                    held = any(event in self.attention_held for event in events)
                    self.attention_held.update(events)
                if not held:
                    continue
            text = self.attention_text(task, group[0]) if len(group) == 1 else self.merged_attention_text(task, group)
            sent = self.send_to_session(task, text, events[0])
            with self.attention_lock:
                self.attention_held.difference_update(events)
            if sent:
                self.note_attention_reported(group)
            if not sent:
                with self.inbox_lock:
                    self.inbox_failed.update(events)

    def attention_groups(self, task, items):
        """发给会话的提醒怎么合（文件头 ⑥「要处置的提醒」）：说的是父任务自己的一项一组；说的是子任务的，种类、提醒门槛、原因都相同的合成一组（分片排久那一种，
        几十片同一刻跨过同一道门槛、原因逐字相同），组里照任务号从小到大。→ [[项…]…]，次序照每组头一项在 items 里的次序。task wait 那一侧不经这里、照旧逐项报。
        弄坏开关 inbox-attention-per-shard 下照改前一项一组（逐片各发一条）。"""
        if lib.broken("inbox-attention-per-shard", self.environment):
            return [[item] for item in items]
        groups = {}
        for index, item in enumerate(items):
            key = ("own", index) if item.get("task") == task["id"] else ("children", item.get("kind"), item.get("threshold"), item.get("repush", 0), item.get("why"))
            groups.setdefault(key, []).append(item)
        return [sorted(group, key=lambda item: int(item.get("task") or 0)) for group in groups.values()]

    def merged_attention_text(self, task, group):
        """几片子任务合成的那一条提醒的正文（attention_groups）：第一行「task-server：提醒（…，不是结局）任务 <父任务号> <中文名>（<种类>，子任务 <号的区间>，共 <几> 片；第 <N> 回）」，
        再写依据（照号最小的那一片）、原因、出路（照号最小的那一片，别的片换成那一片的号；那一片的项带 loop_leak 的另起一行写那一段）、日志（那一片的），
        末尾是那一片的 TASK_SERVER_ATTENTION 一行原样。"""
        first = group[0]
        found = self.store.task(first["task"])
        log = self.task_log_path(found[0]) if found is not None and found[1] else ""
        repushed = f"，没处置、第 {first['repush']} 回再推" if first.get("repush") else ""
        round_number, previous_at = self.attention_round(first)
        subject = found[0] if found is not None and found[1] else None
        return "\n".join([reminder_headline(task, first, round_number, previous_at, f"{repushed}，子任务 {task_id_ranges([item['task'] for item in group])}，共 {len(group)} 片", subject),
                          *self.repush_lines(first),
                          f"依据：{first.get('basis')}（子任务 {first['task']}；同一道提醒门槛、同一原因的各片合成这一条）", f"原因：{first.get('why')}",
                          f"出路：{first.get('next')}（以子任务 {first['task']} 为例，别的片换成那一片的号）",
                          *([first["loop_leak"]] if first.get("loop_leak") else []), f"日志：{log or '无（还没起过）'}",
                          lib.attention_line(task["id"], first)])

    def push_suspects(self, task):
        """这条任务（分片的连同子任务）在跑的尝试正疑似卡住、这一段还没报的，发给提交它的会话；记账与 wait 同一份 suspect_reported（一段只报一次）。
        弄坏开关 inbox-reported-every-round 下报过的照发。"""
        records = [task]
        if task.get("sharded_entry"):
            with contextlib.suppress(StoreRefused):
                records += self.children_of(task["id"])
        for record in records:
            for attempt, valid in self.store.attempts_of(record["id"]):
                suspect = attempt.get("suspect") or {}
                if not valid or attempt.get("state") not in ACTIVE_ATTEMPT_STATES or not suspect.get("active"):
                    continue
                event = ("suspect", attempt["id"], suspect.get("segment"))
                with self.suspect_lock:
                    reported = self.suspect_reported.get(attempt["id"]) == suspect.get("segment") and not lib.broken("inbox-reported-every-round", self.environment)
                with self.inbox_lock:
                    failed = event in self.inbox_failed
                if reported or failed:
                    continue
                if self.send_to_session(task, self.suspect_text(task, record, attempt, suspect), event):
                    with self.suspect_lock:
                        self.suspect_reported[attempt["id"]] = suspect.get("segment")
                    self.persist_restart_surviving_state("suspect_reported")

    def outcome_text(self, reply):
        """结局那一条的正文：第一行「task-server：结局（任务已结束）任务 <号> <中文名>（<种类>，退 <码>；第 1 回）」（outcome_headline），再写原因、出路、确认标识那几行（flag_lines）、
        日志（照 wait 打的 TASK_SERVER_REASON 的 why、next 与末行的 log），末尾是 wait 的末行原样。
        连着撞上限之后打回的（理由行带 loop_leak=，task_server_lib.py 文件头「打回的理由」）在「出路：」之后另起一行写那一段。"""
        task = reply["task"]
        fields = dict(lib.reply_reason_fields(reply))
        attempts = lib.outcome_attempts(reply)
        flags_first = lib.broken("inbox-flag-lines-before-evidence", self.environment)
        return "\n".join([outcome_headline(task["id"], task.get("outcome"), task.get("exit_code")), *(self.flag_lines(task) if flags_first else []),
                          f"原因：{fields.get('why')}", f"出路：{fields.get('next')}", *([fields["loop_leak"]] if fields.get("loop_leak") else []), *([] if flags_first else self.flag_lines(task)),
                          f"日志：{(attempts[-1].get('log') if attempts else '') or '无（一次都没起）'}", lib.reply_result_line(reply, self.environment)])

    def attention_text(self, task, item):
        """要处置的提醒那一条的正文：第一行「task-server：提醒（任务还在排 / 还在跑，不是结局）任务 <号> <中文名>（<种类>；第 <N> 回）」（reminder_headline；重推的接「上一回在 <日期>」），
        再写依据、原因、出路、确认标识那几行（flag_lines）、日志（照 wait 打的 TASK_SERVER_ATTENTION），末尾是那一行原样；
        项带 loop_leak 的（repeated-cap-hits）在「出路：」之后另起一行写那一段。"""
        part = f"，子任务 {item['task']}" if item.get("task") != task["id"] else ""
        repushed = f"，没处置、第 {item['repush']} 回再推" if item.get("repush") else ""
        found = self.store.task(item["task"])
        log = self.task_log_path(found[0]) if found is not None and found[1] else ""
        round_number, previous_at = self.attention_round(item)
        flags_first = lib.broken("inbox-flag-lines-before-evidence", self.environment)
        subject = found[0] if found is not None and found[1] else None
        return "\n".join([reminder_headline(task, item, round_number, previous_at, f"{repushed}{part}", subject), *(self.flag_lines(task) if flags_first else []), *self.repush_lines(item),
                          f"依据：{item.get('basis')}", f"原因：{item.get('why')}",
                          f"出路：{item.get('next')}", *([item["loop_leak"]] if item.get("loop_leak") else []), *([] if flags_first else self.flag_lines(task)), f"日志：{log or '无（还没起过）'}",
                          lib.attention_line(task["id"], item)])

    def repush_lines(self, item):
        """再推的那一回（item 带 repush）正文另写的两行：之前推过什么（reminder_history_of 里这一种最近几回：隔了多久、当时的依据），此刻还差什么（这一行的修改意见）；
        不是再推的交 []。"""
        if not item.get("repush"):
            return []
        now = time.time()
        history = self.reminder_history_of(item["task"], item["kind"])[-3:]
        shown = "；".join(f"{(now - float(entry.get('at') or now)) / 60:.0f} 分钟前报过"
                         + (f"（第 {entry['repush']} 回再推）" if entry.get("repush") else "") + f"：{entry.get('basis')}" for entry in history)
        return [f"之前推过：{shown or '库里读不到推过的记录'}",
                f"此刻还差：{item.get('next')}（满 {self.anomaly_repush_seconds * 2 ** (item['repush'] - 1):.0f} 秒还没人处置，服务再推这一回）"]

    def suspect_text(self, task, record, attempt, suspect):
        """疑似卡住那一条的正文：第一行「task-server：疑似卡住（任务还在跑、服务没停它）任务 <号> 疑似卡住（suspected-stall，第 <段> 段；第 1 回）」（suspect_headline），再写依据、出路、日志（照 wait 打的 TASK_SERVER_SUSPECT），末尾是那一行原样。"""
        part = f"，子任务 {record['id']}" if record["id"] != task["id"] else ""
        shown = dict(suspect, attempt=attempt.get("number"), host=attempt.get("host"), log=attempt.get("log"))
        return "\n".join([suspect_headline(task, suspect.get("segment"), part), f"依据：{suspect.get('why')}",
                          f"出路：任务照跑、服务不停它；{lib.anomaly_advice('suspected-stall', task['id'], self.environment)}（结局照发）",
                          f"日志：{attempt.get('log') or '无'}", lib.suspect_line(task["id"], shown)])

    def runaway_loop(self):
        """任务之外的进程空转（文件头 ⑥）单独一个线程：读第二台要过 ssh，不挡发消息那一路。一轮出事不停服务：打出来、下一轮再判。"""
        while not self.stopping.wait(self.runaway_scan_seconds):
            try:
                self.watch_runaway_processes()
            except Exception:
                say("  ! 这一轮扫任务之外的进程出了异常（下一轮再判）：\n" + traceback.format_exc())

    def process_listings(self):
        """两台此刻的进程表（lib.PROCESS_LIST_COLUMNS 那几列）→ [(台, 输出)]；读不到的那台不在里面。自证里只读 TASK_SERVER_SELFTEST_PROCESS_LIST 点名的文件（当本机的）。"""
        override = lib.selftest_override("TASK_SERVER_SELFTEST_PROCESS_LIST", self.environment)
        if override or lib.fast(self.environment):
            with contextlib.suppress(OSError):
                if override:
                    with open(override, encoding="utf-8") as handle:
                        return [(LOCAL, handle.read())]
            return []
        listings = []
        with contextlib.suppress(OSError, subprocess.TimeoutExpired):
            completed = subprocess.run(["ps", "-U", str(os.getuid()), "-o", lib.PROCESS_LIST_COLUMNS], capture_output=True, text=True, env=self.environment,
                                       stdin=subprocess.DEVNULL, timeout=PROCESS_LIST_SECONDS)
            if completed.returncode == 0:
                listings.append((LOCAL, completed.stdout))
        peer = self.resources.peer
        if peer is not None and peer.available and not self.resources.peer_cooling_down():   # 第二台上一回读不成还在冷却里的不去连
            code, output, _ = peer.run(f'ps -U "$(id -u)" -o {lib.PROCESS_LIST_COLUMNS}', timeout=PROCESS_LIST_SECONDS)
            if code == 0:
                listings.append((PEER, output))
        return listings

    def registered_session_processes(self):
        """本机此刻活着的 Claude Code 进程 → {进程号: 会话号}；登记读不了交空表。"""
        try:
            registry = lib.session_registry_library()
            directory = lib.selftest_override(lib.SESSIONS_DIRECTORY_OVERRIDE, self.environment) or registry.registry_directory({})
            return {entry.pid: entry.session_id for entry in registry.session_registry_entries(directory) or [] if entry.is_alive}
        except Exception:   # 载不进、读的时候抛错：这一回当没有
            return {}

    def runaway_protected_pids(self, listing):
        """本机不碰的进程号：登记的 Claude Code 进程与它们的祖先、服务自己与它的祖先。"""
        parents = lib.process_parents(listing)
        protected = set()
        for pid in [*self.registered_session_processes(), os.getpid()]:
            while pid and pid not in protected:
                protected.add(pid)
                pid = parents.get(pid, 0)
        return protected

    def runaway_owner(self, pid, listing):
        """本机这个进程属于哪个会话：顺着父进程找到登记的 Claude Code 进程，交会话号；找不到交空串。"""
        parents = lib.process_parents(listing)
        sessions = self.registered_session_processes()
        seen = set()
        pid = parents.get(pid, 0)   # 从父进程起找：会话进程自己空转的不算它名下的进程，不停它
        while pid in parents and pid not in seen:
            if pid in sessions:
                return sessions[pid]
            seen.add(pid)
            pid = parents[pid]
        return sessions.get(pid, "")

    def watch_runaway_processes(self):
        """任务之外的进程空转（文件头 ⑥）：两台各扫一次，判中的写告警、发给所属会话，只报不停。弄坏开关 runaway-process-unreported 下不报。"""
        if lib.broken("runaway-process-unreported", self.environment):
            return
        now, present = time.time(), set()
        for host, listing in self.process_listings():
            for found in lib.runaway_processes(listing, RUNAWAY_MINIMUM_CPU_SECONDS, RUNAWAY_MINIMUM_FRACTION, self.environment):
                key = (host, found["pid"])
                present.add(key)
                entry = self.runaway_notified.get(key)
                # 上次报它之后才起的同号进程是另一个（换服务的空档里原来那个退了、号被复用）：从零数，不接着原来那个的报数停它；
                # elapsed 是那一台量的时长，与两台的钟对不对得上无关。弄坏开关 runaway-reused-pid-carried 下照号接着数
                if (entry is not None and entry["reported_at"] and now - found["elapsed"] > entry["reported_at"]
                        and not lib.broken("runaway-reused-pid-carried", self.environment)):
                    entry = None
                if entry is None:
                    entry = self.runaway_notified[key] = {"reported_at": 0.0, "count": 0, "alerted": False}
                if entry.get("stopped") or now - entry["reported_at"] < self.runaway_renotify_seconds:
                    continue   # 同一个进程只停一次：停不掉的那一条告警写了回读还在
                owner = self.runaway_owner(found["pid"], listing) if host == LOCAL else ""
                targets = self.runaway_targets(owner)
                label = lib.HOST_NAMES[host]
                if not owner:
                    protected = host == LOCAL and found["pid"] in self.runaway_protected_pids(listing)
                    stoppable = (lib.runaway_unowned_stoppable(found, self.environment) and not protected) or lib.broken("runaway-protected-stopped", self.environment)
                    if stoppable and not lib.broken("runaway-unowned-not-stopped", self.environment):
                        stopped = self.stop_runaway_process(host, found["pid"])
                        entry["stopped"] = True
                        self.alert("runaway-process-stopped", 0, f"{label}进程 {found['pid']}（{found['command']}）认不出发起的会话，服务直接按号终止：{'回读已不在' if stopped else '回读还在'}",
                                   "看是谁留下的，改掉那个写法；回读还在的按进程号查它为什么停不掉")
                    elif not entry["alerted"]:
                        entry["alerted"] = True
                        self.alert("runaway-process", 0, f"{label}进程 {found['pid']}（{found['command']}）在空转，认不出发起的会话，属于不碰的那一类（cgroup {found['cgroup']}）",
                                   "看它是不是该这么忙；是系统服务的查那个服务")
                    entry["reported_at"] = now
                    continue
                if entry["count"] >= RUNAWAY_REPORTS_BEFORE_STOP and not lib.broken("runaway-process-not-stopped", self.environment):
                    stopped = self.stop_runaway_process(host, found["pid"])
                    entry["stopped"] = True
                    line = f"本机进程 {found['pid']}（{found['command']}）报了 {entry['count']} 次还在，服务按号停了：{'回读已不在' if stopped else '回读还在'}"
                    self.alert("runaway-process-stopped", 0, line, "看是哪条命令留下的，改掉那个写法；回读还在的按进程号查它为什么停不掉")
                    for task in targets:
                        self.send_to_session(task, f"{INBOX_HEADLINE}任务之外的进程空转 已停 本机进程 {found['pid']}\n{line}", None)
                    continue
                stop = f"python3 .claude/singlefs-ai-sop/scripts/proc.py stop {found['pid']}"
                basis = (f"已跑 {found['elapsed']} 秒、累计 CPU {found['cpu']} 秒（占 {found['cpu'] / max(found['elapsed'], 1):.0%}），"
                         f"命令名 {found['command']}、cgroup {found['cgroup']}，不在任务 slice 里；是你这个会话发起的")
                after = f"报满 {RUNAWAY_REPORTS_BEFORE_STOP} 次还在，服务按号停掉；还在的每 {self.runaway_renotify_seconds:.0f} 秒再报一次"
                next_step = f"看它在做什么：用不上的现在就按进程号停（{label}上跑 {stop}）。{after}"
                if not entry["alerted"]:
                    entry["alerted"] = True
                    self.alert("runaway-process", 0, f"{label}进程 {found['pid']}：{basis}", next_step)
                entry["reported_at"] = now
                entry["count"] += 1
                for task in targets:
                    self.send_to_session(task, "\n".join([f"{INBOX_HEADLINE}任务之外的进程空转 {label}进程 {found['pid']}", f"依据：{basis}", f"出路：{next_step}"]), None)
        for key in [key for key in self.runaway_notified if key not in present]:
            del self.runaway_notified[key]
        self.persist_restart_surviving_state("runaway_notified")

    def runaway_targets(self, owner):
        """任务之外的进程空转发给谁 → [任务…]（send_to_session 照任务记的地址发）：谁的进程提醒谁，只发所属会话，地址取它最近一条找得到收件 socket 的任务；认不出所属或没有地址的交空表。
        只挑带收件地址的那几行（分片的子任务没有 inbox，不占那 20 行：同一会话最近交了不少于 20 片的分片任务时照样找得到它的父任务）。
        弄坏开关 runaway-sent-to-every-session 下发给服务健康的那几个会话，runaway-targets-counting-children 下子任务也占那 20 行。"""
        if lib.broken("runaway-sent-to-every-session", self.environment):
            return list(self.health_recipients().values())
        if not owner:
            return []
        where = "session = ?" if lib.broken("runaway-targets-counting-children", self.environment) else "session = ? AND json_extract(record, '$.inbox') IS NOT NULL"
        for task, valid in self.store.select("tasks", where, (owner,), order="id DESC LIMIT 20"):
            if valid and self.inbox_reachable(task):
                return [task]
        return []

    def stop_runaway_process(self, host, pid):
        """在那一台上按进程号停（两台同一个次序与判法：先 TERM，过宽限还在发 KILL）→ 回读 /proc 已不在（僵尸算不在）。本机 lib.stop_process_here，第二台经 ssh 跑 lib.stop_process_script。"""
        grace = 1 if lib.fast(self.environment) else int(RUNAWAY_STOP_GRACE_SECONDS)
        if host == PEER:
            return self.resources.peer.run(lib.stop_process_script(pid, grace), timeout=2 * grace + PROCESS_LIST_SECONDS)[0] == 0
        return lib.stop_process_here(pid, grace)

    def unit_unreadable_report(self):
        """/health 报的读不到单元状态（文件头 ①「读不到单元状态」）：→ ([{attempt, task, number, host, unit, seconds, presence, detail}…], 那一句或空串)。
        只读对账记下的那一段（unit_unreadable）与不经 systemd 最近一回核到的（unit_presence），不现读 systemd。"""
        now, entries = time.monotonic(), []
        for attempt_id, entry in sorted(dict(self.unit_unreadable).items()):
            verdict, detail = self.unit_presence_now(attempt_id)
            entries.append({"attempt": attempt_id, "task": entry["task"], "number": entry["number"], "host": entry["host"], "unit": entry["unit"],
                            "seconds": round(now - entry["since"], 1), "presence": verdict, "detail": detail})
        if not entries:
            return [], ""
        shown = "；".join(f"任务 {entry['task']} 第 {entry['number']} 次（派在{lib.HOST_NAMES.get(entry['host'], entry['host'])}，单元 {entry['unit']}，"
                         f"连着 {entry['seconds']:.0f} 秒，不经 systemd 核到{lib.PRESENCE_WORDS[entry['presence']]}）" for entry in entries[:UNIT_UNREADABLE_HEALTH_SHOWN])
        more = f"，另有 {len(entries) - UNIT_UNREADABLE_HEALTH_SHOWN} 次" if len(entries) > UNIT_UNREADABLE_HEALTH_SHOWN else ""
        return entries, (f"{UNIT_UNREADABLE_HEALTH_TEXT}（systemctl --user show 起不来、超时或没打 ActiveState），{len(entries)} 次：{shown}{more}；"
                         "服务不杀、不收尾、不记结局，照记 running（出路：在本机核用户级 systemd，systemctl --user show -p ActiveState <单元> 答得上之后服务照常判）")

    # ── 记账对账（文件头 ②「内存」的记账对账）──
    def booking_audit_loop(self):
        """每 booking_audit_seconds 一轮：读最近一份余量（第二台取探测线程那一份，不连第二台），逐台对内存记账（audit_bookings）；一轮出事不停服务。"""
        while not self.stopping.wait(self.booking_audit_seconds):
            try:
                self.audit_bookings(self.resources.snapshot(), time.time())
            except Exception:
                say("  ! 这一轮记账对账出了异常（下一轮再判）：\n" + traceback.format_exc())

    def audit_bookings(self, snapshot, now):
        """记账对账一轮：内存（audit_memory_bookings），判出的推给被挡住、还在排的任务所在的会话（push_booking_drift）。只报不停任务。弄坏开关 booking-audit-skipped 下不对账。"""
        if lib.broken("booking-audit-skipped", self.environment):
            return
        self.audit_memory_bookings(snapshot, now)
        self.refresh_booking_ledger(snapshot, now)
        self.push_booking_drift(now)

    @staticmethod
    def running_booking_text(basis, used, cap):
        """在跑记账为什么记这么多的那一句（booking_row 与 ledger_row 共用；basis 是 running_booking 交的 BOOKED_FROM_* 之一）：写清取的是此刻实测、实测到了上限，
        还是报告里没有它的 scope 记整份上限。"""
        human = gibibytes_text
        if basis == BOOKED_WITHOUT_SCOPE:
            return f"报告里没有它的 scope（刚起、读不到用量或这一轮这台没报告），记整份上限 {human(cap)}"
        if basis == BOOKED_WHILE_YOUNG:
            return f"起跑不满 {CPU_MEASURED_AFTER_SECONDS:.0f} 秒，照定的规矩记整份上限 {human(cap)}（满了之后照实测记；不算漂移）"
        if basis == BOOKED_FROM_CAP:
            return f"此刻实测 {human(int(used or 0))} 到了它的上限，记上限 {human(cap)}"
        return f"照此刻实测的用量 {human(int(used or 0))} 记"

    def booking_row(self, attempt, usage):
        """一条在跑尝试的记账明细 {label, task, booked, used, cap, why}：记多少（running_booking）、实占多少（报告里它的 scope 回收不掉的用量，没有 scope 是 None）、
        为什么记这么多（running_booking_text）。"""
        label, cap = attempt.get("label") or "", int(attempt.get("memory_cap") or 0)
        used = usage.get(label)
        booked, basis = self.running_booking(attempt, usage)
        return {"label": label, "task": attempt.get("task_id"), "booked": booked, "used": used, "cap": cap, "basis": basis, "why": self.running_booking_text(basis, used, cap)}

    def audit_memory_bookings(self, snapshot, now):
        """内存：逐台比「这台在跑尝试的记账之和」与对账的基准「报告里各 scope 回收不掉的用量之和」；
        不少于 BOOKING_DRIFT_RATIO 倍且多出 BOOKING_DRIFT_BYTES、连着 BOOKING_DRIFT_ROUNDS 轮，判记账漂移，写进 booking_drift['memory'][台]（逐条列各尝试记多少、实占多少、
        为什么记这么多）；这一轮对得上了的清掉。读不到这台的报告、这台没有在跑的不判（连数清零）。有 scope 的照实测记（文件头 ②「内存」），
        多出来的只来自报告里没有它的 scope、记整份上限的那几条。"""
        for host in self.machines:
            key, report = ("memory", host), snapshot.reports.get(host)
            attempts = [attempt for attempt in self.booking_attempts() if attempt.get("host") == host] if report is not None else []
            if not attempts:
                self.forget_drift(key)
                continue
            usage = self.report_scope_usage(report)
            rows = [self.booking_row(attempt, usage) for attempt in attempts]
            # 起跑不满两分钟、照规矩记整份上限的那几条（BOOKED_WHILE_YOUNG）不是漂移：对账时它们的记账照实占算（任务 15822 第 2 次尝试起跑 103 秒时被报成漂移）；
            # 弄坏开关 booking-audit-young-counted 下照整份上限算
            young_counted = lib.broken("booking-audit-young-counted", self.environment)
            booked = sum(int(row["used"] or 0) if row["basis"] == BOOKED_WHILE_YOUNG and not young_counted else row["booked"] for row in rows)
            used = sum(usage.values())
            expected = used
            if not (booked >= BOOKING_DRIFT_RATIO * expected and booked - expected >= BOOKING_DRIFT_BYTES):
                self.forget_drift(key)
                continue
            with self.booking_drift_lock:
                rounds = self.booking_audit_state.get(key, 0) + 1
                self.booking_audit_state[key] = rounds
                if rounds < BOOKING_DRIFT_ROUNDS:
                    continue
                known = self.booking_drift["memory"].get(host)
                shown = "；".join(f"任务 {row['task']} 尝试 {row['label']} 记 {gibibytes_text(row['booked'])}、实占 {'读不到' if row['used'] is None else gibibytes_text(row['used'])}：{row['why']}" for row in rows)
                text = (f"{self.machines[host].label}在跑的内存记账之和 {gibibytes_text(booked)}，实测各 scope 回收不掉的用量之和 {gibibytes_text(used)}"
                        f"（对账的基准；连着 {rounds} 轮不少于基准的 {BOOKING_DRIFT_RATIO:g} 倍、多出 {gibibytes_text(BOOKING_DRIFT_BYTES)} 以上）；逐条：{shown}")
                self.booking_drift["memory"][host] = {"since": known["since"] if known else now, "rounds": rounds, "booked": booked, "used": used, "expected": expected,
                                                      "attempts": rows, "text": text}
            if known is None:
                say(f"  ! 记账漂移：{text}")

    def forget_drift(self, key):
        """这一项（("memory", 台)）这一轮对得上、或不再判：连数清零、判出的记录删掉、推过的记录忘掉（再漂移再推）。"""
        with self.booking_drift_lock:
            self.booking_audit_state.pop(key, None)
            if self.booking_drift[key[0]].pop(key[1], None) is not None:
                say(f"  · 记账漂移（{key[0]} {key[1]}）好了")
            for notified in [notified for notified in self.booking_drift_notified if notified[0] == key]:
                del self.booking_drift_notified[notified]
        self.persist_restart_surviving_state("booking_drift_notified")

    def ledger_row(self, attempt, usage):
        """一条在跑尝试在记账明细里的一行（refresh_booking_ledger）：{task, number, label, cap, used, booked, basis, at_cap, text}。
        basis 说取的是哪一个：stopped（单元已确认停了、收尾没做完，不计），或 running_booking 交的 no-scope（报告里没有它的 scope，记整份上限）、
        measured（照报告里它的 scope 此刻回收不掉的用量）、cap（实测到了它的上限、记上限）；at_cap 为真是记的就是上限。
        text 照 running_booking_text 写清取的是哪一个与几个数。"""
        label, cap = attempt.get("label") or "", int(attempt.get("memory_cap") or 0)
        row = {"task": attempt.get("task_id"), "number": attempt.get("number"), "label": label, "cap": cap, "used": usage.get(label),
               "booked": 0, "basis": "stopped", "at_cap": False}
        if attempt.get("id") in self.unit_stopped:
            row["text"] = "单元已确认停了、收尾还没做完，不计"
            return row
        row["booked"], row["basis"] = self.running_booking(attempt, usage)
        row["at_cap"] = row["booked"] >= cap
        row["text"] = self.running_booking_text(row["basis"], row["used"], cap)
        return row

    def refresh_booking_ledger(self, snapshot, now):
        """在跑记账的逐条明细（/health 的 booking_ledger 一栏；control bookings 打出来）：逐台列这台每一条在跑的尝试——上限、此刻实占（报告里它的 scope 回收不掉的用量）、
        记多少、为什么记这么多（ledger_row）——与这台的合计：总额、余量、在跑上限之和、各 scope 实占合计、记账合计、照公式（总额 − 余量 − 记账合计）还放得下多少、
        实测 slice 还放得下多少。这台这一轮没有余量报告的只记一句。弄坏开关 bookings-ledger-unreported 下留空表。"""
        ledger = {}
        if not lib.broken("bookings-ledger-unreported", self.environment):
            human = gibibytes_text
            running = self.store.running_attempts()
            for host in self.machines:
                label, report = self.machines[host].label, snapshot.reports.get(host)
                if report is None:
                    ledger[host] = {"at": round(now, 1), "read": False, "rows": [], "title": f"{label}：这一轮没有余量报告（探测没回报告），记不出",
                                    "total": f"{label}：合计记不出"}
                    continue
                usage = self.report_scope_usage(report)
                rows = sorted((self.ledger_row(attempt, usage) for attempt in running if attempt.get("host") == host), key=lambda row: -row["booked"])
                margin = self.host_margin(host, snapshot)[0]
                slice_total, booked = snapshot.total(host), sum(row["booked"] for row in rows)
                caps, used = sum(row["cap"] for row in rows), sum(usage.values())
                room = None if margin is None else snapshot.total(host) - margin - booked   # 文件头 ②「内存」的公式
                slice_room = (report.get("memory") or {}).get("slice_room")
                title = (f"{label}：slice 总额 {human(slice_total)}、余量 {'量不到' if margin is None else human(margin)}、在跑 {len(rows)} 条的上限之和 {human(caps)}、"
                         f"各 scope 实占合计 {human(used)}")
                total = (f"{label}合计：记账 {human(booked)}（实占 {human(used)}），照公式（总额减余量减记账合计）放得下 "
                         f"{'量不到余量，算不出' if room is None else human(max(room, 0))}"
                         f"{'' if not isinstance(slice_room, int) else '；实测 slice 还放得下 ' + human(max(slice_room, 0))}")
                ledger[host] = {"at": round(now, 1), "read": True, "slice_total": slice_total, "margin": margin, "caps": caps, "used": used, "booked": booked,
                                "room": room, "slice_room": slice_room if isinstance(slice_room, int) else None, "rows": rows, "title": title, "total": total}
        with self.booking_drift_lock:
            self.booking_ledger = ledger

    def booking_ledger_snapshot(self):
        """/health 的 booking_ledger 一栏：{台: refresh_booking_ledger 记的那一台的明细}；记账对账还没做过一轮、或弄坏开关 bookings-ledger-unreported 下是空表。"""
        with self.booking_drift_lock:
            return json.loads(json.dumps(self.booking_ledger))

    def booking_drift_snapshot(self):
        """/health 的 booking_drift 一栏：{memory: {台: 明细}}，没有漂移的是空表。"""
        with self.booking_drift_lock:
            return json.loads(json.dumps({"memory": self.booking_drift["memory"]}))

    def drift_recipients(self):
        """被内存记账漂移挡住、还在排的任务所在的会话：{会话: 它那条任务（分片的子任务取父任务，收件地址在父任务上）}。看排队原因里写着 RUNNING_BOOKED_SUM_TEXT 的；
        照会话号找不到收件 socket 的不算。"""
        recipients = {}
        for task, valid in self.store.select("tasks", "state = 'queued'"):
            if not valid or RUNNING_BOOKED_SUM_TEXT not in self.waiting_reasons.get(task["id"], ""):
                continue
            found = self.store.task(task["parent_id"]) if task.get("parent_id") else (task, True)
            owner = found[0] if found is not None and found[1] else None
            if owner is not None and self.inbox_reachable(owner):
                recipients[owner.get("session", "")] = owner
        return recipients

    def push_booking_drift(self, now):
        """判出的记账漂移推给被挡住、还在排的任务所在的会话：同一件事对同一个会话判出来当场推一回，之后照 reminder_threshold（门槛从 BOOKING_DRIFT_PUSH_SECONDS 起、每翻一倍一道）
        每跨过一道门槛再推一回，同一道门槛不重推；好了再漂移再推（forget_drift 忘掉推过的）。"""
        with self.booking_drift_lock:
            current = [(("memory", host), entry) for host, entry in self.booking_drift["memory"].items()]
        for key, entry in current:
            crossed = reminder_threshold(now - entry["since"], BOOKING_DRIFT_PUSH_SECONDS)
            again = f"（判出满 {now - entry['since']:.0f} 秒还在，再推一回）" if crossed else ""
            for session, owner in self.drift_recipients().items():
                with self.booking_drift_lock:
                    if crossed <= self.booking_drift_notified.get((key, session), -1):
                        continue
                    self.booking_drift_notified[(key, session)] = crossed
                self.persist_restart_surviving_state("booking_drift_notified")   # 先记再发：发的当中服务没了，下一回起来不再推同一道门槛
                self.send_to_session(owner, "\n".join([f"{INBOX_HEADLINE}记账漂移 内存记账{again}", f"问题：{self.drift_text_shown(entry)}",
                                                       "出路：服务只报不停任务；挡着你排队的是上面这几条在跑的尝试多记的账：它们跑完或 task-server/task cancel 就放掉；"
                                                       "账一直不回落的把这一条交给维护 task-server 的会话（task-server/control check 也列着）"]), None)

    @staticmethod
    def drift_text_shown(entry):
        """推给会话的正文：逐条列的尝试至多 BOOKING_DRIFT_ROWS_SHOWN 条，多的写还有几条（/health 与 control check 列全）。"""
        rows = entry["attempts"]
        if len(rows) <= BOOKING_DRIFT_ROWS_SHOWN:
            return entry["text"]
        return f"{entry['text'].split('；逐条：')[0]}；逐条：" + "；".join(
            f"任务 {row['task']} 尝试 {row['label']} 记 {gibibytes_text(row['booked'])}、实占 {'读不到' if row['used'] is None else gibibytes_text(row['used'])}：{row['why']}"
            for row in rows[:BOOKING_DRIFT_ROWS_SHOWN]) + f"；还有 {len(rows) - BOOKING_DRIFT_ROWS_SHOWN} 条"

    def health_recipients(self):
        """名下有在排、在跑任务（分片的父任务也算）、此刻照会话号找得到收件 socket 的会话（inbox_reachable）：{会话: 它最新那一条这样的任务}（服务健康发给它们）。"""
        recipients = {}
        for task, valid in self.store.select("tasks", "state != 'finished'"):
            if valid and not task.get("parent_id") and self.inbox_reachable(task):
                recipients[task.get("session", "")] = task   # 照任务号从小到大：留最新那一条的地址
        return recipients

    def health_problems(self):
        """服务自己盯的健康（文件头 ⑥「服务健康」）：库写不写得进（试写一行）、派活判排头能起却一直没起成与看进展
        （/health 的 problem 读的那两句，排队线程每一轮算好的）、自己的单元在不在（is-active 的输出）、NFS 共享前三项（nfs_loop 复核的那一份）、
        第二台用不了满 peer_unavailable_notify_seconds（note_peer 记的）。→ [(种类, 名字, 那一句)]。"""
        problems = []
        writable, problem = self.store.health_write()
        if not writable:
            problems.append(("database-unwritable", "库写不进", f"库写不进：{problem}"))
        if self.starved_text:
            problems.append(("head-not-started", "派活判排头能起却一直没起成", self.starved_text))
        if self.progress_text:
            problems.append(("no-progress", "看进展：放得下却一直一条都没起", self.progress_text))
        state = own_unit_state(self.environment)
        if state not in lib.RUNNING_UNIT_STATES:
            problems.append(("unit-missing", "单元不在", f"单元 {lib.UNIT_NAME} 不在（is-active 打的是「{state}」）"))
        nfs = self.nfs_snapshot()
        if not nfs.get("ok", True):
            problems.append(("nfs", "NFS 共享判不过", f"NFS 共享判不过：{nfs.get('problem')}（出路：{nfs.get('next_step')}）"))
        peer = self.peer_health()
        since = peer.get("since")
        if peer.get("available") is False and isinstance(since, (int, float)) and time.time() - since >= self.peer_unavailable_notify_seconds:
            problems.append(("peer-unavailable", "第二台用不了", f"第二台用不了已 {time.time() - since:.0f} 秒：{peer.get('why')}（派得到第二台的这时只派本机）"))
        return problems

    def watch_health(self):
        """服务健康（文件头 ⑥）：查出一样，往每个名下有在排、在跑任务的会话发一条「task-server：服务健康 <哪一样>」；同一样对同一个会话只发一次，
        这一轮查不出了（好了）就忘掉，再坏再发。没有要发给的会话时不查（试写库、起 systemctl 都省掉），记下的也忘掉。
        弄坏开关 health-unwatched 下不查不发，health-alerted-every-round 下每一轮照发。"""
        if lib.broken("health-unwatched", self.environment):
            return
        recipients = self.health_recipients()
        if not recipients:
            self.health_notified.clear()
            self.persist_restart_surviving_state("health_notified")
            return
        problems = self.health_problems()
        found = {kind for kind, _, _ in problems}
        for kind in [kind for kind in self.health_notified if kind not in found]:
            say(f"  · 服务健康：{kind} 好了")
            del self.health_notified[kind]
        for kind, name, line in problems:
            told = self.health_notified.setdefault(kind, set())
            for session, task in recipients.items():
                if session in told and not lib.broken("health-alerted-every-round", self.environment):
                    continue
                told.add(session)   # 发不出去也记下：同一样不对这个会话重试到死（send_to_session 写了告警）
                self.persist_restart_surviving_state("health_notified")   # 先记再发：发的当中服务没了，下一回起来不再发同一条
                self.send_to_session(task, "\n".join([f"{INBOX_HEADLINE}服务健康 {name}", f"问题：{line}", f"出路：{HEALTH_NEXT_STEP}"]), None)
        self.persist_restart_surviving_state("health_notified")

    def selftest_advance_fault(self):
        """只在自证里：TASK_SERVER_SELFTEST_ADVANCE_FAILS=<文件>，那份文件在时 drive_sharded_parents 每推一个父任务先抛一个错（演分片父任务推不动）。"""
        arm = lib.selftest_override("TASK_SERVER_SELFTEST_ADVANCE_FAILS", self.environment)
        if arm and os.path.exists(arm):
            raise RuntimeError("自证造的推不动（TASK_SERVER_SELFTEST_ADVANCE_FAILS 点名的文件在）")

    def note_advance_failure(self, parent_id, why):
        """分片父任务这一轮推不动（drive_sharded_parents）：记进内存、整份记进库（换服务后照库接着数），attention_items 读。
        连着推不动是同一回（count 加一），推通之后再推不动是新的一回（提醒门槛加一）。"""
        with self.attention_lock:
            current = self.advance_failures.get(parent_id)
            if current is None:
                episode = self.advance_episodes.get(parent_id, 0) + 1
                self.advance_episodes[parent_id] = episode
                self.advance_failures[parent_id] = {"episode": episode, "why": why, "count": 1}
            else:
                current.update(why=why, count=current["count"] + 1)
        self.persist_restart_surviving_state("shard_advance")

    def note_advance_recovered(self, parent_id):
        with self.attention_lock:
            self.advance_failures.pop(parent_id, None)
        self.persist_restart_surviving_state("shard_advance")

    def log_changed_at(self, attempt, now):
        """日志文件的修改时刻换到 time.monotonic() 的钟上（不晚于 now）；读不到交 now。"""
        if lib.broken("stall-clock-reset-on-restart", self.environment):
            return now
        try:
            age = time.time() - os.path.getmtime(attempt.get("log", ""))
        except OSError:
            return now
        return now - max(0.0, age)

    def scope_cpu_usec(self, attempt):
        """这一次尝试此刻累计用了多少 CPU：attempt_runner 每次采样写进尝试目录的 cpu_usec（这一条子孙进程所在各 scope 的合计，里层包装各开的 scope 也在内）；
        读不到交 None。弄坏开关 outer-scope-cpu-only 下只读外层那一个 scope（改前的样子）。"""
        try:
            if lib.broken("outer-scope-cpu-only", self.environment):
                with open(self.attempt_paths(attempt)[2], encoding="utf-8") as handle:
                    scope = handle.read().strip()
                with open(os.path.join(scope, "cpu.stat"), encoding="utf-8") as handle:
                    return next(int(line.split()[1]) for line in handle if line.startswith("usage_usec "))
            with open(os.path.join(attempt.get("directory", ""), "cpu_usec"), encoding="utf-8") as handle:
                return int(handle.read().strip())
        except (OSError, ValueError, StopIteration, IndexError):
            return None

    def scope_thread_reading(self, attempt):
        """这一次尝试最近一轮的线程读数（本机的 Machine.thread_reading；文件头 ②「CPU 闸」的可跑线程数）：attempt_runner 每轮采样写进尝试目录的 lib.THREADS_FILE_NAME
        （JSON：sample、runnable、total、top_threads、top_name），读不到、写坏的交 None。"""
        try:
            with open(os.path.join(attempt.get("directory", ""), lib.THREADS_FILE_NAME), encoding="utf-8") as handle:
                return thread_reading_of(json.load(handle))
        except (OSError, ValueError):
            return None

    def scope_memory_peak(self, attempt):
        """这一次尝试进程内存（anon 加 shmem）见过的最大，字节：attempt_runner 每轮采、涨了就写进尝试目录的 memory_peak（文件头 ①「顶在上限上卡住」）；
        读不到（还没采到）交 None。弄坏开关 memory-peak-with-page-cache 下照改前读外层 scope（尝试目录的 scope 文件记着）的 memory.peak（含页缓存）。"""
        try:
            if lib.broken("memory-peak-with-page-cache", self.environment):
                with open(self.attempt_paths(attempt)[2], encoding="utf-8") as handle:
                    scope = handle.read().strip()
                with open(os.path.join(scope, "memory.peak"), encoding="utf-8") as handle:
                    return int(handle.read().strip())
            with open(os.path.join(attempt.get("directory", ""), lib.MEMORY_PEAK_FILE_NAME), encoding="utf-8") as handle:
                return int(handle.read().strip())
        except (OSError, ValueError):
            return None

    def scope_memory_current(self, attempt):
        """这一次尝试外层 scope（内存包装给它开的、尝试目录的 scope 文件记着）此刻进程用的内存（memory.stat 的 anon 加 shmem，memory_peaks_lib 的 process_memory_bytes），
        字节；读不到（还没采到 scope、scope 已收尾）交 None（本机的 Machine.memory_now；judge_stall_stop 判贴着上限用）。
        弄坏开关 memory-peak-with-page-cache 下照改前读 memory.current（含页缓存）。"""
        try:
            with open(self.attempt_paths(attempt)[2], encoding="utf-8") as handle:
                scope = handle.read().strip()
            if lib.broken("memory-peak-with-page-cache", self.environment):
                with open(os.path.join(scope, "memory.current"), encoding="utf-8") as handle:
                    return int(handle.read().strip())
        except (OSError, ValueError):
            return None
        return self.peaks.process_memory_bytes(scope) or None   # 0 是 scope 已收尾（目录不在）：同读不到

    def scope_blocked_processes(self, attempt):
        """这一次尝试此刻在 D 状态（不可中断的 IO 等待）的进程数（本机的 Machine.io_blocked；文件头 ①「卡在 IO 上」）：数的是尝试目录 started 记的 runner 进程号
        连同它的子孙（照父进程号往下：单元里的与内存包装、里层包装各开的 scope 里的都在内）与外层 scope（尝试目录的 scope 文件记着）cgroup.procs 里的进程，
        各读 <proc_root>/<号>/stat 的状态字段；进程号与 scope 两样都读不到交 None，读不到状态的那几个进程跳过。"""
        _, started_path, scope_path = self.attempt_paths(attempt)
        members, readable = set(), False
        try:
            with open(started_path, encoding="utf-8") as handle:
                runner = int(handle.read().strip())
            members |= descendant_processes(runner, self.proc_root) | {runner}
            readable = True
        except (OSError, ValueError):
            pass
        try:
            with open(scope_path, encoding="utf-8") as handle:
                scope = handle.read().strip()
        except OSError:
            scope = ""
        listed = cgroup_processes(scope) if scope else None
        if listed is not None:
            members |= {int(word) for word in listed if word.isdigit()}
            readable = True
        if not readable:
            return None
        return sum(1 for member in members if process_run_state(member, self.proc_root) == "D")

    def log_content(self, attempt):
        """→ 日志里算「在涨」的那部分字节数（读不到交 -1）。两台同一段：整份的大小减去起法那一层打的进度行（MULTI_HOST_RUN_STEP、MULTI_HOST_RUN_PEER_CPU、
        MULTI_HOST_RUN_PEER_MEMORY、MULTI_HOST_RUN_PEER_MEMORY_STAT、MULTI_HOST_RUN_PEER_GPU、MULTI_HOST_RUN_PEER_IO_WAIT）与服务自己接的 lib.INBOX_LOG_MARK、lib.LONG_RUN_LOG_MARK 那几行；从上次读到的位置接着读整行，最近一行 MULTI_HOST_RUN_PEER_CPU 的 usage_usec 记进 log_offsets（第二台的 Machine.cpu_usec 读它），
        最近一行 MULTI_HOST_RUN_PEER_MEMORY 的此刻内存与读到过的峰值记进 peer_memory（第二台的 Machine.memory_now 读它），
        这一回读到的几行 MULTI_HOST_RUN_PEER_IO_WAIT 与 MULTI_HOST_RUN_PEER_THREADS 接在还没被读走的后面（peer_io_rounds、peer_thread_rounds；第二台的 Machine.io_blocked、
        thread_reading 读时并起来：merge_peer_io_rounds、merge_peer_thread_rounds），
        read=yes 的 MULTI_HOST_RUN_PEER_MEMORY_STAT 里 current_bytes 最大的那一行记进 peer_memory_stat（弄坏开关 peak-memory-stat-read-at-exit 下记最后一行），
        以 lib.HEARTBEAT_LOG_MARK 起头的心跳行同样不算（弄坏开关 heartbeat-counted-as-log 下算）。
        同一次尝试同一时刻只一个线程在读（LOG_CONTENT_LOCKS 按尝试记录 id；排队线程判卡住与收尾线程都调它）：不会两个线程从同一个位置各读一遍、
        把同一段的 IO 等待与线程读数各接一回。弄坏开关 log-content-unlocked 下不拿锁。"""
        if lib.broken("log-content-unlocked", self.environment):
            return self.read_log_content(attempt)
        with keyed_lock(LOG_CONTENT_LOCKS, LOG_CONTENT_LOCKS_GUARD, attempt.get("id")):
            return self.read_log_content(attempt)

    def read_log_content(self, attempt):
        """log_content 拿着那一次尝试的锁做的那一段（判法见 log_content）。"""
        path = attempt.get("log", "")
        try:
            size = os.path.getsize(path)
        except OSError:
            return -1
        offset, noise, cpu = self.log_offsets.get(attempt["id"], (0, 0, None))
        memory = self.peer_memory.get(attempt["id"], (None, None))
        stat_at_peak = self.peer_memory_stat.get(attempt["id"])
        blocked_rounds = []   # 这一回读到的每一行 MULTI_HOST_RUN_PEER_IO_WAIT：blocked，read=no 记 None
        thread_rounds = []   # 这一回读到的每一行 MULTI_HOST_RUN_PEER_THREADS：ThreadReading，read=no 记 None
        read_at = time.time()
        # MULTI_HOST_RUN_PEER_GPU 是 multi-host-run.sh 每一轮照打的显存行：服务不读它（卡归 GPU 批处理验证服务），只不算日志在涨
        noise_marks = ("MULTI_HOST_RUN_STEP ", "MULTI_HOST_RUN_PEER_EXTERNAL_KILL ", "MULTI_HOST_RUN_PEER_GPU ", lib.INBOX_LOG_MARK + " ", lib.LONG_RUN_LOG_MARK + " ")
        if not lib.broken("log-noise-marks-incomplete", self.environment):
            # 第二台那一层收尾打的峰值与留着编译目录的几行同样不算日志在涨。弄坏开关 log-noise-marks-incomplete 下算
            noise_marks += tuple(mark + " " for mark in (lib.PEER_MEMORY_PEAK_MARK, lib.PEER_BUILD_PEAK_MARK, lib.PEER_BUILD_KEPT_MARK))
        if not lib.broken("heartbeat-counted-as-log", self.environment):
            noise_marks += (lib.HEARTBEAT_LOG_MARK + " ",)   # 命令自己打的心跳行：不算日志在涨
        with contextlib.suppress(OSError), open(path, "rb") as handle:
            while size > offset:   # 一次至多读 LOG_SCAN_BYTES：服务重启之后头一回接着读的是整份日志
                handle.seek(offset)
                chunk = handle.read(min(size - offset, LOG_SCAN_BYTES))
                complete = chunk[:chunk.rfind(b"\n") + 1] or (chunk if len(chunk) == LOG_SCAN_BYTES else b"")   # 一整块都没有换行的照整块往前走
                if not complete:
                    break
                line_end = offset
                for line in complete.splitlines(keepends=True):
                    line_end += len(line)
                    text = line.decode("utf-8", "replace").strip()
                    found = PEER_CPU_LINE.match(text)
                    if found:
                        cpu = int(found.group(2)) if found.group(3) == "yes" else cpu
                    held = PEER_MEMORY_LINE.match(text)
                    if held:   # read=no：这一轮没读到，此刻的不算数、峰值照留
                        memory = (int(held.group(2)), max(int(held.group(3)), memory[1] or 0)) if held.group(4) == "yes" else (None, memory[1])
                    waited = PEER_IO_WAIT_LINE.match(text)
                    if waited:
                        blocked_rounds.append(int(waited.group(2)) if waited.group(3) == "yes" else None)
                    threaded = PEER_THREADS_LINE.match(text)
                    if threaded:
                        thread_rounds.append(ThreadReading(*(int(value) for value in threaded.group(1, 2, 3, 4)), threaded.group(5)) if threaded.group(6) == "yes" else None)
                    stated = PEER_MEMORY_STAT_LINE.match(text)
                    if stated and stated.group(8) == "yes" and (stat_at_peak is None or int(stated.group(2)) > stat_at_peak["current"]
                                                                or lib.broken("peak-memory-stat-read-at-exit", self.environment)):
                        stat_at_peak = dict(zip(("current", *lib.MEMORY_STAT_NAMES), (int(value) for value in stated.groups()[1:7])))
                    killed_line = PEER_EXTERNAL_KILL_LINE.match(text)
                    if killed_line:
                        self.peer_external_kill[attempt["id"]] = (killed_line.group(1), int(killed_line.group(2)), killed_line.group(3))
                    if found or held or waited or threaded or stated or text.startswith(noise_marks):   # 服务自己接的那几行（文件头 ⑥）与命令的心跳行同样不算在涨
                        noise += len(line)
                offset += len(complete)
        self.log_offsets[attempt["id"]] = (offset, noise, cpu)
        if memory != (None, None):
            self.peer_memory[attempt["id"]] = memory
        if stat_at_peak is not None:
            self.peer_memory_stat[attempt["id"]] = stat_at_peak
        # 读到的几行接在还没被读走的后面（peer_io_rounds、peer_thread_rounds；PeerMachine.io_blocked、thread_reading 读时并起来）。
        # 弄坏开关 peer-readings-overwritten 下换成这一回读到的（同一轮里第二回读日志把前一回读到的那几行盖掉）
        overwritten = lib.broken("peer-readings-overwritten", self.environment)
        for table, rounds in ((self.peer_io_rounds, blocked_rounds), (self.peer_thread_rounds, thread_rounds)):
            if rounds:
                table[attempt["id"]] = list(rounds) if overwritten else table.get(attempt["id"], []) + rounds
        return size - noise

    def merge_peer_io_rounds(self, attempt):
        """第二台这一次尝试上一次读之后接下来的几行 MULTI_HOST_RUN_PEER_IO_WAIT 并起来（PeerMachine.io_blocked 调）：取最少的（有一轮没有 D 状态的进程或读不到，
        就不算每一轮都在等 IO；有一行 read=no 记 None）记进 peer_io_wait、清掉接着的那几行；一行都没接到的不动。→ peer_io_wait 里那一份。"""
        with keyed_lock(LOG_CONTENT_LOCKS, LOG_CONTENT_LOCKS_GUARD, attempt.get("id")):
            rounds = self.peer_io_rounds.pop(attempt.get("id"), None)
            if rounds:
                self.peer_io_wait[attempt["id"]] = None if None in rounds else min(rounds)
            return self.peer_io_wait.get(attempt.get("id"))

    def merge_peer_thread_rounds(self, attempt):
        """第二台这一次尝试上一次读之后接下来的几行 MULTI_HOST_RUN_PEER_THREADS 并起来（PeerMachine.thread_reading 调）：读到了的取最大（文件头 ②「CPU 闸」的可跑线程数：
        段内取最大；采样号、可跑的各取最大，占线程最多的进程取总数最大那一行的；都是 read=no 记 None）记进 peer_threads、清掉接着的那几行；一行都没接到的不动。"""
        with keyed_lock(LOG_CONTENT_LOCKS, LOG_CONTENT_LOCKS_GUARD, attempt.get("id")):
            rounds = self.peer_thread_rounds.pop(attempt.get("id"), None)
            if rounds:
                readable = [reading for reading in rounds if reading is not None]
                busiest = max(readable, key=lambda reading: reading.total) if readable else None
                self.peer_threads[attempt["id"]] = None if busiest is None else ThreadReading(
                    max(reading.sample for reading in readable), max(reading.runnable for reading in readable), busiest.total, busiest.top_threads, busiest.top_name)
            return self.peer_threads.get(attempt.get("id"))

    def signal_note(self, task, attempt, result, exit_code):
        """这一次被信号终止时给人看的那一句（文件头 ①「收尾」的信号）：尝试单元自己收到的停的信号（attempt_runner 记的 stopped_signal，它转给了命令）、
        或命令的退出码是 128 + 信号号（SIGNAL_EXIT_CODES），写出信号名，再写服务这一次有没有发过停的信号、是哪一步发的（service_stop_step）；不是信号交空串。
        弄坏开关 signal-exit-unnamed 下一律交空串。"""
        if lib.broken("signal-exit-unnamed", self.environment):
            return ""
        received = int((result or {}).get("stopped_signal") or 0)
        if received:
            head = f"尝试单元收到 {signal_name(received)}（attempt_runner 记的 stopped_signal），转给了命令"
        elif isinstance(exit_code, int) and exit_code in SIGNAL_EXIT_CODES:
            head = f"命令被 {signal_name(exit_code - 128)} 终止（退出码 {exit_code}）"
        else:
            return ""
        step = self.service_stop_step(task, attempt)
        return f"{head}；{'服务这一次发过停的信号：' + step if step else '服务这一次没有发过停的信号'}"

    def service_stop_step(self, task, attempt):
        """服务停这一次尝试是哪一步（signal_note 用）：判卡住、四小时硬上限、取消各照记下的那一样；都没记而 stop_running 对它发过停的写 stop_running；
        都没有交空串。"""
        for marked, step in ((attempt.get("stalled"), "判卡住（judge_stall_stop）"),
                             (attempt.get("run_hard_limit_stop"), "四小时硬上限（judge_run_hard_limit）"), (task.get("cancel_requested"), "取消（task cancel 或撤下这一片）"),
                             (attempt.get("id") in self.service_stopped_attempts, "停尝试的单元（stop_running）")):
            if marked:
                return step
        return ""

    def external_kill_note(self, task, attempt, result, outcome):
        """这一次尝试是不是被不是服务发的 TERM / KILL 杀掉的（文件头 ① 与 task_server_lib.py 文件头「错误类型」killed-externally）→ 说明一句，不是交空串。
        只在照退出码记的结局是 command-failed、服务这一次没有发过停的信号（service_stop_step）、单元自己也没收到停的信号（result 的 stopped_signal）时判；
        这一次尝试进程树里见过的进程号（Machine.process_ids_seen；第二台的进程号服务这一侧拿不到，走 peer_external_kill_note）里有一个在这一台同一时段（尝试开跑前几秒到结束后 EXTERNAL_KILL_LOG_AFTER_SECONDS 秒）
        的 earlyoom 日志（journalctl -t earlyoom）或内核日志（journalctl -k）里被点名杀了的，说明写那一行与进程号；日志读不到、没有对上的交空串，仍记 command-failed，不猜。
        自证里没换假 journalctl（TASK_SERVER_JOURNALCTL）的世界不读真日志。弄坏开关 external-kill-unrecognized 下一律交空串。"""
        environment = self.environment
        if outcome != "command-failed" or lib.broken("external-kill-unrecognized", environment) or (result or {}).get("stopped_signal"):
            return ""
        if self.service_stop_step(task, attempt):
            return ""
        if attempt.get("host") == PEER:
            return self.peer_external_kill_note(attempt)
        if lib.in_selftest(environment) and not environment.get("TASK_SERVER_JOURNALCTL"):
            return ""
        machine = self.machines.get(attempt.get("host"))
        process_ids = machine.process_ids_seen(attempt, result) if machine is not None else None
        if not process_ids:
            return ""
        ended = float((result or {}).get("ended_at") or time.time())
        since = float(attempt.get("started_at") or (result or {}).get("started_at") or ended) - 5.0
        for source, line in self.kill_log_lines(since, ended + EXTERNAL_KILL_LOG_AFTER_SECONDS):
            killed = lib.external_kill_pid(line, environment)
            if killed in process_ids:
                return (f"命令被不是服务发的信号杀掉：这一台的 {source} 日志里「{line.strip()[:200]}」点名的进程 {killed} 是这一次尝试进程树里的"
                        f"（attempt_runner 每轮采样记下的 {len(process_ids)} 个进程号之一）；服务这一次没有发过停的信号")
        return ""

    def peer_external_kill_note(self, attempt):
        """第二台的尝试被外部信号杀掉的说明（external_kill_note 的第二台那一路）：服务这一侧拿不到第二台的进程号，认的是 multi-host-run.sh 在第二台那一侧记下这一趟进程树的进程号、
        命令退非 0 之后读第二台的 earlyoom 与内核日志、点名这些进程号之一被杀时打的 MULTI_HOST_RUN_PEER_EXTERNAL_KILL 那一行（log_content 记进 peer_external_kill）；
        日志读到底再看，没有这一行交空串（仍是 command-failed）；那一行记的是内核「Memory cgroup out of memory」（lib.memory_cgroup_kill_line：某个内存组到了它自己的上限，
        不是整机内存紧张）的同样交空串，照原来的路判。multi-host-run.sh 那一侧不分这两种，服务这一侧按 line= 的内容分。
        弄坏开关 peer-external-kill-line-ignored 下不认这一行，memory-cgroup-kill-counted 下内存组那一种也认。"""
        self.log_content(attempt)
        found = self.peer_external_kill.get(attempt["id"])
        if found is None or lib.broken("peer-external-kill-line-ignored", self.environment):
            return ""
        source, killed, line = found
        if lib.memory_cgroup_kill_line(line, self.environment):   # 某个内存组到了它自己的上限（「Memory cgroup out of memory」）：不是外部，照原来的路判
            return ""
        return (f"命令被不是服务发的信号杀掉：第二台的 {source} 日志里「{line.strip()[:200]}」点名的进程 {killed} 是这一趟进程树里的"
                f"（multi-host-run.sh 在第二台那一侧记下的进程号，命令退非 0 之后读第二台的日志打了 MULTI_HOST_RUN_PEER_EXTERNAL_KILL 那一行）；服务这一次没有发过停的信号")

    def kill_log_lines(self, since, until):
        """[(来源, 一行)]：本机 since 到 until（Unix 秒）之间 earlyoom（journalctl -t earlyoom）与内核（journalctl -k）打的日志，只留 lib.external_kill_pid 认得的杀进程那种行；
        某一路起不来、超时、退非 0（没权限读这份日志这类）的跳过那一路。"""
        found = []
        for source, selector in (("earlyoom", ["-t", "earlyoom"]), ("内核", ["-k"])):
            try:
                completed = subprocess.run([self.journalctl, "--no-pager", "-q", "-o", "cat", f"--since=@{int(since)}", f"--until=@{int(until) + 1}", *selector],
                                           capture_output=True, text=True, errors="replace", env=self.environment, stdin=subprocess.DEVNULL, timeout=EXTERNAL_KILL_LOG_SECONDS)
            except (OSError, subprocess.SubprocessError):
                continue
            if completed.returncode == 0:
                found += [(source, line) for line in completed.stdout.splitlines() if lib.external_kill_pid(line, self.environment) is not None]
        return found

    def finish_attempt(self, task, attempt, result):
        # 有结果不等于收干净。保持 starting/running，现有资源账会继续保留这次占用。
        if attempt.get("cleanup_pending") and time.time() < attempt.get("cleanup_retry_at", 0):
            return
        if attempt.get("host") == LOCAL and not attempt.get("cleanup_pending"):   # 本机的尝试：单元已停（读得到、cgroup.procs 空）收尾慢也不占 CPU 与内存的账；第二台的由取消回读放
            self.release_stopped_booking(attempt)
        self.selftest_pause("attempt-finishing")
        found = self.store.task(task["id"])   # 现读：取消那一侧改过 cancel_requested（不拿对账时读的那一份）
        if found is not None and found[1]:
            task = found[0]
        exit_code = result.get("exit_code") if result else None
        cancelled = bool(task.get("cancel_requested")) and (result is None or bool(result.get("stopped_signal")))
        tail = log_tail(attempt.get("log", ""))
        machine = self.machines[attempt["host"]]
        exit_code, judged_host, runner_exit = machine.command_exit(exit_code, tail)   # 起法那一层打进日志的命令自己的退出码（Machine.command_exit）
        if runner_exit is not None:
            attempt["peer_runner_exit_code"] = runner_exit
        outcome = lib.classify_attempt(exit_code, judged_host, tail, cancelled, self.environment)
        if attempt.get("stalled") and not cancelled and (result is None or result.get("stopped_signal")):
            outcome = "memory-cap-hit" if attempt.get("stalled_at_cap") else "stalled"   # 顶在上限上卡住的按撞上限办（judge_stall_stop）
        if (attempt.get("run_hard_limit_stop") and not cancelled and (result is None or result.get("stopped_signal"))
                and not lib.broken("run-hard-limit-outcome-unmarked", self.environment)):
            outcome = lib.RUN_HARD_LIMIT_OUTCOME   # 照四小时硬上限判停的（judge_run_hard_limit）：settle_attempt 不重排，照结局收尾
        signal_note = self.signal_note(task, attempt, result, exit_code)
        if signal_note:   # 被信号终止的：结局原因句接这一句（task_server_lib.reason_fields），服务日志另记一行
            attempt["signal_note"] = signal_note
            say(f"  · 任务 {task['id']} 第 {attempt.get('number')} 次尝试：{signal_note}")
        kill_note = self.external_kill_note(task, attempt, result, outcome)
        if kill_note:   # 被不是服务发的 TERM / KILL 杀掉、earlyoom 或内核日志里有它的进程号：单独一种结局，照 retry_decision 重排（task_server_lib.py 文件头「错误类型」killed-externally）
            outcome = lib.EXTERNAL_KILL_OUTCOME
            attempt["external_kill_note"] = kill_note
            say(f"  · 任务 {task['id']} 第 {attempt.get('number')} 次尝试：{kill_note}")
        self.log_content(attempt)   # 把日志读到底：第二台每轮那几行最后一轮之后才到的照算（peer_memory_stat）
        peak_segment = self.finish_peak_segment_cores(attempt, machine, time.time())   # 这一次的峰值段核数（文件头 ②「CPU 闸」）：记 ended 时写进尝试
        gate_threads = self.finish_cpu_gate_threads(attempt, peak_segment)   # 连同可跑线程数的 CPU 闸线程数（文件头 ②「CPU 闸」）：记 ended 时写进尝试，记录读它
        # 脚本永远不报自己要多少内存（用户 2026-11-13 定「脚本永远不要报自己要多少」）：上限只照命令键的记录定、撞了乘 1.5，命令打的任何「要多少」行一律不认
        if outcome == "memory-cap-hit" and not lib.broken("cap-hit-stat-unreported", self.environment):
            # 内存最高那一刻的明细（文件头 ①「内存最高那一刻」）：抄进结局说明，分得清是堆还是页缓存
            attempt["peak_memory_note"] = lib.peak_memory_note(machine.memory_stat_at_peak(attempt, result))
        if outcome == "squeezed-out":   # 被谁挤掉：同一台此刻在跑的别的尝试（打回的理由 by= 那一栏，task_server_lib.reason_fields）
            attempt["squeezed_by"] = "、".join(f"任务 {other.get('task_id')}（上限 {other.get('memory_cap_text')}）"
                                              for other in self.store.running_attempts()
                                              if other.get("host") == attempt.get("host") and other.get("id") != attempt.get("id"))
        # 钉了的命令退 254 记被挤掉、照重排办（文件头 ②「内存」的钉值；任务 10289 钉值等于 slice 总上限，内存包装退 254 写「这一条的结果不算数，重跑」）。弄坏开关 pinned-squeeze-as-cap-hit 下照撞上限办
        squeezed_pinned = attempt.get("prediction") == lib.PINNED_HISTORY and not lib.broken("pinned-squeeze-as-cap-hit", self.environment)
        if outcome == "squeezed-out" and not squeezed_pinned and not lib.broken("squeeze-at-total-not-cap-hit", self.environment):
            # 这一条的上限已经到了那台上限收到的那个数（总上限减那台此刻的余量）：slice 满就是它自己撞了上限（实测：上限与总上限都是 50G 的那一条被内核在 slice 那一层杀，记成被挤掉、原样重排了三次）
            total = self.usable_maxima(self.resources.snapshot()).get(attempt.get("host", ""))   # 上限收到的那个数（booked_at_cap）；余量量不到的是 None，不改判
            if total and int(attempt.get("memory_cap") or 0) >= total:
                outcome = "memory-cap-hit"
        peer_clean, peer_note = self.confirm_peer_cleanup(attempt)
        local_clean, leftover = self.stop_leftovers(attempt, result)
        if leftover:
            attempt["leftover_note"] = leftover
        elif local_clean and attempt.get("cleanup_pending"):
            attempt["leftover_note"] = "清理重查已确认，本机进程已收干净"
        if not peer_clean or not local_clean:
            note = "；".join(part for part in (peer_note, leftover) if part)
            changed = attempt.get("cleanup_pending") != note
            attempt.update(cleanup_pending=note, cleanup_retry_at=time.time() + CLEANUP_RETRY_SECONDS)
            self.unit_stopped.discard(attempt["id"])   # 清理没确认：资源占用保留，在跑记账照数
            self.store.update("attempts", attempt)
            if changed:
                say(f"TASK_CLEANUP time={time.time():.6f} attempt={attempt['id']} status=pending detail={note}")
                self.alert("cleanup-pending", task["id"], f"任务 {task['id']} 清理尚未确认，保留资源占用：{note}",
                           "服务会重查；检查该尝试日志与远端连接。不要删进程号、退出码或清理状态文件来放行")
            return
        attempt.pop("cleanup_pending", None)
        attempt.pop("cleanup_retry_at", None)
        say(f"TASK_CLEANUP time={time.time():.6f} attempt={attempt['id']} status=confirmed")
        self.log_growth.pop(attempt.get("id"), None)
        self.log_offsets.pop(attempt.get("id"), None)
        self.hard_limit_stops.discard(attempt.get("id"))
        self.peer_memory.pop(attempt.get("id"), None)
        self.peer_io_wait.pop(attempt.get("id"), None)
        self.peer_threads.pop(attempt.get("id"), None)
        for table in (self.peer_io_rounds, self.peer_thread_rounds):
            table.pop(attempt.get("id"), None)
        self.thread_overruns.pop(attempt.get("id"), None)
        self.io_waits.pop(attempt.get("id"), None)
        self.activity_windows.pop(attempt.get("id"), None)
        for table in (self.progress_read_at, self.progress_files, self.progress_noted):
            table.pop(attempt.get("id"), None)
        self.service_stopped_attempts.pop(attempt.get("id"), None)
        self.running_threads_cache.pop(attempt.get("id"), None)
        self.unit_stopped.discard(attempt.get("id"))
        with self.running_usage_lock:
            self.running_peak_samples.pop(attempt.get("id"), None)
            self.running_peak_cores.pop(attempt.get("id"), None)
            self.running_peak_runnable.pop(attempt.get("id"), None)
            self.running_latest_cores.pop(attempt.get("id"), None)
            self.running_latest_runnable.pop(attempt.get("id"), None)
        self.cpu_windows.pop(attempt.get("id"), None)   # suspect_reported 不删：wait 那一侧可能刚读到这一次还在跑，删了会把报过的那一段再报一遍
        attempt.update(state="ended", exit_code=exit_code, outcome=outcome, ended_at=round((result or {}).get("ended_at") or time.time(), 3),
                       peak_metric=lib.PROCESS_MEMORY_PEAK_METRIC)   # 峰值口径一栏（task_server_lib.py 文件头「资源记录」）：这一版记的都是进程内存
        attempt[PEAK_SEGMENT_CORES_COLUMN] = peak_segment
        attempt[CPU_GATE_THREADS_COLUMN] = gate_threads
        attempt["seconds"] = round(attempt["ended_at"] - attempt.get("started_at", attempt["ended_at"]), 3)
        if result and not lib.broken("resources-unrecorded", self.environment):
            for name in ("memory_peak", "cpu_usec", "cores"):
                attempt[name] = result.get(name)
            attempt["build_peak"] = result.get("build_directory_peak")
            if result.get("seconds") is not None:
                attempt["seconds"] = result["seconds"]
        if not lib.broken("resources-unrecorded", self.environment):
            # 起法那一层打进日志的峰值（Machine.ended_peaks：第二台是 multi-host-run.sh 打的 MULTI_HOST_RUN_PEER_MEMORY_PEAK 与 MULTI_HOST_RUN_PEER_BUILD_PEAK）：
            # attempt_runner 量不到的那一项照它记
            for name, value in machine.ended_peaks(attempt, tail).items():
                if attempt.get(name) is None:
                    attempt[name] = value
            self.peer_memory_stat.pop(attempt.get("id"), None)
            self.peer_external_kill.pop(attempt.get("id"), None)
        kept_label = machine.kept_build_label(tail)
        if kept_label and attempt.get("build_directory") and os.path.basename(attempt["build_directory"]) == kept_label:
            # 起法那一层报这一次的编译目录留着了（第二台带 --keep-build-directory 的那一趟）：记成没删，重排到同一台的那一次接着用，换台或有了结局时服务删
            attempt["build_directory_removed"], attempt["build_directory_kept"] = False, True
        if attempt.get("memory_peak") is None and lib.broken("unmeasured-recorded-as-zero", self.environment):
            attempt["memory_peak"] = 0   # 弄坏开关：量不到的记成 0
        self.store.update("attempts", attempt)
        say(f"TASK_RESOURCES_RELEASED time={time.time():.6f} attempt={attempt['id']} outcome={outcome}")
        self.resources.invalidate()
        self.unit_checked_at.pop(attempt["id"], None)
        self.unit_gone_since.pop(attempt["id"], None)
        self.forget_unit_unreadable(attempt["id"])
        try:
            self.settle_attempt(task, attempt, result)
        except Exception as error:   # 尝试已记成 ended：这里出错不能留下「任务 running、尝试 ended」的孤儿，告警、下一轮对账 settle_ended_attempts 补收尾
            say(f"  ✗ 任务 {task['id']} 第 {attempt['number']} 次尝试的结局 {outcome} 已记，定任务去向时出错（{type(error).__name__}: {error}）：下一轮对账补收尾")
            self.alert("settle-failed", task["id"], f"任务 {task['id']} 第 {attempt['number']} 次尝试结局 {outcome} 已记、定去向时出错：{type(error).__name__}: {error}",
                       "服务下一轮对账会补收尾（settle_ended_attempts）；还 running 就看 journalctl --user -u singlefs-task-server.service 里这一句前后的栈")

    def labelled_scope(self, attempt):
        """本机 slice 下这次尝试的 scope；确认没有交空串，无法查询交 None。"""
        label = attempt.get("label") or ""
        if not label:
            return ""
        directory = self.peaks.slice_directory_of(self.environment.get("RUN_WITH_MEMORY_CAP_SLICE") or self.peaks.DEFAULT_SLICE,
                                                  self.environment.get("RUN_WITH_MEMORY_CAP_CGROUP_ROOT") or "/sys/fs/cgroup")
        if directory is None:
            return None
        if directory:
            try:
                with os.scandir(directory) as entries:
                    list(entries)
            except FileNotFoundError:
                return ""
            except OSError:
                return None
        for name, entry in self.peaks.slice_scopes(directory).items():
            if entry.get(self.peaks.SCOPE_LABEL_KEY) == label and name.endswith(".scope"):
                return os.path.join(directory, name)
        return ""

    def confirm_peer_cleanup(self, attempt):
        """新尝试的远端清理回执。失联、包装被杀或回执读不了都保留占用；恢复连接后重试收尾。
        同一次尝试同一时刻只起一趟 multi-host-run.sh --recover-cleanup（对账与停单元的后台线程、取消那一侧都会来确认）：按尝试记录 id 拿一把锁
        （锁表 PEER_CLEANUP_LOCKS，放下就删），后到的等前一趟放下、再读回执判，读到 confirmed 就不再起。
        弄坏开关 peer-cleanup-recovered-concurrently 下不拿锁，几处同时读到 pending 各起一趟、各往日志追加一份。"""
        path = attempt.get("peer_cleanup_file")
        if not path:   # 升级前的尝试没有此协议；不能凭空推断它写过新回执。
            return True, ""
        if lib.broken("peer-cleanup-recovered-concurrently", self.environment):
            return self.peer_cleanup_confirmed(attempt, path)
        with keyed_lock(PEER_CLEANUP_LOCKS, PEER_CLEANUP_LOCKS_GUARD, attempt.get("id")):
            return self.peer_cleanup_confirmed(attempt, path)

    def peer_cleanup_confirmed(self, attempt, path):
        """confirm_peer_cleanup 拿着那一次尝试的锁做的那一段：读回执，pending 的起一趟 --recover-cleanup、输出追加进那一次的日志，再读回执判。→ (确认了没有, 没确认的那一句)。"""
        record = read_json_file(path)
        if record and record.get("status") in ("not-started", "confirmed"):
            return True, ""
        if record and record.get("status") == "pending":
            try:
                completed = subprocess.run(["bash", self.peer_runner, "--recover-cleanup", path], capture_output=True, text=True,
                                           env=lib.ssh_multiplexed_environment(self.environment, self.environment),   # 服务起的 ssh 复用连接（task_server_lib.py 文件头「主机」）
                                           stdin=subprocess.DEVNULL, timeout=60)
                # 不凭退出码放行：回执必须原子写成 confirmed。
                with open(attempt["log"], "a", encoding="utf-8") as handle:
                    handle.write(completed.stdout + completed.stderr)
            except (OSError, subprocess.TimeoutExpired):
                pass
            record = read_json_file(path)
            if record and record.get("status") == "confirmed":
                return True, ""
        leftovers = [str(line)[:200] for line in ((record or {}).get("scope_leftovers") or [])][:PEER_SCOPE_LEFTOVERS_SHOWN]
        if leftovers:   # multi-host-run.sh 停了第二台这一趟内存包装的 scope 还留着进程（它文件头 ⑥）：说明写留下的进程，提醒换成核脚本的那一句
            return False, f"{PEER_SCOPE_LEFTOVERS_MARK}{'；'.join(leftovers)}（清理回执 {path}）"
        return False, f"远端清理回执未确认（{path}），尚不能放掉这一次的资源占用"

    def stop_leftovers(self, attempt, result):
        """尝试正常结束（结果已写、没被停）之后留下的进程（文件头 ①「留下的进程」）：单元里的由 attempt_runner 退出之前自己停掉、回读，记在 result.json 的
        unit_leftovers（它一退 systemd 就收单元、当场杀掉剩下的，服务那时回读不到）；外层 scope（内存包装开的，单元收尾时不跟着停）由服务回读，
        还有进程就发一次 systemctl --user kill --signal=SIGKILL（只对这一次自己的 scope 名），至多等 LEFTOVER_STOP_SECONDS 秒回读 cgroup.procs 空了没有。
        → (确认清空, 说明)。未知与仍有进程都交 False，后续对账重查。"""
        if lib.broken("leftover-processes-kept", self.environment) or not attempt.get("unit"):
            return True, ""
        if result is None and attempt.get("unit_unreadable_gone"):   # 读不到单元状态时不经 systemd 核过这一次的进程都不在（unit_unreadable_gone）：没有要停的
            return True, f"单元状态读不到，不经 systemd 核过这一次的进程都不在：{attempt['unit_unreadable_gone']}"
        found, remaining = {}, {}
        unit_leftovers = (result or {}).get("unit_leftovers") or {}
        if unit_leftovers.get("processes"):
            found[attempt["unit"]] = [str(process) for process in unit_leftovers["processes"]]
        # runner 的回读只是当时的快照；还剩或读不了时重新查 cgroup，不用旧 PID 杀可能复用的进程。
        if not unit_leftovers.get("verified") or unit_leftovers.get("remaining"):
            left = unit_processes(attempt["unit"], self.environment)
            if left is None:
                return False, f"单元 {attempt['unit']} 的进程读不了"
            runner_pid = str((result or {}).get("runner_pid") or "")
            left = [pid for pid in left if pid != runner_pid]
            if left:
                found[attempt["unit"]] = left
                with contextlib.suppress(OSError, subprocess.TimeoutExpired):
                    subprocess.run(["systemctl", "--user", "kill", "--signal=SIGKILL", attempt["unit"]],
                                   capture_output=True, env=self.environment, stdin=subprocess.DEVNULL, timeout=30)
                current = unit_processes(attempt["unit"], self.environment)
                if current is None:
                    return False, f"停单元 {attempt['unit']} 后无法回读"
                remaining[attempt["unit"]] = [pid for pid in current if pid != runner_pid]
        scope = str((result or {}).get("scope") or "") or self.labelled_scope(attempt)   # 命令比一轮采样还短时按标签找
        if scope is None:
            return False, "无法查询本机 scope，保留资源占用"
        if scope.startswith("/sys/fs/cgroup/") and scope.endswith(".scope"):
            name = os.path.basename(scope)
            left = cgroup_processes(scope)
            if left is None:
                return False, f"scope {name} 的进程读不了"
            if left:
                with contextlib.suppress(OSError, subprocess.TimeoutExpired):
                    subprocess.run(["systemctl", "--user", "kill", "--signal=SIGKILL", name], capture_output=True, env=self.environment, stdin=subprocess.DEVNULL, timeout=30)
                deadline = time.monotonic() + LEFTOVER_STOP_SECONDS
                still_there = cgroup_processes(scope)
                while still_there and time.monotonic() <= deadline:
                    time.sleep(0.2)
                    still_there = cgroup_processes(scope)
                if still_there is None:
                    return False, f"停 scope {name} 后无法回读"
                found[name], remaining[name] = left, still_there
        if not found:
            return True, ""
        counted = "、".join(f"{name} 里 {len(left)} 个（进程号 {' '.join(left[:8])}{' …' if len(left) > 8 else ''}）" for name, left in found.items())
        still = "、".join(f"{name} 还剩 {len(left)} 个" for name, left in remaining.items() if left)
        text = (f"命令退出之后还留着进程：{counted}（命令起的孙进程没收干净）"
                + (f"，回读 {still}，保留资源占用" if still else "，服务已停掉，回读 cgroup 空了"))
        say(f"  · 第 {attempt.get('number')} 次尝试{text}")
        return not bool(still), text

    # ── 收尾不在排队线程里做（文件头 ①「收尾不在排队线程里做」）──
    def finish_off_loop(self, task, attempt, result):
        """收尾一次尝试（finish_attempt）放到后台线程里做：第二台的清理回执（--recover-cleanup 经 ssh，至多 60 秒）、
        留下的进程回读（至多 LEFTOVER_STOP_SECONDS）、记结局时删编译目录（第二台 --remove-build-directory）都在那个线程里，
        排队线程照常对账别的尝试、派活。同一次尝试同一时刻只一个线程在做（finishing，按尝试记录 id；对账与补收尾见到它就跳过），做完放下、叫醒排队。
        弄坏开关 finish-in-scheduler 下照在排队线程里当场做。"""
        if attempt.get("cleanup_pending") and time.time() < attempt.get("cleanup_retry_at", 0):
            return   # 清理没确认的还没到重查时刻（finish_attempt 开头同一判）：不起线程

        def work():
            self.finish_attempt(task, attempt, result)
        if lib.broken("finish-in-scheduler", self.environment):
            work()
            return
        with self.finishing_lock:
            if attempt["id"] in self.finishing:
                return
            self.finishing.add(attempt["id"])
        threading.Thread(target=self.finish_in_background, args=(attempt["id"], work), name=f"task-server-finish-{attempt['id']}", daemon=True).start()

    def finish_in_background(self, attempt_id, work):
        """finish_off_loop 起的那个线程：做 work，出事打出来（尝试没记 ended 的下一轮对账再判，记了 ended 而任务没收尾的由 settle_ended_attempts 补），放下、叫醒排队。"""
        try:
            work()
        except Exception:
            say(f"  ! 尝试（行 {attempt_id}）收尾出了异常（下一轮对账再判）：\n" + traceback.format_exc())
        finally:
            with self.finishing_lock:
                self.finishing.discard(attempt_id)
            self.wake()

    def finishing_now(self, attempt_id):
        """这一次尝试的收尾此刻在后台线程里做着没有（finish_off_loop）。"""
        with self.finishing_lock:
            return attempt_id in self.finishing

    def finished_preface(self, task, attempt, outcome):
        """收尾那一刻结局说明前面接的几句（文件头 ①「留下的进程」与 task_server_lib.py 文件头「续跑」）：留下的进程停掉了的那一句；
        结局不是 succeeded、这一次跑超过 lib.LONG_RUN_SECONDS、命令又没登记续跑（lib.resumable_command）的另写「这条跑了 N 分钟、没有续跑：重交会从头跑」。
        弄坏开关 long-run-not-noted 下不写后一句。"""
        parts = [attempt["leftover_note"]] if attempt.get("leftover_note") else []
        seconds = float(attempt.get("seconds") or 0)
        limit = float(lib.selftest_override("TASK_SERVER_SELFTEST_LONG_RUN_SECONDS", self.environment) or lib.LONG_RUN_SECONDS)
        if (outcome not in ("succeeded", "cancelled") and seconds > limit and not lib.broken("long-run-not-noted", self.environment)
                and not lib.resumable_command(task.get("command") or [], task.get("cwd"), self.environment)):
            parts.append(f"这条跑了 {seconds / 60:.0f} 分钟、没有续跑：重交会从头跑")
        return "".join(part + "；" for part in parts)

    def settle_attempt(self, task, attempt, result=None):
        """尝试的结局已记进库（state=ended、outcome）之后定这条任务的去向（文件头「重试」）：告警、取消、钉值与改工作区的不重排、retry_decision 重排或收尾。
        与记尝试分开写：这一步半途出错时尝试已是 ended、任务还 running，由 reconcile 的 settle_ended_attempts 补收尾，不留孤儿；补收尾时 result 是 None。"""
        outcome, exit_code = attempt.get("outcome", ""), attempt.get("exit_code")
        if outcome == "peer-run-failed":   # 第二台那一趟没跑成：写告警、/health 的 peer 一栏记最近一次（本文件头 ②「机器」）
            self.note_peer_run_failure(task, attempt)
        if outcome == lib.PRUNE_SERVER_UNAVAILABLE_OUTCOME:   # 命令退 75：起一次 prune-server/control check，不通写告警（只报，不起、不停它）
            self.check_prune_server(task, attempt)
        if outcome == "cancelled":
            self.finalize(task, outcome, exit_code, "提交它的会话取消、单元已停")
            return
        counts = dict(task.get("outcome_counts") or {})
        counts[outcome] = counts.get(outcome, 0) + 1
        if outcome == lib.RUN_HARD_LIMIT_OUTCOME:
            self.settle_run_hard_limit(task, attempt, exit_code, counts)
            return
        # 结局与重排的说明前面写明（task status 的说明一行看得见）：顶在上限上卡住的那一次（judge_stall_stop）
        at_cap = f"{attempt.get('stall_note')}；" if (outcome == "memory-cap-hit" and attempt.get("stalled_at_cap")) or (outcome == "stalled" and attempt.get("stall_note")) else ""   # 判卡住的依据写进说明，不再是「没有说明」
        if outcome == "memory-cap-hit" and attempt.get("peak_memory_note"):   # 内存最高那一刻的明细（finish_attempt 记的）：重排与打回的说明同样写
            at_cap += f"{attempt['peak_memory_note']}；"
        leftover = f"{attempt['leftover_note']}；" if attempt.get("leftover_note") else ""   # 命令退出之后留下的进程停掉了（stop_leftovers）：重排的说明同样写
        outcome_preface = at_cap
        if outcome == "memory-cap-hit" and attempt.get("prediction") == lib.PINNED_HISTORY and not lib.broken("pinned-cap-grown", self.environment):
            # 钉值撞了上限不放大（文件头 ②「内存」的钉值）：上限就是配置的总上限，到顶照 memory-cap-at-maximum 打回
            self.finalize(task, "memory-cap-at-maximum", exit_code,
                          f"{outcome_preface}{self.finished_preface(task, attempt, 'memory-cap-at-maximum')}钉值（多机配置的 {lib.PIN_SOURCE}）{attempt.get('memory_cap_text', '')} 也撞了上限：钉值不放大，打回",
                          outcome_counts=counts, maximum_host=attempt.get("host", ""), maximum_cap_text=attempt.get("memory_cap_text", ""))
            return
        if outcome in WORKSPACE_WRITER_NO_RETRY_OUTCOMES and self.modifies_workspace(task) and not lib.broken("workspace-writers-retried", self.environment):
            # 改主工作区的命令（换钩子、打补丁、暂存）跑第二遍结果就不同：凡是会重排的结局一律不重排，打回提交方自己改了重交，不绕过
            #（用户：「改主工作区的命令撞上限不再原样重排、打回让提交方修改 后提交，不能绕过」）
            self.finalize(task, outcome, exit_code,
                          f"{outcome_preface}{self.finished_preface(task, attempt, outcome)}这条命令改主工作区，跑第二遍结果就变（补丁已打上的再打会被拒），服务不重排：改好再重交，"
                          f"或先在副本里跑；下次这条命令照这把键近 {RECORD_WINDOW} 次最大起记，撞过的这一次按 {attempt.get('memory_cap_text', '')} 算、不放大", outcome_counts=counts)
            return
        maxima = self.usable_maxima(self.resources.snapshot()) if outcome == "memory-cap-hit" else {}   # 乘 1.5 照上限收到的那个数比
        fixed_host = PEER if self.peer_only_entry(task) else ""
        if fixed_host:   # 弄坏开关 host-lists-restored：照改前只派第二台的入口不换台
            maxima = {fixed_host: maxima.get(fixed_host)} if maxima else {}
        decision = lib.retry_decision(counts, outcome, int(attempt.get("memory_cap") or 0), attempt.get("host", ""), task.get("next_host", "") or fixed_host,
                                      maxima, self.environment)
        if not decision.retry:
            maximum = ({"maximum_host": attempt.get("host", ""), "maximum_cap_text": attempt.get("memory_cap_text", "")}
                       if decision.outcome == "memory-cap-at-maximum" else {})
            self.finalize(task, decision.outcome, exit_code, (outcome_preface + self.finished_preface(task, attempt, decision.outcome) + decision.why).rstrip("；"),
                          outcome_counts=counts, **maximum)
            return
        if lib.broken("retry-build-directory-fresh", self.environment) and not lib.broken("retry-build-directory-kept", self.environment):
            # 弄坏开关：照改前重排时当场删这一份，下一次从零编（task_server_lib.py 文件头「编译目录」的重试复用）
            self.remove_build_directory(task, attempt, result)
            self.store.update("attempts", attempt)
        # 别的情形这一份留着：下一次起在同一台就接着用，换了台在起的时候删（start_attempt 的 kept_build_directory）；最后那一次的在记下结局时删
        number = int(attempt.get("number") or 0)
        try:   # 重排在一个事务里判：任务还 running、这一次是它最新那一次、没被取消（取消那一侧改过的不拿旧字典盖掉）
            task = self.store.change("tasks", task["id"], dict(state="queued", next_cap=decision.cap_bytes, next_host=decision.host or "",
                                                                note=at_cap + leftover + decision.why, outcome_counts=counts),
                                     guard=lambda current: ("cancelled" if current.get("cancel_requested") else
                                                            "" if current.get("state") == "running" and int(current.get("attempt_count") or 0) == number else "state"))
        except StoreRefused as refused:
            if refused.kind == "guard" and str(refused) == "cancelled":
                self.finalize(task, "cancelled", exit_code, f"第 {number} 次尝试 {outcome}，重排之前提交它的会话取消了：不再起", outcome_counts=counts)
            elif refused.kind == "tampered":
                self.tampered(task["id"], task, "任务记录")
            return
        say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试 {outcome}：{at_cap}{decision.why}，上限 {lib.mebibytes_text(decision.cap_bytes)} 重排")
        if task.get("role") == "shard" and lib.broken("shard-retry-restarts-all", self.environment):
            for sibling in self.children_of(task["parent_id"]):   # 弄坏开关：成了的兄弟片也重排、再跑一遍
                if sibling.get("role") == "shard" and sibling["id"] != task["id"] and sibling.get("outcome") == "succeeded":
                    self.store.change("tasks", sibling["id"], dict(state="queued", outcome="", exit_code=None, note="弄坏开关 shard-retry-restarts-all：跟着重排"),
                                      reopen=True)
        self.wake()

    def remove_build_directory(self, task, attempt, result):
        """被下一次尝试顶替的那一次的编译目录（文件头「编译目录」）：有进程开着里面的文件就不删、写告警。"""
        directory = attempt.get("build_directory")
        machine = self.machines.get(attempt.get("host")) or self.machines[LOCAL]
        if not directory or attempt.get("build_directory_removed") or not machine.build_directory_exists(directory):   # 起法那一层自己删的（第二台没留着的）记成已删
            return
        removed, lines = machine.remove_build_directory(directory)   # 两台同一段：本机就地删，第二台经 multi-host-run.sh --remove-build-directory
        attempt["build_directory_removed"] = removed
        if not removed:
            self.alert("build-directory-left", task["id"], f"第 {attempt.get('number')} 次尝试的编译目录 {directory} 没删成：" + "；".join(line.strip() for line in lines),
                       machine.removal_howto(directory))

    def kept_build_directory(self, task, number):
        """重排的这一次（第 number 次）起之前：上一次尝试服务在本机建的编译目录还留着（没删、还在）交那一次的尝试记录，没有交 None
        （文件头「编译目录」的重试复用）。第二台那一份带 --keep-build-directory 跑完留着的（multi-host-run.sh 报了留着，build_directory_kept）同样算，
        没留着的跑完就删了（记成已删），不算。在不在照那一台的接口判（Machine.build_directory_exists）。"""
        if number <= 1:
            return None
        for attempt, valid in self.store.attempts_of(task["id"]):
            machine = self.machines.get(attempt.get("host")) or self.machines[LOCAL]
            if (valid and int(attempt.get("number") or 0) == number - 1 and attempt.get("build_directory") and not attempt.get("build_directory_removed")
                    and machine.build_directory_exists(attempt["build_directory"])):
                return attempt
        return None

    def remove_finished_build_directories(self, task):
        """有了结局的这条任务各次尝试留下的编译目录一律删（文件头「编译目录」；第二台留着的经 multi-host-run.sh --remove-build-directory 删）：
        提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的本机那几份留着现场、
        交给提交它的会话用 task clean 删；有进程还开着里面的文件就不删、写告警（同 remove_build_directory）。弄坏开关 finished-build-kept 下照改前留着。"""
        if lib.broken("finished-build-kept", self.environment):
            return
        with self.build_removal_lock:   # 取消之后删产物那一侧同一把锁（remove_cancelled_artifacts）
            self.remove_finished_build_directories_locked(task)

    def remove_finished_build_directories_locked(self, task):
        """remove_finished_build_directories 拿着 build_removal_lock 做的那一段。"""
        # 留现场的只留本机那几份（task clean 删的就是它们）；第二台留着的照删：task clean 不连第二台，改前第二台那一份跑完就删
        keep_local = (task.get("environment") or {}).get("SINGLEFS_KEEP_TEST_TEMPORARY_FILES") == "1"
        handled = set()   # 重排接着用的几次尝试记的是同一个目录：删一次，几条记录都记成已删
        for attempt, valid in self.store.attempts_of(task["id"]):
            directory = attempt.get("build_directory")
            machine = self.machines.get(attempt.get("host")) or self.machines[LOCAL]
            if not valid or not directory or attempt.get("build_directory_removed") or (keep_local and machine.name == LOCAL):
                continue
            if (attempt.get("host"), directory) in handled or not machine.build_directory_exists(directory):
                removed, lines = (attempt.get("host"), directory) in handled, []
            else:
                self.selftest_pause("finished-build-removal")
                removed, lines = machine.remove_build_directory(directory)   # 两台同一段：本机就地删，第二台经 multi-host-run.sh --remove-build-directory
            if removed:
                handled.add((attempt.get("host"), directory))
                self.mark_build_directory_removed(attempt)
            elif not machine.build_directory_exists(directory):
                continue
            else:
                self.alert("build-directory-left", task["id"], f"第 {attempt.get('number')} 次尝试的编译目录 {directory} 没删成：" + "；".join(line.strip() for line in lines),
                           machine.removal_howto(directory))

    def mark_build_directory_removed(self, attempt):
        """记这一次尝试的编译目录已删（记下结局时、取消之后、task clean 三处删完回读不在了之后调）：只并进 build_directory_removed 这一栏，
        在一个事务里现读库里那一行再写（Store.change），不拿调用方手里先读的那份字典整行写回：HTTP 线程删目录的这一会儿，对账可能已把这一次记成 ended、
        记下结局与峰值。行不在或校验不过的记不上：服务日志说一句（目录已删，不当没删）。弄坏开关 attempt-row-overwritten 下拿手里那份整行写回。"""
        attempt["build_directory_removed"] = True
        if lib.broken("attempt-row-overwritten", self.environment):
            self.store.update("attempts", attempt)
            return
        try:
            attempt.update(self.store.change("attempts", attempt["id"], {"build_directory_removed": True}))
        except StoreRefused as refused:
            say(f"  ! 任务 {attempt.get('task_id')} 第 {attempt.get('number')} 次尝试的编译目录已删，记不上「已删」：{refused}")

    def sweep_finished_build_directories(self):
        """服务起来时扫一遍：有了结局的任务还留着的编译目录照 remove_finished_build_directories 删（在后台线程里删，不挡排队）。"""
        for task, valid in self.store.select("tasks", "state = 'finished'"):
            if valid:
                with contextlib.suppress(Exception):   # 一条删不成（记录坏了、文件在用）不挡别的条；没删成的已写告警
                    self.remove_finished_build_directories(task)

    def cancel_artifact_sizes(self, task_id):
        """取消一开始量（文件头「接口」的取消）：这条任务（分片的连同子任务）各次尝试还没删的编译目录删前多大、此刻在起与在跑的尝试是哪几次。
        → {"sizes": {(台, 目录): 字节，第二台本地盘上的是 None}, "active": {尝试记录的 id…}}；名下有校验不过的子任务行交空的（取消那一侧照 record-tampered 办，一份都不删）。"""
        before = {"sizes": {}, "active": set()}
        try:
            owners = [task_id] + [child["id"] for child in self.children_of(task_id)]
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            return before
        for owner in owners:
            for attempt, valid in self.store.attempts_of(owner):
                if not valid:
                    continue
                if attempt.get("state") in ACTIVE_ATTEMPT_STATES:
                    before["active"].add(attempt.get("id"))
                directory, host = attempt.get("build_directory"), attempt.get("host") or LOCAL
                if directory and not attempt.get("build_directory_removed"):
                    before["sizes"].setdefault((host, directory), directory_bytes(directory) if host == LOCAL and os.path.lexists(directory) else None)
        return before

    def remove_cancelled_artifacts(self, task_id, before):
        """取消收尾之后删产物（文件头「接口」的取消；用户：「取消的任务要自己删掉产物 不要残留」「这个加入流程中 让task server去做」）：这条任务（分片的连同子任务）
        每一次尝试的编译目录与现场目录（设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的也删），派到第二台那一趟的本地目录照日志里 multi-host-run.sh ⑥ 那一行认（cancelled_attempt_artifacts）。
        before 是 cancel_artifact_sizes 在取消一开始量的。逐份写进每条任务最后一次尝试的日志；没删成的写告警 build-directory-left。
        → {"removed": [份…], "left": [份…], "lines": [给回话与日志的行…]}。弄坏开关 cancelled-artifacts-kept 下不删、交空的。"""
        if lib.broken("cancelled-artifacts-kept", self.environment):
            return {"removed": [], "left": [], "lines": []}
        try:
            owners = [task_id] + [child["id"] for child in self.children_of(task_id)]
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            return {"removed": [], "left": [], "lines": [f"  ✗ {self.children_refused(task_id, refused, '取消之后一份产物都不删')}"]}
        removed, left, every_line, handled = [], [], [], set()
        with self.build_removal_lock:   # 记下结局时删的那一侧（remove_finished_build_directories）同一把锁：先到的删，后到的回读
            for owner in owners:
                attempts = [attempt for attempt, valid in self.store.attempts_of(owner) if valid]
                lines = []
                for attempt in attempts:
                    for item in self.cancelled_attempt_artifacts(attempt, before, handled):
                        (removed if item["gone"] else left).append(item)
                        lines += cancelled_artifact_lines(item)
                        if not item["gone"]:
                            self.alert("build-directory-left", owner, f"任务 {owner} 取消之后{item['path']} 没删掉：{item['why']}", item["howto"])
                if attempts and lines:
                    append_log_lines(attempts[-1].get("log", ""), lines + [f"  · 取消之后删产物：这条任务删了 {sum(1 for line in lines if line.startswith('  · '))} 份、"
                                                                           f"没删成 {sum(1 for line in lines if line.startswith('  ✗ '))} 份"])
                every_line += lines
        every_line.append(f"  · 取消之后删产物：删了 {len(removed)} 份、没删成 {len(left)} 份" if removed or left
                          else "  · 取消之后删产物：各次尝试都没留下编译目录、现场目录与第二台的本地目录，没有要删的")
        return {"removed": removed, "left": left, "lines": every_line}

    def cancelled_attempt_artifacts(self, attempt, before, handled):
        """一次尝试取消之后的产物（remove_cancelled_artifacts 用）：→ [{task, attempt, host, kind, path, bytes_before, bytes_after, gone, how, why, howto}]。
        编译目录（本机就地删、第二台留着的经 multi-host-run.sh --remove-build-directory 删）：这里还在的删、回读，已被记下结局那一侧删了的照取消一开始量的报删前；
        现场目录（共享工作根下 SCENES_ROOT_NAME/<尝试标签>，两台同一个绝对路径）：在的量了大小就地删、回读；
        取消时在起、在跑的第二台那一次：日志里有 PEER_TRIP_REMOVED_MARK 那一行（「回读不在了」）算 multi-host-run.sh 删了这一趟的本地目录与 runs/ 下三个文件；
        没有那一行的先读这一次的清理回执（peer_cleanup_file）：还是 not-started、日志里也没有 PEER_TRIP_BUILT_MARKS 那两行的，multi-host-run.sh 还没建这一趟的本地目录，
        当已清理、不列（与 confirm_peer_cleanup 认 not-started 同一口径）；回执到过 pending（pending、confirmed）、读不了、没有回执，或建过目录的，算没删成。
        弄坏开关 cancel-not-started-receipt-unread 下不读回执，没有那一行一律算没删成。
        handled 记删过的（台, 目录）：重排接着用的几次记的是同一份，只报一次。"""
        items = []
        host = attempt.get("host") or LOCAL
        machine = self.machines.get(host) or self.machines[LOCAL]
        base = {"task": attempt.get("task_id"), "attempt": attempt.get("number"), "host": host}
        directory = attempt.get("build_directory")
        key = (host, directory)
        if directory and key not in handled and (key in before["sizes"] or not attempt.get("build_directory_removed")):
            handled.add(key)
            size = before["sizes"].get(key)
            item = dict(base, kind="编译目录", path=directory, bytes_before=size, bytes_after=0, gone=True, why="", howto=machine.removal_howto(directory))
            present = (os.path.lexists(directory) if host == LOCAL else not attempt.get("build_directory_removed"))
            if not present:
                item["how"] = "记下结局时已删、回读不在了"
            else:
                if host == LOCAL and size is None:
                    item["bytes_before"] = directory_bytes(directory)
                done, lines = machine.remove_build_directory(directory)   # 本机删完回读不在了才交 True；第二台经 --remove-build-directory 回读
                gone = done and (not os.path.lexists(directory) if host == LOCAL else True)
                item.update(gone=gone, how="这一次取消删的、回读不在了" if gone else "",
                            why="" if gone else ("；".join(line.strip() for line in lines if line.strip()) or "删完还在"),
                            bytes_after=0 if gone else (directory_bytes(directory) if host == LOCAL else None))
                if gone:
                    self.mark_build_directory_removed(attempt)
            items.append(item)
        scene = attempt.get("scene_directory")
        if scene and os.path.lexists(scene) and ("scene", scene) not in handled:   # 现场目录在共享工作根下，两台同一个绝对路径，就地删（task_server_lib.py 文件头「现场」）
            handled.add(("scene", scene))
            size = directory_bytes(scene)
            shutil.rmtree(scene, ignore_errors=True)
            gone = not os.path.lexists(scene)
            items.append(dict(base, kind="现场目录", path=scene, bytes_before=size, bytes_after=0 if gone else directory_bytes(scene), gone=gone,
                              how="这一次取消删的、回读不在了" if gone else "", why="" if gone else "rm 之后还在（权限不对或有进程在往里写）",
                              howto=f"停掉往里写的进程，再 rm -r {shlex.quote(scene)}"))
        if host == PEER and attempt.get("id") in before["active"]:
            label = attempt.get("label") or "?"
            trip = f"第二台这一趟的本地目录 builds/{label} 与 runs/ 下 {label}.pid、.exit、.log"
            log_lines = log_tail(attempt.get("log", "")).splitlines()
            reported = any(PEER_TRIP_REMOVED_MARK in line and PEER_TRIP_READ_BACK_MARK in line for line in log_lines)
            receipt = read_json_file(attempt["peer_cleanup_file"]) if attempt.get("peer_cleanup_file") else None
            receipt_status = receipt.get("status") if isinstance(receipt, dict) else None
            built = any(mark in line for line in log_lines for mark in PEER_TRIP_BUILT_MARKS)
            if not reported and receipt_status == "not-started" and not built and not lib.broken("cancel-not-started-receipt-unread", self.environment):
                return items   # 第二台这一趟的本地目录还没建（同 confirm_peer_cleanup 认 not-started）：没有要删的，不列
            receipt_text = ((f"清理回执 {receipt_status}" if receipt_status else "清理回执读不到" if attempt.get("peer_cleanup_file") else "这一次没有清理回执")
                            + ("、日志里有建了这一趟的本地目录之后才打的那一行" if built else ""))
            items.append(dict(base, kind="本地目录", path=trip, bytes_before=attempt.get("build_peak"), bytes_after=None if not reported else 0, gone=reported,
                              how="multi-host-run.sh 的 ⑥ 删了、回读不在了" if reported else "",
                              why="" if reported else (f"这一次尝试的日志里没有 multi-host-run.sh 打的「{PEER_TRIP_REMOVED_MARK} …{PEER_TRIP_READ_BACK_MARK}」那一行"
                                                       f"（{receipt_text}），第二台那一份删没删判不了"),
                              howto=(f"到第二台多机配置 BUILD_ROOT_BY_HOST 里它那一项的目录下核 builds/{label} 与 runs/{label}.pid / .exit / .log，"
                                     "在的话先停开着它们的进程（python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>）再删")))
        return items

    def remove_scenes(self, task_id):
        """删这条任务（分片的连同子任务）各次尝试的现场目录（task_server_lib.py 文件头「现场」）：→ (删了的 [{path, bytes}], 没删成的 [{path, bytes, kind, why}])。"""
        removed, left = [], []
        try:
            owners = [task_id] + [child["id"] for child in self.children_of(task_id)]
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            return [], [{"path": self.scenes_root, "bytes": 0, "kind": "现场", "why": self.children_refused(task_id, refused, "一份现场都不删")}]
        for owner in owners:
            for attempt, _ in self.store.attempts_of(owner):
                scene = attempt.get("scene_directory")
                if not scene or not os.path.lexists(scene):
                    continue
                size = directory_bytes(scene)
                shutil.rmtree(scene, ignore_errors=True)
                if os.path.lexists(scene):
                    left.append({"path": scene, "bytes": size, "kind": "现场", "why": "rm 之后还在（权限不对或有进程在往里写）"})
                else:
                    removed.append({"path": scene, "bytes": size})
        return removed, left

    def scene_due(self, task_id, now):
        """这条任务的现场到没到该删的时候（task_server_lib.py 文件头「现场」）：库里没有了的到了；校验不过的不动；分片的子任务照父任务；
        要提交方 task wait 取走结局满 scene_keep_seconds，没有会话的（人手 --manual，不一定起 wait）按结局时刻算；提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的只由 task clean 删（弄坏开关 scene-keep-ignored 下照删）。
        弄坏开关 scene-sessionless-kept 下没有会话的也要等取走。"""
        found = self.store.task(task_id)
        if found is not None and found[1] and found[0].get("parent_id"):
            found = self.store.task(found[0]["parent_id"])
        if found is None:
            return True
        task, valid = found
        if not valid or task.get("state") != "finished":
            return False
        if not task.get("collected") and (task.get("session") or lib.broken("scene-sessionless-kept", self.environment)):
            return False
        if (task.get("environment") or {}).get("SINGLEFS_KEEP_TEST_TEMPORARY_FILES") == "1" and not lib.broken("scene-keep-ignored", self.environment):
            return False
        return now - float(task.get("collected_at") or task.get("finished_at") or now) >= self.scene_keep_seconds

    def sweep_scenes(self):
        """排队线程每一轮看一遍现场根（task_server_lib.py 文件头「现场」）：这一个库的标签的那几个目录（task-<库标签>-<任务号>-<第几次>）到了该删的就删；
        别的标签的不动。弄坏开关 scene-kept-after-collect 下一个都不删。"""
        if lib.broken("scene-kept-after-collect", self.environment):
            return
        try:
            names = os.listdir(self.scenes_root)
        except OSError:
            return
        prefix, now = f"task-{self.store.tag}-", time.time()
        for name in names:
            task_text = name[len(prefix):].rsplit("-", 1)[0] if name.startswith(prefix) else ""
            if task_text.isdigit() and self.scene_due(int(task_text), now):
                shutil.rmtree(os.path.join(self.scenes_root, name), ignore_errors=True)

    def finalize(self, task, outcome, attempt_exit, note="", overwrite=False, push_parent=True, **fields):
        """记下这条任务的结局（一个事务里读—判—写，只并进结局那几栏与 fields；已结了局的不再改，弄坏开关 finished-cancel-overwrites 那一支给 overwrite）。
        → 记下了没有。子任务记下之后驱动它的父任务（drive_parent）；正在推父任务的那一步自己撤兄弟片时给 push_parent=False，不再套一层推。"""
        changes = dict(fields, state="finished", outcome=outcome, exit_code=lib.final_exit_code(outcome, attempt_exit), finished_at=round(time.time(), 3), note=note)
        try:
            record = self.store.change("tasks", task["id"], changes,
                                       guard=None if overwrite else (lambda current: "已结了局" if current.get("state") == "finished" else ""))
        except StoreRefused as refused:
            if refused.kind == "tampered":
                self.tampered(task["id"], task, "任务记录")
            elif refused.kind != "guard":
                raise
            return False
        task.clear()
        task.update(record)
        self.forget_prediction(task["id"])   # 收尾就删掉存的那一份预测（文件头 ②「预测每条只算一次」）
        if lib.broken("clean-races-finished-removal", self.environment):   # 弄坏开关：先叫醒 wait 再删，拿到结局的会话接着 task clean 与这一侧同时删
            self.notify_finished(task["id"])
        self.remove_finished_build_directories(task)   # 用户：「服务build之后必须删除历史」——记下结局就删它各次尝试的编译目录
        if not lib.broken("clean-races-finished-removal", self.environment):   # 删完再叫醒 wait（文件头「接口」的 clean）
            self.notify_finished(task["id"])
        if lib.broken("scene-removed-at-finish", self.environment):   # 弄坏开关：现场照「有结局就删」那一刻删（task_server_lib.py 文件头「现场」）
            self.remove_scenes(task["id"])
        say(f"  · 任务 {task['id']} 结局 {outcome}（尝试 {task.get('attempt_count', 0)} 次{'；' + note if note else ''}）")
        if task.get("parent_id") and push_parent:
            self.crash_point(f"child-finished-{task.get('role')}")
            self.advance_parent(task)
        return True

    def advance_parent(self, child):
        """子任务记下结局之后推它的父任务：照库里父任务的阶段与子任务集合现推（drive_parent）；这一步没走到的（服务没了、抛错）由每一轮对账补推。"""
        self.drive_parent(child["parent_id"])

    # ── ④ 分片（文件头「分片」与 task_server_lib.py 文件头「分片」）──
    def children_of(self, parent_id):
        """父任务名下的子任务记录（照关联列 task_id 挑）。有一行校验不过（关联列与正文对不上、写进之后被改过，TS-05）抛 StoreRefused("tampered")，
        说明里点名那几行：调用方照 record-tampered 办（不认、不动、写告警、回 409），不当它没事。弄坏开关 child-rows-unverified 下照改前丢掉校验位、整份交回。"""
        rows = self.store.select("tasks", "task_id = ?", (parent_id,))
        unverified = [record.get("id") for record, valid in rows if not valid]
        if unverified and not lib.broken("child-rows-unverified", self.environment):
            raise StoreRefused("tampered", f"任务 {parent_id} 名下有校验不过的子任务记录（行 {unverified}，关联列与正文对不上或写进之后被改过）")
        return [record for record, _ in rows]

    def children_refused(self, parent_id, refused, consequence):
        """children_of 判出名下有校验不过的子任务行（refused）：照 record-tampered 的处理写告警（tampered_cancel_refusal 的写法），交回话里那一句
        「<哪几行校验不过>：<consequence>」；调用方不认那几行、不往下做，回 409。"""
        detail = f"{refused}：{consequence}"
        if (parent_id, detail) not in self.alerted_child_refusals:   # 同一件同一句只告警一次（status 每读一次都会走到这里）
            self.alerted_child_refusals.add((parent_id, detail))
            self.alert("record-tampered", parent_id, detail, "查是谁直接写了库（只有 task-server 读写它）；那几行对应的单元与目录要动由人照记录核过再动")
        return detail

    def shard_directory(self, parent_id):
        return os.path.join(lib.shards_root(), f"{self.store.tag}-{parent_id}")

    def shard_binaries_removal(self, parent_id, session, process_id):
        """删分片父任务计划目录下的 lib.SHARD_PLAN_BINARIES_NAME（计划那一步预先编好、各片与合并共用的测试二进制；文件头 ④）：父任务有了结局（finalize_parent）与 task clean 时调。
        → (删了的 [{path, bytes}], 没删的 [{path, bytes, kind, why}])；那个子目录不在的交 ([], [])。只删这一个子目录：照真路径要恰好是分片根下 <库标签>-<号>/plan 里的那一份，
        是符号链接或上面哪一层指到别处、真路径对不上的不删，服务日志打一行。删了的写份数（文件个数）与字节数、不删的写原因，各记一行操作日志（action shard-binaries）。
        弄坏开关 shard-binaries-kept 下一律不删，shard-binaries-path-unchecked 下不核路径、照真路径删。"""
        target = os.path.join(self.shard_directory(parent_id), "plan", lib.SHARD_PLAN_BINARIES_NAME)
        if lib.broken("shard-binaries-kept", self.environment) or not os.path.lexists(target):
            return [], []
        resolved = os.path.realpath(target)
        own = os.path.join(os.path.realpath(lib.shards_root()), f"{self.store.tag}-{parent_id}", "plan", lib.SHARD_PLAN_BINARIES_NAME)
        if resolved != own and not lib.broken("shard-binaries-path-unchecked", self.environment):
            why = f"真路径 {resolved} 不是这一条任务自己的计划目录下的那一份（{own}），不删；核过它是什么之后人手删 {target}"
            say(f"  ! 分片的任务 {parent_id} 计划目录下的 {lib.SHARD_PLAN_BINARIES_NAME}：{why}")
            self.operation("shard-binaries", parent_id, session, process_id, f"没删 {target}：{why}")
            return [], [{"path": target, "bytes": 0, "kind": "测试二进制", "why": why}]
        files = sum(len(names) for _, _, names in os.walk(resolved))
        size = directory_bytes(resolved)
        shutil.rmtree(resolved, ignore_errors=True)
        if os.path.lexists(resolved):
            why = "rm 之后还在（权限不对或有进程开着里面的文件）"
            self.operation("shard-binaries", parent_id, session, process_id, f"没删成 {target}（{files} 份、{size} 字节）：{why}")
            return [], [{"path": target, "bytes": size, "kind": "测试二进制", "why": why}]
        self.operation("shard-binaries", parent_id, session, process_id, f"删了 {target}：{files} 份、{size} 字节")
        say(f"  · 分片的任务 {parent_id} 删了计划目录下的 {lib.SHARD_PLAN_BINARIES_NAME}（{files} 份、{size} 字节）")
        return [{"path": target, "bytes": size}], []

    def find_child(self, children, role, index=0):
        return next((child for child in children if child.get("role") == role and int(child.get("shard_index") or 0) == index), None)

    @staticmethod
    def child_compiles(parent, role):
        """这一步子任务服务给不给编译目录（create_child 记进子任务的 compiles；计划那一步的键照它带编不编，lib.shard_child_key）：
        计划、各片、合并跑的是父命令，父任务会编的照给。"""
        return bool(parent.get("compiles")) and role in lib.SHARD_ROLES

    def child_key_text(self, parent, role, index, count):
        """这一步子任务的命令键原文（create_child 建子任务照它）：父任务点了 lib.KEY_VARIABLE 的取它，不然父命令照词折叠，再接这一步那一段
        （计划那一步的那一段带这一步编不编，child_compiles）。"""
        compiles = self.child_compiles(parent, role)
        base = parent["environment"].get(lib.KEY_VARIABLE)
        if not base and lib.broken("shard-key-from-joined-command", self.environment):
            base = " ".join(parent["command"])   # 弄坏开关：用空格接父命令的词，条目名里带空格的后半截漏进键里
        elif not base:
            # 父命令照词折叠（memory_peaks_lib 的 command_key 收词表：带空格的条目名、路径仍是一个词），再接这一步那一段；
            # 截短照这一段的长度先截父任务那一截，接的那一段不被键长上限截掉（截掉了各片落一个键）
            marker = len(lib.shard_child_key("", role, index, count, self.environment, compiles=compiles))
            base = self.peaks.command_key(lib.selection_class_words(lib.cargo_test_class_words(lib.key_words(parent["command"], self.environment), self.environment), self.environment))[:max(0, self.peaks.KEY_LENGTH_LIMIT - marker)]
        return lib.shard_child_key(base, role, index, count, self.environment, compiles=compiles)

    def create_child(self, parent, role, index=0, count=0, label=""):
        """建这一步的子任务；同一父任务同一步同一片已有的交已有的那一个（库里 (parent_id, role, shard_index) 唯一，TS-02）。计划、各片与合并都不钉台，哪台由服务挑。
        父任务 task promote 提前过的，子任务带上同一份 promoted（弄坏开关 promote-shards-refused 下不带）。"""
        existing = self.find_child(self.children_of(parent["id"]), role, index)
        if existing is not None:
            return existing
        command = parent["command"]
        key_text = self.child_key_text(parent, role, index, count)
        environment = dict(parent["environment"], **{lib.KEY_VARIABLE: key_text})
        child = {"command": command, "cwd": parent["cwd"], "environment": environment, "session": parent.get("session", ""),
                 "submitter_pid": parent.get("submitter_pid", 0), "submitted_at": round(time.time(), 3),
                 "command_key": self.command_key(command, key_text), "state": "queued", "outcome": "", "exit_code": None,
                 "collected": True, "next_cap": None, "next_host": "", "outcome_counts": {}, "note": "", "cancel_requested": False, "attempt_count": 0,
                 "task_id": parent["id"], "parent_id": parent["id"], "role": role, "shard_index": index, "shard_count": count,
                 "label": label, "compiles": self.child_compiles(parent, role)}   # 计划、各片、合并跑的是父命令：父任务会编的照给编译目录
        if parent.get("promoted") and not lib.broken("promote-shards-refused", self.environment):
            child["promoted"] = dict(parent["promoted"])   # task promote 提前过的父任务：计划、各片、合并照样排最前（promote）
        try:
            child = self.store.insert("tasks", child)
        except sqlite3.IntegrityError:   # 库里已有同一步同一片：交那一个；按关联列找不到它的（关联列改过，TS-05）当记录改过
            existing = self.find_child(self.children_of(parent["id"]), role, index)
            if existing is None:
                raise StoreRefused("tampered", f"任务 {parent['id']} 的{role} {index} 已在库里、关联列却不指它") from None
            return existing
        step = {"plan": "计划", "merge": "合并"}.get(role, f"第 {index}/{count} 片")
        say(f"  · 任务 {parent['id']} 的{step}排进队：子任务 {child['id']}")
        return child

    def finalize_parent(self, parent, child, outcome, child_exit, note=""):
        failed = {}
        if outcome != "succeeded" and child is not None and not lib.broken("shard-failure-unnamed", self.environment):
            failed = {"task": child["id"], "role": child.get("role"), "index": child.get("shard_index", 0), "count": child.get("shard_count", 0),
                      "maximum_host": child.get("maximum_host", ""), "maximum_cap_text": child.get("maximum_cap_text", "")}
        try:
            parent = self.store.change("tasks", parent["id"], dict(state="finished", outcome=outcome, exit_code=lib.final_exit_code(outcome, child_exit),
                                                                   finished_at=round(time.time(), 3), note=note or (child or {}).get("note", ""), failed_step=failed),
                                       guard=lambda current: "已结了局" if current.get("state") == "finished" else "")
        except StoreRefused as refused:
            if refused.kind == "tampered":
                self.tampered(parent["id"], parent, "任务记录")
            elif refused.kind != "guard":
                raise
            return
        try:   # 有了结局就删计划那一步放的测试二进制（成、败、取消都算；文件头 ④），删完再叫醒 wait；删不成不挡记结局与叫醒
            self.shard_binaries_removal(parent["id"], parent.get("session", ""), 0)
        except (OSError, lib.Unavailable) as error:
            say(f"  ! 分片的任务 {parent['id']} 计划目录下的 {lib.SHARD_PLAN_BINARIES_NAME} 这一回没删成（{type(error).__name__}: {error}）：task clean {parent['id']} 再删")
        self.notify_finished(parent["id"])
        say(f"  · 分片的任务 {parent['id']} 结局 {outcome}{'（' + note + '）' if note else ''}")
        if outcome == "succeeded":   # 整件成了：各片的产物删掉，计划目录留着（它下面的测试二进制上面已删）
            directory = self.shard_directory(parent["id"])
            with contextlib.suppress(OSError):
                for name in os.listdir(directory):
                    if name.startswith("output-"):
                        shutil.rmtree(os.path.join(directory, name), ignore_errors=True)

    def drive_sharded_parents(self):
        """每一轮对账：还没结局的分片父任务各现推一遍（drive_parent；TS-02：子任务收尾与父任务推进之间服务没了、推进半途抛错的，在这里补推）。
        一条推不动（抛错）只写告警、不挡别的。弄坏开关 sharded-parents-unreconciled 下不推（改前的样子：只在子任务收尾那一刻推一次）。"""
        if lib.broken("sharded-parents-unreconciled", self.environment):
            return
        for parent, valid in self.store.select("tasks", "state = 'sharded'"):
            try:
                self.selftest_advance_fault()
                self.drive_parent(parent["id"])
                self.note_advance_recovered(parent["id"])
            except Exception as error:
                self.note_advance_failure(parent["id"], f"{type(error).__name__}: {error}")   # wait 叫醒提交方（文件头 ①「要处置的提醒」）
                say(f"  ✗ 分片的任务 {parent['id']} 这一轮推不动（{type(error).__name__}: {error}）：下一轮再推\n" + traceback.format_exc())
                self.alert("shard-advance-failed", parent["id"], f"分片的任务 {parent['id']} 推下一步时出错：{type(error).__name__}: {error}",
                           "服务每一轮对账照库里的阶段再推；一直推不动就看 journalctl --user -u singlefs-task-server.service 里这一句前后的栈")

    def drive_parent(self, parent_id):
        """照库里父任务的阶段（phase：plan、shards、merge、cancelling）与它的子任务集合现推下一步（文件头 ④；TS-02）。幂等：缺的子任务照计划补建
        （同一步同一片只一个），该收尾的收尾，已结了局的不动。计划读通之后片的计划（shard_plan）写进父任务，各片照它补建。"""
        found = self.store.task(parent_id)
        if found is None:
            return
        parent, valid = found
        if not valid:
            self.tampered(parent_id, parent, "任务记录")
            return
        if parent.get("state") != "sharded":
            return
        try:
            children = self.children_of(parent_id)
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            self.tampered(parent_id, parent, "子任务记录")
            return
        try:
            self.drive_parent_phase(parent, children)
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            self.tampered(parent_id, parent, "子任务记录")

    def drive_parent_phase(self, parent, children):
        """drive_parent 的那一步（文件头 ④）：照阶段推，读到的子任务集合是 children（都核过）。"""
        parent_id = parent["id"]
        phase = parent.get("phase") or "plan"
        if phase == "cancelling":   # 取消没停干净（TS-04）：子任务都有了结局才记整件 cancelled
            if all(child.get("state") == "finished" for child in children):
                self.finalize_parent(parent, None, "cancelled", None, "提交它的会话取消了整件（在跑的子任务都停了）")
            return
        if phase != lib.RUN_HARD_LIMIT_PHASE:
            over = self.parent_run_hard_limit(parent, children)
            if over is not None:
                try:
                    parent = self.store.change("tasks", parent_id, {"phase": lib.RUN_HARD_LIMIT_PHASE, "run_hard_limit_stop": over},
                                               guard=lambda current: "" if current.get("state") == "sharded" and current.get("phase") != "cancelling" else "阶段已变")
                except StoreRefused as refused:
                    if refused.kind == "tampered":
                        raise
                    return
                say(f"  · 分片的任务 {parent_id} {over['how']}，到了 {self.run_hard_limit_seconds:.0f} 秒的硬上限：在排的撤、在跑的停，打回")
                phase = lib.RUN_HARD_LIMIT_PHASE
        if phase == lib.RUN_HARD_LIMIT_PHASE:   # 整件满了四小时硬上限（文件头 ⑦「四小时硬上限」）：在跑的由各自的 check_stall 停，都收尾了才记结局，不合并
            over = parent.get("run_hard_limit_stop") or {}
            self.cancel_queued_children(children, "hard-limit", f"整件到了四小时硬上限，没起：{over.get('how') or '没记'}")
            if all(child.get("state") == "finished" for child in children):
                self.finalize_parent(parent, None, lib.RUN_HARD_LIMIT_OUTCOME, None,
                                     f"{over.get('how') or '没记'}，到了 {self.run_hard_limit_seconds:.0f} 秒的硬上限：在排的子任务撤了、在跑的停了，不合并，--why 放不过")
            return
        if phase == "plan":
            plan = self.find_child(children, "plan")
            if plan is None:   # 父任务记下了、计划那一步还没建（TS-02 的「父任务插入后」）
                self.create_child(parent, "plan")
                self.wake()
                return
            if plan.get("state") != "finished":
                return
            if plan.get("outcome") != "succeeded" and not lib.broken("plan-failure-proceeds", self.environment):   # 弄坏开关：计划没成照拆片
                self.finalize_parent(parent, plan, plan.get("outcome") or "launcher-lost", plan.get("exit_code"), f"计划那一步（子任务 {plan['id']}）{plan.get('outcome')}")
                return
            shards, problem = lib.read_shard_plan(os.path.join(self.shard_directory(parent_id), "plan", lib.PLAN_FILE))
            if problem:
                self.finalize_parent(parent, plan, "shard-plan-refused", None, problem)
                return
            try:
                parent = self.store.change("tasks", parent_id, dict(phase="shards", shard_count=len(shards), shard_plan=shards),
                                           guard=lambda current: "" if current.get("state") == "sharded" and (current.get("phase") or "plan") == "plan" else "阶段已变")
            except StoreRefused:
                return
            phase = "shards"
        if phase == "shards":
            planned = parent.get("shard_plan") or []
            created = False
            for index, shard in enumerate(planned):
                if self.find_child(children, "shard", index) is None:
                    children.append(self.create_child(parent, "shard", index, len(planned), label=shard.get("label", "")))
                    created = True
                    self.crash_point("shard-inserted")
            shards = [self.find_child(children, "shard", index) for index in range(len(planned))]
            failed = self.first_failed_shard(shards)
            if failed is not None:
                note = f"第 {failed.get('shard_index')}/{failed.get('shard_count')} 片（子任务 {failed['id']}）{failed.get('outcome')}"
                if lib.broken("shard-siblings-abandoned", self.environment):   # 弄坏开关：照改前父任务当场收尾，在跑的兄弟片没人收
                    self.finalize_parent(parent, failed, failed.get("outcome") or "launcher-lost", failed.get("exit_code"), note)
                    self.cancel_queued_children(shards, "sibling", f"同一件的子任务 {failed['id']} 没成，不起")
                    return
                # 文件头「分片」：兄弟片照排、照跑到底，结局照收（判过的块照样入库），都收齐了父任务才记那一片的结局，不合并
                if all(shard.get("state") == "finished" for shard in shards):
                    self.finalize_parent(parent, failed, failed.get("outcome") or "launcher-lost", failed.get("exit_code"), note)
                return
            done = [shard for shard in shards if shard.get("state") == "finished" and shard.get("outcome") == "succeeded"]
            skipped = [shard for shard in shards if self.cancelled_alone(shard)]
            if not (len(done) + len(skipped) == len(shards) or (done and lib.broken("merge-before-all-green", self.environment))):
                if created:
                    self.wake()
                return
            self.write_outputs(parent, done)
            missing = sorted(int(shard.get("shard_index") or 0) for shard in skipped)
            try:
                parent = self.store.change("tasks", parent_id, {"phase": "merge", "missing_shards": missing},
                                           guard=lambda current: "" if current.get("state") == "sharded" and current.get("phase") == "shards" else "阶段已变")
            except StoreRefused:
                return
            self.crash_point("merge-outputs-written")
            phase = "merge"
        if phase == "merge":
            merge = self.find_child(children, "merge")
            if merge is None:   # 进了合并那一步、合并的子任务还没建（outputs.tsv 照各片最后一次尝试重写）
                if not os.path.exists(os.path.join(self.shard_directory(parent_id), "outputs.tsv")):
                    self.write_outputs(parent, [child for child in children if child.get("role") == "shard" and child.get("outcome") == "succeeded"])
                self.create_child(parent, "merge", 0, parent.get("shard_count", 0))
                self.crash_point("merge-inserted")
                self.wake()
                return
            if merge.get("state") != "finished":
                return
            if merge.get("outcome") == "succeeded" or lib.broken("merge-failure-succeeds", self.environment):   # 弄坏开关：合并没成照记成了
                self.finalize_parent(parent, merge, "succeeded", merge.get("exit_code"))
            else:
                self.finalize_parent(parent, merge, merge.get("outcome") or "launcher-lost", merge.get("exit_code"), f"合并那一步（子任务 {merge['id']}）{merge.get('outcome')}")

    @staticmethod
    def cancelled_alone(shard):
        """人手单独取消的那一片（cancel 记的 cancel_origin=session，结局 cancelled）：不算没成，合并照起、点名缺它（文件头「分片」）。"""
        return shard.get("state") == "finished" and shard.get("outcome") == "cancelled" and shard.get("cancel_origin") == "session"

    def first_failed_shard(self, shards):
        """最先没成的那一片：有了结局、不是 succeeded，人手单独取消的与兄弟片没成被服务撤掉的（cancel_origin=sibling）不算；照记下结局的先后、同一刻取片号小的。没有交 None。"""
        failed = [shard for shard in shards if shard.get("state") == "finished" and shard.get("outcome") != "succeeded" and not self.cancelled_alone(shard)
                  and not (shard.get("outcome") == "cancelled" and shard.get("cancel_origin") == "sibling")]
        return min(failed, key=lambda shard: (float(shard.get("finished_at") or 0), int(shard.get("shard_index") or 0)), default=None)

    def cancel_queued_children(self, children, origin, note):
        """撤掉 children 里还在排的那几条（兄弟片没成：origin=sibling；弄坏开关 parent-cancel-spares-running 下整件被取消也走这里）：拿那一条的锁、
        在一个事务里判还 queued 才记 cancel_requested 与 cancel_origin（起单元那一侧拿同一把锁、claim_attempt 判 cancel_requested，记上之后就不会再起），
        再记 cancelled；判的时候已经起了的不动，照跑到底。记了的那一份字典就地换成库里的新样子。"""
        for child in children:
            if child.get("state") != "queued":
                continue
            with self.store.task_lock(child["id"]):
                try:
                    current = self.store.change("tasks", child["id"], {"cancel_requested": True, "cancel_origin": origin},
                                                guard=lambda row: "" if row.get("state") == "queued" else "不在排")
                except StoreRefused as refused:
                    if refused.kind == "tampered":
                        raise
                    continue
                if self.finalize(current, "cancelled", None, note, push_parent=False):
                    child.clear()
                    child.update(current)

    def write_outputs(self, parent, shards):
        """合并那一步（文件头「分片」③）读的 outputs.tsv：各片最后一次尝试的产物目录（先写临时文件再改名，重写也是同一份）。"""
        lines = []
        for shard in sorted(shards, key=lambda record: record.get("shard_index", 0)):
            attempts = [attempt for attempt, _ in self.store.attempts_of(shard["id"])]
            last = attempts[-1] if attempts else {}
            lines.append(f"{shard.get('shard_index', 0)}\t{last.get('host', '')}\t{last.get('shard_output', '')}")
        directory = self.shard_directory(parent["id"])
        lib.ensure_private_directory(directory)
        outputs = os.path.join(directory, "outputs.tsv")
        temporary = outputs + ".partial"
        with open(temporary, "w", encoding="utf-8") as handle:
            handle.write("".join(line + "\n" for line in lines))
        os.replace(temporary, outputs)

    def whole_threshold(self, host, snapshot):
        """整片的界（字节）：这台可用上限（总上限减这台此刻的余量，host_capacity；booked_at_cap 收到的那个数）× WHOLE_FRACTION（oversized-budget 提醒的界）。
        只对这一轮派得了的台调（place 先滤掉了余量量不到的台）；量不到的抛 ValueError，不退回别的数。"""
        capacity, why = self.host_capacity(host, snapshot)
        if capacity is None:
            raise ValueError(f"整片的界算不了：{why}")
        return int(capacity * WHOLE_FRACTION)

    def is_whole(self, booked, host, snapshot):
        return int(booked or 0) >= self.whole_threshold(host, snapshot)

    def aged(self, task, now):
        """提交满 aged_seconds 还在排：高优先（文件头「排队」）。task promote 提前过的不在这里：它排最前（queue_order）、不过 CPU 闸（cpu_judgement），内存放得下就起，不挡后面
        （弄坏开关 promote-aged-blocks-queue 下提前过的排满时限照算，起不来就挡住后面全部）。"""
        if task.get("promoted") and not lib.broken("promote-aged-blocks-queue", self.environment):
            return False
        return now - float(task.get("submitted_at") or now) >= self.aged_seconds and not lib.broken("aged-task-not-prioritized", self.environment)

    def queue_order(self, queued, now):
        """排队的先后：task promote 提前过的最前，再是高优先的，各自按原来的次序（排序是稳定的）。弄坏开关 promote-queue-order-ignored 下不看提前过没有。"""
        promoted_ignored = lib.broken("promote-queue-order-ignored", self.environment)
        return sorted(queued, key=lambda row: (promoted_ignored or not row[0].get("promoted"), not self.aged(row[0], now)))

    def promote(self, task_id, session, process_id):
        """task promote <号>（客户端那一条执行前由钩子弹窗要用户确认）→ (HTTP 状态码, 回话)：只许提交它的会话；任务要在排（queued）；
        记 promoted {at}，下一轮排队起它排最前（queue_order）、不过 CPU 闸（cpu_judgement，内存放得下就起）。
        提前过的再提前照 200、不改。分片确认过的父任务（sharded）同样许提前：父任务记 promoted，此刻在排的子任务各记一份，之后建的计划、各片、合并照父任务继承（create_child）。
        改完当场叫醒排队线程（wake），不干等下一轮。
        弄坏开关 promote-session-unchecked 下不看是不是提交它的会话，promote-unwoken 下不叫醒，promote-shards-refused 下分片的父任务照「不在排」拒。"""
        found = self.store.task(task_id)
        if found is None:
            return 404, {"detail": f"没有任务 {task_id}"}
        task, valid = found
        if not valid:
            self.tampered(task_id, task, "任务记录")
            return 409, {"detail": f"任务 {task_id} 的记录校验不过，已记 record-tampered，不提前"}
        if task.get("session") != session and not lib.broken("promote-session-unchecked", self.environment):
            self.operation("promote-refused", task_id, session, process_id, "不是提交它的会话")
            return 403, {"detail": f"任务 {task_id} 不是这个会话提交的，不许提前"}
        if task.get("promoted"):
            return 200, {"state": task.get("state"), "detail": f"任务 {task_id} 原来就提前过"}
        promotable = ("queued",) if lib.broken("promote-shards-refused", self.environment) else ("queued", "sharded")
        mark = {"promoted": {"at": round(time.time(), 3)}}
        try:
            self.store.change("tasks", task_id, mark, guard=lambda current: "" if current.get("state") in promotable else "不在排")
        except StoreRefused as refused:
            if refused.kind != "guard":
                raise
            return 409, {"detail": f"任务 {task_id} 此刻 {task.get('state')}、不在排，不用提前"}
        if task.get("state") == "sharded":   # 分片的父任务：此刻在排的子任务各记一份（之后建的照父任务继承，create_child）
            for child in self.children_of(task_id):
                with contextlib.suppress(StoreRefused):
                    self.store.change("tasks", child["id"], mark, guard=lambda current: "" if current.get("state") == "queued" and not current.get("promoted") else "不在排")
        self.operation("promote", task_id, session, process_id, "提前到队头")
        if not lib.broken("promote-unwoken", self.environment):
            self.wake()   # 当场排一轮，不干等 TICK_SECONDS
        return 200, {"state": task.get("state"), "detail": f"任务 {task_id} 提前到队头：下一轮排队起排最前、不过 CPU 闸，内存放得下就起"}

    def positions_in(self, ordered, now):
        """排好先后的在排任务各排第几：{任务号: (排第几（从 1 起）, 在排共几条, 排在它前面而已排满时限的几条)}。
        先后就是 queue_order 给的（排满时限的在前、各自按任务号）；排满时限还起不来的那一条挡住后面全部（文件头 ②「排队先后」①），所以第三个数另报。"""
        positions, aged_ahead = {}, 0
        for index, row in enumerate(ordered):
            positions[row[0]["id"]] = (index + 1, len(ordered), aged_ahead)
            if self.aged(row[0], now):
                aged_ahead += 1
        return positions

    def queue_position_reply(self, task):
        """status 里排着的任务的全局排位：{position, queued, aged_ahead}；不在上一轮的排队表里（刚进队还没排过一轮、不在 queued）交 None。"""
        found = self.queue_positions.get(task["id"]) if task.get("state") == "queued" else None
        return None if found is None else {"position": found[0], "queued": found[1], "aged_ahead": found[2]}

    def head_judged_apart(self, pass_state=None):
        """只给弄坏开关 health-head-judged-apart 与 health-judges-head：照改前 /health 自己另判一套排头放不放得下，交 HeadVerdict（队空交 None）。
        另判的只看内存（judged_on_host），不看 CPU 闸与编译目录占盘（任务 7568 卡在 CPU 闸、2365 等一件 /health 不认得的事，都被报成放得下却没起）。
        pass_state 给了照它取排头、用它的余量；没给（health-judges-head 下 /health 现调）就现读库与余量。"""
        if pass_state is None:
            queued = [row for row in self.store.select("tasks", "state = 'queued'", order="id") if row[1]]
            view = None
        else:
            queued = sorted((row for row in pass_state["queued"] if row[1] and row[0]["id"] not in pass_state["gone"]), key=lambda row: row[0]["id"])
            view = pass_state["view"]
        if not queued:
            return None
        head = self.queue_order(queued, time.time())[0][0]
        view = view or PassView(self, "health")
        snapshot = view.snapshot()
        if snapshot.hold:
            return HeadVerdict(head["id"], HEAD_BLOCKED, "另判：自证的假余量写了 hold")
        prediction = self.prediction_of(head)
        for host in self.candidate_hosts(head, snapshot):
            if host not in snapshot.reports or self.host_limit(prediction, host, snapshot)[0] is None:
                continue   # 余量量不到的台这一轮不派（文件头 ②「余量」）
            verdict = self.judged_on_host(head, prediction, host, snapshot, view.pending_of(host))[1]
            if verdict.fits:
                return HeadVerdict(head["id"], HEAD_NOT_STARTED, "另判：内存放得下")
        return HeadVerdict(head["id"], HEAD_BLOCKED, "另判：放不下")

    def starved_problem(self, now, fresh=False):
        """派活判排头能起却连着 starved_seconds 没起成：交一句问题，不然交空串。只读派活这一轮给排头的结论（head_verdict，place_queued 记下），不另判放不放得下：
        派活判不能起的（等负载、等占盘、排在排满时限的任务后面、第二台这一轮起满、起之前复核没过，原因都在排队原因里）与判不了的一律不报，不认排队原因里的字样。
        服务暂停着（文件头 ⑤「暂停」）排着不起是要的，交空串；弄坏开关 pause-judges-starved 下照改前另判（暂停时排队线程不判，没有结论可读）。
        弄坏开关 health-head-judged-apart 下照改前另判一套（head_judged_apart，用这一轮的余量）；health-judges-head 下由 /health 现调、fresh 为真：另判一套且现读库与余量；
        blocked-head-counted-as-starved 下派活判不能起的排头也当成判能起没起成。"""
        if self.pause is not None:
            if not lib.broken("pause-judges-starved", self.environment):
                self.fits_since = {}
                return ""
            fresh = True
        if fresh or lib.broken("health-head-judged-apart", self.environment):
            try:
                verdict = self.head_judged_apart(None if fresh else self.pass_state)
            except (OSError, ValueError, KeyError, subprocess.SubprocessError):
                return ""
        else:
            verdict = self.head_verdict
        startable = verdict is not None and (verdict.kind == HEAD_NOT_STARTED
                                             or (verdict.kind == HEAD_BLOCKED and lib.broken("blocked-head-counted-as-starved", self.environment)))
        if not startable:
            self.fits_since = {}
            return ""
        since = self.fits_since.get(verdict.task_id, now)
        self.fits_since = {verdict.task_id: since}
        if now - since > self.starved_seconds and not lib.broken("starved-head-unreported", self.environment):
            return f"任务 {verdict.task_id} 排在最前、派活判它此刻放得下，却 {now - since:.0f} 秒没起成（{verdict.why}）"
        return ""

    def progress_problem(self, now):
        """看进展（文件头「接口」；派活判法自己判错时要有人看得到：队列卡死 4 个多小时、51 条排着只跑 3 条那一次）：有排着的任务、某台此刻照用户定的公式
        （room_by_the_formula）放得下排着的最小那条的记账，而这台连着 progress_seconds 一条都没起，交一句点名哪台、照公式还能记多少、最小那条是哪条；不然交空串。
        不读派活的结论（head_verdict、排队原因、Placement），只读这一轮排队读的余量（PassView，不另读）、库里在排的任务与起没起（progress_started_hosts）。
        派活本来就该等的几种各自照余量另外不算（progress_smallest_fitting）。服务暂停着不判（排着不起是要的），判不了交空串。
        弄坏开关 progress-unwatched 下一律交空串。"""
        try:
            started = self.progress_started_hosts()
            if self.pause is not None or lib.broken("progress-unwatched", self.environment):
                self.progress_since = {}
                return ""
            for host in started:
                self.progress_since.pop(host, None)   # 这台这一轮起过：从此刻重新算
            view = (self.pass_state or {}).get("view")
            queued = [task for task, valid in self.store.select("tasks", "state = 'queued'", order="id") if valid] if view is not None else []
            if not queued:
                self.progress_since = {}
                return ""
            human = self.host_dispatch.human
            problems = []
            for host in MACHINE_ORDER:
                fitting = self.progress_smallest_fitting(host, queued, view)
                if fitting is None:
                    self.progress_since.pop(host, None)
                    continue
                since = self.progress_since.setdefault(host, now)
                if now - since >= self.progress_seconds:
                    room, task_id, booked = fitting
                    problems.append(f"{self.machines[host].label}照公式（{self.formula_label()}）还能记 {human(room)}、排着的最小那条任务 {task_id}（记 {human(booked)}）放得下，"
                                    f"这台却 {now - since:.0f} 秒一条都没起（看进展，不看派活的结论）")
            return "；".join(problems)
        except (OSError, ValueError, KeyError, subprocess.SubprocessError, sqlite3.Error, StoreRefused):
            return ""

    def progress_started_hosts(self):
        """看进展：尝试号比上一轮数到的最大号大的那几次尝试起在哪几台（起了又收尾的也数得到），交台的集合；头一回调只记下此刻的最大号、交空集。"""
        seen = self.progress_attempt_seen
        top, hosts = seen or 0, set()
        for attempt, _ in self.store.select("attempts", "id > ?", (seen if seen is not None else 0,)):
            top = max(top, int(attempt["id"]))
            if seen is not None:
                hosts.add(attempt.get("host"))
        self.progress_attempt_seen = top
        return hosts

    def progress_smallest_fitting(self, host, queued, view):
        """看进展的一台：照公式这台还能记的（room_by_the_formula）放得下排着的最小那条的记账，交 (还能记的, 那条的任务号, 它在这台的记账)，放不下或不判交 None。
        不判的：自证的 hold、这台余量量不到或连不上；这台 slice 兜底停派着（slice_floor_problem）。
        挑最小那条时跳过派不到这台的、这台给不出上限的、预测峰值比这台大而别处可能装得下的（hosts_short_of_record：这台不派它），与本来就该等的：
        这台负载过 CPU 闸（cpu_problem）、编译目录的盘不够（build_disk_problem）、slice 实测不到下限（slice_floor_problem）。
        弄坏开关 progress-gates-ignored 下照公式放得下就算：不看 CPU、占盘与 slice 兜底。"""
        snapshot = view.snapshot()
        if snapshot.hold or host not in snapshot.reports or not snapshot.reachable.get(host):
            return None
        room = self.room_by_the_formula(host, snapshot)
        if room is None:
            return None
        gated = not lib.broken("progress-gates-ignored", self.environment)
        if gated and (self.slice_floor_problem(host, snapshot) or self.machine_memory_problem(host, snapshot)):
            return None   # slice 兜底、整机内存线停派着（slice_floor_problem、machine_memory_problem）：本来就该等
        smallest = None
        for task in queued:
            if host not in self.candidate_hosts(task, snapshot):
                continue
            prediction = self.prediction_of(task)
            if self.host_limit(prediction, host, snapshot)[0] is None:
                continue
            if host in self.hosts_short_of_record(prediction, self.candidate_hosts(task, snapshot), snapshot):
                continue   # 这台不派它：预测峰值比这台大、别处可能装得下（judge_placement 照同一处判）
            if gated:
                reason = self.waiting_reasons.get(task["id"], "")
                if "已排满时限" in reason or "不插队" in reason:
                    continue   # 照排队规则本来就该等的：排在排满时限的任务后面，或前面有没起的而它不许插队（place_queued_into、jump_refusal）
                if self.cpu_problem(task, host, snapshot) or self.build_disk_problem(task, host, snapshot, view):
                    continue   # 等负载、等占盘
            booked = int(self.booked_at_cap(prediction, host, snapshot).memory_booked or 0)
            if smallest is None or booked < smallest[1]:
                smallest = (task["id"], booked)
        if smallest is None or smallest[1] > room:
            return None
        return room, smallest[0], smallest[1]

    def place_queued(self):
        """排一轮；排队原因写进一张新表、这一轮排完才换上（status 读不到排了一半的表）。派活给这一轮每条的结论（HeadVerdict）同样记下，
        排头（这一轮头一条判了的：排队的先后，收尾了的、记录坏了的不算）那一条换进 head_verdict，/health 判「判能起却没起成」只读它。"""
        waiting, verdicts = {}, {}
        try:
            self.place_queued_into(waiting, verdicts)
        finally:
            self.waiting_reasons = waiting
            self.head_verdict = next(iter(verdicts.values()), None)

    def place_queued_into(self, waiting, verdicts):
        """排一轮的本体：waiting 收每条排着的在等什么，verdicts 照排队的先后收派活给每条的结论（{任务号: HeadVerdict}，每条只记头一个）。
        自证的 TASK_SERVER_SELFTEST_PLACEMENT_PAUSED 那份文件在时照判、判能起的不起（记判能起却没起成）；TASK_SERVER_SELFTEST_PLACEMENT_REFUSED 那份文件在时
        每条都判成不能起（演派活判法判错，看进展那一道该报）。"""
        self.pass_state = None   # 这一轮没排就是 None
        # 第二台起满了才隔 PEER_REPASS_SECONDS 再排（repass_soon）：每一轮开头先清，队空、暂停着提前回的那几轮不再照上一轮的留着、主循环不每 2 秒空转
        # （loop 审核发现 12）；弄坏开关 repass-kept-when-idle 下照队里有活的那几轮才清
        if not lib.broken("repass-kept-when-idle", self.environment):
            self.repass_soon = False

        def judged(task, kind, why=""):
            verdicts.setdefault(task["id"], HeadVerdict(task["id"], kind, why))

        pause = self.pause_in_force()
        queued = self.store.select("tasks", "state = 'queued'", order="id DESC" if lib.broken("queue-reversed", self.environment) else "id")
        # 全队（暂停着也是全队）：排位（queue_positions）与留着的预测（keep_predictions_of）照它算，别的会话排着的任务排位不丢、预测不清。
        # 弄坏开关 pause-queue-positions-filtered 下照放行的那几条算
        whole = queued
        if pause is not None:   # 服务暂停着（文件头 ⑤「暂停」）：只排放行的会话交的，别的会话的这一轮不起；排着的任务在等什么由 waiting_text 照暂停写
            queued = [row for row in queued if self.allowed_during_pause(row[0], pause)]
            if lib.broken("pause-queue-positions-filtered", self.environment):
                whole = queued
            if not queued:
                return
        if not queued:
            self.pass_state = {"queued": [], "gone": set(), "view": None}
            self.selftest_read("pass view=- queued=0")   # 队空的一轮也记（自证的 next_pass 等它）
            return
        now = time.time()
        self.repass_soon = False
        started_on = {}   # {台: 这一轮在那台真起了几条（尝试在 starting 或 running）}：有的台一轮至多起 Machine.starts_per_pass 条
        queued = self.queue_order(queued, now)
        self.queue_positions = self.positions_in(self.queue_order(whole, now), now)
        awaiting = {task["id"] for task, _ in self.store.select("tasks", "state = ?", (lib.AWAITING_CONFIRMATION_STATE,))}   # 确认单算过的预测留着，进队之后照用（文件头 ⑦）
        self.keep_predictions_of({task["id"] for task, _ in whole} | awaiting)   # 不在队里了的（起了、收尾了）删掉存的预测
        view = PassView(self, "pass")   # 这一轮的余量：每台读一次、各条共用（文件头 ②「余量一轮只读一次」）
        gone = set()   # 这一轮起了或收尾了的任务号（判排头时跳过它们）
        self.pass_state = {"queued": queued, "gone": gone, "view": view}
        self.selftest_read(f"pass view={view.number} queued={len(queued)}")
        # 排队只照两条规则：排满时限还起不来的挡住后面全部；前面有没起的，后面的只许比在跑的剩余时长短的插（jump_refusal）

        def behind(position, why):
            for later, _ in queued[position + 1:]:
                waiting[later["id"]] = why

        earlier_blocked = []   # 这一轮判过、还没起的任务号（照排队先后）：它后面的任务要起就是插队，照 jump_refusal 判
        for position, (task, valid) in enumerate(queued):
            if not valid:
                self.tampered(task["id"], task, "任务记录")
                gone.add(task["id"])
                continue
            try:
                if self.placement_refused and os.path.exists(self.placement_refused):
                    placement = Placement("blocked", why="自证：TASK_SERVER_SELFTEST_PLACEMENT_REFUSED 那份文件在，派活一律判不能起（演派活判法判错）")
                else:
                    placement = self.place(task, view)
            except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
                say(f"  ! 任务 {task['id']} 这一轮判不了放不放得下（{type(error).__name__}: {error}），留在队里")
                waiting[task["id"]] = f"这一轮判不了放不放得下（{type(error).__name__}: {error}），下一轮再判"
                judged(task, HEAD_BLOCKED, waiting[task["id"]])
                if lib.broken("judge-error-blocks-queue", self.environment):
                    behind(position, f"排在任务 {task['id']} 后面：它这一轮判不了放不放得下")
                    return   # 弄坏开关：照改前一条判不了就挡住整队
                continue   # 只跳过它这一轮，后面的照判（一条记录坏了不该把整队挡住）
            if placement.kind == "never":
                self.finalize(task, "never-fits", None, placement.why)
                gone.add(task["id"])
                continue
            if placement.kind == "unmet":
                self.finalize(task, "run-condition-unmet", None, placement.why)
                gone.add(task["id"])
                continue
            self.budget_check(task, view)   # 文件头 ②「预算」：记账贴着某一台整片界的记一条 oversized-budget
            if placement.kind == "blocked":
                waiting[task["id"]] = placement.why
                judged(task, HEAD_BLOCKED, placement.why)
                if self.blocks_later(task):
                    earlier_blocked.append(task["id"])
                if self.aged(task, now) and not lib.broken("aged-blocked-lets-others-start", self.environment):
                    # 排满时限（两小时）还起不来的这一条挡住排在它后面的全部任务：它起来之前，后面的哪一台都不起。
                    # 不这样，线以下一直有要得少的任务补位，要得多的这一条永远够不着（任务 14789 要 31 线程，排了三个半小时一次没起）。
                    behind(position, f"排在任务 {task['id']} 后面：它已排满时限（{self.aged_seconds / 3600:.1f} 小时）还没起，它起来之前后面的都不起")
                    return
                if lib.broken("backfill-disabled", self.environment):
                    behind(position, f"排在任务 {task['id']} 后面")
                    return   # 弄坏开关：退回严格排队，排头放不下后面的都等
                continue
            machine = self.machines[placement.host]
            if (machine.starts_per_pass is not None and started_on.get(placement.host, 0) >= machine.starts_per_pass
                    and not lib.broken("peer-starts-unthrottled", self.environment)):
                waiting[task["id"]] = f"{machine.label}这一轮已起 {machine.starts_per_pass} 条（一齐起太多 ssh 会被重置），{PEER_REPASS_SECONDS:.0f} 秒后再排"
                judged(task, HEAD_BLOCKED, waiting[task["id"]])
                self.repass_soon = True
                continue
            if self.placement_paused and os.path.exists(self.placement_paused):
                judged(task, HEAD_NOT_STARTED, "自证：TASK_SERVER_SELFTEST_PLACEMENT_PAUSED 那份文件在，排队线程判完不起（起之前复核也不做）")
                continue   # 自证：照判、不起
            placement, why = self.launch_recheck(task, placement, view)
            if placement is None:   # 起之前现读的数放不下：留在队里（文件头 ②「起之前复核」）
                waiting[task["id"]] = f"起之前复核没过（现算的上限、现读的余量）：{why}"
                judged(task, HEAD_BLOCKED, waiting[task["id"]])
                if self.blocks_later(task):
                    earlier_blocked.append(task["id"])
                if self.aged(task, now) and not lib.broken("aged-blocked-lets-others-start", self.environment):
                    behind(position, f"排在任务 {task['id']} 后面：它已排满时限（{self.aged_seconds / 3600:.1f} 小时）还没起，它起来之前后面的都不起")
                    return
                continue
            if earlier_blocked and not lib.broken("jump-length-unchecked", self.environment):
                # 排在它前面的任务还没起，这一条要起就是插队：只许有上次运行时长、且不超过这台在跑的任务剩余时长的插（jump_refusal）
                jump = self.jump_refusal(task, placement.host, earlier_blocked[0], now)
                if jump:
                    waiting[task["id"]] = jump
                    judged(task, HEAD_BLOCKED, jump)
                    earlier_blocked.append(task["id"])
                    continue
            attempt, held = self.start_attempt_unless_paused(task, placement)
            if held:
                return   # 判完放得下、起之前服务暂停了（pause 与这一处共锁）：这一轮到此为止，一条都不再起
            gone.add(task["id"])
            self.selftest_read(f"start task={task['id']} host={placement.host} view={view.number}")   # 自证：哪一轮在哪台起的（peer-starts-throttled 照它数每轮起几条）
            if attempt is not None and attempt.get("state") in ACTIVE_ATTEMPT_STATES:
                judged(task, HEAD_STARTED)
                view.book(placement.host, placement.booked, placement.disk_booked)   # 这一轮后面的照加过的判（文件头 ②「余量一轮只读一次」）
                started_on[placement.host] = started_on.get(placement.host, 0) + 1   # 只数真起了的：没起成的（取消了、systemd-run 起不来）不占这一轮的名额
            else:
                judged(task, HEAD_NOT_STARTED, f"起尝试没成：{'没记下尝试' if attempt is None else '尝试 ' + str(attempt.get('state')) + '、' + str(attempt.get('outcome'))}")
                if lib.broken("failed-start-counted", self.environment):   # 弄坏开关：照改前没起成的也算进这一轮在那台起了几条
                    started_on[placement.host] = started_on.get(placement.host, 0) + 1

    def blocks_later(self, task):
        """这一轮判过、没起的这一条算不算「排在前面还没起的」（place_queued_into 的 earlier_blocked：它后面的要起就是插队，照 jump_refusal 判）：
        task promote 提前过的不算（它排最前，起不来不挡后面）。弄坏开关 promote-blocked-counts-as-earlier 下提前过的也算。"""
        return not task.get("promoted") or lib.broken("promote-blocked-counts-as-earlier", self.environment)

    def jump_refusal(self, task, host, head, now):
        """插队规则：排在前面的任务 head 还没起，这一条想先在 host 上起。许插交空串，不许交排队原因。
        在跑的都算不出剩余时长时，有上次运行时长的许插。算得出时两条都满足才许：这一条有上次的运行时长（last_run_seconds：这把命令键上次真跑完用了多久，是记录、不估）；这个时长不超过 host 上在跑的任务剩余时长里最长的那个
        （剩余 = 那条任务上次的运行时长减已跑；没有上次时长、或已经跑过上次时长的不算数）。这样插进去的在挡着排头的那几条跑完之前就结束，不耽误排头。"""
        estimate = self.last_run_seconds(task)[0]   # 这把命令键上次真跑完用了多久：记录，不是估的
        if estimate is None:
            return f"排在任务 {head} 后面、它还没起：这一条没有上次的运行时长，不插队"
        remaining = []
        for attempt in self.booking_attempts():
            if attempt.get("host") != host:
                continue
            found = self.store.task(attempt.get("task_id"))
            expected = self.last_run_seconds(found[0])[0] if found else None
            elapsed = max(0.0, now - float(attempt.get("started_at") or now))
            if expected is not None and expected > elapsed:
                remaining.append(expected - elapsed)
        label = self.machines[host].label
        if not remaining:
            return ""   # 在跑的都算不出剩余时长（第一次跑、没有上次运行时长）：有上次运行时长的这一条许插（用户定）；队头排满时限之后照样把后面全挡住
        if estimate > max(remaining):
            return (f"排在任务 {head} 后面、它还没起：这一条上次跑了 {estimate:.0f} 秒，超过{label}在跑的任务剩余的时长（最长 {max(remaining):.0f} 秒），不插队")
        return ""

    def host_eligible(self, task, host, snapshot):
        """这一条派不派得到这一台（task_server_lib.py 文件头「挑机器」）：只看这一台此刻连不连得上（Snapshot.reachable），余量由 place 判；共享根外的路径、共享根、
        判定库三种在提交时就拒了（文件头 ②「提交时拒」），这里不再判。没有只派本机的名单：哪个脚本派到第二台跑红了由脚本那一侧修。
        第二台在同一棵共享的树里、同一个绝对路径上跑：改主工作区的工具照样派得过去。"""
        if not snapshot.reachable.get(host):
            return False
        if self.machines[host].always_reachable:
            return True   # 下面两个弄坏开关照改前只派服务所在这一台
        if self.restored_local_only(task):
            return False   # 弄坏开关 host-lists-restored：照改前点到只在本机的名单的不派第二台
        if lib.broken("workspace-writers-local-only", self.environment) and self.modifies_workspace(task):
            return False   # 弄坏开关：照改前把改主工作区的工具只派本机
        return True

    def restored_local_only(self, task):
        """弄坏开关 host-lists-restored 下：命令点到删之前只在本机的那几张名单（RESTORED_HOST_LISTS 的目录与入口；bash -c 这类整条写在一个词里的按词里出现的目录认）交 True；
        开关没开一律 False。"""
        if not lib.broken("host-lists-restored", self.environment):
            return False
        root = os.path.realpath(self.peer_root)
        cwd = os.path.realpath(task["cwd"])
        directories = RESTORED_HOST_LISTS["local_directories"]
        prefixes = [os.path.join(root, directory) + os.sep for directory in directories]
        if (any(os.path.normpath(os.path.join(cwd, word)).startswith(prefix) for word in task["command"] for prefix in prefixes)
                or any(re.search(r"(?:^|[\s\x27\x22=;|&(<>/])" + re.escape(directory) + "/", word) for word in task["command"] for directory in directories if " " in word)):
            return True
        relative = [os.path.relpath(os.path.normpath(os.path.join(cwd, word)), root) for word in task["command"] if not word.startswith("-")]
        return any(path in RESTORED_HOST_LISTS["local_entries"] for path in relative)

    def modifies_workspace(self, task):
        """这条命令改不改主工作区（改了跑第二遍结果就不同，撞上限不原样重排）：WORKSPACE_WRITERS 里的工具；合入后验证（MERGE_AND_VERIFY_ENTRY）不算，
        弄坏开关 merge-and-verify-counted-as-writer 下照算（撞上限不重排、中断了报 workspace-write-interrupted）；
        hook-swap.py 带 --feed-only 的只喂不换、不写文件，不算（任务 2461：喂钩子撞了 44M 被当成改工作区打回）。弄坏开关 feed-only-counted-as-writer 下照算。
        带了 WORKSPACE_READ_ONLY_ARGUMENTS 之一的只读不写，不算（apply-writer-patch.py --dry-run 只在临时树上核：任务 7442 撞 476M、7444 撞 4096M，都被当成改工作区打回；
        自证、列项、读结局同理：任务 7543、7549）。弄坏开关 read-only-arguments-counted-as-writer 下照算。"""
        relative = self.command_paths_in_repository(task)
        if (any(argument in WORKSPACE_READ_ONLY_ARGUMENTS for argument in task["command"])
                and not lib.broken("read-only-arguments-counted-as-writer", self.environment)):
            return False
        # 合入后验证（merge-and-verify.sh）不算改主工作区：它只在自己拷的快照上打补丁与验，主工作区一个字节不写（它文件头「快照（不拿锁）」），撞了上限照常放大重排
        if MERGE_AND_VERIFY_ENTRY in relative and lib.broken("merge-and-verify-counted-as-writer", self.environment):
            return True
        return self.workspace_swap_tool(task)

    def command_paths_in_repository(self, task):
        """命令里不带 - 的词当路径，照任务的工作目录折成相对本仓根的路径（modifies_workspace、workspace_swap_tool 认脚本用）。"""
        root = os.path.realpath(self.peer_root)
        cwd = os.path.realpath(task.get("cwd") or root)
        return [os.path.relpath(os.path.normpath(os.path.join(cwd, word)), root) for word in task["command"] if not word.startswith("-")]

    def workspace_swap_tool(self, task):
        """命令是不是 WORKSPACE_WRITERS 里的工具在换文件、打补丁（换钩子 hook-swap.py、打补丁 apply-writer-patch.py、stage-mine.py）：这几个工具把结局文件写到一半被杀，之后所有会话的换上都被挡
        （workspace_writer_floor 只给它们留余量）。不含合入后验证 merge-and-verify（不写主工作区，它的内存是编译与测试的，照这把键的记录定上限）。
        带 WORKSPACE_READ_ONLY_ARGUMENTS 之一的只读不写、hook-swap.py 带 --feed-only 的只喂不换，都不算；对应的弄坏开关 read-only-arguments-counted-as-writer、
        feed-only-counted-as-writer 下照算。"""
        relative = self.command_paths_in_repository(task)
        if (any(argument in WORKSPACE_READ_ONLY_ARGUMENTS for argument in task["command"])
                and not lib.broken("read-only-arguments-counted-as-writer", self.environment)):
            return False
        if HOOK_SWAP_ENTRY in relative and "--feed-only" in task["command"] and not lib.broken("feed-only-counted-as-writer", self.environment):
            return False
        return any(path in WORKSPACE_WRITERS for path in relative)

    def placement_refusal(self, command, cwd, environment):
        """提交时拒（文件头 ②「提交时拒」）：依次判 ① 共享根外的路径、② 共享根、③ 判定库、④ 临时 prune-server 包装里会编的、⑤ 不点名包的全仓编译，
        命中头一种交 {problem: 那一句, next_step: 出路}，都没命中交 None。
        environment 是这一条带进任务的环境（split_environment 收下的那几个）。五个弄坏开关各关一种。"""
        if not lib.broken("outside-path-accepted", self.environment):
            refusal = self.find_outside_shared_path(command, cwd)
            if refusal:
                return refusal
        if not lib.broken("shared-roots-outside-accepted", self.environment):
            refusal = self.shared_roots_problem(environment)
            if refusal:
                return refusal
        if not lib.broken("verdict-library-family-accepted", self.environment):
            refusal = self.verdict_library_refusal(command, cwd)
            if refusal:
                return refusal
        if not lib.broken("temporary-service-heavy-accepted", self.environment):
            refusal = self.temporary_service_heavy_refusal(command, cwd, environment)
            if refusal:
                return refusal
        if not lib.broken("whole-workspace-cargo-accepted", self.environment):
            return self.whole_workspace_cargo_refusal(command, cwd, environment)
        return None

    def manual_refusal(self, manual, session):
        """人手交的认法（异常表 manual-from-session、sessionless-unmarked）：会话照进程树读（session_of_peer），客户端说了不算。带 --manual 而进程树上有会话的拒，
        会话不靠它躲开推送；没有会话又没带 --manual 的拒，随手填的收件地址换不来进队。合的交 None。
        弄坏开关 manual-from-session-accepted、sessionless-unmarked-accepted 下各不判。"""
        if manual and session and not lib.broken("manual-from-session-accepted", self.environment):
            return {"problem": f"带了 --manual，而这条命令是会话 {session} 起的：这一条不收、不跑", "next_step": lib.anomaly_advice("manual-from-session", "", self.environment)}
        if not manual and not session and not lib.broken("sessionless-unmarked-accepted", self.environment):
            return {"problem": "这条命令不是从 Claude Code 会话起的，也没带 --manual：这一条不收、不跑", "next_step": lib.anomaly_advice("sessionless-unmarked", "", self.environment)}
        return None

    def full_gate_refusal(self, command, cwd, full_gate):
        """全量门禁标识（task submit --full-gate）只认提交时的门禁、不带 --check 与 --item（lib.commit_gate_entry）：别的命令带它交 {problem, next_step}；没带或合的交 None。
        弄坏开关 full-gate-any-command-accepted 下不判。"""
        if not full_gate or lib.commit_gate_entry(list(command), cwd) or lib.broken("full-gate-any-command-accepted", self.environment):
            return None
        return {"problem": f"带了 --full-gate，而 {shlex.join(command)[:200]} 不是不指定范围的提交时门禁：这一条不收、不跑",
                "next_step": lib.anomaly_advice("full-gate-not-commit-entry", "", self.environment)}

    def whole_package_refusal(self, command, cwd, environment, confirmation_path, full_gate=False):
        """⑥ 整包测试与不指定范围的跑法没带确认标识（文件头 ②「提交时拒」⑥）：给了确认文件的照 lib.whole_package_confirmation_problem 判形式，不合交 {problem, next_step}；
        没给而命令是整包测试（cargo_runs 认的 cargo test 照 lib.whole_package_cargo_test，命令位置上的入口照 lib.whole_package_script）、
        或是不带选条参数的门禁与入口（lib.unscoped_run_entry），又没带全量门禁标识（full_gate，full_gate_refusal 先核过）的，交 {problem, next_step}；别的交 None。
        弄坏开关 whole-package-unconfirmed-accepted 下没给标识的不判。
        不指定范围的提交时门禁（lib.commit_gate_entry）没带全量门禁标识的先拒，整包确认标识顶替不了它（用户定：全量门禁都要带 --full-gate）；
        弄坏开关 commit-gate-without-full-gate-accepted 下不判。"""
        if (not full_gate and lib.commit_gate_entry(list(command), cwd)
                and not lib.broken("commit-gate-without-full-gate-accepted", self.environment)):
            return {"problem": f"不指定范围的提交时门禁 {shlex.join(command)[:200]} 没带 --full-gate：全量门禁都带它交（整包确认标识顶替不了），这一条不收、不跑",
                    "next_step": "照原命令加上全量标识重交：task-server/task submit --full-gate -- <命令>；只想跑一部分的，改成指定范围（--check <格>、--item <项>）"}
        if confirmation_path:
            problem = lib.whole_package_confirmation_problem(confirmation_path, self.containing_shared_root)
            return {"problem": f"{problem}：这一条不收、不跑", "next_step": lib.WHOLE_PACKAGE_NEXT_STEP} if problem else None
        if lib.broken("whole-package-unconfirmed-accepted", self.environment) or full_gate:
            return None
        reasons = [lib.whole_package_script(list(command), cwd, self.environment)]
        reasons += [lib.whole_package_cargo_test(run.rest) for run in self.cargo_runs({"command": list(command), "cwd": cwd, "environment": environment})
                    if run.subcommand == "test"]
        found = next((reason for reason in reasons if reason), "")
        if found:
            return {"problem": f"整包测试没带确认标识：{found}；日常开发的测试只点名改到的测试目标与测试函数，这一条不收、不跑", "next_step": lib.WHOLE_PACKAGE_NEXT_STEP}
        unscoped = lib.unscoped_run_entry(list(command), cwd, self.environment)
        if unscoped:
            return {"problem": f"不指定范围的跑法没带确认标识：{unscoped}；不指定范围的只在提交时跑，这一条不收、不跑", "next_step": lib.UNSCOPED_NEXT_STEP}
        return None

    def unnamed_run_refusal(self, command, cwd, environment, confirmation_path, full_gate=False):
        """⑦ 启动测试与验证没点名（文件头 ②「提交时拒」⑦）：提交的命令与 bash -c 里命令位置上的每一条（lib.shell_inner_commands）照 lib.unnamed_run_problem 判，
        头一处没点名的交 {problem, next_step}，都点了名交 None。带 --full-gate、带了 --whole-package-confirmed（⑥ 先核过形式）的照那个函数放行。
        自证的小世界里 lib.UNNAMED_RUN_SELFTEST_VARIABLE 写 off 的不判；弄坏开关 unnamed-run-refusal-skipped 下不判。"""
        if lib.broken("unnamed-run-refusal-skipped", self.environment) or lib.selftest_override(lib.UNNAMED_RUN_SELFTEST_VARIABLE, self.environment) == "off":
            return None
        for one in [list(command)] + lib.shell_inner_commands(list(command), self.environment):
            found = lib.unnamed_run_problem(one, cwd, full_gate, bool(confirmation_path), self.environment)
            if found:
                return {"problem": f"{found['problem']}：这一条不收、不跑", "next_step": found["next_step"]}
        return None

    def card_declaration_refusal(self, command, cwd, environment):
        """⑧ 入口文件头写了卡的声明（文件头 ②「提交时拒」⑧；异常表 card-declaration）：提交的命令与 bash -c 里命令位置上的每一条（lib.shell_inner_commands）
        起的脚本照 lib.card_declaration_problem 判，头一处写了的交 {problem, next_step}，都没写交 None。卡归 GPU 批处理验证服务，服务不认这几种声明、不分卡、不清场，
        带着它们的入口当场拒，不照没写那样收（.claude/agent-common.md「里程碑 17 收尾之前不兼容任何旧东西」）。弄坏开关在 lib.card_declaration_problem。"""
        for one in [list(command)] + lib.shell_inner_commands(list(command), self.environment):
            found = lib.card_declaration_problem(one, cwd, self.environment)
            if found:
                return found
        return None

    def submit_refusal(self, command, cwd, environment, confirmation_path="", full_gate=False):
        """提交时拒的全部（文件头 ②「提交时拒」）：交任务那一刻（POST /tasks）与服务起来时重核在排的（revalidate_queued）都照这一份判，加一种提交时拒只改这里。
        environment 是交上来的环境（交任务那一刻是请求里的，重核时是任务带的那一份）。依次判：环境变量名单（split_environment）、套了包装或派发器（nested_problem）、
        编译目录指到仓里（target_directory_refusal）、①–⑤（placement_refusal）、⑥ 整包测试与不指定范围的跑法（whole_package_refusal）、
        ⑦ 启动测试与验证没点名（unnamed_run_refusal）、⑧ 入口文件头写了卡的声明（card_declaration_refusal），命中头一种就停。
        → (带进任务的环境, None 或 (回话 detail 的键, 那一项的值, 给人看的一句))。"""
        carried, refused = lib.split_environment(environment, self.environment)
        if refused:
            return carried, ("refused", [list(pair) for pair in refused], "环境变量 " + "、".join(name for name, _ in refused) + " 现在不收")
        nested = self.nested_problem(command, cwd)
        if nested:
            return carried, ("nested", nested, str(nested))
        target = lib.target_directory_refusal(carried, command, cwd, self.environment)
        if target:
            return carried, ("target", target, str(target))
        for refusal in (self.placement_refusal(command, cwd, carried), self.full_gate_refusal(command, cwd, full_gate),
                        self.whole_package_refusal(command, cwd, carried, confirmation_path, full_gate),
                        self.unnamed_run_refusal(command, cwd, carried, confirmation_path, full_gate),
                        self.card_declaration_refusal(command, cwd, carried)):
            if refusal:
                return carried, ("placement", refusal, f"{refusal['problem']}（出路：{refusal['next_step']}）")
        return carried, None

    def whole_workspace_cargo_refusal(self, command, cwd, environment):
        """⑤ 不点名包的全仓编译（文件头 ②「提交时拒」⑤）：命令位置上直接起的 WHOLE_WORKSPACE_SUBCOMMANDS 那几种 cargo（cargo_runs 认的，入口脚本里面的不读）
        带 --workspace / --all 的，或没带 -p / --package 而照 lib.cargo_manifest_scope 落到虚拟清单、起的目录认不出、清单读不了的，交 {problem, next_step}；
        都点了包、落到包自己的清单、往上没有清单的交 None。"""
        for run in self.cargo_runs({"command": list(command), "cwd": cwd, "environment": environment}):
            if run.subcommand not in WHOLE_WORKSPACE_SUBCOMMANDS:
                continue
            values = lib.cargo_option_values(run.rest)
            named = f"cargo {run.subcommand}"
            flags = [flag for flag in WHOLE_WORKSPACE_FLAGS if values.get(flag)]
            if flags:
                problem = f"{named} 带了 {'、'.join(flags)}：编整个工作区的每一个包"
            elif values["-p"]:
                continue
            elif run.directory is None:
                problem = f"{named} 没点名包（-p / --package），起的目录认不出，判不了它编不编整个工作区"
            else:
                manifest, kind, why = lib.cargo_manifest_scope(run.directory)
                if kind in (lib.MANIFEST_PACKAGE, lib.MANIFEST_NONE):
                    continue
                problem = (f"{named} 没点名包（-p / --package），在 {run.directory} 起，清单 {manifest} 读不了（{why}），判不了它编不编整个工作区"
                           if kind == lib.MANIFEST_UNREADABLE else
                           f"{named} 没点名包（-p / --package），在 {run.directory} 起、照工作区清单 {manifest} 编它的每一个成员")
            return {"problem": f"{problem}：不点名包的全仓编译一条就占一整台，这一条不收、不跑",
                    "next_step": WHOLE_WORKSPACE_NEXT_STEP}
        return None

    def temporary_service_heavy_refusal(self, command, cwd, environment):
        """④ 临时 prune-server 包装里会编的（文件头 ②「提交时拒」④）：剥不开（不是这层包装）的交 None；剥出的原命令照 compiles 有会编的 cargo 的，
        交 {problem, next_step}（点名原命令），不是交 None。剥开只用来判，收下的照旧整条执行。"""
        words = lib.unwrapped_temporary_service(command, self.environment)
        if words == list(command):
            return None
        if not self.compiles({"command": words, "cwd": cwd, "environment": environment}):
            return None
        return {"problem": (f"临时 prune-server 的包装 {lib.TEMPORARY_SERVICE_SCRIPT} {lib.TEMPORARY_SERVICE_FLAG} 里套的原命令有会编的 cargo："
                            "这层包装只给脚本自证用，服务只照外层那一条给编译目录，包装里的认不进来；这一条不收、不跑"),
                "next_step": (f"去掉包装重交：task-server/task submit -- {shlex.join(words)}"
                              "（会编的命令对真服务判，验证包的 cargo test 不套这层包装）")}

    def find_outside_shared_path(self, command, cwd):
        """① 共享根外的路径（文件头 ②「提交时拒」）：只认整个 shell 词是绝对路径的。命令的每个词：以 / 起头的整个词、「名字=绝对路径」「--选项=绝对路径」的值；
        shell 的 -c 脚本那一个词（shell_script_word_indexes）照 shell 词切开（shell_word_paths），每个以 / 起头的 shell 词与「名字=」的值；
        别的词中间夹着的 /（参数值里的「（接的是 /）」这类）不算。此刻在的、以 / 起头的整个词照整个词判；两台相同的系统路径（HOST_INDEPENDENT_PATHS）不判。
        相对路径照工作目录解，工作目录在共享根下它们也在。→ 头一个不在的 {problem, next_step}，都在交 None；落在 VOLATILE_ROOT 下的另说一句重启会清。
        紧跟在 ${…}、$(…) 后面的 /… 是展开之后的尾巴（编译目录这类运行时才给的），不当绝对路径；弄坏开关 expansion-tail-as-path 下照改前也当。
        弄坏开关 quoted-word-paths-ignored 下不认词里的路径（改前的样子）；path-check-substring 下每个带空白的词都按子串（QUOTED_WORD_PATH）认，词中间夹着的 / 也当路径。"""
        found = ""
        real_cwd = os.path.realpath(cwd)
        if not self.containing_shared_root(real_cwd):
            found, named = cwd, f"工作目录 {real_cwd}"
        else:
            scripts = shell_script_word_indexes(command)
            for index, word in enumerate(command):
                if word.startswith("/") and os.path.lexists(word):
                    candidates = [word]
                elif lib.broken("path-check-substring", self.environment) and any(character.isspace() for character in word):
                    candidates = QUOTED_WORD_PATH.findall(word)
                elif index in scripts:
                    candidates = ([] if lib.broken("quoted-word-paths-ignored", self.environment)
                                  else shell_word_paths(word, SHELL_WORD_PATH_WITH_EXPANSION_TAILS) if lib.broken("expansion-tail-as-path", self.environment)
                                  else shell_word_paths(word))
                else:
                    value = word if word.startswith("/") else word.partition("=")[2]
                    candidates = [value] if value.startswith("/") else []
                found = next((path for path in candidates
                              if not any(path == entry or (entry.endswith("/") and path.startswith(entry)) for entry in HOST_INDEPENDENT_PATHS)
                              and not self.containing_shared_root(os.path.realpath(path))), "")
                if found:
                    named = f"命令里点到的 {found}"
                    break
        if not found:
            return None
        roots = "、".join(self.shared_roots) or "一个根都没写"
        volatile = any(os.path.normpath(path) == VOLATILE_ROOT or os.path.normpath(path).startswith(VOLATILE_ROOT + os.sep) for path in (found, os.path.realpath(found)))
        lost = (f"；它落在 {VOLATILE_ROOT} 下：重启会清掉 {VOLATILE_ROOT}，任务做完了、重启之后那里的产物就没了" if volatile else "")
        return {"problem": (f"{named} 不在多机配置 {lib.NFS_ROOTS_KEY} 的任何一段下（{roots}）：任务两台都要跑得起，第二台照同一个绝对路径读写，"
                            f"读不到本机那一份、写出的本机也读不到{lost}；这一条不收、不跑"),
                "next_step": f"把它改放 {lib.NFS_ROOTS_KEY} 某一段下（{roots}；要留的产物放共享工作根 {self.work_root} 下），再提交"}

    def verdict_library_refusal(self, command, cwd):
        """③ 判定库（文件头 ②「提交时拒」）：带不带判定库只照登记表 .claude/gate.d/verdict-library-scripts.tsv 判（registry_families），不另问别的脚本：
        命令里（临时 prune-server 的包装剥开、照原命令认）有一个词点到登记表里带判定库的一行（第三列不以「不算：」「无判定库：」起头），而那一行第三列不以 prune-server 起头的，
        交 {problem, next_step}，点名那一族（登记的路径）；登记表读不了、写坏了的同样拒（判不了）。都不是交 None。
        词的认法（registry_family_of）：整个词，词里有空白的（bash -c 那一段）按空白拆开的每一段；照写的、照工作目录解开的、解开符号链接的，等于登记的路径或以「/登记的路径」收尾。
        弄坏开关 registry-service-column-ignored 下第三列写 prune-server 的也拒。"""
        families, problem = self.registry_families()
        if problem:
            return {"problem": f"判不了这条命令带不带判定库：{problem}；这一条不收、不跑",
                    "next_step": f"把登记表 {VERDICT_LIBRARY_REGISTRY} 修好（列的写法在它文件头），再提交"}
        words = lib.unwrapped_temporary_service(command, self.environment)
        found = self.registry_family_of(words, cwd, families)
        if not found:
            return None
        if families[found].startswith(REGISTRY_SERVICE_MARK) and not lib.broken("registry-service-column-ignored", self.environment):
            return None
        return {"problem": (f"这条命令带判定库，那一族是登记表 {VERDICT_LIBRARY_REGISTRY} 的 {found}，第三列还没写 {REGISTRY_SERVICE_MARK}："
                            "判定写在按台的库里，派到哪台就落在哪台，两台读不到同一份；这一条不收、不跑"),
                "next_step": (f"把 {found} 那一族的判定库读写接进 prune-server（.claude/rules/verification.md「判定库只有一份」），"
                              f"再把登记表那一行第三列翻成 {REGISTRY_SERVICE_MARK}，然后提交")}

    def registry_family_of(self, words, cwd, families):
        """words 里头一个点到登记表带判定库那一行的词：交登记的路径，没有交空串（认法见 verdict_library_refusal）。展开不了的词（带 $ 或反引号）不认。"""
        root = os.path.realpath(self.peer_root)
        library = [path for path, where in families.items() if not where.startswith(REGISTRY_NO_LIBRARY_PREFIXES)]
        for word in words:
            for piece in (word.split() if any(character.isspace() for character in word) else [word]):
                piece = piece.strip("'\"")
                if not piece or piece.startswith("-") or "$" in piece or "`" in piece:
                    continue
                piece = piece.partition("=")[2] if "=" in piece and not piece.startswith(("/", ".")) else piece
                if "/" not in piece:
                    continue
                resolved = os.path.normpath(os.path.join(cwd, os.path.expanduser(piece)))
                candidates = {os.path.normpath(piece), resolved, os.path.realpath(resolved)}
                for path in library:
                    registered = os.path.join(root, path)
                    if any(candidate in (path, registered, os.path.realpath(registered)) or candidate.endswith("/" + path) for candidate in candidates):
                        return path
        return ""

    def registry_families(self):
        """登记表 .claude/gate.d/verdict-library-scripts.tsv（相对仓根）：→ ({第一列的路径: 第三列（判定库放哪）}, 读不了或写坏了的那一句；读得了是空串)。
        第三列以「prune-server」起头的那一族判定库读写全在服务里、两台读的是同一份，提交时照收（判定库服务化的会话定的规矩）；以「不算：」「无判定库：」起头的没有判定库。
        表的修改时刻没变就用上一次读的。"""
        path = os.path.join(os.path.realpath(self.peer_root), VERDICT_LIBRARY_REGISTRY)
        try:
            stamp = os.stat(path).st_mtime_ns
        except OSError as error:
            return {}, f"登记表 {path} 读不了（{error}）"
        cached = getattr(self, "registry_cache", None)
        if cached and cached[0] == stamp:
            return cached[1], ""
        families = {}
        try:
            with open(path, encoding="utf-8") as handle:
                for number, line in enumerate(handle, 1):
                    if not line.strip() or line.startswith("#"):
                        continue
                    columns = line.rstrip("\n").split("\t")
                    if len(columns) < 3:
                        return {}, f"登记表 {path} 第 {number} 行不到三列"
                    families[columns[0].strip()] = columns[2].strip()
        except (OSError, UnicodeDecodeError) as error:
            return {}, f"登记表 {path} 读不了（{error}）"
        self.registry_cache = (stamp, families)
        return families, ""

    def containing_shared_root(self, real):
        """real（真路径）落在哪个 NFS 共享根下：交那个根，不在交空串（find_outside_shared_path 与 shared_roots_problem 共用）。"""
        return next((root for root in self.shared_roots if real == root or real.startswith(root.rstrip(os.sep) + os.sep)), "")

    def attempt_shared_roots(self, task):
        """这一次交给命令的两个共享根（task_server_lib.py 文件头「资源记录」后的「共享根」）：{变量名: 值}；任务自己带了的（白名单里的）照任务带的。"""
        values = {lib.DATA_ROOT_VARIABLE: self.data_root, lib.WORK_ROOT_VARIABLE: self.work_root}
        for name in lib.SHARED_ROOT_VARIABLES:
            if task["environment"].get(name):
                values[name] = task["environment"][name]
        return values

    def shared_roots_problem(self, environment):
        """② 共享根（文件头 ②「提交时拒」）：这一次要交给命令的两个共享根（attempt_shared_roots：服务自己的，任务带了的照任务带的）有不在 NFS_SHARED_ROOTS 某一段下的
        交 {problem, next_step}（点名变量与值），都在交 None。相对路径算不在（第二台照它自己的目录解）。"""
        outside = [f"{name}={value}" for name, value in self.attempt_shared_roots({"environment": environment}).items()
                   if not os.path.isabs(value) or not self.containing_shared_root(os.path.realpath(value))]
        if not outside:
            return None
        roots = "、".join(self.shared_roots) or "一个根都没写"
        return {"problem": (f"{'、'.join(outside)} 不在多机配置 {lib.NFS_ROOTS_KEY} 的任何一段下（{roots}）：命令在第二台经它写的，两台读不到同一份；"
                            "这一条不收、不跑"),
                "next_step": (f"配多机配置 {self.configuration.path}：{lib.DATA_ROOT_KEY} 写在 {lib.NFS_ROOTS_KEY} 某一段下、服务环境里的 {lib.WORK_ROOT_VARIABLE} "
                              f"也放在那下面（或往 {lib.NFS_ROOTS_KEY} 加上它所在的根），改好重起服务；是任务自己带的那个变量，就去掉它或改成共享根下的路径，再提交")}

    def candidate_hosts(self, task, snapshot):
        """这一件能派的几台，按先后排（task_server_lib.py 文件头「挑机器」）：重排钉了台的（retry_decision 换到总上限更大的那一台）只那一台；
        第二台派不过去的只本机；分片的各片第二台在前；别的本机在前。
        弄坏开关 host-lists-restored 下照改前点到只在第二台跑的入口的只第二台。"""
        pinned = task.get("next_host") or ""
        if pinned:
            return [pinned]
        if self.peer_only_entry(task):
            return [PEER]
        order = list(MACHINE_ORDER)
        if task.get("role") == "shard" and not lib.broken("shard-not-sent-to-peer", self.environment):
            order.reverse()
        return [host for host in order if self.host_eligible(task, host, snapshot)]

    def peer_only_entry(self, task):
        """弄坏开关 host-lists-restored 下：命令点到删之前只在第二台跑的入口（RESTORED_HOST_LISTS 的 peer_entries）交 (入口, 理由)，没点到交 None；
        开关没开一律 None（没有只派第二台的入口）。词照相对仓根的路径比（与改本机工作区的工具同一个比法）。"""
        if not lib.broken("host-lists-restored", self.environment):
            return None
        root = os.path.realpath(self.peer_root)
        directory = task.get("cwd") or root
        for word in task.get("command") or []:
            if word.startswith("-"):
                continue
            relative = os.path.relpath(os.path.normpath(os.path.join(directory, word)), root)
            if relative in RESTORED_HOST_LISTS["peer_entries"]:
                return relative, "弄坏开关 host-lists-restored：照改前只在第二台跑"
        return None

    def host_margin(self, host, snapshot):
        """这台此刻的余量（文件头 ②「余量」）：→ (字节, 量不到的原因)。取这台自己报告里那一项（host-dispatch.py 的 report_margin：memory_peaks_lib 现量、按 1 GiB 向上取整），
        两台各用各的，不跨轮存；这台没报告（探测失败）或报告里量不到的交 None 与原因，不退回固定的数。
        弄坏开关 margin-local-for-both 下第二台也取本机报告里那一项；margin-fallback-fixed 下量不到的退回 BREAK_FIXED_MARGIN_BYTES。"""
        name = lib.HOST_NAMES.get(host, host)
        source = LOCAL if lib.broken("margin-local-for-both", self.environment) else host   # 弄坏开关：两台都取本机报告里的那一项
        report = snapshot.reports.get(source)
        if report is None:
            return None, f"{name}此刻查不到余量（探测没回报告{'：' + snapshot.down[host] if snapshot.down.get(host) else ''}）"
        margin, problem = self.host_dispatch.report_margin(report["memory"])
        if margin is None and lib.broken("margin-fallback-fixed", self.environment):
            return BREAK_FIXED_MARGIN_BYTES, ""
        if margin is None:
            return None, f"{name}此刻量不到余量（{problem}）"
        return margin, ""

    def host_capacity(self, host, snapshot):
        """这台单条能给到的最大上限（文件头 ②「余量」）：→ (这台总上限 − 这台此刻的余量, 这台这一轮不派的原因)。余量量不到、或总上限不比余量大的交 None 与原因。
        弄坏开关 booked-up-to-slice-total 下不减余量（照改前收到总上限）。"""
        name = lib.HOST_NAMES.get(host, host)
        margin, why = self.host_margin(host, snapshot)
        if margin is None:
            return None, why
        total = self.host_total(host, snapshot)
        if lib.broken("booked-up-to-slice-total", self.environment):
            return total, ""
        if total <= margin:
            human = self.host_dispatch.human
            return None, f"{name}的总上限 {human(total)} 不比此刻的余量 {human(margin)} 大，一条都给不出上限"
        return total - margin, ""

    def pinned_on_host(self, prediction, host, snapshot):
        """钉了的命令在这台上的钉值（字节，文件头 ②「内存」的钉值）：多机配置 SLICE_MEMORY_CAP_BY_HOST 里这台那一项（prediction.memory_by_host，配多少就是多少）
        减这台此刻的余量（host_margin）。调用方先经 host_capacity 判过这台量得到余量。弄坏开关 pinned-cap-this-host-only 下两台都取服务这一台那一项；
        pinned-cap-at-slice-total 下不减余量（照那一项记满）。"""
        source = LOCAL if lib.broken("pinned-cap-this-host-only", self.environment) or host not in prediction.memory_by_host else host
        value = int(prediction.memory_by_host[source])
        if lib.broken("pinned-cap-at-slice-total", self.environment):
            return value
        return value - self.host_margin(host, snapshot)[0]

    def host_limit(self, prediction, host, snapshot):
        """这一条在这台上能给到的最大上限：→ (字节, 这台这一轮不派的原因)（文件头 ②「余量」）。这台没报告、余量量不到、总上限不比余量大，或钉了的命令在这台的钉值
        不大于 0（那一项不比这台此刻的余量大），交 None 与原因；这台这一轮不派它，不退回别的数、不兜底。弄坏开关 pinned-nonpositive-accepted 下钉值不大于 0 的也照派。"""
        name = lib.HOST_NAMES.get(host, host)
        if host not in snapshot.reports:
            return None, f"{name}此刻查不到余量（探测没回报告{'：' + snapshot.down[host] if snapshot.down.get(host) else ''}）"
        capacity, why = self.host_capacity(host, snapshot)
        if capacity is None:
            return None, why
        if prediction is not None and prediction.memory_history == lib.PINNED_HISTORY and prediction.memory_by_host:
            value = self.pinned_on_host(prediction, host, snapshot)
            if value <= 0 and not lib.broken("pinned-nonpositive-accepted", self.environment):
                human = self.host_dispatch.human
                return None, (f"{name}：钉值取多机配置的 {lib.PIN_SOURCE} 里这台那一项，减这台此刻的余量 {human(self.host_margin(host, snapshot)[0])} 之后不大于 0，"
                              f"这台一条都给不出钉值；出路：把多机配置 {self.configuration.path} 的 {lib.PIN_SOURCE} 里这台那一项改大（写的是那台 slice 的总上限），"
                              "或 python3 task-server/control unpin-cap <命令…> 撤掉钉值")
        return capacity, ""

    def usable_maxima(self, snapshot):
        """每台单条能给到的最大上限（retry_decision 乘 1.5 与「上限到了总上限」照它比）：这台总上限减这台此刻的余量（host_capacity）；量不到的是 None（查不到）。"""
        result = {}
        for host, maximum in snapshot.maxima().items():
            result[host] = None if maximum is None else self.host_capacity(host, snapshot)[0]
        return result

    def budget_check(self, task, view):
        """预算（文件头 ②「预算」）：这一条在某一台上的记账（booked_at_cap：收到那台可用上限为止）不小于那台整片界（whole_threshold）的，
        把一项 oversized-budget 记进 budget_notices（同一条只记一次）；预测峰值比那台大、别处可能装得下而不派那台的（hosts_short_of_record）不算那一台。
        记账收到可用上限为止，没有超过可用上限的记账，不打回。判不了（量不到余量、预测读不了）的不记，下一轮再判。弄坏开关 oversized-budget-silent 下不记。"""
        human = self.host_dispatch.human
        try:
            prediction = self.prediction_of(task)
            snapshot = view.snapshot()
            judged = []   # [(台, 这台上的记账, 这台可用上限, 这台整片界)]
            candidates = list(self.candidate_hosts(task, snapshot))
            short = self.hosts_short_of_record(prediction, candidates, snapshot)
            for host in candidates:
                if host not in snapshot.reports or host in short:
                    continue
                limit = self.host_limit(prediction, host, snapshot)[0]
                if limit is None:
                    continue
                booked = int(self.booked_at_cap(prediction, host, snapshot).memory_booked or 0)
                judged.append((host, booked, limit, self.whole_threshold(host, snapshot)))
        except (OSError, ValueError, KeyError, subprocess.SubprocessError):
            return
        if not judged:
            return
        names = lib.HOST_NAMES
        # 记账从哪来那一句（budget_source 要按命令键扫库）只在记 oversized-budget 那一支里算（loop 审核发现 1：每条在排的每一轮为它扫一次库）；
        # 弄坏开关 budget-source-every-pass 下每次先算
        source = self.budget_source(task, prediction) if lib.broken("budget-source-every-pass", self.environment) else None
        whole_on = [host for host, booked, limit, threshold in judged if threshold <= booked <= limit]   # 不超过可用上限而不小于整片界：在那台上能起、起了占满一整台
        if whole_on and task["id"] not in self.budget_notices and not lib.broken("oversized-budget-silent", self.environment):
            bookings = "、".join(f"{names.get(host, host)} {human(booked)}" for host, booked, _, _ in judged)
            maxima = "、".join(f"{names.get(host, host)} {human(limit)}" for host, _, limit, _ in judged)
            thresholds = "、".join(f"{names.get(host, host)} {human(threshold)}" for host, _, _, threshold in judged)
            source = source or self.budget_source(task, prediction)
            self.budget_notices[task["id"]] = self.attention_item(
                "oversized-budget", 1, task["id"], f"记账 {bookings}；可用上限 {maxima}；贴着整台的界（可用上限的 {WHOLE_FRACTION:.0%}）{thresholds}；记账从哪来：{source}",
                (f"记账不小于{'、'.join(names.get(host, host) for host in whole_on)}贴着整台的界：它起来就差不多占满那一整台，别的任务在那台都得等它；"
                 "一条命令要一整台的内存，多半是范围点多了、没分片或同时起的个数照上限开"))

    def budget_source(self, task, prediction):
        """记账从哪来（budget_check 的依据）：照预测的 memory_history 说一句，照记录的另写近几次里用了几次。"""
        human = self.host_dispatch.human
        predicted = human(int(prediction.memory_predicted or 0))
        history = prediction.memory_history
        if history == lib.PINNED_HISTORY:
            return f"钉值：多机配置 {lib.PIN_SOURCE} 里那一台那一项减余量（control pin-cap 钉的）"
        if history == lib.RECORD_HISTORY:
            measurements = self.recorded_measurements(task)
            return f"这把命令键记下的峰值（近 {min(len(measurements), RECORD_WINDOW)} 次里最大 {predicted}）{self.folded_key_note(task)}"
        texts = {lib.UNRECORDED_HISTORY: "没有记录的缺省",
                 "retry": "撞了上限之后乘 1.5 重排的那一档"}
        return texts.get(history, f"来历 {history}")

    def folded_key_note(self, task):
        """记录来源那一句的尾巴：命令键原文；键里有折掉的点名项、文件参数（memory_peaks_lib 的 FOLD_TOKENS 那几个记号）时写明同一入口各项共用这一份记录、怎么按项记。
        弄坏开关 folded-key-unnamed 下不写（照改前只写「这把命令键」）。"""
        key = str(task.get("command_key") or "")
        if not key or lib.broken("folded-key-unnamed", self.environment):
            return ""
        folded = [token for token in self.host_dispatch.peaks.FOLD_TOKENS if token in key]
        if not folded:
            return f"；命令键「{key}」"
        return (f"；命令键「{key}」里的 {'、'.join(folded)} 是折掉的点名项或文件参数：同一入口各项共用这一份记录，"
                f"要按项记就提交时带 {lib.KEY_VARIABLE}=<这一项的键>，或在 scripts/hosts/memory_peaks_lib.py 的 ITEM_KEPT_CELLS 登记那一格")

    def booked_at_cap(self, prediction, host, snapshot):
        """准入按上限记账（文件头 ②「内存」的准入记账；用户：「谁超了杀谁 不要连累别人」「检查内存分配 不得大于配置中的设定」）：这台上记的账取上限，不取峰值；
        预测峰值、上限与记账都收到这台的可用上限（host_limit：这台总上限减这台此刻的余量）为止，钉值、记录、缺省、撞上限重排四种来历一样收，记录比整台大的也不判永远放不下；
        比这台大、别处装得下的不派这台由 judge_placement 照 hosts_short_of_record 判。只对 host_limit 交了数的台调，交不出数的抛 ValueError（不退回别的数）。
        弄坏开关 booked-at-peak 下记峰值、不按上限记账；booking-over-host-limit 下钉值之外预测峰值比可用上限大的照原样记（judge_host 判那一台永远放不下）。"""
        if lib.broken("booked-at-peak", self.environment):
            return prediction
        limit, why = self.host_limit(prediction, host, snapshot)
        if limit is None:
            raise ValueError(f"这台这一轮不派：{why}")
        if prediction.memory_history == lib.PINNED_HISTORY and prediction.memory_by_host:
            # 钉值按台（文件头 ②「内存」的钉值）：判哪一台就取多机配置 SLICE_MEMORY_CAP_BY_HOST 里那一台那一项、减那台此刻的余量
            value = self.pinned_on_host(prediction, host, snapshot)
            prediction = dataclasses.replace(prediction, memory_predicted=value, memory_cap=value, memory_booked=value)
        # 收到这台总上限减这台此刻的余量为止：超过总上限 scope 管不住它、撞的是整个 slice；slice 里挂着的那一截回收不掉，记满一整台永远放不下
        cap = min(int(prediction.memory_cap or 0), limit)
        booked = max(int(prediction.memory_booked or 0), cap)
        predicted = int(prediction.memory_predicted or 0)
        if predicted > limit and prediction.memory_history != lib.PINNED_HISTORY and lib.broken("booking-over-host-limit", self.environment):
            return dataclasses.replace(prediction, memory_cap=cap, memory_booked=booked)   # 弄坏开关：峰值与记账不收，judge_host 判这台永远放不下
        return dataclasses.replace(prediction, memory_predicted=min(predicted, limit), memory_cap=cap, memory_booked=min(booked, limit))

    def hosts_short_of_record(self, prediction, hosts, snapshot):
        """预测峰值比哪几台的可用上限大、而别处可能装得下（文件头 ②「内存」的准入记账）：→ {台: 这台不派的那一句}。
        别处可能装得下 = hosts 里有可用上限（host_limit）不小于预测峰值的台、有这一轮给不出可用上限的台，或多机配置里有一台此刻连不上（它回来之后可能装得下，place 照它排着）：
        比预测峰值小的那几台照永远放不下判、不派，任务去装得下的那台或排着等。没有可能装得下的别处时交空表：每一台都照收到可用上限的数判（booked_at_cap），
        不因为记录比整台大就判永远放不下，收到之后还撞上限的照 retry_decision 打回 memory-cap-at-maximum。钉值不在这里判（钉值就是判的那一台那一项）。
        弄坏开关 record-over-every-host-never-fits 下没有可能装得下的别处时也交那几台（每一台都比预测峰值小就判永远放不下）。"""
        predicted = int(prediction.memory_predicted or 0)
        limits = {host: self.host_limit(prediction, host, snapshot)[0] for host in hosts}
        short = {host: limit for host, limit in limits.items() if limit is not None and predicted > limit}
        if not short or prediction.memory_history == lib.PINNED_HISTORY:
            return {}
        names = lib.HOST_NAMES
        elsewhere = [f"{names.get(host, host)}装得下" for host, limit in limits.items() if limit is not None and predicted <= limit]
        elsewhere += [f"{names.get(host, host)}这一轮给不出可用上限" for host, limit in limits.items() if limit is None]
        elsewhere += [f"{names.get(host, host)}此刻用不了、回来之后可能装得下" for host in MACHINE_ORDER
                      if host not in limits and snapshot.expected.get(host) and not snapshot.reachable.get(host)]
        if not elsewhere and not lib.broken("record-over-every-host-never-fits", self.environment):
            return {}
        human = self.host_dispatch.human
        where = "、".join(elsewhere) if elsewhere else "能派的每一台都比它小"
        return {host: f"内存：这一条的预测峰值 {human(predicted)} 比这台的可用上限 {human(limit)}（总上限减此刻的余量）大，{where}：这台不派"
                for host, limit in short.items()}

    def running_booking(self, attempt, usage):
        """在跑的这一次尝试记多少、取的是哪一个（文件头 ②「内存」的在跑记账）→ (字节, BOOKED_FROM_* 之一)：usage 是这台报告里各 scope 此刻回收不掉的用量（report_scope_usage）。
        报告里有它的 scope 的，只照此刻实测记：min(它的上限, scope 此刻的用量)，不看任何预先估的数、不加固定余量；实测到了上限取 cap，别的取 measured；
        报告里没有它的 scope 的（刚起、读不了、那台这一轮没报告）记整份上限，取 no-scope。
        它自己的上限（memory.max）照旧，超了只杀它。弄坏开关 running-booked-at-cap 下一律记整份上限，
        running-booked-with-margin 下有 scope 的在实测上再加 RUNNING_BOOKING_MARGIN_BREAK_BYTES（不超过上限）。"""
        cap = int(attempt.get("memory_cap") or 0)
        if lib.broken("running-booked-at-cap", self.environment):
            return cap, BOOKED_FROM_CAP
        used = usage.get(attempt.get("label") or "")
        if used is None:
            return cap, BOOKED_WITHOUT_SCOPE
        # 起跑不满 CPU_MEASURED_AFTER_SECONDS（两分钟）的照整份上限记：刚起的还没涨起来，此刻的实测几乎是 0，照它记时同一把里交的几十条会一起放进来
        # （任务 15262 到 15299 共 38 条十秒内全起，本机内存用到 55G、swap 15G）。满两分钟之后照实测。
        started = float(attempt.get("started_at") or 0)
        young = lib.selftest_override(lib.YOUNG_BOOKING_VARIABLE, self.environment)
        if (started and time.time() - started < (float(young) if young else CPU_MEASURED_AFTER_SECONDS)
                and not lib.broken("running-young-booked-by-usage", self.environment)):
            return cap, BOOKED_WHILE_YOUNG
        wanted = int(used) + (RUNNING_BOOKING_MARGIN_BREAK_BYTES if lib.broken("running-booked-with-margin", self.environment) else 0)
        if wanted >= cap:
            return cap, BOOKED_FROM_CAP
        return wanted, BOOKED_FROM_USAGE

    def running_booked(self, attempt, usage):
        """在跑的这一次尝试记多少（字节）：running_booking 交的那个数。"""
        return self.running_booking(attempt, usage)[0]

    def note_running_sample(self, table, attempt_id, now, value, window_seconds, merge):
        """CPU 闸用的读数窗口（note_peak_segment）：往 table（running_peak_samples）里这一条尝试名下记一回 now 时读到的 value，
        RUNNING_PEAK_SAMPLE_SECONDS 之内的几回并成一条（时刻取先到的，值取 merge(先到的, 后到的)：一轮排队里同一条要算好几回记账，窗口里不按回数攒）。
        窗口起点那一端（至少 window_seconds 之前的里最新的一条）也留着、再往前的不留（累计量要两端读数相减），窗口里只留不到 window_seconds 的。→ (服务头一回读到它的时刻, 窗口里的读数表（时刻从早到晚）)。
        别的尝试那一条不在这里删：读数隔得比窗口久（服务一轮慢、算账隔得久）时起点那一端照样要用；收尾的由 finish_attempt 删，不在在跑名单里的由 running_threads 删（prune_running_samples）。"""
        with self.running_usage_lock:
            entry = table.setdefault(attempt_id, {"first_seen": now, "samples": []})
            samples = ([sample for sample in entry["samples"] if now - sample[0] >= window_seconds][-1:]
                       + [sample for sample in entry["samples"] if now - sample[0] < window_seconds])
            if samples and now - samples[-1][0] < RUNNING_PEAK_SAMPLE_SECONDS:
                samples[-1] = (samples[-1][0], merge(samples[-1][1], value))
            else:
                samples.append((now, value))
            entry["samples"] = samples
            return entry["first_seen"], list(samples)

    def prune_running_samples(self, alive):
        """CPU 闸的读数表（running_peak_samples）、各段最大（running_peak_cores）与可跑线程数（running_peak_runnable）里删掉不在 alive（库里在跑的尝试号）里的：
        收尾不经 finish_attempt 的（记录被改、单元没起成）不留在表里。running_threads 每回调。"""
        with self.running_usage_lock:
            for table in (self.running_peak_samples, self.running_peak_cores, self.running_peak_runnable,
                          self.running_latest_cores, self.running_latest_runnable):
                for stale in [number for number in table if number not in alive]:
                    del table[stale]

    @staticmethod
    def total_label():
        """公式里总额那一截的写法。"""
        return "配置的总上限"

    def formula_label(self):
        """公式那一句里的写法（看进展、记账明细）：total_label 接「减余量减在跑的记账之和」。"""
        return f"{self.total_label()}减余量减{RUNNING_BOOKED_SUM_TEXT}"

    def room_by_the_formula(self, host, snapshot):
        """这台还能记多少（公式，只认这一个）：配置的总上限 − 余量 − 在跑的记账之和（running_booked，用量照这一份 snapshot 里这台的报告）。量不到余量交 None。"""
        margin = self.host_margin(host, snapshot)[0]
        if margin is None:
            return None
        usage = self.report_scope_usage(snapshot.reports.get(host))
        booked = sum(self.running_booked(attempt, usage) for attempt in self.booking_attempts() if attempt.get("host") == host)
        return snapshot.total(host) - margin - booked

    def verdict_within_the_formula(self, verdict, on_host, host, snapshot, pending_memory=None):
        """放得下的再过一遍公式（room_by_the_formula）：记账超过它就放不下，原因照 memory_wait_text 写（pending_memory 是交给 judge_host 的待涨量，没给就不写 slice 那一侧减过之后剩多少）。
        弄坏开关 memory-wait-text-unnamed 下照只写要记多少与公式只剩多少的那一句。"""
        room = self.room_by_the_formula(host, snapshot)
        if not verdict.fits or room is None or int(on_host.memory_booked or 0) <= room:
            return verdict
        if lib.broken("memory-wait-text-unnamed", self.environment):
            why = f"内存：要记 {int(on_host.memory_booked or 0) >> 20} MiB，配置的总上限减余量减{RUNNING_BOOKED_SUM_TEXT}只剩 {max(room, 0) >> 20} MiB"
        else:
            why = self.memory_wait_text(on_host, host, snapshot, pending_memory)
        return dataclasses.replace(verdict, fits=False, why=why)

    def memory_wait_text(self, on_host, host, snapshot, pending_memory=None):
        """内存放不下时排队原因里的那一句（文件头 ②「内存」的排队原因）：要记多少（这一条的上限）；两个数各是什么——账面还能记多少（room_by_the_formula，写出总上限、余量与
        在跑的记账之和）、slice 实测还放得下多少（报告的 slice_room：总上限减 slice 里进程用的；给了 pending_memory 的另写减余量与这个待涨量之后剩多少）；给了 pending_memory 而
        整机空闲减它不够要的空闲（记录的峰值加余量、不超过上限，同 judge_host）的另写一句；末尾点名这台记账最多的至多 MEMORY_WAIT_TOP_RUNNING 条在跑任务、各自的记账与实占。"""
        human = self.host_dispatch.human
        booked, predicted, cap = int(on_host.memory_booked or 0), int(on_host.memory_predicted or 0), int(on_host.memory_cap or 0)
        margin = self.host_margin(host, snapshot)[0]
        report = snapshot.reports.get(host)
        memory = (report or {}).get("memory") or {}
        usage = self.report_scope_usage(report)
        rows = sorted(((self.running_booked(attempt, usage), attempt) for attempt in self.booking_attempts() if attempt.get("host") == host),
                      key=lambda row: (-row[0], row[1].get("task_id") or 0))
        if margin is None:
            account = "账面还能记多少算不出（这台量不到余量）"
        else:
            in_use = sum(amount for amount, _ in rows)
            account = (f"账面还能记 {human(max(snapshot.total(host) - margin - in_use, 0))}（总上限 {human(snapshot.total(host))}"
                       f" − 余量 {human(margin)} − {RUNNING_BOOKED_SUM_TEXT} {human(in_use)}）")
        slice_room = memory.get("slice_room")
        if isinstance(slice_room, int) and not isinstance(slice_room, bool):
            measured = f"slice 实测还放得下 {human(max(slice_room, 0))}（总上限减 slice 里进程用的 {human(int(memory.get('slice_used') or 0))}）"
            if pending_memory is not None and margin is not None:
                measured += f"，减余量与在跑的还没涨到的 {human(int(pending_memory))} 之后剩 {human(max(slice_room - margin - int(pending_memory), 0))}"
        else:
            measured = "slice 实测还放得下多少这一轮没读到"
        machine = ""
        free = memory.get("free")
        if pending_memory is not None and margin is not None and isinstance(free, int) and not isinstance(free, bool):
            needed = min(predicted + margin, cap) if cap else predicted + margin
            if free - int(pending_memory) < needed:
                machine = f"；整机空闲 {human(max(free, 0))} 减在跑的还没涨到的之后剩 {human(max(free - int(pending_memory), 0))}，不够要的空闲 {human(needed)}"
        named = []
        for amount, attempt in rows[:MEMORY_WAIT_TOP_RUNNING]:
            used = usage.get(attempt.get("label") or "")
            named.append(f"任务 {attempt.get('task_id')} 记 {human(amount)}、实占 {'读不到' if used is None else human(used)}")
        heaviest = f"；这台记账最多的：{'；'.join(named)}" if named else "；服务在这台没有在跑的任务"
        return f"内存：要记 {human(booked)}（这一条的上限），放不下——{account}；{measured}{machine}{heaviest}"

    def memory_verdict_reworded(self, verdict, on_host, host, snapshot, pending_memory):
        """judge_host 判内存放不下的那一句（以 JUDGE_HOST_MEMORY_PREFIX 起头，「此刻空闲」是整机空闲减待涨量、不是机器的可用内存）换成 memory_wait_text；同一判决里别的几句照留。
        judge_host 把内存那一句放在最前、几句之间用「；」接（那一句里没有「；」）。放得下的、永远放不下的照原样。弄坏开关 memory-wait-text-unnamed 下照原样。"""
        if verdict.fits or verdict.never or lib.broken("memory-wait-text-unnamed", self.environment) or not verdict.why.startswith(JUDGE_HOST_MEMORY_PREFIX):
            return verdict
        _, separator, rest = verdict.why.partition("；")
        return dataclasses.replace(verdict, why=self.memory_wait_text(on_host, host, snapshot, pending_memory) + separator + rest)

    def slice_floor_problem(self, host, snapshot):
        """slice 兜底（文件头 ②「内存」的 slice 兜底）：这台 slice 此刻实测还放得下的（报告里的 slice_room：总上限 − slice 里进程用的内存）要不小于 slice_room_floor，
        不到的交排队原因（以 SLICE_FLOOR_TEXT 起头、写明是实测余量不够）；放得下、或报告里没有这一项交空串。
        slice_room_floor 是 0（自证的小世界）时不判。弄坏开关 slice-floor-ignored 下不判。"""
        floor = self.slice_room_floor
        if floor <= 0 or lib.broken("slice-floor-ignored", self.environment):
            return ""
        memory = (snapshot.reports.get(host) or {}).get("memory") or {}
        room = memory.get("slice_room")
        if not isinstance(room, int) or isinstance(room, bool) or room >= floor:
            return ""
        human = self.host_dispatch.human
        return (f"{SLICE_FLOOR_TEXT}：实测余量不够——这台 singlefs-heavy.slice 此刻实测只剩 {human(max(room, 0))}（总上限 {human(int(memory.get('slice_total') or 0))} − "
                f"slice 里进程用的 {human(int(memory.get('slice_used') or 0))}），放不下下限 {human(floor)}：这一轮不派新任务，等在跑的降下来或跑完")

    def machine_memory_problem(self, host, snapshot):
        """整机内存线（文件头 ②「内存」的整机内存线）：这台整机可用内存占总量的比例（报告里 mem_available ÷ mem_total）与 swap 空闲占总量的比例（swap_free ÷ swap_total）都低于
        MACHINE_MEMORY_LINE_FRACTION 时交排队原因（这一轮不往这台派新任务，写两个比例与两个字节数），不然交空串。earlyoom 在这两样都低于 10% 时才杀进程，这一道在它之前停派；
        只挡新派，不停在跑的。报告里没有这几项（没有 swap 的机器 swap_total 是 0、读不到是 None）的，没有 swap 的按 swap 空闲 0% 算（可用内存低于线就停派），读不到的不判。
        弄坏开关 machine-memory-line-ignored 下不判。"""
        if lib.broken("machine-memory-line-ignored", self.environment):
            return ""
        memory = (snapshot.reports.get(host) or {}).get("memory") or {}
        total, available = memory.get("mem_total"), memory.get("mem_available")
        swap_total, swap_free = memory.get(self.peaks.SWAP_TOTAL_KEY), memory.get(self.peaks.SWAP_FREE_KEY)
        numbers = (total, available, swap_total, swap_free)
        if any(not isinstance(number, int) or isinstance(number, bool) or number < 0 for number in numbers) or total <= 0:
            return ""
        memory_share = available / total
        swap_share = swap_free / swap_total if swap_total > 0 else 0.0
        if memory_share >= MACHINE_MEMORY_LINE_FRACTION or swap_share >= MACHINE_MEMORY_LINE_FRACTION:
            return ""
        human = self.host_dispatch.human
        return (f"{MACHINE_MEMORY_LINE_TEXT}：这台整机可用内存 {human(available)}（占总量 {human(total)} 的 {memory_share:.1%}）、"
                f"swap 空闲 {human(swap_free)}（占 swap 总量 {human(swap_total)} 的 {swap_share:.1%}），都低于 {MACHINE_MEMORY_LINE_FRACTION:.0%}"
                f"（earlyoom 在两样都低于 10% 时杀进程）：这一轮不派新任务，等整机空下来")

    def shard_cap_fitted_to_room(self, task, on_host, host, snapshot, pending_memory):
        """分片的片：记满上限放不下、预测放得下时，上限与记账一起压到这台此刻放得下的量（task_server_lib.py 文件头「资源记录」的准入）。
        上限之和照旧不超过 slice，超了只杀它自己；压过的上限撞了照「重试」放大，放不下就排着。钉值不压。弄坏开关 shard-cap-not-fitted 下不压。"""
        if task.get("role") != "shard" or on_host.memory_history == lib.PINNED_HISTORY or lib.broken("shard-cap-not-fitted", self.environment):
            return on_host
        room = self.room_by_the_formula(host, snapshot)
        if room is None:
            return on_host
        if int(on_host.memory_predicted or 0) <= room < int(on_host.memory_booked or 0):
            return dataclasses.replace(on_host, memory_cap=room, memory_booked=room)
        return on_host

    def judged_on_host(self, task, prediction, host, snapshot, pending_memory):
        """这一条在这台上的内存放不放得下：→ (这台上的记账, judge_host 的判决)。排队（judge_placement）、起之前复核（launch_recheck）都调这一处（弄坏开关下另判排头的 head_judged_apart 同样）：
        记账取上限（booked_at_cap）、分片的片压到放得下（shard_cap_fitted_to_room），judge_host 判余量，放得下的再过用户定的公式（verdict_within_the_formula），次序不变。
        只对 host_limit 交了数的台调。不判卡：任务不拿卡（卡归 GPU 批处理验证服务），judge_host 的卡那一项交假。"""
        on_host = self.shard_cap_fitted_to_room(task, self.booked_at_cap(prediction, host, snapshot), host, snapshot, pending_memory)   # 记账 = 这台上的上限（文件头 ②「内存」）
        judged_pending = pending_memory
        verdict = self.host_dispatch.judge_host(host, snapshot.reports[host], on_host, False, judged_pending, 0,
                                                snapshot.memory_caps.get(host), self.host_margin(host, snapshot)[0])
        verdict = self.memory_verdict_reworded(verdict, on_host, host, snapshot, judged_pending)
        return on_host, self.verdict_within_the_formula(verdict, on_host, host, snapshot, judged_pending)

    def slice_floor_verdict(self, verdict, on_host, host, snapshot):
        """判决再过 slice 兜底（slice_floor_problem，比下限）：实测余量不够的放不下、原因以那一句起头（内存那一判本来就放不下的接在后面）；永远放不下的照原样。"""
        why = self.slice_floor_problem(host, snapshot)
        if not why or verdict.never:
            return verdict
        return dataclasses.replace(verdict, fits=False, never=False, why=why if verdict.fits else f"{why}；{verdict.why}")

    def host_admission(self, task, prediction, host, snapshot, view, pending_memory=None):
        """这一条此刻在这台上放不放得下：→ (这台上的记账, 判决)。派活（judge_placement）与起之前复核（launch_recheck）都只调这一处：
        内存（judged_on_host）之后判 slice 兜底（slice_floor_verdict：比这一条的预测峰值，实测余量不够的原因写在最前，内存那一判放不下的接在后面），
        放得下的再判整机内存线（machine_memory_problem）、编译目录占盘（build_disk_problem）与 CPU 闸（cpu_problem），
        哪一样放不下判决就是放不下、原因写那一样。
        排队先后的两条规则不在这里判（place_queued_into）。/health 不另判，只读派活给排头的结论（starved_problem）。只对 host_limit 交了数的台调。
        pending_memory 是调用方已读的这台待涨量（view.pending_of 交的那一个），没给就现读。"""
        if pending_memory is None:
            pending_memory = view.pending_of(host)   # 服务账上与派发器账上还没涨到量的，连同这一轮起了的
        on_host, verdict = self.judged_on_host(task, prediction, host, snapshot, pending_memory)
        verdict = self.slice_floor_verdict(verdict, on_host, host, snapshot)
        for why in (self.machine_memory_problem(host, snapshot), self.build_disk_problem(task, host, snapshot, view),
                    self.cpu_problem(task, host, snapshot)):
            if why and verdict.fits:   # 内存放得下、占盘或 CPU 放不下
                verdict = dataclasses.replace(verdict, fits=False, never=False, why=why)
        return on_host, verdict

    def recorded_prediction(self, task, base):
        """服务自己的资源记录（文件头 ②「内存」与 task_server_lib.py 文件头「资源记录」）：同一命令键最近 RECORD_WINDOW 次量到的峰值，
        服务自己的尝试按结束时刻从新到旧排，量不到的不算、不占名额。服务自己的尝试只用跑完的：退 0 的记量到的峰值，撞了这一条上限的按它的上限算；
        别的结局的峰值不进、不占名额。预测、上限与记账都等于近 RECORD_WINDOW 次最大，不乘 CAP_GROWTH_FACTOR；撞了上限照「重试」乘它放大。
        被停、判红的那几次（取消、命令自己退非 0、卡住、到四小时这类）量到的峰值只当下限（stopped_peak_floor）：比窗口里最大的还大才抬上去，小的不拉低、不占名额；
        窗口空着的，下限比没有记录的缺省（base）大才照它起，不然交 None 照缺省起。一次都没有交 None。"""
        window = sorted(self.recorded_measurements(task), reverse=True)[:RECORD_WINDOW]
        floor = self.stopped_peak_floor(task)
        if not window and (floor is None or floor <= int(base.memory_predicted or 0)):
            return None
        predicted = max([value for _, value in window] + ([floor] if floor is not None else []))
        cap = math.ceil(predicted / lib.MEBIBYTE) * lib.MEBIBYTE
        return dataclasses.replace(base, memory_predicted=predicted, memory_cap=cap, memory_booked=predicted, memory_history=lib.RECORD_HISTORY)

    def recorded_measurements(self, task, command_key=None, fits=None, skipped_tasks=frozenset()):
        """同一命令键服务自己结束了的尝试里进定额度的那几个量 → [(结束时刻, 字节)]。跑完（退 0）的记量到的峰值，撞了这一条上限的记它的上限（cap_hit_measurement）；别的结局不进。
        command_key 换读哪把键，fits 换「这一条记录与这一次的活相当吗」（缺省 record_fits_task），skipped_tasks 里的任务号的尝试不读。"""
        measurements = []
        fits = fits or self.record_fits_task
        unfinished_counted = lib.broken("unfinished-peaks-counted", self.environment)   # 弄坏开关：哪种结局的峰值都进
        key = task["command_key"] if command_key is None else command_key
        completed_later = False   # 从新到旧读：读到这一条时，它之后这把键有没有跑完过
        for attempt, valid in self.store.select("attempts", "command_key = ? AND state = 'ended'", (key,), order="id DESC LIMIT 200"):
            if not valid or not self.current_metric(attempt) or not fits(task, attempt) or attempt.get("task_id") in skipped_tasks:
                continue   # 峰值口径不是这一版的（含页缓存的）不读（task_server_lib.py 文件头「资源记录」的峰值口径一栏）；片数对不上的片不读
            outcome = attempt.get("outcome")
            cap_hit = outcome in lib.RECORD_CAP_HIT_OUTCOMES and not lib.broken("cap-hit-not-counted", self.environment)
            completed = outcome in lib.RECORD_COMPLETED_OUTCOMES
            if not (cap_hit or completed or unfinished_counted):
                continue   # 没跑完的那一次：提前失败的峰值偏小，不代表这条命令要多少
            when, peak = float(attempt.get("ended_at") or 0), attempt.get("memory_peak")
            if completed:
                completed_later = True
            if cap_hit and attempt.get("memory_cap"):
                measurements.append((when, self.cap_hit_measurement(attempt, unresolved=not completed_later)))
            elif completed and lib.broken("completed-peak-ignored", self.environment):
                continue   # 弄坏开关：跑完那次量到的峰值不进
            elif isinstance(peak, int) and (peak > 0 or lib.broken("unmeasured-recorded-as-zero", self.environment)):
                measurements.append((when, peak))
        return measurements

    def stopped_peak_floor(self, task):
        """同一命令键服务自己结束了的尝试里，不算跑完也不是撞上限的那几次（取消、命令自己退非 0、卡住、到四小时、被挤掉这类：被停或判红前量到的峰值）里
        最近 RECORD_WINDOW 次量到的峰值的最大值（字节）；一次都没有交 None。只当下限用（recorded_prediction）：这一次至少要这么多，提前退出的峰值偏小也不拉低记录。
        弄坏开关 stopped-peaks-not-counted 下交 None（被停的峰值丢掉，下一次从更低的档起、再撞一次）。"""
        if lib.broken("stopped-peaks-not-counted", self.environment):
            return None
        peaks = []
        for attempt, valid in self.store.select("attempts", "command_key = ? AND state = 'ended'", (task["command_key"],), order="id DESC LIMIT 200"):
            if not valid or not self.current_metric(attempt) or not self.record_fits_task(task, attempt):
                continue
            outcome = attempt.get("outcome")
            if outcome in lib.RECORD_COMPLETED_OUTCOMES or outcome in lib.RECORD_CAP_HIT_OUTCOMES:
                continue
            peak = attempt.get("memory_peak")
            if isinstance(peak, int) and not isinstance(peak, bool) and peak > 0:
                peaks.append((float(attempt.get("ended_at") or 0), peak))
        window = sorted(peaks, reverse=True)[:RECORD_WINDOW]
        return max(peak for _, peak in window) if window else None

    def cap_hit_measurement(self, attempt, unresolved=False):
        """撞了上限那一次进近几次最高的量（字节）：只经 lib.cap_hit_record_bytes 算（control drop-history --attempt-records-over-gib 判删哪几行算的同一个数）：
        峰值到了上限的一半或量不到的取上限，之后这把键还没有一次跑完的（unresolved）取上限乘 1.5，峰值远低于上限的取峰值（点 9 行也记 45.6G 那一类）。
        脚本永远不报自己要多少内存（用户 2026-11-13 定「脚本永远不要报自己要多少」）：上限只照命令键的记录定、撞了乘 1.5，命令打的任何「要多少」行一律不认。
        任务内撞上限之后的重试阶梯不看这里（next_cap）。弄坏开关 cap-hit-low-peak-counted-at-cap 下峰值远低于上限的也取上限。"""
        return lib.cap_hit_record_bytes(int(attempt["memory_cap"]), attempt.get("memory_peak"), unresolved, self.environment)

    def completed_runs(self, task):
        """库里同一命令键（与内存记录同一把键：路径、条目照词折过，--check 的格名照留）命令自己跑到退出的尝试，结束时刻最晚的在前 → [(结束时刻, 用时, 退没退 0)…]。
        命令自己跑到退出的（lib.RUN_COMPLETED_OUTCOMES：succeeded、command-failed）才算，被信号杀的 command-failed（SIGNAL_EXIT_CODES）不算；
        服务判出或打断的结局（撞上限、超时、卡住被停、被挤掉、取消、照进度打回停的）不在里面。
        弄坏开关 estimate-by-exact-command 下只认命令逐词相同的（任务 7585 与 7447 只差补丁目录，内存认得同键、上次运行时长却没有）。"""
        runs = []
        for attempt, valid in self.store.select("attempts", "command_key = ? AND state = 'ended'", (task["command_key"],), order="id DESC LIMIT 200"):
            if lib.broken("estimate-by-exact-command", self.environment) and attempt.get("command") != task.get("command"):
                continue
            if attempt.get("outcome") == "command-failed" and attempt.get("exit_code") in SIGNAL_EXIT_CODES:
                continue   # 被信号杀的：不是命令自己跑到退出
            if valid and attempt.get("outcome") in lib.RUN_COMPLETED_OUTCOMES and isinstance(attempt.get("seconds"), (int, float)):
                runs.append((float(attempt.get("ended_at") or 0), float(attempt["seconds"]), attempt.get("outcome") == "succeeded"))
        return sorted(runs, reverse=True)

    def attempt_shard_count(self, attempt):
        """这一次尝试所在那一趟共几片（交 int；不是片、记不到交 0）：尝试自己记的 shard_count（start_attempt 起尝试时写）；没记的读它的子任务的 shard_count。"""
        count = attempt.get("shard_count")
        if count is None:
            found = self.store.task(attempt.get("task_id"))
            count = found[0].get("shard_count") if found is not None and found[1] else None
        return int(count or 0)

    def record_fits_task(self, task, attempt):
        """定额度读的这一条记录（recorded_measurements、stopped_peak_floor）与这一次的活相当吗：片（role 是 shard）只认片数与这一片相同的那几趟
        （片的键不带片数，别的趟的同一片号落在同一把键上，活不一样大：两片那一趟的第 0 片比 64 片那一趟的大得多）；不是片的都算。
        弄坏开关 record-shard-count-unchecked 下片也不核片数。"""
        if task.get("role") != "shard" or lib.broken("record-shard-count-unchecked", self.environment):
            return True
        return self.attempt_shard_count(attempt) == int(task.get("shard_count") or 0)   # 片数对不上的这一把键的记录不当自己的，也不借

    def last_run_seconds(self, task):
        """这把命令键上次真跑完用了多久（记录，不是估的；只给插队规则 jump_refusal 用）→ (秒数或 None, 这个数是不是取自退非 0 的那一次)。判据：退 0 的取最近一次；退非 0 的取近 RECORD_WINDOW 次里最长的一次，
        比退 0 的那个数长才取它；没有退 0 的记录时它跑满 lib.FAILED_RUN_RECORD_MINIMUM_SECONDS 才取它，没跑满的交 None。
        弄坏开关 estimate-only-succeeded 下退非 0 的都不算，estimate-latest-any-outcome 下不分退码取最近一次，estimate-short-failure-counted 下早退的也算，
        estimate-failure-outside-window 下退非 0 的不限近几次，estimate-longest-succeeded 下退 0 的取最长一次。"""
        runs = self.completed_runs(task)
        if lib.broken("estimate-latest-any-outcome", self.environment):
            return (runs[0][1], not runs[0][2]) if runs else (None, False)
        succeeded = [seconds for _, seconds, clean in runs if clean]
        latest = None if not succeeded else max(succeeded) if lib.broken("estimate-longest-succeeded", self.environment) else succeeded[0]
        if lib.broken("estimate-only-succeeded", self.environment):
            return latest, False
        window = runs if lib.broken("estimate-failure-outside-window", self.environment) else runs[:RECORD_WINDOW]
        failed = max((seconds for _, seconds, clean in window if not clean), default=None)
        if failed is None:
            return latest, False
        minimum = 0.0 if lib.broken("estimate-short-failure-counted", self.environment) else lib.FAILED_RUN_RECORD_MINIMUM_SECONDS
        counts = failed >= minimum if latest is None else failed > latest
        return (failed, True) if counts else (latest, False)

    def unrecorded_prediction(self, task):
        """没有记录的命令（task_server_lib.py 文件头「资源记录」）：预测、上限与记账都取 UNRECORDED_MEMORY_DEFAULT_BYTES，不乘 1.5；撞了 250 由 retry_decision 放大重排。
        钉值、重排、服务自己的记录都从这一份换掉。
        弄坏开关 unrecorded-default-grown 下上限与记账都乘 CAP_GROWTH_FACTOR，unrecorded-booked-above-cap 下只把记账乘它。"""
        dispatch = self.host_dispatch
        default = lib.UNRECORDED_MEMORY_DEFAULT_BYTES
        if lib.broken("unrecorded-default-grown", self.environment):
            default = math.ceil(default * lib.CAP_GROWTH_FACTOR / lib.MEBIBYTE) * lib.MEBIBYTE
        booked = default
        if lib.broken("unrecorded-booked-above-cap", self.environment):
            booked = math.ceil(default * lib.CAP_GROWTH_FACTOR / lib.MEBIBYTE) * lib.MEBIBYTE
        return dispatch.Prediction(task["command_key"], default, default, booked, lib.UNRECORDED_HISTORY)

    def prediction_of(self, task):
        """这一条的内存预测（文件头 ②「预测每条只算一次」）：第一次算出后存下，之后直接用，不再读峰值表、不再扫记录；next_cap 变了作废重算。
        弄坏开关 prediction-recomputed-every-pass 下每次都现算。同一条拿它自己那一把锁（prediction_task_locks），别的任务的取、算不等它。"""
        with keyed_lock(self.prediction_task_locks, self.prediction_lock, task["id"]):
            with self.prediction_lock:
                stored = self.predictions.get(task["id"])
            if stored is not None and stored[0] == task.get("next_cap") and not lib.broken("prediction-recomputed-every-pass", self.environment):
                return stored[1]
            return self.store_prediction(task)

    def fresh_prediction(self, task):
        """起之前复核用：现算一次、换掉存的那一份。"""
        with keyed_lock(self.prediction_task_locks, self.prediction_lock, task["id"]):
            return self.store_prediction(task)

    def store_prediction(self, task):
        """现算（computed_prediction：扫这把键的记录、读钉值）不拿 prediction_lock，算完再拿着它存；调用方拿着这一条自己那一把锁。
        算的时候存的预测整表作废了（prediction_generation 变了）的不存：算的是作废之前的钉值，下一回照新的现算。"""
        with self.prediction_lock:
            generation = self.prediction_generation
        prediction = self.computed_prediction(task)
        with self.prediction_lock:
            if generation == self.prediction_generation:
                self.predictions[task["id"]] = (task.get("next_cap"), prediction)
        return prediction

    def forget_prediction(self, task_id):
        with self.prediction_lock:
            self.predictions.pop(task_id, None)
            self.build_disk_predictions.pop(task_id, None)
            self.admission_threads_cache.pop(task_id, None)
            self.pin_unreadable_reasons.pop(task_id, None)
            self.budget_notices.pop(task_id, None)

    def keep_predictions_of(self, task_ids):
        """只留还在队里的那几条存的预测。"""
        with self.prediction_lock:
            self.predictions = {task_id: stored for task_id, stored in self.predictions.items() if task_id in task_ids}
            self.build_disk_predictions = {task_id: stored for task_id, stored in self.build_disk_predictions.items() if task_id in task_ids}
            self.admission_threads_cache = {task_id: stored for task_id, stored in self.admission_threads_cache.items() if task_id in task_ids}
            self.pin_unreadable_reasons = {task_id: why for task_id, why in self.pin_unreadable_reasons.items() if task_id in task_ids}

    # ── 编译目录占盘（文件头 ②「编译目录占盘」；records/2026-10-31-task-server巡检.md 问题 76）──
    def build_disk_prediction(self, task):
        """这条命令的编译目录占盘预测（字节）：同一命令键最近 RECORD_WINDOW 次跑成（succeeded）而量到了占盘峰值（build_peak，两台都记）的尝试取最大，
        取法同内存峰值（recorded_prediction）；一次都没量到交 0。每条在排任务只算一次。"""
        with self.prediction_lock:
            if task["id"] in self.build_disk_predictions:
                return self.build_disk_predictions[task["id"]]
        measured = []
        for attempt, valid in self.store.select("attempts", "command_key = ? AND state = 'ended'", (task["command_key"],), order="id DESC LIMIT 200"):
            peak = attempt.get("build_peak")
            if valid and attempt.get("outcome") in lib.RECORD_COMPLETED_OUTCOMES and isinstance(peak, int) and not isinstance(peak, bool) and peak > 0:
                measured.append((float(attempt.get("ended_at") or 0), peak))
        predicted = max((peak for _, peak in sorted(measured, reverse=True)[:RECORD_WINDOW]), default=0)
        with self.prediction_lock:
            self.build_disk_predictions[task["id"]] = predicted
        return predicted

    def server_disk_pending(self, host):
        """服务账上这台在跑的每一次尝试编译目录还要涨的占盘：max(0, 它起的时候记的占盘 − 此刻量到的)；此刻量到的照那一台的接口读（Machine.build_bytes：
        本机读尝试目录里 attempt_runner 写的 build_bytes，第二台量不到），量不到的照记的算。"""
        pending = 0
        for attempt in self.store.running_attempts():
            if attempt.get("host") != host or not attempt.get("disk_booked"):
                continue
            used = self.machines[host].build_bytes(attempt) or 0
            pending += max(0, int(attempt["disk_booked"]) - used)
        return pending

    def build_disk_problem(self, task, host, snapshot, view):
        """这一条在这台上编译目录占盘放不放得下：那台盘余量（报告里的 DISK_REPORT_KEY）减去在跑的还要涨的，
        要放得下这条命令的占盘预测加 BUILD_DISK_FLOOR_BYTES；放不下交排队原因（点名占盘），放得下、不会编又没量过、余量读不到的交空串。
        弄坏开关 build-disk-unpredicted 下一律空串（改前只看第二台的下限）。"""
        if lib.broken("build-disk-unpredicted", self.environment):
            return ""
        available = (snapshot.reports.get(host) or {}).get(DISK_REPORT_KEY)
        if not isinstance(available, int) or isinstance(available, bool):
            return ""
        predicted = self.build_disk_prediction(task)
        if not predicted and not task.get("compiles"):
            return ""
        pending = view.disk_pending_of(host)
        needed = predicted + BUILD_DISK_FLOOR_BYTES
        if available - pending >= needed:
            return ""
        return (f"编译目录占盘：这条命令量过的编译目录峰值 {gibibytes_text(predicted)} 加下限 {gibibytes_text(BUILD_DISK_FLOOR_BYTES)} 要 {gibibytes_text(needed)}，"
                f"{lib.HOST_NAMES.get(host, host)}盘此刻剩 {gibibytes_text(available)}、在跑的编译目录还要涨 {gibibytes_text(pending)}，放不下"
                "（等在跑的跑完、task clean 删掉留下的编译目录）")

    def computed_prediction(self, task):
        """现算（扫服务自己的记录；不读内存峰值表）：文件头 ②「内存」。"""
        self.selftest_read(f"predict task={task['id']}")
        base = self.unrecorded_prediction(task)
        pinned = self.pinned_prediction(task, base)
        if pinned is not None:
            return pinned
        if task.get("next_cap"):
            cap = int(task["next_cap"])
            prediction = dataclasses.replace(base, memory_predicted=cap, memory_cap=cap, memory_booked=cap, memory_history="retry")
        else:
            recorded = None if lib.broken("record-ignored-for-cap", self.environment) else self.recorded_prediction(task, base)
            prediction = recorded or base   # 这把键自己没有记录：从 4G 起，别的键、同一趟的兄弟片都不借，撞了再放大；有记录的照近几次最大，不抬到 4G（文件头 ②「内存」）
        return self.memory_floor_applied(prediction, task)

    def workspace_writer_floor(self, task):
        """换文件、打补丁的工具（workspace_swap_tool：换钩子、打补丁）内存上限的下限（字节）：撞上限被杀会把换到一半的结局文件留在主工作区，之后所有会话的换上都被挡，
        所以上限留足余量，不照峰值×1.5 抠（任务 13711 峰值不到上限就撞了 858M）。不是这类命令的（含合入后验证）、自证里换了内存下限（TASK_SERVER_SELFTEST_MEMORY_FLOOR_BYTES）的交 0。
        弄坏开关 workspace-writer-cap-unpadded 下交 0。"""
        if (task is None or lib.broken("workspace-writer-cap-unpadded", self.environment) or lib.selftest_override(lib.MEMORY_FLOOR_VARIABLE, self.environment).isdigit()
                or not self.workspace_swap_tool(task)):
            return 0
        return WORKSPACE_WRITER_FLOOR_BYTES

    def memory_floor(self):
        """内存上限的下限（字节；文件头 ②「内存」的内存上限下限）：lib.MEMORY_FLOOR_BYTES（1 GiB）。自证的小世界里由 TASK_SERVER_SELFTEST_MEMORY_FLOOR_BYTES 换
        （task_server_lib.py 文件头「只在自证里认的变量」：各格写 0，下限与开关只红点名的几格）。弄坏开关 cap-floor-skipped 下是 0（不设下限），
        recorded-cap-floored 下是 lib.MEMORY_FLOOR_RAISED_BYTES（4G）。"""
        override = lib.selftest_override(lib.MEMORY_FLOOR_VARIABLE, self.environment)
        if override.isdigit():
            return int(override)
        if lib.broken("cap-floor-skipped", self.environment):
            return 0
        return lib.MEMORY_FLOOR_RAISED_BYTES if lib.broken("recorded-cap-floored", self.environment) else lib.MEMORY_FLOOR_BYTES

    def memory_floor_applied(self, prediction, task=None):
        """上限与记账不低于内存上限的下限（文件头 ②「内存」的内存上限下限）：钉值之外上限低于 memory_floor 的，上限取它、记账取它与原记账的大者，预测的峰值照原样；
        换文件、打补丁的工具另取 workspace_writer_floor 与它的大者。钉值不动（computed_prediction 里钉值先交了，不到这里；这里再认一次 PINNED_HISTORY）。
        收到台能给的最大上限为止由 booked_at_cap 做，这里不看台。"""
        floor = max(self.memory_floor(), self.workspace_writer_floor(task))
        if prediction.memory_history == lib.PINNED_HISTORY or int(prediction.memory_cap or 0) >= floor:
            return prediction
        return dataclasses.replace(prediction, memory_cap=floor, memory_booked=max(int(prediction.memory_booked or 0), floor))

    # ── 钉值（文件头 ②「内存」的钉值；用户：「变异的内存上限直接按照 根目录设置的50gb走」「50 这个是读取的配置 不是随便给的」）──
    def pin_record(self, command_key):
        """库里这个命令键的钉值记录（history 表 state=pin 那一种）；没有交 None。校验不过的不认。"""
        for record, valid in self.store.select("history", "command_key = ? AND state = ?", (command_key, lib.PIN_STATE)):
            if valid:
                return record
        return None

    def configured_slice_caps(self):
        """多机配置 SLICE_MEMORY_CAP_BY_HOST 里两台各自那一项（字节）现读：→ ({"local": 字节, "peer": 字节} 或 None, 读不到的原因)（文件头 ②「内存」的钉值）。
        先照 task_server_lib.py 的 configured_by_host 切、判（每台恰好一项、地址都在 HOSTS 里），再照 Resources.addresses 把这一台与另一台的地址映到 local / peer；
        每一项写法照 scripts/hosts/run-with-memory-cap.sh 读它的那一句（正整数加 K / M / G / T）。"""
        values, problem = lib.configured_by_host(self.configuration.path, lib.PIN_SOURCE, self.environment)
        if values is None:
            return None, problem
        sizes = {}
        for address, text in values.items():   # 先判写法、再认台：写法不对的配置多机配置判法也判不过，认台那一步就说不出是哪一项写错了
            found = SLICE_CAP_VALUE.match(text)
            if not found:
                return None, f"多机配置 {self.configuration.path} 里 {lib.PIN_SOURCE} 写法不对：{address} 那一项「{text}」要写成正整数加 K / M / G / T"
            sizes[address] = int(found.group(1)) * SIZE_UNITS[found.group(2)]
        addresses, problem = self.resources.addresses()
        if addresses is None:
            return None, f"认不出多机配置 {lib.HOSTS_KEY} 里哪一项是服务所在这一台，{lib.PIN_SOURCE} 按台取不了（{problem}）"
        caps = {}
        for host, machine in self.machines.items():
            address = machine.address(addresses)
            if address not in sizes:   # 只在弄坏开关 by-host-missing-accepted 下走到：少了的那一台没有那一项
                return None, f"多机配置 {self.configuration.path} 的 {lib.PIN_SOURCE} 里没有{lib.HOST_NAMES[host]}（{address}）那一项"
            caps[host] = sizes[address]
        return caps, ""

    def pinned_totals(self, record):
        """钉了的命令两台各取的总上限（字节）：现读多机配置 SLICE_MEMORY_CAP_BY_HOST 里两台各自那一项（配多少就是多少，不记字面的数）；
        弄坏开关 pinned-cap-literal 下取钉的那一刻记下的数。→ ({"local": 字节, "peer": 字节} 或 None, 读不到的原因)。"""
        if lib.broken("pinned-cap-literal", self.environment) and isinstance(record.get("bytes_at_pin"), dict):
            return record["bytes_at_pin"], ""
        return self.configured_slice_caps()

    def pin_value(self, record, snapshot):
        """钉值此刻两台各是多少字节（control pin-cap 与 status 报的那个数）：那一台单条能给到的最大上限 = pinned_totals 里那一项减那一台此刻的余量
        （host_margin；派活时 booked_at_cap 照同一个口径现算，不存），超过这台单条能给到的最大上限（host_capacity：slice 总上限减余量）的当那个数用（pin_over_limit_lines 报）。
        → ({台: 字节}（量得到余量的那几台）或 None, 原因）：配置读不到交 None 与原因；哪一台量不到余量的不给它的数，原因写它。
        弄坏开关 pinned-cap-at-slice-total 下不减余量，pinned-over-limit-unreported 下超了也照那一项减余量报。"""
        caps, problem = self.pinned_totals(record)
        if caps is None:
            return None, problem
        values, problems = {}, []
        for host, value in caps.items():
            margin, why = self.host_margin(host, snapshot)
            if margin is None:
                problems.append(why)
                continue
            values[host] = value if lib.broken("pinned-cap-at-slice-total", self.environment) else value - margin
            limit = self.host_capacity(host, snapshot)[0]
            if limit is not None and values[host] > limit and not lib.broken("pinned-over-limit-unreported", self.environment):
                values[host] = limit
        return values, "；".join(problems)

    def pin_over_limit_lines(self, record, snapshot):
        """钉值超过这台单条能给到的最大上限的那几台（文件头 ②「内存」的钉值）：多机配置那一项减此刻的余量大过 host_capacity（slice 总上限减余量）的各一句，当 host_capacity 用；control pin-cap 回读时打。
        弄坏开关 pinned-over-limit-unreported 下交空表。"""
        if lib.broken("pinned-over-limit-unreported", self.environment):
            return []
        caps, _ = self.pinned_totals(record)
        human = self.host_dispatch.human
        lines = []
        for host, value in sorted((caps or {}).items()):
            margin, limit = self.host_margin(host, snapshot)[0], self.host_capacity(host, snapshot)[0]
            if margin is not None and limit is not None and value - margin > limit:
                lines.append(f"{lib.HOST_NAMES.get(host, host)}：{lib.PIN_SOURCE} 那一项 {human(value)} 减此刻的余量 {human(margin)} 是 {human(value - margin)}，"
                             f"超过这台 slice 总上限减余量的 {human(limit)}，钉值当 {human(limit)} 用")
        return lines

    def pin_refusal(self, record, snapshot):
        """control pin-cap 当场拒的那一句（文件头 ②「内存」的钉值）：量得到余量的哪一台，钉值（那一项减此刻的余量）不大于 0，就交那一句与出路；都大于 0 交空串。
        弄坏开关 pinned-nonpositive-accepted 下一律交空串。"""
        if lib.broken("pinned-nonpositive-accepted", self.environment):
            return ""
        caps, _ = self.pinned_totals(record)
        values, _ = self.pin_value(record, snapshot)
        human = self.host_dispatch.human
        low = [f"{lib.HOST_NAMES.get(host, host)}那一项 {human(caps[host])} 减此刻的余量 {human(caps[host] - value)} 是 {human(value)}"
               for host, value in sorted((values or {}).items()) if value <= 0]
        if not low:
            return ""
        return (f"多机配置 {self.configuration.path} 的 {lib.PIN_SOURCE} 里{'；'.join(low)}，不大于 0：照这个数那一台一条都起不来，没钉。"
                f"出路：把多机配置里那一项改大（写的是那台 slice 的总上限，配多少就是多少；余量由 task-server 每台现量、不在配置里），再 pin-cap")

    def pinned_prediction(self, task, base):
        """钉了的命令：预测、上限与记账都取钉值（学到的峰值、next_cap 都不顶掉它）；没钉的交 None。钉了而配置读不到的交一份 memory_history 是
        lib.PIN_UNREADABLE_HISTORY 的（数照 base，只给排队的别处算账用），读不到的那一句记进 pin_unreadable_reasons，place 照它记 run-condition-unmet、不起（文件头 ②「内存」的钉值）。
        弄坏开关 pinned-cap-ignored 下不认钉值；pinned-cap-grown 下撞过上限的照 next_cap 放大；pin-unreadable-predicted 下读不到的照改前交 None（照常预测，说一句）。"""
        if lib.broken("pinned-cap-ignored", self.environment):
            return None
        if task.get("role") in ("plan", "merge"):   # 分片的计划与合并只算键、核账：命令键与父任务相同也不吃钉值，照自己的记录定，不占整台
            return None
        record = self.pin_record(task["command_key"])
        if record is None and task.get("role") == "shard" and task.get("parent_id"):
            # 分片的各片照父任务的钉值（父任务钉了整台，片按自己的命令键查不到钉值、会从小上限一档档爬）；计划与合并只算键、核账，照自己的记录定，不占整台
            parent = self.store.task(task["parent_id"])
            if parent is not None and parent[1]:
                record = self.pin_record(parent[0].get("command_key", ""))
        if record is None:
            return None
        if task.get("next_cap") and lib.broken("pinned-cap-grown", self.environment):
            return None
        caps, problem = self.pinned_totals(record)
        if caps is None:
            if lib.broken("pin-unreadable-predicted", self.environment):
                say(f"  ! 任务 {task['id']} 的命令钉了上限（取多机配置的 {lib.PIN_SOURCE}），此刻读不到：{problem}；这一回照常预测")
                return None
            self.pin_unreadable_reasons[task["id"]] = (
                f"这条命令钉了上限（取多机配置的 {lib.PIN_SOURCE} 里派到的那一台那一项），此刻读不到：{problem}；没起，不退回学到的记录。"
                f"出路：把多机配置 {self.configuration.path} 的 {lib.PIN_SOURCE} 写对（一行，<地址>=<正整数加 K / M / G / T> 空白分隔，{lib.HOSTS_KEY} 里每一台恰好一项），"
                f"或 python3 task-server/control unpin-cap <命令…> 撤掉钉值，再重交")
            return dataclasses.replace(base, memory_history=lib.PIN_UNREADABLE_HISTORY)
        # 数先取服务这一台那一项（没落到哪一台时的口径），判哪一台时 booked_at_cap 照 memory_by_host 换成那一台那一项、减那台此刻的余量（pinned_on_host）；
        # 存下来的只是配置里的那一项，余量每一轮现量，不存
        value = caps[LOCAL]
        return dataclasses.replace(base, memory_predicted=value, memory_cap=value, memory_booked=value, memory_history=lib.PINNED_HISTORY, memory_by_host=dict(caps))

    def pin_cap(self, command, pinned, session, process_id):
        """control pin-cap / unpin-cap（接口 POST /pins）：照 memory_peaks_lib 换算命令键，钉上或撤掉；→ 回话（键、此刻钉没钉、这一回改没改、钉值此刻多少）。"""
        key = self.command_key(command, None)
        record = self.pin_record(key)
        changed = False
        snapshot = self.resources.snapshot()
        if pinned:   # 那一项不比那台此刻的余量大：当场拒，不钉、不改已有的（文件头 ②「内存」的钉值）
            refusal = self.pin_refusal(record or {}, snapshot)
            if refusal:
                self.operation("pin-cap", 0, session, process_id, f"{key[:200]}（拒了：{refusal[:200]}）")
                return dict(self.pins_reply(key, snapshot=snapshot), refused=refusal)
        if pinned and record is None:
            record = {"state": lib.PIN_STATE, "session": "", "command_key": key, "kind": lib.PIN_STATE, "source": lib.PIN_SOURCE,
                      "command": shlex.join(command)[:500], "pinned_at": round(time.time(), 3)}
            if lib.broken("pinned-cap-literal", self.environment):
                record["bytes_at_pin"] = self.configured_slice_caps()[0]
            self.store.insert("history", record)
            changed = True
        elif not pinned:
            for found, _ in self.store.select("history", "command_key = ? AND state = ?", (key, lib.PIN_STATE)):
                self.store.delete_row("history", found["id"])
                changed = True
        self.operation("pin-cap" if pinned else "unpin-cap", 0, session, process_id, f"{key[:200]}（{'改了' if changed else '原来就是这样'}）")
        with self.prediction_lock:
            self.predictions = {}   # 存的预测作废：在排的下一轮照新的钉值现算
            self.prediction_generation += 1   # 此刻正在算的那几条算完不存（store_prediction）
        return self.pins_reply(key, changed, snapshot)

    def pins_reply(self, key, changed=False, snapshot=None):
        record = self.pin_record(key)
        caps, problem = self.pin_value(record, snapshot or self.resources.snapshot()) if record is not None else (None, "")
        over_limit = self.pin_over_limit_lines(record, snapshot or self.resources.snapshot()) if record is not None else []
        return {"command_key": key, "pinned": record is not None, "changed": changed, "source": lib.PIN_SOURCE, "bytes_by_host": caps, "problem": problem, "over_limit": over_limit}

    def server_pending(self, host, report):
        """内存字节：服务自己账上这台在跑的每一次尝试还没涨到量的那一截 max(0, 它的记账 − 它的 scope 此刻回收不掉的用量)（文件头 ②「内存」）；
        记账照 running_booked（有 scope 的照实测，这一截是 0），报告里没有它的 scope 的记整份上限：待涨量只来自刚起、还没读数的那几条。
        两台同一段：用量照 report（这一轮读这台余量的那一份报告，没有是 None）里的在跑的 scope（report_scope_usage，scope 照进程环境里的
        TASK_SERVER_ATTEMPT_LABEL 对到尝试的 label 上，memory_peaks_lib.py 文件头「在跑的 scope」），对不上这一次尝试的 label、或没有报告的记整份上限。"""
        if lib.broken("server-pending-ignored", self.environment):
            return 0
        memory = 0
        usage = self.report_scope_usage(report)
        for attempt in self.booking_attempts():
            if attempt.get("host") != host:
                continue
            # 在跑的记账（文件头 ②「内存」）：还没涨到量的那一截 = 记账 − 它此刻回收不掉的用量；弄坏开关 booked-at-peak 下用起的时候记的那一份
            booked = (attempt.get("booked", 0) or 0) if lib.broken("booked-at-peak", self.environment) else self.running_booked(attempt, usage)
            used = usage.get(attempt.get("label") or "")
            memory += booked if used is None else max(0, booked - used)   # 报告里没有它的 scope：整份上限，不记 0
        return memory

    def report_scope_usage(self, report):
        """{标签: 回收不掉的用量（字节）}：一台报告里 slice 下各个在跑的 scope（memory_peaks_lib.SCOPES_REPORT_KEY，那份文件头「在跑的 scope」）。
        没标签的、用量读不了（不是非负整数）的不进；几个 scope 带同一个标签取最小的那一份（文件头 ②「内存」）。报告是 None 或没有这一项交空表。"""
        usage = {}
        scopes = ((report or {}).get("memory") or {}).get(self.peaks.SCOPES_REPORT_KEY)
        for entry in (scopes.values() if isinstance(scopes, dict) else ()):
            if not isinstance(entry, dict):
                continue
            label, used = entry.get(self.peaks.SCOPE_LABEL_KEY), entry.get(self.peaks.SCOPE_USED_KEY)
            if not label or not isinstance(label, str) or not isinstance(used, int) or isinstance(used, bool) or used < 0:
                continue
            usage[label] = min(used, usage.get(label, used))
        return usage

    def place(self, task, view):
        """这一轮判这一条放不放得下、放哪台（judge_placement）；判成永远放不下而多机配置里有一台此刻连不上的，改判此刻放不下、排着等它回来再判
        （一台停机不把只有它放得下的任务判死）。弄坏开关 down-host-judged-never 下判永远放不下。"""
        placement = self.judge_placement(task, view)
        if placement.kind == "start":   # 靠 CPU 闸要核数少的名额起的，记下那一句（文件头 ②「CPU 闸」的名额；起之前复核照现读的那一份重判）
            return dataclasses.replace(placement, cpu_small_slot=self.cpu_small_slot_note(task, placement.host, view.snapshot()))
        if placement.kind != "never" or lib.broken("down-host-judged-never", self.environment):
            return placement
        snapshot = view.snapshot()
        down = [host for host in MACHINE_ORDER if snapshot.expected.get(host) and not snapshot.reachable.get(host)]
        if not down:
            return placement
        names = "、".join(self.machines[host].label for host in down)
        return dataclasses.replace(placement, kind="blocked", why=f"{placement.why}；{names}此刻用不了，等它回来再判放不放得下")

    def judge_placement(self, task, view):
        """这一轮判这一条放不放得下、放哪台：预测用存的那一份（prediction_of），余量与待涨量用这一轮读的那一份（view）。
        排队先后的两条规则（排满时限的挡住后面、插队只许比在跑的剩余短的）不在这里判，在排队那一轮里（place_queued_into、jump_refusal）。"""
        snapshot = view.snapshot()
        if snapshot.hold:
            return Placement("blocked", why="自证的假余量写了 hold")
        hosts = self.candidate_hosts(task, snapshot)
        if task.get("role") == "shard":   # 这一片要拿到它所在那一台在 HOSTS 里的地址（task_server_lib.py 文件头「分片」②）：认不出就不起
            address, problem = self.shard_host_address(LOCAL)
            if not address:
                return Placement("unmet", why=problem)
        peer_only = self.peer_only_entry(task) if hosts == [PEER] else None
        if len(hosts) == 1 and not snapshot.reachable.get(hosts[0]) and not lib.broken("pinned-peer-unavailable-waits", self.environment):
            only, why = self.machines[hosts[0]].label, snapshot.down.get(hosts[0], "")   # 钉了台（重排换台、弄坏开关下只在第二台跑的入口）而那一台此刻连不上
            if peer_only:
                return Placement("never", why=f"钉在{only}（只在第二台跑的入口 {peer_only[0]}：{peer_only[1]}）而{only}此刻用不了：{why}")
            return Placement("never", why=f"钉在{only}而{only}此刻用不了：{why}")
        if peer_only and snapshot.reachable.get(PEER) and not self.host_eligible(task, PEER, snapshot):
            return Placement("never", why=f"只在第二台跑的入口 {peer_only[0]}（{peer_only[1]}），这一条派不过去：工作目录要是仓根、参数里的本机路径要带得过去、"
                                          "不点只在本机的目录与写本机工作区的工具（task_server_lib.py 文件头「挑机器」）")
        if task.get("compiles_problem"):   # 会编的声明写坏了：不猜，打回（task_server_lib.py 文件头「编译目录」的入口声明）
            return Placement("unmet", why=f"会编的声明写坏了：{task['compiles_problem']}。出路：改成「{lib.DECLARED_COMPILES_MARK} {lib.COMPILES_DECLARED_WORD}」或删掉那一行再重交")
        prediction = self.prediction_of(task)
        if prediction.memory_history == lib.PIN_UNREADABLE_HISTORY:   # 钉了而配置读不到：不起，记 run-condition-unmet（pinned_prediction）
            return Placement("unmet", why=self.pin_unreadable_reasons.get(task["id"], f"这条命令钉了上限而多机配置的 {lib.PIN_SOURCE} 此刻读不到"))
        verdicts, free_after_pending, remaining, booked = {}, {}, {}, {}
        unusable = {}   # {台: 这一轮不派它的原因}：余量量不到（探测失败、cgroup 读不出）、总上限不比余量大、钉值不大于 0（文件头 ②「余量」）
        for host in hosts:
            if host not in snapshot.reports:
                if snapshot.down_kind.get(host) != DISK_FLOOR_KIND:   # 盘余量不到下限的那台不是量不到余量，照 down_text 写
                    unusable[host] = self.host_limit(prediction, host, snapshot)[1]
                continue
            limit, why = self.host_limit(prediction, host, snapshot)
            if limit is None:
                unusable[host] = why
                continue
            pending_memory = view.pending_of(host)
            on_host, verdicts[host] = self.host_admission(task, prediction, host, snapshot, view, pending_memory)   # 内存、编译目录占盘（文件头 ②「编译目录占盘」）与 CPU 闸
            booked[host] = on_host.memory_booked
            free_after_pending[host] = snapshot.reports[host]["memory"]["free"] - pending_memory
            remaining[host] = min(free_after_pending[host], int(snapshot.reports[host]["memory"].get("slice_room") or 0) - pending_memory,
                                  self.host_total(host, snapshot) - pending_memory)   # 均衡挑台用：空闲、slice 余量、总上限减已记的账三者的小者
        # 预测峰值比这台大、别处可能装得下的这台不派（文件头 ②「内存」的准入记账；hosts_short_of_record）：照永远放不下判，别处都放不下时 place 照「等它回来」排着
        for host, why in self.hosts_short_of_record(prediction, hosts, snapshot).items():
            if host in verdicts:
                verdicts[host] = dataclasses.replace(verdicts[host], fits=False, never=True, why=why)
        # 哪一台此刻用不了（盘余量不到下限、余量查不成、连不上这类；task_server_lib.py 文件头「挑机器」）：排队原因带上那一句，不只写别的台放不下
        down_text = "".join(f"（{self.machines[host].label}此刻用不了：{snapshot.down[host]}）" for host in MACHINE_ORDER
                            if snapshot.expected.get(host) and snapshot.down.get(host) and host not in snapshot.reports and host not in unusable)
        unusable_text = "；".join(unusable.values())
        if unusable and len(unusable) == len(hosts) and not lib.broken("margin-unmeasured-waits", self.environment):
            # 能派的每一台这一轮都量不到余量（或钉值不大于 0）：运行条件不满足，不起，不退回固定的数（task_server_lib.py 文件头「错误类型」的 run-condition-unmet）
            return Placement("unmet", why=f"能派的每一台这一轮都给不出上限，不起：{unusable_text}。出路：照上面那一句修好那台的余量读数"
                                          "（singlefs-heavy.slice 与它的子 cgroup 的 memory.current、memory.stat 读得了，systemctl --user 查得了，第二台连得上）或多机配置，再重交")
        if not verdicts:
            return Placement("blocked", why="能派的那几台此刻都查不到余量" + down_text + (f"（{unusable_text}）" if unusable_text else ""))
        # 能派而盘余量不到下限的台（DISK_FLOOR_KIND：此刻用不了，盘腾出来之后可能放得下；hosts_short_of_record 照「给不出可用上限」把别的台判成不派）：
        # 有这样的台，各台内存都永远放不下也不判永远放不下，排着等。弄坏开关 disk-floor-host-judged-never 下盘不够的台不算
        held_back = [] if lib.broken("disk-floor-host-judged-never", self.environment) else [
            host for host in hosts if host not in snapshot.reports and snapshot.down_kind.get(host) == DISK_FLOOR_KIND]
        common = {"history": prediction.memory_history, "disk_booked": self.build_disk_prediction(task)}
        fitting = [host for host in hosts if host in verdicts and verdicts[host].fits]
        if (lib.broken("peer-overflow-blocked", self.environment) and hosts[:1] == [LOCAL] and LOCAL in verdicts and not verdicts[LOCAL].fits
                and not verdicts[LOCAL].never):
            fitting = [host for host in fitting if host == LOCAL]   # 弄坏开关：照改前本机在前的放不下也不派第二台
        if task.get("role") == "shard":
            chosen = fitting[0] if fitting else None   # 分片照计划：放得下的里取排在最前的那台，不按均衡改台
        elif lib.broken("balance-ignored", self.environment):
            chosen = fitting[0] if fitting else None   # 弄坏开关：放得下的里取排在最前的，不比哪台更空
        else:   # 放得下的里挑放下之后剩余占这台总上限比例大的，一样大取排在前面的（task_server_lib.py 文件头「挑机器」）
            chosen = max(fitting, key=lambda host: ((remaining[host] - booked[host]) / max(1, self.host_total(host, snapshot)),
                                                    -hosts.index(host))) if fitting else None
        if chosen is None and lib.broken("launch-when-unplaceable", self.environment):
            chosen = next(iter(verdicts))
        if chosen is not None:
            return Placement("start", host=chosen, cap=verdicts[chosen].cap, booked=booked[chosen], **common)
        reasons = "；".join(f"{self.machines[host].label}：{verdict.why}" for host, verdict in verdicts.items())
        reasons += down_text + (f"；{unusable_text}（这一轮不派）" if unusable_text else "")
        if all(verdict.never for verdict in verdicts.values()) and not unusable and not held_back:   # 有台量不到余量、或盘不够（held_back）：下一轮它可能放得下
            return Placement("never", why=reasons)
        return Placement("blocked", why=reasons)

    def launch_recheck(self, task, placement, view):
        """起之前复核（文件头 ②「起之前复核」）：现算这一条的预测、现读它要去的那台的余量与待涨量（缓存作废），按现读的数再判一次那台放不放得下。
        → (放得下时照现算的数改过上限与记账的 Placement，放不下时 None, 放不下的那一句)。现读的那份换进这一轮，后面的照它判。
        弄坏开关 launch-recheck-skipped 下不复核；launch-when-unplaceable 那一种照它的样子起（放不下也起），同样不复核。"""
        if lib.broken("launch-recheck-skipped", self.environment) or lib.broken("launch-when-unplaceable", self.environment):
            return placement, ""
        host = placement.host
        prediction = self.fresh_prediction(task)
        if prediction.memory_history == lib.PIN_UNREADABLE_HISTORY:   # 排队那一轮读得到、起之前读不到：这一轮不起，下一轮 place 记 run-condition-unmet
            self.selftest_read(f"recheck task={task['id']} host={host} fits=0")
            return None, self.pin_unreadable_reasons.get(task["id"], f"钉值读不到（多机配置的 {lib.PIN_SOURCE}）")
        self.selftest_pause("launch-recheck")
        judged = view.snapshot()   # 排队那一轮判它用的那一份
        rechecked = self.resources.recheck(host, judged)
        # 现读的那份照排队那一轮的读法再过一遍（peer_view：复核这一刻第二台没挂上共享根的当它此刻用不了，没起的 slice 照配置补）；
        # 弄坏开关 recheck-nfs-unchecked 下只补没起的 slice，复核这一刻才冒出来的 NFS 挂载问题不判
        snapshot = self.unstarted_slices_filled(rechecked) if lib.broken("recheck-nfs-unchecked", self.environment) else self.peer_view(rechecked)
        view.refresh(host, snapshot)
        limit, why = self.host_limit(prediction, host, snapshot)   # 现读的这一份里这台量不到余量、或钉值不大于 0：这一轮不起（文件头 ②「余量」）
        if limit is None:
            self.selftest_read(f"recheck task={task['id']} host={host} fits=0")
            return None, why
        on_host, verdict = self.host_admission(task, prediction, host, snapshot, view)
        self.selftest_read(f"recheck task={task['id']} host={host} fits={int(verdict.fits)}")
        if not verdict.fits:
            return None, f"{lib.HOST_NAMES.get(host, host)}：{verdict.why}"
        fresh = dataclasses.replace(placement, cap=verdict.cap, booked=on_host.memory_booked,
                                    history=prediction.memory_history, cpu_small_slot=self.cpu_small_slot_note(task, host, snapshot))
        return fresh, ""

    def cpu_problem(self, task, host, snapshot):
        """CPU 闸（文件头 ②「CPU 闸」）：交排队原因，放得下（含靠要核数少的名额放行的）交空串。判法在 cpu_judgement。"""
        return self.cpu_judgement(task, host, snapshot)[0]

    def cpu_small_slot_note(self, task, host, snapshot):
        """这一条起在这台上要不要靠要核数少的名额（文件头 ②「CPU 闸」的名额）：要的交一句说明（起的时候记进尝试的 cpu_small_slot、起跑那一行照写），不要或放不下交空串。判法在 cpu_judgement。"""
        why, note = self.cpu_judgement(task, host, snapshot)
        return "" if why else note

    def cpu_small_slots_used(self, host):
        """这台此刻占着要核数少的名额的尝试条数（文件头 ②「CPU 闸」的名额）：在跑的（booking_attempts：单元已确认停了的不算）、记了 cpu_small_slot 的，
        去掉已让出名额的（small_slot_released）。照库里的尝试数，服务重启后照样认。"""
        now, machine = time.time(), self.machines.get(host)
        return sum(1 for attempt in self.booking_attempts()
                   if attempt.get("host") == host and attempt.get("cpu_small_slot") and not self.small_slot_released(attempt, machine, now))

    def small_slot_released(self, attempt, machine, now):
        """靠名额起的这一次尝试是不是已让出名额（cpu_small_slots_used）：起跑满 CPU_MEASURED_AFTER_SECONDS、量得到 CPU（machine 是那一台的 Machine）、
        峰值段核数（peak_segment_running_cores，与 running_threads 的峰值段那一支同一个取法）超过 CPU_SMALL_TASK_CORES 的让出；没满的、量不到的、服务还没量满一段的照占着。
        尝试上的 cpu_small_slot 不动。弄坏开关 cpu-small-slot-heavy-still-counted 下一律照占。"""
        if lib.broken("cpu-small-slot-heavy-still-counted", self.environment) or machine is None:
            return False
        elapsed = now - float(attempt.get("started_at") or now)
        if elapsed < CPU_MEASURED_AFTER_SECONDS:
            return False
        usage = machine.cpu_usec(attempt)
        if usage is None:
            return False
        cores = self.peak_segment_running_cores(attempt, usage, elapsed, now, int(attempt.get("threads") or 0))
        return cores is not None and cores > CPU_SMALL_TASK_CORES

    def cpu_judgement(self, task, host, snapshot):
        """CPU 闸（文件头 ②「CPU 闸」）→ (排队原因, 靠名额放行的那一句)：max(这台此刻的负载（余量报告里 1 分钟负载与可跑进程数的大者）, 服务在这台在跑的尝试的线程之和) + 这一条的线程
        超过核数 × CPU_OVERSUBSCRIPTION 的，这一条要的核数不超过 CPU_SMALL_TASK_CORES、这台靠名额在跑的不到 CPU_SMALL_TASK_SLOTS 条时放行（排队原因空、交名额那一句），
        别的交排队原因（名额满了的原因里写「CPU_SMALL_SLOT_TEXT N 个已用满」）；不超的两样都空。在跑的线程之和一直记着（不看起了多久：进程在 D 状态等 IO 的那几条负载里看不出，
        醒来照样是这么多线程），与负载取大、不相加（负载里已有在跑的那几条）。报告里读不到核数或负载的这台这一轮不派，交以 CPU_UNREAD_TEXT 起头的那一句（不走名额）。
        原因里的负载写成「负载 N（其中等 IO 的线程 M）」（报告的 blocked_threads；弄坏开关 cpu-load-io-unsplit 下不拆开写）。
        弄坏开关 cpu-gate-ignored 下不判，cpu-threads-settling-only 下只记起了不到 CPU_SETTLE_BREAK_SECONDS 秒的尝试的线程、与负载相加，cpu-unread-passed 下读不到的放行，
        cpu-small-slots-unlimited 下名额不设上限，cpu-small-slots-withheld 下不给名额，cpu-oversubscription-three 下照 CPU_OVERSUBSCRIPTION_LOOSE 倍判。
        task promote 提前过的（任务记了 promoted）不判、交两个空串；弄坏开关 promote-cpu-gate-applied 下照判。
        在跑的靠名额起的哪几条占着名额照 cpu_small_slots_used。"""
        if lib.broken("cpu-gate-ignored", self.environment):
            return "", ""
        # task promote 提前过的：内存放得下就起，不过 CPU 闸（用户定「提交任务只要是 内存允许的情况下立即执行」）；弄坏开关 promote-cpu-gate-applied 下照过 CPU 闸
        if task.get("promoted") and not lib.broken("promote-cpu-gate-applied", self.environment):
            return "", ""
        cpu = (snapshot.reports[host]["memory"] or {}).get(self.peaks.CPU_REPORT_KEY) or {}
        cpus = cpu.get("cpus")
        loads = [value for value in (cpu.get("load1"), cpu.get("procs_running")) if value is not None]
        if not cpus or not loads:
            if lib.broken("cpu-unread-passed", self.environment):
                return "", ""
            missing = "、".join(name for name, value in (("核数", cpus), ("负载", loads)) if not value)
            return f"{CPU_UNREAD_TEXT}{missing}（余量报告里的 {self.peaks.CPU_REPORT_KEY} 那一项），CPU 闸判不了，这一轮不派", ""
        wanted, wanted_how = self.admission_threads(task)
        oversubscription = CPU_OVERSUBSCRIPTION_LOOSE if lib.broken("cpu-oversubscription-three", self.environment) else CPU_OVERSUBSCRIPTION
        override = lib.selftest_override(lib.CPU_OVERSUBSCRIPTION_VARIABLE, self.environment)
        if override and not lib.broken("cpu-oversubscription-three", self.environment):
            oversubscription = int(float(override)) if float(override).is_integer() else float(override)
        load, limit = max(loads), int(cpus * oversubscription)
        blocked = cpu.get("blocked_threads")   # 此刻状态 D（多半在等 IO）的线程数：负载里含它们，原因里拆开写，闸的判法不看它
        waiting_io = f"（其中等 IO 的线程 {blocked}）" if isinstance(blocked, int) and not isinstance(blocked, bool) and not lib.broken("cpu-load-io-unsplit", self.environment) else ""
        if lib.broken("cpu-threads-settling-only", self.environment):
            now = time.time()
            settling = sum(int(attempt.get("threads") or 0) for attempt in self.store.running_attempts()
                           if attempt.get("host") == host and attempt.get("started_at") and 0 <= now - float(attempt["started_at"]) < CPU_SETTLE_BREAK_SECONDS)
            occupied, what = load + settling, f"负载 {load:.0f}{waiting_io}、{CPU_SETTLE_BREAK_SECONDS} 秒内刚起的 {settling} 线程"
        else:
            threads = self.running_threads(host, time.time())
            occupied, what = max(load, threads), f"负载 {load:.0f}{waiting_io}、服务在这台在跑的尝试共 {threads} 线程（取大者）"
        if occupied + wanted <= limit:
            return "", ""
        over = f"这台此刻{what}，再起这一条（{wanted} 线程，{wanted_how}）超过 {cpus} 核 × {oversubscription} = {limit}"
        if wanted > CPU_SMALL_TASK_CORES or lib.broken("cpu-small-slots-withheld", self.environment):
            return f"CPU：{over}，等负载降下来或在跑的跑完", ""
        used = self.cpu_small_slots_used(host)
        if used >= CPU_SMALL_TASK_SLOTS and not lib.broken("cpu-small-slots-unlimited", self.environment):
            return (f"CPU：{over}；这一条要的核数不超过 {CPU_SMALL_TASK_CORES}，而这台{CPU_SMALL_SLOT_TEXT} {CPU_SMALL_TASK_SLOTS} 个已用满，"
                    "等负载降下来、在跑的跑完或靠名额起的跑完"), ""
        return "", (f"靠{CPU_SMALL_SLOT_TEXT}起的（这台靠名额在跑的连这一条 {used + 1} 条，名额 {CPU_SMALL_TASK_SLOTS} 个）：{over}，"
                    f"要的核数不超过 {CPU_SMALL_TASK_CORES}，超线照放")

    def admission_threads(self, task):
        """CPU 闸里这一条自己要起的记几线程（文件头 ②「CPU 闸」）→ (线程数, 照什么记的那半句)：同一命令键近 RECORD_WINDOW 次跑到头的、记了 CPU 闸线程数
        （尝试收尾时记的 cpu_gate_threads）的那几次取最大、向上取整、至少 1、不超过登记数（threads_of）；一次都没有的量到之前先按登记的线程数（threads_by_record）。
        每条在排的算一次就存下（admission_threads_cache，跟内存预测一起清）。弄坏开关 cpu-wanted-by-registered-threads 下一律照登记数。"""
        registered = self.threads_of(task) or 1
        if lib.broken("cpu-wanted-by-registered-threads", self.environment):
            return registered, "照登记的线程数"
        cached = self.admission_threads_cache.get(task.get("id"))
        if cached is not None:
            return cached
        found = self.threads_by_record(task.get("command_key"), registered)
        self.admission_threads_cache[task.get("id")] = found
        return found

    def threads_by_record(self, command_key, registered):
        """CPU 闸里一条命令记几线程的取法，起之前的这一条（admission_threads）与在跑却还没量满一段的（running_threads）共用 → (线程数, 照什么记的那半句)：
        同一命令键近 RECORD_WINDOW 次跑到头的——跑成（succeeded），或命令自己退非 0 而不是被信号杀的（command-failed，退出码不在 SIGNAL_EXIT_CODES 里：弄坏开关轮这类按设计退非 0 的命令也有记录）——
        记了 CPU 闸线程数（尝试的 cpu_gate_threads：收尾时 finish_cpu_gate_threads 记的峰值段核数与可跑线程数的最大取大、不超过登记数）的那几次取最大、向上取整、至少 1、
        不超过登记数（registered）；被停的、撞上限的、被外部信号杀的不算；没记 cpu_gate_threads 的尝试（这一栏之前的旧尝试）不读。
        一次都没有、或没有命令键的量到之前先按登记的线程数（在跑的那一条量满一段之后 running_threads 照实测）。
        弄坏开关 threads-record-succeeded-only 下只认跑成的，cpu-unrecorded-by-one-core 下没有记录的先按 1 核（改前的样子），
        cpu-cores-by-average 下读尝试的平均在用核数（cores：attempt_runner 记的 cpu_usec ÷ 用时）、几次取平均。"""
        by_average = lib.broken("cpu-cores-by-average", self.environment)
        column = "cores" if by_average else CPU_GATE_THREADS_COLUMN
        cores = []
        for attempt, valid in (self.store.select("attempts", "command_key = ? AND state = 'ended'", (command_key,), order="id DESC LIMIT 200") if command_key else ()):
            used = attempt.get(column)
            ran_to_the_end = attempt.get("outcome") == "succeeded" or (attempt.get("outcome") == "command-failed" and attempt.get("exit_code") not in SIGNAL_EXIT_CODES
                                                                       and not lib.broken("threads-record-succeeded-only", self.environment))
            if valid and ran_to_the_end and isinstance(used, (int, float)) and not isinstance(used, bool) and used >= 0:
                cores.append(float(used))
                if len(cores) == RECORD_WINDOW:
                    break
        if cores:
            value, how = (sum(cores) / len(cores), "平均在用核数的平均") if by_average else (max(cores), "CPU 闸线程数（峰值段核数与可跑线程数取大）的最大")
            return min(registered, max(1, math.ceil(round(value, 3)))), f"照这条命令近 {len(cores)} 次跑到头的{how}"
        if lib.broken("cpu-unrecorded-by-one-core", self.environment):   # 改前的样子：没有记录的先按 1 核
            return min(max(1, registered), 1), "没有跑到头的核数记录，先按 1 核"
        # 没有核数记录的新命令量到之前按登记的线程数算（用户定）：按 1 核算时一把交的 38 条新命令全过了闸、十秒内齐起，每条实际是一个满作业的编译。
        # 起跑满 CPU_MEASURED_AFTER_SECONDS 之后 running_threads 改照实测；跑过一次有了记录的照记录。
        return max(1, registered), f"没有跑到头的核数记录，量到之前先按登记的 {max(1, registered)} 线程"

    def booking_attempts(self):
        """CPU 与内存的在跑记账数哪些尝试（running_threads、room_by_the_formula、beside_whole_capped、server_pending）：库里在跑的，去掉单元已确认停了、收尾还没做完的
        （unit_stopped；release_stopped_booking）。不在在跑名单里的号从 unit_stopped 里清掉。"""
        running = self.store.running_attempts()
        self.unit_stopped.intersection_update(attempt.get("id") for attempt in running)
        return [attempt for attempt in running if attempt.get("id") not in self.unit_stopped]

    def unit_confirmed_stopped(self, attempt):
        """这一次尝试的单元此刻确认停了没有：ActiveState 不在跑（unit_states）而且 cgroup.procs 读到了、是空的（unit_processes）。两样有一样读不到不算停（读不到状态决不当它停了）。"""
        if not attempt.get("unit"):
            return False
        states, processes = unit_states(attempt["unit"], self.environment), unit_processes(attempt["unit"], self.environment)
        return states is not None and states[1] not in lib.RUNNING_UNIT_STATES and processes is not None and not processes

    def release_stopped_booking(self, attempt, confirmed=None):
        """单元确认停了的这一刻起，这一次尝试不再进 CPU 与内存的在跑记账（unit_stopped、booking_attempts）；收尾（删目录、取峰值、落结局）照旧做完，到记 ended 时 finish_attempt 清掉这一条。
        confirmed 给了的（取消那一侧已回读过）照它，没给的现读（unit_confirmed_stopped）。→ 单元是不是确认停了。弄坏开关 unit-stopped-still-booked 下不放。"""
        if lib.broken("unit-stopped-still-booked", self.environment):
            return False
        if confirmed is None:
            confirmed = self.unit_confirmed_stopped(attempt)
        if confirmed and attempt["id"] not in self.unit_stopped:
            self.unit_stopped.add(attempt["id"])
            say(f"  · 尝试 {attempt.get('label')} 的单元已停（ActiveState 不在跑、cgroup.procs 空）：CPU 与内存的在跑记账放掉，收尾接着做")
        return confirmed

    def running_threads(self, host, now):
        """CPU 闸里服务在这台在跑的尝试记几线程（文件头 ②「CPU 闸」）：每一条都记一回累计 CPU 读数（latest_running_cores 里的 note_peak_segment：峰值段要两端，没满的也记），
        起跑满 CPU_MEASURED_AFTER_SECONDS、服务量满了一段的记它最近那一段的在用核数（latest_running_cores）；
        起跑已满这么久而服务还没量满一段的（服务刚重启）照它到此刻的累计 CPU ÷ 已跑秒数记（弄坏开关 cpu-restart-unmeasured-by-record 下照下一句）；起跑没满的、量不到 CPU 的照 threads_by_record（同 admission_threads 的取法）取它命令键的记录，没有命令键或没有记录的先按登记的线程数记。
        取出来的记录核数按尝试号存下（running_threads_cache），这一次尝试收尾时删（finish_attempt），不在在跑名单里的这里也删；读数表里不在库里在跑的也删（prune_running_samples）。
        弄坏开关 cpu-threads-registered-only 下一律照登记数记，cpu-running-unmeasured-registered 下没实测到的一律照登记数记（不取记录）。"""
        total = 0
        running = self.booking_attempts()
        live = {attempt.get("id") for attempt in running}
        for stale in [number for number in list(self.running_threads_cache) if number not in live]:
            self.running_threads_cache.pop(stale, None)
        self.prune_running_samples(live | set(self.unit_stopped))   # 单元已停、收尾还没做完的不进记账，读数照留（收尾记峰值段要用）
        for attempt in running:
            if attempt.get("host") != host:
                continue
            registered = int(attempt.get("threads") or 0)
            elapsed = now - float(attempt.get("started_at") or now)
            machine = self.machines.get(host)
            if lib.broken("cpu-threads-registered-only", self.environment):
                total += registered
                continue
            usage = machine.cpu_usec(attempt) if machine is not None else None
            measured = self.latest_running_cores(attempt, usage, elapsed, now, registered) if usage is not None else None
            if measured is not None and elapsed >= CPU_MEASURED_AFTER_SECONDS:
                total += measured
            elif (usage is not None and elapsed >= CPU_MEASURED_AFTER_SECONDS and registered > 0
                  and not lib.broken("cpu-restart-unmeasured-by-record", self.environment)):
                # 起跑已满两分钟、服务还没量满一段的（服务刚重启）：累计 CPU 是 cgroup 记的、重启不丢，照它 ÷ 已跑秒数记这一次到此刻的平均在用核数；
                # 不退回命令键的记录或登记数（没有记录的新命令重启后那两分钟各记 31 线程，两台的账一下涨满、排着的全被 CPU 闸挡住）。量满一段之后照最近那一段记。
                total += min(registered, max(1, math.ceil(round(usage / 1e6 / elapsed, 3))))
            elif lib.broken("cpu-running-unmeasured-registered", self.environment):
                total += registered
            else:
                total += self.unmeasured_threads(attempt, registered)
        return total

    def latest_running_cores(self, attempt, usage, elapsed, now, registered):
        """实测到 CPU 的在跑尝试（usage 是 now 的累计 CPU 微秒）此刻在 CPU 闸里记几线程：最近量满的那一段（CPU_MEASURED_AFTER_SECONDS 秒）的在用核数向上取整，与最近一回读到的可跑线程数取大，至少 1、不超过登记数。
        取的是此刻的，不是到此刻的最大：任务忙完一段闲下来，账上跟着降，排着的才起得来（记最大时两台负载 61 与 49、账上却记 85 与 78 线程，队头一直够不着线）。
        到此刻的最大照旧由 peak_segment_running_cores 记，只给收尾时的核数记录与要核数少的名额用。一段都没量满交 None。弄坏开关 cpu-latest-runnable-ignored 下不看可跑线程数。"""
        if self.peak_segment_running_cores(attempt, usage, elapsed, now, registered) is None:
            return None
        with self.running_usage_lock:
            cores = self.running_latest_cores.get(attempt.get("id"))
            runnable = self.running_latest_runnable.get(attempt.get("id"), 0)
        if cores is None:
            return None
        if lib.broken("cpu-latest-runnable-ignored", self.environment):
            runnable = 0
        return min(max(registered, 1), max(1, math.ceil(round(cores, 3)), runnable))

    def peak_segment_running_cores(self, attempt, usage, elapsed, now, registered):
        """在跑尝试的峰值段核数在 CPU 闸里记几线程（latest_running_cores 先过它，small_slot_released 也只看它）：note_peak_segment（这一次尝试到此刻各段在用核数的最大，
        每段 CPU_MEASURED_AFTER_SECONDS 秒）向上取整、至少 1、不超过登记数（registered；多于登记线程数的核数是命令里别的进程（编译器、子进程）占的，闸按登记数管）：
        前半段编译、录制用核少、后半段满核的任务按后半段记。量满了一段的另与到此刻读到过的可跑线程数的最大（running_peak_runnable，note_threads 记）取大：
        机器挤满时在用核数被压低，可跑线程数照样是它想用的那么多。→ 线程数；一段都没量满交 None。
        弄坏开关 cpu-running-cores-over-registered 下不受登记数限制，cpu-cores-by-average 下换成到此刻的平均在用核数（usage 除以已跑秒数 elapsed，不看可跑线程数）。"""
        if lib.broken("cpu-cores-by-average", self.environment):
            if elapsed <= 0:
                return None
            measured = max(1, math.ceil(usage / 1e6 / elapsed))
        else:
            peak = self.note_peak_segment(attempt, now, usage)
            if peak is None:
                return None
            with self.running_usage_lock:
                runnable = self.running_peak_runnable.get(attempt.get("id"), 0)
            measured = max(1, math.ceil(round(peak, 3)), runnable)
        return measured if lib.broken("cpu-running-cores-over-registered", self.environment) else min(max(registered, 1), measured)

    def note_threads(self, attempt):
        """每一轮读一回这一次尝试最近的线程读数（Machine.thread_reading；check_stall 调；文件头 ②「CPU 闸」的可跑线程数、①「要处置的提醒」的 threads-over-registered）：
        读不到的这一轮什么都不记。可跑线程数取大记进 running_peak_runnable（峰值段那一支）。同一个采样号（ThreadReading.sample）读过的不再算一轮：线程总数超过登记数的 THREADS_OVER_REGISTERED_FACTOR 倍、其中可跑的也超过登记数
        （只起一堆不干活线程的编译器、链接器可跑的到不了登记数）的连着 THREADS_OVER_REGISTERED_ROUNDS 轮，thread_overruns 记下判出那一轮（之后不再改：每次尝试只推一回）；
        有一轮不超就从头数。弄坏开关 cpu-runnable-threads-unread 下不记可跑线程数，threads-over-registered-idle-counted 下不看可跑的超没超。"""
        reading = self.machines[attempt["host"]].thread_reading(attempt)
        if reading is None:
            return
        attempt_id, registered = attempt.get("id"), int(attempt.get("threads") or 0)
        if not lib.broken("cpu-runnable-threads-unread", self.environment):
            with self.running_usage_lock:
                self.running_peak_runnable[attempt_id] = max(reading.runnable, self.running_peak_runnable.get(attempt_id, 0))
                self.running_latest_runnable[attempt_id] = reading.runnable
        overrun = self.thread_overruns.get(attempt_id) or {"sample": None, "streak": 0, "flagged": None}
        if overrun["flagged"] is not None or reading.sample == overrun["sample"]:
            return
        # 占线程最多的那一个进程自己也要超过登记数的两倍：编译时 cargo 起的一堆 rustc 各开几个线程、加起来多，没有哪一个进程没读线程上限
        # （任务 15895 编译那一段共 392 个线程、可跑 39 个、占线程最多的 cargo 只有 34 个，被报过一回）
        over = (registered > 0 and reading.total > THREADS_OVER_REGISTERED_FACTOR * registered
                and (reading.runnable > registered or lib.broken("threads-over-registered-idle-counted", self.environment))
                and (reading.top_threads > THREADS_OVER_REGISTERED_FACTOR * registered or lib.broken("threads-over-registered-many-processes-counted", self.environment)))
        streak = overrun["streak"] + 1 if over else 0
        self.thread_overruns[attempt_id] = {"sample": reading.sample, "streak": streak,
                                            "flagged": (reading, registered) if streak >= THREADS_OVER_REGISTERED_ROUNDS else None}

    def finish_cpu_gate_threads(self, attempt, peak_segment):
        """收尾时这一次尝试记的 CPU 闸线程数（尝试的 cpu_gate_threads，threads_by_record 读；文件头 ②「CPU 闸」）：
        min(登记数, max(收尾记的峰值段核数 peak_segment, 到此刻读到过的可跑线程数的最大 running_peak_runnable))，保留三位小数；peak_segment 是 None 的交 None（这一次不进记录）。"""
        if peak_segment is None:
            return None
        with self.running_usage_lock:
            runnable = self.running_peak_runnable.get(attempt.get("id"), 0)
        return round(min(max(int(attempt.get("threads") or 0), 1), max(peak_segment, runnable)), 3)

    def threads_over_registered_items(self, record, attempts):
        """线程总数远超登记数（文件头 ①「要处置的提醒」的 threads-over-registered）：在跑的那一次尝试 note_threads 判出过的交一项，提醒门槛是尝试的次数（每次尝试只推一回），
        不然交空表。只提醒、照跑、不停它。弄坏开关 threads-over-registered-silent 下不报。"""
        if lib.broken("threads-over-registered-silent", self.environment):
            return []
        active = [attempt for attempt in attempts if attempt.get("state") in ACTIVE_ATTEMPT_STATES]
        flagged = (self.thread_overruns.get(active[-1]["id"]) or {}).get("flagged") if active else None
        if flagged is None:
            return []
        (reading, registered), attempt, task_id = flagged, active[-1], record["id"]
        return [self.attention_item(
            "threads-over-registered", max(1, int(attempt.get("number") or 1)), task_id,
            f"第 {attempt.get('number')} 次尝试在{self.machines[attempt['host']].label}连着 {THREADS_OVER_REGISTERED_ROUNDS} 轮线程总数超过登记数 {registered} 的 "
            f"{THREADS_OVER_REGISTERED_FACTOR} 倍：判出那一轮共 {reading.total} 个线程、其中可跑的 {reading.runnable} 个，占线程最多的进程是 {reading.top_name}（{reading.top_threads} 个线程）",
            "开的线程远多于登记数：几条一起跑时挤满整台的核，CPU 闸照登记数放进来的别的任务都被拖慢",
            f"服务照跑、不停它；照跑就再起 task-server/task wait {task_id}")]

    def note_peak_segment(self, attempt, now, usage):
        """CPU 闸记峰值段核数用的读数（文件头 ②「CPU 闸」）：往 running_peak_samples 里这一次尝试名下记一回 now 时的累计 CPU 微秒 usage（note_running_sample：
        窗口 CPU_MEASURED_AFTER_SECONDS 秒、留起点那一端），窗口两端相隔满 CPU_MEASURED_AFTER_SECONDS 时算这一段的在用核数（两端累计读数之差 ÷ 相隔秒数），
        与这一次尝试此前各段的取大存进 running_peak_cores。→ 到此刻各段在用核数的最大值（小数）；一段都没量满交 None（刚起、服务刚重启）。
        服务每一轮在 check_stall 里、算账时（latest_running_cores）各记一回；收尾时 finish_peak_segment_cores 取、finish_attempt 删。"""
        attempt_id = attempt.get("id")
        _, samples = self.note_running_sample(self.running_peak_samples, attempt_id, now, usage, CPU_MEASURED_AFTER_SECONDS, earlier_reading)
        with self.running_usage_lock:
            if len(samples) >= 2 and samples[-1][0] - samples[0][0] >= CPU_MEASURED_AFTER_SECONDS and samples[-1][1] >= samples[0][1]:
                cores = (samples[-1][1] - samples[0][1]) / 1e6 / (samples[-1][0] - samples[0][0])
                self.running_peak_cores[attempt_id] = max(cores, self.running_peak_cores.get(attempt_id, 0.0))
                self.running_latest_cores[attempt_id] = cores
            return self.running_peak_cores.get(attempt_id)

    def finish_peak_segment_cores(self, attempt, machine, now):
        """收尾时这一次尝试记的峰值段核数（尝试的 peak_segment_cores，threads_by_record 读）：收尾这一刻再读一回累计 CPU（machine 是那一台的 Machine），
        连同服务一路记的读数（note_peak_segment）取各段在用核数的最大；一段都没量满的（跑不满 CPU_MEASURED_AFTER_SECONDS，或服务重启后还没量满一段）照这一刻的累计读数除以已跑秒数：整次就是一段。
        这一刻读不到、或接口交的不是数（cpu_reading）的照读不到办：量满过一段的交到此刻各段的最大，没量满过的交 None（这一次不进核数记录），收尾照常往下走。
        保留三位小数。读数表不在这里删（清理没确认、下一轮重收尾时照样取得到），finish_attempt 记 ended 时删。
        弄坏开关 peak-segment-reading-unchecked 下接口交的东西不核、照数用。"""
        usage = machine.cpu_usec(attempt)
        if not lib.broken("peak-segment-reading-unchecked", self.environment):
            usage = cpu_reading(usage)
        if usage is not None:
            peak = self.note_peak_segment(attempt, now, usage)
        else:
            with self.running_usage_lock:
                peak = self.running_peak_cores.get(attempt.get("id"))
        if peak is None and usage is not None:
            elapsed = now - float(attempt.get("started_at") or now)
            peak = usage / 1e6 / elapsed if elapsed > 0 else None
        return None if peak is None else round(peak, 3)

    def unmeasured_threads(self, attempt, registered):
        """一条在跑、还没实测到核数的尝试记几线程：照它命令键的记录（threads_by_record），按尝试号存下第一次取的（running_threads_cache）。"""
        found = self.running_threads_cache.get(attempt.get("id"))
        if found is None:
            found = self.threads_by_record(attempt.get("command_key"), registered)
            self.running_threads_cache[attempt.get("id")] = found
        return found[0]

    def host_total(self, host, snapshot):
        """这台的总上限（字节）：Snapshot.total（slice 的总上限，另有收到的数取小者）。"""
        return snapshot.total(host)

    # ── ③ 起 ──
    def threads_of(self, task):
        """这一件的线程数（task_server_lib.py 文件头「线程」）：一律是登记的每台默认重活线程数（用户：「每次都开31」）。"""
        return self.threads

    def attempt_command(self, task, placement, label, environment, build):
        """这一次尝试在单元里起的那一条（文件头 ③「起」）：两台同一段——命令外面套 capped.sh <线程数>，再交那一台的起法（Machine.command：本机是内存包装，
        第二台是 multi-host-run.sh --peer；build 是这一次的编译目录，没给是 None）。外面不套清场：任务不拿卡（卡归 GPU 批处理验证服务）。"""
        threads = [] if lib.broken("threads-unset", self.environment) else [str(self.threads_of(task))]
        capped = ["bash", lib.THREAD_WRAPPER, *threads] if threads else []
        return self.machines[placement.host].command(task, placement, label, environment, capped, build)

    def shard_plan_directory(self, task):
        return os.path.join(self.shard_directory(task["parent_id"]), "plan")

    def shard_host_address(self, host):
        """分片的一片所在那一台在 HOSTS 里的地址（task_server_lib.py 文件头「分片」②的 TASK_SERVER_SHARD_HOST）：→ (地址或空串, 认不出的原因)。
        服务照 Resources.addresses 认出自己这一台：派本机（local）的是这一台的地址，派第二台（peer）的是另一台的地址。"""
        addresses, problem = self.resources.addresses()
        if addresses is None:
            return "", (f"分片要把这一片所在那一台在多机配置 {lib.HOSTS_KEY} 里的地址交给命令（{lib.SHARD_HOST_VARIABLE}），此刻认不出 {lib.HOSTS_KEY} 里哪一项是"
                        f"服务所在这一台：{problem}")
        return self.machines[host].address(addresses), ""

    def shard_environment(self, task, placement, label, number, environment, address):
        """分片那几步的环境（task_server_lib.py 文件头「分片」）：→ 这一片的产物目录（共享工作根下，两台同一个绝对路径）。
        address 是这一片所在那一台在 HOSTS 里的地址（shard_host_address）；弄坏开关 shard-host-role-passed 下照改前给 local / peer。"""
        role = task["role"]
        directory = self.shard_directory(task["parent_id"])
        lib.ensure_private_directory(directory)
        plan = self.shard_plan_directory(task)
        lib.ensure_private_directory(plan)
        environment[lib.SHARD_PHASE_VARIABLE] = role
        environment[lib.SHARD_PLAN_VARIABLE] = plan
        if role == "merge":
            environment[lib.SHARD_OUTPUTS_VARIABLE] = os.path.join(directory, "outputs.tsv")
            found = self.store.task(task["parent_id"])
            if found is not None and found[1] and not lib.broken("shard-missing-unnamed", self.environment):   # 父任务校验不过的 claim_attempt 拒、这一次不起
                environment[lib.SHARD_MISSING_VARIABLE] = ",".join(str(index) for index in found[0].get("missing_shards") or [])
            return ""
        if role == "plan":
            environment[lib.SHARD_MAXIMUM_VARIABLE] = str(lib.MAXIMUM_SHARDS)
            return ""
        environment[lib.SHARD_VARIABLE] = f"{task['shard_index']}/{task['shard_count']}"
        environment[lib.SHARD_HOST_VARIABLE] = placement.host if lib.broken("shard-host-role-passed", self.environment) else address
        output = os.path.join(directory, f"output-{task['shard_index']}-attempt-{number}")
        lib.ensure_private_directory(output)
        environment[lib.SHARD_OUTPUT_VARIABLE] = output
        return output

    def start_attempt(self, task, placement):
        """起这一次尝试，交它的尝试记录；没起（判放得下之后被取消了、父任务已结了局、状态变了）交 None（文件头 ③「起」）。
        先照现读的任务（不拿调用方手里的旧字典）备好目录、环境与说明，再拿这条任务的锁：claim_attempt 一个事务里判 queued 且没取消、分尝试号、
        插尝试、改任务（TS-01、TS-03），之后起单元，起完才放锁——取消在起之前到的不起，在起之后到的等单元起完再停、回读。
        弄坏开关 cancel-race-unguarded 下照改前：开头判一次、不拿锁、先插尝试再拿调用方那份旧字典整行写回任务。"""
        unguarded = lib.broken("cancel-race-unguarded", self.environment)
        current = self.store.task(task["id"])
        if current is None or not current[1] or current[0].get("state") != "queued" or current[0].get("cancel_requested"):
            if current is not None and current[1] and current[0].get("state") == "queued" and current[0].get("cancel_requested"):
                self.finalize(current[0], "cancelled", None, "判放得下之后、起之前被取消了，没起")
            return None   # 这一轮判放得下之后、起之前被取消了（接口那一侧当场收尾）：不起
        if not unguarded:
            task = current[0]
        shard_address = ""
        if task.get("role") == "shard":   # 这一片所在那一台的地址（文件头「分片」②）：place 判过认得出，这里现取；配置在这一瞬之间改坏了的照 run-condition-unmet 收尾、不起
            shard_address, problem = self.shard_host_address(placement.host)
            if not shard_address:
                self.finalize(task, "run-condition-unmet", None, problem)
                return None
        number = int(task.get("attempt_count", 0)) + 1
        if number > 1 and lib.broken("retry-overwrites-attempts", self.environment):
            for earlier, _ in self.store.attempts_of(task["id"]):   # 弄坏开关：重试这一次覆盖前几次的记录
                self.store.delete_attempt(earlier["id"])
        label = f"task-{self.store.tag}-{task['id']}-{number}"
        directory = os.path.join(self.configuration.attempts_directory, label)
        for path in (self.configuration.logs_directory, self.configuration.attempts_directory, directory):
            lib.ensure_private_directory(path)
        log = os.path.join(self.configuration.logs_directory, f"task-{self.store.tag}-{task['id']}.attempt-{number}.log")
        environment = {name: value for name, value in self.environment.items() if not name.startswith("TASK_SERVER_")}
        environment.update(task["environment"])
        environment.update(TASK_SERVER_TASK=str(task["id"]), TASK_SERVER_ATTEMPT=str(number))
        peer_cleanup_file = os.path.join(directory, "peer-cleanup.json") if placement.host == PEER else ""
        if peer_cleanup_file:
            write_json_atomically(peer_cleanup_file, {"status": "not-started", "label": label})
            environment["TASK_SERVER_PEER_CLEANUP_FILE"] = peer_cleanup_file
        if not lib.broken("memory-cap-not-passed", self.environment):   # 这一次的内存上限（字节，十进制）交给命令（文件头 ③「环境」）
            environment[lib.MEMORY_CAP_VARIABLE] = str(int(placement.cap))
        if not lib.broken("source-repository-not-passed", self.environment):   # 提交方那一份仓的根交给命令（文件头 ③「环境」）：第二台照它拼两台一致的路径
            environment[lib.SOURCE_REPOSITORY_VARIABLE] = os.path.realpath(self.peer_root)
        if not lib.broken("shared-roots-not-passed", self.environment):   # 两个共享根交给命令（文件头 ③「环境」）：第二台原样带本机的值，不经 ~ 取
            environment.update(self.attempt_shared_roots(task))
        machine = self.machines[placement.host]
        for each in self.machines.values():
            environment.pop(each.admission_variable, None)
        if not (machine.admission_break and lib.broken(machine.admission_break, self.environment)):   # 那一台最外层那一层只收服务直接起的这一次（文件头 ③「起」）
            environment[machine.admission_variable] = f"{task['id']}.{number}"
        if not lib.broken("attempt-label-not-passed", self.environment):   # 两台的 scope 照它对到这一次尝试上（server_pending、memory_peaks_lib.py 文件头「在跑的 scope」）
            environment[lib.ATTEMPT_LABEL_VARIABLE] = label
        environment[lib.THREAD_ADMITTED_VARIABLE] = f"{task['id']}.{number}"   # capped.sh 只认服务套的这一层（文件头 ③「起」）
        # 任务不拿卡（卡归 GPU 批处理验证服务，.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：服务自己环境里带着的选卡变量（lib.CARD_CHOICE_VARIABLES）
        # 不交下去，进程见到它会当成分给自己的卡；任务带的那几个提交时就拒了。弄坏开关 card-variables-passed-to-attempt 下照交
        if not lib.broken("card-variables-passed-to-attempt", self.environment):
            for name in lib.CARD_CHOICE_VARIABLES:
                environment.pop(name, None)
        if task.get("session"):
            environment[lib.SESSION_VARIABLE] = task["session"]
        if lib.in_selftest(self.environment):
            for name in (lib.SELFTEST_MARK_VARIABLE, "TASK_SERVER_SELFTEST_WORLD"):
                environment[name] = self.environment.get(name, "")
        # 给内存包装的两样（文件头 ③「环境」；包装读了就 unset，第二台的由 multi-host-run.sh 设给那边那一层）：退出时峰值写进尝试目录（task_server_lib.py 文件头
        # 「资源记录」的退出时峰值，弄坏开关 peak-read-only-by-sampling 下不给），现场目录在共享工作根下（同一份文件头「现场」）
        exit_peak_file = "" if lib.broken("peak-read-only-by-sampling", self.environment) else os.path.join(directory, lib.EXIT_PEAK_FILE_NAME)
        if exit_peak_file:
            environment[lib.PEAK_OUTPUT_VARIABLE] = exit_peak_file
        scene_directory = os.path.join(self.scenes_root, label)
        environment[lib.SCENE_VARIABLE] = scene_directory
        build_directory, made_here, reused = "", True, False
        if (task.get("compiles") and not any(task["environment"].get(name) for name in lib.TARGET_DIRECTORY_VARIABLES)
                and not lib.broken("target-directory-unset", self.environment)):
            # 文件头「编译目录」：两台同一段，照那一台的接口（Machine.build_directory）给这一次尝试自己的编译目录；服务在本机建的任务记下结局时删（设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的留给 task clean），
            # 那一台的起法自己建的（multi-host-run.sh 在第二台本地盘上 builds/<标签>/）跑完由它删，服务只记下在哪
            build_directory, made_here = machine.build_directory(label)
            kept = self.kept_build_directory(task, number)
            reusable = made_here or not lib.broken("peer-retry-build-directory-fresh", self.environment)   # 弄坏开关：照改前第二台那一份不接着用
            if kept is not None and reusable and kept.get("host") == placement.host and not lib.broken("retry-build-directory-fresh", self.environment):
                # 重排的这一次与上一次在同一台、上一次的编译目录还留着：接着用那一份（task_server_lib.py 文件头「编译目录」的重试复用；两台同一段：
                # 本机经 CARGO_TARGET_DIR 指过去，第二台经 multi-host-run.sh --reuse-build-directory 点名那一份，PeerMachine.command）
                build_directory, reused = kept["build_directory"], True
            elif kept is not None and not lib.broken("retry-build-directory-kept", self.environment):
                # 换了台：上一次留着的那一份这一次用不上，当场删（有进程开着里面的文件就不删、写告警）
                self.remove_build_directory(task, kept, None)
                self.store.update("attempts", kept)
            if made_here and build_directory:
                os.makedirs(build_directory, mode=0o700, exist_ok=True)
                environment["CARGO_TARGET_DIR"] = os.path.join(build_directory, "target")
        shard_output = self.shard_environment(task, placement, label, number, environment, shard_address) if task.get("role") in lib.SHARD_ROLES else ""
        argv = self.attempt_command(task, placement, label, environment, BuildDirectory(build_directory, reused) if build_directory else None)
        specification = os.path.join(directory, "spec.json")
        detached = not lib.broken("no-detach", self.environment)
        unit = f"singlefs-task-{self.store.tag}-{task['id']}-{number}.service" if detached else ""
        write_json_atomically(specification, {"unit": unit,
            "argv": argv, "cwd": task["cwd"], "environment": environment, "log": log, "directory": directory, "host": placement.host,
            "fast": lib.fast(self.environment),
            "fake_scope_directory": lib.selftest_override("TASK_SERVER_FAKE_SCOPE_DIRECTORY", self.environment),
            "build_directory": build_directory if made_here else "", "wake_socket": self.configuration.socket,
            "wake_skipped": lib.broken("runner-wake-skipped", self.environment), "leftovers_kept": lib.broken("leftover-processes-kept", self.environment),
            "leftovers_unit_unchecked": lib.broken("leftovers-outside-own-unit", self.environment), "exit_peak_file": exit_peak_file,
            "leftovers_error_uncaught": lib.broken("leftovers-error-uncaught", self.environment),
            "start_error_uncaught": lib.broken("runner-start-error-uncaught", self.environment),
            "final_sample_skipped": lib.broken("runner-final-sample-skipped", self.environment),
            "memory_peak_file": os.path.join(directory, lib.MEMORY_PEAK_FILE_NAME),
            "memory_peak_with_page_cache": lib.broken("memory-peak-with-page-cache", self.environment),
            "memory_stat_at_exit": lib.broken("peak-memory-stat-read-at-exit", self.environment),
            "threads_state_unread": lib.broken("runner-threads-state-unread", self.environment),
            "exit_peak_only_when_unsampled": lib.broken("exit-peak-only-when-unsampled", self.environment)})
        attempt = {"task_id": task["id"], "number": number, "host": placement.host, "memory_cap": placement.cap, "memory_cap_text": lib.mebibytes_text(placement.cap),
                   "booked": placement.booked, "disk_booked": placement.disk_booked,
                   "prediction": placement.history,
                   "cpu_small_slot": placement.cpu_small_slot, "label": label, "peer_cleanup_file": peer_cleanup_file,
                   "unit": unit, "log": log, "directory": directory,
                   "spec": specification, "state": "starting", "session": "", "command_key": task["command_key"], "command": task.get("command"), "started_at": round(time.time(), 3),
                   "threads": self.threads_of(task) if not lib.broken("threads-unset", self.environment) else None, "build_directory": build_directory,
                   "build_directory_removed": bool(build_directory) and not made_here, "build_directory_reused": reused,
                   "shard_output": shard_output, "scene_directory": scene_directory,
                   "role": task.get("role") or "",   # 分步的角色（计划、片、合并；普通任务空串）
                   "shard_count": task.get("shard_count")}   # 片：这一趟共几片（attempt_shard_count；定额度只认片数相同的记录，record_fits_task）
        self.selftest_pause("start-checked")
        if unguarded:
            attempt = self.store.insert("attempts", attempt)
            task.update(state="running", attempt_count=number)
            self.store.update("tasks", task, force=True)
            self.crash_point("before-unit")
            append_log_lines(log, self.submit_warning_lines(task))
            self.launch(task, attempt)
            return attempt
        with self.store.task_lock(task["id"]):
            try:
                attempt, task = self.store.claim_attempt(task["id"], number, attempt, {"state": "running"}, between=lambda: self.crash_point("attempt-claimed"),
                                                         split=lib.broken("attempt-claim-split", self.environment))
            except StoreRefused as refused:
                shutil.rmtree(directory, ignore_errors=True)   # 备好的这一次没起：它自己的尝试目录与编译目录删掉
                if build_directory and made_here and not reused:   # 接着用的上一次那一份不在这里删：任务有了结局时照常删
                    shutil.rmtree(build_directory, ignore_errors=True)
                found = self.store.task(task["id"])
                if refused.kind == "tampered" and found is not None:
                    self.tampered(task["id"], found[0], "任务记录")
                elif refused.kind in ("parent-finished", "parent-cancelling") and found is not None and found[1]:
                    self.finalize(found[0], "cancelled", None, f"{refused}：没起", cancel_origin="parent")   # 服务撤的（理由行照 cancel_origin 写，lib.reason_fields）
                elif refused.kind == "cancelled" and found is not None and found[1]:
                    self.finalize(found[0], "cancelled", None, f"{refused}：没起")
                else:
                    say(f"  · 任务 {task['id']} 这一次不起：{refused}")
                return None
            self.crash_point("before-unit")
            self.selftest_pause("claimed")
            append_log_lines(log, self.submit_warning_lines(task))   # 提交时警告进日志头（文件头 ②「提交时警告」）：起单元之前写，命令的输出接在后面
            self.launch(task, attempt)
        return attempt

    def crash_point(self, name):
        """只在自证里：TASK_SERVER_SELFTEST_CRASH_POINT=<点>:<文件> 那份文件在时，服务在这一点当场退出（文件头 ③「只起一次」与分片恢复那几格用）。"""
        setting = lib.selftest_override("TASK_SERVER_SELFTEST_CRASH_POINT", self.environment)
        point, _, arm = setting.partition(":")
        if point == name and arm and os.path.exists(arm):
            os.remove(arm)
            say(f"  · 自证造的崩溃：{name}")
            os._exit(70)

    def selftest_pause(self, name):
        """只在自证里：TASK_SERVER_SELFTEST_PAUSE_POINT=<点>:<目录>，目录里 arm-<点> 在时删掉它、建 reached-<点>，等 release-<点> 在了才往下走
        （至多 SELFTEST_PAUSE_SECONDS 秒；精确卡住起单元与取消交错的那几格用）。"""
        setting = lib.selftest_override("TASK_SERVER_SELFTEST_PAUSE_POINT", self.environment)
        point, _, directory = setting.partition(":")
        if point != name or not directory or not os.path.exists(os.path.join(directory, f"arm-{name}")):
            return
        os.remove(os.path.join(directory, f"arm-{name}"))
        with open(os.path.join(directory, f"reached-{name}"), "w", encoding="utf-8"):
            pass
        deadline = time.monotonic() + SELFTEST_PAUSE_SECONDS
        while time.monotonic() < deadline and not os.path.exists(os.path.join(directory, f"release-{name}")):
            time.sleep(0.05)

    def launch(self, task, attempt):
        if not os.path.isdir(task["cwd"]):
            started, why = False, f"工作目录 {task['cwd']} 不在"
        else:
            started, why = self.start_unit(task, attempt)
        if started is None:   # 回执超时：起没起不确定（TS-06），照单元名读回再定，判不了留在 starting、账照记
            attempt.update(start_uncertain_at=round(time.time(), 3), note=why)
            readback = self.start_readback(attempt)
            if readback == "started":
                attempt["state"] = "running"
                self.store.update("attempts", attempt)
                say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试：{why}；照单元名读回它起了，照常跑")
                return
            self.store.update("attempts", attempt)
            self.alert("start-uncertain", task["id"], f"任务 {task['id']} 第 {attempt['number']} 次尝试：{why}；照单元名 {attempt.get('unit')} 读回 {readback}，"
                       "起没起还判不了：尝试留在 starting、账照记",
                       f"服务每一轮对账照单元名读回：起了就照常跑，{self.start_uncertain_grace_seconds:.0f} 秒里单元一直不在、起跑标记也没有才记 unit-start-failed；"
                       "要停它照常 task-server/task cancel")
            return
        if not started and not lib.broken("start-failure-ignored", self.environment):
            attempt.update(state="ended", outcome="unit-start-failed", ended_at=round(time.time(), 3), note=why)
            self.store.update("attempts", attempt)
            self.finalize(task, "unit-start-failed", None, why)
            return
        self.crash_point("after-unit")
        attempt["state"] = "running"
        self.store.update("attempts", attempt)
        say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试起了：{lib.HOST_NAMES.get(attempt['host'], attempt['host'])}、上限 {attempt['memory_cap_text']}"
            f"、单元 {attempt['unit'] or '（服务的子进程）'}、日志 {attempt['log']}"
            f"{'；' + attempt['cpu_small_slot'] if attempt.get('cpu_small_slot') else ''}")

    def start_readback(self, attempt):
        """起单元的回执超时之后照预先定好的单元名读回（TS-06）：→ started（单元在跑，或起跑标记、结果已在）、not-found（单元不在、标记也没有）、
        或一句读不到的原因。"""
        result_path, started_path, _ = self.attempt_paths(attempt)
        if os.path.exists(started_path) or os.path.exists(result_path):
            return "started"
        states = unit_states(attempt.get("unit", ""), self.environment)
        if states is None:
            return "读不到（systemctl --user show 起不来或超时）"
        if states[1] in lib.RUNNING_UNIT_STATES:
            return "started"
        return "not-found" if states[0] == "not-found" else f"LoadState={states[0]}、ActiveState={states[1]}"

    def nested_outer_unit(self):
        """套在外层单元里的服务（文件头 ③「套在外层单元里的服务」）：在自证里、或自己的环境里带着尝试标签的，交它所在的用户级单元名（/proc/self/cgroup 照 lib.enclosing_user_unit 认），
        起尝试单元照它 BindsTo、After；真服务交空串。认不出的交空串、打一句。弄坏开关 nested-units-unbound 下交空串。"""
        if not (lib.in_selftest(self.environment) or self.environment.get(lib.ATTEMPT_LABEL_VARIABLE)) or lib.broken("nested-units-unbound", self.environment):
            return ""
        try:
            with open(SELF_CGROUP_PATH, encoding="utf-8") as handle:
                unit = lib.enclosing_user_unit(handle.read())
        except OSError:
            unit = ""
        if not unit:
            say(f"  ! 这一份服务套在自证或一次尝试里跑，而 {SELF_CGROUP_PATH} 里认不出它所在的用户级单元（不在 user@<uid>.service 之下）：尝试单元不随外层停")
        return unit

    def start_unit(self, task, attempt):
        """起尝试的单元：→ (True 起了 / False 确定没起 / None 回执超时、起没起不确定, 说明)。超时只算不确定（TS-06：systemd-run 是发给用户级 systemd 的请求，
        调用端超时不说明管理器没收下），弄坏开关 start-timeout-as-failure 下照改前当没起。"""
        runner = [sys.executable, "-B", lib.RUNNER_SCRIPT, attempt["spec"]]
        if not attempt.get("unit"):   # 弄坏开关 no-detach：起成服务自己的子进程
            child = subprocess.Popen(runner, cwd=task["cwd"], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            self.children.append(child)
            attempt["child_pid"] = child.pid
            return True, ""
        bound = ["-p", f"BindsTo={self.outer_unit}", "-p", f"After={self.outer_unit}"] if self.outer_unit else []   # 外层单元一停这一个跟着停（文件头 ③「套在外层单元里的服务」）
        command = [self.systemd_run, "--user", f"--unit={attempt['unit']}", "--collect", "--quiet", "-p", "KillMode=mixed",
                   "-p", f"TimeoutStopSec={UNIT_STOP_SECONDS}", *([] if lib.broken("nofile-limit-dropped", self.environment) else ["-p", nofile_property()]), *bound,
                   f"--working-directory={task['cwd']}", "--", *runner]
        timeout = float(lib.selftest_override("TASK_SERVER_SELFTEST_START_TIMEOUT_SECONDS", self.environment) or START_TIMEOUT_SECONDS)
        try:
            completed = subprocess.run(command, capture_output=True, text=True, env=self.environment, stdin=subprocess.DEVNULL, timeout=timeout)
        except subprocess.TimeoutExpired as error:
            if lib.broken("start-timeout-as-failure", self.environment):
                return False, f"systemd-run 起不来（{type(error).__name__}: {error}）"
            return None, f"systemd-run {timeout:g} 秒没回执（起没起不确定）"
        except OSError as error:
            return False, f"systemd-run 起不来（{type(error).__name__}: {error}）"
        if completed.returncode != 0:
            lines = [line for line in (completed.stderr + completed.stdout).splitlines() if line.strip()]
            return False, f"systemd-run 退 {completed.returncode}：{lines[-1].strip() if lines else '没有输出'}"
        return True, ""

    def compiles(self, task):
        """命令里命令位置上有会编的 cargo（task_server_lib.py 文件头「编译目录」；认法是 .claude/hooks/lib_workspace_cargo_lock.py 的 cargo_invocations，只 import 不改），
        或是不带 --server-binary 的临时 prune-server 包装（包装自己在编译目录里现编 prune-server；lib.temporary_service_builds_server，文件头「资源记录」命令键那一段后）：
        外层这一条算会编的，派到哪台服务都给编译目录（不带它，本机不给编译目录、包装退 2，第二台却由 multi-host-run.sh 照例给）；
        命令位置上起的入口脚本文件头声明了「# task-server-compiles: yes」的同样算（compiles_declared：门禁入口里面才起 cargo，命令那一层认不出）。"""
        if lib.temporary_service_builds_server(task["command"], task.get("environment"), self.environment):
            return True
        if self.compiles_declared(task)[0]:
            return True
        try:
            runs = self.cargo_lock.cargo_invocations(shlex.join(task["command"]), task["cwd"], self.project_code_runs.shell_words, self.project_code_runs, dict(task["environment"]))
            return any(run.subcommand in self.cargo_lock.BUILDING_SUBCOMMANDS for run in runs)
        except Exception as error:   # 切词库出事：只看命令本身那一层，宁可多给一个编译目录
            say(f"  ! 任务的命令切词出了事（{type(error).__name__}: {error}），会不会编只看命令本身那一层")
            index = lib.command_word_index(task["command"])
            words = task["command"][index:]
            return bool(words) and os.path.basename(words[0]) == "cargo" and any(word in self.cargo_lock.BUILDING_SUBCOMMANDS for word in words[1:3])

    def cargo_runs(self, task):
        """命令里命令位置上起的每一条 cargo（认法同 compiles）；切词库出事交空表（照没有 cargo 办，不挡提交）。"""
        try:
            return self.cargo_lock.cargo_invocations(shlex.join(task["command"]), task["cwd"], self.project_code_runs.shell_words, self.project_code_runs,
                                                     dict(task["environment"]))
        except Exception as error:
            say(f"  ! 任务的命令切词出了事（{type(error).__name__}: {error}），照没有 cargo 办")
            return []

    def repository_target_bypass(self, task):
        """提交时警告（文件头 ②「提交时警告」）：这一条取仓根 target/ 下的东西的那几处说明（lib.repository_target_references），没有交 []。
        仓根是服务的仓根（自证里是假仓根）与 lib.repository_roots 那几棵树。判法出事不挡提交（这一条只警告），写进服务日志、交 []。
        弄坏开关 repository-target-bypass-unwarned 下交 []。"""
        if lib.broken("repository-target-bypass-unwarned", self.environment):
            return []
        roots = list(dict.fromkeys(os.path.realpath(root) for root in [self.peer_root, *lib.repository_roots()]))
        try:
            return lib.repository_target_references(task["command"], task["cwd"], roots, task.get("environment") or {})
        except Exception as error:
            say(f"  ! 判命令取不取仓根 target/ 时出了事（{type(error).__name__}: {error}），这一条不警告")
            return []

    def submit_warning_lines(self, task):
        """提交时警告的几行（文件头 ②「提交时警告」）：绕过编译目录的两行（bypass_warning_lines）接交任务时会话名下有结局满时限没人取的两行（unclaimed_warning_lines）；
        分片的子任务照父任务的。没有交 []。"""
        owner = task
        if task.get("parent_id"):
            parent = self.store.task(task["parent_id"])
            if parent is not None and parent[1]:
                owner = parent[0]
        return self.flag_lines(owner) + self.bypass_warning_lines(owner) + self.unclaimed_warning_lines(owner)

    def unclaimed_before(self, session, now):
        """交任务这一刻，这个会话名下结局满 unclaimed_warning_seconds 还没交给它的任务号（文件头 ②「提交时警告」②；消息没送到、也没起 wait；子任务不算，人手的空会话不判）。
        弄坏开关 unclaimed-unwarned 下交 []。"""
        if not session or lib.broken("unclaimed-unwarned", self.environment):
            return []
        return [task["id"] for task, valid in self.store.select("tasks", "session = ? AND state = 'finished'", (session,))
                if valid and not task.get("parent_id") and not task.get("collected") and now - float(task.get("finished_at") or now) >= self.unclaimed_warning_seconds]

    def unclaimed_warning_lines(self, task):
        """交任务时会话名下有结局满时限没人取的两行（文件头 ②「提交时警告」②）：照任务记的 unclaimed_at_submit 与会话号、任务号现拼；没有交 []。"""
        found = task.get("unclaimed_at_submit") or []
        if not found:
            return []
        shown = found[:lib.BYPASS_HITS_SHOWN]
        more = f"…另有 {len(found) - len(shown)} 条" if len(found) > len(shown) else ""
        return [f"✗ 严厉警告：会话 {task.get('session')} 有 {len(found)} 条任务跑完了没人取（{'、'.join(str(number) for number in shown)}{more}）："
                f"结局满 {self.unclaimed_warning_seconds / 60:.0f} 分钟没交给提交它的会话（服务发的消息没送到，也没起 task wait）；这一回交的任务 {task['id']} 照收照跑",
                "→ 怎么办：task-server/task wait <号> 取走（" + "、".join(f"task-server/task wait {number}" for number in shown) + "），照结局办完再 task-server/task clean <号>"]

    def bypass_warning_lines(self, owner):
        """绕过编译目录的两行（文件头 ②「提交时警告」①）：照 owner（分片的子任务由 submit_warning_lines 换成父任务）记的 target_bypass 与会话号、任务号现拼。没有交 []。"""
        found = owner.get("target_bypass") or []
        if not found:
            return []
        shown = "；".join(found[:lib.BYPASS_HITS_SHOWN]) + (f"；另有 {len(found) - lib.BYPASS_HITS_SHOWN} 处" if len(found) > lib.BYPASS_HITS_SHOWN else "")
        return [f"✗ 严厉警告：会话 {owner.get('session') or '无（人手）'} 交的任务 {owner['id']} 绕过了服务给的编译目录，取仓根 target/ 下的二进制（{shown}）："
                "仓根 target/ 下是早先编的那一份，可能是合并前的旧版本，别的会话的 cargo 随时会换掉它；服务照收照跑，跑出来的结果不算这一次的代码",
                f"→ 怎么办：编译交给服务给的编译目录，二进制取这一趟编出来的位置（会编的命令拿到的 CARGO_TARGET_DIR 下，或 cargo build --message-format=json 报的 executable）；"
                f"入口脚本里不写仓根 target/ 的路径，没给 CARGO_TARGET_DIR 时也不退回仓根 target/；改完 task-server/task cancel {owner['id']}、重交"]

    def command_script(self, task):
        """命令位置上起的那个文件（command_word_index 剥过包装之后那一个词，照工作目录解成真路径）；没有交空串。"""
        words = task["command"]
        index = lib.command_word_index(words)
        return os.path.realpath(os.path.join(task["cwd"], words[index])) if index < len(words) else ""

    def compiles_declared(self, task):
        """入口会编的声明（task_server_lib.py 文件头「编译目录」的入口声明）：命令位置上起的脚本文件头写了「# task-server-compiles: yes」→ (声明了没有, 写坏的那一句)。
        单独 --selftest、带 LISTING_ONLY_ARGUMENTS 之一的（只自证、只列项，不编）不算。"""
        script = self.command_script(task)
        words = task["command"]
        rest = words[lib.command_word_index(words) + 1:]
        if not script or rest == ["--selftest"] or any(word in LISTING_ONLY_ARGUMENTS for word in rest):
            return False, ""
        return lib.declared_compiles(script, self.environment)

    def nested_problem(self, command, cwd):
        """提交的命令里套了不收的包装、写了不收的参数：命令外层与 bash / sh -c 引号里每一条命令位置上的命令都照 nested_dispatch 判（只钻 -c 那一段，
        不读被起脚本的正文：脚本自己在里面怎么起不归提交这一关）；交第一条说明，没有交空串。"""
        found = lib.nested_dispatch(command, self.environment)
        if found or lib.broken("inner-commands-unchecked", self.environment):
            return found
        return self.inner_nested(command, cwd, 0)

    def inner_nested(self, words, directory, depth):
        index = lib.command_word_index(words)
        if index >= len(words) - 1 or words[index] != "-c" or depth >= MAXIMUM_LAUNCHER_DEPTH:
            return ""
        try:   # 照原样切：按 && || ; | & 与括号切成一条条命令，外壳（包装、nice、timeout）不剥，交给 nested_dispatch 判
            lexer = shlex.shlex(words[index + 1], posix=True, punctuation_chars=True)
            lexer.whitespace_split = True
            tokens = list(lexer)
        except ValueError:   # 引号不配对这类拆不开的：照外层判过的放行（钩子那一层另判写法）
            return ""
        commands, current = [], []
        for token in tokens:
            if token and set(token) <= set("&|;()"):
                commands.append(current)
                current = []
            else:
                current.append(token)
        commands.append(current)
        for command in commands:
            if not command:
                continue
            found = lib.nested_dispatch(command, self.environment) or self.inner_nested(command, directory, depth + 1)
            if found:
                return found
        return ""

    def revalidate_queued(self):
        """服务起来时把还在排的任务照现行的提交规则再核一遍（用户：「按规矩来 如果有问题就按照严格的逻辑，由脚本打回给提交放重发」）：
        提交时拒的全部照交任务那一刻同一份判（submit_refusal：环境变量名单、包装前缀与资源参数、编译目录、①–⑤ 与 ⑥ 整包测试、不指定范围的跑法、⑦ 启动测试与验证没点名；确认文件照任务记的那一份），
        在排的与待确认的都核；核不过的记 resubmit-required、写明哪一条与出路，结局照常推给提交它的会话（经 wait 与收件地址），改了重交。服务自己建的分片子任务不核。
        弄坏开关 queued-refusals-not-rechecked 下只核环境变量名单、包装前缀与编译目录三样。"""
        for task, valid in self.store.select("tasks", "state IN ('queued', ?)", (lib.AWAITING_CONFIRMATION_STATE,), order="id"):
            if not valid:
                continue
            if task.get("parent_id") or lib.broken("queued-not-revalidated", self.environment):
                continue
            problems = []
            if lib.broken("queued-refusals-not-rechecked", self.environment):
                _, refused = lib.split_environment(task.get("environment") or {}, self.environment)
                if refused:
                    problems.append("环境变量 " + "、".join(name for name, _ in refused) + " 现在不收")
                nested = self.nested_problem(task["command"], task["cwd"])
                if nested:
                    problems.append(nested)
                target = lib.target_directory_refusal(task.get("environment") or {}, task["command"], task["cwd"], self.environment)
                if target:
                    problems.append(str(target))
            else:
                _, refusal = self.submit_refusal(task["command"], task["cwd"], task.get("environment") or {}, task.get("whole_package_confirmed") or "", bool(task.get("full_gate")))
                if refusal is not None:
                    problems.append(refusal[2])
            if problems:
                say(f"  · 任务 {task['id']} 照现行规则不再收（{'；'.join(problems)}）：打回提交它的会话重发")
                self.finalize(task, "resubmit-required", None,
                              "服务规则改了，这条照现行规则不再收：" + "；".join(problems) + "。改了再 task-server/task submit 重交")

    def command_key(self, command, explicit_key):
        """命令键（task_server_lib.py 文件头「资源记录」的命令键）：先剥掉不改跑法的环境变量前缀与临时 prune-server 的包装（lib.key_words），cargo test 只留包、特性与档（lib.cargo_test_class_words），本仓门禁与入口脚本的 --check、--item、--only 去值归同类（lib.selection_class_words），
        再照 memory_peaks_lib.command_key 换算；包着临时 prune-server 的前面另接 lib.TEMPORARY_SERVICE_KEY_MARK（同一 scope 里多一份服务进程，与不包的不共用记录）。"""
        key = self.peaks.command_key(lib.selection_class_words(lib.cargo_test_class_words(lib.key_words(command, self.environment), self.environment), self.environment), explicit_key)
        if lib.temporary_service_wrapped(command, self.environment):
            return f"{lib.TEMPORARY_SERVICE_KEY_MARK} {key}"[:self.peaks.KEY_LENGTH_LIMIT]
        return key

    def submit(self, command, cwd, environment, session, process_id, inbox=None, whole_package="", full_gate=False, trace=None):
        """收下一条提交：inbox 是 task submit 交来的会话收件地址 {socket, token}（文件头 ⑥），两样都在才记，不然记 {} 并打一行「这条任务没有会话的收件地址」。
        whole_package 是核过形式的整包确认文件（文件头 ②「提交时拒」⑥），记在任务上。trace 给了（SubmitTrace）的照几步记时刻：认任务（命令键、会不会编、分片与按项）、记库、进队。"""
        step = trace.step if trace is not None else (lambda name: None)
        inbox = inbox or {}
        inbox = {"socket": inbox["socket"], "token": inbox["token"]} if inbox.get("socket") and inbox.get("token") else {}
        task = {"command": command, "cwd": cwd, "environment": environment, "session": session, "submitter_pid": process_id, "inbox": inbox,
                "unclaimed_at_submit": self.unclaimed_before(session, time.time()),   # 提交时警告 ②：会话名下有结局满时限没人取的
                "submitted_at": round(time.time(), 3), "command_key": self.command_key(command, environment.get("RUN_WITH_MEMORY_CAP_KEY")),
                "state": "queued", "outcome": "", "exit_code": None, "collected": False, "next_cap": None, "next_host": "", "outcome_counts": {},
                "note": "", "cancel_requested": False, "attempt_count": 0, "task_id": 0}
        task["compiles"] = self.compiles(task)
        task["compiles_problem"] = self.compiles_declared(task)[1]   # 会编的声明写坏了：派活时打回 run-condition-unmet（judge_placement）
        task["target_bypass"] = self.repository_target_bypass(task)   # 提交时警告（文件头 ②「提交时警告」）：照收照跑，三处打警告
        entry = lib.sharded_entry(command, cwd, self.environment)
        if entry:   # 文件头 ④：父任务不进队，先排计划那一步
            task.update(state="sharded", sharded_entry=entry, phase="plan", shard_count=0)
        # 交任务的二次确认（文件头 ⑦）：先记待确认、不进队、不建子任务；confirm 两个数对得上才照 awaiting_state 进队（照提交时刻的任务号排位置）
        if whole_package:
            task["whole_package_confirmed"] = whole_package
        if full_gate:
            task["full_gate"] = True
        awaiting = not lib.broken("confirm-skipped", self.environment)
        if awaiting:
            task.update(awaiting_state=task["state"], state=lib.AWAITING_CONFIRMATION_STATE)
        step("认任务")
        task = self.store.insert("tasks", task)
        step("记库")
        self.operation("submit", task["id"], session, process_id, shlex.join(command)[:300])
        for line in self.submit_warning_lines(task):
            say(f"  {line}")
        if not inbox:
            say(f"  · 任务 {task['id']}：这条任务没有会话的收件地址（task submit 没交 {lib.INBOX_SOCKET_VARIABLE} 与 {lib.INBOX_TOKEN_VARIABLE}），出事不发消息，结局只经 task wait 取")
        if not awaiting:
            self.enter_queue(task)
            step("进队")
        return task

    def enter_queue(self, task):
        """这条任务进队（确认过的，或弄坏开关 confirm-skipped 下交了就进）：分片的先建计划那一步（文件头 ④）。"""
        if task.get("state") == "sharded":
            self.crash_point("parent-inserted")
            self.create_child(task, "plan")
        self.wake()

    def confirmation(self, task):
        """待确认那一条的确认单（文件头 ⑦）→ {items, tasks, lines}：items 是命令里点名的项数（lib.named_scope），tasks 是这个会话此刻在排在跑的父任务条数（不算它自己，
        也不算别的待确认的），lines 是给人看的几行：整条命令、范围声明四段原文、项数、崩溃用例数与域（lib.crash_scope_text，不是崩溃入口又没点的不打）、在排在跑几条与其中命令键相同或点的项交叠的逐条列号、预测内存与记录来源、
        交的时候判得出的异常（预算贴着整台、按给的上限开并行；照异常表那一行带修改意见）。算不出的那一样照写算不出，不挡确认。"""
        named = lib.named_scope(task["command"], self.environment)
        entry = lib.named_scope_entry(task["command"], self.environment)
        mine = {(entry, name, value) for name, value in named}
        others = [record for record, valid in self.store.select("tasks", "session = ? AND state IN ('queued', 'running', 'sharded')", (task.get("session") or "",))
                  if valid and not record.get("parent_id") and record.get("id") != task["id"]]
        overlaps = []
        for other in others:
            words = other.get("command") or []
            shared = sorted(mine & {(lib.named_scope_entry(words, self.environment), name, value) for name, value in lib.named_scope(words, self.environment)})
            if other.get("command_key") == task.get("command_key"):
                overlaps.append(f"{other['id']}（命令键相同）")
            elif shared:
                overlaps.append(f"{other['id']}（都点了 " + "、".join(f"{name} {value}".strip() for _, name, value in shared[:5]) + "）")
        human = self.host_dispatch.human
        try:
            prediction = self.prediction_of(task)
            memory = (f"内存上限 {lib.mebibytes_text(int(prediction.memory_cap or 0))}（这把键记录的峰值 {human(int(prediction.memory_predicted or 0))}）"
                      f"（记录来源：{self.budget_source(task, prediction)}）")
        except Exception as error:   # 钉值读不到、库出错：照写算不出，确认之后排队那一轮照判
            memory = f"内存上限此刻算不出（{type(error).__name__}: {error}）"
        found = []
        try:
            self.budget_check(task, PassView(self, "confirm"))
            found += [f"{item['kind']}：{item['basis']}；修改意见：{item['next']}" for item in [self.budget_notices.get(task["id"])] + self.fills_any_cap_items(task) if item]
        except Exception as error:
            found.append(f"预算与按上限开并行此刻判不了（{type(error).__name__}: {error}）")
        scope = lib.scope_declaration_segments(task.get("environment") or {}, task["command"])
        crash = lib.crash_scope_text(task["command"], task.get("cwd") or self.peer_root, self.environment)
        lines = [f"确认单（任务 {task['id']}，待确认、不进队；满 {self.confirm_timeout_seconds:.0f} 秒不确认服务取消）：",
                 f"  命令：{shlex.join(task['command'])}",
                 "  范围声明：" + ("；".join(scope) if scope else "没带 SINGLEFS_RUN_SCOPE"),
                 f"  项数 {len(named)}：{lib.named_scope_text(named, self.scope_is_whole(task), self.environment)}",
                 *([f"  崩溃用例与域：{crash}"] if crash else []),
                 f"  这个会话此刻在排在跑 {len(others)} 条；其中命令键相同或点的项交叠的：{'、'.join(overlaps) or '没有'}",
                 f"  {memory}",
                 "  交的时候判得出的异常：" + ("；".join(found) if found else "没有")]
        return {"items": len(named), "tasks": len(others), "lines": lines}

    def scope_is_whole(self, task):
        """命令里一项都没点时，这条命令算不算整包、不指定范围（确认单「项数」那一行，文件头 ⑦）：整包测试的入口（lib.whole_package_script）、不带选条参数的门禁与入口（lib.unscoped_run_entry）、
        cargo 起的（cargo_runs）算；别的是脚本自己定范围，服务算不出。判不了的（读不了脚本这类）当脚本自己定。"""
        command, cwd = list(task["command"]), task.get("cwd") or self.peer_root
        try:
            # bash / sh -c 的脚本里命令位置上起的入口同样判（lib.shell_inner_commands）：合入后验证包在 bash -c 里，给了补丁目录的不是整包
            return bool(any(lib.whole_package_script(one, cwd, self.environment) or lib.unscoped_run_entry(one, cwd, self.environment)
                            for one in [command, *lib.shell_inner_commands(command, self.environment)])
                        or self.cargo_runs({"command": command, "cwd": cwd, "environment": task.get("environment") or {}}))
        except Exception as error:   # 确认单算不出的那一样照写，不挡确认
            say(f"  ! 任务 {task.get('id')} 确认单判不了是不是整包（{type(error).__name__}: {error}）：照脚本自己定范围写")
            return False

    def confirm(self, task_id, session, process_id, items, tasks):
        """task confirm（文件头 ⑦）→ (HTTP 状态码, 回话)：只许提交它的会话；任务要在待确认；给的项数与条数都与服务此刻算的（confirmation）相同才进队
        （照 awaiting_state 改状态、建子任务，任务号不变、照提交时刻排位置），对不上回 409、带服务算的两个数与确认单。拿这条任务的锁改，与取消、满时限取消不撞。
        弄坏开关 confirm-counts-unchecked 下不核两个数。"""
        found = self.store.task(task_id)
        if found is None:
            return 404, {"detail": f"没有任务 {task_id}"}
        task, valid = found
        if not valid:
            self.tampered(task_id, task, "任务记录")
            return 409, {"detail": f"任务 {task_id} 的记录校验不过，已记 record-tampered，不认这次确认"}
        if task.get("session") != session:
            self.operation("confirm-refused", task_id, session, process_id, "不是提交它的会话")
            return 403, {"detail": f"任务 {task_id} 不是这个会话提交的，不许确认"}
        if task.get("state") != lib.AWAITING_CONFIRMATION_STATE:
            return 409, {"detail": f"任务 {task_id} 不在待确认（此刻 {task.get('state')}），不用确认"}
        with self.session_lock(session):   # 核数与改状态在同一把会话锁里：同会话后到的确认照先到的进队之后的条数核
            mismatch = self.counts_mismatch(task, items, tasks, session, process_id, "confirm-refused", "confirm-counts-mismatch")
            if mismatch:
                return mismatch
            self.selftest_pause("confirm-counted")
            with self.store.task_lock(task_id):
                target = task.get("awaiting_state") or "queued"
                fields = {"state": target, "confirmed_at": round(time.time(), 3)}
                try:
                    task = self.store.change("tasks", task_id, fields,
                                             guard=lambda current: "" if current.get("state") == lib.AWAITING_CONFIRMATION_STATE else "不在待确认")
                except StoreRefused as refused:
                    if refused.kind != "guard":
                        raise
                    return 409, {"detail": f"任务 {task_id} 这一会儿已不在待确认（取消了，或满时限被服务取消）"}
                self.operation("confirm", task_id, session, process_id, f"items={items} tasks={tasks}")
        self.enter_queue(task)
        return 200, {"confirmed": True, "state": task.get("state"), "detail": f"任务 {task_id} 确认了：项数 {items}、这个会话在排在跑 {tasks} 条，照提交时刻排进队"}

    @contextlib.contextmanager
    def session_lock(self, session):
        """同一会话的 task confirm 串行（文件头 ⑦）：核数（counts_mismatch）与写库（改状态）在这一把锁里做，
        后到的照先到的进队之后的条数核，同会话并发确认绕不过在排在跑的条数。锁表 session_locks 照 keyed_lock 管（放下就删）；
        锁的次序是先会话锁、后任务锁。弄坏开关 confirm-counts-outside-session-lock 下不拿这把锁。"""
        if lib.broken("confirm-counts-outside-session-lock", self.environment):
            yield
        else:
            with keyed_lock(self.session_locks, self.session_locks_guard, session or ""):
                yield

    def counts_mismatch(self, task, items, tasks, session, process_id, refused_action, advice_kind):
        """task confirm 的核数（文件头 ⑦）：给的项数与在排在跑的条数都与服务此刻算的（confirmation）相同交 None；
        对不上记一条 refused_action 操作、交 (409, {detail: {mismatch, items, tasks, lines, problem, next_step}})，出路照异常表 advice_kind 那一行。
        弄坏开关 confirm-counts-unchecked 下不核。"""
        sheet = self.confirmation(task)
        if (items, tasks) == (sheet["items"], sheet["tasks"]) or lib.broken("confirm-counts-unchecked", self.environment):
            return None
        self.operation(refused_action, task["id"], session, process_id, f"给 items={items} tasks={tasks}，服务算 items={sheet['items']} tasks={sheet['tasks']}")
        return 409, {"detail": {"mismatch": True, "items": sheet["items"], "tasks": sheet["tasks"], "lines": sheet["lines"],
                                "problem": f"给的项数 {items}、条数 {tasks} 与服务此刻算的项数 {sheet['items']}、条数 {sheet['tasks']} 不同",
                                "next_step": lib.anomaly_advice(advice_kind, task["id"], self.environment)}}

    def confirm_long_run(self, task_id, session, process_id, why):
        """task confirm <号> --why（文件头 ⑦「跑满两小时」）→ (HTTP 状态码, 回话)：只许提交它的会话；--why 去掉空白至少 LONG_RUN_WHY_MINIMUM_CHARACTERS 个字；任务要在跑、
        有一次在跑的尝试、累计满 run_limit_seconds、没带 --full-gate。确认过的再确认照 200、不改原来那一句。合了记 long_run_confirmed {why, at}、往在跑那一次的日志接一行（不算日志在涨）。
        弄坏开关 long-run-confirm-unchecked 下不核会话、在不在跑与满没满界。"""
        found = self.store.task(task_id)
        if found is None:
            return 404, {"detail": f"没有任务 {task_id}"}
        task, valid = found
        if not valid:
            self.tampered(task_id, task, "任务记录")
            return 409, {"detail": f"任务 {task_id} 的记录校验不过，已记 record-tampered，不认这次确认"}
        unchecked = lib.broken("long-run-confirm-unchecked", self.environment)
        if task.get("session") != session and not unchecked:
            self.operation("long-run-confirm-refused", task_id, session, process_id, "不是提交它的会话")
            return 403, {"detail": f"任务 {task_id} 不是这个会话提交的，不许确认"}
        stated = " ".join((why or "").split())
        problem = lib.long_run_confirmation_problem(stated, self.environment)
        if problem:
            return 409, {"detail": f"{problem}（{lib.long_run_confirm_command(task_id)}）"}
        acknowledged = self.acknowledge_anomalies(task_id, session, process_id, stated)
        noted = f"任务 {task_id} 已记照跑（{'、'.join(acknowledged)}）：{stated}；这几种提醒不再再推" if acknowledged else ""
        if task.get("long_run_confirmed"):
            return 200, {"state": task.get("state"), "detail": noted or f"任务 {task_id} 原来就确认过：{task['long_run_confirmed'].get('why')}"}
        with self.store.task_lock(task_id):
            attempts = [attempt for attempt, attempt_valid in self.store.attempts_of(task_id) if attempt_valid]
            active = [attempt for attempt in attempts if attempt.get("state") in ACTIVE_ATTEMPT_STATES]
            if noted and (task.get("state") != "running" or not active or self.full_gate_of(task)
                          or (self.run_limit_accumulated(task, active[-1])[0] if active else 0.0) < self.run_limit_seconds) and not unchecked:
                return 200, {"state": task.get("state"), "detail": noted}
            if (task.get("state") != "running" or not active) and not unchecked:
                shards = "（分片的按各片确认：task-server/task status 看子任务号）" if task.get("sharded_entry") else ""
                return 409, {"detail": f"任务 {task_id} 此刻 {task.get('state')}、不在跑，不用确认{shards}"}
            if self.full_gate_of(task) and not unchecked:
                return 409, {"detail": f"任务 {task_id} 带 --full-gate，不提醒，不用确认"}
            total = self.run_limit_accumulated(task, active[-1])[0] if active else 0.0
            if total < self.run_limit_seconds and not unchecked:
                return 409, {"detail": f"任务 {task_id} 累计才跑了 {total:.0f} 秒，不到 {self.run_limit_seconds:.0f} 秒的界，不用确认"}
            try:
                self.store.change("tasks", task_id, {"long_run_confirmed": {"why": stated, "at": round(time.time(), 3)}},
                                  guard=lambda current: "" if current.get("state") == "running" or unchecked else "不在跑")
            except StoreRefused as refused:
                if refused.kind != "guard":
                    raise
                return 409, {"detail": f"任务 {task_id} 这一会儿已不在跑"}
        self.operation("long-run-confirm", task_id, session, process_id, stated[:200])
        if active:
            append_log_lines(active[-1].get("log", ""), [f"{lib.LONG_RUN_LOG_MARK} 长跑已确认：{stated}"])
        return 200, {"state": task.get("state"), "detail": f"任务 {task_id} 长跑已确认：{stated}；不再提醒，判卡住、内存上限与 {self.run_hard_limit_seconds:.0f} 秒的硬上限照旧"}

    def expire_unconfirmed(self):
        """交了满 confirm_timeout_seconds 没确认的（文件头 ⑦；异常表 unconfirmed）：拿这条任务的锁现读、还在待确认才收尾 cancelled，说明写「没确认，已取消」与修改意见；
        结局照常发给提交它的会话（push_outcomes）。弄坏开关 unconfirmed-kept 下一直留着。"""
        if lib.broken("unconfirmed-kept", self.environment):
            return
        now = time.time()
        for task, valid in self.store.select("tasks", "state = ?", (lib.AWAITING_CONFIRMATION_STATE,)):
            if not valid or now - float(task.get("submitted_at") or now) < self.confirm_timeout_seconds:
                continue
            with self.store.task_lock(task["id"]):
                found = self.store.task(task["id"])
                if found is None or not found[1] or found[0].get("state") != lib.AWAITING_CONFIRMATION_STATE:
                    continue
                self.finalize(found[0], "cancelled", None,
                              (f"没确认，已取消：交了满 {self.confirm_timeout_seconds:.0f} 秒没 task-server/task confirm {task['id']}，一次都没起；"
                               + lib.anomaly_advice("unconfirmed", task["id"], self.environment)))

    def clean(self, task_id, session, process_id):
        """→ (HTTP 状态码, 回话)。删这条任务（分片的连同子任务）留下的编译目录与现场，分片的父任务另删计划目录下的测试二进制（shard_binaries_removal）：
        只许提交它的会话、有了结局才删（task_server_lib.py 文件头「编译目录」「现场」，文件头 ④）。
        第二台那一趟的编译目录 multi-host-run.sh 自己删；留着（build_directory_kept）而记结局时没删成的照第二台的删法删（PeerMachine.remove_build_directory）。
        子任务或尝试的记录校验不过（关联列改过这类，TS-05）的一份都不删。
        删的那一段拿 build_removal_lock、锁里现读各次尝试（文件头「接口」的取消那一段末尾）。"""
        found = self.store.task(task_id)
        if found is None:
            return 404, {"detail": f"没有任务 {task_id}"}
        task, valid = found
        if not valid:
            self.tampered(task_id, task, "任务记录")
            return 409, {"detail": f"任务 {task_id} 的记录校验不过，已记 record-tampered，不认这次删除"}
        if task.get("session") != session and not lib.broken("clean-session-unchecked", self.environment):
            self.operation("clean-refused", task_id, session, process_id, "不是提交它的会话")
            return 403, {"detail": f"任务 {task_id} 不是这个会话提交的（提交它的会话与这一次的不同），不许删它的编译目录"}
        if task.get("state") != "finished":
            return 409, {"detail": f"任务 {task_id} 还没有结局（{task.get('state')}）：等 wait 拿到结局再 clean"}
        try:
            owners = [task] + self.children_of(task_id)
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            return 409, {"detail": self.children_refused(task_id, refused, "一份编译目录与现场都不删")}
        # 删编译目录与现场拿 build_removal_lock（记下结局时删、取消之后删同一把）：各次尝试在锁里现读，先到的删、后到的读到已删就不再删；
        # 弄坏开关 clean-races-finished-removal 下不拿锁，与记下结局那一侧同时删
        with (contextlib.nullcontext() if lib.broken("clean-races-finished-removal", self.environment) else self.build_removal_lock):
            attempts = [(record, record_valid) for owner in owners for record, record_valid in self.store.attempts_of(owner["id"])]
            unverified = [record.get("id") for record, record_valid in attempts if not record_valid]
            if unverified:
                self.alert("record-tampered", task_id, f"任务 {task_id} 名下有校验不过的尝试记录（行 {unverified}）：一份编译目录都不删",
                           "查是谁直接写了库（只有 task-server 读写它）")
                return 409, {"detail": f"任务 {task_id} 名下有校验不过的尝试记录（行 {unverified}，关联列与正文对不上或写进之后被改过）：一份都不删"}
            removed, left = [], []
            for attempt, _ in attempts:
                directory = attempt.get("build_directory")
                if not directory or attempt.get("build_directory_removed"):
                    continue
                # 第二台留着的那一份（带 --keep-build-directory 的那一趟、记结局时没删成）：照那一台的删法删（Machine.remove_build_directory，multi-host-run.sh --remove-build-directory），
                # 大小量不到记 0。弄坏开关 clean-peer-build-left 下照本机那一路判（本机没有这个路径，一份不删）
                remote = (attempt.get("host") or LOCAL) != LOCAL and attempt.get("build_directory_kept") and not lib.broken("clean-peer-build-left", self.environment)
                if not remote and not os.path.lexists(directory):   # 已删的、起法那一层自己删了的不再删
                    continue
                size = 0 if remote else directory_bytes(directory)
                if lib.broken("clean-removes-nothing", self.environment):
                    done, lines = True, []
                elif remote:
                    done, lines = self.machines[attempt["host"]].remove_build_directory(directory)
                else:
                    done, lines = self.build_directories.remove_run_build_directory(directory)
                if done and (remote or not os.path.lexists(directory)):
                    self.mark_build_directory_removed(attempt)
                    removed.append({"path": directory, "bytes": size})
                else:
                    left.append({"path": directory, "bytes": size, "kind": "编译目录", "why": "；".join(line.strip() for line in lines) or "删完还在"})
            if not lib.broken("scene-left-by-clean", self.environment):   # 各次尝试的现场一并删（task_server_lib.py 文件头「现场」）
                scenes_removed, scenes_left = self.remove_scenes(task_id)
                removed += scenes_removed
                left += scenes_left
            if task.get("sharded_entry"):   # 分片的父任务：计划目录下预先编好的测试二进制一并删（文件头 ④）
                binaries_removed, binaries_left = self.shard_binaries_removal(task_id, session, process_id)
                removed += binaries_removed
                left += binaries_left
        # 没删成的写明是哪一份、为什么（照 remove_run_build_directory 交回的原因）：操作日志与回话都带；弄坏开关 clean-failure-unexplained 下照改前只记份数
        explained = not lib.broken("clean-failure-unexplained", self.environment)
        reasons = "；".join(f"{item['kind']} {item['path']}：{item['why']}" for item in left)
        self.operation("clean", task_id, session, process_id, f"删了 {len(removed)} 份、没删成 {len(left)} 份" + (f"：{reasons}" if left and explained else ""))
        if left:
            return 409, {"detail": {"text": reasons if explained else f"没删成 {len(left)} 份", "removed": removed, "left": left if explained else []}}
        return 200, {"removed": removed, "left": []}

    def cancel(self, task_id, session, process_id):
        """→ (HTTP 状态码, 回话)。只许提交它的会话；在跑的停单元、回读停了才回 stopped。
        这条任务的锁拿着判：在队里的当场记 cancelled（起单元那一侧拿同一把锁，判放得下之后起之前到的就不起）；已经起了的等起完、停单元、回读（TS-01）。"""
        found = self.store.task(task_id)
        if found is None:
            return 404, {"detail": f"没有任务 {task_id}"}
        task, valid = found
        if not valid:
            self.tampered(task_id, task, "任务记录")
            return 409, {"detail": f"任务 {task_id} 的记录校验不过，已记 record-tampered，不认这次取消"}
        if task.get("session") != session and not lib.broken("cancel-session-unchecked", self.environment):
            self.operation("cancel-refused", task_id, session, process_id, "不是提交它的会话")
            return 403, {"detail": f"任务 {task_id} 不是这个会话提交的（提交它的会话与这一次的不同），不许取消"}
        if task.get("state") == lib.AWAITING_CONFIRMATION_STATE:   # 待确认（文件头 ⑦）：没进队、没有子任务与尝试，拿锁现读还在待确认就当场收尾
            with self.store.task_lock(task_id):
                found = self.store.task(task_id)
                if found is not None and found[1] and found[0].get("state") == lib.AWAITING_CONFIRMATION_STATE:
                    self.operation("cancel", task_id, session, process_id)
                    self.finalize(found[0], "cancelled", None, "待确认时被取消，没进队")
                    return 200, {"stopped": True, "state": "finished", "outcome": "cancelled", "detail": "待确认时取消，没进队", "artifacts": {"removed": [], "left": [], "lines": []}}
            found = self.store.task(task_id)   # 这一会儿确认了：照进了队的取消
            if found is None or not found[1]:
                return 409, {"detail": f"任务 {task_id} 的记录读不回或校验不过，不认这次取消"}
            task = found[0]
        # 取消的不留产物（文件头「接口」的取消）：删前的大小在停之前量，记下结局那一侧先删了的照这一份报
        before = (self.cancel_artifact_sizes(task_id) if task.get("state") != "finished" or task.get("sharded_entry")
                  else {"sizes": {}, "active": set()})
        try:
            has_children = any(child.get("role") in lib.SHARD_ROLES for child in self.children_of(task_id))
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            has_children = True   # 名下有校验不过的子任务行：照分片的父任务办，cancel_sharded 判出来回 409、一个单元都不停
        parent = bool(task.get("sharded_entry")) or task.get("state") == "sharded" or has_children   # 分片的父任务：名下有分片子任务的也算
        if parent and not lib.broken("sharded-cancel-finished-first", self.environment):
            return self.cancel_sharded(task, session, process_id, before)
        if not parent:
            refusal = self.tampered_cancel_refusal(task_id, task)
            if refusal is not None:
                return refusal
        if task.get("state") == "finished" and not parent:
            if lib.broken("finished-cancel-overwrites", self.environment):
                self.finalize(task, "cancelled", None, "弄坏开关 finished-cancel-overwrites：已有结局的也改记 cancelled", overwrite=True)
            return 200, {"stopped": True, "state": "finished", "outcome": task.get("outcome"), "detail": "已经有结局了，没什么可取消"}
        if task.get("state") == "finished":   # 弄坏开关 sharded-cancel-finished-first：改前的样子，分片的父任务已有结局就不再看子任务
            return 200, {"stopped": True, "state": "finished", "outcome": task.get("outcome"), "detail": "已经有结局了，没什么可取消"}
        if task.get("state") == "sharded":   # 弄坏开关 sharded-cancel-finished-first：改前的样子，先给父任务记结局、再停子任务，第二次取消见它已有结局就回停了
            self.store.change("tasks", task_id, {"cancel_requested": True})
            self.operation("cancel", task_id, session, process_id)
            self.finalize_parent(task, None, "cancelled", None, "提交它的会话取消了整件")
            problems = self.cancel_children(task_id)
            if problems:
                return 500, {"stopped": False, "detail": "；".join(problems)}
            return 200, {"stopped": True, "state": "finished", "outcome": "cancelled", "detail": "整件取消了：在排的子任务没起，在跑的单元已停"}
        # 分片的子任务被人手单独取消（不是整件被取消、不是兄弟片没成被服务撤掉）记 cancel_origin=session：只撤这一片，别的片照跑、合并照起（drive_parent_phase）
        alone = {"cancel_origin": "session"} if task.get("parent_id") and not lib.broken("shard-cancel-ends-whole", self.environment) else {}
        with self.store.task_lock(task_id):
            try:
                task = self.store.change("tasks", task_id, dict(alone, cancel_requested=True),
                                         guard=lambda current: "已结了局" if current.get("state") == "finished" else "")
            except StoreRefused as refused:
                if refused.kind != "guard":
                    raise
                outcome = (self.store.task(task_id) or ({},))[0].get("outcome")
                if outcome == "record-tampered":   # 判的这一会儿对账判出校验不过、记了 record-tampered：照样不报停了（TS-05）
                    return self.tampered_cancel_refusal(task_id, dict(task, outcome=outcome))
                return 200, {"stopped": True, "state": "finished", "outcome": outcome, "detail": "已经有结局了，没什么可取消"}
            self.operation("cancel", task_id, session, process_id)
            queued = task.get("state") == "queued" and not lib.broken("queued-cancel-left-queued", self.environment)
            if queued:
                self.finalize(task, "cancelled", None, "在队里时被取消，没起")
        if queued:   # 撞上限重排回队、上一次留着编译目录的，出了这条任务的锁再删
            return 200, self.with_cancelled_artifacts(task_id, before, {"stopped": True, "state": "finished", "outcome": "cancelled", "detail": "在队里时取消，没起"})
        self.selftest_pause("cancel-unlocked")
        status, reply = self.stop_running(task_id)
        if (status != 200 or lib.broken("cancel-outcome-unchecked", self.environment)   # 弄坏开关：照改前停了就回 stopped，不等结局写回、不核是不是 cancelled
                or lib.broken("cancel-no-readback", self.environment)):
            return status, reply
        status, reply = self.cancel_outcome_reply(task_id, reply)
        return status, (self.with_cancelled_artifacts(task_id, before, reply) if status == 200 else reply)

    def with_cancelled_artifacts(self, task_id, before, reply):
        """取消回 stopped 之前删产物（remove_cancelled_artifacts），回话另带 artifacts、detail 后面接删了几份。"""
        artifacts = self.remove_cancelled_artifacts(task_id, before)
        if not artifacts["lines"]:   # 弄坏开关 cancelled-artifacts-kept：不删、不报
            return reply
        return dict(reply, artifacts=artifacts, detail=f"{reply.get('detail', '')}；{artifacts['lines'][-1].strip().lstrip('· ')}")

    def cancel_outcome_reply(self, task_id, stopped):
        """停了单元、回读过之后（stopped 是 stop_running 的回话）等对账把结局写回，照写回的结局回（TS-03）：结局是 cancelled 才回 stopped；
        别的结局（取消到的时候命令已自己跑完、退 0 记了 succeeded 这类）回 409、点名那个结局，不报停了；至多等 cancel_settle_seconds，等不到回 500、不报停了。"""
        found = self.await_finished(task_id, time.monotonic() + self.cancel_settle_seconds)
        if found is not None and not found[1]:
            return self.tampered_cancel_refusal(task_id, dict(found[0], outcome="record-tampered"))
        if found is None or found[0].get("state") != "finished":
            return 500, {"stopped": False, "accepted": True, "state": (found or ({},))[0].get("state"),
                         "detail": f"单元已停、回读过，等了 {self.cancel_settle_seconds:.0f} 秒结局还没写回：结局写回之前不报停了；"
                                   f"task-server/task status {task_id} 看结局"}
        task = found[0]
        if task.get("outcome") == "record-tampered":
            return self.tampered_cancel_refusal(task_id, task)
        if task.get("outcome") != "cancelled":
            return 409, {"stopped": False, "state": "finished", "outcome": task.get("outcome"), "exit_code": task.get("exit_code"),
                         "detail": f"取消到的时候任务 {task_id} 已自己结束：结局 {task.get('outcome')}（退 {task.get('exit_code')}），不是取消；产物照这个结局认"}
        return 200, dict(stopped, state="finished", outcome="cancelled", detail="单元已停（回读 ActiveState 不在跑、cgroup.procs 空），结局 cancelled 已写回")

    def await_finished(self, task_id, deadline):
        """停了单元、回读过之后等对账把这条任务记成 finished（cancel_outcome_reply 与取消整件的 cancel_children 共用）：每 CANCEL_SETTLE_POLL_SECONDS 现读一次、
        叫醒排队线程，读到 finished 或校验不过、或到了 deadline（time.monotonic 的时刻）就交最后读到的 (正文, 校验过没有)，库里没有交 None。"""
        while True:
            found = self.store.task(task_id)
            if found is not None and (not found[1] or found[0].get("state") == "finished"):
                return found
            if time.monotonic() >= deadline:
                return found
            self.wake()
            time.sleep(CANCEL_SETTLE_POLL_SECONDS)

    def tampered_cancel_refusal(self, task_id, task):
        """取消一条不分片的任务之前核它名下的记录（TS-05：停止必须拒绝未验证的记录）：名下有校验不过的尝试记录（关联列与正文对不上这类），
        或任务已因校验不过记了 record-tampered（服务不认它名下的单元，一个都没停），回 409、不报停了；都核得过交 None。"""
        unverified = [record.get("id") for record, valid in self.store.attempts_of(task_id) if not valid]
        if unverified:
            self.alert("record-tampered", task_id, f"任务 {task_id} 名下有校验不过的尝试记录（行 {unverified}）：取消不停任何单元",
                       "查是谁直接写了库（只有 task-server 读写它）；对账会记这几行 ended、停它们的单元（报 tampered-unit-stop）")
            return 409, {"stopped": False, "detail": f"任务 {task_id} 名下有校验不过的尝试记录（行 {unverified}，关联列与正文对不上或写进之后被改过）：一个单元都不停"}
        if task.get("outcome") == "record-tampered":
            return 409, {"stopped": False, "detail": f"任务 {task_id} 已因记录校验不过记了 record-tampered：服务不认它名下的单元、一个都没停，不报停了；"
                                                     "它在跑的单元由对账停，结果看告警 tampered-unit-stop"}
        return None

    def cancel_sharded(self, parent, session, process_id, before):
        """取消分片的整件（TS-04）：父任务没结局的先记 cancel_requested、阶段改成 cancelling（不先记结局）；在排的子任务当场收尾，在跑的逐个停、回读；
        都停了、读回过才给整件记 cancelled、回 stopped。有没停的回 500（取消已记下、整件留在 cancelling），再次取消照样重查、重停活动的子任务；
        父任务已有结局而子任务还在跑的同样停它们。弄坏开关 sharded-cancel-children-left 下照改前只给父任务记 cancelled、子任务不动。"""
        parent_id = parent["id"]
        was_finished = parent.get("state") == "finished"
        with self.store.task_lock(parent_id):
            if parent.get("state") != "finished":
                with contextlib.suppress(StoreRefused):
                    parent = self.store.change("tasks", parent_id, {"cancel_requested": True, "phase": "cancelling"},
                                               guard=lambda current: "已结了局" if current.get("state") == "finished" else "")
        self.operation("cancel", parent_id, session, process_id)
        if lib.broken("sharded-cancel-children-left", self.environment) and parent.get("state") != "finished":
            self.finalize_parent(parent, None, "cancelled", None, "提交它的会话取消了整件")
            return 200, {"stopped": True, "state": "finished", "outcome": "cancelled", "detail": "（弄坏开关 sharded-cancel-children-left：子任务没动）"}
        try:   # 关联列改过的（TS-05）：认不出哪些子任务是它的，一个都不停
            children = self.children_of(parent_id)
            problems = self.cancel_children(parent_id)
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            return 409, {"stopped": False, "accepted": parent.get("state") != "finished", "detail": self.children_refused(parent_id, refused, "一个单元都不停")}
        if problems:
            return 500, {"stopped": False, "accepted": True, "state": parent.get("state"), "phase": parent.get("phase"),
                         "detail": "取消已记下（整件留在 cancelling，还没回读到停的子任务下一次 cancel 照样重停）：" + "；".join(problems)}
        current = self.store.task(parent_id)
        if current is not None and current[0].get("state") != "finished":
            self.finalize_parent(current[0], None, "cancelled", None, "提交它的会话取消了整件")
        outcome = ((self.store.task(parent_id) or ({},))[0]).get("outcome")
        if not was_finished:
            return 200, self.with_cancelled_artifacts(parent_id, before, {"stopped": True, "state": "finished", "outcome": outcome, "detail": "整件取消了：在排的子任务没起，在跑的单元已停、回读过"})
        if not any(record.get("state") != "finished" for record in children):
            return 200, {"stopped": True, "state": "finished", "outcome": outcome, "detail": "已经有结局了，没什么可取消"}
        return 200, self.with_cancelled_artifacts(parent_id, before, {"stopped": True, "state": "finished", "outcome": outcome, "detail": "整件早有结局；还在跑的子任务单元已停、回读过"})

    def cancel_children(self, parent_id):
        """取消父任务名下还没结局的子任务（整件被取消，cancel_origin=parent）：在排的记 cancelled，在跑的停单元、回读，再等每一条停了的在库里记成 finished
        （await_finished，同不分片的取消那一种等法，几条共用一个 cancel_settle_seconds 的时限）；→ 没停成、等不到结局的那几句（点名子任务号）。
        调用方（cancel_sharded）没有这几句才给父任务记结局、删产物：父任务的结局不早于子任务，删产物时子任务的尝试都已对账收尾。
        弄坏开关 parent-cancel-spares-running 下照兄弟片没成那一路办（cancel_queued_children）：只撤在排的，在跑的不停；
        parent-finalized-before-children 下停了单元就交，不等子任务的结局。"""
        if lib.broken("parent-cancel-spares-running", self.environment):
            self.cancel_queued_children(self.children_of(parent_id), "parent", "整件被取消，没起")
            return []
        problems, stopped = [], []
        for child in self.children_of(parent_id):
            if child.get("state") == "finished":
                continue
            with self.store.task_lock(child["id"]):
                try:
                    child = self.store.change("tasks", child["id"], {"cancel_requested": True, "cancel_origin": "parent"},
                                              guard=lambda current: "已结了局" if current.get("state") == "finished" else "")
                except StoreRefused:
                    continue
                if child.get("state") == "queued":
                    self.finalize(child, "cancelled", None, "整件被取消，没起")
                    continue
            status, reply = self.stop_running(child["id"])
            if status != 200:
                problems.append(f"子任务 {child['id']}：{reply.get('detail', '')}")
            else:
                stopped.append(child["id"])
        if lib.broken("parent-finalized-before-children", self.environment):
            return problems
        deadline = time.monotonic() + self.cancel_settle_seconds
        for child_id in stopped:
            found = self.await_finished(child_id, deadline)
            if found is not None and not found[1]:
                problems.append(f"子任务 {child_id}：单元已停、回读过，记录校验不过，不认它的结局")
            elif found is None or found[0].get("state") != "finished":
                problems.append(f"子任务 {child_id}：单元已停、回读过，等了 {self.cancel_settle_seconds:.0f} 秒结局还没写回（{(found or ({},))[0].get('state')}）")
        return problems

    def stop_running(self, task_id, attempt_id=None):
        """停这个任务在跑的那几次尝试的单元、回读停了（取消那一侧按任务停全部）；给了 attempt_id 的只停那一次（判卡住的后台线程，judged_attempt_only）。
        → (HTTP 状态码, 回话)。名下有校验不过的尝试记录（关联列 task_id 与正文对不上这类，TS-05）的一个都不停、回 409：认不出那一行是谁的。
        自证里停完往排队读数记一行「stop task=<号> attempt=<号或 all> stopped=<停了的尝试记录 id，逗号接；一个都没有写 ->」。"""
        if lib.broken("cancel-no-readback", self.environment):
            return 200, {"stopped": True, "state": "running", "detail": "（弄坏开关 cancel-no-readback：没停、没回读）"}
        self.selftest_pause("stop-attempts-read")
        rows = self.store.attempts_of(task_id)
        unverified = [record.get("id") for record, valid in rows if not valid]
        if unverified:
            self.alert("record-tampered", task_id, f"任务 {task_id} 名下有校验不过的尝试记录（行 {unverified}）：取消不停任何单元",
                       "查是谁直接写了库（只有 task-server 读写它）；对账会记这几行 ended、停它们的单元（报 tampered-unit-stop）")
            return 409, {"stopped": False, "detail": f"任务 {task_id} 名下有校验不过的尝试记录（行 {unverified}，关联列与正文对不上或写进之后被改过）：一个单元都不停"}
        attempts = [record for record, _ in rows if record.get("state") in ACTIVE_ATTEMPT_STATES and (attempt_id is None or record.get("id") == attempt_id)]
        skip_stop = lib.selftest_override("TASK_SERVER_SELFTEST_STOP_FAILS", self.environment)
        skipped = bool(skip_stop) and os.path.exists(skip_stop)
        if skipped:   # 自证：演这一次 systemctl stop 没停成（只这一次）
            os.remove(skip_stop)
        problems = []
        for attempt in attempts:
            self.service_stopped_attempts[attempt.get("id")] = time.time()
            if not attempt.get("unit"):
                child = next((child for child in self.children if child.pid == attempt.get("child_pid")), None)
                if child is not None:
                    child.send_signal(signal.SIGTERM)
                    with contextlib.suppress(subprocess.TimeoutExpired):
                        child.wait(timeout=UNIT_STOP_SECONDS)
                continue
            if not skipped:
                with contextlib.suppress(OSError, subprocess.TimeoutExpired):
                    subprocess.run(["systemctl", "--user", "stop", attempt["unit"]], capture_output=True, env=self.environment, stdin=subprocess.DEVNULL,
                                   timeout=UNIT_STOP_SECONDS + 30)
            states = unit_states(attempt["unit"], self.environment)
            processes = unit_processes(attempt["unit"], self.environment)
            if states is None or states[1] in lib.RUNNING_UNIT_STATES or processes is None or processes:
                problems.append(f"单元 {attempt['unit']} 停了之后回读：状态 {states[1] if states else '读不到'}、cgroup 里进程 "
                                f"{'读不到' if processes is None else len(processes)} 个")
            else:
                self.release_stopped_booking(attempt, True)   # 回读到单元停了：CPU 与内存的在跑记账先放，第二台的清理回执与收尾慢不占账
                peer_clean, peer_note = self.confirm_peer_cleanup(attempt)
                if not peer_clean:
                    problems.append(peer_note)
        self.wake()
        self.selftest_read(f"stop task={task_id} attempt={attempt_id if attempt_id is not None else 'all'} "
                           f"stopped={','.join(str(attempt.get('id')) for attempt in attempts) or '-'}")
        if problems:
            return 500, {"stopped": False, "detail": "；".join(problems)}
        return 200, {"stopped": True, "state": "running", "units": [attempt["unit"] for attempt in attempts],
                     "detail": "单元已停（回读 ActiveState 不在跑、cgroup.procs 空），结局由对账写回"}

    def mark_collected(self, task_id):
        with contextlib.suppress(StoreRefused):
            self.store.change("tasks", task_id, lambda current: {"collected": True, "collected_at": round(time.time(), 3)},
                              guard=lambda current: "" if current.get("state") == "finished" and not current.get("collected") else "不用记")

    def summary(self, task_id):
        found = self.store.task(task_id)
        if found is None:
            return None
        task, valid = found
        if not valid:
            self.tampered(task_id, task, "任务记录")
            task, valid = self.store.task(task_id)
        attempts = []
        for attempt, attempt_valid in self.store.attempts_of(task_id):
            attempt["valid"] = attempt_valid
            attempts.append(attempt)
        now = time.time()
        for attempt in attempts:
            if attempt.get("state") in ACTIVE_ATTEMPT_STATES and not lib.broken("running-progress-unreported", self.environment):
                attempt["progress"] = self.running_progress(task, attempt, now)
        warnings = self.submit_warning_lines(task) if valid else []
        if task.get("inbox"):   # 会话令牌不出服务（文件头 ⑥）：回话里只留收件 socket
            task = dict(task, inbox={"socket": task["inbox"].get("socket", "")})
        summary = {"task": task, "valid": valid, "attempts": attempts}
        if warnings:   # 提交时警告（文件头 ②「提交时警告」）：status 照打
            summary["warnings"] = warnings
        pin = self.pin_record(task.get("command_key", "")) if task.get("command_key") else None
        if pin is not None:   # 这条命令钉了上限（文件头 ②「内存」的钉值）：status 写明是钉值、此刻多少
            summary["pinned_cap"] = self.pins_reply(task["command_key"])
        if task.get("state") == lib.AWAITING_CONFIRMATION_STATE:   # 待确认（文件头 ⑦）：status 照此刻现算的确认单打
            summary["confirmation"] = self.confirmation(task)
        if task.get("state") in ("queued", "sharded"):
            if not lib.broken("waiting-reason-unreported", self.environment):
                summary["waiting_for"] = self.waiting_text(task)
            summary["queue_position"] = self.queue_position_reply(task)
        if task.get("sharded_entry"):
            try:
                children = self.children_of(task_id)
            except StoreRefused as refused:   # 名下有校验不过的子任务行：那几行不列、不认，写告警，children_unverified 点名（TS-05）
                if refused.kind != "tampered":
                    raise
                summary["children_unverified"] = self.children_refused(task_id, refused, "status 不列那几行")
                children = [record for record, valid in self.store.select("tasks", "task_id = ?", (task_id,)) if valid]
            summary["children"] = [self.summary(child["id"]) for child in children]
        return summary

    def running_progress(self, task, attempt, now):
        """在跑的那一次（文件头「卡住」）：已跑秒数、日志多少秒没涨、到此刻平均在用核数（本机读 scope 的 cpu.stat）、核数提醒的界、
        日志不涨时最近一回读到的进展与它是多少秒前读的（progress_noted；没读到过是 None）；量不到的是 None。"""
        elapsed = now - float(attempt.get("started_at") or now)
        seen = self.log_growth.get(attempt.get("id"))
        usage = self.machines[attempt["host"]].cpu_usec(attempt)
        cores = round(usage / 1e6 / elapsed, 3) if usage is not None and elapsed > 0 else None
        suspect = attempt.get("suspect") or {}
        noted = self.progress_noted.get(attempt.get("id"))   # 日志不涨时最近一回读到的进展（note_attempt_progress）
        return {"elapsed_seconds": round(elapsed, 3),
                "progress_note": None if noted is None else noted[1], "progress_age_seconds": None if noted is None else round(time.monotonic() - noted[0], 3),
                "idle_core_line": round(int(self.threads_of(task) or 0) * LONG_RUN_IDLE_CORE_FRACTION),
                "idle_core_after_seconds": self.long_run_attention_seconds,
                "suspect": suspect if suspect.get("active") else None,
                "log_quiet_seconds": None if seen is None else round(time.monotonic() - seen[1], 3), "cores": cores,
                "computing_quietly": bool(seen) and bool(seen[4]),
                "threads": self.threads_of(task), "stall_seconds": self.stall_seconds}

    def session_tasks(self, session):
        listed = []
        for task, valid in self.store.select("tasks", "session = ?", (session,)):
            if task.get("parent_id"):
                continue   # 子任务列在父任务名下（/tasks/<父任务号>）
            unclaimed = task.get("state") == "finished" and not task.get("collected")
            if unclaimed and lib.broken("unclaimed-not-listed", self.environment):
                continue
            if task.get("state") != "finished" or unclaimed:
                listed.append({"id": task["id"], "state": task.get("state"), "outcome": task.get("outcome"), "exit_code": task.get("exit_code"),
                               "unclaimed": unclaimed, "valid": valid, "command": shlex.join(task.get("command") or [])[:200],
                               "phase": task.get("phase", ""), "warnings": self.submit_warning_lines(task) if valid else [],
                               "waiting_for": "" if lib.broken("waiting-reason-unreported", self.environment) else self.waiting_text(task),
                               "queue_position": self.queue_position_reply(task)})
        return listed


# ── 接口 ──
def build_application(scheduler):
    from fastapi import FastAPI, HTTPException, Request
    from pydantic import BaseModel
    from starlette.concurrency import run_in_threadpool
    from typing import Dict, List

    application = FastAPI(title="singlefs task-server", docs_url=None, redoc_url=None, openapi_url=None)
    store = scheduler.store
    environment = scheduler.environment
    wait_fallback = scheduler.tick_seconds   # wait 的兜底：结局的通知丢了，至多等一轮主循环再查库（文件头「通知」）
    waiters = {}
    loop_holder = {}

    def release_waiters(task_id):
        for event in waiters.pop(task_id, []):
            event.set()

    def finished(task_id):
        loop = loop_holder.get("loop")
        if loop is not None:
            loop.call_soon_threadsafe(release_waiters, task_id)

    scheduler.finished_listeners.append(finished)

    class SubmitBody(BaseModel):
        command: List[str]
        cwd: str
        environment: Dict[str, str] = {}
        inbox: Dict[str, str] = {}   # 会话的收件地址 {socket, token}（文件头 ⑥；task submit 从会话的环境读）
        whole_package_confirmed: str = ""   # 整包确认文件（task submit --whole-package-confirmed；文件头 ②「提交时拒」⑥）
        manual: bool = False   # 人手交（task submit --manual）：Scheduler.manual_refusal 判
        full_gate: bool = False   # 全量门禁标识（task submit --full-gate）：Scheduler.full_gate_refusal 判

    def peer_of(request):
        client = request.client
        if client is None or not str(client.host).startswith("uid-"):
            raise HTTPException(status_code=403, detail="读不到调用方的进程号与 uid（只认经 unix socket 来的）")
        if int(str(client.host)[4:]) != os.getuid():
            raise HTTPException(status_code=403, detail="调用方的 uid 不是服务自己的")
        return int(client.port)

    def session_of(process_id):
        return lib.session_of_peer(process_id, scheduler.boundary, environment)

    @application.get("/health")
    def health():
        """只做库写入试探与排队线程心跳；派活判排头能起却一直没起成、看进展那一句都读排队线程每一轮算好的（文件头「接口」），不现读余量、不起子进程；
        读不到单元状态那一句与 unit_unreadable 栏读对账记下的那一段（Scheduler.unit_unreadable_report，文件头 ①「读不到单元状态」）。
        弄坏开关 unit-unreadable-health-unreported 下不报那一句、unit_unreadable 栏是空表。"""
        writable, problem = store.health_write()
        now = time.time()
        since = now - scheduler.last_tick
        starved = scheduler.starved_problem(now, fresh=True) if lib.broken("health-judges-head", environment) else scheduler.starved_text
        stalled = scheduler.progress_text
        unreadable, unreadable_text = ([], "") if lib.broken("unit-unreadable-health-unreported", environment) else scheduler.unit_unreadable_report()
        problems = ([] if writable else [f"库写不进：{problem}"]) + ([starved] if starved else []) + ([stalled] if stalled else []) + ([unreadable_text] if unreadable_text else [])
        counts = {state: store.count("tasks", state) for state in ("queued", "running")}
        pause = None if lib.broken("pause-health-unreported", environment) else scheduler.pause   # 暂停不让 ok 变 false（文件头 ⑤「暂停」）
        other_running = None if pause is None else scheduler.running_outside_allowed(pause, counts["running"])
        report_allow = not lib.broken("pause-allow-unreported", environment)   # 放行的会话、别的会话在跑几个（文件头 ⑤「暂停」）
        return {"ok": not problems, "problem": "；".join(problems), "database_writable": writable, "scheduler_seconds_since_tick": round(since, 3),
                "tag": store.tag, "pid": os.getpid(), **counts, "nfs": scheduler.nfs_snapshot(), "peer": scheduler.peer_health(), "booking_drift": scheduler.booking_drift_snapshot(), "booking_ledger": scheduler.booking_ledger_snapshot(), "unit_unreadable": unreadable,
                "paused": pause is not None,
                "pause": None if pause is None else {"reason": pause["reason"], "since": pause["since"], "text": scheduler.pause_text(pause, counts["running"], other_running),
                                                     **({"allow_session": pause["allow_session"], "other_running": other_running} if report_allow else {})}}

    @application.post("/tasks")
    async def submit(body: SubmitBody, request: Request):
        # 收到的时刻在事件循环上记（SubmitTrace）：活在线程池里做，排进线程等了多久也记得下；回话时打一行进出日志
        trace = SubmitTrace(scheduler.environment)
        outcome = "出错"
        try:
            reply = await run_in_threadpool(submit_in_thread, body, request, trace)
            outcome = "收下"
            return reply
        except HTTPException as refused:
            outcome = f"拒了（{'、'.join(refused.detail) if isinstance(refused.detail, dict) else '请求不对'}）"
            raise
        except Exception as error:
            outcome = f"出错（{type(error).__name__}）"
            raise
        finally:
            trace.finish(outcome)

    def submit_in_thread(body, request, trace):
        trace.step("进线程")
        process_id = peer_of(request)
        if not body.command:
            raise HTTPException(status_code=400, detail="没给命令")
        if not os.path.isabs(body.cwd) or not os.path.isdir(body.cwd):
            raise HTTPException(status_code=400, detail=f"工作目录 {body.cwd} 不是在的绝对路径")
        carried, refusal = scheduler.submit_refusal(body.command, body.cwd, body.environment, body.whole_package_confirmed, body.full_gate)   # 提交时拒的全部当场拒、不进库（文件头 ②「提交时拒」）
        trace.step("提交时拒")
        if refusal is not None:
            raise HTTPException(status_code=400, detail={refusal[0]: refusal[1]})
        session = session_of(process_id)
        manual_problem = scheduler.manual_refusal(body.manual, session)
        trace.step("认会话")
        if manual_problem is not None:
            raise HTTPException(status_code=400, detail={"placement": manual_problem})
        task = scheduler.submit(body.command, body.cwd, carried, session, process_id, {} if body.manual else body.inbox, whole_package=body.whole_package_confirmed,
                                full_gate=body.full_gate, trace=trace)
        trace.task_id = task["id"]
        reply = {"id": task["id"], "session": task["session"], "state": task.get("state"),
                 "sharded": task.get("sharded_entry", ""), "compiles": task.get("compiles", False), "inbox": bool(task.get("inbox")),
                 "warnings": scheduler.submit_warning_lines(task)}   # 提交时警告（文件头 ②「提交时警告」）：客户端打到标准错误
        trace.step("提交时警告")
        if task.get("state") == lib.AWAITING_CONFIRMATION_STATE:   # 交任务的二次确认（文件头 ⑦）：确认单与确认的时限
            reply.update(confirmation=scheduler.confirmation(task), confirm_timeout_seconds=scheduler.confirm_timeout_seconds)
            trace.step("确认单")
        return reply

    class ConfirmBody(BaseModel):
        items: int
        tasks: int

    @application.post("/tasks/{task_id}/confirm")
    def confirm(task_id: int, body: ConfirmBody, request: Request):
        process_id = peer_of(request)
        status, reply = scheduler.confirm(task_id, session_of(process_id), process_id, body.items, body.tasks)
        if status != 200:
            raise HTTPException(status_code=status, detail=reply.get("detail", reply))
        return reply

    @application.post("/tasks/{task_id}/promote")
    def promote(task_id: int, request: Request):
        process_id = peer_of(request)
        status, reply = scheduler.promote(task_id, session_of(process_id), process_id)
        if status != 200:
            raise HTTPException(status_code=status, detail=reply.get("detail", reply))
        return reply

    class LongRunConfirmBody(BaseModel):
        why: str

    @application.post("/tasks/{task_id}/long-run-confirm")
    def confirm_long_run(task_id: int, body: LongRunConfirmBody, request: Request):
        process_id = peer_of(request)
        status, reply = scheduler.confirm_long_run(task_id, session_of(process_id), process_id, body.why)
        if status != 200:
            raise HTTPException(status_code=status, detail=reply.get("detail", reply))
        return reply

    @application.get("/tasks/{task_id}")
    def show(task_id: int, request: Request):
        peer_of(request)
        found = scheduler.summary(task_id)
        if found is None:
            raise HTTPException(status_code=404, detail=f"没有任务 {task_id}")
        return found

    @application.get("/tasks/{task_id}/wait")
    async def wait(task_id: int, request: Request, timeout: float = 20.0):
        process_id = peer_of(request)
        session = session_of(process_id)
        deadline = time.monotonic() + max(0.0, min(timeout, WAIT_LIMIT_SECONDS))
        loop_holder.setdefault("loop", asyncio.get_running_loop())
        while True:
            event = asyncio.Event()   # 先登记、再查库：查完到登记之间记下的结局不会漏掉
            waiters.setdefault(task_id, []).append(event)
            try:
                found = await run_in_threadpool(store.task, task_id)   # 读库、核签名放进线程池：事件循环上收提交与别的请求，不让一群等结局的轮着读库挡住它
                if found is None:
                    raise HTTPException(status_code=404, detail=f"没有任务 {task_id}")
                task, valid = found
                if not valid or task.get("state") == "finished":
                    if valid and session == task.get("session"):
                        await run_in_threadpool(scheduler.mark_collected, task_id)
                    summary = await run_in_threadpool(scheduler.summary, task_id)
                    reply = dict(summary, finished=True)
                    if valid and not lib.broken("attention-finished-unreported", scheduler.environment):   # 结局有了还没报的提醒（出结局时才判的那几种）：打在理由行前面，退码仍是任务的
                        reply["attention"] = await run_in_threadpool(scheduler.unreported_attention, task_id, session)
                    return reply
                suspect = await run_in_threadpool(scheduler.unreported_suspect, task_id, session)
                if suspect is not None:   # 疑似卡住（文件头 ①「疑似卡住」）：任务照跑，wait 打一行、退出；一段只报一次
                    return {"finished": False, "state": task.get("state"), "suspect": suspect}
                attention = await run_in_threadpool(scheduler.unreported_attention, task_id, session)
                if attention:   # 要处置的提醒（文件头 ①「要处置的提醒」）：任务照旧在排或在跑，wait 打一行、退出；同一种同一道提醒门槛只报一次
                    return {"finished": False, "state": task.get("state"), "attention": attention}
                if time.monotonic() >= deadline:
                    return {"finished": False, "state": task.get("state")}
                with contextlib.suppress(asyncio.TimeoutError):
                    await asyncio.wait_for(event.wait(), timeout=max(0.01, min(wait_fallback, deadline - time.monotonic())))
            finally:
                with contextlib.suppress(ValueError, KeyError):
                    waiters[task_id].remove(event)

    @application.post("/tasks/{task_id}/cancel")
    def cancel(task_id: int, request: Request):
        process_id = peer_of(request)
        status, reply = scheduler.cancel(task_id, session_of(process_id), process_id)
        if status != 200:
            raise HTTPException(status_code=status, detail=reply.get("detail", reply))
        return reply

    @application.post("/tasks/{task_id}/clean")
    def clean(task_id: int, request: Request):
        process_id = peer_of(request)
        status, reply = scheduler.clean(task_id, session_of(process_id), process_id)
        if status != 200:
            raise HTTPException(status_code=status, detail=reply.get("detail", reply))
        return reply

    class PinBody(BaseModel):
        command: List[str]
        pinned: bool

    @application.post("/pins")
    def pin(body: PinBody, request: Request):
        process_id = peer_of(request)
        if not body.command:
            raise HTTPException(status_code=400, detail="没给命令")
        reply = scheduler.pin_cap(body.command, body.pinned, session_of(process_id), process_id)
        if reply.get("refused"):   # 钉值不大于 0（文件头 ②「内存」的钉值）：control 照 HTTP 状态打回、把这一句与出路打出来
            raise HTTPException(status_code=422, detail=reply["refused"])
        return reply

    class PinQuery(BaseModel):
        command: List[str]

    @application.post("/pins/show")
    def show_pin(body: PinQuery, request: Request):
        peer_of(request)
        return scheduler.pins_reply(scheduler.command_key(body.command, None))

    class PauseBody(BaseModel):
        reason: str = ""
        allow_session: str = ""

    @application.post("/pause")
    def pause_queue(body: PauseBody, request: Request):
        process_id = peer_of(request)   # 只认服务自己的 uid：同别的写操作
        return scheduler.pause_service(" ".join(body.reason.split())[:PAUSE_REASON_CHARACTERS], body.allow_session.strip(), session_of(process_id), process_id)

    @application.post("/resume")
    def resume_queue(request: Request):
        process_id = peer_of(request)
        return scheduler.resume_service(session_of(process_id), process_id)

    @application.post("/wake")
    def wake(request: Request):
        peer_of(request)   # 只认经 unix socket、同一 uid 来的（attempt_runner.py 写完结果调它）
        scheduler.wake()
        return {"ok": True}

    @application.get("/sessions/self")
    def mine(request: Request):
        process_id = peer_of(request)
        session = session_of(process_id)
        return {"session": session, "tasks": scheduler.session_tasks(session)}

    return application


def peer_credentials_protocol():
    from uvicorn.protocols.http.h11_impl import H11Protocol

    class PeerCredentialsProtocol(H11Protocol):
        """连上时读 SO_PEERCRED，把 (uid-<uid>, 进程号) 放进 scope 的 client（接口那一侧 peer_of 读它）。"""

        def connection_made(self, transport):
            super().connection_made(transport)
            connection = transport.get_extra_info("socket")
            try:
                raw = connection.getsockopt(socket.SOL_SOCKET, socket.SO_PEERCRED, struct.calcsize("3i"))
                process_id, uid, _ = struct.unpack("3i", raw)
                self.client = (f"uid-{uid}", process_id)
            except (OSError, AttributeError):
                self.client = None
    return PeerCredentialsProtocol


def bind_socket(path):
    """绑 unix socket（600）；同一路径上已有一份在回话交 None；路径太长或绑不上抛 lib.Unavailable（点名那个路径）。"""
    problem = lib.socket_path_problem(path)
    if problem:
        raise lib.Unavailable(problem, "把多机配置里的 TASK_SERVER_RUNTIME_DIRECTORY 改到短一点的目录（task_server_lib.py 文件头「路径」）")
    if os.path.exists(path):
        probe = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        probe.settimeout(2)
        try:
            probe.connect(path)
            probe.close()
            return None
        except OSError:
            os.remove(path)
    listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    try:
        listener.bind(path)
    except OSError as error:
        listener.close()
        raise lib.Unavailable(f"socket {path} 绑不上（{type(error).__name__}: {error}）",
                              "看那个目录在不在、属主与权限对不对；路径太长的照 task_server_lib.py 文件头「路径」把 TASK_SERVER_RUNTIME_DIRECTORY 写短") from None
    os.chmod(path, 0o600)
    listener.listen(128)
    return listener


PROCESS_NAME = b"task-server"   # 服务主线程的进程名（/proc/<pid>/comm）：两台 earlyoom 的 --avoid 照这个名字认它、整机内存吃紧时不挑它


def name_this_process():
    """把服务主线程的进程名设成 PROCESS_NAME（prctl PR_SET_NAME）；设不成只少这一层免杀，不挡服务起。"""
    with contextlib.suppress(OSError, AttributeError):
        import ctypes
        ctypes.CDLL(None, use_errno=True).prctl(15, PROCESS_NAME, 0, 0, 0)   # 15 = PR_SET_NAME


def serve():
    lib.refuse_selftest_variables_or_continue()
    name_this_process()
    environment = dict(os.environ)
    try:
        lib.require_modules(environment)
        missing_tools = [tool for tool in ("systemctl", "systemd-run") if shutil.which(tool) is None]
        if missing_tools:
            raise lib.Unavailable(f"PATH 上没有 {'、'.join(missing_tools)}", "装 systemd（用户级 systemd 起每一次尝试的单元）")
        configuration = lib.read_configuration(environment)
        key = lib.read_signing_key(configuration.signing_key_file)
        threads = registered_threads(environment)
        multiplexing_problem = lib.enable_ssh_multiplexing(environment, configuration.state_directory)   # 服务起的 ssh 一律复用连接（task_server_lib.py 文件头「主机」）
        if multiplexing_problem:
            say(f"  ! {multiplexing_problem}")
        nfs_report = lib.require_nfs_share(configuration, environment, probe=True)   # 没有 NFS 共享不起（task_server_lib.py 文件头「NFS 共享」），在 Scheduler 建起之前
        lib.shared_data_root(environment)   # 共享数据根与工作根取不到（多机配置没写 SHARED_DATA_ROOT 这类）不起、退 78，出路点名那一行；Scheduler 照同一套取
        lib.work_root()
    except lib.Unavailable as error:
        lib.report_unavailable_and_exit(error, "task-server 起不来")
    os.umask(0o077)
    for directory in (configuration.state_directory, configuration.runtime_directory, configuration.logs_directory, configuration.attempts_directory,
                      configuration.shards_directory, configuration.scenes_directory, configuration.build_directory):
        lib.ensure_private_directory(directory)
    import uvicorn
    try:
        store = Store(configuration.database, key, environment)
    except lib.Unavailable as error:   # 库里有违反业务唯一的行（task_server_lib.py 文件头「库」）
        lib.report_unavailable_and_exit(error, "task-server 起不来")
    scheduler = Scheduler(configuration, store, environment, threads, nfs_report)
    try:
        listener = bind_socket(configuration.socket)
    except lib.Unavailable as error:
        lib.report_unavailable_and_exit(error, "task-server 起不来")
    if listener is None:
        say(f"  ✗ {configuration.socket} 上已有一份 task-server 在回话")
        say(f"  → 怎么办：只留一份；看 systemctl --user status {lib.UNIT_NAME}")
        return 3
    scheduler.revalidate_queued()   # 规则改了的：还在排的照现行提交规则再核，不合的打回提交方重发（文件头「排队」）
    scheduler.thread.start()
    scheduler.nfs_thread.start()
    scheduler.inbox_thread.start()   # 发消息给会话、盯服务健康（文件头 ⑥）
    scheduler.runaway_thread.start()   # 任务之外的进程空转（文件头 ⑥）
    scheduler.booking_audit_thread.start()   # 记账对账（文件头 ②「内存」的记账对账）
    if not scheduler.resources.fake_path:
        scheduler.peer_probe_thread.start()   # 第二台的余量（文件头 ②「第二台的余量不在排队线程里读」）
    threading.Thread(target=scheduler.sweep_finished_build_directories, name="task-server-build-sweep", daemon=True).start()   # 起来时清掉已有结局的任务留下的编译目录
    say(f"  · task-server 起了（进程 {os.getpid()}，socket {configuration.socket}，库 {configuration.database}，标签 {store.tag}；"
        f"NFS 共享根 {'、'.join(entry['root'] for entry in nfs_report['roots']) or '没判（弄坏开关 nfs-unchecked）'} 四项都过）")
    config = uvicorn.Config(build_application(scheduler), http=peer_credentials_protocol(), loop="asyncio", lifespan="off", log_level="warning",
                            access_log=False, timeout_graceful_shutdown=5)
    server = uvicorn.Server(config)
    try:
        server.run(sockets=[listener])
    finally:
        scheduler.stop()
        listener.close()
        with contextlib.suppress(FileNotFoundError):
            os.remove(configuration.socket)
        with contextlib.suppress(Exception):   # 复用的主连接逐个关掉（task_server_lib.py 文件头「主机」）；关不成的留到 ControlPersist 到点
            lib.close_ssh_masters(environment, configuration.path)
        say("  · task-server 停了（在跑的任务在各自的单元里接着跑）")
    return 0


def main(arguments):
    if arguments == ["serve"]:
        return serve()
    print("  ✗ task_server.py 用法：serve", file=sys.stderr)
    print(f"  → 怎么办：服务由用户级 systemd 单元 {lib.UNIT_NAME} 起（{lib.INSTALL_COMMAND}）；提交任务用 task-server/task submit -- <命令…>", file=sys.stderr)
    return lib.USAGE_EXIT


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
