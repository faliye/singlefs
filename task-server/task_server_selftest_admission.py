"""task_server_selftest.py 拆出的一份：内存预算与记账、挑机器放不放得下、磁盘与 IO 压力闸的各格（总说明在 task_server_selftest.py 文件头）。"""
import os
import contextlib
import io
import threading
import time
import types
import unittest.mock
import task_server_lib as lib
from task_server_selftest_fakes import (
    AT_CAP_PEAK_BYTES, AT_CAP_PINNED_TEXT, AT_CAP_SEED_PEAK_BYTES, BLOCK_SCRIPT, GIBIBYTE, HOLD_UNTIL_RELEASED_SCRIPT, MEBIBYTE, PEAKS,
    PLACEMENT_WAIT_SECONDS, POLL_SECONDS, SEEDED_ENDED_AT, SELFTEST_LOCAL_ADDRESS, SELFTEST_MARGIN_GIBIBYTES, SELFTEST_PEER_ADDRESS, SERVER,
    SESSION_A,
)
from task_server_selftest_world import (
    ATTENTION_WAIT_SECONDS, INBOX_TOKEN_SAMPLE, LONG_RUN_WHY_SAMPLE, RUNNING_BOOKING_LABEL, SelftestFailure, attention_problems, build_slot_tree,
    describe, fake_cargo_extra, first_attempt, loaded_host_dispatch, memory_report, process_memory_stat, running_booking_scheduler, running_shards,
    seed_ended_attempt, shard_children,
)


# margin-measured-per-host：两台 slice 总上限都是 50G；记录峰值 48G 的那一条上限与记账都是 49152M（近五次最大，不乘 1.5）：
# 余量 1G 的台（可用上限 49G）装得下，余量 3G 的台（47G）峰值本身就装不下
PER_HOST_SMALL_MARGIN_GIBIBYTES = 1
PER_HOST_LARGE_MARGIN_GIBIBYTES = 3
PER_HOST_PEAK_BYTES = 48 * GIBIBYTE
PER_HOST_CAP_TEXT = "49152M"

# page-cache-not-counted-as-process-memory：假 scope 的 memory.current 与 memory.peak 钉在上限 1536M（库里记的峰值 1536M，上限照它），里面进程用的只有 anon 96M，
# 其余是页缓存与内核（任务 8853 的样子：anon 921M、file 22163M、kernel 650M，memory.peak 顶到上限 50176M）
PAGE_CACHE_ANON_BYTES = 96 * MEBIBYTE
PAGE_CACHE_KERNEL_BYTES = 32 * MEBIBYTE
PAGE_CACHE_FILE_BYTES = AT_CAP_PEAK_BYTES - PAGE_CACHE_ANON_BYTES - PAGE_CACHE_KERNEL_BYTES


def cell_page_cache_not_counted_as_process_memory(world):
    """页缓存撑满 scope 不算进程内存（attempt_runner.py 文件头 ③ ⑥、task_server.py 文件头 ①「顶在上限上卡住」）：假 scope 的 memory.stat 是 anon 96M、file 1408M、
    kernel 32M，memory.current 与 memory.peak 钉在上限 1536M，CPU 一直不涨。跑完的那一条记的峰值是进程用的 96M、不是 1536M；
    日志不涨的那一条（库里 1536M、上限 1536M）满卡住阈值照记 stalled、只一次尝试，不判顶在上限上卡住、不乘 1.5 重排。
    弄坏开关 memory-peak-with-page-cache 下照改前读 memory.peak 与 memory.current：记 1536M、判顶在上限上卡住重排，这一格红。"""
    scope = os.path.join(world.root, "page-cache-scope")
    os.makedirs(scope)
    for name, text in (("cpu.stat", "usage_usec 1000\n"), ("memory.peak", f"{AT_CAP_PEAK_BYTES}\n"), ("memory.current", f"{AT_CAP_PEAK_BYTES}\n"),
                       ("memory.stat", process_memory_stat(PAGE_CACHE_ANON_BYTES, PAGE_CACHE_FILE_BYTES, PAGE_CACHE_KERNEL_BYTES))):
        with open(os.path.join(scope, name), "w", encoding="utf-8") as handle:
            handle.write(text)
    world.seed_peak("selftest-page-cache-quiet", AT_CAP_SEED_PEAK_BYTES)
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    finished = world.finished(world.submit(["bash", "-c", "for i in 1 2 3; do echo tick $i; sleep 0.5; done", "page-cache-finished"],
                                           extra={lib.KEY_VARIABLE: "selftest-page-cache-finished"}), timeout=60)
    quiet = world.finished(world.submit(["bash", "-c", "echo started; sleep 30", "page-cache-quiet"], extra={lib.KEY_VARIABLE: "selftest-page-cache-quiet"}),
                           timeout=60)
    problems = []
    recorded = first_attempt(finished).get("memory_peak", "缺这一栏")
    if finished["task"].get("outcome") != "succeeded" or recorded != PAGE_CACHE_ANON_BYTES:
        problems.append(f"跑完的那一条应当记进程用的 {PAGE_CACHE_ANON_BYTES}（anon），不是 memory.peak 的 {AT_CAP_PEAK_BYTES}，实际 {describe(finished)}、记 {recorded}")
    seen = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in quiet["attempts"]]
    note = first_attempt(quiet).get("stall_note") or ""
    if seen != [(AT_CAP_PINNED_TEXT, "stalled")] or quiet["task"].get("outcome") != "stalled" or "顶在上限上卡住" in note:
        problems.append(f"日志不涨、页缓存撑满上限 {AT_CAP_PINNED_TEXT} 而进程只用 {PAGE_CACHE_ANON_BYTES} 的应当记 stalled、只一次尝试、说明不写「顶在上限上卡住」，"
                        f"实际各次 {seen}、{describe(quiet)}、说明「{note}」")
    if problems:
        raise SelftestFailure("页缓存不算进程内存：" + "；".join(problems), world.server_log_lines())
    return f"页缓存撑满上限 {AT_CAP_PINNED_TEXT}：跑完的记进程用的 {recorded}，日志不涨的记 stalled（「{note[:60]}」），不判顶在上限上卡住"


def cell_pinned_peer_unavailable(world):
    key = "selftest-pinned-peer"
    world.resources(local=(30, 30), peer=(25, 100), peer_cap_gibibytes=64)
    world.seed_peak(key, 20 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250"], extra={lib.KEY_VARIABLE: key}, cwd=world.repo)
    world.wait_for(task_id, lambda summary: bool(summary["attempts"]) and summary["attempts"][0].get("state") == "ended", "第一次尝试收尾")
    world.resources(local=(30, 30))   # 第二台此刻用不了了
    try:
        summary = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    except SelftestFailure as failure:
        raise SelftestFailure(f"本机已给到它的总上限 30G 还撞、换到第二台钉在那里重排、第二台随即用不了：应当判 never-fits 收尾，实际 {failure}", failure.extra_lines) from None
    task = summary["task"]
    attempts = [(attempt.get("host"), attempt.get("outcome")) for attempt in summary["attempts"]]
    if attempts != [("local", "memory-cap-hit")] or task.get("outcome") != "never-fits" or task.get("exit_code") != 241 or "钉在第二台" not in (task.get("note") or ""):
        raise SelftestFailure(f"钉在第二台重排、第二台用不了：应当只有本机那一次撞上限的尝试、结局 never-fits（退 241）、说钉在第二台而它用不了，"
                              f"实际尝试 {attempts}、{describe(summary)}、说明「{task.get('note')}」", world.server_log_lines())
    return "本机到顶换到第二台钉着重排、第二台随即用不了：判 never-fits（退 241），不在本机另起"


def cell_admission_booked_at_cap(world):
    key = "selftest-booked-at-cap"
    world.seed_peak(key, 30 * GIBIBYTE)   # 预测、上限与记账都是 30G（近五次最大，不乘 1.5）
    world.resources(local=(50, 50))
    world.start_server()
    releases = [os.path.join(world.root, f"release-{number}") for number in (1, 2)]
    first = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", releases[0]], extra={lib.KEY_VARIABLE: key})
    try:
        running = world.running(first)
        second = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", releases[1]], extra={lib.KEY_VARIABLE: key})
        # 看几秒第二条起没起（起了的话队就空了，排队线程不再记「pass」那一行，不拿 next_pass 等）
        waiting = world.summary(second)
        deadline = time.monotonic() + ADMISSION_WATCH_SECONDS
        while time.monotonic() < deadline and not waiting["attempts"]:
            time.sleep(POLL_SECONDS)
            waiting = world.summary(second)
    finally:
        open(releases[0], "w").close()
    open(releases[1], "w").close()
    later = world.finished(second)
    booked = running["attempts"][0].get("booked")
    if waiting["task"].get("state") != "queued" or waiting["attempts"] or booked != 30 * GIBIBYTE or later["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"两条各记过峰值 30G（上限与记账都是 30G）、本机 50G：头一条记账 30G，第二条（再记 30G 就超过 50G）应当排着、头一条放了才起；"
                              f"实际头一条记账 {booked} 字节、第二条先是 {waiting['task'].get('state')}、尝试 {len(waiting['attempts'])} 次、最后 {describe(later)}",
                              world.server_log_lines())
    return "两条各记过峰值 30G 在 50G 的台上：头一条记账 30G（上限）、第二条排着，头一条放了才起"


SHARD_FITTED_FLOOR_BYTES = 30 * GIBIBYTE   # shard-cap-fitted-to-room 这一格的内存上限下限（TASK_SERVER_SELFTEST_MEMORY_FLOOR_BYTES）：两片各记过峰值 20G，上限与记账抬到 30G、预测仍是 20G


def cell_shard_cap_fitted_to_room(world):
    """分片的片记满上限放不下、预测放得下时，上限压到放得下的量（task_server_lib.py 文件头「资源记录」的准入；task_server.py 的 shard_cap_fitted_to_room）。
    记录定的预测、上限与记账相等（近五次最大，不乘 1.5），只有内存上限的下限把上限与记账抬到预测之上时才有「记满上限放不下、预测放得下」，这一格照这一种造：
    下限设 30G（SHARD_FITTED_FLOOR_BYTES），两片各记过峰值 20G（上限与记账抬到 30G）、本机 54G、余量 2G：第一片上限 30G，第二片压到 54G − 2G − 30G = 22G，
    两片同时跑、上限之和恰好等于总上限减余量，两片的 prediction 都记 RECORD_HISTORY（照各片自己的键读到了记录），整件成了。
    弄坏开关 shard-cap-not-fitted 下第二片排着，shard-key-position-folded 下各片读不到自己的记录，这一格红。"""
    key = "selftest-shard-cap-fitted"
    for index in (0, 1):
        world.seed_peak(lib.shard_child_key(key, "shard", index, 2), 20 * GIBIBYTE, shard_count=2)
    world.resources(local=(54, 54), peer_down="自证：第二台此刻用不了")
    release = os.path.join(world.root, "release-fitted")
    spec = world.write_spec("spec-fitted.json", {"shards": [{"block": release}, {"block": release}]})
    world.start_server(extra={lib.MEMORY_FLOOR_VARIABLE: str(SHARD_FITTED_FLOOR_BYTES)})
    task_id = world.submit(spec, cwd=world.repo, extra={lib.KEY_VARIABLE: key})
    try:
        summary = world.wait_for(task_id, lambda summary: len(running_shards(summary)) == 2, "两片同时在本机跑起来")
        firsts = [child["attempts"][0] for child in shard_children(summary).values()]
        caps = sorted(int(attempt.get("memory_cap") or 0) for attempt in firsts)
        histories = sorted({str(attempt.get("prediction")) for attempt in firsts})
    finally:
        open(release, "w").close()
    finished = world.finished(task_id)
    floor = SHARD_FITTED_FLOOR_BYTES
    if (caps != [(54 - SELFTEST_MARGIN_GIBIBYTES) * GIBIBYTE - floor, floor] or histories != [lib.RECORD_HISTORY]
            or finished["task"].get("outcome") != "succeeded"):
        raise SelftestFailure(f"下限 30G、两片各记过峰值 20G、本机 54G、余量 {SELFTEST_MARGIN_GIBIBYTES}G：第一片上限 30G，第二片压到总上限减余量减第一片"
                              f"（上限之和恰好等于总上限减余量）、两片 prediction 都记 {lib.RECORD_HISTORY}、整件成了；"
                              f"实际上限 {caps}、prediction {histories}、整件 {describe(finished)}", world.server_log_lines())
    return f"下限抬到 30G 的两片（峰值各 20G）在本机 54G：上限 {[cap // GIBIBYTE for cap in caps]}G，同时跑、之和不超过总上限减余量"


ADMISSION_WATCH_SECONDS = 3   # admission-booked-at-cap 看第二条起没起看这么久（一轮排队 0.1 秒）


def cell_margin_measured_per_host(world):
    """余量按台现量（task_server.py 文件头 ②「余量」；host_capacity、host_margin）：两台 slice 总上限都是 50G，记录峰值 48G 的那一条上限与记账都是 49152M
    （近五次最大，不乘 1.5），它只在余量不大于 2G 的台上装得下。只有本机、本机余量 1G（可用上限 49G）：在本机起、跑完；
    本机余量 3G（可用上限 47G，峰值本身就装不下）、第二台余量 1G（49G）：派第二台、跑完。
    弄坏开关 margin-local-for-both 下第二台也减本机的 3G、两台都比峰值小，照收到的 47G 起跑（上限不是 49152M），这一格红。"""
    key = "selftest-margin-per-host"
    world.seed_peak(key, PER_HOST_PEAK_BYTES)
    world.resources(local=(100, 50), local_margin=PER_HOST_SMALL_MARGIN_GIBIBYTES)
    world.start_server()
    problems = []
    scenarios = (("只有本机、本机余量 1G", "local", {"local": (100, 50), "local_margin": PER_HOST_SMALL_MARGIN_GIBIBYTES}),
                 ("本机余量 3G、第二台余量 1G", "peer", {"local": (100, 50), "peer": (100, 50), "peer_cap_gibibytes": 50,
                                                      "local_margin": PER_HOST_LARGE_MARGIN_GIBIBYTES, "peer_margin": PER_HOST_SMALL_MARGIN_GIBIBYTES}))
    for what, host, scenario in scenarios:
        world.resources(**scenario)
        summary = world.finished(world.submit(["bash", "-c", "exit 0", f"margin-per-host-{host}"], extra={lib.KEY_VARIABLE: key}), timeout=PLACEMENT_WAIT_SECONDS)
        attempt = first_attempt(summary)
        if (attempt.get("host") != host or attempt.get("memory_cap_text") != PER_HOST_CAP_TEXT or attempt.get("booked") != PER_HOST_PEAK_BYTES
                or summary["task"].get("outcome") != "succeeded"):
            problems.append(f"{what}：应当在{lib.HOST_NAMES.get(host, host)}起、上限与记账 {PER_HOST_CAP_TEXT}、跑完，"
                            f"实际台 {attempt.get('host')}、记账 {attempt.get('booked')} 字节、{describe(summary)}")
    if problems:
        raise SelftestFailure("余量按台：" + "；".join(problems), world.server_log_lines())
    return f"峰值 48G 的那一条：余量 1G 的本机起得来；本机余量 3G 装不下时派余量 1G 的第二台，上限都是 {PER_HOST_CAP_TEXT}，两台各用各的余量"


def cell_margin_unmeasured_not_placed(world):
    """余量量不到不派、不退回固定值（task_server.py 文件头 ②「余量」）：本机报告里余量是 null（量不到）、第二台量得到：这一条不在本机起、在第二台起；
    两台都量不到：一次都不起，结局 run-condition-unmet（退 78），说明点名两台各自量不到余量。看排两轮之后的样子，不比秒数。
    弄坏开关 margin-fallback-fixed 下本机退回 2G、照在本机起；margin-unmeasured-waits 下两台都量不到时照排着。"""
    world.resources(local=(100, 100), peer=(100, 100), peer_cap_gibibytes=64, local_margin=None)
    world.start_server()
    problems = []
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "margin-unmeasured-local"]), timeout=PLACEMENT_WAIT_SECONDS)
    attempt = first_attempt(summary)
    if attempt.get("host") != "peer" or summary["task"].get("outcome") != "succeeded":
        problems.append(f"本机量不到余量、第二台量得到：应当只在第二台起、跑完（本机两台都空 100G，退回固定 2G 的写法下会挑本机），实际台 {attempt.get('host')}、{describe(summary)}")
    world.resources(local=(100, 100), peer=(100, 100), peer_cap_gibibytes=64, local_margin=None, peer_margin=None)
    task_id = world.submit(["bash", "-c", "exit 0", "margin-unmeasured-both"])
    world.next_pass()
    world.next_pass()
    summary = world.summary(task_id)
    task = summary["task"]
    note = task.get("note") or ""
    if (task.get("outcome") != "run-condition-unmet" or task.get("exit_code") != lib.RUN_CONDITION_EXIT or summary["attempts"] or world.calls_of(task_id)
            or "本机此刻量不到余量" not in note or "第二台此刻量不到余量" not in note):
        if task.get("state") == "queued":
            world.client(["cancel", str(task_id)])
        problems.append(f"两台都量不到余量：应当一次都不起、结局 run-condition-unmet（退 {lib.RUN_CONDITION_EXIT}）、说明点名本机与第二台都量不到余量，"
                        f"实际状态 {task.get('state')}、{describe(summary)}、内存包装起了 {len(world.calls_of(task_id))} 次、说明「{note[:300]}」、"
                        f"排队原因「{summary.get('waiting_for')}」")
    if problems:
        raise SelftestFailure("余量量不到：" + "；".join(problems), world.server_log_lines())
    return "本机量不到余量时只派第二台；两台都量不到时一次都不起、记 run-condition-unmet、说明点名两台"


def cell_pin_cap_refused_within_margin(world):
    """钉值不大于 0（task_server.py 文件头 ②「内存」的钉值）：SLICE_MEMORY_CAP_BY_HOST 两台都写 2G、两台此刻的余量都是假余量 2G（减完是 0）：
    control pin-cap 当场拒（退非 0、说明点名 SLICE_MEMORY_CAP_BY_HOST 与把那一项改大的出路）、没钉；配置写 20G 时钉上的命令，配置改成 2G 之后再交：
    两台都不派、结局 run-condition-unmet（退 78）、说明写钉值不大于 0。弄坏开关 pinned-nonpositive-accepted 下照钉、照派。"""
    world.resources(local=(100, 100), peer=(100, 100), peer_cap_gibibytes=64)
    world.set_slice_cap("2G")
    world.start_server()
    problems = []
    refused_probe = ["bash", "-c", "exit 0", "pin-within-margin-refused"]
    refused = world.control(["pin-cap", "--", *refused_probe])
    said = (refused.stdout + refused.stderr).strip()
    if refused.returncode == 0 or lib.PIN_SOURCE not in said or "改大" not in said or "钉上了" in said:
        problems.append(f"两台那一项 2G 不比余量 2G 大：control pin-cap 应当退非 0、说明点名 {lib.PIN_SOURCE} 与改大的出路、不说钉上了，实际退 {refused.returncode}、「{said[-400:]}」")
    left = world.control(["unpin-cap", "--", *refused_probe])
    if left.returncode != 0 or "原来就没钉" not in left.stdout:
        problems.append(f"拒了的 pin-cap 不该钉上：unpin-cap 应当说原来就没钉，实际退 {left.returncode}、「{(left.stdout + left.stderr).strip()[-300:]}」")
    world.set_slice_cap("20G")
    probe = ["bash", "-c", "exit 0", "pin-within-margin-dispatch"]
    pinned = world.control(["pin-cap", "--", *probe])
    if pinned.returncode != 0 or "钉上了" not in pinned.stdout:
        problems.append(f"配置写 20G 时 control pin-cap 应当退 0、说钉上了，实际退 {pinned.returncode}、「{(pinned.stdout + pinned.stderr).strip()[-300:]}」")
    world.set_slice_cap("2G")
    task_id = world.submit(probe)
    world.next_pass()
    world.next_pass()
    summary = world.summary(task_id)
    task = summary["task"]
    note = task.get("note") or ""
    if (task.get("outcome") != "run-condition-unmet" or task.get("exit_code") != lib.RUN_CONDITION_EXIT or summary["attempts"] or world.calls_of(task_id)
            or "不大于 0" not in note):
        if task.get("state") == "queued":
            world.client(["cancel", str(task_id)])
        problems.append(f"钉了的命令在配置改成 2G 之后：两台的钉值都不大于 0，应当一次都不起、结局 run-condition-unmet（退 {lib.RUN_CONDITION_EXIT}）、说明写钉值不大于 0，"
                        f"实际状态 {task.get('state')}、{describe(summary)}、说明「{note[:300]}」、排队原因「{summary.get('waiting_for')}」")
    if problems:
        raise SelftestFailure("钉值不大于 0：" + "；".join(problems), world.server_log_lines())
    return "那一项不比余量大时 control pin-cap 当场拒、没钉；已钉的命令配置改小到不比余量大之后不派、记 run-condition-unmet"


# peer-booked-like-local：峰值 20480M（20G），上限与记账照它；在跑的每条此刻回收不掉 1G；两台都是 slice 50G、余量 1G、空闲 40G
BOTH_HOSTS_PEAK_BYTES = 20480 * MEBIBYTE
BOTH_HOSTS_CAP_TEXT = "20480M"
BOTH_HOSTS_USED_BYTES = 19 * GIBIBYTE   # 在跑的六条各自此刻回收不掉的：记账照它记 19G（task_server.py 的 running_booking）
BOTH_HOSTS_RELEASE = "release-both-hosts"   # 相对假仓根：本机那几条在 world.repo 里等它，假第二台照 fake-peer-wait 等同一个名字
SELFTEST_TIGHT_MARGIN_GIBIBYTES = 1         # 这几格两台此刻的余量（两台相同：margin-local-for-both 下第二台读本机那一项，判定不变）


def occupy_hosts(world, command, key, hosts_running):
    """照 hosts_running 的先后在指定的台上起一条条占着到放行为止的任务：派本机那几条时假余量只给本机（第二台没报告），派第二台那几条时本机给不出上限
    （总上限 1G 不比余量 1G 大，这一轮不派本机：不是放不下，peer-overflow-blocked 那一种不挡）。交 {台: [任务号…]}，每一条都核过起在那一台。"""
    started = {"local": [], "peer": []}
    for host in hosts_running:
        if host == "local":
            world.resources(local=(100, 100), local_margin=SELFTEST_TIGHT_MARGIN_GIBIBYTES)
        else:
            world.resources(local=(1, 1), local_margin=SELFTEST_TIGHT_MARGIN_GIBIBYTES, peer=(100, 100), peer_cap_gibibytes=100,
                            peer_margin=SELFTEST_TIGHT_MARGIN_GIBIBYTES)
        task_id = world.submit(command, extra={lib.KEY_VARIABLE: key}, cwd=world.repo)
        attempt = first_attempt(world.running(task_id))
        if attempt.get("host") != host:
            raise SelftestFailure(f"占着{lib.HOST_NAMES.get(host, host)}的那一条应当起在{lib.HOST_NAMES.get(host, host)}，实际起在 {attempt.get('host')}", world.server_log_lines())
        started[host].append(task_id)
    return started


def cell_peer_booked_like_local(world):
    """第二台与本机同一段（task_server.py 文件头 ②「内存」的在跑记账）：两台各三条在跑、各上限 20G、各自此刻回收不掉 19G（两台都照那一台报告里 label
    对得上的那个 scope；记账 = min(上限, 用量) = 19G）；两台都是 slice 50G、余量 1G、空闲 40G、进程用 3G：每台待涨量 0，第四条 20G
    两台都放不下（每台记账之和 57G 加它超过总上限减余量 49G），排两轮之后照排着、排队原因两台都写。放行之后第四条照起、跑完。
    挡住第四条的是公式（记账之和），不经待涨量：弄坏开关 server-pending-ignored 下判法不变，这一格不配它。"""
    world.seed_peak("selftest-both-hosts", BOTH_HOSTS_PEAK_BYTES)
    world.resources(local=(100, 100), local_margin=SELFTEST_TIGHT_MARGIN_GIBIBYTES)
    world.start_server()
    command = ["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "both-hosts", BOTH_HOSTS_RELEASE, f"fake-peer-wait={BOTH_HOSTS_RELEASE}"]
    problems = []
    try:
        started = occupy_hosts(world, command, "selftest-both-hosts", ["local"] * 3 + ["peer"] * 3)
        labels = [first_attempt(world.summary(task_id)).get("label") for task_id in started["peer"]]
        local_labels = [first_attempt(world.summary(task_id)).get("label") for task_id in started["local"]]
        caps = [first_attempt(world.summary(task_id)).get("memory_cap_text") for task_id in started["local"] + started["peer"]]
        if caps != [BOTH_HOSTS_CAP_TEXT] * 6 or not all(labels + local_labels):
            problems.append(f"占着两台的六条应当各上限 {BOTH_HOSTS_CAP_TEXT}、都有 label，实际上限 {caps}、label {labels + local_labels}")
        world.resources(local=(40, 50), local_slice_used=3, local_margin=SELFTEST_TIGHT_MARGIN_GIBIBYTES, local_scopes={label: BOTH_HOSTS_USED_BYTES for label in local_labels},
                        peer=(40, 50), peer_cap_gibibytes=50, peer_slice_used=3, peer_margin=SELFTEST_TIGHT_MARGIN_GIBIBYTES,
                        peer_scopes={label: BOTH_HOSTS_USED_BYTES for label in labels})
        fourth = world.submit(["bash", "-c", "exit 0", "both-hosts-fourth", "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-both-hosts"}, cwd=world.repo)
        world.next_pass()
        world.next_pass()
        waiting = world.summary(fourth)
        reason = waiting.get("waiting_for") or ""
        if waiting["task"].get("state") != "queued" or waiting["attempts"] or "本机：" not in reason or "第二台：" not in reason:
            problems.append(f"两台各三条 20G 在跑（各用 19G）、slice 50G、余量 1G：第四条 20G 两台都放不下，应当排着、排队原因两台都写；"
                            f"实际状态 {waiting['task'].get('state')}、{describe(waiting)}、排队原因「{reason[:300]}」")
    finally:
        open(os.path.join(world.repo, BOTH_HOSTS_RELEASE), "w").close()
    summaries = [world.finished(task_id, timeout=120) for task_id in [*started.get("local", []), *started.get("peer", []), fourth]] if not problems else []
    if any(summary["task"].get("outcome") != "succeeded" for summary in summaries):
        problems.append(f"放行之后七条都应当跑完，实际 {[describe(summary) for summary in summaries]}")
    if problems:
        raise SelftestFailure("两台同一个口径：" + "；".join(problems), world.server_log_lines())
    return "两台各三条 20G 在跑、各用 19G：第二台照报告里的 scope 记账（实测 19G），同本机，第四条 20G 两台都排着；放行之后跑完"


# peer-scope-missing-booked-at-cap：记录的峰值 5G；内存上限的下限（TASK_SERVER_SELFTEST_MEMORY_FLOOR_BYTES）15G 把上限与新起的记账抬到 15G（15360M）；
# 报告里那条 scope 此刻回收不掉 3G
MISSING_SCOPE_PEAK_BYTES = 5 * GIBIBYTE
MISSING_SCOPE_FLOOR_BYTES = 15 * GIBIBYTE
MISSING_SCOPE_CAP_TEXT = "15360M"
MISSING_SCOPE_USED_BYTES = 3 * GIBIBYTE
MISSING_SCOPE_RELEASE = "release-missing-scope"


def cell_peer_scope_missing_booked_at_cap(world):
    """第二台报告里缺某条在跑尝试的 scope（还没起来、读不了）：那一条记整份上限，不记 0；有 scope 的照实测记、不记上限（task_server.py 的 server_pending、
    running_booking）。只派第二台（本机总上限 1G 不比余量 1G 大）。三条同一把键：记录的峰值 5G，内存上限的下限 15G 把上限与新起的记账抬到 15G（实测比上限小，才分得出记实测与记上限）。
    第二台 slice 50G、余量 1G、空闲 20G，两条在跑各上限 15G：报告里只有其中一条的 scope（回收不掉 3G、进程用 3G；记实测 3G）时待涨量 0 + 15G、空闲剩 3G，
    第三条（要的空闲 6G：预测 5G 加余量 1G；记账 15G）放不下、排两轮照排着（缺的记 0 的写法下待涨量 2G、空闲剩 18G，放得下）；
    报告补上另一条的 scope（也是 3G、进程共用 6G）之后两条各记 5G、待涨量 2G + 2G、空闲剩 16G、slice 减余量减待涨量剩 39G、公式剩 39G：第三条放得下、起在第二台
    （有 scope 的也记上限的写法下待涨量 12G + 12G、空闲剩不下，照样放不下；弄坏开关 running-booked-at-cap 下就是这样）。
    弄坏开关 server-pending-ignored 下在跑的不记待涨量：头一段第三条就起了。"""
    world.seed_peak("selftest-missing-scope", MISSING_SCOPE_PEAK_BYTES)
    world.resources(local=(1, 1), local_margin=SELFTEST_TIGHT_MARGIN_GIBIBYTES)
    world.start_server(extra={lib.MEMORY_FLOOR_VARIABLE: str(MISSING_SCOPE_FLOOR_BYTES)})
    command = ["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "missing-scope", MISSING_SCOPE_RELEASE, f"fake-peer-wait={MISSING_SCOPE_RELEASE}"]
    problems = []
    third = None
    try:
        started = occupy_hosts(world, command, "selftest-missing-scope", ["peer"] * 2)["peer"]
        labels = [first_attempt(world.summary(task_id)).get("label") for task_id in started]
        caps = [first_attempt(world.summary(task_id)).get("memory_cap_text") for task_id in started]
        if caps != [MISSING_SCOPE_CAP_TEXT] * 2 or not all(labels):
            problems.append(f"占着第二台的两条应当各上限 {MISSING_SCOPE_CAP_TEXT}、都有 label，实际上限 {caps}、label {labels}")
        scenario = {"local": (1, 1), "local_margin": SELFTEST_TIGHT_MARGIN_GIBIBYTES, "peer": (20, 50), "peer_cap_gibibytes": 50,
                    "peer_margin": SELFTEST_TIGHT_MARGIN_GIBIBYTES}
        world.resources(**scenario, peer_slice_used=3, peer_scopes={labels[0]: MISSING_SCOPE_USED_BYTES})
        third = world.submit(["bash", "-c", "exit 0", "missing-scope-third", "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-missing-scope"}, cwd=world.repo)
        world.next_pass()
        world.next_pass()
        waiting = world.summary(third)
        if waiting["task"].get("state") != "queued" or waiting["attempts"]:
            problems.append(f"报告里缺一条在跑的 scope：那一条应当记整份上限 15G（有 scope 的那一条记实测 3G），第三条放不下、排着；实际状态 {waiting['task'].get('state')}、{describe(waiting)}、"
                            f"排队原因「{(waiting.get('waiting_for') or '')[:300]}」")
        world.resources(**scenario, peer_slice_used=6, peer_scopes={label: MISSING_SCOPE_USED_BYTES for label in labels})
        placed = world.wait_for(third, lambda found: bool(found["attempts"]) or found["task"].get("state") != "queued", "照补上的 scope 起来")
        if first_attempt(placed).get("host") != "peer":
            problems.append(f"报告补上那一条的 scope 之后两条各记实测 3G、不记上限，第三条应当起在第二台（待涨量 0），实际 {describe(placed)}、排队原因「{(placed.get('waiting_for') or '')[:300]}」")
    finally:
        open(os.path.join(world.repo, MISSING_SCOPE_RELEASE), "w").close()
    summaries = [world.finished(task_id, timeout=120) for task_id in [*started, third]] if not problems else []
    if any(summary["task"].get("outcome") != "succeeded" for summary in summaries):
        problems.append(f"放行之后三条都应当跑完，实际 {[describe(summary) for summary in summaries]}")
    if problems:
        raise SelftestFailure("第二台缺 scope 的记整份上限：" + "；".join(problems), world.server_log_lines())
    return "第二台报告里缺一条在跑的 scope 时那一条记整份上限 15G、第三条排着；补上之后两条各记实测 3G（不记上限）、第三条起在第二台"


# two-hosts-booked-within-capacity：三种各四条（上限与记账等于记录的峰值：15G、9G，没记录的记 4G，合计 112G）交给两台各 slice 50G、余量 1G（各能记 49G）
INVARIANT_KINDS = (("selftest-invariant-large", 15 * GIBIBYTE), ("selftest-invariant-medium", 9 * GIBIBYTE), ("selftest-invariant-small", None))
INVARIANT_COUNT_PER_KIND = 4
INVARIANT_FILL_PASSES = 6      # 放出来之后看几轮（第二台每轮至多起 PEER_STARTS_PER_PASS 条：12 条几轮里就起到装不下为止）
INVARIANT_DRAIN_PASSES = 40    # 每放行一批之后至多看几轮：在排的起完了就不再看（一轮 0.1 秒，放行的那几条收尾、在排的接着起都在这里面）
INVARIANT_RELEASES = ("release-invariant-early", "release-invariant-late")


def booked_caps_by_host(world):
    """此刻库里在起、在跑的尝试按台的上限之和（字节）与在排的条数：照服务读回的规则核过的那几行，核不过的不算。"""
    caps = {"local": 0, "peer": 0}
    for record, valid in world.signed_rows("attempts"):
        if valid and record.get("state") in ("starting", "running"):
            caps[record.get("host")] = caps.get(record.get("host"), 0) + int(record.get("memory_cap") or 0)
    queued = sum(1 for record, valid in world.signed_rows("tasks") if valid and record.get("state") == "queued")
    return caps, queued


def cell_two_hosts_booked_within_capacity(world):
    """总的不变量（task_server_lib.py 文件头「资源记录」的准入按上限记账）：两台各 slice 50G、余量 1G，12 条合计要记 112G、两台一齐放出来跑一段排队，
    分两批放行：每一轮之后读库，每台在起、在跑的尝试的上限之和都不超过那台总上限减余量 49G（这一格的任务峰值都装得下，没有「峰值本身装不下」的那一种例外）；
    装不下的排着，排过的里有一轮还有在排的，两台都分到，放行之后都跑完。第二台在跑的待涨量照报告里的 scope（这一格的报告不报 scope：记整份上限）。
    这一格封顶的是公式（verdict_within_the_formula：总上限减余量减在跑的记账之和），不经待涨量（server_pending）：报告的空闲（200G）与 slice 余量（50G）不随在跑的涨落，
    弄坏开关 server-pending-ignored 下上限之和照样不超过 49G，这个开关不在这一格判红（它红在待涨量与均衡挑台那几格，开关表在 task_server_lib.py 文件头）。"""
    for key, peak in INVARIANT_KINDS:
        if peak is not None:
            world.seed_peak(key, peak)
    both = {"local": (200, 50), "local_margin": SELFTEST_TIGHT_MARGIN_GIBIBYTES, "peer": (200, 50), "peer_cap_gibibytes": 50,
            "peer_margin": SELFTEST_TIGHT_MARGIN_GIBIBYTES}
    world.resources(**both, hold=True)
    world.start_server()
    capacity = (50 - SELFTEST_TIGHT_MARGIN_GIBIBYTES) * GIBIBYTE
    tasks = []
    for position in range(INVARIANT_COUNT_PER_KIND):
        for key, _ in INVARIANT_KINDS:
            release = INVARIANT_RELEASES[len(tasks) % len(INVARIANT_RELEASES)]
            command = ["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, f"invariant-{position}", release, f"fake-peer-wait={release}"]
            tasks.append(world.submit(command, extra={lib.KEY_VARIABLE: key}, cwd=world.repo))
    world.resources(**both)   # 一齐放出来
    world.api("POST", "/wake")
    samples, over = [], []

    def sample():
        world.next_pass()
        caps, queued = booked_caps_by_host(world)
        samples.append((caps, queued))
        over.extend(f"{lib.HOST_NAMES.get(host, host)}上限之和 {total / GIBIBYTE:.1f}G（还有 {queued} 条在排）" for host, total in caps.items() if total > capacity)
        return queued

    try:
        for _ in range(INVARIANT_FILL_PASSES):
            sample()
        for release in INVARIANT_RELEASES:   # 分两批放行：放了的收尾、在排的接着起，每一轮都照样核
            open(os.path.join(world.repo, release), "w").close()
            for _ in range(INVARIANT_DRAIN_PASSES):
                if not sample():
                    break
    finally:
        for release in INVARIANT_RELEASES:
            open(os.path.join(world.repo, release), "w").close()
    summaries = [world.finished(task_id, timeout=120) for task_id in tasks]
    hosts = {first_attempt(summary).get("host") for summary in summaries}
    problems = []
    if over:
        problems.append(f"每台在跑的上限之和应当不超过总上限减余量 {capacity / GIBIBYTE:.0f}G，实际有 {len(over)} 次超过：{'；'.join(over[:6])}")
    if not any(queued for _, queued in samples):
        problems.append(f"12 条合计 112G 比两台能记的 98G 多：应当有一轮还有在排的，实际各轮在排 {[queued for _, queued in samples]}")
    if hosts != {"local", "peer"} or any(summary["task"].get("outcome") != "succeeded" for summary in summaries):
        problems.append(f"两台都应当分到、都跑完，实际落在 {sorted(host or '-' for host in hosts)}、{[describe(summary) for summary in summaries if summary['task'].get('outcome') != 'succeeded']}")
    if problems:
        raise SelftestFailure("两台的上限之和：" + "；".join(problems), [f"各轮 {caps}、在排 {queued}" for caps, queued in samples] + world.server_log_lines())
    peak = {host: max(caps[host] for caps, _ in samples) / GIBIBYTE for host in ("local", "peer")}
    return f"12 条合计 112G 排在两台各 49G 里：{len(samples)} 轮读库，每台上限之和至多 {peak}G，都跑完"


def cell_build_disk_predicted(world):
    """问题 76：编译目录占盘按命令记峰值（本机 attempt_runner 量服务给的编译目录），派活时那一台盘余量放得下「预测 + 下限」才派，放不下排着、
    排队原因点名占盘；本机与第二台同样判。"""
    problems = []
    world.resources(local=(30, 30))
    world.start_server()
    path = f"{world.fake_cargo}:{os.environ.get('PATH', '/usr/bin:/bin')}"
    built = world.finished(world.submit(["cargo", "build"], extra={"PATH": path}))
    peak = (built["attempts"] or [{}])[0].get("build_peak")
    if not isinstance(peak, int) or peak < 65536:
        problems.append(f"会编的那一次应当量下编译目录占盘峰值（假 cargo 写了 64K），实际 {peak}、{describe(built)}")
    floor = SERVER.BUILD_DISK_FLOOR_BYTES
    world.seed_attempt({"command_key": "selftest-disk-local", "outcome": "succeeded", "memory_peak": GIBIBYTE, "build_peak": 30 * GIBIBYTE})
    world.seed_attempt({"command_key": "selftest-disk-peer", "outcome": "succeeded", "memory_peak": 10 * GIBIBYTE, "build_peak": 30 * GIBIBYTE})
    for host, key in (("local", "selftest-disk-local"), ("peer", "selftest-disk-peer")):
        small = floor + 10 * GIBIBYTE
        if host == "local":
            world.resources(local=(30, 30), local_disk_bytes=small)
        else:
            world.resources(local=(8, 30), local_disk_bytes=10 * floor, peer=(100, 100), peer_cap_gibibytes=64, peer_disk_bytes=small)
        task_id = world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: key}, cwd=world.repo)
        world.next_pass()
        world.next_pass()
        held = world.summary(task_id)
        if held["task"].get("state") != "queued" or "编译目录占盘" not in (held.get("waiting_for") or ""):
            problems.append(f"{lib.HOST_NAMES[host]}盘余 {small // GIBIBYTE}G、这条命令量过 30G（加下限 {floor // GIBIBYTE}G 放不下）：应当排着、排队原因点名占盘，"
                            f"实际 {held['task'].get('state')}、在等「{held.get('waiting_for')}」、{describe(held)}")
            continue
        if host == "local":
            world.resources(local=(30, 30), local_disk_bytes=10 * floor)
        else:
            world.resources(local=(8, 30), local_disk_bytes=10 * floor, peer=(100, 100), peer_cap_gibibytes=64, peer_disk_bytes=10 * floor)
        finished = world.finished(task_id)
        attempt = (finished["attempts"] or [{}])[0]
        if finished["task"].get("outcome") != "succeeded" or attempt.get("host") != host or attempt.get("disk_booked") != 30 * GIBIBYTE:
            problems.append(f"{lib.HOST_NAMES[host]}盘放得下之后应当在那一台起、占盘记 30G，实际 {describe(finished)}、在 {attempt.get('host')}、记 {attempt.get('disk_booked')}")
    if problems:
        raise SelftestFailure("编译目录占盘：" + "；".join(problems), world.server_log_lines())
    return f"量下编译目录峰值 {peak} 字节；本机与第二台盘余量放不下「30G + 下限」时排着、原因点名占盘，放得下才起、占盘记 30G"


CLAMP_HOST_SLICE_GIBIBYTES = 50          # cap-hit-record-over-every-host-starts-clamped：只有本机、slice 总上限 50G、假余量 2G：可用上限 48G
CLAMP_CAP_HIT_BYTES = 45 * GIBIBYTE       # 同键上一次撞的上限（峰值到了上限）、之后没跑完过：进定额度的量按它乘 1.5 是 67.5G，比 48G 大
CLAMP_HOST_LIMIT_TEXT = "49152M"          # 48G：总上限 50G 减余量 2G
CLAMP_TINY_SLICE_GIBIBYTES = 5            # booking-within-host-limit-every-history：本机 slice 总上限 5G、假余量 2G，可用上限 3G；四种来历的预测都比它大
CLAMP_PINNED_ITEM_BYTES = 8 * GIBIBYTE    # 钉值那一条：多机配置里这台那一项 8G，减余量 2G 是 6G
CLAMP_RECORDED_BYTES = 7 * GIBIBYTE       # 记录那一条：近几次最大 7G
CLAMP_RETRY_BYTES = 4608 * MEBIBYTE       # 撞上限重排那一条：3072M 撞了乘 1.5


def cell_cap_hit_record_over_every_host_starts_clamped(world):
    """记录比每一台都大照收到可用上限起跑（task_server.py 文件头 ②「内存」的准入记账；booked_at_cap、hosts_short_of_record）：只有本机、slice 总上限 50G、余量 2G
    （可用上限 48G），同键上一次撞了 45G 的上限（峰值到了上限）、之后这把键没跑完过：进定额度的量是上限乘 1.5 的 67.5G，比本机大，也没有别的台；
    新交的一条不判 never-fits，在本机按 48G（49152M，总上限减余量）起跑、跑完，尝试行的上限与记账都不大于 48G。
    弄坏开关 booking-over-host-limit 下记账照 67.5G 记、judge_host 判永远放不下，record-over-every-host-never-fits 下本机照永远放不下判，都记 never-fits，这一格红。"""
    key = "selftest-record-over-every-host"
    world.resources(local=(100, CLAMP_HOST_SLICE_GIBIBYTES))
    world.start_server()
    seed_ended_attempt(world, key, SEEDED_ENDED_AT, "memory-cap-hit", memory_cap=CLAMP_CAP_HIT_BYTES, memory_peak=CLAMP_CAP_HIT_BYTES)
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=0"], extra={lib.KEY_VARIABLE: key}))
    attempt = first_attempt(summary)
    limit = (CLAMP_HOST_SLICE_GIBIBYTES - SELFTEST_MARGIN_GIBIBYTES) * GIBIBYTE
    if (summary["task"].get("outcome") != "succeeded" or len(summary["attempts"]) != 1 or attempt.get("host") != "local"
            or attempt.get("memory_cap_text") != CLAMP_HOST_LIMIT_TEXT or int(attempt.get("memory_cap") or 0) > limit or int(attempt.get("booked") or 0) > limit):
        raise SelftestFailure(f"同键撞过 45G、之后没跑完过（进定额度 67.5G），只有本机、可用上限 48G：应当在本机按 {CLAMP_HOST_LIMIT_TEXT} 起跑一次、跑完，"
                              f"上限与记账不大于 48G（{limit} 字节），不判 never-fits；实际台 {attempt.get('host')}、上限 {attempt.get('memory_cap_text')}"
                              f"（{attempt.get('memory_cap')} 字节）、记账 {attempt.get('booked')} 字节、{describe(summary)}、说明「{summary['task'].get('note')}」",
                              world.server_log_lines())
    return f"进定额度 67.5G 比本机可用上限 48G 大：照 {CLAMP_HOST_LIMIT_TEXT} 起跑、跑完，不判 never-fits"


def cell_booking_within_host_limit_every_history(world):
    """任何来历算出的上限与记账都不大于判的那一台的可用上限（task_server.py 的 booked_at_cap；文件头 ②「内存」的准入记账）：本机 slice 总上限 5G、余量 2G，可用上限 3G；
    钉值（多机配置那一项 8G 减余量是 6G）、记录（7G）、没有记录的缺省（UNRECORDED_MEMORY_DEFAULT_BYTES，4G）、撞上限重排（4608M）四条的预测都比 3G 大：
    booked_at_cap 交给 judge_host 的预测峰值、上限与记账都不大于 3G。可用上限照这一格自己算（总上限减余量），不读 host_limit。套在假调度器上，不起服务。
    弄坏开关 booking-over-host-limit 下钉值之外的三条照原样记，booked-at-peak 下四条都照原样，booked-up-to-slice-total 下收到 5G（不减余量），这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.host_dispatch = lib.load_module("host_dispatch_for_booking_cell", os.path.join(lib.HOSTS_DIRECTORY, "host-dispatch.py"))
    report = memory_report(50, CLAMP_TINY_SLICE_GIBIBYTES)
    slice_total = int(report["memory"]["slice_total"])
    snapshot = types.SimpleNamespace(reports={SERVER.LOCAL: report}, down={}, memory_caps={}, total=lambda host: slice_total)
    limit = (CLAMP_TINY_SLICE_GIBIBYTES - SELFTEST_MARGIN_GIBIBYTES) * GIBIBYTE
    prediction_type = fake.host_dispatch.Prediction
    default = lib.UNRECORDED_MEMORY_DEFAULT_BYTES
    pinned = CLAMP_PINNED_ITEM_BYTES - SELFTEST_MARGIN_GIBIBYTES * GIBIBYTE
    cases = (("钉值", prediction_type("selftest-booking-pinned", pinned, pinned, pinned, lib.PINNED_HISTORY, memory_by_host={SERVER.LOCAL: CLAMP_PINNED_ITEM_BYTES})),
             ("记录", prediction_type("selftest-booking-record", CLAMP_RECORDED_BYTES, CLAMP_RECORDED_BYTES, CLAMP_RECORDED_BYTES, lib.RECORD_HISTORY)),
             ("没有记录的缺省", prediction_type("selftest-booking-default", default, default, default, lib.UNRECORDED_HISTORY)),
             ("撞上限重排", prediction_type("selftest-booking-retry", CLAMP_RETRY_BYTES, CLAMP_RETRY_BYTES, CLAMP_RETRY_BYTES, "retry")))
    problems = []
    for what, prediction in cases:
        if int(prediction.memory_recorded_peak) <= limit:
            problems.append(f"{what}：样本的预测 {prediction.memory_recorded_peak} 字节应当比可用上限 {limit} 字节大（这一格的样本写坏了）")
            continue
        booked = SERVER.Scheduler.booked_at_cap(fake, prediction, SERVER.LOCAL, snapshot)
        over = {name: int(getattr(booked, name) or 0) for name in ("memory_recorded_peak", "memory_cap", "memory_booked") if int(getattr(booked, name) or 0) > limit}
        if over:
            problems.append(f"{what}（{prediction.memory_history}）：预测峰值、上限与记账都应当不大于可用上限 {limit} 字节，实际超过的 {over}")
    if problems:
        raise SelftestFailure("记账收到可用上限：" + "；".join(problems))
    return f"钉值、记录、缺省、撞上限重排 {len(cases)} 条的预测都比可用上限 3G 大：交给 judge_host 的预测峰值、上限与记账都不大于 3G"


def cell_oversized_budget_reported(world):
    """预算贴着整台（task_server.py 文件头 ①「要处置的提醒」的 oversized-budget）：两台 50G（可用上限 48G、界 40.8G），记过峰值 45G 的命令记账 45G（不乘 1.5）：不超过可用上限、不小于整片界，
    照常排、起来跑；wait 打一行 kind=oversized-budget、依据写记账、可用上限与记账从哪来、退 ATTENTION_EXIT；放行之后跑完。
    弄坏开关 oversized-budget-silent 下不报，wait 等到命令自己退，这一格红。"""
    key = "selftest-oversized-budget"
    release = os.path.join(world.root, "release-oversized-budget")
    world.resources(local=(50, 50), peer=(50, 50), peer_cap_gibibytes=50)
    world.seed_peak(key, 45 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra={lib.KEY_VARIABLE: key}, cwd=world.repo)
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    problems, fields = attention_problems(completed, task_id, "oversized-budget", lib.ATTENTION_EXIT)
    basis = fields.get("basis", "")
    if "记账" not in basis or "可用上限" not in basis or "记账从哪来" not in basis or "峰值" not in basis:
        problems.append(f"依据应当写记账、可用上限与记账从哪来（这把命令键记下的峰值），实际「{basis}」")
    open(release, "w").close()
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"只提醒、照跑：放行之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("预算贴着整台：" + "；".join(problems), world.server_log_lines())
    return f"记账 45G 不小于整片界 40.8G：wait 打 kind=oversized-budget（依据「{basis[:80]}」），照跑完"


def cell_oversized_budget_acknowledged_after_restart(world):
    """换服务之后照跑照样记得上（task_server.py 文件头 ⑥「没处置时」）：预算贴着整台的那一条照 oversized-budget-reported 造（记账 45G、界 40.8G），wait 报过 oversized-budget；
    服务停了再起（budget_notices 只在内存，换上之后这一条没有开着的提醒），提交它的会话 confirm --why 退 0、回「已记照跑（oversized-budget）」。
    弄坏开关 anomaly-acknowledge-open-only 下只认开着的，回 409「不用确认」，这一格红。"""
    key = "selftest-oversized-budget-restart"
    release = os.path.join(world.root, "release-oversized-budget-restart")
    world.resources(local=(50, 50), peer=(50, 50), peer_cap_gibibytes=50)
    world.seed_peak(key, 45 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra={lib.KEY_VARIABLE: key}, cwd=world.repo)
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    problems, _ = attention_problems(completed, task_id, "oversized-budget", lib.ATTENTION_EXIT)
    world.stop_server()
    world.start_server()
    confirmed = world.client(["confirm", str(task_id), "--why", LONG_RUN_WHY_SAMPLE])
    said = confirmed.stdout + confirmed.stderr
    if confirmed.returncode != 0 or "已记照跑（oversized-budget）" not in said:
        problems.append(f"换服务之后 confirm --why 应当退 0、回「已记照跑（oversized-budget）」，实际退 {confirmed.returncode}：{said[-300:]}")
    open(release, "w").close()
    world.finished(task_id)
    if problems:
        raise SelftestFailure("换服务之后记照跑：" + "；".join(problems), world.server_log_lines())
    return "oversized-budget 报过、服务停了再起，confirm --why 照样记上照跑"


IO_PRESSURE_SAMPLE = ("some avg10=0.00 avg60=1.50 avg300=2.00 total=10\n"
                      "full avg10=70.00 avg60=80.50 avg300=81.49 total=99\n")   # /proc/pressure/io 的样子：full 那一行的 avg300 是 81.49


def io_pressure_configuration(world, name, limit_text):
    """磁盘压力那几格的多机配置：这一格自己的一份文件（只写 HOSTS 与 TASK_SERVER_IO_PRESSURE_LIMIT；limit_text 是 None 就不写那一行）→ 路径。"""
    path = os.path.join(world.root, f"io-pressure-{name}.env")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(f"{lib.HOSTS_KEY}={SELFTEST_LOCAL_ADDRESS} {SELFTEST_PEER_ADDRESS}\n"
                     + ("" if limit_text is None else f"{lib.IO_PRESSURE_LIMIT_KEY}={limit_text}\n"))
    return path


def cell_io_pressure_gate_by_host(world):
    """磁盘压力闸（task_server.py 的 io_pressure_problem；文件头 ②「磁盘压力」）：多机配置 TASK_SERVER_IO_PRESSURE_LIMIT 写了 50，哪一台的 full avg300 读数超过它，
    会编译的任务在那一台上排着，排队原因以「磁盘压力：」起头并点名那一台、读数与阈值；读数不超过（含恰好等于）的、不会编译的、没写阈值的、这一台读不到读数的都放行；
    两台各判各的读数（本机低、第二台高：第二台排着、本机放行）；阈值写坏了的，会编译的排着、原因点名那个键，不会编译的放行。
    弄坏开关 io-pressure-ignored 下一律放行，io-pressure-applies-to-all 下不会编译的也排，io-pressure-limit-defaulted 下没写阈值的补缺省 50，
    io-pressure-peer-reads-local 下第二台照本机的读数判，io-pressure-wait-unnamed 下排队原因不点名磁盘压力、这一台与读数，这一格红。"""
    def snapshot(local, peer):
        reports = {host: ({} if value is None else {SERVER.IO_PRESSURE_REPORT_KEY: value}) for host, value in ((SERVER.LOCAL, local), (SERVER.PEER, peer))}
        return SERVER.Snapshot(reports, {}, {SERVER.LOCAL: True, SERVER.PEER: True}, {})
    compiling, plain = {"id": 1, "compiles": True}, {"id": 2, "compiles": False}
    problems = []
    for what, limit, host, task, local, peer, blocked in (
            ("本机 81.49 超过阈值 50、会编译的", "50", SERVER.LOCAL, compiling, 81.49, 5.0, True),
            ("本机 10 低于阈值 50", "50", SERVER.LOCAL, compiling, 10.0, 5.0, False),
            ("本机恰好等于阈值 50", "50", SERVER.LOCAL, compiling, 50.0, 5.0, False),
            ("本机 81.49 超过阈值、不会编译的", "50", SERVER.LOCAL, plain, 81.49, 5.0, False),
            ("没写阈值、本机 99", None, SERVER.LOCAL, compiling, 99.0, 99.0, False),
            ("第二台 81.49、本机 10：判第二台", "50", SERVER.PEER, compiling, 10.0, 81.49, True),
            ("第二台 81.49、本机 10：判本机", "50", SERVER.LOCAL, compiling, 10.0, 81.49, False),
            ("本机读不到读数", "50", SERVER.LOCAL, compiling, None, 5.0, False)):
        fake = object.__new__(SERVER.Scheduler)
        fake.environment = dict(os.environ)
        fake.resources = types.SimpleNamespace(configuration_path=io_pressure_configuration(world, "gate", limit))
        why = SERVER.Scheduler.io_pressure_problem(fake, task, host, snapshot(local, peer))
        reading = local if host == SERVER.LOCAL else peer
        if blocked:
            wanted = (lib.HOST_NAMES[host], f"{reading:.2f}", f"登记的 {float(limit):g}%")
            if not why.startswith("磁盘压力：") or any(word not in why for word in wanted):
                problems.append(f"{what}：应当排着、原因以「磁盘压力：」起头并写 {list(wanted)}，实际「{why}」")
        elif why:
            problems.append(f"{what}：应当放行，实际排队原因「{why}」")
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.resources = types.SimpleNamespace(configuration_path=io_pressure_configuration(world, "bad", "abc"))
    for what, task, blocked in (("阈值写坏、会编译的", compiling, True), ("阈值写坏、不会编译的", plain, False)):
        why = SERVER.Scheduler.io_pressure_problem(fake, task, SERVER.LOCAL, snapshot(81.49, 5.0))
        if blocked != (lib.IO_PRESSURE_LIMIT_KEY in why and why.startswith("磁盘压力：")) or (not blocked and why):
            problems.append(f"{what}：应当{'排着、原因点名 ' + lib.IO_PRESSURE_LIMIT_KEY if blocked else '放行'}，实际「{why}」")
    if problems:
        raise SelftestFailure("磁盘压力闸：" + "；".join(problems))
    return "读数超过阈值的那一台上会编译的排着、原因点名磁盘压力；不超、不会编译、没写阈值、读不到的放行；两台各判各的；阈值写坏的会编译的排着"


def cell_io_pressure_reading_recorded(world):
    """这台的磁盘压力读数记进余量报告（task_server.py 的 Resources.note_io_pressure、io_pressure_full_avg300；task_server_lib.py 的 io_pressure_limit、parse_optional_keys）：
    /proc/pressure/io 里 full 那一行的 avg300 取 81.49（some 那一行不算，没有 full 行的交 None）；多机配置写了阈值的才读：假读数（自证的 JSON）与机器接口（Machine.io_pressure）
    各读到 81.49 记进报告的 IO_PRESSURE_REPORT_KEY，读不到的不记；没写阈值的不读、不记。阈值要写成大于 0、不超过 100 的数，别的写法读多机配置时判坏、点名那个键。
    弄坏开关 io-pressure-limit-defaulted 下没写阈值的也读，io-pressure-limit-unchecked 下写坏的阈值读多机配置时不判坏，这一格红。"""
    problems = []
    if SERVER.io_pressure_full_avg300(IO_PRESSURE_SAMPLE) != 81.49 or SERVER.io_pressure_full_avg300("some avg10=0.00 avg60=1.50 avg300=2.00 total=10\n") is not None:
        problems.append("读数应当取 full 那一行的 avg300（81.49），只有 some 行的交 None")
    for what, limit, fake_json, machine_answer, wanted in (
            ("写了阈值、假读数", "50", {"io_pressure": {SERVER.LOCAL: IO_PRESSURE_SAMPLE}}, (1, "", "不该走到机器接口"), 81.49),
            ("写了阈值、读机器接口", "50", None, (0, IO_PRESSURE_SAMPLE, ""), 81.49),
            ("没写阈值、不读", None, None, (0, IO_PRESSURE_SAMPLE, ""), None),
            ("写了阈值、机器接口读不到", "50", None, (1, "", "读不了"), None)):
        resources = object.__new__(SERVER.Resources)
        resources.environment = dict(os.environ)
        resources.configuration_path = io_pressure_configuration(world, "reading", limit)
        machine = types.SimpleNamespace(name=SERVER.LOCAL, label="本机", io_pressure=lambda found, answer=machine_answer: answer)
        report = {}
        SERVER.Resources.note_io_pressure(resources, machine, report, fake_json)
        if report.get(SERVER.IO_PRESSURE_REPORT_KEY) != wanted:
            problems.append(f"{what}：报告里的读数应当是 {wanted}，实际 {report.get(SERVER.IO_PRESSURE_REPORT_KEY)}")
    path = os.path.join(world.root, "io-pressure-limit-forms.env")
    for text, accepted in (("50", True), ("0.5", True), ("100", True), ("0", False), ("101", False), ("abc", False), ("-5", False), ("5e1", False), ("", False)):
        _, found = lib.parse_optional_keys(f"{lib.IO_PRESSURE_LIMIT_KEY}={text}\n", path)
        if bool(found) == accepted or (not accepted and found and lib.IO_PRESSURE_LIMIT_KEY not in found[0]):
            problems.append(f"阈值写「{text}」：应当{'认' if accepted else '判坏并点名 ' + lib.IO_PRESSURE_LIMIT_KEY}，实际问题 {found}")
    if problems:
        raise SelftestFailure("磁盘压力读数：" + "；".join(problems))
    return "full 行 avg300 记进报告（写了阈值才读，读不到不记）；阈值只认大于 0、不超过 100 的数"


def cell_io_pressure_queues_compiling_task(world):
    """磁盘压力高时会编译的任务排着、不会编译的照放、没写阈值不按压力排（task_server.py 的 io_pressure_problem、host_admission；真服务，假余量）：
    多机配置写了 TASK_SERVER_IO_PRESSURE_LIMIT=50、本机读数 81.49：cargo build 排着（status 的「在等：」点名磁盘压力、本机与读数），一条不编译的命令照跑完；
    读数降到 10 之后排着的那条跑完；配置那一行删掉、读数 95：会编译的照跑完。
    弄坏开关 io-pressure-ignored 下排不住，io-pressure-applies-to-all 下不编译的也排，io-pressure-limit-defaulted 下没写阈值的也按 50 排，
    io-pressure-wait-unnamed 下排队原因不点名磁盘压力，这一格红。
    不编译的那一条排在没起的会编译的那一条后面，要起就是插队（jump_refusal：没有上次运行时长的不插队）：它在库里先记了一次跑完的运行时长，
    排队原因就只剩磁盘压力那一道可能挡它。"""
    problems = []
    world.set_io_pressure_limit("50")
    world.resources(local=(30, 30), local_io_pressure=81.49)
    plain_command = ["bash", "-c", "exit 0", "io-pressure-plain"]
    world.seed_duration("selftest-io-pressure-plain", 1, plain_command)
    world.start_server()
    path = f"{world.fake_cargo}:{os.environ.get('PATH', '/usr/bin:/bin')}"
    compiling = world.submit(["cargo", "build"], extra={"PATH": path})
    plain = world.submit(plain_command, extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-io-pressure-plain"})
    try:
        plain_summary = world.finished(plain, timeout=PLACEMENT_WAIT_SECONDS)
    except SelftestFailure as failure:
        plain_summary = None
        problems.append(f"不会编译的命令应当照放、跑完，实际 {failure}")
    world.next_pass()
    world.next_pass()
    held = world.summary(compiling)
    reason = held.get("waiting_for") or ""
    if held["task"].get("state") != "queued" or held["attempts"] or "磁盘压力" not in reason or "本机" not in reason or "81.49" not in reason:
        problems.append(f"读数 81.49 超过阈值 50：会编译的应当排着、没起尝试、排队原因点名磁盘压力、本机与读数，实际 {held['task'].get('state')}、{len(held['attempts'])} 次尝试、在等「{reason}」")
    if plain_summary is not None and plain_summary["task"].get("outcome") != "succeeded":
        problems.append(f"不会编译的命令应当跑完，实际 {describe(plain_summary)}")
    world.resources(local=(30, 30), local_io_pressure=10.0)
    try:
        lowered = world.finished(compiling, timeout=PLACEMENT_WAIT_SECONDS)
        if lowered["task"].get("outcome") != "succeeded":
            problems.append(f"读数降到 10 之后会编译的应当跑完，实际 {describe(lowered)}")
    except SelftestFailure as failure:
        problems.append(f"读数降到 10 之后会编译的应当跑完，实际 {failure}")
    world.set_io_pressure_limit(None)
    world.resources(local=(30, 30), local_io_pressure=95.0)
    unconfigured = world.submit(["cargo", "build"], extra={"PATH": path})
    try:
        summary = world.finished(unconfigured, timeout=PLACEMENT_WAIT_SECONDS)
        if summary["task"].get("outcome") != "succeeded":
            problems.append(f"没写阈值、读数 95：会编译的应当照跑完，实际 {describe(summary)}")
    except SelftestFailure as failure:
        problems.append(f"没写阈值、读数 95：会编译的应当照放、跑完，实际 {failure}")
    if problems:
        raise SelftestFailure("磁盘压力高时会编译的任务排着：" + "；".join(problems), world.server_log_lines())
    return f"读数 81.49 超过阈值 50：会编译的排着、在等「{reason[:80]}」，不编译的跑完；降到 10 跑完；没写阈值读数 95 也跑完"


def cell_budget_source_names_folded_key(world):
    """记录来源写出命令键原文，键里有折掉的点名项时写明同一入口各项共用这一份记录、怎么按项记（task_server.py 的 folded_key_note）；没折的只写键原文。
    套在假调度器上，不起服务。弄坏开关 folded-key-unnamed 下两样都不写，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.host_dispatch = lib.load_module("host_dispatch_for_folded_key_cell", os.path.join(lib.HOSTS_DIRECTORY, "host-dispatch.py"))
    token = fake.host_dispatch.peaks.FOLD_TOKENS[0]
    folded = SERVER.Scheduler.folded_key_note(fake, {"command_key": f"bash .claude/gate.d/sample.sh --item {token}"})
    plain = SERVER.Scheduler.folded_key_note(fake, {"command_key": "bash .claude/gate.d/sample.sh --check one"})
    problems = []
    if f"--item {token}」里的 {token}" not in folded or lib.KEY_VARIABLE not in folded or "ITEM_KEPT_CELLS" not in folded:
        problems.append(f"键里折过项时应当写键原文、点名 {token}、出路带 {lib.KEY_VARIABLE} 与 ITEM_KEPT_CELLS，实际「{folded}」")
    if plain != "；命令键「bash .claude/gate.d/sample.sh --check one」":
        problems.append(f"没折的键应当只写键原文，实际「{plain}」")
    if problems:
        raise SelftestFailure("记录来源写不写命令键：" + "；".join(problems))
    return f"记录来源写出命令键原文，折过项的写明各项共用记录与出路（「{folded[:80]}…」）"


# running-booked-at-measured-usage：(说什么, 上限, 此刻 scope 用量, 该记多少, 取的是哪一个, 公式剩多少, 待涨量)，单位 GiB；slice 30G、余量 1G，公式的总额就是总上限 30G
RUNNING_BOOKING_CASES = (("上限 20G、此刻 2G", 20, 2, 2, SERVER.BOOKED_FROM_USAGE, 27, 0),
                         ("上限 20G、此刻 12G", 20, 12, 12, SERVER.BOOKED_FROM_USAGE, 17, 0),
                         ("上限 8G、此刻 9G", 8, 9, 8, SERVER.BOOKED_FROM_CAP, 21, 0))


def cell_running_booked_at_measured_usage(world):
    """在跑的有 scope 的尝试只照此刻实测记账、不超过上限、不加固定余量、不看任何预先估的数（task_server.py 的 running_booking、room_by_the_formula、server_pending；
    文件头 ②「内存」的在跑记账）：本机 slice 30G、余量 1G，一条在跑（RUNNING_BOOKING_CASES 三种）——上限 20G、此刻 2G 记 2G（measured）、公式剩 27G、排头 12G 放得下、待涨量 0；
    此刻 12G 记 12G（measured）、公式剩 17G；上限 8G、此刻 9G 记上限 8G（cap）、公式剩 21G。
    弄坏开关 running-booked-at-cap 下头一种记 20G，running-booked-with-margin 下记 3G，这一格红。"""
    dispatch = loaded_host_dispatch()
    head = types.SimpleNamespace(memory_booked=12 * GIBIBYTE)
    fits = dispatch.Verdict(SERVER.LOCAL, True, False, 12 * GIBIBYTE, "内存放得下")
    problems, shown = [], []
    for what, cap, used, want, basis, want_room, want_pending in RUNNING_BOOKING_CASES:
        fake, snapshot, report = running_booking_scheduler(used * GIBIBYTE, used * GIBIBYTE, cap_bytes=cap * GIBIBYTE)
        booked, got_basis = SERVER.Scheduler.running_booking(fake, fake.store.running_attempts()[0], SERVER.Scheduler.report_scope_usage(fake, report))
        room = SERVER.Scheduler.room_by_the_formula(fake, SERVER.LOCAL, snapshot)
        verdict = SERVER.Scheduler.verdict_within_the_formula(fake, fits, head, SERVER.LOCAL, snapshot)
        pending = SERVER.Scheduler.server_pending(fake, SERVER.LOCAL, report)
        if (booked, got_basis, room, pending) != (want * GIBIBYTE, basis, want_room * GIBIBYTE, want_pending * GIBIBYTE) or not verdict.fits:
            problems.append(f"{what}：应当记 {want}G（{basis}）、公式剩 {want_room}G、排头 12G 放得下、待涨量 {want_pending}G，"
                            f"实际记 {booked / GIBIBYTE:g}G（{got_basis}）、公式剩 {room / GIBIBYTE:g}G、判决 {verdict}、待涨量 {pending / GIBIBYTE:g}G")
        shown.append(f"{what}记 {want}G（{basis}）")
    if problems:
        raise SelftestFailure("在跑的只照此刻实测记账：" + "；".join(problems))
    return "；".join(shown) + "；排头 12G 都放得下"


def cell_running_young_booked_at_cap(world):
    """起跑不满两分钟的在跑尝试照整份上限记（task_server.py 的 running_booking、running_booking_text；任务 15262 到 15299 共 38 条十秒内全起那一回）：
    时限照 120 秒，上限 20G、此刻 2G 的一条——起跑 10 秒的记 20G（young），为什么那一句写「不算漂移」；起跑 200 秒的记实测 2G（measured）。套在假调度器上，不起服务。
    弄坏开关 running-young-booked-by-usage 下起跑 10 秒的记 2G，这一格红。"""
    problems = []
    for age, want, basis in ((10, 20, SERVER.BOOKED_WHILE_YOUNG), (200, 2, SERVER.BOOKED_FROM_USAGE)):
        fake, _, report = running_booking_scheduler(2 * GIBIBYTE, 2 * GIBIBYTE)
        fake.environment = dict(fake.environment, **{lib.YOUNG_BOOKING_VARIABLE: "120"})
        attempt = dict(fake.store.running_attempts()[0], started_at=time.time() - age)
        usage = SERVER.Scheduler.report_scope_usage(fake, report)
        booked, got = SERVER.Scheduler.running_booking(fake, attempt, usage)
        why = SERVER.Scheduler.running_booking_text(got, usage.get(RUNNING_BOOKING_LABEL), 20 * GIBIBYTE)
        if (booked, got) != (want * GIBIBYTE, basis) or (basis == SERVER.BOOKED_WHILE_YOUNG and "不算漂移" not in why):
            problems.append(f"起跑 {age} 秒的应当记 {want}G（{basis}），刚起的那一句写「不算漂移」，实际记 {booked / GIBIBYTE:g}G（{got}）、「{why}」")
    if problems:
        raise SelftestFailure("起跑不满两分钟照整份上限记：" + "；".join(problems))
    return "上限 20G、此刻 2G：起跑 10 秒的记 20G（young，不算漂移）；起跑 200 秒的记实测 2G"


def cell_running_booked_unscoped_at_cap(world):
    """报告里没有在跑尝试的 scope（刚起、读不了）的照整份上限记账（task_server.py 的 running_booked、room_by_the_formula、verdict_within_the_formula）：本机 slice 30G、余量 1G，
    一条在跑、上限 20G、报告里没有它的 scope：记 20G，公式剩 9G，排头 25G 放不下、原因写「在跑的记账之和」。slice 兜底（slice_floor_problem）：slice 实测只剩 3G
    （不到 SLICE_ROOM_FLOOR_BYTES）这一轮不派、原因以「slice 兜底」起头，剩 20G 的不挡。
    弄坏开关 slice-floor-ignored 下不判 slice 兜底，这一格红。"""
    dispatch = loaded_host_dispatch()
    head = types.SimpleNamespace(memory_booked=25 * GIBIBYTE, memory_recorded_peak=25 * GIBIBYTE, memory_cap=25 * GIBIBYTE)
    fits = dispatch.Verdict(SERVER.LOCAL, True, False, 25 * GIBIBYTE, "内存放得下")
    problems = []
    fake, snapshot, _ = running_booking_scheduler(GIBIBYTE // 2, None)
    room = SERVER.Scheduler.room_by_the_formula(fake, SERVER.LOCAL, snapshot)
    verdict = SERVER.Scheduler.verdict_within_the_formula(fake, fits, head, SERVER.LOCAL, snapshot)
    if room != 9 * GIBIBYTE or verdict.fits or SERVER.RUNNING_BOOKED_SUM_TEXT not in verdict.why:
        problems.append(f"报告里没有它的 scope：应当照上限 20G 记、公式剩 9G、排头 25G 放不下、原因写「{SERVER.RUNNING_BOOKED_SUM_TEXT}」，实际公式剩 {room}、判决 {verdict}")
    for used, held in ((27, True), (10, False)):
        fake, snapshot, _ = running_booking_scheduler(GIBIBYTE // 2, int(0.4 * GIBIBYTE), slice_used_gibibytes=used)
        why = SERVER.Scheduler.slice_floor_problem(fake, SERVER.LOCAL, snapshot)
        if held != why.startswith(SERVER.SLICE_FLOOR_TEXT):
            problems.append(f"slice 里进程用 {used}G（总上限 30G）：应当{'停派、原因以「' + SERVER.SLICE_FLOOR_TEXT + '」起头' if held else '不挡'}，实际「{why}」")
    if problems:
        raise SelftestFailure("没有 scope 的照上限记账、slice 兜底：" + "；".join(problems))
    return "读不到 scope 的照上限 20G 记、公式剩 9G、排头 25G 放不下；slice 只剩 3G 时停派"


ADMISSION_PROBE = ("selftest-admission-probe", 10)   # admission-by-measured-bookings 与 memory-wait-names-both-numbers 再来的那一条：(命令键, 上限与记账 GiB)
# 那两格的三种在跑：(说什么, [(上限, scope 此刻用量)]（GiB）, slice 里进程用的 GiB)；slice 总额 50G、余量 1G、整机空闲 100G
ADMISSION_ROOMY = ("上限之和 48G、实测只占 6G", [(12, 1.5)] * 4, 6)
ADMISSION_CAPS_OVER_TOTAL = ("上限之和 70G（比总额 50G 大）、实测只占 6G", [(14, 1.2)] * 5, 6)
ADMISSION_MEASURED_TIGHT = ("上限之和 48G、实测占 42G（slice 实测余量 8G）", [(12, 10.5)] * 4, 42)


def admission_scheduler(running, slice_used_gibibytes):
    """照实测记账准入那两格的假调度器（真的 Scheduler 方法，套在假的库与余量报告上）：本机 slice 总额 50G、余量 1G、整机空闲 100G，在跑的 running 是 [(上限, scope 此刻用量)]（GiB），
    任务号 701 起依次编，slice 里进程用 slice_used_gibibytes。→ (假调度器, Snapshot, 报告)。"""
    attempts = [{"id": number, "task_id": 700 + number, "host": SERVER.LOCAL, "label": f"task-selftest-admission-{number}", "memory_cap": int(cap * GIBIBYTE)}
                for number, (cap, _) in enumerate(running, 1)]
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.peaks = PEAKS
    fake.host_dispatch = loaded_host_dispatch()
    fake.store = types.SimpleNamespace(running_attempts=lambda: [dict(attempt) for attempt in attempts])
    fake.host_margin = lambda host, snapshot: (GIBIBYTE, "")
    fake.slice_room_floor = SERVER.SLICE_ROOM_FLOOR_BYTES
    fake.unit_stopped = set()
    report = memory_report(100, 50, slice_used_gibibytes=slice_used_gibibytes,
                           scopes={attempt["label"]: int(used * GIBIBYTE) for attempt, (_, used) in zip(attempts, running)})
    return fake, SERVER.Snapshot({SERVER.LOCAL: report}, {}, {SERVER.LOCAL: True}, {}), report


def admission_judged(case):
    """照 case（ADMISSION_* 之一）建假调度器，判再来的那一条（ADMISSION_PROBE）：judged_on_host（judge_host、公式）再过 slice_floor_verdict，同 host_admission 的内存那几步。
    → (假调度器, Snapshot, 这台上的记账, 判决)。"""
    _, running, slice_used = case
    fake, snapshot, report = admission_scheduler(running, slice_used)
    key, size = ADMISSION_PROBE
    prediction = fake.host_dispatch.Prediction(key, size * GIBIBYTE, size * GIBIBYTE, size * GIBIBYTE, lib.RECORD_HISTORY)
    pending = SERVER.Scheduler.server_pending(fake, SERVER.LOCAL, report)
    on_host, verdict = SERVER.Scheduler.judged_on_host(fake, {"id": 1, "command_key": key}, prediction, SERVER.LOCAL, snapshot, pending)
    return fake, snapshot, on_host, SERVER.Scheduler.slice_floor_verdict(fake, verdict, on_host, SERVER.LOCAL, snapshot)


def cell_admission_by_measured_bookings(world):
    """准入只看在跑的实测记账之和，在跑的上限之和多大都不挡（task_server.py 的 running_booking、room_by_the_formula、judged_on_host；文件头 ②「内存」的公式；
    2026-11-12 两台各记 73G、74G 而实占 2.5G、4.6G，排着的几十条起不来）：本机 slice 总额 50G、余量 1G，再来一条上限 10G 的——
    在跑上限之和 48G、实测只占 6G：起得来；上限之和 70G（比总额大）、实测只占 6G：照样起得来（50 − 1 − 6 ≥ 10）；上限之和 48G 而实测占 42G：排着，
    原因写「账面还能记 7.0 GiB」。套在假调度器上，不起服务。
    弄坏开关 running-booked-at-cap 下第二种照上限之和 70G 记、放不下，这一格红。"""
    problems, shown = [], []
    for case, fits in ((ADMISSION_ROOMY, True), (ADMISSION_CAPS_OVER_TOTAL, True), (ADMISSION_MEASURED_TIGHT, False)):
        _, _, _, verdict = admission_judged(case)
        if verdict.fits != fits:
            problems.append(f"{case[0]}：上限 10G 的应当{'起得来' if fits else '排着'}，实际判决 {verdict}")
        shown.append(f"{case[0]}：{'起得来' if verdict.fits else '排着'}")
    _, _, _, tight = admission_judged(ADMISSION_MEASURED_TIGHT)
    if "账面还能记 7.0 GiB" not in tight.why:
        problems.append(f"{ADMISSION_MEASURED_TIGHT[0]}：原因应当写「账面还能记 7.0 GiB」，实际「{tight.why}」")
    if problems:
        raise SelftestFailure("准入只看实测记账：" + "；".join(problems))
    return "；".join(shown)


def cell_memory_wait_names_both_numbers(world):
    """内存放不下的排队原因写明两个数各是什么、点名记账最多的在跑任务（task_server.py 的 memory_wait_text、memory_verdict_reworded、verdict_within_the_formula；
    judge_host 那一句的「此刻空闲」是整机空闲减待涨量，看着像机器的可用内存）：本机 slice 总额 50G、余量 1G，在跑四条各上限 12G、各实占 10.5G，再来一条 10G 的——
    原因写「账面还能记 7.0 GiB」（连同总上限、余量与在跑的记账之和 42.0 GiB）、「slice 实测还放得下 8.0 GiB」，点名记账最多的两条（任务 701、702 各记 10.5 GiB、实占 10.5 GiB），
    第三条不点名，不写「此刻空闲」、不写乘几；judge_host 放得下、只有公式放不下的那一判（verdict_within_the_formula）写同一句。套在假调度器上，不起服务。
    弄坏开关 memory-wait-text-unnamed 下照 judge_host 与公式原来那一句写，这一格红。"""
    fake, snapshot, on_host, verdict = admission_judged(ADMISSION_MEASURED_TIGHT)
    wanted = ("账面还能记 7.0 GiB", f"{SERVER.RUNNING_BOOKED_SUM_TEXT} 42.0 GiB", "slice 实测还放得下 8.0 GiB",
              "任务 701 记 10.5 GiB、实占 10.5 GiB", "任务 702 记 10.5 GiB、实占 10.5 GiB")
    problems = []
    missing = [piece for piece in wanted if piece not in verdict.why]
    if verdict.fits or missing or "任务 703" in verdict.why or "此刻空闲" in verdict.why or "×" in verdict.why:
        problems.append(f"judge_host 判放不下：原因应当写 {'、'.join(wanted)}、不点名任务 703、不写「此刻空闲」、不写乘几，缺 {missing}，实际「{verdict.why}」")
    fits = fake.host_dispatch.Verdict(SERVER.LOCAL, True, False, on_host.memory_cap, "内存放得下")
    formula = SERVER.Scheduler.verdict_within_the_formula(fake, fits, on_host, SERVER.LOCAL, snapshot)
    if formula.fits or "账面还能记 7.0 GiB" not in formula.why or "slice 实测还放得下 8.0 GiB" not in formula.why:
        problems.append(f"只有公式放不下：原因应当写账面还能记 7.0 GiB 与 slice 实测还放得下 8.0 GiB，实际 {formula}")
    if problems:
        raise SelftestFailure("内存放不下的排队原因：" + "；".join(problems))
    return f"原因写「{verdict.why[:160]}」"


def cell_budget_source_only_when_reported(world):
    """预算那一判不记提醒时不为「记账从哪来」扫库（task_server.py 的 budget_check；loop 审核发现 1）：记 4G、本机可用上限 48G、整片界 40G：
    不记 oversized-budget、一次都不算记账来源。弄坏开关 budget-source-every-pass 下每一轮都算，这一格红。"""
    calls = []
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.host_dispatch = types.SimpleNamespace(human=lambda size: f"{size / GIBIBYTE:.1f} GiB")
    fake.prediction_of = lambda task: types.SimpleNamespace(memory_recorded_peak=4 * GIBIBYTE)
    fake.candidate_hosts = lambda task, snapshot: [SERVER.LOCAL]
    fake.host_limit = lambda prediction, host, snapshot: (48 * GIBIBYTE, "")
    fake.booked_at_cap = lambda prediction, host, snapshot: types.SimpleNamespace(memory_booked=4 * GIBIBYTE)
    fake.whole_threshold = lambda host, snapshot: 40 * GIBIBYTE
    fake.budget_notices = {}
    fake.budget_source = lambda task, prediction: calls.append(task["id"]) or "自证"
    snapshot = types.SimpleNamespace(reports={SERVER.LOCAL: {}}, reachable={SERVER.LOCAL: True, SERVER.PEER: True})
    SERVER.Scheduler.budget_check(fake, {"id": 7}, types.SimpleNamespace(snapshot=lambda: snapshot))
    if fake.budget_notices or calls:
        raise SelftestFailure(f"记 4G、可用上限 48G：应当不记提醒、不算记账来源，实际提醒 {fake.budget_notices}、算了 {len(calls)} 次")
    return "不记提醒的一条不算记账来源"


DRIFT_LABELS = ("task-selftest-drift-1", "task-selftest-drift-2")   # booking-drift-reported：两条在跑尝试的标签
DRIFT_QUEUED_TASK = 90   # 同一格里被挡住、还在排的那条任务


def booking_audit_scheduler(attempts):
    """booking-drift-reported 的假调度器：本机 slice 30G，在跑的 attempts（各上限 12G），排着一条任务（被「在跑的记账之和」挡着、记着会话令牌）；
    send_to_session 记下发出去的正文。→ (假调度器, 发出去的正文表)。"""
    fake, sent = object.__new__(SERVER.Scheduler), []
    queued = {"id": DRIFT_QUEUED_TASK, "state": "queued", "session": SESSION_A, "inbox": {"token": INBOX_TOKEN_SAMPLE}}
    fake.environment, fake.peaks = dict(os.environ), PEAKS
    fake.host_dispatch = loaded_host_dispatch()   # audit_bookings 末了的记账明细（refresh_booking_ledger）要读余量报告里的余量
    fake.machines = {SERVER.LOCAL: types.SimpleNamespace(label="本机")}
    fake.store = types.SimpleNamespace(running_attempts=lambda: [dict(attempt) for attempt in attempts],
                                       select=lambda table, where="1", parameters=(), order="id": [(queued, True)] if table == "tasks" else [],
                                       task=lambda task_id: (queued, True) if task_id == DRIFT_QUEUED_TASK else None,
                                       write_restart_surviving_state=lambda name, state: None)   # 推到的提醒门槛记进库（persist_restart_surviving_state）：这里不起库，写成空
    fake.restart_surviving_lock, fake.restart_surviving_written = threading.Lock(), {}
    fake.unit_stopped = set()
    fake.booking_audit_state, fake.booking_drift, fake.booking_drift_notified = {}, {"memory": {}}, {}
    fake.booking_drift_lock = threading.Lock()
    fake.waiting_reasons = {DRIFT_QUEUED_TASK: f"内存：要记 12288 MiB，配置的总上限减余量减{SERVER.RUNNING_BOOKED_SUM_TEXT}只剩 0 MiB"}
    fake.inbox_reachable = lambda task: True
    fake.send_to_session = lambda task, text, event: sent.append(text) or True
    return fake, sent


def cell_booking_drift_reported(world):
    """服务自己定时对账（task_server.py 的 audit_bookings、audit_memory_bookings、booking_row、push_booking_drift；不等别的会话汇报）：
    有 scope 的在跑尝试照实测记账，对账的基准是各 scope 实测之和，漂移只来自报告里没有它的 scope、记整份上限的那几条。
    本机两条在跑各上限 12G、报告里都没有它们的 scope（各记 12G、实占读不到，实测合计 0），
    连着三轮：头两轮不判，第三轮判记账漂移，/health 的 booking_drift 一栏（booking_drift_snapshot）列出两条各记 12G、实占读不到、为什么（报告里没有它的 scope，记整份上限）；
    推给排着的任务所在的会话恰好一次（正文写「记账漂移」与两条的记账与实占），再对两轮不重推；过了门槛（600 秒）再推一回、首行写再推一回；
    报告里有 scope、各实占 11G 的照实测各记 11G，不报、不推。
    弄坏开关 booking-audit-skipped 下不对账，这一格红。"""
    gib, problems = GIBIBYTE, []
    attempts = [{"id": number, "task_id": 70 + number, "host": SERVER.LOCAL, "label": label, "memory_cap": 12 * gib} for number, label in enumerate(DRIFT_LABELS, 1)]
    start = time.time()

    def audit(fake, used_each, rounds, first=0):
        """对 rounds 轮账；used_each 是报告里每条的 scope 回收不掉的用量，None 是报告里没有这两条的 scope。"""
        report = memory_report(30, 30, scopes={} if used_each is None else {label: used_each for label in DRIFT_LABELS})
        snapshot = SERVER.Snapshot({SERVER.LOCAL: report}, {}, {SERVER.LOCAL: True}, {})
        for number in range(first, first + rounds):
            SERVER.Scheduler.audit_bookings(fake, snapshot, start + 60 * number)

    fake, sent = booking_audit_scheduler(attempts)
    audit(fake, None, 2)
    early = SERVER.Scheduler.booking_drift_snapshot(fake)["memory"]
    audit(fake, None, 1, first=2)
    drift = SERVER.Scheduler.booking_drift_snapshot(fake)["memory"].get(SERVER.LOCAL) or {}
    rows = drift.get("attempts") or []
    if early or [(row["label"], row["booked"], row["used"]) for row in rows] != [(label, 12 * gib, None) for label in DRIFT_LABELS]:
        problems.append(f"连着三轮记 12G、报告里没有 scope：前两轮不判、第三轮判漂移并逐条列两条的记账与实占（读不到），"
                        f"实际前两轮 {bool(early)}、第三轮 {[(row.get('label'), row.get('booked'), row.get('used')) for row in rows]}")
    if rows and not all("没有它的 scope" in row["why"] and "12.0G" in row["why"] for row in rows):
        problems.append(f"逐条应当写为什么（报告里没有它的 scope，记整份上限 12.0G），实际 {[row['why'] for row in rows]}")
    if len(sent) != 1 or "记账漂移" not in sent[0] or not all(label in sent[0] for label in DRIFT_LABELS) or "12.0G" not in sent[0] or "读不到" not in sent[0]:
        problems.append(f"判出的当场应当推给被挡住的会话一回、正文写记账漂移与两条的记账与实占，实际推了 {len(sent)} 回：{[text[:200] for text in sent]}")
    audit(fake, None, 2, first=3)
    if len(sent) != 1:
        problems.append(f"同一件事同一道门槛不重推：再对两轮之后应当还是 1 回，实际 {len(sent)} 回")
    audit(fake, None, 1, first=2 + int(SERVER.BOOKING_DRIFT_PUSH_SECONDS // 60) + 1)
    if len(sent) != 2 or "再推一回" not in sent[-1].split("\n", 1)[0]:
        problems.append(f"过了门槛（{SERVER.BOOKING_DRIFT_PUSH_SECONDS:g} 秒）应当再推一回、首行写再推一回，实际共 {len(sent)} 回：{[text.splitlines()[0] for text in sent]}")
    calm, calm_sent = booking_audit_scheduler(attempts)
    audit(calm, 11 * gib, 5)
    if SERVER.Scheduler.booking_drift_snapshot(calm)["memory"] or calm_sent:
        problems.append(f"报告里有 scope、各实占 11G（照实测各记 11G）：不该报、不该推，实际 {SERVER.Scheduler.booking_drift_snapshot(calm)['memory']}、推了 {len(calm_sent)} 回")
    if problems:
        raise SelftestFailure("服务定时对账：" + "；".join(problems))
    return "报告里没有 scope 的两条各记 12G 连三轮：判记账漂移、逐条列记账实占与原因、推给排着的会话一回（过了门槛再推）；有 scope 的照实测记、不报"


YOUNG_AUDIT_SECONDS = "120"     # booking-audit-young-not-drift：起跑不满这么多秒照整份上限记（YOUNG_BOOKING_VARIABLE），同 running_booking 的两分钟
YOUNG_AUDIT_AGE_SECONDS = 10    # 两条在跑尝试起跑了这么久：在时限之内
YOUNG_AUDIT_USED_BYTES = 512 * MEBIBYTE   # 报告里每条 scope 此刻回收不掉的用量：两条合 1G，记整份上限的 24G 是它的 24 倍、多出 23G，照记账算就够漂移的界


def cell_booking_audit_young_not_drift(world):
    """刚起的不报漂移，对账那一层（task_server.py 的 audit_memory_bookings；文件头 ②「内存」的记账对账；running_booking 那一层由 running-young-booked-at-cap 判）：
    本机两条在跑各上限 12G、起跑 10 秒（时限 120 秒）、报告里有各自的 scope、各实占 512M：booking_row 照规矩记整份上限 12G（young），
    连着对 BOOKING_DRIFT_ROUNDS 轮再多两轮账：对账照实占算，不判记账漂移、/health 的 booking_drift 一栏空着、不推给排着的会话。套在假调度器上，不起服务。
    弄坏开关 booking-audit-young-counted 下对账照整份上限算，第三轮判漂移、推一回，这一格红。"""
    gib = GIBIBYTE
    start = time.time()
    attempts = [{"id": number, "task_id": 70 + number, "host": SERVER.LOCAL, "label": label, "memory_cap": 12 * gib, "started_at": start - YOUNG_AUDIT_AGE_SECONDS}
                for number, label in enumerate(DRIFT_LABELS, 1)]
    fake, sent = booking_audit_scheduler(attempts)
    fake.environment = dict(fake.environment, **{lib.YOUNG_BOOKING_VARIABLE: YOUNG_AUDIT_SECONDS})
    report = memory_report(30, 30, scopes={label: YOUNG_AUDIT_USED_BYTES for label in DRIFT_LABELS})
    snapshot = SERVER.Snapshot({SERVER.LOCAL: report}, {}, {SERVER.LOCAL: True}, {})
    usage = SERVER.Scheduler.report_scope_usage(fake, report)
    problems = []
    rows = [SERVER.Scheduler.booking_row(fake, attempt, usage) for attempt in attempts]
    if [(row["booked"], row["basis"]) for row in rows] != [(12 * gib, SERVER.BOOKED_WHILE_YOUNG)] * len(attempts):
        problems.append(f"起跑 {YOUNG_AUDIT_AGE_SECONDS} 秒的两条应当照规矩记整份上限 12G（{SERVER.BOOKED_WHILE_YOUNG}；这一格的样本前提），"
                        f"实际 {[(row['booked'], row['basis']) for row in rows]}")
    rounds = SERVER.BOOKING_DRIFT_ROUNDS + 2
    for number in range(rounds):
        SERVER.Scheduler.audit_bookings(fake, snapshot, start + 60 * number)
    drift = SERVER.Scheduler.booking_drift_snapshot(fake)["memory"]
    if drift or sent:
        problems.append(f"刚起的两条各记 12G、各实占 512M，连对 {rounds} 轮：对账照实占算，不该判漂移、不该推，"
                        f"实际 booking_drift {drift}、推了 {len(sent)} 回：{[text[:200] for text in sent]}")
    if problems:
        raise SelftestFailure("刚起的对账不报漂移：" + "；".join(problems))
    return f"起跑 {YOUNG_AUDIT_AGE_SECONDS} 秒的两条各记整份上限 12G、各实占 512M：连对 {rounds} 轮，对账照实占算、不判漂移、不推"


def machine_memory_scheduler():
    """machine-memory-line-holds-dispatch 的假调度器：只备 machine_memory_problem 用到的几样。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.peaks = PEAKS
    fake.host_dispatch = loaded_host_dispatch()
    return fake


def machine_memory_snapshot(total_gibibytes, available_gibibytes, swap_total_gibibytes=None, swap_free_gibibytes=None):
    """本机一份报告：整机内存总量与可用量（GiB），swap 总量与空闲（给了才写这两项）。"""
    memory = {"mem_total": int(total_gibibytes * GIBIBYTE), "mem_available": int(available_gibibytes * GIBIBYTE)}
    if swap_total_gibibytes is not None:
        memory[PEAKS.SWAP_TOTAL_KEY], memory[PEAKS.SWAP_FREE_KEY] = int(swap_total_gibibytes * GIBIBYTE), int(swap_free_gibibytes * GIBIBYTE)
    return types.SimpleNamespace(reports={SERVER.LOCAL: {"memory": memory}})


def cell_machine_memory_line_holds_dispatch(world):
    """整机可用内存接近 earlyoom 的线时不派新任务（task_server.py 的 machine_memory_problem；文件头 ②「内存」的整机内存线）：整机 100G、swap 10G，线是 12%。
    可用 9G（9%）、swap 空闲 0.5G（5%）：不派、原因以「整机内存线」起头，写 9.0% 与 5.0% 两个比例；可用 9%、swap 空闲 6G（60%）：仍派；可用 20%、swap 空闲 5%：仍派；
    没有 swap（总量 0）、可用 9%：不派（swap 空闲按 0% 算）；报告里没有 swap 两项：不判。弄坏开关 machine-memory-line-ignored 下不判，这一格红。"""
    fake = machine_memory_scheduler()
    problems = []
    for what, snapshot, held in (("可用 9%、swap 空闲 5%", machine_memory_snapshot(100, 9, 10, 0.5), True), ("可用 9%、swap 空闲 60%", machine_memory_snapshot(100, 9, 10, 6), False),
                                 ("可用 20%、swap 空闲 5%", machine_memory_snapshot(100, 20, 10, 0.5), False), ("可用 9%、没有 swap", machine_memory_snapshot(100, 9, 0, 0), True),
                                 ("报告里没有 swap 两项", machine_memory_snapshot(100, 9), False)):
        text = SERVER.Scheduler.machine_memory_problem(fake, SERVER.LOCAL, snapshot)
        if bool(text) != held or (held and not text.startswith(SERVER.MACHINE_MEMORY_LINE_TEXT)):
            problems.append(f"{what}：应当{'不派、原因以「' + SERVER.MACHINE_MEMORY_LINE_TEXT + '」起头' if held else '照派'}，实际「{text}」")
        if what == "可用 9%、swap 空闲 5%" and ("9.0%" not in text or "5.0%" not in text):
            problems.append(f"{what}：原因应当写出两个比例 9.0% 与 5.0%，实际「{text}」")
    if problems:
        raise SelftestFailure("整机内存线：" + "；".join(problems))
    return "可用 9% 且 swap 空闲 5%：不派、原因写两个比例；swap 空闲 60%、可用 20%、报告里没有 swap：照派；没有 swap 的按空闲 0% 算"


def cell_workspace_writer_cap_has_headroom(world):
    """换文件、打补丁的工具内存上限留余量（task_server.py 的 workspace_writer_floor、memory_floor_applied、workspace_swap_tool；撞上限被杀会把换到一半的结局文件留在主工作区，
    之后所有会话的换上都被挡）：记录的峰值 600 MiB 的预测，换钩子（scripts/gate/hook-swap.py）与打补丁（scripts/gate/apply-writer-patch.py）的上限与记账不低于
    WORKSPACE_WRITER_FLOOR_BYTES（4 GiB），预测的峰值照原样；同样的预测，别的脚本、hook-swap.py 带 --feed-only（只喂不换）、apply-writer-patch.py 带 --dry-run（只读）、
    合入后验证（scripts/gate/merge-and-verify.sh，它的内存照这把键的记录定，cap-hit-rerun-from-record 钉着）只取内存下限 MEMORY_FLOOR_BYTES（1 GiB）。套在假调度器上，不起服务。
    弄坏开关 workspace-writer-cap-unpadded 下换钩子的也只到 1 GiB，recorded-cap-floored 下下限抬到 4G、只取下限的那几样兜到 4096 MiB，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.peer_root = {name: value for name, value in os.environ.items() if name != lib.MEMORY_FLOOR_VARIABLE}, lib.REPOSITORY_ROOT
    prediction = loaded_host_dispatch().Prediction("selftest-writer-key", 600 * MEBIBYTE, 600 * MEBIBYTE, 600 * MEBIBYTE, lib.RECORD_HISTORY)
    problems = []
    for what, command, wanted in (("换钩子", ["python3", "scripts/gate/hook-swap.py", "--spec", "x.json"], SERVER.WORKSPACE_WRITER_FLOOR_BYTES),
                                  ("打补丁", ["python3", "scripts/gate/apply-writer-patch.py", "p.patch"], SERVER.WORKSPACE_WRITER_FLOOR_BYTES),
                                  ("别的脚本", ["python3", "scripts/gate/other-tool.py"], lib.MEMORY_FLOOR_BYTES),
                                  ("打补丁只读", ["python3", "scripts/gate/apply-writer-patch.py", "--dry-run", "p.patch"], lib.MEMORY_FLOOR_BYTES),
                                  ("合入后验证", ["bash", "scripts/gate/merge-and-verify.sh", "p.patch"], lib.MEMORY_FLOOR_BYTES),
                                  ("换钩子只喂不换", ["python3", "scripts/gate/hook-swap.py", "--feed-only", "--spec", "x.json"], lib.MEMORY_FLOOR_BYTES)):
        applied = SERVER.Scheduler.memory_floor_applied(fake, prediction, {"command": command, "cwd": lib.REPOSITORY_ROOT})
        if applied.memory_cap != wanted or applied.memory_booked != wanted or applied.memory_recorded_peak != 600 * MEBIBYTE:
            problems.append(f"{what}：600 MiB 的预测应当取上限与记账 {wanted // MEBIBYTE} MiB、预测的峰值仍 600 MiB，实际上限 {applied.memory_cap // MEBIBYTE} MiB、"
                            f"记账 {applied.memory_booked // MEBIBYTE} MiB、预测 {applied.memory_recorded_peak // MEBIBYTE} MiB")
    if problems:
        raise SelftestFailure("换文件、打补丁的工具内存上限留余量：" + "；".join(problems))
    return "换钩子与打补丁的上限与记账取 4 GiB（预测峰值不变），别的脚本、只喂不换的、只读的打补丁、合入后验证只取 1 GiB 的下限"


def ledger_fake():
    """booking-ledger-rows 的假调度器与 Snapshot：本机 slice 总额 50G、余量 1G，在跑五条——L1（上限 20G，scope 用 5G）、L2（上限 12G，报告里没有它的 scope）、
    L3（上限 5G，单元已确认停了）、L4（上限 2G，scope 用满 2G）、L5（上限 10G，scope 用 4G）。"""
    attempts = [{"id": number, "task_id": 100 + number, "number": 1, "host": SERVER.LOCAL, "label": f"L{number}", "memory_cap": cap * GIBIBYTE}
                for number, cap in ((1, 20), (2, 12), (3, 5), (4, 2), (5, 10))]
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.peaks = PEAKS
    fake.machines = {SERVER.LOCAL: types.SimpleNamespace(label="本机")}
    fake.store = types.SimpleNamespace(running_attempts=lambda: [dict(attempt) for attempt in attempts])
    fake.host_margin = lambda host, snapshot: (GIBIBYTE, "")
    fake.unit_stopped = {3}
    fake.booking_drift_lock = threading.Lock()
    fake.booking_ledger = {}
    report = memory_report(30, 50, scopes={"L1": 5 * GIBIBYTE, "L4": 2 * GIBIBYTE, "L5": 4 * GIBIBYTE})
    return fake, types.SimpleNamespace(reports={SERVER.LOCAL: report}, total=lambda host: 50 * GIBIBYTE)


def cell_booking_ledger_rows(world):
    """在跑记账的逐条明细（task_server.py 的 ledger_row、running_booking_text、refresh_booking_ledger、booking_ledger_snapshot；task-server/control bookings 打出来）：
    假世界里五条各走一支——L1 此刻实测 5G，记 5G（measured）；L2 报告里没有它的 scope，记整份上限 12G（no-scope）；L3 单元已停，不计（0，stopped）；
    L4 实测 2G 到了它的上限 2G，记上限 2G（cap、at_cap）；L5 此刻实测 4G，记 4G（measured）。每条的原因写清取的是哪一个与几个数。
    合计记 23G，照公式（50G − 1G − 23G）放得下 26G，在跑上限之和 49G，各 scope 实占合计 11G。
    control bookings 照 /health 的这一栏逐台打：头一行写 slice 总额、余量与上限之和，每条一行写上限、实占、记多少、取的是哪一个与原因，末行写合计与放得下多少。
    弄坏开关 bookings-ledger-unreported 下这一栏是空表，running-booked-at-cap 下 L1 记 20G，running-booked-with-margin 下 L1 记 6G，这一格红。"""
    import importlib.machinery
    import importlib.util
    now = time.time()
    fake, snapshot = ledger_fake()
    SERVER.Scheduler.refresh_booking_ledger(fake, snapshot, now)
    entry = SERVER.Scheduler.booking_ledger_snapshot(fake).get(SERVER.LOCAL) or {}
    rows = {row["label"]: row for row in entry.get("rows") or []}
    problems = []
    wanted_rows = (("L1", SERVER.BOOKED_FROM_USAGE, 5, False, ("此刻实测的用量 5.0G",)),
                   ("L2", SERVER.BOOKED_WITHOUT_SCOPE, 12, True, ("没有它的 scope", "12.0G")), ("L3", "stopped", 0, False, ("单元已确认停了",)),
                   ("L4", SERVER.BOOKED_FROM_CAP, 2, True, ("到了它的上限", "记上限 2.0G")),
                   ("L5", SERVER.BOOKED_FROM_USAGE, 4, False, ("此刻实测的用量 4.0G",)))
    for label, basis, booked, at_cap, words in wanted_rows:
        row = rows.get(label) or {}
        if row.get("basis") != basis or row.get("booked") != booked * GIBIBYTE or row.get("at_cap") != at_cap or not all(word in (row.get("text") or "") for word in words):
            problems.append(f"{label}：应当 {basis}、记 {booked}G、at_cap {at_cap}、原因写 {'、'.join(words)}，实际 {row}")
    if entry.get("booked") != 23 * GIBIBYTE or entry.get("room") != 26 * GIBIBYTE or entry.get("caps") != 49 * GIBIBYTE or entry.get("used") != 11 * GIBIBYTE:
        problems.append(f"合计应当记 23G、照公式放得下 26G、上限之和 49G、实占合计 11G，实际 记 {entry.get('booked')}、放得下 {entry.get('room')}、上限之和 {entry.get('caps')}、实占 {entry.get('used')}")
    if "放得下 26.0G" not in (entry.get("total") or "") or "×" in (entry.get("total") or ""):
        problems.append(f"合计那一行应当写照公式放得下 26.0G、不写乘几，实际「{entry.get('total')}」")
    loader = importlib.machinery.SourceFileLoader("control_for_bookings", lib.CONTROL_SCRIPT)
    control = importlib.util.module_from_spec(importlib.util.spec_from_loader("control_for_bookings", loader))
    loader.exec_module(control)
    health = {"booking_ledger": SERVER.Scheduler.booking_ledger_snapshot(fake)}
    printed = io.StringIO()
    with unittest.mock.patch.object(lib, "require_server", return_value=health), unittest.mock.patch.object(lib, "read_configuration", return_value=None), contextlib.redirect_stdout(printed):
        code = control.bookings()
    lines = [line for line in printed.getvalue().splitlines() if line.strip()]
    if code != 0 or len(lines) != 7 or "slice 总额 50.0G" not in lines[0] or "放得下 26.0G" not in lines[-1] or sum(1 for line in lines if "第 1 次：上限" in line) != 5:
        problems.append(f"control bookings 应当退 0、打 7 行（头一行、五条、末行合计）、头一行写 slice 总额 50.0G、末行写放得下 26.0G，实际退 {code}、{len(lines)} 行：{lines}")
    for label, basis, *_ in wanted_rows:
        if not any(f"任务 {rows.get(label, {}).get('task')} " in line and f"（{basis}）" in line for line in lines):
            problems.append(f"control bookings 应当有一行写 {label} 那条任务与（{basis}），实际 {lines}")
    if problems:
        raise SelftestFailure("在跑记账的逐条明细：" + "；".join(problems))
    return ("五条各走一支：实测 5G 记 5G、没有 scope 记上限 12G、单元已停不计、实测到了上限记上限 2G、实测 4G 记 4G；"
            "合计记 23G、照公式放得下 26G；control bookings 逐条写取的是哪一个与原因、末行写合计")


DISK_FLOOR_RECORD_BYTES = 35 * GIBIBYTE   # disk-floor-peer-not-never-fits：同键记录的峰值，比本机可用上限（30G 减余量 2G）大、比第二台的（64G 减 2G）小


def cell_disk_floor_peer_not_never_fits(world):
    """第二台盘余量不到下限时记录比本机大的任务排着、不判 never-fits（task_server.py 的 judge_placement、hosts_short_of_record）：同键上一次跑完峰值 35G，
    本机可用上限 28G、第二台 62G 而第二台盘只剩不到下限：任务排着（queued、没有尝试），排队原因写第二台盘只剩；第二台盘够了照派第二台跑完。
    弄坏开关 disk-floor-host-judged-never 下盘不够的第二台不算「回来可能放得下」，本机判永远放不下、任务记 never-fits，这一格红。"""
    key = "selftest-disk-floor-record"
    floor = SERVER.DISK_FLOOR_BYTES
    world.resources(local=(30, 30), peer=(100, 100), peer_cap_gibibytes=64, peer_df=f"     Avail\n{floor - GIBIBYTE}\n")
    world.start_server()
    world.seed_peak(key, DISK_FLOOR_RECORD_BYTES)
    task_id = world.submit(["bash", "-c", "exit 0", "disk-floor-record"], cwd=world.repo, extra={lib.KEY_VARIABLE: key})
    world.next_pass()
    world.next_pass()
    held = world.summary(task_id)
    problems = []
    if held["task"].get("state") != "queued" or held["attempts"] or "第二台盘只剩" not in (held.get("waiting_for") or ""):
        problems.append(f"第二台盘不够、本机放不下：应当排着、没有尝试、排队原因写「第二台盘只剩」，实际 {describe(held)}、排队原因「{held.get('waiting_for')}」、"
                        f"说明「{held['task'].get('note')}」")
    else:
        world.resources(local=(30, 30), peer=(100, 100), peer_cap_gibibytes=64, peer_disk_bytes=floor + GIBIBYTE)
        done = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
        if done["task"].get("outcome") != "succeeded" or (done["attempts"] or [{}])[-1].get("host") != "peer":
            problems.append(f"第二台盘够了应当照派第二台跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("第二台盘不够时不判 never-fits：" + "；".join(problems), world.server_log_lines())
    return "第二台盘不够、本机放不下 35G 的记录：排着；第二台盘够了派过去跑完"


CREDIT_SLOT_GIBIBYTES = 30   # build-disk-credits-free-slot：组里那一个记下的占盘与这条命令量过的编译目录峰值，都是 30G
CREDIT_SPARE_GIBIBYTES = 10  # 那一格本机盘余量比编译目录占盘的下限多这么多：从零编（30G 加下限）放不下，接着用（扣掉 30G 只剩下限）放得下


def cell_build_disk_credits_free_slot(world):
    """接着用组里那一个时，编译目录占盘那一判扣掉它已占的盘（task_server.py 的 free_build_slot_bytes）：这一格的树在本机的组里有一个空着的、记着占盘 30G，
    这条命令量过的编译目录峰值也是 30G，本机盘余量只比下限多 10G：扣掉之后放得下，两轮之内起了、拿的就是组里那一个、跑完。
    弄坏开关 build-disk-reuse-uncredited 下照从零编算（30G 加下限放不下），一直排着、排队原因点名占盘，这一格红。"""
    floor = SERVER.BUILD_DISK_FLOOR_BYTES
    world.resources(local=(30, 30), local_disk_bytes=floor + CREDIT_SPARE_GIBIBYTES * GIBIBYTE)
    world.start_server()
    tree = os.path.realpath(build_slot_tree(world, "tree-disk-credit"))
    slot = os.path.join(world.builds, "selftest-seeded-slot")
    os.makedirs(os.path.join(slot, "target"), exist_ok=True)
    now = round(time.time(), 3)
    world.insert_signed_row("build_slots", {"task_id": 0, "state": "present", "session": "", "command_key": tree, "tree": tree, "host": SERVER.LOCAL, "directory": slot,
                                            "opened_by": "selftest-seeded-slot", "opened_at": now, "used_at": now, "ready": True, "bytes": CREDIT_SLOT_GIBIBYTES * GIBIBYTE})
    key = "selftest-disk-credit"
    world.seed_attempt({"command_key": key, "outcome": "succeeded", "memory_peak": GIBIBYTE, "build_peak": CREDIT_SLOT_GIBIBYTES * GIBIBYTE})
    task_id = world.submit(["cargo", "build"], extra=dict(fake_cargo_extra(world), **{lib.KEY_VARIABLE: key}), cwd=tree)
    world.next_pass()
    world.next_pass()
    held = world.summary(task_id)
    if held["task"].get("state") == "queued" and "编译目录占盘" in (held.get("waiting_for") or ""):
        raise SelftestFailure(f"组里空着的那一个已占 {CREDIT_SLOT_GIBIBYTES}G、这条命令量过 {CREDIT_SLOT_GIBIBYTES}G、盘余量比下限多 {CREDIT_SPARE_GIBIBYTES}G：扣掉之后放得下，"
                              f"不该因占盘排着，实际排着、在等「{held.get('waiting_for')}」", world.server_log_lines())
    finished = world.finished(task_id)
    build = first_attempt(finished).get("build_directory") or ""
    if finished["task"].get("outcome") != "succeeded" or build != slot:
        raise SelftestFailure(f"应当拿组里那一个（{slot}）跑完，实际 {describe(finished)}、编译目录「{build}」", world.server_log_lines())
    return f"接着用组里已占 {CREDIT_SLOT_GIBIBYTES}G 的那一个：盘余量只比下限多 {CREDIT_SPARE_GIBIBYTES}G 也起了、跑完"
