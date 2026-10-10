#!/usr/bin/env python3
"""#9 钉值重取（K5）：把新的进度文件里每个片行的 I-3.14 计数项删掉、重算每行 CRC-32C，整份 SHA-256 与旧钉值比。
用法：golden-progress-file-strip-check.py <进度文件>    旧钉值 11977571…（2026-10-06 删除线第 24 批那一版）。
进度文件取自 checker 档 `crash_enumeration_sharded_across_processes` 的 golden 用例留下的那一份（重型测试，由主 agent 跑；本轮正推腿没跑、没有那份文件）。
片行格式：layer0_progress.rs `slice_line`：`key=value` 以空格隔开，末尾 ` checksum=<正文 CRC-32C 八位小写十六进制>`；
checker_evaluated_states / checker_violated_states / checker_not_applicable_states 三个字段是 `name:count,name:count` 或 `empty`。
先核每一行自己的校验和（文件没被改过），再删项重算。"""
import hashlib, re, sys

OLD_PIN = "119775719aa569efddcf00022a0ac5f1d70a2102af181e379a70aa6e4afd6cb5"
MAPS = ("checker_evaluated_states", "checker_violated_states", "checker_not_applicable_states")

def crc32c_table():
    table = []
    for i in range(256):
        c = i
        for _ in range(8):
            c = (c >> 1) ^ 0x82F63B78 if c & 1 else c >> 1
        table.append(c)
    return table
TABLE = crc32c_table()
def crc32c(data: bytes) -> int:
    c = 0xFFFFFFFF
    for b in data:
        c = TABLE[(c ^ b) & 0xFF] ^ (c >> 8)
    return c ^ 0xFFFFFFFF

def check_own_checksum(line: str) -> str:
    body, _, tail = line.rpartition(" checksum=")
    assert f"{crc32c(body.encode()):08x}" == tail, f"这一行自己的校验和对不上：{line[:80]}"
    return body

def strip_i_3_14(body: str) -> tuple[str, int]:
    removed = 0
    fields = body.split(" ")
    for position, field in enumerate(fields):
        key, _, value = field.partition("=")
        if key in MAPS and value != "empty":
            kept = [entry for entry in value.split(",") if not entry.startswith("I-3.14:")]
            removed += len(value.split(",")) - len(kept)
            fields[position] = f"{key}={','.join(kept) if kept else 'empty'}"
    return " ".join(fields), removed

def main(path: str) -> int:
    text = open(path, encoding="utf-8").read()
    out, removed_total, slice_lines = [], 0, 0
    for line in text.split("\n"):
        if line == "":
            continue
        body = check_own_checksum(line)
        if body.startswith("slice="):
            body, removed = strip_i_3_14(body)
            removed_total += removed
            slice_lines += 1
        out.append(f"{body} checksum={crc32c(body.encode()):08x}")
    stripped = "\n".join(out) + ("\n" if text.endswith("\n") else "")
    digest = hashlib.sha256(stripped.encode()).hexdigest()
    print(f"片行 {slice_lines} 行，删掉 I-3.14 项 {removed_total} 个")
    print(f"删后 SHA-256 {digest}")
    print(f"旧钉值       {OLD_PIN}")
    print("相同：新进度文件与旧版只差 I-3.14 项" if digest == OLD_PIN else "不同：除 I-3.14 之外还有别的变化被钉进去了")
    return 0 if digest == OLD_PIN else 1

if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
