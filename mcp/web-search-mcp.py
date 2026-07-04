# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "mcp",
#   "rich",
#   "ollama",
# ]
# ///
"""
MCP stdio server exposing Ollama web_search and web_fetch as tools.

Environment:
- OLLAMA_API_KEY (required): if set, will be used as Authorization header.
"""

from __future__ import annotations

import asyncio
import sys
from typing import Any, Dict

from ollama import Client

try:
  from mcp.server.fastmcp import FastMCP
  _FASTMCP_AVAILABLE = True
except Exception:
  _FASTMCP_AVAILABLE = False

if not _FASTMCP_AVAILABLE:
  from mcp.server import Server
  from mcp.server.stdio import stdio_server

client = Client()

def _web_search_impl(query: str, max_results: int = 3) -> Dict[str, Any]:
  print(f"[mcp] web_search(query={query!r}, max_results={max_results})", file=sys.stderr, flush=True)
  res = client.web_search(query=query, max_results=max_results)
  import json
  dumped = res.model_dump()
  print(f"[mcp] RAW RESPONSE ollama.com:\n{json.dumps(dumped, indent=2, ensure_ascii=False)[:3000]}", file=sys.stderr, flush=True)
  return dumped

def _web_fetch_impl(url: str) -> Dict[str, Any]:
  print(f"[mcp] web_fetch(url={url!r})", file=sys.stderr, flush=True)
  res = client.web_fetch(url=url)
  return res.model_dump()

if _FASTMCP_AVAILABLE:
  app = FastMCP('ollama-search-fetch')

  @app.tool()
  def web_search(query: str, max_results: int = 3) -> Dict[str, Any]:
    """Perform a web search using Ollama's hosted search API."""
    return _web_search_impl(query=query, max_results=max_results)

  @app.tool()
  def web_fetch(url: str) -> Dict[str, Any]:
    """Fetch the content of a web page for the provided URL."""
    return _web_fetch_impl(url=url)

  if __name__ == '__main__':
    print("[mcp] MCP server ready (FastMCP)", file=sys.stderr, flush=True)
    app.run()
else:
  server = Server('ollama-search-fetch')

  @server.tool()
  async def web_search(query: str, max_results: int = 3) -> Dict[str, Any]:
    """Perform a web search using Ollama's hosted search API."""
    return await asyncio.to_thread(_web_search_impl, query, max_results)

  @server.tool()
  async def web_fetch(url: str) -> Dict[str, Any]:
    """Fetch the content of a web page for the provided URL."""
    return await asyncio.to_thread(_web_fetch_impl, url)

  async def _main() -> None:
    async with stdio_server() as (read, write):
      await server.run(read, write)

  if __name__ == '__main__':
    print("[mcp] MCP server ready (stdio)", file=sys.stderr, flush=True)
    asyncio.run(_main())
