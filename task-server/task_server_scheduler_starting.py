"""task_server.py 拆出的一份：Scheduler 的一部分方法：起尝试与起单元、提交、确认、取消、清理、摘要（Scheduler 在 task_server.py，经继承取到它们）。"""
import os
import sys
import task_server_lib as lib
import contextlib
import shlex
import shutil
import signal
import subprocess
import time
from task_server_constants import (
    ACTIVE_ATTEMPT_STATES, CANCEL_SETTLE_POLL_SECONDS, LISTING_ONLY_ARGUMENTS, LOCAL, LONG_RUN_IDLE_CORE_FRACTION, MAXIMUM_LAUNCHER_DEPTH, PEER,
    RECORD_WINDOW, SELFTEST_PAUSE_SECONDS, SELF_CGROUP_PATH, START_TIMEOUT_SECONDS, UNIT_STOP_SECONDS, append_log_lines, keyed_lock, nofile_property,
    say, write_json_atomically,
)
from task_server_store import StoreRefused
from task_server_machines import BuildDirectory, PassView, unit_processes, unit_states


class SchedulerStarting:
    """Scheduler 的一部分方法：起尝试与起单元、提交、确认、取消、清理、摘要（Scheduler 在 task_server.py，经继承取到它们）。"""

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
        build_directory, made_here, reused, slot = "", True, False, None
        if self.server_gives_build_directory(task):
            # task_server_lib.py 文件头「编译目录」：两台同一段，拿这棵树在这一台的组里一个（take_build_slot）；本机经 CARGO_TARGET_DIR 指过去，
            # 第二台由 multi-host-run.sh 在它本地盘上新开、或照 --reuse-build-directory 接着用组里已有的（PeerMachine.command）。组满、都占着的这一次不起，留在队里下一轮再判
            slot, reused = self.take_build_slot(task, placement.host, label, number)
            if slot is None:
                shutil.rmtree(directory, ignore_errors=True)
                say(f"  · 任务 {task['id']} 这一次不起：这棵树在{lib.HOST_NAMES.get(placement.host, placement.host)}的编译目录组满、都占着")
                return None
            build_directory, made_here = slot.get("directory") or "", machine.name == LOCAL
        try:   # 拿到之后出了什么事（建目录失败也算）都放掉占着（release_build_slot_reservation）
            if slot is not None and made_here:
                os.makedirs(build_directory, mode=0o700, exist_ok=True)
                environment["CARGO_TARGET_DIR"] = os.path.join(build_directory, "target")
            return self.start_attempt_with_build_directory(task, placement, label, number, directory, log, environment, unguarded, shard_address,
                                                           exit_peak_file, scene_directory, peer_cleanup_file, BuildDirectory(build_directory, reused) if build_directory else None,
                                                           made_here, slot)
        finally:
            if slot is not None:
                self.release_build_slot_reservation(slot["id"])

    def start_attempt_with_build_directory(self, task, placement, label, number, directory, log, environment, unguarded, shard_address,
                                           exit_peak_file, scene_directory, peer_cleanup_file, build, made_here, slot):
        """start_attempt 拿到编译目录（或不给）之后的那一段：写说明、插尝试、起单元（文件头 ③「起」）；slot 是组里那一行（不给编译目录的是 None），
        这一段跑完 start_attempt 放掉它的占着。→ 同 start_attempt。"""
        build_directory, reused = (build.directory, build.reused) if build is not None else ("", False)
        shard_output = self.shard_environment(task, placement, label, number, environment, shard_address) if task.get("role") in lib.SHARD_ROLES else ""
        argv = self.attempt_command(task, placement, label, environment, build)
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
                   "build_slot": slot["id"] if slot is not None else None, "build_tree": slot.get("tree") if slot is not None else "", "build_directory_reused": reused,
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
                shutil.rmtree(directory, ignore_errors=True)   # 备好的这一次没起：它自己的尝试目录删掉；编译目录归这棵树的组，留着（task_server_lib.py 文件头「编译目录」）
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
        """命令键（task_server_lib.py 文件头「资源记录」的命令键）：先剥掉不改跑法的环境变量前缀与临时 prune-server 的包装（lib.key_words），cargo test 只留包、特性与档（lib.cargo_test_class_words），本仓门禁与入口脚本的命令词折成仓内相对路径、--check、--item、--only 去值归同类（lib.selection_class_words），
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
        # 交任务的二次确认（文件头 ⑦）：判成长任务的先记待确认、不进队、不建子任务；confirm 两个数对得上才照 awaiting_state 进队（照提交时刻的任务号排位置）；
        # 不是长任务的直接进队、任务行记 direct_queue
        if whole_package:
            task["whole_package_confirmed"] = whole_package
        if full_gate:
            task["full_gate"] = True
        long_reason, short_reason = self.long_task_judgement(task)
        awaiting = bool(long_reason) and not lib.broken("confirm-skipped", self.environment)
        if awaiting:
            task.update(awaiting_state=task["state"], state=lib.AWAITING_CONFIRMATION_STATE, long_reason=long_reason)
        else:
            task["direct_queue"] = short_reason or "弄坏开关 confirm-skipped：判成长任务的也不等确认"
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

    def long_task_judgement(self, task):
        """文件头 ⑦「只有长任务要确认」：交的这一条判不判成长任务 → (判成长任务的依据, 不是长任务的依据)，两句里只有一句非空。task 是还没入库的任务字典
        （command_key、sharded_entry、full_gate、whole_package_confirmed 已写好）。判据四条，照下面的次序判、先中的就算：库里这把命令键近 RECORD_WINDOW 次跑到退出的用时最大的超过
        lib.LONG_TASK_SECONDS；不指定范围的整趟（范围声明种类 whole、--full-gate、--whole-package-confirmed）；分片；没有用时记录又是会编译的 cargo 测试或按项跑的门禁（--item）。
        自证的小世界（lib.CONFIRM_RULE_SELFTEST_VARIABLE 写 all）与弄坏开关 short-task-held 下，没判成长任务的也判成长任务（依据写不分长短一律待确认）。
        弄坏开关 long-task-record-unread、long-task-latest-only、long-task-whole-unjudged、long-task-shard-unjudged、long-task-unrecorded-unjudged 各去掉或改窄一条判据。"""
        environment = self.environment
        runs = self.completed_runs(task)
        window = [seconds for _, seconds, _ in runs[:RECORD_WINDOW]]
        longest = (window[0] if lib.broken("long-task-latest-only", environment) else max(window)) if window else 0.0
        if window and longest > lib.LONG_TASK_SECONDS and not lib.broken("long-task-record-unread", environment):
            return lib.LONG_TASK_RECORD_REASON.format(count=len(window), seconds=longest, limit=lib.LONG_TASK_SECONDS), ""
        marks = [mark for mark, hit in (("范围声明种类是 whole", lib.declared_scope_kind(task.get("environment") or {}, task["command"]) == lib.WHOLE_SCOPE_KIND),
                                        ("带 --full-gate", bool(task.get("full_gate"))), ("带 --whole-package-confirmed", bool(task.get("whole_package_confirmed")))) if hit]
        if marks and not lib.broken("long-task-whole-unjudged", environment):
            return lib.LONG_TASK_WHOLE_REASON.format(marks="、".join(marks)), ""
        if task.get("sharded_entry") and not lib.broken("long-task-shard-unjudged", environment):
            return lib.LONG_TASK_SHARD_REASON.format(entry=task["sharded_entry"]), ""
        if not window and not lib.broken("long-task-unrecorded-unjudged", environment):
            tests = any(run.subcommand in lib.CARGO_TEST_SUBCOMMANDS for run in self.cargo_runs(task))
            items = any(name == "--item" for name, _ in lib.named_scope(task["command"], environment))
            if tests or items:
                return lib.LONG_TASK_UNRECORDED_REASON.format(what=lib.LONG_TASK_UNRECORDED_TESTS if tests else lib.LONG_TASK_UNRECORDED_ITEMS), ""
        if lib.selftest_override(lib.CONFIRM_RULE_SELFTEST_VARIABLE, environment) == "all":
            return lib.LONG_TASK_HELD_REASON.format(note="自证的小世界："), ""
        if lib.broken("short-task-held", environment):
            return lib.LONG_TASK_HELD_REASON.format(note="弄坏开关 short-task-held："), ""
        if window:
            return "", lib.SHORT_TASK_RECORD_REASON.format(count=len(window), seconds=longest, limit=lib.LONG_TASK_SECONDS)
        return "", lib.SHORT_TASK_UNRECORDED_REASON

    def enter_queue(self, task):
        """这条任务进队（确认过的，或没判成长任务、或弄坏开关 confirm-skipped 下交了就进）：分片的先建计划那一步（文件头 ④）。"""
        if task.get("state") == "sharded":
            self.crash_point("parent-inserted")
            self.create_child(task, "plan")
        self.wake()

    def confirmation(self, task):
        """待确认那一条的确认单（文件头 ⑦）→ {items, tasks, lines}：items 是命令里点名的项数（lib.named_scope），tasks 是这个会话此刻在排在跑的父任务条数（不算它自己，
        也不算别的待确认的），lines 是给人看的几行：整条命令、判成长任务的依据（任务行的 long_reason；弄坏开关 long-task-reason-unsaid 下不写）、范围声明四段原文、项数、崩溃用例数与域（lib.crash_scope_text，不是崩溃入口又没点的不打）、在排在跑几条与其中命令键相同或点的项交叠的逐条列号、预测内存与记录来源、
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
            memory = (f"内存上限 {lib.mebibytes_text(int(prediction.memory_cap or 0))}（这把键记录的峰值 {human(int(prediction.memory_recorded_peak or 0))}）"
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
                 *([f"  判成长任务：{task['long_reason']}"] if task.get("long_reason") and not lib.broken("long-task-reason-unsaid", self.environment) else []),
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
        直接进队的任务（任务行记着 direct_queue）不用确认，不论此刻在排、在跑还是已有结局都回 200「已在队里，不用确认」、不核两个数。
        弄坏开关 confirm-counts-unchecked 下不核两个数，confirm-direct-refused 下对直接进队的回 409。"""
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
            if task.get("direct_queue") and not lib.broken("confirm-direct-refused", self.environment):   # 直接进队的（没判成长任务）：不用确认，照旧认得、回 200
                return 200, {"confirmed": False, "state": task.get("state"), "detail": f"任务 {task_id} 已在队里，不用确认（直接进队：{task['direct_queue']}）"}
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
        """→ (HTTP 状态码, 回话)。删这条任务（分片的连同子任务）自己的现场，分片的父任务另删计划目录下的测试二进制（shard_binaries_removal）：
        只许提交它的会话、有了结局才删（task_server_lib.py 文件头「现场」，文件头 ④）。编译目录不删：它归这棵树的组，树删了才跟着删
        （task_server_lib.py 文件头「编译目录」）。子任务或尝试的记录校验不过（关联列改过这类，TS-05）的一份都不删。
        弄坏开关 clean-removes-build-slot 下照删各次尝试用过的组里那一个（本机就地删、第二台经 Machine.remove_build_directory）、组里删掉那一行。"""
        found = self.store.task(task_id)
        if found is None:
            return 404, {"detail": f"没有任务 {task_id}"}
        task, valid = found
        if not valid:
            self.tampered(task_id, task, "任务记录")
            return 409, {"detail": f"任务 {task_id} 的记录校验不过，已记 record-tampered，不认这次删除"}
        if task.get("session") != session and not lib.broken("clean-session-unchecked", self.environment):
            self.operation("clean-refused", task_id, session, process_id, "不是提交它的会话")
            return 403, {"detail": f"任务 {task_id} 不是这个会话提交的（提交它的会话与这一次的不同），不许删它的现场"}
        if task.get("state") != "finished":
            return 409, {"detail": f"任务 {task_id} 还没有结局（{task.get('state')}）：等 wait 拿到结局再 clean"}
        try:
            owners = [task] + self.children_of(task_id)
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            return 409, {"detail": self.children_refused(task_id, refused, "一份现场都不删")}
        attempts = [(record, record_valid) for owner in owners for record, record_valid in self.store.attempts_of(owner["id"])]
        unverified = [record.get("id") for record, record_valid in attempts if not record_valid]
        if unverified:
            self.alert("record-tampered", task_id, f"任务 {task_id} 名下有校验不过的尝试记录（行 {unverified}）：一份现场都不删",
                       "查是谁直接写了库（只有 task-server 读写它）")
            return 409, {"detail": f"任务 {task_id} 名下有校验不过的尝试记录（行 {unverified}，关联列与正文对不上或写进之后被改过）：一份都不删"}
        removed, left = [], []
        if lib.broken("clean-removes-build-slot", self.environment):
            self.clean_build_slots_of(attempts, removed, left)
        if not lib.broken("scene-left-by-clean", self.environment):   # 各次尝试的现场一并删（task_server_lib.py 文件头「现场」）
            scenes_removed, scenes_left = self.remove_scenes(task_id)
            removed += scenes_removed
            left += scenes_left
        if task.get("sharded_entry"):   # 分片的父任务：计划目录下预先编好的测试二进制一并删（文件头 ④）
            binaries_removed, binaries_left = self.shard_binaries_removal(task_id, session, process_id)
            removed += binaries_removed
            left += binaries_left
        # 没删成的写明是哪一份、为什么：操作日志与回话都带；弄坏开关 clean-failure-unexplained 下只记份数
        explained = not lib.broken("clean-failure-unexplained", self.environment)
        reasons = "；".join(f"{item['kind']} {item['path']}：{item['why']}" for item in left)
        self.operation("clean", task_id, session, process_id, f"删了 {len(removed)} 份、没删成 {len(left)} 份" + (f"：{reasons}" if left and explained else ""))
        if left:
            return 409, {"detail": {"text": reasons if explained else f"没删成 {len(left)} 份", "removed": removed, "left": left if explained else []}}
        return 200, {"removed": removed, "left": []}

    def clean_build_slots_of(self, attempts, removed, left):
        """只给弄坏开关 clean-removes-build-slot：task clean 照删这条任务各次尝试用过的组里那一个，删了的组里删掉那一行（改坏的就是「clean 不删整组」那一条）。"""
        with self.build_pool_lock:
            slots = {attempt.get("build_slot"): attempt for attempt, _ in attempts if attempt.get("build_slot")}
            for slot_id, attempt in slots.items():
                machine = self.machines.get(attempt.get("host") or LOCAL) or self.machines[LOCAL]
                directory = attempt.get("build_directory") or ""
                done, lines = machine.remove_build_directory(directory)
                if done:
                    self.store.delete_row("build_slots", slot_id)
                    removed.append({"path": directory, "bytes": 0})
                else:
                    left.append({"path": directory, "bytes": 0, "kind": "编译目录", "why": "；".join(line.strip() for line in lines) or "删完还在"})

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
        # 取消的不留产物（文件头「接口」的取消）：在起、在跑的是哪几次在停之前记（第二台那一趟的本地目录照它核）
        before = (self.cancel_artifact_sizes(task_id) if task.get("state") != "finished" or task.get("sharded_entry")
                  else {"active": set()})
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
