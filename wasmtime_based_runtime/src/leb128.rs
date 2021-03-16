// Even though consts are used, the error of being unused is still received.
#![allow(unused)]
use anyhow::Result;

const MAGIC_NUMBER: &[u8; 4] = b"\0asm";
const SIZE_OF_DYLINK_STRING: &u8 = &6_u8;
const DYLINK_NAME: &str = "dylink";

#[derive(Debug)]
pub struct Dylink {
    pub mem_size: u32,
    pub mem_align: u32,
    pub table_size: u32,
    pub table_align: u32,
    pub needed_dynlibs: Vec<String>,
}

impl Dylink {
    // TODO: there should be a check to not exceed binary bounds.
    // Taken from a source file compiled with Emscripten...
    fn get_leb128(binary: &[u8], position: &mut usize) -> u32 {
        let mut ret: u32 = 0;
        let mut mul: u32 = 1;
        loop {
            *position += 1;
            let byte = binary[*position];
            ret += (((byte & 0x7f) as u32) * mul);
            mul *= 0x80;
            if (byte & 0x80) == 0 {
                break;
            }
        }

        return ret;
    }

    fn iter_section_name(binary: &[u8], position: &mut usize) {
        for c in DYLINK_NAME.chars() {
            *position += 1;
            assert_eq!(&binary[*position], &(c as u8));
        }
    }

    pub fn check_dylink(binary: &[u8]) -> Result<Self> {
        let mut needed_dynlibs: Vec<String> = Vec::new();

        assert_eq!(MAGIC_NUMBER, &binary[0..4]);
        assert_eq!(binary[8], 0, "dylink section needs to be first");

        let mut position: usize = 9;
        let _section_size = Dylink::get_leb128(binary, &mut position);

        assert_eq!(&binary[position], SIZE_OF_DYLINK_STRING);

        Dylink::iter_section_name(binary, &mut position);

        let mem_size = Dylink::get_leb128(binary, &mut position);
        let mem_align = Dylink::get_leb128(binary, &mut position);
        let table_size = Dylink::get_leb128(binary, &mut position);
        let table_align = Dylink::get_leb128(binary, &mut position);
        let needed_dynlibs_count = Dylink::get_leb128(binary, &mut position);

        for _ in 0..needed_dynlibs_count {
            let name_length = Dylink::get_leb128(binary, &mut position);
            position += 1;
            let name = &binary[position..position + name_length as usize];
            position += name_length as usize - 1;
            let name = std::str::from_utf8(&name)?;
            needed_dynlibs.push(name.to_string());
        }

        Ok(Self {
            mem_size,
            mem_align,
            table_size,
            table_align,
            needed_dynlibs,
        })
    }
}

// wasm-objdump result of "side.wasm";
// Section Details:
// Custom:
//  - name: "dylink"
//  - mem_size     : 5243072
//  - mem_p2align  : 4
//  - table_size   : 0
//  - table_p2align: 0

#[test]
fn check_with_no_dynamic_libraries() -> Result<()> {
    let dylink = Dylink::check_dylink(&[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x00, 0x0f, 0x06, 0x64, 0x79, 0x6c, 0x69,
        0x6e, 0x6b, 0xc0, 0x81, 0xc0, 0x02, 0x04, 0x00, 0x00, 0x00,
    ])?;

    assert_eq!(dylink.mem_size, 5243072);
    assert_eq!(dylink.mem_align, 4);
    assert_eq!(dylink.table_size, 0);
    assert_eq!(dylink.table_align, 0);

    Ok(())
}

// wasm-objdump result of "dynamic_linking_example2.wasm":
// Section Details:
// Custom:
//  - name: "dylink"
//  - mem_size     : 30
//  - mem_p2align  : 0
//  - table_size   : 0
//  - table_p2align: 0
//  - needed_dynlibs[1]:
//   - dynamic_linking_example_side.wasm

#[test]
fn check_with_dynamic_library() -> Result<()> {
    let dylink = Dylink::check_dylink(&[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2e, 0x06, 0x64, 0x79, 0x6c, 0x69,
        0x6e, 0x6b, 0x1e, 0x00, 0x00, 0x00, 0x01, 0x21, 0x64, 0x79, 0x6e, 0x61, 0x6d, 0x69, 0x63,
        0x5f, 0x6c, 0x69, 0x6e, 0x6b, 0x69, 0x6e, 0x67, 0x5f, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c,
        0x65, 0x5f, 0x73, 0x69, 0x64, 0x65, 0x2e, 0x77, 0x61, 0x73, 0x6d,
    ])?;

    assert_eq!(dylink.mem_size, 30);
    assert_eq!(dylink.mem_align, 0);
    assert_eq!(dylink.table_size, 0);
    assert_eq!(dylink.table_align, 0);
    assert_eq!(
        dylink.needed_dynlibs[0],
        "dynamic_linking_example_side.wasm"
    );

    Ok(())
}

// wasm-objdump result of "dynamic_linking_example_multiple_dependencies.wasm":
// Section Details:
// Custom:
//  - name: "dylink"
//  - mem_size     : 30
//  - mem_p2align  : 0
//  - table_size   : 0
//  - table_p2align: 0
//  - needed_dynlibs[4]:
//   - dynamic_linking_example_side.wasm
//   - dynamic_linking_example.wasm
//   - main.wasm
//   - bar.wasm

#[test]
fn check_with_dynamic_libraries() -> Result<()> {
    let dylink = Dylink::check_dylink(&[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2e, 0x06, 0x64, 0x79, 0x6c, 0x69,
        0x6e, 0x6b, 0x1e, 0x00, 0x00, 0x00, 0x04, 0x21, 0x64, 0x79, 0x6e, 0x61, 0x6d, 0x69, 0x63,
        0x5f, 0x6c, 0x69, 0x6e, 0x6b, 0x69, 0x6e, 0x67, 0x5f, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c,
        0x65, 0x5f, 0x73, 0x69, 0x64, 0x65, 0x2e, 0x77, 0x61, 0x73, 0x6d, 0x1c, 0x64, 0x79, 0x6e,
        0x61, 0x6d, 0x69, 0x63, 0x5f, 0x6c, 0x69, 0x6e, 0x6b, 0x69, 0x6e, 0x67, 0x5f, 0x65, 0x78,
        0x61, 0x6d, 0x70, 0x6c, 0x65, 0x2e, 0x77, 0x61, 0x73, 0x6d, 0x09, 0x6d, 0x61, 0x69, 0x6e,
        0x2e, 0x77, 0x61, 0x73, 0x6d, 0x08, 0x62, 0x61, 0x72, 0x2e, 0x77, 0x61, 0x73, 0x6d,
    ])?;

    assert_eq!(dylink.mem_size, 30);
    assert_eq!(dylink.mem_align, 0);
    assert_eq!(dylink.table_size, 0);
    assert_eq!(dylink.table_align, 0);
    assert_eq!(
        dylink.needed_dynlibs[0],
        "dynamic_linking_example_side.wasm"
    );
    assert_eq!(dylink.needed_dynlibs[1], "dynamic_linking_example.wasm");
    assert_eq!(dylink.needed_dynlibs[2], "main.wasm");
    assert_eq!(dylink.needed_dynlibs[3], "bar.wasm");

    Ok(())
}
