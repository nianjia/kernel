#![feature(lang_items)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(alloc)]
#![feature(ptr_internals)]
#![no_std]

#[macro_use]
extern crate system;
extern crate spin;
extern crate selfalloc;
extern crate alloc;

#[macro_use]
mod vga_buffer;

use system::CAddr;

/// Decode a code in the PS/2 scan code set 1 (legacy set).
///
/// Difference between set 1 and sets 2 & 3:
///   http://wiki.osdev.org/%228042%22_PS/2_Controller#Translation
///
/// Reference table:
///   http://www.computer-engineering.org/ps2keyboard/scancodes1.html
pub fn from_scancode_printable(code: usize) -> Option<char> {
    let printable = match code {
        0x1e => 'a',
        0x30 => 'b',
        0x2e => 'c',
        0x20 => 'd',
        0x12 => 'e',
        0x21 => 'f',
        0x22 => 'g',
        0x23 => 'h',
        0x17 => 'i',
        0x24 => 'j',
        0x25 => 'k',
        0x26 => 'l',
        0x32 => 'm',
        0x31 => 'n',
        0x18 => 'o',
        0x19 => 'p',
        0x10 => 'q',
        0x13 => 'r',
        0x1f => 's',
        0x14 => 't',
        0x16 => 'u',
        0x2f => 'v',
        0x11 => 'w',
        0x2d => 'x',
        0x15 => 'y',
        0x2c => 'z',
        0x0b => '0',
        0x02 => '1',
        0x03 => '2',
        0x04 => '3',
        0x05 => '4',
        0x06 => '5',
        0x07 => '6',
        0x08 => '7',
        0x09 => '8',
        0x0a => '9',
        0x29 => '`',
        0x0c => '-',
        0x0d => '=',
        0x2b => '\\',
        0x39 => ' ',
        0x1a => '[',
        0x1b => ']',
        0x27 => ';',
        0x28 => '\'',
        0x33 => ',',
        0x34 => '.',
        0x35 => '/',
        0x37 => '*', // Keypad
        0x4a => '-', // Keypad
        0x4e => '+', // Keypad
        _ => return None,
    };

    Some(printable)
}

pub fn from_scancode(code: usize) -> Key {
    if code == 0x1C {
        Key::Enter
    } else {
        let printable = from_scancode_printable(code);
        if printable.is_some() {
            Key::Printable(printable.unwrap())
        } else {
            Key::Nonprintable
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
pub enum Key {
    Printable(char),
    Enter,
    Nonprintable
}

static mut IS_PARENT: bool = true;

#[lang="start"]
#[no_mangle]
#[allow(private_no_mangle_fns)]
fn start(_argc: isize, _argv: *const *const u8) {
    if unsafe { IS_PARENT } {
        unsafe { IS_PARENT = false; }
        parent_main();
    } else {
        child_main();
    }
    loop {};
}

fn parent_main() {
    unsafe { system::set_task_buffer_addr(0x90001000); }
    system_print!("parent rinit started.");

    // Test allocator
    unsafe { selfalloc::setup_allocator(CAddr::from(2), CAddr::from(3), 0x1000000000); }
    {
        use alloc::boxed::Box;
        let heap_test = Box::new(42);
        system_print!("testing heap: {:?}", heap_test);
    }

    system_print!("parent stack addr: 0x{:x}.",
                  system::task_buffer_addr() as usize);
    print!("Child entry should be at: 0x{:x} ({})\nChild stack pointer should be at: 0x{:x} ({})\n",
           start as *const () as usize, start as *const () as usize,
           0x70000000 + (0x1000 * 4 - 4), 0x70000000 + (0x1000 * 4 - 4));
    print!(">>> ");
    let mut lastkey = Key::Nonprintable;
    let mut command = [0u8; 32];
    let mut command_size = 0;
    loop {
        let key = from_scancode(system::channel_take_raw(CAddr::from(254)) as usize);
        if key == lastkey {
            continue;
        } else {
            lastkey = key.clone();
        }
        match key {
            Key::Printable(c) => {
                print!("{}", c);
                if command_size < 32 {
                    command[command_size] = c as u8;
                    command_size += 1;
                }
            }
            Key::Enter => {
                print!("\n");
                execute_command(::core::str::from_utf8(&command[0..command_size]).unwrap());
                command = [0u8; 32];
                command_size = 0;
            }
            _ => (),
        }
    }
}

fn start_child() {
    system::retype_task(CAddr::from(2), CAddr::from(249));
    system::task_set_stack_pointer(CAddr::from(249), 0x70000000 + (0x1000 * 4 - 4));
    system::task_set_instruction_pointer(CAddr::from(249), start as *const () as u64);
    system::task_set_cpool(CAddr::from(249), CAddr::from(0));
    system::task_set_top_page_table(CAddr::from(249), CAddr::from(3));
    system::task_set_buffer(CAddr::from(249), CAddr::from(250));
    system::task_set_active(CAddr::from(249));
}

fn child_main() {
    unsafe { system::set_task_buffer_addr(0x90003000); }

    system_print!("child rinit started.");
    system_print!("parent stack addr: 0x{:x}.",
                  system::task_buffer_addr() as usize);
    loop {
        let value: u64 = system::channel_take(CAddr::from(255));
        system_print!("Received from master: {:?}", value);
    }
}

fn parse_usize(s: &str, prefix: &str) -> Option<(usize, usize)> {
    if s.len() >= prefix.len() + 4 && &s[0..prefix.len()] == prefix {
        let st = &s[(prefix.len()+1)..s.len()];
        let mut split = st.split(' ');
        let o1: usize = split.next().unwrap().parse().unwrap();
        let o2: usize = split.next().unwrap().parse().unwrap();
        return Some((o1, o2));
    } else {
        return None;
    }
}

fn execute_command(s: &str) {
    if s == "list" {
        print!("Listing task cpool ...\n");
        system::debug_cpool_list();
    } else if s == "start child" {
        start_child();
        print!("Child started.\n");
    } else if s.len() >= 6 && &s[0..4] == "echo" {
        print!("{}\n", &s[5..s.len()]);
    } else if s.len() >= 6 && &s[0..8] == "send raw" {
        let value: u64 = (&s[9..s.len()]).parse().unwrap();
        system::channel_put(CAddr::from(255), value);
        print!("Sent raw to child through channel 255\n");
    } else if s.len() >= 6 && &s[0..8] == "send cap" {
        let value: u64 = (&s[9..s.len()]).parse().unwrap();
        system::channel_put_cap(CAddr::from(255), CAddr::from(value as u8));
        print!("Sent cap to child through channel 255\n");
    } else if let Some((source, target)) = parse_usize(s, "retype cpool") {
        system::retype_cpool(CAddr::from(source as u8), CAddr::from(target as u8));
        print!("Operation finished.\n");
    } else if let Some((source, target)) = parse_usize(s, "retype task") {
        system::retype_task(CAddr::from(source as u8), CAddr::from(target as u8));
        print!("Operation finished.\n");
    } else if let Some((target, ptr)) = parse_usize(s, "set stack") {
        system::task_set_stack_pointer(CAddr::from(target as u8), ptr as u64);
        print!("Operation finished.\n");
    } else if let Some((target, ptr)) = parse_usize(s, "set instruction") {
        system::task_set_instruction_pointer(CAddr::from(target as u8), ptr as u64);
        print!("Operation finished.\n");
    } else if let Some((target, cpool)) = parse_usize(s, "set cpool") {
        system::task_set_cpool(CAddr::from(target as u8), CAddr::from(cpool as u8));
        print!("Operation finished.\n");
    } else if let Some((target, table)) = parse_usize(s, "set table") {
        system::task_set_top_page_table(CAddr::from(target as u8), CAddr::from(table as u8));
        print!("Operation finished.\n");
    } else if let Some((target, buffer)) = parse_usize(s, "set buffer") {
        system::task_set_buffer(CAddr::from(target as u8), CAddr::from(buffer as u8));
        print!("Operation finished.\n");
    } else if let Some((target, status)) = parse_usize(s, "set active") {
        if status == 0 {
            system::task_set_inactive(CAddr::from(target as u8));
        } else {
            system::task_set_active(CAddr::from(target as u8));
        }
        print!("Operation finished.\n");
    } else {
        print!("Unknown command.\n");
    }
    print!(">>> ");
}

#[allow(dead_code)]
fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}
