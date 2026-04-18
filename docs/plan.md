# TCPing Go to Rust Refactoring Plan

## Goal Description
Refactor the existing tcping Golang project to Rust while maintaining full feature compatibility, including TCP connectivity testing, DNS resolution, multiple output formats (console, JSON, CSV, SQLite), and comprehensive statistics tracking.

## Acceptance Criteria

Following TDD philosophy, each criterion includes positive and negative tests for deterministic verification.

- AC-1: Rust implementation maintains exact CLI flag compatibility with Go version
  - Positive Tests (expected to PASS):
    - All existing Go flags (-4, -6, -r, -c, -j, -pretty, -no-color, -D, -csv, -v, -u, -i, -t, -db, -I, -show-source-address, -show-failures-only, -h) work identically
    - Both "host port" and "host:port" argument formats are supported
    - Flag validation and error messages match Go implementation
  - Negative Tests (expected to FAIL):
    - Invalid flag combinations produce appropriate error messages
    - Missing required arguments show usage help
    - Invalid port numbers or timeouts are rejected

- AC-2: Core TCP connectivity testing functionality preserved
  - Positive Tests (expected to PASS):
    - TCP connections to valid hosts/ports succeed with RTT measurement
    - Connection timeouts are properly handled
    - IPv4 and IPv6 connectivity work correctly
    - Interface binding (-I flag) functions as expected
  - Negative Tests (expected to FAIL):
    - Connections to unreachable hosts/timeout correctly
    - Invalid IP addresses or hostnames are rejected
    - Interface binding to non-existent interfaces fails appropriately

- AC-3: DNS resolution and hostname retry functionality maintained
  - Positive Tests (expected to PASS):
    - Hostname resolution works for valid domains
    - Automatic retry after failed probes (-r flag) functions correctly
    - IP address changes during runtime are tracked
  - Negative Tests (expected to FAIL):
    - Invalid hostnames produce appropriate DNS errors
    - Retry logic respects the configured retry count

- AC-4: Multiple output format support preserved
  - AC-4.1: Console output with color support
    - Positive: Colorized output works with -no-color flag disabling colors
    - Negative: Color escape sequences don't appear when -no-color is set
  - AC-4.2: JSON output (-j and -pretty flags)
    - Positive: JSON output matches Go format structure
    - Negative: Invalid JSON structures are rejected
  - AC-4.3: CSV output (-csv flag)
    - Positive: CSV files contain all probe data with proper headers
    - Negative: Invalid CSV paths produce appropriate errors
  - AC-4.4: SQLite database output (-db flag)
    - Positive: Database schema matches Go implementation
    - Negative: Database connection failures are handled gracefully

- AC-5: Statistics and metrics tracking maintained
  - Positive Tests (expected to PASS):
    - Uptime/downtime calculations are accurate
    - RTT statistics (min/avg/max) are computed correctly
    - Packet loss percentages are calculated accurately
    - Longest uptime/downtime periods are tracked
  - Negative Tests (expected to FAIL):
    - Statistics calculations handle edge cases (zero probes, all failures, all successes)
    - Invalid statistical data is handled gracefully

- AC-6: Signal handling and graceful shutdown
  - Positive Tests (expected to PASS):
    - SIGINT and SIGTERM trigger graceful shutdown with statistics
    - Statistics are printed when Enter key is pressed
    - Database connections are properly closed on exit
  - Negative Tests (expected to FAIL):
    - Forceful termination doesn't corrupt output files
    - Resource leaks are prevented during shutdown

- AC-7: Cross-platform Linux support with required crates
  - Positive Tests (expected to PASS):
    - Application compiles and runs on Linux with clap, tokio, rusqlite
    - All functionality works on standard Linux distributions
  - Negative Tests (expected to FAIL):
    - Platform-specific features don't break Linux compatibility
    - Crate dependencies resolve correctly on Linux

## Path Boundaries

Path boundaries define the acceptable range of implementation quality and choices.

### Upper Bound (Maximum Acceptable Scope)
The implementation includes all features from the Go version with full test coverage, comprehensive error handling, and optimized performance. This includes exact CLI flag compatibility, all output formats (console, JSON, CSV, SQLite), statistics tracking, signal handling, and cross-platform Linux support using the specified Rust crates (clap, tokio, rusqlite).

### Lower Bound (Minimum Acceptable Scope)
The implementation includes core TCP connectivity testing with basic CLI flag support, console output, and essential statistics. This maintains the fundamental functionality while potentially simplifying some advanced features like database output or complex signal handling, but still satisfies all acceptance criteria for basic operation.

### Allowed Choices
- Can use: Rust standard library patterns, async/await where appropriate, error handling with Result types, struct-based data modeling
- Cannot use: Go-specific patterns, external binaries or shell commands for core functionality, non-Rust dependencies for TCP operations

> **Note on Deterministic Designs**: The draft specifies deterministic choices for crates (clap, tokio, rusqlite) and synchronous operation. The path boundaries reflect these fixed constraints while allowing implementation flexibility within Rust idioms.

## Feasibility Hints and Suggestions

> **Note**: This section is for reference and understanding only. These are conceptual suggestions, not prescriptive requirements.

### Conceptual Approach
1. **Core Architecture**: Create a main struct similar to Go's `tcping` struct that holds configuration, statistics, and printer implementations
2. **CLI Interface**: Use clap for command-line parsing with exact flag matching to Go implementation
3. **TCP Probes**: Implement synchronous TCP connectivity testing using Rust's `std::net` with timeout support
4. **Output System**: Create trait-based printer system for different output formats (console, JSON, CSV, SQLite)
5. **Statistics Tracking**: Maintain identical statistical calculations for RTT, uptime/downtime, packet loss
6. **Signal Handling**: Use tokio for signal handling while keeping core logic synchronous

### Relevant References
- `tcping.go` - Main Go implementation with TCP probing logic and statistics
- `db.go` - SQLite database output implementation
- `csv.go` - CSV file output implementation  
- `statsprinter.go` - Console output formatting and color handling
- Go flag package documentation for exact CLI behavior matching

## Dependencies and Sequence

### Milestones
1. **Milestone 1**: Core TCP connectivity and basic CLI
   - Phase A: Set up Rust project structure with Cargo.toml and required dependencies
   - Phase B: Implement basic TCP probing functionality with timeout support
   - Phase C: Create CLI interface with clap matching Go flag structure

2. **Milestone 2**: Output system implementation
   - Phase A: Implement console output with color support and statistics
   - Phase B: Add JSON output format with pretty printing option
   - Phase C: Implement CSV and SQLite output formats

3. **Milestone 3**: Advanced features and testing
   - Phase A: Add DNS resolution and hostname retry logic
   - Phase B: Implement signal handling and graceful shutdown
   - Phase C: Comprehensive testing and edge case handling

Dependencies: Core TCP functionality (Milestone 1) must be completed before output systems (Milestone 2). Advanced features (Milestone 3) depend on both core functionality and output systems being operational.

## Task Breakdown

Each task must include exactly one routing tag:
- `coding`: implemented by Claude
- `analyze`: executed via Codex (`/humanize:ask-codex`)

| Task ID | Description | Target AC | Tag (`coding`/`analyze`) | Depends On |
|---------|-------------|-----------|----------------------------|------------|
| task1 | Set up Rust project structure with Cargo.toml and dependencies | AC-7 | coding | - |
| task2 | Implement core TCP connectivity testing with timeout support | AC-2 | coding | task1 |
| task3 | Create CLI interface with clap matching exact Go flag structure | AC-1 | coding | task1 |
| task4 | Implement console output with color support and basic statistics | AC-4.1 | coding | task2, task3 |
| task5 | Add JSON output format with pretty printing option | AC-4.2 | coding | task4 |
| task6 | Implement CSV output format for probe data | AC-4.3 | coding | task4 |
| task7 | Implement SQLite database output with matching schema | AC-4.4 | coding | task4 |
| task8 | Add DNS resolution and hostname retry functionality | AC-3 | coding | task2 |
| task9 | Implement signal handling and graceful shutdown | AC-6 | coding | task2 |
| task10 | Add comprehensive statistics tracking (RTT, uptime/downtime) | AC-5 | coding | task4 |
| task11 | Create comprehensive test suite for all functionality | All ACs | coding | task10 |
| task12 | Review Rust implementation patterns and error handling | All ACs | analyze | - |
| task13 | Validate CLI flag compatibility with Go version | AC-1 | analyze | task3 |
| task14 | Review output format compatibility and data integrity | AC-4 | analyze | task5, task6, task7 |

## Claude-Codex Deliberation

### Agreements
- The refactoring must maintain exact CLI flag compatibility with the Go version
- Core TCP connectivity testing functionality must be preserved
- All output formats (console, JSON, CSV, SQLite) must be supported
- The specified Rust crates (clap, tokio, rusqlite) should be used
- Synchronous operation should be maintained for simplicity as specified

### Resolved Disagreements
- N/A - Codex analysis failed to complete, proceeding with Claude analysis

### Convergence Status
- Final Status: `partially_converged` (Codex analysis failed, proceeding with Claude analysis)

## Pending User Decisions

- DEC-1: Synchronous vs asynchronous implementation approach
  - Claude Position: Follow draft requirement for synchronous simplicity while using tokio for signal handling
  - Codex Position: *Pending Codex analysis*
  - Tradeoff Summary: Synchronous is simpler but may limit performance; async could handle more concurrent operations
  - Decision Status: `PENDING`

- DEC-2: Error handling strategy for Rust implementation
  - Claude Position: Use Rust's Result types with comprehensive error variants
  - Codex Position: *Pending Codex analysis*  
  - Tradeoff Summary: Rust's type system provides better error safety but may require more boilerplate
  - Decision Status: `PENDING`

- DEC-3: Testing strategy for CLI compatibility
  - Claude Position: Create comprehensive integration tests comparing Go and Rust CLI output
  - Codex Position: *Pending Codex analysis*
  - Tradeoff Summary: Extensive testing ensures compatibility but increases development time
  - Decision Status: `PENDING`

## Implementation Notes

### Code Style Requirements
- Implementation code and comments must NOT contain plan-specific terminology such as "AC-", "Milestone", "Step", "Phase", or similar workflow markers
- These terms are for plan documentation only, not for the resulting codebase
- Use descriptive, domain-appropriate naming in code instead

### Key Technical Considerations
- **Error Handling**: Use Rust's Result types with comprehensive error variants for safe error propagation
- **Data Structures**: Create Rust structs that mirror Go's tcping, userInput, and printer interfaces
- **Concurrency**: While keeping core logic synchronous, use tokio for signal handling and async I/O where beneficial
- **Testing**: Create comprehensive integration tests that compare Rust output with Go output for compatibility verification
- **Performance**: Leverage Rust's zero-cost abstractions while maintaining readability
- **Cross-platform**: Focus on Linux support first as specified, but design with future cross-platform compatibility in mind

### Documentation Requirements
- **README.md**: Must include project description, prerequisites, installation steps, usage examples, configuration options, and project structure overview
- **Code Documentation**: Use Rust doc comments for public APIs and complex logic
- **Commit Messages**: Use conventional commit prefix `feat:` for all commits as specified

## Output File Convention

This template is used to produce the main output file (e.g., `plan.md`).

### Translated Language Variant

When `alternative_plan_language` resolves to a supported language name through merged config loading, a translated variant of the output file is also written after the main file. Humanize loads config from merged layers in this order: default config, optional user config, then optional project config; `alternative_plan_language` may be set at any of those layers. The variant filename is constructed by inserting `_<code>` (the ISO 639-1 code from the built-in mapping table) immediately before the file extension:

- `plan.md` becomes `plan_<code>.md` (e.g. `plan_zh.md` for Chinese, `plan_ko.md` for Korean)
- `docs/my-plan.md` becomes `docs/my-plan_<code>.md`
- `output` (no extension) becomes `output_<code>`

The translated variant file contains a full translation of the main plan file's current content in the configured language. All identifiers (`AC-*`, task IDs, file paths, API names, command flags) remain unchanged, as they are language-neutral.

When `alternative_plan_language` is empty, absent, set to `"English"`, or set to an unsupported language, no translated variant is written. Humanize does not auto-create `.humanize/config.json` when no project config file is present.

--- Original Design Draft Start ---

# Requirement

refactor current tcping golang project to a rust project with full features

1. use clap, tokio, rusqlite
2. maintain exact CLI flag compatibility
3. keep it synchronous for simplicity
4. support Linux first

---

## Standard Deliverables (mandatory for every project)

- **README.md** — must be included at the project root with: project title & description, prerequisites, installation steps, usage examples with code snippets, configuration options, and project structure overview.
- **Git commits** — use conventional commit prefix `feat:` for all commits.

--- Original Design Draft End ---
