# Fuzzing Exim

This repo contains a fuzzer for [Exim](https://exim.org) that aims to be more on the "professional"
side of fuzzing.
The individual SMTP packets are passed to Exim via a shared memory channel, the fuzzer has custom
mutations tailored towards the SMTP protocol and we tried to make the target as scalable as possible
by reducing its system calls to a minimum.

We used this fuzzer to fuzz the pre-release version of Exim 4.99 and could find and fix one vulnerability
and multiple other bugs before they could make it into the final release. See [Findings](#Findings) for more details.

If you want to run this fuzzer yourself, follow the instructions below.

## Build Exim
1. Checkout the `Exim` submodule:
```
git submodule update --init --depth=1 ./Exim
```
2. Create Exim's Makefile by copying [LocalMakefile](./LocalMakefile) to `./Exim/src/Local/Makefile`
   and adjust all variables to your system
3. Apply the patches in [patches](./patches)
4. Compile exim by invoking
```
export EXIM_RELEASE_VERSION="fuzz"
make
```
5. Prepare Exim's runtime by copying [exim.conf](./exim.conf) to the `CONFIGURE_FILE` location and setting its
   permission bits to `640`

## Build libdesock
Checkout the `libdesock` submodule
```
git submodule update --init --depth=1 ./libdesock
```
and copy our custom [hooks.c](./hooks.c) into the `src/` directory of libdesock.
Then, execute:
```
cd libdesock
meson setup ./build
cd build
meson configure -D allow_dup_stdin=true -D multiple_requests=true -D request_delimiter="--------"
meson compile
```

## Build the fuzzer
Go into [fuzzer](./fuzzer) and execute:
```
cargo build --release
```

## Start fuzzing
Create an output directory:
```
mkdir output
```
and invoke the fuzzer
```
./fuzzer/target/release/fuzzer fuzz --output ./output/ --libdesock libdesock/build/libdesock.so --dict ./smtp.dict -- ./Exim/src/build-Linux-x86_64/exim -bdf
```

## Findings
1. Use-After-Free and Double-Free in an error path when too many invalid SMTP commands have been sent. Exim calls
   `fclose()` on multiple FILE handles but can be coerced into using them after closing. Fixed in commit
   [d73d4529ce71ef5c54883bc1a573f8a275630cca](https://github.com/Exim/exim/commit/d73d4529ce71ef5c54883bc1a573f8a275630cca).
2. And of course a couple of NULL dereferences: [Bug #3136](https://bugs.exim.org/show_bug.cgi?id=3136), [Bug #3137](https://bugs.exim.org/show_bug.cgi?id=3137)

