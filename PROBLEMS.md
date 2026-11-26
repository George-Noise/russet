### 2026-11-26 — A bare `cargo test --features mcp` outside verify.sh hung 10 min in the recycled sandbox

- What happened: after the tests-green checkpoint I ran `cargo test --profile gates --features mcp` as a BARE command (to compare the mcp suite count against the previous round's 392) and the tool call hit its 10-minute cap — killed. Expected: the suite runs green in ~10 s (the previous round's log); actual: a silent hang.
- How it showed up: exactly SANDBOX.md's trap 10 / the verify.sh header's own warning — in a recycled sandbox the session stdin is an open pipe, every mcp test binary spawns the MCP stdin reader on it, and EOF never comes; verify.sh avoids it with `exec </dev/null` at the top, which my bare command bypassed. The trap is documented; I ran into it anyway by muscle memory ("run the tests") instead of going through verify.sh.
- Cost: one 10-minute tool call, killed and re-issued; the gates-lane target stayed warm (the hang was in the test binary, not the compile).
- What could improve: none, one-off — the rule was already written down and verify.sh is the sanctioned path; the only lesson is to never run the mcp suite bare, which the docs already say.
