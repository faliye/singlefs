1.  
Row 1: Seconds: 900, Milliseconds: 900000  
Row 2: Seconds: 600, Milliseconds: 600000  
Row 3: Seconds: 240, Milliseconds: 240000  
Row 4: Positive  
Row 5: Positive  
Row 6: Positive  
The positive sign in Row 4 means the default 900-second timeout exceeds the Bash tool’s 600-second foreground cap, so a 900-second call cannot finish within that cap. Row 5’s positive sign shows 900 seconds exceeds the agent’s stricter 240-second ceiling, making it impossible for the call to finish within that limit. Row 6’s positive sign confirms the Bash tool’s cap (600s) is higher than the agent’s ceiling (240s), meaning the agent’s rule imposes a tighter restriction. This would be wrong if any of Row 4, Row 5, or Row 6 had a negative or zero sign, indicating the default timeout was not longer than the respective cap.  

2.  
No. The leg cannot compute in advance whether the call will finish within the foreground cap because the actual network request duration is unknown until runtime; only the maximum timeout (ASK_LOCAL_TIMEOUT) is known, not the actual time taken. This would require knowing the precise network response time beforehand, which is impossible without external data (e.g., historical averages or real-time monitoring). This would be wrong if the actual duration of the request was predictable from available facts (e.g., if the script included a fixed known duration or external metrics).  

3.  
| Fact | Requirement | Condition for change |  
|------|-------------|----------------------|  
| C | Foreground by default, background fallback if ASK_LOCAL_TIMEOUT > Bash cap | ASK_LOCAL_TIMEOUT > Bash tool cap (600s) |  
| D | Background unconditionally | None |  
| E1 | Background if long-lived task; otherwise foreground | Whether the task qualifies as "long-lived" (non-exhaustive examples: compiling, mutation tables, replays) |  

| Pair | 300s expected duration | 700s expected duration | Same for both? |  
|------|------------------------|------------------------|----------------|  
| C vs D | Same (background) | Same (background) | Yes |  
| C vs E1 | Different (C: background; E1: foreground if not long-lived) | Same (C: background; E1: background if long-lived) | No |  
| D vs E1 | Different (D: background; E1: foreground if not long-lived) | Same (D: background; E1: background if long-lived) | No |  
This would be wrong if the "long-lived" classification for E1 was explicitly defined in the facts (e.g., "all calls exceeding 300s are long-lived"), making E1’s behavior deterministic and consistent with other facts.  

4.  
Sub-case (i):  
- Range: >240 seconds to 900 seconds (Bash tool kills before curl’s 900s timeout).  
- File status: Partially written (data written before timeout).  
- Exit code: 124 (standard timeout exit code).  
Derived from facts: ASK_LOCAL_TIMEOUT default (900s) and agent-common.md’s 240s ceiling. Inferred: File partial write and exit code 124 (standard shell behavior not specified in facts). This would be wrong if the file was fully written or empty when killed, or if the exit code was not 124.  

Sub-case (ii):  
- Range: >600 seconds to 900 seconds (Bash tool kills before curl’s 900s timeout).  
- File status: Partially written (data written before timeout).  
- Exit code: 124 (standard timeout exit code).  
Derived from facts: ASK_LOCAL_TIMEOUT default (900s) and Bash tool’s 600s cap. Inferred: File partial write and exit code 124 (standard shell behavior not specified in facts). This would be wrong if the file was fully written or empty when killed, or if the exit code was not 124.
