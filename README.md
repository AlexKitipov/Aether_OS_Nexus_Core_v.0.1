# Aether OS Nexus Core (v0.1)

AetherOS Nexus Core v0.1 is a Rust-based hybrid microkernel concept focused on security, modularity, capability-based isolation, and V-Node services.

## Repository structure

```text
.
├─ README.md
├─ LICENSE
└─ docs/
   ├─ specs/
   │  ├─ core-documentation-v0.1.0.txt
   │  └─ overview-v0.1.txt
   └─ releases/
      ├─ Aether_OS_Nexus_Core_v.0.1.pdf
      └─ AetherOS_Nexus_Core_v0.1.0.zip
```

## What was reorganized

- Moved long-form technical text documents into `docs/specs/`.
- Moved release artifacts (PDF + ZIP) into `docs/releases/`.
- Normalized the ZIP filename to avoid spaces/special punctuation.

## Notes

The documentation currently describes a broader workspace layout (`kernel/`, `vnode/`, `src/`, etc.).
Those source directories are not present in this snapshot repository yet; this repo currently contains the foundational documentation and packaged artifacts for v0.1.
