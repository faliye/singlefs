#!/usr/bin/env python3
# 正推腿自造变异（第 10z 批第三块 K3）：在草稿目录的树副本上，每条改一处（改前串在测试段之前恰好命中一次）、
# 跑 `cargo test -p singlefs-harness --lib gpu_memory_ledger`（harness 档）、还原。红 = 有用例拦住；绿 = 照绿（存活）。
import os, subprocess, sys, shutil, time
TREE = os.path.expanduser("~/.local/share/singlefs/work/m3-batch10z-c-code-r1-sonnet/tree")
TARGET = os.path.expanduser("~/.local/share/singlefs/work/m3-batch10z-c-code-r1-sonnet/target")
SRC = os.path.join(TREE, "crates/singlefs-harness/src/gpu_memory_ledger.rs")
REPO = "/home/fy5090/code/singlefs"
M = []
def m(name, old, new): M.append((name, old, new))

m("M01 读数不压（take_in_a_reading 的调用去掉）",
  "books_of_the_card.take_in_a_reading(*reading);", "let _ = reading;")
m("M02 销账把那一份换成这一笔（不累加）",
  "self.returned_possibly_still_on_the_card_mebibytes = self\n                    .returned_possibly_still_on_the_card_mebibytes\n                    .saturating_add(mebibytes);\n                true",
  "self.returned_possibly_still_on_the_card_mebibytes = mebibytes;\n                true")
m("M03 销账不记进卡上可能还没还的",
  ".saturating_add(mebibytes);\n                true", ".saturating_add(0);\n                true")
m("M04 设备丢了重起：旧设备那一份不记进卡上可能还没还的",
  ".saturating_add(mebibytes_left_for_the_driver);\n    }", ".saturating_add(0);\n    }")
m("M05 设备丢了重起：驱动那一份仍记成 DeviceStarted",
  "self.driver_share = DriverShareOnTheCard::DeviceLostNotStartedAgain;", "self.driver_share = DriverShareOnTheCard::DeviceStarted;")
m("M06 实占先扣的那一份去掉（不扣销了账的）",
  ".saturating_sub(booked.returned_possibly_still_on_the_card_mebibytes)", "")
m("M07 实占里属于许出去的：不封顶在许出去的",
  "            .min(promised_mebibytes),", "            ,")
m("M08 压法：永不往下压",
  "returned_possibly_still_on_the_card_mebibytes.min(mebibytes)\n        }\n        OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => {",
  "returned_possibly_still_on_the_card_mebibytes\n        }\n        OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => {")
m("M09 压法：没列或另有不走账的设备时压到 0",
  "OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => {\n            returned_possibly_still_on_the_card_mebibytes\n        }\n    }\n}",
  "OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => {\n            0\n        }\n    }\n}")
m("M10 重起判：恰差 1 MiB 放不下也放行",
  "if capacity_mebibytes < booked.leases_mebibytes {", "if capacity_mebibytes + 1 < booked.leases_mebibytes {")
m("M11 prepare：丢了再来（DeviceLostNotStartedAgain）直接放行",
  "            DriverShareOnTheCard::DeviceLostNotStartedAgain => {\n                self.driver_share_of_a_new_device_fits_now(&mut books, card_position)\n            }",
  "            DriverShareOnTheCard::DeviceLostNotStartedAgain => Ok(()),")
m("M12 prepare：设备起着丢了第一次也不判放不放得下",
  "                    self.mebibytes_each_process_keeps_on_a_card,\n                );\n                self.driver_share_of_a_new_device_fits_now(&mut books, card_position)",
  "                    self.mebibytes_each_process_keeps_on_a_card,\n                );\n                Ok(())")
m("M13 借了一笔在 DeviceLostNotStartedAgain 时把驱动那一份记回登记了、没起",
  "            DriverShareOnTheCard::RegisteredDeviceNotStarted\n            | DriverShareOnTheCard::DeviceStarted\n            | DriverShareOnTheCard::DeviceLostNotStartedAgain => {}\n        }\n    }\n}",
  "            DriverShareOnTheCard::RegisteredDeviceNotStarted\n            | DriverShareOnTheCard::DeviceStarted => {}\n            DriverShareOnTheCard::DeviceLostNotStartedAgain => {\n                self.driver_share = DriverShareOnTheCard::RegisteredDeviceNotStarted;\n            }\n        }\n    }\n}")
m("M14 登记了没有：DeviceLostNotStartedAgain 算没登记",
  "            DriverShareOnTheCard::RegisteredDeviceNotStarted\n            | DriverShareOnTheCard::DeviceStarted\n            | DriverShareOnTheCard::DeviceLostNotStartedAgain => true,",
  "            DriverShareOnTheCard::RegisteredDeviceNotStarted\n            | DriverShareOnTheCard::DeviceStarted => true,\n            DriverShareOnTheCard::DeviceLostNotStartedAgain => false,")
m("M15 报到：不等其余几段",
  "while *works_not_yet_ended > 0 {", "while false && *works_not_yet_ended > 0 {")
m("M16 报到：最后一段报完不叫醒等着的",
  "            self.every_work_ended.notify_all();\n        }\n    }\n}\n\n/// 一段活的报到", "        }\n    }\n}\n\n/// 一段活的报到")
m("M17 报到：没报到就丢掉时不补报",
  "        if !self.has_ended {\n            self.together.end_one_work();", "        if self.has_ended {\n            self.together.end_one_work();")
m("M18 卡线程那一圈：不问叫停",
  "while !must_stop() {", "while true || !must_stop() {")
m("M19 块队：领序号最大的",
  "self.blocks_not_taken_locked().pop_first()", "self.blocks_not_taken_locked().pop_last()")
m("M20 块队：交回两次不断言",
  "assert!(\n            is_newly_given_back,\n            \"块 {pending_index} 交回时已经在队里：一块领一次只交回一次\"\n        );", "let _ = is_newly_given_back;")
m("M21 按块序写库：写库失败之后到的照写（continue 去掉）",
  "            if self.storing_error.is_some() {\n                continue;\n            }\n", "")
m("M22 几轮：叫停也再起一轮",
  "StopOfTheRun::Requested => AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort,", "StopOfTheRun::Requested => AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory,")
m("M23 几轮：没有没报过的卡也再起一轮",
  "StopOfTheRun::NotRequested if cards_not_short_of_memory == 0 => {", "StopOfTheRun::NotRequested if cards_not_short_of_memory == 99 => {")
m("M24 几轮：这一轮没卡报 OOM 也不当叫停（落进再起一轮）",
  "    if cards_short_of_memory_this_round == 0 {\n        return AfterARoundOfTheCards::StoppedOnRequest;\n    }\n", "")
m("M25 认 OOM：范围里接到的不记进设备",
  "        if let Some(caught) = caught_in_the_scope {\n            self.record_an_out_of_memory(caught);\n        }\n", "        let _ = caught_in_the_scope;\n")
m("M26 认 OOM：先认后等",
  "        ending.end_and_wait_for_the_others();\n        self.out_of_memory_explaining_a_failure_of_the_work(work.identifier(), reports_before)",
  "        let explained = self.out_of_memory_explaining_a_failure_of_the_work(work.identifier(), reports_before);\n        ending.end_and_wait_for_the_others();\n        explained")
m("M27 重起判读数：用没压过的占着（驱动那一份 take_back 之后不再读）",
  "        let readings = self.readings_taken_into_the_books(books)?;\n        let booked = books[card_position].booked();",
  "        let readings = (self.read_card_memory)().map_err(|reason| BookingRefusal::CardMemoryUnreadable { reason })?;\n        let booked = books[card_position].booked();")
m("M28 借：驱动那一份在 DeviceLostNotStartedAgain 时不算许出去的",
  "        DriverShareOnTheCard::RegisteredDeviceNotStarted\n        | DriverShareOnTheCard::DeviceStarted\n        | DriverShareOnTheCard::DeviceLostNotStartedAgain => booked\n            .leases_mebibytes\n            .saturating_add(mebibytes_left_for_the_driver),",
  "        DriverShareOnTheCard::RegisteredDeviceNotStarted\n        | DriverShareOnTheCard::DeviceStarted => booked\n            .leases_mebibytes\n            .saturating_add(mebibytes_left_for_the_driver),\n        DriverShareOnTheCard::DeviceLostNotStartedAgain => booked.leases_mebibytes,")

def run(idx):
    name, old, new = M[idx]
    text = open(SRC).read()
    head, sep, tail = text.partition("#[cfg(test)]\nmod tests {")
    n = head.count(old)
    if n != 1:
        return f"{name}: 改前串在测试段之前命中 {n} 次，跳过"
    open(SRC, "w").write(head.replace(old, new) + sep + tail)
    try:
        t0 = time.time()
        env = dict(os.environ, CARGO_TARGET_DIR=TARGET)
        import signal
        cmd = ["nice","-n","19","bash",f"{REPO}/scripts/hosts/run-with-memory-cap.sh","4G","bash",f"{REPO}/scripts/hosts/capped.sh","8",
               "cargo","test","--offline","-p","singlefs-harness","--lib"]
        proc = subprocess.Popen(cmd, cwd=TREE, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, start_new_session=True)
        try:
            out, _ = proc.communicate(timeout=int(os.environ.get("MUT_TIMEOUT", "90")))
            rc = proc.returncode
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid, signal.SIGTERM)
            try:
                out, _ = proc.communicate(timeout=20)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid, signal.SIGKILL)
                out, _ = proc.communicate()
            rc = "TIMEOUT"
        reds = [l.split()[1] for l in out.splitlines() if l.startswith("test ") and l.endswith("FAILED")]
        if rc == 0:
            verdict = "照绿（存活）"
        elif rc == "TIMEOUT":
            verdict = "超时（挂住，算被拦住）" + (" 红的：" + ",".join(x.split("::")[-1] for x in reds) if reds else "")
        elif "error[" in out or "could not compile" in out:
            verdict = "编不过"
        else:
            verdict = "红：" + ",".join(x.split("::")[-1] for x in reds)
        return f"{name}: {verdict} ({time.time()-t0:.0f}s)"
    finally:
        open(SRC, "w").write(text)

if __name__ == "__main__":
    a = [int(x) for x in sys.argv[1:]] or list(range(len(M)))
    for i in a:
        print(run(i), flush=True)
