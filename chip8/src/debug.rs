use crate::constants::{MEM_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::error::DebugChipError;
use crate::Chip8;

/// The debug functions.
impl Chip8 {
    /// Returns a copy of the memory.
    pub fn get_mem(&self) -> [u8; MEM_SIZE] {
        self.mem
    }

    /// Returns a copy of the register array.
    pub fn get_regs(&self) -> [u8; 0x10] {
        self.v
    }

    /// Returns the program counter, the stack pointer and the index register
    /// in this order.
    pub fn get_pointers(&self) -> (u16, usize, u16) {
        (self.pc, self.sp, self.i)
    }

    /// Returns the delay timer and the sound timer, in thi order.
    pub fn get_timers(&self) -> (u8, u8) {
        (self.dt, self.st)
    }

    /// Returns a copy of the stack.
    pub fn get_stack(&self) -> [u16; 16] {
        self.stack
    }

    /// Returns the keypad status.
    pub fn get_keypad(&self) -> [bool; 16] {
        self.keypad
    }

    /// Writes a value at the given memory address.
    pub fn set_mem(&mut self, addr: usize, val: u8) -> Result<(), DebugChipError> {
        if addr > 0xfff {
            return Err(DebugChipError::AddrOutOfBounds(addr));
        }
        self.mem[addr] = val;

        Ok(())
    }

    /// Writes a pixel on the frame buffer.
    /// Does not compute collision.
    pub fn set_fb(&mut self, x: usize, y: usize, pixel: bool) -> Result<(), DebugChipError> {
        if x >= SCREEN_WIDTH || y >= SCREEN_HEIGHT {
            return Err(DebugChipError::NoPixel(x, y));
        }
        self.fb[y][x] = pixel;

        Ok(())
    }

    /// Writes a value in the given register.
    pub fn set_reg(&mut self, reg: usize, val: u8) -> Result<(), DebugChipError> {
        if reg > 0x10 {
            return Err(DebugChipError::NoRegister(reg));
        }
        self.v[reg] = val;

        Ok(())
    }

    /// Sets the index register.
    pub fn set_i(&mut self, val: u16) -> Result<(), DebugChipError> {
        if val & 0xf000 != 0 {
            return Err(DebugChipError::IndexTooBig(val));
        }

        self.i = val;
        Ok(())
    }

    /// Sets the delay timer.
    pub fn set_dt(&mut self, val: u8) {
        self.dt = val;
    }

    /// Sets the sound timer.
    pub fn set_st(&mut self, val: u8) {
        self.st = val;
    }

    /// Sets the program counter.
    pub fn set_pc(&mut self, val: u16) -> Result<(), DebugChipError> {
        if val & 0xf000 != 0 {
            return Err(DebugChipError::PcOutOfBounds(val));
        }

        self.pc = val;
        Ok(())
    }

    /// Sets the stack pointer.
    pub fn set_sp(&mut self, val: usize) -> Result<(), DebugChipError> {
        if val > 15 {
            return Err(DebugChipError::SpOutOfBounds(val));
        }

        self.sp = val;
        Ok(())
    }

    /// Writes a value on the stack at the given position.
    pub fn set_stack(&mut self, pos: usize, val: u16) -> Result<(), DebugChipError> {
        if pos > 15 {
            return Err(DebugChipError::SpOutOfBounds(pos));
        }
        if val & 0xf000 > 0 {
            return Err(DebugChipError::StackAddrOutOfBounds(val));
        }

        self.stack[pos] = val;
        Ok(())
    }
}
