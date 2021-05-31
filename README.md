# ipynbat

Kinda like `cat` but only for Jupyter Notebooks + it goes full LSD-grade rainbow colors if its stdout looks like a TTY. Simultaneously very different from `cat` because it doesn't concatenate stuff and just displays the first file you give it.

This project is very very WIP and it `panic!`s a lot. Actually there's no proper error handling whatsoever for the time being.

## Features

  - [x] Display a nice terminal representation of a Jupyter notebook
  - [ ] Actually concatenates file like its name suggests
  - [ ] Can run notebooks by itself (W.I.P, currently displays cached cell output)
  - [x] Low-quality rip-off of [`bat`](https://github.com/sharkdp/bat) 's terminal UI style

## Install

`cargo install --git https://github.com/edgarogh/ipynbat` should work.

## Contributions

Don't.

The project's structure is too unstable at the moment, and the code is horrible.

## License & credits

You can enjoy `ipynbat` under the GPL-3.0 license. As jokingly said above, the style and color of its output is more than inspired from the great [`bat`](https://github.com/sharkdp/bat) project (licensed as MIT or Apache-2.0) that also happens to be written in Rust; make sure to install it because it is incredibly more useful, and actually concatenates stuff.
