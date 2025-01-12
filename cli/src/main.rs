extern crate getopts;
extern crate riscv_emu_rust;

mod popup_terminal;
mod dummy_terminal;

use riscv_emu_rust::Emulator;
use riscv_emu_rust::cpu::Xlen;
use riscv_emu_rust::terminal::Terminal;
use popup_terminal::PopupTerminal;
use dummy_terminal::DummyTerminal;

use std::env;
use std::fs::File;
use std::io::Read;

use getopts::Options;

#[warn(unused_imports)]
use riscv_emu_rust::cpu;
use riscv_emu_rust::mmu;

enum TerminalType {
	PopupTerminal,
	DummyTerminal
}

fn print_usage(program: &str, opts: Options) {
	let usage = format!("Usage: {} program_file [options]", program);
	print!("{}", opts.usage(&usage));
}

fn get_terminal(terminal_type: TerminalType) -> Box<dyn Terminal> {
	match terminal_type {
		TerminalType::PopupTerminal => Box::new(PopupTerminal::new()),
		TerminalType::DummyTerminal => Box::new(DummyTerminal::new()),
	}
}

fn main () -> std::io::Result<()> {
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();

	let mut opts = Options::new();
	opts.optopt("x", "xlen", "Set bit mode. Default is auto detect from elf file", "32|64");
	opts.optopt("f", "fs", "File system image file", "xv6/fs.img");
	opts.optopt("d", "dtb", "Device tree file", "linux/dtb");
	opts.optflag("n", "no_terminal", "No popup terminal");
	opts.optflag("h", "help", "Show this help menu");
	opts.optflag("p", "page_cache", "Enable experimental page cache optimization");

	let matches = match opts.parse(&args[1..]) {
		Ok(m) => m,
		Err(f) => {
			println!("{}", f.to_string());
			print_usage(&program, opts);
			// @TODO: throw error?
			return Ok(());
		}
	};

	if matches.opt_present("h") {
		print_usage(&program, opts);
		return Ok(());
	}

	if args.len() < 2 {
		print_usage(&program, opts);
		// @TODO: throw error?
		return Ok(());
	}

	let fs_contents = match matches.opt_str("f") {
		Some(path) => {
			let mut file = File::open(path)?;
			let mut contents = vec![];
			file.read_to_end(&mut contents)?;
			contents
		}
		None => vec![]
	};

	let mut has_dtb = false;
	let dtb_contents = match matches.opt_str("d") {
		Some(path) => {
			has_dtb = true;
			let mut file = File::open(path)?;
			let mut contents = vec![];
			file.read_to_end(&mut contents)?;
			contents
		}
		None => vec![]
	};

	let elf_filename = args[1].clone();
	let mut elf_file = File::open(elf_filename)?;
	let mut elf_contents = vec![];
	elf_file.read_to_end(&mut elf_contents)?;

	let terminal_type = match matches.opt_present("n") {
		true => {
			println!("No popup terminal mode. Output will be flushed on your terminal but you can not input.");
			TerminalType::DummyTerminal
		},
		false => TerminalType::PopupTerminal
	};

	let mut emulator = Emulator::new(get_terminal(terminal_type));
	emulator.setup_program(elf_contents);
	
	match matches.opt_str("x") {
		Some(x) => match x.as_str() {
			"32" => {
				println!("Force to 32-bit mode.");
				emulator.update_xlen(Xlen::Bit32);
			},
			"64" => {
				println!("Force to 64-bit mode.");
				emulator.update_xlen(Xlen::Bit64);
			},
			_ => {
				print_usage(&program, opts);
				// @TODO: throw error?
				return Ok(());
			}
		},
		None => {}
	};

	emulator.setup_filesystem(fs_contents);
	if has_dtb {
		emulator.setup_dtb(dtb_contents);
	}
	if matches.opt_present("h") {
		emulator.enable_page_cache(true);
	}
	emulator.run();
	Ok(())
}


use std::io;

#[test]
pub fn shstack_test() {
	let mut cpu = cpu::Cpu::new(get_terminal(TerminalType::DummyTerminal));
	cpu.get_mut_mmu().init_memory(0x1000);
	cpu.update_pc(mmu::DRAM_BASE);

	// init registers
	//cpu.x[1] = 0x1234i64;
	cpu.x[2] = (mmu::DRAM_BASE + 0x800) as i64; //stack pointer
	cpu.x[8] = (mmu::DRAM_BASE + 0x800) as i64; //base pointer

	/*
	// simple program that returns 3
	let v: Vec<u16> = vec![
		0x450d, // li a0,3
		0x8082,   // ret
	];
	*/

	// program that causes BOF
	let v = vec![
		// jal ra, 0x1004 <main+0x0> (jump to pc+4byte) 最初にcall mainの疑似命令を行う
		0xef,0x00, 0x40, 0x00,
		// main関数の先頭から
		0x01,0x11,0x06,0xEC,0x22,0xE8,0x00,0x10,0x93,0x07,0x10,0x04,0x23,0x04,0xF4,0xFE,0x93,0x07,0x10,0x04,0xA3,0x04,0xF4,0xFE,0x93,0x07,0x10,0x04,0x23,0x05,0xF4,0xFE,0x93,0x07,0x10,0x04,0xA3,0x05,0xF4,0xFE,0x93,0x07,0x10,0x04,0x23,0x06,0xF4,0xFE,0x93,0x07,0x10,0x04,0xA3,0x06,0xF4,0xFE,0x93,0x07,0x10,0x04,0x23,0x07,0xF4,0xFE,0x93,0x07,0x10,0x04,0xA3,0x07,0xF4,0xFE,0x93,0x07,0x10,0x04,0x23,0x08,0xF4,0xFE,0x93,0x07,0x10,0x04,0xA3,0x08,0xF4,0xFE,0x93,0x07,0x10,0x04,0x23,0x09,0xF4,0xFE,0x93,0x07,0x10,0x04,0xA3,0x09,0xF4,0xFE,0x93,0x07,0x10,0x04,0x23,0x0A,0xF4,0xFE,0x93,0x07,0x10,0x04,0xA3,0x0A,0xF4,0xFE,0x93,0x07,0x10,0x04,0x23,0x0B,0xF4,0xFE,0x93,0x07,0x10,0x04,0xA3,0x0B,0xF4,0xFE,0x93,0x07,0x10,0x04,0x23,0x0C,0xF4,0xFE,0x93,0x07,0x10,0x04,0xA3,0x0C,0xF4,0xFE,0x93,0x07,0x10,0x04,0x23,0x0D,0xF4,0xFE,0x93,0x07,0x10,0x04,0xA3,0x0D,0xF4,0xFE,0x93,0x07,0x10,0x04,0x23,0x0E,0xF4,0xFE,0x93,0x07,0x10,0x04,0xA3,0x0E,0xF4,0xFE,0x93,0x07,0x10,0x04,0x23,0x0F,0xF4,0xFE,0x93,0x07,0x10,0x04,0xA3,0x0F,0xF4,0xFE,0x93,0x07,0x10,0x04,0x23,0x00,0xF4,0x00,0x93,0x07,0x10,0x04,0xA3,0x00,0xF4,0x00,0x93,0x07,0x10,0x04,0x23,0x01,0xF4,0x00,0x93,0x07,0x10,0x04,0xA3,0x01,0xF4,0x00,0x93,0x07,0x10,0x04,0x23,0x02,0xF4,0x00,0x93,0x07,0x10,0x04,0xA3,0x02,0xF4,0x00,0x93,0x07,0x10,0x04,0x23,0x03,0xF4,0x00,0x93,0x07,0x10,0x04,0xA3,0x03,0xF4,0x00,0x93,0x07,0x10,0x04,0x23,0x04,0xF4,0x00,0x93,0x07,0x10,0x04,0xA3,0x04,0xF4,0x00,0x93,0x07,0x10,0x04,0x23,0x05,0xF4,0x00,0x93,0x07,0x10,0x04,0xA3,0x05,0xF4,0x00,0x93,0x07,0x10,0x04,0x23,0x06,0xF4,0x00,0x93,0x07,0x10,0x04,0xA3,0x06,0xF4,0x00,0x93,0x07,0x10,0x04,0x23,0x07,0xF4,0x00,0x93,0x07,0x10,0x04,0xA3,0x07,0xF4,0x00,0x93,0x07,0x10,0x04,0x23,0x08,0xF4,0x00,0x93,0x07,0x10,0x04,0xA3,0x08,0xF4,0x00,0x93,0x07,0x10,0x04,0x23,0x09,0xF4,0x00,0x93,0x07,0x10,0x04,0xA3,0x09,0xF4,0x00,0x93,0x07,0x10,0x04,0x23,0x0A,0xF4,0x00,0x93,0x07,0x10,0x04,0xA3,0x0A,0xF4,0x00,0x93,0x07,0x10,0x04,0x23,0x0B,0xF4,0x00,0x93,0x07,0x10,0x04,0xA3,0x0B,0xF4,0x00,0x93,0x07,0x10,0x04,0x23,0x0C,0xF4,0x00,0x93,0x07,0x10,0x04,0xA3,0x0C,0xF4,0x00,0x93,0x07,0x10,0x04,0x23,0x0D,0xF4,0x00,0x93,0x07,0x10,0x04,0xA3,0x0D,0xF4,0x00,0x93,0x07,0x10,0x04,0x23,0x0E,0xF4,0x00,0x93,0x07,0x10,0x04,0xA3,0x0E,0xF4,0x00,0x93,0x07,0x10,0x04,0x23,0x0F,0xF4,0x00,0x93,0x07,0x10,0x04,0xA3,0x0F,0xF4,0x00,0x93,0x07,0x10,0x04,0x23,0x00,0xF4,0x02,0x93,0x07,0x10,0x04,0xA3,0x00,0xF4,0x02,0x93,0x07,0x10,0x04,0x23,0x01,0xF4,0x02,0x93,0x07,0x10,0x04,0xA3,0x01,0xF4,0x02,0x93,0x07,0x10,0x04,0x23,0x02,0xF4,0x02,0x93,0x07,0x10,0x04,0xA3,0x02,0xF4,0x02,0x93,0x07,0x10,0x04,0x23,0x03,0xF4,0x02,0x93,0x07,0x10,0x04,0xA3,0x03,0xF4,0x02,0x93,0x07,0x10,0x04,0x23,0x04,0xF4,0x02,0x93,0x07,0x10,0x04,0xA3,0x04,0xF4,0x02,0x93,0x07,0x10,0x04,0x23,0x05,0xF4,0x02,0x93,0x07,0x10,0x04,0xA3,0x05,0xF4,0x02,0x93,0x07,0x10,0x04,0x23,0x06,0xF4,0x02,0x93,0x07,0x10,0x04,0xA3,0x06,0xF4,0x02,0x93,0x07,0x10,0x04,0x23,0x07,0xF4,0x02,0x93,0x07,0x10,0x04,0xA3,0x07,0xF4,0x02,0x93,0x07,0x10,0x04,0x23,0x08,0xF4,0x02,0x93,0x07,0x10,0x04,0xA3,0x08,0xF4,0x02,0x93,0x07,0x10,0x04,0x23,0x09,0xF4,0x02,0x93,0x07,0x10,0x04,0xA3,0x09,0xF4,0x02,0x93,0x07,0x10,0x04,0x23,0x0A,0xF4,0x02,0x93,0x07,0x10,0x04,0xA3,0x0A,0xF4,0x02,0x93,0x07,0x10,0x04,0x23,0x0B,0xF4,0x02,0x93,0x07,0x10,0x04,0xA3,0x0B,0xF4,0x02,0xEF,0x00,0x00,0x01,0x81,0x47,0x3E,0x85,0xE2,0x60,0x42,0x64,0x05,0x61,0x82,0x80,0x41,0x11,0x22,0xE4,0x00,0x08,0x81,0x47,0x3E,0x85,0x22,0x64,0x41,0x01,0x82,0x80,
		];
	
	
	let mut ofs = 0;
	for by in v {
		match cpu.get_mut_mmu().store_bytes(mmu::DRAM_BASE + ofs, by, 1) {
			Ok(()) => {},
			Err(_e) => panic!("Failed to store")
		};
		ofs += 1;
	}

	// start
	println!("* program start *");
	dump_reg(&cpu);
	
	let mut _cnt = 0;
	loop {
		//et mut s = String::new();
		//io::stdin().read_line(&mut s);
		cpu.tick();
		//dump_reg(&cpu);
		//dump_mem(&mut cpu, mmu::DRAM_BASE + 0x780, 25);

		// break when returning from main
		if cpu.pc == mmu::DRAM_BASE + 0x296 + 4 {
			cpu.tick();
			break;
		}
		_cnt += 1;
	}
	println!("* program end *");
	
	dump_reg(&cpu);
	
	//dump_mem(&mut cpu, mmu::DRAM_BASE + 0x780, 40);

}

pub fn dump_mem(cpu: &mut cpu::Cpu, begin: u64, byte: u64) {
	println!("----- memory -----");
	for r in 1..=byte {
		print!("0x{:X} | ", begin + (r as u64)*8);
		for i in 0..=7 {
			let t = cpu.mmu.load_bytes(begin + (r as u64)*8 + i, 1).unwrap_or_default();
			print!("{:02X} ", t);
		}
		println!("");
	}
	println!("-----------------");
}

pub fn dump_reg(cpu: &cpu::Cpu) {
	println!("----- registers  -----");
	println!("pc: 0x{:08X}", cpu.pc);
	println!("ra: 0x{:08X}", cpu.x[1]);
	println!("sp: 0x{:08X}", cpu.x[2]);
	println!("fp: 0x{:08X}", cpu.x[8]);
	println!("----------------------");
}
