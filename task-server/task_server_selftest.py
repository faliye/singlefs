"""task-server 自证的各格（python3 task-server/task_server_lib.py --selftest 起；这一份单独不起）。做法、只在自证里认的变量与弄坏开关写在 task_server_lib.py 文件头。各格按讲的那一块分在同目录的 task_server_selftest_<部分>.py 里（task_server_lib.py 文件头列了各份），这一份汇总成 CELLS 并起各格。"""
import os
import sys

sys.dont_write_bytecode = True

import concurrent.futures  # noqa: E402
import contextlib  # noqa: E402
import dataclasses  # noqa: E402
import hashlib  # noqa: E402
import io  # noqa: E402
import itertools  # noqa: E402
import json  # noqa: E402
import re  # noqa: E402
import shlex  # noqa: E402
import shutil  # noqa: E402
import signal  # noqa: E402
import socket  # noqa: E402
import sqlite3  # noqa: E402
import subprocess  # noqa: E402
import tempfile  # noqa: E402
import threading  # noqa: E402
import time  # noqa: E402
import traceback  # noqa: E402
import types  # noqa: E402
import unittest  # noqa: E402
import unittest.mock  # noqa: E402

import task_server_lib as lib  # noqa: E402
from task_server_selftest_fakes import (
    AT_CAP_PEAK_BYTES, AT_CAP_PINNED_TEXT, AT_CAP_SEED_PEAK_BYTES, BLOCK_SCRIPT, CELL_TIMEOUT_SECONDS, CLIENT_GROUP_STOP_SECONDS,
    CONFIRM_BUDGET_SECONDS, CONFIRM_RETRIES, DEFAULT_INBOX_TOKEN, FAILURE_PREFIX, FAKE_CARGO, FAKE_GPU_SERVICE_CONTROL, FAKE_MEMORY_WRAPPER,
    FAKE_PEER_RUNNER, FAKE_PRUNE_CHECK, FAKE_SHARDED, FAKE_SSH, FAKE_SYSTEMCTL, FAKE_SYSTEMD_RUN_DELAYED, FAKE_SYSTEMD_RUN_REFUSING,
    FAKE_SYSTEMD_RUN_SELECTIVE, FAKE_SYSTEMD_RUN_SILENT, FAKE_TEMPORARY_SERVICE, GIBIBYTE, HOLD_UNTIL_RELEASED_SCRIPT, LOCAL_CARD_ZERO, MEBIBYTE,
    PEAKS, PLACEMENT_WAIT_SECONDS, POLL_SECONDS, PROGRESS_IGNORED_FILES, PROGRESS_SAMPLE_SECONDS, SEEDED_ENDED_AT, SELFTEST_LOCAL_ADDRESS,
    SELFTEST_MARGIN_GIBIBYTES, SELFTEST_MULTI_HOST_KEYS, SELFTEST_PEER_ADDRESS, SELFTEST_SLICE_CAP_TEXT, SERVER, SESSION_A, SESSION_B,
    SOCKET_ROOT_PREFIX, SOCKET_ROOT_VARIABLE, STATE_TIMEOUT_SECONDS, UNRECORDED_DEFAULT_PINNED_TEXT, WAIT_CEILING_FACTOR, WAIT_SLICE_SECONDS,
    WORLD_SOCKET_PREFIX,
)
from task_server_selftest_world import (
    ATTENTION_QUIET_SECONDS, ATTENTION_WAIT_SECONDS, CPU_GATE_HEAVY_KEY, CPU_GATE_PROBE_KEY, FAKE_COMMIT_GATE, FAKE_JOURNALCTL_EARLYOOM,
    FAKE_MERGE_ENTRY, FILLS_CAP_SEEDED_CAP_BYTES, FILLS_CAP_SEEDED_PEAK_BYTES, FakeInbox, HEALTH_READ_SECONDS, INBOX_QUIET_SECONDS,
    INBOX_TOKEN_SAMPLE, INBOX_WAIT_SECONDS, KEEP_TEMPORARY_EXTRA, LONG_RUN_KIND, LONG_RUN_WHY_SAMPLE, LONG_TICKS_SCRIPT, NO_BORROW_ENTRY_SCRIPT,
    Patience, QUEUED_SHARD_KEY, REPUSH_SAMPLE_SECONDS, RUNNING_BOOKING_LABEL, RUN_LIMIT_SAMPLE_SECONDS, SCENE_COMMAND_EXIT, SCOPE_COMMIT,
    SCOPE_DAILY, SELFTEST_IDLE_CPU, STARVED_HEALTH_TEXT, SelftestFailure, World, append_lines, attempt_has_run_for, attention_fields,
    attention_lines, attention_lines, attention_problems, blocking_command, build_slot_tree, by_host_text, cargo_targets, child_roles,
    cpu_gate_scheduler, describe, fake_cargo_extra, first_attempt, health_until, inbox_problems, loaded_host_dispatch, long_run_split_scheduler,
    memory_report, outcome_head, peer_log_scheduler, process_gone, process_memory_stat, queued_shards, raise_cell_problems, raw_column_update,
    read_health_patiently, read_text, read_text, reminder_head, reminders_after, restart_surviving_scheduler, running_booking_scheduler,
    running_shards, scene_of, seed_ended_attempt, seed_peak_readings, server_modules, shard_children, shard_outcomes, shown_local_pin,
    start_burning_scope, start_inbox_server, start_three_shards, systemctl_start_count, table_advice, verified_row, verifying_store, wait_last_line,
    wait_path, write_executable, write_fake_entry,
)
from task_server_selftest_client import (
    ABSENT_TASK_ID, COMMAND_WORD_SAMPLES, CONTROL_CHECK_EXTRA_SECONDS, CONTROL_CHECK_LIMIT_SECONDS, DISCONNECT_NOTICE_SECONDS,
    DROP_HISTORY_CAP_BYTES, DROP_HISTORY_LIMIT_GIBIBYTES, DROP_HISTORY_LOW_PEAK_BYTES, DROP_HISTORY_NEAR_CAP_PEAK_BYTES,
    DROP_HISTORY_SMALL_PEAK_BYTES, EXERCISED_BY, HEAVY_THREADS, LOG_LINES_PER_ATTEMPT, LOG_SCRIPT, LOG_TAIL_PINNED_LINES, LONGEST_RUNTIME_NAME,
    RANDOM_NAME_CHARACTERS, RUNNING_ATTEMPTS_QUERY, RUNNING_ATTEMPTS_QUERY_HOLDERS, SESSION_FORGED, SLOW_WORLD_BASE_SECONDS, SLOW_WORLD_STEPS,
    SLOW_WORLD_STEP_SECONDS, SOCKET_CONNECT_SECONDS, SOCKET_PATH_ALLOWANCE, STAGED_PEAK_BYTES, SUBMIT_TRACE_RECEIVED, SUBMIT_TRACE_STEPS,
    TICK_PROMPT_TICK_SECONDS, TOO_DEEP_SOCKET_DIRECTORY, UNREACHABLE_NOTICE, WAIT_LONG_POLL_SECONDS, attempt_lines, cell_association_column_verified,
    cell_attempt_row_not_overwritten, cell_check_missing, cell_child_rows_verified, cell_command_word_index_skips_option_values,
    cell_confirm_retries_follow_churn, cell_control_check_probe_bounded, cell_control_check_reads_service_nfs_report,
    cell_control_drop_history_oversized_records, cell_control_key_stats_by_task, cell_control_key_stats_dump, cell_control_key_stats_reported,
    cell_control_readback, cell_control_start, cell_control_watch_removed, cell_environment_prefix_stripped_from_key, cell_environment_whitelist,
    cell_error_types, cell_exit_code_passthrough, cell_forged_session, cell_install, cell_lookup_indexes_used, cell_nested_dispatch_refused,
    cell_operation_log, cell_prune_server_unavailable_outcome, cell_resident_entries_forgotten, cell_restart_keeps_running,
    cell_retired_outcome_status_generic, cell_retired_variables_refused, cell_running_attempts_read_once, cell_running_attempts_verified_only,
    cell_selftest_variables_refused, cell_server_unreachable, cell_slow_world_patient_wait, cell_socket_path_too_long, cell_start_once,
    cell_state_placement, cell_store_write_rolls_back_on_error, cell_submit_trace_logged, cell_tampered_row_not_started, cell_task_log,
    cell_thirty_second_tick_prompt, cell_threads_fixed, cell_unit_start_failed, code_objects_of, drop_history_rows, key_stats_seeder, log_printed,
    query_plan_text, raw_record_update, server_socket_count,
)
from task_server_selftest_queueing import (
    AGED_BACKDATE_MARGIN_SECONDS, ALTERNATION_THRESHOLD_BYTES, AT_TOTAL_CAP_PINNED_TEXT, CPU_GATE_BUSY_KEY, CPU_GATE_PEAK_KEY, DUTY_SPREAD_SCRIPT,
    FORMULA_HEAD_BLOCKER_PEAK_GIBIBYTES, FORMULA_HEAD_PEAK_GIBIBYTES, FORMULA_HEAD_ROOMY_GIBIBYTES, FORMULA_HEAD_WATCH_SECONDS, HUNDRED_RELEASE,
    NO_BORROW_HEAD, NO_BORROW_OTHER_HEAD, NO_BORROW_PARENT, NO_BORROW_SIBLING, PASS_COUNT, PASS_QUEUED_COUNT, PAUSE_SETTLE_SECONDS,
    PEER_RECOVER_HOLD_SECONDS, PROMOTE_WAIT_SECONDS, RECHECK_FAILED_TEXT, THREADS_OVER_REGISTERED_BINARY, attempt_order, attention_of_kind,
    backdate_past_aged, cell_aged_blocked_task_blocks_later, cell_aged_head_at_slice_total_starts, cell_aged_task_judged_first, cell_arrival_order,
    cell_balanced_when_both_fit, cell_blocked_head_not_starved, cell_cpu_gate_counts_running_threads, cell_cpu_gate_holds_busy_host,
    cell_cpu_gate_line_twice_cores, cell_cpu_gate_measured_cores, cell_cpu_gate_new_task_by_recorded_cores,
    cell_cpu_gate_new_task_counts_failed_runs, cell_cpu_gate_peak_segment_cores, cell_cpu_gate_runnable_threads,
    cell_cpu_gate_running_after_restart_by_average, cell_cpu_gate_running_average_capped_by_registered, cell_cpu_gate_running_by_latest_reading,
    cell_cpu_gate_running_started_by_recorded_cores, cell_cpu_gate_small_task_slots, cell_cpu_gate_unrecorded_by_registered_threads,
    cell_cpu_reason_splits_io_wait, cell_cpu_unread_not_stalled, cell_down_host_waits_not_never, cell_every_command_reaches_peer,
    cell_finish_peak_segment_cpu_unread, cell_health_head_cpu_gate_not_starved, cell_hundred_five_minute_tasks_balanced,
    cell_idle_short_not_blocking, cell_judge_error_skips_only_that_task, cell_jump_only_same_resource, cell_jump_only_shorter_than_running, cell_launch_rechecked,
    cell_pass_booking_carried, cell_pass_reads_resources_once, cell_peer_cleanup_off_scheduler, cell_peer_cooldown_skips_connect,
    cell_peer_probe_off_scheduler, cell_peer_starts_throttled, cell_peer_when_local_full, cell_prediction_computed_once,
    cell_promote_placement_rules, cell_promote_queue_order, cell_promote_skips_cpu_gate, cell_promote_wakes_and_promotes_shards,
    cell_queue_holds_unplaceable, cell_queued_revalidated_on_restart, cell_queued_shows_waiting_reason, cell_queued_whole_package_rechecked,
    cell_repass_cleared_when_idle, cell_retry_goes_where_it_fits, cell_shard_start_borrows_nothing, cell_starved_head_judged_by_the_formula,
    cell_starved_head_reported, cell_stop_joins_peer_probe, cell_threads_over_registered_peer_reported, cell_threads_over_registered_reported,
    cell_three_duties_two_sessions, cell_unstarted_slice_uses_configured_total, cell_whole_starts_beside_running_fragment, peer_probe_resources,
    peer_threads_script, promote_fake, recorded_host_line, seed_busy_cores, write_unmet_script,
)
from task_server_selftest_admission import (
    ADMISSION_CAPS_OVER_TOTAL, ADMISSION_MEASURED_TIGHT, ADMISSION_PROBE, ADMISSION_ROOMY, ADMISSION_WATCH_SECONDS, BOTH_HOSTS_CAP_TEXT,
    BOTH_HOSTS_PEAK_BYTES, BOTH_HOSTS_RELEASE, BOTH_HOSTS_USED_BYTES, CLAMP_CAP_HIT_BYTES, CLAMP_HOST_LIMIT_TEXT, CLAMP_HOST_SLICE_GIBIBYTES,
    CLAMP_PINNED_ITEM_BYTES, CLAMP_RECORDED_BYTES, CLAMP_RETRY_BYTES, CLAMP_TINY_SLICE_GIBIBYTES, CREDIT_SLOT_GIBIBYTES, CREDIT_SPARE_GIBIBYTES,
    DISK_FLOOR_RECORD_BYTES, DRIFT_LABELS, DRIFT_QUEUED_TASK, INVARIANT_COUNT_PER_KIND, INVARIANT_DRAIN_PASSES, INVARIANT_FILL_PASSES,
    INVARIANT_KINDS, INVARIANT_RELEASES, IO_PRESSURE_SAMPLE, MISSING_SCOPE_CAP_TEXT, MISSING_SCOPE_FLOOR_BYTES, MISSING_SCOPE_PEAK_BYTES,
    MISSING_SCOPE_RELEASE, MISSING_SCOPE_USED_BYTES, PAGE_CACHE_ANON_BYTES, PAGE_CACHE_FILE_BYTES, PAGE_CACHE_KERNEL_BYTES, PER_HOST_CAP_TEXT,
    PER_HOST_LARGE_MARGIN_GIBIBYTES, PER_HOST_PEAK_BYTES, PER_HOST_SMALL_MARGIN_GIBIBYTES, RUNNING_BOOKING_CASES, SELFTEST_TIGHT_MARGIN_GIBIBYTES,
    SHARD_FITTED_FLOOR_BYTES, YOUNG_AUDIT_AGE_SECONDS, YOUNG_AUDIT_SECONDS, YOUNG_AUDIT_USED_BYTES, admission_judged, admission_scheduler,
    booked_caps_by_host, booking_audit_scheduler, cell_admission_booked_at_cap, cell_admission_by_measured_bookings,
    cell_booking_audit_young_not_drift, cell_booking_drift_reported, cell_booking_ledger_rows, cell_booking_within_host_limit_every_history,
    cell_budget_source_names_folded_key, cell_budget_source_only_when_reported, cell_build_disk_credits_free_slot, cell_build_disk_predicted,
    cell_cap_hit_record_over_every_host_starts_clamped, cell_disk_floor_peer_not_never_fits, cell_io_pressure_gate_by_host,
    cell_io_pressure_queues_compiling_task, cell_io_pressure_reading_recorded, cell_machine_memory_line_holds_dispatch,
    cell_margin_measured_per_host, cell_margin_unmeasured_not_placed, cell_memory_wait_names_both_numbers,
    cell_oversized_budget_acknowledged_after_restart, cell_oversized_budget_reported, cell_page_cache_not_counted_as_process_memory,
    cell_peer_booked_like_local, cell_peer_scope_missing_booked_at_cap, cell_pin_cap_refused_within_margin, cell_pinned_peer_unavailable,
    cell_running_booked_at_measured_usage, cell_running_booked_unscoped_at_cap, cell_running_young_booked_at_cap, cell_shard_cap_fitted_to_room,
    cell_two_hosts_booked_within_capacity, cell_workspace_writer_cap_has_headroom, io_pressure_configuration, ledger_fake, machine_memory_scheduler,
    machine_memory_snapshot, occupy_hosts,
)
from task_server_selftest_memory import (
    ACTIVITY_GROWTH_PER_TICK_BYTES, ACTIVITY_GROWTH_TICK_SECONDS, ACTIVITY_HELD_WINDOWS, ACTIVITY_QUIET_COMMAND, ACTIVITY_STALL_SECONDS,
    AT_CAP_GROWN_PINNED_TEXT, CAP_HIT_GROWN_PINNED_TEXT, CAP_HIT_PINNED_TEXT, CAP_HIT_SEED_PEAK_BYTES, ESTIMATE_COMMAND_KEY, FAKE_APPLY_WRITER_PATCH,
    FLOOR_NEAR_PEAK_BYTES, FLOOR_RETRY_TEXT, FLOOR_TEXT, FLOOR_TINY_PEAK_BYTES, FULLY_APPLIED_OUTCOME, HALF_APPLIED_OUTCOME, HISTORY_CAP_HIT_BYTES,
    HISTORY_CAP_HIT_TEXT, HISTORY_SMALL_PEAK_BYTES, HISTORY_SMALL_PEAK_TEXT, MEMORY_FLOOR_RECORDED_SEED_BYTES, MEMORY_FLOOR_RECORDED_TEXT,
    MEMORY_FLOOR_RETRY_TEXT, PEAK_STAT_ANON_BYTES, PEAK_STAT_FILE_BYTES, PEAK_STAT_KEY, PEAK_STAT_KILLED_SCRIPT, PEAK_STAT_TEXTS,
    PEER_PINNED_CURRENT_BYTES, PEER_PINNED_SEED_PEAK_BYTES, PEER_STAT_KEY, PEER_STAT_SCRIPT, PINNED_BELOW_FLOOR_BYTES, PINNED_BELOW_FLOOR_SLICE_TEXT,
    PINNED_LOCAL_TEXT, PINNED_PEER_TEXT, RECORDED_SMALL_CAP_TEXT, RECORDED_SMALL_PEAK_BYTES, RECORD_WINDOW_OVERRUN, SHORT_EXIT_PEAK_BYTES,
    SHORT_SAMPLED_PEAK_BYTES, STOPPED_UNIT_ATTEMPT, UNFINISHED_PEAK_BYTES, UNRECORDED_BURST_COUNT, UNRECORDED_GROWTH_PINNED_TEXTS,
    WINDOW_HEAD_BASE_BYTES, WINDOW_HEAD_GROWTH_BYTES, WINDOW_HEAD_ROUND_SECONDS, WINDOW_HEAD_STALL_SECONDS, WindowReadingMachine,
    by_host_written_wrong, cell_busy_cpu_below_cap_keeps_running, cell_busy_cpu_pinned_at_cap_retried, cell_by_host_missing_host_refused,
    cell_by_host_outside_address_refused, cell_cap_hit_low_peak_not_recorded_at_cap, cell_cap_hit_starts_above, cell_completed_after_cap_hit,
    cell_history_peak_first_attempt, cell_history_peak_unrecorded_default, cell_history_peak_window_raises, cell_hook_swap_outcome_judged,
    cell_last_run_seconds_from_records,
    cell_memory_cap_at_maximum, cell_memory_cap_growth, cell_memory_cap_passed_to_command, cell_memory_growth_not_stalled,
    cell_memory_window_head_read_at_open, cell_peak_memory_stat_in_note, cell_peer_memory_peak_recorded, cell_peer_peak_memory_stat_in_note,
    cell_peer_pinned_at_cap_busy_cpu_retried, cell_pin_cap_help_not_pinned, cell_pinned_cap_by_host, cell_pinned_cap_from_configuration,
    cell_pinned_cap_over_limit_reported, cell_pinned_cap_unreadable_refused, cell_pinned_squeezed_requeued, cell_recorded_cap_not_floored,
    cell_resource_record, cell_retry_build_directory_reused, cell_retry_cap_not_floored, cell_retry_keeps_earlier_attempts, cell_retry_kinds,
    cell_short_command_peak_recorded, cell_squeezed_at_total_is_cap_hit, cell_stall_reads_log_before_cpu, cell_stalled_at_cap_retried,
    cell_stopped_peaks_floor_record, cell_unfinished_peak_ignored, cell_unit_stopped_booking_released, cell_unmeasured_not_zero,
    cell_unrecorded_burst_balanced, cell_unrecorded_cap_growth, cell_unrecorded_default_small, cell_workspace_write_blocked_reminded,
    cell_workspace_write_complete_not_reported, cell_workspace_write_interrupted_reported, cell_workspace_writer_not_retried,
    cell_wrapped_hook_swap_reminded, cell_wrapped_patch_directory_resolved, cell_wrapped_patch_directory_unresolvable,
    cell_wrapped_previous_swap_named, cell_wrapped_read_only_hook_swap_reminded,
    cell_wrapped_writer_interrupted_judged, cell_wrapped_writer_read_only_retried, cell_wrapped_writers_each_judged, cell_wrapped_writer_script_body_not_retried, cell_wrapped_writer_shell_string_not_retried,
    cell_writer_dry_run_retried, grow_process_memory, patched_server_name, peer_world, task_command_ok, wrapper_cap_texts, write_patch_writer_world,
)
from task_server_selftest_limits import (
    ACROSS_CAP_HITS_SCRIPT, CLIENT_NO_SUCH_TASK_EXIT, EXTERNAL_KILL_COMMAND, FILLS_CAP_CLIMB_CAPS, FILLS_CAP_CLIMB_PEAK_PERCENT,
    FILLS_CAP_GROWN_CAPS, FILLS_CAP_RECORD_CAPS, FILLS_CAP_RECORD_PEAKS, HARD_LIMIT_OUT_OF_REACH_SECONDS, LONG_RUN_ACROSS_ROUNDS,
    LONG_RUN_IDLE_ATTENTION_SECONDS, LONG_RUN_SELFTEST_SECONDS, LONG_RUN_SELFTEST_SLEEP_SECONDS, LOOP_LEAK_BOUNCE_CAPS, LOOP_LEAK_BOUNCE_KEY,
    LOOP_LEAK_BOUNCE_PEAKS, LOOP_LEAK_SCOPE_FILES, MACHINE_OOM_KILL_LINE, OLD_METRIC_PEAK_BYTES, PEER_CGROUP_KILL_LINE, PEER_KILL_LINE,
    RELEASE_HOLD_SCRIPT, RELEASE_WAIT_SECONDS, REPEATED_CAP_HITS_KEY, REPEATED_CAP_HITS_RELEASE, RUN_HARD_LIMIT_ACROSS_SECONDS,
    RUN_HARD_LIMIT_ADVICE, RUN_HARD_LIMIT_CONFIRM_SECONDS, RUN_HARD_LIMIT_EXIT, RUN_HARD_LIMIT_NOTE, RUN_HARD_LIMIT_SHARD_SECONDS,
    RUN_LIMIT_ACROSS_SECONDS, RUN_LIMIT_SHORT_SECONDS, SPLIT_EXEMPT_ITEM, cell_external_kill_peer_line_recorded_and_retried,
    cell_external_kill_recorded_and_retried, cell_fills_any_cap_cap_hits_not_reported, cell_fills_any_cap_record_caps_not_reported,
    cell_fills_any_cap_reported, cell_long_run_check_not_exempt, cell_long_run_confirm_refused, cell_long_run_confirm_silences,
    cell_long_run_confirmed_covers_shards, cell_long_run_failure_noted, cell_long_run_full_gate_not_reminded, cell_long_run_idle_cores_reported,
    cell_long_run_merge_and_verify_not_exempt, cell_long_run_reminded_not_stopped, cell_long_run_reminder_across_cap_hits,
    cell_long_run_sessionless_noted, cell_long_run_split_reminded_at_outcome, cell_loop_leak_check_on_bounce, cell_loop_leak_trend_sentences,
    cell_old_metric_peaks_not_counted, cell_repeated_cap_hits_reminded_once, cell_run_hard_limit_applies_after_restart,
    cell_run_hard_limit_arithmetic, cell_run_hard_limit_cap_hit_resets, cell_run_hard_limit_full_gate_not_stopped, cell_run_hard_limit_shard_stopped,
    cell_run_hard_limit_stop_retried, cell_run_hard_limit_stopped, cell_run_hard_limit_why_still_stops, cell_run_hard_limit_writer_stopped,
    cell_shard_hard_limit_by_each_piece, cell_signal_exit_named, external_kill_scheduler, external_kill_unit_problems, hard_limit_fake_scheduler,
    hard_limit_problems, hard_limit_settings, held_command, held_until_finished, long_run_head, peer_external_kill_scheduler, reminded_problems,
    running_attempt_log, signal_note_scheduler, split_items_of,
)
from task_server_selftest_stalls import (
    ATTENTION_IO_WAIT_SECONDS, ENCLOSING_UNIT_SAMPLES, FAKE_BLOCKED_PROCESS, FALLBACK_TEMPORARY_BASE, GRACE_CELL_GRACE_SECONDS,
    GRACE_CELL_STALL_SECONDS, HEARTBEAT_MIXED_SCRIPT, HEARTBEAT_ONLY_SCRIPT, IO_WAIT_SUSPECT_WAITS, LEFTOVERS_ERROR_EXIT, LEFTOVERS_ERROR_RUNNER,
    LEFTOVER_GONE_SECONDS, LOG_QUIET_PINNED_SECONDS, OUTER_UNIT_LAUNCHER, OUTER_UNIT_STOP_SECONDS, PEER_HELD_WINDOWS, PEER_HOLD_ROUNDS,
    PEER_STALL_SECONDS, PEER_STOP_QUIET_SECONDS, PROGRESS_FREE_GIBIBYTES, PROGRESS_HEALTH_TEXT, PROGRESS_WATCH_SECONDS, RUNAWAY_RESTART_GAP_SECONDS,
    RUNAWAY_RESTART_OLD_PID, RUNAWAY_RESTART_PID, RUNNER_FINAL_SAMPLE_WRAPPER, RUNNER_THREADS_NAME, RUNNER_THREADS_PROGRAM, RUNNER_THREADS_SECONDS,
    RUNNER_THREADS_SLEEPING, RUNNER_THREADS_SPINNING, STOPPED_UNIT_STATES, UNIT_UNREADABLE_HOLD_SECONDS, UNIT_UNREADABLE_WAIT_SCRIPT,
    VERDICT_PROGRESS_ONCE_SCRIPT, VERDICT_PROGRESS_STEADY_SCRIPT, cell_attempt_progress_readings, cell_attempt_unit_nofile_raised,
    cell_cancel_start_race, cell_computing_quietly_not_stalled, cell_heartbeat_lines_not_log_growth, cell_io_wait_local_not_stalled,
    cell_io_wait_peer_not_stalled, cell_leftover_processes_stopped, cell_leftovers_error_isolated, cell_leftovers_only_in_own_unit,
    cell_log_content_serialized, cell_log_noise_marks_not_growth, cell_peer_readings_merged_across_reads, cell_peer_ssh_multiplexed,
    cell_peer_stall_judged_by_peer_cpu, cell_progress_line_names_progress, cell_progress_stall_reported, cell_quiet_log_busy_spared_by_progress,
    cell_runaway_process_notified, cell_runaway_process_stopped_after_reports, cell_runaway_reports_survive_restart,
    cell_runaway_reused_pid_counted_afresh, cell_runaway_targets_skip_children, cell_runner_final_sample_after_exit,
    cell_runner_start_error_recorded, cell_runner_threads_file_counts, cell_running_status_shows_progress, cell_selftest_units_stop_with_outer,
    cell_stall_clock_from_log_mtime, cell_stall_stop_spares_next_attempt, cell_stall_waits_for_grace, cell_stalled_attempt_stopped,
    cell_suspected_stall_notified, cell_unit_unreadable_exit_file_settles, cell_unit_unreadable_gone_recorded_lost,
    cell_unit_unreadable_running_reminded, cell_unit_unreadable_unverified_health_reported, cell_verdict_library_progress_not_stalled,
    idle_cpu_scope, peer_hold_script, peer_io_wait_script, progress_quiet_scheduler, quiet_log_busy_problems, release_all, runaway_listing_line,
    runaway_restart_scheduler, suspect_lines, unit_active_state, unit_unreadable_lines, wait_unit_unreadable_line,
)
from task_server_selftest_notifying import (
    ADVICE_SAMPLE_TASK, ATTENTION_ONCE_QUIET_SECONDS, ATTENTION_ONCE_SECONDS, ATTENTION_QUEUE_SECONDS, ATTENTION_RESTART_SECONDS,
    CLEANUP_PROTOCOL_TESTS, FakeInboxScheduler, GPU_BLOCK_NOT_JUDGEABLE_SELFTEST_LINE, GPU_CARD_FAILED_SELFTEST_LINE,
    GPU_NOT_RUN_LINE_AFTER_TEST_START, GPU_NOT_RUN_SAMPLE_TESTS, GPU_ON_CONFIGURATION_KEYS, GPU_PEER_SELFTEST_HOST, GPU_SERVICE_START_TEXT,
    HEADLINE_FACTS, HEALTH_PROBES, HEALTH_SETTLE_SECONDS, INBOX_COLLECTED_SECONDS, INBOX_RESEND_TASKS, INBOX_SHARD_ATTENTION_SECONDS,
    INBOX_SHARD_BACKDATE_MARGIN_SECONDS, INBOX_SHARD_KEY, PEER_HEALTH_HEAD, PEER_SLOW_LOG_TEXT, PROGRESS_LOG_LINE_ROUNDS,
    PROGRESS_PUSH_SAMPLE_SECONDS, REASON_LIVE_CASES, REASON_PENDING_QUIET_SECONDS, REMINDER_LEDGER_KIND, REMINDER_RESTART_HEALTH,
    SLOW_RESOURCES_SECONDS, UNCLAIMED_REPUSH_SAMPLE_SECONDS, UNCLAIMED_WARNING_SAMPLE_SECONDS, cell_anomaly_acknowledged_not_repushed,
    cell_anomaly_advice_in_texts, cell_anomaly_repushed_with_history, cell_anomaly_table_matches_kinds, cell_attention_cleanup_pending,
    cell_attention_line_why_clipped, cell_attention_queue_waits_for_reason, cell_attention_queued_too_long, cell_attention_quiet_busy_log,
    cell_attention_reported_once, cell_attention_reported_survives_restart, cell_attention_shard_advance_failed,
    cell_attention_type_in_anomaly_table, cell_cleanup_protocol_unit_tests, cell_confirmation_scope_text,
    cell_gpu_block_not_judgeable_one_per_stream, cell_gpu_block_not_judgeable_outlet_follows_claim,
    cell_gpu_card_lines_match_the_service_fields, cell_gpu_cases_not_run_reported, cell_gpu_health_survives_restart,
    cell_gpu_health_unread_host_survives_restart, cell_gpu_not_run_line_forms,
    cell_gpu_not_run_marks_per_line, cell_gpu_service_report_keeps_unread_hosts, cell_gpu_service_required_when_gpu_on,
    cell_gpu_service_untouched_when_gpu_off, cell_health_answers_while_resources_slow, cell_health_database_unwritable_notified,
    cell_health_gpu_block_not_judgeable_notified, cell_health_gpu_card_failed_notified, cell_health_nfs_notified,
    cell_health_peer_unavailable_notified, cell_health_unit_missing_notified, cell_inbox_headlines_tell_kind_state_round, cell_inbox_outcome_failed,
    cell_inbox_outcome_succeeded, cell_inbox_outcomes_resent_after_error, cell_inbox_queued_too_long_once, cell_inbox_shard_attention_merged,
    cell_inbox_socket_follows_session, cell_inbox_unreachable_task_runs, cell_nfs_share_required, cell_nfs_share_verified,
    cell_peer_health_slow_rechecked, cell_peer_health_slow_within_recheck_period, cell_peer_unavailable_alerted, cell_previous_swap_running_reminded,
    cell_reason_line_per_type, cell_recheck_judges_disk, cell_recheck_judges_nfs, cell_reminder_ledger_other_shape_read_afresh,
    cell_reminder_thresholds_every_doubling, cell_reminders_not_resent_after_restart, cell_running_progress_pushed, cell_unclaimed_finished,
    cell_unclaimed_outcome_repushed, cell_unclaimed_results_warned, gate_library_marker_sample, gpu_not_run_judged, gpu_on_configuration,
    gpu_service_calls, gpu_service_line_fields_of_the_source, headline_facts, health_notified_once, inbox_outcome_case, queued_reminder_scheduler,
    reason_problems, reminder_ledger_task, reminder_restart_scheduler, server_output_text, submitted_sheet, unclaimed_head, write_gpu_service_exit,
    write_unit_state,
)
from task_server_selftest_shards import (
    CANCEL_ARTIFACT_WAIT_SECONDS, CANCEL_LOCAL_SCRIPT, CARGO_CLASS_BASE, CARGO_CLASS_FAKE_WORDS, CARGO_CLASS_PEAK_BYTES, CHILD_SETTLE_HOLD_SECONDS,
    CRASH_CASE_SELECTION, CRASH_GATE_WORDS, PLAN_KEY_COMPILED_BYTES, PLAN_KEY_HEAD, PLAN_KEY_UNCOMPILED_BYTES, PROBE_CHILD_GONE_SECONDS,
    SCENE_GONE_SECONDS, SHARD_BINARIES_NAMES, SHARD_BINARIES_NOTE_SECONDS, SHARD_CRASH_POINTS, SHARD_KEY_BASES, SHARD_MERGE_EXIT, SHARD_PEAK_KEY,
    SHARD_PLAN_EXIT, SHRUNK_DOMAIN_VARIABLE, SPACED_ITEM_NAME, SPACED_ITEM_TAIL, TASK_LOCK_ROUNDS, UNSHARDED_PINNED_ARGUMENTS,
    cell_cancel_after_natural_end, cell_cancel_finished, cell_cancel_own_session_only, cell_cancel_queued, cell_cancel_sharded_in_plan,
    cell_cancel_sharded_running, cell_cancelled_artifacts_removed, cell_cancelled_before_peer_trip_built, cell_cargo_test_key_by_package,
    cell_compiles_declared_entry, cell_entry_script_key_by_repository_path, cell_failure_scene_kept_until_collected,
    cell_merge_on_peer_reads_shard_outputs, cell_parent_cancel_stops_every_shard, cell_parent_cancel_waits_for_children,
    cell_parent_cancelling_child_not_claimed, cell_plan_key_by_compiles, cell_plan_key_without_shrunk_domain, cell_scene_sessionless_swept,
    cell_selection_key_by_script, cell_selection_key_positionals_after_options, cell_shard_binaries_removed_at_outcome,
    cell_shard_cancel_alone_merges, cell_shard_child_key_folded_by_words, cell_shard_failure_waits_for_running, cell_shard_host_is_address,
    cell_shard_key_without_count, cell_shard_keys_kept_apart, cell_shard_merge_failed, cell_shard_outputs_written_by_two_drivers,
    cell_shard_peak_record_count_matched, cell_shard_plan_failed, cell_shard_retry_isolated, cell_shard_siblings_share_a_host,
    cell_shard_to_peer_by_plan, cell_sharded_cancel_retried, cell_sharded_progress_recovered, cell_stop_ends_probe_children,
    cell_task_locks_released, cell_unsharded_arguments, exited_within, shard_binaries_notes, shard_plan_directory_of,
)
from task_server_selftest_shared_roots import (
    CARD_CHANNEL_PREFIX, CARD_CHOICE_SAMPLE, CARD_GATE_ENTRY, FAKE_RECOVER_CLEANUP, NFS_PROBE_READ_DELAY_SECONDS, SLASH_IN_ARGUMENT,
    VOLATILE_SAMPLE_PATH, cell_attempt_gets_no_card, cell_attempt_label_passed_to_command, cell_data_root_key_required,
    cell_gate_stages_sent_to_peer, cell_nfs_automount_recognized, cell_nfs_probe_concurrent_runs, cell_outside_shared_path_refused,
    cell_path_check_whole_words, cell_peer_admission_marked, cell_peer_cleanup_recovered_once, cell_peer_command_exit_split, cell_peer_disk_floor,
    cell_peer_retry_build_directory_reused, cell_peer_run_failed_retried, cell_peer_runs_in_shared_tree, cell_shared_roots_outside_refused,
    cell_shared_roots_passed_to_command, cell_source_repository_passed_to_command, peer_env_words,
)
from task_server_selftest_refusals import (
    BYPASS_NEXT_STEP_HEAD, BYPASS_SCRIPT_TEXT, BYPASS_WARNING_HEAD, CARD_DECLARATION_SAMPLES, CLIENT_REFUSED_EXIT, CONFIRM_HOLD_SECONDS,
    CONFIRM_LOCK_PROBE_SECONDS, CONFIRM_SCOPE_SAMPLE, CONFIRM_TIMEOUT_SAMPLE_SECONDS, COPY_PROOF_ADVICE_TEXTS, COPY_PROOF_BUILD, COPY_PROOF_NO_BUILD,
    FAKE_LOAD_SIMULATION, FEATURE_GATED_CHECK, FEATURE_ITEMS_ACCEPTED_CASES, FEATURE_ITEMS_OTHERS_REFUSED_CASES, LOAD_SIMULATION_SCRIPT,
    LONG_TASK_DIRECT_FAKE_ENTRIES, LONG_TASK_JUDGED, LONG_TASK_RECORD_KEYS, LONG_TASK_WHOLE_SCOPE, SELFTEST_SAMPLE_SCRIPT_TEXT,
    SERVICE_TARGET_SCRIPT_TEXT, UNNAMED_CRASH_GATE, UNNAMED_PATCH_DIRECTORY, UNNAMED_RUN_JUDGED, UNSCOPED_RUN_CASES, WHOLE_PACKAGE_COMMANDS,
    WHOLE_PACKAGE_CONFIRMATION_TEXT, WHOLE_PACKAGE_SINGLE_CELLS, WHOLE_WORKSPACE_MEMBER, WHOLE_WORKSPACE_NEXT_TEXTS, WRAPPED_SELFTEST_SCRIPT,
    cell_card_declarations_refused, cell_commit_gate_query_forms_accepted, cell_concurrent_confirms_serialized, cell_confirm_on_queued_task_accepted,
    cell_confirm_sheet_reads_bash_c_inner_commands, cell_copy_proof_build_refused, cell_copy_proof_start_refused, cell_full_gate_flag,
    cell_long_task_by_record, cell_long_task_sharded, cell_long_task_unrecorded_test_or_item, cell_long_task_whole_run, cell_manual_submit_path,
    cell_repository_target_bypass_warned, cell_short_task_queued_directly, cell_submit_awaits_confirmation, cell_temporary_service_heavy_refused,
    cell_temporary_service_own_server_compiles, cell_temporary_service_wrapper_compiles, cell_temporary_service_wrapper_unwrapped,
    cell_test_starting_cells_listed, cell_unconfirmed_cancelled, cell_unnamed_cargo_exact_accepted, cell_unnamed_cargo_target_refused,
    cell_unnamed_crash_gate_cases_only_accepted, cell_unnamed_crash_gate_whole_items_refused, cell_unnamed_exemptions_accepted,
    cell_unnamed_gate_cells_judged, cell_unnamed_merge_named_accepted, cell_unnamed_merge_unnamed_refused, cell_unnamed_prove_red_named_accepted,
    cell_unnamed_prove_red_unnamed_refused, cell_unscoped_runs_refused, cell_verdict_library_family_refused, cell_whole_package_confirmation_form,
    cell_whole_package_confirmed_still_reminded, cell_whole_package_feature_items_accepted, cell_whole_package_feature_items_others_refused,
    cell_whole_package_named_accepted, cell_whole_package_single_cells_refused, cell_whole_package_unconfirmed_refused,
    cell_whole_workspace_cargo_refused, copy_proof_cancel, copy_proof_run, copy_proof_try, copy_proof_world, judged_direct_problems, judged_id,
    judged_long_problems, judged_submit, submit_confirmed, submit_raw, submitted_refusal_problems, unnamed_run_problems, write_confirmation,
)
from task_server_selftest_lifecycle import (
    DRAINED_FRAGMENT_PEAK_BYTES, PAUSED_WATCH_SECONDS, PAUSE_REASON_SAMPLE, POOL_GONE_SECONDS, RETRY_AFTER_RELEASE_SCRIPT, SHARD_ADVANCE_PARENT_ID,
    TAMPERED_FOREIGN_COLUMN, TAMPERED_HOLDER_KEY, TAMPERED_SUMMARY_READS, TAMPERED_WAITER_KEY, TAMPERED_WAITER_SECONDS,
    cell_advance_episodes_survive_restart, cell_attempt_claim_atomic, cell_build_directory_outside_repository, cell_build_pool_removed_with_tree,
    cell_build_slot_concurrent_distinct, cell_build_slot_reused_on_same_tree, cell_build_slot_separate_per_tree, cell_clean_failure_explained,
    cell_clean_keeps_build_pool, cell_ended_attempt_settled_after_restart, cell_failed_start_not_counted, cell_pause_allow_reported,
    cell_pause_allow_survives_restart, cell_pause_allow_switches_session, cell_pause_allow_waiting_text, cell_pause_allows_one_session,
    cell_pause_holds_new_attempts, cell_pause_keeps_queue_positions, cell_pause_lets_running_finish, cell_pause_not_starved, cell_pause_reported,
    cell_pause_survives_restart, cell_pause_waits_for_starting_attempt, cell_restart_surviving_tampered_not_adopted,
    cell_resume_clears_allowed_session, cell_resume_declaration_first_word_only, cell_resume_releases_queue, cell_start_timeout_read_back,
    cell_tampered_attempt_ended_once, cell_tampered_attempt_unit_stopped, cell_unpooled_label_directories_removed, cell_unpooled_label_directory_of_running_attempt_kept, held_in_queue, pause_by_api, pause_health,
    tampered_alerts,
)


def cell_type_table_covered(world):
    missing = sorted(set(lib.OUTCOME_TYPES) - set(EXERCISED_BY))
    extra = sorted(set(EXERCISED_BY) - set(lib.OUTCOME_TYPES))
    unknown_cells = sorted({cell for cell in EXERCISED_BY.values() if cell not in CELLS})
    if missing or extra or unknown_cells:
        raise SelftestFailure(f"错误类型表每一种都要有格喂到：没格喂的 {missing}、表里没有的 {extra}、点名的格不在 {unknown_cells}")
    return f"错误类型表 {len(lib.OUTCOME_TYPES)} 种都有格喂到"


def cell_parallelism_registration_refused(world):
    """各格并行的路数取不出登记的每台默认重活线程数就判红，不退回按核数（本文件的 parallelism；.claude/heavy-thread-default 文件头：写坏或删掉这份文件，取法一律拒绝，不退回按核数算）：
    造四份样本交给 parallelism：SINGLEFS_THREAD_CAP=abc、登记文件不在、登记那一行没写理由，都交 (None, 原因, 出路)，原因里点名是哪一样（变量名与值、文件路径、第几行），出路不空；
    登记「7  # 样本理由」交 7，SINGLEFS_THREAD_CAP=5 盖过登记交 5。弄坏开关 parallelism-falls-back-to-cores 下前三份按核数交数，这一格红。"""
    good, bad, missing = (os.path.join(world.root, name) for name in ("registration-good", "registration-no-reason", "registration-missing"))
    for path, text in ((good, "7  # 样本理由\n"), (bad, "7\n")):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
    refused = (("SINGLEFS_THREAD_CAP=abc", {"SINGLEFS_THREAD_CAP": "abc"}, good, "SINGLEFS_THREAD_CAP=abc"),
               ("登记文件不在", {}, missing, missing),
               ("登记那一行没写理由", {}, bad, "第 1 行"))
    accepted = (("登记 7", {}, good, 7), ("SINGLEFS_THREAD_CAP=5 盖过登记的 7", {"SINGLEFS_THREAD_CAP": "5"}, good, 5))
    cores = max(1, os.cpu_count() or 1)
    problems = []
    for what, environment, registration, mention in refused:
        found, reason, howto = parallelism(environment, registration)
        if found is not None or mention not in reason or not howto:
            problems.append(f"{what}：应当交 (None, 点名「{mention}」的原因, 不空的出路)，实际 ({found}, 「{reason}」, 「{howto}」)；不退回按核数（本机 {cores} 核）")
    for what, environment, registration, wanted in accepted:
        found, reason, howto = parallelism(environment, registration)
        if found != wanted or reason or howto:
            problems.append(f"{what}：应当交 ({wanted}, 空, 空)，实际 ({found}, 「{reason}」, 「{howto}」)")
    if problems:
        raise SelftestFailure("并行度取登记的线程数：" + "；".join(problems))
    return f"取不出登记的线程数的 {len(refused)} 份样本都判红、点名了原因；登记值与 SINGLEFS_THREAD_CAP 的 {len(accepted)} 份照取"


CELLS = {
    "retired-outcome-status-generic": cell_retired_outcome_status_generic,
    "pause-allows-one-session": cell_pause_allows_one_session, "pause-allow-waiting-text": cell_pause_allow_waiting_text,
    "pause-allow-switches-session": cell_pause_allow_switches_session, "resume-clears-allowed-session": cell_resume_clears_allowed_session,
    "pause-allow-survives-restart": cell_pause_allow_survives_restart, "pause-allow-reported": cell_pause_allow_reported,
    "pause-holds-new-attempts": cell_pause_holds_new_attempts, "pause-lets-running-finish": cell_pause_lets_running_finish,
    "pause-survives-restart": cell_pause_survives_restart, "resume-releases-queue": cell_resume_releases_queue, "pause-reported": cell_pause_reported,
    "pause-not-starved": cell_pause_not_starved, "pause-waits-for-starting-attempt": cell_pause_waits_for_starting_attempt,
    "attention-queued-too-long": cell_attention_queued_too_long,
    "attention-cleanup-pending": cell_attention_cleanup_pending,
    "attention-shard-advance-failed": cell_attention_shard_advance_failed, "attention-reported-once": cell_attention_reported_once,
    "attention-reported-survives-restart": cell_attention_reported_survives_restart, "attention-queue-waits-for-reason": cell_attention_queue_waits_for_reason,
    "page-cache-not-counted-as-process-memory": cell_page_cache_not_counted_as_process_memory,
    "attention-type-in-anomaly-table": cell_attention_type_in_anomaly_table, "temporary-service-wrapper-compiles": cell_temporary_service_wrapper_compiles,
    "temporary-service-own-server-compiles": cell_temporary_service_own_server_compiles,
    "slow-world-patient-wait": cell_slow_world_patient_wait,
    "parallelism-registration-refused": cell_parallelism_registration_refused,
    "shard-child-key-folded-by-words": cell_shard_child_key_folded_by_words,
    "workspace-write-interrupted-reported": cell_workspace_write_interrupted_reported,
    "workspace-write-complete-not-reported": cell_workspace_write_complete_not_reported,
    "resume-declaration-first-word-only": cell_resume_declaration_first_word_only,
    "check-missing": cell_check_missing, "install": cell_install, "control-readback": cell_control_readback, "server-unreachable": cell_server_unreachable,
    "exit-code-passthrough": cell_exit_code_passthrough, "restart-keeps-running": cell_restart_keeps_running, "start-once": cell_start_once,
    "queue-holds-unplaceable": cell_queue_holds_unplaceable, "arrival-order": cell_arrival_order,
    "prediction-computed-once": cell_prediction_computed_once, "pass-reads-resources-once": cell_pass_reads_resources_once,
    "pass-booking-carried": cell_pass_booking_carried, "launch-rechecked": cell_launch_rechecked, "every-command-reaches-peer": cell_every_command_reaches_peer,
    "error-types": cell_error_types,
    "unit-start-failed": cell_unit_start_failed, "memory-cap-growth": cell_memory_cap_growth, "memory-cap-at-maximum": cell_memory_cap_at_maximum,
    "unrecorded-default-small": cell_unrecorded_default_small, "unrecorded-cap-growth": cell_unrecorded_cap_growth,
    "unrecorded-burst-balanced": cell_unrecorded_burst_balanced,
    "retry-kinds": cell_retry_kinds, "cancel-own-session-only": cell_cancel_own_session_only,
    "tampered-row-not-started": cell_tampered_row_not_started, "queued-revalidated-on-restart": cell_queued_revalidated_on_restart, "forged-session": cell_forged_session, "environment-whitelist": cell_environment_whitelist,
    "retired-variables-refused": cell_retired_variables_refused,
    "nested-dispatch-refused": cell_nested_dispatch_refused, "resource-record": cell_resource_record, "operation-log": cell_operation_log,
    "unclaimed-finished": cell_unclaimed_finished, "state-placement": cell_state_placement,
    "inbox-outcome-succeeded": cell_inbox_outcome_succeeded, "inbox-outcome-failed": cell_inbox_outcome_failed,
    "inbox-queued-too-long-once": cell_inbox_queued_too_long_once, "inbox-unreachable-task-runs": cell_inbox_unreachable_task_runs,
    "health-database-unwritable-notified": cell_health_database_unwritable_notified, "health-unit-missing-notified": cell_health_unit_missing_notified,
    "health-nfs-notified": cell_health_nfs_notified, "health-peer-unavailable-notified": cell_health_peer_unavailable_notified,
    "health-gpu-card-failed-notified": cell_health_gpu_card_failed_notified,
    "health-gpu-block-not-judgeable-notified": cell_health_gpu_block_not_judgeable_notified,
    "gpu-service-report-keeps-unread-hosts": cell_gpu_service_report_keeps_unread_hosts,
    "gpu-card-lines-match-the-service-fields": cell_gpu_card_lines_match_the_service_fields,
    "gpu-health-survives-restart": cell_gpu_health_survives_restart,
    "gpu-health-unread-host-survives-restart": cell_gpu_health_unread_host_survives_restart,
    "gpu-block-not-judgeable-outlet-follows-claim": cell_gpu_block_not_judgeable_outlet_follows_claim,
    "gpu-block-not-judgeable-one-per-stream": cell_gpu_block_not_judgeable_one_per_stream,
    "control-watch-removed": cell_control_watch_removed, "unclaimed-results-warned": cell_unclaimed_results_warned,
    "retry-keeps-earlier-attempts": cell_retry_keeps_earlier_attempts, "build-directory-outside-repository": cell_build_directory_outside_repository,
    "retry-build-directory-reused": cell_retry_build_directory_reused,
    "history-peak-first-attempt": cell_history_peak_first_attempt, "history-peak-window-raises": cell_history_peak_window_raises,
    "history-peak-unrecorded-default": cell_history_peak_unrecorded_default, "writer-dry-run-retried": cell_writer_dry_run_retried,
    "health-head-cpu-gate-not-starved": cell_health_head_cpu_gate_not_starved,
    "cap-hit-record-over-every-host-starts-clamped": cell_cap_hit_record_over_every_host_starts_clamped,
    "booking-within-host-limit-every-history": cell_booking_within_host_limit_every_history,
    "control-drop-history-oversized-records": cell_control_drop_history_oversized_records,
    "oversized-budget-reported": cell_oversized_budget_reported, "long-run-idle-cores-reported": cell_long_run_idle_cores_reported,
    "fills-any-cap-reported": cell_fills_any_cap_reported,
    "old-metric-peaks-not-counted": cell_old_metric_peaks_not_counted,
    "anomaly-table-matches-kinds": cell_anomaly_table_matches_kinds, "anomaly-advice-in-texts": cell_anomaly_advice_in_texts,
    "anomaly-repushed-with-history": cell_anomaly_repushed_with_history, "anomaly-acknowledged-not-repushed": cell_anomaly_acknowledged_not_repushed, "oversized-budget-acknowledged-after-restart": cell_oversized_budget_acknowledged_after_restart, "runaway-reports-survive-restart": cell_runaway_reports_survive_restart, "runaway-reused-pid-counted-afresh": cell_runaway_reused_pid_counted_afresh, "advance-episodes-survive-restart": cell_advance_episodes_survive_restart, "restart-surviving-tampered-not-adopted": cell_restart_surviving_tampered_not_adopted, "budget-source-names-folded-key": cell_budget_source_names_folded_key, "unclaimed-outcome-repushed": cell_unclaimed_outcome_repushed, "running-progress-pushed": cell_running_progress_pushed,
    "inbox-socket-follows-session": cell_inbox_socket_follows_session, "submit-awaits-confirmation": cell_submit_awaits_confirmation,
    "unconfirmed-cancelled": cell_unconfirmed_cancelled,
    "long-task-by-record": cell_long_task_by_record, "long-task-whole-run": cell_long_task_whole_run, "long-task-sharded": cell_long_task_sharded,
    "long-task-unrecorded-test-or-item": cell_long_task_unrecorded_test_or_item, "short-task-queued-directly": cell_short_task_queued_directly,
    "confirm-on-queued-task-accepted": cell_confirm_on_queued_task_accepted,
    "cleanup-protocol-unit-tests": cell_cleanup_protocol_unit_tests,
    "fills-any-cap-cap-hits-not-reported": cell_fills_any_cap_cap_hits_not_reported,
    "busy-cpu-pinned-at-cap-retried": cell_busy_cpu_pinned_at_cap_retried, "busy-cpu-below-cap-keeps-running": cell_busy_cpu_below_cap_keeps_running,
    "peer-pinned-at-cap-busy-cpu-retried": cell_peer_pinned_at_cap_busy_cpu_retried, "attention-quiet-busy-log": cell_attention_quiet_busy_log,
    "io-wait-local-not-stalled": cell_io_wait_local_not_stalled, "io-wait-peer-not-stalled": cell_io_wait_peer_not_stalled,
    "peer-health-slow-rechecked": cell_peer_health_slow_rechecked, "peak-memory-stat-in-note": cell_peak_memory_stat_in_note,
    "peer-peak-memory-stat-in-note": cell_peer_peak_memory_stat_in_note,
    "loop-leak-trend-sentences": cell_loop_leak_trend_sentences,
    "pinned-cap-over-limit-reported": cell_pinned_cap_over_limit_reported, "pinned-squeezed-requeued": cell_pinned_squeezed_requeued,
    "pin-cap-help-not-pinned": cell_pin_cap_help_not_pinned, "scene-sessionless-swept": cell_scene_sessionless_swept,
    "loop-leak-check-on-bounce": cell_loop_leak_check_on_bounce, "repeated-cap-hits-reminded-once": cell_repeated_cap_hits_reminded_once,
    "long-run-reminded-not-stopped": cell_long_run_reminded_not_stopped, "long-run-reminder-across-cap-hits": cell_long_run_reminder_across_cap_hits,
    "long-run-full-gate-not-reminded": cell_long_run_full_gate_not_reminded, "long-run-check-not-exempt": cell_long_run_check_not_exempt,
    "long-run-merge-and-verify-not-exempt": cell_long_run_merge_and_verify_not_exempt,
    "long-run-confirm-silences": cell_long_run_confirm_silences, "long-run-confirm-refused": cell_long_run_confirm_refused,
    "long-run-sessionless-noted": cell_long_run_sessionless_noted,
    "whole-package-unconfirmed-refused": cell_whole_package_unconfirmed_refused, "whole-package-named-accepted": cell_whole_package_named_accepted,
    "whole-package-confirmation-form": cell_whole_package_confirmation_form, "whole-package-confirmed-still-reminded": cell_whole_package_confirmed_still_reminded,
    "whole-workspace-cargo-refused": cell_whole_workspace_cargo_refused,
    "copy-proof-build-refused": cell_copy_proof_build_refused, "copy-proof-start-refused": cell_copy_proof_start_refused,
    "progress-stall-reported": cell_progress_stall_reported,
    "peer-retry-build-directory-reused": cell_peer_retry_build_directory_reused,
    "repository-target-bypass-warned": cell_repository_target_bypass_warned, "cancelled-artifacts-removed": cell_cancelled_artifacts_removed,
    "cancelled-before-peer-trip-built": cell_cancelled_before_peer_trip_built, "inbox-outcomes-resent-after-error": cell_inbox_outcomes_resent_after_error,
    "thirty-second-tick-prompt": cell_thirty_second_tick_prompt, "starved-head-reported": cell_starved_head_reported,
    "blocked-head-not-starved": cell_blocked_head_not_starved, "starved-head-judged-by-the-formula": cell_starved_head_judged_by_the_formula,
    "aged-task-judged-first": cell_aged_task_judged_first,
    "unmeasured-not-zero": cell_unmeasured_not_zero, "retry-goes-where-it-fits": cell_retry_goes_where_it_fits,
    "peer-runs-in-shared-tree": cell_peer_runs_in_shared_tree, "clean-failure-explained": cell_clean_failure_explained, "stalled-attempt-stopped": cell_stalled_attempt_stopped, "cpu-unread-not-stalled": cell_cpu_unread_not_stalled,
    "running-status-shows-progress": cell_running_status_shows_progress, "stall-waits-for-grace": cell_stall_waits_for_grace, "computing-quietly-not-stalled": cell_computing_quietly_not_stalled, "peer-stall-judged-by-peer-cpu": cell_peer_stall_judged_by_peer_cpu, "three-duties-two-sessions": cell_three_duties_two_sessions,
    "hundred-five-minute-tasks-balanced": cell_hundred_five_minute_tasks_balanced, "stalled-at-cap-retried": cell_stalled_at_cap_retried, "aged-head-at-slice-total-starts": cell_aged_head_at_slice_total_starts, "peer-run-failed-retried": cell_peer_run_failed_retried, "squeezed-at-total-is-cap-hit": cell_squeezed_at_total_is_cap_hit, "peer-starts-throttled": cell_peer_starts_throttled, "attempt-unit-nofile-raised": cell_attempt_unit_nofile_raised, "queued-shows-waiting-reason": cell_queued_shows_waiting_reason,
    "unfinished-peak-ignored": cell_unfinished_peak_ignored, "completed-after-cap-hit": cell_completed_after_cap_hit,
    "cap-hit-starts-above": cell_cap_hit_starts_above,
    "balanced-when-both-fit": cell_balanced_when_both_fit, "peer-when-local-full": cell_peer_when_local_full, "shard-to-peer-by-plan": cell_shard_to_peer_by_plan,
    "shard-siblings-share-a-host": cell_shard_siblings_share_a_host,
    "shard-cap-fitted-to-room": cell_shard_cap_fitted_to_room, "shard-keys-kept-apart": cell_shard_keys_kept_apart,
    "socket-path-too-long": cell_socket_path_too_long, "threads-fixed": cell_threads_fixed,
    "shard-retry-isolated": cell_shard_retry_isolated,
    "task-log": cell_task_log, "cancel-queued": cell_cancel_queued, "cancel-finished": cell_cancel_finished,
    "cancel-sharded-in-plan": cell_cancel_sharded_in_plan, "cancel-sharded-running": cell_cancel_sharded_running,
    "shard-plan-failed": cell_shard_plan_failed, "shard-merge-failed": cell_shard_merge_failed, "unsharded-arguments": cell_unsharded_arguments,
    "control-start": cell_control_start, "gpu-service-required-when-gpu-on": cell_gpu_service_required_when_gpu_on,
    "gpu-service-untouched-when-gpu-off": cell_gpu_service_untouched_when_gpu_off, "pinned-peer-unavailable": cell_pinned_peer_unavailable,
    "stall-clock-from-log-mtime": cell_stall_clock_from_log_mtime,
    "memory-cap-passed-to-command": cell_memory_cap_passed_to_command, "source-repository-passed-to-command": cell_source_repository_passed_to_command,
    "shared-roots-passed-to-command": cell_shared_roots_passed_to_command,
    "peer-admission-marked": cell_peer_admission_marked, "peer-disk-floor": cell_peer_disk_floor, "suspected-stall-notified": cell_suspected_stall_notified,
    "pinned-cap-from-configuration": cell_pinned_cap_from_configuration,
    "pinned-cap-unreadable-refused": cell_pinned_cap_unreadable_refused,
    "pinned-cap-by-host": cell_pinned_cap_by_host, "by-host-missing-host-refused": cell_by_host_missing_host_refused,
    "by-host-outside-address-refused": cell_by_host_outside_address_refused, "shard-host-is-address": cell_shard_host_is_address,
    "gate-stages-sent-to-peer": cell_gate_stages_sent_to_peer,
    "admission-booked-at-cap": cell_admission_booked_at_cap, "health-answers-while-resources-slow": cell_health_answers_while_resources_slow,
    "leftover-processes-stopped": cell_leftover_processes_stopped, "leftovers-only-in-own-unit": cell_leftovers_only_in_own_unit, "long-run-failure-noted": cell_long_run_failure_noted,
    "leftovers-error-isolated": cell_leftovers_error_isolated, "inbox-shard-attention-merged": cell_inbox_shard_attention_merged,
    "attempt-label-passed-to-command": cell_attempt_label_passed_to_command,
    "whole-starts-beside-running-fragment": cell_whole_starts_beside_running_fragment,
    "ended-attempt-settled-after-restart": cell_ended_attempt_settled_after_restart,
    "cpu-gate-holds-busy-host": cell_cpu_gate_holds_busy_host, "unstarted-slice-uses-configured-total": cell_unstarted_slice_uses_configured_total, "down-host-waits-not-never": cell_down_host_waits_not_never,
    "environment-prefix-stripped-from-key": cell_environment_prefix_stripped_from_key,
    "verdict-library-family-refused": cell_verdict_library_family_refused, "workspace-writer-not-retried": cell_workspace_writer_not_retried,
    "wrapped-writer-shell-string-not-retried": cell_wrapped_writer_shell_string_not_retried,
    "wrapped-writer-script-body-not-retried": cell_wrapped_writer_script_body_not_retried,
    "wrapped-writer-read-only-retried": cell_wrapped_writer_read_only_retried,
    "retry-cap-not-floored": cell_retry_cap_not_floored, "recorded-cap-not-floored": cell_recorded_cap_not_floored,
    "cap-hit-low-peak-not-recorded-at-cap": cell_cap_hit_low_peak_not_recorded_at_cap, "runaway-process-notified": cell_runaway_process_notified, "runaway-process-stopped-after-reports": cell_runaway_process_stopped_after_reports,
    "margin-measured-per-host": cell_margin_measured_per_host,
    "margin-unmeasured-not-placed": cell_margin_unmeasured_not_placed, "pin-cap-refused-within-margin": cell_pin_cap_refused_within_margin,
    "peer-booked-like-local": cell_peer_booked_like_local, "peer-scope-missing-booked-at-cap": cell_peer_scope_missing_booked_at_cap,
    "two-hosts-booked-within-capacity": cell_two_hosts_booked_within_capacity, "judge-error-skips-only-that-task": cell_judge_error_skips_only_that_task,
    "nfs-share-required": cell_nfs_share_required, "nfs-share-verified": cell_nfs_share_verified,
    "peer-unavailable-alerted": cell_peer_unavailable_alerted, "reason-line-per-type": cell_reason_line_per_type,
    "selftest-variables-refused": cell_selftest_variables_refused, "type-table-covered": cell_type_table_covered,
    "cancel-start-race": cell_cancel_start_race, "sharded-progress-recovered": cell_sharded_progress_recovered,
    "attempt-claim-atomic": cell_attempt_claim_atomic, "sharded-cancel-retried": cell_sharded_cancel_retried,
    "association-column-verified": cell_association_column_verified, "start-timeout-read-back": cell_start_timeout_read_back,
    "build-disk-predicted": cell_build_disk_predicted, "peer-command-exit-split": cell_peer_command_exit_split,
    "nfs-automount-recognized": cell_nfs_automount_recognized, "peer-memory-peak-recorded": cell_peer_memory_peak_recorded,
    "prune-server-unavailable-outcome": cell_prune_server_unavailable_outcome,
    "temporary-service-wrapper-unwrapped": cell_temporary_service_wrapper_unwrapped,
    "temporary-service-heavy-refused": cell_temporary_service_heavy_refused,
    "data-root-key-required": cell_data_root_key_required,
    "outside-shared-path-refused": cell_outside_shared_path_refused, "path-check-whole-words": cell_path_check_whole_words,
    "shared-roots-outside-refused": cell_shared_roots_outside_refused,
    "nfs-probe-concurrent-runs": cell_nfs_probe_concurrent_runs,
    "short-command-peak-recorded": cell_short_command_peak_recorded, "merge-on-peer-reads-shard-outputs": cell_merge_on_peer_reads_shard_outputs,
    "shard-outputs-written-by-two-drivers": cell_shard_outputs_written_by_two_drivers,
    "stop-ends-probe-children": cell_stop_ends_probe_children, "failure-scene-kept-until-collected": cell_failure_scene_kept_until_collected,
    "shard-failure-waits-for-running": cell_shard_failure_waits_for_running, "shard-cancel-alone-merges": cell_shard_cancel_alone_merges,
    "parent-cancel-stops-every-shard": cell_parent_cancel_stops_every_shard, "cancel-after-natural-end": cell_cancel_after_natural_end,
    "failed-start-not-counted": cell_failed_start_not_counted, "child-rows-verified": cell_child_rows_verified, "task-locks-released": cell_task_locks_released,
    "attempt-row-not-overwritten": cell_attempt_row_not_overwritten, "parent-cancel-waits-for-children": cell_parent_cancel_waits_for_children,
    "recheck-judges-disk": cell_recheck_judges_disk,
    "recheck-judges-nfs": cell_recheck_judges_nfs,
    "stall-stop-spares-next-attempt": cell_stall_stop_spares_next_attempt, "peer-cleanup-recovered-once": cell_peer_cleanup_recovered_once,
    "tampered-attempt-ended-once": cell_tampered_attempt_ended_once, "tampered-attempt-unit-stopped": cell_tampered_attempt_unit_stopped,
    "manual-submit-path": cell_manual_submit_path, "full-gate-flag": cell_full_gate_flag,
    "commit-gate-query-forms-accepted": cell_commit_gate_query_forms_accepted,
    "running-attempts-verified-only": cell_running_attempts_verified_only,
    "concurrent-confirms-serialized": cell_concurrent_confirms_serialized,
    "resident-entries-forgotten": cell_resident_entries_forgotten,
    "whole-package-single-cells-refused": cell_whole_package_single_cells_refused, "unscoped-runs-refused": cell_unscoped_runs_refused,
    "whole-package-feature-items-accepted": cell_whole_package_feature_items_accepted,
    "whole-package-feature-items-others-refused": cell_whole_package_feature_items_others_refused,
    "attempt-gets-no-card": cell_attempt_gets_no_card, "card-declarations-refused": cell_card_declarations_refused,
    "queued-whole-package-rechecked": cell_queued_whole_package_rechecked,
    "unnamed-merge-unnamed-refused": cell_unnamed_merge_unnamed_refused, "unnamed-merge-named-accepted": cell_unnamed_merge_named_accepted,
    "unnamed-cargo-target-refused": cell_unnamed_cargo_target_refused, "unnamed-cargo-exact-accepted": cell_unnamed_cargo_exact_accepted,
    "unnamed-prove-red-unnamed-refused": cell_unnamed_prove_red_unnamed_refused, "unnamed-prove-red-named-accepted": cell_unnamed_prove_red_named_accepted,
    "unnamed-exemptions-accepted": cell_unnamed_exemptions_accepted, "unnamed-gate-cells-judged": cell_unnamed_gate_cells_judged,
    "unnamed-crash-gate-whole-items-refused": cell_unnamed_crash_gate_whole_items_refused, "unnamed-crash-gate-cases-only-accepted": cell_unnamed_crash_gate_cases_only_accepted,
    "test-starting-cells-listed": cell_test_starting_cells_listed,
    "run-hard-limit-arithmetic": cell_run_hard_limit_arithmetic,
    "shard-hard-limit-by-each-piece": cell_shard_hard_limit_by_each_piece,
    "run-hard-limit-stopped": cell_run_hard_limit_stopped,
    "run-hard-limit-why-still-stops": cell_run_hard_limit_why_still_stops,
    "run-hard-limit-full-gate-not-stopped": cell_run_hard_limit_full_gate_not_stopped,
    "run-hard-limit-cap-hit-resets": cell_run_hard_limit_cap_hit_resets,
    "run-hard-limit-writer-stopped": cell_run_hard_limit_writer_stopped,
    "run-hard-limit-shard-stopped": cell_run_hard_limit_shard_stopped,
    "run-hard-limit-applies-after-restart": cell_run_hard_limit_applies_after_restart,
    "run-hard-limit-stop-retried": cell_run_hard_limit_stop_retried,
    "confirmation-scope-text": cell_confirmation_scope_text,
    "selftest-units-stop-with-outer": cell_selftest_units_stop_with_outer,
    "running-booked-unscoped-at-cap": cell_running_booked_unscoped_at_cap,
    "shard-key-without-count": cell_shard_key_without_count,
    "stopped-peaks-floor-record": cell_stopped_peaks_floor_record, "compiles-declared-entry": cell_compiles_declared_entry,
    "parent-cancelling-child-not-claimed": cell_parent_cancelling_child_not_claimed, "cpu-gate-counts-running-threads": cell_cpu_gate_counts_running_threads,
    "cpu-gate-measured-cores": cell_cpu_gate_measured_cores,
    "lookup-indexes-used": cell_lookup_indexes_used, "running-attempts-read-once": cell_running_attempts_read_once,
    "budget-source-only-when-reported": cell_budget_source_only_when_reported, "repass-cleared-when-idle": cell_repass_cleared_when_idle,
    "peer-probe-off-scheduler": cell_peer_probe_off_scheduler,
    "peer-cleanup-off-scheduler": cell_peer_cleanup_off_scheduler,
    "unit-unreadable-exit-file-settles": cell_unit_unreadable_exit_file_settles,
    "unit-unreadable-gone-recorded-lost": cell_unit_unreadable_gone_recorded_lost, "unit-unreadable-running-reminded": cell_unit_unreadable_running_reminded,
    "unit-unreadable-unverified-health-reported": cell_unit_unreadable_unverified_health_reported,
    "running-booked-at-measured-usage": cell_running_booked_at_measured_usage, "running-young-booked-at-cap": cell_running_young_booked_at_cap, "cpu-gate-unrecorded-by-registered-threads": cell_cpu_gate_unrecorded_by_registered_threads, "cpu-gate-running-by-latest-reading": cell_cpu_gate_running_by_latest_reading, "cpu-gate-running-after-restart-by-average": cell_cpu_gate_running_after_restart_by_average, "submit-trace-logged": cell_submit_trace_logged, "cargo-test-key-by-package": cell_cargo_test_key_by_package, "selection-key-by-script": cell_selection_key_by_script,
    "entry-script-key-by-repository-path": cell_entry_script_key_by_repository_path,
    "memory-growth-not-stalled": cell_memory_growth_not_stalled,
    "cpu-gate-new-task-by-recorded-cores": cell_cpu_gate_new_task_by_recorded_cores,
    "cpu-gate-running-started-by-recorded-cores": cell_cpu_gate_running_started_by_recorded_cores,
    "shard-peak-record-count-matched": cell_shard_peak_record_count_matched, "heartbeat-lines-not-log-growth": cell_heartbeat_lines_not_log_growth,
    "signal-exit-named": cell_signal_exit_named,
    "finish-peak-segment-cpu-unread": cell_finish_peak_segment_cpu_unread,
    "fills-any-cap-record-caps-not-reported": cell_fills_any_cap_record_caps_not_reported,
    "unit-stopped-booking-released": cell_unit_stopped_booking_released,
    "cpu-reason-splits-io-wait": cell_cpu_reason_splits_io_wait,
    "peer-health-slow-within-recheck-period": cell_peer_health_slow_within_recheck_period,
    "control-check-probe-bounded": cell_control_check_probe_bounded,
    "control-check-reads-service-nfs-report": cell_control_check_reads_service_nfs_report,
    "promote-skips-cpu-gate": cell_promote_skips_cpu_gate,
    "promote-queue-order": cell_promote_queue_order,
    "progress-line-names-progress": cell_progress_line_names_progress,
    "attempt-progress-readings": cell_attempt_progress_readings,
    "quiet-log-busy-spared-by-progress": cell_quiet_log_busy_spared_by_progress,
    "peer-ssh-multiplexed": cell_peer_ssh_multiplexed,
    "shard-start-borrows-nothing": cell_shard_start_borrows_nothing,
    "plan-key-by-compiles": cell_plan_key_by_compiles,
    "booking-drift-reported": cell_booking_drift_reported,
    "booking-audit-young-not-drift": cell_booking_audit_young_not_drift,
    "cpu-gate-running-average-capped-by-registered": cell_cpu_gate_running_average_capped_by_registered,
    "reminders-not-resent-after-restart": cell_reminders_not_resent_after_restart,
    "reminder-thresholds-every-doubling": cell_reminder_thresholds_every_doubling,
    "reminder-ledger-other-shape-read-afresh": cell_reminder_ledger_other_shape_read_afresh,
    "long-run-split-reminded-at-outcome": cell_long_run_split_reminded_at_outcome,
    "workspace-write-blocked-reminded": cell_workspace_write_blocked_reminded,
    "previous-swap-running-reminded": cell_previous_swap_running_reminded,
    "wrapped-hook-swap-reminded": cell_wrapped_hook_swap_reminded,
    "wrapped-previous-swap-named": cell_wrapped_previous_swap_named,
    "wrapped-writer-interrupted-judged": cell_wrapped_writer_interrupted_judged,
    "wrapped-writers-each-judged": cell_wrapped_writers_each_judged,
    "wrapped-patch-directory-resolved": cell_wrapped_patch_directory_resolved,
    "wrapped-patch-directory-unresolvable": cell_wrapped_patch_directory_unresolvable,
    "hook-swap-outcome-judged": cell_hook_swap_outcome_judged,
    "wrapped-read-only-hook-swap-reminded": cell_wrapped_read_only_hook_swap_reminded,
    "attention-line-why-clipped": cell_attention_line_why_clipped,
    "last-run-seconds-from-records": cell_last_run_seconds_from_records,
    "cpu-gate-small-task-slots": cell_cpu_gate_small_task_slots,
    "io-pressure-gate-by-host": cell_io_pressure_gate_by_host,
    "io-pressure-reading-recorded": cell_io_pressure_reading_recorded,
    "io-pressure-queues-compiling-task": cell_io_pressure_queues_compiling_task,
    "gpu-cases-not-run-reported": cell_gpu_cases_not_run_reported,
    "gpu-not-run-line-forms": cell_gpu_not_run_line_forms,
    "gpu-not-run-marks-per-line": cell_gpu_not_run_marks_per_line,
    "inbox-headlines-tell-kind-state-round": cell_inbox_headlines_tell_kind_state_round,
    "external-kill-peer-line-recorded-and-retried": cell_external_kill_peer_line_recorded_and_retried,
    "confirm-sheet-reads-bash-c-inner-commands": cell_confirm_sheet_reads_bash_c_inner_commands,
    "workspace-writer-cap-has-headroom": cell_workspace_writer_cap_has_headroom,
    "control-key-stats-reported": cell_control_key_stats_reported,
    "control-key-stats-by-task": cell_control_key_stats_by_task,
    "control-key-stats-dump": cell_control_key_stats_dump,
    "machine-memory-line-holds-dispatch": cell_machine_memory_line_holds_dispatch,
    "external-kill-recorded-and-retried": cell_external_kill_recorded_and_retried,
    "booking-ledger-rows": cell_booking_ledger_rows,
    "admission-by-measured-bookings": cell_admission_by_measured_bookings,
    "aged-blocked-task-blocks-later": cell_aged_blocked_task_blocks_later, "jump-only-shorter-than-running": cell_jump_only_shorter_than_running,
    "jump-only-same-resource": cell_jump_only_same_resource,
    "booking-beyond-idle-machine-not-blocking": cell_idle_short_not_blocking,
    "memory-wait-names-both-numbers": cell_memory_wait_names_both_numbers,
    "cpu-gate-new-task-counts-failed-runs": cell_cpu_gate_new_task_counts_failed_runs,
    "confirm-retries-follow-churn": cell_confirm_retries_follow_churn,
    "cpu-gate-peak-segment-cores": cell_cpu_gate_peak_segment_cores,
    "cpu-gate-line-twice-cores": cell_cpu_gate_line_twice_cores,
    "cpu-gate-runnable-threads": cell_cpu_gate_runnable_threads,
    "threads-over-registered-reported": cell_threads_over_registered_reported,
    "threads-over-registered-peer-reported": cell_threads_over_registered_peer_reported,
    "shard-binaries-removed-at-outcome": cell_shard_binaries_removed_at_outcome,
    "runner-threads-file-counts": cell_runner_threads_file_counts,
    "store-write-rolls-back-on-error": cell_store_write_rolls_back_on_error,
    "disk-floor-peer-not-never-fits": cell_disk_floor_peer_not_never_fits,
    "stall-reads-log-before-cpu": cell_stall_reads_log_before_cpu,
    "peer-readings-merged-across-reads": cell_peer_readings_merged_across_reads,
    "log-content-serialized": cell_log_content_serialized,
    "log-noise-marks-not-growth": cell_log_noise_marks_not_growth,
    "runaway-targets-skip-children": cell_runaway_targets_skip_children,
    "promote-wakes-and-promotes-shards": cell_promote_wakes_and_promotes_shards,
    "promote-placement-rules": cell_promote_placement_rules,
    "pause-keeps-queue-positions": cell_pause_keeps_queue_positions,
    "stop-joins-peer-probe": cell_stop_joins_peer_probe,
    "peer-cooldown-skips-connect": cell_peer_cooldown_skips_connect,
    "build-slot-reused-on-same-tree": cell_build_slot_reused_on_same_tree, "build-slot-separate-per-tree": cell_build_slot_separate_per_tree,
    "build-slot-concurrent-distinct": cell_build_slot_concurrent_distinct, "build-pool-removed-with-tree": cell_build_pool_removed_with_tree,
    "unpooled-label-directories-removed": cell_unpooled_label_directories_removed,
    "unpooled-label-directory-of-running-attempt-kept": cell_unpooled_label_directory_of_running_attempt_kept,
    "clean-keeps-build-pool": cell_clean_keeps_build_pool, "build-disk-credits-free-slot": cell_build_disk_credits_free_slot,
    "command-word-index-skips-option-values": cell_command_word_index_skips_option_values,
    "runner-start-error-recorded": cell_runner_start_error_recorded,
    "runner-final-sample-after-exit": cell_runner_final_sample_after_exit,
    "selection-key-positionals-after-options": cell_selection_key_positionals_after_options, "plan-key-without-shrunk-domain": cell_plan_key_without_shrunk_domain,
    "memory-window-head-read-at-open": cell_memory_window_head_read_at_open, "verdict-library-progress-not-stalled": cell_verdict_library_progress_not_stalled,
    "long-run-confirmed-covers-shards": cell_long_run_confirmed_covers_shards,
}
UNPREPARED_CELLS = ("install",)


def run_cell(name, directory):
    if not lib.in_selftest():
        print("  ✗ --selftest-cell 只由 task_server_lib.py --selftest 起（它建自证的标记）", file=sys.stderr)
        print("  → 怎么办：跑 python3 task-server/task_server_lib.py --selftest；只跑几格设 TASK_SERVER_SELFTEST_CELLS=<格名,…>", file=sys.stderr)
        return lib.USAGE_EXIT
    if name not in CELLS:
        print(f"  ✗ 没有这一格：{name}（有 {'、'.join(CELLS)}）", file=sys.stderr)
        print("  → 怎么办：照上面的格名写", file=sys.stderr)
        return lib.USAGE_EXIT
    os.makedirs(directory, exist_ok=True)
    world = World(name, directory, prepare=name not in UNPREPARED_CELLS)
    try:
        detail = CELLS[name](world)
        print(f"  · 对 {name}：{detail}")
        return 0
    except SelftestFailure as failure:
        print(f"  {FAILURE_PREFIX}{name}：{failure}")   # gate-lint:detail
        for line in failure.extra_lines:
            for piece in str(line).splitlines()[-12:]:
                print(f"       | {piece}")
        return 1
    except Exception as error:   # 格子自己出事也照这一格不对报，不吞
        print(f"  {FAILURE_PREFIX}{name}：格子出了异常（{type(error).__name__}: {error}）")   # gate-lint:detail
        for piece in traceback.format_exc().splitlines()[-12:]:
            print(f"       | {piece}")
        return 1
    finally:
        world.close()


def short_temporary_base():
    """各格 socket 目录的公共上一层建在哪（task_server_lib.py 文件头「自证」）：系统临时目录接上最长那一条 socket 路径放得下就用它，放不下用 /tmp。"""
    base = tempfile.gettempdir()
    if len(os.fsencode(base)) + SOCKET_PATH_ALLOWANCE <= lib.SOCKET_PATH_MAXIMUM_BYTES:
        return base
    return FALLBACK_TEMPORARY_BASE


def parallelism(environment=None, registration_file=None):
    """各格并行跑的路数（run_all）：每台默认重活线程数的登记值，scripts/hosts/heavy_threads.py 的 default_heavy_threads 现读（SINGLEFS_THREAD_CAP 设了照它）。
    → (路数, 取不出的原因, 出路)。取不出登记值（登记文件不在、读不了、写坏，SINGLEFS_THREAD_CAP 不是正整数，heavy_threads.py 载不进来）交 (None, 原因, 出路)，
    由 run_all 判红，不退回按核数（.claude/heavy-thread-default 文件头：写坏或删掉这份文件，取法一律拒绝，不退回按核数算）。
    environment、registration_file 只给 parallelism-registration-refused 这一格造样本用，不给就是此刻的环境与仓里的登记文件。
    弄坏开关 parallelism-falls-back-to-cores 下取不出就按核数（改前的样子）。"""
    environment = os.environ if environment is None else environment
    try:
        module = lib.load_module("heavy_threads", os.path.join(lib.HOSTS_DIRECTORY, "heavy_threads.py"))
        return max(1, int(module.default_heavy_threads(environment, registration_file=registration_file))), "", ""
    except Exception as error:
        if lib.broken("parallelism-falls-back-to-cores"):
            return max(1, os.cpu_count() or 1), "", ""
        return None, str(error) if hasattr(error, "howto") else f"{type(error).__name__}: {error}", getattr(error, "howto", "") or "看 scripts/hosts/heavy_threads.py 与 .claude/heavy-thread-default 的文件头"


def run_all(arguments):
    # 套在假调度器上的格（fake.environment 拷的是本进程环境）同样照小世界的两个数：CPU 闸的倍数 2、刚起跑照上限记那一条不看
    os.environ.setdefault(lib.CPU_OVERSUBSCRIPTION_VARIABLE, "2")
    os.environ.setdefault(lib.YOUNG_BOOKING_VARIABLE, "0")
    items = [item for item in os.environ.get(lib.BREAK_VARIABLE, "").split(",") if item]
    unknown = [item for item in items if item not in lib.KNOWN_BREAK_ITEMS]
    if unknown:
        print(f"  ✗ {lib.BREAK_VARIABLE} 里认不出的项：{'、'.join(unknown)}")
        print("  → 怎么办：照 task_server_lib.py 文件头的开关表写")
        return lib.USAGE_EXIT
    # 同一格点了两次只跑一次：两份同名的格并行写同一个格目录、同一份 .out 与 .exit，收回时按格名只数得到一份（任务 13267 点了 46 个名、两格重名，收回 44 份）
    selected = list(dict.fromkeys(name for name in os.environ.get("TASK_SERVER_SELFTEST_CELLS", "").split(",") if name)) or list(CELLS)
    wrong_names = [name for name in selected if name not in CELLS]
    if wrong_names:
        print(f"  ✗ TASK_SERVER_SELFTEST_CELLS 里没有的格：{'、'.join(wrong_names)}（有 {'、'.join(CELLS)}）")
        print("  → 怎么办：照上面的格名写，或不设它跑全部格")
        return lib.USAGE_EXIT
    parallel, reason, howto = parallelism()
    if parallel is None:   # 取不出登记的线程数：判红，不退回按核数；还没建临时目录，没有要收的
        print(f"  ✗ task_server_lib.py 自证：并行度取不出每台默认重活线程数（scripts/hosts/heavy_threads.py）：{reason}")
        print(f"  → 怎么办：{howto}")
        return 1
    root = tempfile.mkdtemp(prefix="tss-")
    socket_root = tempfile.mkdtemp(prefix=SOCKET_ROOT_PREFIX, dir=short_temporary_base())
    mark = os.path.join(root, "selftest-mark")
    with open(mark, "w", encoding="utf-8") as handle:
        handle.write(lib.SELFTEST_MARK_TEXT + "\n")
    environment = dict(os.environ, **{lib.SELFTEST_MARK_VARIABLE: mark, SOCKET_ROOT_VARIABLE: socket_root})
    workers = min(parallel, len(selected))
    library = os.path.join(lib.SCRIPT_DIRECTORY, "task_server_lib.py")

    def one(name):
        output_path = os.path.join(root, f"{name}.out")
        with open(output_path, "w", encoding="utf-8") as output:
            try:
                completed = subprocess.run([sys.executable, "-B", library, "--selftest-cell", name, os.path.join(root, name)], env=environment, stdin=subprocess.DEVNULL,
                                           stdout=output, stderr=subprocess.STDOUT, timeout=CELL_TIMEOUT_SECONDS)
                code = completed.returncode
            except subprocess.TimeoutExpired:
                code = None
        with open(os.path.join(root, f"{name}.exit"), "w", encoding="utf-8") as handle:
            handle.write("timeout" if code is None else str(code))
    try:
        with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
            list(pool.map(one, selected))
        exits = {}
        for name in selected:
            with contextlib.suppress(OSError):
                with open(os.path.join(root, f"{name}.exit"), encoding="utf-8") as handle:
                    exits[name] = handle.read().strip()
        failures = []
        for name in selected:
            with contextlib.suppress(OSError):
                with open(os.path.join(root, f"{name}.out"), encoding="utf-8", errors="replace") as handle:
                    sys.stdout.write(handle.read())
            if exits.get(name) != "0":
                failures.append(name)
                if exits.get(name) == "timeout":
                    print(f"  {FAILURE_PREFIX}{name}：这一格 {CELL_TIMEOUT_SECONDS} 秒没跑完")   # gate-lint:detail
        if len(exits) != len(selected):
            print(f"  ✗ task_server_lib.py 自证：派出 {len(selected)} 格、收回 {len(exits)} 格，对不上，整份作废")
            print("  → 怎么办：看上面缺了哪一格的输出；整份重跑一次")
            return 1
        if failures:
            print(f"  ✗ task_server_lib.py 自证 {len(selected)} 格里 {len(failures)} 格不对（逐格在上面）")   # gate-lint:summary
            print("  → 怎么办：按不对的那一格修服务、客户端或 control；设着 TASK_SERVER_BREAK 的话点名的格本来就该红（开关表在 task_server_lib.py 文件头）")
            return 1
        print(f"  ✓ task_server_lib.py 自证 {len(selected)} 格都对（{lib.SELFTEST_PARALLELISM_TEXT.format(workers=workers)}，每格一份真的 uvicorn 挂在临时 socket 上；各格讲什么见上面逐格那一行）")
        return 0
    finally:
        shutil.rmtree(socket_root, ignore_errors=True)
        shutil.rmtree(root, ignore_errors=True)
