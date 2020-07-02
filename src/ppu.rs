use register::Register;
use memory::Memory;
use rom::Rom;
use rom::Mirrorings;
use display::Display;

/**
 * RP2A03
 * The comments about PPU spec are based on https://wiki.nesdev.com/w/index.php/PPU
 */
pub struct Ppu {
	pub frame: u32,

	// 341 cycles per scan line. 0-340.
	pub cycle: u16,

	// 262 scan lines per frame. 0-261
	scanline: u16,

	// manage a case where vblank doesn't set due to 0x2002 read
	suppress_vblank: bool,

	// -- For background pixels

	//
	register_first_store: bool,

	//
	fine_x_scroll: u8,

	// 8-bit register and its latch for nametable
	name_table_latch: u8,
	name_table: Register<u8>,

	// Two 16-bit shift registers and their latches for two pattern table tiles
	pattern_table_low_latch: u8,
	pattern_table_high_latch: u8,
	pattern_table_low: Register<u16>,
	pattern_table_high: Register<u16>,

	// Two 16-bit shift registers and their latches for palette attributes.
	attribute_table_low_latch: u8,
	attribute_table_high_latch: u8,
	attribute_table_low: Register<u16>,
	attribute_table_high: Register<u16>,

	//
	current_vram_address: u16,
	temporal_vram_address: u16,

	// Internal VRAM read buffer used for VRAM read(ppudata load).
	vram_read_buffer: u8,

	vram: Memory,

	// -- For sprites pixels

	sprite_availables: [bool; 256],
	sprite_ids: [u8; 256],
	sprite_palette_addresses: [u16; 256],
	sprite_priorities: [u8; 256],

	// Primary OAM, holds 64 sprites for the frame
	primary_oam: SpritesManager,

	// Secondary OAM, holds 8 sprites for the current scanline
	secondary_oam: SpritesManager,

	// -- Eight + one CPU memory-mapped registers

	// 0x2000. See PpuControlRegister comment.
	ppuctrl: PpuControlRegister,

	// 0x2001. See PpuMaskRegister comment.
	ppumask: PpuMaskRegister,

	// 0x2002. See PpuStatusRegister comment.
	ppustatus: PpuStatusRegister,

	// 0x2003. OAM address 8-bit register.
	// Used for OAM access.
	// Write-only.
	oamaddr: Register<u8>,

	// 0x2004. OAM Data 8-bit register.
	// Used for data transfer from/to OAM[oamaddr].
	// Writes increments oamaddr after the write.
	oamdata: Register<u8>,

	// 0x2005. PPU scrolling position 8-bit register.
	// Used to change the scroll position.
	// Write-twice.
	ppuscroll: Register<u8>,

	// 0x2006. PPU address 8-bit register.
	// Used to set vram_address (0x0000-0x3FFF).
	// First store sets the higher byte while
	// second store sets the lower byte of the address.
	// Write-twice.
	ppuaddr: Register<u8>,

	// 0x2007. PPU data 8-bit register.
	// Used to transfer data from/to VRAM[vram_address].
	// The access increments vram_addr after the read/write.
	ppudata: Register<u8>,

	// 0x4014. OAM DMA 8-bit register.
	// This port is located on the CPU.
	// Writing 0xXX will upload 256 bytes of data from CPU page
	// 0xXX00-0xXXFF to the internal PPU OAM.
	// Write-only.
	oamdma: Register<u8>,

	// Internal data bus used for communication with CPU.
	// This bus behaves as an 8-bit dynamic latch due to large capacitance.
	// Witing to any PPU port(register) or reading any readable
	// PPU port will fill this latch. Reading write-only register returns
	// the latch's current value.
	// @TODO: Support decay. Decaying after a frame or so.
	data_bus: u8,

	// -- 

	display: Box<dyn Display>,

	pub nmi_interrupted: bool,
	pub irq_interrupted: bool
}

static PALETTES: [u32; 0x40] = [
    /* 0x00 */ 0xff757575,
    /* 0x01 */ 0xff8f1b27,
    /* 0x02 */ 0xffab0000,
    /* 0x03 */ 0xff9f0047,
    /* 0x04 */ 0xff77008f,
    /* 0x05 */ 0xff1300ab,
    /* 0x06 */ 0xff0000a7,
    /* 0x07 */ 0xff000b7f,
    /* 0x08 */ 0xff002f43,
    /* 0x09 */ 0xff004700,
    /* 0x0a */ 0xff005100,
    /* 0x0b */ 0xff173f00,
    /* 0x0c */ 0xff5f3f1b,
    /* 0x0d */ 0xff000000,
    /* 0x0e */ 0xff000000,
    /* 0x0f */ 0xff000000,
    /* 0x10 */ 0xffbcbcbc,
    /* 0x11 */ 0xffef7300,
    /* 0x12 */ 0xffef3b23,
    /* 0x13 */ 0xfff30083,
    /* 0x14 */ 0xffbf00bf,
    /* 0x15 */ 0xff5b00e7,
    /* 0x16 */ 0xff002bdb,
    /* 0x17 */ 0xff0f4fcb,
    /* 0x18 */ 0xff00738b,
    /* 0x19 */ 0xff009700,
    /* 0x1a */ 0xff00ab00,
    /* 0x1b */ 0xff3b9300,
    /* 0x1c */ 0xff8b8300,
    /* 0x1d */ 0xff000000,
    /* 0x1e */ 0xff000000,
    /* 0x1f */ 0xff000000,
    /* 0x20 */ 0xffffffff,
    /* 0x21 */ 0xffffbf3f,
    /* 0x22 */ 0xffff975f,
    /* 0x23 */ 0xfffd8ba7,
    /* 0x24 */ 0xffff7bf7,
    /* 0x25 */ 0xffb777ff,
    /* 0x26 */ 0xff6377ff,
    /* 0x27 */ 0xff3b9bff,
    /* 0x28 */ 0xff3fbff3,
    /* 0x29 */ 0xff13d383,
    /* 0x2a */ 0xff4bdf4f,
    /* 0x2b */ 0xff98f858,
    /* 0x2c */ 0xffdbeb00,
    /* 0x2d */ 0xff000000,
    /* 0x2e */ 0xff000000,
    /* 0x2f */ 0xff000000,
    /* 0x30 */ 0xffffffff,
    /* 0x31 */ 0xffffe7ab,
    /* 0x32 */ 0xffffd7c7,
    /* 0x33 */ 0xffffcbd7,
    /* 0x34 */ 0xffffc7ff,
    /* 0x35 */ 0xffdbc7ff,
    /* 0x36 */ 0xffb3bfff,
    /* 0x37 */ 0xffabdbff,
    /* 0x38 */ 0xffa3e7ff,
    /* 0x39 */ 0xffa3ffe3,
    /* 0x3a */ 0xffbff3ab,
    /* 0x3b */ 0xffcfffb3,
    /* 0x3c */ 0xfff3ff9f,
    /* 0x3d */ 0xff000000,
    /* 0x3e */ 0xff000000,
    /* 0x3f */ 0xff000000
];

impl Ppu {
	pub fn new(display: Box<dyn Display>) -> Self {
		Ppu {
			frame: 0,
			cycle: 0,
			scanline: 0,
			suppress_vblank: false,
			fine_x_scroll: 0,
			current_vram_address: 0,
			temporal_vram_address: 0,
			vram: Memory::new(vec![0; 16 * 1024]), // 16KB
			data_bus: 0,
			name_table_latch: 0,
			attribute_table_low_latch: 0,
			attribute_table_high_latch: 0,
			pattern_table_low_latch: 0,
			pattern_table_high_latch: 0,
			register_first_store: true,
			sprite_availables: [false; 256],
			sprite_ids: [0; 256],
			sprite_palette_addresses: [0; 256],
			sprite_priorities: [0; 256],
			oamaddr: Register::<u8>::new(),
			oamdata: Register::<u8>::new(),
			oamdma: Register::<u8>::new(),
			primary_oam: SpritesManager::new(Memory::new(vec![0; 256])), // primary 256B
			secondary_oam: SpritesManager::new(Memory::new(vec![0; 32])), // secondary 32B
			vram_read_buffer: 0,
			ppuaddr: Register::<u8>::new(),
			ppudata: Register::<u8>::new(),
			ppuctrl: PpuControlRegister::new(),
			ppumask: PpuMaskRegister::new(),
			ppustatus: PpuStatusRegister::new(),
			ppuscroll: Register::<u8>::new(),
			name_table: Register::<u8>::new(),
			attribute_table_low: Register::<u16>::new(),
			attribute_table_high: Register::<u16>::new(),
			pattern_table_low: Register::<u16>::new(),
			pattern_table_high: Register::<u16>::new(),
			display: display,
			nmi_interrupted: false,
			irq_interrupted: false
		}
	}

	pub fn bootup(&mut self) {
		self.ppustatus.store(0x80);
	}

	pub fn reset(&mut self) {
		self.ppuctrl.store(0x00);
		self.ppumask.store(0x00);
		self.ppuscroll.store(0x00);
		self.ppudata.store(0x00);
		self.register_first_store = true;
		self.frame = 0;
		// not sure if I should really reset scanline and cycle
		// but I do for now
		self.scanline = 0;
		self.cycle = 0;
	}

	pub fn step(&mut self, rom: &mut Rom) {
		self.render_pixel(rom);
		self.shift_registers();
		self.fetch(rom);
		self.evaluate_sprites(rom);
		self.update_flags(rom);
		self.countup_scroll_counters();
		self.countup_cycle();
	}

	pub fn load_register(&mut self, address: u16, rom: &Rom) -> u8 {
		match address {
			// ppustatus load
			0x2002 => {
				let value = self.ppustatus.load();

				// clear vblank after reading 0x2002
				self.ppustatus.clear_vblank();

				self.register_first_store = true;

				// unused 4 lsb bits don't override data bus.
				self.data_bus = (value & 0xE0) | (self.data_bus & 0x1F);

				// reading 0x2002 at cycle=0 and scanline=241
				// won't set vblank flag (7-bit) or fire NMI.
				// reading 0x2002 at cycle=1or2 and scanline=241
				// returns the data as vblank flag is set,
				// clears the flag, and won't fire NMI

				// Note: update_flags() which can set vblank is called
				// after this method in the same cycle, so set supress_vblank true
				// even at cycle=1 not only cycle=0

				if self.scanline == 241 && (self.cycle == 0 || self.cycle == 1) {
					self.suppress_vblank = true;
				}

				value | match self.scanline == 241 && (self.cycle == 1 || self.cycle == 2) {
					true => 0x80,
					false => 0x00
				}
			},
			// oamdata load
			0x2004 => {
				let value = self.primary_oam.load(self.oamaddr.load());
				self.data_bus = value;
				value
			},
			// ppudata load
			0x2007 => {
				// Reading ppudata updates the VRAM read buffer with VRAM[vram_address]
				// and returns the internal VRAM read buffer.
				// They work differently depending on the reading address.
				// 0x0000-0x3EFF: Update the buffer after returning the content of the buffer
				// 0x3F00-0x3FFF: Immediately update the buffer before returning the content of the buffer
				let value = self.load(self.current_vram_address, rom);
				let return_value = match self.current_vram_address {
					0..=0x3EFF => self.vram_read_buffer,
					_ => value
				};
				self.vram_read_buffer = value;

				// @TODO: Support greyscale if needed

				// Accessing ppudata increments vram_address
				self.increment_vram_address();
				self.data_bus = return_value;
				self.data_bus
			},
			_ => self.data_bus
		}
	}

	pub fn store_register(&mut self, address: u16, value: u8, rom: &mut Rom) {
		// Writing to any PPU port(register) from CPU fills the latch (data_bus).
		self.data_bus = value;

		match address {
			// ppuctrl store
			0x2000 => {
				// Ignore the write to this register
				// for about 30k cycles after power/reset.
				// But I found some test roms writes to 0x2000
				// right after power on so commenting out so far.

				//if self.frame == 0 && self.scanline <= 88 {
				//	return;
				//}

				let previous_nmi_enabled = self.ppuctrl.is_nmi_enabled();
				self.ppuctrl.store(value);

				// Immediately generate an NMI if the PPU is currently
				// in vertical blank, PPUSTATUS vblank flag is still set,
				// and changing the NMI flag from 0 to 1
				if self.ppustatus.is_vblank() &&
					!previous_nmi_enabled &&
					self.ppuctrl.is_nmi_enabled() {
					self.nmi_interrupted = true;
				}

				// Copy the 1-0 bits of value to 11-10 bits of temporal vram_address for scrolling
				// Refer to http://wiki.nesdev.com/w/index.php/PPU_scrolling
				self.temporal_vram_address &= 0xF3FF;
				self.temporal_vram_address |= ((value as u16) & 0x3) << 10;
			},
			// ppumask store
			0x2001 => {
				self.ppumask.store(value);
			},
			// oamaddr store
			0x2003 => {
				self.oamaddr.store(value);
			},
			// oamdata store
			0x2004 => {
				self.oamdata.store(value);
				self.primary_oam.store(self.oamaddr.load(), value);
				self.oamaddr.increment();
			},
			// ppuscroll store
			0x2005 => {
				self.ppuscroll.store(value);

				if self.register_first_store {
					// Copy 2-0 bits of the value to fine_x_scroll and
					// 7-3 bits of the value to 4-0 bits of the temporal vram_address for scrolling
					// Refer to http://wiki.nesdev.com/w/index.php/PPU_scrolling
					self.fine_x_scroll = value & 0x7;
					self.temporal_vram_address &= 0xFFE0;
					self.temporal_vram_address |= ((value as u16) >> 3) & 0x1F;
				} else {
					// Copy 2-0 bits of the value to 14-12 bits of the temporal vram_address and
					// 7-3 bits of the value to 9-5 bits of the temporal vram_address for scrolling
					// Refer to http://wiki.nesdev.com/w/index.php/PPU_scrolling
					self.temporal_vram_address &= 0x8C1F;
					self.temporal_vram_address |= ((value as u16) & 0xF8) << 2;
					self.temporal_vram_address |= ((value as u16) & 0x7) << 12;
				}

				self.register_first_store = !self.register_first_store;
			},
			// ppuaddr store
			0x2006 => {
				// First store sets the higher byte of the vram_address.
				// Second store sets the lower byte of the vram_address.
				// Refer to http://wiki.nesdev.com/w/index.php/PPU_scrolling
				if self.register_first_store {
					self.temporal_vram_address &= 0x00FF;
					self.temporal_vram_address |= ((value as u16) & 0x3F) << 8;
				} else {
					self.ppuaddr.store(value);
					self.temporal_vram_address &= 0xFF00;
					self.temporal_vram_address |= value as u16;
					self.current_vram_address = self.temporal_vram_address;
				}

				self.register_first_store = !self.register_first_store;
			},
			// ppudata store
			0x2007 => {
				self.ppudata.store(value);
				self.store(self.current_vram_address, value, rom);
				// Accessing ppudata increments vram_address
				self.increment_vram_address();
			},
			// oamdma store
			0x4014 => {
				self.oamdma.store(value);
				// DMA is processed in Cpu
			},
			_ => {}
		}
	}

	fn load(&self, mut address: u16, rom: &Rom) -> u8 {
		address = address & 0x3FFF;  // just in case

		// 0x0000 - 0x1FFF is mapped with cartridge's CHR-ROM if exists.
		// Otherwise load from VRAM.

		match address < 0x2000 && rom.has_chr_rom() {
			true => rom.load(address as u32),
			false => self.vram.load(self.convert_vram_address(address, rom) as u32)
		}
	}

	fn store(&mut self, mut address: u16, value: u8, rom: &mut Rom) {
		address = address & 0x3FFF;  // just in case

		// 0x0000 - 0x1FFF is mapped with cartridge's CHR-ROM if exists.
		// Otherwise store to VRAM.

		match address < 0x2000 && rom.has_chr_rom() {
			true => rom.store(address as u32, value),
			false => self.vram.store(self.convert_vram_address(address, rom) as u32, value)
		};
	}

	fn convert_vram_address(&self, address: u16, rom: &Rom) -> u16 {
		// 0x0000 - 0x0FFF: pattern table 0
		// 0x1000 - 0x1FFF: pattern table 1
		// 0x2000 - 0x23FF: nametable 0
		// 0x2400 - 0x27FF: nametable 1
		// 0x2800 - 0x2BFF: nametable 2
		// 0x2C00 - 0x2FFF: nametable 3
		// 0x3000 - 0x3EFF: Mirrors of 0x2000 - 0x2EFF
		// 0x3F00 - 0x3F1F: Palette RAM indices
		// 0x3F20 - 0x3FFF: Mirrors of 0x3F00 - 0x3F1F

		match address {
			0..=0x1FFF => address,
			0x2000..=0x3EFF => self.get_name_table_address_with_mirroring(address & 0x2FFF, rom),
			_ /* 0x3F00..=0x3FFF */ => {
				// Addresses for palette
				// 0x3F10/0x3F14/0x3F18/0x3F1C are mirrors of
				// 0x3F00/0x3F04/0x3F08/0x3F0C.
				match address {
					0x3F10 => 0x3F00,
					0x3F14 => 0x3F04,
					0x3F18 => 0x3F08,
					0x3F1C => 0x3F0C,
					_ => address
				}
			}
		}
	}

	fn render_pixel(&mut self, rom: &Rom) {
		// Note: this comparison order is for performance.
		if self.cycle >= 257 || self.scanline >= 240 || self.cycle == 0 {
			return;
		}

		// guaranteed that cycle is equal to or greater than 1 here, see the above
		let x = (self.cycle - 1) % 256; // @TODO: Somehow -2 generates pixel at right position, why?
		let y = self.scanline;

		let background_visible = self.ppumask.is_background_visible() &&
			(self.ppumask.is_left_most_background_visible() || x >= 8);
		let sprites_visible = self.ppumask.is_sprites_visible() &&
			(self.ppumask.is_left_most_sprites_visible() || x >= 8);
		let background_palette_address = self.get_background_palette_address();
		let sprite_available = self.sprite_availables[x as usize];
		let sprite_palette_address = self.sprite_palette_addresses[x as usize];
		let sprite_id = self.sprite_ids[x as usize];
		let sprite_priority = self.sprite_priorities[x as usize];

		let is_background_pixel_zero = !background_visible || (background_palette_address & 0x3) == 0;
		let is_sprite_pixel_zero = !sprites_visible || !sprite_available || (sprite_palette_address & 0x3) == 0;

		// Select output color
		// |  bg | sprite | pri |        out |
		// | --- | ------ | --- | ---------- |
		// |   0 |      0 |   - | bg(0x3F00) |
		// |   0 |    1-3 |   - |     sprite |
		// | 1-3 |      0 |   - |         bg |
		// | 1-3 |    1-3 |   0 |     sprite |
		// | 1-3 |    1-3 |   1 |         bg |

		let palette_address = match is_background_pixel_zero {
			true => match is_sprite_pixel_zero {
				true => 0x3F00, // universal_background_palette_address
				false => sprite_palette_address
			},
			false => match is_sprite_pixel_zero {
				true => background_palette_address,
				false => match sprite_priority == 0 {
					true => sprite_palette_address,
					false => background_palette_address
				}
			}
		};

		let c = self.get_emphasis_color(self.load_palette(self.load(palette_address, rom)));

		// Sprite zero hit test.
		// Set zero hit flag when a nonzero pixel of sprite 0 overlaps
		// a nonzero background pixel.
		// Hit doesn't trigger in any area where the background or sprites are hidden.
		// Sprite priority doesn't have effect to zero hit.
		if sprite_id == 0 &&
			!is_sprite_pixel_zero &&
			!is_background_pixel_zero {
			self.ppustatus.set_zero_hit();
		}

		self.display.render_pixel(x, y, c);
	}

	fn get_background_palette_address(&self) -> u16 {
		// fine_x_scroll selects 16-bit shifts register.
		let pos = 15 - (self.fine_x_scroll & 0xF);

		let offset = (self.attribute_table_high.load_bit(pos) << 3) |
			(self.attribute_table_low.load_bit(pos) << 2) |
			(self.pattern_table_high.load_bit(pos) << 1) |
			self.pattern_table_low.load_bit(pos);

		// background palette indices are in 0x3F00-0x3F0F
		0x3F00 + offset as u16
	}

	fn shift_registers(&mut self) {
		if self.scanline >= 240 && self.scanline <= 260 {
			return;
		}

		if (self.cycle >= 1 && self.cycle <= 256) ||
			(self.cycle >= 329 && self.cycle <= 336) {
			self.pattern_table_low.shift(0);
			self.pattern_table_high.shift(0);
			self.attribute_table_low.shift(0);
			self.attribute_table_high.shift(0);
		}
	}

	fn fetch(&mut self, rom: &Rom) {
		// No fetch during post-rendering scanline 240 and vblank interval 241-260
		if self.scanline >= 240 && self.scanline <= 260 {
			return;
		}

		// In visible scanlines 0-239
		// Cycle 0:
		//   Idle
		// Cycle 1-256:
		//   The data for each tile is fetched during this phase.
		//   Each memory access takes 2 cycles to complete,
		//   and 4 must be performed per tile
		//     - Nametable byte
		//     - Attribute table byte
		//     - Pattern table tile low
		//     - Pattern table tile high
		// Cycle 257-320: @TODO
		// Cycle 321-336: @TODO
		// Cycle 337-340: @TODO

		if self.cycle == 0 {
			return;
		}

		if (self.cycle >= 257 && self.cycle <= 320) || self.cycle >= 337 {
			return;
		}

		// Every 8 cycles, the data for the next tile is loaded into the upper 8 bits
		// of this shift register. Meanwhile, the pixel to render is fetched from one
		// of the lower 8 bits.

		// These registers are fed by a latch which contains the palette attribute
		// for the next tile. Every 8 cycles, the latch is loaded with the palette
		// attribute for the next tile.

		// In each 8 cycles,
		// 0-1: @TODO
		// 2-3: @TODO
		// 4-5: @TODO
		// 6-7: @TODO

		// self.cycle is equal to or greater than 1 here, see the above.

		match (self.cycle - 1) % 8 {
			0 => {
				self.fetch_name_table(rom);
				self.name_table.store(self.name_table_latch);
				self.attribute_table_low.store_lower_byte(self.attribute_table_low_latch);
				self.attribute_table_high.store_lower_byte(self.attribute_table_high_latch);
				self.pattern_table_low.store_lower_byte(self.pattern_table_low_latch);
				self.pattern_table_high.store_lower_byte(self.pattern_table_high_latch);
			},
			2 => self.fetch_attribute_table(rom),
			4 => self.fetch_pattern_table_low(rom),
			6 => self.fetch_pattern_table_high(rom),
			_ => {}
		};
	}

	fn fetch_name_table(&mut self, rom: &Rom) {
		// A nametable is a 1024 byte area of memory used by the PPU to lay out backgrounds.
		// Each byte in the nametable controls one 8x8 pixel character cell, and each nametable
		// has 30 rows of 32 tiles each, for 960 (0x3C0) bytes; the rest is used by each nametable's
		// attribute table. With each tile being 8x8 pixels, this makes a total of 256x240 pixels
		// in one map, the same size as one full screen.

		// Name table entries are in 0x2000-0x2FFF.
		// Four name tables and 0x400 bytes per name table.
		// The last 64 bytes of each include attribute table.
		// Nametable 0: 0x2000-0x23FF (attribute table 0x23C0-0x23FF)
		// Nametable 1: 0x2400-0x27FF (attribute table 0x27C0-0x27FF)
		// Nametable 2: 0x2800-0x2BFF (attribute table 0x2BC0-0x2BFF)
		// Nametable 3: 0x2C00-0x2FFF (attribute table 0x2FC0-0x2FFF)

		// Here fetches a tile of a nametable.

		// address is from http://wiki.nesdev.com/w/index.php/PPU_scrolling
		self.name_table_latch = self.load(0x2000 | (self.current_vram_address & 0x0FFF), rom);
	}

	fn fetch_attribute_table(&mut self, rom: &Rom) {
		// @TODO: Implement properly

		// The attribute table is a 64-byte array at the end of each nametable
		// that controls which palette is assigned to each part of the background.
		// Each attribute table, starting at 0x23C0, 0x27C0, 0x2BC0, or 0x2FC0,
		// is arranged as an 8x8 byte array.

		// vram_address
		// 13-12: fine y scroll
		// 11-10: nametable select
		// 9-5: coarse y scroll
		// 4-0: coarse x scroll
		let v = self.current_vram_address;
		let coarse_y = (v >> 5) & 0x1F;
		let coarse_x = v & 0x1F;

		// attribute address the lower 12-bits
		// 11-10: name table select
		// 9-6: attribute offset (960 bytes)
		// 5-3: high 3bits of coarse y
		// 2-0: high 3bits of coarse x
		// From http://wiki.nesdev.com/w/index.php/PPU_scrolling
		let byte = self.load(0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07), rom);

		// byte includes four two bits
		// 7-6: bottom right
		// 5-4: bottom left
		// 3-2: top right
		// 1-0: top left

		// @TODO: Optimize pos calculation with bit wise operation?
		let is_bottom = (coarse_y & 0x3) >= 2;
		let is_right = (coarse_x & 0x3) >= 2;
		let pos = match is_bottom {
			true => {
				match is_right {
					true => 6, // bottomright
					false => 4 // bottomleft
				}
			},
			false => {
				match is_right {
					true => 2, // topright
					false => 0 // topleft
				}
			}
		};

		let value = (byte >> pos) & 0x3;

		self.attribute_table_high_latch = match value & 2 {
			2 => 0xff,
			_ => 0
		};

		self.attribute_table_low_latch = match value & 1 {
			1 => 0xff,
			_ => 0
		};
	}

	fn fetch_pattern_table_low(&mut self, rom: &Rom) {
		let fine_scroll_y = (self.current_vram_address >> 12) & 0x7;
		let index = self.ppuctrl.background_pattern_table_base_address() +
			((self.name_table.load() as u16) << 4) + fine_scroll_y;
		self.pattern_table_low_latch = self.load(index, rom);
	}

	fn fetch_pattern_table_high(&mut self, rom: &Rom) {
		let fine_scroll_y = (self.current_vram_address >> 12) & 0x7;
		let index = self.ppuctrl.background_pattern_table_base_address() +
			((self.name_table.load() as u16) << 4) + fine_scroll_y;
		self.pattern_table_high_latch = self.load(index + 0x8, rom);
	}

	fn update_flags(&mut self, rom: &mut Rom) {
		if self.cycle == 1 {
			if self.scanline == 241 {
				// set vblank and occur NMI at cycle 1 in scanline 241
				if !self.suppress_vblank {
					self.ppustatus.set_vblank();
				}
				self.suppress_vblank = false;
				// Pixels for this frame should be ready so update the display
				self.display.vblank();
			} else if self.scanline == 261 {
				// clear vblank, sprite zero hit flag,
				// and sprite overflow flags at cycle 1 in pre-render line 261
				self.ppustatus.clear_vblank();
				self.ppustatus.clear_zero_hit();
				self.ppustatus.clear_overflow();
			}
		}

		// According to http://wiki.nesdev.com/w/index.php/PPU_frame_timing#VBL_Flag_Timing
		// reading 0x2002 at cycle=2 and scanline=241 can suppress NMI
		// so firing NMI at some cycles away not at cycle=1 so far

		// There is a chance that CPU 0x2002 read gets the data vblank flag set
		// before CPU starts NMI interrupt routine.
		// CPU instructions take multiple CPU clocks to complete.
		// If CPU starts an operation of an istruction including 0x2002 read right before
		// PPU sets vblank flag and fires NMI,
		// the 0x2002 read gets the data with vblank flag set even before
		// CPU starts NMI routine.
		//
		//    CPU                              PPU
		// 1. instruction operation start
		// 2.   - doing something              vblank start and fire NMI
		// 3.   - read 0x2002 with
		//        vblank flag set
		// 4.   - doing something
		// 5. Notice NMI and start
		//    NMI routine
		//
		// It seems some games rely on this behavior.
		// To simulate this behavior we fire NMI at cycle=20 so far.
		// If CPU reads 0x2002 between PPU cycle 3~20 it gets data
		// vblank flag set before NMI routine.
		// (reading at cycle 1~2 suppresses NMI, see load_register())
		// @TODO: Safer and more appropriate approach.

		if self.cycle == 20 && self.scanline == 241 {
			if self.ppustatus.is_vblank() &&
				self.ppuctrl.is_nmi_enabled() {
				self.nmi_interrupted = true;
			}
		}

		// @TODO: check this driving IRQ counter for MMC3Mapper timing is correct
		// @TODO: This is MMC3Mapper specific. Should this be here?

		if self.cycle == 340 && self.scanline <= 240 &&
			self.ppumask.is_background_visible() &&
			rom.irq_interrupted() {
				self.irq_interrupted = true
		}
	}

	fn countup_scroll_counters(&mut self) {
		if !self.ppumask.is_background_visible() && !self.ppumask.is_sprites_visible() {
			return;
		}

		if self.scanline >= 240 && self.scanline <= 260 {
			return;
		}

		if self.scanline == 261 {
			if self.cycle >= 280 && self.cycle <= 304 {
				self.current_vram_address &= !0x7BE0;
				self.current_vram_address |= self.temporal_vram_address & 0x7BE0;
			}
		}

		if self.cycle == 0 || (self.cycle >= 258 && self.cycle <= 320) {
			return;
		}

		if (self.cycle % 8) == 0 {
			let mut v = self.current_vram_address;

			// this is from http://wiki.nesdev.com/w/index.php/PPU_scrolling
			if (v & 0x1F) == 31 {
				v &= !0x1F;
				v ^= 0x400;
			} else {
				v += 1;
			}

			self.current_vram_address = v;
		}

		if self.cycle == 256 {
			// Increments the vertical position of vram_address
			// @TODO: Only if rendering is enabled?
			let mut v = self.current_vram_address;

			// From http://wiki.nesdev.com/w/index.php/PPU_scrolling
			if (v & 0x7000) != 0x7000 {
				v += 0x1000;
			} else {
				v &= !0x7000;
				let mut y = (v & 0x3E0) >> 5;

				if y == 29 {
					y = 0;
					v ^= 0x800;
				} else if y == 31 {
					y = 0;
				} else {
					y += 1;
				}

				v = (v & !0x3E0) | (y << 5);
			}

			self.current_vram_address = v;
		} else if self.cycle == 257 {
			// Copies all bits related to horizontal position from
			// temporal to current vram_address
			// @TODO: Only if rendering is enabled?
			// From http://wiki.nesdev.com/w/index.php/PPU_scrolling
			self.current_vram_address &= !0x41F;
			self.current_vram_address |= self.temporal_vram_address & 0x41F;
		}
	}

	fn countup_cycle(&mut self) {
		// cycle:    0 - 340
		// scanline: 0 - 261
		self.cycle += 1;
		if self.cycle > 340 {
			self.cycle = 0;
			self.scanline += 1;

			if self.scanline > 261 {
				self.scanline = 0;
				self.frame += 1;
			}
		}
	}

	//

	fn increment_vram_address(&mut self) {
		// Increments vram address based on ppuctrl, 1 or 32
		self.current_vram_address += self.ppuctrl.increment_address_size() as u16;
		self.current_vram_address &= 0x7FFF;
		self.ppuaddr.store(self.current_vram_address as u8 & 0xFF);
	}

	fn evaluate_sprites(&mut self, rom: &Rom) {
		// oamaddr is set to 0 during cycle 257-320 of the pre-render and visible scanlines.
		// @TODO: Optimize
		if (self.scanline < 240 || self.scanline == 261) &&
			self.cycle >= 257 && self.cycle <= 320 {
			self.oamaddr.store(0);
		}

		// During all visible scanlines(0-239),
		// the PPU scans through OAM to determine which sprites
		// to render on the next scanline

		if self.scanline >= 240 {
			return;
		}

		// Cycles
		// 1-64: Secondary OAM is initialized to 0xff.
		// 65-256: Sprite evaluation
		// 257-320: Sprite fetches
		// 321-340+0: Background render pipeline initialization
		if self.cycle == 1 {
			// Initialize at a time at cycle 1 due to performance
			// and simplicity so far
			self.secondary_oam.reset();
		} else if self.cycle == 257 {
			// Evaluate at a time at cycle 257 due to performance
			// and simplicity so far
			self.process_sprite_pixels(rom);
		}
	}

	fn process_sprite_pixels(&mut self, rom: &Rom) {
		for i in 0..self.sprite_availables.len() {
			self.sprite_availables[i] = false;
		}

		let y = self.scanline as u8;
		let height = self.ppuctrl.sprite_height();
		let mut n = 0;

		// Find up to eight sprite on this scan line from primary OAM and
		// copy them to secondary OAM.
		// And process all bits of a scanline for sprites here now
		// for the performance and simplicity.
		for i in 0..64 {
			let s = self.primary_oam.get(i);
			if s.on(y, height) {
				if n >= 8 {
					// Set sprite overflow flag if
					// more than eight sprites appear on a scanline
					self.ppustatus.set_overflow();
					break;
				}
				let base_x = s.get_x();
				let y_in_sprite = s.get_y_in_sprite(y, height);
				let msb = s.get_palette_num() as u16;
				for j in 0..8 {
					//
					if base_x as u16 + j as u16 >= 256 {
						break;
					}
					let x = base_x + j;
					// No override with later sprites
					if self.sprite_availables[x as usize] {
						continue;
					}
					let x_in_sprite = match s.horizontal_flip() {
						true => 7 - j,
						false => j
					};
					// pattern table holds the lowest two bits of palette memory address
					let lsb = self.get_pattern_table_element_for_sprite(&s, x_in_sprite, y_in_sprite, height, rom) as u16;
					// the lowest two 0 bits means transparent (=no sprite pixel)
					if lsb != 0 {
						self.sprite_availables[x as usize] = true;
						// Sprite palette indices are in 0x3F10-0x3F1F
						self.sprite_palette_addresses[x as usize] = 0x3F10 | (msb << 2) | lsb;
						self.sprite_ids[x as usize] = i;
						self.sprite_priorities[x as usize] = s.get_priority();
					}
				}
				self.secondary_oam.copy(n, s);
				n += 1;
			}
		}
	}

	fn get_name_table_address_with_mirroring(&self, address: u16, rom: &Rom) -> u16 {
		let name_table_address = address & 0x2C00;
		(address & 0x3FF) | match rom.mirroring_type() {
			Mirrorings::SingleScreen => 0x2000,
			Mirrorings::Horizontal => match name_table_address {
				0x2000 => 0x2000,
				0x2400 => 0x2000,
				0x2800 => 0x2800,
				_ /* 0x2C00 */ => 0x2800
			},
			Mirrorings::Vertical => match name_table_address {
				0x2000 => 0x2000,
				0x2400 => 0x2400,
				0x2800 => 0x2000,
				_ /* 0x2C00 */ => 0x2400
			},
			Mirrorings::FourScreen => name_table_address
		}
	}

	fn get_pattern_table_element_for_sprite(&self, s: &Sprite, x_in_sprite: u8, y_in_sprite: u8, height: u8, rom: &Rom) -> u8 {
		// Get an element from pattern table consisting of the lowest two bits
		// of palette memory address for sprites

		// 8x8 sprite and 8x16 sprite calculates tile address differently
		let address = match height == 8 {
			true => {
				// 8x8 sprite
				// ppuctrl selects base address 0x0000 or 0x1000
				let base_address = self.ppuctrl.sprite_pattern_table_base_address();
				// Each tile has 16bytes
				let byte_offset = s.get_tile_index() as u16 * 0x10;
				// A tile has 8x2 rows
				let row = y_in_sprite as u16;
				base_address + byte_offset + row
			},
			false => {
				// 8x16 sprite
				// Ignore base address selected by ppuctrl but
				// 0-bit of sprite tile_index selects 0x0000 or 0x1000 
				let tile_index = s.get_tile_index() as u16;
				let base_address = (tile_index & 1) * 0x1000;
				// 7-1 bits of tile_index maps to 0-254 tiles
				let byte_offset = (tile_index & 0xFE) * 0x10;
				// Eatch 16-byte tile has 8x8 pixels then bottom half pixel needs to see next tile
				let row = ((y_in_sprite % 8) + ((y_in_sprite & 0x8) << 1)) as u16;
				base_address + byte_offset + row
			}
		};

		// Each tile has 16bytes (8x2 rows)
		// The first 8bytes in a tile are for 0-bit,
		// while the second 8bytes are for 1-bit of palette memory address
		let lower_bits = self.load(address, rom);
		let higher_bits = self.load(address + 8, rom);
		let pos = 7 - x_in_sprite; // xxx_bits[7:0] corresponds to x_in_sprite[0:7] 
		(((higher_bits >> pos) & 1) << 1) | ((lower_bits >> pos) & 1)
	}

	fn load_palette(&self, address: u8) -> u32 {
		// In greyscale mode, mask the palette index with 0x30 and
		// read from the grey column 0x00, 0x10, 0x20, or 0x30
		let mask = match self.ppumask.is_greyscale() {
			true => 0x30,
			false => 0xFF
		};
		PALETTES[(address & mask) as usize] & 0xFFFFFF
	}

	fn get_emphasis_color(&self, mut c: u32) -> u32 {
		// Color emphasis bases on ppumask
		// @TODO: Implement properly
		if self.ppumask.is_emphasis_red() {
			c = c | 0x00FF0000;
		}
		if self.ppumask.is_emphasis_green() {
			c = c | 0x0000FF00;
		}
		if self.ppumask.is_emphasis_blue() {
			c = c | 0x000000FF;
		}
		c
	}

	pub fn get_display(&self) -> &Box<dyn Display> {
		&self.display
	}
}

// PPU control 8-bit register.
// CPU memory-mapped at 0x2000
// Write-only

pub struct PpuControlRegister {
	register: Register<u8>
}

impl PpuControlRegister {
	fn new() -> Self {
		PpuControlRegister {
			register: Register::<u8>::new()
		}
	}

	fn _load(&self) -> u8 {
		self.register.load()
	}

	fn store(&mut self, value: u8) {
		self.register.store(value);
	}

	// Bit 7. Flag indicating whether generating NMI
	// at the start of the vertical blanking interval
	// -- 0: disabled, 1: enabled
	fn is_nmi_enabled(&self) -> bool {
		self.register.is_bit_set(7)
	}

	// Bit 6. PPU master/slave select
	// @TODO: Implement

	// Bit 5. Sprite height
	// -- 0: 8 (8x8 pixels), 1: 16 (8x16 pixels)
	fn sprite_height(&self) -> u8 {
		match self.register.is_bit_set(5) {
			false => 8,
			true => 16
		}
	}

	// Bit 4. Background pattern table address
	// -- 0: 0x0000, 1: 0x1000
	fn background_pattern_table_base_address(&self) -> u16 {
		match self.register.is_bit_set(4) {
			false => 0,
			true => 0x1000
		}
	}

	// Bit 3. Sprite pattern table address for 8x8 sprites
	// -- 0: 0x0000, 1: 0x1000
	fn sprite_pattern_table_base_address(&self) -> u16 {
		match self.register.is_bit_set(3) {
			false => 0,
			true => 0x1000
		}
	}

	// Bit 2. VRAM address increment per CPU read/write of PPUDATA
	// -- 0: 1, 1: 32
	fn increment_address_size(&self) -> u8 {
		match self.register.is_bit_set(2) {
			false => 1,
			true => 32
		}
	}

	// Bit 0-1. Base nametable address
	// -- 0: 0x2000, 1: 0x2400, 2: 0x2800, 3: 0x2C00
	fn _base_name_table_address(&self) -> u16 {
		match self.register.load_bits(0, 2) {
			0 => 0x2000,
			1 => 0x2400,
			2 => 0x2800,
			_ => 0x2C00
		}
	}
}

// PPU mask 8-bit register
// CPU memory-mapped at 0x2001
// Write-only
pub struct PpuMaskRegister {
	register: Register<u8>
}

impl PpuMaskRegister {
	fn new() -> Self {
		PpuMaskRegister {
			register: Register::<u8>::new()
		}
	}

	fn _load(&self) -> u8 {
		self.register.load()
	}

	fn store(&mut self, value: u8) {
		self.register.store(value);
	}

	// Bit 7. Emphasizes blue
	fn is_emphasis_blue(&self) -> bool {
		self.register.is_bit_set(7)
	}

	// Bit 6. Emphasizes green on the NTSC, while red on the PAL
	fn is_emphasis_green(&self) -> bool {
		self.register.is_bit_set(6)
	}

	// Bit 5. Emphasizes ref on the NTSC, while green on the PAL
	fn is_emphasis_red(&self) -> bool {
		self.register.is_bit_set(5)
	}

	// Bit 4. Show sprites.
	// -- 0: invisible, 1: visible
	fn is_sprites_visible(&self) -> bool {
		self.register.is_bit_set(4)
	}

	// Bit 3. Show background.
	// -- 0: invisible, 1: visible
	fn is_background_visible(&self) -> bool {
		self.register.is_bit_set(3)
	}

	// Bit 2. Show sprites in leftmost 8 pixels of screen.
	// -- 0: invisible, 1: visible
	fn is_left_most_sprites_visible(&self) -> bool {
		self.register.is_bit_set(2)
	}

	// Bit 1. Show background in leftmost 8 pixels of screen.
	// -- 0: invisible, 1: visible
	fn is_left_most_background_visible(&self) -> bool {
		self.register.is_bit_set(1)
	}

	// Bit 0. Greyscale
	// -- 0: normal color, 1: produce a greyscale display
	fn is_greyscale(&self) -> bool {
		self.register.is_bit_set(0)
	}
}

// PPU status 8-bit register
// CPU memory-mapped at 0x2002
// Read-only
pub struct PpuStatusRegister {
	register: Register<u8>
}

impl PpuStatusRegister {
	fn new() -> Self {
		PpuStatusRegister {
			register: Register::<u8>::new()
		}
	}

	fn load(&self) -> u8 {
		self.register.load()
	}

	fn store(&mut self, value: u8) {
		// @TOOD: Whether should we update unused 0-5 bits?
		self.register.store(value);
	}

	// Bit 7. Vertical blank.
	// Set at dot 1 of line 241, cleared after reading 0x2002
	// or dot 1 of the pre-render line 261
	// -- 0: not in vblank, 1: in vblank
	fn set_vblank(&mut self) {
		self.register.set_bit(7);
	}

	fn clear_vblank(&mut self) {
		self.register.clear_bit(7);
	}

	fn is_vblank(&mut self) -> bool {
		self.register.is_bit_set(7)
	}

	// Bit 6. Sprite zero hit.
	// Set when a nonzero pixel of sprite 0 overlaps a nonzero background pixel.
	// Cleared at dot 1 of the pre-render line. Used for raster timing.
	fn set_zero_hit(&mut self) {
		self.register.set_bit(6);
	}

	fn clear_zero_hit(&mut self) {
		self.register.clear_bit(6);
	}

	// Bit 5. Sprite overflow.
	// Set more than eight sprites appear on a scanline.
        // Cleared at dot 1 of the pre-render line 261.
	fn set_overflow(&mut self) {
		self.register.set_bit(5);
	}

	fn clear_overflow(&mut self) {
		self.register.clear_bit(5);
	}
}

pub struct SpritesManager {
	memory: Memory
}

impl SpritesManager {
	fn new(memory: Memory) -> Self {
		SpritesManager {
			memory: memory
		}
	}

	fn load(&self, address: u8) -> u8 {
		self.memory.load(address as u32)
	}

	fn store(&mut self, address: u8, value: u8) {
		self.memory.store(address as u32, value);
	}

	fn get_num(&self) -> u8 {
		(self.memory.capacity() / 4) as u8
	}

	fn get(&self, index: u8) -> Sprite {
		Sprite {
			byte0: self.load(index * 4 + 0),
			byte1: self.load(index * 4 + 1),
			byte2: self.load(index * 4 + 2),
			byte3: self.load(index * 4 + 3)
		}
	}

	fn copy(&mut self, index: u8, sprite: Sprite) {
		self.store(index * 4 + 0, sprite.byte0);
		self.store(index * 4 + 1, sprite.byte1);
		self.store(index * 4 + 2, sprite.byte2);
		self.store(index * 4 + 3, sprite.byte3);
	}

	fn reset(&mut self) {
		for i in 0..self.get_num() {
			self.store(i * 4 + 0, 0xff);
			self.store(i * 4 + 1, 0xff);
			self.store(i * 4 + 2, 0xff);
			self.store(i * 4 + 3, 0xff);
		}
	}
}

struct Sprite {
	byte0: u8,
	byte1: u8,
	byte2: u8,
	byte3: u8
}

impl Sprite {
	fn get_y(&self) -> u8 {
		self.byte0
	}

	fn get_x(&self) -> u8 {
		self.byte3
	}

	fn get_tile_index(&self) -> u8 {
		self.byte1
	}

	// the lowest two bits of byte2 holds the
	// 3-2 bits of palette memory address
	fn get_palette_num(&self)-> u8 {
		self.byte2 & 0x3
	}

	fn get_priority(&self) -> u8 {
		(self.byte2 >> 5) & 1
	}

	fn horizontal_flip(&self) -> bool {
		((self.byte2 >> 6) & 1) == 1
	}

	fn vertical_flip(&self) -> bool {
		((self.byte2 >> 7) & 1) == 1
	}

	fn on(&self, y: u8, height: u8) -> bool {
		(y >= self.get_y()) && (y < self.get_y() + height)
	}

	fn get_y_in_sprite(&self, y: u8, height: u8) -> u8 {
		// Assumes self.on(y, height) is true
		match self.vertical_flip() {
			true => height - 1 - (y - self.get_y()),
			false => y - self.get_y()
		}
	}
}
