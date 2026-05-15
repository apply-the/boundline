# JVM Language Guidance

Use Java, Kotlin, Scala, and Groovy with explicit dependency flow and immutable domain modeling.

- Prefer constructor injection and immutable records, data classes, or case classes for domain values.
- Separate application orchestration from domain rules and from repository or adapter implementations.
- Use specific exception types or typed results for expected failures, and keep stack traces meaningful.
- Use `Optional` or nullable wrappers only at boundaries where absence is real; do not spread them across the core model blindly.
- Keep framework annotations and persistence concerns out of the core domain when they are not part of the rule itself.
- Make transaction, thread, and resource boundaries explicit instead of relying on hidden container behavior.
