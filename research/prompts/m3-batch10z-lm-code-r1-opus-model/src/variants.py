import sys, os
src_dir, out_dir = sys.argv[1], sys.argv[2]
os.makedirs(out_dir, exist_ok=True)
variants = {
  # 名字: (源文件, 旧串, 新串)
  "crash-0-original": ("crash_amplification.rs", None, None),
  "crash-1-row-never-stop": ("crash_amplification.rs", "        checking_threads(),\n        stop_is_requested,", "        checking_threads(),\n        &|| false,"),
  "crash-2-fool-never-stop-naming-it": ("crash_amplification.rs", "        checking_threads(),\n        stop_is_requested,", "        checking_threads(),\n        &|| { let _ = stop_is_requested; false },"),
  "crash-3-fool-never-ask-naming-it": ("crash_amplification.rs", "        if stop_is_requested() {\n            break;\n        }", "        if false && stop_is_requested() {\n            break;\n        }"),
  "gpu-0-original": ("gpu_unit_checks.rs", None, None),
  "gpu-1-row-never-lost": ("gpu_unit_checks.rs", "                    |started: &GpuCard| started.device_condition().is_lost(),\n                    |started: &GpuCard| {\n                        device_found_idle_by_a_poll(", "                    |_started: &GpuCard| false,\n                    |started: &GpuCard| {\n                        device_found_idle_by_a_poll("),
  "gpu-2-fool-never-call-the-ledger": ("gpu_unit_checks.rs", "            fn take_in_the_devices_found_idle(&self) {\n                self.ledger.take_in_the_devices_kept_before_booking(", "            fn take_in_the_devices_found_idle(&self) {\n                if false { self.ledger.take_in_the_devices_kept_before_booking("),
  "gpu-3-fool-poll-always-idle": ("gpu_unit_checks.rs", "            fn take_in_the_devices_found_idle(&self) {\n", "            fn take_in_the_devices_found_idle(&self) {\n                fn device_found_idle_by_a_poll<T>(_polled: &T) -> bool { true }\n"),
}
for name, (f, old, new) in variants.items():
    s = open(os.path.join(src_dir, f)).read()
    if old is not None:
        n = s.count(old)
        if n != 1:
            sys.exit(f"{name}: 命中 {n} 次")
        s = s.replace(old, new)
        if name == "gpu-2-fool-never-call-the-ledger":
            tail_old = "                );\n            }\n\n            /// 借到 `booking`"
            m = s.count(tail_old)
            if m != 1:
                sys.exit(f"{name} 收尾: 命中 {m} 次")
            s = s.replace(tail_old, "                ); }\n            }\n\n            /// 借到 `booking`")
    open(os.path.join(out_dir, name + ".rs"), "w").write(s)
    print(name)
