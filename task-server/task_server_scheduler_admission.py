"""task_server.py 拆出的一份：Scheduler 的一部分方法：挑机器、内存预算与记账、资源记录与预测、钉值、磁盘与线程与 CPU 闸（Scheduler 在 task_server.py，经继承取到它们）。"""
import os
import task_server_lib as lib
import dataclasses
import math
import shlex
import subprocess
import time
from task_server_constants import (
    ACTIVE_ATTEMPT_STATES, BOOKED_FROM_CAP, BOOKED_FROM_USAGE, BOOKED_WHILE_YOUNG, BOOKED_WITHOUT_SCOPE, BREAK_FIXED_MARGIN_BYTES,
    BUILD_DISK_FLOOR_BYTES, BUILD_POOL_FULL_PREFIX, CPU_GATE_THREADS_COLUMN, CPU_MEASURED_AFTER_SECONDS, CPU_OVERSUBSCRIPTION,
    CPU_OVERSUBSCRIPTION_LOOSE, CPU_SETTLE_BREAK_SECONDS, CPU_SMALL_SLOT_TEXT, CPU_SMALL_TASK_CORES, CPU_SMALL_TASK_SLOTS, CPU_UNREAD_TEXT,
    DISK_FLOOR_KIND, DISK_REPORT_KEY, IDLE_SHORT_TEXT, IO_PRESSURE_REPORT_KEY, JUDGE_HOST_MEMORY_PREFIX, LOCAL, MACHINE_MEMORY_LINE_FRACTION,
    MACHINE_MEMORY_LINE_TEXT, MEMORY_WAIT_TOP_RUNNING, PEER, RECORD_WINDOW, RESTORED_HOST_LISTS, RUNNING_BOOKED_SUM_TEXT,
    RUNNING_BOOKING_MARGIN_BREAK_BYTES, RUNNING_PEAK_SAMPLE_SECONDS, SIGNAL_EXIT_CODES, SIZE_UNITS, SLICE_CAP_VALUE, SLICE_FLOOR_TEXT,
    THREADS_OVER_REGISTERED_FACTOR, THREADS_OVER_REGISTERED_ROUNDS, WHOLE_FRACTION, WORKSPACE_WRITER_FLOOR_BYTES, cpu_reading, earlier_reading,
    keyed_lock, say,
)
from task_server_machines import MACHINE_ORDER, Placement, gibibytes_text, unit_processes, unit_states


class SchedulerAdmission:
    """Scheduler 的一部分方法：挑机器、内存预算与记账、资源记录与预测、钉值、磁盘与线程与 CPU 闸（Scheduler 在 task_server.py，经继承取到它们）。"""

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

    def note_idle_short(self, task, placement):
        """这一条在每一台上都起不来（placement.idle_short）：把一项 booking-beyond-idle-machine 记进 idle_short_notices（同一条只记一次；attention_of_record 照它报，
        提醒门槛 1、只推一回），排队原因里已写各台各是什么。弄坏开关 idle-short-silent 下不记。"""
        if task["id"] in self.idle_short_notices or lib.broken("idle-short-silent", self.environment):
            return
        self.idle_short_notices[task["id"]] = self.attention_item(
            "booking-beyond-idle-machine", 1, task["id"], placement.idle_short,
            "它等的是整机自己空下来，不是排在前面的任务；服务不让它挡后面的，后面放得下的照起")

    def note_idle_short_judged(self, task, placement):
        """排队那一轮对这一条的「在每一台上都起不来」那一判有了结论就把它记进 idle_short_judged：放得下的（start），或放不下而 idle_short_text 真对每一台判过的（placement.idle_short_settled，
        不管判出的是不是在每一台上都起不来）；这一轮有一台判不成的撤掉。排队线程里别的原因给它写了排队原因（waiting_reasons；排在排满时限的任务后面、这一轮判不了放不放得下）不算。
        attention_of_record 在排的 oversized-budget 要等这一项：先记 idle_short_notices（note_idle_short 或删掉）、后记这一项，读的一侧先读这一项、后读 idle_short_notices。"""
        if placement.kind == "start" or placement.idle_short_settled:
            self.idle_short_judged.add(task["id"])
        else:
            self.idle_short_judged.discard(task["id"])

    def budget_source(self, task, prediction):
        """记账从哪来（budget_check 的依据）：照预测的 memory_history 说一句，照记录的另写近几次里用了几次。"""
        human = self.host_dispatch.human
        predicted = human(int(prediction.memory_recorded_peak or 0))
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
            prediction = dataclasses.replace(prediction, memory_recorded_peak=value, memory_cap=value, memory_booked=value)
        # 收到这台总上限减这台此刻的余量为止：超过总上限 scope 管不住它、撞的是整个 slice；slice 里挂着的那一截回收不掉，记满一整台永远放不下
        cap = min(int(prediction.memory_cap or 0), limit)
        booked = max(int(prediction.memory_booked or 0), cap)
        predicted = int(prediction.memory_recorded_peak or 0)
        if predicted > limit and prediction.memory_history != lib.PINNED_HISTORY and lib.broken("booking-over-host-limit", self.environment):
            return dataclasses.replace(prediction, memory_cap=cap, memory_booked=booked)   # 弄坏开关：峰值与记账不收，judge_host 判这台永远放不下
        return dataclasses.replace(prediction, memory_recorded_peak=min(predicted, limit), memory_cap=cap, memory_booked=min(booked, limit))

    def hosts_short_of_record(self, prediction, hosts, snapshot):
        """预测峰值比哪几台的可用上限大、而别处可能装得下（文件头 ②「内存」的准入记账）：→ {台: 这台不派的那一句}。
        别处可能装得下 = hosts 里有可用上限（host_limit）不小于预测峰值的台、有这一轮给不出可用上限的台，或多机配置里有一台此刻连不上（它回来之后可能装得下，place 照它排着）：
        比预测峰值小的那几台照永远放不下判、不派，任务去装得下的那台或排着等。没有可能装得下的别处时交空表：每一台都照收到可用上限的数判（booked_at_cap），
        不因为记录比整台大就判永远放不下，收到之后还撞上限的照 retry_decision 打回 memory-cap-at-maximum。钉值不在这里判（钉值就是判的那一台那一项）。
        弄坏开关 record-over-every-host-never-fits 下没有可能装得下的别处时也交那几台（每一台都比预测峰值小就判永远放不下）。"""
        predicted = int(prediction.memory_recorded_peak or 0)
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
        booked, predicted, cap = int(on_host.memory_booked or 0), int(on_host.memory_recorded_peak or 0), int(on_host.memory_cap or 0)
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
        if int(on_host.memory_recorded_peak or 0) <= room < int(on_host.memory_booked or 0):
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
        放得下的再判整机内存线（machine_memory_problem）、编译目录占盘（build_disk_problem）、编译目录组（build_pool_problem）与 CPU 闸（cpu_problem），
        哪一样放不下判决就是放不下、原因写那一样。
        排队先后的两条规则不在这里判（place_queued_into）。/health 不另判，只读派活给排头的结论（starved_problem）。只对 host_limit 交了数的台调。
        pending_memory 是调用方已读的这台待涨量（view.pending_of 交的那一个），没给就现读。"""
        if pending_memory is None:
            pending_memory = view.pending_of(host)   # 服务账上与派发器账上还没涨到量的，连同这一轮起了的
        on_host, verdict = self.judged_on_host(task, prediction, host, snapshot, pending_memory)
        verdict = self.slice_floor_verdict(verdict, on_host, host, snapshot)
        for why in (self.machine_memory_problem(host, snapshot), self.build_disk_problem(task, host, snapshot, view), self.build_pool_problem(task, host),
                    self.cpu_problem(task, host, snapshot), self.io_pressure_problem(task, host, snapshot)):
            if why and verdict.fits:   # 内存放得下、占盘、编译目录组或 CPU 放不下
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
        if not window and (floor is None or floor <= int(base.memory_recorded_peak or 0)):
            return None
        predicted = max([value for _, value in window] + ([floor] if floor is not None else []))
        cap = math.ceil(predicted / lib.MEBIBYTE) * lib.MEBIBYTE
        return dataclasses.replace(base, memory_recorded_peak=predicted, memory_cap=cap, memory_booked=predicted, memory_history=lib.RECORD_HISTORY)

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
            self.idle_short_notices.pop(task_id, None)
            self.idle_short_judged.discard(task_id)

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
        要放得下这条命令的占盘预测（接着用组里那一个的，扣掉它已占的盘：free_build_slot_bytes）加 BUILD_DISK_FLOOR_BYTES；放不下交排队原因（点名占盘），
        放得下、不会编又没量过、余量读不到的交空串。
        弄坏开关 build-disk-unpredicted 下一律空串（改前只看第二台的下限）。"""
        if lib.broken("build-disk-unpredicted", self.environment):
            return ""
        available = (snapshot.reports.get(host) or {}).get(DISK_REPORT_KEY)
        if not isinstance(available, int) or isinstance(available, bool):
            return ""
        predicted = self.build_disk_prediction(task)
        if not predicted and not task.get("compiles"):
            return ""
        credit = self.free_build_slot_bytes(task, host) if predicted else 0   # 接着用组里那一个：它已占的盘不再算要涨的（文件头 ②「编译目录组」）
        pending = view.disk_pending_of(host)
        needed = max(0, predicted - credit) + BUILD_DISK_FLOOR_BYTES
        if available - pending >= needed:
            return ""
        reused = f"（接着用组里那一个，它已占 {gibibytes_text(credit)}，扣掉之后还要涨 {gibibytes_text(max(0, predicted - credit))}）" if credit else ""
        return (f"编译目录占盘：这条命令量过的编译目录峰值 {gibibytes_text(predicted)}{reused}加下限 {gibibytes_text(BUILD_DISK_FLOOR_BYTES)} 要 {gibibytes_text(needed)}，"
                f"{lib.HOST_NAMES.get(host, host)}盘此刻剩 {gibibytes_text(available)}、在跑的编译目录还要涨 {gibibytes_text(pending)}，放不下"
                "（等在跑的跑完；不用了的仓副本删掉，它的编译目录组在服务下一回扫树时跟着删）")

    def io_pressure_problem(self, task, host, snapshot):
        """这一条在这台上因磁盘压力放不放行（文件头 ②「磁盘压力」）：交排队原因，放行交空串。只管会编译的任务（task 的 compiles）：多机配置 lib.IO_PRESSURE_LIMIT_KEY 写了阈值，
        而这台报告里的 IO_PRESSURE_REPORT_KEY（/proc/pressure/io 的 full avg300，Resources.note_io_pressure 记）超过它的排着，原因点名磁盘压力、这一台、读数与阈值（status 的「在等：」照写）；
        没写阈值的不按磁盘压力排（不补缺省）；阈值写坏的，原因照写、会编译的这一轮不派；这台读不到读数的不按压力排。两台同一套判法，各读各的读数。
        弄坏开关 io-pressure-ignored 下一律放行，io-pressure-applies-to-all 下不会编译的也排，io-pressure-limit-defaulted 下没写阈值的补缺省（lib.io_pressure_limit），
        io-pressure-peer-reads-local 下第二台照本机的读数判，io-pressure-wait-unnamed 下排队原因不点名磁盘压力、这一台与读数。"""
        if lib.broken("io-pressure-ignored", self.environment):
            return ""
        if not task.get("compiles") and not lib.broken("io-pressure-applies-to-all", self.environment):
            return ""
        limit, problem = lib.io_pressure_limit(self.resources.configuration_path, self.environment)
        if problem:
            return f"磁盘压力：{problem}，会编译的任务这一轮不派"
        if limit is None:
            return ""
        measured_host = LOCAL if lib.broken("io-pressure-peer-reads-local", self.environment) else host
        reading = (snapshot.reports.get(measured_host) or {}).get(IO_PRESSURE_REPORT_KEY)
        if reading is None or reading <= limit:
            return ""
        if lib.broken("io-pressure-wait-unnamed", self.environment):
            return "等资源"
        return (f"磁盘压力：{lib.HOST_NAMES.get(host, host)}此刻 /proc/pressure/io 的 full avg300 是 {reading:.2f}%，超过多机配置 {lib.IO_PRESSURE_LIMIT_KEY} 登记的 {limit:g}%，"
                "这条命令会编译，等磁盘缓过来（在跑的编译写盘多时压力高）")

    def computed_prediction(self, task):
        """现算（扫服务自己的记录；不读内存峰值表）：文件头 ②「内存」。"""
        self.selftest_read(f"predict task={task['id']}")
        base = self.unrecorded_prediction(task)
        pinned = self.pinned_prediction(task, base)
        if pinned is not None:
            return pinned
        if task.get("next_cap"):
            cap = int(task["next_cap"])
            prediction = dataclasses.replace(base, memory_recorded_peak=cap, memory_cap=cap, memory_booked=cap, memory_history="retry")
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
        return dataclasses.replace(base, memory_recorded_peak=value, memory_cap=value, memory_booked=value, memory_history=lib.PINNED_HISTORY, memory_by_host=dict(caps))

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
        undecided = bool(unusable or held_back or down_text)   # 有一台这一轮判不成：idle_short_text 不判（下一轮可能放得下），这一轮不算对这一条判过
        idle_short = self.idle_short_text(task, prediction, verdicts, undecided, snapshot)
        if idle_short:
            reasons += f"；{IDLE_SHORT_TEXT}（{idle_short}）"
        return Placement("blocked", why=reasons, build_tree=self.pool_only_tree(task, verdicts), idle_short=idle_short, idle_short_settled=not undecided)

    def verdict_with_service_tasks_gone(self, prediction, host, snapshot):
        """这台上服务自己在跑的任务全收尾、放出它们占的内存之后，这一条放不放得下：→ (judge_host 的判决, 放出的字节数)（没有待涨量：在跑的都收尾了，没涨到的那一截不会再涨）。
        放出的 = 每一次在跑的尝试此刻回收不掉的用量（报告里它的 scope，report_scope_usage）；读不到它的 scope 的（刚起、读不了）取它的上限（它占的不会比上限多），
        往多里算放出的，宁可判成「等它们收尾会放得下」也不误判成起不来。整机空闲与 slice 放得下的各加上放出的那一截，别的照这一轮读的报告；记账照 booked_at_cap。
        只对 host_limit 交了数的台调。"""
        report = snapshot.reports[host]
        memory = report["memory"]
        usage = self.report_scope_usage(report)
        released = 0
        for attempt in self.booking_attempts():
            if attempt.get("host") != host:
                continue
            used = usage.get(attempt.get("label") or "")
            released += int(attempt.get("memory_cap") or 0) if used is None else used
        alone = dict(report, memory=dict(memory, free=memory["free"] + released, slice_room=memory["slice_room"] + released))
        verdict = self.host_dispatch.judge_host(host, alone, self.booked_at_cap(prediction, host, snapshot), False, 0, 0,
                                                snapshot.memory_caps.get(host), self.host_margin(host, snapshot)[0])
        return verdict, released

    def idle_short_text(self, task, prediction, verdicts, some_host_undecided, snapshot):
        """这一条在每一台上都起不来（文件头 ②「内存」的整机空闲不够）：→ 各台各一句、「；」接起来；不是这种交空串。
        每一台此刻都放不下，而且就算服务在那台跑的任务全收尾（verdict_with_service_tasks_gone）整机空闲或 slice 放得下的也不够要的、或那台本来就判永远放不下，
        才算：它等的不是别的任务，等的是整机自己空下来，排在前面的哪条跑完都帮不上它，所以不拿它挡后面的（place_queued_into）。
        分片的片不判（它的上限会照放得下的量压）；有一台这一轮给不出上限、查不到余量、盘不够或此刻连不上（some_host_undecided），下一轮可能放得下，不算。
        弄坏开关 idle-short-unjudged 下一律交空串。"""
        if (lib.broken("idle-short-unjudged", self.environment) or task.get("role") == "shard" or some_host_undecided or not verdicts
                or any(verdict.fits for verdict in verdicts.values())):
            return ""
        human = self.host_dispatch.human
        sentences = []   # 各台各一句
        for host, verdict in verdicts.items():
            name = lib.HOST_NAMES.get(host, host)
            if verdict.never:
                sentences.append(f"{name}：永远放不下")
                continue
            alone, released = self.verdict_with_service_tasks_gone(prediction, host, snapshot)
            if alone.fits:
                return ""
            sentences.append(f"{name}：服务在这台跑的任务全收尾、放出 {human(released)} 之后，{alone.why}")
        return "；".join(sentences)

    def pool_only_tree(self, task, verdicts):
        """这一条放不下，是不是放得下的每一台都只缺这棵树的编译目录名额（判决的原因以 BUILD_POOL_FULL_PREFIX 起头：内存、占盘放得下才轮到它）→ 那棵树的仓根；
        不是（有一台缺的是内存、CPU、占盘、磁盘压力，或没有放得下的台）交空串。永远放不下的台（never）不算：这一条本来就用不了它。"""
        open_verdicts = [verdict for verdict in verdicts.values() if not verdict.never]
        if not open_verdicts or not all(verdict.why.startswith(BUILD_POOL_FULL_PREFIX) for verdict in open_verdicts):
            return ""
        return lib.build_tree_root(task["cwd"])

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
        到此刻的最大照旧由 peak_segment_running_cores 记，只给收尾时的核数记录与要核数少的名额用。一段都没量满交 None。弄坏开关 cpu-latest-runnable-ignored 下不看可跑线程数，
        cpu-running-cores-over-registered 下不受登记数限制（这里与 peak_segment_running_cores 各限一回，两处都不限才红）。"""
        if self.peak_segment_running_cores(attempt, usage, elapsed, now, registered) is None:
            return None
        with self.running_usage_lock:
            cores = self.running_latest_cores.get(attempt.get("id"))
            runnable = self.running_latest_runnable.get(attempt.get("id"), 0)
        if cores is None:
            return None
        if lib.broken("cpu-latest-runnable-ignored", self.environment):
            runnable = 0
        measured = max(1, math.ceil(round(cores, 3)), runnable)
        return measured if lib.broken("cpu-running-cores-over-registered", self.environment) else min(max(registered, 1), measured)

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
