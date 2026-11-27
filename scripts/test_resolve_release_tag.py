#!/usr/bin/env python3
"""Unit tests for resolve-release-tag.py (no network)."""
from __future__ import annotations

import importlib.util
import io
import json
import sys
import unittest
from pathlib import Path
from unittest import mock
from urllib.error import HTTPError, URLError


def load_resolver():
    path = Path(__file__).with_name("resolve-release-tag.py")
    spec = importlib.util.spec_from_file_location("resolve_release_tag", path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


class ResolveReleaseTagTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load_resolver()

    def test_major_resolution_picks_highest_patch(self):
        rels = [
            {"tag_name": "v8.0.1", "draft": False, "assets": [{"name": "lacan-x86_64.tar.gz"}]},
            {"tag_name": "v8.0.2", "draft": False, "assets": [{"name": "lacan-x86_64.tar.gz"}]},
        ]
        self.assertEqual(self.mod.resolve_major("8", rels), "v8.0.2")

    def test_http_502_retries_then_succeeds(self):
        payload = json.dumps({"tag_name": "v8.0.1"}).encode()
        ok = io.BytesIO(payload)

        def urlopen_side_effect(req, timeout=60):
            urlopen_side_effect.calls += 1
            if urlopen_side_effect.calls < 3:
                raise HTTPError(
                    req.full_url,
                    502,
                    "Bad Gateway",
                    hdrs=None,
                    fp=None,
                )
            return ok

        urlopen_side_effect.calls = 0
        req = mock.Mock(full_url="https://example.test/releases/latest")
        with mock.patch.object(self.mod.urllib.request, "urlopen", side_effect=urlopen_side_effect):
            with mock.patch.object(self.mod.time, "sleep") as sleep:
                data = self.mod.fetch_json(req)
        self.assertEqual(data["tag_name"], "v8.0.1")
        self.assertEqual(urlopen_side_effect.calls, 3)
        self.assertEqual(sleep.call_count, 2)

    def test_http_429_honors_retry_after(self):
        err = HTTPError(
            "https://example.test/",
            429,
            "Too Many Requests",
            hdrs={"Retry-After": "9"},
            fp=None,
        )

        def urlopen_side_effect(req, timeout=60):
            urlopen_side_effect.calls += 1
            if urlopen_side_effect.calls == 1:
                raise err
            return io.BytesIO(b"[]")

        urlopen_side_effect.calls = 0
        req = mock.Mock(full_url="https://example.test/releases")
        with mock.patch.object(self.mod.urllib.request, "urlopen", side_effect=urlopen_side_effect):
            with mock.patch.object(self.mod.time, "sleep") as sleep:
                data = self.mod.fetch_json(req)
        self.assertEqual(data, [])
        sleep.assert_called_once_with(9.0)

    def test_http_404_fails_fast(self):
        req = mock.Mock(full_url="https://example.test/nope")
        with mock.patch.object(
            self.mod.urllib.request,
            "urlopen",
            side_effect=HTTPError(req.full_url, 404, "Not Found", hdrs=None, fp=None),
        ):
            with mock.patch.object(self.mod.time, "sleep") as sleep:
                with self.assertRaises(HTTPError):
                    self.mod.fetch_json(req)
        sleep.assert_not_called()

    def test_urlerror_retries_then_fails(self):
        req = mock.Mock(full_url="https://example.test/releases")

        def urlopen_side_effect(req, timeout=60):
            raise URLError("timed out")

        with mock.patch.object(self.mod.urllib.request, "urlopen", side_effect=urlopen_side_effect):
            with mock.patch.object(self.mod.time, "sleep"):
                with self.assertRaises(URLError):
                    self.mod.fetch_json(req)


if __name__ == "__main__":
    raise SystemExit(unittest.main())