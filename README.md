# Workflow Orchestration MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-workflow.svg)](https://crates.io/crates/mcp-workflow)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

Multi-step workflow orchestration for AI agents — define workflows with typed steps, run instances, manage approvals, track execution history. 13 tools with seeded demo workflows.

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-workflow/main/docs/assets/architecture.svg" alt="MCP Workflow Architecture" width="850"/>
</p>

## Tools (13)

### Definitions (3)

| Tool | Purpose | Risk |
|------|---------|------|
| `list_workflows` | List all workflow definitions | read_only |
| `get_workflow` | Get workflow with all steps | read_only |
| `create_workflow` | Create a new workflow | internal_write |

### Instances (6)

| Tool | Purpose | Risk |
|------|---------|------|
| `start_workflow` | Start a new instance | internal_write |
| `list_instances` | List instances (filter by status) | read_only |
| `get_instance` | Get instance details + context | read_only |
| `advance_step` | Advance to next step | internal_write |
| `cancel_instance` | Cancel a running instance | internal_write |
| `retry_instance` | Retry a failed instance | internal_write |

### Approvals (2)

| Tool | Purpose | Risk |
|------|---------|------|
| `list_approvals` | List pending approvals | read_only |
| `resolve_approval` | Approve or reject | internal_write |

### Observability (2)

| Tool | Purpose | Risk |
|------|---------|------|
| `get_history` | Get execution history | read_only |
| `get_stats` | Running/waiting/completed counts | read_only |

## Step Types

| Type | Behavior |
|------|----------|
| `Action` | Execute a task, auto-advance |
| `Approval` | Pause until approved/rejected |
| `Condition` | Branch based on context |
| `Notification` | Send notification, auto-advance |
| `Wait` | Pause for external event |

## Installation

```bash
cargo install mcp-workflow
```

## Configuration

No configuration needed — starts with 3 demo workflows:

- **Employee Onboarding** — IT setup → Manager approval → Welcome email
- **Production Deployment** — Tests → Security scan → Lead approval → Deploy → Notify
- **Expense Approval** — Submit → Manager review → Finance approval → Reimburse

## Client Configuration

### Claude Desktop / Kiro / Cursor

```json
{
  "mcpServers": {
    "workflow": {
      "command": "mcp-workflow",
      "args": []
    }
  }
}
```

## Usage Examples

### Start an onboarding workflow
```
"Start the employee onboarding workflow for Frank in Engineering"
→ start_workflow(workflow_id="wf-onboard", context={"employee": "Frank", "department": "Engineering"})
```

### Approve a pending request
```
"Approve the pending manager approval"
→ list_approvals → resolve_approval(approval_id="apr-1", approved=true)
```

### Create a custom workflow
```
"Create a bug fix workflow: fix code → code review → deploy"
→ create_workflow(name="Bug Fix", steps=[{name:"Fix code", step_type:"action"}, {name:"Code review", step_type:"approval", assignee:"senior-dev"}, {name:"Deploy", step_type:"action"}])
```

## MCP Server Manifest

```toml
server_id = "mcp_workflow"
display_name = "Workflow"
version = "1.0.0"
domain = "platform-core"
risk_level = "medium"
writes_allowed = "gated"
```

## License

Apache-2.0

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

Built with ❤️ by [Zavora AI](https://zavora.ai)

## rmcp and MCP compatibility

This server is built with [`rmcp` 3.1.2](https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.1.2) and requires Rust 1.94.1 or newer. The rmcp 3 rollout retains legacy MCP initialization compatibility and targets MCP protocol revisions `2025-11-25` and `2026-07-28`.

## MCP 2026-07-28 rollout (P4 workflow/business)

This server uses `rmcp` 3.1.2 and `adk-mcp-sdk` 0.2 with a minimum supported
Rust version of **1.94.1**. It accepts stateless MCP 2026 requests with
per-request protocol, client identity, and capability metadata while retaining
the legacy MCP 2025-11-25 initialize flow for ordinary tools.

- **Tasks:** `task_create`, `task_update`, `task_list`
- **MRTR approvals:** None; this server exposes no manifest-classified protected operations.
- **Discovery and routing:** rmcp serves on-demand discovery and validates the
  per-request protocol envelope; HTTP deployments can route with `Mcp-Method`
  and `Mcp-Name`. The packaged binary currently uses stdio.
- **Caching:** `tools/list` returns a public `ttlMs` of 60,000 for MCP 2026;
  rmcp omits the cache fields for legacy clients.
- **Deprecated extensions:** this server does not add new Roots, Sampling, or
  dynamic client-registration dependencies.

Protected tools require `MCP_REQUEST_STATE_KEY` with at least 32 high-entropy
bytes. All replicas must share that key so sealed approval state can resume on
another instance. Approval state is bound to the client identity, tool, and
arguments and expires after two minutes. Missing identity, invalid state,
rejection, or legacy protocol use fails closed. Task records are process-local
for the current stdio runtime; use a durable task store before deploying the
server behind scale-to-zero HTTP infrastructure.
