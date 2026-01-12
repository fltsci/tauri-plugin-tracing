---
"tracing": minor
"tracing-js": minor
---

Add log rotation support with configurable time-based rotation periods (Daily, Hourly, Minutely, Never) and retention strategies (KeepAll, KeepOne, KeepSome). New builder methods: `with_rotation()` for rotation period, `with_rotation_strategy()` for file retention policy.
