# Sudoku Solvers

## What's this?

This is my final project for a non-linear optimization class. It implements
three different algorithms to solve sudoku puzzles, *d'après* [a paper by Eric
C. Chi and Kenneth Lange][paper] which outline the three different techniques,
namely:

 * Backtracking
 * Simulated annealing
 * Alternated projection

Each of these techniques is implemented as a separate binary, respectively
`backtracking`, `annealing` and `projection`.

## Building

The repository is written in [Rust][rust], and consists of a global project
with [cargo workspaces][workspaces] for each of the aforementioned binaries.
Therefore, to build all the binaries in release mode, and assuming `cargo` is
installed, one only needs to

```
cargo build --release
```

The binaries can thereafter be found in `target/release/`.

## .sudoku Format

For interoperability, all of the provided binaries read sudoku input in a
custom-designed `.sudoku` format. The format is visually intuitive, and
examples can be found in [`example.sudoku`](example.sudoku) or
[`simple.sudoku`](simple.sudoku). More information about the format can be
found in [FORMATTING.txt](FORMATTING.txt).

## .schedule Format

The `.schedule` format is used in the simulated annealing solver to specify the
annealing schedule in a portable/cross-language--friendly format. Python
scripts generating the provided example annealing schedules can be found in
[`schelude_gen/`](schedule_gen/). More information about the format can be
found in the help screen of `annealing`; `annealing --help`.

## Sudoku Grep

To more easily check that a given output is correct (or find conflicting
pairs), this repository also includes a "sudoku grep", the source code for
which is in `grep/`, and found, after building, in `target/release/skgrep`.
`skgrep` was made to be easily used with the provided solvers;

```
backtracking input.sudoku | skgrep
annealing input.sudoku input.schedule | tail +2 | skgrep
projection 10_000 1e-6 input.sudoku | tail +2 | skgrep
```

See `skgrep --help` for more information.

## Source Code Quality

Although the code was written with intentions of readability and performance,
it is very likely a relatively naïve attempt at solving the proposed problem.
Furthermore, the project is not commented for the general audience; I have
tried to comment liberally where I think I am doing something less obvious (and
for this check also the `git log`s), but I was not exhaustive or very
descriptive (for example, there are little to no documentation strings). Rust
crimes, namely `unsafe { &mut (*const T as *mut T) }` are also abound in the
`annealing` workspace. Therefore, I cannot recommend using this repository for
anything other than an example implementation.

## License

For the sake of completeness, I have licensed this repository under the  GNU
General Public License v3 License. In [informal terms][tldrlegal], this means

    You may copy, distribute and modify the software as long as you track
    changes/dates in source files. Any modifications to or software including
    (via compiler) GPL-licensed code must also be made available under the GPL
    along with build & install instructions.

The full license text can be found in [`LICENSE.txt`](LICENSE.txt).


[paper]: https://arxiv.org/abs/1203.2295
[rust]: https://www.rust-lang.org/
[workspaces]: https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html
[tldrlegal]: https://tldrlegal.com/license/gnu-general-public-license-v3-(gpl-3)
