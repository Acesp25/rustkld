# RustKLD
FreeBSD Rust Kernel Drivers

Based on the works of Johannes Lundberg and David Young. 
Planning on expanding this with different drivers as I learn more.

[![Build Status](https://github.com/Acesp25/rustkld/actions/workflows/vmactions.yml/badge.svg)](https://github.com/Acesp25/rustkld/actions/workflows/vmactions.yml)

See [Overview](docs/overview.md) to get started learning about this enviroment's structure, how it works, and how to build your own driver.

## Setup
1. Clone the repo
2. Install rust via Rustup: https://rustup.rs/
3. Install llvm19: ```pkg install llvm19```
4. do ```rustup component add rust-src```
5. do ```cargo install cargo-make``` 

## Running
### Hello World Driver
1. cd into drivers/hello , generate the bindings & build the kld by: ```cargo make build-kmod``` 
2. Load the kld: ```kldload ./hello.ko```
3. Check its existance: ```kldstat | grep "hello"```
4. Unload the kld: ```kldunload hello```
5. Clean up: ```make clean && cargo clean```

### Character Driver
1. cd into drivers/char , generate the bindings & build the kld by: ```cargo make build-kmod``` 
2. Load the kld: ```kldload ./char_dev.ko``` 
3. Check its existance: ```kldstat | grep "char_dev"```
4. Check the registration with devfs: ```ls /dev/echo```
5. Write to dev path: ```echo "Hello :D" > /dev/echo```
6. Read echo'ed message: ```cat /dev/echo```
7. Unload the kld: ```kldunload char_dev```
8. Clean up: ```make clean && cargo clean```
