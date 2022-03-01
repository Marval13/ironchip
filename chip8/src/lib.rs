use rand::{thread_rng, Rng};

mod constants;
use constants::{FONT_OFFSET, FONT_SPRITES, MEM_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH};

mod debug;

pub mod error;
use error::ChipError;

/// Returns the hi nibble (four leftmost bits) of a byte
fn hi_nib(b: u8) -> u8 {
    (b & 0xf0) >> 4
}

/// Returns the low nibble (four rightmost bits) of a byte
fn lo_nib(b: u8) -> u8 {
    b & 0x0f
}

/// The main structure.
///
/// It manages all the emulation data, and represents the whole backend.
#[derive(Debug)]
pub struct Chip8 {
    mem: [u8; MEM_SIZE],
    fb: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
    v: [u8; 0x10],
    i: u16,
    dt: u8,
    st: u8,
    pc: u16,
    sp: usize, // should be u8, but eh
    stack: [u16; 16],
    keypad: [bool; 16],
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}

impl Chip8 {
    /// Returns a new instance of the structure.
    ///
    /// The instance structured already has the font sprites loaded in memory,
    /// and the `pc` register set to `0x200`.
    pub fn new() -> Self {
        let mut mem = [0; MEM_SIZE];
        mem[FONT_OFFSET..FONT_OFFSET + FONT_SPRITES.len()].copy_from_slice(&FONT_SPRITES);

        Chip8 {
            mem,
            fb: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            v: [0; 0x10],
            i: 0,
            dt: 0,
            st: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            keypad: [false; 16],
        }
    }

    pub fn reset(&mut self) {
        self.mem = [0; MEM_SIZE];
        self.mem[FONT_OFFSET..FONT_OFFSET + FONT_SPRITES.len()].copy_from_slice(&FONT_SPRITES);
        self.fb = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT];
        self.v = [0; 0x10];
        self.i = 0;
        self.dt = 0;
        self.st = 0;
        self.pc = 0x200;
        self.sp = 0;
        self.stack = [0; 16];
        self.keypad = [false; 16];
    }

    /// Returns true if the buzzer is on.
    pub fn buzzer(&self) -> bool {
        self.st > 0
    }

    /// Sets key `k` as pressed.
    pub fn key_down(&mut self, k: usize) {
        self.keypad[k] = true;
    }

    /// Sets key `k` as depressed.
    pub fn key_up(&mut self, k: usize) {
        self.keypad[k] = false;
    }

    /// Returns the frame buffer.
    pub fn fb(&self) -> &[[bool; SCREEN_WIDTH]; SCREEN_HEIGHT] {
        &self.fb
    }

    /// Loads the given rom in memory.
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), ChipError> {
        if rom.len() > 0xe00 {
            return Err(ChipError::RomTooBig(rom.len()));
        }
        self.mem[0x200..0x200 + rom.len()].copy_from_slice(rom);
        Ok(())
    }

    fn nnn(&self) -> u16 {
        (self.mem[self.pc as usize] as u16 & 0x0f) << 8 | self.mem[self.pc as usize + 1] as u16
    }

    /// Advances the emulation up until the next frame.
    /// Each frame executes `n` instructions.
    pub fn frame(&mut self, n: usize) -> Result<(), ChipError> {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }
        for _ in 0..n {
            //self.step();
            let last_op = self.step()?;
            if last_op & 0xf000 == 0xd000 {
                break;
            }
        }
        Ok(())
    }

    /// Reads and executes the next operation.
    pub fn step(&mut self) -> Result<u16, ChipError> {
        if self.pc as usize > MEM_SIZE {
            return Err(ChipError::PcOutOfBounds(self.pc));
        }
        let hi_op = self.mem[self.pc as usize];
        let lo_op = self.mem[self.pc as usize + 1];
        let op = ((hi_op as u16) << 8) | (lo_op as u16);

        match hi_op & 0xf0 {
            0x00 => match lo_op {
                0xe0 => self.opcode_cls(),
                0xee => self.opcode_ret(),
                _ => return Err(ChipError::UnrecognizedOpcode(op)),
            },
            0x10 => self.opcode_jp(self.nnn()),
            0x20 => self.opcode_call(self.nnn())?,
            0x30 => {
                let x = lo_nib(hi_op) as usize;
                self.opcode_se(x, lo_op);
            }
            0x40 => {
                let x = lo_nib(hi_op) as usize;
                self.opcode_sne(x, lo_op);
            }
            0x50 => {
                let x = lo_nib(hi_op) as usize;
                let y = hi_nib(lo_op) as usize;
                self.opcode_se_r(x, y);
            }
            0x60 => {
                let x = lo_nib(hi_op) as usize;
                self.opcode_ld(x, lo_op);
            }
            0x70 => {
                let x = (hi_op & 0x0f) as usize;
                self.opcode_add(x, lo_op);
            }
            0x80 => {
                let x = lo_nib(hi_op) as usize;
                let y = hi_nib(lo_op) as usize;
                match lo_nib(lo_op) {
                    0x00 => self.opcode_ld_r(x, y),
                    0x01 => self.opcode_or(x, y),
                    0x02 => self.opcode_and(x, y),
                    0x03 => self.opcode_xor(x, y),
                    0x04 => self.opcode_add_r(x, y),
                    0x05 => self.opcode_sub(x, y),
                    0x06 => self.opcode_shr(x, y),
                    0x07 => self.opcode_subn(x, y),
                    0x0e => self.opcode_shl(x, y),
                    _ => return Err(ChipError::UnrecognizedOpcode(op)),
                }
            }
            0x90 => {
                let x = lo_nib(hi_op) as usize;
                let y = hi_nib(lo_op) as usize;
                self.opcode_sne_r(x, y);
            }
            0xa0 => self.opcode_ld_i(self.nnn()),
            0xb0 => self.opcode_jp_r(self.nnn()),
            0xc0 => {
                let x = lo_nib(hi_op) as usize;
                self.opcode_rnd(x, lo_op);
            }
            0xd0 => {
                let x = lo_nib(hi_op) as usize;
                let y = hi_nib(lo_op) as usize;
                let n = lo_nib(lo_op) as usize;
                self.opcode_drw(x, y, n);
            }
            0xe0 => match lo_op {
                0x9e => self.opcode_skp(lo_nib(hi_op) as usize),
                0xa1 => self.opcode_sknp(lo_nib(hi_op) as usize),
                _ => return Err(ChipError::UnrecognizedOpcode(op)),
            },
            0xf0 => {
                let x = lo_nib(hi_op) as usize;
                match lo_op {
                    0x07 => self.opcode_ld_dt(x),
                    0x0a => self.opcode_ld_k(x),
                    0x15 => self.opcode_ld_dt_r(x),
                    0x18 => self.opcode_ld_st(x),
                    0x1e => self.opcode_add_i(x),
                    0x29 => self.opcode_ld_digit(x),
                    0x33 => self.opcode_ld_bcd(x),
                    0x55 => self.opcode_ld_mass_store(x),
                    0x65 => self.opcode_ld_mass_load(x),
                    _ => return Err(ChipError::UnrecognizedOpcode(op)),
                }
            }
            _ => return Err(ChipError::UnrecognizedOpcode(op)),
        }

        self.pc += 2;
        Ok(op)
    }

    fn opcode_cls(&mut self) {
        self.fb = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT];
    }

    fn opcode_ret(&mut self) {
        self.pc = self.stack[self.sp];
        self.sp -= 1;
    }

    fn opcode_jp(&mut self, addr: u16) {
        self.pc = addr;
        self.pc -= 2;
    }

    fn opcode_call(&mut self, addr: u16) -> Result<(), ChipError> {
        if self.sp >= 15 {
            return Err(ChipError::SpOutOfBounds(self.sp));
        }
        self.sp += 1;
        self.stack[self.sp] = self.pc;
        self.pc = addr;
        self.pc -= 2;
        Ok(())
    }

    fn opcode_se(&mut self, x: usize, byte: u8) {
        if self.v[x] == byte {
            self.pc += 2;
        }
    }

    fn opcode_sne(&mut self, x: usize, byte: u8) {
        if self.v[x] != byte {
            self.pc += 2;
        }
    }

    fn opcode_se_r(&mut self, x: usize, y: usize) {
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }

    fn opcode_ld(&mut self, x: usize, byte: u8) {
        self.v[x] = byte;
    }

    fn opcode_add(&mut self, x: usize, byte: u8) {
        self.v[x] = self.v[x].wrapping_add(byte);
    }

    fn opcode_ld_r(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
    }

    fn opcode_or(&mut self, x: usize, y: usize) {
        self.v[x] |= self.v[y];
    }

    fn opcode_and(&mut self, x: usize, y: usize) {
        self.v[x] &= self.v[y];
    }

    fn opcode_xor(&mut self, x: usize, y: usize) {
        self.v[x] ^= self.v[y];
    }

    fn opcode_add_r(&mut self, x: usize, y: usize) {
        let (res, overflow) = self.v[x].overflowing_add(self.v[y]);
        self.v[0xf] = if overflow { 1 } else { 0 };
        self.v[x] = res;
    }

    fn opcode_sub(&mut self, x: usize, y: usize) {
        let (res, overflow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[0xf] = if overflow { 0 } else { 1 }; // NOT borrow
        self.v[x] = res;
    }

    fn opcode_shr(&mut self, x: usize, _y: usize) {
        // for now y is unused
        self.v[0xf] = self.v[x] & 1;
        self.v[x] >>= 1;
    }

    fn opcode_subn(&mut self, x: usize, y: usize) {
        let (res, overflow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[0xf] = if overflow { 0 } else { 1 }; // NOT borrow
        self.v[x] = res;
    }

    fn opcode_shl(&mut self, x: usize, _y: usize) {
        // for now y is unused
        self.v[0xf] = (self.v[x] >> 7) & 1;
        self.v[x] <<= 1;
    }

    fn opcode_sne_r(&mut self, x: usize, y: usize) {
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }

    fn opcode_ld_i(&mut self, addr: u16) {
        self.i = addr;
    }

    fn opcode_jp_r(&mut self, addr: u16) {
        self.pc = addr + (self.v[0] as u16);
    }

    fn opcode_rnd(&mut self, x: usize, byte: u8) {
        self.v[x] = thread_rng().gen_range(0..=0xff) & byte;
    }

    fn opcode_drw(&mut self, x: usize, y: usize, n: usize) {
        let bytes = &self.mem[(self.i as usize)..(self.i as usize) + n];
        self.v[0xf] = 0;
        let x = (self.v[x] as usize) % SCREEN_WIDTH;
        let y = (self.v[y] as usize) % SCREEN_HEIGHT;

        for (j, byte) in bytes.iter().enumerate() {
            let p_y = y + j;
            if p_y >= SCREEN_HEIGHT {
                break;
            }
            for i in 0..8 {
                let p_x = x + i;
                if p_x >= SCREEN_WIDTH {
                    break;
                }
                let p_mask = ((byte >> (7 - i)) & 1) == 1;
                if self.fb[p_y][p_x] && p_mask {
                    self.v[0xf] = 1;
                }
                self.fb[p_y][p_x] ^= p_mask;
            }
        }
    }

    fn opcode_skp(&mut self, x: usize) {
        if self.keypad[self.v[x] as usize] {
            self.pc += 2;
        }
    }

    fn opcode_sknp(&mut self, x: usize) {
        if !self.keypad[self.v[x] as usize] {
            self.pc += 2;
        }
    }

    fn opcode_ld_dt(&mut self, x: usize) {
        self.v[x] = self.dt;
    }

    fn opcode_ld_k(&mut self, x: usize) {
        if self.keypad.iter().all(|&e| !e) {
            self.pc -= 2;
        } else {
            let press = self
                .keypad
                .iter()
                .enumerate()
                .filter(|(_, &p)| p)
                .map(|(i, _)| i)
                .next()
                .unwrap();
            self.v[x] = press as u8;
        }
    }

    fn opcode_ld_dt_r(&mut self, x: usize) {
        self.dt = self.v[x];
    }

    fn opcode_ld_st(&mut self, x: usize) {
        self.st = self.v[x];
    }

    fn opcode_add_i(&mut self, x: usize) {
        self.i += self.v[x] as u16;
    }

    fn opcode_ld_digit(&mut self, x: usize) {
        self.i = FONT_OFFSET as u16 + 5 * self.v[x] as u16;
    }

    fn opcode_ld_bcd(&mut self, x: usize) {
        let i = self.i as usize;
        self.mem[i] = self.v[x] / 100;
        self.mem[i + 1] = (self.v[x] % 100) / 10;
        self.mem[i + 2] = self.v[x] % 10;
    }

    fn opcode_ld_mass_store(&mut self, x: usize) {
        let i = self.i as usize;
        for r in 0..=x {
            self.mem[i + r] = self.v[r];
        }
    }

    fn opcode_ld_mass_load(&mut self, x: usize) {
        let i = self.i as usize;
        for r in 0..=x {
            self.v[r] = self.mem[i + r];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chip_with_rom(rom: &[u8]) -> Chip8 {
        let mut chip = Chip8::new();
        chip.load_rom(rom).expect("error loading rom");
        chip
    }

    #[test]
    fn jump() {
        let mut chip = chip_with_rom(&[0x13, 0x21, 0x00, 0x00, 0x00, 0x00]);
        chip.step().expect("emulation error");
        assert_eq!(chip.pc, 0x321);
    }

    #[test]
    fn call_and_return() {
        let mut chip = chip_with_rom(&[0x22, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xee]);
        chip.step().expect("emulation error");
        assert_eq!(chip.pc, 0x208);

        chip.step().expect("emulation error");
        assert_eq!(chip.pc, 0x202);
    }

    #[test]
    fn opcodes_skp_sknp() {
        let mut chip = chip_with_rom(&[
            0xe2, 0x9e, 0x00, 0x00, 0xe2, 0x9e, 0xe4, 0xa1, 0x00, 0x00, 0xe4, 0xa1, 0x00,
        ]);
        chip.v[2] = 5;
        chip.v[4] = 1;

        chip.key_down(5);
        chip.step().expect("emulation error");
        chip.step().expect("emulation error");
        assert_eq!(chip.pc, 0x204);

        chip.key_up(5);
        chip.step().expect("emulation error");
        assert_eq!(chip.pc, 0x206);

        chip.step().expect("emulation error");
        assert_eq!(chip.pc, 0x20a);

        chip.key_down(1);
        chip.step().expect("emulation error");
        assert_eq!(chip.pc, 0x20c);
    }

    #[test]
    fn load_from_keypad() {
        let mut chip = chip_with_rom(&[0xf0, 0x0a, 0x00, 0x00]);
        chip.step().expect("emulation error");
        chip.step().expect("emulation error");
        chip.step().expect("emulation error");
        chip.step().expect("emulation error");
        assert_eq!(chip.pc, 0x200);

        chip.key_down(5);
        chip.key_down(8);
        chip.step().expect("emulation error");
        assert_eq!(chip.pc, 0x202);
        assert_eq!(chip.v[0], 5);
    }

    #[test]
    fn mass_store() {
        let mut chip = chip_with_rom(&[0xf3, 0x55]);
        chip.i = 0x220;
        chip.v[0] = 0x06;
        chip.v[1] = 0x0c;
        chip.v[2] = 0x16;
        chip.v[3] = 0x1c;
        chip.v[4] = 0x26;

        chip.step().expect("emulation error");
        assert_eq!(chip.mem[0x220], 0x06);
        assert_eq!(chip.mem[0x221], 0x0c);
        assert_eq!(chip.mem[0x222], 0x16);
        assert_eq!(chip.mem[0x223], 0x1c);
        assert_eq!(chip.mem[0x224], 0x00);
    }

    #[test]
    fn mass_load() {
        let mut chip = chip_with_rom(&[0xf3, 0x65]);
        chip.i = 0x220;
        chip.mem[0x220] = 0x06;
        chip.mem[0x221] = 0x0c;
        chip.mem[0x222] = 0x16;
        chip.mem[0x223] = 0x1c;
        chip.mem[0x224] = 0x26;

        chip.step().expect("emulation error");
        assert_eq!(chip.v[0], 0x06);
        assert_eq!(chip.v[1], 0x0c);
        assert_eq!(chip.v[2], 0x16);
        assert_eq!(chip.v[3], 0x1c);
        assert_eq!(chip.v[4], 0x00);
    }

    #[test]
    fn shift_right_left() {
        let mut chip = chip_with_rom(&[0x82, 0x36, 0x86, 0x3e]);
        chip.v[2] = 0b10101010;
        chip.v[3] = 0b10101010;
        chip.v[6] = 0b10101010;

        chip.step().expect("emulation error");
        assert_eq!(chip.v[2], 0b01010101);
        assert_eq!(chip.v[0xf], 0);
        assert_eq!(chip.v[3], 0b10101010);

        chip.step().expect("emulation error");
        assert_eq!(chip.v[6], 0b01010100);
        assert_eq!(chip.v[0xf], 1);
        assert_eq!(chip.v[3], 0b10101010);
    }
}
