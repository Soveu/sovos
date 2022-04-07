# `sovos`

An x86-64 kernel written in Rust

## Dependencies
This project is as dependency-free as possible.
Nightly Rust is a must (it rhymes, lol),
`miri` and `clippy` might be useful for dev purposes.

### Building
Currently, for building you would only need `uart_16550` and `arrayvec`.

### Running
For running, it needs `qemu-system-x86_64` and UEFI image under `/usr/share/edk2/ovmf/OVMF_CODE.fd`


## Commands
### Building
Just `cargo xtask build`!
It creates a directory named `fat/`, that is later attached to QEMU.

### Running
Running is also simple, `cargo xtask run`.
QEMU is started with `-nographic` mode and serial output is directed to stdout.
If you want to attach GDB to the VM (`-s` and `-S`), you would need to uncomment
additional parameters in `xtask/src/main.rs`

It is possible to copy the contents of `fat/` directory into a real FAT32 drive
and run it on real hardware with UEFI, but there are currently no guarantees that
it will work (mostly because serial port is hardcoded to 0x3F8 and EFI text
protocols usage is limited as for now)

### Cleaning
To remove _all_ the artifacts, run `cargo xtask clean all`


## Structure
* `kernel/` contains, well, the actual kernel code (work has not started there [YET!])
* `uefi_wrapper/` is a thing that performs initial setup, gathers information
  about hardware, (will probably) decompress kernel binary and jump to it
* `libs/` is anything that can be separated, reused or just requires testing
  that would otherwise be much harder to do under kernel environment
* `xtask/` is the "script" that makes building and running easier

Inside those directories, there should recursively be readmes about useful stuff


## Philosophy
There aren't many rules, just
* Take one detail from a problem and try to think about a good way to solve it
* Explore new ideas, don't restrict yourself to the known
* Think of scalability, if the solution could handle tens of cores,
  terabytes of memory, thousands of processes


## Todolist (kinda?)
- [ ] Documentation
  - [x] This readme
  - [ ] Document `kernel/`
  - [ ] Document `libs/`
  - [ ] Document `libs/freetree`
- [ ] Physical memory allocator
  - [x] The POC is finished! (`libs/freetree/src/poc.rs`)
  - [ ] Work on "flavors" of the POC
  - [ ] <IDEA> Maybe we could write a parallel skiplist?
- [ ] `uefi_wrapper`
  - [ ] Use UEFI text protocol if possible before exiting boot services
  - [ ] Check CPU features and capabilities
        (we assume at least Sandybridge features, see `kernel/README.md`)
  - [ ] Figure out if it is possible to just load the kernel ELF into memory
        and jump into it
  - [ ] After writing a memory allocator, we could actually jump into kernel
- [ ] Virtual memory mapping
  - [ ] The basic structures for x64 are there, but I think I have an idea for a rewrite
- [ ] VirtIO
  - [ ] I'd like to work on network things first, to have a UDP or maybe even
        TCP stack to connect to instead of using serial port
  - [ ] Block devices
- [ ] Support for RISC-V
  - [ ] Support for Rv39, 48 and 57
  - [ ] Other things needed in `libs/cpu`
  - [ ] Idk, figure out what is needed to boot
