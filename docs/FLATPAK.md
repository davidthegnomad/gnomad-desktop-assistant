# Flatpak packaging — Gnomad Desktop Assistant

Gnomad’s primary Linux artifacts are **`.deb`**, **`.rpm`**, and **AppImage** from Tauri CI. **Flatpak** is an optional community/store format for sandboxed desktop distribution.

---

## Prerequisites

On a Linux host with Flatpak installed:

```bash
sudo apt install flatpak flatpak-builder   # Debian/Ubuntu
flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install flathub org.freedesktop.Platform//23.08 org.freedesktop.Sdk//23.08 \
  org.freedesktop.Sdk.Extension.node20//23.08 org.freedesktop.Sdk.Extension.rust-stable//23.08
```

---

## Build from source

From the repository root:

```bash
bash packaging/flatpak/build.sh          # build only
bash packaging/flatpak/build.sh --install # build + user install
```

Manifest: [`packaging/flatpak/com.gnomadstudio.Gnomad.yml`](../packaging/flatpak/com.gnomadstudio.Gnomad.yml)

First build compiles the full Tauri app inside the Flatpak SDK (15–30+ minutes).

After install:

```bash
flatpak run com.gnomadstudio.Gnomad
```

Export a single-file bundle for sideloading:

```bash
flatpak build-bundle packaging/flatpak/repo gnomad.flatpak com.gnomadstudio.Gnomad
```

---

## Permissions (finish-args)

The manifest requests:

| Permission | Why |
|------------|-----|
| `home` | Workspace, knowledge library, chat history |
| `network` | Cloud LLM APIs, updates |
| `wayland` / `x11` | UI |
| `secrets` | API keys via libsecret |

**Agent shell tools** require broad filesystem access under `--filesystem=home`. Flathub review may ask for justification — document that Gnomad is a desktop automation assistant.

---

## Flathub submission

1. Fork [flathub/flathub](https://github.com/flathub/flathub) and add a PR with this manifest (often as a separate repo + `flatpak-builder` JSON).
2. Include [`com.gnomadstudio.Gnomad.metainfo.xml`](../packaging/flatpak/com.gnomadstudio.Gnomad.metainfo.xml).
3. Pin sources to release tags rather than `type: dir` for production.

---

## Related

- [LINUX_PACKAGES.md](LINUX_PACKAGES.md) — deb/rpm/AppImage (recommended default)
- [BUILD_PLATFORMS.md](BUILD_PLATFORMS.md) — native Tauri builds
- [RELEASE_RUNBOOK.md](RELEASE_RUNBOOK.md) — tagged releases

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
