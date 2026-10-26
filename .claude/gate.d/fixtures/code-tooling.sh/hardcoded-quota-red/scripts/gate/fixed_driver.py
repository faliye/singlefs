#!/usr/bin/env python3
"""样本：python 里的几种写死。"""
import argparse

DEVICE_MEMORY_CAP = "6G"
host_count = 2


def command(binary):
    return ["bash", "scripts/hosts/run-with-memory-cap.sh", "8G", binary]


def options():
    parser = argparse.ArgumentParser()
    parser.add_argument("--memory-cap", default="4G")
    return parser, {"hosts": 3}
