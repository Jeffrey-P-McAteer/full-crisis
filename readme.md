
# Full-Crisis

![icon/full-crisis-icon.128.png](icon/full-crisis-icon.128.png)

Join historic disasters as emergency response personnel and hone your crisis-solving skills while saving the world!


# TODOs

 - [x] Begin an Icon
 - [ ] Make the Icon a lot better
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


# Research and References

We use Macroquad because it is a simple framework that supports PC + web
 - https://macroquad.rs/


# License

The code in this repository is under the GPLv2 license, see `LICENSE.txt` for details.
The auto-upgrade clause has been removed because your legal rights shouldn't have that sort of volatility.

