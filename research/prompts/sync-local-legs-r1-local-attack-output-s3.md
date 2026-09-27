1.  
Row 1: Seconds: 900, Milliseconds: 900000  
Row 2: Seconds: 600, Milliseconds: 600000  
Row 3: Seconds: 240, Milliseconds: 240000  
Row 4: Positive  
Row 5: Positive  
Row 6: Positive  

The positive Row 4 difference confirms the default ASK_LOCAL_TIMEOUT (900s) exceeds the Bash tool’s foreground cap (600s), meaning a 900-second call would be terminated by the tool before completing. Row 5 shows the default exceeds the stricter agent-common.md ceiling (240s), making termination even more certain. Row 6 reveals the Bash tool’s cap (600s) is higher than agent-common.md’s limit (240s), so the tighter rule governs. This would be wrong if any row difference was negative or zero.  

2.  
No. The leg cannot compute in advance whether the call will finish inside the foreground cap because the actual runtime depends on unpredictable network conditions and server response times, which are not provided by any fact. What would be sufficient is a guaranteed fixed runtime (e.g., "this request always completes in ≤300 seconds") or explicit historical data about the specific call’s duration. This would be wrong if the actual runtime could be reliably predicted from the given facts.  

3.  

| Fact | Requirement for ask-local.sh | Condition for foreground/background change |  
|------|-----------------------------|--------------------------------------------|  
| C    | Foreground by default, background fallback if ASK_LOCAL_TIMEOUT > 600s | ASK_LOCAL_TIMEOUT value > 600s |  
| D    | Background unconditionally | None |  
| E1   | Background only if classified as long-lived (unspecified for ask-local.sh) | Classification as long-lived (not defined) |  

| Pair       | 300s expected runtime | 700s expected runtime | Same answers? |  
|------------|-----------------------|-----------------------|---------------|  
| C vs D     | Same (background)     | Same (background)     | Yes           |  
| C vs E1    | Same (background)     | Same (background)     | Yes           |  
| D vs E1    | Same (background)     | Same (background)     | Yes           |  

This would be wrong if, for example, Fact E1 required foreground execution for ask-local.sh regardless of runtime.  

4.  
Sub-case (i): Range >240s to 900s. The destination file is partially written (since output is being streamed during execution), and the exit code is non-zero (indic 124 for timeout termination).  
Sub-case (ii): Range >600s to 900s. The destination file is partially written, and the exit code is non-zero.  
Derived from facts: Timeout values (E3’s 240s, B’s 600s) and curl’s 900s limit (Fact A). Inferred: File state and exit code behavior (shell redirection semantics and timeout handling). This would be wrong if the file was empty or fully written when killed, or the exit code was zero.
