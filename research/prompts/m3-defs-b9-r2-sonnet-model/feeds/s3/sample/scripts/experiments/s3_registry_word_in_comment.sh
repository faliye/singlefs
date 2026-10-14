#!/usr/bin/env bash
default_threads=8   # 以后改成 heavy_thread_default
[[ -n "${S3_CMT_THREADS:-}" ]] || S3_CMT_THREADS=$default_threads
