
# Full-Crisis

![icon/full-crisis-icon.128.png](icon/full-crisis-icon.128.png)

Join historic disasters as emergency response personnel and hone your crisis-solving skills while saving the world!


# TODOs

 - [x] Begin an Icon
 - [ ] Make the Icon a lot better
 - [ ] Finish building webpage generator - http://full-crisis.jmcateer.com/ see `update_github_pages.py` for details
 - [ ] Design a "Game" format
    - We want to be able to point at a folder and place _all_ story details there.
    - Multi-lingual design is a plus; we'll make the text storage in a manner that authors will write all languages at the same place as the phrase/event in question.
    - Graphics, music, links between them. GUI Level editor? Graph of plot?
 - [ ] Design some game-file format that will hold relevant data (will be trivial after game format designed)
 - [ ] Write stories
 - [ ] Build release pipeline to use GitHub CI to build releases for:
    - [ ] Web via Github Pages
    - [ ] Windows-x64 `.exe` file
    - [ ] Is it worth researching building a `.app` bundle w/o Apple's toolchain? Probably not worth it.
    - Linux folks are capable of running `cargo run --release`

# Development Dependencies

The following programs must be installed and available on your system's `PATH`

 - `python` (3.10+ or so, anything modern)
 - `git`
 - `cargo`
    - See https://rustup.rs/

For cross-compilation, install `systemd-nspawn`.

The script `run_docker_from_container.py` will download + run an Arch Linux container (rootfs located under `target/docker-on-arch`) with
this folder mounted under `/full-crisis` within the nspawn container. We will then use https://github.com/cross-rs/cross to perform final cross-compilation.

# Runtime Dependencies

If you do NOT have hardware-accelerated OpenGL rendering available (typically my testing VMs), install Mesa which has software-rendering fallback options - https://github.com/pal1000/mesa-dist-win?tab=readme-ov-file#downloads



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

We use Macroquad because it is a simple framework that supports PC + web
 - https://macroquad.rs/

 - https://github.com/cross-rs/cross
 - https://mwalkowski.com/post/container-inception-docker-in-nspawn-container/


# License

The code in this repository is under the GPLv2 license, see `LICENSE.txt` for details.
The auto-upgrade clause has been removed because your legal rights shouldn't have that sort of volatility.

