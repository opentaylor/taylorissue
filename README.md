<div align="center">

<img src="docs/images/logo.png" alt="TaylorIssue Logo" width="200">

# 一修哥 (TaylorIssue)

The world's only OpenClaw repair software. An agent that manages agents.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.0.1-orange.svg)]()
[![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8D8.svg?logo=tauri)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-19-61DAFB.svg?logo=react)](https://react.dev/)
[![Release](https://img.shields.io/badge/release-March%2019%202026-green.svg)]()

[[中文 README](README_zh.md)]

*No technical skills needed to install. No money needed to repair.*
*Install, use, repair, and uninstall — TaylorIssue helps you finish everything in one go.*

</div>

## News

[2026-03-19] We released v0.0.1. This is the initial release of TaylorIssue, featuring one-click install, one-click repair, one-click uninstall, agent chat, and skill management for OpenClaw.

## What is TaylorIssue

TaylorIssue (一修哥) is a desktop application for installing, using, repairing, and managing [OpenClaw](https://github.com/nicepkg/openclaw), the open-source AI agent framework. Think of it as an agent that manages agents. It uses AI to automate every step of OpenClaw's lifecycle, so you never have to touch the terminal.

### Core Features

- One-click install with AI-guided flow that auto-detects your environment and installs Git, Node.js, and OpenClaw for you
- One-click repair with system scan, automatic diagnostics, and fixes for common issues, plus custom issue descriptions for AI-powered diagnosis
- One-click uninstall with granular cleanup options for services, packages, workspace, config, and data
- Agent chat that lets you talk to your OpenClaw agents directly from the app
- Skill management for browsing, installing, and managing agent skills from both local sources and the ClawHub community marketplace
- Model configuration that sets up your LLM provider in seconds, with a built-in guide to free API providers

## Quick Start

### Overview

The dashboard gives you a bird's-eye view of TaylorIssue, quick model configuration, and project metadata.

<div align="center">
<img src="docs/images/overview.png" alt="Overview" width="700">
</div>

### Install

One-click OpenClaw installation. The AI assistant walks through system detection, Git setup, Node.js setup, OpenClaw installation, configuration, and gateway verification, all automatically.

<div align="center">
<img src="docs/images/install.png" alt="Install" width="700">
</div>

### Quick Fix

System scan and repair. Detects anomalies like stopped services, missing configs, expiring certificates, and disk issues. Supports both automatic fixes and custom issue descriptions for AI diagnosis.

<div align="center">
<img src="docs/images/quick-fix.png" alt="Quick Fix" width="700">
</div>

### Uninstall

Granular uninstall with fine-grained control over what gets removed, including services, packages, workspace, config files, and historical data.

<div align="center">
<img src="docs/images/uninstall.png" alt="Uninstall" width="700">
</div>

### Agent Chat

Chat directly with your OpenClaw agents. Select from available agents in the sidebar and start a conversation.

<div align="center">
<img src="docs/images/message.png" alt="Agent Chat" width="700">
</div>

### Skill Management

View and manage installed OpenClaw skills. Browse the ClawHub community marketplace to discover and install new skills with one click.

<div align="center">
<img src="docs/images/skills.png" alt="Skills" width="700">
</div>

## Installation

### Prerequisites

- [Node.js](https://nodejs.org/) (>= 18) with pnpm
- [Rust](https://www.rust-lang.org/tools/install) toolchain for Tauri
- System dependencies for Tauri as described in the [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/)

### Build from Source

```bash
git clone https://github.com/tczhangzhi/taylorissue.git
cd taylorissue
pnpm install
pnpm tauri dev
```

To build for production, run `pnpm build && pnpm tauri build`.

## Development

| Command | Description |
|---------|-------------|
| `pnpm dev` | Start the Vite dev server on port 1420 (frontend only) |
| `pnpm tauri dev` | Start the full Tauri app in development mode |
| `pnpm build` | Build the frontend for production |
| `pnpm tauri build` | Build the distributable desktop app |

## Roadmap

- [ ] Memory, making knowledge management effortless
- [ ] Skill, making tool usage effortless
- [ ] Cron, making proactive interaction effortless
- [ ] Security, making privacy protection effortless
- [ ] Multi-Agent, making scaling and collaboration effortless
- [ ] Artifact, making output delivery effortless

## Acknowledgements

Maintainers

- Zhang Zhi, PhD Student, Department of Computing, The Hong Kong Polytechnic University
- Liu Yan, Professor, Department of Computing, The Hong Kong Polytechnic University

Sponsors

- Chen Gong, PhD, Department of Computing, The Hong Kong Polytechnic University

## License

[MIT License](LICENSE) © 2026 OpenTaylor
