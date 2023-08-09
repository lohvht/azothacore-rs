# Azothacore

Azothacore is an attempt at rewriting an World of Warcraft emulator based on TrinityCore/Azerothcore
tagged to version 7.3.5 (rev 269672), using Rust.

7.3.5 source reference is based on TrinityCore's [7.3.5/269672](https://github.com/TrinityCore/TrinityCore/tree/7.3.5/26972) release.

It is completely open source and still a work in progress; Community involvement is highly encouraged.

# Requirements

Ensure that you're on a *NIGHTLY* toolchain for rust as the project uses a few features that are not
stable yet.

TODO: FILL THIS IN WITH CMAKE/C++ REQUIREMENTS AND TEST OUT BUILDS FOR WINDOWS

# Roadmap checklist (To be updated as it goes)
- [x] Extractors/Generators
    - [x] Map & DB2
    - [x] Vmap4 Extractor
    - [x] Vmap4 Assembler
    - [x] Mmap Generator
- [ ] Loading game assets
- [ ] Loading DB fixes
- [ ] Migrations / DB reloader
- [ ] Authserver netcode
- [ ] Worldserver netcode
- [ ] ...


# Special Thanks
Special Thanks goes out to the authors of the following World of Warcraft emulators for providing a source to
reference for this following implementation
- The authors of AzerothCore - https://github.com/azerothcore/azerothcore-wotlk
- The authors of TrinityCore - https://github.com/TrinityCore/TrinityCore