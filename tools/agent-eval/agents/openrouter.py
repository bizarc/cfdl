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
import urllib.error
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
# A single task that has spent this much has lost the plot; stop paying for it
# and let it answer with what it has. Override with CFDL_EVAL_MAX_COST.
DEFAULT_TASK_COST_CAP = 1.50


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


def _ssl_context():
    """A CA-verified TLS context. python.org framework builds ship no system
    CA bundle, so default urllib HTTPS fails outright; certifi supplies one."""
    import ssl

    try:
        import certifi

        return ssl.create_default_context(cafile=certifi.where())
    except ImportError:
        return ssl.create_default_context()


RETRIES = 5


def chat(base_url: str, api_key: str, payload: dict) -> dict:
    """One chat completion, with retries on the transient failures the smoke
    runs hit in the wild: 429 rate limits, 5xx, network errors, and truncated
    or non-JSON response bodies. Only a non-retryable 4xx raises immediately —
    that is a real request problem, and retrying it would just repeat it."""
    import time

    last = None
    for attempt in range(RETRIES):
        request = urllib.request.Request(
            f"{base_url}/chat/completions",
            data=json.dumps(payload).encode("utf-8"),
            headers={
                "Authorization": f"Bearer {api_key}",
                "Content-Type": "application/json",
            },
        )
        try:
            with urllib.request.urlopen(
                request, timeout=600, context=_ssl_context()
            ) as response:
                return json.loads(response.read().decode("utf-8"))
        except urllib.error.HTTPError as err:
            body = err.read().decode("utf-8", "replace")[:500]
            last = f"API {err.code} from {base_url}: {body}"
            if err.code not in (408, 429, 500, 502, 503, 504):
                raise RuntimeError(last) from None
        except (urllib.error.URLError, TimeoutError, json.JSONDecodeError, OSError) as err:
            last = f"{type(err).__name__}: {err}"
        wait = 8 * (attempt + 1)
        print(f"retryable failure ({last}); retrying in {wait}s", file=sys.stderr)
        time.sleep(wait)
    raise RuntimeError(f"gave up after {RETRIES} attempts: {last}")


def tool_calls_pending(choice: dict) -> bool:
    return bool(choice.get("tool_calls"))


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
    cost_cap = float(os.environ.get("CFDL_EVAL_MAX_COST", DEFAULT_TASK_COST_CAP))
    spend = {"cost": 0.0, "prompt_tokens": 0, "completion_tokens": 0, "calls": 0}
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
                # OpenRouter returns the actual charge for the call, so a
                # baseline reports observed spend rather than list-price math.
                "usage": {"include": True},
            })
            usage = reply.get("usage") or {}
            spend["cost"] += float(usage.get("cost") or 0.0)
            spend["prompt_tokens"] += int(usage.get("prompt_tokens") or 0)
            spend["completion_tokens"] += int(usage.get("completion_tokens") or 0)
            spend["calls"] += 1
            choice = reply["choices"][0]["message"]
            messages.append(choice)
            if spend["cost"] >= cost_cap and tool_calls_pending(choice):
                # Out of budget mid-loop: ask for the answer as it stands
                # rather than abandoning the task with nothing.
                messages.append({
                    "role": "user",
                    "content": "Budget reached. Output your best answer now as "
                    'exactly one fenced json block: {"files": {"model.cfdl": "..."}}',
                })
                print(f"cost cap ${cost_cap} reached", file=sys.stderr)
                continue
            tool_calls = choice.get("tool_calls") or []
            if not tool_calls:
                answer = common.extract_files(choice.get("content") or "")
                if answer:
                    answer["usage"] = spend
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
        print(
            f"agent hit the turn limit without a final answer "
            f"(spent ${spend['cost']:.4f} over {spend['calls']} calls)",
            file=sys.stderr,
        )
        return 1
    finally:
        bridge.close()


if __name__ == "__main__":
    sys.exit(main())
