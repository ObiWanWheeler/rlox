import sys
from typing import List

# types like
# TypeName: field Type, field Type, ...


def define_ast(output_file, base, types: List[str]):
    with open(output_file, 'w') as f:
        f.write(f'pub enum {base} {{\n')

        for t in types:
            spl = t.split(':')
            name = spl[0].strip()
            f.write(f'{name} {{\n')

            fields = spl[1].strip().split(',')
            for field in fields:
                spl = field.strip().split(' ', maxsplit=1)
                field_name = spl[0]
                type = spl[1]

                f.write(f'    {field_name.strip()}: {type.strip()},\n')
            f.write('},\n\n')
        f.write("}\n\n")

        f.write(f'pub trait Visitor<R> {{\n')

        f.write('  fn visit_expr(&mut self, expr: &Expr) -> R;\n')
        f.write("}\n\n")


if __name__ == "__main__":
    output_file = sys.argv[1]
    types = [
        "Binary: left Box<Expr>, right Box<Expr>, operator Token",
        "Grouping: expression Box<Expr>",
        "Literal: value String",
        "Unary: operator Token, right Box<Expr>",
    ]
    define_ast(output_file, "Expr", types)
