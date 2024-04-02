// CPU 6802 Flags
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
    pub stack_pointer: u8,
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

            // 0xFD is the default stack pointer value
            stack_pointer: 0xFD,
        }
    }

    fn set_flag(&mut self, flag: Flag, condition: bool) {
        if condition {
            self.status |= flag as u8;
        } else {
            self.status &= !(flag as u8);
        }
    }

    fn read_memory(&self, address: u16) -> u8 {
        println!(
            "addres inside readmeory {:?} {:?}",
            address, self.memory[address as usize]
        );
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
        println!(" test {:?} - {:?}", low_byte, high_byte);
        (high_byte << 8) | (low_byte as u16)
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

    fn stack_push(&mut self, value: u8) {
        // Stack is located at 0x0100 to 0x01FF
        self.write_memory(0x0100 + self.stack_pointer as u16, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.read_memory(0x0100 + self.stack_pointer as u16)
    }

    fn load_program_into_memory(&mut self, program: Vec<u8>) {
        for (i, &byte) in program.iter().enumerate() {
            self.write_memory(i as u16, byte);
        }
    }

    // fn update_break_command_flag(&mut self, set: bool) {
    //     if set {
    //         self.status |= Flag::Break as u8;
    //     } else {
    //         self.status &= !(Flag::Break as u8);
    //     }
    // }

    fn set_zero_flag(&mut self, register: u8) {
        // if register = 0b0000_0000
        // set Zero flag to register
        self.set_flag(Flag::Zero, register == 0);
    }

    fn set_negative_flag(&mut self, register: u8) {
        self.set_flag(Flag::Negative, register & Flag::Negative as u8 != 0);
    }

    fn set_zero_negative_flag(&mut self, register: u8) {
        self.set_zero_flag(register);
        self.set_negative_flag(register);
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
                println!("absolute address {}", address);
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
                let address = self.read_memory(self.program_counter) as u16;
                let pointer = address.wrapping_add(self.register_x as u16);
                println!("{:?}", pointer);
                let final_address = self.read_memory_16bit(pointer);
                final_address
            }

            AddressingMode::IndirectY => {
                let address = self.read_memory(self.program_counter) as u16;
                let final_address = self.read_memory_16bit(address);
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
        self.set_zero_negative_flag(self.register_a);
    }

    // LDX (Load X Register)
    fn ldx(&mut self, mode: AddressingMode) {
        let address = self.select_addressing_mode(mode);
        self.register_x = self.read_memory(address);
        self.program_counter += 1;
        self.set_zero_negative_flag(self.register_x);
    }

    // LDY (Load Y Register)
    fn ldy(&mut self, mode: AddressingMode) {
        let address = self.select_addressing_mode(mode);
        self.register_y = self.read_memory(address);
        self.program_counter += 1;
        self.set_zero_negative_flag(self.register_y);
    }

    // STA (Store Accumulator)
    fn sta(&mut self, mode: AddressingMode) {
        let address = self.select_addressing_mode(mode);
        self.write_memory(address, self.register_a);
        self.program_counter += 1;
    }

    // STX (Store X Register)
    fn stx(&mut self, mode: AddressingMode) {
        let address = self.select_addressing_mode(mode);
        self.write_memory(address, self.register_x);
        self.program_counter += 1;
    }

    // STY (Store Y Register)
    fn sty(&mut self, mode: AddressingMode) {
        let address = self.select_addressing_mode(mode);
        self.write_memory(address, self.register_y);
        self.program_counter += 1;
    }

    // -----------------------------
    // Register Transfer
    // TAX, TAY, TXA, TYA
    // -----------------------------

    // TAX (Transfer Accumulator to X)
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.set_zero_negative_flag(self.register_x);
    }

    // TAY (Transfer Accumulator to Y)
    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.set_zero_negative_flag(self.register_y);
    }

    // TXA (Transfer X to Accumulator)
    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.set_zero_negative_flag(self.register_a);
    }

    // TYA (Transfer Y to Accumulator)
    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.set_zero_negative_flag(self.register_a);
    }

    // -----------------------------
    // Stack Operations
    // TSX, TXS, PHA, PHP, PLA, PLP
    // -----------------------------

    // TSX (Transfer Stack Pointer to X)
    fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.set_zero_negative_flag(self.register_x);
    }

    // TXS (Transfer X to Stack Pointer)
    fn txs(&mut self) {
        self.stack_pointer = self.register_x;
    }

    // PHA (Push Accumulator)
    fn pha(&mut self) {
        self.stack_push(self.register_a);
    }

    // PHP (Push Processor Status)
    fn php(&mut self) {
        self.stack_push(self.status);
    }

    // PLA (Pull Accumulator)
    fn pla(&mut self) {
        self.register_a = self.stack_pop();
        self.set_zero_negative_flag(self.register_a);
    }

    // PLP (Pull Processor Status)
    fn plp(&mut self) {
        self.status = self.stack_pop();
    }

    // -----------------------------
    // Logical
    // AND, EOR, ORA, BIT
    // -----------------------------

    // AND (Logical AND)

    fn and(&mut self, mode: AddressingMode) {
        let address = self.select_addressing_mode(mode);
        let value = self.read_memory(address);
        self.register_a &= value;
        self.program_counter += 1;
        self.set_zero_negative_flag(self.register_a);
    }

    fn eor(&mut self, mode: AddressingMode) {
        let address = self.select_addressing_mode(mode);
        let value = self.read_memory(address);
        self.register_a ^= value;
        self.program_counter += 1;
        self.set_zero_negative_flag(self.register_a);
    }

    fn ora(&mut self, mode: AddressingMode) {
        let address = self.select_addressing_mode(mode);
        let value = self.read_memory(address);
        self.register_a |= value;
        self.program_counter += 1;

        println!("register_a {:x} {:?} {:?}", self.register_a, value, address);
        self.set_zero_negative_flag(self.register_a);
    }

    fn bit(&mut self, mode: AddressingMode) {
        let address = self.select_addressing_mode(mode);
        let value = self.read_memory(address);
        let and = self.register_a & value;
        self.set_flag(Flag::Zero, and == 0);
        self.set_flag(Flag::Negative, value & Flag::Negative as u8 > 0);
        self.set_flag(Flag::Overflow, value & Flag::Overflow as u8 > 0);
        self.program_counter += 1;
    }

    // EOR (Exclusive OR)
    // ORA (Logical Inclusive OR)
    // BIT (Bit Test)

    // -----------------------------
    // System Function
    // BRK, NOP, RTI
    // -----------------------------

    fn brk(&mut self) {
        // self.update_break_command_flag(true);
        self.program_counter += 1;
    }

    fn nop(&mut self) {
        self.program_counter += 1;
    }

    fn reset_cpu(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = 0;
        self.program_counter = 0;
        self.stack_pointer = 0xFD;
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

                // LDX (Load X Register)
                0xA2 => self.ldx(AddressingMode::Immediate),
                0xA6 => self.ldx(AddressingMode::ZeroPage),
                0xB6 => self.ldx(AddressingMode::ZeroPageY),
                0xAE => self.ldx(AddressingMode::Absolute),
                0xBE => self.ldx(AddressingMode::AbsoluteY),

                // LDY (Load Y Register)
                0xA0 => self.ldy(AddressingMode::Immediate),
                0xA4 => self.ldy(AddressingMode::ZeroPage),
                0xB4 => self.ldy(AddressingMode::ZeroPageX),
                0xAC => self.ldy(AddressingMode::Absolute),
                0xBC => self.ldy(AddressingMode::AbsoluteX),

                // STA (Store Accumulator)
                0x85 => self.sta(AddressingMode::ZeroPage),
                0x95 => self.sta(AddressingMode::ZeroPageX),
                0x8D => self.sta(AddressingMode::Absolute),
                0x9D => self.sta(AddressingMode::AbsoluteX),
                0x99 => self.sta(AddressingMode::AbsoluteY),
                0x81 => self.sta(AddressingMode::IndirectX),
                0x91 => self.sta(AddressingMode::IndirectY),

                // STX (Store X Register)
                0x86 => self.stx(AddressingMode::ZeroPage),
                0x96 => self.stx(AddressingMode::ZeroPageY),
                0x8E => self.stx(AddressingMode::Absolute),

                // STY (Store Y Register)
                0x84 => self.sty(AddressingMode::ZeroPage),
                0x94 => self.sty(AddressingMode::ZeroPageX),
                0x8C => self.sty(AddressingMode::Absolute),

                // -----------------------------
                // Register Transfer
                // TAX, TAY, TXA, TYA
                // -----------------------------

                // TAX (Transfer Accumulator to X)
                0xAA => self.tax(),

                // TAY (Transfer Accumulator to Y)
                0xA8 => self.tay(),

                // TXA (Transfer X to Accumulator)
                0x8A => self.txa(),

                // TYA (Transfer Y to Accumulator)
                0x98 => self.tya(),

                // -----------------------------
                // Stack Operations
                // TSX, TXS, PHA, PHP, PLA, PLP
                // -----------------------------

                // TSX (Transfer Stack Pointer to X)
                0xBA => self.tsx(),

                // TXS (Transfer X to Stack Pointer)
                0x9A => self.txs(),

                // PHA (Push Accumulator)
                0x48 => self.pha(),

                // PHP (Push Processor Status)
                0x08 => self.php(),

                // PLA (Pull Accumulator)
                0x68 => self.pla(),

                // plp (Pull Processor Status)
                0x28 => self.plp(),

                // -----------------------------
                // Logical
                // AND, EOR, ORA, BIT
                // -----------------------------

                // AND (Logical AND)
                0x29 => self.and(AddressingMode::Immediate),
                0x25 => self.and(AddressingMode::ZeroPage),
                0x35 => self.and(AddressingMode::ZeroPageX),
                0x2D => self.and(AddressingMode::Absolute),
                0x3D => self.and(AddressingMode::AbsoluteX),
                0x39 => self.and(AddressingMode::AbsoluteY),
                0x21 => self.and(AddressingMode::IndirectX),
                0x31 => self.and(AddressingMode::IndirectY),

                // EOR (Exclusive OR)
                0x49 => self.eor(AddressingMode::Immediate),
                0x45 => self.eor(AddressingMode::ZeroPage),
                0x55 => self.eor(AddressingMode::ZeroPageX),
                0x4D => self.eor(AddressingMode::Absolute),
                0x5D => self.eor(AddressingMode::AbsoluteX),
                0x59 => self.eor(AddressingMode::AbsoluteY),
                0x41 => self.eor(AddressingMode::IndirectX),
                0x51 => self.eor(AddressingMode::IndirectY),

                // ORA (Logical Inclusive OR)
                0x09 => self.ora(AddressingMode::Immediate),
                0x05 => self.ora(AddressingMode::ZeroPage),
                0x15 => self.ora(AddressingMode::ZeroPageX),
                0x0D => self.ora(AddressingMode::Absolute),
                0x1D => self.ora(AddressingMode::AbsoluteX),
                0x19 => self.ora(AddressingMode::AbsoluteY),
                0x01 => self.ora(AddressingMode::IndirectX),
                0x11 => self.ora(AddressingMode::IndirectY),

                // BIT (Bit Test)
                0x24 => self.bit(AddressingMode::ZeroPage),
                0x2C => self.bit(AddressingMode::Absolute),

                // -----------------------------
                // System Function
                // BRK, NOP, RTI
                // -----------------------------

                // BRK (Break)
                0x00 => {
                    self.brk();
                    break;
                }

                // NOP (No Operation)
                0xEA => self.nop(),

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
        cpu.memory[0x37] = 0x37;
        cpu.interpret(vec![0xa1, 0x84, 0x00]); // Execute LDA with indirect X addressing mode
        assert_eq!(cpu.register_a, 0x37); // Check that register_a contains the value 0x37
    }

    #[test]
    fn test_0xb1_lda_indirect_y() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x05;
        cpu.memory[0x84] = 0x37; // Set up memory so that address 0x84 contains the value 0x37
        cpu.memory[0x37] = 0x37;
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

    // LDY (Load Y Register)

    // LDY STATUS FLAG

    #[test]
    fn test_0xa0_ldy_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa0, 0x00, 0x00]);
        assert_eq!(cpu.register_y, 0);
        assert_eq!(cpu.status, 2); // 2 is zero flag 0b10
    }

    #[test]
    fn test_0xa0_ldy_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa0, 0xE0, 0x00]);

        assert_eq!(cpu.register_y, 224);
        assert_eq!(cpu.status, 128) //
    }

    // LDY ADDRESSING MODE
    #[test]
    fn test_0xa0_ldy_immediate() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa0, 0x05, 0x00]);
        assert_eq!(cpu.register_y, 5); // load 5 to register 5
                                       // The Zero flag should be clear
        assert_eq!(cpu.status & 0b0000_0010, 0);
        assert_eq!(cpu.status & 0b1000_0000, 0);
    }

    #[test]
    fn test_0xa4_ldy_zero_page() {
        let mut cpu = CPU::new();
        cpu.memory[0x84] = 0x37; // Set up memory so that address 0x84 contains the value 0x37
        cpu.interpret(vec![0xa4, 0x84, 0x00]); // Execute LDY with zero page addressing mode
        assert_eq!(cpu.register_y, 0x37); // Check that register_y contains the value 0x37
    }

    #[test]
    fn test_0xb4_ldy_zero_page_x() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x05;
        cpu.memory[0x80 + cpu.register_x as usize] = 0x37; // Set up memory so that address 0x80 + X contains the value 0x37
        cpu.interpret(vec![0xb4, 0x80, 0x00]); // Execute LDY with zero page X addressing mode
        assert_eq!(cpu.register_y, 0x37); // Check that register_y contains the value 0x37
    }

    #[test]
    fn test_0xac_ldy_absolute() {
        let mut cpu = CPU::new();
        cpu.memory[0x1234] = 0x37; // Set up memory so that address 0x1234 contains the value 0x37
        cpu.interpret(vec![0xac, 0x34, 0x12]); // Execute LDY with absolute addressing mode
        assert_eq!(cpu.register_y, 0x37); // Check that register_y contains the value 0x37
    }

    #[test]
    fn test_0xbc_ldy_absolute_x() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x05;
        cpu.memory[0x1234 + cpu.register_x as usize] = 0x37; // Set up memory so that address 0x1234 + X contains the value 0x37
        cpu.interpret(vec![0xbc, 0x34, 0x12]); // Execute LDY with absolute X addressing mode
        assert_eq!(cpu.register_y, 0x37); // Check that register_y contains the value 0x37
    }

    // STA (Store Accumulator)

    // STA ADDRESSING MODE

    #[test]
    fn test_0x85_sta_zero_page() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x37; // Set up register_a so that it contains the value 0x37
        cpu.interpret(vec![0x85, 0x84, 0x00]); // Execute STA with zero page addressing mode
        assert_eq!(cpu.memory[0x84], 0x37); // Check that memory address 0x84 contains the value 0x37
    }

    #[test]
    fn test_0x95_sta_zero_page_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x37; // Set up register_a so that it contains the value 0x37
        cpu.register_x = 0x05;
        cpu.interpret(vec![0x95, 0x80, 0x00]); // Execute STA with zero page X addressing mode
        assert_eq!(cpu.memory[0x80 + cpu.register_x as usize], 0x37); // Check that memory address 0x80 + X contains the value 0x37
    }

    #[test]
    fn test_0x8d_sta_absolute() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x37; // Set up register_a so that it contains the value 0x37
        cpu.interpret(vec![0x8d, 0x34, 0x12]); // Execute STA with absolute addressing mode
        assert_eq!(cpu.memory[0x1234], 0x37); // Check that memory address 0x1234 contains the value 0x37
    }

    #[test]
    fn test_0x9d_sta_absolute_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x37; // Set up register_a so that it contains the value 0x37
        cpu.register_x = 0x05;
        cpu.interpret(vec![0x9d, 0x34, 0x12]); // Execute STA with absolute X addressing mode
        assert_eq!(cpu.memory[0x1234 + cpu.register_x as usize], 0x37); // Check that memory address 0x1234 + X contains the value 0x37
    }

    #[test]
    fn test_0x99_sta_absolute_y() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x37; // Set up register_a so that it contains the value 0x37
        cpu.register_y = 0x05;
        cpu.interpret(vec![0x99, 0x34, 0x12]); // Execute STA with absolute Y addressing mode
        assert_eq!(cpu.memory[0x1234 + cpu.register_y as usize], 0x37); // Check that memory address 0x1234 + Y contains the value 0x37
    }

    #[test]
    fn test_0x81_sta_indirect_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x37; // Set up register_a so that it contains the value 0x37
        cpu.register_x = 0x05;
        cpu.memory[0x84 + cpu.register_x as usize] = 0x37; // Set up memory so that address 0x84 + X contains the value 0x37
        cpu.interpret(vec![0x81, 0x84, 0x00]); // Execute STA with indirect X addressing mode
        assert_eq!(cpu.memory[0x84 + cpu.register_x as usize], 0x37); // Check that memory address 0x84 + X contains the value 0x37
    }

    #[test]
    fn test_0x91_sta_indirect_y() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x37; // Set up register_a so that it contains the value 0x37
        cpu.register_y = 0x05;
        cpu.memory[0x84] = 0x37; // Set up memory so that address 0x84 contains the value 0x37
        cpu.interpret(vec![0x91, 0x84, 0x00]); // Execute STA with indirect Y addressing mode
        assert_eq!(cpu.memory[0x84], 0x37); // Check that memory address 0x84 + Y contains the value 0x37
    }

    // STX (Store X Register)

    // STX ADDRESSING MODE

    #[test]
    fn test_0x86_stx_zero_page() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x37; // Set up register_x so that it contains the value 0x37
        cpu.interpret(vec![0x86, 0x84, 0x00]); // Execute STX with zero page addressing mode
        assert_eq!(cpu.memory[0x84], 0x37); // Check that memory address 0x84 contains the value 0x37
    }

    #[test]
    fn test_0x96_stx_zero_page_y() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x37; // Set up register_x so that it contains the value 0x37
        cpu.register_y = 0x05;
        cpu.interpret(vec![0x96, 0x80, 0x00]); // Execute STX with zero page Y addressing mode
        assert_eq!(cpu.memory[0x80 + cpu.register_y as usize], 0x37); // Check that memory address 0x80 + Y contains the value 0x37
    }

    #[test]
    fn test_0x8e_stx_absolute() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x37; // Set up register_x so that it contains the value 0x37
        cpu.interpret(vec![0x8e, 0x34, 0x12]); // Execute STX with absolute addressing mode
        assert_eq!(cpu.memory[0x1234], 0x37); // Check that memory address 0x1234 contains the value 0x37
    }

    // STY (Store Y Register)

    // STY ADDRESSING MODE

    #[test]
    fn test_0x84_sty_zero_page() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x37; // Set up register_y so that it contains the value 0x37
        cpu.interpret(vec![0x84, 0x84, 0x00]); // Execute STY with zero page addressing mode
        assert_eq!(cpu.memory[0x84], 0x37); // Check that memory address 0x84 contains the value 0x37
    }

    #[test]
    fn test_0x94_sty_zero_page_x() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x37; // Set up register_y so that it contains the value 0x37
        cpu.register_x = 0x05;
        cpu.interpret(vec![0x94, 0x80, 0x00]); // Execute STY with zero page X addressing mode
        assert_eq!(cpu.memory[0x80 + cpu.register_x as usize], 0x37); // Check that memory address 0x80 + X contains the value 0x37
    }

    #[test]
    fn test_0x8c_sty_absolute() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x37; // Set up register_y so that it contains the value 0x37
        cpu.interpret(vec![0x8c, 0x34, 0x12]); // Execute STY with absolute addressing mode
        assert_eq!(cpu.memory[0x1234], 0x37); // Check that memory address 0x1234 contains the value 0x37
    }

    // -----------------------------
    // Register Transfer
    // TAX, TAY, TXA, TYA
    // -----------------------------

    // TAX (Transfer Accumulator to X)
    #[test]
    fn test_0xaa_tax() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x37; // Set up register_a so that it contains the value 0x37
        cpu.interpret(vec![0xaa, 0x00]); // Execute TAX
        assert_eq!(cpu.register_x, 0x37); // Check that register_x contains the value 0x37
    }

    // TAY (Transfer Accumulator to Y)
    #[test]
    fn test_0xa8_tay() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x37;
        cpu.interpret(vec![0xa8, 0x00]);
        assert_eq!(cpu.register_y, 0x37);
    }

    // TXA (Transfer X to Accumulator)
    #[test]
    fn test_0x8a_txa() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x37;
        cpu.interpret(vec![0x8a, 0x00]);
        assert_eq!(cpu.register_a, 0x37);
    }

    // TYA (Transfer Y to Accumulator)
    #[test]
    fn test_0x98_tya() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x37;
        cpu.interpret(vec![0x98, 0x00]);
        assert_eq!(cpu.register_a, 0x37);
    }

    // -----------------------------
    // Stack Operations
    // TSX, TXS, PHA, PHP, PLA, PLP
    // -----------------------------

    // TSX (Transfer Stack Pointer to X)
    #[test]
    fn test_0xba_tsx() {
        let mut cpu = CPU::new();
        cpu.stack_pointer = 0x37;
        cpu.interpret(vec![0xba, 0x00]);
        assert_eq!(cpu.register_x, 0x37);
    }

    // TXS (Transfer X to Stack Pointer)
    #[test]
    fn test_0x9a_txs() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x37;
        cpu.interpret(vec![0x9a, 0x00]);
        assert_eq!(cpu.stack_pointer, 0x37);
    }

    // PHA (Push Accumulator)
    #[test]
    fn test_0x48_pha() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x37;
        cpu.interpret(vec![0x48, 0x00]);
        assert_eq!(cpu.stack_pop(), 0x37);
    }

    // PHP (Push Processor Status)
    #[test]
    fn test_0x08_php() {
        let mut cpu = CPU::new();
        cpu.status = 0x37;
        cpu.interpret(vec![0x08, 0x00]);
        assert_eq!(cpu.stack_pop(), 0x37);
    }

    // PLA (Pull Accumulator)
    #[test]
    fn test_0x68_pla() {
        let mut cpu = CPU::new();
        cpu.stack_push(0x37);
        cpu.interpret(vec![0x68, 0x00]);
        assert_eq!(cpu.register_a, 0x37);
    }

    // PLP (Pull Processor Status)
    #[test]
    fn test_0x28_plp() {
        let mut cpu = CPU::new();
        cpu.stack_push(0x37);
        cpu.interpret(vec![0x28, 0x00]);
        assert_eq!(cpu.status, 0x37);
    }

    // -----------------------------
    // Logical
    // AND, EOR, ORA, BIT
    // -----------------------------

    // AND (Logical AND)
    #[test]
    fn test_0x29_and_immediate() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.interpret(vec![0x29, 0b1100_1100]);
        assert_eq!(cpu.register_a, 0b1000_1000);
    }

    #[test]
    fn test_0x25_and_zero_page() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.memory[0x84] = 0b1100_1100;
        cpu.interpret(vec![0x25, 0x84]);
        assert_eq!(cpu.register_a, 0b1000_1000);
    }

    #[test]
    fn test_0x35_and_zero_page_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.register_x = 0x05;
        cpu.memory[0x80 + cpu.register_x as usize] = 0b1100_1100;
        cpu.interpret(vec![0x35, 0x80]);
        assert_eq!(cpu.register_a, 0b1000_1000);
    }

    #[test]
    fn test_0x2d_and_absolute() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.memory[0x1234] = 0b1100_1100;
        cpu.interpret(vec![0x2d, 0x34, 0x12]);
        assert_eq!(cpu.register_a, 0b1000_1000);
    }

    #[test]
    fn test_0x3d_and_absolute_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.register_x = 0x05;
        cpu.memory[0x1234 + cpu.register_x as usize] = 0b1100_1100;
        cpu.interpret(vec![0x3d, 0x34, 0x12]);
        assert_eq!(cpu.register_a, 0b1000_1000);
    }

    #[test]
    fn test_0x39_and_absolute_y() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.register_y = 0x05;
        cpu.memory[0x1234 + cpu.register_y as usize] = 0b1100_1100;
        cpu.interpret(vec![0x39, 0x34, 0x12]);
        assert_eq!(cpu.register_a, 0b1000_1000);
    }

    #[test]
    fn test_0x21_and_indirect_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.register_x = 0x05;
        cpu.memory[0x84 + cpu.register_x as usize] = 0b1100_1100;
        cpu.memory[0b1100_1100] = 0b1100_1100;
        cpu.interpret(vec![0x21, 0x84]);
        assert_eq!(cpu.register_a, 0b1000_1000);
    }

    #[test]
    fn test_0x31_and_indirect_y() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.register_y = 0x05;
        cpu.memory[0x84] = 0b1100_1100;
        cpu.memory[0b1100_1100] = 0b1100_1100;
        cpu.interpret(vec![0x31, 0x84]);
        assert_eq!(cpu.register_a, 0b1000_1000);
    }

    // EOR (Exclusive OR)
    #[test]
    fn test_0x49_eor_immediate() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.interpret(vec![0x49, 0b1100_1100]);
        assert_eq!(cpu.register_a, 0b0110_0110);
    }

    #[test]
    fn test_0x45_eor_zero_page() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.memory[0x84] = 0b1100_1100;
        cpu.interpret(vec![0x45, 0x84]);
        assert_eq!(cpu.register_a, 0b0110_0110);
    }

    #[test]
    fn test_0x55_eor_zero_page_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.register_x = 0x05;
        cpu.memory[0x80 + cpu.register_x as usize] = 0b1100_1100;
        cpu.interpret(vec![0x55, 0x80]);
        assert_eq!(cpu.register_a, 0b0110_0110);
    }

    #[test]
    fn test_0x4d_eor_absolute() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.memory[0x1234] = 0b1100_1100;
        cpu.interpret(vec![0x4d, 0x34, 0x12]);
        assert_eq!(cpu.register_a, 0b0110_0110);
    }

    #[test]
    fn test_0x5d_eor_absolute_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.register_x = 0x05;
        cpu.memory[0x1234 + cpu.register_x as usize] = 0b1100_1100;
        cpu.interpret(vec![0x5d, 0x34, 0x12]);
        assert_eq!(cpu.register_a, 0b0110_0110);
    }

    #[test]
    fn test_0x59_eor_absolute_y() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.register_y = 0x05;
        cpu.memory[0x1234 + cpu.register_y as usize] = 0b1100_1100;
        cpu.interpret(vec![0x59, 0x34, 0x12]);
        assert_eq!(cpu.register_a, 0b0110_0110);
    }

    #[test]
    fn test_0x41_eor_indirect_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.register_x = 0x05;
        cpu.memory[0x84 + cpu.register_x as usize] = 0b1100_1100;
        cpu.memory[0b1100_1100] = 0b1100_1100;
        cpu.interpret(vec![0x41, 0x84]);
        assert_eq!(cpu.register_a, 0b0110_0110);
    }

    #[test]
    fn test_0x51_eor_indirect_y() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.register_y = 0x05;
        cpu.memory[0x84] = 0b1100_1100;
        cpu.memory[0b1100_1100] = 0b1100_1100;
        cpu.interpret(vec![0x51, 0x84]);
        assert_eq!(cpu.register_a, 0b0110_0110);
    }

    // ORA (Logical Inclusive OR)

    #[test]
    fn test_0x09_ora_immediate() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.interpret(vec![0x09, 0b1100_1100]);
        assert_eq!(cpu.register_a, 0b1110_1110);
    }

    #[test]
    fn test_0x05_ora_zero_page() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.memory[0x84] = 0b1100_1100;
        cpu.interpret(vec![0x05, 0x84]);
        assert_eq!(cpu.register_a, 0b1110_1110);
    }

    #[test]
    fn test_0x15_ora_zero_page_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.register_x = 0x05;
        cpu.memory[0x80 + cpu.register_x as usize] = 0b1100_1100;
        cpu.interpret(vec![0x15, 0x80]);
        assert_eq!(cpu.register_a, 0b1110_1110);
    }

    #[test]
    fn test_0x0d_ora_absolute() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.memory[0x1234] = 0b1100_1100;
        cpu.interpret(vec![0x0d, 0x34, 0x12]);
        assert_eq!(cpu.register_a, 0b1110_1110);
    }

    #[test]
    fn test_0x1d_ora_absolute_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.register_x = 0x05;
        cpu.memory[0x1234 + cpu.register_x as usize] = 0b1100_1100;
        cpu.interpret(vec![0x1d, 0x34, 0x12]);
        assert_eq!(cpu.register_a, 0b1110_1110);
    }

    #[test]
    fn test_0x19_ora_absolute_y() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.register_y = 0x05;
        cpu.memory[0x1234 + cpu.register_y as usize] = 0b1100_1100;
        cpu.interpret(vec![0x19, 0x34, 0x12]);
        assert_eq!(cpu.register_a, 0b1110_1110);
    }

    #[test]
    fn test_0x01_ora_indirect_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.register_x = 0x05;
        cpu.memory[0x84 + cpu.register_x as usize] = 0b1100_1100;
        cpu.memory[0b1100_1100] = 0b1100_1100;
        cpu.interpret(vec![0x01, 0x84]);
        assert_eq!(cpu.register_a, 0b1110_1110);
    }

    #[test]
    fn test_0x11_ora_indirect_y() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.register_y = 0x05;
        cpu.memory[0x84] = 0b1100_1100;
        cpu.memory[0b1100_1100] = 0b1100_1100;
        cpu.interpret(vec![0x11, 0x84]);
        assert_eq!(cpu.register_a, 0b1110_1110);
    }

    // BIT (Bit Test)
    #[test]
    fn test_0x24_bit_zero_page() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.memory[0x84] = 0b1100_1100;
        cpu.interpret(vec![0x24, 0x84]);

        // status result us 192
        // 0b1100_0000
        // zero flag not set
        // but negative and overflow flag set
        assert_eq!(cpu.status & 0b0000_0010, 0);
        assert_eq!(cpu.status & 0b1000_0000, 0b1000_0000);
        assert_eq!(cpu.status & 0b0100_0000, 0b0100_0000);
    }

    #[test]
    fn test_0x2c_bit_absolute() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.memory[0x1234] = 0b1100_1100;
        cpu.interpret(vec![0x2c, 0x34, 0x12]);
        println!("{:?}", cpu.status);
        assert_eq!(cpu.status & 0b0000_0010, 0);
        assert_eq!(cpu.status & 0b1000_0000, 0b1000_0000);
        assert_eq!(cpu.status & 0b0100_0000, 0b0100_0000);
    }

    // -----------------------------
    // System Function
    // BRK, NOP, RTI
    // -----------------------------å

    // NOP (No Operation)
    #[test]
    fn test_0xea_nop() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xea]);
        assert_eq!(cpu.status, 0);
    }
}
