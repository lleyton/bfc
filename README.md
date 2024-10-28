# bfc

A naive [Brainfuck](https://brainfuck.org/) compiler frontend for [QBE](https://c9x.me/compile/)... since I wanted to play around with it.

## Usage

```sh
cargo run source.b > source.sse
qbe -o source.s source.sse
cc source.s
./a.out
```
