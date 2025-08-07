
## logs

```
# step1

## install cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

source ~/.cargo/env

# or
# source ~/.cargo/env.fish

cd my-cli
mkdir step1-hello-world
cd step1-hello-world

cargo init --name hello-cli

# after implementation
cargo build
cargo run
cargo run -- --name Alice
```
