# RSH 

RSH is a simple shell written in rust, build for educational purpose. My goal is to learn more about rust and how interpreters work under the hood.

## Progress
| Feature | Progress |
| ------- | -------- |
| exit | ✅ Done |
| cd | ✅ Done |
| ls | ✅ Done |
| pwd | ✅ Done |
| echo | ✅ Done |
| let | ✅ Done |
| touch | ✅ Done |
| cat | ✅ Done |
| mkdir | ✅ Done |
| arithmetic | ✅ Done |
| external commands | ✅ Done |
| comparisons | ✅ Done |
| command substitution | ✅ Done |
| pipe | ✅ Done |

## Building from source

```
# clone & open this repo
git clone https://github.com/Nattakit92/rsh.git && cd rsh

# try
cargo run

# build the from source
cargo build --release

# move rsh to cargo/bin
mv target/release/rsh $HOME/.cargo/bin

# run rsh
rsh

```
