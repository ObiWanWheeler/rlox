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
        
        f.write(f'pub trait Visitor<R, E> {{\n')

        f.write(f'  fn visit_{base.lower()}(&mut self, {base.lower()}: &{base}) -> Result<R, E>;\n')
        f.write("}\n\n")


if __name__ == "__main__":
    expr_types = [
        "Assign: name Token, value Box<Expr>",
        "Binary: left Box<Expr>, right Box<Expr>, operator Token",
        "Call: callee Box<Expr>, paren Token, arguments Vec<Box<Expr>>",
        "Get: object Box<Expr>, name Token",
        "Set: object Box<Expr>, name Token, value Box<Expr>",
        "Grouping: expression Box<Expr>",
        "Literal: value LiteralType",
        "Logical: left Box<Expr>, operator Token, right Box<Expr>"
        "Unary: operator Token, right Box<Expr>",
        "Variable: name Token"
    ]
    #define_ast("src/expr_new.rs", "Expr", expr_types)

    stmt_types = [
            "Block: statements Vec<Box<Stmt>>",
            "Expression: expression Expr",
            "If: condition Expr, then_branch Box<Stmt>, else_branch Option<Box<Stmt>>",
            "While: condition Expr, then_branch Box<Stmt>, finally_branch Option<Box<Stmt>>",
            "Print: expression Expr",
            "Break: token Token",
            "Return: token Token, return_value Option<Expr>",
            "Var: name Token, initializer Option<Expr>",
            "Class: name Token, methods Vec<Box<Stmt>>"
            "Function: name Token, parameters Vec<Token>, body Vec<Box<Stmt>>",
            ]    
    define_ast("src/stmt_new.rs", "Stmt", stmt_types)

