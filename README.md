# Kepler

> Round and round we go!

An experimental operating system, developed for fun by a depressed college student. Right now things are in very early stages, but it does boot!

### Trying it Out

Things are still very early, there isn't much to see. If curious however, make sure `qemu` is installed on your system and execute `cargo run`.

### Implementation Status

Note that crates mentioned here are components of this project, found in the `crates/` directory. Any name collisions with existing projects are not intentional.

| Feature | Status |
| --- | --- |
| Boot | Complete (via Limine) |
| Virtual Memory | Partial, a basic allocation mechanism is available but needs to be reworked. (See the `vm-types` crate). |
| Interrupts | Partial, awaiting complete support by the `hal-x86_64` crate. |
| Processes and Scheduling | In progress. The `hal` crate recently gained support for context switching, so the scheduler and task system itself is now being built. (See the `langrange` crate) |
| I/O | Partial, see the `nvme` crate |
| Randomness | Partial. The `entropy` crate provides a complete randomness pool that will be used later. |
| Userspace | lol no|