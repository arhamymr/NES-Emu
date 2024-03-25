// CPU 6802 Flags

use std::ops::Add;

#[derive(Debug, PartialEq)]
pub enum Flag {
    Carry = 0b0000_0001,
    Zero = 0b0000_0010,
    Interrupt = 0b0000_0100,
    Decimal = 0b0000_1000,
    Break = 0b0001_0000,
    Unused = 0b0010_0000,
    Overflow = 0b0100_0000,
    Negative = 0b1000_0000,
}

enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
}

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    pub memory: [u8; 65536],
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 65536],
        }
    }

    fn read_memory(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write_memory(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    // Read memory and merge 2 bytees into 16 bit
    // 0x00FF (low byte) and 0xFF00 (high byte)
    // merge with bitwise OR
    // to get 0xFFFF
    fn read_memory_16bit(&self, address: u16) -> u16 {
        let low_byte = self.read_memory(address) as u16;
        let high_byte = self.read_memory(address + 1) as u16;
        (high_byte << 8) | low_byte
    }

    // split 16 bit data into 2 bytes and write to memory
    // 0xFFFF split into 0x00FF and 0xFF00
    fn write_memory_16bit(&mut self, address: u16, value: u16) {
        let low_byte = value as u8;
        let high_byte = (value >> 8) as u8;

        // little endian format
        // lowbite first and highbyte second
        self.write_memory(address, low_byte);
        self.write_memory(address + 1, high_byte);
    }

    fn load_program_into_memory(&mut self, program: Vec<u8>) {
        for (i, &byte) in program.iter().enumerate() {
            self.write_memory(i as u16, byte);
        }
    }

    fn update_zero_and_negative_flag(&mut self, register: u8) {
        // set Zero flag to register A
        // if register A is = 0b0000_0000
        if register == 0 {
            // Status set as Zero flag (0b0000_0010)
            self.status |= Flag::Zero as u8;
        } else {
            // if register A != 0
            // clear zero flag by using bitwise AND
            self.status &= !(Flag::Zero as u8);
        }

        // if register A is = 0b11100000
        // compare register A with 0b1000_0000
        // 0b1110_0000 & 0b1000_0000 = 0b1000_0000 (128) Negative flag
        // Check MSB (Most Significant Bit) register A set to 1
        // Register A AND Negative Flag != 0 or result equal to 128 (0b1000_0000)
        if register & Flag::Negative as u8 != 0 {
            // Status set as Negative flag
            // Register Status now is for example in zero flag
            // Status = 0b000_0010
            // set negative flag
            // now status is 0b1000_0010
            self.status |= Flag::Negative as u8;
        } else {
            // if current accumulator is not negative
            // clear negative flag by using bitwise AND
            // and flip the negative flag to 0b0111_1111
            // Status = 0b000_0010 (zero flag)
            // 0b0000_0010 & 0b0111_1111 = 0b0000_0010
            self.status &= !(Flag::Negative as u8);
        }
    }

    fn select_addressing_mode(&mut self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.read_memory(self.program_counter) as u16,
            AddressingMode::ZeroPageX => {
                let base = self.read_memory(self.program_counter as u16);
                let zero_page_address = base.wrapping_add(self.register_x);
                zero_page_address as u16
            }
            AddressingMode::ZeroPageY => {
                let base = self.read_memory(self.program_counter);
                let zero_page_address = base.wrapping_add(self.register_y);
                zero_page_address as u16
            }
            AddressingMode::Absolute => {
                let address = self.read_memory_16bit(self.program_counter);
                self.program_counter += 1;
                address
            }
            AddressingMode::AbsoluteX => {
                let address = self.read_memory_16bit(self.program_counter);
                let final_address = address.wrapping_add(self.register_x as u16);
                self.program_counter += 1;
                final_address
            }
            AddressingMode::AbsoluteY => {
                let address = self.read_memory_16bit(self.program_counter);
                let final_address = address.wrapping_add(self.register_y as u16);
                self.program_counter += 1;
                final_address
            }
            AddressingMode::IndirectX => {
                let base = self.read_memory(self.program_counter) as u16;
                let final_address = base.wrapping_add(self.register_x as u16);
                final_address
            }
            AddressingMode::IndirectY => {
                let base = self.read_memory(self.program_counter) as u16;
                let final_address = base.wrapping_add(self.register_y as u16);
                final_address
            }
        }
    }

    // -----------------------------
    // LOAD / STORE Operations
    // LDA, LDX, LDY , STA, STX, STY
    // -----------------------------

    // LDA (Load Accumulator)
    fn lda(&mut self, mode: AddressingMode) {
        let address = self.select_addressing_mode(mode);
        self.register_a = self.read_memory(address);
        self.program_counter += 1;
        self.update_zero_and_negative_flag(self.register_a);
    }

    // LDX
    fn ldx(&mut self, mode: AddressingMode) {
        let address = self.select_addressing_mode(mode);
        self.register_x = self.read_memory(address);
        self.program_counter += 1;
        self.update_zero_and_negative_flag(self.register_x);
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.load_program_into_memory(program);
        self.program_counter = 0;
        loop {
            let opscode = self.memory[self.program_counter as usize];
            // get opscode and go to next instruction
            self.program_counter += 1;

            match opscode {
                // -----------------------------
                // LOAD / STORE Operations
                // LDA, LDX, LDY , STA, STX, STY
                // -----------------------------

                // LDA (Load Accumulator)
                0xA9 => self.lda(AddressingMode::Immediate),
                0xA5 => self.lda(AddressingMode::ZeroPage),
                0xB5 => self.lda(AddressingMode::ZeroPageX),
                0xAD => self.lda(AddressingMode::Absolute),
                0xBD => self.lda(AddressingMode::AbsoluteX),
                0xB9 => self.lda(AddressingMode::AbsoluteY),
                0xA1 => self.lda(AddressingMode::IndirectX),
                0xB1 => self.lda(AddressingMode::IndirectY),

                // LDX (Load )
                0xA2 => self.ldx(AddressingMode::Immediate),
                0xA6 => self.ldx(AddressingMode::ZeroPage),
                0xB6 => self.ldx(AddressingMode::ZeroPageY),
                0xAE => self.ldx(AddressingMode::Absolute),
                0xBE => self.ldx(AddressingMode::AbsoluteY),
                // END OF LOAD / STORE Operations
                0x00 => {
                    println!("BRK");
                    break;
                }
                _ => {
                    println!("Unrecognized opscode: {:x}", opscode);
                    break;
                }
            }
        }
    }
}

// -----------------------------
// TEST Section
// -----------------------------

#[cfg(test)]
mod test {
    use super::*;

    // -----------------------------
    // LOAD / STORE Operations
    // LDA, LDX, LDY , STA, STX, STY
    // -----------------------------

    // LDA (Load Accumulator)

    // LDA STATUS FLAG

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.status, 2); // 2 is zero flag 0b10
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0xE0, 0x00]);

        assert_eq!(cpu.register_a, 224);
        assert_eq!(cpu.status, 128) //
    }

    // LDA ADDRESSING MODE

    #[test]
    fn test_0xa9_lda_immediate() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 5); // load 5 to register 5
                                       // The Zero flag should be clear
        assert_eq!(cpu.status & 0b0000_0010, 0);
        assert_eq!(cpu.status & 0b1000_0000, 0);
    }

    #[test]
    fn test_0xa5_lda_zero_page() {
        let mut cpu = CPU::new();
        cpu.memory[0x84] = 0x37; // Set up memory so that address 0x84 contains the value 0x37
        cpu.interpret(vec![0xa5, 0x84, 0x00]); // Execute LDA with zero page addressing mode
        assert_eq!(cpu.register_a, 0x37); // Check that register_a contains the value 0x37
    }

    #[test]
    fn test_0xb5_lda_zero_page_x() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x05;
        cpu.memory[0x80 + cpu.register_x as usize] = 0x37; // Set up memory so that address 0x80 + X contains the value 0x37
        cpu.interpret(vec![0xb5, 0x80, 0x00]); // Execute LDA with zero page X addressing mode
        assert_eq!(cpu.register_a, 0x37); // Check that register_a contains the value 0x37
    }

    #[test]
    fn test_0xad_lda_absolute() {
        let mut cpu = CPU::new();
        cpu.memory[0x1234] = 0x37; // Set up memory so that address 0x1234 contains the value 0x37
        cpu.interpret(vec![0xad, 0x34, 0x12]); // Execute LDA with absolute addressing mode
        assert_eq!(cpu.register_a, 0x37); // Check that register_a contains the value 0x37
    }

    #[test]
    fn test_0xbd_lda_absolute_x() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x05;
        cpu.memory[0x1234 + cpu.register_x as usize] = 0x37; // Set up memory so that address 0x1234 + X contains the value 0x37
        cpu.interpret(vec![0xbd, 0x34, 0x12]); // Execute LDA with absolute X addressing mode
        assert_eq!(cpu.register_a, 0x37); // Check that register_a contains the value 0x37
    }

    #[test]
    fn test_0xb9_lda_absolute_y() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x05;
        cpu.memory[0x1234 + cpu.register_y as usize] = 0x37; // Set up memory so that address 0x1234 + Y contains the value 0x37
        cpu.interpret(vec![0xb9, 0x34, 0x12]); // Execute LDA with absolute Y addressing mode
        assert_eq!(cpu.register_a, 0x37); // Check that register_a contains the value 0x37
    }

    #[test]
    fn test_0xa1_lda_indirect_x() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x05;
        cpu.memory[0x84 + cpu.register_x as usize] = 0x37; // Set up memory so that address 0x84 + X contains the value 0x37
        cpu.interpret(vec![0xa1, 0x84, 0x00]); // Execute LDA with indirect X addressing mode
        assert_eq!(cpu.register_a, 0x37); // Check that register_a contains the value 0x37
    }

    #[test]
    fn test_0xb1_lda_indirect_y() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x05;
        cpu.memory[0x84 + cpu.register_y as usize] = 0x37; // Set up memory so that address 0x84 contains the value 0x37
        cpu.interpret(vec![0xb1, 0x84, 0x00]); // Execute LDA with indirect Y addressing mode
        assert_eq!(cpu.register_a, 0x37); // Check that register_a contains the value 0x37
    }

    // LDX (Load X Register)

    // LDX STATUS FLAG

    #[test]
    fn test_0xa2_ldx_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa2, 0x00, 0x00]);
        assert_eq!(cpu.register_x, 0);
        assert_eq!(cpu.status, 2); // 2 is zero flag 0b10
    }

    #[test]
    fn test_0xa2_ldx_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa2, 0xE0, 0x00]);

        assert_eq!(cpu.register_x, 224);
        assert_eq!(cpu.status, 128) //
    }

    // LDX ADDRESSING MODE

    #[test]
    fn test_0xa2_ldx_immediate() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa2, 0x05, 0x00]);
        assert_eq!(cpu.register_x, 5); // load 5 to register 5
                                       // The Zero flag should be clear
        assert_eq!(cpu.status & 0b0000_0010, 0);
        assert_eq!(cpu.status & 0b1000_0000, 0);
    }

    #[test]
    fn test_0xa6_ldx_zero_page() {
        let mut cpu = CPU::new();
        cpu.memory[0x84] = 0x37; // Set up memory so that address 0x84 contains the value 0x37
        cpu.interpret(vec![0xa6, 0x84, 0x00]); // Execute LDX with zero page addressing mode
        assert_eq!(cpu.register_x, 0x37); // Check that register_x contains the value 0x37
    }

    #[test]
    fn test_0xb6_ldx_zero_page_y() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x05;
        cpu.memory[0x80 + cpu.register_y as usize] = 0x37; // Set up memory so that address 0x80 + Y contains the value 0x37
        cpu.interpret(vec![0xb6, 0x80, 0x00]); // Execute LDX with zero page Y addressing mode
        assert_eq!(cpu.register_x, 0x37); // Check that register_x contains the value 0x37
    }

    #[test]
    fn test_0xae_ldx_absolute() {
        let mut cpu = CPU::new();
        cpu.memory[0x1234] = 0x37; // Set up memory so that address 0x1234 contains the value 0x37
        cpu.interpret(vec![0xae, 0x34, 0x12]); // Execute LDX with absolute addressing mode
        assert_eq!(cpu.register_x, 0x37); // Check that register_x contains the value 0x37
    }

    #[test]
    fn test_0xbe_ldx_absolute_y() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x05;
        cpu.memory[0x1234 + cpu.register_y as usize] = 0x37; // Set up memory so that address 0x1234 + Y contains the value 0x37
        cpu.interpret(vec![0xbe, 0x34, 0x12]); // Execute LDX with absolute Y addressing mode
        assert_eq!(cpu.register_x, 0x37); // Check that register_x contains the value 0x37
    }
}
