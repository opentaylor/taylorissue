# Changelog

## v0.1.2 - 2026-03-24

### New Features

- Added LLM provider selection (OpenAI / Anthropic) in overview and settings
- Added codebase and issue-based repair for fixing problems in the official package itself

### Improvements

- Optimized repair prompts for faster and more thorough diagnostics
- Removed unnecessary direct provider test from model request check — only the gateway matters
- Doctor repair now only fixes errors, ignoring warnings
- Prevented auto-configuration with placeholder API keys during repair
- Added route guard to redirect users to set API key before accessing other pages
- Improved Windows compatibility across all repair and install prompts

## v0.1.1 - 2026-03-23

### Bug Fixes

- Fixed a bug in one-click repair for OpenClaw
- Fixed a bug in one-click install for OpenClaw

## v0.1.0 - 2026-03-21

### New Features

- One-click install with AI-guided flow that auto-detects your environment and installs Git, Node.js, and OpenClaw for you
- One-click repair with system scan, automatic diagnostics, and fixes for common issues, plus custom issue descriptions for AI-powered diagnosis
- One-click uninstall with granular cleanup options for services, packages, workspace, config, and data
- Agent chat that lets you talk to your OpenClaw agents directly from the app
- Skill management for browsing, installing, and managing agent skills from both local sources and the ClawHub community marketplace
- Model configuration that sets up your LLM provider in seconds, with a built-in guide to free API providers
