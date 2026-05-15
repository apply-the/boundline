# Security Boundary Guardian

Review whether the bounded change preserved input validation, authorization, secret handling, and safe error disclosure at its boundaries.

- Escalate changes that accept new external input without validation or schema tightening.
- Escalate changes that move authorization checks away from the use-case boundary.
- Escalate unsafe error disclosure, raw rendering, unsafe file handling, or secret exposure.
- Keep findings tied to the changed boundary so remediation is concrete.
