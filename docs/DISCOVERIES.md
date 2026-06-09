# Discoveries Index

Per-phiếu discovery reports. Most recent first.

| Phiếu | Date | Summary |
|---|---|---|
| [P004](discoveries/P004.md) | 2026-06-09 | block-unsafe-merge port: anchors #10-13 CORRECT; fail-CLOSED divergence CỐ Ý (gh diff fail→BLOCK, ngược 3 hook kia fail-open); io.rs `command` field additive (P006/P007 reuse); pure-fn 22 unit tests + 4 gh-free integration; no mock seam/dep; env-fallback CLAUDE_TOOL_INPUT not ported; 28/28 tests pass. |
| [P003](discoveries/P003.md) | 2026-06-09 | block-env-edit port: anchors #10-12 CORRECT; env-fallback CLAUDE_TOOL_INPUT not ported (conscious gap — stdin always piped, shared harness change = Tầng 1); regex::Regex verbatim ^\.env($|\.); basename via rsplit('/'); 24/24 tests pass. |
| [P002](discoveries/P002.md) | 2026-06-09 | architect-guard port: anchors #12-15 CORRECT; env fallback divergence (script-dir→cwd) accepted; std isolation sufficient (no `tempfile` dep); string match pattern; `#[allow(dead_code)]` removed from BLOCK/block; 14/14 tests pass. |
| [P001](discoveries/P001.md) | 2026-06-09 | Scaffold CLI 5-subcmd: all 8 anchors CORRECT; clap kebab-case auto-derived; `#[allow(dead_code)]` for forward-declared scaffold items; 8/8 tests pass. |
