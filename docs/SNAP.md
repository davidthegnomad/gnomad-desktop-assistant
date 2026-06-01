# Snap packaging — Gnomad Desktop Assistant

**Snap** is optional packaging for Ubuntu and other distros with `snapd`. Gnomad uses **classic confinement** because the agent runs shell commands, accesses the workspace filesystem, and integrates with the system tray.

---

## Prerequisites

```bash
sudo snap install snapcraft --classic
```

Build on **Linux** after Tauri Linux dependencies are installed (see [BUILD_PLATFORMS.md](BUILD_PLATFORMS.md)).

---

## Build

From the repository root:

```bash
bash packaging/snap/build.sh
```

This will:

1. Run `npm run tauri:build:linux:deb` if no `.deb` exists
2. Extract the `.deb` into a classic snap via `snapcraft pack`

Output: `packaging/snap/gnomad.snap`

Install locally (unsigned):

```bash
sudo snap install --dangerous packaging/snap/gnomad.snap
gnomad
```

---

## Manifest

[`packaging/snap/snapcraft.yaml`](../packaging/snap/snapcraft.yaml)

| Setting | Value | Notes |
|---------|-------|-------|
| `confinement` | `classic` | Required for shell agent + full home access |
| `base` | `core22` | Ubuntu 22.04 runtime |
| `grade` | `devel` | Change to `stable` for store submission |

---

## Snap Store (optional)

1. Register the `gnomad` snap name on [snapcraft.io](https://snapcraft.io).
2. Switch to **strict** confinement only if you split agent features into a separate interface — not recommended for this app class.
3. Use `snapcraft upload` after connecting the store.

For most users, **deb** or **AppImage** from GitHub Releases remain the supported path.

---

## Related

- [LINUX_PACKAGES.md](LINUX_PACKAGES.md)
- [FLATPAK.md](FLATPAK.md)
- [RELEASE_RUNBOOK.md](RELEASE_RUNBOOK.md)

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
