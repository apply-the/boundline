# Boundline Starter Prompts

- I want to turn this idea into a bounded implementation plan.
- Help me fix a failing test with Boundline.
- Continue the active Boundline session.
- Inspect the latest Boundline trace and tell me the next safe action.
- Help me decompose this broad initiative into bounded stages.
- Route this governed stage through `/boundline:govern`.

These prompts must route users into Boundline's real session-native runtime. `.boundline/session.json` remains authoritative, and assistants should preserve CLI-reported `next_command`, blocked, clarification-required, failed, exhausted, and terminal states.
Chat history is not authoritative.
