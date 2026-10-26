#!/usr/bin/env bash
# peer_trip4.sh：只看 InsideALookupSite 的 drop 在 after 与 dropless 两份副本优化后的 IR 里到底剩什么（t3_drop.sh 带行打印）。
set -u
COPIES="after dropless" bash model/tools/t3_drop.sh
