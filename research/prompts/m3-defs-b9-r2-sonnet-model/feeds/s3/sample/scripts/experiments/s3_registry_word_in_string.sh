#!/usr/bin/env bash
default_threads="用 heavy_thread_default 取，这里先写个 8"
[[ -n "${S3_STR_THREADS:-}" ]] || S3_STR_THREADS=$default_threads
