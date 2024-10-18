import hashlib
import os
import subprocess
import tempfile
from dataclasses import dataclass

@dataclass
class Patch:
    path: str
    before: bytes
    after: bytes

@dataclass
class PatchTarget:
    name: str
    version: str
    hash: str
    patches: list[Patch]

    def url(self) -> str:
        return f"https://crates.io/api/v1/crates/{self.name}/{self.version}/download"

targets: list[PatchTarget] = [
    PatchTarget(name="sha2", version="0.10.8", hash="793db75ad2bcafc3ffa7c68b215fee268f537982cd901d132f89c6343f3a3dc8", patches=[
        # restore asm for fast sha256 on aarch64
        # backport of https://github.com/RustCrypto/hashes/commit/1f792033dc24fb2a5f8e6f193dce477040138fd4
        Patch(path="src/sha256.rs", before=b'#[cfg(all(feature = "asm", target_arch = "aarch64"))]', after=b'#[cfg(target_arch = "aarch64")]'),
    ]),
    PatchTarget(name="grammers-mtproto", version="0.7.0", hash="0d057562ccd5ac7437683534634e469875db27827a7febd42cb6b775ffd01dff", patches=[
        # disable gzip for fast uploading
        Patch(path="src/lib.rs", before=b"\npub const DEFAULT_COMPRESSION_THRESHOLD: Option<usize> = Some(", after=b"\npub const DEFAULT_COMPRESSION_THRESHOLD: Option<usize> = None; // Some("),
    ])
]

# --- patch process
root_dir = "vendor"

os.makedirs(root_dir, exist_ok=False)

for target in targets:
    print(f"Downloading {target.name} v{target.version}")
    with tempfile.TemporaryDirectory(prefix="patch2crate") as td:
        file_path = os.path.join(td, "crate.tar.gz")

        subprocess.run(["curl", "-Lfo", file_path, target.url()]).check_returncode()
        # print(target.url())
        with open(file_path, "rb") as f:
            hash = hashlib.sha256(f.read()).hexdigest()
            if hash != target.hash:
                raise Exception(f"Checksum Mismatch in {target.name} v{target.version} (expected={target.hash}, received={hash})")

        subprocess.run(["tar", "xf", file_path], cwd=root_dir).check_returncode()

        pkg_root_dir = os.path.join(root_dir, target.name + "-" + target.version)

        if not os.path.isdir(pkg_root_dir):
            raise Exception(f"Package doesn't exist in {pkg_root_dir}")
        
        for patch in target.patches:
            print(f"Patching to {patch.path}...")
            with open(os.path.join(pkg_root_dir, patch.path), "r+b") as f:
                content = f.read()
                count = content.count(patch.before)
                if count != 1:
                    raise Exception(f"Failed to patch! can't determinate patch point (count={count})")
                content = content.replace(patch.before, patch.after)
                f.seek(0)
                f.truncate(len(content))
                f.write(content)
            print("Done!")
