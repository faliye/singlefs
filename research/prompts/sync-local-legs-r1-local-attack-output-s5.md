1.  
Row 1: Seconds: 900, Milliseconds: 900000  
Row 2: Seconds: 600, Milliseconds: 600000  
Row 3: Seconds: 240, Milliseconds: 240000  
Row 4: Positive  
Row 5: Positive  
Row 6: Positive  
Row 4's positive sign shows the default 900-second timeout exceeds the Bash tool's 600-second cap, meaning foreground calls would be terminated by the tool before curl's timeout. Row 5's positive sign confirms the default exceeds the stricter 240-second agent-common cap. Row 6's positive sign indicates the Bash tool's cap is higher than the agent's rule, making the agent's cap the effective limit. This would be wrong if Row 4's sign were negative or zero.  

2.  
No. The actual duration of the network request is unknown, even with knowledge of ASK_LOCAL_TIMEOUT (which only specifies the maximum possible time). The leg cannot determine in advance whether the call will finish within the foreground cap because the request may complete earlier than the maximum but still exceed the cap (e.g., ASK_LOCAL_TIMEOUT=300s, but actual time=250s). This would be wrong if the actual duration of the network request was guaranteed to be known in advance or consistently less than the cap.  

3.  
| Fact | Requirement for ask-local.sh | Checkable condition? |  
|------|-----------------------------|----------------------|  
| C | Foreground by default with background fallback if ASK_LOCAL_TIMEOUT > 600s | Yes (ASK_LOCAL_TIMEOUT > 600s) |  
| D | Background unconditionally | No |  
| E1 | Background only if long-lived task (ask-local.sh status unspecified) | No (condition not checkable for this task) |  

| Pair | 300s action same? | 700s action same? | Answers same for both times? |  
|------|-------------------|-------------------|----------------------------|  
| C vs D | No | Yes | No |  
| C vs E1 | Yes | No | No |  
| D vs E1 | No | No | Yes |  

For C vs D: At 300s (below 600s), C requires foreground while D requires background (different); at 700s, both require background (same). For C vs E1: Assuming ask-local.sh is not long-lived (not in examples), both require foreground at 300s (same), but C requires background at 700s while E1 requires foreground (different). For D vs E1: D always requires background; E1 requires foreground (if not long-lived) for both times, so actions differ in both cases. This would be wrong if D vs E1 required different actions for one time but not the other.  

4.  
Sub-case (i): Range 240 < time ≤ 900 seconds. Destination file is partially written; exit code is 124. Derived from Facts A and E3 (curl timeout and agent-common cap). Inference: Shell terminates the command early, leaving the file in a partially written state (not stated in facts).  
Sub-case (ii): Range 600 < time ≤ 900 seconds. Destination file is partially written; exit code is 124. Derived from Facts A and B (curl timeout and Bash tool cap). Inference: Shell terminates the command early, leaving the file in a partially written state (not stated in facts).  
This would be wrong if the destination file was always empty when terminated by timeout.
