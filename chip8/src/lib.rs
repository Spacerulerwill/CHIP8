use rand::random;

const FONTSET_SIZE: usize = 80;

// Each character is 5 rows of 8 pixels, but amount of rows can vary
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const V_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

#[derive(Debug)]
pub struct Chip8 {
    program_counter: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_registers: [u8; V_REGISTERS],
    i_register: u16,
    delay_timer_register: u8,
    sound_timer_register: u8,
    stack_pointer: u8,
    stack: [u16; STACK_SIZE],
    key_states: [bool; NUM_KEYS],
}

// All chip 8 programs start at 0x200 because historically, the intepreter itself was stored in the first 512 bytes
const START_ADDR: u16 = 0x200;

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Self {
            program_counter: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_registers: [0; V_REGISTERS],
            i_register: 0,
            delay_timer_register: 0,
            sound_timer_register: 0,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            key_states: [false; NUM_KEYS],
        };
        chip8.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        chip8
    }

    /// Tick and execute an instruction
    pub fn tick(&mut self) {
        let opcode = self.fetch();
        self.execute(opcode);
    }

    /// Runs every frame - count down timers
    pub fn tick_timers(&mut self) {
        if self.delay_timer_register > 0 {
            self.delay_timer_register -= 1;
        }

        if self.sound_timer_register > 0 {
            self.sound_timer_register -= 1;
            if self.sound_timer_register == 0 {
                // BEEP
            }
        }
    }

    fn fetch(&mut self) -> u16 {
        let high_byte = self.ram[self.program_counter as usize] as u16;
        let low_byte = self.ram[(self.program_counter + 1) as usize] as u16;
        let opcode = (high_byte << 8) | low_byte;
        self.program_counter += 2;
        opcode
    }

    fn execute(&mut self, opcode: u16) {
        let digit1 = (opcode & 0xF000) >> 12;
        let digit2 = (opcode & 0x0F00) >> 8;
        let digit3 = (opcode & 0x00F0) >> 4;
        let digit4 = opcode & 0x000F;
        match (digit1, digit2, digit3, digit4) {
            // 0000 - NOP - no operation
            (0x0, 0x0, 0x0, 0x0) => return,
            // 00E0 - CLS - Clear the display
            (0x0, 0x0, 0xE, 0x0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            // 00EE - RET - Return from a subroutine
            (0x0, 0x0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.program_counter = ret_addr;
            }
            // 1nnn - JP addr - Jump to location nnn
            (0x1, _, _, _) => {
                let nnn = opcode & 0x0FFF;
                self.program_counter = nnn;
            }
            // 2nnn- CALL addr - Call subroutine at nnn
            (0x2, _, _, _) => {
                let nnn = opcode & 0x0FFF;
                self.push(self.program_counter);
                self.program_counter = nnn;
            }
            // 3xkk - SE Vx, byte - Skip next instruction if Vx = kk
            (0x3, _, _, _) => {
                let register = digit2 as usize;
                let byte = (opcode & 0x00FF) as u8;
                if self.v_registers[register] == byte {
                    self.program_counter += 2;
                }
            }
            // 4xkk - SNE Vx, byte - Skip next instruction if Vx != kk
            (0x4, _, _, _) => {
                let register = digit2 as usize;
                let byte = (opcode & 0x00FF) as u8;
                if self.v_registers[register] != byte {
                    self.program_counter += 2;
                }
            }
            // 5xy0 - SE Vx, Vy - Skip next instruction if Vx = Vy
            (0x5, _, _, 0x0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_registers[x] == self.v_registers[y] {
                    self.program_counter += 2;
                }
            }
            // 6xkk - LD Vx, byte - Set Vx = kk
            (0x6, _, _, _) => {
                let register = digit2 as usize;
                let byte = (opcode & 0x00FF) as u8;
                self.v_registers[register] = byte;
            }
            // 7xkk - ADD Vx, byte - Set Vx = Vx + kk
            (0x7, _, _, _) => {
                let register = digit2 as usize;
                let value = (opcode & 0x00FF) as u8;
                self.v_registers[register] = self.v_registers[register].wrapping_add(value);
            }
            // 8xy0 - LD Vx, Vy - Store value of register Vy in register Vx
            (0x8, _, _, 0x0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] = self.v_registers[y];
            }
            // 8xy1 - OR Vx, Vy - Set Vx = Vx Or Vy
            (0x8, _, _, 0x1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] = self.v_registers[x] | self.v_registers[y];
            }
            // 8xy2 - AND Vx, Vy - Set Vx = Vx AND Vy
            (0x8, _, _, 0x2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] = self.v_registers[x] & self.v_registers[y];
            }
            // 8xy3 - XOR Vx, Vy - Set Vx = Vx XOR Vy
            (0x8, _, _, 0x3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] = self.v_registers[x] ^ self.v_registers[y];
            }
            // 8xy4 - ADD Vx, Vy - Set Vx = Vx + Vy, Set VF = carry
            (0x8, _, _, 0x4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (new_vx, carry) = self.v_registers[x].overflowing_add(self.v_registers[y]);
                let new_vf = if carry { 1 } else { 0 };
                self.v_registers[x] = new_vx;
                self.v_registers[0xF] = new_vf;
            }
            // 8xy5 - SUB
            (0x8, _, _, 0x5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (new_vx, borrow) = self.v_registers[x].overflowing_sub(self.v_registers[y]);
                self.v_registers[x] = new_vx;
                self.v_registers[0xF] = if borrow { 0 } else { 1 };
            }
            // 8xy6 - SHR Vx - Set VX = Vx >> 1
            (0x8, _, _, 0x6) => {
                let x = digit2 as usize;
                let lsb = self.v_registers[x] & 1;
                self.v_registers[x] >>= 1;
                self.v_registers[0xF] = lsb;
            }
            // 8xy7 - SUBM Vx, Vy - Set Vx = V, Set VF = NOT borrow
            (0x8, _, _, 0x7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (new_vx, borrow) = self.v_registers[y].overflowing_sub(self.v_registers[x]);
                self.v_registers[x] = new_vx;
                self.v_registers[0xF] = if borrow { 0 } else { 1 };
            }
            // 8xyE - SHL Vx - Set Vx = Vx SHL 1
            (0x8, _, _, 0xE) => {
                let x = digit2 as usize;
                let msb = (self.v_registers[x] >> 7) & 1;
                self.v_registers[x] <<= 1;
                self.v_registers[0xF] = msb;
            }
            // 9xy0 - SNE Vx, Vy - Skip next instruction if Vx != Vy
            (0x9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_registers[x] != self.v_registers[y] {
                    self.program_counter += 2;
                }
            }
            // Annn - LD I, addr - Set I = nnn
            (0xA, _, _, _) => {
                let nnn = opcode & 0x0FFF;
                self.i_register = nnn;
            }
            // Bnnn - JP V0, addr - Jump to location nnn + V0
            (0xB, _, _, _) => {
                let nnn = opcode & 0x0FFF;
                self.program_counter = nnn + (self.v_registers[0] as u16);
            }
            // Cxkk - RND Vx, byte - Set Vx = random byte AND kk
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let kk = (opcode & 0x00FF) as u8;
                let byte: u8 = random();
                self.v_registers[x] = byte & kk;
            }
            // Dxyn - DRW Vx, Vy, nibble - Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
            (0xD, _, _, _) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let n = digit4;

                // (x, y) coordinate for sprite
                let x_coord = self.v_registers[x] as u16;
                let y_coord = self.v_registers[y] as u16;

                let mut flipped = false;
                for y in 0..n {
                    let addr = self.i_register + y as u16;
                    let pixels = self.ram[addr as usize];

                    for x in 0..8 {
                        // Use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b1000_0000 >> x)) != 0 {
                            // Sprites should wrap around screen, so apply modulo
                            let x = (x_coord + x) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y) as usize % SCREEN_HEIGHT;

                            // Get our pixel's index for our 1D screen array
                            let idx = x + SCREEN_WIDTH * y;
                            // Check if we're about to flip the pixel and set
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                if flipped {
                    self.v_registers[0xF] = 1;
                } else {
                    self.v_registers[0xF] = 0;
                }
            }
            // Ex9E - SKP Vx - Skip next instruction if key with the value of Vx is pressed
            (0xE, _, 0x9, 0xE) => {
                let x = digit2 as usize;
                if self.key_states[self.v_registers[x] as usize] {
                    self.program_counter += 2;
                }
            }
            // ExA1 - SKNP Vx - Skip next instruction if key with the value of Vx is not pressed
            (0xE, _, 0xA, 0x1) => {
                let x = digit2 as usize;
                if !self.key_states[self.v_registers[x] as usize] {
                    self.program_counter += 2;
                }
            }
            // Fx07 - LD Vx, DT - Set Vx = delay timer value
            (0xF, _, 0x0, 0x7) => {
                let x = digit2 as usize;
                self.v_registers[x] = self.delay_timer_register;
            }
            // Fx0A - LD Vx, K - Wait for a key press, store the value of the key in Vx
            (0xF, _, 0x0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.key_states.len() {
                    if self.key_states[i] {
                        self.v_registers[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.program_counter -= 2;
                }
            }
            // Fx15 - LD DT, Vx - Set delay timer = Vx
            (0xF, _, 0x1, 0x5) => {
                let x = digit2 as usize;
                self.delay_timer_register = self.v_registers[x];
            }
            // Fx18 - LD ST, Vx - Set sound timer = Vx
            (0xF, _, 0x1, 0x8) => {
                let x = digit2 as usize;
                self.sound_timer_register = self.v_registers[x];
            }
            // Fx1E - ADD I, Vx - Set I = I + Vx
            (0xF, _, 0x1, 0xE) => {
                let x = digit2 as usize;
                self.i_register = self.i_register.wrapping_add(self.v_registers[x] as u16);
            }
            // Fx29 - LD F, Vx - Set I = location of sprite for digit Vx
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_registers[x] as u16;
                self.i_register = c * 5;
            }
            // Fx33 - LD B, Vx - Store BCD representation of Vx in memory locations I, I+1, I+2
            (0xF, _, 0x3, 0x3) => {
                let x = digit2 as usize;
                let vx = self.v_registers[x];
                let hundreds = vx / 100;
                let tens = (vx / 10) % 10;
                let digits = vx % 10;
                self.ram[self.i_register as usize] = hundreds;
                self.ram[(self.i_register + 1) as usize] = tens;
                self.ram[(self.i_register + 2) as usize] = digits;
            }
            // Fx55 - LD [I], Vx - Store registers V0 through Vx in memory starting at location I
            (0xF, _, 0x5, 0x5) => {
                let x = digit2 as usize;
                for i in 0..=x {
                    self.ram[self.i_register as usize + i] = self.v_registers[i];
                }
            }
            // Fx65 - LD Vx, [I] - Read registers V0 through Vx from memory starting at location I
            (0xF, _, 0x6, 0x5) => {
                let x = digit2 as usize;
                for i in 0..=x {
                    self.v_registers[i] = self.ram[self.i_register as usize + i];
                }
            }
            _ => panic!(
                "Invalid opcode: {:#06x} at address {}",
                opcode, self.program_counter
            ),
        }
    }

    fn push(&mut self, val: u16) {
        self.stack[self.stack_pointer as usize] = val;
        self.stack_pointer += 1
    }

    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.key_states[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }
}
