# Workflow Orchestration MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-workflow.svg)](https://crates.io/crates/mcp-workflow)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Multi-step workflow orchestration for AI agents — define workflows, run instances, manage approvals, track state machines. 13 tools with seeded demo workflows.

## Tools (13)

| Tool | Purpose | Risk |
|------|---------|------|
| `list_workflows` | List workflow definitions | read_only |
| `get_workflow` | Get workflow with steps | read_only |
| `create_workflow` | Create a new workflow | internal_write |
| `start_workflow` | Start a workflow instance | internal_write |
| `list_instances` | List running instances | read_only |
| `get_instance` | Get instance details + history | read_only |
| `advance_step` | Advance to next step | internal_write |
| `cancel_instance` | Cancel a running instance | internal_write |
| `list_approvals` | List pending approvals | read_only |
| `resolve_approval` | Approve or reject | internal_write |
| `get_history` | Get execution history | read_only |
| `retry_instance` | Retry a failed instance | internal_write |
| `get_stats` | Running/waiting/completed counts | read_only |

## Installation

```bash
cargo install mcp-workflow
```

## Configuration

No configuration needed — starts with 3 demo workflows (Employee Onboarding, Production Deployment, Expense Approval).

```json
{ "mcpServers": { "workflow": { "command": "mcp-workflow" } } }
```

## License

Apache-2.0 — Part of [ADK-Rust Enterprise](https://enterprise.adk-rust.com)
