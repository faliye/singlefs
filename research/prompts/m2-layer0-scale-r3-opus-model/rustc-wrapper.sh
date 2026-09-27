#!/bin/sh
# RUSTC_WRAPPER：第一个参数是真 rustc，其余是 cargo 给的参数；只在编译 crate 时加一个 cfg（-vV 之类的查询原样放过）。
rustc="$1"; shift
case " $* " in *" --crate-name "*) exec "$rustc" "$@" --cfg opus_probe ;; *) exec "$rustc" "$@" ;; esac
