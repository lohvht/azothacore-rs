# Azothacore

Azothacore is an attempt at rewriting an World of Warcraft emulator based on TrinityCore/Azerothcore
tagged to version 7.3.5 (rev 269672), using Rust.

7.3.5 source reference is based on TrinityCore's [7.3.5/269672](https://github.com/TrinityCore/TrinityCore/tree/7.3.5/26972) release.

Referencing Database files notes from here: https://github.com/TrinityCore/TrinityCore/commit/7f2b7dc9c2165d2608742473a931f55b1c1a753a

It is completely open source and still a work in progress; Community involvement is highly encouraged.

Run the following to clone the repo, including submodules.

```
git clone --recurse-submodules git@github.com:lohvht/azothacore-rs.git
```

# Requirements

Ensure that you're on a *NIGHTLY* toolchain for rust as the project uses a few features that are not
stable yet.

There are several other dependencies required to compile this project other than a **nightly** build
due to the project's dependencies on non-rust components. These are the crates in the `crates/`
directory ending with `-sys`. As well as [recastnavigation-sys](https://github.com/andriyDev/recastnavigation-rs-sys)

These are the following requirements needed at the moment:
- cmake
- clang
- C/C++ compiler

For `casclib-sys`, Cmake and a C++ compiler is required to compile [CascLib](https://github.com/ladislav-zezula/CascLib)
while clang is required to parse the C++ headers to generate bindings to CascLib.

A C++ compiler and clang is also required to compile recastnavigation-sys.

TODO: FILL THIS IN WITH CMAKE/C++ REQUIREMENTS AND TEST OUT BUILDS FOR WINDOWS/Other Systems

# Roadmap checklist (To be updated as it goes)
- [x] Extractors/Generators
    - [x] Map & DB2
    - [x] Vmap4 Extractor
    - [x] Vmap4 Assembler
    - [x] Mmap Generator
- [ ] CI/CD helpers
    - [ ] Database pending updates / archive / etc (to prevent clashes)
    - [ ] CI lint / format
    - [ ] compile check for windows / linux / macos
    - [ ] Tests + Coverage
      - [x] Coverage backbone code
- [ ] Authserver netcode
    - [x] Login REST service
    - [x] Auth session handling
    - [ ] SSL/TLS cert patching / override
    - [ ] Test can login to login screen in UI
      - [x] Can login and see RealmList
- [ ] Loading game assets
- [ ] Loading DB fixes
- [x] Migrations / DB reloader
- [ ] Worldserver netcode
- [ ] ...


# Special Thanks
Special Thanks goes out to the authors of the following World of Warcraft emulators for providing a source to
reference for this following implementation
- The authors of AzerothCore - https://github.com/azerothcore/azerothcore-wotlk
- The authors of TrinityCore - https://github.com/TrinityCore/TrinityCore
