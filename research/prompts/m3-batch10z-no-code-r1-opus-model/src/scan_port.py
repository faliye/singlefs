#!/usr/bin/env python3
"""m3-batch10z-no-code-r1 攻方 K4：照搬 gpu_unit_checks.rs 测试 `the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking` 的取法与比法
（按测试模块开头切、找签名、取到第一处缩进十二格的右括号、去掉 // 开头的行与空白、整句比），对一份文件判绿红。用法：scan_port.py <gpu_unit_checks.rs>"""
import sys
src = open(sys.argv[1]).read()
code = src.split("#[cfg(test)]\nmod tests {")[0]
sig = "fn take_in_the_devices_found_idle(&self) {"
assert code.count(sig) == 1, "签名不止一处"
after = code[code.index(sig) + len(sig):]
body = after[:after.index("\n            }\n")]
got = ''.join(ch for line in body.split('\n') if not line.lstrip().startswith('//') for ch in line if not ch.isspace())
expected = ("self.ledger.take_in_the_devices_kept_before_booking("
 "|card_position|self.cards[card_position].started.started_now(),"
 "|started:&GpuCard|started.device_condition().is_lost(),"
 "|started:&GpuCard|{device_found_idle_by_a_poll(&std::panic::catch_unwind("
 "std::panic::AssertUnwindSafe(||{started.device().poll(wgpu::PollType::Poll)}),))},"
 "|started:&GpuCard|{started.device_on_the_books.as_deref().expect("
 "\"卡池起的设备起好时放进了账上的那一台（started_card）\")},"
 ");")
print("SCAN", "绿" if got == expected else "红", sys.argv[1])
