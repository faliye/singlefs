#!/usr/bin/env python3
"""给冻结副本拷贝里的运行库用例共用搭建加探针要摆的三样故障：写报错之前先放过几个写、屏障报错之前先放过几个屏障、接下来几个屏障报错。
只改交进来的那一份文件；逐条旧串都要恰好一处，不是就退出、不写。用法：patch_the_fixture.py <共用搭建>。"""
import sys

path = sys.argv[1]
with open(path, encoding="utf-8") as source:
    text = source.read()
for old, new in [
    (
        "    pub writes_left_to_fail: Arc<AtomicU64>,\n",
        "    pub writes_left_to_fail: Arc<AtomicU64>,\n"
        "    /// 探针摆的：写报错之前先放过几个写。\n"
        "    pub writes_let_through_before_the_faults: Arc<AtomicU64>,\n"
        "    /// 探针摆的：屏障报错之前先放过几个屏障。\n"
        "    pub barriers_let_through_before_the_faults: Arc<AtomicU64>,\n"
        "    /// 探针摆的：接下来还有几个屏障报错。\n"
        "    pub barriers_left_to_fail: Arc<AtomicU64>,\n",
    ),
    (
        "    ) -> Result<(), BlockDeviceError> {\n        let fails = self\n",
        "    ) -> Result<(), BlockDeviceError> {\n"
        "        let is_let_through = self\n"
        "            .faults\n"
        "            .writes_let_through_before_the_faults\n"
        "            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |left| {\n"
        "                left.checked_sub(1)\n"
        "            })\n"
        "            .is_ok();\n"
        "        if is_let_through {\n"
        "            return Ok(());\n"
        "        }\n"
        "        let fails = self\n",
    ),
    (
        "    fn barrier(&mut self) -> Result<(), BlockDeviceError> {\n"
        "        self.note(RecordedDeviceOperation::Barrier {\n",
        "    fn barrier(&mut self) -> Result<(), BlockDeviceError> {\n"
        "        let is_let_through = self\n"
        "            .faults\n"
        "            .barriers_let_through_before_the_faults\n"
        "            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |left| {\n"
        "                left.checked_sub(1)\n"
        "            })\n"
        "            .is_ok();\n"
        "        if !is_let_through\n"
        "            && self\n"
        "                .faults\n"
        "                .barriers_left_to_fail\n"
        "                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |left| {\n"
        "                    left.checked_sub(1)\n"
        "                })\n"
        "                .is_ok()\n"
        "        {\n"
        "            return Err(BlockDeviceError::OutOfRange {\n"
        "                offset: DeviceOffsetInBytes(0),\n"
        "                length: 0,\n"
        "                device_size: 0,\n"
        "            });\n"
        "        }\n"
        "        self.note(RecordedDeviceOperation::Barrier {\n",
    ),
]:
    if text.count(old) != 1:
        sys.exit(f"{path}: {old!r} 不是恰好一处")
    text = text.replace(old, new)
with open(path, "w", encoding="utf-8") as target:
    target.write(text)
