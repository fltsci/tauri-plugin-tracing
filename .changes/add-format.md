---
"tracing": minor
"tracing-js": minor
---

Add log formatting options via `with_format()` and related methods. Configure output style (`LogFormat::Full`, `Compact`, or `Pretty`) and control what information is displayed: `with_file()`, `with_line_number()`, `with_thread_ids()`, `with_thread_names()`, `with_target_display()`, `with_level()`. Also adds `FormatOptions` struct and `configured_format()`/`configured_format_options()` getters.
