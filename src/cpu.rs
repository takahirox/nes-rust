use register::Register;
use memory::Memory;
use rom::{HEADER_SIZE, Rom};
use ppu::Ppu;
use apu::Apu;
use button;
use joypad;
use joypad::Joypad;
use input::Input;
use display::Display;
use audio::Audio;

fn to_joypad_button(button: button::Button) -> joypad::Button {
	match button {
		button::Button::Joypad1A |
		button::Button::Joypad2A => joypad::Button::A,
		button::Button::Joypad1B |
		button::Button::Joypad2B => joypad::Button::B,
		button::Button::Joypad1Up |
		button::Button::Joypad2Up => joypad::Button::Up,
		button::Button::Joypad1Down |
		button::Button::Joypad2Down => joypad::Button::Down,
		button::Button::Joypad1Left |
		button::Button::Joypad2Left => joypad::Button::Left,
		button::Button::Joypad1Right |
		button::Button::Joypad2Right => joypad::Button::Right,
		button::Button::Start => joypad::Button::Start,
		button::Button::Select => joypad::Button::Select,
		_ => joypad::Button::A // dummy @TODO: Throw an error?
	}
}

/**
 * Ricoh 6502
 * Refer to https://wiki.nesdev.com/w/index.php/CPU
 */
pub struct Cpu {
	power_on: bool,

	// registers
	pc: Register<u16>,
	sp: Register<u8>,
	a: Register<u8>,
	x: Register<u8>,
	y: Register<u8>,
	p: CpuStatusRegister,

	// CPU inside RAM
	ram: Memory,

	// manage additional stall cycles eg. DMA or branch success
	stall_cycles: u16,

	input: Box<dyn Input>,

	// other devices
	ppu: Ppu,
	apu: Apu,
	joypad1: Joypad,
	joypad2: Joypad,
	rom: Rom
}

// interrupts

pub enum Interrupts {
	NMI,
	RESET,
	IRQ,
	BRK  // not interrupt but instruction
}

fn interrupt_handler_address(interrupt_type: Interrupts) -> u16 {
	match interrupt_type {
		Interrupts::NMI => 0xFFFA,
		Interrupts::RESET => 0xFFFC,
		Interrupts::IRQ => 0xFFFE,
		Interrupts::BRK => 0xFFFE
	}
}

enum InstructionTypes {
	INV,
	ADC,
	AND,
	ASL,
	BCC,
	BCS,
	BEQ,
	BIT,
	BMI,
	BNE,
	BPL,
	BRK,
	BVC,
	BVS,
	CLC,
	CLD,
	CLI,
	CLV,
	CMP,
	CPX,
	CPY,
	DEC,
	DEX,
	DEY,
	EOR,
	INC,
	INX,
	INY,
	JMP,
	JSR,
	LDA,
	LDX,
	LDY,
	LSR,
	NOP,
	ORA,
	PHA,
	PHP,
	PLA,
	PLP,
	ROL,
	ROR,
	RTI,
	RTS,
	SBC,
	SEC,
	SED,
	SEI,
	STA,
	STX,
	STY,
	TAX,
	TAY,
	TSX,
	TXA,
	TXS,
	TYA
}

fn instruction_name(instruction_type: InstructionTypes) -> &'static str {
	match instruction_type {
		InstructionTypes::INV => "inv",
		InstructionTypes::ADC => "adc",
		InstructionTypes::AND => "and",
		InstructionTypes::ASL => "asl",
		InstructionTypes::BCC => "bcc",
		InstructionTypes::BCS => "bcs",
		InstructionTypes::BEQ => "beq",
		InstructionTypes::BIT => "bit",
		InstructionTypes::BMI => "bmi",
		InstructionTypes::BNE => "bne",
		InstructionTypes::BPL => "bpl",
		InstructionTypes::BRK => "brk",
		InstructionTypes::BVC => "bvc",
		InstructionTypes::BVS => "bvs",
		InstructionTypes::CLC => "clc",
		InstructionTypes::CLD => "cld",
		InstructionTypes::CLI => "cli",
		InstructionTypes::CLV => "clv",
		InstructionTypes::CMP => "cmp",
		InstructionTypes::CPX => "cpx",
		InstructionTypes::CPY => "cpy",
		InstructionTypes::DEC => "dec",
		InstructionTypes::DEX => "dex",
		InstructionTypes::DEY => "dey",
		InstructionTypes::EOR => "eor",
		InstructionTypes::INC => "inc",
		InstructionTypes::INX => "inx",
		InstructionTypes::INY => "iny",
		InstructionTypes::JMP => "jmp",
		InstructionTypes::JSR => "jsr",
		InstructionTypes::LDA => "lda",
		InstructionTypes::LDX => "ldx",
		InstructionTypes::LDY => "ldy",
		InstructionTypes::LSR => "lsr",
		InstructionTypes::NOP => "nop",
		InstructionTypes::ORA => "qra",
		InstructionTypes::PHA => "pha",
		InstructionTypes::PHP => "php",
		InstructionTypes::PLA => "pla",
		InstructionTypes::PLP => "plp",
		InstructionTypes::ROL => "rol",
		InstructionTypes::ROR => "ror",
		InstructionTypes::RTI => "rti",
		InstructionTypes::RTS => "rts",
		InstructionTypes::SBC => "sbc",
		InstructionTypes::SEC => "sec",
		InstructionTypes::SED => "sed",
		InstructionTypes::SEI => "sei",
		InstructionTypes::STA => "sta",
		InstructionTypes::STX => "stx",
		InstructionTypes::STY => "sty",
		InstructionTypes::TAX => "tax",
		InstructionTypes::TAY => "tay",
		InstructionTypes::TSX => "tsx",
		InstructionTypes::TXA => "txa",
		InstructionTypes::TXS => "txs",
		InstructionTypes::TYA => "tya"
	}
}

enum AddressingModes {
	Immediate,
	Absolute,
	IndexedAbsoluteX,
	IndexedAbsoluteY,
	ZeroPage,
	IndexedZeroPageX,
	IndexedZeroPageY,
	Implied,
	Accumulator,
	Indirect,
	IndexedIndirectX,
	IndexedIndirectY,
	Relative
}

struct Operation {
	instruction_type: InstructionTypes,
	cycle: u8,
	addressing_mode: AddressingModes
}

// @TODO: Replace with static array?
fn operation(opc: u8) -> Operation {
	match opc {
		0x00 => Operation {
			instruction_type: InstructionTypes::BRK,
			cycle: 7,
			addressing_mode: AddressingModes::Implied
		},
		0x01 => Operation {
			instruction_type: InstructionTypes::ORA,
			cycle: 6,
			addressing_mode: AddressingModes::IndexedIndirectX
		},
		// 0x02 => invalid
		// 0x03 => invalid
		// 0x04 => invalid
		0x05 => Operation {
			instruction_type: InstructionTypes::ORA,
			cycle: 3,
			addressing_mode: AddressingModes::ZeroPage
		},
		0x06 => Operation {
			instruction_type: InstructionTypes::ASL,
			cycle: 5,
			addressing_mode: AddressingModes::ZeroPage
		},
		// 0x07 => invalid
		0x08 => Operation {
			instruction_type: InstructionTypes::PHP,
			cycle: 3,
			addressing_mode: AddressingModes::Implied
		},
		0x09 => Operation {
			instruction_type: InstructionTypes::ORA,
			cycle: 2,
			addressing_mode: AddressingModes::Immediate
		},
		0x0A => Operation {
			instruction_type: InstructionTypes::ASL,
			cycle: 2,
			addressing_mode: AddressingModes::Accumulator
		},
		// 0x0B => invalid
		// 0x0C => invalid
		0x0D => Operation {
			instruction_type: InstructionTypes::ORA,
			cycle: 4,
			addressing_mode: AddressingModes::Absolute
		},
		0x0E => Operation {
			instruction_type: InstructionTypes::ASL,
			cycle: 6,
			addressing_mode: AddressingModes::Absolute
		},
		// 0x0F => invalid
		0x10 => Operation {
			instruction_type: InstructionTypes::BPL,
			cycle: 2, // +1 if branch succeeds, +2 if to a new page
			addressing_mode: AddressingModes::Relative
		},
		0x11 => Operation {
			instruction_type: InstructionTypes::ORA,
			cycle: 5, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedIndirectY
		},
		// 0x12 => invalid
		// 0x13 => invalid
		// 0x14 => invalid
		0x15 => Operation {
			instruction_type: InstructionTypes::ORA,
			cycle: 4,
			addressing_mode: AddressingModes::IndexedZeroPageX
		},
		0x16 => Operation {
			instruction_type: InstructionTypes::ASL,
			cycle: 6,
			addressing_mode: AddressingModes::IndexedZeroPageX
		},
		// 0x17 => invalid
		0x18 => Operation {
			instruction_type: InstructionTypes::CLC,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		0x19 => Operation {
			instruction_type: InstructionTypes::ORA,
			cycle: 4, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedAbsoluteY
		},
		// 0x1A => invalid
		// 0x1B => invalid
		// 0x1C => invalid
		0x1D => Operation {
			instruction_type: InstructionTypes::ORA,
			cycle: 4, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedAbsoluteX
		},
		0x1E => Operation {
			instruction_type: InstructionTypes::ASL,
			cycle: 7,
			addressing_mode: AddressingModes::IndexedAbsoluteX
		},
		// 0x1F => invalid
		0x20 => Operation {
			instruction_type: InstructionTypes::JSR,
			cycle: 6,
			addressing_mode: AddressingModes::Absolute
		},
		0x21 => Operation {
			instruction_type: InstructionTypes::AND,
			cycle: 6,
			addressing_mode: AddressingModes::IndexedIndirectX
		},
		// 0x22 => invalid
		// 0x23 => invalid
		0x24 => Operation {
			instruction_type: InstructionTypes::BIT,
			cycle: 3,
			addressing_mode: AddressingModes::ZeroPage
		},
		0x25 => Operation {
			instruction_type: InstructionTypes::AND,
			cycle: 3,
			addressing_mode: AddressingModes::ZeroPage
		},
		0x26 => Operation {
			instruction_type: InstructionTypes::ROL,
			cycle: 5,
			addressing_mode: AddressingModes::ZeroPage
		},
		// 0x27 => invalid
		0x28 => Operation {
			instruction_type: InstructionTypes::PLP,
			cycle: 4,
			addressing_mode: AddressingModes::Implied
		},
		0x29 => Operation {
			instruction_type: InstructionTypes::AND,
			cycle: 2,
			addressing_mode: AddressingModes::Immediate
		},
		0x2A => Operation {
			instruction_type: InstructionTypes::ROL,
			cycle: 2,
			addressing_mode: AddressingModes::Accumulator
		},
		// 0x2B => invalid
		0x2C => Operation {
			instruction_type: InstructionTypes::BIT,
			cycle: 4,
			addressing_mode: AddressingModes::Absolute
		},
		0x2D => Operation {
			instruction_type: InstructionTypes::AND,
			cycle: 4,
			addressing_mode: AddressingModes::Absolute
		},
		0x2E => Operation {
			instruction_type: InstructionTypes::ROL,
			cycle: 6,
			addressing_mode: AddressingModes::Absolute
		},
		// 0x2F => invalid
		0x30 => Operation {
			instruction_type: InstructionTypes::BMI,
			cycle: 2, // +1 if branch succeeds, +2 if to a new page
			addressing_mode: AddressingModes::Relative
		},
		0x31 => Operation {
			instruction_type: InstructionTypes::AND,
			cycle: 5, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedIndirectY
		},
		// 0x32 => invalid
		// 0x33 => invalid
		// 0x34 => invalid
		0x35 => Operation {
			instruction_type: InstructionTypes::AND,
			cycle: 4,
			addressing_mode: AddressingModes::IndexedZeroPageX
		},
		0x36 => Operation {
			instruction_type: InstructionTypes::ROL,
			cycle: 6,
			addressing_mode: AddressingModes::IndexedZeroPageX
		},
		// 0x37 => invalid
		0x38 => Operation {
			instruction_type: InstructionTypes::SEC,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		0x39 => Operation {
			instruction_type: InstructionTypes::AND,
			cycle: 4, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedAbsoluteY
		},
		// 0x3A => invalid
		// 0x3B => invalid
		// 0x3C => invalid
		0x3D => Operation {
			instruction_type: InstructionTypes::AND,
			cycle: 4, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedAbsoluteX
		},
		0x3E => Operation {
			instruction_type: InstructionTypes::ROL,
			cycle: 7,
			addressing_mode: AddressingModes::IndexedAbsoluteX
		},
		// 0x3F => invalid
		0x40 => Operation {
			instruction_type: InstructionTypes::RTI,
			cycle: 6,
			addressing_mode: AddressingModes::Implied
		},
		0x41 => Operation {
			instruction_type: InstructionTypes::EOR,
			cycle: 6,
			addressing_mode: AddressingModes::IndexedIndirectX
		},
		// 0x42 => invalid
		// 0x43 => invalid
		// 0x44 => invalid
		0x45 => Operation {
			instruction_type: InstructionTypes::EOR,
			cycle: 3,
			addressing_mode: AddressingModes::ZeroPage
		},
		0x46 => Operation {
			instruction_type: InstructionTypes::LSR,
			cycle: 5,
			addressing_mode: AddressingModes::ZeroPage
		},
		// 0x47 => invalid
		0x48 => Operation {
			instruction_type: InstructionTypes::PHA,
			cycle: 3,
			addressing_mode: AddressingModes::Implied
		},
		0x49 => Operation {
			instruction_type: InstructionTypes::EOR,
			cycle: 2,
			addressing_mode: AddressingModes::Immediate
		},
		0x4A => Operation {
			instruction_type: InstructionTypes::LSR,
			cycle: 2,
			addressing_mode: AddressingModes::Accumulator
		},
		// 0x4B => invalid
		0x4C => Operation {
			instruction_type: InstructionTypes::JMP,
			cycle: 3,
			addressing_mode: AddressingModes::Absolute
		},
		0x4D => Operation {
			instruction_type: InstructionTypes::EOR,
			cycle: 4,
			addressing_mode: AddressingModes::Absolute
		},
		0x4E => Operation {
			instruction_type: InstructionTypes::LSR,
			cycle: 6,
			addressing_mode: AddressingModes::Absolute
		},
		// 0x4F => invalid
		0x50 => Operation {
			instruction_type: InstructionTypes::BVC,
			cycle: 2, // +1 if branch succeeds, +2 if to a new page
			addressing_mode: AddressingModes::Relative
		},
		0x51 => Operation {
			instruction_type: InstructionTypes::EOR,
			cycle: 5, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedIndirectY
		},
		// 0x52 => invalid
		// 0x53 => invalid
		// 0x54 => invalid
		0x55 => Operation {
			instruction_type: InstructionTypes::EOR,
			cycle: 4,
			addressing_mode: AddressingModes::IndexedZeroPageX
		},
		0x56 => Operation {
			instruction_type: InstructionTypes::LSR,
			cycle: 6,
			addressing_mode: AddressingModes::IndexedZeroPageX
		},
		// 0x57 => invalid
		0x58 => Operation {
			instruction_type: InstructionTypes::CLI,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		0x59 => Operation {
			instruction_type: InstructionTypes::EOR,
			cycle: 4, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedAbsoluteY
		},
		// 0x5A => invalid
		// 0x5B => invalid
		// 0x5C => invalid
		0x5D => Operation {
			instruction_type: InstructionTypes::EOR,
			cycle: 4, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedAbsoluteX
		},
		0x5E => Operation {
			instruction_type: InstructionTypes::LSR,
			cycle: 7,
			addressing_mode: AddressingModes::IndexedAbsoluteX
		},
		// 0x5F => invalid
		0x60 => Operation {
			instruction_type: InstructionTypes::RTS,
			cycle: 6,
			addressing_mode: AddressingModes::Implied
		},
		0x61 => Operation {
			instruction_type: InstructionTypes::ADC,
			cycle: 6,
			addressing_mode: AddressingModes::IndexedIndirectX
		},
		// 0x62 => invalid
		// 0x63 => invalid
		// 0x64 => invalid
		0x65 => Operation {
			instruction_type: InstructionTypes::ADC,
			cycle: 3,
			addressing_mode: AddressingModes::ZeroPage
		},
		0x66 => Operation {
			instruction_type: InstructionTypes::ROR,
			cycle: 5,
			addressing_mode: AddressingModes::ZeroPage
		},
		// 0x67 => invalid
		0x68 => Operation {
			instruction_type: InstructionTypes::PLA,
			cycle: 4,
			addressing_mode: AddressingModes::Implied
		},
		0x69 => Operation {
			instruction_type: InstructionTypes::ADC,
			cycle: 2,
			addressing_mode: AddressingModes::Immediate
		},
		0x6A => Operation {
			instruction_type: InstructionTypes::ROR,
			cycle: 2,
			addressing_mode: AddressingModes::Accumulator
		},
		// 0x6B => invalid
		0x6C => Operation {
			instruction_type: InstructionTypes::JMP,
			cycle: 5,
			addressing_mode: AddressingModes::Indirect
		},
		0x6D => Operation {
			instruction_type: InstructionTypes::ADC,
			cycle: 4,
			addressing_mode: AddressingModes::Absolute
		},
		0x6E => Operation {
			instruction_type: InstructionTypes::ROR,
			cycle: 6,
			addressing_mode: AddressingModes::Absolute
		},
		// 0x6F => invalid
		0x70 => Operation {
			instruction_type: InstructionTypes::BVS,
			cycle: 2, // +1 if branch succeeds, +2 if to a new page
			addressing_mode: AddressingModes::Relative
		},
		0x71 => Operation {
			instruction_type: InstructionTypes::ADC,
			cycle: 5, // @TODO +1 if page crossed
			addressing_mode: AddressingModes::IndexedIndirectY
		},
		// 0x72 => invalid
		// 0x73 => invalid
		// 0x74 => invalid
		0x75 => Operation {
			instruction_type: InstructionTypes::ADC,
			cycle: 4,
			addressing_mode: AddressingModes::IndexedZeroPageX
		},
		0x76 => Operation {
			instruction_type: InstructionTypes::ROR,
			cycle: 6,
			addressing_mode: AddressingModes::IndexedZeroPageX
		},
		// 0x77 => invalid
		0x78 => Operation {
			instruction_type: InstructionTypes::SEI,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		0x79 => Operation {
			instruction_type: InstructionTypes::ADC,
			cycle: 4, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedAbsoluteY
		},
		// 0x7A => invalid
		// 0x7B => invalid
		// 0x7C => invalid
		0x7D => Operation {
			instruction_type: InstructionTypes::ADC,
			cycle: 4, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedAbsoluteX
		},
		0x7E => Operation {
			instruction_type: InstructionTypes::ROR,
			cycle: 7,
			addressing_mode: AddressingModes::IndexedAbsoluteX
		},
		// 0x7F => invalid
		// 0x80 => invalid
		0x81 => Operation {
			instruction_type: InstructionTypes::STA,
			cycle: 6,
			addressing_mode: AddressingModes::IndexedIndirectX
		},
		// 0x82 => invalid
		// 0x83 => invalid
		0x84 => Operation {
			instruction_type: InstructionTypes::STY,
			cycle: 3,
			addressing_mode: AddressingModes::ZeroPage
		},
		0x85 => Operation {
			instruction_type: InstructionTypes::STA,
			cycle: 3,
			addressing_mode: AddressingModes::ZeroPage
		},
		0x86 => Operation {
			instruction_type: InstructionTypes::STX,
			cycle: 3,
			addressing_mode: AddressingModes::ZeroPage
		},
		// 0x87 => invalid
		0x88 => Operation {
			instruction_type: InstructionTypes::DEY,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		// 0x89 => invalid
		0x8A => Operation {
			instruction_type: InstructionTypes::TXA,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		// 0x8B => invalid
		0x8C => Operation {
			instruction_type: InstructionTypes::STY,
			cycle: 4,
			addressing_mode: AddressingModes::Absolute
		},
		0x8D => Operation {
			instruction_type: InstructionTypes::STA,
			cycle: 4,
			addressing_mode: AddressingModes::Absolute
		},
		0x8E => Operation {
			instruction_type: InstructionTypes::STX,
			cycle: 4,
			addressing_mode: AddressingModes::Absolute
		},
		// 0x8F => invalid
		0x90 => Operation {
			instruction_type: InstructionTypes::BCC,
			cycle: 2, // +1 if branch suceeds, +2 if to a new page
			addressing_mode: AddressingModes::Relative
		},
		0x91 => Operation {
			instruction_type: InstructionTypes::STA,
			cycle: 6,
			addressing_mode: AddressingModes::IndexedIndirectY
		},
		// 0x92 => invalid
		// 0x93 => invalid
		0x94 => Operation {
			instruction_type: InstructionTypes::STY,
			cycle: 4,
			addressing_mode: AddressingModes::IndexedZeroPageX
		},
		0x95 => Operation {
			instruction_type: InstructionTypes::STA,
			cycle: 4,
			addressing_mode: AddressingModes::IndexedZeroPageX
		},
		0x96 => Operation {
			instruction_type: InstructionTypes::STX,
			cycle: 4,
			addressing_mode: AddressingModes::IndexedZeroPageY
		},
		// 0x97 => invalid
		0x98 => Operation {
			instruction_type: InstructionTypes::TYA,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		0x99 => Operation {
			instruction_type: InstructionTypes::STA,
			cycle: 5,
			addressing_mode: AddressingModes::IndexedAbsoluteY
		},
		0x9A => Operation {
			instruction_type: InstructionTypes::TXS,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		// 0x9B => invalid
		// 0x9C => invalid
		0x9D => Operation {
			instruction_type: InstructionTypes::STA,
			cycle: 5,
			addressing_mode: AddressingModes::IndexedAbsoluteX
		},
		// 0x9E => invalid
		// 0x9F => invalid
		0xA0 => Operation {
			instruction_type: InstructionTypes::LDY,
			cycle: 2,
			addressing_mode: AddressingModes::Immediate
		},
		0xA1 => Operation {
			instruction_type: InstructionTypes::LDA,
			cycle: 6,
			addressing_mode: AddressingModes::IndexedIndirectX
		},
		0xA2 => Operation {
			instruction_type: InstructionTypes::LDX,
			cycle: 2,
			addressing_mode: AddressingModes::Immediate
		},
		// 0xA3 => invalid
		0xA4 => Operation {
			instruction_type: InstructionTypes::LDY,
			cycle: 3,
			addressing_mode: AddressingModes::ZeroPage
		},
		0xA5 => Operation {
			instruction_type: InstructionTypes::LDA,
			cycle: 3,
			addressing_mode: AddressingModes::ZeroPage
		},
		0xA6 => Operation {
			instruction_type: InstructionTypes::LDX,
			cycle: 3,
			addressing_mode: AddressingModes::ZeroPage
		},
		// 0xA7 => invalid
		0xA8 => Operation {
			instruction_type: InstructionTypes::TAY,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		0xA9 => Operation {
			instruction_type: InstructionTypes::LDA,
			cycle: 2,
			addressing_mode: AddressingModes::Immediate
		},
		0xAA => Operation {
			instruction_type: InstructionTypes::TAX,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		// 0xAB => invalid
		0xAC => Operation {
			instruction_type: InstructionTypes::LDY,
			cycle: 4,
			addressing_mode: AddressingModes::Absolute
		},
		0xAD => Operation {
			instruction_type: InstructionTypes::LDA,
			cycle: 4,
			addressing_mode: AddressingModes::Absolute
		},
		0xAE => Operation {
			instruction_type: InstructionTypes::LDX,
			cycle: 4,
			addressing_mode: AddressingModes::Absolute
		},
		// 0xAF => invalid
		0xB0 => Operation {
			instruction_type: InstructionTypes::BCS,
			cycle: 2, // +1 if branch succeeds, +2 if to a new page
			addressing_mode: AddressingModes::Relative
		},
		0xB1 => Operation {
			instruction_type: InstructionTypes::LDA,
			cycle: 5, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedIndirectY
		},
		// 0xB2 => invalid
		// 0xB3 => invalid
		0xB4 => Operation {
			instruction_type: InstructionTypes::LDY,
			cycle: 4,
			addressing_mode: AddressingModes::IndexedZeroPageX
		},
		0xB5 => Operation {
			instruction_type: InstructionTypes::LDA,
			cycle: 4,
			addressing_mode: AddressingModes::IndexedZeroPageX
		},
		0xB6 => Operation {
			instruction_type: InstructionTypes::LDX,
			cycle: 4,
			addressing_mode: AddressingModes::IndexedZeroPageY
		},
		// 0xB7 => invalid
		0xB8 => Operation {
			instruction_type: InstructionTypes::CLV,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		0xB9 => Operation {
			instruction_type: InstructionTypes::LDA,
			cycle: 4, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedAbsoluteY
		},
		0xBA => Operation {
			instruction_type: InstructionTypes::TSX,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		// 0xBB => invalid
		0xBC => Operation {
			instruction_type: InstructionTypes::LDY,
			cycle: 4, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedAbsoluteX
		},
		0xBD => Operation {
			instruction_type: InstructionTypes::LDA,
			cycle: 4, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedAbsoluteX
		},
		0xBE => Operation {
			instruction_type: InstructionTypes::LDX,
			cycle: 4, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedAbsoluteY
		},
		// 0xBF => invalid
		0xC0 => Operation {
			instruction_type: InstructionTypes::CPY,
			cycle: 2,
			addressing_mode: AddressingModes::Immediate
		},
		0xC1 => Operation {
			instruction_type: InstructionTypes::CMP,
			cycle: 6,
			addressing_mode: AddressingModes::IndexedIndirectX
		},
		// 0xC2 => invalid
		// 0xC3 => invalid
		0xC4 => Operation {
			instruction_type: InstructionTypes::CPY,
			cycle: 3,
			addressing_mode: AddressingModes::ZeroPage
		},
		0xC5 => Operation {
			instruction_type: InstructionTypes::CMP,
			cycle: 3,
			addressing_mode: AddressingModes::ZeroPage
		},
		0xC6 => Operation {
			instruction_type: InstructionTypes::DEC,
			cycle: 5,
			addressing_mode: AddressingModes::ZeroPage
		},
		// 0xC7 => invalid
		0xC8 => Operation {
			instruction_type: InstructionTypes::INY,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		0xC9 => Operation {
			instruction_type: InstructionTypes::CMP,
			cycle: 2,
			addressing_mode: AddressingModes::Immediate
		},
		0xCA => Operation {
			instruction_type: InstructionTypes::DEX,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		// 0xCB => invalid
		0xCC => Operation {
			instruction_type: InstructionTypes::CPY,
			cycle: 4,
			addressing_mode: AddressingModes::Absolute
		},
		0xCD => Operation {
			instruction_type: InstructionTypes::CMP,
			cycle: 4,
			addressing_mode: AddressingModes::Absolute
		},
		0xCE => Operation {
			instruction_type: InstructionTypes::DEC,
			cycle: 6,
			addressing_mode: AddressingModes::Absolute
		},
		// 0xCF => invalid
		0xD0 => Operation {
			instruction_type: InstructionTypes::BNE,
			cycle: 2, // +1 if branch succeeds, +2 if to a new page
			addressing_mode: AddressingModes::Relative
		},
		0xD1 => Operation {
			instruction_type: InstructionTypes::CMP,
			cycle: 5, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedIndirectY
		},
		// 0xD2 => invalid
		// 0xD3 => invalid
		// 0xD4 => invalid
		0xD5 => Operation {
			instruction_type: InstructionTypes::CMP,
			cycle: 4,
			addressing_mode: AddressingModes::IndexedZeroPageX
		},
		0xD6 => Operation {
			instruction_type: InstructionTypes::DEC,
			cycle: 6,
			addressing_mode: AddressingModes::IndexedZeroPageX
		},
		// 0xD7 => invalid
		0xD8 => Operation {
			instruction_type: InstructionTypes::CLD,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		0xD9 => Operation {
			instruction_type: InstructionTypes::CMP,
			cycle: 4, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedAbsoluteY
		},
		// 0xDA => invalid
		// 0xDB => invalid
		// 0xDC => invalid
		0xDD => Operation {
			instruction_type: InstructionTypes::CMP,
			cycle: 4, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedAbsoluteX
		},
		0xDE => Operation {
			instruction_type: InstructionTypes::DEC,
			cycle: 7,
			addressing_mode: AddressingModes::IndexedAbsoluteX
		},
		// 0xDF => invalid
		0xE0 => Operation {
			instruction_type: InstructionTypes::CPX,
			cycle: 2,
			addressing_mode: AddressingModes::Immediate
		},
		0xE1 => Operation {
			instruction_type: InstructionTypes::SBC,
			cycle: 6,
			addressing_mode: AddressingModes::IndexedIndirectX
		},
		// 0xE2 => invalid
		// 0xE3 => invalid
		0xE4 => Operation {
			instruction_type: InstructionTypes::CPX,
			cycle: 3,
			addressing_mode: AddressingModes::ZeroPage
		},
		0xE5 => Operation {
			instruction_type: InstructionTypes::SBC,
			cycle: 3,
			addressing_mode: AddressingModes::ZeroPage
		},
		0xE6 => Operation {
			instruction_type: InstructionTypes::INC,
			cycle: 5,
			addressing_mode: AddressingModes::ZeroPage
		},
		// 0xE7 => invalid
		0xE8 => Operation {
			instruction_type: InstructionTypes::INX,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		0xE9 => Operation {
			instruction_type: InstructionTypes::SBC,
			cycle: 2,
			addressing_mode: AddressingModes::Immediate
		},
		0xEA => Operation {
			instruction_type: InstructionTypes::NOP,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		// 0xEB => invalid
		0xEC => Operation {
			instruction_type: InstructionTypes::CPX,
			cycle: 4,
			addressing_mode: AddressingModes::Absolute
		},
		0xED => Operation {
			instruction_type: InstructionTypes::SBC,
			cycle: 4,
			addressing_mode: AddressingModes::Absolute
		},
		0xEE => Operation {
			instruction_type: InstructionTypes::INC,
			cycle: 6,
			addressing_mode: AddressingModes::Absolute
		},
		// 0xEF => invalid
		0xF0 => Operation {
			instruction_type: InstructionTypes::BEQ,
			cycle: 2, // +1 if branch succeeds, +2 if to a new page
			addressing_mode: AddressingModes::Relative
		},
		0xF1 => Operation {
			instruction_type: InstructionTypes::SBC,
			cycle: 5, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedIndirectY
		},
		// 0xF2 => invalid
		// 0xF3 => invalid
		// 0xF4 => invalid
		0xF5 => Operation {
			instruction_type: InstructionTypes::SBC,
			cycle: 4,
			addressing_mode: AddressingModes::IndexedZeroPageX
		},
		0xF6 => Operation {
			instruction_type: InstructionTypes::INC,
			cycle: 6,
			addressing_mode: AddressingModes::IndexedZeroPageX
		},
		// 0xF7 => invalid
		0xF8 => Operation {
			instruction_type: InstructionTypes::SED,
			cycle: 2,
			addressing_mode: AddressingModes::Implied
		},
		0xF9 => Operation {
			instruction_type: InstructionTypes::SBC,
			cycle: 4, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedAbsoluteY
		},
		// 0xFA => invalid
		// 0xFB => invalid
		// 0xFC => invalid
		0xFD => Operation {
			instruction_type: InstructionTypes::SBC,
			cycle: 4, // +1 if page crossed
			addressing_mode: AddressingModes::IndexedAbsoluteX
		},
		0xFE => Operation {
			instruction_type: InstructionTypes::INC,
			cycle: 7,
			addressing_mode: AddressingModes::IndexedAbsoluteX
		},
		// 0xFF => invalid
		_ => Operation {
			instruction_type: InstructionTypes::INV,
			cycle: 1,
			addressing_mode: AddressingModes::Immediate // dummy
		}
	}
}

impl Cpu {
	pub fn new(input: Box<dyn Input>, display: Box<dyn Display>, audio: Box<dyn Audio>) -> Self {
		Cpu {
			power_on: false,
			pc: Register::<u16>::new(),
			sp: Register::<u8>::new(),
			a: Register::<u8>::new(),
			x: Register::<u8>::new(),
			y: Register::<u8>::new(),
			p: CpuStatusRegister::new(),
			ram: Memory::new(vec![0; 64 * 1024]), // 64KB
			stall_cycles: 0,
			input: input,
			ppu: Ppu::new(display),
			apu: Apu::new(audio),
			joypad1: Joypad::new(),
			joypad2: Joypad::new(),
			rom: Rom::new(vec![0; HEADER_SIZE]) // dummy
		}
	}

	pub fn set_rom(&mut self, rom: Rom) {
		self.rom = rom;
	}

	pub fn bootup(&mut self) {
		self.power_on = true;
		self.bootup_internal();
		self.ppu.bootup();
		self.apu.bootup();
	}

	fn bootup_internal(&mut self) {
		self.p.store(0x34);
		self.a.clear();
		self.x.clear();
		self.y.clear();
		self.sp.store(0xFD);

		for i in 0..0x10 {
			self.store(0x4000 + i, 0);
		}

		self.store(0x4015, 0);
		self.store(0x4017, 0);

		self.interrupt(Interrupts::RESET);
	}

	pub fn reset(&mut self) {
		self.reset_internal();
		self.ppu.reset();
		self.apu.reset();
		self.interrupt(Interrupts::RESET);
	}

	pub fn is_power_on(&self) -> bool {
		self.power_on
	}

	fn reset_internal(&mut self) {
		self.sp.sub(3);
		self.p.set_i();
	}

	pub fn get_ppu(&self) -> &Ppu {
		&self.ppu
	}

	pub fn get_mut_apu(&mut self) -> &mut Apu {
		&mut self.apu
	}

	pub fn get_mut_input(&mut self) -> &mut Box<dyn Input> {
		&mut self.input
	}

	//

	pub fn step(&mut self) {
		let stall_cycles = self.step_internal();
		for _i in 0..stall_cycles * 3 {
			self.ppu.step(&mut self.rom);
		}
		for _i in 0..stall_cycles {
			// No reference to CPU from APU so detecting if APU DMC needs
			// CPU memory data, loading data, and sending to APU if needed
			// @TODO: Simplify
			let dmc_sample_data = match self.apu.dmc_needs_cpu_memory_data() {
				true => {
					// The CPU is stalled for up to 4 CPU cycles
					// @TODO: Fix me
					self.stall_cycles += 4;
					self.load(self.apu.dmc_sample_address())
				}
				false => 0
			};
			self.apu.step(dmc_sample_data);
		}
	}

	pub fn step_frame(&mut self) {
		// Input handling should be here? Or under nes.rs?
		self.handle_inputs();
		// @TODO: More precise frame update detection?
		let ppu_frame = self.ppu.frame;
		loop {
			self.step();
			if ppu_frame != self.ppu.frame {
				break;
			}
		}
	}

	fn handle_inputs(&mut self) {
		while let Some((button, event)) = self.input.get_input() {
			match button {
				button::Button::Poweroff => {
					self.power_on = false;
				},
				button::Button::Reset => {
					self.reset();
				},
				button::Button::Select |
				button::Button::Start |
				button::Button::Joypad1A |
				button::Button::Joypad1B |
				button::Button::Joypad1Up |
				button::Button::Joypad1Down |
				button::Button::Joypad1Left |
				button::Button::Joypad1Right => {
					self.joypad1.handle_input(to_joypad_button(button), event);
				},
				button::Button::Joypad2A |
				button::Button::Joypad2B |
				button::Button::Joypad2Up |
				button::Button::Joypad2Down |
				button::Button::Joypad2Left |
				button::Button::Joypad2Right => {
					self.joypad2.handle_input(to_joypad_button(button), event);
				}
			}
		}
	}

	fn step_internal(&mut self) -> u16 {
		// @TODO: What if both NMI and IRQ happen?
		if self.ppu.nmi_interrupted {
			self.ppu.nmi_interrupted = false;
			self.interrupt(Interrupts::NMI);
		}
		if self.ppu.irq_interrupted {
			self.ppu.irq_interrupted = false;
			self.interrupt(Interrupts::IRQ);
		}
		if self.apu.irq_interrupted {
			self.apu.irq_interrupted = false;
			self.interrupt(Interrupts::IRQ);
		}

		let opc = self.fetch();
		let op = self.decode(opc);
		self.operate(&op);
		let stall_cycles = self.stall_cycles;
		self.stall_cycles = 0;
		stall_cycles + op.cycle as u16
	}

	fn fetch(&mut self) -> u8 {
		let opc = self.load(self.pc.load());
		self.pc.increment();
		opc
	}

	fn decode(&self, opc: u8) -> Operation {
		operation(opc)
	}

	fn jump_to_interrupt_handler(&mut self, interrupt_type: Interrupts) {
		let address = interrupt_handler_address(interrupt_type);
		let value = self.load_2bytes(address);
		self.pc.store(value);
	}

	fn do_branch(&mut self, op: &Operation, flag: bool) {
		let result = self.load_with_addressing_mode(&op);
		if flag {
			// stall_cycle + 1 if branch succeeds
			self.stall_cycles += 1;
			let current_page = self.pc.load() & 0xff00;
			self.pc.add(result);
			if current_page != (self.pc.load() & 0xff00) {
				// stall_cycle + 1 if across page
				self.stall_cycles += 1;
			}
		}
	}

	// @TODO: Clean up if needed
	fn operate(&mut self, op: &Operation) {
		match op.instruction_type {
			InstructionTypes::ADC => {
				let src1 = self.a.load();
				let src2 = self.load_with_addressing_mode(&op);
				let c = match self.p.is_c() {
					true => 1,
					false => 0
				} as u16;
				let result = (src1 as u16).wrapping_add(src2).wrapping_add(c);
				self.a.store(result as u8);
				self.update_n(result);
				self.update_z(result);
				self.update_c(result);
				if !(((src1 ^ src2 as u8) & 0x80) != 0) && ((src2 as u8 ^ result as u8) & 0x80) != 0 {
					self.p.set_v();
				} else {
					self.p.clear_v();
				}
			},
			InstructionTypes::AND => {
				let src1 = self.a.load();
				let src2 = self.load_with_addressing_mode(&op);
				let result = (src1 as u16) & src2;
				self.a.store(result as u8);
				self.update_n(result);
				self.update_z(result);
			},
			InstructionTypes::ASL => {
				let result = self.update_memory_with_addressing_mode(op, |src: u8| {
					(src as u16) << 1
				});
				self.update_n(result);
				self.update_z(result);
				self.update_c(result);
			},
			InstructionTypes::BCC => {
				let flag = !self.p.is_c();
				self.do_branch(&op, flag);
			},
			InstructionTypes::BCS => {
				let flag = self.p.is_c();
				self.do_branch(&op, flag);
			},
			InstructionTypes::BEQ => {
				let flag = self.p.is_z();
				self.do_branch(&op, flag);
			},
			// @TODO: check logic
			InstructionTypes::BIT => {
				let src1 = self.a.load();
				let src2 = self.load_with_addressing_mode(&op);
				let result = (src1 as u16) & src2;
				self.update_n(src2);
				self.update_z(result);
				if (src2 & 0x40) == 0 {
					self.p.clear_v();
				} else {
					self.p.set_v();
				}
			},
			InstructionTypes::BMI => {
				let flag = self.p.is_n();
				self.do_branch(&op, flag);
			},
			InstructionTypes::BNE => {
				let flag = !self.p.is_z();
				self.do_branch(&op, flag);
			},
			InstructionTypes::BPL => {
				let flag = !self.p.is_n();
				self.do_branch(&op, flag);
			},
			InstructionTypes::BRK => {
				self.pc.increment(); // seems like necessary
				self.p.set_a();
				self.p.set_b();
				self.interrupt(Interrupts::BRK);
			},
			InstructionTypes::BVC => {
				let flag = !self.p.is_v();
				self.do_branch(&op, flag);
			},
			InstructionTypes::BVS => {
				let flag = self.p.is_v();
				self.do_branch(&op, flag);
			},
			InstructionTypes::CLC => {
				self.p.clear_c();
			},
			InstructionTypes::CLD => {
				self.p.clear_d();
			},
			InstructionTypes::CLI => {
				self.p.clear_i();
			},
			InstructionTypes::CLV => {
				self.p.clear_v();
			},
			InstructionTypes::CMP | InstructionTypes::CPX | InstructionTypes::CPY => {
				let src1 = match op.instruction_type {
					InstructionTypes::CMP => {
						self.a.load()
					},
					InstructionTypes::CPX => {
						self.x.load()
					},
					_ => { //InstructionTypes::CPY
						self.y.load()
					}
				};
				let src2 = self.load_with_addressing_mode(&op);
				let result = (src1 as u16).wrapping_sub(src2);
				self.update_n(result);
				self.update_z(result);
				if src1 as u16 >= src2 {
					self.p.set_c();
				} else {
					self.p.clear_c();
				}
			},
			InstructionTypes::DEC => {
				let result = self.update_memory_with_addressing_mode(op, |src: u8| {
					(src as u16).wrapping_sub(1)
				});
				self.update_n(result);
				self.update_z(result);
			},
			InstructionTypes::DEX | InstructionTypes::DEY => {
				let result = match op.instruction_type {
					InstructionTypes::DEX => {
						let src = self.x.load();
						let result = (src as u16).wrapping_sub(1);
						self.x.store(result as u8);
						result
					},
					_ => { // InstructionTypes::DEY
						let src = self.y.load();
						let result = (src as u16).wrapping_sub(1);
						self.y.store(result as u8);
						result
					}
				};
				self.update_n(result);
				self.update_z(result);
			},
			InstructionTypes::EOR => {
				let src1 = self.a.load();
				let src2 = self.load_with_addressing_mode(&op);
				let result = (src1 as u16) ^ src2;
				self.a.store(result as u8);
				self.update_n(result);
				self.update_z(result);
			},
			InstructionTypes::INC => {
				let result = self.update_memory_with_addressing_mode(op, |src: u8| {
					(src as u16).wrapping_add(1)
				});
				self.update_n(result);
				self.update_z(result);
			},
			InstructionTypes::INV => {
				// @TODO: Throw?
				println!("INV operation");
			},
			InstructionTypes::INX | InstructionTypes::INY => {
				let result = match op.instruction_type {
					InstructionTypes::INX => {
						let src = self.x.load();
						let result = (src as u16).wrapping_add(1);
						self.x.store(result as u8);
						result
					},
					_ => { // InstructionTypes::INY
						let src = self.y.load();
						let result = (src as u16).wrapping_add(1);
						self.y.store(result as u8);
						result
					}
				};
				self.update_n(result);
				self.update_z(result);
			},
			// TODO: check the logic.
			InstructionTypes::JMP => {
				let address = self.get_address_with_addressing_mode(op);
				self.pc.store(address);
			},
			// TODO: check the logic.
			InstructionTypes::JSR => {
				let address = self.get_address_with_addressing_mode(op);
				self.pc.decrement();
				let value = self.pc.load();
				self.push_stack_2bytes(value);
				self.pc.store(address);
			},
			InstructionTypes::LDA | InstructionTypes::LDX | InstructionTypes::LDY => {
				let result = match op.instruction_type {
					InstructionTypes::LDA => {
						let result = self.load_with_addressing_mode(&op);
						self.a.store(result as u8);
						result
					},
					InstructionTypes::LDX => {
						let result = self.load_with_addressing_mode(&op);
						self.x.store(result as u8);
						result
					},
					_ /*InstructionTypes::LDY*/ => {
						let result = self.load_with_addressing_mode(&op);
						self.y.store(result as u8);
						result
					}
				};
				self.update_n(result);
				self.update_z(result);
			},
			InstructionTypes::LSR => {
				let result = match op.addressing_mode {
					AddressingModes::Accumulator => {
						let src = self.a.load();
						if (src & 1) == 0 {
							self.p.clear_c();
						} else {
							self.p.set_c();
						}
						let result = (src as u16) >> 1;
						self.a.store(result as u8);
						result
					},
					_ => {
						let address = self.get_address_with_addressing_mode(op);
						let src = self.load(address);
						if (src & 1) == 0 {
							self.p.clear_c();
						} else {
							self.p.set_c();
						}
						let result = (src as u16) >> 1;
						self.store(address, result as u8);
						result
					}
				};
				self.p.clear_n();
				self.update_z(result);
			},
			InstructionTypes::NOP => {},
			InstructionTypes::ORA => {
				let src1 = self.a.load();
				let src2 = self.load_with_addressing_mode(op);
				let result = (src1 as u16) | src2;
				self.a.store(result as u8);
				self.update_n(result);
				self.update_z(result);
			},
			InstructionTypes::PHA => {
				let value = self.a.load();
				self.push_stack(value);
			},
			InstructionTypes::PHP => {
				self.p.set_a();
				self.p.set_b();
				let value = self.p.load();
				self.push_stack(value);
			},
			InstructionTypes::PLA => {
				let result = self.pop_stack() as u16;
				self.a.store(result as u8);
				self.update_n(result);
				self.update_z(result);
			},
			InstructionTypes::PLP => {
				let value = self.pop_stack();
				self.p.store(value);
			},
			InstructionTypes::ROL => {
				let result = match op.addressing_mode {
					AddressingModes::Accumulator => {
						let src = self.a.load();
						let c = match self.p.is_c() {
							true => 1,
							false => 0
						} as u16;
						let result = ((src as u16) << 1) | c;
						self.a.store(result as u8);
						result
					},
					_ => {
						let address = self.get_address_with_addressing_mode(op);
						let src = self.load(address);
						let c = match self.p.is_c() {
							true => 1,
							false => 0
						} as u16;
						let result = ((src as u16) << 1) | c;
						self.store(address, result as u8);
						result
					}
				};
				self.update_n(result);
				self.update_z(result);
				self.update_c(result);
			},
			InstructionTypes::ROR => {
				let result = match op.addressing_mode {
					AddressingModes::Accumulator => {
						let src = self.a.load();
						let c = match self.p.is_c() {
							true => 0x80,
							false => 0
						} as u16;
						let result = ((src as u16) >> 1) | c;
						self.a.store(result as u8);
						if (src & 1) == 0 {
							self.p.clear_c();
						} else {
							self.p.set_c();
						}
						result
					},
					_ => {
						let address = self.get_address_with_addressing_mode(op);
						let src = self.load(address);
						let c = match self.p.is_c() {
							true => 0x80,
							false => 0
						} as u16;
						let result = ((src as u16) >> 1) | c;
						self.store(address, result as u8);
						if (src & 1) == 0 {
							self.p.clear_c();
						} else {
							self.p.set_c();
						}
						result
					}
				};
				self.update_n(result);
				self.update_z(result);
			},
			// TODO: check logic.
			InstructionTypes::RTI => {
				let value = self.pop_stack();
				self.p.store(value);
				let value2 = self.pop_stack_2bytes();
				self.pc.store(value2);
			},
			// TODO: check logic.
			InstructionTypes::RTS => {
				let value = self.pop_stack_2bytes().wrapping_add(1);
				self.pc.store(value);
			},
			InstructionTypes::SBC => {
				let src1 = self.a.load();
				let src2 = self.load_with_addressing_mode(&op);
				let c = match self.p.is_c() {
					true => 0,
					false => 1
				} as u16;
				let result = (src1 as u16).wrapping_sub(src2).wrapping_sub(c);
				self.a.store(result as u8);
				self.update_n(result);
				self.update_z(result);
				// TODO: check if this logic is right.
				if src1 as u16 >= src2.wrapping_add(c) {
					self.p.set_c();
				} else {
					self.p.clear_c();
				}
				// TODO: implement right overflow logic.
				//       this is just a temporal logic.
				if ((src1 ^ result as u8) & 0x80) != 0 && ((src1 ^ src2 as u8) & 0x80) != 0 {
					self.p.set_v();
				} else {
					self.p.clear_v();
				}
			},
			InstructionTypes::SEC => {
				self.p.set_c();
			},
			InstructionTypes::SED => {
				self.p.set_d();
			},
			InstructionTypes::SEI => {
				self.p.set_i();
			},
			InstructionTypes::STA => {
				let value = self.a.load();
				self.store_with_addressing_mode(&op, value);
			},
			InstructionTypes::STX => {
				let value = self.x.load();
				self.store_with_addressing_mode(&op, value);
			},
			InstructionTypes::STY => {
				let value = self.y.load();
				self.store_with_addressing_mode(&op, value);
			},
			InstructionTypes::TAX => {
				let result = self.a.load() as u16;
				self.x.store(result as u8);
				self.update_n(result);
				self.update_z(result);
			},
			InstructionTypes::TAY => {
				let result = self.a.load() as u16;
				self.y.store(result as u8);
				self.update_n(result);
				self.update_z(result);
			},
			InstructionTypes::TSX => {
				let result = self.sp.load() as u16;
				self.x.store(result as u8);
				self.update_n(result);
				self.update_z(result);
			},
			InstructionTypes::TXA => {
				let result = self.x.load() as u16;
				self.a.store(result as u8);
				self.update_n(result);
				self.update_z(result);
			},
			InstructionTypes::TXS => {
				let result = self.x.load();
				self.sp.store(result);
			},
			InstructionTypes::TYA => {
				let result = self.y.load() as u16;
				self.a.store(result as u8);
				self.update_n(result);
				self.update_z(result);
			}
		}
	}

	pub fn load(&mut self, address: u16) -> u8 {
		// 0x0000 - 0x07FF: 2KB internal RAM
		// 0x0800 - 0x1FFF: Mirrors of 0x0000 - 0x07FF (repeats every 0x800 bytes)

		if address < 0x2000 {
			return self.ram.load((address & 0x07FF) as u32);
		}

		// 0x2000 - 0x2007: PPU registers
		// 0x2008 - 0x3FFF: Mirrors of 0x2000 - 0x2007 (repeats every 8 bytes)

		if address >= 0x2000 && address < 0x4000 {
			return self.ppu.load_register(address & 0x2007, &self.rom);
		}

		if address >= 0x4000 && address < 0x4014 {
			return self.apu.load_register(address);
		}

		if address == 0x4014 {
			return self.ppu.load_register(address, &self.rom);
		}

		if address == 0x4015 {
			return self.apu.load_register(address);
		}

		if address == 0x4016 {
			return self.joypad1.load_register();
		}

		if address == 0x4017 {
			return self.joypad2.load_register();
		}

		if address >= 0x4017 && address < 0x4020 {
			return self.apu.load_register(address);
		}

		if address >= 0x4020 && address < 0x6000 {
			return self.ram.load(address as u32);
		}

		if address >= 0x6000 && address < 0x8000 {
			return self.ram.load(address as u32);
		}

		if address >= 0x8000 {
			return self.rom.load(address as u32);
		}

		0 // dummy
	}

	fn load_2bytes(&mut self, address: u16) -> u16 {
		let byte_low = self.load(address) as u16;
		let byte_high = self.load(address.wrapping_add(1)) as u16;
		(byte_high << 8) | byte_low
	}

	fn load_2bytes_from_zeropage(&mut self, address: u16) -> u16 {
		self.ram.load((address & 0xff) as u32) as u16 | ((self.ram.load((address.wrapping_add(1) & 0xff) as u32) as u16) << 8)
	}

	fn load_2bytes_in_page(&mut self, address: u16) -> u16 {
		let addr1 = address;
		let addr2 = (address & 0xff00) | ((address.wrapping_add(1)) & 0xff);
		let byte_low = self.load(addr1) as u16;
		let byte_high = self.load(addr2) as u16;
		(byte_high << 8) | byte_low
	}

	fn store(&mut self, address: u16, value: u8) {
		// 0x0000 - 0x07FF: 2KB internal RAM
		// 0x0800 - 0x1FFF: Mirrors of 0x0000 - 0x07FF (repeats every 0x800 bytes)

		if address < 0x2000 {
			self.ram.store((address & 0x07FF) as u32, value);
		}

		// 0x2000 - 0x2007: PPU registers
		// 0x2008 - 0x3FFF: Mirrors of 0x2000 - 0x2007 (repeats every 8 bytes)

		if address >= 0x2000 && address < 0x4000 {
			self.ppu.store_register(address & 0x2007, value, &mut self.rom);
		}

		if address >= 0x4000 && address < 0x4014 {
			self.apu.store_register(address, value);
		}

		// @TODO: clean up

		if address == 0x4014 {
			self.ppu.store_register(address, value, &mut self.rom);

			// DMA.
			// Writing 0xXX will upload 256 bytes of data from CPU page
			// 0xXX00-0xXXFF to the internal PPU OAM.
			let offset = (value as u16) << 8;
			for i in 0..256 {
				let data = self.load(offset + i);
				self.ppu.store_register(0x2004, data, &mut self.rom);
			}

			// @TODO
			self.stall_cycles += 514;
		}

		if address == 0x4015 {
			self.apu.store_register(address, value);
		}

		if address == 0x4016 {
			self.joypad1.store_register(value);
			self.joypad2.store_register(value); // to clear the joypad2 state
		}

		if address >= 0x4017 && address < 0x4020 {
			self.apu.store_register(address, value);
		}

		// cartridge space

		if address >= 0x4020 && address < 0x6000 {
			self.ram.store(address as u32, value);
		}

		// 0x6000 - 0x7FFF: Battery Backed Save or Work RAM

		if address >= 0x6000 && address < 0x8000 {
			self.ram.store(address as u32, value);
		}

		// 0x8000 - 0xFFFF: ROM

		if address >= 0x8000 {
			self.rom.store(address as u32, value);
		}
	}

	pub fn interrupt(&mut self, interrupt_type: Interrupts) {
		// @TODO: Optimize

		match interrupt_type {
			Interrupts::IRQ => {
				if self.p.is_i() {
					return;
				}
			},
			_ => {}
		}

		match interrupt_type {
			Interrupts::RESET => {},
			_ => {
				match interrupt_type {
					Interrupts::BRK => {},
					_ => self.p.clear_b()
				};
				self.p.set_a();

				let value = self.pc.load();
				self.push_stack_2bytes(value);
				let value2 = self.p.load();
				self.push_stack(value2);
				self.p.set_i();
			}
		};

		self.jump_to_interrupt_handler(interrupt_type);
	}

	fn load_with_addressing_mode(&mut self, op: &Operation) -> u16 {
		match op.addressing_mode {
			AddressingModes::Accumulator => {
				self.a.load() as u16
			},
			_ => {
				let address = self.get_address_with_addressing_mode(&op);
				let value = self.load(address) as u16;
				match op.addressing_mode {
					// expects that relative addressing mode is used only for load.
					AddressingModes::Relative => {
						// TODO: confirm if this logic is right.
						if (value & 0x80) != 0 {
							value | 0xff00
						} else {
							value
						}
					},
					_ => value
				}
			}
		}
	}

	fn store_with_addressing_mode(&mut self, op: &Operation, value: u8) {
		match op.addressing_mode {
			AddressingModes::Accumulator => {
				self.a.store(value);
			},
			_ => {
				let address = self.get_address_with_addressing_mode(op);
				self.store(address, value);
			}
		};
	}

	fn update_memory_with_addressing_mode<F>(&mut self, op: &Operation, func: F) -> u16 where F: Fn(u8) -> u16 {
		match op.addressing_mode {
			AddressingModes::Accumulator => {
				let src = self.a.load();
				let result = func(src);
				self.a.store(result as u8);
				result
			},
			_ => {
				let address = self.get_address_with_addressing_mode(op);
				let src = self.load(address);
				let result = func(src);
				self.store(address, result as u8);
				result
			}
		}
	}

	fn get_address_with_addressing_mode(&mut self, op: &Operation) -> u16 {
		match op.addressing_mode {
			AddressingModes::Immediate | AddressingModes::Relative => {
				let address = self.pc.load();
				self.pc.increment();
				address
			},
			AddressingModes::Absolute | AddressingModes::IndexedAbsoluteX | AddressingModes::IndexedAbsoluteY => {
				let address = self.load_2bytes(self.pc.load());
				self.pc.increment_by_2();
				let effective_address = address.wrapping_add(match op.addressing_mode {
					AddressingModes::IndexedAbsoluteX => self.x.load(),
					AddressingModes::IndexedAbsoluteY => self.y.load(),
					_ => 0
				} as u16);
				match op.instruction_type {
					InstructionTypes::ADC |
					InstructionTypes::AND |
					InstructionTypes::CMP |
					InstructionTypes::EOR |
					InstructionTypes::LDA |
					InstructionTypes::LDY |
					InstructionTypes::LDX |
					InstructionTypes::ORA |
					InstructionTypes::SBC => {
						// stall_cycles + 1 if page is crossed
						if (address & 0xff00) != (effective_address & 0xff00) {
							self.stall_cycles += 1;
						}
					},
					_ => {}
				};
				effective_address
			},
			AddressingModes::ZeroPage | AddressingModes::IndexedZeroPageX | AddressingModes::IndexedZeroPageY => {
				let address = self.pc.load();
				let address2 = self.load(address) as u16;
				self.pc.increment();
				address2.wrapping_add(match op.addressing_mode {
					AddressingModes::IndexedZeroPageX => self.x.load(),
					AddressingModes::IndexedZeroPageY => self.y.load(),
					_ => 0
				} as u16) & 0xFF
			},
			AddressingModes::Indirect => {
				let address = self.pc.load();
				let tmp = self.load_2bytes(address);
				self.pc.increment_by_2();
				self.load_2bytes_in_page(tmp)
			},
			AddressingModes::IndexedIndirectX => {
				let address = self.pc.load();
				let tmp = self.load(address);
				self.pc.increment();
				self.load_2bytes_from_zeropage(((tmp.wrapping_add(self.x.load())) & 0xFF) as u16)
			},
			AddressingModes::IndexedIndirectY => {
				let address = self.pc.load();
				let tmp = self.load(address);
				self.pc.increment();
				let address2 = self.load_2bytes_from_zeropage(tmp as u16);
				let effective_address = address2.wrapping_add(self.y.load() as u16);
				match op.instruction_type {
					InstructionTypes::AND |
					InstructionTypes::CMP |
					InstructionTypes::EOR |
					InstructionTypes::LDA |
					InstructionTypes::ORA |
					InstructionTypes::SBC => {
						// stall_cycles + 1 if page is crossed
						if (address2 & 0xff00) != (effective_address & 0xff00) {
							self.stall_cycles += 1;
						}
					},
					_ => {}
				};
				effective_address
			},
			_ => {
				// @TODO: Throw?
				println!("Unknown addressing mode.");
				0
			}
		}
	}

	fn update_n(&mut self, value: u16) {
		if (value & 0x80) == 0 {
			self.p.clear_n();
		} else {
			self.p.set_n();
		}
	}

	fn update_z(&mut self, value: u16) {
		if (value & 0xff) == 0 {
			self.p.set_z();
		} else {
			self.p.clear_z();
		}
	}

	fn update_c(&mut self, value: u16) {
		if (value & 0x100) == 0 {
			self.p.clear_c();
		} else {
			self.p.set_c();
		}
	}

	fn get_stack_address(&self) -> u16 {
		self.sp.load() as u16 + 0x100
	}

	fn push_stack(&mut self, value: u8) {
		let address = self.get_stack_address();
		self.store(address, value);
		self.sp.decrement();
	}

	fn push_stack_2bytes(&mut self, value: u16) {
		let address = self.get_stack_address();
		self.store(address, ((value >> 8) & 0xff) as u8);
		self.sp.decrement();
		let address2 = self.get_stack_address();
		self.store(address2, (value & 0xff) as u8);
		self.sp.decrement();
	}

	fn pop_stack(&mut self) -> u8 {
		self.sp.increment();
		self.load(self.get_stack_address())
	}

	fn pop_stack_2bytes(&mut self) -> u16 {
		self.sp.increment();
		let byte_low = self.load(self.get_stack_address()) as u16;
		self.sp.increment();
		let byte_high = self.load(self.get_stack_address()) as u16;
		(byte_high << 8) | byte_low
	}

	pub fn dump(&mut self) -> String {
		let opc = self.load(self.pc.load());
		let op = self.decode(opc);
		"p:".to_owned() + &self.p.dump() + &" ".to_owned() +
		&"pc:".to_owned() + &self.pc.dump() + &format!("(0x{:02x})", opc) + &" ".to_owned() +
		&"sp:".to_owned() + &self.sp.dump() + &" ".to_owned() +
		&"a:".to_owned() + &self.a.dump() + &" ".to_owned() +
		&"x:".to_owned() + &self.x.dump() + &" ".to_owned() +
		&"y:".to_owned() + &self.y.dump() + &" ".to_owned() +
		instruction_name(op.instruction_type) + &" ".to_owned() +
		&self.dump_addressing_mode(op.addressing_mode, self.pc.load().wrapping_add(1))
	}

	fn dump_addressing_mode(&mut self, mode: AddressingModes, pc: u16) -> String {
		match mode {
			AddressingModes::Immediate => {
				"#".to_owned() + &format!("0x{:02x} ", self.load(pc)) +
				&"immediate".to_owned()
			},
			AddressingModes::Relative => {
				format!("0x{:02x} ", self.load(pc) as i8) +
				&"relative".to_owned()
			},
			AddressingModes::Absolute => {
				let address = self.load_2bytes(pc);
				format!("0x{:04x} ", address) +
				&format!("(0x{:02x}) ", self.load(address) as i8) +
				&"absolute".to_owned()
			},
			AddressingModes::IndexedAbsoluteX => {
				let address = self.load_2bytes(pc);
				format!("0x{:04x},X ", address) +
				&format!("(0x{:02x}) ", self.load((self.x.load() as u16).wrapping_add(address)) as i8) +
				&"indexed_absolute_x".to_owned()
			},
			AddressingModes::IndexedAbsoluteY => {
				let address = self.load_2bytes(pc);
				format!("0x{:04x},Y ", address) +
				&format!("(0x{:02x}) ", self.load((self.y.load() as u16).wrapping_add(address)) as i8) +
				&"indexed_absolute_y".to_owned()
			},
			AddressingModes::ZeroPage => {
				let address = self.load(pc);
				format!("0x{:02x} ", address) +
				&format!("(0x{:02x}) ", self.load(address as u16) as i8) +
				&"zero_page".to_owned()
			},
			AddressingModes::IndexedZeroPageX => {
				let address = self.load(pc);
				format!("0x{:02x},X ", address) +
				&format!("(0x{:02x}) ", self.load(self.x.load().wrapping_add(address) as u16) as i8) +
				&"indexed_zero_page_x".to_owned()
			},
			AddressingModes::IndexedZeroPageY => {
				let address = self.load(pc);
				format!("0x{:02x},Y ", address) +
				&format!("(0x{:02x}) ", self.load(self.y.load().wrapping_add(address) as u16) as i8) +
				&"indexed_zero_page_y".to_owned()
			},
			AddressingModes::Indirect => {
				let address = self.load_2bytes(pc);
				let address2 = self.load_2bytes(address);
				format!("0x{:04x} ", address) +
				&format!("(0x{:04x}(0x{:02x})) ", address2, self.load(address2) as i8) +
				&"indirect".to_owned()
			},
			AddressingModes::IndexedIndirectX => {
				let address = self.load(pc) as u16;
				let address2 = (self.x.load() as u16).wrapping_add(address);
				format!("0x{:02x},X ", address) +
				&format!("(0x{:04x}(0x{:02x})) ", address2, self.load(address2) as i8) +
				&"indexed_indirect_x".to_owned()
			},
			AddressingModes::IndexedIndirectY => {
				let address = self.load(pc) as u16;
				let address2 = self.load_2bytes_from_zeropage(address).wrapping_add(self.x.load() as u16);
				format!("0x{:02x},Y ", address) +
				&format!("(0x{:04x}(0x{:02x})) ", address2, self.load(address2) as i8) +
				&"indexed_indirect_y".to_owned()
			},
			AddressingModes::Accumulator => {
				format!("A0x{:02x} ", self.a.load()) +
				&"accumulator".to_owned()
			},
			_ => { "".to_owned() }
		}
	}
}

pub struct CpuStatusRegister {
	register: Register<u8>
}

impl CpuStatusRegister {
	pub fn new() -> Self {
		CpuStatusRegister {
			register: Register::<u8>::new()
		}
	}

	pub fn load(&self) -> u8 {
		self.register.load()
	}

	pub fn store(&mut self, value: u8) {
		self.register.store(value);
	}

	pub fn is_n(&self) -> bool {
		self.register.is_bit_set(7)
	}

	pub fn set_n(&mut self) {
		self.register.set_bit(7);
	}

	pub fn clear_n(&mut self) {
		self.register.clear_bit(7);
	}

	pub fn is_v(&self) -> bool {
		self.register.is_bit_set(6)
	}

	pub fn set_v(&mut self) {
		self.register.set_bit(6);
	}

	pub fn clear_v(&mut self) {
		self.register.clear_bit(6);
	}

	// 5-bit is unused bit (but somehow set from BRK) and no name.
	// I named random name "a".

	pub fn is_a(&self) -> bool {
		self.register.is_bit_set(5)
	}

	pub fn set_a(&mut self) {
		self.register.set_bit(5);
	}

	pub fn clear_a(&mut self) {
		self.register.clear_bit(5);
	}

	pub fn is_b(&self) -> bool {
		self.register.is_bit_set(4)
	}

	pub fn set_b(&mut self) {
		self.register.set_bit(4);
	}

	pub fn clear_b(&mut self) {
		self.register.clear_bit(4);
	}

	pub fn is_d(&self) -> bool {
		self.register.is_bit_set(3)
	}

	pub fn set_d(&mut self) {
		self.register.set_bit(3);
	}

	pub fn clear_d(&mut self) {
		self.register.clear_bit(3);
	}

	pub fn is_i(&self) -> bool {
		self.register.is_bit_set(2)
	}

	pub fn set_i(&mut self) {
		self.register.set_bit(2);
	}

	pub fn clear_i(&mut self) {
		self.register.clear_bit(2);
	}

	pub fn is_z(&self) -> bool {
		self.register.is_bit_set(1)
	}

	pub fn set_z(&mut self) {
		self.register.set_bit(1);
	}

	pub fn clear_z(&mut self) {
		self.register.clear_bit(1);
	}

	pub fn is_c(&self) -> bool {
		self.register.is_bit_set(0)
	}

	pub fn set_c(&mut self) {
		self.register.set_bit(0);
	}

	pub fn clear_c(&mut self) {
		self.register.clear_bit(0);
	}

	fn dump(&self) -> String {
		self.register.dump() +
		&"(".to_owned() +
		match self.is_n() { true => &"N", false => &"-" }.to_owned() +
		match self.is_v() { true => &"V", false => &"-" }.to_owned() +
		match self.is_a() { true => &"A", false => &"-" }.to_owned() +
		match self.is_b() { true => &"B", false => &"-" }.to_owned() +
		match self.is_d() { true => &"D", false => &"-" }.to_owned() +
		match self.is_i() { true => &"I", false => &"-" }.to_owned() +
		match self.is_z() { true => &"Z", false => &"-" }.to_owned() +
		match self.is_c() { true => &"C", false => &"-" }.to_owned() +
		&")".to_owned()
	}
}
