use std::borrow::Cow;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::result::Result as ResultOf;

use structopt::StructOpt;

type Error = Cow<'static, str>;
type Result<T = (), E = Error> = ResultOf<T, E>;

#[derive(Debug, StructOpt)]
#[structopt(name = "bfc")]
struct Args {
    /// Input .bf file
    #[structopt(parse(from_os_str))]
    file: PathBuf,
    /// Output .asm file, default <file> with file extension changed
    #[structopt(short, long)]
    out: Option<PathBuf>,
    /// The tape size to allocate in the output program
    #[structopt(long, default_value = "1048576")]
    tape_size: u64,
}

mod code;
use code::Code;

fn main() -> Result {
    let args = Args::from_args();
    let code = read_code(&args.file)?;
    let out_file = args
        .out
        .as_ref()
        .map_or_else(|| Cow::Owned(change_ext(&args.file, "asm")), Cow::Borrowed);
    compile(code.iter().cloned(), out_file.as_ref(), args.tape_size)
        .map_err(|err| format!("Error compiling to {}: {}", out_file.display(), err))?;

    println!("Done! Output has been written to {}.", out_file.display());
    println!("You can compile it by running the following commands:");
    let obj_file = change_ext(&out_file, "o");
    println!(
        "  nasm -f elf64 -o {} {}",
        change_ext(&out_file, "o").display(),
        out_file.display()
    );
    println!(
        "  ld -o {} {}",
        change_ext(&out_file, "exe").display(),
        obj_file.display()
    );

    Ok(())
}

fn change_ext(path: &PathBuf, ext: &str) -> PathBuf {
    let mut clone = path.clone();
    clone.set_extension(ext);
    clone
}

fn read_code(file: &PathBuf) -> Result<Vec<Code>> {
    use std::convert::TryFrom;
    use std::io::Read;

    let mut vec = vec![];
    for byte in fs::File::open(file)
        .map_err(|err| format!("Cannot open {}: {}", file.display(), err))?
        .bytes()
    {
        let byte = byte.map_err(|err| format!("Cannot read from {}: {}", file.display(), err))?;
        if let Ok(code) = Code::try_from(char::from(byte)) {
            vec.push(code);
        }
    }
    Ok(vec)
}

fn compile<I, P>(codes: I, out_file: &P, tape_size: u64) -> io::Result<()>
where
    I: IntoIterator<Item = Code>,
    P: AsRef<Path>,
{
    use std::io::Write;

    let mut out = fs::File::create(out_file)?;

    writeln!(out, "section .bss")?;
    writeln!(out, "  tape_ptr RESQ 1")?;
    writeln!(out, "  tape RESB {}", tape_size)?;

    writeln!(out, "section .text")?;
    writeln!(out, "  global _start")?;
    writeln!(out, "_start:")?;
    writeln!(out, "  mov EAX, tape+{}", tape_size / 2)?;

    let mut loop_open = 0usize;
    let mut loop_close = 0usize;
    for code in codes {
        match code {
            Code::MemInc => writeln!(out, "  inc BYTE [EAX]")?,
            Code::MemDec => writeln!(out, "  dec BYTE [EAX]")?,
            Code::PtrInc => writeln!(out, "  inc EAX")?,
            Code::PtrDec => writeln!(out, "  dec EAX")?,
            Code::SysWrite => {
                writeln!(out, "  mov tape_ptr, eax")?;
                writeln!(out, "  mov eax, [tape_ptr]")?;
                writeln!(out, "  mov ebx [tape_ptr+4]")?;
                writeln!(out, "  mov ecx, [tape_ptr+8]")?;
                writeln!(out, "  mov edx, [tape_ptr+12]")?;
                writeln!(out, "  mov esi, [tape_ptr+16]")?;
                writeln!(out, "  mov edi, [tape_ptr+20]")?;
                writeln!(out, "  int 0x80")?;
                writeln!(out, "  mov [tape_ptr], eax")?;
                writeln!(out, "  mov eax, tape_ptr")?;
            }
            Code::SysRead => {
                writeln!(out, "  mov [eax], [[eax]]")?;
            }
            Code::LoopStart => {
                loop_open += 1;
                writeln!(out, "label_{}:", loop_open)?;
            }
            Code::LoopEnd => {
                loop_close += 1;
                if loop_close > loop_open {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Compile error: Found a `]` code without a matching `[`",
                    ))?;
                }
                writeln!(out, "  jne label_{}", loop_close)?;
            }
        }
    }

    if loop_open > loop_close {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Compile error: Reached end of file with {} `[` code(s) unclosed",
        ))?;
    }

    Ok(())
}
