"""Offline release safety tests: no compilers, credentials or external writes."""

import argparse
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("release", Path(__file__).parents[1] / "release.py")
release = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(release)


class BundleTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.metadata = {"version": "0.1.0", "commit": "a" * 40}

    def bundle(self, name, asset="linux.tar.gz", contents=b"binary", metadata=None):
        folder = self.root / name
        folder.mkdir()
        path = folder / asset
        path.write_bytes(contents)
        metadata = metadata or self.metadata
        release.seal(folder, metadata, [(path, "native", metadata)])
        return folder

    def test_detects_tampered_binary_and_manifest(self):
        bundle = self.bundle("one")
        (bundle / "linux.tar.gz").write_bytes(b"broken")
        with self.assertRaisesRegex(release.ReleaseError, "Corrupt asset"):
            release.verify_bundle(bundle)
        bundle = self.bundle("two")
        data = release.read_json(bundle / "manifest.json")
        data["version"] = "0.2.0"
        release.write_json(bundle / "manifest.json", data)
        with self.assertRaises(release.ReleaseError):
            release.verify_bundle(bundle)

    def test_collects_platforms_but_rejects_different_commits(self):
        first = self.bundle("linux")
        second = self.bundle("mac", "mac.tar.gz")
        output = self.root / "combined"
        release.collect(argparse.Namespace(bundles=[first, second], output=output))
        self.assertEqual(set(release.verify_bundle(output)["assets"]), {"linux.tar.gz", "mac.tar.gz"})
        different = self.bundle("other", metadata=dict(self.metadata, commit="b" * 40))
        failed = self.root / "failed"
        with self.assertRaisesRegex(release.ReleaseError, "different versions/commits"):
            release.collect(argparse.Namespace(bundles=[first, different], output=failed))
        self.assertFalse(failed.exists())

    def test_conflicting_duplicate_assets_are_not_overwritten(self):
        first = self.bundle("one")
        second = self.bundle("two", contents=b"other binary")
        with self.assertRaisesRegex(release.ReleaseError, "same asset"):
            release.collect(argparse.Namespace(bundles=[first, second], output=self.root / "out"))

    def test_duplicate_identical_assets_can_be_collected(self):
        first = self.bundle("one")
        second = self.bundle("two")
        release.collect(argparse.Namespace(bundles=[first, second], output=self.root / "out"))
        self.assertEqual(len(release.verify_bundle(self.root / "out")["assets"]), 1)

    def test_rejects_unsafe_asset_paths_even_with_matching_checksums(self):
        bundle = self.bundle("one")
        data = release.read_json(bundle / "manifest.json")
        data["assets"]["../escape"] = data["assets"].pop("linux.tar.gz")
        release.write_json(bundle / "manifest.json", data)
        with self.assertRaisesRegex(release.ReleaseError, "Invalid asset"):
            release.verify_bundle(bundle)

    def test_checksums_must_cover_exact_manifest(self):
        bundle = self.bundle("one")
        with (bundle / "SHA256SUMS").open("a") as stream:
            stream.write("a" * 64 + "  unlisted.tar.gz\n")
        with self.assertRaisesRegex(release.ReleaseError, "exact release manifest"):
            release.verify_bundle(bundle)

    def test_archive_is_stable_and_includes_hidden_deployment_config(self):
        source = self.root / "files"
        source.mkdir()
        (source / ".npmrc").write_text("ignore-scripts=true\n")
        (source / "solcore").write_bytes(b"exe")
        (source / "solcore").chmod(0o755)
        first, second = self.root / "one.tar.gz", self.root / "two.tar.gz"
        for path in (first, second):
            release.pack_tree(source, path, "solcore", 1234567890)
        self.assertEqual(first.read_bytes(), second.read_bytes())
        extracted = self.root / "extracted"
        release.unpack(first, extracted)
        self.assertEqual((extracted / "solcore" / ".npmrc").read_text(), "ignore-scripts=true\n")
        self.assertTrue((extracted / "solcore" / "solcore").stat().st_mode & 0o100)

    def test_archive_cannot_escape_extraction_directory(self):
        archive = self.root / "evil.tar.gz"
        with tarfile.open(archive, "w:gz") as stream:
            info = tarfile.TarInfo("../escape")
            info.size = 4
            stream.addfile(info, io.BytesIO(b"evil"))
        with self.assertRaises(tarfile.FilterError):
            release.unpack(archive, self.root / "destination")
        self.assertFalse((self.root / "escape").exists())


class PublicationTests(unittest.TestCase):
    def setUp(self):
        self.metadata = {"version": "0.1.0", "commit": "a" * 40}
        self.args = argparse.Namespace(repo="org/project", bundle="bundle")

    def test_remote_tag_mismatch_stops_publication(self):
        with patch.object(release, "remote_tag", return_value={}), patch.object(release, "run", return_value="b" * 40):
            with self.assertRaisesRegex(release.ReleaseError, "another commit"):
                release.check_remote_tag("org/project", "v0.1.0", "a" * 40)

    def test_published_release_is_never_clobbered(self):
        with patch.object(release, "github_context", return_value=(self.metadata, "v0.1.0")), \
                patch.object(release, "remote_release", return_value={"isDraft": False}), \
                patch.object(release, "run") as run:
            with self.assertRaisesRegex(release.ReleaseError, "already public"):
                release.stage_github(self.args)
            run.assert_not_called()

    def test_failed_remote_asset_verification_prevents_publication(self):
        with patch.object(release, "github_context", return_value=(self.metadata, "v0.1.0")), \
                patch.object(release, "check_remote_tag"), \
                patch.object(release, "verify_remote", side_effect=release.ReleaseError("checksum differs")), \
                patch.object(release, "run") as run:
            with self.assertRaisesRegex(release.ReleaseError, "checksum differs"):
                release.publish_github(self.args)
            run.assert_not_called()

    def test_prerelease_never_becomes_latest(self):
        metadata = dict(self.metadata, version="0.1.0-rc.1")
        with patch.object(release, "github_context", return_value=(metadata, "v0.1.0-rc.1")), \
                patch.object(release, "check_remote_tag"), patch.object(release, "verify_remote"), \
                patch.object(release, "run") as run:
            release.publish_github(self.args)
            self.assertIn("--latest=false", run.call_args.args)
            self.assertIn("--prerelease=true", run.call_args.args)

    def test_playground_default_is_dry_run_and_never_rebuilds(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            payload = root / "payload"
            (payload / "dist").mkdir(parents=True)
            release.write_json(payload / "dist" / "release.json", self.metadata)
            bundle = root / "bundle"
            bundle.mkdir()
            archive = bundle / "web.tar.gz"
            release.pack_tree(payload, archive, "solcore-playground-v0.1.0", 1234567890)
            release.seal(bundle, self.metadata, [(archive, "playground", self.metadata)])
            args = argparse.Namespace(bundle=bundle, execute=False)
            with patch.object(release, "run") as run, patch.object(release, "node_version"):
                release.deploy_playground(args)
                commands = [call.args for call in run.call_args_list]
                self.assertEqual(commands, [("npm", "ci", "--include=dev", "--ignore-scripts"),
                                            ("npm", "run", "deploy:check")])
            args.execute = True
            with patch.object(release, "run") as run, patch.object(release, "node_version"):
                release.deploy_playground(args)
                self.assertEqual(run.call_args.args, ("npm", "run", "deploy"))


class VersionTests(unittest.TestCase):
    def test_accepts_semver_and_rejects_ambiguous_tags(self):
        for value in ("0.1.0", "1.2.3-rc.1", "1.0.0-alpha.0"):
            release.validate_version(value)
        for value in ("v0.1.0", "01.2.3", "1.2", "1.2.3-01", "1.2.3+build", "1.2.3/escape"):
            with self.subTest(value=value), self.assertRaises(release.ReleaseError):
                release.validate_version(value)

    def test_bump_updates_cargo_and_npm_lockfiles_without_updating_dependencies(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "Cargo.toml").write_text(
                '[workspace]\nmembers = ["crates/fixture"]\nresolver = "3"\n'
                '[workspace.package]\nversion = "0.1.0"\n')
            crate = root / "crates" / "fixture"
            (crate / "src").mkdir(parents=True)
            (crate / "src" / "lib.rs").write_text("")
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "solcore-release-fixture"\nversion.workspace = true\nedition = "2024"\n')
            (root / "playground").mkdir()
            release.write_json(root / "playground" / "package.json", {"version": "0.1.0"})
            release.write_json(root / "playground" / "package-lock.json", {
                "version": "0.1.0", "packages": {"": {"version": "0.1.0"},
                "../crates/wasm/pkg": {"version": "0.1.0"},
                "node_modules/example": {"version": "7.8.9", "integrity": "unchanged"}}})
            with patch.object(release, "ROOT", root):
                release.set_version("0.2.0-rc.1")
                self.assertEqual(release.check_versions(root), "0.2.0-rc.1")
                with (root / "Cargo.lock").open("rb") as stream:
                    lock = release.tomllib.load(stream)
                self.assertEqual(lock["package"][0]["version"], "0.2.0-rc.1")
                npm_lock = release.read_json(root / "playground" / "package-lock.json")
                self.assertEqual(npm_lock["packages"]["../crates/wasm/pkg"]["version"], "0.2.0-rc.1")
                self.assertEqual(npm_lock["packages"]["node_modules/example"],
                                 {"version": "7.8.9", "integrity": "unchanged"})


class SnapshotTests(unittest.TestCase):
    def test_preparation_uses_only_committed_sources_and_rejects_dirty_rebuild(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "repo"
            root.mkdir()
            (root / "Cargo.toml").write_text('[workspace.package]\nversion = "0.1.0"\n')
            (root / "playground").mkdir()
            (root / "crates").mkdir()
            release.write_json(root / "playground" / "package.json", {"version": "0.1.0"})
            release.write_json(root / "playground" / "package-lock.json",
                               {"version": "0.1.0", "packages": {"": {"version": "0.1.0"}}})
            (root / "releases").mkdir()
            (root / "releases" / "v0.1.0.md").write_text("Initial release\n")
            for command in (("init", "--quiet"), ("add", "."),
                            ("-c", "user.name=Release test", "-c", "user.email=test@example.invalid",
                             "commit", "--quiet", "-m", "Release fixture")):
                subprocess.run(["git", *command], cwd=root, check=True)
            (root / ".env").write_text("DO_NOT_DISTRIBUTE=secret\n")
            output = Path(tmp) / "bundle"
            args = argparse.Namespace(output=output, components=[], e2e=False)
            with patch.object(release, "ROOT", root):
                release.prepare(args)
                manifest = release.verify_bundle(output)
                source = next(name for name, info in manifest["assets"].items() if info["kind"] == "source")
                extracted = Path(tmp) / "extracted"
                release.unpack(output / source, extracted)
                self.assertFalse(list(extracted.rglob(".env")))
                self.assertEqual((extracted / "solcore-v0.1.0" / "releases" / "v0.1.0.md").read_text(),
                                 "Initial release\n")
                (root / "releases" / "v0.1.0.md").write_text("Uncommitted change\n")
                args.output = Path(tmp) / "dirty"
                with self.assertRaisesRegex(release.ReleaseError, "Commit tracked changes"):
                    release.prepare(args)
                self.assertFalse(args.output.exists())


if __name__ == "__main__":
    unittest.main()
