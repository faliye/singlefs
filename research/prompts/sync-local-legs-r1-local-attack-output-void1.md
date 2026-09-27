1.  
Row 1. Seconds: 900. Milliseconds: 900000.  
Row 2. Seconds: 600. Milliseconds: 600000.  
Row 3. Seconds: 240. Milliseconds: 240000.  
Row 4. Positive.  
Row 5. Positive.  
Row 6. Positive.  
The positive sign of Row 4 indicates that the default ASK_LOCAL_TIMEOUT (900 seconds) exceeds the Bash tool's foreground cap (600 seconds), meaning a 900-second call would be terminated by the Bash tool before completing. Row 5's positive sign shows the default exceeds the agent-common ceiling (240 seconds), so the agent-common rule would terminate the call earlier than the Bash tool's cap. Row 6's positive sign confirms the Bash tool's cap is stricter stricter than the agent-common ceiling, but the agent-common rule imposes a tighter limit. This would be wrong if any row's numerical values or signs did not match the facts provided.

2.  
Yes, the leg can compute in advance whether the call will finish within the foreground timeout cap by checking the value of ASK_LOCAL_TIMEOUT against the agent-common.md ceiling (240 seconds, per Fact E3). If ASK_LOCAL_TIMEOUT ≤ 240 seconds, the call will finish within the cap (since the actual runtime cannot exceed ASK_LOCAL_TIMEOUT). If ASK_LOCAL_TIMEOUT > 240 seconds, the Bash tool will kill the command at 240 seconds, so it will not finish normally. This relies on Fact A (to know ASK_LOCAL_TIMEOUT's value) and Fact E3 (the 240-second cap). This would be wrong if ASK_LOCAL_TIMEOUT's value was not accessible before the call or if Fact E3 did not define the cap.

3.  
| Fact | Requirement for ask-local.sh | Checkable condition for foreground/background switch? |  
|------|-----------------------------|------------------------------------------------------|  
| C | Foreground by default with background fallback when ASK_LOCAL_TIMEOUT > 600 seconds | Yes: ASK_LOCAL_TIMEOUT > Bash tool's cap (600 seconds) |  
| D | Background unconditionally | No: always background regardless of conditions |  
| E1 | Background if long-lived task; otherwise foreground | No: "long-lived task" is undefined in E1 (examples are non-exhaustive) |  
| Pair | 300-second call | 700-second call | Same action for both? |  
| C vs D | Different (C: foreground; D: background) | Same (both background) | No |  
| C vs E1 | Different (C: foreground; E1: background if long-lived) | Same (both background if long-lived) | No |  
| D vs E1 | Same (both background if long-lived) | Same (both background if long-lived) | Yes |  

4.  
Sub-case (i): If the request runs between 240 and 900 seconds, the Bash tool terminates the command at 240 seconds. The output file will be partially written (if any data was written before termination) or empty (if termination occurred before writing), and the exit code will indicate a timeout (e.g., 124). This is uses facts for the command structure and timeout parameter (Fact C and E3), but file state and exit code are inferred from shell behavior.  
Sub-case (ii): If the request runs between 600 and 900 seconds, the Bash tool kills it at 600 seconds. The output file will be partially written or empty, and the exit code will indicate a timeout. This uses facts for command structure and timeout parameter (Fact C and B), but file state and exit code are inferred. This would be wrong if the output file was always empty when killed or if the exit code did not reflect a timeout.