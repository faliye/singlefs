#!/usr/bin/env python3
"""K5 钉值复核（攻方 m3-batch52-54-code-r1 opus）：从原理重算第 52 批 dispatch 与 unlink 全量闭式的钉值；对第 52、54 批实现员留的证据文件
重算分片用例两个钉值（打的行、进度文件）并核「去掉新不变量那几项之后逐字相同」。

用法：k5.py <仓根> <第 52 批证据目录> <第 54 批证据根>
  第 52 批证据目录：放 previous-golden-progress-1d92a7ad.txt 与 layer0-progress-…-555baf1b….txt
  第 54 批证据根：放 printed-b53.txt、printed-b53-without-new-two.txt 与 evidence/progress-b50-1553ba8e.txt、evidence/progress-b53-92ff4595.txt
"""
import hashlib
import re
import sys
from pathlib import Path


def crc32c(data: bytes) -> int:
    crc = 0xFFFFFFFF
    for byte in data:
        crc ^= byte
        for _ in range(8):
            crc = (crc >> 1) ^ (0x82F63B78 if crc & 1 else 0)
    return crc ^ 0xFFFFFFFF


def stripped(line: str, dropped: list[str]) -> str:
    line = re.sub(r" checksum=[0-9a-f]{8}$", "", line)
    line = re.sub(r"plan=[0-9a-f]{64}", "plan=X", line)
    for invariant in dropped:
        line = re.sub(r"(?<=[=,])" + re.escape(invariant) + r":\d+,?", "", line)
    return re.sub(r",(?= |$)", "", line)


def progress_pair(old: Path, new: Path, dropped: list[str]) -> str:
    a, b = old.read_text().splitlines(), new.read_text().splitlines()
    same = sum(1 for x, y in zip(a, b) if stripped(x, []) == stripped(y, dropped))
    crc_ok = sum(1 for line in b if (m := re.match(r"^(.*) checksum=([0-9a-f]{8})$", line)) and crc32c(m.group(1).encode()) == int(m.group(2), 16))
    return (f"old={old.name} sha256={hashlib.sha256(old.read_bytes()).hexdigest()[:16]} new={new.name} "
            f"sha256={hashlib.sha256(new.read_bytes()).hexdigest()[:16]} lines={len(a)}/{len(b)} identical_after_dropping={same} new_crc32c_ok={crc_ok}")


def main() -> int:
    repo, b52, b54 = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
    gpu = (repo / "crates/singlefs-checker-tier/src/crash_judge_gpu.rs").read_text()
    scratch_words = int(re.search(r"pub const SCRATCH_WORDS: usize = ([0-9_]+);", gpu).group(1).replace("_", ""))
    per_state = scratch_words * 4
    print(f"K5_DISPATCH scratch_words={scratch_words} per_state_bytes={per_state} "
          f"states_per_lane={2_147_483_644 // per_state} states_in_12_gib={12 * 1024 ** 3 // per_state} "
          f"mebibytes_for_45={1 + -(-45 * (per_state + 2 * 14 * 4) // 1_048_576)}")
    two_states = 1_448_083_588 - ((1 << 20) - 1) + ((1 << 24) - 1)
    domain = 1_448_083_638 - ((1 << 20) - 1) + ((1 << 24) - 1)
    print(f"K5_UNLINK_CLOSED_FORM two_states_per_write={two_states} domain={domain}")
    print("K5_PROGRESS_B52 " + progress_pair(b52 / "previous-golden-progress-1d92a7ad.txt",
                                            next(b52.glob("layer0-progress-*555baf1b*.txt")),
                                            ["I-9.30", "I-9.31", "I-9.32", "I-9.33", "I-9.34", "I-9.35"]))
    print("K5_PROGRESS_B54 " + progress_pair(b54 / "evidence/progress-b50-1553ba8e.txt", b54 / "evidence/progress-b53-92ff4595.txt", ["I-9.36", "I-9.37"]))
    printed = (b54 / "printed-b53.txt").read_text()
    without = (b54 / "printed-b53-without-new-two.txt").read_text()
    masked = "".join(line + "\n" for line in printed.splitlines())
    masked_without = "".join(line + "\n" for line in without.splitlines())
    print(f"K5_PRINTED_B54 lines={len(printed.splitlines())} sha256={hashlib.sha256(masked.encode()).hexdigest()} "
          f"without_two_sha256={hashlib.sha256(masked_without.encode()).hexdigest()} "
          f"removing_the_two_items_gives_the_without_file={printed.replace(' I-9.36=0/0/46 I-9.37=0/0/46', '') == without}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
