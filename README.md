# mathcli
Have you ever been grapping logs, and gotten it down to a stream of numbers, and wished you could sum all those numbers? Now you can!

Alternatives are ugly: https://stackoverflow.com/questions/450799/shell-command-to-sum-integers-one-per-line

And this is simple!
```
$ printf '2\n3\n\n' | mathcli mul
$ 6
```

### Install
1. Install cargo: https://doc.rust-lang.org/cargo/getting-started/installation.html
2. `cargo install mathcli`

### Contributing
Feel free to make a CR.
