
# Full-Crisis

![icon/full-crisis-icon.128.png](icon/full-crisis-icon.128.png)

Join historic disasters as emergency response personnel and hone your crisis-solving skills while saving the world!

## Documentation

- **[Crisis Game Format Documentation](playable-crises/README.md)** - Complete guide to creating and structuring crisis scenarios

# Development Dependencies

The following programs must be installed and available on your system's `PATH`

 - `python` (3.10+ or so, anything modern)
 - `uv` (because it tames python's packaging chaos)
 - `git`
 - `cargo`
    - See https://rustup.rs/
    - `wasm-pack` for web development (cargo install wasm-pack, make sure to zero `RUSTFLAGS=` if you have any set)


# Build: For your machine only

This is the easiest one! Simply build via cargo and find your executable under `./target/release/full-crisis[.exe]`.

```bash
cargo build --release
```

# Build: NSpawn Cross-Compilation

Cross-compilation is supported on Linux machines with `systemd-nspawn` available and a kernel newer than `5.8` (ie built with `cgroup2` or better).

The script `old_scripts/cross-compile-using-arch-container.py` will download + run an Arch Linux container (rootfs located under `target/docker-on-arch`) with
this folder mounted under `/full-crisis` within the nspawn container. We will then use https://github.com/cross-rs/cross to perform final cross-compilation.

```bash
uv run ./old_scripts/cross-compile-using-arch-container.py
```

# Build: Zig Cross-Compilation

At the moment this technique DOES NOT WORK. Zig may be used if `zig` is installed and you are running on a Linux machine.

```bash
uv run ./old_scripts/zig-build-all-targets.py
```

# Build: Jeff's tiny local cloud

If you're at my house and physically connected to a server's switch, run

```bash
uv run lcloud-compile-all.py
```

Which uses 2 KVM VMs to perform native builds on windows and mac, then copies artifacts from `./target` back to your machine.


# Runtime Dependencies

If you do NOT have hardware-accelerated OpenGL rendering available (typically my testing VMs), install Mesa which has software-rendering fallback options - https://github.com/pal1000/mesa-dist-win?tab=readme-ov-file#downloads

# Steps to build for all targets (assuming starting on Arch + local cloud stuff available at `169.254.0.0/16`)

 - `uv run lcloud-compile-all.py host`
 - `uv run update-github-pages.py`

# Build/CI research

```bash
cargo build --timings --release
# \o/ graphs under target/cargo-timings/
```

# Vocabulary

 - Player
    - The actual player playing the game
 - Player Character / Character
    - The character in the game being controlled by the Player
 - Playable Crisis
    - I'm sure we'll end up calling a folder containing plot, characters, and decision data a "Chrisis" but to avoid overloading the word too much we'll stick the adjective "Playable" before it to make clear this is a definition of a game scenario.
    - A Playable Crisis starts with a folder whose specific definition is still being designed, but it will hold graphics, audio, text, and relationships to drive a game plot forward (or backward, or circles; it'll be a graph of some sort).
    - As a reminder the plural of Crisis is Crises (chris-IS / chris-EZ). Let's make sure the right one is used, especially for structure names,

 - TODO rest of things so we don't end up w/ 3 different words describing "the map"/"globe"/"scene" like other projects


# Research and References

 - https://unikraft.org/ - for a potential server?

# Incomplete experiments

 - [ ] MacOS Menu integration
    - Because we use `iced` which does not expose this control, we will need to either use another crate which can hijack `winit` behavior such as `muda` - https://docs.rs/muda/latest/muda/#note-for-winit-or-tao-users
    - or build some crazy multi-process capability but that sounds dumb and bad. TODO research `muda` integration so Macs can have menus!



# License

The code in this repository is under the GPLv2 license, see `LICENSE.txt` for details.
The auto-upgrade clause has been removed because your legal rights shouldn't have that sort of volatility.

