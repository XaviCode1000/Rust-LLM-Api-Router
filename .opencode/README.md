# OpenCode Configuration for Rust LLM API Router

This directory contains OpenCode configuration for working with the Rust LLM API Router project.

## Configuration Files

- `opencode.json`: Main OpenCode configuration with MCP server settings and agent permissions
- `README.md`: This file

## Setup Instructions

1. **No Docker required**: This configuration uses the remote GitHub MCP server
2. **Simple setup**: Just create the configuration file and authenticate

### Quick Start:
```bash
# Restart OpenCode or reload configuration
```

## Available Tools

### GitHub MCP Server Tools (Remote)
All agents have access to GitHub MCP tools:
- Repository management (`github_repos_*`)
- Issue and PR management (`github_issues_*`, `github_pull_requests_*`)
- GitHub Actions (`github_actions_*`)
- Code security (`github_code_security_*`)
- And many more...

### Context7 Tools
All agents have access to Context7 tools:
- Documentation search (`context7_*`)
- Code examples and API references

## Agent Access

All agents (build, plan, general, explore) have full access to:
- ✅ GitHub MCP server tools
- ✅ Context7 documentation tools
- ✅ Project-specific instructions

## Authentication

The remote GitHub MCP server uses OAuth:
1. First time you use a GitHub tool, OpenCode will prompt for authentication
2. It will open your browser for GitHub login
3. Authorize the required permissions
4. Tokens are stored securely and automatically managed

## Usage Examples

```bash
# Start OpenCode
opencode start

# Use GitHub tools in your prompts (all agents can access)
"Find Rust API examples for Actix-web routing using github tools"

# Use Context7 for documentation (all agents can access)
"Search for Tokio async runtime documentation using context7"

# Switch between agents using Tab key
# All agents have access to GitHub and Context7 tools
```

## Configuration Details

This setup uses:
- **Remote GitHub MCP Server**: `https://api.githubcopilot.com/mcp/`
- **Remote Context7 MCP Server**: `https://mcp.context7.com/mcp`
- **OAuth Authentication**: Secure GitHub login flow
- **No Docker Required**: Pure remote services
- **All Agents Access**: Build, Plan, General, and Explore agents all have full access to MCP tools

## Troubleshooting

If you encounter issues:
1. Check your internet connection
2. Verify OpenCode is updated to the latest version
3. Check OpenCode logs for error messages
4. Try authenticating manually:
   ```bash
   opencode mcp auth github
   ```

For more information about OpenCode MCP configuration, see:
- https://opencode.ai/docs/mcp-servers/
- https://github.com/github/github-mcp-server
