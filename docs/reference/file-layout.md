# File Layout

Boundline relies on specific directories and files to manage the AI session lifecycle and store configuration safely. By default, these files are kept inside the `.boundline` directory at the root of your workspace.

## Core Directories

- `.boundline/`: The root directory for Boundline's local state and configuration. It is recommended to add `.boundline/traces` and `.boundline/session.json` to `.gitignore`.
- `.boundline/traces/`: Stores trace files generated during execution, tracking all operations, validations, and decisions made.
- `.boundline/checkpoints/`: Holds rollback manifests and state copies created before mutative operations, allowing seamless recovery in case of failure.
- `.boundline/context-intelligence/`: Stores the derived retrieval index, companion manifest, repository-map state, and freshness-bound snapshot-cache artifacts used by local context selection.

## Configuration & State Files

- `.boundline/session.json`: The active session state file. Boundline maintains context and pointers for the ongoing delivery iteration here.
- `.boundline/config.toml`: Workspace configuration. Contains overriding rules for guardians, guidance configurations, and routing settings.
- `.boundline/context-intelligence/manifest.json`: The derived semantic-index and substrate manifest. It is disposable runtime state, not reviewed memory.
