# Iron Chip
Iron Chip is a Chip-8 interpreter written in Rust.

The backend provides the emulation and some simple tools for debugging.
It expects the frontend to call the `Chip8::frame(n)` function 60 times a second, passing as argument the number of instructions that will be executed each frame.

The frontend uses SDL2 to provide video and audio, and to process input.
It is mostly adapted from the SDL2 examples.

## Usage
Drag and drop a ROM on the window to open it.

The keypad scheme is mapped to the keyboard as follows:
```
   +---+---+---+---+     +---+---+---+---+
   | 1 | 2 | 3 | C |     | 1 | 2 | 3 | 4 |
   +---+---+---+---+     +---+---+---+---+
   | 4 | 5 | 7 | D |     | Q | W | E | R |
   +---+---+---+---+     +---+---+---+---+
   | 7 | 8 | 9 | E |     | A | S | D | F |
   +---+---+---+---+     +---+---+---+---+
   | A | 0 | B | F |     | Z | X | C | V |
   +---+---+---+---+     +---+---+---+---+
```
The `P` button pauses and unpauses emulation.

Iron Chip can also be used from command line: use the `--help` flag to check the available commands.

## Sources
The specifications were taken from [Cowgod's Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM) and this excellent [guide by Tobias Langhoff](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/).