/**
 * LLM Router CLI - Tool for interacting with llm-router from OpenCode
 * 
 * This tool allows OpenCode agents to:
 * - List providers and their status
 * - List accounts
 * - Login to providers
 * - List available models
 * - Check authentication status
 */

import { tool } from "@opencode-ai/plugin"
import path from "path"

const LLM_ROUTER_BIN = "/home/gazadev/Dev/my_apps/Rust-LLM-Api-Router/target/release/llm-router"

export const llmRouterProviderList = tool({
  description: "List all configured providers and their status (enabled/disabled, configured/not set)",
  args: {},
  async execute(args, context) {
    try {
      const result = await Bun.$`${LLM_ROUTER_BIN} provider list`.text()
      return result
    } catch (error) {
      return `Error listing providers: ${error.message}`
    }
  },
})

export const llmRouterAccountList = tool({
  description: "List all configured accounts with API keys",
  args: {},
  async execute(args, context) {
    try {
      const result = await Bun.$`${LLM_ROUTER_BIN} account list`.text()
      return result
    } catch (error) {
      return `Error listing accounts: ${error.message}`
    }
  },
})

export const llmRouterModels = tool({
  description: "List available models for a specific provider (requires authentication)",
  args: {
    provider: tool.schema.string().describe("Provider ID (e.g., groq, openai, openrouter, mistral)"),
  },
  async execute(args, context) {
    try {
      const result = await Bun.$`${LLM_ROUTER_BIN} provider models --provider ${args.provider}`.text()
      return result
    } catch (error) {
      return `Error listing models: ${error.message}`
    }
  },
})

export const llmRouterAuthStatus = tool({
  description: "Check authentication status for all providers",
  args: {},
  async execute(args, context) {
    try {
      const providers = await Bun.$`${LLM_ROUTER_BIN} provider list`.text()
      const accounts = await Bun.$`${LLM_ROUTER_BIN} account list`.text()
      return `=== Providers ===\n${providers}\n\n=== Accounts ===\n${accounts}`
    } catch (error) {
      return `Error checking status: ${error.message}`
    }
  },
})

export const llmRouterChat = tool({
  description: "Send a chat message to an LLM provider (requires server running)",
  args: {
    message: tool.schema.string().describe("Message to send to the LLM"),
    model: tool.schema.string().optional().describe("Model to use (e.g., groq:llama-3.3-70b-versatile)"),
  },
  async execute(args, context) {
    const model = args.model || "groq:llama-3.3-70b-versatile"
    const message = args.message
    
    try {
      // Call the local server
      const response = await fetch("http://localhost:8080/v1/chat/completions", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          model: model,
          messages: [{ role: "user", content: message }],
        }),
      })
      
      if (!response.ok) {
        return `Error: HTTP ${response.status} - ${await response.text()}`
      }
      
      const data = await response.json()
      return data.choices?.[0]?.message?.content || "No response"
    } catch (error) {
      return `Error: ${error.message}. Make sure the server is running with: llm-router --port 8080`
    }
  },
})

export const llmRouterServerStatus = tool({
  description: "Check if the llm-router server is running",
  args: {},
  async execute(args, context) {
    try {
      const response = await fetch("http://localhost:8080/health")
      if (response.ok) {
        const data = await response.json()
        return `✅ Server is running: ${JSON.stringify(data)}`
      }
      return `⚠️ Server responded with status: ${response.status}`
    } catch (error) {
      return `❌ Server is not running. Start it with: llm-router --port 8080`
    }
  },
})
