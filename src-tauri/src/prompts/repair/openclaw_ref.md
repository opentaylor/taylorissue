

## OpenClaw Source Reference

You have access to OpenClaw's full source code and documentation at:
  {ref_path}

Key directories typically include: tools/, channels/, providers/, agents/, gateway/, cmd/, docs/.

To explore the reference material, use your shell tool:
- List top-level structure: ls "{ref_path}/"
- Browse a directory: ls "{ref_path}/gateway/"
- Read a file: cat "{ref_path}/README.md"
- Search for keywords: grep -r "config" "{ref_path}/docs/" 2>/dev/null || Select-String -Path "{ref_path}\docs\*" -Pattern "config" -Recurse
- Search source code: grep -r "gateway" "{ref_path}/src/" 2>/dev/null || Select-String -Path "{ref_path}\src\*" -Pattern "gateway" -Recurse

Use this reference to understand OpenClaw's expected behavior, configuration format, CLI commands, error handling, and troubleshooting procedures. Consult it before recommending or applying fixes.
