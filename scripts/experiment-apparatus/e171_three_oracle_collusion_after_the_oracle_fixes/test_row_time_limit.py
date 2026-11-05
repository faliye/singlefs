"""row_driver.py 的行限时与装置命令：装置命令里没有内存包装与 --memory-cap；超过限时由驱动杀掉整组、退出码记 253（登记第一节读法 5 的超时）；
限时之内照原样交回装置自己的退出码。弄坏开关 E171_ROW_DRIVER_BREAK 见 row_driver.py 文件头。

跑法：在这个目录里 `python3 -B -m unittest test_row_time_limit`。
"""
import os
import subprocess
import tempfile
import time
import unittest

import row_driver


class DeviceCommandTests(unittest.TestCase):
    def test_the_device_command_starts_the_timing_wrapper_itself_with_no_wrapper_in_front(self) -> None:
        command = row_driver.device_command("/bin/device", "/out/dump", ["--streams", "1,2"])
        self.assertEqual(command[1:5], ["-B", row_driver.DEVICE_TIMING_SCRIPT, "run", "--"])
        self.assertEqual(command[5:], ["/bin/device", "row", "--output-directory", "/out/dump", "--streams", "1,2"])
        self.assertFalse(any(word.endswith(".sh") or "memory-cap" in word for word in command), command)


class RowTimeLimitTests(unittest.TestCase):
    def test_a_row_over_the_time_limit_is_killed_as_a_group_and_exits_253(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            marker = os.path.join(root, "grandchild-alive")
            # 子进程再起一个睡着的孙进程，超时要把整组杀掉：孙进程不留下标记文件
            command = ["bash", "-c", f"(sleep 3; : > {marker}) & sleep 30"]
            started = time.monotonic()
            exit_code = row_driver.run_with_time_limit(command, 1, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            self.assertEqual(exit_code, row_driver.ROW_TIMEOUT_EXIT_CODE)
            self.assertLess(time.monotonic() - started, 10)
            time.sleep(3.5)
            self.assertFalse(os.path.exists(marker), "孙进程没被一起杀掉")

    def test_a_row_inside_the_time_limit_returns_its_own_exit_code(self) -> None:
        self.assertEqual(row_driver.run_with_time_limit(["bash", "-c", "exit 7"], 30, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL), 7)


if __name__ == "__main__":
    unittest.main()
