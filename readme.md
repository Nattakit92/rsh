# RSH 

RSH is a simple shell written in rust, build for educational purpose. My goal is to learn more about rust and how interpreters work under the hood.

## Feature
- [x] exit 
- [x] cd 
- [x] ls
- [x] pwd 
- [x] echo 
- [x] variables
- [x] touch 
- [x] cat 
- [x] mkdir 
- [x] arithmetic 
- [x] external commands 
- [x] comparisons 
- [x] command substitution 
- [x] pipeline
- [x] config file
- [x] alias
- [x] history
- [x] execute file
- [ ] if else
- [ ] comments
- [ ] command/filename/argument completion

 
## Installation Guide

Make sure you have rust installed.
```
rustc --version
```

### Install through cargo

> cargo install rshl

### Manual installed

```
# clone & open this repo
git clone https://github.com/Nattakit92/rsh.git && cd rsh

# build the from source
cargo build --release

# move rsh to cargo/bin
mv target/release/rsh $HOME/.cargo/bin

# cleaning up
cd .. && rm -rf rsh

```
## Usage

To use rsh simply open the terminal and run `rsh`

## Troubleshooting
> Unknown command: rsh

Check if bin directory exist
```
ls $HOME/.cargo/bin/
```
If .cargo/bin did not exist 
```
mkdir $HOME/.cargo $HOME.cargo/bin
```

Check if bin directory is in PATH environment variable
```
env| grep .cargo/bin
```
If bin directory is not in PATH environment variable.
Add the following line to your shell configuration file and restart your terminal:
```
export $HOME/.cargo/bin
```
