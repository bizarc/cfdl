#!/usr/bin/env python3
"""An OpenAI-compatible API agent for the eval — OpenRouter, xAI, OpenAI, etc.

Adapter contract (runner.py `--agent cmd:`): task JSON on stdin,
`{"files": {...}}` on stdout. Everything else goes to stderr.

The model gets the five loop tools as function definitions; each tool call is
bridged to a local `cfdl-mcp --packs` stdio server, so the model never touches
the filesystem and `diff` is never offered. Same sandbox properties as the
Claude adapter; same prompt (agents/common.py), so scores compare.

Environment:
  OPENROUTER_API_KEY   required — set it in your shell; it is read from the
                       environment only and never logged
  CFDL_EVAL_MODEL      model slug (default "x-ai/grok-4"); any chat model on
                       the endpoint that supports tool calling
  OPENROUTER_BASE_URL  default "https://openrouter.ai/api/v1"; point at any
                       OpenAI-compatible endpoint (e.g. https://api.x.ai/v1)

Self-check without a key: `python3 openrouter.py --self-check` spawns the
tool bridge and lists the tools it would offer.
"""

import importlib.util
import json
import os
import pathlib
import subprocess
import sys
import urllib.request

sys.stderr.reconfigure(encoding="utf-8")

ROOT = pathlib.Path(__file__).resolve().parents[3]
_spec = importlib.util.spec_from_file_location(
    "common", pathlib.Path(__file__).parent / "common.py"
)
common = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(common)

ALLOWED_TOOLS = ("compile", "run", "lookup", "skeleton", "explain")
MAX_TURNS = 40


class ToolBridge:
    """A cfdl-mcp stdio server as an OpenAI tools backend."""

    def __init__(self):
        self.proc = subprocess.Popen(
            [str(ROOT / "target" / "debug" / "cfdl-mcp"), "--packs", str(ROOT / "packs")],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            encoding="utf-8",
        )
        self._id = 0
        self._rpc("initialize", {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {"name": "openrouter-adapter", "version": "0"},
        })
        self._notify("notifications/initialized")
        listing = self._rpc("tools/list", {})
        self.tools = [
            {
                "type": "function",
                "function": {
                    "name": tool["name"],
                    "description": tool.get("description", ""),
                    "parameters": tool.get("inputSchema", {"type": "object"}),
                },
            }
            for tool in listing["tools"]
            if tool["name"] in ALLOWED_TOOLS
        ]

    def _send(self, message: dict):
        self.proc.stdin.write(json.dumps(message) + "\n")
        self.proc.stdin.flush()

    def _notify(self, method: str):
        self._send({"jsonrpc": "2.0", "method": method})

    def _rpc(self, method: str, params: dict):
        self._id += 1
        self._send({"jsonrpc": "2.0", "id": self._id, "method": method, "params": params})
        while True:
            line = self.proc.stdout.readline()
            if not line:
                raise RuntimeError("cfdl-mcp exited")
            message = json.loads(line)
            if message.get("id") == self._id:
                if "error" in message:
                    raise RuntimeError(json.dumps(message["error"]))
                return message["result"]

    def call(self, name: str, arguments: dict) -> str:
        if name not in ALLOWED_TOOLS:
            return json.dumps({"error": f"tool '{name}' is not available"})
        try:
            result = self._rpc("tools/call", {"name": name, "arguments": arguments})
        except RuntimeError as err:
            return json.dumps({"error": str(err)[:2000]})
        if result.get("structuredContent") is not None:
            return json.dumps(result["structuredContent"])[:120_000]
        return json.dumps(result.get("content", []))[:120_000]

    def close(self):
        try:
            self.proc.stdin.close()
            self.proc.terminate()
        except OSError:
            pass


def chat(base_url: str, api_key: str, payload: dict) -> dict:
    request = urllib.request.Request(
        f"{base_url}/chat/completions",
        data=json.dumps(payload).encode("utf-8"),
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        },
    )
    with urllib.request.urlopen(request, timeout=600) as response:
        return json.loads(response.read().decode("utf-8"))


def main() -> int:
    if "--self-check" in sys.argv[1:]:
        bridge = ToolBridge()
        names = [t["function"]["name"] for t in bridge.tools]
        bridge.close()
        print(f"self-check: bridge offers {names}", file=sys.stderr)
        return 0 if sorted(names) == sorted(ALLOWED_TOOLS) else 1

    api_key = os.environ.get("OPENROUTER_API_KEY")
    if not api_key:
        print("OPENROUTER_API_KEY is not set", file=sys.stderr)
        return 2
    model = os.environ.get("CFDL_EVAL_MODEL", "x-ai/grok-4")
    base_url = os.environ.get("OPENROUTER_BASE_URL", "https://openrouter.ai/api/v1")

    task = json.load(sys.stdin)
    bridge = ToolBridge()
    messages = [
        {"role": "system", "content": common.PROMPT_HEADER},
        {"role": "user", "content": common.build_prompt(task)},
    ]
    try:
        for _ in range(MAX_TURNS):
            reply = chat(base_url, api_key, {
                "model": model,
                "messages": messages,
                "tools": bridge.tools,
            })
            choice = reply["choices"][0]["message"]
            messages.append(choice)
            tool_calls = choice.get("tool_calls") or []
            if not tool_calls:
                answer = common.extract_files(choice.get("content") or "")
                if answer:
                    print(json.dumps(answer))
                    return 0
                # One nudge: the model stopped without the required block.
                messages.append({
                    "role": "user",
                    "content": "Output your final answer now as exactly one fenced "
                    'json block: {"files": {"model.cfdl": "..."}}',
                })
                continue
            for call in tool_calls:
                function = call["function"]
                try:
                    arguments = json.loads(function.get("arguments") or "{}")
                except json.JSONDecodeError:
                    arguments = {}
                messages.append({
                    "role": "tool",
                    "tool_call_id": call["id"],
                    "content": bridge.call(function["name"], arguments),
                })
        print("agent hit the turn limit without a final answer", file=sys.stderr)
        return 1
    finally:
        bridge.close()


if __name__ == "__main__":
    sys.exit(main())
