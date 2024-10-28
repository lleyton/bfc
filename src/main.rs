use qbe::{Function, Instr, Linkage, Module, Type, Value};
use std::{env, fs, io, process, str::Chars};

#[derive(Debug)]
enum Operation {
    Increment,
    Decrement,
    MoveRight,
    MoveLeft,
    Write,
    Read,
    Loop(Program),
}

#[derive(Debug)]
struct Program(Vec<Operation>);

impl Program {
    fn parse(chars: &mut Chars<'_>) -> Program {
        let mut program = vec![];

        while let Some(c) = chars.next() {
            match c {
                '+' => program.push(Operation::Increment),
                '-' => program.push(Operation::Decrement),
                '>' => program.push(Operation::MoveRight),
                '<' => program.push(Operation::MoveLeft),
                '.' => program.push(Operation::Write),
                ',' => program.push(Operation::Read),
                '[' => program.push(Operation::Loop(Self::parse(chars))),
                ']' => break,
                _ => {}
            };
        }

        Program(program)
    }

    fn emit_part(&self, f: &mut Function, block_count: &mut u32) {
        for op in &self.0 {
            match op {
                Operation::Increment => {
                    f.assign_instr(
                        Value::Temporary("value".into()),
                        Type::Byte,
                        Instr::Load(Type::UnsignedByte, Value::Temporary("pointer".into())),
                    );
                    f.assign_instr(
                        Value::Temporary("value".into()),
                        Type::Byte,
                        Instr::Add(Value::Temporary("value".into()), Value::Const(1)),
                    );
                    f.add_instr(Instr::Store(
                        Type::Byte,
                        Value::Temporary("pointer".into()),
                        Value::Temporary("value".into()),
                    ));
                }
                Operation::Decrement => {
                    f.assign_instr(
                        Value::Temporary("value".into()),
                        Type::Byte,
                        Instr::Load(Type::UnsignedByte, Value::Temporary("pointer".into())),
                    );
                    f.assign_instr(
                        Value::Temporary("value".into()),
                        Type::Byte,
                        Instr::Sub(Value::Temporary("value".into()), Value::Const(1)),
                    );
                    f.add_instr(Instr::Store(
                        Type::Byte,
                        Value::Temporary("pointer".into()),
                        Value::Temporary("value".into()),
                    ));
                }

                Operation::MoveRight => f.assign_instr(
                    Value::Temporary("pointer".into()),
                    Type::Long,
                    Instr::Add(Value::Temporary("pointer".into()), Value::Const(1)),
                ),
                Operation::MoveLeft => f.assign_instr(
                    Value::Temporary("pointer".into()),
                    Type::Long,
                    Instr::Sub(Value::Temporary("pointer".into()), Value::Const(1)),
                ),
                Operation::Write => {
                    f.assign_instr(
                        Value::Temporary("value".into()),
                        Type::Byte,
                        Instr::Load(Type::UnsignedByte, Value::Temporary("pointer".into())),
                    );
                    f.add_instr(Instr::Call(
                        "putchar".into(),
                        vec![(Type::UnsignedByte, Value::Temporary("value".into()))],
                    ));
                }
                Operation::Read => {
                    f.assign_instr(
                        Value::Temporary("value".into()),
                        Type::Byte,
                        Instr::Call("getchar".into(), vec![]),
                    );
                    f.add_instr(Instr::Store(
                        Type::Byte,
                        Value::Temporary("pointer".into()),
                        Value::Temporary("value".into()),
                    ));
                }
                Operation::Loop(ops) => {
                    let body_label = format!("body{block_count}");
                    let end_label = format!("end{block_count}");
                    *block_count = *block_count + 1;

                    f.assign_instr(
                        Value::Temporary("value".into()),
                        Type::Byte,
                        Instr::Load(Type::UnsignedByte, Value::Temporary("pointer".into())),
                    );
                    f.add_instr(Instr::Jnz(
                        Value::Temporary("value".into()),
                        body_label.clone(),
                        end_label.clone(),
                    ));
                    f.add_block(body_label.clone());
                    ops.emit_part(f, block_count);
                    f.assign_instr(
                        Value::Temporary("value".into()),
                        Type::Byte,
                        Instr::Load(Type::UnsignedByte, Value::Temporary("pointer".into())),
                    );
                    f.add_instr(Instr::Jnz(
                        Value::Temporary("value".into()),
                        body_label.clone(),
                        end_label.clone(),
                    ));
                    f.add_block(end_label);
                }
            }
        }
    }

    fn emit(&self, f: &mut Function) {
        f.add_block("start");
        f.assign_instr(
            Value::Temporary("pointer".into()),
            Type::Long,
            Instr::Alloc4(30_000),
        );

        let mut block_count = 0;
        self.emit_part(f, &mut block_count);

        f.add_instr(Instr::Ret(None));
    }
}

fn main() -> io::Result<()> {
    let Some(file) = env::args().nth(1) else {
        eprintln!("bfc: usage: source.b");
        process::exit(1);
    };
    let file = fs::read_to_string(file)?;
    let program = Program::parse(&mut file.chars());

    let mut module = Module::new();
    let mut main = Function::new(Linkage::public(), "main", vec![], None);
    program.emit(&mut main);
    module.add_function(main);

    print!("{}", module);

    Ok(())
}
