use std::env;
use std::fs;
use std::path::Path;

fn read_leb_u32(bytes: &[u8], idx: &mut usize) -> Option<u32> {
    let mut result: u32 = 0;
    let mut shift = 0;
    loop {
        if *idx >= bytes.len() {
            return None;
        }
        let byte = bytes[*idx];
        *idx += 1;
        result |= ((byte & 0x7f) as u32) << shift;
        if byte & 0x80 == 0 {
            return Some(result);
        }
        shift += 7;
        if shift > 35 {
            return None;
        }
    }
}

fn strip_custom_sections(input: &[u8], strip_names: &[&[u8]]) -> Option<Vec<u8>> {
    if input.len() < 8 || &input[0..4] != b"\0asm" {
        return None;
    }

    let mut out = Vec::with_capacity(input.len());
    out.extend_from_slice(&input[0..8]);

    let mut idx = 8;
    while idx < input.len() {
        let section_start = idx;
        let id = *input.get(idx)?;
        idx += 1;
        let size = read_leb_u32(input, &mut idx)? as usize;
        let payload_start = idx;
        let payload_end = payload_start.checked_add(size)?;
        if payload_end > input.len() {
            return None;
        }

        let mut keep = true;
        if id == 0 {
            let mut name_idx = payload_start;
            let name_len = read_leb_u32(input, &mut name_idx)? as usize;
            let name_end = name_idx.checked_add(name_len)?;
            if name_end <= payload_end {
                let name = &input[name_idx..name_end];
                if strip_names.iter().any(|candidate| *candidate == name) {
                    keep = false;
                }
            }
        }

        if keep {
            out.extend_from_slice(&input[section_start..payload_end]);
        }
        idx = payload_end;
    }

    Some(out)
}

fn main() {
    let mut args = env::args().skip(1);
    let input = match args.next() {
        Some(value) => value,
        None => {
            eprintln!("Usage: wasm_strip <input.wasm> [output.wasm]");
            std::process::exit(1);
        }
    };
    let output = args.next().unwrap_or_else(|| format!("{}.stripped.wasm", input));

    let bytes = fs::read(&input).unwrap_or_else(|err| {
        eprintln!("Failed to read {}: {}", input, err);
        std::process::exit(1);
    });

    let stripped = strip_custom_sections(&bytes, &[b"__wasm_bindgen_unstable", b"name", b"producers"])
        .unwrap_or_else(|| {
            eprintln!("Failed to parse wasm file: {}", input);
            std::process::exit(1);
        });

    let out_path = Path::new(&output);
    if let Err(err) = fs::write(out_path, stripped) {
        eprintln!("Failed to write {}: {}", out_path.display(), err);
        std::process::exit(1);
    }
}
