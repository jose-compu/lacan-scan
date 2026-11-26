"""Security guard patterns copied from agent repos (issue #101)."""

import re

CURL_PIPE_SHELL = re.compile(r"curl\s+[^|\s]+\| sh")
WGET_PIPE_BASH = re.compile(r"wget\s+[^|\s]+\| bash")

PATTERN_TABLE = {
    "curl_pipe_sh": r"curl\s+[^|\s]+\| sh",
    "wget_pipe_bash": r"wget\s+[^|\s]+\| bash",
}