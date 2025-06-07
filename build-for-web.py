# /// script
# requires-python = ">=3.12"
# dependencies = [
#
# ]
# ///


# The plan here is to cross-compile a PE32 32-bit .exe binary,
# then pack a dist/* dir to be loadable from a web browser using
# https://www.boxedwine.org as a runtime.
# If we need web-specific changes such as no animations for web,
# use #[cfg(target_pointer_width = "64")] or some such similar compile-time option.




